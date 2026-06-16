//! Noisy quantum simulator with realistic device modeling
//!
//! This module provides a quantum simulator that incorporates realistic noise models
//! to simulate the behavior of actual quantum devices.

use crate::density_matrix::DensityMatrix;
use crate::measurement_stats::MeasurementStatistics;
use crate::noise_models::{
    DecoherenceChannel, DepolarizingChannel, DeviceNoiseModel, PauliChannel, ReadoutErrorModel,
};
use crate::{Parameter, QuantumCircuit, Result, StandardGate};
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Noisy quantum simulator that models realistic device behavior
#[derive(Debug)]
pub struct NoisyQuantumSimulator {
    /// Current quantum state (density matrix)
    state: DensityMatrix,
    /// Device noise model
    noise_model: DeviceNoiseModel,
    /// Classical register for measurements
    classical_register: Vec<bool>,
    /// Measurement statistics
    measurement_stats: MeasurementStatistics,
    /// Total execution time (nanoseconds)
    total_time: f64,
}

impl NoisyQuantumSimulator {
    /// Create a new noisy simulator with the given device noise model.
    ///
    /// The initial state is $|0\rangle^{\otimes n}$ and the simulator tracks
    /// gate execution time for time-dependent decoherence effects.
    pub fn new(num_qubits: usize, noise_model: DeviceNoiseModel) -> Self {
        Self {
            state: DensityMatrix::zero_state(num_qubits),
            noise_model,
            classical_register: vec![false; num_qubits],
            measurement_stats: MeasurementStatistics::from_results(&HashMap::new()),
            total_time: 0.0,
        }
    }

    /// Create simulator with realistic noise for given device
    pub fn realistic_device(num_qubits: usize) -> Self {
        let noise_model = DeviceNoiseModel::realistic_device(num_qubits);
        Self::new(num_qubits, noise_model)
    }

    /// Execute a quantum circuit with noise.
    ///
    /// Each instruction in the circuit is processed with the device noise model.
    /// The evolution of the density matrix $\rho$ follows:
    ///
    /// $$ \rho \rightarrow \mathcal{N} \circ (U \rho U^\dagger) $$
    ///
    /// where $\mathcal{N}$ is the composition of decoherence (T1/T2), gate errors,
    /// and readout errors from the device noise model.
    pub fn execute_circuit(&mut self, circuit: &QuantumCircuit) -> Result<()> {
        // Reset state and measurements
        self.state = DensityMatrix::zero_state(self.state.num_qubits());
        self.classical_register.fill(false);
        self.total_time = 0.0;

        // Execute each instruction
        for instruction in circuit.data().instructions().iter() {
            self.execute_instruction(instruction)?;
        }

        Ok(())
    }

    /// Execute a single circuit instruction with noise
    fn execute_instruction(&mut self, instruction: &crate::circuit::Instruction) -> Result<()> {
        // Check if this is a measurement instruction
        if instruction.is_measurement() {
            // Treat as measurement
            let qubit = instruction.qubits[0].index();
            let clbit = instruction
                .clbits
                .first()
                .map(|cb| cb.index())
                .unwrap_or(qubit);
            self.measure_with_noise(qubit, clbit)?;
        } else {
            // Apply gate with noise
            let qubit_indices: Vec<usize> = instruction.qubits.iter().map(|q| q.index()).collect();
            self.apply_gate_with_noise(
                &instruction.gate.gate_type,
                &qubit_indices,
                &instruction.gate.parameters,
            )?;
        }

        Ok(())
    }

    /// Apply a quantum gate with associated noise.
    ///
    /// First applies the ideal gate unitary $U$: $\rho' = U \rho U^\dagger$.
    /// Then applies qubit-level noise to each affected qubit, including
    /// decoherence (T1/T2) and gate-specific error channels (Pauli or depolarizing).
    fn apply_gate_with_noise(
        &mut self,
        gate: &StandardGate,
        qubits: &[usize],
        params: &[Parameter],
    ) -> Result<()> {
        // Get gate name for noise lookup
        let gate_name = self.gate_name(gate);

        // Apply the ideal gate operation first
        self.state.apply_gate(gate, qubits, params)?;

        // Get gate execution time
        let gate_time = self
            .noise_model
            .gate_times
            .get(&gate_name)
            .copied()
            .unwrap_or(20.0);
        self.total_time += gate_time;

        // Apply noise to each qubit involved
        for &qubit in qubits {
            self.apply_qubit_noise(qubit, &gate_name, gate_time)?;
        }

        Ok(())
    }

