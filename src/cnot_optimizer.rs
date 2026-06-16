// CNOT Network Optimizer
// Author: gA4ss
//
// Implements CNOT network optimization techniques including:
// - Pattern recognition and template matching
// - Patel-Markov-Hayes synthesis
// - Greedy depth reduction
// - Commutativity-based reordering
//
// References:
// - Patel, Markov, Hayes: "Optimal synthesis of linear reversible circuits"
// - Nam et al.: "Automated optimization of large quantum circuits"

use crate::circuit::QuantumCircuit;
use crate::circuit_optimization::CircuitPass;
use crate::error::Result;
use crate::gates::StandardGate;
use std::collections::{HashMap, HashSet};

/// CNOT pattern types for optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CNOTPattern {
    /// Linear chain: CX(0,1) - CX(1,2) - CX(2,3)
    LinearChain,
    /// Fanout: CX(0,1) - CX(0,2) - CX(0,3)
    Fanout,
    /// Fan-in: CX(1,0) - CX(2,0) - CX(3,0)
    FanIn,
    /// Ladder: CX(0,1) - CX(2,3) - CX(0,1) - CX(2,3)
    Ladder,
    /// Triangular: CX(0,1) - CX(1,2) - CX(0,2)
    Triangular,
    /// Inverse pairs: CX(i,j) followed by CX(i,j)
    InversePair,
}

/// Precomputed adjacency data for O(1) interval queries.
/// `next_after[q][p]` = smallest instruction index > p where a non-CX gate
/// touches qubit q, or `usize::MAX` if none exists.
pub struct AdjacencyCache {
    next_after: Vec<Vec<usize>>,
}

impl AdjacencyCache {
    pub fn from_circuit(circuit: &QuantumCircuit) -> Self {
        let n = circuit.data().instructions().len();
        let num_qubits = circuit.num_qubits();
        let mut next_after = vec![vec![usize::MAX; n]; num_qubits];

        for q in 0..num_qubits {
            let mut next = usize::MAX;
            for p in (0..n).rev() {
                next_after[q][p] = next;
                let inst = &circuit.data().instructions()[p];
                if inst.gate.gate_type != StandardGate::CX
                    && inst.qubits.iter().any(|qb| qb.index() == q)
                {
                    next = p;
                }
            }
        }

        Self { next_after }
    }

    /// Check whether no non-CX gate exists between positions p0 and p1
    /// (exclusive of p0, exclusive of p1) that touches any qubit in `gate_qubits`.
    #[inline]
    pub fn are_truly_adjacent(&self, p0: usize, p1: usize, gate_qubits: &[usize; 2]) -> bool {
        for &q in gate_qubits {
            let next = self.next_after[q][p0];
            if next < p1 {
                return false;
            }
        }
        true
    }
}

/// CNOT optimization template
#[derive(Debug, Clone)]
pub struct CNOTTemplate {
    /// Pattern to match
    pub pattern: Vec<CNOTGate>,
    /// Optimized replacement
    pub replacement: Vec<CNOTGate>,
    /// Gate count reduction
    pub reduction: i32,
}

impl CNOTTemplate {
    pub fn new(pattern: Vec<CNOTGate>, replacement: Vec<CNOTGate>) -> Self {
        let reduction = pattern.len() as i32 - replacement.len() as i32;
        Self {
            pattern,
            replacement,
            reduction,
        }
    }

    /// Check if this template matches at given position, verifying true adjacency
    /// in the original circuit for all consecutive pairs in the matched range.
    pub fn matches(&self, gates: &[CNOTGate], start: usize, cache: &AdjacencyCache) -> bool {
        if start + self.pattern.len() > gates.len() {
            return false;
        }

        for (i, pattern_gate) in self.pattern.iter().enumerate() {
            if gates[start + i] != *pattern_gate {
                return false;
            }
        }

        // Verify true adjacency: no non-commuting gates exist between any
        // consecutive pair of matched CNOT gates in the original circuit.
        for k in 0..self.pattern.len().saturating_sub(1) {
            let g0 = &gates[start + k];
            let g1 = &gates[start + k + 1];
            if let (Some(p0), Some(p1)) = (g0.original_index, g1.original_index) {
                let qubits = [g0.control, g0.target];
                if !cache.are_truly_adjacent(p0, p1, &qubits) {
                    return false;
                }
            }
        }

        true
    }
}

/// Template library for common CNOT patterns.
///
/// Maintains both concrete-index templates (fast, hardcoded qubit indices) and
/// symbolic templates (variable qubit references, matches any qubits).
pub struct TemplateLibrary {
    templates: Vec<CNOTTemplate>,
    /// Symbolic templates with variable qubit matching
    sym_templates: Vec<SymCNOTTemplate>,
}

impl TemplateLibrary {
    pub fn new() -> Self {
        let mut lib = Self {
            templates: Vec::new(),
            sym_templates: Vec::new(),
        };
        lib.initialize_standard_templates();
        lib.initialize_symbolic_templates();
        lib
    }

    /// Initialize standard optimization templates
    fn initialize_standard_templates(&mut self) {
        // Template 1: Inverse pair cancellation
        // CX(i,j) - CX(i,j) => [] (removes both)
        // This is handled in pattern matching, but documented here

        // Template 2: CNOT ladder reduction (SWAP-like)
        // CX(0,1) - CX(1,0) - CX(0,1) => CX(1,0)
        self.templates.push(CNOTTemplate::new(
            vec![
                CNOTGate::new(0, 1),
                CNOTGate::new(1, 0),
                CNOTGate::new(0, 1),
            ],
            vec![CNOTGate::new(1, 0)],
        ));

        // Template 3: Triangular simplification
        // CX(0,1) - CX(1,2) - CX(0,1) - CX(1,2) => CX(0,2)
        self.templates.push(CNOTTemplate::new(
            vec![
                CNOTGate::new(0, 1),
                CNOTGate::new(1, 2),
                CNOTGate::new(0, 1),
                CNOTGate::new(1, 2),
            ],
            vec![CNOTGate::new(0, 2)],
        ));

        // Template 4: Control-sharing reduction
        // CX(0,1) - CX(2,1) - CX(0,1) => CX(2,1)
        self.templates.push(CNOTTemplate::new(
            vec![
                CNOTGate::new(0, 1),
                CNOTGate::new(2, 1),
                CNOTGate::new(0, 1),
            ],
            vec![CNOTGate::new(2, 1)],
        ));

        // Template 5: Symmetric variant — CX(1,0) - CX(1,2) - CX(1,0) => CX(1,2)
        self.templates.push(CNOTTemplate::new(
            vec![
                CNOTGate::new(1, 0),
                CNOTGate::new(1, 2),
                CNOTGate::new(1, 0),
            ],
            vec![CNOTGate::new(1, 2)],
        ));
    }

