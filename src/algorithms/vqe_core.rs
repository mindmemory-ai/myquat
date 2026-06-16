//! Variational Quantum Eigensolver (VQE) - Core Algorithm
//! Author: gA4ss
//!
//! Complete VQE implementation with:
//! - Energy evaluation
//! - Classical optimizer integration
//! - Gradient computation (parameter-shift rule)
//! - Convergence monitoring

use crate::error::{MyQuatError, Result};
use crate::hamiltonian::{Hamiltonian, PauliOperator, PauliString};
use crate::QuantumCircuit;
use num_complex::Complex64;

/// Apply Pauli string to a basis state index
/// Returns (resulting index, phase factor)
fn apply_pauli_string(index: usize, paulis: &[PauliOperator]) -> (usize, Complex64) {
    let mut result_index = index;
    let mut phase = Complex64::new(1.0, 0.0);

    for (qubit, pauli) in paulis.iter().enumerate() {
        let bit = (index >> qubit) & 1;

        match pauli {
            PauliOperator::I => {}
            PauliOperator::X => {
                // X flips the bit
                result_index ^= 1 << qubit;
            }
            PauliOperator::Y => {
                // Y flips the bit and adds phase
                result_index ^= 1 << qubit;
                phase *= if bit == 0 {
                    Complex64::new(0.0, 1.0) // i
                } else {
                    Complex64::new(0.0, -1.0) // -i
                };
            }
            PauliOperator::Z => {
                // Z adds phase based on bit value
                if bit == 1 {
                    phase *= Complex64::new(-1.0, 0.0);
                }
            }
        }
    }

    (result_index, phase)
}

/// Simplified Pauli expectation value computation
fn compute_pauli_expectation_simple(state: &[Complex64], pauli_string: &PauliString) -> Complex64 {
    let n = pauli_string.num_qubits();
    let dim = 1 << n;
    let mut result = Complex64::new(0.0, 0.0);

    for i in 0..dim {
        let (j, phase) = apply_pauli_string(i, &pauli_string.operators);
        result += state[i].conj() * phase * state[j];
    }

    result
}

/// Simple circuit simulation (returns state vector)
fn simulate_circuit_simple(circuit: &QuantumCircuit) -> Result<Vec<Complex64>> {
    use crate::gates::StandardGate;

    let n_qubits = circuit.num_qubits();
    let dim = 1 << n_qubits;

    // Initialize to |0...0>
    let mut state = vec![Complex64::new(0.0, 0.0); dim];
    state[0] = Complex64::new(1.0, 0.0);

    // Apply each gate
    for inst in circuit.data().instructions() {
        let gate_type = inst.gate.gate_type;
        let qubits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();

        match gate_type {
            StandardGate::H => {
                apply_single_qubit_gate(&mut state, qubits[0], n_qubits, |a, b| {
                    let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
                    (
                        Complex64::new(inv_sqrt2, 0.0) * (a + b),
                        Complex64::new(inv_sqrt2, 0.0) * (a - b),
                    )
                });
            }
            StandardGate::X => {
                apply_single_qubit_gate(&mut state, qubits[0], n_qubits, |a, b| (b, a));
            }
            StandardGate::Y => {
                apply_single_qubit_gate(&mut state, qubits[0], n_qubits, |a, b| {
                    (Complex64::new(0.0, 1.0) * b, Complex64::new(0.0, -1.0) * a)
                });
            }
            StandardGate::Z => {
                apply_single_qubit_gate(&mut state, qubits[0], n_qubits, |a, b| (a, -b));
            }
            StandardGate::S => {
                apply_single_qubit_gate(&mut state, qubits[0], n_qubits, |a, b| {
                    (a, Complex64::new(0.0, 1.0) * b)
                });
            }
            StandardGate::T => {
                let phase = Complex64::new(1.0 / 2.0_f64.sqrt(), 1.0 / 2.0_f64.sqrt());
                apply_single_qubit_gate(&mut state, qubits[0], n_qubits, |a, b| (a, phase * b));
            }
            StandardGate::Rx | StandardGate::Ry | StandardGate::Rz => {
                if let Some(param) = inst.gate.parameters.first() {
                    let theta = param.numeric_value().unwrap_or(0.0);
                    match gate_type {
                        StandardGate::Rx => {
                            let cos = (theta / 2.0).cos();
                            let sin = (theta / 2.0).sin();
                            apply_single_qubit_gate(&mut state, qubits[0], n_qubits, |a, b| {
                                (
                                    Complex64::new(cos, 0.0) * a - Complex64::new(0.0, sin) * b,
                                    Complex64::new(cos, 0.0) * b - Complex64::new(0.0, sin) * a,
                                )
                            });
                        }
                        StandardGate::Ry => {
                            let cos = (theta / 2.0).cos();
                            let sin = (theta / 2.0).sin();
                            apply_single_qubit_gate(&mut state, qubits[0], n_qubits, |a, b| {
                                (
                                    Complex64::new(cos, 0.0) * a - Complex64::new(sin, 0.0) * b,
                                    Complex64::new(sin, 0.0) * a + Complex64::new(cos, 0.0) * b,
                                )
                            });
                        }
                        StandardGate::Rz => {
                            let phase_minus = Complex64::new(0.0, -theta / 2.0).exp();
                            let phase_plus = Complex64::new(0.0, theta / 2.0).exp();
                            apply_single_qubit_gate(&mut state, qubits[0], n_qubits, |a, b| {
                                (phase_minus * a, phase_plus * b)
                            });
                        }
                        _ => unreachable!(),
                    }
                }
            }
            StandardGate::CX => {
                apply_cx_gate(&mut state, qubits[0], qubits[1], n_qubits);
            }
            _ => {
                // Skip unsupported gates for now
            }
        }
    }

    Ok(state)
}

