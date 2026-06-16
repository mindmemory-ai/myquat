//! Device topology and connectivity constraints
//!
//! This module provides comprehensive support for modeling quantum device topologies,
//! including connectivity graphs, routing algorithms, and constraint validation.

use crate::{QuantumCircuit, Result, StandardGate};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Quantum device topology representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceTopology {
    /// Number of physical qubits
    pub num_qubits: usize,
    /// Connectivity graph: qubit -> set of connected qubits
    pub connectivity: HashMap<usize, HashSet<usize>>,
    /// Gate execution times for different gate types (nanoseconds)
    pub gate_times: HashMap<String, f64>,
    /// Two-qubit gate fidelities for each edge
    pub edge_fidelities: HashMap<(usize, usize), f64>,
    /// Single-qubit gate fidelities
    pub qubit_fidelities: HashMap<usize, f64>,
    /// Readout fidelities
    pub readout_fidelities: HashMap<usize, f64>,
    /// Device name/identifier
    pub name: String,
}

impl DeviceTopology {
    /// Create a new device topology
    pub fn new(name: String, num_qubits: usize) -> Self {
        Self {
            num_qubits,
            connectivity: HashMap::new(),
            gate_times: HashMap::new(),
            edge_fidelities: HashMap::new(),
            qubit_fidelities: HashMap::new(),
            readout_fidelities: HashMap::new(),
            name,
        }
    }

    /// Create a linear topology (qubits connected in a line)
    pub fn linear(num_qubits: usize) -> Self {
        let mut topology = Self::new("Linear".to_string(), num_qubits);

        // Connect adjacent qubits
        for i in 0..num_qubits {
            let mut neighbors = HashSet::new();
            if i > 0 {
                neighbors.insert(i - 1);
            }
            if i < num_qubits - 1 {
                neighbors.insert(i + 1);
            }
            topology.connectivity.insert(i, neighbors);
        }

        // Set default fidelities and times
        topology.set_default_parameters();
        topology
    }

    /// Create a grid topology (2D lattice)
    pub fn grid(rows: usize, cols: usize) -> Self {
        let num_qubits = rows * cols;
        let mut topology = Self::new("Grid".to_string(), num_qubits);

        // Connect qubits in 2D grid
        for row in 0..rows {
            for col in 0..cols {
                let qubit = row * cols + col;
                let mut neighbors = HashSet::new();

                // Connect to adjacent qubits (up, down, left, right)
                if row > 0 {
                    neighbors.insert((row - 1) * cols + col); // up
                }
                if row < rows - 1 {
                    neighbors.insert((row + 1) * cols + col); // down
                }
                if col > 0 {
                    neighbors.insert(row * cols + (col - 1)); // left
                }
                if col < cols - 1 {
                    neighbors.insert(row * cols + (col + 1)); // right
                }

                topology.connectivity.insert(qubit, neighbors);
            }
        }

        topology.set_default_parameters();
        topology
    }

    /// Create a heavy-hex topology (IBM-style)
    pub fn heavy_hex(distance: usize) -> Self {
        // Simplified heavy-hex for demonstration
        // Real implementation would follow IBM's heavy-hex pattern
        let num_qubits = match distance {
            1 => 5,                   // 5-qubit heavy-hex
            2 => 16,                  // 16-qubit heavy-hex
            3 => 27,                  // 27-qubit heavy-hex
            _ => distance * distance, // Approximate
        };

        let mut topology = Self::new("Heavy-Hex".to_string(), num_qubits);

        // For simplicity, create a connected graph
        // Real heavy-hex would have specific connectivity pattern
        for i in 0..num_qubits {
            let mut neighbors = HashSet::new();
            // Connect to a few neighbors (simplified)
            for j in 0..num_qubits {
                if i != j && ((i as i32 - j as i32).abs() <= 2) {
                    neighbors.insert(j);
                }
            }
            topology.connectivity.insert(i, neighbors);
        }

        topology.set_default_parameters();
        topology
    }

    /// Create a fully connected topology (all-to-all)
    pub fn fully_connected(num_qubits: usize) -> Self {
        let mut topology = Self::new("Fully-Connected".to_string(), num_qubits);

        for i in 0..num_qubits {
            let neighbors: HashSet<usize> = (0..num_qubits).filter(|&j| j != i).collect();
            topology.connectivity.insert(i, neighbors);
        }

        topology.set_default_parameters();
        topology
    }

