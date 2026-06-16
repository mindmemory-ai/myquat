//! Deoptimization - Reverse optimization for circuit extraction
//!
//! Author: gA4ss
//!
//! This module provides strategies to undo compiler optimizations, enabling
//! more accurate Hamiltonian extraction from optimized quantum circuits.
//!
//! # Overview
//!
//! Modern quantum compilers apply aggressive optimizations that transform
//! circuits to reduce gate count and depth. While beneficial for execution,
//! these optimizations make it difficult to extract the original algorithm
//! structure, particularly for Hamiltonian time evolution circuits.
//!
//! The deoptimization module implements three complementary strategies:
//!
//! 1. **KAK Restoration**: Recognizes optimized Pauli rotation patterns and
//!    restores them to their original form using KAK decomposition.
//!
//! 2. **Template Matching**: Identifies Trotter-Suzuki decomposition patterns
//!    using graph-based template matching algorithms.
//!
//! 3. **Temporal Analysis**: Infers time evolution parameters from rotation
//!    angles using Suzuki coefficient fingerprinting.
//!
//! # Architecture
//!
//! The module is organized around the [`DeoptStrategy`] trait, which defines
//! a common interface for all deoptimization strategies. The [`DeoptimizationPipeline`]
//! combines multiple strategies and applies them sequentially with confidence-based
//! early stopping.
//!
//! # Usage
//!
//! ## Basic Pipeline Usage
//!
//! ```rust
//! use myquat::circuit::QuantumCircuit;
//! use myquat::deoptimization::DeoptimizationPipeline;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut circuit = QuantumCircuit::new(2, 0);
//! // ... add gates to circuit ...
//!
//! // Use default pipeline with all strategies
//! let pipeline = DeoptimizationPipeline::default();
//! let (restored, confidence) = pipeline.restore(&circuit)?;
//!
//! println!("Restoration confidence: {:.2}%", confidence * 100.0);
//! # Ok(())
//! # }
//! ```
//!
//! ## Custom Pipeline Configuration
//!
//! ```rust
//! use myquat::deoptimization::{
//!     DeoptimizationPipeline,
//!     KakRestorationStrategy,
//!     TemplateMatchingStrategy,
//! };
//!
//! let pipeline = DeoptimizationPipeline::new()
//!     .add_strategy(Box::new(KakRestorationStrategy::new()))
//!     .add_strategy(Box::new(TemplateMatchingStrategy::default()))
//!     .with_threshold(0.90)  // High confidence threshold
//!     .with_early_stop(true)
//!     .with_max_iterations(10);
//! ```
//!
//! ## Detailed Results
//!
//! ```rust
//! # use myquat::circuit::QuantumCircuit;
//! # use myquat::deoptimization::DeoptimizationPipeline;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let circuit = QuantumCircuit::new(2, 0);
//! let pipeline = DeoptimizationPipeline::default();
//! let result = pipeline.restore_detailed(&circuit)?;
//!
//! println!("Overall confidence: {:.2}", result.confidence);
//! println!("Strategies applied: {}", result.strategies_applied);
//! println!("Early stopped: {}", result.early_stopped);
//!
//! for (name, conf) in result.strategy_confidences {
//!     println!("  {}: {:.2}", name, conf);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Analyzing Without Modification
//!
//! ```rust
//! # use myquat::circuit::QuantumCircuit;
//! # use myquat::deoptimization::DeoptimizationPipeline;
//! # let circuit = QuantumCircuit::new(2, 0);
//! let pipeline = DeoptimizationPipeline::default();
//! let analysis = pipeline.analyze(&circuit);
//!
//! for (strategy, confidence) in analysis {
//!     println!("{}: {:.2}%", strategy, confidence * 100.0);
//! }
//! ```
//!
//! # Strategies
//!
//! ## KAK Restoration
//!
//! Recognizes Pauli rotation patterns that have been optimized through KAK
//! decomposition. This strategy is most effective for circuits with single
//! and two-qubit Pauli rotations.
//!
//! **Use when**: Circuit contains Rx, Ry, Rz gates and CNOTs.
//!
//! ## Template Matching
//!
//! Uses graph-based pattern matching to identify Trotter-Suzuki decomposition
//! structures. Builds a graph representation of the circuit and matches it
//! against known Hamiltonian templates.
//!
//! **Use when**: Circuit follows Trotter decomposition patterns (XX, YY, ZZ interactions).
//!
//! ## Temporal Analysis
//!
//! Analyzes rotation angles to infer time evolution parameters. Uses Suzuki
//! coefficient fingerprinting to identify the decomposition order and time step.
//!
//! **Use when**: Circuit uses specific Suzuki coefficients (orders 4, 6, 8, 10).
//!
//! # Benchmarking
//!
//! The [`BenchmarkSuite`] provides comprehensive performance evaluation:
//!
//! ```rust
//! use myquat::deoptimization::{BenchmarkSuite, BenchmarkCircuit, OptimizationLevel};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut suite = BenchmarkSuite::new();
//!
//! // Generate test circuit
//! let circuit = BenchmarkSuite::generate_hamiltonian_simulation(4, 2);
//!
//! // Run benchmark
//! let result = suite.run_test(
//!     BenchmarkCircuit::HamiltonianSimulation,
//!     OptimizationLevel::Light,
//!     circuit,
//! )?;
//!
//! println!("Accuracy: {:.1}%", result.accuracy() * 100.0);
//! println!("Time: {} μs", result.restoration_time_us);
//! # Ok(())
//! # }
//! ```
//!
//! # Performance Considerations
//!
//! - **Early Stopping**: Enable for better performance when high confidence is achieved
//! - **Strategy Order**: Place faster strategies first (KAK → Template → Temporal)
//! - **Threshold Tuning**: Higher thresholds (0.9+) for critical applications
//! - **Max Iterations**: Limit iterations to prevent excessive computation
//!
//! # Examples
//!
//! See `examples/deoptimization_demo.rs` for comprehensive usage examples.

