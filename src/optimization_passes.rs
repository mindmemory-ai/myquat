//! Advanced optimization passes
//!
//! This module provides sophisticated circuit optimization algorithms
//! including commutation analysis, gate scheduling, and hardware mapping.

use crate::circuit_optimizer::OptimizationGate;
use crate::error::{MyQuatError, Result};
use crate::{HardwareTopology, QuantumCircuit, StandardGate};
use std::collections::{HashMap, HashSet};

/// Dependency graph for gate scheduling
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Gates in the circuit
    gates: Vec<OptimizationGate>,
    /// Dependencies between gates (gate_id -> dependent_gate_ids)
    dependencies: HashMap<usize, Vec<usize>>,
    /// Reverse dependencies (gate_id -> gates_that_depend_on_it)
    #[allow(dead_code)]
    reverse_dependencies: HashMap<usize, Vec<usize>>,
}

impl DependencyGraph {
    /// Build dependency graph from circuit
    pub fn from_circuit(circuit: &QuantumCircuit) -> Self {
        let mut gates = Vec::new();
        let mut dependencies = HashMap::new();
        let mut reverse_dependencies = HashMap::new();

        // Convert circuit instructions to optimization gates
        for (i, instruction) in circuit.data().instructions().iter().enumerate() {
            if !instruction.is_measurement() {
                let qubits: Vec<usize> = instruction.qubits.iter().map(|q| q.index()).collect();
                let opt_gate = OptimizationGate::new(
                    instruction.gate.gate_type,
                    qubits,
                    instruction.gate.parameters.clone(),
                    i,
                );
                gates.push(opt_gate);
            }
        }

        // Build dependencies based on qubit usage
        for i in 0..gates.len() {
            let mut deps = Vec::new();
            let gate_qubits: HashSet<_> = gates[i].qubits.iter().collect();

            // Find all previous gates that use the same qubits
            for j in 0..i {
                let prev_qubits: HashSet<_> = gates[j].qubits.iter().collect();

                // If gates share qubits and don't commute, add dependency
                if !gate_qubits.is_disjoint(&prev_qubits) && !gates[i].commutes_with(&gates[j]) {
                    deps.push(j);
                    reverse_dependencies
                        .entry(j)
                        .or_insert_with(Vec::new)
                        .push(i);
                }
            }

            dependencies.insert(i, deps);
        }

        DependencyGraph {
            gates,
            dependencies,
            reverse_dependencies,
        }
    }

    /// Get gates that can be executed in parallel at current level
    pub fn get_parallel_gates(&self, executed: &HashSet<usize>) -> Vec<usize> {
        let mut ready_gates = Vec::new();

        for (gate_id, deps) in &self.dependencies {
            if executed.contains(gate_id) {
                continue;
            }

            // Check if all dependencies are satisfied
            if deps.iter().all(|dep| executed.contains(dep)) {
                ready_gates.push(*gate_id);
            }
        }

        ready_gates
    }

    /// Calculate critical path length
    pub fn critical_path_length(&self) -> usize {
        let mut depths = vec![0; self.gates.len()];

        // Topological sort to calculate depths
        for i in 0..self.gates.len() {
            if let Some(deps) = self.dependencies.get(&i) {
                let max_dep_depth = deps.iter().map(|&dep| depths[dep]).max().unwrap_or(0);
                depths[i] = max_dep_depth + 1;
            }
        }

        depths.into_iter().max().unwrap_or(0)
    }
}

/// Gate scheduling algorithm
pub struct GateScheduler {
    /// Hardware topology constraints
    topology: Option<HardwareTopology>,
}

impl GateScheduler {
    /// Create a new gate scheduler
    pub fn new(topology: Option<HardwareTopology>) -> Self {
        GateScheduler { topology }
    }

    /// Schedule gates to minimize circuit depth
    pub fn schedule(&self, circuit: &QuantumCircuit) -> Result<QuantumCircuit> {
        let dep_graph = DependencyGraph::from_circuit(circuit);
        let scheduled_gates = self.asap_schedule(&dep_graph)?;

        // Rebuild circuit with scheduled gates
        self.build_scheduled_circuit(circuit, &scheduled_gates)
    }

    /// As Soon As Possible (ASAP) scheduling
    fn asap_schedule(&self, graph: &DependencyGraph) -> Result<Vec<Vec<usize>>> {
        let mut executed = HashSet::new();
        let mut schedule = Vec::new();

        while executed.len() < graph.gates.len() {
            let ready_gates = graph.get_parallel_gates(&executed);

            if ready_gates.is_empty() {
                return Err(MyQuatError::circuit_error("Circular dependency in circuit"));
            }

            // Group gates that can execute in parallel
            let parallel_level = self.select_parallel_gates(&ready_gates, &graph.gates)?;

            for &gate_id in &parallel_level {
                executed.insert(gate_id);
            }

            schedule.push(parallel_level);
        }

        Ok(schedule)
    }

