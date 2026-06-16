//! Backward Conversion Demo: Quantum Circuit to Hamiltonian
//!
//! Author: gA4ss
//!
//! This example demonstrates the extraction of Hamiltonians from quantum circuits.
//! It analyzes circuit patterns and exports results in LaTeX and Markdown formats.

use myquat::hamiltonian::*;
use myquat::*;
use std::fs;

fn main() -> Result<()> {
    println!("========================================");
    println!("  Backward Conversion: Circuit -> H");
    println!("========================================\n");

    // Create output directory
    fs::create_dir_all("output")?;

    // Example 1: Simple rotation circuit
    example_rotation_circuit()?;

    // Example 2: Two-qubit entangling circuit
    example_entangling_circuit()?;

    // Example 3: Trotter pattern recognition
    example_trotter_pattern()?;

    // Example 4: Complex multi-qubit circuit
    example_complex_circuit()?;

    // Example 5: VQE-style parametric circuit
    example_parametric_circuit()?;

    println!("\n========================================");
    println!("  All examples completed!");
    println!("  Check output/ directory for LaTeX and Markdown files");
    println!("========================================");

    Ok(())
}

/// Example 1: Simple rotation circuit
fn example_rotation_circuit() -> Result<()> {
    println!("Example 1: Simple Rotation Circuit");
    println!("-----------------------------------\n");

    // Create a circuit with single-qubit rotations
    let mut circuit = QuantumCircuit::new(2, 0);
    circuit.rx(0, Parameter::Float(0.8))?;
    circuit.ry(1, Parameter::Float(0.6))?;
    circuit.rz(0, Parameter::Float(0.4))?;

    println!("Original Circuit:");
    println!("{}", CircuitVisualizer::to_ascii_art(&circuit));
    println!("Gates: {}\n", circuit.size());

    // Analyze the circuit
    let analyzer = CircuitAnalyzer::new();
    let analysis = analyzer.analyze(&circuit)?;

    println!("Extracted Hamiltonian:");
    println!("{}\n", analysis.hamiltonian);
    println!("Number of terms: {}", analysis.hamiltonian.num_terms());
    println!("Is Hermitian: {}", analysis.hamiltonian.is_hermitian());

    // Generate documents
    let latex = generate_latex_rotation(&circuit, &analysis);
    fs::write("output/backward1_rotation.tex", latex)?;

    let markdown = generate_markdown_rotation(&circuit, &analysis);
    fs::write("output/backward1_rotation.md", markdown)?;

    println!("\nOutput files:");
    println!("  - output/backward1_rotation.tex");
    println!("  - output/backward1_rotation.md\n");

    Ok(())
}

/// Example 2: Two-qubit entangling circuit
fn example_entangling_circuit() -> Result<()> {
    println!("Example 2: Two-Qubit Entangling Circuit");
    println!("---------------------------------------\n");

    // Create a circuit with two-qubit gates
    let mut circuit = QuantumCircuit::new(2, 0);
    circuit.cnot(0, 1)?;
    circuit.cz(0, 1)?;
    circuit.rx(0, Parameter::Float(0.5))?;
    circuit.ry(1, Parameter::Float(0.3))?;
    circuit.cnot(0, 1)?;

    println!("Original Circuit:");
    println!("{}", CircuitVisualizer::to_ascii_art(&circuit));
    println!("Gates: {}\n", circuit.size());

    // Analyze
    let analyzer = CircuitAnalyzer::new();
    let analysis = analyzer.analyze(&circuit)?;

    println!("Extracted Hamiltonian:");
    println!("{}\n", analysis.hamiltonian);
    println!("Number of terms: {}", analysis.hamiltonian.num_terms());

    // Generate documents
    let latex = generate_latex_entangling(&circuit, &analysis);
    fs::write("output/backward2_entangling.tex", latex)?;

    let markdown = generate_markdown_entangling(&circuit, &analysis);
    fs::write("output/backward2_entangling.md", markdown)?;

    println!("\nOutput files:");
    println!("  - output/backward2_entangling.tex");
    println!("  - output/backward2_entangling.md\n");

    Ok(())
}

