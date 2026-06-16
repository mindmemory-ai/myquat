//! Test TrotterAwarePass with and without step boundaries
//! Author: gA4ss

use myquat::error::Result;
use myquat::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
use myquat::hamiltonian::{
    CompilationStrategy, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString,
    TrotterOrder,
};
use myquat::parameter::Parameter;
use num_complex::Complex64;

fn main() -> Result<()> {
    let mut hamiltonian = Hamiltonian::new(4);
    let terms = [
        ("IIII", -0.8105),
        ("IIIZ", 0.1721),
        ("IIZI", -0.2228),
        ("IZII", 0.1721),
        ("ZIII", -0.2228),
        ("IIZZ", 0.1686),
        ("IZIZ", 0.1205),
        ("IZZI", 0.1686),
        ("ZIIZ", 0.1686),
        ("ZIZI", 0.1205),
        ("ZZII", 0.1686),
        ("IIXX", 0.0454),
        ("IIYY", 0.0454),
        ("XXII", 0.0454),
        ("YYII", 0.0454),
    ];
    for (ps, coeff) in &terms {
        hamiltonian.add_term(PauliString::from_str(ps)?, Complex64::new(*coeff, 0.0))?;
    }

    // Test 1: With TrotterAwarePass (default, runs automatically before optimization)
    let config = CompilerConfig {
        trotter_order: TrotterOrder::First,
        trotter_steps: 10,
        evolution_time: 1.0,
        adaptive: false,
        adaptive_tolerance: 1e-3,
        min_step_size: 1e-6,
        max_step_size: 1.0,
        hbar: 1.0,
        skip_identities: true,
        group_commuting_terms: true,
        apply_circuit_optimization: false, // No optimization — just TrotterAwarePass
        auto_optimize_grouping: true,
        layout_aware_grouping: false,
        optimization_strategy: CompilationStrategy::PauliLevel,
        cross_step_synthesis: false,
        block_grouping_strategy: BlockGroupingStrategy::QWC,
        pauli_gadget_optimization: myquat::hamiltonian::GadgetOptimizationStrategy::IdenticalOnly,
        alternate_reverse_steps: true,
        clifford_enhanced_blocks: false,
        ..Default::default()
    };

    // Compile WITHOUT optimization to see raw + TrotterAware effect
    let compiler = HamiltonianCompiler::new(config.clone());
    let circuit = compiler.compile(&hamiltonian)?;
    let cx: usize = circuit
        .data()
        .instructions()
        .iter()
        .filter(|i| matches!(i.gate.gate_type, myquat::gates::StandardGate::CX))
        .count();
    println!(
        "RAW + TrotterAwarePass: {} gates, {} CX, depth: {}",
        circuit.size(),
        cx,
        circuit.depth()
    );

    // Test 2: Check if step_boundaries metadata exists
    let metadata = circuit.data().metadata();
    let boundaries = metadata.get("step_boundaries");
    println!("Step boundaries metadata: {:?}", boundaries);

    // Test 3: Manual test — build two identical steps and check cancellation
    println!("\n=== Manual step-boundary test ===");
    let mut test = myquat::QuantumCircuit::new(2, 0);
    // Step 1: H(0) - CX(0,1) - Rz(1, 0.5) - CX(0,1) - H(0)
    test.h(0)?;
    test.cx(0, 1)?;
    test.rz(1, Parameter::Float(0.5))?;
    test.cx(0, 1)?;
    test.h(0)?;
    let boundary1 = test.size(); // 5 gates
                                 // Step 2: Same thing
    test.h(0)?;
    test.cx(0, 1)?;
    test.rz(1, Parameter::Float(0.3))?;
    test.cx(0, 1)?;
    test.h(0)?;
    let boundary2 = test.size(); // 10 gates

    println!("Before TrotterAwarePass: {} gates", test.size());
    // Set step boundaries
    test.data_mut().set_metadata(
        "step_boundaries".to_string(),
        format!("{},{}", boundary1, boundary2),
    );

    // Run TrotterAwarePass
    use myquat::circuit_optimization::{CircuitPass, TrotterAwarePass};
    TrotterAwarePass::new().run(&mut test)?;

    println!("After TrotterAwarePass: {} gates", test.size());
    println!("Expected: H(0) at end of step 1 cancels with H(0) at start of step 2 -> 8 gates");
    // Should be 8: H-CX-Rz-CX-H-CX-Rz-CX-H (the two middle H cancel)
    // Actually: H at pos 4 (last of step 1) and H at pos 5 (first of step 2) cancel
    // Result: 8 gates

    for (i, inst) in test.data().instructions().iter().enumerate() {
        let qbits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();
        println!("  {}: {:?} qubits={:?}", i, inst.gate.gate_type, qbits);
    }

    Ok(())
}