    /// Add a connection between two qubits
    pub fn add_edge(&mut self, qubit1: usize, qubit2: usize) {
        if qubit1 >= self.num_qubits || qubit2 >= self.num_qubits {
            return;
        }

        self.connectivity.entry(qubit1).or_default().insert(qubit2);
        self.connectivity.entry(qubit2).or_default().insert(qubit1);
    }

    /// Remove a connection between two qubits
    pub fn remove_edge(&mut self, qubit1: usize, qubit2: usize) {
        if let Some(neighbors) = self.connectivity.get_mut(&qubit1) {
            neighbors.remove(&qubit2);
        }
        if let Some(neighbors) = self.connectivity.get_mut(&qubit2) {
            neighbors.remove(&qubit1);
        }
    }

    /// Check if two qubits are connected
    pub fn are_connected(&self, qubit1: usize, qubit2: usize) -> bool {
        self.connectivity
            .get(&qubit1)
            .map(|neighbors| neighbors.contains(&qubit2))
            .unwrap_or(false)
    }

    /// Get all neighbors of a qubit
    pub fn neighbors(&self, qubit: usize) -> HashSet<usize> {
        self.connectivity.get(&qubit).cloned().unwrap_or_default()
    }

    /// Calculate shortest path between two qubits using BFS
    pub fn shortest_path(&self, start: usize, end: usize) -> Option<Vec<usize>> {
        if start == end {
            return Some(vec![start]);
        }

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = HashMap::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(current) = queue.pop_front() {
            if let Some(neighbors) = self.connectivity.get(&current) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        parent.insert(neighbor, current);
                        queue.push_back(neighbor);

                        if neighbor == end {
                            // Reconstruct path
                            let mut path = vec![end];
                            let mut current = end;
                            while let Some(&p) = parent.get(&current) {
                                path.push(p);
                                current = p;
                            }
                            path.reverse();
                            return Some(path);
                        }
                    }
                }
            }
        }

        None // No path found
    }

    /// Calculate distance between two qubits
    pub fn distance(&self, qubit1: usize, qubit2: usize) -> Option<usize> {
        self.shortest_path(qubit1, qubit2)
            .map(|path| path.len() - 1)
    }

    /// Get all edges in the topology
    pub fn edges(&self) -> Vec<(usize, usize)> {
        let mut edges = Vec::new();
        for (&qubit, neighbors) in &self.connectivity {
            for &neighbor in neighbors {
                if qubit < neighbor {
                    // Avoid duplicates
                    edges.push((qubit, neighbor));
                }
            }
        }
        edges
    }

    /// Set default parameters for the topology
    fn set_default_parameters(&mut self) {
        // Default gate times (nanoseconds)
        self.gate_times.insert("X".to_string(), 20.0);
        self.gate_times.insert("Y".to_string(), 20.0);
        self.gate_times.insert("Z".to_string(), 0.0);
        self.gate_times.insert("H".to_string(), 20.0);
        self.gate_times.insert("S".to_string(), 0.0);
        self.gate_times.insert("T".to_string(), 0.0);
        self.gate_times.insert("CNOT".to_string(), 200.0);
        self.gate_times.insert("CZ".to_string(), 200.0);

        // Default single-qubit fidelities
        for i in 0..self.num_qubits {
            self.qubit_fidelities.insert(i, 0.999);
            self.readout_fidelities.insert(i, 0.98);
        }

        // Default two-qubit fidelities
        for (q1, q2) in self.edges() {
            self.edge_fidelities.insert((q1, q2), 0.99);
            self.edge_fidelities.insert((q2, q1), 0.99);
        }
    }

    /// Validate a circuit against this topology
    pub fn validate_circuit(&self, circuit: &QuantumCircuit) -> TopologyValidationResult {
        let mut result = TopologyValidationResult::new();

        for instruction in circuit.data().instructions() {
            match &instruction.gate.gate_type {
                StandardGate::CX
                | StandardGate::CY
                | StandardGate::CZ
                | StandardGate::CH
                | StandardGate::CRx
                | StandardGate::CRy
                | StandardGate::CRz
                | StandardGate::Swap
                | StandardGate::ISwap => {
                    if instruction.qubits.len() >= 2 {
                        let q1 = instruction.qubits[0].index();
                        let q2 = instruction.qubits[1].index();

                        if !self.are_connected(q1, q2) {
                            result.violations.push(TopologyViolation {
                                violation_type: TopologyViolationType::ConnectivityViolation,
                                qubits: vec![q1, q2],
                                gate: format!("{:?}", instruction.gate.gate_type),
                                message: format!(
                                    "Qubits {} and {} are not connected in topology",
                                    q1, q2
                                ),
                            });
                        }
                    }
                }
                _ => {
                    // Single-qubit gates - check if qubit exists
                    for qubit in &instruction.qubits {
                        let q = qubit.index();
                        if q >= self.num_qubits {
                            result.violations.push(TopologyViolation {
                                violation_type: TopologyViolationType::InvalidQubit,
                                qubits: vec![q],
                                gate: format!("{:?}", instruction.gate.gate_type),
                                message: format!("Qubit {} does not exist in topology", q),
                            });
                        }
                    }
                }
            }
        }

        result.is_valid = result.violations.is_empty();
        result
    }
}

