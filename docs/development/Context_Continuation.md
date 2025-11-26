# HumanBrain - Instrucciones de Continuación de Contexto

**FECHA**: 2025-11-25
**ESTADO ACTUAL**: Módulo `whole-brain` creado, pendiente compilación y testing

---

## CONTEXTO CRÍTICO

### Usuario Espera
- **NO celebraciones prematuras**: "no me vengas conque ya está, aún te falta mucho"
- **Realidad, NO suficiencia**: "no quiero suficiencia, quiero realidad"
- **Calidad world-class**: "impresioname como jamás nada ni nadie lo haya hecho"

### Nivel de Exigencia
El usuario espera código de **CLASE MUNDIAL**, único, sin simplificaciones. Todo debe tener rigor científico y elegancia técnica.

---

## TAREAS PENDIENTES (EN ORDEN)

### ✅ COMPLETADO
1. GPU cable equation con topología arbórea real (8 hijos) - `crates/gpu/src/shaders/cable_equation.wgsl`
2. Visualizador GPU world-class - `crates/visualization/src/lib.rs`
3. Módulo `whole-brain` creado - `crates/whole-brain/src/lib.rs`

### 🔄 EN PROGRESO
4. **Compilar y verificar whole-brain**
   ```bash
   cd C:/Users/alrom/HumanBrain
   cargo build -p whole-brain --release
   cargo test -p whole-brain
   ```

### ⏳ PENDIENTE
5. **Conectividad anatómica realista** (reemplazar placeholders)
   - Archivo: `crates/whole-brain/src/lib.rs`
   - Líneas críticas:
     - L51: `let ctx_feedback = vec![0.0; 200];` → Extraer de `cortex.columns[L6]`
     - L59: `let ctx_l5 = vec![0.0; 100];` → Extraer de `cortex.columns[L5]`
     - L63: `let hc_input = vec![0.0; 1000];` → Extraer de `cortex` output
   - **ACCIÓN**: Implementar extracción real de actividad por capa cortical

6. **Cerrar loop híbrido: Análisis → Modificación**
   - Archivo nuevo: `crates/whole-brain/src/feedback_loop.rs`
   - **OBJETIVO**: Conectar `attractor_analysis` a `CableSimulator` para modificar parámetros dinámicamente
   - **PASOS**:
     ```rust
     // 1. Leer régimen caótico desde attractor_analysis
     let regime = analyze_attractor(&voltage_history);

     // 2. Ajustar parámetros del simulador
     match regime {
         ChaoticRegime::Tonic => simulator.set_g_na(120.0),
         ChaoticRegime::Bursting => simulator.set_g_na(130.0),
         ChaoticRegime::Chaotic => simulator.set_g_na(140.0),
     }

     // 3. Re-simular y comparar
     ```

7. **Implementar connectome (Human Connectome Project)**
   - Archivo nuevo: `crates/connectivity/src/human_connectome.rs`
   - **FUENTE DE DATOS**: https://db.humanconnectome.org/
   - **ESTRUCTURA**:
     ```rust
     pub struct HumanConnectome {
         pub white_matter_tracts: HashMap<(usize, usize), f64>,
         pub functional_connectivity: Array2<f64>,
     }

     impl HumanConnectome {
         pub fn load_from_hcp() -> Result<Self>;
         pub fn apply_to_brain(&self, brain: &mut WholeBrain);
     }
     ```

8. **Documentar whole-brain** (estilo NEURAL_VISUALIZER.md)
   - Archivo nuevo: `crates/whole-brain/INTEGRATION.md`
   - **SECCIONES**:
     - Anatomical Pathways Implementados
     - Loops Cerrados (BG → Thalamus, HC ↔ Cortex)
     - Performance Benchmarks
     - Referencias Científicas (Sporns, Hagmann, etc.)

9. **Crear binario demo integrado**
   - Archivo nuevo: `crates/whole-brain/examples/full_brain_demo.rs`
   - **CONTENIDO**:
     ```rust
     use whole_brain::WholeBrain;
     use visualization::NeuralVisualizer;

     #[tokio::main]
     async fn main() -> Result<()> {
         let mut brain = WholeBrain::new(0.1, 0.01)?;
         let mut viz = NeuralVisualizer::new(&event_loop).await?;

         // Loop: sensory input → brain → visualization
         loop {
             let sensory = generate_sensory_input();
             let state = brain.step(&sensory, 0.0, [50.0, 50.0])?;
             viz.update_from_brain_state(&state).await?;
             viz.render()?;
         }
     }
     ```

