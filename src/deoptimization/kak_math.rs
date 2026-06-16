// deoptimization/kak_math.rs - KAK decomposition mathematics
// Author: gA4ss
//
// Mathematical utilities for Kraus-Cirac (KAK) decomposition of 2-qubit gates.
// Used to analyze and restore Pauli rotations from decomposed gate sequences.

use nalgebra as na;
use num_complex::Complex64;

/// Type alias for 4x4 complex matrix (2-qubit unitary)
#[allow(dead_code)]
pub type Matrix4c = na::Matrix4<Complex64>;

/// Type alias for 2x2 complex matrix (single-qubit unitary)
#[allow(dead_code)]
pub type Matrix2c = na::Matrix2<Complex64>;

/// Pauli matrices
#[allow(dead_code)]
pub struct PauliMatrices;

#[allow(dead_code)]
impl PauliMatrices {
    /// Pauli I matrix
    pub fn i() -> Matrix2c {
        Matrix2c::new(
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(1.0, 0.0),
        )
    }

    /// Pauli X matrix
    pub fn x() -> Matrix2c {
        Matrix2c::new(
            Complex64::new(0.0, 0.0),
            Complex64::new(1.0, 0.0),
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
        )
    }

    /// Pauli Y matrix
    pub fn y() -> Matrix2c {
        Matrix2c::new(
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, -1.0),
            Complex64::new(0.0, 1.0),
            Complex64::new(0.0, 0.0),
        )
    }

    /// Pauli Z matrix
    pub fn z() -> Matrix2c {
        Matrix2c::new(
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(-1.0, 0.0),
        )
    }
}

/// Single-qubit gate matrices
#[allow(dead_code)]
pub struct SingleQubitGates;

#[allow(dead_code)]
impl SingleQubitGates {
    /// Hadamard gate
    pub fn h() -> Matrix2c {
        let val = 1.0 / 2.0_f64.sqrt();
        Matrix2c::new(
            Complex64::new(val, 0.0),
            Complex64::new(val, 0.0),
            Complex64::new(val, 0.0),
            Complex64::new(-val, 0.0),
        )
    }

    /// Rz rotation gate
    pub fn rz(theta: f64) -> Matrix2c {
        let half_theta = theta / 2.0;
        Matrix2c::new(
            Complex64::new(half_theta.cos(), -half_theta.sin()),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(half_theta.cos(), half_theta.sin()),
        )
    }

    /// Rx rotation gate
    pub fn rx(theta: f64) -> Matrix2c {
        let half_theta = theta / 2.0;
        let c = half_theta.cos();
        let s = half_theta.sin();
        Matrix2c::new(
            Complex64::new(c, 0.0),
            Complex64::new(0.0, -s),
            Complex64::new(0.0, -s),
            Complex64::new(c, 0.0),
        )
    }

    /// Ry rotation gate
    pub fn ry(theta: f64) -> Matrix2c {
        let half_theta = theta / 2.0;
        let c = half_theta.cos();
        let s = half_theta.sin();
        Matrix2c::new(
            Complex64::new(c, 0.0),
            Complex64::new(-s, 0.0),
            Complex64::new(s, 0.0),
            Complex64::new(c, 0.0),
        )
    }

    /// S gate (phase gate)
    pub fn s() -> Matrix2c {
        Matrix2c::new(
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 1.0),
        )
    }

    /// S dagger gate
    pub fn sdg() -> Matrix2c {
        Matrix2c::new(
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, -1.0),
        )
    }
}

/// Two-qubit gate matrices
#[allow(dead_code)]
pub struct TwoQubitGates;

#[allow(dead_code)]
impl TwoQubitGates {
    /// CNOT gate (control on qubit 0, target on qubit 1)
    pub fn cnot() -> Matrix4c {
        Matrix4c::new(
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(0.0, 0.0),
            Complex64::new(1.0, 0.0),
            Complex64::new(0.0, 0.0),
        )
    }
}

/// Tensor product of two 2x2 matrices to form a 4x4 matrix
#[allow(dead_code)]
pub fn tensor_product(a: &Matrix2c, b: &Matrix2c) -> Matrix4c {
    let mut result = Matrix4c::zeros();

    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                for l in 0..2 {
                    let row = i * 2 + k;
                    let col = j * 2 + l;
                    result[(row, col)] = a[(i, j)] * b[(k, l)];
                }
            }
        }
    }

    result
}

/// Pauli string representation
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct PauliString {
    /// Pauli operators for each qubit (I, X, Y, Z)
    pub paulis: Vec<char>,
    /// Rotation angle
    pub angle: f64,
}

#[allow(dead_code)]
impl PauliString {
    /// Create new Pauli string
    pub fn new(paulis: &str, angle: f64) -> Self {
        Self {
            paulis: paulis.chars().collect(),
            angle,
        }
    }

