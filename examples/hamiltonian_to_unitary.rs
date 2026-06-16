//! MyQuat Hamiltonian to QASM Exporter
//! Author: gA4ss
//!
//! Compiles Hamiltonian and exports circuit to QASM for verification.

use myquat::hamiltonian::{
    CompilationStrategy, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString,
    TrotterOrder,
};
use myquat::Result;
use num_complex::Complex64;
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <num_qubits> <pauli_term1:coeff1> ...", args[0]);
        std::process::exit(1);
    }

    let num_qubits: usize = args[1].parse().expect("Invalid number of qubits");

    // Create Hamiltonian
    let mut hamiltonian = Hamiltonian::new(num_qubits);

    // Parse and add terms
    for term_str in &args[2..] {
        let parts: Vec<&str> = term_str.split(':').collect();
        if parts.len() != 2 {
            continue;
        }

        let pauli_str = parts[0];
        let coeff: f64 = parts[1]
            .parse()
            .expect(&format!("Invalid coefficient: {}", parts[1]));

        hamiltonian.add_term(
            PauliString::from_str(pauli_str)?,
            Complex64::new(coeff, 0.0),
        )?;
    }

    // Compile circuit
    let config = CompilerConfig {
        trotter_order: TrotterOrder::First,
        trotter_steps: 1,
        evolution_time: 0.1,
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
        block_grouping_strategy: myquat::hamiltonian::pauli_synthesis::BlockGroupingStrategy::QWC,
        pauli_gadget_optimization: myquat::hamiltonian::GadgetOptimizationStrategy::IdenticalOnly,
        alternate_reverse_steps: true,
        clifford_enhanced_blocks: false,
        ..Default::default()
    };

    let compiler = HamiltonianCompiler::new(config);
    let circuit = compiler.compile(&hamiltonian)?;

    // Export to QASM
    use myquat::QasmExporter;
    let qasm_str = QasmExporter::new().export(&circuit)?;

    println!("{}", qasm_str);

    Ok(())
}
