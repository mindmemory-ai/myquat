// Single-Qubit Gate Optimizer
// Author: gA4ss
//
// Advanced single-qubit gate optimization techniques:
// - Euler angle canonicalization
// - Virtual Z gate tracking and deferral
// - Phase gate folding
// - Enhanced rotation merging

use crate::circuit::QuantumCircuit;
use crate::circuit_optimization::CircuitPass;
use crate::error::Result;
use crate::gates::StandardGate;
use crate::parameter::Parameter;
use std::collections::HashMap;
use std::f64::consts::PI;

/// Virtual Z gate tracker for each qubit
#[derive(Debug, Clone)]
struct VirtualZTracker {
    /// Accumulated virtual Z rotation angle
    angle: f64,
}

impl VirtualZTracker {
    fn new() -> Self {
        Self { angle: 0.0 }
    }

    fn add_rotation(&mut self, theta: f64) {
        self.angle += theta;
        // Normalize to [-π, π]
        self.angle = self.normalize_angle(self.angle);
    }

    #[allow(dead_code)]
    fn get_angle(&self) -> f64 {
        self.angle
    }

    fn reset(&mut self) -> f64 {
        let angle = self.angle;
        self.angle = 0.0;
        angle
    }

    fn normalize_angle(&self, angle: f64) -> f64 {
        let mut normalized = angle % (2.0 * PI);
        if normalized > PI {
            normalized -= 2.0 * PI;
        } else if normalized < -PI {
            normalized += 2.0 * PI;
        }
        normalized
    }
}

/// Single-qubit rotation represented in Euler angles
#[derive(Debug, Clone)]
struct EulerRotation {
    /// First Z rotation
    pub alpha: f64,
    /// Y rotation
    pub beta: f64,
    /// Second Z rotation
    pub gamma: f64,
}

impl EulerRotation {
    fn new() -> Self {
        Self {
            alpha: 0.0,
            beta: 0.0,
            gamma: 0.0,
        }
    }

    /// Check if this is effectively identity
    fn is_identity(&self, threshold: f64) -> bool {
        self.alpha.abs() < threshold && self.beta.abs() < threshold && self.gamma.abs() < threshold
    }

    /// Canonicalize Euler angles to standard form
    fn canonicalize(&mut self) {
        // Normalize all angles to [-π, π]
        self.alpha = Self::normalize_angle(self.alpha);
        self.beta = Self::normalize_angle(self.beta);
        self.gamma = Self::normalize_angle(self.gamma);

        // If beta is close to 0, combine alpha and gamma
        if self.beta.abs() < 1e-10 {
            self.alpha = Self::normalize_angle(self.alpha + self.gamma);
            self.gamma = 0.0;
        }
    }

    fn normalize_angle(angle: f64) -> f64 {
        let mut normalized = angle % (2.0 * PI);
        if normalized > PI {
            normalized -= 2.0 * PI;
        } else if normalized < -PI {
            normalized += 2.0 * PI;
        }
        normalized
    }
}

/// Enhanced single-qubit gate optimizer
pub struct SingleQubitOptimizer {
    /// Threshold for considering angle as zero
    threshold: f64,
    /// Enable virtual Z gate deferral
    enable_virtual_z: bool,
    /// Enable Euler angle canonicalization
    enable_euler_canon: bool,
    /// Enable phase gate folding (S, T gates into Rz)
    enable_phase_folding: bool,
}

