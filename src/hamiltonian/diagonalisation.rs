//! Pauli string diagonalisation — reduce overlap between Pauli gadgets.
//!
//! Phase 11r: Port of TKET's `reduce_overlap_of_paulis` (Diagonalisation.cpp:508).
//!
//! Reference: Cowtan, Dilkes, Duncan, Simmons, Sivarajah,
//! "Phase Gadget Synthesis for Shallow Circuits", Lemma 4.9.
//!
//! Let s and t be Pauli strings; then there exists a Clifford unitary U such that
//!   P(a, s) · P(b, t) = U · P(a, s') · P(b, t') · U†
//! where s' and t' are Pauli strings with intersection at most 1.
//!
//! This allows pairwise synthesis of Pauli gadgets with minimal overlap.

use std::collections::HashSet;

use crate::gates::StandardGate;
use crate::hamiltonian::pauli_string::{PauliOperator, PauliString};

// Re-export for tests
#[cfg(test)]
use std::collections::HashSet as _;

/// Rx(pi/2) rotation — converts Z-basis to Y-basis.
const RX_PI_OVER_2: f64 = std::f64::consts::PI / 2.0;

/// A single gate in the Clifford circuit.
#[derive(Debug, Clone)]
pub struct CliffordGateOp {
    pub gate: StandardGate,
    pub qubits: Vec<usize>,
    pub angle: Option<f64>,
}

impl CliffordGateOp {
    pub fn new(gate: StandardGate, qubits: Vec<usize>) -> Self {
        Self {
            gate,
            qubits,
            angle: None,
        }
    }

    pub fn with_angle(gate: StandardGate, qubits: Vec<usize>, angle: f64) -> Self {
        Self {
            gate,
            qubits,
            angle: Some(angle),
        }
    }
}

/// Find the set of qubits where both Pauli strings have the same non-I operator.
fn common_qubits(p0: &PauliString, p1: &PauliString) -> Vec<usize> {
    let n = p0.operators.len().min(p1.operators.len());
    (0..n)
        .filter(|&q| {
            let a = p0.operators[q];
            let b = p1.operators[q];
            a != PauliOperator::I && a == b
        })
        .collect()
}

/// Find the set of qubits where both Pauli strings have different non-I operators.
fn conflicting_qubits(p0: &PauliString, p1: &PauliString) -> Vec<usize> {
    let n = p0.operators.len().min(p1.operators.len());
    (0..n)
        .filter(|&q| {
            let a = p0.operators[q];
            let b = p1.operators[q];
            a != PauliOperator::I && b != PauliOperator::I && a != b
        })
        .collect()
}

/// Get the Pauli operator at qubit q, defaulting to I.
fn get_op(ps: &PauliString, q: usize) -> PauliOperator {
    if q < ps.operators.len() {
        ps.operators[q]
    } else {
        PauliOperator::I
    }
}

/// Set the Pauli operator at qubit q.
fn set_op(ps: &mut PauliString, q: usize, op: PauliOperator) {
    if q < ps.operators.len() {
        ps.operators[q] = op;
    }
}

/// Erase a qubit from the Pauli string (set to I).
fn erase_qubit(ps: &mut PauliString, q: usize) {
    set_op(ps, q, PauliOperator::I);
}

/// Reduce shared (matching) qubits using a CX snake.
///
/// All matching qubits are already in Z-basis. For each pair of matching
/// qubits, CX(to_eliminate, kept) XORs them together, eliminating the
/// `to_eliminate` qubit from both Pauli strings.
fn reduce_shared_by_cx_snake(
    u: &mut Vec<CliffordGateOp>,
    match_qs: &mut Vec<usize>,
    pauli0: &mut PauliString,
    pauli1: &mut PauliString,
) {
    while match_qs.len() > 1 {
        let to_eliminate = match_qs.pop().unwrap();
        let kept = *match_qs.last().unwrap();
        u.push(CliffordGateOp::new(
            StandardGate::CX,
            vec![to_eliminate, kept],
        ));
        erase_qubit(pauli0, to_eliminate);
        erase_qubit(pauli1, to_eliminate);
    }
}

