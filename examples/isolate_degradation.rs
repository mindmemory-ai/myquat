//! Isolate which optimization pass causes PauliLevel degradation at scale.
//! Key finding: 2 × opt(1-step) = 82, opt(2-step) = 86. 4 gates LOST.
//! Hypothesis: BlockConsolidationPass KAK gets confused at larger scale.
use myquat::circuit_optimization::{BlockConsolidationPass, CircuitPass, PassManager};
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

        // Synthesize 1, 2, 4, 10 steps
        for &steps in &[1, 2, 4, 10] {
            let config = CompilerConfig {
                trotter_order: TrotterOrder::First,
                trotter_steps: steps,
                evolution_time: 1.0,
                optimization_strategy: strat,
                apply_circuit_optimization: false,
                ..CompilerConfig::default()
            };
            let raw = HamiltonianCompiler::new(config).compile(&h).unwrap();

            // Apply ONLY BlockConsolidationPass (no lv2 first)
            let mut bc_only = raw.clone();
            BlockConsolidationPass::new().run(&mut bc_only).unwrap();
            let bc_cx = bc_only
                .data()
                .instructions()
                .iter()
                .filter(|i| i.gate.gate_type == StandardGate::CX)
                .count();

            // Apply lv2 + BlockConsolidationPass
            let mut lv2_bc = raw.clone();
            PassManager::level_2().run(&mut lv2_bc).unwrap();
            BlockConsolidationPass::new().run(&mut lv2_bc).unwrap();
            let lv2_bc_cx = lv2_bc
                .data()
                .instructions()
                .iter()
                .filter(|i| i.gate.gate_type == StandardGate::CX)
                .count();

            // Full lv2+lv4
            let mut full = raw.clone();
            PassManager::level_2().run(&mut full).unwrap();
            PassManager::level_4().run(&mut full).unwrap();

            // Expected: steps * opt(1-step)
            let mut opt_1 = {
                let config_1 = CompilerConfig {
                    trotter_order: TrotterOrder::First,
                    trotter_steps: 1,
                    evolution_time: 1.0,
                    optimization_strategy: strat,
                    apply_circuit_optimization: false,
                    ..CompilerConfig::default()
                };
                let raw_1 = HamiltonianCompiler::new(config_1).compile(&h).unwrap();
                let mut opt_1 = raw_1.clone();
                PassManager::level_2().run(&mut opt_1).unwrap();
                PassManager::level_4().run(&mut opt_1).unwrap();
                opt_1
            };
            let mut concat = opt_1.clone();
            for _ in 1..steps {
                for inst in opt_1.data().instructions() {
                    concat.data_mut().add_instruction(inst.clone()).unwrap();
                }
            }
            let concat_cx = concat
                .data()
                .instructions()
                .iter()
                .filter(|i| i.gate.gate_type == StandardGate::CX)
                .count();

            let raw_cx = raw
                .data()
                .instructions()
                .iter()
                .filter(|i| i.gate.gate_type == StandardGate::CX)
                .count();
            let full_cx = full
                .data()
                .instructions()
                .iter()
                .filter(|i| i.gate.gate_type == StandardGate::CX)
                .count();

            println!("{} steps: raw={}g/{}cx | BC_only={}g/{}cx | lv2+BC={}g/{}cx | full(lv2+4)={}g/{}cx | {}×opt1={}g/{}cx | lost={}g",
                steps,
                raw.size(), raw_cx,
                bc_only.size(), bc_cx,
                lv2_bc.size(), lv2_bc_cx,
                full.size(), full_cx,
                steps, concat.size(), concat_cx,
                full.size() as i32 - concat.size() as i32,
            );
        }
    }
}
