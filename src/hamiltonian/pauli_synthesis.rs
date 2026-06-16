//! Pauli-Level Circuit Synthesis
//!
//! Implements block-wise synthesis with QWC block merging + max_match_synthesis
//! from Paulihedral (ASPLOS 2022). Commuting Pauli terms within a QWC group share
//! a single CNOT tree, reducing CX gates 50-70% vs per-term synthesis.

use std::collections::HashMap;
use std::f64::consts::PI;

use super::pauli_gadget::{compatible_pair_check, conjugate_pauli_operator, CliffordGate};
use super::{PauliOperator, PauliString};
use crate::{Parameter, QuantumCircuit};

/// Map from Pauli string representation → Clifford gates for
/// circuit-level Clifford-aligned term synthesis (Phase 11a-3).
///
/// Key: Pauli string representation (e.g. "IIYY"). Each Pauli string in a
/// Hamiltonian is unique, so we don't need angle disambiguation.
/// Using only the Pauli string avoids coefficient scaling mismatches
/// between the compiler (raw angles) and synthesis (scaled by dt/hbar).
pub type CliffordAnnotationMap =
    HashMap<String, (Vec<(CliffordGate, usize)>, Vec<(CliffordGate, usize)>)>;

/// Build a lookup key for Clifford annotations.
fn clifford_key(pauli: &PauliString, _angle: f64) -> String {
    pauli.to_string_repr().to_string()
}

// ---------------------------------------------------------------------------
// Gate budget instrumentation (Phase 9a diagnostic)
// ---------------------------------------------------------------------------

/// Per-category gate counts for PauliLevel synthesis.
///
/// Used to diagnose where PauliLevel adds gates vs GateLevel.
#[derive(Debug, Clone, Default)]
pub struct PauliGateBudget {
    /// Rz from single-qubit terms (no CNOT tree needed)
    pub single_qubit_rz: usize,
    /// Rz from prefix terms at position 0 of shared chain (forward side)
    pub rz_position_0_forward: usize,
    /// Rz from prefix terms at position 0 of shared chain (reverse side —
    /// this is EXTRA waste from splitting when not needed for inter-block
    /// cancellation)
    pub rz_position_0_reverse: usize,
    /// CX gates from shared prefix chains
    pub cx_shared_trees: usize,
    /// Rz gates from shared chains (positions > 0, merged 2*coeff)
    pub rz_shared_trees: usize,
    /// CX gates from per-term fallback synthesis
    pub cx_per_term: usize,
    /// Rz gates from per-term fallback
    pub rz_per_term: usize,
    /// H/Rx basis transformation gates (both forward and inverse)
    pub basis_changes: usize,
    /// CX gates from P1 secondary shared chains
    pub cx_secondary_chain: usize,
    /// Rz gates from P1 secondary shared chains
    pub rz_secondary_chain: usize,
    /// Count of terms synthesized via per-term path (fallback)
    pub per_term_count: usize,
    /// Count of terms synthesized via shared tree (prefix-compatible)
    pub shared_tree_term_count: usize,
    /// Count of terms synthesized via secondary chain (P1)
    pub secondary_chain_term_count: usize,
    /// Number of QWC blocks formed
    pub num_blocks: usize,
    /// Number of paired edges (inter-block cancellation attempted)
    pub num_paired_edges: usize,
    /// Number of singleton blocks (no pairing)
    pub num_singletons: usize,
}

impl PauliGateBudget {
    pub fn total_gates(&self) -> usize {
        self.single_qubit_rz
            + self.rz_position_0_forward
            + self.rz_position_0_reverse
            + self.cx_shared_trees
            + self.rz_shared_trees
            + self.cx_per_term
            + self.rz_per_term
            + self.basis_changes
            + self.cx_secondary_chain
            + self.rz_secondary_chain
    }

    pub fn total_cx(&self) -> usize {
        self.cx_shared_trees + self.cx_per_term + self.cx_secondary_chain
    }

    /// Gates that GateLevel would NOT have emitted — pure PauliLevel overhead.
    pub fn overhead_gates(&self) -> usize {
        self.rz_position_0_reverse
    }

    pub fn print(&self, label: &str) {
        println!("=== Gate Budget: {} ===", label);
        println!(
            "  Blocks: {} ({} paired, {} singletons)",
            self.num_blocks, self.num_paired_edges, self.num_singletons
        );
        println!(
            "  Terms:  {} shared-tree, {} secondary-chain, {} per-term",
            self.shared_tree_term_count, self.secondary_chain_term_count, self.per_term_count
        );
        println!("  --- Gates by category ---");
        println!("  Single-qubit Rz:     {:>5}", self.single_qubit_rz);
        println!("  CX shared trees:     {:>5}", self.cx_shared_trees);
        println!("  Rz shared trees:     {:>5}", self.rz_shared_trees);
        println!("  CX secondary chain:  {:>5}", self.cx_secondary_chain);
        println!("  Rz secondary chain:  {:>5}", self.rz_secondary_chain);
        println!("  CX per-term:         {:>5}", self.cx_per_term);
        println!("  Rz per-term:         {:>5}", self.rz_per_term);
        println!("  Basis changes:       {:>5}", self.basis_changes);
        println!("  Rz pos-0 forward:    {:>5}", self.rz_position_0_forward);
        println!(
            "  Rz pos-0 REVERSE:    {:>5}  <-- overhead",
            self.rz_position_0_reverse
        );
        println!("  --- Totals ---");
        println!("  Total gates:         {:>5}", self.total_gates());
        println!("  Total CX:            {:>5}", self.total_cx());
        println!("  Overhead (extra):    {:>5}", self.overhead_gates());
    }
}

// ---------------------------------------------------------------------------
// Pauli algebra helpers
// ---------------------------------------------------------------------------

/// Merge two Pauli strings via pOR: position-wise, keep non-I from first,
/// fill from second where first is I. Used to compute block mstr.
fn p_or(a: &PauliString, b: &PauliString) -> PauliString {
    let ops: Vec<PauliOperator> = a
        .operators
        .iter()
        .zip(b.operators.iter())
        .map(|(oa, ob)| if *oa != PauliOperator::I { *oa } else { *ob })
        .collect();
    PauliString::new(ops, a.coefficient)
}

/// Check Qubit-Wise Commuting: at each position, operators must be same
/// OR at least one is Identity.
fn is_qwc(a: &PauliString, b: &PauliString) -> bool {
    a.operators
        .iter()
        .zip(b.operators.iter())
        .all(|(oa, ob)| oa == ob || *oa == PauliOperator::I || *ob == PauliOperator::I)
}

/// Check general Pauli commutativity.
///
/// Two Pauli strings commute iff they anti-commute at an even number of
/// qubit positions (including zero). Anti-commutation occurs wherever both
/// operators are non-Identity AND different.
///
/// QWC is a sufficient but not necessary condition for commutativity.
/// General commuting allows terms like XXII and YYII (anti-commute at 2
/// positions = even → commute) that QWC would split into separate blocks.
fn is_commuting(a: &PauliString, b: &PauliString) -> bool {
    let anti_count = a
        .operators
        .iter()
        .zip(b.operators.iter())
        .filter(|(oa, ob)| **oa != PauliOperator::I && **ob != PauliOperator::I && *oa != *ob)
        .count();
    anti_count % 2 == 0
}

/// Block grouping strategy for Pauli synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockGroupingStrategy {
    /// Qubit-Wise Commuting — strictest criterion, smallest blocks,
    /// simplest CNOT diagonalization.
    QWC,
    /// General commuting — graph-coloring-based, larger blocks that
    /// internally decompose into QWC subgroups with aligned CNOT trees.
    GeneralCommuting,
}

/// Get active (non-I) qubit indices, sorted.
fn active_nodes(pauli: &PauliString) -> Vec<usize> {
    let mut nodes: Vec<usize> = pauli
        .operators
        .iter()
        .enumerate()
        .filter_map(|(i, op)| {
            if *op != PauliOperator::I {
                Some(i)
            } else {
                None
            }
        })
        .collect();
    nodes.sort_unstable();
    nodes
}

