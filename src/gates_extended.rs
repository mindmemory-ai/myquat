//! Extended quantum gate library
//!
//! This module provides additional quantum gates beyond the standard set,
//! including advanced parametric gates, composite gates, and specialized operations.

use crate::error::{MyQuatError, Result};
use crate::gate_decomposition::arbitrary_rotation_matrix;
use crate::gates::GateOperation;
use crate::linalg::{LinalgBackend, LinalgResult};
use crate::parameter::Parameter;
use ndarray::Array2;
use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// PI is used in tests
#[cfg(test)]
use std::f64::consts::PI;

/// Extended quantum gates beyond the standard set
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExtendedGate {
    // Advanced single-qubit gates
    SqrtX, // Square root of X
    SqrtY, // Square root of Y
    SqrtZ, // Square root of Z
    SqrtH, // Square root of Hadamard

    // Arbitrary axis rotations
    R(f64, f64, f64), // Rotation around arbitrary axis (nx, ny, nz)

    // Phase gates
    GlobalPhase(f64), // Global phase gate
    PhaseShift(f64),  // Phase shift gate

    // Advanced parametric gates
    RXX(f64), // Two-qubit XX rotation
    RYY(f64), // Two-qubit YY rotation
    RZZ(f64), // Two-qubit ZZ rotation
    RZX(f64), // Two-qubit ZX rotation

    // Composite gates
    QFT(usize),  // Quantum Fourier Transform
    IQFT(usize), // Inverse QFT

    // Custom user-defined gate (matrix stored as real/imag pairs for serialization)
    Custom {
        name: String,
        matrix_real: Vec<Vec<f64>>, // Real parts
        matrix_imag: Vec<Vec<f64>>, // Imaginary parts
        num_qubits: usize,
        parameters: Vec<Parameter>,
    },
}

impl ExtendedGate {
    /// Create a square root of X gate
    pub fn sqrt_x() -> Self {
        ExtendedGate::SqrtX
    }

    /// Create a square root of Y gate
    pub fn sqrt_y() -> Self {
        ExtendedGate::SqrtY
    }

    /// Create an arbitrary axis rotation
    pub fn rotation_axis(nx: f64, ny: f64, nz: f64, angle: f64) -> Self {
        // Normalize the axis
        let norm = (nx * nx + ny * ny + nz * nz).sqrt();
        ExtendedGate::R(nx / norm * angle, ny / norm * angle, nz / norm * angle)
    }

    /// Create a two-qubit XX rotation
    pub fn rxx(angle: f64) -> Self {
        ExtendedGate::RXX(angle)
    }

    /// Create a two-qubit YY rotation  
    pub fn ryy(angle: f64) -> Self {
        ExtendedGate::RYY(angle)
    }

    /// Create a two-qubit ZZ rotation
    pub fn rzz(angle: f64) -> Self {
        ExtendedGate::RZZ(angle)
    }

    /// Create a custom gate from matrix
    pub fn custom(name: String, matrix: Array2<Complex64>) -> Result<Self> {
        let dim = matrix.nrows();
        if matrix.ncols() != dim {
            return Err(MyQuatError::circuit_error("Gate matrix must be square"));
        }

        // Check if dimension is a power of 2
        let num_qubits = (dim as f64).log2();
        if num_qubits.fract() != 0.0 {
            return Err(MyQuatError::circuit_error(
                "Gate matrix dimension must be power of 2",
            ));
        }

        // Convert Array2 to separate real/imag matrices for serialization
        let mut matrix_real = Vec::new();
        let mut matrix_imag = Vec::new();
        for i in 0..dim {
            let mut real_row = Vec::new();
            let mut imag_row = Vec::new();
            for j in 0..dim {
                real_row.push(matrix[[i, j]].re);
                imag_row.push(matrix[[i, j]].im);
            }
            matrix_real.push(real_row);
            matrix_imag.push(imag_row);
        }

        Ok(ExtendedGate::Custom {
            name,
            matrix_real,
            matrix_imag,
            num_qubits: num_qubits as usize,
            parameters: Vec::new(),
        })
    }

    /// Get matrix from custom gate
    fn get_custom_matrix(&self) -> Result<Array2<Complex64>> {
        match self {
            ExtendedGate::Custom {
                matrix_real,
                matrix_imag,
                ..
            } => {
                let dim = matrix_real.len();
                let mut matrix = Array2::zeros((dim, dim));
                for i in 0..dim {
                    for j in 0..dim {
                        matrix[[i, j]] = Complex64::new(matrix_real[i][j], matrix_imag[i][j]);
                    }
                }
                Ok(matrix)
            }
            _ => Err(MyQuatError::circuit_error("Not a custom gate")),
        }
    }
}

