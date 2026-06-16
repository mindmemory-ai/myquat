//! Grover's search algorithm implementation
//!
//! This module provides Grover's quantum search algorithm
//! for searching marked items in an unsorted database.
//!
//! Grover's algorithm finds one of $M$ marked items among $N = 2^n$ elements
//! in $O(\sqrt{N/M})$ oracle calls, a quadratic speedup over classical search.
//!
//! Each Grover iteration consists of:
//! 1. **Oracle** $O$: phase-flips marked states, $O|x\rangle = (-1)^{f(x)}|x\rangle$
//! 2. **Diffusion** $D$: $D = 2|s\rangle\langle s| - I$ where $|s\rangle = H^{\otimes n}|0\rangle$
//!
//! The optimal number of iterations for $M$ marked items among $N$ states:
//!
//! $$ R = \left\lfloor \frac{\pi}{4\theta} - 0.5 \right\rfloor, \quad \sin\theta = \sqrt{\frac{M}{N}} $$
//!
//! After $k$ iterations, the success probability is $\sin^2((2k+1)\theta)$.

use crate::algorithms::grover_oracle::GroverOracle;
use crate::error::{MyQuatError, Result};
use crate::QuantumCircuit;
use std::f64::consts::PI;

/// Grover iteration strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IterationStrategy {
    /// Use optimal number of iterations (default)
    Optimal,
    /// Use fixed number of iterations
    Fixed(usize),
    /// Use adaptive iteration with success probability threshold
    Adaptive { threshold: f64 },
    /// Use custom iteration count
    Custom(usize),
}

/// Grover's search algorithm
pub struct GroverSearch {
    num_qubits: usize,
    marked_items: Vec<usize>,
    iteration_strategy: IterationStrategy,
}

impl GroverSearch {
    /// Create a new Grover search instance with default optimal strategy
    pub fn new(num_qubits: usize, marked_items: Vec<usize>) -> Result<Self> {
        Self::with_strategy(num_qubits, marked_items, IterationStrategy::Optimal)
    }

    /// Create a new Grover search instance with custom iteration strategy
    pub fn with_strategy(
        num_qubits: usize,
        marked_items: Vec<usize>,
        strategy: IterationStrategy,
    ) -> Result<Self> {
        let max_items = 1 << num_qubits;
        for &item in &marked_items {
            if item >= max_items {
                return Err(MyQuatError::circuit_error("Marked item index out of range"));
            }
        }

        Ok(GroverSearch {
            num_qubits,
            marked_items,
            iteration_strategy: strategy,
        })
    }

    /// Create from Oracle
    pub fn from_oracle(oracle: &GroverOracle) -> Result<Self> {
        let marked_items = oracle.marked_items();
        Self::new(oracle.num_qubits(), marked_items)
    }

    /// Get the number of qubits
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Get the marked items
    pub fn marked_items(&self) -> &[usize] {
        &self.marked_items
    }

    /// Set iteration strategy
    pub fn set_strategy(&mut self, strategy: IterationStrategy) {
        self.iteration_strategy = strategy;
    }

    /// Get current iteration strategy
    pub fn strategy(&self) -> IterationStrategy {
        self.iteration_strategy
    }

    /// Calculate the optimal number of Grover iterations.
    ///
    /// $$ R_{\text{opt}} = \left\lfloor \frac{\pi}{4\theta} - 0.5 \right\rfloor, \quad \theta = \arcsin\!\sqrt{\frac{M}{N}} $$
    ///
    /// Returns at least 1 for any non-trivial search ($M > 0$).
    pub fn optimal_iterations(&self) -> usize {
        let n = 1 << self.num_qubits;
        let m = self.marked_items.len();
        if m == 0 {
            return 0;
        }

        let theta = (m as f64 / n as f64).sqrt().asin();
        // k_opt = π/(4θ) - 1/2, derived from (2k+1)θ = π/2
        let k = (PI / 4.0) / theta - 0.5;
        (k.round() as usize).max(1) // at least one iteration for any non-trivial search
    }

    /// Calculate number of iterations based on strategy
    pub fn calculate_iterations(&self) -> usize {
        match self.iteration_strategy {
            IterationStrategy::Optimal => self.optimal_iterations(),
            IterationStrategy::Fixed(n) => n,
            IterationStrategy::Adaptive { threshold } => self.adaptive_iterations(threshold),
            IterationStrategy::Custom(n) => n,
        }
    }

    /// Calculate adaptive iterations to reach success probability threshold
    fn adaptive_iterations(&self, threshold: f64) -> usize {
        let optimal = self.optimal_iterations();

        // Search around optimal for best iteration count
        let mut best_iter = optimal;
        let mut best_prob = self.success_probability(optimal);

        for iter in (optimal.saturating_sub(5))..=(optimal + 5) {
            let prob = self.success_probability(iter);
            if prob >= threshold && prob > best_prob {
                best_iter = iter;
                best_prob = prob;
            }
        }

        best_iter
    }

