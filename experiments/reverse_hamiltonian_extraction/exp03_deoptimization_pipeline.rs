#![allow(clippy::all)]
#![allow(unused_imports)]
#![allow(unused_mut)]
// Experiment 03: Deoptimization Pipeline Comprehensive Evaluation
//
// Author: gA4ss
//
// Evaluates the 5-strategy deoptimization pipeline across all circuit types
// (Hamiltonian simulation, VQE, QAOA, qDRIFT), measuring per-strategy
// confidence, cumulative confidence, early stopping behavior, restoration
// quality, and the effect of different threshold settings.
// Outputs a detailed JSON report.

use myquat::circuit::QuantumCircuit;
use myquat::deoptimization::{
    BenchmarkCircuit, BenchmarkSuite, DeoptStrategy, DeoptimizationPipeline,
    KakRestorationStrategy, OptimizationLevel, QdriftRestorationStrategy, TemplateMatchingStrategy,
    TemporalAnalysisStrategy, VqeRestorationStrategy,
};
use myquat::hamiltonian::{
    constructors, CircuitAnalyzer, CompilerConfig, HamiltonianCompiler, TrotterOrder,
};
use serde_json::{json, Value};
use std::time::Instant;

/// Run a detailed pipeline test on a single circuit
fn run_pipeline_test(label: &str, circuit: &QuantumCircuit, circuit_type: &str) -> Value {
    let pipeline = DeoptimizationPipeline::default();

    // Per-strategy analysis (without modification)
    let strategy_scores = pipeline.analyze(circuit);

    // Detailed restoration
    let start = Instant::now();
    let result = match pipeline.restore_detailed(circuit) {
        Ok(r) => r,
        Err(e) => {
            return json!({
                "label": label,
                "circuit_type": circuit_type,
                "status": "error",
                "error": format!("{}", e),
            });
        }
    };
    let restore_time_us = start.elapsed().as_micros();

    // Run extraction on restored circuit
    let analyzer = CircuitAnalyzer::new();
    let extraction = analyzer.analyze(&result.circuit).ok();

    json!({
        "label": label,
        "circuit_type": circuit_type,
        "status": "ok",
        "input_gate_count": circuit.size(),
        "input_num_qubits": circuit.num_qubits(),
        "restored_gate_count": result.circuit.size(),
        "overall_confidence": result.confidence,
        "strategies_applied": result.strategies_applied,
        "early_stopped": result.early_stopped,
        "strategy_confidences": result.strategy_confidences.iter()
            .map(|(name, conf)| json!({"strategy": name, "confidence": conf}))
            .collect::<Vec<_>>(),
        "pre_restoration_scores": strategy_scores.iter()
            .map(|(name, conf)| json!({"strategy": name, "confidence": conf}))
            .collect::<Vec<_>>(),
        "restore_time_us": restore_time_us,
        "extraction_result": extraction.as_ref().map(|a| json!({
            "num_terms": a.hamiltonian.num_terms(),
            "trotter_order": a.trotter_order.as_ref().map(|o| format!("{:?}", o)),
            "order_confidence": a.order_confidence,
        })),
    })
}

/// Test the effect of different confidence thresholds
fn run_threshold_scan(circuit: &QuantumCircuit, label: &str) -> Vec<Value> {
    let thresholds = [0.3, 0.5, 0.7, 0.85, 0.95, 1.0];
    let mut results = Vec::new();

    for &threshold in &thresholds {
        let pipeline = DeoptimizationPipeline::default_pipeline()
            .with_threshold(threshold)
            .with_early_stop(true);

        let start = Instant::now();
        let result = pipeline.restore_detailed(circuit);
        let elapsed_us = start.elapsed().as_micros();

        if let Ok(r) = result {
            results.push(json!({
                "label": label,
                "threshold": threshold,
                "confidence": r.confidence,
                "strategies_applied": r.strategies_applied,
                "early_stopped": r.early_stopped,
                "restored_gates": r.circuit.size(),
                "time_us": elapsed_us,
            }));
        }
    }
    results
}

/// Test each strategy in isolation
fn run_isolated_strategy_tests(
    circuit: &QuantumCircuit,
    label: &str,
    circuit_type: &str,
) -> Vec<Value> {
    let strategies: Vec<(Box<dyn DeoptStrategy>, &str)> = vec![
        (Box::new(KakRestorationStrategy::new()), "KAK"),
        (Box::new(TemplateMatchingStrategy::new()), "Template"),
        (Box::new(TemporalAnalysisStrategy::new()), "Temporal"),
        (Box::new(VqeRestorationStrategy::new()), "VQE"),
        (Box::new(QdriftRestorationStrategy::new()), "qDRIFT"),
    ];

    let mut results = Vec::new();
    for (strategy, name) in &strategies {
        let confidence = strategy.confidence(circuit);
        let start = Instant::now();
        let restored = strategy.apply(circuit);
        let elapsed_us = start.elapsed().as_micros();

        results.push(json!({
            "label": label,
            "circuit_type": circuit_type,
            "strategy": name,
            "confidence": confidence,
            "apply_time_us": elapsed_us,
            "success": restored.is_ok(),
            "restored_gates": restored.as_ref().map(|c| c.size()).ok(),
        }));
    }
    results
}

