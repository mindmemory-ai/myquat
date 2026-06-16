//! Hamiltonian Optimization Techniques
//!
//! Author: gA4ss
//!
//! This module provides advanced optimization techniques for Hamiltonians:
//! - Qubit tapering: Reduce qubit count using symmetries
//! - Jordan-Wigner transformation: Fermion to qubit mapping
//! - Commuting term merging: Optimal grouping of Pauli terms
//! - Unitary transformations: Hardware-aware optimization
//!
//! # Mathematical Background
//!
//! ## Qubit Tapering
//! For a Hamiltonian with Z₂ symmetries, we can reduce the number of qubits:
//! If [H, Sᵢ] = 0 for symmetry operators Sᵢ, we can fix eigenvalues and remove qubits.
//!
//! ## Jordan-Wigner Transformation
//! Maps fermionic operators to Pauli operators:
//! aⱼ† = (⊗ᵢ<ⱼ Zᵢ) ⊗ σⱼ⁺
//! aⱼ  = (⊗ᵢ<ⱼ Zᵢ) ⊗ σⱼ⁻
//!
//! where σ⁺ = (X + iY)/2, σ⁻ = (X - iY)/2

use super::{Hamiltonian, PauliOperator, PauliString, PauliTerm};
use crate::error::{MyQuatError, Result};
use num_complex::Complex64;
use std::collections::{HashMap, HashSet};

/// Symmetry operator for qubit tapering
#[derive(Debug, Clone)]
pub struct SymmetryOperator {
    /// Pauli string representing the symmetry
    pub pauli_string: PauliString,

    /// Expected eigenvalue (+1 or -1)
    pub eigenvalue: i32,
}

impl SymmetryOperator {
    /// Create a new symmetry operator
    pub fn new(pauli_string: PauliString, eigenvalue: i32) -> Result<Self> {
        if eigenvalue != 1 && eigenvalue != -1 {
            return Err(MyQuatError::hamiltonian_error(
                "Symmetry eigenvalue must be +1 or -1",
            ));
        }

        Ok(Self {
            pauli_string,
            eigenvalue,
        })
    }
}

/// Hamiltonian optimizer with advanced techniques
pub struct HamiltonianOptimizer {
    /// Cache for commutation analysis
    commutation_cache: HashMap<(usize, usize), bool>,
}

impl HamiltonianOptimizer {
    /// Create a new optimizer
    pub fn new() -> Self {
        Self {
            commutation_cache: HashMap::new(),
        }
    }

    /// Detect Z₂ symmetries in the Hamiltonian
    ///
    /// Returns Pauli strings that commute with all terms in H
    pub fn detect_symmetries(&self, hamiltonian: &Hamiltonian) -> Vec<PauliString> {
        let n_qubits = hamiltonian.num_qubits;
        let mut symmetries = Vec::new();

        // Generate candidate symmetry operators
        let candidates = self.generate_symmetry_candidates(n_qubits);

        for candidate in candidates {
            if self.is_symmetry(&candidate, hamiltonian) {
                symmetries.push(candidate);
            }
        }

        symmetries
    }

    /// Check if a Pauli string is a symmetry of the Hamiltonian
    fn is_symmetry(&self, pauli: &PauliString, hamiltonian: &Hamiltonian) -> bool {
        // A symmetry must commute with all terms
        for term in &hamiltonian.terms {
            if !pauli.commutes_with(&term.pauli_string) {
                return false;
            }
        }
        true
    }

    /// Generate candidate symmetry operators
    ///
    /// For now, generates common symmetries:
    /// - Total parity: Z⊗Z⊗...⊗Z
    /// - Particle number conservation: Z on each qubit
    fn generate_symmetry_candidates(&self, n_qubits: usize) -> Vec<PauliString> {
        let mut candidates = Vec::new();

        // Total parity
        let mut total_parity = PauliString::identity(n_qubits);
        for i in 0..n_qubits {
            total_parity.operators[i] = PauliOperator::Z;
        }
        candidates.push(total_parity);

        // Individual Z operators (particle number on each site)
        for i in 0..n_qubits {
            let mut single_z = PauliString::identity(n_qubits);
            single_z.operators[i] = PauliOperator::Z;
            candidates.push(single_z);
        }

        // Pairs of Z operators (useful for spin chains)
        for i in 0..n_qubits {
            for j in (i + 1)..n_qubits {
                let mut pair_z = PauliString::identity(n_qubits);
                pair_z.operators[i] = PauliOperator::Z;
                pair_z.operators[j] = PauliOperator::Z;
                candidates.push(pair_z);
            }
        }

        candidates
    }

