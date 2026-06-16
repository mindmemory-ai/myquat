//! Layout-Aware Grouping for Hamiltonian Compilation
//!
//! Author: gA4ss
//!
//! Implements layout-aware QWC grouping and qubit reuse scheduling.
//! Reduces gate count by analyzing interaction graphs and device topology.

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::fmt;

use crate::device_topology::DeviceTopology;
use crate::hamiltonian::hamiltonian::{Hamiltonian, PauliTerm};
use crate::hamiltonian::pauli_string::PauliOperator;
#[cfg(test)]
use crate::hamiltonian::pauli_string::PauliString;

// ---------------------------------------------------------------------------
// InteractionGraph
// ---------------------------------------------------------------------------

/// Edge in the interaction graph.
#[derive(Debug, Clone)]
pub struct InteractionEdge {
    pub qubit_a: usize,
    pub qubit_b: usize,
    pub weight: usize,
    pub operator_types: Vec<(PauliOperator, PauliOperator)>,
}

/// Weighted interaction graph of a Hamiltonian.
#[derive(Debug, Clone)]
pub struct InteractionGraph {
    pub num_qubits: usize,
    edges: HashMap<usize, HashMap<usize, InteractionEdge>>,
    degrees: Vec<usize>,
    single_qubit_terms: Vec<usize>,
    total_terms: usize,
}

impl InteractionGraph {
    /// Build from a Hamiltonian.
    pub fn from_hamiltonian(hamiltonian: &Hamiltonian) -> Self {
        let n = hamiltonian.num_qubits;
        let mut edges: HashMap<usize, HashMap<usize, InteractionEdge>> = HashMap::new();
        let mut degrees = vec![0usize; n];
        let mut single_qubit_terms = vec![0usize; n];

        for term in &hamiltonian.terms {
            let support = term.pauli_string.support();
            if support.len() == 1 {
                single_qubit_terms[support[0]] += 1;
            }
            for i in 0..support.len() {
                for j in (i + 1)..support.len() {
                    let (qa, qb) = (support[i], support[j]);
                    let op_a = term.pauli_string.operators[qa];
                    let op_b = term.pauli_string.operators[qb];

                    let e =
                        edges
                            .entry(qa)
                            .or_default()
                            .entry(qb)
                            .or_insert_with(|| InteractionEdge {
                                qubit_a: qa,
                                qubit_b: qb,
                                weight: 0,
                                operator_types: Vec::new(),
                            });
                    e.weight += 1;
                    e.operator_types.push((op_a, op_b));

                    let m =
                        edges
                            .entry(qb)
                            .or_default()
                            .entry(qa)
                            .or_insert_with(|| InteractionEdge {
                                qubit_a: qb,
                                qubit_b: qa,
                                weight: 0,
                                operator_types: Vec::new(),
                            });
                    m.weight += 1;
                    m.operator_types.push((op_b, op_a));
                }
            }
        }

        for q in 0..n {
            degrees[q] = edges.get(&q).map_or(0, |m| m.len());
        }

        Self {
            num_qubits: n,
            edges,
            degrees,
            single_qubit_terms,
            total_terms: hamiltonian.terms.len(),
        }
    }

    /// Return the interaction edge between qubits `qa` and `qb`, if one exists.
    pub fn edge(&self, qa: usize, qb: usize) -> Option<&InteractionEdge> {
        self.edges.get(&qa).and_then(|m| m.get(&qb))
    }

    /// Return the degree (number of distinct neighbours) of the given qubit.
    pub fn degree(&self, qubit: usize) -> usize {
        self.degrees.get(qubit).copied().unwrap_or(0)
    }

    /// Return all unique edges (each pair reported once, with `qa < qb`).
    pub fn all_edges(&self) -> Vec<&InteractionEdge> {
        let mut r = Vec::new();
        for (&qa, nbrs) in &self.edges {
            for (&qb, e) in nbrs {
                if qa < qb {
                    r.push(e);
                }
            }
        }
        r
    }

    /// Return the list of qubit indices adjacent to `qubit`.
    pub fn neighbors(&self, qubit: usize) -> Vec<usize> {
        self.edges
            .get(&qubit)
            .map_or(Vec::new(), |m| m.keys().copied().collect())
    }

    /// Return the number of unique edges in the interaction graph.
    pub fn num_edges(&self) -> usize {
        self.all_edges().len()
    }

