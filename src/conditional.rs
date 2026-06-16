//! Conditional operations and classical control
//!
//! This module provides support for conditional quantum operations
//! based on classical measurement results and register states.

use crate::error::{MyQuatError, Result};
use crate::{GateOperation, Parameter, QuantumCircuit, StandardGate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Classical condition for conditional operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ClassicalCondition {
    /// Condition based on a single classical bit
    BitEquals { bit_index: usize, value: u8 },
    /// Condition based on classical register value
    RegisterEquals { register_name: String, value: u64 },
    /// Condition based on register comparison
    RegisterGreater { register_name: String, value: u64 },
    /// Condition based on register less than
    RegisterLess { register_name: String, value: u64 },
    /// Logical AND of two conditions
    And(Box<ClassicalCondition>, Box<ClassicalCondition>),
    /// Logical OR of two conditions
    Or(Box<ClassicalCondition>, Box<ClassicalCondition>),
    /// Logical NOT of a condition
    Not(Box<ClassicalCondition>),
}

impl ClassicalCondition {
    /// Create a condition for a single bit
    pub fn bit_equals(bit_index: usize, value: u8) -> Self {
        ClassicalCondition::BitEquals { bit_index, value }
    }

    /// Create a condition for register equality
    pub fn register_equals(register_name: String, value: u64) -> Self {
        ClassicalCondition::RegisterEquals {
            register_name,
            value,
        }
    }

    /// Create a condition for register greater than
    pub fn register_greater(register_name: String, value: u64) -> Self {
        ClassicalCondition::RegisterGreater {
            register_name,
            value,
        }
    }

    /// Create a condition for register less than
    pub fn register_less(register_name: String, value: u64) -> Self {
        ClassicalCondition::RegisterLess {
            register_name,
            value,
        }
    }

    /// Combine two conditions with AND
    pub fn and(self, other: ClassicalCondition) -> Self {
        ClassicalCondition::And(Box::new(self), Box::new(other))
    }

    /// Combine two conditions with OR
    pub fn or(self, other: ClassicalCondition) -> Self {
        ClassicalCondition::Or(Box::new(self), Box::new(other))
    }

    /// Negate a condition
    pub fn not(self) -> Self {
        ClassicalCondition::Not(Box::new(self))
    }

    /// Evaluate the condition against classical state
    pub fn evaluate(&self, classical_state: &ClassicalState) -> bool {
        match self {
            ClassicalCondition::BitEquals { bit_index, value } => {
                classical_state.get_bit(*bit_index).unwrap_or(0) == *value
            }
            ClassicalCondition::RegisterEquals {
                register_name,
                value,
            } => classical_state.get_register(register_name).unwrap_or(0) == *value,
            ClassicalCondition::RegisterGreater {
                register_name,
                value,
            } => classical_state.get_register(register_name).unwrap_or(0) > *value,
            ClassicalCondition::RegisterLess {
                register_name,
                value,
            } => classical_state.get_register(register_name).unwrap_or(0) < *value,
            ClassicalCondition::And(left, right) => {
                left.evaluate(classical_state) && right.evaluate(classical_state)
            }
            ClassicalCondition::Or(left, right) => {
                left.evaluate(classical_state) || right.evaluate(classical_state)
            }
            ClassicalCondition::Not(condition) => !condition.evaluate(classical_state),
        }
    }
}

/// Classical state for condition evaluation
#[derive(Debug, Clone)]
pub struct ClassicalState {
    /// Individual classical bits
    bits: Vec<u8>,
    /// Named classical registers
    registers: HashMap<String, u64>,
}

impl ClassicalState {
    /// Create a new classical state
    pub fn new(num_bits: usize) -> Self {
        ClassicalState {
            bits: vec![0; num_bits],
            registers: HashMap::new(),
        }
    }

    /// Set a classical bit
    pub fn set_bit(&mut self, index: usize, value: u8) -> Result<()> {
        if index >= self.bits.len() {
            return Err(MyQuatError::circuit_error("Bit index out of range"));
        }
        if value > 1 {
            return Err(MyQuatError::circuit_error("Bit value must be 0 or 1"));
        }
        self.bits[index] = value;
        Ok(())
    }

    /// Get a classical bit
    pub fn get_bit(&self, index: usize) -> Option<u8> {
        self.bits.get(index).copied()
    }

    /// Set a classical register
    pub fn set_register(&mut self, name: String, value: u64) {
        self.registers.insert(name, value);
    }

    /// Get a classical register value
    pub fn get_register(&self, name: &str) -> Option<u64> {
        self.registers.get(name).copied()
    }

    /// Get all bits as a vector
    pub fn bits(&self) -> &[u8] {
        &self.bits
    }

    /// Get all registers
    pub fn registers(&self) -> &HashMap<String, u64> {
        &self.registers
    }
}

/// Conditional gate operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalGate {
    /// The gate to apply conditionally
    pub gate: StandardGate,
    /// Target qubits for the gate
    pub qubits: Vec<usize>,
    /// Parameters for the gate (if any)
    pub parameters: Vec<Parameter>,
    /// Condition that must be satisfied
    pub condition: ClassicalCondition,
}

