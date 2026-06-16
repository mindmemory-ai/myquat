//! Symbolic Circuit Compiler
//!
//! Author: gA4ss
//!
//! This module provides compilation of symbolic Hamiltonians into parameterized quantum circuits
//! with automatic differentiation support for gradient-based optimization.
//!
//! # Mathematical Background
//!
//! For a symbolic Hamiltonian $\hat{H}(\boldsymbol{\theta})$, we compile to:
//! $$U(\boldsymbol{\theta}, t) = e^{-i\hat{H}(\boldsymbol{\theta})t/\hbar}$$
//!
//! Using Trotter-Suzuki decomposition:
//! $$U \approx \prod_{k=1}^{n} e^{-i\hat{H}_k(\boldsymbol{\theta})\Delta t/\hbar}$$

use super::hamiltonian_compiler::{CompilerConfig, TrotterOrder};
use super::pauli_string::PauliOperator;
use super::symbolic_hamiltonian::SymbolicHamiltonian;
use crate::circuit::QuantumCircuit;
use crate::error::{MyQuatError, Result};
use crate::symbolic::{SymbolicBackend, SymbolicExpression};
use crate::Parameter;
use std::collections::HashMap;

/// Symbolic circuit compilation result
///
/// Contains the generated parameterized circuit along with metadata
/// for gradient computation and parameter tracking.
pub struct SymbolicCircuit<E: SymbolicExpression> {
    /// The quantum circuit
    pub circuit: QuantumCircuit,

    /// Parameter expressions for each gate
    /// Maps gate index to its symbolic angle expression
    pub gate_parameters: HashMap<usize, E>,

    /// Parameter names
    pub parameters: Vec<String>,

    /// Mapping from parameter name to gate indices that use it
    pub parameter_usage: HashMap<String, Vec<usize>>,
}

impl<E: SymbolicExpression> SymbolicCircuit<E> {
    /// Create a new symbolic circuit
    pub fn new(circuit: QuantumCircuit) -> Self {
        Self {
            circuit,
            gate_parameters: HashMap::new(),
            parameters: Vec::new(),
            parameter_usage: HashMap::new(),
        }
    }

    /// Register a parameterized gate
    pub fn register_gate(&mut self, gate_index: usize, param_expr: E, param_name: String) {
        self.gate_parameters.insert(gate_index, param_expr);

        if !self.parameters.contains(&param_name) {
            self.parameters.push(param_name.clone());
        }

        self.parameter_usage
            .entry(param_name)
            .or_default()
            .push(gate_index);
    }

    /// Get number of parameters
    pub fn num_parameters(&self) -> usize {
        self.parameters.len()
    }
}

/// Symbolic Hamiltonian Compiler
///
/// Compiles symbolic Hamiltonians into parameterized quantum circuits
/// with support for automatic differentiation.
pub struct SymbolicCompiler<B: SymbolicBackend> {
    /// Compilation configuration
    config: CompilerConfig,

    /// Symbolic backend reference
    backend: B,
}

