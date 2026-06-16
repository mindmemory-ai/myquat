// deoptimization/vqe_templates.rs - VQE ansatz template library and recognition
//
// Author: gA4ss
//
// Provides template definitions and pattern recognition for Variational Quantum
// Eigensolver (VQE) ansatz circuits, including Hardware Efficient Ansatz (HEA)
// and UCCSD ansatz types. This module enables the deoptimization pipeline to
// identify and restore VQE circuit structures from optimized forms.

use super::DeoptStrategy;
use crate::circuit::QuantumCircuit;
use crate::error::{MyQuatError, Result};
use crate::gates::StandardGate;
use crate::parameter::Parameter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Data Structures
// ---------------------------------------------------------------------------

/// Entanglement pattern used in VQE ansatz layers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntanglementPattern {
    /// Linear: CNOT(i, i+1) for consecutive qubits
    Linear,
    /// Full: all-to-all CNOT connections
    Full,
    /// Circular: linear + wrap-around CNOT(n-1, 0)
    Circular,
    /// Custom pairs
    Custom(Vec<(usize, usize)>),
}

/// VQE ansatz type classification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VQEType {
    /// Hardware Efficient Ansatz with rotation + entanglement layers
    HardwareEfficient,
    /// Unitary Coupled Cluster Singles and Doubles
    UCCSD,
    /// Real amplitudes ansatz (RY + CX only)
    RealAmplitudes,
    /// EfficientSU2 ansatz
    EfficientSU2,
    /// Unknown VQE variant
    Unknown,
}

/// Rotation gate type used in ansatz layers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RotationType {
    RY,
    RZ,
    RX,
}

impl RotationType {
    fn matches_gate(&self, gate: StandardGate) -> bool {
        match self {
            RotationType::RY => gate == StandardGate::Ry,
            RotationType::RZ => gate == StandardGate::Rz,
            RotationType::RX => gate == StandardGate::Rx,
        }
    }
}

/// A single layer in a VQE ansatz circuit
#[derive(Debug, Clone, PartialEq)]
pub enum AnsatzLayer {
    /// Rotation layer: parallel single-qubit rotations on all qubits
    Rotation {
        rotation_type: RotationType,
        /// Parameter values (one per qubit)
        params: Vec<f64>,
    },
    /// Entanglement layer: two-qubit entangling gates
    Entanglement {
        pattern: EntanglementPattern,
        gate: StandardGate,
    },
}

/// Hardware Efficient Ansatz template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HEATemplate {
    /// Number of qubits
    pub num_qubits: usize,
    /// Number of repetition layers (each layer = rotation + entanglement)
    pub num_layers: usize,
    /// Rotation gate types per sub-layer (e.g. [RY, RZ] means each layer has RY then RZ)
    pub rotation_gates: Vec<RotationType>,
    /// Entanglement pattern
    pub entanglement: EntanglementPattern,
    /// Entangling gate type (usually CX)
    pub entangling_gate: StandardGate,
}

impl HEATemplate {
    /// Create a standard RealAmplitudes template (RY + linear CX)
    pub fn real_amplitudes(num_qubits: usize, num_layers: usize) -> Self {
        Self {
            num_qubits,
            num_layers,
            rotation_gates: vec![RotationType::RY],
            entanglement: EntanglementPattern::Linear,
            entangling_gate: StandardGate::CX,
        }
    }

    /// Create a standard EfficientSU2 template (RY + RZ + linear CX)
    pub fn efficient_su2(num_qubits: usize, num_layers: usize) -> Self {
        Self {
            num_qubits,
            num_layers,
            rotation_gates: vec![RotationType::RY, RotationType::RZ],
            entanglement: EntanglementPattern::Linear,
            entangling_gate: StandardGate::CX,
        }
    }

    /// Expected number of gates in a circuit matching this template
    pub fn expected_gate_count(&self) -> usize {
        let rotation_per_layer = self.rotation_gates.len() * self.num_qubits;
        let entangling_per_layer = match &self.entanglement {
            EntanglementPattern::Linear => self.num_qubits.saturating_sub(1),
            EntanglementPattern::Full => self.num_qubits * (self.num_qubits - 1) / 2,
            EntanglementPattern::Circular => self.num_qubits,
            EntanglementPattern::Custom(pairs) => pairs.len(),
        };
        self.num_layers * (rotation_per_layer + entangling_per_layer)
    }
}

