//! GateFusion baseline comparison — standalone vs pipeline strategies
use myquat::circuit_optimization::PassManager;
use myquat::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
use myquat::hamiltonian::{
    CompilationStrategy, CompilerConfig, GadgetOptimizationStrategy, Hamiltonian,
    HamiltonianCompiler, PauliString, TrotterOrder,
};
use myquat::optimization_passes::GateFusion;
use ndarray::Array2;
use num_complex::Complex64;
use std::collections::HashMap;

fn count_cx(circuit: &myquat::QuantumCircuit) -> usize {
    circuit
        .data()
        .instructions()
        .iter()
        .filter(|i| i.gate.gate_type == myquat::StandardGate::CX)
        .count()
}

fn compute_fidelity(u_ref: &Array2<Complex64>, u_test: &Array2<Complex64>) -> f64 {
    let d = u_ref.nrows();
    let mut tr = Complex64::new(0.0, 0.0);
    for i in 0..d {
        for j in 0..d {
            tr += u_ref[(j, i)].conj() * u_test[(j, i)];
        }
    }
    tr.norm() / d as f64
}

fn benchmark_hamiltonian(name: &str, h: &Hamiltonian, order: TrotterOrder, steps: usize) {
    let order_label = format!("{:?}", order);
    let config = CompilerConfig {
        trotter_order: order,
        trotter_steps: steps,
        evolution_time: 1.0,
        hbar: 1.0,
        skip_identities: true,
        group_commuting_terms: true,
        apply_circuit_optimization: false,
        auto_optimize_grouping: true,
        layout_aware_grouping: false,
        optimization_strategy: CompilationStrategy::PauliLevel,
        cross_step_synthesis: false,
        block_grouping_strategy: BlockGroupingStrategy::QWC,
        pauli_gadget_optimization: GadgetOptimizationStrategy::IdenticalOnly,
        alternate_reverse_steps: true,
        clifford_enhanced_blocks: false,
        adaptive: false,
        adaptive_tolerance: 1e-3,
        min_step_size: 1e-6,
        max_step_size: 1.0,
        ..Default::default()
    };

    let raw = HamiltonianCompiler::new(config).compile(h).unwrap();
    let u_raw = raw.unitary(&HashMap::new()).unwrap();

    println!(
        "\n=== {} (order {}, {} steps) ===",
        name, order_label, steps
    );
    println!(
        "Raw:              {:>4} gates, {:>3} CX, depth {:>3}",
        raw.size(),
        count_cx(&raw),
        raw.depth()
    );

    // Strategy 1: level_2 only (current default)
    let mut s1 = raw.clone();
    PassManager::level_2().run(&mut s1).unwrap();
    let u1 = s1.unitary(&HashMap::new()).unwrap();
    let f1 = compute_fidelity(&u_raw, &u1);
    println!(
        "level_2:          {:>4} gates, {:>3} CX, depth {:>3}, fid {:.6}",
        s1.size(),
        count_cx(&s1),
        s1.depth(),
        f1
    );

    // Strategy 2: GateFusion → level_2
    let mut s2 = GateFusion::fuse_single_qubit_gates(&raw).unwrap();
    PassManager::level_2().run(&mut s2).unwrap();
    let u2 = s2.unitary(&HashMap::new()).unwrap();
    let f2 = compute_fidelity(&u_raw, &u2);
    println!(
        "GF → level_2:     {:>4} gates, {:>3} CX, depth {:>3}, fid {:.6}",
        s2.size(),
        count_cx(&s2),
        s2.depth(),
        f2
    );

    // Strategy 3: GateFusion → level_4
    let mut s3 = GateFusion::fuse_single_qubit_gates(&raw).unwrap();
    PassManager::level_4().run(&mut s3).unwrap();
    let u3 = s3.unitary(&HashMap::new()).unwrap();
    let f3 = compute_fidelity(&u_raw, &u3);
    println!(
        "GF → level_4:     {:>4} gates, {:>3} CX, depth {:>3}, fid {:.6}",
        s3.size(),
        count_cx(&s3),
        s3.depth(),
        f3
    );

    // Strategy 4: GateFusion → level_5 (full production)
    let mut s4 = GateFusion::fuse_single_qubit_gates(&raw).unwrap();
    PassManager::level_5().run(&mut s4).unwrap();
    let u4 = s4.unitary(&HashMap::new()).unwrap();
    let f4 = compute_fidelity(&u_raw, &u4);
    println!(
        "GF → level_5:     {:>4} gates, {:>3} CX, depth {:>3}, fid {:.6}",
        s4.size(),
        count_cx(&s4),
        s4.depth(),
        f4
    );
}

fn main() {
    // H2_4q
    let mut h2 = Hamiltonian::new(4);
    let terms: [(&str, f64); 15] = [
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
        h2.add_term(
            PauliString::from_str(ps).unwrap(),
            num_complex::Complex64::new(*coeff, 0.0),
        )
        .unwrap();
    }
    benchmark_hamiltonian("H2 (4q, 15 terms)", &h2, TrotterOrder::First, 10);

    // Heisenberg-4
    let h4 = myquat::hamiltonian::constructors::heisenberg_model(4, 1.0, 1.0, 1.0).unwrap();
    benchmark_hamiltonian("Heisenberg-4 (12 terms)", &h4, TrotterOrder::First, 10);

    // TFIM-4
    let tfim4 = myquat::hamiltonian::constructors::ising_model(4, 1.0, 1.0).unwrap();
    benchmark_hamiltonian("TFIM-4 (7 terms)", &tfim4, TrotterOrder::First, 10);

    // H2_4q with 2nd order
    benchmark_hamiltonian("H2_4q 2nd order", &h2, TrotterOrder::Second, 5);

    println!("\n=== Summary ===");
    println!("Baseline (from CLAUDE.md): 375 gates, 152 CX, depth 193, fid 0.999892");
    println!("TKET target: 329 gates, ~200 CX, depth 200");
}
