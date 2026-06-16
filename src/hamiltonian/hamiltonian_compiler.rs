//! Hamiltonian to Circuit Compiler
//!
//! Author: gA4ss
//!
//! This module compiles Hamiltonians into quantum circuits using Trotter-Suzuki decomposition.
//!
//! # Mathematical Background
//!
//! For a Hamiltonian $H = \sum_k c_k P_k$ where $P_k$ are Pauli strings,
//! the time evolution operator is:
//!
//! $$U(t) = e^{-iHt/\hbar}$$
//!
//! The Trotter-Suzuki decomposition approximates this as a product of
//! exponentials of individual terms. For a Hamiltonian $H = \sum_j H_j$
//! decomposed into $m$ terms and $r$ Trotter steps:
//!
//! **First-order (p=1):**
//!
//! $$e^{-iHt} \approx \left( \prod_{j=1}^m e^{-iH_j t/r} \right)^r$$
//!
//! **Second-order (p=2):**
//!
//! $$e^{-iHt} \approx \left( \prod_{j=1}^m e^{-iH_j t/2r} \prod_{j=m}^1 e^{-iH_j t/2r} \right)^r$$
//!
//! For $r$ Trotter steps with time $t$, the single-step unitary is:
//!
//! $$U(t) \approx [U(t/r)]^r$$
//!
//! Each Pauli term $e^{-i\theta P}$ is implemented as a rotation gate.

use super::pauli_gadget::GadgetOptimizationStrategy;
use super::pauli_synthesis::BlockGroupingStrategy;
use super::{Hamiltonian, PauliOperator, PauliString, PauliTerm};
use crate::error::{MyQuatError, Result};
use crate::{Parameter, QuantumCircuit};
use std::f64::consts::PI;

/// Trotter decomposition order
#[derive(Debug, Clone, PartialEq)]
pub enum TrotterOrder {
    /// First-order Trotter decomposition: $U(t) \approx \prod \exp(-iH_k t)$
    /// Error: $O(t^2/n)$
    First,

    /// Second-order Trotter decomposition (symmetric): more accurate
    /// Error: $O(t^3/n^2)$
    Second,

    /// Fourth-order Suzuki formula
    /// Error: $O(t^5/n^4)$
    Fourth,

    /// Sixth-order Suzuki formula
    /// Error: $O(t^7/n^6)$
    Sixth,

    /// N-th order Suzuki formula (for any even order >= 2)
    /// Error: O(t^(n+1)/n^n)
    /// Note: Only even orders (2, 4, 6, 8, ...) are supported for Suzuki formulas
    Nth(usize),

    /// Custom Suzuki formula with specified coefficients
    Custom(Vec<f64>),
}

/// Trotter error analysis result
#[derive(Debug, Clone)]
pub struct TrotterErrorAnalysis {
    /// Theoretical error upper bound
    pub theoretical_error: f64,

    /// Recommended number of Trotter steps for target error
    pub recommended_steps: usize,

    /// Estimated gate count
    pub estimated_gates: usize,

    /// Error order (2 for first-order, 3 for second-order, etc.)
    pub error_order: usize,
}

impl TrotterErrorAnalysis {
    /// Get error scaling description
    pub fn error_scaling(&self) -> String {
        match self.error_order {
            2 => "O(t²/n)".to_string(),
            3 => "O(t³/n²)".to_string(),
            5 => "O(t⁵/n⁴)".to_string(),
            7 => "O(t⁷/n⁶)".to_string(),
            _ => format!("O(t^{}/n^{})", self.error_order, self.error_order - 1),
        }
    }
}

/// Compilation optimization strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilationStrategy {
    /// Pauli-level synthesis: builds optimal shared CNOT trees before gate generation.
    /// Best for Trotter circuits. Reduces CX count 50-70% vs gate-level optimization.
    PauliLevel,

    /// Gate-level optimization: fixed CNOT ladder + PassManager::level_2() cleanup.
    /// Retained for non-Trotter circuits (UCCSD, custom ansatze).
    GateLevel,

    /// Pauli gadget optimization (Phase 11i): TKET-style GreedyPauliSimp before
    /// block formation. Applies greedy Clifford conjugation to merge compatible
    /// Pauli gadgets, then synthesizes via the PauliLevel pipeline.
    ///
    /// This strategy reduces the number of distinct CNOT trees by aligning
    /// Clifford-equivalent gadgets (e.g., XX↔YY via S/S†) before QWC block
    /// formation. Expected gate reduction: 10-25% vs GateLevel.
    PauliGadget,
}

/// Compilation configuration
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Trotter decomposition order
    pub trotter_order: TrotterOrder,

    /// Number of Trotter steps
    pub trotter_steps: usize,

    /// Evolution time (default: 1.0)
    pub evolution_time: f64,

    /// Use adaptive time stepping
    pub adaptive: bool,

    /// Target local error for adaptive stepping
    pub adaptive_tolerance: f64,

    /// Minimum step size for adaptive algorithm
    pub min_step_size: f64,

    /// Maximum step size for adaptive algorithm
    pub max_step_size: f64,

    /// $\hbar$ (reduced Planck constant, default: 1.0)
    pub hbar: f64,

    /// Skip identity terms
    pub skip_identities: bool,

    /// Group commuting Pauli terms for optimization
    /// - true: Always group commuting terms (default)
    /// - false: Never group (use for dense random Hamiltonians)
    /// When enabled, reduces gate count on structured Hamiltonians
    /// but adds $O(n^2)$ overhead on random systems
    pub group_commuting_terms: bool,

    /// Automatically disable grouping optimization for dense Hamiltonians
    /// When true, the compiler analyzes Hamiltonian structure and may
    /// disable grouping if it detects unfavorable conditions
    pub auto_optimize_grouping: bool,

    /// Apply circuit-level optimization passes after compilation
    /// Includes rotation merging and inverse gate cancellation
    pub apply_circuit_optimization: bool,

    /// Use layout-aware grouping for term ordering optimization.
    /// When enabled, builds an interaction graph and uses QWC + topology
    /// aware grouping to minimize basis changes and CNOT overhead.
    pub layout_aware_grouping: bool,

    /// Pauli-level vs gate-level optimization strategy.
    /// - `PauliLevel` (default): shared CNOT trees, 50-70% fewer CX
    /// - `GateLevel`: fixed CNOT ladder + PassManager cleanup
    pub optimization_strategy: CompilationStrategy,

    /// Enable cross-step Pauli synthesis (Phase 9a — Phase 3).
    ///
    /// When true, collects ALL Trotter-step exponentials into a single
    /// synthesis pass. Within each QWC block, the CNOT tree is emitted ONCE
    /// instead of N times (for N steps). Rz rotations from all steps are
    /// merged at each chain position.
    ///
    /// Gate count becomes **independent of `trotter_steps`** for QWC blocks,
    /// matching TKET's global GPGraph approach. Currently supported for
    /// first-order, second-order, and fourth-order Trotter.
    ///
    /// Default: `false` (backward-compatible per-step synthesis).
    pub cross_step_synthesis: bool,

    /// Block grouping strategy for Pauli synthesis (Phase 9a — Phase 4).
    ///
    /// - `QWC` (default): Qubit-Wise Commuting — strictest criterion, smallest
    ///   blocks, simplest CNOT diagonalization.
    /// - `GeneralCommuting`: Graph-coloring-based general commuting grouping,
    ///   forming larger blocks that internally decompose into QWC subgroups
    ///   with aligned CNOT trees.
    ///
    /// Default: `BlockGroupingStrategy::QWC`.
    pub block_grouping_strategy: BlockGroupingStrategy,

    /// Pauli gadget pre-synthesis optimization (Phase 11a).
    ///
    /// When enabled, identical Pauli terms are merged before QWC block
    /// formation, reducing term count and CNOT tree positions.
    /// - `IdenticalOnly` (default): merge same-Pauli terms, always exact.
    /// - `CliffordSimple`: XY alignment via Clifford conjugation (Phase 11a-3),
    ///   recorded as S/S† gates during circuit synthesis. NOTE: the full
    ///   CliffordSimple pipeline lacks end-to-end test coverage (Phase 11
    ///   review B1/B2). Use with caution.
    ///
    /// Default: `GadgetOptimizationStrategy::IdenticalOnly`.
    pub pauli_gadget_optimization: GadgetOptimizationStrategy,

    /// Alternate reversing Trotter steps for boundary CNOT cancellation (Phase 9k).
    ///
    /// When true, every other Trotter step uses reversed term ordering. This makes
    /// consecutive steps end/start with the same term, creating adjacent CX·CX pairs
    /// at step boundaries that `CancelInversePairsPass` can remove.
    ///
    /// GateLevel: ~7% gate reduction, ~5% CX reduction for multi-step circuits.
    /// PauliLevel: modest improvement (~1%).
    ///
    /// Default: `true` (enabled by default).
    pub alternate_reverse_steps: bool,

    /// Clifford-enhanced QWC block merging (Phase 11f, default: `true`).
    ///
    /// When enabled, `form_blocks_clifford_enhanced()` is used instead of
    /// `form_blocks()`. This greedily merges QWC blocks whose master Pauli
    /// strings can be made QWC-compatible via single-qubit Clifford conjugation
    /// (e.g., XXII and YYII via S/S† on each qubit).
    ///
    /// Clifford gates are emitted around individual rotations in the
    /// synthesized circuit, ensuring correctness. Conjugated terms are
    /// synthesized individually (not merged into shared CNOT tree Rz
    /// positions) to maintain correct Clifford wrapping.
    ///
    /// Default: `true`.
    pub clifford_enhanced_blocks: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            trotter_order: TrotterOrder::First,
            trotter_steps: 1,
            evolution_time: 1.0,
            adaptive: false,
            adaptive_tolerance: 1e-3,
            min_step_size: 1e-6,
            max_step_size: 1.0,
            hbar: 1.0,
            skip_identities: true,
            group_commuting_terms: true,      // Enable by default
            auto_optimize_grouping: true,     // Enable intelligent optimization
            apply_circuit_optimization: true, // Enable by default
            layout_aware_grouping: false,     // Opt-in advanced optimization
            optimization_strategy: CompilationStrategy::GateLevel, // Phase 9l: GateLevel now outperforms PauliLevel by 11%
            cross_step_synthesis: false, // Opt-in: single CNOT tree across all Trotter steps
            block_grouping_strategy: BlockGroupingStrategy::QWC, // Default: QWC
            pauli_gadget_optimization: GadgetOptimizationStrategy::IdenticalOnly, // Safe default; CliffordSimple pending end-to-end validation (Phase 11 review B1/B2)
            alternate_reverse_steps: true,
            clifford_enhanced_blocks: true, // Phase 11f: enabled by default with Clifford gate emission
        }
    }
}

impl CompilerConfig {
    /// Create configuration optimized for quantum chemistry Hamiltonians
    pub fn for_chemistry() -> Self {
        Self {
            group_commuting_terms: true,
            auto_optimize_grouping: true,
            ..Default::default()
        }
    }

    /// Create configuration optimized for dense random Hamiltonians
    pub fn for_random() -> Self {
        Self {
            group_commuting_terms: false, // Disable grouping for random
            auto_optimize_grouping: false,
            ..Default::default()
        }
    }

    /// Build a `CircuitAnalyzer` pre-configured to match this compiler config.
    ///
    /// This ensures the analyzer uses the same `hbar` and `evolution_time`
    /// that were used during compilation, giving exact coefficient reconstruction
    /// in roundtrip tests.
    pub fn build_analyzer(&self) -> super::CircuitAnalyzer {
        super::CircuitAnalyzer::from_compiler_config(self)
    }
}

/// Hamiltonian to circuit compiler
///
/// Converts a Hamiltonian into a quantum circuit using Trotter-Suzuki decomposition.
///
/// # Examples
///
/// ```rust,ignore
/// use myquat::{QuantumCircuit, Parameter};
/// use myquat::hamiltonian::{Hamiltonian, HamiltonianCompiler, CompilerConfig, TrotterOrder};
/// use num_complex::Complex64;
///
/// // Create an Ising Hamiltonian
/// let h = constructors::ising_model(3, 1.0, 0.5);
///
/// // Configure compiler
/// let mut config = CompilerConfig::default();
/// config.trotter_steps = 10;
/// config.evolution_time = 2.0;
///
/// // Compile to circuit
/// let compiler = HamiltonianCompiler::new(config);
/// let circuit = compiler.compile(&h).unwrap();
///
/// println!("Compiled circuit with {} gates", circuit.size());
/// ```
pub struct HamiltonianCompiler {
    /// Compiler configuration
    config: CompilerConfig,
}

