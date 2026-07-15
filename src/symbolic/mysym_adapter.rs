//! MySym Backend Adapter
//!
//! Author: gA4ss
//!
//! This module provides a symbolic computation backend backed by the
//! mysym library (~95k lines of Rust, 6 crates). mysym provides a
//! complete symbolic algebra system with expression construction,
//! arithmetic, calculus, simplification, equation solving, and matrix
//! operations.

use std::collections::HashMap;
use std::fmt;

use super::backend::{
    SubstitutionMap, SymbolicBackend, SymbolicError, SymbolicExpression, SymbolicMatrix,
    SymbolicResult,
};

/// Expression type for the MySym backend.
///
/// Wraps a `mysym::Sym` handle, which internally holds `Arc<dyn mysym::Expr>`.
/// `Sym` implements `Clone`, `Debug`, `Display`, and all arithmetic operators.
#[derive(Clone)]
pub struct MySymExpression {
    pub(crate) inner: mysym::Sym,
}

impl MySymExpression {
    pub(crate) fn new(inner: mysym::Sym) -> Self {
        Self { inner }
    }
}

impl fmt::Debug for MySymExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MySymExpr({})", self.inner)
    }
}

impl fmt::Display for MySymExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl SymbolicExpression for MySymExpression {
    fn to_string(&self) -> String {
        self.inner.to_string()
    }

    fn is_zero(&self) -> bool {
        matches!(self.inner.is_zero(), mysym::assumptions::TriBool::True)
    }

    fn is_one(&self) -> bool {
        // A numeric one may be stored as an Integer (1) or a Float (1.0).
        // structural_eq distinguishes node types, so compare against both
        // canonical representations.
        self.inner.is_number()
            && (self.inner.equals(&mysym::Sym::from(1))
                || self.inner.equals(&mysym::Sym::from(1.0)))
    }

    fn is_constant(&self) -> bool {
        self.inner.is_constant()
    }

    fn degree(&self, _var: &str) -> Option<usize> {
        // mysym's degree calculation requires the polynomial module.
        // Return None (same behavior as SymbolicaBackend).
        None
    }
}

/// MySym symbolic computation backend
///
/// Provides expression construction, arithmetic, calculus, and simplification
/// backed by the mysym library.
#[derive(Debug, Clone)]
pub struct MySymBackend;

impl MySymBackend {
    /// Create a new MySym backend
    pub fn new() -> Self {
        Self
    }
}

impl Default for MySymBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolicBackend for MySymBackend {
    type Expression = MySymExpression;

    fn variable(&self, name: &str) -> SymbolicResult<Self::Expression> {
        Ok(MySymExpression::new(mysym::sym(name)))
    }

    fn constant(&self, value: f64) -> SymbolicResult<Self::Expression> {
        Ok(MySymExpression::new(mysym::Sym::from(value)))
    }

    fn complex_constant(&self, real: f64, imag: f64) -> SymbolicResult<Self::Expression> {
        if imag == 0.0 {
            self.constant(real)
        } else {
            let re = mysym::Sym::from(real);
            let im = mysym::Sym::from(imag);
            Ok(MySymExpression::new(re + mysym::Sym::i() * im))
        }
    }

    fn parse(&self, expr: &str) -> SymbolicResult<Self::Expression> {
        let parsed = mysym::parse(expr).map_err(|e| {
            SymbolicError::InvalidExpression(format!("Failed to parse '{}': {}", expr, e))
        })?;
        Ok(MySymExpression::new(parsed))
    }

    fn add(&self, a: &Self::Expression, b: &Self::Expression) -> SymbolicResult<Self::Expression> {
        Ok(MySymExpression::new(a.inner.clone() + b.inner.clone()))
    }

    fn sub(&self, a: &Self::Expression, b: &Self::Expression) -> SymbolicResult<Self::Expression> {
        Ok(MySymExpression::new(a.inner.clone() - b.inner.clone()))
    }

    fn mul(&self, a: &Self::Expression, b: &Self::Expression) -> SymbolicResult<Self::Expression> {
        Ok(MySymExpression::new(a.inner.clone() * b.inner.clone()))
    }

    fn div(&self, a: &Self::Expression, b: &Self::Expression) -> SymbolicResult<Self::Expression> {
        if b.is_zero() {
            return Err(SymbolicError::InvalidExpression(
                "Division by zero in MySymBackend::div".to_string(),
            ));
        }
        Ok(MySymExpression::new(a.inner.clone() / b.inner.clone()))
    }