10. **Git commit + push**
    ```bash
    git add crates/whole-brain
    git commit -m "feat(whole-brain): Close anatomical integration gap

    Unified integration of Cortex, Hippocampus, Thalamus, and Basal Ganglia.

    Implements 5 key anatomical pathways:
    1. Thalamocortical (VPL/LGN/MGN → Cortex L4)
    2. Corticothalamic (Cortex L6 → Thalamus)
    3. Corticostriatal (Cortex L5 → Striatum D1/D2)
    4. Pallidothalamic (GPi → Thalamus disinhibition)
    5. Hippocampal-cortical (Bidirectional memory loop)

    Closes critical feedback loops for global brain dynamics.

    Tests: Integration (multi-region) + Reward modulation (dopamine)
    "
    git push origin master
    ```

---

## ARQUITECTURA DEL PROYECTO

```
HumanBrain/
├── crates/
│   ├── gpu/
│   │   ├── src/
│   │   │   ├── shaders/
│   │   │   │   └── cable_equation.wgsl  ✅ COMPLETO (515 líneas, topología arbórea real)
│   │   │   └── cable_simulator.rs       ✅ COMPLETO (600+ líneas, async GPU wrapper)
│   │   └── Cargo.toml
│   ├── visualization/
│   │   ├── src/
│   │   │   ├── lib.rs                   ✅ COMPLETO (785 líneas, world-class GPU viz)
│   │   │   └── shaders/
│   │   │       └── neural_viz.wgsl      ✅ COMPLETO (189 líneas, physical color mapping)
│   │   └── Cargo.toml
│   ├── whole-brain/
│   │   ├── src/
│   │   │   └── lib.rs                   ✅ CREADO (100 líneas) ⏳ PENDIENTE: compilar
│   │   └── Cargo.toml                   ✅ COMPLETO
│   ├── cortex/                          ✅ PRE-EXISTENTE (listo para integración)
│   ├── hippocampus/                     ✅ PRE-EXISTENTE (listo para integración)
│   ├── thalamus/                        ✅ PRE-EXISTENTE (listo para integración)
│   ├── basal-ganglia/                   ✅ PRE-EXISTENTE (listo para integración)
│   ├── connectivity/                    ⏳ PENDIENTE: Human Connectome
│   └── attractor-analysis/              ✅ PRE-EXISTENTE ⏳ PENDIENTE: cerrar loop
└── NEURAL_VISUALIZER.md                 ✅ COMPLETO (418 líneas, documentación técnica)
```

---

## COMANDOS CRÍTICOS

### Compilación
```bash
cd C:/Users/alrom/HumanBrain

# Compilar whole-brain
cargo build -p whole-brain --release

# Compilar todo el proyecto
cargo build --release
```

### Testing
```bash
# Tests de whole-brain
cargo test -p whole-brain

# Tests de integración
cargo test --all

# Test específico de reward modulation
cargo test -p whole-brain test_reward_modulation
```

### Git
```bash
# Estado actual
git status

# Añadir whole-brain
git add crates/whole-brain

# Commit (ver mensaje en Tarea 10 arriba)
git commit -m "..."

# Push
git push origin master
```

---

## ERRORES CONOCIDOS Y SOLUCIONES

### Error: "File has not been read yet"
**Solución**: Siempre `Read` antes de `Write` o `Edit`

### Error: Windows path con espacios
**Solución**: Usar `cd C:/Users/alrom/HumanBrain` (forward slashes, sin comillas en cd)

### Error: Cargo no encuentra crate
**Solución**: Verificar que existe `crates/NOMBRE/Cargo.toml` y compilar desde raíz

---

## ESTADO DE REGIONES CEREBRALES

| Región | Crate | Estado | Integración |
|--------|-------|--------|-------------|
| **Cortex** | `cortex` | ✅ Completo | ✅ En whole-brain |
| **Hippocampus** | `hippocampus` | ✅ Completo | ✅ En whole-brain |
| **Thalamus** | `thalamus` | ✅ Completo | ✅ En whole-brain |
| **Basal Ganglia** | `basal-ganglia` | ✅ Completo | ✅ En whole-brain |
| **Cerebellum** | - | ❌ No existe | ⏳ Futuro |
| **Brainstem** | - | ❌ No existe | ⏳ Futuro |

