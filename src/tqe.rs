//! TQE (Two-Qubit Entangled Clifford) synthesis.
//!
//! Phase 11x: Implements TKET's GreedyPauliSimp TQE optimization —
//! 2-qubit Clifford gates (XX, XY, XZ, YX, YY, YZ, ZX, ZY, ZZ) used
//! to reduce Pauli string weight. Each TQE gate is a specific Clifford
//! operation that can eliminate one non-identity position in a Pauli
//! string, enabling more efficient CNOT ladder synthesis.
//!
//! # Reference
//! - TKET `Transformations/GreedyPauliOptimisation.cpp`
//! - Cowtan et al., "Phase Gadget Synthesis for Shallow Circuits" (EPTCS 318, 2020)

use crate::gates::StandardGate;
use crate::hamiltonian::pauli_string::{PauliOperator, PauliString};

/// The 9 types of two-qubit entangled Clifford (TQE) gates.
///
/// Each TQE type is defined by the Pauli basis on each qubit:
/// - First letter: Pauli on qubit `a`
/// - Second letter: Pauli on qubit `b`
///
/// The gate is implemented as: basis_change(a) · basis_change(b) · ZZPhase · un_basis_change(b) · un_basis_change(a)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TQEType {
    XX = 0,
    XY,
    XZ,
    YX,
    YY,
    YZ,
    ZX,
    ZY,
    ZZ,
}

impl TQEType {
    /// All 9 TQE types.
    pub const ALL: [TQEType; 9] = [
        TQEType::XX,
        TQEType::XY,
        TQEType::XZ,
        TQEType::YX,
        TQEType::YY,
        TQEType::YZ,
        TQEType::ZX,
        TQEType::ZY,
        TQEType::ZZ,
    ];

    /// Convert a TQE gate into its Clifford circuit decomposition.
    ///
    /// Each TQE is equivalent to: basis(a) · basis(b) · ZZPhase(π/2) · basis†(b) · basis†(a)
    /// where the basis change is H for X, V (Rx(π/2)) for Y, identity for Z.
    ///
    /// Returns a list of (gate, qubits) pairs implementing the TQE.
    pub fn to_gates(self, a: usize, b: usize) -> Vec<(StandardGate, Vec<usize>)> {
        match self {
            // XX: H(a)·CX(a,b)·H(a)  (equivalent to CZ when acting on Z⊗Z)
            TQEType::XX => vec![
                (StandardGate::H, vec![a]),
                (StandardGate::CX, vec![a, b]),
                (StandardGate::H, vec![a]),
            ],
            // XY: H(a)·CY(a,b)·H(a)
            TQEType::XY => {
                // CY = Sdg(b)·CX(a,b)·S(b)
                vec![
                    (StandardGate::H, vec![a]),
                    (StandardGate::Sdg, vec![b]),
                    (StandardGate::CX, vec![a, b]),
                    (StandardGate::S, vec![b]),
                    (StandardGate::H, vec![a]),
                ]
            }
            // XZ: CX(b,a)
            TQEType::XZ => vec![(StandardGate::CX, vec![b, a])],
            // YX: H(b)·CY(b,a)·H(b)
            TQEType::YX => vec![
                (StandardGate::H, vec![b]),
                (StandardGate::Sdg, vec![a]),
                (StandardGate::CX, vec![b, a]),
                (StandardGate::S, vec![a]),
                (StandardGate::H, vec![b]),
            ],
            // YY: V(a)·CY(a,b)·Vdg(a)  where V = Rx(π/2)
            TQEType::YY => vec![
                (StandardGate::Rx, vec![a]), // V (approximated — full impl uses Rx(π/2))
                (StandardGate::Sdg, vec![b]),
                (StandardGate::CX, vec![a, b]),
                (StandardGate::S, vec![b]),
                (StandardGate::Rx, vec![a]), // Vdg
            ],
            // YZ: CY(b,a)
            TQEType::YZ => vec![
                (StandardGate::Sdg, vec![a]),
                (StandardGate::CX, vec![b, a]),
                (StandardGate::S, vec![a]),
            ],
            // ZX: CX(a,b)
            TQEType::ZX => vec![(StandardGate::CX, vec![a, b])],
            // ZY: CY(a,b)
            TQEType::ZY => vec![
                (StandardGate::Sdg, vec![b]),
                (StandardGate::CX, vec![a, b]),
                (StandardGate::S, vec![b]),
            ],
            // ZZ: CZ(a,b)
            TQEType::ZZ => vec![
                (StandardGate::H, vec![b]),
                (StandardGate::CX, vec![a, b]),
                (StandardGate::H, vec![b]),
            ],
        }
    }
}