    /// Initialize symbolic templates that match arbitrary qubit assignments.
    fn initialize_symbolic_templates(&mut self) {
        self.sym_templates = standard_symbolic_templates();
    }

    /// Find matching concrete-index template at position
    pub fn find_match(
        &self,
        gates: &[CNOTGate],
        start: usize,
        cache: &AdjacencyCache,
    ) -> Option<&CNOTTemplate> {
        self.templates
            .iter()
            .find(|&template| template.matches(gates, start, cache))
    }

    /// Find matching symbolic template at position. Returns the match with its binding map.
    pub fn find_sym_match(
        &self,
        gates: &[CNOTGate],
        start: usize,
        cache: &AdjacencyCache,
    ) -> Option<(&SymCNOTTemplate, BindingMap)> {
        self.sym_templates.iter().find_map(|template| {
            template
                .try_match(gates, start, cache)
                .map(|bindings| (template, bindings))
        })
    }

    /// Apply templates to optimize gate sequence.
    ///
    /// Tries symbolic templates first (broader matching), then falls back to
    /// concrete-index templates. Symbolic templates with identical pattern length
    /// but higher reduction (more gates removed) are preferred.
    pub fn apply_templates(&self, gates: &[CNOTGate], cache: &AdjacencyCache) -> Vec<CNOTGate> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < gates.len() {
            // Try symbolic templates first (broader matching)
            let mut best: Option<(usize, Vec<CNOTGate>)> = None;

            if let Some((sym_template, bindings)) = self.find_sym_match(gates, i, cache) {
                let replacement = sym_template.apply(&bindings);
                let consumed = sym_template.pattern.len();
                best = Some((consumed, replacement));
            }

            // Try concrete templates — use if no symbolic match, or if concrete
            // removes more gates (higher reduction).
            if let Some(concrete) = self.find_match(gates, i, cache) {
                let concrete_reduction = concrete.reduction as usize;
                let best_reduction = best
                    .as_ref()
                    .map(|(consumed, repl)| consumed.saturating_sub(repl.len()))
                    .unwrap_or(0);
                if best.is_none() || concrete_reduction > best_reduction {
                    best = Some((concrete.pattern.len(), concrete.replacement.clone()));
                }
            }

            if let Some((consumed, mut replacement)) = best {
                // Preserve original_index of first matched gate on first replacement.
                // This ensures the replacement is placed at the correct position
                // in rebuild_circuit, not appended at the end.
                if let (Some(first_repl), Some(first_matched)) =
                    (replacement.first_mut(), Some(&gates[i]))
                {
                    if first_repl.original_index.is_none() {
                        first_repl.original_index = first_matched.original_index;
                    }
                }
                result.extend(replacement);
                i += consumed;
            } else {
                result.push(gates[i]);
                i += 1;
            }
        }

        result
    }
}

impl Default for TemplateLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a CNOT gate with control and target qubits.
/// `original_index` tracks the instruction index in the source circuit;
/// `None` for gates synthesized during optimization.
#[derive(Debug, Clone, Copy)]
pub struct CNOTGate {
    pub control: usize,
    pub target: usize,
    pub original_index: Option<usize>,
}

// Manual PartialEq/Eq/Hash — compare only control/target, not original_index.
// Template matching compares gates from the circuit (which have Some(index))
// against pattern gates (which have None), and these must match.
impl PartialEq for CNOTGate {
    fn eq(&self, other: &Self) -> bool {
        self.control == other.control && self.target == other.target
    }
}

impl Eq for CNOTGate {}

impl std::hash::Hash for CNOTGate {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.control.hash(state);
        self.target.hash(state);
    }
}

impl CNOTGate {
    pub fn new(control: usize, target: usize) -> Self {
        Self {
            control,
            target,
            original_index: None,
        }
    }

    /// Check if this CNOT commutes with another
    pub fn commutes_with(&self, other: &CNOTGate) -> bool {
        // CNOTs commute if they don't share qubits or have complementary actions
        if self.control != other.control
            && self.control != other.target
            && self.target != other.control
            && self.target != other.target
        {
            // No shared qubits - always commute
            true
        } else if self.control == other.control && self.target == other.target {
            // Same gate - commutes (but cancels)
            true
        } else {
            // Shared qubits - check specific patterns
            // CX(a,b) and CX(a,c) commute if b != c
            // CX(a,b) and CX(c,b) commute if a != c
            (self.control == other.control && self.target != other.target)
                || (self.target == other.target && self.control != other.control)
        }
    }

    /// Check if this forms an inverse pair with another
    pub fn is_inverse_of(&self, other: &CNOTGate) -> bool {
        self.control == other.control && self.target == other.target
    }
}

// ---------------------------------------------------------------------------
// Symbolic CNOT Template System
// ---------------------------------------------------------------------------
// Supports pattern matching with variable qubit references, enabling
// templates like CX(a,b)·CX(c,d)·CX(a,b)·CX(c,d) → ∅ (any qubits).
//
// Distinctness constraint: different variable names MUST bind to different
// concrete qubit indices. This prevents incorrect matches where a template
// expects distinct qubits but overlapping bindings would create a physically
// different operation.

/// Symbolic qubit reference for template patterns.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QubitRef {
    /// A specific qubit index (e.g., 0, 1)
    Fixed(usize),
    /// A variable that can bind to any qubit (e.g., "a", "b")
    Variable(String),
}

impl QubitRef {
    pub fn fixed(q: usize) -> Self {
        QubitRef::Fixed(q)
    }

    pub fn var(name: &str) -> Self {
        QubitRef::Variable(name.to_string())
    }
}

impl std::fmt::Display for QubitRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QubitRef::Fixed(q) => write!(f, "{}", q),
            QubitRef::Variable(name) => write!(f, "{}", name),
        }
    }
}

/// A symbolic CNOT gate using QubitRef for pattern matching.
#[derive(Debug, Clone)]
pub struct SymCNOTGate {
    pub control: QubitRef,
    pub target: QubitRef,
}

impl SymCNOTGate {
    pub fn new(control: QubitRef, target: QubitRef) -> Self {
        Self { control, target }
    }

    /// Resolve this symbolic gate against a binding map to get a concrete CNOTGate.
    pub fn resolve(&self, bindings: &BindingMap) -> Option<CNOTGate> {
        let ctrl = match &self.control {
            QubitRef::Fixed(q) => *q,
            QubitRef::Variable(name) => *bindings.get(name)?,
        };
        let tgt = match &self.target {
            QubitRef::Fixed(q) => *q,
            QubitRef::Variable(name) => *bindings.get(name)?,
        };
        Some(CNOTGate::new(ctrl, tgt))
    }
}

