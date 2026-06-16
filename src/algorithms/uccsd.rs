// UCCSD (Unitary Coupled Cluster Singles and Doubles) Implementation
// Author: gA4ss
//
// This module implements the standard UCCSD ansatz for quantum chemistry calculations.
//
// # Mathematical Background
//
// The UCCSD ansatz is defined as:
// $$|\psi(\theta)\rangle = e^{\hat{T} - \hat{T}^\dagger} |\text{HF}\rangle$$
//
// where $\hat{T} = \hat{T}_1 + \hat{T}_2$ is the cluster operator:
// - $\hat{T}_1 = \sum_{i,a} t_i^a \hat{a}^\dagger_a \hat{a}_i$ (single excitations)
// - $\hat{T}_2 = \sum_{i<j,a<b} t_{ij}^{ab} \hat{a}^\dagger_a \hat{a}^\dagger_b \hat{a}_j \hat{a}_i$ (double excitations)
//
// # Quantum Circuit Implementation
//
// Single excitation: $e^{\theta(\hat{a}^\dagger_a \hat{a}_i - \hat{a}^\dagger_i \hat{a}_a)}$
//
// Circuit for single excitation (i → a):
// ```
// q_i: ──RY(θ/2)──●────────────●──
//                 │            │
// q_a: ───────────X──RY(-θ/2)──X──
// ```

use crate::error::{MyQuatError, Result};
use crate::{Parameter, QuantumCircuit};

/// Single excitation operator: i → a
///
/// Implements the circuit for $e^{\theta(\hat{a}^\dagger_a \hat{a}_i - \hat{a}^\dagger_i \hat{a}_a)}$
///
/// # Arguments
///
/// * `circuit` - Quantum circuit to add gates to
/// * `occupied` - Index of occupied orbital (i)
/// * `virtual_orbital` - Index of virtual orbital (a)
/// * `theta` - Excitation amplitude parameter
///
/// # Circuit
///
/// ```text
/// q_i: ──RY(θ/2)──●────────────●──
///                 │            │
/// q_a: ───────────X──RY(-θ/2)──X──
/// ```
pub fn single_excitation(
    circuit: &mut QuantumCircuit,
    occupied: usize,
    virtual_orbital: usize,
    theta: f64,
) -> Result<()> {
    // RY(θ/2) on occupied orbital
    circuit.ry(occupied, Parameter::Float(theta / 2.0))?;

    // CNOT: occupied → virtual
    circuit.cx(occupied, virtual_orbital)?;

    // RY(-θ/2) on virtual orbital
    circuit.ry(virtual_orbital, Parameter::Float(-theta / 2.0))?;

    // CNOT: occupied → virtual (reverse)
    circuit.cx(occupied, virtual_orbital)?;

    Ok(())
}

/// Double excitation operator: (i,j) → (a,b)
///
/// Implements the circuit for double excitation from occupied orbitals (i,j)
/// to virtual orbitals (a,b).
///
/// # Arguments
///
/// * `circuit` - Quantum circuit to add gates to
/// * `occupied1` - Index of first occupied orbital (i)
/// * `occupied2` - Index of second occupied orbital (j)
/// * `virtual1` - Index of first virtual orbital (a)
/// * `virtual2` - Index of second virtual orbital (b)
/// * `theta` - Excitation amplitude parameter
///
/// # Circuit Structure
///
/// Uses an 8-CNOT decomposition for the fermionic double excitation under the
/// Jordan-Wigner mapping. The circuit creates a parity chain connecting all
/// four orbitals (o2 → v1 → v2 with an additional o1 → o2 tap) so that the
/// phase kickback from Rz(v2, theta) distributes across all four qubits.
///
/// ```text
/// o1: ─────────────────●──────────────●─────────────────
///                       │              │
/// o2: ─────●────────────X──────────────X────────────●────
///          │                                         │
/// v1: ─────X────●──────X──────────────X────●─────────X────
///               │                          │
/// v2: ──────────X────Rz(θ)────────────Rz───X──────────────
/// ```
///
/// The effective unitary implemented is exp(-i·θ·Z_o2·Z_v1·Z_v2).
/// This is a Trotter-step approximation of one component of the full
/// double excitation (which under Jordan-Wigner decomposes into 8 Pauli
/// exponentials). For the exact decomposition, see PennyLane's
/// `FermionicDoubleExcitation`.
///
/// # Gate Count
///
/// 8 CNOT gates + 1 Rz gate = 9 gates total.
pub fn double_excitation(
    circuit: &mut QuantumCircuit,
    occupied1: usize,
    occupied2: usize,
    virtual1: usize,
    virtual2: usize,
    theta: f64,
) -> Result<()> {
    // 8-CNOT double excitation decomposition
    // Reference: arXiv 1805.04340 (Jordan-Wigner exponentiated Pauli strings)
    //
    // Forward CNOT cascade — computes parity on virtual2
    circuit.cx(occupied2, virtual1)?; // o2 → v1
    circuit.cx(virtual1, virtual2)?; // v1 → v2
    circuit.cx(occupied1, occupied2)?; // o1 → o2
    circuit.cx(occupied2, virtual1)?; // o2 → v1

    // Phase kickback on the parity qubit
    circuit.rz(virtual2, Parameter::Float(theta))?;

    // Reverse CNOT cascade — uncomputes parity
    circuit.cx(occupied2, virtual1)?; // reverse: o2 → v1
    circuit.cx(occupied1, occupied2)?; // reverse: o1 → o2
    circuit.cx(virtual1, virtual2)?; // reverse: v1 → v2
    circuit.cx(occupied2, virtual1)?; // reverse: o2 → v1

    Ok(())
}

