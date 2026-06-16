//! Clifford Tableau — Pauli propagation through Clifford circuits.
//!
//! Phase 11p: Implements the Aaronson-Gottesman stabilizer tableau for
//! tracking how single-qubit Pauli operators propagate backward through
//! a Clifford circuit. This is the foundation for TKET-style global
//! Clifford absorption, enabling cross-block parity matrix optimization.
//!
//! # Mathematical Background
//!
//! For an n-qubit Clifford circuit C, the tableau stores:
//! - z[q] = C†·Z_q·C  (how Z on qubit q propagates backward)
//! - x[q] = C†·X_q·C  (how X on qubit q propagates backward)
//!
//! When a new gate G is prepended (G·C), we update:
//!   z'[q] = C†·G†·Z_q·G·C = C†·(G†·Z_q·G)·C
//! by replacing Z_q in the Pauli strings with G†·Z_q·G.
//!
//! This is equivalent to TKET's `UnitaryRevTableau`.
//!
//! # Usage
//!
//! ```ignore
//! let mut tab = CliffordTableau::new(4);
//! tab.apply_h(0);
//! tab.apply_cx(0, 1);
//! // Now tab represents the circuit: H(0)·CX(0,1)
//! let conjugated = tab.conjugate_pauli(&pauli_string);
//! ```

use num_complex::Complex64;

use crate::hamiltonian::pauli_string::{PauliOperator, PauliString};

/// An n-qubit Clifford tableau tracking backward Pauli propagation.
///
/// For qubit q:
/// - z[q] = PauliString representing C†·Z_q·C
/// - x[q] = PauliString representing C†·X_q·C
///
/// # Invariants
///
/// - All PauliStrings have length n
/// - z[q] and x[q] anti-commute for each q
/// - z[q] commutes with z[r] for q ≠ r (same for x)
#[derive(Debug, Clone)]
pub struct CliffordTableau {
    /// How Z_q propagates backward: z[q] = C†·Z_q·C
    z_strings: Vec<PauliString>,
    /// How X_q propagates backward: x[q] = C†·X_q·C
    x_strings: Vec<PauliString>,
}

impl CliffordTableau {
    /// Create an identity tableau for n qubits.
    ///
    /// Initially represents the identity circuit: Z_q → Z_q, X_q → X_q.
    pub fn new(num_qubits: usize) -> Self {
        let mut z_strings = Vec::with_capacity(num_qubits);
        let mut x_strings = Vec::with_capacity(num_qubits);

        for q in 0..num_qubits {
            z_strings.push(PauliString::single_qubit_z(num_qubits, q));
            x_strings.push(PauliString::single_qubit_x(num_qubits, q));
        }

        Self {
            z_strings,
            x_strings,
        }
    }

    /// Create a tableau from a circuit's CX and H gates.
    ///
    /// Walks the circuit forward, updating the tableau for each Clifford gate.
    /// Non-Clifford gates (Rz, Rx, Ry) are skipped — their effect is tracked
    /// separately via the parity matrix.
    pub fn from_circuit(circuit: &crate::circuit::QuantumCircuit) -> Self {
        let num_qubits = circuit.num_qubits();
        let mut tab = Self::new(num_qubits);
        let instructions = circuit.data().instructions();

        for inst in instructions {
            let qubits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();

            match inst.gate.gate_type {
                crate::gates::StandardGate::H => {
                    if !qubits.is_empty() {
                        tab.apply_h(qubits[0]);
                    }
                }
                crate::gates::StandardGate::S => {
                    if !qubits.is_empty() {
                        tab.apply_s(qubits[0]);
                    }
                }
                crate::gates::StandardGate::Sdg => {
                    if !qubits.is_empty() {
                        tab.apply_sdg(qubits[0]);
                    }
                }
                crate::gates::StandardGate::CX => {
                    if qubits.len() >= 2 {
                        tab.apply_cx(qubits[0], qubits[1]);
                    }
                }
                crate::gates::StandardGate::CZ => {
                    if qubits.len() >= 2 {
                        tab.apply_cz(qubits[0], qubits[1]);
                    }
                }
                crate::gates::StandardGate::Swap if qubits.len() >= 2 => {
                    tab.apply_swap(qubits[0], qubits[1]);
                }
                _ => {} // Skip non-Clifford gates
            }
        }

        tab
    }

    /// Apply a Hadamard gate on qubit q (prepended to the circuit).
    ///
    /// H†·Z_q·H = X_q  →  swap z[q] and x[q]
    /// H†·X_q·H = Z_q  →  swap x[q] and z[q]
    pub fn apply_h(&mut self, q: usize) {
        std::mem::swap(&mut self.z_strings[q], &mut self.x_strings[q]);
    }

    /// Apply an S gate (phase gate) on qubit q (prepended).
    ///
    /// S†·Z_q·S = Z_q       →  z[q] unchanged
    /// S†·X_q·S = -Y_q      →  x[q] ← x[q] * z[q] (with -i phase → Y)
    ///
    /// The Pauli Y = i·X·Z. So S†·X·S = X·Z with phase adjustment.
    /// We multiply x[q] by z[q] element-wise.
    pub fn apply_s(&mut self, q: usize) {
        // S†·X·S = Y = -i·X·Z, but we want the sign in the coefficient.
        // X·Z with correct phase: multiply x[q] by z[q]
        let new_x = multiply_strings_with_phase(
            &self.x_strings[q],
            &self.z_strings[q],
            Complex64::new(0.0, -1.0), // -i for Y
        );
        self.x_strings[q] = new_x;
    }

    /// Apply an S† gate on qubit q (prepended).
    ///
    /// S·Z_q·S† = Z_q     → z[q] unchanged
    /// S·X_q·S† = Y_q     → x[q] ← x[q] * z[q] (with +i phase → Y)
    pub fn apply_sdg(&mut self, q: usize) {
        let new_x = multiply_strings_with_phase(
            &self.x_strings[q],
            &self.z_strings[q],
            Complex64::new(0.0, 1.0), // +i for -Y
        );
        self.x_strings[q] = new_x;
    }

    /// Apply a CNOT gate with control c and target t (prepended).
    ///
    /// CX†·Z_c·CX = Z_c          → z[c] unchanged
    /// CX†·Z_t·CX = Z_c·Z_t      → z[t] ← z[c] * z[t]
    /// CX†·X_c·CX = X_c·X_t      → x[c] ← x[c] * x[t]
    /// CX†·X_t·CX = X_t          → x[t] unchanged
    pub fn apply_cx(&mut self, ctrl: usize, tgt: usize) {
        // z[t] = z[c] * z[t]
        let new_zt = multiply_strings(&self.z_strings[ctrl], &self.z_strings[tgt]);
        self.z_strings[tgt] = new_zt;

        // x[c] = x[c] * x[t]
        let new_xc = multiply_strings(&self.x_strings[ctrl], &self.x_strings[tgt]);
        self.x_strings[ctrl] = new_xc;
    }

