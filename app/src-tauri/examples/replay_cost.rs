//! What a snapshot of a long run costs to restore (ROADMAP 3.5, spec 2026-11-08-run-modes-design.md): a restore
//! replays the line from zero, so its cost grows with the snapshot's virtual time. Since 4.3 the line steps its physics
//! every millisecond (spec 2026-11-09-component-behaviour-design.md). The bundled demo, line started.
//!
//!   cargo run --release --example replay_cost [simulated-seconds]

use std::time::Instant;

use twinova_lib::demo::demo;
use twinova_lib::engine::Engine;

fn main() -> Result<(), String> {
    let seconds: i64 = std::env::args()
        .nth(1)
        .map_or(Ok(600), |s| s.parse())
        .map_err(|e| format!("simulated seconds: {e}"))?;
    let project = demo();
    let (mut engine, _) = Engine::new(&project)?;
    engine.press("%IX1.0");
    let started = Instant::now();
    while engine.time_ms() < seconds * 1_000 {
        engine.step()?;
    }
    let run = started.elapsed();
    let saved = engine.save();
    let started = Instant::now();
    let restored = Engine::restore(&project, &saved)?;
    let restore = started.elapsed();
    assert_eq!(
        restored.snapshot(),
        engine.snapshot(),
        "the restored state is the saved one"
    );
    println!("{seconds} s of virtual time: run {run:?}, restore {restore:?}");
    Ok(())
}
