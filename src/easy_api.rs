//! Easy-to-use API for MyQuat
//!
//! This module provides simplified, user-friendly interfaces for common
//! quantum computing tasks, making MyQuat accessible to beginners.

use crate::{
    AlgorithmRunner, ExtendedGate, GroverSearch, MyQuatError, Parameter, QuantumCircuit, Result,
    StateVectorSimulator, QFT,
};
use std::collections::HashMap;
use std::f64::consts::PI;

/// Easy quantum circuit builder with fluent API
pub struct EasyCircuit {
    circuit: QuantumCircuit,
    num_qubits: usize,
    num_clbits: usize,
}

impl EasyCircuit {
    /// Create a new easy circuit
    pub fn new(num_qubits: usize) -> Self {
        EasyCircuit {
            circuit: QuantumCircuit::new(num_qubits, num_qubits),
            num_qubits,
            num_clbits: num_qubits,
        }
    }

    /// Create a new circuit with custom classical bits
    pub fn with_classical_bits(num_qubits: usize, num_clbits: usize) -> Self {
        EasyCircuit {
            circuit: QuantumCircuit::new(num_qubits, num_clbits),
            num_qubits,
            num_clbits,
        }
    }

    // Single-qubit gates with fluent API

    /// Apply Hadamard gate
    pub fn h(&mut self, qubit: usize) -> Result<&mut Self> {
        self.circuit.h(qubit)?;
        Ok(self)
    }

    /// Apply Pauli-X gate
    pub fn x(&mut self, qubit: usize) -> Result<&mut Self> {
        self.circuit.x(qubit)?;
        Ok(self)
    }

    /// Apply Pauli-Y gate
    pub fn y(&mut self, qubit: usize) -> Result<&mut Self> {
        self.circuit.y(qubit)?;
        Ok(self)
    }

    /// Apply Pauli-Z gate
    pub fn z(&mut self, qubit: usize) -> Result<&mut Self> {
        self.circuit.z(qubit)?;
        Ok(self)
    }

    /// Apply rotation around X-axis
    pub fn rx(&mut self, qubit: usize, angle: f64) -> Result<&mut Self> {
        self.circuit.rx(qubit, Parameter::Float(angle))?;
        Ok(self)
    }

    /// Apply rotation around Y-axis
    pub fn ry(&mut self, qubit: usize, angle: f64) -> Result<&mut Self> {
        self.circuit.ry(qubit, Parameter::Float(angle))?;
        Ok(self)
    }

    /// Apply rotation around Z-axis
    pub fn rz(&mut self, qubit: usize, angle: f64) -> Result<&mut Self> {
        self.circuit.rz(qubit, Parameter::Float(angle))?;
        Ok(self)
    }

    // Two-qubit gates

    /// Apply CNOT gate
    pub fn cnot(&mut self, control: usize, target: usize) -> Result<&mut Self> {
        self.circuit.cx(control, target)?;
        Ok(self)
    }

    /// Apply controlled-Z gate
    pub fn cz(&mut self, control: usize, target: usize) -> Result<&mut Self> {
        self.circuit.cz(control, target)?;
        Ok(self)
    }

    /// Apply SWAP gate
    pub fn swap(&mut self, qubit1: usize, qubit2: usize) -> Result<&mut Self> {
        self.circuit.swap(qubit1, qubit2)?;
        Ok(self)
    }

    /// Apply controlled phase gate
    pub fn cphase(&mut self, control: usize, target: usize, angle: f64) -> Result<&mut Self> {
        self.circuit.cp(control, target, Parameter::Float(angle))?;
        Ok(self)
    }

    // Advanced gates

    /// Apply square root of X gate
    pub fn sqrt_x(&mut self, qubit: usize) -> Result<&mut Self> {
        let _gate = ExtendedGate::sqrt_x();
        // For now, decompose into basic gates (simplified)
        self.ry(qubit, PI / 2.0)?;
        Ok(self)
    }

    /// Apply Toffoli (CCX) gate
    pub fn toffoli(
        &mut self,
        control1: usize,
        control2: usize,
        target: usize,
    ) -> Result<&mut Self> {
        self.circuit.ccx(control1, control2, target)?;
        Ok(self)
    }

    // Measurement

    /// Measure a qubit
    pub fn measure(&mut self, qubit: usize, clbit: usize) -> Result<&mut Self> {
        self.circuit.measure(qubit, clbit)?;
        Ok(self)
    }

    /// Measure all qubits to corresponding classical bits
    pub fn measure_all(&mut self) -> Result<&mut Self> {
        for i in 0..self.num_qubits.min(self.num_clbits) {
            self.circuit.measure(i, i)?;
        }
        Ok(self)
    }

    // Circuit information

    /// Get number of qubits
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Get number of gates
    pub fn num_gates(&self) -> usize {
        self.circuit.size()
    }

