use super::*;
use crate::layout::{BELT_GRIP, Belt, Exit, Products, Sensor, SensorKind, Sink, Source, layout};
use std::path::Path;
use twinova_project::storage::open_folder;

pub(super) fn belt(id: &str, x: Real, length: Real, speed: Real, closed_exit: bool) -> Belt {
    Belt {
        id: id.into(),
        pose: Pose::from_translation(Vector::new(x, 0.0, 0.0)),
        length,
        width: 0.6,
        height: 0.8,
        speed,
        grip: BELT_GRIP,
        closed_exit,
        arc: None,
        guards: [false; 2],
        skew: 0.0,
        openings: Vec::new(),
        turntable: false,
    }
}

pub(super) fn source(belt: usize, first_ns: i64, interval_ns: i64, max_count: i64) -> Source {
    Source {
        id: "source".into(),
        belt,
        interval_ns,
        first_ns,
        max_count,
        products: Products::carton(),
    }
}

/// Belt A (3 m, 0.5 m/s) then belt B (2 m) at `speed_b`, a sink after B, one carton from a source on A.
fn two_belts(speed_b: Real, cartons: i64) -> World {
    let mut w = World::new(Layout {
        belts: vec![
            belt("a", 0.0, 3.0, 0.5, false),
            belt("b", 3.0, 2.0, speed_b, false),
        ],
        transfers: vec![],
        stoppers: vec![],
        pushers: vec![],
        turntables: vec![],
        sources: vec![source(0, 1_000_000, 1_000_000_000, cartons)],
        sinks: vec![Sink {
            id: "sink".into(),
            exit: Exit::End(1),
        }],
        sensors: vec![],
        scanners: vec![],
    });
    w.set_belt_running(0, true);
    w.set_belt_running(1, true);
    w
}

pub(super) fn run(w: &mut World, ms: usize) {
    for _ in 0..ms {
        w.step();
    }
}

fn x(w: &World, k: usize) -> f32 {
    w.cartons()[k].position[0]
}

#[test]
fn a_belt_carries_a_carton_at_its_speed() {
    let mut w = two_belts(0.5, 1);
    run(&mut w, 2001);
    let v = w.world.bodies[w.products[0].body].linvel().x;
    assert!((v - 0.5).abs() < 0.005, "{v}");
    let y = w.cartons()[0].position[1];
    assert!(
        (y - (0.8 + CARTON_HALF)).abs() < 0.002,
        "rests on the belt: {y}"
    );
}

#[test]
fn a_stopped_belt_holds_its_cartons() {
    let mut w = two_belts(0.5, 1);
    w.set_belt_running(0, false);
    run(&mut w, 1000);
    assert!(x(&w, 0) < CARTON_HALF + 0.01, "{}", x(&w, 0));
}

/// The carton crosses the seam between A and B without lifting, at B's speed or held by a stopped B.
#[test]
fn a_carton_passes_the_seam_without_lifting() {
    for (speed_b, delivered) in [(0.5, 1), (0.25, 1), (1.0, 1), (0.0, 0)] {
        let mut w = two_belts(speed_b, 1);
        let mut lift: f32 = 0.0;
        for ms in 1..=16_000 {
            w.step();
            // From 200 ms: the carton is born 1 mm above the belt and has settled by then.
            if let Some(c) = w.cartons().first().filter(|_| ms > 200) {
                lift = lift.max(c.position[1] - (0.8 + CARTON_HALF));
            }
        }
        assert!(lift < 0.0005, "B at {speed_b}: lifted {lift} m");
        assert_eq!(w.delivered(0), delivered, "B at {speed_b}");
        if speed_b == 0.0 {
            let front = x(&w, 0) + CARTON_HALF;
            assert!(
                (3.0..3.2).contains(&front),
                "held on the seam: front at {front}"
            );
        }
    }
}

#[test]
fn cartons_accumulate_against_a_stopped_belt_and_all_leave_when_it_starts() {
    let mut w = two_belts(0.0, 5);
    run(&mut w, 15_000);
    let cartons = w.cartons();
    assert_eq!(cartons.len(), 5);
    for pair in cartons.windows(2) {
        let gap = pair[0].position[0] - pair[1].position[0] - 2.0 * CARTON_HALF;
        assert!(
            (-0.005..0.01).contains(&gap),
            "in contact, no overlap: {gap}"
        );
    }
    w.layout.belts[1].speed = 0.5;
    w.set_belt_running(1, true);
    run(&mut w, 15_000);
    assert_eq!(
        (w.delivered(0), w.cartons().len()),
        (5, 0),
        "none lost or stuck"
    );
}

