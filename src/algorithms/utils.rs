//! Algorithm execution utilities
//!
//! This module provides common utilities for running and analyzing
//! quantum algorithms.

use crate::error::Result;
use crate::{QuantumCircuit, StateVectorSimulator};
use std::collections::HashMap;

/// Algorithm execution utilities
pub struct AlgorithmRunner;

impl AlgorithmRunner {
    /// Run algorithm and return measurement results
    pub fn run_algorithm(circuit: QuantumCircuit, shots: usize) -> Result<HashMap<String, usize>> {
        let mut simulator = StateVectorSimulator::new(circuit.num_qubits(), circuit.num_clbits());
        let mut total_results = HashMap::new();

        for _ in 0..shots {
            let shot_results = simulator.execute_circuit(&circuit)?;
            // Merge results from this shot
            for (bitstring, count) in shot_results {
                *total_results.entry(bitstring).or_insert(0) += count;
            }
        }

        Ok(total_results)
    }

    /// Calculate success probability for Grover search
    pub fn calculate_grover_success_probability(
        results: &HashMap<String, usize>,
        marked_items: &[usize],
        _num_qubits: usize,
    ) -> f64 {
        let total_shots: usize = results.values().sum();
        if total_shots == 0 {
            return 0.0;
        }

        let mut success_count = 0;
        for (bitstring, count) in results {
            if let Ok(value) = usize::from_str_radix(bitstring, 2) {
                if marked_items.contains(&value) {
                    success_count += count;
                }
            }
        }

        success_count as f64 / total_shots as f64
    }

    /// Calculate expectation value for a given observable
    pub fn calculate_expectation_value(
        results: &HashMap<String, usize>,
        observable_values: &HashMap<String, f64>,
    ) -> f64 {
        let total_shots: usize = results.values().sum();
        if total_shots == 0 {
            return 0.0;
        }

        let mut expectation = 0.0;
        for (bitstring, count) in results {
            if let Some(&value) = observable_values.get(bitstring) {
                expectation += value * (*count as f64) / (total_shots as f64);
            }
        }

        expectation
    }

    /// Calculate variance for measurement results
    pub fn calculate_variance(
        results: &HashMap<String, usize>,
        observable_values: &HashMap<String, f64>,
    ) -> f64 {
        let expectation = Self::calculate_expectation_value(results, observable_values);
        let total_shots: usize = results.values().sum();
        if total_shots == 0 {
            return 0.0;
        }

        let mut variance = 0.0;
        for (bitstring, count) in results {
            if let Some(&value) = observable_values.get(bitstring) {
                let diff = value - expectation;
                variance += diff * diff * (*count as f64) / (total_shots as f64);
            }
        }

        variance
    }

    /// Find the most probable measurement outcome
    pub fn most_probable_outcome(results: &HashMap<String, usize>) -> Option<String> {
        results
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(bitstring, _)| bitstring.clone())
    }

    /// Calculate entropy of measurement results
    pub fn calculate_entropy(results: &HashMap<String, usize>) -> f64 {
        let total_shots: usize = results.values().sum();
        if total_shots == 0 {
            return 0.0;
        }

        let mut entropy = 0.0;
        for &count in results.values() {
            if count > 0 {
                let prob = count as f64 / total_shots as f64;
                entropy -= prob * prob.log2();
            }
        }

        entropy
    }

    /// Compare two sets of results using fidelity
    pub fn calculate_fidelity(
        results1: &HashMap<String, usize>,
        results2: &HashMap<String, usize>,
    ) -> f64 {
        let total1: usize = results1.values().sum();
        let total2: usize = results2.values().sum();

        if total1 == 0 || total2 == 0 {
            return 0.0;
        }

        let mut fidelity = 0.0;
        let mut all_states = std::collections::HashSet::new();

        for state in results1.keys() {
            all_states.insert(state.clone());
        }
        for state in results2.keys() {
            all_states.insert(state.clone());
        }

        for state in all_states {
            let prob1 = results1.get(&state).unwrap_or(&0);
            let prob2 = results2.get(&state).unwrap_or(&0);

            let p1 = *prob1 as f64 / total1 as f64;
            let p2 = *prob2 as f64 / total2 as f64;

            fidelity += (p1 * p2).sqrt();
        }

        fidelity
    }

    /// Generate summary statistics for results
    pub fn generate_statistics(results: &HashMap<String, usize>) -> AlgorithmStatistics {
        let total_shots: usize = results.values().sum();
        let num_outcomes = results.len();
        let most_probable = Self::most_probable_outcome(results);
        let entropy = Self::calculate_entropy(results);

        let max_count = results.values().max().unwrap_or(&0);
        let max_probability = if total_shots > 0 {
            *max_count as f64 / total_shots as f64
        } else {
            0.0
        };

        AlgorithmStatistics {
            total_shots,
            num_outcomes,
            most_probable,
            max_probability,
            entropy,
        }
    }
}

