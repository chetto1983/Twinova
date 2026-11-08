//! What a snapshot of a long run costs to restore (ROADMAP 3.5, spec 2026-11-08-run-modes-design.md): a restore
//! replays every scan from zero, so its cost grows with the snapshot's virtual time. Default program, line started.
//!
//!   cargo run --release --example replay_cost [simulated-seconds]

use std::time::Instant;

use twinova_lib::commands::DEFAULT_PROGRAM;
use twinova_lib::engine::Engine;
use twinova_lib::plc::Plc;

fn main() -> Result<(), String> {
    let seconds: i64 = std::env::args()
        .nth(1)
        .map_or(Ok(3_600), |s| s.parse())
        .map_err(|e| format!("simulated seconds: {e}"))?;
    let program = || Plc::build(DEFAULT_PROGRAM);

    let mut engine = Engine::new(program()?)?;
    engine.press(true, false)?;
    engine.step()?;
    engine.press(false, false)?;
    let started = Instant::now();
    while engine.time_ms() < seconds * 1_000 {
        engine.step()?;
    }
    let run = started.elapsed();
    let saved = engine.save();

    let started = Instant::now();
    let restored = Engine::restore(program()?, &saved)?;
    let restore = started.elapsed();
    assert_eq!(restored.time_ms(), engine.time_ms());
    assert!(restored.snapshot().motor, "the line was started");

    println!(
        "{seconds} s simulated: run {:.2} s ({:.0} × real time), restore {:.2} s",
        run.as_secs_f64(),
        seconds as f64 / run.as_secs_f64(),
        restore.as_secs_f64()
    );
    Ok(())
}
