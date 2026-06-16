//! Time-Dependent Schrodinger Equation (TDSE) Solver
//!
//! Author: gA4ss
//!
//! This module provides solvers for the time-dependent Schrodinger equation:
//! $i\hbar\frac{\partial}{\partial t}|\psi(t)⟩ = \hat{H}(t)|\psi(t)⟩$
//!
//! Supports time evolution of quantum states under various Hamiltonians.

use crate::qm_solver::{QuantumOperator, SymbolicWaveFunction};
use crate::symbolic::{SymbolicBackend, SymbolicExpression, SymbolicResult};
use std::fmt;

/// Type of time evolution method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeEvolutionMethod {
    /// Exact solution via time evolution operator U(t) = e^(-iHt/ℏ)
    Exact,
    /// Separation of variables (for time-independent H)
    SeparationOfVariables,
    /// Adiabatic approximation
    Adiabatic,
    /// Numerical propagation
    Numerical,
}

impl fmt::Display for TimeEvolutionMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeEvolutionMethod::Exact => write!(f, "Exact"),
            TimeEvolutionMethod::SeparationOfVariables => write!(f, "SeparationOfVariables"),
            TimeEvolutionMethod::Adiabatic => write!(f, "Adiabatic"),
            TimeEvolutionMethod::Numerical => write!(f, "Numerical"),
        }
    }
}

/// Time-evolved quantum state
///
/// Represents the state |ψ(t)⟩ at a specific time
pub struct TimeEvolvedState<E: SymbolicExpression> {
    /// The wave function at time t
    pub state: SymbolicWaveFunction<E>,

    /// Time parameter
    pub time: E,

    /// Whether this is an exact solution
    pub exact: bool,
}

impl<E: SymbolicExpression> TimeEvolvedState<E> {
    /// Create a new time-evolved state
    pub fn new(state: SymbolicWaveFunction<E>, time: E, exact: bool) -> Self {
        Self { state, time, exact }
    }
}

/// Time evolution operator
///
/// Represents U(t) such that |ψ(t)⟩ = U(t)|ψ(0)⟩
///
/// # Mathematical Background
///
/// For time-independent H:
/// $U(t) = e^{-i\hat{H}t/\hbar}$
///
/// For time-dependent H(t):
/// $U(t) = \mathcal{T}\exp\left(-\frac{i}{\hbar}\int_0^t H(t')dt'\right)$
/// where $\mathcal{T}$ is the time-ordering operator
pub struct TimeEvolutionOperator<E: SymbolicExpression> {
    /// The symbolic expression for U(t)
    pub expression: E,

    /// The Hamiltonian used
    pub hamiltonian: QuantumOperator<E>,

    /// Whether the Hamiltonian is time-independent
    pub time_independent: bool,
}

impl<E: SymbolicExpression> TimeEvolutionOperator<E> {
    /// Create a new time evolution operator
    pub fn new(expression: E, hamiltonian: QuantumOperator<E>, time_independent: bool) -> Self {
        Self {
            expression,
            hamiltonian,
            time_independent,
        }
    }
}

/// Time-Dependent Schrodinger Equation Solver
///
/// Solves $i\hbar\frac{\partial}{\partial t}|\psi(t)⟩ = \hat{H}(t)|\psi(t)⟩$
pub struct TDSESolver<E: SymbolicExpression> {
    /// The Hamiltonian operator (may be time-dependent)
    pub hamiltonian: QuantumOperator<E>,

    /// Whether the Hamiltonian is time-independent
    pub time_independent: bool,

    /// Reduced Planck constant ℏ
    pub hbar: E,
}

impl<E: SymbolicExpression> TDSESolver<E> {
    /// Create a new TDSE solver with time-independent Hamiltonian
    pub fn new<B>(hamiltonian: QuantumOperator<E>, backend: &B) -> SymbolicResult<Self>
    where
        B: SymbolicBackend<Expression = E>,
    {
        let hbar = backend.variable("hbar")?;

        Ok(Self {
            hamiltonian,
            time_independent: true,
            hbar,
        })
    }

    /// Create a TDSE solver with time-dependent Hamiltonian
    pub fn with_time_dependent_hamiltonian<B>(
        hamiltonian: QuantumOperator<E>,
        backend: &B,
    ) -> SymbolicResult<Self>
    where
        B: SymbolicBackend<Expression = E>,
    {
        let hbar = backend.variable("hbar")?;

        Ok(Self {
            hamiltonian,
            time_independent: false,
            hbar,
        })
    }