/// Single excitation operator for UCCSD: $a^\dagger_i a_j - a^\dagger_j a_i$
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleExcitation {
    /// Occupied orbital index
    pub occupied: usize,
    /// Virtual orbital index
    pub virtual_idx: usize,
    /// Variational parameter value
    pub parameter: f64,
}

/// Double excitation operator for UCCSD: $a^\dagger_i a^\dagger_j a_k a_l$
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoubleExcitation {
    /// Occupied orbital indices
    pub occupied: (usize, usize),
    /// Virtual orbital indices
    pub virtual_idx: (usize, usize),
    /// Variational parameter value
    pub parameter: f64,
}

/// UCCSD Ansatz template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UCCSDTemplate {
    /// Number of qubits (spin-orbitals)
    pub num_qubits: usize,
    /// Number of electrons
    pub num_electrons: usize,
    /// Single excitation operators
    pub single_excitations: Vec<SingleExcitation>,
    /// Double excitation operators
    pub double_excitations: Vec<DoubleExcitation>,
}

impl UCCSDTemplate {
    /// Create a minimal UCCSD template for a given system
    pub fn new(num_qubits: usize, num_electrons: usize) -> Self {
        let mut singles = Vec::new();
        let mut doubles = Vec::new();

        // Generate all single excitations: occupied -> virtual
        for occ in 0..num_electrons {
            for virt in num_electrons..num_qubits {
                singles.push(SingleExcitation {
                    occupied: occ,
                    virtual_idx: virt,
                    parameter: 0.0,
                });
            }
        }

        // Generate all double excitations
        for i in 0..num_electrons {
            for j in (i + 1)..num_electrons {
                for a in num_electrons..num_qubits {
                    for b in (a + 1)..num_qubits {
                        doubles.push(DoubleExcitation {
                            occupied: (i, j),
                            virtual_idx: (a, b),
                            parameter: 0.0,
                        });
                    }
                }
            }
        }

        Self {
            num_qubits,
            num_electrons,
            single_excitations: singles,
            double_excitations: doubles,
        }
    }
}

/// Matched VQE template result
#[derive(Debug, Clone)]
pub struct VQEMatchResult {
    /// Detected VQE type
    pub vqe_type: VQEType,
    /// Number of qubits
    pub num_qubits: usize,
    /// Number of detected ansatz layers
    pub num_layers: usize,
    /// Rotation gate types detected
    pub rotation_types: Vec<RotationType>,
    /// Entanglement pattern detected
    pub entanglement: EntanglementPattern,
    /// Detected layer structure
    pub layers: Vec<AnsatzLayer>,
    /// Confidence score for this match
    pub confidence: f64,
}

// ---------------------------------------------------------------------------
// VQE Pattern Detection
// ---------------------------------------------------------------------------