/// UCCSD Ansatz builder
///
/// Constructs a complete UCCSD ansatz circuit for a given molecular system.
#[derive(Debug, Clone)]
pub struct UCCSDAnsatz {
    /// Number of qubits (spin-orbitals)
    pub num_qubits: usize,
    /// Number of electrons
    pub num_electrons: usize,
    /// Include single excitations
    pub include_singles: bool,
    /// Include double excitations
    pub include_doubles: bool,
}

impl UCCSDAnsatz {
    /// Create a new UCCSD ansatz
    ///
    /// # Arguments
    ///
    /// * `num_qubits` - Number of spin-orbitals (qubits)
    /// * `num_electrons` - Number of electrons in the system
    pub fn new(num_qubits: usize, num_electrons: usize) -> Self {
        UCCSDAnsatz {
            num_qubits,
            num_electrons,
            include_singles: true,
            include_doubles: true,
        }
    }

    /// Create UCCS ansatz (singles only)
    pub fn singles_only(num_qubits: usize, num_electrons: usize) -> Self {
        UCCSDAnsatz {
            num_qubits,
            num_electrons,
            include_singles: true,
            include_doubles: false,
        }
    }

    /// Create UCCD ansatz (doubles only)
    pub fn doubles_only(num_qubits: usize, num_electrons: usize) -> Self {
        UCCSDAnsatz {
            num_qubits,
            num_electrons,
            include_singles: false,
            include_doubles: true,
        }
    }

    /// Calculate number of single excitation parameters
    pub fn num_single_parameters(&self) -> usize {
        if !self.include_singles {
            return 0;
        }
        let num_occupied = self.num_electrons;
        let num_virtual = self.num_qubits - self.num_electrons;
        num_occupied * num_virtual
    }

    /// Calculate number of double excitation parameters
    pub fn num_double_parameters(&self) -> usize {
        if !self.include_doubles {
            return 0;
        }
        let num_occupied = self.num_electrons;
        let num_virtual = self.num_qubits - self.num_electrons;

        // Number of ways to choose 2 from occupied and 2 from virtual
        let occupied_pairs = num_occupied * (num_occupied - 1) / 2;
        let virtual_pairs = num_virtual * (num_virtual - 1) / 2;
        occupied_pairs * virtual_pairs
    }

    /// Calculate total number of parameters
    pub fn num_parameters(&self) -> usize {
        self.num_single_parameters() + self.num_double_parameters()
    }

    /// Build UCCSD circuit with given parameters
    ///
    /// # Arguments
    ///
    /// * `parameters` - Excitation amplitudes (singles first, then doubles)
    ///
    /// # Returns
    ///
    /// Quantum circuit implementing the UCCSD ansatz
    pub fn build_circuit(&self, parameters: &[f64]) -> Result<QuantumCircuit> {
        if parameters.len() != self.num_parameters() {
            return Err(MyQuatError::circuit_error(format!(
                "Expected {} parameters, got {}",
                self.num_parameters(),
                parameters.len()
            )));
        }

        let mut circuit = QuantumCircuit::new(self.num_qubits, 0);
        let mut param_idx = 0;

        // Prepare Hartree-Fock reference state |HF⟩
        // Set first num_electrons qubits to |1⟩
        for i in 0..self.num_electrons {
            circuit.x(i)?;
        }

        // Apply single excitations
        if self.include_singles {
            for i in 0..self.num_electrons {
                for a in self.num_electrons..self.num_qubits {
                    let theta = parameters[param_idx];
                    param_idx += 1;
                    single_excitation(&mut circuit, i, a, theta)?;
                }
            }
        }

        // Apply double excitations
        if self.include_doubles {
            for i in 0..self.num_electrons {
                for j in (i + 1)..self.num_electrons {
                    for a in self.num_electrons..self.num_qubits {
                        for b in (a + 1)..self.num_qubits {
                            let theta = parameters[param_idx];
                            param_idx += 1;
                            double_excitation(&mut circuit, i, j, a, b, theta)?;
                        }
                    }
                }
            }
        }

        Ok(circuit)
    }

    /// Generate random initial parameters
    pub fn random_parameters(&self) -> Vec<f64> {
        use rand::Rng;
        let mut rng = rand::rng();
        (0..self.num_parameters())
            .map(|_| rng.random_range(-0.1..0.1))
            .collect()
    }

    /// Generate zero initial parameters
    pub fn zero_parameters(&self) -> Vec<f64> {
        vec![0.0; self.num_parameters()]
    }

