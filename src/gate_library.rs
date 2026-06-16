// Gate Library and Decomposition
// Author: gA4ss
//
// Provides decompositions for common quantum gates into basic gate sets

use crate::error::Result;
use crate::parameter::Parameter;
use crate::QuantumCircuit;
use std::f64::consts::PI;

/// Standard gate set for decomposition
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GateSet {
    /// Clifford+T gate set: {H, S, T, CNOT}
    CliffordT,
    /// Universal gate set: {Rx, Ry, Rz, CNOT}
    Universal,
    /// IBM gate set: {U1, U2, U3, CNOT}
    IBM,
    /// Rigetti gate set: {RZ, RX($\pi/2$), CZ}
    Rigetti,
    /// IonQ gate set: {GPi, GPi2, MS}
    IonQ,
}

/// Gate decomposer for common quantum gates
pub struct GateDecomposer {
    gate_set: GateSet,
}

impl GateDecomposer {
    /// Create a new gate decomposer
    pub fn new(gate_set: GateSet) -> Self {
        GateDecomposer { gate_set }
    }

    /// Get the current gate set
    pub fn gate_set(&self) -> GateSet {
        self.gate_set
    }

    /// Set the gate set
    pub fn set_gate_set(&mut self, gate_set: GateSet) {
        self.gate_set = gate_set;
    }

    // ===== Hadamard Gate Decompositions =====

    /// Decompose Hadamard gate into the current gate set
    pub fn decompose_hadamard(&self, circuit: &mut QuantumCircuit, qubit: usize) -> Result<()> {
        match self.gate_set {
            GateSet::CliffordT => {
                // H is native in Clifford+T
                circuit.h(qubit)?;
            }
            GateSet::Universal => {
                // $H = R_z(\pi/2) R_y(\pi/2) R_z(\pi/2)$
                circuit.rz(qubit, Parameter::from(PI / 2.0))?;
                circuit.ry(qubit, Parameter::from(PI / 2.0))?;
                circuit.rz(qubit, Parameter::from(PI / 2.0))?;
            }
            GateSet::IBM => {
                // $H = U_2(0, \pi)$
                circuit.ry(qubit, Parameter::from(PI / 2.0))?;
                circuit.rz(qubit, Parameter::from(PI))?;
            }
            GateSet::Rigetti => {
                // $H = R_Z(\pi/2) R_X(\pi/2) R_Z(\pi/2)$
                circuit.rz(qubit, Parameter::from(PI / 2.0))?;
                circuit.rx(qubit, Parameter::from(PI / 2.0))?;
                circuit.rz(qubit, Parameter::from(PI / 2.0))?;
            }
            GateSet::IonQ => {
                // $H = \text{GPi2}(0) \text{GPi}(\pi/2)$
                circuit.ry(qubit, Parameter::from(PI / 2.0))?;
                circuit.rx(qubit, Parameter::from(PI))?;
            }
        }
        Ok(())
    }

    // ===== Pauli Gate Decompositions =====

    /// Decompose X gate
    pub fn decompose_x(&self, circuit: &mut QuantumCircuit, qubit: usize) -> Result<()> {
        match self.gate_set {
            GateSet::CliffordT => {
                // X = H Z H
                circuit.h(qubit)?;
                circuit.z(qubit)?;
                circuit.h(qubit)?;
            }
            GateSet::Universal => {
                // $X = R_x(\pi)$
                circuit.rx(qubit, Parameter::from(PI))?;
            }
            GateSet::IBM => {
                // $X = U_3(\pi, 0, \pi)$
                circuit.ry(qubit, Parameter::from(PI))?;
            }
            GateSet::Rigetti => {
                // $X = R_X(\pi)$
                circuit.rx(qubit, Parameter::from(PI))?;
            }
            GateSet::IonQ => {
                // X = GPi(0)
                circuit.rx(qubit, Parameter::from(PI))?;
            }
        }
        Ok(())
    }

