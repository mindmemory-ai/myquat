//! CI/CD Benchmark Integration Script
//! 
//! This script runs benchmarks in CI/CD environments and detects performance regressions.

use myquat::*;
use std::env;
use std::process;

fn main() -> Result<()> {
    println!("🔄 CI/CD Benchmark Integration");
    println!("=============================");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("full");
    
    match mode {
        "quick" => run_quick_benchmarks()?,
        "full" => run_full_benchmarks()?,
        "regression" => run_regression_check()?,
        _ => {
            eprintln!("Usage: {} [quick|full|regression]", args[0]);
            process::exit(1);
        }
    }
    
    Ok(())
}

/// Run quick benchmarks for fast CI feedback
fn run_quick_benchmarks() -> Result<()> {
    println!("⚡ Running Quick Benchmarks");
    println!("---------------------------");
    
    let config = BenchmarkConfig {
        iterations: 10,
        warmup_iterations: 2,
        max_time_seconds: 30,
        measure_memory: false,
        qubit_range: vec![2, 4, 6],
        depth_range: vec![10, 50],
    };
    
    let mut suite = BenchmarkSuite::with_config(config);
    
    // Run only essential benchmarks
    run_essential_benchmarks(&mut suite)?;
    
    // Save results
    suite.save_results("ci_quick_results.json")?;
    
    // Check for major regressions
    check_major_regressions(&suite)?;
    
    println!("✅ Quick benchmarks completed");
    Ok(())
}

/// Run full benchmark suite
fn run_full_benchmarks() -> Result<()> {
    println!("🔬 Running Full Benchmark Suite");
    println!("-------------------------------");
    
    let config = BenchmarkConfig {
        iterations: 50,
        warmup_iterations: 5,
        max_time_seconds: 300, // 5 minutes max
        measure_memory: true,
        qubit_range: vec![2, 4, 6, 8, 10],
        depth_range: vec![10, 50, 100, 200],
    };
    
    let mut suite = BenchmarkSuite::with_config(config);
    suite.run_all()?;
    
    // Save results with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("ci_full_results_{}.json", timestamp);
    suite.save_results(&filename)?;
    
    // Generate report
    let report = suite.generate_report();
    std::fs::write("benchmark_report.md", report)?;
    
    println!("✅ Full benchmarks completed");
    println!("📊 Results saved to: {}", filename);
    println!("📋 Report saved to: benchmark_report.md");
    
    Ok(())
}

/// Run regression detection
fn run_regression_check() -> Result<()> {
    println!("🔍 Running Regression Check");
    println!("---------------------------");
    
    // Load current results
    let current_results = load_latest_results()?;
    
    // Initialize regression detector
    let mut detector = RegressionDetector::with_history_file("performance_history.json".to_string());
    detector.load_history()?;
    
    // Get git commit hash if available
    let commit_hash = get_git_commit_hash();
    
    // Add current results to history
    detector.add_results(&current_results, commit_hash)?;
    detector.save_history()?;
    
    // Detect regressions
    let regression_results = detector.detect_regressions(&current_results);
    
    // Generate regression report
    let report = detector.generate_report(&regression_results);
    std::fs::write("regression_report.md", report)?;
    
    // Check for critical regressions
    let critical_regressions: Vec<_> = regression_results.iter()
        .filter(|r| r.change_type == ChangeType::Regression && r.is_significant)
        .collect();
    
    if !critical_regressions.is_empty() {
        println!("🚨 Critical regressions detected:");
        for regression in &critical_regressions {
            println!("  - {}: {:.1}% slower", 
                regression.benchmark_name, 
                (regression.change_ratio - 1.0) * 100.0);
        }
        
        // Exit with error code to fail CI
        if env::var("CI_FAIL_ON_REGRESSION").unwrap_or("true".to_string()) == "true" {
            println!("❌ CI failing due to performance regressions");
            process::exit(1);
        }
    } else {
        println!("✅ No critical regressions detected");
    }
    
    // Report improvements
    let improvements: Vec<_> = regression_results.iter()
        .filter(|r| r.change_type == ChangeType::Improvement && r.is_significant)
        .collect();
    
    if !improvements.is_empty() {
        println!("🎉 Performance improvements detected:");
        for improvement in &improvements {
            println!("  - {}: {:.1}% faster", 
                improvement.benchmark_name, 
                (1.0 - improvement.change_ratio) * 100.0);
        }
    }
    
    println!("📋 Regression report saved to: regression_report.md");
    Ok(())
}

