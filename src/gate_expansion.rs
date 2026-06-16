// Universal n-Qubit Gate Expansion
// Author: gA4ss
//
// Provides expansion of arbitrary n-qubit gates to basic gate sets

use crate::error::{MyQuatError, Result};
use crate::gates::StandardGate;
use crate::parameter::Parameter;
use crate::QuantumCircuit;
use ndarray::Array2;
use num_complex::Complex64;

/// Gate expansion strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpansionStrategy {
    /// Expand to CNOT + single-qubit gates
    CNOTBased,
    /// Expand to CZ + single-qubit gates  
    CZBased,
    /// Expand to minimal depth
    MinimalDepth,
    /// Expand to minimal gate count
    MinimalGates,
}

/// Universal gate expander
pub struct GateExpander {
    strategy: ExpansionStrategy,
}

impl GateExpander {
    /// Create a new gate expander
    pub fn new(strategy: ExpansionStrategy) -> Self {
        GateExpander { strategy }
    }

    /// Get the current expansion strategy
    pub fn strategy(&self) -> ExpansionStrategy {
        self.strategy
    }

    /// Set the expansion strategy
    pub fn set_strategy(&mut self, strategy: ExpansionStrategy) {
        self.strategy = strategy;
    }

    /// Expand a multi-controlled gate
    pub fn expand_multi_controlled(
        &self,
        circuit: &mut QuantumCircuit,
        controls: &[usize],
        target: usize,
        gate: StandardGate,
    ) -> Result<()> {
        match controls.len() {
            0 => {
                // No controls, just apply the gate
                self.apply_single_gate(circuit, target, gate)?;
            }
            1 => {
                // Single control, use standard controlled gate
                self.apply_controlled_gate(circuit, controls[0], target, gate)?;
            }
            2 => {
                // Double control (Toffoli-like)
                self.expand_double_controlled(circuit, controls[0], controls[1], target, gate)?;
            }
            _ => {
                // Multiple controls, use recursive decomposition
                self.expand_n_controlled(circuit, controls, target, gate)?;
            }
        }
        Ok(())
    }

    /// Apply a single-qubit gate
    fn apply_single_gate(
        &self,
        circuit: &mut QuantumCircuit,
        qubit: usize,
        gate: StandardGate,
    ) -> Result<()> {
        match gate {
            StandardGate::X => circuit.x(qubit)?,
            StandardGate::Y => circuit.y(qubit)?,
            StandardGate::Z => circuit.z(qubit)?,
            StandardGate::H => circuit.h(qubit)?,
            StandardGate::S => circuit.s(qubit)?,
            StandardGate::T => circuit.t(qubit)?,
            _ => {
                return Err(MyQuatError::circuit_error(format!(
                    "Unsupported single-qubit gate: {:?}",
                    gate
                )))
            }
        }
        Ok(())
    }

    /// Apply a controlled gate
    fn apply_controlled_gate(
        &self,
        circuit: &mut QuantumCircuit,
        control: usize,
        target: usize,
        gate: StandardGate,
    ) -> Result<()> {
        match gate {
            StandardGate::X => circuit.cx(control, target)?,
            StandardGate::Y => circuit.cy(control, target)?,
            StandardGate::Z => circuit.cz(control, target)?,
            StandardGate::H => {
                // CH decomposition
                circuit.ry(target, Parameter::Float(std::f64::consts::PI / 4.0))?;
                circuit.cx(control, target)?;
                circuit.ry(target, Parameter::Float(-std::f64::consts::PI / 4.0))?;
            }
            _ => {
                return Err(MyQuatError::circuit_error(format!(
                    "Unsupported controlled gate: {:?}",
                    gate
                )))
            }
        }
        Ok(())
    }

    /// Expand double-controlled gate (Toffoli-like)
    fn expand_double_controlled(
        &self,
        circuit: &mut QuantumCircuit,
        c1: usize,
        c2: usize,
        target: usize,
        gate: StandardGate,
    ) -> Result<()> {
        match gate {
            StandardGate::X => {
                // Standard Toffoli decomposition
                circuit.h(target)?;
                circuit.cx(c2, target)?;
                circuit.tdg(target)?;
                circuit.cx(c1, target)?;
                circuit.t(target)?;
                circuit.cx(c2, target)?;
                circuit.tdg(target)?;
                circuit.cx(c1, target)?;
                circuit.t(c2)?;
                circuit.t(target)?;
                circuit.h(target)?;
                circuit.cx(c1, c2)?;
                circuit.t(c1)?;
                circuit.tdg(c2)?;
                circuit.cx(c1, c2)?;
            }
            StandardGate::Z => {
                // CCZ decomposition
                circuit.cx(c2, target)?;
                circuit.tdg(target)?;
                circuit.cx(c1, target)?;
                circuit.t(target)?;
                circuit.cx(c2, target)?;
                circuit.tdg(target)?;
                circuit.cx(c1, target)?;
                circuit.t(c2)?;
                circuit.t(target)?;
                circuit.cx(c1, c2)?;
                circuit.t(c1)?;
                circuit.tdg(c2)?;
                circuit.cx(c1, c2)?;
            }
            _ => {
                return Err(MyQuatError::circuit_error(format!(
                    "Unsupported double-controlled gate: {:?}",
                    gate
                )))
            }
        }
        Ok(())
    }

