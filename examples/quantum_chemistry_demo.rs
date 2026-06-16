//! Quantum Chemistry Demonstration
//!
//! Author: gA4ss
//!
//! This example demonstrates quantum chemistry applications including
//! molecular Hamiltonians, electronic structure methods, and VQE calculations.

use myquat::qm_solver::quantum_chemistry::*;
use myquat::symbolic::{create_symbolica_backend, SymbolicBackend, SymbolicResult};

fn main() -> SymbolicResult<()> {
    println!("=== Quantum Chemistry Applications ===\n");

    // Create symbolic backend
    let backend = create_symbolica_backend();

    // 1. Molecular Hamiltonians
    println!("1. Molecular Hamiltonians");
    molecular_hamiltonians_demo(&backend)?;

    // 2. Electronic structure methods
    println!("\n2. Electronic Structure Methods");
    electronic_structure_demo(&backend)?;

    // 3. VQE calculations
    println!("\n3. Variational Quantum Eigensolver (VQE)");
    vqe_demo(&backend)?;

    // 4. Common molecules
    println!("\n4. Common Molecules");
    molecules_demo(&backend)?;

    // 5. Qubit mappings
    println!("\n5. Fermion-to-Qubit Mappings");
    qubit_mapping_demo(&backend)?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

/// Demonstrate molecular Hamiltonians
fn molecular_hamiltonians_demo(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    println!("   Born-Oppenheimer Approximation");
    println!("   Separate nuclear and electronic motion");

    // Electronic Hamiltonian components
    println!("\n   Electronic Hamiltonian:");
    println!("   H_el = T_e + V_ne + V_ee + V_nn");

    let hbar = backend.variable("hbar")?;
    let m_e = backend.variable("m_e")?;

    // Kinetic energy
    let t_e = hamiltonian::kinetic_energy(2, &hbar, &m_e, backend)?;
    println!("\n   Kinetic Energy:");
    println!("   T_e = -∑ᵢ (ℏ²/2mₑ)∇ᵢ²");
    println!("   Result: {}", t_e);

    // Nuclear-electron attraction
    let charges = vec![1.0, 1.0];
    let v_ne = hamiltonian::nuclear_electron_attraction(&charges, backend)?;
    println!("\n   Nuclear-Electron Attraction:");
    println!("   V_ne = -∑ᵢ∑ₐ Zₐe²/(4πε₀|rᵢ-Rₐ|)");
    println!("   Result: {}", v_ne);

    // Electron-electron repulsion
    let v_ee = hamiltonian::electron_electron_repulsion(2, backend)?;
    println!("\n   Electron-Electron Repulsion:");
    println!("   V_ee = ∑ᵢ<ⱼ e²/(4πε₀|rᵢ-rⱼ|)");
    println!("   Result: {}", v_ee);

    // Nuclear-nuclear repulsion
    let positions = vec![(0.0, 0.0, 0.0), (0.74, 0.0, 0.0)];
    let v_nn = hamiltonian::nuclear_nuclear_repulsion(&charges, &positions, backend)?;
    println!("\n   Nuclear-Nuclear Repulsion:");
    println!("   V_nn = ∑ₐ<ᵦ ZₐZᵦe²/(4πε₀|Rₐ-Rᵦ|)");
    println!("   Result: {}", v_nn);

    // Full Hamiltonian
    let h_el = hamiltonian::electronic_hamiltonian(&t_e, &v_ne, &v_ee, &v_nn, backend)?;
    println!("\n   Complete Electronic Hamiltonian:");
    println!("   H_el = {}", h_el);

    // Molecular integrals
    println!("\n   Molecular Integrals:");
    println!("   One-electron: h_pq = ⟨φₚ|T+V_ne|φᵧ⟩");
    println!("   Two-electron: h_pqrs = ⟨φₚφᵧ|V_ee|φᵣφₛ⟩");

    let h_1e = hamiltonian::one_electron_integrals(2, backend)?;
    println!(
        "   Generated {} one-electron integrals",
        h_1e.len() * h_1e[0].len()
    );

    println!("   ✅ Molecular Hamiltonians demonstrated");
    Ok(())
}

/// Demonstrate electronic structure methods
fn electronic_structure_demo(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    println!("   Ab Initio Methods");

    // Hartree-Fock
    println!("\n   1. Hartree-Fock (HF) Method");
    let hf = electronic_structure::HartreeFock::new(2, 2, backend)?;

    println!("   Self-Consistent Field (SCF):");
    println!("   F|φᵢ⟩ = εᵢ|φᵢ⟩");
    println!("   Fock operator: F = h + G");
    println!("   Number of electrons: {}", hf.n_electrons);
    println!("   Number of orbitals: {}", hf.n_orbitals);
    println!("   Total energy: {}", hf.total_energy);

    // Configuration Interaction
    println!("\n   2. Configuration Interaction (CI)");
    let ci = electronic_structure::ConfigurationInteraction::new(4, backend)?;

    println!("   Full CI expansion:");
    println!("   |Ψ⟩ = ∑ᵢ cᵢ|Φᵢ⟩");
    println!("   Number of configurations: {}", ci.ci_matrix.len());
    println!("   CI energy: {}", ci.ci_energy);

    // Coupled Cluster
    println!("\n   3. Coupled Cluster (CC) Theory");
    let cc = electronic_structure::CoupledCluster::new(backend)?;

    println!("   Exponential ansatz:");
    println!("   |Ψ⟩ = e^T|Φ₀⟩");
    println!("   Cluster operator: {}", cc.cluster_operator);

    let e_ccsd = cc.ccsd_energy(backend)?;
    println!("   CCSD energy: {}", e_ccsd);

    println!("\n   Method Hierarchy:");
    println!("   HF < MP2 < CCSD < CCSD(T) < Full CI");
    println!("   (Increasing accuracy and computational cost)");

    println!("   ✅ Electronic structure methods demonstrated");
    Ok(())
}

/// Demonstrate VQE calculations
fn vqe_demo(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    println!("   Quantum Algorithm for Molecular Energies");

    // VQE with UCCSD ansatz
    println!("\n   1. UCCSD Ansatz");
    let h = backend.variable("H_mol")?;
    let vqe_uccsd = vqe::VQECalculation::new(vqe::AnsatzType::UCCSD, 4, h.clone(), backend)?;

    println!("   Ansatz: {}", vqe_uccsd.ansatz);
    println!("   |ψ(θ)⟩ = e^(T-T†)|Φ₀⟩");
    println!("   Parameters: {}", vqe_uccsd.n_parameters);

    let uccsd_ansatz = vqe_uccsd.uccsd_ansatz(backend)?;
    println!("   Ansatz expression: {}", uccsd_ansatz);

    // Energy expectation
    let params = vec![0.1, 0.2, 0.3, 0.4];
    let energy = vqe_uccsd.energy_expectation(&params, backend)?;
    println!("\n   Energy Expectation:");
    println!("   E(θ) = ⟨ψ(θ)|H|ψ(θ)⟩");
    println!("   Result: {}", energy);

    // Gradient
    let gradients = vqe_uccsd.energy_gradient(&params, backend)?;
    println!("\n   Energy Gradient:");
    println!("   ∂E/∂θᵢ for optimization");
    println!("   Number of gradients: {}", gradients.len());

    // Hardware-efficient ansatz
    println!("\n   2. Hardware-Efficient Ansatz");
    let vqe_he = vqe::VQECalculation::new(vqe::AnsatzType::HardwareEfficient, 6, h, backend)?;

    println!("   Ansatz: {}", vqe_he.ansatz);
    println!("   Optimized for NISQ devices");
    println!("   Parameters: {}", vqe_he.n_parameters);

    println!("\n   VQE Workflow:");
    println!("   1. Prepare ansatz |ψ(θ)⟩");
    println!("   2. Measure ⟨H⟩");
    println!("   3. Classical optimization");
    println!("   4. Update parameters θ");
    println!("   5. Repeat until convergence");

    println!("   ✅ VQE calculations demonstrated");
    Ok(())
}

/// Demonstrate common molecules
fn molecules_demo(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    println!("   Molecular Systems");

    // Hydrogen molecule
    println!("\n   1. Hydrogen Molecule (H₂)");
    let h2 = molecules::h2(0.74, backend)?;

    println!("   Formula: H₂");
    println!("   Electrons: {}", h2.n_electrons);
    println!("   Orbitals: {}", h2.n_orbitals);
    println!("   Atoms: {}", h2.n_atoms());
    println!("   Nuclear charge: {}", h2.total_nuclear_charge());
    println!("   Bond length: 0.74 Å (equilibrium)");
    println!("   Hamiltonian: {}", h2.hamiltonian);

    // Lithium hydride
    println!("\n   2. Lithium Hydride (LiH)");
    let lih = molecules::lih(1.595, backend)?;

    println!("   Formula: LiH");
    println!("   Electrons: {}", lih.n_electrons);
    println!("   Orbitals: {}", lih.n_orbitals);
    println!("   Atoms: {}", lih.n_atoms());
    println!("   Nuclear charge: {}", lih.total_nuclear_charge());
    println!("   Bond length: 1.595 Å");

    // Water molecule
    println!("\n   3. Water Molecule (H₂O)");
    let h2o = molecules::h2o(backend)?;

    println!("   Formula: H₂O");
    println!("   Electrons: {}", h2o.n_electrons);
    println!("   Orbitals: {}", h2o.n_orbitals);
    println!("   Atoms: {}", h2o.n_atoms());
    println!("   Nuclear charge: {}", h2o.total_nuclear_charge());
    println!("   Geometry: Bent (104.45°)");

    println!("\n   Applications:");
    println!("   - Ground state energy calculation");
    println!("   - Excited states");
    println!("   - Reaction pathways");
    println!("   - Molecular properties");

    println!("   ✅ Common molecules demonstrated");
    Ok(())
}

/// Demonstrate qubit mappings
fn qubit_mapping_demo(backend: &impl SymbolicBackend) -> SymbolicResult<()> {
    println!("   Fermion-to-Qubit Transformations");
    println!("   Map fermionic operators to Pauli operators");

    let n_orbitals = 4;

    // Jordan-Wigner
    println!("\n   1. Jordan-Wigner Transformation");
    let jw = vqe::fermion_to_qubit_mapping(vqe::QubitMapping::JordanWigner, n_orbitals, backend)?;

    println!("   Mapping: {}", vqe::QubitMapping::JordanWigner);
    println!("   aᵢ† → (σ₁⁺...σᵢ₋₁⁺)σᵢ⁺");
    println!("   Qubits needed: {}", n_orbitals);
    println!("   Result: {}", jw);

    // Bravyi-Kitaev
    println!("\n   2. Bravyi-Kitaev Transformation");
    let bk = vqe::fermion_to_qubit_mapping(vqe::QubitMapping::BravyiKitaev, n_orbitals, backend)?;

    println!("   Mapping: {}", vqe::QubitMapping::BravyiKitaev);
    println!("   More efficient for some systems");
    println!("   Qubits needed: {}", n_orbitals);
    println!("   Result: {}", bk);

    // Parity
    println!("\n   3. Parity Transformation");
    let parity = vqe::fermion_to_qubit_mapping(vqe::QubitMapping::Parity, n_orbitals, backend)?;

    println!("   Mapping: {}", vqe::QubitMapping::Parity);
    println!("   Preserves particle number");
    println!("   Qubits needed: {}", n_orbitals);
    println!("   Result: {}", parity);

    println!("\n   Comparison:");
    println!("   - Jordan-Wigner: Simple, local operators");
    println!("   - Bravyi-Kitaev: Balanced locality");
    println!("   - Parity: Good for symmetry exploitation");

    println!("   ✅ Qubit mappings demonstrated");
    Ok(())
}
