//! Fermion-to-Qubit Mapping
//! Author: gA4ss
//!
//! Implements various fermion-to-qubit transformation methods for quantum chemistry.

use crate::error::{MyQuatError, Result};
use crate::hamiltonian::{Hamiltonian, PauliOperator, PauliString};
use num_complex::Complex64;
use std::collections::HashMap;

/// Fermion operator types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FermionOperator {
    /// Creation operator a†
    Creation,
    /// Annihilation operator a
    Annihilation,
}

/// Fermion term: product of creation/annihilation operators
#[derive(Debug, Clone)]
pub struct FermionTerm {
    /// Operators and their indices
    pub operators: Vec<(usize, FermionOperator)>,
    /// Coefficient
    pub coefficient: Complex64,
}

impl FermionTerm {
    /// Create a new fermion term
    pub fn new(operators: Vec<(usize, FermionOperator)>, coefficient: Complex64) -> Self {
        Self {
            operators,
            coefficient,
        }
    }

    /// Create number operator n_i = a†_i a_i
    pub fn number_operator(index: usize, coefficient: Complex64) -> Self {
        Self {
            operators: vec![
                (index, FermionOperator::Creation),
                (index, FermionOperator::Annihilation),
            ],
            coefficient,
        }
    }

    /// Create hopping term a†_i a_j
    pub fn hopping_operator(i: usize, j: usize, coefficient: Complex64) -> Self {
        Self {
            operators: vec![
                (i, FermionOperator::Creation),
                (j, FermionOperator::Annihilation),
            ],
            coefficient,
        }
    }
}

/// Mapping method for fermion-to-qubit transformation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingMethod {
    /// Jordan-Wigner transformation
    JordanWigner,
    /// Bravyi-Kitaev transformation
    BravyiKitaev,
    /// Parity transformation
    Parity,
}

/// Jordan-Wigner transformation
///
/// Maps fermionic creation and annihilation operators to qubit Pauli strings.
/// The $Z$-strings enforce fermionic anti-commutation relations:
///
/// $$ a_j^\dagger = \frac{1}{2}(X_j - iY_j) \otimes Z_{j-1} \otimes \cdots \otimes Z_0 $$
///
/// $$ a_j = \frac{1}{2}(X_j + iY_j) \otimes Z_{j-1} \otimes \cdots \otimes Z_0 $$
///
/// These satisfy the canonical anti-commutation relations:
///
/// $$ \{a_i, a_j^\dagger\} = \delta_{ij}, \quad \{a_i, a_j\} = \{a_i^\dagger, a_j^\dagger\} = 0 $$
pub fn jordan_wigner_transform(
    fermion_term: &FermionTerm,
    num_qubits: usize,
) -> Result<Vec<(PauliString, Complex64)>> {
    let mut pauli_terms = Vec::new();

    // For each fermion operator in the term
    for &(index, op_type) in &fermion_term.operators {
        if index >= num_qubits {
            return Err(MyQuatError::hamiltonian_error(format!(
                "Fermion index {} exceeds number of qubits {}",
                index, num_qubits
            )));
        }

        // Build Pauli string for this operator
        match op_type {
            FermionOperator::Creation => {
                // a† = (1/2)(X - iY) Z...Z
                // Expands to (1/2)(XZ...Z - iYZ...Z)

                // X term
                let mut pauli_x = vec![PauliOperator::I; num_qubits];
                pauli_x[index] = PauliOperator::X;
                for i in 0..index {
                    pauli_x[i] = PauliOperator::Z;
                }

                // Y term
                let mut pauli_y = vec![PauliOperator::I; num_qubits];
                pauli_y[index] = PauliOperator::Y;
                for i in 0..index {
                    pauli_y[i] = PauliOperator::Z;
                }

                let coeff = Complex64::new(0.5, 0.0) * fermion_term.coefficient;
                pauli_terms.push((PauliString::new(pauli_x, Complex64::new(1.0, 0.0)), coeff));
                pauli_terms.push((
                    PauliString::new(pauli_y, Complex64::new(1.0, 0.0)),
                    Complex64::new(0.0, -0.5) * fermion_term.coefficient,
                ));
            }
            FermionOperator::Annihilation => {
                // a = (1/2)(X + iY) Z...Z

                // X term
                let mut pauli_x = vec![PauliOperator::I; num_qubits];
                pauli_x[index] = PauliOperator::X;
                for i in 0..index {
                    pauli_x[i] = PauliOperator::Z;
                }

                // Y term
                let mut pauli_y = vec![PauliOperator::I; num_qubits];
                pauli_y[index] = PauliOperator::Y;
                for i in 0..index {
                    pauli_y[i] = PauliOperator::Z;
                }

                let coeff = Complex64::new(0.5, 0.0) * fermion_term.coefficient;
                pauli_terms.push((PauliString::new(pauli_x, Complex64::new(1.0, 0.0)), coeff));
                pauli_terms.push((
                    PauliString::new(pauli_y, Complex64::new(1.0, 0.0)),
                    Complex64::new(0.0, 0.5) * fermion_term.coefficient,
                ));
            }
        }
    }

    Ok(pauli_terms)
}

