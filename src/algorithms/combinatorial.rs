//! Combinatorial Optimization Problems
//! Author: gA4ss
//!
//! Defines common combinatorial optimization problems for QAOA.

use std::collections::HashMap;

/// Graph representation for optimization problems
#[derive(Debug, Clone)]
pub struct Graph {
    pub num_vertices: usize,
    pub edges: Vec<(usize, usize)>,
    pub edge_weights: HashMap<(usize, usize), f64>,
}

impl Graph {
    /// Create a new graph
    pub fn new(num_vertices: usize) -> Self {
        Self {
            num_vertices,
            edges: Vec::new(),
            edge_weights: HashMap::new(),
        }
    }

    /// Add an unweighted edge
    pub fn add_edge(&mut self, u: usize, v: usize) {
        self.edges.push((u, v));
        self.edge_weights.insert((u, v), 1.0);
        self.edge_weights.insert((v, u), 1.0);
    }

    /// Add a weighted edge
    pub fn add_weighted_edge(&mut self, u: usize, v: usize, weight: f64) {
        self.edges.push((u, v));
        self.edge_weights.insert((u, v), weight);
        self.edge_weights.insert((v, u), weight);
    }

    /// Get edge weight (returns 0.0 if edge doesn't exist)
    pub fn edge_weight(&self, u: usize, v: usize) -> f64 {
        *self.edge_weights.get(&(u, v)).unwrap_or(&0.0)
    }
}

/// MaxCut problem definition
#[derive(Debug, Clone)]
pub struct MaxCutProblem {
    pub graph: Graph,
}

impl MaxCutProblem {
    /// Create a new MaxCut problem
    pub fn new(graph: Graph) -> Self {
        Self { graph }
    }

    /// Evaluate cut value for a bitstring
    pub fn evaluate(&self, bitstring: &[bool]) -> f64 {
        let mut cut_value = 0.0;

        for &(u, v) in &self.graph.edges {
            if u < bitstring.len() && v < bitstring.len() && bitstring[u] != bitstring[v] {
                cut_value += self.graph.edge_weight(u, v);
            }
        }

        cut_value
    }

    /// Find optimal cut by brute force (for small graphs)
    pub fn brute_force_solution(&self) -> (Vec<bool>, f64) {
        let n = self.graph.num_vertices;
        if n > 20 {
            panic!("Graph too large for brute force");
        }

        let mut best_cut = vec![false; n];
        let mut best_value = 0.0;

        for mask in 0..(1 << n) {
            let bitstring: Vec<bool> = (0..n).map(|i| (mask & (1 << i)) != 0).collect();
            let value = self.evaluate(&bitstring);

            if value > best_value {
                best_value = value;
                best_cut = bitstring;
            }
        }

        (best_cut, best_value)
    }
}

/// Number Partitioning problem
#[derive(Debug, Clone)]
pub struct NumberPartitionProblem {
    pub numbers: Vec<f64>,
}

impl NumberPartitionProblem {
    /// Create a new Number Partitioning problem
    pub fn new(numbers: Vec<f64>) -> Self {
        Self { numbers }
    }

    /// Evaluate partition quality (minimize difference)
    pub fn evaluate(&self, partition: &[bool]) -> f64 {
        let sum_a: f64 = self
            .numbers
            .iter()
            .enumerate()
            .filter(|(i, _)| partition[*i])
            .map(|(_, v)| v)
            .sum();

        let sum_b: f64 = self
            .numbers
            .iter()
            .enumerate()
            .filter(|(i, _)| !partition[*i])
            .map(|(_, v)| v)
            .sum();

        (sum_a - sum_b).abs()
    }
}

/// Portfolio Optimization problem
#[derive(Debug, Clone)]
pub struct PortfolioProblem {
    /// Expected returns
    pub returns: Vec<f64>,
    /// Risk (covariance matrix flattened)
    pub risks: Vec<Vec<f64>>,
    /// Budget constraint (number of assets to select)
    pub budget: usize,
    /// Risk aversion parameter
    pub lambda: f64,
}

impl PortfolioProblem {
    /// Create a new Portfolio problem
    pub fn new(returns: Vec<f64>, risks: Vec<Vec<f64>>, budget: usize, lambda: f64) -> Self {
        Self {
            returns,
            risks,
            budget,
            lambda,
        }
    }

