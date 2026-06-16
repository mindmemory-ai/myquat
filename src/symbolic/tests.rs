//! Unit Tests for Symbolic Computation Module
//!
//! Author: gA4ss
//!
//! Comprehensive tests for the symbolic backend and Symbolica adapter.
//!
//! Note: Due to Symbolica's licensing restrictions in unlicensed mode,
//! only one instance can run at a time. The main test is marked with #[ignore]
//! and must be run separately:
//!
//! ```bash
//! cargo test --lib symbolic::tests::test_symbolic_backend_comprehensive -- --ignored
//! ```

#[cfg(test)]
mod tests {
    use crate::symbolic::{SymbolicBackend, SymbolicExpression, SymbolicaBackend};
    use std::collections::HashMap;

    #[test]
    #[ignore = "Symbolica licensing: run separately with --ignored flag"]
    fn test_symbolic_backend_comprehensive() {
        let backend = SymbolicaBackend::new();

        // Test variable creation
        let x = backend.variable("x").unwrap();
        let y = backend.variable("y").unwrap();
        assert!(!x.is_zero());
        assert!(!x.is_one());

        // Test constants
        let zero = backend.constant(0.0).unwrap();
        assert!(zero.is_zero());
        let one = backend.constant(1.0).unwrap();
        assert!(one.is_one());
        let two = backend.constant(2.0).unwrap();

        // Test parsing
        let expr = backend.parse("x^2 + 2*x + 1").unwrap();
        assert!(!expr.is_zero());

        // Test arithmetic operations
        let sum = backend.add(&x, &y).unwrap();
        assert!(!sum.is_zero());

        let diff = backend.sub(&x, &y).unwrap();
        assert!(!diff.is_zero());

        let prod = backend.mul(&x, &two).unwrap();
        assert!(!prod.is_zero());

        let quot = backend.div(&x, &two).unwrap();
        assert!(!quot.is_zero());

        assert!(backend.div(&x, &zero).is_err());

        let squared = backend.pow(&x, &two).unwrap();
        assert!(!squared.is_zero());

        let neg_x = backend.neg(&x).unwrap();
        assert!(!neg_x.is_zero());

        // Test mathematical functions
        let exp_x = backend.exp(&x).unwrap();
        assert!(!exp_x.is_zero());

        let ln_x = backend.ln(&x).unwrap();
        assert!(!ln_x.is_zero());

        let sin_x = backend.sin(&x).unwrap();
        assert!(!sin_x.is_zero());

        let cos_x = backend.cos(&x).unwrap();
        assert!(!cos_x.is_zero());

        let sqrt_x = backend.sqrt(&x).unwrap();
        assert!(!sqrt_x.is_zero());

        let abs_x = backend.abs(&x).unwrap();
        assert!(!abs_x.is_zero());

        // Test differentiation
        let x_squared = backend.parse("x^2").unwrap();
        let deriv = backend.differentiate(&x_squared, "x", 1).unwrap();
        assert!(!deriv.is_zero());

        let x_cubed = backend.parse("x^3").unwrap();
        let deriv2 = backend.differentiate(&x_cubed, "x", 2).unwrap();
        assert!(!deriv2.is_zero());

        // Test simplification operations
        let expr2 = backend.parse("(x+1)^2").unwrap();
        let expanded = backend.expand(&expr2).unwrap();
        assert!(!expanded.is_zero());

        let expr3 = backend.parse("x^2 - 1").unwrap();
        let factored = backend.factor(&expr3).unwrap();
        assert!(!factored.is_zero());

        let expr4 = backend.parse("x + x").unwrap();
        let simplified = backend.simplify(&expr4).unwrap();
        assert!(!simplified.is_zero());

        let expr5 = backend.parse("x + 2*x + 3*x").unwrap();
        let collected = backend.collect(&expr5, "x").unwrap();
        assert!(!collected.is_zero());

        // Test substitution
        let expr6 = backend.parse("x + y").unwrap();
        let mut subs = HashMap::new();
        subs.insert("x".to_string(), one.clone());
        subs.insert("y".to_string(), two.clone());
        let result = backend.substitute(&expr6, &subs).unwrap();
        assert!(!result.is_zero());

        // Test matrix operations
        let a = backend.constant(1.0).unwrap();
        let b = backend.constant(2.0).unwrap();
        let c = backend.constant(3.0).unwrap();
        let d = backend.constant(4.0).unwrap();

        let matrix = backend
            .matrix(vec![vec![a.clone(), b.clone()], vec![c.clone(), d.clone()]])
            .unwrap();

        assert_eq!(matrix.rows, 2);
        assert_eq!(matrix.cols, 2);
        assert!(matrix.is_square());

        let elem = matrix.get(0, 0).unwrap();
        assert!(elem.is_one());

        // Test matrix multiplication
        let identity = backend
            .matrix(vec![
                vec![one.clone(), zero.clone()],
                vec![zero.clone(), one.clone()],
            ])
            .unwrap();

        let mat_result = backend.matrix_mul(&identity, &identity).unwrap();
        assert_eq!(mat_result.rows, 2);
        assert_eq!(mat_result.cols, 2);

        // Test determinant
        let det = backend.determinant(&matrix).unwrap();
        assert!(!det.is_zero());

        // Test trace
        let tr = backend.trace(&matrix).unwrap();
        assert!(!tr.is_zero());

        // Test commutator
        let mat_a = backend
            .matrix(vec![
                vec![x.clone(), zero.clone()],
                vec![zero.clone(), x.clone()],
            ])
            .unwrap();

        let mat_b = backend
            .matrix(vec![
                vec![y.clone(), zero.clone()],
                vec![zero.clone(), y.clone()],
            ])
            .unwrap();

        let comm = backend.commutator(&mat_a, &mat_b).unwrap();
        assert_eq!(comm.rows, 2);
        assert_eq!(comm.cols, 2);

        // Test expectation value
        let state = backend
            .matrix(vec![vec![one.clone()], vec![zero.clone()]])
            .unwrap();

        let expectation = backend.expectation_value(&identity, &state).unwrap();
        assert!(!expectation.is_zero());

        // Test display and to_string
        let display_str = format!("{}", x);
        assert!(!display_str.is_empty());

        let str_repr = SymbolicExpression::to_string(&x);
        assert!(!str_repr.is_empty());
    }
}
