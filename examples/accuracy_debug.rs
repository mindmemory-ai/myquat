//! Accuracy debugging — verify H2_4q compilation fidelity
use myquat::circuit_optimization::PassManager;
use myquat::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
use myquat::hamiltonian::{
    CompilationStrategy, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString,
    TrotterOrder,
};
use std::collections::HashMap;

fn main() {
    let mut h = Hamiltonian::new(4);
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
        h.add_term(
            PauliString::from_str(ps).unwrap(),
            num_complex::Complex64::new(*coeff, 0.0),
        )
        .unwrap();
    }

    let config = CompilerConfig {
        trotter_order: TrotterOrder::First,
        trotter_steps: 10,
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
        pauli_gadget_optimization: myquat::hamiltonian::GadgetOptimizationStrategy::IdenticalOnly,
        alternate_reverse_steps: true,
        clifford_enhanced_blocks: false,
        adaptive: false,
        adaptive_tolerance: 1e-3,
        min_step_size: 1e-6,
        max_step_size: 1.0,
    };

    let mut c = HamiltonianCompiler::new(config).compile(&h).unwrap();
    let u_before = c.unitary(&HashMap::new()).unwrap();

    PassManager::level_2().run(&mut c).unwrap();
    let u_after = c.unitary(&HashMap::new()).unwrap();

    let mut max_diff = 0.0f64;
    let d = u_before.nrows();
    for i in 0..d {
        for j in 0..d {
            let diff = (u_before[(i, j)] - u_after[(i, j)]).norm();
            if diff > max_diff {
                max_diff = diff;
            }
        }
    }

    println!("H2_4q accuracy: max_diff={:.2e}", max_diff);
    assert!(max_diff < 1e-8, "Optimization corrupted fidelity!");
    println!("OK");
}
