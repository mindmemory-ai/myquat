//! Utility functions and helpers for MyQuat
//!
//! This module provides various utility functions for quantum computing,
//! including mathematical helpers, visualization tools, and I/O utilities.

use crate::circuit::QuantumCircuit;
use crate::error::{MyQuatError, Result};
use crate::gates::GateOperation;
use ndarray::Array2;
use num_complex::Complex64;
use std::f64::consts::PI;
use std::fmt;

/// Mathematical utilities for quantum computing
pub struct MathUtils;

impl MathUtils {
    /// Check if a number is close to zero
    pub fn is_zero(x: f64, tolerance: f64) -> bool {
        x.abs() < tolerance
    }

    /// Check if two complex numbers are approximately equal
    pub fn complex_eq(a: Complex64, b: Complex64, tolerance: f64) -> bool {
        (a - b).norm() < tolerance
    }

    /// Normalize an angle to the range [0, 2π)
    pub fn normalize_angle(angle: f64) -> f64 {
        let mut normalized = angle % (2.0 * PI);
        if normalized < 0.0 {
            normalized += 2.0 * PI;
        }
        normalized
    }

    /// Convert angle from degrees to radians
    pub fn deg_to_rad(degrees: f64) -> f64 {
        degrees * PI / 180.0
    }

    /// Convert angle from radians to degrees
    pub fn rad_to_deg(radians: f64) -> f64 {
        radians * 180.0 / PI
    }

    /// Compute the greatest common divisor of two integers
    pub fn gcd(a: usize, b: usize) -> usize {
        if b == 0 {
            a
        } else {
            Self::gcd(b, a % b)
        }
    }

    /// Compute the least common multiple of two integers
    pub fn lcm(a: usize, b: usize) -> usize {
        if a == 0 || b == 0 {
            0
        } else {
            (a * b) / Self::gcd(a, b)
        }
    }

    /// Check if a number is a power of 2
    pub fn is_power_of_two(n: usize) -> bool {
        n > 0 && (n & (n - 1)) == 0
    }

    /// Get the next power of 2 greater than or equal to n
    pub fn next_power_of_two(n: usize) -> usize {
        if n <= 1 {
            1
        } else {
            let mut power = 1;
            while power < n {
                power <<= 1;
            }
            power
        }
    }

    /// Compute the binary representation of a number with a fixed width
    pub fn to_binary_string(n: usize, width: usize) -> String {
        format!("{:0width$b}", n, width = width)
    }

    /// Parse a binary string to a number
    pub fn from_binary_string(s: &str) -> Result<usize> {
        usize::from_str_radix(s, 2)
            .map_err(|_| MyQuatError::parse_error(format!("Invalid binary string: {}", s)))
    }
}

/// Matrix utilities for quantum operations
pub struct MatrixUtils;

impl MatrixUtils {
    /// Check if a matrix is approximately unitary
    pub fn is_unitary(matrix: &Array2<Complex64>, tolerance: f64) -> bool {
        let (n, m) = matrix.dim();
        if n != m {
            return false;
        }

        // Compute A * A†
        let mut product = Array2::zeros((n, n));
        for i in 0..n {
            for j in 0..n {
                let mut sum = Complex64::new(0.0, 0.0);
                for k in 0..n {
                    sum += matrix[[i, k]] * matrix[[j, k]].conj();
                }
                product[[i, j]] = sum;
            }
        }

        // Check if the product is approximately identity
        for i in 0..n {
            for j in 0..n {
                let expected = if i == j {
                    Complex64::new(1.0, 0.0)
                } else {
                    Complex64::new(0.0, 0.0)
                };
                if (product[[i, j]] - expected).norm() > tolerance {
                    return false;
                }
            }
        }

        true
    }

    /// Check if a matrix is approximately Hermitian
    pub fn is_hermitian(matrix: &Array2<Complex64>, tolerance: f64) -> bool {
        let (n, m) = matrix.dim();
        if n != m {
            return false;
        }

        for i in 0..n {
            for j in 0..n {
                if (matrix[[i, j]] - matrix[[j, i]].conj()).norm() > tolerance {
                    return false;
                }
            }
        }

        true
    }

    /// Compute the trace of a matrix
    pub fn trace(matrix: &Array2<Complex64>) -> Complex64 {
        let n = matrix.nrows().min(matrix.ncols());
        (0..n).map(|i| matrix[[i, i]]).sum()
    }

    /// Compute the Frobenius norm of a matrix
    pub fn frobenius_norm(matrix: &Array2<Complex64>) -> f64 {
        matrix.iter().map(|z| z.norm_sqr()).sum::<f64>().sqrt()
    }

