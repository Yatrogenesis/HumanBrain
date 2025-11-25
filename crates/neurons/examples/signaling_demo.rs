//! Demo: Intracellular signaling cascades
//!
//! Demonstrates how molecular signaling pathways respond to different
//! receptor activations and calcium influx patterns.

use neurons::IntracellularSignaling;

fn main() {
    println!("=== Intracellular Signaling Demo ===\n");

    // Demo 1: D1 dopamine receptor activation (Gs/cAMP/PKA)
    println!("Demo 1: D1 Dopamine Receptor Activation");
    println!("----------------------------------------");
    let mut sig = IntracellularSignaling::default();

    println!("Initial state:");
    println!("  cAMP: {:.3} µM", sig.camp);
    println!("  PKA activity: {:.3}", sig.pka_activity);

    // Simulate sustained D1 activation (1 second)
    for _ in 0..100 {
        sig.activate_gs_pathway(0.8, 0.01);
    }

    println!("\nAfter 1s of D1 activation (80%):");
    println!("  cAMP: {:.3} µM", sig.camp);
    println!("  PKA activity: {:.3}", sig.pka_activity);
    println!("  Weight modulation: {:.3}x", sig.synaptic_weight_modulation());

    // Demo 2: NMDA-mediated calcium influx (Ca/CaM/CaMKII)
    println!("\n\nDemo 2: NMDA Receptor Activation");
    println!("----------------------------------");
    let mut sig2 = IntracellularSignaling::default();

    println!("Initial state:");
    println!("  Ca²⁺: {:.6} µM ({:.1} nM)", sig2.calcium, sig2.calcium * 1000.0);
    println!("  CaMKII activity: {:.3}", sig2.camkii_activity);

    // Strong NMDA burst (500ms)
    for _ in 0..50 {
        sig2.update_calcium_signaling(0.01, 0.01);  // 10 µM/s influx
    }

    println!("\nDuring NMDA burst (500ms):");
    println!("  Ca²⁺: {:.6} µM ({:.1} nM)", sig2.calcium, sig2.calcium * 1000.0);
    println!("  Ca₄-CaM: {:.3}", sig2.cam_ca4);
    println!("  CaMKII activity: {:.3}", sig2.camkii_activity);

    // Continue without influx to show CaMKII persistence
    for _ in 0..100 {
        sig2.update_calcium_signaling(0.0, 0.01);
    }

    println!("\nAfter 1s without Ca²⁺ influx:");
    println!("  Ca²⁺: {:.6} µM (back to baseline)", sig2.calcium);
    println!("  CaMKII activity: {:.3} (persistent!)", sig2.camkii_activity);
    println!("  Weight modulation: {:.3}x", sig2.synaptic_weight_modulation());

    // Demo 3: Combined pathways for LTP induction
    println!("\n\nDemo 3: LTP Induction (D1 + NMDA)");
    println!("-----------------------------------");
    let mut sig3 = IntracellularSignaling::default();

    // Simulate coincident D1 and NMDA activation
    for i in 0..200 {
        let d1_activation = if i < 100 { 0.7 } else { 0.0 };
        let mglu_activation = 0.0;
        let ca_influx = if i < 50 { 0.01 } else { 0.0 };

        sig3.step(0.01, d1_activation, mglu_activation, ca_influx);

        if i % 50 == 0 {
            println!("\nt = {}ms:", i * 10);
            println!("  cAMP: {:.3} µM, PKA: {:.3}", sig3.camp, sig3.pka_activity);
            println!("  Ca²⁺: {:.6} µM, CaMKII: {:.3}", sig3.calcium, sig3.camkii_activity);
            println!("  pCREB: {:.3}", sig3.creb_phospho);
            println!("  Weight modulation: {:.3}x", sig3.synaptic_weight_modulation());
        }
    }

    println!("\n\nFinal state (after 2s):");
    println!("  IEG expression: {:.3}", sig3.ieg_expression);
    println!("  pCREB: {:.3}", sig3.creb_phospho);
    println!("  Synaptic weight: {:.3}x ({}% potentiation)",
             sig3.synaptic_weight_modulation(),
             (sig3.synaptic_weight_modulation() - 1.0) * 100.0);

    println!("\n=== End of Demo ===");
}
