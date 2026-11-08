//! The scene and its controller as nexosim models on one thread (spec 2026-11-07-des-core-design.md, "Decision";
//! plan 2026-11-08-des-core-integration.md). One scan is a same-instant message chain in M1's lockstep order:
//! the controller scans with the last photocell value and sends `motor`, the belt steps and sends the photocell back.
//! truST's harness and the belt are not serializable, so each lives in its model's environment (nexosim 1.0,
//! model.rs "Model environment"); nexosim's save and restore do not cover them.

use std::time::Duration;

use nexosim::model::{BuildContext, Context, Model, ProtoModel, schedulable};
use nexosim::ports::{
    EventQueueReader, EventSinkReader, EventSource, Output, SinkState, event_queue,
};
use nexosim::simulation::{EventId, Mailbox, SimInit, Simulation};
use nexosim::time::MonotonicTime;
use serde::{Deserialize, Serialize};

use crate::plc::{Inputs, Plc};
use crate::scene::{Part, Scene};

pub const SCAN_MS: i64 = 10;
const SCAN: Duration = Duration::from_millis(SCAN_MS as u64);

/// What the controller reports after each scan. A runtime fault stops the scans.
#[derive(Debug, Clone, Default)]
pub struct ControllerState {
    pub motor: bool,
    pub jam: Option<bool>,
    pub fault: Option<String>,
}

/// What the belt reports after each step or change.
#[derive(Debug, Clone, Default)]
pub struct BeltState {
    pub parts: Vec<Part>,
    pub delivered: u64,
    pub outlet_blocked: bool,
    pub sensor: bool,
}

