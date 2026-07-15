//! Tests for the linalg backend module.
//!
//! Tests cover:
//! - NdArrayBackend construction, indexing, and arithmetic
//! - Kronecker product identity
//! - Transpose and conjugate transpose
//! - Hermitian eigenvalue decomposition
//! - MyMatBackend construction and operations
//! - Factory and config creation

#[cfg(test)]
mod linalg_tests {
    use crate::linalg::backend::LinalgBackend;
    use crate::linalg::backend::LinalgError;
    use crate::linalg::backend::LinalgScalar;
    use crate::linalg::config::{LinalgBackendType, LinalgConfig};
    use crate::linalg::factory::create_backend;
    use crate::linalg::factory::create_default_backend;
    use crate::linalg::factory::LinalgBackendImpl;
    use crate::linalg::mymat_adapter::MyMatBackend;
    use crate::linalg::ndarray_adapter::NdArrayBackend;
    use num_complex::Complex64;

    fn c(re: f64, im: f64) -> Complex64 {
        num_complex::Complex::new(re, im)
    }

    // ========================================================================
    // NdArrayBackend — Construction
    // ========================================================================

    #[test]
    fn test_ndarray_zeros_and_get_set() {
        let b = NdArrayBackend::new();
        let mut m = b.zeros_matrix(2, 3).unwrap();
        assert_eq!(b.dim(&m), (2, 3));
        assert_eq!(b.nrows(&m), 2);
        assert_eq!(b.ncols(&m), 3);

        b.set_matrix(&mut m, 0, 1, c(42.0, 0.0)).unwrap();
        let val = b.get_matrix(&m, 0, 1).unwrap();
        assert_eq!(val, c(42.0, 0.0));
    }

    #[test]
    fn test_ndarray_eye() {
        let b = NdArrayBackend::new();
        let i3 = b.eye(3).unwrap();
        assert_eq!(b.dim(&i3), (3, 3));
        for i in 0..3 {
            assert_eq!(b.get_matrix(&i3, i, i).unwrap(), c(1.0, 0.0));
        }
        // Off-diagonal should be zero
        assert_eq!(b.get_matrix(&i3, 0, 1).unwrap(), c(0.0, 0.0));
    }

    #[test]
    fn test_ndarray_complex_identity() {
        let b = NdArrayBackend::new();
        let i2 = b.complex_identity(2).unwrap();
        assert_eq!(b.dim(&i2), (2, 2));
        assert_eq!(b.get_matrix(&i2, 0, 0).unwrap(), c(1.0, 0.0));
        assert_eq!(b.get_matrix(&i2, 0, 1).unwrap(), c(0.0, 0.0));
    }

    #[test]
    fn test_ndarray_from_shape_vec() {
        let b = NdArrayBackend::new();
        let m = b
            .from_shape_vec(
                2,
                2,
                vec![c(1.0, 0.0), c(2.0, 3.0), c(4.0, 5.0), c(6.0, 0.0)],
            )
            .unwrap();
        assert_eq!(b.dim(&m), (2, 2));
        assert_eq!(b.get_matrix(&m, 0, 1).unwrap(), c(2.0, 3.0));
    }

