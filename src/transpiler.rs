//! Quantum circuit transpilation and optimization
//!
//! This module provides tools for optimizing quantum circuits,
//! including gate decomposition, circuit depth reduction, and hardware mapping.

use crate::circuit::{Instruction, QuantumCircuit, Qubit};
use crate::error::{MyQuatError, Result};
use crate::gates::{Gate, GateOperation, StandardGate};
use crate::parameter::Parameter;
use std::collections::{HashMap, HashSet};

/// A transpiler pass that transforms quantum circuits
pub trait TranspilerPass {
    /// Get the name of this pass
    fn name(&self) -> &str;

    /// Run the pass on a circuit
    fn run(&self, circuit: &mut QuantumCircuit) -> Result<()>;

    /// Check if this pass requires analysis before running
    fn requires_analysis(&self) -> bool {
        false
    }
}

/// Pass manager for running multiple transpiler passes
pub struct PassManager {
    passes: Vec<Box<dyn TranspilerPass>>,
}

impl PassManager {
    /// Create a new pass manager
    pub fn new() -> Self {
        PassManager { passes: Vec::new() }
    }

    /// Add a pass to the manager
    pub fn add_pass(&mut self, pass: Box<dyn TranspilerPass>) {
        self.passes.push(pass);
    }

    /// Run all passes on a circuit
    pub fn run(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        for pass in &self.passes {
            pass.run(circuit)?;
        }
        Ok(())
    }

    /// Create a default optimization pass manager
    pub fn default_optimization() -> Self {
        let mut pm = PassManager::new();
        pm.add_pass(Box::new(RemoveIdentityGates));
        pm.add_pass(Box::new(CombineRotations));
        pm.add_pass(Box::new(CancelInverses));
        pm.add_pass(Box::new(CommuteThroughClifford));
        pm
    }
}

impl Default for PassManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Pass to remove identity gates
#[derive(Debug)]
pub struct RemoveIdentityGates;

impl TranspilerPass for RemoveIdentityGates {
    fn name(&self) -> &str {
        "remove_identity_gates"
    }

    fn run(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        let data = circuit.data_mut();
        data.instructions_mut()
            .retain(|instruction| !matches!(instruction.gate.gate_type, StandardGate::I));
        Ok(())
    }
}

/// Pass to combine consecutive rotation gates on the same qubit
#[derive(Debug)]
pub struct CombineRotations;

impl TranspilerPass for CombineRotations {
    fn name(&self) -> &str {
        "combine_rotations"
    }

    fn run(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        let data = circuit.data_mut();
        let instructions = data.instructions_mut();

        let mut i = 0;
        while i < instructions.len() {
            if i + 1 >= instructions.len() {
                break;
            }

            let current = &instructions[i];
            let next = &instructions[i + 1];

            // Check if we can combine two rotation gates
            if can_combine_rotations(current, next) {
                if let Some(combined) = combine_rotation_gates(current, next)? {
                    instructions[i] = combined;
                    instructions.remove(i + 1);
                    continue; // Don't increment i, check again from the same position
                }
            }

            i += 1;
        }

        Ok(())
    }
}

/// Pass to cancel inverse gate pairs
#[derive(Debug)]
pub struct CancelInverses;

impl TranspilerPass for CancelInverses {
    fn name(&self) -> &str {
        "cancel_inverses"
    }

    fn run(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        let data = circuit.data_mut();
        let instructions = data.instructions_mut();

        let mut i = 0;
        while i < instructions.len() {
            if i + 1 >= instructions.len() {
                break;
            }

            let current = &instructions[i];
            let next = &instructions[i + 1];

            // Check if gates are inverses of each other on the same qubits
            if are_inverse_gates(current, next) {
                instructions.remove(i + 1);
                instructions.remove(i);
                i = i.saturating_sub(1);
                continue;
            }

            i += 1;
        }

        Ok(())
    }
}

/// Pass to commute single-qubit gates through Clifford gates
#[derive(Debug)]
pub struct CommuteThroughClifford;

impl TranspilerPass for CommuteThroughClifford {
    fn name(&self) -> &str {
        "commute_through_clifford"
    }

    fn run(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        let data = circuit.data_mut();
        let instructions = data.instructions_mut();

        let mut i = 0;
        while i < instructions.len() {
            if i + 1 >= instructions.len() {
                break;
            }

            // Look for patterns where we can commute gates
            if can_commute_gates(&instructions[i], &instructions[i + 1]) {
                instructions.swap(i, i + 1);
                i = i.saturating_sub(1);
                continue;
            }

            i += 1;
        }

        Ok(())
    }
}

