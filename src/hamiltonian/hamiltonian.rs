//! Hamiltonian Data Structure
//!
//! Author: gA4ss
//!
//! This module provides the core Hamiltonian representation as a sum of Pauli strings.
//!
//! # Mathematical Background
//!
//! A quantum Hamiltonian can be expressed as:
//! $$\hat{H} = \sum_i c_i P_i$$
//! where $P_i$ are Pauli strings and $c_i$ are complex coefficients.

use super::pauli_string::{PauliOperator, PauliString};
use crate::error::{MyQuatError, Result};
use crate::Parameter;
use num_complex::Complex64;
use std::collections::HashMap;
use std::fmt;

/// Pauli term: coefficient × Pauli string
///
/// Represents a single term in the Hamiltonian:
///
/// $$ c_i \cdot P_i, \quad P_i \in \{I, X, Y, Z\}^{\otimes n} $$
///
/// where $c_i \in \mathbb{C}$ and $P_i$ is an $n$-qubit Pauli string.
#[derive(Debug, Clone)]
pub struct PauliTerm {
    /// The Pauli string
    pub pauli_string: PauliString,

    /// Numerical coefficient
    pub coefficient: Complex64,

    /// Optional symbolic parameter name
    pub parameter: Option<String>,
}

impl PauliTerm {
    /// Create a new Pauli term
    pub fn new(pauli_string: PauliString, coefficient: Complex64) -> Self {
        Self {
            pauli_string,
            coefficient,
            parameter: None,
        }
    }

    /// Create a parametric Pauli term
    pub fn with_parameter(pauli_string: PauliString, parameter: String) -> Self {
        Self {
            pauli_string,
            coefficient: Complex64::new(1.0, 0.0),
            parameter: Some(parameter),
        }
    }

    /// Get effective coefficient (including Pauli string's coefficient)
    pub fn effective_coefficient(&self) -> Complex64 {
        self.coefficient * self.pauli_string.coefficient
    }

    /// Scale by a factor
    pub fn scale(&self, factor: Complex64) -> PauliTerm {
        PauliTerm {
            pauli_string: self.pauli_string.clone(),
            coefficient: self.coefficient * factor,
            parameter: self.parameter.clone(),
        }
    }

    /// Convert to LaTeX representation
    pub fn to_latex(&self) -> String {
        let coeff_str = if self.coefficient.im.abs() < 1e-10 {
            if (self.coefficient.re - 1.0).abs() < 1e-10 {
                String::new()
            } else if (self.coefficient.re + 1.0).abs() < 1e-10 {
                "-".to_string()
            } else {
                format!("{:.4}", self.coefficient.re)
            }
        } else {
            format!("({:.4} + {:.4}i)", self.coefficient.re, self.coefficient.im)
        };

        let pauli_latex = self.pauli_string.to_latex();

        if let Some(param) = &self.parameter {
            if coeff_str.is_empty() {
                format!("{} {}", param, pauli_latex)
            } else {
                format!("{} {} {}", coeff_str, param, pauli_latex)
            }
        } else if coeff_str.is_empty() {
            pauli_latex
        } else {
            format!("{} {}", coeff_str, pauli_latex)
        }
    }
}

impl fmt::Display for PauliTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(param) = &self.parameter {
            write!(
                f,
                "({} * {}) * {}",
                self.coefficient, param, self.pauli_string
            )
        } else {
            write!(f, "{} * {}", self.coefficient, self.pauli_string)
        }
    }
}

/// Quantum Hamiltonian
///
/// Represents $\hat{H} = \sum_i c_i P_i$ where $P_i$ are Pauli strings.
///
/// # Examples
///
/// ```
/// use myquat::hamiltonian::{Hamiltonian, PauliString};
/// use num_complex::Complex64;
///
/// let mut h = Hamiltonian::new(2);
///
/// // Add Ising interaction: -J * Z₀Z₁
/// let zz = PauliString::from_str("ZZ").unwrap();
/// h.add_term(zz, Complex64::new(-1.0, 0.0));
///
/// // Add transverse field: -h * X₀
/// let x0 = PauliString::from_str("XI").unwrap();
/// h.add_term(x0, Complex64::new(-0.5, 0.0));
/// ```
#[derive(Debug, Clone)]
pub struct Hamiltonian {
    /// Number of qubits
    pub num_qubits: usize,

    /// List of Pauli terms
    pub terms: Vec<PauliTerm>,

    /// Symbolic parameters and their values
    pub parameters: HashMap<String, Parameter>,

    /// Constant energy offset
    pub constant_term: Complex64,
}

impl Hamiltonian {
    /// Create a new Hamiltonian for n qubits
    pub fn new(num_qubits: usize) -> Self {
        Self {
            num_qubits,
            terms: Vec::new(),
            parameters: HashMap::new(),
            constant_term: Complex64::new(0.0, 0.0),
        }
    }

