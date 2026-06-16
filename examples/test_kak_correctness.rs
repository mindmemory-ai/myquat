//! Direct test: does Shannon decomposition produce correct unitaries?
use myquat::*;
use ndarray::Array2;
use num_complex::Complex64;

fn main() {
    // Test case 1: CX(0,1), Rz(1, 0.5), CX(0,1) — Rzz rotation
    println!("=== Test 1: CX.Rz.CX (Rzz) ===");
    {
        let mut c = QuantumCircuit::new(2, 0);
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.5)).unwrap();
        c.cx(0, 1).unwrap();
        let u = c.unitary(&std::collections::HashMap::new()).unwrap();
        test_via_circuit(&u);
    }

    // Test case 2: H(0),H(1),CX(0,1),Rz(1,0.5),CX(0,1),H(0),H(1)
    println!("\n=== Test 2: H.H.CX.Rz.CX.H.H ===");
    {
        let mut c = QuantumCircuit::new(2, 0);
        c.h(0).unwrap();
        c.h(1).unwrap();
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.5)).unwrap();
        c.cx(0, 1).unwrap();
        c.h(0).unwrap();
        c.h(1).unwrap();
        let u = c.unitary(&std::collections::HashMap::new()).unwrap();
        test_via_circuit(&u);
    }

    // Test case 3: CX(0,1),Rz(1,0.5),CX(0,1),H(0) — asymmetric
    println!("\n=== Test 3: CX.Rz.CX.H(0) ===");
    {
        let mut c = QuantumCircuit::new(2, 0);
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.5)).unwrap();
        c.cx(0, 1).unwrap();
        c.h(0).unwrap();
        let u = c.unitary(&std::collections::HashMap::new()).unwrap();
        test_via_circuit(&u);
    }
}

fn test_via_circuit(unitary: &Array2<Complex64>) {
    use crate::two_qubit_decompose::{TwoQubitBasis, TwoQubitDecomposer, TwoQubitMatrixBuilder};
    let decomposer = TwoQubitDecomposer::new(TwoQubitBasis::CX);

    let shannon = match decomposer.shannon_decomposition(unitary.view()) {
        Ok(s) => s,
        Err(e) => {
            println!("  Shannon failed: {:?}", e);
            return;
        }
    };
    println!("  num_cnots = {}", shannon.num_cnots);

    let gates = TwoQubitDecomposer::synthesize_shannon_to_gates(&shannon);

    // Build circuit from gates and compute unitary
    let mut c = QuantumCircuit::new(2, 0);
    for (gate, angle, qubit) in &gates {
        match gate {
            crate::gates::StandardGate::Rz => {
                c.rz(*qubit, Parameter::Float(*angle)).unwrap();
            }
            crate::gates::StandardGate::Ry => {
                c.ry(*qubit, Parameter::Float(*angle)).unwrap();
            }
            crate::gates::StandardGate::CX => {
                c.cx(0, 1).unwrap();
            }
            _ => {}
        }
    }
    let reconstructed = c.unitary(&std::collections::HashMap::new()).unwrap();

    let diff = unitary - &reconstructed;
    let n = 4.0_f64;
    let frob = (diff.iter().map(|c| c.norm_sqr()).sum::<f64>() / n).sqrt();

    let mut trace = Complex64::new(0.0, 0.0);
    for i in 0..4 {
        for j in 0..4 {
            trace += reconstructed[[i, j]] * unitary[[i, j]].conj();
        }
    }
    let pi = (2.0 * n - 2.0 * trace.norm()).max(0.0).sqrt() / n.sqrt();

    println!(
        "  Synthesized {} gates, frob={:.6}, pi={:.6}",
        gates.len(),
        frob,
        pi
    );
    for (gate, angle, qubit) in gates.iter().take(12) {
        println!("    {:?}({:.4}) on q{}", gate, angle, qubit);
    }
    if pi > 0.001 {
        println!("  ** BUG: Shannon roundtrip error > 0.001 **");
    } else {
        println!("  ** OK **");
    }
}
