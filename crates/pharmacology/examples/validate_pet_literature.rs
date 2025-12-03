//! PET Literature Validation
//!
//! Validates model predictions against PET imaging data from clinical studies.
//! Calculates real %error for receptor occupancy predictions.
//!
//! Run with: cargo run --example validate_pet_literature

use pharmacology::pharmacokinetics::{PkDatabase, RouteOfAdministration, calculate_brain_concentration};
use pharmacology::clinical_literature::{ClinicalLiteratureDb, calculate_occupancy_from_ki, ValidationResult};

fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║       HumanBrain - PET Literature Validation                     ║");
    println!("║       Comparing Model Predictions vs Clinical PET Data           ║");
    println!("║       Target Error: < 5%                                         ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    let pk_db = PkDatabase::new();
    let lit_db = ClinicalLiteratureDb::new();

    let mut results: Vec<ValidationResult> = Vec::new();
    let tolerance = 0.05; // 5% error tolerance

    // ================================================================
    // BENZODIAZEPINES - GABA-A Occupancy
    // ================================================================
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  BENZODIAZEPINES (GABA-A via [11C]Flumazenil PET)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Diazepam 10mg oral
    // Literature: 15% occupancy (Lingford-Hughes 2002)
    // Ki for benzodiazepine site: ~3 nM (flumazenil displacement)
    if let Some(brain_um) = calculate_brain_concentration("diazepam", 10.0, 70.0, RouteOfAdministration::Oral, &pk_db) {
        let ki_nm = 3.0; // Approximate Ki for BZ site
        let model_occupancy = calculate_occupancy_from_ki(brain_um, ki_nm);
        let lit_occupancy = 15.0;

        let result = ValidationResult::new("diazepam", "GABA-A occupancy", model_occupancy, lit_occupancy, tolerance);
        print_result(&result);
        results.push(result);
    }

    // Midazolam 7.5mg IV
    // Literature: 35% occupancy
    if let Some(pk) = pk_db.get("midazolam") {
        // IV gives immediate distribution
        let dose_mg = 7.5;
        let weight_kg = 70.0;
        let vd = pk.vd_l_kg * weight_kg;
        let cmax_plasma_mg_l = dose_mg / vd;
        let cmax_plasma_um = (cmax_plasma_mg_l * 1000.0) / pk.molecular_weight;
        let brain_um = cmax_plasma_um * pk.brain_partition * (1.0 - pk.protein_binding);

        let ki_nm = 2.5; // Midazolam higher affinity
        let model_occupancy = calculate_occupancy_from_ki(brain_um, ki_nm);
        let lit_occupancy = 35.0;

        let result = ValidationResult::new("midazolam", "GABA-A occupancy", model_occupancy, lit_occupancy, tolerance);
        print_result(&result);
        results.push(result);
    }

    // Alprazolam 1mg oral
    // Literature: 22% occupancy
    if let Some(brain_um) = calculate_brain_concentration("alprazolam", 1.0, 70.0, RouteOfAdministration::Oral, &pk_db) {
        let ki_nm = 2.0;
        let model_occupancy = calculate_occupancy_from_ki(brain_um, ki_nm);
        let lit_occupancy = 22.0;

        let result = ValidationResult::new("alprazolam", "GABA-A occupancy", model_occupancy, lit_occupancy, tolerance);
        print_result(&result);
        results.push(result);
    }

    // ================================================================
    // ANTIPSYCHOTICS - D2 Occupancy
    // ================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  ANTIPSYCHOTICS (D2 via [11C]Raclopride PET)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Haloperidol 5mg oral
    // Literature: 70% D2 occupancy (Farde 1992)
    // Ki = 0.5 nM from ChEMBL
    if let Some(brain_um) = calculate_brain_concentration("haloperidol", 5.0, 70.0, RouteOfAdministration::Oral, &pk_db) {
        let ki_nm = 0.5;
        let model_occupancy = calculate_occupancy_from_ki(brain_um, ki_nm);
        let lit_occupancy = 70.0;

        let result = ValidationResult::new("haloperidol", "D2 occupancy", model_occupancy, lit_occupancy, tolerance);
        print_result(&result);
        results.push(result);
    }

    // Haloperidol 10mg oral
    // Literature: 80% D2 occupancy
    if let Some(brain_um) = calculate_brain_concentration("haloperidol", 10.0, 70.0, RouteOfAdministration::Oral, &pk_db) {
        let ki_nm = 0.5;
        let model_occupancy = calculate_occupancy_from_ki(brain_um, ki_nm);
        let lit_occupancy = 80.0;

        let result = ValidationResult::new("haloperidol 10mg", "D2 occupancy", model_occupancy, lit_occupancy, tolerance);
        print_result(&result);
        results.push(result);
    }

    // ================================================================
    // SSRIs - SERT Occupancy
    // ================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  SSRIs (SERT via [11C]DASB PET)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Fluoxetine 20mg oral (steady state)
    // Literature: 80% SERT occupancy (Meyer 2004)
    // Ki = 1.1 nM from ChEMBL
    if let Some(brain_um) = calculate_brain_concentration("fluoxetine", 20.0, 70.0, RouteOfAdministration::Oral, &pk_db) {
        // Steady state accumulation factor ~5x for fluoxetine (t1/2 = 72h)
        let steady_state_brain_um = brain_um * 5.0;
        let ki_nm = 1.1;
        let model_occupancy = calculate_occupancy_from_ki(steady_state_brain_um, ki_nm);
        let lit_occupancy = 80.0;

        let result = ValidationResult::new("fluoxetine (SS)", "SERT occupancy", model_occupancy, lit_occupancy, tolerance);
        print_result(&result);
        results.push(result);
    }

    // Sertraline 50mg oral (steady state)
    // Literature: 77% SERT occupancy
    // Ki = 0.1 nM (very high affinity)
    if let Some(pk) = pk_db.get("sertraline") {
        // Estimate steady state
        let dose_mg = 50.0;
        let weight_kg = 70.0;
        let vd = pk.vd_l_kg * weight_kg;
        let cmax_plasma_mg_l = (dose_mg * pk.bioavailability_oral) / vd;
        let cmax_plasma_um = (cmax_plasma_mg_l * 1000.0) / 306.2; // MW sertraline
        let brain_um = cmax_plasma_um * 5.0 * (1.0 - 0.98); // High protein binding
        let steady_state_brain_um = brain_um * 3.0; // t1/2 ~26h

        let ki_nm = 0.1;
        let model_occupancy = calculate_occupancy_from_ki(steady_state_brain_um, ki_nm);
        let lit_occupancy = 77.0;

        let result = ValidationResult::new("sertraline (SS)", "SERT occupancy", model_occupancy, lit_occupancy, tolerance);
        print_result(&result);
        results.push(result);
    }

    // ================================================================
    // OPIOIDS - μ-Opioid Occupancy
    // ================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  OPIOIDS (μ-OR via [11C]Carfentanil PET)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Morphine 10mg IV
    // Literature: 42% μ-OR occupancy (Melichar 2005)
    // Ki = 0.3 nM
    if let Some(pk) = pk_db.get("morphine") {
        let dose_mg = 10.0;
        let weight_kg = 70.0;
        let vd = pk.vd_l_kg * weight_kg;
        let cmax_plasma_mg_l = dose_mg / vd;
        let cmax_plasma_um = (cmax_plasma_mg_l * 1000.0) / pk.molecular_weight;
        let brain_um = cmax_plasma_um * pk.brain_partition * (1.0 - pk.protein_binding);

        let ki_nm = 0.3;
        let model_occupancy = calculate_occupancy_from_ki(brain_um, ki_nm);
        let lit_occupancy = 42.0;

        let result = ValidationResult::new("morphine", "μ-OR occupancy", model_occupancy, lit_occupancy, tolerance);
        print_result(&result);
        results.push(result);
    }

    // Fentanyl 100mcg IV
    // Literature: 35% μ-OR occupancy
    // Ki ~1 nM
    if let Some(pk) = pk_db.get("fentanyl") {
        let dose_mg = 0.1; // 100 mcg
        let weight_kg = 70.0;
        let vd = pk.vd_l_kg * weight_kg;
        let cmax_plasma_mg_l = dose_mg / vd;
        let cmax_plasma_um = (cmax_plasma_mg_l * 1000.0) / pk.molecular_weight;
        let brain_um = cmax_plasma_um * pk.brain_partition * (1.0 - pk.protein_binding);

        let ki_nm = 1.0;
        let model_occupancy = calculate_occupancy_from_ki(brain_um, ki_nm);
        let lit_occupancy = 35.0;

        let result = ValidationResult::new("fentanyl", "μ-OR occupancy", model_occupancy, lit_occupancy, tolerance);
        print_result(&result);
        results.push(result);
    }

    // ================================================================
    // ANESTHETICS
    // ================================================================
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  ANESTHETICS");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Propofol 2mg/kg IV
    // Literature: ~50% GABA-A occupancy at LOC
    if let Some(pk) = pk_db.get("propofol") {
        let dose_mg = 140.0; // 2 mg/kg * 70 kg
        let weight_kg = 70.0;
        let vd = pk.vd_l_kg * weight_kg;
        let cmax_plasma_mg_l = dose_mg / vd;
        let cmax_plasma_um = (cmax_plasma_mg_l * 1000.0) / pk.molecular_weight;
        let brain_um = cmax_plasma_um * pk.brain_partition * (1.0 - pk.protein_binding);

        // Propofol EC50 for GABA-A potentiation ~1-3 μM
        let ec50_um = 2.0;
        let model_occupancy = 100.0 * brain_um / (ec50_um + brain_um);
        let lit_occupancy = 50.0;

        let result = ValidationResult::new("propofol", "GABA-A effect", model_occupancy, lit_occupancy, tolerance);
        print_result(&result);
        results.push(result);
    }

    // ================================================================
    // SUMMARY
    // ================================================================
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                        VALIDATION SUMMARY                        ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");

    let total = results.len();
    let passed: Vec<_> = results.iter().filter(|r| r.within_tolerance).collect();
    let failed: Vec<_> = results.iter().filter(|r| !r.within_tolerance).collect();

    println!("║  Total validations:    {:>3}                                      ║", total);
    println!("║  PASS (<5% error):     {:>3}                                      ║", passed.len());
    println!("║  FAIL (>5% error):     {:>3}                                      ║", failed.len());

    let avg_error: f64 = results.iter().map(|r| r.percent_error).sum::<f64>() / total as f64;
    println!("║  Average error:      {:>5.1}%                                     ║", avg_error);

    let pass_rate = 100.0 * passed.len() as f64 / total as f64;
    println!("║  Pass rate:          {:>5.1}%                                     ║", pass_rate);
    println!("╚══════════════════════════════════════════════════════════════════╝");

    if !failed.is_empty() {
        println!("\n⚠️  Failed validations (>5% error):");
        for r in &failed {
            println!("   - {}: {:.1}% error (model: {:.1}%, lit: {:.1}%)",
                r.drug, r.percent_error, r.model_value, r.literature_value);
        }
    }

    println!("\n📚 References:");
    println!("   - Lingford-Hughes A et al. (2002) Neuropsychopharmacology 27:867");
    println!("   - Farde L et al. (1992) Arch Gen Psychiatry 49:538");
    println!("   - Meyer JH et al. (2004) Am J Psychiatry 161:826");
    println!("   - Melichar JK et al. (2005) Neuropsychopharmacology 30:516");
}

fn print_result(result: &ValidationResult) {
    let status = if result.within_tolerance { "✓ PASS" } else { "✗ FAIL" };
    let color_start = if result.within_tolerance { "" } else { "" };

    println!(
        "  {} {:20} | Model: {:5.1}% | Lit: {:5.1}% | Error: {:5.1}%",
        status,
        result.drug,
        result.model_value,
        result.literature_value,
        result.percent_error
    );
}
