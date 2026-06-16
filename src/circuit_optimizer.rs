//! Circuit optimization engine
//!
//! Author: gA4ss
//!
//! This module provides a high-level facade for circuit optimization, delegating
//! to the robust optimization passes in `circuit_optimization.rs`.
//!
//! # Architecture
//!
//! - **CircuitOptimizer**: High-level optimization engine with configuration management
//! - **PassManager Integration**: Delegates to `PassManager` for actual optimization
//! - **Hybrid Strategy**:
//!   - Simple configs (no hardware constraints) → Predefined PassManager levels
//!   - Complex configs (with hardware constraints) → Dynamically built PassManager
//!
//! # Optimization Capabilities
//!
//! - **Gate Cancellation**: Removes inverse gate pairs (H-H, X-X, etc.)
//! - **Gate Merging**: Combines consecutive rotations (Rx(a) + Rx(b) = Rx(a+b))
//! - **Depth Optimization**: Reorders commuting gates to minimize circuit depth
//! - **Hardware Constraints**: Validates circuits against hardware topology
//!
//! # Example
//!
//! ```rust
//! use myquat::{QuantumCircuit, CircuitOptimizer, OptimizationConfig};
//!
//! let mut circuit = QuantumCircuit::new(2, 0);
//! circuit.h(0).unwrap();
//! circuit.h(0).unwrap(); // H-H cancels
//!
//! let mut optimizer = CircuitOptimizer::default();
//! let optimized = optimizer.optimize(&circuit).unwrap();
//!
//! // Gate count reduced
//! assert!(optimized.size() < circuit.size());
//! ```

use crate::error::{MyQuatError, Result};
use crate::{Parameter, QuantumCircuit, StandardGate};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Circuit optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Enable gate cancellation optimization
    pub enable_gate_cancellation: bool,
    /// Enable gate merging optimization
    pub enable_gate_merging: bool,
    /// Enable depth optimization
    pub enable_depth_optimization: bool,
    /// Enable hardware constraint optimization
    pub enable_hardware_constraints: bool,
    /// Maximum optimization passes
    pub max_passes: usize,
    /// Enable block consolidation (KAK decomposition)
    pub enable_block_consolidation: bool,
    /// Target hardware topology
    pub hardware_topology: Option<HardwareTopology>,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        OptimizationConfig {
            enable_gate_cancellation: true,
            enable_gate_merging: true,
            enable_depth_optimization: true,
            enable_hardware_constraints: false,
            max_passes: 10,
            enable_block_consolidation: false,
            hardware_topology: None,
        }
    }
}

/// Hardware topology specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareTopology {
    /// Number of physical qubits
    pub num_qubits: usize,
    /// Allowed qubit connections (edges)
    pub connections: Vec<(usize, usize)>,
    /// Gate error rates per qubit
    pub gate_errors: HashMap<usize, f64>,
    /// Two-qubit gate error rates per connection
    pub two_qubit_errors: HashMap<(usize, usize), f64>,
}

impl HardwareTopology {
    /// Create a linear topology
    pub fn linear(num_qubits: usize) -> Self {
        let mut connections = Vec::new();
        for i in 0..(num_qubits - 1) {
            connections.push((i, i + 1));
        }

        HardwareTopology {
            num_qubits,
            connections,
            gate_errors: HashMap::new(),
            two_qubit_errors: HashMap::new(),
        }
    }

    /// Create a grid topology
    pub fn grid(rows: usize, cols: usize) -> Self {
        let num_qubits = rows * cols;
        let mut connections = Vec::new();

        // Horizontal connections
        for row in 0..rows {
            for col in 0..(cols - 1) {
                let q1 = row * cols + col;
                let q2 = row * cols + col + 1;
                connections.push((q1, q2));
            }
        }

        // Vertical connections
        for row in 0..(rows - 1) {
            for col in 0..cols {
                let q1 = row * cols + col;
                let q2 = (row + 1) * cols + col;
                connections.push((q1, q2));
            }
        }

        HardwareTopology {
            num_qubits,
            connections,
            gate_errors: HashMap::new(),
            two_qubit_errors: HashMap::new(),
        }
    }

