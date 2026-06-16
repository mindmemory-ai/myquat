#![allow(clippy::all)]
#![allow(unused_imports)]
#![allow(unused_mut)]
// Experiment 04: VQE Ansatz Recognition Accuracy
//
// Author: gA4ss
//
// Systematically evaluates VQE ansatz pattern detection across three ansatz
// families (EfficientSU2, RealAmplitudes, UCCSD), varying qubit counts,
// circuit depths, entanglement patterns, and rotation angle distributions.
// Also tests false-positive rates on non-VQE circuits.
// Outputs a detailed JSON report.

use myquat::circuit::QuantumCircuit;
use myquat::deoptimization::{
    detect_vqe_pattern, BenchmarkSuite, DeoptStrategy, VQEType, VqeRestorationStrategy,
};
use myquat::parameter::Parameter;
use serde_json::{json, Value};
use std::time::Instant;

/// Run VQE detection on a single circuit
fn run_vqe_test(label: &str, circuit: &QuantumCircuit, expected_type: Option<&str>) -> Value {
    let strategy = VqeRestorationStrategy::new();

    let start = Instant::now();
    let confidence = strategy.confidence(circuit);
    let detect_time_us = start.elapsed().as_micros();

    let start = Instant::now();
    let detection = detect_vqe_pattern(circuit);
    let pattern_time_us = start.elapsed().as_micros();

    let start = Instant::now();
    let restored = strategy.apply(circuit);
    let restore_time_us = start.elapsed().as_micros();

    let detected_type = detection.as_ref().map(|d| format!("{:?}", d.vqe_type));
    let correct = match expected_type {
        Some(exp) => detected_type.as_deref() == Some(exp),
        None => detection.is_none(), // expect no detection for non-VQE
    };

    json!({
        "label": label,
        "expected_type": expected_type,
        "detected_type": detected_type,
        "correct": correct,
        "confidence": confidence,
        "detect_time_us": detect_time_us,
        "pattern_time_us": pattern_time_us,
        "restore_time_us": restore_time_us,
        "input_gates": circuit.size(),
        "input_qubits": circuit.num_qubits(),
        "restored_gates": restored.as_ref().map(|c| c.size()).ok(),
        "detection_details": detection.as_ref().map(|d| json!({
            "vqe_type": format!("{:?}", d.vqe_type),
            "num_layers": d.num_layers,
            "confidence": d.confidence,
        })),
    })
}

/// Generate EfficientSU2 with custom angle spread
fn gen_esu2_varied(n: usize, depth: usize, angle_scale: f64) -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(n, 0);
    for layer in 0..depth {
        for i in 0..n {
            let a = angle_scale * (0.1 * (layer * n + i) as f64 + 0.15);
            circuit.ry(i, Parameter::Float(a)).unwrap();
            circuit.rz(i, Parameter::Float(a * 0.7)).unwrap();
        }
        for i in 0..n.saturating_sub(1) {
            circuit.cx(i, i + 1).unwrap();
        }
    }
    circuit
}

/// Generate RealAmplitudes with custom angle spread
fn gen_real_varied(n: usize, depth: usize, angle_scale: f64) -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(n, 0);
    for layer in 0..depth {
        for i in 0..n {
            let a = angle_scale * (0.1 * (layer * n + i) as f64 + 0.2);
            circuit.ry(i, Parameter::Float(a)).unwrap();
        }
        for i in 0..n.saturating_sub(1) {
            circuit.cx(i, i + 1).unwrap();
        }
    }
    for i in 0..n {
        circuit.ry(i, Parameter::Float(angle_scale * 0.5)).unwrap();
    }
    circuit
}

/// Generate a non-VQE Trotter circuit (should not be detected as VQE)
fn gen_trotter_circuit(n: usize, steps: usize) -> QuantumCircuit {
    BenchmarkSuite::generate_hamiltonian_simulation(n, steps)
}

