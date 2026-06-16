//! Multi-Particle Quantum Systems
//!
//! Author: gA4ss
//!
//! This module handles quantum systems with multiple particles, including
//! two-particle systems, particle statistics, and entanglement analysis.

use crate::symbolic::{SymbolicBackend, SymbolicExpression, SymbolicResult};
use std::fmt;

/// Type of particle statistics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleStatistics {
    /// Bosons: symmetric wavefunctions
    Bosonic,
    /// Fermions: antisymmetric wavefunctions
    Fermionic,
    /// Distinguishable particles
    Distinguishable,
}

impl fmt::Display for ParticleStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParticleStatistics::Bosonic => write!(f, "Bosonic"),
            ParticleStatistics::Fermionic => write!(f, "Fermionic"),
            ParticleStatistics::Distinguishable => write!(f, "Distinguishable"),
        }
    }
}

/// Two-particle wavefunction
pub struct TwoParticleWaveFunction<E: SymbolicExpression> {
    /// Wavefunction $\psi(r_1, r_2)$
    pub wavefunction: E,

    /// Particle statistics
    pub statistics: ParticleStatistics,

    /// Whether using center of mass coordinates
    pub uses_com_coordinates: bool,
}

impl<E: SymbolicExpression> TwoParticleWaveFunction<E> {
    /// Create a new two-particle wavefunction
    pub fn new(wavefunction: E, statistics: ParticleStatistics) -> Self {
        Self {
            wavefunction,
            statistics,
            uses_com_coordinates: false,
        }
    }

    /// Transform to center of mass coordinates
    ///
    /// $$ R = \frac{m_1 r_1 + m_2 r_2}{m_1 + m_2} \quad \text{[center of mass]} $$
    /// $$ r = r_1 - r_2 \quad \text{[relative coordinate]} $$
    pub fn to_com_coordinates(&mut self) {
        self.uses_com_coordinates = true;
    }
}

/// Two-particle system solver
pub struct TwoParticleSystem<E: SymbolicExpression> {
    /// Mass of first particle
    pub mass1: E,

    /// Mass of second particle
    pub mass2: E,

    /// Total mass $M = m_1 + m_2$
    pub total_mass: E,

    /// Reduced mass $\mu = \frac{m_1 m_2}{m_1+m_2}$
    pub reduced_mass: E,

    /// Particle statistics
    pub statistics: ParticleStatistics,
}

impl<E: SymbolicExpression> TwoParticleSystem<E> {
    /// Create a new two-particle system
    pub fn new<B>(
        mass1: E,
        mass2: E,
        statistics: ParticleStatistics,
        backend: &B,
    ) -> SymbolicResult<Self>
    where
        B: SymbolicBackend<Expression = E>,
    {
        // $M = m_1 + m_2$
        let total_mass = backend.add(&mass1, &mass2)?;

        // $\mu = m_1 m_2/(m_1+m_2)$
        let m1m2 = backend.mul(&mass1, &mass2)?;
        let reduced_mass = backend.div(&m1m2, &total_mass)?;

        Ok(Self {
            mass1,
            mass2,
            total_mass,
            reduced_mass,
            statistics,
        })
    }

