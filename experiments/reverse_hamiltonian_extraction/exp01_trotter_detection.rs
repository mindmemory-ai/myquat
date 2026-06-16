#![allow(clippy::all)]
#![allow(unused_imports)]
#![allow(unused_mut)]
// Experiment 01: Trotter Order Detection Accuracy
//
// Author: gA4ss
//
// Evaluates the CircuitAnalyzer's ability to correctly detect Trotter-Suzuki
// decomposition orders (1st through 10th) across different Hamiltonians,
// qubit counts, Trotter steps, and evolution times.
// Outputs a detailed JSON report.

use myquat::hamiltonian::{
    constructors, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString, TrotterOrder,
};
use num_complex::Complex64;
use serde_json::{json, Value};
use std::time::Instant;

/// Single test case result
fn run_detection_test(
    label: &str,
    hamiltonian: &Hamiltonian,
    order: TrotterOrder,
    steps: usize,
    evolution_time: f64,
) -> Value {
    let config = CompilerConfig {
        trotter_order: order.clone(),
        trotter_steps: steps,
        evolution_time,
        group_commuting_terms: false,
        auto_optimize_grouping: false,
        apply_circuit_optimization: false,
        ..Default::default()
    };
    let compiler = HamiltonianCompiler::new(config.clone());

    let start = Instant::now();
    let circuit = match compiler.compile(hamiltonian) {
        Ok(c) => c,
        Err(e) => {
            return json!({
                "label": label,
                "expected_order": format!("{:?}", order),
                "steps": steps,
                "evolution_time": evolution_time,
                "status": "compile_error",
                "error": format!("{}", e),
            });
        }
    };
    let compile_time_us = start.elapsed().as_micros();

    let analyzer = config.build_analyzer();
    let start = Instant::now();
    let analysis = match analyzer.analyze(&circuit) {
        Ok(a) => a,
        Err(e) => {
            return json!({
                "label": label,
                "expected_order": format!("{:?}", order),
                "steps": steps,
                "evolution_time": evolution_time,
                "gate_count": circuit.size(),
                "status": "analysis_error",
                "error": format!("{}", e),
            });
        }
    };
    let analysis_time_us = start.elapsed().as_micros();

    let detected_order = analysis.trotter_order.as_ref().map(|o| format!("{:?}", o));
    let expected_order_str = format!("{:?}", order);
    let correct = detected_order.as_deref() == Some(&expected_order_str);

    json!({
        "label": label,
        "expected_order": expected_order_str,
        "detected_order": detected_order,
        "correct": correct,
        "order_confidence": analysis.order_confidence,
        "detected_trotter_steps": analysis.trotter_steps,
        "detected_evolution_time": analysis.evolution_time,
        "expected_steps": steps,
        "expected_evolution_time": evolution_time,
        "gate_count": circuit.size(),
        "num_hamiltonian_terms": analysis.hamiltonian.num_terms(),
        "compile_time_us": compile_time_us,
        "analysis_time_us": analysis_time_us,
        "status": "ok",
    })
}

fn build_ising(n: usize, j: f64, h: f64) -> Hamiltonian {
    constructors::ising_model(n, j, h).unwrap()
}

fn build_heisenberg(n: usize) -> Hamiltonian {
    constructors::heisenberg_model(n, 1.0, 1.0, 1.0).unwrap()
}

fn build_transverse_field(n: usize) -> Hamiltonian {
    // H = -sum Z_i Z_{i+1} - h sum X_i
    let mut h = Hamiltonian::new(n);
    for i in 0..n - 1 {
        let mut ops = vec![myquat::hamiltonian::PauliOperator::I; n];
        ops[i] = myquat::hamiltonian::PauliOperator::Z;
        ops[i + 1] = myquat::hamiltonian::PauliOperator::Z;
        let ps = PauliString::new(ops, Complex64::new(1.0, 0.0));
        h.add_term(ps, Complex64::new(-1.0, 0.0)).unwrap();
    }
    for i in 0..n {
        let mut ops = vec![myquat::hamiltonian::PauliOperator::I; n];
        ops[i] = myquat::hamiltonian::PauliOperator::X;
        let ps = PauliString::new(ops, Complex64::new(1.0, 0.0));
        h.add_term(ps, Complex64::new(-0.5, 0.0)).unwrap();
    }
    h
}

