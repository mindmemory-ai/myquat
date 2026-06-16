//! Hamiltonian Benchmark Tool
//! Author: gA4ss
//!
//! Command-line tool for benchmarking Hamiltonian compilation.
//! Supports configurable Trotter order, steps, and evolution time.

use myquat::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
use myquat::hamiltonian::{
    CompilationStrategy, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString,
    TrotterOrder,
};
use myquat::Result;
use num_complex::Complex64;
use std::collections::HashMap;
use std::env;
use std::time::Instant;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 6 {
        eprintln!(
            "Usage: {} <num_qubits> <trotter_steps> <evolution_time> <order> <pauli:coeff> ...",
            args[0]
        );
        eprintln!(
            "Example: {} 4 10 1.0 2 IIII:-0.81 IIIZ:0.17 IIZI:-0.22",
            args[0]
        );
        std::process::exit(1);
    }

    let num_qubits: usize = args[1].parse().expect("Invalid number of qubits");
    let trotter_steps: usize = args[2].parse().expect("Invalid trotter steps");
    let evolution_time: f64 = args[3].parse().expect("Invalid evolution time");
    let order: usize = args[4].parse().expect("Invalid Trotter order");

    // Create Hamiltonian
    let mut hamiltonian = Hamiltonian::new(num_qubits);

    // Parse and add terms
    for term_str in &args[5..] {
        let parts: Vec<&str> = term_str.split(':').collect();
        if parts.len() != 2 {
            eprintln!("Invalid term format: {}", term_str);
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

    // Map order to TrotterOrder
    let trotter_order = match order {
        1 => TrotterOrder::First,
        2 => TrotterOrder::Second,
        4 => TrotterOrder::Fourth,
        6 => TrotterOrder::Sixth,
        n => TrotterOrder::Nth(n),
    };

    // Compile with optimization
    let config = CompilerConfig {
        trotter_order,
        trotter_steps,
        evolution_time,
        adaptive: false,
        adaptive_tolerance: 1e-3,
        min_step_size: 1e-6,
        max_step_size: 1.0,
        hbar: 1.0,
        skip_identities: true,
        group_commuting_terms: true,
        apply_circuit_optimization: true,
        auto_optimize_grouping: true,
        layout_aware_grouping: false,
        optimization_strategy: CompilationStrategy::GateLevel, // Phase 9l: GateLevel now outperforms PauliLevel by 11%
        cross_step_synthesis: false,
        block_grouping_strategy: BlockGroupingStrategy::QWC,
        alternate_reverse_steps: true,
        clifford_enhanced_blocks: false,
        pauli_gadget_optimization: myquat::hamiltonian::GadgetOptimizationStrategy::IdenticalOnly,
        ..Default::default()
    };

    let compiler = HamiltonianCompiler::new(config);

    let start = Instant::now();
    let circuit = compiler.compile(&hamiltonian)?;
    let compile_time = start.elapsed();
    let compilation_time_ms = compile_time.as_micros() as f64 / 1000.0;

    // Compute unitary matrix for accuracy verification (only for ≤6 qubits)
    // Above 6 qubits, the 2^n x 2^n matrix becomes too large for JSON
    let unitary_json = if num_qubits <= 6 {
        match circuit.unitary(&HashMap::new()) {
            Ok(mat) => {
                let flat: Vec<String> = mat
                    .iter()
                    .map(|z| format!("[{:.12e},{:.12e}]", z.re, z.im))
                    .collect();
                format!("\"unitary\": [{}]", flat.join(","))
            }
            Err(_) => "\"unitary\": null".to_string(),
        }
    } else {
        "\"unitary\": null".to_string()
    };

    // Output Hamiltonian terms for independent exact computation
    let terms_json: Vec<String> = hamiltonian
        .terms
        .iter()
        .map(|t| {
            format!(
                "[\"{}\",{:.12e}]",
                t.pauli_string.to_string_repr(),
                t.coefficient.re
            )
        })
        .collect();

    // Output results as JSON
    println!("{{\"compilation_time_ms\": {:.4}, \"gate_count\": {}, \"circuit_depth\": {}, \"num_qubits\": {}, \"trotter_steps\": {}, \"trotter_order\": {}, {}, \"terms\": [{}]}}",
        compilation_time_ms,
        circuit.size(),
        circuit.depth(),
        circuit.num_qubits(),
        trotter_steps,
        order,
        unitary_json,
        terms_json.join(","));

    Ok(())
}