mod benchmark;
mod kak;
mod kak_math;
mod pauli_basis;
mod qdrift_strategy;
mod template;
mod temporal;
mod trotter_template;
mod vqe_templates;

pub use benchmark::{BenchmarkCircuit, BenchmarkResult, BenchmarkSuite, OptimizationLevel};
pub use kak::KakRestorationStrategy;
pub use pauli_basis::{PauliPatternMatcher, PauliRotation};
pub use qdrift_strategy::{
    detect_qdrift_pattern, QdriftDetection, QdriftRestorationStrategy, QdriftTerm,
};
pub use template::TemplateMatchingStrategy;
pub use temporal::{EvolutionParams, TemporalAnalysisStrategy};
pub use trotter_template::{HamiltonianTerm, TrotterStep, TrotterTemplate, TrotterTemplateBuilder};
pub use vqe_templates::{
    detect_vqe_pattern, AnsatzLayer, EntanglementPattern, HEATemplate, RotationType, UCCSDTemplate,
    VQEMatchResult, VQEType, VqeRestorationStrategy,
};

use crate::circuit::QuantumCircuit;
use crate::error::Result;

/// Trait for deoptimization strategies
///
/// Each strategy attempts to restore original circuit structure by
/// reversing specific optimization patterns.
pub trait DeoptStrategy: Send + Sync {
    /// Apply the deoptimization strategy to a circuit
    fn apply(&self, circuit: &QuantumCircuit) -> Result<QuantumCircuit>;

    /// Get the name of this strategy
    fn name(&self) -> &str;

    /// Get confidence score for this strategy on given circuit (0.0 to 1.0)
    fn confidence(&self, circuit: &QuantumCircuit) -> f64;
}

/// Restoration result with detailed diagnostic information
///
/// Contains the restored circuit along with confidence scores and
/// metadata about the restoration process.
///
/// # Fields
///
/// * `circuit` - The restored quantum circuit
/// * `confidence` - Overall confidence score in [0.0, 1.0]
/// * `strategy_confidences` - Per-strategy confidence scores
/// * `strategies_applied` - Number of strategies that were executed
/// * `early_stopped` - Whether early stopping was triggered
///
/// # Examples
///
/// ```rust
/// # use myquat::circuit::QuantumCircuit;
/// # use myquat::deoptimization::DeoptimizationPipeline;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let circuit = QuantumCircuit::new(2, 0);
/// let pipeline = DeoptimizationPipeline::default();
/// let result = pipeline.restore_detailed(&circuit)?;
///
/// if result.confidence > 0.8 {
///     println!("High confidence restoration!");
///     println!("Applied {} strategies", result.strategies_applied);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct RestorationResult {
    /// Restored circuit
    pub circuit: QuantumCircuit,
    /// Overall confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Strategy-specific confidences
    pub strategy_confidences: Vec<(String, f64)>,
    /// Number of strategies applied
    pub strategies_applied: usize,
    /// Whether early stopping was triggered
    pub early_stopped: bool,
}

