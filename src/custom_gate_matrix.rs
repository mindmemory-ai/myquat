// Custom Gate Matrix Computation
// Author: gA4ss
//
// Provides matrix computation for custom quantum gates

use crate::error::{MyQuatError, Result};
use crate::gates::StandardGate;
use crate::parameter::Parameter;
use ndarray::Array2;
use num_complex::Complex64;
use std::collections::HashMap;
use std::f64::consts::PI;

/// Custom gate matrix computer
pub struct CustomGateMatrix;

impl CustomGateMatrix {
    /// Compute matrix for a standard gate with parameters
    pub fn compute_standard_gate(
        gate: StandardGate,
        params: &[Parameter],
        symbols: &HashMap<String, f64>,
    ) -> Result<Array2<Complex64>> {
        match gate {
            // Single-qubit gates without parameters
            StandardGate::I => Ok(Self::identity_matrix()),
            StandardGate::X => Ok(Self::pauli_x_matrix()),
            StandardGate::Y => Ok(Self::pauli_y_matrix()),
            StandardGate::Z => Ok(Self::pauli_z_matrix()),
            StandardGate::H => Ok(Self::hadamard_matrix()),
            StandardGate::S => Ok(Self::s_matrix()),
            StandardGate::Sdg => Ok(Self::sdg_matrix()),
            StandardGate::T => Ok(Self::t_matrix()),
            StandardGate::Tdg => Ok(Self::tdg_matrix()),

            // Parametric single-qubit gates
            StandardGate::Rx => {
                let theta = Self::eval_param(&params[0], symbols)?;
                Ok(Self::rx_matrix(theta))
            }
            StandardGate::Ry => {
                let theta = Self::eval_param(&params[0], symbols)?;
                Ok(Self::ry_matrix(theta))
            }
            StandardGate::Rz => {
                let theta = Self::eval_param(&params[0], symbols)?;
                Ok(Self::rz_matrix(theta))
            }
            StandardGate::P => {
                let theta = Self::eval_param(&params[0], symbols)?;
                Ok(Self::phase_matrix(theta))
            }
            StandardGate::U1 => {
                let lambda = Self::eval_param(&params[0], symbols)?;
                Ok(Self::u1_matrix(lambda))
            }
            StandardGate::U2 => {
                let phi = Self::eval_param(&params[0], symbols)?;
                let lambda = Self::eval_param(&params[1], symbols)?;
                Ok(Self::u2_matrix(phi, lambda))
            }
            StandardGate::U3 | StandardGate::U => {
                let theta = Self::eval_param(&params[0], symbols)?;
                let phi = Self::eval_param(&params[1], symbols)?;
                let lambda = Self::eval_param(&params[2], symbols)?;
                Ok(Self::u3_matrix(theta, phi, lambda))
            }

            // Two-qubit gates
            StandardGate::CX => Ok(Self::cx_matrix()),
            StandardGate::CY => Ok(Self::cy_matrix()),
            StandardGate::CZ => Ok(Self::cz_matrix()),
            StandardGate::CH => Ok(Self::ch_matrix()),
            StandardGate::Swap => Ok(Self::swap_matrix()),
            StandardGate::ISwap => Ok(Self::iswap_matrix()),

            // Parametric two-qubit gates
            StandardGate::CRx => {
                let theta = Self::eval_param(&params[0], symbols)?;
                Ok(Self::crx_matrix(theta))
            }
            StandardGate::CRy => {
                let theta = Self::eval_param(&params[0], symbols)?;
                Ok(Self::cry_matrix(theta))
            }
            StandardGate::CRz => {
                let theta = Self::eval_param(&params[0], symbols)?;
                Ok(Self::crz_matrix(theta))
            }
            StandardGate::CP => {
                let theta = Self::eval_param(&params[0], symbols)?;
                Ok(Self::cp_matrix(theta))
            }

            // Three-qubit gates
            StandardGate::CCX => Ok(Self::ccx_matrix()),
            StandardGate::CSwap => Ok(Self::cswap_matrix()),

            _ => Err(MyQuatError::circuit_error(format!(
                "Matrix computation not implemented for gate: {:?}",
                gate
            ))),
        }
    }