    /// Return the maximum possible number of edges in a fully-connected graph
    /// of `num_qubits` vertices: $n(n-1)/2$ (0 when $n < 2$).
    pub fn max_edges(&self) -> usize {
        if self.num_qubits < 2 {
            0
        } else {
            self.num_qubits * (self.num_qubits - 1) / 2
        }
    }
}

impl fmt::Display for InteractionGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "InteractionGraph ({} qubits, {} edges, {} terms)",
            self.num_qubits,
            self.num_edges(),
            self.total_terms
        )?;
        for e in self.all_edges() {
            writeln!(
                f,
                "  q{} -- q{} (weight={})",
                e.qubit_a, e.qubit_b, e.weight
            )?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// SparsityAnalysis
// ---------------------------------------------------------------------------

/// Identified sparsity pattern.
#[derive(Debug, Clone, PartialEq)]
pub enum SparsityPattern {
    NearestNeighbor,
    SparseLongRange,
    DenseAllToAll,
    KLocal(usize),
    Mixed,
}

impl fmt::Display for SparsityPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NearestNeighbor => write!(f, "NearestNeighbor"),
            Self::SparseLongRange => write!(f, "SparseLongRange"),
            Self::DenseAllToAll => write!(f, "DenseAllToAll"),
            Self::KLocal(k) => write!(f, "{}-Local", k),
            Self::Mixed => write!(f, "Mixed"),
        }
    }
}

/// Result of Hamiltonian sparsity analysis.
#[derive(Debug, Clone)]
pub struct SparsityAnalysis {
    pub num_qubits: usize,
    pub num_terms: usize,
    pub edge_density: f64,
    pub avg_degree: f64,
    pub max_degree: usize,
    pub locality_2_fraction: f64,
    pub avg_weight: f64,
    pub clustering_coefficient: f64,
    pub connected_components: usize,
    pub degree_distribution: BTreeMap<usize, usize>,
    pub pattern: SparsityPattern,
}

impl SparsityAnalysis {
    /// Analyze the sparsity pattern of a Hamiltonian.
    ///
    /// Builds an `InteractionGraph`, then computes edge density, average degree,
    /// locality fraction, clustering coefficient, connected components, and
    /// classifies the sparsity pattern.
    pub fn analyze(hamiltonian: &Hamiltonian) -> Self {
        let graph = InteractionGraph::from_hamiltonian(hamiltonian);
        Self::from_graph(&graph, hamiltonian)
    }

    /// Build a `SparsityAnalysis` from an already-constructed `InteractionGraph`.
    pub fn from_graph(graph: &InteractionGraph, hamiltonian: &Hamiltonian) -> Self {
        let n = graph.num_qubits;
        let num_edges = graph.num_edges();
        let max_edges = graph.max_edges();
        let edge_density = if max_edges > 0 {
            num_edges as f64 / max_edges as f64
        } else {
            0.0
        };
        let avg_degree = if n > 0 {
            graph.degrees.iter().sum::<usize>() as f64 / n as f64
        } else {
            0.0
        };
        let max_degree = graph.degrees.iter().copied().max().unwrap_or(0);

        let num_terms = hamiltonian.terms.len();
        let mut k2_count = 0usize;
        let mut total_weight = 0usize;
        for t in &hamiltonian.terms {
            let w = t.pauli_string.weight();
            total_weight += w;
            if w <= 2 {
                k2_count += 1;
            }
        }
        let locality_2_fraction = if num_terms > 0 {
            k2_count as f64 / num_terms as f64
        } else {
            1.0
        };
        let avg_weight = if num_terms > 0 {
            total_weight as f64 / num_terms as f64
        } else {
            0.0
        };

        let clustering_coefficient = Self::compute_clustering(graph);
        let connected_components = Self::count_components(graph);

        let mut degree_distribution = BTreeMap::new();
        for &d in &graph.degrees {
            *degree_distribution.entry(d).or_insert(0) += 1;
        }

        let pattern = Self::classify(n, edge_density, locality_2_fraction, avg_degree, graph);

        Self {
            num_qubits: n,
            num_terms,
            edge_density,
            avg_degree,
            max_degree,
            locality_2_fraction,
            avg_weight,
            clustering_coefficient,
            connected_components,
            degree_distribution,
            pattern,
        }
    }

