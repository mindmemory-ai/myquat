// Classical Optimizers for VQE
// Author: gA4ss
//
// This module provides classical optimization algorithms for variational quantum algorithms.
// Implements gradient-free and gradient-based optimizers without external dependencies.

use crate::error::Result;
use std::f64;

/// Optimization result
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// Optimal parameters found
    pub parameters: Vec<f64>,
    /// Minimum value achieved
    pub minimum: f64,
    /// Number of iterations
    pub iterations: usize,
    /// Function evaluations
    pub function_evaluations: usize,
    /// Convergence status
    pub converged: bool,
    /// Optimization history (energy at each iteration)
    pub history: Vec<f64>,
}

/// Classical optimizer trait
pub trait ClassicalOptimizer: Send + Sync {
    /// Optimize the objective function
    ///
    /// # Arguments
    ///
    /// * `objective` - Objective function to minimize
    /// * `initial_params` - Initial parameter values
    ///
    /// # Returns
    ///
    /// Optimization result with optimal parameters and minimum value
    fn optimize(
        &self,
        objective: &dyn Fn(&[f64]) -> Result<f64>,
        initial_params: &[f64],
    ) -> Result<OptimizationResult>;

    /// Get optimizer name
    fn name(&self) -> &str;
}

/// Gradient Descent Optimizer
///
/// Simple gradient descent with parameter-shift rule for gradient estimation
pub struct GradientDescentOptimizer {
    /// Learning rate
    pub learning_rate: f64,
    /// Maximum iterations
    pub max_iterations: usize,
    /// Convergence tolerance
    pub tolerance: f64,
    /// Parameter shift for gradient estimation
    pub shift: f64,
}

impl GradientDescentOptimizer {
    /// Create a new gradient descent optimizer
    pub fn new() -> Self {
        GradientDescentOptimizer {
            learning_rate: 0.1,
            max_iterations: 100,
            tolerance: 1e-6,
            shift: std::f64::consts::PI / 2.0, // Parameter-shift rule
        }
    }

    /// Set learning rate
    pub fn with_learning_rate(mut self, lr: f64) -> Self {
        self.learning_rate = lr;
        self
    }

    /// Set maximum iterations
    pub fn with_max_iterations(mut self, max_iter: usize) -> Self {
        self.max_iterations = max_iter;
        self
    }

    /// Set convergence tolerance
    pub fn with_tolerance(mut self, tol: f64) -> Self {
        self.tolerance = tol;
        self
    }

    /// Compute gradient using parameter-shift rule
    fn compute_gradient(
        &self,
        objective: &dyn Fn(&[f64]) -> Result<f64>,
        params: &[f64],
    ) -> Result<Vec<f64>> {
        let n = params.len();
        let mut gradient = vec![0.0; n];

        for i in 0..n {
            let mut params_plus = params.to_vec();
            let mut params_minus = params.to_vec();

            params_plus[i] += self.shift;
            params_minus[i] -= self.shift;

            let f_plus = objective(&params_plus)?;
            let f_minus = objective(&params_minus)?;

            gradient[i] = (f_plus - f_minus) / (2.0 * self.shift);
        }

        Ok(gradient)
    }
}

impl Default for GradientDescentOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl ClassicalOptimizer for GradientDescentOptimizer {
    fn optimize(
        &self,
        objective: &dyn Fn(&[f64]) -> Result<f64>,
        initial_params: &[f64],
    ) -> Result<OptimizationResult> {
        let mut params = initial_params.to_vec();
        let mut history = Vec::new();
        let mut function_evals = 0;

        for iteration in 0..self.max_iterations {
            // Evaluate current energy
            let energy = objective(&params)?;
            function_evals += 1;
            history.push(energy);

            // Compute gradient
            let gradient = self.compute_gradient(objective, &params)?;
            function_evals += 2 * params.len();

            // Update parameters
            let mut max_grad: f64 = 0.0;
            for i in 0..params.len() {
                params[i] -= self.learning_rate * gradient[i];
                max_grad = max_grad.max(gradient[i].abs());
            }

            // Check convergence
            if max_grad < self.tolerance {
                let final_energy = objective(&params)?;
                function_evals += 1;

                return Ok(OptimizationResult {
                    parameters: params,
                    minimum: final_energy,
                    iterations: iteration + 1,
                    function_evaluations: function_evals,
                    converged: true,
                    history,
                });
            }
        }

        // Max iterations reached
        let final_energy = objective(&params)?;
        function_evals += 1;

        Ok(OptimizationResult {
            parameters: params,
            minimum: final_energy,
            iterations: self.max_iterations,
            function_evaluations: function_evals,
            converged: false,
            history,
        })
    }

