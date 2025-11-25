//! Circadian rhythm and sleep-wake cycles.

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircadianClock {
    pub time_of_day: f64,
    pub circadian_phase: f64,
    pub melatonin_level: f64,
    pub cortisol_level: f64,
    pub sleep_pressure: f64,
    pub circadian_drive: f64,
}

impl CircadianClock {
    pub fn new() -> Self {
        Self {
            time_of_day: 12.0,
            circadian_phase: 0.0,
            melatonin_level: 0.0,
            cortisol_level: 0.5,
            sleep_pressure: 0.0,
            circadian_drive: 1.0,
        }
    }

    pub fn step(&mut self, dt_hours: f64, light_exposure: f64, is_awake: bool) {
        self.time_of_day += dt_hours;
        if self.time_of_day >= 24.0 {
            self.time_of_day -= 24.0;
        }

        let tau = 24.1;
        let light_adjustment = 0.1 * light_exposure * ((self.time_of_day - 16.0) / 4.0).cos();
        self.circadian_phase += (2.0 * PI / tau + light_adjustment) * dt_hours;
        if self.circadian_phase > 2.0 * PI {
            self.circadian_phase -= 2.0 * PI;
        }

        self.melatonin_level = (0.5 + 0.5 * (self.circadian_phase + PI).cos()).max(0.0);
        self.melatonin_level *= 1.0 - 0.8 * light_exposure;

        self.cortisol_level = (0.5 + 0.5 * self.circadian_phase.cos()).max(0.0);

        if is_awake {
            self.sleep_pressure += 0.04 * dt_hours;
        } else {
            self.sleep_pressure -= 0.15 * dt_hours;
        }
        self.sleep_pressure = self.sleep_pressure.clamp(0.0, 1.0);

        self.circadian_drive = 0.5 + 0.5 * self.circadian_phase.cos();
    }

    pub fn sleep_propensity(&self) -> f64 {
        (self.sleep_pressure + 1.0 - self.circadian_drive) / 2.0
    }

    pub fn cortical_excitability(&self) -> f64 {
        1.0 - self.sleep_propensity()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepStage { Awake, N1, N2, N3, REM }

pub struct SleepStageController {
    pub current_stage: SleepStage,
    pub stage_duration: f64,
}

impl SleepStageController {
    pub fn new() -> Self {
        Self { current_stage: SleepStage::Awake, stage_duration: 0.0 }
    }

    pub fn step(&mut self, dt_hours: f64, sleep_propensity: f64) {
        self.stage_duration += dt_hours;
        if sleep_propensity > 0.7 && self.current_stage == SleepStage::Awake {
            self.current_stage = SleepStage::N1;
            self.stage_duration = 0.0;
        } else if self.current_stage == SleepStage::N1 && self.stage_duration > 0.1 {
            self.current_stage = SleepStage::N2;
            self.stage_duration = 0.0;
        } else if self.current_stage == SleepStage::N2 && self.stage_duration > 0.3 {
            self.current_stage = SleepStage::N3;
            self.stage_duration = 0.0;
        } else if self.current_stage == SleepStage::N3 && self.stage_duration > 0.5 {
            self.current_stage = SleepStage::REM;
            self.stage_duration = 0.0;
        } else if self.current_stage == SleepStage::REM && self.stage_duration > 0.2 {
            self.current_stage = if sleep_propensity < 0.3 { SleepStage::Awake } else { SleepStage::N2 };
            self.stage_duration = 0.0;
        }
    }

    pub fn delta_power(&self) -> f64 {
        match self.current_stage {
            SleepStage::N3 => 1.0,
            SleepStage::N2 => 0.5,
            _ => 0.0,
        }
    }
}