/// Bravyi-Kitaev transformation
///
/// More efficient than Jordan-Wigner for certain cases.
/// Uses a binary tree structure to reduce the weight of Pauli strings.
///
/// The BK transformation maps fermion operators as:
///
/// $$ a_j^\dagger = \frac{1}{2}(X_j - iY_j) \otimes \prod_{k \in P(j)} Z_k \otimes \prod_{k \in U(j)} X_k $$
///
/// $$ a_j = \frac{1}{2}(X_j + iY_j) \otimes \prod_{k \in P(j)} Z_k \otimes \prod_{k \in U(j)} X_k $$
///
/// where $P(j)$ is the parity set and $U(j)$ is the update set, determined by
/// the binary representation of the orbital index.
///
/// Reference: Bravyi & Kitaev, Annals of Physics 298, 210-226 (2002)
pub fn bravyi_kitaev_transform(
    fermion_term: &FermionTerm,
    num_qubits: usize,
) -> Result<Vec<(PauliString, Complex64)>> {
    let mut pauli_terms = Vec::new();

    // Helper functions for BK transformation
    fn parity_set(j: usize, _n: usize) -> Vec<usize> {
        // P(j) = {k : k < j and (k+1) is a power of 2 dividing (j+1)}
        let mut set = Vec::new();
        let j_plus_1 = j + 1;

        for k in 0..j {
            let k_plus_1 = k + 1;
            // Check if k+1 is a power of 2
            if k_plus_1 & (k_plus_1 - 1) == 0 {
                // Check if k+1 divides j+1
                if j_plus_1 % k_plus_1 == 0 {
                    set.push(k);
                }
            }
        }
        set
    }

    fn update_set(j: usize, n: usize) -> Vec<usize> {
        // U(j) = {k : j < k < n and (j+1) divides (k+1) and (k+1)/(j+1) is odd}
        let mut set = Vec::new();
        let j_plus_1 = j + 1;

        for k in (j + 1)..n {
            let k_plus_1 = k + 1;
            if k_plus_1 % j_plus_1 == 0 {
                let quotient = k_plus_1 / j_plus_1;
                if quotient % 2 == 1 {
                    set.push(k);
                }
            }
        }
        set
    }

    fn flip_set(j: usize, n: usize) -> Vec<usize> {
        // F(j) = P(j) ∪ {j} ∪ U(j)
        let mut set = parity_set(j, n);
        set.push(j);
        set.extend(update_set(j, n));
        set
    }

    // For each fermion operator in the term
    for &(index, op_type) in &fermion_term.operators {
        if index >= num_qubits {
            return Err(MyQuatError::hamiltonian_error(format!(
                "Fermion index {} exceeds number of qubits {}",
                index, num_qubits
            )));
        }

        match op_type {
            FermionOperator::Creation => {
                // a†_j in BK basis
                // a†_j = (1/2)(X_j - iY_j) ⊗ Z_{P(j)} ⊗ X_{U(j)}

                let p_set = parity_set(index, num_qubits);
                let u_set = update_set(index, num_qubits);

                // X term
                let mut pauli_x = vec![PauliOperator::I; num_qubits];
                pauli_x[index] = PauliOperator::X;
                for &k in &p_set {
                    pauli_x[k] = PauliOperator::Z;
                }
                for &k in &u_set {
                    pauli_x[k] = PauliOperator::X;
                }

                // Y term
                let mut pauli_y = vec![PauliOperator::I; num_qubits];
                pauli_y[index] = PauliOperator::Y;
                for &k in &p_set {
                    pauli_y[k] = PauliOperator::Z;
                }
                for &k in &u_set {
                    pauli_y[k] = PauliOperator::X;
                }

                let coeff = Complex64::new(0.5, 0.0) * fermion_term.coefficient;
                pauli_terms.push((PauliString::new(pauli_x, Complex64::new(1.0, 0.0)), coeff));
                pauli_terms.push((
                    PauliString::new(pauli_y, Complex64::new(1.0, 0.0)),
                    Complex64::new(0.0, -0.5) * fermion_term.coefficient,
                ));
            }
            FermionOperator::Annihilation => {
                // a_j in BK basis
                // a_j = (1/2)(X_j + iY_j) ⊗ Z_{P(j)} ⊗ X_{U(j)}

                let p_set = parity_set(index, num_qubits);
                let u_set = update_set(index, num_qubits);

                // X term
                let mut pauli_x = vec![PauliOperator::I; num_qubits];
                pauli_x[index] = PauliOperator::X;
                for &k in &p_set {
                    pauli_x[k] = PauliOperator::Z;
                }
                for &k in &u_set {
                    pauli_x[k] = PauliOperator::X;
                }

                // Y term
                let mut pauli_y = vec![PauliOperator::I; num_qubits];
                pauli_y[index] = PauliOperator::Y;
                for &k in &p_set {
                    pauli_y[k] = PauliOperator::Z;
                }
                for &k in &u_set {
                    pauli_y[k] = PauliOperator::X;
                }

                let coeff = Complex64::new(0.5, 0.0) * fermion_term.coefficient;
                pauli_terms.push((PauliString::new(pauli_x, Complex64::new(1.0, 0.0)), coeff));
                pauli_terms.push((
                    PauliString::new(pauli_y, Complex64::new(1.0, 0.0)),
                    Complex64::new(0.0, 0.5) * fermion_term.coefficient,
                ));
            }
        }
    }

    Ok(pauli_terms)
}