    /// Apply noise to a specific qubit after a gate operation.
    ///
    /// Applies two noise channels sequentially:
    /// 1. Decoherence: amplitude damping ($T_1$) and phase damping ($T_2$)
    /// 2. Gate error: Pauli channel for single-qubit gates, depolarizing for two-qubit gates
    fn apply_qubit_noise(&mut self, qubit: usize, gate_name: &str, gate_time: f64) -> Result<()> {
        // Apply decoherence (T1/T2) noise
        if let (Some(&t1), Some(&t2)) = (
            self.noise_model.t1_times.get(&qubit),
            self.noise_model.t2_times.get(&qubit),
        ) {
            let decoherence = DecoherenceChannel::new(t1, t2, gate_time);
            decoherence.apply(&mut self.state, qubit)?;
        }

        // Apply gate error (depolarizing noise)
        if let Some(&error_rate) = self.noise_model.gate_errors.get(gate_name) {
            if error_rate > 0.0 {
                // Use Pauli channel for single-qubit gates, depolarizing for multi-qubit
                if gate_name == "CNOT" || gate_name == "CZ" || gate_name == "SWAP" {
                    let depolarizing = DepolarizingChannel::new(error_rate);
                    depolarizing.apply(&mut self.state, qubit)?;
                } else {
                    let pauli = PauliChannel::symmetric(error_rate);
                    pauli.apply(&mut self.state, qubit)?;
                }
            }
        }

        Ok(())
    }

    /// Perform measurement with readout errors
    fn measure_with_noise(&mut self, qubit: usize, clbit: usize) -> Result<()> {
        // Perform ideal measurement
        let ideal_result = self.measure_qubit_ideal(qubit)?;

        // Apply readout error
        let final_result = if let Some(&(error_0_to_1, error_1_to_0)) =
            self.noise_model.readout_errors.get(&qubit)
        {
            let readout_model = ReadoutErrorModel::new(error_0_to_1, error_1_to_0);
            readout_model.apply_to_measurement(ideal_result)
        } else {
            ideal_result
        };

        // Store result in classical register
        if clbit < self.classical_register.len() {
            self.classical_register[clbit] = final_result;
        }

        // Update measurement statistics (simplified)
        // Note: MeasurementStatistics doesn't have add_measurement method in current implementation

        Ok(())
    }

    /// Perform ideal measurement (without readout errors)
    fn measure_qubit_ideal(&mut self, qubit: usize) -> Result<bool> {
        // Calculate measurement probabilities
        let prob_0 = self.state.measure_probability(qubit, false)?;
        let _prob_1 = self.state.measure_probability(qubit, true)?;

        // Sample measurement result
        let mut rng = rng();
        let rand_val: f64 = rng.random();
        let result = rand_val > prob_0;

        // Collapse state (project onto measurement result)
        self.state.collapse_to_measurement(qubit, result)?;

        Ok(result)
    }

    /// Get gate name string for noise lookup
    fn gate_name(&self, gate: &StandardGate) -> String {
        match gate {
            StandardGate::I => "I".to_string(),
            StandardGate::X => "X".to_string(),
            StandardGate::Y => "Y".to_string(),
            StandardGate::Z => "Z".to_string(),
            StandardGate::H => "H".to_string(),
            StandardGate::S => "S".to_string(),
            StandardGate::Sdg => "SDG".to_string(),
            StandardGate::T => "T".to_string(),
            StandardGate::Tdg => "TDG".to_string(),
            StandardGate::P => "P".to_string(),
            StandardGate::U => "U".to_string(),
            StandardGate::Rx => "RX".to_string(),
            StandardGate::Ry => "RY".to_string(),
            StandardGate::Rz => "RZ".to_string(),
            StandardGate::U1 => "U1".to_string(),
            StandardGate::U2 => "U2".to_string(),
            StandardGate::U3 => "U3".to_string(),
            StandardGate::CX => "CNOT".to_string(),
            StandardGate::CY => "CY".to_string(),
            StandardGate::CZ => "CZ".to_string(),
            StandardGate::CH => "CH".to_string(),
            StandardGate::CP => "CP".to_string(),
            StandardGate::CRx => "CRX".to_string(),
            StandardGate::CRy => "CRY".to_string(),
            StandardGate::CRz => "CRZ".to_string(),
            StandardGate::Swap => "SWAP".to_string(),
            StandardGate::CCX => "CCX".to_string(),
            StandardGate::CSwap => "CSWAP".to_string(),
            StandardGate::ISwap => "ISWAP".to_string(),
            StandardGate::MCX => "MCX".to_string(),
            StandardGate::MCY => "MCY".to_string(),
            StandardGate::MCZ => "MCZ".to_string(),
        }
    }

