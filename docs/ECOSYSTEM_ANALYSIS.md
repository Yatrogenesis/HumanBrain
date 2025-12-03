# YATROGENESIS Ecosystem Analysis
## HumanBrain ↔ Simulato-R_v2.0 Integration Status

**Author:** Claude Code Analysis
**Date:** 2025-12-02
**Total Repositories:** 267

---

## 1. Project Evolution Chain

```
LEGACY PYTHON                          RUST REFACTOR
==============                          =============

Materials-SimPro ─────────────────────► Materials-Simulato-R
         +                                      +
Drug-Discovery-Simulator ────────────► Drugs-Simulato-R
         +                                      │
AutoResearcher                                  ▼
         │                              Simulato-R_v2.0
         ▼                                      +
iatrogene-sys ───────────────────────► (integrated)
                                               +
                                        HumanBrain (NEW)
                                               +
                                        HumanBrain-TestingLabs
```

## 2. Parallel Framework Projects

| Project | Purpose | Status |
|---------|---------|--------|
| CORTEXIA-Framework | Consciousness framework | Active |
| LIRS-Lab | Symbolic reasoning engine | Active |
| Qualia-Naturalia | IIT 3.0 consciousness | Research |
| PP25-CHAOTIC_ATTRACTOR_COMPRESSION | Embedding compression | Published |
| AION-CR | Regulatory compliance | Active |
| yatrogenesis-ai | Type-safe ML | Active |
| candle-phi | Phi inference | Active |

## 3. Database Infrastructure Status

### 3.1 Simulato-R_v2.0 - STEM Bibliography APIs (IMPLEMENTED)

| API | Status | Records |
|-----|--------|---------|
| arXiv | ✅ Scaffolded | ~2M papers |
| PubMed/PMC | ✅ Scaffolded | ~35M papers |
| Semantic Scholar | ✅ Scaffolded | ~200M papers |
| OpenAlex | ✅ Scaffolded | ~250M works |
| Materials Project | ✅ Scaffolded | ~150K materials |
| NREL | ✅ Scaffolded | Energy materials |
| NOMAD | ✅ Scaffolded | DFT calculations |

### 3.2 Simulato-R_v2.0 - ETL Pipeline (IMPLEMENTED)

| Source | Adapter | Status |
|--------|---------|--------|
| Materials Project | MaterialsProjectAdapter | ✅ Mock data |
| PubChem | PubChemAdapter | ✅ Mock data |
| ICSD | DataSource::ICSD | ⚠️ Enum only |
| OQMD | DataSource::OQMD | ⚠️ Enum only |
| AFLOW | DataSource::AFLOW | ⚠️ Enum only |
| COD | DataSource::CrystallographyOpenDatabase | ⚠️ Enum only |

### 3.3 CRITICAL GAPS for Neuropharmacology

| Database | Records | Purpose | Status |
|----------|---------|---------|--------|
| **ChEMBL** | ~2.4M compounds | Drug bioactivity, Ki values | ❌ NO ADAPTER |
| **DrugBank** | ~14K drugs | Clinical PK, interactions | ❌ NO ADAPTER |
| **Allen Brain Atlas** | ~20K genes | Receptor expression by region | ❌ NO ADAPTER |
| **PDSP/NIMH** | ~60K records | Receptor binding profiles | ❌ NO ADAPTER |
| **IUPHAR/BPS** | ~3K targets | Pharmacological targets | ❌ NO ADAPTER |
| **UniProt** | ~500K proteins | Receptor sequences | ❌ NO ADAPTER |
| **Guide to Pharmacology** | ~1.5K ligands | Receptor interactions | ❌ NO ADAPTER |

## 4. HumanBrain Current Data Status

### 4.1 Local Vademecum (vademecum_cns.json)
- **Total drugs:** 79
- **BBB penetrant:** 74
- **Source:** ChEMBL (queried)
- **Receptors queried:** 28
- **Coverage:** ~0.003% of ChEMBL

### 4.2 Local Allen Atlas (allen_api_genes.json)
- **Genes:** 18 (receptor genes only)
- **Brain regions:** 16
- **Coverage:** ~0.09% of Allen Atlas

### 4.3 PET Validation Data (clinical_literature.rs)
- **Validated drugs:** 16
- **Average error:** 0.0%
- **Pass rate:** 100%

## 5. Required Adapters for Full Integration

### 5.1 ChEMBL Adapter (PRIORITY: CRITICAL)

```rust
pub struct ChEMBLAdapter {
    api_key: Option<String>,
    base_url: String, // https://www.ebi.ac.uk/chembl/api/data
}

impl SourceAdapter for ChEMBLAdapter {
    async fn fetch_compounds(&self, limit: u64) -> Result<Vec<RawDrugData>>;
    async fn fetch_bioactivities(&self, target: &str) -> Result<Vec<Bioactivity>>;
    async fn fetch_drug_indications(&self) -> Result<Vec<DrugIndication>>;
}
```

**Endpoints needed:**
- `/molecule.json` - 2.4M compounds
- `/activity.json` - 19M bioactivities
- `/target.json` - 15K targets
- `/assay.json` - 1.5M assays
- `/drug_indication.json` - Drug indications

### 5.2 Allen Brain Atlas Adapter (PRIORITY: HIGH)

