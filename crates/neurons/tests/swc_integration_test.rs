//! Integration tests for SWC morphology loading

use neurons::SWCMorphology;
use std::path::PathBuf;

fn get_test_data_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_data");
    path.push(filename);
    path
}

#[test]
fn test_load_layer23_pyramidal() {
    let path = get_test_data_path("layer23_pyramidal.swc");
    let morph = SWCMorphology::from_file(&path).expect("Failed to load L2/3 pyramidal");

    assert_eq!(morph.name, "layer23_pyramidal");
    assert!(morph.points.len() > 0, "Should have points");

    // Check soma exists (type 1)
    let soma_count = morph.points.iter().filter(|p| p.point_type == 1).count();
    assert_eq!(soma_count, 1, "Should have exactly one soma");

    // Check has dendrites
    let dendrite_count = morph.points.iter().filter(|p| p.point_type == 3 || p.point_type == 4).count();
    assert!(dendrite_count > 0, "Should have dendrites");

    // Check has axon
    let axon_count = morph.points.iter().filter(|p| p.point_type == 2).count();
    assert!(axon_count > 0, "Should have axon");

    let length = morph.total_dendritic_length();
    assert!(length > 0.0, "Total dendritic length should be positive");
    println!("L2/3 Pyramidal - Total dendritic length: {:.2} µm", length);

    let branches = morph.count_branch_points();
    println!("L2/3 Pyramidal - Branch points: {}", branches);
}

#[test]
fn test_load_layer5_pyramidal() {
    let path = get_test_data_path("layer5_pyramidal.swc");
    let morph = SWCMorphology::from_file(&path).expect("Failed to load L5 pyramidal");

    assert_eq!(morph.name, "layer5_pyramidal");
    assert!(morph.points.len() > 0, "Should have points");

    // L5 pyramidals have larger soma
    let soma = morph.points.iter().find(|p| p.point_type == 1).expect("Should have soma");
    assert!(soma.radius > 10.0, "L5 pyramidal should have larger soma (>10µm)");

    let length = morph.total_dendritic_length();
    assert!(length > 0.0, "Total dendritic length should be positive");
    println!("L5 Pyramidal - Total dendritic length: {:.2} µm", length);

    let branches = morph.count_branch_points();
    println!("L5 Pyramidal - Branch points: {}", branches);

    // L5 pyramidals should have more complex morphology than L2/3
    assert!(morph.points.len() > 30, "L5 should have extensive morphology");
}

#[test]
fn test_load_parvalbumin_interneuron() {
    let path = get_test_data_path("parvalbumin_interneuron.swc");
    let morph = SWCMorphology::from_file(&path).expect("Failed to load PV interneuron");

    assert_eq!(morph.name, "parvalbumin_interneuron");
    assert!(morph.points.len() > 0, "Should have points");

    // PV interneurons have smaller soma
    let soma = morph.points.iter().find(|p| p.point_type == 1).expect("Should have soma");
    assert!(soma.radius < 10.0, "PV interneuron should have smaller soma (<10µm)");

    // PV interneurons have extensive local axon
    let axon_count = morph.points.iter().filter(|p| p.point_type == 2).count();
    assert!(axon_count > 15, "PV interneuron should have extensive local axon");
    println!("PV Interneuron - Axon segments: {}", axon_count);

    let length = morph.total_dendritic_length();
    println!("PV Interneuron - Total dendritic length: {:.2} µm", length);

    let branches = morph.count_branch_points();
    println!("PV Interneuron - Branch points: {}", branches);
}

#[test]
fn test_compare_morphologies() {
    let l23 = SWCMorphology::from_file(&get_test_data_path("layer23_pyramidal.swc")).unwrap();
    let l5 = SWCMorphology::from_file(&get_test_data_path("layer5_pyramidal.swc")).unwrap();
    let pv = SWCMorphology::from_file(&get_test_data_path("parvalbumin_interneuron.swc")).unwrap();

    let l23_length = l23.total_dendritic_length();
    let l5_length = l5.total_dendritic_length();
    let pv_length = pv.total_dendritic_length();

    println!("\n=== Morphology Comparison ===");
    println!("L2/3 Pyramidal: {:.2} µm, {} branches, {} points",
             l23_length, l23.count_branch_points(), l23.points.len());
    println!("L5 Pyramidal:   {:.2} µm, {} branches, {} points",
             l5_length, l5.count_branch_points(), l5.points.len());
    println!("PV Interneuron: {:.2} µm, {} branches, {} points",
             pv_length, pv.count_branch_points(), pv.points.len());

    // L5 should have longer dendrites than L2/3
    assert!(l5_length > l23_length, "L5 pyramidal should have longer dendrites than L2/3");

    // PV interneurons typically have more compact dendrites than pyramidal cells
    assert!(pv_length < l5_length, "PV interneuron should have more compact dendrites than L5");
}
