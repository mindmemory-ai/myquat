//! Forward Conversion Demo: Hamiltonian to Quantum Circuit
//!
//! Author: gA4ss
//!
//! This example demonstrates the conversion from Hamiltonian to quantum circuit
//! using Trotter-Suzuki decomposition. It exports results in LaTeX and Markdown
//! formats for documentation and publication.

use myquat::hamiltonian::*;
use myquat::*;
use num_complex::Complex64;
use std::fs;

fn main() -> Result<()> {
    println!("========================================");
    println!("  Forward Conversion: H -> Circuit");
    println!("========================================\n");

    // Create output directory
    fs::create_dir_all("output")?;

    // Example 1: Single-qubit Hamiltonian
    example_single_qubit()?;

    // Example 2: Ising model
    example_ising_model()?;

    // Example 3: Heisenberg model
    example_heisenberg_model()?;

    // Example 4: Custom Hamiltonian
    example_custom_hamiltonian()?;

    // Example 5: Comparison of Trotter orders
    example_trotter_comparison()?;

    println!("\n========================================");
    println!("  All examples completed!");
    println!("  Check output/ directory for LaTeX and Markdown files");
    println!("========================================");

    Ok(())
}

/// Example 1: Single-qubit Hamiltonian H = X + 0.5*Z
fn example_single_qubit() -> Result<()> {
    println!("Example 1: Single-Qubit Hamiltonian");
    println!("------------------------------------\n");

    // Create Hamiltonian: H = X + 0.5*Z
    let mut h = Hamiltonian::new(1);

    let x = PauliString::from_str("X")?;
    h.add_term(x, Complex64::new(1.0, 0.0))?;

    let z = PauliString::from_str("Z")?;
    h.add_term(z, Complex64::new(0.5, 0.0))?;

    println!("Hamiltonian:");
    println!("{}\n", h);

    // Compile to circuit
    let config = CompilerConfig {
        trotter_order: TrotterOrder::Second,
        trotter_steps: 5,
        evolution_time: 1.0,
        ..Default::default()
    };

    let compiler = HamiltonianCompiler::new(config.clone());
    let circuit = compiler.compile(&h)?;

    println!("Circuit Info:");
    println!("  Qubits: {}", circuit.num_qubits());
    println!("  Gates: {}", circuit.size());
    println!("  Trotter order: Second");
    println!("  Trotter steps: 5");
    println!("  Evolution time: 1.0\n");

    // Generate LaTeX document
    let latex = generate_latex_single_qubit(&h, &circuit, &config);
    fs::write("output/example1_single_qubit.tex", latex)?;

    // Generate Markdown document
    let markdown = generate_markdown_single_qubit(&h, &circuit, &config);
    fs::write("output/example1_single_qubit.md", markdown)?;

    println!("Output files:");
    println!("  - output/example1_single_qubit.tex");
    println!("  - output/example1_single_qubit.md\n");

    Ok(())
}

/// Example 2: Ising model
fn example_ising_model() -> Result<()> {
    println!("Example 2: Ising Model");
    println!("----------------------\n");

    // Create 3-qubit Ising model
    let h = constructors::ising_model(3, 1.0, 0.5)?;

    println!("Hamiltonian:");
    println!("{}\n", h);

    // Compile with first-order Trotter
    let config = CompilerConfig {
        trotter_order: TrotterOrder::First,
        trotter_steps: 10,
        evolution_time: 2.0,
        ..Default::default()
    };

    let compiler = HamiltonianCompiler::new(config.clone());
    let circuit = compiler.compile(&h)?;

    println!("Circuit Info:");
    println!("  Qubits: {}", circuit.num_qubits());
    println!("  Gates: {}", circuit.size());
    println!("  Trotter order: First");
    println!("  Trotter steps: 10");
    println!("  Evolution time: 2.0\n");

    // Generate documents
    let latex = generate_latex_ising(&h, &circuit, &config);
    fs::write("output/example2_ising_model.tex", latex)?;

    let markdown = generate_markdown_ising(&h, &circuit, &config);
    fs::write("output/example2_ising_model.md", markdown)?;

    println!("Output files:");
    println!("  - output/example2_ising_model.tex");
    println!("  - output/example2_ising_model.md\n");

    Ok(())
}

