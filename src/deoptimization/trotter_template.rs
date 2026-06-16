// deoptimization/trotter_template.rs - Trotter decomposition templates
// Author: gA4ss
//
// Standard templates for Trotter-Suzuki product formulas.
// Used for pattern matching to identify Trotter structures in optimized circuits.

use serde::{Deserialize, Serialize};

/// Represents a term in a Hamiltonian (e.g., ZZ, XX, YY)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HamiltonianTerm {
    /// Pauli string (e.g., "ZZ", "XX", "XY")
    pub pauli_string: String,
    /// Coefficient (coupling strength)
    pub coefficient: f64,
    /// Qubits this term acts on
    pub qubits: Vec<usize>,
}

impl HamiltonianTerm {
    /// Create new Hamiltonian term
    pub fn new(pauli_string: &str, coefficient: f64, qubits: Vec<usize>) -> Self {
        Self {
            pauli_string: pauli_string.to_string(),
            coefficient,
            qubits,
        }
    }
}

/// Represents a Trotter decomposition step
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrotterStep {
    /// The Hamiltonian term being applied
    pub term: HamiltonianTerm,
    /// Time coefficient (e.g., dt, dt/2 for Suzuki)
    pub time_factor: f64,
    /// Order in the sequence
    pub order: usize,
}

impl TrotterStep {
    /// Create new Trotter step
    pub fn new(term: HamiltonianTerm, time_factor: f64, order: usize) -> Self {
        Self {
            term,
            time_factor,
            order,
        }
    }
}

/// Trotter decomposition template
///
/// Represents the standard structure of a Trotter-Suzuki product formula.
/// Used for matching against circuit patterns to identify Trotter decompositions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrotterTemplate {
    /// Template name (e.g., "First-Order", "Second-Order-Symmetric")
    pub name: String,
    /// Trotter order (1, 2, 4, 6, etc.)
    pub order: usize,
    /// Sequence of steps in this template
    pub steps: Vec<TrotterStep>,
    /// Total evolution time parameter
    pub evolution_time: f64,
    /// Number of Trotter steps
    pub num_steps: usize,
    /// Whether this is a symmetric decomposition
    pub is_symmetric: bool,
}

impl TrotterTemplate {
    /// Create new template
    pub fn new(name: &str, order: usize) -> Self {
        Self {
            name: name.to_string(),
            order,
            steps: Vec::new(),
            evolution_time: 1.0,
            num_steps: 1,
            is_symmetric: false,
        }
    }

    /// Set evolution time
    pub fn with_evolution_time(mut self, time: f64) -> Self {
        self.evolution_time = time;
        self
    }

    /// Set number of Trotter steps
    pub fn with_num_steps(mut self, n: usize) -> Self {
        self.num_steps = n;
        self
    }

    /// Set symmetry flag
    pub fn with_symmetry(mut self, symmetric: bool) -> Self {
        self.is_symmetric = symmetric;
        self
    }