/// Generate a random rotation circuit (should not be detected as VQE)
fn gen_random_rotations(n: usize, num_gates: usize) -> QuantumCircuit {
    let mut circuit = QuantumCircuit::new(n, 0);
    for i in 0..num_gates {
        let q = i % n;
        let angle = 0.1 * i as f64 + 0.37;
        match i % 5 {
            0 => {
                circuit.rx(q, Parameter::Float(angle)).unwrap();
            }
            1 => {
                circuit.ry(q, Parameter::Float(angle)).unwrap();
            }
            2 => {
                circuit.rz(q, Parameter::Float(angle)).unwrap();
            }
            3 => {
                let q2 = (q + 1) % n;
                if q != q2 {
                    circuit.cx(q, q2).unwrap();
                }
            }
            _ => {
                circuit.h(q).unwrap();
            }
        }
    }
    circuit
}

fn main() {
    let experiment_start = Instant::now();
    let mut results: Vec<Value> = Vec::new();

    // ========== True positive tests ==========

    // EfficientSU2: sweep qubits x depth x angle_scale
    for &n in &[3, 4, 5, 6, 8] {
        for &d in &[1, 2, 3, 5, 7] {
            for &scale in &[0.5, 1.0, 2.0] {
                let c = gen_esu2_varied(n, d, scale);
                let label = format!("esu2_{}qb_d{}_s{}", n, d, scale);
                results.push(run_vqe_test(&label, &c, Some("EfficientSU2")));
            }
        }
    }

    // Also test via BenchmarkSuite generator
    for &n in &[4, 6, 8] {
        for &d in &[2, 3, 5] {
            let c = BenchmarkSuite::generate_vqe_ansatz(n, d);
            let label = format!("esu2_bench_{}qb_d{}", n, d);
            results.push(run_vqe_test(&label, &c, Some("EfficientSU2")));
        }
    }

    // RealAmplitudes: sweep qubits x depth x angle_scale
    for &n in &[3, 4, 5, 6, 8] {
        for &d in &[1, 2, 3, 5, 7] {
            for &scale in &[0.5, 1.0, 2.0] {
                let c = gen_real_varied(n, d, scale);
                let label = format!("real_{}qb_d{}_s{}", n, d, scale);
                results.push(run_vqe_test(&label, &c, Some("RealAmplitudes")));
            }
        }
    }

    // BenchmarkSuite RealAmplitudes
    for &n in &[4, 6] {
        for &d in &[3, 5, 7] {
            let c = BenchmarkSuite::generate_vqe_real_amplitudes(n, d);
            let label = format!("real_bench_{}qb_d{}", n, d);
            results.push(run_vqe_test(&label, &c, Some("RealAmplitudes")));
        }
    }

    // UCCSD: sweep qubit/electron combos
    let uccsd_configs: Vec<(usize, usize)> =
        vec![(4, 2), (6, 2), (6, 3), (8, 2), (8, 3), (8, 4), (10, 2)];
    for &(n, ne) in &uccsd_configs {
        let c = BenchmarkSuite::generate_vqe_uccsd(n, ne);
        let label = format!("uccsd_{}qb_{}e", n, ne);
        // UCCSD detection may return UCCSD or EfficientSU2 depending on structure
        results.push(run_vqe_test(&label, &c, None)); // check detection, don't assert type
    }

    // ========== True negative tests (non-VQE circuits) ==========

    // Trotter circuits (should NOT be detected as VQE)
    for &n in &[3, 4, 5, 6] {
        for &steps in &[1, 2, 3] {
            let c = gen_trotter_circuit(n, steps);
            let label = format!("neg_trotter_{}qb_{}steps", n, steps);
            results.push(run_vqe_test(&label, &c, None)); // None = expect no VQE detection
        }
    }

    // QAOA circuits
    for &n in &[3, 4, 5] {
        for &p in &[1, 2, 3] {
            let c = BenchmarkSuite::generate_qaoa(n, p);
            let label = format!("neg_qaoa_{}qb_p{}", n, p);
            results.push(run_vqe_test(&label, &c, None));
        }
    }

    // qDRIFT circuits
    for &n in &[3, 4] {
        for &samples in &[10, 30] {
            let c = BenchmarkSuite::generate_qdrift(n, samples);
            let label = format!("neg_qdrift_{}qb_n{}", n, samples);
            results.push(run_vqe_test(&label, &c, None));
        }
    }

    // Random rotation circuits
    for &n in &[3, 4, 5] {
        for &g in &[10, 30, 60] {
            let c = gen_random_rotations(n, g);
            let label = format!("neg_random_{}qb_{}gates", n, g);
            results.push(run_vqe_test(&label, &c, None));
        }
    }

    let elapsed_ms = experiment_start.elapsed().as_millis();

    // ---- Summary computation ----
    // Split into positive (expected VQE type) and negative (expected None)
    let positive_results: Vec<&Value> = results
        .iter()
        .filter(|r| r["expected_type"].as_str().is_some())
        .collect();
    let negative_results: Vec<&Value> = results
        .iter()
        .filter(|r| {
            r["expected_type"].is_null()
                && r["label"].as_str().is_some_and(|l| l.starts_with("neg_"))
        })
        .collect();
    let uccsd_results: Vec<&Value> = results
        .iter()
        .filter(|r| r["label"].as_str().is_some_and(|l| l.starts_with("uccsd_")))
        .collect();

    let pos_correct = positive_results
        .iter()
        .filter(|r| r["correct"].as_bool() == Some(true))
        .count();
    let neg_correct = negative_results
        .iter()
        .filter(|r| r["detected_type"].is_null())
        .count();
    let uccsd_detected = uccsd_results
        .iter()
        .filter(|r| r["detected_type"].as_str().is_some())
        .count();

    // Per ansatz type
    let ansatz_types = ["EfficientSU2", "RealAmplitudes"];
    let mut per_ansatz = Vec::new();
    for &atype in &ansatz_types {
        let subset: Vec<&&Value> = positive_results
            .iter()
            .filter(|r| r["expected_type"].as_str() == Some(atype))
            .collect();
        let sub_correct = subset
            .iter()
            .filter(|r| r["correct"].as_bool() == Some(true))
            .count();
        let avg_conf: f64 = subset
            .iter()
            .filter_map(|r| r["confidence"].as_f64())
            .sum::<f64>()
            / subset.len().max(1) as f64;
        per_ansatz.push(json!({
            "ansatz_type": atype,
            "total": subset.len(),
            "correct": sub_correct,
            "accuracy_pct": sub_correct as f64 / subset.len().max(1) as f64 * 100.0,
            "avg_confidence": avg_conf,
        }));
    }

    // Per qubit count for positive tests
    let mut per_qubit = Vec::new();
    for &n in &[3, 4, 5, 6, 8] {
        let subset: Vec<&&Value> = positive_results
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
        per_qubit.push(json!({
            "num_qubits": n,
            "total": subset.len(),
            "correct": sub_correct,
            "accuracy_pct": sub_correct as f64 / subset.len() as f64 * 100.0,
        }));
    }

    let report = json!({
        "experiment": "exp04_vqe_recognition",
        "description": "VQE ansatz recognition accuracy across EfficientSU2, RealAmplitudes, UCCSD with varying parameters; includes false-positive tests on non-VQE circuits",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "summary": {
            "total_tests": results.len(),
            "positive_tests": positive_results.len(),
            "positive_correct": pos_correct,
            "positive_accuracy_pct": pos_correct as f64 / positive_results.len().max(1) as f64 * 100.0,
            "negative_tests": negative_results.len(),
            "negative_correct": neg_correct,
            "false_positive_rate_pct": (1.0 - neg_correct as f64 / negative_results.len().max(1) as f64) * 100.0,
            "uccsd_tests": uccsd_results.len(),
            "uccsd_detected": uccsd_detected,
            "total_time_ms": elapsed_ms,
        },
        "per_ansatz_summary": per_ansatz,
        "per_qubit_summary": per_qubit,
        "detailed_results": results,
    });

    let output_path =
        "experiments/reverse_hamiltonian_extraction/reports/exp04_vqe_recognition.json";
    std::fs::create_dir_all("experiments/reverse_hamiltonian_extraction/reports").unwrap();
    std::fs::write(output_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    println!("Report written to {}", output_path);
    println!(
        "Positive accuracy: {}/{} ({:.1}%)",
        pos_correct,
        positive_results.len(),
        pos_correct as f64 / positive_results.len().max(1) as f64 * 100.0
    );
    println!(
        "False positive rate: {:.1}%",
        (1.0 - neg_correct as f64 / negative_results.len().max(1) as f64) * 100.0
    );
}