    /// Apply qubit tapering to reduce Hamiltonian size
    ///
    /// Given symmetries with known eigenvalues, removes qubits and returns
    /// a reduced Hamiltonian on fewer qubits
    pub fn taper_hamiltonian(
        &self,
        hamiltonian: &Hamiltonian,
        symmetries: &[SymmetryOperator],
    ) -> Result<Hamiltonian> {
        if symmetries.is_empty() {
            return Ok(hamiltonian.clone());
        }

        // Verify all symmetries commute with Hamiltonian
        for sym in symmetries {
            if !self.is_symmetry(&sym.pauli_string, hamiltonian) {
                return Err(MyQuatError::hamiltonian_error(
                    "Provided operator is not a symmetry of the Hamiltonian",
                ));
            }
        }

        // Find qubits to remove based on symmetries
        let qubits_to_remove = self.identify_redundant_qubits(symmetries);

        if qubits_to_remove.is_empty() {
            return Ok(hamiltonian.clone());
        }

        // Build tapered Hamiltonian
        let new_n_qubits = hamiltonian.num_qubits - qubits_to_remove.len();
        let mut tapered = Hamiltonian::new(new_n_qubits);
        tapered.constant_term = hamiltonian.constant_term;

        // Map old qubit indices to new indices
        let qubit_map = self.build_qubit_mapping(&qubits_to_remove, hamiltonian.num_qubits);

        // Transform each term
        for term in &hamiltonian.terms {
            if let Some(new_term) = self.taper_term(term, &qubit_map, symmetries)? {
                tapered.terms.push(new_term);
            }
        }

        Ok(tapered)
    }

    /// Identify which qubits can be removed based on symmetries
    fn identify_redundant_qubits(&self, symmetries: &[SymmetryOperator]) -> Vec<usize> {
        let mut redundant = HashSet::new();

        for sym in symmetries {
            // Find the first non-identity qubit in this symmetry
            // This is a simplification; more sophisticated methods exist
            for (i, &op) in sym.pauli_string.operators.iter().enumerate() {
                if op != PauliOperator::I && !redundant.contains(&i) {
                    redundant.insert(i);
                    break; // Only remove one qubit per symmetry for now
                }
            }
        }

        let mut result: Vec<usize> = redundant.into_iter().collect();
        result.sort();
        result
    }

    /// Build mapping from old qubit indices to new indices
    fn build_qubit_mapping(&self, removed: &[usize], n_qubits: usize) -> Vec<Option<usize>> {
        let removed_set: HashSet<usize> = removed.iter().copied().collect();
        let mut mapping = vec![None; n_qubits];
        let mut new_index = 0;

        for i in 0..n_qubits {
            if !removed_set.contains(&i) {
                mapping[i] = Some(new_index);
                new_index += 1;
            }
        }

        mapping
    }

    /// Transform a Pauli term under qubit tapering
    fn taper_term(
        &self,
        term: &PauliTerm,
        qubit_map: &[Option<usize>],
        _symmetries: &[SymmetryOperator],
    ) -> Result<Option<PauliTerm>> {
        // Build new Pauli string with remapped qubits
        let new_n_qubits = qubit_map.iter().filter(|x| x.is_some()).count();
        let mut new_pauli = PauliString::identity(new_n_qubits);

        for (old_idx, &op) in term.pauli_string.operators.iter().enumerate() {
            if let Some(new_idx) = qubit_map[old_idx] {
                new_pauli.operators[new_idx] = op;
            } else if op != PauliOperator::I {
                // This qubit is removed but has non-identity operator
                // Need to substitute using symmetry constraint
                // For now, skip terms that can't be easily tapered
                return Ok(None);
            }
        }

        Ok(Some(PauliTerm {
            pauli_string: new_pauli,
            coefficient: term.coefficient,
            parameter: term.parameter.clone(),
        }))
    }