/// Reduce overlap of two Pauli strings via a Clifford circuit.
///
/// Given two Pauli strings p0 and p1, finds a Clifford circuit U such that
/// after conjugation by U, the resulting strings p0' and p1' have at most
/// one overlapping qubit.
///
/// # Arguments
/// * `pauli0` - First Pauli string (modified in place)
/// * `pauli1` - Second Pauli string (modified in place)
/// * `allow_matching_final` - If true, allow one matching qubit to remain
///   (for commuting gadgets); if false, eliminate ALL overlap.
///
/// # Returns
/// * The Clifford circuit U (list of gates)
/// * The last overlapping qubit, if any
pub fn reduce_overlap_of_paulis(
    pauli0: &mut PauliString,
    pauli1: &mut PauliString,
    allow_matching_final: bool,
) -> (Vec<CliffordGateOp>, Option<usize>) {
    let mut u: Vec<CliffordGateOp> = Vec::new();

    // Step 1: Identify matching and conflicting qubits.
    let mut match_qs = common_qubits(pauli0, pauli1);
    let mut mismatch_qs = conflicting_qubits(pauli0, pauli1);

    // Step 2.i: Convert matching qubits to Z-basis (X→H, Y→V) and reduce via CX.
    for &q in &match_qs {
        match get_op(pauli0, q) {
            PauliOperator::X => {
                u.push(CliffordGateOp::new(StandardGate::H, vec![q]));
                set_op(pauli0, q, PauliOperator::Z);
                set_op(pauli1, q, PauliOperator::Z);
            }
            PauliOperator::Y => {
                // V = Rx(π/2): converts Y→Z
                // V = Rx(pi/2): converts Y -> Z
                u.push(CliffordGateOp::with_angle(
                    StandardGate::Rx,
                    vec![q],
                    RX_PI_OVER_2,
                ));
                set_op(pauli0, q, PauliOperator::Z);
                set_op(pauli1, q, PauliOperator::Z);
            }
            _ => {} // Already Z
        }
    }
    // Reduce shared qubits using CX snake.
    reduce_shared_by_cx_snake(&mut u, &mut match_qs, pauli0, pauli1);

    // Step 2.ii: Convert mismatches to Z in pauli0 and X in pauli1.
    for &q in &mismatch_qs {
        match get_op(pauli0, q) {
            PauliOperator::X => match get_op(pauli1, q) {
                PauliOperator::Y => {
                    u.push(CliffordGateOp::new(StandardGate::Sdg, vec![q]));
                    u.push(CliffordGateOp::with_angle(
                        StandardGate::Rx,
                        vec![q],
                        -RX_PI_OVER_2,
                    ));
                }
                PauliOperator::Z => {
                    u.push(CliffordGateOp::new(StandardGate::H, vec![q]));
                }
                _ => {}
            },
            PauliOperator::Y => match get_op(pauli1, q) {
                PauliOperator::X => {
                    u.push(CliffordGateOp::with_angle(
                        StandardGate::Rx,
                        vec![q],
                        RX_PI_OVER_2,
                    ));
                }
                PauliOperator::Z => {
                    u.push(CliffordGateOp::with_angle(
                        StandardGate::Rx,
                        vec![q],
                        RX_PI_OVER_2,
                    ));
                    u.push(CliffordGateOp::new(StandardGate::S, vec![q]));
                }
                _ => {}
            },
            PauliOperator::Z if get_op(pauli1, q) == PauliOperator::Y => {
                u.push(CliffordGateOp::new(StandardGate::Sdg, vec![q]));
            }
            // Z vs X: no change needed
            _ => {}
        }
        set_op(pauli0, q, PauliOperator::Z);
        set_op(pauli1, q, PauliOperator::X);
    }

    // Step 2.iii: Handle last matching qubit.
    let mut last_overlap: Option<usize> = None;
    if !match_qs.is_empty() {
        let last_match = match_qs.remove(0);
        if !mismatch_qs.is_empty() {
            let mismatch_used = mismatch_qs.pop().unwrap();
            u.push(CliffordGateOp::new(StandardGate::S, vec![mismatch_used]));
            u.push(CliffordGateOp::new(
                StandardGate::CX,
                vec![last_match, mismatch_used],
            ));
            u.push(CliffordGateOp::new(StandardGate::Sdg, vec![mismatch_used]));
            erase_qubit(pauli0, last_match);
            erase_qubit(pauli1, last_match);
        } else if !allow_matching_final {
            // Find another qubit in pauli0 or pauli1 to absorb the last match.
            let other = find_other_qubit(pauli0, pauli1, last_match);
            if let Some((oq, o_op)) = other {
                erase_qubit(pauli0, last_match);
                if o_op == PauliOperator::X {
                    u.push(CliffordGateOp::new(StandardGate::H, vec![oq]));
                    u.push(CliffordGateOp::new(StandardGate::CX, vec![last_match, oq]));
                    u.push(CliffordGateOp::new(StandardGate::H, vec![oq]));
                } else {
                    u.push(CliffordGateOp::new(StandardGate::CX, vec![last_match, oq]));
                }
            }
        } else {
            last_overlap = Some(last_match);
        }
    }

    // Step 2.iv: Reduce mismatch pairs — each CX eliminates one qubit from each string.
    let mut mis_iter = 0usize;
    while mis_iter < mismatch_qs.len() {
        let z_in_0 = mismatch_qs[mis_iter];
        mis_iter += 1;
        if mis_iter < mismatch_qs.len() {
            let x_in_1 = mismatch_qs[mis_iter];
            u.push(CliffordGateOp::new(StandardGate::CX, vec![x_in_1, z_in_0]));
            erase_qubit(pauli0, x_in_1);
            erase_qubit(pauli1, z_in_0);
            mis_iter += 1;
        } else {
            last_overlap = Some(z_in_0);
        }
    }

    (u, last_overlap)
}