/// Example 3: Trotter pattern recognition
fn example_trotter_pattern() -> Result<()> {
    println!("Example 3: Trotter Pattern Recognition");
    println!("--------------------------------------\n");

    // Create a circuit that looks like Trotter decomposition
    let mut circuit = QuantumCircuit::new(2, 0);

    // Repeat the pattern 3 times (simulating Trotter steps)
    for _ in 0..3 {
        circuit.rx(0, Parameter::Float(0.2))?;
        circuit.ry(1, Parameter::Float(0.15))?;
        circuit.cnot(0, 1)?;
        circuit.rz(1, Parameter::Float(0.1))?;
        circuit.cnot(0, 1)?;
    }

    println!("Original Circuit (with repeated pattern):");
    println!("Gates: {}\n", circuit.size());

    // Analyze
    let analyzer = CircuitAnalyzer::new();
    let analysis = analyzer.analyze(&circuit)?;

    println!("Extracted Hamiltonian:");
    println!("{}\n", analysis.hamiltonian);

    if let Some(steps) = analysis.trotter_steps {
        println!("Detected Trotter steps: {}", steps);
        println!("This suggests the circuit implements time evolution");
        println!("with {} repeated applications of the Hamiltonian.\n", steps);
    }

    // Generate documents
    let latex = generate_latex_trotter(&circuit, &analysis);
    fs::write("output/backward3_trotter.tex", latex)?;

    let markdown = generate_markdown_trotter(&circuit, &analysis);
    fs::write("output/backward3_trotter.md", markdown)?;

    println!("Output files:");
    println!("  - output/backward3_trotter.tex");
    println!("  - output/backward3_trotter.md\n");

    Ok(())
}

/// Example 4: Complex multi-qubit circuit
fn example_complex_circuit() -> Result<()> {
    println!("Example 4: Complex Multi-Qubit Circuit");
    println!("--------------------------------------\n");

    // Create a 3-qubit circuit
    let mut circuit = QuantumCircuit::new(3, 0);

    // Mix of single and two-qubit gates
    circuit.rx(0, Parameter::Float(0.5))?;
    circuit.ry(1, Parameter::Float(0.3))?;
    circuit.rz(2, Parameter::Float(0.4))?;
    circuit.cnot(0, 1)?;
    circuit.rz(1, Parameter::Float(0.2))?;
    circuit.cnot(0, 1)?;
    circuit.cnot(1, 2)?;
    circuit.rx(0, Parameter::Float(0.1))?;

    println!("Original Circuit:");
    println!("Gates: {}\n", circuit.size());

    // Analyze
    let analyzer = CircuitAnalyzer::new();
    let analysis = analyzer.analyze(&circuit)?;

    println!("Extracted Hamiltonian:");
    println!("{}\n", analysis.hamiltonian);
    println!("Number of terms: {}", analysis.hamiltonian.num_terms());
    println!("Qubits: {}", analysis.hamiltonian.num_qubits);

    // Term breakdown
    println!("\nTerm breakdown:");
    for (i, term) in analysis.hamiltonian.terms.iter().enumerate() {
        println!(
            "  Term {}: {} * {}",
            i,
            term.coefficient,
            term.pauli_string.to_string_repr()
        );
    }

    // Generate documents
    let latex = generate_latex_complex(&circuit, &analysis);
    fs::write("output/backward4_complex.tex", latex)?;

    let markdown = generate_markdown_complex(&circuit, &analysis);
    fs::write("output/backward4_complex.md", markdown)?;

    println!("\nOutput files:");
    println!("  - output/backward4_complex.tex");
    println!("  - output/backward4_complex.md\n");

    Ok(())
}