/// Result of topology validation
#[derive(Debug, Clone)]
pub struct TopologyValidationResult {
    /// Whether the circuit is valid for this topology
    pub is_valid: bool,
    /// List of violations found
    pub violations: Vec<TopologyViolation>,
}

impl TopologyValidationResult {
    fn new() -> Self {
        Self {
            is_valid: true,
            violations: Vec::new(),
        }
    }
}

/// A topology constraint violation
#[derive(Debug, Clone)]
pub struct TopologyViolation {
    /// Type of violation
    pub violation_type: TopologyViolationType,
    /// Qubits involved in the violation
    pub qubits: Vec<usize>,
    /// Gate that caused the violation
    pub gate: String,
    /// Human-readable message
    pub message: String,
}

/// Types of topology violations
#[derive(Debug, Clone, PartialEq)]
pub enum TopologyViolationType {
    /// Two qubits are not connected but a two-qubit gate is applied
    ConnectivityViolation,
    /// Qubit index is out of range
    InvalidQubit,
    /// Gate is not supported on this topology
    UnsupportedGate,
}

/// Circuit routing and mapping utilities
pub struct CircuitRouter {
    topology: DeviceTopology,
}

impl CircuitRouter {
    /// Create a new circuit router
    pub fn new(topology: DeviceTopology) -> Self {
        Self { topology }
    }

    /// Route a circuit to fit the topology constraints
    pub fn route_circuit(&self, circuit: &QuantumCircuit) -> Result<QuantumCircuit> {
        // Simple routing algorithm - insert SWAP gates as needed
        let mut routed_circuit =
            QuantumCircuit::new(self.topology.num_qubits, circuit.num_clbits());

        // Current logical-to-physical qubit mapping
        let mut mapping: HashMap<usize, usize> =
            (0..circuit.num_qubits()).map(|i| (i, i)).collect();

        for instruction in circuit.data().instructions() {
            match &instruction.gate.gate_type {
                StandardGate::CX
                | StandardGate::CY
                | StandardGate::CZ
                | StandardGate::CH
                | StandardGate::CRx
                | StandardGate::CRy
                | StandardGate::CRz
                | StandardGate::Swap
                | StandardGate::ISwap => {
                    if instruction.qubits.len() >= 2 {
                        let logical_q1 = instruction.qubits[0].index();
                        let logical_q2 = instruction.qubits[1].index();
                        let physical_q1 = mapping[&logical_q1];
                        let physical_q2 = mapping[&logical_q2];

                        if !self.topology.are_connected(physical_q1, physical_q2) {
                            // Need to route - find path and insert SWAPs
                            if let Some(path) =
                                self.topology.shortest_path(physical_q1, physical_q2)
                            {
                                // Insert SWAP gates to move qubits closer
                                for i in 0..path.len() - 2 {
                                    routed_circuit.swap(path[i], path[i + 1])?;
                                    // Update mapping
                                    self.update_mapping_after_swap(
                                        &mut mapping,
                                        path[i],
                                        path[i + 1],
                                    );
                                }
                            }
                        }

                        // Apply the original gate with updated mapping
                        match &instruction.gate.gate_type {
                            StandardGate::CX => {
                                routed_circuit.cx(mapping[&logical_q1], mapping[&logical_q2])?
                            }
                            StandardGate::CZ => {
                                routed_circuit.cz(mapping[&logical_q1], mapping[&logical_q2])?
                            }
                            _ => {
                                // For other gates, add as identity for now
                                routed_circuit.i(mapping[&logical_q1])?;
                            }
                        }
                    }
                }
                _ => {
                    // Single-qubit gates - apply directly
                    if !instruction.qubits.is_empty() {
                        let logical_q = instruction.qubits[0].index();
                        let physical_q = mapping[&logical_q];

                        match &instruction.gate.gate_type {
                            StandardGate::X => routed_circuit.x(physical_q)?,
                            StandardGate::Y => routed_circuit.y(physical_q)?,
                            StandardGate::Z => routed_circuit.z(physical_q)?,
                            StandardGate::H => routed_circuit.h(physical_q)?,
                            StandardGate::S => routed_circuit.s(physical_q)?,
                            StandardGate::T => routed_circuit.t(physical_q)?,
                            _ => routed_circuit.i(physical_q)?,
                        }
                    }
                }
            }
        }

        Ok(routed_circuit)
    }