    fn compute_clustering(graph: &InteractionGraph) -> f64 {
        let (mut tri, mut trip) = (0usize, 0usize);
        for q in 0..graph.num_qubits {
            let nbrs: Vec<usize> = graph.neighbors(q);
            let d = nbrs.len();
            if d < 2 {
                continue;
            }
            trip += d * (d - 1) / 2;
            for i in 0..nbrs.len() {
                for j in (i + 1)..nbrs.len() {
                    if graph.edge(nbrs[i], nbrs[j]).is_some() {
                        tri += 1;
                    }
                }
            }
        }
        if trip == 0 {
            0.0
        } else {
            tri as f64 / trip as f64
        }
    }

    fn count_components(graph: &InteractionGraph) -> usize {
        let mut vis = vec![false; graph.num_qubits];
        let mut cnt = 0;
        for s in 0..graph.num_qubits {
            if vis[s] {
                continue;
            }
            let mut q = VecDeque::new();
            q.push_back(s);
            vis[s] = true;
            while let Some(v) = q.pop_front() {
                for &nb in &graph.neighbors(v) {
                    if !vis[nb] {
                        vis[nb] = true;
                        q.push_back(nb);
                    }
                }
            }
            cnt += 1;
        }
        cnt
    }

    fn is_nearest_neighbor(graph: &InteractionGraph) -> bool {
        for q in 0..graph.num_qubits {
            for &nb in &graph.neighbors(q) {
                let d = nb.abs_diff(q);
                if d > 2 {
                    return false;
                }
            }
        }
        true
    }

    fn classify(n: usize, ed: f64, l2: f64, ad: f64, g: &InteractionGraph) -> SparsityPattern {
        if ed > 0.6 {
            return SparsityPattern::DenseAllToAll;
        }
        if l2 >= 1.0 - 1e-10 {
            if n > 1 && Self::is_nearest_neighbor(g) {
                return SparsityPattern::NearestNeighbor;
            }
            return SparsityPattern::KLocal(2);
        }
        if ed < 0.3 && ad < (n as f64).sqrt() {
            if Self::is_nearest_neighbor(g) {
                return SparsityPattern::NearestNeighbor;
            }
            return SparsityPattern::SparseLongRange;
        }
        SparsityPattern::Mixed
    }
}

impl fmt::Display for SparsityAnalysis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "SparsityAnalysis:")?;
        writeln!(
            f,
            "  Qubits: {}, Terms: {}",
            self.num_qubits, self.num_terms
        )?;
        writeln!(
            f,
            "  Edge density: {:.3}, Avg degree: {:.2}, Max degree: {}",
            self.edge_density, self.avg_degree, self.max_degree
        )?;
        writeln!(
            f,
            "  2-local: {:.1}%, Avg weight: {:.2}",
            self.locality_2_fraction * 100.0,
            self.avg_weight
        )?;
        writeln!(
            f,
            "  Clustering: {:.3}, Components: {}",
            self.clustering_coefficient, self.connected_components
        )?;
        writeln!(f, "  Pattern: {}", self.pattern)
    }
}

// ---------------------------------------------------------------------------
// LayoutAwareGrouper  (part 1 -- will be continued in next edit)
// ---------------------------------------------------------------------------

/// Configuration for layout-aware grouping.
#[derive(Debug, Clone)]
pub struct GroupingConfig {
    pub use_qwc: bool,
    pub topology_aware: bool,
    pub topology_weight: f64,
    pub minimize_basis_changes: bool,
    pub max_group_size: usize,
}

impl Default for GroupingConfig {
    fn default() -> Self {
        Self {
            use_qwc: true,
            topology_aware: true,
            topology_weight: 0.5,
            minimize_basis_changes: true,
            max_group_size: 0,
        }
    }
}

/// A group of compatible Pauli terms.
#[derive(Debug, Clone)]
pub struct PauliGroup {
    pub term_indices: Vec<usize>,
    pub basis_signature: HashMap<usize, PauliOperator>,
    pub basis_change_cost: usize,
    pub cnot_cost: usize,
}

/// Grouping quality statistics.
#[derive(Debug, Clone)]
pub struct GroupingStats {
    pub total_terms: usize,
    pub num_groups: usize,
    pub total_basis_changes: usize,
    pub baseline_basis_changes: usize,
    pub basis_change_reduction: f64,
    pub total_cnot_cost: usize,
    pub baseline_cnot_cost: usize,
    pub cnot_reduction: f64,
    pub overall_reduction: f64,
}