    /// Expand n-controlled gate using recursive decomposition
    fn expand_n_controlled(
        &self,
        _circuit: &mut QuantumCircuit,
        controls: &[usize],
        _target: usize,
        _gate: StandardGate,
    ) -> Result<()> {
        let n = controls.len();

        // Multi-controlled gate with n > 2 controls requires O(n) ancilla qubits
        // or an O(n²) circuit using the Barenco et al. decomposition.
        // Return an error until a proper implementation is available.
        // (n=2 is handled by expand_double_controlled via the caller expand_multi_controlled.)
        Err(MyQuatError::circuit_error(format!(
            "Multi-controlled gate with {} controls not yet implemented. \
             Use explicit gate decomposition for CCX/CCZ (n=2) or decompose manually.",
            n
        )))
    }

    /// Expand arbitrary unitary to basic gates
    pub fn expand_unitary(
        &self,
        circuit: &mut QuantumCircuit,
        matrix: &Array2<Complex64>,
        qubits: &[usize],
    ) -> Result<()> {
        let n = qubits.len();
        let dim = 1 << n;

        if matrix.nrows() != dim || matrix.ncols() != dim {
            return Err(MyQuatError::circuit_error(format!(
                "Matrix dimension {} doesn't match {} qubits",
                matrix.nrows(),
                n
            )));
        }

        match n {
            1 => self.expand_single_qubit_unitary(circuit, matrix, qubits[0])?,
            2 => self.expand_two_qubit_unitary(circuit, matrix, qubits[0], qubits[1])?,
            _ => {
                // For n > 2, use recursive decomposition
                // This is a placeholder - full implementation would use
                // Quantum Shannon Decomposition or similar
                return Err(MyQuatError::circuit_error(
                    "Arbitrary n-qubit unitary expansion not yet fully implemented",
                ));
            }
        }

        Ok(())
    }

    /// Expand single-qubit unitary using ZYZ decomposition
    fn expand_single_qubit_unitary(
        &self,
        circuit: &mut QuantumCircuit,
        matrix: &Array2<Complex64>,
        qubit: usize,
    ) -> Result<()> {
        // Use ZYZ decomposition
        use crate::gate_decomposition::zyz_decomposition;

        let angles = zyz_decomposition(matrix)?;

        // Apply Rz(φ) Ry(θ) Rz(λ)
        if angles.phi.abs() > 1e-10 {
            circuit.rz(qubit, Parameter::Float(angles.phi))?;
        }
        if angles.theta.abs() > 1e-10 {
            circuit.ry(qubit, Parameter::Float(angles.theta))?;
        }
        if angles.lambda.abs() > 1e-10 {
            circuit.rz(qubit, Parameter::Float(angles.lambda))?;
        }

        Ok(())
    }