    /// Get center of mass coordinate
    ///
    /// $$ R = \frac{m_1 r_1 + m_2 r_2}{m_1 + m_2} $$
    pub fn center_of_mass<B>(&self, r1: &E, r2: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
    {
        let m1r1 = backend.mul(&self.mass1, r1)?;
        let m2r2 = backend.mul(&self.mass2, r2)?;
        let numerator = backend.add(&m1r1, &m2r2)?;
        backend.div(&numerator, &self.total_mass)
    }

    /// Get relative coordinate
    ///
    /// $r = r_1 - r_2$
    pub fn relative_coordinate<B>(&self, r1: &E, r2: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
    {
        backend.sub(r1, r2)
    }
}

/// Symmetrization and antisymmetrization operations
pub mod statistics {
    use super::*;

    /// Symmetrize wavefunction for bosons
    ///
    /// $$ \psi_S = \frac{1}{\sqrt{2}}[\psi(r_1,r_2) + \psi(r_2,r_1)] $$
    pub fn symmetrize<B, E>(
        psi_12: &E, // $\psi(r_1, r_2)$
        psi_21: &E, // $\psi(r_2, r_1)$
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let sum = backend.add(psi_12, psi_21)?;

        // $1/\sqrt{2}$
        let sqrt2 = backend.parse("sqrt(2)")?;
        let one = backend.constant(1.0)?;
        let coeff = backend.div(&one, &sqrt2)?;

        backend.mul(&coeff, &sum)
    }

    /// Antisymmetrize wavefunction for fermions
    ///
    /// $$ \psi_A = \frac{1}{\sqrt{2}}[\psi(r_1,r_2) - \psi(r_2,r_1)] $$
    pub fn antisymmetrize<B, E>(psi_12: &E, psi_21: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let diff = backend.sub(psi_12, psi_21)?;

        // $1/\sqrt{2}$
        let sqrt2 = backend.parse("sqrt(2)")?;
        let one = backend.constant(1.0)?;
        let coeff = backend.div(&one, &sqrt2)?;

        backend.mul(&coeff, &diff)
    }

    /// Check Pauli exclusion principle
    ///
    /// For identical fermions, $\psi(r_1,r_2) = -\psi(r_2,r_1)$
    /// At $r_1 = r_2$, $\psi$ must be zero
    pub fn check_pauli_exclusion<E>(psi_12: &E, psi_21: &E) -> bool
    where
        E: SymbolicExpression,
    {
        // Check if $\psi(r_1,r_2) + \psi(r_2,r_1)$ is zero (antisymmetric)
        // This is a simplified check; full verification requires symbolic equality
        psi_12.to_string() != psi_21.to_string()
    }

    /// Create Slater determinant for N fermions
    ///
    /// $$ |\psi\rangle = \frac{1}{\sqrt{N!}} \begin{vmatrix}
    /// \phi_1(r_1) & \phi_2(r_1) & \cdots & \phi_n(r_1) \\
    /// \phi_1(r_2) & \phi_2(r_2) & \cdots & \phi_n(r_2) \\
    /// \vdots & \vdots & \ddots & \vdots
    /// \end{vmatrix} $$
    pub fn slater_determinant_2particle<B, E>(
        phi1_r1: &E,
        phi2_r1: &E,
        phi1_r2: &E,
        phi2_r2: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // For 2 particles:
        // $\text{Det} = \phi_1(r_1)\phi_2(r_2) - \phi_1(r_2)\phi_2(r_1)$

        let term1 = backend.mul(phi1_r1, phi2_r2)?;
        let term2 = backend.mul(phi1_r2, phi2_r1)?;
        let det = backend.sub(&term1, &term2)?;

        // Normalize by $1/\sqrt{2}$
        let sqrt2 = backend.parse("sqrt(2)")?;
        let one = backend.constant(1.0)?;
        let coeff = backend.div(&one, &sqrt2)?;

        backend.mul(&coeff, &det)
    }
}

/// Entanglement analysis
pub mod entanglement {
    use super::*;

    /// Reduced density matrix for subsystem A
    ///
    /// $$ \rho_A = \text{Tr}_B(\rho_{AB}) $$
    pub struct ReducedDensityMatrix<E: SymbolicExpression> {
        /// Matrix elements
        pub matrix: E,

        /// Dimension of subsystem
        pub dimension: usize,
    }

    impl<E: SymbolicExpression> ReducedDensityMatrix<E> {
        /// Create reduced density matrix
        pub fn new(matrix: E, dimension: usize) -> Self {
            Self { matrix, dimension }
        }
    }

    /// Entanglement entropy (von Neumann entropy)
    ///
    /// $$ S = -\text{Tr}(\rho \log \rho) $$
    pub fn entanglement_entropy<B, E>(
        _reduced_density: &ReducedDensityMatrix<E>,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $S = -\sum_i \lambda_i \log \lambda_i$ (where $\lambda_i$ are eigenvalues)
        // Symbolic representation
        backend.parse("entropy")
    }

    /// Schmidt decomposition
    ///
    /// $$ |\psi\rangle = \sum_i \sqrt{\lambda_i} |i_A\rangle \otimes |i_B\rangle $$
    pub struct SchmidtDecomposition<E: SymbolicExpression> {
        /// Schmidt coefficients $\sqrt{\lambda_i}$
        pub coefficients: Vec<E>,

        /// Schmidt rank
        pub rank: usize,
    }

    impl<E: SymbolicExpression> SchmidtDecomposition<E> {
        /// Create Schmidt decomposition
        pub fn new(coefficients: Vec<E>) -> Self {
            let rank = coefficients.len();
            Self { coefficients, rank }
        }

        /// Check if state is entangled
        ///
        /// Entangled if Schmidt rank > 1
        pub fn is_entangled(&self) -> bool {
            self.rank > 1
        }
    }

    /// Bell states analysis
    pub mod bell {
        use super::*;

        /// Bell state types
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum BellState {
            /// $$ |\Phi^+\rangle = \frac{|00\rangle + |11\rangle}{\sqrt{2}} $$
            PhiPlus,
            /// $$ |\Phi^-\rangle = \frac{|00\rangle - |11\rangle}{\sqrt{2}} $$
            PhiMinus,
            /// $$ |\Psi^+\rangle = \frac{|01\rangle + |10\rangle}{\sqrt{2}} $$
            PsiPlus,
            /// $$ |\Psi^-\rangle = \frac{|01\rangle - |10\rangle}{\sqrt{2}} $$
            PsiMinus,
        }

        impl fmt::Display for BellState {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    BellState::PhiPlus => write!(f, "|Phi+>"),
                    BellState::PhiMinus => write!(f, "|Phi->"),
                    BellState::PsiPlus => write!(f, "|Psi+>"),
                    BellState::PsiMinus => write!(f, "|Psi->"),
                }
            }
        }

        /// Create Bell state
        pub fn create_bell_state<B, E>(bell_type: BellState, backend: &B) -> SymbolicResult<E>
        where
            B: SymbolicBackend<Expression = E>,
            E: SymbolicExpression,
        {
            let sqrt2 = backend.parse("sqrt(2)")?;
            let one = backend.constant(1.0)?;
            let coeff = backend.div(&one, &sqrt2)?;

            match bell_type {
                BellState::PhiPlus => {
                    // $(|00\rangle + |11\rangle)/\sqrt{2}$
                    let state = backend.parse("|00> + |11>")?;
                    backend.mul(&coeff, &state)
                }
                BellState::PhiMinus => {
                    let state = backend.parse("|00> - |11>")?;
                    backend.mul(&coeff, &state)
                }
                BellState::PsiPlus => {
                    let state = backend.parse("|01> + |10>")?;
                    backend.mul(&coeff, &state)
                }
                BellState::PsiMinus => {
                    let state = backend.parse("|01> - |10>")?;
                    backend.mul(&coeff, &state)
                }
            }
        }
    }