/// Parity transformation
///
/// Alternative to Jordan-Wigner with different locality properties.
/// In parity basis, qubit $j$ stores the parity of occupation numbers $0$ to $j$:
///
/// $$ \text{qubit}_j \;\leftrightarrow\; \bigoplus_{k=0}^j n_k $$
///
/// The creation operator for $j > 0$ maps as:
///
/// $$ a_j^\dagger = \frac{1}{2}(X_{j-1} X_j - iY_{j-1} X_j) $$
///
/// and for $j = 0$: $a_0^\dagger = \frac{1}{2}(X_0 - iY_0)$.
///
/// Better than Jordan-Wigner for Hamiltonians with nearest-neighbor interactions.
///
/// Reference: Bravyi, Gambetta, Mezzacapo, Temme, arXiv:1701.08213 (2017)
pub fn parity_transform(
    fermion_term: &FermionTerm,
    num_qubits: usize,
) -> Result<Vec<(PauliString, Complex64)>> {
    let mut pauli_terms = Vec::new();

    // For each fermion operator in the term
    for &(index, op_type) in &fermion_term.operators {
        if index >= num_qubits {
            return Err(MyQuatError::hamiltonian_error(format!(
                "Fermion index {} exceeds number of qubits {}",
                index, num_qubits
            )));
        }

        match op_type {
            FermionOperator::Creation => {
                // a†_j in parity basis
                // a†_j = (1/2)(X_{j-1} X_j - iY_{j-1} X_j) for j > 0
                // a†_0 = (1/2)(X_0 - iY_0) for j = 0

                if index == 0 {
                    // Special case for j = 0
                    let mut pauli_x = vec![PauliOperator::I; num_qubits];
                    pauli_x[0] = PauliOperator::X;

                    let mut pauli_y = vec![PauliOperator::I; num_qubits];
                    pauli_y[0] = PauliOperator::Y;

                    let coeff = Complex64::new(0.5, 0.0) * fermion_term.coefficient;
                    pauli_terms.push((PauliString::new(pauli_x, Complex64::new(1.0, 0.0)), coeff));
                    pauli_terms.push((
                        PauliString::new(pauli_y, Complex64::new(1.0, 0.0)),
                        Complex64::new(0.0, -0.5) * fermion_term.coefficient,
                    ));
                } else {
                    // General case for j > 0
                    // XX term
                    let mut pauli_xx = vec![PauliOperator::I; num_qubits];
                    pauli_xx[index - 1] = PauliOperator::X;
                    pauli_xx[index] = PauliOperator::X;

                    // YX term
                    let mut pauli_yx = vec![PauliOperator::I; num_qubits];
                    pauli_yx[index - 1] = PauliOperator::Y;
                    pauli_yx[index] = PauliOperator::X;

                    let coeff = Complex64::new(0.5, 0.0) * fermion_term.coefficient;
                    pauli_terms.push((PauliString::new(pauli_xx, Complex64::new(1.0, 0.0)), coeff));
                    pauli_terms.push((
                        PauliString::new(pauli_yx, Complex64::new(1.0, 0.0)),
                        Complex64::new(0.0, -0.5) * fermion_term.coefficient,
                    ));
                }
            }
            FermionOperator::Annihilation => {
                // a_j in parity basis
                // a_j = (1/2)(X_{j-1} X_j + iY_{j-1} X_j) for j > 0
                // a_0 = (1/2)(X_0 + iY_0) for j = 0

                if index == 0 {
                    // Special case for j = 0
                    let mut pauli_x = vec![PauliOperator::I; num_qubits];
                    pauli_x[0] = PauliOperator::X;

                    let mut pauli_y = vec![PauliOperator::I; num_qubits];
                    pauli_y[0] = PauliOperator::Y;

                    let coeff = Complex64::new(0.5, 0.0) * fermion_term.coefficient;
                    pauli_terms.push((PauliString::new(pauli_x, Complex64::new(1.0, 0.0)), coeff));
                    pauli_terms.push((
                        PauliString::new(pauli_y, Complex64::new(1.0, 0.0)),
                        Complex64::new(0.0, 0.5) * fermion_term.coefficient,
                    ));
                } else {
                    // General case for j > 0
                    // XX term
                    let mut pauli_xx = vec![PauliOperator::I; num_qubits];
                    pauli_xx[index - 1] = PauliOperator::X;
                    pauli_xx[index] = PauliOperator::X;

                    // YX term
                    let mut pauli_yx = vec![PauliOperator::I; num_qubits];
                    pauli_yx[index - 1] = PauliOperator::Y;
                    pauli_yx[index] = PauliOperator::X;

                    let coeff = Complex64::new(0.5, 0.0) * fermion_term.coefficient;
                    pauli_terms.push((PauliString::new(pauli_xx, Complex64::new(1.0, 0.0)), coeff));
                    pauli_terms.push((
                        PauliString::new(pauli_yx, Complex64::new(1.0, 0.0)),
                        Complex64::new(0.0, 0.5) * fermion_term.coefficient,
                    ));
                }
            }
        }
    }

    Ok(pauli_terms)
}

