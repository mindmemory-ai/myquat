#![allow(clippy::all)]
#![allow(unused_imports)]
#![allow(unused_mut)]
// Experiment 07: Noise Robustness of Hamiltonian Extraction
//
// Author: gA4ss
//
// Evaluates how extraction accuracy degrades under rotation angle noise.
// Simulates noise by perturbing gate rotation angles with Gaussian noise of
// varying standard deviations ($\sigma \in [0, 0.5]$). Tests on Ising and
// Heisenberg Hamiltonians with multiple qubit counts and Trotter steps.
// Measures: (1) coefficient reconstruction error vs noise, (2) Trotter
// order detection accuracy vs noise, (3) deoptimization confidence vs noise.
// Outputs a detailed JSON report.

use myquat::circuit::QuantumCircuit;
use myquat::deoptimization::DeoptimizationPipeline;
use myquat::gates::StandardGate;
use myquat::hamiltonian::{
    constructors, CircuitAnalyzer, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliOperator,
    PauliString, TrotterOrder,
};
use myquat::parameter::Parameter;
use num_complex::Complex64;
use serde_json::{json, Value};
use std::time::Instant;

/// Add Gaussian noise to rotation angles in a circuit.
/// Uses a deterministic pseudo-random sequence for reproducibility.
fn add_noise_to_circuit(circuit: &QuantumCircuit, sigma: f64, seed: u64) -> QuantumCircuit {
    let mut noisy = QuantumCircuit::new(circuit.num_qubits(), 0);
    let instructions = circuit.data().instructions();

    // Simple deterministic pseudo-random generator (xorshift64)
    let mut state = seed;
    let mut next_gaussian = |state: &mut u64| -> f64 {
        // Box-Muller transform from uniform pseudo-random
        let u1 = {
            *state ^= *state << 13;
            *state ^= *state >> 7;
            *state ^= *state << 17;
            (*state as f64) / (u64::MAX as f64)
        };
        let u2 = {
            *state ^= *state << 13;
            *state ^= *state >> 7;
            *state ^= *state << 17;
            (*state as f64) / (u64::MAX as f64)
        };
        let u1 = u1.max(1e-15); // avoid log(0)
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    };

    for inst in instructions {
        let qubits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();

        // Check if this is a parametric rotation gate
        let is_rotation = matches!(
            inst.gate.gate_type,
            StandardGate::Rx | StandardGate::Ry | StandardGate::Rz
        );

        if is_rotation && !inst.gate.parameters.is_empty() {
            if let Parameter::Float(theta) = &inst.gate.parameters[0] {
                let noise = sigma * next_gaussian(&mut state);
                let noisy_theta = theta + noise;
                match inst.gate.gate_type {
                    StandardGate::Rx => {
                        noisy.rx(qubits[0], Parameter::Float(noisy_theta)).unwrap();
                    }
                    StandardGate::Ry => {
                        noisy.ry(qubits[0], Parameter::Float(noisy_theta)).unwrap();
                    }
                    StandardGate::Rz => {
                        noisy.rz(qubits[0], Parameter::Float(noisy_theta)).unwrap();
                    }
                    _ => {}
                }
            }
        } else {
            // Reproduce non-rotation gates
            match inst.gate.gate_type {
                StandardGate::H => {
                    noisy.h(qubits[0]).unwrap();
                }
                StandardGate::X => {
                    noisy
                        .rx(qubits[0], Parameter::Float(std::f64::consts::PI))
                        .unwrap();
                }
                StandardGate::CX => {
                    noisy.cx(qubits[0], qubits[1]).unwrap();
                }
                StandardGate::Rx => {
                    if let Some(Parameter::Float(t)) = inst.gate.parameters.first() {
                        noisy.rx(qubits[0], Parameter::Float(*t)).unwrap();
                    }
                }
                StandardGate::Ry => {
                    if let Some(Parameter::Float(t)) = inst.gate.parameters.first() {
                        noisy.ry(qubits[0], Parameter::Float(*t)).unwrap();
                    }
                }
                StandardGate::Rz => {
                    if let Some(Parameter::Float(t)) = inst.gate.parameters.first() {
                        noisy.rz(qubits[0], Parameter::Float(*t)).unwrap();
                    }
                }
                _ => {
                    // For other gates, try to reproduce by type
                    // (simplified: skip unknown gates)
                }
            }
        }
    }
    noisy
}