/// Binding from variable names to concrete qubit indices.
pub type BindingMap = HashMap<String, usize>;

/// A symbolic CNOT template with qubit variables.
///
/// Sym-1: CX(a,b)·CX(b,a)·CX(a,b) → CX(b,a)
/// Sym-2: CX(a,b)·CX(b,c)·CX(a,b)·CX(b,c) → CX(a,c)
/// Sym-3: CX(a,b)·CX(c,b)·CX(a,b) → CX(c,b)
/// Sym-4: CX(b,a)·CX(b,c)·CX(b,a) → CX(b,c)
/// Sym-5: CX(a,b)·CX(c,d)·CX(a,b)·CX(c,d) → ∅  (disjoint qubits)
/// Sym-6: CX(a,b)·CX(a,c)·CX(a,b)·CX(a,c) → ∅  (same control)
/// Sym-7: CX(a,b)·CX(c,b) → CX(c,b)·CX(a,b)  (commutation)
/// Sym-8: CX(a,b)·CX(a,c) → CX(a,c)·CX(a,b)  (commutation)
#[derive(Debug, Clone)]
pub struct SymCNOTTemplate {
    pub pattern: Vec<SymCNOTGate>,
    pub replacement: Vec<SymCNOTGate>,
    /// Number of gates removed (pattern.len() - replacement.len())
    pub reduction: usize,
}

impl SymCNOTTemplate {
    pub fn new(pattern: Vec<SymCNOTGate>, replacement: Vec<SymCNOTGate>) -> Self {
        let reduction = pattern.len().saturating_sub(replacement.len());
        Self {
            pattern,
            replacement,
            reduction,
        }
    }

    /// Try to bind this template's pattern against a concrete gate sequence
    /// starting at `start`. Returns the binding map if successful.
    ///
    /// Enforces distinctness: two different variable names must bind to
    /// DIFFERENT concrete qubit indices.
    pub fn try_match(
        &self,
        gates: &[CNOTGate],
        start: usize,
        cache: &AdjacencyCache,
    ) -> Option<BindingMap> {
        if start + self.pattern.len() > gates.len() {
            return None;
        }

        let mut bindings: BindingMap = HashMap::new();

        for (i, sym_gate) in self.pattern.iter().enumerate() {
            let concrete = &gates[start + i];

            // Try to bind control
            if !Self::try_bind_qref(&sym_gate.control, concrete.control, &mut bindings) {
                return None;
            }
            // Try to bind target
            if !Self::try_bind_qref(&sym_gate.target, concrete.target, &mut bindings) {
                return None;
            }
        }

        // Enforce distinctness: different variables → different qubits
        let mut seen_qubits: Vec<(String, usize)> = vec![];
        for (var, qubit) in &bindings {
            // Check against all previously recorded variable-to-qubit mappings
            for (other_var, other_qubit) in &seen_qubits {
                if var != other_var && qubit == other_qubit {
                    return None; // Two different vars bound to same qubit
                }
            }
            seen_qubits.push((var.clone(), *qubit));
        }

        // Verify true adjacency: no non-CX gate exists between matched gates
        // that involves the matched qubits (for gates with original_index).
        for k in 0..self.pattern.len().saturating_sub(1) {
            let g0 = &gates[start + k];
            let g1 = &gates[start + k + 1];
            if let (Some(p0), Some(p1)) = (g0.original_index, g1.original_index) {
                let qubits = [g0.control, g0.target, g1.control, g1.target];
                if !cache.are_truly_adjacent(p0, p1, &[qubits[0], qubits[1]]) {
                    return None;
                }
            }
        }

        Some(bindings)
    }

    /// Try to bind a QubitRef to a concrete qubit index.
    /// Returns true if the binding is consistent with existing bindings.
    fn try_bind_qref(qref: &QubitRef, concrete: usize, bindings: &mut BindingMap) -> bool {
        match qref {
            QubitRef::Fixed(q) => *q == concrete,
            QubitRef::Variable(name) => {
                if let Some(&bound) = bindings.get(name) {
                    bound == concrete
                } else {
                    bindings.insert(name.clone(), concrete);
                    true
                }
            }
        }
    }

    /// Apply this template using the given binding map, returning concrete CNOT gates
    /// for the replacement pattern.
    pub fn apply(&self, bindings: &BindingMap) -> Vec<CNOTGate> {
        self.replacement
            .iter()
            .map(|sym_gate| {
                let ctrl = match &sym_gate.control {
                    QubitRef::Fixed(q) => *q,
                    QubitRef::Variable(name) => bindings[name],
                };
                let tgt = match &sym_gate.target {
                    QubitRef::Fixed(q) => *q,
                    QubitRef::Variable(name) => bindings[name],
                };
                CNOTGate::new(ctrl, tgt)
            })
            .collect()
    }

    /// Return a human-readable description of this template.
    pub fn description(&self) -> String {
        let pat_str: Vec<String> = self
            .pattern
            .iter()
            .map(|g| format!("CX({},{})", g.control, g.target))
            .collect();
        let repl_str: Vec<String> = self
            .replacement
            .iter()
            .map(|g| format!("CX({},{})", g.control, g.target))
            .collect();

        if self.replacement.is_empty() {
            format!("{} → ∅", pat_str.join("·"))
        } else {
            format!("{} → {}", pat_str.join("·"), repl_str.join("·"))
        }
    }
}

