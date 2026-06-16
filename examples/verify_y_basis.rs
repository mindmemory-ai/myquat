//! Verify correctness of Y basis change: S·H vs Rx(π/2)
//! Tests that PauliLevel with corrected Y basis produces the right unitary.
use myquat::hamiltonian::*;
use myquat::*;
use num_complex::Complex64;

fn main() {
    // Test: single Y term e^{-iθ Y} on 1 qubit
    for (label, coeff) in [("Y_1q", 1.0)] {
        let mut h = Hamiltonian::new(1);
        h.add_term(
            PauliString::from_str("Y").unwrap(),
            Complex64::new(coeff, 0.0),
        )
        .unwrap();

        for (strat, name) in &[
            (CompilationStrategy::PauliLevel, "PauliLevel"),
            (CompilationStrategy::GateLevel, "GateLevel"),
        ] {
            let config = CompilerConfig {
                trotter_order: TrotterOrder::First,
                trotter_steps: 1,
                evolution_time: 1.0,
                optimization_strategy: *strat,
                apply_circuit_optimization: false,
                ..CompilerConfig::default()
            };
            let circuit = HamiltonianCompiler::new(config).compile(&h).unwrap();
            let unitary = circuit.unitary(&std::collections::HashMap::new()).unwrap();

            // Expected: exp(-i * 1.0 * Y) = cos(1)·I - i·sin(1)·Y
            // Y = [[0,-i],[i,0]]
            let c = 1.0_f64.cos();
            let s = 1.0_f64.sin();
            let expected = ndarray::arr2(&[
                [Complex64::new(c, 0.0), Complex64::new(-s, 0.0)],
                [Complex64::new(s, 0.0), Complex64::new(c, 0.0)],
            ]);

            let err = frobenius_diff(&unitary, &expected);
            println!(
                "{} Y_1q: frob_err={:.6} ({} instructions)",
                name,
                err,
                circuit.size()
            );
            if err > 0.01 {
                println!("  WARNING: large error! Circuit:");
                for inst in circuit.data().instructions() {
                    let q: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();
                    let p: Vec<String> = inst
                        .gate
                        .parameters
                        .iter()
                        .map(|p| format!("{:?}", p))
                        .collect();
                    println!(
                        "    {:?}{} on {:?}",
                        inst.gate.gate_type,
                        if p.is_empty() {
                            String::new()
                        } else {
                            format!("({})", p.join(","))
                        },
                        q
                    );
                }
            }
        }
    }

    // Test: YY term e^{-iθ Y₀Y₁} on 2 qubits
    println!();
    for (label, coeff) in [("YY_2q", 0.5)] {
        let mut h = Hamiltonian::new(2);
        h.add_term(
            PauliString::from_str("YY").unwrap(),
            Complex64::new(coeff, 0.0),
        )
        .unwrap();

        for (strat, name) in &[
            (CompilationStrategy::PauliLevel, "PauliLevel"),
            (CompilationStrategy::GateLevel, "GateLevel"),
        ] {
            let config = CompilerConfig {
                trotter_order: TrotterOrder::First,
                trotter_steps: 1,
                evolution_time: 1.0,
                optimization_strategy: *strat,
                apply_circuit_optimization: false,
                ..CompilerConfig::default()
            };
            let circuit = HamiltonianCompiler::new(config).compile(&h).unwrap();
            let unitary = circuit.unitary(&std::collections::HashMap::new()).unwrap();

            // Expected: exp(-i * 0.5 * YY) with YY = Y⊗Y
            // Using nalgebra-like computation or direct matrix exponential
            // For simplicity, check against GateLevel which we trust
            let symbols: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
            let unitary_vec: Vec<Complex64> = unitary.iter().cloned().collect();

            println!("{} YY_2q: {} instructions", name, circuit.size());
            for inst in circuit.data().instructions() {
                let q: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();
                let p: Vec<String> = inst
                    .gate
                    .parameters
                    .iter()
                    .map(|p| format!("{:?}", p))
                    .collect();
                println!(
                    "  {:?}{} on {:?}",
                    inst.gate.gate_type,
                    if p.is_empty() {
                        String::new()
                    } else {
                        format!("({})", p.join(","))
                    },
                    q
                );
            }
        }
    }
}

fn frobenius_diff(a: &ndarray::Array2<Complex64>, b: &ndarray::Array2<Complex64>) -> f64 {
    let diff = a - b;
    let n = diff.nrows() as f64;
    (diff.iter().map(|c| c.norm_sqr()).sum::<f64>() / n).sqrt()
}
