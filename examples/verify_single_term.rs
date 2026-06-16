//! Verify synthesis correctness for single Pauli terms
//! Each Pauli exponential has a known matrix form - verify the circuit matches.
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

fn compute_unitary(circuit: &myquat::QuantumCircuit) -> Option<Array2<Complex64>> {
    circuit.unitary(&HashMap::new()).ok()
}

/// Build exact exp(-iθ P) for a Pauli string via direct computation
fn exact_pauli_exp(pauli_str: &str, theta: f64) -> Array2<Complex64> {
    let n = pauli_str.len();
    let dim = 1usize << n;
    let mut mat = Array2::zeros((dim, dim));

    for i in 0..dim {
        // Compute P|i⟩: Pauli string acting on basis state |i>
        // For basis state i (binary representation), P flips bits and adds phases
        let mut phase = Complex64::new(1.0, 0.0);
        let mut flipped = i;
        for (pos, ch) in pauli_str.chars().enumerate() {
            let bit = (i >> (n - 1 - pos)) & 1;
            match ch {
                'I' => {}
                'X' => {
                    flipped ^= 1 << (n - 1 - pos);
                }
                'Y' => {
                    flipped ^= 1 << (n - 1 - pos);
                    if bit == 0 {
                        phase *= Complex64::new(0.0, 1.0);
                    } else {
                        phase *= Complex64::new(0.0, -1.0);
                    }
                }
                'Z' => {
                    if bit == 1 {
                        phase *= Complex64::new(-1.0, 0.0);
                    }
                }
                _ => panic!("Invalid Pauli char: {}", ch),
            }
        }
        mat[(flipped, i)] = phase;
    }

    // exp(-iθP) = cos(θ)I - i sin(θ)P
    let cos_t = Complex64::new(theta.cos(), 0.0);
    let sin_t = Complex64::new(-theta.sin(), 0.0); // -i sin(θ)
    let mut eye = Array2::eye(dim);
    eye.mapv_inplace(|x| x * cos_t);
    mat.mapv_inplace(|x| x * sin_t);
    eye + mat
}

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

fn main() -> Result<()> {
    // Test single Pauli terms with each strategy
    let test_terms = [
        ("ZZII", 0.5), // 2-qubit Z term on qubits 0,1
        ("IIZZ", 0.3), // 2-qubit Z term on qubits 2,3
        ("XXII", 0.2), // 2-qubit X term
        ("ZIII", 0.1), // single Z
    ];

    for (pauli_str, theta) in &test_terms {
        println!("\n=== Testing term: {} (θ={}) ===", pauli_str, theta);

        let exact = exact_pauli_exp(pauli_str, *theta);
        let n = pauli_str.len();

        for strategy in &[
            CompilationStrategy::PauliLevel,
            CompilationStrategy::GateLevel,
        ] {
            let mut h = Hamiltonian::new(n);
            h.add_term(
                PauliString::from_str(pauli_str)?,
                Complex64::new(*theta, 0.0),
            )?;

            let config = CompilerConfig {
                trotter_order: TrotterOrder::First,
                trotter_steps: 1,
                evolution_time: 1.0, // theta already includes time factor
                adaptive: false,
                adaptive_tolerance: 1e-3,
                min_step_size: 1e-6,
                max_step_size: 1.0,
                hbar: 1.0,
                skip_identities: false, // Don't skip identities for single-term test
                group_commuting_terms: true,
                apply_circuit_optimization: false,
                auto_optimize_grouping: true,
                layout_aware_grouping: false,
                optimization_strategy: *strategy,
                cross_step_synthesis: false,
                block_grouping_strategy: BlockGroupingStrategy::QWC,
                pauli_gadget_optimization:
                    myquat::hamiltonian::GadgetOptimizationStrategy::IdenticalOnly,
                clifford_enhanced_blocks: false,
                ..Default::default()
            };

            let compiler = HamiltonianCompiler::new(config);
            let circuit = compiler.compile(&h)?;
            let gates = circuit.size();
            let cx: usize = circuit
                .data()
                .instructions()
                .iter()
                .filter(|i| matches!(i.gate.gate_type, myquat::gates::StandardGate::CX))
                .count();

            if let Some(u) = compute_unitary(&circuit) {
                let dist = phase_insensitive_dist(&exact, &u);
                let status = if dist < 1e-6 {
                    "OK"
                } else if dist < 1e-3 {
                    "CLOSE"
                } else if dist < 0.1 {
                    "OFF"
                } else {
                    "WRONG!"
                };
                println!(
                    "  {:?}: {} gates ({} CX), dist={:.8} {}",
                    strategy, gates, cx, dist, status
                );

                if dist > 1e-3 {
                    // Print the circuit to debug
                    println!("    Circuit gates:");
                    for (i, inst) in circuit.data().instructions().iter().enumerate() {
                        let qbits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();
                        let params: Vec<String> = inst
                            .gate
                            .parameters
                            .iter()
                            .map(|p| format!("{:.6}", p))
                            .collect();
                        println!(
                            "      {}: {:?} params={:?} qubits={:?}",
                            i, inst.gate.gate_type, params, qbits
                        );
                    }

                    // Print first column of exact vs computed unitary
                    println!("    Exact unitary first column:");
                    for i in 0..(1 << n) {
                        println!(
                            "      [{}] = {:.6}+{:.6}i",
                            i,
                            exact[(i, 0)].re,
                            exact[(i, 0)].im
                        );
                    }
                    println!("    Computed unitary first column:");
                    for i in 0..(1 << n) {
                        println!("      [{}] = {:.6}+{:.6}i", i, u[(i, 0)].re, u[(i, 0)].im);
                    }
                }
            }
        }
    }

    Ok(())
}
