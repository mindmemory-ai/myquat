//! Multi-Hamiltonian validation: verify no regressions from Phase 10 changes.
//! Tests Heisenberg-4, Heisenberg-6, Ising-4, Ising-6 with both GateLevel and PauliLevel.

use myquat::hamiltonian::{
    constructors, CompilationStrategy, CompilerConfig, HamiltonianCompiler, TrotterOrder,
};

fn count_cx(c: &myquat::QuantumCircuit) -> usize {
    c.data()
        .instructions()
        .iter()
        .filter(|i| matches!(i.gate.gate_type, myquat::gates::StandardGate::CX))
        .count()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let num_steps = 10;
    let dt = 1.0;

    println!(
        "=== Multi-Hamiltonian Validation ({} steps, 1st order) ===\n",
        num_steps
    );
    println!(
        "{:<22} {:>12} {:>12} {:>12} {:>12} {:>16} {:>16}",
        "Hamiltonian", "GL Gates", "GL CX", "PL Gates", "PL CX", "GL Depth", "PL Depth"
    );
    println!("{}", "-".repeat(112));

    let hams: Vec<(&str, Box<dyn Fn() -> myquat::hamiltonian::Hamiltonian>)> = vec![
        (
            "Heisenberg-4",
            Box::new(|| constructors::heisenberg_model(4, 1.0, 1.0, 1.0).unwrap()),
        ),
        (
            "Heisenberg-6",
            Box::new(|| constructors::heisenberg_model(6, 1.0, 1.0, 1.0).unwrap()),
        ),
        (
            "TFIM-4",
            Box::new(|| constructors::ising_model(4, 1.0, 0.5).unwrap()),
        ),
        (
            "TFIM-6",
            Box::new(|| constructors::ising_model(6, 1.0, 0.5).unwrap()),
        ),
    ];

    for (name, mk_ham) in &hams {
        let h = mk_ham();

        // GateLevel
        let mut gl_config = CompilerConfig::default();
        gl_config.trotter_order = TrotterOrder::First;
        gl_config.trotter_steps = num_steps;
        gl_config.evolution_time = dt;
        gl_config.optimization_strategy = CompilationStrategy::GateLevel;
        gl_config.apply_circuit_optimization = true;
        gl_config.alternate_reverse_steps = true;
        let gl_c = HamiltonianCompiler::new(gl_config).compile(&h)?;
        let gl_gates = gl_c.size();
        let gl_cx = count_cx(&gl_c);
        let gl_depth = gl_c.depth();

        // PauliLevel
        let mut pl_config = CompilerConfig::default();
        pl_config.trotter_order = TrotterOrder::First;
        pl_config.trotter_steps = num_steps;
        pl_config.evolution_time = dt;
        pl_config.optimization_strategy = CompilationStrategy::PauliLevel;
        pl_config.apply_circuit_optimization = true;
        pl_config.alternate_reverse_steps = true;
        let pl_c = HamiltonianCompiler::new(pl_config).compile(&h)?;
        let pl_gates = pl_c.size();
        let pl_cx = count_cx(&pl_c);
        let pl_depth = pl_c.depth();

        println!(
            "{:<22} {:>12} {:>12} {:>12} {:>12} {:>16} {:>16}",
            name, gl_gates, gl_cx, pl_gates, pl_cx, gl_depth, pl_depth
        );
    }

    Ok(())
}
