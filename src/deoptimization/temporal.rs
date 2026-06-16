// deoptimization/temporal.rs - Temporal angle analysis
// Author: gA4ss
//
// Analyzes rotation angles to infer evolution time using Suzuki
// coefficient fingerprints.

use super::DeoptStrategy;
use crate::circuit::QuantumCircuit;
use crate::error::Result;
use crate::gates::StandardGate;
use crate::parameter::Parameter;

/// Suzuki coefficients for different orders
const SUZUKI_COEFFICIENTS: &[(usize, f64)] = &[
    (2, 0.0), // Not used for order 2
    (4, 0.414_490_771_794_375_7),
    (6, 0.373_072_169_334_813_1),
    (8, 0.35959009351690036),
    (10, 0.352_924_269_905_115_6),
];

/// Inferred evolution parameters
#[derive(Debug, Clone, PartialEq)]
pub struct EvolutionParams {
    /// Trotter order
    pub order: usize,
    /// Evolution time step
    pub dt: f64,
    /// Number of Trotter steps
    pub num_steps: usize,
    /// Confidence score
    pub confidence: f64,
}

/// Strategy for inferring evolution parameters from rotation angles
///
/// Uses the special numerical values of Suzuki coefficients as
/// fingerprints to identify Trotter decomposition parameters.
#[derive(Debug, Clone)]
pub struct TemporalAnalysisStrategy {
    /// Tolerance for floating point comparison
    tolerance: f64,
    /// Maximum number of candidates to return
    max_candidates: usize,
}

impl TemporalAnalysisStrategy {
    /// Create new strategy with default settings
    pub fn new() -> Self {
        Self {
            tolerance: 1e-10,
            max_candidates: 5,
        }
    }

    /// Set tolerance for angle matching
    pub fn with_tolerance(mut self, tol: f64) -> Self {
        self.tolerance = tol;
        self
    }

    /// Set maximum number of candidates
    pub fn with_max_candidates(mut self, max: usize) -> Self {
        self.max_candidates = max;
        self
    }

    /// Get Suzuki coefficient for given order
    pub fn get_suzuki_coeff(order: usize) -> Option<f64> {
        SUZUKI_COEFFICIENTS
            .iter()
            .find(|(o, _)| *o == order)
            .map(|(_, c)| *c)
    }

    /// Extract rotation angles from circuit
    fn extract_angles(&self, circuit: &QuantumCircuit) -> Vec<f64> {
        let mut angles = Vec::new();
        let instructions = circuit.data().instructions();

        for inst in instructions {
            // Check for parametric rotation gates
            let is_rotation = matches!(
                inst.gate.gate_type,
                StandardGate::Rx
                    | StandardGate::Ry
                    | StandardGate::Rz
                    | StandardGate::P
                    | StandardGate::CRx
                    | StandardGate::CRy
                    | StandardGate::CRz
                    | StandardGate::CP
            );

            if is_rotation && !inst.gate.parameters.is_empty() {
                if let Parameter::Float(angle) = inst.gate.parameters[0] {
                    angles.push(angle.abs());
                }
            }
        }

        angles
    }

    /// Find greatest common divisor of angles (up to tolerance)
    fn find_gcd_angles(&self, angles: &[f64]) -> Option<f64> {
        if angles.is_empty() {
            return None;
        }

        if angles.len() == 1 {
            return Some(angles[0]);
        }

        // Start with smallest non-zero angle
        let mut gcd = *angles
            .iter()
            .filter(|&&a| a > self.tolerance)
            .min_by(|a, b| a.partial_cmp(b).unwrap())?;

        // Try to find a common divisor
        for _ in 0..100 {
            // Iteration limit
            let mut all_divisible = true;

            for &angle in angles {
                if angle < self.tolerance {
                    continue;
                }

                // Check if angle is a multiple of gcd
                let ratio = angle / gcd;
                let rounded = ratio.round();

                if (ratio - rounded).abs() > self.tolerance {
                    all_divisible = false;
                    // Reduce gcd
                    gcd = angle % gcd;
                    if gcd < self.tolerance {
                        return None;
                    }
                    break;
                }
            }

            if all_divisible {
                return Some(gcd);
            }
        }

        None
    }

    /// Check if an angle matches a Suzuki coefficient pattern
    fn match_suzuki_pattern(&self, angle: f64, dt: f64) -> Option<usize> {
        for &(order, coeff) in SUZUKI_COEFFICIENTS {
            if order == 2 {
                continue; // Skip order 2
            }

            // Check if angle ≈ coeff * dt
            let expected = coeff * dt;
            if (angle - expected).abs() < self.tolerance {
                return Some(order);
            }
        }
        None
    }

