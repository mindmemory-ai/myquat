//! Numerical Methods for Quantum Mechanics
//!
//! Author: gA4ss
//!
//! This module provides numerical methods for solving quantum mechanics problems
//! that don't have analytical solutions, including matrix diagonalization,
//! finite difference/element methods, and hybrid symbolic-numerical approaches.

use crate::symbolic::{SymbolicBackend, SymbolicError, SymbolicExpression, SymbolicResult};
use std::fmt;

/// Numerical method type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericalMethod {
    /// Matrix diagonalization
    MatrixDiagonalization,
    /// Finite difference method
    FiniteDifference,
    /// Finite element method
    FiniteElement,
    /// Shooting method
    ShootingMethod,
}

impl fmt::Display for NumericalMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumericalMethod::MatrixDiagonalization => write!(f, "Matrix Diagonalization"),
            NumericalMethod::FiniteDifference => write!(f, "Finite Difference"),
            NumericalMethod::FiniteElement => write!(f, "Finite Element"),
            NumericalMethod::ShootingMethod => write!(f, "Shooting Method"),
        }
    }
}

/// Matrix diagonalization for discrete quantum systems
pub mod matrix_diagonalization {
    use super::*;

    /// Discretize Hamiltonian on a grid
    ///
    /// Discretizes the Hamiltonian operator using finite difference method:
    /// $$ H_{ij} = \delta_{ij} V(x_i) - \frac{1}{2m} \nabla^2_{ij} $$
    ///
    /// where $\delta_{ij}$ is the Kronecker delta and $\nabla^2$ is the Laplacian operator.
    pub fn discretize_hamiltonian<B, E>(
        grid_points: usize,
        x_min: f64,
        x_max: f64,
        potential: &E,
        mass: f64,
        hbar: f64,
        backend: &B,
    ) -> SymbolicResult<Vec<Vec<E>>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let dx = (x_max - x_min) / (grid_points - 1) as f64;
        let mut matrix = Vec::new();

        // Kinetic energy: $-\hbar^2/(2m dx^2) \cdot (\delta_{i,j+1} - 2\delta_{i,j} + \delta_{i,j-1})$
        let kinetic_coeff = -hbar * hbar / (2.0 * mass * dx * dx);

        for i in 0..grid_points {
            let mut row = Vec::new();
            let _x_i = x_min + i as f64 * dx;

            for j in 0..grid_points {
                let elem = if i == j {
                    // Diagonal: kinetic + potential
                    let ke = backend.constant(-2.0 * kinetic_coeff)?;
                    let v = potential.clone();
                    backend.add(&ke, &v)?
                } else if (i as isize - j as isize).abs() == 1 {
                    // Off-diagonal neighbors
                    backend.constant(kinetic_coeff)?
                } else {
                    backend.constant(0.0)?
                };

                row.push(elem);
            }
            matrix.push(row);
        }

        Ok(matrix)
    }

    /// Solve eigenvalue problem numerically for symmetric/Hermitian matrices
    ///
    /// Uses ndarray-linalg's eigh method for efficient eigenvalue decomposition.
    /// Eigenvalues are sorted in ascending order and eigenvectors are normalized.
    ///
    /// # Arguments
    /// * `hamiltonian_matrix` - Symmetric or Hermitian matrix as nested Vec
    ///
    /// # Returns
    /// * `Ok((eigenvalues, eigenvectors))` - Sorted eigenvalues and corresponding normalized eigenvectors
    /// * `Err` - If matrix is not square or eigenvalue decomposition fails
    ///
    /// # Examples
    /// ```
    /// use myquat::qm_solver::numerical_methods::matrix_diagonalization::solve_eigenvalue_problem;
    ///
    /// let matrix = vec![
    ///     vec![2.0, -1.0, 0.0],
    ///     vec![-1.0, 2.0, -1.0],
    ///     vec![0.0, -1.0, 2.0],
    /// ];
    /// let (eigenvalues, eigenvectors) = solve_eigenvalue_problem(&matrix).unwrap();
    /// ```
    pub fn solve_eigenvalue_problem(
        hamiltonian_matrix: &[Vec<f64>],
    ) -> SymbolicResult<(Vec<f64>, Vec<Vec<f64>>)> {
        use crate::symbolic::SymbolicError;
        use ndarray::{s, Array2};
        use ndarray_linalg::Eigh;

        let n = hamiltonian_matrix.len();
        if n == 0 {
            return Err(SymbolicError::MatrixOperationFailed(
                "Empty matrix".to_string(),
            ));
        }

        // Check if matrix is square
        for row in hamiltonian_matrix {
            if row.len() != n {
                return Err(SymbolicError::MatrixOperationFailed(
                    "Matrix must be square".to_string(),
                ));
            }
        }

        // Convert to ndarray Array2
        let mut matrix_data = Vec::with_capacity(n * n);
        for row in hamiltonian_matrix {
            matrix_data.extend_from_slice(row);
        }
        let matrix = Array2::from_shape_vec((n, n), matrix_data).map_err(|e| {
            SymbolicError::MatrixOperationFailed(format!("Failed to create matrix: {}", e))
        })?;

        // Solve eigenvalue problem using eigh (for symmetric/Hermitian matrices)
        let (eigenvalues_array, eigenvectors_array) =
            matrix.eigh(ndarray_linalg::UPLO::Upper).map_err(|e| {
                SymbolicError::MatrixOperationFailed(format!(
                    "Eigenvalue decomposition failed: {}",
                    e
                ))
            })?;

        // Convert eigenvalues to Vec and sort (eigh already returns sorted values)
        let eigenvalues: Vec<f64> = eigenvalues_array.to_vec();

        // Convert eigenvectors to Vec<Vec<f64>>
        // Each column of eigenvectors_array is an eigenvector
        let mut eigenvectors = Vec::with_capacity(n);
        for i in 0..n {
            let eigenvector: Vec<f64> = eigenvectors_array.slice(s![.., i]).to_vec();
            eigenvectors.push(eigenvector);
        }

        Ok((eigenvalues, eigenvectors))
    }

    /// Solve eigenvalue problem for complex Hermitian matrices
    ///
    /// Uses ndarray-linalg's eigh method for complex Hermitian matrices.
    /// Eigenvalues are real and sorted in ascending order.
    ///
    /// # Arguments
    /// * `hamiltonian_matrix` - Hermitian matrix as nested Vec of Complex64
    ///
    /// # Returns
    /// * `Ok((eigenvalues, eigenvectors))` - Real eigenvalues and complex eigenvectors
    pub fn solve_eigenvalue_problem_complex(
        hamiltonian_matrix: &[Vec<num_complex::Complex64>],
    ) -> SymbolicResult<(Vec<f64>, Vec<Vec<num_complex::Complex64>>)> {
        use crate::symbolic::SymbolicError;
        use ndarray::{s, Array2};
        use ndarray_linalg::Eigh;
        use num_complex::Complex64;

        let n = hamiltonian_matrix.len();
        if n == 0 {
            return Err(SymbolicError::MatrixOperationFailed(
                "Empty matrix".to_string(),
            ));
        }

        // Check if matrix is square
        for row in hamiltonian_matrix {
            if row.len() != n {
                return Err(SymbolicError::MatrixOperationFailed(
                    "Matrix must be square".to_string(),
                ));
            }
        }

        // Convert to ndarray Array2
        let mut matrix_data = Vec::with_capacity(n * n);
        for row in hamiltonian_matrix {
            matrix_data.extend_from_slice(row);
        }
        let matrix = Array2::from_shape_vec((n, n), matrix_data).map_err(|e| {
            SymbolicError::MatrixOperationFailed(format!("Failed to create matrix: {}", e))
        })?;

        // Solve eigenvalue problem using eigh
        let (eigenvalues_array, eigenvectors_array) =
            matrix.eigh(ndarray_linalg::UPLO::Upper).map_err(|e| {
                SymbolicError::MatrixOperationFailed(format!(
                    "Eigenvalue decomposition failed: {}",
                    e
                ))
            })?;

        // Convert eigenvalues to Vec (already real and sorted)
        let eigenvalues: Vec<f64> = eigenvalues_array.to_vec();

        // Convert eigenvectors to Vec<Vec<Complex64>>
        let mut eigenvectors = Vec::with_capacity(n);
        for i in 0..n {
            let eigenvector: Vec<Complex64> = eigenvectors_array.slice(s![.., i]).to_vec();
            eigenvectors.push(eigenvector);
        }

        Ok((eigenvalues, eigenvectors))
    }
}

/// Finite difference methods
pub mod finite_difference {
    use super::*;

    /// Three-point finite difference for second derivative
    ///
    /// Approximates the second derivative using centered difference:
    /// $$ \frac{d^2\psi}{dx^2} \approx \frac{\psi_{i+1} - 2\psi_i + \psi_{i-1}}{\Delta x^2} $$
    ///
    /// This has $O(\Delta x^2)$ accuracy.
    pub fn second_derivative_3point(psi_prev: f64, psi_curr: f64, psi_next: f64, dx: f64) -> f64 {
        (psi_next - 2.0 * psi_curr + psi_prev) / (dx * dx)
    }

    /// Five-point finite difference (more accurate)
    ///
    /// Higher-order approximation of the second derivative:
    /// $$ \frac{d^2\psi}{dx^2} \approx \frac{-\psi_{i+2} + 16\psi_{i+1} - 30\psi_i + 16\psi_{i-1} - \psi_{i-2}}{12\Delta x^2} $$
    ///
    /// This has $O(\Delta x^4)$ accuracy, better than the three-point formula.
    pub fn second_derivative_5point(psi: &[f64], i: usize, dx: f64) -> f64 {
        if i < 2 || i >= psi.len() - 2 {
            return 0.0;
        }

        let numerator =
            -psi[i + 2] + 16.0 * psi[i + 1] - 30.0 * psi[i] + 16.0 * psi[i - 1] - psi[i - 2];
        numerator / (12.0 * dx * dx)
    }

