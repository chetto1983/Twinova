//! A conveyor's drive feedback, as its native model publishes it every millisecond (spec
//! 2026-11-10-run-pace-native-feedback-design.md, decision 2): the velocity on a VFD's scale and the encoder pulse.
//! The encoder itself is the world's travel of the surface.
//!
//! Translated from Open Industry Project at `1e7488c` (MIT; THIRD_PARTY_NOTICES.md),
//! `examples/parts/_shared/conveyorDrive.js` `publishDrive`. In 4.4b these were lines of each conveyor's ST block, a
//! third of the field's cycle (measured); the block keeps the ramp, which is logic.

use twinova_project::Project;
use twinova_project::model::ParamValue;

/// The field's millisecond, in seconds.
const STEP_S: f64 = 0.001;

#[derive(Debug, Clone, PartialEq)]
pub struct Feedback {
    /// The surface speed that reads 30, m/s (OIP `speedAt30`).
    speed_at_30: f32,
    /// Surface travel between pulses, m (OIP `inchesPerPulse`).
    pulse_pitch: f64,
    /// Surface travel since the last pulse, m.
    since_pulse: f64,
}

impl Feedback {
    /// The pitch stays the parameter's f64: through an f32, a pitch of exactly one millisecond's travel would miss
    /// every other pulse.
    pub fn new(speed_at_30: f32, pulse_pitch: f64) -> Self {
        Self {
            speed_at_30,
            pulse_pitch,
            since_pulse: 0.0,
        }
    }

    /// The feedback of `instance`, from its `speed_at_30` and `pulse_pitch`.
    pub fn of(project: &Project, instance: &str) -> Self {
        let inst = project.scene.instances.iter().find(|i| i.id == instance);
        let real = |name: &str| {
            let inst = inst?;
            let def = project.definitions.get(&inst.definition)?;
            let default = def
                .parameters
                .iter()
                .find(|p| p.name == name)
                .map(|p| p.default);
            match inst.parameters.get(name).copied().or(default) {
                Some(ParamValue::Real(x)) => Some(x),
                _ => None,
            }
        };
        Self::new(
            real("speed_at_30").unwrap_or(0.0) as f32,
            real("pulse_pitch").unwrap_or(0.0),
        )
    }

    /// The effective speed as a share of the speed at 30, times 30: stopped reads 0.
    pub fn velocity(&self, speed: f32) -> f32 {
        if self.speed_at_30 > 0.0 {
            speed / self.speed_at_30 * 30.0
        } else {
            0.0
        }
    }

    /// One millisecond at `speed`: whether the surface completed a pitch of travel in it, in either direction. A hair
    /// of tolerance, because the travel's steps do not sum to the pitch exactly.
    pub fn pulse(&mut self, speed: f32) -> bool {
        if self.pulse_pitch <= 0.0 {
            return false;
        }
        self.since_pulse += f64::from(speed.abs()) * STEP_S;
        let pulse = self.since_pulse >= self.pulse_pitch * (1.0 - 1e-9);
        if pulse {
            let pitches = (self.since_pulse / self.pulse_pitch + 1e-9)
                .floor()
                .max(1.0);
            self.since_pulse = (self.since_pulse - self.pulse_pitch * pitches).max(0.0);
        }
        pulse
    }
}

#[cfg(test)]
#[path = "drive_tests.rs"]
mod tests;