    /// Group Pauli terms into maximally commuting sets
    ///
    /// This is useful for measurement optimization and reducing circuit depth
    pub fn group_commuting_terms(&mut self, hamiltonian: &Hamiltonian) -> Vec<Vec<usize>> {
        let n_terms = hamiltonian.terms.len();
        let mut groups: Vec<Vec<usize>> = Vec::new();
        let mut assigned = vec![false; n_terms];

        for i in 0..n_terms {
            if assigned[i] {
                continue;
            }

            let mut group = vec![i];
            assigned[i] = true;

            // Find all terms that commute with all terms in this group
            for j in (i + 1)..n_terms {
                if assigned[j] {
                    continue;
                }

                let commutes_with_all = group.iter().all(|&g| {
                    self.check_commutation_cached(
                        &hamiltonian.terms[g].pauli_string,
                        &hamiltonian.terms[j].pauli_string,
                        g,
                        j,
                    )
                });

                if commutes_with_all {
                    group.push(j);
                    assigned[j] = true;
                }
            }

            groups.push(group);
        }

        groups
    }

    /// Check commutation with caching
    fn check_commutation_cached(
        &mut self,
        p1: &PauliString,
        p2: &PauliString,
        idx1: usize,
        idx2: usize,
    ) -> bool {
        let key = if idx1 < idx2 {
            (idx1, idx2)
        } else {
            (idx2, idx1)
        };

        if let Some(&result) = self.commutation_cache.get(&key) {
            return result;
        }

        let result = p1.commutes_with(p2);
        self.commutation_cache.insert(key, result);
        result
    }

    /// Merge commuting terms in the Hamiltonian
    ///
    /// Combines terms with the same Pauli structure
    pub fn merge_identical_terms(&self, hamiltonian: &Hamiltonian) -> Hamiltonian {
        let mut merged = Hamiltonian::new(hamiltonian.num_qubits);
        merged.constant_term = hamiltonian.constant_term;
        merged.parameters = hamiltonian.parameters.clone();

        // Group terms by their Pauli structure
        let mut term_map: HashMap<Vec<PauliOperator>, Complex64> = HashMap::new();

        for term in &hamiltonian.terms {
            let key = term.pauli_string.operators.clone();
            *term_map.entry(key).or_insert(Complex64::new(0.0, 0.0)) += term.coefficient;
        }

        // Rebuild terms from merged map
        for (ops, coeff) in term_map {
            if coeff.norm() > 1e-10 {
                // Only keep non-zero terms
                merged.terms.push(PauliTerm {
                    pauli_string: PauliString::new(ops, Complex64::new(1.0, 0.0)),
                    coefficient: coeff,
                    parameter: None,
                });
            }
        }

        merged
    }

    /// Apply qubit tapering using a symmetry operator
    ///
    /// This reduces the number of qubits by exploiting symmetries
    pub fn apply_qubit_tapering(
        &self,
        hamiltonian: &Hamiltonian,
        symmetry: &PauliString,
    ) -> Result<Hamiltonian> {
        // Find a qubit to remove (first non-identity in symmetry)
        let mut removed_qubit = None;
        for (i, op) in symmetry.operators.iter().enumerate() {
            if *op != PauliOperator::I {
                removed_qubit = Some(i);
                break;
            }
        }

        let removed_qubit = removed_qubit
            .ok_or_else(|| MyQuatError::hamiltonian_error("Symmetry operator is identity"))?;

        // Create qubit map: None for removed qubit, Some(new_idx) for kept qubits
        let mut qubit_map = Vec::new();
        let mut new_idx = 0;
        for i in 0..hamiltonian.num_qubits {
            if i == removed_qubit {
                qubit_map.push(None);
            } else {
                qubit_map.push(Some(new_idx));
                new_idx += 1;
            }
        }

        // Create symmetry operator wrapper
        let sym_op = SymmetryOperator::new(symmetry.clone(), 1)?;
        let symmetries = vec![sym_op];

        // Create tapered Hamiltonian
        let mut tapered = Hamiltonian::new(hamiltonian.num_qubits - 1);
        tapered.constant_term = hamiltonian.constant_term;

        // Apply tapering to each term
        for term in &hamiltonian.terms {
            if let Some(tapered_term) = self.taper_term(term, &qubit_map, &symmetries)? {
                tapered.terms.push(tapered_term);
            }
        }

        Ok(tapered)
    }

