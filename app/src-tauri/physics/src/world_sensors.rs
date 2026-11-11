//! The sensors on the line (spec 2026-11-11-sensors-products-design.md): beams across a conveyor that see products, and
//! barcode scanners that read each product's code once. They see products only, as OIP's beams see cargo only.
//!
//! Translated from Open Industry Project at `1e7488c` (MIT; THIRD_PARTY_NOTICES.md): the colour classification
//! (`examples/parts/color-sensor/part.js` `classify`), the laser's reading (`examples/parts/laser-sensor/`) and the
//! barcode scanner's read-once and good-read pulse (`examples/parts/barcode-scanner/behavior.ts` `step`).

use rapier3d::prelude::*;

use super::World;
use crate::layout::{Belt, SensorKind};

/// The diffuse sensor's beam, above the running surface (spec 2026-11-10); the scanner's scan line is OIP's 5 cm too.
const BEAM_HEIGHT: Real = 0.05;
/// How far out from the bed edge the new sensors' lens sits (OIP's snap: 20 mm), on each side for a through-beam.
const LENS_OUT: Real = 0.02;
/// How long the good-read stays true after a read (OIP `PULSE`).
const READ_PULSE_NS: i64 = 250_000_000;

/// What a sensor tells the controller.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Reading {
    Detected(bool),
    /// The 1-based index of the colour seen, 0 for none.
    Color(i32),
    /// To the nearest product, m; the range when clear.
    Distance(Real),
}

/// A scanner's outputs: its good-read pulse, how many it read, the last code.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Scan {
    pub read: bool,
    pub count: u32,
    pub code: i32,
}

/// A scanner's state between steps.
#[derive(Default)]
pub(super) struct ScannerState {
    pulse_until_ns: i64,
    count: u32,
    code: i32,
}

/// The index of the listed colour nearest `color` by its worst channel, 1-based, among those within `tolerance` on
/// every channel; the earlier on a tie; 0 for none.
pub fn classify(color: [u8; 3], colors: &[[u8; 3]], tolerance: u8) -> i32 {
    let mut best = (0, u8::MAX);
    for (k, listed) in colors.iter().enumerate() {
        let d = (0..3)
            .map(|c| color[c].abs_diff(listed[c]))
            .max()
            .expect("three channels");
        if d <= tolerance && (best.0 == 0 || d < best.1) {
            best = (k as i32 + 1, d);
        }
    }
    best.0
}

/// A frame at a mount on a conveyor, x along it and z across it: a curve's at its angle.
fn mount_frame(b: &Belt, at: Real) -> (Pose, Real) {
    match b.arc {
        None => (b.pose, at),
        Some(arc) => (b.pose * arc.frame(b.width, at), 0.0),
    }
}

impl World {
    /// A beam across a conveyor at a mount, from `start` (across, from the centreline) towards +z, as far as `range`:
    /// how far it hits a moving body first, and which product that is, if it is one.
    fn beam(
        &self,
        belt: usize,
        at: Real,
        start: Real,
        range: Real,
    ) -> Option<(Option<usize>, Real)> {
        let b = &self.layout.belts[belt];
        let (frame, x) = mount_frame(b, at);
        let origin = frame.transform_point(Vector::new(x, b.height + BEAM_HEIGHT, start));
        let ray = Ray::new(origin, frame.rotation * Vector::Z);
        let (collider, distance) =
            (self.world).cast_ray(&ray, range, true, QueryFilter::only_dynamic())?;
        let body = self.world.colliders[collider].parent();
        let product = self.products.iter().position(|p| Some(p.body) == body);
        Some((product, distance))
    }

    /// What a sensor reads after the step.
    pub fn reading(&self, sensor: usize) -> Reading {
        let s = &self.layout.sensors[sensor];
        let half = self.layout.belts[s.belt].width / 2.0;
        let lens = -half - LENS_OUT;
        match &s.kind {
            SensorKind::Diffuse { range } => {
                Reading::Detected(self.beam(s.belt, s.at, -half, *range).is_some())
            }
            SensorKind::ThroughBeam => {
                let gap = 2.0 * (half + LENS_OUT);
                Reading::Detected(self.beam(s.belt, s.at, lens, gap).is_some())
            }
            SensorKind::Color {
                range,
                colors,
                tolerance,
            } => {
                let seen = self.beam(s.belt, s.at, lens, *range);
                let color = seen.and_then(|(product, _)| self.products[product?].color);
                Reading::Color(color.map_or(0, |c| classify(c, colors, *tolerance)))
            }
            SensorKind::Laser { max_range } => {
                let seen = self.beam(s.belt, s.at, lens, *max_range);
                Reading::Distance(seen.map_or(*max_range, |(_, d)| d))
            }
        }
    }

    /// Each scanner's scan line across its conveyor: an unread product it crosses while its centre is in the read
    /// window is read and marked, so it is never read again.
    pub(super) fn scan(&mut self) {
        for k in 0..self.layout.scanners.len() {
            let s = &self.layout.scanners[k];
            let b = &self.layout.belts[s.belt];
            let (window, half) = (s.depth / 2.0, b.width / 2.0);
            let read = (self.beam(s.belt, s.at, -half, b.width))
                .and_then(|(p, _)| p)
                .filter(|&p| {
                    let centre = self.local(s.belt, &self.products[p]);
                    !self.products[p].read && (centre.x - s.at).abs() <= window
                });
            let state = &mut self.scanners[k];
            if let Some(p) = read {
                let product = &mut self.products[p];
                product.read = true;
                state.count += 1;
                state.code = product.code.unwrap_or(0);
                state.pulse_until_ns = self.t_ns + READ_PULSE_NS;
            }
        }
    }

    /// A scanner's outputs after the step.
    pub fn scanned(&self, scanner: usize) -> Scan {
        let s = &self.scanners[scanner];
        Scan {
            read: self.t_ns < s.pulse_until_ns,
            count: s.count,
            code: s.code,
        }
    }
}

#[cfg(test)]
#[path = "world_sensors_tests.rs"]
mod tests;
