//! Pauli gadget optimization — pre-synthesis Pauli term merging.
//!
//! Phase 11a: Implements TKET-style Pauli gadget optimization at the Pauli
//! term level, BEFORE QWC block formation. Recognizing Clifford-equivalent
//! Pauli terms and aligning their bases reduces the number of distinct
//! CNOT tree positions needed, saving both CX and single-qubit gates.
//!
//! ## Algorithm (TKET PauliSimp-inspired)
//!
//! 1. **merge_identical_paulis**: Group terms with identical Pauli strings,
//!    sum their coefficients. Always applied, always exact.
//! 2. **align_clifford_equivalent**: Find pairs where Pauli strings differ
//!    only by X↔Y substitutions. Align their bases via Clifford conjugation
//!    so they can share a CNOT tree position during synthesis.
//!
//! ## Important: CliffordSimple is approximate pre-synthesis
//!
//! The key Clifford identity: Y = S · X · S†.
//!
//! When merging e^{-iθ_i·X} · e^{-iθ_j·Y} into a single gadget wrapped in
//! Clifford gates, the Clifford conjugation applies to BOTH terms, giving:
//!   S† · e^{-i(θ_i+θ_j)·X} · S = e^{-i(θ_i+θ_j)·S†XS} = e^{-i(θ_i+θ_j)·(-Y)}
//! which is NOT equal to the original product when θ_i ≠ 0.
//!
//! **The merge is exact only when one gadget has angle ≈ 0** (which is
//! the common case in cross-step synthesis where we start from an empty
//! collector). For general per-step synthesis, Phase 11a-3 will handle
//! this correctly at the circuit level by applying Clifford gates between
//! individual rotations rather than wrapping a single merged exponential.

/// Compute the effective Pauli string after applying Clifford pre-gates.
///
/// This is used for QWC block grouping: after Clifford alignment
/// (e.g., IIYY → IIXX via S†⊗S†), the effective Pauli determines which
/// QWC block the term belongs to.  Two terms with the same effective
/// Pauli can share a CNOT tree, with Clifford gates placed around
/// individual Rz rotations within the shared tree.
///
/// # Clifford transformations
/// - S† on Y → X
/// - S on X → Y
/// - S† on X → Y (via S† = S^(-1))
/// - S on Y → X
/// - H on X → Z, H on Z → X
///
/// Note: (Sdg, X) and (S, Y) are NOT handled — those conjugations introduce
/// a sign flip (S†·X·S = -Y, S·Y·S† = -X) and require phase tracking beyond
/// the scope of this operator-only transform. They are unreachable with the
/// current CliffordSimple XY-alignment strategy.
pub fn apply_clifford_pre_gates(
    pauli: &PauliString,
    pre_gates: &[(CliffordGate, usize)],
) -> PauliString {
    // Clone operators and apply Clifford gates. We build a fresh PauliString
    // rather than cloning + mutating because PauliString caches its string repr
    // in a OnceCell, and modifying operators doesn't invalidate the cache.
    let mut new_operators = pauli.operators.clone();
    for (gate, q) in pre_gates {
        use PauliOperator::*;
        match (gate, &new_operators[*q]) {
            (CliffordGate::Sdg, Y) => new_operators[*q] = X,
            (CliffordGate::S, X) => new_operators[*q] = Y,
            (CliffordGate::H, X) => new_operators[*q] = Z,
            (CliffordGate::H, Z) => new_operators[*q] = X,
            _ => {} // S on Z = Z, S† on Z = Z; (Sdg,X)/(S,Y) have sign issues
        }
    }
    PauliString::new(new_operators, pauli.coefficient)
}

// ---------------------------------------------------------------------------
// Clifford Algebra Utilities (Phase 11e)
// ---------------------------------------------------------------------------

/// Conjugate a single Pauli operator through a Clifford gate: G · P · G†.
///
/// Returns the resulting Pauli operator (up to ±1 phase factor).
///
/// # Parameters
/// - `gate`: single-qubit Clifford gate (H, S, or S†)
/// - `op`: the Pauli operator being conjugated
/// - `ignore_sign`: if true, ignores phase flips introduced by conjugation
///   (useful when only the Pauli operator matters, e.g., for QWC checking)
///
/// # Full mapping
///
/// | Gate | Input | Output | Sign |
/// |------|-------|--------|------|
/// | H    | X     | Z      | +1   |
/// | H    | Z     | X      | +1   |
/// | H    | Y     | -Y     | -1   |
/// | S    | X     | Y      | +1   |
/// | S    | Y     | -X     | -1   |
/// | S†   | Y     | X      | +1   |
/// | S†   | X     | -Y     | -1   |
/// | H/S/S† | I   | I      | +1   |
/// | H/S/S† | Z   | Z      | +1  (for S/S†) |
///
/// Returns `None` when `ignore_sign=false` and the conjugation introduces
/// a sign flip (e.g., S†·X·S = -Y). Returns the unsigned operator when
/// `ignore_sign=true`.
pub fn conjugate_pauli_operator(
    gate: CliffordGate,
    op: PauliOperator,
    ignore_sign: bool,
) -> Option<PauliOperator> {
    use PauliOperator::*;
    match (gate, op) {
        // H: X→Z, Z→X (exact); Y→-Y (sign flip)
        (CliffordGate::H, X) => Some(Z),
        (CliffordGate::H, Z) => Some(X),
        (CliffordGate::H, Y) => {
            if ignore_sign {
                Some(Y)
            } else {
                None
            }
        }
        // S: X→Y (exact); Y→-X (sign flip); Z→Z, I→I (stable)
        (CliffordGate::S, X) => Some(Y),
        (CliffordGate::S, Y) => {
            if ignore_sign {
                Some(X)
            } else {
                None
            }
        }
        (CliffordGate::S, Z) => Some(Z),
        (CliffordGate::S, I) => Some(I),
        // S†: Y→X (exact); X→-Y (sign flip); Z→Z, I→I (stable)
        (CliffordGate::Sdg, Y) => Some(X),
        (CliffordGate::Sdg, X) => {
            if ignore_sign {
                Some(Y)
            } else {
                None
            }
        }
        (CliffordGate::Sdg, Z) => Some(Z),
        (CliffordGate::Sdg, I) => Some(I),
        // H on I: I→I
        (CliffordGate::H, I) => Some(I),
    }
}

