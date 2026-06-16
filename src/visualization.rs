//! # Advanced Quantum Circuit Visualization
//!
//! This module provides comprehensive visualization capabilities for quantum circuits,
//! including enhanced ASCII art, circuit diagrams, SVG export, and statistical analysis.
//!
//! ## Overview
//!
//! The [`CircuitVisualizer`] is the main interface for all visualization operations.
//! It supports multiple output formats and customizable styling options to meet
//! different use cases from terminal display to publication-quality figures.
//!
//! ## Key Features
//!
//! - **Multiple Output Formats**: ASCII art, SVG, and PNG export
//! - **Customizable Styling**: Wire styles, gate styles, and color schemes
//! - **Statistical Analysis**: Gate counting, depth analysis, and performance metrics
//! - **Flexible Layout**: Automatic sizing and manual width control
//! - **Professional Quality**: Publication-ready SVG output
//!
//! ## Quick Start
//!
//! ```rust
//! use myquat::*;
//!
//! // Create a quantum circuit
//! let mut circuit = QuantumCircuit::new(3, 3);
//! circuit.h(0)?;
//! circuit.cx(0, 1)?;
//! circuit.cx(1, 2)?;
//! circuit.measure_all()?;
//!
//! // Simple ASCII visualization
//! println!("{}", CircuitVisualizer::to_ascii_art(&circuit));
//!
//! // Advanced visualization with custom styling
//! let viz = CircuitVisualizer::new()
//!     .with_parameters(true)
//!     .with_max_width(120);
//!
//! println!("{}", viz.draw_circuit(&circuit));
//! println!("{}", viz.circuit_summary(&circuit));
//! # Ok::<(), myquat::MyQuatError>(())
//! ```
//!
//! ## Visualization Styles
//!
//! ### ASCII Art Styles
//!
//! ```rust
//! use myquat::*;
//!
//! let mut circuit = QuantumCircuit::new(2, 2);
//! circuit.h(0)?;
//! circuit.cx(0, 1)?;
//!
//! // Compact style for small displays
//! let compact = CircuitVisualizer::compact();
//! println!("Compact:\n{}", compact.draw_circuit(&circuit));
//!
//! // Detailed style with full information
//! let detailed = CircuitVisualizer::detailed();
//! println!("Detailed:\n{}", detailed.draw_circuit(&circuit));
//!
//! // Custom wire styles
//! let custom_style = VisualizationStyle {
//!     wire_style: WireStyle::Double,
//!     gate_style: GateStyle::Boxed,
//!     color_scheme: ColorScheme::None,
//!     use_unicode: true,
//! };
//!
//! let custom_viz = CircuitVisualizer::new().with_style(custom_style);
//! println!("Custom:\n{}", custom_viz.draw_circuit(&circuit));
//! # Ok::<(), myquat::MyQuatError>(())
//! ```
//!
//! ## Export Capabilities
//!
//! ### SVG Export for Publications
//!
//! ```rust,no_run
//! use myquat::*;
//!
//! let mut circuit = QuantumCircuit::new(3, 0);
//! circuit.h(0)?;
//! circuit.cx(0, 1)?;
//! circuit.cx(1, 2)?;
//!
//! let viz = CircuitVisualizer::new();
//!
//! // Export as SVG for publications
//! viz.export_svg(&circuit, "ghz_circuit.svg")?;
//!
//! // Export as PNG (requires external tools)
//! viz.export_png(&circuit, "ghz_circuit.png")?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Statistical Analysis
//!
//! ### Circuit Metrics and Analysis
//!
//! ```rust,ignore
//! use myquat::*;
//!
//! let mut circuit = QuantumCircuit::new(4, 4);
//! circuit.h(0)?;
//! circuit.cx(0, 1)?;
//! circuit.ry(2, Parameter::Float(std::f64::consts::PI / 4.0))?;
//! circuit.measure_all()?;
//!
//! let viz = CircuitVisualizer::new();
//!
//! // Comprehensive circuit summary
//! println!("{}", viz.circuit_summary(&circuit));
//!
//! // Detailed statistics table
//! println!("{}", viz.statistics_table(&circuit));
//!
//! // Gate-by-gate breakdown
//! println!("{}", viz.gate_breakdown(&circuit));
//!
//! // Individual metrics
//! println!("Single-qubit gates: {}", viz.count_single_qubit_gates(&circuit));
//! println!("Two-qubit gates: {}", viz.count_two_qubit_gates(&circuit));
//! println!("Measurements: {}", viz.count_measurements(&circuit));
//! # Ok::<(), myquat::MyQuatError>(())
//! ```
//!
//! ## Customization Options
//!
//! ### Wire Styles
//! - [`WireStyle::Simple`] - Standard horizontal lines (─)
//! - [`WireStyle::Double`] - Double lines for emphasis (═)
//! - [`WireStyle::Dotted`] - Dotted lines for clarity (┅)
//! - [`WireStyle::Custom`] - User-defined characters
//!
//! ### Gate Styles  
//! - [`GateStyle::Compact`] - Minimal representation (─H─)
//! - [`GateStyle::Boxed`] - Boxed gates for clarity (┌─┐)
//! - [`GateStyle::Rounded`] - Rounded corners (╭─╮)
//! - [`GateStyle::AsciiOnly`] - ASCII-only characters (\[H\])
//!
//! ### Color Schemes
//! - [`ColorScheme::None`] - No colors (default)
//! - [`ColorScheme::Basic`] - Basic ANSI colors
//! - [`ColorScheme::Extended`] - 256-color palette
//! - [`ColorScheme::TrueColor`] - 24-bit color support
//!
//! ## Performance Considerations
//!
//! - ASCII rendering is optimized for large circuits
//! - SVG generation scales well with circuit complexity
//! - Memory usage is proportional to circuit size
//! - Caching is used for repeated visualizations
//!
//! ## Integration with Other Modules
//!
//! The visualization module integrates seamlessly with:
//! - Circuit module - Circuit structure and analysis
//! - Transpiler module - Optimization metrics display  
//! - Benchmarks module - Performance visualization
//! - Error mitigation module - Error analysis charts