/// Example 5: VQE-style parametric circuit
fn example_parametric_circuit() -> Result<()> {
    println!("Example 5: VQE-Style Parametric Circuit");
    println!("---------------------------------------\n");

    // Create a parametric circuit (common in VQE)
    // Note: Using numeric values since symbolic parameters are not yet supported in analysis
    let mut circuit = QuantumCircuit::new(2, 0);

    // Use numeric parameters (in real VQE these would be optimized)
    circuit.rx(0, Parameter::Float(0.8))?; // theta
    circuit.ry(1, Parameter::Float(0.6))?; // phi
    circuit.cnot(0, 1)?;
    circuit.rz(1, Parameter::Float(0.5))?;
    circuit.cnot(0, 1)?;
    circuit.rz(0, Parameter::Float(0.4))?; // gamma

    println!("Original Circuit (VQE-style ansatz):");
    println!("{}", CircuitVisualizer::to_ascii_art(&circuit));
    println!("Gates: {}\n", circuit.size());

    // Analyze
    let analyzer = CircuitAnalyzer::new();
    let analysis = analyzer.analyze(&circuit)?;

    println!("Extracted Hamiltonian:");
    println!("{}\n", analysis.hamiltonian);

    println!("Note: In actual VQE applications, these gate angles would be");
    println!("symbolic parameters (theta, phi, gamma) to be optimized.");
    println!("Symbolic parameter support is planned for future extensions.\n");

    // Generate documents
    let latex = generate_latex_parametric(&circuit, &analysis);
    fs::write("output/backward5_parametric.tex", latex)?;

    let markdown = generate_markdown_parametric(&circuit, &analysis);
    fs::write("output/backward5_parametric.md", markdown)?;

    println!("Output files:");
    println!("  - output/backward5_parametric.tex");
    println!("  - output/backward5_parametric.md\n");

    Ok(())
}

// LaTeX generation functions

fn generate_latex_rotation(circuit: &QuantumCircuit, analysis: &CircuitAnalysis) -> String {
    format!(
        r#"\documentclass{{article}}
\usepackage{{amsmath}}
\usepackage{{physics}}

\title{{Circuit Analysis: Single-Qubit Rotations}}
\author{{MyQuat Framework}}

\begin{{document}}

\maketitle

\section{{Original Circuit}}

The input quantum circuit contains {} gates on {} qubits:
\begin{{itemize}}
\item RX gate on qubit 0 with angle 0.8
\item RY gate on qubit 1 with angle 0.6
\item RZ gate on qubit 0 with angle 0.4
\end{{itemize}}

\section{{Extracted Hamiltonian}}

{}

\section{{Interpretation}}

Each rotation gate $R_\mu(\theta)$ corresponds to the Hamiltonian term:
$$R_\mu(\theta) = e^{{-i\theta\sigma_\mu/2}}$$

Therefore, the effective Hamiltonian is:
$$\hat{{H}}_{{eff}} = \frac{{0.8}}{{2}} \sigma_x^{{(0)}} + \frac{{0.6}}{{2}} \sigma_y^{{(1)}} + \frac{{0.4}}{{2}} \sigma_z^{{(0)}}$$

\section{{Physical Meaning}}

This Hamiltonian represents:
\begin{{itemize}}
\item Independent single-qubit operations
\item No entanglement between qubits
\item Can be implemented in parallel
\end{{itemize}}

\section{{Circuit Statistics}}

\begin{{tabular}}{{ll}}
Total gates & {} \\
Number of terms & {} \\
Hermitian & {} \\
\end{{tabular}}

\end{{document}}
"#,
        circuit.size(),
        circuit.num_qubits(),
        analysis.hamiltonian.to_latex(),
        circuit.size(),
        analysis.hamiltonian.num_terms(),
        if analysis.hamiltonian.is_hermitian() {
            "Yes"
        } else {
            "No"
        }
    )
}