impl GateOperation for ExtendedGate {
    fn name(&self) -> &str {
        match self {
            ExtendedGate::SqrtX => "sqrt_x",
            ExtendedGate::SqrtY => "sqrt_y",
            ExtendedGate::SqrtZ => "sqrt_z",
            ExtendedGate::SqrtH => "sqrt_h",
            ExtendedGate::R(_, _, _) => "r_axis",
            ExtendedGate::GlobalPhase(_) => "global_phase",
            ExtendedGate::PhaseShift(_) => "phase_shift",
            ExtendedGate::RXX(_) => "rxx",
            ExtendedGate::RYY(_) => "ryy",
            ExtendedGate::RZZ(_) => "rzz",
            ExtendedGate::RZX(_) => "rzx",
            ExtendedGate::QFT(_) => "qft",
            ExtendedGate::IQFT(_) => "iqft",
            ExtendedGate::Custom { name, .. } => name,
        }
    }

    fn num_qubits(&self) -> usize {
        match self {
            ExtendedGate::SqrtX
            | ExtendedGate::SqrtY
            | ExtendedGate::SqrtZ
            | ExtendedGate::SqrtH
            | ExtendedGate::R(_, _, _)
            | ExtendedGate::GlobalPhase(_)
            | ExtendedGate::PhaseShift(_) => 1,

            ExtendedGate::RXX(_)
            | ExtendedGate::RYY(_)
            | ExtendedGate::RZZ(_)
            | ExtendedGate::RZX(_) => 2,

            ExtendedGate::QFT(n) | ExtendedGate::IQFT(n) => *n,
            ExtendedGate::Custom { num_qubits, .. } => *num_qubits,
        }
    }

    fn num_parameters(&self) -> usize {
        match self {
            ExtendedGate::SqrtX
            | ExtendedGate::SqrtY
            | ExtendedGate::SqrtZ
            | ExtendedGate::SqrtH => 0,

            ExtendedGate::R(_, _, _) => 3,
            ExtendedGate::GlobalPhase(_)
            | ExtendedGate::PhaseShift(_)
            | ExtendedGate::RXX(_)
            | ExtendedGate::RYY(_)
            | ExtendedGate::RZZ(_)
            | ExtendedGate::RZX(_) => 1,

            ExtendedGate::QFT(_) | ExtendedGate::IQFT(_) => 0,
            ExtendedGate::Custom { parameters, .. } => parameters.len(),
        }
    }

    fn matrix(
        &self,
        _params: &[Parameter],
        _symbols: &HashMap<String, f64>,
    ) -> Result<Array2<Complex64>> {
        match self {
            ExtendedGate::SqrtX => {
                let mut matrix = Array2::zeros((2, 2));
                matrix[[0, 0]] = Complex64::new(0.5, 0.5);
                matrix[[0, 1]] = Complex64::new(0.5, -0.5);
                matrix[[1, 0]] = Complex64::new(0.5, -0.5);
                matrix[[1, 1]] = Complex64::new(0.5, 0.5);
                Ok(matrix)
            }

            ExtendedGate::SqrtY => {
                let mut matrix = Array2::zeros((2, 2));
                matrix[[0, 0]] = Complex64::new(0.5, 0.5);
                matrix[[0, 1]] = Complex64::new(-0.5, -0.5);
                matrix[[1, 0]] = Complex64::new(0.5, 0.5);
                matrix[[1, 1]] = Complex64::new(0.5, 0.5);
                Ok(matrix)
            }

            ExtendedGate::R(nx, ny, nz) => {
                // Arbitrary axis rotation using Rodrigues formula
                let norm = (nx * nx + ny * ny + nz * nz).sqrt();
                if norm < 1e-10 {
                    // Identity for zero rotation
                    Ok(Array2::eye(2).mapv(|x| Complex64::new(x, 0.0)))
                } else {
                    let angle = norm;
                    let nx_norm = nx / norm;
                    let ny_norm = ny / norm;
                    let nz_norm = nz / norm;
                    Ok(arbitrary_rotation_matrix(nx_norm, ny_norm, nz_norm, angle))
                }
            }

            ExtendedGate::RXX(angle) => {
                let cos_half = (angle / 2.0).cos();
                let sin_half = (angle / 2.0).sin();
                let mut matrix = Array2::zeros((4, 4));

                matrix[[0, 0]] = Complex64::new(cos_half, 0.0);
                matrix[[0, 3]] = Complex64::new(0.0, -sin_half);
                matrix[[1, 1]] = Complex64::new(cos_half, 0.0);
                matrix[[1, 2]] = Complex64::new(0.0, -sin_half);
                matrix[[2, 1]] = Complex64::new(0.0, -sin_half);
                matrix[[2, 2]] = Complex64::new(cos_half, 0.0);
                matrix[[3, 0]] = Complex64::new(0.0, -sin_half);
                matrix[[3, 3]] = Complex64::new(cos_half, 0.0);

                Ok(matrix)
            }

            ExtendedGate::Custom { .. } => self.get_custom_matrix(),

            _ => Err(MyQuatError::circuit_error(
                "Gate matrix not implemented yet",
            )),
        }
    }