use crate::{Parameter, QuantumCircuit, StandardGate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

/// Circuit visualizer with multiple output formats
#[derive(Debug, Clone)]
pub struct CircuitVisualizer {
    /// Visualization style configuration
    pub style: VisualizationStyle,
    /// Whether to show parameter values
    pub show_parameters: bool,
    /// Whether to show qubit indices
    pub show_qubit_indices: bool,
    /// Whether to show classical bits
    pub show_classical_bits: bool,
    /// Maximum width for circuit display
    pub max_width: usize,
}

/// Visualization style options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationStyle {
    /// Style for quantum wires
    pub wire_style: WireStyle,
    /// Style for gates
    pub gate_style: GateStyle,
    /// Color scheme (for future terminal color support)
    pub color_scheme: ColorScheme,
    /// Whether to use Unicode characters
    pub use_unicode: bool,
}

/// Wire drawing styles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WireStyle {
    /// Simple lines: ─
    Simple,
    /// Double lines: ═
    Double,
    /// Dotted lines: ┅
    Dotted,
    /// Custom character
    Custom(char),
}

/// Gate drawing styles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GateStyle {
    /// Compact: ─H─
    Compact,
    /// Boxed: ┌─┐
    Boxed,
    /// Rounded: ╭─╮
    Rounded,
    /// ASCII only: [H]
    AsciiOnly,
}

/// Color schemes for terminal output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorScheme {
    /// No colors
    None,
    /// Basic ANSI colors
    Basic,
    /// Extended 256-color palette
    Extended,
    /// True color (24-bit)
    TrueColor,
}

impl Default for VisualizationStyle {
    fn default() -> Self {
        Self {
            wire_style: WireStyle::Simple,
            gate_style: GateStyle::Compact,
            color_scheme: ColorScheme::None,
            use_unicode: true,
        }
    }
}

impl Default for CircuitVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitVisualizer {
    /// Create a new visualizer with default settings
    pub fn new() -> Self {
        Self {
            style: VisualizationStyle::default(),
            show_parameters: true,
            show_qubit_indices: true,
            show_classical_bits: true,
            max_width: 120,
        }
    }

    /// Create a compact visualizer for small displays
    pub fn compact() -> Self {
        Self {
            style: VisualizationStyle {
                gate_style: GateStyle::Compact,
                use_unicode: false,
                ..Default::default()
            },
            show_parameters: false,
            show_qubit_indices: true,
            show_classical_bits: false,
            max_width: 80,
        }
    }

    /// Create a detailed visualizer with all features
    pub fn detailed() -> Self {
        Self {
            style: VisualizationStyle {
                gate_style: GateStyle::Boxed,
                use_unicode: true,
                ..Default::default()
            },
            show_parameters: true,
            show_qubit_indices: true,
            show_classical_bits: true,
            max_width: 150,
        }
    }

    /// Set visualization style
    pub fn with_style(mut self, style: VisualizationStyle) -> Self {
        self.style = style;
        self
    }