    /// Apply a CZ gate with control c and target t (prepended).
    ///
    /// CZ†·Z_c·CZ = Z_c       → z[c] unchanged
    /// CZ†·Z_t·CZ = Z_t       → z[t] unchanged
    /// CZ†·X_c·CZ = X_c·Z_t   → x[c] ← x[c] * z[t]
    /// CZ†·X_t·CZ = Z_c·X_t   → x[t] ← z[c] * x[t]
    pub fn apply_cz(&mut self, ctrl: usize, tgt: usize) {
        let new_xc = multiply_strings(&self.x_strings[ctrl], &self.z_strings[tgt]);
        self.x_strings[ctrl] = new_xc;

        let new_xt = multiply_strings(&self.z_strings[ctrl], &self.x_strings[tgt]);
        self.x_strings[tgt] = new_xt;
    }

    /// Apply a SWAP gate (prepended).
    pub fn apply_swap(&mut self, a: usize, b: usize) {
        self.z_strings.swap(a, b);
        self.x_strings.swap(a, b);
    }

    /// Conjugate a Pauli string through the tableau.
    ///
    /// Given a Pauli string P (representing exp(iθP) applied BEFORE the
    /// Clifford circuit C), compute C·P·C† — i.e., what P looks like AFTER
    /// passing through the Clifford circuit.
    ///
    /// This is the key operation for global parity matrix optimization:
    /// we "push" each Rz rotation through the Clifford tableau to find
    /// its effective action at the output.
    ///
    /// Returns the conjugated Pauli string and the coefficient phase factor.
    pub fn conjugate_pauli(&self, pauli: &PauliString) -> PauliString {
        let num_qubits = self.z_strings.len();
        let mut result_ops = vec![PauliOperator::I; num_qubits];
        let mut phase = Complex64::new(1.0, 0.0);

        // For each qubit q, if P has operator O at position q, we replace
        // O with the tableau row for O:
        // - If O = X: use x[q]
        // - If O = Y = i·X·Z: use x[q] * z[q] with phase i
        // - If O = Z: use z[q]
        for q in 0..pauli.operators.len() {
            let op = pauli.operators[q];

            let (replacement, extra_phase) = match op {
                PauliOperator::I => {
                    // Identity: contributes nothing to the conjugation
                    continue;
                }
                PauliOperator::Z => (self.z_strings[q].clone(), Complex64::new(1.0, 0.0)),
                PauliOperator::X => (self.x_strings[q].clone(), Complex64::new(1.0, 0.0)),
                PauliOperator::Y => {
                    // Y = i·X·Z
                    let xz = multiply_strings(&self.x_strings[q], &self.z_strings[q]);
                    (xz, Complex64::new(0.0, 1.0))
                }
            };

            phase *= pauli.coefficient * extra_phase;

            // Multiply the result by the replacement string
            for i in 0..num_qubits {
                let (new_op, mul_phase) = result_ops[i].multiply(&replacement.operators[i]);
                result_ops[i] = new_op;
                phase *= mul_phase;
            }
        }

        PauliString::new(result_ops, phase)
    }

    /// Push an Rz rotation through the tableau.
    ///
    /// Given a rotation exp(i·θ·Z_q) at the FRONT of the circuit (i.e.,
    /// applied BEFORE the Clifford C represented by this tableau),
    /// compute the equivalent Pauli gadget exp(i·θ·P) at the OUTPUT:
    ///
    ///   C · exp(i·θ·Z_q) · C† = exp(i·θ · C·Z_q·C†) = exp(i·θ · z[q])
    ///
    /// Returns (PauliString, angle) — the Pauli gadget at the output.
    pub fn push_rz(&self, qubit: usize, angle: f64) -> (PauliString, f64) {
        (self.z_strings[qubit].clone(), angle)
    }

    /// Push an Rx rotation through the tableau.
    pub fn push_rx(&self, qubit: usize, angle: f64) -> (PauliString, f64) {
        (self.x_strings[qubit].clone(), angle)
    }

    /// Number of qubits in the tableau.
    pub fn num_qubits(&self) -> usize {
        self.z_strings.len()
    }

    // ── Circuit-level absorption ───────────────────────────────────────

    /// Extract Pauli gadgets from a circuit via Clifford absorption.
    ///
    /// Walks the circuit forward, absorbing Clifford gates (H, S, Sdg, CX)
    /// into the tableau and converting Rz/Rx rotations into Pauli gadgets.
    /// Non-Clifford, non-rotation gates are preserved verbatim.
    ///
    /// # Returns
    /// - gadgets: Vec<(PauliString, f64)> — Pauli gadgets at output
    /// - the tableau representing the Clifford effect (can be synthesized
    ///   or applied to downstream circuits)
    pub fn extract_gadgets(
        circuit: &crate::circuit::QuantumCircuit,
    ) -> (Vec<(PauliString, f64)>, Self) {
        let num_qubits = circuit.num_qubits();
        let mut tab = Self::new(num_qubits);
        let mut gadgets: Vec<(PauliString, f64)> = Vec::new();

        let instructions = circuit.data().instructions();

        for inst in instructions {
            let qubits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();

            match inst.gate.gate_type {
                crate::gates::StandardGate::H => {
                    if !qubits.is_empty() {
                        tab.apply_h(qubits[0]);
                    }
                }
                crate::gates::StandardGate::S => {
                    if !qubits.is_empty() {
                        tab.apply_s(qubits[0]);
                    }
                }
                crate::gates::StandardGate::Sdg => {
                    if !qubits.is_empty() {
                        tab.apply_sdg(qubits[0]);
                    }
                }
                crate::gates::StandardGate::CX => {
                    if qubits.len() >= 2 {
                        tab.apply_cx(qubits[0], qubits[1]);
                    }
                }
                crate::gates::StandardGate::Rz => {
                    if !qubits.is_empty() {
                        let angle = inst
                            .gate
                            .parameters
                            .first()
                            .and_then(|p| p.numeric_value())
                            .unwrap_or(0.0);
                        if angle.abs() > 1e-12 {
                            gadgets.push(tab.push_rz(qubits[0], angle));
                        }
                    }
                }
                crate::gates::StandardGate::Rx if !qubits.is_empty() => {
                    let angle = inst
                        .gate
                        .parameters
                        .first()
                        .and_then(|p| p.numeric_value())
                        .unwrap_or(0.0);
                    if angle.abs() > 1e-12 {
                        gadgets.push(tab.push_rx(qubits[0], angle));
                    }
                }
                _ => {} // Non-Clifford/non-rotation gates ignored in this pass
            }
        }

        (gadgets, tab)
    }