/// Compute how a TQE gate transforms a pair of Pauli operators.
///
/// Given TQE gate `g` acting on qubits (a, b) with Pauli operators (p_a, p_b),
/// returns the transformed operators (p_a', p_b') and a sign flag.
///
/// This is the core lookup table from TKET's TQE_PAULI_MAP (144 entries).
pub fn tqe_conjugate_pauli_pair(
    g: TQEType,
    p_a: PauliOperator,
    p_b: PauliOperator,
) -> (PauliOperator, PauliOperator, bool) {
    use PauliOperator::*;
    use TQEType::*;

    match (g, p_a, p_b) {
        // ── XX ──
        (XX, X, X) => (X, X, true),
        (XX, X, Y) => (I, Y, true),
        (XX, X, Z) => (I, Z, true),
        (XX, X, I) => (X, I, true),
        (XX, Y, X) => (Y, I, true),
        (XX, Y, Y) => (Z, Z, true),
        (XX, Y, Z) => (Z, Y, false),
        (XX, Y, I) => (Y, X, true),
        (XX, Z, X) => (Z, I, true),
        (XX, Z, Y) => (Y, Z, false),
        (XX, Z, Z) => (Y, Y, true),
        (XX, Z, I) => (Z, X, true),
        (XX, I, X) => (I, X, true),
        (XX, I, Y) => (X, Y, true),
        (XX, I, Z) => (X, Z, true),
        (XX, I, I) => (I, I, true),

        // ── XY ──
        (XY, X, X) => (I, X, true),
        (XY, X, Y) => (X, Y, true),
        (XY, X, Z) => (I, Z, true),
        (XY, X, I) => (X, I, true),
        (XY, Y, X) => (Z, Z, false),
        (XY, Y, Y) => (Y, I, true),
        (XY, Y, Z) => (Z, X, true),
        (XY, Y, I) => (Y, Y, true),
        (XY, Z, X) => (Y, Z, true),
        (XY, Z, Y) => (Z, I, true),
        (XY, Z, Z) => (Y, X, false),
        (XY, Z, I) => (Z, Y, true),
        (XY, I, X) => (X, X, true),
        (XY, I, Y) => (I, Y, true),
        (XY, I, Z) => (X, Z, true),
        (XY, I, I) => (I, I, true),

        // ── XZ ──
        (XZ, X, X) => (I, X, true),
        (XZ, X, Y) => (I, Y, true),
        (XZ, X, Z) => (X, Z, true),
        (XZ, X, I) => (X, I, true),
        (XZ, Y, X) => (Z, Y, true),
        (XZ, Y, Y) => (Z, X, false),
        (XZ, Y, Z) => (Y, I, true),
        (XZ, Y, I) => (Y, Z, true),
        (XZ, Z, X) => (Y, Y, false),
        (XZ, Z, Y) => (Y, X, true),
        (XZ, Z, Z) => (Z, I, true),
        (XZ, Z, I) => (Z, Z, true),
        (XZ, I, X) => (X, X, true),
        (XZ, I, Y) => (X, Y, true),
        (XZ, I, Z) => (I, Z, true),
        (XZ, I, I) => (I, I, true),

        // ── YX ──
        (YX, X, X) => (X, I, true),
        (YX, X, Y) => (Z, Z, false),
        (YX, X, Z) => (Z, Y, true),
        (YX, X, I) => (X, X, true),
        (YX, Y, X) => (Y, X, true),
        (YX, Y, Y) => (I, Y, true),
        (YX, Y, Z) => (I, Z, true),
        (YX, Y, I) => (Y, I, true),
        (YX, Z, X) => (Z, I, true),
        (YX, Z, Y) => (X, Z, false),
        (YX, Z, Z) => (X, Y, true),
        (YX, Z, I) => (Z, I, true),
        (YX, I, X) => (I, X, true),
        (YX, I, Y) => (Y, Y, true),
        (YX, I, Z) => (Y, Z, true),
        (YX, I, I) => (I, I, true),

        // ── YY ──
        (YY, X, X) => (Z, Z, true),
        (YY, X, Y) => (X, I, true),
        (YY, X, Z) => (Z, X, false),
        (YY, X, I) => (X, Y, true),
        (YY, Y, X) => (I, X, true),
        (YY, Y, Y) => (Y, Y, true),
        (YY, Y, Z) => (I, Z, true),
        (YY, Y, I) => (Y, I, true),
        (YY, Z, X) => (X, Z, false),
        (YY, Z, Y) => (Y, I, true),
        (YY, Z, Z) => (X, X, true),
        (YY, Z, I) => (Z, Y, true),
        (YY, I, X) => (Y, X, true),
        (YY, I, Y) => (I, Y, true),
        (YY, I, Z) => (Y, Z, true),
        (YY, I, I) => (I, I, true),

        // ── YZ ──
        (YZ, X, X) => (Z, Y, false),
        (YZ, X, Y) => (Z, X, true),
        (YZ, X, Z) => (X, I, true),
        (YZ, X, I) => (X, Z, true),
        (YZ, Y, X) => (I, X, true),
        (YZ, Y, Y) => (I, Y, true),
        (YZ, Y, Z) => (Y, Z, true),
        (YZ, Y, I) => (Y, I, true),
        (YZ, Z, X) => (X, Y, true),
        (YZ, Z, Y) => (X, X, false),
        (YZ, Z, Z) => (Z, I, true),
        (YZ, Z, I) => (Z, Z, true),
        (YZ, I, X) => (Y, X, true),
        (YZ, I, Y) => (Y, Y, true),
        (YZ, I, Z) => (I, Z, true),
        (YZ, I, I) => (I, I, true),

        // ── ZX ──
        (ZX, X, X) => (X, I, true),
        (ZX, X, Y) => (Y, Z, true),
        (ZX, X, Z) => (Y, Y, false),
        (ZX, X, I) => (X, X, true),
        (ZX, Y, X) => (Y, I, true),
        (ZX, Y, Y) => (X, Z, false),
        (ZX, Y, Z) => (X, Y, true),
        (ZX, Y, I) => (Y, X, true),
        (ZX, Z, X) => (Z, X, true),
        (ZX, Z, Y) => (I, Y, true),
        (ZX, Z, Z) => (I, Z, true),
        (ZX, Z, I) => (Z, I, true),
        (ZX, I, X) => (I, X, true),
        (ZX, I, Y) => (Z, Y, true),
        (ZX, I, Z) => (Z, Z, true),
        (ZX, I, I) => (I, I, true),

        // ── ZY ──
        (ZY, X, X) => (Y, Z, false),
        (ZY, X, Y) => (X, I, true),
        (ZY, X, Z) => (Y, X, true),
        (ZY, X, I) => (X, Y, true),
        (ZY, Y, X) => (X, Z, true),
        (ZY, Y, Y) => (Y, I, true),
        (ZY, Y, Z) => (X, X, false),
        (ZY, Y, I) => (Y, Y, true),
        (ZY, Z, X) => (I, X, true),
        (ZY, Z, Y) => (Z, Y, true),
        (ZY, Z, Z) => (I, Z, true),
        (ZY, Z, I) => (Z, I, true),
        (ZY, I, X) => (Z, X, true),
        (ZY, I, Y) => (I, Y, true),
        (ZY, I, Z) => (Z, Z, true),
        (ZY, I, I) => (I, I, true),

        // ── ZZ ──
        (ZZ, X, X) => (Y, Y, true),
        (ZZ, X, Y) => (Y, X, false),
        (ZZ, X, Z) => (X, I, true),
        (ZZ, X, I) => (X, Z, true),
        (ZZ, Y, X) => (X, Y, false),
        (ZZ, Y, Y) => (X, X, true),
        (ZZ, Y, Z) => (Y, I, true),
        (ZZ, Y, I) => (Y, Z, true),
        (ZZ, Z, X) => (I, X, true),
        (ZZ, Z, Y) => (I, Y, true),
        (ZZ, Z, Z) => (Z, Z, true),
        (ZZ, Z, I) => (Z, I, true),
        (ZZ, I, X) => (Z, X, true),
        (ZZ, I, Y) => (Z, Y, true),
        (ZZ, I, Z) => (I, Z, true),
        (ZZ, I, I) => (I, I, true),
    }
}

