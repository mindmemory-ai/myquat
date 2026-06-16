//! Diagnostic: experimentally determine magic basis properties for KAK fix.
//! Build known unitaries, compute their magic-basis transform, extract eigenvalues,
//! and determine the correct mapping from eigenvalues to KAK coefficients.
use ndarray::Array2;
use num_complex::Complex64;
use std::f64::consts::PI;

type C = Complex64;

fn main() {
    let sqrt2_inv = 1.0 / 2.0_f64.sqrt();
    let i = Complex64::new(0.0, 1.0);

    // Magic basis matrix M (from to_magic_basis)
    let mut magic = Array2::zeros((4, 4));
    magic[[0, 0]] = C::new(sqrt2_inv, 0.0);
    magic[[0, 3]] = i * sqrt2_inv;
    magic[[1, 1]] = i * sqrt2_inv;
    magic[[1, 2]] = C::new(sqrt2_inv, 0.0);
    magic[[2, 1]] = i * sqrt2_inv;
    magic[[2, 2]] = C::new(-sqrt2_inv, 0.0);
    magic[[3, 0]] = C::new(sqrt2_inv, 0.0);
    magic[[3, 3]] = -i * sqrt2_inv;
    let magic_dag = magic.mapv(|z| z.conj()).t().to_owned();

    // Helper: transform to magic basis
    let to_magic = |u: &Array2<C>| -> Array2<C> { magic_dag.dot(u).dot(&magic) };

    // Helper: build XX, YY, ZZ interaction in computational basis
    let build_xx = || -> Array2<C> {
        // X⊗X
        let mut m = Array2::zeros((4, 4));
        m[[0, 3]] = C::new(1.0, 0.0);
        m[[1, 2]] = C::new(1.0, 0.0);
        m[[2, 1]] = C::new(1.0, 0.0);
        m[[3, 0]] = C::new(1.0, 0.0);
        m
    };
    let build_yy = || -> Array2<C> {
        // Y⊗Y = [[0,0,0,-1],[0,0,1,0],[0,1,0,0],[-1,0,0,0]]
        let mut m = Array2::zeros((4, 4));
        m[[0, 3]] = C::new(-1.0, 0.0);
        m[[1, 2]] = C::new(1.0, 0.0);
        m[[2, 1]] = C::new(1.0, 0.0);
        m[[3, 0]] = C::new(-1.0, 0.0);
        m
    };
    let build_zz = || -> Array2<C> {
        // Z⊗Z = diag(1, -1, -1, 1)
        let mut m = Array2::zeros((4, 4));
        m[[0, 0]] = C::new(1.0, 0.0);
        m[[1, 1]] = C::new(-1.0, 0.0);
        m[[2, 2]] = C::new(-1.0, 0.0);
        m[[3, 3]] = C::new(1.0, 0.0);
        m
    };

    println!("=== Part 1: Verify XX, YY, ZZ are diagonal in magic basis ===\n");

    let ops: [(&str, fn() -> Array2<C>); 3] =
        [("XX", build_xx), ("YY", build_yy), ("ZZ", build_zz)];
    for (name, build_fn) in &ops {
        let op_comp = build_fn();
        let op_magic = to_magic(&op_comp);
        println!("{} in magic basis:", name);
        for i_row in 0..4 {
            print!("  [");
            for j in 0..4 {
                let v = op_magic[[i_row, j]];
                if v.norm() < 1e-10 {
                    print!("{:>10}", "0");
                } else {
                    print!("{:>8.2}{:+.2}i", v.re, v.im);
                }
            }
            println!(" ]");
        }
        // Extract diagonal as eigenvalues
        let diag: Vec<C> = (0..4).map(|k| op_magic[[k, k]]).collect();
        println!(
            "  diag: {:?}",
            diag.iter().map(|c| format!("{:.1}", c)).collect::<Vec<_>>()
        );
        println!();
    }

    println!("=== Part 2: Verify interaction matrix K(a,b,c) in magic basis ===\n");

    // For K = exp(i(a·XX + b·YY + c·ZZ)), in magic basis this should be diagonal.
    // The diagonal entries depend on the magic basis convention.
    // Let's compute for specific (a,b,c) values.

    // Use nalgebra to compute matrix exponential
    let exp_mat = |a: f64, b: f64, c: f64| -> Array2<C> {
        let xx = build_xx();
        let yy = build_yy();
        let zz = build_zz();
        // H = a*XX + b*YY + c*ZZ
        let mut h = Array2::zeros((4, 4));
        for i_row in 0..4 {
            for j in 0..4 {
                h[[i_row, j]] = C::new(0.0, 1.0)
                    * (C::new(a, 0.0) * xx[[i_row, j]]
                        + C::new(b, 0.0) * yy[[i_row, j]]
                        + C::new(c, 0.0) * zz[[i_row, j]]);
            }
        }
        // Matrix exponential via eigendecomposition of Pauli products
        // Since XX, YY, ZZ commute pairwise, we can compute eigenvalues of H directly
        // H = a·XX + b·YY + c·ZZ
        // The eigenvalues of H in computational basis are ±a ± b ± c
        // In magic basis, H should be diagonal

        // Actually, let's build exp(i(a·XX + b·YY + c·ZZ)) directly
        // Since XX, YY, ZZ all commute, the exponential factorizes:
        // exp(i(a·XX + b·YY + c·ZZ)) = exp(ia·XX) · exp(ib·YY) · exp(ic·ZZ)
        // Each exp(it·PP) = cos(t)·I + i·sin(t)·PP for Pauli products

        let exp_xx = {
            let mut m = Array2::<C>::eye(4);
            let ct = C::new(a.cos(), 0.0);
            let st = C::new(a.sin(), 0.0);
            for i_row in 0..4 {
                for j in 0..4 {
                    m[[i_row, j]] = ct * m[[i_row, j]] + C::new(0.0, 1.0) * st * xx[[i_row, j]];
                }
            }
            m
        };
        let exp_yy = {
            let mut m = Array2::<C>::eye(4);
            let ct = C::new(b.cos(), 0.0);
            let st = C::new(b.sin(), 0.0);
            for i_row in 0..4 {
                for j in 0..4 {
                    m[[i_row, j]] = ct * m[[i_row, j]] + C::new(0.0, 1.0) * st * yy[[i_row, j]];
                }
            }
            m
        };
        let exp_zz = {
            let mut m = Array2::<C>::eye(4);
            let ct = C::new(c.cos(), 0.0);
            let st = C::new(c.sin(), 0.0);
            for i_row in 0..4 {
                for j in 0..4 {
                    m[[i_row, j]] = ct * m[[i_row, j]] + C::new(0.0, 1.0) * st * zz[[i_row, j]];
                }
            }
            m
        };
        exp_xx.dot(&exp_yy).dot(&exp_zz)
    };

    // Test cases for (a,b,c)
    let test_coeffs = [
        (0.0, 0.0, 0.25),     // pure ZZ interaction (Rzz(0.5))
        (0.25, 0.0, 0.0),     // pure XX interaction
        (0.0, 0.25, 0.0),     // pure YY interaction
        (PI / 4.0, 0.0, 0.0), // CX-equivalent
        (0.1, 0.2, 0.3),      // generic
    ];

    for &(a, b, c) in &test_coeffs {
        let k_comp = exp_mat(a, b, c);
        let k_magic = to_magic(&k_comp);
        println!("K({:.3}, {:.3}, {:.3}) in magic basis:", a, b, c);

        // Check if diagonal
        let mut max_offdiag = 0.0_f64;
        for i_row in 0..4 {
            for j in 0..4 {
                if i_row != j {
                    max_offdiag = max_offdiag.max(k_magic[[i_row, j]].norm());
                }
            }
        }
        println!("  max off-diagonal: {:.2e}", max_offdiag);

        // Print diagonal
        print!("  diag: [");
        for k in 0..4 {
            let v = k_magic[[k, k]];
            let phase = v.arg();
            print!("{:.4}∠{:.4}°, ", v.norm(), phase * 180.0 / PI);
        }
        println!("]");

        // Extract phases
        let phases: Vec<f64> = (0..4).map(|k| k_magic[[k, k]].arg()).collect();
        println!(
            "  phases: {:?}",
            phases
                .iter()
                .map(|p| format!("{:.4}", p))
                .collect::<Vec<_>>()
        );

        // Compute KAK coefficients from phases using the formula from current code:
        // x = (p3 - p2 - p1 + p0)/4, y = (p3 - p2 + p1 - p0)/4, z = (p3 + p2 - p1 - p0)/4
        // BUT we need the correct ORDERING of phases
        let ps = phases;
        // Try all permutations to see which gives correct (a,b,c)
        println!(
            "  Trying to recover ({:.3},{:.3},{:.3}) from phases:",
            a, b, c
        );

        // The KAK interaction matrix in magic basis should have phases:
        // For the magic basis in this code (rows are Bell states):
        // |Φ⁺⟩ = row 0, |Ψ⁺⟩ = row 1, |Ψ⁻⟩ = row 2, |Φ⁻⟩ = row 3
        //
        // XX eigenvalues on Bell: +|Φ⁺⟩, +|Ψ⁺⟩, -|Ψ⁻⟩, -|Φ⁻⟩
        // YY eigenvalues on Bell: -|Φ⁺⟩, +|Ψ⁺⟩, -|Ψ⁻⟩, +|Φ⁻⟩
        // ZZ eigenvalues on Bell: +|Φ⁺⟩, -|Ψ⁺⟩, -|Ψ⁻⟩, +|Φ⁻⟩
        //
        // So K(a,b,c) = exp(i(a·XX+b·YY+c·ZZ)) in magic basis:
        // |Φ⁺⟩: exp(i(+a -b +c))
        // |Ψ⁺⟩: exp(i(+a +b -c))
        // |Ψ⁻⟩: exp(i(-a -b -c))
        // |Φ⁻⟩: exp(i(-a +b +c))

        let expected = [a - b + c, a + b - c, -a - b - c, -a + b + c];
        println!(
            "  Expected phases from derivation: {:?}",
            expected
                .iter()
                .map(|p| format!("{:.4}", p))
                .collect::<Vec<_>>()
        );
        println!(
            "  Actual phases from matrix exp:   {:?}",
            ps.iter().map(|p| format!("{:.4}", p)).collect::<Vec<_>>()
        );

        // Check if any ordering matches
        for perm_idx in 0..24 {
            let perm = permutation(perm_idx);
            let mut matched = true;
            for k in 0..4 {
                let diff = (ps[perm[k]] - expected[k]).abs();
                if diff > 1e-2 {
                    matched = false;
                    break;
                }
            }
            if matched {
                println!("  MATCH with permutation: {:?}", perm);
            }
        }
        println!();
    }

    println!("=== Part 3: Test U_B^T · U_B approach ===\n");

    // For K(a,b,c) in magic basis, K_B is diagonal with entries exp(i·d_k)
    // K_B^T · K_B = diag(exp(2i·d_k)) since K_B is diagonal
    // The eigenvalues are exp(2i·d_k)

    for &(a, b, c) in &test_coeffs {
        let k_comp = exp_mat(a, b, c);
        let k_magic = to_magic(&k_comp);

        // Verify K_magic is diagonal
        let diag_phases: Vec<f64> = (0..4).map(|k| k_magic[[k, k]].arg()).collect();

        // Compute U_B^T · U_B (transpose!)
        let k_magic_t = k_magic.t().to_owned();
        let v = k_magic_t.dot(&k_magic);

        // V should also be diagonal = diag(exp(2i·d_k))
        // Extract eigenvalues
        let v_diag: Vec<C> = (0..4).map(|k| v[[k, k]]).collect();
        let v_phases: Vec<f64> = v_diag.iter().map(|c| c.arg()).collect();

        println!("K({:.3},{:.3},{:.3}):", a, b, c);
        println!(
            "  K_B diag phases:      {:?}",
            diag_phases
                .iter()
                .map(|p| format!("{:.4}", p))
                .collect::<Vec<_>>()
        );
        println!(
            "  K_B^T·K_B diag phases: {:?}",
            v_phases
                .iter()
                .map(|p| format!("{:.4}", p))
                .collect::<Vec<_>>()
        );

        // Check: v_phases should be 2*diag_phases (mod 2π)
        for k in 0..4 {
            let diff = (v_phases[k] - 2.0 * diag_phases[k]).abs();
            println!(
                "    k={}: 2*diag={:.4}, v_phase={:.4}, diff={:.4}",
                k,
                2.0 * diag_phases[k],
                v_phases[k],
                diff
            );
        }
        println!();
    }

    println!("=== Part 4: Test with a full unitary (CX gate) ===\n");

    // CX gate in computational basis
    let mut cx = Array2::<C>::zeros((4, 4));
    cx[[0, 0]] = C::new(1.0, 0.0);
    cx[[1, 1]] = C::new(1.0, 0.0);
    cx[[2, 3]] = C::new(1.0, 0.0);
    cx[[3, 2]] = C::new(1.0, 0.0);

    let cx_magic = to_magic(&cx);
    println!("CX in magic basis:");
    for i_row in 0..4 {
        print!("  [");
        for j in 0..4 {
            let v = cx_magic[[i_row, j]];
            if v.norm() < 1e-10 {
                print!("{:>10}", "0");
            } else {
                print!("{:>8.2}{:+.2}i", v.re, v.im);
            }
        }
        println!(" ]");
    }

    // Compute U_B^T · U_B for CX
    let cx_magic_t = cx_magic.t().to_owned();
    let v_cx = cx_magic_t.dot(&cx_magic);
    println!("\nCX: U_B^T·U_B:");
    for i_row in 0..4 {
        print!("  [");
        for j in 0..4 {
            let v = v_cx[[i_row, j]];
            if v.norm() < 1e-10 {
                print!("{:>10}", "0");
            } else {
                print!("{:>8.2}{:+.2}i", v.re, v.im);
            }
        }
        println!(" ]");
    }

    // Extract eigenvalues from V
    let mut nalg_v = nalgebra::Matrix4::<C>::zeros();
    for i_row in 0..4 {
        for j in 0..4 {
            nalg_v[(i_row, j)] = v_cx[[i_row, j]];
        }
    }
    if let Some(schur) = nalgebra::Schur::try_new(nalg_v, 1e-10, 100) {
        let (_q, t) = schur.unpack();
        let eig: Vec<C> = (0..4).map(|k| t[(k, k)]).collect();
        let phases: Vec<f64> = eig.iter().map(|c| c.arg()).collect();
        println!(
            "  eigenvalues phases: {:?}",
            phases
                .iter()
                .map(|p| format!("{:.4}", p))
                .collect::<Vec<_>>()
        );
        println!(
            "  phases/4: {:?}",
            phases
                .iter()
                .map(|p| format!("{:.4}", p / 4.0))
                .collect::<Vec<_>>()
        );
        // For CX, KAK coeffs should be [π/4, 0, 0]
    }
}

fn permutation(idx: usize) -> [usize; 4] {
    let perms = [
        [0, 1, 2, 3],
        [0, 1, 3, 2],
        [0, 2, 1, 3],
        [0, 2, 3, 1],
        [0, 3, 1, 2],
        [0, 3, 2, 1],
        [1, 0, 2, 3],
        [1, 0, 3, 2],
        [1, 2, 0, 3],
        [1, 2, 3, 0],
        [1, 3, 0, 2],
        [1, 3, 2, 0],
        [2, 0, 1, 3],
        [2, 0, 3, 1],
        [2, 1, 0, 3],
        [2, 1, 3, 0],
        [2, 3, 0, 1],
        [2, 3, 1, 0],
        [3, 0, 1, 2],
        [3, 0, 2, 1],
        [3, 1, 0, 2],
        [3, 1, 2, 0],
        [3, 2, 0, 1],
        [3, 2, 1, 0],
    ];
    perms[idx]
}
