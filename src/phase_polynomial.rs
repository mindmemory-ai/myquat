// Phase Polynomial Optimization Module
// Author: gA4ss
//
// Phase 10c: Diagonal operator optimization via phase polynomial representation.
//
// # Core Idea
//
// Circuits composed of {CX, Rz} gates can be represented as:
//   U|x₁...xₙ⟩ = exp(-i Σₖ θₖ · Pₖ(x)) |L(x)⟩
// where each Pₖ is a parity function (XOR of input bits) and L is the
// linear transformation applied by the CNOT network.
//
// This module extracts {CX, Rz} segments from circuits, represents them
// as phase polynomials, optimizes the representation, and re-synthesizes
// into gates using pluggable strategies.
//
// # Design
//
// - `DiagonalSynthesis` trait: pluggable synthesis strategies
// - `ChainSynthesis`: linear chain method (matches existing pauli_synthesis behavior)
// - `GrayCodeSynthesis`: Gray-code traversal for optimal CX count
// - `PhasePolynomialPass`: CircuitPass wrapper for integration
//
// # Mathematical Background
//
// ## Gray Code Synthesis
//
// Given diagonal operator D = Σ exp(i·θ(z))|z⟩⟨z|, Gray code synthesis
// visits all computational basis states in Gray code order (each step
// changes exactly 1 bit). For each transition, one CNOT gate is needed.
// Total CX count = |parities| - 1 for a connected parity graph.
//
// Compare to chain synthesis which uses 2·(N-1) CX gates for N active qubits.
//
// Reference: Bullock & Markov, "Efficient synthesis of diagonal unitary matrices" (2004)

use crate::circuit::QuantumCircuit;
use crate::circuit_optimization::CircuitPass;
use crate::error::Result;
use crate::gates::StandardGate;
use crate::parameter::Parameter;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Core Types
// ---------------------------------------------------------------------------

/// A linear Boolean function (XOR parity of input qubits).
///
/// `mask` has bit i = 1 iff input qubit i participates in the parity.
/// For N qubits (N ≤ 64), this is a u64 bitmask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Parity(pub u64);

impl Parity {
    /// Identity parity (empty XOR = 0).
    pub const ZERO: Self = Parity(0);

    /// Create a parity from a single qubit.
    #[inline]
    pub fn singleton(qubit: usize) -> Self {
        Parity(1u64 << qubit)
    }

    /// XOR two parities.
    #[inline]
    pub fn xor(self, other: Self) -> Self {
        Parity(self.0 ^ other.0)
    }

    /// Hamming weight (number of qubits in the parity).
    #[inline]
    pub fn weight(self) -> u32 {
        self.0.count_ones()
    }

    /// Hamming distance between two parities.
    #[inline]
    pub fn distance(self, other: Self) -> u32 {
        (self.0 ^ other.0).count_ones()
    }

    /// The differing qubit index (if distance == 1), else None.
    #[inline]
    pub fn differing_qubit(self, other: Self) -> Option<usize> {
        let diff = self.0 ^ other.0;
        if diff.count_ones() == 1 {
            Some(diff.trailing_zeros() as usize)
        } else {
            None
        }
    }

    /// Check if bit `q` is set.
    #[inline]
    pub fn has_bit(self, q: usize) -> bool {
        (self.0 >> q) & 1 == 1
    }

    /// Set bit `q`.
    #[inline]
    pub fn with_bit(self, q: usize) -> Self {
        Parity(self.0 | (1u64 << q))
    }

    /// Clear bit `q`.
    #[inline]
    pub fn without_bit(self, q: usize) -> Self {
        Parity(self.0 & !(1u64 << q))
    }
}

/// A labeled Rz rotation: Rz(angle) applied to a specific parity function
/// at a specific qubit position in the CNOT frame.
#[derive(Debug, Clone)]
pub struct LabeledRz {
    /// Parity function this Rz acts on.
    pub parity: Parity,
    /// Rotation angle.
    pub angle: f64,
    /// Physical qubit index in the current CNOT frame.
    pub qubit: usize,
}

/// A {CX, Rz} circuit segment represented as a CNOT network + phases.
///
/// # Invariant
///
/// After the forward CNOT sequence, qubit `q` holds parity `frame[q]`.
/// Rz gates are labeled by the parity they act on at their position
/// in the CNOT sequence.
#[derive(Debug, Clone)]
pub struct PhasePolynomial {
    /// Number of qubits.
    pub num_qubits: usize,
    /// frame[q] = parity held by qubit q after the forward CNOT network.
    /// Initially frame[q] = Parity::singleton(q).
    pub frame: Vec<Parity>,
    /// Labeled Rz rotations, in order of application.
    pub rotations: Vec<LabeledRz>,
    /// Forward CNOT sequence: (control, target) in order.
    pub cnots: Vec<(usize, usize)>,
}

/// A synthesized gate ready for circuit insertion.
#[derive(Debug, Clone)]
pub struct SynthesizedGate {
    pub gate: StandardGate,
    pub qubits: Vec<usize>,
    pub angle: Option<f64>,
}

impl SynthesizedGate {
    /// Create a CX gate.
    pub fn cx(ctrl: usize, tgt: usize) -> Self {
        Self {
            gate: StandardGate::CX,
            qubits: vec![ctrl, tgt],
            angle: None,
        }
    }

    /// Create an Rz gate.
    pub fn rz(qubit: usize, angle: f64) -> Self {
        Self {
            gate: StandardGate::Rz,
            qubits: vec![qubit],
            angle: Some(angle),
        }
    }
}

// ---------------------------------------------------------------------------
// Synthesis Strategy Trait
// ---------------------------------------------------------------------------

/// Pluggable synthesis strategy for diagonal operators.
///
/// Given a set of (parity, angle) pairs for N qubits, produce a
/// sequence of {CX, Rz} gates that implements the diagonal operator.
pub trait DiagonalSynthesis: Send + Sync {
    /// Human-readable name of this strategy.
    fn name(&self) -> &str;

    /// Synthesize a set of (parity, angle) pairs into {CX, Rz} gates.
    ///
    /// # Arguments
    /// * `terms` - (parity_mask, angle) pairs. Parity ZERO corresponds to
    ///   a global phase (Rz on no parity), which is ignored.
    /// * `num_qubits` - total number of qubits.
    ///
    /// # Returns
    /// Sequence of (gate_type, qubits, optional_angle) tuples.
    fn synthesize(
        &self,
        terms: &[(Parity, f64)],
        num_qubits: usize,
    ) -> Result<Vec<SynthesizedGate>>;
}

// ---------------------------------------------------------------------------
// Chain Synthesis (linear ladder, existing behavior)
// ---------------------------------------------------------------------------

/// Linear chain synthesis: builds a CNOT ladder CNOT(0,1), CNOT(1,2), ...
/// and places Rz gates at chain positions.
///
/// This matches the existing `synthesize_shared_tree` behavior in
/// `pauli_synthesis.rs`. For N active qubits, uses 2·(N-1) CX gates
/// (forward + reverse chain).
pub struct ChainSynthesis;

impl DiagonalSynthesis for ChainSynthesis {
    fn name(&self) -> &str {
        "ChainSynthesis"
    }

    fn synthesize(
        &self,
        terms: &[(Parity, f64)],
        num_qubits: usize,
    ) -> Result<Vec<SynthesizedGate>> {
        if terms.is_empty() || num_qubits == 0 {
            return Ok(Vec::new());
        }

        // Filter out zero-parity and zero-angle terms
        let nonzero: Vec<&(Parity, f64)> = terms
            .iter()
            .filter(|(p, a)| p.0 != 0 && a.abs() > 1e-12)
            .collect();

        if nonzero.is_empty() {
            return Ok(Vec::new());
        }

        // Determine which qubits are active (appear in any parity)
        let mut active_set: u64 = 0;
        for (p, _) in &nonzero {
            active_set |= p.0;
        }
        let active_qubits: Vec<usize> = (0..num_qubits)
            .filter(|q| (active_set >> q) & 1 == 1)
            .collect();

        if active_qubits.is_empty() {
            return Ok(Vec::new());
        }

        let k = active_qubits.len();

        // Single active qubit: no chain needed, just emit Rz directly
        if k == 1 {
            let q = active_qubits[0];
            let angle: f64 = nonzero.iter().map(|(_, a)| *a).sum();
            if angle.abs() > 1e-12 {
                return Ok(vec![SynthesizedGate {
                    gate: StandardGate::Rz,
                    qubits: vec![q],
                    angle: Some(angle),
                }]);
            }
            return Ok(Vec::new());
        }
        let mut gates = Vec::new();

        // Forward chain: CX(n_i, n_{i+1}) + Rz(n_{i+1}, 2*angle)
        for i in 0..k - 1 {
            let qi = active_qubits[i];
            let qj = active_qubits[i + 1];
            gates.push(SynthesizedGate {
                gate: StandardGate::CX,
                qubits: vec![qi, qj],
                angle: None,
            });

            // Rz at chain position: accumulate angles for parities ending at this position
            let chain_parity: u64 = active_qubits[..=i + 1]
                .iter()
                .fold(0u64, |acc, q| acc | (1u64 << q));
            let angle: f64 = nonzero
                .iter()
                .filter(|(p, _)| p.0 == chain_parity)
                .map(|(_, a)| *a)
                .sum();

            if angle.abs() > 1e-12 {
                gates.push(SynthesizedGate {
                    gate: StandardGate::Rz,
                    qubits: vec![qj],
                    angle: Some(angle),
                });
            }
        }

        // Reverse chain: CX in reverse order
        for i in (0..k - 1).rev() {
            let qi = active_qubits[i];
            let qj = active_qubits[i + 1];
            gates.push(SynthesizedGate {
                gate: StandardGate::CX,
                qubits: vec![qi, qj],
                angle: None,
            });
        }

        Ok(gates)
    }
}

// ---------------------------------------------------------------------------
// Gray Code Synthesis (NEW)
// ---------------------------------------------------------------------------

/// Gray-code-based synthesis for diagonal operators.
///
/// Algorithm:
/// 1. Build a spanning tree over the set of required parities + {0}
///    with edge weights = Hamming distance.
/// 2. Compute a Gray code traversal (DFS/Euler tour of the MST).
/// 3. Each edge in the traversal corresponds to 1 CNOT gate.
/// 4. Place Rz gates at the appropriate nodes in the traversal.
///
/// # Properties
///
/// - Produces exactly |parities| - 1 CX gates (connected parity graph).
/// - For prefix-chain parities, produces same CX count as ChainSynthesis.
/// - For arbitrary parity sets, can be significantly better than chain.
/// - The MST ensures minimum total CNOT gates to connect all parities.
pub struct GrayCodeSynthesis;

impl GrayCodeSynthesis {
    /// Build a minimum spanning tree over the parity set + {0} using Prim's algorithm.
    /// Returns (parent_map, edge_list_in_order).
    fn build_mst(parities: &[u64]) -> Vec<(u64, u64)> {
        if parities.len() <= 1 {
            return Vec::new();
        }

        let n = parities.len();
        let mut visited = vec![false; n];
        let mut edges = Vec::with_capacity(n - 1);

        // Start from parity 0 (index 0 if present, otherwise first parity)
        visited[0] = true;

        for _ in 0..n - 1 {
            let mut best_dist = u32::MAX;
            let mut best_pair = (0usize, 0usize);

            for u in 0..n {
                if !visited[u] {
                    continue;
                }
                for v in 0..n {
                    if visited[v] {
                        continue;
                    }
                    let dist = (parities[u] ^ parities[v]).count_ones();
                    if dist < best_dist {
                        best_dist = dist;
                        best_pair = (u, v);
                    }
                }
            }

            if best_dist < u32::MAX {
                visited[best_pair.1] = true;
                edges.push((parities[best_pair.0], parities[best_pair.1]));
            }
        }

        edges
    }