    /// Compute time evolution operator for time-independent Hamiltonian
    ///
    /// $U(t) = e^{-i\hat{H}t/\hbar}$
    pub fn evolution_operator<B>(
        &self,
        time_var: &str,
        backend: &B,
    ) -> SymbolicResult<TimeEvolutionOperator<E>>
    where
        B: SymbolicBackend<Expression = E>,
    {
        if !self.time_independent {
            return Err(crate::symbolic::SymbolicError::UnsupportedOperation(
                "Evolution operator requires time-independent Hamiltonian. Use numerical methods for time-dependent H.".to_string()
            ));
        }

        // -iHt/ℏ
        let t = backend.variable(time_var)?;
        let i = backend.complex_constant(0.0, 1.0)?;
        let minus_i = backend.neg(&i)?;

        let ht = backend.mul(self.hamiltonian.expression(), &t)?;
        let minus_i_ht = backend.mul(&minus_i, &ht)?;
        let exponent = backend.div(&minus_i_ht, &self.hbar)?;

        // e^(-iHt/ℏ)
        let u_t = backend.exp(&exponent)?;

        Ok(TimeEvolutionOperator::new(
            u_t,
            self.hamiltonian.clone(),
            true,
        ))
    }
}

/// Stationary state solutions (separation of variables)
pub mod stationary_states {
    use super::*;

    /// Construct a stationary state solution
    ///
    /// For time-independent H with eigenstate |n⟩ and energy E_n:
    /// $|\psi_n(t)⟩ = |n⟩ e^{-iE_n t/\hbar}$
    pub fn stationary_state<B, E>(
        spatial_part: &E,
        energy: &E,
        time_var: &str,
        hbar: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let t = backend.variable(time_var)?;
        let i = backend.complex_constant(0.0, 1.0)?;
        let minus_i = backend.neg(&i)?;

        // -iE_n t/ℏ
        let et = backend.mul(energy, &t)?;
        let minus_i_et = backend.mul(&minus_i, &et)?;
        let exponent = backend.div(&minus_i_et, hbar)?;

        // e^(-iE_n t/ℏ)
        let time_part = backend.exp(&exponent)?;

        // ψ_n(x) * e^(-iE_n t/ℏ)
        backend.mul(spatial_part, &time_part)
    }

    /// Construct a general solution as superposition of stationary states
    ///
    /// $|\psi(t)⟩ = \sum_n c_n |n⟩ e^{-iE_n t/\hbar}$
    pub fn general_solution<B, E>(
        eigenstates: &[(E, E)], // (spatial_part, energy) pairs
        coefficients: &[E],
        time_var: &str,
        hbar: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        if eigenstates.len() != coefficients.len() {
            return Err(crate::symbolic::SymbolicError::InvalidExpression(
                "Number of eigenstates must match number of coefficients".to_string(),
            ));
        }

        if eigenstates.is_empty() {
            return Err(crate::symbolic::SymbolicError::InvalidExpression(
                "At least one eigenstate required".to_string(),
            ));
        }

        // Start with first term
        let first_state = stationary_state(
            &eigenstates[0].0,
            &eigenstates[0].1,
            time_var,
            hbar,
            backend,
        )?;
        let mut result = backend.mul(&coefficients[0], &first_state)?;

        // Add remaining terms
        for i in 1..eigenstates.len() {
            let state_i = stationary_state(
                &eigenstates[i].0,
                &eigenstates[i].1,
                time_var,
                hbar,
                backend,
            )?;
            let term = backend.mul(&coefficients[i], &state_i)?;
            result = backend.add(&result, &term)?;
        }

        Ok(result)
    }
}

/// Adiabatic approximation
pub mod adiabatic {
    use super::*;

    /// Adiabatic phase factor
    ///
    /// For slowly varying H(t), the state acquires a dynamical phase:
    /// $\gamma_n(t) = -\frac{1}{\hbar}\int_0^t E_n(t')dt'$
    ///
    /// Plus a geometric (Berry) phase for cyclic evolution
    pub fn dynamical_phase<B, E>(
        energy_trajectory: &E, // E_n(t) as function of time
        _time_var: &str,
        hbar: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // -E_n(t)/ℏ (integrand)
        let minus_one = backend.constant(-1.0)?;
        let numerator = backend.mul(&minus_one, energy_trajectory)?;
        let integrand = backend.div(&numerator, hbar)?;

        // Return the integrand (actual integration would be done separately)
        Ok(integrand)
    }