/// Check if a TQE gate commutes with a pair of Pauli operators.
pub fn tqe_commutes(g: TQEType, p_a: PauliOperator, p_b: PauliOperator) -> bool {
    let (new_a, new_b, sign) = tqe_conjugate_pauli_pair(g, p_a, p_b);
    // TQE commutes if the Pauli operators are unchanged and sign is true
    p_a == new_a && p_b == new_b && sign
}

/// Cost increase of applying a TQE to a pair of Pauli operators.
///
/// Cost = (#non-I after) - (#non-I before). Negative means the TQE reduces weight.
pub fn tqe_cost_increase(g: TQEType, p_a: PauliOperator, p_b: PauliOperator) -> i32 {
    let old_non_i = (p_a != PauliOperator::I) as i32 + (p_b != PauliOperator::I) as i32;
    let (new_a, new_b, _sign) = tqe_conjugate_pauli_pair(g, p_a, p_b);
    let new_non_i = (new_a != PauliOperator::I) as i32 + (new_b != PauliOperator::I) as i32;
    new_non_i - old_non_i
}

/// Return all TQE gates that reduce a pair of non-identity Pauli operators.
///
/// For a pair (P, Q) where both are non-identity, returns the TQEs that can
/// make at least one of them identity.
pub fn tqe_reduction_types(p_a: PauliOperator, p_b: PauliOperator) -> Vec<TQEType> {
    use PauliOperator::*;
    use TQEType::*;

    match (p_a, p_b) {
        (X, X) => vec![XY, XZ, YX, ZX],
        (X, Y) => vec![XX, XZ, YY, ZY],
        (X, Z) => vec![XX, XY, YZ, ZZ],
        (Y, X) => vec![XX, YY, YZ, ZX],
        (Y, Y) => vec![XY, YX, YZ, ZY],
        (Y, Z) => vec![XZ, YX, YY, ZZ],
        (Z, X) => vec![XX, YX, ZY, ZZ],
        (Z, Y) => vec![XY, YY, ZX, ZZ],
        (Z, Z) => vec![XZ, YZ, ZX, ZY],
        _ => vec![], // At least one I — no reduction possible
    }
}