/// Compare Hamiltonians and compute coefficient error
fn compute_coeff_error(original: &Hamiltonian, extracted: &Hamiltonian) -> (f64, f64, f64) {
    let mut orig_map = std::collections::HashMap::new();
    for term in &original.terms {
        let label = format!("{}", term.pauli_string);
        let coeff = term.effective_coefficient();
        orig_map
            .entry(label)
            .and_modify(|c: &mut Complex64| *c += coeff)
            .or_insert(coeff);
    }

    let mut ext_map = std::collections::HashMap::new();
    for term in &extracted.terms {
        let label = format!("{}", term.pauli_string);
        let coeff = term.effective_coefficient();
        ext_map
            .entry(label)
            .and_modify(|c: &mut Complex64| *c += coeff)
            .or_insert(coeff);
    }

    let mut total_error = 0.0;
    let mut max_error = 0.0_f64;
    let mut count = 0;

    for (label, &orig_c) in &orig_map {
        if orig_c.norm() < 1e-12 {
            continue;
        }
        let ext_c = ext_map
            .get(label)
            .copied()
            .unwrap_or(Complex64::new(0.0, 0.0));
        let error = (orig_c - ext_c).norm() / orig_c.norm();
        total_error += error;
        max_error = max_error.max(error);
        count += 1;
    }

    let avg_error = if count > 0 {
        total_error / count as f64
    } else {
        0.0
    };
    let match_rate = if !orig_map.is_empty() {
        count as f64
            / orig_map
                .values()
                .filter(|c| c.norm() > 1e-12)
                .count()
                .max(1) as f64
    } else {
        1.0
    };

    (avg_error, max_error, match_rate)
}

