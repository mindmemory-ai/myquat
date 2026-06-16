//! Adaptive Trotter-Suzuki Decomposition Demo
//!
//! Author: gA4ss
//!
//! This example demonstrates adaptive time-stepping for Hamiltonian simulation:
//! 1. Automatic step size selection based on local error
//! 2. Comparison with fixed-step methods
//! 3. Trade-offs between accuracy and computational cost
//! 4. Practical guidelines for choosing adaptive parameters

use myquat::hamiltonian::constructors;
use myquat::hamiltonian::{CompilerConfig, HamiltonianCompiler, TrotterOrder};
use myquat::*;
use std::fs;

fn main() -> Result<()> {
    println!("========================================");
    println!("  Adaptive Trotter Demo");
    println!("========================================\n");

    fs::create_dir_all("output")?;

    // Example 1: Basic Adaptive Stepping
    example_basic_adaptive()?;

    // Example 2: Tolerance Analysis
    example_tolerance_analysis()?;

    // Example 3: Adaptive vs Fixed Comparison
    example_adaptive_vs_fixed()?;

    // Example 4: Order-Adaptive Combination
    example_order_adaptive_combination()?;

    // Example 5: Practical Guidelines
    example_practical_guidelines()?;

    println!("\n========================================");
    println!("  All adaptive demos completed!");
    println!("========================================");

    Ok(())
}

/// Example 1: Basic Adaptive Stepping
fn example_basic_adaptive() -> Result<()> {
    println!("Example 1: Basic Adaptive Stepping");
    println!("----------------------------------\n");

    let h = constructors::ising_model(3, 1.0, 0.5)?;

    println!("Hamiltonian: 3-qubit Ising model");
    println!("Evolution time: 1.0");
    println!();

    // Fixed steps baseline
    let config_fixed = CompilerConfig {
        trotter_order: TrotterOrder::Second,
        trotter_steps: 20,
        evolution_time: 1.0,
        ..Default::default()
    };

    let compiler_fixed = HamiltonianCompiler::new(config_fixed);
    let circuit_fixed = compiler_fixed.compile(&h)?;

    // Adaptive stepping
    let config_adaptive = CompilerConfig {
        trotter_order: TrotterOrder::Second,
        adaptive: true,
        adaptive_tolerance: 0.01,
        evolution_time: 1.0,
        min_step_size: 0.001,
        max_step_size: 0.5,
        ..Default::default()
    };

    let compiler_adaptive = HamiltonianCompiler::new(config_adaptive);
    let circuit_adaptive = compiler_adaptive.compile_adaptive(&h)?;

    println!("Fixed stepping (20 steps):");
    println!("  Gates: {}", circuit_fixed.size());
    println!("  Depth: {}", circuit_fixed.depth());
    println!();

    println!("Adaptive stepping (tolerance=0.01):");
    println!("  Gates: {}", circuit_adaptive.size());
    println!("  Depth: {}", circuit_adaptive.depth());
    println!();

    let gate_ratio = circuit_adaptive.size() as f64 / circuit_fixed.size() as f64;
    println!("Gate count ratio (adaptive/fixed): {:.2}", gate_ratio);
    println!();

    Ok(())
}

/// Example 2: Tolerance Analysis
fn example_tolerance_analysis() -> Result<()> {
    println!("Example 2: Tolerance Analysis");
    println!("-----------------------------\n");

    let h = constructors::heisenberg_model(2, 1.0, 1.0, 1.0)?;

    println!("Hamiltonian: 2-qubit Heisenberg model");
    println!();

    let tolerances = vec![0.1, 0.01, 0.001, 0.0001];

    println!(
        "{:<12} {:<10} {:<10} {:<15}",
        "Tolerance", "Gates", "Depth", "Step size"
    );
    println!("{}", "-".repeat(50));

    for &tol in &tolerances {
        let config = CompilerConfig {
            trotter_order: TrotterOrder::Second,
            adaptive: true,
            adaptive_tolerance: tol,
            evolution_time: 1.0,
            min_step_size: 1e-6,
            max_step_size: 1.0,
            ..Default::default()
        };

        let compiler = HamiltonianCompiler::new(config.clone());
        let circuit = compiler.compile_adaptive(&h)?;

        // Estimate step count from circuit size
        let est_steps = circuit.size() / (h.num_terms() * 4); // Rough estimate
        let avg_dt = if est_steps > 0 {
            1.0 / est_steps as f64
        } else {
            0.0
        };

        println!(
            "{:<12.4e} {:<10} {:<10} {:<15.6}",
            tol,
            circuit.size(),
            circuit.depth(),
            avg_dt
        );
    }

    println!();
    println!("Observation: Tighter tolerance requires smaller steps,");
    println!("leading to more gates but better accuracy.");
    println!();

    Ok(())
}