/// Deoptimization pipeline for restoring original circuit structure
///
/// Combines multiple deoptimization strategies in a sequential pipeline
/// with confidence-based early stopping and iteration limits.
///
/// # Configuration
///
/// * `confidence_threshold` - Minimum confidence to trigger early stop (default: 0.85)
/// * `early_stop` - Whether to stop when threshold is reached (default: true)
/// * `max_iterations` - Maximum number of strategy applications (default: 10)
///
/// # Strategy Execution
///
/// Strategies are applied in the order they were added. Each strategy:
/// 1. Calculates confidence score for the current circuit
/// 2. If confidence > 0, applies transformation
/// 3. Updates cumulative confidence (weighted average)
/// 4. Checks early stop condition
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use myquat::deoptimization::DeoptimizationPipeline;
/// # use myquat::circuit::QuantumCircuit;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let circuit = QuantumCircuit::new(2, 0);
/// // Use default pipeline with all strategies
/// let pipeline = DeoptimizationPipeline::default();
/// let (restored, confidence) = pipeline.restore(&circuit)?;
/// # Ok(())
/// # }
/// ```
///
/// ## Custom Configuration
///
/// ```rust
/// use myquat::deoptimization::{
///     DeoptimizationPipeline,
///     KakRestorationStrategy,
///     TemplateMatchingStrategy,
/// };
///
/// let pipeline = DeoptimizationPipeline::new()
///     .add_strategy(Box::new(KakRestorationStrategy::new()))
///     .add_strategy(Box::new(TemplateMatchingStrategy::default()))
///     .with_threshold(0.90)
///     .with_early_stop(true)
///     .with_max_iterations(5);
/// ```
///
/// ## Analyzing Confidence
///
/// ```rust
/// # use myquat::circuit::QuantumCircuit;
/// # use myquat::deoptimization::DeoptimizationPipeline;
/// # let circuit = QuantumCircuit::new(2, 0);
/// let pipeline = DeoptimizationPipeline::default();
///
/// // Check confidence without modifying circuit
/// let scores = pipeline.analyze(&circuit);
/// for (strategy, conf) in scores {
///     println!("{}: {:.2}", strategy, conf);
/// }
/// ```
pub struct DeoptimizationPipeline {
    strategies: Vec<Box<dyn DeoptStrategy>>,
    confidence_threshold: f64,
    early_stop: bool,
    max_iterations: usize,
}