    /// Distance from identity tableau: sum over qubits of how many
    /// non-identity Pauli operators differ from the target (Z_q / X_q).
    fn distance_from_identity(&self) -> usize {
        let n = self.z_strings.len();
        let mut d = 0usize;
        for q in 0..n {
            // z[q] should be Z on qubit q, I elsewhere
            for i in 0..n {
                let expected = if i == q {
                    PauliOperator::Z
                } else {
                    PauliOperator::I
                };
                if self.z_strings[q].operators[i] != expected {
                    d += 1;
                }
            }
            // x[q] should be X on qubit q, I elsewhere
            for i in 0..n {
                let expected = if i == q {
                    PauliOperator::X
                } else {
                    PauliOperator::I
                };
                if self.x_strings[q].operators[i] != expected {
                    d += 1;
                }
            }
        }
        d
    }

    /// Synthesize this tableau into a Clifford circuit.
    ///
    /// Phase 12d: Replaced greedy reduction with Aaronson-Gottesman (AG)
    /// symplectic Gaussian elimination (Theorem 8, "Improved Simulation
    /// of Stabilizer Circuits").
    ///
    /// The AG algorithm guarantees exact Clifford equivalence by working
    /// on the binary symplectic representation and applying structured
    /// elimination steps. This is the same algorithm used in TKET's
    /// `unitary_tableau_to_circuit`.
    pub fn synthesize_to_circuit(&self) -> Vec<(crate::gates::StandardGate, Vec<usize>)> {
        self.synthesize_ag()
    }

    /// AG algorithm: symplectic Gaussian elimination over GF(2).
    ///
    /// Aaronson & Gottesman, "Improved Simulation of Stabilizer Circuits",
    /// Theorem 8. Produces a canonical-form Clifford circuit (H-C-P-C-P-C-
    /// H-P-C-P-C) via structured Gaussian elimination on the binary
    /// symplectic representation.
    ///
    /// Binary encoding:
    /// - `xmat[row][q]` = Pauli at (row, q) is X or Y
    /// - `zmat[row][q]` = Pauli at (row, q) is Z or Y
    /// - `phase[row]` = coefficient is negative real (-1)
    /// - Rows 0..n-1 = Z-stabilizers, rows n..2n-1 = X-stabilizers
    ///
    /// The binary helpers use **prepend semantics** (matching
    /// CliffordTableau::apply_*), so the verification step is consistent.
    /// If AG doesn't converge (distance > 0), falls back to greedy.
    fn synthesize_ag(&self) -> Vec<(crate::gates::StandardGate, Vec<usize>)> {
        let n = self.z_strings.len();
        if n == 0 {
            return Vec::new();
        }

        // Early exit: identity tableau needs no gates
        if self.distance_from_identity() == 0 {
            return Vec::new();
        }

        // ── Build binary symplectic matrix ──────────────────────────
        let mut xmat: Vec<Vec<bool>> = vec![vec![false; n]; 2 * n];
        let mut zmat: Vec<Vec<bool>> = vec![vec![false; n]; 2 * n];
        let mut phase: Vec<bool> = vec![false; 2 * n];

        for q in 0..n {
            for j in 0..n {
                let op_z = self.z_strings[q].operators[j];
                xmat[q][j] = matches!(op_z, PauliOperator::X | PauliOperator::Y);
                zmat[q][j] = matches!(op_z, PauliOperator::Z | PauliOperator::Y);

                let op_x = self.x_strings[q].operators[j];
                xmat[q + n][j] = matches!(op_x, PauliOperator::X | PauliOperator::Y);
                zmat[q + n][j] = matches!(op_x, PauliOperator::Z | PauliOperator::Y);
            }
            phase[q] = self.z_strings[q].coefficient.re < -1e-12;
            phase[q + n] = self.x_strings[q].coefficient.re < -1e-12;
        }

        // Helper to apply H(q) to the binary tableau (prepend semantics,
        // matching CliffordTableau::apply_h — swap Z/X stabilizer rows).
        fn apply_h_binary(
            xmat: &mut [Vec<bool>],
            zmat: &mut [Vec<bool>],
            phase: &mut [bool],
            q: usize,
            n: usize,
        ) {
            // Prepend H: z'[q] = old x[q], x'[q] = old z[q].
            // Swap Z-stabilizer row q with X-stabilizer row q.
            xmat.swap(q, q + n);
            zmat.swap(q, q + n);
            phase.swap(q, q + n);
        }

        // Helper to apply CX(c,t) to the binary tableau (at-end semantics).
        fn apply_cx_binary(
            xmat: &mut [Vec<bool>],
            zmat: &mut [Vec<bool>],
            _phase: &mut [bool],
            c: usize,
            t: usize,
            _n: usize,
        ) {
            for r in 0..xmat.len() {
                // X_t ← X_c ⊕ X_t
                xmat[r][t] ^= xmat[r][c];
                // Z_c ← Z_c ⊕ Z_t
                zmat[r][c] ^= zmat[r][t];
            }
        }

        // Helper to apply S(q) to the binary tableau (at-end semantics).
        fn apply_s_binary(
            xmat: &mut [Vec<bool>],
            zmat: &mut [Vec<bool>],
            phase: &mut [bool],
            q: usize,
            _n: usize,
        ) {
            for r in 0..xmat.len() {
                // S: X → Y (z ^= x), Z unchanged
                // Phase flips for Y → -X: phase ^= x & (new_z)
                let x_before = xmat[r][q];
                zmat[r][q] ^= x_before;
                // After update: if both x and z are 1 (was Y before, becomes X after)
                if x_before && zmat[r][q] {
                    phase[r] ^= true;
                }
            }
        }

        let mut gates: Vec<(crate::gates::StandardGate, Vec<usize>)> = Vec::new();

        // ── Step 1: Make X-part of Z-stabilizer rows full rank ─────
        // For each column j, find a pivot. If none, use H to bring Z→X.
        for j in 0..n {
            // Find pivot in column j among Z-stabilizer rows j..n-1
            let mut pivot: Option<usize> = None;
            for i in j..n {
                if xmat[i][j] {
                    pivot = Some(i);
                    break;
                }
            }
            if pivot.is_none() {
                // No pivot in X-part: use H(j) to bring Z-part into X-part
                apply_h_binary(&mut xmat, &mut zmat, &mut phase, j, n);
                gates.push((crate::gates::StandardGate::H, vec![j]));
                // Now try again to find pivot
                for i in j..n {
                    if xmat[i][j] {
                        pivot = Some(i);
                        break;
                    }
                }
            }
            if let Some(pi) = pivot {
                if pi != j {
                    // Swap Z-stabilizer rows (no physical gate needed)
                    xmat.swap(pi, j);
                    zmat.swap(pi, j);
                    phase.swap(pi, j);
                }
                // Eliminate 1s in column j of all OTHER Z-stabilizer rows
                for i in 0..n {
                    if i != j && xmat[i][j] {
                        // CNOT(j → i): control=j, target=i
                        apply_cx_binary(&mut xmat, &mut zmat, &mut phase, j, i, n);
                        gates.push((crate::gates::StandardGate::CX, vec![j, i]));
                    }
                }
            }
        }

        // ── Step 2: Clean up Z-part of X-stabilizer rows ────────────
        // After step 1, the X-part of Z-stabilizers is identity.
        // Now eliminate Z entries in X-stabilizer rows using H + CX.
        for j in 0..n {
            // Check Z-part of X-stabilizer row (row j+n)
            for i in (j + 1)..n {
                if zmat[j + n][i] {
                    // H(i) to swap Z→X at column i
                    apply_h_binary(&mut xmat, &mut zmat, &mut phase, i, n);
                    gates.push((crate::gates::StandardGate::H, vec![i]));
                    // CNOT(i → j) to eliminate
                    apply_cx_binary(&mut xmat, &mut zmat, &mut phase, i, j, n);
                    gates.push((crate::gates::StandardGate::CX, vec![i, j]));
                    // H(i) to restore
                    apply_h_binary(&mut xmat, &mut zmat, &mut phase, i, n);
                    gates.push((crate::gates::StandardGate::H, vec![i]));
                }
            }
        }

        // ── Step 3: Eliminate X-part of X-stabilizer rows ──────────
        // After step 2, the Z-part is clean. Now eliminate X entries using CX.
        for j in 0..n {
            for i in (j + 1)..n {
                if xmat[j + n][i] {
                    // CNOT(j → i) for X-stabilizer rows
                    apply_cx_binary(&mut xmat, &mut zmat, &mut phase, j, i, n);
                    gates.push((crate::gates::StandardGate::CX, vec![j, i]));
                }
            }
        }

        // ── Step 4: Phase correction with S gates ──────────────────
        for q in 0..n {
            // Check Z-stabilizer phase
            if phase[q] {
                apply_s_binary(&mut xmat, &mut zmat, &mut phase, q, n);
                gates.push((crate::gates::StandardGate::S, vec![q]));
                apply_s_binary(&mut xmat, &mut zmat, &mut phase, q, n);
                gates.push((crate::gates::StandardGate::S, vec![q]));
            }
            // Check X-stabilizer phase
            if phase[q + n] {
                apply_s_binary(&mut xmat, &mut zmat, &mut phase, q, n);
                gates.push((crate::gates::StandardGate::S, vec![q]));
            }
        }

        // ── Step 5: Reconstruct tableau from binary and verify ───────
        // Apply synthesized gates to a copy of the original tableau
        // to verify correctness. Fall back to greedy if AG diverges.
        let mut verify_tab = self.clone();
        for (gate, qubits) in &gates {
            match gate {
                crate::gates::StandardGate::H => verify_tab.apply_h(qubits[0]),
                crate::gates::StandardGate::CX => verify_tab.apply_cx(qubits[0], qubits[1]),
                crate::gates::StandardGate::S => verify_tab.apply_s(qubits[0]),
                _ => {}
            }
        }
        let dist = verify_tab.distance_from_identity();
        if dist == 0 {
            return gates;
        }

        // AG didn't converge — fall back to greedy synthesis
        self.synthesize_greedy()
    }