    /// Get the 2-qubit Pauli matrix for this string
    pub fn to_matrix(&self) -> Matrix4c {
        if self.paulis.len() != 2 {
            panic!("Only 2-qubit Pauli strings supported");
        }

        let p0 = self.get_pauli_matrix(self.paulis[0]);
        let p1 = self.get_pauli_matrix(self.paulis[1]);

        tensor_product(&p0, &p1)
    }

    fn get_pauli_matrix(&self, p: char) -> Matrix2c {
        match p {
            'I' => PauliMatrices::i(),
            'X' => PauliMatrices::x(),
            'Y' => PauliMatrices::y(),
            'Z' => PauliMatrices::z(),
            _ => panic!("Invalid Pauli operator: {}", p),
        }
    }

    /// Compute the rotation operator exp(-i * angle * P)
    pub fn to_rotation_matrix(&self) -> Matrix4c {
        let pauli = self.to_matrix();
        expm_pauli(&pauli, self.angle)
    }
}

/// Matrix exponential for Pauli operators: exp(-i * theta * P)
///
/// Uses the identity: exp(-i*θ*P) = cos(θ)I - i*sin(θ)P
/// which is valid for Pauli matrices since P² = I
#[allow(dead_code)]
fn expm_pauli(pauli: &Matrix4c, theta: f64) -> Matrix4c {
    let i = Complex64::new(0.0, 1.0);
    let cos_theta = Complex64::new(theta.cos(), 0.0);
    let sin_theta = Complex64::new(theta.sin(), 0.0);

    // exp(-i*θ*P) = cos(θ)I - i*sin(θ)P
    let identity = Matrix4c::identity();
    identity * cos_theta - pauli * i * sin_theta
}

/// Compare two matrices with tolerance
pub fn matrices_approx_equal(a: &Matrix4c, b: &Matrix4c, tol: f64) -> bool {
    for i in 0..4 {
        for j in 0..4 {
            let diff = (a[(i, j)] - b[(i, j)]).norm();
            if diff > tol {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f64::consts::PI;

    #[test]
    fn test_pauli_matrices() {
        let x = PauliMatrices::x();
        let y = PauliMatrices::y();
        let z = PauliMatrices::z();
        let i = PauliMatrices::i();

        // Test X² = I
        let x2 = x * x;
        assert!(matrices_approx_equal(
            &tensor_product(&x2, &PauliMatrices::i()),
            &tensor_product(&i, &PauliMatrices::i()),
            1e-10
        ));
    }

    #[test]
    fn test_hadamard() {
        let h = SingleQubitGates::h();
        let h2 = h * h;
        let i = PauliMatrices::i();

        // H² = I (up to global phase)
        for i_idx in 0..2 {
            for j in 0..2 {
                assert_relative_eq!(h2[(i_idx, j)].norm(), i[(i_idx, j)].norm(), epsilon = 1e-10);
            }
        }
    }

    #[test]
    fn test_cnot() {
        let cnot = TwoQubitGates::cnot();

        // CNOT should be unitary
        let cnot_dag = cnot.adjoint();
        let product = cnot * cnot_dag;
        let identity = Matrix4c::identity();

        assert!(matrices_approx_equal(&product, &identity, 1e-10));
    }

    #[test]
    fn test_tensor_product() {
        let x = PauliMatrices::x();
        let z = PauliMatrices::z();

        let xz = tensor_product(&x, &z);

        // Check dimensions
        assert_eq!(xz.nrows(), 4);
        assert_eq!(xz.ncols(), 4);
    }

    #[test]
    fn test_pauli_string_zz() {
        let zz = PauliString::new("ZZ", PI / 4.0);
        let matrix = zz.to_rotation_matrix();

        // Should be unitary
        let dag = matrix.adjoint();
        let product = matrix * dag;
        let identity = Matrix4c::identity();

        assert!(matrices_approx_equal(&product, &identity, 1e-10));
    }

    #[test]
    fn test_rz_gate() {
        let theta = PI / 3.0;
        let rz = SingleQubitGates::rz(theta);

        // Rz should be unitary
        let rz_dag = rz.adjoint();
        let product = rz * rz_dag;
        let identity = Matrix2c::identity();

        for i in 0..2 {
            for j in 0..2 {
                assert_relative_eq!(
                    product[(i, j)].norm(),
                    identity[(i, j)].norm(),
                    epsilon = 1e-10
                );
            }
        }
    }

    #[test]
    fn test_matrices_approx_equal() {
        let a = Matrix4c::identity();
        let b = Matrix4c::identity();

        assert!(matrices_approx_equal(&a, &b, 1e-10));

        let c = Matrix4c::identity() * Complex64::new(1.1, 0.0);
        assert!(!matrices_approx_equal(&a, &c, 0.01));
    }
}