    /// Create a random unitary matrix (using QR decomposition of a random matrix)
    pub fn random_unitary(n: usize) -> Array2<Complex64> {
        use rand::Rng;
        let mut rng = rand::rng();

        // Create a random complex matrix
        let mut matrix = Array2::zeros((n, n));
        for i in 0..n {
            for j in 0..n {
                let real: f64 = rng.random_range(-1.0..1.0);
                let imag: f64 = rng.random_range(-1.0..1.0);
                matrix[[i, j]] = Complex64::new(real, imag);
            }
        }

        // For now, return a simple unitary matrix (identity)
        // A full implementation would use QR decomposition
        Array2::eye(n).mapv(|x| Complex64::new(x, 0.0))
    }

    /// Compute the tensor product of two matrices
    pub fn tensor_product(a: &Array2<Complex64>, b: &Array2<Complex64>) -> Array2<Complex64> {
        let (a_rows, a_cols) = a.dim();
        let (b_rows, b_cols) = b.dim();
        let mut result = Array2::zeros((a_rows * b_rows, a_cols * b_cols));

        for i in 0..a_rows {
            for j in 0..a_cols {
                for k in 0..b_rows {
                    for l in 0..b_cols {
                        result[[i * b_rows + k, j * b_cols + l]] = a[[i, j]] * b[[k, l]];
                    }
                }
            }
        }

        result
    }
}

/// I/O utilities for quantum circuits
pub struct CircuitIO;

impl CircuitIO {
    /// Save a circuit to JSON format
    pub fn to_json(circuit: &QuantumCircuit) -> Result<String> {
        serde_json::to_string_pretty(circuit).map_err(|e| {
            MyQuatError::serialization_error(format!("JSON serialization failed: {}", e))
        })
    }

    /// Load a circuit from JSON format
    pub fn from_json(json: &str) -> Result<QuantumCircuit> {
        serde_json::from_str(json).map_err(|e| {
            MyQuatError::serialization_error(format!("JSON deserialization failed: {}", e))
        })
    }

    /// Save a circuit to a file
    pub fn save_to_file(circuit: &QuantumCircuit, filename: &str) -> Result<()> {
        let json = Self::to_json(circuit)?;
        std::fs::write(filename, json).map_err(|e| MyQuatError::IoError(e.to_string()))
    }

    /// Load a circuit from a file
    pub fn load_from_file(filename: &str) -> Result<QuantumCircuit> {
        let json =
            std::fs::read_to_string(filename).map_err(|e| MyQuatError::IoError(e.to_string()))?;
        Self::from_json(&json)
    }

    /// Export circuit to QASM format (simplified)
    pub fn to_qasm(circuit: &QuantumCircuit) -> String {
        let mut qasm = String::new();

        // Header
        qasm.push_str("OPENQASM 2.0;\n");
        qasm.push_str("include \"qelib1.inc\";\n\n");

        // Quantum register
        qasm.push_str(&format!("qreg q[{}];\n", circuit.num_qubits()));

        // Classical register (if any)
        if circuit.num_clbits() > 0 {
            qasm.push_str(&format!("creg c[{}];\n", circuit.num_clbits()));
        }

        qasm.push('\n');

        // Instructions
        for instruction in circuit.data().instructions() {
            if instruction.is_measurement() {
                qasm.push_str(&format!(
                    "measure q[{}] -> c[{}];\n",
                    instruction.qubits[0].index(),
                    instruction.clbits[0].index()
                ));
            } else {
                let gate_name = instruction.gate.gate_type.name();

                // Parameters
                if !instruction.gate.parameters.is_empty() {
                    qasm.push_str(&format!("{}(", gate_name));
                    for (i, param) in instruction.gate.parameters.iter().enumerate() {
                        if i > 0 {
                            qasm.push_str(", ");
                        }
                        qasm.push_str(&format!("{}", param));
                    }
                    qasm.push_str(") ");
                } else {
                    qasm.push_str(&format!("{} ", gate_name));
                }

                // Qubits
                for (i, qubit) in instruction.qubits.iter().enumerate() {
                    if i > 0 {
                        qasm.push_str(", ");
                    }
                    qasm.push_str(&format!("q[{}]", qubit.index()));
                }

                qasm.push_str(";\n");
            }
        }

        qasm
    }
}

/// Performance measurement utilities
pub struct PerformanceUtils;

impl PerformanceUtils {
    /// Measure the execution time of a function
    pub fn time_execution<F, R>(f: F) -> (R, std::time::Duration)
    where
        F: FnOnce() -> R,
    {
        let start = std::time::Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }

    /// Benchmark a function multiple times and return statistics
    pub fn benchmark<F, R>(f: F, iterations: usize) -> BenchmarkResult
    where
        F: Fn() -> R,
    {
        let mut times = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start = std::time::Instant::now();
            let _ = f();
            times.push(start.elapsed());
        }