    /// Select gates that can execute in parallel considering hardware constraints
    fn select_parallel_gates(
        &self,
        ready_gates: &[usize],
        gates: &[OptimizationGate],
    ) -> Result<Vec<usize>> {
        let mut selected = Vec::new();
        let mut used_qubits = HashSet::new();

        for &gate_id in ready_gates {
            let gate = &gates[gate_id];
            let gate_qubits: HashSet<_> = gate.qubits.iter().collect();

            // Check if gate conflicts with already selected gates
            if gate_qubits.is_disjoint(&used_qubits) {
                // Check hardware constraints if topology is specified
                if let Some(ref topology) = self.topology {
                    if !self.satisfies_hardware_constraints(gate, topology) {
                        continue;
                    }
                }

                selected.push(gate_id);
                used_qubits.extend(gate.qubits.iter());
            }
        }

        Ok(selected)
    }

    /// Check if gate satisfies hardware constraints
    fn satisfies_hardware_constraints(
        &self,
        gate: &OptimizationGate,
        topology: &HardwareTopology,
    ) -> bool {
        if gate.qubits.len() == 2 {
            topology.are_connected(gate.qubits[0], gate.qubits[1])
        } else {
            true // Single-qubit gates are always allowed
        }
    }

    /// Build circuit from scheduled gates
    fn build_scheduled_circuit(
        &self,
        original: &QuantumCircuit,
        schedule: &[Vec<usize>],
    ) -> Result<QuantumCircuit> {
        let mut new_circuit = QuantumCircuit::new(original.num_qubits(), original.num_clbits());
        let original_instructions = original.data().instructions();

        for level in schedule {
            for &gate_id in level {
                if let Some(instruction) = original_instructions.get(gate_id) {
                    if !instruction.is_measurement() {
                        // Apply the gate to the new circuit
                        match instruction.gate.gate_type {
                            StandardGate::H => {
                                if instruction.qubits.len() == 1 {
                                    new_circuit.h(instruction.qubits[0].index())?;
                                }
                            }
                            StandardGate::X => {
                                if instruction.qubits.len() == 1 {
                                    new_circuit.x(instruction.qubits[0].index())?;
                                }
                            }
                            StandardGate::CX if instruction.qubits.len() == 2 => {
                                new_circuit.cx(
                                    instruction.qubits[0].index(),
                                    instruction.qubits[1].index(),
                                )?;
                            }
                            _ => {
                                // Add more gate types as needed
                            }
                        }
                    }
                }
            }
        }

        Ok(new_circuit)
    }
}

/// Commutation analysis for gate reordering
pub struct CommutationAnalyzer;

impl CommutationAnalyzer {
    /// Find all pairs of commuting gates in a circuit
    pub fn find_commuting_pairs(circuit: &QuantumCircuit) -> Vec<(usize, usize)> {
        let mut commuting_pairs = Vec::new();
        let gates: Vec<_> = circuit
            .data()
            .instructions()
            .iter()
            .enumerate()
            .filter_map(|(i, instruction)| {
                if !instruction.is_measurement() {
                    let qubits: Vec<usize> = instruction.qubits.iter().map(|q| q.index()).collect();
                    Some(OptimizationGate::new(
                        instruction.gate.gate_type,
                        qubits,
                        instruction.gate.parameters.clone(),
                        i,
                    ))
                } else {
                    None
                }
            })
            .collect();

        for i in 0..gates.len() {
            for j in (i + 1)..gates.len() {
                if gates[i].commutes_with(&gates[j]) {
                    commuting_pairs.push((i, j));
                }
            }
        }

        commuting_pairs
    }

    /// Reorder gates to minimize depth using commutation rules
    pub fn reorder_for_depth(circuit: &QuantumCircuit) -> Result<QuantumCircuit> {
        // Simplified reordering - full implementation would use more sophisticated algorithms
        Ok(circuit.clone())
    }
}

/// Hardware mapping for quantum circuits
pub struct HardwareMapper {
    /// Target hardware topology
    topology: HardwareTopology,
}

impl HardwareMapper {
    /// Create a new hardware mapper
    pub fn new(topology: HardwareTopology) -> Self {
        HardwareMapper { topology }
    }

    /// Map logical qubits to physical qubits
    pub fn map_circuit(&self, circuit: &QuantumCircuit) -> Result<(QuantumCircuit, QubitMapping)> {
        let mapping = self.find_initial_mapping(circuit)?;
        let mapped_circuit = self.apply_mapping(circuit, &mapping)?;

        Ok((mapped_circuit, mapping))
    }

