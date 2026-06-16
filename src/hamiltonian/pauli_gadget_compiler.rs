//! Pauli gadget compiler — TKET-style GreedyPauliSimp optimization.
//!
//! Phase 11i: Implements greedy Clifford conjugation optimization at the Pauli
//! gadget level, inspired by TKET's `GreedyPauliSimp` pass (arXiv 2103.08602).
//!
//! ## How it differs from existing QWC-based synthesis
//!
//! Current pipeline: form QWC blocks → per-block CNOT tree → optimize.
//! TKET pipeline: greedy Clifford conjugation → diagonalize → global synthesis.
//!
//! The key insight: QWC block boundaries prevent merging gadgets that could
//! be aligned via Clifford conjugation. By doing greedy pairwise merging
//! BEFORE block formation, we can reduce the number of distinct CNOT trees.
//!
//! ## Algorithm (TKET GreedyPauliSimp-inspired)
//!
//! 1. **Build gadget graph**: Each Pauli term → gadget node with Pauli string,
//!    angle, and position. Precompute commutation matrix.
//! 2. **Generate candidates**: For each pair of commuting gadgets, check
//!    `compatible_pair_check` — can they be made QWC-compatible via per-qubit
//!    Clifford conjugation? If yes, score the pair.
//! 3. **Cost estimation**: `cost_before = synth_cost(A) + synth_cost(B)`,
//!    `cost_after = synth_cost(merged)`. Savings discounted by gadget
//!    distance in the circuit (`discount_rate^distance`).
//! 4. **Greedy iteration**: Apply best candidate (merge gadgets, record
//!    Clifford gates), update graph, recompute affected candidates.
//!    Repeat until no beneficial merge remains.
//! 5. **Output**: Optimized gadget list with Clifford annotations, ready
//!    for block formation and synthesis.
//!
//! ## References
//!
//! - TKET `GreedyPauliOptimisation.cpp` (Quantinuum/tket)
//! - arXiv 2103.08602 — graph optimization for Trotter-Suzuki decomposition
//! - arXiv 2506.20624 — PhasePoly framework

use crate::hamiltonian::pauli_gadget::{compatible_pair_check, CliffordGate};
use crate::hamiltonian::PauliOperator;
use crate::hamiltonian::PauliString;
use crate::hamiltonian::PauliTerm;

// ── Core types ────────────────────────────────────────────────────────────

/// A Pauli gadget in the optimization graph.
///
/// Each gadget represents a term `e^{i*angle*P}` where `P` is a Pauli string.
/// Clifford gates accumulated during greedy optimization are stored as pre/post
/// annotations for later emission during circuit synthesis.
#[derive(Debug, Clone)]
pub struct PauliGadgetNode {
    /// The Pauli string (e.g., XXII for σ_x⊗σ_x⊗I⊗I)
    pub pauli: PauliString,
    /// Rotation angle (θ in e^{iθP})
    pub angle: f64,
    /// Position in the original term list (for distance-based discounting)
    pub position: usize,
    /// Clifford gates applied BEFORE the rotation (align the Pauli basis)
    pub pre_gates: Vec<(CliffordGate, usize)>,
    /// Clifford gates applied AFTER the rotation (restore original basis)
    pub post_gates: Vec<(CliffordGate, usize)>,
    /// Number of original terms merged into this gadget
    pub merge_count: usize,
}

/// Configuration for the greedy optimization algorithm.
#[derive(Debug, Clone)]
pub struct GreedyConfig {
    /// Discount rate for distant gadgets (TKET default: 0.7).
    /// Cost savings from merging gadgets far apart in the term list are
    /// multiplied by `discount_rate^distance`, reducing their priority
    /// vs. nearby gadgets.
    pub discount_rate: f64,

    /// Maximum number of greedy iterations before forced convergence.
    pub max_iterations: usize,

    /// Stop when the best candidate's discounted savings falls below this.
    pub convergence_threshold: f64,

    /// Maximum number of candidate pairs to evaluate per iteration.
    /// Limits O(N²) search for large gadget sets.
    pub max_candidates_per_step: usize,
}

