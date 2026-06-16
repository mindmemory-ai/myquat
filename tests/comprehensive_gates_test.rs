use myquat::{gates::Gate, Parameter, QuantumCircuit};
use std::collections::HashMap;
use std::f64::consts::PI;

#[test]
fn test_all_single_qubit_gates() {
    let symbols = HashMap::new();

    // Test all Pauli gates
    let i_gate = Gate::i();
    let i_matrix = i_gate.matrix(&symbols).unwrap();
    assert_eq!(i_matrix.dim(), (2, 2));

    let x_gate = Gate::x();
    let x_matrix = x_gate.matrix(&symbols).unwrap();
    assert_eq!(x_matrix.dim(), (2, 2));

    let y_gate = Gate::y();
    let y_matrix = y_gate.matrix(&symbols).unwrap();
    assert_eq!(y_matrix.dim(), (2, 2));

    let z_gate = Gate::z();
    let z_matrix = z_gate.matrix(&symbols).unwrap();
    assert_eq!(z_matrix.dim(), (2, 2));

    // Test Clifford gates
    let h_gate = Gate::h();
    let h_matrix = h_gate.matrix(&symbols).unwrap();
    assert_eq!(h_matrix.dim(), (2, 2));

    let s_gate = Gate::s();
    let s_matrix = s_gate.matrix(&symbols).unwrap();
    assert_eq!(s_matrix.dim(), (2, 2));

    let sdg_gate = Gate::sdg();
    let sdg_matrix = sdg_gate.matrix(&symbols).unwrap();
    assert_eq!(sdg_matrix.dim(), (2, 2));

    let t_gate = Gate::t();
    let t_matrix = t_gate.matrix(&symbols).unwrap();
    assert_eq!(t_matrix.dim(), (2, 2));

    let tdg_gate = Gate::tdg();
    let tdg_matrix = tdg_gate.matrix(&symbols).unwrap();
    assert_eq!(tdg_matrix.dim(), (2, 2));

    // Test rotation gates
    let rx_gate = Gate::rx(Parameter::new_float(PI / 4.0));
    let rx_matrix = rx_gate.matrix(&symbols).unwrap();
    assert_eq!(rx_matrix.dim(), (2, 2));

    let ry_gate = Gate::ry(Parameter::new_float(PI / 4.0));
    let ry_matrix = ry_gate.matrix(&symbols).unwrap();
    assert_eq!(ry_matrix.dim(), (2, 2));

    let rz_gate = Gate::rz(Parameter::new_float(PI / 4.0));
    let rz_matrix = rz_gate.matrix(&symbols).unwrap();
    assert_eq!(rz_matrix.dim(), (2, 2));

    let p_gate = Gate::p(Parameter::new_float(PI / 4.0));
    let p_matrix = p_gate.matrix(&symbols).unwrap();
    assert_eq!(p_matrix.dim(), (2, 2));

    // Test universal gates
    let u1_gate = Gate::u1(Parameter::new_float(PI / 4.0));
    let u1_matrix = u1_gate.matrix(&symbols).unwrap();
    assert_eq!(u1_matrix.dim(), (2, 2));

    let u2_gate = Gate::u2(Parameter::new_float(0.0), Parameter::new_float(PI));
    let u2_matrix = u2_gate.matrix(&symbols).unwrap();
    assert_eq!(u2_matrix.dim(), (2, 2));

    let u3_gate = Gate::u3(
        Parameter::new_float(PI / 2.0),
        Parameter::new_float(0.0),
        Parameter::new_float(PI),
    );
    let u3_matrix = u3_gate.matrix(&symbols).unwrap();
    assert_eq!(u3_matrix.dim(), (2, 2));

    let u_gate = Gate::u(
        Parameter::new_float(PI / 2.0),
        Parameter::new_float(0.0),
        Parameter::new_float(PI),
    );
    let u_matrix = u_gate.matrix(&symbols).unwrap();
    assert_eq!(u_matrix.dim(), (2, 2));
}