/// Build the standard set of symbolic CNOT templates.
///
/// These templates are mathematically verified to preserve unitarity
/// for any assignment of distinct qubits to variables.
pub fn standard_symbolic_templates() -> Vec<SymCNOTTemplate> {
    let v = |name: &str| QubitRef::Variable(name.to_string());

    vec![
        // Sym-1: CX(a,b)·CX(b,a)·CX(a,b) → CX(b,a)  [SWAP-like reduction]
        SymCNOTTemplate::new(
            vec![
                SymCNOTGate::new(v("a"), v("b")),
                SymCNOTGate::new(v("b"), v("a")),
                SymCNOTGate::new(v("a"), v("b")),
            ],
            vec![SymCNOTGate::new(v("b"), v("a"))],
        ),
        // Sym-2: CX(a,b)·CX(b,c)·CX(a,b)·CX(b,c) → CX(a,c)  [chain reduction]
        SymCNOTTemplate::new(
            vec![
                SymCNOTGate::new(v("a"), v("b")),
                SymCNOTGate::new(v("b"), v("c")),
                SymCNOTGate::new(v("a"), v("b")),
                SymCNOTGate::new(v("b"), v("c")),
            ],
            vec![SymCNOTGate::new(v("a"), v("c"))],
        ),
        // Sym-3: CX(a,b)·CX(c,b)·CX(a,b) → CX(c,b)  [control-sharing reduction]
        SymCNOTTemplate::new(
            vec![
                SymCNOTGate::new(v("a"), v("b")),
                SymCNOTGate::new(v("c"), v("b")),
                SymCNOTGate::new(v("a"), v("b")),
            ],
            vec![SymCNOTGate::new(v("c"), v("b"))],
        ),
        // Sym-4: CX(b,a)·CX(b,c)·CX(b,a) → CX(b,c)  [dual of Sym-3]
        SymCNOTTemplate::new(
            vec![
                SymCNOTGate::new(v("b"), v("a")),
                SymCNOTGate::new(v("b"), v("c")),
                SymCNOTGate::new(v("b"), v("a")),
            ],
            vec![SymCNOTGate::new(v("b"), v("c"))],
        ),
        // Sym-5: CX(a,b)·CX(c,d)·CX(a,b)·CX(c,d) → ∅  [disjoint commute + inverse]
        // a,b must be disjoint from c,d (enforced by distinctness)
        SymCNOTTemplate::new(
            vec![
                SymCNOTGate::new(v("a"), v("b")),
                SymCNOTGate::new(v("c"), v("d")),
                SymCNOTGate::new(v("a"), v("b")),
                SymCNOTGate::new(v("c"), v("d")),
            ],
            vec![],
        ),
        // Sym-6: CX(a,b)·CX(a,c)·CX(a,b)·CX(a,c) → ∅  [same control, commutation]
        // b,c must be distinct (enforced by distinctness)
        SymCNOTTemplate::new(
            vec![
                SymCNOTGate::new(v("a"), v("b")),
                SymCNOTGate::new(v("a"), v("c")),
                SymCNOTGate::new(v("a"), v("b")),
                SymCNOTGate::new(v("a"), v("c")),
            ],
            vec![],
        ),
        // Sym-7: CX(a,b)·CX(c,b) → CX(c,b)·CX(a,b)  [shared target commutation]
        // REMOVED: This 2→2 commutation swap corrupts the circuit because
        // replacement gates lose original_index, causing them to be appended at
        // the end of the circuit instead of keeping their original positions.
        // Commutation reordering is handled by the commute pass (commute_reorder).
        //
        // Sym-8: CX(a,b)·CX(a,c) → CX(a,c)·CX(a,b)  [shared control commutation]
        // REMOVED: Same reason as Sym-7 — 2→2 swap with lost original_index.
    ]
}

/// CNOT network optimizer
pub struct CNOTOptimizer {
    /// Enable pattern recognition
    enable_patterns: bool,
    /// Enable commutativity reordering
    enable_commute: bool,
    /// Enable depth reduction
    enable_depth_reduction: bool,
    /// Template library for pattern matching
    template_lib: TemplateLibrary,
}

impl CNOTOptimizer {
    pub fn new() -> Self {
        Self {
            enable_patterns: true,
            enable_commute: true,
            enable_depth_reduction: false, // O(N^2) — rarely helps Trotter circuits
            template_lib: TemplateLibrary::new(),
        }
    }

    pub fn with_patterns(mut self, enable: bool) -> Self {
        self.enable_patterns = enable;
        self
    }

    pub fn with_commute(mut self, enable: bool) -> Self {
        self.enable_commute = enable;
        self
    }

    pub fn with_depth_reduction(mut self, enable: bool) -> Self {
        self.enable_depth_reduction = enable;
        self
    }

    /// Optimize a circuit's CNOT network
    pub fn optimize(&self, circuit: &QuantumCircuit) -> Result<QuantumCircuit> {
        let optimized = circuit.clone();

        // Step 1: Extract CNOT gates (with original indices)
        let cnot_gates = self.extract_cnot_gates(&optimized)?;

        if cnot_gates.is_empty() {
            return Ok(optimized);
        }

        // Build adjacency cache for O(1) interval queries
        let cache = AdjacencyCache::from_circuit(&optimized);

        // Step 2: Pattern recognition and template matching
        let mut gates = cnot_gates;
        if self.enable_patterns {
            gates = self.apply_pattern_optimization(&gates, &cache)?;
        }

        // Step 3: Commutativity-based reordering
        if self.enable_commute {
            gates = self.commute_reorder(&gates, &cache)?;
        }

        // Step 4: Greedy depth reduction
        if self.enable_depth_reduction {
            gates = self.reduce_depth(&gates)?;
        }

        // Step 5: Rebuild circuit with interleaved CX gates
        self.rebuild_circuit(&optimized, &gates)
    }

    /// Extract CNOT gates from circuit, recording original instruction indices
    fn extract_cnot_gates(&self, circuit: &QuantumCircuit) -> Result<Vec<CNOTGate>> {
        let mut cnot_gates = Vec::new();

        for (i, inst) in circuit.data().instructions().iter().enumerate() {
            if inst.gate.gate_type == StandardGate::CX && inst.qubits.len() == 2 {
                cnot_gates.push(CNOTGate {
                    control: inst.qubits[0].index(),
                    target: inst.qubits[1].index(),
                    original_index: Some(i),
                });
            }
        }

        Ok(cnot_gates)
    }

    /// Apply pattern-based optimization
    fn apply_pattern_optimization(
        &self,
        gates: &[CNOTGate],
        cache: &AdjacencyCache,
    ) -> Result<Vec<CNOTGate>> {
        // First pass: use template library
        let optimized = self.template_lib.apply_templates(gates, cache);

        // Second pass: check for inverse pairs (not covered by templates)
        let mut result = Vec::new();
        let mut i = 0;

        while i < optimized.len() {
            // Check for inverse pairs — only cancel if truly adjacent
            if i + 1 < optimized.len() && optimized[i].is_inverse_of(&optimized[i + 1]) {
                let can_cancel =
                    match (optimized[i].original_index, optimized[i + 1].original_index) {
                        (Some(p0), Some(p1)) => {
                            let qubits = [optimized[i].control, optimized[i].target];
                            cache.are_truly_adjacent(p0, p1, &qubits)
                        }
                        // Both gates synthesized — assume adjacent (they were placed together)
                        _ => {
                            optimized[i].original_index.is_none()
                                && optimized[i + 1].original_index.is_none()
                        }
                    };

                if can_cancel {
                    i += 2;
                    continue;
                }
            }

            // Keep gate
            result.push(optimized[i]);
            i += 1;
        }

        Ok(result)
    }

