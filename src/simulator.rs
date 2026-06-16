//! Quantum state collapse simulation system
//!
//! This module provides quantum circuit simulation with state collapse functionality,
//! including measurement operations and conditional gate execution.

use crate::circuit::Qubit;
use crate::quantum_info::QuantumState;
use crate::{MyQuatError, QuantumCircuit, Result};
use ndarray::Array1;
use num_complex::Complex64;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

/// Measurement result containing the measured bit value and probability
#[derive(Debug, Clone)]
pub struct MeasurementResult {
    /// The measured bit value (0 or 1)
    pub bit_value: u8,
    /// The probability of this measurement outcome
    pub probability: f64,
    /// The qubit that was measured
    pub qubit: usize,
    /// The classical bit where result was stored
    pub clbit: usize,
}

/// Classical register to store measurement results
#[derive(Debug, Clone)]
pub struct ClassicalRegister {
    /// Bit values stored in the register
    bits: Vec<u8>,
    /// Number of bits in the register
    num_bits: usize,
}

impl ClassicalRegister {
    /// Create a new classical register with specified number of bits
    pub fn new(num_bits: usize) -> Self {
        ClassicalRegister {
            bits: vec![0; num_bits],
            num_bits,
        }
    }

    /// Get the number of bits
    pub fn num_bits(&self) -> usize {
        self.num_bits
    }

    /// Set a bit value
    pub fn set_bit(&mut self, index: usize, value: u8) -> Result<()> {
        if index >= self.num_bits {
            return Err(MyQuatError::circuit_error(format!(
                "Classical bit index {} out of range (max: {})",
                index,
                self.num_bits - 1
            )));
        }
        if value > 1 {
            return Err(MyQuatError::circuit_error(format!(
                "Invalid bit value {} (must be 0 or 1)",
                value
            )));
        }
        self.bits[index] = value;
        Ok(())
    }

    /// Get a bit value
    pub fn get_bit(&self, index: usize) -> Result<u8> {
        if index >= self.num_bits {
            return Err(MyQuatError::circuit_error(format!(
                "Classical bit index {} out of range (max: {})",
                index,
                self.num_bits - 1
            )));
        }
        Ok(self.bits[index])
    }

    /// Get all bit values
    pub fn bits(&self) -> &[u8] {
        &self.bits
    }

    /// Convert to integer representation
    pub fn to_int(&self) -> usize {
        let mut result = 0;
        for (i, &bit) in self.bits.iter().enumerate() {
            result |= (bit as usize) << i;
        }
        result
    }
}

/// Quantum state vector simulator with collapse functionality
#[derive(Debug, Clone)]
pub struct StateVectorSimulator {
    /// Current quantum state
    state: QuantumState,
    /// Classical register for measurement results
    classical_register: ClassicalRegister,
    /// Random number generator seed (for reproducible results)
    rng_seed: Option<u64>,
    /// History of measurement results
    measurement_history: Vec<MeasurementResult>,
}

impl StateVectorSimulator {
    /// Create a new simulator with specified number of qubits and classical bits
    pub fn new(num_qubits: usize, num_clbits: usize) -> Self {
        StateVectorSimulator {
            state: QuantumState::zero_state(num_qubits),
            classical_register: ClassicalRegister::new(num_clbits),
            rng_seed: None,
            measurement_history: Vec::new(),
        }
    }

    /// Create a new simulator with a specific initial state
    pub fn new_with_state(initial_state: QuantumState, num_clbits: usize) -> Self {
        StateVectorSimulator {
            state: initial_state,
            classical_register: ClassicalRegister::new(num_clbits),
            rng_seed: None,
            measurement_history: Vec::new(),
        }
    }

    /// Set the random number generator seed for reproducible results
    pub fn set_seed(&mut self, seed: u64) {
        self.rng_seed = Some(seed);
    }

    /// Get the current quantum state
    pub fn state(&self) -> &QuantumState {
        &self.state
    }

    /// Get the classical register
    pub fn classical_register(&self) -> &ClassicalRegister {
        &self.classical_register
    }

    /// Get the measurement history
    pub fn measurement_history(&self) -> &[MeasurementResult] {
        &self.measurement_history
    }

    /// Reset the simulator to initial state
    pub fn reset(&mut self) {
        self.state = QuantumState::zero_state(self.state.num_qubits());
        self.classical_register = ClassicalRegister::new(self.classical_register.num_bits());
        self.measurement_history.clear();
    }