#[test]
fn test_all_two_qubit_gates() {
    let symbols = HashMap::new();

    // Test controlled gates
    let cx_gate = Gate::cx();
    let cx_matrix = cx_gate.matrix(&symbols).unwrap();
    assert_eq!(cx_matrix.dim(), (4, 4));

    let cy_gate = Gate::cy();
    let cy_matrix = cy_gate.matrix(&symbols).unwrap();
    assert_eq!(cy_matrix.dim(), (4, 4));

    let cz_gate = Gate::cz();
    let cz_matrix = cz_gate.matrix(&symbols).unwrap();
    assert_eq!(cz_matrix.dim(), (4, 4));

    let ch_gate = Gate::ch();
    let ch_matrix = ch_gate.matrix(&symbols).unwrap();
    assert_eq!(ch_matrix.dim(), (4, 4));

    // Test controlled rotation gates
    let crx_gate = Gate::crx(Parameter::new_float(PI / 4.0));
    let crx_matrix = crx_gate.matrix(&symbols).unwrap();
    assert_eq!(crx_matrix.dim(), (4, 4));

    let cry_gate = Gate::cry(Parameter::new_float(PI / 4.0));
    let cry_matrix = cry_gate.matrix(&symbols).unwrap();
    assert_eq!(cry_matrix.dim(), (4, 4));

    let crz_gate = Gate::crz(Parameter::new_float(PI / 4.0));
    let crz_matrix = crz_gate.matrix(&symbols).unwrap();
    assert_eq!(crz_matrix.dim(), (4, 4));

    let cp_gate = Gate::cp(Parameter::new_float(PI / 4.0));
    let cp_matrix = cp_gate.matrix(&symbols).unwrap();
    assert_eq!(cp_matrix.dim(), (4, 4));

    // Test swap gates
    let swap_gate = Gate::swap();
    let swap_matrix = swap_gate.matrix(&symbols).unwrap();
    assert_eq!(swap_matrix.dim(), (4, 4));

    let iswap_gate = Gate::iswap();
    let iswap_matrix = iswap_gate.matrix(&symbols).unwrap();
    assert_eq!(iswap_matrix.dim(), (4, 4));
}

#[test]
fn test_all_three_qubit_gates() {
    let symbols = HashMap::new();

    // Test three-qubit gates
    let ccx_gate = Gate::ccx();
    let ccx_matrix = ccx_gate.matrix(&symbols).unwrap();
    assert_eq!(ccx_matrix.dim(), (8, 8));

    let cswap_gate = Gate::cswap();
    let cswap_matrix = cswap_gate.matrix(&symbols).unwrap();
    assert_eq!(cswap_matrix.dim(), (8, 8));

    // Test multi-controlled gates
    let mcx_gate = Gate::mcx();
    let mcx_matrix = mcx_gate.matrix(&symbols).unwrap();
    assert_eq!(mcx_matrix.dim(), (8, 8));

    let mcy_gate = Gate::mcy();
    let mcy_matrix = mcy_gate.matrix(&symbols).unwrap();
    assert_eq!(mcy_matrix.dim(), (8, 8));

    let mcz_gate = Gate::mcz();
    let mcz_matrix = mcz_gate.matrix(&symbols).unwrap();
    assert_eq!(mcz_matrix.dim(), (8, 8));
}

