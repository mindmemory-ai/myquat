#![allow(clippy::all)]
#![allow(unused_imports)]
#![allow(unused_mut)]
// Experiment 05: qDRIFT Circuit Detection and Restoration
//
// Author: gA4ss
//
// Systematically evaluates qDRIFT pattern detection and restoration across
// varying qubit counts, sample sizes, with/without ZZ gadgets, and different
// lambda/tau parameters. Also tests false-positive rates on non-qDRIFT circuits.
// Outputs a detailed JSON report.

use myquat::circuit::QuantumCircuit;
use myquat::deoptimization::{
    detect_qdrift_pattern, BenchmarkSuite, DeoptStrategy, QdriftRestorationStrategy,
};
use myquat::gates::StandardGate;
use myquat::parameter::Parameter;
use serde_json::{json, Value};
use std::time::Instant;

/// Generate a qDRIFT circuit with custom lambda and tau
fn gen_qdrift_custom(
    num_qubits: usize,
    num_samples: usize,
    lambda: f64,
    tau: f64,
) -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(num_qubits, 0);
    let theta = lambda * tau / num_samples as f64;

    let term_types = [StandardGate::Rx, StandardGate::Ry, StandardGate::Rz];
    for i in 0..num_samples {
        let q = (i * 7 + 3) % num_qubits;
        let gate = term_types[(i * 13 + 5) % 3];
        match gate {
            StandardGate::Rx => circuit.rx(q, Parameter::Float(theta)).unwrap(),
            StandardGate::Ry => circuit.ry(q, Parameter::Float(theta)).unwrap(),
            _ => circuit.rz(q, Parameter::Float(theta)).unwrap(),
        };
    }
    circuit
}

/// Generate a qDRIFT circuit with ZZ gadgets and custom params
fn gen_qdrift_zz_custom(
    num_qubits: usize,
    num_samples: usize,
    lambda: f64,
    tau: f64,
) -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(num_qubits, 0);
    let theta = lambda * tau / num_samples as f64;

    for i in 0..num_samples {
        let selector = (i * 11 + 7) % 4;
        if selector < 3 {
            let q = (i * 7 + 3) % num_qubits;
            match selector {
                0 => circuit.rx(q, Parameter::Float(theta)).unwrap(),
                1 => circuit.ry(q, Parameter::Float(theta)).unwrap(),
                _ => circuit.rz(q, Parameter::Float(theta)).unwrap(),
            };
        } else {
            let q0 = i % num_qubits;
            let q1 = (q0 + 1) % num_qubits;
            if q0 != q1 {
                circuit.cx(q0, q1).unwrap();
                circuit.rz(q1, Parameter::Float(theta)).unwrap();
                circuit.cx(q0, q1).unwrap();
            }
        }
    }
    circuit
}

/// Run qDRIFT detection test
fn run_qdrift_test(label: &str, circuit: &QuantumCircuit, is_qdrift: bool) -> Value {
    let strategy = QdriftRestorationStrategy::new();

    let start = Instant::now();
    let confidence = strategy.confidence(circuit);
    let conf_time_us = start.elapsed().as_micros();

    let start = Instant::now();
    let detection = detect_qdrift_pattern(circuit);
    let detect_time_us = start.elapsed().as_micros();

    let start = Instant::now();
    let restored = strategy.apply(circuit);
    let restore_time_us = start.elapsed().as_micros();

    let detected = detection.is_some();
    let correct = if is_qdrift { detected } else { !detected };

    json!({
        "label": label,
        "is_qdrift": is_qdrift,
        "detected": detected,
        "correct": correct,
        "confidence": confidence,
        "conf_time_us": conf_time_us,
        "detect_time_us": detect_time_us,
        "restore_time_us": restore_time_us,
        "input_gates": circuit.size(),
        "input_qubits": circuit.num_qubits(),
        "restored_gates": restored.as_ref().map(|c| c.size()).ok(),
        "detection_details": detection.as_ref().map(|d| json!({
            "num_samples": d.num_samples,
            "lambda": d.lambda,
            "tau": d.tau,
            "num_inferred_terms": d.inferred_terms.len(),
            "estimated_samples": d.inferred_terms.iter().map(|t| t.sample_count).sum::<usize>(),
        })),
    })
}

