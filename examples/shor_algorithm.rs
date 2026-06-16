//! # Shor's Algorithm Implementation
//!
//! This example demonstrates Shor's algorithm for integer factorization,
//! one of the most famous quantum algorithms with practical applications.

use myquat::*;
use std::f64::consts::PI;

fn main() -> Result<()> {
    println!("🔢 Shor's Algorithm for Integer Factorization");
    println!("==============================================\n");

    // Demonstrate Shor's algorithm for small numbers
    demo_shor_15()?;
    demo_shor_21()?;
    demo_quantum_period_finding()?;
    demo_continued_fractions()?;

    println!("🎯 Shor's Algorithm Summary:");
    println!("- Quantum period finding is the core quantum subroutine");
    println!("- Classical post-processing extracts factors from periods");
    println!("- Exponential speedup over classical factoring algorithms");
    println!("- Threatens current RSA cryptography when scaled up");

    Ok(())
}

/// Demo: Shor's algorithm for factoring N=15
fn demo_shor_15() -> Result<()> {
    println!("🎯 Demo 1: Factoring N = 15");
    println!("---------------------------\n");

    let n = 15;
    println!("Target number to factor: N = {}", n);

    // Step 1: Choose random a coprime to N
    let a = 7; // We know gcd(7, 15) = 1
    println!("Chosen base: a = {} (gcd({}, {}) = 1)", a, a, n);

    // Step 2: Quantum period finding
    println!("\n🔬 Quantum Period Finding:");
    let period = quantum_period_finding(a, n)?;
    println!("Found period r = {} (a^r ≡ 1 mod N)", period);

    // Verify the period
    let verification = modular_exponentiation(a, period, n);
    println!(
        "Verification: {}^{} mod {} = {}",
        a, period, n, verification
    );

    // Step 3: Classical post-processing
    println!("\n🧮 Classical Post-processing:");
    if period % 2 == 0 {
        let half_period = period / 2;
        let factor1_candidate = gcd(modular_exponentiation(a, half_period, n) - 1, n);
        let factor2_candidate = gcd(modular_exponentiation(a, half_period, n) + 1, n);

        println!("Half period: r/2 = {}", half_period);
        println!(
            "{}^{} mod {} = {}",
            a,
            half_period,
            n,
            modular_exponentiation(a, half_period, n)
        );

        let factors = find_factors(factor1_candidate, factor2_candidate, n);
        if let Some((f1, f2)) = factors {
            println!("✅ Success! Factors found: {} × {} = {}", f1, f2, n);
        } else {
            println!("❌ This attempt failed, would retry with different 'a'");
        }
    } else {
        println!("❌ Period is odd, would retry with different 'a'");
    }

    println!();
    Ok(())
}

/// Demo: Shor's algorithm for factoring N=21
fn demo_shor_21() -> Result<()> {
    println!("🎯 Demo 2: Factoring N = 21");
    println!("---------------------------\n");

    let n = 21;
    let a = 8; // gcd(8, 21) = 1

    println!("Target: N = {}, Base: a = {}", n, a);

    // Quantum period finding
    let period = quantum_period_finding(a, n)?;
    println!("Quantum period finding: r = {}", period);

    // Classical factorization
    if period % 2 == 0 {
        let half_period = period / 2;
        let x = modular_exponentiation(a, half_period, n);
        let factor1 = gcd(x - 1, n);
        let factor2 = gcd(x + 1, n);

        println!("{}^{} mod {} = {}", a, half_period, n, x);
        println!(
            "gcd({} - 1, {}) = gcd({}, {}) = {}",
            x,
            n,
            x - 1,
            n,
            factor1
        );
        println!(
            "gcd({} + 1, {}) = gcd({}, {}) = {}",
            x,
            n,
            x + 1,
            n,
            factor2
        );

        if factor1 > 1 && factor1 < n {
            println!("✅ Factor found: {} × {} = {}", factor1, n / factor1, n);
        } else if factor2 > 1 && factor2 < n {
            println!("✅ Factor found: {} × {} = {}", factor2, n / factor2, n);
        }
    }

    println!();
    Ok(())
}

