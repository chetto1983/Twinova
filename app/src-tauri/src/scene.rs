//! M1's belt stepped by a fixed time step (spec 2026-11-07-tauri-shell-design.md, "The scene model"). The app now
//! runs the closed-form belt (`belt.rs`); this one is the fixed-step reference it is benchmarked against
//! (`examples/belt_benchmark.rs`, spec 2026-11-08-analytic-belt-design.md). Positions are the front of each part
//! along the belt, in metres; parts are ordered front first.

use serde::Serialize;

use crate::belt::{
    ARRIVAL_INTERVAL_NS, LENGTH_NM, NM_PER_M, NS_PER_S, PART_NM, PHOTOCELL_NM, SPEED_NM_PER_S,
};

pub const BELT_LENGTH_M: f64 = LENGTH_NM as f64 / NM_PER_M as f64;
pub const BELT_SPEED_M_S: f64 = SPEED_NM_PER_S as f64 / NM_PER_M as f64;
pub const PART_LENGTH_M: f64 = PART_NM as f64 / NM_PER_M as f64;
pub const SPAWN_INTERVAL_S: f64 = ARRIVAL_INTERVAL_NS as f64 / NS_PER_S as f64;
pub const PHOTOCELL_AT_M: f64 = PHOTOCELL_NM as f64 / NM_PER_M as f64;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Part {
    pub id: u64,
    /// Position of the part's front edge.
    pub front_m: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Scene {
    pub parts: Vec<Part>,
    pub delivered: u64,
    pub outlet_blocked: bool,
    next_id: u64,
    since_spawn_s: f64,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            parts: Vec::new(),
            delivered: 0,
            outlet_blocked: false,
            next_id: 1,
            // The first part arrives on the first step.
            since_spawn_s: SPAWN_INTERVAL_S,
        }
    }
}

impl Scene {
    /// TRUE while a part covers the photocell's point.
    pub fn photocell(&self) -> bool {
        self.parts
            .iter()
            .any(|p| p.front_m >= PHOTOCELL_AT_M && p.front_m - PART_LENGTH_M <= PHOTOCELL_AT_M)
    }

    /// Advances the scene by `dt_s` seconds with the motor on or off.
    pub fn step(&mut self, dt_s: f64, motor: bool) {
        if motor {
            self.move_parts(BELT_SPEED_M_S * dt_s);
        }
        self.since_spawn_s += dt_s;
        self.spawn();
    }

    fn move_parts(&mut self, distance_m: f64) {
        let end_limit = if self.outlet_blocked {
            BELT_LENGTH_M
        } else {
            f64::INFINITY
        };
        let mut limit = end_limit;
        for part in &mut self.parts {
            part.front_m = (part.front_m + distance_m).min(limit);
            limit = part.front_m - PART_LENGTH_M;
        }
        let before = self.parts.len();
        self.parts
            .retain(|p| p.front_m < BELT_LENGTH_M || self.outlet_blocked);
        self.delivered += (before - self.parts.len()) as u64;
    }