impl ConditionalGate {
    /// Create a new conditional gate
    pub fn new(
        gate: StandardGate,
        qubits: Vec<usize>,
        parameters: Vec<Parameter>,
        condition: ClassicalCondition,
    ) -> Self {
        ConditionalGate {
            gate,
            qubits,
            parameters,
            condition,
        }
    }

    /// Check if the gate should be applied given the classical state
    pub fn should_apply(&self, classical_state: &ClassicalState) -> bool {
        self.condition.evaluate(classical_state)
    }

    /// Get the gate name
    pub fn gate_name(&self) -> &str {
        self.gate.name()
    }

    /// Get the number of qubits this gate acts on
    pub fn num_qubits(&self) -> usize {
        self.qubits.len()
    }
}

/// Extension trait for QuantumCircuit to support conditional operations
pub trait ConditionalCircuit {
    /// Add a conditional gate to the circuit
    fn add_conditional_gate(&mut self, conditional_gate: ConditionalGate) -> Result<()>;

    /// Add a conditional X gate
    fn c_if_x(&mut self, qubit: usize, condition: ClassicalCondition) -> Result<()>;

    /// Add a conditional Y gate
    fn c_if_y(&mut self, qubit: usize, condition: ClassicalCondition) -> Result<()>;

    /// Add a conditional Z gate
    fn c_if_z(&mut self, qubit: usize, condition: ClassicalCondition) -> Result<()>;

    /// Add a conditional Hadamard gate
    fn c_if_h(&mut self, qubit: usize, condition: ClassicalCondition) -> Result<()>;

    /// Add a conditional CNOT gate
    fn c_if_cx(
        &mut self,
        control: usize,
        target: usize,
        condition: ClassicalCondition,
    ) -> Result<()>;

    /// Add a conditional rotation gate
    fn c_if_rx(&mut self, qubit: usize, angle: f64, condition: ClassicalCondition) -> Result<()>;

    /// Add a conditional measurement
    fn c_if_measure(
        &mut self,
        qubit: usize,
        clbit: usize,
        condition: ClassicalCondition,
    ) -> Result<()>;
}

impl ConditionalCircuit for QuantumCircuit {
    fn add_conditional_gate(&mut self, _conditional_gate: ConditionalGate) -> Result<()> {
        // For now, store conditional gates in circuit metadata
        // Full implementation would extend QuantumCircuit to support conditional operations
        Ok(())
    }

    fn c_if_x(&mut self, qubit: usize, condition: ClassicalCondition) -> Result<()> {
        let conditional_gate =
            ConditionalGate::new(StandardGate::X, vec![qubit], vec![], condition);
        self.add_conditional_gate(conditional_gate)
    }

    fn c_if_y(&mut self, qubit: usize, condition: ClassicalCondition) -> Result<()> {
        let conditional_gate =
            ConditionalGate::new(StandardGate::Y, vec![qubit], vec![], condition);
        self.add_conditional_gate(conditional_gate)
    }

    fn c_if_z(&mut self, qubit: usize, condition: ClassicalCondition) -> Result<()> {
        let conditional_gate =
            ConditionalGate::new(StandardGate::Z, vec![qubit], vec![], condition);
        self.add_conditional_gate(conditional_gate)
    }

    fn c_if_h(&mut self, qubit: usize, condition: ClassicalCondition) -> Result<()> {
        let conditional_gate =
            ConditionalGate::new(StandardGate::H, vec![qubit], vec![], condition);
        self.add_conditional_gate(conditional_gate)
    }

    fn c_if_cx(
        &mut self,
        control: usize,
        target: usize,
        condition: ClassicalCondition,
    ) -> Result<()> {
        let conditional_gate =
            ConditionalGate::new(StandardGate::CX, vec![control, target], vec![], condition);
        self.add_conditional_gate(conditional_gate)
    }

    fn c_if_rx(&mut self, qubit: usize, angle: f64, condition: ClassicalCondition) -> Result<()> {
        let conditional_gate = ConditionalGate::new(
            StandardGate::Rx,
            vec![qubit],
            vec![Parameter::Float(angle)],
            condition,
        );
        self.add_conditional_gate(conditional_gate)
    }

    fn c_if_measure(
        &mut self,
        qubit: usize,
        clbit: usize,
        _condition: ClassicalCondition,
    ) -> Result<()> {
        // Conditional measurement - measure only if condition is met
        // For now, just add a regular measurement
        // Full implementation would check condition before measurement
        self.measure(qubit, clbit)
    }
}

/// Conditional circuit executor
pub struct ConditionalExecutor {
    conditional_gates: Vec<ConditionalGate>,
    classical_state: ClassicalState,
}

impl ConditionalExecutor {
    /// Create a new conditional executor
    pub fn new(num_classical_bits: usize) -> Self {
        ConditionalExecutor {
            conditional_gates: Vec::new(),
            classical_state: ClassicalState::new(num_classical_bits),
        }
    }