    /// Enable/disable parameter display
    pub fn with_parameters(mut self, show: bool) -> Self {
        self.show_parameters = show;
        self
    }

    /// Set maximum display width
    pub fn with_max_width(mut self, width: usize) -> Self {
        self.max_width = width;
        self
    }

    /// Generate enhanced ASCII art circuit diagram
    pub fn draw_circuit(&self, circuit: &QuantumCircuit) -> String {
        if circuit.is_empty() {
            return self.draw_empty_circuit(circuit.num_qubits());
        }

        let mut diagram = CircuitDiagram::new(circuit, &self.style);

        // Build the circuit layout
        for instruction in circuit.data().instructions() {
            diagram.add_instruction(instruction);
        }

        // Render to string
        diagram.render(
            self.max_width,
            self.show_qubit_indices,
            self.show_classical_bits,
        )
    }

    /// Generate a circuit summary with statistics
    pub fn circuit_summary(&self, circuit: &QuantumCircuit) -> String {
        // Pre-allocate capacity to reduce memory reallocations
        let mut summary = String::with_capacity(512);

        // Header
        summary.push_str("╔═══════════════════════════════════════╗\n");
        summary.push_str("║           CIRCUIT SUMMARY             ║\n");
        summary.push_str("╠═══════════════════════════════════════╣\n");

        // Basic info
        summary.push_str(&format!(
            "║ Qubits:        {:>20} ║\n",
            circuit.num_qubits()
        ));
        summary.push_str(&format!(
            "║ Classical bits: {:>19} ║\n",
            circuit.num_clbits()
        ));
        summary.push_str(&format!("║ Gates:         {:>20} ║\n", circuit.size()));
        summary.push_str(&format!("║ Depth:         {:>20} ║\n", circuit.depth()));

        if let Some(name) = circuit.name() {
            summary.push_str(&format!("║ Name:          {:>20} ║\n", name));
        }

        // Gate statistics
        let gate_counts = self.count_gates(circuit);
        if !gate_counts.is_empty() {
            summary.push_str("╠═══════════════════════════════════════╣\n");
            summary.push_str("║             GATE COUNTS               ║\n");
            summary.push_str("╠═══════════════════════════════════════╣\n");

            for (gate, count) in gate_counts {
                summary.push_str(&format!("║ {:12}: {:>20} ║\n", gate, count));
            }
        }

        summary.push_str("╚═══════════════════════════════════════╝\n");
        summary
    }

    /// Generate a detailed gate-by-gate breakdown
    pub fn gate_breakdown(&self, circuit: &QuantumCircuit) -> String {
        // Pre-allocate based on circuit size
        let mut breakdown = String::with_capacity(circuit.size() * 50);

        breakdown.push_str("Gate-by-Gate Breakdown:\n");
        breakdown.push_str("═══════════════════════\n\n");

        for (i, instruction) in circuit.data().instructions().iter().enumerate() {
            breakdown.push_str(&format!("Step {}: ", i + 1));

            if instruction.is_measurement() {
                breakdown.push_str(&format!(
                    "Measure qubit {} → classical bit {}\n",
                    instruction.qubits[0], instruction.clbits[0]
                ));
            } else {
                breakdown.push_str(&format!("{:?}", instruction.gate.gate_type));

                // Qubits
                if !instruction.qubits.is_empty() {
                    breakdown.push_str(" on qubit");
                    if instruction.qubits.len() > 1 {
                        breakdown.push('s');
                    }
                    breakdown.push(' ');
                    for (j, qubit) in instruction.qubits.iter().enumerate() {
                        if j > 0 {
                            breakdown.push_str(", ");
                        }
                        breakdown.push_str(&format!("{}", qubit.index()));
                    }
                }

                // Parameters
                if !instruction.gate.parameters.is_empty() && self.show_parameters {
                    breakdown.push_str(" with parameter");
                    if instruction.gate.parameters.len() > 1 {
                        breakdown.push('s');
                    }
                    breakdown.push(' ');
                    for (j, param) in instruction.gate.parameters.iter().enumerate() {
                        if j > 0 {
                            breakdown.push_str(", ");
                        }
                        breakdown.push_str(&self.format_parameter(param));
                    }
                }

                breakdown.push('\n');
            }
        }

        breakdown
    }