    #[test]
    fn test_ndarray_vector_ops() {
        let b = NdArrayBackend::new();
        let v = b
            .from_vec(vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0)])
            .unwrap();
        assert_eq!(b.len_vector(&v), 3);
        assert_eq!(b.get_vector(&v, 1).unwrap(), c(2.0, 0.0));

        let mut v2 = b.zeros_vector(3).unwrap();
        b.set_vector(&mut v2, 0, c(10.0, 0.0)).unwrap();
        assert_eq!(b.get_vector(&v2, 0).unwrap(), c(10.0, 0.0));
    }

    // ========================================================================
    // NdArrayBackend — Linear Algebra
    // ========================================================================

    #[test]
    fn test_ndarray_dot_identity() {
        let b = NdArrayBackend::new();
        let i2 = b.complex_identity(2).unwrap();
        let x = b
            .from_shape_vec(
                2,
                2,
                vec![c(0.0, 0.0), c(1.0, 0.0), c(1.0, 0.0), c(0.0, 0.0)],
            )
            .unwrap();
        let result = b.dot(&i2, &x).unwrap();
        for i in 0..2 {
            for j in 0..2 {
                let expected = b.get_matrix(&x, i, j).unwrap();
                let actual = b.get_matrix(&result, i, j).unwrap();
                assert!(
                    (actual - expected).norm() < 1e-10,
                    "Mismatch at ({},{}): {:?} vs {:?}",
                    i,
                    j,
                    actual,
                    expected
                );
            }
        }
    }

    #[test]
    fn test_ndarray_dot_vec() {
        let b = NdArrayBackend::new();
        let m = b
            .from_shape_vec(
                2,
                2,
                vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)],
            )
            .unwrap();
        let v = b.from_vec(vec![c(1.0, 0.0), c(1.0, 0.0)]).unwrap();
        let result = b.dot_vec(&m, &v).unwrap();
        assert_eq!(b.len_vector(&result), 2);
        assert_eq!(b.get_vector(&result, 0).unwrap(), c(3.0, 0.0)); // 1*1 + 2*1
        assert_eq!(b.get_vector(&result, 1).unwrap(), c(7.0, 0.0)); // 3*1 + 4*1
    }

    #[test]
    fn test_ndarray_kronecker_identity() {
        let b = NdArrayBackend::new();
        let i2 = b.complex_identity(2).unwrap();
        let k = b.kronecker(&i2, &i2).unwrap();
        assert_eq!(b.dim(&k), (4, 4));
        let i4 = b.complex_identity(4).unwrap();
        for i in 0..4 {
            for j in 0..4 {
                let diff =
                    (b.get_matrix(&k, i, j).unwrap() - b.get_matrix(&i4, i, j).unwrap()).norm();
                assert!(diff < 1e-10, "Mismatch at ({},{}): diff={}", i, j, diff);
            }
        }
    }

    #[test]
    fn test_ndarray_kronecker_pauli_x() {
        // X ⊗ I should give the correct 4x4 matrix
        let b = NdArrayBackend::new();
        let x = b
            .from_shape_vec(
                2,
                2,
                vec![c(0.0, 0.0), c(1.0, 0.0), c(1.0, 0.0), c(0.0, 0.0)],
            )
            .unwrap();
        let i2 = b.complex_identity(2).unwrap();
        let k = b.kronecker(&x, &i2).unwrap();
        assert_eq!(b.dim(&k), (4, 4));
        // X⊗I swaps top 2 rows with bottom 2 rows
        assert_eq!(b.get_matrix(&k, 0, 0).unwrap(), c(0.0, 0.0));
        assert_eq!(b.get_matrix(&k, 2, 2).unwrap(), c(0.0, 0.0));
    }

    #[test]
    fn test_ndarray_transpose() {
        let b = NdArrayBackend::new();
        let m = b
            .from_shape_vec(
                2,
                3,
                vec![
                    c(1.0, 0.0),
                    c(2.0, 0.0),
                    c(3.0, 0.0),
                    c(4.0, 0.0),
                    c(5.0, 0.0),
                    c(6.0, 0.0),
                ],
            )
            .unwrap();
        let mt = b.transpose(&m).unwrap();
        assert_eq!(b.dim(&mt), (3, 2));
        assert_eq!(b.get_matrix(&mt, 0, 1).unwrap(), c(4.0, 0.0));
        assert_eq!(b.get_matrix(&mt, 2, 0).unwrap(), c(3.0, 0.0));
    }

    #[test]
    fn test_ndarray_conjugate_transpose() {
        let b = NdArrayBackend::new();
        let m = b
            .from_shape_vec(
                2,
                2,
                vec![c(1.0, 0.0), c(0.0, 1.0), c(0.0, -1.0), c(1.0, 0.0)],
            )
            .unwrap();
        let h = b.conjugate_transpose(&m).unwrap();
        // Hermitian of [[1, i], [-i, 1]] is itself (this is a Hermitian matrix)
        assert!((b.get_matrix(&h, 0, 0).unwrap() - c(1.0, 0.0)).norm() < 1e-10);
        assert!((b.get_matrix(&h, 1, 0).unwrap() - c(0.0, -1.0)).norm() < 1e-10);
    }

    // ========================================================================
    // NdArrayBackend — Element-wise Operations
    // ========================================================================

    #[test]
    fn test_ndarray_mapv() {
        let b = NdArrayBackend::new();
        let m = b
            .from_shape_vec(
                2,
                2,
                vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)],
            )
            .unwrap();
        let doubled = b.mapv(&m, &|x| x * c(2.0, 0.0)).unwrap();
        assert_eq!(b.get_matrix(&doubled, 0, 0).unwrap(), c(2.0, 0.0));
        assert_eq!(b.get_matrix(&doubled, 1, 1).unwrap(), c(8.0, 0.0));
    }

    #[test]
    fn test_ndarray_mapv_inplace() {
        let b = NdArrayBackend::new();
        let mut m = b.eye(2).unwrap();
        b.mapv_inplace(&mut m, &|x| x * c(3.0, 0.0)).unwrap();
        assert_eq!(b.get_matrix(&m, 0, 0).unwrap(), c(3.0, 0.0));
    }

    #[test]
    fn test_ndarray_fill_and_assign() {
        let b = NdArrayBackend::new();
        let mut m = b.zeros_matrix(2, 2).unwrap();
        b.fill(&mut m, c(5.0, 0.0)).unwrap();
        assert_eq!(b.get_matrix(&m, 0, 0).unwrap(), c(5.0, 0.0));

        let i2 = b.complex_identity(2).unwrap();
        b.assign(&mut m, &i2).unwrap();
        assert_eq!(b.get_matrix(&m, 0, 0).unwrap(), c(1.0, 0.0));
    }

    #[test]
    fn test_ndarray_scalar_mul() {
        let b = NdArrayBackend::new();
        let m = b
            .from_shape_vec(
                2,
                2,
                vec![c(1.0, 0.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, 0.0)],
            )
            .unwrap();
        let scaled = b.scalar_mul(&m, c(2.0, 0.0)).unwrap();
        assert_eq!(b.get_matrix(&scaled, 0, 0).unwrap(), c(2.0, 0.0));
        assert_eq!(b.get_matrix(&scaled, 1, 1).unwrap(), c(8.0, 0.0));
    }

    // ========================================================================
    // NdArrayBackend — Decompositions
    // ========================================================================

    #[test]
    fn test_ndarray_eigh_hermitian_pauli_z() {
        let b = NdArrayBackend::new();
        let z = b
            .from_shape_vec(
                2,
                2,
                vec![c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(-1.0, 0.0)],
            )
            .unwrap();
        let (evals, _evecs) = b.eigh(&z).unwrap();
        assert_eq!(evals.len(), 2);
        // Eigenvalues of Z are ±1
        assert!((evals.iter().sum::<f64>() - 0.0).abs() < 1e-10);
        assert!((evals[0].abs() - 1.0).abs() < 1e-10);
        assert!((evals[1].abs() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ndarray_eigh_identity() {
        let b = NdArrayBackend::new();
        let i3 = b.complex_identity(3).unwrap();
        let (evals, evecs) = b.eigh(&i3).unwrap();
        assert_eq!(evals.len(), 3);
        for e in &evals {
            assert!((e - 1.0).abs() < 1e-10);
        }
        // Eigenvectors should form identity
        assert_eq!(b.dim(&evecs), (3, 3));
    }

    #[test]
    fn test_ndarray_schur_identity() {
        let b = NdArrayBackend::new();
        let i4 = b.complex_identity(4).unwrap();
        let result = b.schur_decomposition(&i4, 1e-10, 100).unwrap();
        assert_eq!(b.dim(&result.q), (4, 4));
        assert_eq!(b.dim(&result.t), (4, 4));
    }

    // ========================================================================
    // NdArrayBackend — Views
    // ========================================================================

    #[test]
    fn test_ndarray_view_roundtrip() {
        let b = NdArrayBackend::new();
        let m = b.complex_identity(2).unwrap();
        let view = LinalgBackend::view(&b, &m);
        let owned = LinalgBackend::to_owned(&b, &view).unwrap();
        assert_eq!(b.dim(&owned), (2, 2));
        assert_eq!(b.get_matrix(&owned, 0, 0).unwrap(), c(1.0, 0.0));
    }

    #[test]
    fn test_ndarray_vector_view() {
        let b = NdArrayBackend::new();
        let mut v = b.from_vec(vec![c(1.0, 0.0), c(2.0, 0.0)]).unwrap();
        let _view = b.view_vector(&v);
        let _mut_view = b.view_mut_vector(&mut v);
    }

    // ========================================================================
    // Factory & Config
    // ========================================================================

    #[test]
    fn test_factory_default_is_mymat() {
        let config = LinalgConfig::default();
        assert_eq!(config.backend, LinalgBackendType::MyMat);
    }

    #[test]
    fn test_factory_mymat_config() {
        let config = LinalgConfig::mymat();
        assert_eq!(config.backend, LinalgBackendType::MyMat);
    }

    #[test]
    fn test_create_default_backend_is_mymat() {
        let backend_enum = create_default_backend();
        match backend_enum {
            LinalgBackendImpl::MyMat(b) => {
                let eye = b.eye(3).unwrap();
                assert_eq!(b.nrows(&eye), 3);
            }
            _ => panic!("Expected MyMat backend as default"),
        }
    }

    #[test]
    fn test_create_backend_ndarray_and_use() {
        let config = LinalgConfig::ndarray();
        let backend_enum = create_backend(&config);
        match backend_enum {
            LinalgBackendImpl::NdArray(b) => {
                let eye = b.eye(3).unwrap();
                assert_eq!(b.nrows(&eye), 3);
            }
            _ => panic!("Expected NdArray backend"),
        }
    }

    #[test]
    fn test_create_backend_mymat_and_use() {
        let config = LinalgConfig::mymat();
        let backend_enum = create_backend(&config);
        match backend_enum {
            LinalgBackendImpl::MyMat(b) => {
                let eye = b.eye(3).unwrap();
                assert_eq!(b.nrows(&eye), 3);
                let val = b.get_matrix(&eye, 0, 0).unwrap();
                assert!((val - Complex64::new(1.0, 0.0)).norm() < 1e-10);
                let val_off = b.get_matrix(&eye, 0, 1).unwrap();
                assert!((val_off - Complex64::new(0.0, 0.0)).norm() < 1e-10);
            }
            _ => panic!("Expected MyMat backend"),
        }
    }

    // ========================================================================
    // LinalgScalar trait
    // ========================================================================

    #[test]
    fn test_linalg_scalar_f64() {
        assert_eq!(<f64 as LinalgScalar>::zero(), 0.0);
        assert_eq!(<f64 as LinalgScalar>::one(), 1.0);
        assert_eq!(<f64 as LinalgScalar>::from_f64(3.14), 3.14);
        assert_eq!(<f64 as LinalgScalar>::to_f64(3.14), 3.14);
        assert_eq!(<f64 as LinalgScalar>::norm(&5.0), 5.0);
        assert_eq!(<f64 as LinalgScalar>::norm_sqr(&3.0), 9.0);
    }

    #[test]
    fn test_linalg_scalar_complex64() {
        let z = c(3.0, 4.0);
        assert_eq!(c(0.0, 0.0), <Complex64 as LinalgScalar>::zero());
        assert_eq!(c(1.0, 0.0), <Complex64 as LinalgScalar>::one());
        assert!((<Complex64 as LinalgScalar>::norm(&z) - 5.0).abs() < 1e-10);
        assert!((<Complex64 as LinalgScalar>::norm_sqr(&z) - 25.0).abs() < 1e-10);
    }
}
