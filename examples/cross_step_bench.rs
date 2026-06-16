//! Cross-step Pauli synthesis benchmark — H₂ (4-qubit)
//!
//! Compares per-step vs cross-step gate counts and accuracy.
//!
//! Usage:
//!   cargo run --example cross_step_bench

use myquat::hamiltonian::hamiltonian_compiler::{
    CompilationStrategy, CompilerConfig, HamiltonianCompiler, TrotterOrder,
};
use myquat::hamiltonian::pauli_synthesis::{
    compile_cross_step_pauli_synthesis, compile_step_pauli_synthesis, BlockGroupingStrategy,
};
use myquat::hamiltonian::{Hamiltonian, PauliString};
use myquat::{QuantumCircuit, StandardGate};
use num_complex::Complex64;

fn main() {
    let h = build_h2_4q();

    println!("═══════════════════════════════════════════════════════");
    println!("  Phase 9a: Cross-Step Synthesis Benchmark — H₂ (4 qubits)");
    println!("═══════════════════════════════════════════════════════");
    println!();

    // ── Per-step PauliLevel ───────────────────────────────────────
    for steps in &[1usize, 5, 10] {
        let (per_g, per_cx, per_unitary) = compile_per_step(&h, *steps);
        let (cross_g, cross_cx, cross_unitary) = compile_cross_step(&h, *steps);

        let gate_save = per_g as isize - cross_g as isize;
        let cx_save = per_cx as isize - cross_cx as isize;
        let fidelity = unitary_fidelity(&per_unitary, &cross_unitary);

        println!("─── trotter_steps = {} ───", steps);
        println!("  Per-step:   {:>5} gates, {:>4} CX", per_g, per_cx);
        println!(
            "  Cross-step: {:>5} gates, {:>4} CX  (save {} gates, {} CX)",
            cross_g, cross_cx, gate_save, cx_save
        );
        println!("  Cross-step vs per-step fidelity: {:.6}", fidelity);
        if cx_save > 0 {
            let reduction = cx_save as f64 / per_cx as f64 * 100.0;
            println!("  CX reduction: {:.1}%", reduction);
        }
        println!();
    }

    // ── Compare cross-step with GateLevel ─────────────────────────
    println!("─── Comparison with GateLevel ───");
    for steps in &[1usize, 5, 10] {
        let (gl_g, gl_cx, _) = compile_gatelevel(&h, *steps);
        let (cross_g, cross_cx, _) = compile_cross_step(&h, *steps);
        println!(
            "  steps={}: GateLevel {:>5}g/{:>4}CX, Cross-step {:>5}g/{:>4}CX (Δ {}g/{}CX)",
            steps,
            gl_g,
            gl_cx,
            cross_g,
            cross_cx,
            cross_g as isize - gl_g as isize,
            cross_cx as isize - gl_cx as isize,
        );
    }

    // ── Cross-step with second-order Trotter ──────────────────────
    println!();
    println!("─── Second-order Cross-step ───");
    let h = build_h2_4q();
    let mut c2 = QuantumCircuit::new(4, 0);
    compile_cross_step_pauli_synthesis(
        &mut c2,
        &h,
        1.0,
        1.0,
        &TrotterOrder::Second,
        5,
        true,
        BlockGroupingStrategy::QWC,
        None,
        None,
        false,
    )
    .unwrap();
    let c2_g = c2.size();
    let c2_cx = count_cx(&c2);
    println!(
        "  Cross-step (order=2, steps=5): {} gates, {} CX",
        c2_g, c2_cx
    );

    let per_step_2 = compile_second_order_per_step(&h, 5);
    println!(
        "  Per-step   (order=2, steps=5): {} gates, {} CX",
        per_step_2.0, per_step_2.1
    );

    println!();
    println!("═══════════════════════════════════════════════════════");
    println!("  Done.");
    println!("═══════════════════════════════════════════════════════");
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

fn count_cx(c: &QuantumCircuit) -> usize {
    c.data()
        .instructions()
        .iter()
        .filter(|i| i.gate.gate_type == StandardGate::CX)
        .count()
}

fn count_gates(c: &QuantumCircuit) -> usize {
    c.size()
}

fn compile_per_step(h: &Hamiltonian, steps: usize) -> (usize, usize, Vec<Vec<Complex64>>) {
    let mut circuit = QuantumCircuit::new(4, 0);
    let dt = 1.0 / steps as f64;
    let terms: Vec<&myquat::hamiltonian::PauliTerm> = h.terms.iter().collect();
    for _ in 0..steps {
        compile_step_pauli_synthesis(
            &mut circuit,
            &terms,
            dt,
            1.0,
            false,
            BlockGroupingStrategy::QWC,
            None,
            None,
            None,
            false,
        )
        .unwrap();
    }
    let u = circuit.unitary(&std::collections::HashMap::new()).unwrap();
    let mat: Vec<Vec<Complex64>> = u
        .outer_iter()
        .map(|row| row.iter().map(|c| Complex64::new(c.re, c.im)).collect())
        .collect();
    (count_gates(&circuit), count_cx(&circuit), mat)
}

fn compile_cross_step(h: &Hamiltonian, steps: usize) -> (usize, usize, Vec<Vec<Complex64>>) {
    let mut circuit = QuantumCircuit::new(4, 0);
    compile_cross_step_pauli_synthesis(
        &mut circuit,
        h,
        1.0,
        1.0,
        &TrotterOrder::First,
        steps,
        true,
        BlockGroupingStrategy::QWC,
        None,
        None,
        false,
    )
    .unwrap();
    let u = circuit.unitary(&std::collections::HashMap::new()).unwrap();
    let mat: Vec<Vec<Complex64>> = u
        .outer_iter()
        .map(|row| row.iter().map(|c| Complex64::new(c.re, c.im)).collect())
        .collect();
    (count_gates(&circuit), count_cx(&circuit), mat)
}

fn compile_gatelevel(h: &Hamiltonian, steps: usize) -> (usize, usize, Vec<Vec<Complex64>>) {
    let mut config = CompilerConfig::default();
    config.optimization_strategy = CompilationStrategy::GateLevel;
    config.trotter_steps = steps;
    config.trotter_order = TrotterOrder::Nth(1);
    config.group_commuting_terms = true;
    config.hbar = 1.0;
    config.evolution_time = 1.0;
    config.apply_circuit_optimization = false; // raw, no post-processing

    let compiler = HamiltonianCompiler::new(config);
    let circuit = compiler.compile(h).unwrap();
    let u = circuit.unitary(&std::collections::HashMap::new()).unwrap();
    let mat: Vec<Vec<Complex64>> = u
        .outer_iter()
        .map(|row| row.iter().map(|c| Complex64::new(c.re, c.im)).collect())
        .collect();
    (count_gates(&circuit), count_cx(&circuit), mat)
}

fn compile_second_order_per_step(h: &Hamiltonian, steps: usize) -> (usize, usize) {
    let mut config = CompilerConfig::default();
    config.optimization_strategy = CompilationStrategy::PauliLevel;
    config.trotter_steps = steps;
    config.trotter_order = TrotterOrder::Second;
    config.group_commuting_terms = true;
    config.hbar = 1.0;
    config.evolution_time = 1.0;
    config.apply_circuit_optimization = false;
    config.cross_step_synthesis = false;

    let compiler = HamiltonianCompiler::new(config);
    let circuit = compiler.compile(h).unwrap();
    (count_gates(&circuit), count_cx(&circuit))
}

/// Phase-insensitive fidelity: |Tr(U_a^† · U_b)| / dim
fn unitary_fidelity(a: &[Vec<Complex64>], b: &[Vec<Complex64>]) -> f64 {
    let dim = a.len();
    let mut trace = Complex64::new(0.0, 0.0);
    for i in 0..dim {
        for j in 0..dim {
            trace += a[j][i].conj() * b[j][i];
        }
    }
    trace.norm() / dim as f64
}