/// Example 3: Heisenberg model
fn example_heisenberg_model() -> Result<()> {
    println!("Example 3: Heisenberg Model");
    println!("---------------------------\n");

    // Create 3-qubit Heisenberg model
    let h = constructors::heisenberg_model(3, 1.0, 1.0, 1.0)?;

    println!("Hamiltonian:");
    println!("{}\n", h);

    // Compile with second-order Trotter
    let config = CompilerConfig {
        trotter_order: TrotterOrder::Second,
        trotter_steps: 8,
        evolution_time: 1.5,
        ..Default::default()
    };

    let compiler = HamiltonianCompiler::new(config.clone());
    let circuit = compiler.compile(&h)?;

    println!("Circuit Info:");
    println!("  Qubits: {}", circuit.num_qubits());
    println!("  Gates: {}", circuit.size());
    println!("  Trotter order: Second");
    println!("  Trotter steps: 8");
    println!("  Evolution time: 1.5\n");

    // Generate documents
    let latex = generate_latex_heisenberg(&h, &circuit, &config);
    fs::write("output/example3_heisenberg_model.tex", latex)?;

    let markdown = generate_markdown_heisenberg(&h, &circuit, &config);
    fs::write("output/example3_heisenberg_model.md", markdown)?;

    println!("Output files:");
    println!("  - output/example3_heisenberg_model.tex");
    println!("  - output/example3_heisenberg_model.md\n");

    Ok(())
}

/// Example 4: Custom Hamiltonian
fn example_custom_hamiltonian() -> Result<()> {
    println!("Example 4: Custom Hamiltonian");
    println!("-----------------------------\n");

    // Create custom 2-qubit Hamiltonian
    let mut h = Hamiltonian::new(2);

    // Add XY coupling
    let xy = PauliString::from_str("XY")?;
    h.add_term(xy, Complex64::new(0.7, 0.0))?;

    // Add ZZ interaction
    let zz = PauliString::from_str("ZZ")?;
    h.add_term(zz, Complex64::new(-0.3, 0.0))?;

    // Add single-qubit terms
    let xi = PauliString::from_str("XI")?;
    h.add_term(xi, Complex64::new(0.5, 0.0))?;

    let iz = PauliString::from_str("IZ")?;
    h.add_term(iz, Complex64::new(-0.2, 0.0))?;

    println!("Hamiltonian:");
    println!("{}\n", h);

    // Compile
    let config = CompilerConfig {
        trotter_order: TrotterOrder::Second,
        trotter_steps: 6,
        evolution_time: 1.0,
        ..Default::default()
    };

    let compiler = HamiltonianCompiler::new(config.clone());
    let circuit = compiler.compile(&h)?;

    println!("Circuit Info:");
    println!("  Qubits: {}", circuit.num_qubits());
    println!("  Gates: {}", circuit.size());
    println!("  Trotter order: Second");
    println!("  Trotter steps: 6");
    println!("  Evolution time: 1.0\n");

    // Generate documents
    let latex = generate_latex_custom(&h, &circuit, &config);
    fs::write("output/example4_custom.tex", latex)?;

    let markdown = generate_markdown_custom(&h, &circuit, &config);
    fs::write("output/example4_custom.md", markdown)?;

    println!("Output files:");
    println!("  - output/example4_custom.tex");
    println!("  - output/example4_custom.md\n");

    Ok(())
}

/// Example 5: Comparison of Trotter orders
fn example_trotter_comparison() -> Result<()> {
    println!("Example 5: Trotter Order Comparison");
    println!("-----------------------------------\n");

    // Use a simple Hamiltonian for comparison
    let mut h = Hamiltonian::new(2);
    let xx = PauliString::from_str("XX")?;
    h.add_term(xx, Complex64::new(1.0, 0.0))?;
    let yy = PauliString::from_str("YY")?;
    h.add_term(yy, Complex64::new(1.0, 0.0))?;

    println!("Hamiltonian:");
    println!("{}\n", h);

    // First-order Trotter
    let config1 = CompilerConfig {
        trotter_order: TrotterOrder::First,
        trotter_steps: 5,
        evolution_time: 1.0,
        ..Default::default()
    };
    let compiler1 = HamiltonianCompiler::new(config1.clone());
    let circuit1 = compiler1.compile(&h)?;

    // Second-order Trotter
    let config2 = CompilerConfig {
        trotter_order: TrotterOrder::Second,
        trotter_steps: 5,
        evolution_time: 1.0,
        ..Default::default()
    };
    let compiler2 = HamiltonianCompiler::new(config2.clone());
    let circuit2 = compiler2.compile(&h)?;

    println!("First-order Trotter:");
    println!("  Gates: {}", circuit1.size());
    println!("  Error: O(t^2/n) = O(0.2)\n");

    println!("Second-order Trotter:");
    println!("  Gates: {}", circuit2.size());
    println!("  Error: O(t^3/n^2) = O(0.04)\n");

    // Generate documents
    let latex = generate_latex_comparison(&h, &circuit1, &circuit2, &config1, &config2);
    fs::write("output/example5_trotter_comparison.tex", latex)?;

    let markdown = generate_markdown_comparison(&h, &circuit1, &circuit2, &config1, &config2);
    fs::write("output/example5_trotter_comparison.md", markdown)?;

    println!("Output files:");
    println!("  - output/example5_trotter_comparison.tex");
    println!("  - output/example5_trotter_comparison.md\n");

    Ok(())
}

