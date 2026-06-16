//! Benchmarks for MyQuat gate operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use myquat::{
    gates::Gate,
    transpiler::{PassManager, TranspilerPass},
    Parameter, QuantumCircuit,
};
use std::collections::HashMap;
use std::f64::consts::PI;

fn benchmark_gate_creation(c: &mut Criterion) {
    c.bench_function("create_pauli_x", |b| b.iter(|| Gate::x()));

    c.bench_function("create_hadamard", |b| b.iter(|| Gate::h()));

    c.bench_function("create_rotation", |b| {
        b.iter(|| Gate::rx(black_box(Parameter::new_float(PI / 4.0))))
    });

    c.bench_function("create_cnot", |b| b.iter(|| Gate::cx()));
}

fn benchmark_gate_matrices(c: &mut Criterion) {
    let symbols = HashMap::new();

    c.bench_function("pauli_x_matrix", |b| {
        let gate = Gate::x();
        b.iter(|| gate.matrix(black_box(&symbols)))
    });

    c.bench_function("hadamard_matrix", |b| {
        let gate = Gate::h();
        b.iter(|| gate.matrix(black_box(&symbols)))
    });

    c.bench_function("rotation_matrix", |b| {
        let gate = Gate::rx(Parameter::new_float(PI / 4.0));
        b.iter(|| gate.matrix(black_box(&symbols)))
    });

    c.bench_function("cnot_matrix", |b| {
        let gate = Gate::cx();
        b.iter(|| gate.matrix(black_box(&symbols)))
    });
}

fn benchmark_circuit_construction(c: &mut Criterion) {
    c.bench_function("create_empty_circuit", |b| {
        b.iter(|| QuantumCircuit::new(black_box(5), black_box(5)))
    });

    c.bench_function("build_bell_circuit", |b| {
        b.iter(|| {
            let mut circuit = QuantumCircuit::new(2, 2);
            circuit.h(0).unwrap();
            circuit.cx(0, 1).unwrap();
            circuit.measure(0, 0).unwrap();
            circuit.measure(1, 1).unwrap();
            circuit
        })
    });

    c.bench_function("build_large_circuit", |b| {
        b.iter(|| {
            let mut circuit = QuantumCircuit::new(10, 0);
            for i in 0..10 {
                circuit.h(i).unwrap();
            }
            for i in 0..9 {
                circuit.cx(i, i + 1).unwrap();
            }
            circuit
        })
    });
}

fn benchmark_circuit_analysis(c: &mut Criterion) {
    let mut circuit = QuantumCircuit::new(10, 0);
    for i in 0..10 {
        circuit.h(i).unwrap();
        circuit.rz(i, Parameter::new_float(PI / 4.0)).unwrap();
    }
    for i in 0..9 {
        circuit.cx(i, i + 1).unwrap();
    }

    c.bench_function("circuit_depth", |b| b.iter(|| black_box(&circuit).depth()));

    c.bench_function("circuit_size", |b| b.iter(|| black_box(&circuit).size()));

    c.bench_function("gate_counts", |b| {
        b.iter(|| myquat::transpiler::CircuitAnalysis::gate_counts(black_box(&circuit)))
    });
}

fn benchmark_transpilation(c: &mut Criterion) {
    let mut unoptimized_circuit = QuantumCircuit::new(5, 0);
    // Add many redundant gates
    for _ in 0..10 {
        for i in 0..5 {
            unoptimized_circuit.i(i).unwrap();
            unoptimized_circuit.s(i).unwrap();
            unoptimized_circuit.sdg(i).unwrap();
            unoptimized_circuit.x(i).unwrap();
            unoptimized_circuit.x(i).unwrap();
        }
    }

    c.bench_function("remove_identity_gates", |b| {
        b.iter(|| {
            let mut circuit = unoptimized_circuit.clone();
            let pass = myquat::transpiler::RemoveIdentityGates;
            pass.run(&mut circuit).unwrap();
            circuit
        })
    });

    c.bench_function("cancel_inverses", |b| {
        b.iter(|| {
            let mut circuit = unoptimized_circuit.clone();
            let pass = myquat::transpiler::CancelInverses;
            pass.run(&mut circuit).unwrap();
            circuit
        })
    });

    c.bench_function("full_optimization", |b| {
        b.iter(|| {
            let mut circuit = unoptimized_circuit.clone();
            let pass_manager = PassManager::default_optimization();
            pass_manager.run(&mut circuit).unwrap();
            circuit
        })
    });
}

fn benchmark_parameter_operations(c: &mut Criterion) {
    let param = Parameter::new_symbol("theta");
    let mut symbols = HashMap::new();
    symbols.insert("theta".to_string(), PI / 4.0);

    c.bench_function("parameter_evaluation", |b| {
        b.iter(|| param.evaluate(black_box(&symbols)))
    });

    let complex_param = Parameter::new_expression(myquat::parameter::ParameterExpression::Add(
        Box::new(myquat::parameter::ParameterExpression::Parameter(
            Parameter::new_symbol("a"),
        )),
        Box::new(myquat::parameter::ParameterExpression::Mul(
            Box::new(myquat::parameter::ParameterExpression::Parameter(
                Parameter::new_float(2.0),
            )),
            Box::new(myquat::parameter::ParameterExpression::Parameter(
                Parameter::new_symbol("b"),
            )),
        )),
    ));

    symbols.insert("a".to_string(), 1.0);
    symbols.insert("b".to_string(), 2.0);

    c.bench_function("complex_parameter_evaluation", |b| {
        b.iter(|| complex_param.evaluate(black_box(&symbols)))
    });

    c.bench_function("parameter_binding", |b| {
        let mut circuit = QuantumCircuit::new(3, 0);
        circuit.rx(0, Parameter::new_symbol("theta")).unwrap();
        circuit.ry(1, Parameter::new_symbol("phi")).unwrap();
        circuit.rz(2, Parameter::new_symbol("lambda")).unwrap();

        let mut bind_symbols = HashMap::new();
        bind_symbols.insert("theta".to_string(), PI / 3.0);
        bind_symbols.insert("phi".to_string(), PI / 4.0);
        bind_symbols.insert("lambda".to_string(), PI / 6.0);

        b.iter(|| circuit.bind_parameters(black_box(&bind_symbols)))
    });
}

fn benchmark_unitary_computation(c: &mut Criterion) {
    let symbols = HashMap::new();

    c.bench_function("single_qubit_unitary", |b| {
        let mut circuit = QuantumCircuit::new(1, 0);
        circuit.h(0).unwrap();
        circuit.s(0).unwrap();
        circuit.t(0).unwrap();

        b.iter(|| circuit.unitary(black_box(&symbols)))
    });

    c.bench_function("two_qubit_unitary", |b| {
        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.h(1).unwrap();

        b.iter(|| circuit.unitary(black_box(&symbols)))
    });

    c.bench_function("three_qubit_unitary", |b| {
        let mut circuit = QuantumCircuit::new(3, 0);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.cx(1, 2).unwrap();

        b.iter(|| circuit.unitary(black_box(&symbols)))
    });
}

criterion_group!(
    benches,
    benchmark_gate_creation,
    benchmark_gate_matrices,
    benchmark_circuit_construction,
    benchmark_circuit_analysis,
    benchmark_transpilation,
    benchmark_parameter_operations,
    benchmark_unitary_computation
);

criterion_main!(benches);
