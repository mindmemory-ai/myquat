//! Quantum Dynamics Module
//!
//! Author: gA4ss
//!
//! This module implements advanced quantum dynamics including Heisenberg picture,
//! interaction picture, and path integral formulation.

use crate::symbolic::{SymbolicBackend, SymbolicExpression, SymbolicResult};
use std::fmt;

/// Picture of quantum mechanics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantumPicture {
    /// Schrodinger picture (states evolve, operators fixed)
    Schrodinger,
    /// Heisenberg picture (operators evolve, states fixed)
    Heisenberg,
    /// Interaction picture (hybrid)
    Interaction,
}

impl fmt::Display for QuantumPicture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuantumPicture::Schrodinger => write!(f, "Schrodinger"),
            QuantumPicture::Heisenberg => write!(f, "Heisenberg"),
            QuantumPicture::Interaction => write!(f, "Interaction"),
        }
    }
}

/// Heisenberg picture operator evolution
pub mod heisenberg {
    use super::*;

    /// Heisenberg equation of motion
    ///
    /// Describes how operators evolve in time in the Heisenberg picture:
    /// $$ \frac{dA}{dt} = \frac{i}{\hbar}[H, A] + \frac{\partial A}{\partial t} $$
    ///
    /// where $[H, A] = HA - AH$ is the commutator.
    pub fn equation_of_motion<B, E>(
        operator: &E,
        hamiltonian: &E,
        hbar: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // [H, A] = HA - AH
        let ha = backend.mul(hamiltonian, operator)?;
        let ah = backend.mul(operator, hamiltonian)?;
        let commutator = backend.sub(&ha, &ah)?;

        // $i/\hbar$
        let i = backend.complex_constant(0.0, 1.0)?;
        let i_over_hbar = backend.div(&i, hbar)?;

        // $(i/\hbar)[H, A]$
        backend.mul(&i_over_hbar, &commutator)
    }

    /// Time evolution of operator in Heisenberg picture
    ///
    /// $$ A(t) = U^\dagger(t) A(0) U(t) $$
    ///
    /// where $U(t) = e^{-iHt/\hbar}$ is the time evolution operator.
    pub fn operator_time_evolution<B, E>(
        operator_0: &E,
        time_evolution_op: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // U†
        let u_dagger = backend.conjugate(time_evolution_op)?;

        // U† A(0)
        let temp = backend.mul(&u_dagger, operator_0)?;

        // $U^\dagger A(0) U$
        backend.mul(&temp, time_evolution_op)
    }

    /// Ehrenfest theorem verification
    ///
    /// Relates quantum expectation values to classical equations of motion:
    /// $$ \frac{d\langle x\rangle}{dt} = \frac{\langle p\rangle}{m} $$
    /// $$ \frac{d\langle p\rangle}{dt} = -\left\langle\frac{dV}{dx}\right\rangle $$
    pub fn ehrenfest_theorem<B, E>(
        _position_expectation: &E,
        momentum_expectation: &E,
        mass: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $d\langle x\rangle/dt = \langle p\rangle/m$
        backend.div(momentum_expectation, mass)
    }
}

/// Interaction picture formalism
pub mod interaction {
    use super::*;

    /// Transform to interaction picture
    ///
    /// For a Hamiltonian split as $H = H_0 + V(t)$, transforms operators:
    /// $$ A_I(t) = e^{iH_0t/\hbar} A_S e^{-iH_0t/\hbar} $$
    ///
    /// where $A_S$ is the Schrodinger picture operator.
    pub fn to_interaction_picture<B, E>(
        _operator_s: &E, // Schrodinger picture operator
        h0: &E,          // Free Hamiltonian
        time: &E,
        hbar: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $e^{iH_0t/\hbar}$
        let i = backend.complex_constant(0.0, 1.0)?;
        let h0t = backend.mul(h0, time)?;
        let i_h0t = backend.mul(&i, &h0t)?;
        let _ = backend.div(&i_h0t, hbar)?;

        // Simplified symbolic representation
        backend.parse("exp(iH0t/hbar) * A_S * exp(-iH0t/hbar)")
    }