    /// Infer evolution parameters from circuit
    pub fn infer_parameters(&self, circuit: &QuantumCircuit) -> Vec<EvolutionParams> {
        let angles = self.extract_angles(circuit);

        if angles.is_empty() {
            return Vec::new();
        }

        let mut candidates = Vec::new();

        // Try to find GCD (potential dt)
        if let Some(gcd) = self.find_gcd_angles(&angles) {
            // Check for Suzuki coefficient patterns
            for &angle in &angles {
                if let Some(order) = self.match_suzuki_pattern(angle, gcd) {
                    // Calculate confidence based on how many angles match
                    let mut matches = 0;
                    for &a in &angles {
                        if let Some(o) = self.match_suzuki_pattern(a, gcd) {
                            if o == order {
                                matches += 1;
                            }
                        }
                    }

                    let confidence = matches as f64 / angles.len() as f64;

                    candidates.push(EvolutionParams {
                        order,
                        dt: gcd,
                        num_steps: 1, // Simplified: assume 1 step
                        confidence,
                    });
                }
            }
        }

        // Also try dt as multiples/divisors of angles
        let unique_angles: Vec<f64> = {
            let mut sorted = angles.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            sorted.dedup_by(|a, b| (*a - *b).abs() < self.tolerance);
            sorted
        };

        for &base_angle in &unique_angles {
            for &(order, coeff) in SUZUKI_COEFFICIENTS {
                if order == 2 {
                    continue;
                }

                // Try dt = angle / coeff
                let potential_dt = base_angle / coeff;

                if potential_dt < self.tolerance {
                    continue;
                }

                // Check how many angles match this hypothesis
                let mut matches = 0;
                for &angle in &angles {
                    // Check if angle ≈ coeff * dt for any Suzuki coefficient
                    for &(o, c) in SUZUKI_COEFFICIENTS {
                        if o == 2 {
                            continue;
                        }
                        let expected = c * potential_dt;
                        if (angle - expected).abs() < self.tolerance * 10.0 {
                            matches += 1;
                            break;
                        }
                    }
                }

                if matches > 0 {
                    let confidence = matches as f64 / angles.len() as f64;

                    candidates.push(EvolutionParams {
                        order,
                        dt: potential_dt,
                        num_steps: 1,
                        confidence,
                    });
                }
            }
        }

        // Sort by confidence (descending)
        candidates.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        // Remove duplicates and limit results
        candidates.dedup_by(|a, b| (a.order == b.order) && ((a.dt - b.dt).abs() < self.tolerance));

        candidates.truncate(self.max_candidates);

        candidates
    }
}

impl Default for TemporalAnalysisStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl DeoptStrategy for TemporalAnalysisStrategy {
    fn apply(&self, circuit: &QuantumCircuit) -> Result<QuantumCircuit> {
        // Analyze circuit to infer parameters
        let _params = self.infer_parameters(circuit);

        // For now, return original circuit
        // Full implementation would reconstruct Hamiltonian with inferred dt
        Ok(circuit.clone())
    }

    fn name(&self) -> &str {
        "Temporal Analysis"
    }

