//! Performance regression detection system
//!
//! This module provides automated detection of performance regressions by comparing
//! current benchmark results with historical baselines.

use crate::benchmarks::BenchmarkResult;
use crate::error::{MyQuatError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Regression detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionConfig {
    /// Threshold for performance regression (e.g., 1.2 = 20% slower)
    pub regression_threshold: f64,
    /// Threshold for performance improvement (e.g., 0.8 = 20% faster)
    pub improvement_threshold: f64,
    /// Minimum number of samples required for comparison
    pub min_samples: usize,
    /// Statistical confidence level (0.0 to 1.0)
    pub confidence_level: f64,
    /// Whether to use relative or absolute thresholds
    pub use_relative_threshold: bool,
}

impl Default for RegressionConfig {
    fn default() -> Self {
        RegressionConfig {
            regression_threshold: 1.2,  // 20% slower is a regression
            improvement_threshold: 0.8, // 20% faster is an improvement
            min_samples: 5,
            confidence_level: 0.95,
            use_relative_threshold: true,
        }
    }
}

/// Historical performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceHistory {
    /// Benchmark name
    pub benchmark_name: String,
    /// Historical results (most recent first)
    pub results: Vec<HistoricalResult>,
    /// Statistical baseline
    pub baseline: PerformanceBaseline,
}

/// Single historical result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalResult {
    /// Timestamp of the benchmark run
    pub timestamp: String,
    /// Git commit hash (if available)
    pub commit_hash: Option<String>,
    /// Benchmark result
    pub result: BenchmarkResult,
    /// Environment information
    pub environment: EnvironmentInfo,
}

/// Performance baseline for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    /// Mean execution time in nanoseconds
    pub mean_time_ns: f64,
    /// Standard deviation in nanoseconds
    pub std_dev_ns: f64,
    /// Number of samples used to calculate baseline
    pub sample_count: usize,
    /// Confidence interval bounds
    pub confidence_interval: (f64, f64),
    /// Last updated timestamp
    pub last_updated: String,
}

/// Environment information for benchmark runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    /// Operating system
    pub os: String,
    /// CPU model
    pub cpu: String,
    /// Available memory in GB
    pub memory_gb: f64,
    /// Rust version
    pub rust_version: String,
    /// Compiler flags
    pub compiler_flags: Vec<String>,
}

impl Default for EnvironmentInfo {
    fn default() -> Self {
        EnvironmentInfo {
            os: std::env::consts::OS.to_string(),
            cpu: "Unknown".to_string(),
            memory_gb: 0.0,
            rust_version: "Unknown".to_string(),
            compiler_flags: vec![],
        }
    }
}

/// Regression detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionResult {
    /// Benchmark name
    pub benchmark_name: String,
    /// Current performance
    pub current_time_ns: u64,
    /// Baseline performance
    pub baseline_time_ns: f64,
    /// Performance change ratio (current/baseline)
    pub change_ratio: f64,
    /// Type of change detected
    pub change_type: ChangeType,
    /// Statistical significance
    pub is_significant: bool,
    /// Confidence level of the detection
    pub confidence: f64,
    /// Detailed analysis
    pub analysis: String,
}

/// Type of performance change
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Performance regression (slower)
    Regression,
    /// Performance improvement (faster)
    Improvement,
    /// No significant change
    NoChange,
    /// Insufficient data for comparison
    InsufficientData,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::Regression => write!(f, "REGRESSION"),
            ChangeType::Improvement => write!(f, "IMPROVEMENT"),
            ChangeType::NoChange => write!(f, "NO_CHANGE"),
            ChangeType::InsufficientData => write!(f, "INSUFFICIENT_DATA"),
        }
    }
}

/// Performance regression detector
pub struct RegressionDetector {
    config: RegressionConfig,
    history: HashMap<String, PerformanceHistory>,
    history_file: String,
}

impl RegressionDetector {
    /// Create a new regression detector
    pub fn new(config: RegressionConfig, history_file: String) -> Self {
        RegressionDetector {
            config,
            history: HashMap::new(),
            history_file,
        }
    }

    /// Create detector with default configuration
    pub fn with_history_file(history_file: String) -> Self {
        Self::new(RegressionConfig::default(), history_file)
    }