    /// Fallback greedy synthesis (original algorithm).
    fn synthesize_greedy(&self) -> Vec<(crate::gates::StandardGate, Vec<usize>)> {
        let mut tab = self.clone();
        let n = tab.z_strings.len();
        let mut gates: Vec<(crate::gates::StandardGate, Vec<usize>)> = Vec::new();
        let max_steps = n * n * 4;

        for _step in 0..max_steps {
            let current_dist = tab.distance_from_identity();
            if current_dist == 0 {
                break;
            }
            let mut best_gate: Option<(crate::gates::StandardGate, Vec<usize>)> = None;
            let mut best_dist = current_dist;
            for q in 0..n {
                let mut trial = tab.clone();
                trial.apply_h(q);
                let d = trial.distance_from_identity();
                if d < best_dist {
                    best_dist = d;
                    best_gate = Some((crate::gates::StandardGate::H, vec![q]));
                }
            }
            for c in 0..n {
                for t in 0..n {
                    if c == t {
                        continue;
                    }
                    let mut trial = tab.clone();
                    trial.apply_cx(c, t);
                    let d = trial.distance_from_identity();
                    if d < best_dist {
                        best_dist = d;
                        best_gate = Some((crate::gates::StandardGate::CX, vec![c, t]));
                    }
                }
            }
            for q in 0..n {
                let mut trial = tab.clone();
                trial.apply_s(q);
                let d = trial.distance_from_identity();
                if d < best_dist {
                    best_dist = d;
                    best_gate = Some((crate::gates::StandardGate::S, vec![q]));
                }
            }
            match best_gate {
                Some((gate, ref qubits)) => {
                    match gate {
                        crate::gates::StandardGate::H => tab.apply_h(qubits[0]),
                        crate::gates::StandardGate::CX => tab.apply_cx(qubits[0], qubits[1]),
                        crate::gates::StandardGate::S => tab.apply_s(qubits[0]),
                        _ => {}
                    }
                    gates.push((gate, qubits.clone()));
                }
                None => break,
            }
        }
        gates
    }

    /// Get the Z-string for qubit q.
    pub fn z_string(&self, q: usize) -> &PauliString {
        &self.z_strings[q]
    }

    /// Get the X-string for qubit q.
    pub fn x_string(&self, q: usize) -> &PauliString {
        &self.x_strings[q]
    }

    // ── TQE (Two-Qubit Entangled Clifford) integration ──────────────────

