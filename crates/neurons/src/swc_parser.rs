//! SWC file parser for NeuroMorpho.org reconstructions
//!
//! The SWC format is a standard format for neuronal morphology reconstructions.
//! Each line represents a point in 3D space with a radius and connectivity information.
//!
//! # Format
//!
//! ```text
//! # Columns: n T x y z R parent
//! 1 1 0.0 0.0 0.0 10.0 -1    # Soma (type 1, no parent)
//! 2 3 0.0 0.0 15.0 2.0 1     # Dendrite (type 3, parent=soma)
//! 3 3 5.0 0.0 20.0 1.5 2     # Dendrite continuation
//! 4 2 0.0 -10.0 0.0 0.5 1    # Axon (type 2)
//! ```
//!
//! # Point Types
//!
//! - 1 = Soma
//! - 2 = Axon
//! - 3 = Basal dendrite
//! - 4 = Apical dendrite
//! - 5 = Custom (fork point, end point)

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// A single point in an SWC morphology file
#[derive(Debug, Clone)]
pub struct SWCPoint {
    /// Unique identifier for this point
    pub id: usize,
    /// Point type (1=soma, 2=axon, 3=basal dendrite, 4=apical dendrite, 5=custom)
    pub point_type: usize,
    /// X coordinate in microns
    pub x: f64,
    /// Y coordinate in microns
    pub y: f64,
    /// Z coordinate in microns
    pub z: f64,
    /// Radius in microns
    pub radius: f64,
    /// Parent point ID (-1 for root/soma)
    pub parent_id: isize,
}

/// Complete neuronal morphology parsed from an SWC file
#[derive(Debug)]
pub struct SWCMorphology {
    /// All points in the morphology
    pub points: Vec<SWCPoint>,
    /// Name of the morphology (from filename)
    pub name: String,
}

impl SWCMorphology {
    /// Parse an SWC file from the given path
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::Path;
    /// use neurons::SWCMorphology;
    ///
    /// let morph = SWCMorphology::from_file(Path::new("neuron.swc")).unwrap();
    /// println!("Loaded {} with {} points", morph.name, morph.points.len());
    /// ```
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut points = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            // Skip comments and empty lines
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            // Parse: n T x y z R parent
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 7 {
                continue;
            }

            let point = SWCPoint {
                id: parts[0].parse()?,
                point_type: parts[1].parse()?,
                x: parts[2].parse()?,
                y: parts[3].parse()?,
                z: parts[4].parse()?,
                radius: parts[5].parse()?,
                parent_id: parts[6].parse()?,
            };

            points.push(point);
        }

        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self { points, name })
    }

    /// Calculate total dendritic length (both basal and apical)
    ///
    /// Returns the sum of all dendritic segment lengths in microns.
    pub fn total_dendritic_length(&self) -> f64 {
        let mut length = 0.0;

        for point in &self.points {
            // Type 3 = basal dendrite, Type 4 = apical dendrite
            if point.point_type == 3 || point.point_type == 4 {
                if let Some(parent) = self.points.iter().find(|p| p.id as isize == point.parent_id) {
                    let dx = point.x - parent.x;
                    let dy = point.y - parent.y;
                    let dz = point.z - parent.z;
                    length += (dx*dx + dy*dy + dz*dz).sqrt();
                }
            }
        }

        length
    }

    /// Count the number of branch points in the morphology
    ///
    /// A branch point is any point that has more than one child.
    pub fn count_branch_points(&self) -> usize {
        let mut children_count = vec![0; self.points.len() + 1];

        for point in &self.points {
            if point.parent_id >= 0 {
                children_count[point.parent_id as usize] += 1;
            }
        }

        children_count.iter().filter(|&&c| c > 1).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_swc_parser() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Test SWC").unwrap();
        writeln!(file, "1 1 0 0 0 10 -1").unwrap();
        writeln!(file, "2 3 0 0 15 2 1").unwrap();
        writeln!(file, "3 3 5 0 20 1.5 2").unwrap();
        let morph = SWCMorphology::from_file(file.path()).unwrap();
        assert_eq!(morph.points.len(), 3);
        assert_eq!(morph.points[0].point_type, 1);
        assert!(morph.total_dendritic_length() > 0.0);
    }

    #[test]
    fn test_dendritic_length() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "1 1 0 0 0 10 -1").unwrap();
        writeln!(file, "2 3 0 0 10 2 1").unwrap();
        writeln!(file, "3 3 0 0 20 1.5 2").unwrap();
        let morph = SWCMorphology::from_file(file.path()).unwrap();
        let length = morph.total_dendritic_length();
        assert!((length - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_branch_points() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "1 1 0 0 0 10 -1").unwrap();
        writeln!(file, "2 3 0 0 10 2 1").unwrap();
        writeln!(file, "3 3 5 0 15 1.5 2").unwrap();
        writeln!(file, "4 3 -5 0 15 1.5 2").unwrap();
        let morph = SWCMorphology::from_file(file.path()).unwrap();
        let branches = morph.count_branch_points();
        assert_eq!(branches, 1);
    }
}