#[test]
fn a_source_adds_a_carton_each_interval_while_its_entry_is_free() {
    let mut w = two_belts(0.5, 3);
    run(&mut w, 1);
    assert_eq!(w.cartons().len(), 1, "the first at 1 ms");
    run(&mut w, 999);
    assert_eq!(w.cartons().len(), 1, "at 1000 ms");
    run(&mut w, 1);
    assert_eq!(
        w.cartons().len(),
        2,
        "the second at 1001 ms, 1 s after the first, the entry free"
    );
    run(&mut w, 5000);
    assert_eq!(w.cartons().len() as u64 + w.delivered(0), 3, "max_count 3");
    let ids: Vec<u64> = w.cartons().iter().map(|c| c.id).collect();
    assert!(ids.iter().all(|id| (1..=3).contains(id)));
}

#[test]
fn a_full_entry_delays_the_next_carton() {
    let mut w = two_belts(0.5, 0);
    w.set_belt_running(0, false);
    run(&mut w, 2000);
    assert_eq!(w.cartons().len(), 1, "the entry stays occupied");
    w.set_belt_running(0, true);
    run(&mut w, 1000);
    assert_eq!(w.cartons().len(), 2, "the next lands once the entry frees");
}

#[test]
fn a_blocked_sink_holds_and_a_free_one_delivers() {
    let mut w = two_belts(0.5, 2);
    assert_eq!((w.sinks(), w.sink_id(0)), (1, "sink"));
    w.set_sink_blocked(0, true);
    assert!(w.sink_blocked(0));
    run(&mut w, 15_000);
    assert_eq!(w.delivered(0), 0);
    let front = x(&w, 0) + CARTON_HALF;
    // The solver lets a resting contact sink by a few micrometres.
    assert!((4.98..5.001).contains(&front), "against the wall: {front}");
    w.set_sink_blocked(0, false);
    w.set_sink_blocked(0, false);
    assert!(!w.sink_blocked(0));
    run(&mut w, 3000);
    assert_eq!(w.delivered(0), 2);
}

#[test]
fn a_closed_exit_holds_products_instead_of_dropping_them() {
    let mut w = World::new(Layout {
        belts: vec![belt("a", 0.0, 3.0, 0.5, true)],
        transfers: vec![],
        stoppers: vec![],
        pushers: vec![],
        turntables: vec![],
        sources: vec![source(0, 1_000_000, 0, 1)],
        sinks: vec![],
        sensors: vec![],
        scanners: vec![],
    });
    assert_eq!(w.sinks(), 0);
    w.set_belt_running(0, true);
    run(&mut w, 10_000);
    let front = x(&w, 0) + CARTON_HALF;
    assert!((2.98..3.001).contains(&front), "{front}");
}

#[test]
fn a_sensor_sees_a_carton_while_it_crosses_the_beam() {
    let mut w = World::new(Layout {
        belts: vec![belt("a", 0.0, 3.0, 0.5, false)],
        transfers: vec![],
        stoppers: vec![],
        pushers: vec![],
        turntables: vec![],
        sources: vec![source(0, 1_000_000, 0, 1)],
        sinks: vec![Sink {
            id: "sink".into(),
            exit: Exit::End(0),
        }],
        sensors: vec![Sensor {
            id: "eye".into(),
            belt: 0,
            at: 2.7,
            kind: SensorKind::Diffuse { range: 0.7 },
        }],
        scanners: vec![],
    });
    w.set_belt_running(0, true);
    let (mut on, mut off) = (None, None);
    for ms in 1..=8000 {
        w.step();
        match ((w.reading(0) == Reading::Detected(true)), on, off) {
            (true, None, _) => on = Some(ms),
            (false, Some(_), None) => off = Some(ms),
            _ => {}
        }
    }
    // The front reaches 2.7 m after about (2.7 - 0.3) / 0.5 = 4.8 s, less the start-up slip; the carton covers the
    // beam for 0.3 m, about 0.6 s, until it is delivered at the belt's end.
    let (on, off) = (on.unwrap(), off.unwrap());
    assert!((4750..4900).contains(&on), "on at {on} ms");
    assert!((on + 550..on + 650).contains(&off), "off at {off} ms");
    let short = Sensor {
        id: "short".into(),
        belt: 0,
        at: 2.7,
        kind: SensorKind::Diffuse { range: 0.1 },
    };
    w.layout.sensors = vec![short];
    assert!(!(w.reading(0) == Reading::Detected(true)));
}

#[test]
fn equal_runs_give_equal_bits_on_every_platform() {
    let fingerprint = || {
        let mut w = two_belts(0.3, 5);
        run(&mut w, 8000);
        w.fingerprint()
    };
    let first = fingerprint();
    assert_eq!(first, fingerprint());
    // Pinned on Linux x86-64 (2026-11-10, Coulomb friction, one solver iteration), equal in dev and release. The
    // windows-2022 CI job runs this test too: with enhanced-determinism the same bits must come out there
    // (research 15).
    // Re-pinned 2026-11-11 for 4.4d: the fingerprint covers each product's size, colour, code and read mark, and the
    // carton's mass is set (2.7 kg) instead of following from its rounded core (1.76 kg).
    assert_eq!(first, 0x6e1f_b275_927f_76a9, "{first:#018x}");
}

