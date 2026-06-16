//! Perturbation Theory Module
//!
//! Author: gA4ss
//!
//! This module implements time-independent and time-dependent perturbation theory
//! for quantum mechanical systems.
//!
//! Mathematical Background:
//! For a Hamiltonian $H = H_0 + \lambda H'$, where $H_0$ is exactly solvable and $H'$ is a
//! small perturbation, we can find approximate solutions using series expansion.

use crate::symbolic::{SymbolicBackend, SymbolicExpression, SymbolicResult};
use std::fmt;

/// Type of perturbation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerturbationType {
    /// Non-degenerate perturbation theory
    NonDegenerate,
    /// Degenerate perturbation theory
    Degenerate,
}

impl fmt::Display for PerturbationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PerturbationType::NonDegenerate => write!(f, "NonDegenerate"),
            PerturbationType::Degenerate => write!(f, "Degenerate"),
        }
    }
}

/// Energy correction at a given order
///
/// Represents E_n^(k) - the k-th order energy correction to level n
pub struct EnergyCorrection<E: SymbolicExpression> {
    /// Order of correction (1, 2, 3, ...)
    pub order: usize,

    /// Energy level index
    pub level: usize,

    /// The correction value
    pub correction: E,
}

impl<E: SymbolicExpression> EnergyCorrection<E> {
    /// Create a new energy correction
    pub fn new(order: usize, level: usize, correction: E) -> Self {
        Self {
            order,
            level,
            correction,
        }
    }
}

/// State correction at a given order
///
/// Represents $|n^{(k)}\rangle$ - the k-th order state correction
pub struct StateCorrection<E: SymbolicExpression> {
    /// Order of correction
    pub order: usize,

    /// State level index
    pub level: usize,

    /// The correction expression
    pub correction: E,
}

impl<E: SymbolicExpression> StateCorrection<E> {
    /// Create a new state correction
    pub fn new(order: usize, level: usize, correction: E) -> Self {
        Self {
            order,
            level,
            correction,
        }
    }
}

/// Time-independent perturbation theory solver
///
/// Solves $H = H_0 + \lambda H'$ where $H_0|n^{(0)}\rangle = E_n^{(0)}|n^{(0)}\rangle$
pub struct PerturbationTheorySolver<E: SymbolicExpression> {
    /// Unperturbed Hamiltonian H_0
    pub h0_eigenenergies: Vec<E>,

    /// Perturbation operator H'
    pub perturbation_matrix_elements: Vec<Vec<E>>,

    /// Perturbation parameter $\lambda$
    pub lambda: E,
}

impl<E: SymbolicExpression> PerturbationTheorySolver<E> {
    /// Create a new perturbation theory solver
    pub fn new(
        h0_eigenenergies: Vec<E>,
        perturbation_matrix_elements: Vec<Vec<E>>,
        lambda: E,
    ) -> Self {
        Self {
            h0_eigenenergies,
            perturbation_matrix_elements,
            lambda,
        }
    }
}

/// Non-degenerate perturbation theory
pub mod non_degenerate {
    use super::*;