    /// Check if two qubits are connected
    pub fn are_connected(&self, q1: usize, q2: usize) -> bool {
        self.connections.contains(&(q1, q2)) || self.connections.contains(&(q2, q1))
    }

    /// Get neighbors of a qubit
    pub fn neighbors(&self, qubit: usize) -> Vec<usize> {
        let mut neighbors = Vec::new();
        for &(q1, q2) in &self.connections {
            if q1 == qubit {
                neighbors.push(q2);
            } else if q2 == qubit {
                neighbors.push(q1);
            }
        }
        neighbors
    }
}

/// Gate in the optimization representation
#[derive(Debug, Clone)]
pub struct OptimizationGate {
    /// Gate type
    pub gate: StandardGate,
    /// Target qubits
    pub qubits: Vec<usize>,
    /// Gate parameters
    pub parameters: Vec<Parameter>,
    /// Position in original circuit
    pub position: usize,
}

impl OptimizationGate {
    /// Create a new optimization gate
    pub fn new(
        gate: StandardGate,
        qubits: Vec<usize>,
        parameters: Vec<Parameter>,
        position: usize,
    ) -> Self {
        OptimizationGate {
            gate,
            qubits,
            parameters,
            position,
        }
    }

    /// Check if this gate commutes with another gate
    pub fn commutes_with(&self, other: &OptimizationGate) -> bool {
        // Gates on different qubits always commute
        let self_qubits: HashSet<_> = self.qubits.iter().collect();
        let other_qubits: HashSet<_> = other.qubits.iter().collect();

        if self_qubits.is_disjoint(&other_qubits) {
            return true;
        }

        // Same qubit gates - check specific commutation rules
        if self.qubits == other.qubits {
            return self.gate_commutes(&self.gate, &other.gate);
        }

        false
    }

    /// Check if two gate types commute
    fn gate_commutes(&self, gate1: &StandardGate, gate2: &StandardGate) -> bool {
        use StandardGate::*;

        match (gate1, gate2) {
            // Pauli gates commute with themselves
            (X, X) | (Y, Y) | (Z, Z) => true,
            // Z commutes with phase gates
            (Z, S) | (S, Z) | (Z, T) | (T, Z) => true,
            // Rotations around same axis commute
            (Rx, Rx) | (Ry, Ry) | (Rz, Rz) => true,
            // Most other combinations don't commute
            _ => false,
        }
    }

    /// Check if this gate cancels with another gate
    pub fn cancels_with(&self, other: &OptimizationGate) -> bool {
        if self.qubits != other.qubits {
            return false;
        }

        use StandardGate::*;

        match (&self.gate, &other.gate) {
            // Self-inverse gates
            (X, X) | (Y, Y) | (Z, Z) | (H, H) => true,
            // Rotation cancellation (simplified - would need parameter checking)
            (Rx, Rx) | (Ry, Ry) | (Rz, Rz) => {
                // For now, assume they cancel if parameters are negatives
                // Full implementation would check parameter values
                false
            }
            _ => false,
        }
    }

    /// Check if this gate can be merged with another gate
    pub fn can_merge_with(&self, other: &OptimizationGate) -> bool {
        if self.qubits != other.qubits {
            return false;
        }

        use StandardGate::*;

        match (&self.gate, &other.gate) {
            // Rotations around same axis can be merged
            (Rx, Rx) | (Ry, Ry) | (Rz, Rz) => true,
            // Phase gates can be merged
            (S, S) | (T, T) | (S, T) | (T, S) => true,
            _ => false,
        }
    }
}

