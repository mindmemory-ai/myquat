// Gate Decomposition Module
// Author: gA4ss
//
// This module provides gate decomposition algorithms, particularly the ZYZ decomposition
// for arbitrary single-qubit rotations and the decomposition of R(nx,ny,nz) gates.
//
// # Mathematical Background
//
// ## ZYZ Decomposition
//
// Any single-qubit unitary U can be decomposed as:
// $$ U = e^{i\alpha} R_z(\phi) R_y(\theta) R_z(\lambda) $$
//
// where:
// - $\alpha$ is the global phase
// - $\phi, \theta, \lambda$ are Euler angles
// - $R_z(\phi) = \begin{bmatrix} e^{-i\phi/2} & 0 \\ 0 & e^{i\phi/2} \end{bmatrix}$
// - $R_y(\theta) = \begin{bmatrix} \cos(\theta/2) & -\sin(\theta/2) \\ \sin(\theta/2) & \cos(\theta/2) \end{bmatrix}$
//
// ## Arbitrary Axis Rotation
//
// $$ R(\hat{n}, \theta) = \exp\left(-i\frac{\theta}{2} \hat{n} \cdot \vec{\sigma}\right) $$
//
// where $\hat{n} = (n_x, n_y, n_z)$ is a unit vector and $\vec{\sigma} = (\sigma_x, \sigma_y, \sigma_z)$ are Pauli matrices.
//
// Using Rodrigues' rotation formula:
// $$ R(\hat{n}, \theta) = \cos(\theta/2)I - i\sin(\theta/2)(n_x\sigma_x + n_y\sigma_y + n_z\sigma_z) $$

use crate::error::{MyQuatError, Result};
use crate::linalg::{LinalgBackend, LinalgResult, NdArrayBackend};
use ndarray::Array2;
use num_complex::Complex64;
use std::f64::consts::PI;

/// ZYZ Euler angles for single-qubit rotation
#[derive(Debug, Clone, Copy)]
pub struct ZYZAngles {
    /// First Z rotation angle $\phi$
    pub phi: f64,
    /// Y rotation angle $\theta$
    pub theta: f64,
    /// Second Z rotation angle $\lambda$
    pub lambda: f64,
    /// Global phase $\alpha$
    pub global_phase: f64,
}

impl ZYZAngles {
    /// Create new ZYZ angles
    pub fn new(phi: f64, theta: f64, lambda: f64) -> Self {
        ZYZAngles {
            phi,
            theta,
            lambda,
            global_phase: 0.0,
        }
    }

    /// Create ZYZ angles with global phase
    pub fn with_global_phase(phi: f64, theta: f64, lambda: f64, global_phase: f64) -> Self {
        ZYZAngles {
            phi,
            theta,
            lambda,
            global_phase,
        }
    }

    /// Normalize angles to standard ranges
    pub fn normalize(&mut self) {
        // Normalize $\theta$ to $[0, \pi]$ first
        if self.theta < 0.0 {
            self.theta = -self.theta;
            self.phi += PI;
            self.lambda += PI;
        }
        if self.theta > PI {
            self.theta = 2.0 * PI - self.theta;
            self.phi += PI;
            self.lambda += PI;
        }

        // Then normalize to $[0, 2\pi)$ for Z rotations
        self.phi = self.phi.rem_euclid(2.0 * PI);
        self.lambda = self.lambda.rem_euclid(2.0 * PI);

        // Normalize global phase to $[0, 2\pi)$
        self.global_phase = self.global_phase.rem_euclid(2.0 * PI);
    }
}

