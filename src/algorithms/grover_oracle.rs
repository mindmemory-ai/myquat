// Grover Oracle Construction Module
// Author: gA4ss
//
// Provides flexible and general-purpose Oracle construction for Grover's algorithm

use crate::error::{MyQuatError, Result};
use crate::QuantumCircuit;

/// Oracle type for Grover's algorithm
pub enum OracleType {
    /// Mark specific items by index
    Exact(Vec<usize>),
    /// Mark items satisfying a boolean function
    Function(Box<dyn Fn(usize) -> bool + Send + Sync>),
    /// Mark items matching a pattern (binary string with wildcards)
    Pattern(String),
    /// Mark items in a range
    Range { start: usize, end: usize },
    /// Mark items satisfying SAT formula
    SAT(SATFormula),
}

/// SAT formula for Oracle construction
#[derive(Debug, Clone)]
pub struct SATFormula {
    /// Clauses in CNF form (each clause is a disjunction of literals)
    /// Literal: (variable_index, is_positive)
    pub clauses: Vec<Vec<(usize, bool)>>,
    pub num_variables: usize,
}

impl SATFormula {
    /// Create a new SAT formula
    pub fn new(num_variables: usize) -> Self {
        SATFormula {
            clauses: Vec::new(),
            num_variables,
        }
    }

    /// Add a clause to the formula
    /// Example: add_clause(&[(0, true), (1, false)]) adds (x0 OR NOT x1)
    pub fn add_clause(&mut self, literals: Vec<(usize, bool)>) {
        self.clauses.push(literals);
    }

    /// Evaluate the formula for a given assignment
    pub fn evaluate(&self, assignment: usize) -> bool {
        for clause in &self.clauses {
            let mut clause_satisfied = false;
            for &(var_idx, is_positive) in clause {
                let bit_value = (assignment >> var_idx) & 1 == 1;
                if bit_value == is_positive {
                    clause_satisfied = true;
                    break;
                }
            }
            if !clause_satisfied {
                return false;
            }
        }
        true
    }
}

/// General-purpose Oracle builder for Grover's algorithm
pub struct GroverOracle {
    num_qubits: usize,
    oracle_type: OracleType,
}

impl GroverOracle {
    /// Create a new Oracle for exact item marking
    pub fn exact(num_qubits: usize, marked_items: Vec<usize>) -> Result<Self> {
        let max_items = 1 << num_qubits;
        for &item in &marked_items {
            if item >= max_items {
                return Err(MyQuatError::circuit_error(format!(
                    "Marked item {} out of range [0, {})",
                    item, max_items
                )));
            }
        }

        Ok(GroverOracle {
            num_qubits,
            oracle_type: OracleType::Exact(marked_items),
        })
    }

    /// Create an Oracle using a boolean function
    /// The function takes an item index and returns true if it should be marked
    pub fn from_function<F>(num_qubits: usize, f: F) -> Self
    where
        F: Fn(usize) -> bool + Send + Sync + 'static,
    {
        GroverOracle {
            num_qubits,
            oracle_type: OracleType::Function(Box::new(f)),
        }
    }

    /// Create an Oracle from a binary pattern with wildcards
    /// Example: "10*1" marks items matching 10X1 where X can be 0 or 1
    pub fn from_pattern(pattern: &str) -> Result<Self> {
        let num_qubits = pattern.len();
        if num_qubits == 0 {
            return Err(MyQuatError::circuit_error("Empty pattern"));
        }

        // Validate pattern
        for c in pattern.chars() {
            if c != '0' && c != '1' && c != '*' {
                return Err(MyQuatError::circuit_error(format!(
                    "Invalid character '{}' in pattern",
                    c
                )));
            }
        }

        Ok(GroverOracle {
            num_qubits,
            oracle_type: OracleType::Pattern(pattern.to_string()),
        })
    }

    /// Create an Oracle for a range of items
    pub fn from_range(num_qubits: usize, start: usize, end: usize) -> Result<Self> {
        let max_items = 1 << num_qubits;
        if start >= max_items || end > max_items || start >= end {
            return Err(MyQuatError::circuit_error(format!(
                "Invalid range [{}, {}) for {} qubits",
                start, end, num_qubits
            )));
        }

        Ok(GroverOracle {
            num_qubits,
            oracle_type: OracleType::Range { start, end },
        })
    }

    /// Create an Oracle from a SAT formula
    pub fn from_sat(formula: SATFormula) -> Result<Self> {
        if formula.num_variables == 0 {
            return Err(MyQuatError::circuit_error("SAT formula has no variables"));
        }

        Ok(GroverOracle {
            num_qubits: formula.num_variables,
            oracle_type: OracleType::SAT(formula),
        })
    }

    /// Get the number of qubits
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Get all marked items (compute if necessary)
    pub fn marked_items(&self) -> Vec<usize> {
        let max_items = 1 << self.num_qubits;
        let mut marked = Vec::new();

        match &self.oracle_type {
            OracleType::Exact(items) => items.clone(),
            OracleType::Function(f) => {
                for i in 0..max_items {
                    if f(i) {
                        marked.push(i);
                    }
                }
                marked
            }
            OracleType::Pattern(pattern) => {
                for i in 0..max_items {
                    if self.matches_pattern(i, pattern) {
                        marked.push(i);
                    }
                }
                marked
            }
            OracleType::Range { start, end } => (*start..*end).collect(),
            OracleType::SAT(formula) => {
                for i in 0..max_items {
                    if formula.evaluate(i) {
                        marked.push(i);
                    }
                }
                marked
            }
        }
    }