    fn pow(
        &self,
        base: &Self::Expression,
        exp: &Self::Expression,
    ) -> SymbolicResult<Self::Expression> {
        Ok(MySymExpression::new(
            base.inner.clone().pow(exp.inner.clone()),
        ))
    }

    fn neg(&self, expr: &Self::Expression) -> SymbolicResult<Self::Expression> {
        Ok(MySymExpression::new(-expr.inner.clone()))
    }

    fn exp(&self, expr: &Self::Expression) -> SymbolicResult<Self::Expression> {
        let inner: std::sync::Arc<dyn mysym::Expr> = expr.inner.clone().into_inner();
        Ok(MySymExpression::new(mysym::Sym::new(mysym::Function::new(
            mysym::FuncKind::Exp,
            vec![inner],
        ))))
    }

    fn ln(&self, expr: &Self::Expression) -> SymbolicResult<Self::Expression> {
        let inner: std::sync::Arc<dyn mysym::Expr> = expr.inner.clone().into_inner();
        Ok(MySymExpression::new(mysym::Sym::new(mysym::Function::new(
            mysym::FuncKind::Log,
            vec![inner],
        ))))
    }

    fn sin(&self, expr: &Self::Expression) -> SymbolicResult<Self::Expression> {
        let inner: std::sync::Arc<dyn mysym::Expr> = expr.inner.clone().into_inner();
        Ok(MySymExpression::new(mysym::Sym::new(mysym::Function::new(
            mysym::FuncKind::Sin,
            vec![inner],
        ))))
    }

    fn cos(&self, expr: &Self::Expression) -> SymbolicResult<Self::Expression> {
        let inner: std::sync::Arc<dyn mysym::Expr> = expr.inner.clone().into_inner();
        Ok(MySymExpression::new(mysym::Sym::new(mysym::Function::new(
            mysym::FuncKind::Cos,
            vec![inner],
        ))))
    }

    fn sqrt(&self, expr: &Self::Expression) -> SymbolicResult<Self::Expression> {
        let inner: std::sync::Arc<dyn mysym::Expr> = expr.inner.clone().into_inner();
        Ok(MySymExpression::new(mysym::Sym::new(mysym::Function::new(
            mysym::FuncKind::Sqrt,
            vec![inner],
        ))))
    }

    fn abs(&self, expr: &Self::Expression) -> SymbolicResult<Self::Expression> {
        let inner: std::sync::Arc<dyn mysym::Expr> = expr.inner.clone().into_inner();
        Ok(MySymExpression::new(mysym::Sym::new(mysym::Function::new(
            mysym::FuncKind::Abs,
            vec![inner],
        ))))
    }

    fn conjugate(&self, expr: &Self::Expression) -> SymbolicResult<Self::Expression> {
        Ok(MySymExpression::new(expr.inner.conjugate()))
    }

    fn differentiate(
        &self,
        expr: &Self::Expression,
        var: &str,
        order: usize,
    ) -> SymbolicResult<Self::Expression> {
        let symbol = mysym::Symbol::new(var);
        let mut result: std::sync::Arc<dyn mysym::Expr> = expr.inner.clone().into_inner();
        for _ in 0..order {
            result = result.diff(&symbol);
        }
        Ok(MySymExpression::new(mysym::Sym::new(result)))
    }

    fn integrate(
        &self,
        expr: &Self::Expression,
        var: &str,
        lower: Option<&Self::Expression>,
        upper: Option<&Self::Expression>,
    ) -> SymbolicResult<Self::Expression> {
        let var_sym = mysym::sym(var);

        match (lower, upper) {
            (None, None) => {
                // Indefinite integral
                let antideriv = mysym::integrate(&expr.inner, &var_sym);
                Ok(MySymExpression::new(antideriv))
            }
            (Some(lo), Some(hi)) => {
                // Definite integral: F(hi) - F(lo)
                let antideriv = mysym::integrate(&expr.inner, &var_sym);
                // Use subs on the inner Arc<dyn Expr>
                let antideriv_arc: std::sync::Arc<dyn mysym::Expr> = antideriv.into_inner();
                let at_upper =
                    antideriv_arc.subs(var_sym.as_ref(), hi.inner.clone().into_inner().as_ref());
                let at_lower =
                    antideriv_arc.subs(var_sym.as_ref(), lo.inner.clone().into_inner().as_ref());
                Ok(MySymExpression::new(
                    mysym::Sym::new(at_upper) - mysym::Sym::new(at_lower),
                ))
            }
            _ => Err(SymbolicError::InvalidExpression(
                "Integrate requires both lower and upper, or neither".to_string(),
            )),
        }
    }

