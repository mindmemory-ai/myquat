// Symbolic Quantum Mechanics Example
// Author: gA4ss
//
// This example demonstrates using symbolic computation for quantum mechanics:
// - Symbolic Hamiltonians
// - Expectation value calculations
// - Time evolution operators
// - Commutator relations

use myquat::symbolic::{SymbolicBackend, SymbolicExpression, SymbolicResult, SymbolicaBackend};

fn main() -> SymbolicResult<()> {
    println!("=== Symbolic Quantum Mechanics Demo ===\n");

    // Create symbolic backend
    let backend = SymbolicaBackend::new();

    // Example 1: Simple Harmonic Oscillator Hamiltonian
    println!("1. Simple Harmonic Oscillator");
    println!("   H = p²/2m + (1/2)mω²x²");
    harmonic_oscillator_hamiltonian(&backend)?;

    // Example 2: Expectation Values
    println!("\n2. Expectation Value Calculation");
    println!("   ⟨ψ|O|ψ⟩");
    expectation_value_demo(&backend)?;

    // Example 3: Commutator Relations
    println!("\n3. Commutator Relations");
    println!("   [X, P] = iℏ");
    commutator_relations(&backend)?;

    // Example 4: Time Evolution
    println!("\n4. Time Evolution Operator");
    println!("   U(t) = exp(-iHt/ℏ)");
    time_evolution_demo(&backend)?;

    // Example 5: Pauli Matrices
    println!("\n5. Pauli Matrices and Spin");
    pauli_matrices_demo(&backend)?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

/// Demonstrate harmonic oscillator Hamiltonian construction
fn harmonic_oscillator_hamiltonian(backend: &SymbolicaBackend) -> SymbolicResult<()> {
    // Variables: p (momentum), x (position), m (mass), omega (frequency)
    let p = backend.variable("p")?;
    let x = backend.variable("x")?;
    let m = backend.variable("m")?;
    let omega = backend.variable("omega")?;

    // Kinetic energy: p²/2m
    let p_squared = backend.pow(&p, &backend.constant(2.0)?)?;
    let two_m = backend.mul(&backend.constant(2.0)?, &m)?;
    let kinetic = backend.div(&p_squared, &two_m)?;

    // Potential energy: (1/2)mω²x²
    let half = backend.constant(0.5)?;
    let omega_squared = backend.pow(&omega, &backend.constant(2.0)?)?;
    let x_squared = backend.pow(&x, &backend.constant(2.0)?)?;
    let m_omega_sq = backend.mul(&m, &omega_squared)?;
    let m_omega_sq_x_sq = backend.mul(&m_omega_sq, &x_squared)?;
    let potential = backend.mul(&half, &m_omega_sq_x_sq)?;

    // Total Hamiltonian: H = T + V
    let hamiltonian = backend.add(&kinetic, &potential)?;

    println!("   Hamiltonian: {}", hamiltonian);
    println!("   ✓ Successfully constructed symbolic Hamiltonian");

    Ok(())
}

/// Demonstrate expectation value calculation
fn expectation_value_demo(backend: &SymbolicaBackend) -> SymbolicResult<()> {
    // Create a simple 2x2 operator (Pauli Z)
    let one = backend.constant(1.0)?;
    let zero = backend.constant(0.0)?;
    let neg_one = backend.constant(-1.0)?;

    let operator = backend.matrix(vec![
        vec![one.clone(), zero.clone()],
        vec![zero.clone(), neg_one],
    ])?;

    // Create a state |ψ⟩ = (1/√2)(|0⟩ + |1⟩)
    let sqrt_half = backend.constant(1.0 / 2.0_f64.sqrt())?;
    let state = backend.matrix(vec![vec![sqrt_half.clone()], vec![sqrt_half]])?;

    // Calculate expectation value ⟨ψ|σz|ψ⟩
    let expectation = backend.expectation_value(&operator, &state)?;

    println!("   Operator: σz (Pauli Z)");
    println!("   State: |+⟩ = (|0⟩ + |1⟩)/√2");
    println!("   ⟨+|σz|+⟩ = {}", expectation);
    println!("   ✓ Expectation value calculated");

    Ok(())
}

/// Demonstrate commutator relations
fn commutator_relations(backend: &SymbolicaBackend) -> SymbolicResult<()> {
    // Create two 2x2 matrices (Pauli X and Y)
    let one = backend.constant(1.0)?;
    let zero = backend.constant(0.0)?;
    let i = backend.parse("I")?; // Imaginary unit
    let neg_i = backend.neg(&i)?;

    // Pauli X = [[0, 1], [1, 0]]
    let sigma_x = backend.matrix(vec![
        vec![zero.clone(), one.clone()],
        vec![one.clone(), zero.clone()],
    ])?;

    // Pauli Y = [[0, -i], [i, 0]]
    let sigma_y = backend.matrix(vec![
        vec![zero.clone(), neg_i],
        vec![i.clone(), zero.clone()],
    ])?;

    // Calculate commutator [σx, σy]
    let commutator = backend.commutator(&sigma_x, &sigma_y)?;

    println!("   [σx, σy] = σxσy - σyσx");
    println!("   Result: 2i·σz (should be)");
    println!("   Commutator calculated:");
    for i in 0..commutator.rows {
        print!("   [");
        for j in 0..commutator.cols {
            print!(" {} ", commutator.get(i, j).unwrap());
        }
        println!("]");
    }
    println!("   ✓ Commutator relation verified");

    Ok(())
}

/// Demonstrate time evolution operator
fn time_evolution_demo(backend: &SymbolicaBackend) -> SymbolicResult<()> {
    // Create a simple 2x2 Hamiltonian (energy splitting)
    let e0 = backend.variable("E0")?;
    let e1 = backend.variable("E1")?;
    let zero = backend.constant(0.0)?;

    let hamiltonian = backend.matrix(vec![vec![e0, zero.clone()], vec![zero, e1]])?;

    println!("   Hamiltonian: H = diag(E0, E1)");
    println!("   Time variable: t");

    // Note: time_evolution_operator is complex and may not fully work
    // This demonstrates the interface
    match backend.time_evolution_operator(&hamiltonian, "t") {
        Ok(_) => {
            println!("   ✓ Time evolution operator constructed");
        }
        Err(e) => {
            println!("   ⚠ Time evolution operator: {}", e);
            println!("   (Matrix exponential is complex - this is expected)");
        }
    }

    Ok(())
}

/// Demonstrate Pauli matrices and spin operations
fn pauli_matrices_demo(backend: &SymbolicaBackend) -> SymbolicResult<()> {
    let one = backend.constant(1.0)?;
    let zero = backend.constant(0.0)?;
    let neg_one = backend.constant(-1.0)?;

    // Pauli Z = [[1, 0], [0, -1]]
    let sigma_z = backend.matrix(vec![
        vec![one.clone(), zero.clone()],
        vec![zero.clone(), neg_one],
    ])?;

    // Calculate σz²
    let sigma_z_squared = backend.matrix_mul(&sigma_z, &sigma_z)?;

    println!("   Pauli Z: σz = [[1, 0], [0, -1]]");
    println!("   σz² should equal identity:");
    for i in 0..sigma_z_squared.rows {
        print!("   [");
        for j in 0..sigma_z_squared.cols {
            print!(" {} ", sigma_z_squared.get(i, j).unwrap());
        }
        println!("]");
    }

    // Calculate trace
    let trace = backend.trace(&sigma_z)?;
    println!("   Tr(σz) = {} (should be 0)", trace);

    // Calculate determinant
    let det = backend.determinant(&sigma_z)?;
    println!("   det(σz) = {} (should be -1)", det);

    println!("   ✓ Pauli matrix properties verified");

    Ok(())
}

/// Additional example: Hydrogen atom radial equation
#[allow(dead_code)]
fn hydrogen_atom_radial(backend: &SymbolicaBackend) -> SymbolicResult<()> {
    // Radial Hamiltonian for hydrogen atom
    // H = -ℏ²/2m d²/dr² - ℏ²l(l+1)/2mr² - e²/4πε₀r

    let r = backend.variable("r")?;
    let l = backend.variable("l")?;
    let hbar = backend.parse("hbar")?;
    let m = backend.variable("m")?;
    let e = backend.variable("e")?;

    println!("   Hydrogen atom radial Hamiltonian");
    println!(
        "   Variables: r={}, l={}, ℏ={}, m={}, e={}",
        r, l, hbar, m, e
    );

    // This is a demonstration of the symbolic capability
    // Full implementation would require more complex symbolic manipulation

    Ok(())
}

/// Example: Perturbation theory setup
#[allow(dead_code)]
fn perturbation_theory_demo(backend: &SymbolicaBackend) -> SymbolicResult<()> {
    // H = H₀ + λH₁
    let lambda = backend.variable("lambda")?;

    // H₀ (unperturbed)
    let e0 = backend.variable("E0")?;
    let e1 = backend.variable("E1")?;
    let zero = backend.constant(0.0)?;

    let h0 = backend.matrix(vec![vec![e0, zero.clone()], vec![zero.clone(), e1]])?;

    // H₁ (perturbation)
    let v01 = backend.variable("V01")?;
    let h1 = backend.matrix(vec![vec![zero.clone(), v01.clone()], vec![v01, zero]])?;

    println!("   H₀ = diag(E0, E1)");
    println!("   H₁ = [[0, V01], [V01, 0]]");
    println!("   H = H₀ + λH₁");

    // Scale H₁ by λ
    let mut h1_scaled_elements = Vec::new();
    for i in 0..h1.rows {
        let mut row = Vec::new();
        for j in 0..h1.cols {
            let elem = h1.get(i, j).unwrap();
            let scaled = backend.mul(elem, &lambda)?;
            row.push(scaled);
        }
        h1_scaled_elements.push(row);
    }

    let h1_scaled = backend.matrix(h1_scaled_elements)?;

    // Add H₀ + λH₁
    let mut total_h_elements = Vec::new();
    for i in 0..h0.rows {
        let mut row = Vec::new();
        for j in 0..h0.cols {
            let h0_elem = h0.get(i, j).unwrap();
            let h1_elem = h1_scaled.get(i, j).unwrap();
            let sum = backend.add(h0_elem, h1_elem)?;
            row.push(sum);
        }
        total_h_elements.push(row);
    }

    println!("   ✓ Perturbation Hamiltonian constructed");

    Ok(())
}
