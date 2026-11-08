//! The closed-form belt against M1's fixed-step belt over one simulated hour (ROADMAP 3.3, spec
//! 2026-11-08-analytic-belt-design.md): photocell edge error and computing cost. Motor on, outlet free.
//!
//!   cargo run --release --example belt_benchmark

use std::time::Instant;

use twinova_lib::belt::{Belt, NS_PER_S};
use twinova_lib::scene::Scene;

const HOUR_NS: i64 = 3_600 * NS_PER_S;

/// Rising edges of the photocell, exact, from the belt's own events.
fn analytic() -> (Vec<i64>, u64) {
    let mut belt = Belt::default();
    belt.set_motor(0, true);
    let (mut edges, mut events, mut covered) = (Vec::new(), 0, false);
    while let Some(t) = belt.next_event().filter(|t| *t <= HOUR_NS) {
        belt.wake(t);
        events += 1;
        let now = belt.photocell_at(t);
        if now && !covered {
            edges.push(t);
        }
        covered = now;
    }
    (edges, events)
}

/// What stepping M1's belt every `dt_ns` sees: for each part, its arrival (the end of the step that adds it) and its
/// rising edge (the end of the first step that finds it covering the photocell); and the number of steps.
fn stepped(dt_ns: i64) -> (Vec<(i64, i64)>, u64) {
    let mut scene = Scene::default();
    let (mut arrivals, mut parts, mut t, mut steps) = (Vec::new(), Vec::new(), 0, 0);
    let mut covered = false;
    while t < HOUR_NS {
        let before = scene.parts.len();
        scene.step(dt_ns as f64 / NS_PER_S as f64, true);
        t += dt_ns;
        steps += 1;
        if scene
            .parts
            .last()
            .is_some_and(|p| p.id as usize > before + scene.delivered as usize)
        {
            arrivals.push(t);
        }
        let now = scene.photocell();
        if now && !covered {
            parts.push((arrivals[parts.len()], t));
        }
        covered = now;
    }
    (parts, steps)
}

fn timed<T>(f: impl Fn() -> T) -> (T, f64) {
    (0..3)
        .map(|_| {
            let start = Instant::now();
            let out = f();
            (out, start.elapsed().as_secs_f64() * 1e3)
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap()
}

fn main() {
    let ((exact, events), ms) = timed(analytic);
    println!(
        "closed form: {} edges, {events} events, {ms:.2} ms for one simulated hour",
        exact.len()
    );
    // A part's front reaches the photocell 4.8 s after it arrives (2.40 m at 0.5 m/s): the exact edge of each
    // stepped part comes from its own arrival, so the error measured is the sampling error alone.
    const TRAVEL_NS: i64 = 4_800_000_000;
    for dt_ms in [10, 1] {
        let ((parts, steps), ms) = timed(|| stepped(dt_ms * 1_000_000));
        let late: Vec<i64> = parts
            .iter()
            .map(|(arrival, seen)| seen - (arrival + TRAVEL_NS))
            .collect();
        let (min, max) = (late.iter().min().unwrap(), late.iter().max().unwrap());
        let mean = late.iter().sum::<i64>() as f64 / late.len() as f64;
        let (first, last) = (parts[0].0, parts[parts.len() - 1].0);
        let drift = last - first - (parts.len() as i64 - 1) * 2 * NS_PER_S;
        println!(
            "fixed step {dt_ms:>2} ms: {} edges, {steps} steps, {ms:.2} ms; edge late by min {:.3} / mean {:.3} / \
             max {:.3} ms; arrivals drift {:.3} ms over the hour",
            parts.len(),
            *min as f64 / 1e6,
            mean / 1e6,
            *max as f64 / 1e6,
            drift as f64 / 1e6
        );
    }
}