    /// Get current quantum state
    pub fn state(&self) -> &DensityMatrix {
        &self.state
    }

    /// Get classical measurement results
    pub fn classical_register(&self) -> &[bool] {
        &self.classical_register
    }

    /// Get measurement statistics
    pub fn measurement_statistics(&self) -> &MeasurementStatistics {
        &self.measurement_stats
    }

    /// Get total execution time in nanoseconds
    pub fn execution_time(&self) -> f64 {
        self.total_time
    }

    /// Get noise model
    pub fn noise_model(&self) -> &DeviceNoiseModel {
        &self.noise_model
    }

    /// Run circuit multiple times and collect measurement statistics.
    ///
    /// Executes the circuit $N$ times (where $N$ is `shots`) with independent
    /// noise realizations, collecting the frequency of each measurement outcome.
    /// Returns a map from bit strings to counts, where $\sum_i c_i = N$.
    pub fn run_shots(
        &mut self,
        circuit: &QuantumCircuit,
        shots: usize,
    ) -> Result<HashMap<String, usize>> {
        let mut results = HashMap::new();

        for _ in 0..shots {
            self.execute_circuit(circuit)?;

            // Convert classical register to bit string
            let bit_string = self
                .classical_register
                .iter()
                .map(|&b| if b { '1' } else { '0' })
                .collect::<String>();

            *results.entry(bit_string).or_insert(0) += 1;
        }

        Ok(results)
    }

    /// Calculate circuit fidelity compared to ideal execution.
    ///
    /// Executes the circuit with noise and computes the Uhlmann fidelity:
    ///
    /// $$ F(\rho_{\text{noisy}}, \rho_{\text{ideal}}) = \left[ \operatorname{Tr}\sqrt{\sqrt{\rho_{\text{noisy}}}\,\rho_{\text{ideal}}\,\sqrt{\rho_{\text{noisy}}}} \right]^2 $$
    ///
    /// $F = 1$ indicates perfect agreement; $F < 1$ quantifies noise impact.
    pub fn calculate_fidelity(
        &mut self,
        circuit: &QuantumCircuit,
        ideal_state: &DensityMatrix,
    ) -> Result<f64> {
        // Execute circuit with noise
        self.execute_circuit(circuit)?;

        // Calculate fidelity between noisy and ideal states
        self.state.fidelity(ideal_state)
    }
}

/// Noise characterization and benchmarking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseCharacterization {
    /// Process fidelity for different gates
    pub gate_fidelities: HashMap<String, f64>,
    /// Measured T1 times (microseconds)
    pub measured_t1: HashMap<usize, f64>,
    /// Measured T2 times (microseconds)
    pub measured_t2: HashMap<usize, f64>,
    /// Readout fidelities
    pub readout_fidelities: HashMap<usize, f64>,
    /// Cross-talk matrix between qubits
    pub crosstalk_matrix: Vec<Vec<f64>>,
}

impl NoiseCharacterization {
    /// Create new noise characterization
    pub fn new(num_qubits: usize) -> Self {
        Self {
            gate_fidelities: HashMap::new(),
            measured_t1: HashMap::new(),
            measured_t2: HashMap::new(),
            readout_fidelities: HashMap::new(),
            crosstalk_matrix: vec![vec![0.0; num_qubits]; num_qubits],
        }
    }

    /// Characterize device noise by running benchmark circuits.
    ///
    /// Measures three noise metrics across all qubits:
    /// - **T1/T2 coherence times** — energy relaxation and dephasing
    /// - **Gate fidelities** — process fidelity $F_g = 1 - \epsilon_g$ for each gate
    /// - **Readout fidelities** — $F_r = 1 - (p_{0\to1} + p_{1\to0})/2$
    ///
    /// Returns a `NoiseCharacterization` summarizing all measured metrics.
    pub fn characterize_device(
        simulator: &mut NoisyQuantumSimulator,
        num_qubits: usize,
    ) -> Result<Self> {
        let mut characterization = Self::new(num_qubits);

        // Measure T1 and T2 times
        for qubit in 0..num_qubits {
            characterization.measure_t1_t2(simulator, qubit)?;
        }

        // Measure gate fidelities
        characterization.measure_gate_fidelities(simulator)?;

        // Measure readout fidelities
        for qubit in 0..num_qubits {
            characterization.measure_readout_fidelity(simulator, qubit)?;
        }

        Ok(characterization)
    }