/// Decompose a single-qubit unitary matrix into ZYZ Euler angles
///
/// Given a 2x2 unitary matrix U, find angles $(\phi, \theta, \lambda, \alpha)$ such that:
/// $$ U = e^{i\alpha} R_z(\phi) R_y(\theta) R_z(\lambda) $$
///
/// # Arguments
///
/// * `matrix` - 2x2 unitary matrix to decompose
///
/// # Returns
///
/// ZYZ Euler angles
///
/// # Mathematical Details
///
/// The decomposition uses the following formulas:
/// - $\theta = 2 \arccos(|U_{00}|)$
/// - $\phi + \lambda = 2 \arg(U_{00})$
/// - $\phi - \lambda = 2 \arg(-U_{10})$
/// - $\alpha = \arg(\det(U))/2$
pub fn zyz_decomposition(matrix: &Array2<Complex64>) -> Result<ZYZAngles> {
    if matrix.nrows() != 2 || matrix.ncols() != 2 {
        return Err(MyQuatError::circuit_error(
            "Matrix must be 2x2 for single-qubit decomposition",
        ));
    }

    // Extract matrix elements
    let u00 = matrix[[0, 0]];
    let u01 = matrix[[0, 1]];
    let u10 = matrix[[1, 0]];
    let u11 = matrix[[1, 1]];

    // Calculate global phase from determinant
    let det = u00 * u11 - u01 * u10;
    let global_phase = det.arg() / 2.0;

    // Calculate theta from |U[0,0]|
    let abs_u00 = u00.norm();
    let theta = 2.0 * abs_u00.acos().min(PI);

    // Handle special cases
    if theta.abs() < 1e-10 {
        // $\theta \approx 0$: $U \approx e^{i\alpha} R_z(\phi+\lambda)$
        // u00 = e^{iα} e^{-i(φ+λ)/2} → φ+λ = -2·arg(u00) + 2α
        let phi_plus_lambda = -2.0 * u00.arg() + 2.0 * global_phase;
        return Ok(ZYZAngles::with_global_phase(
            phi_plus_lambda,
            0.0,
            0.0,
            global_phase,
        ));
    }

    if (theta - PI).abs() < 1e-10 {
        // $\theta \approx \pi$: $U \approx e^{i\alpha} R_z(\phi-\lambda) Y$
        let phi_minus_lambda = (-u10).arg();
        return Ok(ZYZAngles::with_global_phase(
            phi_minus_lambda,
            PI,
            0.0,
            global_phase,
        ));
    }

    // General case: extract angles from matrix elements
    // $U_{00} = e^{i\alpha} e^{-i(\phi+\lambda)/2} \cos(\theta/2)$
    // $U_{10} = e^{i\alpha} e^{i(\phi-\lambda)/2} \sin(\theta/2)$
    //
    // Therefore:
    // $\phi + \lambda = -2\arg(U_{00}) + 2\alpha$
    // $\phi - \lambda = 2\arg(U_{10}) - 2\alpha$

    let phi_plus_lambda = -2.0 * u00.arg() + 2.0 * global_phase;
    let phi_minus_lambda = 2.0 * u10.arg() - 2.0 * global_phase;

    let phi = (phi_plus_lambda + phi_minus_lambda) / 2.0;
    let lambda = (phi_plus_lambda - phi_minus_lambda) / 2.0;

    Ok(ZYZAngles::with_global_phase(
        phi,
        theta,
        lambda,
        global_phase,
    ))
}

/// U3 Euler angles for single-qubit rotation (no global phase).
///
/// The U3 gate covers all of U(2) with 3 real parameters:
/// $$ U3(\theta, \phi, \lambda) = R_z(\phi) R_y(\theta) R_z(\lambda) $$
#[derive(Debug, Clone, Copy)]
pub struct U3Angles {
    /// Y rotation angle $\theta \in [0, \pi]$
    pub theta: f64,
    /// First Z rotation angle $\phi$
    pub phi: f64,
    /// Second Z rotation angle $\lambda$
    pub lambda: f64,
}

impl U3Angles {
    /// Create new U3 angles.
    pub fn new(theta: f64, phi: f64, lambda: f64) -> Self {
        U3Angles { theta, phi, lambda }
    }
}

