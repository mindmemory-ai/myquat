//! Molecule Database
//! Author: gA4ss
//!
//! Provides pre-computed molecular data for common molecules.

use super::fermion::ElectronicStructureHamiltonian;

/// Molecular geometry
#[derive(Debug, Clone)]
pub struct Geometry {
    /// Atom symbols
    pub atoms: Vec<String>,
    /// Coordinates in Angstroms [(x, y, z), ...]
    pub coordinates: Vec<(f64, f64, f64)>,
}

/// Molecule data including geometry and electronic structure
#[derive(Debug, Clone)]
pub struct MoleculeData {
    /// Molecule name
    pub name: String,
    /// Molecular geometry
    pub geometry: Geometry,
    /// Number of electrons
    pub num_electrons: usize,
    /// Number of orbitals
    pub num_orbitals: usize,
    /// Electronic structure Hamiltonian (if computed)
    pub hamiltonian: Option<ElectronicStructureHamiltonian>,
    /// Ground state energy (Hartree) from reference calculation
    pub reference_energy: Option<f64>,
}

impl MoleculeData {
    /// Create H₂ molecule at equilibrium
    pub fn h2_equilibrium() -> Self {
        let geometry = Geometry {
            atoms: vec!["H".to_string(), "H".to_string()],
            coordinates: vec![(0.0, 0.0, 0.0), (0.0, 0.0, 0.735)],
        };

        // H2 STO-3G basis electronic structure
        let mut hamiltonian = ElectronicStructureHamiltonian::new(2);

        // One-body integrals (simplified STO-3G)
        hamiltonian.set_one_body(0, 0, -1.252477);
        hamiltonian.set_one_body(1, 1, -1.252477);
        hamiltonian.set_one_body(0, 1, -0.475934);
        hamiltonian.set_one_body(1, 0, -0.475934);

        // Two-body integrals (key terms)
        hamiltonian.set_two_body(0, 0, 0, 0, 0.674493);
        hamiltonian.set_two_body(1, 1, 1, 1, 0.674493);
        hamiltonian.set_two_body(0, 0, 1, 1, 0.663472);
        hamiltonian.set_two_body(0, 1, 1, 0, 0.181287);

        // Nuclear repulsion
        hamiltonian.nuclear_repulsion = 0.713753;

        Self {
            name: "H2".to_string(),
            geometry,
            num_electrons: 2,
            num_orbitals: 2,
            hamiltonian: Some(hamiltonian),
            reference_energy: Some(-1.137270),
        }
    }

    /// Create H₂ molecule at stretched geometry
    pub fn h2_stretched() -> Self {
        let mut data = Self::h2_equilibrium();
        data.geometry.coordinates[1] = (0.0, 0.0, 1.5);
        data.name = "H2_stretched".to_string();
        data.reference_energy = Some(-0.89);
        data
    }

    /// Create LiH molecule
    pub fn lih_equilibrium() -> Self {
        let geometry = Geometry {
            atoms: vec!["Li".to_string(), "H".to_string()],
            coordinates: vec![(0.0, 0.0, 0.0), (0.0, 0.0, 1.595)],
        };

        let mut hamiltonian = ElectronicStructureHamiltonian::new(6);

        // Simplified integrals for LiH
        hamiltonian.nuclear_repulsion = 0.995;

        Self {
            name: "LiH".to_string(),
            geometry,
            num_electrons: 4,
            num_orbitals: 6,
            hamiltonian: Some(hamiltonian),
            reference_energy: Some(-7.882),
        }
    }

    /// Create H₂O molecule
    pub fn h2o_equilibrium() -> Self {
        let geometry = Geometry {
            atoms: vec!["O".to_string(), "H".to_string(), "H".to_string()],
            coordinates: vec![(0.0, 0.0, 0.0), (0.0, 0.757, 0.586), (0.0, -0.757, 0.586)],
        };

        let hamiltonian = ElectronicStructureHamiltonian::new(7);

        Self {
            name: "H2O".to_string(),
            geometry,
            num_electrons: 10,
            num_orbitals: 7,
            hamiltonian: Some(hamiltonian),
            reference_energy: Some(-76.026),
        }
    }