/// Apply single-qubit gate
fn apply_single_qubit_gate<F>(state: &mut [Complex64], qubit: usize, n_qubits: usize, gate_fn: F)
where
    F: Fn(Complex64, Complex64) -> (Complex64, Complex64),
{
    let dim = 1 << n_qubits;
    let stride = 1 << qubit;

    for i in 0..dim {
        if (i & stride) == 0 {
            let i0 = i;
            let i1 = i | stride;
            let (new_a, new_b) = gate_fn(state[i0], state[i1]);
            state[i0] = new_a;
            state[i1] = new_b;
        }
    }
}

/// Apply CNOT gate
fn apply_cx_gate(state: &mut [Complex64], control: usize, target: usize, n_qubits: usize) {
    let dim = 1 << n_qubits;
    let control_mask = 1 << control;
    let target_mask = 1 << target;

    for i in 0..dim {
        if (i & control_mask) != 0 && (i & target_mask) == 0 {
            let j = i | target_mask;
            state.swap(i, j);
        }
    }
}

/// VQE optimization result
#[derive(Debug, Clone)]
pub struct VQEResult {
    /// Optimal parameters found
    pub optimal_parameters: Vec<f64>,

    /// Ground state energy
    pub ground_state_energy: f64,

    /// Number of iterations
    pub num_iterations: usize,

    /// Energy history during optimization
    pub energy_history: Vec<f64>,

    /// Parameter history (optional, for visualization)
    pub parameter_history: Vec<Vec<f64>>,

    /// Convergence achieved
    pub converged: bool,

    /// Final gradient norm
    pub final_gradient_norm: f64,
}

impl VQEResult {
    /// Create a new VQE result
    pub fn new(
        optimal_parameters: Vec<f64>,
        ground_state_energy: f64,
        num_iterations: usize,
        energy_history: Vec<f64>,
        converged: bool,
    ) -> Self {
        Self {
            optimal_parameters,
            ground_state_energy,
            num_iterations,
            energy_history,
            parameter_history: Vec::new(),
            converged,
            final_gradient_norm: 0.0,
        }
    }

    /// Get energy change from first to last iteration
    pub fn energy_improvement(&self) -> f64 {
        if self.energy_history.len() < 2 {
            return 0.0;
        }
        self.energy_history[0] - self.energy_history[self.energy_history.len() - 1]
    }
}

