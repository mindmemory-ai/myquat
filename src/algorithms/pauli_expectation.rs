// Pauli Expectation Value Computation
// Author: gA4ss
//
// This module provides efficient computation of Pauli expectation values
// for quantum chemistry and VQE applications.
//
// # Mathematical Background
//
// For a quantum state |ψ⟩ and Pauli string P, the expectation value is:
// ⟨P⟩ = ⟨ψ|P|ψ⟩
//
// For a Hamiltonian H = Σ_i c_i P_i, the energy is:
// E = ⟨ψ|H|ψ⟩ = Σ_i c_i ⟨ψ|P_i|ψ⟩
//
// # Optimization Strategies
//
// 1. **Direct state vector method**: O(2^n) for each Pauli string
// 2. **Measurement-based**: Sample from quantum circuit
// 3. **Grouped measurements**: Measure commuting Paulis together

use crate::hamiltonian::{Hamiltonian, PauliOperator, PauliString};
use num_complex::Complex64;
use std::collections::HashMap;

/// Pauli expectation value computer
pub struct PauliExpectationComputer {
    /// Use measurement-based estimation
    use_measurements: bool,
    /// Number of shots for measurement-based estimation
    num_shots: usize,
}

impl PauliExpectationComputer {
    /// Create a new expectation computer with default settings
    pub fn new() -> Self {
        PauliExpectationComputer {
            use_measurements: false,
            num_shots: 1000,
        }
    }

    /// Create a measurement-based expectation computer
    pub fn with_measurements(num_shots: usize) -> Self {
        PauliExpectationComputer {
            use_measurements: true,
            num_shots,
        }
    }

    /// Compute expectation value of a single Pauli string
    ///
    /// # Arguments
    ///
    /// * `state` - Quantum state vector
    /// * `pauli_string` - Pauli string operator
    ///
    /// # Returns
    ///
    /// Complex expectation value ⟨ψ|P|ψ⟩
    pub fn compute_pauli_expectation(
        &self,
        state: &[Complex64],
        pauli_string: &PauliString,
    ) -> Complex64 {
        let n = pauli_string.num_qubits();
        let dim = 1 << n;

        if state.len() != dim {
            return Complex64::new(0.0, 0.0);
        }

        let mut result = Complex64::new(0.0, 0.0);

        // Iterate over all basis states
        for i in 0..dim {
            let (j, phase) = apply_pauli_string(i, &pauli_string.operators);
            result += state[i].conj() * phase * state[j];
        }

        result
    }

    /// Compute Hamiltonian expectation value
    ///
    /// # Arguments
    ///
    /// * `state` - Quantum state vector
    /// * `hamiltonian` - Hamiltonian operator
    ///
    /// # Returns
    ///
    /// Real energy value E = ⟨ψ|H|ψ⟩
    pub fn compute_hamiltonian_expectation(
        &self,
        state: &[Complex64],
        hamiltonian: &Hamiltonian,
    ) -> f64 {
        let mut energy = hamiltonian.constant_term.re;

        for term in &hamiltonian.terms {
            let pauli_exp = self.compute_pauli_expectation(state, &term.pauli_string);
            energy += (term.coefficient * pauli_exp).re;
        }

        energy
    }

    /// Compute expectation value with variance estimation
    ///
    /// Returns (expectation, variance)
    pub fn compute_with_variance(
        &self,
        state: &[Complex64],
        pauli_string: &PauliString,
    ) -> (f64, f64) {
        let expectation = self.compute_pauli_expectation(state, pauli_string).re;

        // Compute ⟨P²⟩ for variance
        // For Pauli operators, P² = I, so ⟨P²⟩ = 1
        let variance = 1.0 - expectation * expectation;

        (expectation, variance.max(0.0))
    }
}