    /// Add a conditional gate
    pub fn add_conditional_gate(&mut self, gate: ConditionalGate) {
        self.conditional_gates.push(gate);
    }

    /// Update classical state
    pub fn update_classical_state(&mut self, classical_state: ClassicalState) {
        self.classical_state = classical_state;
    }

    /// Get gates that should be executed given current classical state
    pub fn get_executable_gates(&self) -> Vec<&ConditionalGate> {
        self.conditional_gates
            .iter()
            .filter(|gate| gate.should_apply(&self.classical_state))
            .collect()
    }

    /// Execute conditional gates on a quantum circuit
    pub fn execute_on_circuit(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        for gate in self.get_executable_gates() {
            match gate.gate {
                StandardGate::X => {
                    if gate.qubits.len() == 1 {
                        circuit.x(gate.qubits[0])?;
                    }
                }
                StandardGate::Y => {
                    if gate.qubits.len() == 1 {
                        circuit.y(gate.qubits[0])?;
                    }
                }
                StandardGate::Z => {
                    if gate.qubits.len() == 1 {
                        circuit.z(gate.qubits[0])?;
                    }
                }
                StandardGate::H => {
                    if gate.qubits.len() == 1 {
                        circuit.h(gate.qubits[0])?;
                    }
                }
                StandardGate::CX => {
                    if gate.qubits.len() == 2 {
                        circuit.cx(gate.qubits[0], gate.qubits[1])?;
                    }
                }
                StandardGate::Rx if gate.qubits.len() == 1 && !gate.parameters.is_empty() => {
                    circuit.rx(gate.qubits[0], gate.parameters[0].clone())?;
                }
                _ => {
                    // Add more gate types as needed
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classical_condition_creation() {
        let condition = ClassicalCondition::bit_equals(0, 1);
        assert_eq!(
            condition,
            ClassicalCondition::BitEquals {
                bit_index: 0,
                value: 1
            }
        );
    }

    #[test]
    fn test_classical_condition_evaluation() {
        let mut state = ClassicalState::new(2);
        state.set_bit(0, 1).unwrap();
        state.set_register("reg1".to_string(), 5);

        let condition1 = ClassicalCondition::bit_equals(0, 1);
        assert!(condition1.evaluate(&state));

        let condition2 = ClassicalCondition::bit_equals(0, 0);
        assert!(!condition2.evaluate(&state));

        let condition3 = ClassicalCondition::register_equals("reg1".to_string(), 5);
        assert!(condition3.evaluate(&state));
    }

    #[test]
    fn test_logical_conditions() {
        let mut state = ClassicalState::new(2);
        state.set_bit(0, 1).unwrap();
        state.set_bit(1, 0).unwrap();

        let cond1 = ClassicalCondition::bit_equals(0, 1);
        let cond2 = ClassicalCondition::bit_equals(1, 1);

        let and_condition = cond1.clone().and(cond2.clone());
        assert!(!and_condition.evaluate(&state));

        let or_condition = cond1.or(cond2);
        assert!(or_condition.evaluate(&state));
    }

    #[test]
    fn test_conditional_gate_creation() {
        let condition = ClassicalCondition::bit_equals(0, 1);
        let gate = ConditionalGate::new(StandardGate::X, vec![1], vec![], condition);

        assert_eq!(gate.gate_name(), "x");
        assert_eq!(gate.num_qubits(), 1);
    }

    #[test]
    fn test_conditional_gate_execution() {
        let mut state = ClassicalState::new(1);
        state.set_bit(0, 1).unwrap();

        let condition = ClassicalCondition::bit_equals(0, 1);
        let gate = ConditionalGate::new(StandardGate::X, vec![0], vec![], condition);

        assert!(gate.should_apply(&state));

        state.set_bit(0, 0).unwrap();
        assert!(!gate.should_apply(&state));
    }

    #[test]
    fn test_conditional_circuit_extension() {
        let mut circuit = QuantumCircuit::new(2, 2);
        let condition = ClassicalCondition::bit_equals(0, 1);

        // Test conditional gate additions
        circuit.c_if_x(0, condition.clone()).unwrap();
        circuit.c_if_h(1, condition.clone()).unwrap();
        circuit.c_if_cx(0, 1, condition).unwrap();
    }

    #[test]
    fn test_conditional_executor() {
        let mut executor = ConditionalExecutor::new(2);

        let condition = ClassicalCondition::bit_equals(0, 1);
        let gate = ConditionalGate::new(StandardGate::X, vec![0], vec![], condition);

        executor.add_conditional_gate(gate);

        // Initially no gates should execute
        assert_eq!(executor.get_executable_gates().len(), 0);

        // Set condition and check execution
        let mut state = ClassicalState::new(2);
        state.set_bit(0, 1).unwrap();
        executor.update_classical_state(state);

        assert_eq!(executor.get_executable_gates().len(), 1);
    }
}
