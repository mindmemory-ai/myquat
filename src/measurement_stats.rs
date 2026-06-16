//! Advanced measurement statistics and analysis
//!
//! This module provides comprehensive statistical analysis of quantum
//! measurement results including histograms, correlations, and visualization.

use crate::error::{MyQuatError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comprehensive measurement statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementStatistics {
    /// Total number of shots
    pub total_shots: usize,
    /// Number of unique outcomes
    pub unique_outcomes: usize,
    /// Most frequent outcome
    pub most_frequent: Option<String>,
    /// Least frequent outcome
    pub least_frequent: Option<String>,
    /// Maximum probability
    pub max_probability: f64,
    /// Minimum probability
    pub min_probability: f64,
    /// Shannon entropy
    pub entropy: f64,
    /// Variance of the distribution
    pub variance: f64,
    /// Standard deviation
    pub std_deviation: f64,
    /// Gini coefficient (measure of inequality)
    pub gini_coefficient: f64,
}

impl MeasurementStatistics {
    /// Calculate comprehensive statistics from measurement results
    pub fn from_results(results: &HashMap<String, usize>) -> Self {
        let total_shots: usize = results.values().sum();
        let unique_outcomes = results.len();

        if total_shots == 0 {
            return MeasurementStatistics::default();
        }

        // Find most and least frequent outcomes
        let (most_frequent, max_count) = results
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(k, &v)| (Some(k.clone()), v))
            .unwrap_or((None, 0));

        let (least_frequent, min_count) = results
            .iter()
            .min_by_key(|(_, &count)| count)
            .map(|(k, &v)| (Some(k.clone()), v))
            .unwrap_or((None, 0));

        let max_probability = max_count as f64 / total_shots as f64;
        let min_probability = min_count as f64 / total_shots as f64;

        // Calculate entropy
        let entropy = Self::calculate_entropy(results, total_shots);

        // Calculate variance and standard deviation
        let mean_probability = 1.0 / unique_outcomes as f64;
        let variance = results
            .values()
            .map(|&count| {
                let prob = count as f64 / total_shots as f64;
                (prob - mean_probability).powi(2)
            })
            .sum::<f64>()
            / unique_outcomes as f64;

        let std_deviation = variance.sqrt();

        // Calculate Gini coefficient
        let gini_coefficient = Self::calculate_gini_coefficient(results, total_shots);

        MeasurementStatistics {
            total_shots,
            unique_outcomes,
            most_frequent,
            least_frequent,
            max_probability,
            min_probability,
            entropy,
            variance,
            std_deviation,
            gini_coefficient,
        }
    }

    /// Calculate Shannon entropy
    fn calculate_entropy(results: &HashMap<String, usize>, total_shots: usize) -> f64 {
        results
            .values()
            .filter(|&&count| count > 0)
            .map(|&count| {
                let prob = count as f64 / total_shots as f64;
                -prob * prob.log2()
            })
            .sum()
    }

    /// Calculate Gini coefficient
    fn calculate_gini_coefficient(results: &HashMap<String, usize>, total_shots: usize) -> f64 {
        let mut probabilities: Vec<f64> = results
            .values()
            .map(|&count| count as f64 / total_shots as f64)
            .collect();
        probabilities.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = probabilities.len();
        if n <= 1 {
            return 0.0;
        }

        let sum_weighted: f64 = probabilities
            .iter()
            .enumerate()
            .map(|(i, &prob)| (2_i32 * i as i32 + 1 - n as i32) as f64 * prob)
            .sum();

        sum_weighted / (n as f64 * probabilities.iter().sum::<f64>())
    }

    /// Check if distribution is uniform within tolerance
    pub fn is_uniform(&self, tolerance: f64) -> bool {
        if self.unique_outcomes == 0 {
            return false;
        }

        let expected_entropy = (self.unique_outcomes as f64).log2();
        (self.entropy - expected_entropy).abs() < tolerance
    }

    /// Check if distribution is concentrated (high max probability)
    pub fn is_concentrated(&self, threshold: f64) -> bool {
        self.max_probability >= threshold
    }
}

impl Default for MeasurementStatistics {
    fn default() -> Self {
        MeasurementStatistics {
            total_shots: 0,
            unique_outcomes: 0,
            most_frequent: None,
            least_frequent: None,
            max_probability: 0.0,
            min_probability: 0.0,
            entropy: 0.0,
            variance: 0.0,
            std_deviation: 0.0,
            gini_coefficient: 0.0,
        }
    }
}

/// Histogram for measurement results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementHistogram {
    /// Bins and their counts
    pub bins: HashMap<String, usize>,
    /// Total number of measurements
    pub total_count: usize,
    /// Bin width (for continuous data)
    pub bin_width: Option<f64>,
}