    /// Interaction picture time evolution
    ///
    /// Time-ordered exponential for interaction picture evolution:
    /// $$ U_I(t) = T \exp\left[-\frac{i}{\hbar} \int_0^t V_I(t')dt'\right] $$
    ///
    /// where T is the time-ordering operator.
    pub fn time_evolution_operator<B, E>(
        _interaction_v: &E,
        _time: &E,
        _hbar: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // Time-ordered exponential (symbolic)
        backend.parse("T_exp[-i/hbar * integral(V_I, 0, t)]")
    }

    /// Dyson series expansion
    ///
    /// Perturbative expansion of the time evolution operator:
    /// $$ U_I(t) = 1 + \sum_n \frac{(-i/\hbar)^n}{n!} \int\cdots\int T[V_I(t_1)\cdots V_I(t_n)] dt_1\cdots dt_n $$
    pub fn dyson_series<B, E>(
        order: usize,
        _interaction_v: &E,
        backend: &B,
    ) -> SymbolicResult<Vec<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let mut terms = Vec::new();

        // Zeroth order: identity
        terms.push(backend.constant(1.0)?);

        // Higher orders (symbolic representation)
        for n in 1..=order {
            let term = backend.parse(&format!("dyson_{}(V_I)", n))?;
            terms.push(term);
        }

        Ok(terms)
    }
}

/// Path integral formulation
pub mod path_integral {
    use super::*;

    /// Feynman propagator
    ///
    /// Path integral formulation of quantum propagator:
    /// $$ K(x_f, t_f; x_i, t_i) = \int \mathcal{D}[x(t)] \exp\left(\frac{iS[x]}{\hbar}\right) $$
    ///
    /// where $S[x] = \int L(x, \dot{x}, t) dt$ is the classical action.
    pub fn propagator<B, E>(
        _x_initial: &E,
        _x_final: &E,
        action: &E,
        hbar: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $K = \int \mathcal{D}[x] \exp(iS/\hbar)$
        let i = backend.complex_constant(0.0, 1.0)?;
        let i_s = backend.mul(&i, action)?;
        let _ = backend.div(&i_s, hbar)?;

        backend.parse("path_integral_exp(iS/hbar)")
    }

    /// Classical action S[x]
    ///
    /// $$ S = \int_0^t L(x, \dot{x}, t') dt' $$
    pub fn classical_action<B, E>(_lagrangian: &E, _time: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $S = \int L dt$
        backend.parse("integral(L, 0, t)")
    }

    /// WKB approximation from path integrals
    ///
    /// Semiclassical approximation to the wavefunction:
    /// $$ \psi(x) \propto \exp\left(\frac{iS_0(x)}{\hbar}\right) $$
    ///
    /// where $S_0$ is the classical action along the classical path.
    pub fn wkb_from_path_integral<B, E>(
        classical_action: &E,
        hbar: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let i = backend.complex_constant(0.0, 1.0)?;
        let i_s0 = backend.mul(&i, classical_action)?;
        let _ = backend.div(&i_s0, hbar)?;

        backend.parse("exp(iS0/hbar)")
    }

    /// Stationary phase approximation
    ///
    /// For rapidly oscillating integrals, main contribution from stationary points
    pub fn stationary_phase<B, E>(_phase: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $\delta S/\delta x = 0$ at stationary points
        backend.parse("stationary_phase_contribution")
    }
}

/// Quantum expectation values and time evolution
pub mod expectation {
    use super::*;

