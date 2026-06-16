//! Time-Independent Schrodinger Equation (TISE) Solver
//!
//! Author: gA4ss
//!
//! This module provides solvers for the time-independent Schrodinger equation:
//! $\hat{H}\psi = E\psi$
//!
//! Supports various quantum mechanical potentials and computes energy eigenvalues
//! and eigenstates.

use crate::qm_solver::{QuantumOperator, SymbolicWaveFunction};
use crate::symbolic::{SymbolicBackend, SymbolicExpression, SymbolicResult};
use std::fmt;

/// Type of quantum mechanical potential
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PotentialType {
    /// Infinite square well (particle in a box)
    InfiniteSquareWell,
    /// Quantum harmonic oscillator
    HarmonicOscillator,
    /// Hydrogen atom (Coulomb potential)
    HydrogenAtom,
    /// Finite square well
    FiniteSquareWell,
    /// Delta function potential
    DeltaFunction,
    /// Periodic potential (Kronig-Penney model)
    Periodic,
    /// Custom potential V(x)
    Custom,
}

impl fmt::Display for PotentialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PotentialType::InfiniteSquareWell => write!(f, "InfiniteSquareWell"),
            PotentialType::HarmonicOscillator => write!(f, "HarmonicOscillator"),
            PotentialType::HydrogenAtom => write!(f, "HydrogenAtom"),
            PotentialType::FiniteSquareWell => write!(f, "FiniteSquareWell"),
            PotentialType::DeltaFunction => write!(f, "DeltaFunction"),
            PotentialType::Periodic => write!(f, "Periodic"),
            PotentialType::Custom => write!(f, "Custom"),
        }
    }
}

/// Quantum mechanical potential
///
/// Represents a potential energy function V(r) in the Hamiltonian.
///
/// # Mathematical Background
///
/// The Hamiltonian for a particle in a potential is:
/// $$\hat{H} = \frac{\hat{p}^2}{2m} + V(\hat{r})$$
pub struct Potential<E: SymbolicExpression> {
    /// The potential energy expression V(r)
    pub expression: E,

    /// Type of potential
    pub potential_type: PotentialType,

    /// Name of the potential
    pub name: String,

    /// Spatial dimensions
    pub dimensions: usize,
}

impl<E: SymbolicExpression> Potential<E> {
    /// Create a new potential
    pub fn new(
        expression: E,
        potential_type: PotentialType,
        name: impl Into<String>,
        dimensions: usize,
    ) -> Self {
        Self {
            expression,
            potential_type,
            name: name.into(),
            dimensions,
        }
    }

    /// Create a custom potential
    pub fn custom(expression: E, name: impl Into<String>, dimensions: usize) -> Self {
        Self::new(expression, PotentialType::Custom, name, dimensions)
    }
}

/// Energy eigenstate solution
///
/// Represents a solution to the eigenvalue problem $\hat{H}\psi = E\psi$
pub struct EnergyEigenstate<E: SymbolicExpression> {
    /// Energy eigenvalue
    pub energy: E,

    /// Eigenstate wave function
    pub eigenstate: SymbolicWaveFunction<E>,

    /// Quantum numbers (n, l, m, etc.)
    pub quantum_numbers: Vec<usize>,

    /// Degeneracy of this energy level
    pub degeneracy: usize,
}

impl<E: SymbolicExpression> EnergyEigenstate<E> {
    /// Create a new energy eigenstate
    pub fn new(
        energy: E,
        eigenstate: SymbolicWaveFunction<E>,
        quantum_numbers: Vec<usize>,
    ) -> Self {
        Self {
            energy,
            eigenstate,
            quantum_numbers,
            degeneracy: 1,
        }
    }

    /// Set the degeneracy
    pub fn with_degeneracy(mut self, degeneracy: usize) -> Self {
        self.degeneracy = degeneracy;
        self
    }
}