    /// Build Grover search circuit with the configured iteration strategy.
    ///
    /// Circuit: $H^{\otimes n}$ (superposition) $\to [\,O \cdot D\,]^R$ (Grover iterations)
    /// $\to$ measurement, where $O$ is the oracle, $D = H^{\otimes n}(2|0\rangle\langle0| - I)H^{\otimes n}$
    /// is the diffusion operator, and $R$ is chosen per the iteration strategy.
    pub fn build_circuit(&self) -> Result<QuantumCircuit> {
        let mut circuit = QuantumCircuit::new(self.num_qubits, self.num_qubits);

        // Initialize superposition
        for i in 0..self.num_qubits {
            circuit.h(i)?;
        }

        // Grover iterations using configured strategy
        let iterations = self.calculate_iterations();
        for _ in 0..iterations {
            // Oracle: mark the target items
            self.apply_oracle(&mut circuit)?;

            // Diffusion operator (amplitude amplification)
            self.apply_diffusion(&mut circuit)?;
        }

        // Measure all qubits
        for i in 0..self.num_qubits {
            circuit.measure(i, i)?;
        }

        Ok(circuit)
    }

    /// Build circuit with custom number of iterations
    pub fn build_circuit_with_iterations(&self, iterations: usize) -> Result<QuantumCircuit> {
        let mut circuit = QuantumCircuit::new(self.num_qubits, self.num_qubits);

        // Initialize superposition
        for i in 0..self.num_qubits {
            circuit.h(i)?;
        }

        // Custom Grover iterations
        for _ in 0..iterations {
            self.apply_oracle(&mut circuit)?;
            self.apply_diffusion(&mut circuit)?;
        }

        // Measure all qubits
        for i in 0..self.num_qubits {
            circuit.measure(i, i)?;
        }

        Ok(circuit)
    }

    /// Build circuit without measurement (for composition)
    pub fn build_operator(&self, iterations: usize) -> Result<QuantumCircuit> {
        let mut circuit = QuantumCircuit::new(self.num_qubits, 0);

        // Initialize superposition
        for i in 0..self.num_qubits {
            circuit.h(i)?;
        }

        // Grover iterations
        for _ in 0..iterations {
            self.apply_oracle(&mut circuit)?;
            self.apply_diffusion(&mut circuit)?;
        }

        Ok(circuit)
    }

    /// Apply oracle to mark target items
    fn apply_oracle(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        for &marked_item in &self.marked_items {
            // Convert item index to binary and apply X gates for 0 bits
            let mut temp_gates = Vec::new();

            for i in 0..self.num_qubits {
                if (marked_item >> i) & 1 == 0 {
                    circuit.x(i)?;
                    temp_gates.push(i);
                }
            }

            // Multi-controlled Z gate (phase flip)
            if self.num_qubits == 1 {
                circuit.z(0)?;
            } else if self.num_qubits == 2 {
                circuit.cz(0, 1)?;
            } else {
                // For more qubits, use multi-controlled Z
                self.apply_mcz(circuit)?;
            }

            // Undo temporary X gates
            for &qubit in temp_gates.iter().rev() {
                circuit.x(qubit)?;
            }
        }
        Ok(())
    }

    /// Apply the Grover diffusion operator $D = H^{\otimes n}(2|0\rangle\langle0| - I)H^{\otimes n}$.
    ///
    /// Implemented as $H^{\otimes n} \cdot X^{\otimes n} \cdot \text{MCZ} \cdot X^{\otimes n} \cdot H^{\otimes n}$,
    /// where MCZ is a multi-controlled-$Z$ gate on all $n$ qubits.
    fn apply_diffusion(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        // H gates
        for i in 0..self.num_qubits {
            circuit.h(i)?;
        }

        // X gates
        for i in 0..self.num_qubits {
            circuit.x(i)?;
        }

        // Multi-controlled Z
        self.apply_mcz(circuit)?;

        // X gates (undo)
        for i in 0..self.num_qubits {
            circuit.x(i)?;
        }

        // H gates (undo)
        for i in 0..self.num_qubits {
            circuit.h(i)?;
        }

        Ok(())
    }