    /// Add a Pauli term to the Hamiltonian
    pub fn add_term(&mut self, pauli_string: PauliString, coefficient: Complex64) -> Result<()> {
        if pauli_string.num_qubits() != self.num_qubits {
            return Err(MyQuatError::hamiltonian_error(format!(
                "Pauli string has {} qubits but Hamiltonian has {} qubits",
                pauli_string.num_qubits(),
                self.num_qubits
            )));
        }

        self.terms.push(PauliTerm::new(pauli_string, coefficient));
        Ok(())
    }

    /// Add a parametric Pauli term
    pub fn add_parametric_term(
        &mut self,
        pauli_string: PauliString,
        parameter: String,
    ) -> Result<()> {
        if pauli_string.num_qubits() != self.num_qubits {
            return Err(MyQuatError::hamiltonian_error(format!(
                "Pauli string has {} qubits but Hamiltonian has {} qubits",
                pauli_string.num_qubits(),
                self.num_qubits
            )));
        }

        self.terms
            .push(PauliTerm::with_parameter(pauli_string, parameter));
        Ok(())
    }

    /// Set a parameter value
    pub fn set_parameter(&mut self, name: String, value: Parameter) {
        self.parameters.insert(name, value);
    }

    /// Add a constant energy offset
    pub fn add_constant(&mut self, constant: Complex64) {
        self.constant_term += constant;
    }

    /// Get number of terms
    pub fn num_terms(&self) -> usize {
        self.terms.len()
    }

    /// Check if Hamiltonian is Hermitian
    ///
    /// A Hamiltonian is Hermitian when all coefficients are real:
    ///
    /// $$ H^\dagger = H \;\Longleftrightarrow\; \forall i:\; c_i \in \mathbb{R} $$
    pub fn is_hermitian(&self) -> bool {
        // All terms must have real coefficients for Hermitian operator
        self.terms
            .iter()
            .all(|term| term.coefficient.im.abs() < 1e-10)
            && self.constant_term.im.abs() < 1e-10
    }

    /// Simplify by combining like terms
    pub fn simplify(&mut self) {
        // Group terms by Pauli string
        let mut grouped: HashMap<String, Complex64> = HashMap::new();

        for term in &self.terms {
            let key = term.pauli_string.to_string_repr().to_string();
            *grouped.entry(key).or_insert(Complex64::new(0.0, 0.0)) += term.effective_coefficient();
        }

        // Rebuild terms list
        self.terms.clear();
        for (pauli_str, coeff) in grouped {
            if coeff.norm() > 1e-10 {
                if let Ok(ps) = PauliString::from_str(&pauli_str) {
                    self.terms.push(PauliTerm::new(ps, coeff));
                }
            }
        }
    }

    /// Scale all terms by a factor
    pub fn scale(&mut self, factor: Complex64) {
        for term in &mut self.terms {
            term.coefficient *= factor;
        }
        self.constant_term *= factor;
    }

    /// Add another Hamiltonian
    pub fn add(&mut self, other: &Hamiltonian) -> Result<()> {
        if self.num_qubits != other.num_qubits {
            return Err(MyQuatError::hamiltonian_error(format!(
                "Cannot add Hamiltonians with different qubit counts: {} and {}",
                self.num_qubits, other.num_qubits
            )));
        }

        self.terms.extend(other.terms.clone());
        self.constant_term += other.constant_term;

        // Merge parameters
        for (name, value) in &other.parameters {
            self.parameters.insert(name.clone(), value.clone());
        }

        Ok(())
    }

    /// Compute commutator with another Hamiltonian $[H_1, H_2]$
    ///
    /// For Pauli strings $P_1, P_2$:
    ///
    /// $$ [P_1, P_2] =
    /// \begin{cases}
    /// 2i\,P_1 P_2 & \text{if they anticommute}, \\
    /// 0 & \text{if they commute}.
    /// \end{cases} $$
    pub fn commutator(&self, other: &Hamiltonian) -> Result<Hamiltonian> {
        // [H1, H2] = H1*H2 - H2*H1
        // For Pauli strings: [P1, P2] = 2i(P1*P2) if they anticommute, 0 if they commute

        let mut result = Hamiltonian::new(self.num_qubits);

        for term1 in &self.terms {
            for term2 in &other.terms {
                if !term1.pauli_string.commutes_with(&term2.pauli_string) {
                    // Anticommute: [P1, P2] = 2i(P1*P2)
                    let product = term1.pauli_string.multiply(&term2.pauli_string)?;
                    let coeff = term1.coefficient * term2.coefficient * Complex64::new(0.0, 2.0);
                    result.add_term(product, coeff)?;
                }
                // If commute, contribution is zero
            }
        }

        result.simplify();
        Ok(result)
    }

