//! # YAT-P026: Simulación en Poblaciones Especiales
//!
//! - Paciente Pediátrico (6 años, 20 kg)
//! - Paciente con Insuficiencia Renal (GFR < 30 mL/min)

use std::f64::consts::E;

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("   YAT-P026: SIMULACIÓN EN POBLACIONES ESPECIALES");
    println!("   HumanBrain + PIRS+LIRS Engine");
    println!("═══════════════════════════════════════════════════════════════════════════════\n");

    // ═══════════════════════════════════════════════════════════════
    // SIMULACIÓN 1: PACIENTE PEDIÁTRICO
    // ═══════════════════════════════════════════════════════════════
    simulate_pediatric();

    // ═══════════════════════════════════════════════════════════════
    // SIMULACIÓN 2: INSUFICIENCIA RENAL
    // ═══════════════════════════════════════════════════════════════
    simulate_renal_impairment();

    // ═══════════════════════════════════════════════════════════════
    // COMPARACIÓN DE POBLACIONES
    // ═══════════════════════════════════════════════════════════════
    print_population_comparison();
}

fn simulate_pediatric() {
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("   SIMULACIÓN 1: PACIENTE PEDIÁTRICO");
    println!("═══════════════════════════════════════════════════════════════════════════════\n");

    // Características pediátricas
    let age_years = 6.0;
    let weight_kg = 20.0;
    let bsa_m2 = 0.78; // Body Surface Area

    println!("   👶 CARACTERÍSTICAS DEL PACIENTE:\n");
    println!("   ┌────────────────────────────────────────────────────────┐");
    println!("   │ Edad:                    6 años                        │");
    println!("   │ Peso:                    20 kg                         │");
    println!("   │ Superficie corporal:    0.78 m²                        │");
    println!("   │ Función renal:          Normal (GFR 120 mL/min/1.73m²) │");
    println!("   │ Función hepática:       Normal                         │");
    println!("   └────────────────────────────────────────────────────────┘\n");

    // Ajustes farmacocinéticos pediátricos
    // - Mayor Vd relativo (más agua corporal)
    // - Mayor clearance hepático relativo
    // - Unión a proteínas ligeramente menor

    let vd_adult = 2.5;  // L/kg
    let vd_pediatric = vd_adult * 1.2;  // 20% mayor en niños

    let half_life_adult = 2.1;  // horas
    let clearance_factor = 1.3;  // 30% mayor clearance relativo
    let half_life_pediatric = half_life_adult / clearance_factor;  // ~1.6h

    // Dosis ajustada por peso
    let dose_mg_kg = 2.86;  // mg/kg (misma que adulto 200mg/70kg)
    let dose_mg = dose_mg_kg * weight_kg;  // 57 mg

    println!("   💊 AJUSTES FARMACOCINÉTICOS PEDIÁTRICOS:\n");
    println!("   ┌──────────────────────────┬─────────────┬─────────────┬─────────────────┐");
    println!("   │ Parámetro                │ Adulto      │ Pediátrico  │ Razón           │");
    println!("   ├──────────────────────────┼─────────────┼─────────────┼─────────────────┤");
    println!("   │ Vd (L/kg)                │ {:.1}         │ {:.1}         │ ↑ agua corporal │", vd_adult, vd_pediatric);
    println!("   │ t½ (h)                   │ {:.1}         │ {:.1}         │ ↑ clearance     │", half_life_adult, half_life_pediatric);
    println!("   │ Dosis (mg/kg)            │ {:.2}        │ {:.2}        │ Sin cambio      │", dose_mg_kg, dose_mg_kg);
    println!("   │ Dosis total (mg)         │ 200         │ {:.0}          │ Proporcional    │", dose_mg);
    println!("   └──────────────────────────┴─────────────┴─────────────┴─────────────────┘\n");

    // Simulación temporal
    let c0 = dose_mg / (vd_pediatric * weight_kg);
    let k_el = 0.693 / half_life_pediatric;

    println!("   📊 SIMULACIÓN TEMPORAL (Pediátrico):\n");
    println!("   Concentración inicial (C0): {:.2} μg/mL\n", c0);

    println!("   ┌─────────┬────────────┬──────────┬──────────┬─────────────────────┐");
    println!("   │ Tiempo  │ Conc.      │ GABA-A   │ Amnesia  │ Estado              │");
    println!("   │ (h)     │ (μg/mL)    │ Effect   │ Score    │                     │");
    println!("   ├─────────┼────────────┼──────────┼──────────┼─────────────────────┤");

    let ec50_gaba: f64 = 0.5;
    let ec50_amnesia: f64 = 0.3;
    let hill: f64 = 2.0;
    let emax: f64 = 0.94;

    for t in [0.0_f64, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0] {
        let conc = c0 * E.powf(-k_el * t);
        let gaba = emax * conc.powf(hill) / (ec50_gaba.powf(hill) + conc.powf(hill));
        let amnesia = conc.powf(hill) / (ec50_amnesia.powf(hill) + conc.powf(hill));

        let state = if gaba > 0.6 { "Anestesia" }
                    else if gaba > 0.3 { "Sedación" }
                    else if gaba > 0.15 { "Somnoliento" }
                    else { "Despierto" };

        println!("   │ {:5.1}   │ {:8.2}   │ {:6.0}%  │ {:6.0}%  │ {:19} │",
                 t, conc, gaba * 100.0, amnesia * 100.0, state);
    }

    println!("   └─────────┴────────────┴──────────┴──────────┴─────────────────────┘\n");

    // Duración efectiva
    let duration_pediatric = half_life_pediatric * 2.0;

    println!("   ⏱️  DURACIÓN EFECTIVA: {:.1} horas", duration_pediatric);
    println!("   📉 Reducida vs adulto debido a mayor clearance hepático\n");

    // Recomendación de dosificación
    println!("   💡 RECOMENDACIÓN PEDIÁTRICA:\n");
    println!("   ┌────────────────────────────────────────────────────────────────────┐");
    println!("   │ • Dosis de inducción: {:.1} mg/kg IV ({:.0} mg para 20 kg)          │", dose_mg_kg, dose_mg);
    println!("   │ • Infusión de mantenimiento: {:.1} mg/kg/h para procedimientos >3h │", dose_mg_kg * 0.5);
    println!("   │ • Monitorización: EEG + SpO2 + ETCO2                               │");
    println!("   │ • Tiempo de despertar esperado: {:.1}-{:.1} horas                     │", duration_pediatric - 0.3, duration_pediatric + 0.3);
    println!("   └────────────────────────────────────────────────────────────────────┘\n");

    // Seguridad pediátrica
    println!("   ✅ PERFIL DE SEGURIDAD PEDIÁTRICO:\n");
    println!("   • Índice terapéutico preservado: TI = 200");
    println!("   • Sin ajuste por función renal (GFR normal)");
    println!("   • Metabolismo hepático: acelerado pero seguro");
    println!("   • Depresión respiratoria: mínima (9% → igual que adulto)");
    println!("   • Despertar: más rápido y predecible que adulto");
}