    fn matrix_with_backend<B: LinalgBackend<Scalar = Complex64>>(
        &self,
        params: &[Parameter],
        symbols: &HashMap<String, f64>,
        backend: &B,
    ) -> LinalgResult<B::Matrix> {
        let nda = self
            .matrix(params, symbols)
            .map_err(|e| crate::linalg::LinalgError::BackendError(e.to_string()))?;
        let (r, c) = nda.dim();
        let data: Vec<Complex64> = nda.iter().copied().collect();
        backend.from_shape_vec(r, c, data)
    }
}

/// Gate builder for creating composite gates
pub struct GateBuilder {
    gates: Vec<(ExtendedGate, Vec<usize>)>,
    num_qubits: usize,
}

impl GateBuilder {
    /// Create a new GateBuilder for a circuit of `num_qubits` qubits.
    pub fn new(num_qubits: usize) -> Self {
        GateBuilder {
            gates: Vec::new(),
            num_qubits,
        }
    }

    /// Append a gate acting on the given qubits.
    ///
    /// Returns `self` for chaining. Returns an error if the number of qubits
    /// does not match the gate's arity or any index is out of bounds.
    pub fn add_gate(&mut self, gate: ExtendedGate, qubits: Vec<usize>) -> Result<&mut Self> {
        if qubits.len() != gate.num_qubits() {
            return Err(MyQuatError::circuit_error("Qubit count mismatch"));
        }

        for &qubit in &qubits {
            if qubit >= self.num_qubits {
                return Err(MyQuatError::circuit_error("Qubit index out of range"));
            }
        }

        self.gates.push((gate, qubits));
        Ok(self)
    }

    /// Assemble all gates added so far into a single composite `ExtendedGate::Custom`.
    ///
    /// The result is a placeholder identity computation; a full implementation
    /// would multiply the constituent matrices in order.
    pub fn build(self, name: String) -> Result<ExtendedGate> {
        // For now, return a placeholder - full implementation would compute composite matrix
        let dim = 1 << self.num_qubits;
        let matrix = Array2::eye(dim).mapv(|x| Complex64::new(x, 0.0));

        // Convert Array2 to separate real/imag matrices
        let mut matrix_real = Vec::new();
        let mut matrix_imag = Vec::new();
        for i in 0..dim {
            let mut real_row = Vec::new();
            let mut imag_row = Vec::new();
            for j in 0..dim {
                real_row.push(matrix[[i, j]].re);
                imag_row.push(matrix[[i, j]].im);
            }
            matrix_real.push(real_row);
            matrix_imag.push(imag_row);
        }

        Ok(ExtendedGate::Custom {
            name,
            matrix_real,
            matrix_imag,
            num_qubits: self.num_qubits,
            parameters: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_sqrt_x_gate() {
        let gate = ExtendedGate::sqrt_x();
        assert_eq!(gate.name(), "sqrt_x");
        assert_eq!(gate.num_qubits(), 1);
        assert_eq!(gate.num_parameters(), 0);

        let matrix = gate.matrix(&[], &HashMap::new()).unwrap();
        assert_eq!(matrix.dim(), (2, 2));

        // Verify that sqrt_x^2 = X
        let x_matrix = &matrix.dot(&matrix);
        assert_relative_eq!(x_matrix[[0, 1]].re, 1.0, epsilon = 1e-10);
        assert_relative_eq!(x_matrix[[1, 0]].re, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_rxx_gate() {
        let angle = PI / 4.0;
        let gate = ExtendedGate::rxx(angle);
        assert_eq!(gate.name(), "rxx");
        assert_eq!(gate.num_qubits(), 2);
        assert_eq!(gate.num_parameters(), 1);

        let matrix = gate.matrix(&[], &HashMap::new()).unwrap();
        assert_eq!(matrix.dim(), (4, 4));
    }

    #[test]
    fn test_custom_gate() {
        let matrix = Array2::eye(2).mapv(|x| Complex64::new(x, 0.0));
        let gate = ExtendedGate::custom("identity".to_string(), matrix).unwrap();

        assert_eq!(gate.name(), "identity");
        assert_eq!(gate.num_qubits(), 1);
    }

    #[test]
    fn test_gate_builder() {
        let mut builder = GateBuilder::new(2);
        builder.add_gate(ExtendedGate::sqrt_x(), vec![0]).unwrap();
        builder.add_gate(ExtendedGate::sqrt_y(), vec![1]).unwrap();

        let composite = builder.build("test_composite".to_string()).unwrap();
        assert_eq!(composite.name(), "test_composite");
        assert_eq!(composite.num_qubits(), 2);
    }
}