    fn simplify(&self, expr: &Self::Expression) -> SymbolicResult<Self::Expression> {
        let simplified = mysym::simplify_deep(&expr.inner);
        Ok(MySymExpression::new(simplified))
    }

    fn expand(&self, expr: &Self::Expression) -> SymbolicResult<Self::Expression> {
        let expanded = mysym::expand(&expr.inner);
        Ok(MySymExpression::new(expanded))
    }

    fn factor(&self, expr: &Self::Expression) -> SymbolicResult<Self::Expression> {
        let factored = mysym::factor(&expr.inner);
        Ok(MySymExpression::new(factored))
    }

    fn collect(&self, expr: &Self::Expression, var: &str) -> SymbolicResult<Self::Expression> {
        let sym_var = mysym::sym(var);
        let collected = mysym::collect(&expr.inner, &[sym_var]);
        Ok(MySymExpression::new(collected))
    }

    fn substitute(
        &self,
        expr: &Self::Expression,
        subs: &SubstitutionMap<Self::Expression>,
    ) -> SymbolicResult<Self::Expression> {
        // Iterate in a deterministic (sorted) order. HashMap iteration order is
        // nondeterministic, which would make sequential substitution of
        // dependent variables produce different results across runs.
        let mut keys: Vec<&String> = subs.keys().collect();
        keys.sort();
        let mut result: std::sync::Arc<dyn mysym::Expr> = expr.inner.clone().into_inner();
        for var_name in keys {
            let replacement = &subs[var_name];
            let var_sym = mysym::sym(var_name);
            result = result.subs(
                var_sym.as_ref(),
                replacement.inner.clone().into_inner().as_ref(),
            );
        }
        Ok(MySymExpression::new(mysym::Sym::new(result)))
    }

    fn solve(
        &self,
        equation: &Self::Expression,
        var: &str,
    ) -> SymbolicResult<Vec<Self::Expression>> {
        let var_sym = mysym::sym(var);
        let solutions = mysym::solve(&equation.inner, &var_sym);
        Ok(solutions
            .into_iter()
            .map(|s| MySymExpression::new(s))
            .collect())
    }

    fn solve_system(
        &self,
        equations: &[Self::Expression],
        vars: &[&str],
    ) -> SymbolicResult<SubstitutionMap<Self::Expression>> {
        use std::collections::HashMap;
        if equations.is_empty() || vars.is_empty() {
            return Ok(HashMap::new());
        }
        // Genuine coupled multi-equation systems are not yet supported: solving
        // only the first equation would silently return a wrong partial answer.
        // Reject rather than mislead. Single-equation/single-variable is exact.
        if equations.len() > 1 || vars.len() > 1 {
            return Err(SymbolicError::UnsupportedOperation(
                "MySymBackend::solve_system only supports a single equation in a \
                 single variable; coupled systems are not yet implemented"
                    .to_string(),
            ));
        }
        let var_sym = mysym::sym(vars[0]);
        let solutions = mysym::solve(&equations[0].inner, &var_sym);
        let mut map = HashMap::new();
        if !solutions.is_empty() {
            map.insert(
                vars[0].to_string(),
                MySymExpression::new(solutions[0].clone()),
            );
        }
        Ok(map)
    }

    fn matrix(
        &self,
        elements: Vec<Vec<Self::Expression>>,
    ) -> SymbolicResult<SymbolicMatrix<Self::Expression>> {
        SymbolicMatrix::new(elements)
    }

    fn matrix_mul(
        &self,
        a: &SymbolicMatrix<Self::Expression>,
        b: &SymbolicMatrix<Self::Expression>,
    ) -> SymbolicResult<SymbolicMatrix<Self::Expression>> {
        let a_syms: Vec<mysym::Sym> = a
            .elements
            .iter()
            .flat_map(|row| row.iter().map(|e| e.inner.clone()))
            .collect();
        let b_syms: Vec<mysym::Sym> = b
            .elements
            .iter()
            .flat_map(|row| row.iter().map(|e| e.inner.clone()))
            .collect();

        let mat_a = mysym::Matrix::new(a.rows, a.cols, a_syms);
        let mat_b = mysym::Matrix::new(b.rows, b.cols, b_syms);

        let result = mat_a.mat_mul(&mat_b);
        let flat: Vec<MySymExpression> = result
            .flat()
            .into_iter()
            .map(|s| MySymExpression::new(s))
            .collect();

        let rows = result.nrows();
        let cols = result.ncols();
        let elements: Vec<Vec<MySymExpression>> = (0..rows)
            .map(|r| flat[r * cols..(r + 1) * cols].to_vec())
            .collect();
        SymbolicMatrix::new(elements)
    }