fn generate_latex_entangling(circuit: &QuantumCircuit, analysis: &CircuitAnalysis) -> String {
    format!(
        r#"\documentclass{{article}}
\usepackage{{amsmath}}
\usepackage{{physics}}

\title{{Circuit Analysis: Two-Qubit Entangling Gates}}
\author{{MyQuat Framework}}

\begin{{document}}

\maketitle

\section{{Original Circuit}}

The circuit contains {} two-qubit entangling gates:
\begin{{itemize}}
\item RXX: Ising XX coupling
\item RYY: Ising YY coupling
\item RZZ: Ising ZZ coupling
\end{{itemize}}

\section{{Extracted Hamiltonian}}

{}

\section{{Gate-to-Hamiltonian Mapping}}

The two-qubit rotation gates map to Hamiltonians:
\begin{{align}}
\text{{RXX}}(\theta) &= e^{{-i\theta \sigma_x \otimes \sigma_x / 2}} \\
\text{{RYY}}(\theta) &= e^{{-i\theta \sigma_y \otimes \sigma_y / 2}} \\
\text{{RZZ}}(\theta) &= e^{{-i\theta \sigma_z \otimes \sigma_z / 2}}
\end{{align}}

\section{{Physical Interpretation}}

These gates create entanglement through:
\begin{{itemize}}
\item XX coupling: Exchange interaction
\item YY coupling: Imaginary exchange
\item ZZ coupling: Diagonal interaction (no state transfer)
\end{{itemize}}

\section{{Applications}}

This type of Hamiltonian appears in:
\begin{{itemize}}
\item Quantum simulation of spin systems
\item Variational quantum algorithms (VQE, QAOA)
\item Quantum approximate optimization
\end{{itemize}}

\end{{document}}
"#,
        circuit.size(),
        analysis.hamiltonian.to_latex()
    )
}

fn generate_latex_trotter(circuit: &QuantumCircuit, analysis: &CircuitAnalysis) -> String {
    let trotter_info = if let Some(steps) = analysis.trotter_steps {
        format!(
            "The analyzer detected {} Trotter steps, suggesting this circuit implements:
$$U(t) \\approx \\left[e^{{-i\\hat{{H}} \\Delta t}}\\right]^{{{}}}$$

where $\\Delta t = t/n$ is the time step.",
            steps, steps
        )
    } else {
        "No Trotter pattern was detected.".to_string()
    };

    format!(
        r#"\documentclass{{article}}
\usepackage{{amsmath}}
\usepackage{{physics}}

\title{{Circuit Analysis: Trotter Pattern Recognition}}
\author{{MyQuat Framework}}

\begin{{document}}

\maketitle

\section{{Original Circuit}}

The circuit contains {} gates with a repeating pattern structure.

\section{{Pattern Recognition}}

{}

\section{{Extracted Hamiltonian}}

{}

The total evolution time can be inferred from the Trotter decomposition structure.

\section{{Reverse Engineering}}

By analyzing the circuit structure, we can:
\begin{{enumerate}}
\item Identify the base Hamiltonian
\item Detect the number of Trotter steps
\item Estimate the evolution time
\item Reconstruct the original simulation intent
\end{{enumerate}}

\section{{Applications}}

Trotter pattern recognition is useful for:
\begin{{itemize}}
\item Circuit optimization and compression
\item Understanding third-party quantum algorithms
\item Verifying Hamiltonian simulation implementations
\end{{itemize}}

\end{{document}}
"#,
        circuit.size(),
        trotter_info,
        analysis.hamiltonian.to_latex()
    )
}

fn generate_latex_complex(circuit: &QuantumCircuit, analysis: &CircuitAnalysis) -> String {
    let mut term_list = String::new();
    for (i, term) in analysis.hamiltonian.terms.iter().enumerate() {
        term_list.push_str(&format!(
            "\\item Term {}: ${}$, Pauli string: ${:?}$\n",
            i,
            term.coefficient.re,
            term.pauli_string.to_string_repr()
        ));
    }

    format!(
        r#"\documentclass{{article}}
\usepackage{{amsmath}}
\usepackage{{physics}}

\title{{Circuit Analysis: Complex Multi-Qubit Circuit}}
\author{{MyQuat Framework}}

\begin{{document}}

\maketitle

\section{{Original Circuit}}

A {}-qubit circuit with {} gates combining single-qubit and two-qubit operations.

\section{{Extracted Hamiltonian}}

{}

\section{{Term Breakdown}}

The Hamiltonian contains {} terms:
\begin{{itemize}}
{}
\end{{itemize}}

\section{{Analysis}}

This complex Hamiltonian shows:
\begin{{itemize}}
\item Mix of local and non-local terms
\item Both diagonal and off-diagonal interactions
\item Potential for rich quantum dynamics
\end{{itemize}}

\section{{Complexity Metrics}}

\begin{{tabular}}{{ll}}
Qubits & {} \\
Terms & {} \\
Gates & {} \\
Average term weight & {:.3} \\
\end{{tabular}}

\end{{document}}
"#,
        circuit.num_qubits(),
        circuit.size(),
        analysis.hamiltonian.to_latex(),
        analysis.hamiltonian.num_terms(),
        term_list,
        analysis.hamiltonian.num_qubits,
        analysis.hamiltonian.num_terms(),
        circuit.size(),
        analysis
            .hamiltonian
            .terms
            .iter()
            .map(|t| t.coefficient.norm())
            .sum::<f64>()
            / analysis.hamiltonian.num_terms() as f64
    )
}