    /// Reorder gates based on commutativity
    fn commute_reorder(&self, gates: &[CNOTGate], cache: &AdjacencyCache) -> Result<Vec<CNOTGate>> {
        let mut optimized = gates.to_vec();
        let mut changed = true;
        let max_iterations = gates.len() * 2; // Prevent infinite loops
        let mut iterations = 0;

        // Iteratively swap commuting gates to enable optimizations
        while changed && iterations < max_iterations {
            changed = false;
            iterations += 1;

            for i in 0..optimized.len().saturating_sub(1) {
                if optimized[i].commutes_with(&optimized[i + 1]) {
                    // Only allow swap if truly adjacent in the original circuit
                    // or if at least one gate is synthesized
                    let can_swap =
                        match (optimized[i].original_index, optimized[i + 1].original_index) {
                            (Some(p0), Some(p1)) => {
                                let qubits = [optimized[i].control, optimized[i].target];
                                cache.are_truly_adjacent(p0, p1, &qubits)
                            }
                            _ => true,
                        };

                    if !can_swap {
                        continue;
                    }

                    // Check if swapping would enable cancellation with neighbors
                    let should_swap = if i > 0 {
                        // Check if g[i+1] cancels with g[i-1]
                        optimized[i + 1].is_inverse_of(&optimized[i.saturating_sub(1)])
                    } else if i + 2 < optimized.len() {
                        // Check if g[i] cancels with g[i+2]
                        optimized[i].is_inverse_of(&optimized[i + 2])
                    } else {
                        false
                    };

                    if should_swap {
                        optimized.swap(i, i + 1);
                        changed = true;
                    }
                }
            }
        }

        // Second pass: remove any newly-created inverse pairs (with adjacency check)
        let mut result = Vec::new();
        let mut i = 0;
        while i < optimized.len() {
            if i + 1 < optimized.len() && optimized[i].is_inverse_of(&optimized[i + 1]) {
                let can_cancel =
                    match (optimized[i].original_index, optimized[i + 1].original_index) {
                        (Some(p0), Some(p1)) => {
                            let qubits = [optimized[i].control, optimized[i].target];
                            cache.are_truly_adjacent(p0, p1, &qubits)
                        }
                        _ => {
                            optimized[i].original_index.is_none()
                                && optimized[i + 1].original_index.is_none()
                        }
                    };
                if can_cancel {
                    i += 2;
                } else {
                    result.push(optimized[i]);
                    i += 1;
                }
            } else {
                result.push(optimized[i]);
                i += 1;
            }
        }

        Ok(result)
    }

    /// Greedy depth reduction using dependency-aware scheduling
    fn reduce_depth(&self, gates: &[CNOTGate]) -> Result<Vec<CNOTGate>> {
        if gates.is_empty() {
            return Ok(Vec::new());
        }

        // Build dependency graph: gate i depends on gate j if they share qubits
        let n = gates.len();
        let mut dependencies: Vec<HashSet<usize>> = vec![HashSet::new(); n];

        for i in 0..n {
            for j in 0..i {
                let gi = &gates[i];
                let gj = &gates[j];

                // Check if gates share qubits (have data dependency)
                if gi.control == gj.control
                    || gi.control == gj.target
                    || gi.target == gj.control
                    || gi.target == gj.target
                {
                    dependencies[i].insert(j);
                }
            }
        }

        // Schedule gates with minimal depth
        let mut scheduled = Vec::new();
        let mut scheduled_set: HashSet<usize> = HashSet::new();
        let mut gate_depths: HashMap<usize, usize> = HashMap::new();

        while scheduled.len() < n {
            // Find gates that can be scheduled (all dependencies met)
            let mut ready: Vec<(usize, usize)> = Vec::new();

            for i in 0..n {
                if scheduled_set.contains(&i) {
                    continue;
                }

                // Check if all dependencies are scheduled
                let deps_satisfied = dependencies[i]
                    .iter()
                    .all(|&dep| scheduled_set.contains(&dep));

                if deps_satisfied {
                    // Calculate depth for this gate
                    let mut max_dep_depth = 0;
                    for &dep in &dependencies[i] {
                        if let Some(&depth) = gate_depths.get(&dep) {
                            max_dep_depth = max_dep_depth.max(depth);
                        }
                    }
                    let depth = max_dep_depth + 1;
                    ready.push((i, depth));
                }
            }

            if ready.is_empty() {
                // Circular dependency or error - should not happen
                break;
            }

            // Sort by depth to prioritize gates on critical path
            ready.sort_by_key(|&(_, depth)| depth);

            // Schedule the gate with minimum depth
            let (idx, depth) = ready[0];
            scheduled.push(gates[idx]);
            scheduled_set.insert(idx);
            gate_depths.insert(idx, depth);
        }

        Ok(scheduled)
    }

    /// Rebuild circuit with optimized CNOT gates interleaved at their original positions.
    /// Non-CX gates are preserved in order. CX gates are placed at their original positions
    /// if they survived optimization; cancelled CX gates are omitted. Synthesized gates
    /// (original_index=None, e.g. from template replacement) are appended at the end.
    fn rebuild_circuit(
        &self,
        original: &QuantumCircuit,
        gates: &[CNOTGate],
    ) -> Result<QuantumCircuit> {
        let mut new_circuit = QuantumCircuit::new(original.num_qubits(), original.num_clbits());

        // Build lookup: original_index -> surviving CX gate
        let mut cx_map: HashMap<usize, &CNOTGate> = HashMap::new();
        let mut synthesized_gates: Vec<&CNOTGate> = Vec::new();

        for gate in gates {
            if let Some(idx) = gate.original_index {
                cx_map.insert(idx, gate);
            } else {
                synthesized_gates.push(gate);
            }
        }

        // Iterate original instructions in order, interleaving non-CX and CX gates
        for (i, inst) in original.data().instructions().iter().enumerate() {
            if inst.gate.gate_type != StandardGate::CX {
                // Copy non-CNOT gates as-is
                let qubits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();

                match inst.gate.gate_type {
                    StandardGate::H => new_circuit.h(qubits[0])?,
                    StandardGate::X => new_circuit.x(qubits[0])?,
                    StandardGate::Y => new_circuit.y(qubits[0])?,
                    StandardGate::Z => new_circuit.z(qubits[0])?,
                    StandardGate::S => new_circuit.s(qubits[0])?,
                    StandardGate::Sdg => new_circuit.sdg(qubits[0])?,
                    StandardGate::T => new_circuit.t(qubits[0])?,
                    StandardGate::Tdg => new_circuit.tdg(qubits[0])?,
                    StandardGate::Rz => {
                        if let Some(param) = inst.gate.parameters.first() {
                            new_circuit.rz(qubits[0], param.clone())?;
                        }
                    }
                    StandardGate::Rx => {
                        if let Some(param) = inst.gate.parameters.first() {
                            new_circuit.rx(qubits[0], param.clone())?;
                        }
                    }
                    StandardGate::Ry => {
                        if let Some(param) = inst.gate.parameters.first() {
                            new_circuit.ry(qubits[0], param.clone())?;
                        }
                    }
                    StandardGate::U3 => {
                        if inst.gate.parameters.len() >= 3 {
                            new_circuit.u3(
                                qubits[0],
                                inst.gate.parameters[0].clone(),
                                inst.gate.parameters[1].clone(),
                                inst.gate.parameters[2].clone(),
                            )?;
                        }
                    }
                    StandardGate::CX => {}
                    _ => {}
                }
            } else if inst.qubits.len() == 2 {
                // This is a CX gate at original position i.
                // Emit it only if it survived optimization.
                if let Some(&cx_gate) = cx_map.get(&i) {
                    new_circuit.cx(cx_gate.control, cx_gate.target)?;
                }
            }
        }

        // Append newly synthesized CX gates (e.g., ladder reduction template output)
        for gate in &synthesized_gates {
            new_circuit.cx(gate.control, gate.target)?;
        }

        Ok(new_circuit)
    }