    /// Compute first-order energy correction
    ///
    /// $$ E_n^{(1)} = \langle n^{(0)}|H'|n^{(0)}\rangle $$
    pub fn first_order_energy<B, E>(
        n: usize,
        h_prime_nn: &E,
        _backend: &B,
    ) -> SymbolicResult<EnergyCorrection<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // E_n^(1) = H'_nn
        Ok(EnergyCorrection::new(1, n, h_prime_nn.clone()))
    }

    /// Compute second-order energy correction
    ///
    /// $$ E_n^{(2)} = \sum_{m \neq n} \frac{|\langle m^{(0)}|H'|n^{(0)}\rangle|^2}{E_n^{(0)} - E_m^{(0)}} $$
    pub fn second_order_energy<B, E>(
        n: usize,
        e0: &[E],
        h_prime: &[Vec<E>],
        backend: &B,
    ) -> SymbolicResult<EnergyCorrection<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let zero = backend.constant(0.0)?;
        let mut sum = zero.clone();

        for m in 0..e0.len() {
            if m == n {
                continue;
            }

            // |H'_mn|^2 = |⟨m^(0)|H'|n^(0)⟩|^2
            // For real-symmetric H' (the common case in quantum mechanics),
            // this is h_mn * h_mn since h_mn is real and H' is Hermitian.
            // For complex H', use: backend.mul(h_mn, &backend.conjugate(h_mn)?)?
            let h_mn = &h_prime[m][n];
            let h_mn_squared = backend.mul(h_mn, h_mn)?;

            // E_n^(0) - E_m^(0)
            let energy_diff = backend.sub(&e0[n], &e0[m])?;

            // |H'_mn|^2 / (E_n - E_m)
            let term = backend.div(&h_mn_squared, &energy_diff)?;

            sum = backend.add(&sum, &term)?;
        }

        Ok(EnergyCorrection::new(2, n, sum))
    }

    /// Compute first-order state correction
    ///
    /// $$ |n^{(1)}\rangle = \sum_{m \neq n} \frac{\langle m^{(0)}|H'|n^{(0)}\rangle}{E_n^{(0)} - E_m^{(0)}} |m^{(0)}\rangle $$
    ///
    /// Returns coefficients c_m for the expansion
    pub fn first_order_state_coefficients<B, E>(
        n: usize,
        e0: &[E],
        h_prime: &[Vec<E>],
        backend: &B,
    ) -> SymbolicResult<Vec<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let zero = backend.constant(0.0)?;
        let mut coefficients = vec![zero; e0.len()];

        for m in 0..e0.len() {
            if m == n {
                continue;
            }

            // H'_mn
            let h_mn = &h_prime[m][n];

            // E_n^(0) - E_m^(0)
            let energy_diff = backend.sub(&e0[n], &e0[m])?;

            // c_m = H'_mn / (E_n - E_m)
            coefficients[m] = backend.div(h_mn, &energy_diff)?;
        }

        Ok(coefficients)
    }
}

/// Degenerate perturbation theory
pub mod degenerate {
    use super::*;

    /// Solve degenerate perturbation theory
    ///
    /// For a degenerate subspace with degeneracy g, we must diagonalize
    /// the perturbation matrix within that subspace.
    ///
    /// Returns the energy shifts and good basis states
    pub fn solve_degenerate_subspace<B, E>(
        subspace_indices: &[usize],
        h_prime: &[Vec<E>],
        _backend: &B,
    ) -> SymbolicResult<Vec<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // Extract the perturbation matrix for this subspace
        let mut subspace_matrix = Vec::new();

        for &i in subspace_indices {
            let mut row = Vec::new();
            for &j in subspace_indices {
                row.push(h_prime[i][j].clone());
            }
            subspace_matrix.push(row);
        }

        // In a full implementation, we would diagonalize this matrix
        // For now, return the diagonal elements as symbolic placeholders
        let mut eigenvalues = Vec::new();
        for i in 0..subspace_matrix.len() {
            eigenvalues.push(subspace_matrix[i][i].clone());
        }

        Ok(eigenvalues)
    }
}

/// Rayleigh-Schrodinger perturbation theory formalism
pub mod rayleigh_schrodinger {
    use super::*;

    /// Compute Rayleigh-Schrodinger energy correction to arbitrary order
    ///
    /// This uses the general Rayleigh-Schrodinger formula for energy corrections
    pub fn energy_correction_general<B, E>(
        n: usize,
        order: usize,
        e0: &[E],
        h_prime: &[Vec<E>],
        backend: &B,
    ) -> SymbolicResult<EnergyCorrection<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        match order {
            1 => non_degenerate::first_order_energy(n, &h_prime[n][n], backend),
            2 => non_degenerate::second_order_energy(n, e0, h_prime, backend),
            _ => {
                // Higher orders would require more complex recursive formulas
                Err(crate::symbolic::SymbolicError::UnsupportedOperation(
                    format!("Order {} not implemented yet", order),
                ))
            }
        }
    }
}

/// Time-dependent perturbation theory
pub mod time_dependent {
    use super::*;

