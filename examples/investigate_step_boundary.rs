//! Investigate step-boundary optimization: why PauliLevel degrades at scale.
use myquat::circuit_optimization::{CircuitPass, PassManager};
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

    for &strat in &[
        CompilationStrategy::PauliLevel,
        CompilationStrategy::GateLevel,
    ] {
        let label = match strat {
            CompilationStrategy::PauliLevel => "PauliLevel",
            CompilationStrategy::PauliGadget => "PauliGadget",
            CompilationStrategy::GateLevel => "GateLevel",
        };
        println!("\n========== {} ==========", label);

        // Synthesize 1 step and 2 steps
        let config_1 = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 1,
            evolution_time: 1.0,
            optimization_strategy: strat,
            apply_circuit_optimization: false,
            ..CompilerConfig::default()
        };
        let raw_1 = HamiltonianCompiler::new(config_1).compile(&h).unwrap();

        let config_2 = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 2,
            evolution_time: 1.0,
            optimization_strategy: strat,
            apply_circuit_optimization: false,
            ..CompilerConfig::default()
        };
        let raw_2 = HamiltonianCompiler::new(config_2).compile(&h).unwrap();

        // Optimize 2-step
        let mut opt_2 = raw_2.clone();
        PassManager::level_2().run(&mut opt_2).unwrap();
        PassManager::level_4().run(&mut opt_2).unwrap();

        // Show step boundary in raw 2-step circuit
        let half = raw_2.size() / 2;
        println!(
            "Raw 2-step: {} instructions (1-step: {})",
            raw_2.size(),
            raw_1.size()
        );
        println!(
            "Step boundary around instruction {}-{}:",
            half - 3,
            half + 2
        );
        for j in (half - 3).max(0)..(half + 3).min(raw_2.size()) {
            let inst = &raw_2.data().instructions()[j];
            let qubits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();
            let params: Vec<String> = inst
                .gate
                .parameters
                .iter()
                .map(|p| format!("{:.4}", p))
                .collect();
            let param_str = if params.is_empty() {
                String::new()
            } else {
                format!("({})", params.join(", "))
            };
            let marker = if j == half { " <-- STEP BOUNDARY" } else { "" };
            println!(
                "  {:3}: {:?}{} on {:?}{}",
                j, inst.gate.gate_type, param_str, qubits, marker
            );
        }

        // Now show optimized 2-step circuit
        println!("\nOptimized 2-step: {} instructions", opt_2.size());
        let opt_half = opt_2.size() / 2;
        println!("Around instruction {}:", opt_half);
        for j in (opt_half - 3).max(0)..(opt_half + 3).min(opt_2.size()) {
            let inst = &opt_2.data().instructions()[j];
            let qubits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();
            let params: Vec<String> = inst
                .gate
                .parameters
                .iter()
                .map(|p| format!("{:.4}", p))
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

        // Count CX gates
        let raw_cx_2 = raw_2
            .data()
            .instructions()
            .iter()
            .filter(|i| i.gate.gate_type == StandardGate::CX)
            .count();
        let opt_cx_2 = opt_2
            .data()
            .instructions()
            .iter()
            .filter(|i| i.gate.gate_type == StandardGate::CX)
            .count();
        println!(
            "\nRaw 2-step CX: {}, Optimized CX: {} (saved {})",
            raw_cx_2,
            opt_cx_2,
            raw_cx_2 - opt_cx_2
        );

        // Compare: 2 * optimized 1-step vs optimized 2-step
        let mut opt_1 = raw_1.clone();
        PassManager::level_2().run(&mut opt_1).unwrap();
        PassManager::level_4().run(&mut opt_1).unwrap();
        // Concatenate by adding instructions from second copy
        let mut concat = opt_1.clone();
        for inst in opt_1.data().instructions() {
            concat.data_mut().add_instruction(inst.clone()).unwrap();
        }
        println!("2 × opt(1-step): {} instructions", concat.size());
        println!("opt(2-step): {} instructions", opt_2.size());
        println!(
            "Step-boundary savings: {} (merged from boundary cancellation)",
            concat.size() as i32 - opt_2.size() as i32
        );
    }
}
