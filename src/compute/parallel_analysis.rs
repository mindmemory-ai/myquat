//! Parallel Circuit Analysis Tools
//! Author: gA4ss
//!
//! Parallel implementations of circuit analysis utilities using Rayon.

use crate::circuit_optimized::OptimizedCircuitData;
use crate::gates::StandardGate;
use rayon::prelude::*;

/// Parallel circuit analysis utilities
pub struct ParallelCircuitAnalysis;

impl ParallelCircuitAnalysis {
    /// Parallel gate counting
    ///
    /// Counts occurrences of a specific gate type in parallel
    pub fn count_gates_parallel(circuit: &OptimizedCircuitData, gate_type: StandardGate) -> usize {
        circuit
            .instructions()
            .par_iter()
            .filter(|inst| inst.gate_type() == gate_type)
            .count()
    }

    /// Parallel circuit depth calculation with dependency analysis
    ///
    /// Note: This is a simplified implementation. Full dependency-based
    /// depth calculation would require memoization.
    pub fn calculate_depth_parallel(circuit: &OptimizedCircuitData) -> usize {
        let instructions = circuit.instructions();
        let dependencies = circuit.dependencies();

        if instructions.is_empty() {
            return 0;
        }

        let depths: Vec<usize> = (0..instructions.len())
            .into_par_iter()
            .map(|i| {
                if dependencies[i].is_empty() {
                    1
                } else {
                    dependencies[i].iter().map(|_dep_idx| 1).max().unwrap_or(0) + 1
                }
            })
            .collect();

        depths.into_par_iter().max().unwrap_or(0)
    }

    /// Parallel qubit usage analysis
    ///
    /// Returns the number of gates acting on each qubit
    pub fn analyze_qubit_usage_parallel(circuit: &OptimizedCircuitData) -> Vec<usize> {
        let num_qubits = circuit.num_qubits();
        let instructions = circuit.instructions();

        (0..num_qubits)
            .into_par_iter()
            .map(|qubit| {
                instructions
                    .iter()
                    .filter(|inst| inst.qubits().contains(&(qubit as u16)))
                    .count()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::Qubit;
    use crate::gates::Gate;

    #[test]
    fn test_parallel_gate_counting() {
        let mut circuit_data = OptimizedCircuitData::new(2, 0).unwrap();

        circuit_data
            .add_instruction(&Gate::h(), &[Qubit::new(0)], &[])
            .unwrap();
        circuit_data
            .add_instruction(&Gate::x(), &[Qubit::new(1)], &[])
            .unwrap();
        circuit_data
            .add_instruction(&Gate::cx(), &[Qubit::new(0), Qubit::new(1)], &[])
            .unwrap();

        let h_count = ParallelCircuitAnalysis::count_gates_parallel(&circuit_data, StandardGate::H);
        assert_eq!(h_count, 1);

        let x_count = ParallelCircuitAnalysis::count_gates_parallel(&circuit_data, StandardGate::X);
        assert_eq!(x_count, 1);
    }

    #[test]
    fn test_parallel_qubit_usage() {
        let mut circuit_data = OptimizedCircuitData::new(2, 0).unwrap();

        circuit_data
            .add_instruction(&Gate::h(), &[Qubit::new(0)], &[])
            .unwrap();
        circuit_data
            .add_instruction(&Gate::x(), &[Qubit::new(1)], &[])
            .unwrap();
        circuit_data
            .add_instruction(&Gate::cx(), &[Qubit::new(0), Qubit::new(1)], &[])
            .unwrap();

        let usage = ParallelCircuitAnalysis::analyze_qubit_usage_parallel(&circuit_data);
        assert_eq!(usage.len(), 2);
        assert!(usage[0] > 0);
        assert!(usage[1] > 0);
    }
}
