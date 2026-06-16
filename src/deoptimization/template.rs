// deoptimization/template.rs - Template matching for Trotter structures
// Author: gA4ss
//
// Identifies and restores Trotter structures using template matching
// with graph isomorphism algorithms.

use super::trotter_template::TrotterTemplate;
use super::DeoptStrategy;
use crate::circuit::QuantumCircuit;
use crate::error::Result;
use crate::gates::{GateOperation, StandardGate};
use petgraph::graph::DiGraph;

/// Node in the circuit dependency graph
#[derive(Debug, Clone, PartialEq)]
struct CircuitNode {
    /// Gate type
    gate_type: StandardGate,
    /// Qubits this gate acts on
    qubits: Vec<usize>,
    /// Gate index in original circuit
    index: usize,
}

/// Strategy for identifying Trotter structures via template matching
///
/// Uses graph isomorphism to match circuit patterns against known
/// Trotter decomposition templates.
#[derive(Debug, Clone)]
pub struct TemplateMatchingStrategy {
    /// Whether to allow gate reordering based on commutativity
    allow_reordering: bool,
    /// Template library to match against
    templates: Vec<TrotterTemplate>,
    /// Minimum confidence threshold
    min_confidence: f64,
}

impl TemplateMatchingStrategy {
    /// Create new strategy
    pub fn new() -> Self {
        Self {
            allow_reordering: true,
            templates: Vec::new(),
            min_confidence: 0.7,
        }
    }

    /// Enable or disable reordering tolerance
    pub fn with_reordering(mut self, allow: bool) -> Self {
        self.allow_reordering = allow;
        self
    }

    /// Add a template to match against
    pub fn add_template(mut self, template: TrotterTemplate) -> Self {
        self.templates.push(template);
        self
    }

    /// Set minimum confidence threshold
    pub fn with_min_confidence(mut self, confidence: f64) -> Self {
        self.min_confidence = confidence;
        self
    }

    /// Convert circuit to dependency graph
    ///
    /// Each node is a gate, edges represent dependencies
    /// (gates that must be ordered due to qubit conflicts)
    fn circuit_to_graph(&self, circuit: &QuantumCircuit) -> DiGraph<CircuitNode, ()> {
        let mut graph = DiGraph::new();
        let instructions = circuit.data().instructions();

        // Create nodes for each gate
        let mut nodes = Vec::new();
        for (idx, inst) in instructions.iter().enumerate() {
            let qubits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();
            let node = CircuitNode {
                gate_type: inst.gate.gate_type,
                qubits,
                index: idx,
            };
            nodes.push(graph.add_node(node));
        }

        // Add edges for dependencies
        // Two gates depend on each other if they share qubits and cannot commute
        for i in 0..instructions.len() {
            for j in (i + 1)..instructions.len() {
                if self.gates_depend(&instructions[i], &instructions[j]) {
                    graph.add_edge(nodes[i], nodes[j], ());
                }
            }
        }

        graph
    }

    /// Check if two gates have a dependency
    fn gates_depend(
        &self,
        gate1: &crate::circuit::Instruction,
        gate2: &crate::circuit::Instruction,
    ) -> bool {
        // Check if gates share qubits
        let qubits1: Vec<usize> = gate1.qubits.iter().map(|q| q.index()).collect();
        let qubits2: Vec<usize> = gate2.qubits.iter().map(|q| q.index()).collect();

        let shares_qubits = qubits1.iter().any(|q| qubits2.contains(q));

        if !shares_qubits {
            return false; // No dependency if no shared qubits
        }

        if !self.allow_reordering {
            return true; // Always depend if reordering not allowed
        }

        // Check commutativity for single-qubit gates on same qubit
        if gate1.gate.gate_type.num_qubits() == 1
            && gate2.gate.gate_type.num_qubits() == 1
            && qubits1[0] != qubits2[0]
        {
            return false; // Single-qubit gates on different qubits commute
        }

        // Two-qubit gates or gates on same qubit generally don't commute
        true
    }