    fn spawn(&mut self) {
        // A new part enters with its rear edge at 0, so it needs the previous part's tail past PART_LENGTH_M.
        let entry_free = self
            .parts
            .last()
            .is_none_or(|p| p.front_m - PART_LENGTH_M >= PART_LENGTH_M);
        if self.since_spawn_s >= SPAWN_INTERVAL_S && entry_free {
            self.parts.push(Part {
                id: self.next_id,
                front_m: PART_LENGTH_M,
            });
            self.next_id += 1;
            self.since_spawn_s = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f64 = 0.01;

    /// At most an hour: a longer request comes from a broken constant, and fails here at once.
    fn run(scene: &mut Scene, seconds: f64, motor: bool) {
        assert!(seconds <= 3_600.0, "{seconds} s requested");
        for _ in 0..(seconds / DT).round() as u64 {
            scene.step(DT, motor);
        }
    }

    #[test]
    fn the_metres_are_the_values_m1_used() {
        assert_eq!(
            (
                BELT_LENGTH_M,
                BELT_SPEED_M_S,
                PART_LENGTH_M,
                SPAWN_INTERVAL_S,
                PHOTOCELL_AT_M
            ),
            (3.0, 0.5, 0.30, 2.0, 2.70)
        );
    }

    #[test]
    fn a_part_arrives_on_the_first_step_and_then_every_interval() {
        let mut s = Scene::default();
        s.step(DT, true);
        assert_eq!(s.parts.len(), 1);
        run(&mut s, SPAWN_INTERVAL_S - 2.0 * DT, true);
        assert_eq!(s.parts.len(), 1, "no second part before the interval");
        run(&mut s, 2.0 * DT, true);
        assert_eq!(s.parts.len(), 2);
    }

    #[test]
    fn a_stopped_belt_does_not_move_and_keeps_its_entry_occupied() {
        let mut s = Scene::default();
        run(&mut s, 10.0, false);
        assert_eq!(
            s.parts,
            vec![Part {
                id: 1,
                front_m: PART_LENGTH_M
            }],
            "the entry stays blocked by the first part"
        );
    }

    #[test]
    fn parts_travel_the_belt_and_are_delivered() {
        let mut s = Scene::default();
        run(&mut s, BELT_LENGTH_M / BELT_SPEED_M_S + 0.05, true);
        assert_eq!(s.delivered, 1);
        assert!(s.parts.iter().all(|p| p.front_m < BELT_LENGTH_M));
    }

    #[test]
    fn a_part_whose_front_reaches_the_end_exactly_is_delivered() {
        // 2.75 + 0.5 * 0.5 = 3.0 exactly in binary floating point.
        let mut s = Scene {
            parts: vec![Part {
                id: 1,
                front_m: 2.75,
            }],
            ..Scene::default()
        };
        s.step(0.5, true);
        assert_eq!(s.delivered, 1);
        let fronts: Vec<f64> = s.parts.iter().map(|p| p.front_m).collect();
        assert_eq!(
            fronts,
            [PART_LENGTH_M],
            "only the part that just entered is left"
        );
    }

    #[test]
    fn every_part_gets_its_own_increasing_id() {
        let mut s = Scene::default();
        // Parts enter at 0, 2 and 4 s; at 5 s the first is at 2.8 m, still on the belt.
        run(&mut s, 2.5 * SPAWN_INTERVAL_S, true);
        let ids: Vec<u64> = s.parts.iter().map(|p| p.id).collect();
        assert_eq!(ids, [1, 2, 3]);
    }

    #[test]
    fn a_blocked_outlet_accumulates_parts_nose_to_tail() {
        let mut s = Scene {
            outlet_blocked: true,
            ..Scene::default()
        };
        run(&mut s, 60.0, true);
        assert_eq!(s.delivered, 0);
        assert_eq!(s.parts[0].front_m, BELT_LENGTH_M);
        for pair in s.parts.windows(2) {
            let gap = pair[0].front_m - PART_LENGTH_M - pair[1].front_m;
            assert!(gap >= -1e-9, "parts overlap: {pair:?}");
        }
        let full = (BELT_LENGTH_M / PART_LENGTH_M).floor() as usize;
        assert_eq!(s.parts.len(), full, "the belt fills to its capacity");
    }

    #[test]
    fn the_photocell_sees_a_part_only_while_it_covers_the_point() {
        let mut s = Scene::default();
        let mut edges = Vec::new();
        let mut last = false;
        for _ in 0..800 {
            s.step(DT, true);
            let now = s.photocell();
            if now != last {
                edges.push((now, s.parts.first().map(|p| p.front_m)));
                last = now;
            }
        }
        assert!(edges.len() >= 2, "{edges:?}");
        assert!(edges[0].0, "the first edge is rising");
        let (_, front) = edges[0];
        assert!((front.unwrap() - PHOTOCELL_AT_M).abs() < BELT_SPEED_M_S * DT + 1e-9);
    }

    #[test]
    fn a_held_part_covers_the_photocell_continuously() {
        let mut s = Scene {
            outlet_blocked: true,
            ..Scene::default()
        };
        run(&mut s, 7.0, true);
        assert!(s.photocell());
        run(&mut s, 5.0, true);
        assert!(s.photocell());
    }
}