/// Statistics for algorithm results
#[derive(Debug, Clone)]
pub struct AlgorithmStatistics {
    pub total_shots: usize,
    pub num_outcomes: usize,
    pub most_probable: Option<String>,
    pub max_probability: f64,
    pub entropy: f64,
}

impl AlgorithmStatistics {
    /// Check if results show good concentration
    pub fn is_well_concentrated(&self, threshold: f64) -> bool {
        self.max_probability >= threshold
    }

    /// Check if results are uniformly distributed
    pub fn is_uniform(&self, tolerance: f64) -> bool {
        if self.num_outcomes == 0 {
            return false;
        }

        let expected_entropy = (self.num_outcomes as f64).log2();
        (self.entropy - expected_entropy).abs() < tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grover_success_probability() {
        let mut results = HashMap::new();
        results.insert("101".to_string(), 30); // Target state
        results.insert("000".to_string(), 20);
        results.insert("111".to_string(), 50);

        let marked_items = vec![5]; // 101 in binary
        let prob =
            AlgorithmRunner::calculate_grover_success_probability(&results, &marked_items, 3);
        assert_eq!(prob, 0.3); // 30/100
    }

    #[test]
    fn test_expectation_value() {
        let mut results = HashMap::new();
        results.insert("00".to_string(), 50);
        results.insert("11".to_string(), 50);

        let mut observable = HashMap::new();
        observable.insert("00".to_string(), 1.0);
        observable.insert("11".to_string(), -1.0);

        let expectation = AlgorithmRunner::calculate_expectation_value(&results, &observable);
        assert_eq!(expectation, 0.0); // (1.0 * 0.5) + (-1.0 * 0.5) = 0
    }

    #[test]
    fn test_most_probable_outcome() {
        let mut results = HashMap::new();
        results.insert("00".to_string(), 10);
        results.insert("01".to_string(), 30);
        results.insert("10".to_string(), 20);

        let most_probable = AlgorithmRunner::most_probable_outcome(&results);
        assert_eq!(most_probable, Some("01".to_string()));
    }

    #[test]
    fn test_entropy_calculation() {
        let mut results = HashMap::new();
        results.insert("0".to_string(), 50);
        results.insert("1".to_string(), 50);

        let entropy = AlgorithmRunner::calculate_entropy(&results);
        assert!((entropy - 1.0).abs() < 1e-10); // Perfect entropy for 2 equal outcomes
    }

    #[test]
    fn test_fidelity_calculation() {
        let mut results1 = HashMap::new();
        results1.insert("00".to_string(), 50);
        results1.insert("11".to_string(), 50);

        let mut results2 = HashMap::new();
        results2.insert("00".to_string(), 60);
        results2.insert("11".to_string(), 40);

        let fidelity = AlgorithmRunner::calculate_fidelity(&results1, &results2);
        assert!(fidelity > 0.9); // Should be high fidelity
    }

    #[test]
    fn test_statistics_generation() {
        let mut results = HashMap::new();
        results.insert("00".to_string(), 70);
        results.insert("01".to_string(), 20);
        results.insert("10".to_string(), 10);

        let stats = AlgorithmRunner::generate_statistics(&results);
        assert_eq!(stats.total_shots, 100);
        assert_eq!(stats.num_outcomes, 3);
        assert_eq!(stats.most_probable, Some("00".to_string()));
        assert_eq!(stats.max_probability, 0.7);
        assert!(stats.is_well_concentrated(0.6));
        assert!(!stats.is_uniform(0.1));
    }
}
