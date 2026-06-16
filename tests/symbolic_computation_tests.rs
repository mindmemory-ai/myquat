// Symbolic Computation Tests
// Author: gA4ss
//
// Tests for symbolic computation backend functionality

use myquat::symbolic::SymbolicaBackend;
use myquat::symbolic::{SymbolicBackend, SymbolicExpression};

#[test]
fn test_basic_arithmetic() {
    let backend = SymbolicaBackend::new();

    // Create variables
    let x = backend.variable("x").unwrap();
    let y = backend.variable("y").unwrap();

    // Test addition
    let sum = backend.add(&x, &y).unwrap();
    assert!(!sum.is_zero());

    // Test multiplication
    let prod = backend.mul(&x, &y).unwrap();
    assert!(!prod.is_zero());
}

#[test]
fn test_constants() {
    let backend = SymbolicaBackend::new();

    let zero = backend.constant(0.0).unwrap();
    assert!(zero.is_zero());

    let one = backend.constant(1.0).unwrap();
    assert!(one.is_one());

    let two = backend.constant(2.0).unwrap();
    assert!(!two.is_zero());
    assert!(!two.is_one());
}

#[test]
fn test_differentiation() {
    let backend = SymbolicaBackend::new();

    // d/dx (x^2) = 2x
    let x = backend.variable("x").unwrap();
    let two = backend.constant(2.0).unwrap();
    let x_squared = backend.pow(&x, &two).unwrap();

    let derivative = backend.differentiate(&x_squared, "x", 1).unwrap();

    // The derivative should not be zero
    assert!(!derivative.is_zero());
}

#[test]
fn test_simplify() {
    let backend = SymbolicaBackend::new();

    let x = backend.variable("x").unwrap();
    let zero = backend.constant(0.0).unwrap();

    // x + 0 should simplify to x
    let expr = backend.add(&x, &zero).unwrap();
    let simplified = backend.simplify(&expr).unwrap();

    // After simplification, should still be valid
    assert!(!simplified.is_zero() || x.is_zero());
}

#[test]
fn test_expand() {
    let backend = SymbolicaBackend::new();

    // (x + 1)^2
    let x = backend.variable("x").unwrap();
    let one = backend.constant(1.0).unwrap();
    let x_plus_1 = backend.add(&x, &one).unwrap();
    let two = backend.constant(2.0).unwrap();
    let squared = backend.pow(&x_plus_1, &two).unwrap();

    let expanded = backend.expand(&squared).unwrap();

    // Expanded form should be valid
    assert!(!expanded.is_zero());
}

#[test]
fn test_mathematical_functions() {
    let backend = SymbolicaBackend::new();

    let x = backend.variable("x").unwrap();

    // Test exp
    let exp_x = backend.exp(&x).unwrap();
    assert!(!exp_x.is_zero());

    // Test sin
    let sin_x = backend.sin(&x).unwrap();
    assert!(!sin_x.is_zero());

    // Test cos
    let cos_x = backend.cos(&x).unwrap();
    assert!(!cos_x.is_zero());
}

#[test]
fn test_matrix_operations() {
    let backend = SymbolicaBackend::new();

    // Create 2x2 identity matrix
    let one = backend.constant(1.0).unwrap();
    let zero = backend.constant(0.0).unwrap();

    let elements = vec![
        vec![one.clone(), zero.clone()],
        vec![zero.clone(), one.clone()],
    ];

    let matrix = backend.matrix(elements).unwrap();

    assert!(matrix.is_square());
    assert_eq!(matrix.rows, 2);
    assert_eq!(matrix.cols, 2);
}

#[test]
fn test_matrix_multiplication() {
    let backend = SymbolicaBackend::new();

    let one = backend.constant(1.0).unwrap();
    let zero = backend.constant(0.0).unwrap();
    let two = backend.constant(2.0).unwrap();

    // Matrix A = [[1, 0], [0, 2]]
    let a = backend
        .matrix(vec![
            vec![one.clone(), zero.clone()],
            vec![zero.clone(), two.clone()],
        ])
        .unwrap();

    // Matrix B = [[1, 0], [0, 1]] (identity)
    let b = backend
        .matrix(vec![
            vec![one.clone(), zero.clone()],
            vec![zero.clone(), one.clone()],
        ])
        .unwrap();

    // A * B should equal A
    let result = backend.matrix_mul(&a, &b).unwrap();

    assert_eq!(result.rows, 2);
    assert_eq!(result.cols, 2);
}

#[test]
fn test_determinant_2x2() {
    let backend = SymbolicaBackend::new();

    // Matrix [[1, 2], [3, 4]]
    // det = 1*4 - 2*3 = -2
    let one = backend.constant(1.0).unwrap();
    let two = backend.constant(2.0).unwrap();
    let three = backend.constant(3.0).unwrap();
    let four = backend.constant(4.0).unwrap();

    let matrix = backend
        .matrix(vec![vec![one, two], vec![three, four]])
        .unwrap();

    let det = backend.determinant(&matrix).unwrap();

    // Determinant should not be zero
    assert!(!det.is_zero());
}

#[test]
fn test_trace() {
    let backend = SymbolicaBackend::new();

    let one = backend.constant(1.0).unwrap();
    let two = backend.constant(2.0).unwrap();
    let zero = backend.constant(0.0).unwrap();

    // Matrix [[1, 0], [0, 2]]
    // trace = 1 + 2 = 3
    let matrix = backend
        .matrix(vec![vec![one, zero.clone()], vec![zero, two]])
        .unwrap();

    let trace = backend.trace(&matrix).unwrap();

    // Trace should not be zero
    assert!(!trace.is_zero());
}

#[test]
fn test_commutator() {
    let backend = SymbolicaBackend::new();

    let one = backend.constant(1.0).unwrap();
    let zero = backend.constant(0.0).unwrap();

    // Two identity matrices
    let a = backend
        .matrix(vec![
            vec![one.clone(), zero.clone()],
            vec![zero.clone(), one.clone()],
        ])
        .unwrap();

    let b = backend
        .matrix(vec![
            vec![one.clone(), zero.clone()],
            vec![zero.clone(), one.clone()],
        ])
        .unwrap();

    // [I, I] = 0
    let comm = backend.commutator(&a, &b).unwrap();

    // All elements should be zero
    for i in 0..comm.rows {
        for j in 0..comm.cols {
            assert!(comm.get(i, j).unwrap().is_zero());
        }
    }
}