fn main() {
    let experiment_start = Instant::now();

    // ---- Generate diverse test circuits ----
    struct TestCircuit {
        label: String,
        circuit: QuantumCircuit,
        ctype: String,
    }

    let mut test_circuits: Vec<TestCircuit> = Vec::new();

    // Hamiltonian simulation circuits with varying sizes
    for &n in &[3, 4, 5, 6, 8] {
        for &steps in &[1, 2, 3] {
            let c = BenchmarkSuite::generate_hamiltonian_simulation(n, steps);
            test_circuits.push(TestCircuit {
                label: format!("ham_sim_{}qb_{}steps", n, steps),
                circuit: c,
                ctype: "hamiltonian_simulation".into(),
            });
        }
    }

    // VQE: EfficientSU2
    for &n in &[3, 4, 6, 8] {
        for &d in &[2, 3, 5] {
            let c = BenchmarkSuite::generate_vqe_ansatz(n, d);
            test_circuits.push(TestCircuit {
                label: format!("vqe_esu2_{}qb_d{}", n, d),
                circuit: c,
                ctype: "vqe_efficientsu2".into(),
            });
        }
    }

    // VQE: RealAmplitudes
    for &n in &[3, 4, 6] {
        for &d in &[2, 3, 5, 7] {
            let c = BenchmarkSuite::generate_vqe_real_amplitudes(n, d);
            test_circuits.push(TestCircuit {
                label: format!("vqe_real_{}qb_d{}", n, d),
                circuit: c,
                ctype: "vqe_realamplitudes".into(),
            });
        }
    }

    // VQE: UCCSD
    for &(n, ne) in &[(4, 2), (6, 2), (6, 3), (8, 2)] {
        let c = BenchmarkSuite::generate_vqe_uccsd(n, ne);
        test_circuits.push(TestCircuit {
            label: format!("vqe_uccsd_{}qb_{}e", n, ne),
            circuit: c,
            ctype: "vqe_uccsd".into(),
        });
    }

    // QAOA
    for &n in &[3, 4, 5, 6] {
        for &p in &[1, 2, 3] {
            let c = BenchmarkSuite::generate_qaoa(n, p);
            test_circuits.push(TestCircuit {
                label: format!("qaoa_{}qb_p{}", n, p),
                circuit: c,
                ctype: "qaoa".into(),
            });
        }
    }

    // qDRIFT
    for &n in &[3, 4, 6] {
        for &samples in &[10, 20, 40, 80] {
            let c = BenchmarkSuite::generate_qdrift(n, samples);
            test_circuits.push(TestCircuit {
                label: format!("qdrift_{}qb_n{}", n, samples),
                circuit: c,
                ctype: "qdrift".into(),
            });
        }
    }

    // qDRIFT with ZZ
    for &n in &[4, 6] {
        for &samples in &[20, 40] {
            let c = BenchmarkSuite::generate_qdrift_with_zz(n, samples);
            test_circuits.push(TestCircuit {
                label: format!("qdrift_zz_{}qb_n{}", n, samples),
                circuit: c,
                ctype: "qdrift_zz".into(),
            });
        }
    }

    // ---- Run pipeline tests ----
    let mut pipeline_results = Vec::new();
    let mut isolated_results = Vec::new();
    let mut threshold_results = Vec::new();

    for tc in &test_circuits {
        pipeline_results.push(run_pipeline_test(&tc.label, &tc.circuit, &tc.ctype));
        isolated_results.extend(run_isolated_strategy_tests(
            &tc.circuit,
            &tc.label,
            &tc.ctype,
        ));
    }

    // Threshold scan on representative circuits
    let representative = [
        "ham_sim_4qb_2steps",
        "vqe_esu2_4qb_d3",
        "qaoa_4qb_p2",
        "qdrift_4qb_n20",
    ];
    for &rep_label in &representative {
        if let Some(tc) = test_circuits.iter().find(|t| t.label == rep_label) {
            threshold_results.extend(run_threshold_scan(&tc.circuit, &tc.label));
        }
    }

    let elapsed_ms = experiment_start.elapsed().as_millis();

    // ---- Compute summaries ----
    let ok_pipeline: Vec<&Value> = pipeline_results
        .iter()
        .filter(|r| r["status"].as_str() == Some("ok"))
        .collect();

    // Per circuit-type summary
    let ctypes = [
        "hamiltonian_simulation",
        "vqe_efficientsu2",
        "vqe_realamplitudes",
        "vqe_uccsd",
        "qaoa",
        "qdrift",
        "qdrift_zz",
    ];
    let mut per_type = Vec::new();
    for &ct in &ctypes {
        let subset: Vec<&&Value> = ok_pipeline
            .iter()
            .filter(|r| r["circuit_type"].as_str() == Some(ct))
            .collect();
        if subset.is_empty() {
            continue;
        }
        let avg_conf: f64 = subset
            .iter()
            .filter_map(|r| r["overall_confidence"].as_f64())
            .sum::<f64>()
            / subset.len() as f64;
        let avg_strategies: f64 = subset
            .iter()
            .filter_map(|r| r["strategies_applied"].as_u64().map(|v| v as f64))
            .sum::<f64>()
            / subset.len() as f64;
        let early_stop_pct: f64 = subset
            .iter()
            .filter(|r| r["early_stopped"].as_bool() == Some(true))
            .count() as f64
            / subset.len() as f64
            * 100.0;
        let avg_time: f64 = subset
            .iter()
            .filter_map(|r| r["restore_time_us"].as_u64().map(|v| v as f64))
            .sum::<f64>()
            / subset.len() as f64;

        per_type.push(json!({
            "circuit_type": ct,
            "count": subset.len(),
            "avg_confidence": avg_conf,
            "avg_strategies_applied": avg_strategies,
            "early_stop_pct": early_stop_pct,
            "avg_restore_time_us": avg_time,
        }));
    }

    // Per-strategy isolation summary
    let strategy_names = ["KAK", "Template", "Temporal", "VQE", "qDRIFT"];
    let mut per_strategy = Vec::new();
    for &sname in &strategy_names {
        let subset: Vec<&Value> = isolated_results
            .iter()
            .filter(|r| r["strategy"].as_str() == Some(sname))
            .collect();
        let avg_conf: f64 = subset
            .iter()
            .filter_map(|r| r["confidence"].as_f64())
            .sum::<f64>()
            / subset.len().max(1) as f64;
        let avg_time: f64 = subset
            .iter()
            .filter_map(|r| r["apply_time_us"].as_u64().map(|v| v as f64))
            .sum::<f64>()
            / subset.len().max(1) as f64;
        // Per circuit type: avg confidence
        let mut conf_by_type = Vec::new();
        for &ct in &ctypes {
            let ct_subset: Vec<&&Value> = subset
                .iter()
                .filter(|r| r["circuit_type"].as_str() == Some(ct))
                .collect();
            if ct_subset.is_empty() {
                continue;
            }
            let ct_avg: f64 = ct_subset
                .iter()
                .filter_map(|r| r["confidence"].as_f64())
                .sum::<f64>()
                / ct_subset.len() as f64;
            conf_by_type.push(json!({"circuit_type": ct, "avg_confidence": ct_avg}));
        }
        per_strategy.push(json!({
            "strategy": sname,
            "total_tests": subset.len(),
            "avg_confidence": avg_conf,
            "avg_apply_time_us": avg_time,
            "confidence_by_circuit_type": conf_by_type,
        }));
    }

    let report = json!({
        "experiment": "exp03_deoptimization_pipeline",
        "description": "Comprehensive evaluation of the 5-strategy deoptimization pipeline across circuit types, with isolated strategy tests and threshold scans",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "summary": {
            "total_circuits": test_circuits.len(),
            "successful_pipeline_tests": ok_pipeline.len(),
            "avg_overall_confidence": ok_pipeline.iter()
                .filter_map(|r| r["overall_confidence"].as_f64())
                .sum::<f64>() / ok_pipeline.len().max(1) as f64,
            "total_time_ms": elapsed_ms,
        },
        "per_circuit_type_summary": per_type,
        "per_strategy_summary": per_strategy,
        "threshold_scan_results": threshold_results,
        "pipeline_detailed_results": pipeline_results,
        "isolated_strategy_results": isolated_results,
    });

    let output_path =
        "experiments/reverse_hamiltonian_extraction/reports/exp03_deoptimization_pipeline.json";
    std::fs::create_dir_all("experiments/reverse_hamiltonian_extraction/reports").unwrap();
    std::fs::write(output_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    println!("Report written to {}", output_path);
    println!("Total circuits tested: {}", test_circuits.len());
}