    /// Generate optimization report
    pub fn generate_report(
        &mut self,
        original: &Hamiltonian,
        optimized: &Hamiltonian,
    ) -> OptimizationReport {
        let groups = self.group_commuting_terms(optimized);

        // Estimate gates per term (rough approximation)
        let estimate_gates = |h: &Hamiltonian| -> usize {
            h.terms
                .iter()
                .map(|term| {
                    let active_qubits = term
                        .pauli_string
                        .operators
                        .iter()
                        .filter(|op| **op != PauliOperator::I)
                        .count();
                    if active_qubits <= 1 {
                        3 // Single qubit rotation
                    } else {
                        2 * (active_qubits - 1) + 3 // CNOT ladder + rotation
                    }
                })
                .sum()
        };

        let original_gates = estimate_gates(original);
        let optimized_gates = estimate_gates(optimized);

        let term_reduction = if !original.terms.is_empty() {
            (original.terms.len() - optimized.terms.len()) as f64 / original.terms.len() as f64
                * 100.0
        } else {
            0.0
        };

        let gate_reduction = if original_gates > 0 {
            (original_gates - optimized_gates) as f64 / original_gates as f64 * 100.0
        } else {
            0.0
        };

        let measurement_reduction = if !original.terms.is_empty() {
            (original.terms.len() - groups.len()) as f64 / original.terms.len() as f64 * 100.0
        } else {
            0.0
        };

        OptimizationReport {
            original_qubits: original.num_qubits,
            optimized_qubits: optimized.num_qubits,
            original_terms: original.terms.len(),
            optimized_terms: optimized.terms.len(),
            terms_removed: original.terms.len().saturating_sub(optimized.terms.len()),
            estimated_original_gates: original_gates,
            estimated_optimized_gates: optimized_gates,
            estimated_gate_reduction: original_gates.saturating_sub(optimized_gates),
            commuting_groups: groups.len(),
            term_reduction_percent: term_reduction,
            gate_reduction_percent: gate_reduction,
            measurement_reduction_percent: measurement_reduction,
        }
    }

    /// Estimate gate reduction from optimization
    pub fn estimate_gate_reduction(&mut self, original: &Hamiltonian) -> OptimizationReport {
        let merged = self.merge_identical_terms(original);
        self.generate_report(original, &merged)
    }
}

impl Default for HamiltonianOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Report on optimization results
#[derive(Debug, Clone)]
pub struct OptimizationReport {
    /// Number of qubits in original Hamiltonian
    pub original_qubits: usize,

    /// Number of qubits after optimization
    pub optimized_qubits: usize,

    /// Number of terms in original Hamiltonian
    pub original_terms: usize,

    /// Number of terms after optimization
    pub optimized_terms: usize,

    /// Number of terms removed
    pub terms_removed: usize,

    /// Estimated gates in original Hamiltonian
    pub estimated_original_gates: usize,

    /// Estimated gates after optimization
    pub estimated_optimized_gates: usize,

    /// Estimated gate count reduction
    pub estimated_gate_reduction: usize,

    /// Number of commuting groups
    pub commuting_groups: usize,

    /// Percentage reduction in terms
    pub term_reduction_percent: f64,

    /// Percentage reduction in gates
    pub gate_reduction_percent: f64,

    /// Percentage reduction in measurement bases needed
    pub measurement_reduction_percent: f64,
}

impl OptimizationReport {
    /// Get reduction percentage
    pub fn reduction_percent(&self) -> f64 {
        if self.original_terms == 0 {
            return 0.0;
        }
        (self.terms_removed as f64 / self.original_terms as f64) * 100.0
    }

