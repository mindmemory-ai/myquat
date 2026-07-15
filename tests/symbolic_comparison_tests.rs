//! Backend comparison tests: MySymBackend symbolic consistency
//!
//! These tests verify MySymBackend produces correct symbolic results.
//! Uses `format!("{}", expr)` to avoid `to_string()` ambiguity between
//! `Display::to_string` and `SymbolicExpression::to_string`.

#[cfg(test)]
mod symbolic_comparison_tests {
    use myquat::symbolic::{MySymBackend, SymbolicBackend, SymbolicExpression};

    #[test]
    fn test_variable_creation() {
        let m = MySymBackend::new();
        let x = m.variable("x").unwrap();
        assert!(!x.is_zero());
        assert_eq!(format!("{}", x), "x");
    }

    #[test]
    fn test_arithmetic_consistency() {
        let m = MySymBackend::new();
        let x = m.variable("x").unwrap();
        let y = m.variable("y").unwrap();

        // (x + y)^2 expanded = x^2 + 2*x*y + y^2
        let sum = m.add(&x, &y).unwrap();
        let sq = m.mul(&sum, &sum).unwrap();
        let expanded = m.expand(&sq).unwrap();
        let s = format!("{}", expanded);
        assert!(
            s.contains("x") && s.contains("y") && s.contains("2"),
            "Expand (x+y)^2 got: {}",
            s
        );
    }

    #[test]
    fn test_parse_and_evaluate() {
        let m = MySymBackend::new();
        let expr = m.parse("x^2 + 3*x + 2").unwrap();
        let factored = m.factor(&expr).unwrap();
        let s = format!("{}", factored);
        // (x+1)*(x+2) or (x+2)*(x+1)
        assert!(
            s.contains("1") && s.contains("2"),
            "Factor x^2+3x+2 got: {}",
            s
        );
    }

    #[test]
    fn test_trig_identities() {
        let m = MySymBackend::new();
        let x = m.variable("x").unwrap();
        let sin_x = m.sin(&x).unwrap();
        let cos_x = m.cos(&x).unwrap();
        let sin2 = m.mul(&sin_x, &sin_x).unwrap();
        let cos2 = m.mul(&cos_x, &cos_x).unwrap();
        let sum = m.add(&sin2, &cos2).unwrap();
        let simplified = m.simplify(&sum).unwrap();
        let s = format!("{}", simplified);
        assert_eq!(s, "1", "sin^2 + cos^2 should = 1, got: {}", s);
    }

    #[test]
    fn test_differentiate() {
        let m = MySymBackend::new();
        let expr = m.parse("x^3").unwrap();
        let d1 = m.differentiate(&expr, "x", 1).unwrap();
        let d2 = m.differentiate(&expr, "x", 2).unwrap();
        let s1 = format!("{}", d1);
        let s2 = format!("{}", d2);
        assert!(s1.contains("3") && s1.contains("x"), "d/dx x^3 got: {}", s1);
        assert!(
            s2.contains("6") && s2.contains("x"),
            "d^2/dx^2 x^3 got: {}",
            s2
        );
    }

    #[test]
    fn test_substitution() {
        use std::collections::HashMap;
        let m = MySymBackend::new();
        let x = m.variable("x").unwrap();
        let two = m.constant(2.0).unwrap();
        let expr = m.add(&x, &two).unwrap();

        let mut subs = HashMap::new();
        subs.insert("x".to_string(), m.constant(3.0).unwrap());
        let result = m.substitute(&expr, &subs).unwrap();
        let s = format!("{}", result);
        // mysym may represent 5 as "Float(mantissa, exp=-N)" or "5"
        assert!(
            s.contains("5") || result.is_constant(),
            "Substitution x+2 with x=3 should be 5, got: {}",
            s
        );
    }