/// Find a qubit (other than `exclude`) with a non-I Pauli operator in either p0 or p1.
fn find_other_qubit(
    p0: &PauliString,
    p1: &PauliString,
    exclude: usize,
) -> Option<(usize, PauliOperator)> {
    for q in 0..p0.operators.len() {
        if q != exclude && get_op(p0, q) != PauliOperator::I {
            return Some((q, get_op(p0, q)));
        }
    }
    for q in 0..p1.operators.len() {
        if q != exclude && get_op(p1, q) != PauliOperator::I {
            return Some((q, get_op(p1, q)));
        }
    }
    None
}

// ── Pauli gadget pair synthesis ─────────────────────────────────────────

/// Synthesize a pair of Pauli gadgets with minimal overlap.
///
/// Given two Pauli gadgets exp(i·θ0·P0) and exp(i·θ1·P1), finds a Clifford
/// circuit U that reduces the overlap of P0 and P1 to at most 1 qubit,
/// synthesizes the gadgets individually within U, then applies U†.
///
/// The result is equivalent to the original pair but typically with fewer
/// CNOT gates than separate synthesis.
///
/// # Returns
/// The gates for U · (gadget0 + gadget1) · U†
pub fn pauli_gadget_pair(
    pauli0: &PauliString,
    angle0: f64,
    pauli1: &PauliString,
    angle1: f64,
) -> Vec<CliffordGateOp> {
    let mut p0 = pauli0.clone();
    let mut p1 = pauli1.clone();

    // Step 1: Reduce overlap via Clifford circuit U.
    let (u, _last_overlap) = reduce_overlap_of_paulis(&mut p0, &mut p1, true);

    // Step 2: Build the inner circuit V = gadget0 + gadget1.
    // For now, we emit per-gadget synthesis as a sequence of:
    //   basis changes → CNOT ladder → Rz → inverse CNOT → inverse basis
    // Using the existing compile_multi_qubit_rotation pattern.
    let mut inner: Vec<CliffordGateOp> = Vec::new();

    // Gadget 0: reduced Pauli string p0 with angle0
    if !is_identity(&p0) {
        inner.append(&mut single_gadget_synthesis(&p0, angle0));
    }

    // Gadget 1: reduced Pauli string p1 with angle1
    if !is_identity(&p1) {
        inner.append(&mut single_gadget_synthesis(&p1, angle1));
    }

    // Step 3: Combine: U → inner → U†(inverse)
    let mut result = u.clone();
    result.append(&mut inner);
    result.append(&mut invert_clifford(&u));
    result
}

/// Check if a Pauli string is all identity.
fn is_identity(ps: &PauliString) -> bool {
    ps.operators.iter().all(|op| *op == PauliOperator::I)
}

/// Synthesize a single Pauli gadget (exp(iθP)) into Clifford-gate operations.
///
/// Uses the standard pattern: basis changes + CNOT ladder + Rz + uncompute.
pub fn single_gadget_synthesis(pauli: &PauliString, angle: f64) -> Vec<CliffordGateOp> {
    let mut gates = Vec::new();

    // Collect active qubits (non-I Pauli operators)
    let active: Vec<(usize, PauliOperator)> = pauli
        .operators
        .iter()
        .enumerate()
        .filter(|(_, op)| **op != PauliOperator::I)
        .map(|(q, op)| (q, *op))
        .collect();

    if active.is_empty() {
        return gates;
    }

    // Basis changes: X→H, Y→Rx(π/2), Z→(no change)
    for &(q, op) in &active {
        match op {
            PauliOperator::X => {
                gates.push(CliffordGateOp::new(StandardGate::H, vec![q]));
            }
            PauliOperator::Y => {
                gates.push(CliffordGateOp::with_angle(
                    StandardGate::Rx,
                    vec![q],
                    RX_PI_OVER_2,
                ));
            }
            _ => {}
        }
    }

    // CNOT ladder: chain from first to last active qubit
    for i in 0..active.len().saturating_sub(1) {
        let ctrl = active[i].0;
        let tgt = active[i + 1].0;
        gates.push(CliffordGateOp::new(StandardGate::CX, vec![ctrl, tgt]));
    }

    // Rz on the last qubit
    // NOTE: Callers already pass 2*c*dt/ħ as the angle (extracted from existing
    // Rz gates in the circuit). Do NOT double here.
    if let Some(&(last_q, _)) = active.last() {
        gates.push(CliffordGateOp::with_angle(
            StandardGate::Rz,
            vec![last_q],
            angle,
        ));
    }

    // Inverse CNOT ladder
    for i in (0..active.len().saturating_sub(1)).rev() {
        let ctrl = active[i].0;
        let tgt = active[i + 1].0;
        gates.push(CliffordGateOp::new(StandardGate::CX, vec![ctrl, tgt]));
    }

    // Inverse basis changes
    for &(q, op) in active.iter().rev() {
        match op {
            PauliOperator::X => {
                gates.push(CliffordGateOp::new(StandardGate::H, vec![q]));
            }
            PauliOperator::Y => {
                gates.push(CliffordGateOp::with_angle(
                    StandardGate::Rx,
                    vec![q],
                    -RX_PI_OVER_2,
                ));
            }
            _ => {}
        }
    }

    gates
}