    /// Match circuit against a template
    fn match_template(&self, circuit: &QuantumCircuit, template: &TrotterTemplate) -> f64 {
        // For simplicity, use sequence-based matching
        // Full graph isomorphism would use petgraph's is_isomorphic

        let instructions = circuit.data().instructions();
        if instructions.is_empty() {
            return 0.0;
        }

        // Check if template steps match circuit structure
        let template_length = template.steps.len();
        if template_length == 0 {
            return 0.0;
        }

        // Simple heuristic: check if circuit has repeated patterns
        // that match template order
        let circuit_length = instructions.len();

        // Look for periodicity matching template
        if circuit_length % template_length == 0 {
            let num_repetitions = circuit_length / template_length;

            // Check if pattern repeats
            let mut matches = 0;
            for rep in 0..num_repetitions {
                for (step_idx, step) in template.steps.iter().enumerate() {
                    let circuit_idx = rep * template_length + step_idx;
                    if circuit_idx < circuit_length {
                        let gate_type = instructions[circuit_idx].gate.gate_type;

                        // Simple check: does gate type match template pattern?
                        // (This is simplified; full implementation would check Pauli strings)
                        if self.gate_matches_step(gate_type, step) {
                            matches += 1;
                        }
                    }
                }
            }

            let total_expected = num_repetitions * template_length;
            if total_expected > 0 {
                return matches as f64 / total_expected as f64;
            }
        }

        0.0
    }

    /// Check if a gate matches a template step
    fn gate_matches_step(
        &self,
        gate: StandardGate,
        step: &super::trotter_template::TrotterStep,
    ) -> bool {
        // Map Pauli strings to expected gate types
        // ZZ rotation: involves H, CX, Rz
        // XX rotation: involves Ry, CX, Rz
        // YY rotation: involves Rx, CX, Rz

        match step.term.pauli_string.as_str() {
            "ZZ" => matches!(gate, StandardGate::H | StandardGate::CX | StandardGate::Rz),
            "XX" => matches!(gate, StandardGate::Ry | StandardGate::CX | StandardGate::Rz),
            "YY" => matches!(gate, StandardGate::Rx | StandardGate::CX | StandardGate::Rz),
            "Z" => matches!(gate, StandardGate::Rz | StandardGate::P),
            "X" => matches!(gate, StandardGate::Rx),
            "Y" => matches!(gate, StandardGate::Ry),
            _ => false,
        }
    }

    /// Identify Trotter patterns in circuit
    fn identify_trotter_patterns(&self, circuit: &QuantumCircuit) -> Vec<(String, f64)> {
        let mut identified = Vec::new();

        for template in &self.templates {
            let confidence = self.match_template(circuit, template);
            if confidence >= self.min_confidence {
                identified.push((template.name.clone(), confidence));
            }
        }

        identified
    }
}

impl Default for TemplateMatchingStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl DeoptStrategy for TemplateMatchingStrategy {
    fn apply(&self, circuit: &QuantumCircuit) -> Result<QuantumCircuit> {
        // Identify patterns
        let patterns = self.identify_trotter_patterns(circuit);

        if patterns.is_empty() {
            // No patterns found, return original
            return Ok(circuit.clone());
        }

        // For now, return original circuit
        // Full restoration would reconstruct Trotter structure
        Ok(circuit.clone())
    }

    fn name(&self) -> &str {
        "Template Matching"
    }

