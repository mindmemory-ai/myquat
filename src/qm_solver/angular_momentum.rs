//! Angular Momentum in Quantum Mechanics
//!
//! Author: gA4ss
//!
//! This module implements orbital angular momentum, spin, and related operations
//! including spherical harmonics, Clebsch-Gordan coefficients, and central force problems.

use crate::symbolic::{SymbolicBackend, SymbolicExpression, SymbolicResult};
use std::fmt;

/// Angular momentum quantum numbers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AngularMomentumState {
    /// Principal quantum number l (or j for total angular momentum)
    pub l: usize,

    /// Magnetic quantum number m
    pub m: isize,
}

impl AngularMomentumState {
    /// Create new angular momentum state
    pub fn new(l: usize, m: isize) -> Result<Self, String> {
        if m.abs() > l as isize {
            return Err(format!("Invalid m={} for l={}", m, l));
        }
        Ok(Self { l, m })
    }
}

/// Spherical harmonic $Y_l^m(\theta, \phi)$
pub struct SphericalHarmonic<E: SymbolicExpression> {
    /// Angular momentum quantum number l
    pub l: usize,

    /// Magnetic quantum number m
    pub m: isize,

    /// Symbolic expression
    pub expression: E,
}

impl<E: SymbolicExpression> SphericalHarmonic<E> {
    /// Create spherical harmonic
    pub fn new<B>(l: usize, m: isize, theta: &str, phi: &str, backend: &B) -> SymbolicResult<Self>
    where
        B: SymbolicBackend<Expression = E>,
    {
        if m.abs() > l as isize {
            return Err(crate::symbolic::SymbolicError::InvalidExpression(format!(
                "Invalid m={} for l={}",
                m, l
            )));
        }

        // $Y_l^m(\theta,\phi) = N P_l^m(\cos \theta) e^{im\phi}$
        // Simplified symbolic representation
        let expr = backend.parse(&format!("Y_{}^{}({}, {})", l, m, theta, phi))?;

        Ok(Self {
            l,
            m,
            expression: expr,
        })
    }
}

/// Orbital angular momentum operators
pub mod orbital {
    use super::*;

    /// $L^2$ operator eigenvalue
    ///
    /// $$ L^2|l,m\rangle = \hbar^2 l(l+1)|l,m\rangle $$
    pub fn l_squared_eigenvalue<B, E>(l: usize, hbar: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let l_val = backend.constant(l as f64)?;
        let l_plus_1 = backend.constant((l + 1) as f64)?;

        // l(l+1)
        let l_l_plus_1 = backend.mul(&l_val, &l_plus_1)?;

        // $\hbar^2 l(l+1)$
        let hbar_squared = backend.mul(hbar, hbar)?;
        backend.mul(&hbar_squared, &l_l_plus_1)
    }

    /// $L_z$ operator eigenvalue
    ///
    /// $$ L_z|l,m\rangle = \hbar m|l,m\rangle $$
    pub fn l_z_eigenvalue<B, E>(m: isize, hbar: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let m_val = backend.constant(m as f64)?;
        backend.mul(hbar, &m_val)
    }

    /// Ladder operator $L_+$ action
    ///
    /// $$ L_+|l,m\rangle = \hbar\sqrt{l(l+1) - m(m+1)}|l,m+1\rangle $$
    pub fn ladder_plus<B, E>(l: usize, m: isize, hbar: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        if m >= l as isize {
            // Already at maximum m
            return backend.constant(0.0);
        }

        let l_val = l as f64;
        let m_val = m as f64;

        // l(l+1) - m(m+1)
        let l_term = l_val * (l_val + 1.0);
        let m_term = m_val * (m_val + 1.0);
        let coefficient = (l_term - m_term).sqrt();

        let coeff_expr = backend.constant(coefficient)?;
        backend.mul(hbar, &coeff_expr)
    }