/// Circuit analysis utilities
pub struct CircuitAnalysis;

impl CircuitAnalysis {
    /// Analyze gate counts in a circuit
    pub fn gate_counts(circuit: &QuantumCircuit) -> HashMap<StandardGate, usize> {
        let mut counts = HashMap::new();

        for instruction in circuit.data().instructions() {
            if !instruction.is_measurement() {
                *counts.entry(instruction.gate.gate_type).or_insert(0) += 1;
            }
        }

        counts
    }

    /// Calculate the total number of two-qubit gates
    pub fn two_qubit_gate_count(circuit: &QuantumCircuit) -> usize {
        circuit
            .data()
            .instructions()
            .iter()
            .filter(|inst| !inst.is_measurement() && inst.gate.gate_type.num_qubits() == 2)
            .count()
    }

    /// Find the critical path (longest path through the circuit)
    pub fn critical_path_length(circuit: &QuantumCircuit) -> usize {
        circuit.depth()
    }

    /// Analyze qubit connectivity requirements
    pub fn connectivity_graph(circuit: &QuantumCircuit) -> HashMap<usize, HashSet<usize>> {
        let mut graph = HashMap::new();

        for instruction in circuit.data().instructions() {
            if instruction.gate.gate_type.num_qubits() == 2 {
                let q1 = instruction.qubits[0].index();
                let q2 = instruction.qubits[1].index();

                graph.entry(q1).or_insert_with(HashSet::new).insert(q2);
                graph.entry(q2).or_insert_with(HashSet::new).insert(q1);
            }
        }

        graph
    }

    /// Count the number of layers in the circuit
    pub fn layer_count(circuit: &QuantumCircuit) -> usize {
        if circuit.is_empty() {
            return 0;
        }

        let mut qubit_layers = vec![0; circuit.num_qubits()];
        let mut max_layer = 0;

        for instruction in circuit.data().instructions() {
            if instruction.is_measurement() {
                continue;
            }

            let current_layer = instruction
                .qubits
                .iter()
                .map(|q| qubit_layers[q.index()])
                .max()
                .unwrap_or(0);

            let next_layer = current_layer + 1;

            for qubit in &instruction.qubits {
                qubit_layers[qubit.index()] = next_layer;
            }

            max_layer = max_layer.max(next_layer);
        }

        max_layer
    }
}

/// Gate decomposition utilities
pub struct GateDecomposer;

impl GateDecomposer {
    /// Decompose a U3 gate into elementary gates
    pub fn decompose_u3(theta: Parameter, phi: Parameter, lambda: Parameter) -> Vec<Gate> {
        // U3(θ,φ,λ) = RZ(φ) RY(θ) RZ(λ)
        vec![Gate::rz(lambda), Gate::ry(theta), Gate::rz(phi)]
    }

    /// Decompose a controlled gate into elementary gates
    pub fn decompose_controlled_gate(
        gate: StandardGate,
        params: Vec<Parameter>,
    ) -> Result<Vec<(Gate, Vec<usize>)>> {
        match gate {
            StandardGate::CX => {
                // CNOT is already elementary
                Ok(vec![(Gate::cx(), vec![0, 1])])
            }
            StandardGate::CZ => {
                // CZ is already elementary
                Ok(vec![(Gate::cz(), vec![0, 1])])
            }
            StandardGate::CRz => {
                if params.len() != 1 {
                    return Err(MyQuatError::transpiler_error(
                        "CRz requires exactly one parameter",
                    ));
                }

                // CRZ(λ) = RZ(λ/2) ⊗ I, CNOT, RZ(-λ/2) ⊗ I, CNOT, RZ(λ/2) ⊗ I
                let half_lambda =
                    Parameter::new_expression(crate::parameter::ParameterExpression::Div(
                        Box::new(crate::parameter::ParameterExpression::Parameter(
                            params[0].clone(),
                        )),
                        Box::new(crate::parameter::ParameterExpression::Parameter(
                            Parameter::new_float(2.0),
                        )),
                    ));
                let neg_half_lambda =
                    Parameter::new_expression(crate::parameter::ParameterExpression::Mul(
                        Box::new(crate::parameter::ParameterExpression::Parameter(
                            Parameter::new_float(-1.0),
                        )),
                        Box::new(crate::parameter::ParameterExpression::Parameter(
                            half_lambda.clone(),
                        )),
                    ));

                Ok(vec![
                    (Gate::rz(half_lambda.clone()), vec![1]),
                    (Gate::cx(), vec![0, 1]),
                    (Gate::rz(neg_half_lambda), vec![1]),
                    (Gate::cx(), vec![0, 1]),
                    (Gate::rz(half_lambda), vec![0]),
                ])
            }
            _ => Err(MyQuatError::transpiler_error(format!(
                "Decomposition not implemented for gate: {:?}",
                gate
            ))),
        }
    }