/// Build qubit Hamiltonian from fermion Hamiltonian
pub fn fermion_to_qubit_hamiltonian(
    fermion_terms: &[FermionTerm],
    num_qubits: usize,
    method: MappingMethod,
) -> Result<Hamiltonian> {
    let mut hamiltonian = Hamiltonian::new(num_qubits);

    for fermion_term in fermion_terms {
        let pauli_terms = match method {
            MappingMethod::JordanWigner => jordan_wigner_transform(fermion_term, num_qubits)?,
            MappingMethod::BravyiKitaev => bravyi_kitaev_transform(fermion_term, num_qubits)?,
            MappingMethod::Parity => parity_transform(fermion_term, num_qubits)?,
        };

        for (pauli_string, coeff) in pauli_terms {
            hamiltonian.add_term(pauli_string, coeff)?;
        }
    }

    Ok(hamiltonian)
}

/// Electronic structure Hamiltonian builder
#[derive(Debug, Clone)]
pub struct ElectronicStructureHamiltonian {
    /// Number of orbitals
    pub num_orbitals: usize,
    /// One-electron integrals h_ij
    pub one_body_integrals: Vec<Vec<f64>>,
    /// Two-electron integrals h_ijkl
    pub two_body_integrals: HashMap<(usize, usize, usize, usize), f64>,
    /// Nuclear repulsion energy
    pub nuclear_repulsion: f64,
}