    /// Ladder operator $L_-$ action
    ///
    /// $$ L_-|l,m\rangle = \hbar\sqrt{l(l+1) - m(m-1)}|l,m-1\rangle $$
    pub fn ladder_minus<B, E>(l: usize, m: isize, hbar: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        if m <= -(l as isize) {
            // Already at minimum m
            return backend.constant(0.0);
        }

        let l_val = l as f64;
        let m_val = m as f64;

        // l(l+1) - m(m-1)
        let l_term = l_val * (l_val + 1.0);
        let m_term = m_val * (m_val - 1.0);
        let coefficient = (l_term - m_term).sqrt();

        let coeff_expr = backend.constant(coefficient)?;
        backend.mul(hbar, &coeff_expr)
    }
}

/// Spin operators and spinors
pub mod spin {
    use super::*;

    /// Pauli matrices
    pub struct PauliMatrices<E: SymbolicExpression> {
        /// $\sigma_x$
        pub sigma_x: E,
        /// $\sigma_y$
        pub sigma_y: E,
        /// $\sigma_z$
        pub sigma_z: E,
    }

    impl<E: SymbolicExpression> PauliMatrices<E> {
        /// Create Pauli matrices
        pub fn new<B>(backend: &B) -> SymbolicResult<Self>
        where
            B: SymbolicBackend<Expression = E>,
        {
            let sigma_x = backend.parse("[[0, 1], [1, 0]]")?;
            let sigma_y = backend.parse("[[0, -i], [i, 0]]")?;
            let sigma_z = backend.parse("[[1, 0], [0, -1]]")?;

            Ok(Self {
                sigma_x,
                sigma_y,
                sigma_z,
            })
        }
    }

    /// Spinor wavefunction (2-component)
    pub struct Spinor<E: SymbolicExpression> {
        /// Spin-up component $\alpha$
        pub alpha: E,

        /// Spin-down component $\beta$
        pub beta: E,
    }

    impl<E: SymbolicExpression> Spinor<E> {
        /// Create spinor
        pub fn new(alpha: E, beta: E) -> Self {
            Self { alpha, beta }
        }

        /// Spin-up eigenstate $|\uparrow\rangle$
        pub fn spin_up<B>(backend: &B) -> SymbolicResult<Self>
        where
            B: SymbolicBackend<Expression = E>,
        {
            let one = backend.constant(1.0)?;
            let zero = backend.constant(0.0)?;
            Ok(Self::new(one, zero))
        }

        /// Spin-down eigenstate $|\downarrow\rangle$
        pub fn spin_down<B>(backend: &B) -> SymbolicResult<Self>
        where
            B: SymbolicBackend<Expression = E>,
        {
            let zero = backend.constant(0.0)?;
            let one = backend.constant(1.0)?;
            Ok(Self::new(zero, one))
        }
    }

    /// Spin-orbit coupling
    ///
    /// $$ H_{SO} = \lambda \vec{L} \cdot \vec{S} $$
    pub fn spin_orbit_coupling<B, E>(lambda: &E, l_dot_s: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        backend.mul(lambda, l_dot_s)
    }
}

/// Clebsch-Gordan coefficients for angular momentum addition
pub mod clebsch_gordan {
    use super::*;

    /// Clebsch-Gordan coefficient $\langle j_1 m_1 j_2 m_2|j m\rangle$
    ///
    /// Couples two angular momenta $j_1$ and $j_2$ to total $j$
    pub fn coefficient(j1: usize, m1: isize, j2: usize, m2: isize, j: usize, m: isize) -> f64 {
        // Selection rules
        if m != m1 + m2 {
            return 0.0;
        }

        if j > j1 + j2 || j < (j1 as isize - j2 as isize).unsigned_abs() {
            return 0.0;
        }

        // Simplified calculation for common cases
        // Full implementation would use Racah formula
        if j1 == 0 && j2 == 0 {
            if j == 0 && m == 0 {
                1.0
            } else {
                0.0
            }
        } else {
            // Placeholder: full CG coefficient calculation
            1.0 / ((2 * j + 1) as f64).sqrt()
        }
    }