fn generate_latex_parametric(circuit: &QuantumCircuit, analysis: &CircuitAnalysis) -> String {
    format!(
        r#"\documentclass{{article}}
\usepackage{{amsmath}}
\usepackage{{physics}}

\title{{Circuit Analysis: Parametric VQE-Style Circuit}}
\author{{MyQuat Framework}}

\begin{{document}}

\maketitle

\section{{Original Circuit}}

A parametric quantum circuit with symbolic parameters $\theta$, $\phi$, and $\gamma$.
This is typical of Variational Quantum Eigensolver (VQE) ans\"atze.

\section{{Extracted Hamiltonian}}

{}

\section{{VQE Context}}

In VQE applications, this Hamiltonian would be:
\begin{{enumerate}}
\item Implemented as a parametric circuit
\item Used to prepare trial wavefunctions
\item Optimized to minimize energy: $E(\theta, \phi, \gamma) = \langle \psi(\theta, \phi, \gamma) | \hat{{H}}_{{target}} | \psi(\theta, \phi, \gamma) \rangle$
\end{{enumerate}}

\section{{Parameter Optimization}}

The symbolic parameters can be optimized using:
\begin{{itemize}}
\item Gradient descent (parameter-shift rule)
\item Natural gradient optimization
\item COBYLA or other gradient-free methods
\end{{itemize}}

\section{{Note on Symbolic Parameters}}

Symbolic parameters in the circuit are currently treated as unit coefficients.
For full VQE integration, consider using the symbolic computation features
in the future extensions.

\end{{document}}
"#,
        analysis.hamiltonian.to_latex()
    )
}

// Markdown generation functions

fn generate_markdown_rotation(circuit: &QuantumCircuit, analysis: &CircuitAnalysis) -> String {
    format!(
        r#"# Circuit Analysis: Single-Qubit Rotations

## Original Circuit

The input quantum circuit contains {} gates on {} qubits:
- RX gate on qubit 0 with angle 0.8
- RY gate on qubit 1 with angle 0.6
- RZ gate on qubit 0 with angle 0.4

## Extracted Hamiltonian

{}

## Interpretation

Each rotation gate R_μ(θ) corresponds to the Hamiltonian term:

$$R_\mu(\theta) = e^{{-i\theta\sigma_\mu/2}}$$

Therefore, the effective Hamiltonian is:

$$\hat{{H}}_{{eff}} = \frac{{0.8}}{{2}} \sigma_x^{{(0)}} + \frac{{0.6}}{{2}} \sigma_y^{{(1)}} + \frac{{0.4}}{{2}} \sigma_z^{{(0)}}$$

## Physical Meaning

This Hamiltonian represents:
- Independent single-qubit operations
- No entanglement between qubits
- Can be implemented in parallel

## Circuit Statistics

- **Total gates**: {}
- **Number of terms**: {}
- **Hermitian**: {}
"#,
        circuit.size(),
        circuit.num_qubits(),
        analysis.hamiltonian.to_markdown(),
        circuit.size(),
        analysis.hamiltonian.num_terms(),
        if analysis.hamiltonian.is_hermitian() {
            "Yes"
        } else {
            "No"
        }
    )
}