pub(super) fn fixture(name: &str) -> twinova_project::Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../project/tests/fixtures/valid")
        .join(name);
    open_folder(&dir).unwrap().opened.project
}

#[test]
fn the_demo_lays_out_its_belt_source_sink_and_sensor() {
    let l = layout(&fixture("demo_m3")).unwrap();
    assert_eq!(l.belts.len(), 1);
    assert_eq!(
        (
            l.belts[0].length,
            l.belts[0].speed,
            l.belts[0].height,
            l.belts[0].closed_exit
        ),
        (3.0, 0.5, 0.8, false)
    );
    assert_eq!(
        l.sources,
        [Source {
            id: "source".into(),
            belt: 0,
            interval_ns: 2_000_000_000,
            first_ns: 10_000_000,
            max_count: 0,
            products: Products::carton(),
        }]
    );
    assert_eq!(
        l.sinks,
        [Sink {
            id: "sink".into(),
            exit: Exit::End(0)
        }]
    );
    assert_eq!(
        l.sensors,
        [Sensor {
            id: "sensor_a".into(),
            belt: 0,
            at: 2.75,
            kind: SensorKind::Diffuse { range: 0.5 },
        }]
    );
}

#[test]
fn a_family_without_a_model_makes_the_line_unrunnable() {
    let mut project = fixture("demo_m3");
    project
        .definitions
        .get_mut("twinova.belt@1")
        .unwrap()
        .generator = "twinova.light_curtain/1".into();
    assert_eq!(
        layout(&project).unwrap_err(),
        "this build cannot run belt_a (twinova.belt): their families have no model yet (ROADMAP 4.4)"
    );
}

#[test]
fn unconnected_ends_are_named() {
    let mut project = fixture("demo_m3");
    project.scene.connections.retain(|c| {
        !matches!(c, twinova_project::model::Connection::Flow { from, .. } if from.0 == "source")
    });
    assert_eq!(layout(&project).unwrap_err(), "source.out is not connected");
    let mut project = fixture("demo_m3");
    project
        .scene
        .connections
        .retain(|c| !matches!(c, twinova_project::model::Connection::Mount { .. }));
    assert_eq!(layout(&project).unwrap_err(), "sensor_a is not mounted");
}

#[test]
fn the_belt_of_a_pair_and_its_side() {
    assert_eq!(surface_of_pair(0, 0), None, "two cartons");
    assert_eq!(surface_of_pair(2, 0), Some((1, false)));
    assert_eq!(surface_of_pair(0, 3), Some((2, true)));
}

#[test]
fn a_wall_closes_the_exit_across_the_belt() {
    let line = Layout {
        belts: vec![belt("a", 1.0, 3.0, 0.5, true)],
        ..Layout::default()
    };
    let wall = exit_wall(&line, Exit::End(0)).unwrap().build();
    let aabb = wall.compute_aabb();
    let (lo, hi) = (aabb.mins, aabb.maxs);
    let near = |a: f32, b: f32| (a - b).abs() < 1e-5;
    assert!(
        near(lo.x, 4.0) && near(hi.x, 4.02),
        "just past the exit: {lo:?} {hi:?}"
    );
    assert!(
        near(lo.y, 0.8) && near(hi.y, 1.2),
        "from the running surface up 0.4 m: {lo:?} {hi:?}"
    );
    assert!(
        near(lo.z, -0.3) && near(hi.z, 0.3),
        "the belt's width: {lo:?} {hi:?}"
    );
}

#[test]
fn a_carton_is_delivered_when_its_front_reaches_the_exit() {
    let mut w = two_belts(0.5, 1);
    let mut last_front = 0.0;
    for _ in 0..20_000 {
        if w.delivered(0) > 0 {
            break;
        }
        if let Some(c) = w.cartons().first() {
            last_front = c.position[0] + CARTON_HALF;
            assert!(c.position[1] > 0.8, "still on the belt");
        }
        w.step();
    }
    assert_eq!(w.delivered(0), 1);
    assert!(
        (4.999..5.0).contains(&last_front),
        "the step before: front at {last_front}"
    );
}