// LaTeX generation functions

fn generate_latex_single_qubit(
    h: &Hamiltonian,
    circuit: &QuantumCircuit,
    config: &CompilerConfig,
) -> String {
    format!(
        r#"\documentclass{{article}}
\usepackage{{amsmath}}
\usepackage{{physics}}
\usepackage{{quantikz}}

\title{{Single-Qubit Hamiltonian Simulation}}
\author{{MyQuat Framework}}
\date{{\today}}

\begin{{document}}

\maketitle

\section{{Hamiltonian}}

The target Hamiltonian is:
{}

This represents a combination of X and Z rotations on a single qubit.

\section{{Physical Interpretation}}

\begin{{itemize}}
\item The $\sigma_x$ term causes rotation around the x-axis
\item The $\sigma_z$ term causes rotation around the z-axis
\item The combined effect is a rotation on the Bloch sphere
\end{{itemize}}

\section{{Time Evolution}}

The time evolution operator is:
$$U(t) = e^{{-i\hat{{H}}t}} = e^{{-i(\sigma_x + 0.5\sigma_z)t}}$$

\section{{Trotter-Suzuki Decomposition}}

We use the second-order Suzuki formula with {} steps and evolution time $t = {}$:
$$U(t) \approx \left[e^{{-i\hat{{H}}_1 \Delta t/2}} e^{{-i\hat{{H}}_2 \Delta t}} e^{{-i\hat{{H}}_1 \Delta t/2}}\right]^n$$

where $\Delta t = t/n = {:.3}$.

\section{{Circuit Implementation}}

The compiled quantum circuit has:
\begin{{itemize}}
\item Qubits: {}
\item Total gates: {}
\item Estimated depth: {}
\end{{itemize}}

\section{{Error Analysis}}

The Trotter error for second-order decomposition is:
$$\epsilon = O\left(\frac{{t^3}}{{n^2}}\right) = O({:.3e})$$

\end{{document}}
"#,
        h.to_latex(),
        config.trotter_steps,
        config.evolution_time,
        config.evolution_time / config.trotter_steps as f64,
        circuit.num_qubits(),
        circuit.size(),
        circuit.size() / 2,
        config.evolution_time.powi(3) / (config.trotter_steps as f64).powi(2)
    )
}

fn generate_latex_ising(
    h: &Hamiltonian,
    circuit: &QuantumCircuit,
    config: &CompilerConfig,
) -> String {
    format!(
        r#"\documentclass{{article}}
\usepackage{{amsmath}}
\usepackage{{physics}}

\title{{Ising Model Hamiltonian Simulation}}
\author{{MyQuat Framework}}

\begin{{document}}

\maketitle

\section{{Ising Model Hamiltonian}}

The transverse-field Ising model Hamiltonian is:
{}

Physical parameters:
\begin{{itemize}}
\item $J = 1.0$ (coupling strength)
\item $h = 0.5$ (transverse field strength)
\item Number of qubits: {}
\end{{itemize}}

\section{{Physical Significance}}

The Ising model describes:
\begin{{itemize}}
\item Nearest-neighbor spin interactions ($Z_i Z_j$ terms)
\item External transverse magnetic field ($X_i$ terms)
\item Competition between quantum tunneling and classical ordering
\end{{itemize}}

\section{{Compilation Statistics}}

\begin{{itemize}}
\item Trotter order: First
\item Trotter steps: {}
\item Evolution time: {}
\item Total gates: {}
\item Step size: $\Delta t = {:.3}$
\end{{itemize}}

\section{{Applications}}

This Hamiltonian is used in:
\begin{{itemize}}
\item Quantum annealing
\item Spin glass studies
\item Phase transition research
\item Quantum optimization (QAOA)
\end{{itemize}}

\end{{document}}
"#,
        h.to_latex(),
        h.num_qubits,
        config.trotter_steps,
        config.evolution_time,
        circuit.size(),
        config.evolution_time / config.trotter_steps as f64
    )
}