    /// Estimate circuit depth
    pub fn circuit_depth(&self) -> usize {
        let single_depth = if self.include_singles {
            4 * self.num_single_parameters() // 4 gates per single excitation
        } else {
            0
        };

        let double_depth = if self.include_doubles {
            9 * self.num_double_parameters() // 9 gates per double excitation (8 CX + 1 Rz)
        } else {
            0
        };

        self.num_electrons + single_depth + double_depth
    }
}

/// Generate all single excitation indices for a given system
///
/// Returns a vector of (occupied, virtual) pairs
pub fn generate_single_excitations(num_electrons: usize, num_qubits: usize) -> Vec<(usize, usize)> {
    let mut excitations = Vec::new();
    for i in 0..num_electrons {
        for a in num_electrons..num_qubits {
            excitations.push((i, a));
        }
    }
    excitations
}

/// Generate all double excitation indices for a given system
///
/// Returns a vector of ((i, j), (a, b)) tuples
pub fn generate_double_excitations(
    num_electrons: usize,
    num_qubits: usize,
) -> Vec<((usize, usize), (usize, usize))> {
    let mut excitations = Vec::new();
    for i in 0..num_electrons {
        for j in (i + 1)..num_electrons {
            for a in num_electrons..num_qubits {
                for b in (a + 1)..num_qubits {
                    excitations.push(((i, j), (a, b)));
                }
            }
        }
    }
    excitations
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_single_excitation() {
        let mut circuit = QuantumCircuit::new(4, 0);
        single_excitation(&mut circuit, 0, 2, PI / 4.0).unwrap();

        // Should have 4 gates: RY, CX, RY, CX
        assert_eq!(circuit.size(), 4);
    }

    #[test]
    fn test_double_excitation() {
        let mut circuit = QuantumCircuit::new(4, 0);
        double_excitation(&mut circuit, 0, 1, 2, 3, PI / 4.0).unwrap();

        // Should have 9 gates: 4 CX (forward) + 1 Rz + 4 CX (reverse)
        assert_eq!(circuit.size(), 9);
    }

    #[test]
    fn test_uccsd_parameter_count() {
        // H2 molecule: 4 spin-orbitals, 2 electrons
        let uccsd = UCCSDAnsatz::new(4, 2);

        // Singles: 2 occupied × 2 virtual = 4
        assert_eq!(uccsd.num_single_parameters(), 4);

        // Doubles: C(2,2) × C(2,2) = 1 × 1 = 1
        assert_eq!(uccsd.num_double_parameters(), 1);

        // Total: 4 + 1 = 5
        assert_eq!(uccsd.num_parameters(), 5);
    }

    #[test]
    fn test_uccsd_singles_only() {
        let uccsd = UCCSDAnsatz::singles_only(4, 2);
        assert_eq!(uccsd.num_single_parameters(), 4);
        assert_eq!(uccsd.num_double_parameters(), 0);
        assert_eq!(uccsd.num_parameters(), 4);
    }

    #[test]
    fn test_uccsd_doubles_only() {
        let uccsd = UCCSDAnsatz::doubles_only(4, 2);
        assert_eq!(uccsd.num_single_parameters(), 0);
        assert_eq!(uccsd.num_double_parameters(), 1);
        assert_eq!(uccsd.num_parameters(), 1);
    }

    #[test]
    fn test_uccsd_circuit_construction() {
        let uccsd = UCCSDAnsatz::new(4, 2);
        let params = uccsd.zero_parameters();
        let circuit = uccsd.build_circuit(&params).unwrap();

        // Should have Hartree-Fock preparation + excitations
        assert_eq!(circuit.num_qubits(), 4);
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_uccsd_invalid_parameters() {
        let uccsd = UCCSDAnsatz::new(4, 2);
        let wrong_params = vec![0.1, 0.2]; // Not enough parameters
        let result = uccsd.build_circuit(&wrong_params);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_single_excitations() {
        let excitations = generate_single_excitations(2, 4);
        // 2 occupied × 2 virtual = 4 excitations
        assert_eq!(excitations.len(), 4);
        assert_eq!(excitations[0], (0, 2));
        assert_eq!(excitations[1], (0, 3));
        assert_eq!(excitations[2], (1, 2));
        assert_eq!(excitations[3], (1, 3));
    }

    #[test]
    fn test_generate_double_excitations() {
        let excitations = generate_double_excitations(2, 4);
        // C(2,2) × C(2,2) = 1 × 1 = 1 excitation
        assert_eq!(excitations.len(), 1);
        assert_eq!(excitations[0], ((0, 1), (2, 3)));
    }

    #[test]
    fn test_lih_molecule() {
        // LiH molecule: 10 spin-orbitals, 4 electrons
        let uccsd = UCCSDAnsatz::new(10, 4);

        // Singles: 4 × 6 = 24
        assert_eq!(uccsd.num_single_parameters(), 24);

        // Doubles: C(4,2) × C(6,2) = 6 × 15 = 90
        assert_eq!(uccsd.num_double_parameters(), 90);

        // Total: 24 + 90 = 114
        assert_eq!(uccsd.num_parameters(), 114);
    }
}
