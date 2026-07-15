// Gate Inverse and Adjoint Operations
// Author: gA4ss
//
// Provides inverse gate computation and verification for quantum gates

use crate::error::{MyQuatError, Result};
use crate::gates::{Gate, StandardGate};
use crate::linalg::{LinalgBackend, LinalgResult, NdArrayBackend};
use crate::parameter::Parameter;
use ndarray::Array2;
use num_complex::Complex64;

/// Gate inverse analyzer
pub struct GateInverse;

impl GateInverse {
    /// Check if a gate is self-inverse (Hermitian)
    pub fn is_self_inverse(gate: &Gate) -> bool {
        Self::is_standard_gate_self_inverse(&gate.gate_type)
    }

    /// Check if a standard gate is self-inverse
    fn is_standard_gate_self_inverse(gate: &StandardGate) -> bool {
        matches!(
            gate,
            StandardGate::H |      // Hadamard
            StandardGate::X |      // Pauli-X
            StandardGate::Y |      // Pauli-Y
            StandardGate::Z |      // Pauli-Z
            StandardGate::CX |     // CNOT
            StandardGate::CY |     // Controlled-Y
            StandardGate::CZ |     // Controlled-Z
            StandardGate::Swap // SWAP
        )
    }

    /// Compute the inverse of a gate
    pub fn inverse(gate: &Gate) -> Result<Gate> {
        let inverse_gate_type = Self::inverse_standard_gate(&gate.gate_type)?;

        // For parametric gates, negate the parameters
        let inverse_params = if !gate.parameters.is_empty() {
            gate.parameters
                .iter()
                .map(Self::negate_parameter)
                .collect::<Result<Vec<_>>>()?
        } else {
            gate.parameters.clone()
        };

        Ok(Gate {
            gate_type: inverse_gate_type,
            parameters: inverse_params,
            label: gate.label.clone(),
        })
    }

    /// Compute inverse of a standard gate type
    fn inverse_standard_gate(gate: &StandardGate) -> Result<StandardGate> {
        let inverse_gate = match gate {
            // Self-inverse gates
            StandardGate::H
            | StandardGate::X
            | StandardGate::Y
            | StandardGate::Z
            | StandardGate::CX
            | StandardGate::CY
            | StandardGate::CZ
            | StandardGate::Swap => *gate,

            // Phase gates
            StandardGate::S => StandardGate::Sdg,
            StandardGate::Sdg => StandardGate::S,
            StandardGate::T => StandardGate::Tdg,
            StandardGate::Tdg => StandardGate::T,

            // Rotation gates - same type, parameters will be negated
            StandardGate::Rx
            | StandardGate::Ry
            | StandardGate::Rz
            | StandardGate::P
            | StandardGate::U1
            | StandardGate::U2
            | StandardGate::U3
            | StandardGate::CRx
            | StandardGate::CRy
            | StandardGate::CRz
            | StandardGate::CP => *gate,

            _ => {
                return Err(MyQuatError::circuit_error(format!(
                    "Inverse not defined for gate: {:?}",
                    gate
                )))
            }
        };

        Ok(inverse_gate)
    }

    /// Negate a parameter
    fn negate_parameter(param: &Parameter) -> Result<Parameter> {
        match param {
            Parameter::Float(value) => Ok(Parameter::Float(-value)),
            Parameter::Symbol(name) => {
                // Create a negated symbolic parameter
                Ok(Parameter::Symbol(format!("-{}", name)))
            }
            Parameter::Expression(_) => {
                // For expressions, we'd need to negate the entire expression
                // For now, return error
                Err(MyQuatError::circuit_error(
                    "Cannot negate complex parameter expressions",
                ))
            }
        }
    }

    /// Check if two gates are inverses of each other
    pub fn are_inverses(gate1: &Gate, gate2: &Gate) -> Result<bool> {
        let inv1 = Self::inverse(gate1)?;
        Ok(Self::gates_equal(&inv1, gate2))
    }

    /// Check if two gates are equal
    fn gates_equal(gate1: &Gate, gate2: &Gate) -> bool {
        gate1.gate_type == gate2.gate_type
            && gate1.parameters.len() == gate2.parameters.len()
            && gate1
                .parameters
                .iter()
                .zip(gate2.parameters.iter())
                .all(|(a, b)| Self::params_equal(a, b))
    }

    /// Check if two parameters are equal
    fn params_equal(p1: &Parameter, p2: &Parameter) -> bool {
        match (p1, p2) {
            (Parameter::Float(v1), Parameter::Float(v2)) => (v1 - v2).abs() < 1e-10,
            (Parameter::Symbol(s1), Parameter::Symbol(s2)) => s1 == s2,
            _ => false,
        }
    }

    /// Verify that gate * gate† = I (identity)
    pub fn verify_inverse_matrix(
        matrix: &Array2<Complex64>,
        inverse_matrix: &Array2<Complex64>,
    ) -> Result<bool> {
        if matrix.nrows() != matrix.ncols() {
            return Err(MyQuatError::circuit_error("Matrix must be square"));
        }

        if matrix.shape() != inverse_matrix.shape() {
            return Err(MyQuatError::circuit_error("Matrices must have same shape"));
        }

        let n = matrix.nrows();
        let product = matrix.dot(inverse_matrix);

        // Check if product is identity
        let mut is_identity = true;
        for i in 0..n {
            for j in 0..n {
                let expected = if i == j { 1.0 } else { 0.0 };
                let diff = (product[[i, j]].re - expected).abs() + product[[i, j]].im.abs();
                if diff > 1e-10 {
                    is_identity = false;
                    break;
                }
            }
            if !is_identity {
                break;
            }
        }

        Ok(is_identity)
    }

