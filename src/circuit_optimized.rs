//! Optimized quantum circuit data structures
//!
//! This module provides performance-optimized circuit data structures
//! with better memory layout and cache efficiency.

use crate::circuit::{ClassicalBit, Qubit};
use crate::error::{MyQuatError, Result};
use crate::gates::{Gate, StandardGate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Compact instruction representation for better cache performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactInstruction {
    /// Gate type as u8 for compact storage
    gate_type: u8,
    /// Parameters stored as a compact array
    parameters: Vec<f64>, // Pre-evaluated parameters for performance
    /// Qubit indices as u16 (supports up to 65535 qubits)
    qubits: Vec<u16>,
    /// Classical bit indices
    clbits: Vec<u16>,
    /// Flags for instruction properties
    flags: u8, // bit 0: is_measurement, bit 1: is_parametric, etc.
}

impl CompactInstruction {
    /// Create a new compact instruction
    pub fn new(gate: &Gate, qubits: &[Qubit], clbits: &[ClassicalBit]) -> Result<Self> {
        let gate_type = Self::gate_type_to_u8(gate.gate_type);

        // Pre-evaluate parameters if possible
        let parameters: Vec<f64> = gate
            .parameters
            .iter()
            .map(|p| p.evaluate(&HashMap::new()).unwrap_or(0.0))
            .collect();

        let qubits: Vec<u16> = qubits.iter().map(|q| q.index() as u16).collect();

        let clbits: Vec<u16> = clbits.iter().map(|c| c.index() as u16).collect();

        let mut flags = 0u8;
        if !clbits.is_empty() {
            flags |= 0b00000001; // is_measurement
        }
        if gate.is_parametric() {
            flags |= 0b00000010; // is_parametric
        }

        Ok(CompactInstruction {
            gate_type,
            parameters,
            qubits,
            clbits,
            flags,
        })
    }

    /// Convert gate type to compact u8 representation
    fn gate_type_to_u8(gate_type: StandardGate) -> u8 {
        match gate_type {
            StandardGate::I => 0,
            StandardGate::X => 1,
            StandardGate::Y => 2,
            StandardGate::Z => 3,
            StandardGate::H => 4,
            StandardGate::S => 5,
            StandardGate::Sdg => 6,
            StandardGate::T => 7,
            StandardGate::Tdg => 8,
            StandardGate::Rx => 9,
            StandardGate::Ry => 10,
            StandardGate::Rz => 11,
            StandardGate::P => 12,
            StandardGate::U => 13,
            StandardGate::U1 => 14,
            StandardGate::U2 => 15,
            StandardGate::U3 => 16,
            StandardGate::CX => 17,
            StandardGate::CY => 19,
            StandardGate::CZ => 20,
            StandardGate::CH => 21,
            StandardGate::CRx => 22,
            StandardGate::CRy => 23,
            StandardGate::CRz => 24,
            StandardGate::CP => 25,
            StandardGate::Swap => 26,
            StandardGate::ISwap => 27,
            StandardGate::CCX => 28,
            StandardGate::CSwap => 29,
            StandardGate::MCX => 32,
            StandardGate::MCY => 33,
            StandardGate::MCZ => 34,
        }
    }

    /// Convert u8 back to gate type
    fn u8_to_gate_type(value: u8) -> StandardGate {
        match value {
            0 => StandardGate::I,
            1 => StandardGate::X,
            2 => StandardGate::Y,
            3 => StandardGate::Z,
            4 => StandardGate::H,
            5 => StandardGate::S,
            6 => StandardGate::Sdg,
            7 => StandardGate::T,
            8 => StandardGate::Tdg,
            9 => StandardGate::Rx,
            10 => StandardGate::Ry,
            11 => StandardGate::Rz,
            12 => StandardGate::P,
            13 => StandardGate::U,
            14 => StandardGate::U1,
            15 => StandardGate::U2,
            16 => StandardGate::U3,
            17 => StandardGate::CX,
            19 => StandardGate::CY,
            20 => StandardGate::CZ,
            21 => StandardGate::CH,
            22 => StandardGate::CRx,
            23 => StandardGate::CRy,
            24 => StandardGate::CRz,
            25 => StandardGate::CP,
            26 => StandardGate::Swap,
            27 => StandardGate::ISwap,
            28 => StandardGate::CCX,
            29 => StandardGate::CSwap,
            32 => StandardGate::MCX,
            33 => StandardGate::MCY,
            34 => StandardGate::MCZ,
            _ => StandardGate::I, // Default fallback
        }
    }

    /// Get the gate type
    pub fn gate_type(&self) -> StandardGate {
        Self::u8_to_gate_type(self.gate_type)
    }

    /// Get parameters
    pub fn parameters(&self) -> &[f64] {
        &self.parameters
    }

    /// Get qubit indices
    pub fn qubits(&self) -> &[u16] {
        &self.qubits
    }

    /// Get classical bit indices
    pub fn clbits(&self) -> &[u16] {
        &self.clbits
    }

    /// Check if this is a measurement instruction
    pub fn is_measurement(&self) -> bool {
        (self.flags & 0b00000001) != 0
    }

    /// Check if this instruction is parametric
    pub fn is_parametric(&self) -> bool {
        (self.flags & 0b00000010) != 0
    }

    /// Get the number of qubits this instruction acts on
    pub fn num_qubits(&self) -> usize {
        self.qubits.len()
    }
}

/// Optimized circuit data structure with better memory layout
#[derive(Debug, Clone)]
pub struct OptimizedCircuitData {
    /// Circuit metadata
    num_qubits: u16,
    num_clbits: u16,
    global_phase: f64, // Pre-evaluated for performance

    /// Instruction storage optimized for cache performance
    /// Instructions are stored in a flat array for better memory locality
    instructions: Vec<CompactInstruction>,

    /// Instruction dependency graph for parallel execution
    /// Each element contains indices of instructions that this instruction depends on
    dependencies: Vec<Vec<usize>>,

    /// Qubit usage tracking for optimization
    /// Tracks which instructions use each qubit
    qubit_usage: Vec<Vec<usize>>,

    /// Circuit depth cache
    depth_cache: Arc<RwLock<Option<usize>>>,

    /// Metadata stored separately to avoid cache pollution
    metadata: HashMap<String, String>,
}

impl OptimizedCircuitData {
    /// Create a new optimized circuit data
    pub fn new(num_qubits: usize, num_clbits: usize) -> Result<Self> {
        if num_qubits > u16::MAX as usize || num_clbits > u16::MAX as usize {
            return Err(MyQuatError::circuit_error(
                "Circuit size exceeds maximum supported size (65535 qubits/clbits)",
            ));
        }

        Ok(OptimizedCircuitData {
            num_qubits: num_qubits as u16,
            num_clbits: num_clbits as u16,
            global_phase: 0.0,
            instructions: Vec::new(),
            dependencies: Vec::new(),
            qubit_usage: vec![Vec::new(); num_qubits],
            depth_cache: Arc::new(RwLock::new(Some(0))),
            metadata: HashMap::new(),
        })
    }

    /// Get the number of qubits
    pub fn num_qubits(&self) -> usize {
        self.num_qubits as usize
    }

    /// Get the number of classical bits
    pub fn num_clbits(&self) -> usize {
        self.num_clbits as usize
    }

    /// Get the number of instructions
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Check if the circuit is empty
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    /// Add an instruction to the circuit
    pub fn add_instruction(
        &mut self,
        gate: &Gate,
        qubits: &[Qubit],
        clbits: &[ClassicalBit],
    ) -> Result<()> {
        // Validate indices
        for qubit in qubits {
            if qubit.index() >= self.num_qubits() {
                return Err(MyQuatError::InvalidQubitIndex {
                    index: qubit.index(),
                    num_qubits: self.num_qubits(),
                });
            }
        }

        for clbit in clbits {
            if clbit.index() >= self.num_clbits() {
                return Err(MyQuatError::InvalidClbitIndex {
                    index: clbit.index(),
                    num_clbits: self.num_clbits(),
                });
            }
        }

        // Create compact instruction
        let compact_inst = CompactInstruction::new(gate, qubits, clbits)?;

        // Calculate dependencies
        let inst_index = self.instructions.len();
        let mut deps = Vec::new();

        // Find instructions that this instruction depends on
        for qubit_idx in compact_inst.qubits() {
            if let Some(&last_inst) = self.qubit_usage[*qubit_idx as usize].last() {
                if !deps.contains(&last_inst) {
                    deps.push(last_inst);
                }
            }
        }

        // Update qubit usage tracking
        for qubit_idx in compact_inst.qubits() {
            self.qubit_usage[*qubit_idx as usize].push(inst_index);
        }

        // Add instruction and dependencies
        self.instructions.push(compact_inst);
        self.dependencies.push(deps);

        // Invalidate depth cache
        if let Ok(mut cache) = self.depth_cache.write() {
            *cache = None;
        }

        Ok(())
    }

    /// Get all instructions
    pub fn instructions(&self) -> &[CompactInstruction] {
        &self.instructions
    }

    /// Get instruction dependencies
    pub fn dependencies(&self) -> &[Vec<usize>] {
        &self.dependencies
    }

    /// Get the circuit depth (cached for performance)
    pub fn depth(&self) -> usize {
        // Try to read from cache first
        if let Ok(cache) = self.depth_cache.read() {
            if let Some(depth) = *cache {
                return depth;
            }
        }

        // Calculate depth using dependency graph
        let mut depths = vec![0; self.instructions.len()];

        for (i, deps) in self.dependencies.iter().enumerate() {
            if deps.is_empty() {
                depths[i] = 1;
            } else {
                let max_dep_depth = deps
                    .iter()
                    .map(|&dep_idx| depths[dep_idx])
                    .max()
                    .unwrap_or(0);
                depths[i] = max_dep_depth + 1;
            }
        }

        let depth = depths.into_iter().max().unwrap_or(0);

        // Cache the result
        if let Ok(mut cache) = self.depth_cache.write() {
            *cache = Some(depth);
        }

        depth
    }

    /// Get instructions that can be executed in parallel at a given depth level
    pub fn parallel_instructions_at_depth(&self, depth_level: usize) -> Vec<usize> {
        let mut result = Vec::new();
        let mut inst_depths = vec![0; self.instructions.len()];

        // Calculate instruction depths
        for (i, deps) in self.dependencies.iter().enumerate() {
            if deps.is_empty() {
                inst_depths[i] = 1;
            } else {
                let max_dep_depth = deps
                    .iter()
                    .map(|&dep_idx| inst_depths[dep_idx])
                    .max()
                    .unwrap_or(0);
                inst_depths[i] = max_dep_depth + 1;
            }
        }

        // Find instructions at the specified depth
        for (i, &depth) in inst_depths.iter().enumerate() {
            if depth == depth_level {
                result.push(i);
            }
        }

        result
    }

    /// Get global phase
    pub fn global_phase(&self) -> f64 {
        self.global_phase
    }

    /// Set global phase
    pub fn set_global_phase(&mut self, phase: f64) {
        self.global_phase = phase;
    }

    /// Get metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Set metadata
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Count gates of a specific type (optimized)
    pub fn count_gates(&self, gate_type: StandardGate) -> usize {
        self.instructions
            .iter()
            .filter(|inst| inst.gate_type() == gate_type)
            .count()
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> CircuitMemoryStats {
        let instruction_size = std::mem::size_of::<CompactInstruction>() * self.instructions.len();
        let dependency_size = self
            .dependencies
            .iter()
            .map(|deps| std::mem::size_of::<usize>() * deps.len())
            .sum::<usize>();
        let qubit_usage_size = self
            .qubit_usage
            .iter()
            .map(|usage| std::mem::size_of::<usize>() * usage.len())
            .sum::<usize>();
        let metadata_size = self
            .metadata
            .iter()
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>();

        CircuitMemoryStats {
            total_size: instruction_size + dependency_size + qubit_usage_size + metadata_size,
            instruction_size,
            dependency_size,
            qubit_usage_size,
            metadata_size,
        }
    }
}

/// Memory usage statistics for circuit data
#[derive(Debug, Clone)]
pub struct CircuitMemoryStats {
    pub total_size: usize,
    pub instruction_size: usize,
    pub dependency_size: usize,
    pub qubit_usage_size: usize,
    pub metadata_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gates::Gate;

    #[test]
    fn test_optimized_circuit_creation() {
        let circuit = OptimizedCircuitData::new(3, 2).unwrap();
        assert_eq!(circuit.num_qubits(), 3);
        assert_eq!(circuit.num_clbits(), 2);
        assert_eq!(circuit.len(), 0);
        assert!(circuit.is_empty());
        assert_eq!(circuit.depth(), 0);
    }

    #[test]
    fn test_compact_instruction() {
        let gate = Gate::h();
        let qubits = vec![Qubit::new(0)];
        let clbits = vec![];

        let inst = CompactInstruction::new(&gate, &qubits, &clbits).unwrap();
        assert_eq!(inst.gate_type(), StandardGate::H);
        assert_eq!(inst.qubits(), &[0u16]);
        assert!(inst.clbits().is_empty());
        assert!(!inst.is_measurement());
    }

    #[test]
    fn test_instruction_dependencies() {
        let mut circuit = OptimizedCircuitData::new(2, 0).unwrap();

        // Add H gate on qubit 0
        circuit
            .add_instruction(&Gate::h(), &[Qubit::new(0)], &[])
            .unwrap();

        // Add CX gate (should depend on the H gate)
        circuit
            .add_instruction(&Gate::cx(), &[Qubit::new(0), Qubit::new(1)], &[])
            .unwrap();

        assert_eq!(circuit.len(), 2);
        assert_eq!(circuit.depth(), 2);

        // Check dependencies
        let deps = circuit.dependencies();
        assert!(deps[0].is_empty()); // H gate has no dependencies
        assert_eq!(deps[1], vec![0]); // CX gate depends on H gate
    }

    #[test]
    fn test_parallel_instructions() {
        let mut circuit = OptimizedCircuitData::new(3, 0).unwrap();

        // Add parallel H gates
        circuit
            .add_instruction(&Gate::h(), &[Qubit::new(0)], &[])
            .unwrap();
        circuit
            .add_instruction(&Gate::h(), &[Qubit::new(1)], &[])
            .unwrap();
        circuit
            .add_instruction(&Gate::h(), &[Qubit::new(2)], &[])
            .unwrap();

        assert_eq!(circuit.depth(), 1);

        // All instructions should be at depth 1
        let parallel_insts = circuit.parallel_instructions_at_depth(1);
        assert_eq!(parallel_insts.len(), 3);
        assert_eq!(parallel_insts, vec![0, 1, 2]);
    }

    #[test]
    fn test_memory_stats() {
        let mut circuit = OptimizedCircuitData::new(2, 2).unwrap();
        circuit
            .add_instruction(&Gate::h(), &[Qubit::new(0)], &[])
            .unwrap();
        circuit
            .add_instruction(&Gate::cx(), &[Qubit::new(0), Qubit::new(1)], &[])
            .unwrap();

        let stats = circuit.memory_stats();
        assert!(stats.total_size > 0);
        assert!(stats.instruction_size > 0);
    }
}