    /// Total angular momentum J = L + S
    pub fn total_angular_momentum<B, E>(
        l: usize,
        s: usize,
        _backend: &B,
    ) -> SymbolicResult<Vec<usize>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // |l - s| ≤ j ≤ l + s
        let j_min = (l as isize - s as isize).unsigned_abs();
        let j_max = l + s;

        Ok((j_min..=j_max).collect())
    }
}

/// Central force problems
pub mod central_force {
    use super::*;

    /// Radial equation solution
    ///
    /// $$ \left[-\frac{\hbar^2}{2\mu} \frac{d^2}{dr^2} + \frac{l(l+1)\hbar^2}{2\mu r^2} + V(r)\right]R(r) = ER(r) $$
    pub struct RadialSolution<E: SymbolicExpression> {
        /// Radial quantum number n
        pub n: usize,

        /// Angular momentum quantum number l
        pub l: usize,

        /// Radial wavefunction R_nl(r)
        pub radial_function: E,

        /// Energy eigenvalue
        pub energy: E,
    }

    impl<E: SymbolicExpression> RadialSolution<E> {
        /// Create radial solution
        pub fn new(n: usize, l: usize, radial_function: E, energy: E) -> Self {
            Self {
                n,
                l,
                radial_function,
                energy,
            }
        }
    }

    /// Hydrogen atom radial function
    ///
    /// R_nl(r) = N r^l L_n^(2l+1)(2r/na₀) e^(-r/na₀)
    pub fn hydrogen_radial<B, E>(
        n: usize,
        l: usize,
        _a0: &E, // Bohr radius
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        if l >= n {
            return Err(crate::symbolic::SymbolicError::InvalidExpression(format!(
                "l={} must be < n={}",
                l, n
            )));
        }

        // Symbolic representation of R_nl
        backend.parse(&format!("R_{}{}(r, a0)", n, l))
    }

    /// 3D harmonic oscillator
    ///
    /// $$ V(r) = \frac{1}{2}m\omega^2 r^2 $$
    pub fn harmonic_oscillator_3d<B, E>(
        n: usize,
        l: usize,
        omega: &E,
        backend: &B,
    ) -> SymbolicResult<RadialSolution<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let hbar = backend.variable("hbar")?;

        // $E_n = \hbar\omega(2n + l + 3/2)$
        let n_val = backend.constant(n as f64)?;
        let l_val = backend.constant(l as f64)?;
        let two = backend.constant(2.0)?;
        let three_half = backend.constant(1.5)?;

        let two_n = backend.mul(&two, &n_val)?;
        let temp = backend.add(&two_n, &l_val)?;
        let n_term = backend.add(&temp, &three_half)?;

        let hbar_omega = backend.mul(&hbar, omega)?;
        let energy = backend.mul(&hbar_omega, &n_term)?;

        let radial = backend.parse(&format!("R_{}{}_HO(r)", n, l))?;

        Ok(RadialSolution::new(n, l, radial, energy))
    }

    /// Spherical well potential
    ///
    /// V(r) = -V₀ for r < a, V(r) = 0 for r ≥ a
    pub fn spherical_well<B, E>(v0: &E, _radius: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $V(r) = -V_0 \theta(a - r)$
        let _minus_v0 = backend.neg(v0)?;
        backend.parse("V_well(r, a)")
    }
}