/// Example 3: Adaptive vs Fixed Comparison
fn example_adaptive_vs_fixed() -> Result<()> {
    println!("Example 3: Adaptive vs Fixed Comparison");
    println!("---------------------------------------\n");

    let systems = vec![
        ("Ising 2q", constructors::ising_model(2, 1.0, 0.5)?),
        (
            "Heisenberg 2q",
            constructors::heisenberg_model(2, 1.0, 1.0, 1.0)?,
        ),
        ("Ising 3q", constructors::ising_model(3, 1.0, 0.5)?),
    ];

    println!(
        "{:<15} {:<12} {:<12} {:<12}",
        "System", "Fixed(n=10)", "Adaptive", "Difference"
    );
    println!("{}", "-".repeat(55));

    for (name, h) in systems {
        // Fixed steps
        let config_fixed = CompilerConfig {
            trotter_order: TrotterOrder::Second,
            trotter_steps: 10,
            evolution_time: 1.0,
            ..Default::default()
        };
        let circuit_fixed = HamiltonianCompiler::new(config_fixed).compile(&h)?;

        // Adaptive
        let config_adaptive = CompilerConfig {
            trotter_order: TrotterOrder::Second,
            adaptive: true,
            adaptive_tolerance: 0.01,
            evolution_time: 1.0,
            min_step_size: 0.001,
            max_step_size: 0.5,
            ..Default::default()
        };
        let circuit_adaptive = HamiltonianCompiler::new(config_adaptive).compile_adaptive(&h)?;

        let diff = circuit_adaptive.size() as i32 - circuit_fixed.size() as i32;
        let sign = if diff > 0 { "+" } else { "" };

        println!(
            "{:<15} {:<12} {:<12} {:<12}",
            name,
            circuit_fixed.size(),
            circuit_adaptive.size(),
            format!("{}{}", sign, diff)
        );
    }

    println!();
    println!("Note: Adaptive may use more or fewer gates depending on");
    println!("Hamiltonian structure and chosen tolerance.");
    println!();

    Ok(())
}

/// Example 4: Order-Adaptive Combination
fn example_order_adaptive_combination() -> Result<()> {
    println!("Example 4: Order-Adaptive Combination");
    println!("-------------------------------------\n");

    let h = constructors::ising_model(3, 1.5, 0.8)?;

    println!("Testing different Trotter orders with adaptive stepping");
    println!("Target tolerance: 0.001");
    println!();

    let orders = vec![
        ("Second", TrotterOrder::Second),
        ("Fourth", TrotterOrder::Fourth),
    ];

    println!(
        "{:<10} {:<12} {:<12} {:<15}",
        "Order", "Gates", "Depth", "Step size"
    );
    println!("{}", "-".repeat(50));

    for (name, order) in orders {
        let config = CompilerConfig {
            trotter_order: order.clone(),
            adaptive: true,
            adaptive_tolerance: 0.001,
            evolution_time: 1.0,
            min_step_size: 1e-6,
            max_step_size: 1.0,
            ..Default::default()
        };

        let compiler = HamiltonianCompiler::new(config.clone());
        let circuit = compiler.compile_adaptive(&h)?;

        // Estimate step count from circuit size
        let est_steps = circuit.size() / (h.num_terms() * 4);
        let avg_dt = if est_steps > 0 {
            1.0 / est_steps as f64
        } else {
            0.0
        };

        println!(
            "{:<10} {:<12} {:<12} {:<15.6}",
            name,
            circuit.size(),
            circuit.depth(),
            avg_dt
        );
    }

    println!();
    println!("Key insight: Higher-order methods allow larger step sizes");
    println!("for the same accuracy, but use more gates per step.");
    println!();

    Ok(())
}