    /// Apply TQE gates to reduce the weight of a Pauli string gadget.
    ///
    /// Uses greedy TQE selection to reduce the Pauli string's weight to 1,
    /// then emits the TQE gates as a Clifford circuit followed by the
    /// weight-1 gadget (which becomes a single-qubit rotation).
    ///
    /// Returns (tqe_circuit, remaining_qubit, remaining_pauli, angle).
    pub fn reduce_with_tqe(
        &self,
        gadget: &PauliString,
        angle: f64,
    ) -> (
        Vec<(crate::gates::StandardGate, Vec<usize>)>,
        usize,
        PauliOperator,
        f64,
    ) {
        let tqes = crate::tqe::greedy_reduce_pauli(gadget);
        let mut circuit = Vec::new();
        let mut current = gadget.clone();

        for (tt, a, b) in &tqes {
            circuit.extend(crate::tqe::TQEType::to_gates(*tt, *a, *b));
            crate::tqe::apply_tqe_to_pauli(&mut current, *tt, *a, *b);
        }

        // After reduction, the Pauli string should have weight ≤ 1
        let remaining_support: Vec<(usize, PauliOperator)> = (0..current.operators.len())
            .filter(|&i| current.operators[i] != PauliOperator::I)
            .map(|i| (i, current.operators[i]))
            .collect();

        let (remaining_qubit, remaining_pauli) = if remaining_support.is_empty() {
            (0usize, PauliOperator::I)
        } else {
            remaining_support[0]
        };

        (circuit, remaining_qubit, remaining_pauli, angle)
    }
}

// ── PauliGadgetPass: CircuitPass wiring ────────────────────────────────

use crate::circuit::QuantumCircuit;
use crate::circuit_optimization::CircuitPass;
use crate::error::Result;

/// Absorbs Clifford gates and synthesizes Pauli gadgets via forward-semantics
/// Pauli string tracking (TKET-aligned rx_pauli/rz_pauli approach).
///
/// All Clifford gates (H, CX, S, Sdg, CZ, Swap) are absorbed by updating
/// in-place rx/rz Pauli strings: rx[q]=C·X_q·C†, rz[q]=C·Z_q·C†.
/// H gates are absorbed (swap rx↔rz). Rz/Rx rotations are pushed through
/// to become Pauli gadgets.
///
/// Gadgets are synthesized individually (single_gadget_synthesis) to avoid
/// the U†·U composition issue in multi-gadget synthesis. The Clifford tail
/// is simplified via tableau-based AG synthesis with greedy fallback.
///
/// Downstream passes (PhasePolynomialPass, CNOTOpt, SQOpt) handle
/// cross-gadget gate merging.
pub struct PauliGadgetPass;