impl MeasurementHistogram {
    /// Create histogram from measurement results
    pub fn from_results(results: &HashMap<String, usize>) -> Self {
        MeasurementHistogram {
            bins: results.clone(),
            total_count: results.values().sum(),
            bin_width: None,
        }
    }

    /// Get probability for a specific outcome
    pub fn probability(&self, outcome: &str) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }

        *self.bins.get(outcome).unwrap_or(&0) as f64 / self.total_count as f64
    }

    /// Get the most probable outcome
    pub fn mode(&self) -> Option<String> {
        self.bins
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(outcome, _)| outcome.clone())
    }

    /// Generate ASCII histogram visualization
    pub fn to_ascii(&self, max_width: usize) -> String {
        if self.bins.is_empty() {
            return "No data".to_string();
        }

        let max_count = *self.bins.values().max().unwrap_or(&0);
        if max_count == 0 {
            return "No measurements".to_string();
        }

        let mut lines = Vec::new();
        let mut sorted_bins: Vec<_> = self.bins.iter().collect();
        sorted_bins.sort_by_key(|(outcome, _)| outcome.as_str());

        for (outcome, &count) in sorted_bins {
            let probability = count as f64 / self.total_count as f64;
            let bar_length = (count * max_width / max_count).max(1);
            let bar = "█".repeat(bar_length);

            lines.push(format!(
                "{:>8} |{:<width$} {} ({:.1}%)",
                outcome,
                bar,
                count,
                probability * 100.0,
                width = max_width
            ));
        }

        lines.join("\n")
    }
}

/// Correlation analysis for multi-qubit measurements
#[derive(Debug, Clone)]
pub struct CorrelationAnalysis {
    /// Correlation matrix between qubits
    pub correlation_matrix: Vec<Vec<f64>>,
    /// Number of qubits
    pub num_qubits: usize,
}

impl CorrelationAnalysis {
    /// Analyze correlations in multi-qubit measurement results
    pub fn from_results(results: &HashMap<String, usize>, num_qubits: usize) -> Result<Self> {
        let total_shots: usize = results.values().sum();
        if total_shots == 0 {
            return Err(MyQuatError::circuit_error("No measurement data"));
        }

        let mut correlation_matrix = vec![vec![0.0; num_qubits]; num_qubits];

        // Calculate pairwise correlations
        for i in 0..num_qubits {
            for j in 0..num_qubits {
                if i == j {
                    correlation_matrix[i][j] = 1.0; // Perfect self-correlation
                } else {
                    correlation_matrix[i][j] =
                        Self::calculate_correlation(results, i, j, total_shots)?;
                }
            }
        }

        Ok(CorrelationAnalysis {
            correlation_matrix,
            num_qubits,
        })
    }

    /// Calculate correlation between two qubits
    fn calculate_correlation(
        results: &HashMap<String, usize>,
        qubit1: usize,
        qubit2: usize,
        total_shots: usize,
    ) -> Result<f64> {
        let mut n00 = 0; // Both qubits 0
        let mut n01 = 0; // Qubit1=0, Qubit2=1
        let mut n10 = 0; // Qubit1=1, Qubit2=0
        let mut n11 = 0; // Both qubits 1

        for (bitstring, &count) in results {
            let bits: Vec<char> = bitstring.chars().collect();
            if qubit1 >= bits.len() || qubit2 >= bits.len() {
                continue;
            }

            let bit1 = bits[qubit1] == '1';
            let bit2 = bits[qubit2] == '1';

            match (bit1, bit2) {
                (false, false) => n00 += count,
                (false, true) => n01 += count,
                (true, false) => n10 += count,
                (true, true) => n11 += count,
            }
        }

        // Calculate Pearson correlation coefficient
        let _p00 = n00 as f64 / total_shots as f64;
        let p01 = n01 as f64 / total_shots as f64;
        let p10 = n10 as f64 / total_shots as f64;
        let p11 = n11 as f64 / total_shots as f64;

        let p1 = p10 + p11; // P(qubit1 = 1)
        let p2 = p01 + p11; // P(qubit2 = 1)

        let covariance = p11 - p1 * p2;
        let variance1 = p1 * (1.0 - p1);
        let variance2 = p2 * (1.0 - p2);

        if variance1 * variance2 == 0.0 {
            Ok(0.0)
        } else {
            Ok(covariance / (variance1 * variance2).sqrt())
        }
    }

    /// Get correlation between two specific qubits
    pub fn get_correlation(&self, qubit1: usize, qubit2: usize) -> Option<f64> {
        if qubit1 < self.num_qubits && qubit2 < self.num_qubits {
            Some(self.correlation_matrix[qubit1][qubit2])
        } else {
            None
        }
    }

    /// Find the most correlated qubit pairs
    pub fn most_correlated_pairs(&self, threshold: f64) -> Vec<(usize, usize, f64)> {
        let mut correlations = Vec::new();

        for i in 0..self.num_qubits {
            for j in (i + 1)..self.num_qubits {
                let corr = self.correlation_matrix[i][j].abs();
                if corr >= threshold {
                    correlations.push((i, j, corr));
                }
            }
        }

        correlations.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        correlations
    }
}