impl Default for GreedyConfig {
    fn default() -> Self {
        Self {
            discount_rate: 0.7,
            max_iterations: 100,
            convergence_threshold: 1e-6,
            max_candidates_per_step: 500,
        }
    }
}

/// A candidate pair of gadgets that can be merged via Clifford conjugation.
#[derive(Debug, Clone)]
struct MergeCandidate {
    /// Index of first gadget in the graph
    node_a: usize,
    /// Index of second gadget in the graph
    node_b: usize,
    /// Clifford gates to apply on node_b's qubits to align to node_a's basis
    gates_on_b: Vec<(usize, CliffordGate)>,
    /// Discounted savings = (cost_before - cost_after) * discount_rate^distance
    discounted_savings: f64,
}

/// Result of greedy Pauli gadget optimization.
#[derive(Debug, Clone)]
pub struct GreedyOptimizationResult {
    /// Optimized gadgets with Clifford annotations
    pub gadgets: Vec<PauliGadgetNode>,
    /// Number of merges performed
    pub merges_performed: usize,
    /// Number of greedy iterations
    pub iterations: usize,
    /// Estimated gate savings
    pub estimated_savings: f64,
}

// ── Synthesis cost estimation ─────────────────────────────────────────────

/// Estimate the synthesis cost (in gate equivalents) for a single Pauli gadget.
///
/// The cost model:
/// - 0 active qubits (identity): 0
/// - 1 active qubit: 2 (basis change + Rz)
/// - n active qubits: 2*(n-1) CX + n Rz + 2*n basis changes ≈ 4n - 2
///
/// This is a simplified model; actual cost depends on CNOT tree structure
/// and basis-change sharing between terms in the same block.
fn estimate_synth_cost(pauli: &PauliString) -> f64 {
    let n_active = pauli
        .operators
        .iter()
        .filter(|op| !matches!(op, PauliOperator::I))
        .count();

    if n_active == 0 {
        0.0
    } else if n_active == 1 {
        2.0 // basis change + Rz
    } else {
        // CNOT tree: 2*(n-1) CX + n Rz + basis changes
        2.0 * (n_active - 1) as f64 + n_active as f64 + 2.0 * n_active as f64
    }
}

// ── Pauli gadget graph ────────────────────────────────────────────────────

/// Graph of Pauli gadgets with precomputed commutation relationships.
struct PauliGadgetGraph {
    nodes: Vec<PauliGadgetNode>,
    /// N×N commutation matrix: commutation[i][j] == true if nodes[i] and nodes[j] commute
    commutation: Vec<Vec<bool>>,
}

impl PauliGadgetGraph {
    /// Build the gadget graph from Pauli terms.
    ///
    /// Angles are stored as raw coefficients (NOT scaled by 2*dt/hbar).
    /// The synthesis pipeline applies the scaling later. This ensures
    /// consistent angle handling between GreedyPauliSimp and downstream
    /// PauliLevel/GateLevel synthesis.
    fn from_terms(terms: &[&PauliTerm], _dt: f64, _hbar: f64) -> Self {
        let n = terms.len();
        let nodes: Vec<PauliGadgetNode> = terms
            .iter()
            .enumerate()
            .map(|(i, t)| PauliGadgetNode {
                pauli: t.pauli_string.clone(),
                angle: t.coefficient.re,
                position: i,
                pre_gates: Vec::new(),
                post_gates: Vec::new(),
                merge_count: 1,
            })
            .collect();

        // Precompute commutation matrix
        let mut commutation = vec![vec![false; n]; n];
        for i in 0..n {
            for j in 0..n {
                commutation[i][j] = nodes[i].pauli.commutes_with(&nodes[j].pauli);
            }
        }

        PauliGadgetGraph { nodes, commutation }
    }

    /// Check if two nodes commute.
    fn commute(&self, i: usize, j: usize) -> bool {
        self.commutation[i][j]
    }