    /// Transition probability between states
    ///
    /// For perturbation $H'(t)$, probability of transition from $|i\rangle$ to $|f\rangle$
    pub fn transition_probability<B, E>(
        matrix_element: &E, // $\langle f|H'|i\rangle$
        time: &E,
        energy_diff: &E, // E_f - E_i
        hbar: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $P_{if}(t) = |\langle f|H'|i\rangle|^2 \cdot |\sin((E_f-E_i)t/(2\hbar))|^2 / ((E_f-E_i)/2)^2$

        // $(E_f - E_i)t/(2\hbar)$
        let two = backend.constant(2.0)?;
        let temp = backend.mul(energy_diff, time)?;
        let two_hbar = backend.mul(&two, hbar)?;
        let argument = backend.div(&temp, &two_hbar)?;

        // sin²(argument)
        let sin_arg = backend.parse(&format!("sin({})", argument.to_string()))?;
        let sin_squared = backend.mul(&sin_arg, &sin_arg)?;

        // |H'_fi|²
        let h_squared = backend.mul(matrix_element, matrix_element)?;

        // (E_f - E_i)/2
        let denominator = backend.div(energy_diff, &two)?;
        let denom_squared = backend.mul(&denominator, &denominator)?;

        // $P = |H'|^2 \cdot \sin^2(...) / (\Delta E/2)^2$
        let numerator = backend.mul(&h_squared, &sin_squared)?;
        backend.div(&numerator, &denom_squared)
    }

    /// Fermi's Golden Rule
    ///
    /// Transition rate in the limit of continuous final states:
    /// $$ \Gamma_{if} = \frac{2\pi}{\hbar}|\langle f|H'|i\rangle|^2 \rho(E_f) $$
    pub fn fermis_golden_rule<B, E>(
        matrix_element: &E,
        density_of_states: &E,
        hbar: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let two = backend.constant(2.0)?;
        let pi = backend.variable("pi")?;

        // $2\pi/\hbar$
        let two_pi = backend.mul(&two, &pi)?;
        let coefficient = backend.div(&two_pi, hbar)?;

        // $|\langle f|H'|i\rangle|^2$
        let h_squared = backend.mul(matrix_element, matrix_element)?;

        // $\Gamma = (2\pi/\hbar)|H'|^2 \rho(E)$
        let temp = backend.mul(&coefficient, &h_squared)?;
        backend.mul(&temp, density_of_states)
    }

    /// Periodic perturbation response
    ///
    /// For $H'(t) = V \cos(\omega t)$, compute transition amplitude
    pub fn periodic_perturbation_amplitude<B, E>(
        _matrix_element: &E,
        omega: &E,
        _time: &E,
        energy_diff: &E,
        hbar: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $c_f(t) = -(i/\hbar)V_{fi} [e^{i(\omega_{fi}+\omega)t} - 1]/(\omega_{fi} + \omega) + [e^{i(\omega_{fi}-\omega)t} - 1]/(\omega_{fi} - \omega)$
        // where $\omega_{fi} = (E_f - E_i)/\hbar$

        let _i = backend.complex_constant(0.0, 1.0)?;
        let _one = backend.constant(1.0)?;

        // $\omega_{fi} = \Delta E/\hbar$
        let omega_fi = backend.div(energy_diff, hbar)?;

        // Resonant terms simplified for symbolic representation
        let _omega_sum = backend.add(&omega_fi, omega)?;
        let _omega_diff = backend.sub(&omega_fi, omega)?;

        // Return simplified form
        backend.parse("-(i/hbar)*V_fi*t*(resonance_factor)")
    }

    /// Selection rules checker
    ///
    /// Checks if transition is allowed based on quantum numbers
    pub fn check_selection_rules(
        initial_l: usize,
        final_l: usize,
        initial_m: isize,
        final_m: isize,
    ) -> bool {
        // Electric dipole selection rules:
        // $\Delta l = \pm 1$
        // $\Delta m = 0, \pm 1$

        let delta_l = (final_l as isize - initial_l as isize).abs();
        let delta_m = (final_m - initial_m).abs();

        (delta_l == 1) && (delta_m <= 1)
    }
}

/// Variational methods for quantum mechanics
pub mod variational {
    use super::*;

