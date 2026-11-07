//! Spike for ROADMAP 3.1: nexosim 1.0 against a plain (t, prio, seq) queue. See README.md.

mod nexo;
mod own;

use std::collections::BTreeSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Instant;

const MS: u64 = 1_000_000;

fn fingerprint(order: &[usize]) -> u64 {
    let mut h = DefaultHasher::new();
    order.hash(&mut h);
    h.finish()
}

fn determinism() {
    println!(
        "A. Same-instant order: 8 emitters tick together every 1 ms for 1 s, into one collector; 20 runs each."
    );
    let mut own_order = Vec::new();
    own::run(8, MS, 1_000 * MS, |_, e| own_order.push(e));
    let own_print = fingerprint(&own_order);
    // Emitter 0 does the most work, emitter 7 none: on several threads the last scheduled finishes first.
    let skewed = |id: usize| (7 - id as u64) * 20_000;
    for (label, work) in [
        ("no work", &(|_: usize| 0u64) as &dyn Fn(usize) -> u64),
        ("skewed work", &skewed),
    ] {
        for threads in [4, 1] {
            let prints: BTreeSet<u64> = (0..20)
                .map(|_| fingerprint(&nexo::run(threads, 8, MS, 1_000 * MS, true, work)))
                .collect();
            println!(
                "  nexosim, {label:11}, {threads} thread(s): {} distinct order(s) in 20 runs; same as the own queue: {}",
                prints.len(),
                prints.contains(&own_print)
            );
        }
    }
    println!(
        "  own queue: ids per instant {:?} (one order by construction)",
        &own_order[..8]
    );
    for threads in [1, 2, 3, 4, 8] {
        let order = nexo::run(threads, 8, MS, 1_000 * MS, true, |_| 0);
        println!(
            "  nexosim, {threads} thread(s): first two instants {:?}",
            &order[..16]
        );
    }
}

fn throughput() {
    println!(
        "B. Throughput: 1 000 emitters, period 1 ms, 1 s simulated = 1 000 000 events; best of 3."
    );
    let best = |f: &dyn Fn()| {
        (0..3)
            .map(|_| {
                let t = Instant::now();
                f();
                t.elapsed()
            })
            .min()
            .unwrap()
    };
    for threads in [1, 4] {
        let d = best(&|| {
            nexo::run(threads, 1_000, MS, 1_000 * MS, false, |_| 0);
        });
        println!(
            "  nexosim, {threads} thread(s): {:>8.1} ms, {:>6.2} M events/s",
            d.as_secs_f64() * 1e3,
            1.0 / d.as_secs_f64()
        );
    }
    let d = best(&|| {
        own::run(1_000, MS, 1_000 * MS, |_, _| {});
    });
    println!(
        "  own queue:            {:>8.1} ms, {:>6.2} M events/s",
        d.as_secs_f64() * 1e3,
        1.0 / d.as_secs_f64()
    );
}

fn save_restore() {
    println!(
        "D. Save at 0.5 s, restore into a new bench, run to 1 s, against one uninterrupted run (8 emitters, 1 ms)."
    );
    for threads in [1, 4] {
        let straight = nexo::run(threads, 8, MS, 1_000 * MS, true, |_| 0);
        let (resumed, bytes) = nexo::run_with_restore(threads, 8, MS, 500 * MS, 1_000 * MS);
        println!(
            "  nexosim, {threads} thread(s): {} events either way: {}; identical order: {}; state {bytes} bytes",
            straight.len(),
            straight.len() == resumed.len(),
            straight == resumed
        );
    }
}

fn main() {
    determinism();
    throughput();
    save_restore();
}