    /// Convert to LaTeX representation
    pub fn to_latex(&self) -> String {
        let mut latex = String::from("\\hat{H} = ");

        if self.terms.is_empty() {
            if self.constant_term.norm() > 1e-10 {
                latex.push_str(&format!("{:.4}", self.constant_term.re));
            } else {
                latex.push('0');
            }
            return latex;
        }

        let term_strs: Vec<String> = self.terms.iter().map(|term| term.to_latex()).collect();

        latex.push_str(&term_strs.join(" + "));

        if self.constant_term.norm() > 1e-10 {
            latex.push_str(&format!(" + {:.4}", self.constant_term.re));
        }

        latex
    }

    /// Convert to Markdown representation
    pub fn to_markdown(&self) -> String {
        let latex = self.to_latex();
        format!("$$\n{}\n$$", latex)
    }

    /// Export as JSON string
    pub fn to_json(&self) -> Result<String> {
        use serde_json::json;

        let terms_json: Vec<_> = self
            .terms
            .iter()
            .map(|term| {
                json!({
                    "pauli_string": term.pauli_string.to_string_repr(),
                    "coefficient": {
                        "real": term.coefficient.re,
                        "imag": term.coefficient.im
                    },
                    "parameter": term.parameter
                })
            })
            .collect();

        let hamiltonian_json = json!({
            "num_qubits": self.num_qubits,
            "terms": terms_json,
            "constant": {
                "real": self.constant_term.re,
                "imag": self.constant_term.im
            }
        });

        serde_json::to_string_pretty(&hamiltonian_json).map_err(|e| {
            MyQuatError::hamiltonian_error(format!("JSON serialization failed: {}", e))
        })
    }

    /// Get all qubits involved in the Hamiltonian
    pub fn support(&self) -> Vec<usize> {
        let mut qubits = std::collections::HashSet::new();
        for term in &self.terms {
            for qubit in term.pauli_string.support() {
                qubits.insert(qubit);
            }
        }
        let mut result: Vec<_> = qubits.into_iter().collect();
        result.sort_unstable();
        result
    }
}

impl fmt::Display for Hamiltonian {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Hamiltonian ({} qubits, {} terms):",
            self.num_qubits,
            self.terms.len()
        )?;
        for (i, term) in self.terms.iter().enumerate() {
            writeln!(f, "  Term {}: {}", i, term)?;
        }
        if self.constant_term.norm() > 1e-10 {
            writeln!(f, "  Constant: {:.4}", self.constant_term.re)?;
        }
        Ok(())
    }
}

/// Common Hamiltonian constructors
pub mod constructors {
    use super::*;

    /// Create Ising model Hamiltonian
    ///
    /// $$H = -J\sum_{\langle i,j\rangle} \sigma_z^{(i)} \sigma_z^{(j)} - h\sum_i \sigma_x^{(i)}$$
    pub fn ising_model(num_qubits: usize, j: f64, h: f64) -> Result<Hamiltonian> {
        let mut hamiltonian = Hamiltonian::new(num_qubits);

        // ZZ interactions
        for i in 0..num_qubits - 1 {
            let mut ops = vec![PauliOperator::I; num_qubits];
            ops[i] = PauliOperator::Z;
            ops[i + 1] = PauliOperator::Z;
            let ps = PauliString::new(ops, Complex64::new(1.0, 0.0));
            hamiltonian.add_term(ps, Complex64::new(-j, 0.0))?;
        }

        // X fields
        for i in 0..num_qubits {
            let ps = PauliString::single_qubit(num_qubits, i, PauliOperator::X)?;
            hamiltonian.add_term(ps, Complex64::new(-h, 0.0))?;
        }

        Ok(hamiltonian)
    }

    /// Create Heisenberg model Hamiltonian
    ///
    /// $$H = \sum_{\langle i,j\rangle} (J_x \sigma_x^{(i)} \sigma_x^{(j)} +
    ///                                   J_y \sigma_y^{(i)} \sigma_y^{(j)} +
    ///                                   J_z \sigma_z^{(i)} \sigma_z^{(j)})$$
    pub fn heisenberg_model(num_qubits: usize, jx: f64, jy: f64, jz: f64) -> Result<Hamiltonian> {
        let mut hamiltonian = Hamiltonian::new(num_qubits);

        for i in 0..num_qubits - 1 {
            // XX term
            let mut ops_xx = vec![PauliOperator::I; num_qubits];
            ops_xx[i] = PauliOperator::X;
            ops_xx[i + 1] = PauliOperator::X;
            let ps_xx = PauliString::new(ops_xx, Complex64::new(1.0, 0.0));
            hamiltonian.add_term(ps_xx, Complex64::new(jx, 0.0))?;

            // YY term
            let mut ops_yy = vec![PauliOperator::I; num_qubits];
            ops_yy[i] = PauliOperator::Y;
            ops_yy[i + 1] = PauliOperator::Y;
            let ps_yy = PauliString::new(ops_yy, Complex64::new(1.0, 0.0));
            hamiltonian.add_term(ps_yy, Complex64::new(jy, 0.0))?;

            // ZZ term
            let mut ops_zz = vec![PauliOperator::I; num_qubits];
            ops_zz[i] = PauliOperator::Z;
            ops_zz[i + 1] = PauliOperator::Z;
            let ps_zz = PauliString::new(ops_zz, Complex64::new(1.0, 0.0));
            hamiltonian.add_term(ps_zz, Complex64::new(jz, 0.0))?;
        }

        Ok(hamiltonian)
    }