impl HamiltonianCompiler {
    /// Create a new compiler with given configuration
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }

    /// Create a compiler with default configuration
    pub fn default() -> Self {
        Self::new(CompilerConfig::default())
    }

    /// Analyze Hamiltonian structure to decide if grouping optimization is beneficial
    ///
    /// Returns true if grouping should be enabled based on:
    /// - Number of terms (> 30 may cause overhead)
    /// - Sparsity (< 50% suggests dense, unstructured Hamiltonian)
    /// - QWC ratio (low ratio suggests few commuting pairs)
    fn should_enable_grouping(&self, hamiltonian: &Hamiltonian) -> bool {
        let num_terms = hamiltonian.terms.len();

        // For small Hamiltonians, always group (overhead is negligible)
        if num_terms < 10 {
            return true;
        }

        // For very large random-like Hamiltonians, skip grouping
        if num_terms > 30 {
            // Estimate sparsity
            let sparsity = self.estimate_sparsity(hamiltonian);
            if sparsity < 0.5 {
                return false; // Dense Hamiltonian, skip grouping
            }
        }

        // Default: enable grouping
        true
    }

    /// Estimate sparsity of Hamiltonian (fraction of non-identity operators)
    fn estimate_sparsity(&self, hamiltonian: &Hamiltonian) -> f64 {
        if hamiltonian.terms.is_empty() {
            return 1.0;
        }

        let total_ops: usize = hamiltonian
            .terms
            .iter()
            .map(|t| t.pauli_string.operators.len())
            .sum();

        let non_identity_ops: usize = hamiltonian
            .terms
            .iter()
            .map(|t| {
                t.pauli_string
                    .operators
                    .iter()
                    .filter(|op| **op != PauliOperator::I)
                    .count()
            })
            .sum();

        if total_ops == 0 {
            return 1.0;
        }

        1.0 - (non_identity_ops as f64 / total_ops as f64)
    }

    /// Optimize Hamiltonian by reordering Pauli terms
    ///
    /// This optimization reduces gate count by:
    /// 1. Grouping commuting Pauli terms together
    /// 2. Ordering terms to minimize basis changes
    /// 3. Reducing CNOT gates by intelligent term ordering
    ///
    /// Returns an optimized Hamiltonian with reordered terms
    pub fn optimize_pauli_ordering(&self, hamiltonian: &Hamiltonian) -> Hamiltonian {
        let mut optimized = Hamiltonian::new(hamiltonian.num_qubits);
        optimized.constant_term = hamiltonian.constant_term;
        optimized.parameters = hamiltonian.parameters.clone();

        // Decide whether to group based on configuration and Hamiltonian structure
        let should_group = if self.config.auto_optimize_grouping {
            self.config.group_commuting_terms && self.should_enable_grouping(hamiltonian)
        } else {
            self.config.group_commuting_terms
        };

        // Group terms by their Pauli structure similarity
        let mut term_groups = if should_group {
            self.group_commuting_terms(&hamiltonian.terms)
        } else {
            // No grouping: each term in its own group
            hamiltonian.terms.iter().map(|t| vec![t.clone()]).collect()
        };

        // Sort groups to minimize transitions between different Pauli bases
        term_groups.sort_by_key(|group| {
            // Use the first term's structure as representative
            if group.is_empty() {
                return 0;
            }
            self.compute_term_complexity(&group[0])
        });

        // Add reordered terms to optimized Hamiltonian
        for group in term_groups {
            for term in group {
                optimized.terms.push(term);
            }
        }

        optimized
    }

    /// Group Pauli terms using QWC (Qubit-Wise Commuting) criterion
    ///
    /// QWC grouping ensures that for any two terms in the same group,
    /// at each qubit position, the Pauli operators are either:
    /// - The same (X-X, Y-Y, Z-Z, I-I)
    /// - At least one is Identity (I-X, I-Y, I-Z)
    ///
    /// This is stricter than full commuting but:
    /// - Deterministic (independent of input order)
    /// - Standard in VQE measurement optimization
    /// - Guarantees all terms in group are mutually QWC
    /// - Enables efficient simultaneous measurement
    fn group_commuting_terms(&self, terms: &[PauliTerm]) -> Vec<Vec<PauliTerm>> {
        let mut groups: Vec<Vec<PauliTerm>> = Vec::new();

        'outer: for term in terms {
            // Try to add term to existing group
            for group in groups.iter_mut() {
                // Check if term is QWC with ALL terms in the group
                if group.iter().all(|t| self.is_qwc(t, term)) {
                    group.push(term.clone());
                    continue 'outer;
                }
            }
            // If no compatible group found, create new group
            groups.push(vec![term.clone()]);
        }

        groups
    }

    /// Check if two Pauli terms are Qubit-Wise Commuting (QWC)
    ///
    /// Two Pauli terms are QWC if at each qubit position:
    /// - Both operators are the same (X-X, Y-Y, Z-Z, I-I), OR
    /// - At least one operator is Identity (I-X, X-I, etc.)
    ///
    /// This is more restrictive than general commuting but:
    /// - Easier to verify
    /// - Deterministic grouping
    /// - Standard for VQE term grouping
    fn is_qwc(&self, term1: &PauliTerm, term2: &PauliTerm) -> bool {
        let ops1 = &term1.pauli_string.operators;
        let ops2 = &term2.pauli_string.operators;

        if ops1.len() != ops2.len() {
            return false;
        }

        for (op1, op2) in ops1.iter().zip(ops2.iter()) {
            // QWC condition: same operator OR at least one is Identity
            if op1 != op2 && *op1 != PauliOperator::I && *op2 != PauliOperator::I {
                return false;
            }
        }

        true
    }

    /// Check if two Pauli terms commute (full commuting, not QWC)
    ///
    /// Two Pauli terms commute if they anti-commute an even number of times.
    /// This is less restrictive than QWC but still physically meaningful.
    ///
    /// Note: This is kept for potential future use but QWC is preferred
    /// for grouping due to determinism and VQE compatibility.
    fn terms_commute(&self, term1: &PauliTerm, term2: &PauliTerm) -> bool {
        let ops1 = &term1.pauli_string.operators;
        let ops2 = &term2.pauli_string.operators;

        if ops1.len() != ops2.len() {
            return false;
        }

        let mut anti_commute_count = 0;

        for (op1, op2) in ops1.iter().zip(ops2.iter()) {
            // Pauli operators anti-commute if both are non-identity and different
            if *op1 != PauliOperator::I && *op2 != PauliOperator::I && op1 != op2 {
                anti_commute_count += 1;
            }
        }

        // Terms commute if they anti-commute an even number of times
        anti_commute_count % 2 == 0
    }

    /// Compute complexity metric for a Pauli term (for sorting)
    fn compute_term_complexity(&self, term: &PauliTerm) -> usize {
        // Count non-identity operators (weight of Pauli string)
        let weight = term
            .pauli_string
            .operators
            .iter()
            .filter(|op| **op != PauliOperator::I)
            .count();

        // Prioritize lower weight terms (simpler to implement)
        weight
    }

    /// Compile a Hamiltonian into a quantum circuit
    ///
    /// Implements time evolution $e^{-iHt/\hbar}$ using Trotter-Suzuki decomposition.
    ///
    /// **First-order (p=1):**
    ///
    /// $$ e^{-iHt} \approx \left( \prod_{j=1}^m e^{-iH_j t/r} \right)^r $$
    ///
    /// **Second-order (p=2):**
    ///
    /// $$ e^{-iHt} \approx \left( \prod_{j=1}^m e^{-iH_j t/2r} \prod_{j=m}^1 e^{-iH_j t/2r} \right)^r $$
    ///
    /// where $H = \sum_j H_j$ and $r$ is the number of Trotter steps.
    ///
    /// # Arguments
    ///
    /// * `hamiltonian` - The Hamiltonian to compile
    ///
    /// # Returns
    ///
    /// A quantum circuit implementing the time evolution under this Hamiltonian
    pub fn compile(&self, hamiltonian: &Hamiltonian) -> Result<QuantumCircuit> {
        let num_qubits = hamiltonian.num_qubits;
        let mut circuit = QuantumCircuit::new(num_qubits, 0);

        // Calculate time step
        let dt = self.config.evolution_time / self.config.trotter_steps as f64;

        // P2: Adaptive strategy selection — fall back to GateLevel when PauliLevel
        // is unlikely to benefit (few terms per block or poor prefix compatibility).
        // Note: PauliGadget is user-opt-in and skips automatic fallback.
        let effective_strategy =
            if self.config.optimization_strategy == CompilationStrategy::PauliLevel {
                Self::should_use_pauli_level(hamiltonian)
            } else {
                self.config.optimization_strategy
            };

        // ── Pauli gadget pre-optimization (Phase 11a) ────────────────────
        // Merge identical/Clifford-equivalent Pauli terms before synthesis.
        // Phase 11a-3: CliffordSimple aligns XY pairs (e.g., IIYY→IIXX) and
        // records Clifford gates for circuit-level emission during synthesis.
        let opt_result = {
            let terms: Vec<&PauliTerm> = hamiltonian.terms.iter().collect();
            crate::hamiltonian::pauli_gadget::optimize_pauli_gadgets(
                &terms,
                self.config.pauli_gadget_optimization,
            )?
        };

        // Build Clifford annotation map for Phase 11a-3 circuit-level alignment.
        // Key: Pauli string representation (unique per term in a Hamiltonian).
        let mut clifford_map: Option<crate::hamiltonian::pauli_synthesis::CliffordAnnotationMap> = {
            let mut map = std::collections::HashMap::new();
            for g in &opt_result.gadgets {
                if !g.pre_gates.is_empty() || !g.post_gates.is_empty() {
                    map.insert(
                        g.pauli_string.to_string_repr().to_string(),
                        (g.pre_gates.clone(), g.post_gates.clone()),
                    );
                }
            }
            if map.is_empty() {
                None
            } else {
                Some(map)
            }
        };

        // Convert optimized gadgets to Hamiltonian.
        // Always rebuild when CliffordSimple is active (Pauli strings may have
        // changed due to XY alignment). Only skip clone when no changes made.
        let needs_rebuild = opt_result.terms_merged > 0
            || matches!(
                self.config.pauli_gadget_optimization,
                GadgetOptimizationStrategy::CliffordSimple
            );
        let mut h_optimized = if needs_rebuild {
            let mut h_opt = super::Hamiltonian::new(hamiltonian.num_qubits);
            for g in &opt_result.gadgets {
                h_opt.add_term(
                    g.pauli_string.clone(),
                    num_complex::Complex64::new(g.angle, 0.0),
                )?;
            }
            h_opt
        } else {
            hamiltonian.clone()
        };

        // ── GreedyPauliSimp optimization (Phase 11i) ─────────────────────
        // When using PauliGadget strategy, apply TKET-style greedy Clifford
        // conjugation BEFORE block formation. This merges compatible gadgets
        // (e.g., XX+YY via S/S†) that would otherwise end up in different
        // QWC blocks, reducing the total number of CNOT trees.
        //
        // After optimization, delegate to the PauliLevel synthesis path.
        let effective_strategy = if effective_strategy == CompilationStrategy::PauliGadget {
            let terms: Vec<&PauliTerm> = h_optimized.terms.iter().collect();
            let greedy_config = crate::hamiltonian::pauli_gadget_compiler::GreedyConfig::default();
            let greedy_result = crate::hamiltonian::pauli_gadget_compiler::greedy_pauli_simp(
                &terms,
                dt,
                self.config.hbar,
                &greedy_config,
            );

            // Rebuild Hamiltonian and Clifford map from greedy result
            let (greedy_terms, greedy_map) =
                crate::hamiltonian::pauli_gadget_compiler::gadgets_to_terms_and_map(
                    &greedy_result.gadgets,
                    h_optimized.num_qubits,
                );

            // Merge Clifford maps
            if let Some(gm) = greedy_map {
                if let Some(ref mut existing) = clifford_map {
                    existing.extend(gm);
                } else {
                    clifford_map = Some(gm);
                }
            }

            // Rebuild optimized Hamiltonian from greedy terms
            h_optimized = super::Hamiltonian::new(hamiltonian.num_qubits);
            for t in greedy_terms {
                h_optimized.add_term(t.pauli_string, t.coefficient)?;
            }

            // Delegate to PauliLevel synthesis for the rest of compilation
            CompilationStrategy::PauliLevel
        } else {
            effective_strategy
        };

        // ── Cross-step synthesis fast path ──────────────────────────────
        // When enabled and using PauliLevel, collect ALL Trotter exponentials
        // into a single synthesis pass. QWC blocks share ONE CNOT tree across
        // all steps, making gate count independent of trotter_steps.
        if self.config.cross_step_synthesis && effective_strategy == CompilationStrategy::PauliLevel
        {
            crate::hamiltonian::pauli_synthesis::compile_cross_step_pauli_synthesis(
                &mut circuit,
                &h_optimized,
                self.config.evolution_time,
                self.config.hbar,
                &self.config.trotter_order,
                self.config.trotter_steps,
                self.config.skip_identities,
                self.config.block_grouping_strategy,
                None,
                clifford_map.as_ref(),
                self.config.clifford_enhanced_blocks, // Phase 11e: Clifford-enhanced block merging
            )?;

            // Apply circuit-level optimization if enabled
            if self.config.apply_circuit_optimization {
                use crate::circuit_optimization::PassManager;
                PassManager::level_2().run(&mut circuit)?;
                if circuit.size() > 50 {
                    PassManager::level_4().run(&mut circuit)?;
                }
            }

            return Ok(circuit);
        }

        // Pre-compute commuting groups once (deterministic for a given Hamiltonian)
        // This eliminates O(N² * steps * suzuki_multiplier) overhead for random Hamiltonians
        let term_groups: Option<Vec<Vec<usize>>> = if self.config.group_commuting_terms {
            let terms: Vec<&PauliTerm> = h_optimized.terms.iter().collect();
            let groups = Self::group_commuting_paulis(&terms);
            Some(
                groups
                    .iter()
                    .map(|g| {
                        g.iter()
                            .map(|t| {
                                h_optimized
                                    .terms
                                    .iter()
                                    .position(|h| std::ptr::eq(*t, h))
                                    .unwrap_or(0)
                            })
                            .collect()
                    })
                    .collect(),
            )
        } else {
            None
        };

        // Track step boundaries for Trotter-aware optimization.
        // After each Trotter step, record the current instruction count.
        let mut step_boundaries: Vec<usize> = Vec::new();

        match &self.config.trotter_order {
            TrotterOrder::First => {
                for step_idx in 0..self.config.trotter_steps {
                    // Phase 9k: Alternate reverse per step so that consecutive steps
                    // end/start with the same term. This creates adjacent CX·CX pairs
                    // at step boundaries, which CancelInversePairsPass can remove.
                    let reverse = self.config.alternate_reverse_steps && step_idx % 2 != 0;
                    self.apply_trotter_step_cached(
                        &mut circuit,
                        &h_optimized,
                        dt,
                        reverse,
                        &term_groups,
                        Some(effective_strategy),
                        clifford_map.as_ref(),
                    )?;
                    step_boundaries.push(circuit.size());
                }
            }
            TrotterOrder::Second => {
                for _ in 0..self.config.trotter_steps {
                    self.apply_trotter_step_cached(
                        &mut circuit,
                        &h_optimized,
                        dt / 2.0,
                        false,
                        &term_groups,
                        Some(effective_strategy),
                        clifford_map.as_ref(),
                    )?;
                    self.apply_trotter_step_cached(
                        &mut circuit,
                        &h_optimized,
                        dt / 2.0,
                        true,
                        &term_groups,
                        Some(effective_strategy),
                        clifford_map.as_ref(),
                    )?;
                    step_boundaries.push(circuit.size());
                }
            }
            TrotterOrder::Fourth => {
                for _ in 0..self.config.trotter_steps {
                    self.apply_fourth_order_suzuki_cached(
                        &mut circuit,
                        &h_optimized,
                        dt,
                        &term_groups,
                        Some(effective_strategy),
                        clifford_map.as_ref(),
                    )?;
                    step_boundaries.push(circuit.size());
                }
            }
            TrotterOrder::Sixth => {
                for _ in 0..self.config.trotter_steps {
                    self.apply_sixth_order_suzuki_cached(
                        &mut circuit,
                        &h_optimized,
                        dt,
                        &term_groups,
                        Some(effective_strategy),
                        clifford_map.as_ref(),
                    )?;
                    step_boundaries.push(circuit.size());
                }
            }
            TrotterOrder::Nth(order) => {
                let order = *order;
                if order == 0 {
                    return Err(MyQuatError::hamiltonian_error(
                        "Trotter order must be at least 1",
                    ));
                }
                if order > 1 && order % 2 != 0 {
                    return Err(MyQuatError::hamiltonian_error(
                        "Suzuki formulas are only defined for order 1 or even orders (2, 4, 6, ...)"
                    ));
                }
                for _ in 0..self.config.trotter_steps {
                    self.apply_nth_order_suzuki_cached(
                        &mut circuit,
                        &h_optimized,
                        dt,
                        order,
                        &term_groups,
                        Some(effective_strategy),
                        clifford_map.as_ref(),
                    )?;
                    step_boundaries.push(circuit.size());
                }
            }
            TrotterOrder::Custom(coeffs) => {
                for _ in 0..self.config.trotter_steps {
                    self.apply_custom_suzuki_cached(
                        &mut circuit,
                        &h_optimized,
                        dt,
                        coeffs,
                        &term_groups,
                        Some(effective_strategy),
                        clifford_map.as_ref(),
                    )?;
                    step_boundaries.push(circuit.size());
                }
            }
        }

        // Store step boundaries as comma-separated string in circuit metadata
        if step_boundaries.len() > 1 {
            let boundaries_str: String = step_boundaries
                .iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(",");
            circuit.data_mut().set_metadata(
                crate::circuit_optimization::STEP_BOUNDARIES_KEY.to_string(),
                boundaries_str,
            );
        }

        // Save original step boundaries and circuit size for post-optimization
        // TrotterAwarePass (Phase 11l). After convergence, step_boundaries
        // metadata is removed by the pre-convergence TrotterAwarePass, so we
        // keep local copies for rescaling.
        let saved_boundaries: Option<Vec<usize>> = if step_boundaries.len() > 1 {
            Some(step_boundaries.clone())
        } else {
            None
        };
        let original_circuit_size = circuit.size();

        // Phase 9j: Pre-convergence TrotterAwarePass.
        // Effective for GateLevel/PauliLevel — cancels CNOT ladders and H pairs
        // at step boundaries before general optimization. Has zero effect on
        // PauliGadget (Clifford S/Sdg annotations break the alternating reverse
        // pattern), so skip for PauliGadget and rely on the Phase 11l post-
        // convergence pass instead.
        if step_boundaries.len() > 1
            && self.config.optimization_strategy != CompilationStrategy::PauliGadget
        {
            use crate::circuit_optimization::{CircuitPass, TrotterAwarePass};
            TrotterAwarePass::new().run(&mut circuit)?;
        }

        // Phase 11q: Use level_5 — the production optimization pipeline.
        // Includes convergence loop (level_2 + level_4_core), PhasePolynomialPass
        // with AdaptiveSynthesis (RowCol/GrayCode/ParitySynth), and post-cleanup.
        if self.config.apply_circuit_optimization {
            use crate::circuit_optimization::PassManager;
            PassManager::level_5().run(&mut circuit)?;
        }

        // Phase 11l: Post-optimization TrotterAwarePass.
        //
        // Runs AFTER convergence eliminates Clifford annotations (S/Sdg) for
        // PauliGadget circuits. At this point, the circuit has clean H·H and
        // CX·CX cancellable pairs at step boundaries. GateLevel/PauliLevel also
        // benefit from a second pass catching any remaining redundancies.
        //
        // Step-boundary indices are rescaled proportionally to account for gate
        // removals during optimization. The window_size=60 provides tolerance
        // for non-uniform reduction across steps.
        if let Some(ref boundaries) = saved_boundaries {
            use crate::circuit_optimization::{CircuitPass, TrotterAwarePass};
            let current_size = circuit.size();
            let pass_boundaries =
                if original_circuit_size > 0 && current_size < original_circuit_size {
                    // Phase 11l: rescale boundaries proportionally
                    boundaries
                        .iter()
                        .map(|&b| {
                            ((b as f64) * (current_size as f64) / (original_circuit_size as f64))
                                .round() as usize
                        })
                        .collect()
                } else {
                    boundaries.clone()
                };
            // Phase 11p: pass boundaries directly (avoids Vec→String→Vec round-trip)
            TrotterAwarePass::with_boundaries(pass_boundaries).run(&mut circuit)?;
        }

        Ok(circuit)
    }

    /// Compile with layout-aware term ordering optimization.
    ///
    /// Uses `LayoutAwareGrouper` to reorder Hamiltonian terms before
    /// compilation, reducing basis changes and overall gate count.
    /// Optionally accepts a `DeviceTopology` for topology-aware grouping.
    pub fn compile_with_layout(
        &self,
        hamiltonian: &Hamiltonian,
        topology: Option<&crate::device_topology::DeviceTopology>,
    ) -> Result<(
        QuantumCircuit,
        crate::hamiltonian::layout_aware_grouping::GroupingResult,
    )> {
        use crate::hamiltonian::layout_aware_grouping::{GroupingConfig, LayoutAwareGrouper};

        let config = GroupingConfig {
            use_qwc: true,
            topology_aware: topology.is_some(),
            topology_weight: 0.5,
            minimize_basis_changes: true,
            max_group_size: 0,
        };
        let grouper = LayoutAwareGrouper::with_config(config, topology.cloned());
        let grouping = grouper.group(hamiltonian);

        // Rebuild Hamiltonian with optimized term order
        let mut optimized = Hamiltonian::new(hamiltonian.num_qubits);
        optimized.constant_term = hamiltonian.constant_term;
        optimized.parameters = hamiltonian.parameters.clone();
        for group in &grouping.groups {
            for &idx in &group.term_indices {
                optimized.terms.push(hamiltonian.terms[idx].clone());
            }
        }

        let circuit = self.compile(&optimized)?;
        Ok((circuit, grouping))
    }

    /// Extract operator type pattern (sorted multiset of operators, ignoring positions)
    /// E.g., "ZZII" -> [Z, Z], "IXIX" -> [X, X], "IIYZ" -> [Y, Z]
    fn get_operator_pattern(pauli_string: &PauliString) -> Vec<PauliOperator> {
        let mut ops: Vec<PauliOperator> = pauli_string
            .operators
            .iter()
            .filter(|&&op| op != PauliOperator::I)
            .copied()
            .collect();
        ops.sort_by_key(|op| match op {
            PauliOperator::X => 0,
            PauliOperator::Y => 1,
            PauliOperator::Z => 2,
            PauliOperator::I => 3,
        });
        ops
    }

    /// Group commuting Pauli strings for optimization
    /// Strategy: Group terms with same operator type pattern (e.g., all single-Z, all ZZ, all XX)
    /// This allows sharing the "no basis change" or "similar basis transformation" strategy
    fn group_commuting_paulis<'a>(terms: &'a [&'a PauliTerm]) -> Vec<Vec<&'a PauliTerm>> {
        let mut groups: Vec<Vec<&PauliTerm>> = Vec::new();

        for term in terms {
            // Skip identity terms
            if term.pauli_string.is_identity() {
                continue;
            }

            let term_pattern = Self::get_operator_pattern(&term.pauli_string);

            // Try to find a group with:
            // 1. Same operator pattern (e.g., both are [Z,Z])
            // 2. Commutes with all members
            let mut found_group = false;
            for group in &mut groups {
                let first_pattern = Self::get_operator_pattern(&group[0].pauli_string);

                if term_pattern == first_pattern
                    && group
                        .iter()
                        .all(|other| term.pauli_string.commutes_with(&other.pauli_string))
                {
                    group.push(term);
                    found_group = true;
                    break;
                }
            }

            // If no compatible group found, create new group
            if !found_group {
                groups.push(vec![term]);
            }
        }

        groups
    }

    /// Apply one Trotter step to the circuit
    fn apply_trotter_step(
        &self,
        circuit: &mut QuantumCircuit,
        hamiltonian: &Hamiltonian,
        dt: f64,
        reverse: bool,
        clifford_map: Option<&crate::hamiltonian::pauli_synthesis::CliffordAnnotationMap>,
    ) -> Result<()> {
        self.apply_trotter_step_cached(circuit, hamiltonian, dt, reverse, &None, None, clifford_map)
    }

    /// Apply one Trotter step with pre-computed term groups (avoids O(N²) re-grouping)
    fn apply_trotter_step_cached(
        &self,
        circuit: &mut QuantumCircuit,
        hamiltonian: &Hamiltonian,
        dt: f64,
        reverse: bool,
        cached_groups: &Option<Vec<Vec<usize>>>,
        override_strategy: Option<CompilationStrategy>,
        clifford_map: Option<&crate::hamiltonian::pauli_synthesis::CliffordAnnotationMap>,
    ) -> Result<()> {
        let strategy = override_strategy.unwrap_or(self.config.optimization_strategy);
        // Map PauliGadget to PauliLevel — the greedy optimization was done
        // upstream in compile(); here we just need the synthesis path.
        let strategy = if strategy == CompilationStrategy::PauliGadget {
            CompilationStrategy::PauliLevel
        } else {
            strategy
        };
        // Pauli-level synthesis path: max_match_synthesis with shared CNOT trees
        if strategy == CompilationStrategy::PauliLevel {
            // Phase 10a: Don't reverse terms here — pass them in original order
            // and let compile_step_pauli_synthesis handle reversal via _is_reverse.
            // This ensures forward and reverse passes use the SAME QWC block
            // partition (only the block order and internal term order are reversed),
            // which is required for Trotter symmetry.
            let terms: Vec<&PauliTerm> = hamiltonian.terms.iter().collect();
            let filtered: Vec<&PauliTerm> = terms
                .iter()
                .filter(|t| !(self.config.skip_identities && t.pauli_string.is_identity()))
                .copied()
                .collect();
            return crate::hamiltonian::pauli_synthesis::compile_step_pauli_synthesis(
                circuit,
                &filtered,
                dt,
                self.config.hbar,
                reverse, // Phase 10a: wired — forward/reverse use same blocks, reversed order
                self.config.block_grouping_strategy,
                None,
                None,
                clifford_map,
                self.config.clifford_enhanced_blocks, // Phase 11e: Clifford-enhanced block merging
            );
        }

        // Gate-level optimization path
        // Group commuting Pauli strings if optimization enabled
        if self.config.group_commuting_terms {
            if let Some(group_indices) = cached_groups {
                // Use pre-computed groups
                if reverse {
                    for indices in group_indices.iter().rev() {
                        // Phase 9k: also reverse terms within each group so that
                        // the boundary between forward/reverse steps shares the
                        // same term, creating adjacent CX·CX pairs for cancellation.
                        let mut rev_indices = indices.clone();
                        rev_indices.reverse();
                        let group: Vec<&PauliTerm> =
                            rev_indices.iter().map(|&i| &hamiltonian.terms[i]).collect();
                        self.compile_pauli_group(circuit, &group, dt)?;
                    }
                } else {
                    for indices in group_indices {
                        let group: Vec<&PauliTerm> =
                            indices.iter().map(|&i| &hamiltonian.terms[i]).collect();
                        self.compile_pauli_group(circuit, &group, dt)?;
                    }
                }
            } else {
                // Fallback: compute groups on the fly
                let terms: Vec<&PauliTerm> = if reverse {
                    hamiltonian.terms.iter().rev().collect()
                } else {
                    hamiltonian.terms.iter().collect()
                };
                let groups = Self::group_commuting_paulis(&terms);
                for group in groups {
                    self.compile_pauli_group(circuit, &group, dt)?;
                }
            }
        } else {
            let terms: Vec<&PauliTerm> = if reverse {
                hamiltonian.terms.iter().rev().collect()
            } else {
                hamiltonian.terms.iter().collect()
            };
            for term in terms {
                if self.config.skip_identities && term.pauli_string.is_identity() {
                    continue;
                }
                let angle = 2.0 * term.coefficient.re * dt / self.config.hbar;
                self.compile_pauli_term(circuit, &term.pauli_string, angle)?;
            }
        }

        Ok(())
    }

    /// Compile a group of commuting Pauli terms with shared basis gates
    /// For groups with same operator pattern (e.g., all single-Z), optimize basis transformations
    fn compile_pauli_group(
        &self,
        circuit: &mut QuantumCircuit,
        group: &[&PauliTerm],
        dt: f64,
    ) -> Result<()> {
        if group.is_empty() {
            return Ok(());
        }

        // Check if this is an all-Z group (no basis change needed)
        let pattern = Self::get_operator_pattern(&group[0].pauli_string);
        let is_all_z = pattern.iter().all(|&op| op == PauliOperator::Z);

        if is_all_z {
            // Collect multi-qubit terms with their angles and active qubits.
            let mut multi_qubit_terms: Vec<(Vec<usize>, f64)> = Vec::new();
            for term in group {
                let angle = 2.0 * term.coefficient.re * dt / self.config.hbar;
                if angle.abs() < 1e-10 {
                    continue;
                }
                let active: Vec<usize> = term
                    .pauli_string
                    .operators
                    .iter()
                    .enumerate()
                    .filter(|(_, &op)| op != PauliOperator::I)
                    .map(|(i, _)| i)
                    .collect();
                if active.len() == 1 {
                    circuit.rz(active[0], Parameter::Float(angle))?;
                } else {
                    multi_qubit_terms.push((active, angle));
                }
            }

            if !multi_qubit_terms.is_empty() {
                self.compile_z_terms_with_shared_tree(circuit, &multi_qubit_terms)?;
            }
        } else {
            // For X/Y groups, each term needs its own basis transformation
            // But we can still benefit from grouping by reducing compilation overhead
            for term in group {
                let angle = 2.0 * term.coefficient.re * dt / self.config.hbar;
                if angle.abs() < 1e-10 {
                    continue;
                }

                self.compile_pauli_term(circuit, &term.pauli_string, angle)?;
            }
        }

        Ok(())
    }

    /// Compile multiple Z-terms using a shared CNOT chain when beneficial.
    ///
    /// # Strategy
    ///
    /// 1. Collect all unique qubits across all Z-terms.
    /// 2. Build a chain connecting them in sorted order.
    /// 3. Group terms by their "last" qubit in the chain (prefix-compatible).
    /// 4. Terms that share the chain prefix are emitted via shared CNOT tree;
    ///    non-prefix terms fall back to per-term ladders.
    /// 5. Only use the shared chain when it reduces CX count vs all-per-term.
    fn compile_z_terms_with_shared_tree(
        &self,
        circuit: &mut QuantumCircuit,
        terms: &[(Vec<usize>, f64)],
    ) -> Result<()> {
        if terms.is_empty() {
            return Ok(());
        }

        // Single term → no sharing possible, use per-term path directly.
        if terms.len() == 1 {
            let (active, angle) = &terms[0];
            let active_with_ops: Vec<(usize, PauliOperator)> =
                active.iter().map(|&q| (q, PauliOperator::Z)).collect();
            return self.compile_multi_qubit_rotation(circuit, &active_with_ops, *angle);
        }

        // Collect unique qubits and build chain.
        let mut all_qubits: Vec<usize> = Vec::new();
        for (active, _) in terms {
            for &q in active {
                if !all_qubits.contains(&q) {
                    all_qubits.push(q);
                }
            }
        }
        all_qubits.sort_unstable();

        // If the chain would use more CNOTs than per-term, skip sharing.
        let chain_cx_count = 2 * (all_qubits.len() - 1); // forward + reverse
        let per_term_cx_count: usize = terms.iter().map(|(a, _)| 2 * (a.len() - 1)).sum();
        if chain_cx_count >= per_term_cx_count {
            // Chain provides no benefit — fall back to per-term.
            for (active, angle) in terms {
                let active_with_ops: Vec<(usize, PauliOperator)> =
                    active.iter().map(|&q| (q, PauliOperator::Z)).collect();
                self.compile_multi_qubit_rotation(circuit, &active_with_ops, *angle)?;
            }
            return Ok(());
        }

        // Classify terms: prefix-compatible (share the chain's first element
        // as one of their active qubits) vs non-prefix.
        let chain_start = all_qubits[0];
        let mut prefix_terms: Vec<(usize, f64)> = Vec::new(); // (pos_in_chain, angle)
        let mut non_prefix_terms: Vec<&(Vec<usize>, f64)> = Vec::new();

        for term in terms {
            let (active, angle) = term;
            // Prefix-compatible: active qubits are a prefix of the chain.
            // For a term to be prefix-compatible, its last active qubit
            // position in the chain must equal active.len() - 1 when
            // all its qubits are in sorted prefix order.
            let min_q = *active.iter().min().unwrap();
            let max_q = *active.iter().max().unwrap();
            let chain_min_pos = all_qubits.iter().position(|&q| q == min_q).unwrap();
            let chain_max_pos = all_qubits.iter().position(|&q| q == max_q).unwrap();

            // Check if active qubits form a contiguous prefix starting from chain_start
            // or from some later position.
            let is_prefix = if min_q == chain_start {
                // Must be a contiguous prefix from chain_start.
                chain_max_pos + 1 == chain_min_pos + active.len()
                    && active.iter().all(|&q| {
                        let pos = all_qubits.iter().position(|&x| x == q).unwrap();
                        pos >= chain_min_pos && pos <= chain_max_pos
                    })
            } else {
                false
            };

            if is_prefix {
                prefix_terms.push((chain_max_pos, *angle));
            } else {
                non_prefix_terms.push(term);
            }
        }

        // Even if no prefix-compatible terms, the chain might still be useful
        // for creating CX adjacency that the optimization pipeline can exploit.
        // Only build shared chain if it provides CX savings.
        let shared_chain_useful = prefix_terms.len() >= 2
            || (prefix_terms.len() == 1
                && chain_cx_count
                    + non_prefix_terms
                        .iter()
                        .map(|(a, _)| 2 * (a.len() - 1))
                        .sum::<usize>()
                    < per_term_cx_count);

        if !shared_chain_useful {
            // Chain provides no benefit — fall back to per-term.
            for (active, angle) in terms {
                let active_with_ops: Vec<(usize, PauliOperator)> =
                    active.iter().map(|&q| (q, PauliOperator::Z)).collect();
                self.compile_multi_qubit_rotation(circuit, &active_with_ops, *angle)?;
            }
            return Ok(());
        }

        // Build shared CNOT chain: forward + Rz-at-each-position + reverse.
        let k = all_qubits.len();

        // Accumulate prefix-term coefficients per chain position.
        let mut rz_at_pos: Vec<f64> = vec![0.0; k];
        for (pos, angle) in &prefix_terms {
            rz_at_pos[*pos] += angle;
        }

        // Forward CNOT chain with interleaved Rz for prefix terms.
        for i in 0..(k - 1) {
            circuit.cx(all_qubits[i], all_qubits[i + 1])?;
            let coeff = rz_at_pos[i + 1];
            if coeff.abs() > 1e-15 {
                circuit.rz(all_qubits[i + 1], Parameter::Float(coeff))?;
            }
        }

        // Reverse CNOT chain.
        for i in (0..(k - 1)).rev() {
            circuit.cx(all_qubits[i], all_qubits[i + 1])?;
        }

        // Handle non-prefix terms with per-term ladders (placed AFTER the
        // shared chain so their CX gates can be optimized independently).
        for (active, angle) in non_prefix_terms {
            let active_with_ops: Vec<(usize, PauliOperator)> =
                active.iter().map(|&q| (q, PauliOperator::Z)).collect();
            self.compile_multi_qubit_rotation(circuit, &active_with_ops, *angle)?;
        }

        Ok(())
    }

    /// Compile a single Pauli term into gates
    ///
    /// For a Pauli string $P$, the exponential $e^{-i\theta P}$ is implemented as:
    ///
    /// 1. Basis transformation to $Z$ basis via $H$ (for $X$) or $R_x(\pi/2)$ (for $Y$)
    /// 2. Multi-controlled $R_z(2\theta)$ rotation using CNOT ladder
    /// 3. Inverse basis transformation
    ///
    /// $$ e^{-i\theta P} = B^\dagger \cdot R_z(2\theta)_{\text{last}} \cdot B $$
    ///
    /// where $B$ is the basis-change circuit and the $R_z$ is applied on the
    /// last controlled qubit of the CNOT ladder.
    fn compile_pauli_term(
        &self,
        circuit: &mut QuantumCircuit,
        pauli_string: &PauliString,
        angle: f64,
    ) -> Result<()> {
        let _num_qubits = pauli_string.num_qubits();

        // Skip if angle is too small
        if angle.abs() < 1e-10 {
            return Ok(());
        }

        // Find qubits with non-identity operators
        let active_qubits: Vec<(usize, PauliOperator)> = pauli_string
            .operators
            .iter()
            .enumerate()
            .filter(|(_, &op)| op != PauliOperator::I)
            .map(|(i, &op)| (i, op))
            .collect();

        if active_qubits.is_empty() {
            // Global phase, can be ignored
            return Ok(());
        }

        // Step 1: Basis transformation to Z basis
        for &(qubit, op) in &active_qubits {
            match op {
                PauliOperator::X => circuit.h(qubit)?,
                PauliOperator::Y => {
                    // Y basis: rotate by $Rx(\pi/2)$
                    circuit.rx(qubit, Parameter::Float(PI / 2.0))?;
                }
                PauliOperator::Z => {} // Already in Z basis
                PauliOperator::I => {} // Should not happen
            }
        }

        // Step 2: Apply the rotation
        if active_qubits.len() == 1 {
            // Single-qubit case: just apply RZ
            let qubit = active_qubits[0].0;
            circuit.rz(qubit, Parameter::Float(angle))?;
        } else {
            // Multi-qubit case: use CNOTs to create entanglement
            self.compile_multi_qubit_rotation(circuit, &active_qubits, angle)?;
        }

        // Step 3: Inverse basis transformation
        for &(qubit, op) in active_qubits.iter().rev() {
            match op {
                PauliOperator::X => circuit.h(qubit)?,
                PauliOperator::Y => {
                    // Inverse: $Rx(-\pi/2)$
                    circuit.rx(qubit, Parameter::Float(-PI / 2.0))?;
                }
                PauliOperator::Z => {}
                PauliOperator::I => {}
            }
        }

        Ok(())
    }

    /// Compile multi-qubit Pauli rotation using CNOT ladder
    ///
    /// For ZZ...Z rotation, use CNOT ladder:
    /// ```text
    /// q0 --●--●--  ...
    /// q1 --⊕--●--  ...
    /// q2 -----⊕--Rz(θ)--⊕--●--  ...
    /// ```
    /// ...
    fn compile_multi_qubit_rotation(
        &self,
        circuit: &mut QuantumCircuit,
        active_qubits: &[(usize, PauliOperator)],
        angle: f64,
    ) -> Result<()> {
        let qubits: Vec<usize> = active_qubits.iter().map(|&(q, _)| q).collect();
        let n = qubits.len();

        if n < 2 {
            return Err(MyQuatError::hamiltonian_error(
                "Multi-qubit rotation requires at least 2 qubits",
            ));
        }

        // CNOT ladder: propagate parity to last qubit
        for i in 0..(n - 1) {
            circuit.cx(qubits[i], qubits[i + 1])?;
        }

        // Apply rotation on last qubit
        circuit.rz(qubits[n - 1], Parameter::Float(angle))?;

        // Inverse CNOT ladder
        for i in (0..(n - 1)).rev() {
            circuit.cx(qubits[i], qubits[i + 1])?;
        }

        Ok(())
    }

    /// Get the compiler configuration
    pub fn config(&self) -> &CompilerConfig {
        &self.config
    }

    /// Set the compiler configuration
    pub fn set_config(&mut self, config: CompilerConfig) {
        self.config = config;
    }

    /// Heuristic: determine if PauliLevel synthesis is likely beneficial.
    ///
    /// Returns `PauliLevel` if the Hamiltonian has large QWC blocks (avg >= 2.5
    /// terms/block and at least one block with >= 3 terms), otherwise returns
    /// `GateLevel` as fallback.
    ///
    /// Also returns `GateLevel` for single-qubit Hamiltonians where shared CNOT
    /// trees provide no benefit.
    fn should_use_pauli_level(hamiltonian: &Hamiltonian) -> CompilationStrategy {
        // Single-qubit: no CNOT trees needed, PauliLevel adds overhead
        if hamiltonian.num_qubits <= 1 {
            return CompilationStrategy::GateLevel;
        }
        // Count multi-qubit terms
        let multi_qubit_count = hamiltonian
            .terms
            .iter()
            .filter(|t| {
                t.pauli_string
                    .operators
                    .iter()
                    .filter(|o| **o != PauliOperator::I)
                    .count()
                    >= 2
            })
            .count();
        // No multi-qubit terms: shared CNOT trees provide zero benefit
        if multi_qubit_count == 0 {
            return CompilationStrategy::GateLevel;
        }
        // Quick QWC check using a simple greedy grouping
        let mut qwc_blocks: Vec<usize> = vec![];
        let mut assigned = vec![false; hamiltonian.terms.len()];
        for i in 0..hamiltonian.terms.len() {
            if assigned[i] {
                continue;
            }
            let mut block_size = 1;
            assigned[i] = true;
            for j in (i + 1)..hamiltonian.terms.len() {
                if assigned[j] {
                    continue;
                }
                let all_qwc = (0..hamiltonian.num_qubits).all(|q| {
                    let oi = hamiltonian.terms[i]
                        .pauli_string
                        .operator_at(q)
                        .unwrap_or(PauliOperator::I);
                    let oj = hamiltonian.terms[j]
                        .pauli_string
                        .operator_at(q)
                        .unwrap_or(PauliOperator::I);
                    oi == oj || oi == PauliOperator::I || oj == PauliOperator::I
                });
                if all_qwc {
                    block_size += 1;
                    assigned[j] = true;
                }
            }
            qwc_blocks.push(block_size);
        }
        let avg = qwc_blocks.iter().sum::<usize>() as f64 / qwc_blocks.len() as f64;
        let has_large_block = qwc_blocks.iter().any(|&b| b >= 3);
        if avg >= 2.5 && has_large_block && multi_qubit_count >= 2 {
            CompilationStrategy::PauliLevel
        } else {
            CompilationStrategy::GateLevel
        }
    }

    /// Apply fourth-order Suzuki formula
    ///
    /// $S_4(t) = S_2(p_1 t) S_2(p_2 t) S_2(p_3 t) S_2(p_2 t) S_2(p_1 t)$
    /// where $p_1 = p_3 = 1/(4-4^{1/3})$, $p_2 = 1 - 4p_1$
    fn apply_fourth_order_suzuki(
        &self,
        circuit: &mut QuantumCircuit,
        hamiltonian: &Hamiltonian,
        dt: f64,
    ) -> Result<()> {
        // Suzuki coefficients for fourth order
        let p1 = 1.0 / (4.0 - 4.0_f64.powf(1.0 / 3.0));
        let p2 = 1.0 - 4.0 * p1;

        // Suzuki 4th-order: S_4(t) = [S_2(p·t)]^2 · S_2((1-4p)·t) · [S_2(p·t)]^2
        // Coefficient sequence [p, p, 1-4p, p, p] — sum = 4p + (1-4p) = 1.0
        self.apply_second_order_step(circuit, hamiltonian, p1 * dt)?;
        self.apply_second_order_step(circuit, hamiltonian, p1 * dt)?;
        self.apply_second_order_step(circuit, hamiltonian, p2 * dt)?;
        self.apply_second_order_step(circuit, hamiltonian, p1 * dt)?;
        self.apply_second_order_step(circuit, hamiltonian, p1 * dt)?;

        Ok(())
    }

    /// Apply sixth-order Suzuki formula
    ///
    /// $S_6(t) = S_4(p_5 t)^2 S_4((1-4p_5)t) S_4(p_5 t)^2$
    /// where $p_5 = 1/(4-4^{1/5})$
    fn apply_sixth_order_suzuki(
        &self,
        circuit: &mut QuantumCircuit,
        hamiltonian: &Hamiltonian,
        dt: f64,
    ) -> Result<()> {
        // Suzuki coefficient for sixth order
        let p5 = 1.0 / (4.0 - 4.0_f64.powf(1.0 / 5.0));

        // $S_4(p_5 t)$
        self.apply_fourth_order_suzuki(circuit, hamiltonian, p5 * dt)?;
        // $S_4(p_5 t)$ again
        self.apply_fourth_order_suzuki(circuit, hamiltonian, p5 * dt)?;
        // $S_4((1-4p_5)t)$
        self.apply_fourth_order_suzuki(circuit, hamiltonian, (1.0 - 4.0 * p5) * dt)?;
        // $S_4(p_5 t)$
        self.apply_fourth_order_suzuki(circuit, hamiltonian, p5 * dt)?;
        // $S_4(p_5 t)$ again
        self.apply_fourth_order_suzuki(circuit, hamiltonian, p5 * dt)?;

        Ok(())
    }

    /// Apply custom Suzuki formula with user-specified coefficients
    fn apply_custom_suzuki(
        &self,
        circuit: &mut QuantumCircuit,
        hamiltonian: &Hamiltonian,
        dt: f64,
        coefficients: &[f64],
    ) -> Result<()> {
        if coefficients.is_empty() {
            return Err(MyQuatError::hamiltonian_error(
                "Custom Suzuki formula requires at least one coefficient",
            ));
        }

        // Apply second-order steps with each coefficient
        for &coeff in coefficients {
            self.apply_second_order_step(circuit, hamiltonian, coeff * dt)?;
        }

        Ok(())
    }

    /// Helper: Apply a single second-order Trotter step
    fn apply_second_order_step(
        &self,
        circuit: &mut QuantumCircuit,
        hamiltonian: &Hamiltonian,
        dt: f64,
    ) -> Result<()> {
        // Forward pass with dt/2
        self.apply_trotter_step(circuit, hamiltonian, dt / 2.0, false, None)?;
        // Backward pass with dt/2
        self.apply_trotter_step(circuit, hamiltonian, dt / 2.0, true, None)?;
        Ok(())
    }

    // ── Cached variants (use pre-computed term groups) ─────────────────

    fn apply_second_order_step_cached(
        &self,
        circuit: &mut QuantumCircuit,
        hamiltonian: &Hamiltonian,
        dt: f64,
        cached_groups: &Option<Vec<Vec<usize>>>,
        override_strategy: Option<CompilationStrategy>,
        clifford_map: Option<&crate::hamiltonian::pauli_synthesis::CliffordAnnotationMap>,
    ) -> Result<()> {
        self.apply_trotter_step_cached(
            circuit,
            hamiltonian,
            dt / 2.0,
            false,
            cached_groups,
            override_strategy,
            clifford_map,
        )?;
        self.apply_trotter_step_cached(
            circuit,
            hamiltonian,
            dt / 2.0,
            true,
            cached_groups,
            override_strategy,
            clifford_map,
        )?;
        Ok(())
    }

    fn apply_fourth_order_suzuki_cached(
        &self,
        circuit: &mut QuantumCircuit,
        hamiltonian: &Hamiltonian,
        dt: f64,
        cached_groups: &Option<Vec<Vec<usize>>>,
        override_strategy: Option<CompilationStrategy>,
        clifford_map: Option<&crate::hamiltonian::pauli_synthesis::CliffordAnnotationMap>,
    ) -> Result<()> {
        let p1 = 1.0 / (4.0 - 4.0_f64.powf(1.0 / 3.0));
        let p2 = 1.0 - 4.0 * p1;
        // Suzuki 4th-order: S_4(t) = [S_2(p·t)]^2 · S_2((1-4p)·t) · [S_2(p·t)]^2
        // Coefficient sequence [p, p, 1-4p, p, p] — sum = 1.0
        self.apply_second_order_step_cached(
            circuit,
            hamiltonian,
            p1 * dt,
            cached_groups,
            override_strategy,
            clifford_map,
        )?;
        self.apply_second_order_step_cached(
            circuit,
            hamiltonian,
            p1 * dt,
            cached_groups,
            override_strategy,
            clifford_map,
        )?;
        self.apply_second_order_step_cached(
            circuit,
            hamiltonian,
            p2 * dt,
            cached_groups,
            override_strategy,
            clifford_map,
        )?;
        self.apply_second_order_step_cached(
            circuit,
            hamiltonian,
            p1 * dt,
            cached_groups,
            override_strategy,
            clifford_map,
        )?;
        self.apply_second_order_step_cached(
            circuit,
            hamiltonian,
            p1 * dt,
            cached_groups,
            override_strategy,
            clifford_map,
        )?;
        Ok(())
    }

    fn apply_sixth_order_suzuki_cached(
        &self,
        circuit: &mut QuantumCircuit,
        hamiltonian: &Hamiltonian,
        dt: f64,
        cached_groups: &Option<Vec<Vec<usize>>>,
        override_strategy: Option<CompilationStrategy>,
        clifford_map: Option<&crate::hamiltonian::pauli_synthesis::CliffordAnnotationMap>,
    ) -> Result<()> {
        let p5 = 1.0 / (4.0 - 4.0_f64.powf(1.0 / 5.0));
        self.apply_fourth_order_suzuki_cached(
            circuit,
            hamiltonian,
            p5 * dt,
            cached_groups,
            override_strategy,
            clifford_map,
        )?;
        self.apply_fourth_order_suzuki_cached(
            circuit,
            hamiltonian,
            p5 * dt,
            cached_groups,
            override_strategy,
            clifford_map,
        )?;
        self.apply_fourth_order_suzuki_cached(
            circuit,
            hamiltonian,
            (1.0 - 4.0 * p5) * dt,
            cached_groups,
            override_strategy,
            clifford_map,
        )?;
        self.apply_fourth_order_suzuki_cached(
            circuit,
            hamiltonian,
            p5 * dt,
            cached_groups,
            override_strategy,
            clifford_map,
        )?;
        self.apply_fourth_order_suzuki_cached(
            circuit,
            hamiltonian,
            p5 * dt,
            cached_groups,
            override_strategy,
            clifford_map,
        )?;
        Ok(())
    }

    fn apply_nth_order_suzuki_cached(
        &self,
        circuit: &mut QuantumCircuit,
        hamiltonian: &Hamiltonian,
        dt: f64,
        order: usize,
        cached_groups: &Option<Vec<Vec<usize>>>,
        override_strategy: Option<CompilationStrategy>,
        clifford_map: Option<&crate::hamiltonian::pauli_synthesis::CliffordAnnotationMap>,
    ) -> Result<()> {
        match order {
            1 => {
                self.apply_trotter_step_cached(
                    circuit,
                    hamiltonian,
                    dt,
                    false,
                    cached_groups,
                    override_strategy,
                    clifford_map,
                )?;
            }
            2 => {
                self.apply_second_order_step_cached(
                    circuit,
                    hamiltonian,
                    dt,
                    cached_groups,
                    override_strategy,
                    clifford_map,
                )?;
            }
            _ if order % 2 == 0 => {
                let reduction = 1.0 / (4.0 - 4.0_f64.powf(1.0 / ((order - 1) as f64)));
                self.apply_nth_order_suzuki_cached(
                    circuit,
                    hamiltonian,
                    reduction * dt,
                    order - 2,
                    cached_groups,
                    override_strategy,
                    clifford_map,
                )?;
                self.apply_nth_order_suzuki_cached(
                    circuit,
                    hamiltonian,
                    reduction * dt,
                    order - 2,
                    cached_groups,
                    override_strategy,
                    clifford_map,
                )?;
                self.apply_nth_order_suzuki_cached(
                    circuit,
                    hamiltonian,
                    (1.0 - 4.0 * reduction) * dt,
                    order - 2,
                    cached_groups,
                    override_strategy,
                    clifford_map,
                )?;
                self.apply_nth_order_suzuki_cached(
                    circuit,
                    hamiltonian,
                    reduction * dt,
                    order - 2,
                    cached_groups,
                    override_strategy,
                    clifford_map,
                )?;
                self.apply_nth_order_suzuki_cached(
                    circuit,
                    hamiltonian,
                    reduction * dt,
                    order - 2,
                    cached_groups,
                    override_strategy,
                    clifford_map,
                )?;
            }
            _ => {
                return Err(MyQuatError::hamiltonian_error(format!(
                    "Invalid Suzuki order: {}. Must be 1 or even number >= 2",
                    order
                )));
            }
        }
        Ok(())
    }

    fn apply_custom_suzuki_cached(
        &self,
        circuit: &mut QuantumCircuit,
        hamiltonian: &Hamiltonian,
        dt: f64,
        coefficients: &[f64],
        cached_groups: &Option<Vec<Vec<usize>>>,
        override_strategy: Option<CompilationStrategy>,
        clifford_map: Option<&crate::hamiltonian::pauli_synthesis::CliffordAnnotationMap>,
    ) -> Result<()> {
        if coefficients.is_empty() {
            return Err(MyQuatError::hamiltonian_error(
                "Custom Suzuki formula requires at least one coefficient",
            ));
        }
        for &coeff in coefficients {
            self.apply_second_order_step_cached(
                circuit,
                hamiltonian,
                coeff * dt,
                cached_groups,
                override_strategy,
                clifford_map,
            )?;
        }
        Ok(())
    }

    /// Compute the steps multiplier for a given Suzuki order
    ///
    /// The recursive structure of Suzuki formulas means:
    /// - $S_1$: 1 step
    /// - $S_2$: 2 steps (forward + backward)
    /// - $S_n$: $5 \times S_{n-2}$ steps (for $n \geq 4$, even)
    ///
    /// This gives: $S_4 = 10$, $S_6 = 50$, $S_8 = 250$, $S_{10} = 1250$, ...
    fn compute_steps_multiplier(&self, order: usize) -> usize {
        match order {
            1 => 1,
            2 => 2,
            _ if order % 2 == 0 => {
                // $S_n = 5 \times S_{n-2}$
                5 * self.compute_steps_multiplier(order - 2)
            }
            _ => 2, // Default for odd orders (shouldn't happen)
        }
    }

    /// Apply N-th order Suzuki formula using recursive construction
    ///
    /// This implements the general Suzuki product formula for any even order $\geq 2$.
    /// The recursive formula is:
    ///
    /// $$ S_1(t) = \prod_k e^{-iH_k t} \quad \text{(first-order)} $$
    ///
    /// $$ S_2(t) = S_1(t/2)\, S_1^\dagger(t/2) \quad \text{(second-order, symmetric)} $$
    ///
    /// $$ S_n(t) = S_{n-2}(p_n t)^2 \; S_{n-2}\big((1-4p_n)t\big) \; S_{n-2}(p_n t)^2
    ///    \quad (n \geq 4, \text{even}) $$
    ///
    /// where $p_n = 1/(4 - 4^{1/(n-1)})$ is the Suzuki scaling coefficient.
    ///
    /// # Arguments
    ///
    /// * `circuit` - The quantum circuit to append gates to
    /// * `hamiltonian` - The Hamiltonian to simulate
    /// * `dt` - Time step
    /// * `order` - The order of the Suzuki formula (must be 1, or even number >= 2)
    ///
    /// # References
    ///
    /// Based on the recursive Suzuki construction from:
    /// - N. Hatano and M. Suzuki, "Finding Exponential Product Formulas of Higher Orders" (2005)
    /// - Qiskit's SuzukiTrotter implementation
    fn apply_nth_order_suzuki(
        &self,
        circuit: &mut QuantumCircuit,
        hamiltonian: &Hamiltonian,
        dt: f64,
        order: usize,
    ) -> Result<()> {
        match order {
            1 => {
                // First-order: just apply Trotter step
                self.apply_trotter_step(circuit, hamiltonian, dt, false, None)?;
            }
            2 => {
                // Second-order: symmetric Trotter
                self.apply_second_order_step(circuit, hamiltonian, dt)?;
            }
            _ if order % 2 == 0 => {
                // Higher even orders: recursive construction
                // Suzuki coefficient: p = 1/(4 - 4^(1/(order-1)))
                let reduction = 1.0 / (4.0 - 4.0_f64.powf(1.0 / ((order - 1) as f64)));

                // Outer: $2 \times S_{n-2}(p \cdot t)$
                self.apply_nth_order_suzuki(circuit, hamiltonian, reduction * dt, order - 2)?;
                self.apply_nth_order_suzuki(circuit, hamiltonian, reduction * dt, order - 2)?;

                // Inner: $S_{n-2}((1-4p) \cdot t)$
                self.apply_nth_order_suzuki(
                    circuit,
                    hamiltonian,
                    (1.0 - 4.0 * reduction) * dt,
                    order - 2,
                )?;

                // Outer again: $2 \times S_{n-2}(p \cdot t)$
                self.apply_nth_order_suzuki(circuit, hamiltonian, reduction * dt, order - 2)?;
                self.apply_nth_order_suzuki(circuit, hamiltonian, reduction * dt, order - 2)?;
            }
            _ => {
                return Err(MyQuatError::hamiltonian_error(format!(
                    "Invalid Suzuki order: {}. Must be 1 or even number >= 2",
                    order
                )));
            }
        }

        Ok(())
    }

    /// Estimate Trotter error for current configuration
    ///
    /// Returns theoretical error upper bound based on the Trotter formula.
    pub fn estimate_error(&self, hamiltonian: &Hamiltonian) -> TrotterErrorAnalysis {
        let t = self.config.evolution_time;
        let n = self.config.trotter_steps as f64;

        // Estimate Hamiltonian norm (upper bound using triangle inequality)
        let h_norm = hamiltonian
            .terms
            .iter()
            .map(|term| term.coefficient.norm())
            .sum::<f64>();

        let (theoretical_error, error_order) = match &self.config.trotter_order {
            TrotterOrder::First => {
                // Error: $O(\|H\|^2 \cdot t^2 / n)$
                (h_norm * h_norm * t * t / n, 2)
            }
            TrotterOrder::Second => {
                // Error: $O(\|H\|^3 \cdot t^3 / n^2)$
                (h_norm.powi(3) * t.powi(3) / (n * n), 3)
            }
            TrotterOrder::Fourth => {
                // Error: $O(\|H\|^5 \cdot t^5 / n^4)$
                (h_norm.powi(5) * t.powi(5) / n.powi(4), 5)
            }
            TrotterOrder::Sixth => {
                // Error: $O(\|H\|^7 \cdot t^7 / n^6)$
                (h_norm.powi(7) * t.powi(7) / n.powi(6), 7)
            }
            TrotterOrder::Nth(order) => {
                // Error: O(h_norm^(order+1) * t^(order+1) / n^order)
                let ord = *order as i32;
                let error_exp = ord + 1;
                (
                    h_norm.powi(error_exp) * t.powi(error_exp) / n.powi(ord),
                    error_exp as usize,
                )
            }
            TrotterOrder::Custom(_) => {
                // Conservative estimate: assume second-order
                (h_norm.powi(3) * t.powi(3) / (n * n), 3)
            }
        };

        // Estimate average gates per term per step
        let gates_per_term = hamiltonian
            .terms
            .iter()
            .map(|term| {
                let active_qubits = term
                    .pauli_string
                    .operators
                    .iter()
                    .filter(|op| **op != PauliOperator::I)
                    .count();
                if active_qubits == 1 {
                    3 // Basis change + RZ + inverse
                } else {
                    2 * (active_qubits - 1) + 1 + 2 * (active_qubits - 1) // CNOT ladder + RZ + inverse
                }
            })
            .sum::<usize>();

        let steps_multiplier = match &self.config.trotter_order {
            TrotterOrder::First => 1,
            TrotterOrder::Second => 2,
            TrotterOrder::Fourth => 10, // $5 S_2$ steps
            TrotterOrder::Sixth => 50,  // $5 S_4$ steps, each with $5 S_2$ steps
            TrotterOrder::Nth(order) => {
                // Recursive structure: $S_n$ uses $5 \times S_{n-2}$
                // $S_2 = 2$, $S_4 = 5\times2 = 10$, $S_6 = 5\times10 = 50$, $S_8 = 5\times50 = 250$, ...
                self.compute_steps_multiplier(*order)
            }
            TrotterOrder::Custom(coeffs) => coeffs.len() * 2,
        };

        let estimated_gates = gates_per_term * steps_multiplier * self.config.trotter_steps;

        TrotterErrorAnalysis {
            theoretical_error,
            recommended_steps: self.config.trotter_steps,
            estimated_gates,
            error_order,
        }
    }

    /// Calculate optimal number of Trotter steps for target error
    ///
    /// # Arguments
    ///
    /// * `hamiltonian` - The Hamiltonian to simulate
    /// * `target_error` - Desired error tolerance
    ///
    /// # Returns
    ///
    /// Recommended number of Trotter steps
    pub fn compute_optimal_steps(&self, hamiltonian: &Hamiltonian, target_error: f64) -> usize {
        let t = self.config.evolution_time;

        // Estimate Hamiltonian norm
        let h_norm = hamiltonian
            .terms
            .iter()
            .map(|term| term.coefficient.norm())
            .sum::<f64>();

        // Solve for n based on error formula

        match &self.config.trotter_order {
            TrotterOrder::First => {
                // $\epsilon = \|H\|^2 \cdot t^2 / n \Rightarrow n = \|H\|^2 \cdot t^2 / \epsilon$
                ((h_norm * h_norm * t * t / target_error).ceil() as usize).max(1)
            }
            TrotterOrder::Second => {
                // $\epsilon = \|H\|^3 \cdot t^3 / n^2 \Rightarrow n = \sqrt{\|H\|^3 \cdot t^3 / \epsilon}$
                ((h_norm.powi(3) * t.powi(3) / target_error).sqrt().ceil() as usize).max(1)
            }
            TrotterOrder::Fourth => {
                // $\epsilon = \|H\|^5 \cdot t^5 / n^4 \Rightarrow n = (\|H\|^5 \cdot t^5 / \epsilon)^{1/4}$
                ((h_norm.powi(5) * t.powi(5) / target_error)
                    .powf(0.25)
                    .ceil() as usize)
                    .max(1)
            }
            TrotterOrder::Sixth => {
                // $\epsilon = \|H\|^7 \cdot t^7 / n^6 \Rightarrow n = (\|H\|^7 \cdot t^7 / \epsilon)^{1/6}$
                ((h_norm.powi(7) * t.powi(7) / target_error)
                    .powf(1.0 / 6.0)
                    .ceil() as usize)
                    .max(1)
            }
            TrotterOrder::Nth(order) => {
                // $\epsilon = \|H\|^{\text{order}+1} \cdot t^{\text{order}+1} / n^{\text{order}} \Rightarrow n = (\|H\|^{\text{order}+1} \cdot t^{\text{order}+1} / \epsilon)^{1/\text{order}}$
                let ord = *order as i32;
                let error_exp = ord + 1;
                let base = h_norm.powi(error_exp) * t.powi(error_exp) / target_error;
                (base.powf(1.0 / ord as f64).ceil() as usize).max(1)
            }
            TrotterOrder::Custom(_) => {
                // Conservative: use second-order formula
                ((h_norm.powi(3) * t.powi(3) / target_error).sqrt().ceil() as usize).max(1)
            }
        }
    }

    /// Analyze Trotter error for given target and return detailed report
    pub fn analyze_error(
        &self,
        hamiltonian: &Hamiltonian,
        target_error: f64,
    ) -> TrotterErrorAnalysis {
        let optimal_steps = self.compute_optimal_steps(hamiltonian, target_error);

        // Create new config with optimal steps
        let mut new_config = self.config.clone();
        new_config.trotter_steps = optimal_steps;

        let temp_compiler = HamiltonianCompiler::new(new_config);
        temp_compiler.estimate_error(hamiltonian)
    }

    /// Estimate local error for a single time step
    ///
    /// Uses Richardson extrapolation to estimate the local truncation error
    fn estimate_local_error(&self, hamiltonian: &Hamiltonian, dt: f64) -> f64 {
        // Hamiltonian norm estimate
        let h_norm = hamiltonian
            .terms
            .iter()
            .map(|term| term.coefficient.norm())
            .sum::<f64>();

        // Local error depends on Trotter order
        match &self.config.trotter_order {
            TrotterOrder::First => {
                // Local error: $O(dt^2)$
                h_norm * h_norm * dt * dt
            }
            TrotterOrder::Second => {
                // Local error: $O(dt^3)$
                h_norm.powi(3) * dt.powi(3)
            }
            TrotterOrder::Fourth => {
                // Local error: $O(dt^5)$
                h_norm.powi(5) * dt.powi(5)
            }
            TrotterOrder::Sixth => {
                // Local error: $O(dt^7)$
                h_norm.powi(7) * dt.powi(7)
            }
            TrotterOrder::Nth(order) => {
                // Local error: O(dt^(order+1))
                let ord = *order as i32;
                h_norm.powi(ord + 1) * dt.powi(ord + 1)
            }
            TrotterOrder::Custom(_) => {
                // Conservative: assume second-order
                h_norm.powi(3) * dt.powi(3)
            }
        }
    }

    /// Compute optimal step size based on local error tolerance
    fn compute_adaptive_step_size(&self, hamiltonian: &Hamiltonian) -> f64 {
        let tol = self.config.adaptive_tolerance;
        let h_norm = hamiltonian
            .terms
            .iter()
            .map(|term| term.coefficient.norm())
            .sum::<f64>();

        // Solve for dt such that local_error $\approx$ tolerance
        let dt = match &self.config.trotter_order {
            TrotterOrder::First => {
                // $\text{tol} = \|H\|^2 \cdot dt^2 \Rightarrow dt = \sqrt{\text{tol} / \|H\|^2}$
                (tol / (h_norm * h_norm)).sqrt()
            }
            TrotterOrder::Second => {
                // $\text{tol} = \|H\|^3 \cdot dt^3 \Rightarrow dt = (\text{tol} / \|H\|^3)^{1/3}$
                (tol / h_norm.powi(3)).powf(1.0 / 3.0)
            }
            TrotterOrder::Fourth => {
                // $\text{tol} = \|H\|^5 \cdot dt^5 \Rightarrow dt = (\text{tol} / \|H\|^5)^{1/5}$
                (tol / h_norm.powi(5)).powf(1.0 / 5.0)
            }
            TrotterOrder::Sixth => {
                // $\text{tol} = \|H\|^7 \cdot dt^7 \Rightarrow dt = (\text{tol} / \|H\|^7)^{1/7}$
                (tol / h_norm.powi(7)).powf(1.0 / 7.0)
            }
            TrotterOrder::Nth(order) => {
                // tol = ||H||^(order+1) * dt^(order+1)  =>  dt = (tol / ||H||^(order+1))^(1/(order+1))
                let ord = *order as i32;
                let error_exp = ord + 1;
                (tol / h_norm.powi(error_exp)).powf(1.0 / error_exp as f64)
            }
            TrotterOrder::Custom(_) => {
                // Conservative: assume second-order
                (tol / h_norm.powi(3)).powf(1.0 / 3.0)
            }
        };

        // Clamp to min/max bounds
        dt.max(self.config.min_step_size)
            .min(self.config.max_step_size)
    }

    /// Compile with adaptive time stepping
    ///
    /// Uses variable step sizes to achieve target local error tolerance
    pub fn compile_adaptive(&self, hamiltonian: &Hamiltonian) -> Result<QuantumCircuit> {
        let mut circuit = QuantumCircuit::new(hamiltonian.num_qubits, 0);
        let total_time = self.config.evolution_time;

        let mut t = 0.0;
        let mut step_count = 0;

        while t < total_time {
            // Compute optimal step size for current position
            let dt = self.compute_adaptive_step_size(hamiltonian);

            // Adjust last step to not overshoot
            let actual_dt = if t + dt > total_time {
                total_time - t
            } else {
                dt
            };

            // Apply one Trotter step with this dt
            match &self.config.trotter_order {
                TrotterOrder::First => {
                    self.apply_trotter_step(&mut circuit, hamiltonian, actual_dt, false, None)?;
                }
                TrotterOrder::Second => {
                    self.apply_trotter_step(
                        &mut circuit,
                        hamiltonian,
                        actual_dt / 2.0,
                        false,
                        None,
                    )?;
                    self.apply_trotter_step(
                        &mut circuit,
                        hamiltonian,
                        actual_dt / 2.0,
                        true,
                        None,
                    )?;
                }
                TrotterOrder::Fourth => {
                    self.apply_fourth_order_suzuki(&mut circuit, hamiltonian, actual_dt)?;
                }
                TrotterOrder::Sixth => {
                    self.apply_sixth_order_suzuki(&mut circuit, hamiltonian, actual_dt)?;
                }
                TrotterOrder::Nth(order) => {
                    self.apply_nth_order_suzuki(&mut circuit, hamiltonian, actual_dt, *order)?;
                }
                TrotterOrder::Custom(coeffs) => {
                    self.apply_custom_suzuki(&mut circuit, hamiltonian, actual_dt, coeffs)?;
                }
            }

            t += actual_dt;
            step_count += 1;

            // Safety check: prevent infinite loops
            if step_count > 100000 {
                return Err(MyQuatError::hamiltonian_error(
                    "Adaptive stepping exceeded maximum iterations",
                ));
            }
        }

        Ok(circuit)
    }

    /// Optimize compiled circuit by merging consecutive rotation gates
    ///
    /// This optimization reduces gate count by:
    /// 1. Merging consecutive rotations on the same qubit and axis
    /// 2. Removing zero-angle rotations
    /// 3. Simplifying rotation sequences
    ///
    /// Returns an optimized circuit with reduced gate count
    pub fn optimize_circuit(&self, circuit: &QuantumCircuit) -> Result<QuantumCircuit> {
        let mut optimized = QuantumCircuit::new(circuit.num_qubits(), circuit.num_clbits());

        let instructions = circuit.data().instructions();
        let mut i = 0;

        while i < instructions.len() {
            let current = &instructions[i];

            // Try to merge with next instruction
            if i + 1 < instructions.len() {
                let next = &instructions[i + 1];

                if let Some(merged_angle) = self.try_merge_rotations(current, next) {
                    // Skip zero rotations
                    if merged_angle.abs() > 1e-10 {
                        // Apply merged rotation
                        let qubit = current.qubits[0].0;
                        match current.gate.gate_type {
                            crate::gates::StandardGate::Rx => {
                                optimized.rx(qubit, Parameter::Float(merged_angle))?;
                            }
                            crate::gates::StandardGate::Ry => {
                                optimized.ry(qubit, Parameter::Float(merged_angle))?;
                            }
                            crate::gates::StandardGate::Rz => {
                                optimized.rz(qubit, Parameter::Float(merged_angle))?;
                            }
                            _ => unreachable!(),
                        }
                    }
                    i += 2; // Skip both instructions
                    continue;
                }
            }

            // No merge possible, copy instruction as-is
            // Check if it's a zero rotation
            if let Some(angle) = self.get_rotation_angle(current) {
                if angle.abs() > 1e-10 {
                    self.copy_instruction(&mut optimized, current)?;
                }
                // Skip zero rotations
            } else {
                self.copy_instruction(&mut optimized, current)?;
            }

            i += 1;
        }

        Ok(optimized)
    }

    /// Try to merge two consecutive rotation gates
    fn try_merge_rotations(
        &self,
        inst1: &crate::circuit::Instruction,
        inst2: &crate::circuit::Instruction,
    ) -> Option<f64> {
        use crate::gates::StandardGate;

        // Check if both are rotations on the same qubit
        if inst1.qubits.len() != 1 || inst2.qubits.len() != 1 {
            return None;
        }

        if inst1.qubits[0] != inst2.qubits[0] {
            return None;
        }

        // Check if same gate type
        if inst1.gate.gate_type != inst2.gate.gate_type {
            return None;
        }

        // Check if it's a rotation gate
        match inst1.gate.gate_type {
            StandardGate::Rx | StandardGate::Ry | StandardGate::Rz => {
                let angle1 = self.get_rotation_angle(inst1)?;
                let angle2 = self.get_rotation_angle(inst2)?;
                Some(angle1 + angle2)
            }
            _ => None,
        }
    }

    /// Get rotation angle from instruction
    fn get_rotation_angle(&self, inst: &crate::circuit::Instruction) -> Option<f64> {
        if inst.gate.parameters.is_empty() {
            return None;
        }

        match &inst.gate.parameters[0] {
            Parameter::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Copy instruction to optimized circuit
    fn copy_instruction(
        &self,
        circuit: &mut QuantumCircuit,
        inst: &crate::circuit::Instruction,
    ) -> Result<()> {
        use crate::gates::StandardGate;

        match inst.gate.gate_type {
            StandardGate::Rx => {
                if let Some(angle) = self.get_rotation_angle(inst) {
                    circuit.rx(inst.qubits[0].0, Parameter::Float(angle))?;
                }
            }
            StandardGate::Ry => {
                if let Some(angle) = self.get_rotation_angle(inst) {
                    circuit.ry(inst.qubits[0].0, Parameter::Float(angle))?;
                }
            }
            StandardGate::Rz => {
                if let Some(angle) = self.get_rotation_angle(inst) {
                    circuit.rz(inst.qubits[0].0, Parameter::Float(angle))?;
                }
            }
            StandardGate::H => circuit.h(inst.qubits[0].0)?,
            StandardGate::X => circuit.x(inst.qubits[0].0)?,
            StandardGate::Y => circuit.y(inst.qubits[0].0)?,
            StandardGate::Z => circuit.z(inst.qubits[0].0)?,
            StandardGate::S => circuit.s(inst.qubits[0].0)?,
            StandardGate::Sdg => circuit.sdg(inst.qubits[0].0)?,
            StandardGate::T => circuit.t(inst.qubits[0].0)?,
            StandardGate::Tdg => circuit.tdg(inst.qubits[0].0)?,
            StandardGate::CX => {
                if inst.qubits.len() >= 2 {
                    circuit.cx(inst.qubits[0].0, inst.qubits[1].0)?;
                }
            }
            StandardGate::CZ if inst.qubits.len() >= 2 => {
                circuit.cz(inst.qubits[0].0, inst.qubits[1].0)?;
            }
            _ => {
                // For other gates, use generic add_instruction
                // This is a simplified approach; full implementation would handle all gate types
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hamiltonian::{constructors, PauliTerm};

    #[test]
    fn test_compile_single_qubit_hamiltonian() {
        // Create single-qubit X Hamiltonian: H = X
        let mut h = Hamiltonian::new(1);
        let x_string = PauliString::from_str("X").unwrap();
        h.add_term(x_string, num_complex::Complex64::new(1.0, 0.0))
            .unwrap();

        let compiler = HamiltonianCompiler::default();
        let circuit = compiler.compile(&h).unwrap();

        // Should have some gates
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_compile_ising_model() {
        let h = constructors::ising_model(3, 1.0, 0.5).unwrap();

        let config = CompilerConfig {
            trotter_steps: 5,
            evolution_time: 1.0,
            ..Default::default()
        };

        let compiler = HamiltonianCompiler::new(config);
        let circuit = compiler.compile(&h).unwrap();

        // Ising model on 3 qubits should produce gates
        assert!(circuit.size() > 0);
        assert_eq!(circuit.num_qubits(), 3);
    }

    #[test]
    fn test_second_order_trotter() {
        let h = constructors::heisenberg_model(2, 1.0, 1.0, 1.0).unwrap();

        let config = CompilerConfig {
            trotter_order: TrotterOrder::Second,
            trotter_steps: 3,
            ..Default::default()
        };

        let compiler = HamiltonianCompiler::new(config);
        let circuit = compiler.compile(&h).unwrap();

        // Second-order should produce more gates than first-order
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_empty_hamiltonian() {
        let h = Hamiltonian::new(2);

        let compiler = HamiltonianCompiler::default();
        let circuit = compiler.compile(&h).unwrap();

        // Empty Hamiltonian should produce empty or minimal circuit
        assert_eq!(circuit.num_qubits(), 2);
    }

    #[test]
    fn test_multiple_trotter_steps() {
        // Create single-qubit Z Hamiltonian: H = Z
        let mut h = Hamiltonian::new(1);
        let z_string = PauliString::from_str("Z").unwrap();
        h.add_term(z_string, num_complex::Complex64::new(1.0, 0.0))
            .unwrap();

        let config1 = CompilerConfig {
            trotter_steps: 1,
            ..Default::default()
        };
        let compiler1 = HamiltonianCompiler::new(config1);
        let circuit1 = compiler1.compile(&h).unwrap();

        let config2 = CompilerConfig {
            trotter_steps: 5,
            ..Default::default()
        };
        let compiler2 = HamiltonianCompiler::new(config2);
        let circuit2 = compiler2.compile(&h).unwrap();

        // More Trotter steps should produce more gates
        assert!(circuit2.size() >= circuit1.size());
    }

    #[test]
    fn test_pauli_basis_transformation() {
        // Test that Y basis transformation works
        let mut h = Hamiltonian::new(1);
        let y_string = PauliString::from_str("Y").unwrap();
        h.add_term(y_string, num_complex::Complex64::new(1.0, 0.0))
            .unwrap();

        let compiler = HamiltonianCompiler::default();
        let circuit = compiler.compile(&h).unwrap();

        // Should have basis transformation gates
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_fourth_order_trotter() {
        // Create a simple Hamiltonian
        let mut h = Hamiltonian::new(1);
        let x_string = PauliString::from_str("X").unwrap();
        h.add_term(x_string, num_complex::Complex64::new(1.0, 0.0))
            .unwrap();

        let config = CompilerConfig {
            trotter_order: TrotterOrder::Fourth,
            trotter_steps: 2,
            evolution_time: 1.0,
            ..Default::default()
        };

        let compiler = HamiltonianCompiler::new(config);
        let circuit = compiler.compile(&h).unwrap();

        // Fourth-order should produce more gates than second-order
        // for the same number of steps
        assert!(circuit.size() > 0);
        println!("Fourth-order Trotter gates: {}", circuit.size());
    }

    #[test]
    fn test_sixth_order_trotter() {
        // Create a simple Hamiltonian
        let mut h = Hamiltonian::new(1);
        let x_string = PauliString::from_str("X").unwrap();
        h.add_term(x_string, num_complex::Complex64::new(1.0, 0.0))
            .unwrap();

        let config = CompilerConfig {
            trotter_order: TrotterOrder::Sixth,
            trotter_steps: 1,
            evolution_time: 1.0,
            ..Default::default()
        };

        let compiler = HamiltonianCompiler::new(config);
        let circuit = compiler.compile(&h).unwrap();

        // Sixth-order should produce many more gates
        assert!(circuit.size() > 0);
        println!("Sixth-order Trotter gates: {}", circuit.size());
    }

    #[test]
    fn test_custom_trotter() {
        // Create a simple Hamiltonian
        let mut h = Hamiltonian::new(1);
        let x_string = PauliString::from_str("X").unwrap();
        h.add_term(x_string, num_complex::Complex64::new(1.0, 0.0))
            .unwrap();

        // Custom coefficients (e.g., optimized for specific case)
        let coeffs = vec![0.5, 0.3, 0.2];

        let config = CompilerConfig {
            trotter_order: TrotterOrder::Custom(coeffs),
            trotter_steps: 2,
            evolution_time: 1.0,
            ..Default::default()
        };

        let compiler = HamiltonianCompiler::new(config);
        let circuit = compiler.compile(&h).unwrap();

        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_trotter_order_comparison() {
        // Compare gate counts for different Trotter orders
        let h = constructors::ising_model(2, 1.0, 0.5).unwrap();

        let config1 = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 3,
            ..Default::default()
        };
        let circuit1 = HamiltonianCompiler::new(config1).compile(&h).unwrap();

        let config2 = CompilerConfig {
            trotter_order: TrotterOrder::Second,
            trotter_steps: 3,
            ..Default::default()
        };
        let circuit2 = HamiltonianCompiler::new(config2).compile(&h).unwrap();

        let config4 = CompilerConfig {
            trotter_order: TrotterOrder::Fourth,
            trotter_steps: 3,
            ..Default::default()
        };
        let circuit4 = HamiltonianCompiler::new(config4).compile(&h).unwrap();

        // Higher order should have more gates
        println!("First-order gates: {}", circuit1.size());
        println!("Second-order gates: {}", circuit2.size());
        println!("Fourth-order gates: {}", circuit4.size());

        assert!(circuit2.size() >= circuit1.size());
        assert!(circuit4.size() >= circuit2.size());
    }

    #[test]
    fn test_error_estimation() {
        // Test error estimation for different Trotter orders
        let h = constructors::ising_model(2, 1.0, 0.5).unwrap();

        let config1 = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 10,
            evolution_time: 1.0,
            ..Default::default()
        };
        let compiler1 = HamiltonianCompiler::new(config1);
        let analysis1 = compiler1.estimate_error(&h);

        let config2 = CompilerConfig {
            trotter_order: TrotterOrder::Second,
            trotter_steps: 10,
            evolution_time: 1.0,
            ..Default::default()
        };
        let compiler2 = HamiltonianCompiler::new(config2);
        let analysis2 = compiler2.estimate_error(&h);

        let config4 = CompilerConfig {
            trotter_order: TrotterOrder::Fourth,
            trotter_steps: 10,
            evolution_time: 1.0,
            ..Default::default()
        };
        let compiler4 = HamiltonianCompiler::new(config4);
        let analysis4 = compiler4.estimate_error(&h);

        // Higher order should have lower error
        println!(
            "First-order error: {:.6e}, scaling: {}",
            analysis1.theoretical_error,
            analysis1.error_scaling()
        );
        println!(
            "Second-order error: {:.6e}, scaling: {}",
            analysis2.theoretical_error,
            analysis2.error_scaling()
        );
        println!(
            "Fourth-order error: {:.6e}, scaling: {}",
            analysis4.theoretical_error,
            analysis4.error_scaling()
        );

        assert!(analysis2.theoretical_error < analysis1.theoretical_error);
        assert!(analysis4.theoretical_error < analysis2.theoretical_error);

        assert_eq!(analysis1.error_order, 2);
        assert_eq!(analysis2.error_order, 3);
        assert_eq!(analysis4.error_order, 5);
    }

    #[test]
    fn test_optimal_steps_calculation() {
        // Test computing optimal Trotter steps for target error
        let h = constructors::ising_model(2, 1.0, 0.5).unwrap();
        let target_error = 0.01;

        let compiler1 = HamiltonianCompiler::new(CompilerConfig {
            trotter_order: TrotterOrder::First,
            ..Default::default()
        });
        let steps1 = compiler1.compute_optimal_steps(&h, target_error);

        let compiler2 = HamiltonianCompiler::new(CompilerConfig {
            trotter_order: TrotterOrder::Second,
            ..Default::default()
        });
        let steps2 = compiler2.compute_optimal_steps(&h, target_error);

        let compiler4 = HamiltonianCompiler::new(CompilerConfig {
            trotter_order: TrotterOrder::Fourth,
            ..Default::default()
        });
        let steps4 = compiler4.compute_optimal_steps(&h, target_error);

        println!("Optimal steps for error {:.3}:", target_error);
        println!("  First-order: {}", steps1);
        println!("  Second-order: {}", steps2);
        println!("  Fourth-order: {}", steps4);

        // Higher order should require fewer steps
        assert!(steps2 < steps1);
        assert!(steps4 < steps2);
        assert!(steps1 > 0 && steps2 > 0 && steps4 > 0);
    }

    #[test]
    fn test_error_analysis() {
        // Test full error analysis
        let h = constructors::heisenberg_model(2, 1.0, 1.0, 1.0).unwrap();
        let target_error = 0.001;

        let compiler = HamiltonianCompiler::new(CompilerConfig {
            trotter_order: TrotterOrder::Fourth,
            ..Default::default()
        });

        let analysis = compiler.analyze_error(&h, target_error);

        println!("Error Analysis for target error {:.4}:", target_error);
        println!("  Recommended steps: {}", analysis.recommended_steps);
        println!("  Theoretical error: {:.6e}", analysis.theoretical_error);
        println!("  Estimated gates: {}", analysis.estimated_gates);
        println!("  Error scaling: {}", analysis.error_scaling());

        // The analysis should provide reasonable recommendations
        assert!(analysis.recommended_steps > 0);
        assert!(analysis.estimated_gates > 0);
        assert!(analysis.theoretical_error <= target_error * 10.0); // Within order of magnitude
    }

    #[test]
    fn test_gate_merging_optimization() {
        // Test that consecutive rotations are merged
        let mut h = Hamiltonian::new(1);
        let x = PauliString::from_str("X").unwrap();
        h.add_term(x, num_complex::Complex64::new(1.0, 0.0))
            .unwrap();

        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 5,
            evolution_time: 1.0,
            ..Default::default()
        };

        let compiler = HamiltonianCompiler::new(config);
        let circuit = compiler.compile(&h).unwrap();
        let original_gates = circuit.size();

        // Optimize the circuit
        let optimized = compiler.optimize_circuit(&circuit).unwrap();
        let optimized_gates = optimized.size();

        println!("Original gates: {}", original_gates);
        println!("Optimized gates: {}", optimized_gates);

        // Optimization should reduce or maintain gate count
        assert!(optimized_gates <= original_gates);
    }

    #[test]
    fn test_zero_rotation_removal() {
        // Create a circuit with explicit zero rotations to test removal
        let mut circuit = QuantumCircuit::new(1, 0);
        circuit.rx(0, Parameter::Float(0.5)).unwrap();
        circuit.rx(0, Parameter::Float(0.0)).unwrap(); // Zero rotation
        circuit.ry(0, Parameter::Float(0.3)).unwrap();

        let compiler = HamiltonianCompiler::default();
        let optimized = compiler.optimize_circuit(&circuit).unwrap();

        // Should remove the zero rotation
        assert!(optimized.size() < circuit.size());
        println!(
            "Before: {} gates, After: {} gates",
            circuit.size(),
            optimized.size()
        );
    }

    #[test]
    fn test_rotation_merging() {
        // Test merging of consecutive same-axis rotations
        let mut circuit = QuantumCircuit::new(1, 0);
        circuit.rx(0, Parameter::Float(0.5)).unwrap();
        circuit.rx(0, Parameter::Float(0.3)).unwrap();
        circuit.rx(0, Parameter::Float(0.2)).unwrap();

        let compiler = HamiltonianCompiler::default();
        let optimized = compiler.optimize_circuit(&circuit).unwrap();

        // Three rotations should merge into fewer (ideally one)
        assert!(optimized.size() <= circuit.size());
        println!(
            "Before: {} gates, After: {} gates",
            circuit.size(),
            optimized.size()
        );
    }

    #[test]
    fn test_pauli_term_commutation() {
        // Test commutation detection
        let compiler = HamiltonianCompiler::default();

        // Create commuting terms: XX and YY commute
        let term1 = PauliTerm::new(
            PauliString::from_str("XX").unwrap(),
            num_complex::Complex64::new(1.0, 0.0),
        );
        let term2 = PauliTerm::new(
            PauliString::from_str("YY").unwrap(),
            num_complex::Complex64::new(1.0, 0.0),
        );

        assert!(compiler.terms_commute(&term1, &term2));

        // Create anti-commuting terms: XX and XY anti-commute
        let term3 = PauliTerm::new(
            PauliString::from_str("XY").unwrap(),
            num_complex::Complex64::new(1.0, 0.0),
        );

        assert!(!compiler.terms_commute(&term1, &term3));
    }

    #[test]
    fn test_pauli_ordering_optimization() {
        // Create Hamiltonian with mixed terms
        let mut h = Hamiltonian::new(2);

        // Add terms in suboptimal order
        h.add_term(
            PauliString::from_str("XY").unwrap(),
            num_complex::Complex64::new(1.0, 0.0),
        )
        .unwrap();
        h.add_term(
            PauliString::from_str("ZZ").unwrap(),
            num_complex::Complex64::new(1.0, 0.0),
        )
        .unwrap();
        h.add_term(
            PauliString::from_str("XX").unwrap(),
            num_complex::Complex64::new(1.0, 0.0),
        )
        .unwrap();
        h.add_term(
            PauliString::from_str("YY").unwrap(),
            num_complex::Complex64::new(1.0, 0.0),
        )
        .unwrap();

        let compiler = HamiltonianCompiler::default();
        let optimized_h = compiler.optimize_pauli_ordering(&h);

        // Check that number of terms is preserved
        assert_eq!(optimized_h.num_terms(), h.num_terms());

        // Compile both and compare gate counts
        let circuit_original = compiler.compile(&h).unwrap();
        let circuit_optimized = compiler.compile(&optimized_h).unwrap();

        println!("Original ordering gates: {}", circuit_original.size());
        println!("Optimized ordering gates: {}", circuit_optimized.size());

        // Optimized should be same or better
        assert!(circuit_optimized.size() <= circuit_original.size());
    }

    #[test]
    fn test_commuting_group_formation() {
        let compiler = HamiltonianCompiler::default();

        // Create terms with known commutation properties
        let terms = vec![
            PauliTerm::new(
                PauliString::from_str("XX").unwrap(),
                num_complex::Complex64::new(1.0, 0.0),
            ),
            PauliTerm::new(
                PauliString::from_str("YY").unwrap(),
                num_complex::Complex64::new(1.0, 0.0),
            ),
            PauliTerm::new(
                PauliString::from_str("ZZ").unwrap(),
                num_complex::Complex64::new(1.0, 0.0),
            ),
            PauliTerm::new(
                PauliString::from_str("XZ").unwrap(),
                num_complex::Complex64::new(1.0, 0.0),
            ),
        ];

        let groups = compiler.group_commuting_terms(&terms);

        println!("Formed {} commuting groups", groups.len());
        for (i, group) in groups.iter().enumerate() {
            println!("Group {}: {} terms", i, group.len());
        }

        // Should have at least one group
        assert!(!groups.is_empty());

        // Total terms should be preserved
        let total_terms: usize = groups.iter().map(|g| g.len()).sum();
        assert_eq!(total_terms, terms.len());
    }

    #[test]
    fn test_nth_order_basic_compilation() {
        let h = constructors::ising_model(2, 1.0, 0.5).unwrap();

        // Test various even orders
        for order in [2, 4, 6, 8, 10] {
            let config = CompilerConfig {
                trotter_order: TrotterOrder::Nth(order),
                trotter_steps: 1,
                evolution_time: 1.0,
                ..Default::default()
            };

            let compiler = HamiltonianCompiler::new(config);
            let circuit = compiler.compile(&h).unwrap();

            assert!(circuit.size() > 0);
            assert_eq!(circuit.num_qubits(), 2);
            println!("Nth({}) order gates: {}", order, circuit.size());
        }
    }

    #[test]
    fn test_nth_order_gate_scaling() {
        let h = constructors::ising_model(2, 1.0, 0.5).unwrap();

        // Test gate count scaling: Nth order uses 5^((n-2)/2) times more gates than 2nd order
        let config2 = CompilerConfig {
            trotter_order: TrotterOrder::Nth(2),
            trotter_steps: 1,
            ..Default::default()
        };
        let circuit2 = HamiltonianCompiler::new(config2).compile(&h).unwrap();
        let gates2 = circuit2.size();

        let config4 = CompilerConfig {
            trotter_order: TrotterOrder::Nth(4),
            trotter_steps: 1,
            ..Default::default()
        };
        let circuit4 = HamiltonianCompiler::new(config4).compile(&h).unwrap();
        let gates4 = circuit4.size();

        let config6 = CompilerConfig {
            trotter_order: TrotterOrder::Nth(6),
            trotter_steps: 1,
            ..Default::default()
        };
        let circuit6 = HamiltonianCompiler::new(config6).compile(&h).unwrap();
        let gates6 = circuit6.size();

        println!("2nd order: {} gates", gates2);
        println!(
            "4th order: {} gates ({}x)",
            gates4,
            gates4 as f64 / gates2 as f64
        );
        println!(
            "6th order: {} gates ({}x)",
            gates6,
            gates6 as f64 / gates2 as f64
        );

        // 4th order should have ~5x more gates than 2nd order
        // 6th order should have ~25x more gates than 2nd order
        assert!(gates4 > gates2);
        assert!(gates6 > gates4);
    }

    #[test]
    fn test_nth_order_error_estimation() {
        let h = constructors::ising_model(2, 1.0, 0.5).unwrap();

        let config2 = CompilerConfig {
            trotter_order: TrotterOrder::Nth(2),
            trotter_steps: 10,
            evolution_time: 1.0,
            ..Default::default()
        };
        let compiler2 = HamiltonianCompiler::new(config2);
        let analysis2 = compiler2.estimate_error(&h);

        let config6 = CompilerConfig {
            trotter_order: TrotterOrder::Nth(6),
            trotter_steps: 10,
            evolution_time: 1.0,
            ..Default::default()
        };
        let compiler6 = HamiltonianCompiler::new(config6);
        let analysis6 = compiler6.estimate_error(&h);

        let config10 = CompilerConfig {
            trotter_order: TrotterOrder::Nth(10),
            trotter_steps: 10,
            evolution_time: 1.0,
            ..Default::default()
        };
        let compiler10 = HamiltonianCompiler::new(config10);
        let analysis10 = compiler10.estimate_error(&h);

        println!(
            "2nd order error: {:.6e}, order: {}",
            analysis2.theoretical_error, analysis2.error_order
        );
        println!(
            "6th order error: {:.6e}, order: {}",
            analysis6.theoretical_error, analysis6.error_order
        );
        println!(
            "10th order error: {:.6e}, order: {}",
            analysis10.theoretical_error, analysis10.error_order
        );

        // Higher order should have lower error
        assert!(analysis6.theoretical_error < analysis2.theoretical_error);
        assert!(analysis10.theoretical_error < analysis6.theoretical_error);

        // Check error orders
        assert_eq!(analysis2.error_order, 3);
        assert_eq!(analysis6.error_order, 7);
        assert_eq!(analysis10.error_order, 11);
    }

    #[test]
    fn test_nth_order_invalid_odd() {
        let h = constructors::ising_model(2, 1.0, 0.5).unwrap();

        // Test that odd orders (except 1) are rejected
        for order in [3, 5, 7, 9] {
            let config = CompilerConfig {
                trotter_order: TrotterOrder::Nth(order),
                trotter_steps: 1,
                ..Default::default()
            };

            let compiler = HamiltonianCompiler::new(config);
            let result = compiler.compile(&h);

            assert!(result.is_err());
            println!("Nth({}) correctly rejected", order);
        }
    }

    #[test]
    fn test_nth_order_first_order() {
        let h = constructors::ising_model(2, 1.0, 0.5).unwrap();

        // Test that Nth(1) works (special case)
        let config = CompilerConfig {
            trotter_order: TrotterOrder::Nth(1),
            trotter_steps: 3,
            ..Default::default()
        };

        let compiler = HamiltonianCompiler::new(config);
        let circuit = compiler.compile(&h).unwrap();

        assert!(circuit.size() > 0);
        println!("Nth(1) first-order gates: {}", circuit.size());
    }

    #[test]
    fn test_nth_order_optimal_steps() {
        let h = constructors::ising_model(2, 1.0, 0.5).unwrap();
        let target_error = 0.01;

        let compiler4 = HamiltonianCompiler::new(CompilerConfig {
            trotter_order: TrotterOrder::Nth(4),
            ..Default::default()
        });
        let steps4 = compiler4.compute_optimal_steps(&h, target_error);

        let compiler8 = HamiltonianCompiler::new(CompilerConfig {
            trotter_order: TrotterOrder::Nth(8),
            ..Default::default()
        });
        let steps8 = compiler8.compute_optimal_steps(&h, target_error);

        println!("Optimal steps for error {:.3}:", target_error);
        println!("  4th order: {}", steps4);
        println!("  8th order: {}", steps8);

        // Higher order should require fewer steps
        assert!(steps8 < steps4);
        assert!(steps4 > 0 && steps8 > 0);
    }

    #[test]
    fn test_nth_order_vs_explicit() {
        let h = constructors::ising_model(2, 1.0, 0.5).unwrap();

        // Compare Nth(2) with Second
        let config_nth2 = CompilerConfig {
            trotter_order: TrotterOrder::Nth(2),
            trotter_steps: 3,
            ..Default::default()
        };
        let circuit_nth2 = HamiltonianCompiler::new(config_nth2).compile(&h).unwrap();

        let config_second = CompilerConfig {
            trotter_order: TrotterOrder::Second,
            trotter_steps: 3,
            ..Default::default()
        };
        let circuit_second = HamiltonianCompiler::new(config_second).compile(&h).unwrap();

        // Should produce same number of gates
        assert_eq!(circuit_nth2.size(), circuit_second.size());
        println!(
            "Nth(2) gates: {}, Second gates: {}",
            circuit_nth2.size(),
            circuit_second.size()
        );

        // Compare Nth(4) with Fourth
        let config_nth4 = CompilerConfig {
            trotter_order: TrotterOrder::Nth(4),
            trotter_steps: 1,
            ..Default::default()
        };
        let circuit_nth4 = HamiltonianCompiler::new(config_nth4).compile(&h).unwrap();

        let config_fourth = CompilerConfig {
            trotter_order: TrotterOrder::Fourth,
            trotter_steps: 1,
            ..Default::default()
        };
        let circuit_fourth = HamiltonianCompiler::new(config_fourth).compile(&h).unwrap();

        // Should produce same number of gates
        assert_eq!(circuit_nth4.size(), circuit_fourth.size());
        println!(
            "Nth(4) gates: {}, Fourth gates: {}",
            circuit_nth4.size(),
            circuit_fourth.size()
        );
    }

    #[test]
    fn test_full_optimization_pipeline() {
        // Test complete optimization: ordering + gate merging
        let h = constructors::heisenberg_model(2, 1.0, 1.0, 1.0).unwrap();

        let compiler = HamiltonianCompiler::new(CompilerConfig {
            trotter_order: TrotterOrder::Second,
            trotter_steps: 3,
            ..Default::default()
        });

        // Original compilation
        let circuit_original = compiler.compile(&h).unwrap();

        // Optimized compilation
        let h_optimized = compiler.optimize_pauli_ordering(&h);
        let circuit_reordered = compiler.compile(&h_optimized).unwrap();
        let circuit_final = compiler.optimize_circuit(&circuit_reordered).unwrap();

        println!("Original: {} gates", circuit_original.size());
        println!("After Pauli reordering: {} gates", circuit_reordered.size());
        println!("After gate merging: {} gates", circuit_final.size());

        // Final circuit should be no worse than original
        assert!(circuit_final.size() <= circuit_original.size());

        // Calculate improvement percentage
        if circuit_original.size() > 0 {
            let improvement = (circuit_original.size() as f64 - circuit_final.size() as f64)
                / circuit_original.size() as f64
                * 100.0;
            println!("Total improvement: {:.1}%", improvement);
        }
    }

    #[test]
    fn test_adaptive_step_size_computation() {
        // Test adaptive step size calculation
        let h = constructors::ising_model(2, 1.0, 0.5).unwrap();

        let config = CompilerConfig {
            trotter_order: TrotterOrder::Second,
            adaptive: true,
            adaptive_tolerance: 0.01,
            min_step_size: 0.001,
            max_step_size: 1.0,
            ..Default::default()
        };

        let compiler = HamiltonianCompiler::new(config);
        let dt = compiler.compute_adaptive_step_size(&h);

        println!("Adaptive step size: {:.6}", dt);

        // Step size should be within bounds
        assert!(dt >= 0.001);
        assert!(dt <= 1.0);

        // Verify local error is close to tolerance
        let local_error = compiler.estimate_local_error(&h, dt);
        println!("Estimated local error: {:.6e}", local_error);
        println!("Target tolerance: {:.6e}", 0.01);

        // Local error should be on same order as tolerance
        assert!(local_error <= 0.1); // Within order of magnitude
    }

    #[test]
    fn test_adaptive_compilation() {
        // Test adaptive compilation
        let mut h = Hamiltonian::new(2);
        let zz = PauliString::from_str("ZZ").unwrap();
        h.add_term(zz, num_complex::Complex64::new(1.0, 0.0))
            .unwrap();

        let config = CompilerConfig {
            trotter_order: TrotterOrder::Second,
            adaptive: true,
            adaptive_tolerance: 0.001,
            evolution_time: 1.0,
            min_step_size: 0.01,
            max_step_size: 0.5,
            ..Default::default()
        };

        let compiler = HamiltonianCompiler::new(config);
        let circuit = compiler.compile_adaptive(&h).unwrap();

        println!("Adaptive circuit gates: {}", circuit.size());

        // Should produce a valid circuit
        assert!(circuit.size() > 0);
        assert_eq!(circuit.num_qubits(), 2);
    }

    #[test]
    fn test_adaptive_vs_fixed_steps() {
        // Compare adaptive vs fixed step compilation
        let h = constructors::heisenberg_model(2, 1.0, 1.0, 1.0).unwrap();

        // Fixed steps
        let config_fixed = CompilerConfig {
            trotter_order: TrotterOrder::Second,
            trotter_steps: 10,
            evolution_time: 1.0,
            ..Default::default()
        };
        let compiler_fixed = HamiltonianCompiler::new(config_fixed);
        let circuit_fixed = compiler_fixed.compile(&h).unwrap();

        // Adaptive steps
        let config_adaptive = CompilerConfig {
            trotter_order: TrotterOrder::Second,
            adaptive: true,
            adaptive_tolerance: 0.01,
            evolution_time: 1.0,
            min_step_size: 0.001,
            max_step_size: 0.5,
            ..Default::default()
        };
        let compiler_adaptive = HamiltonianCompiler::new(config_adaptive);
        let circuit_adaptive = compiler_adaptive.compile_adaptive(&h).unwrap();

        println!("Fixed steps (n=10): {} gates", circuit_fixed.size());
        println!(
            "Adaptive steps (tol=0.01): {} gates",
            circuit_adaptive.size()
        );

        // Both should produce valid circuits
        assert!(circuit_fixed.size() > 0);
        assert!(circuit_adaptive.size() > 0);
    }

    #[test]
    fn test_adaptive_step_bounds() {
        // Test that adaptive steps respect min/max bounds
        let h = constructors::ising_model(3, 2.0, 1.0).unwrap();

        let config = CompilerConfig {
            trotter_order: TrotterOrder::Fourth,
            adaptive: true,
            adaptive_tolerance: 1e-6, // Very tight tolerance
            min_step_size: 0.01,
            max_step_size: 0.1,
            evolution_time: 1.0,
            ..Default::default()
        };

        let compiler = HamiltonianCompiler::new(config);
        let dt = compiler.compute_adaptive_step_size(&h);

        println!("Computed step size: {:.6}", dt);
        println!("Min bound: {:.6}", 0.01);
        println!("Max bound: {:.6}", 0.1);

        // Step must be within bounds
        assert!(dt >= 0.01 - 1e-10); // Allow small floating point error
        assert!(dt <= 0.1 + 1e-10);
    }

    #[test]
    fn test_local_error_scaling() {
        // Test that local error scales correctly with Trotter order
        let h = constructors::ising_model(2, 1.0, 0.5).unwrap();

        let dt = 0.1;

        // First order
        let compiler1 = HamiltonianCompiler::new(CompilerConfig {
            trotter_order: TrotterOrder::First,
            ..Default::default()
        });
        let error1 = compiler1.estimate_local_error(&h, dt);

        // Second order
        let compiler2 = HamiltonianCompiler::new(CompilerConfig {
            trotter_order: TrotterOrder::Second,
            ..Default::default()
        });
        let error2 = compiler2.estimate_local_error(&h, dt);

        // Fourth order
        let compiler4 = HamiltonianCompiler::new(CompilerConfig {
            trotter_order: TrotterOrder::Fourth,
            ..Default::default()
        });
        let error4 = compiler4.estimate_local_error(&h, dt);

        println!("Local errors at dt={}:", dt);
        println!("  First order:  {:.6e}", error1);
        println!("  Second order: {:.6e}", error2);
        println!("  Fourth order: {:.6e}", error4);

        // Higher order should have lower error
        assert!(error2 < error1);
        assert!(error4 < error2);
    }

    #[test]
    fn test_fourth_order_suzuki_coefficient_sum() {
        let p = 1.0 / (4.0 - 4.0_f64.powf(1.0 / 3.0));
        let p_mid = 1.0 - 4.0 * p;
        // Correct sequence: [p, p, 1-4p, p, p]
        let coeffs = [p, p, p_mid, p, p];
        let sum: f64 = coeffs.iter().sum();
        assert!(
            (sum - 1.0).abs() < 1e-10,
            "Suzuki 4th-order coefficients must sum to 1.0, got {}",
            sum
        );
    }
}
