// tests/test_deoptimization.rs - Integration tests for deoptimization module
// Author: gA4ss

use myquat::deoptimization::*;
use myquat::*;

#[test]
fn test_pipeline_creation() {
    let pipeline = DeoptimizationPipeline::new();
    assert_eq!(pipeline.num_strategies(), 0);
}

#[test]
fn test_default_pipeline() {
    let pipeline = DeoptimizationPipeline::default();
    assert_eq!(
        pipeline.num_strategies(),
        5,
        "Default pipeline should have 5 strategies"
    );
}

#[test]
fn test_pipeline_with_custom_threshold() {
    let pipeline = DeoptimizationPipeline::new().with_threshold(0.95);

    // Threshold is private, but we can test that it compiles and runs
    assert_eq!(pipeline.num_strategies(), 0);
}

#[test]
fn test_pipeline_restore_empty_circuit() {
    let circuit = QuantumCircuit::new(2, 2);
    let pipeline = DeoptimizationPipeline::default();

    let result = pipeline.restore(&circuit);
    assert!(result.is_ok());

    let (restored, confidence) = result.unwrap();
    assert_eq!(restored.num_qubits(), 2);
    assert_eq!(restored.num_clbits(), 2);
    // Empty circuit should have 0 confidence (nothing to restore)
    assert_eq!(confidence, 0.0);
}

#[test]
fn test_pipeline_restore_simple_circuit() {
    let mut circuit = QuantumCircuit::new(2, 2);
    circuit.h(0).unwrap();
    circuit.cx(0, 1).unwrap();

    let pipeline = DeoptimizationPipeline::default();
    let result = pipeline.restore(&circuit);

    assert!(result.is_ok());
    let (restored, _confidence) = result.unwrap();
    assert_eq!(restored.num_qubits(), 2);
}

#[test]
fn test_kak_strategy_creation() {
    let strategy = KakRestorationStrategy::new();
    assert_eq!(strategy.name(), "KAK Restoration");
}

#[test]
fn test_kak_strategy_with_confidence() {
    let strategy = KakRestorationStrategy::new().with_min_confidence(0.8);

    // Strategy should be created successfully
    assert_eq!(strategy.name(), "KAK Restoration");
}

#[test]
fn test_template_matching_strategy() {
    let strategy = TemplateMatchingStrategy::new();
    assert_eq!(strategy.name(), "Template Matching");
}

#[test]
fn test_template_matching_with_reordering() {
    let strategy = TemplateMatchingStrategy::new().with_reordering(false);

    assert_eq!(strategy.name(), "Template Matching");
}

#[test]
fn test_temporal_analysis_strategy() {
    let strategy = TemporalAnalysisStrategy::new();
    assert_eq!(strategy.name(), "Temporal Analysis");
}

#[test]
fn test_temporal_analysis_suzuki_coefficients() {
    // Test known Suzuki coefficients
    let coeff_4 = TemporalAnalysisStrategy::get_suzuki_coeff(4);
    assert!(coeff_4.is_some());
    assert!((coeff_4.unwrap() - 0.41449077179437573).abs() < 1e-10);

    let coeff_6 = TemporalAnalysisStrategy::get_suzuki_coeff(6);
    assert!(coeff_6.is_some());
    assert!((coeff_6.unwrap() - 0.37307216933481307).abs() < 1e-10);

    let coeff_8 = TemporalAnalysisStrategy::get_suzuki_coeff(8);
    assert!(coeff_8.is_some());

    let coeff_10 = TemporalAnalysisStrategy::get_suzuki_coeff(10);
    assert!(coeff_10.is_some());

    // Non-existent coefficient
    let coeff_invalid = TemporalAnalysisStrategy::get_suzuki_coeff(12);
    assert!(coeff_invalid.is_none());
}

#[test]
fn test_temporal_analysis_with_tolerance() {
    let strategy = TemporalAnalysisStrategy::new().with_tolerance(1e-8);

    assert_eq!(strategy.name(), "Temporal Analysis");
}

#[test]
fn test_pipeline_early_stop() {
    let pipeline = DeoptimizationPipeline::new()
        .with_early_stop(true)
        .with_threshold(0.9);

    assert_eq!(pipeline.num_strategies(), 0);
}

#[test]
fn test_pipeline_no_early_stop() {
    let pipeline = DeoptimizationPipeline::new().with_early_stop(false);

    assert_eq!(pipeline.num_strategies(), 0);
}

#[test]
fn test_multiple_strategies_in_pipeline() {
    let pipeline = DeoptimizationPipeline::new()
        .add_strategy(Box::new(KakRestorationStrategy::new()))
        .add_strategy(Box::new(TemplateMatchingStrategy::new()))
        .add_strategy(Box::new(TemporalAnalysisStrategy::new()));

    assert_eq!(pipeline.num_strategies(), 3);
}

#[test]
fn test_strategy_confidence_on_empty_circuit() {
    let circuit = QuantumCircuit::new(2, 2);

    let kak = KakRestorationStrategy::new();
    assert_eq!(kak.confidence(&circuit), 0.0);

    let template = TemplateMatchingStrategy::new();
    assert_eq!(template.confidence(&circuit), 0.0);

    let temporal = TemporalAnalysisStrategy::new();
    assert_eq!(temporal.confidence(&circuit), 0.0);
}
