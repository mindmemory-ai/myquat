#![allow(clippy::all)]
#![allow(unused_imports)]
#![allow(unused_mut)]
// Experiment 06: Scalability and Performance Benchmark
//
// Author: gA4ss
//
// Measures how extraction time and memory usage scale with circuit size.
// Tests: (1) gate count scaling at fixed qubit count, (2) qubit count scaling
// at fixed depth, (3) deoptimization pipeline throughput, and (4) per-strategy
// overhead. Reports throughput in gates/second and microseconds per gate.
// Outputs a detailed JSON report.

use myquat::circuit::QuantumCircuit;
use myquat::deoptimization::{
    BenchmarkSuite, DeoptStrategy, DeoptimizationPipeline, KakRestorationStrategy,
    QdriftRestorationStrategy, TemplateMatchingStrategy, TemporalAnalysisStrategy,
    VqeRestorationStrategy,
};
use myquat::hamiltonian::{
    constructors, CircuitAnalyzer, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliOperator,
    PauliString, TrotterOrder,
};
use num_complex::Complex64;
use serde_json::{json, Value};
use std::time::Instant;

const WARMUP_ITERATIONS: usize = 2;
const MEASURE_ITERATIONS: usize = 5;

/// Time a closure over multiple iterations, return (avg_us, min_us, max_us)
fn bench_fn<F: FnMut()>(mut f: F, warmup: usize, iters: usize) -> (f64, u128, u128) {
    for _ in 0..warmup {
        f();
    }
    let mut times = Vec::with_capacity(iters);
    for _ in 0..iters {
        let start = Instant::now();
        f();
        times.push(start.elapsed().as_micros());
    }
    let min = *times.iter().min().unwrap();
    let max = *times.iter().max().unwrap();
    let avg = times.iter().sum::<u128>() as f64 / iters as f64;
    (avg, min, max)
}

fn build_ising(n: usize) -> Hamiltonian {
    constructors::ising_model(n, 1.0, 0.5).unwrap()
}