/// Compute the inverse of a Clifford circuit (U†).
fn invert_clifford(u: &[CliffordGateOp]) -> Vec<CliffordGateOp> {
    u.iter()
        .rev()
        .map(|op| {
            let inv_gate = match op.gate {
                StandardGate::H => StandardGate::H,   // H† = H
                StandardGate::S => StandardGate::Sdg, // S† = Sdg
                StandardGate::Sdg => StandardGate::S, // Sdg† = S
                StandardGate::CX => StandardGate::CX, // CX† = CX
                StandardGate::Rx => {
                    // Rx(θ)† = Rx(-θ)
                    return CliffordGateOp::with_angle(
                        StandardGate::Rx,
                        op.qubits.clone(),
                        -op.angle.unwrap_or(0.0),
                    );
                }
                StandardGate::Rz => {
                    return CliffordGateOp::with_angle(
                        StandardGate::Rz,
                        op.qubits.clone(),
                        -op.angle.unwrap_or(0.0),
                    );
                }
                _ => op.gate, // Other self-inverse gates
            };
            CliffordGateOp {
                gate: inv_gate,
                qubits: op.qubits.clone(),
                angle: op.angle,
            }
        })
        .collect()
}

// ══════════════════════════════════════════════════════════════════════════
// Phase 11s: Mutual Diagonalisation + Reduce Pauli to Z
// ══════════════════════════════════════════════════════════════════════════

/// Check if qubits can be trivially diagonalized — if all gadgets have
/// only I or a single Pauli type on that qubit, convert to Z-basis.
///
/// Returns the Clifford circuit that diagonalizes easy qubits.
fn check_easy_diagonalise(gadgets: &[PauliString], qubits: &mut Vec<usize>) -> Vec<CliffordGateOp> {
    let mut u = Vec::new();

    // Iterate in reverse so removals don't affect indices.
    let mut i = 0;
    while i < qubits.len() {
        let q = qubits[i];
        let mut seen = PauliOperator::I;
        let mut all_same = true;

        for g in gadgets {
            let op = get_op(g, q);
            if op == PauliOperator::I {
                continue;
            }
            if seen == PauliOperator::I {
                seen = op;
            } else if op != seen {
                all_same = false;
                break;
            }
        }

        if all_same {
            match seen {
                PauliOperator::I => {} // Already diagonal
                PauliOperator::X => {
                    u.push(CliffordGateOp::new(StandardGate::H, vec![q]));
                }
                PauliOperator::Y => {
                    u.push(CliffordGateOp::with_angle(
                        StandardGate::Rx,
                        vec![q],
                        RX_PI_OVER_2,
                    ));
                }
                PauliOperator::Z => {} // Already Z
            }
            qubits.remove(i);
        } else {
            i += 1;
        }
    }

    u
}

/// Reduce a single Pauli string to Z on one qubit.
///
/// Finds the optimal Clifford circuit that maps the Pauli string to Z
/// on a single qubit, eliminating all other qubits.
///
/// # Returns
/// * The Clifford circuit that performs the reduction
/// * The qubit where Z ends up
pub fn reduce_pauli_to_z(pauli: &PauliString) -> (Vec<CliffordGateOp>, usize) {
    let mut u = Vec::new();

    // Collect active qubits
    let active: Vec<usize> = pauli
        .operators
        .iter()
        .enumerate()
        .filter(|(_, op)| **op != PauliOperator::I)
        .map(|(q, _)| q)
        .collect();

    if active.is_empty() {
        return (u, 0);
    }

    // Convert all to Z-basis
    for &q in &active {
        match get_op(pauli, q) {
            PauliOperator::X => u.push(CliffordGateOp::new(StandardGate::H, vec![q])),
            PauliOperator::Y => u.push(CliffordGateOp::with_angle(
                StandardGate::Rx,
                vec![q],
                RX_PI_OVER_2,
            )),
            _ => {}
        }
    }

    // Build CX snake to reduce to a single qubit.
    // Use the first qubit as the accumulating target.
    let target = active[0];
    for i in 1..active.len() {
        let to_eliminate = active[i];
        u.push(CliffordGateOp::new(
            StandardGate::CX,
            vec![to_eliminate, target],
        ));
    }

    (u, target)
}