    /// Find initial qubit mapping using graph algorithms
    fn find_initial_mapping(&self, circuit: &QuantumCircuit) -> Result<QubitMapping> {
        let logical_qubits = circuit.num_qubits();

        if logical_qubits > self.topology.num_qubits {
            return Err(MyQuatError::circuit_error("Not enough physical qubits"));
        }

        // Simple mapping - assign logical qubits to first N physical qubits
        let mut mapping = QubitMapping::new();
        for i in 0..logical_qubits {
            mapping.map_qubit(i, i);
        }

        Ok(mapping)
    }

    /// Apply qubit mapping to circuit
    fn apply_mapping(
        &self,
        circuit: &QuantumCircuit,
        mapping: &QubitMapping,
    ) -> Result<QuantumCircuit> {
        let mut mapped_circuit =
            QuantumCircuit::new(self.topology.num_qubits, circuit.num_clbits());

        for instruction in circuit.data().instructions() {
            if !instruction.is_measurement() {
                let mapped_qubits: std::result::Result<Vec<_>, MyQuatError> = instruction
                    .qubits
                    .iter()
                    .map(|q| {
                        mapping.get_physical_qubit(q.index()).ok_or_else(|| {
                            MyQuatError::circuit_error(format!(
                                "No mapping for logical qubit {}",
                                q.index()
                            ))
                        })
                    })
                    .collect();

                let mapped_qubits = mapped_qubits?;

                // Check if mapped gate satisfies hardware constraints
                if mapped_qubits.len() == 2
                    && !self
                        .topology
                        .are_connected(mapped_qubits[0], mapped_qubits[1])
                {
                    // Need to insert SWAP gates - simplified implementation
                    return Err(MyQuatError::circuit_error(
                        "Gate requires non-adjacent qubits - SWAP insertion needed",
                    ));
                }

                // Apply gate with mapped qubits (simplified)
                match instruction.gate.gate_type {
                    StandardGate::H => {
                        if mapped_qubits.len() == 1 {
                            mapped_circuit.h(mapped_qubits[0])?;
                        }
                    }
                    StandardGate::X => {
                        if mapped_qubits.len() == 1 {
                            mapped_circuit.x(mapped_qubits[0])?;
                        }
                    }
                    StandardGate::CX if mapped_qubits.len() == 2 => {
                        mapped_circuit.cx(mapped_qubits[0], mapped_qubits[1])?;
                    }
                    _ => {
                        // Add more gate types as needed
                    }
                }
            }
        }

        Ok(mapped_circuit)
    }
}

/// Qubit mapping between logical and physical qubits
#[derive(Debug, Clone)]
pub struct QubitMapping {
    /// Logical to physical qubit mapping
    logical_to_physical: HashMap<usize, usize>,
    /// Physical to logical qubit mapping
    physical_to_logical: HashMap<usize, usize>,
}

impl Default for QubitMapping {
    fn default() -> Self {
        Self::new()
    }
}

impl QubitMapping {
    /// Create a new empty mapping
    pub fn new() -> Self {
        QubitMapping {
            logical_to_physical: HashMap::new(),
            physical_to_logical: HashMap::new(),
        }
    }

    /// Map a logical qubit to a physical qubit
    pub fn map_qubit(&mut self, logical: usize, physical: usize) {
        self.logical_to_physical.insert(logical, physical);
        self.physical_to_logical.insert(physical, logical);
    }

    /// Get physical qubit for logical qubit
    pub fn get_physical_qubit(&self, logical: usize) -> Option<usize> {
        self.logical_to_physical.get(&logical).copied()
    }

    /// Get logical qubit for physical qubit
    pub fn get_logical_qubit(&self, physical: usize) -> Option<usize> {
        self.physical_to_logical.get(&physical).copied()
    }
}

/// Gate fusion optimization
pub struct GateFusion;