fn main() {
    let experiment_start = Instant::now();

    let orders: Vec<(TrotterOrder, &str)> = vec![
        (TrotterOrder::First, "1st"),
        (TrotterOrder::Second, "2nd"),
        (TrotterOrder::Fourth, "4th"),
        (TrotterOrder::Sixth, "6th"),
        (TrotterOrder::Nth(8), "8th"),
        (TrotterOrder::Nth(10), "10th"),
    ];

    let qubit_counts = [3, 4, 5, 6];
    let step_counts = [1, 2, 3, 5];
    let evolution_times = [0.1, 0.5, 1.0, 2.0];

    let hamiltonians: Vec<(&str, Box<dyn Fn(usize) -> Hamiltonian>)> = vec![
        ("ising_j1_h05", Box::new(|n| build_ising(n, 1.0, 0.5))),
        ("heisenberg", Box::new(|n| build_heisenberg(n))),
        ("transverse_field", Box::new(|n| build_transverse_field(n))),
    ];

    let mut results: Vec<Value> = Vec::new();
    let mut total = 0usize;
    let mut correct = 0usize;

    for (order, order_label) in &orders {
        for &n_qubits in &qubit_counts {
            for (h_name, h_builder) in &hamiltonians {
                let h = h_builder(n_qubits);
                for &steps in &step_counts {
                    for &t in &evolution_times {
                        let label = format!(
                            "{}_{}qb_{}_{}_steps{}_t{}",
                            order_label, n_qubits, h_name, order_label, steps, t
                        );
                        let result = run_detection_test(&label, &h, order.clone(), steps, t);
                        total += 1;
                        if result["correct"].as_bool().unwrap_or(false) {
                            correct += 1;
                        }
                        results.push(result);
                    }
                }
            }
        }
    }

    let elapsed_ms = experiment_start.elapsed().as_millis();

    // Per-order summary
    let mut per_order_summary = Vec::new();
    for (order, order_label) in &orders {
        let order_str = format!("{:?}", order);
        let order_results: Vec<&Value> = results
            .iter()
            .filter(|r| r["expected_order"].as_str() == Some(&order_str))
            .collect();
        let order_total = order_results.len();
        let order_correct = order_results
            .iter()
            .filter(|r| r["correct"].as_bool().unwrap_or(false))
            .count();
        let avg_confidence: f64 = order_results
            .iter()
            .filter_map(|r| r["order_confidence"].as_f64())
            .sum::<f64>()
            / order_total.max(1) as f64;
        let avg_analysis_time: f64 = order_results
            .iter()
            .filter_map(|r| r["analysis_time_us"].as_u64().map(|v| v as f64))
            .sum::<f64>()
            / order_total.max(1) as f64;

        per_order_summary.push(json!({
            "order": order_label,
            "total": order_total,
            "correct": order_correct,
            "accuracy_pct": if order_total > 0 { order_correct as f64 / order_total as f64 * 100.0 } else { 0.0 },
            "avg_confidence": avg_confidence,
            "avg_analysis_time_us": avg_analysis_time,
        }));
    }

    // Per-hamiltonian summary
    let mut per_hamiltonian_summary = Vec::new();
    for (h_name, _) in &hamiltonians {
        let h_results: Vec<&Value> = results
            .iter()
            .filter(|r| r["label"].as_str().is_some_and(|l| l.contains(h_name)))
            .collect();
        let h_total = h_results.len();
        let h_correct = h_results
            .iter()
            .filter(|r| r["correct"].as_bool().unwrap_or(false))
            .count();
        per_hamiltonian_summary.push(json!({
            "hamiltonian": h_name,
            "total": h_total,
            "correct": h_correct,
            "accuracy_pct": if h_total > 0 { h_correct as f64 / h_total as f64 * 100.0 } else { 0.0 },
        }));
    }

    // Per-qubit summary
    let mut per_qubit_summary = Vec::new();
    for &n in &qubit_counts {
        let q_results: Vec<&Value> = results
            .iter()
            .filter(|r| {
                r["label"]
                    .as_str()
                    .is_some_and(|l| l.contains(&format!("{}qb", n)))
            })
            .collect();
        let q_total = q_results.len();
        let q_correct = q_results
            .iter()
            .filter(|r| r["correct"].as_bool().unwrap_or(false))
            .count();
        per_qubit_summary.push(json!({
            "num_qubits": n,
            "total": q_total,
            "correct": q_correct,
            "accuracy_pct": if q_total > 0 { q_correct as f64 / q_total as f64 * 100.0 } else { 0.0 },
        }));
    }

    let report = json!({
        "experiment": "exp01_trotter_detection",
        "description": "Trotter-Suzuki order detection accuracy across orders 1-10, multiple Hamiltonians, qubit counts, step counts, and evolution times",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "parameters": {
            "orders": ["1st", "2nd", "4th", "6th", "8th", "10th"],
            "qubit_counts": qubit_counts,
            "step_counts": step_counts,
            "evolution_times": evolution_times,
            "hamiltonians": hamiltonians.iter().map(|(n, _)| n).collect::<Vec<_>>(),
        },
        "summary": {
            "total_tests": total,
            "total_correct": correct,
            "overall_accuracy_pct": correct as f64 / total.max(1) as f64 * 100.0,
            "total_time_ms": elapsed_ms,
        },
        "per_order_summary": per_order_summary,
        "per_hamiltonian_summary": per_hamiltonian_summary,
        "per_qubit_summary": per_qubit_summary,
        "detailed_results": results,
    });

    let output_path =
        "experiments/reverse_hamiltonian_extraction/reports/exp01_trotter_detection.json";
    std::fs::create_dir_all("experiments/reverse_hamiltonian_extraction/reports").unwrap();
    std::fs::write(output_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    println!("Report written to {}", output_path);
    println!(
        "Overall accuracy: {}/{} ({:.1}%)",
        correct,
        total,
        correct as f64 / total.max(1) as f64 * 100.0
    );
}
