//! Pauli String Representation
//!
//! Author: gA4ss
//!
//! This module provides data structures and operations for Pauli strings,
//! which are the fundamental building blocks of quantum Hamiltonians.
//!
//! # Mathematical Background
//!
//! A Pauli string is a tensor product of Pauli operators:
//! $$P = \sigma_{i_1} \otimes \sigma_{i_2} \otimes \cdots \otimes \sigma_{i_n}$$
//! where each $\sigma_i \in \{I, X, Y, Z\}$.
//!
//! Pauli strings form a basis for Hermitian operators on n qubits.

use crate::error::{MyQuatError, Result};
use num_complex::Complex64;
use std::cell::OnceCell;
use std::fmt;
use std::ops::{Mul, Neg};

/// Pauli operator types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PauliOperator {
    /// Identity operator I = [[1, 0], [0, 1]]
    I,
    /// Pauli-X operator σ_x = [[0, 1], [1, 0]]
    X,
    /// Pauli-Y operator σ_y = [[0, -i], [i, 0]]
    Y,
    /// Pauli-Z operator σ_z = [[1, 0], [0, -1]]
    Z,
}

impl PauliOperator {
    /// Parse a character into a Pauli operator
    pub fn from_char(c: char) -> Result<Self> {
        match c.to_ascii_uppercase() {
            'I' => Ok(PauliOperator::I),
            'X' => Ok(PauliOperator::X),
            'Y' => Ok(PauliOperator::Y),
            'Z' => Ok(PauliOperator::Z),
            _ => Err(MyQuatError::hamiltonian_error(format!(
                "Invalid Pauli operator: {}",
                c
            ))),
        }
    }

    /// Convert to character representation
    pub fn to_char(&self) -> char {
        match self {
            PauliOperator::I => 'I',
            PauliOperator::X => 'X',
            PauliOperator::Y => 'Y',
            PauliOperator::Z => 'Z',
        }
    }

    /// Check if this operator commutes with another
    pub fn commutes_with(&self, other: &PauliOperator) -> bool {
        match (self, other) {
            (PauliOperator::I, _) | (_, PauliOperator::I) => true,
            (a, b) if a == b => true,
            _ => false,
        }
    }

    /// Multiply two Pauli operators
    /// Returns (result, phase) where phase is in {1, -1, i, -i}
    pub fn multiply(&self, other: &PauliOperator) -> (PauliOperator, Complex64) {
        match (self, other) {
            // Identity cases
            (PauliOperator::I, p) | (p, PauliOperator::I) => (*p, Complex64::new(1.0, 0.0)),

            // Same operators: σ² = I
            (PauliOperator::X, PauliOperator::X)
            | (PauliOperator::Y, PauliOperator::Y)
            | (PauliOperator::Z, PauliOperator::Z) => (PauliOperator::I, Complex64::new(1.0, 0.0)),

            // XY = iZ, YX = -iZ
            (PauliOperator::X, PauliOperator::Y) => (PauliOperator::Z, Complex64::new(0.0, 1.0)),
            (PauliOperator::Y, PauliOperator::X) => (PauliOperator::Z, Complex64::new(0.0, -1.0)),

            // YZ = iX, ZY = -iX
            (PauliOperator::Y, PauliOperator::Z) => (PauliOperator::X, Complex64::new(0.0, 1.0)),
            (PauliOperator::Z, PauliOperator::Y) => (PauliOperator::X, Complex64::new(0.0, -1.0)),

            // ZX = iY, XZ = -iY
            (PauliOperator::Z, PauliOperator::X) => (PauliOperator::Y, Complex64::new(0.0, 1.0)),
            (PauliOperator::X, PauliOperator::Z) => (PauliOperator::Y, Complex64::new(0.0, -1.0)),
        }
    }
}

impl fmt::Display for PauliOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

/// Pauli string: tensor product of Pauli operators
///
/// Represents: c * (σ_{i_1} ⊗ σ_{i_2} ⊗ ... ⊗ σ_{i_n})
///
/// # Examples
///
/// ```
/// use myquat::hamiltonian::PauliString;
///
/// // Create "XYZ" Pauli string
/// let ps = PauliString::from_str("XYZ").unwrap();
/// assert_eq!(ps.num_qubits(), 3);
/// ```
#[derive(Debug, Clone)]
pub struct PauliString {
    /// Pauli operators for each qubit
    pub operators: Vec<PauliOperator>,

    /// Global coefficient (includes phase from multiplications)
    pub coefficient: Complex64,

    /// Cached string representation
    string_repr: OnceCell<String>,
}

impl PauliString {
    /// Create a new Pauli string
    pub fn new(operators: Vec<PauliOperator>, coefficient: Complex64) -> Self {
        Self {
            operators,
            coefficient,
            string_repr: OnceCell::new(),
        }
    }

