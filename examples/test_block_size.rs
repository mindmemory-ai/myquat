//! Test if increasing max_block_size fixes PauliLevel BC degradation at scale.
//! Hypothesis: max_block_size=10 truncates blocks at step boundaries.
use myquat::circuit_optimization::{BlockConsolidationPass, CircuitPass};
use myquat::hamiltonian::{self, PauliOperator, PauliString, *};
use myquat::*;
use num_complex::Complex64;

fn main() {
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

    println!("Testing max_block_size impact on PauliLevel optimization");
    println!(
        "{:<12} | {:<10} | {:<10} | {:<10} | {:<10} | {:<10}",
        "Steps", "Raw", "BC(size=10)", "BC(size=20)", "BC(size=50)", "BC(size=∞)"
    );
    println!("{:-<75}", "");

    for &steps in &[1, 2, 4, 10] {
        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: steps,
            evolution_time: 1.0,
            optimization_strategy: CompilationStrategy::PauliLevel,
            apply_circuit_optimization: false,
            ..CompilerConfig::default()
        };
        let raw = HamiltonianCompiler::new(config).compile(&h).unwrap();

        // BC with various max_block_size
        let sizes = [10, 20, 50, usize::MAX];
        let mut results = Vec::new();
        for &mbs in &sizes {
            let mut c = raw.clone();
            let bc = if mbs == usize::MAX {
                BlockConsolidationPass::with_max_block_size(1000)
            } else {
                BlockConsolidationPass::with_max_block_size(mbs)
            };
            bc.run(&mut c).unwrap();
            results.push(c.size());
        }

        print!("{:>5} steps ", steps);
        print!("| {:>10} ", raw.size());
        for r in &results {
            let diff = raw.size() as i32 - *r as i32;
            print!("| {:>3} ({:+}) ", r, diff);
        }
        println!();

        // Also show CX counts
        let raw_cx = raw
            .data()
            .instructions()
            .iter()
            .filter(|i| i.gate.gate_type == StandardGate::CX)
            .count();
        print!("{:>5} CX   ", "");
        print!("| {:>10} ", raw_cx);
        for (i, &mbs) in sizes.iter().enumerate() {
            let mut c = raw.clone();
            let bc = if mbs == usize::MAX {
                BlockConsolidationPass::with_max_block_size(1000)
            } else {
                BlockConsolidationPass::with_max_block_size(mbs)
            };
            bc.run(&mut c).unwrap();
            let cx = c
                .data()
                .instructions()
                .iter()
                .filter(|i| i.gate.gate_type == StandardGate::CX)
                .count();
            let diff = raw_cx as i32 - cx as i32;
            print!("| {:>3} ({:+}) ", cx, diff);
        }
        println!();
    }

    // Compare unitaries at 1 step to verify correctness
    println!("\n=== Unitary accuracy check (1 step) ===");
    let config = CompilerConfig {
        trotter_order: TrotterOrder::First,
        trotter_steps: 1,
        evolution_time: 1.0,
        optimization_strategy: CompilationStrategy::PauliLevel,
        apply_circuit_optimization: false,
        ..CompilerConfig::default()
    };
    let raw = HamiltonianCompiler::new(config).compile(&h).unwrap();
    let target = raw.unitary(&std::collections::HashMap::new()).unwrap();

    for &mbs in &[10, 20, 50, 1000] {
        let mut c = raw.clone();
        let bc = BlockConsolidationPass::with_max_block_size(mbs);
        bc.run(&mut c).unwrap();
        let unitary = c.unitary(&std::collections::HashMap::new()).unwrap();
        let err = frobenius_diff(&unitary, &target);
        println!("BC(size={}): {} gates, frob_err={:.6}", mbs, c.size(), err);
    }
}

fn frobenius_diff(a: &ndarray::Array2<Complex64>, b: &ndarray::Array2<Complex64>) -> f64 {
    let diff = a - b;
    let n = diff.nrows() as f64;
    (diff.iter().map(|c| c.norm_sqr()).sum::<f64>() / n).sqrt()
}