#[test]
fn a_sensor_reaches_as_far_as_its_range() {
    let mut w = World::new(Layout {
        belts: vec![belt("a", 0.0, 3.0, 0.0, false)],
        transfers: vec![],
        stoppers: vec![],
        pushers: vec![],
        turntables: vec![],
        sources: vec![source(0, 1_000_000, 0, 1)],
        sinks: vec![],
        sensors: vec![
            Sensor {
                id: "near".into(),
                belt: 0,
                at: CARTON_HALF,
                kind: SensorKind::Diffuse { range: 0.2 },
            },
            Sensor {
                id: "short".into(),
                belt: 0,
                at: CARTON_HALF,
                kind: SensorKind::Diffuse { range: 0.1 },
            },
        ],
        scanners: vec![],
    });
    run(&mut w, 100);
    // The beam starts at the belt's edge (z = -0.3); the carton spans z = -0.15..0.15.
    assert!(
        (w.reading(0) == Reading::Detected(true)),
        "0.2 m reaches the carton"
    );
    assert!(!(w.reading(1) == Reading::Detected(true)), "0.1 m does not");
}

#[test]
fn a_moved_and_turned_belt_keeps_its_pose_and_each_sensor_its_mount() {
    let mut project = fixture("demo_m3");
    let belt_a = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == "belt_a")
        .unwrap();
    belt_a.position = [1.0, 0.0, 2.0];
    let half = std::f64::consts::FRAC_1_SQRT_2;
    belt_a.rotation = [0.0, half, 0.0, half];
    let mut second = project
        .scene
        .instances
        .iter()
        .find(|i| i.id == "sensor_a")
        .unwrap()
        .clone();
    second.id = "sensor_b".into();
    second.signals.insert("detected".into(), "%IX2.0".into());
    project.scene.instances.push(second);
    project
        .scene
        .connections
        .push(twinova_project::model::Connection::Mount {
            host: ("belt_a".into(), "rail".into()),
            device: ("sensor_b".into(), "mount".into()),
            position: [1.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
        });
    let l = layout(&project).unwrap();
    assert_eq!(l.belts[0].pose.translation, Vector::new(1.0, 0.0, 2.0));
    assert!((l.belts[0].pose.rotation.y - half as f32).abs() < 1e-6);
    let at: Vec<(String, Real)> = l.sensors.iter().map(|s| (s.id.clone(), s.at)).collect();
    assert_eq!(
        at,
        [
            ("sensor_a".to_string(), 2.75),
            ("sensor_b".to_string(), 1.0)
        ]
    );
    assert_eq!((l.belt("belt_a"), l.belt("nothing")), (Some(0), None));
}

/// Rapier orders a pair by collider handle, so in `World::new` a belt, built first, is collider 1. The hook must also
/// carry a carton the right way when the belt is collider 2: here the carton exists before the belt.
#[test]
fn a_belt_carries_the_same_way_whichever_collider_it_is() {
    let mut world = PhysicsWorld::default();
    world.integration_parameters.dt = STEP_S;
    let carton = world.insert_body(
        RigidBodyBuilder::dynamic()
            .translation(Vector::new(0.5, 0.95, 0.0))
            .can_sleep(false),
    );
    world.insert_collider(
        ColliderBuilder::cuboid(0.15, 0.15, 0.15).friction(CARTON_FRICTION),
        Some(carton),
    );
    world.insert_collider(
        ColliderBuilder::cuboid(1.5, 0.05, 0.3)
            .translation(Vector::new(1.5, 0.75, 0.0))
            .friction(BELT_GRIP / CARTON_FRICTION)
            .friction_combine_rule(CoefficientCombineRule::Multiply)
            .user_data(1)
            .active_hooks(ActiveHooks::MODIFY_SOLVER_CONTACTS),
        None,
    );
    let hooks = Surfaces {
        surface: vec![Drive::Linear(Vector::new(0.5, 0.0, 0.0))],
    };
    for _ in 0..1000 {
        world.step_with_events(&hooks, &());
    }
    let v = world.bodies[carton].linvel().x;
    assert!((v - 0.5).abs() < 0.01, "{v}");
}

#[test]
fn a_sink_takes_only_what_reaches_its_own_belt() {
    // Longer than A: its carton passes A's exit line, 0.5 m to the side, and stops at its own exit.
    let mut beside = belt("b", 0.0, 3.5, 0.5, true);
    beside.pose = Pose::from_translation(Vector::new(0.0, 0.0, 0.5));
    let mut w = World::new(Layout {
        belts: vec![belt("a", 0.0, 3.0, 0.0, false), beside],
        transfers: vec![],
        stoppers: vec![],
        pushers: vec![],
        turntables: vec![],
        sources: vec![source(1, 1_000_000, 0, 1)],
        sinks: vec![Sink {
            id: "sink".into(),
            exit: Exit::End(0),
        }],
        sensors: vec![],
        scanners: vec![],
    });
    w.set_belt_running(1, true);
    run(&mut w, 10_000);
    assert_eq!(
        (w.delivered(0), w.cartons().len()),
        (0, 1),
        "held by its own belt's closed exit"
    );
}