impl DeoptimizationPipeline {
    /// Create a new pipeline with default settings
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
            confidence_threshold: 0.85,
            early_stop: true,
            max_iterations: 10,
        }
    }

    /// Create a default pipeline with all strategies (including VQE)
    pub fn default_pipeline() -> Self {
        Self::new()
            .add_strategy(Box::new(KakRestorationStrategy::new()))
            .add_strategy(Box::new(TemplateMatchingStrategy::new()))
            .add_strategy(Box::new(TemporalAnalysisStrategy::new()))
            .add_strategy(Box::new(VqeRestorationStrategy::new()))
            .add_strategy(Box::new(QdriftRestorationStrategy::new()))
    }

    /// Add a strategy to the pipeline
    pub fn add_strategy(mut self, strategy: Box<dyn DeoptStrategy>) -> Self {
        self.strategies.push(strategy);
        self
    }

    /// Set confidence threshold for early stopping
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    /// Enable/disable early stopping
    pub fn with_early_stop(mut self, enabled: bool) -> Self {
        self.early_stop = enabled;
        self
    }

    /// Set maximum number of iterations
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Get number of strategies in pipeline
    pub fn num_strategies(&self) -> usize {
        self.strategies.len()
    }

    /// Restore original circuit structure from optimized circuit
    ///
    /// # Arguments
    /// * `circuit` - The optimized circuit to restore
    ///
    /// # Returns
    /// Restored circuit with confidence score
    pub fn restore(&self, circuit: &QuantumCircuit) -> Result<(QuantumCircuit, f64)> {
        let result = self.restore_detailed(circuit)?;
        Ok((result.circuit, result.confidence))
    }

    /// Restore with detailed result information
    pub fn restore_detailed(&self, circuit: &QuantumCircuit) -> Result<RestorationResult> {
        let mut current = circuit.clone();
        let mut strategy_confidences = Vec::new();
        let mut strategies_applied = 0;
        let mut early_stopped = false;

        // Apply strategies in sequence
        for (iteration, strategy) in self.strategies.iter().enumerate() {
            if iteration >= self.max_iterations {
                break;
            }

            let confidence = strategy.confidence(&current);
            let name = strategy.name().to_string();

            strategy_confidences.push((name.clone(), confidence));

            if confidence > 0.0 {
                current = strategy.apply(&current)?;
                strategies_applied += 1;

                // Calculate cumulative confidence
                let cumulative = self.calculate_cumulative_confidence(&strategy_confidences);

                // Early stop if confidence is high enough
                if self.early_stop && cumulative >= self.confidence_threshold {
                    early_stopped = true;
                    break;
                }
            }
        }

        let final_confidence = self.calculate_cumulative_confidence(&strategy_confidences);

        Ok(RestorationResult {
            circuit: current,
            confidence: final_confidence,
            strategy_confidences,
            strategies_applied,
            early_stopped,
        })
    }

    /// Calculate cumulative confidence from individual strategy confidences
    fn calculate_cumulative_confidence(&self, confidences: &[(String, f64)]) -> f64 {
        if confidences.is_empty() {
            return 0.0;
        }

        // Use weighted average, giving more weight to recent strategies
        let total: f64 = confidences
            .iter()
            .enumerate()
            .map(|(i, (_, conf))| {
                let weight = (i + 1) as f64 / confidences.len() as f64;
                conf * weight
            })
            .sum();

        let weight_sum: f64 = (1..=confidences.len())
            .map(|i| i as f64 / confidences.len() as f64)
            .sum();

        if weight_sum > 0.0 {
            total / weight_sum
        } else {
            0.0
        }
    }

    /// Analyze circuit without applying transformations
    ///
    /// Returns confidence scores for each strategy without modifying the circuit
    pub fn analyze(&self, circuit: &QuantumCircuit) -> Vec<(String, f64)> {
        self.strategies
            .iter()
            .map(|s| (s.name().to_string(), s.confidence(circuit)))
            .collect()
    }

    /// Check if pipeline is empty
    pub fn is_empty(&self) -> bool {
        self.strategies.is_empty()
    }
}

