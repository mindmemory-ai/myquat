//! # Quantum Circuit Implementation
//!
//! This module provides the core quantum circuit functionality for MyQuat, including
//! circuit construction, gate application, measurement operations, and circuit analysis.
//!
//! ## Overview
//!
//! The [`QuantumCircuit`] is the fundamental data structure for representing quantum
//! computations. It maintains a sequence of quantum operations (gates and measurements)
//! that can be applied to qubits and classical bits.
//!
//! ## Key Features
//!
//! - **Flexible Circuit Construction**: Add gates, measurements, and barriers
//! - **Parameter Support**: Parametric gates with symbolic and numeric parameters
//! - **Circuit Analysis**: Depth calculation, gate counting, and optimization metrics
//! - **Serialization**: JSON import/export for circuit persistence
//! - **Validation**: Automatic qubit and classical bit range checking
//!
//! ## Basic Usage
//!
//! ```rust
//! use myquat::*;
//!
//! // Create a 3-qubit, 3-classical-bit circuit
//! let mut circuit = QuantumCircuit::new(3, 3);
//!
//! // Add quantum gates
//! circuit.h(0)?;                    // Hadamard on qubit 0
//! circuit.cx(0, 1)?;                // CNOT from qubit 0 to 1
//! circuit.ry(2, Parameter::Float(std::f64::consts::PI / 4.0))?; // Parametric rotation
//!
//! // Add measurements
//! circuit.measure(0, 0)?;           // Measure qubit 0 to classical bit 0
//! circuit.measure_all()?;           // Measure all remaining qubits
//!
//! // Circuit analysis
//! println!("Circuit depth: {}", circuit.depth());
//! println!("Gate count: {}", circuit.size());
//! println!("Is empty: {}", circuit.is_empty());
//! # Ok::<(), myquat::MyQuatError>(())
//! ```
//!
//! ## Advanced Features
//!
//! ### Parametric Circuits
//!
//! ```rust
//! use myquat::*;
//! use std::collections::HashMap;
//!
//! // Create a parametric circuit
//! let mut circuit = QuantumCircuit::new(2, 0);
//! circuit.ry(0, Parameter::Symbol("theta".to_string()))?;
//! circuit.cx(0, 1)?;
//!
//! // Bind parameters
//! let mut params = HashMap::new();
//! params.insert("theta".to_string(), std::f64::consts::PI / 3.0);
//! let bound_circuit = circuit.bind_parameters(&params)?;
//! # Ok::<(), myquat::MyQuatError>(())
//! ```
//!
//! ### Circuit Composition
//!
//! ```rust
//! use myquat::*;
//!
//! // Create sub-circuits
//! let mut bell_circuit = QuantumCircuit::new(2, 0);
//! bell_circuit.h(0)?;
//! bell_circuit.cx(0, 1)?;
//!
//! // Compose circuits
//! let mut main_circuit = QuantumCircuit::new(4, 0);
//! main_circuit.h(2)?;
//! // main_circuit.compose(&bell_circuit, &[0, 1])?; // Would compose bell_circuit on qubits 0,1
//! # Ok::<(), myquat::MyQuatError>(())
//! ```
//!
//! ## Circuit Data Structure
//!
//! The circuit maintains:
//! - **Instructions**: Sequence of gates and measurements
//! - **Qubit Register**: Number of quantum bits
//! - **Classical Register**: Number of classical bits  
//! - **Metadata**: Circuit name, creation time, etc.
//!
//! ## Performance Considerations
//!
//! - Circuit operations are optimized for sequential access
//! - Gate application has O(1) complexity
//! - Circuit depth calculation is cached and updated incrementally
//! - Memory usage scales linearly with the number of instructions analysis.

use crate::error::{MyQuatError, Result};
use crate::gates::{Gate, GateOperation, StandardGate};
use crate::linalg::{LinalgBackend, LinalgResult, NdArrayBackend};
use crate::parameter::Parameter;
use ndarray::Array2;
use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A quantum bit (qubit) identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Qubit(pub usize);

impl Qubit {
    /// Create a new qubit with the given index
    pub fn new(index: usize) -> Self {
        Qubit(index)
    }