fn main() {
    let experiment_start = Instant::now();
    let mut results: Vec<Value> = Vec::new();

    let noise_levels: Vec<f64> = vec![0.0, 0.001, 0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.3, 0.5];
    let num_trials = 5; // Multiple trials per noise level for statistical stability

    // Hamiltonians to test
    let test_cases: Vec<(&str, usize, usize, TrotterOrder)> = vec![
        ("ising_4qb_1st_s2", 4, 2, TrotterOrder::First),
        ("ising_4qb_2nd_s2", 4, 2, TrotterOrder::Second),
        ("ising_6qb_1st_s2", 6, 2, TrotterOrder::First),
        ("heis_4qb_1st_s2", 4, 2, TrotterOrder::First),
        ("heis_4qb_2nd_s2", 4, 2, TrotterOrder::Second),
        ("heis_6qb_1st_s2", 6, 2, TrotterOrder::First),
    ];

    for (case_label, n, steps, order) in &test_cases {
        let ham = if case_label.starts_with("ising") {
            constructors::ising_model(*n, 1.0, 0.5).unwrap()
        } else {
            constructors::heisenberg_model(*n, 1.0, 1.0, 1.0).unwrap()
        };

        let config = CompilerConfig {
            trotter_order: order.clone(),
            trotter_steps: *steps,
            evolution_time: 1.0,
            group_commuting_terms: false,
            auto_optimize_grouping: false,
            apply_circuit_optimization: false,
            ..Default::default()
        };
        let compiler = HamiltonianCompiler::new(config);
        let clean_circuit = compiler.compile(&ham).unwrap();

        for &sigma in &noise_levels {
            let mut trial_coeff_errors = Vec::new();
            let mut trial_max_errors = Vec::new();
            let mut trial_match_rates = Vec::new();
            let mut trial_order_correct = Vec::new();
            let mut trial_order_conf = Vec::new();
            let mut trial_deopt_conf = Vec::new();

            for trial in 0..num_trials {
                let seed = (sigma * 1e6) as u64 + trial as u64 * 12345 + 42;
                let noisy_circuit = add_noise_to_circuit(&clean_circuit, sigma, seed);

                // Extraction
                let analyzer = CircuitAnalyzer::new();
                if let Ok(analysis) = analyzer.analyze(&noisy_circuit) {
                    let (avg_err, max_err, match_rate) =
                        compute_coeff_error(&ham, &analysis.hamiltonian);
                    trial_coeff_errors.push(avg_err);
                    trial_max_errors.push(max_err);
                    trial_match_rates.push(match_rate);

                    let expected_order = format!("{:?}", order);
                    let detected = analysis.trotter_order.as_ref().map(|o| format!("{:?}", o));
                    trial_order_correct.push(detected.as_deref() == Some(&expected_order));
                    trial_order_conf.push(analysis.order_confidence);
                }

                // Deoptimization confidence
                let pipeline = DeoptimizationPipeline::default();
                if let Ok(result) = pipeline.restore_detailed(&noisy_circuit) {
                    trial_deopt_conf.push(result.confidence);
                }
            }

            let n_trials = trial_coeff_errors.len();
            let avg_coeff_err = trial_coeff_errors.iter().sum::<f64>() / n_trials.max(1) as f64;
            let avg_max_err = trial_max_errors.iter().sum::<f64>() / n_trials.max(1) as f64;
            let avg_match_rate = trial_match_rates.iter().sum::<f64>() / n_trials.max(1) as f64;
            let order_accuracy =
                trial_order_correct.iter().filter(|&&c| c).count() as f64 / n_trials.max(1) as f64;
            let avg_order_conf = trial_order_conf.iter().sum::<f64>() / n_trials.max(1) as f64;
            let avg_deopt_conf =
                trial_deopt_conf.iter().sum::<f64>() / trial_deopt_conf.len().max(1) as f64;

            // Standard deviation of coefficient error
            let std_coeff_err = if n_trials > 1 {
                let mean = avg_coeff_err;
                (trial_coeff_errors
                    .iter()
                    .map(|e| (e - mean).powi(2))
                    .sum::<f64>()
                    / (n_trials - 1) as f64)
                    .sqrt()
            } else {
                0.0
            };

            results.push(json!({
                "case": case_label,
                "num_qubits": n,
                "trotter_order": format!("{:?}", order),
                "trotter_steps": steps,
                "noise_sigma": sigma,
                "num_trials": num_trials,
                "gate_count": clean_circuit.size(),
                "avg_coeff_error": avg_coeff_err,
                "std_coeff_error": std_coeff_err,
                "avg_max_coeff_error": avg_max_err,
                "avg_term_match_rate": avg_match_rate,
                "order_detection_accuracy": order_accuracy,
                "avg_order_confidence": avg_order_conf,
                "avg_deopt_confidence": avg_deopt_conf,
            }));
        }
    }

    let elapsed_ms = experiment_start.elapsed().as_millis();

    // ---- Summary: error vs noise level (aggregated across all cases) ----
    let mut noise_summary = Vec::new();
    for &sigma in &noise_levels {
        let subset: Vec<&Value> = results
            .iter()
            .filter(|r| (r["noise_sigma"].as_f64().unwrap_or(-1.0) - sigma).abs() < 1e-12)
            .collect();
        if subset.is_empty() {
            continue;
        }
        let avg_ce: f64 = subset
            .iter()
            .filter_map(|r| r["avg_coeff_error"].as_f64())
            .sum::<f64>()
            / subset.len() as f64;
        let avg_oda: f64 = subset
            .iter()
            .filter_map(|r| r["order_detection_accuracy"].as_f64())
            .sum::<f64>()
            / subset.len() as f64;
        let avg_dc: f64 = subset
            .iter()
            .filter_map(|r| r["avg_deopt_confidence"].as_f64())
            .sum::<f64>()
            / subset.len() as f64;

        noise_summary.push(json!({
            "noise_sigma": sigma,
            "num_cases": subset.len(),
            "avg_coeff_error": avg_ce,
            "avg_order_detection_accuracy": avg_oda,
            "avg_deopt_confidence": avg_dc,
        }));
    }

    // Per-case summary at sigma=0 vs sigma=0.1
    let mut case_comparison = Vec::new();
    for (case_label, _, _, _) in &test_cases {
        let clean = results.iter().find(|r| {
            r["case"].as_str() == Some(case_label)
                && r["noise_sigma"].as_f64().is_some_and(|s| s.abs() < 1e-12)
        });
        let noisy = results.iter().find(|r| {
            r["case"].as_str() == Some(case_label)
                && r["noise_sigma"]
                    .as_f64()
                    .is_some_and(|s| (s - 0.1).abs() < 1e-12)
        });
        if let (Some(c), Some(n)) = (clean, noisy) {
            case_comparison.push(json!({
                "case": case_label,
                "clean_coeff_error": c["avg_coeff_error"],
                "noisy_01_coeff_error": n["avg_coeff_error"],
                "clean_order_accuracy": c["order_detection_accuracy"],
                "noisy_01_order_accuracy": n["order_detection_accuracy"],
                "clean_deopt_confidence": c["avg_deopt_confidence"],
                "noisy_01_deopt_confidence": n["avg_deopt_confidence"],
            }));
        }
    }

    let report = json!({
        "experiment": "exp07_noise_robustness",
        "description": "Noise robustness of Hamiltonian extraction: coefficient error, Trotter order detection, and deoptimization confidence vs rotation angle noise sigma",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "parameters": {
            "noise_levels": noise_levels,
            "num_trials_per_point": num_trials,
            "test_cases": test_cases.iter().map(|(l, n, s, o)| json!({
                "label": l, "num_qubits": n, "steps": s, "order": format!("{:?}", o)
            })).collect::<Vec<_>>(),
        },
        "summary": {
            "total_data_points": results.len(),
            "total_time_ms": elapsed_ms,
        },
        "noise_level_summary": noise_summary,
        "clean_vs_noisy_comparison": case_comparison,
        "detailed_results": results,
    });

    let output_path =
        "experiments/reverse_hamiltonian_extraction/reports/exp07_noise_robustness.json";
    std::fs::create_dir_all("experiments/reverse_hamiltonian_extraction/reports").unwrap();
    std::fs::write(output_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    println!("Report written to {}", output_path);
    println!(
        "Total data points: {}",
        report["summary"]["total_data_points"]
    );
}
