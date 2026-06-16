//! Minimal test to isolate the BC unitary bug.
//! Compare: original block unitary vs KAK-computed unitary.
use myquat::circuit_optimization::{BlockConsolidationPass, CircuitPass};
use myquat::*;
use num_complex::Complex64;

fn main() {
    // Test 1: [H(0), H(1), CX(0,1), Rz(1, 0.5), CX(0,1), H(0), H(1)]
    println!("=== Test 1: XX-type block ===");
    let (orig, bc) = test_bc(&|c| {
        c.h(0).unwrap();
        c.h(1).unwrap();
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.5)).unwrap();
        c.cx(0, 1).unwrap();
        c.h(0).unwrap();
        c.h(1).unwrap();
    });
    print_results(&orig, &bc);

    // Test 2: [CX(0,1), Rz(1, 0.5), CX(0,1)] — palindrome
    println!("\n=== Test 2: Palindrome CX-Rz-CX ===");
    let (orig2, bc2) = test_bc(&|c| {
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.5)).unwrap();
        c.cx(0, 1).unwrap();
    });
    print_results(&orig2, &bc2);

    // Test 3: [CX(0,1), Rz(1, 0.5), CX(0,1), H(0)] — asymmetric
    println!("\n=== Test 3: CX-Rz-CX-H ===");
    let (orig3, bc3) = test_bc(&|c| {
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.5)).unwrap();
        c.cx(0, 1).unwrap();
        c.h(0).unwrap();
    });
    print_results(&orig3, &bc3);

    // Test 4: [CX(0,1), Rz(1, 0.5), CX(0,1), H(0), H(1)] — H on both qubits
    println!("\n=== Test 4: CX-Rz-CX-H-H ===");
    let (orig4, bc4) = test_bc(&|c| {
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.5)).unwrap();
        c.cx(0, 1).unwrap();
        c.h(0).unwrap();
        c.h(1).unwrap();
    });
    print_results(&orig4, &bc4);

    // Test 5: [Rx(0), Rx(1), CX(0,1), Rz(1, 0.3), CX(0,1), Rx(0), Rx(1)] — YY block
    println!("\n=== Test 5: YY-type block ===");
    let pi2 = std::f64::consts::PI / 2.0;
    let (orig5, bc5) = test_bc(&|c| {
        c.rx(0, Parameter::Float(pi2)).unwrap();
        c.rx(1, Parameter::Float(pi2)).unwrap();
        c.cx(0, 1).unwrap();
        c.rz(1, Parameter::Float(0.3)).unwrap();
        c.cx(0, 1).unwrap();
        c.rx(0, Parameter::Float(-pi2)).unwrap();
        c.rx(1, Parameter::Float(-pi2)).unwrap();
    });
    print_results(&orig5, &bc5);
}

fn test_bc<F>(build: &F) -> (QuantumCircuit, QuantumCircuit)
where
    F: Fn(&mut QuantumCircuit),
{
    let mut c = QuantumCircuit::new(2, 0);
    build(&mut c);
    let mut bc = c.clone();
    BlockConsolidationPass::new().run(&mut bc).unwrap();
    (c, bc)
}

fn print_results(orig: &QuantumCircuit, bc: &QuantumCircuit) {
    let ou = orig.unitary(&std::collections::HashMap::new()).unwrap();
    let bu = bc.unitary(&std::collections::HashMap::new()).unwrap();
    let err = frobenius_diff(&ou, &bu);
    let err_pi = frobenius_diff_phase_insensitive(&ou, &bu);
    let orig_cx = count_cx(orig);
    let bc_cx = count_cx(bc);
    println!("  Original: {}g/{}cx", orig.size(), orig_cx);
    println!(
        "  BC:       {}g/{}cx, frob_err={:.6}, phase_insensitive={:.6}",
        bc.size(),
        bc_cx,
        err,
        err_pi
    );
    print!("  BC gates: ");
    for inst in bc.data().instructions() {
        let q: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();
        let p: Vec<String> = inst
            .gate
            .parameters
            .iter()
            .map(|p| format!("{:.3}", p))
            .collect();
        let ps = if p.is_empty() {
            String::new()
        } else {
            format!("({})", p.join(","))
        };
        print!("{:?}{}@{:?} ", inst.gate.gate_type, ps, q);
    }
    println!();
    if err_pi > 0.01 {
        println!("  ** BUG: phase-insensitive error > 0.01 **");
    }
}

fn count_cx(c: &QuantumCircuit) -> usize {
    c.data()
        .instructions()
        .iter()
        .filter(|i| i.gate.gate_type == StandardGate::CX)
        .count()
}

fn frobenius_diff(a: &ndarray::Array2<Complex64>, b: &ndarray::Array2<Complex64>) -> f64 {
    let diff = a - b;
    let n = diff.nrows() as f64;
    (diff.iter().map(|c| c.norm_sqr()).sum::<f64>() / n).sqrt()
}

/// Phase-insensitive frobenius norm: min_φ ||U - e^{iφ}V||_F / sqrt(N)
/// ||U - e^{iφ}V||_F^2 = ||U||_F^2 + ||V||_F^2 - 2 Re(e^{iφ} tr(V U^†))
///                     = 2N - 2 |tr(V U^†)|  (with optimal φ)
fn frobenius_diff_phase_insensitive(
    a: &ndarray::Array2<Complex64>,
    b: &ndarray::Array2<Complex64>,
) -> f64 {
    let n = a.nrows();
    // tr(V U^†) = sum_{i,j} b[i,j] * conj(a[i,j])
    let mut trace = Complex64::new(0.0, 0.0);
    for i in 0..n {
        for j in 0..n {
            trace += b[[i, j]] * a[[i, j]].conj();
        }
    }
    let n_f = n as f64;
    let val = 2.0 * n_f - 2.0 * trace.norm();
    if val < 0.0 {
        0.0
    } else {
        (val / n_f).sqrt()
    }
}