    /// Expand two-qubit unitary using KAK decomposition
    fn expand_two_qubit_unitary(
        &self,
        circuit: &mut QuantumCircuit,
        matrix: &Array2<Complex64>,
        q1: usize,
        q2: usize,
    ) -> Result<()> {
        // Use KAK decomposition from two_qubit_decompose module
        use crate::two_qubit_decompose::{TwoQubitBasis, TwoQubitDecomposer};

        let decomposer = TwoQubitDecomposer::new(TwoQubitBasis::CX);
        let decomposition = decomposer.decompose(matrix.view())?;

        // Decompose single-qubit pre/post gates via ZYZ into circuit gates.
        // Gate application order is REVERSED from the algebraic KAK form
        // (circuit left-multiplies): apply post gates first, then K, then pre gates.
        let emit_zyz =
            |circ: &mut QuantumCircuit, gate_2x2: &Array2<Complex64>, qubit: usize| -> Result<()> {
                if let Ok((alpha, beta, gamma, _phase)) =
                    TwoQubitDecomposer::zyz_decomposition(gate_2x2)
                {
                    let eps = 1e-12;
                    if gamma.abs() > eps {
                        circ.rz(qubit, crate::parameter::Parameter::Float(gamma))?;
                    }
                    if beta.abs() > eps {
                        circ.ry(qubit, crate::parameter::Parameter::Float(beta))?;
                    }
                    if alpha.abs() > eps {
                        circ.rz(qubit, crate::parameter::Parameter::Float(alpha))?;
                    }
                }
                Ok(())
            };

        // Post-gates (B1 ⊗ B2): applied FIRST in circuit time
        emit_zyz(circuit, &decomposition.post_gates.0, q1)?;
        emit_zyz(circuit, &decomposition.post_gates.1, q2)?;

        // Interaction K = exp(i(x·XX + y·YY + z·ZZ)): apply CX gates
        // For the CX basis, each non-trivial KAK coefficient pair requires 2 CX gates,
        // except the single (pi/4, 0, 0) CX-equivalent which requires 1.
        // We use the Shannon decomposition pattern based on num_basis_gates.
        match decomposition.num_basis_gates {
            0 => {
                // No interaction: identity, nothing to emit
            }
            1 => {
                // Single CX-equivalent
                circuit.cx(q1, q2)?;
            }
            2 => {
                // Two CX pattern: CX·(I⊗Rz(2z))·CX
                circuit.cx(q1, q2)?;
                if decomposition.coefficients[2].abs() > 1e-12 {
                    circuit.rz(
                        q2,
                        crate::parameter::Parameter::Float(2.0 * decomposition.coefficients[2]),
                    )?;
                }
                circuit.cx(q1, q2)?;
            }
            3 => {
                // Three CX pattern: CX·(I⊗Rz(2z))·CX·(Ry(2y)⊗Ry(-2y))·CX
                circuit.cx(q1, q2)?;
                if decomposition.coefficients[2].abs() > 1e-12 {
                    circuit.rz(
                        q2,
                        crate::parameter::Parameter::Float(2.0 * decomposition.coefficients[2]),
                    )?;
                }
                circuit.cx(q1, q2)?;
                if decomposition.coefficients[1].abs() > 1e-12 {
                    circuit.ry(
                        q1,
                        crate::parameter::Parameter::Float(2.0 * decomposition.coefficients[1]),
                    )?;
                    circuit.ry(
                        q2,
                        crate::parameter::Parameter::Float(-2.0 * decomposition.coefficients[1]),
                    )?;
                }
                circuit.cx(q1, q2)?;
            }
            _ => unreachable!("num_basis_gates > 3"),
        }

        // Pre-gates (A1 ⊗ A2): applied LAST in circuit time
        emit_zyz(circuit, &decomposition.pre_gates.0, q1)?;
        emit_zyz(circuit, &decomposition.pre_gates.1, q2)?;

        Ok(())
    }

    /// Expand SWAP to CNOTs
    pub fn expand_swap(&self, circuit: &mut QuantumCircuit, q1: usize, q2: usize) -> Result<()> {
        circuit.cx(q1, q2)?;
        circuit.cx(q2, q1)?;
        circuit.cx(q1, q2)?;
        Ok(())
    }

    /// Expand Fredkin (CSWAP) to basic gates
    pub fn expand_fredkin(
        &self,
        circuit: &mut QuantumCircuit,
        control: usize,
        t1: usize,
        t2: usize,
    ) -> Result<()> {
        circuit.cx(t2, t1)?;
        // Toffoli(control, t1, t2)
        self.expand_double_controlled(circuit, control, t1, t2, StandardGate::X)?;
        circuit.cx(t2, t1)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expander_creation() {
        let expander = GateExpander::new(ExpansionStrategy::CNOTBased);
        assert_eq!(expander.strategy(), ExpansionStrategy::CNOTBased);
    }

    #[test]
    fn test_strategy_change() {
        let mut expander = GateExpander::new(ExpansionStrategy::CNOTBased);
        expander.set_strategy(ExpansionStrategy::CZBased);
        assert_eq!(expander.strategy(), ExpansionStrategy::CZBased);
    }

    #[test]
    fn test_expand_swap() {
        let expander = GateExpander::new(ExpansionStrategy::CNOTBased);
        let mut circuit = QuantumCircuit::new(2, 0);

        expander.expand_swap(&mut circuit, 0, 1).unwrap();
        assert_eq!(circuit.size(), 3); // 3 CNOTs
    }

    #[test]
    fn test_expand_multi_controlled_single() {
        let expander = GateExpander::new(ExpansionStrategy::CNOTBased);
        let mut circuit = QuantumCircuit::new(2, 0);

        expander
            .expand_multi_controlled(&mut circuit, &[0], 1, StandardGate::X)
            .unwrap();
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_expand_multi_controlled_double() {
        let expander = GateExpander::new(ExpansionStrategy::CNOTBased);
        let mut circuit = QuantumCircuit::new(3, 0);

        expander
            .expand_multi_controlled(&mut circuit, &[0, 1], 2, StandardGate::X)
            .unwrap();
        assert!(circuit.size() > 10); // Toffoli has many gates
    }

    #[test]
    fn test_expand_fredkin() {
        let expander = GateExpander::new(ExpansionStrategy::CNOTBased);
        let mut circuit = QuantumCircuit::new(3, 0);

        expander.expand_fredkin(&mut circuit, 0, 1, 2).unwrap();
        assert!(circuit.size() > 10);
    }
}