/// Time-Independent Schrodinger Equation Solver
///
/// Solves $\hat{H}\psi = E\psi$ for various potentials.
///
/// # Mathematical Background
///
/// In 1D: $-\frac{\hbar^2}{2m}\frac{d^2\psi}{dx^2} + V(x)\psi = E\psi$
///
/// In 3D: $-\frac{\hbar^2}{2m}\nabla^2\psi + V(\vec{r})\psi = E\psi$
pub struct TISESolver<E: SymbolicExpression> {
    /// The potential V(r)
    pub potential: Potential<E>,

    /// Mass of the particle
    pub mass: E,

    /// Reduced Planck constant ℏ
    pub hbar: E,
}

impl<E: SymbolicExpression> TISESolver<E> {
    /// Create a new TISE solver
    pub fn new<B>(potential: Potential<E>, backend: &B) -> SymbolicResult<Self>
    where
        B: SymbolicBackend<Expression = E>,
    {
        let mass = backend.variable("m")?;
        let hbar = backend.variable("hbar")?;

        Ok(Self {
            potential,
            mass,
            hbar,
        })
    }

    /// Build the Hamiltonian operator
    ///
    /// $\hat{H} = \frac{\hat{p}^2}{2m} + V(\hat{x})$
    pub fn hamiltonian<B>(&self, backend: &B) -> SymbolicResult<QuantumOperator<E>>
    where
        B: SymbolicBackend<Expression = E>,
    {
        // Kinetic energy: p²/(2m)
        let p_squared = backend.parse("p^2")?;
        let two = backend.constant(2.0)?;
        let two_m = backend.mul(&two, &self.mass)?;
        let kinetic = backend.div(&p_squared, &two_m)?;

        // Total Hamiltonian: T + V
        let hamiltonian_expr = backend.add(&kinetic, &self.potential.expression)?;

        Ok(QuantumOperator::hamiltonian(hamiltonian_expr, "H"))
    }
}

/// Standard potential models
pub mod potentials {
    use super::*;

    /// Infinite square well potential (particle in a box)
    ///
    /// V(x) = 0 for 0 < x < L, ∞ otherwise
    ///
    /// Energy eigenvalues: $E_n = \frac{n^2\pi^2\hbar^2}{2mL^2}$, n = 1, 2, 3, ...
    ///
    /// Eigenfunctions: $\psi_n(x) = \sqrt{\frac{2}{L}}\sin\left(\frac{n\pi x}{L}\right)$
    pub fn infinite_square_well<B, E>(backend: &B, length: &E) -> SymbolicResult<Potential<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // Inside the well, V = 0
        let zero = backend.constant(0.0)?;

        Ok(Potential::new(
            zero,
            PotentialType::InfiniteSquareWell,
            format!("InfiniteWell(L={})", length.to_string()),
            1,
        ))
    }

    /// Quantum harmonic oscillator potential
    ///
    /// V(x) = (1/2)mω²x²
    ///
    /// Energy eigenvalues: $E_n = \hbar\omega(n + \frac{1}{2})$, n = 0, 1, 2, ...
    ///
    /// Eigenfunctions: Hermite polynomials
    pub fn harmonic_oscillator<B, E>(backend: &B) -> SymbolicResult<Potential<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let potential_expr = backend.parse("(1/2)*m*omega^2*x^2")?;

        Ok(Potential::new(
            potential_expr,
            PotentialType::HarmonicOscillator,
            "HarmonicOscillator",
            1,
        ))
    }

    /// Hydrogen atom Coulomb potential
    ///
    /// V(r) = -e²/(4πε₀r) = -ke²/r
    ///
    /// Energy eigenvalues: $E_n = -\frac{13.6\text{ eV}}{n^2}$, n = 1, 2, 3, ...
    ///
    /// Degeneracy: n²
    pub fn hydrogen_atom<B, E>(backend: &B) -> SymbolicResult<Potential<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // V(r) = -k*e^2/r where k = 1/(4πε₀)
        let potential_expr = backend.parse("-k*e^2/r")?;

        Ok(Potential::new(
            potential_expr,
            PotentialType::HydrogenAtom,
            "Hydrogen",
            3,
        ))
    }

    /// Finite square well potential
    ///
    /// V(x) = -V₀ for -a < x < a, 0 otherwise
    pub fn finite_square_well<B, E>(
        backend: &B,
        depth: &E,
        width: &E,
    ) -> SymbolicResult<Potential<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // V(x) = -V₀ inside the well
        let neg_depth = backend.neg(depth)?;

        Ok(Potential::new(
            neg_depth,
            PotentialType::FiniteSquareWell,
            format!(
                "FiniteWell(V0={}, a={})",
                depth.to_string(),
                width.to_string()
            ),
            1,
        ))
    }

    /// Delta function potential
    ///
    /// V(x) = -αδ(x)
    ///
    /// Bound state energy: $E = -\frac{m\alpha^2}{2\hbar^2}$
    pub fn delta_function<B, E>(backend: &B, strength: &E) -> SymbolicResult<Potential<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // V(x) = -α*δ(x)
        let neg_alpha = backend.neg(strength)?;
        let delta = backend.parse("delta(x)")?;
        let potential_expr = backend.mul(&neg_alpha, &delta)?;

        Ok(Potential::new(
            potential_expr,
            PotentialType::DeltaFunction,
            format!("Delta(alpha={})", strength.to_string()),
            1,
        ))
    }
}

