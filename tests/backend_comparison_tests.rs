//! Backend comparison tests: MyMatBackend vs NdArrayBackend numerical consistency
//!
//! These tests verify that switching to MyMatBackend produces
//! numerically identical results to NdArrayBackend for all
//! key quantum computing operations.

#[cfg(test)]
mod backend_comparison_tests {
    use myquat::gates::{GateOperation, StandardGate};
    use myquat::linalg::{LinalgBackend, MyMatBackend, NdArrayBackend};
    use num_complex::Complex64;
    use std::collections::HashMap;

    fn c(re: f64, im: f64) -> Complex64 {
        Complex64::new(re, im)
    }

    fn compare_matrices<ND, MM>(
        nd: &ND,
        mm: &MM,
        nd_mat: &ND::Matrix,
        mm_mat: &MM::Matrix,
        rows: usize,
        cols: usize,
        label: &str,
        tol: f64,
    ) -> bool
    where
        ND: LinalgBackend<Scalar = Complex64>,
        MM: LinalgBackend<Scalar = Complex64>,
    {
        for i in 0..rows {
            for j in 0..cols {
                let nd_v = nd.get_matrix(nd_mat, i, j).unwrap();
                let mm_v = mm.get_matrix(mm_mat, i, j).unwrap();
                if (nd_v - mm_v).norm() > tol {
                    eprintln!(
                        "{} MISMATCH at ({},{}): nd={}, mm={}, diff={}",
                        label,
                        i,
                        j,
                        nd_v,
                        mm_v,
                        (nd_v - mm_v).norm()
                    );
                    return false;
                }
            }
        }
        true
    }

    #[test]
    fn test_identity_matrix_identical() {
        let nd = NdArrayBackend::new();
        let mm = MyMatBackend::new();
        let nd_id = nd.complex_identity(4).unwrap();
        let mm_id = mm.complex_identity(4).unwrap();
        assert!(compare_matrices(
            &nd, &mm, &nd_id, &mm_id, 4, 4, "Identity", 1e-12
        ));
    }

    #[test]
    fn test_kronecker_identical() {
        let nd = NdArrayBackend::new();
        let mm = MyMatBackend::new();
        let nd_i2 = nd.complex_identity(2).unwrap();
        let mm_i2 = mm.complex_identity(2).unwrap();
        let nd_kron = nd.kronecker(&nd_i2, &nd_i2).unwrap();
        let mm_kron = mm.kronecker(&mm_i2, &mm_i2).unwrap();
        assert!(compare_matrices(
            &nd,
            &mm,
            &nd_kron,
            &mm_kron,
            4,
            4,
            "Kronecker",
            1e-12
        ));
    }

    #[test]
    fn test_matrix_multiply_identical() {
        let nd = NdArrayBackend::new();
        let mm = MyMatBackend::new();
        let data = vec![c(1.0, 0.0), c(2.0, 1.0), c(3.0, -1.0), c(4.0, 0.0)];
        let nd_a = nd.from_shape_vec(2, 2, data.clone()).unwrap();
        let mm_a = mm.from_shape_vec(2, 2, data).unwrap();
        let nd_i2 = nd.complex_identity(2).unwrap();
        let mm_i2 = mm.complex_identity(2).unwrap();
        let nd_r = nd.dot(&nd_a, &nd_i2).unwrap();
        let mm_r = mm.dot(&mm_a, &mm_i2).unwrap();
        assert!(compare_matrices(
            &nd, &mm, &nd_r, &mm_r, 2, 2, "Multiply", 1e-12
        ));
    }

    #[test]
    fn test_conjugate_transpose_identical() {
        let nd = NdArrayBackend::new();
        let mm = MyMatBackend::new();
        let data = vec![c(1.0, 0.0), c(0.0, 2.0), c(0.0, -3.0), c(4.0, 0.0)];
        let nd_b = nd.from_shape_vec(2, 2, data.clone()).unwrap();
        let mm_b = mm.from_shape_vec(2, 2, data).unwrap();
        let nd_ct = nd.conjugate_transpose(&nd_b).unwrap();
        let mm_ct = mm.conjugate_transpose(&mm_b).unwrap();
        assert!(compare_matrices(
            &nd,
            &mm,
            &nd_ct,
            &mm_ct,
            2,
            2,
            "ConjTrans",
            1e-12
        ));
    }