/// Classical optimizer interface
pub trait ClassicalOptimizer {
    /// Minimize the objective function
    fn minimize(
        &mut self,
        initial_params: &[f64],
        objective: &mut dyn FnMut(&[f64]) -> f64,
        gradient: Option<&mut dyn FnMut(&[f64]) -> Vec<f64>>,
    ) -> Result<Vec<f64>>;

    /// Get optimizer name
    fn name(&self) -> &str;
}

/// Simple gradient descent optimizer
pub struct GradientDescentOptimizer {
    max_iterations: usize,
    learning_rate: f64,
    tolerance: f64,
}

impl GradientDescentOptimizer {
    /// Create a new gradient descent optimizer
    pub fn new(max_iterations: usize, learning_rate: f64, tolerance: f64) -> Self {
        Self {
            max_iterations,
            learning_rate,
            tolerance,
        }
    }
}

impl ClassicalOptimizer for GradientDescentOptimizer {
    fn minimize(
        &mut self,
        initial_params: &[f64],
        _objective: &mut dyn FnMut(&[f64]) -> f64,
        gradient: Option<&mut dyn FnMut(&[f64]) -> Vec<f64>>,
    ) -> Result<Vec<f64>> {
        if gradient.is_none() {
            return Err(MyQuatError::circuit_error(
                "Gradient descent requires gradient function",
            ));
        }

        let gradient_fn = gradient.unwrap();
        let mut params = initial_params.to_vec();

        for iter in 0..self.max_iterations {
            let grad = gradient_fn(&params);
            let grad_norm: f64 = grad.iter().map(|&g| g * g).sum::<f64>().sqrt();

            if grad_norm < self.tolerance {
                println!("Converged after {} iterations", iter);
                break;
            }

            // Update parameters
            for (p, g) in params.iter_mut().zip(grad.iter()) {
                *p -= self.learning_rate * g;
            }
        }

        Ok(params)
    }

    fn name(&self) -> &str {
        "GradientDescent"
    }
}

/// Simple COBYLA-like optimizer (Nelder-Mead simplex)
pub struct SimplexOptimizer {
    max_iterations: usize,
    tolerance: f64,
}

impl SimplexOptimizer {
    /// Create a new simplex optimizer
    pub fn new(max_iterations: usize, tolerance: f64) -> Self {
        Self {
            max_iterations,
            tolerance,
        }
    }
}

impl ClassicalOptimizer for SimplexOptimizer {
    fn minimize(
        &mut self,
        initial_params: &[f64],
        objective: &mut dyn FnMut(&[f64]) -> f64,
        _gradient: Option<&mut dyn FnMut(&[f64]) -> Vec<f64>>,
    ) -> Result<Vec<f64>> {
        // Simplified Nelder-Mead implementation
        let n = initial_params.len();
        let mut simplex: Vec<Vec<f64>> = Vec::new();

        // Initialize simplex
        simplex.push(initial_params.to_vec());
        for i in 0..n {
            let mut point = initial_params.to_vec();
            point[i] += 0.1;
            simplex.push(point);
        }

        // Evaluate initial simplex
        let mut values: Vec<f64> = simplex.iter().map(|p| objective(p)).collect();

        for _iter in 0..self.max_iterations {
            // Sort by function value
            let mut indices: Vec<usize> = (0..simplex.len()).collect();
            indices.sort_by(|&i, &j| values[i].partial_cmp(&values[j]).unwrap());

            // Check convergence
            let best = values[indices[0]];
            let worst = values[indices[n]];
            if (worst - best).abs() < self.tolerance {
                break;
            }

            // Reflection
            let worst_idx = indices[n];
            let centroid: Vec<f64> = (0..n)
                .map(|dim| indices[..n].iter().map(|&i| simplex[i][dim]).sum::<f64>() / n as f64)
                .collect();

            let reflected: Vec<f64> = centroid
                .iter()
                .zip(simplex[worst_idx].iter())
                .map(|(&c, &w)| 2.0 * c - w)
                .collect();

            let reflected_value = objective(&reflected);

            if reflected_value < values[indices[0]] {
                // Expansion
                let expanded: Vec<f64> = centroid
                    .iter()
                    .zip(reflected.iter())
                    .map(|(&c, &r)| c + 2.0 * (r - c))
                    .collect();

                let expanded_value = objective(&expanded);
                if expanded_value < reflected_value {
                    simplex[worst_idx] = expanded;
                    values[worst_idx] = expanded_value;
                } else {
                    simplex[worst_idx] = reflected;
                    values[worst_idx] = reflected_value;
                }
            } else if reflected_value < values[indices[n - 1]] {
                simplex[worst_idx] = reflected;
                values[worst_idx] = reflected_value;
            } else {
                // Contraction
                let contracted: Vec<f64> = centroid
                    .iter()
                    .zip(simplex[worst_idx].iter())
                    .map(|(&c, &w)| (c + w) / 2.0)
                    .collect();

                simplex[worst_idx] = contracted;
                values[worst_idx] = objective(&simplex[worst_idx]);
            }
        }

        // Return best point
        let best_idx = values
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap();

        Ok(simplex[best_idx].clone())
    }