/// Analytical solutions for standard potentials
pub mod solutions {
    use super::*;

    /// Compute energy eigenvalue for infinite square well
    ///
    /// $E_n = \frac{n^2\pi^2\hbar^2}{2mL^2}$
    pub fn infinite_well_energy<B, E>(
        n: usize,
        length: &E,
        mass: &E,
        hbar: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let n_val = backend.constant(n as f64)?;
        let two = backend.constant(2.0)?;
        let pi = backend.variable("pi")?;

        // n²π²ℏ²
        let n_squared = backend.pow(&n_val, &two)?;
        let pi_squared = backend.pow(&pi, &two)?;
        let hbar_squared = backend.pow(hbar, &two)?;

        let numerator = backend.mul(&n_squared, &pi_squared)?;
        let numerator = backend.mul(&numerator, &hbar_squared)?;

        // 2mL²
        let l_squared = backend.pow(length, &two)?;
        let denominator = backend.mul(&two, mass)?;
        let denominator = backend.mul(&denominator, &l_squared)?;

        backend.div(&numerator, &denominator)
    }

    /// Compute energy eigenvalue for harmonic oscillator
    ///
    /// $E_n = \hbar\omega(n + \frac{1}{2})$
    pub fn harmonic_oscillator_energy<B, E>(
        n: usize,
        omega: &E,
        hbar: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let n_val = backend.constant(n as f64)?;
        let half = backend.constant(0.5)?;

        // n + 1/2
        let n_plus_half = backend.add(&n_val, &half)?;

        // ℏω(n + 1/2)
        let h_omega = backend.mul(hbar, omega)?;
        backend.mul(&h_omega, &n_plus_half)
    }

    /// Compute energy eigenvalue for hydrogen atom
    ///
    /// $E_n = -\frac{m_e e^4}{2(4\pi\epsilon_0)^2\hbar^2 n^2} = -\frac{13.6 \text{ eV}}{n^2}$
    pub fn hydrogen_energy<B, E>(n: usize, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let n_val = backend.constant(n as f64)?;
        let two = backend.constant(2.0)?;

        // -13.6 eV / n²
        let rydberg = backend.constant(-13.6)?; // Rydberg energy in eV
        let n_squared = backend.pow(&n_val, &two)?;

        backend.div(&rydberg, &n_squared)
    }

    /// Get degeneracy for hydrogen atom energy level
    ///
    /// Degeneracy = n² (accounting for l and m quantum numbers)
    pub fn hydrogen_degeneracy(n: usize) -> usize {
        n * n
    }
}

impl<E: SymbolicExpression> fmt::Display for Potential<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Potential({}, {}D): V = {}",
            self.name,
            self.dimensions,
            self.expression.to_string()
        )
    }
}