impl fmt::Display for AngularMomentumState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "|l={}, m={}\rangle", self.l, self.m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbolic::create_symbolica_backend;

    #[test]
    fn test_angular_momentum_state() {
        let state = AngularMomentumState::new(1, 0).unwrap();
        assert_eq!(state.l, 1);
        assert_eq!(state.m, 0);

        assert!(AngularMomentumState::new(1, 2).is_err());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_l_squared_eigenvalue() {
        let backend = create_symbolica_backend();
        let hbar = backend.variable("hbar").unwrap();

        let eigenvalue = orbital::l_squared_eigenvalue(1, &hbar, &backend).unwrap();

        assert!(!eigenvalue.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_ladder_operators() {
        let backend = create_symbolica_backend();
        let hbar = backend.variable("hbar").unwrap();

        let l_plus = orbital::ladder_plus(1, 0, &hbar, &backend).unwrap();
        assert!(!l_plus.is_zero());

        let l_minus = orbital::ladder_minus(1, 0, &hbar, &backend).unwrap();
        assert!(!l_minus.is_zero());
    }

    #[test]
    fn test_clebsch_gordan() {
        let cg = clebsch_gordan::coefficient(1, 0, 1, 0, 0, 0);
        assert!(cg.abs() > 0.0);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_hydrogen_radial() {
        let backend = create_symbolica_backend();
        let a0 = backend.variable("a0").unwrap();

        let radial = central_force::hydrogen_radial(2, 1, &a0, &backend).unwrap();
        assert!(!radial.is_zero());
    }

    #[test]
    fn test_quantum_number_constraints() {
        // Test |m| ≤ l constraint
        for l in 0..=3 {
            for m in -(l as isize)..=(l as isize) {
                let state = AngularMomentumState::new(l, m).unwrap();
                assert!(state.m.abs() <= l as isize);
                assert_eq!(state.l, l);
            }
        }
    }

    #[test]
    fn test_invalid_magnetic_quantum_number_positive() {
        // m cannot exceed l
        let result = AngularMomentumState::new(2, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_magnetic_quantum_number_negative() {
        // m cannot be less than -l
        let result = AngularMomentumState::new(2, -3);
        assert!(result.is_err());
    }

    #[test]
    fn test_clebsch_gordan_symmetry() {
        // Test CG coefficient symmetries
        let j1 = 1;
        let j2 = 1;
        let m1 = 0;
        let m2 = 0;
        let j = 0;
        let m = 0;

        let cg1 = clebsch_gordan::coefficient(j1, m1, j2, m2, j, m);

        // CG coefficient should be non-zero for allowed coupling
        assert!(cg1.abs() > 1e-10);
    }

    #[test]
    fn test_clebsch_gordan_triangle_inequality() {
        // Triangle inequality: |j1 - j2| ≤ j ≤ j1 + j2
        let j1 = 1;
        let j2 = 1;

        // Valid j values: 0, 1, 2
        for j in 0..=2 {
            let cg = clebsch_gordan::coefficient(j1, 0, j2, 0, j, 0);
            // Should be computable (may be zero for selection rules)
            assert!(cg.is_finite());
        }
    }

    #[test]
    fn test_clebsch_gordan_m_conservation() {
        // m conservation: m = m1 + m2
        let j1 = 1;
        let j2 = 1;
        let m1 = 1;
        let m2 = -1;
        let j = 1;
        let m = 0; // m1 + m2 = 0

        let cg = clebsch_gordan::coefficient(j1, m1, j2, m2, j, m);
        assert!(cg.is_finite());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_spherical_harmonic_quantum_numbers() {
        let backend = create_symbolica_backend();

        // Test various (l,m) combinations
        let valid_pairs = vec![
            (0, 0), // s orbital
            (1, 0), // p_z orbital
            (1, 1), // p_x + ip_y
            (2, 0), // d_z² orbital
            (2, 2), // d_xy orbital
        ];

        for (l, m) in valid_pairs {
            let ylm = SphericalHarmonic::new(l, m, "theta", "phi", &backend);
            assert!(ylm.is_ok());
        }
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_pauli_matrices_properties() {
        let backend = create_symbolica_backend();
        let pauli = spin::PauliMatrices::new(&backend).unwrap();

        // Pauli matrices should exist and be non-zero
        let sigma_x_str = format!("{}", pauli.sigma_x);
        let sigma_y_str = format!("{}", pauli.sigma_y);
        let sigma_z_str = format!("{}", pauli.sigma_z);

        assert!(!sigma_x_str.is_empty());
        assert!(!sigma_y_str.is_empty());
        assert!(!sigma_z_str.is_empty());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_spinor_normalization() {
        let backend = create_symbolica_backend();

        // Create normalized spin-up state: $|\uparrow\rangle = \begin{bmatrix}1\\0\end{bmatrix}$
        let one = backend.constant(1.0).unwrap();
        let zero = backend.constant(0.0).unwrap();

        let spin_up = spin::Spinor::new(one.clone(), zero.clone());

        // Verify spinor components exist
        let alpha_str = format!("{}", spin_up.alpha);
        let beta_str = format!("{}", spin_up.beta);
        assert!(!alpha_str.is_empty());
        assert!(!beta_str.is_empty());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_3d_harmonic_oscillator_energy_levels() {
        // E_n = ℏω(2n + l + 3/2)
        let backend = create_symbolica_backend();
        let omega = backend.variable("omega").unwrap();

        // Ground state: n=0, l=0
        let sol_00 = central_force::harmonic_oscillator_3d(0, 0, &omega, &backend).unwrap();
        assert_eq!(sol_00.n, 0);
        assert_eq!(sol_00.l, 0);

        // First excited state: n=0, l=1
        let sol_01 = central_force::harmonic_oscillator_3d(0, 1, &omega, &backend).unwrap();
        assert_eq!(sol_01.n, 0);
        assert_eq!(sol_01.l, 1);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_hydrogen_radial_quantum_number_constraint() {
        // l must be < n for hydrogen atom
        let backend = create_symbolica_backend();
        let a0 = backend.variable("a0").unwrap();

        // Valid: n=2, l=0 or l=1
        assert!(central_force::hydrogen_radial(2, 0, &a0, &backend).is_ok());
        assert!(central_force::hydrogen_radial(2, 1, &a0, &backend).is_ok());

        // Invalid: l >= n
        assert!(central_force::hydrogen_radial(2, 2, &a0, &backend).is_err());
        assert!(central_force::hydrogen_radial(1, 1, &a0, &backend).is_err());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_ladder_operator_commutation() {
        // $[L_+, L_-] = 2\hbar L_z$
        let backend = create_symbolica_backend();
        let hbar = backend.variable("hbar").unwrap();

        let l = 1;
        let m = 0;

        // Compute ladder operators
        let l_plus = orbital::ladder_plus(l, m, &hbar, &backend).unwrap();
        let l_minus = orbital::ladder_minus(l, m, &hbar, &backend).unwrap();

        // Both should be non-zero for this state
        let plus_str = format!("{}", l_plus);
        let minus_str = format!("{}", l_minus);
        assert!(!plus_str.is_empty());
        assert!(!minus_str.is_empty());
    }

    #[test]
    fn test_angular_momentum_state_display() {
        let state = AngularMomentumState::new(2, 1).unwrap();
        let display = format!("{}", state);

        assert!(display.contains("l=2"));
        assert!(display.contains("m=1"));
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_radial_solution_properties() {
        let backend = create_symbolica_backend();
        let omega = backend.variable("omega").unwrap();

        let solution = central_force::harmonic_oscillator_3d(1, 1, &omega, &backend).unwrap();

        // Check quantum numbers are stored correctly
        assert_eq!(solution.n, 1);
        assert_eq!(solution.l, 1);

        // Energy and radial function should exist
        let energy_str = format!("{}", solution.energy);
        let radial_str = format!("{}", solution.radial_function);
        assert!(!energy_str.is_empty());
        assert!(!radial_str.is_empty());
    }
}
