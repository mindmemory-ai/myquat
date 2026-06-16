// deoptimization/pauli_basis.rs - Pauli basis transformations
// Author: gA4ss
//
// Tools for transforming between computational basis and Pauli basis,
// and for identifying Pauli rotations from gate sequences.
//
// NOTE: Full pattern matching implementation will be completed in Week 3 (Task 2.1)
// This module currently provides the基础 data structures and简化 API.

/// Identified Pauli rotation from gate sequence
#[derive(Debug, Clone, PartialEq)]
pub struct PauliRotation {
    /// Pauli string (e.g., "XX", "ZZ", "XY")
    pub pauli_string: String,
    /// Rotation angle in radians
    pub angle: f64,
    /// Qubits involved
    pub qubits: Vec<usize>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
}

impl PauliRotation {
    /// Create new Pauli rotation
    pub fn new(pauli_string: &str, angle: f64) -> Self {
        let num_qubits = pauli_string.len();
        Self {
            pauli_string: pauli_string.to_string(),
            angle,
            qubits: (0..num_qubits).collect(),
            confidence: 1.0,
        }
    }

    /// Create with specific qubits
    pub fn with_qubits(pauli_string: &str, angle: f64, qubits: Vec<usize>) -> Self {
        Self {
            pauli_string: pauli_string.to_string(),
            angle,
            qubits,
            confidence: 1.0,
        }
    }

    /// Set confidence score
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }
}

/// Pattern matcher for common 2-qubit Pauli rotation decompositions
///
/// This will be fully implemented in Week 3 (Task 2.1: KAK Restoration Algorithm)
pub struct PauliPatternMatcher {
    tolerance: f64,
}

impl PauliPatternMatcher {
    /// Create new pattern matcher
    pub fn new() -> Self {
        Self { tolerance: 1e-10 }
    }

    /// Set matching tolerance
    pub fn with_tolerance(mut self, tol: f64) -> Self {
        self.tolerance = tol;
        self
    }
}

impl Default for PauliPatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_pauli_rotation_creation() {
        let rot = PauliRotation::new("ZZ", PI / 4.0);
        assert_eq!(rot.pauli_string, "ZZ");
        assert_eq!(rot.angle, PI / 4.0);
        assert_eq!(rot.qubits, vec![0, 1]);
        assert_eq!(rot.confidence, 1.0);
    }

    #[test]
    fn test_pauli_rotation_with_qubits() {
        let rot = PauliRotation::with_qubits("XY", PI / 2.0, vec![2, 3]);
        assert_eq!(rot.qubits, vec![2, 3]);
    }

    #[test]
    fn test_pauli_rotation_with_confidence() {
        let rot = PauliRotation::new("ZZ", PI / 4.0).with_confidence(0.85);
        assert_eq!(rot.confidence, 0.85);
    }

    #[test]
    fn test_pattern_matcher_creation() {
        let matcher = PauliPatternMatcher::new();
        assert_eq!(matcher.tolerance, 1e-10);
    }

    #[test]
    fn test_pattern_matcher_with_tolerance() {
        let matcher = PauliPatternMatcher::new().with_tolerance(1e-8);
        assert_eq!(matcher.tolerance, 1e-8);
    }
}