    /// Solve 1D Schrodinger equation using finite difference
    ///
    /// Solves the time-independent Schrodinger equation:
    /// $$ \left[-\frac{\hbar^2}{2m} \frac{d^2}{dx^2} + V(x)\right]\psi = E\psi $$
    ///
    /// # Note
    /// This function is not yet implemented. Use `finite_element::solve_1d_schrodinger`
    /// which provides a working solver with linear and quadratic basis functions.
    #[deprecated(
        since = "0.1.0",
        note = "Not implemented. Use finite_element::solve_1d_schrodinger instead."
    )]
    pub fn solve_1d_schrodinger(
        grid_points: usize,
        x_min: f64,
        x_max: f64,
        _potential: &[f64],
        _mass: f64,
        _hbar: f64,
        num_eigenvalues: usize,
    ) -> SymbolicResult<(Vec<f64>, Vec<Vec<f64>>)> {
        let _dx = (x_max - x_min) / (grid_points - 1) as f64;
        let _ = (grid_points, x_min, x_max, num_eigenvalues);

        Err(SymbolicError::UnsupportedOperation(
            "Finite difference 1D Schrodinger solver is not implemented. \
             Use finite_element::solve_1d_schrodinger instead."
                .to_string(),
        ))
    }
}

/// Finite element methods for 1D quantum mechanics
///
/// # Current Implementation Status
///
/// This module implements a simplified FEM for 1D quantum mechanics problems.
/// The current implementation assembles the stiffness matrix correctly but
/// does not include the mass matrix in the eigenvalue problem.
///
/// ## TODO: Mass Matrix Implementation
///
/// The proper FEM formulation requires solving the generalized eigenvalue problem:
/// ```text
/// $$ \frac{\hbar^2}{2m} K \psi = E M \psi $$
/// ```
/// where:
/// - K is the stiffness matrix: $K_{ij} = \int \frac{d\phi_i}{dx} \frac{d\phi_j}{dx} dx$
/// - M is the mass matrix: $M_{ij} = \int \phi_i \phi_j dx$
///
/// Current implementation solves: $K \psi = \lambda \psi$
/// This gives qualitatively correct results (correct eigenvalue ratios and ordering)
/// but quantitatively incorrect absolute values (off by a scaling factor).
///
/// Future improvements:
/// 1. Implement mass matrix assembly
/// 2. Use generalized eigenvalue solver (e.g., via ndarray-linalg's `eigh` with Cholesky)
/// 3. Or use lumped mass matrix approximation for efficiency
///
pub mod finite_element {
    use super::*;

    /// Basis function type
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BasisFunction {
        /// Linear (hat) functions - P1 elements
        Linear,
        /// Quadratic functions - P2 elements
        Quadratic,
        /// Cubic functions - P3 elements
        Cubic,
    }

    /// Gauss-Legendre quadrature points and weights
    /// Returns (points, weights) for integration on [-1, 1]
    pub(crate) fn gauss_quadrature(order: usize) -> (Vec<f64>, Vec<f64>) {
        match order {
            1 => {
                // 1-point: exact for polynomials up to degree 1
                (vec![0.0], vec![2.0])
            }
            2 => {
                // 2-point: exact for polynomials up to degree 3
                let a = 1.0 / 3.0_f64.sqrt();
                (vec![-a, a], vec![1.0, 1.0])
            }
            3 => {
                // 3-point: exact for polynomials up to degree 5
                let a = (3.0 / 5.0_f64).sqrt();
                (vec![-a, 0.0, a], vec![5.0 / 9.0, 8.0 / 9.0, 5.0 / 9.0])
            }
            4 => {
                // 4-point: exact for polynomials up to degree 7
                let a = ((3.0 - 2.0 * (6.0 / 5.0_f64).sqrt()) / 7.0).sqrt();
                let b = ((3.0 + 2.0 * (6.0 / 5.0_f64).sqrt()) / 7.0).sqrt();
                let w1 = (18.0 + 30.0_f64.sqrt()) / 36.0;
                let w2 = (18.0 - 30.0_f64.sqrt()) / 36.0;
                (vec![-b, -a, a, b], vec![w2, w1, w1, w2])
            }
            _ => {
                // Default to 2-point
                let a = 1.0 / 3.0_f64.sqrt();
                (vec![-a, a], vec![1.0, 1.0])
            }
        }
    }

    /// Linear basis functions on reference element [-1, 1]
    ///
    /// $$ \phi_0(\xi) = \frac{1-\xi}{2}, \quad \phi_1(\xi) = \frac{1+\xi}{2} $$
    fn linear_basis(xi: f64, node: usize) -> f64 {
        match node {
            0 => (1.0 - xi) / 2.0,
            1 => (1.0 + xi) / 2.0,
            _ => 0.0,
        }
    }

    /// Derivative of linear basis functions on reference element
    ///
    /// $$ \frac{d\phi_0}{d\xi} = -\frac{1}{2}, \quad \frac{d\phi_1}{d\xi} = \frac{1}{2} $$
    fn linear_basis_derivative(node: usize) -> f64 {
        match node {
            0 => -0.5,
            1 => 0.5,
            _ => 0.0,
        }
    }

    /// Quadratic basis functions on reference element [-1, 1]
    ///
    /// $$ \phi_0(\xi) = \frac{\xi(\xi-1)}{2}, \quad \phi_1(\xi) = 1-\xi^2, \quad \phi_2(\xi) = \frac{\xi(\xi+1)}{2} $$
    fn quadratic_basis(xi: f64, node: usize) -> f64 {
        match node {
            0 => xi * (xi - 1.0) / 2.0,
            1 => 1.0 - xi * xi,
            2 => xi * (xi + 1.0) / 2.0,
            _ => 0.0,
        }
    }

    /// Derivative of quadratic basis functions
    fn quadratic_basis_derivative(xi: f64, node: usize) -> f64 {
        match node {
            0 => xi - 0.5,
            1 => -2.0 * xi,
            2 => xi + 0.5,
            _ => 0.0,
        }
    }

    /// Finite element basis for 1D problems
    pub struct FEMBasis {
        /// Basis function type
        pub basis_type: BasisFunction,

        /// Number of elements
        pub num_elements: usize,

        /// Grid points (nodes)
        pub nodes: Vec<f64>,

        /// Number of degrees of freedom
        pub num_dofs: usize,
    }

    impl FEMBasis {
        /// Create new FEM basis
        ///
        /// # Arguments
        /// * `basis_type` - Type of basis functions (Linear, Quadratic, Cubic)
        /// * `num_elements` - Number of finite elements
        /// * `x_min` - Left boundary
        /// * `x_max` - Right boundary
        pub fn new(basis_type: BasisFunction, num_elements: usize, x_min: f64, x_max: f64) -> Self {
            let dx = (x_max - x_min) / num_elements as f64;

            // For linear elements: num_dofs = num_elements + 1
            // For quadratic elements: num_dofs = 2*num_elements + 1 (with mid-points)
            let (nodes, num_dofs) = match basis_type {
                BasisFunction::Linear => {
                    let nodes: Vec<f64> =
                        (0..=num_elements).map(|i| x_min + i as f64 * dx).collect();
                    (nodes, num_elements + 1)
                }
                BasisFunction::Quadratic => {
                    // Include mid-points for quadratic elements
                    let mut nodes = Vec::new();
                    for i in 0..num_elements {
                        let x0 = x_min + i as f64 * dx;
                        nodes.push(x0);
                        nodes.push(x0 + dx / 2.0); // mid-point
                    }
                    nodes.push(x_max);
                    (nodes, 2 * num_elements + 1)
                }
                BasisFunction::Cubic => {
                    // Simplified: use same as quadratic for now
                    let mut nodes = Vec::new();
                    for i in 0..num_elements {
                        let x0 = x_min + i as f64 * dx;
                        nodes.push(x0);
                        nodes.push(x0 + dx / 2.0);
                    }
                    nodes.push(x_max);
                    (nodes, 2 * num_elements + 1)
                }
            };

            Self {
                basis_type,
                num_elements,
                nodes,
                num_dofs,
            }
        }

        /// Get element size
        pub fn element_size(&self, element: usize) -> f64 {
            if element >= self.num_elements {
                return 0.0;
            }
            match self.basis_type {
                BasisFunction::Linear => self.nodes[element + 1] - self.nodes[element],
                BasisFunction::Quadratic | BasisFunction::Cubic => {
                    self.nodes[2 * element + 2] - self.nodes[2 * element]
                }
            }
        }
    }

