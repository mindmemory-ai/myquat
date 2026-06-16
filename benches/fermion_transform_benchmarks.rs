// Fermion Transformation Benchmarks
// Author: gA4ss
//
// Performance comparison of different fermion-to-qubit mapping methods

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use myquat::hamiltonian::{
    bravyi_kitaev_transform, jordan_wigner_transform, parity_transform,
    ElectronicStructureHamiltonian, FermionOperator, FermionTerm, MappingMethod,
};
use num_complex::Complex64;

fn bench_single_operator_transforms(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_operator_transforms");

    for num_qubits in [4, 8, 12, 16].iter() {
        let term = FermionTerm::new(
            vec![(num_qubits / 2, FermionOperator::Creation)],
            Complex64::new(1.0, 0.0),
        );

        group.bench_with_input(
            BenchmarkId::new("jordan_wigner", num_qubits),
            num_qubits,
            |b, &n| {
                b.iter(|| jordan_wigner_transform(black_box(&term), black_box(n)).unwrap());
            },
        );

        group.bench_with_input(
            BenchmarkId::new("bravyi_kitaev", num_qubits),
            num_qubits,
            |b, &n| {
                b.iter(|| bravyi_kitaev_transform(black_box(&term), black_box(n)).unwrap());
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parity", num_qubits),
            num_qubits,
            |b, &n| {
                b.iter(|| parity_transform(black_box(&term), black_box(n)).unwrap());
            },
        );
    }

    group.finish();
}

fn bench_number_operator_transforms(c: &mut Criterion) {
    let mut group = c.benchmark_group("number_operator_transforms");

    for num_qubits in [4, 8, 12].iter() {
        let term = FermionTerm::number_operator(0, Complex64::new(1.0, 0.0));

        group.bench_with_input(
            BenchmarkId::new("jordan_wigner", num_qubits),
            num_qubits,
            |b, &n| {
                b.iter(|| jordan_wigner_transform(black_box(&term), black_box(n)).unwrap());
            },
        );

        group.bench_with_input(
            BenchmarkId::new("bravyi_kitaev", num_qubits),
            num_qubits,
            |b, &n| {
                b.iter(|| bravyi_kitaev_transform(black_box(&term), black_box(n)).unwrap());
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parity", num_qubits),
            num_qubits,
            |b, &n| {
                b.iter(|| parity_transform(black_box(&term), black_box(n)).unwrap());
            },
        );
    }

    group.finish();
}

fn bench_hamiltonian_transforms(c: &mut Criterion) {
    let mut group = c.benchmark_group("hamiltonian_transforms");

    for num_orbitals in [2, 4, 6].iter() {
        let mut es = ElectronicStructureHamiltonian::new(*num_orbitals);

        // Add some one-body terms
        for i in 0..*num_orbitals {
            es.set_one_body(i, i, -1.0 - i as f64 * 0.1);
        }

        // Add some two-body terms
        for i in 0..*num_orbitals {
            for j in 0..*num_orbitals {
                if i != j {
                    es.set_two_body(i, i, j, j, 0.1);
                }
            }
        }

        es.nuclear_repulsion = 0.5;

        group.bench_with_input(
            BenchmarkId::new("jordan_wigner", num_orbitals),
            num_orbitals,
            |b, _| {
                b.iter(|| {
                    es.to_qubit_hamiltonian(black_box(MappingMethod::JordanWigner))
                        .unwrap()
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("bravyi_kitaev", num_orbitals),
            num_orbitals,
            |b, _| {
                b.iter(|| {
                    es.to_qubit_hamiltonian(black_box(MappingMethod::BravyiKitaev))
                        .unwrap()
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parity", num_orbitals),
            num_orbitals,
            |b, _| {
                b.iter(|| {
                    es.to_qubit_hamiltonian(black_box(MappingMethod::Parity))
                        .unwrap()
                });
            },
        );
    }

    group.finish();
}

fn bench_pauli_weight(c: &mut Criterion) {
    let mut group = c.benchmark_group("pauli_weight_analysis");

    // Measure the average Pauli weight for different transformations
    for num_qubits in [8, 12, 16].iter() {
        let term = FermionTerm::hopping_operator(0, num_qubits - 1, Complex64::new(1.0, 0.0));

        group.bench_with_input(
            BenchmarkId::new("jordan_wigner_weight", num_qubits),
            num_qubits,
            |b, &n| {
                b.iter(|| {
                    let pauli_terms =
                        jordan_wigner_transform(black_box(&term), black_box(n)).unwrap();
                    // Count non-identity operators
                    let _weight: usize = pauli_terms
                        .iter()
                        .map(|(ps, _)| {
                            ps.operators()
                                .iter()
                                .filter(|&op| *op != myquat::hamiltonian::PauliOperator::I)
                                .count()
                        })
                        .sum();
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("bravyi_kitaev_weight", num_qubits),
            num_qubits,
            |b, &n| {
                b.iter(|| {
                    let pauli_terms =
                        bravyi_kitaev_transform(black_box(&term), black_box(n)).unwrap();
                    let _weight: usize = pauli_terms
                        .iter()
                        .map(|(ps, _)| {
                            ps.operators()
                                .iter()
                                .filter(|&op| *op != myquat::hamiltonian::PauliOperator::I)
                                .count()
                        })
                        .sum();
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parity_weight", num_qubits),
            num_qubits,
            |b, &n| {
                b.iter(|| {
                    let pauli_terms = parity_transform(black_box(&term), black_box(n)).unwrap();
                    let _weight: usize = pauli_terms
                        .iter()
                        .map(|(ps, _)| {
                            ps.operators()
                                .iter()
                                .filter(|&op| *op != myquat::hamiltonian::PauliOperator::I)
                                .count()
                        })
                        .sum();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_single_operator_transforms,
    bench_number_operator_transforms,
    bench_hamiltonian_transforms,
    bench_pauli_weight
);
criterion_main!(benches);