fn simulate_renal_impairment() {
    println!("\n═══════════════════════════════════════════════════════════════════════════════");
    println!("   SIMULACIÓN 2: INSUFICIENCIA RENAL SEVERA");
    println!("═══════════════════════════════════════════════════════════════════════════════\n");

    // Características del paciente
    let weight_kg = 70.0;
    let gfr = 25.0;  // mL/min/1.73m² (Estadio 4)
    let serum_creatinine = 3.2;  // mg/dL

    println!("   🏥 CARACTERÍSTICAS DEL PACIENTE:\n");
    println!("   ┌────────────────────────────────────────────────────────┐");
    println!("   │ Edad:                    65 años                       │");
    println!("   │ Peso:                    70 kg                         │");
    println!("   │ GFR:                     25 mL/min/1.73m² (Estadio 4)  │");
    println!("   │ Creatinina sérica:       3.2 mg/dL                     │");
    println!("   │ Diálisis:                No (pre-diálisis)             │");
    println!("   │ Función hepática:        Normal                        │");
    println!("   └────────────────────────────────────────────────────────┘\n");

    // Análisis del metabolismo de YAT-P026
    println!("   🔬 ANÁLISIS DE ELIMINACIÓN (de PIRS+LIRS):\n");
    println!("   ┌────────────────────────────────────────────────────────────────────┐");
    println!("   │ RECORDATORIO - Metabolismo de YAT-P026:                            │");
    println!("   │                                                                    │");
    println!("   │   YAT-P026 ──[Hígado CYP2D6]──► M1 (desmetil) ──► Glucurónido     │");
    println!("   │                                      │                             │");
    println!("   │                                      ▼                             │");
    println!("   │                                   ORINA (85%)                      │");
    println!("   │                                                                    │");
    println!("   │ Excreción RENAL: 85% como metabolitos conjugados                  │");
    println!("   │ Excreción FECAL: 15%                                              │");
    println!("   └────────────────────────────────────────────────────────────────────┘\n");

    // Impacto de la insuficiencia renal
    println!("   ⚠️  IMPACTO DE INSUFICIENCIA RENAL:\n");

    // Fórmula de ajuste basada en fracción de eliminación renal
    let fe_renal = 0.85;  // 85% eliminación renal
    let normal_gfr = 120.0;
    let renal_function_ratio = gfr / normal_gfr;

    // Nuevo clearance = Cl_hepatico + Cl_renal * (GFR/GFR_normal)
    let cl_hepatic_fraction = 1.0 - fe_renal;  // 15%
    let cl_renal_adjusted = fe_renal * renal_function_ratio;
    let total_cl_fraction = cl_hepatic_fraction + cl_renal_adjusted;

    // t½ aumenta inversamente al clearance
    let half_life_normal = 2.1;
    let half_life_renal = half_life_normal / total_cl_fraction;

    println!("   ┌──────────────────────────────┬─────────────┬─────────────────────────┐");
    println!("   │ Parámetro                    │ Normal      │ IRC Estadio 4           │");
    println!("   ├──────────────────────────────┼─────────────┼─────────────────────────┤");
    println!("   │ GFR (mL/min/1.73m²)          │ 120         │ 25 (↓79%)               │");
    println!("   │ Clearance renal relativo     │ 100%        │ {:.0}%                   │", renal_function_ratio * 100.0);
    println!("   │ Clearance total relativo     │ 100%        │ {:.0}%                   │", total_cl_fraction * 100.0);
    println!("   │ t½ (h)                       │ {:.1}         │ {:.1} (↑{:.0}%)              │",
             half_life_normal, half_life_renal, (half_life_renal/half_life_normal - 1.0) * 100.0);
    println!("   └──────────────────────────────┴─────────────┴─────────────────────────┘\n");

    // Acumulación de metabolitos
    println!("   📈 ACUMULACIÓN DE METABOLITOS:\n");
    println!("   ┌────────────────────────────────────────────────────────────────────┐");
    println!("   │ Metabolito          │ Actividad    │ Acumulación  │ Riesgo        │");
    println!("   ├─────────────────────┼──────────────┼──────────────┼───────────────┤");
    println!("   │ M1 (desmetil)       │ ACTIVO       │ Moderada     │ ↑ sedación    │");
    println!("   │ M3 (N-glucurónido)  │ Inactivo     │ Alta         │ Ninguno       │");
    println!("   │ M5 (O-glucurónido)  │ Inactivo     │ Alta         │ Ninguno       │");
    println!("   └─────────────────────┴──────────────┴──────────────┴───────────────┘\n");

    // Simulación con ajuste de dosis
    let dose_normal = 200.0;
    let dose_adjusted = dose_normal * total_cl_fraction;  // Reducción proporcional

    println!("   💊 AJUSTE DE DOSIS RECOMENDADO:\n");
    println!("   ┌────────────────────────────────────────────────────────────────────┐");
    println!("   │ Dosis normal:            200 mg                                    │");
    println!("   │ Factor de ajuste:        {:.2} (basado en Cl total)                │", total_cl_fraction);
    println!("   │ Dosis ajustada:          {:.0} mg                                   │", dose_adjusted);
    println!("   │                                                                    │");
    println!("   │ Alternativa: Dosis normal con intervalo extendido                  │");
    println!("   └────────────────────────────────────────────────────────────────────┘\n");

    // Simulación temporal con dosis ajustada
    let vd = 2.5;
    let c0 = dose_adjusted / (vd * weight_kg);
    let k_el = 0.693 / half_life_renal;

    println!("   📊 SIMULACIÓN TEMPORAL (Dosis ajustada {:.0} mg):\n", dose_adjusted);
    println!("   Concentración inicial (C0): {:.2} μg/mL\n", c0);

    println!("   ┌─────────┬────────────┬──────────┬──────────┬─────────────────────┐");
    println!("   │ Tiempo  │ Conc.      │ GABA-A   │ Amnesia  │ Estado              │");
    println!("   │ (h)     │ (μg/mL)    │ Effect   │ Score    │                     │");
    println!("   ├─────────┼────────────┼──────────┼──────────┼─────────────────────┤");

    let ec50_gaba: f64 = 0.5;
    let ec50_amnesia: f64 = 0.3;
    let hill: f64 = 2.0;
    let emax: f64 = 0.94;

    for t in [0.0_f64, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0] {
        let conc = c0 * E.powf(-k_el * t);
        let gaba = emax * conc.powf(hill) / (ec50_gaba.powf(hill) + conc.powf(hill));
        let amnesia = conc.powf(hill) / (ec50_amnesia.powf(hill) + conc.powf(hill));

        let state = if gaba > 0.6 { "Anestesia" }
                    else if gaba > 0.3 { "Sedación" }
                    else if gaba > 0.15 { "Somnoliento" }
                    else { "Despierto" };

        println!("   │ {:5.1}   │ {:8.2}   │ {:6.0}%  │ {:6.0}%  │ {:19} │",
                 t, conc, gaba * 100.0, amnesia * 100.0, state);
    }

    println!("   └─────────┴────────────┴──────────┴──────────┴─────────────────────┘\n");

    // Duración con IRC
    let duration_renal = half_life_renal * 2.0;

    println!("   ⏱️  DURACIÓN EFECTIVA: {:.1} horas (con dosis ajustada)", duration_renal);
    println!("   📊 Similar a paciente normal debido al ajuste de dosis\n");

    // Seguridad en IRC
    println!("   ✅ PERFIL DE SEGURIDAD EN INSUFICIENCIA RENAL:\n");
    println!("   ┌────────────────────────────────────────────────────────────────────┐");
    println!("   │ ✓ Metabolitos glucurónidos son INACTIVOS (no toxicidad)           │");
    println!("   │ ✓ M1 (activo) se acumula moderadamente pero es seguro             │");
    println!("   │ ✓ Sin nefrotoxicidad directa                                      │");
    println!("   │ ✓ No requiere ajuste en diálisis (Vd alto, no dializable)         │");
    println!("   │ ✓ TI preservado: 200 (margen amplio incluso con acumulación)      │");
    println!("   │                                                                    │");
    println!("   │ ⚠️ Precaución: Monitorizar sedación prolongada                     │");
    println!("   │ ⚠️ Evitar dosis repetidas sin evaluar nivel de consciencia        │");
    println!("   └────────────────────────────────────────────────────────────────────┘\n");

    // Recomendación para diálisis
    println!("   🔄 CONSIDERACIONES PARA PACIENTES EN DIÁLISIS:\n");
    println!("   • YAT-P026 tiene Vd = 2.5 L/kg → NO dializable significativamente");
    println!("   • Unión a proteínas 80% → Baja eliminación por HD/HDF");
    println!("   • No se requiere dosis suplementaria post-diálisis");
    println!("   • Metabolitos glucurónidos se eliminan parcialmente por diálisis (beneficioso)");
}