/// Statistical test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalTests {
    /// Chi-square test for uniformity
    pub chi_square_test: ChiSquareTest,
    /// Kolmogorov-Smirnov test
    pub ks_test: Option<KSTest>,
}

/// Chi-square test for uniformity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChiSquareTest {
    /// Chi-square statistic
    pub chi_square: f64,
    /// Degrees of freedom
    pub degrees_of_freedom: usize,
    /// P-value (approximate)
    pub p_value: f64,
    /// Is the distribution significantly different from uniform?
    pub is_significant: bool,
}

/// Kolmogorov-Smirnov test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KSTest {
    /// KS statistic
    pub ks_statistic: f64,
    /// P-value (approximate)
    pub p_value: f64,
    /// Is the distribution significantly different from expected?
    pub is_significant: bool,
}

impl StatisticalTests {
    /// Perform statistical tests on measurement results
    pub fn analyze(results: &HashMap<String, usize>) -> Self {
        let chi_square_test = Self::chi_square_uniformity_test(results);

        StatisticalTests {
            chi_square_test,
            ks_test: None, // Could implement KS test
        }
    }

    /// Chi-square test for uniformity
    fn chi_square_uniformity_test(results: &HashMap<String, usize>) -> ChiSquareTest {
        let total_shots: usize = results.values().sum();
        let num_outcomes = results.len();

        if total_shots == 0 || num_outcomes == 0 {
            return ChiSquareTest {
                chi_square: 0.0,
                degrees_of_freedom: 0,
                p_value: 1.0,
                is_significant: false,
            };
        }

        let expected = total_shots as f64 / num_outcomes as f64;
        let chi_square: f64 = results
            .values()
            .map(|&observed| {
                let diff = observed as f64 - expected;
                diff * diff / expected
            })
            .sum();

        let degrees_of_freedom = num_outcomes - 1;

        // Approximate p-value calculation (simplified)
        let p_value = if chi_square > 20.0 { 0.001 } else { 0.1 };
        let is_significant = p_value < 0.05;

        ChiSquareTest {
            chi_square,
            degrees_of_freedom,
            p_value,
            is_significant,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measurement_statistics() {
        let mut results = HashMap::new();
        results.insert("00".to_string(), 25);
        results.insert("01".to_string(), 25);
        results.insert("10".to_string(), 25);
        results.insert("11".to_string(), 25);

        let stats = MeasurementStatistics::from_results(&results);
        assert_eq!(stats.total_shots, 100);
        assert_eq!(stats.unique_outcomes, 4);
        assert!(stats.is_uniform(0.1));
        assert!(!stats.is_concentrated(0.5));
    }

    #[test]
    fn test_histogram() {
        let mut results = HashMap::new();
        results.insert("0".to_string(), 30);
        results.insert("1".to_string(), 70);

        let histogram = MeasurementHistogram::from_results(&results);
        assert_eq!(histogram.probability("0"), 0.3);
        assert_eq!(histogram.probability("1"), 0.7);
        assert_eq!(histogram.mode(), Some("1".to_string()));
    }

    #[test]
    fn test_correlation_analysis() {
        let mut results = HashMap::new();
        results.insert("00".to_string(), 40);
        results.insert("01".to_string(), 10);
        results.insert("10".to_string(), 10);
        results.insert("11".to_string(), 40);

        let correlation = CorrelationAnalysis::from_results(&results, 2).unwrap();
        let corr_01 = correlation.get_correlation(0, 1).unwrap();

        // Should be positive correlation since 00 and 11 are more frequent
        assert!(corr_01 > 0.0);
    }

    #[test]
    fn test_chi_square_test() {
        let mut results = HashMap::new();
        results.insert("0".to_string(), 50);
        results.insert("1".to_string(), 50);

        let tests = StatisticalTests::analyze(&results);
        assert!(tests.chi_square_test.chi_square < 1.0); // Should be close to uniform
    }

    #[test]
    fn test_gini_coefficient() {
        // Uniform distribution should have low Gini coefficient
        let mut uniform_results = HashMap::new();
        uniform_results.insert("0".to_string(), 50);
        uniform_results.insert("1".to_string(), 50);

        let uniform_stats = MeasurementStatistics::from_results(&uniform_results);
        assert!(uniform_stats.gini_coefficient < 0.1);

        // Concentrated distribution should have high Gini coefficient
        let mut concentrated_results = HashMap::new();
        concentrated_results.insert("0".to_string(), 95);
        concentrated_results.insert("1".to_string(), 5);

        let concentrated_stats = MeasurementStatistics::from_results(&concentrated_results);
        assert!(concentrated_stats.gini_coefficient > 0.3); // Lower threshold for test
    }
}
