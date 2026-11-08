//! M1's belt in closed form, with sensor edges exact to the nanosecond (spec 2026-11-08-analytic-belt-design.md;
//! the scheme is JaamSim's `EntityConveyor`, research r1 §4). Positions are integer nanometres and times integer
//! nanoseconds. Between two events every moving part travels the same distance, so a front is a pure function of
//! time: `front_ref + ⌊v·(t − t_ref)⌋`, capped by the part ahead and, with the outlet blocked, by the end. The
//! references move only at events (`set_motor`, `set_outlet_blocked`, `wake`), never on a read, so reading the belt
//! cannot change it.

use crate::scene::Part;

// The scene's dimensions, in the integer units that are the single source of truth (scene.rs derives its metres).
pub const NM_PER_M: i64 = 1_000_000_000;
pub const NS_PER_S: i64 = 1_000_000_000;
pub const LENGTH_NM: i64 = 3_000_000_000;
pub const PART_NM: i64 = 300_000_000;
pub const PHOTOCELL_NM: i64 = 2_700_000_000;
pub const SPEED_NM_PER_S: i64 = 500_000_000;
pub const ARRIVAL_INTERVAL_NS: i64 = 2_000_000_000;
/// M1's first part arrives at its first scan, 10 ms.
const FIRST_ARRIVAL_NS: i64 = 10_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Front {
    id: u64,
    nm: i64,
}

#[derive(Debug, Clone)]
pub struct Belt {
    /// Fronts at `t_ref`, head first.
    fronts: Vec<Front>,
    t_ref: i64,
    motor: bool,
    outlet_blocked: bool,
    delivered: u64,
    next_id: u64,
    next_arrival: i64,
    /// Events handled so far, to catch a belt that asks for absurdly many (see `wake`).
    wakes: u64,
}

impl Default for Belt {
    fn default() -> Self {
        Self {
            fronts: Vec::new(),
            t_ref: 0,
            motor: false,
            outlet_blocked: false,
            delivered: 0,
            next_id: 1,
            next_arrival: FIRST_ARRIVAL_NS,
            wakes: 0,
        }
    }
}

/// Distance travelled in `dt` ns, rounded down to the nanometre.
fn travel(dt: i64) -> i64 {
    (i128::from(SPEED_NM_PER_S) * i128::from(dt) / i128::from(NS_PER_S)) as i64
}

/// The first time after `t_ref` at which a front moving from `from` reaches `to`, rounded up to the nanosecond.
fn reach(t_ref: i64, from: i64, to: i64) -> i64 {
    let num = i128::from(to - from) * i128::from(NS_PER_S);
    let den = i128::from(SPEED_NM_PER_S);
    t_ref + ((num + den - 1) / den) as i64
}

impl Belt {
    /// Fronts at `t`, head first, and the number of parts that left the belt since `t_ref`.
    fn fronts_at(&self, t: i64) -> (Vec<Front>, u64) {
        debug_assert!(
            t >= self.t_ref,
            "the belt is read at {t} ns, before its last event at {} ns",
            self.t_ref
        );
        let moved = if self.motor {
            travel(t - self.t_ref)
        } else {
            0
        };
        let mut limit = if self.outlet_blocked {
            LENGTH_NM
        } else {
            i64::MAX
        };
        let mut fronts = Vec::with_capacity(self.fronts.len());
        for f in &self.fronts {
            let nm = (f.nm + moved).min(limit);
            limit = nm - PART_NM;
            fronts.push(Front { id: f.id, nm });
        }
        let before = fronts.len();
        fronts.retain(|f| f.nm < LENGTH_NM || self.outlet_blocked);
        let left = (before - fronts.len()) as u64;
        (fronts, left)
    }

    /// Brings the references to `t`: the parts that left are counted, and a due part arrives if the entry is free.
    fn rebase(&mut self, t: i64) {
        let (fronts, left) = self.fronts_at(t);
        self.fronts = fronts;
        self.delivered += left;
        self.t_ref = t;
        let entry_free = self.fronts.last().is_none_or(|f| f.nm - PART_NM >= PART_NM);
        if t >= self.next_arrival && entry_free {
            self.fronts.push(Front {
                id: self.next_id,
                nm: PART_NM,
            });
            self.next_id += 1;
            self.next_arrival = t + ARRIVAL_INTERVAL_NS;
        }
    }

    pub fn motor_is(&self, on: bool) -> bool {
        self.motor == on
    }

    pub fn set_motor(&mut self, t: i64, on: bool) {
        if on != self.motor {
            self.rebase(t);
            self.motor = on;
        }
    }

    pub fn set_outlet_blocked(&mut self, t: i64, blocked: bool) {
        if blocked != self.outlet_blocked {
            self.rebase(t);
            self.outlet_blocked = blocked;
        }
    }

    /// Handles the event scheduled for `t` by `next_event`.
    pub fn wake(&mut self, t: i64) {
        debug_assert_eq!(
            self.next_event(),
            Some(t),
            "a wake-up at {t} that is not the belt's next event"
        );
        // About 2 events per simulated second on the free belt (7 196 in an hour, examples/belt_benchmark.rs). A belt
        // asking for thousands per second is broken, and stops here rather than running for hours of wall time.
        self.wakes += 1;
        debug_assert!(
            self.wakes <= 1_000 + 10_000 * (t / NS_PER_S) as u64,
            "{} events by {t} ns",
            self.wakes
        );
        self.rebase(t);
    }

    /// The time of the next change the belt must handle after its last event: an arrival, a part leaving, or a front
    /// crossing a point where the photocell or the entry change. `None` when nothing will change on its own.
    pub fn next_event(&self) -> Option<i64> {
        let mut next = None;
        let mut candidate = |t: i64| next = Some(next.map_or(t, |n: i64| n.min(t)));
        let entry_free = self.fronts.last().is_none_or(|f| f.nm - PART_NM >= PART_NM);
        if entry_free {
            candidate(self.next_arrival.max(self.t_ref));
        }
        if !self.motor {
            return next;
        }
        let mut stop = if self.outlet_blocked {
            LENGTH_NM
        } else {
            i64::MAX
        };
        for f in &self.fronts {
            // 2 * PART_NM is where the entry frees: only the last part can be short of it.
            let mut targets = vec![PHOTOCELL_NM, PHOTOCELL_NM + PART_NM + 1, 2 * PART_NM];
            if !self.outlet_blocked {
                targets.push(LENGTH_NM);
            }
            for to in targets {
                if f.nm < to && to <= stop {
                    candidate(reach(self.t_ref, f.nm, to));
                }
            }
            stop = stop.saturating_sub(PART_NM);
        }
        next
    }

    /// TRUE while a part covers the photocell's point.
    pub fn photocell_at(&self, t: i64) -> bool {
        let (fronts, _) = self.fronts_at(t);
        fronts
            .iter()
            .any(|f| f.nm >= PHOTOCELL_NM && f.nm - PART_NM <= PHOTOCELL_NM)
    }

    pub fn delivered_at(&self, t: i64) -> u64 {
        self.delivered + self.fronts_at(t).1
    }

    pub fn outlet_blocked(&self) -> bool {
        self.outlet_blocked
    }

    /// The parts at `t`, in metres, for the view.
    pub fn parts_at(&self, t: i64) -> Vec<Part> {
        self.fronts_at(t)
            .0
            .iter()
            .map(|f| Part {
                id: f.id,
                front_m: f.nm as f64 / NM_PER_M as f64,
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "belt_tests.rs"]
mod tests;
