//! Deep dive: what does each optimization pass do to PauliLevel vs GateLevel?
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
        println!("\n========== {} (1 step) ==========", label);

        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 1,
            evolution_time: 1.0,
            optimization_strategy: strat,
            apply_circuit_optimization: false,
            ..CompilerConfig::default()
        };
        let raw = HamiltonianCompiler::new(config).compile(&h).unwrap();

        // Apply individual optimizations from lv2 one by one
        let mut after_sqo = raw.clone();
        crate::single_qubit_optimizer::SingleQubitOptimizer::new()
            .run(&mut after_sqo)
            .unwrap();
        println!(
            "  Raw: {} → SingleQubitOptimizer: {} (delta: {})",
            raw.size(),
            after_sqo.size(),
            after_sqo.size() as i32 - raw.size() as i32
        );

        let mut after_cip = raw.clone();
        crate::circuit_optimization::CancelInversePairsPass::new()
            .run(&mut after_cip)
            .unwrap();
        println!(
            "  Raw: {} → CancelInversePairs: {} (delta: {})",
            raw.size(),
            after_cip.size(),
            after_cip.size() as i32 - raw.size() as i32
        );

        let mut after_cnot = raw.clone();
        crate::cnot_optimizer::CNOTOptimizer::new()
            .run(&mut after_cnot)
            .unwrap();
        println!(
            "  Raw: {} → CNOTOptimizer: {} (delta: {})",
            raw.size(),
            after_cnot.size(),
            after_cnot.size() as i32 - raw.size() as i32
        );

        let mut after_cc = raw.clone();
        crate::circuit_optimization::CommutativeCancellationPass::new()
            .run(&mut after_cc)
            .unwrap();
        println!(
            "  Raw: {} → CommutativeCancellation: {} (delta: {})",
            raw.size(),
            after_cc.size(),
            after_cc.size() as i32 - raw.size() as i32
        );

        // Full lv2
        let mut lv2 = raw.clone();
        PassManager::level_2().run(&mut lv2).unwrap();
        println!(
            "  Raw: {} → Full lv2: {} (delta: {})",
            raw.size(),
            lv2.size(),
            lv2.size() as i32 - raw.size() as i32
        );

        // Full lv4
        let mut lv4 = raw.clone();
        PassManager::level_2().run(&mut lv4).unwrap();
        PassManager::level_4().run(&mut lv4).unwrap();
        println!(
            "  Raw: {} → Full lv2+4: {} (delta: {})",
            raw.size(),
            lv4.size(),
            lv4.size() as i32 - raw.size() as i32
        );

        // Show the raw instruction structure
        println!("\n  First 30 raw instructions:");
        for (j, inst) in raw.data().instructions().iter().take(30).enumerate() {
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
                "    {:3}: {:?}{} on {:?}",
                j, inst.gate.gate_type, param_str, qubits
            );
        }

        // Show lv4 optimized instructions
        println!("\n  lv2+4 optimized instructions:");
        for (j, inst) in lv4.data().instructions().iter().enumerate() {
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
                "    {:3}: {:?}{} on {:?}",
                j, inst.gate.gate_type, param_str, qubits
            );
        }
    }
}