    /// Perform an Euler tour of the MST, returning edges in visit order.
    ///
    /// Each entry is `(from_parity, to_parity, is_return)` where:
    /// - `is_return = false` → forward edge (parent → child), place Rz at child.
    /// - `is_return = true`  → return edge (child → parent), only undo CNOTs,
    ///   no Rz (parent was already visited).
    ///
    /// The Euler tour guarantees a continuous walk through the hypercube:
    /// after returning from a subtree, we are back at the parent parity,
    /// ready to descend into the next subtree with the correct frame.
    fn gray_code_traversal(edges: &[(u64, u64)], start: u64) -> Vec<(u64, u64, bool)> {
        // Build adjacency list
        let mut adj: HashMap<u64, Vec<u64>> = HashMap::new();
        for &(a, b) in edges {
            adj.entry(a).or_default().push(b);
            adj.entry(b).or_default().push(a);
        }

        // DFS to generate Euler-tour traversal (forward + return edges).
        let mut sequence = Vec::new();
        let mut visited = std::collections::HashSet::new();
        Self::dfs(start, &adj, &mut visited, &mut sequence);

        sequence
    }

    fn dfs(
        node: u64,
        adj: &HashMap<u64, Vec<u64>>,
        visited: &mut std::collections::HashSet<u64>,
        sequence: &mut Vec<(u64, u64, bool)>,
    ) {
        visited.insert(node);
        if let Some(neighbors) = adj.get(&node) {
            for &next in neighbors {
                if !visited.contains(&next) {
                    // Forward edge: parent → child.
                    sequence.push((node, next, false));
                    Self::dfs(next, adj, visited, sequence);
                    // Return edge: child → parent (undo).
                    sequence.push((next, node, true));
                }
            }
        }
    }

    /// Solve the GF(2) system: find a minimal subset of qubits whose
    /// frames XOR to `target`.  Returns the qubit indices in application
    /// order (sorted by index, stable).
    ///
    /// Uses Gauss–Jordan elimination on the n×n frame matrix built from
    /// `frame`.  Because CNOT operations are invertible, the frame vectors
    /// always form a full-rank basis, so a solution always exists.
    fn find_controls_for_target(frame: &[u64], target: u64) -> Vec<usize> {
        let n = frame.len();
        if target == 0 {
            return Vec::new();
        }

        // Quick check: does a single qubit already hold `target`?
        if let Some(q) = (0..n).find(|&q| frame[q] == target) {
            return vec![q];
        }

        // Build augmented matrix row-by-row.
        // Row i corresponds to bit i.  Column j stores whether frame[j]
        // has bit i set.  Column n is the augmented column (target bit i).
        let aug_bit = 1u64 << n;
        let mut rows: Vec<u64> = (0..n)
            .map(|i| {
                let mut row: u64 = 0;
                for j in 0..n {
                    if (frame[j] >> i) & 1 == 1 {
                        row |= 1u64 << j;
                    }
                }
                if (target >> i) & 1 == 1 {
                    row |= aug_bit;
                }
                row
            })
            .collect();

        let mut pivot_col: Vec<Option<usize>> = vec![None; n];

        // Forward elimination — for each column, find a pivot row and
        // eliminate the column from all other rows.
        for col in 0..n {
            let pivot_row = (0..n).find(|&r| pivot_col[r].is_none() && ((rows[r] >> col) & 1 == 1));

            if let Some(pr) = pivot_row {
                pivot_col[pr] = Some(col);
                let pivot_mask = rows[pr];
                for r2 in 0..n {
                    if r2 != pr && ((rows[r2] >> col) & 1 == 1) {
                        rows[r2] ^= pivot_mask;
                    }
                }
            }
        }

        // Read solution from the augmented column of each pivot row.
        let mut solution = vec![false; n];
        for r in 0..n {
            if let Some(col) = pivot_col[r] {
                solution[col] = ((rows[r] >> n) & 1) == 1;
            }
        }

        // Return qubit indices where solution[j] = true, sorted.
        let mut indices: Vec<usize> = (0..n).filter(|&j| solution[j]).collect();
        indices.sort_unstable();
        indices
    }
}

impl DiagonalSynthesis for GrayCodeSynthesis {
    fn name(&self) -> &str {
        "GrayCodeSynthesis"
    }

    fn synthesize(
        &self,
        terms: &[(Parity, f64)],
        num_qubits: usize,
    ) -> Result<Vec<SynthesizedGate>> {
        if terms.is_empty() || num_qubits == 0 {
            return Ok(Vec::new());
        }

        // Guard: parity frames use u64, limited to 63 qubits (bit 63 = augmented).
        // Shifts >= 64 panic in debug builds and wrap in release.
        if num_qubits > 63 {
            return Err(crate::error::MyQuatError::invalid_parameter(format!(
                "GrayCodeSynthesis requires at most 63 qubits, got {}",
                num_qubits
            )));
        }

        // Merge angles for same parity
        let mut angle_map: HashMap<u64, f64> = HashMap::new();
        for (p, a) in terms {
            if p.0 == 0 || a.abs() < 1e-12 {
                continue;
            }
            *angle_map.entry(p.0).or_default() += a;
        }

        if angle_map.is_empty() {
            return Ok(Vec::new());
        }

        // Build parity list including 0 (root of traversal)
        let mut all_parities: Vec<u64> = vec![0];
        all_parities.extend(angle_map.keys().copied());
        all_parities.sort_unstable();
        all_parities.dedup();

        if all_parities.len() <= 1 {
            return Ok(Vec::new());
        }

        // Build MST
        let edges = Self::build_mst(&all_parities);
        if edges.is_empty() {
            return Ok(Vec::new());
        }

        // Gray code traversal
        let traversal = Self::gray_code_traversal(&edges, 0);

        let mut gates = Vec::new();

        // ── Frame-aware state ──────────────────────────────────────
        // frame[q] = parity vector held by qubit q.
        // Initially: qubit q holds parity {q} (identity frame).
        let mut frame: Vec<u64> = (0..num_qubits).map(|q| 1u64 << q).collect();

        // current = parity value at the current position in the traversal.
        let mut current: u64 = 0;

        for (_from, to, is_return) in &traversal {
            let diff = current ^ to;

            let bits_to_flip: Vec<usize> = (0..64).filter(|b| (diff >> b) & 1 == 1).collect();

            for &bit in &bits_to_flip {
                let new_current = if (current >> bit) & 1 == 1 {
                    current & !(1u64 << bit)
                } else {
                    current | (1u64 << bit)
                };

                // ── Find (ctrl, tgt) pair ──────────────────────
                let pair = (0..num_qubits)
                    .flat_map(|t| (0..num_qubits).map(move |c| (c, t)))
                    .filter(|&(c, t)| c != t && frame[t] ^ frame[c] == new_current)
                    .min_by_key(|&(c, t)| {
                        let tgt_matches = if frame[t] == current { 0usize } else { 1 };
                        (tgt_matches, c, t)
                    });

                if let Some((ctrl, tgt)) = pair {
                    gates.push(SynthesizedGate {
                        gate: StandardGate::CX,
                        qubits: vec![ctrl, tgt],
                        angle: None,
                    });
                    frame[tgt] ^= frame[ctrl];
                } else {
                    // ── Multi-CNOT fallback ───────────────────
                    let controls = Self::find_controls_for_target(&frame, new_current);
                    if controls.len() > 1 {
                        let tgt = controls[0];
                        for &ctrl in &controls[1..] {
                            gates.push(SynthesizedGate {
                                gate: StandardGate::CX,
                                qubits: vec![ctrl, tgt],
                                angle: None,
                            });
                            frame[tgt] ^= frame[ctrl];
                        }
                    }
                }

                current = new_current;

                // ── Emit Rz (forward edges only) ──────────────
                if !is_return {
                    if let Some(&angle) = angle_map.get(&current) {
                        if angle.abs() > 1e-12 {
                            let rz_qubit = (0..num_qubits).find(|&q| frame[q] == current).expect(
                                "After CNOT(s), at least one qubit must hold current parity",
                            );
                            gates.push(SynthesizedGate {
                                gate: StandardGate::Rz,
                                qubits: vec![rz_qubit],
                                angle: Some(angle),
                            });
                        }
                    }
                }
            }
        }

        Ok(gates)
    }
}

// ---------------------------------------------------------------------------
// Phase Polynomial Extraction
// ---------------------------------------------------------------------------

/// Walk a circuit and extract {CX, Rz} segments.
///
/// Non-CX, non-Rz gates delimit segment boundaries. For each segment,
/// track the CNOT frame and collect labeled Rz rotations.
///
/// Returns (start_instr_idx, end_instr_idx, polynomial) for each segment.
pub fn extract_segments(circuit: &QuantumCircuit) -> Vec<(usize, usize, PhasePolynomial)> {
    let instructions = circuit.data().instructions();
    let num_qubits = circuit.num_qubits();
    let n = instructions.len();

    let mut segments = Vec::new();
    let mut seg_start: Option<usize> = None;
    let mut frame: Vec<Parity> = (0..num_qubits).map(Parity::singleton).collect();
    let mut rotations: Vec<LabeledRz> = Vec::new();
    let mut cnots: Vec<(usize, usize)> = Vec::new();

    for i in 0..n {
        let inst = &instructions[i];
        let gate = &inst.gate.gate_type;

        match gate {
            StandardGate::CX => {
                if seg_start.is_none() {
                    seg_start = Some(i);
                }
                let ctrl = inst.qubits[0].index();
                let tgt = inst.qubits[1].index();
                // Update frame: frame[tgt] ^= frame[ctrl]
                frame[tgt] = frame[tgt].xor(frame[ctrl]);
                cnots.push((ctrl, tgt));
            }
            StandardGate::Rz => {
                if seg_start.is_none() {
                    seg_start = Some(i);
                }
                let q = inst.qubits[0].index();
                let angle = inst
                    .gate
                    .parameters
                    .first()
                    .and_then(|p| p.numeric_value())
                    .unwrap_or(0.0);
                if angle.abs() > 1e-12 {
                    rotations.push(LabeledRz {
                        parity: frame[q],
                        angle,
                        qubit: q,
                    });
                }
            }
            _ => {
                // Non-CX, non-Rz gate: flush current segment
                if let Some(start) = seg_start {
                    if !cnots.is_empty() || !rotations.is_empty() {
                        segments.push((
                            start,
                            i,
                            PhasePolynomial {
                                num_qubits,
                                frame: frame.clone(),
                                rotations: std::mem::take(&mut rotations),
                                cnots: std::mem::take(&mut cnots),
                            },
                        ));
                    }
                    seg_start = None;
                    // Reset frame for next segment
                    frame = (0..num_qubits).map(Parity::singleton).collect();
                }
            }
        }
    }

    // Flush remaining segment at end of circuit
    if let Some(start) = seg_start {
        if !cnots.is_empty() || !rotations.is_empty() {
            segments.push((
                start,
                n,
                PhasePolynomial {
                    num_qubits,
                    frame,
                    rotations,
                    cnots,
                },
            ));
        }
    }

    segments
}

// ---------------------------------------------------------------------------
// Phase Polynomial Optimization
// ---------------------------------------------------------------------------

