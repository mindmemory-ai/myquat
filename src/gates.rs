//! Quantum gate definitions and operations
//!
//! This module provides the core quantum gate functionality, including
//! standard gates, custom gates, and gate matrices.

use crate::error::{MyQuatError, Result};
use crate::linalg::{LinalgBackend, LinalgResult, NdArrayBackend};
use crate::parameter::Parameter;
use crate::{constants::*, Complex};
use ndarray::Array2;
use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Trait for quantum gate operations
pub trait GateOperation {
    /// Get the name of the gate
    fn name(&self) -> &str;

    /// Get the number of qubits this gate acts on
    fn num_qubits(&self) -> usize;

    /// Get the number of parameters this gate requires
    fn num_parameters(&self) -> usize;

    /// Get the unitary matrix representation of the gate
    fn matrix(
        &self,
        params: &[Parameter],
        symbols: &HashMap<String, f64>,
    ) -> Result<Array2<Complex64>>;

    /// Get the gate matrix using a linear algebra backend.
    /// The default implementation converts via ndarray; override for native performance.
    fn matrix_with_backend<B: LinalgBackend<Scalar = Complex64>>(
        &self,
        params: &[Parameter],
        symbols: &HashMap<String, f64>,
        backend: &B,
    ) -> LinalgResult<B::Matrix>
    where
        Self: Sized;

    /// Check if this gate is its own inverse
    fn is_self_inverse(&self) -> bool {
        false
    }

    /// Get the inverse of this gate, if it exists
    fn inverse(&self) -> Option<Box<dyn GateOperation>> {
        None
    }

    /// Check if this gate commutes with another gate
    fn commutes_with(&self, _other: &dyn GateOperation) -> bool {
        false
    }
}

/// Standard quantum gates enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StandardGate {
    // Single-qubit gates
    I,   // Identity
    X,   // Pauli-X (NOT)
    Y,   // Pauli-Y
    Z,   // Pauli-Z
    H,   // Hadamard
    S,   // S gate (phase)
    Sdg, // S dagger
    T,   // T gate
    Tdg, // T dagger
    Rx,  // Rotation around X-axis
    Ry,  // Rotation around Y-axis
    Rz,  // Rotation around Z-axis
    P,   // Phase gate
    U,   // Universal single-qubit gate (3 parameters)
    U1,  // Single-parameter U gate
    U2,  // Two-parameter U gate
    U3,  // Three-parameter U gate

    // Two-qubit gates
    CX,    // CNOT
    CY,    // Controlled-Y
    CZ,    // Controlled-Z
    CH,    // Controlled-Hadamard
    CRx,   // Controlled rotation around X
    CRy,   // Controlled rotation around Y
    CRz,   // Controlled rotation around Z
    CP,    // Controlled phase
    Swap,  // Swap gate
    ISwap, // iSWAP gate

    // Three-qubit gates
    CCX,   // Toffoli (CCNOT)
    CSwap, // Fredkin (controlled-SWAP)

    // Multi-qubit gates
    MCX, // Multi-controlled X
    MCY, // Multi-controlled Y
    MCZ, // Multi-controlled Z
}

impl StandardGate {
    /// Get all standard gates
    pub fn all_gates() -> Vec<StandardGate> {
        vec![
            StandardGate::I,
            StandardGate::X,
            StandardGate::Y,
            StandardGate::Z,
            StandardGate::H,
            StandardGate::S,
            StandardGate::Sdg,
            StandardGate::T,
            StandardGate::Tdg,
            StandardGate::Rx,
            StandardGate::Ry,
            StandardGate::Rz,
            StandardGate::P,
            StandardGate::U,
            StandardGate::U1,
            StandardGate::U2,
            StandardGate::U3,
            StandardGate::CX,
            StandardGate::CY,
            StandardGate::CZ,
            StandardGate::CH,
            StandardGate::CRx,
            StandardGate::CRy,
            StandardGate::CRz,
            StandardGate::CP,
            StandardGate::Swap,
            StandardGate::ISwap,
            StandardGate::CCX,
            StandardGate::CSwap,
            StandardGate::MCX,
            StandardGate::MCY,
            StandardGate::MCZ,
        ]
    }

    /// Get the inverse of this gate
    pub fn inverse(&self) -> StandardGate {
        match self {
            StandardGate::S => StandardGate::Sdg,
            StandardGate::Sdg => StandardGate::S,
            StandardGate::T => StandardGate::Tdg,
            StandardGate::Tdg => StandardGate::T,
            // Self-inverse gates
            StandardGate::I
            | StandardGate::X
            | StandardGate::Y
            | StandardGate::Z
            | StandardGate::H
            | StandardGate::CX
            | StandardGate::CY
            | StandardGate::CZ
            | StandardGate::CH
            | StandardGate::Swap
            | StandardGate::CCX
            | StandardGate::CSwap => *self,
            // Parametric gates - inverse requires parameter negation
            _ => *self, // For now, return the same gate (proper handling would negate parameters)
        }
    }
}

impl GateOperation for StandardGate {
    fn name(&self) -> &str {
        match self {
            StandardGate::I => "i",
            StandardGate::X => "x",
            StandardGate::Y => "y",
            StandardGate::Z => "z",
            StandardGate::H => "h",
            StandardGate::S => "s",
            StandardGate::Sdg => "sdg",
            StandardGate::T => "t",
            StandardGate::Tdg => "tdg",
            StandardGate::Rx => "rx",
            StandardGate::Ry => "ry",
            StandardGate::Rz => "rz",
            StandardGate::P => "p",
            StandardGate::U => "u",
            StandardGate::U1 => "u1",
            StandardGate::U2 => "u2",
            StandardGate::U3 => "u3",
            StandardGate::CX => "cx",
            StandardGate::CY => "cy",
            StandardGate::CZ => "cz",
            StandardGate::CH => "ch",
            StandardGate::CRx => "crx",
            StandardGate::CRy => "cry",
            StandardGate::CRz => "crz",
            StandardGate::CP => "cp",
            StandardGate::Swap => "swap",
            StandardGate::ISwap => "iswap",
            StandardGate::CCX => "ccx",
            StandardGate::CSwap => "cswap",
            StandardGate::MCX => "mcx",
            StandardGate::MCY => "mcy",
            StandardGate::MCZ => "mcz",
        }
    }

