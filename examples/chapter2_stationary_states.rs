//! Chapter 2: Time-Independent Schrödinger Equation — Demo
//! Author: gA4ss
//!
//! Demonstrates stationary states using analytical solutions and numerical grids.
//! Uses `ndarray` for grid-based computation and `myquat::qm_solver` for types.

use ndarray::Array1;
use std::f64::consts::PI;

/// Simple 1D grid for numerical demonstrations.
struct Grid1D {
    x_min: f64,
    x_max: f64,
    n: usize,
    dx: f64,
}

impl Grid1D {
    fn new(x_min: f64, x_max: f64, n: usize) -> Self {
        let dx = (x_max - x_min) / (n as f64 - 1.0);
        Self {
            x_min,
            x_max,
            n,
            dx,
        }
    }

    fn x(&self, i: usize) -> f64 {
        self.x_min + (i as f64) * self.dx
    }
}

/// Analytical eigenfunctions for infinite square well.
fn well_eigenfunction(n: usize, a: f64, grid: &Grid1D) -> Array1<f64> {
    let norm = (2.0 / a).sqrt();
    Array1::from(
        (0..grid.n)
            .map(|i| {
                let x = grid.x(i);
                if x >= 0.0 && x <= a {
                    norm * ((n as f64 * PI * x) / a).sin()
                } else {
                    0.0
                }
            })
            .collect::<Vec<_>>(),
    )
}

fn main() {
    println!("Chapter 2: Time-Independent Schrodinger Equation");
    println!("=================================================\n");

    // 2.1 Infinite Square Well
    println!("--- 2.1 Infinite Square Well ---");
    let a = 1.0; // well width
    let grid = Grid1D::new(-0.2, a + 0.2, 256);
    println!(
        "Grid: {} points from {:.1} to {:.1}, dx={:.4}",
        grid.n, grid.x_min, grid.x_max, grid.dx
    );

    // Analytical energies: E_n = n^2 pi^2 hbar^2 / (2 m a^2) = n^2 pi^2 / 2 (units)
    println!("\nEigenvalues and normalization checks:");
    for n in 1..=4 {
        let energy = (n as f64).powi(2) * PI.powi(2) / 2.0;
        let psi = well_eigenfunction(n, a, &grid);
        // Trapezoidal integration: sum |psi|^2 * dx
        let prob_mass: f64 = psi.iter().map(|v| v.powi(2)).sum::<f64>();
        let prob_mass_trap = prob_mass * grid.dx;
        let prob_mid = 0.0; // edge points vanish for sin
        let norm_check = prob_mass_trap - prob_mid * grid.dx / 2.0;
        println!(
            "  n={}: E={:.6}, normalization = {:.6}",
            n, energy, norm_check
        );

        // Probability at center for even/odd parity
        let center_idx = grid.n / 2;
        let psi_center = psi[center_idx];
        println!(
            "       psi(a/2={:.1}) = {:.6}",
            grid.x(center_idx),
            psi_center
        );
    }

    // 2.3 Harmonic Oscillator
    println!("\n--- 2.3 Harmonic Oscillator ---");
    let omega = 1.0; // frequency
    let ho_grid = Grid1D::new(-5.0, 5.0, 512);

    // Ground state: psi_0(x) = pi^{-1/4} * exp(-x^2/2)
    let ho_ground: Vec<f64> = (0..ho_grid.n)
        .map(|i| {
            let x = ho_grid.x(i);
            (1.0_f64 / PI).powf(0.25) * (-0.5_f64 * x.powi(2)).exp()
        })
        .collect();
    let norm: f64 = ho_ground.iter().map(|v| v.powi(2)).sum::<f64>() * ho_grid.dx;
    println!("Ground state: omega={}, E0=0.5", omega);
    println!("  norm check: {:.6}", norm);

    // First excited: psi_1(x) = sqrt(2) * x * psi_0(x)
    let ho_first: Vec<f64> = ho_ground
        .iter()
        .enumerate()
        .map(|(i, psi0)| 2.0_f64.sqrt() * ho_grid.x(i) * psi0)
        .collect();
    let norm1: f64 = ho_first.iter().map(|v| v.powi(2)).sum::<f64>() * ho_grid.dx;
    println!("First excited: E1=1.5, norm check: {:.6}", norm1);

    // Orthogonality check
    let overlap: f64 = ho_ground
        .iter()
        .zip(ho_first.iter())
        .map(|(g, f)| g * f)
        .sum::<f64>()
        * ho_grid.dx;
    println!("  <psi_0|psi_1> = {:.2e} (should be ~0)", overlap);

    // 2.4 Finite Square Well (qualitative)
    println!("\n--- 2.4 Finite Square Well ---");
    let v0 = 10.0; // well depth
    let w = 1.0; // well width
    let f_grid = Grid1D::new(-3.0, 3.0, 512);
    println!(
        "Finite well: V0={}, width={}, {} grid points",
        v0, w, f_grid.n
    );
    // Counting bound states approximation
    let k: f64 = (2.0_f64 * v0).sqrt(); // sqrt(2mV0/hbar^2) with m=hbar=1
    let approx_bound = (k * w / PI).ceil() as usize;
    println!("  Approximate bound states: {}", approx_bound);

    println!("\nChapter 2 demo completed.");
}