    /// Create identity Pauli string of given length
    pub fn identity(num_qubits: usize) -> Self {
        Self::new(vec![PauliOperator::I; num_qubits], Complex64::new(1.0, 0.0))
    }

    /// Parse from string format: "XYZII"
    pub fn from_str(s: &str) -> Result<Self> {
        let operators: Result<Vec<_>> = s.chars().map(PauliOperator::from_char).collect();

        Ok(Self::new(operators?, Complex64::new(1.0, 0.0)))
    }

    /// Create from operator at specific qubit position
    pub fn single_qubit(num_qubits: usize, qubit: usize, op: PauliOperator) -> Result<Self> {
        if qubit >= num_qubits {
            return Err(MyQuatError::hamiltonian_error(format!(
                "Qubit index {} out of range for {} qubits",
                qubit, num_qubits
            )));
        }

        let mut operators = vec![PauliOperator::I; num_qubits];
        operators[qubit] = op;
        Ok(Self::new(operators, Complex64::new(1.0, 0.0)))
    }

    /// Create a single-qubit Z on the given qubit index (no Result).
    pub fn single_qubit_z(num_qubits: usize, qubit: usize) -> Self {
        let mut operators = vec![PauliOperator::I; num_qubits];
        operators[qubit] = PauliOperator::Z;
        Self::new(operators, Complex64::new(1.0, 0.0))
    }

    /// Create a single-qubit X on the given qubit index (no Result).
    pub fn single_qubit_x(num_qubits: usize, qubit: usize) -> Self {
        let mut operators = vec![PauliOperator::I; num_qubits];
        operators[qubit] = PauliOperator::X;
        Self::new(operators, Complex64::new(1.0, 0.0))
    }

    /// Get number of qubits
    pub fn num_qubits(&self) -> usize {
        self.operators.len()
    }

    /// Get string representation: "XYZII"
    pub fn to_string_repr(&self) -> &str {
        self.string_repr
            .get_or_init(|| self.operators.iter().map(|op| op.to_char()).collect())
    }

    /// Check if this is an identity string (all I operators)
    pub fn is_identity(&self) -> bool {
        self.operators.iter().all(|op| *op == PauliOperator::I)
    }

    /// Count number of non-identity operators
    pub fn weight(&self) -> usize {
        self.operators
            .iter()
            .filter(|op| **op != PauliOperator::I)
            .count()
    }

    /// Check if this Pauli string commutes with another
    pub fn commutes_with(&self, other: &PauliString) -> bool {
        if self.num_qubits() != other.num_qubits() {
            return false;
        }

        // Count positions where operators don't commute
        let anti_commute_count = self
            .operators
            .iter()
            .zip(other.operators.iter())
            .filter(|(a, b)| !a.commutes_with(b))
            .count();

        // Pauli strings commute if they anti-commute at even number of positions
        anti_commute_count % 2 == 0
    }

    /// Multiply with another Pauli string
    ///
    /// Returns the product Pauli string with updated coefficient
    pub fn multiply(&self, other: &PauliString) -> Result<PauliString> {
        if self.num_qubits() != other.num_qubits() {
            return Err(MyQuatError::hamiltonian_error(format!(
                "Cannot multiply Pauli strings of different lengths: {} and {}",
                self.num_qubits(),
                other.num_qubits()
            )));
        }

        let mut result_ops = Vec::with_capacity(self.num_qubits());
        let mut total_phase = self.coefficient * other.coefficient;

        for (op1, op2) in self.operators.iter().zip(other.operators.iter()) {
            let (result_op, phase) = op1.multiply(op2);
            result_ops.push(result_op);
            total_phase *= phase;
        }

        Ok(PauliString::new(result_ops, total_phase))
    }

    /// Scale by a complex coefficient
    pub fn scale(&self, factor: Complex64) -> PauliString {
        PauliString::new(self.operators.clone(), self.coefficient * factor)
    }