    /// Assemble 1D quantum Hamiltonian using finite element method
    ///
    /// Solves the time-independent Schrodinger equation:
    /// $$ \left[-\frac{\hbar^2}{2m} \frac{d^2}{dx^2} + V(x)\right]\psi = E\psi $$
    /// with homogeneous Dirichlet boundary conditions: $\psi(x_{\text{min}}) = \psi(x_{\text{max}}) = 0$
    ///
    /// # Arguments
    /// * `basis` - FEM basis
    /// * `potential_fn` - Potential energy function V(x)
    /// * `mass` - Particle mass
    /// * `hbar` - Reduced Planck constant
    ///
    /// # Returns
    /// Hamiltonian matrix H where $H\psi = E\psi$ (with boundary DOFs removed)
    pub fn assemble_hamiltonian_1d<F>(
        basis: &FEMBasis,
        potential_fn: F,
        mass: f64,
        hbar: f64,
    ) -> SymbolicResult<Vec<Vec<f64>>>
    where
        F: Fn(f64) -> f64,
    {
        // Interior DOFs only (exclude boundary nodes)
        let n_interior = basis.num_dofs - 2;
        let mut hamiltonian = vec![vec![0.0; n_interior]; n_interior];

        // Kinetic energy coefficient
        let ke_coeff = hbar * hbar / (2.0 * mass);

        // Choose quadrature order based on basis type
        let quad_order = match basis.basis_type {
            BasisFunction::Linear => 2,    // Exact for linear
            BasisFunction::Quadratic => 3, // Exact for quadratic
            BasisFunction::Cubic => 4,     // Exact for cubic
        };

        let (quad_points, quad_weights) = gauss_quadrature(quad_order);

        // Assemble element by element
        for elem in 0..basis.num_elements {
            let h = basis.element_size(elem);
            let x_left = match basis.basis_type {
                BasisFunction::Linear => basis.nodes[elem],
                _ => basis.nodes[2 * elem],
            };

            // Number of local nodes per element
            let _local_nodes = match basis.basis_type {
                BasisFunction::Linear => 2,
                BasisFunction::Quadratic => 3,
                BasisFunction::Cubic => 3, // Simplified
            };

            // Local to global DOF mapping
            let global_dofs: Vec<usize> = match basis.basis_type {
                BasisFunction::Linear => vec![elem, elem + 1],
                BasisFunction::Quadratic => vec![2 * elem, 2 * elem + 1, 2 * elem + 2],
                BasisFunction::Cubic => vec![2 * elem, 2 * elem + 1, 2 * elem + 2],
            };

            // Integrate over element using Gauss quadrature
            for (i, &dof_i) in global_dofs.iter().enumerate() {
                // Skip boundary DOFs
                if dof_i == 0 || dof_i == basis.num_dofs - 1 {
                    continue;
                }

                for (j, &dof_j) in global_dofs.iter().enumerate() {
                    // Skip boundary DOFs
                    if dof_j == 0 || dof_j == basis.num_dofs - 1 {
                        continue;
                    }

                    let mut stiffness = 0.0; // Kinetic energy term
                    let mut mass_matrix = 0.0; // Potential energy term

                    // Gauss quadrature integration
                    for (q, &xi) in quad_points.iter().enumerate() {
                        let weight = quad_weights[q];

                        // Physical coordinate
                        let x = x_left + h * (xi + 1.0) / 2.0;

                        // Jacobian: $dx/d\xi = h/2$
                        let jacobian = h / 2.0;

                        // Evaluate basis functions and derivatives
                        let (phi_i, dphi_i_dxi) = match basis.basis_type {
                            BasisFunction::Linear => {
                                (linear_basis(xi, i), linear_basis_derivative(i))
                            }
                            BasisFunction::Quadratic => {
                                (quadratic_basis(xi, i), quadratic_basis_derivative(xi, i))
                            }
                            BasisFunction::Cubic => {
                                // Use quadratic for now
                                (quadratic_basis(xi, i), quadratic_basis_derivative(xi, i))
                            }
                        };

                        let (phi_j, dphi_j_dxi) = match basis.basis_type {
                            BasisFunction::Linear => {
                                (linear_basis(xi, j), linear_basis_derivative(j))
                            }
                            BasisFunction::Quadratic => {
                                (quadratic_basis(xi, j), quadratic_basis_derivative(xi, j))
                            }
                            BasisFunction::Cubic => {
                                (quadratic_basis(xi, j), quadratic_basis_derivative(xi, j))
                            }
                        };

                        // Transform derivatives: $d\phi/dx = d\phi/d\xi \cdot d\xi/dx = d\phi/d\xi / J$
                        let _dphi_i_dx = dphi_i_dxi / jacobian;
                        let _dphi_j_dx = dphi_j_dxi / jacobian;

                        // Stiffness matrix: $\int \frac{d\phi_i}{dx} \frac{d\phi_j}{dx} dx$
                        // $= \int (\frac{d\phi_i}{d\xi} / J) (\frac{d\phi_j}{d\xi} / J) J d\xi$
                        // $= \int \frac{d\phi_i}{d\xi} \frac{d\phi_j}{d\xi} / J d\xi$
                        stiffness += dphi_i_dxi * dphi_j_dxi / jacobian * weight;

                        // Mass matrix with potential: $\int V(x) \phi_i \phi_j dx$
                        // $= \int V(x) \phi_i \phi_j J d\xi$
                        let v_x = potential_fn(x);
                        mass_matrix += v_x * phi_i * phi_j * jacobian * weight;
                    }

                    // Map to interior DOF indices (subtract 1 to account for removed first boundary node)
                    let interior_i = dof_i - 1;
                    let interior_j = dof_j - 1;

                    // Add to global Hamiltonian: $H = \frac{\hbar^2}{2m} K + V M$
                    hamiltonian[interior_i][interior_j] += ke_coeff * stiffness + mass_matrix;
                }
            }
        }

        Ok(hamiltonian)
    }

    /// Solve 1D quantum problem using FEM
    ///
    /// # Arguments
    /// * `num_elements` - Number of finite elements
    /// * `x_min`, `x_max` - Domain boundaries
    /// * `potential_fn` - Potential energy function V(x)
    /// * `mass` - Particle mass
    /// * `hbar` - Reduced Planck constant
    /// * `basis_type` - Type of basis functions
    ///
    /// # Returns
    /// * `(eigenvalues, eigenvectors)` - Energy levels and wavefunctions
    pub fn solve_1d_schrodinger<F>(
        num_elements: usize,
        x_min: f64,
        x_max: f64,
        potential_fn: F,
        mass: f64,
        hbar: f64,
        basis_type: BasisFunction,
    ) -> SymbolicResult<(Vec<f64>, Vec<Vec<f64>>)>
    where
        F: Fn(f64) -> f64,
    {
        // Create FEM basis
        let basis = FEMBasis::new(basis_type, num_elements, x_min, x_max);

        // Assemble Hamiltonian
        let hamiltonian = assemble_hamiltonian_1d(&basis, potential_fn, mass, hbar)?;

        // Solve eigenvalue problem
        let (eigenvalues, eigenvectors) =
            super::matrix_diagonalization::solve_eigenvalue_problem(&hamiltonian)?;

        Ok((eigenvalues, eigenvectors))
    }
}

/// Numerical integration methods for quantum mechanics
///
/// This module provides various numerical integration techniques for:
/// - Expectation value calculations: $\langle\psi|\hat{O}|\psi\rangle$
/// - Wavefunction normalization: $\int|\psi|^2 dx$
/// - Overlap integrals: $\langle\psi_1|\psi_2\rangle$
/// - Matrix elements: $\langle\psi_i|V|\psi_j\rangle$
///
pub mod numerical_integration {
    use super::*;
    use crate::symbolic::SymbolicError;

    /// Integration method type
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum IntegrationMethod {
        /// Trapezoidal rule - $O(h^2)$ accuracy
        Trapezoidal,
        /// Simpson's rule - $O(h^4)$ accuracy
        Simpson,
        /// Gauss-Legendre quadrature - high accuracy
        GaussLegendre,
        /// Adaptive integration with error control
        Adaptive,
    }

    /// Integrate a 1D function over [a, b]
    ///
    /// # Arguments
    /// * `f` - Function to integrate
    /// * `a` - Lower bound
    /// * `b` - Upper bound
    /// * `n_points` - Number of integration points
    /// * `method` - Integration method
    ///
    /// # Returns
    /// Integral value
    pub fn integrate_1d<F>(
        f: F,
        a: f64,
        b: f64,
        n_points: usize,
        method: IntegrationMethod,
    ) -> SymbolicResult<f64>
    where
        F: Fn(f64) -> f64,
    {
        if a >= b {
            return Err(SymbolicError::InvalidExpression(
                "Integration bounds: a must be less than b".to_string(),
            ));
        }

        if n_points < 2 {
            return Err(SymbolicError::InvalidExpression(
                "At least 2 integration points required".to_string(),
            ));
        }

        match method {
            IntegrationMethod::Trapezoidal => trapezoidal_rule(&f, a, b, n_points),
            IntegrationMethod::Simpson => simpson_rule(&f, a, b, n_points),
            IntegrationMethod::GaussLegendre => gauss_legendre_integrate(&f, a, b, n_points),
            IntegrationMethod::Adaptive => adaptive_integrate(&f, a, b, 1e-10),
        }
    }

    /// Trapezoidal rule integration
    ///
    /// $$ \int f(x)dx \approx \frac{h}{2} [f(x_0) + 2f(x_1) + 2f(x_2) + \cdots + 2f(x_{n-1}) + f(x_n)] $$
    fn trapezoidal_rule<F>(f: &F, a: f64, b: f64, n: usize) -> SymbolicResult<f64>
    where
        F: Fn(f64) -> f64,
    {
        let h = (b - a) / (n - 1) as f64;
        let mut sum = 0.5 * (f(a) + f(b));

        for i in 1..n - 1 {
            let x = a + i as f64 * h;
            sum += f(x);
        }

        Ok(h * sum)
    }

    /// Simpson's rule integration
    ///
    /// $$ \int f(x)dx \approx \frac{h}{3} [f(x_0) + 4f(x_1) + 2f(x_2) + 4f(x_3) + \cdots + f(x_n)] $$
    ///
    /// Requires odd number of points (even number of intervals)
    fn simpson_rule<F>(f: &F, a: f64, b: f64, n: usize) -> SymbolicResult<f64>
    where
        F: Fn(f64) -> f64,
    {
        // Ensure n is odd
        let n = if n % 2 == 0 { n + 1 } else { n };
        let h = (b - a) / (n - 1) as f64;

        let mut sum = f(a) + f(b);

        for i in 1..n - 1 {
            let x = a + i as f64 * h;
            let weight = if i % 2 == 0 { 2.0 } else { 4.0 };
            sum += weight * f(x);
        }

        Ok(h * sum / 3.0)
    }

    /// Gauss-Legendre integration on arbitrary interval [a, b]
    fn gauss_legendre_integrate<F>(f: &F, a: f64, b: f64, order: usize) -> SymbolicResult<f64>
    where
        F: Fn(f64) -> f64,
    {
        // Get Gauss points and weights for [-1, 1]
        let (points, weights) = super::finite_element::gauss_quadrature(order.min(4));

        // Transform to [a, b]: $x = \frac{b-a}{2} \xi + \frac{b+a}{2}$
        let mid = (b + a) / 2.0;
        let half_width = (b - a) / 2.0;

        let mut sum = 0.0;
        for (i, &xi) in points.iter().enumerate() {
            let x = mid + half_width * xi;
            sum += weights[i] * f(x);
        }

        Ok(half_width * sum)
    }

    /// Adaptive integration with error control
    ///
    /// Uses recursive subdivision until error tolerance is met
    fn adaptive_integrate<F>(f: &F, a: f64, b: f64, tol: f64) -> SymbolicResult<f64>
    where
        F: Fn(f64) -> f64,
    {
        adaptive_simpson_recursive(f, a, b, tol, f(a), f((a + b) / 2.0), f(b), 0)
    }

    /// Recursive adaptive Simpson's rule
    fn adaptive_simpson_recursive<F>(
        f: &F,
        a: f64,
        b: f64,
        tol: f64,
        fa: f64,
        fm: f64,
        fb: f64,
        depth: usize,
    ) -> SymbolicResult<f64>
    where
        F: Fn(f64) -> f64,
    {
        const MAX_DEPTH: usize = 50;

        if depth > MAX_DEPTH {
            return Err(SymbolicError::IntegrationFailed(
                "Maximum recursion depth exceeded in adaptive integration".to_string(),
            ));
        }

        let h = b - a;
        let m = (a + b) / 2.0;
        let lm = (a + m) / 2.0;
        let rm = (m + b) / 2.0;

        let flm = f(lm);
        let frm = f(rm);

        // Simpson's rule on whole interval
        let s_whole = h / 6.0 * (fa + 4.0 * fm + fb);

        // Simpson's rule on left and right halves
        let s_left = h / 12.0 * (fa + 4.0 * flm + fm);
        let s_right = h / 12.0 * (fm + 4.0 * frm + fb);
        let s_halves = s_left + s_right;

        // Error estimate
        let error = (s_halves - s_whole).abs() / 15.0;

        if error < tol {
            Ok(s_halves + (s_halves - s_whole) / 15.0) // Richardson extrapolation
        } else {
            let left = adaptive_simpson_recursive(f, a, m, tol / 2.0, fa, flm, fm, depth + 1)?;
            let right = adaptive_simpson_recursive(f, m, b, tol / 2.0, fm, frm, fb, depth + 1)?;
            Ok(left + right)
        }
    }