/// Demo: Quantum Period Finding Circuit
fn demo_quantum_period_finding() -> Result<()> {
    println!("🔬 Demo 3: Quantum Period Finding Circuit");
    println!("----------------------------------------\n");

    let n = 15;
    let a = 7;

    println!(
        "Building quantum period finding circuit for a={}, N={}",
        a, n
    );

    // Create the quantum circuit for period finding
    let circuit = create_period_finding_circuit(a, n)?;

    println!("📊 Circuit Statistics:");
    let viz = CircuitVisualizer::new();
    println!("{}", viz.statistics_table(&circuit));

    println!("🎨 Circuit Visualization (first 8 qubits):");
    // Create a simplified version for visualization
    let demo_circuit = create_simplified_period_finding_demo()?;
    println!("{}", CircuitVisualizer::to_ascii_art(&demo_circuit));

    println!("🔍 Key Components:");
    println!("1. Superposition: Create equal superposition of all counting qubits");
    println!("2. Controlled Modular Exponentiation: Apply U^(2^j) operations");
    println!("3. Inverse QFT: Extract period information from phase");
    println!("4. Measurement: Collapse to computational basis");

    Ok(())
}

/// Demo: Continued Fractions for Period Extraction
fn demo_continued_fractions() -> Result<()> {
    println!("🧮 Demo 4: Continued Fractions for Period Extraction");
    println!("---------------------------------------------------\n");

    println!("After measuring the quantum period finding circuit,");
    println!("we get a measurement outcome that encodes the period information.");
    println!();

    // Simulate some measurement outcomes
    let measurement_outcomes = vec![
        (85, 256),  // Represents 85/256
        (171, 256), // Represents 171/256
        (0, 256),   // Represents 0/256
        (128, 256), // Represents 128/256
    ];

    println!("Example measurement outcomes (outcome/2^n):");
    for (outcome, total) in &measurement_outcomes {
        println!(
            "  {}/{} = {:.6}",
            outcome,
            total,
            *outcome as f64 / *total as f64
        );

        // Apply continued fractions
        let period_candidates = continued_fraction_convergents(*outcome as f64 / *total as f64);
        println!("    Continued fraction convergents:");
        for (_i, (num, den)) in period_candidates.iter().take(5).enumerate() {
            if *den <= 15 && *den > 0 {
                // Reasonable period bounds
                println!("      {}/{} → potential period r = {}", num, den, den);
            }
        }
        println!();
    }

    println!("💡 The correct period will appear as a denominator in the convergents");
    println!("   and will satisfy a^r ≡ 1 (mod N) when verified classically.");

    Ok(())
}

/// Quantum period finding (simulated)
fn quantum_period_finding(a: u64, n: u64) -> Result<u64> {
    // For demonstration, we'll return the known periods
    // In a real implementation, this would involve:
    // 1. Creating superposition of counting qubits
    // 2. Controlled modular exponentiation
    // 3. Inverse quantum Fourier transform
    // 4. Measurement and classical post-processing

    match (a, n) {
        (7, 15) => Ok(4), // 7^4 ≡ 1 (mod 15)
        (8, 21) => Ok(2), // 8^2 ≡ 1 (mod 21)
        (2, 15) => Ok(4), // 2^4 ≡ 1 (mod 15)
        _ => {
            // General case: find period by brute force for demo
            for r in 1..n {
                if modular_exponentiation(a, r, n) == 1 {
                    return Ok(r);
                }
            }
            Err(MyQuatError::circuit_error("Period not found".to_string()))
        }
    }
}

/// Create quantum period finding circuit
fn create_period_finding_circuit(_a: u64, n: u64) -> Result<QuantumCircuit> {
    // Estimate number of qubits needed
    let counting_qubits = (n as f64).log2().ceil() as usize * 2; // 2n qubits for counting
    let work_qubits = (n as f64).log2().ceil() as usize; // n qubits for modular arithmetic

    let total_qubits = counting_qubits + work_qubits;
    let mut circuit = QuantumCircuit::new(total_qubits, counting_qubits);

    // Step 1: Initialize work register to |1⟩
    circuit.x(counting_qubits)?; // First work qubit to |1⟩

    // Step 2: Create superposition in counting register
    for i in 0..counting_qubits {
        circuit.h(i)?;
    }

    // Step 3: Controlled modular exponentiation
    // This is simplified - real implementation would need modular arithmetic circuits
    for i in 0..counting_qubits {
        // Apply controlled U^(2^i) where U|x⟩ = |ax mod N⟩
        // For demonstration, we'll use placeholder operations
        for work_qubit in counting_qubits..total_qubits {
            circuit.cx(i, work_qubit)?;
        }
    }

    // Step 4: Inverse QFT on counting register
    inverse_qft(&mut circuit, 0, counting_qubits)?;

    // Step 5: Measure counting register
    for i in 0..counting_qubits {
        circuit.measure(i, i)?;
    }

    Ok(circuit)
}

