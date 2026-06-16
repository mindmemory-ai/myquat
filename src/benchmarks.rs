//! Benchmark suite for performance testing and regression detection
//!
//! This module provides comprehensive benchmarking capabilities for the MyQuat library,
//! including circuit construction, gate operations, simulation, and optimization performance.

use crate::algorithms::{GroverSearch, VQEAnsatz, QFT};
use crate::circuit_optimizer::{CircuitOptimizer, OptimizationConfig};
use crate::error::{MyQuatError, Result};
use crate::hardware_constraints::HardwareValidator;
use crate::qasm::QasmExporter;
use crate::{Parameter, QuantumCircuit, StandardGate, StateVectorSimulator};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of iterations for each benchmark
    pub iterations: usize,
    /// Warmup iterations before actual measurement
    pub warmup_iterations: usize,
    /// Maximum allowed time per benchmark in seconds
    pub max_time_seconds: u64,
    /// Whether to include memory usage measurements
    pub measure_memory: bool,
    /// Qubit range for scalability tests
    pub qubit_range: Vec<usize>,
    /// Circuit depth range for complexity tests
    pub depth_range: Vec<usize>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        BenchmarkConfig {
            iterations: 100,
            warmup_iterations: 10,
            max_time_seconds: 60,
            measure_memory: false,
            qubit_range: vec![2, 4, 6, 8, 10, 12, 16, 20],
            depth_range: vec![10, 50, 100, 200, 500],
        }
    }
}