impl GateFusion {
    /// Fuse compatible single-qubit gates on each qubit using U3 decomposition.
    ///
    /// Consecutive single-qubit gates on the same qubit are multiplied together
    /// and decomposed back into at most 3 rotations (Rz-Ry-Rz).
    /// Uses U3 decomposition — no global phase issue (unlike ZYZ).
    pub fn fuse_single_qubit_gates(circuit: &QuantumCircuit) -> Result<QuantumCircuit> {
        use crate::gate_decomposition::u3_decomposition;
        use ndarray::Array2;
        use num_complex::Complex64;

        let instructions = circuit.data().instructions();
        let num_qubits = circuit.num_qubits();
        let mut result = QuantumCircuit::new(num_qubits, circuit.num_clbits());

        // Per-qubit accumulated fused matrix + stored gate metadata.
        // When count=1, reconstruct via copy_instruction_to_circuit.
        // When count>1, decompose the fused matrix via U3.
        #[derive(Clone)]
        struct StoredGate {
            gate_type: StandardGate,
            qubit: usize,
            params: Vec<crate::parameter::Parameter>,
        }

        let mut accumulators: Vec<Option<(Array2<Complex64>, StoredGate)>> = vec![None; num_qubits];
        let mut accum_count: Vec<usize> = vec![0; num_qubits];

        let emit_fused = |circuit: &mut QuantumCircuit,
                          qubit: usize,
                          matrix: &Array2<Complex64>|
         -> Result<()> {
            let eps = 1e-10;
            // Phase 9m: Check if matrix is diagonal (pure Z rotation).
            // For Trotter circuits, fusion yields diagonal matrices — use
            // direct Rz emission to avoid ZYZ introducing spurious Ry gates.
            let is_diagonal = matrix[[0, 1]].norm() < eps && matrix[[1, 0]].norm() < eps;
            if is_diagonal {
                // U = [[e^{-iθ/2}, 0], [0, e^{iθ/2}]] = Rz(θ)
                // Extract θ from diagonal entries: u00/u11 = e^{-iθ}
                let u00 = matrix[[0, 0]];
                let u11 = matrix[[1, 1]];
                // θ = -i * ln(u00/u11) = arg(u11) - arg(u00)
                let theta = u11.arg() - u00.arg();
                if theta.abs() > eps {
                    circuit.rz(qubit, crate::parameter::Parameter::Float(theta))?;
                }
                return Ok(());
            }
            let angles = u3_decomposition(matrix)?;
            // U3 decomposition: U = Rz(phi) · Ry(theta) · Rz(lambda).
            // No global phase — U3 exactly covers all of U(2).
            if angles.lambda.abs() > eps {
                circuit.rz(qubit, crate::parameter::Parameter::Float(angles.lambda))?;
            }
            if angles.theta.abs() > eps {
                circuit.ry(qubit, crate::parameter::Parameter::Float(angles.theta))?;
            }
            if angles.phi.abs() > eps {
                circuit.rz(qubit, crate::parameter::Parameter::Float(angles.phi))?;
            }
            Ok(())
        };

        let flush = |circuit: &mut QuantumCircuit,
                     accumulators: &mut [Option<(Array2<Complex64>, StoredGate)>],
                     accum_count: &mut [usize]|
         -> Result<()> {
            for q in 0..accumulators.len() {
                if let Some((ref mat, ref stored)) = accumulators[q] {
                    if accum_count[q] == 1 {
                        // Single gate: reconstruct via gate-specific method
                        let gate = crate::gates::Gate {
                            gate_type: stored.gate_type,
                            parameters: stored.params.clone(),
                            label: None,
                        };
                        crate::circuit_optimization::copy_instruction_to_circuit(
                            circuit,
                            &gate,
                            &[stored.qubit],
                        )?;
                    } else if accum_count[q] > 1 {
                        emit_fused(circuit, q, mat)?;
                    }
                }
                accumulators[q] = None;
                accum_count[q] = 0;
            }
            Ok(())
        };

        let mut i = 0;
        while i < instructions.len() {
            let inst = &instructions[i];

            if inst.is_measurement() || inst.qubits.len() != 1 {
                flush(&mut result, &mut accumulators, &mut accum_count)?;
                let qubits_vec: Vec<usize> = inst.qubits.iter().map(|qb| qb.index()).collect();
                if inst.is_measurement() {
                    if !inst.clbits.is_empty() {
                        result.measure(qubits_vec[0], inst.clbits[0].index())?;
                    }
                } else {
                    crate::circuit_optimization::copy_instruction_to_circuit(
                        &mut result,
                        &inst.gate,
                        &qubits_vec,
                    )?;
                }
                i += 1;
                continue;
            }

            let q = inst.qubits[0].index();
            let gate_matrix =
                Self::single_qubit_matrix(&inst.gate.gate_type, &inst.gate.parameters);

            if let Some(mat) = gate_matrix {
                if let Some((ref mut acc, _)) = accumulators[q] {
                    *acc = mat.dot(&*acc);
                } else {
                    let stored = StoredGate {
                        gate_type: inst.gate.gate_type,
                        qubit: q,
                        params: inst.gate.parameters.clone(),
                    };
                    accumulators[q] = Some((mat, stored));
                }
                accum_count[q] += 1;
            } else {
                // Unknown single-qubit gate: flush all, then emit via copy
                flush(&mut result, &mut accumulators, &mut accum_count)?;
                let qubits_vec: Vec<usize> = inst.qubits.iter().map(|qb| qb.index()).collect();
                crate::circuit_optimization::copy_instruction_to_circuit(
                    &mut result,
                    &inst.gate,
                    &qubits_vec,
                )?;
            }
            i += 1;
        }

        flush(&mut result, &mut accumulators, &mut accum_count)?;
        Ok(result)
    }