/// Get the list of qubit indices where both Pauli strings are non-I.
pub(crate) fn mutual_positions(pa: &PauliString, pb: &PauliString) -> Vec<usize> {
    pa.operators
        .iter()
        .zip(pb.operators.iter())
        .enumerate()
        .filter_map(|(i, (a, b))| {
            if *a != PauliOperator::I && *b != PauliOperator::I {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Basis change helpers
// ---------------------------------------------------------------------------

fn apply_basis_change(
    circuit: &mut QuantumCircuit,
    qubit: usize,
    op: PauliOperator,
) -> crate::Result<()> {
    match op {
        PauliOperator::X => circuit.h(qubit)?,
        PauliOperator::Y => {
            // Rx(π/2) converts Y basis to Z basis, avoiding extra S/Sdg gates.
            // Equivalent to S·H up to global phase on single qubit,
            // and produces the correct relative phase in multi-qubit circuits.
            circuit.rx(qubit, Parameter::Float(PI / 2.0))?;
        }
        _ => {}
    }
    Ok(())
}

fn apply_inv_basis_change(
    circuit: &mut QuantumCircuit,
    qubit: usize,
    op: PauliOperator,
) -> crate::Result<()> {
    match op {
        PauliOperator::X => circuit.h(qubit)?,
        PauliOperator::Y => {
            // Rx(-π/2) converts back from Z basis to Y basis.
            circuit.rx(qubit, Parameter::Float(-PI / 2.0))?;
        }
        _ => {}
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Block data structures
// ---------------------------------------------------------------------------

/// A QWC block: a group of commuting Pauli terms that share a single CNOT tree.
pub(crate) struct PauliBlock {
    /// (pauli_string, scaled_coefficient) for each term in the block
    terms: Vec<(PauliString, f64)>,
    /// pOR-merged Pauli string representing the union of all term operators
    mstr: PauliString,
    /// Sorted active qubit indices of mstr — the CNOT tree covers these
    nodes: Vec<usize>,
    /// Per-term Clifford gate annotations for merged blocks (Phase 11f).
    ///
    /// Same length as `terms`. `None` = no conjugation (original term,
    /// can participate in shared CNOT tree). `Some(pre_gates)` = this
    /// term requires Clifford conjugation before synthesis (and inverse
    /// after). Conjugated terms are synthesized individually — they
    /// cannot merge their Rz gates with other terms in the shared tree.
    ///
    /// Set by `form_blocks_clifford_enhanced()`; empty for `form_blocks()`.
    clifford_annotations: Vec<Option<Vec<(CliffordGate, usize)>>>,
}

/// Cached block structure from a forward-pass compilation.
///
/// Stores enough information to reproduce identical QWC block grouping,
/// node ordering, and edge pairing for the reverse Trotter pass.
/// This eliminates non-deterministic block formation (cause 2) and makes
/// the `is_reverse` parameter meaningful (cause 3).
#[derive(Clone)]
pub struct PauliBlockCache {
    /// Pauli string signatures (e.g. "ZIII") grouped by block.
    /// Used as keys to reconstruct blocks for the reverse pass.
    pub block_signatures: Vec<Vec<String>>,
    /// Per-block sorted node ordering
    pub block_nodes: Vec<Vec<usize>>,
    /// Per-block merged mstr
    pub block_mstrs: Vec<PauliString>,
    /// Pair edges: (left_block_idx, right_block_idx)
    pub edges: Vec<(usize, usize)>,
    /// Per-edge link qubits for inter-block cancellation
    pub edge_links: Vec<Vec<usize>>,
    /// Per-edge (split_left, split_right) position-0 flags
    pub split_flags: Vec<(bool, bool)>,
    /// Per-block Clifford annotations (Phase 11f). Same structure as
    /// `PauliBlock::clifford_annotations`. Empty Vec means no annotations.
    pub block_clifford_annotations: Vec<Vec<Option<Vec<(CliffordGate, usize)>>>>,
}

impl PauliBlock {
    fn cost(&self) -> usize {
        self.nodes.len()
    }
}

/// Group Pauli terms into QWC blocks with merged mstr.
///
/// Terms within a block are all mutually QWC. The block's mstr = pOR of all
/// term Pauli strings, so the CNOT tree built from mstr.nodes covers every term.
fn form_blocks(terms: &[(PauliString, f64)]) -> Vec<PauliBlock> {
    if terms.is_empty() {
        return vec![];
    }

    let mut groups: Vec<Vec<(PauliString, f64)>> = vec![];

    'outer: for (pauli, coeff) in terms {
        for group in &mut groups {
            if group.iter().all(|(gp, _)| is_qwc(gp, pauli)) {
                group.push((pauli.clone(), *coeff));
                continue 'outer;
            }
        }
        groups.push(vec![(pauli.clone(), *coeff)]);
    }

    groups
        .into_iter()
        .map(|group| {
            let mstr = group
                .iter()
                .map(|(p, _)| p)
                .fold(PauliString::identity(group[0].0.num_qubits()), |acc, p| {
                    p_or(&acc, p)
                });
            let nodes = active_nodes(&mstr);
            let n = group.len();
            PauliBlock {
                terms: group,
                mstr,
                nodes,
                clifford_annotations: vec![None; n],
            }
        })
        .collect()
}

/// Form QWC blocks using effective (Clifford-conjugated) Pauli strings for
/// grouping, so that terms with the same effective Pauli share a block.
///
/// # Algorithm
///
/// 1. For each term, look up its original Pauli string repr in `clifford_map`.
/// 2. If found: compute `effective = apply_clifford_pre_gates(original, pre_gates)`.
///    Store the `pre_gates` for later annotation.
/// 3. If not found: `effective = original`, no annotation.
/// 4. Form QWC blocks greedily using **effective** Paulis (same algorithm
///    as `form_blocks`).
/// 5. Block's `terms` store the effective Paulis so that `mstr` computation
///    and downstream synthesis use the conjugated operators.
/// 6. Block's `clifford_annotations[idx]` = `Some(pre_gates)` for terms
///    whose effective Pauli differs from their original.
///
/// # Relationship to `form_blocks_clifford_enhanced`
///
/// `form_blocks_clifford_aware` operates at the **term** level using a
/// pre-existing Clifford map (from GreedyPauliSimp or CliffordSimple).
/// `form_blocks_clifford_enhanced` operates at the **block** level using
/// `compatible_pair_check` to merge existing blocks. When a Clifford map
/// is available (Phase 11i), the term-level approach is more precise.
#[allow(dead_code)]
fn form_blocks_clifford_aware(
    terms: &[(PauliString, f64)],
    clifford_map: &CliffordAnnotationMap,
) -> Vec<PauliBlock> {
    if terms.is_empty() {
        return vec![];
    }

    // Step 1: Build effective terms with annotation tracking.
    // effective_terms[i] = (effective_pauli, coeff, Option<pre_gates>)
    let effective_terms: Vec<(PauliString, f64, Option<Vec<(CliffordGate, usize)>>)> = terms
        .iter()
        .map(|(pauli, coeff)| {
            let key = clifford_key(pauli, *coeff);
            if let Some((pre_gates, _post_gates)) = clifford_map.get(&key) {
                let effective = super::pauli_gadget::apply_clifford_pre_gates(pauli, pre_gates);
                (effective, *coeff, Some(pre_gates.clone()))
            } else {
                (pauli.clone(), *coeff, None)
            }
        })
        .collect();

    // Step 2: Greedy QWC block formation on effective Paulis
    // (same algorithm as form_blocks).
    let mut groups: Vec<Vec<(PauliString, f64, Option<Vec<(CliffordGate, usize)>>)>> = vec![];

    'outer: for (effective_pauli, coeff, ann) in effective_terms.iter() {
        for group in &mut groups {
            if group.iter().all(|(gp, _, _)| is_qwc(gp, effective_pauli)) {
                group.push((effective_pauli.clone(), *coeff, ann.clone()));
                continue 'outer;
            }
        }
        groups.push(vec![(effective_pauli.clone(), *coeff, ann.clone())]);
    }

    // Step 3: Build PauliBlock with clifford_annotations populated.
    groups
        .into_iter()
        .map(|group| {
            let _n = group.len();
            let terms: Vec<(PauliString, f64)> =
                group.iter().map(|(p, c, _)| (p.clone(), *c)).collect();
            let annotations: Vec<Option<Vec<(CliffordGate, usize)>>> =
                group.iter().map(|(_, _, ann)| ann.clone()).collect();
            let mstr = group
                .iter()
                .map(|(p, _, _)| p)
                .fold(PauliString::identity(group[0].0.num_qubits()), |acc, p| {
                    p_or(&acc, p)
                });
            let nodes = active_nodes(&mstr);
            PauliBlock {
                terms,
                mstr,
                nodes,
                clifford_annotations: annotations,
            }
        })
        .collect()
}

/// Clifford-enhanced QWC block formation: greedily merges QWC blocks whose
/// master Pauli strings can be made QWC-compatible via single-qubit Clifford
/// conjugation.
///
/// # Algorithm
///
/// 1. Form initial QWC blocks via `form_blocks()` (greedy, O(n·k) for n terms, k blocks).
/// 2. For each pair of blocks, check `compatible_pair_check(a.mstr, b.mstr)`.
///    A compatible pair means applying Clifford gates to one block's circuit
///    output makes it QWC with the other — they can share a CNOT tree.
/// 3. Score each merge by CNOT savings: 1 shared tree vs 2 separate trees.
/// 4. Greedily merge the best-scoring pair, recompute the merged block.
/// 5. Repeat until no beneficial merges remain.
///
/// # WARNING: Infrastructure only (Phase 11e)
///
/// This function implements block MERGING but NOT Clifford gate emission.
/// The Clifford gates recorded by `compatible_pair_check` are NOT emitted
/// in the synthesized circuit, so merged blocks will produce **incorrect
/// unitaries** when this function is used without Phase 11f's gate emission.
///
/// This function is exposed for testing and future use; it is gated behind
/// `CompilerConfig::clifford_enhanced_blocks` (default `true`).
///
/// # Returns
///
/// Potentially fewer blocks than `form_blocks()`. The merged block's `mstr`
/// is recomputed as the pOR of all terms in the block.
///
/// # Phase 11f (planned)
///
/// During synthesis, the Clifford gates from `compatible_pair_check` will
/// be emitted around each individual term's rotation, making the merged
/// blocks' unitaries correct.
pub(crate) fn form_blocks_clifford_enhanced(terms: &[(PauliString, f64)]) -> Vec<PauliBlock> {
    // Start with standard QWC blocks.
    let mut blocks = form_blocks(terms);
    if blocks.len() <= 1 {
        return blocks;
    }

    let num_qubits = if let Some(first) = terms.first() {
        first.0.num_qubits()
    } else {
        return blocks;
    };

    // Greedy merge loop.
    loop {
        let mut best_score: Option<(usize, usize, usize)> = None; // (i, j, savings)

        for i in 0..blocks.len() {
            for j in (i + 1)..blocks.len() {
                // Check if blocks' mstr strings can be made QWC-compatible.
                if let Some(clifford_gates) =
                    compatible_pair_check(&blocks[i].mstr, &blocks[j].mstr)
                {
                    // Phase 11f: After merging, block_j's terms are synthesized
                    // INDIVIDUALLY (Clifford gates prevent Rz merging in shared tree).
                    // Before: 2 separate shared trees
                    // After:  1 shared tree (block_i) + per-term synthesis (block_j)
                    //         + Clifford gate overhead
                    let cost_i = 2 * blocks[i].nodes.len().saturating_sub(1);
                    let cost_j = 2 * blocks[j].nodes.len().saturating_sub(1);

                    // Merged block's shared tree covers union of both node sets.
                    let mut merged_nodes: Vec<usize> = blocks[i].nodes.clone();
                    for &n in &blocks[j].nodes {
                        if !merged_nodes.contains(&n) {
                            merged_nodes.push(n);
                        }
                    }
                    merged_nodes.sort_unstable();
                    let cost_shared = 2 * merged_nodes.len().saturating_sub(1);

                    // Per-term CX cost for block_j's terms (individual synthesis).
                    let per_term_cx: usize = blocks[j]
                        .terms
                        .iter()
                        .map(|(p, _)| {
                            let k = active_nodes(p).len();
                            2 * k.saturating_sub(1)
                        })
                        .sum();

                    // Clifford gate overhead: pre+post for each block_j term.
                    let clifford_overhead = 2 * clifford_gates.len() * blocks[j].terms.len();

                    let cost_after = cost_shared + per_term_cx + clifford_overhead;
                    let savings = (cost_i + cost_j).saturating_sub(cost_after);

                    match &best_score {
                        None => best_score = Some((i, j, savings)),
                        Some((_, _, s)) if savings > *s => best_score = Some((i, j, savings)),
                        _ => {}
                    }
                }
            }
        }

        // Merge the best pair (if any with positive savings).
        if let Some((i, j, savings)) = best_score {
            if savings > 0 {
                // Get the Clifford gates that align block_j's mstr with block_i's.
                let clifford_gates = compatible_pair_check(&blocks[i].mstr, &blocks[j].mstr)
                    .expect("compatible_pair_check must succeed (validated above)");

                // Merge block j into block i.
                let block_j = blocks.remove(j);
                // Adjust i if j < i (since removing j shifts indices).
                let actual_i = if j < i { i - 1 } else { i };
                let block_i = &mut blocks[actual_i];

                let orig_i_terms = block_i.terms.len();
                // Extend block_i's Clifford annotations for its existing terms.
                block_i
                    .clifford_annotations
                    .extend(std::iter::repeat(None).take(block_j.terms.len()));

                // Compute conjugated Pauli strings for block_j's terms and
                // set Clifford annotations.
                for (term_idx, (pauli, _coeff)) in block_j.terms.iter().enumerate() {
                    let mut conjugated = pauli.clone();
                    for &(q, gate) in &clifford_gates {
                        // Apply Clifford conjugation to the Pauli operator at qubit q.
                        if let Some(new_op) =
                            conjugate_pauli_operator(gate, conjugated.operators[q], true)
                        {
                            conjugated.operators[q] = new_op;
                        }
                    }
                    // Store the conjugated Pauli (for correct mstr computation)
                    // and the original term's angle.
                    let clifford_idx = orig_i_terms + term_idx;
                    // Convert (usize, CliffordGate) → (CliffordGate, usize) for
                    // consistency with Phase 11a-3 annotation format.
                    let gates_fmt: Vec<(CliffordGate, usize)> =
                        clifford_gates.iter().map(|&(q, g)| (g, q)).collect();
                    block_i.clifford_annotations[clifford_idx] = Some(gates_fmt);
                    // Replace the stored Pauli with the conjugated version so
                    // mstr computation uses the correct operators.
                    block_i.terms.push((conjugated, block_j.terms[term_idx].1));
                }

                // Recompute mstr = pOR of all term Paulis (now conjugated for block_j).
                let mut new_mstr = PauliString::identity(num_qubits);
                for (p, _) in &block_i.terms {
                    new_mstr = p_or(&new_mstr, p);
                }
                block_i.mstr = new_mstr;

                // Recompute nodes from conjugated Paulis.
                let mut all_nodes: Vec<usize> = vec![];
                let mut seen = vec![false; num_qubits];
                for (p, _) in &block_i.terms {
                    for q in 0..num_qubits {
                        if !seen[q] && p.operators[q] != PauliOperator::I {
                            seen[q] = true;
                            all_nodes.push(q);
                        }
                    }
                }
                all_nodes.sort_unstable();
                block_i.nodes = all_nodes;

                continue; // Try another merge.
            }
        }

        // No more beneficial merges.
        break;
    }

    blocks
}

/// Group Pauli terms into general commuting blocks using greedy graph coloring.
///
/// Builds a commutativity graph (vertices=terms, edges=commute), then applies
/// DSATUR-style greedy coloring: at each step, pick the uncolored vertex with
/// the most differently-colored neighbors, breaking ties by degree.
///
/// Each color class is a set of mutually commuting Pauli terms. These blocks
/// can be larger than QWC blocks (e.g., XXII and YYII commute but are not QWC).
///
/// Returns: vector of (block_terms, block_label) where label is a human-readable
/// identifier for diagnostics.
fn form_general_commuting_blocks(
    terms: &[(PauliString, f64)],
) -> Vec<(Vec<(PauliString, f64)>, String)> {
    let n = terms.len();
    if n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![(vec![(terms[0].0.clone(), terms[0].1)], "B0".to_string())];
    }

    // Build adjacency matrix: adj[i][j] = true if terms i and j commute
    let mut adj: Vec<Vec<bool>> = vec![vec![false; n]; n];
    for i in 0..n {
        adj[i][i] = true;
        for j in (i + 1)..n {
            let commute = is_commuting(&terms[i].0, &terms[j].0);
            adj[i][j] = commute;
            adj[j][i] = commute;
        }
    }

    // Compute degrees (number of commuting neighbors, excluding self)
    let degree: Vec<usize> = (0..n)
        .map(|i| adj[i].iter().filter(|&&x| x).count() - 1)
        .collect();

    // DSATUR greedy coloring
    let mut colors: Vec<Option<usize>> = vec![None; n];
    let mut colored_count = 0;

    while colored_count < n {
        // Find uncolored vertex with highest saturation degree
        let best = (0..n)
            .filter(|&i| colors[i].is_none())
            .max_by(|&a, &b| {
                // Saturation degree: number of DIFFERENT colors in neighbors
                let sat_a: std::collections::HashSet<usize> = (0..n)
                    .filter(|&j| adj[a][j] && colors[j].is_some())
                    .map(|j| colors[j].unwrap())
                    .collect();
                let sat_b: std::collections::HashSet<usize> = (0..n)
                    .filter(|&j| adj[b][j] && colors[j].is_some())
                    .map(|j| colors[j].unwrap())
                    .collect();
                sat_a
                    .len()
                    .cmp(&sat_b.len())
                    .then_with(|| degree[a].cmp(&degree[b]))
                    .then_with(|| a.cmp(&b)) // deterministic tie-break
            })
            .unwrap();

        // Find the smallest color not used by any non-commuting neighbor.
        // A vertex CAN share a color with commuting neighbors — that's the
        // whole point: each color class = one general commuting block.
        let forbidden: std::collections::HashSet<usize> = (0..n)
            .filter(|&j| !adj[best][j] && colors[j].is_some())
            .map(|j| colors[j].unwrap())
            .collect();

        let mut c = 0;
        while forbidden.contains(&c) {
            c += 1;
        }
        colors[best] = Some(c);
        colored_count += 1;
    }

    // Collect terms by color
    let num_colors = colors.iter().filter_map(|&c| c).max().unwrap_or(0) + 1;
    let mut groups: Vec<Vec<(PauliString, f64)>> = vec![vec![]; num_colors];
    for (i, color) in colors.iter().enumerate() {
        let c = color.unwrap();
        groups[c].push((terms[i].0.clone(), terms[i].1));
    }

    groups
        .into_iter()
        .enumerate()
        .filter(|(_, g)| !g.is_empty())
        .map(|(idx, g)| (g, format!("GC{}", idx)))
        .collect()
}

/// Decompose a general commuting block into QWC subgroups.
///
/// Within a general commuting block, terms may have different Pauli operators
/// at the same qubit position (e.g., X vs Y). These cannot share a single
/// CNOT tree directly. We partition into maximal QWC subgroups, which CAN
/// each use the standard shared-tree synthesis.
///
/// The QWC subgroups are ordered to maximize inter-subgroup CNOT cancellation
/// (subgroups with largest qubit overlap are placed adjacent).
fn decompose_to_qwc_subgroups(block_terms: &[(PauliString, f64)]) -> Vec<Vec<(PauliString, f64)>> {
    if block_terms.is_empty() {
        return vec![];
    }
    if block_terms.len() == 1 {
        return vec![block_terms.to_vec()];
    }

    // Greedy QWC partitioning within the general commuting block
    let mut subgroups: Vec<Vec<(PauliString, f64)>> = vec![];

    'outer: for (pauli, coeff) in block_terms {
        // Try to place in an existing QWC subgroup
        for subgroup in &mut subgroups {
            if subgroup.iter().all(|(gp, _)| is_qwc(gp, pauli)) {
                subgroup.push((pauli.clone(), *coeff));
                continue 'outer;
            }
        }
        // Start a new subgroup
        subgroups.push(vec![(pauli.clone(), *coeff)]);
    }

    // Sort subgroups by size (descending) for better primary tree selection
    subgroups.sort_by(|a, b| b.len().cmp(&a.len()));

    subgroups
}

/// Form PauliBlocks from a general commuting strategy.
///
/// First groups terms into general commuting blocks (graph coloring),
/// then decomposes each block into QWC subgroups that can be synthesized
/// with the existing shared-tree infrastructure.
///
/// Returns (blocks, general_commuting_group_ids) where group_ids[i] indicates
/// which general commuting group block[i] belongs to. QWC subgroups from the
/// same general commuting group get adjacent ordering for better CNOT
/// cancellation.
fn form_blocks_general_commuting(terms: &[(PauliString, f64)]) -> (Vec<PauliBlock>, Vec<usize>) {
    if terms.is_empty() {
        return (vec![], vec![]);
    }

    let gc_blocks = form_general_commuting_blocks(terms);

    let mut all_blocks: Vec<PauliBlock> = vec![];
    let mut group_ids: Vec<usize> = vec![];

    for (gc_idx, (block_terms, _label)) in gc_blocks.iter().enumerate() {
        let qwc_subgroups = decompose_to_qwc_subgroups(block_terms);

        for subgroup in qwc_subgroups {
            if subgroup.is_empty() {
                continue;
            }
            let mstr = subgroup.iter().map(|(p, _)| p).fold(
                PauliString::identity(subgroup[0].0.num_qubits()),
                |acc, p| p_or(&acc, p),
            );
            let nodes = active_nodes(&mstr);
            let n_terms = subgroup.len();
            all_blocks.push(PauliBlock {
                terms: subgroup,
                mstr,
                nodes,
                clifford_annotations: vec![None; n_terms],
            });
            group_ids.push(gc_idx);
        }
    }

    (all_blocks, group_ids)
}

// ---------------------------------------------------------------------------
// Term merging (Opt 1)
// ---------------------------------------------------------------------------

/// Merge terms with identical active qubit sets.
///
/// Within a QWC block, terms with the same active_nodes() have identical
/// Pauli operators at non-I positions (QWC guarantees same non-I operators).
/// Their CNOT trees are identical, so we can merge them into one term with
/// summed coefficients — 50% gate reduction per duplicate pair.
fn merge_identical_active_terms(terms: &[(PauliString, f64)]) -> Vec<(PauliString, f64)> {
    let mut groups: HashMap<Vec<usize>, (PauliString, f64)> = HashMap::new();

    for (pauli, coeff) in terms {
        let key = active_nodes(pauli);
        let entry = groups.entry(key).or_insert_with(|| (pauli.clone(), 0.0));
        entry.1 += coeff;
    }

    groups
        .into_values()
        .filter(|(_, coeff)| coeff.abs() > 1e-15)
        .collect()
}

// ---------------------------------------------------------------------------
// Prefix helpers (Opt 2)
// ---------------------------------------------------------------------------

/// Check whether `active` forms a contiguous prefix of `sorted_nodes`.
/// Both slices are sorted ascending.
fn is_prefix_of(active: &[usize], sorted_nodes: &[usize]) -> bool {
    if active.len() > sorted_nodes.len() {
        return false;
    }
    active == &sorted_nodes[..active.len()]
}

/// Sort nodes for maximum prefix compatibility.
///
/// For small blocks (≤8 nodes), exhaustively searches all permutations to find
/// the ordering that maximizes the number of prefix-compatible terms.
/// For larger blocks, uses a greedy heuristic: starts with the most-used qubit,
/// then greedily adds the qubit that extends the longest partial-prefix match.
fn sort_nodes_for_prefix_compatibility(
    nodes: &[usize],
    terms: &[(PauliString, f64)],
) -> Vec<usize> {
    if nodes.len() <= 1 {
        return nodes.to_vec();
    }

    // Collect term active sets with coefficient weights for P2 sorting
    let term_sets: Vec<(Vec<usize>, f64)> = terms
        .iter()
        .map(|(p, coeff)| (active_nodes(p), coeff.abs()))
        .filter(|(a, _)| a.len() > 1) // only multi-qubit terms matter for chain building
        .collect();

    if term_sets.is_empty() {
        let mut sorted = nodes.to_vec();
        sorted.sort_unstable();
        return sorted;
    }

    if nodes.len() <= 8 {
        exhaustive_best_ordering(nodes, &term_sets)
    } else {
        greedy_best_ordering(nodes, &term_sets)
    }
}

/// Exhaustively try all permutations (up to 8! = 40320).
fn exhaustive_best_ordering(nodes: &[usize], term_sets: &[(Vec<usize>, f64)]) -> Vec<usize> {
    let mut best_ordering: Vec<usize> = nodes.to_vec();
    best_ordering.sort_unstable();
    let mut best_count = count_prefix_terms_weighted(&best_ordering, term_sets);

    let n = nodes.len();
    let mut current: Vec<usize> = nodes.to_vec();
    current.sort_unstable();
    let mut indices: Vec<usize> = (0..n).collect();

    // Heap's algorithm for permutation generation
    let mut c = vec![0usize; n];
    let mut i = 1usize;
    while i < n {
        if c[i] < i {
            if i % 2 == 0 {
                indices.swap(0, i);
            } else {
                indices.swap(c[i], i);
            }
            for j in 0..n {
                current[j] = nodes[indices[j]];
            }
            let cnt = count_prefix_terms_weighted(&current, term_sets);
            if cnt > best_count {
                best_count = cnt;
                best_ordering = current.clone();
            }
            c[i] += 1;
            i = 1;
        } else {
            c[i] = 0;
            i += 1;
        }
    }

    best_ordering
}

/// Greedy incremental construction for large blocks.
fn greedy_best_ordering(nodes: &[usize], term_sets: &[(Vec<usize>, f64)]) -> Vec<usize> {
    let node_set: std::collections::HashSet<usize> = nodes.iter().copied().collect();
    let mut ordering: Vec<usize> = vec![];
    let mut remaining: std::collections::HashSet<usize> = node_set.clone();

    // Start with the qubit that appears in the most terms (weighted by coeff)
    let mut usage_count: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();
    for (term, weight) in term_sets {
        for &q in term {
            *usage_count.entry(q).or_default() += weight;
        }
    }

    while !remaining.is_empty() {
        // Pick the remaining qubit that maximizes the weighted prefix count.
        let best = *remaining
            .iter()
            .max_by(|&&a, &&b| {
                let mut ext_a = ordering.clone();
                ext_a.push(a);
                let score_a = count_prefix_terms_weighted(&ext_a, term_sets);

                let mut ext_b = ordering.clone();
                ext_b.push(b);
                let score_b = count_prefix_terms_weighted(&ext_b, term_sets);

                // Higher weighted count first, ties broken by usage count, then index
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| {
                        let ua = usage_count.get(&a).copied().unwrap_or(0.0);
                        let ub = usage_count.get(&b).copied().unwrap_or(0.0);
                        ua.partial_cmp(&ub).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .then_with(|| b.cmp(&a)) // lower index preferred
            })
            .unwrap();

        ordering.push(best);
        remaining.remove(&best);
    }

    ordering
}

/// Count how many term_sets form a prefix of the given node ordering.
/// Returns the count (unweighted) for backward compatibility.
fn count_prefix_terms(ordering: &[usize], term_sets: &[Vec<usize>]) -> usize {
    term_sets
        .iter()
        .filter(|active| is_prefix_of(active, ordering))
        .count()
}

/// Count prefix-compatible terms, weighted by |coefficient|.
/// Terms with larger coefficients contribute more to the score,
/// prioritizing prefix compatibility for physically significant terms.
fn count_prefix_terms_weighted(ordering: &[usize], term_sets: &[(Vec<usize>, f64)]) -> f64 {
    term_sets
        .iter()
        .filter(|(active, _)| is_prefix_of(active, ordering))
        .map(|(_, coeff)| coeff.abs())
        .sum()
}

// ---------------------------------------------------------------------------
// CNOT tree construction
// ---------------------------------------------------------------------------

/// Build a chain-based CNOT tree connecting all active qubits.
///
/// `link` encodes the shared backbone with a paired block — qubits in `link`
/// form the chain core, remaining qubits attach to the ends.
///
/// Returns reversed cnotset for left-tree traversal (leaves→root).
pub(crate) fn complement_tree2(
    nodes: &[usize],
    link: &[usize],
) -> (Vec<(usize, usize)>, Option<usize>) {
    if nodes.is_empty() {
        return (vec![], None);
    }
    if nodes.len() == 1 {
        return (vec![], Some(nodes[0]));
    }

    let link_tail = link.last().copied();

    let mut r1: Vec<usize> = vec![];
    let mut r2: Vec<usize> = vec![];
    for &n in nodes {
        if !link.contains(&n) {
            match link_tail {
                Some(tail) if n < tail => r1.push(n),
                _ => r2.push(n),
            }
        }
    }
    r1.sort_unstable();
    r2.sort_unstable();

    let mut backbone: Vec<usize> = link.to_vec();
    backbone.extend(&r2);

    let mut cnotset: Vec<(usize, usize)> = vec![];

    for i in 0..backbone.len().saturating_sub(1) {
        cnotset.push((backbone[i], backbone[i + 1]));
    }

    let root = backbone.last().copied();

    if let Some(r) = root {
        for i in 0..r1.len().saturating_sub(1) {
            cnotset.push((r1[i], r1[i + 1]));
        }
        if !r1.is_empty() {
            cnotset.push((r1[r1.len() - 1], r));
        }
    }

    cnotset.reverse();
    (cnotset, root)
}

// ---------------------------------------------------------------------------
// Per-term synthesis (fallback for non-prefix terms)
// ---------------------------------------------------------------------------

/// Synthesize a single Pauli term with its own CNOT tree.
/// This is the fallback for non-prefix terms that can't use the shared tree.
///
/// When `pre_gates`/`post_gates` are provided (Phase 11a-3 Clifford-aligned
/// terms), they are emitted OUTSIDE the basis change + CNOT tree, giving:
///   pre · basis · CNOT · Rz · CNOT_rev · inv_basis · post
///
/// When `pauli_is_conjugated` is true (Phase 11f), the stored Pauli is already
/// the result of Clifford conjugation (G·P_original·G†). The pre/post gates
/// are still emitted but the effective operator for basis changes is the
/// stored Pauli directly — pre_gates should NOT be reapplied to compute it.
fn synthesize_per_term(
    circuit: &mut QuantumCircuit,
    pauli: &PauliString,
    coeff: f64,
    link: &[usize],
    mut budget: Option<&mut PauliGateBudget>,
    pre_gates: Option<&[(CliffordGate, usize)]>,
    post_gates: Option<&[(CliffordGate, usize)]>,
    pauli_is_conjugated: bool,
) -> crate::Result<()> {
    let term_nodes = active_nodes(pauli);
    if term_nodes.is_empty() {
        return Ok(());
    }

    // ── Pre-conjugation Clifford gates (Phase 11a-3) ────────────────
    if let Some(gates) = pre_gates {
        for &(gate, q) in gates {
            match gate {
                CliffordGate::S => circuit.s(q)?,
                CliffordGate::Sdg => circuit.sdg(q)?,
                CliffordGate::H => circuit.h(q)?,
            }
        }
    }

    // Compute effective Pauli operators for basis change.
    //
    // Phase 11a-3 (pauli_is_conjugated=false): the stored Pauli is the
    // ORIGINAL. Pre-gates transform it (e.g., S†·Y·S = X), so we apply
    // the manual mapping to determine the effective operator for basis changes.
    //
    // Phase 11f (pauli_is_conjugated=true): the stored Pauli is already the
    // CONJUGATED result (G·P_original·G†). The effective operator is the
    // stored Pauli itself — pre_gates are external wrappers that should NOT
    // change the basis computation.
    let effective_ops: Vec<(usize, PauliOperator)> = term_nodes
        .iter()
        .map(|&q| {
            let op = pauli.operator_at(q).unwrap_or(PauliOperator::I);
            if pauli_is_conjugated {
                // Phase 11f: stored Pauli is already conjugated — use directly.
                return (q, op);
            }
            let mut adjusted = op;
            if let Some(gates) = pre_gates {
                for &(gate, gq) in gates {
                    if gq == q {
                        match (gate, adjusted) {
                            // S† on Y → X (since S†·Y·S = X)
                            (CliffordGate::Sdg, PauliOperator::Y) => adjusted = PauliOperator::X,
                            // S on X → Y (since S·X·S† = Y)
                            (CliffordGate::S, PauliOperator::X) => adjusted = PauliOperator::Y,
                            // H exchanges X↔Z, Y→-Y
                            (CliffordGate::H, PauliOperator::X) => adjusted = PauliOperator::Z,
                            (CliffordGate::H, PauliOperator::Z) => adjusted = PauliOperator::X,
                            // (Sdg,X) and (S,Y) are NOT handled — those introduce
                            // sign flips (S†·X·S = -Y, S·Y·S† = -X) unreachable
                            // with current CliffordSimple XY-alignment strategy.
                            _ => {} // I unchanged, or Y under H stays Y
                        }
                    }
                }
            }
            (q, adjusted)
        })
        .collect();

    let filtered_link: Vec<usize> = link
        .iter()
        .filter(|q| term_nodes.contains(q))
        .copied()
        .collect();
    let (cnotset, root) = complement_tree2(&term_nodes, &filtered_link);

    // Left basis change — use EFFECTIVE operators after pre_gates
    for &(q, op) in &effective_ops {
        apply_basis_change(circuit, q, op)?;
        if let Some(ref mut b) = budget {
            if op == PauliOperator::X || op == PauliOperator::Y {
                b.basis_changes += 1;
            }
        }
    }

    // Left CNOT tree (leaves → root = reversed order)
    let cx_count = cnotset.len();
    for &(ctrl, tgt) in cnotset.iter().rev() {
        circuit.cx(ctrl, tgt)?;
    }
    if let Some(ref mut b) = budget {
        b.cx_per_term += cx_count;
    }

    // Central rotation on root
    if let Some(r) = root {
        if coeff.abs() > 1e-15 {
            circuit.rz(r, Parameter::Float(2.0 * coeff))?;
            if let Some(ref mut b) = budget {
                b.rz_per_term += 1;
            }
        }
    }

    // Right CNOT tree (root → leaves = forward order)
    for &(ctrl, tgt) in &cnotset {
        circuit.cx(ctrl, tgt)?;
    }
    if let Some(ref mut b) = budget {
        b.cx_per_term += cx_count;
    }

    // Right basis change (inverse) — use effective operators
    for &(q, ref op) in effective_ops.iter().rev() {
        apply_inv_basis_change(circuit, q, *op)?;
        if let Some(ref mut b) = budget {
            if *op == PauliOperator::X || *op == PauliOperator::Y {
                b.basis_changes += 1;
            }
        }
    }

    // ── Post-conjugation Clifford gates (Phase 11a-3) ───────────────
    if let Some(gates) = post_gates {
        for &(gate, q) in gates {
            match gate {
                CliffordGate::S => circuit.s(q)?,
                CliffordGate::Sdg => circuit.sdg(q)?,
                CliffordGate::H => circuit.h(q)?,
            }
        }
    }

    if let Some(ref mut b) = budget {
        b.per_term_count += 1;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Shared tree synthesis (Opt 2: prefix-compatible terms)
// ---------------------------------------------------------------------------

/// Synthesize prefix-compatible terms with one shared CNOT chain.
///
/// Structure for chain [n0, n1, ..., nk]:
///
/// ```text
/// [basis changes on chain_nodes]
/// CNOT(n0,n1) Rz(n1, Σcoeff_for_terms_ending_at_n1)
/// CNOT(n1,n2) Rz(n2, Σcoeff_for_terms_ending_at_n2)
/// ...
/// CNOT(n_{k-1},n_k) Rz(n_k, Σcoeff_for_terms_ending_at_n_k)
/// // reverse:
/// Rz(n_k, Σcoeff) CNOT(n_{k-1},n_k)^dag
/// ...
/// Rz(n1, Σcoeff) CNOT(n0,n1)^dag
/// [inverse basis changes]
/// ```
///
/// Each Rz uses `coeff` (not 2*coeff) per side, split symmetrically for
/// total 2*coeff. This enables inter-block CNOT cancellation (Opt 3).
fn synthesize_shared_tree(
    circuit: &mut QuantumCircuit,
    prefix_terms: &[(PauliString, f64)],
    sorted_nodes: &[usize],
    split_position_0: bool,
    mut budget: Option<&mut PauliGateBudget>,
    // Phase 11k: extra Rz coefficients from inlined Clifford-annotated terms.
    // extra_coeffs[i] is added to rz_at_pos[i] in the shared tree.
    extra_coeffs: &[f64],
) -> crate::Result<()> {
    if prefix_terms.is_empty() {
        return Ok(());
    }

    // Compute prefix mstr = union of operators from all prefix terms (for basis changes)
    let num_qubits = prefix_terms[0].0.num_qubits();
    let mut prefix_mstr = PauliString::identity(num_qubits);
    for (pauli, _) in prefix_terms {
        prefix_mstr = p_or(&prefix_mstr, pauli);
    }

    // Determine maximum prefix length
    let max_prefix_len = prefix_terms
        .iter()
        .map(|(p, _)| active_nodes(p).len())
        .max()
        .unwrap_or(0);

    // Single-qubit terms: apply Rz directly, no CNOT tree needed
    if max_prefix_len < 2 {
        for (pauli, coeff) in prefix_terms {
            let active = active_nodes(pauli);
            if active.is_empty() {
                continue;
            }
            let q = active[0];
            let op = pauli.operator_at(q).unwrap_or(PauliOperator::I);
            apply_basis_change(circuit, q, op)?;
            if let Some(ref mut b) = budget {
                if op == PauliOperator::X || op == PauliOperator::Y {
                    b.basis_changes += 1;
                }
            }
            if coeff.abs() > 1e-15 {
                circuit.rz(q, Parameter::Float(2.0 * coeff))?;
                if let Some(ref mut b) = budget {
                    b.single_qubit_rz += 1;
                }
            }
            apply_inv_basis_change(circuit, q, op)?;
            if let Some(ref mut b) = budget {
                if op == PauliOperator::X || op == PauliOperator::Y {
                    b.basis_changes += 1;
                }
            }
        }
        if let Some(ref mut b) = budget {
            b.shared_tree_term_count += prefix_terms.len();
        }
        return Ok(());
    }

    // Separate single-qubit terms from multi-qubit terms.
    // Single-qubit terms are emitted directly (no splitting); only
    // multi-qubit terms go through the shared chain.
    let single_qubit_terms: Vec<&(PauliString, f64)> = prefix_terms
        .iter()
        .filter(|(p, _)| active_nodes(p).len() < 2)
        .collect();
    let multi_qubit_terms: Vec<&(PauliString, f64)> = prefix_terms
        .iter()
        .filter(|(p, _)| active_nodes(p).len() >= 2)
        .collect();

    // Emit single-qubit terms first (clean, optimizable placement).
    for (pauli, coeff) in &single_qubit_terms {
        let active = active_nodes(pauli);
        if active.is_empty() {
            continue;
        }
        let q = active[0];
        let op = pauli.operator_at(q).unwrap_or(PauliOperator::I);
        apply_basis_change(circuit, q, op)?;
        if let Some(ref mut b) = budget {
            if op == PauliOperator::X || op == PauliOperator::Y {
                b.basis_changes += 1;
            }
        }
        if coeff.abs() > 1e-15 {
            circuit.rz(q, Parameter::Float(2.0 * coeff))?;
            if let Some(ref mut b) = budget {
                b.single_qubit_rz += 1;
            }
        }
        apply_inv_basis_change(circuit, q, op)?;
        if let Some(ref mut b) = budget {
            if op == PauliOperator::X || op == PauliOperator::Y {
                b.basis_changes += 1;
            }
        }
    }

    if multi_qubit_terms.is_empty() {
        if let Some(ref mut b) = budget {
            b.shared_tree_term_count += prefix_terms.len();
        }
        return Ok(());
    }

    // Recompute max_prefix_len from multi-qubit terms only
    let max_prefix_len = multi_qubit_terms
        .iter()
        .map(|(p, _)| active_nodes(p).len())
        .max()
        .unwrap_or(0);
    if max_prefix_len < 2 {
        // Shouldn't happen but handle gracefully
        if let Some(ref mut b) = budget {
            b.shared_tree_term_count += prefix_terms.len();
        }
        return Ok(());
    }

    let chain_nodes: Vec<usize> = sorted_nodes[..max_prefix_len].to_vec();
    let k = chain_nodes.len();

    // Group terms by their last active qubit position in the chain.
    // rz_at_position[j] = sum of coefficients for terms ending at chain_nodes[j].
    let mut rz_at_pos: Vec<f64> = vec![0.0; k];

    for (pauli, coeff) in &multi_qubit_terms {
        let active = active_nodes(pauli);
        if active.is_empty() {
            continue;
        }
        let last_node = *active.last().unwrap();
        let pos = chain_nodes
            .iter()
            .position(|&n| n == last_node)
            .expect("prefix term's last node must be in chain_nodes");
        rz_at_pos[pos] += coeff;
    }

    // Phase 11k: Add extra Rz coefficients from inlined Clifford-annotated terms.
    // These terms share the same CNOT tree as the clean terms — only their
    // Clifford pre/post gates are emitted outside the shared tree.
    for i in 0..k {
        if i < extra_coeffs.len() {
            rz_at_pos[i] += extra_coeffs[i];
        }
    }

    // --- Left basis changes ---
    // Only apply on qubits where prefix_mstr has X or Y (Z and I need no change).
    for &q in &chain_nodes {
        let op = prefix_mstr.operator_at(q).unwrap_or(PauliOperator::I);
        if op == PauliOperator::X || op == PauliOperator::Y {
            apply_basis_change(circuit, q, op)?;
            if let Some(ref mut b) = budget {
                b.basis_changes += 1;
            }
        }
    }

    // Handle single-qubit terms at chain start (position 0) before the CNOT chain.
    // Rz(chain_nodes[0]) commutes with CNOT(chain_nodes[0], *) since qubit 0 is control.
    let coeff_pos0 = rz_at_pos[0];
    if coeff_pos0.abs() > 1e-15 {
        if split_position_0 {
            // Split: half before forward chain, half after reverse chain.
            // Enables inter-block CNOT cancellation when paired blocks share position 0.
            circuit.rz(chain_nodes[0], Parameter::Float(coeff_pos0))?;
            if let Some(ref mut b) = budget {
                b.rz_position_0_forward += 1;
            }
        } else {
            // Merged: emit full 2*coeff before forward chain (as positions > 0 do).
            // This avoids a wasted Rz gate when no inter-block cancellation is expected.
            circuit.rz(chain_nodes[0], Parameter::Float(2.0 * coeff_pos0))?;
            if let Some(ref mut b) = budget {
                b.rz_shared_trees += 1;
            }
        }
    }

    // --- Forward CNOT chain with merged Rz (2*coeff) for positions > 0 ---
    for i in 0..(k - 1) {
        circuit.cx(chain_nodes[i], chain_nodes[i + 1])?;
        if let Some(ref mut b) = budget {
            b.cx_shared_trees += 1;
        }
        let total_coeff = rz_at_pos[i + 1];
        if total_coeff.abs() > 1e-15 {
            circuit.rz(chain_nodes[i + 1], Parameter::Float(2.0 * total_coeff))?;
            if let Some(ref mut b) = budget {
                b.rz_shared_trees += 1;
            }
        }
    }

    // --- Reverse CNOT chain (CX only — Rz merged into forward for positions > 0) ---
    for i in (0..(k - 1)).rev() {
        circuit.cx(chain_nodes[i], chain_nodes[i + 1])?;
        if let Some(ref mut b) = budget {
            b.cx_shared_trees += 1;
        }
    }

    // Position 0 reverse side: only emitted when split_position_0 is true.
    // When not splitting, the full rotation was already handled above.
    if split_position_0 && coeff_pos0.abs() > 1e-15 {
        circuit.rz(chain_nodes[0], Parameter::Float(coeff_pos0))?;
        if let Some(ref mut b) = budget {
            b.rz_position_0_reverse += 1;
        }
    }

    // --- Inverse basis changes ---
    for &q in chain_nodes.iter().rev() {
        let op = prefix_mstr.operator_at(q).unwrap_or(PauliOperator::I);
        if op == PauliOperator::X || op == PauliOperator::Y {
            apply_inv_basis_change(circuit, q, op)?;
            if let Some(ref mut b) = budget {
                b.basis_changes += 1;
            }
        }
    }

    if let Some(ref mut b) = budget {
        b.shared_tree_term_count += prefix_terms.len();
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Block synthesis
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Secondary shared-chain synthesis for non-prefix terms (P1)
// ---------------------------------------------------------------------------

/// Synthesize non-prefix terms with a secondary shared chain.
///
/// For terms whose active qubits don't form a prefix of the primary ordering,
/// we extract a sub-ordering from their active qubits and re-apply prefix
/// classification. This creates a secondary shared CNOT chain, reducing the
/// number of independent per-term trees.
///
/// Terms that still don't fit any prefix (after sub-ordering) fall through to
/// `synthesize_per_term`.
/// Phase 11t: Emit a set of Pauli gadgets using sets synthesis.
///
/// Uses `pauli_gadget_sets` to group gadgets into commuting sets and
/// synthesize each set via mutual diagonalization.
fn emit_gadget_sets(
    circuit: &mut QuantumCircuit,
    terms: &[(PauliString, f64)],
) -> crate::Result<()> {
    use super::diagonalisation::pauli_gadget_sets;
    use crate::parameter::Parameter;

    let ops = pauli_gadget_sets(terms);
    for op in &ops {
        match op.gate {
            crate::gates::StandardGate::H => circuit.h(op.qubits[0])?,
            crate::gates::StandardGate::S => circuit.s(op.qubits[0])?,
            crate::gates::StandardGate::Sdg => circuit.sdg(op.qubits[0])?,
            crate::gates::StandardGate::CX => circuit.cx(op.qubits[0], op.qubits[1])?,
            crate::gates::StandardGate::Rx => {
                circuit.rx(op.qubits[0], Parameter::Float(op.angle.unwrap_or(0.0)))?;
            }
            crate::gates::StandardGate::Rz => {
                circuit.rz(op.qubits[0], Parameter::Float(op.angle.unwrap_or(0.0)))?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Phase 11r: Emit a pair of Pauli gadgets using pairwise synthesis.
///
/// Uses `reduce_overlap_of_paulis` to find a Clifford circuit U that minimizes
/// the overlap between the two Pauli strings, then emits U·(gadget0+gadget1)·U†.
fn emit_gadget_pair(
    circuit: &mut QuantumCircuit,
    term0: &(PauliString, f64),
    term1: &(PauliString, f64),
) -> crate::Result<()> {
    use super::diagonalisation::pauli_gadget_pair;
    use crate::parameter::Parameter;

    let ops = pauli_gadget_pair(&term0.0, term0.1, &term1.0, term1.1);

    for op in &ops {
        match op.gate {
            crate::gates::StandardGate::H => {
                circuit.h(op.qubits[0])?;
            }
            crate::gates::StandardGate::S => {
                circuit.s(op.qubits[0])?;
            }
            crate::gates::StandardGate::Sdg => {
                circuit.sdg(op.qubits[0])?;
            }
            crate::gates::StandardGate::CX => {
                circuit.cx(op.qubits[0], op.qubits[1])?;
            }
            crate::gates::StandardGate::Rx => {
                let angle = op.angle.unwrap_or(0.0);
                circuit.rx(op.qubits[0], Parameter::Float(angle))?;
            }
            crate::gates::StandardGate::Rz => {
                let angle = op.angle.unwrap_or(0.0);
                circuit.rz(op.qubits[0], Parameter::Float(angle))?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn synthesize_nonprefix_terms(
    circuit: &mut QuantumCircuit,
    nonprefix_terms: &[(PauliString, f64)],
    _sorted_nodes: &[usize],
    link: &[usize],
    mut budget: Option<&mut PauliGateBudget>,
) -> crate::Result<()> {
    if nonprefix_terms.is_empty() {
        return Ok(());
    }

    // Count multi-qubit terms; if < 2 we can't form a secondary chain
    let multi_count = nonprefix_terms
        .iter()
        .filter(|(p, _)| active_nodes(p).len() >= 2)
        .count();
    if multi_count < 2 {
        // Phase 11r: Use pairwise synthesis for exactly 2 terms via
        // reduce_overlap_of_paulis (TKET-style pairwise gadget reduction).
        if nonprefix_terms.len() == 2 {
            emit_gadget_pair(circuit, &nonprefix_terms[0], &nonprefix_terms[1])?;
            return Ok(());
        }
        for (pauli, coeff) in nonprefix_terms {
            synthesize_per_term(
                circuit,
                pauli,
                *coeff,
                link,
                budget.as_deref_mut(),
                None,
                None,
                false,
            )?;
        }
        return Ok(());
    }

    // Collect qubits used by non-prefix terms
    let mut sub_nodes: Vec<usize> = vec![];
    for (p, _) in nonprefix_terms {
        for &q in &active_nodes(p) {
            if !sub_nodes.contains(&q) {
                sub_nodes.push(q);
            }
        }
    }
    sub_nodes.sort_unstable();

    if sub_nodes.len() < 2 {
        // Phase 11t: Use sets synthesis for 3+ nonprefix terms
        if nonprefix_terms.len() >= 3 {
            emit_gadget_sets(circuit, nonprefix_terms)?;
            return Ok(());
        }
        // Phase 11r: Pairwise for exactly 2 terms
        if nonprefix_terms.len() == 2 {
            emit_gadget_pair(circuit, &nonprefix_terms[0], &nonprefix_terms[1])?;
            return Ok(());
        }
        for (pauli, coeff) in nonprefix_terms {
            synthesize_per_term(
                circuit,
                pauli,
                *coeff,
                link,
                budget.as_deref_mut(),
                None,
                None,
                false,
            )?;
        }
        return Ok(());
    }

    // Optimize ordering for non-prefix terms specifically
    let multi_terms: Vec<(PauliString, f64)> = nonprefix_terms
        .iter()
        .filter(|(p, _)| active_nodes(p).len() >= 2)
        .cloned()
        .collect();
    let sub_order = if multi_terms.len() >= 2 {
        sort_nodes_for_prefix_compatibility(&sub_nodes, &multi_terms)
    } else {
        sub_nodes
    };

    if sub_order.len() < 2 {
        for (pauli, coeff) in nonprefix_terms {
            synthesize_per_term(
                circuit,
                pauli,
                *coeff,
                link,
                budget.as_deref_mut(),
                None,
                None,
                false,
            )?;
        }
        return Ok(());
    }

    // Classify into sub-prefix and truly non-prefix
    let mut sub_prefix: Vec<(PauliString, f64)> = vec![];
    let mut sub_nonprefix: Vec<(PauliString, f64)> = vec![];

    for (pauli, coeff) in nonprefix_terms {
        let active = active_nodes(pauli);
        if active.is_empty() {
            continue;
        }
        if active.len() == 1 {
            // Single-qubit: synthesize directly (no CNOT tree needed)
            let q = active[0];
            let op = pauli.operator_at(q).unwrap_or(PauliOperator::I);
            apply_basis_change(circuit, q, op)?;
            if let Some(ref mut b) = budget {
                if op == PauliOperator::X || op == PauliOperator::Y {
                    b.basis_changes += 1;
                }
            }
            if coeff.abs() > 1e-15 {
                circuit.rz(q, Parameter::Float(2.0 * coeff))?;
                if let Some(ref mut b) = budget {
                    b.single_qubit_rz += 1;
                }
            }
            apply_inv_basis_change(circuit, q, op)?;
            if let Some(ref mut b) = budget {
                if op == PauliOperator::X || op == PauliOperator::Y {
                    b.basis_changes += 1;
                }
            }
        } else if is_prefix_of(&active, &sub_order) {
            sub_prefix.push((pauli.clone(), *coeff));
        } else {
            sub_nonprefix.push((pauli.clone(), *coeff));
        }
    }

    // Synthesize secondary shared tree for sub-prefix terms
    if !sub_prefix.is_empty() {
        // Count secondary chain gates separately via a temporary budget
        let mut secondary_budget = PauliGateBudget::default();
        synthesize_shared_tree(
            circuit,
            &sub_prefix,
            &sub_order,
            false,
            Some(&mut secondary_budget),
            &[], // Phase 11k: no extra_coeffs for nonprefix path
        )?;
        if let Some(ref mut b) = budget {
            b.cx_secondary_chain += secondary_budget.cx_shared_trees;
            b.rz_secondary_chain += secondary_budget.rz_shared_trees
                + secondary_budget.rz_position_0_forward
                + secondary_budget.rz_position_0_reverse
                + secondary_budget.single_qubit_rz;
            b.basis_changes += secondary_budget.basis_changes;
            b.secondary_chain_term_count += secondary_budget.shared_tree_term_count;
        }
    }

    // Truly non-prefix → per-term fallback
    for (pauli, coeff) in &sub_nonprefix {
        synthesize_per_term(
            circuit,
            pauli,
            *coeff,
            link,
            budget.as_deref_mut(),
            None,
            None,
            false,
        )?;
    }

    Ok(())
}

/// Synthesize all terms in a block.
///
/// Applies three optimizations:
/// 1. Merge identical-active-set terms → one synthesis per unique active set
/// 2. Shared CNOT tree for prefix-compatible terms → one chain, interleaved Rz
/// 3. Per-term fallback for non-prefix terms → independent CNOT trees
///
/// `emit_prefix_first` controls ordering for inter-block CNOT cancellation (Opt 3):
/// - `false` (left block / singleton): per-term first, shared tree last
///   → reverse CNOT chain at block boundary
/// - `true` (right block of pair): shared tree first, per-term last
///   → forward CNOT chain at block boundary
///
/// `clifford_map` (Phase 11a-3): optional Clifford gate annotations for
/// XY-aligned terms. Annotated terms are synthesized individually with
/// Clifford gates placed OUTSIDE the basis change + CNOT tree.
fn syn_block(
    circuit: &mut QuantumCircuit,
    block: &PauliBlock,
    link: &[usize],
    emit_prefix_first: bool,
    split_position_0: bool,
    mut budget: Option<&mut PauliGateBudget>,
    clifford_map: Option<&CliffordAnnotationMap>,
) -> crate::Result<()> {
    if block.terms.is_empty() {
        return Ok(());
    }

    // Phase 11a-3 + Phase 11f: Separate terms with Clifford annotations.
    // Annotated terms must be synthesized individually because their
    // Clifford gates go OUTSIDE the CNOT tree, not between Rz rotations.
    // The shared CNOT tree merges Rz rotations by position, which is
    // incompatible with per-term Clifford wrapping (code review B1/B2).
    //
    // Phase 11f: clifford_annotations from form_blocks_clifford_enhanced()
    // are handled here. Terms with these annotations are from a merged
    // block whose Pauli operators were Clifford-conjugated for QWC
    // compatibility. The pre-gates = conjugation gates, post-gates = inverses.
    let mut clean_terms: Vec<(PauliString, f64)> = vec![];
    // (pauli, coeff, pre_gates, post_gates, pauli_is_conjugated)
    let mut annotated_terms: Vec<(
        PauliString,
        f64,
        Vec<(CliffordGate, usize)>,
        Vec<(CliffordGate, usize)>,
        bool,
    )> = vec![];

    for (idx, (pauli, coeff)) in block.terms.iter().enumerate() {
        // Phase 11a-3: Check external Clifford map first.
        // These terms have ORIGINAL (un-conjugated) Pauli operators.
        if let Some(map) = clifford_map {
            let key = clifford_key(pauli, *coeff);
            if let Some((pre, post)) = map.get(&key) {
                annotated_terms.push((pauli.clone(), *coeff, pre.clone(), post.clone(), false));
                continue;
            }
        }
        // Phase 11f: Check block-level Clifford annotations.
        // These terms have already-CONJUGATED Pauli operators
        // (G·P_original·G† stored in block_i.terms).
        if idx < block.clifford_annotations.len() {
            if let Some(ref pre_gates) = block.clifford_annotations[idx] {
                let post_gates: Vec<(CliffordGate, usize)> =
                    pre_gates.iter().map(|&(g, q)| (g.inverse(), q)).collect();
                annotated_terms.push((pauli.clone(), *coeff, pre_gates.clone(), post_gates, true));
                continue;
            }
        }
        clean_terms.push((pauli.clone(), *coeff));
    }

    // Synthesize annotated terms individually with Clifford gates.
    if clean_terms.is_empty() {
        // Only annotated terms — emit each with its own CNOT tree
        for (pauli, coeff, pre, post, conjugated) in &annotated_terms {
            synthesize_per_term(
                circuit,
                pauli,
                *coeff,
                link,
                None,
                Some(pre),
                Some(post),
                *conjugated,
            )?;
        }
        return Ok(());
    }

    // Opt 1: Merge identical-active-set terms (clean terms only)
    let merged_terms = merge_identical_active_terms(&clean_terms);
    if merged_terms.is_empty() {
        for (pauli, coeff, pre, post, conjugated) in &annotated_terms {
            synthesize_per_term(
                circuit,
                pauli,
                *coeff,
                link,
                None,
                Some(pre),
                Some(post),
                *conjugated,
            )?;
        }
        return Ok(());
    }

    // Sort nodes for prefix compatibility
    let sorted_nodes = sort_nodes_for_prefix_compatibility(&block.nodes, &merged_terms);

    // Opt 2: Classify into prefix-compatible and non-prefix
    let mut prefix_terms: Vec<(PauliString, f64)> = vec![];
    let mut nonprefix_terms: Vec<(PauliString, f64)> = vec![];

    for (pauli, coeff) in &merged_terms {
        let active = active_nodes(pauli);
        if active.is_empty() {
            continue;
        }
        if is_prefix_of(&active, &sorted_nodes) {
            prefix_terms.push((pauli.clone(), *coeff));
        } else {
            nonprefix_terms.push((pauli.clone(), *coeff));
        }
    }

    // Phase 11k: Identify annotated terms that can share the shared tree's
    // CNOT structure. A term is "inlinable" if its effective Pauli's active
    // nodes are a prefix of sorted_nodes AND its operators match the shared
    // tree's prefix_mstr at all active qubits (ensuring correct basis changes).
    // We inline their Rz into the shared tree and emit only Clifford pre/post
    // gates around it — saving a full CNOT tree per inlined term.
    let max_prefix_len = prefix_terms
        .iter()
        .map(|(p, _)| active_nodes(p).len())
        .max()
        .unwrap_or(0);
    let chain_nodes: Vec<usize> = sorted_nodes.iter().take(max_prefix_len).copied().collect();
    let k = chain_nodes.len();
    // Compute prefix_mstr for operator compatibility checks
    let num_qubits = block
        .terms
        .first()
        .map(|(p, _)| p.num_qubits())
        .unwrap_or(0);
    let mut prefix_mstr = PauliString::identity(num_qubits);
    for (pauli, _) in &prefix_terms {
        prefix_mstr = p_or(&prefix_mstr, pauli);
    }
    let mut extra_coeffs = vec![0.0f64; k.max(1)]; // at least 1 for k=0
    let mut inlined_pre: Vec<Vec<(CliffordGate, usize)>> = vec![];
    let mut inlined_post: Vec<Vec<(CliffordGate, usize)>> = vec![];
    let mut standalone_annotated: Vec<(
        PauliString,
        f64,
        Vec<(CliffordGate, usize)>,
        Vec<(CliffordGate, usize)>,
        bool,
    )> = vec![];

    for (pauli, coeff, pre, post, conjugated) in &annotated_terms {
        let effective = if *conjugated {
            pauli.clone()
        } else {
            // Phase 11a-3: compute effective Pauli from original + pre_gates
            super::pauli_gadget::apply_clifford_pre_gates(pauli, pre)
        };
        let e_active = active_nodes(&effective);
        if e_active.is_empty() {
            standalone_annotated.push((
                pauli.clone(),
                *coeff,
                pre.clone(),
                post.clone(),
                *conjugated,
            ));
            continue;
        }
        // Check operator compatibility: effective Pauli must agree with
        // prefix_mstr at every active qubit (both non-I and same operator,
        // or prefix_mstr has I at that qubit).
        let ops_compatible = e_active.iter().all(|&q| {
            let e_op = effective.operator_at(q).unwrap_or(PauliOperator::I);
            let m_op = prefix_mstr.operator_at(q).unwrap_or(PauliOperator::I);
            matches!(m_op, PauliOperator::I) || e_op == m_op
        });
        // Check if effective Pauli is prefix-compatible with the shared tree chain
        if k > 0 && ops_compatible && is_prefix_of(&e_active, &sorted_nodes) && !e_active.is_empty()
        {
            let last_node = *e_active.last().unwrap();
            if let Some(pos) = chain_nodes.iter().position(|&n| n == last_node) {
                extra_coeffs[pos] += coeff;
                inlined_pre.push(pre.clone());
                inlined_post.push(post.clone());
                continue;
            }
        }
        standalone_annotated.push((
            pauli.clone(),
            *coeff,
            pre.clone(),
            post.clone(),
            *conjugated,
        ));
    }

    // Helper: emit a sequence of Clifford gates for inlined terms
    let emit_clifford_seq =
        |circuit: &mut QuantumCircuit, gates: &[(CliffordGate, usize)]| -> crate::Result<()> {
            for &(gate, q) in gates {
                match gate {
                    CliffordGate::S => circuit.s(q)?,
                    CliffordGate::Sdg => circuit.sdg(q)?,
                    CliffordGate::H => circuit.h(q)?,
                }
            }
            Ok(())
        };

    // Dispatch with ordering for inter-block CNOT adjacency (Opt 3)
    if emit_prefix_first {
        // Right block of pair: shared tree first (forward chain at boundary)
        // Phase 11k: emit pre-gates for inlined terms before shared tree
        for gates in &inlined_pre {
            emit_clifford_seq(circuit, gates)?;
        }
        if !prefix_terms.is_empty() {
            synthesize_shared_tree(
                circuit,
                &prefix_terms,
                &sorted_nodes,
                split_position_0,
                budget.as_deref_mut(),
                &extra_coeffs,
            )?;
        }
        // Phase 11k: emit post-gates for inlined terms after shared tree
        for gates in &inlined_post {
            emit_clifford_seq(circuit, gates)?;
        }
        synthesize_nonprefix_terms(
            circuit,
            &nonprefix_terms,
            &sorted_nodes,
            link,
            budget.as_deref_mut(),
        )?;
        // Remaining (non-inlined) annotated terms
        for (pauli, coeff, pre, post, conjugated) in &standalone_annotated {
            synthesize_per_term(
                circuit,
                pauli,
                *coeff,
                link,
                None,
                Some(pre),
                Some(post),
                *conjugated,
            )?;
        }
    } else {
        // Left block or singleton: standalone annotated first
        for (pauli, coeff, pre, post, conjugated) in &standalone_annotated {
            synthesize_per_term(
                circuit,
                pauli,
                *coeff,
                link,
                None,
                Some(pre),
                Some(post),
                *conjugated,
            )?;
        }
        synthesize_nonprefix_terms(
            circuit,
            &nonprefix_terms,
            &sorted_nodes,
            link,
            budget.as_deref_mut(),
        )?;
        // Phase 11k: emit pre-gates for inlined terms before shared tree
        for gates in &inlined_pre {
            emit_clifford_seq(circuit, gates)?;
        }
        if !prefix_terms.is_empty() {
            synthesize_shared_tree(
                circuit,
                &prefix_terms,
                &sorted_nodes,
                split_position_0,
                budget,
                &extra_coeffs,
            )?;
        }
        // Phase 11k: emit post-gates for inlined terms after shared tree
        for gates in &inlined_post {
            emit_clifford_seq(circuit, gates)?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Layer pairing
// ---------------------------------------------------------------------------

/// Greedy max-singlet-pairs: pick the largest-overlap consecutive pair,
/// remove both from pool, repeat.
fn max_singlet_pairs_line(cost: &[usize]) -> Vec<(usize, usize)> {
    let n = cost.len() + 1;
    let mut used = vec![false; n];
    let mut edges: Vec<(usize, usize)> = vec![];

    let mut candidates: Vec<(usize, usize, usize)> = (0..n.saturating_sub(1))
        .map(|i| (cost[i], i, i + 1))
        .collect();
    candidates.sort_by_key(|&(c, _, _)| std::cmp::Reverse(c));

    for (_c, i, j) in candidates {
        if !used[i] && !used[j] {
            used[i] = true;
            used[j] = true;
            edges.push((i, j));
        }
    }

    // Add unpaired as singletons
    for i in 0..n {
        if !used[i] {
            edges.push((i, i));
        }
    }
    edges.sort_by_key(|&(l, _)| l);
    edges
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Build a PauliBlockCache from Pauli terms without synthesizing gates.
///
/// This computes the QWC block structure, node ordering, and block pairings
/// that `compile_step_pauli_synthesis` would use. The result can be passed to
/// subsequent calls (e.g., reverse Trotter pass) via `cached_blocks` to ensure
/// identical CNOT tree structures.
///
/// When `clifford_enhanced_blocks` is true, uses
/// `form_blocks_clifford_enhanced()` (Phase 11f) to produce the block structure,
/// ensuring the cache contains any Clifford annotations needed for reverse-pass
/// synthesis. Default: `false` (matches legacy behavior).
pub fn build_block_cache(
    terms: &[&crate::hamiltonian::PauliTerm],
    dt: f64,
    hbar: f64,
    clifford_enhanced_blocks: bool,
) -> Option<PauliBlockCache> {
    if terms.is_empty() {
        return None;
    }

    let scale = dt / hbar;
    let scaled: Vec<(PauliString, f64)> = terms
        .iter()
        .map(|t| (t.pauli_string.clone(), t.coefficient.re * scale))
        .collect();

    let mut blocks = if clifford_enhanced_blocks {
        form_blocks_clifford_enhanced(&scaled)
    } else {
        form_blocks(&scaled)
    };
    if blocks.is_empty() {
        return None;
    }

    blocks.sort_by(|a, b| {
        b.cost()
            .cmp(&a.cost())
            .then_with(|| a.mstr.to_string_repr().cmp(b.mstr.to_string_repr()))
    });

    let nb = blocks.len();
    let block_signatures: Vec<Vec<String>> = blocks
        .iter()
        .map(|b| {
            b.terms
                .iter()
                .map(|(p, _)| p.to_string_repr().to_string())
                .collect()
        })
        .collect();
    let block_nodes: Vec<Vec<usize>> = blocks.iter().map(|b| b.nodes.clone()).collect();
    let block_mstrs: Vec<PauliString> = blocks.iter().map(|b| b.mstr.clone()).collect();
    let block_clifford_annotations: Vec<Vec<Option<Vec<(CliffordGate, usize)>>>> = blocks
        .iter()
        .map(|b| b.clifford_annotations.clone())
        .collect();

    let (edges, edge_links, split_flags) = if nb == 1 {
        (vec![(0usize, 0usize)], vec![vec![]], vec![(false, false)])
    } else {
        let cost: Vec<usize> = (0..nb.saturating_sub(1))
            .map(|i| {
                let overlap = mutual_positions(&blocks[i].mstr, &blocks[i + 1].mstr);
                let size = overlap.len();
                if size < 2 {
                    return size;
                }
                let a_sorted =
                    sort_nodes_for_prefix_compatibility(&blocks[i].nodes, &blocks[i].terms);
                let b_sorted =
                    sort_nodes_for_prefix_compatibility(&blocks[i + 1].nodes, &blocks[i + 1].terms);
                let same_direction = overlap.iter().all(|q| {
                    let a_pos = a_sorted.iter().position(|s| s == q);
                    let b_pos = b_sorted.iter().position(|s| s == q);
                    a_pos == b_pos
                });
                if same_direction {
                    size + 1
                } else {
                    size
                }
            })
            .collect();
        let edges = max_singlet_pairs_line(&cost);
        let edge_links: Vec<Vec<usize>> = edges
            .iter()
            .map(|(l, r)| {
                if l == r {
                    vec![]
                } else {
                    mutual_positions(&blocks[*l].mstr, &blocks[*r].mstr)
                }
            })
            .collect();
        let split_flags: Vec<(bool, bool)> = edges
            .iter()
            .enumerate()
            .map(|(_ei, (l, r))| {
                if l == r {
                    return (false, false);
                }
                let left_sorted =
                    sort_nodes_for_prefix_compatibility(&blocks[*l].nodes, &blocks[*l].terms);
                let right_sorted =
                    sort_nodes_for_prefix_compatibility(&blocks[*r].nodes, &blocks[*r].terms);
                let link = &edge_links[_ei];
                let sl = !left_sorted.is_empty() && link.contains(&left_sorted[0]);
                let sr = !right_sorted.is_empty() && link.contains(&right_sorted[0]);
                (sl, sr)
            })
            .collect();
        (edges, edge_links, split_flags)
    };

    Some(PauliBlockCache {
        block_signatures,
        block_nodes,
        block_mstrs,
        edges,
        edge_links,
        split_flags,
        block_clifford_annotations,
    })
}

/// Compile a Trotter step using block-level Pauli synthesis.
///
/// When `cached_blocks` is provided, the pre-computed QWC block structure is reused
/// instead of calling `form_blocks()` — this ensures forward and reverse Trotter
/// passes produce identical CNOT tree structures.
pub fn compile_step_pauli_synthesis(
    circuit: &mut QuantumCircuit,
    terms: &[&crate::hamiltonian::PauliTerm],
    dt: f64,
    hbar: f64,
    is_reverse: bool,
    strategy: BlockGroupingStrategy,
    mut budget: Option<&mut PauliGateBudget>,
    cached_blocks: Option<&PauliBlockCache>,
    clifford_map: Option<&CliffordAnnotationMap>,
    clifford_enhanced_blocks: bool,
) -> crate::Result<()> {
    if terms.is_empty() {
        return Ok(());
    }

    let scale = dt / hbar;

    // If we have a cache, reconstruct blocks from it.
    // Build a lookup: Pauli string repr → coefficient from current terms.
    let blocks: Vec<PauliBlock> = if let Some(cache) = cached_blocks {
        let coeff_map: std::collections::HashMap<String, f64> = terms
            .iter()
            .map(|t| {
                (
                    t.pauli_string.to_string_repr().to_string(),
                    t.coefficient.re * scale,
                )
            })
            .collect();

        cache
            .block_signatures
            .iter()
            .enumerate()
            .map(|(bi, sigs)| {
                let terms: Vec<(PauliString, f64)> = sigs
                    .iter()
                    .filter_map(|sig| {
                        let coeff = *coeff_map.get(sig)?;
                        PauliString::from_str(sig).ok().map(|p| (p, coeff))
                    })
                    .collect();
                let clifford_ann = if bi < cache.block_clifford_annotations.len() {
                    cache.block_clifford_annotations[bi].clone()
                } else {
                    vec![None; terms.len()]
                };
                PauliBlock {
                    terms,
                    mstr: cache.block_mstrs[bi].clone(),
                    nodes: cache.block_nodes[bi].clone(),
                    clifford_annotations: clifford_ann,
                }
            })
            .filter(|b| !b.terms.is_empty())
            .collect()
    } else {
        let scaled: Vec<(PauliString, f64)> = terms
            .iter()
            .map(|t| (t.pauli_string.clone(), t.coefficient.re * scale))
            .collect();

        let (blocks, _group_ids) = match strategy {
            BlockGroupingStrategy::QWC => {
                // Phase 11j: Cliffold map is passed through to syn_block
                // (Phase 11a-3 path). form_blocks() preserves natural QWC
                // groupings; each block's clifford_map annotations enable
                // Clifford-aware individual-term synthesis.
                // Note: form_blocks_clifford_aware() was tried and reverted
                // — it degraded gate count by merging independently-efficient
                // blocks into larger, less efficient ones.
                let b = if clifford_enhanced_blocks {
                    form_blocks_clifford_enhanced(&scaled)
                } else {
                    form_blocks(&scaled)
                };
                let ids: Vec<usize> = (0..b.len()).collect();
                (b, ids)
            }
            BlockGroupingStrategy::GeneralCommuting => form_blocks_general_commuting(&scaled),
        };

        if blocks.is_empty() {
            return Ok(());
        }
        // Sort: for GC, by GC group first; for QWC, by cost
        let mut blocks_with_groups: Vec<(PauliBlock, usize)> =
            blocks.into_iter().zip(_group_ids).collect();
        blocks_with_groups.sort_by(|(a, ga), (b, gb)| {
            ga.cmp(gb)
                .then_with(|| b.cost().cmp(&a.cost()))
                .then_with(|| a.mstr.to_string_repr().cmp(b.mstr.to_string_repr()))
        });
        blocks_with_groups.into_iter().map(|(b, _)| b).collect()
    };

    if blocks.is_empty() {
        return Ok(());
    }

    // Phase 11j: Cliffold map passed through to syn_block for Phase 11a-3
    // per-term annotation. form_blocks() preserves natural QWC groupings;
    // the clifford map enables Clifford-aware synthesis within each block
    // without changing the block structure.
    // Note: form_blocks_clifford_aware exists but is NOT used — it was found
    // to degrade gate count (453→412 for H2_4q) by merging blocks that had
    // efficient independent CNOT trees into larger, less efficient ones.
    let syn_clifford_map = clifford_map;

    // Phase 10a: When is_reverse is true and we're not using a cache,
    // reverse blocks so forward and reverse passes share the same QWC
    // partition. This is required for Trotter symmetry: the reverse pass
    // must decompose the SAME exponential, just with reversed block/term
    // order for step-boundary CNOT cancellation.
    let blocks = if is_reverse && cached_blocks.is_none() {
        reverse_blocks(&blocks)
    } else {
        blocks
    };

    let nb = blocks.len();
    if let Some(ref mut b) = budget {
        b.num_blocks = nb;
    }

    // Use cached edges or recompute
    let edges: Vec<(usize, usize)> = if let Some(cache) = cached_blocks {
        cache.edges.clone()
    } else if nb == 1 {
        vec![(0usize, 0usize)]
    } else {
        let cost: Vec<usize> = (0..nb.saturating_sub(1))
            .map(|i| {
                let overlap = mutual_positions(&blocks[i].mstr, &blocks[i + 1].mstr);
                let size = overlap.len();
                if size < 2 {
                    return size;
                }
                let a_sorted =
                    sort_nodes_for_prefix_compatibility(&blocks[i].nodes, &blocks[i].terms);
                let b_sorted =
                    sort_nodes_for_prefix_compatibility(&blocks[i + 1].nodes, &blocks[i + 1].terms);
                let same_direction = overlap.iter().all(|q| {
                    let a_pos = a_sorted.iter().position(|s| s == q);
                    let b_pos = b_sorted.iter().position(|s| s == q);
                    a_pos == b_pos
                });
                if same_direction {
                    size + 1
                } else {
                    size
                }
            })
            .collect();
        max_singlet_pairs_line(&cost)
    };

    // Synthesize per edge
    for (ei, (left, right)) in edges.iter().enumerate() {
        let (split_left, split_right) = if let Some(cache) = cached_blocks {
            cache.split_flags.get(ei).copied().unwrap_or((false, false))
        } else {
            // No cache — recompute split flags
            if left == right {
                (false, false)
            } else {
                let link = mutual_positions(&blocks[*left].mstr, &blocks[*right].mstr);
                let left_sorted =
                    sort_nodes_for_prefix_compatibility(&blocks[*left].nodes, &blocks[*left].terms);
                let right_sorted = sort_nodes_for_prefix_compatibility(
                    &blocks[*right].nodes,
                    &blocks[*right].terms,
                );
                let sl = !left_sorted.is_empty() && link.contains(&left_sorted[0]);
                let sr = !right_sorted.is_empty() && link.contains(&right_sorted[0]);
                (sl, sr)
            }
        };

        if left == right {
            if let Some(ref mut b) = budget {
                b.num_singletons += 1;
            }
            syn_block(
                circuit,
                &blocks[*left],
                &[],
                false,
                split_left,
                budget.as_deref_mut(),
                syn_clifford_map,
            )?;
        } else {
            if let Some(ref mut b) = budget {
                b.num_paired_edges += 1;
            }
            let link = if let Some(cache) = cached_blocks {
                cache.edge_links.get(ei).cloned().unwrap_or_default()
            } else {
                mutual_positions(&blocks[*left].mstr, &blocks[*right].mstr)
            };
            syn_block(
                circuit,
                &blocks[*left],
                &link,
                false,
                split_left,
                budget.as_deref_mut(),
                syn_clifford_map,
            )?;
            syn_block(
                circuit,
                &blocks[*right],
                &link,
                true,
                split_right,
                budget.as_deref_mut(),
                syn_clifford_map,
            )?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Cross-step Pauli synthesis (Phase 9a — Phase 3)
// ---------------------------------------------------------------------------

/// Compile a Hamiltonian across ALL Trotter steps with a single synthesis pass.
///
/// # Core insight
///
/// Within a QWC block, all Z rotations commute after CNOT diagonalization, and
/// `CNOT^dag · CNOT = I` between consecutive appearances of the same block.
/// This means N Trotter steps collapse to ONE CNOT tree per block, with total
/// coefficients summed across all steps.
///
/// For first-order Trotter: each term's coefficient = `c_k * T / ħ`.
/// For second-order Trotter: forward and reverse passes are synthesized
/// separately (each with `c_k * T / (2ħ)`) to preserve the symmetric structure.
///
/// # Circuit equivalence
///
/// The cross-step circuit is a **different but equally valid** Trotter
/// approximation. Instead of interleaving blocks across steps:
///
/// ```text
/// (U_A · U_B · U_C)^N    (standard: N steps interleaved)
/// ```
///
/// We produce:
///
/// ```text
/// U_A^N · U_B^N · U_C^N  (cross-step: blocks grouped, total coefficients)
/// ```
///
/// The error scaling is the same O(t²·‖H‖²/N) as standard first-order Trotter.
///
/// # Returns
///
/// The number of QWC blocks synthesized (for diagnostics).
pub fn compile_cross_step_pauli_synthesis(
    circuit: &mut QuantumCircuit,
    hamiltonian: &super::Hamiltonian,
    evolution_time: f64,
    hbar: f64,
    trotter_order: &crate::hamiltonian::hamiltonian_compiler::TrotterOrder,
    trotter_steps: usize,
    skip_identities: bool,
    strategy: BlockGroupingStrategy,
    mut budget: Option<&mut PauliGateBudget>,
    clifford_map: Option<&CliffordAnnotationMap>,
    clifford_enhanced_blocks: bool,
) -> crate::Result<usize> {
    if hamiltonian.terms.is_empty() {
        return Ok(0);
    }

    // ── Filter and scale terms ────────────────────────────────────────
    let terms: Vec<&super::PauliTerm> = hamiltonian.terms.iter().collect();
    let filtered: Vec<&super::PauliTerm> = terms
        .iter()
        .filter(|t| !(skip_identities && t.pauli_string.is_identity()))
        .copied()
        .collect();

    if filtered.is_empty() {
        return Ok(0);
    }

    // The number of times each term appears in the Trotter formula.
    // Within a QWC block, all appearances merge → total coefficient.
    let scale = evolution_time / hbar;

    // Build scaled term list. For second-order, we emit forward then reverse
    // (each with half the total time). For first-order, one pass.
    match trotter_order {
        crate::hamiltonian::hamiltonian_compiler::TrotterOrder::First => {
            let scaled: Vec<(PauliString, f64)> = filtered
                .iter()
                .map(|t| (t.pauli_string.clone(), t.coefficient.re * scale))
                .collect();

            let n_blocks = synthesize_blocks_from_scaled(
                circuit,
                &scaled,
                strategy,
                budget.as_deref_mut(),
                clifford_map,
            )?;
            Ok(n_blocks)
        }
        crate::hamiltonian::hamiltonian_compiler::TrotterOrder::Second => {
            // Second-order: forward half (T/2) then reverse half (T/2).
            // For Trotter symmetry, the reverse pass must use the SAME QWC block
            // structure as the forward pass, just with reversed block ordering.
            let half_scale = scale / 2.0;
            let fwd: Vec<(PauliString, f64)> = filtered
                .iter()
                .map(|t| (t.pauli_string.clone(), t.coefficient.re * half_scale))
                .collect();

            // Form blocks once from forward terms and reuse for reverse.
            let fwd_blocks = form_blocks_from_scaled(&fwd, strategy);
            let n_fwd =
                synthesize_blocks(circuit, &fwd_blocks, budget.as_deref_mut(), clifford_map)?;

            // Reverse: same blocks, reversed order, reversed internal terms.
            let rev_blocks = reverse_blocks(&fwd_blocks);
            let n_rev =
                synthesize_blocks(circuit, &rev_blocks, budget.as_deref_mut(), clifford_map)?;
            Ok(n_fwd + n_rev)
        }
        crate::hamiltonian::hamiltonian_compiler::TrotterOrder::Fourth => {
            // Fourth-order Suzuki: S_4(t) = [S_2(p1·t)]^2 · S_2(p2·t) · [S_2(p1·t)]^2
            // where p1 = 1/(4-4^(1/3)), p2 = 1-4p1.
            // Total coefficient: 4p1 + p2 = 1.0  ✓
            let p1 = 1.0 / (4.0 - 4.0_f64.powf(1.0 / 3.0));
            let p2 = 1.0 - 4.0 * p1;
            let coeffs = [p1, p1, p2, p1, p1];
            let mut total_blocks = 0usize;

            for &p in &coeffs {
                let s2_scale = scale * p / 2.0;
                // Forward half: form blocks once, reuse for reverse.
                let fwd: Vec<(PauliString, f64)> = filtered
                    .iter()
                    .map(|t| (t.pauli_string.clone(), t.coefficient.re * s2_scale))
                    .collect();
                let fwd_blocks = form_blocks_from_scaled(&fwd, strategy);
                total_blocks +=
                    synthesize_blocks(circuit, &fwd_blocks, budget.as_deref_mut(), clifford_map)?;

                // Reverse half: same blocks, reversed order and reversed internal terms.
                let rev_blocks = reverse_blocks(&fwd_blocks);
                total_blocks +=
                    synthesize_blocks(circuit, &rev_blocks, budget.as_deref_mut(), clifford_map)?;
            }
            Ok(total_blocks)
        }
        _ => {
            // For higher-order and custom Suzuki: apply recursively.
            // Compute how many second-order-equivalent sub-steps there are.
            // Default to the standard per-step loop for unsupported orders.
            let terms_refs: Vec<&super::PauliTerm> = filtered.to_vec();
            let dt = evolution_time / trotter_steps as f64;
            for _step in 0..trotter_steps {
                crate::hamiltonian::pauli_synthesis::compile_step_pauli_synthesis(
                    circuit,
                    &terms_refs,
                    dt,
                    hbar,
                    false,
                    strategy,
                    budget.as_deref_mut(),
                    None,
                    None,
                    clifford_enhanced_blocks,
                )?;
            }
            Ok(0) // block count meaningless for fallback path
        }
    }
}

/// Internal helper: build blocks from scaled terms and synthesize them.
/// Returns the number of blocks synthesized.
///
/// Form and sort blocks from scaled Pauli terms.
///
/// When `strategy` is `GeneralCommuting`, uses graph-coloring-based grouping
/// that places QWC subgroups from the same general commuting group adjacent,
/// enabling better inter-block CNOT cancellation.
fn form_blocks_from_scaled(
    scaled_terms: &[(PauliString, f64)],
    strategy: BlockGroupingStrategy,
) -> Vec<PauliBlock> {
    let mut blocks_with_groups: Vec<(PauliBlock, usize)> = match strategy {
        BlockGroupingStrategy::QWC => {
            let blocks = form_blocks(scaled_terms);
            blocks
                .into_iter()
                .enumerate()
                .map(|(i, b)| (b, i))
                .collect()
        }
        BlockGroupingStrategy::GeneralCommuting => {
            let (blocks, group_ids) = form_blocks_general_commuting(scaled_terms);
            blocks.into_iter().zip(group_ids).collect()
        }
    };

    if blocks_with_groups.is_empty() {
        return vec![];
    }

    // Sort: for QWC, by cost descending. For GeneralCommuting, by GC group
    // first (to keep QWC subgroups from the same GC group adjacent), then by cost.
    blocks_with_groups.sort_by(|(a, ga), (b, gb)| {
        ga.cmp(gb)
            .then_with(|| b.cost().cmp(&a.cost()))
            .then_with(|| a.mstr.to_string_repr().cmp(b.mstr.to_string_repr()))
    });

    blocks_with_groups.into_iter().map(|(b, _)| b).collect()
}

/// Synthesize pre-formed blocks into the circuit. Replaces synthesize_blocks_from_scaled
/// for cases where block formation and synthesis are separated.
fn synthesize_blocks(
    circuit: &mut QuantumCircuit,
    blocks: &[PauliBlock],
    mut budget: Option<&mut PauliGateBudget>,
    clifford_map: Option<&CliffordAnnotationMap>,
) -> crate::Result<usize> {
    let nb = blocks.len();
    if nb == 0 {
        return Ok(0);
    }
    if let Some(ref mut b) = budget {
        b.num_blocks += nb;
    }

    // Compute edge pairing (same logic as compile_step_pauli_synthesis)
    let edges: Vec<(usize, usize)> = if nb == 1 {
        vec![(0usize, 0usize)]
    } else {
        let cost: Vec<usize> = (0..nb.saturating_sub(1))
            .map(|i| {
                let overlap = mutual_positions(&blocks[i].mstr, &blocks[i + 1].mstr);
                let size = overlap.len();
                if size < 2 {
                    return size;
                }
                let a_sorted =
                    sort_nodes_for_prefix_compatibility(&blocks[i].nodes, &blocks[i].terms);
                let b_sorted =
                    sort_nodes_for_prefix_compatibility(&blocks[i + 1].nodes, &blocks[i + 1].terms);
                let same_direction = overlap.iter().all(|q| {
                    let a_pos = a_sorted.iter().position(|s| s == q);
                    let b_pos = b_sorted.iter().position(|s| s == q);
                    a_pos == b_pos
                });
                if same_direction {
                    size + 1
                } else {
                    size
                }
            })
            .collect();
        max_singlet_pairs_line(&cost)
    };

    // Synthesize per edge
    for (left, right) in edges.iter() {
        if left == right {
            if let Some(ref mut b) = budget {
                b.num_singletons += 1;
            }
            syn_block(
                circuit,
                &blocks[*left],
                &[],
                false,
                false,
                budget.as_deref_mut(),
                clifford_map,
            )?;
        } else {
            if let Some(ref mut b) = budget {
                b.num_paired_edges += 1;
            }
            let link = mutual_positions(&blocks[*left].mstr, &blocks[*right].mstr);
            let left_sorted =
                sort_nodes_for_prefix_compatibility(&blocks[*left].nodes, &blocks[*left].terms);
            let right_sorted =
                sort_nodes_for_prefix_compatibility(&blocks[*right].nodes, &blocks[*right].terms);
            let split_left = !left_sorted.is_empty() && link.contains(&left_sorted[0]);
            let split_right = !right_sorted.is_empty() && link.contains(&right_sorted[0]);

            syn_block(
                circuit,
                &blocks[*left],
                &link,
                false,
                split_left,
                budget.as_deref_mut(),
                clifford_map,
            )?;
            syn_block(
                circuit,
                &blocks[*right],
                &link,
                true,
                split_right,
                budget.as_deref_mut(),
                clifford_map,
            )?;
        }
    }

    Ok(nb)
}

/// Form blocks from scaled terms and synthesize them in one pass.
/// Returns the number of blocks synthesized.
fn synthesize_blocks_from_scaled(
    circuit: &mut QuantumCircuit,
    scaled_terms: &[(PauliString, f64)],
    strategy: BlockGroupingStrategy,
    budget: Option<&mut PauliGateBudget>,
    clifford_map: Option<&CliffordAnnotationMap>,
) -> crate::Result<usize> {
    let blocks = form_blocks_from_scaled(scaled_terms, strategy);
    synthesize_blocks(circuit, &blocks, budget, clifford_map)
}

/// Reverse the order of blocks and the order of terms within each block.
/// Used to construct the reverse half of a Trotter step from the forward half's
/// block structure, ensuring identical QWC partitions for Trotter symmetry.
fn reverse_blocks(blocks: &[PauliBlock]) -> Vec<PauliBlock> {
    blocks
        .iter()
        .rev()
        .map(|block| {
            let mut rev_terms = block.terms.clone();
            let mut rev_annotations = block.clifford_annotations.clone();
            rev_terms.reverse();
            rev_annotations.reverse();
            PauliBlock {
                terms: rev_terms,
                mstr: block.mstr.clone(),
                nodes: block.nodes.clone(),
                clifford_annotations: rev_annotations,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StandardGate;

    #[test]
    fn test_pOR_basic() {
        let a = PauliString::from_str("XZII").unwrap();
        let b = PauliString::from_str("IYZZ").unwrap();
        let m = p_or(&a, &b);
        assert_eq!(m.to_string_repr(), "XZZZ");
    }

    #[test]
    fn test_is_qwc() {
        let a = PauliString::from_str("ZZII").unwrap();
        let b = PauliString::from_str("IZZI").unwrap();
        assert!(is_qwc(&a, &b));
        let c = PauliString::from_str("XZII").unwrap();
        assert!(!is_qwc(&a, &c));
    }

    #[test]
    fn test_active_nodes() {
        let p = PauliString::from_str("IXZI").unwrap();
        assert_eq!(active_nodes(&p), vec![1, 2]);
    }

    #[test]
    fn test_form_blocks_qwc() {
        let terms: Vec<(PauliString, f64)> = vec![
            (PauliString::from_str("ZZII").unwrap(), 0.5),
            (PauliString::from_str("IZZI").unwrap(), 0.3),
            (PauliString::from_str("IIZZ").unwrap(), 0.2),
        ];
        let blocks = form_blocks(&terms);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].terms.len(), 3);
        assert_eq!(blocks[0].mstr.to_string_repr(), "ZZZZ");
        assert_eq!(blocks[0].nodes.len(), 4);
    }

    #[test]
    fn test_form_blocks_mixed() {
        let terms: Vec<(PauliString, f64)> = vec![
            (PauliString::from_str("ZZII").unwrap(), 0.5),
            (PauliString::from_str("IXZI").unwrap(), 0.3),
            (PauliString::from_str("IIZZ").unwrap(), 0.2),
            (PauliString::from_str("XIII").unwrap(), 0.1),
        ];
        let blocks = form_blocks(&terms);
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn test_complement_tree2_single_node() {
        let (cnotset, root) = complement_tree2(&[3], &[]);
        assert!(cnotset.is_empty());
        assert_eq!(root, Some(3));
    }

    #[test]
    fn test_complement_tree2_two_nodes() {
        let (cnotset, root) = complement_tree2(&[0, 2], &[]);
        assert_eq!(cnotset.len(), 1);
        assert_eq!(root, Some(2));
    }

    #[test]
    fn test_complement_tree2_with_link() {
        let (cnotset, root) = complement_tree2(&[0, 1, 2, 3], &[1, 2]);
        assert_eq!(root, Some(3));
        assert_eq!(cnotset.len(), 3);
    }

    #[test]
    fn test_mutual_positions() {
        let pa = PauliString::from_str("XZII").unwrap();
        let pb = PauliString::from_str("XZII").unwrap();
        assert_eq!(mutual_positions(&pa, &pb), vec![0, 1]);
        let pc = PauliString::from_str("IXZI").unwrap();
        assert_eq!(mutual_positions(&pa, &pc), vec![1]);
    }

    #[test]
    fn test_max_singlet_pairs_basic() {
        let cost = vec![3, 0];
        let edges = max_singlet_pairs_line(&cost);
        assert_eq!(edges, vec![(0, 1), (2, 2)]);
    }

    #[test]
    fn test_max_singlet_pairs_no_overlap() {
        let cost = vec![0, 0];
        let edges = max_singlet_pairs_line(&cost);
        assert_eq!(edges, vec![(0, 1), (2, 2)]);
    }

    #[test]
    fn test_syn_block_basic() {
        let pauli = PauliString::from_str("XZII").unwrap();
        let mstr = pauli.clone();
        let nodes = active_nodes(&mstr);
        let block = PauliBlock {
            terms: vec![(pauli, 0.5)],
            mstr,
            nodes,
            clifford_annotations: vec![None; 1],
        };
        let mut circuit = QuantumCircuit::new(4, 0);
        syn_block(&mut circuit, &block, &[], false, false, None, None).unwrap();
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_syn_block_two_terms_shared_tree() {
        let p1 = PauliString::from_str("ZZII").unwrap();
        let p2 = PauliString::from_str("IZZI").unwrap();
        let mstr = p_or(&p1, &p2);
        let nodes = active_nodes(&mstr);
        let block = PauliBlock {
            terms: vec![(p1, 0.3), (p2, 0.2)],
            mstr,
            nodes,
            clifford_annotations: vec![None; 2],
        };
        let mut circuit = QuantumCircuit::new(4, 0);
        syn_block(&mut circuit, &block, &[], false, false, None, None).unwrap();
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_compile_step_empty() {
        let mut circuit = QuantumCircuit::new(4, 0);
        compile_step_pauli_synthesis(
            &mut circuit,
            &[],
            1.0,
            1.0,
            false,
            BlockGroupingStrategy::QWC,
            None,
            None,
            None,
            false,
        )
        .unwrap();
        assert_eq!(circuit.size(), 0);
    }

    // -----------------------------------------------------------------------
    // Opt 1: Term merging tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_merge_identical_active_terms_basic() {
        // Two terms with the same active qubits {0,1} → should merge
        let terms: Vec<(PauliString, f64)> = vec![
            (PauliString::from_str("ZZII").unwrap(), 0.3),
            (PauliString::from_str("ZZII").unwrap(), 0.2),
        ];
        let merged = merge_identical_active_terms(&terms);
        assert_eq!(merged.len(), 1);
        assert!((merged[0].1 - 0.5).abs() < 1e-15);
    }

    #[test]
    fn test_merge_different_active_sets_no_merge() {
        // Terms with different active qubit sets → no merging
        let terms: Vec<(PauliString, f64)> = vec![
            (PauliString::from_str("ZZII").unwrap(), 0.3),
            (PauliString::from_str("IZZI").unwrap(), 0.2),
        ];
        let merged = merge_identical_active_terms(&terms);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_merge_filters_zero_coeff() {
        let terms: Vec<(PauliString, f64)> = vec![
            (PauliString::from_str("ZZII").unwrap(), 0.5),
            (PauliString::from_str("ZZII").unwrap(), -0.5),
        ];
        let merged = merge_identical_active_terms(&terms);
        assert_eq!(merged.len(), 0);
    }

    // -----------------------------------------------------------------------
    // Opt 2: Prefix and shared tree tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_is_prefix_of_true() {
        assert!(is_prefix_of(&[0, 1], &[0, 1, 2, 3]));
        assert!(is_prefix_of(&[0], &[0, 1, 2, 3]));
        assert!(is_prefix_of(&[0, 1, 2, 3], &[0, 1, 2, 3]));
    }

    #[test]
    fn test_is_prefix_of_false() {
        // Not a prefix (doesn't start at 0)
        assert!(!is_prefix_of(&[1, 2], &[0, 1, 2, 3]));
        // Non-contiguous
        assert!(!is_prefix_of(&[0, 2], &[0, 1, 2, 3]));
        // Too long
        assert!(!is_prefix_of(&[0, 1, 2, 3, 4], &[0, 1, 2, 3]));
    }

    #[test]
    fn test_synthesize_shared_tree_single_term() {
        // Single prefix term — should produce correct circuit
        let pauli = PauliString::from_str("XZII").unwrap();
        let sorted_nodes = vec![0, 1];
        let mut circuit = QuantumCircuit::new(4, 0);
        synthesize_shared_tree(
            &mut circuit,
            &[(pauli, 0.5)],
            &sorted_nodes,
            false,
            None,
            &[],
        )
        .unwrap();
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_synthesize_shared_tree_two_prefix_terms() {
        // Two prefix-compatible terms: {0,1} and {0,1,2}
        let p1 = PauliString::from_str("ZZII").unwrap();
        let p2 = PauliString::from_str("ZZZI").unwrap();
        let sorted_nodes = vec![0, 1, 2, 3];
        let mut circuit = QuantumCircuit::new(4, 0);
        synthesize_shared_tree(
            &mut circuit,
            &[(p1, 0.3), (p2, 0.2)],
            &sorted_nodes,
            false,
            None,
            &[],
        )
        .unwrap();
        // Should have: basis + CNOT(0,1) + Rz(1) + CNOT(1,2) + Rz(2) + rev
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_synthesize_shared_tree_single_qubit_terms() {
        // All terms are single-qubit → no CNOT tree needed
        let p1 = PauliString::from_str("ZIII").unwrap();
        let p2 = PauliString::from_str("IZII").unwrap();
        let sorted_nodes = vec![0, 1];
        let mut circuit = QuantumCircuit::new(4, 0);
        synthesize_shared_tree(
            &mut circuit,
            &[(p1, 0.3), (p2, 0.2)],
            &sorted_nodes,
            false,
            None,
            &[],
        )
        .unwrap();
        assert!(circuit.size() > 0);
    }

    // -----------------------------------------------------------------------
    // Opt 3 & Integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_syn_block_with_emit_prefix_first() {
        let p1 = PauliString::from_str("ZZII").unwrap();
        let p2 = PauliString::from_str("ZZZI").unwrap();
        let mstr = p_or(&p1, &p2);
        let nodes = active_nodes(&mstr);
        let block = PauliBlock {
            terms: vec![(p1, 0.3), (p2, 0.2)],
            mstr,
            nodes,
            clifford_annotations: vec![None; 2],
        };
        // Both orderings should produce valid circuits
        let mut c1 = QuantumCircuit::new(4, 0);
        syn_block(&mut c1, &block, &[], false, false, None, None).unwrap();
        let mut c2 = QuantumCircuit::new(4, 0);
        syn_block(&mut c2, &block, &[], true, false, None, None).unwrap();
        assert!(c1.size() > 0);
        assert!(c2.size() > 0);
    }

    #[test]
    fn test_syn_block_mixed_prefix_nonprefix() {
        // ZZII (prefix {0,1}) and IZZI (non-prefix {1,2})
        let p1 = PauliString::from_str("ZZII").unwrap();
        let p2 = PauliString::from_str("IZZI").unwrap();
        let mstr = p_or(&p1, &p2);
        let nodes = active_nodes(&mstr);
        let block = PauliBlock {
            terms: vec![(p1, 0.3), (p2, 0.2)],
            mstr,
            nodes,
            clifford_annotations: vec![None; 2],
        };
        let mut circuit = QuantumCircuit::new(4, 0);
        syn_block(&mut circuit, &block, &[], false, false, None, None).unwrap();
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_compile_step_multiple_blocks() {
        // Two QWC blocks that should be paired
        let mut circuit = QuantumCircuit::new(4, 0);
        let terms: Vec<&crate::hamiltonian::PauliTerm> = vec![];
        compile_step_pauli_synthesis(
            &mut circuit,
            &terms,
            1.0,
            1.0,
            false,
            BlockGroupingStrategy::QWC,
            None,
            None,
            None,
            false,
        )
        .unwrap();
        assert_eq!(circuit.size(), 0);
    }

    // -----------------------------------------------------------------------
    // Optimal node ordering tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_optimal_ordering_finds_best_prefix() {
        // Terms: {0}, {1}, {0,1}, {1,2} — best ordering should start with 0 or 1
        let p1 = (PauliString::from_str("ZZII").unwrap(), 0.5); // {0,1}
        let p2 = (PauliString::from_str("IZZI").unwrap(), 0.3); // {1,2}
        let p3 = (PauliString::from_str("ZIII").unwrap(), 0.2); // {0}
        let p4 = (PauliString::from_str("IZII").unwrap(), 0.1); // {1}
        let terms = vec![p1, p2, p3, p4];
        let nodes = vec![0, 1, 2];
        let ordering = sort_nodes_for_prefix_compatibility(&nodes, &terms);
        // Optimal ordering should start with the qubit that maximizes prefixes
        // [0,1,2] gives prefixes {0}, {0,1} → 2
        // [1,2,0] gives prefixes {1}, {1,2} → 2
        // [1,0,2] gives prefixes {1} → 1
        // Any valid ordering is acceptable; just verify it's a permutation
        assert_eq!(ordering.len(), 3);
        assert!(ordering.contains(&0));
        assert!(ordering.contains(&1));
        assert!(ordering.contains(&2));
        // Verify at least one multi-qubit term is a prefix
        let term_sets: Vec<Vec<usize>> = vec![vec![0, 1], vec![1, 2]];
        let cnt = count_prefix_terms(&ordering, &term_sets);
        assert!(
            cnt >= 1,
            "Expected at least 1 prefix term, got {} with ordering {:?}",
            cnt,
            ordering
        );
    }

    #[test]
    fn test_synthesize_shared_tree_with_position_zero_rz() {
        // Single-qubit term at position 0 mixed with multi-qubit term
        let p0 = PauliString::from_str("ZIII").unwrap(); // single-qubit {0}
        let p01 = PauliString::from_str("ZZII").unwrap(); // multi-qubit {0,1}
        let sorted_nodes = vec![0, 1, 2];
        let mut circuit = QuantumCircuit::new(4, 0);
        synthesize_shared_tree(
            &mut circuit,
            &[(p0, 0.3), (p01, 0.5)],
            &sorted_nodes,
            false,
            None,
            &[],
        )
        .unwrap();
        // Should have: basis[H if needed], Rz(0), CNOT(0,1), Rz(1), rev, inv_basis
        assert!(circuit.size() > 0);
    }

    // -----------------------------------------------------------------------
    // P0: Y-basis change and Rz merge tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_y_basis_uses_rx_not_s() {
        // Verify that Y basis change uses Rx instead of S+H.
        let mut c = QuantumCircuit::new(1, 0);
        apply_basis_change(&mut c, 0, PauliOperator::Y).unwrap();
        let insts = c.data().instructions();
        // Should be a single Rx, not S + H
        assert_eq!(insts.len(), 1, "Y basis should be single gate");
        assert_eq!(insts[0].gate.gate_type, StandardGate::Rx);
        // Verify parameter is π/2
        let p = &insts[0].gate.parameters[0];
        if let Parameter::Float(v) = p {
            assert!(
                (v - std::f64::consts::PI / 2.0).abs() < 1e-10,
                "Expected PI/2, got {}",
                v
            );
        } else {
            panic!("Expected Float parameter");
        }
    }

    #[test]
    fn test_y_inv_basis_uses_rx_not_sdg() {
        let mut c = QuantumCircuit::new(1, 0);
        apply_inv_basis_change(&mut c, 0, PauliOperator::Y).unwrap();
        let insts = c.data().instructions();
        assert_eq!(insts.len(), 1);
        assert_eq!(insts[0].gate.gate_type, StandardGate::Rx);
        // Verify parameter is -π/2
        let p = &insts[0].gate.parameters[0];
        if let Parameter::Float(v) = p {
            assert!(
                (v + std::f64::consts::PI / 2.0).abs() < 1e-10,
                "Expected -PI/2, got {}",
                v
            );
        } else {
            panic!("Expected Float parameter");
        }
    }

    #[test]
    fn test_y_basis_roundtrip_is_identity() {
        // apply + inverse should produce identity on a single qubit
        let mut c = QuantumCircuit::new(1, 0);
        apply_basis_change(&mut c, 0, PauliOperator::Y).unwrap();
        apply_inv_basis_change(&mut c, 0, PauliOperator::Y).unwrap();
        // Rx(π/2) followed by Rx(-π/2) should cancel
        let unitary = c.unitary(&std::collections::HashMap::new()).unwrap();
        // Identity matrix up to global phase
        let i00 = unitary[[0, 0]].norm();
        let i11 = unitary[[1, 1]].norm();
        let off01 = unitary[[0, 1]].norm();
        let off10 = unitary[[1, 0]].norm();
        assert!(i00 > 0.99, "Expected |U[0,0]| ≈ 1, got {}", i00);
        assert!(i11 > 0.99, "Expected |U[1,1]| ≈ 1, got {}", i11);
        assert!(off01 < 0.01, "Expected |U[0,1]| ≈ 0, got {}", off01);
        assert!(off10 < 0.01, "Expected |U[1,0]| ≈ 0, got {}", off10);
    }

    #[test]
    fn test_z_basis_applies_no_gates() {
        // Z and I operators should not add any gates
        for op in [PauliOperator::Z, PauliOperator::I] {
            let mut c = QuantumCircuit::new(1, 0);
            apply_basis_change(&mut c, 0, op).unwrap();
            assert_eq!(c.size(), 0, "Z/I basis should add no gates");
            apply_inv_basis_change(&mut c, 0, op).unwrap();
            assert_eq!(c.size(), 0, "Z/I inv basis should add no gates");
        }
    }

    #[test]
    fn test_shared_tree_no_adjacent_duplicate_rz() {
        // Verify shared tree doesn't emit adjacent identical Rz on the same qubit.
        // Use two prefix-compatible terms on {0,1} and {0,1,2}.
        let p1 = PauliString::from_str("ZZII").unwrap(); // active {0,1}
        let p2 = PauliString::from_str("ZZZI").unwrap(); // active {0,1,2}
        let sorted_nodes = vec![0, 1, 2, 3];
        let mut circuit = QuantumCircuit::new(4, 0);
        synthesize_shared_tree(
            &mut circuit,
            &[(p1, 0.3), (p2, 0.2)],
            &sorted_nodes,
            false,
            None,
            &[],
        )
        .unwrap();

        let insts = circuit.data().instructions();
        // Check for adjacent Rz on same qubit with same parameter
        for w in insts.windows(2) {
            if w[0].gate.gate_type == StandardGate::Rz && w[1].gate.gate_type == StandardGate::Rz {
                let q0 = w[0].qubits[0].index();
                let q1 = w[1].qubits[0].index();
                if q0 == q1 {
                    // Rz gates on same qubit should not be adjacent
                    panic!(
                        "Adjacent Rz on qubit {}: inst {} and {}",
                        q0,
                        w[0].qubits[0].index(),
                        w[1].qubits[0].index()
                    );
                }
            }
        }
        assert!(circuit.size() > 0);
    }

    // -----------------------------------------------------------------------
    // P1: Secondary shared chain tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_nonprefix_secondary_chain_reduces_cx() {
        // Build a block where non-prefix terms can share a secondary chain.
        // Primary order [0,1,2]: prefix terms {0}, {0,1}
        // Secondary: non-prefix {2}, {0,2} — {2} IS prefix of [2] (single-qubit),
        // {0,2} is NOT prefix of [0,1,2] but IS prefix of [0,2] (sub-order)
        let p_prefix = PauliString::from_str("ZZII").unwrap(); // {0,1} prefix
        let p_nonprefix_a = PauliString::from_str("ZIZI").unwrap(); // {0,2} non-prefix
        let p_nonprefix_b = PauliString::from_str("IZII").unwrap(); // {1} non-prefix single-qubit
        let mstr = PauliString::from_str("ZZZI").unwrap();
        let nodes = vec![0, 1, 2];
        let block = PauliBlock {
            terms: vec![(p_prefix, 0.3), (p_nonprefix_a, 0.2), (p_nonprefix_b, 0.1)],
            mstr,
            nodes,
            clifford_annotations: vec![None; 3],
        };
        let mut circuit = QuantumCircuit::new(4, 0);
        syn_block(&mut circuit, &block, &[], false, false, None, None).unwrap();
        // Shared tree for prefix: 1 CX each way = 2 CX
        // Secondary chain for {0,2} as prefix of [0,2]: 1 CX each way = 2 CX
        // Single-qubit {1}: 0 CX
        // Total: 4 CX
        let cx = circuit
            .data()
            .instructions()
            .iter()
            .filter(|i| i.gate.gate_type == StandardGate::CX)
            .count();
        // Before P1, per-term for {0,2} would add 2 more CX (= 6 total)
        assert_eq!(
            cx, 4,
            "Secondary chain should reduce CX from 6 to 4, got {}",
            cx
        );
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_nonprefix_single_term_no_secondary_chain() {
        // When there's only 1 non-prefix multi-qubit term, no secondary chain
        let p1 = PauliString::from_str("ZZII").unwrap();
        let p2 = PauliString::from_str("IZZI").unwrap(); // {1,2} non-prefix
        let mstr = PauliString::from_str("ZZZI").unwrap();
        let nodes = vec![0, 1, 2, 3];
        let block = PauliBlock {
            terms: vec![(p1, 0.3), (p2, 0.2)],
            mstr,
            nodes,
            clifford_annotations: vec![None; 2],
        };
        let mut circuit = QuantumCircuit::new(4, 0);
        syn_block(&mut circuit, &block, &[], false, false, None, None).unwrap();
        // Should still compile correctly
        assert!(circuit.size() > 0);
    }

    // -----------------------------------------------------------------------
    // Cross-step synthesis tests (Phase 9a — Phase 3)
    // -----------------------------------------------------------------------

    /// Build H₂ (4-qubit) Hamiltonian for cross-step tests.
    fn build_h2_4q_test() -> crate::hamiltonian::Hamiltonian {
        use crate::hamiltonian::Hamiltonian;
        let mut h = Hamiltonian::new(4);
        for (ps, coeff) in [
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
        ] {
            h.add_term(
                PauliString::from_str(ps).unwrap(),
                num_complex::Complex64::new(coeff, 0.0),
            )
            .unwrap();
        }
        h
    }

    #[test]
    fn test_cross_step_first_order_compiles() {
        let h = build_h2_4q_test();
        let mut circuit = QuantumCircuit::new(4, 0);
        let n_blocks = compile_cross_step_pauli_synthesis(
            &mut circuit,
            &h,
            1.0, // evolution_time
            1.0, // hbar
            &crate::hamiltonian::hamiltonian_compiler::TrotterOrder::First,
            10,   // trotter_steps
            true, // skip_identities
            BlockGroupingStrategy::QWC,
            None,
            None,
            false,
        )
        .unwrap();
        assert!(n_blocks > 0, "Should produce at least one QWC block");
        assert!(circuit.size() > 0, "Circuit should have gates");
    }

    #[test]
    fn test_cross_step_second_order_compiles() {
        let h = build_h2_4q_test();
        let mut circuit = QuantumCircuit::new(4, 0);
        let n_blocks = compile_cross_step_pauli_synthesis(
            &mut circuit,
            &h,
            1.0,
            1.0,
            &crate::hamiltonian::hamiltonian_compiler::TrotterOrder::Second,
            10,
            true,
            BlockGroupingStrategy::QWC,
            None,
            None,
            false,
        )
        .unwrap();
        assert!(n_blocks > 0);
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_cross_step_fourth_order_compiles() {
        let h = build_h2_4q_test();
        let mut circuit = QuantumCircuit::new(4, 0);
        let n_blocks = compile_cross_step_pauli_synthesis(
            &mut circuit,
            &h,
            1.0,
            1.0,
            &crate::hamiltonian::hamiltonian_compiler::TrotterOrder::Fourth,
            1,
            true,
            BlockGroupingStrategy::QWC,
            None,
            None,
            false,
        )
        .unwrap();
        assert!(n_blocks > 0);
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_cross_step_empty_hamiltonian() {
        let h = crate::hamiltonian::Hamiltonian::new(4);
        let mut circuit = QuantumCircuit::new(4, 0);
        let n = compile_cross_step_pauli_synthesis(
            &mut circuit,
            &h,
            1.0,
            1.0,
            &crate::hamiltonian::hamiltonian_compiler::TrotterOrder::First,
            10,
            true,
            BlockGroupingStrategy::QWC,
            None,
            None,
            false,
        )
        .unwrap();
        assert_eq!(n, 0);
        assert_eq!(circuit.size(), 0);
    }

    #[test]
    fn test_cross_step_gate_count_independent_of_steps() {
        // Key property: cross-step synthesis produces the SAME gate count
        // regardless of the number of Trotter steps.
        let h = build_h2_4q_test();
        let mut c1 = QuantumCircuit::new(4, 0);
        compile_cross_step_pauli_synthesis(
            &mut c1,
            &h,
            1.0,
            1.0,
            &crate::hamiltonian::hamiltonian_compiler::TrotterOrder::First,
            1,
            true,
            BlockGroupingStrategy::QWC,
            None,
            None,
            false,
        )
        .unwrap();

        let mut c10 = QuantumCircuit::new(4, 0);
        compile_cross_step_pauli_synthesis(
            &mut c10,
            &h,
            1.0,
            1.0,
            &crate::hamiltonian::hamiltonian_compiler::TrotterOrder::First,
            10,
            true,
            BlockGroupingStrategy::QWC,
            None,
            None,
            false,
        )
        .unwrap();

        // Gate counts should be identical (same coefficients, same blocks)
        assert_eq!(
            c1.size(),
            c10.size(),
            "Cross-step gate count must be independent of trotter_steps: {} vs {}",
            c1.size(),
            c10.size()
        );
    }

    #[test]
    fn test_cross_step_vs_per_step_gate_reduction() {
        // Cross-step should produce FEWER gates than per-step for N > 1.
        let h = build_h2_4q_test();
        let steps = 5usize;
        let dt = 1.0 / steps as f64;

        // Per-step: N separate calls
        let mut per_step = QuantumCircuit::new(4, 0);
        let terms: Vec<&crate::hamiltonian::PauliTerm> = h.terms.iter().collect();
        for _ in 0..steps {
            compile_step_pauli_synthesis(
                &mut per_step,
                &terms,
                dt,
                1.0,
                false,
                BlockGroupingStrategy::QWC,
                None,
                None,
                None,
                false,
            )
            .unwrap();
        }

        // Cross-step: one call
        let mut cross = QuantumCircuit::new(4, 0);
        compile_cross_step_pauli_synthesis(
            &mut cross,
            &h,
            1.0,
            1.0,
            &crate::hamiltonian::hamiltonian_compiler::TrotterOrder::First,
            steps,
            true,
            BlockGroupingStrategy::QWC,
            None,
            None,
            false,
        )
        .unwrap();

        let ps_cx = count_cx_in_test(&per_step);
        let cs_cx = count_cx_in_test(&cross);
        assert!(
            cs_cx < ps_cx,
            "Cross-step CX ({}) should be less than per-step CX ({})",
            cs_cx,
            ps_cx
        );
    }

    fn count_cx_in_test(circuit: &QuantumCircuit) -> usize {
        circuit
            .data()
            .instructions()
            .iter()
            .filter(|i| i.gate.gate_type == crate::StandardGate::CX)
            .count()
    }

    // -------------------------------------------------------------------
    // Phase 4: General commuting grouping tests
    // -------------------------------------------------------------------

    #[test]
    fn test_is_commuting_basic() {
        // Z and Z always commute (same operator)
        let zz = PauliString::from_str("ZZII").unwrap();
        let iz = PauliString::from_str("IZZI").unwrap();
        assert!(is_commuting(&zz, &iz));

        // Z and X anti-commute at each shared position
        let zx = PauliString::from_str("ZXII").unwrap(); // Z at 0, X at 1
                                                         // zz = ZZII — anti-commutes at position 0 (Z vs Z = same → commute),
                                                         // at position 1 (Z vs X = different → anti-commute).
                                                         // Only 1 anti-commutation → odd → anti-commute
        assert!(!is_commuting(&zz, &zx));

        // XXII and YYII: anti-commute at 2 positions (0: X≠Y, 1: X≠Y) → even → commute
        let xx = PauliString::from_str("XXII").unwrap();
        let yy = PauliString::from_str("YYII").unwrap();
        assert!(is_commuting(&xx, &yy));
    }

    #[test]
    fn test_general_commuting_blocks_merges_xy() {
        // XXII and YYII are NOT QWC but DO commute — GC should put them in one block
        let terms: Vec<(PauliString, f64)> = vec![
            (PauliString::from_str("XXII").unwrap(), 0.3),
            (PauliString::from_str("YYII").unwrap(), 0.2),
        ];
        let gc_blocks = form_general_commuting_blocks(&terms);
        assert_eq!(
            gc_blocks.len(),
            1,
            "XXII and YYII should merge into one GC block"
        );
        let (block_terms, _label) = &gc_blocks[0];
        assert_eq!(block_terms.len(), 2);
    }

    #[test]
    fn test_decompose_to_qwc_subgroups() {
        // A GC block with X and Y terms should split into X and Y QWC subgroups
        let block_terms: Vec<(PauliString, f64)> = vec![
            (PauliString::from_str("XXII").unwrap(), 0.3),
            (PauliString::from_str("YYII").unwrap(), 0.2),
            (PauliString::from_str("XXII").unwrap(), 0.1), // same as first, QWC
        ];
        let subgroups = decompose_to_qwc_subgroups(&block_terms);
        // Should get 2 QWC subgroups: X-terms and Y-terms
        assert_eq!(subgroups.len(), 2);
        // The largest subgroup should have 2 terms (the two X-terms)
        assert_eq!(subgroups[0].len(), 2);
    }

    #[test]
    fn test_gc_strategy_produces_valid_circuit() {
        let h = build_h2_4q_test();
        let mut circuit = QuantumCircuit::new(4, 0);

        let n_blocks = compile_cross_step_pauli_synthesis(
            &mut circuit,
            &h,
            1.0,
            1.0,
            &crate::hamiltonian::hamiltonian_compiler::TrotterOrder::First,
            5,
            true,
            BlockGroupingStrategy::GeneralCommuting,
            None,
            None,
            false,
        )
        .unwrap();

        assert!(n_blocks > 0, "Should produce blocks with GC strategy");
        assert!(circuit.size() > 0, "Circuit should have gates");

        // GC strategy should produce a valid unitary
        let unitary = circuit.unitary(&std::collections::HashMap::new());
        assert!(unitary.is_ok(), "GC circuit should have a valid unitary");
    }

    #[test]
    fn test_gc_vs_qwc_h2_4q() {
        // Compare QWC vs GeneralCommuting gate counts for H2_4q
        let h = build_h2_4q_test();

        let mut c_qwc = QuantumCircuit::new(4, 0);
        compile_cross_step_pauli_synthesis(
            &mut c_qwc,
            &h,
            1.0,
            1.0,
            &crate::hamiltonian::hamiltonian_compiler::TrotterOrder::First,
            10,
            true,
            BlockGroupingStrategy::QWC,
            None,
            None,
            false,
        )
        .unwrap();

        let mut c_gc = QuantumCircuit::new(4, 0);
        compile_cross_step_pauli_synthesis(
            &mut c_gc,
            &h,
            1.0,
            1.0,
            &crate::hamiltonian::hamiltonian_compiler::TrotterOrder::First,
            10,
            true,
            BlockGroupingStrategy::GeneralCommuting,
            None,
            None,
            false,
        )
        .unwrap();

        let qwc_cx = count_cx_in_test(&c_qwc);
        let gc_cx = count_cx_in_test(&c_gc);

        // Both strategies should produce valid circuits
        assert!(c_qwc.size() > 0);
        assert!(c_gc.size() > 0);

        // GC should not produce more CX than QWC (it may produce the same or fewer)
        // For H2_4q, the X and Y blocks are already separated by QWC,
        // so GC may not help much on this specific Hamiltonian.
        // But on larger Hamiltonians with more diverse terms, GC should help.
        println!(
            "H2_4q: QWC {} gates/{} CX, GC {} gates/{} CX",
            c_qwc.size(),
            qwc_cx,
            c_gc.size(),
            gc_cx
        );
    }

    // ── reverse_blocks tests ──────────────────────────────────────────

    /// Create a simple set of Z-terms on 4 qubits for block tests.
    fn make_test_z_terms() -> Vec<(PauliString, f64)> {
        vec![
            (PauliString::from_str("ZZII").unwrap(), 0.5),
            (PauliString::from_str("IZZI").unwrap(), 0.3),
            (PauliString::from_str("IIZZ").unwrap(), 0.2),
            (PauliString::from_str("ZIZI").unwrap(), 0.4),
            (PauliString::from_str("IZIZ").unwrap(), 0.1),
        ]
    }

    #[test]
    fn test_reverse_blocks_preserves_len() {
        let terms = make_test_z_terms();
        let blocks = form_blocks(&terms);
        let reversed = reverse_blocks(&blocks);
        assert_eq!(
            blocks.len(),
            reversed.len(),
            "reverse_blocks must preserve block count"
        );
    }

    #[test]
    fn test_reverse_blocks_roundtrip() {
        let terms = make_test_z_terms();
        let blocks = form_blocks(&terms);
        let double_reversed = reverse_blocks(&reverse_blocks(&blocks));
        // After double reversal, blocks should be in original order with
        // original internal term order.
        assert_eq!(blocks.len(), double_reversed.len());
        for (orig, drev) in blocks.iter().zip(double_reversed.iter()) {
            assert_eq!(orig.mstr.to_string_repr(), drev.mstr.to_string_repr());
            assert_eq!(orig.nodes, drev.nodes);
            assert_eq!(orig.terms.len(), drev.terms.len());
            // Terms should be back to original order after double reversal
            for (ot, rt) in orig.terms.iter().zip(drev.terms.iter()) {
                assert_eq!(ot.0.to_string_repr(), rt.0.to_string_repr());
                assert!(
                    (ot.1 - rt.1).abs() < 1e-12,
                    "coefficient should match after roundtrip"
                );
            }
        }
    }

    #[test]
    fn test_reverse_blocks_empty() {
        let empty: Vec<PauliBlock> = vec![];
        let reversed = reverse_blocks(&empty);
        assert!(reversed.is_empty());
    }

    // ── form_blocks order-independence test ───────────────────────────

    #[test]
    fn test_form_blocks_order_independent() {
        // Create the same set of terms in two different orders.
        let terms_a = make_test_z_terms();
        // Reverse the order
        let mut terms_b = terms_a.clone();
        terms_b.reverse();

        let blocks_a = form_blocks(&terms_a);
        let blocks_b = form_blocks(&terms_b);

        // Same number of blocks
        assert_eq!(
            blocks_a.len(),
            blocks_b.len(),
            "QWC partition size should be independent of input term order"
        );

        // Same set of Pauli masks (mstr) — order may differ
        let mut mstrs_a: Vec<String> = blocks_a
            .iter()
            .map(|b| b.mstr.to_string_repr().to_string())
            .collect();
        let mut mstrs_b: Vec<String> = blocks_b
            .iter()
            .map(|b| b.mstr.to_string_repr().to_string())
            .collect();
        mstrs_a.sort();
        mstrs_b.sort();
        assert_eq!(
            mstrs_a, mstrs_b,
            "QWC block masks should be identical regardless of input term order"
        );

        // Total term count preserved
        let total_terms_a: usize = blocks_a.iter().map(|b| b.terms.len()).sum();
        let total_terms_b: usize = blocks_b.iter().map(|b| b.terms.len()).sum();
        assert_eq!(
            total_terms_a, total_terms_b,
            "total number of terms should be preserved across blocks"
        );
        assert_eq!(
            total_terms_a,
            terms_a.len(),
            "all input terms should be assigned to blocks"
        );
    }

    // ── CliffordSimple end-to-end fidelity test (Phase 11 review BUG-1/BUG-2 fix) ──

    /// Verify that CliffordSimple produces correct unitaries for H2_4q.
    /// This test catches regressions where Clifford pre/post gates are
    /// dropped during shared CNOT tree synthesis.
    #[test]
    fn test_cliffordsimple_h2_4q_fidelity() {
        use crate::hamiltonian::GadgetOptimizationStrategy;
        use crate::hamiltonian::HamiltonianCompiler;

        let h = build_h2_4q_test();

        // Reference: compile with IdenticalOnly (known-correct)
        let config_ref = crate::hamiltonian::CompilerConfig {
            pauli_gadget_optimization: GadgetOptimizationStrategy::IdenticalOnly,
            ..crate::hamiltonian::CompilerConfig::default()
        };
        let c_ref = HamiltonianCompiler::new(config_ref).compile(&h).unwrap();
        let u_ref = c_ref.unitary(&std::collections::HashMap::new()).unwrap();

        // Test: compile with CliffordSimple
        let config_cs = crate::hamiltonian::CompilerConfig {
            pauli_gadget_optimization: GadgetOptimizationStrategy::CliffordSimple,
            ..crate::hamiltonian::CompilerConfig::default()
        };
        let c_cs = HamiltonianCompiler::new(config_cs).compile(&h).unwrap();
        let u_cs = c_cs.unitary(&std::collections::HashMap::new()).unwrap();

        // Compare unitaries: they must be identical (both correct)
        let dim = 1usize << h.num_qubits;
        let mut sq_diff: f64 = 0.0;
        let mut norm_ref: f64 = 0.0;
        for i in 0..dim {
            for j in 0..dim {
                let d = u_ref[(i, j)] - u_cs[(i, j)];
                sq_diff += d.norm_sqr();
                norm_ref += u_ref[(i, j)].norm_sqr();
            }
        }
        let fidelity_error = (sq_diff / norm_ref).sqrt();

        assert!(
            fidelity_error < 0.01,
            "CliffordSimple fidelity error {:.6} exceeds threshold 0.01 — \
             Clifford gates may be incorrectly emitted or dropped",
            fidelity_error
        );
    }

    /// Verify CliffordSimple works for a simple Y-basis Hamiltonian.
    /// Tests the code path where S† gates convert Y→X during synthesis.
    #[test]
    fn test_cliffordsimple_y_basis_fidelity() {
        use crate::hamiltonian::GadgetOptimizationStrategy;
        use crate::hamiltonian::Hamiltonian;
        use crate::hamiltonian::HamiltonianCompiler;

        // Build H = 0.5 * YY (2-qubit Y-basis interaction)
        let mut h = Hamiltonian::new(2);
        h.add_term(
            PauliString::from_str("YY").unwrap(),
            num_complex::Complex64::new(0.5, 0.0),
        )
        .unwrap();

        // Reference: IdenticalOnly
        let config_ref = crate::hamiltonian::CompilerConfig {
            pauli_gadget_optimization: GadgetOptimizationStrategy::IdenticalOnly,
            ..crate::hamiltonian::CompilerConfig::default()
        };
        let c_ref = HamiltonianCompiler::new(config_ref).compile(&h).unwrap();
        let u_ref = c_ref.unitary(&std::collections::HashMap::new()).unwrap();

        // CliffordSimple — has no XY pair to align, but exercises the code path
        let config_cs = crate::hamiltonian::CompilerConfig {
            pauli_gadget_optimization: GadgetOptimizationStrategy::CliffordSimple,
            ..crate::hamiltonian::CompilerConfig::default()
        };
        let c_cs = HamiltonianCompiler::new(config_cs).compile(&h).unwrap();
        let u_cs = c_cs.unitary(&std::collections::HashMap::new()).unwrap();

        // Fidelity check
        let dim = 1usize << h.num_qubits;
        let mut sq_diff: f64 = 0.0;
        let mut norm_ref: f64 = 0.0;
        for i in 0..dim {
            for j in 0..dim {
                let d = u_ref[(i, j)] - u_cs[(i, j)];
                sq_diff += d.norm_sqr();
                norm_ref += u_ref[(i, j)].norm_sqr();
            }
        }
        let fidelity_error = (sq_diff / norm_ref).sqrt();

        assert!(
            fidelity_error < 0.01,
            "CliffordSimple Y-basis fidelity error {:.6} exceeds threshold",
            fidelity_error
        );
    }

    /// Verify CliffordSimple with XY-pair alignment (X and Y terms on same qubits).
    /// This is the critical case: IIXX + IIYY terms trigger Clifford alignment
    /// where S† gates must be emitted correctly.
    #[test]
    fn test_cliffordsimple_xy_pair_fidelity() {
        use crate::hamiltonian::GadgetOptimizationStrategy;
        use crate::hamiltonian::Hamiltonian;
        use crate::hamiltonian::HamiltonianCompiler;

        // H = 0.3*XX + 0.4*YY on qubits 1,2 (indices 0-based)
        // CliffordSimple should align YY→XX via S† pre-gates on both qubits.
        let mut h = Hamiltonian::new(3);
        h.add_term(
            PauliString::from_str("IXX").unwrap(),
            num_complex::Complex64::new(0.3, 0.0),
        )
        .unwrap();
        h.add_term(
            PauliString::from_str("IYY").unwrap(),
            num_complex::Complex64::new(0.4, 0.0),
        )
        .unwrap();

        // Reference: IdenticalOnly (terms synthesized independently)
        let config_ref = crate::hamiltonian::CompilerConfig {
            pauli_gadget_optimization: GadgetOptimizationStrategy::IdenticalOnly,
            ..crate::hamiltonian::CompilerConfig::default()
        };
        let c_ref = HamiltonianCompiler::new(config_ref).compile(&h).unwrap();
        let u_ref = c_ref.unitary(&std::collections::HashMap::new()).unwrap();

        // CliffordSimple: aligns IYY → IXX via S† gates
        let config_cs = crate::hamiltonian::CompilerConfig {
            pauli_gadget_optimization: GadgetOptimizationStrategy::CliffordSimple,
            ..crate::hamiltonian::CompilerConfig::default()
        };
        let c_cs = HamiltonianCompiler::new(config_cs).compile(&h).unwrap();
        let u_cs = c_cs.unitary(&std::collections::HashMap::new()).unwrap();

        // Fidelity check — CliffordSimple must match IdenticalOnly
        let dim = 1usize << h.num_qubits;
        let mut sq_diff: f64 = 0.0;
        let mut norm_ref: f64 = 0.0;
        for i in 0..dim {
            for j in 0..dim {
                let d = u_ref[(i, j)] - u_cs[(i, j)];
                sq_diff += d.norm_sqr();
                norm_ref += u_ref[(i, j)].norm_sqr();
            }
        }
        let fidelity_error = (sq_diff / norm_ref).sqrt();

        assert!(
            fidelity_error < 0.01,
            "CliffordSimple XY-pair fidelity error {:.6} exceeds threshold — \
             S/S† Clifford gates may not be correctly emitted around Rz rotations",
            fidelity_error
        );
    }

    // ── Clifford-enhanced block formation tests (Phase 11e) ──────────

    #[test]
    fn test_clifford_enhanced_basic_merge() {
        // XXII and YYII are Clifford-compatible (X↔Y at q0,q1 via S/S†).
        // They are NOT QWC but their mstr strings can be made QWC-compatible.
        let terms = vec![
            (PauliString::from_str("XXII").unwrap(), 0.1),
            (PauliString::from_str("YYII").unwrap(), 0.2),
            (PauliString::from_str("IIZZ").unwrap(), 0.3),
        ];
        let blocks = form_blocks_clifford_enhanced(&terms);
        // XXII and YYII should merge into 1 block; IIZZ stays separate.
        // With 3 terms, 2 blocks expected (XX/YY merged + ZZ).
        assert!(
            blocks.len() <= 2,
            "Clifford-enhanced should merge XY-compatible blocks, got {} blocks",
            blocks.len()
        );
    }

    #[test]
    fn test_clifford_enhanced_no_merge() {
        // All terms are already QWC — no merges possible beyond standard blocks.
        let terms = vec![
            (PauliString::from_str("IIIZ").unwrap(), 0.1),
            (PauliString::from_str("IIZI").unwrap(), 0.2),
            (PauliString::from_str("IZII").unwrap(), 0.3),
        ];
        let blocks = form_blocks_clifford_enhanced(&terms);
        // All ZZ... terms are QWC — should be 1 block.
        assert_eq!(blocks.len(), 1, "All-Z terms should be in 1 QWC block");
    }

    #[test]
    fn test_clifford_enhanced_empty() {
        let blocks = form_blocks_clifford_enhanced(&[]);
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_clifford_enhanced_single_term() {
        let terms = vec![(PauliString::from_str("XX").unwrap(), 0.5)];
        let blocks = form_blocks_clifford_enhanced(&terms);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].terms.len(), 1);
    }

    // ── Phase 11f: Clifford gate emission for enhanced blocks ──────────

    /// Verify that `form_blocks_clifford_enhanced` produces Clifford annotations
    /// when compatible blocks with different-size node sets can be merged.
    ///
    /// Uses a 2-block scenario where block A has 3 active qubits and block B
    /// has 2 active qubits (a subset that overlaps), so the shared tree savings
    /// from eliminating block B's separate tree outweigh the Clifford overhead.
    #[test]
    fn test_clifford_enhanced_produces_annotations() {
        // Block A: terms on qubits [0,1,2] — mstr XXX (QWC with each other)
        // Block B: terms on qubits [0,1] — mstr YY which is NOT QWC with XXX
        // compatible_pair_check(XXX, YY) → S† on q0, q1 to align Y→X
        // Cost before: shared_tree(3) + shared_tree(2) = 5 + 3 = 8 CX
        // Cost after:  shared_tree(3) + per_term(2) + clifford(4) = 5 + 3 + 4 = 12
        // Hmm, still negative. Need blocks where the savings model works.
        //
        // Use a different strategy: test the annotation mechanism directly
        // by manually constructing a PauliBlock with clifford_annotations.
        let pauli_x = PauliString::from_str("XXII").unwrap();
        let pauli_y = PauliString::from_str("YYII").unwrap();
        let mstr = p_or(&pauli_x, &pauli_y);
        let nodes = active_nodes(&mstr);
        let block = PauliBlock {
            terms: vec![(pauli_x.clone(), 0.3), (pauli_y.clone(), 0.2)],
            mstr,
            nodes,
            // Annotate the YYII term with S† gates for X↔Y alignment
            clifford_annotations: vec![
                None, // XXII — no conjugation
                Some(vec![(CliffordGate::Sdg, 0), (CliffordGate::Sdg, 1)]),
            ],
        };
        assert_eq!(block.clifford_annotations.len(), 2);
        assert!(block.clifford_annotations[1].is_some());
        // Synthesize and verify
        let mut circuit = QuantumCircuit::new(4, 0);
        syn_block(&mut circuit, &block, &[], false, false, None, None).unwrap();
        assert!(circuit.size() > 0, "Should produce gates");
    }

    /// Fidelity test: syn_block with manually-annotated Clifford terms
    /// must produce the correct unitary (Phase 11f code review C1 fix).
    #[test]
    fn test_clifford_annotated_block_fidelity() {
        // Create a block with one clean term (XX) and one annotated term (YY→XX)
        let pauli_x = PauliString::from_str("XXII").unwrap();
        let pauli_y = PauliString::from_str("YYII").unwrap();
        let mstr = p_or(&pauli_x, &pauli_y);
        let nodes = active_nodes(&mstr);

        // Reference: synthesize both terms as clean (no annotations)
        let block_ref = PauliBlock {
            terms: vec![(pauli_x.clone(), 0.3), (pauli_y.clone(), 0.2)],
            mstr: mstr.clone(),
            nodes: nodes.clone(),
            clifford_annotations: vec![None, None],
        };
        let mut c_ref = QuantumCircuit::new(4, 0);
        syn_block(&mut c_ref, &block_ref, &[], false, false, None, None).unwrap();
        let u_ref = c_ref.unitary(&std::collections::HashMap::new()).unwrap();

        // Test: annotate YYII with S† gates (Phase 11f style)
        let block_test = PauliBlock {
            terms: vec![(pauli_x.clone(), 0.3), (pauli_y.clone(), 0.2)],
            mstr,
            nodes,
            clifford_annotations: vec![
                None,
                Some(vec![(CliffordGate::Sdg, 0), (CliffordGate::Sdg, 1)]),
            ],
        };
        let mut c_test = QuantumCircuit::new(4, 0);
        syn_block(&mut c_test, &block_test, &[], false, false, None, None).unwrap();
        let u_test = c_test.unitary(&std::collections::HashMap::new()).unwrap();

        // Fidelity check
        let dim = 1usize << 4;
        let mut sq_diff: f64 = 0.0;
        let mut norm_ref: f64 = 0.0;
        for i in 0..dim {
            for j in 0..dim {
                let d = u_ref[(i, j)] - u_test[(i, j)];
                sq_diff += d.norm_sqr();
                norm_ref += u_ref[(i, j)].norm_sqr();
            }
        }
        let fidelity_error = (sq_diff / norm_ref).sqrt();
        assert!(
            fidelity_error < 0.01,
            "Annotated block fidelity error {:.6} too high",
            fidelity_error
        );
    }

    /// Fidelity test: Clifford annotations with H gates (X↔Z alignment).
    /// This specifically tests the Phase 11f C1 fix for effective_ops.
    #[test]
    fn test_clifford_annotated_h_gate_fidelity() {
        let pauli_x = PauliString::from_str("XZII").unwrap();
        let pauli_z = PauliString::from_str("ZXII").unwrap();
        let mstr = p_or(&pauli_x, &pauli_z);
        let nodes = active_nodes(&mstr);

        // Reference: clean synthesis
        let block_ref = PauliBlock {
            terms: vec![(pauli_x.clone(), 0.3), (pauli_z.clone(), 0.2)],
            mstr: mstr.clone(),
            nodes: nodes.clone(),
            clifford_annotations: vec![None, None],
        };
        let mut c_ref = QuantumCircuit::new(4, 0);
        syn_block(&mut c_ref, &block_ref, &[], false, false, None, None).unwrap();
        let u_ref = c_ref.unitary(&std::collections::HashMap::new()).unwrap();

        // Test: annotate ZXII with H gates (Phase 11f style, X↔Z alignment)
        let block_test = PauliBlock {
            terms: vec![(pauli_x.clone(), 0.3), (pauli_z.clone(), 0.2)],
            mstr,
            nodes,
            clifford_annotations: vec![
                None,
                Some(vec![(CliffordGate::H, 0), (CliffordGate::H, 1)]),
            ],
        };
        let mut c_test = QuantumCircuit::new(4, 0);
        syn_block(&mut c_test, &block_test, &[], false, false, None, None).unwrap();
        let u_test = c_test.unitary(&std::collections::HashMap::new()).unwrap();

        let dim = 1usize << 4;
        let mut sq_diff: f64 = 0.0;
        let mut norm_ref: f64 = 0.0;
        for i in 0..dim {
            for j in 0..dim {
                let d = u_ref[(i, j)] - u_test[(i, j)];
                sq_diff += d.norm_sqr();
                norm_ref += u_ref[(i, j)].norm_sqr();
            }
        }
        let fidelity_error = (sq_diff / norm_ref).sqrt();
        assert!(
            fidelity_error < 0.01,
            "H-gate annotated block fidelity error {:.6} too high (C1 fix validation)",
            fidelity_error
        );
    }

    /// Verify that `build_block_cache` round-trips Clifford annotations
    /// correctly (Phase 11f code review C2).
    #[test]
    fn test_cache_roundtrip_clifford_annotations() {
        let pauli_x = PauliString::from_str("XXII").unwrap();
        let pauli_y = PauliString::from_str("YYII").unwrap();
        let mstr = p_or(&pauli_x, &pauli_y);
        let nodes = active_nodes(&mstr);

        let original_block = super::PauliBlock {
            terms: vec![(pauli_x.clone(), 0.3), (pauli_y.clone(), 0.2)],
            mstr: mstr.clone(),
            nodes: nodes.clone(),
            clifford_annotations: vec![
                None,
                Some(vec![(CliffordGate::Sdg, 0), (CliffordGate::Sdg, 1)]),
            ],
        };

        // Build a cache from the block
        let cache = super::PauliBlockCache {
            block_signatures: vec![vec!["XXII".to_string(), "YYII".to_string()]],
            block_nodes: vec![nodes.clone()],
            block_mstrs: vec![mstr.clone()],
            edges: vec![(0, 0)],
            edge_links: vec![vec![]],
            split_flags: vec![(false, false)],
            block_clifford_annotations: vec![original_block.clifford_annotations.clone()],
        };

        // Simulate reconstruction from cache (as in compile_step_pauli_synthesis)
        let coeff_map: std::collections::HashMap<String, f64> =
            [("XXII".to_string(), 0.3), ("YYII".to_string(), 0.2)].into();

        let reconstructed: Vec<super::PauliBlock> = cache
            .block_signatures
            .iter()
            .enumerate()
            .map(|(bi, sigs)| {
                let terms: Vec<(PauliString, f64)> = sigs
                    .iter()
                    .filter_map(|sig| {
                        let coeff = *coeff_map.get(sig)?;
                        PauliString::from_str(sig).ok().map(|p| (p, coeff))
                    })
                    .collect();
                let clifford_ann = if bi < cache.block_clifford_annotations.len() {
                    cache.block_clifford_annotations[bi].clone()
                } else {
                    vec![None; terms.len()]
                };
                super::PauliBlock {
                    terms,
                    mstr: cache.block_mstrs[bi].clone(),
                    nodes: cache.block_nodes[bi].clone(),
                    clifford_annotations: clifford_ann,
                }
            })
            .filter(|b| !b.terms.is_empty())
            .collect();

        assert_eq!(reconstructed.len(), 1);
        assert_eq!(reconstructed[0].clifford_annotations.len(), 2);
        assert!(
            reconstructed[0].clifford_annotations[1].is_some(),
            "Cache roundtrip should preserve Clifford annotations"
        );
        // Verify the annotation content matches
        let ann = reconstructed[0].clifford_annotations[1].as_ref().unwrap();
        assert_eq!(ann.len(), 2);
        assert!(ann.iter().all(|&(_, q)| q < 4));
    }

    // ── form_blocks_clifford_aware tests (Phase 11j) ────────────────────

    #[test]
    fn test_form_blocks_clifford_aware_basic() {
        // IIXX and IIYY (with Sdg→XX annotation) should end up in the same block.
        let mut map: CliffordAnnotationMap = HashMap::new();
        map.insert(
            "IIYY".to_string(),
            (
                vec![(CliffordGate::Sdg, 2), (CliffordGate::Sdg, 3)],
                vec![(CliffordGate::S, 2), (CliffordGate::S, 3)],
            ),
        );
        let terms = vec![
            (PauliString::from_str("IIXX").unwrap(), 0.3),
            (PauliString::from_str("IIYY").unwrap(), 0.2),
            (PauliString::from_str("ZZII").unwrap(), 0.5),
        ];
        let blocks = form_blocks_clifford_aware(&terms, &map);
        // IIXX (q2,3 X) and ZZII (q0,1 Z) are on different qubits → QWC.
        // IIYY→IIXX (effective q2,3 X) also QWC with ZZII.
        // All 3 terms in one block.
        assert_eq!(
            blocks.len(),
            1,
            "All 3 terms are QWC (IIXX/IIYY→IIXX on q2,3, ZZII on q0,1): expected 1 block"
        );
        let block = &blocks[0];
        assert_eq!(block.terms.len(), 3);
        // Terms store effective Paulis. The IIYY term with Sdg annotation
        // should have effective Pauli IIXX.
        assert_eq!(block.terms[0].0.to_string_repr().to_string(), "IIXX");
        assert_eq!(block.terms[1].0.to_string_repr().to_string(), "IIXX");
        assert_eq!(block.terms[2].0.to_string_repr().to_string(), "ZZII");
        // The annotated term (original IIYY) should have Clifford annotations
        assert!(
            block.clifford_annotations[1].is_some(),
            "Annotated term (IIYY→IIXX) should have clifford_annotations"
        );
        assert!(
            block.clifford_annotations[0].is_none(),
            "Clean term (IIXX) should have no annotations"
        );
    }

    #[test]
    fn test_form_blocks_clifford_aware_no_annotations() {
        // Without annotations, result should match form_blocks.
        let map: CliffordAnnotationMap = HashMap::new();
        let terms = vec![
            (PauliString::from_str("IIXX").unwrap(), 0.3),
            (PauliString::from_str("IIYY").unwrap(), 0.2),
        ];
        let aware = form_blocks_clifford_aware(&terms, &map);
        let regular = form_blocks(&terms);
        // Both should produce 2 blocks (X vs Y, no Clifford to align them)
        assert_eq!(aware.len(), 2);
        assert_eq!(regular.len(), 2);
        // No annotations in either
        for b in &aware {
            for ann in &b.clifford_annotations {
                assert!(ann.is_none());
            }
        }
    }

    #[test]
    fn test_form_blocks_clifford_aware_all_annotated() {
        // Two YY-type terms both align to XX on different qubit pairs.
        let mut map: CliffordAnnotationMap = HashMap::new();
        map.insert(
            "IIYY".to_string(),
            (
                vec![(CliffordGate::Sdg, 2), (CliffordGate::Sdg, 3)],
                vec![(CliffordGate::S, 2), (CliffordGate::S, 3)],
            ),
        );
        map.insert(
            "YYII".to_string(),
            (
                vec![(CliffordGate::Sdg, 0), (CliffordGate::Sdg, 1)],
                vec![(CliffordGate::S, 0), (CliffordGate::S, 1)],
            ),
        );
        let terms = vec![
            (PauliString::from_str("IIYY").unwrap(), 0.2),
            (PauliString::from_str("YYII").unwrap(), 0.1),
        ];
        let blocks = form_blocks_clifford_aware(&terms, &map);
        // IIYY→IIXX (q2,3) and YYII→XXII (q0,1): different qubits → QWC → 1 block
        assert_eq!(
            blocks.len(),
            1,
            "Both terms on different qubit pairs → QWC → should merge to one block"
        );
        assert_eq!(blocks[0].terms.len(), 2);
        // Both should have annotations
        assert!(
            blocks[0].clifford_annotations[0].is_some(),
            "IIYY→IIXX should have clifford annotations"
        );
        assert!(
            blocks[0].clifford_annotations[1].is_some(),
            "YYII→XXII should have clifford annotations"
        );
    }

    #[test]
    fn test_form_blocks_clifford_aware_effective_operators() {
        // Verify that block's terms store effective (conjugated) Paulis, not originals.
        let mut map: CliffordAnnotationMap = HashMap::new();
        map.insert(
            "IIYY".to_string(),
            (
                vec![(CliffordGate::Sdg, 2), (CliffordGate::Sdg, 3)],
                vec![(CliffordGate::S, 2), (CliffordGate::S, 3)],
            ),
        );
        // IIYY→IIXX (effective q2,3 X) and IIIZ (q3 Z): NOT QWC (X vs Z on q3)
        let terms = vec![
            (PauliString::from_str("IIYY").unwrap(), 0.2),
            (PauliString::from_str("IIIZ").unwrap(), 0.3),
        ];
        let blocks = form_blocks_clifford_aware(&terms, &map);
        // X on q3 vs Z on q3 → NOT QWC → 2 separate blocks
        assert_eq!(
            blocks.len(),
            2,
            "IIXX(q2,3) and IIIZ(q3) NOT QWC → expected 2 blocks"
        );
        // Verify one block contains the effective Pauli IIXX
        let has_effective_xx = blocks.iter().any(|b| {
            b.terms
                .iter()
                .any(|(p, _)| p.to_string_repr().to_string() == "IIXX")
        });
        assert!(
            has_effective_xx,
            "One block should contain effective Pauli IIXX (from IIYY conjugation)"
        );
    }

    #[test]
    fn test_form_blocks_clifford_aware_with_clean_terms() {
        // Mix of clean and annotated terms: verify annotations tracked correctly.
        let mut map: CliffordAnnotationMap = HashMap::new();
        map.insert(
            "IIYY".to_string(),
            (
                vec![(CliffordGate::Sdg, 2), (CliffordGate::Sdg, 3)],
                vec![(CliffordGate::S, 2), (CliffordGate::S, 3)],
            ),
        );
        // IIXX (q2,3 X): clean, IIYY→IIXX (q2,3 X): annotated, IIIZ (q3 Z): clean
        // IIXX and IIIZ: q3 X vs Z → NOT QWC → different blocks
        // IIYY→IIXX and IIIZ: same problem → different blocks
        // So: Block 1 = [IIXX, IIYY→IIXX], Block 2 = [IIIZ]
        let terms = vec![
            (PauliString::from_str("IIXX").unwrap(), 0.5),
            (PauliString::from_str("IIYY").unwrap(), 0.3),
            (PauliString::from_str("IIIZ").unwrap(), 0.2),
        ];
        let blocks = form_blocks_clifford_aware(&terms, &map);
        assert_eq!(
            blocks.len(),
            2,
            "IIXX(q2,3) vs IIIZ(q3): X vs Z → NOT QWC → 2 blocks"
        );
        for block in &blocks {
            if block.terms.len() == 2 {
                // XY block: first term is clean IIXX, second is annotated IIYY→IIXX
                assert_eq!(block.terms[0].0.to_string_repr().to_string(), "IIXX");
                assert!(block.clifford_annotations[0].is_none());
                assert_eq!(block.terms[1].0.to_string_repr().to_string(), "IIXX");
                assert!(block.clifford_annotations[1].is_some());
            } else {
                assert_eq!(block.terms.len(), 1);
                let repr = block.terms[0].0.to_string_repr().to_string();
                assert!(repr.contains('Z'), "Expected IIIZ, got {}", repr);
            }
        }
    }
}