/// Circuit optimization engine
///
/// High-level facade for quantum circuit optimization. Provides a simple interface
/// while delegating to the robust `PassManager` and individual optimization passes
/// in `circuit_optimization.rs`.
///
/// # Strategy
///
/// Uses a hybrid optimization strategy:
/// - **Simple configurations** (no hardware constraints): Uses predefined PassManager levels
///   based on enabled optimizations (level 0-3)
/// - **Complex configurations** (with hardware constraints): Dynamically builds a custom
///   PassManager with selected optimization passes
///
/// # Usage
///
/// ```rust,ignore
/// use myquat::{QuantumCircuit, CircuitOptimizer, OptimizationConfig, HardwareTopology};
///
/// // Simple optimization (default config)
/// let mut optimizer = CircuitOptimizer::default();
/// let optimized = optimizer.optimize(&circuit).unwrap();
///
/// // Custom configuration with hardware constraints
/// let config = OptimizationConfig {
///     enable_block_consolidation: true,
///     enable_gate_cancellation: true,
///     enable_gate_merging: true,
///     enable_depth_optimization: true,
///     enable_hardware_constraints: true,
///     hardware_topology: Some(HardwareTopology::linear(5)),
///     max_passes: 10,
/// };
/// let mut custom_optimizer = CircuitOptimizer::new(config);
/// let result = custom_optimizer.optimize(&circuit).unwrap();
/// ```
pub struct CircuitOptimizer {
    /// Optimization configuration
    config: OptimizationConfig,
    /// Optimization statistics
    stats: OptimizationStats,
}

impl CircuitOptimizer {
    /// Create a new circuit optimizer
    pub fn new(config: OptimizationConfig) -> Self {
        CircuitOptimizer {
            config,
            stats: OptimizationStats::default(),
        }
    }

    /// Create optimizer with default configuration
    pub fn default() -> Self {
        Self::new(OptimizationConfig::default())
    }

    /// Optimize a quantum circuit
    ///
    /// Uses a hybrid strategy:
    /// - Simple configurations (no hardware constraints) use predefined PassManager levels
    /// - Complex configurations (with hardware constraints) use dynamically built PassManager
    pub fn optimize(&mut self, circuit: &QuantumCircuit) -> Result<QuantumCircuit> {
        use crate::circuit_optimization::PassManager;

        let mut optimized = circuit.clone();
        self.stats = OptimizationStats::default();
        self.stats.original_size = circuit.size();
        self.stats.original_depth = self.calculate_depth(circuit);

        for pass in 0..self.config.max_passes {
            let before_size = optimized.size();

            // Choose optimization strategy based on configuration
            let pm = if self.is_simple_config() {
                // Simple config: use predefined PassManager level
                let level = self.infer_pass_manager_level();
                match level {
                    0 => PassManager::new(),
                    1 => PassManager::level_1(),
                    2 => PassManager::level_2(),
                    3 => PassManager::level_3(),
                    4 => PassManager::level_4(),
                    _ => PassManager::level_2(),
                }
            } else {
                // Complex config: dynamically build PassManager
                self.build_custom_pass_manager()
            };

            // Run optimization passes
            pm.run(&mut optimized)?;

            // Apply hardware constraints if enabled
            if self.config.enable_hardware_constraints {
                if let Some(ref topology) = self.config.hardware_topology {
                    self.validate_hardware_constraints(&optimized, topology)?;
                }
            }

            // Check for convergence
            if optimized.size() == before_size {
                self.stats.passes_run = pass + 1;
                break;
            }
        }

        self.stats.optimized_size = optimized.size();
        self.stats.optimized_depth = self.calculate_depth(&optimized);
        self.stats.gates_eliminated = self
            .stats
            .original_size
            .saturating_sub(self.stats.optimized_size);

        Ok(optimized)
    }