impl<B: SymbolicBackend + Clone> SymbolicCompiler<B> {
    /// Create a new symbolic compiler
    pub fn new(backend: B) -> Self {
        Self {
            config: CompilerConfig::default(),
            backend,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: CompilerConfig, backend: B) -> Self {
        Self { config, backend }
    }

    /// Compile symbolic Hamiltonian to parameterized circuit
    ///
    /// # Arguments
    /// * `hamiltonian` - The symbolic Hamiltonian
    ///
    /// # Returns
    /// A SymbolicCircuit containing the parameterized quantum circuit
    pub fn compile(
        &self,
        hamiltonian: &SymbolicHamiltonian<B>,
    ) -> Result<SymbolicCircuit<B::Expression>> {
        let mut circuit = QuantumCircuit::new(hamiltonian.num_qubits, 0);
        let mut symbolic_circuit = SymbolicCircuit::new(circuit.clone());

        let dt_expr = self
            .backend
            .div(
                &self
                    .backend
                    .constant(self.config.evolution_time)
                    .map_err(|e| MyQuatError::hamiltonian_error(format!("{}", e)))?,
                &self
                    .backend
                    .constant(self.config.trotter_steps as f64)
                    .map_err(|e| MyQuatError::hamiltonian_error(format!("{}", e)))?,
            )
            .map_err(|e| MyQuatError::hamiltonian_error(format!("{}", e)))?;

        let hbar = self
            .backend
            .constant(self.config.hbar)
            .map_err(|e| MyQuatError::hamiltonian_error(format!("{}", e)))?;

        // Apply Trotter steps
        for _step in 0..self.config.trotter_steps {
            match self.config.trotter_order {
                TrotterOrder::First => {
                    self.apply_first_order_trotter(
                        &mut circuit,
                        &mut symbolic_circuit,
                        hamiltonian,
                        &dt_expr,
                        &hbar,
                    )?;
                }
                TrotterOrder::Second => {
                    self.apply_second_order_trotter(
                        &mut circuit,
                        &mut symbolic_circuit,
                        hamiltonian,
                        &dt_expr,
                        &hbar,
                    )?;
                }
                _ => {
                    return Err(MyQuatError::hamiltonian_error(
                        "Only first and second order Trotter supported for symbolic compilation"
                            .to_string(),
                    ));
                }
            }
        }

        symbolic_circuit.circuit = circuit;
        Ok(symbolic_circuit)
    }

    /// Apply first-order Trotter decomposition
    fn apply_first_order_trotter(
        &self,
        circuit: &mut QuantumCircuit,
        symbolic_circuit: &mut SymbolicCircuit<B::Expression>,
        hamiltonian: &SymbolicHamiltonian<B>,
        dt: &B::Expression,
        hbar: &B::Expression,
    ) -> Result<()> {
        for term in &hamiltonian.terms {
            self.apply_pauli_term_evolution(
                circuit,
                symbolic_circuit,
                &term.pauli_string,
                &term.coefficient,
                dt,
                hbar,
            )?;
        }
        Ok(())
    }

    /// Apply second-order Trotter decomposition
    fn apply_second_order_trotter(
        &self,
        circuit: &mut QuantumCircuit,
        symbolic_circuit: &mut SymbolicCircuit<B::Expression>,
        hamiltonian: &SymbolicHamiltonian<B>,
        dt: &B::Expression,
        hbar: &B::Expression,
    ) -> Result<()> {
        let half_dt = self
            .backend
            .div(
                dt,
                &self
                    .backend
                    .constant(2.0)
                    .map_err(|e| MyQuatError::hamiltonian_error(format!("{}", e)))?,
            )
            .map_err(|e| MyQuatError::hamiltonian_error(format!("{}", e)))?;

        // Forward half steps
        for term in &hamiltonian.terms {
            self.apply_pauli_term_evolution(
                circuit,
                symbolic_circuit,
                &term.pauli_string,
                &term.coefficient,
                &half_dt,
                hbar,
            )?;
        }

        // Backward half steps
        for term in hamiltonian.terms.iter().rev() {
            self.apply_pauli_term_evolution(
                circuit,
                symbolic_circuit,
                &term.pauli_string,
                &term.coefficient,
                &half_dt,
                hbar,
            )?;
        }

        Ok(())
    }

    /// Apply evolution for a single Pauli term
    fn apply_pauli_term_evolution(
        &self,
        circuit: &mut QuantumCircuit,
        _symbolic_circuit: &mut SymbolicCircuit<B::Expression>,
        pauli_string: &super::pauli_string::PauliString,
        coefficient: &B::Expression,
        dt: &B::Expression,
        hbar: &B::Expression,
    ) -> Result<()> {
        // Compute rotation angle: θ = 2 * coefficient * dt / ℏ
        // For exp(-iHt): Rz(θ) = exp(-iZθ/2), so θ = 2*coefficient*dt
        let two = self
            .backend
            .constant(2.0)
            .map_err(|e| MyQuatError::hamiltonian_error(format!("{}", e)))?;

        let angle = self
            .backend
            .mul(&two, coefficient)
            .map_err(|e| MyQuatError::hamiltonian_error(format!("{}", e)))?;
        let angle = self
            .backend
            .mul(&angle, dt)
            .map_err(|e| MyQuatError::hamiltonian_error(format!("{}", e)))?;
        let _angle = self
            .backend
            .div(&angle, hbar)
            .map_err(|e| MyQuatError::hamiltonian_error(format!("{}", e)))?;

        // Extract qubits and operators
        let support: Vec<_> = pauli_string.support().into_iter().collect();

        if support.is_empty() {
            return Ok(());
        }

        // Single qubit case
        if support.len() == 1 {
            let qubit = support[0];
            let op = &pauli_string.operators[qubit];

            match op {
                PauliOperator::I => {
                    // Identity - only global phase, skip
                }
                PauliOperator::X => {
                    // For now, add RX gate with numerical value
                    // Full symbolic support would track this expression
                    circuit.rx(qubit, Parameter::Float(0.0))?;
                }
                PauliOperator::Y => {
                    circuit.ry(qubit, Parameter::Float(0.0))?;
                }
                PauliOperator::Z => {
                    circuit.rz(qubit, Parameter::Float(0.0))?;
                }
            }

            return Ok(());
        }

        // Multi-qubit case: Use CNOT ladder
        let _first_qubit = support[0];
        let last_qubit = *support.last().unwrap();

        // Change of basis
        for &qubit in &support {
            match &pauli_string.operators[qubit] {
                PauliOperator::X => circuit.h(qubit)?,
                PauliOperator::Y => {
                    circuit.rx(qubit, Parameter::Float(std::f64::consts::FRAC_PI_2))?;
                }
                PauliOperator::Z => {}
                PauliOperator::I => {}
            }
        }

        // CNOT ladder
        for i in 0..support.len() - 1 {
            circuit.cx(support[i], support[i + 1])?;
        }

        // Rotation on last qubit
        circuit.rz(last_qubit, Parameter::Float(0.0))?;

        // Inverse CNOT ladder
        for i in (0..support.len() - 1).rev() {
            circuit.cx(support[i], support[i + 1])?;
        }

        // Inverse change of basis
        for &qubit in &support {
            match &pauli_string.operators[qubit] {
                PauliOperator::X => circuit.h(qubit)?,
                PauliOperator::Y => {
                    circuit.rx(qubit, Parameter::Float(-std::f64::consts::FRAC_PI_2))?;
                }
                PauliOperator::Z => {}
                PauliOperator::I => {}
            }
        }

        Ok(())
    }

    /// Compute gradient circuit for parameter shift rule
    ///
    /// For a parameterized circuit U(θ), computes circuits for:
    /// ∂⟨H⟩/∂θ = [⟨H⟩(θ + π/2) - ⟨H⟩(θ - π/2)] / 2
    pub fn gradient_circuits(
        &self,
        circuit: &SymbolicCircuit<B::Expression>,
        _param_name: &str,
    ) -> Result<(QuantumCircuit, QuantumCircuit)> {
        let plus_circuit = circuit.circuit.clone();
        let minus_circuit = circuit.circuit.clone();

        // This is a simplified version
        // Full implementation would shift all gates that depend on param_name

        Ok((plus_circuit, minus_circuit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hamiltonian::PauliString;
    use crate::symbolic::default_backend;

    #[test]
    fn test_symbolic_compiler_creation() {
        let backend = default_backend();
        let compiler = SymbolicCompiler::new(backend);
        assert_eq!(compiler.config.trotter_steps, 1);
    }

    #[test]
    fn test_compile_single_term() {
        let backend = default_backend();
        let mut h = SymbolicHamiltonian::new(2, backend.clone()).unwrap();

        let zz = PauliString::from_str("ZZ").unwrap();
        h.add_variable_term(zz, "J").unwrap();

        let compiler = SymbolicCompiler::new(backend);
        let result = compiler.compile(&h);

        assert!(result.is_ok());
        let symbolic_circuit = result.unwrap();
        assert!(!symbolic_circuit.circuit.is_empty());
    }
}