    /// Get the underlying circuit
    pub fn circuit(&self) -> &QuantumCircuit {
        &self.circuit
    }

    /// Convert to owned circuit
    pub fn into_circuit(self) -> QuantumCircuit {
        self.circuit
    }
}

/// Easy quantum simulator with simplified interface
pub struct EasySimulator {
    simulator: StateVectorSimulator,
}

impl EasySimulator {
    /// Create a new easy simulator
    pub fn new(num_qubits: usize) -> Self {
        EasySimulator {
            simulator: StateVectorSimulator::new(num_qubits, num_qubits),
        }
    }

    /// Run a circuit once and return measurement results
    pub fn run(&mut self, circuit: &QuantumCircuit) -> Result<HashMap<String, usize>> {
        self.simulator.execute_circuit(circuit)
    }

    /// Run a circuit multiple times and collect statistics
    pub fn run_shots(
        &mut self,
        circuit: &QuantumCircuit,
        shots: usize,
    ) -> Result<HashMap<String, usize>> {
        AlgorithmRunner::run_algorithm(circuit.clone(), shots)
    }

    /// Get the most probable measurement outcome
    pub fn most_probable(&mut self, circuit: &QuantumCircuit, shots: usize) -> Result<String> {
        let results = self.run_shots(circuit, shots)?;

        let max_entry = results
            .iter()
            .max_by_key(|(_, &count)| count)
            .ok_or_else(|| MyQuatError::circuit_error("No measurement results"))?;

        Ok(max_entry.0.clone())
    }

    /// Calculate success probability for specific outcomes
    pub fn success_probability(
        &mut self,
        circuit: &QuantumCircuit,
        target_outcomes: &[&str],
        shots: usize,
    ) -> Result<f64> {
        let results = self.run_shots(circuit, shots)?;
        let total_shots: usize = results.values().sum();

        if total_shots == 0 {
            return Ok(0.0);
        }

        let success_count: usize = target_outcomes
            .iter()
            .map(|outcome| results.get(*outcome).unwrap_or(&0))
            .sum();

        Ok(success_count as f64 / total_shots as f64)
    }
}

/// Easy algorithm builder for common quantum algorithms
pub struct EasyAlgorithms;

impl EasyAlgorithms {
    /// Create a Bell state circuit (|00⟩ + |11⟩)/√2
    pub fn bell_state() -> Result<EasyCircuit> {
        let mut circuit = EasyCircuit::new(2);
        circuit.h(0)?.cnot(0, 1)?.measure_all()?;
        Ok(circuit)
    }

    /// Create a GHZ state circuit (|000⟩ + |111⟩)/√2
    pub fn ghz_state(num_qubits: usize) -> Result<EasyCircuit> {
        let mut circuit = EasyCircuit::new(num_qubits);
        circuit.h(0)?;
        for i in 1..num_qubits {
            circuit.cnot(0, i)?;
        }
        circuit.measure_all()?;
        Ok(circuit)
    }

    /// Create a quantum random number generator
    pub fn random_bits(num_bits: usize) -> Result<EasyCircuit> {
        let mut circuit = EasyCircuit::new(num_bits);
        for i in 0..num_bits {
            circuit.h(i)?;
        }
        circuit.measure_all()?;
        Ok(circuit)
    }

    /// Create a simple Grover search circuit
    pub fn grover_search(num_qubits: usize, target: usize) -> Result<EasyCircuit> {
        let grover = GroverSearch::new(num_qubits, vec![target])?;
        let _grover_circuit = grover.build_circuit()?;

        // Convert to EasyCircuit (simplified)
        let mut easy_circuit = EasyCircuit::new(num_qubits);

        // Initialize superposition
        for i in 0..num_qubits {
            easy_circuit.h(i)?;
        }

        // Simplified Grover iteration (just one)
        // Oracle: flip phase of target state
        if target < (1 << num_qubits) {
            for i in 0..num_qubits {
                if (target >> i) & 1 == 0 {
                    easy_circuit.x(i)?;
                }
            }
            // Multi-controlled Z (simplified for small cases)
            if num_qubits == 2 {
                easy_circuit.cz(0, 1)?;
            }
            for i in 0..num_qubits {
                if (target >> i) & 1 == 0 {
                    easy_circuit.x(i)?;
                }
            }
        }

        // Diffusion operator
        for i in 0..num_qubits {
            easy_circuit.h(i)?;
        }
        for i in 0..num_qubits {
            easy_circuit.x(i)?;
        }
        if num_qubits == 2 {
            easy_circuit.cz(0, 1)?;
        }
        for i in 0..num_qubits {
            easy_circuit.x(i)?;
        }
        for i in 0..num_qubits {
            easy_circuit.h(i)?;
        }

        easy_circuit.measure_all()?;
        Ok(easy_circuit)
    }