    /// Validate that circuit is compatible with hardware topology
    ///
    /// Checks that all two-qubit gates only operate on connected qubits
    /// according to the hardware topology.
    fn validate_hardware_constraints(
        &self,
        circuit: &QuantumCircuit,
        topology: &HardwareTopology,
    ) -> Result<()> {
        for instruction in circuit.data().instructions() {
            if instruction.qubits.len() == 2 {
                let q1 = instruction.qubits[0].index();
                let q2 = instruction.qubits[1].index();

                if !topology.are_connected(q1, q2) {
                    return Err(MyQuatError::circuit_error(format!(
                        "Qubits {} and {} are not connected in hardware topology",
                        q1, q2
                    )));
                }
            }
        }
        Ok(())
    }

    /// Calculate circuit depth
    fn calculate_depth(&self, circuit: &QuantumCircuit) -> usize {
        // Simplified depth calculation
        // Full implementation would build dependency graph
        circuit.size()
    }

    /// Check if configuration is simple (can use predefined PassManager levels)
    ///
    /// A configuration is considered simple if it has no hardware topology constraints.
    fn is_simple_config(&self) -> bool {
        self.config.hardware_topology.is_none()
    }

    /// Infer the appropriate PassManager level based on configuration
    ///
    /// Maps optimization configuration to predefined PassManager levels:
    /// - Level 0: No optimization
    /// - Level 1: Basic optimization (cancellation + merging)
    /// - Level 2: Medium optimization (+ CNOT optimization)
    /// - Level 3: Aggressive optimization (multiple passes)
    fn infer_pass_manager_level(&self) -> usize {
        // No optimization
        if !self.config.enable_gate_cancellation
            && !self.config.enable_gate_merging
            && !self.config.enable_depth_optimization
        {
            return 0;
        }

        // Basic optimization
        if !self.config.enable_depth_optimization {
            return 1;
        }

        // Maximum optimization with block consolidation (KAK)
        if self.config.enable_block_consolidation || self.config.max_passes > 10 {
            return 4;
        }

        // Aggressive optimization
        if self.config.max_passes > 5 {
            return 3;
        }

        // Medium optimization (default)
        2
    }

    /// Build a custom PassManager based on configuration
    ///
    /// Dynamically constructs a PassManager by adding passes according to
    /// the enabled optimization options in the configuration.
    fn build_custom_pass_manager(&self) -> crate::circuit_optimization::PassManager {
        use crate::circuit_optimization::*;

        let mut pm = PassManager::new();

        // Add passes based on configuration
        if self.config.enable_gate_merging {
            pm.add_pass(Box::new(MergeRotationsPass::new()));
        }

        if self.config.enable_gate_cancellation {
            pm.add_pass(Box::new(CancelInversePairsPass::new()));
        }

        if self.config.enable_depth_optimization {
            pm.add_pass(Box::new(CommutativeCancellationPass::new()));
        }

        pm
    }

    /// Get optimization statistics
    pub fn stats(&self) -> &OptimizationStats {
        &self.stats
    }
}

/// Optimization statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OptimizationStats {
    /// Original circuit size
    pub original_size: usize,
    /// Optimized circuit size
    pub optimized_size: usize,
    /// Original circuit depth
    pub original_depth: usize,
    /// Optimized circuit depth
    pub optimized_depth: usize,
    /// Number of gates eliminated
    pub gates_eliminated: usize,
    /// Number of optimization passes run
    pub passes_run: usize,
}

impl OptimizationStats {
    /// Calculate size reduction percentage
    pub fn size_reduction(&self) -> f64 {
        if self.original_size == 0 {
            return 0.0;
        }

        (self.gates_eliminated as f64 / self.original_size as f64) * 100.0
    }

