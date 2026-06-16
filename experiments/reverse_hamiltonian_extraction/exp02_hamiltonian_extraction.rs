#![allow(clippy::all)]
#![allow(unused_imports)]
#![allow(unused_mut)]
// Experiment 02: Hamiltonian Coefficient Extraction Accuracy
//
// Author: gA4ss
//
// Round-trip test: Hamiltonian -> compile to circuit -> extract Hamiltonian -> compare.
// Measures coefficient reconstruction error, term matching rate, and fidelity
// across Ising, Heisenberg, and custom Hamiltonians with varying parameters.
// Outputs a detailed JSON report.

use myquat::hamiltonian::{
    constructors, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliOperator, PauliString,
    TrotterOrder,
};
use num_complex::Complex64;
use serde_json::{json, Value};
use std::time::Instant;

/// Compare two Hamiltonians and return metrics
fn compare_hamiltonians(original: &Hamiltonian, extracted: &Hamiltonian) -> Value {
    let orig_terms = &original.terms;
    let ext_terms = &extracted.terms;

    // Build map: pauli_string_label -> coefficient for extracted
    let mut ext_map = std::collections::HashMap::new();
    for term in ext_terms {
        let label = format!("{}", term.pauli_string);
        let coeff = term.effective_coefficient();
        ext_map
            .entry(label)
            .and_modify(|c: &mut Complex64| *c += coeff)
            .or_insert(coeff);
    }

    let mut orig_map = std::collections::HashMap::new();
    for term in orig_terms {
        let label = format!("{}", term.pauli_string);
        let coeff = term.effective_coefficient();
        orig_map
            .entry(label)
            .and_modify(|c: &mut Complex64| *c += coeff)
            .or_insert(coeff);
    }

    // Term matching
    let mut matched = 0;
    let mut total_coeff_error = 0.0;
    let mut max_coeff_error = 0.0_f64;
    let mut term_details = Vec::new();

    for (label, &orig_c) in &orig_map {
        if orig_c.norm() < 1e-12 {
            continue;
        }
        if let Some(&ext_c) = ext_map.get(label) {
            let error = (orig_c - ext_c).norm() / orig_c.norm().max(1e-12);
            total_coeff_error += error;
            max_coeff_error = max_coeff_error.max(error);
            matched += 1;
            term_details.push(json!({
                "pauli": label,
                "original_re": orig_c.re,
                "original_im": orig_c.im,
                "extracted_re": ext_c.re,
                "extracted_im": ext_c.im,
                "relative_error": error,
                "matched": true,
            }));
        } else {
            term_details.push(json!({
                "pauli": label,
                "original_re": orig_c.re,
                "original_im": orig_c.im,
                "extracted_re": null,
                "relative_error": 1.0,
                "matched": false,
            }));
            total_coeff_error += 1.0;
            max_coeff_error = max_coeff_error.max(1.0);
        }
    }

    // Spurious terms in extracted but not in original
    let mut spurious = 0;
    for (label, &ext_c) in &ext_map {
        if ext_c.norm() < 1e-12 {
            continue;
        }
        if !orig_map.contains_key(label) {
            spurious += 1;
        }
    }

    let orig_nonzero = orig_map.values().filter(|c| c.norm() > 1e-12).count();
    let avg_coeff_error = if orig_nonzero > 0 {
        total_coeff_error / orig_nonzero as f64
    } else {
        0.0
    };
    let match_rate = if orig_nonzero > 0 {
        matched as f64 / orig_nonzero as f64
    } else {
        1.0
    };

    json!({
        "original_terms": orig_nonzero,
        "extracted_terms": ext_map.values().filter(|c| c.norm() > 1e-12).count(),
        "matched_terms": matched,
        "spurious_terms": spurious,
        "term_match_rate": match_rate,
        "avg_relative_coeff_error": avg_coeff_error,
        "max_relative_coeff_error": max_coeff_error,
        "term_details": term_details,
    })
}