/// Create a simplified demo circuit for visualization
fn create_simplified_period_finding_demo() -> Result<QuantumCircuit> {
    let mut circuit = QuantumCircuit::new(8, 4);

    // Counting qubits (0-3) in superposition
    for i in 0..4 {
        circuit.h(i)?;
    }

    // Work qubits (4-7) initialized
    circuit.x(4)?;

    // Controlled operations (simplified)
    circuit.cx(0, 4)?;
    circuit.cx(1, 5)?;
    circuit.cx(2, 6)?;
    circuit.cx(3, 7)?;

    // Inverse QFT (simplified)
    circuit.h(3)?;
    circuit.cx(2, 3)?;
    circuit.h(2)?;
    circuit.cx(1, 2)?;
    circuit.cx(1, 3)?;
    circuit.h(1)?;
    circuit.cx(0, 1)?;
    circuit.cx(0, 2)?;
    circuit.cx(0, 3)?;
    circuit.h(0)?;

    // Measurements
    for i in 0..4 {
        circuit.measure(i, i)?;
    }

    Ok(circuit)
}

/// Inverse Quantum Fourier Transform
fn inverse_qft(circuit: &mut QuantumCircuit, start: usize, num_qubits: usize) -> Result<()> {
    for i in 0..num_qubits {
        let qubit = start + num_qubits - 1 - i;

        // Hadamard gate
        circuit.h(qubit)?;

        // Controlled phase rotations
        for j in 0..i {
            let control = start + num_qubits - 1 - j;
            let angle = -PI / (1 << (i - j)) as f64;
            circuit.cp(control, qubit, Parameter::Float(angle))?;
        }
    }

    // Swap qubits to reverse order
    for i in 0..num_qubits / 2 {
        circuit.swap(start + i, start + num_qubits - 1 - i)?;
    }

    Ok(())
}

/// Modular exponentiation: compute (base^exp) mod modulus
fn modular_exponentiation(base: u64, exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    }

    let mut result = 1;
    let mut base = base % modulus;
    let mut exp = exp;

    while exp > 0 {
        if exp % 2 == 1 {
            result = (result * base) % modulus;
        }
        exp = exp >> 1;
        base = (base * base) % modulus;
    }

    result
}

/// Greatest Common Divisor using Euclidean algorithm
fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// Find factors from candidates
fn find_factors(candidate1: u64, candidate2: u64, n: u64) -> Option<(u64, u64)> {
    if candidate1 > 1 && candidate1 < n && n % candidate1 == 0 {
        Some((candidate1, n / candidate1))
    } else if candidate2 > 1 && candidate2 < n && n % candidate2 == 0 {
        Some((candidate2, n / candidate2))
    } else {
        None
    }
}

/// Continued fraction convergents
fn continued_fraction_convergents(x: f64) -> Vec<(u64, u64)> {
    let mut convergents = Vec::new();
    let mut x = x;
    let mut h_prev2 = 0u64;
    let mut h_prev1 = 1u64;
    let mut k_prev2 = 1u64;
    let mut k_prev1 = 0u64;

    for _ in 0..10 {
        // Limit iterations
        if x.abs() < 1e-10 {
            break;
        }

        let a = x.floor() as u64;

        // Check for potential overflow
        if a > 1000 || h_prev1 > u64::MAX / 10 || k_prev1 > u64::MAX / 10 {
            break;
        }

        let h_curr = a.saturating_mul(h_prev1).saturating_add(h_prev2);
        let k_curr = a.saturating_mul(k_prev1).saturating_add(k_prev2);

        if k_curr > 0 && k_curr < 1000 {
            // Keep denominators reasonable
            convergents.push((h_curr, k_curr));
        }

        let new_x = x - a as f64;
        if new_x.abs() < 1e-10 {
            break;
        }

        x = 1.0 / new_x;
        h_prev2 = h_prev1;
        h_prev1 = h_curr;
        k_prev2 = k_prev1;
        k_prev1 = k_curr;
    }

    convergents
}