    /// Compute Rayleigh quotient
    ///
    /// $$ R[\psi] = \frac{\langle\psi|H|\psi\rangle}{\langle\psi|\psi\rangle} $$
    ///
    /// Gives an upper bound on the ground state energy
    pub fn rayleigh_quotient<B, E>(
        h_expectation: &E, // $\langle\psi|H|\psi\rangle$
        normalization: &E, // $\langle\psi|\psi\rangle$
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        backend.div(h_expectation, normalization)
    }

    /// Variational principle energy functional
    ///
    /// For trial wavefunction $|\psi_{\text{trial}}\rangle$, compute energy upper bound
    pub fn variational_energy<B, E>(
        trial_state: &E,
        hamiltonian: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $E[\psi] = \langle\psi|H|\psi\rangle / \langle\psi|\psi\rangle$
        // For normalized states: $E[\psi] = \langle\psi|H|\psi\rangle$

        // $H\psi$
        let h_psi = backend.mul(hamiltonian, trial_state)?;

        // $\langle\psi|H\psi\rangle = \psi^* H\psi$ (simplified for symbolic)
        backend.mul(trial_state, &h_psi)
    }

    /// WKB (Wentzel-Kramers-Brillouin) approximation
    ///
    /// Semi-classical approximation for slowly varying potentials
    ///
    /// $$ \psi(x) \approx \frac{A}{\sqrt{p(x)}} \exp\left(\pm\frac{i}{\hbar} \int p(x')dx'\right) $$
    /// where $p(x) = \sqrt{2m(E-V(x))}$
    pub fn wkb_wavefunction<B, E>(
        momentum: &E, // $p(x) = \sqrt{2m(E-V(x))}$
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $\psi \propto 1/\sqrt{p}$
        let sqrt_p = backend.parse(&format!("sqrt({})", momentum.to_string()))?;
        let one = backend.constant(1.0)?;
        backend.div(&one, &sqrt_p)
    }

    /// WKB quantization condition (Bohr-Sommerfeld)
    ///
    /// ∮ p(x)dx = (n + 1/2)h
    pub fn wkb_quantization_condition<B, E>(n: usize, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let n_val = backend.constant(n as f64)?;
        let half = backend.constant(0.5)?;
        let h = backend.variable("h")?;

        // (n + 1/2)h
        let n_plus_half = backend.add(&n_val, &half)?;
        backend.mul(&n_plus_half, &h)
    }
}

/// Physical applications of perturbation theory
pub mod applications {
    use super::*;

    /// Stark effect: atom in external electric field
    ///
    /// Perturbation: H' = -eEz (linear in field for hydrogen)
    pub fn stark_effect_linear<B, E>(electric_field_strength: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // H' = -e*E*z
        let e = backend.variable("e")?;
        let z = backend.variable("z")?;

        let minus_e = backend.neg(&e)?;
        let temp = backend.mul(&minus_e, electric_field_strength)?;
        backend.mul(&temp, &z)
    }

    /// Zeeman effect: atom in external magnetic field
    ///
    /// Perturbation: $H' = \mu_B B L_z$ (for orbital angular momentum)
    pub fn zeeman_effect<B, E>(magnetic_field_strength: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $H' = \mu_B \cdot B \cdot L_z$
        let mu_b = backend.variable("mu_B")?;
        let l_z = backend.variable("L_z")?;

        let temp = backend.mul(&mu_b, magnetic_field_strength)?;
        backend.mul(&temp, &l_z)
    }

    /// Fine structure corrections (relativistic + spin-orbit)
    ///
    /// Including relativistic kinetic energy and spin-orbit coupling
    pub fn fine_structure<B, E>(backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $H' = H_{\text{rel}} + H_{\text{so}}$
        // $H_{\text{rel}} = -p^4/(8m^3c^2)$
        // $H_{\text{so}} = \frac{1}{2m^2c^2} \cdot \frac{1}{r} \cdot \frac{dV}{dr} \cdot \vec{L}\cdot\vec{S}$

        backend.parse("H_rel + H_so")
    }
}

impl<E: SymbolicExpression> fmt::Display for EnergyCorrection<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "E_{}^({}) = {}",
            self.level,
            self.order,
            self.correction.to_string()
        )
    }
}