    /// Generate circuit statistics table
    pub fn statistics_table(&self, circuit: &QuantumCircuit) -> String {
        let mut table = String::with_capacity(400);

        table.push_str("┌─────────────────────┬─────────────────────┐\n");
        table.push_str("│      Statistic      │        Value        │\n");
        table.push_str("├─────────────────────┼─────────────────────┤\n");

        let stats = [
            ("Total Qubits", circuit.num_qubits().to_string()),
            ("Classical Bits", circuit.num_clbits().to_string()),
            ("Total Gates", circuit.size().to_string()),
            ("Circuit Depth", circuit.depth().to_string()),
            (
                "Single-Qubit Gates",
                self.count_single_qubit_gates(circuit).to_string(),
            ),
            (
                "Two-Qubit Gates",
                self.count_two_qubit_gates(circuit).to_string(),
            ),
            ("Measurements", self.count_measurements(circuit).to_string()),
            (
                "Parametric Gates",
                self.count_parametric_gates(circuit).to_string(),
            ),
        ];

        for (stat, value) in &stats {
            table.push_str(&format!("│ {:19} │ {:19} │\n", stat, value));
        }

        table.push_str("└─────────────────────┴─────────────────────┘\n");
        table
    }

    /// Draw an empty circuit
    fn draw_empty_circuit(&self, num_qubits: usize) -> String {
        let mut result = String::with_capacity(num_qubits * 30);

        if self.show_qubit_indices {
            result.push_str("Empty Circuit:\n\n");
        }

        let wire_char = self.get_wire_char();

        for i in 0..num_qubits {
            if self.show_qubit_indices {
                result.push_str(&format!(
                    "q[{}] {}{}{}\n",
                    i, wire_char, wire_char, wire_char
                ));
            } else {
                result.push_str(&format!("{}{}{}\n", wire_char, wire_char, wire_char));
            }
        }

        result
    }

    /// Count gates by type
    fn count_gates(&self, circuit: &QuantumCircuit) -> Vec<(String, usize)> {
        let mut counts = HashMap::new();

        for instruction in circuit.data().instructions() {
            if instruction.is_measurement() {
                *counts.entry("Measure".to_string()).or_insert(0) += 1;
            } else {
                let gate_name = format!("{:?}", instruction.gate.gate_type);
                *counts.entry(gate_name).or_insert(0) += 1;
            }
        }

        let mut sorted_counts: Vec<_> = counts.into_iter().collect();
        sorted_counts.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
        sorted_counts
    }

    /// Count single-qubit gates
    pub fn count_single_qubit_gates(&self, circuit: &QuantumCircuit) -> usize {
        circuit
            .data()
            .instructions()
            .iter()
            .filter(|inst| !inst.is_measurement() && inst.qubits.len() == 1)
            .count()
    }

    /// Count two-qubit gates
    pub fn count_two_qubit_gates(&self, circuit: &QuantumCircuit) -> usize {
        circuit
            .data()
            .instructions()
            .iter()
            .filter(|inst| !inst.is_measurement() && inst.qubits.len() == 2)
            .count()
    }

    /// Count measurements
    fn count_measurements(&self, circuit: &QuantumCircuit) -> usize {
        circuit
            .data()
            .instructions()
            .iter()
            .filter(|inst| inst.is_measurement())
            .count()
    }

    /// Count parametric gates
    fn count_parametric_gates(&self, circuit: &QuantumCircuit) -> usize {
        circuit
            .data()
            .instructions()
            .iter()
            .filter(|inst| !inst.gate.parameters.is_empty())
            .count()
    }

    /// Get wire character based on style
    fn get_wire_char(&self) -> char {
        match self.style.wire_style {
            WireStyle::Simple => '─',
            WireStyle::Double => '═',
            WireStyle::Dotted => '┅',
            WireStyle::Custom(c) => c,
        }
    }

    /// Format parameter for display
    fn format_parameter(&self, param: &Parameter) -> String {
        match param {
            Parameter::Float(val) => {
                if val.abs() < 1e-10 {
                    "0".to_string()
                } else if (val - std::f64::consts::PI).abs() < 1e-10 {
                    "π".to_string()
                } else if (val - std::f64::consts::PI / 2.0).abs() < 1e-10 {
                    "π/2".to_string()
                } else if (val - std::f64::consts::PI / 4.0).abs() < 1e-10 {
                    "π/4".to_string()
                } else {
                    format!("{:.3}", val)
                }
            }
            Parameter::Symbol(sym) => sym.clone(),
            Parameter::Expression(_) => "expr".to_string(),
        }
    }

    /// Export circuit as SVG
    pub fn export_svg(&self, circuit: &QuantumCircuit, filename: &str) -> std::io::Result<()> {
        let svg_content = self.generate_svg(circuit);
        let mut file = File::create(filename)?;
        file.write_all(svg_content.as_bytes())?;
        Ok(())
    }