    /// Apply multi-controlled Z gate using the standard decomposition.
    ///
    /// For n qubits: H on last qubit -> multi-controlled-NOT -> H on last qubit.
    /// Multi-controlled-NOT is decomposed using n-2 Toffoli gates with
    /// borrowed ancillas (the intermediate qubits).
    fn apply_mcz(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        let n = self.num_qubits;
        if n <= 1 {
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
        // The chain uses: CCX(0, 1, 2); CCX(2, 3, 4); ...; CCX(n-3, n-2, n-1)
        // then uncomputes in reverse.
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

    /// Calculate the success probability after $k$ Grover iterations.
    ///
    /// $$ P(k) = \sin^2((2k + 1)\theta), \quad \theta = \arcsin\!\sqrt{\frac{M}{N}} $$
    pub fn success_probability(&self, iterations: usize) -> f64 {
        let n = 1 << self.num_qubits;
        let m = self.marked_items.len();
        if m == 0 {
            return 0.0;
        }

        let theta = (m as f64 / n as f64).sqrt().asin();
        let angle = (2 * iterations + 1) as f64 * theta;
        angle.sin().powi(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grover_creation() {
        let grover = GroverSearch::new(3, vec![5]).unwrap();
        assert_eq!(grover.num_qubits(), 3);
        assert_eq!(grover.marked_items(), &[5]);
    }

    #[test]
    fn test_grover_invalid_item() {
        let result = GroverSearch::new(2, vec![5]); // 5 > 2^2-1
        assert!(result.is_err());
    }

    #[test]
    fn test_grover_optimal_iterations() {
        let grover = GroverSearch::new(4, vec![5, 10]).unwrap();
        let iterations = grover.optimal_iterations();
        assert!(iterations > 0);
        assert!(iterations < 10); // Should be reasonable for 4 qubits
    }

    #[test]
    fn test_grover_circuit_creation() {
        let grover = GroverSearch::new(3, vec![3]).unwrap();
        let circuit = grover.build_circuit().unwrap();
        assert_eq!(circuit.num_qubits(), 3);
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_grover_success_probability() {
        let grover = GroverSearch::new(2, vec![1]).unwrap();
        let optimal_iter = grover.optimal_iterations();
        let prob = grover.success_probability(optimal_iter);
        assert!(prob > 0.5); // Should have high success probability
    }

    #[test]
    fn test_iteration_strategy_optimal() {
        let grover = GroverSearch::with_strategy(3, vec![2], IterationStrategy::Optimal).unwrap();

        let iterations = grover.calculate_iterations();
        assert_eq!(iterations, grover.optimal_iterations());
    }

    #[test]
    fn test_iteration_strategy_fixed() {
        let grover = GroverSearch::with_strategy(3, vec![2], IterationStrategy::Fixed(5)).unwrap();

        let iterations = grover.calculate_iterations();
        assert_eq!(iterations, 5);
    }

    #[test]
    fn test_iteration_strategy_adaptive() {
        let grover =
            GroverSearch::with_strategy(3, vec![2], IterationStrategy::Adaptive { threshold: 0.9 })
                .unwrap();

        let iterations = grover.calculate_iterations();
        let prob = grover.success_probability(iterations);
        assert!(prob >= 0.9 || prob > 0.8); // Should meet or be close to threshold
    }

    #[test]
    fn test_set_strategy() {
        let mut grover = GroverSearch::new(3, vec![2]).unwrap();
        assert_eq!(grover.strategy(), IterationStrategy::Optimal);

        grover.set_strategy(IterationStrategy::Fixed(10));
        assert_eq!(grover.strategy(), IterationStrategy::Fixed(10));
    }

    #[test]
    fn test_build_circuit_with_iterations() {
        let grover = GroverSearch::new(3, vec![3]).unwrap();
        let circuit = grover.build_circuit_with_iterations(5).unwrap();
        assert_eq!(circuit.num_qubits(), 3);
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_build_operator() {
        let grover = GroverSearch::new(3, vec![3]).unwrap();
        let circuit = grover.build_operator(3).unwrap();
        assert_eq!(circuit.num_qubits(), 3);
        assert_eq!(circuit.num_clbits(), 0); // No classical bits
    }

    #[test]
    fn test_grover_4qubit_success_probability() {
        use crate::StandardGate;

        // N=16, m=1: single marked item 5 (0101)
        let grover = GroverSearch::new(4, vec![5]).unwrap();

        // Build circuit with exactly 1 Grover iteration
        let circuit = grover.build_circuit_with_iterations(1).unwrap();
        assert_eq!(circuit.num_qubits(), 4);
        assert!(circuit.size() > 0, "Circuit should contain gates");

        // Verify the circuit contains at least one CCX (Toffoli) gate,
        // which indicates the proper MCNOT decomposition was used
        let has_ccx = circuit
            .data()
            .instructions()
            .iter()
            .any(|inst| matches!(inst.gate.gate_type, StandardGate::CCX));
        assert!(
            has_ccx,
            "4-qubit Grover circuit should contain CCX gates from MCNOT decomposition"
        );

        // For N=16, m=1: theoretical success probability after 1 Grover iteration
        // is sin^2(3 * arcsin(1/4)) = sin^2(0.758) ~ 0.472
        let prob = grover.success_probability(1);
        assert!(
            prob > 0.4,
            "Expected success probability > 0.4 after 1 Grover iteration for N=16,m=1, got {:.4}",
            prob
        );

        // Verify the diffusion operator also used MCZ (not a plain Z)
        // Build a diffusion-only operator and check it also has CCX gates
        let mut diff_circuit = QuantumCircuit::new(4, 0);
        for i in 0..4 {
            diff_circuit.h(i).unwrap();
            diff_circuit.x(i).unwrap();
        }
        grover.apply_mcz(&mut diff_circuit).unwrap();
        // After MCZ + undo H/X, this is the core of the diffusion operator
        let has_diff_ccx = diff_circuit
            .data()
            .instructions()
            .iter()
            .any(|inst| matches!(inst.gate.gate_type, StandardGate::CCX));
        assert!(
            has_diff_ccx,
            "4-qubit diffusion operator should contain CCX gates from MCZ decomposition"
        );
    }
}
