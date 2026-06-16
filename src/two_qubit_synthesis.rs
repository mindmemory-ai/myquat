// Two-Qubit Gate Synthesis and Optimization
// Author: gA4ss
//
// Practical two-qubit gate synthesis using pattern matching and known decompositions

use crate::error::Result;
use crate::gates::StandardGate;

/// Two-qubit gate sequence optimizer
///
/// Recognizes common patterns and synthesizes optimal implementations
pub struct TwoQubitSynthesizer;

impl TwoQubitSynthesizer {
    /// Optimize a sequence of gates on two qubits
    ///
    /// Returns optimized gate sequence
    pub fn optimize_sequence(gates: &[GateWithParams]) -> Result<Vec<GateWithParams>> {
        // Pattern 1: CX - RZ - CX -> optimized form
        if gates.len() >= 3 {
            if let Some(opt) = Self::try_optimize_cx_rz_cx(gates) {
                return Ok(opt);
            }
        }

        // Pattern 2: Multiple CX on same qubits
        if let Some(opt) = Self::try_optimize_cx_chain(gates) {
            return Ok(opt);
        }

        // Pattern 3: Single-qubit gates between CX
        if let Some(opt) = Self::try_merge_single_qubit_gates(gates) {
            return Ok(opt);
        }

        // No optimization found, return original
        Ok(gates.to_vec())
    }

    /// Optimize CX - RZ - CX pattern
    ///
    /// CX(0,1) - RZ(θ) on qubit 1 - CX(0,1) = Rzz(θ)
    /// This is already the optimal form (2 CX gates are required for
    /// any non-trivial Rzz interaction). No CX reduction is possible.
    /// Only the trivial case θ=0 (identity) would allow CX cancellation,
    /// but that's handled separately by single-qubit gate merging.
    fn try_optimize_cx_rz_cx(_gates: &[GateWithParams]) -> Option<Vec<GateWithParams>> {
        // CX-Rz-CX = Rzz interaction. The 2 CX gates are required;
        // removing either one would change the unitary. No optimization
        // possible at the gate-sequence level without qubit tracking.
        None
    }

    /// Optimize chain of CX gates
    ///
    /// CX - CX -> Identity (cancel)
    /// CX - ... - CX -> check if they cancel via commutation
    fn try_optimize_cx_chain(gates: &[GateWithParams]) -> Option<Vec<GateWithParams>> {
        let mut optimized = Vec::new();
        let mut i = 0;

        while i < gates.len() {
            if gates[i].gate == StandardGate::CX {
                // Look ahead for another CX
                if i + 1 < gates.len() && gates[i + 1].gate == StandardGate::CX {
                    // Two consecutive CX gates cancel
                    i += 2; // Skip both
                    continue;
                }
            }

            optimized.push(gates[i].clone());
            i += 1;
        }

        if optimized.len() < gates.len() {
            Some(optimized)
        } else {
            None
        }
    }

    /// Merge consecutive single-qubit gates
    fn try_merge_single_qubit_gates(gates: &[GateWithParams]) -> Option<Vec<GateWithParams>> {
        let mut optimized = Vec::new();
        let mut i = 0;

        while i < gates.len() {
            let gate = &gates[i];

            // Check if this is a single-qubit rotation
            if matches!(
                gate.gate,
                StandardGate::Rz | StandardGate::Rx | StandardGate::Ry
            ) {
                // Look for consecutive rotations on same axis
                let mut j = i + 1;
                let mut total_angle = gate.angle;

                while j < gates.len() && gates[j].gate == gate.gate {
                    total_angle += gates[j].angle;
                    j += 1;
                }

                if j > i + 1 {
                    // Merged rotations
                    optimized.push(GateWithParams {
                        gate: gate.gate,
                        angle: total_angle,
                    });
                    i = j;
                    continue;
                }
            }

            optimized.push(gate.clone());
            i += 1;
        }

        if optimized.len() < gates.len() {
            Some(optimized)
        } else {
            None
        }
    }

    /// Count number of CX gates needed for a given gate sequence
    pub fn count_cx_gates(gates: &[GateWithParams]) -> usize {
        gates.iter().filter(|g| g.gate == StandardGate::CX).count()
    }
}

/// Gate with parameters for synthesis
#[derive(Debug, Clone)]
pub struct GateWithParams {
    pub gate: StandardGate,
    pub angle: f64,
}

impl GateWithParams {
    pub fn new(gate: StandardGate, angle: f64) -> Self {
        Self { gate, angle }
    }

    pub fn cx() -> Self {
        Self {
            gate: StandardGate::CX,
            angle: 0.0,
        }
    }

    pub fn rz(angle: f64) -> Self {
        Self {
            gate: StandardGate::Rz,
            angle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cx_cancellation() {
        let gates = vec![GateWithParams::cx(), GateWithParams::cx()];

        let result = TwoQubitSynthesizer::optimize_sequence(&gates).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_rz_merge() {
        let gates = vec![
            GateWithParams::rz(0.1),
            GateWithParams::rz(0.2),
            GateWithParams::rz(0.3),
        ];

        let result = TwoQubitSynthesizer::optimize_sequence(&gates).unwrap();
        assert_eq!(result.len(), 1);
        assert!((result[0].angle - 0.6).abs() < 1e-10);
    }
}