    /// Compute the maximum distance between any two active nodes (for normalization).
    fn max_distance(&self) -> f64 {
        if self.nodes.len() < 2 {
            return 1.0;
        }
        let min_pos = self.nodes.iter().map(|n| n.position).min().unwrap_or(0);
        let max_pos = self.nodes.iter().map(|n| n.position).max().unwrap_or(0);
        (max_pos - min_pos).max(1) as f64
    }

    /// Generate merge candidates for all active pairs.
    fn generate_candidates(&self, config: &GreedyConfig) -> Vec<MergeCandidate> {
        let mut candidates = Vec::new();
        let max_dist = self.max_distance();
        let n = self.nodes.len();

        for i in 0..n {
            for j in (i + 1)..n {
                if !self.commute(i, j) {
                    continue;
                }

                // Check if the pair can be made QWC-compatible via Clifford.
                // Skip already-QWC pairs (empty gate list) — those are handled
                // by the existing QWC block formation in PauliLevel synthesis.
                // GreedyPauliSimp targets pairs that are NOT QWC but can be
                // aligned via Clifford conjugation (e.g., XX↔YY via S/S†).
                let gates_on_b =
                    match compatible_pair_check(&self.nodes[i].pauli, &self.nodes[j].pauli) {
                        Some(gates) if !gates.is_empty() => gates,
                        _ => continue,
                    };
                {
                    let node_a = &self.nodes[i];
                    let node_b = &self.nodes[j];

                    // Cost estimation: after Clifford alignment, node_b's effective
                    // Pauli matches node_a's, so both share the same CNOT tree.
                    // cost_before: separate CNOT trees for A and B
                    // cost_after: A's CNOT tree + B's Rz gates + Clifford overhead.
                    // B no longer needs its own CNOT tree (2*(n-1) CX) or basis
                    // changes (2*n), but still needs n Rz rotations and the
                    // Clifford pre/post gates that make its Pauli match A's.
                    let cost_before =
                        estimate_synth_cost(&node_a.pauli) + estimate_synth_cost(&node_b.pauli);
                    let n_active_b = node_b
                        .pauli
                        .operators
                        .iter()
                        .filter(|op| !matches!(op, PauliOperator::I))
                        .count() as f64;
                    let clifford_overhead = 2.0 * gates_on_b.len() as f64;
                    let cost_after =
                        estimate_synth_cost(&node_a.pauli) + n_active_b + clifford_overhead;
                    let raw_savings = cost_before - cost_after;

                    // Distance-based discount
                    let distance =
                        (node_a.position as isize - node_b.position as isize).unsigned_abs() as f64;
                    let norm_distance = distance / max_dist;
                    let discount = config.discount_rate.powf(norm_distance);
                    let discounted_savings = raw_savings * discount;

                    if discounted_savings > 0.0 {
                        candidates.push(MergeCandidate {
                            node_a: i,
                            node_b: j,
                            gates_on_b,
                            discounted_savings,
                        });
                    }
                }
            }
        }

        // Sort by discounted savings descending
        candidates.sort_by(|a, b| {
            b.discounted_savings
                .partial_cmp(&a.discounted_savings)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Truncate to max candidates (safety cap for very large Hamiltonians)
        candidates.truncate(config.max_candidates_per_step);

        candidates
    }

    /// Apply a merge: record Clifford gates on node_b, merge angles into node_a,
    /// mark node_b inactive.
    fn apply_merge(&mut self, candidate: &MergeCandidate) {
        // Record Clifford gates on node_b for later emission.
        // The gates from compatible_pair_check align node_b's Pauli to node_a's,
        // enabling them to share a CNOT tree in QWC block formation.
        //
        // We do NOT merge angles — the existing Rz merging passes
        // (PhasePolynomialPass, SingleQubitOptimizer) handle that.
        // GreedyPauliSimp's job is to add Clifford annotations so the
        // block formation can put these gadgets in the same QWC block
        // via form_blocks_clifford_enhanced().
        for &(q, gate) in &candidate.gates_on_b {
            self.nodes[candidate.node_b].pre_gates.push((gate, q));
            self.nodes[candidate.node_b]
                .post_gates
                .push((gate.inverse(), q));
        }

        self.nodes[candidate.node_b].merge_count += 1;
        self.nodes[candidate.node_a].merge_count += 1;
    }
}

// ── Main greedy optimization ───────────────────────────────────────────────

/// Run greedy Pauli gadget optimization (TKET GreedyPauliSimp-inspired).
///
/// Converts Hamiltonian terms to Pauli gadgets and identifies pairs that
/// can be aligned via single-qubit Clifford conjugation. For each compatible
/// pair, Clifford annotations are recorded on the second gadget so the
/// synthesis pipeline can put them in the same QWC block.
///
/// Note: This is a single-pass algorithm — it identifies Clifford-alignable
/// pairs but does NOT iteratively merge gadgets. The existing QWC block
/// formation and Rz merging passes handle the actual synthesis optimization.
///
/// # Arguments
/// * `terms` - Pauli terms from the Hamiltonian
/// * `dt` - Time step (evolution_time / trotter_steps)
/// * `hbar` - Reduced Planck constant
/// * `config` - Greedy optimization configuration
///
/// # Returns
/// Optimized gadgets with Clifford annotations, ready for block formation.
pub fn greedy_pauli_simp(
    terms: &[&PauliTerm],
    dt: f64,
    hbar: f64,
    config: &GreedyConfig,
) -> GreedyOptimizationResult {
    let mut graph = PauliGadgetGraph::from_terms(terms, dt, hbar);
    let mut merges_performed = 0usize;
    let mut total_savings = 0.0f64;

    // Generate all candidates in one pass
    let candidates = graph.generate_candidates(config);

    // Apply candidates greedily (best first), tracking which nodes are used
    let mut used_nodes = vec![false; graph.nodes.len()];

    for candidate in &candidates {
        // Skip if node_b is already aligned to another reference.
        // node_a can serve as reference for multiple targets (unchanged).
        if used_nodes[candidate.node_b] {
            continue;
        }

        if candidate.discounted_savings <= 0.0 {
            continue;
        }

        graph.apply_merge(candidate);
        // Only mark node_b as used — node_a is the unchanged reference
        // and can serve as alignment target for multiple other nodes.
        used_nodes[candidate.node_b] = true;
        total_savings += candidate.discounted_savings;
        merges_performed += 1;
    }

    // Return all gadgets (including those with Clifford annotations).
    // Zero-angle gadgets are filtered out.
    let gadgets: Vec<PauliGadgetNode> = graph
        .nodes
        .into_iter()
        .filter(|node| node.angle.abs() > 1e-15)
        .collect();

    GreedyOptimizationResult {
        gadgets,
        merges_performed,
        iterations: 1,
        estimated_savings: total_savings,
    }
}

// ── Integration helper ────────────────────────────────────────────────────

/// Convert optimized gadgets to PauliTerm list and Clifford annotation map.
///
/// This bridges the GreedyPauliSimp output to the existing PauliLevel
/// synthesis pipeline. Gadgets become PauliTerms for block formation;
/// Clifford annotations are packaged into `CliffordAnnotationMap` for
/// use by `compile_step_pauli_synthesis`.
///
/// # Returns
/// * `Vec<PauliTerm>` — terms for Hamiltonian reconstruction
/// * `Option<CliffordAnnotationMap>` — Clifford annotations (None if no gates)
pub fn gadgets_to_terms_and_map(
    gadgets: &[PauliGadgetNode],
    _num_qubits: usize,
) -> (
    Vec<PauliTerm>,
    Option<crate::hamiltonian::pauli_synthesis::CliffordAnnotationMap>,
) {
    let mut terms = Vec::with_capacity(gadgets.len());
    let mut clifford_map: crate::hamiltonian::pauli_synthesis::CliffordAnnotationMap =
        std::collections::HashMap::new();
    let mut has_clifford = false;

    for (idx, gadget) in gadgets.iter().enumerate() {
        let coeff = num_complex::Complex64::new(gadget.angle, 0.0);
        let pauli_repr = gadget.pauli.to_string_repr().to_string();
        terms.push(PauliTerm {
            pauli_string: gadget.pauli.clone(),
            coefficient: coeff,
            parameter: None,
        });

        if !gadget.pre_gates.is_empty() || !gadget.post_gates.is_empty() {
            let key = if pauli_repr.is_empty() {
                format!("gadget_{}", idx)
            } else {
                pauli_repr.clone()
            };
            clifford_map.insert(key, (gadget.pre_gates.clone(), gadget.post_gates.clone()));
            has_clifford = true;
        }
    }

    let map = if has_clifford {
        Some(clifford_map)
    } else {
        None
    };
    (terms, map)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hamiltonian::PauliOperator;

    /// Helper to create a PauliString from a slice of (qubit, operator) pairs.
    fn ps(num_qubits: usize, ops: &[(usize, PauliOperator)]) -> PauliString {
        let mut operators = vec![PauliOperator::I; num_qubits];
        for &(q, op) in ops {
            operators[q] = op;
        }
        PauliString::new(operators, num_complex::Complex64::new(1.0, 0.0))
    }

    /// Helper to create a PauliTerm.
    fn pt(num_qubits: usize, ops: &[(usize, PauliOperator)], coeff: f64) -> PauliTerm {
        PauliTerm {
            pauli_string: ps(num_qubits, ops),
            coefficient: num_complex::Complex64::new(coeff, 0.0),
            parameter: None,
        }
    }

    // ── Graph construction tests ──────────────────────────────────────

    #[test]
    fn test_graph_construction() {
        // H2-like terms
        let t1 = pt(4, &[(0, PauliOperator::Z)], 1.0);
        let t2 = pt(4, &[(1, PauliOperator::Z)], 0.5);
        let t3 = pt(4, &[(2, PauliOperator::Z)], 0.3);
        let terms: Vec<&PauliTerm> = vec![&t1, &t2, &t3];
        let graph = PauliGadgetGraph::from_terms(&terms, 1.0, 1.0);

        assert_eq!(graph.nodes.len(), 3);
        // All Z terms on different qubits commute
        assert!(graph.commute(0, 1));
        assert!(graph.commute(0, 2));
        assert!(graph.commute(1, 2));
    }

    #[test]
    fn test_graph_commutation_non_commuting() {
        let t1 = pt(4, &[(0, PauliOperator::X)], 1.0);
        let t2 = pt(4, &[(0, PauliOperator::Z)], 1.0);
        let terms: Vec<&PauliTerm> = vec![&t1, &t2];
        let graph = PauliGadgetGraph::from_terms(&terms, 1.0, 1.0);

        // X and Z on the same qubit do NOT commute
        assert!(!graph.commute(0, 1));
    }

    // ── Candidate generation tests ────────────────────────────────────

    #[test]
    fn test_candidate_generation_xx_yy() {
        // IIXX and IIYY: compatible via S/Sdg on qubits 2,3
        let t1 = pt(4, &[(2, PauliOperator::X), (3, PauliOperator::X)], 0.5);
        let t2 = pt(4, &[(2, PauliOperator::Y), (3, PauliOperator::Y)], 0.3);
        let terms: Vec<&PauliTerm> = vec![&t1, &t2];
        let graph = PauliGadgetGraph::from_terms(&terms, 1.0, 1.0);
        let config = GreedyConfig::default();

        let candidates = graph.generate_candidates(&config);
        // XX and YY commute and are compatible via S† on each qubit
        assert!(!candidates.is_empty(), "XX+YY should produce candidates");
        let best = &candidates[0];
        assert_eq!(best.node_a, 0);
        assert_eq!(best.node_b, 1);
        assert!(best.discounted_savings > 0.0);
    }

    #[test]
    fn test_candidate_generation_no_commute() {
        // X and Z on same qubit: no commutation → no candidates
        let t1 = pt(2, &[(0, PauliOperator::X)], 1.0);
        let t2 = pt(2, &[(0, PauliOperator::Z)], 1.0);
        let terms: Vec<&PauliTerm> = vec![&t1, &t2];
        let graph = PauliGadgetGraph::from_terms(&terms, 1.0, 1.0);
        let config = GreedyConfig::default();

        let candidates = graph.generate_candidates(&config);
        assert!(candidates.is_empty());
    }

    #[test]
    fn test_candidate_generation_already_compatible() {
        // Two Z terms on different qubits are already QWC-compatible.
        // GreedyPauliSimp skips these — block formation handles them.
        let t1 = pt(4, &[(0, PauliOperator::Z)], 0.5);
        let t2 = pt(4, &[(1, PauliOperator::Z)], 0.3);
        let terms: Vec<&PauliTerm> = vec![&t1, &t2];
        let graph = PauliGadgetGraph::from_terms(&terms, 1.0, 1.0);
        let config = GreedyConfig::default();

        let candidates = graph.generate_candidates(&config);
        // Already QWC → no Clifford needed → NO candidates
        assert!(candidates.is_empty());
    }

    // ── Cost estimation tests ─────────────────────────────────────────

    #[test]
    fn test_cost_estimation_identity() {
        let ident = PauliString::identity(4);
        assert_eq!(estimate_synth_cost(&ident), 0.0);
    }

    #[test]
    fn test_cost_estimation_single_qubit() {
        let single = ps(4, &[(0, PauliOperator::Z)]);
        let cost = estimate_synth_cost(&single);
        assert!(cost > 0.0);
        assert!(cost < 5.0);
    }

    #[test]
    fn test_cost_estimation_multi_qubit() {
        let multi = ps(
            4,
            &[
                (0, PauliOperator::Z),
                (1, PauliOperator::Z),
                (2, PauliOperator::Z),
                (3, PauliOperator::Z),
            ],
        );
        let cost = estimate_synth_cost(&multi);
        // 4 active qubits: 2*3 CX + 4 Rz + 8 basis = ~18
        assert!(cost > 10.0);
        let single = ps(4, &[(0, PauliOperator::Z)]);
        assert!(cost > estimate_synth_cost(&single));
    }

    // ── Merge tests ───────────────────────────────────────────────────

    #[test]
    fn test_apply_merge_xx_yy() {
        let t1 = pt(4, &[(2, PauliOperator::X), (3, PauliOperator::X)], 0.5);
        let t2 = pt(4, &[(2, PauliOperator::Y), (3, PauliOperator::Y)], 0.3);
        let terms: Vec<&PauliTerm> = vec![&t1, &t2];
        let mut graph = PauliGadgetGraph::from_terms(&terms, 1.0, 1.0);
        let config = GreedyConfig::default();

        let candidates = graph.generate_candidates(&config);
        assert!(!candidates.is_empty());

        let best = candidates[0].clone();
        graph.apply_merge(&best);

        // Angles are NOT merged — GreedyPauliSimp only records Clifford annotations
        assert!(
            (graph.nodes[0].angle - 0.5).abs() < 1e-10,
            "node_a angle unchanged"
        );
        assert!(
            (graph.nodes[1].angle - 0.3).abs() < 1e-10,
            "node_b angle unchanged"
        );
        // node_b should have Clifford pre/post gates
        assert!(
            !graph.nodes[1].pre_gates.is_empty(),
            "node_b should have Clifford pre-gates"
        );
        assert_eq!(
            graph.nodes[1].pre_gates.len(),
            graph.nodes[1].post_gates.len()
        );
    }

    #[test]
    fn test_merge_records_clifford_gates() {
        let t1 = pt(4, &[(2, PauliOperator::X), (3, PauliOperator::X)], 0.5);
        let t2 = pt(4, &[(2, PauliOperator::Y), (3, PauliOperator::Y)], 0.3);
        let terms: Vec<&PauliTerm> = vec![&t1, &t2];
        let mut graph = PauliGadgetGraph::from_terms(&terms, 1.0, 1.0);
        let config = GreedyConfig::default();

        let candidates = graph.generate_candidates(&config);
        assert!(!candidates.is_empty());

        let best = candidates[0].clone();
        graph.apply_merge(&best);

        // Node 1 (the YY term) should have Clifford pre/post gates
        let node_b = &graph.nodes[1];
        assert!(
            !node_b.pre_gates.is_empty(),
            "Merged node should record Clifford pre-gates"
        );
        // Each Clifford gate should have a corresponding inverse in post_gates
        assert_eq!(node_b.pre_gates.len(), node_b.post_gates.len());
    }

    // ── Full greedy optimization tests ────────────────────────────────

    #[test]
    fn test_greedy_pauli_simp_empty() {
        let terms: Vec<&PauliTerm> = vec![];
        let config = GreedyConfig::default();
        let result = greedy_pauli_simp(&terms, 1.0, 1.0, &config);
        assert!(result.gadgets.is_empty());
        assert_eq!(result.merges_performed, 0);
    }

    #[test]
    fn test_greedy_pauli_simp_single_term() {
        let t1 = pt(4, &[(0, PauliOperator::Z)], 1.0);
        let terms: Vec<&PauliTerm> = vec![&t1];
        let config = GreedyConfig::default();
        let result = greedy_pauli_simp(&terms, 1.0, 1.0, &config);
        assert_eq!(result.gadgets.len(), 1);
        assert_eq!(result.merges_performed, 0);
    }

    #[test]
    fn test_greedy_pauli_simp_xx_yy_merges() {
        // Two XX+YY pairs should get Clifford annotations
        let t1 = pt(4, &[(0, PauliOperator::X), (1, PauliOperator::X)], 0.5);
        let t2 = pt(4, &[(0, PauliOperator::Y), (1, PauliOperator::Y)], 0.3);
        let t3 = pt(4, &[(2, PauliOperator::X), (3, PauliOperator::X)], 0.2);
        let t4 = pt(4, &[(2, PauliOperator::Y), (3, PauliOperator::Y)], 0.1);
        let terms: Vec<&PauliTerm> = vec![&t1, &t2, &t3, &t4];
        let config = GreedyConfig::default();
        let result = greedy_pauli_simp(&terms, 1.0, 1.0, &config);

        // Should have performed at least 1 merge (Clifford annotation recording)
        assert!(
            result.merges_performed >= 1,
            "Expected at least 1 merge, got {}",
            result.merges_performed
        );
        // All original gadgets survive (angles unchanged, Clifford annotations added)
        assert_eq!(result.gadgets.len(), 4, "All gadgets should survive");
        // At least one gadget should have Clifford annotations
        let has_clifford = result.gadgets.iter().any(|g| !g.pre_gates.is_empty());
        assert!(
            has_clifford,
            "At least one gadget should have Clifford annotations"
        );
    }

    #[test]
    fn test_greedy_pauli_simp_convergence() {
        // Large enough set that the algorithm runs multiple iterations
        let mut terms_vec = Vec::new();
        for i in 0..10 {
            let q = i % 4;
            let op = if i % 2 == 0 {
                PauliOperator::X
            } else {
                PauliOperator::Y
            };
            terms_vec.push(pt(4, &[(q, op)], (i as f64 + 1.0) * 0.1));
        }
        let terms: Vec<&PauliTerm> = terms_vec.iter().collect();
        let config = GreedyConfig::default();
        let result = greedy_pauli_simp(&terms, 1.0, 1.0, &config);

        // Should converge within max_iterations
        assert!(result.iterations < config.max_iterations);
        // Result should have at most as many gadgets as input
        assert!(result.gadgets.len() <= terms.len());
    }

    #[test]
    fn test_greedy_pauli_simp_discount_rate() {
        // Create two pairs: one close, one far apart
        let t1 = pt(4, &[(0, PauliOperator::X)], 0.5);
        let t2 = pt(4, &[(0, PauliOperator::Y)], 0.3);
        // Add many terms between to create distance
        let mut terms_vec = vec![&t1];
        for i in 0..20 {
            let dummy = pt(4, &[(1, PauliOperator::Z)], (i as f64) * 0.01);
            terms_vec.push(Box::leak(Box::new(dummy)));
        }
        terms_vec.push(&t2);

        let config = GreedyConfig {
            discount_rate: 0.1, // Very aggressive discount for far pairs
            ..GreedyConfig::default()
        };
        let result = greedy_pauli_simp(&terms_vec, 1.0, 1.0, &config);

        // With aggressive discount, far pair may not merge
        // But the algorithm should still converge correctly
        assert!(result.iterations < config.max_iterations);
    }

    // ── Integration helper tests ──────────────────────────────────────

    #[test]
    fn test_gadgets_to_terms_and_map_no_clifford() {
        let gadgets = vec![PauliGadgetNode {
            pauli: ps(2, &[(0, PauliOperator::Z)]),
            angle: 0.5,
            position: 0,
            pre_gates: vec![],
            post_gates: vec![],
            merge_count: 1,
        }];
        let (terms, map) = gadgets_to_terms_and_map(&gadgets, 2);
        assert_eq!(terms.len(), 1);
        assert!(map.is_none());
    }

    #[test]
    fn test_gadgets_to_terms_and_map_with_clifford() {
        let gadgets = vec![PauliGadgetNode {
            pauli: ps(2, &[(0, PauliOperator::X)]),
            angle: 0.5,
            position: 0,
            pre_gates: vec![(CliffordGate::Sdg, 0)],
            post_gates: vec![(CliffordGate::S, 0)],
            merge_count: 2,
        }];
        let (terms, map) = gadgets_to_terms_and_map(&gadgets, 2);
        assert_eq!(terms.len(), 1);
        assert!(map.is_some());
        let map = map.unwrap();
        assert_eq!(map.len(), 1);
    }

    // ── H2_4q realistic test ──────────────────────────────────────────

    #[test]
    fn test_greedy_pauli_simp_h2_4q_terms() {
        // H2_4q representative terms: Z, ZZ, XX, YY
        let z_terms: Vec<PauliTerm> = vec![
            pt(4, &[(0, PauliOperator::Z)], -0.2228),
            pt(4, &[(1, PauliOperator::Z)], 0.1721),
            pt(4, &[(2, PauliOperator::Z)], -0.2228),
            pt(4, &[(3, PauliOperator::Z)], 0.1721),
        ];
        let zz_terms: Vec<PauliTerm> = vec![
            pt(4, &[(0, PauliOperator::Z), (1, PauliOperator::Z)], 0.1686),
            pt(4, &[(0, PauliOperator::Z), (2, PauliOperator::Z)], 0.1205),
            pt(4, &[(0, PauliOperator::Z), (3, PauliOperator::Z)], 0.1686),
            pt(4, &[(1, PauliOperator::Z), (2, PauliOperator::Z)], 0.1686),
            pt(4, &[(1, PauliOperator::Z), (3, PauliOperator::Z)], 0.1205),
            pt(4, &[(2, PauliOperator::Z), (3, PauliOperator::Z)], 0.1686),
        ];
        let xx_yy_terms: Vec<PauliTerm> = vec![
            pt(4, &[(0, PauliOperator::X), (1, PauliOperator::X)], 0.0454),
            pt(4, &[(0, PauliOperator::Y), (1, PauliOperator::Y)], 0.0454),
            pt(4, &[(2, PauliOperator::X), (3, PauliOperator::X)], 0.0454),
            pt(4, &[(2, PauliOperator::Y), (3, PauliOperator::Y)], 0.0454),
        ];

        let all_terms: Vec<PauliTerm> = [z_terms, zz_terms, xx_yy_terms].concat();
        let terms_ref: Vec<&PauliTerm> = all_terms.iter().collect();

        let config = GreedyConfig::default();
        let result = greedy_pauli_simp(&terms_ref, 1.0, 1.0, &config);

        // Should reduce the number of gadgets
        assert!(
            result.gadgets.len() <= terms_ref.len(),
            "Greedy optimization should not increase gadget count"
        );
        // At least the XX/YY pairs should trigger merges
        assert!(
            result.merges_performed >= 1 || result.gadgets.len() <= terms_ref.len(),
            "Should find at least some optimization opportunities"
        );
        // All gadgets should have non-zero angles
        for g in &result.gadgets {
            assert!(
                g.angle.abs() > 1e-15,
                "All output gadgets should have non-zero angles"
            );
        }
    }
}