fn main() {
    let experiment_start = Instant::now();
    let mut results: Vec<Value> = Vec::new();

    // ========== True positive: qDRIFT circuits ==========

    // Sweep qubit count x sample size (single-qubit only)
    for &n in &[2, 3, 4, 5, 6, 8] {
        for &samples in &[5, 10, 20, 40, 60, 80, 120] {
            let c = gen_qdrift_custom(n, samples, 2.0, 1.0);
            let label = format!("qdrift_{}qb_n{}_l2_t1", n, samples);
            results.push(run_qdrift_test(&label, &c, true));
        }
    }

    // Sweep lambda and tau
    let lambda_tau_pairs: Vec<(f64, f64)> = vec![
        (0.5, 0.5),
        (1.0, 1.0),
        (2.0, 1.0),
        (3.0, 0.5),
        (5.0, 0.2),
        (1.0, 3.0),
    ];
    for &(lambda, tau) in &lambda_tau_pairs {
        for &n in &[3, 4, 6] {
            let c = gen_qdrift_custom(n, 30, lambda, tau);
            let label = format!("qdrift_{}qb_n30_l{}_t{}", n, lambda, tau);
            results.push(run_qdrift_test(&label, &c, true));
        }
    }

    // qDRIFT with ZZ gadgets
    for &n in &[3, 4, 5, 6] {
        for &samples in &[10, 20, 30, 50, 80] {
            let c = gen_qdrift_zz_custom(n, samples, 3.0, 1.0);
            let label = format!("qdrift_zz_{}qb_n{}", n, samples);
            results.push(run_qdrift_test(&label, &c, true));
        }
    }

    // BenchmarkSuite generators
    for &n in &[4, 6] {
        for &samples in &[20, 40] {
            let c = BenchmarkSuite::generate_qdrift(n, samples);
            let label = format!("qdrift_bench_{}qb_n{}", n, samples);
            results.push(run_qdrift_test(&label, &c, true));

            let c2 = BenchmarkSuite::generate_qdrift_with_zz(n, samples);
            let label2 = format!("qdrift_bench_zz_{}qb_n{}", n, samples);
            results.push(run_qdrift_test(&label2, &c2, true));
        }
    }

    // ========== True negative: non-qDRIFT circuits ==========

    // Trotter circuits
    for &n in &[3, 4, 5, 6] {
        for &steps in &[1, 2, 3] {
            let c = BenchmarkSuite::generate_hamiltonian_simulation(n, steps);
            let label = format!("neg_trotter_{}qb_{}s", n, steps);
            results.push(run_qdrift_test(&label, &c, false));
        }
    }

    // VQE circuits
    for &n in &[3, 4, 6] {
        let c = BenchmarkSuite::generate_vqe_ansatz(n, 3);
        let label = format!("neg_vqe_esu2_{}qb", n);
        results.push(run_qdrift_test(&label, &c, false));

        let c2 = BenchmarkSuite::generate_vqe_real_amplitudes(n, 3);
        let label2 = format!("neg_vqe_real_{}qb", n);
        results.push(run_qdrift_test(&label2, &c2, false));
    }

    // QAOA circuits
    for &n in &[3, 4, 5] {
        let c = BenchmarkSuite::generate_qaoa(n, 2);
        let label = format!("neg_qaoa_{}qb", n);
        results.push(run_qdrift_test(&label, &c, false));
    }

    // Pure H/CX circuits (no rotations at all)
    for &n in &[3, 4] {
        let mut c = QuantumCircuit::new(n, 0);
        for i in 0..n {
            c.h(i).unwrap();
        }
        for i in 0..n - 1 {
            c.cx(i, i + 1).unwrap();
        }
        for i in 0..n {
            c.h(i).unwrap();
        }
        let label = format!("neg_hcx_{}qb", n);
        results.push(run_qdrift_test(&label, &c, false));
    }

    let elapsed_ms = experiment_start.elapsed().as_millis();

    // ---- Summary ----
    let pos_results: Vec<&Value> = results
        .iter()
        .filter(|r| r["is_qdrift"].as_bool() == Some(true))
        .collect();
    let neg_results: Vec<&Value> = results
        .iter()
        .filter(|r| r["is_qdrift"].as_bool() == Some(false))
        .collect();

    let pos_correct = pos_results
        .iter()
        .filter(|r| r["correct"].as_bool() == Some(true))
        .count();
    let neg_correct = neg_results
        .iter()
        .filter(|r| r["correct"].as_bool() == Some(true))
        .count();

    // Per qubit count
    let mut per_qubit = Vec::new();
    for &n in &[2, 3, 4, 5, 6, 8] {
        let subset: Vec<&&Value> = pos_results
            .iter()
            .filter(|r| r["input_qubits"].as_u64() == Some(n as u64))
            .collect();
        if subset.is_empty() {
            continue;
        }
        let sub_correct = subset
            .iter()
            .filter(|r| r["correct"].as_bool() == Some(true))
            .count();
        let avg_conf: f64 = subset
            .iter()
            .filter_map(|r| r["confidence"].as_f64())
            .sum::<f64>()
            / subset.len() as f64;
        per_qubit.push(json!({
            "num_qubits": n,
            "total": subset.len(),
            "correct": sub_correct,
            "accuracy_pct": sub_correct as f64 / subset.len() as f64 * 100.0,
            "avg_confidence": avg_conf,
        }));
    }

    // Per sample count (only single-qubit qDRIFT with lambda=2 tau=1)
    let mut per_sample = Vec::new();
    for &s in &[5, 10, 20, 40, 60, 80, 120] {
        let subset: Vec<&&Value> = pos_results
            .iter()
            .filter(|r| {
                r["label"]
                    .as_str()
                    .is_some_and(|l| l.contains(&format!("_n{}_l2", s)))
            })
            .collect();
        if subset.is_empty() {
            continue;
        }
        let sub_correct = subset
            .iter()
            .filter(|r| r["correct"].as_bool() == Some(true))
            .count();
        let avg_conf: f64 = subset
            .iter()
            .filter_map(|r| r["confidence"].as_f64())
            .sum::<f64>()
            / subset.len() as f64;
        per_sample.push(json!({
            "num_samples": s,
            "total": subset.len(),
            "correct": sub_correct,
            "accuracy_pct": sub_correct as f64 / subset.len() as f64 * 100.0,
            "avg_confidence": avg_conf,
        }));
    }

    // ZZ vs non-ZZ comparison
    let zz_results: Vec<&&Value> = pos_results
        .iter()
        .filter(|r| r["label"].as_str().is_some_and(|l| l.contains("_zz_")))
        .collect();
    let nozz_results: Vec<&&Value> = pos_results
        .iter()
        .filter(|r| r["label"].as_str().is_some_and(|l| !l.contains("_zz_")))
        .collect();

    let zz_avg_conf: f64 = zz_results
        .iter()
        .filter_map(|r| r["confidence"].as_f64())
        .sum::<f64>()
        / zz_results.len().max(1) as f64;
    let nozz_avg_conf: f64 = nozz_results
        .iter()
        .filter_map(|r| r["confidence"].as_f64())
        .sum::<f64>()
        / nozz_results.len().max(1) as f64;

    let report = json!({
        "experiment": "exp05_qdrift_detection",
        "description": "qDRIFT detection accuracy across qubit counts, sample sizes, lambda/tau params, with/without ZZ gadgets; includes false-positive tests",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "summary": {
            "total_tests": results.len(),
            "positive_tests": pos_results.len(),
            "positive_correct": pos_correct,
            "positive_accuracy_pct": pos_correct as f64 / pos_results.len().max(1) as f64 * 100.0,
            "negative_tests": neg_results.len(),
            "negative_correct": neg_correct,
            "false_positive_rate_pct": (1.0 - neg_correct as f64 / neg_results.len().max(1) as f64) * 100.0,
            "total_time_ms": elapsed_ms,
        },
        "zz_comparison": {
            "with_zz_count": zz_results.len(),
            "with_zz_avg_confidence": zz_avg_conf,
            "without_zz_count": nozz_results.len(),
            "without_zz_avg_confidence": nozz_avg_conf,
        },
        "per_qubit_summary": per_qubit,
        "per_sample_summary": per_sample,
        "detailed_results": results,
    });

    let output_path =
        "experiments/reverse_hamiltonian_extraction/reports/exp05_qdrift_detection.json";
    std::fs::create_dir_all("experiments/reverse_hamiltonian_extraction/reports").unwrap();
    std::fs::write(output_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    println!("Report written to {}", output_path);
    println!(
        "Detection rate: {}/{} ({:.1}%)",
        pos_correct,
        pos_results.len(),
        pos_correct as f64 / pos_results.len().max(1) as f64 * 100.0
    );
    println!(
        "False positive rate: {:.1}%",
        (1.0 - neg_correct as f64 / neg_results.len().max(1) as f64) * 100.0
    );
}