    fn name(&self) -> &str {
        "SimplexOptimizer"
    }
}

/// VQE algorithm configuration
pub struct VQEConfig {
    /// Maximum number of iterations
    pub max_iterations: usize,

    /// Energy convergence tolerance
    pub energy_tolerance: f64,

    /// Gradient convergence tolerance
    pub gradient_tolerance: f64,

    /// Whether to compute gradients
    pub use_gradients: bool,

    /// Store parameter history for visualization
    pub store_history: bool,

    /// Parameter shift for gradient computation
    pub parameter_shift: f64,
}

impl Default for VQEConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            energy_tolerance: 1e-6,
            gradient_tolerance: 1e-5,
            use_gradients: false,
            store_history: true,
            parameter_shift: std::f64::consts::PI / 2.0,
        }
    }
}

/// Variational Quantum Eigensolver
pub struct VQE {
    /// Configuration
    config: VQEConfig,

    /// Classical optimizer
    optimizer: Box<dyn ClassicalOptimizer>,

    /// Energy evaluation count
    eval_count: usize,
}

impl VQE {
    /// Create a new VQE instance
    pub fn new(optimizer: Box<dyn ClassicalOptimizer>, config: VQEConfig) -> Self {
        Self {
            config,
            optimizer,
            eval_count: 0,
        }
    }

    /// Create VQE with default configuration
    pub fn with_default_optimizer() -> Self {
        let optimizer = Box::new(SimplexOptimizer::new(100, 1e-6));
        Self::new(optimizer, VQEConfig::default())
    }

    /// Compute expectation value of Hamiltonian
    pub fn compute_expectation(
        &mut self,
        circuit: &QuantumCircuit,
        hamiltonian: &Hamiltonian,
    ) -> Result<f64> {
        self.eval_count += 1;

        // Simple state vector simulation
        let state = simulate_circuit_simple(circuit)?;

        // Compute <ψ|H|ψ>
        let mut energy = hamiltonian.constant_term;
        for term in &hamiltonian.terms {
            let pauli_exp = compute_pauli_expectation_simple(&state, &term.pauli_string);
            energy += (term.coefficient * pauli_exp).re;
        }

        Ok(energy.re)
    }