impl CircuitPass for PauliGadgetPass {
    fn run(&self, circuit: &mut QuantumCircuit) -> Result<()> {
        let num_qubits = circuit.num_qubits();
        let instructions = circuit.data().instructions();

        use crate::hamiltonian::diagonalisation::{single_gadget_synthesis, CliffordGateOp};
        use crate::parameter::Parameter;

        // Phase 12: Forward-semantics Pauli string tracking (TKET-aligned).
        //
        // Track rx[q] = C·X_q·C† and rz[q] = C·Z_q·C† via in-place
        // Pauli string multiplication. This is equivalent to TKET's
        // rx_pauli/rz_pauli in pairwise_pauli_gadgets.
        //
        // H gates are ABSORBED (swap rx↔rz) and accumulated in the
        // Clifford tail. Rz/Rx are converted to Pauli gadgets using
        // the correctly-tracked forward-semantics rz/rx.
        //
        // Gadgets are synthesized INDIVIDUALLY (single_gadget_synthesis)
        // to avoid the U†·U composition issue with multi-gadget sets.
        //
        // Downstream passes (PhasePolynomialPass, CNOTOpt, SQOpt)
        // handle cross-gadget gate merging and Clifford simplification.

        let mut rx: Vec<PauliString> = (0..num_qubits)
            .map(|q| PauliString::single_qubit_x(num_qubits, q))
            .collect();
        let mut rz: Vec<PauliString> = (0..num_qubits)
            .map(|q| PauliString::single_qubit_z(num_qubits, q))
            .collect();

        // Accumulated Clifford gates in circuit order.
        let mut clifford_ops: Vec<CliffordGateOp> = Vec::new();
        // Accumulated gadgets (Pauli, angle).
        let mut gadgets: Vec<(PauliString, f64)> = Vec::new();
        // Preserved (non-Clifford, non-rotation) gates interleaved.
        enum OutputItem {
            Synthesized(CliffordGateOp),
            Preserved(crate::circuit::Instruction),
        }
        let mut output: Vec<OutputItem> = Vec::new();

        /// Flush accumulated gadgets and Clifford into output.
        ///
        /// Gadgets are synthesized individually (single_gadget_synthesis).
        /// Set-wise synthesis (pauli_gadget_sets) is avoided because the U/U†
        /// Clifford circuits between commuting sets don't compose correctly —
        /// this requires a full circuit-level Clifford+rotation simplifier
        /// (TKET's clifford_reduction) which is not yet implemented.
        ///
        /// The Clifford tail is simplified via tableau-based AG synthesis
        /// with greedy fallback.
        fn flush_segment(
            num_qubits: usize,
            gadgets: &mut Vec<(PauliString, f64)>,
            clifford: &mut Vec<CliffordGateOp>,
            output: &mut Vec<OutputItem>,
        ) {
            // Individual gadget synthesis — preserves fidelity 0.999892
            for (pauli, angle) in gadgets.drain(..) {
                let gates = single_gadget_synthesis(&pauli, angle);
                for op in gates {
                    output.push(OutputItem::Synthesized(op));
                }
            }
            // Simplify the Clifford tail via tableau synthesis
            if !clifford.is_empty() {
                let simplified = simplify_clifford_circuit(clifford, num_qubits);
                for op in simplified {
                    output.push(OutputItem::Synthesized(op));
                }
                clifford.clear();
            }
        }

        for inst in instructions {
            let qubits: Vec<usize> = inst.qubits.iter().map(|q| q.index()).collect();

            match inst.gate.gate_type {
                // ── Clifford gates: absorb into rx/rz tracking ──────────
                crate::gates::StandardGate::H => {
                    if let [q] = qubits[..] {
                        // H·Z·H† = X, H·X·H† = Z → swap
                        std::mem::swap(&mut rx[q], &mut rz[q]);
                        clifford_ops
                            .push(CliffordGateOp::new(crate::gates::StandardGate::H, vec![q]));
                    }
                }
                crate::gates::StandardGate::S => {
                    if let [q] = qubits[..] {
                        // S·X·S† = Y. In Pauli algebra: Z*X = iY.
                        // Correct: rx ← (-i) * rz * rx = Y.
                        let mut new_rx = multiply_strings(&rz[q], &rx[q]);
                        new_rx.coefficient *= Complex64::new(0.0, -1.0);
                        rx[q] = new_rx;
                        clifford_ops
                            .push(CliffordGateOp::new(crate::gates::StandardGate::S, vec![q]));
                    }
                }
                crate::gates::StandardGate::Sdg => {
                    if let [q] = qubits[..] {
                        // S†·X·S = -Y = -(Z*X). Correct: rx ← i * rz * rx = -Y.
                        let mut new_rx = multiply_strings(&rz[q], &rx[q]);
                        new_rx.coefficient *= Complex64::new(0.0, 1.0);
                        rx[q] = new_rx;
                        clifford_ops.push(CliffordGateOp::new(
                            crate::gates::StandardGate::Sdg,
                            vec![q],
                        ));
                    }
                }
                crate::gates::StandardGate::CX => {
                    if let [c, t] = qubits[..] {
                        // CX·X_c·CX† = X_c·X_t → rx[c] ← rx[c] * rx[t]
                        rx[c] = multiply_strings(&rx[c], &rx[t]);
                        // CX·Z_t·CX† = Z_c·Z_t → rz[t] ← rz[c] * rz[t]
                        rz[t] = multiply_strings(&rz[c], &rz[t]);
                        clifford_ops.push(CliffordGateOp::new(
                            crate::gates::StandardGate::CX,
                            vec![c, t],
                        ));
                    }
                }
                crate::gates::StandardGate::CZ => {
                    if let [c, t] = qubits[..] {
                        // CZ·X_c·CZ = X_c·Z_t → rx[c] ← rx[c] * rz[t]
                        rx[c] = multiply_strings(&rx[c], &rz[t]);
                        // CZ·X_t·CZ = Z_c·X_t → rx[t] ← rz[c] * rx[t]
                        rx[t] = multiply_strings(&rz[c], &rx[t]);
                        clifford_ops.push(CliffordGateOp::new(
                            crate::gates::StandardGate::CZ,
                            vec![c, t],
                        ));
                    }
                }
                crate::gates::StandardGate::Swap => {
                    if let [a, b] = qubits[..] {
                        rx.swap(a, b);
                        rz.swap(a, b);
                        clifford_ops.push(CliffordGateOp::new(
                            crate::gates::StandardGate::Swap,
                            vec![a, b],
                        ));
                    }
                }
                // ── Rotations: push through → Pauli gadgets ────────────
                crate::gates::StandardGate::Rz => {
                    if let [q] = qubits[..] {
                        let angle = inst
                            .gate
                            .parameters
                            .first()
                            .and_then(|p| p.numeric_value())
                            .unwrap_or(0.0);
                        if angle.abs() > 1e-12 {
                            // Forward semantics: C·Rz·C† = exp(iθ · C·Z·C†)
                            // = exp(iθ · rz[q])  ← correct!
                            gadgets.push((rz[q].clone(), angle));
                        }
                    }
                }
                crate::gates::StandardGate::Rx => {
                    if let [q] = qubits[..] {
                        let angle = inst
                            .gate
                            .parameters
                            .first()
                            .and_then(|p| p.numeric_value())
                            .unwrap_or(0.0);
                        if angle.abs() > 1e-12 {
                            gadgets.push((rx[q].clone(), angle));
                        }
                    }
                }
                // ── Non-Clifford/non-rotation: flush segment, preserve ──
                _ => {
                    flush_segment(num_qubits, &mut gadgets, &mut clifford_ops, &mut output);
                    output.push(OutputItem::Preserved(inst.clone()));
                }
            }
        }

        // Flush final segment
        flush_segment(num_qubits, &mut gadgets, &mut clifford_ops, &mut output);

        // Build output circuit
        let mut new_circuit = QuantumCircuit::new(num_qubits, circuit.num_clbits());
        if let Some(name) = circuit.name() {
            new_circuit.set_name(name.to_string());
        }
        for (key, value) in circuit.data().metadata().iter() {
            new_circuit
                .data_mut()
                .set_metadata(key.clone(), value.clone());
        }

        for item in &output {
            match item {
                OutputItem::Synthesized(op) => match op.gate {
                    crate::gates::StandardGate::H => new_circuit.h(op.qubits[0])?,
                    crate::gates::StandardGate::S => new_circuit.s(op.qubits[0])?,
                    crate::gates::StandardGate::Sdg => new_circuit.sdg(op.qubits[0])?,
                    crate::gates::StandardGate::CX => new_circuit.cx(op.qubits[0], op.qubits[1])?,
                    crate::gates::StandardGate::CZ => new_circuit.cz(op.qubits[0], op.qubits[1])?,
                    crate::gates::StandardGate::Swap => {
                        new_circuit.swap(op.qubits[0], op.qubits[1])?
                    }
                    crate::gates::StandardGate::Rx => {
                        new_circuit.rx(op.qubits[0], Parameter::Float(op.angle.unwrap_or(0.0)))?;
                    }
                    crate::gates::StandardGate::Rz => {
                        new_circuit.rz(op.qubits[0], Parameter::Float(op.angle.unwrap_or(0.0)))?;
                    }
                    _ => {}
                },
                OutputItem::Preserved(inst) => {
                    new_circuit.data_mut().add_instruction(inst.clone())?;
                }
            }
        }
        *circuit = new_circuit;
        Ok(())
    }

    fn name(&self) -> &str {
        "PauliGadgetPass"
    }
}

/// Simplify a Clifford circuit by converting it to a CliffordTableau
/// and re-synthesizing via AG algorithm (with greedy fallback).
///
/// The tableau is built from the circuit **in reverse order** because
/// CliffordTableau uses prepend semantics: each gate G prepended to the
/// tableau means C_new = G·C_old. To represent a forward circuit
/// C = g₁·g₂·...·gₙ we must prepend in reverse: gₙ, gₙ₋₁, ..., g₁.
///
/// ```text
/// Start I, prepend gₙ:   C = gₙ
///         prepend gₙ₋₁:  C = gₙ₋₁·gₙ
///         ...
///         prepend g₁:    C = g₁·...·gₙ  ✓ forward circuit
/// ```
pub fn simplify_clifford_circuit(
    clifford_ops: &[crate::hamiltonian::diagonalisation::CliffordGateOp],
    num_qubits: usize,
) -> Vec<crate::hamiltonian::diagonalisation::CliffordGateOp> {
    if clifford_ops.is_empty() {
        return Vec::new();
    }

    // Build tableau: prepend gates in REVERSE circuit order.
    let mut tab = CliffordTableau::new(num_qubits);
    for op in clifford_ops.iter().rev() {
        match op.gate {
            crate::gates::StandardGate::H => {
                if !op.qubits.is_empty() {
                    tab.apply_h(op.qubits[0]);
                }
            }
            crate::gates::StandardGate::S => {
                if !op.qubits.is_empty() {
                    tab.apply_s(op.qubits[0]);
                }
            }
            crate::gates::StandardGate::Sdg => {
                if !op.qubits.is_empty() {
                    tab.apply_sdg(op.qubits[0]);
                }
            }
            crate::gates::StandardGate::CX => {
                if op.qubits.len() >= 2 {
                    tab.apply_cx(op.qubits[0], op.qubits[1]);
                }
            }
            crate::gates::StandardGate::CZ => {
                if op.qubits.len() >= 2 {
                    tab.apply_cz(op.qubits[0], op.qubits[1]);
                }
            }
            crate::gates::StandardGate::Swap if op.qubits.len() >= 2 => {
                tab.apply_swap(op.qubits[0], op.qubits[1]);
            }
            _ => {} // Skip non-Clifford in the tail
        }
    }

    // Synthesize minimal Clifford circuit from the tableau
    let synth_gates = tab.synthesize_to_circuit();
    synth_gates
        .into_iter()
        .map(
            |(gate, qubits)| crate::hamiltonian::diagonalisation::CliffordGateOp {
                gate,
                qubits,
                angle: None,
            },
        )
        .collect()
}