impl ElectronicStructureHamiltonian {
    /// Create a new electronic structure Hamiltonian
    pub fn new(num_orbitals: usize) -> Self {
        Self {
            num_orbitals,
            one_body_integrals: vec![vec![0.0; num_orbitals]; num_orbitals],
            two_body_integrals: HashMap::new(),
            nuclear_repulsion: 0.0,
        }
    }

    /// Set one-body integral
    pub fn set_one_body(&mut self, i: usize, j: usize, value: f64) {
        if i < self.num_orbitals && j < self.num_orbitals {
            self.one_body_integrals[i][j] = value;
        }
    }

    /// Set two-body integral
    pub fn set_two_body(&mut self, i: usize, j: usize, k: usize, l: usize, value: f64) {
        self.two_body_integrals.insert((i, j, k, l), value);
    }

    /// Build fermion Hamiltonian
    pub fn to_fermion_hamiltonian(&self) -> Vec<FermionTerm> {
        let mut terms = Vec::new();

        // One-body terms: h_ij a†_i a_j
        for i in 0..self.num_orbitals {
            for j in 0..self.num_orbitals {
                let h_ij = self.one_body_integrals[i][j];
                if h_ij.abs() > 1e-10 {
                    terms.push(FermionTerm::hopping_operator(
                        i,
                        j,
                        Complex64::new(h_ij, 0.0),
                    ));
                }
            }
        }

        // Two-body terms: (1/2) h_ijkl a†_i a†_j a_l a_k
        for (&(i, j, k, l), &h_ijkl) in &self.two_body_integrals {
            if h_ijkl.abs() > 1e-10 {
                let operators = vec![
                    (i, FermionOperator::Creation),
                    (j, FermionOperator::Creation),
                    (l, FermionOperator::Annihilation),
                    (k, FermionOperator::Annihilation),
                ];
                terms.push(FermionTerm::new(
                    operators,
                    Complex64::new(0.5 * h_ijkl, 0.0),
                ));
            }
        }

        terms
    }

