//! The M1 scene (one belt, the default ST program) as two nexosim models, against M1's own lockstep engine.
//! nexosim 1.0 docs: model.rs "Model environment" (`Model::Env`, `ProtoModel`), lib.rs "Authoring models".

use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;

use nexosim::model::{BuildContext, Context, Model, ProtoModel, schedulable};
use nexosim::ports::{EventSinkReader, Output, SinkState, event_queue};
use nexosim::simulation::{Mailbox, SimInit};
use nexosim::time::MonotonicTime;
use serde::{Deserialize, Serialize};
use twinova_lib::commands::DEFAULT_PROGRAM;
use twinova_lib::engine::{Engine, SCAN_MS};
use twinova_lib::plc::{Inputs, Plc};
use twinova_lib::scene::Scene;

const SCAN: Duration = Duration::from_millis(SCAN_MS as u64);

fn scene_with(outlet_blocked: bool) -> Scene {
    let mut scene = Scene::default();
    scene.outlet_blocked = outlet_blocked;
    scene
}

fn plc_print(motor: bool, jam: Option<bool>) -> u64 {
    let mut h = DefaultHasher::new();
    (motor, jam).hash(&mut h);
    h.finish()
}

fn scene_print(scene: &Scene) -> u64 {
    let mut h = DefaultHasher::new();
    (scene.photocell(), scene.delivered).hash(&mut h);
    for part in &scene.parts {
        (part.id, part.front_m.to_bits()).hash(&mut h);
    }
    h.finish()
}

/// The ST controller: one scan every 10 ms, with the sensor value the scene last sent.
#[derive(Serialize, Deserialize)]
pub struct PlcModel {
    pub motor: Output<bool>,
    pub print: Output<u64>,
    sensor: bool,
    start: bool,
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

    #[nexosim(schedulable)]
    async fn scan(&mut self, _: (), cx: &Context<Self>, plc: &mut Plc) {
        let inputs = Inputs {
            sensor: self.sensor,
            start: std::mem::take(&mut self.start),
            stop: false,
        };
        let motor = plc.scan(inputs, SCAN_MS).expect("scan");
        self.print.send(plc_print(motor, plc.bool_var("jam"))).await;
        self.motor.send(motor).await;
        cx.schedule_event(SCAN, schedulable!(Self::scan), ())
            .unwrap();
    }
}

/// Builds the controller with its truST program, which is not serializable and so lives in the environment. The
/// ports are connected on the prototype, then moved into the model.
pub struct ProtoPlc {
    plc: Plc,
    motor: Output<bool>,
    print: Output<u64>,
}

impl ProtoModel for ProtoPlc {
    type Model = PlcModel;

    fn build(self, _: &mut BuildContext<Self>) -> (PlcModel, Plc) {
        (
            PlcModel {
                motor: self.motor,
                print: self.print,
                sensor: false,
                start: true,
            },
            self.plc,
        )
    }
}

/// The belt: advances one scan with the motor the controller decided, then reports its photocell.
#[derive(Serialize, Deserialize)]
pub struct SceneModel {
    pub sensor: Output<bool>,
    pub print: Output<u64>,
}

#[Model(type Env = Scene)]
impl SceneModel {
    pub async fn motor(&mut self, on: bool, _: &Context<Self>, scene: &mut Scene) {
        scene.step(SCAN.as_secs_f64(), on);
        self.print.send(scene_print(scene)).await;
        self.sensor.send(scene.photocell()).await;
    }
}

pub struct ProtoScene {
    scene: Scene,
    sensor: Output<bool>,
    print: Output<u64>,
}

impl ProtoModel for ProtoScene {
    type Model = SceneModel;

    fn build(self, _: &mut BuildContext<Self>) -> (SceneModel, Scene) {
        (
            SceneModel {
                sensor: self.sensor,
                print: self.print,
            },
            self.scene,
        )
    }
}

/// Per-scan prints of the controller and of the scene, as M1's engine produces them.
pub fn reference(outlet_blocked: bool, seconds: u64) -> (Vec<u64>, Vec<u64>) {
    let mut engine = Engine::new(Plc::build(DEFAULT_PROGRAM).unwrap());
    engine.set_outlet_blocked(outlet_blocked);
    engine.press(true, false);
    let mut scene = scene_with(outlet_blocked);
    let (mut plc, mut belt) = (Vec::new(), Vec::new());
    for _ in 0..seconds * 1000 / SCAN_MS as u64 {
        engine.step().unwrap();
        let s = engine.snapshot();
        plc.push(plc_print(s.motor, s.jam));
        // The engine keeps its scene private: replay it with the engine's motor to print the same state.
        scene.step(SCAN.as_secs_f64(), s.motor);
        belt.push(scene_print(&scene));
        assert_eq!(scene.photocell(), s.sensor);
        assert_eq!(scene.delivered, s.delivered);
    }
    (plc, belt)
}

/// The same scene on nexosim, on one thread.
pub fn nexosim(outlet_blocked: bool, seconds: u64) -> (Vec<u64>, Vec<u64>) {
    let (plc_mbox, scene_mbox) = (Mailbox::new(), Mailbox::new());
    let (plc_sink, mut plc_prints) = event_queue(SinkState::Enabled);
    let (scene_sink, mut scene_prints) = event_queue(SinkState::Enabled);
    let mut plc = ProtoPlc {
        plc: Plc::build(DEFAULT_PROGRAM).unwrap(),
        motor: Output::default(),
        print: Output::default(),
    };
    let mut scene = ProtoScene {
        scene: scene_with(outlet_blocked),
        sensor: Output::default(),
        print: Output::default(),
    };
    plc.motor.connect(SceneModel::motor, &scene_mbox);
    plc.print.connect_sink(plc_sink);
    scene.sensor.connect(PlcModel::sensor, &plc_mbox);
    scene.print.connect_sink(scene_sink);
    let mut simu = SimInit::with_num_threads(1)
        .add_model(plc, plc_mbox, "plc")
        .add_model(scene, scene_mbox, "scene")
        .init(MonotonicTime::EPOCH)
        .unwrap();
    simu.step_until(Duration::from_secs(seconds)).unwrap();
    (
        std::iter::from_fn(|| plc_prints.try_read()).collect(),
        std::iter::from_fn(|| scene_prints.try_read()).collect(),
    )
}