/// Detect VQE ansatz patterns in a quantum circuit
pub fn detect_vqe_pattern(circuit: &QuantumCircuit) -> Option<VQEMatchResult> {
    let instructions = circuit.data().instructions();
    if instructions.is_empty() {
        return None;
    }

    // Classify every gate
    let gate_infos: Vec<GateInfo> = instructions
        .iter()
        .enumerate()
        .map(|(idx, inst)| {
            let qubits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();
            GateInfo {
                index: idx,
                gate_type: inst.gate.gate_type,
                qubits,
                param: inst.gate.parameters.first().and_then(|p| match p {
                    Parameter::Float(v) => Some(*v),
                    _ => None,
                }),
            }
        })
        .collect();

    // Try to detect layered structure
    let layers = detect_layers(&gate_infos, circuit.num_qubits());

    if layers.is_empty() {
        return None;
    }

    // Analyse detected layers
    let (rotation_layers, entangle_layers) = count_layer_types(&layers);

    if rotation_layers == 0 {
        return None;
    }

    // Require at least one entanglement layer — pure rotation circuits
    // (e.g. Trotter steps) are not VQE ansätze.
    if entangle_layers == 0 {
        return None;
    }

    // Determine VQE type
    let rotation_types = collect_rotation_types(&layers);
    let entanglement = detect_entanglement_pattern(&layers, circuit.num_qubits());
    let vqe_type = classify_vqe_type(&rotation_types, &entanglement);

    // Calculate confidence
    let confidence = calculate_vqe_confidence(
        &layers,
        &rotation_types,
        &entanglement,
        circuit.num_qubits(),
        circuit.size(),
    );

    if confidence < 0.78 {
        return None;
    }

    let num_layers = rotation_layers.min(entangle_layers + 1);

    Some(VQEMatchResult {
        vqe_type,
        num_qubits: circuit.num_qubits(),
        num_layers,
        rotation_types,
        entanglement,
        layers,
        confidence,
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct GateInfo {
    index: usize,
    gate_type: StandardGate,
    qubits: Vec<usize>,
    param: Option<f64>,
}

fn is_rotation_gate(g: StandardGate) -> bool {
    matches!(g, StandardGate::Rx | StandardGate::Ry | StandardGate::Rz)
}

fn is_entangling_gate(g: StandardGate) -> bool {
    matches!(
        g,
        StandardGate::CX | StandardGate::CZ | StandardGate::CY | StandardGate::Swap
    )
}

/// Split the gate list into consecutive layers of the same category.
///
/// Handles two common VQE patterns:
///   (a) Grouped: RY RY RY RY RZ RZ RZ RZ CX CX CX
///   (b) Interleaved: RY RZ RY RZ RY RZ RY RZ CX CX CX
/// Both are split into per-type rotation layers followed by an entanglement layer.
fn detect_layers(gates: &[GateInfo], num_qubits: usize) -> Vec<AnsatzLayer> {
    let mut layers: Vec<AnsatzLayer> = Vec::new();
    let mut i = 0;

    while i < gates.len() {
        if is_rotation_gate(gates[i].gate_type) {
            // Collect ALL consecutive rotation gates (may be mixed types)
            let start = i;
            while i < gates.len() && is_rotation_gate(gates[i].gate_type) {
                i += 1;
            }
            // Group collected rotations by type (preserving order of first occurrence)
            let block = &gates[start..i];
            let mut type_order: Vec<StandardGate> = Vec::new();
            let mut type_params: HashMap<StandardGate, Vec<f64>> = HashMap::new();
            for g in block {
                if !type_params.contains_key(&g.gate_type) {
                    type_order.push(g.gate_type);
                }
                type_params
                    .entry(g.gate_type)
                    .or_default()
                    .push(g.param.unwrap_or(0.0));
            }
            for gt in &type_order {
                let rot_type = match gt {
                    StandardGate::Ry => RotationType::RY,
                    StandardGate::Rz => RotationType::RZ,
                    StandardGate::Rx => RotationType::RX,
                    _ => continue,
                };
                let params = type_params.remove(gt).unwrap_or_default();
                layers.push(AnsatzLayer::Rotation {
                    rotation_type: rot_type,
                    params,
                });
            }
        } else if is_entangling_gate(gates[i].gate_type) {
            // Collect consecutive entangling gates
            let _start = i;
            let ent_gate = gates[i].gate_type;
            let mut pairs: Vec<(usize, usize)> = Vec::new();
            while i < gates.len() && is_entangling_gate(gates[i].gate_type) {
                if gates[i].qubits.len() == 2 {
                    pairs.push((gates[i].qubits[0], gates[i].qubits[1]));
                }
                i += 1;
            }
            let pattern = infer_entanglement_pattern(&pairs, num_qubits);
            layers.push(AnsatzLayer::Entanglement {
                pattern,
                gate: ent_gate,
            });
        } else {
            // Skip other gates (H, S, T, etc.)
            i += 1;
        }
    }

    layers
}

/// Infer entanglement pattern from observed CNOT pairs
fn infer_entanglement_pattern(pairs: &[(usize, usize)], num_qubits: usize) -> EntanglementPattern {
    if pairs.is_empty() {
        return EntanglementPattern::Linear;
    }

    // Check linear pattern: (0,1), (1,2), ..., (n-2, n-1)
    let linear_pairs: Vec<(usize, usize)> = (0..num_qubits.saturating_sub(1))
        .map(|i| (i, i + 1))
        .collect();
    if pairs == linear_pairs.as_slice() {
        return EntanglementPattern::Linear;
    }

    // Check circular: linear + (n-1, 0)
    let mut circular_pairs = linear_pairs.clone();
    if num_qubits > 1 {
        circular_pairs.push((num_qubits - 1, 0));
    }
    if pairs == circular_pairs.as_slice() {
        return EntanglementPattern::Circular;
    }

    // Check full connectivity
    let mut full_pairs: Vec<(usize, usize)> = Vec::new();
    for i in 0..num_qubits {
        for j in (i + 1)..num_qubits {
            full_pairs.push((i, j));
        }
    }
    if pairs.len() == full_pairs.len() {
        return EntanglementPattern::Full;
    }

    EntanglementPattern::Custom(pairs.to_vec())
}

fn count_layer_types(layers: &[AnsatzLayer]) -> (usize, usize) {
    let mut rot = 0;
    let mut ent = 0;
    for l in layers {
        match l {
            AnsatzLayer::Rotation { .. } => rot += 1,
            AnsatzLayer::Entanglement { .. } => ent += 1,
        }
    }
    (rot, ent)
}

fn collect_rotation_types(layers: &[AnsatzLayer]) -> Vec<RotationType> {
    let mut types = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for l in layers {
        if let AnsatzLayer::Rotation { rotation_type, .. } = l {
            if seen.insert(*rotation_type) {
                types.push(*rotation_type);
            }
        }
    }
    types
}

fn detect_entanglement_pattern(layers: &[AnsatzLayer], _num_qubits: usize) -> EntanglementPattern {
    for l in layers {
        if let AnsatzLayer::Entanglement { pattern, .. } = l {
            return pattern.clone();
        }
    }
    EntanglementPattern::Linear
}

fn classify_vqe_type(
    rotation_types: &[RotationType],
    _entanglement: &EntanglementPattern,
) -> VQEType {
    if rotation_types.len() == 1 && rotation_types[0] == RotationType::RY {
        VQEType::RealAmplitudes
    } else if rotation_types.len() == 2
        && rotation_types.contains(&RotationType::RY)
        && rotation_types.contains(&RotationType::RZ)
    {
        VQEType::EfficientSU2
    } else if rotation_types.len() >= 2 {
        VQEType::HardwareEfficient
    } else {
        VQEType::Unknown
    }
}

/// Calculate VQE detection confidence score [0.0, 1.0]
fn calculate_vqe_confidence(
    layers: &[AnsatzLayer],
    _rotation_types: &[RotationType],
    entanglement: &EntanglementPattern,
    num_qubits: usize,
    total_gates: usize,
) -> f64 {
    let (rot_count, ent_count) = count_layer_types(layers);

    // 1. Layer alternation score (40%)
    // VQE alternates rotation and entanglement layers
    let alternation = if rot_count > 0 && ent_count > 0 {
        check_alternation(layers)
    } else if rot_count > 0 {
        0.4 // rotation-only still somewhat VQE-like
    } else {
        0.0
    };

    // 2. Rotation coverage score (25%)
    // Each rotation layer should cover most/all qubits
    let coverage = rotation_coverage_score(layers, num_qubits);

    // 3. Entanglement regularity score (20%)
    let regularity = match entanglement {
        EntanglementPattern::Linear => 1.0,
        EntanglementPattern::Full => 0.9,
        EntanglementPattern::Circular => 0.95,
        EntanglementPattern::Custom(_) => 0.6,
    };

    // 4. Structure consistency (15%)
    // Gate count should be roughly consistent with a VQE template
    let consistency = if total_gates > 0 {
        let expected = rot_count as f64 * num_qubits as f64
            + ent_count as f64 * num_qubits.saturating_sub(1) as f64;
        let ratio = total_gates as f64 / expected.max(1.0);
        if (0.7..=1.5).contains(&ratio) {
            1.0
        } else {
            (1.0 - (ratio - 1.0).abs()).max(0.0)
        }
    } else {
        0.0
    };

    0.40 * alternation + 0.25 * coverage + 0.20 * regularity + 0.15 * consistency
}

/// Check whether layers strictly alternate between rotation and entanglement
fn check_alternation(layers: &[AnsatzLayer]) -> f64 {
    if layers.len() <= 1 {
        return 0.5;
    }
    let mut correct = 0usize;
    let total = layers.len() - 1;
    for w in layers.windows(2) {
        let types_differ = matches!(
            (&w[0], &w[1]),
            (
                AnsatzLayer::Rotation { .. },
                AnsatzLayer::Entanglement { .. }
            ) | (
                AnsatzLayer::Entanglement { .. },
                AnsatzLayer::Rotation { .. }
            )
        );
        if types_differ {
            correct += 1;
        }
    }
    if total > 0 {
        correct as f64 / total as f64
    } else {
        0.5
    }
}

/// How well the rotation layers cover the qubit register
fn rotation_coverage_score(layers: &[AnsatzLayer], num_qubits: usize) -> f64 {
    if num_qubits == 0 {
        return 0.0;
    }
    let mut scores: Vec<f64> = Vec::new();
    for l in layers {
        if let AnsatzLayer::Rotation { params, .. } = l {
            scores.push(params.len() as f64 / num_qubits as f64);
        }
    }
    if scores.is_empty() {
        return 0.0;
    }
    scores.iter().sum::<f64>() / scores.len() as f64
}

// ---------------------------------------------------------------------------
// VQE Restoration Strategy
// ---------------------------------------------------------------------------

/// Strategy for identifying VQE ansatz structures and restoring them
///
/// Detects Hardware Efficient Ansatz (HEA), RealAmplitudes, EfficientSU2
/// and UCCSD patterns in optimized circuits.
///
/// VQE circuits have a characteristic layered structure:
/// ```text
/// Layer 1: RY(theta_1) RY(theta_2) ... RY(theta_n)
/// Layer 2: CX(0,1) CX(1,2) ... CX(n-2,n-1)
/// Layer 3: RY(theta_{n+1}) ...
/// Layer 4: CX(0,1) ...
/// ...
/// ```
///
/// The strategy recognises this alternating pattern and reports high
/// confidence when the circuit matches.
#[derive(Debug, Clone)]
pub struct VqeRestorationStrategy {
    /// Minimum confidence to attempt restoration
    min_confidence: f64,
    /// Known HEA templates to match against
    hea_templates: Vec<HEATemplate>,
}

impl VqeRestorationStrategy {
    /// Create new strategy with sensible defaults
    pub fn new() -> Self {
        Self {
            min_confidence: 0.3, // Lower threshold for VQE (was 0.5 for others)
            hea_templates: Self::default_templates(),
        }
    }

    /// Build a set of commonly used VQE templates
    fn default_templates() -> Vec<HEATemplate> {
        let mut templates = Vec::new();

        // RealAmplitudes variants (2-8 qubits, 1-5 layers)
        for nq in 2..=8 {
            for nl in 1..=5 {
                templates.push(HEATemplate::real_amplitudes(nq, nl));
            }
        }

        // EfficientSU2 variants
        for nq in 2..=8 {
            for nl in 1..=5 {
                templates.push(HEATemplate::efficient_su2(nq, nl));
            }
        }

        templates
    }

    /// Set minimum confidence threshold
    pub fn with_min_confidence(mut self, c: f64) -> Self {
        self.min_confidence = c;
        self
    }

    /// Add a custom HEA template
    pub fn add_template(mut self, tpl: HEATemplate) -> Self {
        self.hea_templates.push(tpl);
        self
    }

    /// Try to find the best matching template for a circuit
    fn find_best_template(&self, circuit: &QuantumCircuit) -> Option<(VQEMatchResult, f64)> {
        let result = detect_vqe_pattern(circuit)?;
        if result.confidence < self.min_confidence {
            return None;
        }

        // Check template library for an exact-ish match
        let mut best_extra = 0.0f64;
        for tpl in &self.hea_templates {
            if tpl.num_qubits != result.num_qubits {
                continue;
            }
            let score = template_match_score(tpl, &result);
            if score > best_extra {
                best_extra = score;
            }
        }

        // Combine detection confidence with template match bonus
        let final_confidence = (result.confidence * 0.6 + best_extra * 0.4).min(1.0);

        Some((result, final_confidence))
    }

    /// Reconstruct a canonical VQE circuit from detected layers
    fn reconstruct_vqe(
        &self,
        _original: &QuantumCircuit,
        match_result: &VQEMatchResult,
    ) -> Result<QuantumCircuit> {
        let nq = match_result.num_qubits;
        let mut circuit = QuantumCircuit::new(nq, 0);

        for layer in &match_result.layers {
            match layer {
                AnsatzLayer::Rotation {
                    rotation_type,
                    params,
                } => {
                    for (qi, param) in params.iter().enumerate() {
                        if qi >= nq {
                            break;
                        }
                        let p = Parameter::Float(*param);
                        match rotation_type {
                            RotationType::RY => {
                                circuit
                                    .ry(qi, p)
                                    .map_err(|e| MyQuatError::circuit_error(e.to_string()))?;
                            }
                            RotationType::RZ => {
                                circuit
                                    .rz(qi, p)
                                    .map_err(|e| MyQuatError::circuit_error(e.to_string()))?;
                            }
                            RotationType::RX => {
                                circuit
                                    .rx(qi, p)
                                    .map_err(|e| MyQuatError::circuit_error(e.to_string()))?;
                            }
                        }
                    }
                }
                AnsatzLayer::Entanglement { pattern, gate } => {
                    let pairs = match pattern {
                        EntanglementPattern::Linear => (0..nq.saturating_sub(1))
                            .map(|i| (i, i + 1))
                            .collect::<Vec<_>>(),
                        EntanglementPattern::Circular => {
                            let mut p: Vec<(usize, usize)> =
                                (0..nq.saturating_sub(1)).map(|i| (i, i + 1)).collect();
                            if nq > 1 {
                                p.push((nq - 1, 0));
                            }
                            p
                        }
                        EntanglementPattern::Full => {
                            let mut p = Vec::new();
                            for i in 0..nq {
                                for j in (i + 1)..nq {
                                    p.push((i, j));
                                }
                            }
                            p
                        }
                        EntanglementPattern::Custom(pairs) => pairs.clone(),
                    };
                    for (ctrl, tgt) in &pairs {
                        match gate {
                            StandardGate::CX => {
                                circuit
                                    .cx(*ctrl, *tgt)
                                    .map_err(|e| MyQuatError::circuit_error(e.to_string()))?;
                            }
                            StandardGate::CZ => {
                                circuit
                                    .cz(*ctrl, *tgt)
                                    .map_err(|e| MyQuatError::circuit_error(e.to_string()))?;
                            }
                            _ => {
                                circuit
                                    .cx(*ctrl, *tgt)
                                    .map_err(|e| MyQuatError::circuit_error(e.to_string()))?;
                            }
                        }
                    }
                }
            }
        }

        Ok(circuit)
    }
}

impl Default for VqeRestorationStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl DeoptStrategy for VqeRestorationStrategy {
    fn apply(&self, circuit: &QuantumCircuit) -> Result<QuantumCircuit> {
        match self.find_best_template(circuit) {
            Some((match_result, _confidence)) => self.reconstruct_vqe(circuit, &match_result),
            None => Ok(circuit.clone()),
        }
    }

    fn name(&self) -> &str {
        "VQE Restoration"
    }

    fn confidence(&self, circuit: &QuantumCircuit) -> f64 {
        match self.find_best_template(circuit) {
            Some((_match_result, confidence)) => confidence,
            None => 0.0,
        }
    }
}

/// Score how well a detected VQE result matches a given HEA template
fn template_match_score(tpl: &HEATemplate, result: &VQEMatchResult) -> f64 {
    let mut score = 0.0;

    // Qubit count must match (already filtered)
    // Layer count similarity (40%)
    let layer_diff = (tpl.num_layers as f64 - result.num_layers as f64).abs();
    let layer_score = (1.0 - layer_diff / tpl.num_layers.max(1) as f64).max(0.0);
    score += 0.4 * layer_score;

    // Rotation type similarity (30%)
    let type_match = tpl
        .rotation_gates
        .iter()
        .filter(|rt| result.rotation_types.contains(rt))
        .count() as f64
        / tpl.rotation_gates.len().max(1) as f64;
    score += 0.3 * type_match;

    // Entanglement pattern match (30%)
    let ent_match = if tpl.entanglement == result.entanglement {
        1.0
    } else {
        0.3
    };
    score += 0.3 * ent_match;

    score
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::QuantumCircuit;
    use crate::parameter::Parameter;
    use std::f64::consts::PI;

    /// Build a simple RealAmplitudes VQE circuit: [RY] [CX] [RY] [CX] [RY]
    fn build_real_amplitudes_circuit(num_qubits: usize, num_layers: usize) -> QuantumCircuit {
        let mut c = QuantumCircuit::new(num_qubits, 0);
        for layer in 0..num_layers {
            // Rotation layer
            for q in 0..num_qubits {
                let angle = 0.1 * (layer * num_qubits + q) as f64;
                c.ry(q, Parameter::Float(angle)).unwrap();
            }
            // Entangling layer
            for q in 0..num_qubits.saturating_sub(1) {
                c.cx(q, q + 1).unwrap();
            }
        }
        // Final rotation
        for q in 0..num_qubits {
            c.ry(q, Parameter::Float(0.5)).unwrap();
        }
        c
    }

    /// Build EfficientSU2 circuit: [RY,RZ] [CX] [RY,RZ] [CX]
    fn build_efficient_su2_circuit(num_qubits: usize, num_layers: usize) -> QuantumCircuit {
        let mut c = QuantumCircuit::new(num_qubits, 0);
        for layer in 0..num_layers {
            for q in 0..num_qubits {
                c.ry(q, Parameter::Float(0.3)).unwrap();
            }
            for q in 0..num_qubits {
                c.rz(q, Parameter::Float(0.2)).unwrap();
            }
            for q in 0..num_qubits.saturating_sub(1) {
                c.cx(q, q + 1).unwrap();
            }
        }
        c
    }

    // -----------------------------------------------------------------------
    // Template data structure tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_hea_real_amplitudes() {
        let tpl = HEATemplate::real_amplitudes(4, 3);
        assert_eq!(tpl.num_qubits, 4);
        assert_eq!(tpl.num_layers, 3);
        assert_eq!(tpl.rotation_gates, vec![RotationType::RY]);
        assert_eq!(tpl.entanglement, EntanglementPattern::Linear);
        // 3 layers * (4 RY + 3 CX) = 21
        assert_eq!(tpl.expected_gate_count(), 21);
    }

    #[test]
    fn test_hea_efficient_su2() {
        let tpl = HEATemplate::efficient_su2(4, 2);
        assert_eq!(tpl.rotation_gates, vec![RotationType::RY, RotationType::RZ]);
        // 2 layers * (4 RY + 4 RZ + 3 CX) = 2 * 11 = 22
        assert_eq!(tpl.expected_gate_count(), 22);
    }

    #[test]
    fn test_uccsd_template_creation() {
        // H2 molecule: 4 qubits, 2 electrons
        let tpl = UCCSDTemplate::new(4, 2);
        assert_eq!(tpl.num_qubits, 4);
        assert_eq!(tpl.num_electrons, 2);
        // Single excitations: 2 occupied * 2 virtual = 4
        assert_eq!(tpl.single_excitations.len(), 4);
        // Double excitations: C(2,2) * C(2,2) = 1 * 1 = 1
        assert_eq!(tpl.double_excitations.len(), 1);
    }

    #[test]
    fn test_uccsd_template_larger() {
        // LiH molecule: 6 qubits, 2 electrons
        let tpl = UCCSDTemplate::new(6, 2);
        // Singles: 2 * 4 = 8
        assert_eq!(tpl.single_excitations.len(), 8);
        // Doubles: C(2,2) * C(4,2) = 1 * 6 = 6
        assert_eq!(tpl.double_excitations.len(), 6);
    }

    // -----------------------------------------------------------------------
    // Pattern detection tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_detect_real_amplitudes() {
        let circuit = build_real_amplitudes_circuit(4, 3);
        let result = detect_vqe_pattern(&circuit);
        assert!(result.is_some());

        let r = result.unwrap();
        assert_eq!(r.num_qubits, 4);
        assert!(r.confidence > 0.5);
        assert!(r.rotation_types.contains(&RotationType::RY));
        assert_eq!(r.entanglement, EntanglementPattern::Linear);
    }

    #[test]
    fn test_detect_efficient_su2() {
        let circuit = build_efficient_su2_circuit(4, 2);
        let result = detect_vqe_pattern(&circuit);
        assert!(result.is_some());

        let r = result.unwrap();
        assert!(r.confidence > 0.4);
        assert!(r.rotation_types.contains(&RotationType::RY));
        assert!(r.rotation_types.contains(&RotationType::RZ));
    }

    #[test]
    fn test_detect_empty_circuit() {
        let circuit = QuantumCircuit::new(4, 0);
        assert!(detect_vqe_pattern(&circuit).is_none());
    }

    #[test]
    fn test_detect_non_vqe_circuit() {
        // A circuit with only H and CX (no rotation parameters) should score low
        let mut circuit = QuantumCircuit::new(4, 0);
        for i in 0..4 {
            circuit.h(i).unwrap();
        }
        circuit.cx(0, 1).unwrap();
        circuit.cx(2, 3).unwrap();
        let result = detect_vqe_pattern(&circuit);
        // Should either be None or have very low confidence
        if let Some(r) = result {
            assert!(r.confidence < 0.3);
        }
    }

    // -----------------------------------------------------------------------
    // Entanglement pattern detection tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_infer_linear_pattern() {
        let pairs = vec![(0, 1), (1, 2), (2, 3)];
        assert_eq!(
            infer_entanglement_pattern(&pairs, 4),
            EntanglementPattern::Linear
        );
    }

    #[test]
    fn test_infer_circular_pattern() {
        let pairs = vec![(0, 1), (1, 2), (2, 3), (3, 0)];
        assert_eq!(
            infer_entanglement_pattern(&pairs, 4),
            EntanglementPattern::Circular
        );
    }

    // -----------------------------------------------------------------------
    // Strategy tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_vqe_strategy_confidence() {
        let strategy = VqeRestorationStrategy::new();
        let circuit = build_real_amplitudes_circuit(4, 3);
        let conf = strategy.confidence(&circuit);
        assert!(conf > 0.3, "VQE confidence should be > 0.3, got {}", conf);
    }

    #[test]
    fn test_vqe_strategy_apply() {
        let strategy = VqeRestorationStrategy::new();
        let circuit = build_real_amplitudes_circuit(4, 2);
        let restored = strategy.apply(&circuit).unwrap();
        assert_eq!(restored.num_qubits(), 4);
        assert!(restored.size() > 0);
    }

    #[test]
    fn test_vqe_strategy_name() {
        let strategy = VqeRestorationStrategy::new();
        assert_eq!(strategy.name(), "VQE Restoration");
    }

    #[test]
    fn test_vqe_strategy_default() {
        let strategy = VqeRestorationStrategy::default();
        assert_eq!(strategy.min_confidence, 0.3);
        assert!(!strategy.hea_templates.is_empty());
    }

    #[test]
    fn test_vqe_strategy_non_vqe_circuit() {
        let strategy = VqeRestorationStrategy::new();
        let circuit = QuantumCircuit::new(4, 0);
        let conf = strategy.confidence(&circuit);
        assert_eq!(conf, 0.0);
    }

    // -----------------------------------------------------------------------
    // Match score tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_template_match_score_perfect() {
        let tpl = HEATemplate::real_amplitudes(4, 3);
        let result = VQEMatchResult {
            vqe_type: VQEType::RealAmplitudes,
            num_qubits: 4,
            num_layers: 3,
            rotation_types: vec![RotationType::RY],
            entanglement: EntanglementPattern::Linear,
            layers: vec![],
            confidence: 0.9,
        };
        let score = template_match_score(&tpl, &result);
        assert!(
            score > 0.9,
            "Perfect match should score > 0.9, got {}",
            score
        );
    }

    #[test]
    fn test_template_match_score_partial() {
        let tpl = HEATemplate::efficient_su2(4, 3);
        let result = VQEMatchResult {
            vqe_type: VQEType::RealAmplitudes,
            num_qubits: 4,
            num_layers: 2,
            rotation_types: vec![RotationType::RY], // missing RZ
            entanglement: EntanglementPattern::Linear,
            layers: vec![],
            confidence: 0.7,
        };
        let score = template_match_score(&tpl, &result);
        assert!(score > 0.3 && score < 0.9, "Partial match score: {}", score);
    }

    // -----------------------------------------------------------------------
    // Benchmark-style VQE circuit test
    // -----------------------------------------------------------------------

    #[test]
    fn test_benchmark_vqe_circuit_detected() {
        // This is the same circuit generated by BenchmarkSuite::generate_vqe_ansatz
        let circuit = BenchmarkSuite_generate_vqe(4, 3);
        let result = detect_vqe_pattern(&circuit);
        assert!(result.is_some(), "Benchmark VQE circuit should be detected");
        let r = result.unwrap();
        assert!(
            r.confidence > 0.3,
            "Benchmark VQE confidence: {}",
            r.confidence
        );
    }

    #[test]
    fn test_benchmark_vqe_strategy_confidence() {
        let strategy = VqeRestorationStrategy::new();
        let circuit = BenchmarkSuite_generate_vqe(4, 3);
        let conf = strategy.confidence(&circuit);
        assert!(conf > 0.3, "Strategy confidence on benchmark VQE: {}", conf);
    }

    /// Reproduce the benchmark VQE circuit generator
    fn BenchmarkSuite_generate_vqe(num_qubits: usize, depth: usize) -> QuantumCircuit {
        let mut circuit = QuantumCircuit::new(num_qubits, 0);
        for _ in 0..depth {
            for i in 0..num_qubits {
                circuit.ry(i, Parameter::Float(0.3)).unwrap();
                circuit.rz(i, Parameter::Float(0.2)).unwrap();
            }
            for i in 0..num_qubits.saturating_sub(1) {
                circuit.cx(i, i + 1).unwrap();
            }
        }
        circuit
    }
}