#[test]
fn test_circuit_with_all_gates() {
    let mut circuit = QuantumCircuit::new(4, 4);

    // Add single-qubit gates
    circuit.i(0).unwrap();
    circuit.x(0).unwrap();
    circuit.y(0).unwrap();
    circuit.z(0).unwrap();
    circuit.h(0).unwrap();
    circuit.s(0).unwrap();
    circuit.sdg(0).unwrap();
    circuit.t(0).unwrap();
    circuit.tdg(0).unwrap();

    // Add rotation gates
    circuit.rx(0, Parameter::new_float(PI / 4.0)).unwrap();
    circuit.ry(0, Parameter::new_float(PI / 4.0)).unwrap();
    circuit.rz(0, Parameter::new_float(PI / 4.0)).unwrap();
    circuit.p(0, Parameter::new_float(PI / 4.0)).unwrap();

    // Add universal gates
    circuit.u1(0, Parameter::new_float(PI / 4.0)).unwrap();
    circuit
        .u2(0, Parameter::new_float(0.0), Parameter::new_float(PI))
        .unwrap();
    circuit
        .u3(
            0,
            Parameter::new_float(PI / 2.0),
            Parameter::new_float(0.0),
            Parameter::new_float(PI),
        )
        .unwrap();
    circuit
        .u(
            0,
            Parameter::new_float(PI / 2.0),
            Parameter::new_float(0.0),
            Parameter::new_float(PI),
        )
        .unwrap();

    // Add two-qubit gates
    circuit.cx(0, 1).unwrap();
    circuit.cy(0, 1).unwrap();
    circuit.cz(0, 1).unwrap();
    circuit.ch(0, 1).unwrap();

    // Add controlled rotation gates
    circuit.crx(0, 1, Parameter::new_float(PI / 4.0)).unwrap();
    circuit.cry(0, 1, Parameter::new_float(PI / 4.0)).unwrap();
    circuit.crz(0, 1, Parameter::new_float(PI / 4.0)).unwrap();
    circuit.cp(0, 1, Parameter::new_float(PI / 4.0)).unwrap();

    // Add swap gates
    circuit.swap(0, 1).unwrap();
    circuit.iswap(0, 1).unwrap();

    // Add three-qubit gates
    circuit.ccx(0, 1, 2).unwrap();
    circuit.cswap(0, 1, 2).unwrap();
    circuit.mcx(0, 1, 2).unwrap();
    circuit.mcy(0, 1, 2).unwrap();
    circuit.mcz(0, 1, 2).unwrap();

    // Test aliases
    circuit.cnot(0, 1).unwrap();
    circuit.toffoli(0, 1, 2).unwrap();
    circuit.fredkin(0, 1, 2).unwrap();

    // Add measurements
    circuit.measure_all().unwrap();

    // Verify circuit properties
    assert_eq!(circuit.num_qubits(), 4);
    assert_eq!(circuit.num_clbits(), 4);
    assert!(circuit.size() > 30); // Should have many gates
    println!("Circuit with all gates created successfully!");
    println!("Total gates: {}", circuit.size());
    println!("Circuit depth: {}", circuit.depth());
}

#[test]
fn test_gate_aliases() {
    let symbols = HashMap::new();

    // Test that aliases produce the same matrices
    let cx1 = Gate::cx();
    let cx2 = Gate::cnot();
    let cx1_matrix = cx1.matrix(&symbols).unwrap();
    let cx2_matrix = cx2.matrix(&symbols).unwrap();

    for i in 0..4 {
        for j in 0..4 {
            assert!((cx1_matrix[[i, j]] - cx2_matrix[[i, j]]).norm() < 1e-10);
        }
    }

    let ccx1 = Gate::ccx();
    let ccx2 = Gate::toffoli();
    let ccx1_matrix = ccx1.matrix(&symbols).unwrap();
    let ccx2_matrix = ccx2.matrix(&symbols).unwrap();

    for i in 0..8 {
        for j in 0..8 {
            assert!((ccx1_matrix[[i, j]] - ccx2_matrix[[i, j]]).norm() < 1e-10);
        }
    }

    let cswap1 = Gate::cswap();
    let cswap2 = Gate::fredkin();
    let cswap1_matrix = cswap1.matrix(&symbols).unwrap();
    let cswap2_matrix = cswap2.matrix(&symbols).unwrap();

    for i in 0..8 {
        for j in 0..8 {
            assert!((cswap1_matrix[[i, j]] - cswap2_matrix[[i, j]]).norm() < 1e-10);
        }
    }
}