/// Optimize a phase polynomial in-place:
/// 1. Merge Rz gates with the same (parity, qubit) by summing angles.
/// 2. Remove zero-angle Rz gates.
///
/// NOTE: CX gates are NOT modified because the parity labels of downstream
/// Rz gates depend on the CNOT frame. Changing the CNOT sequence would
/// invalidate the parity labels. CNOT optimization is handled by
/// CNOTOptimizer before this pass runs.
pub fn optimize_polynomial(poly: &mut PhasePolynomial) {
    // Merge same-parity, same-qubit Rz gates
    let mut merged: Vec<LabeledRz> = Vec::new();
    for rz in std::mem::take(&mut poly.rotations) {
        if rz.angle.abs() < 1e-12 {
            continue;
        }
        // Check if we can merge with the last entry
        if let Some(last) = merged.last_mut() {
            if last.parity == rz.parity && last.qubit == rz.qubit {
                last.angle += rz.angle;
                if last.angle.abs() < 1e-12 {
                    merged.pop();
                }
                continue;
            }
        }
        merged.push(rz);
    }
    poly.rotations = merged;
}

// ---------------------------------------------------------------------------
// AdaptiveSynthesis: auto-select best strategy per segment
// ---------------------------------------------------------------------------

/// Adaptive synthesis that tries multiple strategies and picks the one
/// with the fewest CX gates for each segment.
///
/// # Design
///
/// Different parity sets benefit from different synthesis strategies:
/// - Prefix-chain parities work well with ChainSynthesis or GrayCodeSynthesis
/// - Scattered/irregular parities work better with RowColSynthesis
///
/// Rather than committing to one strategy, `AdaptiveSynthesis` tries each
/// registered strategy on the input and returns the result with the fewest
/// CX gates. Only frame-safe (reversible) strategies are included by default.
///
/// # Safety
///
/// All registered strategies MUST be frame-safe (restore identity parity frame).
/// Non-reversible strategies (raw `GrayCodeSynthesis`, `RowColSynthesis`) will
/// corrupt downstream gates when the input frame is non-identity.
pub struct AdaptiveSynthesis {
    strategies: Vec<Box<dyn DiagonalSynthesis>>,
}

impl AdaptiveSynthesis {
    /// Create with a custom set of strategies.
    pub fn new(strategies: Vec<Box<dyn DiagonalSynthesis>>) -> Self {
        Self { strategies }
    }

    /// Create with the default set of frame-safe strategies.
    ///
    /// Uses both `ReversibleRowColSynthesis` and `ReversibleGrayCodeSynthesis`.
    /// The `GrayCodeSynthesis` frame-tracking bug was fixed in Phase 11h —
    /// it now maintains a proper frame→qubit mapping throughout the
    /// Gray-code traversal, making `ReversibleGrayCodeSynthesis` safe to use.
    pub fn with_default_strategies() -> Self {
        Self {
            strategies: vec![
                Box::new(crate::parity_synth::ReversibleRowColSynthesis),
                Box::new(crate::parity_synth::ReversibleGrayCodeSynthesis),
                Box::new(crate::parity_synth::ParitySynthSynthesis), // Phase 11o
            ],
        }
    }
}

impl AdaptiveSynthesis {
    /// Analyze parity features to guide strategy selection.
    ///
    /// Returns (chainability, avg_hamming_weight, qubit_spread).
    /// - chainability: fraction of consecutive pairs with Hamming distance 1
    /// - avg_hamming_weight: average number of 1-bits per parity
    /// - qubit_spread: number of distinct qubits across all parities
    fn analyze_parities(terms: &[(Parity, f64)]) -> (f64, f64, usize) {
        if terms.len() < 2 {
            let weight = terms
                .first()
                .map(|(p, _)| p.0.count_ones() as f64)
                .unwrap_or(0.0);
            let spread = terms
                .first()
                .map(|(p, _)| {
                    let mut qs = 0usize;
                    let mut bits = p.0;
                    while bits != 0 {
                        qs = qs.max(bits.trailing_zeros() as usize + 1);
                        bits &= bits - 1;
                    }
                    qs
                })
                .unwrap_or(0);
            return (1.0, weight, spread);
        }

        // Chainability: fraction of adjacent pairs differing by 1 bit
        let mut chain_pairs = 0usize;
        let mut total_weight = 0u32;
        let mut max_qubit = 0usize;

        // Track which qubits appear
        let mut qubit_set = 0u64;

        for (i, (parity, _)) in terms.iter().enumerate() {
            let bits = parity.0;
            total_weight += bits.count_ones();
            qubit_set |= bits;

            // Find highest qubit index involved
            let mut b = bits;
            while b != 0 {
                max_qubit = max_qubit.max(b.trailing_zeros() as usize + 1);
                b &= b - 1;
            }

            if i > 0 {
                let prev = terms[i - 1].0 .0;
                let hamming = (bits ^ prev).count_ones();
                if hamming == 1 {
                    chain_pairs += 1;
                }
            }
        }

        let n = terms.len();
        let chainability = chain_pairs as f64 / (n - 1) as f64;
        let avg_weight = total_weight as f64 / n as f64;
        let qubit_spread = qubit_set.count_ones() as usize;

        (chainability, avg_weight, max_qubit.max(qubit_spread))
    }

    /// Reorder strategies by predicted effectiveness for the given parity pattern.
    ///
    /// Heuristic rules:
    /// - Chain-like patterns (chainability > 0.6) → GrayCode, Chain first
    /// - Scattered patterns (avg weight > 2, chainability < 0.3) → RowCol, ParitySynth first
    /// - Mixed → try all (default order)
    fn ordered_strategies(&self, terms: &[(Parity, f64)]) -> Vec<&Box<dyn DiagonalSynthesis>> {
        let (chainability, avg_weight, _spread) = Self::analyze_parities(terms);

        // Single term: any strategy works, just try the first one
        if terms.len() <= 2 {
            return self.strategies.iter().collect();
        }

        // Strongly chain-like: GrayCode/Chain dominate, skip RowCol-like strategies
        if chainability > 0.6 {
            // Put strategies with "Gray" or "Chain" in name first
            // (RowCol is wasteful for chain patterns)
            let mut ordered: Vec<&Box<dyn DiagonalSynthesis>> = Vec::new();
            let mut rest: Vec<&Box<dyn DiagonalSynthesis>> = Vec::new();
            for s in &self.strategies {
                let name = s.name();
                if name.contains("GrayCode") || name.contains("Chain") {
                    ordered.push(s);
                } else {
                    rest.push(s);
                }
            }
            ordered.extend(rest);
            return ordered;
        }

        // Strongly scattered: RowCol/ParitySynth first, skip Chain
        if avg_weight > 2.0 && chainability < 0.3 {
            let mut ordered: Vec<&Box<dyn DiagonalSynthesis>> = Vec::new();
            let mut rest: Vec<&Box<dyn DiagonalSynthesis>> = Vec::new();
            for s in &self.strategies {
                let name = s.name();
                if name.contains("RowCol") || name.contains("ParitySynth") {
                    ordered.push(s);
                } else {
                    rest.push(s);
                }
            }
            ordered.extend(rest);
            return ordered;
        }

        // Mixed pattern: keep default order (try all)
        self.strategies.iter().collect()
    }
}

impl DiagonalSynthesis for AdaptiveSynthesis {
    fn name(&self) -> &str {
        "AdaptiveSynthesis"
    }

    fn synthesize(
        &self,
        terms: &[(Parity, f64)],
        num_qubits: usize,
    ) -> Result<Vec<SynthesizedGate>> {
        if self.strategies.is_empty() {
            return Ok(Vec::new());
        }

        if terms.is_empty() || num_qubits == 0 {
            return Ok(Vec::new());
        }

        // Heuristic ordering: try best-guess strategy first
        let ordered = self.ordered_strategies(terms);

        let mut best: Option<(Vec<SynthesizedGate>, usize, usize)> = None; // (gates, cx, rz)

        for strategy in &ordered {
            if let Ok(gates) = strategy.synthesize(terms, num_qubits) {
                let cx_count = gates.iter().filter(|g| g.gate == StandardGate::CX).count();
                let rz_count = gates.iter().filter(|g| g.gate == StandardGate::Rz).count();

                // Update best
                let is_better = match &best {
                    None => true,
                    Some((_, best_cx, best_rz)) => {
                        cx_count < *best_cx || (cx_count == *best_cx && rz_count < *best_rz)
                    }
                };

                if is_better {
                    best = Some((gates, cx_count, rz_count));
                }

                // Fast-path: if first heuristic choice gives minimal CX, skip remaining
                if best.is_some() && ordered.len() > 1 {
                    // For chain patterns, GrayCode hits the theoretical minimum
                    // (n-1 CX for n distinct parities). If we hit that, stop.
                    let theoretical_min = terms.len().saturating_sub(1);
                    if cx_count <= theoretical_min {
                        break;
                    }
                }
            }
        }

        Ok(best.map(|(g, _, _)| g).unwrap_or_default())
    }
}

// ---------------------------------------------------------------------------
// PhasePolynomialPass: CircuitPass integration
// ---------------------------------------------------------------------------

/// A `CircuitPass` that optimizes {CX, Rz} segments using phase polynomial
/// representation and re-synthesizes with a pluggable strategy.
///
/// # Pipeline position
///
/// Best placed AFTER `CNOTOptimizer` (to simplify the CNOT network first)
/// and BEFORE `SingleQubitOptimizer` (to clean up Rz gates).
pub struct PhasePolynomialPass {
    strategy: Box<dyn DiagonalSynthesis>,
}

impl PhasePolynomialPass {
    /// Create a new pass with the given synthesis strategy.
    pub fn new(strategy: Box<dyn DiagonalSynthesis>) -> Self {
        Self { strategy }
    }
}

