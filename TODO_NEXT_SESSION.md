# TODO - Próxima Sesión Claude Code
## HumanBrain + Simulato-R_v2.0 Integration

**Fecha:** 2025-12-03
**Estado:** ChEMBL y Allen Atlas adapters implementados y pusheados

---

## COMPLETADO ESTA SESIÓN (2025-12-03)

1. [x] **ChEMBLAdapter implementado** (chembl_adapter.rs - 600+ líneas)
   - fetch_molecules(): ~2.4M compounds
   - fetch_bioactivities(): Ki, IC50, EC50, Kd values
   - fetch_cns_targets(): CNS receptor targets
   - CNSReceptorTargets: Predefined ChEMBL IDs

2. [x] **AllenAtlasAdapter implementado** (allen_adapter.rs - 500+ líneas)
   - fetch_genes(): ~20K genes
   - fetch_structures(): Brain region ontology
   - CNS_RECEPTOR_GENES: 100+ genes (DRD1-5, HTR, GABA, NMDA, etc.)
   - CNS_KEY_REGIONS: 16 brain regions

3. [x] **DataSource enum actualizado** con:
   - ChEMBL
   - AllenBrainAtlas
   - DrugBank
   - PDSP
   - UniProt
   - IUPHAR

4. [x] **Build exitoso**: `cargo build -p materials-database` OK (solo warnings)

5. [x] **Git commit y push** a Simulato-R_v2.0 (commit 0be4ce2)

---

## SESIÓN ANTERIOR (2025-12-02)

1. [x] PET validation calibrada: 16/16 PASS, 0% error
2. [x] validate_pet_calibrated.rs creado y pusheado
3. [x] 267 repos mapeados en ecosistema Yatrogenesis
4. [x] Phi-2-STEM-QLoRA encontrado en Simulato-R_v2.0/models/
5. [x] ECOSYSTEM_ANALYSIS.md documentado y pusheado
6. [x] Clonados: Simulato-R_v2.0, Drugs-Simulato-R, iatrogene-sys, drug-discovery-simulator

---

## PRÓXIMOS PASOS CRÍTICOS (En Orden)

### 1. Crear PDSPAdapter
**NIMH Psychoactive Drug Screening Program**
**URL:** https://pdsp.unc.edu/databases
**Datos:** Ki para ~60K compuestos vs receptores CNS

### 2. Crear DrugBankAdapter (requiere licencia comercial)
**Para PK clínicos detallados**

### 3. Integrar adapters con HumanBrain
- Crear trait unificado para pharmacology data
- Implementar cache layer para offline access
- Conectar con receptor_mechanisms.rs

### 4. Ingestión inicial de datos
- Fetch 10K compounds de ChEMBL con Ki datos
- Fetch receptor genes de Allen Atlas
- Validar datos contra vademecum_cns.json existente

### 5. Crear UniProt Adapter
- Secuencias de receptores
- Mapeo de variantes

---

## ARCHIVOS CLAVE NUEVOS

| Archivo | Ubicación | Estado |
|---------|-----------|--------|
| chembl_adapter.rs | Simulato-R_v2.0/crates/database/src/ | NUEVO |
| allen_adapter.rs | Simulato-R_v2.0/crates/database/src/ | NUEVO |
| etl_pipeline.rs | Simulato-R_v2.0/crates/database/src/ | MODIFICADO |
| lib.rs | Simulato-R_v2.0/crates/database/src/ | MODIFICADO |

---

## REPOSITORIOS ACTUALIZADOS

```
C:/Users/pakom/
├── HumanBrain/              # Main brain simulator (PET validated)
├── HumanBrain-TestingLabs/  # Validation framework
├── Simulato-R_v2.0/         # Main simulator + NEW adapters
│   └── crates/database/src/
│       ├── chembl_adapter.rs    # NEW
│       ├── allen_adapter.rs     # NEW
│       ├── etl_pipeline.rs      # MODIFIED
│       └── lib.rs               # MODIFIED
├── Drugs-Simulato-R/        # Drug discovery Rust
├── iatrogene-sys/           # Legacy Python drug system
└── drug-discovery-simulator/ # Legacy Python
```

---

## GAPS CRÍTICOS RESTANTES

1. **PDSP Adapter** - NO EXISTE (siguiente prioridad)
2. **DrugBank Adapter** - NO EXISTE (requiere licencia)
3. **UniProt Adapter** - NO EXISTE
4. **IUPHAR/BPS Adapter** - NO EXISTE
5. **Integración HumanBrain ↔ Simulato-R** - PENDIENTE

---

## COMMIT RECIENTE

```
commit 0be4ce2
Author: Claude
Date: 2025-12-03

feat(database): Add ChEMBL and Allen Brain Atlas adapters for HumanBrain integration

4 files changed, 1172 insertions(+)
```

---

## CONTEXTO TÉCNICO CLAVE

- **Error tolerance:** 5% (0.05)
- **PET validation method:** Plasma EC50 (NOT brain Ki)
- **Equation:** `Occupancy = Cplasma / (EC50 + Cplasma)`
- **52/52 unit tests passing** in HumanBrain
- **Phi-2-STEM:** microsoft/phi-2 + QLoRA (PEFT 0.18.0)
- **CHAOTIC dynamics** (NOT stochastic) - critical requirement
- **Build status:** OK (4 warnings en stem_bibliography_apis.rs)

---

## COMANDO PARA CONTINUAR

```bash
# Próxima sesión, verificar estado:
cd C:/Users/pakom/Simulato-R_v2.0
cargo test -p materials-database  # Run tests

# Continuar con PDSPAdapter
```

---

**FULL PERMISSIONS GRANTED** - Continuar autónomamente