impl Default for PauliExpectationComputer {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply Pauli string to a basis state index
///
/// Returns (resulting index, phase factor)
fn apply_pauli_string(index: usize, paulis: &[PauliOperator]) -> (usize, Complex64) {
    let mut result_index = index;
    let mut phase = Complex64::new(1.0, 0.0);

    for (qubit, pauli) in paulis.iter().enumerate() {
        let bit = (index >> qubit) & 1;

        match pauli {
            PauliOperator::I => {}
            PauliOperator::X => {
                // X flips the bit
                result_index ^= 1 << qubit;
            }
            PauliOperator::Y => {
                // Y flips the bit and adds phase
                result_index ^= 1 << qubit;
                phase *= if bit == 0 {
                    Complex64::new(0.0, 1.0) // i
                } else {
                    Complex64::new(0.0, -1.0) // -i
                };
            }
            PauliOperator::Z => {
                // Z adds phase based on bit value
                if bit == 1 {
                    phase *= Complex64::new(-1.0, 0.0);
                }
            }
        }
    }

    (result_index, phase)
}

/// Grouped Pauli measurement strategy
///
/// Groups commuting Pauli strings for efficient measurement
pub struct GroupedPauliMeasurement {
    /// Groups of commuting Pauli strings
    groups: Vec<Vec<usize>>,
    /// Pauli strings
    pauli_strings: Vec<PauliString>,
}

impl GroupedPauliMeasurement {
    /// Create a new grouped measurement strategy
    pub fn new(pauli_strings: Vec<PauliString>) -> Self {
        let groups = group_commuting_paulis(&pauli_strings);
        GroupedPauliMeasurement {
            groups,
            pauli_strings,
        }
    }

    /// Get number of measurement groups
    pub fn num_groups(&self) -> usize {
        self.groups.len()
    }

    /// Get group indices
    pub fn groups(&self) -> &[Vec<usize>] {
        &self.groups
    }
}

/// Group commuting Pauli strings together
///
/// Two Pauli strings commute if they differ in an even number of positions
/// where both are non-identity and different.
fn group_commuting_paulis(pauli_strings: &[PauliString]) -> Vec<Vec<usize>> {
    let n = pauli_strings.len();
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut assigned = vec![false; n];

    for i in 0..n {
        if assigned[i] {
            continue;
        }

        let mut group = vec![i];
        assigned[i] = true;

        for j in (i + 1)..n {
            if assigned[j] {
                continue;
            }

            // Check if j commutes with all in current group
            let mut commutes_with_all = true;
            for &k in &group {
                if !paulis_commute(&pauli_strings[k], &pauli_strings[j]) {
                    commutes_with_all = false;
                    break;
                }
            }

            if commutes_with_all {
                group.push(j);
                assigned[j] = true;
            }
        }

        groups.push(group);
    }

    groups
}

/// Check if two Pauli strings commute
fn paulis_commute(p1: &PauliString, p2: &PauliString) -> bool {
    if p1.num_qubits() != p2.num_qubits() {
        return false;
    }

    let mut diff_count = 0;

    for i in 0..p1.num_qubits() {
        let op1 = &p1.operators[i];
        let op2 = &p2.operators[i];

        // Count positions where both are non-I and different
        if !matches!(op1, PauliOperator::I)
            && !matches!(op2, PauliOperator::I)
            && !pauli_ops_equal(op1, op2)
        {
            diff_count += 1;
        }
    }

    // Commute if even number of differences
    diff_count % 2 == 0
}

/// Check if two Pauli operators are equal
fn pauli_ops_equal(op1: &PauliOperator, op2: &PauliOperator) -> bool {
    matches!(
        (op1, op2),
        (PauliOperator::I, PauliOperator::I)
            | (PauliOperator::X, PauliOperator::X)
            | (PauliOperator::Y, PauliOperator::Y)
            | (PauliOperator::Z, PauliOperator::Z)
    )
}

/// Efficient expectation value computation using symmetries
pub struct SymmetryAwareExpectation {
    /// Cache for computed expectation values
    cache: HashMap<String, Complex64>,
}

impl SymmetryAwareExpectation {
    /// Create a new symmetry-aware expectation computer
    pub fn new() -> Self {
        SymmetryAwareExpectation {
            cache: HashMap::new(),
        }
    }

