use super::*;
use proptest::prelude::*;

const S: i64 = 1_000_000_000;
const MS: i64 = 1_000_000;

/// Drives the belt like the bench: external changes at their times, the belt's own events in between. Returns, for
/// every handled instant, the belt as it was just before it (to probe the interval that ends there) and the time.
fn drive(belt: &mut Belt, changes: &[(i64, Change)], until: i64) -> Vec<(Belt, i64)> {
    let mut seen = Vec::new();
    let mut changes = changes.iter().peekable();
    loop {
        let own = belt.next_event();
        let external = changes.peek().map(|(t, _)| *t);
        let t = match (own, external) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => break,
        };
        if t > until {
            break;
        }
        seen.push((belt.clone(), t));
        if own == Some(t) {
            belt.wake(t);
        }
        while let Some((_, change)) = changes.next_if(|(at, _)| *at == t) {
            match *change {
                Change::Motor(on) => belt.set_motor(t, on),
                Change::Outlet(blocked) => belt.set_outlet_blocked(t, blocked),
            }
        }
    }
    seen
}

#[derive(Debug, Clone, Copy)]
enum Change {
    Motor(bool),
    Outlet(bool),
}

fn running() -> Belt {
    let mut belt = Belt::default();
    belt.set_motor(0, true);
    belt
}

/// The first handled instant at which `seen_at` holds, checked to be false one nanosecond earlier.
fn first_exact(seen: &[(Belt, i64)], seen_at: impl Fn(&Belt, i64) -> bool) -> Option<i64> {
    let (before, t) = seen.iter().find(|(before, t)| seen_at(before, *t))?;
    assert!(!seen_at(before, t - 1), "not exact at {t}");
    Some(*t)
}

#[test]
fn the_first_part_covers_the_photocell_exactly_when_its_front_reaches_it() {
    let mut belt = running();
    let seen = drive(&mut belt, &[], 5 * S);
    // Arrives at 10 ms with its front at 0.30 m; 2.40 m at 0.5 m/s is 4.8 s.
    assert_eq!(
        first_exact(&seen, |b, t| b.photocell_at(t)),
        Some(4_810 * MS)
    );
}

#[test]
fn parts_leave_the_free_outlet_on_time() {
    let mut belt = running();
    let seen = drive(&mut belt, &[], 60 * S);
    // Part n arrives at 10 ms + 2n s and leaves 5.4 s later: n = 0..=27 have left by 60 s.
    assert_eq!(
        first_exact(&seen, |b, t| b.delivered_at(t) == 1),
        Some(5_410 * MS)
    );
    assert_eq!(
        first_exact(&seen, |b, t| b.delivered_at(t) == 2),
        Some(7_410 * MS)
    );
    assert_eq!(belt.delivered_at(60 * S), 28);
}

#[test]
fn a_blocked_outlet_fills_the_belt_nose_to_tail() {
    let mut belt = running();
    belt.set_outlet_blocked(0, true);
    drive(&mut belt, &[], 60 * S);
    let parts = belt.parts_at(60 * S);
    assert_eq!(belt.delivered_at(60 * S), 0);
    assert_eq!(parts.len(), 10);
    for (i, part) in parts.iter().enumerate() {
        assert_eq!(
            (part.front_m * NM_PER_M as f64).round() as i64,
            LENGTH_NM - i as i64 * PART_NM
        );
    }
    assert!(belt.photocell_at(60 * S));
    assert_eq!(
        belt.next_event(),
        None,
        "a full, blocked belt has nothing left to do"
    );
}

#[test]
fn a_stopped_belt_keeps_its_parts_and_admits_no_new_one() {
    let mut belt = Belt::default();
    drive(&mut belt, &[], 30 * S);
    assert_eq!(
        belt.parts_at(30 * S).len(),
        1,
        "the first part sits in the entry"
    );
    assert_eq!(belt.next_event(), None);
}

#[test]
fn reading_the_belt_does_not_move_it() {
    let mut belt = running();
    drive(&mut belt, &[], 7 * S);
    let first = belt.parts_at(7 * S);
    for t in [7 * S, 7 * S + 1, 9 * S, 7 * S] {
        let _ = (belt.photocell_at(t), belt.parts_at(t), belt.delivered_at(t));
    }
    assert_eq!(belt.parts_at(7 * S), first);
}

fn changes() -> impl Strategy<Value = Vec<(i64, Change)>> {
    prop::collection::vec((1i64..30_000_000, any::<bool>(), any::<bool>()), 0..12).prop_map(|raw| {
        let mut changes: Vec<(i64, Change)> = raw
            .into_iter()
            .map(|(us, motor, on)| {
                let change = if motor {
                    Change::Motor(on)
                } else {
                    Change::Outlet(on)
                };
                (us * 1_000 + 7, change)
            })
            .collect();
        changes.sort_by_key(|(t, _)| *t);
        changes
    })
}

proptest! {
    /// Whatever the motor and the outlet do, the photocell only changes at an instant the belt handles, and exactly
    /// there: one nanosecond earlier it had the other value, and it never changes between two such instants.
    #[test]
    fn photocell_edges_fall_exactly_on_handled_instants(changes in changes(), probes in prop::collection::vec(0.0f64..1.0, 8)) {
        let mut belt = running();
        let seen = drive(&mut belt, &changes, 40 * S);
        let mut previous = 0;
        for (before, t) in &seen {
            let start = before.photocell_at(previous);
            for p in &probes {
                let probe = previous + ((*t - previous) as f64 * p) as i64;
                if probe < *t {
                    prop_assert_eq!(before.photocell_at(probe), start, "changed between {} and {}", previous, t);
                }
            }
            if before.photocell_at(*t) != start {
                prop_assert_eq!(before.photocell_at(*t - 1), start, "the edge at {} is not exact", t);
            }
            previous = *t;
        }
    }
}