    /// Add a step to the template
    pub fn add_step(mut self, step: TrotterStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Get time step size (dt)
    pub fn dt(&self) -> f64 {
        self.evolution_time / self.num_steps as f64
    }
}

/// Builder for Trotter templates
pub struct TrotterTemplateBuilder {
    hamiltonian_terms: Vec<HamiltonianTerm>,
}

impl TrotterTemplateBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            hamiltonian_terms: Vec::new(),
        }
    }

    /// Add a Hamiltonian term
    pub fn add_term(mut self, term: HamiltonianTerm) -> Self {
        self.hamiltonian_terms.push(term);
        self
    }

    /// Build first-order Trotter template
    ///
    /// First-order Trotter: exp(-iHt) ≈ exp(-iH₁dt) exp(-iH₂dt) ... exp(-iHₙdt)
    pub fn build_first_order(&self, evolution_time: f64, num_steps: usize) -> TrotterTemplate {
        let dt = evolution_time / num_steps as f64;
        let mut template = TrotterTemplate::new("First-Order", 1)
            .with_evolution_time(evolution_time)
            .with_num_steps(num_steps);

        // For each Trotter step, apply all Hamiltonian terms in sequence
        for step_idx in 0..num_steps {
            for (term_idx, term) in self.hamiltonian_terms.iter().enumerate() {
                let trotter_step = TrotterStep::new(
                    term.clone(),
                    dt,
                    step_idx * self.hamiltonian_terms.len() + term_idx,
                );
                template = template.add_step(trotter_step);
            }
        }

        template
    }

    /// Build second-order Trotter template (symmetric)
    ///
    /// Second-order Trotter (Strang splitting):
    /// exp(-iHt) ≈ exp(-iH₁dt/2) exp(-iH₂dt/2) ... exp(-iHₙdt) ... exp(-iH₂dt/2) exp(-iH₁dt/2)
    pub fn build_second_order(&self, evolution_time: f64, num_steps: usize) -> TrotterTemplate {
        let dt = evolution_time / num_steps as f64;
        let mut template = TrotterTemplate::new("Second-Order-Symmetric", 2)
            .with_evolution_time(evolution_time)
            .with_num_steps(num_steps)
            .with_symmetry(true);

        let n = self.hamiltonian_terms.len();

        for step_idx in 0..num_steps {
            let base_order = step_idx * (2 * n - 1);

            // Forward half-steps for first n-1 terms
            for (term_idx, term) in self.hamiltonian_terms.iter().take(n - 1).enumerate() {
                let trotter_step = TrotterStep::new(term.clone(), dt / 2.0, base_order + term_idx);
                template = template.add_step(trotter_step);
            }

            // Full step for last term
            if let Some(last_term) = self.hamiltonian_terms.last() {
                let trotter_step = TrotterStep::new(last_term.clone(), dt, base_order + n - 1);
                template = template.add_step(trotter_step);
            }

            // Backward half-steps for first n-1 terms (in reverse)
            for (idx, term) in self.hamiltonian_terms.iter().take(n - 1).rev().enumerate() {
                let trotter_step = TrotterStep::new(term.clone(), dt / 2.0, base_order + n + idx);
                template = template.add_step(trotter_step);
            }
        }

        template
    }

    /// Build fourth-order Trotter template (Suzuki)
    ///
    /// Fourth-order Suzuki formula:
    /// S₄(t) = S₂(p·t)² S₂((1-4p)·t) S₂(p·t)²
    /// where p = 1/(4 - 4^(1/3)) ≈ 0.414490771794376
    pub fn build_fourth_order(&self, evolution_time: f64, num_steps: usize) -> TrotterTemplate {
        // Suzuki coefficient for 4th order
        const P4: f64 = 0.414_490_771_794_375_7;

        let dt = evolution_time / num_steps as f64;
        let mut template = TrotterTemplate::new("Fourth-Order-Suzuki", 4)
            .with_evolution_time(evolution_time)
            .with_num_steps(num_steps)
            .with_symmetry(true);

        let n = self.hamiltonian_terms.len();

        for step_idx in 0..num_steps {
            let mut order_counter = step_idx * n * 5; // 5 S2 applications per step

            // S₂(p·dt) - twice
            for _ in 0..2 {
                order_counter =
                    self.add_second_order_segment(&mut template, P4 * dt, order_counter);
            }

            // S₂((1-4p)·dt)
            order_counter =
                self.add_second_order_segment(&mut template, (1.0 - 4.0 * P4) * dt, order_counter);

            // S₂(p·dt) - twice more
            for _ in 0..2 {
                order_counter =
                    self.add_second_order_segment(&mut template, P4 * dt, order_counter);
            }
        }

        template
    }

    /// Helper to add a second-order segment
    fn add_second_order_segment(
        &self,
        template: &mut TrotterTemplate,
        time: f64,
        start_order: usize,
    ) -> usize {
        let n = self.hamiltonian_terms.len();
        let mut order = start_order;

        // Forward half-steps
        for term in self.hamiltonian_terms.iter().take(n - 1) {
            template
                .steps
                .push(TrotterStep::new(term.clone(), time / 2.0, order));
            order += 1;
        }

        // Full step for last term
        if let Some(last_term) = self.hamiltonian_terms.last() {
            template
                .steps
                .push(TrotterStep::new(last_term.clone(), time, order));
            order += 1;
        }

        // Backward half-steps
        for term in self.hamiltonian_terms.iter().take(n - 1).rev() {
            template
                .steps
                .push(TrotterStep::new(term.clone(), time / 2.0, order));
            order += 1;
        }

        order
    }
}

