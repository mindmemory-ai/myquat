//! Circuit optimization integration tests
//!
//! Author: gA4ss
//!
//! Integration tests for circuit optimization passes including
//! template matching, SWAP routing, and global optimization.

use myquat::circuit_optimization::{
    CircuitPass, GlobalOptimizationPass, MergeRotationsPass, SwapRoutingPass, TemplateMatchingPass,
};
use myquat::{Parameter, QuantumCircuit, Result};

#[test]
fn test_template_matching_integration() -> Result<()> {
    // Create a circuit with H-CX-H pattern
    let mut circuit = QuantumCircuit::new(2, 0);
    circuit.h(1)?;
    circuit.cx(0, 1)?;
    circuit.h(1)?;

    let original_size = circuit.size();
    assert_eq!(original_size, 3);

    // Apply template matching
    let pass = TemplateMatchingPass::new();
    pass.run(&mut circuit)?;

    // Should optimize (reduce gate count or replace with equivalent)
    // The exact result depends on template implementation
    assert!(circuit.size() <= original_size);

    Ok(())
}

#[test]
fn test_rotation_merging_integration() -> Result<()> {
    // Create circuit with consecutive rotations
    let mut circuit = QuantumCircuit::new(1, 0);
    circuit.rz(0, Parameter::Float(std::f64::consts::PI / 4.0))?;
    circuit.rz(0, Parameter::Float(std::f64::consts::PI / 4.0))?;
    circuit.rz(0, Parameter::Float(std::f64::consts::PI / 4.0))?;

    let original_size = circuit.size();
    assert_eq!(original_size, 3);

    // Apply rotation merging
    let pass = MergeRotationsPass::new();
    pass.run(&mut circuit)?;

    // Should merge to single rotation
    assert_eq!(circuit.size(), 1);

    Ok(())
}

#[test]
fn test_swap_routing_integration() -> Result<()> {
    // Create circuit requiring SWAP on linear topology
    let mut circuit = QuantumCircuit::new(4, 0);
    circuit.cx(0, 3)?; // Non-adjacent on linear topology

    let original_size = circuit.size();

    // Apply SWAP routing
    let pass = SwapRoutingPass::linear(4);
    pass.run(&mut circuit)?;

    // Should insert SWAPs
    assert!(circuit.size() > original_size);

    Ok(())
}

#[test]
fn test_global_optimization_integration() -> Result<()> {
    // Create circuit with reorderable gates
    let mut circuit = QuantumCircuit::new(3, 0);
    circuit.h(0)?;
    circuit.h(1)?;
    circuit.h(2)?;
    circuit.cx(0, 1)?;
    circuit.cx(1, 2)?;

    let original_size = circuit.size();

    // Apply global optimization
    let pass = GlobalOptimizationPass::new();
    pass.run(&mut circuit)?;

    // Gate count should remain same (reordering doesn't reduce gates)
    assert_eq!(circuit.size(), original_size);

    Ok(())
}

#[test]
fn test_optimization_pipeline() -> Result<()> {
    // Create a complex circuit
    let mut circuit = QuantumCircuit::new(3, 0);

    // Add H-CX-H pattern
    circuit.h(1)?;
    circuit.cx(0, 1)?;
    circuit.h(1)?;

    // Add consecutive rotations
    circuit.rz(0, Parameter::Float(std::f64::consts::PI / 4.0))?;
    circuit.rz(0, Parameter::Float(std::f64::consts::PI / 4.0))?;

    // Add more gates
    circuit.h(2)?;
    circuit.cx(1, 2)?;

    let original_size = circuit.size();

    // Apply optimization pipeline
    let passes: Vec<Box<dyn CircuitPass>> = vec![
        Box::new(TemplateMatchingPass::new()),
        Box::new(MergeRotationsPass::new()),
        Box::new(GlobalOptimizationPass::new()),
    ];

    for pass in passes {
        pass.run(&mut circuit)?;
    }

    // Should be optimized
    assert!(circuit.size() < original_size);

    Ok(())
}

#[test]
fn test_sparse_state_with_circuit() -> Result<()> {
    use myquat::quantum_info::SparseQuantumState;

    // Create a sparse state
    let sparse = SparseQuantumState::zero_state(10);

    // Verify properties
    assert_eq!(sparse.num_qubits(), 10);
    assert_eq!(sparse.num_nonzero(), 1);
    assert!(sparse.sparsity() < 0.01);

    // Memory should be minimal
    assert!(sparse.memory_usage() < 1000);

    Ok(())
}

#[test]
fn test_state_compression_workflow() -> Result<()> {
    use myquat::quantum_info::{QuantumState, StateCompression};

    // Create a state
    let state = QuantumState::zero_state(8);

    // Compress it
    let (sparse, stats) = StateCompression::lossless_compress(&state);

    // Verify compression
    assert_eq!(stats.fidelity, 1.0);
    assert!(stats.compression_ratio < 0.01);
    assert_eq!(sparse.num_nonzero(), 1);

    Ok(())
}

#[test]
fn test_adaptive_compression_workflow() -> Result<()> {
    use myquat::quantum_info::{QuantumState, StateCompression};
    use ndarray::Array1;
    use num_complex::Complex64;

    // Create a state with varying amplitudes
    let mut amplitudes = Array1::zeros(16);
    amplitudes[0] = Complex64::new(0.9, 0.0);
    amplitudes[1] = Complex64::new(0.3, 0.0);
    amplitudes[2] = Complex64::new(0.2, 0.0);
    amplitudes[3] = Complex64::new(0.1, 0.0);
    let state = QuantumState::new_normalized(amplitudes)?;

    // Adaptive compression
    let result = StateCompression::adaptive_compress(&state, 0.95, 20)?;
    let (sparse, stats) = result;

    // Verify fidelity maintained
    assert!(stats.fidelity >= 0.95);
    assert!(sparse.num_nonzero() <= state.dim());

    Ok(())
}

#[test]
fn test_end_to_end_optimization() -> Result<()> {
    // End-to-end test: create, optimize, verify
    let mut circuit = QuantumCircuit::new(4, 0);

    // Build a realistic circuit
    circuit.h(0)?;
    circuit.h(1)?;
    circuit.cx(0, 1)?;
    circuit.rz(0, Parameter::Float(std::f64::consts::PI / 2.0))?;
    circuit.rz(0, Parameter::Float(std::f64::consts::PI / 4.0))?;
    circuit.h(1)?;
    circuit.cx(0, 1)?;
    circuit.h(1)?;

    let original_size = circuit.size();

    // Optimize
    let template_pass = TemplateMatchingPass::new();
    template_pass.run(&mut circuit)?;

    let merge_pass = MergeRotationsPass::new();
    merge_pass.run(&mut circuit)?;

    // Verify optimization occurred
    assert!(circuit.size() < original_size);

    Ok(())
}