/// Decompose a single-qubit unitary matrix into U3 Euler angles.
///
/// Given a 2×2 unitary matrix $U$, find angles $(\theta, \phi, \lambda)$ such that:
/// $$ U = U3(\theta, \phi, \lambda) = R_z(\phi) R_y(\theta) R_z(\lambda) $$
///
/// Unlike ZYZ decomposition, U3 has **no global phase** — it exactly covers all of U(2).
/// This eliminates the `Rz(λ+α) ≠ e^{iα}·Rz(λ)` issue that makes ZYZ unsafe for
/// multi-qubit circuits.
///
/// # Arguments
///
/// * `matrix` - 2×2 unitary matrix to decompose
///
/// # Returns
///
/// `U3Angles` with $(\theta, \phi, \lambda)$
///
/// # Mathematical Details
///
/// The U3 matrix is:
/// $$ U3(\theta, \phi, \lambda) = \begin{pmatrix}
/// \cos\frac{\theta}{2} & -e^{i\lambda}\sin\frac{\theta}{2} \\
/// e^{i\phi}\sin\frac{\theta}{2} & e^{i(\phi+\lambda)}\cos\frac{\theta}{2}
/// \end{pmatrix} $$
///
/// From a generic unitary $U = \begin{pmatrix} a & b \\ c & d \end{pmatrix}$:
/// - $\theta = 2 \arctan(|c|, |a|)$
/// - $\phi = \arg(c)$ when $\sin(\theta/2) \neq 0$
/// - $\lambda = \arg(-b)$ when $\sin(\theta/2) \neq 0$
pub fn u3_decomposition(matrix: &Array2<Complex64>) -> Result<U3Angles> {
    if matrix.nrows() != 2 || matrix.ncols() != 2 {
        return Err(MyQuatError::circuit_error(
            "Matrix must be 2x2 for single-qubit U3 decomposition",
        ));
    }

    let a = matrix[[0, 0]];
    let b = matrix[[0, 1]];
    let c = matrix[[1, 0]];
    let d = matrix[[1, 1]];

    // U3 requires u00 = cos(θ/2) to be real and non-negative.
    // Extract global phase factor from u00 to normalize it.
    let phase = a.arg();
    let phase_correction = Complex64::new(phase.cos(), -phase.sin()); // e^{-i·phase}
    let a_real = (phase_correction * a).re; // now real and ≥ 0 (up to fp error)
    let b_norm = phase_correction * b;
    let c_norm = phase_correction * c;
    let d_norm = phase_correction * d;

    // θ = 2 * atan2(|c|, |a|) — use the real-valued a_real for stability
    let theta = 2.0 * c_norm.norm().atan2(a_real.abs());

    let eps = 1e-10;

    // Edge case: θ ≈ 0 → normalized matrix ≈ diag(1, e^{i(φ+λ)})
    // Only φ+λ matters; set φ = 0.
    if theta < eps {
        let lambda = d_norm.arg();
        return Ok(U3Angles::new(0.0, 0.0, lambda));
    }

    // Edge case: θ ≈ π → normalized matrix ≈ antidiag(−e^{iλ}, e^{iφ})
    if (theta - PI).abs() < eps {
        let phi = c_norm.arg();
        let lambda = (-b_norm).arg();
        return Ok(U3Angles::new(PI, phi, lambda));
    }

    // General case
    let phi = c_norm.arg();
    let lambda = (-b_norm).arg();

    Ok(U3Angles::new(theta, phi, lambda))
}

/// Decompose an arbitrary axis rotation R(nx, ny, nz, angle) into ZYZ form
///
/// # Arguments
///
/// * `nx`, `ny`, `nz` - Components of rotation axis (will be normalized)
/// * `angle` - Rotation angle in radians
///
/// # Returns
///
/// ZYZ Euler angles equivalent to the rotation
///
/// # Mathematical Details
///
/// The rotation matrix is:
/// R(n̂, θ) = cos(θ/2)I - i*sin(θ/2)(nx*σx + ny*σy + nz*σz)
///
/// This is then decomposed using the standard ZYZ decomposition.
pub fn arbitrary_rotation_to_zyz(nx: f64, ny: f64, nz: f64, angle: f64) -> Result<ZYZAngles> {
    // Normalize the axis
    let norm = (nx * nx + ny * ny + nz * nz).sqrt();
    if norm < 1e-10 {
        return Err(MyQuatError::circuit_error("Rotation axis must be non-zero"));
    }

    let nx_norm = nx / norm;
    let ny_norm = ny / norm;
    let nz_norm = nz / norm;

    // Construct the rotation matrix using Rodrigues formula
    let matrix = arbitrary_rotation_matrix(nx_norm, ny_norm, nz_norm, angle);

    // Decompose into ZYZ
    zyz_decomposition(&matrix)
}