    /// Apply a unitary gate to the quantum state.
    ///
    /// The gate unitary $U$ acts on state $|\psi\rangle$:
    ///
    /// $$ |\psi'\rangle = U |\psi\rangle $$
    ///
    /// For a $k$-qubit gate, $U$ is embedded into the full $2^n$-dimensional
    /// Hilbert space via tensor product with identities on inactive qubits.
    pub fn apply_unitary(&mut self, unitary: &ndarray::Array2<Complex64>) -> Result<()> {
        self.state.apply_unitary(unitary)
    }

    /// Measure a single qubit and collapse the state.
    ///
    /// The measurement outcome $b \in \{0,1\}$ is sampled with probability:
    ///
    /// $$ P(b) = \langle \psi | \Pi_b | \psi \rangle $$
    ///
    /// where $\Pi_b = |b\rangle\langle b| \otimes I_{\text{rest}}$ projects onto
    /// the subspace where the measured qubit is $b$. After measurement the state
    /// collapses:
    ///
    /// $$ |\psi'\rangle = \frac{\Pi_b |\psi\rangle}{\sqrt{P(b)}} $$
    pub fn measure_qubit(
        &mut self,
        qubit_index: usize,
        clbit_index: usize,
    ) -> Result<MeasurementResult> {
        if qubit_index >= self.state.num_qubits() {
            return Err(MyQuatError::circuit_error(format!(
                "Qubit index {} out of range (max: {})",
                qubit_index,
                self.state.num_qubits() - 1
            )));
        }

        // Calculate probabilities for |0⟩ and |1⟩ outcomes
        let (prob_0, prob_1) = self.calculate_measurement_probabilities(qubit_index)?;

        // Perform probabilistic measurement
        let mut rng = if let Some(seed) = self.rng_seed {
            rand::rngs::StdRng::seed_from_u64(seed)
        } else {
            let mut thread_rng = rand::rng();
            rand::rngs::StdRng::from_rng(&mut thread_rng)
        };

        let random_value: f64 = rng.random();
        let (measured_bit, probability) = if random_value < prob_0 {
            (0u8, prob_0)
        } else {
            (1u8, prob_1)
        };

        // Collapse the state based on measurement result
        self.collapse_state(qubit_index, measured_bit)?;

        // Store result in classical register
        self.classical_register.set_bit(clbit_index, measured_bit)?;

        // Record measurement result
        let result = MeasurementResult {
            bit_value: measured_bit,
            probability,
            qubit: qubit_index,
            clbit: clbit_index,
        };

        self.measurement_history.push(result.clone());

        Ok(result)
    }

    /// Calculate measurement probabilities for a qubit.
    ///
    /// For qubit $j$, the probability of outcome $b \in \{0,1\}$ is:
    ///
    /// $$ P(b) = \sum_{x: x_j = b} |\langle x|\psi\rangle|^2 $$
    ///
    /// Returns $(P(0), P(1))$ where $P(0) + P(1) = 1$.
    pub fn calculate_measurement_probabilities(&self, qubit_index: usize) -> Result<(f64, f64)> {
        let num_qubits = self.state.num_qubits();
        let dim = 1 << num_qubits;

        let mut prob_0 = 0.0;
        let mut prob_1 = 0.0;

        // Sum probabilities for all basis states where the target qubit is |0⟩ or |1⟩
        for i in 0..dim {
            let qubit_bit = (i >> (num_qubits - 1 - qubit_index)) & 1;
            let amplitude = self.state.amplitude(i)?;
            let probability = amplitude.norm_sqr();

            if qubit_bit == 0 {
                prob_0 += probability;
            } else {
                prob_1 += probability;
            }
        }

        Ok((prob_0, prob_1))
    }

    /// Collapse the quantum state based on measurement result
    fn collapse_state(&mut self, qubit_index: usize, measured_bit: u8) -> Result<()> {
        let num_qubits = self.state.num_qubits();
        let dim = 1 << num_qubits;
        let mut new_amplitudes = Array1::zeros(dim);

        // Calculate normalization factor
        let (prob_0, prob_1) = self.calculate_measurement_probabilities(qubit_index)?;
        let norm_factor = if measured_bit == 0 {
            1.0 / prob_0.sqrt()
        } else {
            1.0 / prob_1.sqrt()
        };

        // Set amplitudes for states consistent with measurement result
        for i in 0..dim {
            let qubit_bit = (i >> (num_qubits - 1 - qubit_index)) & 1;

            if qubit_bit == measured_bit as usize {
                let old_amplitude = self.state.amplitude(i)?;
                new_amplitudes[i] = old_amplitude * norm_factor;
            }
            // States inconsistent with measurement get zero amplitude
        }

        // Update the state
        self.state = QuantumState::new(new_amplitudes)?;

        Ok(())
    }