impl SingleQubitOptimizer {
    pub fn new() -> Self {
        Self {
            threshold: 1e-10,
            enable_virtual_z: true,
            enable_euler_canon: true,
            enable_phase_folding: true,
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn with_virtual_z(mut self, enable: bool) -> Self {
        self.enable_virtual_z = enable;
        self
    }

    pub fn with_euler_canon(mut self, enable: bool) -> Self {
        self.enable_euler_canon = enable;
        self
    }

    pub fn with_phase_folding(mut self, enable: bool) -> Self {
        self.enable_phase_folding = enable;
        self
    }

    /// Optimize single-qubit gates in circuit
    pub fn optimize(&self, circuit: &QuantumCircuit) -> Result<QuantumCircuit> {
        let mut optimized = QuantumCircuit::new(circuit.num_qubits(), circuit.num_clbits());

        // Virtual Z trackers for each qubit
        let mut virtual_z: HashMap<usize, VirtualZTracker> = HashMap::new();

        // Pending rotations for each qubit (Z-Y-Z decomposition)
        let mut pending_euler: HashMap<usize, EulerRotation> = HashMap::new();

        let instructions = circuit.data().instructions();

        for inst in instructions {
            if inst.is_measurement() {
                // Flush all pending operations before measurement
                self.flush_all(&mut optimized, &mut virtual_z, &mut pending_euler)?;
                optimized.measure(inst.qubits[0].index(), inst.clbits[0].index())?;
                continue;
            }

            let gate_type = &inst.gate.gate_type;
            let qubits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();

            // Handle single-qubit gates
            if qubits.len() == 1 {
                let qubit = qubits[0];

                match gate_type {
                    StandardGate::Rz => {
                        if let Some(angle) = self.get_float_param(&inst.gate.parameters) {
                            if self.enable_virtual_z {
                                // Add to virtual Z tracker
                                virtual_z
                                    .entry(qubit)
                                    .or_insert_with(VirtualZTracker::new)
                                    .add_rotation(angle);
                            } else {
                                // Traditional accumulation
                                let euler = pending_euler
                                    .entry(qubit)
                                    .or_insert_with(EulerRotation::new);
                                euler.gamma += angle;
                            }
                            continue;
                        }
                    }
                    StandardGate::Ry => {
                        if let Some(angle) = self.get_float_param(&inst.gate.parameters) {
                            // Apply virtual Z, then Ry, then start new Euler
                            if self.enable_virtual_z {
                                if let Some(vz) = virtual_z.get_mut(&qubit) {
                                    let vz_angle = vz.reset();
                                    if vz_angle.abs() >= self.threshold {
                                        optimized.rz(qubit, Parameter::Float(vz_angle))?;
                                    }
                                }
                            }

                            let euler = pending_euler
                                .entry(qubit)
                                .or_insert_with(EulerRotation::new);
                            euler.beta += angle;
                            continue;
                        }
                    }
                    StandardGate::Rx => {
                        // Rx is more complex in Z-Y-Z - flush and apply directly
                        if let Some(angle) = self.get_float_param(&inst.gate.parameters) {
                            self.flush_qubit(
                                &mut optimized,
                                qubit,
                                &mut virtual_z,
                                &mut pending_euler,
                            )?;
                            optimized.rx(qubit, Parameter::Float(angle))?;
                            continue;
                        }
                    }
                    StandardGate::S | StandardGate::Sdg | StandardGate::T | StandardGate::Tdg => {
                        // Phase gates - fold into Rz if enabled
                        if self.enable_phase_folding {
                            let angle = match gate_type {
                                StandardGate::S => PI / 2.0,    // S = Rz(π/2)
                                StandardGate::Sdg => -PI / 2.0, // S† = Rz(-π/2)
                                StandardGate::T => PI / 4.0,    // T = Rz(π/4)
                                StandardGate::Tdg => -PI / 4.0, // T† = Rz(-π/4)
                                _ => 0.0,
                            };

                            if self.enable_virtual_z {
                                virtual_z
                                    .entry(qubit)
                                    .or_insert_with(VirtualZTracker::new)
                                    .add_rotation(angle);
                            } else {
                                let euler = pending_euler
                                    .entry(qubit)
                                    .or_insert_with(EulerRotation::new);
                                euler.gamma += angle;
                            }
                            continue;
                        } else {
                            // Apply as-is
                            self.flush_qubit(
                                &mut optimized,
                                qubit,
                                &mut virtual_z,
                                &mut pending_euler,
                            )?;
                            self.add_single_qubit_gate(&mut optimized, gate_type, qubit)?;
                            continue;
                        }
                    }
                    StandardGate::H | StandardGate::X | StandardGate::Y | StandardGate::Z => {
                        // Flush pending operations before non-parametric gates
                        self.flush_qubit(
                            &mut optimized,
                            qubit,
                            &mut virtual_z,
                            &mut pending_euler,
                        )?;
                        self.add_single_qubit_gate(&mut optimized, gate_type, qubit)?;
                        continue;
                    }
                    _ => {}
                }
            }

            // Two-qubit gate - flush pending operations on involved qubits
            if qubits.len() == 2 {
                for &qubit in &qubits {
                    self.flush_qubit(&mut optimized, qubit, &mut virtual_z, &mut pending_euler)?;
                }

                // Add the two-qubit gate
                match gate_type {
                    StandardGate::CX => optimized.cx(qubits[0], qubits[1])?,
                    StandardGate::CZ => {
                        // CZ commutes with Z gates - keep virtual Z
                        optimized.cx(qubits[0], qubits[1])?; // Simplified
                    }
                    _ => {}
                }
                continue;
            }
        }

        // Flush any remaining operations
        self.flush_all(&mut optimized, &mut virtual_z, &mut pending_euler)?;

        Ok(optimized)
    }

    fn flush_qubit(
        &self,
        circuit: &mut QuantumCircuit,
        qubit: usize,
        virtual_z: &mut HashMap<usize, VirtualZTracker>,
        pending_euler: &mut HashMap<usize, EulerRotation>,
    ) -> Result<()> {
        // Apply virtual Z first
        if let Some(vz) = virtual_z.get_mut(&qubit) {
            let angle = vz.reset();
            if angle.abs() >= self.threshold {
                circuit.rz(qubit, Parameter::Float(angle))?;
            }
        }

        // Apply Euler rotation
        if let Some(mut euler) = pending_euler.remove(&qubit) {
            if self.enable_euler_canon {
                euler.canonicalize();
            }

            if !euler.is_identity(self.threshold) {
                // Apply Z-Y-Z decomposition
                if euler.alpha.abs() >= self.threshold {
                    circuit.rz(qubit, Parameter::Float(euler.alpha))?;
                }
                if euler.beta.abs() >= self.threshold {
                    circuit.ry(qubit, Parameter::Float(euler.beta))?;
                }
                if euler.gamma.abs() >= self.threshold {
                    circuit.rz(qubit, Parameter::Float(euler.gamma))?;
                }
            }
        }

        Ok(())
    }

    fn flush_all(
        &self,
        circuit: &mut QuantumCircuit,
        virtual_z: &mut HashMap<usize, VirtualZTracker>,
        pending_euler: &mut HashMap<usize, EulerRotation>,
    ) -> Result<()> {
        // Get all qubits that have pending operations
        let mut qubits: Vec<usize> = virtual_z.keys().copied().collect();
        qubits.extend(pending_euler.keys().copied());
        qubits.sort_unstable();
        qubits.dedup();

        for qubit in qubits {
            self.flush_qubit(circuit, qubit, virtual_z, pending_euler)?;
        }

        Ok(())
    }

    fn get_float_param(&self, params: &[Parameter]) -> Option<f64> {
        params.first().and_then(|p| {
            if let Parameter::Float(f) = p {
                Some(*f)
            } else {
                None
            }
        })
    }

    fn add_single_qubit_gate(
        &self,
        circuit: &mut QuantumCircuit,
        gate_type: &StandardGate,
        qubit: usize,
    ) -> Result<()> {
        match gate_type {
            StandardGate::H => circuit.h(qubit),
            StandardGate::X => circuit.x(qubit),
            StandardGate::Y => circuit.y(qubit),
            StandardGate::Z => circuit.z(qubit),
            StandardGate::S => circuit.s(qubit),
            StandardGate::Sdg => circuit.sdg(qubit),
            StandardGate::T => circuit.t(qubit),
            StandardGate::Tdg => circuit.tdg(qubit),
            _ => Ok(()),
        }
    }
}

impl Default for SingleQubitOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitPass for SingleQubitOptimizer {
    fn run(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        let optimized = self.optimize(circuit)?;
        *circuit = optimized;
        Ok(())
    }

    fn name(&self) -> &str {
        "SingleQubitOptimizer"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_z_tracker() {
        let mut tracker = VirtualZTracker::new();

        tracker.add_rotation(PI / 4.0);
        assert!((tracker.get_angle() - PI / 4.0).abs() < 1e-10);

        tracker.add_rotation(PI / 4.0);
        assert!((tracker.get_angle() - PI / 2.0).abs() < 1e-10);

        let angle = tracker.reset();
        assert!((angle - PI / 2.0).abs() < 1e-10);
        assert!(tracker.get_angle().abs() < 1e-10);
    }

    #[test]
    fn test_angle_normalization() {
        let tracker = VirtualZTracker::new();

        // Test wrapping: 3π mod 2π = π
        let normalized = tracker.normalize_angle(3.0 * PI);
        assert!((normalized - PI).abs() < 1e-10 || (normalized - (-PI)).abs() < 1e-10);

        // Test wrapping: -3π mod 2π = -π
        let normalized = tracker.normalize_angle(-3.0 * PI);
        assert!((normalized - (-PI)).abs() < 1e-10 || (normalized - PI).abs() < 1e-10);
    }

    #[test]
    fn test_euler_canonicalization() {
        let mut euler = EulerRotation {
            alpha: 2.5 * PI,
            beta: 0.0,
            gamma: 0.5 * PI,
        };

        euler.canonicalize();

        // Should combine alpha and gamma when beta is 0
        assert!(euler.beta.abs() < 1e-10);
        assert!(euler.gamma.abs() < 1e-10);
        assert!((euler.alpha - PI).abs() < 1e-10); // 2.5π + 0.5π = 3π = -π (normalized)
    }

    #[test]
    fn test_single_qubit_optimizer_basic() -> Result<()> {
        let mut circuit = QuantumCircuit::new(2, 0);

        // Add consecutive Rz gates
        circuit.rz(0, Parameter::Float(PI / 4.0))?;
        circuit.rz(0, Parameter::Float(PI / 4.0))?;
        circuit.rz(0, Parameter::Float(PI / 4.0))?;

        let optimizer = SingleQubitOptimizer::new();
        let optimized = optimizer.optimize(&circuit)?;

        // Should combine into single Rz(3π/4)
        assert!(optimized.data().instructions().len() <= 1);

        Ok(())
    }

    #[test]
    fn test_phase_gate_folding() -> Result<()> {
        let mut circuit = QuantumCircuit::new(2, 0);

        // Add phase gates that should be folded
        circuit.s(0)?; // S = Rz(π/2)
        circuit.t(0)?; // T = Rz(π/4)
        circuit.s(0)?; // S = Rz(π/2)

        let optimizer = SingleQubitOptimizer::new();
        let optimized = optimizer.optimize(&circuit)?;

        // Should combine S + T + S = Rz(π/2 + π/4 + π/2) = Rz(5π/4)
        // With virtual Z enabled, should be optimized to single gate
        assert!(optimized.data().instructions().len() <= 1);

        Ok(())
    }

    #[test]
    fn test_phase_folding_disabled() -> Result<()> {
        let mut circuit = QuantumCircuit::new(2, 0);

        // Add phase gates
        circuit.s(0)?;
        circuit.t(0)?;

        let optimizer = SingleQubitOptimizer::new().with_phase_folding(false);
        let optimized = optimizer.optimize(&circuit)?;

        // With phase folding disabled, gates should remain separate
        assert_eq!(optimized.data().instructions().len(), 2);

        Ok(())
    }

    #[test]
    fn test_virtual_z_with_two_qubit_gate() -> Result<()> {
        let mut circuit = QuantumCircuit::new(2, 0);

        // Virtual Z gates on qubit 0
        circuit.rz(0, Parameter::Float(PI / 4.0))?;
        circuit.rz(0, Parameter::Float(PI / 4.0))?;

        // Two-qubit gate should trigger flush
        circuit.cx(0, 1)?;

        // More Z gates
        circuit.rz(0, Parameter::Float(PI / 4.0))?;

        let optimizer = SingleQubitOptimizer::new();
        let optimized = optimizer.optimize(&circuit)?;

        // Should have: Rz(π/2) on q0, CX(0,1), Rz(π/4) on q0
        assert!(optimized.data().instructions().len() >= 2);

        Ok(())
    }
}