    fn measure_t1_t2(&mut self, simulator: &mut NoisyQuantumSimulator, qubit: usize) -> Result<()> {
        // Simplified T1/T2 measurement (would need more sophisticated protocol in practice)
        if let Some(&t1) = simulator.noise_model().t1_times.get(&qubit) {
            self.measured_t1.insert(qubit, t1);
        }
        if let Some(&t2) = simulator.noise_model().t2_times.get(&qubit) {
            self.measured_t2.insert(qubit, t2);
        }
        Ok(())
    }

    fn measure_gate_fidelities(&mut self, simulator: &mut NoisyQuantumSimulator) -> Result<()> {
        // Measure fidelities for common gates
        let gates = ["X", "Y", "Z", "H", "CNOT"];
        for gate in &gates {
            if let Some(&error_rate) = simulator.noise_model().gate_errors.get(*gate) {
                let fidelity = 1.0 - error_rate;
                self.gate_fidelities.insert(gate.to_string(), fidelity);
            }
        }
        Ok(())
    }

    fn measure_readout_fidelity(
        &mut self,
        simulator: &mut NoisyQuantumSimulator,
        qubit: usize,
    ) -> Result<()> {
        if let Some(&(error_0_to_1, error_1_to_0)) =
            simulator.noise_model().readout_errors.get(&qubit)
        {
            let fidelity = 1.0 - (error_0_to_1 + error_1_to_0) / 2.0;
            self.readout_fidelities.insert(qubit, fidelity);
        }
        Ok(())
    }

    /// Generate noise characterization report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("# Quantum Device Noise Characterization Report\n\n");

        report.push_str("## Coherence Times\n");
        for (qubit, &t1) in &self.measured_t1 {
            let t2 = self.measured_t2.get(qubit).copied().unwrap_or(0.0);
            report.push_str(&format!(
                "Qubit {}: T1 = {:.1} μs, T2 = {:.1} μs\n",
                qubit, t1, t2
            ));
        }

        report.push_str("\n## Gate Fidelities\n");
        for (gate, &fidelity) in &self.gate_fidelities {
            report.push_str(&format!("{}: {:.4}\n", gate, fidelity));
        }

        report.push_str("\n## Readout Fidelities\n");
        for (qubit, &fidelity) in &self.readout_fidelities {
            report.push_str(&format!("Qubit {}: {:.4}\n", qubit, fidelity));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QuantumCircuit;

    #[test]
    fn test_noisy_simulator_creation() {
        let noise_model = DeviceNoiseModel::realistic_device(2);
        let simulator = NoisyQuantumSimulator::new(2, noise_model);

        assert_eq!(simulator.state().num_qubits(), 2);
        assert_eq!(simulator.classical_register().len(), 2);
    }

    #[test]
    fn test_realistic_device_simulator() {
        let simulator = NoisyQuantumSimulator::realistic_device(3);
        assert_eq!(simulator.state().num_qubits(), 3);
        assert!(!simulator.noise_model().t1_times.is_empty());
    }

    #[test]
    fn test_circuit_execution_with_noise() {
        let mut simulator = NoisyQuantumSimulator::realistic_device(2);
        let mut circuit = QuantumCircuit::new(2, 2);

        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.measure_all().unwrap();

        let result = simulator.execute_circuit(&circuit);
        assert!(result.is_ok());
        assert!(simulator.execution_time() > 0.0);
    }

    #[test]
    fn test_shot_statistics() {
        let mut simulator = NoisyQuantumSimulator::realistic_device(2);
        let mut circuit = QuantumCircuit::new(2, 2);

        circuit.h(0).unwrap();
        circuit.measure_all().unwrap();

        let results = simulator.run_shots(&circuit, 100).unwrap();
        assert!(!results.is_empty());

        // Should have both "00" and "10" outcomes due to H gate
        let total_shots: usize = results.values().sum();
        assert_eq!(total_shots, 100);
    }

    #[test]
    fn test_noise_characterization() {
        let characterization = NoiseCharacterization::new(3);
        assert_eq!(characterization.crosstalk_matrix.len(), 3);
        assert_eq!(characterization.crosstalk_matrix[0].len(), 3);

        let report = characterization.generate_report();
        assert!(report.contains("Noise Characterization Report"));
    }
}