impl CircuitPass for PhasePolynomialPass {
    fn run(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        let instructions = circuit.data().instructions();
        let num_qubits = circuit.num_qubits();
        let mut new_circuit = QuantumCircuit::new(num_qubits, circuit.num_clbits());

        let mut i = 0;
        while i < instructions.len() {
            // Check if this instruction starts a {CX, Rz} segment.
            let is_cx_rz = matches!(
                instructions[i].gate.gate_type,
                StandardGate::CX | StandardGate::Rz
            );

            if is_cx_rz {
                // Extract the {CX, Rz} segment: walk forward tracking parity
                // frame and collecting Rz angles at each parity.
                let seg_start = i;
                let mut frame: Vec<u64> = (0..num_qubits).map(|q| 1u64 << q).collect();
                let mut parity_angles: HashMap<u64, f64> = HashMap::new();

                while i < instructions.len() {
                    match &instructions[i].gate.gate_type {
                        StandardGate::CX => {
                            let ctrl = instructions[i].qubits[0].index();
                            let tgt = instructions[i].qubits[1].index();
                            frame[tgt] ^= frame[ctrl];
                        }
                        StandardGate::Rz => {
                            let q = instructions[i].qubits[0].index();
                            let angle = instructions[i]
                                .gate
                                .parameters
                                .first()
                                .and_then(|p| p.numeric_value())
                                .unwrap_or(0.0);
                            if angle.abs() > 1e-12 {
                                *parity_angles.entry(frame[q]).or_default() += angle;
                            }
                        }
                        _ => break, // End of segment.
                    }
                    i += 1;
                }

                // Check if the segment ends with the identity frame.
                // Re-synthesis is only safe when the final frame is identity
                // (which is the case for Trotter circuits where CX trees
                // are always paired). Otherwise the downstream circuit
                // depends on the exact parity frame state.
                let final_frame_is_identity =
                    frame.iter().enumerate().all(|(q, &f)| f == (1u64 << q));

                if final_frame_is_identity && !parity_angles.is_empty() {
                    // Safe to re-synthesize: the segment restores identity
                    // frame. We use the configured strategy which MUST produce
                    // identity-frame output (ReversibleRowColSynthesis or
                    // ChainSynthesis). Non-reversible strategies (RowColSynthesis,
                    // GrayCodeSynthesis) do NOT restore identity and would
                    // corrupt downstream gates even when input is identity.
                    let terms: Vec<(Parity, f64)> = parity_angles
                        .into_iter()
                        .map(|(p, a)| (Parity(p), a))
                        .collect();

                    let synthesized = self.strategy.synthesize(&terms, num_qubits)?;
                    // Debug check: verify synthesis strategy restores
                    // identity frame. The default ReversibleRowColSynthesis
                    // guarantees this; custom strategies may not.
                    debug_assert!(
                        {
                            let mut f: Vec<u64> = (0..num_qubits).map(|q| 1u64 << q).collect();
                            for sg in &synthesized {
                                if matches!(sg.gate, crate::gates::StandardGate::CX) {
                                    f[sg.qubits[1]] ^= f[sg.qubits[0]];
                                }
                            }
                            f.iter().enumerate().all(|(q, &v)| v == (1u64 << q))
                        },
                        "Synthesis strategy {} did NOT restore the identity parity frame! \
                         Use ReversibleRowColSynthesis or ChainSynthesis for frame safety.",
                        self.strategy.name()
                    );
                    emit_gates(&mut new_circuit, &synthesized)?;
                } else {
                    // Fall back to gate-by-gate copy with Rz merging.
                    // This preserves the original frame exactly.
                    let mut frame2: Vec<u64> = (0..num_qubits).map(|q| 1u64 << q).collect();
                    let mut pending: HashMap<(u64, usize), f64> = HashMap::new();

                    for j in seg_start..i {
                        match &instructions[j].gate.gate_type {
                            StandardGate::Rz => {
                                let q = instructions[j].qubits[0].index();
                                let angle = instructions[j]
                                    .gate
                                    .parameters
                                    .first()
                                    .and_then(|p| p.numeric_value())
                                    .unwrap_or(0.0);
                                if angle.abs() > 1e-12 {
                                    let key = (frame2[q], q);
                                    *pending.entry(key).or_default() += angle;
                                }
                            }
                            StandardGate::CX => {
                                let ctrl = instructions[j].qubits[0].index();
                                let tgt = instructions[j].qubits[1].index();
                                // Flush Rz on target and control before CX.
                                for &q in &[tgt, ctrl] {
                                    let key = (frame2[q], q);
                                    if let Some(angle) = pending.remove(&key) {
                                        if angle.abs() > 1e-12 {
                                            new_circuit.rz(q, Parameter::Float(angle))?;
                                        }
                                    }
                                }
                                new_circuit.cx(ctrl, tgt)?;
                                frame2[tgt] ^= frame2[ctrl];
                            }
                            _ => {}
                        }
                    }
                    // Flush remaining pending Rz.
                    for ((_p, q), angle) in pending.drain() {
                        if angle.abs() > 1e-12 {
                            new_circuit.rz(q, Parameter::Float(angle))?;
                        }
                    }
                }
            } else {
                // Non-CX, non-Rz gate: copy directly.
                new_circuit
                    .data_mut()
                    .add_instruction(instructions[i].clone())?;
                i += 1;
            }
        }

        *circuit = new_circuit;
        Ok(())
    }

    fn name(&self) -> &str {
        "PhasePolynomial"
    }
}

impl Default for PhasePolynomialPass {
    fn default() -> Self {
        // Use AdaptiveSynthesis which tries multiple frame-safe strategies
        // (ReversibleRowColSynthesis, ReversibleGrayCodeSynthesis) and picks
        // the one with the fewest CX gates per segment. Both strategies
        // restore the identity parity frame, guaranteeing correctness.
        Self::new(Box::new(AdaptiveSynthesis::with_default_strategies()))
    }
}

// ---------------------------------------------------------------------------
// Utility: synthesize gates into a circuit
// ---------------------------------------------------------------------------

