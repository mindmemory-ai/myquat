//! Sanity check: verify QuantumCircuit::unitary() works correctly
//! by testing a simple known circuit and then examining raw synthesis output
//! Author: gA4ss

use myquat::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
use myquat::hamiltonian::{
    CompilationStrategy, CompilerConfig, Hamiltonian, HamiltonianCompiler, PauliString,
    TrotterOrder,
};
use myquat::Result;
use ndarray::Array2;
use num_complex::Complex64;
use std::collections::HashMap;

fn phase_insensitive_dist(a: &Array2<Complex64>, b: &Array2<Complex64>) -> f64 {
    let n = a.nrows();
    let mut trace = Complex64::new(0.0, 0.0);
    for i in 0..n {
        for j in 0..n {
            trace += a[(i, j)].conj() * b[(i, j)];
        }
    }
    let angle = trace.im.atan2(trace.re);
    let phase = Complex64::new(angle.cos(), angle.sin()); // e^{i*angle}
    let scale = 1.0 / (n as f64).sqrt();
    let mut diff_norm_sq = 0.0f64;
    for i in 0..n {
        for j in 0..n {
            let d = a[(i, j)] - phase * b[(i, j)];
            diff_norm_sq += d.norm_sqr();
        }
    }
    diff_norm_sq.sqrt() * scale
}

fn compute_unitary(circuit: &myquat::QuantumCircuit) -> Option<Array2<Complex64>> {
    circuit.unitary(&HashMap::new()).ok()
}