    /// Evaluate a parameter to a concrete value
    fn eval_param(param: &Parameter, symbols: &HashMap<String, f64>) -> Result<f64> {
        match param {
            Parameter::Float(v) => Ok(*v),
            Parameter::Symbol(name) => symbols.get(name).copied().ok_or_else(|| {
                MyQuatError::invalid_parameter(format!(
                    "Symbol '{}' not found in symbol table",
                    name
                ))
            }),
            Parameter::Expression(_) => Err(MyQuatError::circuit_error(
                "Expression evaluation not yet implemented",
            )),
        }
    }

    // ===== Single-Qubit Gate Matrices =====

    /// Identity matrix
    pub fn identity_matrix() -> Array2<Complex64> {
        let mut matrix = Array2::zeros((2, 2));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 1]] = Complex64::new(1.0, 0.0);
        matrix
    }

    /// Pauli-X matrix
    pub fn pauli_x_matrix() -> Array2<Complex64> {
        let mut matrix = Array2::zeros((2, 2));
        matrix[[0, 1]] = Complex64::new(1.0, 0.0);
        matrix[[1, 0]] = Complex64::new(1.0, 0.0);
        matrix
    }

    /// Pauli-Y matrix
    pub fn pauli_y_matrix() -> Array2<Complex64> {
        let mut matrix = Array2::zeros((2, 2));
        matrix[[0, 1]] = Complex64::new(0.0, -1.0);
        matrix[[1, 0]] = Complex64::new(0.0, 1.0);
        matrix
    }

    /// Pauli-Z matrix
    pub fn pauli_z_matrix() -> Array2<Complex64> {
        let mut matrix = Array2::zeros((2, 2));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 1]] = Complex64::new(-1.0, 0.0);
        matrix
    }

    /// Hadamard matrix
    pub fn hadamard_matrix() -> Array2<Complex64> {
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        let mut matrix = Array2::zeros((2, 2));
        matrix[[0, 0]] = Complex64::new(inv_sqrt2, 0.0);
        matrix[[0, 1]] = Complex64::new(inv_sqrt2, 0.0);
        matrix[[1, 0]] = Complex64::new(inv_sqrt2, 0.0);
        matrix[[1, 1]] = Complex64::new(-inv_sqrt2, 0.0);
        matrix
    }

    /// S gate matrix
    pub fn s_matrix() -> Array2<Complex64> {
        let mut matrix = Array2::zeros((2, 2));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 1]] = Complex64::new(0.0, 1.0);
        matrix
    }

    /// S dagger matrix
    pub fn sdg_matrix() -> Array2<Complex64> {
        let mut matrix = Array2::zeros((2, 2));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 1]] = Complex64::new(0.0, -1.0);
        matrix
    }

    /// T gate matrix
    pub fn t_matrix() -> Array2<Complex64> {
        let mut matrix = Array2::zeros((2, 2));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 1]] = Complex64::new(0.0, PI / 4.0).exp();
        matrix
    }

    /// T dagger matrix
    pub fn tdg_matrix() -> Array2<Complex64> {
        let mut matrix = Array2::zeros((2, 2));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 1]] = Complex64::new(0.0, -PI / 4.0).exp();
        matrix
    }

    /// Rx rotation matrix
    pub fn rx_matrix(theta: f64) -> Array2<Complex64> {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        let mut matrix = Array2::zeros((2, 2));
        matrix[[0, 0]] = Complex64::new(cos, 0.0);
        matrix[[0, 1]] = Complex64::new(0.0, -sin);
        matrix[[1, 0]] = Complex64::new(0.0, -sin);
        matrix[[1, 1]] = Complex64::new(cos, 0.0);
        matrix
    }

    /// Ry rotation matrix
    pub fn ry_matrix(theta: f64) -> Array2<Complex64> {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        let mut matrix = Array2::zeros((2, 2));
        matrix[[0, 0]] = Complex64::new(cos, 0.0);
        matrix[[0, 1]] = Complex64::new(-sin, 0.0);
        matrix[[1, 0]] = Complex64::new(sin, 0.0);
        matrix[[1, 1]] = Complex64::new(cos, 0.0);
        matrix
    }

    /// Rz rotation matrix
    pub fn rz_matrix(theta: f64) -> Array2<Complex64> {
        let mut matrix = Array2::zeros((2, 2));
        matrix[[0, 0]] = Complex64::new(0.0, -theta / 2.0).exp();
        matrix[[1, 1]] = Complex64::new(0.0, theta / 2.0).exp();
        matrix
    }

    /// Phase gate matrix
    pub fn phase_matrix(theta: f64) -> Array2<Complex64> {
        let mut matrix = Array2::zeros((2, 2));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 1]] = Complex64::new(0.0, theta).exp();
        matrix
    }

    /// U1 gate matrix
    pub fn u1_matrix(lambda: f64) -> Array2<Complex64> {
        Self::phase_matrix(lambda)
    }

    /// U2 gate matrix
    pub fn u2_matrix(phi: f64, lambda: f64) -> Array2<Complex64> {
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        let mut matrix = Array2::zeros((2, 2));
        matrix[[0, 0]] = Complex64::new(inv_sqrt2, 0.0);
        matrix[[0, 1]] = -Complex64::new(0.0, lambda).exp() * inv_sqrt2;
        matrix[[1, 0]] = Complex64::new(0.0, phi).exp() * inv_sqrt2;
        matrix[[1, 1]] = Complex64::new(0.0, phi + lambda).exp() * inv_sqrt2;
        matrix
    }

    /// U3 gate matrix (universal single-qubit gate)
    pub fn u3_matrix(theta: f64, phi: f64, lambda: f64) -> Array2<Complex64> {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        let mut matrix = Array2::zeros((2, 2));
        matrix[[0, 0]] = Complex64::new(cos, 0.0);
        matrix[[0, 1]] = -Complex64::new(0.0, lambda).exp() * sin;
        matrix[[1, 0]] = Complex64::new(0.0, phi).exp() * sin;
        matrix[[1, 1]] = Complex64::new(0.0, phi + lambda).exp() * cos;
        matrix
    }

    // ===== Two-Qubit Gate Matrices =====

    /// CNOT (CX) matrix
    pub fn cx_matrix() -> Array2<Complex64> {
        let mut matrix = Array2::zeros((4, 4));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 1]] = Complex64::new(1.0, 0.0);
        matrix[[2, 3]] = Complex64::new(1.0, 0.0);
        matrix[[3, 2]] = Complex64::new(1.0, 0.0);
        matrix
    }

    /// Controlled-Y matrix
    pub fn cy_matrix() -> Array2<Complex64> {
        let mut matrix = Array2::zeros((4, 4));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 1]] = Complex64::new(1.0, 0.0);
        matrix[[2, 3]] = Complex64::new(0.0, -1.0);
        matrix[[3, 2]] = Complex64::new(0.0, 1.0);
        matrix
    }

    /// Controlled-Z matrix
    pub fn cz_matrix() -> Array2<Complex64> {
        let mut matrix = Array2::zeros((4, 4));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 1]] = Complex64::new(1.0, 0.0);
        matrix[[2, 2]] = Complex64::new(1.0, 0.0);
        matrix[[3, 3]] = Complex64::new(-1.0, 0.0);
        matrix
    }

    /// Controlled-Hadamard matrix
    pub fn ch_matrix() -> Array2<Complex64> {
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        let mut matrix = Array2::zeros((4, 4));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 1]] = Complex64::new(1.0, 0.0);
        matrix[[2, 2]] = Complex64::new(inv_sqrt2, 0.0);
        matrix[[2, 3]] = Complex64::new(inv_sqrt2, 0.0);
        matrix[[3, 2]] = Complex64::new(inv_sqrt2, 0.0);
        matrix[[3, 3]] = Complex64::new(-inv_sqrt2, 0.0);
        matrix
    }

    /// SWAP matrix
    pub fn swap_matrix() -> Array2<Complex64> {
        let mut matrix = Array2::zeros((4, 4));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 2]] = Complex64::new(1.0, 0.0);
        matrix[[2, 1]] = Complex64::new(1.0, 0.0);
        matrix[[3, 3]] = Complex64::new(1.0, 0.0);
        matrix
    }

    /// iSWAP matrix
    pub fn iswap_matrix() -> Array2<Complex64> {
        let mut matrix = Array2::zeros((4, 4));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 2]] = Complex64::new(0.0, 1.0);
        matrix[[2, 1]] = Complex64::new(0.0, 1.0);
        matrix[[3, 3]] = Complex64::new(1.0, 0.0);
        matrix
    }

    /// Controlled-Rx matrix
    pub fn crx_matrix(theta: f64) -> Array2<Complex64> {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        let mut matrix = Array2::zeros((4, 4));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 1]] = Complex64::new(1.0, 0.0);
        matrix[[2, 2]] = Complex64::new(cos, 0.0);
        matrix[[2, 3]] = Complex64::new(0.0, -sin);
        matrix[[3, 2]] = Complex64::new(0.0, -sin);
        matrix[[3, 3]] = Complex64::new(cos, 0.0);
        matrix
    }

    /// Controlled-Ry matrix
    pub fn cry_matrix(theta: f64) -> Array2<Complex64> {
        let cos = (theta / 2.0).cos();
        let sin = (theta / 2.0).sin();
        let mut matrix = Array2::zeros((4, 4));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 1]] = Complex64::new(1.0, 0.0);
        matrix[[2, 2]] = Complex64::new(cos, 0.0);
        matrix[[2, 3]] = Complex64::new(-sin, 0.0);
        matrix[[3, 2]] = Complex64::new(sin, 0.0);
        matrix[[3, 3]] = Complex64::new(cos, 0.0);
        matrix
    }

    /// Controlled-Rz matrix
    pub fn crz_matrix(theta: f64) -> Array2<Complex64> {
        let mut matrix = Array2::zeros((4, 4));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 1]] = Complex64::new(1.0, 0.0);
        matrix[[2, 2]] = Complex64::new(0.0, -theta / 2.0).exp();
        matrix[[3, 3]] = Complex64::new(0.0, theta / 2.0).exp();
        matrix
    }

    /// Controlled-Phase matrix
    pub fn cp_matrix(theta: f64) -> Array2<Complex64> {
        let mut matrix = Array2::zeros((4, 4));
        matrix[[0, 0]] = Complex64::new(1.0, 0.0);
        matrix[[1, 1]] = Complex64::new(1.0, 0.0);
        matrix[[2, 2]] = Complex64::new(1.0, 0.0);
        matrix[[3, 3]] = Complex64::new(0.0, theta).exp();
        matrix
    }

    // ===== Three-Qubit Gate Matrices =====

    /// Toffoli (CCX) matrix
    pub fn ccx_matrix() -> Array2<Complex64> {
        let mut matrix = Array2::eye(8);
        matrix[[6, 6]] = Complex64::new(0.0, 0.0);
        matrix[[7, 7]] = Complex64::new(0.0, 0.0);
        matrix[[6, 7]] = Complex64::new(1.0, 0.0);
        matrix[[7, 6]] = Complex64::new(1.0, 0.0);
        matrix
    }

    /// Fredkin (CSWAP) matrix
    pub fn cswap_matrix() -> Array2<Complex64> {
        let mut matrix = Array2::eye(8);
        matrix[[5, 5]] = Complex64::new(0.0, 0.0);
        matrix[[6, 6]] = Complex64::new(0.0, 0.0);
        matrix[[5, 6]] = Complex64::new(1.0, 0.0);
        matrix[[6, 5]] = Complex64::new(1.0, 0.0);
        matrix
    }

    /// Compute Kronecker product of two matrices
    pub fn kron(a: &Array2<Complex64>, b: &Array2<Complex64>) -> Array2<Complex64> {
        let (m1, n1) = (a.nrows(), a.ncols());
        let (m2, n2) = (b.nrows(), b.ncols());
        let mut result = Array2::zeros((m1 * m2, n1 * n2));

        for i in 0..m1 {
            for j in 0..n1 {
                let scalar = a[[i, j]];
                for k in 0..m2 {
                    for l in 0..n2 {
                        result[[i * m2 + k, j * n2 + l]] = scalar * b[[k, l]];
                    }
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_matrix() {
        let matrix = CustomGateMatrix::identity_matrix();
        assert_eq!(matrix[[0, 0]], Complex64::new(1.0, 0.0));
        assert_eq!(matrix[[1, 1]], Complex64::new(1.0, 0.0));
        assert_eq!(matrix[[0, 1]], Complex64::new(0.0, 0.0));
        assert_eq!(matrix[[1, 0]], Complex64::new(0.0, 0.0));
    }

    #[test]
    fn test_pauli_matrices() {
        let x = CustomGateMatrix::pauli_x_matrix();
        let y = CustomGateMatrix::pauli_y_matrix();
        let z = CustomGateMatrix::pauli_z_matrix();

        // Test X matrix
        assert_eq!(x[[0, 1]], Complex64::new(1.0, 0.0));
        assert_eq!(x[[1, 0]], Complex64::new(1.0, 0.0));

        // Test Y matrix
        assert_eq!(y[[0, 1]], Complex64::new(0.0, -1.0));
        assert_eq!(y[[1, 0]], Complex64::new(0.0, 1.0));

        // Test Z matrix
        assert_eq!(z[[0, 0]], Complex64::new(1.0, 0.0));
        assert_eq!(z[[1, 1]], Complex64::new(-1.0, 0.0));
    }

    #[test]
    fn test_hadamard_matrix() {
        let h = CustomGateMatrix::hadamard_matrix();
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();

        assert!((h[[0, 0]].re - inv_sqrt2).abs() < 1e-10);
        assert!((h[[1, 1]].re + inv_sqrt2).abs() < 1e-10);
    }

    #[test]
    fn test_rotation_matrices() {
        let theta = PI / 4.0;

        let rx = CustomGateMatrix::rx_matrix(theta);
        let ry = CustomGateMatrix::ry_matrix(theta);
        let rz = CustomGateMatrix::rz_matrix(theta);

        // Basic sanity checks
        assert!(rx.nrows() == 2 && rx.ncols() == 2);
        assert!(ry.nrows() == 2 && ry.ncols() == 2);
        assert!(rz.nrows() == 2 && rz.ncols() == 2);
    }

    #[test]
    fn test_cx_matrix() {
        let cx = CustomGateMatrix::cx_matrix();

        assert_eq!(cx[[0, 0]], Complex64::new(1.0, 0.0));
        assert_eq!(cx[[1, 1]], Complex64::new(1.0, 0.0));
        assert_eq!(cx[[2, 3]], Complex64::new(1.0, 0.0));
        assert_eq!(cx[[3, 2]], Complex64::new(1.0, 0.0));
    }

    #[test]
    fn test_compute_standard_gate() {
        let symbols = HashMap::new();

        // Test identity
        let i_matrix =
            CustomGateMatrix::compute_standard_gate(StandardGate::I, &[], &symbols).unwrap();
        assert_eq!(i_matrix.nrows(), 2);

        // Test parametric gate
        let rx_matrix = CustomGateMatrix::compute_standard_gate(
            StandardGate::Rx,
            &[Parameter::Float(PI / 2.0)],
            &symbols,
        )
        .unwrap();
        assert_eq!(rx_matrix.nrows(), 2);
    }

    #[test]
    fn test_kron_product() {
        let i = CustomGateMatrix::identity_matrix();
        let x = CustomGateMatrix::pauli_x_matrix();

        let result = CustomGateMatrix::kron(&i, &x);
        assert_eq!(result.nrows(), 4);
        assert_eq!(result.ncols(), 4);
    }
}