impl Default for DeoptimizationPipeline {
    fn default() -> Self {
        Self::new()
            .add_strategy(Box::new(KakRestorationStrategy::default()))
            .add_strategy(Box::new(TemplateMatchingStrategy::default()))
            .add_strategy(Box::new(TemporalAnalysisStrategy::default()))
            .add_strategy(Box::new(VqeRestorationStrategy::default()))
            .add_strategy(Box::new(QdriftRestorationStrategy::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parameter::Parameter;
    use std::f64::consts::PI;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = DeoptimizationPipeline::new();
        assert_eq!(pipeline.num_strategies(), 0);
        assert!(pipeline.is_empty());
        assert_eq!(pipeline.confidence_threshold, 0.85);
        assert!(pipeline.early_stop);
        assert_eq!(pipeline.max_iterations, 10);
    }

    #[test]
    fn test_pipeline_with_strategies() {
        let pipeline = DeoptimizationPipeline::default();
        assert_eq!(pipeline.num_strategies(), 5);
        assert!(!pipeline.is_empty());
    }

    #[test]
    fn test_default_pipeline() {
        let pipeline = DeoptimizationPipeline::default_pipeline();
        assert_eq!(pipeline.num_strategies(), 5);

        // Should have KAK, Template, Temporal, VQE and qDRIFT strategies
        let circuit = QuantumCircuit::new(2, 0);
        let analysis = pipeline.analyze(&circuit);
        assert_eq!(analysis.len(), 5);
    }

    #[test]
    fn test_confidence_threshold() {
        let pipeline = DeoptimizationPipeline::new().with_threshold(0.9);
        assert_eq!(pipeline.confidence_threshold, 0.9);
    }

    #[test]
    fn test_early_stop() {
        let pipeline = DeoptimizationPipeline::new().with_early_stop(false);
        assert!(!pipeline.early_stop);
    }

    #[test]
    fn test_max_iterations() {
        let pipeline = DeoptimizationPipeline::new().with_max_iterations(20);
        assert_eq!(pipeline.max_iterations, 20);
    }

    #[test]
    fn test_add_strategy() {
        let pipeline =
            DeoptimizationPipeline::new().add_strategy(Box::new(KakRestorationStrategy::new()));
        assert_eq!(pipeline.num_strategies(), 1);
    }

    #[test]
    fn test_restore_empty_circuit() {
        let pipeline = DeoptimizationPipeline::default();
        let circuit = QuantumCircuit::new(2, 0);

        let result = pipeline.restore(&circuit);
        assert!(result.is_ok());

        let (restored, confidence) = result.unwrap();
        assert_eq!(restored.num_qubits(), 2);
        assert!(confidence >= 0.0 && confidence <= 1.0);
    }

    #[test]
    fn test_restore_detailed_empty() {
        let pipeline = DeoptimizationPipeline::default();
        let circuit = QuantumCircuit::new(2, 0);

        let result = pipeline.restore_detailed(&circuit);
        assert!(result.is_ok());

        let res = result.unwrap();
        assert_eq!(res.circuit.num_qubits(), 2);
        assert_eq!(res.strategy_confidences.len(), 5);
        assert!(res.confidence >= 0.0);
        assert!(res.strategies_applied <= 5);
    }

    #[test]
    fn test_restore_with_gates() {
        let pipeline = DeoptimizationPipeline::default();
        let mut circuit = QuantumCircuit::new(2, 0);

        // Add some gates
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.rz(1, Parameter::Float(PI / 4.0)).unwrap();

        let result = pipeline.restore_detailed(&circuit);
        assert!(result.is_ok());

        let res = result.unwrap();
        assert!(res.confidence >= 0.0);
        assert_eq!(res.strategy_confidences.len(), 5);
    }

    #[test]
    fn test_analyze() {
        let pipeline = DeoptimizationPipeline::default();
        let mut circuit = QuantumCircuit::new(2, 0);

        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();

        let analysis = pipeline.analyze(&circuit);
        assert_eq!(analysis.len(), 5);

        // All strategies should return some confidence
        for (name, conf) in &analysis {
            assert!(!name.is_empty());
            assert!(conf >= &0.0 && conf <= &1.0);
        }
    }

    #[test]
    fn test_calculate_cumulative_confidence() {
        let pipeline = DeoptimizationPipeline::new();

        // Test with no confidences
        let confidences = vec![];
        assert_eq!(pipeline.calculate_cumulative_confidence(&confidences), 0.0);

        // Test with single confidence
        let confidences = vec![("Strategy1".to_string(), 0.8)];
        let cum = pipeline.calculate_cumulative_confidence(&confidences);
        assert!(cum > 0.0);

        // Test with multiple confidences
        let confidences = vec![
            ("Strategy1".to_string(), 0.6),
            ("Strategy2".to_string(), 0.8),
            ("Strategy3".to_string(), 0.9),
        ];
        let cum = pipeline.calculate_cumulative_confidence(&confidences);
        assert!(cum >= 0.6 && cum <= 0.9); // Should be weighted average
    }

    #[test]
    fn test_restoration_result() {
        let circuit = QuantumCircuit::new(2, 0);
        let result = RestorationResult {
            circuit: circuit.clone(),
            confidence: 0.85,
            strategy_confidences: vec![
                ("KAK Restoration".to_string(), 0.8),
                ("Template Matching".to_string(), 0.9),
            ],
            strategies_applied: 2,
            early_stopped: true,
        };

        assert_eq!(result.confidence, 0.85);
        assert_eq!(result.strategies_applied, 2);
        assert!(result.early_stopped);
        assert_eq!(result.strategy_confidences.len(), 2);
    }
}
