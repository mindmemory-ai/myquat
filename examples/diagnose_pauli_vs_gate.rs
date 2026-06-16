//! Diagnostic: Compare PauliLevel vs GateLevel gate breakdown for H2_4q
//! Author: gA4ss

use myquat::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
use myquat::hamiltonian::{
    CompilationStrategy, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString,
    TrotterOrder,
};
use myquat::Result;
use num_complex::Complex64;
use std::collections::HashMap;

fn count_gate_types(circuit: &myquat::QuantumCircuit) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for inst in circuit.data().instructions() {
        let name = format!("{:?}", inst.gate.gate_type);
        *counts.entry(name).or_insert(0) += 1;
    }
    counts
}

fn print_breakdown(label: &str, counts: &HashMap<String, usize>) {
    let total: usize = counts.values().sum();
    let cx = counts.get("CX").unwrap_or(&0);
    let rz = counts.get("Rz").unwrap_or(&0);
    let ry = counts.get("Ry").unwrap_or(&0);
    let rx = counts.get("Rx").unwrap_or(&0);
    let h = counts.get("H").unwrap_or(&0);
    let s = counts.get("S").unwrap_or(&0);
    let sdg = counts.get("Sdg").unwrap_or(&0);
    let x = counts.get("X").unwrap_or(&0);
    let y = counts.get("Y").unwrap_or(&0);
    let z = counts.get("Z").unwrap_or(&0);
    let known = cx + rz + ry + rx + h + s + sdg + x + y + z;
    let other = total - known;
    println!(
        "{}: {} total | CX:{} Rz:{} Ry:{} Rx:{} H:{} S:{} Sdg:{} X:{} Y:{} Z:{} other:{}",
        label, total, cx, rz, ry, rx, h, s, sdg, x, y, z, other
    );
}

fn main() -> Result<()> {
    // H2_4q Hamiltonian: 4 qubits, 15 terms
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

    for (pauli_str, coeff) in &terms {
        hamiltonian.add_term(
            PauliString::from_str(pauli_str)?,
            Complex64::new(*coeff, 0.0),
        )?;
    }

    // === RAW synthesis (no optimization) ===
    println!("=== RAW SYNTHESIS (no optimization) ===");

    let mut config = CompilerConfig {
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
        apply_circuit_optimization: false,
        auto_optimize_grouping: true,
        layout_aware_grouping: false,
        optimization_strategy: CompilationStrategy::PauliLevel,
        cross_step_synthesis: false,
        block_grouping_strategy: BlockGroupingStrategy::QWC,
        pauli_gadget_optimization: myquat::hamiltonian::GadgetOptimizationStrategy::IdenticalOnly,
        clifford_enhanced_blocks: false,
        ..Default::default()
    };

    let compiler = HamiltonianCompiler::new(config.clone());
    let c_pauli_raw = compiler.compile(&hamiltonian)?;
    let pauli_raw = count_gate_types(&c_pauli_raw);
    print_breakdown("PauliLevel RAW", &pauli_raw);

    config.optimization_strategy = CompilationStrategy::GateLevel;
    let compiler = HamiltonianCompiler::new(config.clone());
    let c_gate_raw = compiler.compile(&hamiltonian)?;
    let gate_raw = count_gate_types(&c_gate_raw);
    print_breakdown("GateLevel  RAW", &gate_raw);

    let raw_total_p: usize = pauli_raw.values().sum();
    let raw_total_g: usize = gate_raw.values().sum();
    println!(
        "RAW gap: +{} gates",
        raw_total_p as i64 - raw_total_g as i64
    );

    // === Optimized ===
    println!("\n=== WITH OPTIMIZATION (level_2 + level_4) ===");

    config.apply_circuit_optimization = true;
    config.optimization_strategy = CompilationStrategy::PauliLevel;
    let compiler = HamiltonianCompiler::new(config.clone());
    let c_pauli_opt = compiler.compile(&hamiltonian)?;
    let pauli_opt = count_gate_types(&c_pauli_opt);
    print_breakdown("PauliLevel OPT", &pauli_opt);
    let opt_total_p: usize = pauli_opt.values().sum();
    println!("  removed: {} gates", raw_total_p - opt_total_p);

    config.optimization_strategy = CompilationStrategy::GateLevel;
    let compiler = HamiltonianCompiler::new(config.clone());
    let c_gate_opt = compiler.compile(&hamiltonian)?;
    let gate_opt = count_gate_types(&c_gate_opt);
    print_breakdown("GateLevel  OPT", &gate_opt);
    let opt_total_g: usize = gate_opt.values().sum();
    println!("  removed: {} gates", raw_total_g - opt_total_g);

    println!(
        "\nOPT gap: +{} gates (PauliLevel has {} more)",
        opt_total_p as i64 - opt_total_g as i64,
        opt_total_p as i64 - opt_total_g as i64
    );

    // === Check if unitaries match ===
    let u_pauli = c_pauli_opt.unitary(&HashMap::new()).ok();
    let u_gate = c_gate_opt.unitary(&HashMap::new()).ok();
    if let (Some(up), Some(ug)) = (&u_pauli, &u_gate) {
        let n = up.nrows();
        let mut overlap = Complex64::new(0.0, 0.0);
        for i in 0..n {
            for j in 0..n {
                overlap += up[(i, j)].conj() * ug[(i, j)];
            }
        }
        let fidelity = overlap.norm() / (n as f64);
        let dist = (1.0 - fidelity).sqrt();
        println!("\nPhase-insensitive distance: {:.6}", dist);
        if dist > 1e-6 {
            println!("WARNING: PauliLevel and GateLevel produce DIFFERENT unitaries!");
        } else {
            println!("OK: Both produce the same unitary.");
        }
    }

    Ok(())
}
