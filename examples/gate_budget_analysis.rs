//! Gate Budget Analysis for H₂ (4-qubit) Hamiltonian
//!
//! Compares PauliLevel vs GateLevel per-category gate counts to
//! identify where PauliLevel adds overhead vs GateLevel.
//!
//! Usage:
//!   cargo run --example gate_budget_analysis

use myquat::hamiltonian::pauli_synthesis::{
    compile_step_pauli_synthesis, BlockGroupingStrategy, PauliGateBudget,
};
use myquat::hamiltonian::{Hamiltonian, PauliString};
use myquat::QuantumCircuit;
use num_complex::Complex64;

fn main() {
    let h = build_h2_4q();

    println!("═══════════════════════════════════════════");
    println!("  Phase 9a: Gate Budget Analysis — H₂ (4 qubits)");
    println!("═══════════════════════════════════════════");
    println!();

    // ── PauliLevel with budget tracking (1 step) ─────────────────
    let mut circuit = QuantumCircuit::new(4, 0);
    let mut budget = PauliGateBudget::default();
    let dt = 1.0;
    let hbar = 1.0;
    let terms: Vec<&myquat::hamiltonian::PauliTerm> = h.terms.iter().collect();

    compile_step_pauli_synthesis(
        &mut circuit,
        &terms,
        dt,
        hbar,
        false,
        BlockGroupingStrategy::QWC,
        Some(&mut budget),
        None,
        None,
        false,
    )
    .expect("PauliLevel compilation failed");

    budget.print("PauliLevel (1 step)");
    let pl_cx = count_cx(&circuit);
    let pl_size = circuit.size();

    // ── GateLevel comparison ─────────────────────────────────────
    let gl_compiler = build_gatelevel_compiler();
    let gl_circuit = gl_compiler
        .compile(&h)
        .expect("GateLevel compilation failed");
    let gl_cx = count_cx(&gl_circuit);
    let gl_size = gl_circuit.size();

    println!();
    println!("─── Comparison ───");
    println!("  PauliLevel (1 step): {} gates, {} CX", pl_size, pl_cx);
    println!("  GateLevel (1 step):   {} gates, {} CX", gl_size, gl_cx);
    println!(
        "  Overhead:             +{} gates, +{} CX",
        pl_size as isize - gl_size as isize,
        pl_cx as isize - gl_cx as isize
    );

    // ── Multi-step scaling ───────────────────────────────────────
    println!();
    println!("─── Multi-Step Scaling (order 1) ───");
    println!(
        "  {:<5} │ {:<8} │ {:<5} │ {:<8} │ {:<5} │ {:<8}",
        "Steps", "PL Gates", "PL CX", "GL Gates", "GL CX", "PL-GL Gap"
    );
    println!(
        "  {:-<5}─┼─{:-<8}─┼─{:-<5}─┼─{:-<8}─┼─{:-<5}─┼─{:-<8}",
        "", "", "", "", "", ""
    );

    for steps in &[1usize, 2, 5, 10] {
        let (pl_g, pl_c) = compile_n_steps_pauli(&h, *steps);
        let (gl_g, gl_c) = compile_n_steps_gatelevel(&h, *steps);
        let gap = pl_g as isize - gl_g as isize;
        println!(
            "  {:<5} │ {:<8} │ {:<5} │ {:<8} │ {:<5} │ {:+}",
            steps, pl_g, pl_c, gl_g, gl_c, gap
        );
    }

    // ── Root cause summary ───────────────────────────────────────
    let pos0_extra = budget.rz_position_0_reverse;
    println!();
    println!("═══════════════════════════════════════════");
    println!("  Root Cause Confirmation");
    println!("═══════════════════════════════════════════");
    println!(
        "  Cause 1 — Position-0 Rz splitting: {} waste Rz per step",
        pos0_extra
    );
    println!("    → {} Rz over 10 steps", pos0_extra * 10);
    println!("  Cause 4 — Per-step synthesis scope: CX scales linearly with steps");
    println!("    → 10× steps ≈ 10× CX (no cross-step sharing)");
    println!("  Cause 2 — Non-deterministic blocks: see diagnose_block_divergence");
    println!("  Cause 3 — _is_reverse ignored: confirmed in code");
    println!(
        "  Cause 5 — Secondary chain: {} CX tracked separately",
        budget.cx_secondary_chain
    );
    println!("═══════════════════════════════════════════");
}

fn build_h2_4q() -> Hamiltonian {
    let mut h = Hamiltonian::new(4);
    for (ps, coeff) in [
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
    ] {
        h.add_term(
            PauliString::from_str(ps).unwrap(),
            Complex64::new(coeff, 0.0),
        )
        .unwrap();
    }
    h
}

fn count_cx(circuit: &QuantumCircuit) -> usize {
    use myquat::StandardGate;
    circuit
        .data()
        .instructions()
        .iter()
        .filter(|inst| inst.gate.gate_type == StandardGate::CX)
        .count()
}

fn count_cx_in_range(circuit: &QuantumCircuit, skip: usize, take: usize) -> usize {
    use myquat::StandardGate;
    circuit
        .data()
        .instructions()
        .iter()
        .skip(skip)
        .take(take)
        .filter(|inst| inst.gate.gate_type == StandardGate::CX)
        .count()
}

fn compile_n_steps_pauli(h: &Hamiltonian, steps: usize) -> (usize, usize) {
    let mut circuit = QuantumCircuit::new(h.num_qubits, 0);
    let terms: Vec<&myquat::hamiltonian::PauliTerm> = h.terms.iter().collect();
    for _ in 0..steps {
        compile_step_pauli_synthesis(
            &mut circuit,
            &terms,
            1.0,
            1.0,
            false,
            BlockGroupingStrategy::QWC,
            None,
            None,
            None,
            false,
        )
        .expect("PauliLevel step failed");
    }
    (circuit.size(), count_cx(&circuit))
}

fn compile_n_steps_gatelevel(h: &Hamiltonian, steps: usize) -> (usize, usize) {
    use myquat::hamiltonian::hamiltonian_compiler::{
        CompilationStrategy, CompilerConfig, HamiltonianCompiler,
    };
    use myquat::TrotterOrder;

    let mut config = CompilerConfig::default();
    config.optimization_strategy = CompilationStrategy::GateLevel;
    config.trotter_steps = steps;
    config.trotter_order = TrotterOrder::Nth(1);
    config.group_commuting_terms = true;
    config.hbar = 1.0;
    config.evolution_time = 1.0;

    let compiler = HamiltonianCompiler::new(config);
    let circuit = compiler.compile(h).expect("GateLevel compilation failed");

    (circuit.size(), count_cx(&circuit))
}

fn build_gatelevel_compiler() -> myquat::hamiltonian::hamiltonian_compiler::HamiltonianCompiler {
    use myquat::hamiltonian::hamiltonian_compiler::{
        CompilationStrategy, CompilerConfig, HamiltonianCompiler,
    };
    use myquat::TrotterOrder;

    let mut config = CompilerConfig::default();
    config.optimization_strategy = CompilationStrategy::GateLevel;
    config.trotter_steps = 1;
    config.trotter_order = TrotterOrder::Nth(1);
    config.group_commuting_terms = true;
    config.hbar = 1.0;
    config.evolution_time = 1.0;

    HamiltonianCompiler::new(config)
}