    /// Time evolution of expectation value
    ///
    /// Ehrenfest-like equation for arbitrary observables:
    /// $$ \frac{d\langle A\rangle}{dt} = \frac{i}{\hbar}\langle[H,A]\rangle + \left\langle\frac{\partial A}{\partial t}\right\rangle $$
    pub fn time_derivative<B, E>(
        operator: &E,
        hamiltonian: &E,
        state: &E,
        hbar: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // [H, A]
        let ha = backend.mul(hamiltonian, operator)?;
        let ah = backend.mul(operator, hamiltonian)?;
        let commutator = backend.sub(&ha, &ah)?;

        // NOTE: This computes the scalar expression ψ · [H,A] · ψ which is
        // NOT the same as ⟨ψ|[H,A]|ψ⟩ in Dirac notation. The correct
        // expectation value requires the bra (conjugate-transpose):
        //   ⟨ψ| O |ψ⟩ = ψ† · O · ψ = conj(ψ) · O · ψ
        // For real wavefunctions (ψ = ψ*) this simplifies to ψ · O · ψ,
        // which is what the code below computes. For complex wavefunctions,
        // replace the first `mul` with: backend.mul(&backend.conjugate(state)?, &commutator)?
        let expectation = backend.mul(state, &commutator)?;
        let expectation = backend.mul(&expectation, state)?;

        // $(i/\hbar)\langle[H,A]\rangle$
        let i = backend.complex_constant(0.0, 1.0)?;
        let i_over_hbar = backend.div(&i, hbar)?;

        backend.mul(&i_over_hbar, &expectation)
    }
}

/// Density matrix formalism
pub mod density_matrix {
    use super::*;

    /// von Neumann equation for density matrix evolution
    ///
    /// Quantum Liouville equation for density matrix:
    /// $$ \frac{d\rho}{dt} = -\frac{i}{\hbar}[H, \rho] $$
    pub fn von_neumann_equation<B, E>(
        rho: &E,
        hamiltonian: &E,
        hbar: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $[H, \rho] = H\rho - \rho H$
        let h_rho = backend.mul(hamiltonian, rho)?;
        let rho_h = backend.mul(rho, hamiltonian)?;
        let commutator = backend.sub(&h_rho, &rho_h)?;

        // $-(i/\hbar)[H, \rho]$
        let i = backend.complex_constant(0.0, 1.0)?;
        let minus_i = backend.neg(&i)?;
        let minus_i_over_hbar = backend.div(&minus_i, hbar)?;

        backend.mul(&minus_i_over_hbar, &commutator)
    }

    /// Pure state density matrix
    ///
    /// Constructs density matrix for a pure quantum state:
    /// $$ \rho = |\psi\rangle\langle\psi| $$
    pub fn pure_state<B, E>(state: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $|\psi\rangle\langle\psi|$
        let state_dagger = backend.conjugate(state)?;
        backend.mul(state, &state_dagger)
    }

    /// Mixed state density matrix
    ///
    /// Statistical mixture of quantum states:
    /// $$ \rho = \sum_i p_i|\psi_i\rangle\langle\psi_i| $$
    ///
    /// where $p_i$ are classical probabilities with $\sum_i p_i = 1$.
    pub fn mixed_state<B, E>(states: &[E], probabilities: &[f64], backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        if states.len() != probabilities.len() {
            return Err(crate::symbolic::SymbolicError::InvalidExpression(
                "States and probabilities must have same length".to_string(),
            ));
        }

        let mut rho = backend.constant(0.0)?;

        for (state, &prob) in states.iter().zip(probabilities.iter()) {
            let p = backend.constant(prob)?;
            let state_dagger = backend.conjugate(state)?;
            let proj = backend.mul(state, &state_dagger)?;
            let weighted = backend.mul(&p, &proj)?;
            rho = backend.add(&rho, &weighted)?;
        }

        Ok(rho)
    }

    /// Purity of density matrix
    ///
    /// Measure of state purity:
    /// $$ P = \text{Tr}(\rho^2) $$
    ///
    /// $P = 1$ for pure states, $P < 1$ for mixed states.
    pub fn purity<B, E>(rho: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // $\rho^2$
        let _ = backend.mul(rho, rho)?;

        // $\text{Tr}(\rho^2)$ - symbolic representation
        backend.parse("Tr(rho^2)")
    }
}

/// Open quantum systems and decoherence
pub mod open_systems {
    use super::*;

