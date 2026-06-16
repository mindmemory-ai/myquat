// Full H₂ Benchmark Comparison (100 Trotter steps)
// Author: gA4ss

use myquat::error::Result;
use myquat::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
use myquat::hamiltonian::{
    CompilationStrategy, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString,
    TrotterOrder,
};
use num_complex::Complex64;
use std::time::Instant;

fn create_h2_hamiltonian() -> Result<Hamiltonian> {
    let mut h = Hamiltonian::new(4);

    h.add_term(PauliString::from_str("IIII")?, Complex64::new(-0.8105, 0.0))?;
    h.add_term(PauliString::from_str("IIIZ")?, Complex64::new(0.1721, 0.0))?;
    h.add_term(PauliString::from_str("IIZI")?, Complex64::new(-0.2228, 0.0))?;
    h.add_term(PauliString::from_str("IZII")?, Complex64::new(0.1721, 0.0))?;
    h.add_term(PauliString::from_str("ZIII")?, Complex64::new(-0.2228, 0.0))?;
    h.add_term(PauliString::from_str("IIZZ")?, Complex64::new(0.1686, 0.0))?;
    h.add_term(PauliString::from_str("IZIZ")?, Complex64::new(0.1205, 0.0))?;
    h.add_term(PauliString::from_str("IZZI")?, Complex64::new(0.1686, 0.0))?;
    h.add_term(PauliString::from_str("ZIIZ")?, Complex64::new(0.1686, 0.0))?;
    h.add_term(PauliString::from_str("ZIZI")?, Complex64::new(0.1205, 0.0))?;
    h.add_term(PauliString::from_str("ZZII")?, Complex64::new(0.1686, 0.0))?;
    h.add_term(PauliString::from_str("IIXX")?, Complex64::new(0.0454, 0.0))?;
    h.add_term(PauliString::from_str("IIYY")?, Complex64::new(0.0454, 0.0))?;
    h.add_term(PauliString::from_str("IXIX")?, Complex64::new(0.0454, 0.0))?;
    h.add_term(PauliString::from_str("IYIY")?, Complex64::new(0.0454, 0.0))?;

    Ok(h)
}

fn main() -> Result<()> {
    println!("\n{}", "=".repeat(70));
    println!("H₂ Molecule Full Benchmark (100 Trotter Steps)");
    println!("{}", "=".repeat(70));

    let hamiltonian = create_h2_hamiltonian()?;
    println!(
        "\nHamiltonian: {} terms, {} qubits",
        hamiltonian.terms.len(),
        hamiltonian.num_qubits
    );

    // Test 1: Without optimization
    println!("\n--- Test 1: Without Circuit Optimization ---");
    let config_no_opt = CompilerConfig {
        trotter_order: TrotterOrder::Second,
        trotter_steps: 100,
        evolution_time: 1.0,
        adaptive: false,
        adaptive_tolerance: 1e-3,
        min_step_size: 1e-6,
        max_step_size: 1.0,
        hbar: 1.0,
        skip_identities: true,
        group_commuting_terms: true,
        auto_optimize_grouping: true,
        layout_aware_grouping: false,
        apply_circuit_optimization: false,
        optimization_strategy: CompilationStrategy::GateLevel,
        cross_step_synthesis: false,
        block_grouping_strategy: BlockGroupingStrategy::QWC,
        pauli_gadget_optimization: myquat::hamiltonian::GadgetOptimizationStrategy::IdenticalOnly,
        alternate_reverse_steps: true,
        clifford_enhanced_blocks: false,
        ..Default::default()
    };

    let compiler = HamiltonianCompiler::new(config_no_opt);
    let start = Instant::now();
    let circuit_unopt = compiler.compile(&hamiltonian)?;
    let time_unopt = start.elapsed();

    println!(
        "Compilation time: {:.2} ms",
        time_unopt.as_secs_f64() * 1000.0
    );
    println!("Gates: {}", circuit_unopt.size());
    println!("Depth: {}", circuit_unopt.depth());

    // Test 2: With Phase 3 optimization (Level 2)
    println!("\n--- Test 2: With Phase 3 Optimization (Level 2) ---");
    let config_opt = CompilerConfig {
        trotter_order: TrotterOrder::Second,
        trotter_steps: 100,
        evolution_time: 1.0,
        adaptive: false,
        adaptive_tolerance: 1e-3,
        min_step_size: 1e-6,
        max_step_size: 1.0,
        hbar: 1.0,
        skip_identities: true,
        group_commuting_terms: true,
        auto_optimize_grouping: true,
        layout_aware_grouping: false,
        apply_circuit_optimization: true,
        optimization_strategy: CompilationStrategy::GateLevel,
        cross_step_synthesis: false,
        block_grouping_strategy: BlockGroupingStrategy::QWC,
        pauli_gadget_optimization: myquat::hamiltonian::GadgetOptimizationStrategy::IdenticalOnly,
        alternate_reverse_steps: true,
        clifford_enhanced_blocks: false,
        ..Default::default()
    };

    let compiler_opt = HamiltonianCompiler::new(config_opt);
    let start = Instant::now();
    let circuit_opt = compiler_opt.compile(&hamiltonian)?;
    let time_opt = start.elapsed();

    println!(
        "Compilation time: {:.2} ms",
        time_opt.as_secs_f64() * 1000.0
    );
    println!("Gates: {}", circuit_opt.size());
    println!("Depth: {}", circuit_opt.depth());

    // Summary
    println!("\n{}", "=".repeat(70));
    println!("Summary:");
    println!("{}", "=".repeat(70));

    let gate_reduction = if circuit_unopt.size() > 0 {
        ((circuit_unopt.size() - circuit_opt.size()) as f64 / circuit_unopt.size() as f64) * 100.0
    } else {
        0.0
    };

    let depth_reduction = if circuit_unopt.depth() > 0 {
        ((circuit_unopt.depth() - circuit_opt.depth()) as f64 / circuit_unopt.depth() as f64)
            * 100.0
    } else {
        0.0
    };

    println!(
        "\nUnoptimized:  {} gates, {} depth",
        circuit_unopt.size(),
        circuit_unopt.depth()
    );
    println!(
        "Optimized:    {} gates, {} depth",
        circuit_opt.size(),
        circuit_opt.depth()
    );
    println!(
        "\nReduction:    {:.1}% gates, {:.1}% depth",
        gate_reduction, depth_reduction
    );
    println!(
        "Compile time: {:.2} ms -> {:.2} ms",
        time_unopt.as_secs_f64() * 1000.0,
        time_opt.as_secs_f64() * 1000.0
    );

    println!("\n{}", "=".repeat(70));

    Ok(())
}