    #[test]
    fn test_eigh_pauli_z_identical() {
        let nd = NdArrayBackend::new();
        let mm = MyMatBackend::new();
        let z_data = vec![c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(-1.0, 0.0)];
        let nd_z = nd.from_shape_vec(2, 2, z_data.clone()).unwrap();
        let mm_z = mm.from_shape_vec(2, 2, z_data).unwrap();
        let (nd_evals, _) = nd.eigh(&nd_z).unwrap();
        let (mm_evals, _) = mm.eigh(&mm_z).unwrap();

        let mut nd_sorted = nd_evals.clone();
        nd_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut mm_sorted = mm_evals.clone();
        mm_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        assert_eq!(nd_sorted.len(), mm_sorted.len());
        for (a, b) in nd_sorted.iter().zip(mm_sorted.iter()) {
            assert!(
                (a - b).abs() < 1e-10,
                "Eigenvalue mismatch: nd={}, mm={}",
                a,
                b
            );
        }
    }

    #[test]
    fn test_eigh_identity_identical() {
        let nd = NdArrayBackend::new();
        let mm = MyMatBackend::new();
        let nd_id = nd.complex_identity(3).unwrap();
        let mm_id = mm.complex_identity(3).unwrap();
        let (nd_evals, _) = nd.eigh(&nd_id).unwrap();
        let (mm_evals, _) = mm.eigh(&mm_id).unwrap();
        for v in &nd_evals {
            assert!((v - 1.0).abs() < 1e-10);
        }
        for v in &mm_evals {
            assert!((v - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_all_standard_gates_identical() {
        let nd = NdArrayBackend::new();
        let mm = MyMatBackend::new();
        let gates = vec![
            StandardGate::H,
            StandardGate::X,
            StandardGate::Y,
            StandardGate::Z,
            StandardGate::S,
            StandardGate::T,
            StandardGate::Rx,
            StandardGate::Ry,
            StandardGate::Rz,
        ];
        use myquat::parameter::Parameter;
        let params: [Parameter; 1] = [Parameter::Float(std::f64::consts::PI / 3.0)];
        let symbols = HashMap::new();

        for gate in &gates {
            let g_nd = gate.matrix_with_backend(&params, &symbols, &nd).unwrap();
            let g_mm = gate.matrix_with_backend(&params, &symbols, &mm).unwrap();
            let (r, c) = nd.dim(&g_nd);
            assert!(
                compare_matrices(&nd, &mm, &g_nd, &g_mm, r, c, &format!("{:?}", gate), 1e-10),
                "Gate {:?} differs between backends",
                gate
            );
        }
    }

    #[test]
    fn test_scalar_mul_and_transpose_identical() {
        let nd = NdArrayBackend::new();
        let mm = MyMatBackend::new();
        let data = vec![c(1.0, 0.5), c(2.0, -0.5), c(3.0, 0.0), c(4.0, -1.0)];
        let nd_a = nd.from_shape_vec(2, 2, data.clone()).unwrap();
        let mm_a = mm.from_shape_vec(2, 2, data).unwrap();

        // scalar_mul
        let nd_s = nd.scalar_mul(&nd_a, c(2.5, -1.0)).unwrap();
        let mm_s = mm.scalar_mul(&mm_a, c(2.5, -1.0)).unwrap();
        assert!(compare_matrices(
            &nd,
            &mm,
            &nd_s,
            &mm_s,
            2,
            2,
            "ScalarMul",
            1e-12
        ));

        // transpose
        let nd_t = nd.transpose(&nd_a).unwrap();
        let mm_t = mm.transpose(&mm_a).unwrap();
        assert!(compare_matrices(
            &nd,
            &mm,
            &nd_t,
            &mm_t,
            2,
            2,
            "Transpose",
            1e-12
        ));
    }

    #[test]
    fn test_mapv_identical() {
        let nd = NdArrayBackend::new();
        let mm = MyMatBackend::new();
        let data = vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)];
        let nd_a = nd.from_shape_vec(2, 2, data.clone()).unwrap();
        let mm_a = mm.from_shape_vec(2, 2, data).unwrap();

        let nd_r = nd.mapv(&nd_a, &|x| x * c(2.0, 0.0) + c(1.0, 0.0)).unwrap();
        let mm_r = mm.mapv(&mm_a, &|x| x * c(2.0, 0.0) + c(1.0, 0.0)).unwrap();
        assert!(compare_matrices(
            &nd, &mm, &nd_r, &mm_r, 2, 2, "Mapv", 1e-12
        ));
    }

    #[test]
    fn test_unitary_check_identical() {
        let nd = NdArrayBackend::new();
        let mm = MyMatBackend::new();
        // Use a known unitary: H ⊗ I
        let h_gate = StandardGate::H
            .matrix_with_backend(&[], &HashMap::new(), &nd)
            .unwrap();
        let i_gate = nd.complex_identity(2).unwrap();
        let h_tensor_i = nd.kronecker(&h_gate, &i_gate).unwrap();

        let nd_unitary =
            myquat::utils::MatrixUtils::is_unitary_with_backend(&h_tensor_i, 1e-10, &nd).unwrap();
        assert!(nd_unitary, "NdArray: H⊗I should be unitary");

        // Reconstruct with MyMat
        let mm_h = StandardGate::H
            .matrix_with_backend(&[], &HashMap::new(), &mm)
            .unwrap();
        let mm_i = mm.complex_identity(2).unwrap();
        let mm_h_tensor_i = mm.kronecker(&mm_h, &mm_i).unwrap();
        let mm_unitary =
            myquat::utils::MatrixUtils::is_unitary_with_backend(&mm_h_tensor_i, 1e-10, &mm)
                .unwrap();
        assert!(mm_unitary, "MyMat: H⊗I should be unitary");
    }

    // ── Regression tests (code review fixes) ────────────────────────────

    #[test]
    fn test_eigh_ordering_matches_without_presort() {
        // Regression: mymat returns eigenvalues descending, ndarray ascending.
        // After normalization both must agree element-wise WITHOUT pre-sorting.
        let nd = NdArrayBackend::new();
        let mm = MyMatBackend::new();
        // diag(3, 1, 2) — distinct eigenvalues so ordering is observable.
        let data = vec![
            c(3.0, 0.0),
            c(0.0, 0.0),
            c(0.0, 0.0),
            c(0.0, 0.0),
            c(1.0, 0.0),
            c(0.0, 0.0),
            c(0.0, 0.0),
            c(0.0, 0.0),
            c(2.0, 0.0),
        ];
        let nd_m = nd.from_shape_vec(3, 3, data.clone()).unwrap();
        let mm_m = mm.from_shape_vec(3, 3, data).unwrap();
        let (nd_e, _) = nd.eigh(&nd_m).unwrap();
        let (mm_e, _) = mm.eigh(&mm_m).unwrap();
        // Both must be ascending [1, 2, 3] — compared directly, no sort.
        assert_eq!(nd_e.len(), mm_e.len());
        for (a, b) in nd_e.iter().zip(mm_e.iter()) {
            assert!(
                (a - b).abs() < 1e-10,
                "eigh order mismatch: nd={:?} mm={:?}",
                nd_e,
                mm_e
            );
        }
        // Confirm it is actually ascending.
        assert!(
            mm_e[0] < mm_e[1] && mm_e[1] < mm_e[2],
            "MyMat eigh not ascending: {:?}",
            mm_e
        );
    }

    #[test]
    fn test_nontrivial_matmul_identical() {
        // Regression: A·I can't detect a row/col-major transpose bug.
        // Multiply two distinct non-symmetric operands.
        let nd = NdArrayBackend::new();
        let mm = MyMatBackend::new();
        let a = vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)];
        let b = vec![c(5.0, 0.0), c(6.0, 0.0), c(7.0, 0.0), c(8.0, 0.0)];
        let nd_r = nd
            .dot(
                &nd.from_shape_vec(2, 2, a.clone()).unwrap(),
                &nd.from_shape_vec(2, 2, b.clone()).unwrap(),
            )
            .unwrap();
        let mm_r = mm
            .dot(
                &mm.from_shape_vec(2, 2, a).unwrap(),
                &mm.from_shape_vec(2, 2, b).unwrap(),
            )
            .unwrap();
        assert!(compare_matrices(
            &nd,
            &mm,
            &nd_r,
            &mm_r,
            2,
            2,
            "NontrivialMatmul",
            1e-12
        ));
        // Sanity: [[1,2],[3,4]]·[[5,6],[7,8]] = [[19,22],[43,50]]
        assert_eq!(nd.get_matrix(&nd_r, 0, 0).unwrap(), c(19.0, 0.0));
        assert_eq!(nd.get_matrix(&nd_r, 1, 1).unwrap(), c(50.0, 0.0));
    }
}