fn main() -> Result<()> {
    // === Sanity Check 1: Simple Bell state circuit ===
    println!("=== Sanity Check: Bell State Circuit ===");
    let mut bell = myquat::QuantumCircuit::new(2, 0);
    bell.h(0)?;
    bell.cx(0, 1)?;
    let u_bell = compute_unitary(&bell);
    if let Some(u) = &u_bell {
        // Bell state preparation: |00> -> (|00>+|11>)/√2
        // First column of unitary should be [1/√2, 0, 0, 1/√2]
        println!("Bell circuit unitary (first column):");
        for i in 0..4 {
            println!("  [{}] = {:.6} + {:.6}i", i, u[(i, 0)].re, u[(i, 0)].im);
        }
        let expected = 1.0 / (2.0f64).sqrt();
        let err00 = (u[(0, 0)].re - expected).abs();
        let err30 = (u[(3, 0)].re - expected).abs();
        println!("  Error at (0,0): {:.2e}, (3,0): {:.2e}", err00, err30);
        if err00 < 1e-10 && err30 < 1e-10 {
            println!("  Bell state unitary: OK");
        } else {
            println!("  Bell state unitary: WRONG!");
        }
    }

    // === Sanity Check 2: Rz gate ===
    println!("\n=== Sanity Check: Rz(pi/2) gate ===");
    let mut rz = myquat::QuantumCircuit::new(1, 0);
    rz.rz(
        0,
        myquat::parameter::Parameter::Float(std::f64::consts::PI / 2.0),
    )?;
    let u_rz = compute_unitary(&rz);
    if let Some(u) = &u_rz {
        println!("Rz(pi/2) unitary:");
        println!(
            "  [{:.6}+{:.6}i, {:.6}+{:.6}i]",
            u[(0, 0)].re,
            u[(0, 0)].im,
            u[(0, 1)].re,
            u[(0, 1)].im
        );
        println!(
            "  [{:.6}+{:.6}i, {:.6}+{:.6}i]",
            u[(1, 0)].re,
            u[(1, 0)].im,
            u[(1, 1)].re,
            u[(1, 1)].im
        );
        // Expected: diag(exp(-i*pi/4), exp(i*pi/4))
        let e1 = Complex64::new(0.0, -std::f64::consts::PI / 4.0).exp();
        let e2 = Complex64::new(0.0, std::f64::consts::PI / 4.0).exp();
        let err1 = (u[(0, 0)] - e1).norm();
        let err2 = (u[(1, 1)] - e2).norm();
        println!("  Error: {:.2e}, {:.2e}", err1, err2);
        if err1 < 1e-10 && err2 < 1e-10 {
            println!("  Rz gate unitary: OK");
        } else {
            println!("  Rz gate unitary: WRONG!");
        }
    }

    // === Now examine H2_4q raw synthesis ===
    println!("\n=== H2_4q Raw Synthesis (1 step, no optimization) ===");
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
    for (ps, coeff) in &terms {
        hamiltonian.add_term(PauliString::from_str(ps)?, Complex64::new(*coeff, 0.0))?;
    }

    // Single step to examine structure
    let mut config = CompilerConfig {
        trotter_order: TrotterOrder::First,
        trotter_steps: 1,
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

    println!("\n--- PauliLevel 1-step RAW ---");
    let compiler = HamiltonianCompiler::new(config.clone());
    let c = compiler.compile(&hamiltonian)?;
    println!("Gates: {}, Depth: {}", c.size(), c.depth());
    println!("First 20 gates:");
    for (i, inst) in c.data().instructions().iter().take(20).enumerate() {
        let params: Vec<String> = inst
            .gate
            .parameters
            .iter()
            .map(|p| format!("{:.4}", p))
            .collect();
        let qubits: Vec<usize> = inst
            .gate
            .parameters
            .iter()
            .enumerate()
            .map(|(j, _)| {
                if j < inst.qubits.len() {
                    inst.qubits[j].index()
                } else {
                    0
                }
            })
            .collect();
        println!(
            "  {:3}: {:?} params={:?} qubits={:?}",
            i,
            inst.gate.gate_type,
            params,
            inst.qubits.iter().map(|q| q.index()).collect::<Vec<_>>()
        );
    }
    println!("... ({} total gates)", c.size());

    // Verify unitary from 50-step circuit has reasonable structure
    // Compare PauliLevel vs GateLevel for 10 steps
    println!("\n--- Comparing 10-step unitaries ---");
    config.trotter_steps = 10;

    // PauliLevel
    let c_p = HamiltonianCompiler::new(config.clone()).compile(&hamiltonian)?;
    let u_p = compute_unitary(&c_p);
    println!(
        "PauliLevel: {} gates, {} CX",
        c_p.size(),
        c_p.data()
            .instructions()
            .iter()
            .filter(|i| matches!(i.gate.gate_type, myquat::gates::StandardGate::CX))
            .count()
    );

    // GateLevel
    config.optimization_strategy = CompilationStrategy::GateLevel;
    let c_g = HamiltonianCompiler::new(config.clone()).compile(&hamiltonian)?;
    let u_g = compute_unitary(&c_g);
    println!(
        "GateLevel:  {} gates, {} CX",
        c_g.size(),
        c_g.data()
            .instructions()
            .iter()
            .filter(|i| matches!(i.gate.gate_type, myquat::gates::StandardGate::CX))
            .count()
    );

    // Check if they differ by just term ordering within each step
    if let (Some(up), Some(ug)) = (&u_p, &u_g) {
        let d = phase_insensitive_dist(up, ug);
        println!("Phase-insensitive distance: {:.6}", d);

        // For first-order Trotter with 10 steps, the two unitaries
        // should be very similar if they implement the same formula
        // but with different term ordering within each step.
        // The difference should be O(t²/n) = O(0.01) per step.
        // Over 10 steps, the total difference could be O(0.1).
        if d > 0.5 {
            println!(
                "WARNING: Distance {:.3} > 0.5 - possible bug in synthesis!",
                d
            );
            println!("Checking if either circuit has an obvious issue...");
            // Print first and last few gates of each
            println!("\nPauliLevel first 5 gates:");
            for (i, inst) in c_p.data().instructions().iter().take(5).enumerate() {
                let qbits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();
                println!("  {}: {:?} qubits={:?}", i, inst.gate.gate_type, qbits);
            }
            println!("GateLevel first 5 gates:");
            for (i, inst) in c_g.data().instructions().iter().take(5).enumerate() {
                let qbits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();
                println!("  {}: {:?} qubits={:?}", i, inst.gate.gate_type, qbits);
            }
        }
    }

    Ok(())
}