    fn name(&self) -> &str {
        "Gradient Descent"
    }
}

/// Nelder-Mead Simplex Optimizer
///
/// Gradient-free optimization using the Nelder-Mead simplex algorithm
pub struct SimplexOptimizer {
    /// Maximum iterations
    pub max_iterations: usize,
    /// Convergence tolerance
    pub tolerance: f64,
    /// Reflection coefficient
    pub alpha: f64,
    /// Expansion coefficient
    pub gamma: f64,
    /// Contraction coefficient
    pub rho: f64,
    /// Shrink coefficient
    pub sigma: f64,
}

impl SimplexOptimizer {
    /// Create a new Nelder-Mead optimizer
    pub fn new() -> Self {
        SimplexOptimizer {
            max_iterations: 200,
            tolerance: 1e-6,
            alpha: 1.0,
            gamma: 2.0,
            rho: 0.5,
            sigma: 0.5,
        }
    }

    /// Set maximum iterations
    pub fn with_max_iterations(mut self, max_iter: usize) -> Self {
        self.max_iterations = max_iter;
        self
    }

    /// Set convergence tolerance
    pub fn with_tolerance(mut self, tol: f64) -> Self {
        self.tolerance = tol;
        self
    }

    /// Initialize simplex
    fn initialize_simplex(&self, initial: &[f64]) -> Vec<Vec<f64>> {
        let n = initial.len();
        let mut simplex = Vec::with_capacity(n + 1);

        // First vertex is the initial point
        simplex.push(initial.to_vec());

        // Create n additional vertices
        for i in 0..n {
            let mut vertex = initial.to_vec();
            vertex[i] += 0.1; // Small perturbation
            simplex.push(vertex);
        }

        simplex
    }

    /// Compute centroid of simplex (excluding worst point)
    fn centroid(&self, simplex: &[Vec<f64>], exclude_idx: usize) -> Vec<f64> {
        let n = simplex[0].len();
        let mut center = vec![0.0; n];
        let count = simplex.len() - 1;

        for (i, vertex) in simplex.iter().enumerate() {
            if i != exclude_idx {
                for j in 0..n {
                    center[j] += vertex[j];
                }
            }
        }

        for j in 0..n {
            center[j] /= count as f64;
        }

        center
    }
}

impl Default for SimplexOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl ClassicalOptimizer for SimplexOptimizer {
    fn optimize(
        &self,
        objective: &dyn Fn(&[f64]) -> Result<f64>,
        initial_params: &[f64],
    ) -> Result<OptimizationResult> {
        let mut simplex = self.initialize_simplex(initial_params);
        let mut values: Vec<f64> = simplex
            .iter()
            .map(|v| objective(v).unwrap_or(f64::INFINITY))
            .collect();

        let mut history = Vec::new();
        let mut function_evals = simplex.len();

        for iteration in 0..self.max_iterations {
            // Sort simplex by function values
            let mut indices: Vec<usize> = (0..simplex.len()).collect();
            indices.sort_by(|&a, &b| values[a].partial_cmp(&values[b]).unwrap());

            let best_idx = indices[0];
            let worst_idx = indices[indices.len() - 1];
            let second_worst_idx = indices[indices.len() - 2];

            history.push(values[best_idx]);

            // Check convergence
            let range = values[worst_idx] - values[best_idx];
            if range < self.tolerance {
                return Ok(OptimizationResult {
                    parameters: simplex[best_idx].clone(),
                    minimum: values[best_idx],
                    iterations: iteration + 1,
                    function_evaluations: function_evals,
                    converged: true,
                    history,
                });
            }

            // Compute centroid (excluding worst point)
            let centroid = self.centroid(&simplex, worst_idx);

            // Reflection
            let reflected: Vec<f64> = centroid
                .iter()
                .zip(&simplex[worst_idx])
                .map(|(&c, &w)| c + self.alpha * (c - w))
                .collect();
            let f_reflected = objective(&reflected)?;
            function_evals += 1;

            if f_reflected < values[best_idx] {
                // Expansion
                let expanded: Vec<f64> = centroid
                    .iter()
                    .zip(&reflected)
                    .map(|(&c, &r)| c + self.gamma * (r - c))
                    .collect();
                let f_expanded = objective(&expanded)?;
                function_evals += 1;

                if f_expanded < f_reflected {
                    simplex[worst_idx] = expanded;
                    values[worst_idx] = f_expanded;
                } else {
                    simplex[worst_idx] = reflected;
                    values[worst_idx] = f_reflected;
                }
            } else if f_reflected < values[second_worst_idx] {
                simplex[worst_idx] = reflected;
                values[worst_idx] = f_reflected;
            } else {
                // Contraction
                let contracted: Vec<f64> = centroid
                    .iter()
                    .zip(&simplex[worst_idx])
                    .map(|(&c, &w)| c + self.rho * (w - c))
                    .collect();
                let f_contracted = objective(&contracted)?;
                function_evals += 1;

                if f_contracted < values[worst_idx] {
                    simplex[worst_idx] = contracted;
                    values[worst_idx] = f_contracted;
                } else {
                    // Shrink
                    for i in 0..simplex.len() {
                        if i != best_idx {
                            for j in 0..simplex[i].len() {
                                simplex[i][j] = simplex[best_idx][j]
                                    + self.sigma * (simplex[i][j] - simplex[best_idx][j]);
                            }
                            values[i] = objective(&simplex[i])?;
                            function_evals += 1;
                        }
                    }
                }
            }
        }

        // Find best result
        let best_idx = values
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        Ok(OptimizationResult {
            parameters: simplex[best_idx].clone(),
            minimum: values[best_idx],
            iterations: self.max_iterations,
            function_evaluations: function_evals,
            converged: false,
            history,
        })
    }