    /// Get the index of this qubit
    pub fn index(&self) -> usize {
        self.0
    }
}

impl fmt::Display for Qubit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "q[{}]", self.0)
    }
}

/// A classical bit identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ClassicalBit(pub usize);

impl ClassicalBit {
    /// Create a new classical bit with the given index
    pub fn new(index: usize) -> Self {
        ClassicalBit(index)
    }

    /// Get the index of this classical bit
    pub fn index(&self) -> usize {
        self.0
    }
}

impl fmt::Display for ClassicalBit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "c[{}]", self.0)
    }
}

/// A quantum circuit instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    /// The gate to apply
    pub gate: Gate,
    /// The qubits this instruction acts on
    pub qubits: Vec<Qubit>,
    /// The classical bits for measurement (if applicable)
    pub clbits: Vec<ClassicalBit>,
}

impl Instruction {
    /// Create a new instruction
    pub fn new(gate: Gate, qubits: Vec<Qubit>) -> Result<Self> {
        if qubits.len() != gate.gate_type.num_qubits() {
            return Err(MyQuatError::circuit_error(format!(
                "Gate {} requires {} qubits, got {}",
                gate.gate_type.name(),
                gate.gate_type.num_qubits(),
                qubits.len()
            )));
        }

        Ok(Instruction {
            gate,
            qubits,
            clbits: Vec::new(),
        })
    }

    /// Create a new measurement instruction
    pub fn new_measurement(qubit: Qubit, clbit: ClassicalBit) -> Self {
        Instruction {
            gate: Gate::new(StandardGate::I, vec![]).unwrap(), // Placeholder
            qubits: vec![qubit],
            clbits: vec![clbit],
        }
    }

    /// Check if this is a measurement instruction
    pub fn is_measurement(&self) -> bool {
        !self.clbits.is_empty()
    }

    /// Get the maximum qubit index used by this instruction
    pub fn max_qubit_index(&self) -> Option<usize> {
        self.qubits.iter().map(|q| q.index()).max()
    }

    /// Get the maximum classical bit index used by this instruction
    pub fn max_clbit_index(&self) -> Option<usize> {
        self.clbits.iter().map(|c| c.index()).max()
    }
}

/// Core quantum circuit data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitData {
    /// Number of qubits in the circuit
    num_qubits: usize,
    /// Number of classical bits in the circuit
    num_clbits: usize,
    /// List of instructions in the circuit
    instructions: Vec<Instruction>,
    /// Global phase of the circuit
    global_phase: Parameter,
    /// Metadata for the circuit
    metadata: HashMap<String, String>,
}

impl CircuitData {
    /// Create a new empty circuit data
    pub fn new(num_qubits: usize, num_clbits: usize) -> Self {
        CircuitData {
            num_qubits,
            num_clbits,
            instructions: Vec::new(),
            global_phase: Parameter::new_float(0.0),
            metadata: HashMap::new(),
        }
    }

    /// Get the number of qubits
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Get the number of classical bits
    pub fn num_clbits(&self) -> usize {
        self.num_clbits
    }

    /// Get the number of instructions
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Check if the circuit is empty
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    /// Add an instruction to the circuit
    pub fn add_instruction(&mut self, instruction: Instruction) -> Result<()> {
        // Validate qubit indices
        if let Some(max_qubit) = instruction.max_qubit_index() {
            if max_qubit >= self.num_qubits {
                return Err(MyQuatError::InvalidQubitIndex {
                    index: max_qubit,
                    num_qubits: self.num_qubits,
                });
            }
        }

        // Validate classical bit indices
        if let Some(max_clbit) = instruction.max_clbit_index() {
            if max_clbit >= self.num_clbits {
                return Err(MyQuatError::InvalidClbitIndex {
                    index: max_clbit,
                    num_clbits: self.num_clbits,
                });
            }
        }

        self.instructions.push(instruction);
        Ok(())
    }

    /// Get all instructions
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    /// Get mutable access to instructions
    pub fn instructions_mut(&mut self) -> &mut Vec<Instruction> {
        &mut self.instructions
    }

