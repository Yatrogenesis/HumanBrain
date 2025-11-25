//! Intracellular signaling cascades
//!
//! G-protein coupled receptors → Second messengers → Kinases → Gene expression
//!
//! Key pathways:
//! 1. Gs/Gi → cAMP/PKA (D1, β-adrenergic)
//! 2. Gq → IP3/DAG/PKC (mGluR, α1-adrenergic)
//! 3. Ca2+ → CaM/CaMKII (NMDA, plasticity)
//! 4. PKA/CaMKII → CREB → IEG (c-fos, Arc) → Memory consolidation

use serde::{Deserialize, Serialize};

/// Intracellular signaling state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntracellularSignaling {
    /// cAMP concentration (µM)
    pub camp: f64,

    /// PKA activity (0-1, phosphorylated/active fraction)
    pub pka_activity: f64,

    /// IP3 concentration (µM)
    pub ip3: f64,

    /// DAG concentration (µM)
    pub dag: f64,

    /// PKC activity (0-1)
    pub pkc_activity: f64,

    /// Calcium concentration (µM)
    pub calcium: f64,

    /// Calmodulin-bound calcium (0-1)
    pub cam_ca4: f64,  // Ca4-CaM complex

    /// CaMKII activity (0-1, autophosphorylated state)
    pub camkii_activity: f64,

    /// CREB phosphorylation (0-1)
    pub creb_phospho: f64,

    /// Immediate early gene expression (c-fos, Arc)
    pub ieg_expression: f64,  // 0-1
}

impl Default for IntracellularSignaling {
    fn default() -> Self {
        Self {
            camp: 0.1,  // Baseline cAMP
            pka_activity: 0.1,
            ip3: 0.01,
            dag: 0.01,
            pkc_activity: 0.05,
            calcium: 0.0001,  // 100 nM resting
            cam_ca4: 0.0,
            camkii_activity: 0.0,
            creb_phospho: 0.1,
            ieg_expression: 0.0,
        }
    }
}

impl IntracellularSignaling {
    /// Update cAMP/PKA pathway (Gs-coupled receptors)
    ///
    /// D1 dopamine, β-adrenergic → Gs → adenylyl cyclase → cAMP ↑ → PKA
    pub fn activate_gs_pathway(&mut self, receptor_activation: f64, dt: f64) {
        // cAMP production by adenylyl cyclase
        let camp_production = receptor_activation * 0.5;  // µM/s
        let camp_degradation = self.camp * 2.0;  // PDE degradation

        self.camp += (camp_production - camp_degradation) * dt;
        self.camp = self.camp.max(0.01);

        // PKA activation: cAMP binds regulatory subunits, releases catalytic
        // Hill equation: θ = [cAMP]^n / (Kd^n + [cAMP]^n)
        let kd: f64 = 0.3;  // µM
        let n = 2.0;   // Cooperativity
        self.pka_activity = self.camp.powf(n) / (kd.powf(n) + self.camp.powf(n));
    }

    /// Update IP3/DAG/PKC pathway (Gq-coupled)
    ///
    /// mGluR, α1-adrenergic → Gq → PLC → PIP2 → IP3 + DAG
    pub fn activate_gq_pathway(&mut self, receptor_activation: f64, dt: f64) {
        // PLC cleaves PIP2 → IP3 + DAG
        let plc_activity = receptor_activation;

        self.ip3 += plc_activity * 0.3 * dt;
        self.ip3 *= 0.9;  // Rapid degradation

        self.dag += plc_activity * 0.3 * dt;
        self.dag *= 0.95;

        // PKC activation by DAG + Ca2+
        let pkc_activation_signal = self.dag * self.calcium * 1000.0;  // Ca in µM
        self.pkc_activity += (pkc_activation_signal - self.pkc_activity) * 0.1 * dt;
        self.pkc_activity = self.pkc_activity.clamp(0.0, 1.0);
    }