#[test]
fn test_parametric_gates_with_symbols() {
    let mut symbols = HashMap::new();
    symbols.insert("theta".to_string(), PI / 3.0);
    symbols.insert("phi".to_string(), PI / 6.0);
    symbols.insert("lambda".to_string(), PI / 4.0);

    // Test parametric single-qubit gates
    let rx_sym = Gate::rx(Parameter::new_symbol("theta"));
    let rx_matrix = rx_sym.matrix(&symbols).unwrap();
    assert_eq!(rx_matrix.dim(), (2, 2));

    let ry_sym = Gate::ry(Parameter::new_symbol("phi"));
    let ry_matrix = ry_sym.matrix(&symbols).unwrap();
    assert_eq!(ry_matrix.dim(), (2, 2));

    let rz_sym = Gate::rz(Parameter::new_symbol("lambda"));
    let rz_matrix = rz_sym.matrix(&symbols).unwrap();
    assert_eq!(rz_matrix.dim(), (2, 2));

    // Test parametric two-qubit gates
    let crx_sym = Gate::crx(Parameter::new_symbol("theta"));
    let crx_matrix = crx_sym.matrix(&symbols).unwrap();
    assert_eq!(crx_matrix.dim(), (4, 4));

    let cry_sym = Gate::cry(Parameter::new_symbol("phi"));
    let cry_matrix = cry_sym.matrix(&symbols).unwrap();
    assert_eq!(cry_matrix.dim(), (4, 4));

    let crz_sym = Gate::crz(Parameter::new_symbol("lambda"));
    let crz_matrix = crz_sym.matrix(&symbols).unwrap();
    assert_eq!(crz_matrix.dim(), (4, 4));

    // Test universal gates with symbols
    let u_sym = Gate::u(
        Parameter::new_symbol("theta"),
        Parameter::new_symbol("phi"),
        Parameter::new_symbol("lambda"),
    );
    let u_matrix = u_sym.matrix(&symbols).unwrap();
    assert_eq!(u_matrix.dim(), (2, 2));
}

#[test]
fn test_quantum_algorithms_with_new_gates() {
    // Test Bell state preparation with all variants
    for i in 0..4 {
        let mut circuit = QuantumCircuit::new(2, 2);

        // Prepare different Bell states
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();

        match i {
            0 => {}                     // |Φ+⟩
            1 => circuit.z(0).unwrap(), // |Φ-⟩
            2 => circuit.x(1).unwrap(), // |Ψ+⟩
            3 => {
                circuit.z(0).unwrap();
                circuit.x(1).unwrap();
            } // |Ψ-⟩
            _ => unreachable!(),
        }

        circuit.measure_all().unwrap();
        assert!(circuit.size() >= 4); // H + CX + measurements + possible extra gates
    }

    // Test quantum teleportation circuit
    let mut teleport = QuantumCircuit::new(3, 3);

    // Create entangled pair
    teleport.h(1).unwrap();
    teleport.cx(1, 2).unwrap();

    // Bell measurement
    teleport.cx(0, 1).unwrap();
    teleport.h(0).unwrap();
    teleport.measure(0, 0).unwrap();
    teleport.measure(1, 1).unwrap();

    assert_eq!(teleport.num_qubits(), 3);
    assert!(teleport.size() >= 5);

    // Test Grover oracle with Toffoli
    let mut grover = QuantumCircuit::new(4, 0);

    // Initialize superposition
    for i in 0..3 {
        grover.h(i).unwrap();
    }

    // Oracle marking |111⟩
    grover.ccx(0, 1, 3).unwrap();
    grover.cx(2, 3).unwrap();
    grover.z(3).unwrap();
    grover.cx(2, 3).unwrap();
    grover.ccx(0, 1, 3).unwrap();

    assert_eq!(grover.num_qubits(), 4);
    assert!(grover.size() >= 8);
}

#[test]
fn test_circuit_unitary_computation() {
    // Test small circuits can compute unitary matrices
    let mut circuit = QuantumCircuit::new(2, 0);
    circuit.h(0).unwrap();
    circuit.cx(0, 1).unwrap();

    let symbols = HashMap::new();
    let unitary = circuit.unitary(&symbols).unwrap();

    assert_eq!(unitary.dim(), (4, 4));

    // Verify it's unitary (U†U = I)
    use myquat::gates::GateMatrix;
    assert!(GateMatrix::is_unitary(&unitary, 1e-10));
}