    fn num_qubits(&self) -> usize {
        match self {
            StandardGate::I
            | StandardGate::X
            | StandardGate::Y
            | StandardGate::Z
            | StandardGate::H
            | StandardGate::S
            | StandardGate::Sdg
            | StandardGate::T
            | StandardGate::Tdg
            | StandardGate::Rx
            | StandardGate::Ry
            | StandardGate::Rz
            | StandardGate::P
            | StandardGate::U
            | StandardGate::U1
            | StandardGate::U2
            | StandardGate::U3 => 1,

            StandardGate::CX
            | StandardGate::CY
            | StandardGate::CZ
            | StandardGate::CH
            | StandardGate::CRx
            | StandardGate::CRy
            | StandardGate::CRz
            | StandardGate::CP
            | StandardGate::Swap
            | StandardGate::ISwap => 2,

            StandardGate::CCX | StandardGate::CSwap => 3,

            // Multi-qubit gates - variable number, but we'll use a default
            StandardGate::MCX | StandardGate::MCY | StandardGate::MCZ => 3,
        }
    }

    fn num_parameters(&self) -> usize {
        match self {
            StandardGate::Rx
            | StandardGate::Ry
            | StandardGate::Rz
            | StandardGate::P
            | StandardGate::U1
            | StandardGate::CRx
            | StandardGate::CRy
            | StandardGate::CRz
            | StandardGate::CP => 1,

            StandardGate::U2 => 2,
            StandardGate::U | StandardGate::U3 => 3,

            _ => 0,
        }
    }