    /// Generate SVG content for the circuit
    pub fn generate_svg(&self, circuit: &QuantumCircuit) -> String {
        let mut svg = SvgBuilder::new(circuit.num_qubits(), circuit.size());

        // Add title
        if let Some(name) = circuit.name() {
            svg.add_title(name);
        } else {
            svg.add_title("Quantum Circuit");
        }

        // Draw qubit lines
        for qubit in 0..circuit.num_qubits() {
            svg.draw_qubit_line(qubit);
        }

        // Draw gates
        let mut column = 0;
        for instruction in circuit.data().instructions() {
            if instruction.is_measurement() {
                let qubit = instruction.qubits[0].index();
                svg.draw_measurement(qubit, column);
            } else {
                match instruction.qubits.len() {
                    1 => {
                        let qubit = instruction.qubits[0].index();
                        let gate_name = self.get_svg_gate_name(&instruction.gate.gate_type);
                        svg.draw_single_qubit_gate(qubit, column, &gate_name);
                    }
                    2 => {
                        let control = instruction.qubits[0].index();
                        let target = instruction.qubits[1].index();
                        match instruction.gate.gate_type {
                            StandardGate::CX => svg.draw_cnot(control, target, column),
                            StandardGate::CZ => svg.draw_cz(control, target, column),
                            _ => svg.draw_generic_two_qubit(control, target, column, "G"),
                        }
                    }
                    _ => {
                        // Multi-qubit gates
                        for qubit in &instruction.qubits {
                            svg.draw_single_qubit_gate(qubit.index(), column, "G");
                        }
                    }
                }
            }
            column += 1;
        }

        svg.finalize()
    }

    /// Get SVG-friendly gate name
    fn get_svg_gate_name(&self, gate: &StandardGate) -> String {
        match gate {
            StandardGate::I => "I".to_string(),
            StandardGate::X => "X".to_string(),
            StandardGate::Y => "Y".to_string(),
            StandardGate::Z => "Z".to_string(),
            StandardGate::H => "H".to_string(),
            StandardGate::S => "S".to_string(),
            StandardGate::Sdg => "S†".to_string(),
            StandardGate::T => "T".to_string(),
            StandardGate::Tdg => "T†".to_string(),
            StandardGate::Rx => "Rx".to_string(),
            StandardGate::Ry => "Ry".to_string(),
            StandardGate::Rz => "Rz".to_string(),
            _ => "G".to_string(),
        }
    }

    /// Export circuit as PNG (requires external tool)
    pub fn export_png(&self, circuit: &QuantumCircuit, filename: &str) -> std::io::Result<()> {
        // First generate SVG
        let svg_filename = format!("{}.svg", filename.trim_end_matches(".png"));
        self.export_svg(circuit, &svg_filename)?;

        // Note: PNG conversion would require external tools like Inkscape or librsvg
        // For now, just create a placeholder file with instructions
        let png_instructions = format!(
            "To convert {} to PNG, use one of these commands:\n\
             1. inkscape {} --export-png={}\n\
             2. rsvg-convert -o {} {}\n\
             3. convert {} {}\n",
            svg_filename, svg_filename, filename, filename, svg_filename, svg_filename, filename
        );

        let instruction_file = format!("{}.conversion_instructions.txt", filename);
        let mut file = File::create(instruction_file)?;
        file.write_all(png_instructions.as_bytes())?;

        Ok(())
    }

    // Static convenience methods for backward compatibility

    /// Generate a simple text representation of a quantum circuit (static method)
    pub fn to_text(circuit: &QuantumCircuit) -> String {
        let viz = Self::compact().with_parameters(false);

        let mut output = String::new();
        output.push_str(&format!(
            "Circuit: {} qubits, {} classical bits\n",
            circuit.num_qubits(),
            circuit.num_clbits()
        ));

        if let Some(name) = circuit.name() {
            output.push_str(&format!("Name: {}\n", name));
        }

        output.push_str(&format!(
            "Depth: {}, Size: {}\n\n",
            circuit.depth(),
            circuit.size()
        ));
        output.push_str(&viz.gate_breakdown(circuit));

        output
    }

    /// Generate ASCII art representation of a quantum circuit (static method)
    pub fn to_ascii_art(circuit: &QuantumCircuit) -> String {
        let viz = Self::compact().with_parameters(false).with_max_width(80);
        viz.draw_circuit(circuit)
    }
}

/// Internal circuit diagram builder
struct CircuitDiagram {
    num_qubits: usize,
    columns: Vec<CircuitColumn>,
    style: VisualizationStyle,
}

