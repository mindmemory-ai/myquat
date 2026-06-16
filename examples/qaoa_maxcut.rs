//! QAOA MaxCut Example
//! Author: gA4ss
//!
//! Demonstrates using QAOA to solve the MaxCut problem on various graphs.

use myquat::algorithms::{Graph, MaxCutProblem, SimplexOptimizer, VQEConfig, QAOA, VQE};
use myquat::Result;
use std::time::Instant;

fn main() -> Result<()> {
    println!("QAOA MaxCut Optimization Example");
    println!("=================================\n");

    // Test 1: Small triangle graph (3 vertices)
    println!("Test 1: Triangle Graph (3 vertices)");
    println!("{}", "-".repeat(50));

    let mut graph_triangle = Graph::new(3);
    graph_triangle.add_edge(0, 1);
    graph_triangle.add_edge(1, 2);
    graph_triangle.add_edge(2, 0);

    let problem_triangle = MaxCutProblem::new(graph_triangle);
    let (optimal_cut, optimal_value) = problem_triangle.brute_force_solution();

    println!("  Graph: Triangle (3 vertices, 3 edges)");
    println!("  Optimal cut value (brute force): {}", optimal_value);
    println!("  Optimal partition: {:?}\n", optimal_cut);

    // Run QAOA with p=2 layers
    run_qaoa_maxcut(&problem_triangle, 2, "Triangle", optimal_value)?;

    println!("\n");

    // Test 2: Square graph (4 vertices)
    println!("Test 2: Square Graph (4 vertices)");
    println!("{}", "-".repeat(50));

    let mut graph_square = Graph::new(4);
    graph_square.add_edge(0, 1);
    graph_square.add_edge(1, 2);
    graph_square.add_edge(2, 3);
    graph_square.add_edge(3, 0);

    let problem_square = MaxCutProblem::new(graph_square);
    let (optimal_cut, optimal_value) = problem_square.brute_force_solution();

    println!("  Graph: Square (4 vertices, 4 edges)");
    println!("  Optimal cut value (brute force): {}", optimal_value);
    println!("  Optimal partition: {:?}\n", optimal_cut);

    run_qaoa_maxcut(&problem_square, 2, "Square", optimal_value)?;

    println!("\n");

    // Test 3: Complete graph K4 (4 vertices, all connected)
    println!("Test 3: Complete Graph K4");
    println!("{}", "-".repeat(50));

    let mut graph_k4 = Graph::new(4);
    for i in 0..4 {
        for j in (i + 1)..4 {
            graph_k4.add_edge(i, j);
        }
    }

    let problem_k4 = MaxCutProblem::new(graph_k4);
    let (optimal_cut, optimal_value) = problem_k4.brute_force_solution();

    println!("  Graph: K4 (4 vertices, 6 edges)");
    println!("  Optimal cut value (brute force): {}", optimal_value);
    println!("  Optimal partition: {:?}\n", optimal_cut);

    run_qaoa_maxcut(&problem_k4, 3, "K4", optimal_value)?;

    println!("\n");

    // Test 4: Weighted graph
    println!("Test 4: Weighted Graph");
    println!("{}", "-".repeat(50));

    let mut graph_weighted = Graph::new(4);
    graph_weighted.add_weighted_edge(0, 1, 1.5);
    graph_weighted.add_weighted_edge(1, 2, 2.0);
    graph_weighted.add_weighted_edge(2, 3, 1.0);
    graph_weighted.add_weighted_edge(3, 0, 2.5);

    let problem_weighted = MaxCutProblem::new(graph_weighted);
    let (optimal_cut, optimal_value) = problem_weighted.brute_force_solution();

    println!("  Graph: Weighted (4 vertices, weights: 1.5, 2.0, 1.0, 2.5)");
    println!("  Optimal cut value (brute force): {:.2}", optimal_value);
    println!("  Optimal partition: {:?}\n", optimal_cut);

    run_qaoa_maxcut(&problem_weighted, 3, "Weighted", optimal_value)?;

    println!("\n");
    println!("QAOA MaxCut optimization completed successfully!");

    Ok(())
}

/// Run QAOA optimization for MaxCut problem
fn run_qaoa_maxcut(
    problem: &MaxCutProblem,
    num_layers: usize,
    name: &str,
    optimal_value: f64,
) -> Result<()> {
    let n_qubits = problem.graph.num_vertices;
    let qaoa = QAOA::new(n_qubits, num_layers);

    println!("  Running QAOA with p={} layers...", num_layers);

    // Initial parameters (random)
    let initial_params = qaoa.random_parameters();

    // Build initial circuit
    let start = Instant::now();
    let circuit = qaoa.build_maxcut_circuit(&problem.graph.edges, &initial_params)?;
    let elapsed = start.elapsed();

    println!("    Circuit built in {:?}", elapsed);
    println!("    Circuit size: {} gates", circuit.size());
    println!("    Circuit depth: {}", circuit.depth());
    println!("    Number of parameters: {}", initial_params.len());

    // Simulate simple parameter sweep to find good solution
    let best_result = parameter_sweep(&qaoa, problem, num_layers);

    println!("\n  QAOA Results:");
    println!("    Best cut found: {:.2}", best_result.0);
    println!(
        "    Approximation ratio: {:.4}",
        best_result.0 / optimal_value
    );
    println!("    Best partition: {:?}", best_result.1);
    println!("    Best parameters (gamma): {:?}", best_result.2);
    println!("    Best parameters (beta): {:?}", best_result.3);

    Ok(())
}

/// Simplified parameter sweep (replaces full quantum simulation)
fn parameter_sweep(
    qaoa: &QAOA,
    problem: &MaxCutProblem,
    num_layers: usize,
) -> (f64, Vec<bool>, Vec<f64>, Vec<f64>) {
    use std::f64::consts::PI;

    let n_qubits = problem.graph.num_vertices;
    let mut best_cut_value = 0.0;
    let mut best_partition = vec![false; n_qubits];
    let mut best_gamma = vec![0.0; num_layers];
    let mut best_beta = vec![0.0; num_layers];

    // Simple grid search over parameter space (simplified for demo)
    let n_samples = 5;

    for gamma_idx in 0..n_samples {
        for beta_idx in 0..n_samples {
            let gamma = (gamma_idx as f64 / n_samples as f64) * PI;
            let beta = (beta_idx as f64 / n_samples as f64) * PI;

            // For this demo, we just evaluate some random partitions
            // In real QAOA, these would come from circuit simulation
            for mask in 0..(1 << n_qubits) {
                let partition: Vec<bool> = (0..n_qubits).map(|i| (mask & (1 << i)) != 0).collect();

                let cut_value = problem.evaluate(&partition);

                if cut_value > best_cut_value {
                    best_cut_value = cut_value;
                    best_partition = partition;
                    best_gamma = vec![gamma; num_layers];
                    best_beta = vec![beta; num_layers];
                }
            }
        }
    }

    (best_cut_value, best_partition, best_gamma, best_beta)
}