    /// Evaluate portfolio quality (maximize return, minimize risk)
    pub fn evaluate(&self, selection: &[bool]) -> f64 {
        let selected_count = selection.iter().filter(|&&x| x).count();

        // Penalty for violating budget constraint
        if selected_count != self.budget {
            return -1000.0 * (selected_count as f64 - self.budget as f64).abs();
        }

        // Calculate expected return
        let total_return: f64 = self
            .returns
            .iter()
            .enumerate()
            .filter(|(i, _)| selection[*i])
            .map(|(_, r)| r)
            .sum();

        // Calculate risk
        let mut total_risk = 0.0;
        for i in 0..selection.len() {
            if !selection[i] {
                continue;
            }
            for j in 0..selection.len() {
                if !selection[j] {
                    continue;
                }
                total_risk += self.risks[i][j];
            }
        }

        // Objective: maximize return - lambda * risk
        total_return - self.lambda * total_risk
    }
}

/// SAT clause
#[derive(Debug, Clone)]
pub struct SATClause {
    /// Variable indices and their signs (true = positive, false = negative)
    pub literals: Vec<(usize, bool)>,
}

impl SATClause {
    /// Create a new SAT clause
    pub fn new(literals: Vec<(usize, bool)>) -> Self {
        Self { literals }
    }

    /// Check if clause is satisfied
    pub fn is_satisfied(&self, assignment: &[bool]) -> bool {
        for &(var, positive) in &self.literals {
            if var < assignment.len() {
                let value = if positive {
                    assignment[var]
                } else {
                    !assignment[var]
                };
                if value {
                    return true;
                }
            }
        }
        false
    }
}

/// MaxSAT problem
#[derive(Debug, Clone)]
pub struct MaxSATProblem {
    pub num_vars: usize,
    pub clauses: Vec<SATClause>,
}

impl MaxSATProblem {
    /// Create a new MaxSAT problem
    pub fn new(num_vars: usize) -> Self {
        Self {
            num_vars,
            clauses: Vec::new(),
        }
    }

    /// Add a clause
    pub fn add_clause(&mut self, clause: SATClause) {
        self.clauses.push(clause);
    }

    /// Evaluate number of satisfied clauses
    pub fn evaluate(&self, assignment: &[bool]) -> usize {
        self.clauses
            .iter()
            .filter(|c| c.is_satisfied(assignment))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let mut graph = Graph::new(4);
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        graph.add_weighted_edge(2, 3, 2.5);

        assert_eq!(graph.num_vertices, 4);
        assert_eq!(graph.edges.len(), 3);
        assert_eq!(graph.edge_weight(2, 3), 2.5);
        assert_eq!(graph.edge_weight(0, 1), 1.0);
    }

    #[test]
    fn test_maxcut_evaluation() {
        let mut graph = Graph::new(4);
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(3, 0);

        let problem = MaxCutProblem::new(graph);
        let bitstring = vec![false, true, true, false];
        let cut_value = problem.evaluate(&bitstring);

        // Bitstring [F,T,T,F] cuts edges (0,1) and (2,3) = 2 edges
        assert_eq!(cut_value, 2.0);
    }

    #[test]
    fn test_maxcut_brute_force() {
        let mut graph = Graph::new(3);
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);

        let problem = MaxCutProblem::new(graph);
        let (solution, value) = problem.brute_force_solution();

        assert_eq!(value, 2.0);
        assert_eq!(solution.len(), 3);
    }

    #[test]
    fn test_number_partition() {
        let problem = NumberPartitionProblem::new(vec![1.0, 2.0, 3.0, 4.0]);
        let partition = vec![true, true, false, false]; // {1,2} vs {3,4}
        let diff = problem.evaluate(&partition);

        assert_eq!(diff, 4.0); // |3 - 7| = 4
    }

    #[test]
    fn test_sat_clause() {
        let clause = SATClause::new(vec![(0, true), (1, false)]); // x0 OR !x1

        assert!(clause.is_satisfied(&vec![true, false]));
        assert!(clause.is_satisfied(&vec![false, false]));
        assert!(!clause.is_satisfied(&vec![false, true]));
    }

    #[test]
    fn test_maxsat_evaluation() {
        let mut problem = MaxSATProblem::new(3);
        problem.add_clause(SATClause::new(vec![(0, true), (1, false)]));
        problem.add_clause(SATClause::new(vec![(1, true), (2, true)]));

        let assignment = vec![false, true, false];
        let satisfied = problem.evaluate(&assignment);

        assert_eq!(satisfied, 1);
    }
}