    /// Get the global phase
    pub fn global_phase(&self) -> &Parameter {
        &self.global_phase
    }

    /// Set the global phase
    pub fn set_global_phase(&mut self, phase: Parameter) {
        self.global_phase = phase;
    }

    /// Get metadata
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }

    /// Set metadata
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get the depth of the circuit (number of sequential gate layers)
    pub fn depth(&self) -> usize {
        if self.instructions.is_empty() {
            return 0;
        }

        let mut qubit_depths = vec![0; self.num_qubits];

        for instruction in &self.instructions {
            if instruction.is_measurement() {
                continue; // Skip measurements for depth calculation
            }

            let max_depth = instruction
                .qubits
                .iter()
                .map(|q| qubit_depths[q.index()])
                .max()
                .unwrap_or(0);

            for qubit in &instruction.qubits {
                qubit_depths[qubit.index()] = max_depth + 1;
            }
        }

        qubit_depths.into_iter().max().unwrap_or(0)
    }

    /// Count the number of gates of a specific type
    pub fn count_gates(&self, gate_type: StandardGate) -> usize {
        self.instructions
            .iter()
            .filter(|inst| inst.gate.gate_type == gate_type)
            .count()
    }

    /// Get all unique gates used in the circuit
    pub fn gate_types(&self) -> Vec<StandardGate> {
        let mut gates: Vec<_> = self
            .instructions
            .iter()
            .map(|inst| inst.gate.gate_type)
            .collect();
        gates.sort();
        gates.dedup();
        gates
    }

    /// Check if the circuit contains any parametric gates
    pub fn is_parametric(&self) -> bool {
        self.instructions
            .iter()
            .any(|inst| inst.gate.is_parametric())
            || !self.global_phase.is_numeric()
    }

    /// Get all symbols used in the circuit
    pub fn symbols(&self) -> Vec<String> {
        let mut all_symbols = Vec::new();

        // Collect symbols from gates
        for instruction in &self.instructions {
            all_symbols.extend(instruction.gate.symbols());
        }

        // Collect symbols from global phase
        all_symbols.extend(self.global_phase.symbols());

        all_symbols.sort();
        all_symbols.dedup();
        all_symbols
    }

    /// Create a copy of the circuit with parameters bound to specific values
    pub fn bind_parameters(&self, symbols: &HashMap<String, f64>) -> Result<CircuitData> {
        let mut bound_circuit = CircuitData::new(self.num_qubits, self.num_clbits);
        bound_circuit.metadata = self.metadata.clone();

        // Bind global phase
        if let Ok(phase_value) = self.global_phase.evaluate(symbols) {
            bound_circuit.global_phase = Parameter::new_float(phase_value);
        } else {
            bound_circuit.global_phase = self.global_phase.clone();
        }

        // Bind instruction parameters
        for instruction in &self.instructions {
            let mut bound_params = Vec::new();
            for param in &instruction.gate.parameters {
                if let Ok(value) = param.evaluate(symbols) {
                    bound_params.push(Parameter::new_float(value));
                } else {
                    bound_params.push(param.clone());
                }
            }

            let bound_gate = Gate::new(instruction.gate.gate_type, bound_params)?;
            let bound_instruction = Instruction {
                gate: bound_gate,
                qubits: instruction.qubits.clone(),
                clbits: instruction.clbits.clone(),
            };

            bound_circuit.add_instruction(bound_instruction)?;
        }

        Ok(bound_circuit)
    }
}

/// High-level quantum circuit interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumCircuit {
    /// Core circuit data
    data: CircuitData,
    /// Circuit name
    name: Option<String>,
}

impl QuantumCircuit {
    /// Create a new quantum circuit
    pub fn new(num_qubits: usize, num_clbits: usize) -> Self {
        QuantumCircuit {
            data: CircuitData::new(num_qubits, num_clbits),
            name: None,
        }
    }

    /// Create a new quantum circuit with a name
    pub fn new_with_name(num_qubits: usize, num_clbits: usize, name: String) -> Self {
        let mut circuit = Self::new(num_qubits, num_clbits);
        circuit.name = Some(name);
        circuit
    }