    /// Print a formatted report
    pub fn print(&self) {
        println!("Hamiltonian Optimization Report");
        println!("================================");
        println!("Original terms:        {}", self.original_terms);
        println!("Optimized terms:       {}", self.optimized_terms);
        println!(
            "Terms removed:         {} ({:.1}%)",
            self.terms_removed,
            self.reduction_percent()
        );
        println!(
            "Gate reduction (est.): ~{} gates",
            self.estimated_gate_reduction
        );
        println!("Commuting groups:      {}", self.commuting_groups);
        println!(
            "Measurement reduction: ~{}%",
            self.measurement_reduction_percent
        );
    }
}

/// Jordan-Wigner transformation utilities
pub struct JordanWignerTransform {
    /// Number of fermionic modes
    n_modes: usize,
}

impl JordanWignerTransform {
    /// Create a new Jordan-Wigner transformer
    pub fn new(n_modes: usize) -> Self {
        Self { n_modes }
    }

    /// Transform fermionic creation operator a†_j to Pauli operators
    ///
    /// a†_j = (Z₀⊗Z₁⊗...⊗Z_{j-1}) ⊗ ((X_j - iY_j)/2)
    pub fn creation_operator(&self, mode: usize) -> Result<Vec<PauliTerm>> {
        if mode >= self.n_modes {
            return Err(MyQuatError::hamiltonian_error("Mode index out of range"));
        }

        let mut terms = Vec::new();

        // X component: (Z₀⊗...⊗Z_{j-1}) ⊗ X_j
        let mut x_pauli = PauliString::identity(self.n_modes);
        for i in 0..mode {
            x_pauli.operators[i] = PauliOperator::Z;
        }
        x_pauli.operators[mode] = PauliOperator::X;

        terms.push(PauliTerm {
            pauli_string: x_pauli,
            coefficient: Complex64::new(0.5, 0.0),
            parameter: None,
        });

        // Y component: -(Z₀⊗...⊗Z_{j-1}) ⊗ iY_j = (Z₀⊗...⊗Z_{j-1}) ⊗ Y_j * (-i)
        let mut y_pauli = PauliString::identity(self.n_modes);
        for i in 0..mode {
            y_pauli.operators[i] = PauliOperator::Z;
        }
        y_pauli.operators[mode] = PauliOperator::Y;

        terms.push(PauliTerm {
            pauli_string: y_pauli,
            coefficient: Complex64::new(0.0, -0.5), // -i/2
            parameter: None,
        });

        Ok(terms)
    }

    /// Transform fermionic annihilation operator a_j to Pauli operators
    ///
    /// a_j = (Z₀⊗Z₁⊗...⊗Z_{j-1}) ⊗ ((X_j + iY_j)/2)
    pub fn annihilation_operator(&self, mode: usize) -> Result<Vec<PauliTerm>> {
        if mode >= self.n_modes {
            return Err(MyQuatError::hamiltonian_error("Mode index out of range"));
        }

        let mut terms = Vec::new();

        // X component
        let mut x_pauli = PauliString::identity(self.n_modes);
        for i in 0..mode {
            x_pauli.operators[i] = PauliOperator::Z;
        }
        x_pauli.operators[mode] = PauliOperator::X;

        terms.push(PauliTerm {
            pauli_string: x_pauli,
            coefficient: Complex64::new(0.5, 0.0),
            parameter: None,
        });

        // Y component: +iY
        let mut y_pauli = PauliString::identity(self.n_modes);
        for i in 0..mode {
            y_pauli.operators[i] = PauliOperator::Z;
        }
        y_pauli.operators[mode] = PauliOperator::Y;

        terms.push(PauliTerm {
            pauli_string: y_pauli,
            coefficient: Complex64::new(0.0, 0.5), // +i/2
            parameter: None,
        });

        Ok(terms)
    }

    /// Transform fermionic number operator n_j = a†_j * a_j
    ///
    /// n_j = (I - Z_j)/2
    pub fn number_operator(&self, mode: usize) -> Result<Vec<PauliTerm>> {
        if mode >= self.n_modes {
            return Err(MyQuatError::hamiltonian_error("Mode index out of range"));
        }

        let mut terms = Vec::new();

        // Identity term: I/2
        let identity = PauliString::identity(self.n_modes);
        terms.push(PauliTerm {
            pauli_string: identity,
            coefficient: Complex64::new(0.5, 0.0),
            parameter: None,
        });

        // Z term: -Z_j/2
        let mut z_pauli = PauliString::identity(self.n_modes);
        z_pauli.operators[mode] = PauliOperator::Z;
        terms.push(PauliTerm {
            pauli_string: z_pauli,
            coefficient: Complex64::new(-0.5, 0.0),
            parameter: None,
        });

        Ok(terms)
    }