impl BeltState {
    fn of(scene: &Scene) -> Self {
        Self {
            parts: scene.parts.clone(),
            delivered: scene.delivered,
            outlet_blocked: scene.outlet_blocked,
            sensor: scene.photocell(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PlcModel {
    motor: Output<bool>,
    state: Output<ControllerState>,
    sensor: bool,
    start: bool,
    stop: bool,
    motor_on: bool,
}

#[Model(type Env = Plc)]
impl PlcModel {
    #[nexosim(init)]
    async fn init(&mut self, cx: &Context<Self>, _: &mut Plc) {
        cx.schedule_event(SCAN, schedulable!(Self::scan), ())
            .unwrap();
    }

    pub fn sensor(&mut self, on: bool) {
        self.sensor = on;
    }

    /// A start or stop button press, held for the next scan only.
    pub fn press(&mut self, (start, stop): (bool, bool)) {
        self.start |= start;
        self.stop |= stop;
    }

    #[nexosim(schedulable)]
    async fn scan(&mut self, _: (), cx: &Context<Self>, plc: &mut Plc) {
        let inputs = Inputs {
            sensor: self.sensor,
            start: std::mem::take(&mut self.start),
            stop: std::mem::take(&mut self.stop),
        };
        match plc.scan(inputs, SCAN_MS) {
            Ok(motor) => {
                self.motor_on = motor;
                let state = self.state_of(plc, None);
                self.state.send(state).await;
                self.motor.send(motor).await;
                cx.schedule_event(SCAN, schedulable!(Self::scan), ())
                    .unwrap();
            }
            Err(fault) => {
                let state = self.state_of(plc, Some(fault));
                self.state.send(state).await;
            }
        }
    }

    /// Built before any `.await`: truST's harness is not `Sync`, so a reference to it cannot be held across one.
    fn state_of(&self, plc: &Plc, fault: Option<String>) -> ControllerState {
        ControllerState {
            motor: self.motor_on,
            jam: plc.bool_var("jam"),
            fault,
        }
    }
}

/// Builds the controller around its compiled program; the ports are connected here, then moved into the model.
struct ProtoPlc {
    plc: Plc,
    motor: Output<bool>,
    state: Output<ControllerState>,
}

impl ProtoModel for ProtoPlc {
    type Model = PlcModel;

    fn build(self, _: &mut BuildContext<Self>) -> (PlcModel, Plc) {
        let model = PlcModel {
            motor: self.motor,
            state: self.state,
            sensor: false,
            start: false,
            stop: false,
            motor_on: false,
        };
        (model, self.plc)
    }
}

#[derive(Serialize, Deserialize)]
pub struct BeltModel {
    sensor: Output<bool>,
    state: Output<BeltState>,
}

#[Model(type Env = Scene)]
impl BeltModel {
    pub async fn motor(&mut self, on: bool, _: &Context<Self>, scene: &mut Scene) {
        scene.step(SCAN.as_secs_f64(), on);
        self.state.send(BeltState::of(scene)).await;
        self.sensor.send(scene.photocell()).await;
    }

    pub async fn outlet_blocked(&mut self, blocked: bool, _: &Context<Self>, scene: &mut Scene) {
        scene.outlet_blocked = blocked;
        self.state.send(BeltState::of(scene)).await;
    }
}

struct ProtoBelt {
    sensor: Output<bool>,
    state: Output<BeltState>,
}

impl ProtoModel for ProtoBelt {
    type Model = BeltModel;

    fn build(self, _: &mut BuildContext<Self>) -> (BeltModel, Scene) {
        (
            BeltModel {
                sensor: self.sensor,
                state: self.state,
            },
            Scene::default(),
        )
    }
}

/// The bench and its endpoints. States arrive in queues; the caller keeps the latest of each.
pub struct Bench {
    simu: Simulation,
    press: EventId<(bool, bool)>,
    outlet: EventId<bool>,
    controller: EventQueueReader<ControllerState>,
    belt: EventQueueReader<BeltState>,
}

fn text(error: nexosim::simulation::ExecutionError) -> String {
    error.to_string()
}

impl Bench {
    /// The bench at time zero, with the states before the first scan.
    pub fn new(plc: Plc) -> Result<(Self, ControllerState, BeltState), String> {
        let first = ControllerState {
            motor: false,
            jam: plc.bool_var("jam"),
            fault: None,
        };
        let (plc_mbox, belt_mbox) = (Mailbox::new(), Mailbox::new());
        let (controller_sink, controller) = event_queue(SinkState::Enabled);
        let (belt_sink, belt) = event_queue(SinkState::Enabled);
        let mut proto_plc = ProtoPlc {
            plc,
            motor: Output::default(),
            state: Output::default(),
        };
        let mut proto_belt = ProtoBelt {
            sensor: Output::default(),
            state: Output::default(),
        };
        proto_plc.motor.connect(BeltModel::motor, &belt_mbox);
        proto_plc.state.connect_sink(controller_sink);
        proto_belt.sensor.connect(PlcModel::sensor, &plc_mbox);
        proto_belt.state.connect_sink(belt_sink);
        // One thread: same-time events follow the scheduler key (needs the FIFO executor, Cargo.toml).
        let mut init = SimInit::with_num_threads(1);
        let press = EventSource::new()
            .connect(PlcModel::press, &plc_mbox)
            .register(&mut init);
        let outlet = EventSource::new()
            .connect(BeltModel::outlet_blocked, &belt_mbox)
            .register(&mut init);
        let simu = init
            .add_model(proto_plc, plc_mbox, "plc")
            .add_model(proto_belt, belt_mbox, "belt")
            .init(MonotonicTime::EPOCH)
            .map_err(|e| e.to_string())?;
        let bench = Self {
            simu,
            press,
            outlet,
            controller,
            belt,
        };
        Ok((bench, first, BeltState::of(&Scene::default())))
    }

    pub fn time_ms(&self) -> i64 {
        let elapsed = self.simu.time().duration_since(MonotonicTime::EPOCH);
        i64::try_from(elapsed.as_millis()).expect("virtual time fits in i64 ms")
    }

    /// Runs every scan up to `time_ms`.
    pub fn advance_to(&mut self, time_ms: i64) -> Result<(), String> {
        let deadline = MonotonicTime::EPOCH + Duration::from_millis(time_ms as u64);
        self.simu.step_until(deadline).map_err(text)
    }

    pub fn press(&mut self, start: bool, stop: bool) -> Result<(), String> {
        self.simu
            .process_event(&self.press, (start, stop))
            .map_err(text)
    }

    pub fn set_outlet_blocked(&mut self, blocked: bool) -> Result<(), String> {
        self.simu.process_event(&self.outlet, blocked).map_err(text)
    }

    /// The latest state each model reported since the last call, if any.
    pub fn latest(&mut self) -> (Option<ControllerState>, Option<BeltState>) {
        (
            std::iter::from_fn(|| self.controller.try_read()).last(),
            std::iter::from_fn(|| self.belt.try_read()).last(),
        )
    }
}

#[cfg(test)]
#[path = "bench_tests.rs"]
mod tests;