    /// Get the circuit name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Set the circuit name
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    /// Get the number of qubits
    pub fn num_qubits(&self) -> usize {
        self.data.num_qubits()
    }

    /// Get the number of classical bits
    pub fn num_clbits(&self) -> usize {
        self.data.num_clbits()
    }

    /// Get the circuit depth
    pub fn depth(&self) -> usize {
        self.data.depth()
    }

    /// Get the number of instructions
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Check if the circuit is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get access to the circuit data
    pub fn data(&self) -> &CircuitData {
        &self.data
    }

    /// Get mutable access to the circuit data
    pub fn data_mut(&mut self) -> &mut CircuitData {
        &mut self.data
    }

    // Gate application methods

    /// Apply an identity gate
    pub fn i(&mut self, qubit: usize) -> Result<()> {
        self.apply_gate(Gate::i(), vec![qubit])
    }

    /// Apply a Pauli-X gate
    pub fn x(&mut self, qubit: usize) -> Result<()> {
        self.apply_gate(Gate::x(), vec![qubit])
    }

    /// Apply a Pauli-Y gate
    pub fn y(&mut self, qubit: usize) -> Result<()> {
        self.apply_gate(Gate::y(), vec![qubit])
    }

    /// Apply a Pauli-Z gate
    pub fn z(&mut self, qubit: usize) -> Result<()> {
        self.apply_gate(Gate::z(), vec![qubit])
    }

    /// Apply a Hadamard gate
    pub fn h(&mut self, qubit: usize) -> Result<()> {
        self.apply_gate(Gate::h(), vec![qubit])
    }

    /// Apply an S gate
    pub fn s(&mut self, qubit: usize) -> Result<()> {
        self.apply_gate(Gate::s(), vec![qubit])
    }

    /// Apply an S-dagger gate
    pub fn sdg(&mut self, qubit: usize) -> Result<()> {
        self.apply_gate(Gate::sdg(), vec![qubit])
    }

    /// Apply a T gate
    pub fn t(&mut self, qubit: usize) -> Result<()> {
        self.apply_gate(Gate::t(), vec![qubit])
    }

    /// Apply a T-dagger gate
    pub fn tdg(&mut self, qubit: usize) -> Result<()> {
        self.apply_gate(Gate::tdg(), vec![qubit])
    }

    /// Apply an RX rotation gate
    pub fn rx(&mut self, qubit: usize, theta: Parameter) -> Result<()> {
        self.apply_gate(Gate::rx(theta), vec![qubit])
    }

    /// Apply an RY rotation gate
    pub fn ry(&mut self, qubit: usize, theta: Parameter) -> Result<()> {
        self.apply_gate(Gate::ry(theta), vec![qubit])
    }

    /// Apply an RZ rotation gate
    pub fn rz(&mut self, qubit: usize, theta: Parameter) -> Result<()> {
        self.apply_gate(Gate::rz(theta), vec![qubit])
    }

    /// Apply a phase gate
    pub fn p(&mut self, qubit: usize, phi: Parameter) -> Result<()> {
        self.apply_gate(Gate::p(phi), vec![qubit])
    }

    /// Apply universal single-qubit gates
    pub fn u(
        &mut self,
        qubit: usize,
        theta: Parameter,
        phi: Parameter,
        lambda: Parameter,
    ) -> Result<()> {
        self.apply_gate(Gate::u(theta, phi, lambda), vec![qubit])
    }

    /// Apply a U1 gate (phase rotation) on the given qubit.
    ///
    /// Equivalent to $R_z(\lambda)$ up to global phase:
    /// $\mathrm{U1}(\lambda) = \mathrm{diag}(1, e^{i\lambda})$.
    pub fn u1(&mut self, qubit: usize, lambda: Parameter) -> Result<()> {
        self.apply_gate(Gate::u1(lambda), vec![qubit])
    }

    /// Apply a U2 gate on the given qubit.
    ///
    /// $\mathrm{U2}(\phi, \lambda) = \frac{1}{\sqrt{2}}
    /// \begin{pmatrix} 1 & -e^{i\lambda} \\ e^{i\phi} & e^{i(\phi+\lambda)} \end{pmatrix}$.
    pub fn u2(&mut self, qubit: usize, phi: Parameter, lambda: Parameter) -> Result<()> {
        self.apply_gate(Gate::u2(phi, lambda), vec![qubit])
    }