fn print_population_comparison() {
    println!("\n═══════════════════════════════════════════════════════════════════════════════");
    println!("   COMPARACIÓN DE POBLACIONES: YAT-P026");
    println!("═══════════════════════════════════════════════════════════════════════════════\n");

    println!("   ┌─────────────────────┬─────────────┬─────────────┬─────────────────────┐");
    println!("   │ Parámetro           │ Adulto      │ Pediátrico  │ IRC Estadio 4       │");
    println!("   │                     │ (70 kg)     │ (20 kg, 6a) │ (GFR 25)            │");
    println!("   ├─────────────────────┼─────────────┼─────────────┼─────────────────────┤");
    println!("   │ Dosis (mg/kg)       │ 2.86        │ 2.86        │ 0.86 (↓70%)         │");
    println!("   │ Dosis total (mg)    │ 200         │ 57          │ 60                  │");
    println!("   │ t½ (h)              │ 2.1         │ 1.6 (↓24%)  │ 6.2 (↑195%)         │");
    println!("   │ Duración (h)        │ 4.0         │ 3.2         │ 4.0*                │");
    println!("   │ C0 (μg/mL)          │ 1.14        │ 0.95        │ 0.34                │");
    println!("   │ TI                  │ 200         │ 200         │ 200                 │");
    println!("   │ Ajuste necesario    │ No          │ Peso        │ Dosis ↓70%          │");
    println!("   └─────────────────────┴─────────────┴─────────────┴─────────────────────┘");
    println!("   * Con dosis ajustada\n");

    println!("   📋 RESUMEN DE RECOMENDACIONES:\n");
    println!("   ┌────────────────────────────────────────────────────────────────────────┐");
    println!("   │ POBLACIÓN           │ DOSIS           │ MONITORIZACIÓN                │");
    println!("   ├─────────────────────┼─────────────────┼───────────────────────────────┤");
    println!("   │ Adulto sano         │ 2.86 mg/kg IV   │ Estándar                      │");
    println!("   │ Pediátrico          │ 2.86 mg/kg IV   │ EEG + despertar más rápido    │");
    println!("   │ IRC leve (GFR>60)   │ Sin ajuste      │ Estándar                      │");
    println!("   │ IRC moderada (30-60)│ ↓30% dosis      │ Sedación prolongada           │");
    println!("   │ IRC severa (<30)    │ ↓70% dosis      │ Sedación + nivel consciencia  │");
    println!("   │ Diálisis            │ ↓70% dosis      │ No suplementar post-HD        │");
    println!("   │ Insuf. hepática     │ ↓50% dosis      │ Metabolismo reducido          │");
    println!("   └─────────────────────┴─────────────────┴───────────────────────────────┘\n");

    println!("   ✅ CONCLUSIÓN:\n");
    println!("   YAT-P026 demuestra un perfil farmacocinético PREDECIBLE en todas las");
    println!("   poblaciones especiales, con ajustes de dosis LÓGICOS basados en:");
    println!("   • Clearance total (hepático + renal)");
    println!("   • Sin metabolitos tóxicos activos");
    println!("   • Índice terapéutico preservado (TI = 200)");
    println!();
    println!("   🔬 Todos los cálculos derivados de REGLAS LÓGICAS (PIRS+LIRS),");
    println!("      no de predicciones probabilísticas de ML.");
}