    /// Decompose a Toffoli gate (CCX) into elementary gates
    pub fn decompose_toffoli() -> Vec<(Gate, Vec<usize>)> {
        vec![
            (Gate::h(), vec![2]),
            (Gate::cx(), vec![1, 2]),
            (Gate::tdg(), vec![2]),
            (Gate::cx(), vec![0, 2]),
            (Gate::t(), vec![2]),
            (Gate::cx(), vec![1, 2]),
            (Gate::tdg(), vec![2]),
            (Gate::cx(), vec![0, 2]),
            (Gate::t(), vec![1]),
            (Gate::t(), vec![2]),
            (Gate::cx(), vec![0, 1]),
            (Gate::h(), vec![2]),
            (Gate::t(), vec![0]),
            (Gate::tdg(), vec![1]),
            (Gate::cx(), vec![0, 1]),
        ]
    }
}

/// Hardware mapping utilities
pub struct HardwareMapper;

impl HardwareMapper {
    /// Simple linear mapping of logical qubits to physical qubits
    pub fn linear_mapping(
        circuit: &QuantumCircuit,
        num_physical_qubits: usize,
    ) -> Result<HashMap<usize, usize>> {
        if circuit.num_qubits() > num_physical_qubits {
            return Err(MyQuatError::transpiler_error(format!(
                "Circuit requires {} qubits but hardware has only {}",
                circuit.num_qubits(),
                num_physical_qubits
            )));
        }

        let mut mapping = HashMap::new();
        for i in 0..circuit.num_qubits() {
            mapping.insert(i, i);
        }

        Ok(mapping)
    }

    /// Apply a qubit mapping to a circuit
    pub fn apply_mapping(
        circuit: &mut QuantumCircuit,
        mapping: &HashMap<usize, usize>,
    ) -> Result<()> {
        let data = circuit.data_mut();

        for instruction in data.instructions_mut() {
            for qubit in &mut instruction.qubits {
                if let Some(&physical_qubit) = mapping.get(&qubit.index()) {
                    *qubit = Qubit::new(physical_qubit);
                } else {
                    return Err(MyQuatError::transpiler_error(format!(
                        "No mapping found for logical qubit {}",
                        qubit.index()
                    )));
                }
            }
        }

        Ok(())
    }
}

// Helper functions for transpiler passes

fn can_combine_rotations(inst1: &Instruction, inst2: &Instruction) -> bool {
    // Check if both are single-qubit rotation gates on the same qubit
    if inst1.qubits.len() != 1 || inst2.qubits.len() != 1 {
        return false;
    }

    if inst1.qubits[0] != inst2.qubits[0] {
        return false;
    }

    // Check if both are the same type of rotation
    matches!(
        (inst1.gate.gate_type, inst2.gate.gate_type),
        (StandardGate::Rx, StandardGate::Rx)
            | (StandardGate::Ry, StandardGate::Ry)
            | (StandardGate::Rz, StandardGate::Rz)
            | (StandardGate::P, StandardGate::P)
    )
}

fn combine_rotation_gates(inst1: &Instruction, inst2: &Instruction) -> Result<Option<Instruction>> {
    if !can_combine_rotations(inst1, inst2) {
        return Ok(None);
    }

    if inst1.gate.parameters.len() != 1 || inst2.gate.parameters.len() != 1 {
        return Ok(None);
    }

    // Combine parameters by addition
    let combined_param = Parameter::new_expression(crate::parameter::ParameterExpression::Add(
        Box::new(crate::parameter::ParameterExpression::Parameter(
            inst1.gate.parameters[0].clone(),
        )),
        Box::new(crate::parameter::ParameterExpression::Parameter(
            inst2.gate.parameters[0].clone(),
        )),
    ));

    let combined_gate = Gate::new(inst1.gate.gate_type, vec![combined_param])?;
    let combined_instruction = Instruction::new(combined_gate, inst1.qubits.clone())?;

    Ok(Some(combined_instruction))
}