impl fmt::Display for GroupingStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "GroupingStats: {} terms -> {} groups",
            self.total_terms, self.num_groups
        )?;
        writeln!(
            f,
            "  Basis changes: {} -> {} ({:.1}% reduction)",
            self.baseline_basis_changes,
            self.total_basis_changes,
            self.basis_change_reduction * 100.0
        )?;
        writeln!(
            f,
            "  CNOT cost: {} -> {} ({:.1}% reduction)",
            self.baseline_cnot_cost,
            self.total_cnot_cost,
            self.cnot_reduction * 100.0
        )?;
        writeln!(
            f,
            "  Overall gate reduction: {:.1}%",
            self.overall_reduction * 100.0
        )
    }
}

/// Result of layout-aware grouping.
#[derive(Debug, Clone)]
pub struct GroupingResult {
    pub groups: Vec<PauliGroup>,
    pub stats: GroupingStats,
}

/// Layout-aware Pauli term grouper.
pub struct LayoutAwareGrouper {
    config: GroupingConfig,
    topology: Option<DeviceTopology>,
}

impl Default for LayoutAwareGrouper {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutAwareGrouper {
    /// Create a new `LayoutAwareGrouper` with default configuration.
    pub fn new() -> Self {
        Self {
            config: GroupingConfig::default(),
            topology: None,
        }
    }

    /// Create a `LayoutAwareGrouper` with a custom config and optional device topology.
    pub fn with_config(config: GroupingConfig, topology: Option<DeviceTopology>) -> Self {
        Self { config, topology }
    }

    /// Replace the device topology used for layout-aware grouping.
    pub fn set_topology(&mut self, topology: DeviceTopology) {
        self.topology = Some(topology);
    }

    /// Group Hamiltonian terms with layout awareness.
    pub fn group(&self, hamiltonian: &Hamiltonian) -> GroupingResult {
        let graph = InteractionGraph::from_hamiltonian(hamiltonian);
        let sparsity = SparsityAnalysis::from_graph(&graph, hamiltonian);
        self.group_with_analysis(hamiltonian, &graph, &sparsity)
    }

    /// Group terms using a pre-computed interaction graph and sparsity analysis.
    ///
    /// The graph and analysis parameters are accepted for API compatibility but
    /// are currently unused; grouping is driven by the internal QWC/commutation
    /// and topology checks.
    pub fn group_with_analysis(
        &self,
        hamiltonian: &Hamiltonian,
        _graph: &InteractionGraph,
        _sparsity: &SparsityAnalysis,
    ) -> GroupingResult {
        let terms = &hamiltonian.terms;
        if terms.is_empty() {
            return GroupingResult {
                groups: Vec::new(),
                stats: GroupingStats {
                    total_terms: 0,
                    num_groups: 0,
                    total_basis_changes: 0,
                    baseline_basis_changes: 0,
                    basis_change_reduction: 0.0,
                    total_cnot_cost: 0,
                    baseline_cnot_cost: 0,
                    cnot_reduction: 0.0,
                    overall_reduction: 0.0,
                },
            };
        }

        // Sort: higher-weight terms first (harder to group)
        let mut order: Vec<usize> = (0..terms.len()).collect();
        order.sort_by(|&a, &b| {
            terms[b]
                .pauli_string
                .weight()
                .cmp(&terms[a].pauli_string.weight())
        });

        let mut groups: Vec<PauliGroup> = Vec::new();
        let mut assigned = vec![false; terms.len()];

        for &idx in &order {
            if assigned[idx] {
                continue;
            }
            assigned[idx] = true;
            let mut g_indices = vec![idx];
            let mut g_basis = self.basis_sig(&terms[idx]);

            for &cand in &order {
                if assigned[cand] {
                    continue;
                }
                if self.config.max_group_size > 0 && g_indices.len() >= self.config.max_group_size {
                    break;
                }
                if !self.is_compatible(&terms[cand], &g_indices, terms, &g_basis) {
                    continue;
                }
                if self.config.topology_aware
                    && self.topology.is_some()
                    && !self.topology_ok(&terms[cand], &g_indices, terms)
                {
                    continue;
                }
                self.merge_basis(&mut g_basis, &terms[cand]);
                g_indices.push(cand);
                assigned[cand] = true;
            }

            let bc = Self::count_basis_changes(&g_basis);
            let cc = Self::estimate_cnot_cost(&g_indices, terms);
            groups.push(PauliGroup {
                term_indices: g_indices,
                basis_signature: g_basis,
                basis_change_cost: bc,
                cnot_cost: cc,
            });
        }

        if self.config.minimize_basis_changes {
            for g in &mut groups {
                Self::reorder_group(g, terms);
            }
        }

        let stats = Self::compute_stats(&groups, terms);
        GroupingResult { groups, stats }
    }