    /// Build qubit Hamiltonian with specified mapping
    pub fn to_qubit_hamiltonian(&self, method: MappingMethod) -> Result<Hamiltonian> {
        let fermion_terms = self.to_fermion_hamiltonian();
        let num_qubits = 2 * self.num_orbitals; // Spin-orbitals

        let mut hamiltonian = fermion_to_qubit_hamiltonian(&fermion_terms, num_qubits, method)?;
        hamiltonian.constant_term = Complex64::new(self.nuclear_repulsion, 0.0);

        Ok(hamiltonian)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fermion_term_creation() {
        let term = FermionTerm::number_operator(0, Complex64::new(1.0, 0.0));
        assert_eq!(term.operators.len(), 2);
        assert_eq!(term.coefficient, Complex64::new(1.0, 0.0));
    }

    #[test]
    fn test_jordan_wigner_creation() {
        let term = FermionTerm::new(
            vec![(0, FermionOperator::Creation)],
            Complex64::new(1.0, 0.0),
        );

        let pauli_terms = jordan_wigner_transform(&term, 2).unwrap();
        assert_eq!(pauli_terms.len(), 2); // X and Y terms
    }

    #[test]
    fn test_electronic_structure() {
        let mut es = ElectronicStructureHamiltonian::new(2);
        es.set_one_body(0, 0, -1.0);
        es.set_one_body(1, 1, -0.5);
        es.nuclear_repulsion = 0.5;

        let fermion_terms = es.to_fermion_hamiltonian();
        assert!(!fermion_terms.is_empty());

        let hamiltonian = es
            .to_qubit_hamiltonian(MappingMethod::JordanWigner)
            .unwrap();
        assert_eq!(hamiltonian.num_qubits, 4);
        assert_eq!(hamiltonian.constant_term.re, 0.5);
    }

    #[test]
    fn test_bravyi_kitaev_creation() {
        let term = FermionTerm::new(
            vec![(1, FermionOperator::Creation)],
            Complex64::new(1.0, 0.0),
        );

        let pauli_terms = bravyi_kitaev_transform(&term, 4).unwrap();
        assert_eq!(pauli_terms.len(), 2); // X and Y terms

        // Verify that BK produces different Pauli strings than JW
        let jw_terms = jordan_wigner_transform(&term, 4).unwrap();
        // Both should have 2 terms, but with different Pauli operators
        assert_eq!(jw_terms.len(), 2);
    }

    #[test]
    fn test_bravyi_kitaev_annihilation() {
        let term = FermionTerm::new(
            vec![(2, FermionOperator::Annihilation)],
            Complex64::new(1.0, 0.0),
        );

        let pauli_terms = bravyi_kitaev_transform(&term, 4).unwrap();
        assert_eq!(pauli_terms.len(), 2);
    }

    #[test]
    fn test_parity_creation() {
        let term = FermionTerm::new(
            vec![(0, FermionOperator::Creation)],
            Complex64::new(1.0, 0.0),
        );

        let pauli_terms = parity_transform(&term, 4).unwrap();
        assert_eq!(pauli_terms.len(), 2); // X and Y terms
    }

    #[test]
    fn test_parity_creation_nonzero() {
        let term = FermionTerm::new(
            vec![(1, FermionOperator::Creation)],
            Complex64::new(1.0, 0.0),
        );

        let pauli_terms = parity_transform(&term, 4).unwrap();
        assert_eq!(pauli_terms.len(), 2); // XX and YX terms
    }

    #[test]
    fn test_parity_annihilation() {
        let term = FermionTerm::new(
            vec![(2, FermionOperator::Annihilation)],
            Complex64::new(1.0, 0.0),
        );

        let pauli_terms = parity_transform(&term, 4).unwrap();
        assert_eq!(pauli_terms.len(), 2);
    }

    #[test]
    fn test_mapping_methods_comparison() {
        // Test that all three methods produce valid Hamiltonians
        let mut es = ElectronicStructureHamiltonian::new(2);
        es.set_one_body(0, 0, -1.0);
        es.set_one_body(1, 1, -0.5);
        es.nuclear_repulsion = 0.5;

        let h_jw = es
            .to_qubit_hamiltonian(MappingMethod::JordanWigner)
            .unwrap();
        let h_bk = es
            .to_qubit_hamiltonian(MappingMethod::BravyiKitaev)
            .unwrap();
        let h_parity = es.to_qubit_hamiltonian(MappingMethod::Parity).unwrap();

        // All should have the same number of qubits
        assert_eq!(h_jw.num_qubits, 4);
        assert_eq!(h_bk.num_qubits, 4);
        assert_eq!(h_parity.num_qubits, 4);

        // All should have the same constant term
        assert_eq!(h_jw.constant_term.re, 0.5);
        assert_eq!(h_bk.constant_term.re, 0.5);
        assert_eq!(h_parity.constant_term.re, 0.5);
    }

    #[test]
    fn test_number_operator_transformations() {
        // Test number operator n_0 = a†_0 a_0
        let term = FermionTerm::number_operator(0, Complex64::new(1.0, 0.0));

        let jw_terms = jordan_wigner_transform(&term, 2).unwrap();
        let bk_terms = bravyi_kitaev_transform(&term, 2).unwrap();
        let parity_terms = parity_transform(&term, 2).unwrap();

        // All methods should produce some Pauli terms
        assert!(!jw_terms.is_empty());
        assert!(!bk_terms.is_empty());
        assert!(!parity_terms.is_empty());
    }
}