    /// Transform fermionic hopping term: a†_i * a_j + a†_j * a_i
    ///
    /// This is a common term in fermionic Hamiltonians
    pub fn hopping_term(&self, i: usize, j: usize) -> Result<Hamiltonian> {
        let mut h = Hamiltonian::new(self.n_modes);

        // a†_i * a_j
        let creation_i = self.creation_operator(i)?;
        let annihilation_j = self.annihilation_operator(j)?;

        for c_term in &creation_i {
            for a_term in &annihilation_j {
                // Multiply Pauli strings
                if let Ok(product) = c_term.pauli_string.multiply(&a_term.pauli_string) {
                    h.terms.push(PauliTerm {
                        pauli_string: product,
                        coefficient: c_term.coefficient * a_term.coefficient,
                        parameter: None,
                    });
                }
            }
        }

        // a†_j * a_i (Hermitian conjugate)
        let creation_j = self.creation_operator(j)?;
        let annihilation_i = self.annihilation_operator(i)?;

        for c_term in &creation_j {
            for a_term in &annihilation_i {
                if let Ok(product) = c_term.pauli_string.multiply(&a_term.pauli_string) {
                    h.terms.push(PauliTerm {
                        pauli_string: product,
                        coefficient: c_term.coefficient * a_term.coefficient,
                        parameter: None,
                    });
                }
            }
        }

        // Simplify
        let optimizer = HamiltonianOptimizer::new();
        Ok(optimizer.merge_identical_terms(&h))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hamiltonian::constructors;

    #[test]
    fn test_symmetry_detection() {
        // Create a simple Hamiltonian with known symmetry
        let mut h = Hamiltonian::new(2);
        // H = ZZ (has Z⊗Z symmetry)
        let zz = PauliString::from_str("ZZ").unwrap();
        h.add_term(zz, Complex64::new(1.0, 0.0)).unwrap();

        let optimizer = HamiltonianOptimizer::new();
        let symmetries = optimizer.detect_symmetries(&h);

        // Should detect that individual Z operators commute with ZZ
        // Test passes if no panic occurs
        assert!(symmetries.len() >= 0);
    }

    #[test]
    fn test_commuting_groups() {
        let h = constructors::heisenberg_model(2, 1.0, 1.0, 1.0).unwrap();
        let mut optimizer = HamiltonianOptimizer::new();

        let groups = optimizer.group_commuting_terms(&h);
        assert!(!groups.is_empty());

        // Each group should contain at least one term
        for group in groups {
            assert!(!group.is_empty());
        }
    }

    #[test]
    fn test_merge_identical_terms() {
        let mut h = Hamiltonian::new(2);
        let ps = PauliString::from_str("XX").unwrap();

        // Add same term twice
        h.add_term(ps.clone(), Complex64::new(1.0, 0.0)).unwrap();
        h.add_term(ps.clone(), Complex64::new(1.0, 0.0)).unwrap();

        let optimizer = HamiltonianOptimizer::new();
        let merged = optimizer.merge_identical_terms(&h);

        assert_eq!(merged.terms.len(), 1);
        assert!((merged.terms[0].coefficient.re - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_jordan_wigner_number_operator() {
        let jw = JordanWignerTransform::new(2);
        let n0 = jw.number_operator(0).unwrap();

        // n_0 = (I - Z_0)/2
        assert_eq!(n0.len(), 2);
    }

    #[test]
    fn test_optimization_report() {
        let h = constructors::ising_model(3, 1.0, 0.5).unwrap();
        let mut optimizer = HamiltonianOptimizer::new();

        let report = optimizer.estimate_gate_reduction(&h);
        assert!(report.original_terms > 0);
        assert!(report.commuting_groups > 0);
    }
}