/// Benchmark result for a single test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Name of the benchmark
    pub name: String,
    /// Average execution time in nanoseconds
    pub avg_time_ns: u64,
    /// Minimum execution time in nanoseconds
    pub min_time_ns: u64,
    /// Maximum execution time in nanoseconds
    pub max_time_ns: u64,
    /// Standard deviation of execution times
    pub std_dev_ns: f64,
    /// Throughput (operations per second)
    pub throughput: f64,
    /// Memory usage in bytes (if measured)
    pub memory_bytes: Option<usize>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl BenchmarkResult {
    /// Create a new benchmark result from timing measurements
    pub fn from_timings(name: String, timings: &[Duration]) -> Self {
        if timings.is_empty() {
            return BenchmarkResult {
                name,
                avg_time_ns: 0,
                min_time_ns: 0,
                max_time_ns: 0,
                std_dev_ns: 0.0,
                throughput: 0.0,
                memory_bytes: None,
                metadata: HashMap::new(),
            };
        }

        let times_ns: Vec<u64> = timings.iter().map(|d| d.as_nanos() as u64).collect();

        let avg_time_ns = times_ns.iter().sum::<u64>() / times_ns.len() as u64;
        let min_time_ns = *times_ns.iter().min().unwrap();
        let max_time_ns = *times_ns.iter().max().unwrap();

        // Calculate standard deviation
        let variance = times_ns
            .iter()
            .map(|&t| {
                let diff = t as f64 - avg_time_ns as f64;
                diff * diff
            })
            .sum::<f64>()
            / times_ns.len() as f64;
        let std_dev_ns = variance.sqrt();

        // Calculate throughput (operations per second)
        let throughput = if avg_time_ns > 0 {
            1_000_000_000.0 / avg_time_ns as f64
        } else {
            0.0
        };

        BenchmarkResult {
            name,
            avg_time_ns,
            min_time_ns,
            max_time_ns,
            std_dev_ns,
            throughput,
            memory_bytes: None,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the benchmark result
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Get execution time in milliseconds
    pub fn avg_time_ms(&self) -> f64 {
        self.avg_time_ns as f64 / 1_000_000.0
    }

    /// Get execution time in microseconds
    pub fn avg_time_us(&self) -> f64 {
        self.avg_time_ns as f64 / 1_000.0
    }
}

/// Comprehensive benchmark suite
pub struct BenchmarkSuite {
    config: BenchmarkConfig,
    results: Vec<BenchmarkResult>,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite with default configuration
    pub fn new() -> Self {
        BenchmarkSuite {
            config: BenchmarkConfig::default(),
            results: Vec::new(),
        }
    }

    /// Create a new benchmark suite with custom configuration
    pub fn with_config(config: BenchmarkConfig) -> Self {
        BenchmarkSuite {
            config,
            results: Vec::new(),
        }
    }

    /// Run all benchmarks
    pub fn run_all(&mut self) -> Result<()> {
        println!("🚀 Starting MyQuat Benchmark Suite");
        println!(
            "Configuration: {} iterations, {} warmup",
            self.config.iterations, self.config.warmup_iterations
        );
        println!();

        // Circuit construction benchmarks
        self.run_circuit_construction_benchmarks()?;

        // Gate operation benchmarks
        self.run_gate_operation_benchmarks()?;

        // Simulation benchmarks
        self.run_simulation_benchmarks()?;

        // Algorithm benchmarks
        self.run_algorithm_benchmarks()?;

        // Optimization benchmarks
        self.run_optimization_benchmarks()?;

        // Backend benchmarks
        self.run_backend_benchmarks()?;

        // Scalability benchmarks
        self.run_scalability_benchmarks()?;

        println!("✅ Benchmark suite completed!");
        Ok(())
    }

    /// Benchmark circuit construction operations
    pub fn run_circuit_construction_benchmarks(&mut self) -> Result<()> {
        println!("📋 Circuit Construction Benchmarks");
        println!("----------------------------------");

        // Circuit creation benchmark
        let result = self.benchmark("circuit_creation", || {
            let _circuit = QuantumCircuit::new(10, 10);
        })?;
        println!("Circuit Creation: {:.2} μs", result.avg_time_us());

        // Gate addition benchmark
        let result = self.benchmark("gate_addition", || {
            let mut circuit = QuantumCircuit::new(4, 0);
            circuit.h(0).unwrap();
            circuit.cx(0, 1).unwrap();
            circuit.ry(2, Parameter::new_float(1.57)).unwrap();
            circuit.cz(1, 3).unwrap();
        })?;
        println!("Gate Addition (4 gates): {:.2} μs", result.avg_time_us());

        // Large circuit construction
        let result = self.benchmark("large_circuit_construction", || {
            let mut circuit = QuantumCircuit::new(20, 20);
            for i in 0..20 {
                circuit.h(i).unwrap();
            }
            for i in 0..19 {
                circuit.cx(i, i + 1).unwrap();
            }
        })?;
        println!("Large Circuit (40 gates): {:.2} μs", result.avg_time_us());

        println!();
        Ok(())
    }

    /// Benchmark gate operations
    pub fn run_gate_operation_benchmarks(&mut self) -> Result<()> {
        println!("🚪 Gate Operation Benchmarks");
        println!("----------------------------");

        let gates = vec![
            ("H", StandardGate::H),
            ("X", StandardGate::X),
            ("Y", StandardGate::Y),
            ("Z", StandardGate::Z),
            ("S", StandardGate::S),
            ("T", StandardGate::T),
            ("CX", StandardGate::CX),
            ("CZ", StandardGate::CZ),
        ];

        for (name, gate) in gates {
            let qubits_needed = match gate {
                StandardGate::CX | StandardGate::CZ => 2,
                _ => 1,
            };

            let result = self.benchmark(&format!("gate_{}", name.to_lowercase()), || {
                let mut circuit = QuantumCircuit::new(qubits_needed, 0);
                match gate {
                    StandardGate::H => circuit.h(0).unwrap(),
                    StandardGate::X => circuit.x(0).unwrap(),
                    StandardGate::Y => circuit.y(0).unwrap(),
                    StandardGate::Z => circuit.z(0).unwrap(),
                    StandardGate::S => circuit.s(0).unwrap(),
                    StandardGate::T => circuit.t(0).unwrap(),
                    StandardGate::CX => circuit.cx(0, 1).unwrap(),
                    StandardGate::CZ => circuit.cz(0, 1).unwrap(),
                    _ => {}
                }
            })?;
            println!("{} Gate: {:.2} μs", name, result.avg_time_us());
        }

        println!();
        Ok(())
    }

    /// Benchmark simulation operations
    pub fn run_simulation_benchmarks(&mut self) -> Result<()> {
        println!("🔬 Simulation Benchmarks");
        println!("------------------------");

        // Small circuit simulation
        let result = self.benchmark("simulation_small", || {
            let mut circuit = QuantumCircuit::new(4, 0);
            circuit.h(0).unwrap();
            circuit.cx(0, 1).unwrap();
            circuit.cx(1, 2).unwrap();
            circuit.cx(2, 3).unwrap();

            let mut simulator = StateVectorSimulator::new(4, 0);
            simulator.execute_circuit(&circuit).unwrap();
        })?;
        println!("Small Circuit (4 qubits): {:.2} μs", result.avg_time_us());

        // Medium circuit simulation
        let result = self.benchmark("simulation_medium", || {
            let mut circuit = QuantumCircuit::new(8, 0);
            for i in 0..8 {
                circuit.h(i).unwrap();
            }
            for i in 0..7 {
                circuit.cx(i, i + 1).unwrap();
            }

            let mut simulator = StateVectorSimulator::new(8, 0);
            simulator.execute_circuit(&circuit).unwrap();
        })?;
        println!("Medium Circuit (8 qubits): {:.2} μs", result.avg_time_us());

        println!();
        Ok(())
    }

    /// Benchmark quantum algorithms
    fn run_algorithm_benchmarks(&mut self) -> Result<()> {
        println!("🧮 Algorithm Benchmarks");
        println!("-----------------------");

        // QFT benchmark
        let result = self.benchmark("qft_4_qubits", || {
            let qft = QFT::new(4);
            let _circuit = qft.build_circuit().unwrap();
        })?;
        println!("QFT (4 qubits): {:.2} μs", result.avg_time_us());

        // Grover benchmark
        let result = self.benchmark("grover_3_qubits", || {
            let grover = GroverSearch::new(3, vec![5]).unwrap(); // Search for |101⟩
            let _circuit = grover.build_circuit().unwrap();
        })?;
        println!("Grover (3 qubits): {:.2} μs", result.avg_time_us());

        // VQE ansatz benchmark
        let result = self.benchmark("vqe_ansatz_4_qubits", || {
            let vqe = VQEAnsatz::new(4, 2);
            let params = vqe.random_parameters();
            let _circuit = vqe.build_hardware_efficient_ansatz(&params).unwrap();
        })?;
        println!("VQE Ansatz (4 qubits): {:.2} μs", result.avg_time_us());

        println!();
        Ok(())
    }

    /// Benchmark optimization operations
    fn run_optimization_benchmarks(&mut self) -> Result<()> {
        println!("⚡ Optimization Benchmarks");
        println!("--------------------------");

        // Circuit optimization benchmark
        let result = self.benchmark("circuit_optimization", || {
            let mut circuit = QuantumCircuit::new(4, 0);
            circuit.h(0).unwrap();
            circuit.x(0).unwrap();
            circuit.x(0).unwrap(); // Should be optimized away
            circuit.cx(0, 1).unwrap();
            circuit.h(1).unwrap();
            circuit.h(1).unwrap(); // Should be optimized away

            let mut optimizer = CircuitOptimizer::new(OptimizationConfig::default());
            let _optimized = optimizer.optimize(&circuit).unwrap();
        })?;
        println!("Circuit Optimization: {:.2} μs", result.avg_time_us());

        println!();
        Ok(())
    }

    /// Benchmark backend operations
    fn run_backend_benchmarks(&mut self) -> Result<()> {
        println!("🔧 Backend Benchmarks");
        println!("---------------------");

        // QASM export benchmark
        let result = self.benchmark("qasm_export", || {
            let mut circuit = QuantumCircuit::new(4, 4);
            circuit.h(0).unwrap();
            circuit.cx(0, 1).unwrap();
            circuit.cx(1, 2).unwrap();
            circuit.measure_all().unwrap();

            let exporter = QasmExporter::new();
            let _qasm = exporter.export(&circuit).unwrap();
        })?;
        println!("QASM Export: {:.2} μs", result.avg_time_us());

        // Hardware validation benchmark
        let result = self.benchmark("hardware_validation", || {
            let mut circuit = QuantumCircuit::new(4, 0);
            circuit.h(0).unwrap();
            circuit.cx(0, 1).unwrap();
            circuit.cx(1, 2).unwrap();

            let topology = crate::HardwareTopology::linear(4);
            let validator = HardwareValidator::with_topology(topology);
            let _result = validator.validate(&circuit);
        })?;
        println!("Hardware Validation: {:.2} μs", result.avg_time_us());

        println!();
        Ok(())
    }

    /// Benchmark scalability with different qubit counts
    pub fn run_scalability_benchmarks(&mut self) -> Result<()> {
        println!("📈 Scalability Benchmarks");
        println!("-------------------------");

        let qubit_range = self.config.qubit_range.clone();
        for &num_qubits in &qubit_range {
            if num_qubits > 16 {
                continue; // Skip large circuits for now
            }

            let result = self.benchmark(&format!("scalability_{}_qubits", num_qubits), || {
                let mut circuit = QuantumCircuit::new(num_qubits, 0);
                for i in 0..num_qubits {
                    circuit.h(i).unwrap();
                }
                for i in 0..num_qubits - 1 {
                    circuit.cx(i, i + 1).unwrap();
                }

                let mut simulator = StateVectorSimulator::new(num_qubits, 0);
                simulator.execute_circuit(&circuit).unwrap();
            })?;

            println!("{} qubits: {:.2} ms", num_qubits, result.avg_time_ms());
        }

        println!();
        Ok(())
    }

    /// Run a single benchmark
    fn benchmark<F>(&mut self, name: &str, mut operation: F) -> Result<BenchmarkResult>
    where
        F: FnMut(),
    {
        // Warmup
        for _ in 0..self.config.warmup_iterations {
            operation();
        }

        // Actual measurement
        let mut timings = Vec::new();
        let start_time = Instant::now();

        for _ in 0..self.config.iterations {
            let iter_start = Instant::now();
            operation();
            let iter_duration = iter_start.elapsed();
            timings.push(iter_duration);

            // Check timeout
            if start_time.elapsed().as_secs() > self.config.max_time_seconds {
                break;
            }
        }

        let result = BenchmarkResult::from_timings(name.to_string(), &timings);
        self.results.push(result.clone());
        Ok(result)
    }

    /// Get all benchmark results
    pub fn results(&self) -> &[BenchmarkResult] {
        &self.results
    }

    /// Generate a summary report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("# MyQuat Benchmark Report\n\n");

        // Summary statistics
        let total_benchmarks = self.results.len();
        let avg_time_ms: f64 =
            self.results.iter().map(|r| r.avg_time_ms()).sum::<f64>() / total_benchmarks as f64;

        report.push_str("## Summary\n");
        report.push_str(&format!("- Total benchmarks: {}\n", total_benchmarks));
        report.push_str(&format!(
            "- Average execution time: {:.2} ms\n",
            avg_time_ms
        ));
        report.push_str(&format!(
            "- Configuration: {} iterations\n\n",
            self.config.iterations
        ));

        // Detailed results
        report.push_str("## Detailed Results\n\n");
        report.push_str(
            "| Benchmark | Avg Time (μs) | Min Time (μs) | Max Time (μs) | Throughput (ops/s) |\n",
        );
        report.push_str(
            "|-----------|---------------|---------------|---------------|--------------------|\n",
        );

        for result in &self.results {
            report.push_str(&format!(
                "| {} | {:.2} | {:.2} | {:.2} | {:.0} |\n",
                result.name,
                result.avg_time_us(),
                result.min_time_ns as f64 / 1000.0,
                result.max_time_ns as f64 / 1000.0,
                result.throughput
            ));
        }

        report
    }

    /// Save results to JSON file
    pub fn save_results(&self, filename: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.results)
            .map_err(|e| MyQuatError::circuit_error(format!("JSON serialization failed: {}", e)))?;

        std::fs::write(filename, json)
            .map_err(|e| MyQuatError::circuit_error(format!("Failed to write file: {}", e)))?;

        Ok(())
    }
}