/// Find all TQE gates that reduce the weight of a Pauli string.
///
/// Returns a list of (TQEType, qubit_a, qubit_b) pairs that each reduce
/// the string's weight by 1 when applied.
pub fn reduction_tqes(pauli: &PauliString) -> Vec<(TQEType, usize, usize)> {
    // Collect qubits with non-identity support
    let support: Vec<usize> = (0..pauli.operators.len())
        .filter(|&i| pauli.operators[i] != PauliOperator::I)
        .collect();

    let mut tqes = Vec::new();
    for i in 0..support.len() {
        for j in (i + 1)..support.len() {
            let a = support[i];
            let b = support[j];
            let types = tqe_reduction_types(pauli.operators[a], pauli.operators[b]);
            for tt in types {
                tqes.push((tt, a, b));
            }
        }
    }
    tqes
}

/// Apply a TQE gate to a Pauli string (in-place update).
///
/// Updates the Pauli operators at positions `a` and `b` according to the
/// TQE conjugation rules.
pub fn apply_tqe_to_pauli(pauli: &mut PauliString, g: TQEType, a: usize, b: usize) {
    let (new_a, new_b, sign) = tqe_conjugate_pauli_pair(g, pauli.operators[a], pauli.operators[b]);
    pauli.operators[a] = new_a;
    pauli.operators[b] = new_b;
    // Phase adjustments: if sign is false, multiply coefficient by -1
    // (simplified — full implementation tracks phase more carefully)
    if !sign {
        pauli.coefficient *= num_complex::Complex64::new(-1.0, 0.0);
    }
}