    /// Execute a complete quantum circuit with measurements.
    ///
    /// The circuit is executed sequentially: each gate unitary $U_i$ transforms
    /// the state as $|\psi'\rangle = U_i |\psi\rangle$, and each measurement
    /// collapses the state onto a computational basis outcome with probability
    /// given by the Born rule.
    pub fn execute_circuit(&mut self, circuit: &QuantumCircuit) -> Result<HashMap<String, usize>> {
        // Check compatibility
        if circuit.num_qubits() != self.state.num_qubits() {
            return Err(MyQuatError::circuit_error(format!(
                "Circuit has {} qubits but simulator has {}",
                circuit.num_qubits(),
                self.state.num_qubits()
            )));
        }

        if circuit.num_clbits() > self.classical_register.num_bits() {
            return Err(MyQuatError::circuit_error(format!(
                "Circuit requires {} classical bits but simulator has {}",
                circuit.num_clbits(),
                self.classical_register.num_bits()
            )));
        }

        // Execute instructions sequentially
        let symbols = HashMap::new(); // For now, assume no parameters

        for instruction in circuit.data().instructions() {
            if instruction.is_measurement() {
                // Handle measurement instruction
                let qubit_idx = instruction.qubits[0].index();
                let clbit_idx = instruction.clbits[0].index();
                self.measure_qubit(qubit_idx, clbit_idx)?;
            } else {
                // Handle gate instruction
                let gate_matrix = instruction.gate.matrix(&symbols)?;
                let expanded_matrix = self.expand_gate_matrix(&gate_matrix, &instruction.qubits)?;
                self.apply_unitary(&expanded_matrix)?;
            }
        }

        // Return measurement results as counts
        let mut counts = HashMap::new();
        let bit_string = self
            .classical_register
            .bits()
            .iter()
            .rev() // Most significant bit first
            .map(|&b| b.to_string())
            .collect::<String>();
        counts.insert(bit_string, 1);

        Ok(counts)
    }

    /// Expand a gate matrix to act on the full state space.
    ///
    /// For a $k$-qubit gate $g$ on an $n$-qubit system, the expanded unitary is:
    ///
    /// $$ U = I^{\otimes (n-k)} \otimes g $$
    ///
    /// with the gate placed at the tensor product positions corresponding to the
    /// target qubits. Supports 1-qubit and 2-qubit gates.
    pub fn expand_gate_matrix(
        &self,
        gate_matrix: &ndarray::Array2<Complex64>,
        qubits: &[Qubit],
    ) -> Result<ndarray::Array2<Complex64>> {
        let n_qubits = self.state.num_qubits();
        let dim = 1 << n_qubits;
        let mut expanded = ndarray::Array2::zeros((dim, dim));

        match qubits.len() {
            1 => {
                let qubit_idx = qubits[0].index();
                for i in 0..dim {
                    for j in 0..dim {
                        // Check if states i and j differ only in the target qubit
                        let mask = !(1 << (n_qubits - 1 - qubit_idx));
                        if (i & mask) == (j & mask) {
                            let i_bit = (i >> (n_qubits - 1 - qubit_idx)) & 1;
                            let j_bit = (j >> (n_qubits - 1 - qubit_idx)) & 1;
                            expanded[[i, j]] = gate_matrix[[i_bit, j_bit]];
                        }
                    }
                }
            }
            2 => {
                let qubit0_idx = qubits[0].index();
                let qubit1_idx = qubits[1].index();

                for i in 0..dim {
                    for j in 0..dim {
                        // Check if states i and j differ only in the target qubits
                        let mask = !(1 << (n_qubits - 1 - qubit0_idx))
                            & !(1 << (n_qubits - 1 - qubit1_idx));
                        if (i & mask) == (j & mask) {
                            let i_bit0 = (i >> (n_qubits - 1 - qubit0_idx)) & 1;
                            let i_bit1 = (i >> (n_qubits - 1 - qubit1_idx)) & 1;
                            let j_bit0 = (j >> (n_qubits - 1 - qubit0_idx)) & 1;
                            let j_bit1 = (j >> (n_qubits - 1 - qubit1_idx)) & 1;

                            let i_state = i_bit0 * 2 + i_bit1;
                            let j_state = j_bit0 * 2 + j_bit1;
                            expanded[[i, j]] = gate_matrix[[i_state, j_state]];
                        }
                    }
                }
            }
            _ => {
                return Err(MyQuatError::circuit_error(format!(
                    "Matrix expansion not implemented for {}-qubit gates",
                    qubits.len()
                )));
            }
        }

        Ok(expanded)
    }