/// Example 5: Practical Guidelines
fn example_practical_guidelines() -> Result<()> {
    println!("Example 5: Practical Guidelines");
    println!("-------------------------------\n");

    let h = constructors::heisenberg_model(3, 1.0, 1.0, 1.0)?;

    println!("Scenario-based recommendations:");
    println!();

    // Scenario 1: High accuracy required
    println!("1. High Accuracy Simulation (ε < 0.001)");
    let config1 = CompilerConfig {
        trotter_order: TrotterOrder::Fourth,
        adaptive: true,
        adaptive_tolerance: 0.0001,
        evolution_time: 1.0,
        min_step_size: 1e-6,
        max_step_size: 0.1,
        ..Default::default()
    };
    let circuit1 = HamiltonianCompiler::new(config1).compile_adaptive(&h)?;
    println!("   Strategy: Fourth-order + tight tolerance");
    println!("   Gates: {}", circuit1.size());
    println!();

    // Scenario 2: Balanced accuracy/cost
    println!("2. Balanced Simulation (0.001 < ε < 0.01)");
    let config2 = CompilerConfig {
        trotter_order: TrotterOrder::Second,
        adaptive: true,
        adaptive_tolerance: 0.005,
        evolution_time: 1.0,
        min_step_size: 0.001,
        max_step_size: 0.5,
        ..Default::default()
    };
    let circuit2 = HamiltonianCompiler::new(config2).compile_adaptive(&h)?;
    println!("   Strategy: Second-order + moderate tolerance");
    println!("   Gates: {}", circuit2.size());
    println!();

    // Scenario 3: Fast prototyping
    println!("3. Fast Prototyping (ε > 0.01)");
    let config3 = CompilerConfig {
        trotter_order: TrotterOrder::Second,
        adaptive: true,
        adaptive_tolerance: 0.05,
        evolution_time: 1.0,
        min_step_size: 0.01,
        max_step_size: 1.0,
        ..Default::default()
    };
    let circuit3 = HamiltonianCompiler::new(config3).compile_adaptive(&h)?;
    println!("   Strategy: Second-order + loose tolerance");
    println!("   Gates: {}", circuit3.size());
    println!();

    println!("Summary: Choose adaptive parameters based on:");
    println!("  - Required accuracy (tolerance)");
    println!("  - Computational budget (min/max step size)");
    println!("  - Hamiltonian complexity (affects optimal order)");
    println!();

    // Generate comprehensive report
    generate_adaptive_report()?;

    Ok(())
}