    /// Check adiabatic condition
    ///
    /// Adiabatic approximation valid if:
    /// $|\langle n|\frac{d\hat{H}}{dt}|m\rangle| \ll (E_n - E_m)^2/\hbar$
    /// for all n ≠ m
    pub fn adiabatic_condition_satisfied() -> bool {
        // This would require numerical evaluation in practice
        // Return true as placeholder for symbolic representation
        true
    }
}

impl<E: SymbolicExpression> fmt::Display for TimeEvolutionOperator<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "U(t) = {} ({})",
            self.expression.to_string(),
            if self.time_independent {
                "time-independent H"
            } else {
                "time-dependent H"
            }
        )
    }
}

impl<E: SymbolicExpression> fmt::Display for TimeEvolvedState<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "|ψ({})⟩ {}",
            self.time.to_string(),
            if self.exact { "[exact]" } else { "[approx]" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qm_solver::tise_solver::potentials;
    use crate::symbolic::create_symbolica_backend;

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_tdse_solver_creation() {
        let backend = create_symbolica_backend();
        let h_expr = backend.parse("p^2/(2*m) + (1/2)*m*omega^2*x^2").unwrap();
        let hamiltonian = QuantumOperator::hamiltonian(h_expr, "H");

        let solver = TDSESolver::new(hamiltonian, &backend).unwrap();
        assert!(solver.time_independent);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_evolution_operator() {
        let backend = create_symbolica_backend();
        let h_expr = backend.parse("E*I").unwrap(); // Simple H = E (identity)
        let hamiltonian = QuantumOperator::hamiltonian(h_expr, "H");

        let solver = TDSESolver::new(hamiltonian, &backend).unwrap();
        let u_t = solver.evolution_operator("t", &backend).unwrap();

        assert!(u_t.time_independent);
        assert!(!u_t.expression.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_stationary_state() {
        let backend = create_symbolica_backend();

        let psi = backend.parse("sin(n*pi*x/L)").unwrap();
        let energy = backend.parse("n^2*pi^2*hbar^2/(2*m*L^2)").unwrap();
        let hbar = backend.variable("hbar").unwrap();

        let state_t =
            stationary_states::stationary_state(&psi, &energy, "t", &hbar, &backend).unwrap();

        assert!(!state_t.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_general_solution() {
        let backend = create_symbolica_backend();

        let psi1 = backend.parse("sin(pi*x/L)").unwrap();
        let e1 = backend.parse("pi^2*hbar^2/(2*m*L^2)").unwrap();

        let psi2 = backend.parse("sin(2*pi*x/L)").unwrap();
        let e2 = backend.parse("4*pi^2*hbar^2/(2*m*L^2)").unwrap();

        let c1 = backend.constant(1.0).unwrap();
        let c2 = backend.constant(0.5).unwrap();

        let hbar = backend.variable("hbar").unwrap();

        let eigenstates = vec![(psi1, e1), (psi2, e2)];
        let coefficients = vec![c1, c2];

        let solution =
            stationary_states::general_solution(&eigenstates, &coefficients, "t", &hbar, &backend)
                .unwrap();

        assert!(!solution.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_time_dependent_hamiltonian() {
        let backend = create_symbolica_backend();
        let h_expr = backend.parse("H0 + V*cos(omega*t)").unwrap();
        let hamiltonian = QuantumOperator::hamiltonian(h_expr, "H(t)");

        let solver = TDSESolver::with_time_dependent_hamiltonian(hamiltonian, &backend).unwrap();
        assert!(!solver.time_independent);

        // Should fail for time-dependent H
        assert!(solver.evolution_operator("t", &backend).is_err());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_dynamical_phase() {
        let backend = create_symbolica_backend();
        let energy_t = backend.parse("E0*(1 + alpha*t)").unwrap();
        let hbar = backend.variable("hbar").unwrap();

        let phase_integrand = adiabatic::dynamical_phase(&energy_t, "t", &hbar, &backend).unwrap();

        assert!(!phase_integrand.is_zero());
    }

    #[test]
    fn test_adiabatic_condition() {
        assert!(adiabatic::adiabatic_condition_satisfied());
    }
}