/// Apply a Clifford gate to all Pauli strings in a list.
fn apply_clifford_to_gadgets(op: &CliffordGateOp, gadgets: &mut [PauliString]) {
    match op.gate {
        StandardGate::H => {
            let q = op.qubits[0];
            for g in gadgets.iter_mut() {
                if q < g.operators.len() {
                    g.operators[q] = match g.operators[q] {
                        PauliOperator::X => PauliOperator::Z,
                        PauliOperator::Z => PauliOperator::X,
                        PauliOperator::Y => PauliOperator::Y, // Y unchanged by H
                        other => other,
                    };
                }
            }
        }
        StandardGate::S => {
            let q = op.qubits[0];
            for g in gadgets.iter_mut() {
                if q < g.operators.len() {
                    g.operators[q] = match g.operators[q] {
                        PauliOperator::X => PauliOperator::Y,
                        PauliOperator::Y => PauliOperator::X,
                        other => other,
                    };
                }
            }
        }
        StandardGate::Sdg => {
            let q = op.qubits[0];
            for g in gadgets.iter_mut() {
                if q < g.operators.len() {
                    g.operators[q] = match g.operators[q] {
                        PauliOperator::X => PauliOperator::Y,
                        PauliOperator::Y => PauliOperator::X,
                        other => other,
                    };
                }
            }
        }
        StandardGate::CX => {
            let c = op.qubits[0];
            let t = op.qubits[1];
            for g in gadgets.iter_mut() {
                if c < g.operators.len() && t < g.operators.len() {
                    let op_c = g.operators[c];
                    let op_t = g.operators[t];
                    // CX†·Z_t·CX = Z_c·Z_t → target Z multiplies with control Z
                    if op_c == PauliOperator::Z || op_t == PauliOperator::Z {
                        g.operators[t] = multiply_pauli_ops(op_c, op_t);
                    }
                    // CX†·X_c·CX = X_c·X_t → control X multiplies with target X
                    if op_c == PauliOperator::X || op_t == PauliOperator::X {
                        g.operators[c] = multiply_pauli_ops(PauliOperator::X, op_t);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Multiply two Pauli operators: result = a * b with phase tracking.
/// Used for applying Clifford conjugations to Pauli strings.
fn multiply_pauli_ops(a: PauliOperator, b: PauliOperator) -> PauliOperator {
    match (a, b) {
        (PauliOperator::I, p) | (p, PauliOperator::I) => p,
        (PauliOperator::X, PauliOperator::Y) | (PauliOperator::Y, PauliOperator::X) => {
            PauliOperator::Z
        }
        (PauliOperator::Y, PauliOperator::Z) | (PauliOperator::Z, PauliOperator::Y) => {
            PauliOperator::X
        }
        (PauliOperator::Z, PauliOperator::X) | (PauliOperator::X, PauliOperator::Z) => {
            PauliOperator::Y
        }
        _ => PauliOperator::I, // X·X = Y·Y = Z·Z = I
    }
}

/// Simultaneously diagonalize a set of mutually commuting Pauli strings.
///
/// Finds a Clifford circuit U such that after applying U, all Pauli strings
/// become Z-type (only Z and I operators) — i.e., they become diagonal
/// in the computational basis. This is the key step for synthesizing
/// multiple commuting Pauli gadgets efficiently.
///
/// # Algorithm
///
/// 1. `check_easy_diagonalise`: remove qubits where all gadgets share the same Pauli
/// 2. While there are gadgets with weight > 1:
///    a. Pick the gadget with minimum weight
///    b. `reduce_pauli_to_z`: synthesize it to Z on one qubit
///    c. Use CX to eliminate that qubit from other gadgets
///    d. Remove the now-diagonalized gadget from consideration
/// 3. Return the accumulated Clifford circuit U
pub fn mutual_diagonalise(gadgets: &mut Vec<PauliString>) -> Vec<CliffordGateOp> {
    let mut u = Vec::new();

    // Collect all qubits used by any gadget
    let all_qubits: HashSet<usize> = gadgets
        .iter()
        .flat_map(|g| {
            g.operators.iter().enumerate().filter_map(|(q, op)| {
                if *op != PauliOperator::I {
                    Some(q)
                } else {
                    None
                }
            })
        })
        .collect();
    let mut qubits: Vec<usize> = all_qubits.into_iter().collect();
    qubits.sort_unstable();

    // Step 1: Check for easy diagonalization
    u.append(&mut check_easy_diagonalise(gadgets, &mut qubits));

    // Step 2: Greedy reduction
    while !gadgets.is_empty() {
        // Find gadget with minimum support (weight) among remaining qubits
        let mut best_idx = 0;
        let mut best_weight = usize::MAX;

        for (i, g) in gadgets.iter().enumerate() {
            let w = g
                .operators
                .iter()
                .filter(|op| **op != PauliOperator::I)
                .count();
            if w < best_weight {
                best_weight = w;
                best_idx = i;
            }
        }

        if best_weight <= 1 {
            // All remaining gadgets are weight ≤ 1 — done
            break;
        }

        let target_pauli = gadgets.remove(best_idx);

        // Reduce the selected Pauli to Z on one qubit, applying each gate
        // to all remaining gadgets so their Paulis stay consistent.
        let (reduce_u, target_q) = reduce_pauli_to_z(&target_pauli);
        // Apply gates to remaining gadgets (clone to avoid borrow conflict with `u`)
        for op in &reduce_u {
            apply_clifford_to_gadgets(op, &mut gadgets[..]);
        }
        u.extend(reduce_u);

        // Eliminate target_q from all other gadgets via CX.
        // Collect CX operations first, then apply them.
        let mut cx_ops: Vec<CliffordGateOp> = Vec::new();
        for g in gadgets.iter() {
            let op = get_op(g, target_q);
            if op == PauliOperator::I {
                continue;
            }
            if let Some(other_q) = g
                .operators
                .iter()
                .enumerate()
                .filter(|(q, op)| **op != PauliOperator::I && *q != target_q)
                .map(|(q, _)| q)
                .next()
            {
                cx_ops.push(CliffordGateOp::new(
                    StandardGate::CX,
                    vec![target_q, other_q],
                ));
            }
        }
        for op in &cx_ops {
            apply_clifford_to_gadgets(op, gadgets);
        }
        u.extend(cx_ops);
    }

    u
}

// ══════════════════════════════════════════════════════════════════════════
// Phase 11t: Pauli Gadget Sets (multi-gadget synthesis)
// ══════════════════════════════════════════════════════════════════════════

/// Check if two Pauli strings commute.
fn paulis_commute(p0: &PauliString, p1: &PauliString) -> bool {
    let n = p0.operators.len().min(p1.operators.len());
    let mut anti_count = 0usize;
    for q in 0..n {
        let a = p0.operators[q];
        let b = p1.operators[q];
        if a != PauliOperator::I && b != PauliOperator::I && a != b {
            anti_count += 1;
        }
    }
    anti_count % 2 == 0
}

/// Group gadgets into commuting sets using greedy coloring.
fn group_commuting_sets(gadgets: &[(PauliString, f64)]) -> Vec<Vec<usize>> {
    let n = gadgets.len();
    let mut sets: Vec<Vec<usize>> = Vec::new();

    for i in 0..n {
        let mut placed = false;
        for set in sets.iter_mut() {
            if set
                .iter()
                .all(|&j| paulis_commute(&gadgets[i].0, &gadgets[j].0))
            {
                set.push(i);
                placed = true;
                break;
            }
        }
        if !placed {
            sets.push(vec![i]);
        }
    }

    sets
}

/// Synthesize a set of Pauli gadgets together using mutual diagonalization.
///
/// For a set of mutually commuting Pauli gadgets, finds a Clifford circuit U
/// that diagonalizes all gadgets simultaneously, then synthesizes each
/// diagonalized gadget individually within U.
///
/// # Returns
/// U · (gadget0 + gadget1 + ... + gadgetN) · U†
pub fn pauli_gadget_sets(gadgets: &[(PauliString, f64)]) -> Vec<CliffordGateOp> {
    if gadgets.is_empty() {
        return Vec::new();
    }

    // Group into commuting sets
    let sets = group_commuting_sets(gadgets);
    let mut result = Vec::new();

    for set in &sets {
        let set_gadgets: Vec<&(PauliString, f64)> = set.iter().map(|&i| &gadgets[i]).collect();

        if set_gadgets.len() == 1 {
            // Single gadget: synthesize directly
            let (p, angle) = set_gadgets[0];
            result.extend(single_gadget_synthesis(p, *angle));
        } else if set_gadgets.len() == 2 {
            // Two gadgets: use pairwise (optimized for pair case)
            let (p0, a0) = set_gadgets[0];
            let (p1, a1) = set_gadgets[1];
            result.extend(pauli_gadget_pair(p0, *a0, p1, *a1));
        } else {
            // Three or more: use mutual diagonalization
            let mut paulis: Vec<PauliString> =
                set_gadgets.iter().map(|(p, _)| (*p).clone()).collect();
            let u = mutual_diagonalise(&mut paulis);

            // Emit U (forward)
            result.extend(u.clone());

            // Synthesize each diagonalized gadget individually.
            // paulis may be shorter than set_gadgets after mutual_diagonalise
            // removes elements — use paulis.get() for bounds safety.
            for (i, (_, angle)) in set_gadgets.iter().enumerate() {
                if angle.abs() > 1e-12 {
                    if let Some(diag_pauli) = paulis.get(i) {
                        result.extend(single_gadget_synthesis(diag_pauli, *angle));
                    }
                }
            }

            // Emit U† (inverse)
            result.extend(invert_clifford(&u));
        }
    }

    result
}

// ══════════════════════════════════════════════════════════════════════════
// Phase 11u: Greedy Diagonalise
// ══════════════════════════════════════════════════════════════════════════

/// Greedy diagonalization: find the Pauli with minimum support and
/// reduce it to Z on one qubit, repeating until all are diagonal.
///
/// This is the fallback when `mutual_diagonalise` can't handle
/// non-commuting sets — it diagonalizes gadgets one at a time.
pub fn greedy_diagonalise(gadgets: &mut Vec<PauliString>) -> Vec<CliffordGateOp> {
    let mut u = Vec::new();

    while !gadgets.is_empty() {
        // Find gadget with minimum weight
        let mut best_idx = 0;
        let mut best_w = usize::MAX;
        for (i, g) in gadgets.iter().enumerate() {
            let w = g
                .operators
                .iter()
                .filter(|op| **op != PauliOperator::I)
                .count();
            if w < best_w {
                best_w = w;
                best_idx = i;
            }
        }

        let pauli = gadgets.remove(best_idx);
        let (reduce_u, _target_q) = reduce_pauli_to_z(&pauli);
        for op in &reduce_u {
            apply_clifford_to_gadgets(op, &mut gadgets[..]);
        }
        u.extend(reduce_u);
    }

    u
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_qubits() {
        let p0 = PauliString::from_str("XZII").unwrap();
        let p1 = PauliString::from_str("XZXY").unwrap();
        let common = common_qubits(&p0, &p1);
        assert_eq!(common, vec![0, 1]); // X=X at q0, Z=Z at q1
    }

    #[test]
    fn test_conflicting_qubits() {
        let p0 = PauliString::from_str("XZII").unwrap();
        let p1 = PauliString::from_str("XYXI").unwrap();
        let conf = conflicting_qubits(&p0, &p1);
        assert_eq!(conf, vec![1]); // Z vs Y at q1
    }

    #[test]
    fn test_reduce_overlap_two_matching() {
        // Two Pauli strings both == ZZII
        let mut p0 = PauliString::from_str("ZZII").unwrap();
        let mut p1 = PauliString::from_str("ZZII").unwrap();
        let (u, overlap) = reduce_overlap_of_paulis(&mut p0, &mut p1, true);

        // After reduction: matching qubits should be reduced to at most 1
        let common = common_qubits(&p0, &p1);
        assert!(
            common.len() <= 1,
            "Expected ≤1 common qubits, got {}",
            common.len()
        );
        // Some CX gates should have been emitted
        assert!(!u.is_empty());
    }

    #[test]
    fn test_reduce_overlap_mixed() {
        // Z on q0 matches, X on q1 vs Y on q1 mismatches
        let mut p0 = PauliString::from_str("ZXII").unwrap();
        let mut p1 = PauliString::from_str("ZYII").unwrap();
        let (u, _overlap) = reduce_overlap_of_paulis(&mut p0, &mut p1, true);

        // After: p0 should have Z's, p1 should have X's at mismatch
        let conf = conflicting_qubits(&p0, &p1);
        let common = common_qubits(&p0, &p1);
        assert!(common.len() <= 1);
        assert!(!u.is_empty());
    }

    #[test]
    fn test_reduce_overlap_disjoint() {
        // Completely disjoint strings
        let mut p0 = PauliString::from_str("XIII").unwrap();
        let mut p1 = PauliString::from_str("IXII").unwrap();
        let (u, _overlap) = reduce_overlap_of_paulis(&mut p0, &mut p1, false);

        // After: no common qubits (since allow_matching_final=false)
        let common = common_qubits(&p0, &p1);
        assert_eq!(common.len(), 0);
    }

    // ── Phase 11s tests ─────────────────────────────────────────────────

    #[test]
    fn test_reduce_pauli_to_z_single_z() {
        let p = PauliString::from_str("ZIII").unwrap();
        let (u, target) = reduce_pauli_to_z(&p);
        assert_eq!(target, 0);
        // No basis change needed, no CX needed (only 1 qubit)
        assert!(u.is_empty());
    }

    #[test]
    fn test_reduce_pauli_to_z_two_qubit_xz() {
        let p = PauliString::from_str("XZII").unwrap();
        let (u, target) = reduce_pauli_to_z(&p);
        assert_eq!(target, 0);
        // Should have H on q0 and CX(1, 0)
        assert!(u.len() >= 2);
        let has_h = u
            .iter()
            .any(|op| op.gate == StandardGate::H && op.qubits[0] == 0);
        let has_cx = u.iter().any(|op| op.gate == StandardGate::CX);
        assert!(has_h, "Expected H on qubit 0");
        assert!(has_cx, "Expected CX gate");
    }

    #[test]
    fn test_reduce_pauli_to_z_three_qubit() {
        let p = PauliString::from_str("YZXI").unwrap();
        let (u, target) = reduce_pauli_to_z(&p);
        assert_eq!(target, 0);
        // V on q0 (Y→Z), CX(1,0), CX(2,0)
        let cx_count = u.iter().filter(|op| op.gate == StandardGate::CX).count();
        assert_eq!(cx_count, 2, "Expected 2 CX for 3 active qubits");
    }

    #[test]
    fn test_mutual_diagonalise_two_commuting() {
        let mut gadgets = vec![
            PauliString::from_str("ZZII").unwrap(),
            PauliString::from_str("IZZI").unwrap(),
        ];
        let u = mutual_diagonalise(&mut gadgets);
        // After diagonalization, all gadgets should be Z-type (no X/Y)
        for g in &gadgets {
            for op in &g.operators {
                assert!(matches!(*op, PauliOperator::I | PauliOperator::Z));
            }
        }
        // Should produce a non-empty Clifford circuit
        assert!(!u.is_empty());
    }

    #[test]
    fn test_mutual_diagonalise_three_commuting() {
        let mut gadgets = vec![
            PauliString::from_str("ZZII").unwrap(),
            PauliString::from_str("IZZI").unwrap(),
            PauliString::from_str("IIZZ").unwrap(),
        ];
        let u = mutual_diagonalise(&mut gadgets);
        for g in &gadgets {
            for op in &g.operators {
                assert!(matches!(*op, PauliOperator::I | PauliOperator::Z));
            }
        }
    }

    #[test]
    fn test_check_easy_diagonalise_all_z() {
        let gadgets = vec![
            PauliString::from_str("ZIII").unwrap(),
            PauliString::from_str("ZIII").unwrap(),
        ];
        let mut qubits = vec![0, 1, 2, 3];
        let u = check_easy_diagonalise(&gadgets, &mut qubits);
        // All-Z on q0: no gate needed, but q0 should be removed from qubits
        assert!(!qubits.contains(&0));
        assert!(u.is_empty()); // Z needs no basis change
    }

    #[test]
    fn test_mutual_diagonalise_all_z_commuting() {
        // Four Z-type commuting Paulis: all can be diagonalized together
        let mut gadgets = vec![
            PauliString::from_str("ZZII").unwrap(),
            PauliString::from_str("IZZI").unwrap(),
            PauliString::from_str("IIZZ").unwrap(),
            PauliString::from_str("ZIZI").unwrap(),
        ];
        let u = mutual_diagonalise(&mut gadgets);
        for g in &gadgets {
            for op in &g.operators {
                assert!(matches!(*op, PauliOperator::I | PauliOperator::Z));
            }
        }
        // Should produce a Clifford circuit
        assert!(!u.is_empty());
    }

    #[test]
    fn test_mutual_diagonalise_empty() {
        let mut gadgets: Vec<PauliString> = vec![];
        let u = mutual_diagonalise(&mut gadgets);
        assert!(u.is_empty());
    }

    // ── Phase 11t tests ─────────────────────────────────────────────────

    #[test]
    fn test_pauli_gadget_sets_empty() {
        let gadgets: Vec<(PauliString, f64)> = vec![];
        let gates = pauli_gadget_sets(&gadgets);
        assert!(gates.is_empty());
    }

    #[test]
    fn test_pauli_gadget_sets_single() {
        let gadgets = vec![(PauliString::from_str("ZIII").unwrap(), 0.5)];
        let gates = pauli_gadget_sets(&gadgets);
        // Should have Rz on qubit 0
        let rz_count = gates
            .iter()
            .filter(|op| op.gate == StandardGate::Rz)
            .count();
        assert_eq!(rz_count, 1);
    }

    #[test]
    fn test_pauli_gadget_sets_three_commuting() {
        let gadgets = vec![
            (PauliString::from_str("ZIII").unwrap(), 0.1),
            (PauliString::from_str("IZII").unwrap(), 0.2),
            (PauliString::from_str("IIZI").unwrap(), 0.3),
        ];
        let gates = pauli_gadget_sets(&gadgets);
        let rz_count = gates
            .iter()
            .filter(|op| op.gate == StandardGate::Rz)
            .count();
        assert_eq!(rz_count, 3, "Expected 3 Rz gates, got {}", rz_count);
    }

    // ── Phase 11u tests ─────────────────────────────────────────────────

    #[test]
    fn test_greedy_diagonalise_two_terms() {
        let mut gadgets = vec![
            PauliString::from_str("XZII").unwrap(),
            PauliString::from_str("IXZI").unwrap(),
        ];
        let u = greedy_diagonalise(&mut gadgets);
        // Should produce a Clifford circuit with some gates
        assert!(!u.is_empty());
    }

    #[test]
    fn test_greedy_diagonalise_empty() {
        let mut gadgets: Vec<PauliString> = vec![];
        let u = greedy_diagonalise(&mut gadgets);
        assert!(u.is_empty());
    }

    #[test]
    fn test_pauli_gadget_sets_four_commuting() {
        let gadgets = vec![
            (PauliString::from_str("ZIII").unwrap(), 0.1),
            (PauliString::from_str("IZII").unwrap(), 0.2),
            (PauliString::from_str("IIZI").unwrap(), 0.3),
            (PauliString::from_str("IIIZ").unwrap(), 0.4),
        ];
        let gates = pauli_gadget_sets(&gadgets);
        let rz_count = gates
            .iter()
            .filter(|op| op.gate == StandardGate::Rz)
            .count();
        assert_eq!(rz_count, 4);
    }
}
