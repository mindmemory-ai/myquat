// deoptimization/kak.rs - KAK decomposition restoration
// Author: gA4ss
//
// Restores Pauli rotations from decomposed gate sequences using
// KAK (Kraus-Cirac) decomposition analysis.

use super::pauli_basis::PauliRotation;
use super::DeoptStrategy;
use crate::circuit::QuantumCircuit;
use crate::error::Result;
use crate::gates::{Gate, StandardGate};
use crate::parameter::Parameter;

/// Strategy for restoring Pauli rotations from KAK-decomposed sequences
///
/// This strategy identifies gate sequences that result from decomposing
/// 2-qubit Pauli rotations (e.g., exp(-iθ ZZ)) and restores them to
/// their original form.
///
/// Common patterns recognized:
/// - ZZ: H(q0) CX(q0,q1) Rz(θ,q1) CX(q0,q1) H(q0)
/// - XX: Ry(-π/2,q0) Ry(-π/2,q1) CX(q0,q1) Rz(θ,q1) CX(q0,q1) Ry(π/2,q0) Ry(π/2,q1)
/// - YY: Rx(π/2,q0) Rx(π/2,q1) CX(q0,q1) Rz(θ,q1) CX(q0,q1) Rx(-π/2,q0) Rx(-π/2,q1)
#[derive(Debug, Clone)]
pub struct KakRestorationStrategy {
    /// Minimum confidence to attempt restoration
    min_confidence: f64,
    /// Maximum window size for pattern matching
    window_size: usize,
}

impl KakRestorationStrategy {
    /// Create new strategy with default settings
    pub fn new() -> Self {
        Self {
            min_confidence: 0.5,
            window_size: 10,
        }
    }

    /// Set minimum confidence threshold
    pub fn with_min_confidence(mut self, confidence: f64) -> Self {
        self.min_confidence = confidence;
        self
    }

    /// Set maximum window size for pattern matching
    pub fn with_window_size(mut self, size: usize) -> Self {
        self.window_size = size;
        self
    }

    /// Try to match ZZ rotation pattern
    ///
    /// Pattern: H(q0) CX(q0,q1) Rz(θ,q1) CX(q0,q1) H(q0)
    fn match_zz_pattern(&self, gates: &[Gate], qubits: &[usize]) -> Option<PauliRotation> {
        if gates.len() != 5 {
            return None;
        }

        // Expected qubits: [q0, q0, q1, q1, q0, q1, q0]
        // H(q0), CX(q0,q1), Rz(q1), CX(q0,q1), H(q0)
        if qubits.len() != 7 {
            return None;
        }

        // Check pattern: H - CX - Rz - CX - H
        if gates[0].gate_type != StandardGate::H {
            return None;
        }
        if gates[1].gate_type != StandardGate::CX {
            return None;
        }
        if gates[2].gate_type != StandardGate::Rz {
            return None;
        }
        if gates[3].gate_type != StandardGate::CX {
            return None;
        }
        if gates[4].gate_type != StandardGate::H {
            return None;
        }

        // Check qubit consistency
        // First H on q0
        let q0 = qubits[0];
        // First CX: q0, q1
        let cx_ctrl1 = qubits[1];
        let cx_tgt1 = qubits[2];
        // Rz on q1
        let rz_qubit = qubits[3];
        // Second CX: q0, q1
        let cx_ctrl2 = qubits[4];
        let cx_tgt2 = qubits[5];
        // Last H on q0
        let h_qubit = qubits[6];

        // Verify pattern consistency
        if q0 != cx_ctrl1 || q0 != cx_ctrl2 || q0 != h_qubit {
            return None;
        }

        if cx_tgt1 != cx_tgt2 || cx_tgt1 != rz_qubit {
            return None;
        }

        let q1 = cx_tgt1;

        // Extract rotation angle from Rz gate
        if let Some(angle) = self.extract_angle(&gates[2]) {
            return Some(
                PauliRotation::with_qubits("ZZ", angle, vec![q0, q1]).with_confidence(0.95),
            );
        }

        None
    }

    /// Try to match XX rotation pattern
    fn match_xx_pattern(&self, gates: &[Gate], qubits: &[usize]) -> Option<PauliRotation> {
        if gates.len() != 7 || qubits.len() != 7 {
            return None;
        }

        // Pattern: Ry(-π/2) Ry(-π/2) CX Rz(θ) CX Ry(π/2) Ry(π/2)
        // Simplified check - full implementation would verify all angles
        if gates[0].gate_type == StandardGate::Ry
            && gates[1].gate_type == StandardGate::Ry
            && gates[2].gate_type == StandardGate::CX
            && gates[3].gate_type == StandardGate::Rz
            && gates[4].gate_type == StandardGate::CX
            && gates[5].gate_type == StandardGate::Ry
            && gates[6].gate_type == StandardGate::Ry
        {
            if let Some(angle) = self.extract_angle(&gates[3]) {
                let q0 = qubits[0];
                let q1 = qubits[1];
                return Some(
                    PauliRotation::with_qubits("XX", angle, vec![q0, q1]).with_confidence(0.90),
                );
            }
        }

        None
    }