    /// Decompose Y gate
    pub fn decompose_y(&self, circuit: &mut QuantumCircuit, qubit: usize) -> Result<()> {
        match self.gate_set {
            GateSet::CliffordT => {
                // Y = S X S†
                circuit.s(qubit)?;
                circuit.x(qubit)?;
                circuit.sdg(qubit)?;
            }
            GateSet::Universal => {
                // $Y = R_y(\pi)$
                circuit.ry(qubit, Parameter::from(PI))?;
            }
            GateSet::IBM => {
                // $Y = U_3(\pi, \pi/2, \pi/2)$
                circuit.rz(qubit, Parameter::from(PI / 2.0))?;
                circuit.ry(qubit, Parameter::from(PI))?;
                circuit.rz(qubit, Parameter::from(PI / 2.0))?;
            }
            GateSet::Rigetti => {
                // $Y = R_Z(\pi/2) R_X(\pi) R_Z(-\pi/2)$
                circuit.rz(qubit, Parameter::from(PI / 2.0))?;
                circuit.rx(qubit, Parameter::from(PI))?;
                circuit.rz(qubit, Parameter::from(-PI / 2.0))?;
            }
            GateSet::IonQ => {
                // $Y = \text{GPi}(\pi/2)$
                circuit.ry(qubit, Parameter::from(PI))?;
            }
        }
        Ok(())
    }

    /// Decompose Z gate
    pub fn decompose_z(&self, circuit: &mut QuantumCircuit, qubit: usize) -> Result<()> {
        match self.gate_set {
            GateSet::CliffordT => {
                // Z = S S
                circuit.s(qubit)?;
                circuit.s(qubit)?;
            }
            GateSet::Universal => {
                // $Z = R_z(\pi)$
                circuit.rz(qubit, Parameter::from(PI))?;
            }
            GateSet::IBM => {
                // $Z = U_1(\pi)$
                circuit.rz(qubit, Parameter::from(PI))?;
            }
            GateSet::Rigetti => {
                // $Z = R_Z(\pi)$
                circuit.rz(qubit, Parameter::from(PI))?;
            }
            GateSet::IonQ => {
                // $Z = R_Z(\pi)$
                circuit.rz(qubit, Parameter::from(PI))?;
            }
        }
        Ok(())
    }

    // ===== Phase Gates =====

    /// Decompose S gate ($\sqrt{Z}$)
    pub fn decompose_s(&self, circuit: &mut QuantumCircuit, qubit: usize) -> Result<()> {
        match self.gate_set {
            GateSet::CliffordT => {
                // S is native
                circuit.s(qubit)?;
            }
            GateSet::Universal => {
                // $S = R_z(\pi/2)$
                circuit.rz(qubit, Parameter::from(PI / 2.0))?;
            }
            GateSet::IBM => {
                // $S = U_1(\pi/2)$
                circuit.rz(qubit, Parameter::from(PI / 2.0))?;
            }
            GateSet::Rigetti => {
                // $S = R_Z(\pi/2)$
                circuit.rz(qubit, Parameter::from(PI / 2.0))?;
            }
            GateSet::IonQ => {
                // $S = R_Z(\pi/2)$
                circuit.rz(qubit, Parameter::from(PI / 2.0))?;
            }
        }
        Ok(())
    }

    /// Decompose T gate ($\sqrt{S}$)
    pub fn decompose_t(&self, circuit: &mut QuantumCircuit, qubit: usize) -> Result<()> {
        match self.gate_set {
            GateSet::CliffordT => {
                // T is native
                circuit.t(qubit)?;
            }
            GateSet::Universal => {
                // $T = R_z(\pi/4)$
                circuit.rz(qubit, Parameter::from(PI / 4.0))?;
            }
            GateSet::IBM => {
                // $T = U_1(\pi/4)$
                circuit.rz(qubit, Parameter::from(PI / 4.0))?;
            }
            GateSet::Rigetti => {
                // $T = R_Z(\pi/4)$
                circuit.rz(qubit, Parameter::from(PI / 4.0))?;
            }
            GateSet::IonQ => {
                // $T = R_Z(\pi/4)$
                circuit.rz(qubit, Parameter::from(PI / 4.0))?;
            }
        }
        Ok(())
    }

    // ===== Rotation Gates =====