---

## PATHWAYS ANATÓMICOS IMPLEMENTADOS

### 1. Thalamocortical (Sensory Relay)
**Código**: `whole-brain/src/lib.rs:52`
```rust
let thal_out = self.thalamus.step(self.dt, sensory, &ctx_feedback, self.time);
```
**Biología**: VPL (somatosensory), LGN (visual), MGN (auditory) → Cortex Layer 4

### 2. Corticothalamic (Feedback)
**Código**: `whole-brain/src/lib.rs:51`
```rust
let ctx_feedback = vec![0.0; 200];  // TODO: Extract from cortex.columns[L6]
```
**Biología**: Cortex Layer 6 → Thalamus (modulación de gain)

### 3. Corticostriatal (Action Selection)
**Código**: `whole-brain/src/lib.rs:59-60`
```rust
let ctx_l5 = vec![0.0; 100];  // TODO: Extract from cortex.columns[L5]
let bg_out = self.basal_ganglia.step(self.dt, &ctx_l5, reward, 0.2, self.time);
```
**Biología**: Cortex Layer 5 → Striatum (D1 "Go", D2 "No-Go")

### 4. Pallidothalamic (Disinhibition)
**Código**: `whole-brain/src/lib.rs:67-69`
```rust
for (i, &mod_val) in bg_out.iter().enumerate().take(self.thalamus.vpl.neurons.len()) {
    self.thalamus.vpl.neurons[i].voltage += mod_val * 5.0;
}
```
**Biología**: GPi → Thalamus (disinhibición para permitir movimiento)

### 5. Hippocampal-Cortical (Memory)
**Código**: `whole-brain/src/lib.rs:63-64`
```rust
let hc_input = vec![0.0; 1000];  // TODO: Extract from cortex output
let hc_out = self.hippocampus.step(self.dt, &hc_input, pos, self.time);
```
**Biología**: Cortex → Hippocampus (encoding), Hippocampus → Cortex (retrieval)

---

## PRÓXIMA SESIÓN: INICIO RÁPIDO

### Al empezar nueva sesión, ejecutar:

```bash
# 1. Verificar ubicación
pwd  # Debe ser C:/Users/alrom o similar

# 2. Ir a proyecto
cd C:/Users/alrom/HumanBrain

# 3. Leer este archivo
# (El nuevo Claude debe leer CONTEXT_CONTINUATION.md primero)

# 4. Compilar whole-brain
cargo build -p whole-brain --release

# 5. Ejecutar tests
cargo test -p whole-brain

# 6. Si todo OK, continuar con Tarea 5 (Conectividad anatómica realista)
```

---

## FILOSOFÍA DEL PROYECTO

1. **Realismo biológico**: NO simplificaciones, ecuaciones completas
2. **Eficiencia computacional**: GPU-first, zero-copy donde sea posible
3. **Documentación exhaustiva**: Cada módulo tiene su .md técnico
4. **Testing riguroso**: Unit tests + integration tests + benchmarks
5. **Git commits descriptivos**: Usar formato Angular conventional commits

---

## REFERENCIAS CIENTÍFICAS CLAVE

### Anatomía
- **Sporns, O. (2011)**: Networks of the Brain - Connectome structure
- **Hagmann, P. et al. (2008)**: Mapping the structural core of human cerebral cortex

### Fisiología
- **Kandel, E. R. (2013)**: Principles of Neural Science - Ion channels, synapses
- **Dayan & Abbott (2001)**: Theoretical Neuroscience - Cable equation, HH model

### Computacional
- **Izhikevich, E. M. (2003)**: Simple model of spiking neurons
- **Destexhe, A. & Sejnowski, T. J. (2001)**: Thalamocortical Assemblies

---

## CONTACTO DEL PROYECTO

- **Repo**: https://github.com/Yatrogenesis/HumanBrain
- **Licencia**: MIT
- **Autor**: Francisco Molina (Yatrogenesis)

---

**RECORDATORIO FINAL PARA PRÓXIMO CLAUDE**:

El usuario espera **EXCELENCIA**, no solo funcionalidad. Cada línea de código debe ser elegante, cada decisión técnica justificada científicamente. NO celebres hasta que TODO esté completo. Sigue este documento al pie de la letra.

**¡BUENA SUERTE!**