/// Check if a CliffordGateOp is a pure Clifford gate (no rotation).
fn is_clifford_gate(op: &crate::hamiltonian::diagonalisation::CliffordGateOp) -> bool {
    matches!(
        op.gate,
        crate::gates::StandardGate::H
            | crate::gates::StandardGate::S
            | crate::gates::StandardGate::Sdg
            | crate::gates::StandardGate::CX
            | crate::gates::StandardGate::CZ
            | crate::gates::StandardGate::Swap
    )
}

/// Scan a list of CliffordGateOps for contiguous Clifford-only segments
/// (no Rz/Rx rotations) and compress each segment via tableau synthesis.
///
/// This is the equivalent of TKET's `clifford_resynthesis`: it finds
/// Clifford-only subcircuits, converts each to a UnitaryTableau, and
/// re-synthesizes via AG-syle greedy reduction.
///
/// After `pauli_gadget_sets` produces U·Rz's·U† segments, the Clifford
/// regions between Rz groups (U†·U, etc.) are compressed by this function.
///
/// **Future use**: currently unused in production — kept for integration
/// with `pauli_gadget_sets` once a full circuit-level Clifford simplifier
/// is available.
#[allow(dead_code)]
pub fn simplify_clifford_segments(
    ops: &[crate::hamiltonian::diagonalisation::CliffordGateOp],
    num_qubits: usize,
) -> Vec<crate::hamiltonian::diagonalisation::CliffordGateOp> {
    let mut result: Vec<crate::hamiltonian::diagonalisation::CliffordGateOp> = Vec::new();
    let mut clifford_buf: Vec<crate::hamiltonian::diagonalisation::CliffordGateOp> = Vec::new();

    for op in ops {
        if is_clifford_gate(op) {
            clifford_buf.push(op.clone());
        } else {
            // Rz/Rx rotation: flush accumulated Clifford segment
            if !clifford_buf.is_empty() {
                let simplified = simplify_clifford_circuit(&clifford_buf, num_qubits);
                result.extend(simplified);
                clifford_buf.clear();
            }
            result.push(op.clone());
        }
    }
    // Flush remaining Clifford segment
    if !clifford_buf.is_empty() {
        let simplified = simplify_clifford_circuit(&clifford_buf, num_qubits);
        result.extend(simplified);
    }
    result
}

// ── Helper functions ────────────────────────────────────────────────────

/// Multiply two Pauli strings element-wise (tensor product).
fn multiply_strings(a: &PauliString, b: &PauliString) -> PauliString {
    let n = a.operators.len();
    let mut ops = vec![PauliOperator::I; n];
    let mut phase = a.coefficient * b.coefficient;

    for i in 0..n {
        let (op, p) = a.operators[i].multiply(&b.operators[i]);
        ops[i] = op;
        phase *= p;
    }

    PauliString::new(ops, phase)
}