    /// Compute gradient using parameter-shift rule
    pub fn compute_gradient(
        &mut self,
        ansatz_builder: &dyn Fn(&[f64]) -> Result<QuantumCircuit>,
        hamiltonian: &Hamiltonian,
        parameters: &[f64],
    ) -> Result<Vec<f64>> {
        let shift = self.config.parameter_shift;
        let mut gradient = Vec::with_capacity(parameters.len());

        for i in 0..parameters.len() {
            let mut params_plus = parameters.to_vec();
            params_plus[i] += shift;
            let circuit_plus = ansatz_builder(&params_plus)?;
            let energy_plus = self.compute_expectation(&circuit_plus, hamiltonian)?;

            let mut params_minus = parameters.to_vec();
            params_minus[i] -= shift;
            let circuit_minus = ansatz_builder(&params_minus)?;
            let energy_minus = self.compute_expectation(&circuit_minus, hamiltonian)?;

            // Parameter-shift gradient: ∂E/∂θ = (E(θ+π/2) - E(θ-π/2)) / 2
            let grad = (energy_plus - energy_minus) / 2.0;
            gradient.push(grad);
        }

        Ok(gradient)
    }

    /// Run VQE optimization
    pub fn run(
        &mut self,
        ansatz_builder: impl Fn(&[f64]) -> Result<QuantumCircuit>,
        hamiltonian: &Hamiltonian,
        initial_parameters: &[f64],
    ) -> Result<VQEResult> {
        let mut energy_history = Vec::new();
        let mut parameter_history = Vec::new();
        let store_history = self.config.store_history;

        self.eval_count = 0;

        // Clone hamiltonian for closure
        let hamiltonian_clone = hamiltonian.clone();

        // Objective function: compute energy
        let mut objective = |params: &[f64]| -> f64 {
            let circuit = ansatz_builder(params).unwrap();
            let state = simulate_circuit_simple(&circuit).unwrap();

            // Compute energy
            let mut energy = hamiltonian_clone.constant_term;
            for term in &hamiltonian_clone.terms {
                let pauli_exp = compute_pauli_expectation_simple(&state, &term.pauli_string);
                energy += (term.coefficient * pauli_exp).re;
            }

            if store_history {
                energy_history.push(energy.re);
                parameter_history.push(params.to_vec());
            }

            energy.re
        };

        // Run optimization (without gradients for now)
        let optimal_parameters =
            self.optimizer
                .minimize(initial_parameters, &mut objective, None)?;

        // Evaluate final energy
        let final_circuit = ansatz_builder(&optimal_parameters)?;
        let ground_state_energy = self.compute_expectation(&final_circuit, hamiltonian)?;

        // Check convergence
        let converged = if energy_history.len() >= 2 {
            let last_energy = energy_history[energy_history.len() - 1];
            (ground_state_energy - last_energy).abs() < self.config.energy_tolerance
        } else {
            true
        };

        let mut result = VQEResult::new(
            optimal_parameters,
            ground_state_energy,
            energy_history.len(),
            energy_history,
            converged,
        );

        if self.config.store_history {
            result.parameter_history = parameter_history;
        }

        Ok(result)
    }

    /// Get number of energy evaluations performed
    pub fn evaluation_count(&self) -> usize {
        self.eval_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hamiltonian::PauliString;

    #[test]
    fn test_vqe_config_default() {
        let config = VQEConfig::default();
        assert_eq!(config.max_iterations, 100);
        assert!((config.energy_tolerance - 1e-6).abs() < 1e-10);
    }

    #[test]
    fn test_gradient_descent_optimizer() {
        let mut optimizer = GradientDescentOptimizer::new(100, 0.1, 1e-5);

        // Simple quadratic function: f(x) = x^2
        let mut objective = |params: &[f64]| -> f64 { params[0] * params[0] };

        let mut gradient_fn = |params: &[f64]| -> Vec<f64> { vec![2.0 * params[0]] };

        let result = optimizer
            .minimize(&[1.0], &mut objective, Some(&mut gradient_fn))
            .unwrap();

        assert!((result[0]).abs() < 0.1); // Should converge near 0
    }

    #[test]
    fn test_simplex_optimizer() {
        let mut optimizer = SimplexOptimizer::new(100, 1e-5);

        // Simple quadratic function
        let mut objective = |params: &[f64]| -> f64 { params.iter().map(|&x| x * x).sum() };

        let result = optimizer
            .minimize(&[1.0, 1.0], &mut objective, None)
            .unwrap();

        assert!(result[0].abs() < 0.1);
        assert!(result[1].abs() < 0.1);
    }
}