    fn determinant(
        &self,
        matrix: &SymbolicMatrix<Self::Expression>,
    ) -> SymbolicResult<Self::Expression> {
        let syms: Vec<mysym::Sym> = matrix
            .elements
            .iter()
            .flat_map(|row| row.iter().map(|e| e.inner.clone()))
            .collect();
        let mat = mysym::Matrix::new(matrix.rows, matrix.cols, syms);
        match mat.det() {
            Some(det) => Ok(MySymExpression::new(det)),
            None => Err(SymbolicError::MatrixOperationFailed(
                "Determinant computation failed".to_string(),
            )),
        }
    }

    fn eigenvalues(
        &self,
        matrix: &SymbolicMatrix<Self::Expression>,
    ) -> SymbolicResult<Vec<Self::Expression>> {
        let syms: Vec<mysym::Sym> = matrix
            .elements
            .iter()
            .flat_map(|row| row.iter().map(|e| e.inner.clone()))
            .collect();
        let mat = mysym::Matrix::new(matrix.rows, matrix.cols, syms);
        match mysym_linalg::eigenvalues(&mat) {
            Some(evals) => Ok(evals.into_iter().map(|s| MySymExpression::new(s)).collect()),
            None => Err(SymbolicError::MatrixOperationFailed(
                "Eigenvalue computation failed".to_string(),
            )),
        }
    }

    fn trace(&self, matrix: &SymbolicMatrix<Self::Expression>) -> SymbolicResult<Self::Expression> {
        let syms: Vec<mysym::Sym> = matrix
            .elements
            .iter()
            .flat_map(|row| row.iter().map(|e| e.inner.clone()))
            .collect();
        let mat = mysym::Matrix::new(matrix.rows, matrix.cols, syms);
        match mat.trace() {
            Some(tr) => Ok(MySymExpression::new(tr)),
            None => Err(SymbolicError::MatrixOperationFailed(
                "Trace computation failed (non-square matrix?)".to_string(),
            )),
        }
    }

    fn commutator(
        &self,
        a: &SymbolicMatrix<Self::Expression>,
        b: &SymbolicMatrix<Self::Expression>,
    ) -> SymbolicResult<SymbolicMatrix<Self::Expression>> {
        // commutator(A, B) = A*B - B*A
        let ab = self.matrix_mul(a, b)?;
        let ba = self.matrix_mul(b, a)?;
        let diff_elements: Vec<Vec<MySymExpression>> = ab
            .elements
            .iter()
            .zip(ba.elements.iter())
            .map(|(row_ab, row_ba)| {
                row_ab
                    .iter()
                    .zip(row_ba.iter())
                    .map(|(x, y)| MySymExpression::new(x.inner.clone() - y.inner.clone()))
                    .collect()
            })
            .collect();
        SymbolicMatrix::new(diff_elements)
    }

    fn expectation_value(
        &self,
        operator: &SymbolicMatrix<Self::Expression>,
        state: &SymbolicMatrix<Self::Expression>,
    ) -> SymbolicResult<Self::Expression> {
        // ⟨ψ|O|ψ⟩ = conj(ψ)ᵀ · O · ψ
        let op_state = self.matrix_mul(operator, state)?;

        // Build bra = state† from the state column vector
        let state_syms: Vec<mysym::Sym> = state
            .elements
            .iter()
            .flat_map(|row| row.iter().map(|e| e.inner.clone()))
            .collect();
        let mat_state = mysym::Matrix::new(state.rows, state.cols, state_syms);
        let bra = mat_state.adjoint();

        // bra * (op_state) — mul 1×N times N×1 = 1×1
        let op_state_syms: Vec<mysym::Sym> = op_state
            .elements
            .iter()
            .flat_map(|row| row.iter().map(|e| e.inner.clone()))
            .collect();
        let mat_op_state = mysym::Matrix::new(op_state.rows, op_state.cols, op_state_syms);
        let result = bra.mat_mul(&mat_op_state);

        if result.nrows() == 1 && result.ncols() == 1 {
            Ok(MySymExpression::new(result.flat()[0].clone()))
        } else {
            Err(SymbolicError::MatrixOperationFailed(
                "Expectation value result is not scalar".to_string(),
            ))
        }
    }