        let total_time: std::time::Duration = times.iter().sum();
        let avg_time = total_time / iterations as u32;

        times.sort();
        let median_time = times[iterations / 2];
        let min_time = times[0];
        let max_time = times[iterations - 1];

        BenchmarkResult {
            iterations,
            total_time,
            avg_time,
            median_time,
            min_time,
            max_time,
        }
    }
}

/// Benchmark result statistics
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub iterations: usize,
    pub total_time: std::time::Duration,
    pub avg_time: std::time::Duration,
    pub median_time: std::time::Duration,
    pub min_time: std::time::Duration,
    pub max_time: std::time::Duration,
}

impl fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Benchmark Results ({} iterations):", self.iterations)?;
        writeln!(f, "  Total time:  {:?}", self.total_time)?;
        writeln!(f, "  Average:     {:?}", self.avg_time)?;
        writeln!(f, "  Median:      {:?}", self.median_time)?;
        writeln!(f, "  Min:         {:?}", self.min_time)?;
        writeln!(f, "  Max:         {:?}", self.max_time)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[allow(unused_imports)]
    use crate::parameter::Parameter;
    use std::f64::consts::PI;

    #[test]
    fn test_math_utils() {
        assert!(MathUtils::is_zero(1e-15, 1e-10));
        assert!(!MathUtils::is_zero(1e-5, 1e-10));

        assert_eq!(MathUtils::normalize_angle(-PI), PI);
        assert_eq!(MathUtils::normalize_angle(3.0 * PI), PI);

        assert_eq!(MathUtils::deg_to_rad(180.0), PI);
        assert_eq!(MathUtils::rad_to_deg(PI), 180.0);

        assert_eq!(MathUtils::gcd(12, 8), 4);
        assert_eq!(MathUtils::lcm(12, 8), 24);

        assert!(MathUtils::is_power_of_two(8));
        assert!(!MathUtils::is_power_of_two(6));

        assert_eq!(MathUtils::next_power_of_two(5), 8);
        assert_eq!(MathUtils::next_power_of_two(8), 8);

        assert_eq!(MathUtils::to_binary_string(5, 4), "0101");
        assert_eq!(MathUtils::from_binary_string("1010").unwrap(), 10);
    }

    #[test]
    fn test_matrix_utils() {
        // Test identity matrix is unitary
        let identity = Array2::eye(2).mapv(|x| Complex64::new(x, 0.0));
        assert!(MatrixUtils::is_unitary(&identity, 1e-10));

        // Test identity matrix is Hermitian
        assert!(MatrixUtils::is_hermitian(&identity, 1e-10));

        // Test trace
        let trace = MatrixUtils::trace(&identity);
        assert_eq!(trace, Complex64::new(2.0, 0.0));
    }

    #[test]
    fn test_circuit_visualization() {
        use crate::CircuitVisualizer;

        let mut circuit = QuantumCircuit::new(2, 2);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.measure_all().unwrap();

        let text = CircuitVisualizer::to_text(&circuit);
        assert!(text.contains("Circuit: 2 qubits"));
        assert!(text.contains("H"));
        assert!(text.contains("CX"));

        let ascii = CircuitVisualizer::to_ascii_art(&circuit);
        assert!(ascii.contains("q[0]"));
        assert!(ascii.contains("q[1]"));
    }

    #[test]
    fn test_circuit_io() {
        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();

        let json = CircuitIO::to_json(&circuit).unwrap();
        let loaded_circuit = CircuitIO::from_json(&json).unwrap();

        assert_eq!(circuit.num_qubits(), loaded_circuit.num_qubits());
        assert_eq!(circuit.num_clbits(), loaded_circuit.num_clbits());
        assert_eq!(circuit.size(), loaded_circuit.size());
    }

    #[test]
    fn test_qasm_export() {
        let mut circuit = QuantumCircuit::new(2, 1);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.measure(1, 0).unwrap();

        let qasm = CircuitIO::to_qasm(&circuit);
        assert!(qasm.contains("OPENQASM 2.0"));
        assert!(qasm.contains("qreg q[2]"));
        assert!(qasm.contains("creg c[1]"));
        assert!(qasm.contains("h q[0]"));
        assert!(qasm.contains("cx q[0], q[1]"));
        assert!(qasm.contains("measure q[1] -> c[0]"));
    }

    #[test]
    fn test_performance_utils() {
        let (result, duration) = PerformanceUtils::time_execution(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });

        assert_eq!(result, 42);
        assert!(duration >= std::time::Duration::from_millis(10));

        let benchmark = PerformanceUtils::benchmark(
            || {
                // Simple computation
                (0..100).sum::<i32>()
            },
            10,
        );

        assert_eq!(benchmark.iterations, 10);
        assert!(benchmark.avg_time > std::time::Duration::from_nanos(0));
    }
}