    /// Create a single-qubit Hamiltonian
    ///
    /// $$H = \alpha \sigma_x + \beta \sigma_y + \gamma \sigma_z$$
    pub fn single_qubit(alpha: f64, beta: f64, gamma: f64) -> Result<Hamiltonian> {
        let mut hamiltonian = Hamiltonian::new(1);

        if alpha.abs() > 1e-10 {
            let ps = PauliString::from_str("X")?;
            hamiltonian.add_term(ps, Complex64::new(alpha, 0.0))?;
        }

        if beta.abs() > 1e-10 {
            let ps = PauliString::from_str("Y")?;
            hamiltonian.add_term(ps, Complex64::new(beta, 0.0))?;
        }

        if gamma.abs() > 1e-10 {
            let ps = PauliString::from_str("Z")?;
            hamiltonian.add_term(ps, Complex64::new(gamma, 0.0))?;
        }

        Ok(hamiltonian)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hamiltonian_creation() {
        let h = Hamiltonian::new(3);
        assert_eq!(h.num_qubits, 3);
        assert_eq!(h.num_terms(), 0);
    }

    #[test]
    fn test_add_term() {
        let mut h = Hamiltonian::new(2);
        let ps = PauliString::from_str("ZZ").unwrap();
        h.add_term(ps, Complex64::new(-1.0, 0.0)).unwrap();

        assert_eq!(h.num_terms(), 1);
    }

    #[test]
    fn test_hamiltonian_simplify() {
        let mut h = Hamiltonian::new(2);
        let ps1 = PauliString::from_str("ZZ").unwrap();
        let ps2 = PauliString::from_str("ZZ").unwrap();

        h.add_term(ps1, Complex64::new(1.0, 0.0)).unwrap();
        h.add_term(ps2, Complex64::new(2.0, 0.0)).unwrap();

        assert_eq!(h.num_terms(), 2);
        h.simplify();
        assert_eq!(h.num_terms(), 1);
        assert!((h.terms[0].coefficient.re - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_ising_model() {
        let h = constructors::ising_model(3, 1.0, 0.5).unwrap();
        assert_eq!(h.num_qubits, 3);
        // 2 ZZ terms + 3 X terms = 5 terms
        assert_eq!(h.num_terms(), 5);
    }

    #[test]
    fn test_heisenberg_model() {
        let h = constructors::heisenberg_model(2, 1.0, 1.0, 1.0).unwrap();
        assert_eq!(h.num_qubits, 2);
        // 1 pair: XX, YY, ZZ = 3 terms
        assert_eq!(h.num_terms(), 3);
    }

    #[test]
    fn test_hamiltonian_latex() {
        let mut h = Hamiltonian::new(2);
        let ps = PauliString::from_str("ZZ").unwrap();
        h.add_term(ps, Complex64::new(-1.0, 0.0)).unwrap();

        let latex = h.to_latex();
        assert!(latex.contains("\\hat{H}"));
        assert!(latex.contains("\\sigma_z"));
    }

    #[test]
    fn test_hamiltonian_is_hermitian() {
        let h = constructors::ising_model(2, 1.0, 0.5).unwrap();
        assert!(h.is_hermitian());
    }

    #[test]
    fn test_hamiltonian_add() {
        let h1 = constructors::single_qubit(1.0, 0.0, 0.0).unwrap();
        let h2 = constructors::single_qubit(0.0, 1.0, 0.0).unwrap();

        let mut h_total = h1.clone();
        h_total.add(&h2).unwrap();

        assert_eq!(h_total.num_terms(), 2);
    }

    #[test]
    fn test_hamiltonian_support() {
        let mut h = Hamiltonian::new(5);
        let ps = PauliString::from_str("XIZII").unwrap();
        h.add_term(ps, Complex64::new(1.0, 0.0)).unwrap();

        let support = h.support();
        assert_eq!(support, vec![0, 2]);
    }
}