fn run_roundtrip_test(
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
                "status": "analysis_error",
                "error": format!("{}", e),
                "gate_count": circuit.size(),
            });
        }
    };
    let analysis_time_us = start.elapsed().as_micros();

    let comparison = compare_hamiltonians(hamiltonian, &analysis.hamiltonian);

    json!({
        "label": label,
        "status": "ok",
        "trotter_order": format!("{:?}", order),
        "trotter_steps": steps,
        "evolution_time": evolution_time,
        "num_qubits": hamiltonian.num_qubits,
        "gate_count": circuit.size(),
        "compile_time_us": compile_time_us,
        "analysis_time_us": analysis_time_us,
        "detected_order": analysis.trotter_order.as_ref().map(|o| format!("{:?}", o)),
        "order_confidence": analysis.order_confidence,
        "comparison": comparison,
    })
}

fn main() {
    let experiment_start = Instant::now();
    let mut results = Vec::new();

    // ---- Ising models with varying J, h ----
    let ising_params: Vec<(f64, f64)> =
        vec![(0.5, 0.1), (1.0, 0.5), (1.0, 1.0), (2.0, 0.3), (0.1, 2.0)];
    for &(j, h) in &ising_params {
        for &n in &[3, 4, 5, 6] {
            let ham = constructors::ising_model(n, j, h).unwrap();
            for &steps in &[1, 2, 4] {
                let label = format!("ising_j{}_h{}_{}qb_steps{}", j, h, n, steps);
                results.push(run_roundtrip_test(
                    &label,
                    &ham,
                    TrotterOrder::First,
                    steps,
                    1.0,
                ));
            }
        }
    }

    // ---- Heisenberg models with varying Jx, Jy, Jz ----
    let heis_params: Vec<(f64, f64, f64)> = vec![
        (1.0, 1.0, 1.0), // isotropic
        (1.0, 1.0, 0.0), // XY model
        (0.0, 0.0, 1.0), // pure ZZ
        (1.5, 0.5, 1.0), // anisotropic
    ];
    for &(jx, jy, jz) in &heis_params {
        for &n in &[3, 4, 5] {
            let ham = constructors::heisenberg_model(n, jx, jy, jz).unwrap();
            for &steps in &[1, 2, 4] {
                let label = format!("heis_jx{}_jy{}_jz{}_{}qb_steps{}", jx, jy, jz, n, steps);
                results.push(run_roundtrip_test(
                    &label,
                    &ham,
                    TrotterOrder::First,
                    steps,
                    1.0,
                ));
            }
        }
    }

    // ---- Custom Hamiltonians: multi-body terms ----
    for &n in &[4, 5, 6] {
        let mut ham = Hamiltonian::new(n);
        // ZZ chain
        for i in 0..n - 1 {
            let mut ops = vec![PauliOperator::I; n];
            ops[i] = PauliOperator::Z;
            ops[i + 1] = PauliOperator::Z;
            ham.add_term(
                PauliString::new(ops, Complex64::new(1.0, 0.0)),
                Complex64::new(-0.7, 0.0),
            )
            .unwrap();
        }
        // X field
        for i in 0..n {
            let mut ops = vec![PauliOperator::I; n];
            ops[i] = PauliOperator::X;
            ham.add_term(
                PauliString::new(ops, Complex64::new(1.0, 0.0)),
                Complex64::new(-0.3, 0.0),
            )
            .unwrap();
        }
        // Y field (small)
        for i in 0..n {
            let mut ops = vec![PauliOperator::I; n];
            ops[i] = PauliOperator::Y;
            ham.add_term(
                PauliString::new(ops, Complex64::new(1.0, 0.0)),
                Complex64::new(-0.1, 0.0),
            )
            .unwrap();
        }

        for &steps in &[1, 2, 4] {
            let label = format!("custom_zz_x_y_{}qb_steps{}", n, steps);
            results.push(run_roundtrip_test(
                &label,
                &ham,
                TrotterOrder::First,
                steps,
                1.0,
            ));
        }
    }

    // ---- Higher-order round-trips ----
    let orders: Vec<(TrotterOrder, &str)> =
        vec![(TrotterOrder::First, "1st"), (TrotterOrder::Second, "2nd")];
    for (order, olabel) in &orders {
        let ham = constructors::ising_model(4, 1.0, 0.5).unwrap();
        for &steps in &[1, 2, 3] {
            for &t in &[0.5, 1.0, 2.0] {
                let label = format!("roundtrip_ising_4qb_{}_s{}_t{}", olabel, steps, t);
                results.push(run_roundtrip_test(&label, &ham, order.clone(), steps, t));
            }
        }
    }

    // ---- Varying evolution time ----
    let ham = constructors::heisenberg_model(4, 1.0, 1.0, 1.0).unwrap();
    for &t in &[0.01, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0] {
        let label = format!("time_scan_heis_4qb_t{}", t);
        results.push(run_roundtrip_test(&label, &ham, TrotterOrder::First, 2, t));
    }

    let elapsed_ms = experiment_start.elapsed().as_millis();

    // Summary statistics
    let ok_results: Vec<&Value> = results
        .iter()
        .filter(|r| r["status"].as_str() == Some("ok"))
        .collect();

    let avg_match_rate: f64 = ok_results
        .iter()
        .filter_map(|r| r["comparison"]["term_match_rate"].as_f64())
        .sum::<f64>()
        / ok_results.len().max(1) as f64;
    let avg_coeff_error: f64 = ok_results
        .iter()
        .filter_map(|r| r["comparison"]["avg_relative_coeff_error"].as_f64())
        .sum::<f64>()
        / ok_results.len().max(1) as f64;
    let max_coeff_error: f64 = ok_results
        .iter()
        .filter_map(|r| r["comparison"]["max_relative_coeff_error"].as_f64())
        .fold(0.0, f64::max);

    // Per-hamiltonian-type
    let ham_types = ["ising", "heis", "custom", "roundtrip", "time_scan"];
    let mut per_type_summary = Vec::new();
    for &htype in &ham_types {
        let subset: Vec<&&Value> = ok_results
            .iter()
            .filter(|r| r["label"].as_str().is_some_and(|l| l.starts_with(htype)))
            .collect();
        if subset.is_empty() {
            continue;
        }
        let avg_mr: f64 = subset
            .iter()
            .filter_map(|r| r["comparison"]["term_match_rate"].as_f64())
            .sum::<f64>()
            / subset.len() as f64;
        let avg_ce: f64 = subset
            .iter()
            .filter_map(|r| r["comparison"]["avg_relative_coeff_error"].as_f64())
            .sum::<f64>()
            / subset.len() as f64;
        per_type_summary.push(json!({
            "type": htype,
            "count": subset.len(),
            "avg_term_match_rate": avg_mr,
            "avg_coeff_error": avg_ce,
        }));
    }

    let report = json!({
        "experiment": "exp02_hamiltonian_extraction",
        "description": "Round-trip Hamiltonian extraction accuracy: compile -> analyze -> compare coefficients",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "summary": {
            "total_tests": results.len(),
            "successful_tests": ok_results.len(),
            "avg_term_match_rate": avg_match_rate,
            "avg_relative_coeff_error": avg_coeff_error,
            "max_relative_coeff_error": max_coeff_error,
            "total_time_ms": elapsed_ms,
        },
        "per_type_summary": per_type_summary,
        "detailed_results": results,
    });

    let output_path =
        "experiments/reverse_hamiltonian_extraction/reports/exp02_hamiltonian_extraction.json";
    std::fs::create_dir_all("experiments/reverse_hamiltonian_extraction/reports").unwrap();
    std::fs::write(output_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    println!("Report written to {}", output_path);
    println!(
        "Avg term match rate: {:.1}%, Avg coeff error: {:.4}",
        avg_match_rate * 100.0,
        avg_coeff_error
    );
}
