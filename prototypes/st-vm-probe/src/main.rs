//! Probe: does truST's former bytecode VM run a conveyor function block correctly, and how fast?
//! The scan is driven in lockstep: set inputs, advance virtual time by one scan period, run one cycle.

use std::time::Instant;

use trust_runtime::harness::TestHarness;
use trust_runtime::value::{Duration, Value};

const SOURCE: &str = include_str!("conveyor.st");
const SCAN_MS: i64 = 10;

fn scan(plc: &mut TestHarness, sensor: bool) {
    plc.set_input("sensor", sensor);
    plc.advance_time(Duration::from_millis(SCAN_MS));
    plc.cycle();
}

fn output(plc: &TestHarness, name: &str) -> Value {
    plc.get_output(name)
        .unwrap_or_else(|| panic!("no output {name}"))
}

fn check_behaviour() {
    let mut plc = TestHarness::from_source(SOURCE).expect("compile conveyor program");

    plc.set_input("start", true);
    scan(&mut plc, false);
    plc.set_input("start", false);
    assert_eq!(output(&plc, "motor"), Value::Bool(true), "start latches the motor");

    for _ in 0..5 {
        scan(&mut plc, true);
        scan(&mut plc, false);
    }
    assert_eq!(output(&plc, "count"), Value::DInt(5), "one count per rising edge");
    assert_eq!(output(&plc, "jam"), Value::Bool(false));

    // A part held at the end for 2.5 s of virtual time trips the 2 s jam timer and stops the belt.
    for _ in 0..250 {
        scan(&mut plc, true);
    }
    assert_eq!(output(&plc, "jam"), Value::Bool(true), "TON trips after 2 s of virtual time");
    assert_eq!(output(&plc, "motor"), Value::Bool(false), "a jam stops the motor");
    assert_eq!(output(&plc, "count"), Value::DInt(6));
    println!("behaviour: ok (start latch, 6 counts, jam after 2 s virtual, motor off)");
}

fn bench(cycles: u32) {
    let mut plc = TestHarness::from_source(SOURCE).expect("compile conveyor program");
    plc.set_input("start", true);
    let t0 = Instant::now();
    for i in 0..cycles {
        scan(&mut plc, i % 7 == 0);
    }
    let elapsed = t0.elapsed().as_secs_f64();
    println!(
        "bench: {cycles} scans in {:.3} s = {:.0} scans/s ({:.2} us/scan), virtual time {} s",
        elapsed,
        f64::from(cycles) / elapsed,
        elapsed * 1e6 / f64::from(cycles),
        i64::from(cycles) * SCAN_MS / 1000
    );
}

fn main() {
    let t0 = Instant::now();
    TestHarness::from_source(SOURCE).expect("compile conveyor program");
    println!("compile (ST -> bytecode, in process): {:.1} ms", t0.elapsed().as_secs_f64() * 1e3);
    check_behaviour();
    bench(200_000);
}