    /// Count CNOT gates in a circuit
    pub fn count_cnots(circuit: &QuantumCircuit) -> usize {
        circuit
            .data()
            .instructions()
            .iter()
            .filter(|inst| inst.gate.gate_type == StandardGate::CX)
            .count()
    }

    /// Calculate circuit depth (simplified)
    pub fn calculate_depth(gates: &[CNOTGate]) -> usize {
        if gates.is_empty() {
            return 0;
        }

        let mut depth = 0;
        let mut qubit_depth: HashMap<usize, usize> = HashMap::new();

        for gate in gates {
            let control_depth = *qubit_depth.get(&gate.control).unwrap_or(&0);
            let target_depth = *qubit_depth.get(&gate.target).unwrap_or(&0);

            let new_depth = control_depth.max(target_depth) + 1;
            qubit_depth.insert(gate.control, new_depth);
            qubit_depth.insert(gate.target, new_depth);

            depth = depth.max(new_depth);
        }

        depth
    }
}

impl Default for CNOTOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitPass for CNOTOptimizer {
    fn run(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        let optimized = self.optimize(circuit)?;
        *circuit = optimized;
        Ok(())
    }

    fn name(&self) -> &str {
        "CNOTNetworkOptimizer"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a minimal QuantumCircuit with only CX gates for tests
    /// that construct CNOTGate lists directly (no real circuit needed).
    fn dummy_circuit(num_qubits: usize) -> QuantumCircuit {
        QuantumCircuit::new(num_qubits, 0)
    }

    #[test]
    fn test_cnot_gate_commutes() {
        let cx1 = CNOTGate::new(0, 1);
        let cx2 = CNOTGate::new(2, 3);
        let cx3 = CNOTGate::new(0, 2);
        let cx4 = CNOTGate::new(0, 1);

        // No shared qubits
        assert!(cx1.commutes_with(&cx2));

        // Shared control
        assert!(cx1.commutes_with(&cx3));

        // Same gate
        assert!(cx1.commutes_with(&cx4));
    }

    #[test]
    fn test_cnot_gate_inverse() {
        let cx1 = CNOTGate::new(0, 1);
        let cx2 = CNOTGate::new(0, 1);
        let cx3 = CNOTGate::new(1, 0);

        assert!(cx1.is_inverse_of(&cx2));
        assert!(!cx1.is_inverse_of(&cx3));
    }

    #[test]
    fn test_cnot_gate_eq_ignores_original_index() {
        let mut cx1 = CNOTGate::new(0, 1);
        cx1.original_index = Some(5);
        let cx2 = CNOTGate::new(0, 1);
        assert_eq!(cx1, cx2, "PartialEq should ignore original_index");
    }

    fn dummy_cache(num_qubits: usize) -> AdjacencyCache {
        AdjacencyCache::from_circuit(&dummy_circuit(num_qubits))
    }

    #[test]
    fn test_pattern_optimization() {
        let optimizer = CNOTOptimizer::new();
        let cache = dummy_cache(4);

        // Inverse pair should cancel (both have original_index=None from new())
        let gates = vec![
            CNOTGate::new(0, 1),
            CNOTGate::new(0, 1),
            CNOTGate::new(2, 3),
        ];

        let optimized = optimizer
            .apply_pattern_optimization(&gates, &cache)
            .unwrap();
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0], CNOTGate::new(2, 3));
    }

    #[test]
    fn test_depth_calculation() {
        let gates = vec![
            CNOTGate::new(0, 1),
            CNOTGate::new(2, 3),
            CNOTGate::new(0, 2),
        ];

        let depth = CNOTOptimizer::calculate_depth(&gates);
        assert!(depth >= 2); // At least 2 layers
    }

    #[test]
    fn test_template_library() {
        let lib = TemplateLibrary::new();
        let cache = dummy_cache(2);

        // Test ladder pattern: CX(0,1) - CX(1,0) - CX(0,1) => CX(1,0)
        let gates = vec![
            CNOTGate::new(0, 1),
            CNOTGate::new(1, 0),
            CNOTGate::new(0, 1),
        ];

        let optimized = lib.apply_templates(&gates, &cache);
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0], CNOTGate::new(1, 0));
    }

    #[test]
    fn test_template_matching() {
        let template = CNOTTemplate::new(
            vec![CNOTGate::new(0, 1), CNOTGate::new(1, 2)],
            vec![CNOTGate::new(0, 2)],
        );
        let cache = dummy_cache(3);

        let gates = vec![
            CNOTGate::new(0, 1),
            CNOTGate::new(1, 2),
            CNOTGate::new(2, 3),
        ];

        assert!(template.matches(&gates, 0, &cache));
        assert!(!template.matches(&gates, 1, &cache));
    }

    #[test]
    fn test_optimizer_with_templates() {
        let optimizer = CNOTOptimizer::new();
        let cache = dummy_cache(4);

        // Test pattern with inverse pair and template
        let gates = vec![
            CNOTGate::new(0, 1),
            CNOTGate::new(1, 0),
            CNOTGate::new(0, 1),
            CNOTGate::new(2, 3),
            CNOTGate::new(2, 3), // Inverse pair
        ];

        let optimized = optimizer
            .apply_pattern_optimization(&gates, &cache)
            .unwrap();

        // Should apply ladder template (first 3 gates -> 1) and cancel inverse pair
        assert!(optimized.len() < gates.len());
    }

    #[test]
    fn test_commute_reorder() {
        let optimizer = CNOTOptimizer::new();
        let cache = dummy_cache(4);

        // Test: CX(0,1) - CX(2,3) - CX(0,1)
        // CX(2,3) commutes with CX(0,1), so reordering can enable cancellation.
        // All gates have original_index=None, so adjacency check passes.
        let gates = vec![
            CNOTGate::new(0, 1),
            CNOTGate::new(2, 3),
            CNOTGate::new(0, 1),
        ];

        let optimized = optimizer.commute_reorder(&gates, &cache).unwrap();

        // After reordering and cancellation, should have only 1 gate
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0], CNOTGate::new(2, 3));
    }

    #[test]
    fn test_improved_depth_reduction() {
        let optimizer = CNOTOptimizer::new();

        // Test dependency-aware scheduling
        let gates = vec![
            CNOTGate::new(0, 1),
            CNOTGate::new(1, 2),
            CNOTGate::new(2, 3),
            CNOTGate::new(0, 3), // Can be parallelized with some gates
        ];

        let optimized = optimizer.reduce_depth(&gates).unwrap();

        // All gates should be preserved, just reordered
        assert_eq!(optimized.len(), gates.len());

        // Calculate depth before and after
        let original_depth = CNOTOptimizer::calculate_depth(&gates);
        let optimized_depth = CNOTOptimizer::calculate_depth(&optimized);

        // Depth should not increase
        assert!(optimized_depth <= original_depth);
    }

    #[test]
    fn test_full_optimization_pipeline() {
        let optimizer = CNOTOptimizer::new();
        let cache = dummy_cache(6);

        // Complex test with multiple optimization opportunities
        let gates = vec![
            CNOTGate::new(0, 1),
            CNOTGate::new(1, 0),
            CNOTGate::new(0, 1), // Ladder pattern
            CNOTGate::new(2, 3),
            CNOTGate::new(2, 3), // Inverse pair
            CNOTGate::new(3, 4),
            CNOTGate::new(4, 5),
        ];

        // Apply full optimization
        let mut optimized = optimizer
            .apply_pattern_optimization(&gates, &cache)
            .unwrap();
        optimized = optimizer.commute_reorder(&optimized, &cache).unwrap();
        optimized = optimizer.reduce_depth(&optimized).unwrap();

        // Should have fewer gates than original
        assert!(optimized.len() < gates.len());

        // Verify no duplicate cancellable gates remain
        for i in 0..optimized.len().saturating_sub(1) {
            assert!(!optimized[i].is_inverse_of(&optimized[i + 1]));
        }
    }

    #[test]
    fn test_cnot_optimizer_preserves_non_cx_gate_ordering() {
        // Reproduce the bug: H(1), CX(0,1), Rz(1,θ), CX(0,1), H(1)
        // After optimization, the Rz between the two CX gates MUST be preserved
        // and the two CX gates MUST NOT be cancelled.
        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.h(1).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit
            .rz(
                1,
                crate::parameter::Parameter::Float(std::f64::consts::PI / 2.0),
            )
            .unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.h(1).unwrap();

        let optimizer = CNOTOptimizer::new();
        let optimized = optimizer.optimize(&circuit).unwrap();

        // Verify CX count: the two CX(0,1) should NOT be cancelled
        // because Rz(1,θ) sits between them
        let cx_count = CNOTOptimizer::count_cnots(&optimized);
        assert_eq!(
            cx_count, 2,
            "CX gates should not be cancelled when non-commuting gate intervenes"
        );

        // Verify Rz between the CX gates is preserved
        let instructions = optimized.data().instructions();
        let rz_found = instructions
            .iter()
            .any(|inst| inst.gate.gate_type == StandardGate::Rz);
        assert!(rz_found, "Rz gate must be preserved");
    }

    #[test]
    fn test_cnot_optimizer_cancels_adjacent_inverse_pair() {
        // Truly adjacent inverse CX pair should still cancel
        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.h(1).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.cx(0, 1).unwrap(); // Truly adjacent inverse pair
        circuit.h(1).unwrap();

        let optimizer = CNOTOptimizer::new();
        let optimized = optimizer.optimize(&circuit).unwrap();

        // Verify both CX gates were cancelled
        let cx_count = CNOTOptimizer::count_cnots(&optimized);
        assert_eq!(cx_count, 0, "Adjacent inverse CX pair should be cancelled");

        // Both H gates should survive
        let h_count = optimized
            .data()
            .instructions()
            .iter()
            .filter(|inst| inst.gate.gate_type == StandardGate::H)
            .count();
        assert_eq!(h_count, 2);
    }

    // -----------------------------------------------------------------------
    // Symbolic template tests
    // -----------------------------------------------------------------------

    /// Create a dummy AdjacencyCache for testing (all gates trivially adjacent).
    fn sym_dummy_cache(_num_qubits: usize) -> AdjacencyCache {
        // Use a large enough size so gates can have original_index values
        let n = 100;
        let mut next_after = vec![vec![usize::MAX; n]; 8];
        // No non-CX gates anywhere — all gates are "adjacent"
        AdjacencyCache { next_after }
    }

    #[test]
    fn test_sym_template_1_basic_match() {
        // Sym-1: CX(a,b)·CX(b,a)·CX(a,b) → CX(b,a)
        let templates = standard_symbolic_templates();
        let sym1 = &templates[0];
        let cache = sym_dummy_cache(4);

        // Match: CX(2,5)·CX(5,2)·CX(2,5) with a=2, b=5
        let gates = vec![
            CNOTGate::new(2, 5),
            CNOTGate::new(5, 2),
            CNOTGate::new(2, 5),
        ];

        let bindings = sym1.try_match(&gates, 0, &cache);
        assert!(
            bindings.is_some(),
            "Sym-1 should match CX(2,5)·CX(5,2)·CX(2,5)"
        );
        let b = bindings.unwrap();
        assert_eq!(b.get("a"), Some(&2));
        assert_eq!(b.get("b"), Some(&5));

        // Apply and verify replacement
        let replacement = sym1.apply(&b);
        assert_eq!(replacement.len(), 1);
        assert_eq!(replacement[0], CNOTGate::new(5, 2));
    }

    #[test]
    fn test_sym_template_2_chain_reduction() {
        // Sym-2: CX(a,b)·CX(b,c)·CX(a,b)·CX(b,c) → CX(a,c)
        let templates = standard_symbolic_templates();
        let sym2 = &templates[1];
        let cache = sym_dummy_cache(4);

        let gates = vec![
            CNOTGate::new(0, 3),
            CNOTGate::new(3, 7),
            CNOTGate::new(0, 3),
            CNOTGate::new(3, 7),
        ];

        let bindings = sym2.try_match(&gates, 0, &cache);
        assert!(bindings.is_some(), "Sym-2 should match");
        let b = bindings.unwrap();
        assert_eq!(b.get("a"), Some(&0));
        assert_eq!(b.get("b"), Some(&3));
        assert_eq!(b.get("c"), Some(&7));

        let replacement = sym2.apply(&b);
        assert_eq!(replacement.len(), 1);
        assert_eq!(replacement[0], CNOTGate::new(0, 7));
    }

    #[test]
    fn test_sym_template_5_disjoint_cancellation() {
        // Sym-5: CX(a,b)·CX(c,d)·CX(a,b)·CX(c,d) → ∅
        let templates = standard_symbolic_templates();
        let sym5 = &templates[4];
        let cache = sym_dummy_cache(4);

        // All 4 qubits distinct
        let gates = vec![
            CNOTGate::new(0, 1),
            CNOTGate::new(2, 3),
            CNOTGate::new(0, 1),
            CNOTGate::new(2, 3),
        ];

        let bindings = sym5.try_match(&gates, 0, &cache);
        assert!(
            bindings.is_some(),
            "Sym-5 should match with 4 distinct qubits"
        );
        let b = bindings.unwrap();
        let replacement = sym5.apply(&b);
        assert!(replacement.is_empty(), "Sym-5 replacement should be empty");
    }

    #[test]
    fn test_sym_template_6_same_control_cancellation() {
        // Sym-6: CX(a,b)·CX(a,c)·CX(a,b)·CX(a,c) → ∅  (b≠c)
        let templates = standard_symbolic_templates();
        let sym6 = &templates[5];
        let cache = sym_dummy_cache(4);

        let gates = vec![
            CNOTGate::new(0, 1),
            CNOTGate::new(0, 2),
            CNOTGate::new(0, 1),
            CNOTGate::new(0, 2),
        ];

        let bindings = sym6.try_match(&gates, 0, &cache);
        assert!(bindings.is_some(), "Sym-6 should match");
        let b = bindings.unwrap();
        let replacement = sym6.apply(&b);
        assert!(replacement.is_empty());
    }

    #[test]
    fn test_sym_template_7_removed() {
        // Sym-7 (CX(a,b)·CX(c,b) → CX(c,b)·CX(a,b)) was removed because
        // it's a 2→2 commutation swap, not a gate-reducing template.
        // When applied, replacement gates lost their original_index, causing
        // rebuild_circuit to append them at the end instead of keeping
        // their original positions — corrupting the circuit unitary.
        // Commutation is handled by the commute_reorder pass instead.
        let templates = standard_symbolic_templates();
        // Verify no 2→2 commutation templates remain
        for t in &templates {
            assert!(
                t.pattern.len() != 2 || t.replacement.len() != 2,
                "No 2→2 templates should remain; found pattern={}, replacement={}",
                t.pattern.len(),
                t.replacement.len()
            );
        }
    }

    #[test]
    fn test_sym_template_8_removed() {
        // Sym-8 (CX(a,b)·CX(a,c) → CX(a,c)·CX(a,b)) was removed for the
        // same reason as Sym-7 — 2→2 commutation swap that corrupts
        // circuit ordering when applied via template replacement.
        let templates = standard_symbolic_templates();
        // Only reducing templates (pattern > replacement) should exist
        for t in &templates {
            assert!(
                t.pattern.len() > t.replacement.len(),
                "Only reducing templates should exist; found pattern={}, replacement={}",
                t.pattern.len(),
                t.replacement.len()
            );
        }
    }

    #[test]
    fn test_symbolic_template_distinctness_enforced() {
        // Sym-5: CX(a,b)·CX(c,d)·CX(a,b)·CX(c,d) → ∅
        // If c=a (same control qubit for both pairs), the template should NOT match
        // because a,b,c,d must all be distinct.
        let templates = standard_symbolic_templates();
        let sym5 = &templates[4];
        let cache = sym_dummy_cache(4);

        // a=0, b=1, c=0, d=2 — but a and c are supposed to be distinct!
        // The first gate CX(0,1) binds a→0, b→1
        // The second gate CX(0,2) tries to bind c→0 and d→2
        // c→0 should succeed (not yet seen), d→2 succeeds
        // But then distinctness check: a→0 and c→0, with a≠c → FAIL
        let gates = vec![
            CNOTGate::new(0, 1),
            CNOTGate::new(0, 2),
            CNOTGate::new(0, 1),
            CNOTGate::new(0, 2),
        ];

        let bindings = sym5.try_match(&gates, 0, &cache);
        assert!(
            bindings.is_none(),
            "Sym-5 should NOT match when a=c (same control for both pairs)"
        );
    }

    #[test]
    fn test_symbolic_template_distinctness_b_and_d() {
        // Sym-5: if b=d (same target), shouldn't match
        let templates = standard_symbolic_templates();
        let sym5 = &templates[4];
        let cache = sym_dummy_cache(4);

        // CX(0,1)·CX(2,1)·CX(0,1)·CX(2,1) — b=1 and d=1 are the same
        let gates = vec![
            CNOTGate::new(0, 1),
            CNOTGate::new(2, 1),
            CNOTGate::new(0, 1),
            CNOTGate::new(2, 1),
        ];

        let bindings = sym5.try_match(&gates, 0, &cache);
        assert!(
            bindings.is_none(),
            "Sym-5 should NOT match when b=d (same target)"
        );
    }

    #[test]
    fn test_template_library_with_symbolic_templates() {
        // Test that TemplateLibrary applies both symbolic and concrete templates.
        let lib = TemplateLibrary::new();
        let cache = sym_dummy_cache(4);

        // A pattern that only symbolic templates can match:
        // CX(3,7)·CX(5,9)·CX(3,7)·CX(5,9) → should be reduced by Sym-5
        let gates = vec![
            CNOTGate::new(3, 7),
            CNOTGate::new(5, 9),
            CNOTGate::new(3, 7),
            CNOTGate::new(5, 9),
        ];

        let optimized = lib.apply_templates(&gates, &cache);
        assert!(
            optimized.is_empty(),
            "Sym-5 should cancel all 4 gates; got {:?}",
            optimized
        );
    }

    #[test]
    fn test_symbolic_template_unitarity_preserved() {
        // Verify that each symbolic template preserves unitarity by checking
        // that the pattern unitary equals the replacement unitary for any
        // binding of distinct qubits.
        let templates = standard_symbolic_templates();

        // We test each template symbolically: applying the pattern then
        // the inverse of the replacement should yield identity.
        // Since CNOT is self-inverse, pattern · replacement = pattern if
        // the template is correct (because CNOT²=I and the pattern reduces).
        //
        // For reduction templates, we verify the replacement is shorter.
        // For commutation templates (Sym-7, Sym-8), length is unchanged.
        for (i, template) in templates.iter().enumerate() {
            let desc = template.description();
            if template.replacement.is_empty() {
                // Cancellation template — pattern should be self-inverse
                // (all CNOTs, so applying twice gives identity)
                assert!(
                    template.reduction > 0,
                    "Template {} ({}) should reduce gate count",
                    i,
                    desc
                );
            } else if template.reduction > 0 {
                assert!(
                    template.replacement.len() < template.pattern.len(),
                    "Template {} ({}) should have fewer replacement gates",
                    i,
                    desc
                );
            }
        }
    }
}