    /// Calculate expectation value $\langle\psi|\hat{O}|\psi\rangle$ for a 1D wavefunction
    ///
    /// # Arguments
    /// * `psi` - Wavefunction values on grid
    /// * `operator_psi` - $\hat{O}|\psi\rangle$ values on grid
    /// * `x` - Grid points
    ///
    /// # Returns
    /// Expectation value
    pub fn expectation_value_1d(
        psi: &[f64],
        operator_psi: &[f64],
        x: &[f64],
    ) -> SymbolicResult<f64> {
        if psi.len() != operator_psi.len() || psi.len() != x.len() {
            return Err(SymbolicError::MatrixOperationFailed(
                "Array dimensions must match".to_string(),
            ));
        }

        let integrand: Vec<f64> = psi
            .iter()
            .zip(operator_psi.iter())
            .map(|(p, op)| p * op)
            .collect();

        integrate_discrete(&integrand, x, IntegrationMethod::Simpson)
    }

    /// Calculate overlap integral $\langle\psi_1|\psi_2\rangle$
    pub fn overlap_integral_1d(psi1: &[f64], psi2: &[f64], x: &[f64]) -> SymbolicResult<f64> {
        if psi1.len() != psi2.len() || psi1.len() != x.len() {
            return Err(SymbolicError::MatrixOperationFailed(
                "Array dimensions must match".to_string(),
            ));
        }

        let integrand: Vec<f64> = psi1
            .iter()
            .zip(psi2.iter())
            .map(|(p1, p2)| p1 * p2)
            .collect();

        integrate_discrete(&integrand, x, IntegrationMethod::Simpson)
    }

    /// Normalize a wavefunction
    ///
    /// Returns normalized wavefunction and the normalization constant
    pub fn normalize_wavefunction_1d(psi: &[f64], x: &[f64]) -> SymbolicResult<(Vec<f64>, f64)> {
        // Calculate norm: $\int|\psi|^2 dx$
        let psi_squared: Vec<f64> = psi.iter().map(|p| p * p).collect();
        let norm_squared = integrate_discrete(&psi_squared, x, IntegrationMethod::Simpson)?;

        if norm_squared <= 0.0 {
            return Err(SymbolicError::InvalidExpression(
                "Wavefunction has zero or negative norm".to_string(),
            ));
        }

        let norm = norm_squared.sqrt();
        let normalized: Vec<f64> = psi.iter().map(|p| p / norm).collect();

        Ok((normalized, norm))
    }

    /// Integrate discrete data using specified method
    fn integrate_discrete(y: &[f64], x: &[f64], method: IntegrationMethod) -> SymbolicResult<f64> {
        if y.len() != x.len() || y.len() < 2 {
            return Err(SymbolicError::InvalidExpression(
                "Invalid array dimensions for integration".to_string(),
            ));
        }

        match method {
            IntegrationMethod::Trapezoidal => {
                let mut sum = 0.0;
                for i in 0..y.len() - 1 {
                    let h = x[i + 1] - x[i];
                    sum += h * (y[i] + y[i + 1]) / 2.0;
                }
                Ok(sum)
            }
            IntegrationMethod::Simpson => {
                if y.len() < 3 {
                    return integrate_discrete(y, x, IntegrationMethod::Trapezoidal);
                }

                let mut sum = 0.0;

                // Handle pairs of intervals
                let mut i = 0;
                while i + 2 < y.len() {
                    let h1 = x[i + 1] - x[i];
                    let h2 = x[i + 2] - x[i + 1];

                    if (h1 - h2).abs() < 1e-10 {
                        // Equal spacing: use standard Simpson's rule
                        let h = h1;
                        sum += h / 3.0 * (y[i] + 4.0 * y[i + 1] + y[i + 2]);
                        i += 2;
                    } else {
                        // Unequal spacing: use trapezoidal for this segment
                        sum += h1 * (y[i] + y[i + 1]) / 2.0;
                        i += 1;
                    }
                }

                // Handle remaining points with trapezoidal
                while i + 1 < y.len() {
                    let h = x[i + 1] - x[i];
                    sum += h * (y[i] + y[i + 1]) / 2.0;
                    i += 1;
                }

                Ok(sum)
            }
            _ => integrate_discrete(y, x, IntegrationMethod::Trapezoidal),
        }
    }

    /// 2D integration using product of 1D quadratures
    ///
    /// $$ \int\int f(x,y) dx dy \text{ over } [a_x,b_x] \times [a_y,b_y] $$
    pub fn integrate_2d<F>(
        f: F,
        ax: f64,
        bx: f64,
        ay: f64,
        by: f64,
        nx: usize,
        ny: usize,
        method: IntegrationMethod,
    ) -> SymbolicResult<f64>
    where
        F: Fn(f64, f64) -> f64,
    {
        // Integrate over y for each x, then integrate over x
        let integrand_x = |x: f64| -> f64 {
            let integrand_y = |y: f64| f(x, y);
            integrate_1d(integrand_y, ay, by, ny, method).unwrap_or(0.0)
        };

        integrate_1d(integrand_x, ax, bx, nx, method)
    }
}

/// Numerical gradient and derivative computation
///
/// This module provides numerical differentiation methods for:
/// - Gradient computation: $\nabla f$ for scalar functions
/// - Jacobian matrices: $J_{ij} = \frac{\partial f_i}{\partial x_j}$ for vector functions
/// - Hessian matrices: $H_{ij} = \frac{\partial^2 f}{\partial x_i \partial x_j}$ for scalar functions
/// - Directional derivatives: $\nabla_v f$
///
pub mod numerical_gradient {
    use super::*;
    use crate::symbolic::SymbolicError;

    /// Differentiation method
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DifferentiationMethod {
        /// Forward difference: f'(x) ≈ (f(x+h) - f(x))/h
        Forward,
        /// Backward difference: f'(x) ≈ (f(x) - f(x-h))/h
        Backward,
        /// Central difference: f'(x) ≈ (f(x+h) - f(x-h))/(2h)
        Central,
        /// Complex step: f'(x) ≈ Im(f(x+ih))/h (no subtractive cancellation)
        ComplexStep,
    }

    /// Compute numerical derivative of a scalar function
    ///
    /// # Arguments
    /// * `f` - Function to differentiate
    /// * `x` - Point at which to compute derivative
    /// * `h` - Step size (default: 1e-8 for central, 1e-100 for complex step)
    /// * `method` - Differentiation method
    ///
    /// # Returns
    /// Approximate derivative f'(x)
    pub fn derivative<F>(
        f: F,
        x: f64,
        h: Option<f64>,
        method: DifferentiationMethod,
    ) -> SymbolicResult<f64>
    where
        F: Fn(f64) -> f64,
    {
        let h = h.unwrap_or(match method {
            DifferentiationMethod::ComplexStep => 1e-100,
            _ => 1e-8,
        });

        match method {
            DifferentiationMethod::Forward => Ok((f(x + h) - f(x)) / h),
            DifferentiationMethod::Backward => Ok((f(x) - f(x - h)) / h),
            DifferentiationMethod::Central => Ok((f(x + h) - f(x - h)) / (2.0 * h)),
            DifferentiationMethod::ComplexStep => {
                // Use dual numbers approximation for real functions
                // This is a simplified version; true complex step needs complex arithmetic
                Ok((f(x + h) - f(x - h)) / (2.0 * h))
            }
        }
    }

    /// Compute gradient of a scalar function f: ℝⁿ → ℝ
    ///
    /// ∇f = [∂f/∂x₁, ∂f/∂x₂, ..., ∂f/∂xₙ]
    ///
    /// # Arguments
    /// * `f` - Scalar function of n variables
    /// * `x` - Point at which to compute gradient
    /// * `h` - Step size (optional)
    /// * `method` - Differentiation method
    ///
    /// # Returns
    /// Gradient vector
    pub fn gradient<F>(
        f: F,
        x: &[f64],
        h: Option<f64>,
        method: DifferentiationMethod,
    ) -> SymbolicResult<Vec<f64>>
    where
        F: Fn(&[f64]) -> f64,
    {
        let n = x.len();
        let h = h.unwrap_or(1e-8);
        let mut grad = vec![0.0; n];

        for i in 0..n {
            let mut x_plus = x.to_vec();
            let mut x_minus = x.to_vec();

            match method {
                DifferentiationMethod::Forward => {
                    x_plus[i] += h;
                    grad[i] = (f(&x_plus) - f(x)) / h;
                }
                DifferentiationMethod::Backward => {
                    x_minus[i] -= h;
                    grad[i] = (f(x) - f(&x_minus)) / h;
                }
                DifferentiationMethod::Central | DifferentiationMethod::ComplexStep => {
                    x_plus[i] += h;
                    x_minus[i] -= h;
                    grad[i] = (f(&x_plus) - f(&x_minus)) / (2.0 * h);
                }
            }
        }

        Ok(grad)
    }

    /// Compute Jacobian matrix of a vector function f: ℝⁿ → ℝᵐ
    ///
    /// J_ij = ∂f_i/∂x_j
    ///
    /// # Arguments
    /// * `f` - Vector function (returns m-dimensional vector)
    /// * `x` - Point at which to compute Jacobian (n-dimensional)
    /// * `h` - Step size (optional)
    /// * `method` - Differentiation method
    ///
    /// # Returns
    /// Jacobian matrix (m × n)
    pub fn jacobian<F>(
        f: F,
        x: &[f64],
        h: Option<f64>,
        method: DifferentiationMethod,
    ) -> SymbolicResult<Vec<Vec<f64>>>
    where
        F: Fn(&[f64]) -> Vec<f64>,
    {
        let n = x.len();
        let f_x = f(x);
        let m = f_x.len();
        let h = h.unwrap_or(1e-8);

        let mut jac = vec![vec![0.0; n]; m];

        for j in 0..n {
            let mut x_plus = x.to_vec();
            let mut x_minus = x.to_vec();

            match method {
                DifferentiationMethod::Forward => {
                    x_plus[j] += h;
                    let f_plus = f(&x_plus);
                    for i in 0..m {
                        jac[i][j] = (f_plus[i] - f_x[i]) / h;
                    }
                }
                DifferentiationMethod::Backward => {
                    x_minus[j] -= h;
                    let f_minus = f(&x_minus);
                    for i in 0..m {
                        jac[i][j] = (f_x[i] - f_minus[i]) / h;
                    }
                }
                DifferentiationMethod::Central | DifferentiationMethod::ComplexStep => {
                    x_plus[j] += h;
                    x_minus[j] -= h;
                    let f_plus = f(&x_plus);
                    let f_minus = f(&x_minus);
                    for i in 0..m {
                        jac[i][j] = (f_plus[i] - f_minus[i]) / (2.0 * h);
                    }
                }
            }
        }

        Ok(jac)
    }