fn main() {
    let experiment_start = Instant::now();

    // ====== Test 1: Gate count scaling (fixed 4 qubits, vary Trotter steps) ======
    let mut gate_scaling_results = Vec::new();
    let n_qubits = 4;
    for &steps in &[1, 2, 3, 5, 8, 10, 15, 20, 30, 50] {
        let ham = build_ising(n_qubits);
        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: steps,
            evolution_time: 1.0,
            group_commuting_terms: false,
            auto_optimize_grouping: false,
            apply_circuit_optimization: false,
            ..Default::default()
        };
        let compiler = HamiltonianCompiler::new(config);
        let circuit = compiler.compile(&ham).unwrap();
        let gate_count = circuit.size();

        let analyzer = CircuitAnalyzer::new();
        let circuit_clone = circuit.clone();
        let (avg_us, min_us, max_us) = bench_fn(
            || {
                let _ = analyzer.analyze(&circuit_clone);
            },
            WARMUP_ITERATIONS,
            MEASURE_ITERATIONS,
        );

        gate_scaling_results.push(json!({
            "num_qubits": n_qubits,
            "trotter_steps": steps,
            "gate_count": gate_count,
            "avg_time_us": avg_us,
            "min_time_us": min_us,
            "max_time_us": max_us,
            "throughput_gates_per_sec": gate_count as f64 / (avg_us / 1e6),
            "us_per_gate": avg_us / gate_count as f64,
        }));
    }

    // ====== Test 2: Qubit count scaling (fixed depth) ======
    let mut qubit_scaling_results = Vec::new();
    for &n in &[2, 3, 4, 5, 6, 7, 8, 10, 12] {
        let ham = build_ising(n);
        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 2,
            evolution_time: 1.0,
            group_commuting_terms: false,
            auto_optimize_grouping: false,
            apply_circuit_optimization: false,
            ..Default::default()
        };
        let compiler = HamiltonianCompiler::new(config);
        let circuit = match compiler.compile(&ham) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let gate_count = circuit.size();

        let analyzer = CircuitAnalyzer::new();
        let circuit_clone = circuit.clone();
        let (avg_us, min_us, max_us) = bench_fn(
            || {
                let _ = analyzer.analyze(&circuit_clone);
            },
            WARMUP_ITERATIONS,
            MEASURE_ITERATIONS,
        );

        qubit_scaling_results.push(json!({
            "num_qubits": n,
            "gate_count": gate_count,
            "avg_time_us": avg_us,
            "min_time_us": min_us,
            "max_time_us": max_us,
            "throughput_gates_per_sec": gate_count as f64 / (avg_us / 1e6),
            "us_per_gate": avg_us / gate_count.max(1) as f64,
        }));
    }

    // ====== Test 3: Deoptimization pipeline throughput ======
    let mut pipeline_scaling_results = Vec::new();

    // Hamiltonian simulation circuits
    for &n in &[3, 4, 5, 6, 8] {
        for &steps in &[1, 2, 5, 10] {
            let circuit = BenchmarkSuite::generate_hamiltonian_simulation(n, steps);
            let gate_count = circuit.size();
            let pipeline = DeoptimizationPipeline::default();
            let circuit_clone = circuit.clone();

            let (avg_us, min_us, max_us) = bench_fn(
                || {
                    let _ = pipeline.restore(&circuit_clone);
                },
                WARMUP_ITERATIONS,
                MEASURE_ITERATIONS,
            );

            pipeline_scaling_results.push(json!({
                "circuit_type": "hamiltonian_simulation",
                "num_qubits": n,
                "param": steps,
                "gate_count": gate_count,
                "avg_time_us": avg_us,
                "min_time_us": min_us,
                "max_time_us": max_us,
                "throughput_gates_per_sec": gate_count as f64 / (avg_us / 1e6),
            }));
        }
    }

    // VQE circuits
    for &n in &[3, 4, 6, 8] {
        for &d in &[2, 3, 5, 7] {
            let circuit = BenchmarkSuite::generate_vqe_ansatz(n, d);
            let gate_count = circuit.size();
            let pipeline = DeoptimizationPipeline::default();
            let circuit_clone = circuit.clone();

            let (avg_us, min_us, max_us) = bench_fn(
                || {
                    let _ = pipeline.restore(&circuit_clone);
                },
                WARMUP_ITERATIONS,
                MEASURE_ITERATIONS,
            );

            pipeline_scaling_results.push(json!({
                "circuit_type": "vqe",
                "num_qubits": n,
                "param": d,
                "gate_count": gate_count,
                "avg_time_us": avg_us,
                "min_time_us": min_us,
                "max_time_us": max_us,
                "throughput_gates_per_sec": gate_count as f64 / (avg_us / 1e6),
            }));
        }
    }

    // qDRIFT circuits
    for &n in &[3, 4, 6] {
        for &samples in &[10, 20, 50, 100] {
            let circuit = BenchmarkSuite::generate_qdrift(n, samples);
            let gate_count = circuit.size();
            let pipeline = DeoptimizationPipeline::default();
            let circuit_clone = circuit.clone();

            let (avg_us, min_us, max_us) = bench_fn(
                || {
                    let _ = pipeline.restore(&circuit_clone);
                },
                WARMUP_ITERATIONS,
                MEASURE_ITERATIONS,
            );

            pipeline_scaling_results.push(json!({
                "circuit_type": "qdrift",
                "num_qubits": n,
                "param": samples,
                "gate_count": gate_count,
                "avg_time_us": avg_us,
                "min_time_us": min_us,
                "max_time_us": max_us,
                "throughput_gates_per_sec": gate_count as f64 / (avg_us / 1e6),
            }));
        }
    }

    // ====== Test 4: Per-strategy overhead ======
    let mut strategy_overhead_results = Vec::new();
    let test_circuits: Vec<(&str, QuantumCircuit)> = vec![
        (
            "ham_4qb_2s",
            BenchmarkSuite::generate_hamiltonian_simulation(4, 2),
        ),
        (
            "ham_6qb_3s",
            BenchmarkSuite::generate_hamiltonian_simulation(6, 3),
        ),
        ("vqe_4qb_d3", BenchmarkSuite::generate_vqe_ansatz(4, 3)),
        ("vqe_6qb_d3", BenchmarkSuite::generate_vqe_ansatz(6, 3)),
        ("qaoa_4qb_p2", BenchmarkSuite::generate_qaoa(4, 2)),
        ("qdrift_4qb_n30", BenchmarkSuite::generate_qdrift(4, 30)),
        ("qdrift_6qb_n50", BenchmarkSuite::generate_qdrift(6, 50)),
    ];

    let strategies: Vec<(Box<dyn DeoptStrategy>, &str)> = vec![
        (Box::new(KakRestorationStrategy::new()), "KAK"),
        (Box::new(TemplateMatchingStrategy::new()), "Template"),
        (Box::new(TemporalAnalysisStrategy::new()), "Temporal"),
        (Box::new(VqeRestorationStrategy::new()), "VQE"),
        (Box::new(QdriftRestorationStrategy::new()), "qDRIFT"),
    ];

    for (clabel, circuit) in &test_circuits {
        for (strategy, sname) in &strategies {
            let circuit_clone = circuit.clone();
            let (avg_conf_us, _, _) = bench_fn(
                || {
                    let _ = strategy.confidence(&circuit_clone);
                },
                WARMUP_ITERATIONS,
                MEASURE_ITERATIONS,
            );
            let (avg_apply_us, _, _) = bench_fn(
                || {
                    let _ = strategy.apply(&circuit_clone);
                },
                WARMUP_ITERATIONS,
                MEASURE_ITERATIONS,
            );

            strategy_overhead_results.push(json!({
                "circuit": clabel,
                "gate_count": circuit.size(),
                "strategy": sname,
                "avg_confidence_time_us": avg_conf_us,
                "avg_apply_time_us": avg_apply_us,
                "total_time_us": avg_conf_us + avg_apply_us,
            }));
        }
    }

    let elapsed_ms = experiment_start.elapsed().as_millis();

    // ---- Summary statistics ----
    let overall_throughput: f64 = gate_scaling_results
        .iter()
        .filter_map(|r| r["throughput_gates_per_sec"].as_f64())
        .sum::<f64>()
        / gate_scaling_results.len().max(1) as f64;

    let report = json!({
        "experiment": "exp06_scalability",
        "description": "Scalability and performance benchmark: gate count scaling, qubit count scaling, pipeline throughput, per-strategy overhead",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "summary": {
            "avg_extraction_throughput_gates_per_sec": overall_throughput,
            "gate_scaling_points": gate_scaling_results.len(),
            "qubit_scaling_points": qubit_scaling_results.len(),
            "pipeline_scaling_points": pipeline_scaling_results.len(),
            "strategy_overhead_points": strategy_overhead_results.len(),
            "total_time_ms": elapsed_ms,
        },
        "gate_count_scaling": gate_scaling_results,
        "qubit_count_scaling": qubit_scaling_results,
        "pipeline_throughput": pipeline_scaling_results,
        "strategy_overhead": strategy_overhead_results,
    });

    let output_path = "experiments/reverse_hamiltonian_extraction/reports/exp06_scalability.json";
    std::fs::create_dir_all("experiments/reverse_hamiltonian_extraction/reports").unwrap();
    std::fs::write(output_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    println!("Report written to {}", output_path);
    println!(
        "Avg extraction throughput: {:.0} gates/sec",
        overall_throughput
    );
}