    /// Compute the adjoint (Hermitian conjugate) of a matrix.
    /// Delegates to `NdArrayBackend::conjugate_transpose()`.
    pub fn adjoint_matrix(matrix: &Array2<Complex64>) -> Array2<Complex64> {
        let backend = NdArrayBackend::new();
        backend
            .conjugate_transpose(matrix)
            .expect("Conjugate transpose should not fail for valid matrices")
    }

    /// Compute the adjoint using a linear algebra backend
    pub fn adjoint_matrix_with_backend<B: LinalgBackend<Scalar = Complex64>>(
        matrix: &B::Matrix,
        backend: &B,
    ) -> LinalgResult<B::Matrix> {
        backend.conjugate_transpose(matrix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_inverse_gates() {
        let h_gate = Gate {
            gate_type: StandardGate::H,
            parameters: vec![],
            label: None,
        };
        let x_gate = Gate {
            gate_type: StandardGate::X,
            parameters: vec![],
            label: None,
        };
        let y_gate = Gate {
            gate_type: StandardGate::Y,
            parameters: vec![],
            label: None,
        };
        let z_gate = Gate {
            gate_type: StandardGate::Z,
            parameters: vec![],
            label: None,
        };

        assert!(GateInverse::is_self_inverse(&h_gate));
        assert!(GateInverse::is_self_inverse(&x_gate));
        assert!(GateInverse::is_self_inverse(&y_gate));
        assert!(GateInverse::is_self_inverse(&z_gate));
    }

    #[test]
    fn test_non_self_inverse_gates() {
        let s_gate = Gate {
            gate_type: StandardGate::S,
            parameters: vec![],
            label: None,
        };
        let t_gate = Gate {
            gate_type: StandardGate::T,
            parameters: vec![],
            label: None,
        };

        assert!(!GateInverse::is_self_inverse(&s_gate));
        assert!(!GateInverse::is_self_inverse(&t_gate));
    }

    #[test]
    fn test_phase_gate_inverse() {
        let s_gate = Gate {
            gate_type: StandardGate::S,
            parameters: vec![],
            label: None,
        };
        let s_inv = GateInverse::inverse(&s_gate).unwrap();
        assert_eq!(s_inv.gate_type, StandardGate::Sdg);

        let t_gate = Gate {
            gate_type: StandardGate::T,
            parameters: vec![],
            label: None,
        };
        let t_inv = GateInverse::inverse(&t_gate).unwrap();
        assert_eq!(t_inv.gate_type, StandardGate::Tdg);
    }

    #[test]
    fn test_rotation_gate_inverse() {
        let rx = Gate {
            gate_type: StandardGate::Rx,
            parameters: vec![Parameter::Float(1.5)],
            label: None,
        };

        let rx_inv = GateInverse::inverse(&rx).unwrap();

        assert_eq!(rx_inv.gate_type, StandardGate::Rx);
        assert_eq!(rx_inv.parameters.len(), 1);
        if let Parameter::Float(v) = rx_inv.parameters[0] {
            assert!((v + 1.5).abs() < 1e-10);
        } else {
            panic!("Expected float parameter");
        }
    }

    #[test]
    fn test_are_inverses() {
        let s = Gate {
            gate_type: StandardGate::S,
            parameters: vec![],
            label: None,
        };
        let sdg = Gate {
            gate_type: StandardGate::Sdg,
            parameters: vec![],
            label: None,
        };

        assert!(GateInverse::are_inverses(&s, &sdg).unwrap());
        assert!(GateInverse::are_inverses(&sdg, &s).unwrap());
    }

    #[test]
    fn test_adjoint_matrix() {
        // Create a simple 2x2 matrix
        let mut matrix = Array2::zeros((2, 2));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[0, 1]] = Complex64::new(0.0, 1.0);
        matrix[[1, 0]] = Complex64::new(0.0, -1.0);
        matrix[[1, 1]] = Complex64::new(1.0, 0.0);

        let adj = GateInverse::adjoint_matrix(&matrix);

        // Check adjoint properties
        assert_eq!(adj[[0, 0]], Complex64::new(1.0, 0.0));
        assert_eq!(adj[[0, 1]], Complex64::new(0.0, 1.0));
        assert_eq!(adj[[1, 0]], Complex64::new(0.0, -1.0));
        assert_eq!(adj[[1, 1]], Complex64::new(1.0, 0.0));
    }

    #[test]
    fn test_verify_inverse_identity() {
        // Create identity matrix
        let mut identity = Array2::zeros((2, 2));
        identity[[0, 0]] = Complex64::new(1.0, 0.0);
        identity[[1, 1]] = Complex64::new(1.0, 0.0);

        // Identity is its own inverse
        assert!(GateInverse::verify_inverse_matrix(&identity, &identity).unwrap());
    }
}
