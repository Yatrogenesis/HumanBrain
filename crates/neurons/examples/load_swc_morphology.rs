//! Example: Loading and analyzing neuronal morphologies from SWC files
//!
//! Run with: cargo run --package neurons --example load_swc_morphology

use neurons::SWCMorphology;
use std::path::PathBuf;

fn main() {
    println!("=== NeuroMorpho.org SWC Parser Demo ===\n");

    let test_data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");

    // Load Layer 2/3 Pyramidal Cell
    println!("Loading Layer 2/3 Pyramidal Cell...");
    let l23_path = test_data_dir.join("layer23_pyramidal.swc");
    if let Ok(l23) = SWCMorphology::from_file(&l23_path) {
        analyze_morphology(&l23);
    }

    println!("\n{}\n", "=".repeat(50));

    // Load Layer 5 Pyramidal Cell
    println!("Loading Layer 5 Pyramidal Cell...");
    let l5_path = test_data_dir.join("layer5_pyramidal.swc");
    if let Ok(l5) = SWCMorphology::from_file(&l5_path) {
        analyze_morphology(&l5);
    }

    println!("\n{}\n", "=".repeat(50));

    // Load Parvalbumin Interneuron
    println!("Loading Parvalbumin-Positive Interneuron...");
    let pv_path = test_data_dir.join("parvalbumin_interneuron.swc");
    if let Ok(pv) = SWCMorphology::from_file(&pv_path) {
        analyze_morphology(&pv);
    }
}

fn analyze_morphology(morph: &SWCMorphology) {
    println!("Cell: {}", morph.name);
    println!("Total points: {}", morph.points.len());

    // Count different compartment types
    let soma_count = morph.points.iter().filter(|p| p.point_type == 1).count();
    let axon_count = morph.points.iter().filter(|p| p.point_type == 2).count();
    let basal_count = morph.points.iter().filter(|p| p.point_type == 3).count();
    let apical_count = morph.points.iter().filter(|p| p.point_type == 4).count();

    println!("  - Soma points: {}", soma_count);
    println!("  - Axon points: {}", axon_count);
    println!("  - Basal dendrite points: {}", basal_count);
    println!("  - Apical dendrite points: {}", apical_count);

    // Get soma radius
    if let Some(soma) = morph.points.iter().find(|p| p.point_type == 1) {
        println!("  - Soma radius: {:.2} µm", soma.radius);
        println!("  - Soma diameter: {:.2} µm", soma.radius * 2.0);
    }

    // Calculate metrics
    let total_length = morph.total_dendritic_length();
    let branch_points = morph.count_branch_points();

    println!("  - Total dendritic length: {:.2} µm", total_length);
    println!("  - Dendritic branch points: {}", branch_points);

    // Calculate surface area (approximation)
    let mut surface_area = 0.0;
    for point in &morph.points {
        if point.point_type >= 3 {  // Dendrites
            if let Some(parent) = morph.points.iter().find(|p| p.id as isize == point.parent_id) {
                let length = (
                    (point.x - parent.x).powi(2) +
                    (point.y - parent.y).powi(2) +
                    (point.z - parent.z).powi(2)
                ).sqrt();
                let avg_radius = (point.radius + parent.radius) / 2.0;
                surface_area += 2.0 * std::f64::consts::PI * avg_radius * length;
            }
        }
    }
    println!("  - Dendritic surface area: {:.2} µm²", surface_area);

    // Spatial extent
    let x_coords: Vec<f64> = morph.points.iter().map(|p| p.x).collect();
    let y_coords: Vec<f64> = morph.points.iter().map(|p| p.y).collect();
    let z_coords: Vec<f64> = morph.points.iter().map(|p| p.z).collect();

    let x_range = x_coords.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
                - x_coords.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_range = y_coords.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
                - y_coords.iter().cloned().fold(f64::INFINITY, f64::min);
    let z_range = z_coords.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
                - z_coords.iter().cloned().fold(f64::INFINITY, f64::min);

    println!("  - Spatial extent (µm):");
    println!("      X: {:.2}, Y: {:.2}, Z: {:.2}", x_range, y_range, z_range);
}
