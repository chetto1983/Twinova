//! The order of same-time events on the bench's configuration: one thread with the FIFO executor
//! (spec 2026-11-07-des-core-design.md, section E). Fails if the nexosim patch is dropped.

use std::time::Duration;

use nexosim::model::{Context, Model, schedulable};
use nexosim::ports::{EventSinkReader, Output, SinkState, event_queue};
use nexosim::simulation::{Mailbox, SimInit};
use nexosim::time::MonotonicTime;
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Emitter {
    id: usize,
    at_ms: Vec<u64>,
    out: Output<(u64, usize)>,
}

#[Model]
impl Emitter {
    #[nexosim(init)]
    async fn init(&mut self, cx: &Context<Self>) {
        for &at in &self.at_ms {
            cx.schedule_event(Duration::from_millis(at), schedulable!(Self::emit), at)
                .unwrap();
        }
    }

    #[nexosim(schedulable)]
    async fn emit(&mut self, at: u64) {
        self.out.send((at, self.id)).await;
    }
}

/// Runs the emitters, registered in order, and returns `(time, emitter)` in processing order.
fn processing_order(schedules: &[Vec<u64>]) -> Vec<(u64, usize)> {
    let (sink, mut seen) = event_queue(SinkState::Enabled);
    let mut init = SimInit::with_num_threads(1);
    for (id, at_ms) in schedules.iter().enumerate() {
        let mut emitter = Emitter {
            id,
            at_ms: at_ms.clone(),
            out: Output::default(),
        };
        emitter.out.connect_sink(sink.clone());
        init = init.add_model(emitter, Mailbox::new(), &format!("e{id}"));
    }
    let mut simu = init.init(MonotonicTime::EPOCH).unwrap();
    simu.step_until(Duration::from_millis(100)).unwrap();
    std::iter::from_fn(|| seen.try_read()).collect()
}

proptest! {
    /// Any number of models, each with its own event times, many of them equal: events come out by time, and at one
    /// instant by registration order (then, for one model, in the order it scheduled them).
    #[test]
    fn same_time_events_follow_time_then_registration_order(
        schedules in prop::collection::vec(prop::collection::vec(1u64..5, 0..4), 1..9)
    ) {
        let mut expected: Vec<(u64, usize)> = schedules
            .iter()
            .enumerate()
            .flat_map(|(id, at)| at.iter().map(move |&t| (t, id)))
            .collect();
        expected.sort_by_key(|&(t, id)| (t, id));
        prop_assert_eq!(processing_order(&schedules), expected);
    }
}