    /// Load historical performance data
    pub fn load_history(&mut self) -> Result<()> {
        if Path::new(&self.history_file).exists() {
            let content = std::fs::read_to_string(&self.history_file).map_err(|e| {
                MyQuatError::circuit_error(format!("Failed to read history file: {}", e))
            })?;

            self.history = serde_json::from_str(&content).map_err(|e| {
                MyQuatError::circuit_error(format!("Failed to parse history: {}", e))
            })?;
        }
        Ok(())
    }

    /// Save historical performance data
    pub fn save_history(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.history).map_err(|e| {
            MyQuatError::circuit_error(format!("Failed to serialize history: {}", e))
        })?;

        std::fs::write(&self.history_file, content).map_err(|e| {
            MyQuatError::circuit_error(format!("Failed to write history file: {}", e))
        })?;

        Ok(())
    }

    /// Add new benchmark results to history
    pub fn add_results(
        &mut self,
        results: &[BenchmarkResult],
        commit_hash: Option<String>,
    ) -> Result<()> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let environment = self.get_environment_info();

        for result in results {
            let historical_result = HistoricalResult {
                timestamp: timestamp.clone(),
                commit_hash: commit_hash.clone(),
                result: result.clone(),
                environment: environment.clone(),
            };

            let history =
                self.history
                    .entry(result.name.clone())
                    .or_insert_with(|| PerformanceHistory {
                        benchmark_name: result.name.clone(),
                        results: Vec::new(),
                        baseline: PerformanceBaseline {
                            mean_time_ns: result.avg_time_ns as f64,
                            std_dev_ns: result.std_dev_ns,
                            sample_count: 1,
                            confidence_interval: (
                                result.avg_time_ns as f64,
                                result.avg_time_ns as f64,
                            ),
                            last_updated: timestamp.clone(),
                        },
                    });

            // Add new result
            history.results.insert(0, historical_result);

            // Keep only recent results (last 100)
            if history.results.len() > 100 {
                history.results.truncate(100);
            }

            // Update baseline
            let results_clone = history.results.clone();
            Self::update_baseline_static(&mut history.baseline, &results_clone)?;
        }

        Ok(())
    }

    /// Detect regressions in current results compared to baseline
    pub fn detect_regressions(&self, current_results: &[BenchmarkResult]) -> Vec<RegressionResult> {
        let mut regression_results = Vec::new();

        for result in current_results {
            if let Some(history) = self.history.get(&result.name) {
                let regression_result = self.analyze_performance_change(result, history);
                regression_results.push(regression_result);
            } else {
                // No historical data available
                regression_results.push(RegressionResult {
                    benchmark_name: result.name.clone(),
                    current_time_ns: result.avg_time_ns,
                    baseline_time_ns: result.avg_time_ns as f64,
                    change_ratio: 1.0,
                    change_type: ChangeType::InsufficientData,
                    is_significant: false,
                    confidence: 0.0,
                    analysis: "No historical data available for comparison".to_string(),
                });
            }
        }

        regression_results
    }

    /// Analyze performance change for a single benchmark
    fn analyze_performance_change(
        &self,
        current: &BenchmarkResult,
        history: &PerformanceHistory,
    ) -> RegressionResult {
        let baseline = &history.baseline;

        if baseline.sample_count < self.config.min_samples {
            return RegressionResult {
                benchmark_name: current.name.clone(),
                current_time_ns: current.avg_time_ns,
                baseline_time_ns: baseline.mean_time_ns,
                change_ratio: current.avg_time_ns as f64 / baseline.mean_time_ns,
                change_type: ChangeType::InsufficientData,
                is_significant: false,
                confidence: 0.0,
                analysis: format!(
                    "Insufficient historical data ({} samples, need {})",
                    baseline.sample_count, self.config.min_samples
                ),
            };
        }

        let change_ratio = current.avg_time_ns as f64 / baseline.mean_time_ns;

        // Determine change type
        let change_type = if change_ratio >= self.config.regression_threshold {
            ChangeType::Regression
        } else if change_ratio <= self.config.improvement_threshold {
            ChangeType::Improvement
        } else {
            ChangeType::NoChange
        };

        // Statistical significance test (simplified t-test)
        let is_significant = self.is_statistically_significant(
            current.avg_time_ns as f64,
            current.std_dev_ns,
            baseline.mean_time_ns,
            baseline.std_dev_ns,
            baseline.sample_count,
        );

        // Calculate confidence
        let confidence = if is_significant {
            self.config.confidence_level
        } else {
            0.5 // Low confidence if not significant
        };

        // Generate analysis
        let analysis = self.generate_analysis(current, baseline, change_ratio, &change_type);

        RegressionResult {
            benchmark_name: current.name.clone(),
            current_time_ns: current.avg_time_ns,
            baseline_time_ns: baseline.mean_time_ns,
            change_ratio,
            change_type,
            is_significant,
            confidence,
            analysis,
        }
    }

    /// Update performance baseline with new data (static version)
    fn update_baseline_static(
        baseline: &mut PerformanceBaseline,
        results: &[HistoricalResult],
    ) -> Result<()> {
        if results.is_empty() {
            return Ok(());
        }

        // Use recent results for baseline (last 20)
        let recent_results: Vec<_> = results.iter().take(20).collect();
        let times: Vec<f64> = recent_results
            .iter()
            .map(|r| r.result.avg_time_ns as f64)
            .collect();

        // Calculate statistics
        let mean = times.iter().sum::<f64>() / times.len() as f64;
        let variance = times.iter().map(|&t| (t - mean).powi(2)).sum::<f64>() / times.len() as f64;
        let std_dev = variance.sqrt();

        // Calculate confidence interval (assuming normal distribution)
        let t_value = 1.96; // 95% confidence for large samples
        let margin = t_value * std_dev / (times.len() as f64).sqrt();

        baseline.mean_time_ns = mean;
        baseline.std_dev_ns = std_dev;
        baseline.sample_count = times.len();
        baseline.confidence_interval = (mean - margin, mean + margin);
        baseline.last_updated = chrono::Utc::now().to_rfc3339();

        Ok(())
    }

    /// Check if performance change is statistically significant
    fn is_statistically_significant(
        &self,
        current_mean: f64,
        current_std: f64,
        baseline_mean: f64,
        baseline_std: f64,
        baseline_samples: usize,
    ) -> bool {
        // Simplified statistical test
        let pooled_std = ((current_std.powi(2) + baseline_std.powi(2)) / 2.0).sqrt();
        let standard_error = pooled_std * (1.0 + 1.0 / baseline_samples as f64).sqrt();

        if standard_error == 0.0 {
            return false;
        }

        let t_statistic = (current_mean - baseline_mean).abs() / standard_error;
        let critical_value = 1.96; // 95% confidence

        t_statistic > critical_value
    }

    /// Generate detailed analysis text
    fn generate_analysis(
        &self,
        current: &BenchmarkResult,
        baseline: &PerformanceBaseline,
        change_ratio: f64,
        change_type: &ChangeType,
    ) -> String {
        let mut analysis = String::new();

        let percent_change = (change_ratio - 1.0) * 100.0;

        match change_type {
            ChangeType::Regression => {
                analysis.push_str(&format!(
                    "Performance regression detected: {:.1}% slower than baseline. ",
                    percent_change
                ));
                analysis.push_str(&format!(
                    "Current: {:.2} μs, Baseline: {:.2} μs. ",
                    current.avg_time_us(),
                    baseline.mean_time_ns / 1000.0
                ));
            }
            ChangeType::Improvement => {
                analysis.push_str(&format!(
                    "Performance improvement detected: {:.1}% faster than baseline. ",
                    -percent_change
                ));
                analysis.push_str(&format!(
                    "Current: {:.2} μs, Baseline: {:.2} μs. ",
                    current.avg_time_us(),
                    baseline.mean_time_ns / 1000.0
                ));
            }
            ChangeType::NoChange => {
                analysis.push_str(&format!(
                    "Performance is stable: {:.1}% change from baseline. ",
                    percent_change
                ));
            }
            ChangeType::InsufficientData => {
                analysis.push_str("Insufficient historical data for reliable comparison. ");
            }
        }

        analysis.push_str(&format!(
            "Baseline based on {} samples with std dev {:.2} μs.",
            baseline.sample_count,
            baseline.std_dev_ns / 1000.0
        ));

        analysis
    }

    /// Get current environment information
    fn get_environment_info(&self) -> EnvironmentInfo {
        EnvironmentInfo {
            os: std::env::consts::OS.to_string(),
            cpu: self.get_cpu_info(),
            memory_gb: self.get_memory_info(),
            rust_version: self.get_rust_version(),
            compiler_flags: vec!["opt-level=3".to_string()], // Default for release builds
        }
    }

    fn get_cpu_info(&self) -> String {
        // Simplified CPU detection
        if cfg!(target_arch = "x86_64") {
            "x86_64".to_string()
        } else if cfg!(target_arch = "aarch64") {
            "aarch64".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    fn get_memory_info(&self) -> f64 {
        // Simplified memory detection (would need system-specific code)
        8.0 // Default assumption
    }

    fn get_rust_version(&self) -> String {
        std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "Unknown".to_string())
    }

    /// Generate regression report
    pub fn generate_report(&self, regression_results: &[RegressionResult]) -> String {
        let mut report = String::new();

        report.push_str("# Performance Regression Report\n\n");

        // Summary
        let regressions: Vec<_> = regression_results
            .iter()
            .filter(|r| r.change_type == ChangeType::Regression && r.is_significant)
            .collect();
        let improvements: Vec<_> = regression_results
            .iter()
            .filter(|r| r.change_type == ChangeType::Improvement && r.is_significant)
            .collect();

        report.push_str("## Summary\n");
        report.push_str(&format!(
            "- Total benchmarks analyzed: {}\n",
            regression_results.len()
        ));
        report.push_str(&format!(
            "- Significant regressions: {}\n",
            regressions.len()
        ));
        report.push_str(&format!(
            "- Significant improvements: {}\n",
            improvements.len()
        ));
        report.push('\n');

        // Regressions
        if !regressions.is_empty() {
            report.push_str("## 🚨 Performance Regressions\n\n");
            for regression in regressions {
                report.push_str(&format!(
                    "### {}\n- **Change**: {:.1}% slower\n- **Current**: {:.2} μs\n- **Baseline**: {:.2} μs\n- **Analysis**: {}\n\n",
                    regression.benchmark_name,
                    (regression.change_ratio - 1.0) * 100.0,
                    regression.current_time_ns as f64 / 1000.0,
                    regression.baseline_time_ns / 1000.0,
                    regression.analysis
                ));
            }
        }

        // Improvements
        if !improvements.is_empty() {
            report.push_str("## ✅ Performance Improvements\n\n");
            for improvement in improvements {
                report.push_str(&format!(
                    "### {}\n- **Change**: {:.1}% faster\n- **Current**: {:.2} μs\n- **Baseline**: {:.2} μs\n- **Analysis**: {}\n\n",
                    improvement.benchmark_name,
                    (1.0 - improvement.change_ratio) * 100.0,
                    improvement.current_time_ns as f64 / 1000.0,
                    improvement.baseline_time_ns / 1000.0,
                    improvement.analysis
                ));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_regression_config_default() {
        let config = RegressionConfig::default();
        assert_eq!(config.regression_threshold, 1.2);
        assert_eq!(config.improvement_threshold, 0.8);
        assert!(config.use_relative_threshold);
    }

    #[test]
    fn test_change_type_display() {
        assert_eq!(ChangeType::Regression.to_string(), "REGRESSION");
        assert_eq!(ChangeType::Improvement.to_string(), "IMPROVEMENT");
        assert_eq!(ChangeType::NoChange.to_string(), "NO_CHANGE");
    }

    #[test]
    fn test_regression_detector_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let detector =
            RegressionDetector::with_history_file(temp_file.path().to_string_lossy().to_string());

        assert_eq!(detector.config.regression_threshold, 1.2);
        assert!(detector.history.is_empty());
    }

    #[test]
    fn test_environment_info_default() {
        let env = EnvironmentInfo::default();
        assert_eq!(env.os, std::env::consts::OS);
        assert_eq!(env.cpu, "Unknown");
    }
}