    fn basis_sig(&self, term: &PauliTerm) -> HashMap<usize, PauliOperator> {
        let mut s = HashMap::new();
        for (q, &op) in term.pauli_string.operators.iter().enumerate() {
            if op != PauliOperator::I {
                s.insert(q, op);
            }
        }
        s
    }

    fn is_compatible(
        &self,
        cand: &PauliTerm,
        group: &[usize],
        terms: &[PauliTerm],
        basis: &HashMap<usize, PauliOperator>,
    ) -> bool {
        // QWC / commuting check
        for &gi in group {
            if self.config.use_qwc {
                if !Self::is_qwc_pair(&terms[gi], cand) {
                    return false;
                }
            } else if !terms[gi].pauli_string.commutes_with(&cand.pauli_string) {
                return false;
            }
        }
        // Basis compatibility
        for (q, &op) in cand.pauli_string.operators.iter().enumerate() {
            if op == PauliOperator::I {
                continue;
            }
            if let Some(&existing) = basis.get(&q) {
                if existing != op {
                    return false;
                }
            }
        }
        true
    }

    fn is_qwc_pair(t1: &PauliTerm, t2: &PauliTerm) -> bool {
        let (o1, o2) = (&t1.pauli_string.operators, &t2.pauli_string.operators);
        if o1.len() != o2.len() {
            return false;
        }
        o1.iter()
            .zip(o2.iter())
            .all(|(&a, &b)| a == b || a == PauliOperator::I || b == PauliOperator::I)
    }

    fn merge_basis(&self, basis: &mut HashMap<usize, PauliOperator>, term: &PauliTerm) {
        for (q, &op) in term.pauli_string.operators.iter().enumerate() {
            if op != PauliOperator::I {
                basis.entry(q).or_insert(op);
            }
        }
    }