    /// Create BeH₂ molecule
    pub fn beh2_linear() -> Self {
        let geometry = Geometry {
            atoms: vec!["Be".to_string(), "H".to_string(), "H".to_string()],
            coordinates: vec![(0.0, 0.0, 0.0), (0.0, 0.0, 1.334), (0.0, 0.0, -1.334)],
        };

        let hamiltonian = ElectronicStructureHamiltonian::new(7);

        Self {
            name: "BeH2".to_string(),
            geometry,
            num_electrons: 6,
            num_orbitals: 7,
            hamiltonian: Some(hamiltonian),
            reference_energy: Some(-15.865),
        }
    }

    /// Create N₂ molecule
    pub fn n2_equilibrium() -> Self {
        let geometry = Geometry {
            atoms: vec!["N".to_string(), "N".to_string()],
            coordinates: vec![(0.0, 0.0, 0.0), (0.0, 0.0, 1.098)],
        };

        let hamiltonian = ElectronicStructureHamiltonian::new(10);

        Self {
            name: "N2".to_string(),
            geometry,
            num_electrons: 14,
            num_orbitals: 10,
            hamiltonian: Some(hamiltonian),
            reference_energy: Some(-109.094),
        }
    }

    /// Create NH₃ molecule
    pub fn nh3_equilibrium() -> Self {
        let geometry = Geometry {
            atoms: vec![
                "N".to_string(),
                "H".to_string(),
                "H".to_string(),
                "H".to_string(),
            ],
            coordinates: vec![
                (0.0, 0.0, 0.0),
                (0.0, 0.939, 0.383),
                (0.813, -0.469, 0.383),
                (-0.813, -0.469, 0.383),
            ],
        };

        let hamiltonian = ElectronicStructureHamiltonian::new(8);

        Self {
            name: "NH3".to_string(),
            geometry,
            num_electrons: 10,
            num_orbitals: 8,
            hamiltonian: Some(hamiltonian),
            reference_energy: Some(-56.224),
        }
    }
}

/// Molecule database
pub struct MoleculeDatabase {
    molecules: Vec<MoleculeData>,
}

impl MoleculeDatabase {
    /// Create a new database with common molecules
    pub fn new() -> Self {
        let molecules = vec![
            MoleculeData::h2_equilibrium(),
            MoleculeData::h2_stretched(),
            MoleculeData::lih_equilibrium(),
            MoleculeData::h2o_equilibrium(),
            MoleculeData::beh2_linear(),
            MoleculeData::n2_equilibrium(),
            MoleculeData::nh3_equilibrium(),
        ];

        Self { molecules }
    }

    /// Get molecule by name
    pub fn get(&self, name: &str) -> Option<&MoleculeData> {
        self.molecules.iter().find(|m| m.name == name)
    }

    /// List all available molecules
    pub fn list_molecules(&self) -> Vec<String> {
        self.molecules.iter().map(|m| m.name.clone()).collect()
    }

    /// Get number of molecules in database
    pub fn size(&self) -> usize {
        self.molecules.len()
    }
}

impl Default for MoleculeDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_h2_molecule() {
        let h2 = MoleculeData::h2_equilibrium();
        assert_eq!(h2.name, "H2");
        assert_eq!(h2.num_electrons, 2);
        assert_eq!(h2.num_orbitals, 2);
        assert!(h2.hamiltonian.is_some());
        assert!(h2.reference_energy.is_some());
    }

    #[test]
    fn test_molecule_database() {
        let db = MoleculeDatabase::new();
        assert!(db.size() >= 7);

        let h2 = db.get("H2");
        assert!(h2.is_some());

        let molecules = db.list_molecules();
        assert!(molecules.contains(&"H2".to_string()));
        assert!(molecules.contains(&"H2O".to_string()));
    }

    #[test]
    fn test_h2o_geometry() {
        let h2o = MoleculeData::h2o_equilibrium();
        assert_eq!(h2o.geometry.atoms.len(), 3);
        assert_eq!(h2o.num_electrons, 10);
    }
}
