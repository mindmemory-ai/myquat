//! Investigate why PauliLevel per-step is better but 10-step is worse.
//! Hypothesis: Post-synthesis optimization works differently on PauliLevel output.
use myquat::circuit_optimization::PassManager;
use myquat::hamiltonian::{self, PauliOperator, PauliString, *};
use myquat::*;
use num_complex::Complex64;
use std::f64::consts::PI;

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

    let strategies = [
        CompilationStrategy::PauliLevel,
        CompilationStrategy::GateLevel,
    ];
    let step_counts = [1, 2, 4, 10];

    for &steps in &step_counts {
        println!("=== {} Trotter steps ===", steps);
        println!(
            "{:>12} | {:>10} | {:>10} | {:>12} | {:>12} | {:>10}",
            "Strategy", "Raw gates", "Raw CX", "Opt(lv2)", "Opt(lv2+4)", "Depth"
        );
        println!("{:-<80}", "");

        for &strat in &strategies {
            let mut config = CompilerConfig {
                trotter_order: TrotterOrder::First,
                trotter_steps: steps,
                evolution_time: 1.0,
                optimization_strategy: strat,
                apply_circuit_optimization: false, // Get raw first
                ..CompilerConfig::default()
            };

            // Raw (no optimization)
            let circuit = HamiltonianCompiler::new(config).compile(&h).unwrap();
            let raw_gates = circuit.size();
            let raw_cx = circuit
                .data()
                .instructions()
                .iter()
                .filter(|i| i.gate.gate_type == StandardGate::CX)
                .count();

            // Level 2 only
            let mut c2 = circuit.clone();
            PassManager::level_2().run(&mut c2).unwrap();

            // Level 2 + 4
            let mut c4 = circuit.clone();
            PassManager::level_2().run(&mut c4).unwrap();
            PassManager::level_4().run(&mut c4).unwrap();
            let depth4 = c4.depth();

            let label = match strat {
                CompilationStrategy::PauliLevel => "PauliLevel",
                CompilationStrategy::PauliGadget => "PauliGadget",
                CompilationStrategy::GateLevel => "GateLevel",
            };

            println!(
                "{:>12} | {:>10} | {:>10} | {:>12} | {:>12} | {:>10}",
                label,
                raw_gates,
                raw_cx,
                c2.size(),
                c4.size(),
                depth4
            );
        }

        // Show per-step averages for 10-step
        if steps == 10 {
            println!("\n--- Per-step averages ---");
            for &strat in &strategies {
                let mut config = CompilerConfig {
                    trotter_order: TrotterOrder::First,
                    trotter_steps: steps,
                    evolution_time: 1.0,
                    optimization_strategy: strat,
                    apply_circuit_optimization: false,
                    ..CompilerConfig::default()
                };
                let circuit = HamiltonianCompiler::new(config).compile(&h).unwrap();
                let raw = circuit.size();
                let mut c4 = circuit.clone();
                PassManager::level_2().run(&mut c4).unwrap();
                PassManager::level_4().run(&mut c4).unwrap();
                let opt = c4.size();
                let label = match strat {
                    CompilationStrategy::PauliLevel => "PauliLevel",
                    CompilationStrategy::PauliGadget => "PauliGadget",
                    CompilationStrategy::GateLevel => "GateLevel",
                };
                println!(
                    "{}: {} raw/step, {} opt/step",
                    label,
                    raw as f64 / steps as f64,
                    opt as f64 / steps as f64
                );
            }
        }
        println!();
    }

    // Check: how many CX are between Trotter steps?
    println!("=== Step-boundary CNOT analysis (PauliLevel, 10 steps) ===");
    let config = CompilerConfig {
        trotter_order: TrotterOrder::First,
        trotter_steps: 10,
        evolution_time: 1.0,
        optimization_strategy: CompilationStrategy::PauliLevel,
        apply_circuit_optimization: false,
        ..CompilerConfig::default()
    };
    let circuit = HamiltonianCompiler::new(config).compile(&h).unwrap();
    let insts = circuit.data().instructions();

    // Show first 100 instructions to understand structure
    println!("First 50 instructions (raw PauliLevel, 10 steps):");
    for (j, inst) in insts.iter().take(50).enumerate() {
        let qubits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();
        let params: Vec<String> = inst
            .gate
            .parameters
            .iter()
            .map(|p| format!("{:?}", p))
            .collect();
        let param_str = if params.is_empty() {
            String::new()
        } else {
            format!("({})", params.join(", "))
        };
        println!(
            "  {:3}: {:?}{} on {:?}",
            j, inst.gate.gate_type, param_str, qubits
        );
    }
    println!("  ... ({} total instructions)", insts.len());

    // Count patterns: CX(a,b) followed by CX(b,a) or CX(a,b) (inverse pair or duplicate)
    let mut cx_pairs = 0;
    let mut inverse_pairs = 0;
    let windows: Vec<_> = insts.windows(2).enumerate().collect();
    for (_j, w) in windows {
        if w[0].gate.gate_type == StandardGate::CX && w[1].gate.gate_type == StandardGate::CX {
            let q0: Vec<_> = w[0].qubits.iter().map(|q| q.index()).collect();
            let q1: Vec<_> = w[1].qubits.iter().map(|q| q.index()).collect();
            if q0 == q1 {
                cx_pairs += 1; // Adjacent identical CX
            }
            if q0[0] == q1[1] && q0[1] == q1[0] {
                inverse_pairs += 1; // Adjacent inverse CX
            }
        }
    }
    println!("Adjacent identical CX pairs: {} (cancelable)", cx_pairs);
    println!("Adjacent inverse CX pairs: {} (cancelable)", inverse_pairs);
}