fn generate_markdown_entangling(circuit: &QuantumCircuit, analysis: &CircuitAnalysis) -> String {
    format!(
        r#"# Circuit Analysis: Two-Qubit Entangling Gates

## Original Circuit

The circuit contains {} two-qubit entangling gates:
- **RXX**: Ising XX coupling
- **RYY**: Ising YY coupling
- **RZZ**: Ising ZZ coupling

## Extracted Hamiltonian

{}

## Gate-to-Hamiltonian Mapping

The two-qubit rotation gates map to Hamiltonians:

$$\text{{RXX}}(\theta) = e^{{-i\theta \sigma_x \otimes \sigma_x / 2}}$$

$$\text{{RYY}}(\theta) = e^{{-i\theta \sigma_y \otimes \sigma_y / 2}}$$

$$\text{{RZZ}}(\theta) = e^{{-i\theta \sigma_z \otimes \sigma_z / 2}}$$

## Applications

This type of Hamiltonian appears in:
- Quantum simulation of spin systems
- Variational quantum algorithms (VQE, QAOA)
- Quantum approximate optimization
"#,
        circuit.size(),
        analysis.hamiltonian.to_markdown()
    )
}

fn generate_markdown_trotter(circuit: &QuantumCircuit, analysis: &CircuitAnalysis) -> String {
    let trotter_info = if let Some(steps) = analysis.trotter_steps {
        format!(
            "The analyzer detected **{} Trotter steps**, suggesting this circuit implements:

$$U(t) \\approx \\left[e^{{-i\\hat{{H}} \\Delta t}}\\right]^{{{}}}$$

where Δt = t/n is the time step.",
            steps, steps
        )
    } else {
        "No Trotter pattern was detected.".to_string()
    };

    format!(
        r#"# Circuit Analysis: Trotter Pattern Recognition

## Original Circuit

The circuit contains {} gates with a repeating pattern structure.

## Pattern Recognition

{}

## Extracted Hamiltonian

{}

## Reverse Engineering

By analyzing the circuit structure, we can:
1. Identify the base Hamiltonian
2. Detect the number of Trotter steps
3. Estimate the evolution time
4. Reconstruct the original simulation intent
"#,
        circuit.size(),
        trotter_info,
        analysis.hamiltonian.to_markdown()
    )
}

fn generate_markdown_complex(circuit: &QuantumCircuit, analysis: &CircuitAnalysis) -> String {
    let mut term_list = String::new();
    for (i, term) in analysis.hamiltonian.terms.iter().enumerate() {
        term_list.push_str(&format!(
            "- **Term {}**: Coefficient {:.3}, Pauli string: {}\n",
            i,
            term.coefficient.re,
            term.pauli_string.to_string_repr()
        ));
    }

    format!(
        r#"# Circuit Analysis: Complex Multi-Qubit Circuit

## Original Circuit

A {}-qubit circuit with {} gates combining single-qubit and two-qubit operations.

## Extracted Hamiltonian

{}

## Term Breakdown

The Hamiltonian contains {} terms:

{}

## Complexity Metrics

| Metric | Value |
|--------|-------|
| Qubits | {} |
| Terms | {} |
| Gates | {} |
| Average term weight | {:.3} |
"#,
        circuit.num_qubits(),
        circuit.size(),
        analysis.hamiltonian.to_markdown(),
        analysis.hamiltonian.num_terms(),
        term_list,
        analysis.hamiltonian.num_qubits,
        analysis.hamiltonian.num_terms(),
        circuit.size(),
        analysis
            .hamiltonian
            .terms
            .iter()
            .map(|t| t.coefficient.norm())
            .sum::<f64>()
            / analysis.hamiltonian.num_terms() as f64
    )
}

fn generate_markdown_parametric(circuit: &QuantumCircuit, analysis: &CircuitAnalysis) -> String {
    format!(
        r#"# Circuit Analysis: Parametric VQE-Style Circuit

## Original Circuit

A parametric quantum circuit with symbolic parameters θ, φ, and γ.
This is typical of Variational Quantum Eigensolver (VQE) ansätze.

## Extracted Hamiltonian

{}

## VQE Context

In VQE applications, this Hamiltonian would be:
1. Implemented as a parametric circuit
2. Used to prepare trial wavefunctions
3. Optimized to minimize energy

## Parameter Optimization

The symbolic parameters can be optimized using:
- Gradient descent (parameter-shift rule)
- Natural gradient optimization
- COBYLA or other gradient-free methods

## Note

Symbolic parameters in the circuit are currently treated as unit coefficients.
For full VQE integration, consider using the symbolic computation features
in the future extensions.
"#,
        analysis.hamiltonian.to_markdown()
    )
}