    /// Get 2x2 matrix for a single-qubit gate type.
    ///
    /// `params` is the full parameter slice from `inst.gate.parameters`.
    /// Multi-parameter gates (U2, U3) access `params[1]`, `params[2]` directly.
    fn single_qubit_matrix(
        gate_type: &StandardGate,
        params: &[crate::parameter::Parameter],
    ) -> Option<ndarray::Array2<num_complex::Complex64>> {
        use ndarray::Array2;
        use num_complex::Complex64;
        use std::f64::consts::PI;

        let get_float = |idx: usize, default: f64| -> f64 {
            params
                .get(idx)
                .and_then(|p| match p {
                    crate::parameter::Parameter::Float(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or(default)
        };

        let c = Complex64::new;

        Some(match gate_type {
            StandardGate::H => {
                let inv_sqrt2 = 1.0 / (2.0f64).sqrt();
                Array2::from_shape_vec(
                    (2, 2),
                    vec![
                        c(inv_sqrt2, 0.0),
                        c(inv_sqrt2, 0.0),
                        c(inv_sqrt2, 0.0),
                        c(-inv_sqrt2, 0.0),
                    ],
                )
                .unwrap()
            }
            StandardGate::X => Array2::from_shape_vec(
                (2, 2),
                vec![c(0.0, 0.0), c(1.0, 0.0), c(1.0, 0.0), c(0.0, 0.0)],
            )
            .unwrap(),
            StandardGate::Y => Array2::from_shape_vec(
                (2, 2),
                vec![c(0.0, 0.0), c(0.0, -1.0), c(0.0, 1.0), c(0.0, 0.0)],
            )
            .unwrap(),
            StandardGate::Z => Array2::from_shape_vec(
                (2, 2),
                vec![c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(-1.0, 0.0)],
            )
            .unwrap(),
            StandardGate::S => Array2::from_shape_vec(
                (2, 2),
                vec![c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(0.0, 1.0)],
            )
            .unwrap(),
            StandardGate::Sdg => Array2::from_shape_vec(
                (2, 2),
                vec![c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(0.0, -1.0)],
            )
            .unwrap(),
            StandardGate::T => {
                let re = (PI / 4.0).cos();
                let im = (PI / 4.0).sin();
                Array2::from_shape_vec(
                    (2, 2),
                    vec![c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(re, im)],
                )
                .unwrap()
            }
            StandardGate::Tdg => {
                let re = (PI / 4.0).cos();
                let im = -(PI / 4.0).sin();
                Array2::from_shape_vec(
                    (2, 2),
                    vec![c(1.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(re, im)],
                )
                .unwrap()
            }
            StandardGate::Rx => {
                let th = get_float(0, 0.0) / 2.0;
                Array2::from_shape_vec(
                    (2, 2),
                    vec![
                        c(th.cos(), 0.0),
                        c(0.0, -th.sin()),
                        c(0.0, -th.sin()),
                        c(th.cos(), 0.0),
                    ],
                )
                .unwrap()
            }
            StandardGate::Ry => {
                let th = get_float(0, 0.0) / 2.0;
                Array2::from_shape_vec(
                    (2, 2),
                    vec![
                        c(th.cos(), 0.0),
                        c(-th.sin(), 0.0),
                        c(th.sin(), 0.0),
                        c(th.cos(), 0.0),
                    ],
                )
                .unwrap()
            }
            StandardGate::Rz => {
                let th = get_float(0, 0.0) / 2.0;
                Array2::from_shape_vec(
                    (2, 2),
                    vec![
                        c(th.cos(), -th.sin()),
                        c(0.0, 0.0),
                        c(0.0, 0.0),
                        c(th.cos(), th.sin()),
                    ],
                )
                .unwrap()
            }
            StandardGate::P | StandardGate::U1 => {
                let lam = get_float(0, 0.0);
                Array2::from_shape_vec(
                    (2, 2),
                    vec![
                        c(1.0, 0.0),
                        c(0.0, 0.0),
                        c(0.0, 0.0),
                        c(lam.cos(), lam.sin()),
                    ],
                )
                .unwrap()
            }
            // U2(φ, λ) = 1/√2 · [[1, -e^{iλ}], [e^{iφ}, e^{i(φ+λ)}]]
            StandardGate::U2 => {
                let phi = get_float(0, 0.0);
                let lam = get_float(1, 0.0);
                let inv_sqrt2 = 1.0 / (2.0f64).sqrt();
                Array2::from_shape_vec(
                    (2, 2),
                    vec![
                        c(inv_sqrt2, 0.0),
                        c(-inv_sqrt2 * lam.cos(), -inv_sqrt2 * lam.sin()),
                        c(inv_sqrt2 * phi.cos(), inv_sqrt2 * phi.sin()),
                        c(inv_sqrt2 * (phi + lam).cos(), inv_sqrt2 * (phi + lam).sin()),
                    ],
                )
                .unwrap()
            }
            // U3(θ, φ, λ) = Rz(φ)·Ry(θ)·Rz(λ)
            StandardGate::U3 => {
                let theta = get_float(0, 0.0);
                let phi = get_float(1, 0.0);
                let lam = get_float(2, 0.0);
                let t2 = theta / 2.0;
                let ct = t2.cos();
                let st = t2.sin();
                Array2::from_shape_vec(
                    (2, 2),
                    vec![
                        c(ct, 0.0),
                        c(-st * lam.cos(), -st * lam.sin()),
                        c(st * phi.cos(), st * phi.sin()),
                        c(ct * (phi + lam).cos(), ct * (phi + lam).sin()),
                    ],
                )
                .unwrap()
            }
            _ => return None,
        })
    }

    /// Fuse consecutive two-qubit gates of the same type on the same qubits.
    ///
    /// Merges: CX·CX → ∅, CZ·CZ → ∅, CRx(θ₁)·CRx(θ₂) → CRx(θ₁+θ₂), etc.
    /// For self-inverse gates (CX, CZ, SWAP), two consecutive identical gates cancel.
    pub fn fuse_two_qubit_gates(circuit: &QuantumCircuit) -> Result<QuantumCircuit> {
        let instructions = circuit.data().instructions();
        let num_qubits = circuit.num_qubits();
        let mut result = QuantumCircuit::new(num_qubits, circuit.num_clbits());

        let is_self_inverse = |gt: &StandardGate| -> bool {
            matches!(
                gt,
                StandardGate::CX
                    | StandardGate::CZ
                    | StandardGate::CY
                    | StandardGate::CH
                    | StandardGate::Swap
                    | StandardGate::ISwap
            )
        };

        let is_mergeable_rotation = |gt: &StandardGate| -> bool {
            matches!(
                gt,
                StandardGate::CRx | StandardGate::CRy | StandardGate::CRz | StandardGate::CP
            )
        };

        let mut i = 0;
        while i < instructions.len() {
            let inst = &instructions[i];

            if inst.is_measurement() || inst.qubits.len() != 2 {
                result.data_mut().add_instruction(inst.clone())?;
                i += 1;
                continue;
            }

            // Look ahead for an identical gate type on the same qubits
            if i + 1 < instructions.len() {
                let next = &instructions[i + 1];
                if !next.is_measurement()
                    && next.qubits == inst.qubits
                    && next.gate.gate_type == inst.gate.gate_type
                {
                    let gt = &inst.gate.gate_type;

                    if is_self_inverse(gt) {
                        // CX·CX → ∅, CZ·CZ → ∅, etc.
                        i += 2;
                        continue;
                    }

                    if is_mergeable_rotation(gt) {
                        // CRx(θ₁)·CRx(θ₂) → CRx(θ₁+θ₂)
                        let v1 = inst
                            .gate
                            .parameters
                            .first()
                            .and_then(|p| match p {
                                crate::parameter::Parameter::Float(v) => Some(*v),
                                _ => None,
                            })
                            .unwrap_or(0.0);
                        let v2 = next
                            .gate
                            .parameters
                            .first()
                            .and_then(|p| match p {
                                crate::parameter::Parameter::Float(v) => Some(*v),
                                _ => None,
                            })
                            .unwrap_or(0.0);
                        let sum = v1 + v2;
                        let q0 = inst.qubits[0].index();
                        let q1 = inst.qubits[1].index();
                        match gt {
                            StandardGate::CRx => {
                                result.crx(q0, q1, crate::parameter::Parameter::Float(sum))?
                            }
                            StandardGate::CRy => {
                                result.cry(q0, q1, crate::parameter::Parameter::Float(sum))?
                            }
                            StandardGate::CRz => {
                                result.crz(q0, q1, crate::parameter::Parameter::Float(sum))?
                            }
                            StandardGate::CP => {
                                result.cp(q0, q1, crate::parameter::Parameter::Float(sum))?
                            }
                            _ => {}
                        }
                        i += 2;
                        continue;
                    }
                }
            }

            result.data_mut().add_instruction(inst.clone())?;
            i += 1;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_graph_creation() {
        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.x(0).unwrap();

        let dep_graph = DependencyGraph::from_circuit(&circuit);
        assert_eq!(dep_graph.gates.len(), 3);

        // CX should depend on H, X should depend on CX
        assert!(dep_graph.dependencies.get(&1).unwrap().contains(&0));
        assert!(dep_graph.dependencies.get(&2).unwrap().contains(&1));
    }

    #[test]
    fn test_gate_scheduler() {
        let mut circuit = QuantumCircuit::new(3, 0);
        circuit.h(0).unwrap();
        circuit.h(1).unwrap();
        circuit.h(2).unwrap();

        let scheduler = GateScheduler::new(None);
        let scheduled = scheduler.schedule(&circuit).unwrap();

        // All H gates should be parallelizable
        assert_eq!(scheduled.num_qubits(), circuit.num_qubits());
    }

    #[test]
    fn test_commutation_analyzer() {
        let mut circuit = QuantumCircuit::new(3, 0);
        circuit.h(0).unwrap();
        circuit.h(1).unwrap();
        circuit.x(2).unwrap();

        let pairs = CommutationAnalyzer::find_commuting_pairs(&circuit);

        // All gates on different qubits should commute
        assert!(pairs.len() >= 2);
    }

    #[test]
    fn test_hardware_mapper() {
        let topology = HardwareTopology::linear(3);
        let mapper = HardwareMapper::new(topology);

        let mut circuit = QuantumCircuit::new(2, 0);
        circuit.h(0).unwrap();
        circuit.cx(0, 1).unwrap();

        let result = mapper.map_circuit(&circuit);
        assert!(result.is_ok());
    }

    #[test]
    fn test_qubit_mapping() {
        let mut mapping = QubitMapping::new();
        mapping.map_qubit(0, 2);
        mapping.map_qubit(1, 3);

        assert_eq!(mapping.get_physical_qubit(0), Some(2));
        assert_eq!(mapping.get_logical_qubit(2), Some(0));
    }

    #[test]
    fn test_gate_fusion() {
        let mut circuit = QuantumCircuit::new(1, 0);
        circuit.x(0).unwrap();
        circuit.y(0).unwrap();
        circuit.z(0).unwrap();

        let fused = GateFusion::fuse_single_qubit_gates(&circuit).unwrap();
        assert_eq!(fused.num_qubits(), circuit.num_qubits());
    }

    /// GateFusion should preserve unitary for interleaved single- and multi-qubit gates.
    #[test]
    fn test_gate_fusion_preserves_interleaved() {
        use crate::parameter::Parameter;
        use std::collections::HashMap;

        let mut circuit = QuantumCircuit::new(2, 0);
        // Trotter-like pattern: basis, CX, Rz, CX, inverse basis
        circuit.h(0).unwrap();
        circuit.h(1).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.rz(0, Parameter::Float(0.5)).unwrap();
        circuit.rz(1, Parameter::Float(0.3)).unwrap();
        circuit.cx(0, 1).unwrap();
        circuit.h(0).unwrap();
        circuit.h(1).unwrap();

        let u_before = circuit.unitary(&HashMap::new()).unwrap();
        let fused = GateFusion::fuse_single_qubit_gates(&circuit).unwrap();
        let u_after = fused.unitary(&HashMap::new()).unwrap();

        let n = u_before.nrows();
        let mut diff: f64 = 0.0;
        for i in 0..n {
            for j in 0..n {
                diff += (u_before[(i, j)] - u_after[(i, j)]).norm_sqr();
            }
        }
        diff = diff.sqrt();
        assert!(
            diff < 1e-10,
            "GateFusion corrupted interleaved circuit: diff={:.2e}, size {}→{}",
            diff,
            circuit.size(),
            fused.size()
        );
    }

    /// Direct test: fuse_single_qubit_gates on compiled H2_4q circuit, check unitary.
    #[test]
    fn test_gate_fusion_on_h2_4q_raw() {
        use crate::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
        use crate::hamiltonian::{
            CompilationStrategy, CompilerConfig, GadgetOptimizationStrategy, Hamiltonian,
            HamiltonianCompiler, PauliString, TrotterOrder,
        };
        use std::collections::HashMap;

        let mut h = Hamiltonian::new(4);
        let terms: [(&str, f64); 15] = [
            ("IIII", -0.8105),
            ("IIIZ", 0.1721),
            ("IIZI", -0.2228),
            ("IZII", 0.1721),
            ("ZIII", -0.2228),
            ("IIZZ", 0.1686),
            ("IZIZ", 0.1205),
            ("IZZI", 0.1686),
            ("ZIIZ", 0.1686),
            ("ZIZI", 0.1205),
            ("ZZII", 0.1686),
            ("IIXX", 0.0454),
            ("IIYY", 0.0454),
            ("XXII", 0.0454),
            ("YYII", 0.0454),
        ];
        for (ps, coeff) in &terms {
            h.add_term(
                PauliString::from_str(ps).unwrap(),
                num_complex::Complex64::new(*coeff, 0.0),
            )
            .unwrap();
        }
        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 10,
            evolution_time: 1.0,
            adaptive: false,
            adaptive_tolerance: 1e-3,
            min_step_size: 1e-6,
            max_step_size: 1.0,
            hbar: 1.0,
            skip_identities: true,
            group_commuting_terms: true,
            apply_circuit_optimization: false,
            auto_optimize_grouping: true,
            layout_aware_grouping: false,
            optimization_strategy: CompilationStrategy::PauliLevel,
            cross_step_synthesis: false,
            block_grouping_strategy: BlockGroupingStrategy::QWC,
            pauli_gadget_optimization: GadgetOptimizationStrategy::IdenticalOnly,
            alternate_reverse_steps: true,
            clifford_enhanced_blocks: false,
        };
        let c = HamiltonianCompiler::new(config).compile(&h).unwrap();
        let u_before = c.unitary(&HashMap::new()).unwrap();
        let original_size = c.size();

        let fused = GateFusion::fuse_single_qubit_gates(&c).unwrap();
        let u_after = fused.unitary(&HashMap::new()).unwrap();

        let n = u_before.nrows();
        let mut max_diff = 0.0f64;
        for i in 0..n {
            for j in 0..n {
                let d = (u_before[(i, j)] - u_after[(i, j)]).norm();
                if d > max_diff {
                    max_diff = d;
                }
            }
        }
        assert!(
            max_diff < 1e-8,
            "fuse_single_qubit_gates corrupted H2_4q: max_diff={:.2e}, size {}→{}",
            max_diff,
            original_size,
            fused.size()
        );
    }

    /// GateFusion must preserve fidelity on circuits pre-processed by SQOpt.
    #[test]
    fn test_gate_fusion_after_sqopt_preserves_fidelity() {
        use crate::hamiltonian::pauli_synthesis::BlockGroupingStrategy;
        use crate::hamiltonian::{
            CompilationStrategy, CompilerConfig, GadgetOptimizationStrategy, Hamiltonian,
            HamiltonianCompiler, PauliString, TrotterOrder,
        };
        use std::collections::HashMap;

        let mut h = Hamiltonian::new(4);
        let terms: [(&str, f64); 15] = [
            ("IIII", -0.8105),
            ("IIIZ", 0.1721),
            ("IIZI", -0.2228),
            ("IZII", 0.1721),
            ("ZIII", -0.2228),
            ("IIZZ", 0.1686),
            ("IZIZ", 0.1205),
            ("IZZI", 0.1686),
            ("ZIIZ", 0.1686),
            ("ZIZI", 0.1205),
            ("ZZII", 0.1686),
            ("IIXX", 0.0454),
            ("IIYY", 0.0454),
            ("XXII", 0.0454),
            ("YYII", 0.0454),
        ];
        for (ps, coeff) in &terms {
            h.add_term(
                PauliString::from_str(ps).unwrap(),
                num_complex::Complex64::new(*coeff, 0.0),
            )
            .unwrap();
        }
        let config = CompilerConfig {
            trotter_order: TrotterOrder::First,
            trotter_steps: 10,
            evolution_time: 1.0,
            hbar: 1.0,
            skip_identities: true,
            group_commuting_terms: true,
            apply_circuit_optimization: false,
            auto_optimize_grouping: true,
            layout_aware_grouping: false,
            optimization_strategy: CompilationStrategy::PauliLevel,
            cross_step_synthesis: false,
            block_grouping_strategy: BlockGroupingStrategy::QWC,
            pauli_gadget_optimization: GadgetOptimizationStrategy::IdenticalOnly,
            alternate_reverse_steps: true,
            clifford_enhanced_blocks: false,
            adaptive: false,
            adaptive_tolerance: 1e-3,
            min_step_size: 1e-6,
            max_step_size: 1.0,
        };
        let mut c = HamiltonianCompiler::new(config).compile(&h).unwrap();

        // SQOpt first (simulates pipeline ordering)
        use crate::circuit_optimization::CircuitPass;
        use crate::single_qubit_optimizer::SingleQubitOptimizer;
        SingleQubitOptimizer::new().run(&mut c).unwrap();

        let u_before = c.unitary(&HashMap::new()).unwrap();
        let fused = GateFusion::fuse_single_qubit_gates(&c).unwrap();
        let u_after = fused.unitary(&HashMap::new()).unwrap();

        let d = u_before.nrows();
        let mut tr = num_complex::Complex64::new(0.0, 0.0);
        for i in 0..d {
            for j in 0..d {
                tr += u_before[(j, i)].conj() * u_after[(j, i)];
            }
        }
        let fid = tr.norm() / d as f64;
        assert!(
            (fid - 1.0).abs() < 1e-10,
            "GateFusion after SQOpt broke fidelity: {:.2e}, size {}→{}",
            fid,
            c.size(),
            fused.size()
        );
    }
}