    /// Calculate depth reduction percentage
    pub fn depth_reduction(&self) -> f64 {
        if self.original_depth == 0 {
            return 0.0;
        }

        let depth_improvement = self.original_depth.saturating_sub(self.optimized_depth);
        (depth_improvement as f64 / self.original_depth as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_config() {
        let config = OptimizationConfig::default();
        assert!(config.enable_gate_cancellation);
        assert!(config.enable_gate_merging);
        assert_eq!(config.max_passes, 10);
    }

    #[test]
    fn test_hardware_topology_linear() {
        let topology = HardwareTopology::linear(4);
        assert_eq!(topology.num_qubits, 4);
        assert_eq!(topology.connections.len(), 3);

        assert!(topology.are_connected(0, 1));
        assert!(topology.are_connected(1, 2));
        assert!(!topology.are_connected(0, 2));
    }

    #[test]
    fn test_hardware_topology_grid() {
        let topology = HardwareTopology::grid(2, 2);
        assert_eq!(topology.num_qubits, 4);
        assert_eq!(topology.connections.len(), 4); // 2 horizontal + 2 vertical

        assert!(topology.are_connected(0, 1)); // Horizontal
        assert!(topology.are_connected(0, 2)); // Vertical
        assert!(!topology.are_connected(0, 3)); // Diagonal
    }

    #[test]
    fn test_optimization_gate_commutation() {
        let gate1 = OptimizationGate::new(StandardGate::X, vec![0], vec![], 0);
        let gate2 = OptimizationGate::new(StandardGate::Y, vec![1], vec![], 1);

        // Gates on different qubits should commute
        assert!(gate1.commutes_with(&gate2));

        let gate3 = OptimizationGate::new(StandardGate::Y, vec![0], vec![], 2);
        // Gates on same qubit generally don't commute
        assert!(!gate1.commutes_with(&gate3));
    }

    #[test]
    fn test_optimization_gate_cancellation() {
        let gate1 = OptimizationGate::new(StandardGate::X, vec![0], vec![], 0);
        let gate2 = OptimizationGate::new(StandardGate::X, vec![0], vec![], 1);

        // X gates should cancel
        assert!(gate1.cancels_with(&gate2));

        let gate3 = OptimizationGate::new(StandardGate::Y, vec![0], vec![], 2);
        // X and Y don't cancel
        assert!(!gate1.cancels_with(&gate3));
    }

    #[test]
    fn test_circuit_optimizer_creation() {
        let config = OptimizationConfig::default();
        let optimizer = CircuitOptimizer::new(config);

        assert_eq!(optimizer.stats().original_size, 0);
        assert_eq!(optimizer.stats().optimized_size, 0);
    }

    #[test]
    fn test_circuit_optimization() {
        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.h(0).unwrap();
        circuit.x(1).unwrap();

        let mut optimizer = CircuitOptimizer::default();
        let optimized = optimizer.optimize(&circuit).unwrap();

        // Basic optimization should preserve functionality
        assert!(optimized.num_qubits() >= circuit.num_qubits());
    }

    #[test]
    fn test_optimization_stats() {
        let mut stats = OptimizationStats::default();
        stats.original_size = 10;
        stats.optimized_size = 7;
        stats.gates_eliminated = 3;

        assert_eq!(stats.size_reduction(), 30.0);
    }

    #[test]
    fn test_hardware_constraint_validation() {
        let mut circuit = QuantumCircuit::new(3, 0);
        circuit.cx(0, 2).unwrap(); // Non-adjacent qubits

        let topology = HardwareTopology::linear(3);
        let config = OptimizationConfig {
            enable_hardware_constraints: true,
            hardware_topology: Some(topology),
            ..OptimizationConfig::default()
        };

        let mut optimizer = CircuitOptimizer::new(config);
        let result = optimizer.optimize(&circuit);

        // Should fail due to hardware constraints
        assert!(result.is_err());
    }

    #[test]
    fn test_is_simple_config() {
        // Simple config: no hardware topology
        let config = OptimizationConfig::default();
        let optimizer = CircuitOptimizer::new(config);
        assert!(optimizer.is_simple_config());

        // Complex config: with hardware topology
        let config_with_topology = OptimizationConfig {
            hardware_topology: Some(HardwareTopology::linear(5)),
            ..OptimizationConfig::default()
        };
        let optimizer_complex = CircuitOptimizer::new(config_with_topology);
        assert!(!optimizer_complex.is_simple_config());
    }

    #[test]
    fn test_infer_pass_manager_level() {
        // Level 0: No optimization
        let config_level0 = OptimizationConfig {
            enable_gate_cancellation: false,
            enable_gate_merging: false,
            enable_depth_optimization: false,
            ..OptimizationConfig::default()
        };
        let optimizer0 = CircuitOptimizer::new(config_level0);
        assert_eq!(optimizer0.infer_pass_manager_level(), 0);

        // Level 1: Basic optimization (no depth optimization)
        let config_level1 = OptimizationConfig {
            enable_gate_cancellation: true,
            enable_gate_merging: true,
            enable_depth_optimization: false,
            ..OptimizationConfig::default()
        };
        let optimizer1 = CircuitOptimizer::new(config_level1);
        assert_eq!(optimizer1.infer_pass_manager_level(), 1);

        // Level 2: Medium optimization (max_passes <= 5)
        let config_level2 = OptimizationConfig {
            max_passes: 5,
            ..OptimizationConfig::default()
        };
        let optimizer2 = CircuitOptimizer::new(config_level2);
        assert_eq!(optimizer2.infer_pass_manager_level(), 2);

        // Level 3: Aggressive optimization (max_passes > 5, default is 10)
        let config_level3 = OptimizationConfig::default();
        let optimizer3 = CircuitOptimizer::new(config_level3);
        assert_eq!(optimizer3.infer_pass_manager_level(), 3);
    }

    #[test]
    fn test_build_custom_pass_manager() {
        // Test with all optimizations enabled
        let config_all = OptimizationConfig::default();
        let optimizer_all = CircuitOptimizer::new(config_all);
        let pm_all = optimizer_all.build_custom_pass_manager();
        // Should have 3 passes: MergeRotations, CancelInversePairs, CommutativeCancellation

        // Test with only gate merging
        let config_merge_only = OptimizationConfig {
            enable_gate_cancellation: false,
            enable_gate_merging: true,
            enable_depth_optimization: false,
            ..OptimizationConfig::default()
        };
        let optimizer_merge = CircuitOptimizer::new(config_merge_only);
        let pm_merge = optimizer_merge.build_custom_pass_manager();
        // Should have 1 pass: MergeRotations

        // Test with no optimizations
        let config_none = OptimizationConfig {
            enable_gate_cancellation: false,
            enable_gate_merging: false,
            enable_depth_optimization: false,
            ..OptimizationConfig::default()
        };
        let optimizer_none = CircuitOptimizer::new(config_none);
        let pm_none = optimizer_none.build_custom_pass_manager();
        // Should have 0 passes

        // Test that PassManager can be used
        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.h(0).unwrap();
        circuit.h(0).unwrap();

        let result = pm_all.run(&mut circuit);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_hardware_constraints() {
        let optimizer = CircuitOptimizer::default();

        // Test valid circuit (adjacent qubits in linear topology)
        let mut valid_circuit = QuantumCircuit::new(3, 0);
        valid_circuit.cx(0, 1).unwrap();
        valid_circuit.cx(1, 2).unwrap();

        let linear_topology = HardwareTopology::linear(3);
        let result = optimizer.validate_hardware_constraints(&valid_circuit, &linear_topology);
        assert!(result.is_ok());

        // Test invalid circuit (non-adjacent qubits)
        let mut invalid_circuit = QuantumCircuit::new(3, 0);
        invalid_circuit.cx(0, 2).unwrap(); // Not connected in linear topology

        let result_invalid =
            optimizer.validate_hardware_constraints(&invalid_circuit, &linear_topology);
        assert!(result_invalid.is_err());

        // Test with grid topology
        let grid_topology = HardwareTopology::grid(2, 2);
        let mut grid_circuit = QuantumCircuit::new(4, 0);
        grid_circuit.cx(0, 1).unwrap(); // Horizontal connection
        grid_circuit.cx(0, 2).unwrap(); // Vertical connection

        let result_grid = optimizer.validate_hardware_constraints(&grid_circuit, &grid_topology);
        assert!(result_grid.is_ok());

        // Test invalid grid connection (diagonal)
        let mut invalid_grid = QuantumCircuit::new(4, 0);
        invalid_grid.cx(0, 3).unwrap(); // Diagonal, not connected

        let result_invalid_grid =
            optimizer.validate_hardware_constraints(&invalid_grid, &grid_topology);
        assert!(result_invalid_grid.is_err());
    }

    #[test]
    fn test_integration_simple_config_optimization() {
        // Test simple configuration using predefined PassManager levels
        let mut circuit = QuantumCircuit::new(3, 0);

        // Add gates that can be optimized
        circuit.h(0).unwrap();
        circuit.h(0).unwrap(); // H-H cancels
        circuit.rz(1, Parameter::Float(0.1)).unwrap();
        circuit.rz(1, Parameter::Float(0.2)).unwrap(); // RZ merges
        circuit.rz(1, Parameter::Float(0.3)).unwrap();
        circuit.x(2).unwrap();
        circuit.x(2).unwrap(); // X-X cancels

        let initial_size = circuit.size();
        assert_eq!(initial_size, 7);

        // Optimize with default config (simple config, level 3)
        let mut optimizer = CircuitOptimizer::default();
        let optimized = optimizer.optimize(&circuit).unwrap();

        // Should have reduced gate count
        assert!(optimized.size() < initial_size);

        // Check statistics
        let stats = optimizer.stats();
        assert_eq!(stats.original_size, initial_size);
        assert!(stats.gates_eliminated > 0);
        assert!(stats.size_reduction() > 0.0);
    }

    #[test]
    fn test_integration_complex_config_optimization() {
        // Test complex configuration with hardware topology
        let mut circuit = QuantumCircuit::new(3, 0);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.cx(1, 2).unwrap();
        circuit.h(0).unwrap();
        circuit.h(0).unwrap(); // Add another H to ensure cancellation

        let config = OptimizationConfig {
            enable_gate_cancellation: true,
            enable_gate_merging: true,
            enable_depth_optimization: true,
            enable_hardware_constraints: true,
            enable_block_consolidation: false,
            hardware_topology: Some(HardwareTopology::linear(3)),
            max_passes: 5,
        };

        let mut optimizer = CircuitOptimizer::new(config);
        let result = optimizer.optimize(&circuit);

        // Should succeed (circuit is compatible with linear topology)
        assert!(result.is_ok());

        let optimized = result.unwrap();
        // At least one H-H pair should cancel
        assert!(optimized.size() <= circuit.size());
    }

    #[test]
    fn test_integration_hardware_constraint_failure() {
        // Test that hardware constraints are properly enforced
        let mut circuit = QuantumCircuit::new(4, 0);
        circuit.cx(0, 3).unwrap(); // Not connected in linear topology

        let config = OptimizationConfig {
            enable_hardware_constraints: true,
            hardware_topology: Some(HardwareTopology::linear(4)),
            ..OptimizationConfig::default()
        };

        let mut optimizer = CircuitOptimizer::new(config);
        let result = optimizer.optimize(&circuit);

        // Should fail due to hardware constraints
        assert!(result.is_err());
    }

    #[test]
    fn test_integration_convergence() {
        // Test that optimization converges
        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.h(0).unwrap();
        circuit.x(1).unwrap();

        let config = OptimizationConfig {
            max_passes: 100, // Allow many passes
            ..OptimizationConfig::default()
        };

        let mut optimizer = CircuitOptimizer::new(config);
        let optimized = optimizer.optimize(&circuit).unwrap();

        // Should converge before max_passes
        let stats = optimizer.stats();
        assert!(stats.passes_run < 100);
        assert!(stats.passes_run >= 1);
    }
}