    /// Try to match YY rotation pattern
    fn match_yy_pattern(&self, gates: &[Gate], qubits: &[usize]) -> Option<PauliRotation> {
        if gates.len() != 7 || qubits.len() != 7 {
            return None;
        }

        // Pattern: Rx(π/2) Rx(π/2) CX Rz(θ) CX Rx(-π/2) Rx(-π/2)
        if gates[0].gate_type == StandardGate::Rx
            && gates[1].gate_type == StandardGate::Rx
            && gates[2].gate_type == StandardGate::CX
            && gates[3].gate_type == StandardGate::Rz
            && gates[4].gate_type == StandardGate::CX
            && gates[5].gate_type == StandardGate::Rx
            && gates[6].gate_type == StandardGate::Rx
        {
            if let Some(angle) = self.extract_angle(&gates[3]) {
                let q0 = qubits[0];
                let q1 = qubits[1];
                return Some(
                    PauliRotation::with_qubits("YY", angle, vec![q0, q1]).with_confidence(0.90),
                );
            }
        }

        None
    }

    /// Extract rotation angle from a parametric gate
    fn extract_angle(&self, gate: &Gate) -> Option<f64> {
        if gate.parameters.is_empty() {
            return None;
        }

        match &gate.parameters[0] {
            Parameter::Float(angle) => Some(*angle),
            _ => None, // Symbolic parameters not supported yet
        }
    }

    /// Scan circuit for KAK patterns and identify them
    fn identify_patterns(&self, circuit: &QuantumCircuit) -> Vec<(usize, PauliRotation)> {
        let mut identified = Vec::new();

        // Get circuit data
        let data = circuit.data();
        let instructions = data.instructions();
        if instructions.is_empty() {
            return identified;
        }

        // Sliding window search
        for i in 0..instructions.len() {
            // Try different window sizes
            for window in 5..=self.window_size.min(instructions.len() - i) {
                let gates: Vec<Gate> = instructions[i..i + window]
                    .iter()
                    .map(|inst| inst.gate.clone())
                    .collect();

                let qubits: Vec<usize> = instructions[i..i + window]
                    .iter()
                    .flat_map(|inst| inst.qubits.iter().map(|q| q.index()))
                    .collect();

                // Try ZZ pattern (5 gates)
                if window == 5 {
                    if let Some(rot) = self.match_zz_pattern(&gates, &qubits) {
                        if rot.confidence >= self.min_confidence {
                            identified.push((i, rot));
                        }
                    }
                }

                // Try XX/YY patterns (7 gates)
                if window == 7 {
                    if let Some(rot) = self.match_xx_pattern(&gates, &qubits) {
                        if rot.confidence >= self.min_confidence {
                            identified.push((i, rot));
                        }
                    }

                    if let Some(rot) = self.match_yy_pattern(&gates, &qubits) {
                        if rot.confidence >= self.min_confidence {
                            identified.push((i, rot));
                        }
                    }
                }
            }
        }

        identified
    }
}

impl Default for KakRestorationStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl DeoptStrategy for KakRestorationStrategy {
    fn apply(&self, circuit: &QuantumCircuit) -> Result<QuantumCircuit> {
        // Identify all KAK patterns in the circuit
        let patterns = self.identify_patterns(circuit);

        if patterns.is_empty() {
            // No patterns found, return original circuit
            return Ok(circuit.clone());
        }

        // For now, return the original circuit
        // Full restoration (replacing patterns with single Pauli rotations)
        // will be implemented when we add custom gate support
        Ok(circuit.clone())
    }

    fn name(&self) -> &str {
        "KAK Restoration"
    }