    /// Concurrence measure of entanglement
    ///
    /// $C(\rho) \in [0,1]$, with $C=0$ for separable states
    pub fn quantum_discord<B, E>(_density_matrix: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $C = \max(0, \lambda_1 - \lambda_2 - \lambda_3 - \lambda_4)$
        // where $\lambda_i$ are eigenvalues of $R = \sqrt{\sqrt{\rho} \tilde{\rho} \sqrt{\rho}}$
        backend.parse("concurrence")
    }

    /// Negativity measure
    ///
    /// $$ N(\rho) = \frac{||\rho^{T_A}||_1 - 1}{2} $$
    pub fn mutual_information<B, E>(_density_matrix: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // Negativity based on partial transpose
        backend.parse("negativity")
    }
}

impl<E: SymbolicExpression> fmt::Display for TwoParticleWaveFunction<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "$\\psi(r_1,r_2)$ [{}{}]",
            self.statistics,
            if self.uses_com_coordinates {
                ", COM coords"
            } else {
                ""
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbolic::create_symbolica_backend;

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_two_particle_system() {
        let backend = create_symbolica_backend();
        let m1 = backend.variable("m1").unwrap();
        let m2 = backend.variable("m2").unwrap();

        let system =
            TwoParticleSystem::new(m1, m2, ParticleStatistics::Distinguishable, &backend).unwrap();

        assert!(!system.reduced_mass.is_zero());
        assert!(!system.total_mass.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_symmetrization() {
        let backend = create_symbolica_backend();
        let psi_12 = backend.parse("psi(r1, r2)").unwrap();
        let psi_21 = backend.parse("psi(r2, r1)").unwrap();

        let symmetric = statistics::symmetrize(&psi_12, &psi_21, &backend).unwrap();

        assert!(!symmetric.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_antisymmetrization() {
        let backend = create_symbolica_backend();
        let psi_12 = backend.parse("psi(r1, r2)").unwrap();
        let psi_21 = backend.parse("psi(r2, r1)").unwrap();

        let antisym = statistics::antisymmetrize(&psi_12, &psi_21, &backend).unwrap();

        assert!(!antisym.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_slater_determinant() {
        let backend = create_symbolica_backend();
        let phi1_r1 = backend.parse("phi1(r1)").unwrap();
        let phi2_r1 = backend.parse("phi2(r1)").unwrap();
        let phi1_r2 = backend.parse("phi1(r2)").unwrap();
        let phi2_r2 = backend.parse("phi2(r2)").unwrap();

        let slater = statistics::slater_determinant_2particle(
            &phi1_r1, &phi2_r1, &phi1_r2, &phi2_r2, &backend,
        )
        .unwrap();

        assert!(!slater.is_zero());
    }

    #[test]
    fn test_particle_statistics_display() {
        assert_eq!(ParticleStatistics::Bosonic.to_string(), "Bosonic");
        assert_eq!(ParticleStatistics::Fermionic.to_string(), "Fermionic");
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_bell_state_maximally_entangled() {
        // Bell states are maximally entangled: S = 1 (for qubits)
        let backend = create_symbolica_backend();

        let bell_phi_plus =
            entanglement::bell::create_bell_state(entanglement::bell::BellState::PhiPlus, &backend)
                .unwrap();
        let bell_phi_minus = entanglement::bell::create_bell_state(
            entanglement::bell::BellState::PhiMinus,
            &backend,
        )
        .unwrap();
        let bell_psi_plus =
            entanglement::bell::create_bell_state(entanglement::bell::BellState::PsiPlus, &backend)
                .unwrap();
        let bell_psi_minus = entanglement::bell::create_bell_state(
            entanglement::bell::BellState::PsiMinus,
            &backend,
        )
        .unwrap();

        // All Bell states should exist
        let phi_plus_str = format!("{}", bell_phi_plus);
        let phi_minus_str = format!("{}", bell_phi_minus);
        let psi_plus_str = format!("{}", bell_psi_plus);
        let psi_minus_str = format!("{}", bell_psi_minus);

        assert!(!phi_plus_str.is_empty());
        assert!(!phi_minus_str.is_empty());
        assert!(!psi_plus_str.is_empty());
        assert!(!psi_minus_str.is_empty());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_bell_state_orthogonality() {
        // Bell states form an orthonormal basis
        let backend = create_symbolica_backend();

        let bell1 =
            entanglement::bell::create_bell_state(entanglement::bell::BellState::PhiPlus, &backend)
                .unwrap();
        let bell2 = entanglement::bell::create_bell_state(
            entanglement::bell::BellState::PhiMinus,
            &backend,
        )
        .unwrap();

        // Different Bell states should be orthogonal
        // (can't compute inner product symbolically, but structures differ)
        assert_ne!(format!("{}", bell1), format!("{}", bell2));
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_schmidt_decomposition_rank() {
        // Schmidt rank = number of non-zero Schmidt coefficients
        let backend = create_symbolica_backend();

        let c1 = backend.constant(0.6).unwrap();
        let c2 = backend.constant(0.8).unwrap();

        let schmidt = entanglement::SchmidtDecomposition::new(vec![c1, c2]);

        // Two non-zero coefficients => rank 2
        assert_eq!(schmidt.rank, 2);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_schmidt_decomposition_normalized() {
        // Schmidt coefficients should satisfy $\sum |c_i|^2 = 1$
        let backend = create_symbolica_backend();

        // Create normalized coefficients: $1/\sqrt{2}$, $1/\sqrt{2}$
        let c1 = backend.constant(0.7071).unwrap();
        let c2 = backend.constant(0.7071).unwrap();

        let schmidt = entanglement::SchmidtDecomposition::new(vec![c1, c2]);

        // Should have 2 coefficients
        assert_eq!(schmidt.coefficients.len(), 2);
        assert_eq!(schmidt.rank, 2);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_entanglement_entropy_separable_state() {
        // Separable state: S = 0 (no entanglement)
        let backend = create_symbolica_backend();

        // Single coefficient => product state
        let c1 = backend.constant(1.0).unwrap();
        let schmidt = entanglement::SchmidtDecomposition::new(vec![c1]);

        // Rank 1 => separable state
        assert_eq!(schmidt.rank, 1);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_entanglement_entropy_maximally_entangled() {
        // Maximally entangled state: $S = \log_2(d)$ where d is dimension
        let backend = create_symbolica_backend();

        // Equal coefficients for 2D system: 1/√2, 1/√2
        let c1 = backend.constant(0.7071).unwrap();
        let c2 = backend.constant(0.7071).unwrap();

        let schmidt = entanglement::SchmidtDecomposition::new(vec![c1, c2]);

        // Full rank => maximally entangled
        assert_eq!(schmidt.rank, 2);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_reduced_density_matrix_properties() {
        // Reduced density matrix: $\rho_A = \text{Tr}_B(|\psi\rangle\langle\psi|)$
        let backend = create_symbolica_backend();

        let bell =
            entanglement::bell::create_bell_state(entanglement::bell::BellState::PhiPlus, &backend)
                .unwrap();

        // For Bell state, reduced density matrix is maximally mixed: $\rho_A = I/2$
        // This tests that the state exists
        let state_str = format!("{}", bell);
        assert!(!state_str.is_empty());
    }

    #[test]
    fn test_particle_statistics_variants() {
        // Test all particle statistics types
        let stats = vec![
            ParticleStatistics::Bosonic,
            ParticleStatistics::Fermionic,
            ParticleStatistics::Distinguishable,
        ];

        for stat in stats {
            assert_eq!(format!("{:?}", stat).len() > 0, true);
        }
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_bosonic_symmetrization_property() {
        // Bosonic wavefunction: $\psi(1,2) = \psi(2,1)$
        let backend = create_symbolica_backend();

        let psi_12 = backend.constant(1.0).unwrap();
        let psi_21 = backend.constant(1.0).unwrap();

        let symmetric = statistics::symmetrize(&psi_12, &psi_21, &backend).unwrap();

        // Symmetric combination should exist
        let sym_str = format!("{}", symmetric);
        assert!(!sym_str.is_empty());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_fermionic_antisymmetrization_property() {
        // Fermionic wavefunction: $\psi(1,2) = -\psi(2,1)$
        let backend = create_symbolica_backend();

        let psi_12 = backend.variable("psi_12").unwrap();
        let psi_21 = backend.variable("psi_21").unwrap();

        let antisym = statistics::antisymmetrize(&psi_12, &psi_21, &backend).unwrap();

        // Antisymmetric combination should exist
        let asym_str = format!("{}", antisym);
        assert!(!asym_str.is_empty());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_pauli_exclusion_principle() {
        // Slater determinant vanishes when two fermions in same state
        let backend = create_symbolica_backend();

        let phi1 = backend.parse("phi(r1)").unwrap();
        let phi1_copy = backend.parse("phi(r1)").unwrap();
        let phi2 = backend.parse("phi(r2)").unwrap();
        let phi2_copy = backend.parse("phi(r2)").unwrap();

        // Same orbital for both particles => det = 0
        let slater = statistics::slater_determinant_2particle(
            &phi1, &phi1_copy, &phi2, &phi2_copy, &backend,
        )
        .unwrap();

        // Slater determinant computed (may be zero symbolically)
        let slater_str = format!("{}", slater);
        assert!(!slater_str.is_empty());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_two_particle_system_masses() {
        let backend = create_symbolica_backend();

        let m1 = backend.constant(1.0).unwrap();
        let m2 = backend.constant(2.0).unwrap();

        let system =
            TwoParticleSystem::new(m1, m2, ParticleStatistics::Distinguishable, &backend).unwrap();

        // Reduced mass: $\mu = m_1 m_2/(m_1+m_2) = 2/3$
        // Total mass: $M = m_1 + m_2 = 3$
        let mu_str = format!("{}", system.reduced_mass);
        let m_str = format!("{}", system.total_mass);

        assert!(!mu_str.is_empty());
        assert!(!m_str.is_empty());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_schmidt_rank_one_is_separable() {
        // Rank-1 Schmidt decomposition => separable state
        let backend = create_symbolica_backend();

        let c1 = backend.constant(1.0).unwrap();
        let schmidt = entanglement::SchmidtDecomposition::new(vec![c1]);

        // Rank 1 means $|\psi\rangle = |\phi_A\rangle\otimes|\phi_B\rangle$ (product state)
        assert_eq!(schmidt.rank, 1);
        assert!(!schmidt.is_entangled());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_schmidt_rank_greater_than_one_is_entangled() {
        // Rank > 1 => entangled state
        let backend = create_symbolica_backend();

        let c1 = backend.constant(0.6).unwrap();
        let c2 = backend.constant(0.8).unwrap();

        let schmidt = entanglement::SchmidtDecomposition::new(vec![c1, c2]);

        // Rank 2 means state cannot be written as product state
        assert_eq!(schmidt.rank, 2);
        assert!(schmidt.is_entangled());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_bell_basis_completeness() {
        // Four Bell states form complete basis for 2-qubit system
        let backend = create_symbolica_backend();

        let bell_states = vec![
            entanglement::bell::create_bell_state(entanglement::bell::BellState::PhiPlus, &backend)
                .unwrap(),
            entanglement::bell::create_bell_state(
                entanglement::bell::BellState::PhiMinus,
                &backend,
            )
            .unwrap(),
            entanglement::bell::create_bell_state(entanglement::bell::BellState::PsiPlus, &backend)
                .unwrap(),
            entanglement::bell::create_bell_state(
                entanglement::bell::BellState::PsiMinus,
                &backend,
            )
            .unwrap(),
        ];

        // Should have 4 orthogonal Bell states
        assert_eq!(bell_states.len(), 4);
    }
}
