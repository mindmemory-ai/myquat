//! Diagnose PauliLevel vs GateLevel overhead for H2_4q
use myquat::error::Result;
use myquat::hamiltonian::pauli_synthesis::{BlockGroupingStrategy, PauliGateBudget};
use myquat::hamiltonian::{
    CompilationStrategy, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString,
    TrotterOrder,
};
use num_complex::Complex64;

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

    // == PauliLevel, 1 step, NO optimization (RAW) ==
    let config = CompilerConfig {
        trotter_order: TrotterOrder::First,
        trotter_steps: 1,
        evolution_time: 1.0,
        optimization_strategy: CompilationStrategy::PauliLevel,
        apply_circuit_optimization: false,
        group_commuting_terms: true,
        skip_identities: true,
        alternate_reverse_steps: false,
        clifford_enhanced_blocks: false,
        ..Default::default()
    };
    let compiler = HamiltonianCompiler::new(config.clone());
    let circuit = compiler.compile(&h)?;

    println!("=== PauliLevel 1-step RAW ===");
    println!(
        "Gates: {}, CX: {}, Depth: {}",
        circuit.size(),
        count_cx(&circuit),
        circuit.depth()
    );

    // Show instructions grouped by type
    let mut cx_count = 0;
    let mut rz_count = 0;
    let mut h_count = 0;
    let mut rx_count = 0;
    let mut other = 0;
    for inst in circuit.data().instructions().iter() {
        match inst.gate.gate_type {
            myquat::gates::StandardGate::CX => cx_count += 1,
            myquat::gates::StandardGate::Rz => rz_count += 1,
            myquat::gates::StandardGate::H => h_count += 1,
            myquat::gates::StandardGate::Rx => rx_count += 1,
            _ => other += 1,
        }
    }
    println!(
        "  CX: {}, Rz: {}, H: {}, Rx: {}, other: {}",
        cx_count, rz_count, h_count, rx_count, other
    );

    // == GateLevel, 1 step, NO optimization (RAW) ==
    let mut config2 = config.clone();
    config2.optimization_strategy = CompilationStrategy::GateLevel;
    let compiler2 = HamiltonianCompiler::new(config2);
    let circuit2 = compiler2.compile(&h)?;
    println!("\n=== GateLevel 1-step RAW ===");
    println!(
        "Gates: {}, CX: {}, Depth: {}",
        circuit2.size(),
        count_cx(&circuit2),
        circuit2.depth()
    );
    let mut cx2 = 0;
    let mut rz2 = 0;
    let mut h2 = 0;
    let mut rx2 = 0;
    let mut o2 = 0;
    for inst in circuit2.data().instructions().iter() {
        match inst.gate.gate_type {
            myquat::gates::StandardGate::CX => cx2 += 1,
            myquat::gates::StandardGate::Rz => rz2 += 1,
            myquat::gates::StandardGate::H => h2 += 1,
            myquat::gates::StandardGate::Rx => rx2 += 1,
            _ => o2 += 1,
        }
    }
    println!(
        "  CX: {}, Rz: {}, H: {}, Rx: {}, other: {}",
        cx2, rz2, h2, rx2, o2
    );

    // Show first 20 instructions from each for comparison
    println!("\n=== PauliLevel first 30 instructions ===");
    for (i, inst) in circuit.data().instructions().iter().enumerate().take(30) {
        let gate = format!("{:?}", inst.gate.gate_type);
        let qubits: Vec<String> = inst.qubits.iter().map(|q| q.0.to_string()).collect();
        let params: Vec<String> = inst
            .gate
            .parameters
            .iter()
            .map(|p| format!("{:?}", p))
            .collect();
        println!(
            "  {:3}: {:8} q[{}] params={}",
            i,
            gate,
            qubits.join(","),
            params.join(",")
        );
    }

    println!("\n=== GateLevel first 30 instructions ===");
    for (i, inst) in circuit2.data().instructions().iter().enumerate().take(30) {
        let gate = format!("{:?}", inst.gate.gate_type);
        let qubits: Vec<String> = inst.qubits.iter().map(|q| q.0.to_string()).collect();
        let params: Vec<String> = inst
            .gate
            .parameters
            .iter()
            .map(|p| format!("{:?}", p))
            .collect();
        println!(
            "  {:3}: {:8} q[{}] params={}",
            i,
            gate,
            qubits.join(","),
            params.join(",")
        );
    }
    Ok(())
}
