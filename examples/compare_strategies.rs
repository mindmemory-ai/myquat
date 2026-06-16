//! Comprehensive comparison: all synthesis strategies for H2_4q
use myquat::error::Result;
use myquat::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
use myquat::hamiltonian::{
    CompilationStrategy, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString,
    TrotterOrder,
};
use ndarray::Array2;
use num_complex::Complex64;
use std::collections::HashMap;

fn compute_unitary(c: &myquat::QuantumCircuit) -> Option<Array2<Complex64>> {
    c.unitary(&HashMap::new()).ok()
}

fn phase_insensitive_dist(a: &Array2<Complex64>, b: &Array2<Complex64>) -> f64 {
    let n = a.nrows();
    let mut trace = Complex64::new(0.0, 0.0);
    for i in 0..n {
        for j in 0..n {
            trace += a[(i, j)].conj() * b[(i, j)];
        }
    }
    let angle = trace.im.atan2(trace.re);
    let phase = Complex64::new(angle.cos(), angle.sin());
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

fn count_cx(c: &myquat::QuantumCircuit) -> usize {
    c.data()
        .instructions()
        .iter()
        .filter(|i| matches!(i.gate.gate_type, myquat::gates::StandardGate::CX))
        .count()
}

fn main() -> Result<()> {
    let mut h = Hamiltonian::new(4);
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
        h.add_term(PauliString::from_str(ps)?, Complex64::new(*coeff, 0.0))?;
    }

    let base_config = CompilerConfig {
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
        apply_circuit_optimization: true,
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

    println!("=== H2_4q Synthesis Strategy Comparison (10 steps, 1st order) ===\n");
    println!(
        "{:40} {:>8} {:>8} {:>10} {:>10}",
        "Strategy", "Gates", "CX", "Depth", "Fidelity"
    );
    println!("{}", "-".repeat(80));

    // Reference: 100-step Trotter as proxy for exact
    let mut ref_config = base_config.clone();
    ref_config.trotter_steps = 100;
    ref_config.apply_circuit_optimization = false;
    let ref_circuit = HamiltonianCompiler::new(ref_config).compile(&h)?;
    let u_ref = compute_unitary(&ref_circuit);

    let strategies: Vec<(&str, CompilationStrategy, bool, bool)> = vec![
        (
            "Per-step PauliLevel + opt",
            CompilationStrategy::PauliLevel,
            false,
            true,
        ),
        (
            "Per-step PauliLevel RAW",
            CompilationStrategy::PauliLevel,
            false,
            false,
        ),
        (
            "Per-step GateLevel + opt",
            CompilationStrategy::GateLevel,
            false,
            true,
        ),
        (
            "Per-step GateLevel RAW",
            CompilationStrategy::GateLevel,
            false,
            false,
        ),
        (
            "Per-step PauliGadget + opt",
            CompilationStrategy::PauliGadget,
            false,
            true,
        ),
        (
            "Per-step PauliGadget RAW",
            CompilationStrategy::PauliGadget,
            false,
            false,
        ),
        (
            "Cross-step 1st + opt",
            CompilationStrategy::PauliLevel,
            true,
            true,
        ),
    ];

    for (name, strategy, cross_step, opt) in &strategies {
        let mut config = base_config.clone();
        config.optimization_strategy = *strategy;
        config.cross_step_synthesis = *cross_step;
        config.apply_circuit_optimization = *opt;
        let c = HamiltonianCompiler::new(config).compile(&h)?;
        let cx = count_cx(&c);
        let depth = c.depth();
        let u = compute_unitary(&c);
        let fid = u
            .as_ref()
            .and_then(|u| u_ref.as_ref().map(|r| 1.0 - phase_insensitive_dist(u, r)))
            .unwrap_or(0.0);
        println!(
            "{:40} {:>8} {:>8} {:>10} {:>10.6}",
            name,
            c.size(),
            cx,
            depth,
            fid
        );
    }

    // Cross-step with higher Trotter orders
    for order in &[TrotterOrder::Second, TrotterOrder::Fourth] {
        let mut config = base_config.clone();
        config.trotter_order = order.clone();
        config.optimization_strategy = CompilationStrategy::PauliLevel;
        config.cross_step_synthesis = true;
        config.apply_circuit_optimization = true;
        let c = HamiltonianCompiler::new(config).compile(&h)?;
        let cx = count_cx(&c);
        let depth = c.depth();
        let u = compute_unitary(&c);
        let fid = u
            .as_ref()
            .and_then(|u| u_ref.as_ref().map(|r| 1.0 - phase_insensitive_dist(u, r)))
            .unwrap_or(0.0);
        println!(
            "{:40} {:>8} {:>8} {:>10} {:>10.6}",
            format!("Cross-step {:?} + opt", order),
            c.size(),
            cx,
            depth,
            fid
        );
    }

    // 1-step reference
    let mut config_1step = base_config.clone();
    config_1step.trotter_steps = 1;
    config_1step.apply_circuit_optimization = false;
    let c1p = HamiltonianCompiler::new(config_1step.clone()).compile(&h)?;
    println!(
        "\n{:40} {:>8} {:>8}",
        "1-step PauliLevel RAW",
        c1p.size(),
        count_cx(&c1p)
    );
    config_1step.optimization_strategy = CompilationStrategy::GateLevel;
    let c1g = HamiltonianCompiler::new(config_1step).compile(&h)?;
    println!(
        "{:40} {:>8} {:>8}",
        "1-step GateLevel RAW",
        c1g.size(),
        count_cx(&c1g)
    );

    Ok(())
}