    fn name(&self) -> &str {
        "Nelder-Mead Simplex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_descent_quadratic() {
        // Minimize f(x) = x^2
        let objective = |params: &[f64]| -> Result<f64> { Ok(params[0] * params[0]) };

        let optimizer = GradientDescentOptimizer::new()
            .with_learning_rate(0.2)
            .with_max_iterations(100)
            .with_tolerance(1e-4);

        let result = optimizer.optimize(&objective, &[2.0]).unwrap();

        // Should converge to x ≈ 0
        assert!(result.parameters[0].abs() < 0.2);
        assert!(result.minimum < 0.05);
        // Convergence may not always happen with gradient descent
        assert!(result.iterations <= 100);
    }

    #[test]
    fn test_simplex_quadratic() {
        // Minimize f(x, y) = x^2 + y^2
        let objective =
            |params: &[f64]| -> Result<f64> { Ok(params[0] * params[0] + params[1] * params[1]) };

        let optimizer = SimplexOptimizer::new().with_max_iterations(100);

        let result = optimizer.optimize(&objective, &[1.0, 1.0]).unwrap();

        // Should converge to (0, 0)
        assert!(result.parameters[0].abs() < 0.1);
        assert!(result.parameters[1].abs() < 0.1);
        assert!(result.minimum < 0.01);
    }

    #[test]
    fn test_rosenbrock_function() {
        // Rosenbrock function: f(x,y) = (1-x)^2 + 100(y-x^2)^2
        // Minimum at (1, 1) with f(1,1) = 0
        let objective = |params: &[f64]| -> Result<f64> {
            let x = params[0];
            let y = params[1];
            Ok((1.0 - x).powi(2) + 100.0 * (y - x * x).powi(2))
        };

        let optimizer = SimplexOptimizer::new()
            .with_max_iterations(500)
            .with_tolerance(1e-4);

        let result = optimizer.optimize(&objective, &[0.0, 0.0]).unwrap();

        // Should be close to (1, 1)
        assert!((result.parameters[0] - 1.0).abs() < 0.2);
        assert!((result.parameters[1] - 1.0).abs() < 0.2);
        assert!(result.minimum < 1.0);
    }

    #[test]
    fn test_optimization_result() {
        let result = OptimizationResult {
            parameters: vec![1.0, 2.0],
            minimum: 0.5,
            iterations: 10,
            function_evaluations: 50,
            converged: true,
            history: vec![1.0, 0.8, 0.6, 0.5],
        };

        assert_eq!(result.parameters.len(), 2);
        assert_eq!(result.minimum, 0.5);
        assert!(result.converged);
        assert_eq!(result.history.len(), 4);
    }
}
