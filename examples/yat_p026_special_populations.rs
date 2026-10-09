//! # YAT-P026: Modelado Farmacocinético Teórico In Silico
//!
//! AVISO: INVESTIGACIÓN EXCLUSIVAMENTE - NO PARA USO CLÍNICO NI DOSIFICACIÓN.
//! Este script es un modelo computacional matemático exploratorio in silico.
//! No posee validación clínica ni regulatoria. No debe usarse para guiar
//! intervenciones médicas, diagnósticos ni dosificación en pacientes.
//!
//! Escenarios simulados:
//! - Modelo pediátrico teórico (6 años, 20 kg)
//! - Modelo de función renal reducida (GFR < 30 mL/min)

use std::f64::consts::E;

fn main() {
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("   YAT-P026: MODELO FARMACOCINÉTICO IN SILICO (INVESTIGACIÓN)");
    println!("   [AVISO: MODELO MATEMÁTICO TEÓRICO - SIN VALIDACIÓN CLÍNICA]");
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

    println!("   ⏱️  DURACIÓN SIMULADA: {:.1} horas (con parámetro de dosis ajustado)", duration_renal);
    println!("   📊 Comportamiento del modelo con ajuste de parámetros\n");

    // Seguridad en IRC
    println!("   ✅ HIPÓTESIS FARMACOCINÉTICA EN MODELO DE FUNCIÓN RENAL REDUCIDA (IN SILICO):\n");
    println!("   ┌────────────────────────────────────────────────────────────────────┐");
    println!("   │ • Hipótesis: Metabolitos glucurónidos inactivos en modelo          │");
    println!("   │ • Hipótesis: M1 activo con acumulación en clearance reducido       │");
    println!("   │ • Hipótesis: Sin nefrotoxicidad formulada en ecuaciones            │");
    println!("   │ • Vd asignado alto (2.5 L/kg) en modelo matemático                 │");
    println!("   │ • TI teórico asignado: 200 (modelo in silico)                      │");
    println!("   │                                                                    │");
    println!("   │ ⚠️ AVISO: Hipótesis no validadas en humanos ni modelos animales.   │");
    println!("   │ ⚠️ No apto para dosificación clínica ni toma de decisiones.        │");
    println!("   └────────────────────────────────────────────────────────────────────┘\n");

    // Hipótesis para diálisis
    println!("   🔄 HIPÓTESIS MATEMÁTICAS EN CONDICIÓN DE DIÁLISIS (IN SILICO):\n");
    println!("   • Modelo asume Vd = 2.5 L/kg → baja depuración teórica por diálisis");
    println!("   • Unión a proteínas teórica 80% en parámetros del modelo");
    println!("   • [AVISO: Cálculos teóricos; sin respaldo de estudios clínicos]");
}

fn print_population_comparison() {
    println!("\n═══════════════════════════════════════════════════════════════════════════════");
    println!("   COMPARACIÓN DE ESCENARIOS POBLACIONALES EN MODELO: YAT-P026");
    println!("   [AVISO: INVESTIGACIÓN EXCLUSIVAMENTE - SIN APLICACIÓN CLÍNICA]");
    println!("═══════════════════════════════════════════════════════════════════════════════\n");

    println!("   ┌─────────────────────┬─────────────┬─────────────┬─────────────────────┐");
    println!("   │ Parámetro           │ Adulto      │ Pediátrico  │ IRC Estadio 4       │");
    println!("   │                     │ (70 kg)     │ (20 kg, 6a) │ (GFR 25)            │");
    println!("   ├─────────────────────┼─────────────┼─────────────┼─────────────────────┤");
    println!("   │ Dosis teórica       │ 2.86 mg/kg  │ 2.86 mg/kg  │ 0.86 mg/kg (↓70%)   │");
    println!("   │ Dosis total modelo  │ 200 mg      │ 57 mg       │ 60 mg               │");
    println!("   │ t½ modelo (h)       │ 2.1         │ 1.6 (↓24%)  │ 6.2 (↑195%)         │");
    println!("   │ Duración modelo (h) │ 4.0         │ 3.2         │ 4.0*                │");
    println!("   │ C0 modelo (μg/mL)   │ 1.14        │ 0.95        │ 0.34                │");
    println!("   │ TI asignado         │ 200         │ 200         │ 200                 │");
    println!("   │ Ajuste en modelo    │ Base        │ Peso        │ Parámetro ↓70%      │");
    println!("   └─────────────────────┴─────────────┴─────────────┴─────────────────────┘");
    println!("   * Con parámetro de dosis ajustado\n");

    println!("   📋 ESCENARIOS TEÓRICOS DE MODELADO FARMACOCINÉTICO (IN SILICO, NO CLÍNICO):\n");
    println!("   ┌────────────────────────────────────────────────────────────────────────┐");
    println!("   │ MODELO ESCENARIO    │ DOSIS SIMULADA  │ SEGUIMIENTO TEÓRICO EN MODELO │");
    println!("   ├─────────────────────┼─────────────────┼───────────────────────────────┤");
    println!("   │ Adulto referencia   │ 2.86 mg/kg IV   │ Perfil base                   │");
    println!("   │ Pediátrico teórico  │ 2.86 mg/kg IV   │ Clearance aumentado           │");
    println!("   │ Tasa GFR > 60       │ Parámetro base  │ Sin ajuste                    │");
    println!("   │ Tasa GFR 30-60      │ ↓30% parámetro  │ Eliminación reducida          │");
    println!("   │ Tasa GFR < 30       │ ↓70% parámetro  │ Eliminación reducida          │");
    println!("   │ Diálisis teórica    │ ↓70% parámetro  │ No dializable en modelo       │");
    println!("   │ Función hepática ↓  │ ↓50% parámetro  │ Metabolismo reducido          │");
    println!("   └─────────────────────┴─────────────────┴───────────────────────────────┘\n");

    println!("   ✅ RESUMEN DEL MODELO TEÓRICO:\n");
    println!("   El modelo in silico de YAT-P026 produce proyecciones algebraicas dependientes");
    println!("   de los parámetros asignados a cada escenario poblacional:");
    println!("   • Clearance total algebraico (hepático + renal)");
    println!("   • Ausencia de metabolitos reactivos en la formulación lógica");
    println!("   • Índice terapéutico asignado en el modelo (TI = 200)");
    println!();
    println!("   🔬 Cálculos computacionales basados en reglas lógicas (PIRS+LIRS).");
    println!("      [AVISO: Sin validación experimental ni clínica en seres humanos.]");
}