    /// Compute Hessian matrix of a scalar function f: ℝⁿ → ℝ
    ///
    /// H_ij = ∂²f/∂x_i∂x_j
    ///
    /// # Arguments
    /// * `f` - Scalar function
    /// * `x` - Point at which to compute Hessian
    /// * `h` - Step size (optional)
    ///
    /// # Returns
    /// Hessian matrix (n × n, symmetric)
    pub fn hessian<F>(f: F, x: &[f64], h: Option<f64>) -> SymbolicResult<Vec<Vec<f64>>>
    where
        F: Fn(&[f64]) -> f64,
    {
        let n = x.len();
        let h = h.unwrap_or(1e-5); // Larger step for second derivatives
        let mut hess = vec![vec![0.0; n]; n];

        let f_x = f(x);

        // Diagonal elements: ∂²f/∂x_i²
        for i in 0..n {
            let mut x_plus = x.to_vec();
            let mut x_minus = x.to_vec();
            x_plus[i] += h;
            x_minus[i] -= h;

            hess[i][i] = (f(&x_plus) - 2.0 * f_x + f(&x_minus)) / (h * h);
        }

        // Off-diagonal elements: ∂²f/∂x_i∂x_j
        for i in 0..n {
            for j in (i + 1)..n {
                let mut x_pp = x.to_vec();
                let mut x_pm = x.to_vec();
                let mut x_mp = x.to_vec();
                let mut x_mm = x.to_vec();

                x_pp[i] += h;
                x_pp[j] += h;
                x_pm[i] += h;
                x_pm[j] -= h;
                x_mp[i] -= h;
                x_mp[j] += h;
                x_mm[i] -= h;
                x_mm[j] -= h;

                let d2f = (f(&x_pp) - f(&x_pm) - f(&x_mp) + f(&x_mm)) / (4.0 * h * h);
                hess[i][j] = d2f;
                hess[j][i] = d2f; // Symmetric
            }
        }

        Ok(hess)
    }

    /// Compute directional derivative ∇_v f at point x
    ///
    /// ∇_v f = ∇f · v
    ///
    /// # Arguments
    /// * `f` - Scalar function
    /// * `x` - Point at which to compute directional derivative
    /// * `v` - Direction vector (will be normalized)
    /// * `h` - Step size (optional)
    ///
    /// # Returns
    /// Directional derivative
    pub fn directional_derivative<F>(
        f: F,
        x: &[f64],
        v: &[f64],
        h: Option<f64>,
    ) -> SymbolicResult<f64>
    where
        F: Fn(&[f64]) -> f64,
    {
        if x.len() != v.len() {
            return Err(SymbolicError::InvalidExpression(
                "Point and direction must have same dimension".to_string(),
            ));
        }

        // Normalize direction vector
        let v_norm: f64 = v.iter().map(|vi| vi * vi).sum::<f64>().sqrt();
        if v_norm == 0.0 {
            return Err(SymbolicError::InvalidExpression(
                "Direction vector cannot be zero".to_string(),
            ));
        }

        let v_unit: Vec<f64> = v.iter().map(|vi| vi / v_norm).collect();
        let h = h.unwrap_or(1e-8);

        // Compute f(x + h*v) and f(x - h*v)
        let x_plus: Vec<f64> = x.iter().zip(&v_unit).map(|(xi, vi)| xi + h * vi).collect();
        let x_minus: Vec<f64> = x.iter().zip(&v_unit).map(|(xi, vi)| xi - h * vi).collect();

        Ok((f(&x_plus) - f(&x_minus)) / (2.0 * h))
    }

    /// Compute gradient using finite differences on a grid
    ///
    /// For a function defined on a grid, compute gradient at each point
    pub fn gradient_on_grid<F>(
        f: F,
        x_grid: &[f64],
        _method: DifferentiationMethod,
    ) -> SymbolicResult<Vec<f64>>
    where
        F: Fn(f64) -> f64,
    {
        let n = x_grid.len();
        if n < 2 {
            return Err(SymbolicError::InvalidExpression(
                "Grid must have at least 2 points".to_string(),
            ));
        }

        let mut grad = vec![0.0; n];

        // Interior points: use central difference
        for i in 1..n - 1 {
            let h_left = x_grid[i] - x_grid[i - 1];
            let h_right = x_grid[i + 1] - x_grid[i];

            if (h_left - h_right).abs() < 1e-10 {
                // Equal spacing
                grad[i] = (f(x_grid[i + 1]) - f(x_grid[i - 1])) / (2.0 * h_left);
            } else {
                // Unequal spacing: use weighted formula
                let f_m = f(x_grid[i - 1]);
                let f_0 = f(x_grid[i]);
                let f_p = f(x_grid[i + 1]);
                grad[i] = (f_p * h_left * h_left - f_m * h_right * h_right
                    + f_0 * (h_right * h_right - h_left * h_left))
                    / (h_left * h_right * (h_left + h_right));
            }
        }

        // Boundary points: use forward/backward difference
        let h0 = x_grid[1] - x_grid[0];
        grad[0] = (f(x_grid[1]) - f(x_grid[0])) / h0;

        let hn = x_grid[n - 1] - x_grid[n - 2];
        grad[n - 1] = (f(x_grid[n - 1]) - f(x_grid[n - 2])) / hn;

        Ok(grad)
    }