    /// Get specific qubits where operator is not identity
    pub fn support(&self) -> Vec<usize> {
        self.operators
            .iter()
            .enumerate()
            .filter_map(|(i, op)| {
                if *op != PauliOperator::I {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Extract operator at specific qubit
    pub fn operator_at(&self, qubit: usize) -> Option<PauliOperator> {
        self.operators.get(qubit).copied()
    }

    /// Create LaTeX representation
    ///
    /// Examples:
    /// - "XII" → "σ_x^{(0)}"
    /// - "XYZ" → "σ_x^{(0)} σ_y^{(1)} σ_z^{(2)}"
    pub fn to_latex(&self) -> String {
        let ops: Vec<String> = self
            .operators
            .iter()
            .enumerate()
            .filter_map(|(i, op)| match op {
                PauliOperator::I => None,
                PauliOperator::X => Some(format!("\\sigma_x^{{({})}}", i)),
                PauliOperator::Y => Some(format!("\\sigma_y^{{({})}}", i)),
                PauliOperator::Z => Some(format!("\\sigma_z^{{({})}}", i)),
            })
            .collect();

        if ops.is_empty() {
            "I".to_string()
        } else {
            ops.join(" ")
        }
    }

    /// Simplify representation (remove trailing identities)
    pub fn simplified(&self) -> PauliString {
        let mut ops = self.operators.clone();

        // Remove trailing identities
        while let Some(PauliOperator::I) = ops.last() {
            if ops.len() == 1 {
                break; // Keep at least one operator
            }
            ops.pop();
        }

        PauliString::new(ops, self.coefficient)
    }
}

impl PartialEq for PauliString {
    fn eq(&self, other: &Self) -> bool {
        self.operators == other.operators
        // Note: coefficients are not compared for structural equality
    }
}

impl Eq for PauliString {}

impl fmt::Display for PauliString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format coefficient
        let coeff_str = if self.coefficient.im.abs() < 1e-10 {
            format!("{:.4}", self.coefficient.re)
        } else if self.coefficient.re.abs() < 1e-10 {
            format!("{:.4}i", self.coefficient.im)
        } else {
            format!("({:.4} + {:.4}i)", self.coefficient.re, self.coefficient.im)
        };

        write!(f, "{} * {}", coeff_str, self.to_string_repr())
    }
}

impl Mul for &PauliString {
    type Output = Result<PauliString>;

    fn mul(self, rhs: &PauliString) -> Self::Output {
        self.multiply(rhs)
    }
}

impl Neg for PauliString {
    type Output = PauliString;

    fn neg(self) -> Self::Output {
        PauliString::new(self.operators, -self.coefficient)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pauli_operator_multiply() {
        // σ_x * σ_y = iσ_z
        let (result, phase) = PauliOperator::X.multiply(&PauliOperator::Y);
        assert_eq!(result, PauliOperator::Z);
        assert_eq!(phase, Complex64::new(0.0, 1.0));

        // σ_x * σ_x = I
        let (result, phase) = PauliOperator::X.multiply(&PauliOperator::X);
        assert_eq!(result, PauliOperator::I);
        assert_eq!(phase, Complex64::new(1.0, 0.0));
    }

    #[test]
    fn test_pauli_string_from_str() {
        let ps = PauliString::from_str("XYZ").unwrap();
        assert_eq!(ps.num_qubits(), 3);
        assert_eq!(ps.operators[0], PauliOperator::X);
        assert_eq!(ps.operators[1], PauliOperator::Y);
        assert_eq!(ps.operators[2], PauliOperator::Z);
    }

    #[test]
    fn test_pauli_string_identity() {
        let ps = PauliString::identity(4);
        assert_eq!(ps.num_qubits(), 4);
        assert!(ps.is_identity());
    }

    #[test]
    fn test_pauli_string_weight() {
        let ps = PauliString::from_str("XIZI").unwrap();
        assert_eq!(ps.weight(), 2);
    }

    #[test]
    fn test_pauli_string_commutation() {
        let ps1 = PauliString::from_str("XX").unwrap();
        let ps2 = PauliString::from_str("ZZ").unwrap();
        assert!(ps1.commutes_with(&ps2));

        let ps3 = PauliString::from_str("XZ").unwrap();
        let ps4 = PauliString::from_str("ZX").unwrap();
        assert!(ps3.commutes_with(&ps4));

        let ps5 = PauliString::from_str("XI").unwrap();
        let ps6 = PauliString::from_str("ZI").unwrap();
        assert!(!ps5.commutes_with(&ps6));
    }

    #[test]
    fn test_pauli_string_multiply() {
        let ps1 = PauliString::from_str("X").unwrap();
        let ps2 = PauliString::from_str("Y").unwrap();
        let result = ps1.multiply(&ps2).unwrap();

        assert_eq!(result.operators[0], PauliOperator::Z);
        assert_eq!(result.coefficient, Complex64::new(0.0, 1.0));
    }

    #[test]
    fn test_pauli_string_support() {
        let ps = PauliString::from_str("XIZI").unwrap();
        let support = ps.support();
        assert_eq!(support, vec![0, 2]);
    }

    #[test]
    fn test_pauli_string_latex() {
        let ps = PauliString::from_str("XYZ").unwrap();
        let latex = ps.to_latex();
        assert!(latex.contains("\\sigma_x^{(0)}"));
        assert!(latex.contains("\\sigma_y^{(1)}"));
        assert!(latex.contains("\\sigma_z^{(2)}"));
    }

    #[test]
    fn test_single_qubit_pauli() {
        let ps = PauliString::single_qubit(5, 2, PauliOperator::X).unwrap();
        assert_eq!(ps.num_qubits(), 5);
        assert_eq!(ps.operators[2], PauliOperator::X);
        assert_eq!(ps.weight(), 1);
    }
}