fn generate_latex_heisenberg(
    h: &Hamiltonian,
    circuit: &QuantumCircuit,
    config: &CompilerConfig,
) -> String {
    format!(
        r#"\documentclass{{article}}
\usepackage{{amsmath}}
\usepackage{{physics}}

\title{{Heisenberg Model Hamiltonian Simulation}}
\author{{MyQuat Framework}}

\begin{{document}}

\maketitle

\section{{Heisenberg Model}}

The isotropic Heisenberg model Hamiltonian is:
{}

This is the XXX model with $J_x = J_y = J_z = 1.0$.

\section{{Physical Background}}

The Heisenberg model describes:
\begin{{itemize}}
\item Quantum spin-1/2 systems
\item Exchange interactions between neighboring spins
\item SU(2) symmetry (isotropic case)
\item Quantum magnetism
\end{{itemize}}

\section{{Circuit Compilation}}

\begin{{itemize}}
\item Qubits: {}
\item Gates: {}
\item Trotter order: Second
\item Steps: {}
\item Time: {}
\end{{itemize}}

\section{{Advantages of Second-Order Trotter}}

The second-order formula provides:
\begin{{itemize}}
\item Better error scaling: $O(t^3/n^2)$ vs $O(t^2/n)$
\item Symmetric decomposition
\item Higher fidelity for the same number of steps
\end{{itemize}}

\end{{document}}
"#,
        h.to_latex(),
        circuit.num_qubits(),
        circuit.size(),
        config.trotter_steps,
        config.evolution_time
    )
}

fn generate_latex_custom(
    h: &Hamiltonian,
    circuit: &QuantumCircuit,
    config: &CompilerConfig,
) -> String {
    format!(
        r#"\documentclass{{article}}
\usepackage{{amsmath}}
\usepackage{{physics}}

\title{{Custom Two-Qubit Hamiltonian Simulation}}
\author{{MyQuat Framework}}

\begin{{document}}

\masetitle

\section{{Custom Hamiltonian}}

{}

\section{{Term Analysis}}

This Hamiltonian contains:
\begin{{itemize}}
\item $XY$ coupling: Creates entanglement between qubits
\item $ZZ$ interaction: Diagonal term, easier to implement
\item $X_0$ term: Single-qubit rotation on qubit 0
\item $Z_1$ term: Phase rotation on qubit 1
\end{{itemize}}

\section{{Compilation Result}}

\begin{{tabular}}{{ll}}
Qubits & {} \\
Gates & {} \\
Trotter order & Second \\
Steps & {} \\
Evolution time & {} \\
\end{{tabular}}

\end{{document}}
"#,
        h.to_latex(),
        circuit.num_qubits(),
        circuit.size(),
        config.trotter_steps,
        config.evolution_time
    )
}

fn generate_latex_comparison(
    h: &Hamiltonian,
    circuit1: &QuantumCircuit,
    circuit2: &QuantumCircuit,
    config1: &CompilerConfig,
    config2: &CompilerConfig,
) -> String {
    format!(
        r#"\documentclass{{article}}
\usepackage{{amsmath}}
\usepackage{{physics}}

\title{{Trotter Order Comparison}}
\author{{MyQuat Framework}}

\begin{{document}}

\maketitle

\section{{Hamiltonian}}

{}

\section{{Comparison Table}}

\begin{{tabular}}{{lcc}}
\hline
Property & First-Order & Second-Order \\
\hline
Gates & {} & {} \\
Trotter steps & {} & {} \\
Error scaling & $O(t^2/n)$ & $O(t^3/n^2)$ \\
Estimated error & $O(0.2)$ & $O(0.04)$ \\
\hline
\end{{tabular}}

\section{{Trade-offs}}

\begin{{itemize}}
\item First-order: Fewer gates, lower accuracy
\item Second-order: More gates, higher accuracy
\item For the same error, second-order requires fewer steps
\end{{itemize}}

\section{{Recommendation}}

For production use, second-order Trotter is preferred when:
\begin{{itemize}}
\item High fidelity is required
\item Gate count is not the primary constraint
\item Evolution time is not too large
\end{{itemize}}

\end{{document}}
"#,
        h.to_latex(),
        circuit1.size(),
        circuit2.size(),
        config1.trotter_steps,
        config2.trotter_steps
    )
}

// Markdown generation functions (simplified versions)