    /// Apply a U3 gate (generic single-qubit rotation) on the given qubit.
    ///
    /// $\mathrm{U3}(\theta, \phi, \lambda) =
    /// \begin{pmatrix}
    ///   \cos(\theta/2) & -e^{i\lambda}\sin(\theta/2) \\
    ///   e^{i\phi}\sin(\theta/2) & e^{i(\phi+\lambda)}\cos(\theta/2)
    /// \end{pmatrix}$.
    pub fn u3(
        &mut self,
        qubit: usize,
        theta: Parameter,
        phi: Parameter,
        lambda: Parameter,
    ) -> Result<()> {
        self.apply_gate(Gate::u3(theta, phi, lambda), vec![qubit])
    }

    /// Apply a CNOT gate
    pub fn cx(&mut self, control: usize, target: usize) -> Result<()> {
        self.apply_gate(Gate::cx(), vec![control, target])
    }

    /// Apply a CNOT gate (alias for cx)
    pub fn cnot(&mut self, control: usize, target: usize) -> Result<()> {
        self.apply_gate(Gate::cnot(), vec![control, target])
    }

    /// Apply a controlled-Y gate
    pub fn cy(&mut self, control: usize, target: usize) -> Result<()> {
        self.apply_gate(Gate::cy(), vec![control, target])
    }

    /// Apply a controlled-Z gate
    pub fn cz(&mut self, control: usize, target: usize) -> Result<()> {
        self.apply_gate(Gate::cz(), vec![control, target])
    }

    /// Apply a controlled-Hadamard gate
    pub fn ch(&mut self, control: usize, target: usize) -> Result<()> {
        self.apply_gate(Gate::ch(), vec![control, target])
    }

    /// Apply controlled rotation gates
    pub fn crx(&mut self, control: usize, target: usize, theta: Parameter) -> Result<()> {
        self.apply_gate(Gate::crx(theta), vec![control, target])
    }

    /// Apply a controlled-RY rotation gate.
    ///
    /// Rotates the target qubit around the $Y$-axis by $\theta$ when the control is $|1\rangle$.
    pub fn cry(&mut self, control: usize, target: usize, theta: Parameter) -> Result<()> {
        self.apply_gate(Gate::cry(theta), vec![control, target])
    }

    /// Apply a controlled-RZ rotation gate.
    ///
    /// Rotates the target qubit around the $Z$-axis by $\theta$ when the control is $|1\rangle$.
    pub fn crz(&mut self, control: usize, target: usize, theta: Parameter) -> Result<()> {
        self.apply_gate(Gate::crz(theta), vec![control, target])
    }

    /// Apply a controlled phase gate
    pub fn cp(&mut self, control: usize, target: usize, phi: Parameter) -> Result<()> {
        self.apply_gate(Gate::cp(phi), vec![control, target])
    }

    /// Apply a SWAP gate
    pub fn swap(&mut self, qubit1: usize, qubit2: usize) -> Result<()> {
        self.apply_gate(Gate::swap(), vec![qubit1, qubit2])
    }

    /// Apply an iSWAP gate
    pub fn iswap(&mut self, qubit1: usize, qubit2: usize) -> Result<()> {
        self.apply_gate(Gate::iswap(), vec![qubit1, qubit2])
    }

    /// Apply a Toffoli (CCX) gate
    pub fn ccx(&mut self, control1: usize, control2: usize, target: usize) -> Result<()> {
        self.apply_gate(Gate::ccx(), vec![control1, control2, target])
    }

    /// Apply a Toffoli gate (alias for ccx)
    pub fn toffoli(&mut self, control1: usize, control2: usize, target: usize) -> Result<()> {
        self.apply_gate(Gate::toffoli(), vec![control1, control2, target])
    }

    /// Apply a Fredkin (CSWAP) gate
    pub fn cswap(&mut self, control: usize, target1: usize, target2: usize) -> Result<()> {
        self.apply_gate(Gate::cswap(), vec![control, target1, target2])
    }