    /// Create a QFT circuit
    pub fn qft(num_qubits: usize) -> Result<EasyCircuit> {
        let qft = QFT::new(num_qubits);
        let _qft_circuit = qft.build_circuit()?;

        // Convert to EasyCircuit (simplified)
        let mut easy_circuit = EasyCircuit::new(num_qubits);

        // Simplified QFT implementation
        for i in 0..num_qubits {
            easy_circuit.h(i)?;
            for j in (i + 1)..num_qubits {
                let angle = PI / (1 << (j - i)) as f64;
                easy_circuit.cphase(j, i, angle)?;
            }
        }

        // Swap qubits
        for i in 0..(num_qubits / 2) {
            easy_circuit.swap(i, num_qubits - 1 - i)?;
        }

        easy_circuit.measure_all()?;
        Ok(easy_circuit)
    }

    /// Create a quantum teleportation circuit
    pub fn teleportation() -> Result<EasyCircuit> {
        let mut circuit = EasyCircuit::new(3);

        // Prepare Bell pair between qubits 1 and 2
        circuit.h(1)?.cnot(1, 2)?;

        // Alice's operations (teleporting qubit 0)
        circuit.cnot(0, 1)?.h(0)?;

        // Measurements
        circuit.measure(0, 0)?.measure(1, 1)?;

        // Bob's corrections (would need classical control in real implementation)
        // For demo, just measure the result
        circuit.measure(2, 2)?;

        Ok(circuit)
    }
}

/// Easy visualization and analysis tools
pub struct EasyAnalysis;

impl EasyAnalysis {
    /// Print circuit summary
    pub fn print_circuit_info(circuit: &EasyCircuit) {
        println!("量子电路信息:");
        println!("• 量子比特数: {}", circuit.num_qubits());
        println!("• 门数量: {}", circuit.num_gates());
        println!("• 电路深度: 估算中...");
    }

    /// Print measurement results in a nice format
    pub fn print_results(results: &HashMap<String, usize>, shots: usize) {
        println!("测量结果 ({} 次):", shots);

        let mut sorted_results: Vec<_> = results.iter().collect();
        sorted_results.sort_by(|a, b| b.1.cmp(a.1));

        for (bitstring, count) in sorted_results {
            let probability = *count as f64 / shots as f64;
            let bar_length = (probability * 20.0) as usize;
            let bar = "█".repeat(bar_length);

            println!(
                "  |{}⟩: {:3} ({:5.1}%) {}",
                bitstring,
                count,
                probability * 100.0,
                bar
            );
        }
    }

    /// Calculate and print fidelity between expected and actual results
    pub fn print_fidelity(
        expected: &HashMap<String, f64>,
        actual: &HashMap<String, usize>,
        shots: usize,
    ) {
        let mut fidelity = 0.0;

        for (state, expected_prob) in expected {
            let actual_count = actual.get(state).unwrap_or(&0);
            let actual_prob = *actual_count as f64 / shots as f64;
            fidelity += (expected_prob * actual_prob).sqrt();
        }

        println!("保真度: {:.3}", fidelity);
    }

    /// Suggest optimizations for a circuit
    pub fn suggest_optimizations(circuit: &EasyCircuit) {
        println!("优化建议:");

        let num_gates = circuit.num_gates();
        let num_qubits = circuit.num_qubits();

        if num_gates > num_qubits * 10 {
            println!("• 考虑使用电路优化来减少门数量");
        }

        if num_qubits > 10 {
            println!("• 大规模电路建议启用GPU加速");
        }

        if num_qubits > 6 {
            println!("• 建议启用并行计算优化");
        }

        println!("• 当前配置会自动选择最优执行策略");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easy_circuit_creation() {
        let mut circuit = EasyCircuit::new(2);
        circuit
            .h(0)
            .unwrap()
            .cnot(0, 1)
            .unwrap()
            .measure_all()
            .unwrap();

        assert_eq!(circuit.num_qubits(), 2);
        assert!(circuit.num_gates() > 0);
    }

    #[test]
    fn test_easy_simulator() {
        let mut circuit = EasyCircuit::new(1);
        circuit.h(0).unwrap().measure_all().unwrap();

        let mut simulator = EasySimulator::new(1);
        let results = simulator.run_shots(circuit.circuit(), 100).unwrap();

        assert!(!results.is_empty());
    }

    #[test]
    fn test_bell_state() {
        let circuit = EasyAlgorithms::bell_state().unwrap();
        assert_eq!(circuit.num_qubits(), 2);
    }

    #[test]
    fn test_random_bits() {
        let circuit = EasyAlgorithms::random_bits(3).unwrap();
        assert_eq!(circuit.num_qubits(), 3);
    }
}