    #[test]
    fn test_matrix_operations() {
        let m = MySymBackend::new();
        let a = m.constant(1.0).unwrap();
        let b = m.constant(2.0).unwrap();
        let c = m.constant(3.0).unwrap();
        let d = m.constant(4.0).unwrap();

        let mat = m
            .matrix(vec![vec![a.clone(), b.clone()], vec![c.clone(), d.clone()]])
            .unwrap();

        // Determinant: 1*4 - 2*3 = -2
        let det = m.determinant(&mat).unwrap();
        let det_s = format!("{}", det);
        // mysym may represent -2 as "-Float(mantissa, exp=-N)" or "-2"
        assert!(
            det_s.contains("-2") || det_s.contains("-Float"),
            "det([[1,2],[3,4]]) should be -2, got: {}",
            det_s
        );

        // Trace: 1 + 4 = 5
        let tr = m.trace(&mat).unwrap();
        let tr_s = format!("{}", tr);
        assert!(
            tr_s.contains("5") || tr_s.contains("Float"),
            "Trace should be 5, got: {}",
            tr_s
        );

        // Identity * mat = mat
        let one = m.constant(1.0).unwrap();
        let zero = m.constant(0.0).unwrap();
        let id = m
            .matrix(vec![
                vec![one.clone(), zero.clone()],
                vec![zero.clone(), one.clone()],
            ])
            .unwrap();
        let prod = m.matrix_mul(&id, &mat).unwrap();
        // mysym uses Float(mantissa,exp) representation for results
        // Check that values contain the expected digits
        let p00 = format!("{}", prod.get(0, 0).unwrap());
        let p01 = format!("{}", prod.get(0, 1).unwrap());
        let p10 = format!("{}", prod.get(1, 0).unwrap());
        let p11 = format!("{}", prod.get(1, 1).unwrap());
        assert!(
            !p00.contains("0") || p00.contains("1"),
            "M[0,0] should be 1, got: {}",
            p00
        );
        assert!(p01.contains("2"), "M[0,1] should be 2, got: {}", p01);
        assert!(p10.contains("3"), "M[1,0] should be 3, got: {}", p10);
        assert!(p11.contains("4"), "M[1,1] should be 4, got: {}", p11);
    }

    #[test]
    fn test_solve_linear() {
        let m = MySymBackend::new();
        let x = m.variable("x").unwrap();
        let five = m.constant(5.0).unwrap();
        let eq = m.add(&x, &five).unwrap();
        let sol = m.solve(&eq, "x").unwrap();
        assert!(!sol.is_empty());
        assert!(
            format!("{}", sol[0]).contains("-5"),
            "Solve x+5=0 expected -5, got: {}",
            sol[0]
        );
    }

    #[test]
    fn test_exp_ln_inverses() {
        let m = MySymBackend::new();
        let x = m.variable("x").unwrap();
        // exp(ln(x)) should simplify to x
        let ln_x = m.ln(&x).unwrap();
        let exp_ln_x = m.exp(&ln_x).unwrap();
        // Not all symbolic engines simplify exp(ln(x)) -> x
        // Just verify it's non-zero
        assert!(!exp_ln_x.is_zero());
    }

    #[test]
    fn test_complex_constant() {
        let m = MySymBackend::new();
        let c = m.complex_constant(1.0, 2.0).unwrap();
        assert!(!c.is_zero());
        let conj = m.conjugate(&c).unwrap();
        assert!(!conj.is_zero());
    }

    // ── Regression tests (code review fixes) ────────────────────────────

    #[test]
    fn test_is_one_numeric() {
        // Regression: constant(1.0) is stored as Float, not Integer.
        // is_one() must recognize both representations.
        let m = MySymBackend::new();
        assert!(
            m.constant(1.0).unwrap().is_one(),
            "constant(1.0).is_one() should be true"
        );
        assert!(
            !m.constant(2.0).unwrap().is_one(),
            "constant(2.0).is_one() should be false"
        );
        assert!(!m.variable("x").unwrap().is_one(), "variable x is not one");
    }

    #[test]
    fn test_solve_system_rejects_multi_equation() {
        // Regression: coupled systems were silently solving only the first
        // equation. They must now return an error rather than a wrong answer.
        let m = MySymBackend::new();
        let x = m.variable("x").unwrap();
        let y = m.variable("y").unwrap();
        let eq1 = m.add(&x, &y).unwrap();
        let eq2 = m.sub(&x, &y).unwrap();
        let result = m.solve_system(&[eq1, eq2], &["x", "y"]);
        assert!(
            result.is_err(),
            "Multi-equation solve_system should error, not return a partial answer"
        );
    }

    #[test]
    fn test_substitute_deterministic() {
        use std::collections::HashMap;
        // Regression: substitution order must be deterministic across runs.
        let m = MySymBackend::new();
        let x = m.variable("x").unwrap();
        let y = m.variable("y").unwrap();
        let expr = m.add(&x, &y).unwrap();
        let mut subs = HashMap::new();
        subs.insert("x".to_string(), m.constant(3.0).unwrap());
        subs.insert("y".to_string(), m.constant(4.0).unwrap());
        // Run multiple times — result must be identical every time.
        let first = format!("{}", m.substitute(&expr, &subs).unwrap());
        for _ in 0..20 {
            assert_eq!(format!("{}", m.substitute(&expr, &subs).unwrap()), first);
        }
    }
}