    /// Apply a Fredkin gate (alias for cswap)
    pub fn fredkin(&mut self, control: usize, target1: usize, target2: usize) -> Result<()> {
        self.apply_gate(Gate::fredkin(), vec![control, target1, target2])
    }

    /// Apply multi-controlled gates (simplified 3-qubit versions)
    pub fn mcx(&mut self, control1: usize, control2: usize, target: usize) -> Result<()> {
        self.apply_gate(Gate::mcx(), vec![control1, control2, target])
    }

    /// Apply a doubly-controlled Y gate (CCY) on two control and one target qubit.
    ///
    /// Applies a $Y$ gate to the target when both controls are $|1\rangle$.
    pub fn mcy(&mut self, control1: usize, control2: usize, target: usize) -> Result<()> {
        self.apply_gate(Gate::mcy(), vec![control1, control2, target])
    }

    /// Apply a doubly-controlled Z gate (CCZ) on two control and one target qubit.
    ///
    /// Applies a $Z$ gate to the target when both controls are $|1\rangle$.
    pub fn mcz(&mut self, control1: usize, control2: usize, target: usize) -> Result<()> {
        self.apply_gate(Gate::mcz(), vec![control1, control2, target])
    }

    /// Apply a measurement
    pub fn measure(&mut self, qubit: usize, clbit: usize) -> Result<()> {
        let instruction = Instruction::new_measurement(Qubit::new(qubit), ClassicalBit::new(clbit));
        self.data.add_instruction(instruction)
    }

    /// Measure all qubits to corresponding classical bits
    pub fn measure_all(&mut self) -> Result<()> {
        let num_qubits = self.num_qubits();
        let num_clbits = self.num_clbits();

        if num_clbits < num_qubits {
            return Err(MyQuatError::circuit_error(format!(
                "Not enough classical bits ({}) to measure all qubits ({})",
                num_clbits, num_qubits
            )));
        }

        for i in 0..num_qubits {
            self.measure(i, i)?;
        }

        Ok(())
    }

    /// Apply a gate to specified qubits
    fn apply_gate(&mut self, gate: Gate, qubits: Vec<usize>) -> Result<()> {
        let qubit_objects: Vec<Qubit> = qubits.into_iter().map(Qubit::new).collect();
        let instruction = Instruction::new(gate, qubit_objects)?;
        self.data.add_instruction(instruction)
    }

    /// Create a copy of the circuit with parameters bound
    pub fn bind_parameters(&self, symbols: &HashMap<String, f64>) -> Result<QuantumCircuit> {
        let bound_data = self.data.bind_parameters(symbols)?;
        Ok(QuantumCircuit {
            data: bound_data,
            name: self.name.clone(),
        })
    }

    /// Get the unitary matrix representation of the circuit (for small circuits)
    pub fn unitary(&self, symbols: &HashMap<String, f64>) -> Result<Array2<Complex64>> {
        if self.num_qubits() > 10 {
            return Err(MyQuatError::circuit_error(
                "Circuit too large for unitary matrix computation (>10 qubits)".to_string(),
            ));
        }

        let dim = 1 << self.num_qubits();
        let mut unitary = Array2::eye(dim).mapv(|x| Complex64::new(x, 0.0));

        for instruction in self.data.instructions() {
            if instruction.is_measurement() {
                continue; // Skip measurements
            }

            let gate_matrix = instruction.gate.matrix(symbols)?;
            let expanded_matrix = self.expand_gate_matrix(&gate_matrix, &instruction.qubits)?;
            unitary = expanded_matrix.dot(&unitary);
        }

        // Apply global phase
        if let Ok(phase) = self.data.global_phase().evaluate(symbols) {
            let phase_factor = Complex64::new(0.0, phase).exp();
            unitary.mapv_inplace(|x| x * phase_factor);
        }

        Ok(unitary)
    }

