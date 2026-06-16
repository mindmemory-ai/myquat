// examples/trotter_templates.rs - Demonstration of Trotter template system
// Author: gA4ss

use myquat::deoptimization::{HamiltonianTerm, TrotterTemplateBuilder};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Trotter Template Library Demo ===\n");

    // Define a simple Hamiltonian: H = ZZ + 0.5*XX + 0.3*YY
    let builder = TrotterTemplateBuilder::new()
        .add_term(HamiltonianTerm::new("ZZ", 1.0, vec![0, 1]))
        .add_term(HamiltonianTerm::new("XX", 0.5, vec![0, 1]))
        .add_term(HamiltonianTerm::new("YY", 0.3, vec![0, 1]));

    println!("Hamiltonian: H = 1.0·ZZ + 0.5·XX + 0.3·YY\n");

    // First-order Trotter
    println!("1. First-Order Trotter Template");
    println!("   Formula: exp(-iHt) ≈ ∏ exp(-iHᵢdt)");
    let first_order = builder.build_first_order(1.0, 5);
    println!("   Order: {}", first_order.order);
    println!("   Evolution time: {}", first_order.evolution_time);
    println!("   Number of steps: {}", first_order.num_steps);
    println!("   Time step (dt): {}", first_order.dt());
    println!("   Total gates: {}", first_order.steps.len());
    println!("   Structure: Sequential application of all terms");

    // Display first few steps
    println!("\n   First 3 steps:");
    for (i, step) in first_order.steps.iter().take(3).enumerate() {
        println!(
            "   Step {}: {} (time factor: {:.3})",
            i, step.term.pauli_string, step.time_factor
        );
    }

    // Second-order Trotter
    println!("\n2. Second-Order Trotter Template (Symmetric)");
    println!("   Formula: exp(-iHt) ≈ exp(-iH₁dt/2)···exp(-iHₙdt)···exp(-iH₁dt/2)");
    let second_order = builder.build_second_order(1.0, 5);
    println!("   Order: {}", second_order.order);
    println!("   Symmetric: {}", second_order.is_symmetric);
    println!("   Time step (dt): {}", second_order.dt());
    println!("   Total gates: {}", second_order.steps.len());
    println!("   Structure: Strang splitting with symmetric half-steps");

    // Display structure of one step
    println!("\n   One complete step structure:");
    for (i, step) in second_order.steps.iter().take(5).enumerate() {
        println!(
            "   {}: {} (time factor: {:.3})",
            i, step.term.pauli_string, step.time_factor
        );
    }

    // Fourth-order Suzuki
    println!("\n3. Fourth-Order Suzuki Template");
    println!("   Formula: S₄(t) = S₂(p·t)² S₂((1-4p)·t) S₂(p·t)²");
    println!("   where p ≈ 0.4145 (Suzuki coefficient)");
    let fourth_order = builder.build_fourth_order(1.0, 1);
    println!("   Order: {}", fourth_order.order);
    println!("   Symmetric: {}", fourth_order.is_symmetric);
    println!("   Total gates: {}", fourth_order.steps.len());
    println!("   Structure: Nested composition of 2nd-order formulas");

    // Serialization demo
    println!("\n4. Template Serialization");
    let json = serde_json::to_string_pretty(&first_order)?;
    println!("   First-order template as JSON (first 300 chars):");
    println!("   {}", &json.chars().take(300).collect::<String>());
    println!("   ...");

    // Error scaling comparison
    println!("\n5. Error Scaling Analysis");
    println!("   Theoretical error bounds:");
    println!("   - 1st order: O(dt²) per step");
    println!("   - 2nd order: O(dt³) per step");
    println!("   - 4th order: O(dt⁵) per step");
    println!("\n   For dt = 0.1:");
    println!("   - 1st order error: ~0.01");
    println!("   - 2nd order error: ~0.001");
    println!("   - 4th order error: ~0.00001");

    println!("\n6. Application in Reverse Extraction");
    println!("   These templates will be used to:");
    println!("   - Match against optimized circuit patterns");
    println!("   - Identify Trotter order from gate sequences");
    println!("   - Reconstruct original Hamiltonian terms");
    println!("   - Calculate evolution time from rotation angles");

    Ok(())
}