    /// Decompose Rx gate
    pub fn decompose_rx(
        &self,
        circuit: &mut QuantumCircuit,
        qubit: usize,
        angle: Parameter,
    ) -> Result<()> {
        match self.gate_set {
            GateSet::CliffordT => {
                // $R_x(\theta) \approx$ decompose into Clifford+T (approximate)
                // For exact: use Solovay-Kitaev algorithm
                // Simplified: $H R_z(\theta) H$
                circuit.h(qubit)?;
                self.decompose_rz(circuit, qubit, angle)?;
                circuit.h(qubit)?;
            }
            GateSet::Universal => {
                // Rx is native
                circuit.rx(qubit, angle)?;
            }
            GateSet::IBM => {
                // $R_x(\theta) = U_3(\theta, -\pi/2, \pi/2)$
                circuit.rz(qubit, Parameter::from(-PI / 2.0))?;
                circuit.ry(qubit, angle)?;
                circuit.rz(qubit, Parameter::from(PI / 2.0))?;
            }
            GateSet::Rigetti => {
                // Rx is native
                circuit.rx(qubit, angle)?;
            }
            GateSet::IonQ => {
                // $R_x(\theta)$ via GPi gates
                circuit.rx(qubit, angle)?;
            }
        }
        Ok(())
    }

    /// Decompose Ry gate
    pub fn decompose_ry(
        &self,
        circuit: &mut QuantumCircuit,
        qubit: usize,
        angle: Parameter,
    ) -> Result<()> {
        match self.gate_set {
            GateSet::CliffordT => {
                // $R_y(\theta) \approx$ decompose into Clifford+T (approximate)
                // Simplified: $S^\dagger H R_z(\theta) H S$
                circuit.sdg(qubit)?;
                circuit.h(qubit)?;
                self.decompose_rz(circuit, qubit, angle)?;
                circuit.h(qubit)?;
                circuit.s(qubit)?;
            }
            GateSet::Universal => {
                // Ry is native
                circuit.ry(qubit, angle)?;
            }
            GateSet::IBM => {
                // $R_y(\theta) = U_3(\theta, 0, 0)$
                circuit.ry(qubit, angle)?;
            }
            GateSet::Rigetti => {
                // $R_y(\theta) = R_Z(-\pi/2) R_X(\theta) R_Z(\pi/2)$
                circuit.rz(qubit, Parameter::from(-PI / 2.0))?;
                circuit.rx(qubit, angle)?;
                circuit.rz(qubit, Parameter::from(PI / 2.0))?;
            }
            GateSet::IonQ => {
                // Ry via GPi gates
                circuit.ry(qubit, angle)?;
            }
        }
        Ok(())
    }

    /// Decompose Rz gate
    pub fn decompose_rz(
        &self,
        circuit: &mut QuantumCircuit,
        qubit: usize,
        angle: Parameter,
    ) -> Result<()> {
        match self.gate_set {
            GateSet::CliffordT => {
                // $R_z(\theta) \approx$ decompose into T gates (approximate)
                // Exact decomposition requires Solovay-Kitaev
                // For now, use direct Rz
                circuit.rz(qubit, angle)?;
            }
            GateSet::Universal => {
                // Rz is native
                circuit.rz(qubit, angle)?;
            }
            GateSet::IBM => {
                // $R_z(\theta) = U_1(\theta)$
                circuit.rz(qubit, angle)?;
            }
            GateSet::Rigetti => {
                // Rz is native
                circuit.rz(qubit, angle)?;
            }
            GateSet::IonQ => {
                // Rz is native
                circuit.rz(qubit, angle)?;
            }
        }
        Ok(())
    }

    // ===== Two-Qubit Gates =====

    /// Decompose SWAP gate
    pub fn decompose_swap(&self, circuit: &mut QuantumCircuit, q1: usize, q2: usize) -> Result<()> {
        // SWAP = CNOT(q1,q2) CNOT(q2,q1) CNOT(q1,q2)
        circuit.cx(q1, q2)?;
        circuit.cx(q2, q1)?;
        circuit.cx(q1, q2)?;
        Ok(())
    }