    /// Update qubit mapping after a SWAP gate
    fn update_mapping_after_swap(&self, mapping: &mut HashMap<usize, usize>, q1: usize, q2: usize) {
        // Find logical qubits mapped to q1 and q2 and swap them
        let mut logical_q1 = None;
        let mut logical_q2 = None;

        for (&logical, &physical) in mapping.iter() {
            if physical == q1 {
                logical_q1 = Some(logical);
            } else if physical == q2 {
                logical_q2 = Some(logical);
            }
        }

        if let (Some(lq1), Some(lq2)) = (logical_q1, logical_q2) {
            mapping.insert(lq1, q2);
            mapping.insert(lq2, q1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_topology() {
        let topology = DeviceTopology::linear(5);
        assert_eq!(topology.num_qubits, 5);
        assert!(topology.are_connected(0, 1));
        assert!(topology.are_connected(1, 2));
        assert!(!topology.are_connected(0, 2));
        assert!(!topology.are_connected(0, 4));
    }

    #[test]
    fn test_grid_topology() {
        let topology = DeviceTopology::grid(2, 3); // 2x3 grid
        assert_eq!(topology.num_qubits, 6);

        // Check horizontal connections
        assert!(topology.are_connected(0, 1));
        assert!(topology.are_connected(1, 2));
        assert!(topology.are_connected(3, 4));
        assert!(topology.are_connected(4, 5));

        // Check vertical connections
        assert!(topology.are_connected(0, 3));
        assert!(topology.are_connected(1, 4));
        assert!(topology.are_connected(2, 5));

        // Check non-connections
        assert!(!topology.are_connected(0, 2));
        assert!(!topology.are_connected(0, 5));
    }

    #[test]
    fn test_shortest_path() {
        let topology = DeviceTopology::linear(5);

        let path = topology.shortest_path(0, 4).unwrap();
        assert_eq!(path, vec![0, 1, 2, 3, 4]);

        let path = topology.shortest_path(1, 3).unwrap();
        assert_eq!(path, vec![1, 2, 3]);

        let path = topology.shortest_path(2, 2).unwrap();
        assert_eq!(path, vec![2]);
    }

    #[test]
    fn test_distance_calculation() {
        let topology = DeviceTopology::linear(5);

        assert_eq!(topology.distance(0, 4), Some(4));
        assert_eq!(topology.distance(1, 3), Some(2));
        assert_eq!(topology.distance(2, 2), Some(0));
    }

    #[test]
    fn test_circuit_validation() {
        let topology = DeviceTopology::linear(3);
        let mut circuit = QuantumCircuit::new(3, 0);

        // Valid circuit
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap(); // Adjacent qubits
        circuit.cx(1, 2).unwrap(); // Adjacent qubits

        let result = topology.validate_circuit(&circuit);
        assert!(result.is_valid);
        assert!(result.violations.is_empty());

        // Invalid circuit
        let mut invalid_circuit = QuantumCircuit::new(3, 0);
        invalid_circuit.cx(0, 2).unwrap(); // Non-adjacent qubits

        let result = topology.validate_circuit(&invalid_circuit);
        assert!(!result.is_valid);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(
            result.violations[0].violation_type,
            TopologyViolationType::ConnectivityViolation
        );
    }

    #[test]
    fn test_fully_connected_topology() {
        let topology = DeviceTopology::fully_connected(4);

        // All qubits should be connected to all others
        for i in 0..4 {
            for j in 0..4 {
                if i != j {
                    assert!(topology.are_connected(i, j));
                }
            }
        }
    }

    #[test]
    fn test_circuit_router() {
        let topology = DeviceTopology::linear(3);
        let router = CircuitRouter::new(topology);

        let mut circuit = QuantumCircuit::new(3, 0);
        circuit.h(0).unwrap();
        circuit.cx(0, 2).unwrap(); // This will need routing

        let routed = router.route_circuit(&circuit).unwrap();
        // The routed circuit should have additional SWAP gates
        assert!(routed.size() > circuit.size());
    }
}