fn are_inverse_gates(inst1: &Instruction, inst2: &Instruction) -> bool {
    // Check if gates act on the same qubits
    if inst1.qubits != inst2.qubits {
        return false;
    }

    // Check for specific inverse pairs
    match (inst1.gate.gate_type, inst2.gate.gate_type) {
        (StandardGate::S, StandardGate::Sdg)
        | (StandardGate::Sdg, StandardGate::S)
        | (StandardGate::T, StandardGate::Tdg)
        | (StandardGate::Tdg, StandardGate::T) => true,

        // Self-inverse gates
        (a, b) if a == b && a.is_self_inverse() => true,

        // Parametric gates with opposite parameters (simplified check)
        (StandardGate::Rx, StandardGate::Rx)
        | (StandardGate::Ry, StandardGate::Ry)
        | (StandardGate::Rz, StandardGate::Rz)
        | (StandardGate::P, StandardGate::P) => {
            // For now, just check if both have one parameter
            // A full implementation would check if parameters are negatives
            inst1.gate.parameters.len() == 1 && inst2.gate.parameters.len() == 1
        }

        _ => false,
    }
}

fn can_commute_gates(inst1: &Instruction, inst2: &Instruction) -> bool {
    // Gates can commute if they act on disjoint sets of qubits
    let qubits1: HashSet<_> = inst1.qubits.iter().collect();
    let qubits2: HashSet<_> = inst2.qubits.iter().collect();

    qubits1.is_disjoint(&qubits2)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use std::f64::consts::PI;

    #[test]
    fn test_remove_identity_gates() {
        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.i(0).unwrap();
        circuit.x(0).unwrap();
        circuit.i(1).unwrap();
        circuit.h(1).unwrap();

        assert_eq!(circuit.size(), 4);

        let pass = RemoveIdentityGates;
        pass.run(&mut circuit).unwrap();

        assert_eq!(circuit.size(), 2);

        let instructions = circuit.data().instructions();
        assert_eq!(instructions[0].gate.gate_type, StandardGate::X);
        assert_eq!(instructions[1].gate.gate_type, StandardGate::H);
    }

    #[test]
    fn test_cancel_inverses() {
        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.s(0).unwrap();
        circuit.sdg(0).unwrap();
        circuit.x(1).unwrap();
        circuit.x(1).unwrap();

        assert_eq!(circuit.size(), 4);

        let pass = CancelInverses;
        pass.run(&mut circuit).unwrap();

        assert_eq!(circuit.size(), 0);
    }

    #[test]
    fn test_gate_counts() {
        let mut circuit = QuantumCircuit::new(3, 0);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.cx(1, 2).unwrap();
        circuit.h(2).unwrap();

        let counts = CircuitAnalysis::gate_counts(&circuit);
        assert_eq!(counts.get(&StandardGate::H), Some(&2));
        assert_eq!(counts.get(&StandardGate::CX), Some(&2));
    }

    #[test]
    fn test_two_qubit_gate_count() {
        let mut circuit = QuantumCircuit::new(3, 0);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.cz(1, 2).unwrap();
        circuit.swap(0, 2).unwrap();

        let count = CircuitAnalysis::two_qubit_gate_count(&circuit);
        assert_eq!(count, 3); // CX, CZ, SWAP
    }

    #[test]
    fn test_pass_manager() {
        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.i(0).unwrap();
        circuit.s(0).unwrap();
        circuit.sdg(0).unwrap();
        circuit.x(1).unwrap();

        assert_eq!(circuit.size(), 4);

        let pm = PassManager::default_optimization();
        pm.run(&mut circuit).unwrap();

        // Should remove identity and cancel S/Sdg pair
        assert_eq!(circuit.size(), 1);
        assert_eq!(
            circuit.data().instructions()[0].gate.gate_type,
            StandardGate::X
        );
    }

    #[test]
    fn test_toffoli_decomposition() {
        let decomposition = GateDecomposer::decompose_toffoli();
        assert_eq!(decomposition.len(), 15);

        // Check that the decomposition uses only single and two-qubit gates
        for (gate, qubits) in decomposition {
            assert!(qubits.len() <= 2);
            assert!(gate.gate_type.num_qubits() <= 2);
        }
    }

    #[test]
    fn test_linear_mapping() {
        let circuit = QuantumCircuit::new(3, 0);
        let mapping = HardwareMapper::linear_mapping(&circuit, 5).unwrap();

        assert_eq!(mapping.len(), 3);
        assert_eq!(mapping[&0], 0);
        assert_eq!(mapping[&1], 1);
        assert_eq!(mapping[&2], 2);
    }
}