    /// Compute the circuit unitary using a linear algebra backend
    pub fn unitary_with_backend<B: LinalgBackend<Scalar = Complex64>>(
        &self,
        symbols: &HashMap<String, f64>,
        backend: &B,
    ) -> LinalgResult<B::Matrix> {
        let nda = self
            .unitary(symbols)
            .map_err(|e| crate::linalg::LinalgError::BackendError(e.to_string()))?;
        let (r, c) = nda.dim();
        let data: Vec<Complex64> = nda.iter().copied().collect();
        backend.from_shape_vec(r, c, data)
    }

    /// Expand a gate matrix to act on the full circuit space
    fn expand_gate_matrix(
        &self,
        gate_matrix: &Array2<Complex64>,
        qubits: &[Qubit],
    ) -> Result<Array2<Complex64>> {
        let n_qubits = self.num_qubits();
        let dim = 1 << n_qubits;
        let mut expanded = Array2::zeros((dim, dim));

        // For now, implement a simple case for single and two-qubit gates
        match qubits.len() {
            1 => {
                let qubit_idx = qubits[0].index();
                for i in 0..dim {
                    for j in 0..dim {
                        // Check if states i and j differ only in the target qubit
                        let mask = !(1 << (n_qubits - 1 - qubit_idx));
                        if (i & mask) == (j & mask) {
                            let i_bit = (i >> (n_qubits - 1 - qubit_idx)) & 1;
                            let j_bit = (j >> (n_qubits - 1 - qubit_idx)) & 1;
                            expanded[[i, j]] = gate_matrix[[i_bit, j_bit]];
                        }
                    }
                }
            }
            2 => {
                let qubit0_idx = qubits[0].index();
                let qubit1_idx = qubits[1].index();

                for i in 0..dim {
                    for j in 0..dim {
                        // Check if states i and j differ only in the target qubits
                        let mask = !(1 << (n_qubits - 1 - qubit0_idx))
                            & !(1 << (n_qubits - 1 - qubit1_idx));
                        if (i & mask) == (j & mask) {
                            let i_bit0 = (i >> (n_qubits - 1 - qubit0_idx)) & 1;
                            let i_bit1 = (i >> (n_qubits - 1 - qubit1_idx)) & 1;
                            let j_bit0 = (j >> (n_qubits - 1 - qubit0_idx)) & 1;
                            let j_bit1 = (j >> (n_qubits - 1 - qubit1_idx)) & 1;

                            let i_state = i_bit0 * 2 + i_bit1;
                            let j_state = j_bit0 * 2 + j_bit1;
                            expanded[[i, j]] = gate_matrix[[i_state, j_state]];
                        }
                    }
                }
            }
            3 => {
                let qubit0_idx = qubits[0].index();
                let qubit1_idx = qubits[1].index();
                let qubit2_idx = qubits[2].index();

                for i in 0..dim {
                    for j in 0..dim {
                        // Check if states i and j differ only in the target qubits
                        let mask = !(1 << (n_qubits - 1 - qubit0_idx))
                            & !(1 << (n_qubits - 1 - qubit1_idx))
                            & !(1 << (n_qubits - 1 - qubit2_idx));
                        if (i & mask) == (j & mask) {
                            let i_bit0 = (i >> (n_qubits - 1 - qubit0_idx)) & 1;
                            let i_bit1 = (i >> (n_qubits - 1 - qubit1_idx)) & 1;
                            let i_bit2 = (i >> (n_qubits - 1 - qubit2_idx)) & 1;
                            let j_bit0 = (j >> (n_qubits - 1 - qubit0_idx)) & 1;
                            let j_bit1 = (j >> (n_qubits - 1 - qubit1_idx)) & 1;
                            let j_bit2 = (j >> (n_qubits - 1 - qubit2_idx)) & 1;

                            let i_state = i_bit0 * 4 + i_bit1 * 2 + i_bit2;
                            let j_state = j_bit0 * 4 + j_bit1 * 2 + j_bit2;
                            expanded[[i, j]] = gate_matrix[[i_state, j_state]];
                        }
                    }
                }
            }
            _ => {
                return Err(MyQuatError::circuit_error(format!(
                    "Matrix expansion not implemented for {}-qubit gates",
                    qubits.len()
                )));
            }
        }

        Ok(expanded)
    }
}