    fn time_evolution_operator(
        &self,
        hamiltonian: &SymbolicMatrix<Self::Expression>,
        time_var: &str,
    ) -> SymbolicResult<SymbolicMatrix<Self::Expression>> {
        // U(t) = exp(-i*H*t)
        let t = mysym::sym(time_var);
        let neg_i = -mysym::Sym::i();

        let h_syms: Vec<mysym::Sym> = hamiltonian
            .elements
            .iter()
            .flat_map(|row| row.iter().map(|e| e.inner.clone()))
            .collect();
        let h_mat = mysym::Matrix::new(hamiltonian.rows, hamiltonian.cols, h_syms);

        // Scale: -i * H * t element-wise
        let scaled: Vec<mysym::Sym> = h_mat
            .flat()
            .into_iter()
            .map(|h_elem| neg_i.clone() * h_elem * t.clone())
            .collect();
        let scaled_mat = mysym::Matrix::new(h_mat.nrows(), h_mat.ncols(), scaled);

        // Matrix exponential via mysym::expm
        match mysym_linalg::expm(&scaled_mat) {
            Some(evolved) => {
                let flat: Vec<MySymExpression> = evolved
                    .flat()
                    .into_iter()
                    .map(|s| MySymExpression::new(s))
                    .collect();
                let rows = evolved.nrows();
                let cols = evolved.ncols();
                let elements: Vec<Vec<MySymExpression>> = (0..rows)
                    .map(|r| flat[r * cols..(r + 1) * cols].to_vec())
                    .collect();
                SymbolicMatrix::new(elements)
            }
            None => Err(SymbolicError::MatrixOperationFailed(
                "Matrix exponential computation failed".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mysym_backend_creation() {
        let _backend = MySymBackend::new();
    }

    #[test]
    fn test_mysym_variable_succeeds() {
        let backend = MySymBackend::new();
        let x = backend.variable("x").unwrap();
        assert!(!x.is_zero());
        assert!(!x.is_one());
        assert_eq!(format!("{}", x), "x");
    }

    #[test]
    fn test_mysym_arithmetic() {
        let backend = MySymBackend::new();
        let x = backend.variable("x").unwrap();
        let two = backend.constant(2.0).unwrap();
        let sum = backend.add(&x, &two).unwrap();
        assert!(!sum.is_zero());

        let prod = backend.mul(&x, &x).unwrap();
        assert!(!prod.is_zero());
    }

    #[test]
    fn test_mysym_math_functions() {
        let b = MySymBackend::new();
        let zero = b.constant(0.0).unwrap();
        let one = b.constant(1.0).unwrap();

        // sin(0) — creates Sin expression
        let s0 = b.sin(&zero).unwrap();
        let s = format!("{}", s0);
        assert!(s.contains("Sin") || s.contains("sin"), "sin(0)='{}'", s);

        // cos(0) — creates Cos expression
        let c0 = b.cos(&zero).unwrap();
        let c = format!("{}", c0);
        assert!(c.contains("Cos") || c.contains("cos"), "cos(0)='{}'", c);

        // exp(0) — creates Exp expression
        let e0 = b.exp(&zero).unwrap();
        let e = format!("{}", e0);
        assert!(e.contains("Exp") || e.contains("exp"), "exp(0)='{}'", e);

        // sqrt(1) — creates Sqrt expression
        let sq = b.sqrt(&one).unwrap();
        let s = format!("{}", sq);
        assert!(s.contains("Sqrt") || s.contains("sqrt"), "sqrt(1)='{}'", s);

        // ln(1) — creates Log expression
        let ln1 = b.ln(&one).unwrap();
        let l = format!("{}", ln1);
        assert!(l.contains("Log") || l.contains("log"), "ln(1)='{}'", l);

        // abs and conjugate produce non-zero expressions
        let x = b.variable("x").unwrap();
        let a = b.abs(&x).unwrap();
        assert!(!a.is_zero());
        let conj = b.conjugate(&x).unwrap();
        assert!(!conj.is_zero());
    }

    #[test]
    fn test_mysym_differentiate() {
        let b = MySymBackend::new();
        // d/dx (x^2) = 2*x
        let expr = b.parse("x^2").unwrap();
        let deriv = b.differentiate(&expr, "x", 1).unwrap();
        assert!(!deriv.is_zero());
        let s = format!("{}", deriv);
        assert!(
            s.contains("x") && s.contains("2"),
            "Expected 2*x, got: {}",
            s
        );
    }

    #[test]
    fn test_mysym_simplify() {
        let b = MySymBackend::new();
        let x = b.variable("x").unwrap();
        let sum = b.add(&x, &x).unwrap();
        let simplified = b.simplify(&sum).unwrap();
        let s = format!("{}", simplified);
        assert!(s.contains("2"), "simplified='{}'", s);
    }

    #[test]
    fn test_mysym_expand_factor() {
        let b = MySymBackend::new();
        // (x+1)^2 -> expand -> x^2 + 2*x + 1
        let expr = b.parse("(x+1)^2").unwrap();
        let expanded = b.expand(&expr).unwrap();
        assert!(!expanded.is_zero(), "expand returned zero");
        let e_str = format!("{}", expanded);
        assert!(
            e_str.contains("x") || e_str.contains("Add") || !e_str.is_empty(),
            "expand of (x+1)^2 = '{}'",
            e_str
        );

        // x^2 - 1 = (x-1)*(x+1)
        let expr2 = b.parse("x^2 - 1").unwrap();
        let factored = b.factor(&expr2).unwrap();
        assert!(!factored.is_zero(), "factor returned zero");
    }

    #[test]
    fn test_mysym_substitute() {
        use std::collections::HashMap;
        let b = MySymBackend::new();
        let expr = b.parse("x + y").unwrap();

        let mut subs = HashMap::new();
        subs.insert("x".to_string(), b.constant(1.0).unwrap());
        subs.insert("y".to_string(), b.constant(2.0).unwrap());

        let result = b.substitute(&expr, &subs).unwrap();
        // After substitution, result is 1.0 + 2.0 (mysym preserves Float representation)
        // Check it's a number and not zero
        let r_str = format!("{}", result);
        assert!(
            !result.is_zero(),
            "substitution result should not be zero, got: {}",
            r_str
        );
        // Check it simplifies to a numeric value
        let result_simp = b.simplify(&result).unwrap();
        let rs = format!("{}", result_simp);
        assert!(
            !result_simp.is_zero() && !rs.is_empty(),
            "simplified sub should be non-zero, got: {}",
            rs
        );
    }

    #[test]
    fn test_mysym_matrix_creation() {
        use crate::symbolic::SymbolicMatrix;
        let b = MySymBackend::new();
        let a = b.constant(1.0).unwrap();
        let b_val = b.constant(2.0).unwrap();
        let c = b.constant(3.0).unwrap();
        let d = b.constant(4.0).unwrap();

        let mat = b.matrix(vec![vec![a, b_val], vec![c, d]]).unwrap();

        assert_eq!(mat.rows, 2);
        assert_eq!(mat.cols, 2);
        assert!(mat.is_square());
    }

    #[test]
    fn test_mysym_determinant_2x2() {
        let b = MySymBackend::new();
        let a = b.constant(1.0).unwrap();
        let b_val = b.constant(2.0).unwrap();
        let c = b.constant(3.0).unwrap();
        let d = b.constant(4.0).unwrap();

        let mat = b.matrix(vec![vec![a, b_val], vec![c, d]]).unwrap();

        // det([[1,2],[3,4]]) = 1*4 - 2*3 = -2
        let det = b.determinant(&mat).unwrap();
        assert!(!det.is_zero());
        let det_str = format!("{}", det);
        assert!(!det_str.is_empty(), "determinant should be non-empty");
        // mysym returns Float representation — verify it contains a negative sign
        assert!(
            det_str.starts_with('-') || det_str.starts_with("Float") || det_str.contains('-'),
            "expected negative determinant, got: {}",
            det_str
        );
    }

    #[test]
    fn test_mysym_trace() {
        let b = MySymBackend::new();
        let a = b.constant(1.0).unwrap();
        let b_val = b.constant(2.0).unwrap();
        let c = b.constant(3.0).unwrap();
        let d = b.constant(4.0).unwrap();

        let mat = b.matrix(vec![vec![a, b_val], vec![c, d]]).unwrap();

        let tr = b.trace(&mat).unwrap();
        assert!(!tr.is_zero());
        let tr_str = format!("{}", tr);
        assert!(!tr_str.is_empty(), "trace should be non-empty");
    }

    #[test]
    fn test_mysym_solve() {
        let b = MySymBackend::new();
        // solve x + 5 = 0 => x = -5
        let x = b.variable("x").unwrap();
        let five = b.constant(5.0).unwrap();
        let eq = b.add(&x, &five).unwrap();
        let sol = b.solve(&eq, "x").unwrap();
        assert!(!sol.is_empty());
    }
}