    fn matrix(
        &self,
        params: &[Parameter],
        symbols: &HashMap<String, f64>,
    ) -> Result<Array2<Complex64>> {
        // Helper function to evaluate parameters
        let eval_param = |idx: usize| -> Result<f64> {
            if idx >= params.len() {
                return Err(MyQuatError::invalid_parameter(format!(
                    "Gate {} requires {} parameters, got {}",
                    self.name(),
                    self.num_parameters(),
                    params.len()
                )));
            }
            params[idx].evaluate(symbols)
        };

        match self {
            StandardGate::I => Ok(Array2::from_shape_vec((2, 2), vec![ONE, ZERO, ZERO, ONE])?),

            StandardGate::X => Ok(Array2::from_shape_vec((2, 2), vec![ZERO, ONE, ONE, ZERO])?),

            StandardGate::Y => Ok(Array2::from_shape_vec((2, 2), vec![ZERO, -I, I, ZERO])?),

            StandardGate::Z => Ok(Array2::from_shape_vec((2, 2), vec![ONE, ZERO, ZERO, -ONE])?),

            StandardGate::H => {
                let inv_sqrt2 = Complex::new(INV_SQRT2, 0.0);
                Ok(Array2::from_shape_vec(
                    (2, 2),
                    vec![inv_sqrt2, inv_sqrt2, inv_sqrt2, -inv_sqrt2],
                )?)
            }

            StandardGate::S => Ok(Array2::from_shape_vec((2, 2), vec![ONE, ZERO, ZERO, I])?),

            StandardGate::Sdg => Ok(Array2::from_shape_vec((2, 2), vec![ONE, ZERO, ZERO, -I])?),

            StandardGate::T => {
                let t_phase = Complex::new(INV_SQRT2, INV_SQRT2);
                Ok(Array2::from_shape_vec(
                    (2, 2),
                    vec![ONE, ZERO, ZERO, t_phase],
                )?)
            }

            StandardGate::Tdg => {
                let t_phase = Complex::new(INV_SQRT2, -INV_SQRT2);
                Ok(Array2::from_shape_vec(
                    (2, 2),
                    vec![ONE, ZERO, ZERO, t_phase],
                )?)
            }

            StandardGate::Rx => {
                let theta = eval_param(0)?;
                let cos_half = Complex::new((theta / 2.0).cos(), 0.0);
                let sin_half = Complex::new(0.0, -(theta / 2.0).sin());
                Ok(Array2::from_shape_vec(
                    (2, 2),
                    vec![cos_half, sin_half, sin_half, cos_half],
                )?)
            }

            StandardGate::Ry => {
                let theta = eval_param(0)?;
                let cos_half = Complex::new((theta / 2.0).cos(), 0.0);
                let sin_half = Complex::new((theta / 2.0).sin(), 0.0);
                Ok(Array2::from_shape_vec(
                    (2, 2),
                    vec![cos_half, -sin_half, sin_half, cos_half],
                )?)
            }

            StandardGate::Rz => {
                let theta = eval_param(0)?;
                let exp_neg = Complex::new(0.0, -theta / 2.0).exp();
                let exp_pos = Complex::new(0.0, theta / 2.0).exp();
                Ok(Array2::from_shape_vec(
                    (2, 2),
                    vec![exp_neg, ZERO, ZERO, exp_pos],
                )?)
            }

            StandardGate::P => {
                let phi = eval_param(0)?;
                let phase = Complex::new(0.0, phi).exp();
                Ok(Array2::from_shape_vec(
                    (2, 2),
                    vec![ONE, ZERO, ZERO, phase],
                )?)
            }

            StandardGate::U3 | StandardGate::U => {
                let theta = eval_param(0)?;
                let phi = eval_param(1)?;
                let lambda = eval_param(2)?;

                let cos_half = (theta / 2.0).cos();
                let sin_half = (theta / 2.0).sin();
                let exp_phi = Complex::new(0.0, phi).exp();
                let exp_lambda = Complex::new(0.0, lambda).exp();
                let exp_phi_lambda = Complex::new(0.0, phi + lambda).exp();

                Ok(Array2::from_shape_vec(
                    (2, 2),
                    vec![
                        Complex::new(cos_half, 0.0),
                        -exp_lambda * sin_half,
                        exp_phi * sin_half,
                        exp_phi_lambda * cos_half,
                    ],
                )?)
            }

            StandardGate::CX => Ok(Array2::from_shape_vec(
                (4, 4),
                vec![
                    ONE, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO,
                    ZERO, ONE, ZERO,
                ],
            )?),

            StandardGate::CZ => Ok(Array2::from_shape_vec(
                (4, 4),
                vec![
                    ONE, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO,
                    ZERO, ZERO, -ONE,
                ],
            )?),

            StandardGate::Swap => Ok(Array2::from_shape_vec(
                (4, 4),
                vec![
                    ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ONE, ZERO, ZERO, ZERO,
                    ZERO, ZERO, ONE,
                ],
            )?),

            StandardGate::ISwap => Ok(Array2::from_shape_vec(
                (4, 4),
                vec![
                    ONE, ZERO, ZERO, ZERO, ZERO, ZERO, I, ZERO, ZERO, I, ZERO, ZERO, ZERO, ZERO,
                    ZERO, ONE,
                ],
            )?),

            StandardGate::CY => Ok(Array2::from_shape_vec(
                (4, 4),
                vec![
                    ONE, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO, -I, ZERO, ZERO,
                    I, ZERO,
                ],
            )?),

            StandardGate::CH => {
                let inv_sqrt2 = Complex::new(INV_SQRT2, 0.0);
                Ok(Array2::from_shape_vec(
                    (4, 4),
                    vec![
                        ONE, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, inv_sqrt2,
                        inv_sqrt2, ZERO, ZERO, inv_sqrt2, -inv_sqrt2,
                    ],
                )?)
            }

            StandardGate::CRx => {
                let theta = eval_param(0)?;
                let cos_half = Complex::new((theta / 2.0).cos(), 0.0);
                let sin_half = Complex::new(0.0, -(theta / 2.0).sin());
                Ok(Array2::from_shape_vec(
                    (4, 4),
                    vec![
                        ONE, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, cos_half,
                        sin_half, ZERO, ZERO, sin_half, cos_half,
                    ],
                )?)
            }

            StandardGate::CRy => {
                let theta = eval_param(0)?;
                let cos_half = Complex::new((theta / 2.0).cos(), 0.0);
                let sin_half = Complex::new((theta / 2.0).sin(), 0.0);
                Ok(Array2::from_shape_vec(
                    (4, 4),
                    vec![
                        ONE, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, cos_half,
                        -sin_half, ZERO, ZERO, sin_half, cos_half,
                    ],
                )?)
            }

            StandardGate::CRz => {
                let theta = eval_param(0)?;
                let exp_neg = Complex::new(0.0, -theta / 2.0).exp();
                let exp_pos = Complex::new(0.0, theta / 2.0).exp();
                Ok(Array2::from_shape_vec(
                    (4, 4),
                    vec![
                        ONE, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, exp_neg, ZERO,
                        ZERO, ZERO, ZERO, exp_pos,
                    ],
                )?)
            }

            StandardGate::CP => {
                let phi = eval_param(0)?;
                let phase = Complex::new(0.0, phi).exp();
                Ok(Array2::from_shape_vec(
                    (4, 4),
                    vec![
                        ONE, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO,
                        ZERO, ZERO, phase,
                    ],
                )?)
            }

            StandardGate::U1 => {
                let lambda = eval_param(0)?;
                let phase = Complex::new(0.0, lambda).exp();
                Ok(Array2::from_shape_vec(
                    (2, 2),
                    vec![ONE, ZERO, ZERO, phase],
                )?)
            }

            StandardGate::U2 => {
                let phi = eval_param(0)?;
                let lambda = eval_param(1)?;
                let inv_sqrt2 = Complex::new(INV_SQRT2, 0.0);
                let exp_phi = Complex::new(0.0, phi).exp();
                let exp_lambda = Complex::new(0.0, lambda).exp();
                let exp_phi_lambda = Complex::new(0.0, phi + lambda).exp();

                Ok(Array2::from_shape_vec(
                    (2, 2),
                    vec![
                        inv_sqrt2,
                        -exp_lambda * inv_sqrt2,
                        exp_phi * inv_sqrt2,
                        exp_phi_lambda * inv_sqrt2,
                    ],
                )?)
            }

            StandardGate::CCX => Ok(Array2::from_shape_vec(
                (8, 8),
                vec![
                    ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO,
                    ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO,
                    ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO,
                    ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO,
                    ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO,
                ],
            )?),

            StandardGate::CSwap => Ok(Array2::from_shape_vec(
                (8, 8),
                vec![
                    ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO,
                    ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO,
                    ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO,
                    ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO,
                    ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ONE,
                ],
            )?),

            // Multi-controlled gates - simplified implementation for 3 qubits
            StandardGate::MCX | StandardGate::MCY | StandardGate::MCZ => {
                // For now, implement as 3-qubit gates (2 controls + 1 target)
                // This is a simplified version - full implementation would be dynamic
                match self {
                    StandardGate::MCX => {
                        // Same as CCX for 3-qubit case
                        Ok(Array2::from_shape_vec(
                            (8, 8),
                            vec![
                                ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO,
                                ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO,
                                ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO,
                                ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO,
                                ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO,
                                ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO,
                            ],
                        )?)
                    }
                    StandardGate::MCY => {
                        // Multi-controlled Y gate
                        Ok(Array2::from_shape_vec(
                            (8, 8),
                            vec![
                                ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO,
                                ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO,
                                ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO,
                                ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO,
                                ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO,
                                -I, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, I, ZERO,
                            ],
                        )?)
                    }
                    StandardGate::MCZ => {
                        // Multi-controlled Z gate
                        Ok(Array2::from_shape_vec(
                            (8, 8),
                            vec![
                                ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO,
                                ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO,
                                ZERO, ZERO, ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO,
                                ZERO, ZERO, ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO,
                                ZERO, ONE, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ONE,
                                ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, ZERO, -ONE,
                            ],
                        )?)
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    fn matrix_with_backend<B: LinalgBackend<Scalar = Complex64>>(
        &self,
        params: &[Parameter],
        symbols: &HashMap<String, f64>,
        backend: &B,
    ) -> LinalgResult<B::Matrix> {
        // Delegate to the ndarray-based matrix() then convert via backend.
        // Gate matrices are small (2x2, 4x4, 8x8) so the copy is negligible.
        let nda = self
            .matrix(params, symbols)
            .map_err(|e| crate::linalg::LinalgError::BackendError(e.to_string()))?;
        let (r, c) = nda.dim();
        let data: Vec<Complex64> = nda.iter().copied().collect();
        backend.from_shape_vec(r, c, data)
    }

    fn is_self_inverse(&self) -> bool {
        matches!(
            self,
            StandardGate::I
                | StandardGate::X
                | StandardGate::Y
                | StandardGate::Z
                | StandardGate::H
                | StandardGate::CX
                | StandardGate::CY
                | StandardGate::CZ
                | StandardGate::CH
                | StandardGate::Swap
                | StandardGate::CCX
                | StandardGate::CSwap
        )
    }
}

impl fmt::Display for StandardGate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// A concrete gate instance with parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate {
    /// The gate type
    pub gate_type: StandardGate,
    /// Parameters for the gate
    pub parameters: Vec<Parameter>,
    /// Optional label for the gate
    pub label: Option<String>,
}

impl Gate {
    /// Create a new gate instance
    pub fn new(gate_type: StandardGate, parameters: Vec<Parameter>) -> Result<Self> {
        if parameters.len() != gate_type.num_parameters() {
            return Err(MyQuatError::invalid_parameter(format!(
                "Gate {} requires {} parameters, got {}",
                gate_type.name(),
                gate_type.num_parameters(),
                parameters.len()
            )));
        }

        Ok(Gate {
            gate_type,
            parameters,
            label: None,
        })
    }

    /// Create a new gate with a label
    pub fn new_with_label(
        gate_type: StandardGate,
        parameters: Vec<Parameter>,
        label: String,
    ) -> Result<Self> {
        let mut gate = Self::new(gate_type, parameters)?;
        gate.label = Some(label);
        Ok(gate)
    }

    /// Get the gate matrix
    pub fn matrix(&self, symbols: &HashMap<String, f64>) -> Result<Array2<Complex64>> {
        self.gate_type.matrix(&self.parameters, symbols)
    }

    /// Get the gate matrix using the provided linear algebra backend
    pub fn matrix_with_backend<B: LinalgBackend<Scalar = Complex64>>(
        &self,
        symbols: &HashMap<String, f64>,
        backend: &B,
    ) -> LinalgResult<B::Matrix> {
        self.gate_type
            .matrix_with_backend(&self.parameters, symbols, backend)
    }

    /// Check if this gate is parametric (has symbolic parameters)
    pub fn is_parametric(&self) -> bool {
        self.parameters.iter().any(|p| !p.is_numeric())
    }

    /// Get all symbols used in this gate's parameters
    pub fn symbols(&self) -> Vec<String> {
        let mut all_symbols = Vec::new();
        for param in &self.parameters {
            all_symbols.extend(param.symbols());
        }
        all_symbols.sort();
        all_symbols.dedup();
        all_symbols
    }
}

/// Convenience functions for creating common gates
impl Gate {
    /// Identity gate.
    ///
    /// $$ I = \begin{pmatrix} 1 & 0 \\ 0 & 1 \end{pmatrix} $$
    ///
    /// Leaves the qubit state unchanged: $I|\psi\rangle = |\psi\rangle$.
    pub fn i() -> Self {
        Gate::new(StandardGate::I, vec![]).unwrap()
    }

    /// Pauli-X gate (NOT gate, bit-flip).
    ///
    /// $$ X = \begin{pmatrix} 0 & 1 \\ 1 & 0 \end{pmatrix} $$
    ///
    /// Action: $X|0\rangle = |1\rangle$, $X|1\rangle = |0\rangle$.
    /// Self-inverse: $X^2 = I$.
    pub fn x() -> Self {
        Gate::new(StandardGate::X, vec![]).unwrap()
    }

    /// Pauli-Y gate.
    ///
    /// $$ Y = \begin{pmatrix} 0 & -i \\ i & 0 \end{pmatrix} $$
    ///
    /// Action: $Y|0\rangle = i|1\rangle$, $Y|1\rangle = -i|0\rangle$.
    /// Self-inverse: $Y^2 = I$.
    pub fn y() -> Self {
        Gate::new(StandardGate::Y, vec![]).unwrap()
    }

    /// Pauli-Z gate (phase-flip).
    ///
    /// $$ Z = \begin{pmatrix} 1 & 0 \\ 0 & -1 \end{pmatrix} $$
    ///
    /// Action: $Z|0\rangle = |0\rangle$, $Z|1\rangle = -|1\rangle$.
    /// Self-inverse: $Z^2 = I$.
    pub fn z() -> Self {
        Gate::new(StandardGate::Z, vec![]).unwrap()
    }

    /// Hadamard gate.
    ///
    /// $$ H = \frac{1}{\sqrt{2}}\begin{pmatrix} 1 & 1 \\ 1 & -1 \end{pmatrix} $$
    ///
    /// Creates equal superposition: $H|0\rangle = |+\rangle$, $H|1\rangle = |-\rangle$.
    /// Self-inverse: $H^2 = I$.
    pub fn h() -> Self {
        Gate::new(StandardGate::H, vec![]).unwrap()
    }

    /// S gate (phase gate, $\sqrt{Z}$).
    ///
    /// $$ S = \begin{pmatrix} 1 & 0 \\ 0 & i \end{pmatrix} $$
    ///
    /// Adds a $\pi/2$ phase: $S|0\rangle = |0\rangle$, $S|1\rangle = i|1\rangle$.
    /// $S^2 = Z$, $S^4 = I$.
    pub fn s() -> Self {
        Gate::new(StandardGate::S, vec![]).unwrap()
    }

    /// S-dagger gate (inverse of $S$, $S^\dagger$).
    ///
    /// $$ S^\dagger = \begin{pmatrix} 1 & 0 \\ 0 & -i \end{pmatrix} $$
    ///
    /// Adds a $-\pi/2$ phase to $|1\rangle$. Inverse of `s()`.
    pub fn sdg() -> Self {
        Gate::new(StandardGate::Sdg, vec![]).unwrap()
    }

    /// T gate ($\pi/8$ gate, $\sqrt\[4\]{Z}$).
    ///
    /// $$ T = \begin{pmatrix} 1 & 0 \\ 0 & e^{i\pi/4} \end{pmatrix} $$
    ///
    /// Adds a $\pi/4$ phase: $T|1\rangle = e^{i\pi/4}|1\rangle$.
    /// $T^2 = S$, $T^4 = Z$.
    pub fn t() -> Self {
        Gate::new(StandardGate::T, vec![]).unwrap()
    }

    /// T-dagger gate (inverse of $T$, $T^\dagger$).
    ///
    /// $$ T^\dagger = \begin{pmatrix} 1 & 0 \\ 0 & e^{-i\pi/4} \end{pmatrix} $$
    ///
    /// Adds a $-\pi/4$ phase to $|1\rangle$. Inverse of `t()`.
    pub fn tdg() -> Self {
        Gate::new(StandardGate::Tdg, vec![]).unwrap()
    }

    /// Rotation around the X axis by angle $\theta$.
    ///
    /// $$ R_x(\theta) = \exp\left(-i\frac{\theta}{2}X\right) =
    /// \begin{pmatrix}
    /// \cos\frac{\theta}{2} & -i\sin\frac{\theta}{2} \\
    /// -i\sin\frac{\theta}{2} & \cos\frac{\theta}{2}
    /// \end{pmatrix} $$
    ///
    /// # Parameters
    /// - `theta`: rotation angle $\theta$ in radians.
    ///   $R_x(\pi) = -iX$. Identity when $\theta = 0$.
    pub fn rx(theta: Parameter) -> Self {
        Gate::new(StandardGate::Rx, vec![theta]).unwrap()
    }

    /// Rotation around the Y axis by angle $\theta$.
    ///
    /// $$ R_y(\theta) = \exp\left(-i\frac{\theta}{2}Y\right) =
    /// \begin{pmatrix}
    /// \cos\frac{\theta}{2} & -\sin\frac{\theta}{2} \\
    /// \sin\frac{\theta}{2} & \cos\frac{\theta}{2}
    /// \end{pmatrix} $$
    ///
    /// # Parameters
    /// - `theta`: rotation angle $\theta$ in radians.
    ///   $R_y(\pi) = -iY$. Identity when $\theta = 0$.
    pub fn ry(theta: Parameter) -> Self {
        Gate::new(StandardGate::Ry, vec![theta]).unwrap()
    }

    /// Rotation around the Z axis by angle $\theta$.
    ///
    /// $$ R_z(\theta) = \exp\left(-i\frac{\theta}{2}Z\right) =
    /// \begin{pmatrix}
    /// e^{-i\theta/2} & 0 \\
    /// 0 & e^{i\theta/2}
    /// \end{pmatrix} $$
    ///
    /// # Parameters
    /// - `theta`: rotation angle $\theta$ in radians.
    ///   $R_z(\pi) = -iZ = S^2$. Identity when $\theta = 0$.
    pub fn rz(theta: Parameter) -> Self {
        Gate::new(StandardGate::Rz, vec![theta]).unwrap()
    }

    /// Phase gate $P(\phi)$ (also known as $U_1(\phi)$, $R_z(\phi)$ up to global phase).
    ///
    /// $$ P(\phi) = \begin{pmatrix} 1 & 0 \\ 0 & e^{i\phi} \end{pmatrix} $$
    ///
    /// # Parameters
    /// - `phi`: phase angle $\phi$ in radians.
    ///   $P(\pi/2) = S$, $P(\pi/4) = T$, $P(\pi) = Z$.
    pub fn p(phi: Parameter) -> Self {
        Gate::new(StandardGate::P, vec![phi]).unwrap()
    }

    /// Universal single-qubit gate $U(\theta, \phi, \lambda)$ (Qiskit convention).
    ///
    /// $$ U(\theta, \phi, \lambda) = \begin{pmatrix}
    /// \cos\frac{\theta}{2} & -e^{i\lambda}\sin\frac{\theta}{2} \\
    /// e^{i\phi}\sin\frac{\theta}{2} & e^{i(\phi+\lambda)}\cos\frac{\theta}{2}
    /// \end{pmatrix} $$
    ///
    /// Alias for `u3()`. Any single-qubit unitary can be written in this form.
    ///
    /// # Parameters
    /// - `theta`: polar angle $\theta \in [0, \pi]$
    /// - `phi`: azimuthal angle $\phi \in [0, 2\pi)$
    /// - `lambda`: phase angle $\lambda \in [0, 2\pi)$
    pub fn u(theta: Parameter, phi: Parameter, lambda: Parameter) -> Self {
        Gate::new(StandardGate::U, vec![theta, phi, lambda]).unwrap()
    }

    /// Single-parameter U gate $U_1(\lambda)$ (phase gate).
    ///
    /// $$ U_1(\lambda) = P(\lambda) = \begin{pmatrix} 1 & 0 \\ 0 & e^{i\lambda} \end{pmatrix} $$
    ///
    /// Equivalent to `p(lambda)` and $R_z(\lambda)$ up to global phase.
    ///
    /// # Parameters
    /// - `lambda`: phase angle $\lambda$ in radians
    pub fn u1(lambda: Parameter) -> Self {
        Gate::new(StandardGate::U1, vec![lambda]).unwrap()
    }

    /// Two-parameter U gate $U_2(\phi, \lambda)$ (Qiskit convention).
    ///
    /// $$ U_2(\phi, \lambda) = \frac{1}{\sqrt{2}}\begin{pmatrix}
    /// 1 & -e^{i\lambda} \\
    /// e^{i\phi} & e^{i(\phi+\lambda)}
    /// \end{pmatrix} $$
    ///
    /// Equivalent to $U(\pi/2, \phi, \lambda)$.
    ///
    /// # Parameters
    /// - `phi`: phase angle $\phi$ in $e^{i\phi}$ on the lower-left entry
    /// - `lambda`: phase angle $\lambda$ on the upper-right entry
    pub fn u2(phi: Parameter, lambda: Parameter) -> Self {
        Gate::new(StandardGate::U2, vec![phi, lambda]).unwrap()
    }

    /// Three-parameter U gate $U_3(\theta, \phi, \lambda)$ (Qiskit convention).
    ///
    /// $$ U_3(\theta, \phi, \lambda) = \begin{pmatrix}
    /// \cos\frac{\theta}{2} & -e^{i\lambda}\sin\frac{\theta}{2} \\
    /// e^{i\phi}\sin\frac{\theta}{2} & e^{i(\phi+\lambda)}\cos\frac{\theta}{2}
    /// \end{pmatrix} $$
    ///
    /// The most general single-qubit unitary (up to global phase).
    /// Same as `u()`.
    ///
    /// # Parameters
    /// - `theta`: polar angle $\theta \in [0, \pi]$
    /// - `phi`: azimuthal angle $\phi \in [0, 2\pi)$
    /// - `lambda`: phase angle $\lambda \in [0, 2\pi)$
    pub fn u3(theta: Parameter, phi: Parameter, lambda: Parameter) -> Self {
        Gate::new(StandardGate::U3, vec![theta, phi, lambda]).unwrap()
    }

    /// Controlled-NOT gate (CNOT, CX).
    ///
    /// $$ \text{CX} = \begin{pmatrix}
    /// 1 & 0 & 0 & 0 \\
    /// 0 & 1 & 0 & 0 \\
    /// 0 & 0 & 0 & 1 \\
    /// 0 & 0 & 1 & 0
    /// \end{pmatrix} $$
    ///
    /// Flips the target qubit when the control qubit is $|1\rangle$.
    /// Self-inverse: $\text{CX}^2 = I$.
    /// First argument = control, second = target when applied to a circuit.
    pub fn cx() -> Self {
        Gate::new(StandardGate::CX, vec![]).unwrap()
    }

    /// Alias for `cx()` — Controlled-NOT gate.
    ///
    /// See [`Gate::cx`] for full documentation.
    pub fn cnot() -> Self {
        Gate::new(StandardGate::CX, vec![]).unwrap()
    }

    /// Controlled-Y gate.
    ///
    /// $$ \text{CY} = \begin{pmatrix}
    /// 1 & 0 & 0 & 0 \\
    /// 0 & 1 & 0 & 0 \\
    /// 0 & 0 & 0 & -i \\
    /// 0 & 0 & i & 0
    /// \end{pmatrix} $$
    ///
    /// Applies $Y$ to the target when the control is $|1\rangle$.
    /// Self-inverse: $\text{CY}^2 = I$.
    pub fn cy() -> Self {
        Gate::new(StandardGate::CY, vec![]).unwrap()
    }

    /// Controlled-Z gate (CZ, CSIGN).
    ///
    /// $$ \text{CZ} = \begin{pmatrix}
    /// 1 & 0 & 0 & 0 \\
    /// 0 & 1 & 0 & 0 \\
    /// 0 & 0 & 1 & 0 \\
    /// 0 & 0 & 0 & -1
    /// \end{pmatrix} $$
    ///
    /// Applies $Z$ to the target when the control is $|1\rangle$.
    /// Symmetric (control and target are interchangeable). Self-inverse: $\text{CZ}^2 = I$.
    pub fn cz() -> Self {
        Gate::new(StandardGate::CZ, vec![]).unwrap()
    }

    /// Controlled-Hadamard gate.
    ///
    /// $$ \text{CH} = \begin{pmatrix}
    /// 1 & 0 & 0 & 0 \\
    /// 0 & 1 & 0 & 0 \\
    /// 0 & 0 & \frac{1}{\sqrt{2}} & \frac{1}{\sqrt{2}} \\
    /// 0 & 0 & \frac{1}{\sqrt{2}} & -\frac{1}{\sqrt{2}}
    /// \end{pmatrix} $$
    ///
    /// Applies $H$ to the target when the control is $|1\rangle$.
    /// Self-inverse: $\text{CH}^2 = I$.
    pub fn ch() -> Self {
        Gate::new(StandardGate::CH, vec![]).unwrap()
    }

    /// Controlled rotation around X: $CR_x(\theta)$.
    ///
    /// $$ CR_x(\theta) = \begin{pmatrix}
    /// 1 & 0 & 0 & 0 \\
    /// 0 & 1 & 0 & 0 \\
    /// 0 & 0 & \cos\frac{\theta}{2} & -i\sin\frac{\theta}{2} \\
    /// 0 & 0 & -i\sin\frac{\theta}{2} & \cos\frac{\theta}{2}
    /// \end{pmatrix} $$
    ///
    /// Applies $R_x(\theta)$ to the target when the control is $|1\rangle$.
    ///
    /// # Parameters
    /// - `theta`: rotation angle $\theta$ in radians
    pub fn crx(theta: Parameter) -> Self {
        Gate::new(StandardGate::CRx, vec![theta]).unwrap()
    }

    /// Controlled rotation around Y: $CR_y(\theta)$.
    ///
    /// $$ CR_y(\theta) = \begin{pmatrix}
    /// 1 & 0 & 0 & 0 \\
    /// 0 & 1 & 0 & 0 \\
    /// 0 & 0 & \cos\frac{\theta}{2} & -\sin\frac{\theta}{2} \\
    /// 0 & 0 & \sin\frac{\theta}{2} & \cos\frac{\theta}{2}
    /// \end{pmatrix} $$
    ///
    /// Applies $R_y(\theta)$ to the target when the control is $|1\rangle$.
    ///
    /// # Parameters
    /// - `theta`: rotation angle $\theta$ in radians
    pub fn cry(theta: Parameter) -> Self {
        Gate::new(StandardGate::CRy, vec![theta]).unwrap()
    }

    /// Controlled rotation around Z: $CR_z(\theta)$.
    ///
    /// $$ CR_z(\theta) = \begin{pmatrix}
    /// 1 & 0 & 0 & 0 \\
    /// 0 & 1 & 0 & 0 \\
    /// 0 & 0 & e^{-i\theta/2} & 0 \\
    /// 0 & 0 & 0 & e^{i\theta/2}
    /// \end{pmatrix} $$
    ///
    /// Applies $R_z(\theta)$ to the target when the control is $|1\rangle$.
    ///
    /// # Parameters
    /// - `theta`: rotation angle $\theta$ in radians
    pub fn crz(theta: Parameter) -> Self {
        Gate::new(StandardGate::CRz, vec![theta]).unwrap()
    }

    /// Controlled phase gate $CP(\phi)$.
    ///
    /// $$ CP(\phi) = \begin{pmatrix}
    /// 1 & 0 & 0 & 0 \\
    /// 0 & 1 & 0 & 0 \\
    /// 0 & 0 & 1 & 0 \\
    /// 0 & 0 & 0 & e^{i\phi}
    /// \end{pmatrix} $$
    ///
    /// Applies a phase $e^{i\phi}$ when both qubits are $|11\rangle$.
    /// $CP(\pi) = \text{CZ}$.
    ///
    /// # Parameters
    /// - `phi`: phase angle $\phi$ in radians
    pub fn cp(phi: Parameter) -> Self {
        Gate::new(StandardGate::CP, vec![phi]).unwrap()
    }

    /// SWAP gate.
    ///
    /// $$ \text{SWAP} = \begin{pmatrix}
    /// 1 & 0 & 0 & 0 \\
    /// 0 & 0 & 1 & 0 \\
    /// 0 & 1 & 0 & 0 \\
    /// 0 & 0 & 0 & 1
    /// \end{pmatrix} $$
    ///
    /// Swaps the states of two qubits: $\text{SWAP}|a,b\rangle = |b,a\rangle$.
    /// Self-inverse: $\text{SWAP}^2 = I$.
    pub fn swap() -> Self {
        Gate::new(StandardGate::Swap, vec![]).unwrap()
    }

    /// iSWAP gate.
    ///
    /// $$ \text{iSWAP} = \begin{pmatrix}
    /// 1 & 0 & 0 & 0 \\
    /// 0 & 0 & i & 0 \\
    /// 0 & i & 0 & 0 \\
    /// 0 & 0 & 0 & 1
    /// \end{pmatrix} $$
    ///
    /// Swaps states with an additional $i$ phase.
    /// Native two-qubit gate on many superconducting architectures.
    pub fn iswap() -> Self {
        Gate::new(StandardGate::ISwap, vec![]).unwrap()
    }

    /// Toffoli gate (CCNOT, CCX) — controlled-controlled-NOT.
    ///
    /// $$ \text{CCX} = \begin{pmatrix}
    /// 1 & 0 & 0 & 0 & 0 & 0 & 0 & 0 \\
    /// 0 & 1 & 0 & 0 & 0 & 0 & 0 & 0 \\
    /// 0 & 0 & 1 & 0 & 0 & 0 & 0 & 0 \\
    /// 0 & 0 & 0 & 1 & 0 & 0 & 0 & 0 \\
    /// 0 & 0 & 0 & 0 & 1 & 0 & 0 & 0 \\
    /// 0 & 0 & 0 & 0 & 0 & 1 & 0 & 0 \\
    /// 0 & 0 & 0 & 0 & 0 & 0 & 0 & 1 \\
    /// 0 & 0 & 0 & 0 & 0 & 0 & 1 & 0
    /// \end{pmatrix} $$
    ///
    /// Flips the target if both controls are $|1\rangle$.
    /// Universal for classical reversible computation. Self-inverse.
    pub fn ccx() -> Self {
        Gate::new(StandardGate::CCX, vec![]).unwrap()
    }

    /// Alias for `ccx()` — Toffoli (CCNOT) gate.
    ///
    /// See [`Gate::ccx`] for full documentation.
    pub fn toffoli() -> Self {
        Gate::new(StandardGate::CCX, vec![]).unwrap()
    }

    /// Fredkin gate (CSWAP, controlled-SWAP).
    ///
    /// $$ \text{CSWAP} = \begin{pmatrix}
    /// 1 & 0 & 0 & 0 & 0 & 0 & 0 & 0 \\
    /// 0 & 1 & 0 & 0 & 0 & 0 & 0 & 0 \\
    /// 0 & 0 & 1 & 0 & 0 & 0 & 0 & 0 \\
    /// 0 & 0 & 0 & 1 & 0 & 0 & 0 & 0 \\
    /// 0 & 0 & 0 & 0 & 1 & 0 & 0 & 0 \\
    /// 0 & 0 & 0 & 0 & 0 & 0 & 1 & 0 \\
    /// 0 & 0 & 0 & 0 & 0 & 1 & 0 & 0 \\
    /// 0 & 0 & 0 & 0 & 0 & 0 & 0 & 1
    /// \end{pmatrix} $$
    ///
    /// Swaps two target qubits when the control is $|1\rangle$.
    /// Universal for classical reversible computation. Self-inverse.
    pub fn cswap() -> Self {
        Gate::new(StandardGate::CSwap, vec![]).unwrap()
    }

    /// Alias for `cswap()` — Fredkin (CSWAP) gate.
    ///
    /// See [`Gate::cswap`] for full documentation.
    pub fn fredkin() -> Self {
        Gate::new(StandardGate::CSwap, vec![]).unwrap()
    }

    /// Multi-controlled X gate (generalized Toffoli).
    ///
    /// Applies $X$ (bit-flip) to the target when all control qubits are $|1\rangle$.
    /// For 3 qubits (2 controls + target), equivalent to `ccx()`.
    /// For $n$ qubits, needs $n-1$ controls. Implemented as 3-qubit gate for general use;
    /// decomposable into $O(n^2)$ CNOT + single-qubit gates with ancilla.
    pub fn mcx() -> Self {
        Gate::new(StandardGate::MCX, vec![]).unwrap()
    }

    /// Multi-controlled Y gate.
    ///
    /// Applies $Y$ to the target when all control qubits are $|1\rangle$.
    /// For 3 qubits (2 controls + target), applies the controlled-Y operation
    /// on the lowest two states $|110\rangle$ and $|111\rangle$.
    pub fn mcy() -> Self {
        Gate::new(StandardGate::MCY, vec![]).unwrap()
    }

    /// Multi-controlled Z gate.
    ///
    /// Applies $Z$ (phase-flip) to the target when all control qubits are $|1\rangle$.
    /// For 3 qubits (2 controls + target), applies $-1$ phase to $|111\rangle$.
    /// Symmetric in all qubits (any qubit can be considered the target).
    pub fn mcz() -> Self {
        Gate::new(StandardGate::MCZ, vec![]).unwrap()
    }
}

/// Matrix utilities for gates
pub struct GateMatrix;

impl GateMatrix {
    /// Compute the tensor (Kronecker) product of two matrices.
    /// Delegates to `NdArrayBackend::kronecker()`.
    pub fn tensor_product(a: &Array2<Complex64>, b: &Array2<Complex64>) -> Array2<Complex64> {
        let backend = NdArrayBackend::new();
        backend
            .kronecker(a, b)
            .expect("Kronecker product should not fail for valid matrices")
    }

    /// Compute the tensor product using the provided linear algebra backend
    pub fn tensor_product_with_backend<B: LinalgBackend<Scalar = Complex64>>(
        a: &B::Matrix,
        b: &B::Matrix,
        backend: &B,
    ) -> LinalgResult<B::Matrix> {
        backend.kronecker(a, b)
    }

    /// Check if a matrix is unitary (within tolerance)
    pub fn is_unitary(matrix: &Array2<Complex64>, tolerance: f64) -> bool {
        let n = matrix.nrows();
        if n != matrix.ncols() {
            return false;
        }

        // Compute matrix * matrix.conjugate_transpose()
        let mut product = Array2::zeros((n, n));
        for i in 0..n {
            for j in 0..n {
                let mut sum = Complex64::new(0.0, 0.0);
                for k in 0..n {
                    sum += matrix[[i, k]] * matrix[[j, k]].conj();
                }
                product[[i, j]] = sum;
            }
        }

        // Check if the product is identity
        for i in 0..n {
            for j in 0..n {
                let expected = if i == j {
                    Complex64::new(1.0, 0.0)
                } else {
                    Complex64::new(0.0, 0.0)
                };
                let diff = (product[[i, j]] - expected).norm();
                if diff > tolerance {
                    return false;
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_pauli_gates() {
        let symbols = HashMap::new();

        // Test Pauli-X
        let x = Gate::x();
        let x_matrix = x.matrix(&symbols).unwrap();
        assert_eq!(x_matrix[[0, 0]], ZERO);
        assert_eq!(x_matrix[[0, 1]], ONE);
        assert_eq!(x_matrix[[1, 0]], ONE);
        assert_eq!(x_matrix[[1, 1]], ZERO);

        // Test that X is unitary
        assert!(GateMatrix::is_unitary(&x_matrix, 1e-10));
    }

    #[test]
    fn test_hadamard_gate() {
        let symbols = HashMap::new();
        let h = Gate::h();
        let h_matrix = h.matrix(&symbols).unwrap();

        // Check that H is unitary
        assert!(GateMatrix::is_unitary(&h_matrix, 1e-10));

        // Check that H^2 = I
        let h_squared = h_matrix.dot(&h_matrix);
        let identity = Array2::from_shape_vec((2, 2), vec![ONE, ZERO, ZERO, ONE]).unwrap();

        for i in 0..2 {
            for j in 0..2 {
                assert!((h_squared[[i, j]] - identity[[i, j]]).norm() < 1e-10);
            }
        }
    }

    #[test]
    fn test_rotation_gates() {
        let symbols = HashMap::new();

        // Test RX(π) = -iX
        let rx_pi = Gate::rx(Parameter::new_float(PI));
        let rx_matrix = rx_pi.matrix(&symbols).unwrap();

        let x_matrix = Gate::x().matrix(&symbols).unwrap();
        let expected = x_matrix.mapv(|x| -I * x);

        for i in 0..2 {
            for j in 0..2 {
                assert!((rx_matrix[[i, j]] - expected[[i, j]]).norm() < 1e-10);
            }
        }
    }

    #[test]
    fn test_cnot_gate() {
        let symbols = HashMap::new();
        let cx = Gate::cx();
        let cx_matrix = cx.matrix(&symbols).unwrap();

        // Check dimensions
        assert_eq!(cx_matrix.dim(), (4, 4));

        // Check that CNOT is unitary
        assert!(GateMatrix::is_unitary(&cx_matrix, 1e-10));

        // Check that CNOT is self-inverse
        let cx_squared = cx_matrix.dot(&cx_matrix);
        let identity = Array2::eye(4).mapv(|x| Complex64::new(x, 0.0));

        for i in 0..4 {
            for j in 0..4 {
                assert!((cx_squared[[i, j]] - identity[[i, j]]).norm() < 1e-10);
            }
        }
    }

    #[test]
    fn test_parametric_gate() {
        let mut symbols = HashMap::new();
        symbols.insert("theta".to_string(), PI / 4.0);

        let rx_theta = Gate::rx(Parameter::new_symbol("theta"));
        assert!(rx_theta.is_parametric());
        assert_eq!(rx_theta.symbols(), vec!["theta".to_string()]);

        let matrix = rx_theta.matrix(&symbols).unwrap();
        assert!(GateMatrix::is_unitary(&matrix, 1e-10));
    }

    #[test]
    fn test_controlled_gates() {
        let symbols = HashMap::new();

        // Test CY gate
        let cy = Gate::cy();
        let cy_matrix = cy.matrix(&symbols).unwrap();
        assert_eq!(cy_matrix.dim(), (4, 4));
        assert!(GateMatrix::is_unitary(&cy_matrix, 1e-10));

        // Test CZ gate
        let cz = Gate::cz();
        let cz_matrix = cz.matrix(&symbols).unwrap();
        assert_eq!(cz_matrix.dim(), (4, 4));
        assert!(GateMatrix::is_unitary(&cz_matrix, 1e-10));

        // Test CH gate
        let ch = Gate::ch();
        let ch_matrix = ch.matrix(&symbols).unwrap();
        assert_eq!(ch_matrix.dim(), (4, 4));
        assert!(GateMatrix::is_unitary(&ch_matrix, 1e-10));
    }

    #[test]
    fn test_swap_gates() {
        let symbols = HashMap::new();

        // Test SWAP gate
        let swap = Gate::swap();
        let swap_matrix = swap.matrix(&symbols).unwrap();
        assert_eq!(swap_matrix.dim(), (4, 4));
        assert!(GateMatrix::is_unitary(&swap_matrix, 1e-10));

        // Test iSWAP gate
        let iswap = Gate::iswap();
        let iswap_matrix = iswap.matrix(&symbols).unwrap();
        assert_eq!(iswap_matrix.dim(), (4, 4));
        assert!(GateMatrix::is_unitary(&iswap_matrix, 1e-10));
    }

    #[test]
    fn test_three_qubit_gates() {
        let symbols = HashMap::new();

        // Test Toffoli (CCX) gate
        let ccx = Gate::ccx();
        let ccx_matrix = ccx.matrix(&symbols).unwrap();
        assert_eq!(ccx_matrix.dim(), (8, 8));
        assert!(GateMatrix::is_unitary(&ccx_matrix, 1e-10));

        // Test Fredkin (CSWAP) gate
        let cswap = Gate::cswap();
        let cswap_matrix = cswap.matrix(&symbols).unwrap();
        assert_eq!(cswap_matrix.dim(), (8, 8));
        assert!(GateMatrix::is_unitary(&cswap_matrix, 1e-10));
    }

    #[test]
    fn test_universal_gates() {
        let symbols = HashMap::new();

        // Test U1 gate
        let u1 = Gate::u1(Parameter::new_float(PI / 4.0));
        let u1_matrix = u1.matrix(&symbols).unwrap();
        assert_eq!(u1_matrix.dim(), (2, 2));
        assert!(GateMatrix::is_unitary(&u1_matrix, 1e-10));

        // Test U2 gate
        let u2 = Gate::u2(Parameter::new_float(0.0), Parameter::new_float(PI));
        let u2_matrix = u2.matrix(&symbols).unwrap();
        assert_eq!(u2_matrix.dim(), (2, 2));
        assert!(GateMatrix::is_unitary(&u2_matrix, 1e-10));

        // Test U3 gate
        let u3 = Gate::u3(
            Parameter::new_float(PI / 2.0),
            Parameter::new_float(0.0),
            Parameter::new_float(PI),
        );
        let u3_matrix = u3.matrix(&symbols).unwrap();
        assert_eq!(u3_matrix.dim(), (2, 2));
        assert!(GateMatrix::is_unitary(&u3_matrix, 1e-10));
    }

    #[test]
    fn test_controlled_rotation_gates() {
        let mut symbols = HashMap::new();
        symbols.insert("theta".to_string(), PI / 3.0);

        // Test CRX gate
        let crx = Gate::crx(Parameter::new_symbol("theta"));
        let crx_matrix = crx.matrix(&symbols).unwrap();
        assert_eq!(crx_matrix.dim(), (4, 4));
        assert!(GateMatrix::is_unitary(&crx_matrix, 1e-10));

        // Test CRY gate
        let cry = Gate::cry(Parameter::new_symbol("theta"));
        let cry_matrix = cry.matrix(&symbols).unwrap();
        assert_eq!(cry_matrix.dim(), (4, 4));
        assert!(GateMatrix::is_unitary(&cry_matrix, 1e-10));

        // Test CRZ gate
        let crz = Gate::crz(Parameter::new_symbol("theta"));
        let crz_matrix = crz.matrix(&symbols).unwrap();
        assert_eq!(crz_matrix.dim(), (4, 4));
        assert!(GateMatrix::is_unitary(&crz_matrix, 1e-10));
    }
}