    fn confidence(&self, circuit: &QuantumCircuit) -> f64 {
        let params = self.infer_parameters(circuit);

        if params.is_empty() {
            return 0.0;
        }

        // Return highest confidence
        params[0].confidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_strategy_creation() {
        let strategy = TemporalAnalysisStrategy::new();
        assert_eq!(strategy.name(), "Temporal Analysis");
        assert_eq!(strategy.tolerance, 1e-10);
        assert_eq!(strategy.max_candidates, 5);
    }

    #[test]
    fn test_suzuki_coefficients() {
        assert_eq!(
            TemporalAnalysisStrategy::get_suzuki_coeff(4).unwrap(),
            0.41449077179437573
        );
        assert_eq!(
            TemporalAnalysisStrategy::get_suzuki_coeff(6).unwrap(),
            0.37307216933481307
        );
        assert!(TemporalAnalysisStrategy::get_suzuki_coeff(12).is_none());
    }

    #[test]
    fn test_with_tolerance() {
        let strategy = TemporalAnalysisStrategy::new().with_tolerance(1e-8);
        assert_eq!(strategy.tolerance, 1e-8);
    }

    #[test]
    fn test_with_max_candidates() {
        let strategy = TemporalAnalysisStrategy::new().with_max_candidates(10);
        assert_eq!(strategy.max_candidates, 10);
    }

    #[test]
    fn test_extract_angles_empty() {
        let strategy = TemporalAnalysisStrategy::new();
        let circuit = QuantumCircuit::new(2, 0);

        let angles = strategy.extract_angles(&circuit);
        assert_eq!(angles.len(), 0);
    }

    #[test]
    fn test_extract_angles_simple() {
        let strategy = TemporalAnalysisStrategy::new();
        let mut circuit = QuantumCircuit::new(2, 0);

        circuit.rz(0, Parameter::Float(PI / 4.0)).unwrap();
        circuit.rx(1, Parameter::Float(PI / 2.0)).unwrap();

        let angles = strategy.extract_angles(&circuit);
        assert_eq!(angles.len(), 2);
        assert!((angles[0] - PI / 4.0).abs() < 1e-10);
        assert!((angles[1] - PI / 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_extract_angles_non_rotation() {
        let strategy = TemporalAnalysisStrategy::new();
        let mut circuit = QuantumCircuit::new(2, 0);

        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.rz(0, Parameter::Float(PI / 4.0)).unwrap();

        let angles = strategy.extract_angles(&circuit);
        assert_eq!(angles.len(), 1); // Only Rz gate
    }

    #[test]
    fn test_find_gcd_angles_single() {
        let strategy = TemporalAnalysisStrategy::new();
        let angles = vec![0.5];

        let gcd = strategy.find_gcd_angles(&angles);
        assert!(gcd.is_some());
        assert!((gcd.unwrap() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_find_gcd_angles_multiple() {
        let strategy = TemporalAnalysisStrategy::new();
        let angles = vec![0.3, 0.6, 0.9];

        let gcd = strategy.find_gcd_angles(&angles);
        assert!(gcd.is_some());
        // GCD should be close to 0.3
        let gcd_val = gcd.unwrap();
        assert!(gcd_val > 0.0);
        assert!(gcd_val <= 0.3 + 1e-9);
    }

    #[test]
    fn test_find_gcd_angles_empty() {
        let strategy = TemporalAnalysisStrategy::new();
        let angles: Vec<f64> = vec![];

        let gcd = strategy.find_gcd_angles(&angles);
        assert!(gcd.is_none());
    }

    #[test]
    fn test_match_suzuki_pattern() {
        let strategy = TemporalAnalysisStrategy::new();

        // Test 4th order Suzuki coefficient
        let dt = 1.0;
        let angle = 0.41449077179437573; // 4th order coefficient

        let order = strategy.match_suzuki_pattern(angle, dt);
        assert_eq!(order, Some(4));
    }

    #[test]
    fn test_match_suzuki_pattern_no_match() {
        let strategy = TemporalAnalysisStrategy::new();

        let dt = 1.0;
        let angle = 0.999; // Doesn't match any coefficient

        let order = strategy.match_suzuki_pattern(angle, dt);
        assert!(order.is_none());
    }

    #[test]
    fn test_infer_parameters_empty() {
        let strategy = TemporalAnalysisStrategy::new();
        let circuit = QuantumCircuit::new(2, 0);

        let params = strategy.infer_parameters(&circuit);
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_infer_parameters_suzuki_pattern() {
        let strategy = TemporalAnalysisStrategy::new().with_tolerance(1e-8);
        let mut circuit = QuantumCircuit::new(2, 0);

        // Add gates with 4th order Suzuki coefficient pattern
        let dt = 0.1;
        let coeff = 0.41449077179437573;
        circuit.rz(0, Parameter::Float(coeff * dt)).unwrap();
        circuit.rz(1, Parameter::Float(coeff * dt)).unwrap();

        let params = strategy.infer_parameters(&circuit);
        // Should find at least one candidate
        assert!(!params.is_empty());

        if !params.is_empty() {
            assert_eq!(params[0].order, 4);
            assert!((params[0].dt - dt).abs() < 0.01); // Tolerance for inference
        }
    }

    #[test]
    fn test_confidence_empty_circuit() {
        let strategy = TemporalAnalysisStrategy::new();
        let circuit = QuantumCircuit::new(2, 0);

        let conf = strategy.confidence(&circuit);
        assert_eq!(conf, 0.0);
    }

    #[test]
    fn test_confidence_with_angles() {
        let strategy = TemporalAnalysisStrategy::new().with_tolerance(1e-8);
        let mut circuit = QuantumCircuit::new(2, 0);

        // Add some rotation gates
        circuit.rz(0, Parameter::Float(0.1)).unwrap();
        circuit.rx(1, Parameter::Float(0.2)).unwrap();

        let conf = strategy.confidence(&circuit);
        // Confidence should be >= 0.0
        assert!(conf >= 0.0);
    }

    #[test]
    fn test_apply_empty_circuit() {
        let strategy = TemporalAnalysisStrategy::new();
        let circuit = QuantumCircuit::new(2, 0);

        let result = strategy.apply(&circuit);
        assert!(result.is_ok());
    }

    #[test]
    fn test_evolution_params_equality() {
        let params1 = EvolutionParams {
            order: 4,
            dt: 0.1,
            num_steps: 1,
            confidence: 0.95,
        };

        let params2 = EvolutionParams {
            order: 4,
            dt: 0.1,
            num_steps: 1,
            confidence: 0.95,
        };

        assert_eq!(params1, params2);
    }
}