    /// Decompose iSWAP gate
    pub fn decompose_iswap(
        &self,
        circuit: &mut QuantumCircuit,
        q1: usize,
        q2: usize,
    ) -> Result<()> {
        // iSWAP = S(q1) S(q2) H(q1) CNOT(q1,q2) CNOT(q2,q1) H(q2)
        circuit.s(q1)?;
        circuit.s(q2)?;
        circuit.h(q1)?;
        circuit.cx(q1, q2)?;
        circuit.cx(q2, q1)?;
        circuit.h(q2)?;
        Ok(())
    }

    /// Decompose CZ gate
    pub fn decompose_cz(
        &self,
        circuit: &mut QuantumCircuit,
        control: usize,
        target: usize,
    ) -> Result<()> {
        match self.gate_set {
            GateSet::Rigetti => {
                // CZ is native
                circuit.cz(control, target)?;
            }
            _ => {
                // CZ = H(target) CNOT(control, target) H(target)
                circuit.h(target)?;
                circuit.cx(control, target)?;
                circuit.h(target)?;
            }
        }
        Ok(())
    }

    /// Decompose Toffoli (CCX) gate
    pub fn decompose_toffoli(
        &self,
        circuit: &mut QuantumCircuit,
        c1: usize,
        c2: usize,
        target: usize,
    ) -> Result<()> {
        // Standard Toffoli decomposition using 6 CNOTs
        circuit.h(target)?;
        circuit.cx(c2, target)?;
        circuit.tdg(target)?;
        circuit.cx(c1, target)?;
        circuit.t(target)?;
        circuit.cx(c2, target)?;
        circuit.tdg(target)?;
        circuit.cx(c1, target)?;
        circuit.t(c2)?;
        circuit.t(target)?;
        circuit.h(target)?;
        circuit.cx(c1, c2)?;
        circuit.t(c1)?;
        circuit.tdg(c2)?;
        circuit.cx(c1, c2)?;
        Ok(())
    }

    /// Decompose Fredkin (CSWAP) gate
    pub fn decompose_fredkin(
        &self,
        circuit: &mut QuantumCircuit,
        control: usize,
        t1: usize,
        t2: usize,
    ) -> Result<()> {
        // CSWAP = CNOT(t2,t1) Toffoli(control,t1,t2) CNOT(t2,t1)
        circuit.cx(t2, t1)?;
        self.decompose_toffoli(circuit, control, t1, t2)?;
        circuit.cx(t2, t1)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decomposer_creation() {
        let decomposer = GateDecomposer::new(GateSet::CliffordT);
        assert_eq!(decomposer.gate_set(), GateSet::CliffordT);
    }

    #[test]
    fn test_hadamard_decomposition() {
        let decomposer = GateDecomposer::new(GateSet::Universal);
        let mut circuit = QuantumCircuit::new(1, 0);

        decomposer.decompose_hadamard(&mut circuit, 0).unwrap();
        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_pauli_decompositions() {
        let decomposer = GateDecomposer::new(GateSet::CliffordT);
        let mut circuit = QuantumCircuit::new(1, 0);

        decomposer.decompose_x(&mut circuit, 0).unwrap();
        decomposer.decompose_y(&mut circuit, 0).unwrap();
        decomposer.decompose_z(&mut circuit, 0).unwrap();

        assert!(circuit.size() > 0);
    }

    #[test]
    fn test_swap_decomposition() {
        let decomposer = GateDecomposer::new(GateSet::Universal);
        let mut circuit = QuantumCircuit::new(2, 0);

        decomposer.decompose_swap(&mut circuit, 0, 1).unwrap();
        assert_eq!(circuit.size(), 3); // 3 CNOTs
    }

    #[test]
    fn test_toffoli_decomposition() {
        let decomposer = GateDecomposer::new(GateSet::CliffordT);
        let mut circuit = QuantumCircuit::new(3, 0);

        decomposer.decompose_toffoli(&mut circuit, 0, 1, 2).unwrap();
        assert!(circuit.size() > 10); // Multiple gates
    }

    #[test]
    fn test_gate_set_change() {
        let mut decomposer = GateDecomposer::new(GateSet::CliffordT);
        assert_eq!(decomposer.gate_set(), GateSet::CliffordT);

        decomposer.set_gate_set(GateSet::IBM);
        assert_eq!(decomposer.gate_set(), GateSet::IBM);
    }
}