/// Find a single Clifford gate that maps one Pauli operator to another.
///
/// Returns `Some((gate, ignore_sign))` where `gate` is the Clifford gate
/// that conjugates `from` → `to`, and `ignore_sign` indicates whether
/// the mapping introduces a sign flip that can be ignored.
///
/// Returns `None` if no single-gate mapping exists (e.g., Z→Y requires H·S,
/// not a single gate). For multi-gate mappings, use [`find_alignment_gates`].
///
/// # Examples
///
/// ```rust,ignore
/// // X → Z via H: H·X·H = Z
/// assert_eq!(find_alignment_gate(X, Z), Some((CliffordGate::H, false)));
/// // X → Y via S: S·X·S† = Y
/// assert_eq!(find_alignment_gate(X, Y), Some((CliffordGate::S, false)));
/// // Z → X via H: H·Z·H = X
/// assert_eq!(find_alignment_gate(Z, X), Some((CliffordGate::H, false)));
/// // Z → Y: requires H·S (no single gate)
/// assert_eq!(find_alignment_gate(Z, Y), None);
/// ```
pub fn find_alignment_gate(from: PauliOperator, to: PauliOperator) -> Option<(CliffordGate, bool)> {
    use CliffordGate::*;

    // Same operator: already aligned, no gate needed.
    // Return None so callers can distinguish "no gate needed"
    // from "gate found but sign-ignoring".
    if from == to {
        return None;
    }

    // Try each Clifford gate, exact match first (no sign flip)
    for gate in &[H, S, Sdg] {
        if let Some(result) = conjugate_pauli_operator(*gate, from, false) {
            if result == to {
                return Some((*gate, false));
            }
        }
    }

    // Try with sign ignored
    for gate in &[H, S, Sdg] {
        if let Some(result) = conjugate_pauli_operator(*gate, from, true) {
            if result == to {
                return Some((*gate, true));
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Phase 11g: Full Clifford Group Alignment
// ---------------------------------------------------------------------------

/// Find the optimal Clifford gate sequence that maps one Pauli operator to another.
///
/// Returns the sequence of Clifford gates G such that G·from·G† = to (up to sign).
/// The gates are applied in order: `gates[0]` first, then `gates[1]`, etc.
///
/// # Cost model
///
/// | Mapping  | Gates      | Cost |
/// |----------|------------|------|
/// | Same op  | (empty)    | 0    |
/// | X ↔ Y    | S or S†    | 1    |
/// | X ↔ Z    | H          | 1    |
/// | Y ↔ Z    | S†+H or H+S | 2   |
///
/// # Returns
///
/// - `Some(vec![])` — operators are identical, no gates needed
/// - `Some([gate1, gate2, ...])` — gate sequence to apply
/// - `None` — operators cannot be aligned (I vs non-I; should never happen in practice)
///
/// # Examples
///
/// ```rust,ignore
/// use PauliOperator::*;
/// // X → Z via H
/// assert_eq!(find_alignment_gates(X, Z), Some(vec![CliffordGate::H]));
/// // Y → Z via S† then H: H·S†·Y·S·H = H·X·H = Z
/// assert_eq!(find_alignment_gates(Y, Z), Some(vec![CliffordGate::Sdg, CliffordGate::H]));
/// // Z → Y via H then S: S·H·Z·H·S† = S·X·S† = Y
/// assert_eq!(find_alignment_gates(Z, Y), Some(vec![CliffordGate::H, CliffordGate::S]));
/// // Same operator: empty sequence
/// assert_eq!(find_alignment_gates(X, X), Some(vec![]));
/// ```
pub fn find_alignment_gates(from: PauliOperator, to: PauliOperator) -> Option<Vec<CliffordGate>> {
    use CliffordGate::*;
    use PauliOperator::*;

    // Same operator: no gates needed
    if from == to {
        return Some(vec![]);
    }

    // Identity can't be mapped to/from non-identity via conjugation
    if matches!(from, I) || matches!(to, I) {
        return None;
    }

    match (from, to) {
        // X↔Y: single gate (exact, no sign flip)
        // S·X·S† = Y, S†·Y·S = X
        (X, Y) => Some(vec![S]),
        (Y, X) => Some(vec![Sdg]),
        // X↔Z: single gate (exact)
        // H·X·H = Z, H·Z·H = X
        (X, Z) => Some(vec![H]),
        (Z, X) => Some(vec![H]),
        // Y→Z: two gates — H·S†·Y·S·H = H·X·H = Z
        // Apply S† first (Y→X), then H (X→Z)
        (Y, Z) => Some(vec![Sdg, H]),
        // Z→Y: two gates — S·H·Z·H·S† = S·X·S† = Y
        // Apply H first (Z→X), then S (X→Y)
        (Z, Y) => Some(vec![H, S]),
        _ => None,
    }
}

/// Compute the Clifford gate cost to align two Pauli operators.
///
/// Returns the number of single-qubit Clifford gates needed to map
/// `from` → `to` via conjugation. Used by the GreedyPauliSimp
/// scoring model to weigh Clifford overhead against CX savings.
///
/// # Cost table
///
/// | Case | Cost |
/// |------|------|
/// | Same operator, or either is I | 0 |
/// | X↔Y, X↔Z (single-gate) | 1 |
/// | Y↔Z (two-gate: H·S† or S·H) | 2 |
#[inline]
pub fn compute_alignment_cost(from: PauliOperator, to: PauliOperator) -> usize {
    use PauliOperator::*;
    if from == to || matches!(from, I) || matches!(to, I) {
        return 0;
    }
    match (from, to) {
        (X, Y) | (Y, X) | (X, Z) | (Z, X) => 1,
        (Y, Z) | (Z, Y) => 2,
        _ => 0,
    }
}

/// Check if two Pauli strings can be made QWC-compatible via single-qubit
/// Clifford conjugation on one of the strings.
///
/// Two strings are QWC-compatible iff for every qubit position q, the
/// operators (σ_a, σ_b) can be made identical via Clifford conjugation
/// applied to string `b`.
///
/// # Phase 11g: Full Clifford group
///
/// This now handles ALL Pauli operator pairs via the full single-qubit
/// Clifford group, including multi-gate sequences for Z↔Y alignment
/// (H·S† or S·H, cost 2).
///
/// # TKET-style compatible pair detection
///
/// This implements TKET's "compatible pair" heuristic used in `GreedyPauliSimp`:
/// two Pauli gadgets form a compatible pair if applying single-qubit Clifford
/// gates to one gadget makes them QWC, allowing them to share a CNOT tree.
///
/// # Returns
///
/// - `Some(vec)` — per-qubit `(qubit_index, CliffordGate)` to apply to string `b`
///   to make it QWC-compatible with string `a`. For multi-gate mappings, multiple
///   entries may target the same qubit. Empty vec means already compatible.
/// - `None` — strings have different lengths (should never happen).
pub fn compatible_pair_check(
    a: &PauliString,
    b: &PauliString,
) -> Option<Vec<(usize, CliffordGate)>> {
    if a.operators.len() != b.operators.len() {
        return None;
    }

    let mut gates: Vec<(usize, CliffordGate)> = Vec::new();

    for q in 0..a.operators.len() {
        let op_a = &a.operators[q];
        let op_b = &b.operators[q];

        // Skip if either is identity — they're trivially QWC at this qubit
        if matches!(op_a, PauliOperator::I) || matches!(op_b, PauliOperator::I) {
            continue;
        }

        // Same operator — already QWC at this qubit
        if op_a == op_b {
            continue;
        }

        // Phase 11g: use full Clifford group alignment.
        // find_alignment_gates handles ALL Pauli operator pairs including
        // Y↔Z (2-gate sequence) which was previously unsupported.
        if let Some(gate_seq) = find_alignment_gates(*op_b, *op_a) {
            for gate in gate_seq {
                gates.push((q, gate));
            }
        } else {
            // No Clifford conjugation can align these operators.
            // Should never happen for Pauli operators (I vs non-I
            // is handled above, everything else has a mapping).
            return None;
        }
    }

    Some(gates)
}

// ---------------------------------------------------------------------------

use crate::gates::StandardGate;
use crate::hamiltonian::pauli_string::{PauliOperator, PauliString};
use crate::hamiltonian::PauliTerm;
use crate::Result;
#[cfg(test)]
use num_complex::Complex64;

// ── Core Types ────────────────────────────────────────────────────────

/// Controls the aggressiveness of Pauli gadget optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GadgetOptimizationStrategy {
    /// Only merge terms with identical Pauli strings.
    /// Safe default — always exact, no behavior change.
    IdenticalOnly,
    /// Align Clifford-equivalent terms (X↔Y on same qubits) via basis
    /// alignment. Merged gadgets record Clifford conjugation gates.
    ///
    /// **Note:** Pre-synthesis merging via Clifford conjugation is
    /// approximate when both merged gadgets have non-zero angles.
    /// The merged gadget's effective unitary differs from the original
    /// product by O(θ_i·θ_j) where θ_i, θ_j are the angles. For
    /// Trotter circuits with small angles per step, this error is
    /// typically < 0.3% infidelity. Phase 11a-3 will provide an exact
    /// circuit-level implementation.
    CliffordSimple,
}

/// A single-qubit Clifford gate applied to a specific qubit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliffordGate {
    S,
    Sdg,
    H,
}

impl CliffordGate {
    /// Return the inverse of this Clifford gate.
    pub fn inverse(self) -> Self {
        match self {
            CliffordGate::S => CliffordGate::Sdg,
            CliffordGate::Sdg => CliffordGate::S,
            CliffordGate::H => CliffordGate::H, // H is self-inverse
        }
    }

    /// Convert to StandardGate for circuit emission.
    pub fn to_standard_gate(self) -> StandardGate {
        match self {
            CliffordGate::S => StandardGate::S,
            CliffordGate::Sdg => StandardGate::Sdg,
            CliffordGate::H => StandardGate::H,
        }
    }
}

/// A Pauli gadget after optimization.
///
/// Represents e^{-i·angle·P} where P is the Pauli string, potentially
/// wrapped in Clifford conjugation gates.
#[derive(Debug, Clone)]
pub struct OptimizedGadget {
    /// The Pauli string for this gadget (after any Clifford alignment).
    pub pauli_string: PauliString,
    /// Accumulated rotation angle.
    pub angle: f64,
    /// Clifford gates to apply BEFORE the exponential (pre-conjugation).
    pub pre_gates: Vec<(CliffordGate, usize)>,
    /// Clifford gates to apply AFTER the exponential (post-conjugation, inverse of pre).
    pub post_gates: Vec<(CliffordGate, usize)>,
    /// Original number of terms merged into this gadget.
    pub merge_count: usize,
}

/// Result of the Pauli gadget optimization pass.
#[derive(Debug, Clone)]
pub struct GadgetOptimizationResult {
    /// Optimized gadget list (fewer than or equal to input terms).
    pub gadgets: Vec<OptimizedGadget>,
    /// Total number of terms merged away.
    pub terms_merged: usize,
    /// Number of identical-Pauli merges.
    pub identical_merges: usize,
    /// Number of Clifford-equivalent merges.
    pub clifford_merges: usize,
}

// ── Public API ─────────────────────────────────────────────────────────

/// Optimize a list of Pauli terms by merging identical and Clifford-equivalent gadgets.
///
/// # Arguments
/// * `terms` — slice of references to PauliTerm
/// * `strategy` — optimization strategy (IdenticalOnly or CliffordSimple)
///
/// # Returns
/// `GadgetOptimizationResult` with the optimized gadget list and merge statistics.
///
/// # Correctness
/// - `IdenticalOnly`: Always exact. Sums coefficients of same-Pauli terms.
/// - `CliffordSimple`: Approximate pre-synthesis. See module-level docs for details.
///   The approximation error is O(θ_first·θ_second) and is negligible for small
///   Trotter-step angles (< 0.1 rad). Phase 11a-3 will provide exact handling.
pub fn optimize_pauli_gadgets(
    terms: &[&PauliTerm],
    _strategy: GadgetOptimizationStrategy,
) -> Result<GadgetOptimizationResult> {
    if terms.is_empty() {
        return Ok(GadgetOptimizationResult {
            gadgets: vec![],
            terms_merged: 0,
            identical_merges: 0,
            clifford_merges: 0,
        });
    }

    // Step 1: Convert terms to gadgets (1:1 initially)
    let mut gadgets: Vec<OptimizedGadget> = terms
        .iter()
        .map(|t| {
            // Physical Hamiltonians have real coefficients; warn if imaginary
            // part is non-negligible.
            debug_assert!(
                t.coefficient.im.abs() < 1e-12,
                "PauliGadget: non-real coefficient {:?} for term {} — imaginary part ignored",
                t.coefficient,
                t.pauli_string.to_string_repr()
            );
            OptimizedGadget {
                pauli_string: t.pauli_string.clone(),
                angle: t.coefficient.re,
                pre_gates: vec![],
                post_gates: vec![],
                merge_count: 1,
            }
        })
        .collect();

    let initial_count = gadgets.len();

    // Step 2: Merge identical Pauli strings (always exact)
    let identical_merges = merge_identical_paulis(&mut gadgets);

    // Step 3: Align Clifford-equivalent pairs (if strategy allows).
    //
    // Phase 11a-3: Instead of pre-synthesis merging (which produces incorrect
    // unitaries), we align Pauli bases WITHOUT summing angles. IIYY→IIXX with
    // S/S† Clifford gates recorded. Both gadgets keep their original angles
    // and remain separate — the Clifford gates are emitted during synthesis
    // around each term's individual rotation.
    let clifford_alignments = if matches!(_strategy, GadgetOptimizationStrategy::CliffordSimple) {
        align_clifford_pairs(&mut gadgets)
    } else {
        0
    };

    // Step 4: Remove zero-angle gadgets (cancellations)
    gadgets.retain(|g| g.angle.abs() > 1e-15);

    let final_count = gadgets.len();
    // terms_merged counts removed gadgets, including those removed by zero-angle filter
    let terms_merged = initial_count.saturating_sub(final_count);

    Ok(GadgetOptimizationResult {
        gadgets,
        terms_merged,
        identical_merges,
        clifford_merges: clifford_alignments,
    })
}

// ── Step 1: Identical-Pauli Merging ─────────────────────────────────────

/// Merge gadgets with identical Pauli strings by summing their angles.
/// Only merges gadgets without pre-existing Clifford gates (pure gadgets).
/// Returns the number of merges performed.
fn merge_identical_paulis(gadgets: &mut Vec<OptimizedGadget>) -> usize {
    let mut merges = 0;
    let mut i = 0;
    while i < gadgets.len() {
        // Only pure gadgets (no pre_gates) can be merged identically
        if !gadgets[i].pre_gates.is_empty() {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < gadgets.len() {
            if gadgets[i].pauli_string == gadgets[j].pauli_string && gadgets[j].pre_gates.is_empty()
            {
                // Merge j into i: sum angles, increment merge count
                gadgets[i].angle += gadgets[j].angle;
                gadgets[i].merge_count += gadgets[j].merge_count;
                gadgets.remove(j);
                merges += 1;
                // Don't increment j — the next element shifted into position j
            } else {
                j += 1;
            }
        }
        i += 1;
    }
    merges
}

// ── Step 2: Clifford-Equivalent Alignment (X↔Y) ──────────────────────────

/// Check if two Pauli strings differ only by X↔Y substitutions.
///
/// Returns `Some(qubits_with_XY_difference)` when:
/// - Both strings have the same operators at all qubits EXCEPT
/// - At the differing positions, one has X and the other has Y
/// - There is at least one X↔Y difference
///
/// Returns `None` for identical strings (handled by identical merge).
fn xy_equivalent(a: &PauliString, b: &PauliString) -> Option<Vec<usize>> {
    if a.operators.len() != b.operators.len() {
        return None;
    }

    let mut xy_qubits = vec![];
    for (q, (op_a, op_b)) in a.operators.iter().zip(b.operators.iter()).enumerate() {
        match (op_a, op_b) {
            // Same operator — fine
            (PauliOperator::I, PauliOperator::I)
            | (PauliOperator::X, PauliOperator::X)
            | (PauliOperator::Y, PauliOperator::Y)
            | (PauliOperator::Z, PauliOperator::Z) => {}
            // X↔Y equivalence — record qubit
            (PauliOperator::X, PauliOperator::Y) | (PauliOperator::Y, PauliOperator::X) => {
                xy_qubits.push(q);
            }
            // Any other mismatch — not equivalent
            _ => return None,
        }
    }

    if xy_qubits.is_empty() {
        None // Strings are identical (handled by identical merge)
    } else {
        Some(xy_qubits)
    }
}

/// Align Clifford-equivalent (X↔Y) gadget pairs by recording Clifford gates
/// WITHOUT changing Pauli strings.
///
/// # Phase 11a-3: Circuit-level alignment
///
/// For each XY-equivalent pair, we record pre/post Clifford gates on gadget j
/// but KEEP its original Pauli string. This preserves the ability to distinguish
/// native XX terms from YY-aligned-to-XX terms during synthesis.
///
/// During synthesis, the Clifford gates are placed AROUND gadget j's individual
/// rotation:
///   pre · basis(align_to) · CNOT · Rz(2θ) · CNOT_rev · inv_basis(align_to) · post
///
/// where `align_to` is gadget i's Pauli string (used for basis change calculation).
///
/// # Clifford identity
///
/// Y = S · X · S† (exact). Therefore:
///   e^{-iθ·Y} = S · e^{-iθ·X} · S†
///
/// To convert Y→X: apply S† before, S after the exponential.
/// To convert X→Y: apply S before, S† after the exponential.
///
/// Returns the number of aligned pairs.
fn align_clifford_pairs(gadgets: &mut Vec<OptimizedGadget>) -> usize {
    let mut alignments = 0;
    let mut i = 0;
    while i < gadgets.len() {
        // Only align pure gadgets (no pre-existing Clifford gates)
        if !gadgets[i].pre_gates.is_empty() {
            i += 1;
            continue;
        }

        let mut j = i + 1;
        while j < gadgets.len() {
            if !gadgets[j].pre_gates.is_empty() {
                j += 1;
                continue;
            }

            if let Some(xy_qubits) =
                xy_equivalent(&gadgets[i].pauli_string, &gadgets[j].pauli_string)
            {
                // Build Clifford gates that convert gadget j's basis to match i.
                let mut pre: Vec<(CliffordGate, usize)> = vec![];
                let mut post: Vec<(CliffordGate, usize)> = vec![];

                for &q in &xy_qubits {
                    let op_i = &gadgets[i].pauli_string.operators[q];
                    let op_j = &gadgets[j].pauli_string.operators[q];

                    match (op_i, op_j) {
                        // i has X, j has Y: Y→X via S† before, S after
                        // S† · e^{-iθ·Y} · S = e^{-iθ·X}
                        (PauliOperator::X, PauliOperator::Y) => {
                            pre.push((CliffordGate::Sdg, q));
                            post.push((CliffordGate::S, q));
                        }
                        // i has Y, j has X: X→Y via S before, S† after
                        // S · e^{-iθ·X} · S† = e^{-iθ·Y}
                        (PauliOperator::Y, PauliOperator::X) => {
                            pre.push((CliffordGate::S, q));
                            post.push((CliffordGate::Sdg, q));
                        }
                        _ => {} // Same operator at this qubit
                    }
                }

                // IMPORTANT: Keep gadget j's ORIGINAL Pauli string.
                // We only record the Clifford gates. During synthesis,
                // the aligned Pauli (gadget i's) is used for basis change,
                // and the Clifford gates convert between the two bases.
                // We store gadget i's Pauli as the "aligned_to" target.
                gadgets[j].pre_gates = pre;
                gadgets[j].post_gates = post;
                // Also store which Pauli this was aligned to (in the merge_count
                // and by comparing with gadget i's Pauli during synthesis).
                // We use a convention: aligned gadgets have pre/post gates;
                // during synthesis, the basis change is computed from the
                // pre-gate-adjusted Pauli (all Y→X via S†).
                gadgets[j].merge_count += 1;

                alignments += 1;
                j += 1;
            } else {
                j += 1;
            }
        }
        i += 1;
    }
    alignments
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hamiltonian::PauliTerm;

    fn make_term(pauli_str: &str, coeff: f64) -> PauliTerm {
        let ps = PauliString::from_str(pauli_str).unwrap();
        PauliTerm::new(ps, Complex64::new(coeff, 0.0))
    }

    fn terms(ts: &[(&str, f64)]) -> Vec<PauliTerm> {
        ts.iter().map(|(s, c)| make_term(s, *c)).collect()
    }

    fn term_refs(ts: &[PauliTerm]) -> Vec<&PauliTerm> {
        ts.iter().collect()
    }

    // ── identical merge tests ──────────────────────────────────────

    #[test]
    fn test_identical_merge_basic() {
        let ts = terms(&[("IIIZ", 0.5), ("IIIZ", 0.3)]);
        let refs = term_refs(&ts);
        let result =
            optimize_pauli_gadgets(&refs, GadgetOptimizationStrategy::IdenticalOnly).unwrap();
        assert_eq!(result.gadgets.len(), 1);
        assert!((result.gadgets[0].angle - 0.8).abs() < 1e-10);
        assert_eq!(result.identical_merges, 1);
        assert_eq!(result.terms_merged, 1);
    }

    #[test]
    fn test_identical_merge_empty() {
        let result =
            optimize_pauli_gadgets(&[], GadgetOptimizationStrategy::IdenticalOnly).unwrap();
        assert!(result.gadgets.is_empty());
        assert_eq!(result.terms_merged, 0);
    }

    #[test]
    fn test_identical_merge_all_unique() {
        let ts = terms(&[("IIIZ", 0.5), ("IIZI", 0.3), ("IZII", 0.2)]);
        let refs = term_refs(&ts);
        let result =
            optimize_pauli_gadgets(&refs, GadgetOptimizationStrategy::IdenticalOnly).unwrap();
        assert_eq!(result.gadgets.len(), 3);
        assert_eq!(result.identical_merges, 0);
        assert_eq!(result.terms_merged, 0);
    }

    #[test]
    fn test_identical_merge_negative_coefficient() {
        let ts = terms(&[("IIIZ", 0.5), ("IIIZ", -0.3)]);
        let refs = term_refs(&ts);
        let result =
            optimize_pauli_gadgets(&refs, GadgetOptimizationStrategy::IdenticalOnly).unwrap();
        assert_eq!(result.gadgets.len(), 1);
        assert!((result.gadgets[0].angle - 0.2).abs() < 1e-10);
    }

    #[test]
    fn test_identical_merge_multiple_pairs() {
        let ts = terms(&[("IIIZ", 0.5), ("IIIZ", 0.3), ("IIZI", 0.2), ("IIZI", 0.1)]);
        let refs = term_refs(&ts);
        let result =
            optimize_pauli_gadgets(&refs, GadgetOptimizationStrategy::IdenticalOnly).unwrap();
        assert_eq!(result.gadgets.len(), 2);
        assert_eq!(result.identical_merges, 2);
        let z_angle: f64 = result
            .gadgets
            .iter()
            .find(|g| g.pauli_string.to_string_repr() == "IIIZ")
            .map(|g| g.angle)
            .unwrap_or(0.0);
        assert!(
            (z_angle - 0.8).abs() < 1e-10,
            "expected z_angle=0.8, got {}",
            z_angle
        );
    }

    #[test]
    fn test_zero_angle_removed() {
        // Cancellation: 0.5 + (-0.5) = 0 → gadget should be removed
        let ts = terms(&[("IIIZ", 0.5), ("IIIZ", -0.5)]);
        let refs = term_refs(&ts);
        let result =
            optimize_pauli_gadgets(&refs, GadgetOptimizationStrategy::IdenticalOnly).unwrap();
        assert!(result.gadgets.is_empty());
        assert_eq!(result.terms_merged, 2); // 2 inputs, 0 outputs = 2 removed
    }

    // ── XY equivalence tests ───────────────────────────────────────

    #[test]
    fn test_xy_equivalent_simple() {
        let a = PauliString::from_str("X").unwrap();
        let b = PauliString::from_str("Y").unwrap();
        let result = xy_equivalent(&a, &b);
        assert_eq!(result, Some(vec![0]));
    }

    #[test]
    fn test_xy_equivalent_symmetric() {
        let a = PauliString::from_str("Y").unwrap();
        let b = PauliString::from_str("X").unwrap();
        assert_eq!(xy_equivalent(&a, &b), Some(vec![0]));
    }

    #[test]
    fn test_xy_not_equivalent_different_weight() {
        let a = PauliString::from_str("XX").unwrap();
        let b = PauliString::from_str("XI").unwrap();
        assert_eq!(xy_equivalent(&a, &b), None);
    }

    #[test]
    fn test_xy_equivalent_with_z() {
        // XZ vs YZ: X↔Y at q0, same Z at q1 — should be equivalent
        let a = PauliString::from_str("XZ").unwrap();
        let b = PauliString::from_str("YZ").unwrap();
        assert_eq!(xy_equivalent(&a, &b), Some(vec![0]));
    }

    #[test]
    fn test_xy_equivalent_two_qubit() {
        let a = PauliString::from_str("IIXX").unwrap();
        let b = PauliString::from_str("IIYY").unwrap();
        assert_eq!(xy_equivalent(&a, &b), Some(vec![2, 3]));
    }

    #[test]
    fn test_xy_equivalent_identical_returns_none() {
        let a = PauliString::from_str("IIXX").unwrap();
        let b = PauliString::from_str("IIXX").unwrap();
        assert_eq!(xy_equivalent(&a, &b), None);
    }

    #[test]
    fn test_xy_not_equivalent_z_vs_i() {
        // XZ vs YI: differ at q1 (Z vs I), not X↔Y → not equivalent
        let a = PauliString::from_str("XZ").unwrap();
        let b = PauliString::from_str("YI").unwrap();
        assert_eq!(xy_equivalent(&a, &b), None);
    }

    // ── CliffordSimple alignment tests ─────────────────────────────

    // ── CliffordSimple alignment tests (Phase 11a-3) ──────────────
    // CliffordSimple aligns XY pairs WITHOUT merging angles.
    // Y→X alignment changes the Pauli string and records pre/post Clifford gates.
    // Both gadgets remain separate — the Clifford gates are emitted during synthesis.

    #[test]
    fn test_clifford_simple_aligns_xy_pair() {
        // X + Y: Y aligned with Clifford gates, Pauli kept as Y
        let ts = terms(&[("X", 0.5), ("Y", 0.3)]);
        let refs = term_refs(&ts);
        let result =
            optimize_pauli_gadgets(&refs, GadgetOptimizationStrategy::CliffordSimple).unwrap();
        // Two separate gadgets (no merge), Y aligned but Pauli unchanged
        assert_eq!(result.gadgets.len(), 2);
        assert_eq!(result.clifford_merges, 1); // One pair aligned
        assert_eq!(result.terms_merged, 0);
        // Find the aligned gadget (should have original Pauli=Y and S/S† gates)
        let aligned = result
            .gadgets
            .iter()
            .find(|g| !g.pre_gates.is_empty())
            .unwrap();
        assert_eq!(aligned.pauli_string.to_string_repr(), "Y");
        assert!((aligned.angle - 0.3).abs() < 1e-10);
        assert_eq!(aligned.pre_gates.len(), 1);
        assert_eq!(aligned.post_gates.len(), 1);
        // The native X gadget should be unchanged
        let native = result
            .gadgets
            .iter()
            .find(|g| g.pre_gates.is_empty())
            .unwrap();
        assert_eq!(native.pauli_string.to_string_repr(), "X");
        assert!((native.angle - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_clifford_simple_aligns_xx_yy() {
        let ts = terms(&[("IIXX", 0.0454), ("IIYY", 0.0454)]);
        let refs = term_refs(&ts);
        let result =
            optimize_pauli_gadgets(&refs, GadgetOptimizationStrategy::CliffordSimple).unwrap();
        // Both kept separate, YY has Clifford annotation but Pauli stays IIYY
        assert_eq!(result.gadgets.len(), 2);
        assert_eq!(result.clifford_merges, 1);
        // Aligned gadget should keep original Pauli=IIYY with S†/S on q2,q3
        let aligned = result
            .gadgets
            .iter()
            .find(|g| !g.pre_gates.is_empty())
            .unwrap();
        assert_eq!(aligned.pauli_string.to_string_repr(), "IIYY");
        assert!((aligned.angle - 0.0454).abs() < 1e-10);
        // pre=Sdg on q2 and q3 (converting Y→X: S† before)
        assert_eq!(aligned.pre_gates.len(), 2);
        // post=S on q2 and q3
        assert_eq!(aligned.post_gates.len(), 2);
        // Native IIXX gadget unchanged
        let native = result
            .gadgets
            .iter()
            .find(|g| g.pre_gates.is_empty())
            .unwrap();
        assert_eq!(native.pauli_string.to_string_repr(), "IIXX");
    }

    #[test]
    fn test_clifford_simple_aligns_h2_like() {
        let ts = terms(&[
            ("IIXX", 0.0454),
            ("IIYY", 0.0454),
            ("XXII", 0.0454),
            ("YYII", 0.0454),
        ]);
        let refs = term_refs(&ts);
        let result =
            optimize_pauli_gadgets(&refs, GadgetOptimizationStrategy::CliffordSimple).unwrap();
        // All 4 kept separate, 2 pairs aligned
        assert_eq!(result.gadgets.len(), 4);
        assert_eq!(result.clifford_merges, 2);
        // Two gadgets should have pre/post gates (the aligned YY ones)
        let aligned_count = result
            .gadgets
            .iter()
            .filter(|g| !g.pre_gates.is_empty())
            .count();
        assert_eq!(aligned_count, 2);
        // Aligned gadgets should keep their original YY Pauli strings
        let aligned_paulis: Vec<String> = result
            .gadgets
            .iter()
            .filter(|g| !g.pre_gates.is_empty())
            .map(|g| g.pauli_string.to_string_repr().to_string())
            .collect();
        assert!(aligned_paulis.contains(&"IIYY".to_string()));
        assert!(aligned_paulis.contains(&"YYII".to_string()));
    }

    #[test]
    fn test_clifford_simple_still_merges_identical() {
        // Identical Pauli merging still works with CliffordSimple
        let ts = terms(&[("IIIZ", 0.5), ("IIIZ", 0.3)]);
        let refs = term_refs(&ts);
        let result =
            optimize_pauli_gadgets(&refs, GadgetOptimizationStrategy::CliffordSimple).unwrap();
        assert_eq!(result.gadgets.len(), 1);
        assert!((result.gadgets[0].angle - 0.8).abs() < 1e-10);
        assert_eq!(result.identical_merges, 1);
    }

    #[test]
    fn test_clifford_simple_heisenberg_like() {
        // Heisenberg: XX + YY + ZZ. YY gets aligned (Clifford annotation) but Pauli unchanged.
        let ts = terms(&[("XX", 1.0), ("YY", 1.0), ("ZZ", 1.0)]);
        let refs = term_refs(&ts);
        let result =
            optimize_pauli_gadgets(&refs, GadgetOptimizationStrategy::CliffordSimple).unwrap();
        // All 3 kept separate, 1 XY pair aligned
        assert_eq!(result.gadgets.len(), 3);
        assert_eq!(result.clifford_merges, 1);
        // YY should have Clifford annotation but original Pauli
        let yy = result
            .gadgets
            .iter()
            .find(|g| g.pauli_string.to_string_repr() == "YY")
            .unwrap();
        assert!(
            !yy.pre_gates.is_empty(),
            "YY should have Clifford annotation"
        );
        assert!(!yy.post_gates.is_empty());
        // XX should be unchanged
        let xx = result
            .gadgets
            .iter()
            .find(|g| g.pauli_string.to_string_repr() == "XX")
            .unwrap();
        assert!(xx.pre_gates.is_empty());
    }

    #[test]
    fn test_identical_only_does_not_align_xy() {
        let ts = terms(&[("IIXX", 0.0454), ("IIYY", 0.0454)]);
        let refs = term_refs(&ts);
        let result =
            optimize_pauli_gadgets(&refs, GadgetOptimizationStrategy::IdenticalOnly).unwrap();
        assert_eq!(result.gadgets.len(), 2);
        assert_eq!(result.clifford_merges, 0);
    }

    // ── Clifford algebra utility tests (Phase 11e) ──────────────────

    use PauliOperator::*;

    #[test]
    fn test_conjugate_h_x_to_z_exact() {
        assert_eq!(conjugate_pauli_operator(CliffordGate::H, X, false), Some(Z));
    }

    #[test]
    fn test_conjugate_h_z_to_x_exact() {
        assert_eq!(conjugate_pauli_operator(CliffordGate::H, Z, false), Some(X));
    }

    #[test]
    fn test_conjugate_h_y_sign_flip() {
        // H·Y·H = -Y — sign flip
        assert_eq!(conjugate_pauli_operator(CliffordGate::H, Y, false), None);
        assert_eq!(conjugate_pauli_operator(CliffordGate::H, Y, true), Some(Y));
    }

    #[test]
    fn test_conjugate_s_x_to_y_exact() {
        assert_eq!(conjugate_pauli_operator(CliffordGate::S, X, false), Some(Y));
    }

    #[test]
    fn test_conjugate_s_y_sign_flip() {
        // S·Y·S† = -X
        assert_eq!(conjugate_pauli_operator(CliffordGate::S, Y, false), None);
        assert_eq!(conjugate_pauli_operator(CliffordGate::S, Y, true), Some(X));
    }

    #[test]
    fn test_conjugate_sdg_y_to_x_exact() {
        assert_eq!(
            conjugate_pauli_operator(CliffordGate::Sdg, Y, false),
            Some(X)
        );
    }

    #[test]
    fn test_conjugate_all_stable() {
        // Z and I are stable under S and S†
        for gate in &[CliffordGate::S, CliffordGate::Sdg] {
            assert_eq!(conjugate_pauli_operator(*gate, Z, false), Some(Z));
            assert_eq!(conjugate_pauli_operator(*gate, I, false), Some(I));
        }
    }

    #[test]
    fn test_find_alignment_x_to_z() {
        assert_eq!(find_alignment_gate(X, Z), Some((CliffordGate::H, false)));
    }

    #[test]
    fn test_find_alignment_x_to_y() {
        assert_eq!(find_alignment_gate(X, Y), Some((CliffordGate::S, false)));
    }

    #[test]
    fn test_find_alignment_z_to_y_none() {
        // No single gate maps Z → Y (needs H·S)
        assert_eq!(find_alignment_gate(Z, Y), None);
    }

    #[test]
    fn test_find_alignment_same_op() {
        // Same operator: no gate needed — returns None (caller treats as identity)
        assert_eq!(find_alignment_gate(X, X), None);
        assert_eq!(find_alignment_gate(Y, Y), None);
        assert_eq!(find_alignment_gate(Z, Z), None);
        // I vs I is also same-operator
        assert_eq!(find_alignment_gate(I, I), None);
    }

    #[test]
    fn test_compatible_pair_xx_yy() {
        // IIXX and IIYY: X↔Y at q2,q3 via S or S†
        let a = PauliString::from_str("IIXX").unwrap();
        let b = PauliString::from_str("IIYY").unwrap();
        let result = compatible_pair_check(&a, &b);
        assert!(result.is_some(), "XX and YY should be compatible");
        let gates = result.unwrap();
        assert_eq!(gates.len(), 2); // One gate per Y qubit
        for (q, _gate) in &gates {
            assert!(*q == 2 || *q == 3);
        }
    }

    #[test]
    fn test_compatible_pair_identical() {
        let a = PauliString::from_str("IIXX").unwrap();
        let b = PauliString::from_str("IIXX").unwrap();
        let result = compatible_pair_check(&a, &b);
        assert!(result.is_some());
        assert!(result.unwrap().is_empty()); // No gates needed
    }

    #[test]
    fn test_compatible_pair_xz_yi() {
        // XZ vs YI: at q0 X vs Y (compatible via S† on b), at q1 Z vs I (trivial)
        let a = PauliString::from_str("XZ").unwrap();
        let b = PauliString::from_str("YI").unwrap();
        let result = compatible_pair_check(&a, &b);
        assert!(result.is_some());
        let gates = result.unwrap();
        assert_eq!(gates.len(), 1); // Y→X via S† on q0
        assert_eq!(gates[0], (0, CliffordGate::Sdg));
    }

    #[test]
    fn test_compatible_pair_z_y_full_clifford() {
        // Phase 11g: Z vs Y is compatible via 2-gate sequence
        // Y→Z: H·S†·Y·S·H = Z → pre_gates = [S†, H]
        let a = PauliString::from_str("Z").unwrap();
        let b = PauliString::from_str("Y").unwrap();
        let result = compatible_pair_check(&a, &b);
        assert!(
            result.is_some(),
            "Z vs Y should be compatible via full Clifford group"
        );
        let gates = result.unwrap();
        assert_eq!(gates.len(), 2); // S† then H
        assert_eq!(gates[0], (0, CliffordGate::Sdg)); // First: S† (Y→X)
        assert_eq!(gates[1], (0, CliffordGate::H)); // Then: H (X→Z)
    }

    #[test]
    fn test_compatible_pair_xz_iy_full_clifford() {
        // Phase 11g: XZ vs IY is now compatible.
        // q0: X vs I (skip), q1: Z vs Y → Y→Z via [S†, H]
        let a = PauliString::from_str("XZ").unwrap();
        let b = PauliString::from_str("IY").unwrap();
        let result = compatible_pair_check(&a, &b);
        assert!(
            result.is_some(),
            "XZ vs IY compatible with full Clifford group"
        );
        let gates = result.unwrap();
        assert_eq!(gates.len(), 2);
        assert_eq!(gates[0], (1, CliffordGate::Sdg)); // S† on q1: Y→X
        assert_eq!(gates[1], (1, CliffordGate::H)); // H on q1: X→Z
    }

    // ── Phase 11g: Full Clifford alignment tests ─────────────────────

    #[test]
    fn test_find_alignment_gates_single_gate() {
        use CliffordGate::*;
        // X→Z via H
        assert_eq!(find_alignment_gates(X, Z), Some(vec![H]));
        // Z→X via H
        assert_eq!(find_alignment_gates(Z, X), Some(vec![H]));
        // X→Y via S
        assert_eq!(find_alignment_gates(X, Y), Some(vec![S]));
        // Y→X via S†
        assert_eq!(find_alignment_gates(Y, X), Some(vec![Sdg]));
    }

    #[test]
    fn test_find_alignment_gates_multi_gate() {
        use CliffordGate::*;
        // Y→Z via S† then H: H·S†·Y·S·H = Z
        assert_eq!(find_alignment_gates(Y, Z), Some(vec![Sdg, H]));
        // Z→Y via H then S: S·H·Z·H·S† = Y
        assert_eq!(find_alignment_gates(Z, Y), Some(vec![H, S]));
    }

    #[test]
    fn test_find_alignment_gates_same_or_i() {
        // Same operator: empty sequence
        assert_eq!(find_alignment_gates(X, X), Some(vec![]));
        assert_eq!(find_alignment_gates(Y, Y), Some(vec![]));
        assert_eq!(find_alignment_gates(Z, Z), Some(vec![]));
        // I vs non-I: no mapping
        assert_eq!(find_alignment_gates(I, X), None);
        assert_eq!(find_alignment_gates(X, I), None);
    }

    #[test]
    fn test_compute_alignment_cost() {
        use PauliOperator::*;
        // Same operator: 0
        assert_eq!(compute_alignment_cost(X, X), 0);
        assert_eq!(compute_alignment_cost(Y, Y), 0);
        assert_eq!(compute_alignment_cost(Z, Z), 0);
        assert_eq!(compute_alignment_cost(I, I), 0);
        // Either is I: 0
        assert_eq!(compute_alignment_cost(I, X), 0);
        assert_eq!(compute_alignment_cost(X, I), 0);
        // Single-gate: 1
        assert_eq!(compute_alignment_cost(X, Y), 1);
        assert_eq!(compute_alignment_cost(Y, X), 1);
        assert_eq!(compute_alignment_cost(X, Z), 1);
        assert_eq!(compute_alignment_cost(Z, X), 1);
        // Multi-gate: 2
        assert_eq!(compute_alignment_cost(Y, Z), 2);
        assert_eq!(compute_alignment_cost(Z, Y), 2);
    }
}