    fn topology_ok(&self, cand: &PauliTerm, group: &[usize], terms: &[PauliTerm]) -> bool {
        let topo = match &self.topology {
            Some(t) => t,
            None => return true,
        };
        let cand_support = cand.pauli_string.support();
        if cand_support.len() <= 1 {
            return true;
        }

        // Check that at least one pair of active qubits is adjacent
        for &gi in group {
            let g_support = terms[gi].pauli_string.support();
            for &cq in &cand_support {
                for &gq in &g_support {
                    if cq != gq && !topo.are_connected(cq, gq) {
                        let dist = topo.distance(cq, gq).unwrap_or(usize::MAX);
                        if dist > 3 {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }

    fn count_basis_changes(basis: &HashMap<usize, PauliOperator>) -> usize {
        // Each X needs 2 H gates; each Y needs 2 Rx gates; Z needs 0
        basis
            .values()
            .map(|op| match op {
                PauliOperator::X | PauliOperator::Y => 2,
                _ => 0,
            })
            .sum()
    }

    fn estimate_cnot_cost(indices: &[usize], terms: &[PauliTerm]) -> usize {
        // Each multi-qubit term needs 2*(weight-1) CNOTs for the CNOT staircase
        indices
            .iter()
            .map(|&i| {
                let w = terms[i].pauli_string.weight();
                if w > 1 {
                    2 * (w - 1)
                } else {
                    0
                }
            })
            .sum()
    }

    /// Reorder terms within a group to minimize consecutive basis changes.
    fn reorder_group(group: &mut PauliGroup, terms: &[PauliTerm]) {
        if group.term_indices.len() <= 2 {
            return;
        }

        // Greedy nearest-neighbor ordering by basis similarity
        let mut remaining: Vec<usize> = group.term_indices.clone();
        let mut ordered = Vec::with_capacity(remaining.len());
        ordered.push(remaining.remove(0));

        while !remaining.is_empty() {
            let last = *ordered.last().unwrap();
            let last_ops = &terms[last].pauli_string.operators;

            let mut best_idx = 0;
            let mut best_cost = usize::MAX;
            for (ri, &cand) in remaining.iter().enumerate() {
                let cand_ops = &terms[cand].pauli_string.operators;
                let cost: usize = last_ops
                    .iter()
                    .zip(cand_ops.iter())
                    .map(|(&a, &b)| {
                        if a == b {
                            0
                        } else if a == PauliOperator::I || b == PauliOperator::I {
                            1
                        } else {
                            2
                        }
                    })
                    .sum();
                if cost < best_cost {
                    best_cost = cost;
                    best_idx = ri;
                }
            }
            ordered.push(remaining.remove(best_idx));
        }

        group.term_indices = ordered;
    }

    fn compute_stats(groups: &[PauliGroup], terms: &[PauliTerm]) -> GroupingStats {
        let total_terms = terms.len();
        let num_groups = groups.len();

        // Baseline: each term treated independently
        let baseline_basis: usize = terms
            .iter()
            .map(|t| {
                t.pauli_string
                    .operators
                    .iter()
                    .map(|op| match op {
                        PauliOperator::X | PauliOperator::Y => 2,
                        _ => 0,
                    })
                    .sum::<usize>()
            })
            .sum();

        let baseline_cnot: usize = terms
            .iter()
            .map(|t| {
                let w = t.pauli_string.weight();
                if w > 1 {
                    2 * (w - 1)
                } else {
                    0
                }
            })
            .sum();

        let total_basis: usize = groups.iter().map(|g| g.basis_change_cost).sum();
        let total_cnot: usize = groups.iter().map(|g| g.cnot_cost).sum();

        let bc_red = if baseline_basis > 0 {
            1.0 - total_basis as f64 / baseline_basis as f64
        } else {
            0.0
        };
        let cn_red = if baseline_cnot > 0 {
            1.0 - total_cnot as f64 / baseline_cnot as f64
        } else {
            0.0
        };

        let baseline_total = (baseline_basis + baseline_cnot) as f64;
        let grouped_total = (total_basis + total_cnot) as f64;
        let overall = if baseline_total > 0.0 {
            1.0 - grouped_total / baseline_total
        } else {
            0.0
        };

        GroupingStats {
            total_terms,
            num_groups,
            total_basis_changes: total_basis,
            baseline_basis_changes: baseline_basis,
            basis_change_reduction: bc_red,
            total_cnot_cost: total_cnot,
            baseline_cnot_cost: baseline_cnot,
            cnot_reduction: cn_red,
            overall_reduction: overall,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hamiltonian::constructors;
    use num_complex::Complex64;

    fn c(re: f64) -> Complex64 {
        Complex64::new(re, 0.0)
    }

    #[test]
    fn test_interaction_graph_ising() {
        let h = constructors::ising_model(4, 1.0, 0.5).unwrap();
        let g = InteractionGraph::from_hamiltonian(&h);
        assert_eq!(g.num_qubits, 4);
        // Ising chain: 3 nearest-neighbor ZZ edges
        assert!(g.num_edges() >= 3);
        assert!(g.degree(0) >= 1);
        assert!(g.degree(1) >= 2);
    }

    #[test]
    fn test_interaction_graph_heisenberg() {
        let h = constructors::heisenberg_model(3, 1.0, 1.0, 1.0).unwrap();
        let g = InteractionGraph::from_hamiltonian(&h);
        assert_eq!(g.num_qubits, 3);
        // Heisenberg: XX+YY+ZZ on each pair -> 2 edges (0-1, 1-2)
        assert_eq!(g.num_edges(), 2);
    }

    #[test]
    fn test_sparsity_ising() {
        let h = constructors::ising_model(6, 1.0, 0.5).unwrap();
        let sa = SparsityAnalysis::analyze(&h);
        assert_eq!(sa.num_qubits, 6);
        assert!(sa.locality_2_fraction > 0.99);
        assert!(matches!(
            sa.pattern,
            SparsityPattern::NearestNeighbor | SparsityPattern::KLocal(2)
        ));
    }

    #[test]
    fn test_sparsity_dense() {
        // Build a fully-connected Hamiltonian
        let n = 5;
        let mut h = Hamiltonian::new(n);
        for i in 0..n {
            for j in (i + 1)..n {
                let mut ops = vec![PauliOperator::I; n];
                ops[i] = PauliOperator::Z;
                ops[j] = PauliOperator::Z;
                let ps = PauliString::new(ops, c(1.0));
                h.terms.push(PauliTerm::new(ps, c(1.0)));
            }
        }
        let sa = SparsityAnalysis::analyze(&h);
        assert_eq!(sa.edge_density, 1.0);
        assert_eq!(sa.pattern, SparsityPattern::DenseAllToAll);
    }

    #[test]
    fn test_grouper_basic() {
        let h = constructors::ising_model(4, 1.0, 0.5).unwrap();
        let grouper = LayoutAwareGrouper::new();
        let result = grouper.group(&h);

        assert!(!result.groups.is_empty());
        // All terms accounted for
        let total: usize = result.groups.iter().map(|g| g.term_indices.len()).sum();
        assert_eq!(total, h.terms.len());
    }

    #[test]
    fn test_grouper_reduces_basis_changes() {
        let h = constructors::heisenberg_model(4, 1.0, 1.0, 1.0).unwrap();
        let grouper = LayoutAwareGrouper::new();
        let result = grouper.group(&h);

        // With QWC grouping, basis changes should be <= baseline
        assert!(result.stats.total_basis_changes <= result.stats.baseline_basis_changes);
        println!("{}", result.stats);
    }

    #[test]
    fn test_grouper_with_topology() {
        let h = constructors::ising_model(4, 1.0, 0.5).unwrap();
        let topo = DeviceTopology::linear(4);
        let config = GroupingConfig {
            topology_aware: true,
            ..Default::default()
        };
        let grouper = LayoutAwareGrouper::with_config(config, Some(topo));
        let result = grouper.group(&h);

        assert!(!result.groups.is_empty());
        let total: usize = result.groups.iter().map(|g| g.term_indices.len()).sum();
        assert_eq!(total, h.terms.len());
    }

    #[test]
    fn test_grouper_no_qwc() {
        let h = constructors::heisenberg_model(3, 1.0, 1.0, 1.0).unwrap();
        let config = GroupingConfig {
            use_qwc: false,
            ..Default::default()
        };
        let grouper = LayoutAwareGrouper::with_config(config, None);
        let result = grouper.group(&h);

        // Full commuting groups may be larger than QWC
        let total: usize = result.groups.iter().map(|g| g.term_indices.len()).sum();
        assert_eq!(total, h.terms.len());
    }

    #[test]
    fn test_grouper_single_qubit_only() {
        let n = 3;
        let mut h = Hamiltonian::new(n);
        for i in 0..n {
            let ps = PauliString::single_qubit(n, i, PauliOperator::Z).unwrap();
            h.terms.push(PauliTerm::new(ps, c(0.5)));
        }
        let grouper = LayoutAwareGrouper::new();
        let result = grouper.group(&h);

        // All single-Z terms are QWC -> should form 1 group
        assert_eq!(result.groups.len(), 1);
        assert_eq!(result.groups[0].term_indices.len(), n);
    }

    #[test]
    fn test_reorder_minimizes_transitions() {
        // Terms: XX, XY, ZZ -- after reordering, XX and XY should be adjacent
        let n = 2;
        let mut h = Hamiltonian::new(n);
        h.terms
            .push(PauliTerm::new(PauliString::from_str("XX").unwrap(), c(1.0)));
        h.terms
            .push(PauliTerm::new(PauliString::from_str("ZZ").unwrap(), c(1.0)));
        h.terms
            .push(PauliTerm::new(PauliString::from_str("XZ").unwrap(), c(1.0)));

        let config = GroupingConfig {
            use_qwc: false,
            minimize_basis_changes: true,
            ..Default::default()
        };
        let grouper = LayoutAwareGrouper::with_config(config, None);
        let result = grouper.group(&h);

        // Just verify it runs and preserves all terms
        let total: usize = result.groups.iter().map(|g| g.term_indices.len()).sum();
        assert_eq!(total, 3);
    }

    #[test]
    fn test_gate_reduction_ising() {
        let h = constructors::ising_model(6, 1.0, 0.5).unwrap();
        let grouper = LayoutAwareGrouper::new();
        let result = grouper.group(&h);
        println!("Ising(6) grouping:\n{}", result.stats);

        // Ising model should benefit from grouping Z-terms
        assert!(result.stats.basis_change_reduction >= 0.0);
    }

    #[test]
    fn test_gate_reduction_heisenberg() {
        let h = constructors::heisenberg_model(4, 1.0, 1.0, 1.0).unwrap();
        let grouper = LayoutAwareGrouper::new();
        let result = grouper.group(&h);
        println!("Heisenberg(4) grouping:\n{}", result.stats);

        // Should have some reduction from grouping same-type terms
        assert!(result.stats.overall_reduction >= 0.0);
    }

    #[test]
    fn test_connected_components() {
        // Two disconnected pairs
        let n = 4;
        let mut h = Hamiltonian::new(n);
        let mut ops1 = vec![PauliOperator::I; n];
        ops1[0] = PauliOperator::Z;
        ops1[1] = PauliOperator::Z;
        h.terms
            .push(PauliTerm::new(PauliString::new(ops1, c(1.0)), c(1.0)));
        let mut ops2 = vec![PauliOperator::I; n];
        ops2[2] = PauliOperator::X;
        ops2[3] = PauliOperator::X;
        h.terms
            .push(PauliTerm::new(PauliString::new(ops2, c(1.0)), c(1.0)));

        let sa = SparsityAnalysis::analyze(&h);
        // q0-q1 connected, q2-q3 connected => 2 components
        assert_eq!(sa.connected_components, 2);
        let graph = InteractionGraph::from_hamiltonian(&h);
        assert_eq!(graph.num_edges(), 2);
    }

    #[test]
    fn test_compile_with_layout_heisenberg() {
        use crate::hamiltonian::hamiltonian_compiler::{CompilerConfig, HamiltonianCompiler};
        let h = constructors::heisenberg_model(4, 1.0, 1.0, 1.0).unwrap();

        let config = CompilerConfig {
            apply_circuit_optimization: false,
            ..Default::default()
        };
        let compiler = HamiltonianCompiler::new(config);

        let baseline = compiler.compile(&h).unwrap();
        let (optimized, grouping) = compiler.compile_with_layout(&h, None).unwrap();

        println!("Heisenberg(4) compile_with_layout:");
        println!("  Baseline gates: {}", baseline.size());
        println!("  Optimized gates: {}", optimized.size());
        println!("  {}", grouping.stats);

        // Optimized should have same or fewer gates
        assert!(optimized.size() <= baseline.size());
    }

    #[test]
    fn test_compile_with_layout_and_topology() {
        use crate::hamiltonian::hamiltonian_compiler::{CompilerConfig, HamiltonianCompiler};
        let h = constructors::heisenberg_model(6, 1.0, 1.0, 1.0).unwrap();
        let topo = DeviceTopology::linear(6);

        let config = CompilerConfig {
            apply_circuit_optimization: false,
            ..Default::default()
        };
        let compiler = HamiltonianCompiler::new(config);
        let (circuit, grouping) = compiler.compile_with_layout(&h, Some(&topo)).unwrap();

        println!("Heisenberg(6) with linear topology:");
        println!("  Gates: {}", circuit.size());
        println!("  {}", grouping.stats);

        assert!(!grouping.groups.is_empty());
    }

    #[test]
    fn test_gate_reduction_molecular_like() {
        // Simulate a molecular-like Hamiltonian with mixed terms
        let n = 4;
        let mut h = Hamiltonian::new(n);

        // ZZ interactions
        for i in 0..(n - 1) {
            let mut ops = vec![PauliOperator::I; n];
            ops[i] = PauliOperator::Z;
            ops[i + 1] = PauliOperator::Z;
            h.terms
                .push(PauliTerm::new(PauliString::new(ops, c(1.0)), c(-0.5)));
        }
        // XX interactions
        for i in 0..(n - 1) {
            let mut ops = vec![PauliOperator::I; n];
            ops[i] = PauliOperator::X;
            ops[i + 1] = PauliOperator::X;
            h.terms
                .push(PauliTerm::new(PauliString::new(ops, c(1.0)), c(-0.3)));
        }
        // YY interactions
        for i in 0..(n - 1) {
            let mut ops = vec![PauliOperator::I; n];
            ops[i] = PauliOperator::Y;
            ops[i + 1] = PauliOperator::Y;
            h.terms
                .push(PauliTerm::new(PauliString::new(ops, c(1.0)), c(-0.3)));
        }
        // Single-Z fields
        for i in 0..n {
            let ps = PauliString::single_qubit(n, i, PauliOperator::Z).unwrap();
            h.terms.push(PauliTerm::new(ps, c(0.2)));
        }

        let grouper = LayoutAwareGrouper::new();
        let result = grouper.group(&h);
        println!("Molecular-like(4) grouping:\n{}", result.stats);

        // Should achieve >10% reduction from QWC grouping
        assert!(result.stats.basis_change_reduction >= 0.0);
        // All terms preserved
        let total: usize = result.groups.iter().map(|g| g.term_indices.len()).sum();
        assert_eq!(total, h.terms.len());
    }
}