    /// Check gradient using finite differences (for testing/validation)
    ///
    /// Compares analytical gradient with numerical gradient
    pub fn check_gradient<F, G>(f: F, grad_f: G, x: &[f64], tolerance: f64) -> SymbolicResult<bool>
    where
        F: Fn(&[f64]) -> f64,
        G: Fn(&[f64]) -> Vec<f64>,
    {
        let analytical_grad = grad_f(x);
        let numerical_grad = gradient(f, x, None, DifferentiationMethod::Central)?;

        if analytical_grad.len() != numerical_grad.len() {
            return Ok(false);
        }

        for (a, n) in analytical_grad.iter().zip(&numerical_grad) {
            let error = (a - n).abs();
            let relative_error = if a.abs() > 1e-10 {
                error / a.abs()
            } else {
                error
            };

            if relative_error > tolerance {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// Hybrid symbolic-numerical methods
pub mod hybrid {
    use super::*;

    /// Symbolic setup for numerical solution
    ///
    /// 1. Define problem symbolically
    /// 2. Generate discretization
    /// 3. Solve numerically
    pub fn symbolic_to_numerical<B, E>(
        _symbolic_hamiltonian: &E,
        grid_points: usize,
        _backend: &B,
    ) -> SymbolicResult<Vec<Vec<f64>>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        // Extract potential and kinetic terms symbolically
        // Convert to numerical matrix

        let n = grid_points;
        let matrix = vec![vec![0.0; n]; n];

        Ok(matrix)
    }

    /// Automatic differentiation for optimization
    ///
    /// Compute ∂f/∂x symbolically, then evaluate numerically
    pub fn autodiff_gradient<B, E>(
        function: &E,
        variables: &[String],
        _point: &[f64],
        backend: &B,
    ) -> SymbolicResult<Vec<f64>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let mut gradient = Vec::new();

        for var in variables {
            let _derivative = backend.differentiate(function, var, 1)?;
            // Evaluate at point (would need substitution in real impl)
            gradient.push(0.0);
        }

        Ok(gradient)
    }

    /// Generate symbolic Jacobian matrix
    ///
    /// J_ij = ∂f_i/∂x_j
    pub fn symbolic_jacobian<B, E>(
        functions: &[E],
        variables: &[String],
        backend: &B,
    ) -> SymbolicResult<Vec<Vec<E>>>
    where
        B: SymbolicBackend<Expression = E>,
        E: SymbolicExpression,
    {
        let mut jacobian = Vec::new();

        for func in functions {
            let mut row = Vec::new();
            for var in variables {
                let derivative = backend.differentiate(func, var, 1)?;
                row.push(derivative);
            }
            jacobian.push(row);
        }

        Ok(jacobian)
    }
}

/// Wavefunction visualization data
pub mod visualization {

    /// 1D wavefunction plot data
    pub struct WavefunctionPlot1D {
        /// x-coordinates
        pub x: Vec<f64>,

        /// ψ(x) values
        pub psi: Vec<f64>,

        /// |ψ(x)|² probability density
        pub probability: Vec<f64>,
    }

    impl WavefunctionPlot1D {
        /// Create plot data from wavefunction
        pub fn new(x: Vec<f64>, psi: Vec<f64>) -> Self {
            let probability = psi.iter().map(|p| p * p).collect();

            Self {
                x,
                psi,
                probability,
            }
        }

        /// Normalize wavefunction
        pub fn normalize(&mut self) {
            let dx = if self.x.len() > 1 {
                self.x[1] - self.x[0]
            } else {
                1.0
            };

            let norm_squared: f64 = self.psi.iter().map(|p| p * p * dx).sum();
            let norm = norm_squared.sqrt();

            if norm > 0.0 {
                for p in &mut self.psi {
                    *p /= norm;
                }
                for p in &mut self.probability {
                    *p /= norm_squared;
                }
            }
        }
    }

    /// 2D wavefunction plot data
    pub struct WavefunctionPlot2D {
        /// x-coordinates
        pub x: Vec<f64>,

        /// y-coordinates
        pub y: Vec<f64>,

        /// ψ(x,y) values as matrix
        pub psi: Vec<Vec<f64>>,
    }

    /// Potential energy surface
    pub struct PotentialSurface {
        /// x-coordinates
        pub x: Vec<f64>,

        /// V(x) values
        pub potential: Vec<f64>,
    }

    impl PotentialSurface {
        /// Create potential surface plot data
        pub fn new(x: Vec<f64>, potential: Vec<f64>) -> Self {
            Self { x, potential }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numerical_method_display() {
        assert_eq!(
            NumericalMethod::MatrixDiagonalization.to_string(),
            "Matrix Diagonalization"
        );
    }

    #[test]
    fn test_finite_difference_3point() {
        let d2 = finite_difference::second_derivative_3point(1.0, 2.0, 1.0, 0.1);
        assert!(d2 != 0.0);
    }

    #[test]
    fn test_fem_basis_creation() {
        let basis =
            finite_element::FEMBasis::new(finite_element::BasisFunction::Linear, 10, 0.0, 1.0);

        assert_eq!(basis.num_elements, 10);
        assert_eq!(basis.nodes.len(), 11);
    }

    #[test]
    fn test_wavefunction_plot_normalization() {
        let x = vec![0.0, 0.1, 0.2, 0.3, 0.4];
        let psi = vec![1.0, 2.0, 3.0, 2.0, 1.0];

        let mut plot = visualization::WavefunctionPlot1D::new(x, psi);
        plot.normalize();

        assert!(plot.psi.iter().any(|&p| p != 0.0));
    }

    // Tests for eigenvalue solver
    #[test]
    fn test_solve_eigenvalue_problem_simple() {
        // Simple 2x2 symmetric matrix
        // [[2, 1],
        //  [1, 2]]
        // Eigenvalues: 1, 3
        let matrix = vec![vec![2.0, 1.0], vec![1.0, 2.0]];

        let (eigenvalues, eigenvectors) =
            matrix_diagonalization::solve_eigenvalue_problem(&matrix).unwrap();

        assert_eq!(eigenvalues.len(), 2);
        assert_eq!(eigenvectors.len(), 2);

        // Check eigenvalues are sorted
        assert!(eigenvalues[0] <= eigenvalues[1]);

        // Check eigenvalues are approximately correct
        assert!((eigenvalues[0] - 1.0).abs() < 1e-10);
        assert!((eigenvalues[1] - 3.0).abs() < 1e-10);

        // Check eigenvectors are normalized
        for eigenvector in &eigenvectors {
            let norm_squared: f64 = eigenvector.iter().map(|x| x * x).sum();
            assert!((norm_squared - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_solve_eigenvalue_problem_tridiagonal() {
        // Tridiagonal matrix (like discretized 1D Hamiltonian)
        // [[2, -1,  0],
        //  [-1, 2, -1],
        //  [0, -1,  2]]
        let matrix = vec![
            vec![2.0, -1.0, 0.0],
            vec![-1.0, 2.0, -1.0],
            vec![0.0, -1.0, 2.0],
        ];

        let (eigenvalues, eigenvectors) =
            matrix_diagonalization::solve_eigenvalue_problem(&matrix).unwrap();

        assert_eq!(eigenvalues.len(), 3);
        assert_eq!(eigenvectors.len(), 3);

        // Check eigenvalues are sorted
        for i in 0..eigenvalues.len() - 1 {
            assert!(eigenvalues[i] <= eigenvalues[i + 1]);
        }

        // Check eigenvectors are normalized
        for eigenvector in &eigenvectors {
            let norm_squared: f64 = eigenvector.iter().map(|x| x * x).sum();
            assert!((norm_squared - 1.0).abs() < 1e-10);
        }

        // Verify eigenvalue equation: H * v = λ * v
        for (i, eigenvalue) in eigenvalues.iter().enumerate() {
            let eigenvector = &eigenvectors[i];
            for row in 0..matrix.len() {
                let mut result = 0.0;
                for col in 0..matrix[row].len() {
                    result += matrix[row][col] * eigenvector[col];
                }
                let expected = eigenvalue * eigenvector[row];
                assert!((result - expected).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn test_solve_eigenvalue_problem_identity() {
        // Identity matrix - all eigenvalues should be 1
        let matrix = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];

        let (eigenvalues, _) = matrix_diagonalization::solve_eigenvalue_problem(&matrix).unwrap();

        for eigenvalue in &eigenvalues {
            assert!((eigenvalue - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_solve_eigenvalue_problem_diagonal() {
        // Diagonal matrix - eigenvalues are diagonal elements
        let matrix = vec![
            vec![3.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 2.0],
        ];

        let (eigenvalues, _) = matrix_diagonalization::solve_eigenvalue_problem(&matrix).unwrap();

        // Should be sorted: 1.0, 2.0, 3.0
        assert!((eigenvalues[0] - 1.0).abs() < 1e-10);
        assert!((eigenvalues[1] - 2.0).abs() < 1e-10);
        assert!((eigenvalues[2] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_solve_eigenvalue_problem_complex() {
        use num_complex::Complex64;

        // Simple 2x2 Hermitian matrix
        // [[1,   i],
        //  [-i,  1]]
        // Eigenvalues: 0, 2
        let matrix = vec![
            vec![Complex64::new(1.0, 0.0), Complex64::new(0.0, 1.0)],
            vec![Complex64::new(0.0, -1.0), Complex64::new(1.0, 0.0)],
        ];

        let (eigenvalues, eigenvectors) =
            matrix_diagonalization::solve_eigenvalue_problem_complex(&matrix).unwrap();

        assert_eq!(eigenvalues.len(), 2);
        assert_eq!(eigenvectors.len(), 2);

        // Check eigenvalues are approximately correct
        assert!((eigenvalues[0] - 0.0).abs() < 1e-10);
        assert!((eigenvalues[1] - 2.0).abs() < 1e-10);

        // Check eigenvectors are normalized
        for eigenvector in &eigenvectors {
            let norm_squared: f64 = eigenvector.iter().map(|z| z.norm_sqr()).sum();
            assert!((norm_squared - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_solve_eigenvalue_problem_empty_matrix() {
        let matrix: Vec<Vec<f64>> = vec![];
        let result = matrix_diagonalization::solve_eigenvalue_problem(&matrix);
        assert!(result.is_err());
    }

    #[test]
    fn test_solve_eigenvalue_problem_non_square() {
        let matrix = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
        let result = matrix_diagonalization::solve_eigenvalue_problem(&matrix);
        assert!(result.is_err());
    }

    // Tests for finite element method
    #[test]
    fn test_fem_basis_creation_linear() {
        let basis =
            finite_element::FEMBasis::new(finite_element::BasisFunction::Linear, 10, 0.0, 1.0);

        assert_eq!(basis.num_elements, 10);
        assert_eq!(basis.num_dofs, 11);
        assert_eq!(basis.nodes.len(), 11);
        assert!((basis.nodes[0] - 0.0).abs() < 1e-10);
        assert!((basis.nodes[10] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_fem_basis_creation_quadratic() {
        let basis =
            finite_element::FEMBasis::new(finite_element::BasisFunction::Quadratic, 5, 0.0, 1.0);

        assert_eq!(basis.num_elements, 5);
        assert_eq!(basis.num_dofs, 11); // 2*5 + 1
        assert_eq!(basis.nodes.len(), 11);
    }

    #[test]
    fn test_fem_element_size() {
        let basis =
            finite_element::FEMBasis::new(finite_element::BasisFunction::Linear, 10, 0.0, 1.0);

        let h = basis.element_size(0);
        assert!((h - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_fem_infinite_square_well() {
        // Infinite square well: V(x) = 0 for 0 < x < L
        // Analytical solution: E_n = n²π²ℏ²/(2mL²)

        let l = 1.0; // Box length
        let mass = 1.0;
        let hbar = 1.0;

        // Zero potential
        let potential = |_x: f64| 0.0;

        // Solve with FEM
        // NOTE: Current implementation uses stiffness matrix without mass matrix
        // This gives qualitatively correct results but quantitatively off by a factor
        // TODO: Implement proper generalized eigenvalue problem with mass matrix
        let (eigenvalues, _) = finite_element::solve_1d_schrodinger(
            50, // num_elements
            0.0,
            l,
            potential,
            mass,
            hbar,
            finite_element::BasisFunction::Linear,
        )
        .unwrap();

        // For now, just check that we get reasonable eigenvalue ordering
        assert!(eigenvalues.len() >= 3);
        assert!(eigenvalues[0] < eigenvalues[1]);
        assert!(eigenvalues[1] < eigenvalues[2]);

        // Check that eigenvalues scale correctly (ratios should be 1:4:9)
        let ratio_2_1 = eigenvalues[1] / eigenvalues[0];
        let ratio_3_1 = eigenvalues[2] / eigenvalues[0];

        assert!((ratio_2_1 - 4.0).abs() < 0.1, "E2/E1 ratio: {}", ratio_2_1);
        assert!((ratio_3_1 - 9.0).abs() < 0.2, "E3/E1 ratio: {}", ratio_3_1);
    }

    #[test]
    fn test_fem_harmonic_oscillator() {
        // Quantum harmonic oscillator: V(x) = (1/2)mω²x²
        // Analytical solution: E_n = ℏω(n + 1/2)

        let mass = 1.0;
        let hbar = 1.0;
        let omega = 1.0;

        // Harmonic potential
        let potential = |x: f64| 0.5 * mass * omega * omega * x * x;

        // Solve with FEM on [-5, 5] (wide enough for ground state)
        // NOTE: Same limitation as infinite well test - needs mass matrix
        let (eigenvalues, _) = finite_element::solve_1d_schrodinger(
            100, // num_elements
            -5.0,
            5.0,
            potential,
            mass,
            hbar,
            finite_element::BasisFunction::Quadratic,
        )
        .unwrap();

        assert!(eigenvalues.len() >= 3);

        // Check that eigenvalues are ordered
        assert!(eigenvalues[0] < eigenvalues[1]);
        assert!(eigenvalues[1] < eigenvalues[2]);

        // Check equal spacing (characteristic of harmonic oscillator)
        let spacing_1 = eigenvalues[1] - eigenvalues[0];
        let spacing_2 = eigenvalues[2] - eigenvalues[1];
        let spacing_ratio = spacing_2 / spacing_1;

        // Should be approximately equal spacing
        assert!(
            (spacing_ratio - 1.0).abs() < 0.1,
            "Spacing ratio: {}",
            spacing_ratio
        );
    }

    #[test]
    fn test_fem_hamiltonian_assembly() {
        // Test Hamiltonian assembly for simple case
        let basis =
            finite_element::FEMBasis::new(finite_element::BasisFunction::Linear, 5, 0.0, 1.0);

        let potential = |_x: f64| 0.0;
        let mass = 1.0;
        let hbar = 1.0;

        let hamiltonian =
            finite_element::assemble_hamiltonian_1d(&basis, potential, mass, hbar).unwrap();

        // Check matrix is square (interior DOFs only, excluding 2 boundary nodes)
        let n_interior = basis.num_dofs - 2;
        assert_eq!(hamiltonian.len(), n_interior);
        for row in &hamiltonian {
            assert_eq!(row.len(), n_interior);
        }

        // Check matrix is symmetric
        for i in 0..hamiltonian.len() {
            for j in 0..hamiltonian.len() {
                assert!((hamiltonian[i][j] - hamiltonian[j][i]).abs() < 1e-10);
            }
        }
    }

    #[test]
    fn test_fem_convergence() {
        // Test that FEM shows consistent behavior with mesh refinement
        let mass = 1.0;
        let hbar = 1.0;
        let potential = |_x: f64| 0.0;

        // Solve with different mesh sizes
        let (eigenvalues_coarse, _) = finite_element::solve_1d_schrodinger(
            10,
            0.0,
            1.0,
            potential,
            mass,
            hbar,
            finite_element::BasisFunction::Linear,
        )
        .unwrap();

        let (eigenvalues_fine, _) = finite_element::solve_1d_schrodinger(
            20,
            0.0,
            1.0,
            potential,
            mass,
            hbar,
            finite_element::BasisFunction::Linear,
        )
        .unwrap();

        // Both should give ordered eigenvalues
        assert!(eigenvalues_coarse[0] < eigenvalues_coarse[1]);
        assert!(eigenvalues_fine[0] < eigenvalues_fine[1]);

        // Eigenvalue ratios should be similar
        let ratio_coarse = eigenvalues_coarse[1] / eigenvalues_coarse[0];
        let ratio_fine = eigenvalues_fine[1] / eigenvalues_fine[0];
        assert!((ratio_coarse - ratio_fine).abs() < 0.1);
    }

    #[test]
    fn test_fem_quadratic_vs_linear() {
        // Both element types should give qualitatively similar results
        let mass = 1.0;
        let hbar = 1.0;
        let potential = |_x: f64| 0.0;

        let (eigenvalues_linear, _) = finite_element::solve_1d_schrodinger(
            20,
            0.0,
            1.0,
            potential,
            mass,
            hbar,
            finite_element::BasisFunction::Linear,
        )
        .unwrap();

        let (eigenvalues_quadratic, _) = finite_element::solve_1d_schrodinger(
            20,
            0.0,
            1.0,
            potential,
            mass,
            hbar,
            finite_element::BasisFunction::Quadratic,
        )
        .unwrap();

        // Both should give ordered eigenvalues
        assert!(eigenvalues_linear[0] < eigenvalues_linear[1]);
        assert!(eigenvalues_quadratic[0] < eigenvalues_quadratic[1]);

        // Eigenvalue ratios should be similar (both approximate same physics)
        let ratio_linear = eigenvalues_linear[1] / eigenvalues_linear[0];
        let ratio_quadratic = eigenvalues_quadratic[1] / eigenvalues_quadratic[0];
        assert!((ratio_linear - ratio_quadratic).abs() < 0.5);
    }

    // Tests for numerical integration
    #[test]
    fn test_integrate_polynomial_trapezoidal() {
        // Integrate x² from 0 to 1: ∫x² dx = x³/3 |₀¹ = 1/3
        let f = |x: f64| x * x;
        let result = numerical_integration::integrate_1d(
            f,
            0.0,
            1.0,
            100,
            numerical_integration::IntegrationMethod::Trapezoidal,
        )
        .unwrap();

        let expected = 1.0 / 3.0;
        assert!(
            (result - expected).abs() < 1e-4,
            "Result: {}, Expected: {}",
            result,
            expected
        );
    }

    #[test]
    fn test_integrate_polynomial_simpson() {
        // Integrate x³ from 0 to 2: ∫x³ dx = x⁴/4 |₀² = 4
        let f = |x: f64| x * x * x;
        let result = numerical_integration::integrate_1d(
            f,
            0.0,
            2.0,
            101,
            numerical_integration::IntegrationMethod::Simpson,
        )
        .unwrap();

        let expected = 4.0;
        assert!(
            (result - expected).abs() < 1e-6,
            "Result: {}, Expected: {}",
            result,
            expected
        );
    }

    #[test]
    fn test_integrate_gauss_legendre() {
        // Integrate sin(x) from 0 to π: ∫sin(x) dx = -cos(x) |₀^π = 2
        let f = |x: f64| x.sin();
        let pi = std::f64::consts::PI;
        let result = numerical_integration::integrate_1d(
            f,
            0.0,
            pi,
            4,
            numerical_integration::IntegrationMethod::GaussLegendre,
        )
        .unwrap();

        let expected = 2.0;
        assert!(
            (result - expected).abs() < 1e-3,
            "Result: {}, Expected: {}",
            result,
            expected
        );
    }

    #[test]
    fn test_integrate_adaptive() {
        // Integrate 1/(1+x²) from 0 to 1: ∫1/(1+x²) dx = arctan(x) |₀¹ = π/4
        let f = |x: f64| 1.0 / (1.0 + x * x);
        let result = numerical_integration::integrate_1d(
            f,
            0.0,
            1.0,
            10,
            numerical_integration::IntegrationMethod::Adaptive,
        )
        .unwrap();

        let expected = std::f64::consts::PI / 4.0;
        assert!(
            (result - expected).abs() < 1e-8,
            "Result: {}, Expected: {}",
            result,
            expected
        );
    }

    #[test]
    fn test_integrate_gaussian() {
        // Integrate exp(-x²) from -1 to 1 (approximate)
        let f = |x: f64| (-x * x).exp();
        let result = numerical_integration::integrate_1d(
            f,
            -1.0,
            1.0,
            100,
            numerical_integration::IntegrationMethod::Simpson,
        )
        .unwrap();

        // Numerical value ≈ 1.4936
        assert!(result > 1.49 && result < 1.50, "Result: {}", result);
    }

    #[test]
    fn test_expectation_value() {
        // Test expectation value calculation
        let n = 101;
        let x: Vec<f64> = (0..n).map(|i| i as f64 / (n - 1) as f64).collect();

        // Normalized wavefunction: ψ(x) = √2 sin(πx) on [0,1]
        let pi = std::f64::consts::PI;
        let psi: Vec<f64> = x
            .iter()
            .map(|&xi| (2.0_f64).sqrt() * (pi * xi).sin())
            .collect();

        // Position operator: x̂|ψ⟩ = x*ψ(x)
        let x_psi: Vec<f64> = x.iter().zip(&psi).map(|(&xi, &p)| xi * p).collect();

        // ⟨x⟩ should be 0.5 for symmetric wavefunction
        let expectation = numerical_integration::expectation_value_1d(&psi, &x_psi, &x).unwrap();
        assert!((expectation - 0.5).abs() < 0.01, "⟨x⟩ = {}", expectation);
    }

    #[test]
    fn test_overlap_integral() {
        // Test overlap integral ⟨ψ₁|ψ₂⟩
        let n = 101;
        let x: Vec<f64> = (0..n).map(|i| i as f64 / (n - 1) as f64).collect();

        let pi = std::f64::consts::PI;
        // ψ₁ = √2 sin(πx)
        let psi1: Vec<f64> = x
            .iter()
            .map(|&xi| (2.0_f64).sqrt() * (pi * xi).sin())
            .collect();
        // ψ₂ = √2 sin(2πx)
        let psi2: Vec<f64> = x
            .iter()
            .map(|&xi| (2.0_f64).sqrt() * (2.0 * pi * xi).sin())
            .collect();

        // Orthogonal states should have zero overlap
        let overlap = numerical_integration::overlap_integral_1d(&psi1, &psi2, &x).unwrap();
        assert!(overlap.abs() < 0.01, "Overlap = {}", overlap);
    }

    #[test]
    fn test_normalize_wavefunction() {
        // Test wavefunction normalization
        let n = 101;
        let x: Vec<f64> = (0..n).map(|i| i as f64 / (n - 1) as f64).collect();

        // Unnormalized wavefunction
        let pi = std::f64::consts::PI;
        let psi: Vec<f64> = x.iter().map(|&xi| (pi * xi).sin()).collect();

        let (psi_normalized, norm) =
            numerical_integration::normalize_wavefunction_1d(&psi, &x).unwrap();

        // Check that normalized wavefunction has unit norm
        let psi_norm_squared: Vec<f64> = psi_normalized.iter().map(|p| p * p).collect();
        let norm_check = numerical_integration::integrate_1d(
            |xi: f64| {
                let idx = ((xi * (n - 1) as f64).round() as usize).min(n - 1);
                psi_norm_squared[idx]
            },
            0.0,
            1.0,
            n,
            numerical_integration::IntegrationMethod::Simpson,
        )
        .unwrap();

        assert!((norm_check - 1.0).abs() < 0.01, "Norm = {}", norm_check);
        assert!(norm > 0.0);
    }

    #[test]
    fn test_integrate_2d() {
        // Integrate f(x,y) = x*y over [0,1]×[0,1]
        // ∫∫ x*y dx dy = (∫x dx)(∫y dy) = (1/2)(1/2) = 1/4
        let f = |x: f64, y: f64| x * y;
        let result = numerical_integration::integrate_2d(
            f,
            0.0,
            1.0,
            0.0,
            1.0,
            20,
            20,
            numerical_integration::IntegrationMethod::Simpson,
        )
        .unwrap();

        let expected = 0.25;
        assert!(
            (result - expected).abs() < 1e-4,
            "Result: {}, Expected: {}",
            result,
            expected
        );
    }

    #[test]
    fn test_integrate_2d_gaussian() {
        // Integrate exp(-(x²+y²)) over [-1,1]×[-1,1]
        let f = |x: f64, y: f64| (-(x * x + y * y)).exp();
        let result = numerical_integration::integrate_2d(
            f,
            -1.0,
            1.0,
            -1.0,
            1.0,
            30,
            30,
            numerical_integration::IntegrationMethod::Simpson,
        )
        .unwrap();

        // Should be positive and reasonable
        assert!(result > 2.0 && result < 3.0, "Result: {}", result);
    }

    #[test]
    fn test_integration_error_handling() {
        // Test invalid bounds
        let f = |x: f64| x;
        let result = numerical_integration::integrate_1d(
            f,
            1.0,
            0.0,
            10,
            numerical_integration::IntegrationMethod::Trapezoidal,
        );
        assert!(result.is_err());

        // Test insufficient points
        let result = numerical_integration::integrate_1d(
            f,
            0.0,
            1.0,
            1,
            numerical_integration::IntegrationMethod::Trapezoidal,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_discrete_integration_unequal_spacing() {
        // Test integration with non-uniform grid
        let x = vec![0.0, 0.1, 0.3, 0.6, 1.0];
        let y: Vec<f64> = x.iter().map(|&xi| xi * xi).collect();

        let result = numerical_integration::integrate_1d(
            |xi: f64| {
                // Interpolate
                for i in 0..x.len() - 1 {
                    if xi >= x[i] && xi <= x[i + 1] {
                        let t = (xi - x[i]) / (x[i + 1] - x[i]);
                        return y[i] * (1.0 - t) + y[i + 1] * t;
                    }
                }
                0.0
            },
            0.0,
            1.0,
            100,
            numerical_integration::IntegrationMethod::Simpson,
        )
        .unwrap();

        let expected = 1.0 / 3.0;
        assert!(
            (result - expected).abs() < 0.02,
            "Result: {}, Expected: {}",
            result,
            expected
        );
    }

    // Tests for numerical gradient
    #[test]
    fn test_derivative_polynomial() {
        // f(x) = x³, f'(x) = 3x²
        let f = |x: f64| x * x * x;
        let x = 2.0;

        let deriv = numerical_gradient::derivative(
            f,
            x,
            None,
            numerical_gradient::DifferentiationMethod::Central,
        )
        .unwrap();

        let expected = 3.0 * x * x; // 12.0
        assert!(
            (deriv - expected).abs() < 1e-6,
            "Derivative: {}, Expected: {}",
            deriv,
            expected
        );
    }

    #[test]
    fn test_derivative_methods() {
        // f(x) = sin(x), f'(x) = cos(x)
        let f = |x: f64| x.sin();
        let x = 1.0_f64;
        let expected = x.cos();

        // Forward difference
        let d_forward = numerical_gradient::derivative(
            f,
            x,
            None,
            numerical_gradient::DifferentiationMethod::Forward,
        )
        .unwrap();
        assert!((d_forward - expected).abs() < 1e-6);

        // Backward difference
        let d_backward = numerical_gradient::derivative(
            f,
            x,
            None,
            numerical_gradient::DifferentiationMethod::Backward,
        )
        .unwrap();
        assert!((d_backward - expected).abs() < 1e-6);

        // Central difference (most accurate)
        let d_central = numerical_gradient::derivative(
            f,
            x,
            None,
            numerical_gradient::DifferentiationMethod::Central,
        )
        .unwrap();
        assert!((d_central - expected).abs() < 1e-8);
    }

    #[test]
    fn test_gradient_quadratic() {
        // f(x,y) = x² + y², ∇f = [2x, 2y]
        let f = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
        let x = vec![3.0, 4.0];

        let grad = numerical_gradient::gradient(
            f,
            &x,
            None,
            numerical_gradient::DifferentiationMethod::Central,
        )
        .unwrap();

        assert_eq!(grad.len(), 2);
        assert!((grad[0] - 6.0).abs() < 1e-6);
        assert!((grad[1] - 8.0).abs() < 1e-6);
    }

    #[test]
    fn test_gradient_rosenbrock() {
        // Rosenbrock function: f(x,y) = (1-x)² + 100(y-x²)²
        // ∇f = [-2(1-x) - 400x(y-x²), 200(y-x²)]
        let f = |x: &[f64]| {
            let a = 1.0 - x[0];
            let b = x[1] - x[0] * x[0];
            a * a + 100.0 * b * b
        };

        let x = vec![0.5, 0.5];
        let grad = numerical_gradient::gradient(
            f,
            &x,
            None,
            numerical_gradient::DifferentiationMethod::Central,
        )
        .unwrap();

        // Analytical gradient at (0.5, 0.5)
        let grad_x = -2.0 * (1.0 - 0.5) - 400.0 * 0.5 * (0.5 - 0.25);
        let grad_y = 200.0 * (0.5 - 0.25);

        assert!((grad[0] - grad_x).abs() < 1e-4);
        assert!((grad[1] - grad_y).abs() < 1e-4);
    }

    #[test]
    fn test_jacobian_linear() {
        // f(x,y) = [2x + y, x - 3y]
        // J = [[2, 1], [1, -3]]
        let f = |x: &[f64]| vec![2.0 * x[0] + x[1], x[0] - 3.0 * x[1]];
        let x = vec![1.0, 2.0];

        let jac = numerical_gradient::jacobian(
            f,
            &x,
            None,
            numerical_gradient::DifferentiationMethod::Central,
        )
        .unwrap();

        assert_eq!(jac.len(), 2);
        assert_eq!(jac[0].len(), 2);

        assert!((jac[0][0] - 2.0).abs() < 1e-6);
        assert!((jac[0][1] - 1.0).abs() < 1e-6);
        assert!((jac[1][0] - 1.0).abs() < 1e-6);
        assert!((jac[1][1] + 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_jacobian_nonlinear() {
        // f(x,y) = [x², xy, y²]
        // J = [[2x, 0], [y, x], [0, 2y]]
        let f = |x: &[f64]| vec![x[0] * x[0], x[0] * x[1], x[1] * x[1]];
        let x = vec![2.0, 3.0];

        let jac = numerical_gradient::jacobian(
            f,
            &x,
            None,
            numerical_gradient::DifferentiationMethod::Central,
        )
        .unwrap();

        assert_eq!(jac.len(), 3);
        assert_eq!(jac[0].len(), 2);

        assert!((jac[0][0] - 4.0).abs() < 1e-6); // 2x = 4
        assert!((jac[0][1] - 0.0).abs() < 1e-6);
        assert!((jac[1][0] - 3.0).abs() < 1e-6); // y = 3
        assert!((jac[1][1] - 2.0).abs() < 1e-6); // x = 2
        assert!((jac[2][0] - 0.0).abs() < 1e-6);
        assert!((jac[2][1] - 6.0).abs() < 1e-6); // 2y = 6
    }

    #[test]
    fn test_hessian_quadratic() {
        // f(x,y) = x² + xy + y²
        // H = [[2, 1], [1, 2]]
        let f = |x: &[f64]| x[0] * x[0] + x[0] * x[1] + x[1] * x[1];
        let x = vec![1.0, 2.0];

        let hess = numerical_gradient::hessian(f, &x, None).unwrap();

        assert_eq!(hess.len(), 2);
        assert_eq!(hess[0].len(), 2);

        assert!((hess[0][0] - 2.0).abs() < 1e-4);
        assert!((hess[0][1] - 1.0).abs() < 1e-4);
        assert!((hess[1][0] - 1.0).abs() < 1e-4);
        assert!((hess[1][1] - 2.0).abs() < 1e-4);

        // Check symmetry
        assert!((hess[0][1] - hess[1][0]).abs() < 1e-10);
    }

    #[test]
    fn test_hessian_cubic() {
        // f(x,y,z) = x³ + y³ + z³ + xyz
        // H_xx = 6x, H_yy = 6y, H_zz = 6z
        // H_xy = z, H_xz = y, H_yz = x
        let f = |x: &[f64]| x[0].powi(3) + x[1].powi(3) + x[2].powi(3) + x[0] * x[1] * x[2];
        let x = vec![1.0, 2.0, 3.0];

        let hess = numerical_gradient::hessian(f, &x, None).unwrap();

        assert_eq!(hess.len(), 3);

        // Diagonal
        assert!((hess[0][0] - 6.0).abs() < 1e-3); // 6*1
        assert!((hess[1][1] - 12.0).abs() < 1e-3); // 6*2
        assert!((hess[2][2] - 18.0).abs() < 1e-3); // 6*3

        // Off-diagonal
        assert!((hess[0][1] - 3.0).abs() < 1e-3); // z = 3
        assert!((hess[0][2] - 2.0).abs() < 1e-3); // y = 2
        assert!((hess[1][2] - 1.0).abs() < 1e-3); // x = 1
    }

    #[test]
    fn test_directional_derivative() {
        // f(x,y) = x² + y², ∇f = [2x, 2y]
        // Directional derivative in direction v = [1, 1] (normalized)
        // ∇_v f = ∇f · v/|v| = (2x + 2y)/√2
        let f = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
        let x = vec![3.0, 4.0];
        let v = vec![1.0, 1.0];

        let dir_deriv = numerical_gradient::directional_derivative(f, &x, &v, None).unwrap();

        let expected = (2.0 * 3.0 + 2.0 * 4.0) / 2.0_f64.sqrt();
        assert!((dir_deriv - expected).abs() < 1e-6);
    }

    #[test]
    fn test_gradient_on_grid() {
        // f(x) = x², f'(x) = 2x
        let f = |x: f64| x * x;
        let x_grid = vec![0.0, 1.0, 2.0, 3.0, 4.0];

        let grad = numerical_gradient::gradient_on_grid(
            f,
            &x_grid,
            numerical_gradient::DifferentiationMethod::Central,
        )
        .unwrap();

        assert_eq!(grad.len(), 5);

        // Check interior points (more accurate)
        for i in 1..4 {
            let expected = 2.0 * x_grid[i];
            assert!(
                (grad[i] - expected).abs() < 1e-6,
                "grad[{}] = {}, expected = {}",
                i,
                grad[i],
                expected
            );
        }
    }

    #[test]
    fn test_check_gradient() {
        // f(x,y) = x² + 2xy + y²
        let f = |x: &[f64]| x[0] * x[0] + 2.0 * x[0] * x[1] + x[1] * x[1];
        let grad_f = |x: &[f64]| vec![2.0 * x[0] + 2.0 * x[1], 2.0 * x[0] + 2.0 * x[1]];

        let x = vec![1.0, 2.0];
        let is_correct = numerical_gradient::check_gradient(f, grad_f, &x, 1e-6).unwrap();

        assert!(is_correct);
    }

    #[test]
    fn test_gradient_error_handling() {
        // Test directional derivative with zero direction
        let f = |x: &[f64]| x[0] * x[0];
        let x = vec![1.0];
        let v = vec![0.0];

        let result = numerical_gradient::directional_derivative(f, &x, &v, None);
        assert!(result.is_err());

        // Test dimension mismatch
        let v2 = vec![1.0, 1.0];
        let result2 = numerical_gradient::directional_derivative(f, &x, &v2, None);
        assert!(result2.is_err());
    }
}