/// Compute the TQE cost of a Pauli string = weight - 1.
///
/// This is the number of TQE gates needed to reduce the string to weight 1.
pub fn tqe_cost(pauli: &PauliString) -> usize {
    let weight: usize = pauli
        .operators
        .iter()
        .filter(|&&op| op != PauliOperator::I)
        .count();
    if weight == 0 {
        0
    } else {
        weight - 1
    }
}

/// Greedily reduce a Pauli string to weight 1 using TQE gates.
///
/// Returns the sequence of (TQEType, a, b) gates that reduces the Pauli
/// string to weight 1. Each gate reduces weight by exactly 1.
pub fn greedy_reduce_pauli(pauli: &PauliString) -> Vec<(TQEType, usize, usize)> {
    let mut current = pauli.clone();
    let mut gates = Vec::new();

    loop {
        let support: Vec<usize> = (0..current.operators.len())
            .filter(|&i| current.operators[i] != PauliOperator::I)
            .collect();

        if support.len() <= 1 {
            break;
        }

        // Find a TQE that reduces weight by 1
        let mut found = None;
        for i in 0..support.len() {
            for j in (i + 1)..support.len() {
                let a = support[i];
                let b = support[j];
                let types = tqe_reduction_types(current.operators[a], current.operators[b]);
                if !types.is_empty() {
                    found = Some((types[0], a, b));
                    break;
                }
            }
            if found.is_some() {
                break;
            }
        }

        match found {
            Some((tt, a, b)) => {
                apply_tqe_to_pauli(&mut current, tt, a, b);
                gates.push((tt, a, b));
            }
            None => break, // No reduction possible
        }
    }

    gates
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_complex::Complex64;

    fn make_pauli(ops: Vec<PauliOperator>) -> PauliString {
        PauliString::new(ops, Complex64::new(1.0, 0.0))
    }

    #[test]
    fn test_tqe_types_all_distinct() {
        // Verify all 9 types are distinct
        let mut seen = std::collections::HashSet::new();
        for t in &TQEType::ALL {
            assert!(seen.insert(*t as u8));
        }
        assert_eq!(seen.len(), 9);
    }

    #[test]
    fn test_tqe_to_gates_xx() {
        let gates = TQEType::XX.to_gates(0, 1);
        // XX = H(0)·CX(0,1)·H(0)
        assert_eq!(gates.len(), 3);
        assert_eq!(gates[0], (StandardGate::H, vec![0]));
        assert_eq!(gates[1], (StandardGate::CX, vec![0, 1]));
        assert_eq!(gates[2], (StandardGate::H, vec![0]));
    }

    #[test]
    fn test_tqe_to_gates_xz() {
        let gates = TQEType::XZ.to_gates(0, 1);
        // XZ = CX(1,0)
        assert_eq!(gates[0], (StandardGate::CX, vec![1, 0]));
    }

    #[test]
    fn test_tqe_to_gates_zx() {
        let gates = TQEType::ZX.to_gates(0, 1);
        // ZX = CX(0,1)
        assert_eq!(gates[0], (StandardGate::CX, vec![0, 1]));
    }

    #[test]
    fn test_tqe_conjugate_known() {
        use PauliOperator::*;
        // XZ: CX(b,a) = CX(1,0). Conjugating X on a=0 and X on b=1:
        // X(0),X(1) → I(0),X(1) (X on control doesn't change, X on target stays X)
        // Actually from the table: (XZ, X, X) → (I, X, true) — weight reduction!
        let (na, nb, sign) = tqe_conjugate_pauli_pair(TQEType::XZ, X, X);
        assert_eq!(na, I);
        assert_eq!(nb, X);
        assert!(sign);
    }

    #[test]
    fn test_tqe_cost_increase_negative() {
        use PauliOperator::*;
        // XZ transforms (X,X) → (I,X), reducing weight by 1
        let cost = tqe_cost_increase(TQEType::XZ, X, X);
        assert_eq!(cost, -1);
    }

    #[test]
    fn test_tqe_commutes_identity() {
        use PauliOperator::*;
        // All TQEs commute with (I,I)
        for t in &TQEType::ALL {
            assert!(tqe_commutes(*t, I, I));
        }
    }

    #[test]
    fn test_reduction_tqes_simple() {
        use PauliOperator::*;
        // Pauli string X⊗X⊗I on 3 qubits
        let p = make_pauli(vec![X, X, I]);
        let tqes = reduction_tqes(&p);
        // (X,X) pair should give {XY, XZ, YX, ZX}
        assert!(!tqes.is_empty());
        // All should target qubits 0 and 1
        for (_, a, b) in &tqes {
            assert_eq!(*a, 0);
            assert_eq!(*b, 1);
        }
    }

    #[test]
    fn test_reduction_types_all_non_identity_pairs() {
        use PauliOperator::*;
        let non_i = [X, Y, Z];
        for &p1 in &non_i {
            for &p2 in &non_i {
                let types = tqe_reduction_types(p1, p2);
                assert!(
                    !types.is_empty(),
                    "No reduction types for ({:?}, {:?})",
                    p1,
                    p2
                );
                // Each type should actually reduce weight
                for &tt in &types {
                    let cost = tqe_cost_increase(tt, p1, p2);
                    assert_eq!(
                        cost, -1,
                        "TQE {:?} does not reduce ({:?}, {:?})",
                        tt, p1, p2
                    );
                }
            }
        }
    }

    #[test]
    fn test_apply_tqe_reduces_weight() {
        use PauliOperator::*;
        let mut p = make_pauli(vec![X, X, I]);
        let old_weight: usize = p.operators.iter().filter(|&&o| o != I).count();
        apply_tqe_to_pauli(&mut p, TQEType::XZ, 0, 1);
        let new_weight: usize = p.operators.iter().filter(|&&o| o != I).count();
        assert_eq!(new_weight, old_weight - 1);
        assert_eq!(p.operators[0], I);
        assert_eq!(p.operators[1], X);
    }

    #[test]
    fn test_tqe_cost_formula() {
        use PauliOperator::*;
        let p = make_pauli(vec![X, Y, Z, I]); // weight 3
        assert_eq!(tqe_cost(&p), 2); // weight - 1 = 2
        let p2 = make_pauli(vec![X, I, I, I]); // weight 1
        assert_eq!(tqe_cost(&p2), 0);
        let p3 = make_pauli(vec![I, I, I, I]); // weight 0
        assert_eq!(tqe_cost(&p3), 0);
    }

    #[test]
    fn test_greedy_reduce_pauli() {
        use PauliOperator::*;
        // Weight 4 Pauli string — needs 3 TQEs to reduce to weight 1
        let p = make_pauli(vec![X, Y, Z, X]);
        let gates = greedy_reduce_pauli(&p);
        assert_eq!(
            gates.len(),
            3,
            "Expected 3 TQE gates, got {}: {:?}",
            gates.len(),
            gates
        );

        // Apply all TQEs and verify weight ≤ 1
        let mut reduced = p.clone();
        for (tt, a, b) in &gates {
            apply_tqe_to_pauli(&mut reduced, *tt, *a, *b);
        }
        let weight: usize = reduced.operators.iter().filter(|&&o| o != I).count();
        assert!(
            weight <= 1,
            "Final weight should be ≤ 1, got {}: {:?}",
            weight,
            reduced.operators
        );
    }

    #[test]
    fn test_greedy_reduce_empty_trivial() {
        use PauliOperator::*;
        // Weight 1 — needs 0 TQEs
        let p = make_pauli(vec![I, X, I, I]);
        let gates = greedy_reduce_pauli(&p);
        assert!(gates.is_empty());
    }
}