fn generate_markdown_single_qubit(
    h: &Hamiltonian,
    circuit: &QuantumCircuit,
    config: &CompilerConfig,
) -> String {
    format!(
        r#"# Single-Qubit Hamiltonian Simulation

## Hamiltonian

{}

This represents a combination of X and Z rotations on a single qubit.

## Time Evolution

The time evolution operator is:

$$U(t) = e^{{-i\hat{{H}}t}} = e^{{-i(\sigma_x + 0.5\sigma_z)t}}$$

## Trotter-Suzuki Decomposition

- **Trotter order**: Second
- **Steps**: {}
- **Evolution time**: {}
- **Step size**: Δt = {:.3}

## Circuit Implementation

- **Qubits**: {}
- **Total gates**: {}
- **Estimated depth**: {}

## Error Analysis

The Trotter error for second-order decomposition is:

$$\epsilon = O\left(\frac{{t^3}}{{n^2}}\right) = O({:.3e})$$
"#,
        h.to_markdown(),
        config.trotter_steps,
        config.evolution_time,
        config.evolution_time / config.trotter_steps as f64,
        circuit.num_qubits(),
        circuit.size(),
        circuit.size() / 2,
        config.evolution_time.powi(3) / (config.trotter_steps as f64).powi(2)
    )
}

fn generate_markdown_ising(
    h: &Hamiltonian,
    circuit: &QuantumCircuit,
    config: &CompilerConfig,
) -> String {
    format!(
        r#"# Ising Model Hamiltonian Simulation

## Ising Model Hamiltonian

{}

**Physical parameters**:
- J = 1.0 (coupling strength)
- h = 0.5 (transverse field strength)
- Number of qubits: {}

## Compilation Statistics

- **Trotter order**: First
- **Trotter steps**: {}
- **Evolution time**: {}
- **Total gates**: {}
- **Step size**: Δt = {:.3}

## Applications

This Hamiltonian is used in:
- Quantum annealing
- Spin glass studies
- Phase transition research
- Quantum optimization (QAOA)
"#,
        h.to_markdown(),
        h.num_qubits,
        config.trotter_steps,
        config.evolution_time,
        circuit.size(),
        config.evolution_time / config.trotter_steps as f64
    )
}

fn generate_markdown_heisenberg(
    h: &Hamiltonian,
    circuit: &QuantumCircuit,
    config: &CompilerConfig,
) -> String {
    format!(
        r#"# Heisenberg Model Hamiltonian Simulation

## Heisenberg Model

{}

This is the XXX model with Jx = Jy = Jz = 1.0.

## Circuit Compilation

- **Qubits**: {}
- **Gates**: {}
- **Trotter order**: Second
- **Steps**: {}
- **Time**: {}

## Advantages of Second-Order Trotter

- Better error scaling: O(t³/n²) vs O(t²/n)
- Symmetric decomposition
- Higher fidelity for the same number of steps
"#,
        h.to_markdown(),
        circuit.num_qubits(),
        circuit.size(),
        config.trotter_steps,
        config.evolution_time
    )
}

fn generate_markdown_custom(
    h: &Hamiltonian,
    circuit: &QuantumCircuit,
    config: &CompilerConfig,
) -> String {
    format!(
        r#"# Custom Two-Qubit Hamiltonian Simulation

## Custom Hamiltonian

{}

## Term Analysis

This Hamiltonian contains:
- **XY coupling**: Creates entanglement between qubits
- **ZZ interaction**: Diagonal term, easier to implement
- **X₀ term**: Single-qubit rotation on qubit 0
- **Z₁ term**: Phase rotation on qubit 1

## Compilation Result

| Property | Value |
|----------|-------|
| Qubits | {} |
| Gates | {} |
| Trotter order | Second |
| Steps | {} |
| Evolution time | {} |
"#,
        h.to_markdown(),
        circuit.num_qubits(),
        circuit.size(),
        config.trotter_steps,
        config.evolution_time
    )
}

fn generate_markdown_comparison(
    h: &Hamiltonian,
    circuit1: &QuantumCircuit,
    circuit2: &QuantumCircuit,
    config1: &CompilerConfig,
    config2: &CompilerConfig,
) -> String {
    format!(
        r#"# Trotter Order Comparison

## Hamiltonian

{}

## Comparison Table

| Property | First-Order | Second-Order |
|----------|-------------|--------------|
| Gates | {} | {} |
| Trotter steps | {} | {} |
| Error scaling | O(t²/n) | O(t³/n²) |
| Estimated error | O(0.2) | O(0.04) |

## Trade-offs

- **First-order**: Fewer gates, lower accuracy
- **Second-order**: More gates, higher accuracy
- For the same error, second-order requires fewer steps

## Recommendation

For production use, second-order Trotter is preferred when:
- High fidelity is required
- Gate count is not the primary constraint
- Evolution time is not too large
"#,
        h.to_markdown(),
        circuit1.size(),
        circuit2.size(),
        config1.trotter_steps,
        config2.trotter_steps
    )
}
