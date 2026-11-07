//! The same periodic emitters as nexosim models (nexosim 1.0 docs: lib.rs "Authoring models", examples/stepper_motor.rs).

use std::time::Duration;

use nexosim::model::{Context, Model, schedulable};
use nexosim::ports::{EventSinkReader, Output, SinkState, event_queue};
use nexosim::simulation::{Mailbox, SimInit};
use nexosim::time::MonotonicTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Emitter {
    pub out: Output<usize>,
    id: usize,
    period_ns: u64,
    count: u64,
    send: bool,
    /// Busy work before sending, so that threads finish in different orders from run to run.
    work: u64,
}

#[Model]
impl Emitter {
    #[nexosim(init)]
    async fn init(&mut self, cx: &Context<Self>) {
        cx.schedule_event(
            Duration::from_nanos(self.period_ns),
            schedulable!(Self::tick),
            (),
        )
        .unwrap();
    }

    #[nexosim(schedulable)]
    async fn tick(&mut self, _: (), cx: &Context<Self>) {
        self.count += 1;
        let mut x = self.count;
        for i in 0..self.work {
            x = std::hint::black_box(x.wrapping_mul(6364136223846793005).wrapping_add(i));
        }
        std::hint::black_box(x);
        if self.send {
            self.out.send(self.id).await;
        }
        cx.schedule_event(
            Duration::from_nanos(self.period_ns),
            schedulable!(Self::tick),
            (),
        )
        .unwrap();
    }
}

/// Collects the emitter ids in the order the collector processes them.
#[derive(Serialize, Deserialize, Default)]
pub struct Collector {
    pub seen: Output<usize>,
}

#[Model]
impl Collector {
    pub async fn input(&mut self, id: usize) {
        self.seen.send(id).await;
    }
}

type Seen = nexosim::ports::EventQueueReader<usize>;

/// The bench: `emitters` periodic emitters on `threads` threads. With `collect`, every emitter sends its id to one
/// collector at each tick, and the ids come back in the collector's processing order.
fn bench(
    threads: usize,
    emitters: usize,
    period_ns: u64,
    collect: bool,
    work: &dyn Fn(usize) -> u64,
) -> (SimInit, Seen) {
    let collector_mbox = Mailbox::new();
    let mut collector = Collector::default();
    let (sink, seen) = event_queue(SinkState::Enabled);
    collector.seen.connect_sink(sink);
    let mut bench = SimInit::with_num_threads(threads);
    for id in 0..emitters {
        let mut emitter = Emitter {
            out: Output::default(),
            id,
            period_ns,
            count: 0,
            send: collect,
            work: work(id),
        };
        if collect {
            emitter.out.connect(Collector::input, &collector_mbox);
        }
        bench = bench.add_model(emitter, Mailbox::new(), &format!("e{id}"));
    }
    (
        bench.add_model(collector, collector_mbox, "collector"),
        seen,
    )
}

fn drain(seen: &mut Seen) -> Vec<usize> {
    std::iter::from_fn(|| seen.try_read()).collect()
}

/// Runs the bench until `end_ns`.
pub fn run(
    threads: usize,
    emitters: usize,
    period_ns: u64,
    end_ns: u64,
    collect: bool,
    work: impl Fn(usize) -> u64,
) -> Vec<usize> {
    let (bench, mut seen) = bench(threads, emitters, period_ns, collect, &work);
    let mut simu = bench.init(MonotonicTime::EPOCH).unwrap();
    simu.step_until(Duration::from_nanos(end_ns)).unwrap();
    drain(&mut seen)
}

/// Runs until `split_ns`, saves the state, restores it into a freshly built bench and runs on until `end_ns`
/// (nexosim examples/bench_save_restore.rs). Returns the order seen and the size of the saved state.
pub fn run_with_restore(
    threads: usize,
    emitters: usize,
    period_ns: u64,
    split_ns: u64,
    end_ns: u64,
) -> (Vec<usize>, usize) {
    let (bench1, mut seen1) = bench(threads, emitters, period_ns, true, &|_| 0);
    let mut simu = bench1.init(MonotonicTime::EPOCH).unwrap();
    simu.step_until(Duration::from_nanos(split_ns)).unwrap();
    let mut state = Vec::new();
    simu.save(&mut state).unwrap();
    drop(simu);
    let mut order = drain(&mut seen1);
    let (bench2, mut seen2) = bench(threads, emitters, period_ns, true, &|_| 0);
    let mut simu = bench2.restore(&state[..]).unwrap();
    simu.step_until(MonotonicTime::EPOCH + Duration::from_nanos(end_ns))
        .unwrap();
    order.extend(drain(&mut seen2));
    (order, state.len())
}