    /// Check if an item matches a pattern
    /// Pattern is read from left to right as MSB to LSB
    /// Example: "1*0" matches items where MSB=1, LSB=0, middle bit is wildcard
    fn matches_pattern(&self, item: usize, pattern: &str) -> bool {
        let n = pattern.len();
        for (i, c) in pattern.chars().enumerate() {
            // Pattern index i corresponds to bit position (n-1-i) from LSB
            let bit_pos = n - 1 - i;
            let bit = (item >> bit_pos) & 1;
            match c {
                '0' => {
                    if bit != 0 {
                        return false;
                    }
                }
                '1' => {
                    if bit != 1 {
                        return false;
                    }
                }
                '*' => {} // Wildcard matches anything
                _ => return false,
            }
        }
        true
    }

    /// Apply the Oracle to a quantum circuit
    pub fn apply(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        let marked_items = self.marked_items();

        for &marked_item in &marked_items {
            // Convert item index to binary and apply X gates for 0 bits
            let mut temp_gates = Vec::new();

            for i in 0..self.num_qubits {
                if (marked_item >> i) & 1 == 0 {
                    circuit.x(i)?;
                    temp_gates.push(i);
                }
            }

            // Multi-controlled Z gate (phase flip)
            self.apply_multi_controlled_z(circuit)?;

            // Undo temporary X gates
            for &qubit in temp_gates.iter().rev() {
                circuit.x(qubit)?;
            }
        }

        Ok(())
    }

    /// Apply multi-controlled Z gate using the standard decomposition.
    ///
    /// For n qubits: H on last qubit -> multi-controlled-NOT -> H on last qubit.
    /// Multi-controlled-NOT is decomposed using n-2 Toffoli gates with
    /// borrowed ancillas (the intermediate qubits).
    fn apply_multi_controlled_z(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        let n = self.num_qubits;
        if n == 1 {
            circuit.z(0)?;
            return Ok(());
        }
        if n == 2 {
            circuit.cz(0, 1)?;
            return Ok(());
        }

        // MCZ = H(target) · MCNOT · H(target)
        let target = n - 1;
        circuit.h(target)?;

        // Standard n-qubit MCNOT using (n-2) Toffoli gates and borrowed ancillas.
        if n == 3 {
            circuit.ccx(0, 1, target)?;
        } else {
            // Forward chain of Toffoli gates
            circuit.ccx(0, 1, 2)?;
            for i in 2..n.saturating_sub(2) {
                circuit.ccx(i - 1, i, i + 1)?;
            }
            circuit.ccx(n - 3, n - 2, target)?;
            // Uncompute in reverse
            for i in (2..n.saturating_sub(2)).rev() {
                circuit.ccx(i - 1, i, i + 1)?;
            }
            circuit.ccx(0, 1, 2)?;
        }

        circuit.h(target)?;
        Ok(())
    }

    /// Count the number of marked items
    pub fn count_marked(&self) -> usize {
        self.marked_items().len()
    }

    /// Calculate optimal number of Grover iterations
    pub fn optimal_iterations(&self) -> usize {
        let n = 1 << self.num_qubits;
        let m = self.count_marked();
        if m == 0 {
            return 0;
        }

        let theta = (m as f64 / n as f64).sqrt().asin();
        (((std::f64::consts::PI / 4.0) / theta - 0.5).round() as usize).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_oracle() {
        let oracle = GroverOracle::exact(3, vec![2, 5]).unwrap();
        assert_eq!(oracle.num_qubits(), 3);
        assert_eq!(oracle.marked_items(), vec![2, 5]);
    }

    #[test]
    fn test_function_oracle() {
        // Mark even numbers
        let oracle = GroverOracle::from_function(3, |x| x % 2 == 0);
        let marked = oracle.marked_items();
        assert_eq!(marked, vec![0, 2, 4, 6]);
    }

    #[test]
    fn test_pattern_oracle() {
        let oracle = GroverOracle::from_pattern("1*0").unwrap();
        let marked = oracle.marked_items();
        // Pattern 1*0 (MSB to LSB) matches: 100 (4) and 110 (6)
        assert_eq!(marked, vec![4, 6]);
    }

    #[test]
    fn test_range_oracle() {
        let oracle = GroverOracle::from_range(3, 2, 6).unwrap();
        let marked = oracle.marked_items();
        assert_eq!(marked, vec![2, 3, 4, 5]);
    }

    #[test]
    fn test_sat_oracle() {
        let mut formula = SATFormula::new(3);
        // (x0 OR x1) AND (NOT x2)
        formula.add_clause(vec![(0, true), (1, true)]);
        formula.add_clause(vec![(2, false)]);

        let oracle = GroverOracle::from_sat(formula).unwrap();
        let marked = oracle.marked_items();

        // Should mark items where (x0 OR x1) AND NOT x2
        // Binary: 001, 010, 011 -> decimal: 1, 2, 3
        assert!(marked.contains(&1));
        assert!(marked.contains(&2));
        assert!(marked.contains(&3));
    }

    #[test]
    fn test_optimal_iterations() {
        let oracle = GroverOracle::exact(4, vec![5]).unwrap();
        let iterations = oracle.optimal_iterations();
        assert!(iterations > 0);
        assert!(iterations < 10);
    }
}