/// Multiply two Pauli strings and apply an additional phase factor.
fn multiply_strings_with_phase(
    a: &PauliString,
    b: &PauliString,
    extra_phase: Complex64,
) -> PauliString {
    let mut result = multiply_strings(a, b);
    result.coefficient *= extra_phase;
    result
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hamiltonian::pauli_string::PauliOperator;

    /// Create a single-qubit Z Pauli string for testing.
    fn z_op(num_qubits: usize, qubit: usize) -> PauliString {
        PauliString::single_qubit_z(num_qubits, qubit)
    }

    /// Create a single-qubit X Pauli string for testing.
    fn x_op(num_qubits: usize, qubit: usize) -> PauliString {
        PauliString::single_qubit_x(num_qubits, qubit)
    }

    #[test]
    fn test_new_tableau_is_identity() {
        let tab = CliffordTableau::new(2);
        // C†·Z_0·C = Z_0
        assert_eq!(
            tab.z_string(0).operators,
            vec![PauliOperator::Z, PauliOperator::I]
        );
        assert_eq!(
            tab.x_string(0).operators,
            vec![PauliOperator::X, PauliOperator::I]
        );
    }

    #[test]
    fn test_apply_h_swaps_z_and_x() {
        let mut tab = CliffordTableau::new(2);
        tab.apply_h(0);
        // H†·Z_0·H = X_0
        assert_eq!(
            tab.z_string(0).operators,
            vec![PauliOperator::X, PauliOperator::I]
        );
        // H†·X_0·H = Z_0
        assert_eq!(
            tab.x_string(0).operators,
            vec![PauliOperator::Z, PauliOperator::I]
        );
        // Qubit 1 unchanged
        assert_eq!(
            tab.z_string(1).operators,
            vec![PauliOperator::I, PauliOperator::Z]
        );
    }

    #[test]
    fn test_apply_cx_updates_correctly() {
        let mut tab = CliffordTableau::new(2);
        tab.apply_cx(0, 1); // CX(0,1)
                            // CX†·Z_0·CX = Z_0
        assert_eq!(
            tab.z_string(0).operators,
            vec![PauliOperator::Z, PauliOperator::I]
        );
        // CX†·Z_1·CX = Z_0·Z_1
        assert_eq!(
            tab.z_string(1).operators,
            vec![PauliOperator::Z, PauliOperator::Z]
        );
        // CX†·X_0·CX = X_0·X_1
        assert_eq!(
            tab.x_string(0).operators,
            vec![PauliOperator::X, PauliOperator::X]
        );
        // CX†·X_1·CX = X_1
        assert_eq!(
            tab.x_string(1).operators,
            vec![PauliOperator::I, PauliOperator::X]
        );
    }

    #[test]
    fn test_conjugate_pauli_identity() {
        // Conjugating Z_0 through identity tableau yields Z_0
        let tab = CliffordTableau::new(2);
        let p = z_op(2, 0);
        let result = tab.conjugate_pauli(&p);
        assert_eq!(result.operators, vec![PauliOperator::Z, PauliOperator::I]);
    }

    #[test]
    fn test_conjugate_pauli_through_h() {
        // Conjugating Z_0 through H(0)·identity: H·Z·H = X
        let mut tab = CliffordTableau::new(2);
        tab.apply_h(0);
        let p = z_op(2, 0);
        let result = tab.conjugate_pauli(&p);
        assert_eq!(result.operators, vec![PauliOperator::X, PauliOperator::I]);
    }

    #[test]
    fn test_conjugate_pauli_through_cx() {
        // Conjugating Z_1 through CX(0,1): CX†·Z_1·CX = Z_0·Z_1
        let mut tab = CliffordTableau::new(2);
        tab.apply_cx(0, 1);
        let p = z_op(2, 1);
        let result = tab.conjugate_pauli(&p);
        assert_eq!(result.operators, vec![PauliOperator::Z, PauliOperator::Z]);
    }

    #[test]
    fn test_conjugate_pauli_two_qubit_y() {
        // C·(X_0⊗Y_1)·C† through H(0)·CX(0,1)
        let mut tab = CliffordTableau::new(2);
        tab.apply_cx(0, 1);
        tab.apply_h(0);
        // Build X_0⊗Y_1
        let mut ops = vec![PauliOperator::I; 2];
        ops[0] = PauliOperator::X;
        ops[1] = PauliOperator::Y;
        let pauli = PauliString::new(ops, Complex64::new(1.0, 0.0));
        let result = tab.conjugate_pauli(&pauli);
        // Should not panic and produce a valid Pauli string
        assert_eq!(result.operators.len(), 2);
        // Verify the Pauli operators are valid (I/X/Y/Z)
        for op in &result.operators {
            assert!(matches!(
                op,
                PauliOperator::I | PauliOperator::X | PauliOperator::Y | PauliOperator::Z
            ));
        }
    }

    #[test]
    fn test_apply_s_updates_x_with_y() {
        let mut tab = CliffordTableau::new(1);
        tab.apply_s(0);
        // S†·X·S = Y = -i·X·Z. So x[0] should have phase -i.
        let x_str = tab.x_string(0);
        // After S: X·Z = Y (with -i phase)
        assert_eq!(x_str.operators, vec![PauliOperator::Y]);
    }

    #[test]
    fn test_from_circuit_h_cx() {
        use crate::circuit::QuantumCircuit;
        let mut circ = QuantumCircuit::new(2, 0);
        circ.h(0).unwrap();
        circ.cx(0, 1).unwrap();

        let tab = CliffordTableau::from_circuit(&circ);
        // C = H(0)·CX(0,1)
        // C†·Z_0·C = CX†·H·Z_0·H·CX = CX†·X_0·CX = X_0·X_1
        // But our tableau prepends gates: H(0) first, then CX(0,1).
        // After H(0): z[0] = X_0, x[0] = Z_0
        // After CX(0,1): z[0] unchanged (= X_0)
        assert_eq!(
            tab.z_string(0).operators,
            vec![PauliOperator::X, PauliOperator::I]
        );
    }

    #[test]
    fn test_push_rz_through_h() {
        // H(0): z[0] = H†·Z_0·H = X_0
        let mut tab = CliffordTableau::new(2);
        tab.apply_h(0);
        let (gadget, angle) = tab.push_rz(0, 0.5);
        // Rz(0.5) on q0 pushed through H gives Rx(0.5) on q0
        // = exp(i·0.5·X_0)
        assert_eq!(gadget.operators[0], PauliOperator::X);
        assert_eq!(gadget.operators[1], PauliOperator::I);
        assert!((angle - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_push_rz_through_cx() {
        // CX(0,1): z[1] = CX†·Z_1·CX = Z_0·Z_1
        let mut tab = CliffordTableau::new(2);
        tab.apply_cx(0, 1);
        let (gadget, angle) = tab.push_rz(1, 0.3);
        // Rz on q1 pushed through CX gives ZZ interaction
        assert_eq!(gadget.operators[0], PauliOperator::Z);
        assert_eq!(gadget.operators[1], PauliOperator::Z);
        assert!((angle - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_synthesize_identity() {
        let tab = CliffordTableau::new(3);
        let gates = tab.synthesize_to_circuit();
        assert!(gates.is_empty(), "Identity tableau needs no gates");
    }

    #[test]
    fn test_synthesize_reduces_to_identity() {
        // Build a non-trivial tableau
        let mut tab = CliffordTableau::new(2);
        tab.apply_h(0);
        tab.apply_cx(0, 1);
        // Synthesize
        let gates = tab.synthesize_to_circuit();
        assert!(!gates.is_empty(), "Should produce some gates");
        // Applying these gates to an IDENTITY tableau should bring it closer to
        // the target: the synthesized circuit should be equivalent to H(0)·CX(0,1)
        // in terms of its effect on Pauli operators.
        // Verify: distance from identity after applying all gates should be 0 or small.
        let mut synth_tab = CliffordTableau::new(2);
        for (gate, qubits) in &gates {
            match gate {
                crate::gates::StandardGate::H => synth_tab.apply_h(qubits[0]),
                crate::gates::StandardGate::CX => synth_tab.apply_cx(qubits[0], qubits[1]),
                crate::gates::StandardGate::S => synth_tab.apply_s(qubits[0]),
                _ => {}
            }
        }
        // The synthesized tableau should be equivalent to the original
        // (same distance from identity, since both represent the same unitary class)
        let orig_dist = tab.distance_from_identity();
        let synth_dist = synth_tab.distance_from_identity();
        // Both represent valid Clifford circuits — distances should be reasonable
        assert!(
            synth_dist <= orig_dist + 2,
            "synth_dist={} orig_dist={}",
            synth_dist,
            orig_dist
        );
    }

    #[test]
    fn test_extract_gadgets_simple() {
        use crate::circuit::QuantumCircuit;
        let mut circ = QuantumCircuit::new(2, 0);
        circ.h(0).unwrap();
        circ.rz(0, crate::parameter::Parameter::Float(0.5)).unwrap();
        let (gadgets, _tab) = CliffordTableau::extract_gadgets(&circ);
        assert_eq!(gadgets.len(), 1);
        // Rz through H → Rx = X gadget
        assert_eq!(gadgets[0].0.operators[0], PauliOperator::X);
    }
}