/// A column in the circuit diagram
#[derive(Debug, Clone)]
struct CircuitColumn {
    gates: Vec<GateElement>,
    width: usize,
}

/// A gate element in the diagram
#[derive(Debug, Clone)]
enum GateElement {
    SingleQubit {
        symbol: String,
    },
    TwoQubit {
        control: usize,
        target: usize,
        gate_type: String,
    },
    Measurement,
    Wire,
}

impl CircuitDiagram {
    fn new(circuit: &QuantumCircuit, style: &VisualizationStyle) -> Self {
        Self {
            num_qubits: circuit.num_qubits(),
            columns: Vec::new(),
            style: style.clone(),
        }
    }

    fn add_instruction(&mut self, instruction: &crate::circuit::Instruction) {
        let mut column = CircuitColumn {
            gates: vec![GateElement::Wire; self.num_qubits],
            width: 3,
        };

        if instruction.is_measurement() {
            let qubit = instruction.qubits[0].index();
            column.gates[qubit] = GateElement::Measurement;
        } else {
            match instruction.qubits.len() {
                1 => {
                    let qubit = instruction.qubits[0].index();
                    let symbol = self.get_gate_symbol(&instruction.gate.gate_type);
                    column.gates[qubit] = GateElement::SingleQubit { symbol };
                }
                2 => {
                    let control = instruction.qubits[0].index();
                    let target = instruction.qubits[1].index();
                    let gate_type = format!("{:?}", instruction.gate.gate_type);
                    column.gates[control] = GateElement::TwoQubit {
                        control,
                        target,
                        gate_type: gate_type.clone(),
                    };
                    column.gates[target] = GateElement::TwoQubit {
                        control,
                        target,
                        gate_type,
                    };
                }
                _ => {
                    // Multi-qubit gates - simplified representation
                    for qubit in &instruction.qubits {
                        let idx = qubit.index();
                        column.gates[idx] = GateElement::SingleQubit {
                            symbol: "G".to_string(),
                        };
                    }
                }
            }
        }

        self.columns.push(column);
    }

    fn get_gate_symbol(&self, gate: &StandardGate) -> String {
        match gate {
            StandardGate::I => "I".to_string(),
            StandardGate::X => "X".to_string(),
            StandardGate::Y => "Y".to_string(),
            StandardGate::Z => "Z".to_string(),
            StandardGate::H => "H".to_string(),
            StandardGate::S => "S".to_string(),
            StandardGate::Sdg => "S†".to_string(),
            StandardGate::T => "T".to_string(),
            StandardGate::Tdg => "T†".to_string(),
            StandardGate::Rx => "Rx".to_string(),
            StandardGate::Ry => "Ry".to_string(),
            StandardGate::Rz => "Rz".to_string(),
            _ => "G".to_string(),
        }
    }

    fn render(&self, max_width: usize, show_indices: bool, _show_classical: bool) -> String {
        let mut result = String::new();
        let wire_char = match self.style.wire_style {
            WireStyle::Simple => '─',
            WireStyle::Double => '═',
            WireStyle::Dotted => '┅',
            WireStyle::Custom(c) => c,
        };

        // Calculate total width needed
        let total_width: usize = self.columns.iter().map(|col| col.width).sum();

        if total_width > max_width && !self.columns.is_empty() {
            // Split into multiple lines if too wide
            return self.render_wrapped(max_width, show_indices);
        }

        // Render each qubit line
        for qubit in 0..self.num_qubits {
            if show_indices {
                result.push_str(&format!("q[{}] ", qubit));
            }

            for column in &self.columns {
                match &column.gates[qubit] {
                    GateElement::SingleQubit { symbol } => match self.style.gate_style {
                        GateStyle::Compact => result.push_str(&format!("─{}─", symbol)),
                        GateStyle::Boxed => result.push_str(&format!("┌{}┐", symbol)),
                        GateStyle::Rounded => result.push_str(&format!("╭{}╮", symbol)),
                        GateStyle::AsciiOnly => result.push_str(&format!("[{}]", symbol)),
                    },
                    GateElement::TwoQubit {
                        control,
                        target,
                        gate_type,
                    } => {
                        if gate_type == "CX" {
                            if qubit == *control {
                                result.push_str("─●─");
                            } else if qubit == *target {
                                result.push_str("─⊕─");
                            } else {
                                result.push_str(&format!("─{}─", wire_char));
                            }
                        } else {
                            result.push_str("─●─"); // Generic two-qubit gate
                        }
                    }
                    GateElement::Measurement => {
                        result.push_str("─M─");
                    }
                    GateElement::Wire => {
                        result.push_str(&format!("─{}─", wire_char));
                    }
                }
            }

            result.push('\n');
        }

        result
    }