    /// Run the circuit multiple times and collect statistics
    pub fn run_circuit_multiple(
        &mut self,
        circuit: &QuantumCircuit,
        shots: usize,
    ) -> Result<HashMap<String, usize>> {
        let mut total_counts = HashMap::new();

        for _ in 0..shots {
            // Reset to initial state for each run
            self.reset();

            // Execute circuit
            let counts = self.execute_circuit(circuit)?;

            // Accumulate results
            for (bit_string, count) in counts {
                *total_counts.entry(bit_string).or_insert(0) += count;
            }
        }

        Ok(total_counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quantum_info::QuantumInfo;
    use approx::assert_relative_eq;

    #[test]
    fn test_classical_register() {
        let mut reg = ClassicalRegister::new(3);
        assert_eq!(reg.num_bits(), 3);

        reg.set_bit(0, 1).unwrap();
        reg.set_bit(2, 1).unwrap();

        assert_eq!(reg.get_bit(0).unwrap(), 1);
        assert_eq!(reg.get_bit(1).unwrap(), 0);
        assert_eq!(reg.get_bit(2).unwrap(), 1);

        assert_eq!(reg.to_int(), 5); // 101 in binary
    }

    #[test]
    fn test_simulator_creation() {
        let sim = StateVectorSimulator::new(2, 2);
        assert_eq!(sim.state().num_qubits(), 2);
        assert_eq!(sim.classical_register().num_bits(), 2);
        assert_eq!(sim.measurement_history().len(), 0);
    }

    #[test]
    fn test_measurement_probabilities() {
        let mut sim = StateVectorSimulator::new(1, 1);

        // Test |0⟩ state
        let (prob_0, prob_1) = sim.calculate_measurement_probabilities(0).unwrap();
        assert_relative_eq!(prob_0, 1.0, epsilon = 1e-10);
        assert_relative_eq!(prob_1, 0.0, epsilon = 1e-10);

        // Create |+⟩ state (equal superposition)
        let plus_state = QuantumState::plus_state(1);
        sim = StateVectorSimulator::new_with_state(plus_state, 1);

        let (prob_0, prob_1) = sim.calculate_measurement_probabilities(0).unwrap();
        assert_relative_eq!(prob_0, 0.5, epsilon = 1e-10);
        assert_relative_eq!(prob_1, 0.5, epsilon = 1e-10);
    }

    #[test]
    fn test_state_collapse() {
        // Create |+⟩ state
        let plus_state = QuantumState::plus_state(1);
        let mut sim = StateVectorSimulator::new_with_state(plus_state, 1);
        sim.set_seed(42); // For reproducible results

        // Measure the qubit
        let result = sim.measure_qubit(0, 0).unwrap();

        // Check that measurement result is valid
        assert!(result.bit_value == 0 || result.bit_value == 1);
        assert_relative_eq!(result.probability, 0.5, epsilon = 1e-10);

        // Check that state has collapsed
        let (prob_0, prob_1) = sim.calculate_measurement_probabilities(0).unwrap();
        if result.bit_value == 0 {
            assert_relative_eq!(prob_0, 1.0, epsilon = 1e-10);
            assert_relative_eq!(prob_1, 0.0, epsilon = 1e-10);
        } else {
            assert_relative_eq!(prob_0, 0.0, epsilon = 1e-10);
            assert_relative_eq!(prob_1, 1.0, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_bell_state_measurement() {
        // Create Bell state |Φ+⟩ = (|00⟩ + |11⟩)/√2
        let bell_state = QuantumInfo::bell_state(0).unwrap();
        let mut sim = StateVectorSimulator::new_with_state(bell_state, 2);
        sim.set_seed(123);

        // Measure first qubit
        let result1 = sim.measure_qubit(0, 0).unwrap();

        // Measure second qubit
        let result2 = sim.measure_qubit(1, 1).unwrap();

        // In a Bell state, both measurements should give the same result
        assert_eq!(result1.bit_value, result2.bit_value);

        // Each measurement should have 50% probability
        assert_relative_eq!(result1.probability, 0.5, epsilon = 1e-10);
        assert_relative_eq!(result2.probability, 1.0, epsilon = 1e-10); // After first measurement, second is deterministic
    }

    #[test]
    fn test_circuit_execution() {
        let mut circuit = QuantumCircuit::new(2, 2);
        circuit.h(0).unwrap(); // Create superposition
        circuit.cx(0, 1).unwrap(); // Create entanglement
        circuit.measure_all().unwrap(); // Measure both qubits

        let mut sim = StateVectorSimulator::new(2, 2);
        sim.set_seed(456);

        let counts = sim.execute_circuit(&circuit).unwrap();

        // Should get either "00" or "11" due to entanglement
        assert_eq!(counts.len(), 1);
        let bit_string = counts.keys().next().unwrap();
        assert!(bit_string == "00" || bit_string == "11");
    }
}