    /// Compute expectation with caching
    pub fn compute_cached(&mut self, state: &[Complex64], pauli_string: &PauliString) -> Complex64 {
        let key = pauli_string_to_key(pauli_string);

        if let Some(&value) = self.cache.get(&key) {
            return value;
        }

        let computer = PauliExpectationComputer::new();
        let value = computer.compute_pauli_expectation(state, pauli_string);
        self.cache.insert(key, value);

        value
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for SymmetryAwareExpectation {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert Pauli string to cache key
fn pauli_string_to_key(pauli_string: &PauliString) -> String {
    pauli_string
        .operators
        .iter()
        .map(|op| match op {
            PauliOperator::I => 'I',
            PauliOperator::X => 'X',
            PauliOperator::Y => 'Y',
            PauliOperator::Z => 'Z',
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_pauli_expectation_z() {
        // State |0⟩
        let state = vec![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)];
        let pauli = PauliString::new(vec![PauliOperator::Z], Complex64::new(1.0, 0.0));

        let computer = PauliExpectationComputer::new();
        let exp = computer.compute_pauli_expectation(&state, &pauli);

        // ⟨0|Z|0⟩ = 1
        assert!((exp.re - 1.0).abs() < 1e-10);
        assert!(exp.im.abs() < 1e-10);
    }

    #[test]
    fn test_pauli_expectation_x() {
        // State |+⟩ = (|0⟩ + |1⟩)/√2
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        let state = vec![
            Complex64::new(inv_sqrt2, 0.0),
            Complex64::new(inv_sqrt2, 0.0),
        ];
        let pauli = PauliString::new(vec![PauliOperator::X], Complex64::new(1.0, 0.0));

        let computer = PauliExpectationComputer::new();
        let exp = computer.compute_pauli_expectation(&state, &pauli);

        // ⟨+|X|+⟩ = 1
        assert!((exp.re - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_pauli_commutation() {
        // X and Z don't commute
        let px = PauliString::new(vec![PauliOperator::X], Complex64::new(1.0, 0.0));
        let pz = PauliString::new(vec![PauliOperator::Z], Complex64::new(1.0, 0.0));
        assert!(!paulis_commute(&px, &pz));

        // X and X commute
        assert!(paulis_commute(&px, &px));

        // ZI and IZ commute
        let pzi = PauliString::new(
            vec![PauliOperator::Z, PauliOperator::I],
            Complex64::new(1.0, 0.0),
        );
        let piz = PauliString::new(
            vec![PauliOperator::I, PauliOperator::Z],
            Complex64::new(1.0, 0.0),
        );
        assert!(paulis_commute(&pzi, &piz));
    }

    #[test]
    fn test_grouped_measurements() {
        let paulis = vec![
            PauliString::new(
                vec![PauliOperator::Z, PauliOperator::I],
                Complex64::new(1.0, 0.0),
            ),
            PauliString::new(
                vec![PauliOperator::I, PauliOperator::Z],
                Complex64::new(1.0, 0.0),
            ),
            PauliString::new(
                vec![PauliOperator::X, PauliOperator::I],
                Complex64::new(1.0, 0.0),
            ),
        ];

        let grouped = GroupedPauliMeasurement::new(paulis);

        // ZI and IZ should be in same group (commute)
        // XI should be in different group (doesn't commute with ZI)
        assert!(grouped.num_groups() >= 2);
    }

    #[test]
    fn test_expectation_with_variance() {
        let state = vec![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)];
        let pauli = PauliString::new(vec![PauliOperator::Z], Complex64::new(1.0, 0.0));

        let computer = PauliExpectationComputer::new();
        let (exp, var) = computer.compute_with_variance(&state, &pauli);

        // ⟨Z⟩ = 1, Var(Z) = 0 for eigenstate
        assert!((exp - 1.0).abs() < 1e-10);
        assert!(var < 1e-10);
    }

    #[test]
    fn test_cached_expectation() {
        let state = vec![Complex64::new(1.0, 0.0), Complex64::new(0.0, 0.0)];
        let pauli = PauliString::new(vec![PauliOperator::Z], Complex64::new(1.0, 0.0));

        let mut computer = SymmetryAwareExpectation::new();

        // First computation
        let exp1 = computer.compute_cached(&state, &pauli);

        // Second computation (should use cache)
        let exp2 = computer.compute_cached(&state, &pauli);

        assert!((exp1.re - exp2.re).abs() < 1e-10);
    }
}