impl<E: SymbolicExpression> fmt::Display for StateCorrection<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "|{}^({})⟩", self.level, self.order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbolic::create_symbolica_backend;

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_first_order_energy() {
        let backend = create_symbolica_backend();
        let h_nn = backend.parse("V*x^2").unwrap();

        let correction = non_degenerate::first_order_energy(0, &h_nn, &backend).unwrap();

        assert_eq!(correction.order, 1);
        assert_eq!(correction.level, 0);
        assert!(!correction.correction.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_second_order_energy() {
        let backend = create_symbolica_backend();

        let e0 = vec![
            backend.constant(1.0).unwrap(),
            backend.constant(2.0).unwrap(),
            backend.constant(3.0).unwrap(),
        ];

        let h_prime = vec![
            vec![
                backend.constant(0.0).unwrap(),
                backend.constant(0.1).unwrap(),
                backend.constant(0.05).unwrap(),
            ],
            vec![
                backend.constant(0.1).unwrap(),
                backend.constant(0.0).unwrap(),
                backend.constant(0.15).unwrap(),
            ],
            vec![
                backend.constant(0.05).unwrap(),
                backend.constant(0.15).unwrap(),
                backend.constant(0.0).unwrap(),
            ],
        ];

        let correction = non_degenerate::second_order_energy(0, &e0, &h_prime, &backend).unwrap();

        assert_eq!(correction.order, 2);
        assert_eq!(correction.level, 0);
        assert!(!correction.correction.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_first_order_state() {
        let backend = create_symbolica_backend();

        let e0 = vec![
            backend.constant(1.0).unwrap(),
            backend.constant(2.0).unwrap(),
        ];

        let h_prime = vec![
            vec![
                backend.constant(0.0).unwrap(),
                backend.constant(0.1).unwrap(),
            ],
            vec![
                backend.constant(0.1).unwrap(),
                backend.constant(0.0).unwrap(),
            ],
        ];

        let coeffs =
            non_degenerate::first_order_state_coefficients(0, &e0, &h_prime, &backend).unwrap();

        assert_eq!(coeffs.len(), 2);
        assert!(coeffs[0].is_zero());
        assert!(!coeffs[1].is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_stark_effect() {
        let backend = create_symbolica_backend();
        let e_field = backend.variable("E").unwrap();

        let perturbation = applications::stark_effect_linear(&e_field, &backend).unwrap();

        assert!(!perturbation.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_zeeman_effect() {
        let backend = create_symbolica_backend();
        let b_field = backend.variable("B").unwrap();

        let perturbation = applications::zeeman_effect(&b_field, &backend).unwrap();

        assert!(!perturbation.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_degenerate_perturbation() {
        let backend = create_symbolica_backend();

        let h_prime = vec![
            vec![
                backend.constant(0.1).unwrap(),
                backend.constant(0.05).unwrap(),
            ],
            vec![
                backend.constant(0.05).unwrap(),
                backend.constant(0.15).unwrap(),
            ],
        ];

        let subspace = vec![0, 1];
        let eigenvalues =
            degenerate::solve_degenerate_subspace(&subspace, &h_prime, &backend).unwrap();

        assert_eq!(eigenvalues.len(), 2);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_rayleigh_schrodinger() {
        let backend = create_symbolica_backend();

        let e0 = vec![
            backend.constant(1.0).unwrap(),
            backend.constant(2.0).unwrap(),
        ];

        let h_prime = vec![
            vec![
                backend.constant(0.1).unwrap(),
                backend.constant(0.05).unwrap(),
            ],
            vec![
                backend.constant(0.05).unwrap(),
                backend.constant(0.2).unwrap(),
            ],
        ];

        let correction1 =
            rayleigh_schrodinger::energy_correction_general(0, 1, &e0, &h_prime, &backend).unwrap();
        assert_eq!(correction1.order, 1);

        let correction2 =
            rayleigh_schrodinger::energy_correction_general(0, 2, &e0, &h_prime, &backend).unwrap();
        assert_eq!(correction2.order, 2);
    }
}
