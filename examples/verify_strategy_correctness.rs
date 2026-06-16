//! Verify strategy correctness: raw vs optimized, PauliLevel vs GateLevel
//! Author: gA4ss

use myquat::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
use myquat::hamiltonian::{
    CompilationStrategy, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString,
    TrotterOrder,
};
use myquat::Result;
use ndarray::Array2;
use num_complex::Complex64;
use std::collections::HashMap;

fn phase_insensitive_dist(a: &Array2<Complex64>, b: &Array2<Complex64>) -> f64 {
    let n = a.nrows();
    let mut trace = Complex64::new(0.0, 0.0);
    for i in 0..n {
        for j in 0..n {
            trace += a[(i, j)].conj() * b[(i, j)];
        }
    }
    let angle = trace.im.atan2(trace.re);
    let phase = Complex64::new(angle.cos(), angle.sin()); // e^{i*angle}
    let scale = 1.0 / (n as f64).sqrt();
    let mut diff_norm_sq = 0.0f64;
    for i in 0..n {
        for j in 0..n {
            let d = a[(i, j)] - phase * b[(i, j)];
            diff_norm_sq += d.norm_sqr();
        }
    }
    diff_norm_sq.sqrt() * scale
}

fn compute_unitary(circuit: &myquat::QuantumCircuit) -> Option<Array2<Complex64>> {
    circuit.unitary(&HashMap::new()).ok()
}

fn compile(
    hamiltonian: &Hamiltonian,
    strategy: CompilationStrategy,
    optimize: bool,
) -> myquat::QuantumCircuit {
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
        apply_circuit_optimization: optimize,
        auto_optimize_grouping: true,
        layout_aware_grouping: false,
        optimization_strategy: strategy,
        cross_step_synthesis: false,
        block_grouping_strategy: BlockGroupingStrategy::QWC,
        pauli_gadget_optimization: myquat::hamiltonian::GadgetOptimizationStrategy::IdenticalOnly,
        clifford_enhanced_blocks: false,
        ..Default::default()
    };
    let compiler = HamiltonianCompiler::new(config);
    compiler.compile(hamiltonian).unwrap()
}

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

    // Compile all 4 variants
    let c_pr = compile(&hamiltonian, CompilationStrategy::PauliLevel, false);
    let c_po = compile(&hamiltonian, CompilationStrategy::PauliLevel, true);
    let c_gr = compile(&hamiltonian, CompilationStrategy::GateLevel, false);
    let c_go = compile(&hamiltonian, CompilationStrategy::GateLevel, true);

    let u_pr = compute_unitary(&c_pr);
    let u_po = compute_unitary(&c_po);
    let u_gr = compute_unitary(&c_gr);
    let u_go = compute_unitary(&c_go);

    println!("Gate counts:");
    println!(
        "  PauliLevel RAW: {} gates, {} CX",
        c_pr.size(),
        count_cx(&c_pr)
    );
    println!(
        "  PauliLevel OPT: {} gates, {} CX",
        c_po.size(),
        count_cx(&c_po)
    );
    println!(
        "  GateLevel  RAW: {} gates, {} CX",
        c_gr.size(),
        count_cx(&c_gr)
    );
    println!(
        "  GateLevel  OPT: {} gates, {} CX",
        c_go.size(),
        count_cx(&c_go)
    );

    println!("\nPhase-insensitive distances:");
    if let (Some(a), Some(b)) = (&u_pr, &u_gr) {
        let d = phase_insensitive_dist(a, b);
        println!(
            "  PauliLevel_RAW vs GateLevel_RAW:  {:.8} {}",
            d,
            if d > 1e-6 { "<- DIFFER!" } else { "OK" }
        );
    }
    if let (Some(a), Some(b)) = (&u_pr, &u_po) {
        let d = phase_insensitive_dist(a, b);
        println!(
            "  PauliLevel_RAW vs PauliLevel_OPT: {:.8} {}",
            d,
            if d > 1e-6 { "<- OPT CHANGED!" } else { "OK" }
        );
    }
    if let (Some(a), Some(b)) = (&u_gr, &u_go) {
        let d = phase_insensitive_dist(a, b);
        println!(
            "  GateLevel_RAW vs GateLevel_OPT:   {:.8} {}",
            d,
            if d > 1e-6 { "<- OPT CHANGED!" } else { "OK" }
        );
    }
    if let (Some(a), Some(b)) = (&u_po, &u_go) {
        let d = phase_insensitive_dist(a, b);
        println!(
            "  PauliLevel_OPT vs GateLevel_OPT:   {:.8} {}",
            d,
            if d > 1e-6 { "<- DIFFER!" } else { "OK" }
        );
    }

    Ok(())
}

fn count_cx(c: &myquat::QuantumCircuit) -> usize {
    c.data()
        .instructions()
        .iter()
        .filter(|i| matches!(i.gate.gate_type, myquat::gates::StandardGate::CX))
        .count()
}
