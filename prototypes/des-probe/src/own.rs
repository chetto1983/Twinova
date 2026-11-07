//! The baseline: a plain event queue ordered by (time in ns, priority, insertion sequence), research note 05 §6.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub struct Queue {
    heap: BinaryHeap<Reverse<(u64, u8, u64, usize)>>,
    seq: u64,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            seq: 0,
        }
    }

    pub fn schedule(&mut self, t_ns: u64, prio: u8, emitter: usize) {
        self.heap.push(Reverse((t_ns, prio, self.seq, emitter)));
        self.seq += 1;
    }

    pub fn pop(&mut self) -> Option<(u64, usize)> {
        self.heap.pop().map(|Reverse((t, _, _, e))| (t, e))
    }
}

/// `emitters` periodic emitters, all starting at `period_ns`, run until `end_ns`; `visit` sees every event in order.
pub fn run(emitters: usize, period_ns: u64, end_ns: u64, mut visit: impl FnMut(u64, usize)) -> u64 {
    let mut queue = Queue::new();
    for e in 0..emitters {
        queue.schedule(period_ns, 0, e);
    }
    let mut events = 0;
    while let Some((t, e)) = queue.pop() {
        if t > end_ns {
            break;
        }
        visit(t, e);
        events += 1;
        queue.schedule(t + period_ns, 0, e);
    }
    events
}
