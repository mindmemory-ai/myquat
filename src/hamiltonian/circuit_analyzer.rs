//! Circuit to Hamiltonian Analyzer
//!
//! Author: gA4ss
//!
//! This module analyzes quantum circuits and extracts the effective Hamiltonian.
//!
//! # Supported Patterns
//!
//! - **Rotation gates**: RX(θ), RY(θ), RZ(θ) → Pauli Hamiltonians
//! - **Two-qubit rotations**: RXX(θ), RYY(θ), RZZ(θ), RZX(θ) → Interaction terms
//! - **Trotter circuits**: Identify Trotter-Suzuki decomposition patterns
//! - **Parametric circuits**: Extract symbolic Hamiltonians from VQE/QAOA ansätze

use super::hamiltonian_compiler::TrotterOrder;
use super::{Hamiltonian, PauliOperator, PauliString, PauliTerm};
use crate::error::{MyQuatError, Result};
use crate::{Parameter, QuantumCircuit, StandardGate};
use num_complex::Complex64;
use std::collections::{HashMap, HashSet};

/// Gate to Hamiltonian mapping result
#[derive(Debug, Clone)]
pub struct GateHamiltonianMap {
    /// The Hamiltonian term corresponding to this gate
    pub hamiltonian_term: PauliTerm,

    /// Gate index in the circuit
    pub gate_index: usize,

    /// Whether this is part of a Trotter decomposition
    pub is_trotter_term: bool,
}

/// Circuit analysis result
#[derive(Debug)]
pub struct CircuitAnalysis {
    /// Extracted Hamiltonian
    pub hamiltonian: Hamiltonian,

    /// Number of Trotter steps identified
    pub trotter_steps: Option<usize>,

    /// Evolution time (if identified)
    pub evolution_time: Option<f64>,

    /// Gate-to-Hamiltonian mappings
    pub gate_mappings: Vec<GateHamiltonianMap>,

    /// Detected Trotter order (1st, 2nd, 4th, etc.)
    pub trotter_order: Option<TrotterOrder>,

    /// Confidence score for Trotter order detection [0.0, 1.0]
    pub order_confidence: f64,
}

/// Circuit analyzer for Hamiltonian extraction
///
/// Analyzes quantum circuits to extract the effective Hamiltonian they implement.
///
/// # Examples
///
/// ```
/// use myquat::{QuantumCircuit, Parameter};
/// use myquat::hamiltonian::CircuitAnalyzer;
///
/// let mut circuit = QuantumCircuit::new(2, 0);
/// circuit.rz(0, Parameter::Float(0.5)).unwrap();
/// circuit.rz(1, Parameter::Float(0.5)).unwrap();
///
/// let analyzer = CircuitAnalyzer::new();
/// let analysis = analyzer.analyze(&circuit).unwrap();
/// println!("Extracted Hamiltonian: {}", analysis.hamiltonian);
/// ```
pub struct CircuitAnalyzer {
    /// Enable Trotter pattern recognition
    pub recognize_trotter: bool,

    /// Minimum coefficient threshold (ignore smaller terms)
    pub coefficient_threshold: f64,

    /// Reduced Planck constant (default 1.0)
    pub hbar: f64,

    /// Known evolution time. None = unknown, fall back to angle/2 (backward compatible).
    pub evolution_time: Option<f64>,
}

impl CircuitAnalyzer {
    /// Create a new circuit analyzer with default settings
    pub fn new() -> Self {
        Self {
            recognize_trotter: true,
            coefficient_threshold: 1e-10,
            hbar: 1.0,
            evolution_time: None,
        }
    }

    /// Set the reduced Planck constant (chainable builder).
    pub fn with_hbar(mut self, hbar: f64) -> Self {
        self.hbar = hbar;
        self
    }

    /// Set a known evolution time for accurate coefficient reconstruction (chainable builder).
    ///
    /// When set, extracted coefficients use the corrected formula:
    /// `coeff = angle * hbar / (2 * evolution_time)`.
    ///
    /// When `None` (default), falls back to `angle / 2` for backward compatibility.
    pub fn with_evolution_time(mut self, t: f64) -> Self {
        self.evolution_time = Some(t);
        self
    }

    /// Create an analyzer pre-configured from compiler settings.
    ///
    /// When a `CompilerConfig` is available (e.g. in roundtrip tests), this
    /// ensures the analyzer uses the exact same `hbar` and `evolution_time`
    /// that the compiler used, giving precise coefficient reconstruction.
    pub fn from_compiler_config(config: &super::hamiltonian_compiler::CompilerConfig) -> Self {
        Self::new()
            .with_hbar(config.hbar)
            .with_evolution_time(config.evolution_time)
    }

    /// Effective evolution time: the user-specified value, or a default of 1.0
    /// which preserves the existing `angle / 2` behaviour for backward compatibility.
    fn effective_evolution_time(&self) -> f64 {
        self.evolution_time.unwrap_or(1.0)
    }

    /// Analyze a quantum circuit and extract its Hamiltonian
    ///
    /// # Arguments
    ///
    /// * `circuit` - The quantum circuit to analyze
    ///
    /// # Returns
    ///
    /// Circuit analysis including the extracted Hamiltonian
    pub fn analyze(&self, circuit: &QuantumCircuit) -> Result<CircuitAnalysis> {
        let num_qubits = circuit.num_qubits();
        let mut hamiltonian = Hamiltonian::new(num_qubits);
        let mut gate_mappings = Vec::new();

        let instructions = circuit.data().instructions();

        // Phase 1: Structured pattern extraction
        // Recognize [basis_change] [CNOT_ladder] Rz [inv_CNOT_ladder] [inv_basis_change]
        // patterns generated by HamiltonianCompiler::compile_pauli_term.
        let structured_terms = self.extract_structured_pauli_terms(instructions, num_qubits)?;

        if !structured_terms.is_empty() {
            // Structured extraction succeeded -- use it
            for (term, indices) in &structured_terms {
                for &idx in indices {
                    gate_mappings.push(GateHamiltonianMap {
                        hamiltonian_term: term.clone(),
                        gate_index: idx,
                        is_trotter_term: false,
                    });
                }
                hamiltonian.add_term(term.pauli_string.clone(), term.coefficient)?;
            }
        } else {
            // Fallback: gate-by-gate mapping (works for bare rotation circuits)
            for (gate_index, instruction) in instructions.iter().enumerate() {
                if let Some(h_term) = self.instruction_to_hamiltonian(instruction, num_qubits)? {
                    gate_mappings.push(GateHamiltonianMap {
                        hamiltonian_term: h_term.clone(),
                        gate_index,
                        is_trotter_term: false,
                    });
                    hamiltonian.add_term(h_term.pauli_string.clone(), h_term.coefficient)?;
                }
            }
        }

        // Simplify by combining like terms
        hamiltonian.simplify();

        // Try to identify Trotter patterns and order
        let (mut trotter_steps, mut evolution_time, mut trotter_order, mut order_confidence) =
            if self.recognize_trotter {
                self.identify_trotter_pattern_with_order(circuit)?
            } else {
                (None, None, None, 0.0)
            };

        // ── P2: per-step content validation ─────────────────────────────
        // Reject detections where each "step" contains too few unique Pauli
        // terms.  This catches the common false positive where individual
        // Pauli-term blocks (3-gate CX-Rz-CX) are misidentified as Trotter
        // steps by the 2nd-order palindromic-structure detector.
        if let Some(steps) = trotter_steps {
            if steps > 0 && !structured_terms.is_empty() {
                let step_size = instructions.len() / steps;
                let min_unique = (hamiltonian.num_terms() / 2).max(1);
                let mut valid = true;
                for s in 0..steps {
                    let lo = s * step_size;
                    let hi = lo + step_size;
                    let unique: HashSet<String> = structured_terms
                        .iter()
                        .filter(|(_, indices)| indices.iter().any(|&i| i >= lo && i < hi))
                        .map(|(term, _)| term.pauli_string.to_string_repr().to_string())
                        .collect();
                    if unique.len() < min_unique {
                        valid = false;
                        break;
                    }
                }
                if !valid {
                    trotter_steps = None;
                    trotter_order = None;
                    evolution_time = None;
                    order_confidence = 0.0;
                }
            }
        }

        Ok(CircuitAnalysis {
            hamiltonian,
            trotter_steps,
            evolution_time,
            gate_mappings,
            trotter_order,
            order_confidence,
        })
    }

    /// Extract structured Pauli rotation terms from the instruction list.
    ///
    /// Recognizes patterns produced by `HamiltonianCompiler::compile_pauli_term`:
    /// 1. Single-qubit Pauli rotation: `[basis] Rz(angle) [inv_basis]`
    ///    where basis = H for X, Rx(pi/2) for Y, nothing for Z
    /// 2. Multi-qubit Pauli rotation: `[basis_all] CX_ladder Rz(angle) inv_CX_ladder [inv_basis_all]`
    ///
    /// Returns a list of (PauliTerm, gate_indices) for each recognized block.
    fn extract_structured_pauli_terms(
        &self,
        instructions: &[crate::circuit::Instruction],
        num_qubits: usize,
    ) -> Result<Vec<(PauliTerm, Vec<usize>)>> {
        let n = instructions.len();
        if n == 0 {
            return Ok(Vec::new());
        }

        let mut results: Vec<(PauliTerm, Vec<usize>)> = Vec::new();
        let mut i = 0;

        while i < n {
            // Try GateLevel: one Rz = one Pauli term
            if let Some((term, end_idx)) =
                self.try_match_pauli_block(instructions, i, num_qubits)?
            {
                let indices: Vec<usize> = (i..end_idx).collect();
                results.push((term, indices));
                i = end_idx;
            } else if let Some((terms, end_idx)) =
                // Try PauliLevel: shared CNOT tree with multiple Rz gates
                self.try_match_shared_pauli_block(instructions, i, num_qubits)?
            {
                for term in terms {
                    let indices: Vec<usize> = (i..end_idx).collect();
                    results.push((term, indices));
                }
                i = end_idx;
            } else {
                // Not part of a recognized pattern -- check if it is a bare
                // rotation gate (Rx/Ry/Rz without surrounding context)
                if let Some(h_term) =
                    self.instruction_to_hamiltonian(&instructions[i], num_qubits)?
                {
                    results.push((h_term, vec![i]));
                }
                i += 1;
            }
        }

        Ok(results)
    }