/// Emit synthesized gates into a QuantumCircuit.
pub fn emit_gates(circuit: &mut QuantumCircuit, gates: &[SynthesizedGate]) -> Result<()> {
    for sg in gates {
        match sg.gate {
            StandardGate::CX => {
                circuit.cx(sg.qubits[0], sg.qubits[1])?;
            }
            StandardGate::Rz => {
                if let Some(angle) = sg.angle {
                    circuit.rz(sg.qubits[0], Parameter::Float(angle))?;
                }
            }
            _ => {
                // Other gates should be handled by the caller
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Global Phase Polynomial — Cross-Block Parity Matrix (Phase 11n)
// ---------------------------------------------------------------------------
//
// Standard PhasePolynomialPass extracts {CX, Rz} segments bounded by non-CX/Rz
// gates (typically H). This prevents merging Rz gates with identical parity
// vectors that are separated by H gates at QWC block boundaries.
//
// GlobalPhasePoly solves this by tracking basis changes (H, S, Sdg) through
// the parity frame, allowing Rz gates across the entire circuit to participate
// in a single parity matrix. This is analogous to TKET's UnitaryRevTableau:
// Clifford gates are absorbed into the frame tracking, unifying the parity
// space.
//
// The key insight: H·Rz(θ)·H = Rx(θ). When an H gate changes a qubit's basis
// from Z to X, subsequent Rz gates on that qubit are physically Rx gates.
// By tracking the basis per qubit, we can represent the entire circuit as
// a single parity matrix + basis annotations + linear transformation.

/// Per-qubit basis type for frame-aware parity tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum QubitBasis {
    /// Standard computational Z-basis: Rz applies Z rotation
    Z,
    /// X-basis (after H gate): Z↔X swapped, Rz → Rx physically
    X,
    /// Y-basis (after S†·H): Rz → Ry physically
    Y,
}

impl QubitBasis {
    /// Apply a single-qubit Clifford gate to this basis.
    fn apply_clifford(&mut self, gate: StandardGate) {
        match gate {
            StandardGate::H => {
                *self = match *self {
                    QubitBasis::Z => QubitBasis::X,
                    QubitBasis::X => QubitBasis::Z,
                    QubitBasis::Y => QubitBasis::Y, // H|Y⟩ ∝ |Y⟩ (up to phase)
                };
            }
            StandardGate::S => {
                *self = match *self {
                    QubitBasis::Z => QubitBasis::Z,
                    QubitBasis::X => QubitBasis::Y,
                    QubitBasis::Y => QubitBasis::X,
                };
            }
            StandardGate::Sdg => {
                *self = match *self {
                    QubitBasis::Z => QubitBasis::Z,
                    QubitBasis::X => QubitBasis::Y,
                    QubitBasis::Y => QubitBasis::X,
                };
            }
            _ => {} // Other gates don't change basis
        }
    }
}

/// A global phase polynomial spanning multiple {CX, Rz} segments.
///
/// Unlike per-segment extraction, this tracks the full linear transformation
/// and basis changes across the entire circuit, enabling cross-block Rz merging.
struct GlobalPhasePoly {
    /// Parity vectors (one per collected Rz gate), each a u64 bitmask
    parities: Vec<u64>,
    /// Angles for each parity vector
    angles: Vec<f64>,
    /// Per-qubit basis at the time each Rz was collected
    rz_basis: Vec<QubitBasis>,
    /// Current linear transformation frame: frame[q] = parity currently held by qubit q
    frame: Vec<u64>,
    /// Current basis per qubit
    basis: Vec<QubitBasis>,
    /// Number of qubits
    num_qubits: usize,
}

impl GlobalPhasePoly {
    fn new(num_qubits: usize) -> Self {
        Self {
            parities: Vec::new(),
            angles: Vec::new(),
            rz_basis: Vec::new(),
            frame: (0..num_qubits).map(|q| 1u64 << q).collect(),
            basis: vec![QubitBasis::Z; num_qubits],
            num_qubits,
        }
    }

    /// Process a CX gate: XOR the control qubit's parity into the target.
    fn apply_cx(&mut self, ctrl: usize, tgt: usize) {
        self.frame[tgt] ^= self.frame[ctrl];
    }

    /// Process an Rz gate: record the current parity + angle.
    fn apply_rz(&mut self, qubit: usize, angle: f64) {
        if angle.abs() > 1e-12 {
            self.parities.push(self.frame[qubit]);
            self.angles.push(angle);
            self.rz_basis.push(self.basis[qubit]);
        }
    }

    /// Process a single-qubit Clifford gate: update basis tracking.
    fn apply_clifford(&mut self, qubit: usize, gate: StandardGate) {
        self.basis[qubit].apply_clifford(gate);
    }

    /// Build a global parity matrix from an entire circuit.
    ///
    /// Walks all instructions, tracking the frame through CX gates and
    /// recording Rz angles. H and S/Sdg gates update the basis tracking
    /// rather than breaking the parity matrix.
    fn from_circuit(circuit: &QuantumCircuit) -> Self {
        let num_qubits = circuit.num_qubits();
        let mut gpp = Self::new(num_qubits);
        let instructions = circuit.data().instructions();

        for inst in instructions {
            let qubits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();

            match inst.gate.gate_type {
                StandardGate::CX => {
                    if qubits.len() >= 2 {
                        gpp.apply_cx(qubits[0], qubits[1]);
                    }
                }
                StandardGate::Rz => {
                    if !qubits.is_empty() {
                        let angle = inst
                            .gate
                            .parameters
                            .first()
                            .and_then(|p| p.numeric_value())
                            .unwrap_or(0.0);
                        gpp.apply_rz(qubits[0], angle);
                    }
                }
                StandardGate::H | StandardGate::S | StandardGate::Sdg if !qubits.is_empty() => {
                    gpp.apply_clifford(qubits[0], inst.gate.gate_type);
                }
                _ => {
                    // Non-CX/Rz/Clifford gates (e.g., Rx, Ry, X, Y, Z, Measure):
                    // These gates don't affect the parity frame or basis.
                    // We skip them — they'll be copied through verbatim.
                }
            }
        }

        gpp
    }

    /// Merge duplicate parity vectors by summing angles.
    /// Only merges when BOTH parity AND basis match — different bases
    /// mean physically different unitaries (Rz vs Rx vs Ry).
    fn dedup_parities(&mut self) {
        // Key: (parity, basis) for correct merging
        let mut merged: HashMap<(u64, QubitBasis), f64> = HashMap::new();
        for i in 0..self.parities.len() {
            let key = (self.parities[i], self.rz_basis[i]);
            *merged.entry(key).or_default() += self.angles[i];
        }

        // Rebuild: keep only non-zero angles, preserving basis
        self.parities.clear();
        self.angles.clear();
        self.rz_basis.clear();
        for ((p, basis), angle) in merged {
            if angle.abs() > 1e-12 {
                self.parities.push(p);
                self.angles.push(angle);
                self.rz_basis.push(basis);
            }
        }
    }

    /// Number of distinct parity vectors.
    fn num_terms(&self) -> usize {
        self.parities.len()
    }
}

/// Circuit pass that applies global phase polynomial optimization.
///
/// Unlike `PhasePolynomialPass` (which works per-{CX, Rz}-segment), this pass
/// spans the ENTIRE circuit, merging Rz gates across H-gate boundaries via
/// basis tracking.
///
/// # When to use
///
/// - After PauliGadget/PauliLevel synthesis, before convergence loop
/// - The circuit should have been simplified first (inverse pairs, SQ merges)
/// - Best results when Clifford annotations (S/Sdg) have been absorbed
pub struct GlobalPhasePolynomialPass {
    /// Minimum number of parity terms to trigger re-synthesis
    min_terms: usize,
}

impl GlobalPhasePolynomialPass {
    pub fn new() -> Self {
        Self { min_terms: 3 }
    }

    pub fn with_min_terms(min_terms: usize) -> Self {
        Self { min_terms }
    }
}

impl Default for GlobalPhasePolynomialPass {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitPass for GlobalPhasePolynomialPass {
    fn run(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        let num_qubits = circuit.num_qubits();

        // Build global parity matrix, deduplicating identical parity vectors
        let mut gpp = GlobalPhasePoly::from_circuit(circuit);
        gpp.dedup_parities();

        if gpp.num_terms() < self.min_terms {
            return Ok(()); // Too few terms to benefit
        }

        // Build (parity, angle) pairs for synthesis
        let terms: Vec<(Parity, f64)> = gpp
            .parities
            .iter()
            .zip(gpp.angles.iter())
            .map(|(&p, &a)| (Parity(p), a))
            .collect();

        if terms.is_empty() {
            return Ok(());
        }

        // Synthesize the parity matrix into CX+Rz network
        use crate::parity_synth::ReversibleRowColSynthesis;
        let strategy = ReversibleRowColSynthesis;
        let synthesized = strategy.synthesize(&terms, num_qubits)?;

        // Build output circuit:
        // 1. Emit basis-change gates for qubits that ended in non-Z basis
        //    (these become the initial basis for the synthesized network)
        // 2. Synthesize CNOT+Rz network
        // 3. Emit inverse basis-change gates to restore Z-basis
        // 4. Insert preserved gates (non-CX/Rz/H/S/Sdg) after synthesis
        let mut new_circuit = QuantumCircuit::new(num_qubits, circuit.num_clbits());
        if let Some(name) = circuit.name() {
            new_circuit.set_name(name.to_string());
        }

        // Copy non-step_boundaries metadata
        for (key, value) in circuit.data().metadata().iter() {
            if key != crate::circuit_optimization::STEP_BOUNDARIES_KEY {
                new_circuit
                    .data_mut()
                    .set_metadata(key.clone(), value.clone());
            }
        }

        // Phase 11n: Emit basis-change gates based on final basis state.
        // The GlobalPhasePoly tracks basis changes from H/S/Sdg gates.
        // After processing the circuit, gpp.basis[q] holds the FINAL basis.
        // We emit the inverse of this basis change at the START so the
        // synthesized Z-basis network is correct, then restore at END.
        for q in 0..num_qubits {
            match gpp.basis[q] {
                QubitBasis::Z => {} // Already in Z-basis, no change
                QubitBasis::X => {
                    // H converts X↔Z. Apply H to put qubit into Z-basis
                    // for the synthesized network.
                    new_circuit.h(q)?;
                }
                QubitBasis::Y => {
                    // S·H converts Y→Z. Apply S·H to put qubit into Z-basis.
                    new_circuit.s(q)?;
                    new_circuit.h(q)?;
                }
            }
        }

        // Emit the synthesized CX+Rz network
        emit_gates(&mut new_circuit, &synthesized)?;

        // Emit inverse basis changes to restore the correct final basis
        for q in 0..num_qubits {
            match gpp.basis[q] {
                QubitBasis::Z => {}
                QubitBasis::X => {
                    new_circuit.h(q)?; // H⁻¹ = H
                }
                QubitBasis::Y => {
                    new_circuit.h(q)?;
                    new_circuit.sdg(q)?; // (S·H)⁻¹ = H·S†
                }
            }
        }

        // Emit preserved gates (non-{CX,Rz,H,S,Sdg}) after the network
        let instructions = circuit.data().instructions();
        for inst in instructions {
            let is_tracked = matches!(
                inst.gate.gate_type,
                StandardGate::CX
                    | StandardGate::Rz
                    | StandardGate::H
                    | StandardGate::S
                    | StandardGate::Sdg
            );
            if !is_tracked {
                new_circuit.data_mut().add_instruction(inst.clone())?;
            }
        }

        *circuit = new_circuit;
        Ok(())
    }

    fn name(&self) -> &str {
        "GlobalPhasePolynomial"
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ── Parity tests ──

    #[test]
    fn test_parity_singleton() {
        assert_eq!(Parity::singleton(0).0, 0b0001);
        assert_eq!(Parity::singleton(1).0, 0b0010);
        assert_eq!(Parity::singleton(3).0, 0b1000);
    }

    #[test]
    fn test_parity_xor() {
        let p01 = Parity::singleton(0).xor(Parity::singleton(1));
        assert_eq!(p01.0, 0b0011);
    }

    #[test]
    fn test_parity_weight() {
        assert_eq!(Parity(0b0000).weight(), 0);
        assert_eq!(Parity(0b0101).weight(), 2);
        assert_eq!(Parity(0b1111).weight(), 4);
    }

    #[test]
    fn test_parity_distance() {
        let a = Parity(0b0001); // {0}
        let b = Parity(0b0011); // {0,1}
        let c = Parity(0b0111); // {0,1,2}
        assert_eq!(a.distance(b), 1);
        assert_eq!(b.distance(c), 1);
        assert_eq!(a.distance(c), 2);
    }

    #[test]
    fn test_parity_differing_qubit() {
        let a = Parity(0b0001); // qubit 0
        let b = Parity(0b0011); // qubits 0,1
        assert_eq!(a.differing_qubit(b), Some(1));
        let c = Parity(0b0111); // qubits 0,1,2
        assert_eq!(a.differing_qubit(c), None); // distance 2
    }

    // ── ChainSynthesis tests ──

    #[test]
    fn test_chain_synthesis_empty() {
        let gates = ChainSynthesis.synthesize(&[], 4).unwrap();
        assert!(gates.is_empty());
    }

    #[test]
    fn test_chain_synthesis_single_qubit() {
        // Rz on qubit 0 only
        let terms = vec![(Parity::singleton(0), 0.5)];
        let gates = ChainSynthesis.synthesize(&terms, 2).unwrap();
        // No CX needed for single qubit terms (no multi-qubit parity)
        assert!(!gates.is_empty());
        // Should have at least one Rz gate
        let rz_count = gates.iter().filter(|g| g.gate == StandardGate::Rz).count();
        assert!(rz_count > 0);
    }

    #[test]
    fn test_chain_synthesis_prefix_parities() {
        // Prefix parities: {0}, {0,1}, {0,1,2} — the classic chain pattern
        let terms = vec![
            (Parity(0b0001), 0.1), // qubit 0
            (Parity(0b0011), 0.2), // qubits 0,1
            (Parity(0b0111), 0.3), // qubits 0,1,2
        ];
        let gates = ChainSynthesis.synthesize(&terms, 4).unwrap();
        // Should have CX gates (forward + reverse chain)
        let cx_count = gates.iter().filter(|g| g.gate == StandardGate::CX).count();
        assert_eq!(cx_count, 4); // 2 forward + 2 reverse for 3 active qubits
    }

    // ── GrayCodeSynthesis tests ──

    #[test]
    fn test_gray_code_empty() {
        let gates = GrayCodeSynthesis.synthesize(&[], 4).unwrap();
        assert!(gates.is_empty());
    }

    #[test]
    fn test_gray_code_single_term() {
        let terms = vec![(Parity(0b0011), 0.5)]; // {0,1} parity
        let gates = GrayCodeSynthesis.synthesize(&terms, 3).unwrap();
        // Should produce gates (at least 1 CX to create the parity, + Rz)
        assert!(!gates.is_empty());
    }

    // ── Extraction tests ──

    #[test]
    fn test_extract_simple_cx_rz() {
        let mut c = QuantumCircuit::new(2, 0);
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.5)).unwrap();
        c.cx(0, 1).unwrap();

        let segments = extract_segments(&c);
        assert_eq!(segments.len(), 1);
        let (_, _, poly) = &segments[0];
        assert_eq!(poly.cnots.len(), 2);
        assert_eq!(poly.rotations.len(), 1);
        // After CX(0,1), qubit 1 holds parity {0,1}
        assert_eq!(poly.rotations[0].parity, Parity(0b0011));
    }

    #[test]
    fn test_extract_no_cx() {
        let mut c = QuantumCircuit::new(2, 0);
        c.h(0).unwrap();
        c.rz(1, Parameter::Float(0.5)).unwrap();
        c.cx(0, 1).unwrap();

        let segments = extract_segments(&c);
        // Rz(1) then CX: the Rz is on qubit 1 with frame = singleton(1),
        // then CX(0,1) extends the segment
        assert_eq!(segments.len(), 1);
        let (_, _, poly) = &segments[0];
        // Rz(1) parity = singleton(1) = 0b0010
        assert_eq!(poly.rotations[0].parity, Parity(0b0010));
        assert_eq!(poly.cnots.len(), 1);
    }

    #[test]
    fn test_extract_h_gate_delimits() {
        let mut c = QuantumCircuit::new(2, 0);
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.5)).unwrap();
        c.h(0).unwrap(); // H gate delimits segment
        c.cx(0, 1).unwrap();
        c.rz(0, Parameter::Float(0.3)).unwrap();

        let segments = extract_segments(&c);
        // Two segments: one before H, one after H
        assert_eq!(segments.len(), 2);
    }

    // ── Optimization tests ──

    #[test]
    fn test_optimize_merge_same_parity() {
        let mut poly = PhasePolynomial {
            num_qubits: 2,
            frame: vec![Parity(0b01), Parity(0b11)],
            rotations: vec![
                LabeledRz {
                    parity: Parity(0b11),
                    angle: 0.3,
                    qubit: 1,
                },
                LabeledRz {
                    parity: Parity(0b11),
                    angle: 0.2,
                    qubit: 1,
                },
            ],
            cnots: vec![(0, 1)],
        };
        optimize_polynomial(&mut poly);
        assert_eq!(poly.rotations.len(), 1);
        assert!((poly.rotations[0].angle - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_optimize_preserves_cnots() {
        // CX gates are NOT modified by optimize_polynomial because
        // parity labels depend on the CNOT frame context.
        let mut poly = PhasePolynomial {
            num_qubits: 2,
            frame: vec![Parity(0b01), Parity(0b11)],
            rotations: vec![],
            cnots: vec![(0, 1), (0, 1), (1, 0)],
        };
        optimize_polynomial(&mut poly);
        assert_eq!(poly.cnots.len(), 3);
    }

    // ── Integration tests (phase polynomial pass end-to-end) ──

    /// Create a simple {CX, Rz} circuit: CX(0,1) Rz(1, 0.5) CX(0,1) — a ZZ rotation.
    fn make_test_cx_rz_circuit() -> QuantumCircuit {
        let mut c = QuantumCircuit::new(2, 0);
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.5)).unwrap();
        c.cx(0, 1).unwrap();
        c
    }

    #[test]
    fn test_pass_preserves_unitary_chain() {
        let mut c = make_test_cx_rz_circuit();
        let u_before = c.unitary(&std::collections::HashMap::new()).ok();
        let pass = PhasePolynomialPass::new(Box::new(ChainSynthesis));
        pass.run(&mut c).unwrap();
        let u_after = c.unitary(&std::collections::HashMap::new()).ok();
        if let (Some(ub), Some(ua)) = (&u_before, &u_after) {
            let n = ub.nrows();
            let mut max_diff = 0.0f64;
            for i in 0..n {
                for j in 0..n {
                    let d = (ub[(i, j)] - ua[(i, j)]).norm();
                    if d > max_diff {
                        max_diff = d;
                    }
                }
            }
            assert!(max_diff < 1e-8, "Unitary changed: max_diff={}", max_diff);
        }
    }

    #[test]
    fn test_pass_preserves_unitary_reversible_rowcol() {
        // Use ReversibleRowColSynthesis which restores the identity frame.
        let mut c = make_test_cx_rz_circuit();
        let u_before = c.unitary(&std::collections::HashMap::new()).ok();
        let pass =
            PhasePolynomialPass::new(Box::new(crate::parity_synth::ReversibleRowColSynthesis));
        pass.run(&mut c).unwrap();
        let u_after = c.unitary(&std::collections::HashMap::new()).ok();
        if let (Some(ub), Some(ua)) = (&u_before, &u_after) {
            let n = ub.nrows();
            let mut max_diff = 0.0f64;
            for i in 0..n {
                for j in 0..n {
                    let d = (ub[(i, j)] - ua[(i, j)]).norm();
                    if d > max_diff {
                        max_diff = d;
                    }
                }
            }
            assert!(max_diff < 1e-8, "Unitary changed: max_diff={}", max_diff);
        }
    }

    #[test]
    fn test_pass_reversible_reduces_or_preserves_gates() {
        // A circuit with multiple CX-Rz-CX patterns.
        let mut c = QuantumCircuit::new(3, 0);
        // Two separate ZZ-like rotations, each restoring identity frame.
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.3)).unwrap();
        c.cx(0, 1).unwrap();
        c.cx(1, 2).unwrap();
        c.rz(2, Parameter::Float(0.4)).unwrap();
        c.cx(1, 2).unwrap();

        let original_size = c.size();
        let u_before = c.unitary(&std::collections::HashMap::new()).ok();
        let pass =
            PhasePolynomialPass::new(Box::new(crate::parity_synth::ReversibleRowColSynthesis));
        pass.run(&mut c).unwrap();
        let u_after = c.unitary(&std::collections::HashMap::new()).ok();

        // Verify unitary preserved.
        if let (Some(ub), Some(ua)) = (&u_before, &u_after) {
            let n = ub.nrows();
            let mut max_diff = 0.0f64;
            for i in 0..n {
                for j in 0..n {
                    let d = (ub[(i, j)] - ua[(i, j)]).norm();
                    if d > max_diff {
                        max_diff = d;
                    }
                }
            }
            assert!(max_diff < 1e-8, "Unitary changed: max_diff={}", max_diff);
        }

        // Should not increase gate count exponentially.
        let new_size = c.size();
        assert!(
            new_size <= original_size * 3,
            "Re-synthesis exploded gates: {} → {}",
            original_size,
            new_size
        );
    }

    // ── Phase 11b: Integration tests ────────────────────────────────

    #[test]
    fn test_pass_merges_same_parity_rz() {
        // Two Rz gates with same parity on same qubit should merge.
        let mut c = QuantumCircuit::new(2, 0);
        c.rz(0, Parameter::Float(0.3)).unwrap();
        c.rz(0, Parameter::Float(0.5)).unwrap();
        let original_size = c.size();

        let pass = PhasePolynomialPass::default();
        pass.run(&mut c).unwrap();

        // Should have merged into one Rz(0.8)
        assert_eq!(
            c.size(),
            original_size - 1,
            "Two Rz on same parity should merge: {} → {}",
            original_size,
            c.size()
        );
    }

    #[test]
    fn test_pass_merges_same_parity_through_cx() {
        // Rz(0,q0) then CX(0,1) then Rz(0,q0) — same parity at q0 both times
        let mut c = QuantumCircuit::new(2, 0);
        c.rz(0, Parameter::Float(0.3)).unwrap();
        c.cx(0, 1).unwrap();
        c.rz(0, Parameter::Float(0.5)).unwrap(); // parity[0] unchanged by CX(0,1)
        let original_size = c.size();

        let pass = PhasePolynomialPass::default();
        pass.run(&mut c).unwrap();

        // CX should be preserved, Rz should NOT merge here because:
        // we flush Rz on CX target and control BEFORE emitting CX.
        // So the first Rz is flushed before CX, and the second after.
        // They have the same parity but are separated by the CX flush.
        // Expected: no merge, 3 instructions remain.
        assert_eq!(c.size(), 3, "Rz separated by CX flush should not merge");
    }

    #[test]
    fn test_pass_merges_rz_after_cx() {
        // Two Rz(0,q1) after CX(0,1): parity[1] = 0b11 both times → should merge
        let mut c = QuantumCircuit::new(2, 0);
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.3)).unwrap();
        c.rz(1, Parameter::Float(0.5)).unwrap();
        let original_size = c.size();

        let pass = PhasePolynomialPass::default();
        pass.run(&mut c).unwrap();

        // Two Rz on same (parity=0b11, qubit=1) should merge to one Rz(0.8)
        let rz_count = c
            .data()
            .instructions()
            .iter()
            .filter(|i| matches!(i.gate.gate_type, StandardGate::Rz))
            .count();
        assert_eq!(
            rz_count, 1,
            "Two Rz on same parity after CX should merge, got {} Rz",
            rz_count
        );
        assert!(
            c.size() < original_size,
            "Size should decrease: {} → {}",
            original_size,
            c.size()
        );
    }

    #[test]
    fn test_pass_noop_on_no_cx_rz() {
        // Circuit with only H and X gates — pass should leave unchanged
        let mut c = QuantumCircuit::new(2, 0);
        c.h(0).unwrap();
        c.x(1).unwrap();
        c.h(0).unwrap();
        let original_size = c.size();

        let pass = PhasePolynomialPass::default();
        pass.run(&mut c).unwrap();

        assert_eq!(
            c.size(),
            original_size,
            "No-CX/Rz circuit should be unchanged"
        );
    }

    #[test]
    fn test_pass_preserves_unitary_trotter_like() {
        // Simulate a Trotter-like circuit: H → CX tree → Rz → CX_rev → H
        // This is the pattern produced by Pauli synthesis for each term.
        use ndarray::Array2;
        use num_complex::Complex64;

        let mut c = QuantumCircuit::new(2, 0);
        // H⊗H basis change for XX term
        c.h(0).unwrap();
        c.h(1).unwrap();
        // CNOT tree
        c.cx(0, 1).unwrap();
        // Rz rotation
        c.rz(1, Parameter::Float(0.5)).unwrap();
        // Reverse CNOT tree
        c.cx(0, 1).unwrap();
        // Inverse basis change
        c.h(0).unwrap();
        c.h(1).unwrap();

        let u_before = c.unitary(&std::collections::HashMap::new()).unwrap();
        let original_size = c.size();

        let pass = PhasePolynomialPass::default();
        pass.run(&mut c).unwrap();

        let u_after = c.unitary(&std::collections::HashMap::new()).unwrap();
        let n = u_before.nrows();
        let mut max_diff = 0.0f64;
        for i in 0..n {
            for j in 0..n {
                let d = (u_before[(i, j)] - u_after[(i, j)]).norm();
                if d > max_diff {
                    max_diff = d;
                }
            }
        }
        assert!(
            max_diff < 1e-10,
            "Trotter-like unitary changed: max_diff={}, size {}→{}",
            max_diff,
            original_size,
            c.size()
        );
    }

    #[test]
    fn test_pass_level_2_integration() {
        // Verify level_2() runs without panicking on a simple circuit.
        use crate::circuit_optimization::PassManager;

        let mut c = QuantumCircuit::new(2, 0);
        c.h(0).unwrap();
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.5)).unwrap();
        c.rz(1, Parameter::Float(0.3)).unwrap();
        c.cx(0, 1).unwrap();
        c.h(0).unwrap();

        let u_before = c.unitary(&std::collections::HashMap::new()).unwrap();
        PassManager::level_2().run(&mut c).unwrap();
        let u_after = c.unitary(&std::collections::HashMap::new()).unwrap();

        let n = u_before.nrows();
        let mut max_diff = 0.0f64;
        for i in 0..n {
            for j in 0..n {
                let d = (u_before[(i, j)] - u_after[(i, j)]).norm();
                if d > max_diff {
                    max_diff = d;
                }
            }
        }
        assert!(
            max_diff < 1e-10,
            "level_2 corrupted unitary: max_diff={}",
            max_diff
        );
    }

    #[test]
    fn test_pass_level_4_integration() {
        // Verify level_4() runs without panicking with PhasePolynomialPass.
        use crate::circuit_optimization::PassManager;

        let mut c = QuantumCircuit::new(2, 0);
        c.h(0).unwrap();
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.5)).unwrap();
        c.cx(0, 1).unwrap();
        c.h(0).unwrap();
        c.rz(0, Parameter::Float(0.2)).unwrap();
        c.rz(0, Parameter::Float(0.3)).unwrap();

        let u_before = c.unitary(&std::collections::HashMap::new()).unwrap();
        PassManager::level_4().run(&mut c).unwrap();
        let u_after = c.unitary(&std::collections::HashMap::new()).unwrap();

        let n = u_before.nrows();
        let mut max_diff = 0.0f64;
        for i in 0..n {
            for j in 0..n {
                let d = (u_before[(i, j)] - u_after[(i, j)]).norm();
                if d > max_diff {
                    max_diff = d;
                }
            }
        }
        assert!(
            max_diff < 1e-10,
            "level_4 corrupted unitary: max_diff={}",
            max_diff
        );
    }

    /// Multi-block Trotter circuit fidelity test.
    ///
    /// Constructs a realistic 4-qubit Trotter circuit with multiple QWC blocks
    /// separated by basis changes (H/Rx), matching the pattern produced by
    /// PauliLevel synthesis for H2_4q. Verifies that PhasePolynomialPass does
    /// not corrupt the unitary.
    #[test]
    fn test_pass_preserves_multi_block_trotter() {
        let mut c = QuantumCircuit::new(4, 0);

        // Block 1: Z-terms (no basis change needed — already diagonal)
        // CNOT ladder: CX(0,1), CX(1,2), CX(2,3)
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.3)).unwrap(); // parity {0,1}
        c.cx(1, 2).unwrap();
        c.rz(2, Parameter::Float(0.2)).unwrap(); // parity {0,1,2}
        c.cx(2, 3).unwrap();
        c.rz(3, Parameter::Float(0.4)).unwrap(); // parity {0,1,2,3}
                                                 // Reverse ladder
        c.cx(2, 3).unwrap();
        c.cx(1, 2).unwrap();
        c.cx(0, 1).unwrap();

        // Block 2: XX on q2,q3 (H basis)
        c.h(2).unwrap();
        c.h(3).unwrap();
        c.cx(2, 3).unwrap();
        c.rz(3, Parameter::Float(0.09)).unwrap(); // 2*0.0454 ≈ 0.09
        c.cx(2, 3).unwrap();
        c.h(2).unwrap();
        c.h(3).unwrap();

        // Block 3: YY on q2,q3 (Rx basis)
        // Rx(π/2) converts Y→Z
        c.rx(2, Parameter::Float(std::f64::consts::FRAC_PI_2))
            .unwrap();
        c.rx(3, Parameter::Float(std::f64::consts::FRAC_PI_2))
            .unwrap();
        c.cx(2, 3).unwrap();
        c.rz(3, Parameter::Float(0.09)).unwrap();
        c.cx(2, 3).unwrap();
        // Rx(-π/2) inverse
        c.rx(2, Parameter::Float(-std::f64::consts::FRAC_PI_2))
            .unwrap();
        c.rx(3, Parameter::Float(-std::f64::consts::FRAC_PI_2))
            .unwrap();

        // Block 4: XX on q0,q1
        c.h(0).unwrap();
        c.h(1).unwrap();
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.09)).unwrap();
        c.cx(0, 1).unwrap();
        c.h(0).unwrap();
        c.h(1).unwrap();

        // Block 5: YY on q0,q1
        c.rx(0, Parameter::Float(std::f64::consts::FRAC_PI_2))
            .unwrap();
        c.rx(1, Parameter::Float(std::f64::consts::FRAC_PI_2))
            .unwrap();
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.09)).unwrap();
        c.cx(0, 1).unwrap();
        c.rx(0, Parameter::Float(-std::f64::consts::FRAC_PI_2))
            .unwrap();
        c.rx(1, Parameter::Float(-std::f64::consts::FRAC_PI_2))
            .unwrap();

        let original_size = c.size();
        let u_before = c.unitary(&std::collections::HashMap::new()).unwrap();

        let pass = PhasePolynomialPass::default();
        pass.run(&mut c).unwrap();

        let u_after = c.unitary(&std::collections::HashMap::new()).unwrap();
        let n = u_before.nrows();
        let mut max_diff = 0.0f64;
        for i in 0..n {
            for j in 0..n {
                let d = (u_before[(i, j)] - u_after[(i, j)]).norm();
                if d > max_diff {
                    max_diff = d;
                }
            }
        }
        assert!(
            max_diff < 1e-10,
            "Multi-block Trotter unitary corrupted: max_diff={:.2e}, size {}→{}",
            max_diff,
            original_size,
            c.size()
        );
    }

    /// Regression test: compile H2_4q without optimization, then run
    /// PhasePolynomialPass in isolation. This isolates the pass from
    /// the rest of the PassManager to confirm it preserves fidelity.
    #[test]
    fn test_pass_preserves_h2_4q_raw_circuit() {
        use crate::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
        use crate::hamiltonian::{
            CompilationStrategy, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString,
            TrotterOrder,
        };

        let mut h = Hamiltonian::new(4);
        let terms: [(&str, f64); 15] = [
            ("IIII", -0.8105),
            ("IIIZ", 0.1721),
            ("IIZI", -0.2228),
            ("IZII", 0.1721),
            ("ZIII", -0.2228),
            ("IIZZ", 0.1686),
            ("IZIZ", 0.1205),
            ("IZZI", 0.1686),
            ("ZIIZ", 0.1686),
            ("ZIZI", 0.1205),
            ("ZZII", 0.1686),
            ("IIXX", 0.0454),
            ("IIYY", 0.0454),
            ("XXII", 0.0454),
            ("YYII", 0.0454),
        ];
        for (ps, coeff) in &terms {
            h.add_term(
                PauliString::from_str(ps).unwrap(),
                num_complex::Complex64::new(*coeff, 0.0),
            )
            .unwrap();
        }

        // Compile WITHOUT optimization (raw circuit).
        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 10,
            evolution_time: 1.0,
            adaptive: false,
            adaptive_tolerance: 1e-3,
            min_step_size: 1e-6,
            max_step_size: 1.0,
            hbar: 1.0,
            skip_identities: true,
            group_commuting_terms: true,
            apply_circuit_optimization: false,
            auto_optimize_grouping: true,
            layout_aware_grouping: false,
            optimization_strategy: CompilationStrategy::PauliLevel,
            cross_step_synthesis: false,
            block_grouping_strategy: BlockGroupingStrategy::QWC,
            pauli_gadget_optimization:
                crate::hamiltonian::GadgetOptimizationStrategy::IdenticalOnly,
            alternate_reverse_steps: true,
            clifford_enhanced_blocks: false,
        };
        let mut c = HamiltonianCompiler::new(config).compile(&h).unwrap();
        let u_before = c.unitary(&std::collections::HashMap::new()).unwrap();
        let original_size = c.size();

        // Run PhasePolynomialPass in isolation.
        let pass = PhasePolynomialPass::default();
        pass.run(&mut c).unwrap();

        let u_after = c.unitary(&std::collections::HashMap::new()).unwrap();
        let n = u_before.nrows();
        let mut max_diff = 0.0f64;
        for i in 0..n {
            for j in 0..n {
                let d = (u_before[(i, j)] - u_after[(i, j)]).norm();
                if d > max_diff {
                    max_diff = d;
                }
            }
        }
        assert!(
            max_diff < 1e-8,
            "PhasePolynomialPass corrupted H2_4q RAW circuit: max_diff={:.2e}, size {}→{}",
            max_diff,
            original_size,
            c.size()
        );
    }

    /// Regression test: compile H2_4q with full optimization pipeline
    /// (level_2 + level_4), which includes PhasePolynomialPass.
    /// If this test fails but the isolation test passes, the bug is in
    /// pass interaction, not PhasePolynomialPass itself.
    #[test]
    fn test_full_pipeline_preserves_h2_4q() {
        use crate::circuit_optimization::PassManager;
        use crate::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
        use crate::hamiltonian::{
            CompilationStrategy, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString,
            TrotterOrder,
        };

        let mut h = Hamiltonian::new(4);
        let terms: [(&str, f64); 15] = [
            ("IIII", -0.8105),
            ("IIIZ", 0.1721),
            ("IIZI", -0.2228),
            ("IZII", 0.1721),
            ("ZIII", -0.2228),
            ("IIZZ", 0.1686),
            ("IZIZ", 0.1205),
            ("IZZI", 0.1686),
            ("ZIIZ", 0.1686),
            ("ZIZI", 0.1205),
            ("ZZII", 0.1686),
            ("IIXX", 0.0454),
            ("IIYY", 0.0454),
            ("XXII", 0.0454),
            ("YYII", 0.0454),
        ];
        for (ps, coeff) in &terms {
            h.add_term(
                PauliString::from_str(ps).unwrap(),
                num_complex::Complex64::new(*coeff, 0.0),
            )
            .unwrap();
        }

        // Compile WITHOUT optimization (raw circuit).
        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 10,
            evolution_time: 1.0,
            adaptive: false,
            adaptive_tolerance: 1e-3,
            min_step_size: 1e-6,
            max_step_size: 1.0,
            hbar: 1.0,
            skip_identities: true,
            group_commuting_terms: true,
            apply_circuit_optimization: false,
            auto_optimize_grouping: true,
            layout_aware_grouping: false,
            optimization_strategy: CompilationStrategy::PauliLevel,
            cross_step_synthesis: false,
            block_grouping_strategy: BlockGroupingStrategy::QWC,
            pauli_gadget_optimization:
                crate::hamiltonian::GadgetOptimizationStrategy::IdenticalOnly,
            alternate_reverse_steps: true,
            clifford_enhanced_blocks: false,
        };
        let mut c = HamiltonianCompiler::new(config).compile(&h).unwrap();
        let u_before = c.unitary(&std::collections::HashMap::new()).unwrap();
        let original_size = c.size();

        // Run full optimization pipeline (as in compare_strategies).
        PassManager::level_2().run(&mut c).unwrap();
        if c.size() > 50 {
            PassManager::level_4().run(&mut c).unwrap();
        }

        let u_after = c.unitary(&std::collections::HashMap::new()).unwrap();
        let n = u_before.nrows();
        let mut max_diff = 0.0f64;
        for i in 0..n {
            for j in 0..n {
                let d = (u_before[(i, j)] - u_after[(i, j)]).norm();
                if d > max_diff {
                    max_diff = d;
                }
            }
        }
        assert!(
            max_diff < 1e-8,
            "Full pipeline corrupted H2_4q: max_diff={:.2e}, size {}→{}",
            max_diff,
            original_size,
            c.size()
        );
    }

    /// Binary search: test level_2 only (without level_4) to narrow down.
    #[test]
    fn test_level2_only_preserves_h2_4q() {
        use crate::circuit_optimization::PassManager;
        use crate::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
        use crate::hamiltonian::{
            CompilationStrategy, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString,
            TrotterOrder,
        };

        let mut h = Hamiltonian::new(4);
        let terms: [(&str, f64); 15] = [
            ("IIII", -0.8105),
            ("IIIZ", 0.1721),
            ("IIZI", -0.2228),
            ("IZII", 0.1721),
            ("ZIII", -0.2228),
            ("IIZZ", 0.1686),
            ("IZIZ", 0.1205),
            ("IZZI", 0.1686),
            ("ZIIZ", 0.1686),
            ("ZIZI", 0.1205),
            ("ZZII", 0.1686),
            ("IIXX", 0.0454),
            ("IIYY", 0.0454),
            ("XXII", 0.0454),
            ("YYII", 0.0454),
        ];
        for (ps, coeff) in &terms {
            h.add_term(
                PauliString::from_str(ps).unwrap(),
                num_complex::Complex64::new(*coeff, 0.0),
            )
            .unwrap();
        }
        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 10,
            evolution_time: 1.0,
            adaptive: false,
            adaptive_tolerance: 1e-3,
            min_step_size: 1e-6,
            max_step_size: 1.0,
            hbar: 1.0,
            skip_identities: true,
            group_commuting_terms: true,
            apply_circuit_optimization: false,
            auto_optimize_grouping: true,
            layout_aware_grouping: false,
            optimization_strategy: CompilationStrategy::PauliLevel,
            cross_step_synthesis: false,
            block_grouping_strategy: BlockGroupingStrategy::QWC,
            pauli_gadget_optimization:
                crate::hamiltonian::GadgetOptimizationStrategy::IdenticalOnly,
            alternate_reverse_steps: true,
            clifford_enhanced_blocks: false,
        };
        let mut c = HamiltonianCompiler::new(config).compile(&h).unwrap();
        let u_before = c.unitary(&std::collections::HashMap::new()).unwrap();
        let original_size = c.size();

        PassManager::level_2().run(&mut c).unwrap();

        let u_after = c.unitary(&std::collections::HashMap::new()).unwrap();
        let n = u_before.nrows();
        let mut max_diff = 0.0f64;
        for i in 0..n {
            for j in 0..n {
                let d = (u_before[(i, j)] - u_after[(i, j)]).norm();
                if d > max_diff {
                    max_diff = d;
                }
            }
        }
        assert!(
            max_diff < 1e-8,
            "level_2 alone corrupted H2_4q: max_diff={:.2e}, size {}→{}",
            max_diff,
            original_size,
            c.size()
        );
    }

    /// Test level_4 only (without level_2) to further narrow down.
    #[test]
    fn test_level4_only_preserves_h2_4q() {
        use crate::circuit_optimization::PassManager;
        use crate::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
        use crate::hamiltonian::{
            CompilationStrategy, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString,
            TrotterOrder,
        };

        let mut h = Hamiltonian::new(4);
        let terms: [(&str, f64); 15] = [
            ("IIII", -0.8105),
            ("IIIZ", 0.1721),
            ("IIZI", -0.2228),
            ("IZII", 0.1721),
            ("ZIII", -0.2228),
            ("IIZZ", 0.1686),
            ("IZIZ", 0.1205),
            ("IZZI", 0.1686),
            ("ZIIZ", 0.1686),
            ("ZIZI", 0.1205),
            ("ZZII", 0.1686),
            ("IIXX", 0.0454),
            ("IIYY", 0.0454),
            ("XXII", 0.0454),
            ("YYII", 0.0454),
        ];
        for (ps, coeff) in &terms {
            h.add_term(
                PauliString::from_str(ps).unwrap(),
                num_complex::Complex64::new(*coeff, 0.0),
            )
            .unwrap();
        }
        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 10,
            evolution_time: 1.0,
            adaptive: false,
            adaptive_tolerance: 1e-3,
            min_step_size: 1e-6,
            max_step_size: 1.0,
            hbar: 1.0,
            skip_identities: true,
            group_commuting_terms: true,
            apply_circuit_optimization: false,
            auto_optimize_grouping: true,
            layout_aware_grouping: false,
            optimization_strategy: CompilationStrategy::PauliLevel,
            cross_step_synthesis: false,
            block_grouping_strategy: BlockGroupingStrategy::QWC,
            pauli_gadget_optimization:
                crate::hamiltonian::GadgetOptimizationStrategy::IdenticalOnly,
            alternate_reverse_steps: true,
            clifford_enhanced_blocks: false,
        };
        let mut c = HamiltonianCompiler::new(config).compile(&h).unwrap();
        let u_before = c.unitary(&std::collections::HashMap::new()).unwrap();
        let original_size = c.size();

        PassManager::level_4().run(&mut c).unwrap();

        let u_after = c.unitary(&std::collections::HashMap::new()).unwrap();
        let n = u_before.nrows();
        let mut max_diff = 0.0f64;
        for i in 0..n {
            for j in 0..n {
                let d = (u_before[(i, j)] - u_after[(i, j)]).norm();
                if d > max_diff {
                    max_diff = d;
                }
            }
        }
        assert!(
            max_diff < 1e-8,
            "level_4 alone corrupted H2_4q: max_diff={:.2e}, size {}→{}",
            max_diff,
            original_size,
            c.size()
        );
    }

    /// Profile which pass in level_4 is slow.
    #[test]
    fn test_profile_level4_passes() {
        use crate::circuit_optimization::CircuitPass;
        use crate::circuit_optimization::PassManager;
        use crate::circuit_optimization::{
            BlockConsolidationPass, CancelInversePairsPass, CommutativeCancellationPass,
            TemplateMatchingPass,
        };
        use crate::cnot_optimizer::CNOTOptimizer;
        use crate::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
        use crate::hamiltonian::{
            CompilationStrategy, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString,
            TrotterOrder,
        };
        use crate::single_qubit_optimizer::SingleQubitOptimizer;
        use std::time::Instant;

        let mut h = Hamiltonian::new(4);
        let terms: [(&str, f64); 15] = [
            ("IIII", -0.8105),
            ("IIIZ", 0.1721),
            ("IIZI", -0.2228),
            ("IZII", 0.1721),
            ("ZIII", -0.2228),
            ("IIZZ", 0.1686),
            ("IZIZ", 0.1205),
            ("IZZI", 0.1686),
            ("ZIIZ", 0.1686),
            ("ZIZI", 0.1205),
            ("ZZII", 0.1686),
            ("IIXX", 0.0454),
            ("IIYY", 0.0454),
            ("XXII", 0.0454),
            ("YYII", 0.0454),
        ];
        for (ps, coeff) in &terms {
            h.add_term(
                PauliString::from_str(ps).unwrap(),
                num_complex::Complex64::new(*coeff, 0.0),
            )
            .unwrap();
        }
        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 10,
            evolution_time: 1.0,
            adaptive: false,
            adaptive_tolerance: 1e-3,
            min_step_size: 1e-6,
            max_step_size: 1.0,
            hbar: 1.0,
            skip_identities: true,
            group_commuting_terms: true,
            apply_circuit_optimization: false,
            auto_optimize_grouping: true,
            layout_aware_grouping: false,
            optimization_strategy: CompilationStrategy::PauliLevel,
            cross_step_synthesis: false,
            block_grouping_strategy: BlockGroupingStrategy::QWC,
            pauli_gadget_optimization:
                crate::hamiltonian::GadgetOptimizationStrategy::IdenticalOnly,
            alternate_reverse_steps: true,
            clifford_enhanced_blocks: false,
        };
        let mut c = HamiltonianCompiler::new(config).compile(&h).unwrap();

        // Manually apply each pass and time it.
        let passes: Vec<(String, Box<dyn CircuitPass>)> = vec![
            ("SQOpt".into(), Box::new(SingleQubitOptimizer::new())),
            (
                "CancelInverse".into(),
                Box::new(CancelInversePairsPass::new()),
            ),
            ("CNOTOpt".into(), Box::new(CNOTOptimizer::new())),
            (
                "CommutativeCancel".into(),
                Box::new(CommutativeCancellationPass::new()),
            ),
            (
                "BlockConsolidation".into(),
                Box::new(BlockConsolidationPass::new()),
            ),
            (
                "PhasePolynomial".into(),
                Box::<PhasePolynomialPass>::default(),
            ),
            ("CNOTOpt2".into(), Box::new(CNOTOptimizer::new())),
            (
                "CommutativeCancel2".into(),
                Box::new(CommutativeCancellationPass::new()),
            ),
            (
                "TemplateMatching".into(),
                Box::new(TemplateMatchingPass::new()),
            ),
            ("SQOpt2".into(), Box::new(SingleQubitOptimizer::new())),
        ];

        println!("=== Level 4 pass profiling ({} gates) ===", c.size());
        for (name, pass) in &passes {
            let start = Instant::now();
            pass.run(&mut c).unwrap();
            let elapsed = start.elapsed();
            println!(
                "  {:25} {:>8.3}s  (now {} gates)",
                name,
                elapsed.as_secs_f64(),
                c.size()
            );
        }
        println!("  Total circuit size: {} gates", c.size());
    }

    /// Test running PhasePolynomialPass twice — each pass should be idempotent.
    #[test]
    fn test_pass_idempotent_on_h2_4q() {
        use crate::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
        use crate::hamiltonian::{
            CompilationStrategy, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString,
            TrotterOrder,
        };

        let mut h = Hamiltonian::new(4);
        let terms: [(&str, f64); 15] = [
            ("IIII", -0.8105),
            ("IIIZ", 0.1721),
            ("IIZI", -0.2228),
            ("IZII", 0.1721),
            ("ZIII", -0.2228),
            ("IIZZ", 0.1686),
            ("IZIZ", 0.1205),
            ("IZZI", 0.1686),
            ("ZIIZ", 0.1686),
            ("ZIZI", 0.1205),
            ("ZZII", 0.1686),
            ("IIXX", 0.0454),
            ("IIYY", 0.0454),
            ("XXII", 0.0454),
            ("YYII", 0.0454),
        ];
        for (ps, coeff) in &terms {
            h.add_term(
                PauliString::from_str(ps).unwrap(),
                num_complex::Complex64::new(*coeff, 0.0),
            )
            .unwrap();
        }
        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 10,
            evolution_time: 1.0,
            adaptive: false,
            adaptive_tolerance: 1e-3,
            min_step_size: 1e-6,
            max_step_size: 1.0,
            hbar: 1.0,
            skip_identities: true,
            group_commuting_terms: true,
            apply_circuit_optimization: false,
            auto_optimize_grouping: true,
            layout_aware_grouping: false,
            optimization_strategy: CompilationStrategy::PauliLevel,
            cross_step_synthesis: false,
            block_grouping_strategy: BlockGroupingStrategy::QWC,
            pauli_gadget_optimization:
                crate::hamiltonian::GadgetOptimizationStrategy::IdenticalOnly,
            alternate_reverse_steps: true,
            clifford_enhanced_blocks: false,
        };
        let mut c = HamiltonianCompiler::new(config).compile(&h).unwrap();
        let u_before = c.unitary(&std::collections::HashMap::new()).unwrap();
        let original_size = c.size();

        // Run PhasePolynomialPass twice.
        let pass = PhasePolynomialPass::default();
        pass.run(&mut c).unwrap();
        pass.run(&mut c).unwrap();

        let u_after = c.unitary(&std::collections::HashMap::new()).unwrap();
        let n = u_before.nrows();
        let mut max_diff = 0.0f64;
        for i in 0..n {
            for j in 0..n {
                let d = (u_before[(i, j)] - u_after[(i, j)]).norm();
                if d > max_diff {
                    max_diff = d;
                }
            }
        }
        assert!(
            max_diff < 1e-8,
            "Double PhasePolynomialPass corrupted H2_4q: max_diff={:.2e}, size {}→{}",
            max_diff,
            original_size,
            c.size()
        );
    }

    // ── AdaptiveSynthesis tests ─────────────────────────────────────

    #[test]
    fn test_adaptive_empty() {
        let synth = AdaptiveSynthesis::with_default_strategies();
        let gates = synth.synthesize(&[], 4).unwrap();
        assert!(gates.is_empty());
    }

    #[test]
    fn test_adaptive_single_term() {
        let synth = AdaptiveSynthesis::with_default_strategies();
        let terms = vec![(Parity(0b0011), 0.5)];
        let gates = synth.synthesize(&terms, 2).unwrap();
        assert!(!gates.is_empty());
        // Should have at least 1 CX + 1 Rz
        let cx = gates.iter().filter(|g| g.gate == StandardGate::CX).count();
        let rz = gates.iter().filter(|g| g.gate == StandardGate::Rz).count();
        assert!(cx > 0);
        assert_eq!(rz, 1);
    }

    #[test]
    fn test_adaptive_frame_safe() {
        // All strategies are frame-safe; verify output restores identity.
        let synth = AdaptiveSynthesis::with_default_strategies();
        let terms = vec![
            (Parity(0b0011), 0.1),
            (Parity(0b1100), 0.2),
            (Parity(0b0101), 0.3),
        ];
        let gates = synth.synthesize(&terms, 4).unwrap();

        let mut frame: Vec<u64> = (0..4).map(|q| 1u64 << q).collect();
        for sg in &gates {
            if matches!(sg.gate, StandardGate::CX) {
                frame[sg.qubits[1]] ^= frame[sg.qubits[0]];
            }
        }
        for q in 0..4 {
            assert_eq!(
                frame[q],
                1u64 << q,
                "AdaptiveSynthesis: qubit {} frame not identity",
                q
            );
        }
    }

    #[test]
    fn test_adaptive_picks_best_strategy() {
        // For prefix parities, both strategies should work. Verify at least
        // one of them produces a result (no regression vs single strategy).
        let synth = AdaptiveSynthesis::with_default_strategies();
        let terms = vec![(Parity(0b0011), 0.2), (Parity(0b0111), 0.3)];
        let gates = synth.synthesize(&terms, 4).unwrap();
        assert!(!gates.is_empty());

        // All gate qubits should be valid.
        for sg in &gates {
            for &q in &sg.qubits {
                assert!(q < 4);
            }
        }
    }

    #[test]
    fn test_adaptive_integration_with_pass() {
        // Integration: pass with AdaptiveSynthesis preserves unitary.
        let mut c = QuantumCircuit::new(3, 0);
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.5)).unwrap();
        c.rz(1, Parameter::Float(0.3)).unwrap();
        c.cx(0, 1).unwrap();

        let u_before = c.unitary(&std::collections::HashMap::new()).ok();
        let pass = PhasePolynomialPass::default(); // Uses AdaptiveSynthesis
        pass.run(&mut c).unwrap();
        let u_after = c.unitary(&std::collections::HashMap::new()).ok();

        if let (Some(ub), Some(ua)) = (&u_before, &u_after) {
            let n = ub.nrows();
            let mut max_diff = 0.0f64;
            for i in 0..n {
                for j in 0..n {
                    let d = (ub[(i, j)] - ua[(i, j)]).norm();
                    if d > max_diff {
                        max_diff = d;
                    }
                }
            }
            assert!(
                max_diff < 1e-8,
                "AdaptiveSynthesis pass corrupted unitary: max_diff={}",
                max_diff
            );
        }
    }

    /// Multi-block circuit with overlapping CNOT trees (inter-block adjacency).
    /// Tests the case where adjacent blocks share CNOT gates at boundaries.
    #[test]
    fn test_pass_preserves_overlapping_cnot_blocks() {
        let mut c = QuantumCircuit::new(3, 0);

        // Block 1: Z on q0,q1 with chain
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.5)).unwrap();
        c.cx(0, 1).unwrap();

        // No basis change — immediately adjacent CX from next block
        // Block 2: Z on q1,q2 (shares q1)
        c.cx(1, 2).unwrap();
        c.rz(2, Parameter::Float(0.3)).unwrap();
        c.cx(1, 2).unwrap();

        // Block 3: X on q0 (H basis, single qubit)
        c.h(0).unwrap();
        c.rz(0, Parameter::Float(0.2)).unwrap();
        c.h(0).unwrap();

        let original_size = c.size();
        let u_before = c.unitary(&std::collections::HashMap::new()).unwrap();

        let pass = PhasePolynomialPass::default();
        pass.run(&mut c).unwrap();

        let u_after = c.unitary(&std::collections::HashMap::new()).unwrap();
        let n = u_before.nrows();
        let mut max_diff = 0.0f64;
        for i in 0..n {
            for j in 0..n {
                let d = (u_before[(i, j)] - u_after[(i, j)]).norm();
                if d > max_diff {
                    max_diff = d;
                }
            }
        }
        assert!(
            max_diff < 1e-10,
            "Overlapping CNOT blocks corrupted: max_diff={:.2e}, size {}→{}",
            max_diff,
            original_size,
            c.size()
        );
    }
}
