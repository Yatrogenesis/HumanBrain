//! Neuron morphology and dendritic tree structures.

use serde::{Deserialize, Serialize};
use crate::{CompartmentType, Result};

/// Represents a 3D point in space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3D {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn distance(&self, other: &Point3D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Dendritic tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DendriticTree {
    /// Root position (usually soma)
    pub root: Point3D,

    /// Branch segments
    pub branches: Vec<DendriticBranch>,
}

/// A single dendritic branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DendriticBranch {
    /// Start position
    pub start: Point3D,

    /// End position
    pub end: Point3D,

    /// Diameter at start (um)
    pub diameter_start: f64,

    /// Diameter at end (um)
    pub diameter_end: f64,

    /// Parent branch index
    pub parent_idx: Option<usize>,

    /// Branch type
    pub branch_type: CompartmentType,
}

/// Complete neuron morphology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronMorphology {
    /// Neuron type name
    pub neuron_type: String,

    /// Soma position
    pub soma_position: Point3D,

    /// Soma diameter (um)
    pub soma_diameter: f64,

    /// Dendritic trees
    pub dendrites: Vec<DendriticTree>,

    /// Axon tree (if present)
    pub axon: Option<DendriticTree>,
}

impl NeuronMorphology {
    /// Create a simple pyramidal neuron morphology
    pub fn pyramidal_neuron() -> Self {
        let soma_pos = Point3D::new(0.0, 0.0, 0.0);

        // Create apical dendrite
        let mut apical_branches = Vec::new();
        let mut y_pos = 25.0;
        for i in 0..20 {
            apical_branches.push(DendriticBranch {
                start: Point3D::new(0.0, y_pos, 0.0),
                end: Point3D::new(0.0, y_pos + 50.0, 0.0),
                diameter_start: 3.0 - (i as f64 * 0.1),
                diameter_end: 2.8 - (i as f64 * 0.1),
                parent_idx: if i == 0 { None } else { Some(i - 1) },
                branch_type: CompartmentType::ApicalDendrite,
            });
            y_pos += 50.0;
        }

        let apical_tree = DendriticTree {
            root: soma_pos,
            branches: apical_branches,
        };

        // Create basal dendrites
        let mut basal_trees = Vec::new();
        for angle in [0.0_f64, 60.0, 120.0, 180.0, 240.0, 300.0] {
            let rad = angle.to_radians();
            let mut branches = Vec::new();

            for i in 0..5 {
                let length = 30.0;
                let start_r = (i as f64) * length;
                let end_r = start_r + length;

                branches.push(DendriticBranch {
                    start: Point3D::new(
                        start_r * rad.cos(),
                        -10.0 - (i as f64 * 5.0),
                        start_r * rad.sin(),
                    ),
                    end: Point3D::new(
                        end_r * rad.cos(),
                        -15.0 - (i as f64 * 5.0),
                        end_r * rad.sin(),
                    ),
                    diameter_start: 2.0 - (i as f64 * 0.2),
                    diameter_end: 1.8 - (i as f64 * 0.2),
                    parent_idx: if i == 0 { None } else { Some(i - 1) },
                    branch_type: CompartmentType::BasalDendrite,
                });
            }

            basal_trees.push(DendriticTree {
                root: soma_pos,
                branches,
            });
        }

        // Create axon
        let mut axon_branches = Vec::new();
        let mut y_pos = -25.0;
        for i in 0..10 {
            axon_branches.push(DendriticBranch {
                start: Point3D::new(0.0, y_pos, 0.0),
                end: Point3D::new(0.0, y_pos - 100.0, 0.0),
                diameter_start: 1.0,
                diameter_end: 1.0,
                parent_idx: if i == 0 { None } else { Some(i - 1) },
                branch_type: CompartmentType::Axon,
            });
            y_pos -= 100.0;
        }

        let axon_tree = DendriticTree {
            root: soma_pos,
            branches: axon_branches,
        };

        let mut all_dendrites = vec![apical_tree];
        all_dendrites.extend(basal_trees);

        Self {
            neuron_type: "Pyramidal".to_string(),
            soma_position: soma_pos,
            soma_diameter: 25.0,
            dendrites: all_dendrites,
            axon: Some(axon_tree),
        }
    }

    /// Create an interneuron morphology
    pub fn interneuron() -> Self {
        let soma_pos = Point3D::new(0.0, 0.0, 0.0);

        // Interneurons have more symmetric, multipolar dendrites
        let mut dendrites = Vec::new();

        for angle in [0.0_f64, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0] {
            let rad = angle.to_radians();
            let mut branches = Vec::new();

            for i in 0..8 {
                let length = 25.0;
                let start_r = (i as f64) * length;
                let end_r = start_r + length;

                branches.push(DendriticBranch {
                    start: Point3D::new(
                        start_r * rad.cos(),
                        0.0,
                        start_r * rad.sin(),
                    ),
                    end: Point3D::new(
                        end_r * rad.cos(),
                        0.0,
                        end_r * rad.sin(),
                    ),
                    diameter_start: 1.5 - (i as f64 * 0.1),
                    diameter_end: 1.3 - (i as f64 * 0.1),
                    parent_idx: if i == 0 { None } else { Some(i - 1) },
                    branch_type: CompartmentType::Dendrite,
                });
            }

            dendrites.push(DendriticTree {
                root: soma_pos,
                branches,
            });
        }

        Self {
            neuron_type: "Interneuron".to_string(),
            soma_position: soma_pos,
            soma_diameter: 15.0,
            dendrites,
            axon: None, // Simplified - interneurons have local axons
        }
    }

    /// Calculate total dendritic length
    pub fn total_dendritic_length(&self) -> f64 {
        self.dendrites
            .iter()
            .flat_map(|tree| &tree.branches)
            .map(|branch| branch.start.distance(&branch.end))
            .sum()
    }

    /// Calculate total dendritic surface area
    pub fn total_dendritic_surface_area(&self) -> f64 {
        self.dendrites
            .iter()
            .flat_map(|tree| &tree.branches)
            .map(|branch| {
                let length = branch.start.distance(&branch.end);
                let avg_diameter = (branch.diameter_start + branch.diameter_end) / 2.0;
                std::f64::consts::PI * avg_diameter * length
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pyramidal_morphology() {
        let morph = NeuronMorphology::pyramidal_neuron();
        assert_eq!(morph.neuron_type, "Pyramidal");
        assert!(morph.dendrites.len() > 1);
        assert!(morph.axon.is_some());

        let length = morph.total_dendritic_length();
        assert!(length > 0.0);
    }

    #[test]
    fn test_interneuron_morphology() {
        let morph = NeuronMorphology::interneuron();
        assert_eq!(morph.neuron_type, "Interneuron");
        assert!(morph.dendrites.len() > 0);

        let area = morph.total_dendritic_surface_area();
        assert!(area > 0.0);
    }

    #[test]
    fn test_point_distance() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(3.0, 4.0, 0.0);
        let dist = p1.distance(&p2);
        assert!((dist - 5.0).abs() < 1e-10);
    }
}