impl Default for BenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_config_default() {
        let config = BenchmarkConfig::default();
        assert_eq!(config.iterations, 100);
        assert_eq!(config.warmup_iterations, 10);
        assert!(!config.qubit_range.is_empty());
    }

    #[test]
    fn test_benchmark_result_creation() {
        let timings = vec![
            Duration::from_nanos(1000),
            Duration::from_nanos(1200),
            Duration::from_nanos(800),
        ];

        let result = BenchmarkResult::from_timings("test".to_string(), &timings);
        assert_eq!(result.name, "test");
        assert_eq!(result.min_time_ns, 800);
        assert_eq!(result.max_time_ns, 1200);
        assert_eq!(result.avg_time_ns, 1000);
    }

    #[test]
    fn test_benchmark_suite_creation() {
        let suite = BenchmarkSuite::new();
        assert_eq!(suite.results.len(), 0);
        assert_eq!(suite.config.iterations, 100);
    }

    #[test]
    fn test_single_benchmark() {
        let mut suite = BenchmarkSuite::with_config(BenchmarkConfig {
            iterations: 5,
            warmup_iterations: 1,
            ..BenchmarkConfig::default()
        });

        let result = suite
            .benchmark("test_op", || {
                // Simulate some work
                std::thread::sleep(Duration::from_micros(10));
            })
            .unwrap();

        assert_eq!(result.name, "test_op");
        assert!(result.avg_time_us() >= 10.0);
        assert_eq!(suite.results.len(), 1);
    }
}