    fn confidence(&self, circuit: &QuantumCircuit) -> f64 {
        let patterns = self.identify_patterns(circuit);

        if patterns.is_empty() {
            return 0.0;
        }

        // Calculate average confidence across all identified patterns
        let total_confidence: f64 = patterns.iter().map(|(_, rot)| rot.confidence).sum();

        total_confidence / patterns.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_strategy_creation() {
        let strategy = KakRestorationStrategy::new();
        assert_eq!(strategy.name(), "KAK Restoration");
        assert_eq!(strategy.min_confidence, 0.5);
        assert_eq!(strategy.window_size, 10);
    }

    #[test]
    fn test_with_confidence() {
        let strategy = KakRestorationStrategy::new().with_min_confidence(0.7);
        assert_eq!(strategy.min_confidence, 0.7);
    }

    #[test]
    fn test_with_window_size() {
        let strategy = KakRestorationStrategy::new().with_window_size(15);
        assert_eq!(strategy.window_size, 15);
    }

    #[test]
    fn test_extract_angle() {
        let strategy = KakRestorationStrategy::new();

        // Create Rz gate with float parameter
        let gate = Gate::new(StandardGate::Rz, vec![Parameter::Float(PI / 4.0)]).unwrap();

        let angle = strategy.extract_angle(&gate);
        assert!(angle.is_some());
        assert!((angle.unwrap() - PI / 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_extract_angle_no_params() {
        let strategy = KakRestorationStrategy::new();

        // Create H gate (no parameters)
        let gate = Gate::new(StandardGate::H, vec![]).unwrap();

        let angle = strategy.extract_angle(&gate);
        assert!(angle.is_none());
    }

    #[test]
    fn test_match_zz_pattern_structure() {
        let strategy = KakRestorationStrategy::new();

        // Create ZZ pattern gates
        let gates = vec![
            Gate::new(StandardGate::H, vec![]).unwrap(),
            Gate::new(StandardGate::CX, vec![]).unwrap(),
            Gate::new(StandardGate::Rz, vec![Parameter::Float(PI / 4.0)]).unwrap(),
            Gate::new(StandardGate::CX, vec![]).unwrap(),
            Gate::new(StandardGate::H, vec![]).unwrap(),
        ];

        let qubits = vec![0, 0, 1, 1, 0, 1, 0];

        let result = strategy.match_zz_pattern(&gates, &qubits);
        assert!(result.is_some());

        let rot = result.unwrap();
        assert_eq!(rot.pauli_string, "ZZ");
        assert!((rot.angle - PI / 4.0).abs() < 1e-10);
        assert!(rot.confidence >= 0.9);
    }

    #[test]
    fn test_match_zz_pattern_wrong_length() {
        let strategy = KakRestorationStrategy::new();

        // Wrong number of gates
        let gates = vec![
            Gate::new(StandardGate::H, vec![]).unwrap(),
            Gate::new(StandardGate::CX, vec![]).unwrap(),
        ];

        let qubits = vec![0, 0, 1];

        let result = strategy.match_zz_pattern(&gates, &qubits);
        assert!(result.is_none());
    }

    #[test]
    fn test_match_zz_pattern_wrong_gates() {
        let strategy = KakRestorationStrategy::new();

        // Wrong gate types
        let gates = vec![
            Gate::new(StandardGate::X, vec![]).unwrap(), // Should be H
            Gate::new(StandardGate::CX, vec![]).unwrap(),
            Gate::new(StandardGate::Rz, vec![Parameter::Float(PI / 4.0)]).unwrap(),
            Gate::new(StandardGate::CX, vec![]).unwrap(),
            Gate::new(StandardGate::H, vec![]).unwrap(),
        ];

        let qubits = vec![0, 0, 1, 1, 0, 1, 0];

        let result = strategy.match_zz_pattern(&gates, &qubits);
        assert!(result.is_none());
    }

    #[test]
    fn test_identify_patterns_empty_circuit() {
        let strategy = KakRestorationStrategy::new();
        let circuit = QuantumCircuit::new(2, 0);

        let patterns = strategy.identify_patterns(&circuit);
        assert_eq!(patterns.len(), 0);
    }

    #[test]
    fn test_confidence_empty_circuit() {
        let strategy = KakRestorationStrategy::new();
        let circuit = QuantumCircuit::new(2, 0);

        let conf = strategy.confidence(&circuit);
        assert_eq!(conf, 0.0);
    }

    #[test]
    fn test_apply_empty_circuit() {
        let strategy = KakRestorationStrategy::new();
        let circuit = QuantumCircuit::new(2, 0);

        let result = strategy.apply(&circuit);
        assert!(result.is_ok());

        let restored = result.unwrap();
        assert_eq!(restored.num_qubits(), 2);
    }

    #[test]
    fn test_identify_patterns_with_zz() {
        let strategy = KakRestorationStrategy::new();
        let mut circuit = QuantumCircuit::new(2, 0);

        // Add ZZ pattern: H(0) CX(0,1) Rz(θ,1) CX(0,1) H(0)
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.rz(1, Parameter::Float(PI / 4.0)).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.h(0).unwrap();

        let patterns = strategy.identify_patterns(&circuit);
        // Should identify at least one pattern
        // (exact count depends on sliding window behavior)
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_confidence_with_zz() {
        let strategy = KakRestorationStrategy::new();
        let mut circuit = QuantumCircuit::new(2, 0);

        // Add ZZ pattern
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.rz(1, Parameter::Float(PI / 4.0)).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.h(0).unwrap();

        let conf = strategy.confidence(&circuit);
        assert!(conf > 0.0);
    }
}