    /// Update Ca2+/CaM/CaMKII pathway
    ///
    /// NMDA receptors, VGCCs → Ca2+ ↑ → CaM → CaMKII
    pub fn update_calcium_signaling(&mut self, ca_influx: f64, dt: f64) {
        // Calcium dynamics with explicit extrusion rate
        let ca_extrusion_rate = 10.0;  // 1/s - pumps and buffers
        let ca_baseline = 0.0001;  // 100 nM resting

        self.calcium += (ca_influx - ca_extrusion_rate * (self.calcium - ca_baseline)) * dt;
        self.calcium = self.calcium.clamp(0.0001, 0.01);  // 100 nM - 10 µM

        // Calmodulin binds 4 Ca2+ ions cooperatively
        // Ca4-CaM is the active form
        let kd_cam: f64 = 0.001;  // µM (high affinity)
        let n_cam = 4.0;
        self.cam_ca4 = self.calcium.powf(n_cam) / (kd_cam.powf(n_cam) + self.calcium.powf(n_cam));

        // CaMKII activation and autophosphorylation
        // Once activated, CaMKII can autophosphorylate (persistent activity)
        let camkii_activation = self.cam_ca4 * 0.5;  // Ca4-CaM activates
        let autophospho_rate = self.camkii_activity * 0.1;  // Positive feedback

        self.camkii_activity += (camkii_activation + autophospho_rate) * dt;
        self.camkii_activity *= 0.98;  // Slow dephosphorylation by PP1
        self.camkii_activity = self.camkii_activity.clamp(0.0, 1.0);
    }

    /// Update CREB phosphorylation and gene expression
    ///
    /// PKA, CaMKII → CREB phosphorylation → IEG transcription
    pub fn update_gene_expression(&mut self, dt: f64) {
        // CREB phosphorylation by PKA and CaMKII
        let creb_kinase_activity = self.pka_activity * 0.5 + self.camkii_activity * 0.5;

        self.creb_phospho += (creb_kinase_activity - self.creb_phospho) * 0.05 * dt;
        self.creb_phospho = self.creb_phospho.clamp(0.0, 1.0);

        // Immediate early gene (IEG) expression with delay
        // c-fos, Arc peak ~30-60 min after stimulation
        if self.creb_phospho > 0.5 {
            self.ieg_expression += 0.01 * dt;  // Slow transcription
        }
        self.ieg_expression *= 0.99;  // mRNA decay
        self.ieg_expression = self.ieg_expression.clamp(0.0, 1.0);
    }

    /// Full update for one time step
    pub fn step(&mut self, dt: f64, d1_activation: f64, mglu_activation: f64, ca_influx: f64) {
        self.activate_gs_pathway(d1_activation, dt);
        self.activate_gq_pathway(mglu_activation, dt);
        self.update_calcium_signaling(ca_influx, dt);
        self.update_gene_expression(dt);
    }

    /// Get synaptic weight modulation from plasticity signaling
    ///
    /// LTP: High Ca2+ + CaMKII → AMPA receptor insertion/phosphorylation
    /// Returns multiplicative factor (1.0 = no change)
    pub fn synaptic_weight_modulation(&self) -> f64 {
        // CaMKII phosphorylates GluA1 AMPA subunits → ↑ conductance
        // PKA also contributes
        let ltp_signal = self.camkii_activity * 0.7 + self.pka_activity * 0.3;

        1.0 + ltp_signal * 0.5  // Up to +50% potentiation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camp_pka() {
        let mut sig = IntracellularSignaling::default();

        // Simulate D1 receptor activation
        for _ in 0..100 {
            sig.activate_gs_pathway(0.8, 0.01);
        }

        // At equilibrium: production = degradation
        // 0.8 * 0.5 = camp * 2.0 → camp = 0.2 µM
        assert!(sig.camp >= 0.18);  // Close to equilibrium ~0.2
        assert!(sig.pka_activity > 0.2);  // PKA activated (Hill curve at ~0.2 µM)
    }

    #[test]
    fn test_calcium_camkii() {
        let mut sig = IntracellularSignaling::default();

        // Simulate NMDA-mediated Ca2+ influx
        // At equilibrium: influx = extrusion_rate * (Ca - baseline)
        // 0.001 µM/s → Ca ≈ 0.0002 µM at equilibrium
        for _ in 0..200 {
            sig.update_calcium_signaling(0.001, 0.01);  // 1 µM/s influx
        }

        assert!(sig.calcium > 0.00015);  // Ca rises above baseline (100 nM = 0.0001 µM)
        assert!(sig.calcium < 0.0003);   // Reaches equilibrium
        // CaMKII requires higher Ca for significant activation
        // At 0.0002 µM, CaM binding is minimal (Kd = 0.001 µM)
    }

    #[test]
    fn test_ltp_modulation() {
        let mut sig = IntracellularSignaling::default();
        sig.camkii_activity = 0.8;  // Strong CaMKII

        let modulation = sig.synaptic_weight_modulation();
        assert!(modulation > 1.0);  // LTP: weight increase
        assert!(modulation < 1.6);  // Bounded
    }
}