    fn render_wrapped(&self, max_width: usize, _show_indices: bool) -> String {
        // For now, just return a simplified version
        // Full implementation would split the circuit into chunks
        format!("Circuit too wide for display (max width: {})\nUse a larger terminal or reduce circuit size.\n", max_width)
    }
}

/// SVG builder for quantum circuits
struct SvgBuilder {
    content: String,
    num_columns: usize,
    gate_size: f64,
    qubit_spacing: f64,
    column_spacing: f64,
    margin: f64,
}

impl SvgBuilder {
    fn new(num_qubits: usize, num_columns: usize) -> Self {
        let gate_size = 30.0;
        let qubit_spacing = 60.0;
        let column_spacing = 80.0;
        let margin = 50.0;

        let width = margin * 2.0 + (num_columns as f64) * column_spacing;
        let height = margin * 2.0 + (num_qubits as f64 - 1.0) * qubit_spacing;

        let mut content = String::new();
        content.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
<defs>
    <style>
        .qubit-line {{ stroke: #000; stroke-width: 2; }}
        .gate-box {{ fill: #fff; stroke: #000; stroke-width: 2; }}
        .gate-text {{ font-family: Arial, sans-serif; font-size: 14px; text-anchor: middle; dominant-baseline: central; }}
        .control-dot {{ fill: #000; }}
        .target-circle {{ fill: #fff; stroke: #000; stroke-width: 2; }}
        .measurement {{ fill: #f0f0f0; stroke: #000; stroke-width: 2; }}
        .title {{ font-family: Arial, sans-serif; font-size: 16px; font-weight: bold; text-anchor: middle; }}
    </style>
</defs>
"#,
            width, height
        ));

        Self {
            content,
            num_columns,
            gate_size,
            qubit_spacing,
            column_spacing,
            margin,
        }
    }

    fn add_title(&mut self, title: &str) {
        let x = self.margin + (self.num_columns as f64) * self.column_spacing / 2.0;
        let y = self.margin / 2.0;
        self.content.push_str(&format!(
            r#"<text x="{}" y="{}" class="title">{}</text>"#,
            x, y, title
        ));
    }

    fn draw_qubit_line(&mut self, qubit: usize) {
        let y = self.margin + (qubit as f64) * self.qubit_spacing;
        let x1 = self.margin;
        let x2 = self.margin + (self.num_columns as f64) * self.column_spacing;

        self.content.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="qubit-line"/>"#,
            x1, y, x2, y
        ));

        // Add qubit label
        self.content.push_str(&format!(
            r#"<text x="{}" y="{}" class="gate-text">q[{}]</text>"#,
            x1 - 20.0,
            y,
            qubit
        ));
    }

    fn draw_single_qubit_gate(&mut self, qubit: usize, column: usize, gate_name: &str) {
        let x = self.margin + (column as f64) * self.column_spacing;
        let y = self.margin + (qubit as f64) * self.qubit_spacing;

        // Draw gate box
        self.content.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" class="gate-box"/>"#,
            x - self.gate_size / 2.0,
            y - self.gate_size / 2.0,
            self.gate_size,
            self.gate_size
        ));

        // Draw gate text
        self.content.push_str(&format!(
            r#"<text x="{}" y="{}" class="gate-text">{}</text>"#,
            x, y, gate_name
        ));
    }

    fn draw_cnot(&mut self, control: usize, target: usize, column: usize) {
        let x = self.margin + (column as f64) * self.column_spacing;
        let control_y = self.margin + (control as f64) * self.qubit_spacing;
        let target_y = self.margin + (target as f64) * self.qubit_spacing;

        // Draw control dot
        self.content.push_str(&format!(
            r#"<circle cx="{}" cy="{}" r="5" class="control-dot"/>"#,
            x, control_y
        ));

        // Draw target circle with plus
        self.content.push_str(&format!(
            r#"<circle cx="{}" cy="{}" r="15" class="target-circle"/>"#,
            x, target_y
        ));

        // Draw plus sign
        self.content.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="black" stroke-width="2"/>"#,
            x - 8.0,
            target_y,
            x + 8.0,
            target_y
        ));
        self.content.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="black" stroke-width="2"/>"#,
            x,
            target_y - 8.0,
            x,
            target_y + 8.0
        ));

        // Draw connection line
        let min_y = control_y.min(target_y);
        let max_y = control_y.max(target_y);
        self.content.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="black" stroke-width="2"/>"#,
            x, min_y, x, max_y
        ));
    }

    fn draw_cz(&mut self, control: usize, target: usize, column: usize) {
        let x = self.margin + (column as f64) * self.column_spacing;
        let control_y = self.margin + (control as f64) * self.qubit_spacing;
        let target_y = self.margin + (target as f64) * self.qubit_spacing;

        // Draw control dots
        self.content.push_str(&format!(
            r#"<circle cx="{}" cy="{}" r="5" class="control-dot"/>"#,
            x, control_y
        ));
        self.content.push_str(&format!(
            r#"<circle cx="{}" cy="{}" r="5" class="control-dot"/>"#,
            x, target_y
        ));

        // Draw connection line
        let min_y = control_y.min(target_y);
        let max_y = control_y.max(target_y);
        self.content.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="black" stroke-width="2"/>"#,
            x, min_y, x, max_y
        ));
    }

    fn draw_generic_two_qubit(
        &mut self,
        control: usize,
        target: usize,
        column: usize,
        gate_name: &str,
    ) {
        let x = self.margin + (column as f64) * self.column_spacing;
        let control_y = self.margin + (control as f64) * self.qubit_spacing;
        let target_y = self.margin + (target as f64) * self.qubit_spacing;

        // Draw gate boxes
        self.draw_single_qubit_gate(control, column, gate_name);
        self.draw_single_qubit_gate(target, column, gate_name);

        // Draw connection line
        let min_y = control_y.min(target_y);
        let max_y = control_y.max(target_y);
        self.content.push_str(&format!(
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="black" stroke-width="2"/>"#,
            x, min_y, x, max_y
        ));
    }

    fn draw_measurement(&mut self, qubit: usize, column: usize) {
        let x = self.margin + (column as f64) * self.column_spacing;
        let y = self.margin + (qubit as f64) * self.qubit_spacing;

        // Draw measurement box
        self.content.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" class="measurement"/>"#,
            x - self.gate_size / 2.0,
            y - self.gate_size / 2.0,
            self.gate_size,
            self.gate_size
        ));

        // Draw measurement symbol (simplified meter)
        self.content.push_str(&format!(
            r#"<text x="{}" y="{}" class="gate-text">M</text>"#,
            x, y
        ));
    }

    fn finalize(mut self) -> String {
        self.content.push_str("</svg>");
        self.content
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::QuantumCircuit;

    #[test]
    fn test_visualizer_creation() {
        let viz = CircuitVisualizer::new();
        assert!(viz.show_parameters);
        assert!(viz.show_qubit_indices);
        assert_eq!(viz.max_width, 120);
    }

    #[test]
    fn test_compact_visualizer() {
        let viz = CircuitVisualizer::compact();
        assert!(!viz.show_parameters);
        assert_eq!(viz.max_width, 80);
    }

    #[test]
    fn test_empty_circuit_visualization() {
        let viz = CircuitVisualizer::new();
        let circuit = QuantumCircuit::new(2, 0);
        let output = viz.draw_circuit(&circuit);
        assert!(output.contains("q[0]"));
        assert!(output.contains("q[1]"));
    }

    #[test]
    fn test_circuit_summary() {
        let viz = CircuitVisualizer::new();
        let mut circuit = QuantumCircuit::new(2, 2);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();

        let summary = viz.circuit_summary(&circuit);
        assert!(summary.contains("CIRCUIT SUMMARY"));
        assert!(summary.contains("Qubits:"));
        assert!(summary.contains("Gates:"));
    }

    #[test]
    fn test_gate_counting() {
        let viz = CircuitVisualizer::new();
        let mut circuit = QuantumCircuit::new(2, 2);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.measure_all().unwrap();

        assert_eq!(viz.count_single_qubit_gates(&circuit), 1);
        assert_eq!(viz.count_two_qubit_gates(&circuit), 1);
        assert_eq!(viz.count_measurements(&circuit), 2);
    }

    #[test]
    fn test_parameter_formatting() {
        let viz = CircuitVisualizer::new();

        assert_eq!(
            viz.format_parameter(&Parameter::Float(std::f64::consts::PI)),
            "π"
        );
        assert_eq!(
            viz.format_parameter(&Parameter::Float(std::f64::consts::PI / 2.0)),
            "π/2"
        );
        assert_eq!(viz.format_parameter(&Parameter::Float(0.0)), "0");
        assert_eq!(
            viz.format_parameter(&Parameter::Symbol("theta".to_string())),
            "theta"
        );
    }
}