/// Construct the matrix for rotation around arbitrary axis
///
/// $$ R(\hat{n}, \theta) = \cos(\theta/2)I - i\sin(\theta/2)(n_x\sigma_x + n_y\sigma_y + n_z\sigma_z) $$
///
/// # Arguments
///
/// * `nx`, `ny`, `nz` - Components of normalized rotation axis
/// * `angle` - Rotation angle in radians
///
/// # Returns
///
/// 2x2 unitary matrix representing the rotation
pub fn arbitrary_rotation_matrix(nx: f64, ny: f64, nz: f64, angle: f64) -> Array2<Complex64> {
    let cos_half = (angle / 2.0).cos();
    let sin_half = (angle / 2.0).sin();

    let mut matrix = Array2::zeros((2, 2));

    // $R = \cos(\theta/2)I - i\sin(\theta/2)(n_x\sigma_x + n_y\sigma_y + n_z\sigma_z)$
    // where $\sigma_x = \begin{bmatrix}0&1\\1&0\end{bmatrix}$, $\sigma_y = \begin{bmatrix}0&-i\\i&0\end{bmatrix}$, $\sigma_z = \begin{bmatrix}1&0\\0&-1\end{bmatrix}$

    // Diagonal elements
    matrix[[0, 0]] = Complex64::new(cos_half, -sin_half * nz);
    matrix[[1, 1]] = Complex64::new(cos_half, sin_half * nz);

    // Off-diagonal elements
    matrix[[0, 1]] = Complex64::new(-sin_half * ny, -sin_half * nx);
    matrix[[1, 0]] = Complex64::new(sin_half * ny, -sin_half * nx);

    matrix
}

/// Construct ZYZ rotation matrix from Euler angles
///
/// U = e^(iα) Rz(φ) Ry(θ) Rz(λ)
///
/// # Arguments
///
/// * `angles` - ZYZ Euler angles
///
/// # Returns
///
/// 2x2 unitary matrix
pub fn zyz_to_matrix(angles: &ZYZAngles) -> Array2<Complex64> {
    let phi = angles.phi;
    let theta = angles.theta;
    let lambda = angles.lambda;
    let alpha = angles.global_phase;

    let cos_half = (theta / 2.0).cos();
    let sin_half = (theta / 2.0).sin();

    // Global phase factor
    let global_factor = Complex64::new(0.0, alpha).exp();

    let mut matrix = Array2::zeros((2, 2));

    // $U_{00} = e^{i\alpha} \cdot e^{-i(\phi+\lambda)/2} \cdot \cos(\theta/2)$
    let phase_00 = Complex64::new(0.0, -(phi + lambda) / 2.0).exp();
    matrix[[0, 0]] = global_factor * phase_00 * cos_half;

    // $U_{01} = e^{i\alpha} \cdot e^{-i(\phi-\lambda)/2} \cdot (-\sin(\theta/2))$
    let phase_01 = Complex64::new(0.0, -(phi - lambda) / 2.0).exp();
    matrix[[0, 1]] = global_factor * phase_01 * (-sin_half);

    // $U_{10} = e^{i\alpha} \cdot e^{i(\phi-\lambda)/2} \cdot \sin(\theta/2)$
    let phase_10 = Complex64::new(0.0, (phi - lambda) / 2.0).exp();
    matrix[[1, 0]] = global_factor * phase_10 * sin_half;

    // $U_{11} = e^{i\alpha} \cdot e^{i(\phi+\lambda)/2} \cdot \cos(\theta/2)$
    let phase_11 = Complex64::new(0.0, (phi + lambda) / 2.0).exp();
    matrix[[1, 1]] = global_factor * phase_11 * cos_half;

    matrix
}

/// Compute the ZYZ matrix using a linear algebra backend
pub fn zyz_to_matrix_with_backend<B: LinalgBackend<Scalar = Complex64>>(
    angles: &ZYZAngles,
    backend: &B,
) -> LinalgResult<B::Matrix> {
    let nda = zyz_to_matrix(angles);
    let (r, c) = nda.dim();
    let data: Vec<Complex64> = nda.iter().copied().collect();
    backend.from_shape_vec(r, c, data)
}