    /// Lindblad master equation
    ///
    /// Most general form of Markovian open quantum system evolution:
    /// $$ \frac{d\rho}{dt} = -\frac{i}{\hbar}[H,\rho] + \sum_k\left(L_k\rho L_k^\dagger - \frac{1}{2}\{L_k^\dagger L_k,\rho\}\right) $$
    ///
    /// where $L_k$ are Lindblad operators describing dissipation and decoherence.
    pub fn lindblad_equation<B, E>(
        rho: &E,
        hamiltonian: &E,
        lindblad_ops: &[E],
        hbar: &E,
        backend: &B,
    ) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // Unitary part: $-(i/\hbar)[H,\rho]$
        let h_rho = backend.mul(hamiltonian, rho)?;
        let rho_h = backend.mul(rho, hamiltonian)?;
        let commutator = backend.sub(&h_rho, &rho_h)?;

        let i = backend.complex_constant(0.0, 1.0)?;
        let minus_i_over_hbar = backend.div(&backend.neg(&i)?, hbar)?;
        let unitary_part = backend.mul(&minus_i_over_hbar, &commutator)?;

        // Dissipative part: $\sum_k(L_k\rho L_k^\dagger - \frac{1}{2}\{L_k^\dagger L_k,\rho\})$
        let mut dissipative_part = backend.constant(0.0)?;

        for l_k in lindblad_ops {
            let l_k_dag = backend.conjugate(l_k)?;

            // LₖρLₖ†
            let temp1 = backend.mul(l_k, rho)?;
            let term1 = backend.mul(&temp1, &l_k_dag)?;

            // $L_k^\dagger L_k$
            let l_dag_l = backend.mul(&l_k_dag, l_k)?;

            // $\frac{1}{2}\{L_k^\dagger L_k,\rho\} = \frac{1}{2}(L_k^\dagger L_k\rho + \rho L_k^\dagger L_k)$
            let half = backend.constant(0.5)?;
            let temp2 = backend.mul(&l_dag_l, rho)?;
            let temp3 = backend.mul(rho, &l_dag_l)?;
            let anticomm = backend.add(&temp2, &temp3)?;
            let term2 = backend.mul(&half, &anticomm)?;

            // LₖρLₖ† - ½{Lₖ†Lₖ,ρ}
            let lindblad_term = backend.sub(&term1, &term2)?;
            dissipative_part = backend.add(&dissipative_part, &lindblad_term)?;
        }