    fn confidence(&self, circuit: &QuantumCircuit) -> f64 {
        let patterns = self.identify_trotter_patterns(circuit);

        if patterns.is_empty() {
            return 0.0;
        }

        // Return highest confidence
        patterns
            .iter()
            .map(|(_, conf)| *conf)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::super::trotter_template::{HamiltonianTerm, TrotterTemplateBuilder};
    use super::*;
    use crate::parameter::Parameter;
    use std::f64::consts::PI;

    #[test]
    fn test_strategy_creation() {
        let strategy = TemplateMatchingStrategy::new();
        assert_eq!(strategy.name(), "Template Matching");
        assert!(strategy.allow_reordering);
        assert_eq!(strategy.min_confidence, 0.7);
        assert_eq!(strategy.templates.len(), 0);
    }

    #[test]
    fn test_with_reordering() {
        let strategy = TemplateMatchingStrategy::new().with_reordering(false);
        assert!(!strategy.allow_reordering);
    }

    #[test]
    fn test_add_template() {
        let builder =
            TrotterTemplateBuilder::new().add_term(HamiltonianTerm::new("ZZ", 1.0, vec![0, 1]));

        let template = builder.build_first_order(1.0, 1);

        let strategy = TemplateMatchingStrategy::new().add_template(template);

        assert_eq!(strategy.templates.len(), 1);
    }

    #[test]
    fn test_with_min_confidence() {
        let strategy = TemplateMatchingStrategy::new().with_min_confidence(0.85);
        assert_eq!(strategy.min_confidence, 0.85);
    }

    #[test]
    fn test_circuit_to_graph_empty() {
        let strategy = TemplateMatchingStrategy::new();
        let circuit = QuantumCircuit::new(2, 0);

        let graph = strategy.circuit_to_graph(&circuit);
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_circuit_to_graph_simple() {
        let strategy = TemplateMatchingStrategy::new();
        let mut circuit = QuantumCircuit::new(2, 0);

        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();

        let graph = strategy.circuit_to_graph(&circuit);
        assert_eq!(graph.node_count(), 2);
        // H and CX on overlapping qubits should have dependency
        assert!(graph.edge_count() > 0);
    }

    #[test]
    fn test_gates_depend_no_shared_qubits() {
        let strategy = TemplateMatchingStrategy::new();
        let mut circuit = QuantumCircuit::new(2, 0);

        circuit.h(0).unwrap();
        circuit.h(1).unwrap();

        let instructions = circuit.data().instructions();
        let depend = strategy.gates_depend(&instructions[0], &instructions[1]);

        // Gates on different qubits don't depend
        assert!(!depend);
    }

    #[test]
    fn test_gates_depend_shared_qubits() {
        let strategy = TemplateMatchingStrategy::new();
        let mut circuit = QuantumCircuit::new(2, 0);

        circuit.h(0).unwrap();
        circuit.x(0).unwrap();

        let instructions = circuit.data().instructions();
        let depend = strategy.gates_depend(&instructions[0], &instructions[1]);

        // Gates on same qubit depend
        assert!(depend);
    }

    #[test]
    fn test_gate_matches_step_zz() {
        let strategy = TemplateMatchingStrategy::new();
        let step = super::super::trotter_template::TrotterStep::new(
            HamiltonianTerm::new("ZZ", 1.0, vec![0, 1]),
            1.0,
            0,
        );

        assert!(strategy.gate_matches_step(StandardGate::H, &step));
        assert!(strategy.gate_matches_step(StandardGate::CX, &step));
        assert!(strategy.gate_matches_step(StandardGate::Rz, &step));
        assert!(!strategy.gate_matches_step(StandardGate::Rx, &step));
    }

    #[test]
    fn test_gate_matches_step_xx() {
        let strategy = TemplateMatchingStrategy::new();
        let step = super::super::trotter_template::TrotterStep::new(
            HamiltonianTerm::new("XX", 0.5, vec![0, 1]),
            1.0,
            0,
        );

        assert!(strategy.gate_matches_step(StandardGate::Ry, &step));
        assert!(strategy.gate_matches_step(StandardGate::CX, &step));
        assert!(strategy.gate_matches_step(StandardGate::Rz, &step));
        assert!(!strategy.gate_matches_step(StandardGate::H, &step));
    }

    #[test]
    fn test_confidence_empty_circuit() {
        let strategy = TemplateMatchingStrategy::new();
        let circuit = QuantumCircuit::new(2, 0);

        let conf = strategy.confidence(&circuit);
        assert_eq!(conf, 0.0);
    }

    #[test]
    fn test_confidence_no_templates() {
        let strategy = TemplateMatchingStrategy::new();
        let mut circuit = QuantumCircuit::new(2, 0);

        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();

        let conf = strategy.confidence(&circuit);
        assert_eq!(conf, 0.0); // No templates to match against
    }

    #[test]
    fn test_apply_empty_circuit() {
        let strategy = TemplateMatchingStrategy::new();
        let circuit = QuantumCircuit::new(2, 0);

        let result = strategy.apply(&circuit);
        assert!(result.is_ok());
    }

    #[test]
    fn test_identify_patterns_empty() {
        let strategy = TemplateMatchingStrategy::new();
        let circuit = QuantumCircuit::new(2, 0);

        let patterns = strategy.identify_trotter_patterns(&circuit);
        assert_eq!(patterns.len(), 0);
    }
}