impl fmt::Display for QuantumCircuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "QuantumCircuit({} qubits, {} clbits)",
            self.num_qubits(),
            self.num_clbits()
        )?;
        if let Some(name) = &self.name {
            writeln!(f, "Name: {}", name)?;
        }
        writeln!(f, "Depth: {}, Size: {}", self.depth(), self.size())?;

        if !self.data.instructions().is_empty() {
            writeln!(f, "Instructions:")?;
            for (i, instruction) in self.data.instructions().iter().enumerate() {
                if instruction.is_measurement() {
                    writeln!(
                        f,
                        "  {}: measure {} -> {}",
                        i, instruction.qubits[0], instruction.clbits[0]
                    )?;
                } else {
                    write!(f, "  {}: {} ", i, instruction.gate.gate_type)?;
                    for (j, qubit) in instruction.qubits.iter().enumerate() {
                        if j > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", qubit)?;
                    }
                    if !instruction.gate.parameters.is_empty() {
                        write!(f, " (")?;
                        for (j, param) in instruction.gate.parameters.iter().enumerate() {
                            if j > 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{}", param)?;
                        }
                        write!(f, ")")?;
                    }
                    writeln!(f)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_circuit_creation() {
        let circuit = QuantumCircuit::new(3, 2);
        assert_eq!(circuit.num_qubits(), 3);
        assert_eq!(circuit.num_clbits(), 2);
        assert_eq!(circuit.depth(), 0);
        assert_eq!(circuit.size(), 0);
        assert!(circuit.is_empty());
    }

    #[test]
    fn test_gate_application() {
        let mut circuit = QuantumCircuit::new(2, 0);

        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();

        assert_eq!(circuit.size(), 2);
        assert_eq!(circuit.depth(), 2);

        let gates = circuit.data().gate_types();
        assert!(gates.contains(&StandardGate::H));
        assert!(gates.contains(&StandardGate::CX));
    }

    #[test]
    fn test_parametric_circuit() {
        let mut circuit = QuantumCircuit::new(1, 0);
        circuit.rx(0, Parameter::new_symbol("theta")).unwrap();

        assert!(circuit.data().is_parametric());
        assert_eq!(circuit.data().symbols(), vec!["theta".to_string()]);

        let mut symbols = HashMap::new();
        symbols.insert("theta".to_string(), PI / 2.0);

        let bound_circuit = circuit.bind_parameters(&symbols).unwrap();
        assert!(!bound_circuit.data().is_parametric());
    }

    #[test]
    fn test_measurement() {
        let mut circuit = QuantumCircuit::new(2, 2);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.measure(0, 0).unwrap();
        circuit.measure(1, 1).unwrap();

        assert_eq!(circuit.size(), 4);

        let measurements = circuit
            .data()
            .instructions()
            .iter()
            .filter(|inst| inst.is_measurement())
            .count();
        assert_eq!(measurements, 2);
    }

    #[test]
    fn test_circuit_unitary() {
        let mut circuit = QuantumCircuit::new(1, 0);
        circuit.x(0).unwrap();

        let symbols = HashMap::new();
        let unitary = circuit.unitary(&symbols).unwrap();

        // Should be the Pauli-X matrix
        assert_eq!(unitary.dim(), (2, 2));
        assert!((unitary[[0, 0]] - Complex64::new(0.0, 0.0)).norm() < 1e-10);
        assert!((unitary[[0, 1]] - Complex64::new(1.0, 0.0)).norm() < 1e-10);
        assert!((unitary[[1, 0]] - Complex64::new(1.0, 0.0)).norm() < 1e-10);
        assert!((unitary[[1, 1]] - Complex64::new(0.0, 0.0)).norm() < 1e-10);
    }

    #[test]
    fn test_invalid_qubit_index() {
        let mut circuit = QuantumCircuit::new(2, 0);
        let result = circuit.x(2); // Invalid qubit index
        assert!(result.is_err());

        if let Err(MyQuatError::InvalidQubitIndex { index, num_qubits }) = result {
            assert_eq!(index, 2);
            assert_eq!(num_qubits, 2);
        } else {
            panic!("Expected InvalidQubitIndex error");
        }
    }
}