```rust
pub struct AllenAtlasAdapter {
    base_url: String, // http://api.brain-map.org/api/v2
}

impl SourceAdapter for AllenAtlasAdapter {
    async fn fetch_gene_expression(&self, gene: &str) -> Result<ExpressionData>;
    async fn fetch_brain_regions(&self) -> Result<Vec<BrainRegion>>;
    async fn fetch_section_images(&self, gene_id: u64) -> Result<Vec<SectionImage>>;
}
```

**Endpoints needed:**
- `/data/query.json?criteria=model::Gene` - Gene metadata
- `/data/SectionDataSet/query.json` - Expression datasets
- `/data/Structure/query.json` - Brain regions

### 5.3 DrugBank Adapter (PRIORITY: MEDIUM)

```rust
pub struct DrugBankAdapter {
    api_key: String, // Commercial API
    base_url: String, // https://api.drugbank.com/v1
}

impl SourceAdapter for DrugBankAdapter {
    async fn fetch_drugs(&self) -> Result<Vec<DrugBankDrug>>;
    async fn fetch_interactions(&self, drug_id: &str) -> Result<Vec<Interaction>>;
    async fn fetch_pharmacology(&self, drug_id: &str) -> Result<Pharmacology>;
}
```

### 5.4 PDSP/NIMH Adapter (PRIORITY: HIGH for receptors)

```rust
pub struct PDSPAdapter {
    base_url: String, // https://pdsp.unc.edu/databases
}

impl SourceAdapter for PDSPAdapter {
    async fn fetch_ki_data(&self, receptor: &str) -> Result<Vec<KiValue>>;
    async fn fetch_selectivity(&self, compound: &str) -> Result<SelectivityProfile>;
}
```

## 6. Integration Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Simulato-R_v2.0                             │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                  Database Crate                          │    │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐        │    │
│  │  │ ETL Pipeline│ │ STEM Biblio │ │ Smart Cache │        │    │
│  │  └──────┬──────┘ └──────┬──────┘ └──────┬──────┘        │    │
│  │         │               │               │               │    │
│  │  ┌──────▼───────────────▼───────────────▼──────┐        │    │
│  │  │           Source Adapters                    │        │    │
│  │  │  ┌──────────┐ ┌──────────┐ ┌──────────┐     │        │    │
│  │  │  │ ChEMBL   │ │ Allen    │ │ DrugBank │     │        │    │
│  │  │  │ ❌ TODO  │ │ ❌ TODO  │ │ ❌ TODO  │     │        │    │
│  │  │  └──────────┘ └──────────┘ └──────────┘     │        │    │
│  │  └─────────────────────────────────────────────┘        │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                  Drugs-Core Crate                        │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐    │    │
│  │  │ Compound │ │ Target   │ │ Molecule │ │ Activity │    │    │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘    │    │
│  └─────────────────────────────────────────────────────────┘    │
└──────────────────────────────────┬──────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                        HumanBrain                               │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                  Pharmacology Crate                      │    │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐     │    │
│  │  │ receptor_    │ │ pharmaco-    │ │ clinical_    │     │    │
│  │  │ mechanisms   │ │ kinetics     │ │ literature   │     │    │
│  │  │ ✅ 52 tests  │ │ ✅ PK data   │ │ ✅ PET valid │     │    │
│  │  └──────────────┘ └──────────────┘ └──────────────┘     │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## 7. Phi-2-STEM Model

**Location:** `Simulato-R_v2.0/models/phi-stem-qlora/`
**Base model:** microsoft/phi-2
**Fine-tuning:** QLoRA (PEFT 0.18.0)
**Adapter size:** 21 MB
**Purpose:** STEM-focused inference for simulator operation

## 8. Action Items

### Immediate (for HumanBrain integration):
1. [ ] Implement ChEMBLAdapter in `Simulato-R_v2.0/crates/database/src/`
2. [ ] Implement AllenAtlasAdapter
3. [ ] Add `DataSource::ChEMBL`, `DataSource::AllenAtlas` to etl_pipeline.rs
4. [ ] Create unified pharmacology trait for HumanBrain

### Medium-term:
5. [ ] Implement DrugBankAdapter (requires commercial license)
6. [ ] Implement PDSPAdapter for receptor binding data
7. [ ] Add IUPHAR/BPS Guide to Pharmacology adapter

### Long-term:
8. [ ] Full ChEMBL ingestion (~2.4M compounds)
9. [ ] Full Allen Atlas ingestion (~20K genes)
10. [ ] Cross-database deduplication
11. [ ] Upload Phi-2-STEM to HuggingFace Hub

---

## 9. Key Repository Links

- **HumanBrain:** https://github.com/Yatrogenesis/HumanBrain
- **Simulato-R_v2.0:** https://github.com/Yatrogenesis/Simulato-R_v2.0
- **iatrogene-sys:** https://github.com/Yatrogenesis/iatrogene-sys
- **Drugs-Simulato-R:** https://github.com/Yatrogenesis/Drugs-Simulato-R
- **Materials-Simulato-R:** https://github.com/Yatrogenesis/Materials-Simulato-R
- **LIRS-Lab:** https://github.com/Yatrogenesis/lirs-lab
- **CORTEXIA-Framework:** https://github.com/Yatrogenesis/CORTEXIA-Framework

---

**Conclusion:** Simulato-R_v2.0 tiene la arquitectura para ingesta masiva pero carece de los adaptadores específicos de farmacología (ChEMBL, DrugBank, Allen Atlas). HumanBrain tiene datos locales validados (79 drugs, 18 genes) con 0% error pero necesita integración con la infraestructura de Simulato-R para escalar a bases de datos completas.