    /// Try to match a single compiled Pauli-rotation block starting at position `start`.
    ///
    /// The compiler emits the following pattern for $e^{-i\theta P}$
    /// where $P = P_0 \otimes P_1 \otimes \ldots$:
    ///
    /// ```text
    /// [basis_change for each non-Z, non-I qubit]
    /// [CX ladder: CX(q0,q1), CX(q1,q2), ..., CX(q_{k-2},q_{k-1})]
    /// Rz(angle, q_{k-1})
    /// [inverse CX ladder: CX(q_{k-2},q_{k-1}), ..., CX(q0,q1)]
    /// [inverse basis_change]
    /// ```
    ///
    /// For a single active qubit, the CX ladder is absent.
    ///
    /// Returns `Some((PauliTerm, end_index))` on success, `None` if no pattern matches.
    fn try_match_pauli_block(
        &self,
        instructions: &[crate::circuit::Instruction],
        start: usize,
        num_qubits: usize,
    ) -> Result<Option<(PauliTerm, usize)>> {
        use std::f64::consts::PI;
        let n = instructions.len();

        // ---- Step 1: Collect leading basis-change gates ----
        // These are H (for X basis) or Rx(+pi/2) (for Y basis) on distinct qubits.
        let mut pos = start;
        // Map: qubit -> PauliOperator (the basis it was changed to)
        let mut basis_map: HashMap<usize, PauliOperator> = HashMap::new();

        while pos < n {
            let inst = &instructions[pos];
            if inst.gate.gate_type == crate::StandardGate::H && inst.qubits.len() == 1 {
                let q = inst.qubits[0].index();
                if basis_map.contains_key(&q) {
                    break; // already saw a basis change on this qubit
                }
                basis_map.insert(q, PauliOperator::X);
                pos += 1;
            } else if inst.gate.gate_type == crate::StandardGate::Rx && inst.qubits.len() == 1 {
                if let Some(angle) = self.try_extract_angle(&inst.gate.parameters) {
                    if (angle - PI / 2.0).abs() < 1e-6 {
                        let q = inst.qubits[0].index();
                        if basis_map.contains_key(&q) {
                            break;
                        }
                        basis_map.insert(q, PauliOperator::Y);
                        pos += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // ---- Step 2: Collect forward CX ladder ----
        let mut cx_chain: Vec<(usize, usize)> = Vec::new();
        while pos < n {
            let inst = &instructions[pos];
            if inst.gate.gate_type == crate::StandardGate::CX && inst.qubits.len() == 2 {
                let ctrl = inst.qubits[0].index();
                let tgt = inst.qubits[1].index();
                // Verify chain continuity: next CX target should be previous CX target's next qubit
                if !cx_chain.is_empty() {
                    let prev_tgt = cx_chain.last().unwrap().1;
                    if ctrl != prev_tgt {
                        break;
                    }
                }
                cx_chain.push((ctrl, tgt));
                pos += 1;
            } else {
                break;
            }
        }

        // ---- Step 3: Match the central Rz gate ----
        if pos >= n {
            return Ok(None);
        }
        let rz_inst = &instructions[pos];
        if rz_inst.gate.gate_type != crate::StandardGate::Rz || rz_inst.qubits.len() != 1 {
            // If there was no CX chain and no basis changes collected, this is not our pattern
            if cx_chain.is_empty() && basis_map.is_empty() {
                return Ok(None);
            }
            // Had some basis/CX but no Rz follows -- not a valid block.
            // Backtrack: return None so fallback handles these gates individually.
            return Ok(None);
        }
        let rz_qubit = rz_inst.qubits[0].index();
        let rz_angle = match self.try_extract_angle(&rz_inst.gate.parameters) {
            Some(a) => a,
            None => return Ok(None),
        };
        pos += 1;

        // Verify Rz qubit matches the CX chain target (last qubit in the ladder)
        if !cx_chain.is_empty() {
            let expected_rz_qubit = cx_chain.last().unwrap().1;
            if rz_qubit != expected_rz_qubit {
                return Ok(None);
            }
        }

        // ---- Step 4: Match inverse CX ladder ----
        for cx_idx in (0..cx_chain.len()).rev() {
            if pos >= n {
                return Ok(None);
            }
            let inst = &instructions[pos];
            if inst.gate.gate_type != crate::StandardGate::CX || inst.qubits.len() != 2 {
                return Ok(None);
            }
            let ctrl = inst.qubits[0].index();
            let tgt = inst.qubits[1].index();
            if ctrl != cx_chain[cx_idx].0 || tgt != cx_chain[cx_idx].1 {
                return Ok(None);
            }
            pos += 1;
        }

        // ---- Step 5: Match inverse basis-change gates ----
        // They must appear in reverse order and be the inverse of the forward gates.
        // The compiler iterates active_qubits.iter().rev() for the inverse pass.
        let mut remaining_basis: HashMap<usize, PauliOperator> = basis_map.clone();
        while pos < n && !remaining_basis.is_empty() {
            let inst = &instructions[pos];
            if inst.gate.gate_type == crate::StandardGate::H && inst.qubits.len() == 1 {
                let q = inst.qubits[0].index();
                if remaining_basis.get(&q) == Some(&PauliOperator::X) {
                    remaining_basis.remove(&q);
                    pos += 1;
                } else {
                    break;
                }
            } else if inst.gate.gate_type == crate::StandardGate::Rx && inst.qubits.len() == 1 {
                if let Some(angle) = self.try_extract_angle(&inst.gate.parameters) {
                    if (angle + PI / 2.0).abs() < 1e-6 {
                        let q = inst.qubits[0].index();
                        if remaining_basis.get(&q) == Some(&PauliOperator::Y) {
                            remaining_basis.remove(&q);
                            pos += 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // All inverse basis-change gates must have been consumed
        if !remaining_basis.is_empty() {
            return Ok(None);
        }

        // ---- Step 6: Reconstruct the Pauli string ----
        // Determine which qubits are "active" in this rotation
        let mut active_qubits: Vec<usize> = Vec::new();

        if cx_chain.is_empty() {
            // Single-qubit rotation: only the Rz qubit
            active_qubits.push(rz_qubit);
        } else {
            // Multi-qubit: all qubits in the CX chain
            active_qubits.push(cx_chain[0].0); // first control
            for &(_, tgt) in &cx_chain {
                active_qubits.push(tgt);
            }
        }

        // Build PauliString: Z for qubits without basis change,
        // X for H-sandwiched qubits, Y for Rx(pi/2)-sandwiched qubits
        let mut operators = vec![PauliOperator::I; num_qubits];
        for &q in &active_qubits {
            operators[q] = match basis_map.get(&q) {
                Some(&PauliOperator::X) => PauliOperator::X,
                Some(&PauliOperator::Y) => PauliOperator::Y,
                _ => PauliOperator::Z,
            };
        }

        let pauli_string = PauliString::new(operators, Complex64::new(1.0, 0.0));
        // The compiler generates: angle = 2 * coeff * dt / hbar
        // Gate: Rz(angle) = exp(-i * angle/2 * Z)
        // This implements: exp(-i * coeff * dt/hbar * P)
        // Therefore: coeff = angle * hbar / (2 * evolution_time)
        // See spec for derivation — the factor of evolution_time cancels
        // the multi-step summation in simplify().
        let coefficient = Complex64::new(
            rz_angle * self.hbar / (2.0 * self.effective_evolution_time()),
            0.0,
        );

        let term = PauliTerm::new(pauli_string, coefficient);
        Ok(Some((term, pos)))
    }

    /// Try to match a PauliLevel shared-CNOT-tree block.
    ///
    /// PauliLevel synthesis emits QWC blocks with a single shared CNOT tree
    /// and **multiple** Rz gates, one per commuting Pauli term:
    ///
    /// ```text
    /// [shared_basis_changes] [Rz₀?] [CX(0,1), Rz₁?] [CX(1,2), Rz₂?] ...
    ///   [inv_CX_chain] [inv_basis_changes]
    /// ```
    ///
    /// Each Rz at chain position `j` corresponds to a Pauli term whose
    /// active qubits are `chain_nodes[0..=j]` with Z on 0..j-1 and the
    /// basis operator on chain_nodes[j].
    ///
    /// Returns `Some((Vec<PauliTerm>, end_index))` on success.
    fn try_match_shared_pauli_block(
        &self,
        instructions: &[crate::circuit::Instruction],
        start: usize,
        num_qubits: usize,
    ) -> Result<Option<(Vec<PauliTerm>, usize)>> {
        use std::f64::consts::PI;
        let n = instructions.len();

        // ---- Step 1: Collect leading basis-change gates (same as GateLevel) ----
        let mut pos = start;
        let mut basis_map: HashMap<usize, PauliOperator> = HashMap::new();

        while pos < n {
            let inst = &instructions[pos];
            if inst.gate.gate_type == crate::StandardGate::H && inst.qubits.len() == 1 {
                let q = inst.qubits[0].index();
                if basis_map.contains_key(&q) {
                    break;
                }
                basis_map.insert(q, PauliOperator::X);
                pos += 1;
            } else if inst.gate.gate_type == crate::StandardGate::Rx && inst.qubits.len() == 1 {
                if let Some(angle) = self.try_extract_angle(&inst.gate.parameters) {
                    if (angle - PI / 2.0).abs() < 1e-6 {
                        let q = inst.qubits[0].index();
                        if basis_map.contains_key(&q) {
                            break;
                        }
                        basis_map.insert(q, PauliOperator::Y);
                        pos += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // ---- Step 2: Collect Rz + CX interleaved sequence ----
        // PauliLevel pattern (non-split, pos 0 not split):
        //   [Rz? on first node] [CX + Rz? pairs] [inv_CX_chain]
        // Split case also handled: Rz on pos 0 is split into two halves
        // before and after the chain.

        let mut chain_nodes: Vec<usize> = Vec::new();
        let mut rz_entries: Vec<(usize, f64)> = Vec::new(); // (qubit, angle)
        let mut cx_chain: Vec<(usize, usize)> = Vec::new();

        // Optional initial Rz on the would-be first chain node
        if pos < n {
            let inst = &instructions[pos];
            if inst.gate.gate_type == crate::StandardGate::Rz && inst.qubits.len() == 1 {
                if let Some(angle) = self.try_extract_angle(&inst.gate.parameters) {
                    let q = inst.qubits[0].index();
                    rz_entries.push((q, angle));
                    pos += 1;
                }
            }
        }

        // Now collect [CX, optional Rz] pairs — these form the forward chain
        while pos < n {
            let inst = &instructions[pos];
            if inst.gate.gate_type == crate::StandardGate::CX && inst.qubits.len() == 2 {
                let ctrl = inst.qubits[0].index();
                let tgt = inst.qubits[1].index();

                // Verify chain continuity
                if !cx_chain.is_empty() {
                    let prev_tgt = cx_chain.last().unwrap().1;
                    if ctrl != prev_tgt {
                        break;
                    }
                } else {
                    // First CX: record the initial qubit as part of the chain
                    chain_nodes.push(ctrl);
                }
                chain_nodes.push(tgt);
                cx_chain.push((ctrl, tgt));
                pos += 1;

                // Optional Rz after this CX (on the target qubit)
                if pos < n {
                    let next = &instructions[pos];
                    if next.gate.gate_type == crate::StandardGate::Rz && next.qubits.len() == 1 {
                        let q = next.qubits[0].index();
                        if q == tgt {
                            if let Some(angle) = self.try_extract_angle(&next.gate.parameters) {
                                rz_entries.push((q, angle));
                                pos += 1;
                            }
                        }
                    }
                }
            } else {
                break;
            }
        }

        // Need at least one CX in the chain to distinguish from GateLevel
        if cx_chain.is_empty() {
            return Ok(None);
        }

        // ---- Step 3: Match inverse CX chain ----
        for cx_idx in (0..cx_chain.len()).rev() {
            if pos >= n {
                return Ok(None);
            }
            let inst = &instructions[pos];
            if inst.gate.gate_type != crate::StandardGate::CX || inst.qubits.len() != 2 {
                return Ok(None);
            }
            let ctrl = inst.qubits[0].index();
            let tgt = inst.qubits[1].index();
            if ctrl != cx_chain[cx_idx].0 || tgt != cx_chain[cx_idx].1 {
                return Ok(None);
            }
            pos += 1;
        }

        // Optional trailing Rz on chain_nodes[0] (split-position-0 case)
        // Skip it — it's the other half of the split Rz at position 0.
        if pos < n {
            let inst = &instructions[pos];
            if inst.gate.gate_type == crate::StandardGate::Rz && inst.qubits.len() == 1 {
                let q = inst.qubits[0].index();
                if q == chain_nodes[0] {
                    // Skip this split-half Rz (already counted in the forward Rz)
                    pos += 1;
                }
            }
        }

        // ---- Step 4: Match inverse basis-change gates ----
        let mut remaining_basis: HashMap<usize, PauliOperator> = basis_map.clone();
        while pos < n && !remaining_basis.is_empty() {
            let inst = &instructions[pos];
            if inst.gate.gate_type == crate::StandardGate::H && inst.qubits.len() == 1 {
                let q = inst.qubits[0].index();
                if remaining_basis.get(&q) == Some(&PauliOperator::X) {
                    remaining_basis.remove(&q);
                    pos += 1;
                } else {
                    break;
                }
            } else if inst.gate.gate_type == crate::StandardGate::Rx && inst.qubits.len() == 1 {
                if let Some(angle) = self.try_extract_angle(&inst.gate.parameters) {
                    if (angle + PI / 2.0).abs() < 1e-6 {
                        let q = inst.qubits[0].index();
                        if remaining_basis.get(&q) == Some(&PauliOperator::Y) {
                            remaining_basis.remove(&q);
                            pos += 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if !remaining_basis.is_empty() {
            return Ok(None);
        }

        // ---- Step 5: Reconstruct Pauli terms for each Rz ----
        let mut terms = Vec::new();

        for &(rz_qubit, rz_angle) in &rz_entries {
            // Find position of rz_qubit in chain_nodes
            let pos_in_chain = match chain_nodes.iter().position(|&n| n == rz_qubit) {
                Some(p) => p,
                None => continue, // Rz on non-chain qubit: skip
            };

            // Pauli operators:
            // - chain_nodes[0..pos_in_chain]: Z (XOR propagation via CX)
            // - chain_nodes[pos_in_chain] = rz_qubit: basis_operator from basis_map
            // - all other qubits: I
            let mut operators = vec![PauliOperator::I; num_qubits];
            for j in 0..pos_in_chain {
                operators[chain_nodes[j]] = PauliOperator::Z;
            }
            operators[rz_qubit] = match basis_map.get(&rz_qubit) {
                Some(&PauliOperator::X) => PauliOperator::X,
                Some(&PauliOperator::Y) => PauliOperator::Y,
                _ => PauliOperator::Z,
            };

            let pauli_string = PauliString::new(operators, Complex64::new(1.0, 0.0));
            let coefficient = Complex64::new(
                rz_angle * self.hbar / (2.0 * self.effective_evolution_time()),
                0.0,
            );
            terms.push(PauliTerm::new(pauli_string, coefficient));
        }

        if terms.is_empty() {
            return Ok(None);
        }

        Ok(Some((terms, pos)))
    }

    /// Map a single instruction to its Hamiltonian representation
    ///
    /// For a rotation gate $R_P(\theta) = e^{-i\theta P/2}$,
    /// the Hamiltonian contribution is: $H = \theta/2 \cdot P$
    /// where P is the Pauli operator (X, Y, or Z)
    fn instruction_to_hamiltonian(
        &self,
        instruction: &crate::circuit::Instruction,
        num_qubits: usize,
    ) -> Result<Option<PauliTerm>> {
        match instruction.gate.gate_type {
            // Single-qubit rotation gates
            StandardGate::Rx => {
                let theta = self.extract_angle(&instruction.gate.parameters)?;
                let qubit = instruction.qubits[0].index();
                let pauli_string = PauliString::single_qubit(num_qubits, qubit, PauliOperator::X)?;

                // $R_X(\theta) = e^{-i\theta X/2}$, so $H = \theta \cdot \hbar / (2 \cdot t) \cdot X$
                let coefficient = Complex64::new(
                    theta * self.hbar / (2.0 * self.effective_evolution_time()),
                    0.0,
                );
                Ok(Some(PauliTerm::new(pauli_string, coefficient)))
            }

            StandardGate::Ry => {
                let theta = self.extract_angle(&instruction.gate.parameters)?;
                let qubit = instruction.qubits[0].index();
                let pauli_string = PauliString::single_qubit(num_qubits, qubit, PauliOperator::Y)?;

                let coefficient = Complex64::new(
                    theta * self.hbar / (2.0 * self.effective_evolution_time()),
                    0.0,
                );
                Ok(Some(PauliTerm::new(pauli_string, coefficient)))
            }

            StandardGate::Rz => {
                let theta = self.extract_angle(&instruction.gate.parameters)?;
                let qubit = instruction.qubits[0].index();
                let pauli_string = PauliString::single_qubit(num_qubits, qubit, PauliOperator::Z)?;

                let coefficient = Complex64::new(
                    theta * self.hbar / (2.0 * self.effective_evolution_time()),
                    0.0,
                );
                Ok(Some(PauliTerm::new(pauli_string, coefficient)))
            }

            // Two-qubit rotation gates (from extended gates)
            _ => {
                // Check if it's an extended gate
                if let Some(h_term) =
                    self.extended_instruction_to_hamiltonian(instruction, num_qubits)?
                {
                    Ok(Some(h_term))
                } else {
                    // Gate doesn't directly map to a Hamiltonian term
                    Ok(None)
                }
            }
        }
    }

    /// Handle extended gates (RXX, RYY, RZZ, etc.)
    fn extended_instruction_to_hamiltonian(
        &self,
        instruction: &crate::circuit::Instruction,
        num_qubits: usize,
    ) -> Result<Option<PauliTerm>> {
        // Check gate name for extended rotation gates
        let gate_name = format!("{:?}", instruction.gate.gate_type);

        if instruction.qubits.len() == 2 {
            let q0 = instruction.qubits[0].index();
            let q1 = instruction.qubits[1].index();

            // RXX, RYY, RZZ, RZX gates
            let (op0, op1) = if gate_name.contains("RXX") {
                (PauliOperator::X, PauliOperator::X)
            } else if gate_name.contains("RYY") {
                (PauliOperator::Y, PauliOperator::Y)
            } else if gate_name.contains("RZZ") {
                (PauliOperator::Z, PauliOperator::Z)
            } else if gate_name.contains("RZX") {
                (PauliOperator::Z, PauliOperator::X)
            } else {
                return Ok(None);
            };

            let theta = self.extract_angle(&instruction.gate.parameters)?;

            // Create Pauli string for two-qubit interaction
            let mut operators = vec![PauliOperator::I; num_qubits];
            operators[q0] = op0;
            operators[q1] = op1;
            let pauli_string = PauliString::new(operators, Complex64::new(1.0, 0.0));

            // $H = \theta \cdot \hbar / (2 \cdot t) \cdot (P_0 \otimes P_1)$
            let coefficient = Complex64::new(
                theta * self.hbar / (2.0 * self.effective_evolution_time()),
                0.0,
            );
            Ok(Some(PauliTerm::new(pauli_string, coefficient)))
        } else {
            Ok(None)
        }
    }

    /// Extract rotation angle from gate parameters
    fn extract_angle(&self, params: &[Parameter]) -> Result<f64> {
        if params.is_empty() {
            return Err(MyQuatError::hamiltonian_error(
                "Gate missing rotation angle parameter",
            ));
        }

        match &params[0] {
            Parameter::Float(theta) => Ok(*theta),
            Parameter::Symbol(name) => Err(MyQuatError::hamiltonian_error(format!(
                "Symbolic parameter '{}' not yet supported in analysis",
                name
            ))),
            Parameter::Expression(_) => Err(MyQuatError::hamiltonian_error(
                "Parameter expressions not yet supported in analysis",
            )),
        }
    }

    /// Identify Trotter-Suzuki decomposition patterns with order detection
    ///
    /// Detects 1st, 2nd, and 4th order Trotter-Suzuki decompositions by analyzing:
    /// - Gate sequence structure (symmetric vs asymmetric)
    /// - Rotation angle ratios (for 4th order: p_1 = 1/(4-4^{1/3}))
    /// - Pattern repetition count
    fn identify_trotter_pattern_with_order(
        &self,
        circuit: &QuantumCircuit,
    ) -> Result<(Option<usize>, Option<f64>, Option<TrotterOrder>, f64)> {
        let instructions = circuit.data().instructions();

        if instructions.len() < 2 {
            return Ok((None, None, None, 0.0));
        }

        // Try higher orders first (most specific patterns)
        // Try to detect arbitrary even order (8th, 10th, etc.) - check orders 10, 8
        for order in [10usize, 8].iter() {
            if let Some((steps, time, confidence)) =
                self.detect_nth_order_trotter(circuit, *order)?
            {
                return Ok((
                    Some(steps),
                    Some(time),
                    Some(TrotterOrder::Nth(*order)),
                    confidence,
                ));
            }
        }

        // Try to detect 6th order
        if let Some((steps, time, confidence)) = self.detect_sixth_order_trotter(circuit)? {
            return Ok((
                Some(steps),
                Some(time),
                Some(TrotterOrder::Sixth),
                confidence,
            ));
        }

        // Try to detect 4th order
        if let Some((steps, time, confidence)) = self.detect_fourth_order_trotter(circuit)? {
            return Ok((
                Some(steps),
                Some(time),
                Some(TrotterOrder::Fourth),
                confidence,
            ));
        }

        // Try to detect 2nd order (symmetric pattern)
        if let Some((steps, time, confidence)) = self.detect_second_order_trotter(circuit)? {
            return Ok((
                Some(steps),
                Some(time),
                Some(TrotterOrder::Second),
                confidence,
            ));
        }

        // Try to detect 1st order (simple repetition)
        if let Some((steps, time, confidence)) = self.detect_first_order_trotter(circuit)? {
            return Ok((
                Some(steps),
                Some(time),
                Some(TrotterOrder::First),
                confidence,
            ));
        }

        Ok((None, None, None, 0.0))
    }

    /// Detect first-order Trotter decomposition: $S_1(t) = \prod_j e^{-iH_j t}$
    ///
    /// First-order is characterized by simple sequential application of terms
    /// with identical rotation angles across Trotter steps.
    fn detect_first_order_trotter(
        &self,
        circuit: &QuantumCircuit,
    ) -> Result<Option<(usize, f64, f64)>> {
        let instructions = circuit.data().instructions();

        if instructions.len() < 2 {
            return Ok(None);
        }

        // Find smallest repeating unit
        for pattern_size in 1..=(instructions.len() / 2) {
            if instructions.len() % pattern_size != 0 {
                continue;
            }

            let num_steps = instructions.len() / pattern_size;
            if num_steps < 2 {
                continue;
            }

            // Check structural repetition (forward and reversed).
            // alternate_reverse_steps reverses term order in every other step.
            let structural_match = self.check_structural_repetition(instructions, pattern_size)
                || self.check_reversed_repetition(instructions, pattern_size);
            if !structural_match {
                continue;
            }

            // 1st-order: each step applies the same set of Pauli
            // exponentials.  Compare MULTISET of angles (sorted) so
            // that reordered terms from alternate_reverse_steps still match.
            let angles = self.extract_step_angles(instructions, pattern_size)?;
            if angles.is_empty() || angles[0].is_empty() {
                continue;
            }

            let mut ref_sorted = angles[0].clone();
            ref_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let mut is_first_order = true;
            for step_angles in &angles {
                if step_angles.len() != ref_sorted.len() {
                    is_first_order = false;
                    break;
                }
                let mut sorted = step_angles.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                for (&a, &b) in ref_sorted.iter().zip(sorted.iter()) {
                    if (a - b).abs() > self.coefficient_threshold {
                        is_first_order = false;
                        break;
                    }
                }
                if !is_first_order {
                    break;
                }
            }

            if is_first_order && !angles[0].is_empty() {
                // Estimate evolution time from first term: theta = -H_coeff * t
                // For rotation R_P(theta), we have e^{-i*theta/2*P}, so t = theta (simplified)
                let total_angle = angles[0].iter().map(|a| a.abs()).sum::<f64>();
                let evolution_time = total_angle / (num_steps as f64);

                // Confidence based on consistency
                let confidence = 0.7; // Base confidence for 1st order
                return Ok(Some((num_steps, evolution_time, confidence)));
            }
        }

        Ok(None)
    }

    /// Detect second-order Trotter decomposition: $S_2(t) = \prod_j e^{-iH_j t/2} \prod_j^{\leftarrow} e^{-iH_j t/2}$
    ///
    /// Second-order is characterized by symmetric (palindromic) gate structure
    /// within each Trotter step.
    fn detect_second_order_trotter(
        &self,
        circuit: &QuantumCircuit,
    ) -> Result<Option<(usize, f64, f64)>> {
        let instructions = circuit.data().instructions();

        if instructions.len() < 4 {
            return Ok(None);
        }

        // For 2nd order, look for palindromic structure: [A,B,C,...,C,B,A]
        // Collect ALL candidates and return the one with the largest step_size
        // (fewest steps), which is most likely the true Trotter step.
        // Small step_sizes (e.g. 3-gate single-term blocks) are false matches
        // that the post-validation will reject.
        let mut best: Option<(usize, f64, f64, usize)> = None; // (steps, time, conf, step_size)

        // Include step_size = len so the full circuit can be a single
        // 2nd-order Trotter step (the common case for 1-step circuits).
        for step_size in 2..=instructions.len() {
            if instructions.len() % step_size != 0 {
                continue;
            }

            let num_steps = instructions.len() / step_size;

            // Check if each step has palindromic structure
            let mut all_palindromic = true;
            let mut confidence_sum = 0.0;

            for step in 0..num_steps {
                let start = step * step_size;
                let step_instructions = &instructions[start..start + step_size];

                let (is_palindromic, conf) = self.check_palindromic_structure(step_instructions)?;
                if !is_palindromic {
                    all_palindromic = false;
                    break;
                }
                confidence_sum += conf;
            }

            if all_palindromic && num_steps > 0 {
                // Verify angle ratios for 2nd order (half angles at boundaries)
                let angle_confidence = self.verify_second_order_angles(instructions, step_size)?;

                // Estimate evolution time
                let angles = self.extract_step_angles(instructions, step_size)?;
                let evolution_time = if !angles.is_empty() && !angles[0].is_empty() {
                    angles[0].iter().map(|a| a.abs()).sum::<f64>() * 2.0 / (num_steps as f64)
                } else {
                    0.0
                };

                let confidence = (confidence_sum / num_steps as f64) * 0.5 + angle_confidence * 0.5;

                if confidence > 0.6 {
                    // Prefer largest step_size (fewest steps = true Trotter decomposition)
                    let is_better = match best {
                        None => true,
                        Some((_, _, _, best_sz)) => step_size > best_sz,
                    };
                    if is_better {
                        best = Some((num_steps, evolution_time, confidence, step_size));
                    }
                }
            }
        }

        if let Some((steps, time, conf, _)) = best {
            return Ok(Some((steps, time, conf)));
        }
        Ok(None)
    }

    /// Detect fourth-order Trotter decomposition
    ///
    /// Fourth-order uses the recursive formula:
    /// $S_4(t) = S_2(p_1 t) S_2(p_1 t) S_2((1-4p_1)t) S_2(p_1 t) S_2(p_1 t)$
    /// where $p_1 = 1/(4 - 4^{1/3}) \approx 0.41449...$
    fn detect_fourth_order_trotter(
        &self,
        circuit: &QuantumCircuit,
    ) -> Result<Option<(usize, f64, f64)>> {
        let instructions = circuit.data().instructions();

        // 4th order requires at least 5 S_2 blocks
        if instructions.len() < 10 {
            return Ok(None);
        }

        // p_1 coefficient for 4th order Trotter
        let p1 = 1.0 / (4.0 - 4.0_f64.powf(1.0 / 3.0)); // ~0.41449
        let p_center = 1.0 - 4.0 * p1; // ~-0.65796 (negative!)

        // Expected angle ratios in 5-fold structure: [p1, p1, 1-4p1, p1, p1]
        let expected_ratios = [p1, p1, p_center, p1, p1];

        // Try to find S_2 block size
        for s2_size in 2..=(instructions.len() / 5) {
            // Total size for one S_4 step = 5 * s2_size
            let s4_size = 5 * s2_size;

            if instructions.len() % s4_size != 0 {
                continue;
            }

            let num_s4_steps = instructions.len() / s4_size;

            // Extract angles from each of the 5 S_2 blocks
            let mut all_ratios_match = true;
            let mut total_confidence = 0.0;

            for s4_step in 0..num_s4_steps {
                let s4_start = s4_step * s4_size;

                // Get reference angle from first S_2 block
                let first_s2 = &instructions[s4_start..s4_start + s2_size];
                let ref_angles = self.extract_angles_from_block(first_s2)?;

                if ref_angles.is_empty() {
                    all_ratios_match = false;
                    break;
                }

                let ref_sum: f64 = ref_angles.iter().map(|a| a.abs()).sum();
                if ref_sum.abs() < self.coefficient_threshold {
                    continue;
                }

                // Check angle ratios across 5 S_2 blocks
                let mut block_ratios = Vec::new();
                for block_idx in 0..5 {
                    let block_start = s4_start + block_idx * s2_size;
                    let block = &instructions[block_start..block_start + s2_size];
                    let block_angles = self.extract_angles_from_block(block)?;

                    let block_sum: f64 = block_angles.iter().map(|a| a.abs()).sum();
                    // Handle sign for p_center (negative coefficient)
                    let ratio = if block_idx == 2 {
                        // Center block may have opposite sign
                        block_sum / ref_sum * p_center.signum()
                    } else {
                        block_sum / ref_sum
                    };
                    block_ratios.push(ratio);
                }

                // Compare with expected ratios
                let ratio_match = self.compare_angle_ratios(&block_ratios, &expected_ratios);
                if ratio_match < 0.7 {
                    all_ratios_match = false;
                    break;
                }
                total_confidence += ratio_match;
            }

            if all_ratios_match && num_s4_steps > 0 {
                // Also verify S_2 structure within each block
                let s2_confidence =
                    self.verify_s2_structure_in_blocks(instructions, s2_size, num_s4_steps)?;

                // Estimate evolution time
                let angles = self.extract_angles_from_block(&instructions[0..s2_size])?;
                let evolution_time = if !angles.is_empty() {
                    angles.iter().map(|a| a.abs()).sum::<f64>() / p1
                } else {
                    0.0
                };

                let confidence =
                    (total_confidence / num_s4_steps as f64) * 0.6 + s2_confidence * 0.4;

                if confidence > 0.7 {
                    return Ok(Some((num_s4_steps, evolution_time, confidence)));
                }
            }
        }

        Ok(None)
    }

    /// Detect sixth-order Trotter decomposition
    ///
    /// Sixth-order uses recursive formula:
    /// $S_6(t) = S_4(q_1 t)^2 S_4((1-4q_1)t) S_4(q_1 t)^2$
    /// where $q_1 = 1/(4 - 4^{1/5}) \approx 0.37454...$
    fn detect_sixth_order_trotter(
        &self,
        circuit: &QuantumCircuit,
    ) -> Result<Option<(usize, f64, f64)>> {
        let instructions = circuit.data().instructions();

        // 6th order requires 5 S_4 blocks, each S_4 has 5 S_2 blocks
        // Minimum: 5 * 5 * 2 = 50 gates (very small S_2)
        if instructions.len() < 50 {
            return Ok(None);
        }

        // q_1 coefficient for 6th order: 1/(4 - 4^{1/5})
        let q1 = self.compute_suzuki_coefficient(6);
        let q_center = 1.0 - 4.0 * q1;

        // p_1 coefficient for 4th order (used in inner S_4 blocks)
        let p1 = self.compute_suzuki_coefficient(4);

        // Try to find S_4 block size
        // S_4 size must be divisible by 5 (for the 5 S_2 blocks inside)
        for s4_size in (10..=(instructions.len() / 5)).filter(|s| s % 5 == 0) {
            let s6_size = 5 * s4_size;

            if instructions.len() % s6_size != 0 {
                continue;
            }

            let num_s6_steps = instructions.len() / s6_size;

            // Verify 5-fold structure at S_6 level with q_1 ratios
            let mut all_match = true;
            let mut total_confidence = 0.0;

            for s6_step in 0..num_s6_steps {
                let s6_start = s6_step * s6_size;

                // Get reference from first S_4 block
                let first_s4 = &instructions[s6_start..s6_start + s4_size];
                let ref_angles = self.extract_angles_from_block(first_s4)?;

                if ref_angles.is_empty() {
                    all_match = false;
                    break;
                }

                let ref_sum: f64 = ref_angles.iter().map(|a| a.abs()).sum();
                if ref_sum.abs() < self.coefficient_threshold {
                    continue;
                }

                // Check ratios across 5 S_4 blocks
                let mut block_ratios = Vec::new();
                let expected_ratios = [q1, q1, q_center, q1, q1];

                for (block_idx, &_expected) in expected_ratios.iter().enumerate() {
                    let block_start = s6_start + block_idx * s4_size;
                    let block = &instructions[block_start..block_start + s4_size];
                    let block_angles = self.extract_angles_from_block(block)?;

                    let block_sum: f64 = block_angles.iter().map(|a| a.abs()).sum();
                    let ratio = if block_idx == 2 {
                        block_sum / ref_sum * q_center.signum()
                    } else {
                        block_sum / ref_sum
                    };
                    block_ratios.push(ratio);
                }

                let ratio_match = self.compare_angle_ratios(&block_ratios, &expected_ratios);
                if ratio_match < 0.6 {
                    all_match = false;
                    break;
                }
                total_confidence += ratio_match;

                // Also verify inner S_4 structure (5 S_2 blocks with p_1 ratios)
                let s4_confidence = self.verify_s4_structure(
                    &instructions[s6_start..s6_start + s4_size],
                    s4_size / 5,
                    p1,
                )?;
                total_confidence += s4_confidence * 0.5;
            }

            if all_match && num_s6_steps > 0 {
                let evolution_time = self
                    .extract_angles_from_block(&instructions[0..s4_size])?
                    .iter()
                    .map(|a| a.abs())
                    .sum::<f64>()
                    / q1;

                let confidence = total_confidence / (num_s6_steps as f64 * 1.5);

                if confidence > 0.6 {
                    return Ok(Some((num_s6_steps, evolution_time, confidence)));
                }
            }
        }

        Ok(None)
    }

    /// Detect arbitrary even-order Trotter decomposition (n = 8, 10, 12, ...)
    ///
    /// Uses generalized recursive Suzuki formula:
    /// $S_{2k}(t) = S_{2k-2}(p_k t)^2 S_{2k-2}((1-4p_k)t) S_{2k-2}(p_k t)^2$
    /// where $p_k = 1/(4 - 4^{1/(2k-1)})$
    fn detect_nth_order_trotter(
        &self,
        circuit: &QuantumCircuit,
        order: usize,
    ) -> Result<Option<(usize, f64, f64)>> {
        // Only even orders >= 8 (4th and 6th have dedicated functions)
        if order < 8 || order % 2 != 0 {
            return Ok(None);
        }

        let instructions = circuit.data().instructions();

        // Estimate minimum gates: each level adds 5x multiplier
        // S_2: ~2, S_4: ~10, S_6: ~50, S_8: ~250, S_10: ~1250
        let min_gates = self.estimate_min_gates_for_order(order);
        if instructions.len() < min_gates {
            return Ok(None);
        }

        // Coefficient for this order: p_k = 1/(4 - 4^{1/(2k-1)})
        let p_k = self.compute_suzuki_coefficient(order);
        let p_center = 1.0 - 4.0 * p_k;

        // Size of S_{2k-2} block
        let inner_order = order - 2;
        let inner_block_size = self.estimate_block_size_for_order(inner_order, instructions.len());

        if inner_block_size == 0 {
            return Ok(None);
        }

        // Total size for one S_{2k} step = 5 * S_{2k-2}_size
        let step_size = 5 * inner_block_size;

        if instructions.len() % step_size != 0 {
            return Ok(None);
        }

        let num_steps = instructions.len() / step_size;
        if num_steps == 0 {
            return Ok(None);
        }

        // Verify structure
        let mut total_confidence = 0.0;
        let expected_ratios = [p_k, p_k, p_center, p_k, p_k];

        for step in 0..num_steps {
            let step_start = step * step_size;

            // Get reference from first inner block
            let first_block = &instructions[step_start..step_start + inner_block_size];
            let ref_angles = self.extract_angles_from_block(first_block)?;

            if ref_angles.is_empty() {
                return Ok(None);
            }

            let ref_sum: f64 = ref_angles.iter().map(|a| a.abs()).sum();
            if ref_sum.abs() < self.coefficient_threshold {
                continue;
            }

            // Check ratios across 5 inner blocks
            let mut block_ratios = Vec::new();
            for block_idx in 0..5 {
                let block_start = step_start + block_idx * inner_block_size;
                let block = &instructions[block_start..block_start + inner_block_size];
                let block_angles = self.extract_angles_from_block(block)?;

                let block_sum: f64 = block_angles.iter().map(|a| a.abs()).sum();
                let ratio = if block_idx == 2 {
                    block_sum / ref_sum * p_center.signum()
                } else {
                    block_sum / ref_sum
                };
                block_ratios.push(ratio);
            }

            let ratio_match = self.compare_angle_ratios(&block_ratios, &expected_ratios);
            if ratio_match < 0.5 {
                return Ok(None);
            }
            total_confidence += ratio_match;
        }

        let evolution_time = self
            .extract_angles_from_block(&instructions[0..inner_block_size])?
            .iter()
            .map(|a| a.abs())
            .sum::<f64>()
            / p_k;

        let confidence = total_confidence / num_steps as f64;

        if confidence > 0.5 {
            Ok(Some((num_steps, evolution_time, confidence)))
        } else {
            Ok(None)
        }
    }

    /// Compute Suzuki coefficient p_k for order 2k
    ///
    /// Formula: $p_k = 1/(4 - 4^{1/(2k-1)})$
    /// - Order 4: p_1 = 1/(4 - 4^{1/3}) ~ 0.41449
    /// - Order 6: p_2 = 1/(4 - 4^{1/5}) ~ 0.37454
    /// - Order 8: p_3 = 1/(4 - 4^{1/7}) ~ 0.35959
    fn compute_suzuki_coefficient(&self, order: usize) -> f64 {
        if order < 4 || order % 2 != 0 {
            return 0.5; // Default for invalid orders
        }
        let k = order / 2;
        let exponent = 1.0 / (2.0 * k as f64 - 1.0);
        1.0 / (4.0 - 4.0_f64.powf(exponent))
    }

    /// Estimate minimum number of gates for given Trotter order
    fn estimate_min_gates_for_order(&self, order: usize) -> usize {
        match order {
            2 => 2,
            4 => 10,    // 5 * S_2
            6 => 50,    // 5 * S_4 = 5 * 10
            8 => 250,   // 5 * S_6 = 5 * 50
            10 => 1250, // 5 * S_8 = 5 * 250
            _ => 5usize.pow((order / 2) as u32) * 2,
        }
    }

    /// Estimate block size for given order based on total circuit size
    fn estimate_block_size_for_order(&self, order: usize, total_size: usize) -> usize {
        let min_block = self.estimate_min_gates_for_order(order);

        // Try to find a valid block size
        for multiplier in 1..=10 {
            let block_size = min_block * multiplier;
            let step_size = 5 * block_size;
            if total_size % step_size == 0 && total_size / step_size >= 1 {
                return block_size;
            }
        }

        0 // No valid size found
    }

    /// Verify S_4 structure within a block (5 S_2 blocks with p_1 ratios)
    fn verify_s4_structure(
        &self,
        instructions: &[crate::circuit::Instruction],
        s2_size: usize,
        p1: f64,
    ) -> Result<f64> {
        if instructions.len() != 5 * s2_size {
            return Ok(0.0);
        }

        let expected_ratios = [p1, p1, 1.0 - 4.0 * p1, p1, p1];

        let first_s2 = &instructions[0..s2_size];
        let ref_angles = self.extract_angles_from_block(first_s2)?;

        if ref_angles.is_empty() {
            return Ok(0.0);
        }

        let ref_sum: f64 = ref_angles.iter().map(|a| a.abs()).sum();
        if ref_sum.abs() < self.coefficient_threshold {
            return Ok(0.5);
        }

        let mut block_ratios = Vec::new();
        for block_idx in 0..5 {
            let block_start = block_idx * s2_size;
            let block = &instructions[block_start..block_start + s2_size];
            let block_angles = self.extract_angles_from_block(block)?;

            let block_sum: f64 = block_angles.iter().map(|a| a.abs()).sum();
            let ratio = if block_idx == 2 {
                block_sum / ref_sum * (1.0 - 4.0 * p1).signum()
            } else {
                block_sum / ref_sum
            };
            block_ratios.push(ratio);
        }

        Ok(self.compare_angle_ratios(&block_ratios, &expected_ratios))
    }

    /// Check structural repetition (gate types and qubits match)
    fn check_structural_repetition(
        &self,
        instructions: &[crate::circuit::Instruction],
        pattern_size: usize,
    ) -> bool {
        let pattern = &instructions[0..pattern_size];
        let num_repetitions = instructions.len() / pattern_size;

        for rep in 1..num_repetitions {
            let start = rep * pattern_size;
            let current = &instructions[start..start + pattern_size];

            for (inst1, inst2) in pattern.iter().zip(current.iter()) {
                if inst1.gate.gate_type != inst2.gate.gate_type || inst1.qubits != inst2.qubits {
                    return false;
                }
            }
        }

        true
    }

    /// Like `check_structural_repetition` but alternates between forward
    /// and reversed comparison per block, matching `alternate_reverse_steps`
    /// where odd-indexed Trotter steps (1,3,5…) have their terms reversed.
    fn check_reversed_repetition(
        &self,
        instructions: &[crate::circuit::Instruction],
        pattern_size: usize,
    ) -> bool {
        let pattern = &instructions[0..pattern_size];
        let num_repetitions = instructions.len() / pattern_size;
        for rep in 1..num_repetitions {
            let start = rep * pattern_size;
            let current = &instructions[start..start + pattern_size];
            let is_reversed = rep % 2 == 1; // odd steps are reversed
            for (i, inst1) in pattern.iter().enumerate() {
                let inst2 = if is_reversed {
                    &current[pattern_size - 1 - i]
                } else {
                    &current[i]
                };
                if inst1.gate.gate_type != inst2.gate.gate_type || inst1.qubits != inst2.qubits {
                    return false;
                }
            }
        }
        true
    }

    /// Check if instruction sequence has palindromic (symmetric) structure
    fn check_palindromic_structure(
        &self,
        instructions: &[crate::circuit::Instruction],
    ) -> Result<(bool, f64)> {
        let n = instructions.len();
        if n < 2 {
            return Ok((false, 0.0));
        }

        let mut matches = 0;
        let mut total = 0;

        // Check symmetry: compare i-th from start with i-th from end
        for i in 0..n / 2 {
            let front = &instructions[i];
            let back = &instructions[n - 1 - i];

            total += 1;

            // Gate types and qubits should match for symmetry
            if front.gate.gate_type == back.gate.gate_type && front.qubits == back.qubits {
                // Also check if angles are similar (for 2nd order, should be t/2)
                let front_angle = self.try_extract_angle(&front.gate.parameters);
                let back_angle = self.try_extract_angle(&back.gate.parameters);

                if let (Some(fa), Some(ba)) = (front_angle, back_angle) {
                    if (fa - ba).abs() < self.coefficient_threshold * 1000.0 {
                        matches += 1;
                    }
                } else {
                    matches += 1; // Gate structure matches even if can't extract angle
                }
            }
        }

        let confidence = if total > 0 {
            matches as f64 / total as f64
        } else {
            0.0
        };

        Ok((confidence > 0.8, confidence))
    }

    /// Extract rotation angles for each Trotter step
    fn extract_step_angles(
        &self,
        instructions: &[crate::circuit::Instruction],
        step_size: usize,
    ) -> Result<Vec<Vec<f64>>> {
        let num_steps = instructions.len() / step_size;
        let mut all_angles = Vec::new();

        for step in 0..num_steps {
            let start = step * step_size;
            let step_instructions = &instructions[start..start + step_size];
            let angles = self.extract_angles_from_block(step_instructions)?;
            all_angles.push(angles);
        }

        Ok(all_angles)
    }

    /// Extract angles from a block of instructions
    fn extract_angles_from_block(
        &self,
        instructions: &[crate::circuit::Instruction],
    ) -> Result<Vec<f64>> {
        let mut angles = Vec::new();

        for inst in instructions {
            if let Some(angle) = self.try_extract_angle(&inst.gate.parameters) {
                angles.push(angle);
            }
        }

        Ok(angles)
    }

    /// Try to extract angle, returning None if not possible
    fn try_extract_angle(&self, params: &[Parameter]) -> Option<f64> {
        if params.is_empty() {
            return None;
        }

        match &params[0] {
            Parameter::Float(theta) => Some(*theta),
            _ => None,
        }
    }

    /// Verify angle ratios for 2nd order Trotter
    fn verify_second_order_angles(
        &self,
        instructions: &[crate::circuit::Instruction],
        step_size: usize,
    ) -> Result<f64> {
        // For 2nd order, interior terms have full angle, boundary terms have half
        // Structure: [t/2, t, t, ..., t, t/2] when unrolled

        let step_angles = self.extract_step_angles(instructions, step_size)?;
        if step_angles.is_empty() || step_angles[0].is_empty() {
            return Ok(0.5); // Neutral confidence if can't verify
        }

        // Check consistency across steps
        let mut consistency = 0.0;
        let mut count = 0;

        for i in 1..step_angles.len() {
            for j in 0..step_angles[i].len().min(step_angles[0].len()) {
                let ratio = if step_angles[0][j].abs() > self.coefficient_threshold {
                    step_angles[i][j] / step_angles[0][j]
                } else {
                    1.0
                };

                // For 2nd order, ratio should be close to 1.0 (same angles each step)
                if (ratio - 1.0).abs() < 0.1 {
                    consistency += 1.0;
                }
                count += 1;
            }
        }

        Ok(if count > 0 {
            consistency / count as f64
        } else {
            0.5
        })
    }

    /// Compare actual angle ratios with expected 4th order ratios
    fn compare_angle_ratios(&self, actual: &[f64], expected: &[f64]) -> f64 {
        if actual.len() != expected.len() {
            return 0.0;
        }

        let mut total_error = 0.0;
        for (a, e) in actual.iter().zip(expected.iter()) {
            // Normalize comparison
            let error = if e.abs() > self.coefficient_threshold {
                ((a / e) - 1.0).abs()
            } else if a.abs() < self.coefficient_threshold {
                0.0
            } else {
                1.0
            };
            total_error += error;
        }

        let avg_error = total_error / actual.len() as f64;
        // Convert error to confidence (0 error = 1.0 confidence)
        (1.0 - avg_error.min(1.0)).max(0.0)
    }

    /// Verify S_2 palindromic structure within 4th order blocks
    fn verify_s2_structure_in_blocks(
        &self,
        instructions: &[crate::circuit::Instruction],
        s2_size: usize,
        num_s4_steps: usize,
    ) -> Result<f64> {
        let s4_size = 5 * s2_size;
        let mut total_confidence = 0.0;
        let mut block_count = 0;

        for s4_step in 0..num_s4_steps {
            let s4_start = s4_step * s4_size;

            // Check each of the 5 S_2 blocks
            for block_idx in 0..5 {
                let block_start = s4_start + block_idx * s2_size;
                let block = &instructions[block_start..block_start + s2_size];

                let (_, conf) = self.check_palindromic_structure(block)?;
                total_confidence += conf;
                block_count += 1;
            }
        }

        Ok(if block_count > 0 {
            total_confidence / block_count as f64
        } else {
            0.0
        })
    }
}

impl Default for CircuitAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QuantumCircuit;

    #[test]
    fn test_single_qubit_rotation_extraction() {
        let mut circuit = QuantumCircuit::new(1, 0);
        circuit.rx(0, Parameter::Float(0.5)).unwrap();

        let analyzer = CircuitAnalyzer::new();
        let analysis = analyzer.analyze(&circuit).unwrap();

        assert_eq!(analysis.hamiltonian.num_terms(), 1);
        assert!(analysis.hamiltonian.is_hermitian());
    }

    #[test]
    fn test_two_qubit_circuit() {
        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.rz(0, Parameter::Float(1.0)).unwrap();
        circuit.rz(1, Parameter::Float(1.0)).unwrap();

        let analyzer = CircuitAnalyzer::new();
        let analysis = analyzer.analyze(&circuit).unwrap();

        // Should have 2 terms before simplification, or 2 after if different qubits
        assert!(analysis.hamiltonian.num_terms() >= 1);
    }

    #[test]
    fn test_hamiltonian_simplification() {
        let mut circuit = QuantumCircuit::new(1, 0);
        // Add same rotation twice
        circuit.rz(0, Parameter::Float(0.5)).unwrap();
        circuit.rz(0, Parameter::Float(0.5)).unwrap();

        let analyzer = CircuitAnalyzer::new();
        let analysis = analyzer.analyze(&circuit).unwrap();

        // Should simplify to single term with combined coefficient
        assert_eq!(analysis.hamiltonian.num_terms(), 1);
        let term = &analysis.hamiltonian.terms[0];
        // Combined: 0.5/2 + 0.5/2 = 0.5
        assert!((term.coefficient.re - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_trotter_pattern_recognition() {
        let mut circuit = QuantumCircuit::new(2, 0);

        // Create a repeating pattern (simple Trotter)
        // 4 repetitions to make pattern unambiguous
        for _ in 0..4 {
            circuit.rz(0, Parameter::Float(0.1)).unwrap();
            circuit.rz(1, Parameter::Float(0.1)).unwrap();
        }

        let analyzer = CircuitAnalyzer::new();
        let analysis = analyzer.analyze(&circuit).unwrap();

        // Should identify repetitions (4 steps of 2 gates each)
        assert!(analysis.trotter_steps.is_some());
        assert_eq!(analysis.trotter_steps, Some(4));
        assert_eq!(analysis.trotter_order, Some(TrotterOrder::First));
    }

    #[test]
    fn test_empty_circuit() {
        let circuit = QuantumCircuit::new(2, 0);

        let analyzer = CircuitAnalyzer::new();
        let analysis = analyzer.analyze(&circuit).unwrap();

        assert_eq!(analysis.hamiltonian.num_terms(), 0);
        assert_eq!(analysis.trotter_steps, None);
    }

    #[test]
    fn test_first_order_trotter_detection() {
        // First-order Trotter: S_1(t) = prod_j exp(-i H_j t)
        // Simple sequential application with identical angles
        let mut circuit = QuantumCircuit::new(2, 0);
        let dt = 0.1; // time step

        // 4 Trotter steps of first-order
        for _ in 0..4 {
            circuit.rz(0, Parameter::Float(dt)).unwrap();
            circuit.rz(1, Parameter::Float(dt * 0.5)).unwrap();
        }

        let analyzer = CircuitAnalyzer::new();
        let analysis = analyzer.analyze(&circuit).unwrap();

        assert_eq!(analysis.trotter_steps, Some(4));
        assert_eq!(analysis.trotter_order, Some(TrotterOrder::First));
        assert!(analysis.order_confidence > 0.5);
    }

    #[test]
    fn test_second_order_trotter_detection() {
        // Second-order Trotter: S_2(t) = prod_j exp(-i H_j t/2) * prod_j^<- exp(-i H_j t/2)
        // Symmetric (palindromic) structure: [A, B, B, A]
        let mut circuit = QuantumCircuit::new(2, 0);
        let dt = 0.1;

        // 2 Trotter steps of second-order (palindromic structure)
        for _ in 0..2 {
            // Forward: A, B
            circuit.rz(0, Parameter::Float(dt / 2.0)).unwrap();
            circuit.rz(1, Parameter::Float(dt / 2.0)).unwrap();
            // Backward: B, A (symmetric)
            circuit.rz(1, Parameter::Float(dt / 2.0)).unwrap();
            circuit.rz(0, Parameter::Float(dt / 2.0)).unwrap();
        }

        let analyzer = CircuitAnalyzer::new();
        let analysis = analyzer.analyze(&circuit).unwrap();

        assert!(analysis.trotter_steps.is_some());
        // Should detect symmetric structure
        if let Some(order) = analysis.trotter_order {
            assert!(order == TrotterOrder::Second || order == TrotterOrder::First);
        }
    }

    #[test]
    fn test_fourth_order_trotter_detection() {
        // Fourth-order Trotter: S_4(t) = S_2(p1*t)^2 * S_2((1-4p1)*t) * S_2(p1*t)^2
        // where p1 = 1/(4 - 4^{1/3}) ~ 0.41449
        let mut circuit = QuantumCircuit::new(1, 0);

        let p1 = 1.0 / (4.0 - 4.0_f64.powf(1.0 / 3.0));
        let p_center = 1.0 - 4.0 * p1;
        let t = 1.0; // total evolution time

        // Build one S_4 step with 5 S_2 blocks
        // Each S_2 block is palindromic: [forward, backward]
        let angles = [p1 * t, p1 * t, p_center * t, p1 * t, p1 * t];

        for &angle in &angles {
            // Simple S_2 structure: [RZ(a/2), RZ(a/2)]
            circuit.rz(0, Parameter::Float(angle / 2.0)).unwrap();
            circuit.rz(0, Parameter::Float(angle / 2.0)).unwrap();
        }

        let analyzer = CircuitAnalyzer::new();
        let analysis = analyzer.analyze(&circuit).unwrap();

        // Should detect some Trotter structure
        assert!(analysis.trotter_order.is_some() || analysis.trotter_steps.is_some());
    }

    #[test]
    fn test_fourth_order_coefficient_p1() {
        // Verify the 4th order coefficient p1 = 1/(4 - 4^{1/3})
        let p1 = 1.0 / (4.0 - 4.0_f64.powf(1.0 / 3.0));
        let p_center = 1.0 - 4.0 * p1;

        // p1 should be approximately 0.41449
        assert!((p1 - 0.41449).abs() < 0.001);

        // p_center should be negative (approximately -0.65796)
        assert!(p_center < 0.0);
        assert!((p_center - (-0.65796)).abs() < 0.001);

        // Verify sum: 2*p1 + p_center + 2*p1 = 4*p1 + (1-4*p1) = 1
        let sum = 4.0 * p1 + p_center;
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_trotter_order_enum() {
        // Test TrotterOrder enum variants (from hamiltonian_compiler)
        let first = TrotterOrder::First;
        let second = TrotterOrder::Second;
        let fourth = TrotterOrder::Fourth;
        let sixth = TrotterOrder::Sixth;
        let eighth = TrotterOrder::Nth(8);

        assert_eq!(first, TrotterOrder::First);
        assert_eq!(second, TrotterOrder::Second);
        assert_eq!(fourth, TrotterOrder::Fourth);
        assert_eq!(sixth, TrotterOrder::Sixth);
        assert!(matches!(eighth, TrotterOrder::Nth(8)));
    }

    #[test]
    fn test_circuit_analysis_fields() {
        let mut circuit = QuantumCircuit::new(1, 0);
        circuit.rz(0, Parameter::Float(0.5)).unwrap();

        let analyzer = CircuitAnalyzer::new();
        let analysis = analyzer.analyze(&circuit).unwrap();

        // Verify all fields are accessible
        let _ = &analysis.hamiltonian;
        let _ = &analysis.trotter_steps;
        let _ = &analysis.evolution_time;
        let _ = &analysis.gate_mappings;
        let _ = &analysis.trotter_order;
        let _ = &analysis.order_confidence;
    }

    #[test]
    fn test_sixth_order_coefficient() {
        // Verify 6th order coefficient: q_1 = 1/(4 - 4^{1/5})
        let q1 = 1.0 / (4.0 - 4.0_f64.powf(1.0 / 5.0));
        let q_center = 1.0 - 4.0 * q1;

        // q1 should be approximately 0.373066
        assert!((q1 - 0.373066).abs() < 0.001);

        // q_center should be negative
        assert!(q_center < 0.0);

        // Verify sum: 4*q1 + q_center = 1
        let sum = 4.0 * q1 + q_center;
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_suzuki_coefficients_all_orders() {
        // Test Suzuki coefficients for orders 4, 6, 8, 10
        // Formula: p_k = 1/(4 - 4^{1/(2k-1)}) for order 2k
        let analyzer = CircuitAnalyzer::new();

        // Order 4: k=2, exponent=1/3, p = 0.414491
        let p4 = analyzer.compute_suzuki_coefficient(4);
        assert!((p4 - 0.414491).abs() < 0.001);

        // Order 6: k=3, exponent=1/5, p = 0.373066
        let p6 = analyzer.compute_suzuki_coefficient(6);
        assert!((p6 - 0.373066).abs() < 0.001);

        // Order 8: k=4, exponent=1/7, p = 0.359585
        let p8 = analyzer.compute_suzuki_coefficient(8);
        assert!((p8 - 0.359585).abs() < 0.001);

        // Order 10: k=5, exponent=1/9, p = 0.352924
        let p10 = analyzer.compute_suzuki_coefficient(10);
        assert!((p10 - 0.352924).abs() < 0.001);

        // Verify convergence: p_k decreases as k increases
        assert!(p4 > p6);
        assert!(p6 > p8);
        assert!(p8 > p10);

        // All coefficients should satisfy: 4*p_k + (1-4*p_k) = 1
        for order in [4, 6, 8, 10] {
            let p = analyzer.compute_suzuki_coefficient(order);
            let center = 1.0 - 4.0 * p;
            let sum = 4.0 * p + center;
            assert!((sum - 1.0).abs() < 1e-10, "Order {} sum failed", order);
        }
    }

    #[test]
    fn test_estimate_min_gates() {
        let analyzer = CircuitAnalyzer::new();

        // Each order multiplies by 5
        assert_eq!(analyzer.estimate_min_gates_for_order(2), 2);
        assert_eq!(analyzer.estimate_min_gates_for_order(4), 10);
        assert_eq!(analyzer.estimate_min_gates_for_order(6), 50);
        assert_eq!(analyzer.estimate_min_gates_for_order(8), 250);
        assert_eq!(analyzer.estimate_min_gates_for_order(10), 1250);
    }

    #[test]
    fn test_sixth_order_structure() {
        // 6th order: S_6 = S_4(q1)^2 * S_4(1-4q1) * S_4(q1)^2
        // Each S_4 has 5 S_2 blocks, each S_2 has 2 gates
        // Total for one S_6: 5 * 5 * 2 = 50 gates minimum

        let analyzer = CircuitAnalyzer::new();
        let q1 = analyzer.compute_suzuki_coefficient(6);
        let p1 = analyzer.compute_suzuki_coefficient(4);

        // Build a minimal 6th order circuit
        let mut circuit = QuantumCircuit::new(1, 0);
        let t = 1.0;

        // 5 S_4 blocks with q1 ratios
        let s6_angles = [q1 * t, q1 * t, (1.0 - 4.0 * q1) * t, q1 * t, q1 * t];

        for &s4_angle in &s6_angles {
            // Each S_4 has 5 S_2 blocks with p1 ratios
            let s4_angles = [
                p1 * s4_angle.abs(),
                p1 * s4_angle.abs(),
                (1.0 - 4.0 * p1) * s4_angle.abs(),
                p1 * s4_angle.abs(),
                p1 * s4_angle.abs(),
            ];

            for &s2_angle in &s4_angles {
                // Each S_2 has 2 gates (simplified)
                circuit.rz(0, Parameter::Float(s2_angle / 2.0)).unwrap();
                circuit.rz(0, Parameter::Float(s2_angle / 2.0)).unwrap();
            }
        }

        // Circuit should have 5 * 5 * 2 = 50 gates
        assert_eq!(circuit.size(), 50);

        let analysis = analyzer.analyze(&circuit).unwrap();

        // Should detect some pattern (may be 4th or 6th order depending on confidence)
        assert!(analysis.trotter_order.is_some() || analysis.trotter_steps.is_some());
    }

    #[test]
    fn test_high_order_detection_threshold() {
        let analyzer = CircuitAnalyzer::new();

        // Verify minimum gate requirements prevent false positives
        let small_circuit = QuantumCircuit::new(1, 0);
        let analysis = analyzer.analyze(&small_circuit).unwrap();

        // Empty circuit should not detect any order
        assert!(analysis.trotter_order.is_none());
        assert!(analysis.trotter_steps.is_none());
    }

    #[test]
    fn test_roundtrip_ising_model() {
        use super::super::{
            constructors, CompilationStrategy, CompilerConfig, HamiltonianCompiler,
        };

        // Build Ising Hamiltonian: H = -J * ZZ - h * X
        let ham = constructors::ising_model(3, 1.0, 0.5).unwrap();
        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 1,
            evolution_time: 1.0,
            group_commuting_terms: false,
            auto_optimize_grouping: false,
            apply_circuit_optimization: false,
            optimization_strategy: CompilationStrategy::GateLevel,
            ..Default::default()
        };
        let compiler = HamiltonianCompiler::new(config);
        let circuit = compiler.compile(&ham).unwrap();

        let analyzer = CircuitAnalyzer::new();
        let analysis = analyzer.analyze(&circuit).unwrap();

        // Build maps for comparison
        let orig_map: std::collections::HashMap<String, f64> = ham
            .terms
            .iter()
            .map(|t| (format!("{}", t.pauli_string), t.coefficient.re))
            .collect();
        let ext_map: std::collections::HashMap<String, f64> = analysis
            .hamiltonian
            .terms
            .iter()
            .map(|t| (format!("{}", t.pauli_string), t.coefficient.re))
            .collect();

        // All original terms must be present in extracted
        for (label, &orig_c) in &orig_map {
            let ext_c = ext_map
                .get(label)
                .unwrap_or_else(|| panic!("Missing term: {}", label));
            let error = (orig_c - ext_c).abs() / orig_c.abs().max(1e-12);
            assert!(
                error < 1e-6,
                "Term {} coeff mismatch: orig={}, ext={}",
                label,
                orig_c,
                ext_c
            );
        }
        // No spurious terms
        assert_eq!(ext_map.len(), orig_map.len());
    }

    #[test]
    fn test_roundtrip_heisenberg_model() {
        use super::super::{
            constructors, CompilationStrategy, CompilerConfig, HamiltonianCompiler,
        };

        // Heisenberg model: XX + YY + ZZ interactions
        let ham = constructors::heisenberg_model(3, 1.0, 0.8, 0.5).unwrap();
        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 1,
            evolution_time: 1.0,
            group_commuting_terms: false,
            auto_optimize_grouping: false,
            apply_circuit_optimization: false,
            optimization_strategy: CompilationStrategy::GateLevel,
            ..Default::default()
        };
        let compiler = HamiltonianCompiler::new(config);
        let circuit = compiler.compile(&ham).unwrap();

        let analyzer = CircuitAnalyzer::new();
        let analysis = analyzer.analyze(&circuit).unwrap();

        let orig_map: std::collections::HashMap<String, f64> = ham
            .terms
            .iter()
            .map(|t| (format!("{}", t.pauli_string), t.coefficient.re))
            .collect();
        let ext_map: std::collections::HashMap<String, f64> = analysis
            .hamiltonian
            .terms
            .iter()
            .map(|t| (format!("{}", t.pauli_string), t.coefficient.re))
            .collect();

        for (label, &orig_c) in &orig_map {
            let ext_c = ext_map
                .get(label)
                .unwrap_or_else(|| panic!("Missing term: {}", label));
            let error = (orig_c - ext_c).abs() / orig_c.abs().max(1e-12);
            assert!(
                error < 1e-6,
                "Term {} coeff mismatch: orig={}, ext={}",
                label,
                orig_c,
                ext_c
            );
        }
        assert_eq!(ext_map.len(), orig_map.len());
    }

    #[test]
    fn test_roundtrip_multi_step_trotter() {
        use super::super::{
            constructors, CompilationStrategy, CompilerConfig, HamiltonianCompiler,
        };

        // Multi-step Trotter: coefficients should sum correctly
        let ham = constructors::ising_model(3, 1.0, 0.5).unwrap();
        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 4,
            evolution_time: 1.0,
            group_commuting_terms: false,
            auto_optimize_grouping: false,
            apply_circuit_optimization: false,
            optimization_strategy: CompilationStrategy::GateLevel,
            alternate_reverse_steps: false,
            clifford_enhanced_blocks: false, // disable for deterministic round-trip analysis
            ..Default::default()
        };
        let compiler = HamiltonianCompiler::new(config);
        let circuit = compiler.compile(&ham).unwrap();

        let analyzer = CircuitAnalyzer::new();
        let analysis = analyzer.analyze(&circuit).unwrap();

        let orig_map: std::collections::HashMap<String, f64> = ham
            .terms
            .iter()
            .map(|t| (format!("{}", t.pauli_string), t.coefficient.re))
            .collect();
        let ext_map: std::collections::HashMap<String, f64> = analysis
            .hamiltonian
            .terms
            .iter()
            .map(|t| (format!("{}", t.pauli_string), t.coefficient.re))
            .collect();

        for (label, &orig_c) in &orig_map {
            let ext_c = ext_map
                .get(label)
                .unwrap_or_else(|| panic!("Missing term: {}", label));
            let error = (orig_c - ext_c).abs() / orig_c.abs().max(1e-12);
            assert!(
                error < 1e-6,
                "Term {} coeff mismatch: orig={}, ext={}",
                label,
                orig_c,
                ext_c
            );
        }
        assert_eq!(ext_map.len(), orig_map.len());
    }

    #[test]
    fn test_effective_evolution_time_default() {
        let analyzer = CircuitAnalyzer::new();
        assert_eq!(analyzer.effective_evolution_time(), 1.0);
        assert_eq!(analyzer.hbar, 1.0);
        assert!(analyzer.evolution_time.is_none());
    }

    #[test]
    fn test_coefficient_backward_compat() {
        // Without evolution_time set, behavior must match old formula (angle/2).
        let mut circuit = QuantumCircuit::new(1, 0);
        circuit.rz(0, Parameter::Float(1.0)).unwrap();

        let analyzer = CircuitAnalyzer::new(); // no evolution_time → defaults to 1.0
        let analysis = analyzer.analyze(&circuit).unwrap();
        // Rz(1.0) on qubit 0 → coeff = 1.0 * 1.0 / (2 * 1.0) = 0.5
        let term = &analysis.hamiltonian.terms[0];
        let coeff = term.coefficient.re;
        assert!(
            (coeff - 0.5).abs() < 1e-10,
            "Backward compat: expected 0.5, got {}",
            coeff
        );
    }

    #[test]
    fn test_coefficient_with_evolution_time() {
        use crate::hamiltonian::{
            CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliOperator, PauliString,
            TrotterOrder,
        };
        use num_complex::Complex64;

        let mut h = Hamiltonian::new(2);
        let ps = PauliString::new(
            vec![PauliOperator::Z, PauliOperator::I],
            Complex64::new(1.0, 0.0),
        );
        h.add_term(ps, Complex64::new(1.5, 0.0)).unwrap();

        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 2,
            evolution_time: 2.0,
            group_commuting_terms: false,
            auto_optimize_grouping: false,
            apply_circuit_optimization: false,
            ..Default::default()
        };
        let compiler = HamiltonianCompiler::new(config);
        let circuit = compiler.compile(&h).unwrap();

        // Extract WITH evolution_time=2.0 → should recover coeff=1.5
        let analyzer = CircuitAnalyzer::new().with_evolution_time(2.0);
        let analysis = analyzer.analyze(&circuit).unwrap();

        let term = &analysis.hamiltonian.terms[0];
        let err = (term.coefficient.re - 1.5).abs();
        assert!(
            err < 1e-10,
            "With evolution_time=2.0: expected coeff=1.5, got {}, error={}",
            term.coefficient.re,
            err
        );
    }

    #[test]
    fn test_coefficient_with_hbar() {
        let mut circuit = QuantumCircuit::new(1, 0);
        circuit.rz(0, Parameter::Float(2.0)).unwrap();

        let analyzer = CircuitAnalyzer::new().with_hbar(0.5);
        let analysis = analyzer.analyze(&circuit).unwrap();
        // Rz(2.0), hbar=0.5, t=1.0 (default): coeff = 2.0 * 0.5 / 2.0 = 0.5
        let term = &analysis.hamiltonian.terms[0];
        let coeff = term.coefficient.re;
        assert!(
            (coeff - 0.5).abs() < 1e-10,
            "hbar=0.5: expected coeff=0.5, got {}",
            coeff
        );
    }

    #[test]
    fn test_pauli_level_same_active_set() {
        // PauliLevel shared tree for terms with the SAME active qubit set.
        // XX + YY on qubits (0,1): both terms have active=[0,1].
        // PauliLevel emits:
        //   H(0), H(1)  [basis change to X-basis for XX]
        //   Y-basis for YY would be Rx(pi/2,0), Rx(pi/2,1)
        //   Then shared tree with Rz on qubit 1 for each term.
        //
        // Simpler test: two ZZ terms on active=[0,1] with same range:
        //   Z₀Z₁, Z₀Z₁ (same Pauli, different coefficients, merged in Rz).
        // After PauliLevel compilation, they merge into one Rz.
        //
        // Test: shared tree with one Rz = one term.
        let mut circuit = QuantumCircuit::new(2, 0);
        let coeff = 1.5;
        circuit.cx(0, 1).unwrap();
        circuit.rz(1, Parameter::Float(2.0 * coeff)).unwrap();
        circuit.cx(0, 1).unwrap();

        let analyzer = CircuitAnalyzer::new().with_evolution_time(1.0);
        let analysis = analyzer.analyze(&circuit).unwrap();

        // GateLevel should match this (it's just CX-Rz-CX).
        // Verify extraction works either way.
        assert!(analysis.hamiltonian.num_terms() >= 1);
        let term = &analysis.hamiltonian.terms[0];
        assert!(
            (term.coefficient.re - coeff).abs() < 1e-10,
            "Expected coeff={}, got {}",
            coeff,
            term.coefficient.re
        );
    }

    #[test]
    fn test_pauli_level_with_x_basis() {
        // PauliLevel shared tree with basis change:
        // XX on qubits (0,1): H(0), H(1), CX(0,1), Rz(2*coeff,1), CX(0,1), H(1), H(0)
        let mut circuit = QuantumCircuit::new(2, 0);
        let coeff = 0.8;
        circuit.h(0).unwrap();
        circuit.h(1).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.rz(1, Parameter::Float(2.0 * coeff)).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.h(1).unwrap();
        circuit.h(0).unwrap();

        let analyzer = CircuitAnalyzer::new().with_evolution_time(1.0);
        let analysis = analyzer.analyze(&circuit).unwrap();

        assert!(analysis.hamiltonian.num_terms() >= 1);
        // Should extract XX with coefficient 0.8
        let xxyy: Vec<_> = analysis
            .hamiltonian
            .terms
            .iter()
            .filter(|t| format!("{}", t.pauli_string).contains("XX"))
            .collect();
        assert!(!xxyy.is_empty(), "Missing XX term");
        assert!((xxyy[0].coefficient.re - coeff).abs() < 1e-10);
    }
}