impl Default for TrotterTemplateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_hamiltonian() -> Vec<HamiltonianTerm> {
        vec![
            HamiltonianTerm::new("ZZ", 1.0, vec![0, 1]),
            HamiltonianTerm::new("XX", 0.5, vec![0, 1]),
            HamiltonianTerm::new("YY", 0.3, vec![0, 1]),
        ]
    }

    #[test]
    fn test_hamiltonian_term_creation() {
        let term = HamiltonianTerm::new("ZZ", 1.5, vec![0, 1]);
        assert_eq!(term.pauli_string, "ZZ");
        assert_eq!(term.coefficient, 1.5);
        assert_eq!(term.qubits, vec![0, 1]);
    }

    #[test]
    fn test_trotter_template_creation() {
        let template = TrotterTemplate::new("Test", 2)
            .with_evolution_time(2.0)
            .with_num_steps(10);

        assert_eq!(template.name, "Test");
        assert_eq!(template.order, 2);
        assert_eq!(template.evolution_time, 2.0);
        assert_eq!(template.num_steps, 10);
        assert_eq!(template.dt(), 0.2);
    }

    #[test]
    fn test_first_order_template() {
        let terms = create_test_hamiltonian();
        let mut builder = TrotterTemplateBuilder::new();
        for term in terms {
            builder = builder.add_term(term);
        }

        let template = builder.build_first_order(1.0, 5);

        assert_eq!(template.order, 1);
        assert_eq!(template.num_steps, 5);
        assert_eq!(template.steps.len(), 15); // 5 steps * 3 terms
        assert_eq!(template.dt(), 0.2);
    }

    #[test]
    fn test_second_order_template() {
        let terms = create_test_hamiltonian();
        let mut builder = TrotterTemplateBuilder::new();
        for term in terms {
            builder = builder.add_term(term);
        }

        let template = builder.build_second_order(1.0, 1);

        assert_eq!(template.order, 2);
        assert!(template.is_symmetric);
        assert_eq!(template.steps.len(), 5); // 2 half + 1 full + 2 half

        // Check time factors
        assert_eq!(template.steps[0].time_factor, 0.5); // First half-step
        assert_eq!(template.steps[1].time_factor, 0.5); // Second half-step
        assert_eq!(template.steps[2].time_factor, 1.0); // Full step
        assert_eq!(template.steps[3].time_factor, 0.5); // Backward half-step
        assert_eq!(template.steps[4].time_factor, 0.5); // Backward half-step
    }

    #[test]
    fn test_fourth_order_template() {
        let terms = create_test_hamiltonian();
        let mut builder = TrotterTemplateBuilder::new();
        for term in terms {
            builder = builder.add_term(term);
        }

        let template = builder.build_fourth_order(1.0, 1);

        assert_eq!(template.order, 4);
        assert!(template.is_symmetric);
        // 5 S2 applications, each with 5 steps for 3 terms
        assert_eq!(template.steps.len(), 25);
    }

    #[test]
    fn test_template_serialization() {
        let term = HamiltonianTerm::new("ZZ", 1.0, vec![0, 1]);
        let step = TrotterStep::new(term.clone(), 0.5, 0);

        let template = TrotterTemplate::new("Test", 1).add_step(step);

        let json = serde_json::to_string(&template).unwrap();
        let deserialized: TrotterTemplate = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "Test");
        assert_eq!(deserialized.order, 1);
        assert_eq!(deserialized.steps.len(), 1);
    }
}