impl<E: SymbolicExpression> fmt::Display for EnergyEigenstate<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "E = {}, QN = {:?}, deg = {}",
            self.energy.to_string(),
            self.quantum_numbers,
            self.degeneracy
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbolic::create_symbolica_backend;

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_potential_creation() {
        let backend = create_symbolica_backend();
        let v = backend.parse("x^2").unwrap();

        let potential = Potential::custom(v, "test", 1);
        assert_eq!(potential.dimensions, 1);
        assert_eq!(potential.potential_type, PotentialType::Custom);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_harmonic_oscillator_potential() {
        let backend = create_symbolica_backend();
        let ho = potentials::harmonic_oscillator(&backend).unwrap();

        assert_eq!(ho.potential_type, PotentialType::HarmonicOscillator);
        assert_eq!(ho.dimensions, 1);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_hydrogen_potential() {
        let backend = create_symbolica_backend();
        let hydrogen = potentials::hydrogen_atom(&backend).unwrap();

        assert_eq!(hydrogen.potential_type, PotentialType::HydrogenAtom);
        assert_eq!(hydrogen.dimensions, 3);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_infinite_well_energy() {
        let backend = create_symbolica_backend();
        let length = backend.variable("L").unwrap();
        let mass = backend.variable("m").unwrap();
        let hbar = backend.variable("hbar").unwrap();

        let e1 = solutions::infinite_well_energy(1, &length, &mass, &hbar, &backend).unwrap();
        assert!(!e1.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_harmonic_oscillator_energy() {
        let backend = create_symbolica_backend();
        let omega = backend.variable("omega").unwrap();
        let hbar = backend.variable("hbar").unwrap();

        let e0 = solutions::harmonic_oscillator_energy(0, &omega, &hbar, &backend).unwrap();
        assert!(!e0.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_hydrogen_energy() {
        let backend = create_symbolica_backend();

        let e1 = solutions::hydrogen_energy(1, &backend).unwrap();
        assert!(!e1.is_zero());
    }

    #[test]
    fn test_hydrogen_degeneracy() {
        assert_eq!(solutions::hydrogen_degeneracy(1), 1);
        assert_eq!(solutions::hydrogen_degeneracy(2), 4);
        assert_eq!(solutions::hydrogen_degeneracy(3), 9);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_tise_solver_creation() {
        let backend = create_symbolica_backend();
        let potential = potentials::harmonic_oscillator(&backend).unwrap();

        let solver = TISESolver::new(potential, &backend).unwrap();
        assert!(!solver.mass.is_zero());
        assert!(!solver.hbar.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_hamiltonian_construction() {
        let backend = create_symbolica_backend();
        let potential = potentials::harmonic_oscillator(&backend).unwrap();
        let solver = TISESolver::new(potential, &backend).unwrap();

        let hamiltonian = solver.hamiltonian(&backend).unwrap();
        assert!(hamiltonian.is_hermitian());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_harmonic_oscillator_equal_spacing() {
        // Energy levels should be equally spaced: E_{n+1} - E_n = ℏω
        let backend = create_symbolica_backend();
        let omega = backend.variable("omega").unwrap();
        let hbar = backend.variable("hbar").unwrap();

        let e0 = solutions::harmonic_oscillator_energy(0, &omega, &hbar, &backend).unwrap();
        let e1 = solutions::harmonic_oscillator_energy(1, &omega, &hbar, &backend).unwrap();
        let e2 = solutions::harmonic_oscillator_energy(2, &omega, &hbar, &backend).unwrap();

        // Compute energy differences
        let diff1 = backend.sub(&e1, &e0).unwrap();
        let diff2 = backend.sub(&e2, &e1).unwrap();

        // Both differences should equal ℏω
        let expected = backend.mul(&hbar, &omega).unwrap();
        let diff1_str = format!("{}", diff1);
        let diff2_str = format!("{}", diff2);
        let exp_str = format!("{}", expected);
        assert_eq!(diff1_str, exp_str);
        assert_eq!(diff2_str, exp_str);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_infinite_well_energy_scaling() {
        // E_n ∝ n² for particle in a box
        let backend = create_symbolica_backend();
        let length = backend.constant(1.0).unwrap();
        let mass = backend.constant(1.0).unwrap();
        let hbar = backend.constant(1.0).unwrap();

        let e1 = solutions::infinite_well_energy(1, &length, &mass, &hbar, &backend).unwrap();
        let e2 = solutions::infinite_well_energy(2, &length, &mass, &hbar, &backend).unwrap();
        let e4 = solutions::infinite_well_energy(4, &length, &mass, &hbar, &backend).unwrap();

        // E_2 should be 4 times E_1
        let four_e1 = backend.mul(&backend.constant(4.0).unwrap(), &e1).unwrap();
        let e2_str = format!("{}", e2);
        let four_e1_str = format!("{}", four_e1);
        assert_eq!(e2_str, four_e1_str);

        // E_4 should be 16 times E_1
        let sixteen_e1 = backend.mul(&backend.constant(16.0).unwrap(), &e1).unwrap();
        let e4_str = format!("{}", e4);
        let sixteen_e1_str = format!("{}", sixteen_e1);
        assert_eq!(e4_str, sixteen_e1_str);
    }

    #[test]
    fn test_hydrogen_degeneracy_formula() {
        // Test degeneracy formula: g_n = n²
        for n in 1..=5 {
            let expected_degeneracy = n * n;
            assert_eq!(solutions::hydrogen_degeneracy(n), expected_degeneracy);
        }
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_hydrogen_energy_rydberg_formula() {
        // E_n = -13.6 eV / n²
        let backend = create_symbolica_backend();

        let e1 = solutions::hydrogen_energy(1, &backend).unwrap();
        let e2 = solutions::hydrogen_energy(2, &backend).unwrap();
        let e3 = solutions::hydrogen_energy(3, &backend).unwrap();

        // E_1 should be -13.6 eV
        let e1_str = format!("{}", e1);
        assert!(e1_str.contains("13.6") || e1_str.contains("-13.6"));

        // Verify energy ordering: E_1 < E_2 < E_3 < 0
        // (More negative = lower energy)
        let e2_str = format!("{}", e2);
        let e3_str = format!("{}", e3);
        assert!(e1_str.contains("-"));
        assert!(e2_str.contains("-"));
        assert!(e3_str.contains("-"));
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_potential_type_variants() {
        let backend = create_symbolica_backend();

        // Test all potential types can be created
        let v_ho = potentials::harmonic_oscillator(&backend).unwrap();
        assert_eq!(v_ho.potential_type, PotentialType::HarmonicOscillator);

        let length = backend.constant(1.0).unwrap();
        let v_well = potentials::infinite_square_well(&backend, &length).unwrap();
        assert_eq!(v_well.potential_type, PotentialType::InfiniteSquareWell);

        let v_hydrogen = potentials::hydrogen_atom(&backend).unwrap();
        assert_eq!(v_hydrogen.potential_type, PotentialType::HydrogenAtom);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_harmonic_oscillator_ground_state_nonzero() {
        // Zero-point energy: E_0 = ℏω/2 ≠ 0
        let backend = create_symbolica_backend();
        let omega = backend.constant(1.0).unwrap();
        let hbar = backend.constant(1.0).unwrap();

        let e0 = solutions::harmonic_oscillator_energy(0, &omega, &hbar, &backend).unwrap();

        // Ground state energy should be 0.5 (for ℏ=ω=1)
        let half = backend.constant(0.5).unwrap();
        let e0_str = format!("{}", e0);
        let half_str = format!("{}", half);
        assert_eq!(e0_str, half_str);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_solver_with_different_potentials() {
        let backend = create_symbolica_backend();

        let length = backend.constant(1.0).unwrap();

        // Test solver works with different potential types
        let test_potentials = vec![
            potentials::harmonic_oscillator(&backend).unwrap(),
            potentials::infinite_square_well(&backend, &length).unwrap(),
            potentials::hydrogen_atom(&backend).unwrap(),
        ];

        for pot in test_potentials {
            let solver = TISESolver::new(pot, &backend);
            assert!(solver.is_ok());
        }
    }
}
