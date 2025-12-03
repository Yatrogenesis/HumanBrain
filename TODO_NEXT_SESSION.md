# TODO - Próxima Sesión Claude Code
## HumanBrain + Simulato-R_v2.0 Integration

**Fecha:** 2025-12-02
**Estado:** Calibración PET completada, falta integración con Simulato-R

---

## COMPLETADO ESTA SESIÓN

1. [x] PET validation calibrada: 16/16 PASS, 0% error
2. [x] validate_pet_calibrated.rs creado y pusheado
3. [x] 267 repos mapeados en ecosistema Yatrogenesis
4. [x] Phi-2-STEM-QLoRA encontrado en Simulato-R_v2.0/models/
5. [x] ECOSYSTEM_ANALYSIS.md documentado y pusheado
6. [x] Clonados: Simulato-R_v2.0, Drugs-Simulato-R, iatrogene-sys, drug-discovery-simulator

---

## PRÓXIMOS PASOS CRÍTICOS (En Orden)

### 1. Implementar ChEMBLAdapter en Simulato-R_v2.0
**Ubicación:** `C:/Users/pakom/Simulato-R_v2.0/crates/database/src/chembl_adapter.rs`
**API Base:** https://www.ebi.ac.uk/chembl/api/data
**Endpoints:**
- `/molecule.json` - 2.4M compuestos
- `/activity.json` - 19M bioactividades (Ki, IC50, EC50)
- `/target.json` - 15K targets (receptores)
- `/drug_indication.json` - Indicaciones clínicas

### 2. Implementar AllenAtlasAdapter
**Ubicación:** `C:/Users/pakom/Simulato-R_v2.0/crates/database/src/allen_adapter.rs`
**API Base:** http://api.brain-map.org/api/v2
**Endpoints:**
- `/data/query.json?criteria=model::Gene` - 20K genes
- `/data/SectionDataSet/query.json` - Expresión por región
- `/data/Structure/query.json` - Regiones cerebrales

### 3. Actualizar etl_pipeline.rs
**Añadir:**
```rust
pub enum DataSource {
    // ... existing ...
    ChEMBL,
    AllenBrainAtlas,
    DrugBank,
    PDSP,
}
```

### 4. Crear DrugBankAdapter (requiere licencia comercial)
**Para PK clínicos detallados**

### 5. Crear PDSPAdapter
**NIMH Psychoactive Drug Screening Program**
**URL:** https://pdsp.unc.edu/databases
**Datos:** Ki para ~60K compuestos vs receptores CNS

---

## ARCHIVOS CLAVE MODIFICADOS

| Archivo | Ubicación | Estado |
|---------|-----------|--------|
| validate_pet_calibrated.rs | HumanBrain/crates/pharmacology/examples/ | NUEVO |
| validate_pet_literature.rs | HumanBrain/crates/pharmacology/examples/ | NUEVO |
| ECOSYSTEM_ANALYSIS.md | HumanBrain/docs/ | NUEVO |
| etl_pipeline.rs | Simulato-R_v2.0/crates/database/src/ | LEER |
| stem_bibliography_apis.rs | Simulato-R_v2.0/crates/database/src/ | LEER |

---

## REPOSITORIOS CLONADOS LOCALMENTE

```
C:/Users/pakom/
├── HumanBrain/              # Main brain simulator
├── HumanBrain-TestingLabs/  # Validation framework
├── Simulato-R_v2.0/         # Main simulator (Materials + Drugs)
├── Drugs-Simulato-R/        # Drug discovery Rust
├── iatrogene-sys/           # Legacy Python drug system
└── drug-discovery-simulator/ # Legacy Python
```

---

## DATOS LOCALES DISPONIBLES

| Archivo | Ubicación | Contenido |
|---------|-----------|-----------|
| vademecum_cns.json | HumanBrain-TestingLabs/data/ | 79 drugs, 28 receptors |
| allen_api_genes.json | HumanBrain-TestingLabs/data/allen_atlas/ | 18 genes, 16 regions |
| phi-stem-qlora/ | Simulato-R_v2.0/models/ | 21MB QLoRA adapter |

---

## GAPS CRÍTICOS IDENTIFICADOS

1. **ChEMBL Adapter** - NO EXISTE en Rust
2. **Allen Atlas Adapter** - NO EXISTE en Rust
3. **DrugBank Adapter** - NO EXISTE
4. **PDSP Adapter** - NO EXISTE
5. **UniProt Adapter** - NO EXISTE
6. **IUPHAR/BPS Adapter** - NO EXISTE

---

## COMANDO PARA CONTINUAR

```bash
# Próxima sesión, ejecutar:
cd C:/Users/pakom/Simulato-R_v2.0
cargo build  # Verificar estado

# Luego implementar chembl_adapter.rs
```

---

## CONTEXTO TÉCNICO CLAVE

- **Error tolerance:** 5% (0.05)
- **PET validation method:** Plasma EC50 (NOT brain Ki)
- **Equation:** `Occupancy = Cplasma / (EC50 + Cplasma)`
- **52/52 unit tests passing** in HumanBrain
- **Phi-2-STEM:** microsoft/phi-2 + QLoRA (PEFT 0.18.0)
- **CHAOTIC dynamics** (NOT stochastic) - critical requirement

---

**FULL PERMISSIONS GRANTED** - Continuar autónomamente