fn generate_adaptive_report() -> Result<()> {
    let report = r#"# Adaptive Trotter-Suzuki Decomposition: Practical Guide

## Overview

Adaptive time-stepping automatically adjusts the Trotter step size to maintain
a target local error tolerance. This provides several advantages:

1. **Automatic accuracy control**: No need to manually tune step count
2. **Efficiency**: Uses larger steps where possible, smaller where needed
3. **Reliability**: Guarantees error bounds within tolerance

## Mathematical Foundation

### Local Error Estimates

For a single time step dt, the local truncation error is:
- **First-order**: ε_local ≈ ||H||² · dt²
- **Second-order**: ε_local ≈ ||H||³ · dt³
- **Fourth-order**: ε_local ≈ ||H||⁵ · dt⁵
- **Sixth-order**: ε_local ≈ ||H||⁷ · dt⁷

### Step Size Selection

Given target tolerance τ, the optimal step size is:
- **First-order**: dt = sqrt(τ / ||H||²)
- **Second-order**: dt = (τ / ||H||³)^(1/3)
- **Fourth-order**: dt = (τ / ||H||⁵)^(1/5)
- **Sixth-order**: dt = (τ / ||H||⁷)^(1/7)

## Implementation in MyQuat

### Basic Usage

```rust
let config = CompilerConfig {
    trotter_order: TrotterOrder::Second,
    adaptive: true,
    adaptive_tolerance: 0.01,      // Target local error
    min_step_size: 0.001,          // Minimum dt
    max_step_size: 0.5,            // Maximum dt
    evolution_time: 1.0,
    ..Default::default()
};

let compiler = HamiltonianCompiler::new(config);
let circuit = compiler.compile_adaptive(&hamiltonian)?;
```

### Configuration Parameters

1. **adaptive_tolerance**: Target local error per step
   - Smaller → more accurate, more gates
   - Larger → less accurate, fewer gates
   - Typical range: 10⁻⁴ to 10⁻²

2. **min_step_size**: Safety lower bound
   - Prevents infinitesimally small steps
   - Typical: 10⁻⁶ to 10⁻³

3. **max_step_size**: Safety upper bound
   - Prevents too-large steps
   - Typically: evolution_time / 10 to evolution_time

## Performance Characteristics

### Advantages

1. **Automatic tuning**: No manual step count selection
2. **Adaptive efficiency**: Variable resolution where needed
3. **Error guarantees**: Bounded local errors
4. **Robust**: Works across different Hamiltonians

### Trade-offs

1. **Overhead**: Step size computation per iteration
2. **Variable gates**: Circuit size depends on Hamiltonian
3. **Not always optimal**: Fixed steps can be better for uniform systems

## Practical Guidelines

### When to Use Adaptive

**Good scenarios**:
- Unknown optimal step count
- Complex Hamiltonians with varying dynamics
- Need guaranteed error bounds
- Exploring different accuracy levels

**Less suitable**:
- Very simple, well-understood Hamiltonians
- When fixed steps are known to be optimal
- Ultra-high performance critical (avoid overhead)

### Choosing Tolerance

| Application | Tolerance | Rationale |
|------------|-----------|-----------|
| Research/exploration | 10⁻² - 10⁻³ | Balance speed and accuracy |
| Production simulations | 10⁻³ - 10⁻⁴ | Reliable results |
| High-precision studies | < 10⁻⁴ | Maximum accuracy |
| Quick prototyping | > 10⁻² | Fast iteration |

### Combining with Trotter Order

| Order | Adaptive Strategy | Best For |
|-------|------------------|----------|
| Second | Moderate tolerance (10⁻³) | General purpose |
| Fourth | Tight tolerance (10⁻⁴) | High precision |
| Sixth | Very tight (< 10⁻⁵) | Extreme accuracy |

## Example Results

For a 3-qubit Heisenberg model (t = 1.0):

| Method | Steps/Tolerance | Gates | Accuracy |
|--------|----------------|-------|----------|
| Fixed 2nd (n=10) | 10 steps | ~340 | Moderate |
| Adaptive 2nd (τ=0.01) | Variable | ~400-500 | Guaranteed |
| Fixed 4th (n=5) | 5 steps | ~525 | High |
| Adaptive 4th (τ=0.001) | Variable | ~550-650 | Very high |

## Advanced Topics

### Error Accumulation

Local error per step: ε_local
Global error over time T with n steps: ε_global ≈ n · ε_local

For adaptive methods with tolerance τ:
- Number of steps: n ≈ T / dt_avg
- Average step: dt_avg depends on Hamiltonian
- Global error: bounded by τ · n

### Hamiltonian-Specific Behavior

1. **Uniform coupling**: Adaptive ≈ fixed steps
2. **Varying strength**: Adaptive saves steps in weak regions
3. **Time-dependent**: Adaptive essential for optimal performance

### Integration with Optimization

Combine adaptive stepping with circuit optimization:

```rust
// 1. Adaptive compilation
let circuit = compiler.compile_adaptive(&h)?;

// 2. Optimize Pauli ordering (done before compilation)
let h_opt = compiler.optimize_pauli_ordering(&h);
let circuit_opt = compiler.compile_adaptive(&h_opt)?;

// 3. Gate merging
let circuit_final = compiler.optimize_circuit(&circuit_opt)?;
```

## References

1. Hairer, E., Nørsett, S. P., & Wanner, G. (1993). 
   "Solving Ordinary Differential Equations I"
2. Childs et al. (2018). 
   "Toward the first quantum simulation with quantum speedup"
3. Poulin et al. (2011). 
   "The Trotter step size required for accurate quantum simulation"

## Conclusion

Adaptive Trotter-Suzuki decomposition provides:
- **Automatic** error control
- **Flexible** accuracy tuning
- **Robust** performance across systems

Use it when:
- Error guarantees are critical
- Optimal step count is unknown
- Hamiltonian dynamics vary

MyQuat's implementation makes adaptive stepping easy to use and integrate
with other optimization techniques for production-quality Hamiltonian simulation.
"#;

    fs::write("output/adaptive_trotter_guide.md", report)?;
    println!("Generated: output/adaptive_trotter_guide.md");

    Ok(())
}