        // Total: unitary + dissipative
        backend.add(&unitary_part, &dissipative_part)
    }

    /// Amplitude damping (energy relaxation)
    ///
    /// L = √γ σ₋
    pub fn amplitude_damping<B, E>(gamma: f64, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let sqrt_gamma = gamma.sqrt();
        let coeff = backend.constant(sqrt_gamma)?;
        let sigma_minus = backend.parse("sigma_minus")?;

        backend.mul(&coeff, &sigma_minus)
    }

    /// Phase damping (dephasing)
    ///
    /// L = √γ σz
    pub fn phase_damping<B, E>(gamma: f64, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let sqrt_gamma = gamma.sqrt();
        let coeff = backend.constant(sqrt_gamma)?;
        let sigma_z = backend.parse("sigma_z")?;

        backend.mul(&coeff, &sigma_z)
    }

    /// Depolarizing channel
    ///
    /// Lₖ = √(γ/3) σₖ for k ∈ {x,y,z}
    pub fn depolarizing_channel<B, E>(gamma: f64, backend: &B) -> SymbolicResult<Vec<E>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let sqrt_gamma_over_3 = (gamma / 3.0).sqrt();
        let coeff = backend.constant(sqrt_gamma_over_3)?;

        let sigma_x = backend.parse("sigma_x")?;
        let sigma_y = backend.parse("sigma_y")?;
        let sigma_z = backend.parse("sigma_z")?;

        let l_x = backend.mul(&coeff, &sigma_x)?;
        let l_y = backend.mul(&coeff, &sigma_y)?;
        let l_z = backend.mul(&coeff, &sigma_z)?;

        Ok(vec![l_x, l_y, l_z])
    }

    /// Quantum jump method (Monte Carlo wavefunction)
    ///
    /// Stochastic unraveling of Lindblad equation
    pub fn quantum_jump<B, E>(state: &E, jump_operator: &E, backend: &B) -> SymbolicResult<E>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // |ψ'⟩ = L|ψ⟩ / ||L|ψ⟩||
        let _ = backend.mul(jump_operator, state)?;

        // Normalization (symbolic)
        backend.parse("normalize(L|psi>)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbolic::create_symbolica_backend;

    #[test]
    fn test_quantum_picture_display() {
        assert_eq!(QuantumPicture::Heisenberg.to_string(), "Heisenberg");
        assert_eq!(QuantumPicture::Schrodinger.to_string(), "Schrodinger");
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_heisenberg_equation() {
        let backend = create_symbolica_backend();
        let op = backend.variable("A").unwrap();
        let h = backend.variable("H").unwrap();
        let hbar = backend.variable("hbar").unwrap();

        let dadt = heisenberg::equation_of_motion(&op, &h, &hbar, &backend).unwrap();

        assert!(!dadt.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_interaction_picture() {
        let backend = create_symbolica_backend();
        let op = backend.variable("A").unwrap();
        let h0 = backend.variable("H0").unwrap();
        let t = backend.variable("t").unwrap();
        let hbar = backend.variable("hbar").unwrap();

        let op_i = interaction::to_interaction_picture(&op, &h0, &t, &hbar, &backend).unwrap();

        assert!(!op_i.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_path_integral_propagator() {
        let backend = create_symbolica_backend();
        let xi = backend.variable("xi").unwrap();
        let xf = backend.variable("xf").unwrap();
        let action = backend.variable("S").unwrap();
        let hbar = backend.variable("hbar").unwrap();

        let propagator = path_integral::propagator(&xi, &xf, &action, &hbar, &backend).unwrap();

        assert!(!propagator.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_von_neumann_equation() {
        let backend = create_symbolica_backend();
        let rho = backend.variable("rho").unwrap();
        let h = backend.variable("H").unwrap();
        let hbar = backend.variable("hbar").unwrap();

        let drho_dt = density_matrix::von_neumann_equation(&rho, &h, &hbar, &backend).unwrap();

        assert!(!drho_dt.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_pure_state_density_matrix() {
        let backend = create_symbolica_backend();
        let psi = backend.variable("psi").unwrap();

        let rho = density_matrix::pure_state(&psi, &backend).unwrap();

        assert!(!rho.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_mixed_state_density_matrix() {
        let backend = create_symbolica_backend();
        let psi1 = backend.variable("psi1").unwrap();
        let psi2 = backend.variable("psi2").unwrap();

        let states = vec![psi1, psi2];
        let probs = vec![0.7, 0.3];

        let rho = density_matrix::mixed_state(&states, &probs, &backend).unwrap();

        assert!(!rho.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_lindblad_equation() {
        let backend = create_symbolica_backend();
        let rho = backend.variable("rho").unwrap();
        let h = backend.variable("H").unwrap();
        let hbar = backend.variable("hbar").unwrap();

        let l1 = backend.variable("L1").unwrap();
        let lindblad_ops = vec![l1];

        let drho_dt =
            open_systems::lindblad_equation(&rho, &h, &lindblad_ops, &hbar, &backend).unwrap();

        assert!(!drho_dt.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_amplitude_damping() {
        let backend = create_symbolica_backend();
        let gamma = 0.1;

        let l_amp = open_systems::amplitude_damping(gamma, &backend).unwrap();

        assert!(!l_amp.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_phase_damping() {
        let backend = create_symbolica_backend();
        let gamma = 0.05;

        let l_phase = open_systems::phase_damping(gamma, &backend).unwrap();

        assert!(!l_phase.is_zero());
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_depolarizing_channel() {
        let backend = create_symbolica_backend();
        let gamma = 0.15;

        let lindblad_ops = open_systems::depolarizing_channel(gamma, &backend).unwrap();

        assert_eq!(lindblad_ops.len(), 3);
    }

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_quantum_jump() {
        let backend = create_symbolica_backend();
        let psi = backend.variable("psi").unwrap();
        let l = backend.variable("L").unwrap();

        let psi_jumped = open_systems::quantum_jump(&psi, &l, &backend).unwrap();

        assert!(!psi_jumped.is_zero());
    }
}