/// Run essential benchmarks for quick feedback
fn run_essential_benchmarks(suite: &mut BenchmarkSuite) -> Result<()> {
    // Circuit construction
    suite.run_circuit_construction_benchmarks()?;
    
    // Basic gate operations
    suite.run_gate_operation_benchmarks()?;
    
    // Small simulation
    let result = suite.benchmark("simulation_essential", || {
        let mut circuit = QuantumCircuit::new(4, 4);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.cx(1, 2).unwrap();
        circuit.cx(2, 3).unwrap();
        
        let mut simulator = StateVectorSimulator::new(4);
        simulator.execute_circuit(&circuit).unwrap();
    })?;
    
    println!("Essential Simulation (4 qubits): {:.2} μs", result.avg_time_us());
    
    Ok(())
}

/// Check for major performance regressions
fn check_major_regressions(suite: &BenchmarkSuite) -> Result<()> {
    let results = suite.results();
    
    // Define performance thresholds (in microseconds)
    let thresholds = [
        ("circuit_creation", 10.0),
        ("gate_addition", 5.0),
        ("simulation_essential", 1000.0),
    ];
    
    let mut violations = Vec::new();
    
    for (name, threshold) in &thresholds {
        if let Some(result) = results.iter().find(|r| r.name == *name) {
            if result.avg_time_us() > *threshold {
                violations.push((name, result.avg_time_us(), threshold));
            }
        }
    }
    
    if !violations.is_empty() {
        println!("⚠️  Performance threshold violations:");
        for (name, actual, threshold) in &violations {
            println!("  - {}: {:.2} μs (threshold: {:.2} μs)", name, actual, threshold);
        }
        
        // Don't fail CI for threshold violations, just warn
        println!("⚠️  Consider investigating performance issues");
    }
    
    Ok(())
}

/// Load latest benchmark results
fn load_latest_results() -> Result<Vec<BenchmarkResult>> {
    // Try to load from the most recent results file
    let files = ["ci_quick_results.json", "ci_full_results.json"];
    
    for file in &files {
        if std::path::Path::new(file).exists() {
            let content = std::fs::read_to_string(file)
                .map_err(|e| MyQuatError::circuit_error(&format!("Failed to read {}: {}", file, e)))?;
            
            let results: Vec<BenchmarkResult> = serde_json::from_str(&content)
                .map_err(|e| MyQuatError::circuit_error(&format!("Failed to parse {}: {}", file, e)))?;
            
            return Ok(results);
        }
    }
    
    Err(MyQuatError::circuit_error("No benchmark results found"))
}

/// Get git commit hash if available
fn get_git_commit_hash() -> Option<String> {
    use std::process::Command;
    
    Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

/// Print CI environment information
fn print_ci_info() {
    println!("CI Environment Information:");
    
    if let Ok(ci) = env::var("CI") {
        println!("  CI: {}", ci);
    }
    
    if let Ok(github_actions) = env::var("GITHUB_ACTIONS") {
        println!("  GitHub Actions: {}", github_actions);
    }
    
    if let Ok(runner_os) = env::var("RUNNER_OS") {
        println!("  Runner OS: {}", runner_os);
    }
    
    if let Ok(rust_version) = env::var("RUSTC_VERSION") {
        println!("  Rust Version: {}", rust_version);
    }
    
    println!();
}