/// ZYZ decomposition using a linear algebra backend
pub fn zyz_decomposition_with_backend<B: LinalgBackend<Scalar = Complex64>>(
    matrix: &B::Matrix,
    backend: &B,
) -> LinalgResult<ZYZAngles> {
    let u00 = backend.get_matrix(matrix, 0, 0)?;
    let u01 = backend.get_matrix(matrix, 0, 1)?;
    let u10 = backend.get_matrix(matrix, 1, 0)?;
    let u11 = backend.get_matrix(matrix, 1, 1)?;
    let det = u00 * u11 - u01 * u10;
    let global_phase = det.arg() / 2.0;
    let theta = 2.0 * u00.norm().acos();
    let phi_plus_lambda = 2.0 * u00.arg();
    let phi_minus_lambda = 2.0 * (-u10).arg();
    let phi = (phi_plus_lambda + phi_minus_lambda) / 2.0;
    let lambda = (phi_plus_lambda - phi_minus_lambda) / 2.0;
    Ok(ZYZAngles::with_global_phase(
        phi,
        theta,
        lambda,
        global_phase,
    ))
}

/// Verify that a ZYZ decomposition is correct
///
/// Checks that the reconstructed matrix matches the original within tolerance
/// Note: This function accounts for global phase differences
pub fn verify_zyz_decomposition(
    original: &Array2<Complex64>,
    angles: &ZYZAngles,
    tolerance: f64,
) -> bool {
    let reconstructed = zyz_to_matrix(angles);

    // First try direct comparison
    let mut direct_match = true;
    for i in 0..2 {
        for j in 0..2 {
            let diff = (original[[i, j]] - reconstructed[[i, j]]).norm();
            if diff > tolerance {
                direct_match = false;
                break;
            }
        }
        if !direct_match {
            break;
        }
    }

    if direct_match {
        return true;
    }

    // Try with global phase correction
    // Find the global phase from first non-zero element
    let mut global_phase = Complex64::new(1.0, 0.0);
    for i in 0..2 {
        for j in 0..2 {
            if original[[i, j]].norm() > 1e-10 && reconstructed[[i, j]].norm() > 1e-10 {
                global_phase = original[[i, j]] / reconstructed[[i, j]];
                break;
            }
        }
    }

    // Check with global phase correction
    for i in 0..2 {
        for j in 0..2 {
            let corrected = reconstructed[[i, j]] * global_phase;
            let diff = (original[[i, j]] - corrected).norm();
            if diff > tolerance {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_zyz_identity() {
        // Identity matrix should decompose to zero angles
        let identity = Array2::eye(2).mapv(|x| Complex64::new(x, 0.0));
        let angles = zyz_decomposition(&identity).unwrap();

        assert!(angles.theta.abs() < 1e-10);
    }

    #[test]
    fn test_zyz_pauli_x() {
        // Pauli X = [[0, 1], [1, 0]]
        let mut pauli_x = Array2::zeros((2, 2));
        pauli_x[[0, 1]] = Complex64::new(1.0, 0.0);
        pauli_x[[1, 0]] = Complex64::new(1.0, 0.0);

        let angles = zyz_decomposition(&pauli_x).unwrap();

        // $X = R_z(\pi) R_y(\pi) R_z(0)$ (up to global phase)
        assert!((angles.theta - PI).abs() < 1e-10);
    }

    #[test]
    fn test_zyz_hadamard() {
        // Hadamard = 1/√2 * [[1, 1], [1, -1]]
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        let mut hadamard = Array2::zeros((2, 2));
        hadamard[[0, 0]] = Complex64::new(inv_sqrt2, 0.0);
        hadamard[[0, 1]] = Complex64::new(inv_sqrt2, 0.0);
        hadamard[[1, 0]] = Complex64::new(inv_sqrt2, 0.0);
        hadamard[[1, 1]] = Complex64::new(-inv_sqrt2, 0.0);

        let angles = zyz_decomposition(&hadamard).unwrap();

        // Verify reconstruction (relaxed tolerance for numerical precision)
        // Note: Global phase differences are acceptable
        assert!(verify_zyz_decomposition(&hadamard, &angles, 1e-6));
    }

    #[test]
    fn test_arbitrary_rotation_z_axis() {
        // Rotation around Z axis should give simple ZYZ
        let angle = PI / 4.0;
        let angles = arbitrary_rotation_to_zyz(0.0, 0.0, 1.0, angle).unwrap();

        // Should be $R_z(\text{angle})$ with $\theta \approx 0$
        assert!(angles.theta.abs() < 1e-10);
    }

    #[test]
    fn test_arbitrary_rotation_x_axis() {
        // Rotation around X axis
        let angle = PI / 3.0;
        let angles = arbitrary_rotation_to_zyz(1.0, 0.0, 0.0, angle).unwrap();

        // Verify by reconstructing matrix
        let matrix = arbitrary_rotation_matrix(1.0, 0.0, 0.0, angle);
        assert!(verify_zyz_decomposition(&matrix, &angles, 1e-6));
    }

    #[test]
    fn test_arbitrary_rotation_general() {
        // General rotation around (1,1,1) axis
        let angle = PI / 6.0;
        let angles = arbitrary_rotation_to_zyz(1.0, 1.0, 1.0, angle).unwrap();

        // Verify reconstruction
        let matrix = arbitrary_rotation_matrix(
            1.0 / 3.0_f64.sqrt(),
            1.0 / 3.0_f64.sqrt(),
            1.0 / 3.0_f64.sqrt(),
            angle,
        );
        assert!(verify_zyz_decomposition(&matrix, &angles, 1e-6));
    }

    #[test]
    fn test_zyz_roundtrip() {
        // Test that ZYZ -> matrix -> ZYZ gives same angles
        let original_angles = ZYZAngles::new(PI / 4.0, PI / 3.0, PI / 6.0);
        let matrix = zyz_to_matrix(&original_angles);
        let decomposed_angles = zyz_decomposition(&matrix).unwrap();

        // Angles might differ by $2\pi$ and global phase, so use verification function
        assert!(
            verify_zyz_decomposition(&matrix, &decomposed_angles, 1e-6),
            "Roundtrip decomposition failed"
        );
    }

    #[test]
    fn test_angle_normalization() {
        let mut angles = ZYZAngles::new(3.0 * PI, -PI / 4.0, 5.0 * PI);
        angles.normalize();

        assert!(angles.phi >= 0.0 && angles.phi < 2.0 * PI);
        assert!(angles.theta >= 0.0 && angles.theta <= PI);
        assert!(angles.lambda >= 0.0 && angles.lambda < 2.0 * PI);
    }

    // --- U3 Decomposition Tests ---

    fn u3_to_matrix(angles: &U3Angles) -> Array2<Complex64> {
        let c = Complex64::new;
        let t2 = angles.theta / 2.0;
        let cos = c(t2.cos(), 0.0);
        let sin = c(t2.sin(), 0.0);
        let e_iphi = c(angles.phi.cos(), angles.phi.sin());
        let e_ilambda = c(angles.lambda.cos(), angles.lambda.sin());
        let e_isum = c(
            (angles.phi + angles.lambda).cos(),
            (angles.phi + angles.lambda).sin(),
        );

        let mut m = Array2::zeros((2, 2));
        // U3 = [[cos(θ/2), -e^{iλ}sin(θ/2)], [e^{iφ}sin(θ/2), e^{i(φ+λ)}cos(θ/2)]]
        m[[0, 0]] = cos;
        m[[0, 1]] = -e_ilambda * sin;
        m[[1, 0]] = e_iphi * sin;
        m[[1, 1]] = e_isum * cos;
        m
    }

    /// Verify U3 reconstruction matches original up to global phase.
    /// Multiplies original by e^{-i·arg(u00)} before comparing, since U3
    /// drops the global phase (which is unobservable for single-qubit ops).
    fn verify_u3_decomposition(original: &Array2<Complex64>, angles: &U3Angles, tol: f64) -> bool {
        let reconstructed = u3_to_matrix(angles);
        // Normalize phase: multiply both by e^{-i·arg(original[0,0])}
        let phase = original[[0, 0]].arg();
        let norm = Complex64::new(phase.cos(), -phase.sin());
        let orig_norm = Array2::from_shape_vec(
            (2, 2),
            (0..4)
                .map(|k| {
                    let i = k / 2;
                    let j = k % 2;
                    norm * original[[i, j]]
                })
                .collect(),
        )
        .unwrap();
        let diff = orig_norm - &reconstructed;
        let frob: f64 = diff.iter().map(|x| x.norm_sqr()).sum::<f64>().sqrt();
        frob < tol
    }

    #[test]
    fn test_u3_identity() {
        let identity = Array2::eye(2).mapv(|x| Complex64::new(x, 0.0));
        let angles = u3_decomposition(&identity).unwrap();

        assert!(angles.theta.abs() < 1e-10, "θ should be 0 for identity");
        // θ=0: φ+λ = arg(d) = 0, we set φ=0, λ=0
        assert!((angles.phi + angles.lambda).abs() < 1e-10);
    }

    #[test]
    fn test_u3_pauli_x() {
        // Pauli X = [[0, 1], [1, 0]]
        let mut pauli_x = Array2::zeros((2, 2));
        pauli_x[[0, 1]] = Complex64::new(1.0, 0.0);
        pauli_x[[1, 0]] = Complex64::new(1.0, 0.0);

        let angles = u3_decomposition(&pauli_x).unwrap();

        // X = Rz(π) · Ry(π) · Rz(0) or equivalently U3(π, π, 0)
        assert!((angles.theta - PI).abs() < 1e-10, "θ should be π for X");
        assert!(verify_u3_decomposition(&pauli_x, &angles, 1e-10));
    }

    #[test]
    fn test_u3_hadamard() {
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        let mut h = Array2::zeros((2, 2));
        h[[0, 0]] = Complex64::new(inv_sqrt2, 0.0);
        h[[0, 1]] = Complex64::new(inv_sqrt2, 0.0);
        h[[1, 0]] = Complex64::new(inv_sqrt2, 0.0);
        h[[1, 1]] = Complex64::new(-inv_sqrt2, 0.0);

        let angles = u3_decomposition(&h).unwrap();
        // H = U3(π/2, 0, π)
        assert!(
            (angles.theta - PI / 2.0).abs() < 1e-10,
            "θ should be π/2 for H"
        );
        assert!(verify_u3_decomposition(&h, &angles, 1e-10));
    }

    #[test]
    fn test_u3_vs_zyz_consistency() {
        // Compare U3 and ZYZ: for a matrix with det=1 (no global phase),
        // both should produce the same gate sequence (up to angle ordering).
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        let mut h = Array2::zeros((2, 2));
        h[[0, 0]] = Complex64::new(inv_sqrt2, 0.0);
        h[[0, 1]] = Complex64::new(inv_sqrt2, 0.0);
        h[[1, 0]] = Complex64::new(inv_sqrt2, 0.0);
        h[[1, 1]] = Complex64::new(-inv_sqrt2, 0.0);

        let u3_angles = u3_decomposition(&h).unwrap();
        let zyz_angles = zyz_decomposition(&h).unwrap();

        // U3 has no global_phase by design. Verify H reconstruction.
        assert!(verify_u3_decomposition(&h, &u3_angles, 1e-10));

        // ZYZ may have a global_phase. The ZYZ reconstruction should also match H.
        assert!(verify_zyz_decomposition(&h, &zyz_angles, 1e-10));
    }

    #[test]
    fn test_u3_roundtrip() {
        // Test that U3 → matrix → U3 gives same angles (up to degeneracies)
        let angles_in = U3Angles::new(PI / 3.0, PI / 4.0, PI / 6.0);
        let matrix = u3_to_matrix(&angles_in);
        let angles_out = u3_decomposition(&matrix).unwrap();

        assert!(
            verify_u3_decomposition(&matrix, &angles_out, 1e-8),
            "U3 roundtrip should reproduce the matrix"
        );
        // θ should be preserved exactly
        assert!(
            (angles_out.theta - angles_in.theta).abs() < 1e-10,
            "θ should be preserved in roundtrip"
        );
    }
}
