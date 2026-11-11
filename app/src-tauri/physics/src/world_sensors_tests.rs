//! The sensors (spec 2026-11-11-sensors-products-design.md, acceptance 3 to 6). The colour cases are OIP's
//! `colorSensorBehavior.test.ts`, on 0–255 channels.

use super::super::tests::{belt, run, source};
use super::*;
use crate::layout::{Layout, Products, Scanner, Sensor, SensorKind, Source};

const RED: [u8; 3] = [255, 0, 0];
const GREEN: [u8; 3] = [0, 255, 0];
const BLUE: [u8; 3] = [0, 0, 255];

#[test]
fn a_colour_reads_as_the_nearest_listed_one_within_the_tolerance() {
    let rgb = [RED, GREEN, BLUE];
    assert_eq!(classify([250, 10, 5], &rgb, 80), 1, "near red");
    assert_eq!(classify([10, 240, 20], &rgb, 80), 2);
    assert_eq!(classify([0, 0, 255], &rgb, 80), 3);
    assert_eq!(classify([255, 255, 255], &rgb, 80), 0, "white is none");
    assert_eq!(classify([128, 128, 128], &rgb, 80), 0, "grey is none");
    assert_eq!(classify([255, 255, 0], &rgb, 80), 0, "yellow is none");
    assert_eq!(
        classify([255, 0, 0], &[], 80),
        0,
        "an empty list reads none"
    );
    assert_eq!(classify([255, 0, 0], &rgb, 0), 1, "tolerance 0 is exact");
    assert_eq!(classify([254, 0, 0], &rgb, 0), 0);
    assert_eq!(classify([175, 0, 0], &rgb, 80), 1, "the bound is inclusive");
    assert_eq!(classify([174, 0, 0], &rgb, 80), 0);
    // Two listed colours within the tolerance: the nearer wins, the earlier on a tie.
    let close = [[200, 0, 0], [240, 0, 0]];
    assert_eq!(classify([250, 0, 0], &close, 80), 2);
    assert_eq!(classify([220, 0, 0], &close, 80), 1);
    assert_eq!(classify([200, 0, 0], &[[200, 0, 0], [200, 0, 0]], 0), 1);
}

/// A 6 m belt at 0.5 m/s (0.6 m wide, 0.8 m high), a source making `products`, the sensors and scanners given.
fn line(products: Products, sensors: Vec<Sensor>, scanners: Vec<Scanner>) -> World {
    let mut w = World::new(Layout {
        belts: vec![belt("a", 0.0, 6.0, 0.5, true)],
        sources: vec![Source {
            products,
            ..source(0, 1_000_000, 2_000_000_000, 0)
        }],
        sensors,
        scanners,
        ..Layout::default()
    });
    w.set_belt_running(0, true);
    w
}

fn sensor(at: Real, kind: SensorKind) -> Sensor {
    Sensor {
        id: "s".into(),
        belt: 0,
        at,
        kind,
    }
}

/// The steps until `reading` changes, and what it reads then.
fn until_changes(w: &mut World, sensor: usize, limit: usize) -> (usize, Reading) {
    let first = w.reading(sensor);
    for ms in 1..=limit {
        w.step();
        if w.reading(sensor) != first {
            return (ms, w.reading(sensor));
        }
    }
    panic!("{first:?} for {limit} ms")
}

#[test]
fn a_through_beam_sees_a_product_between_its_heads_only() {
    let mut w = line(
        Products::carton(),
        vec![sensor(2.0, SensorKind::ThroughBeam)],
        vec![],
    );
    assert_eq!(w.reading(0), Reading::Detected(false));
    // The carton's front leaves x 0.301 and runs at 0.5 m/s: it reaches the beam at 2.0 m after about 3.4 s.
    let (ms, reading) = until_changes(&mut w, 0, 5000);
    assert_eq!(reading, Reading::Detected(true));
    assert!((3300..3500).contains(&ms), "{ms}");
    let (ms, reading) = until_changes(&mut w, 0, 2000);
    assert_eq!(reading, Reading::Detected(false));
    assert!(
        (550..650).contains(&ms),
        "a 0.3 m carton breaks it for 0.6 s: {ms}"
    );
}

#[test]
fn a_through_beam_reaches_across_the_conveyor_and_no_further() {
    // A product beside the conveyor, past the receiver, is not seen; one at the far edge is.
    let mut w = line(
        Products::carton(),
        vec![sensor(0.15, SensorKind::ThroughBeam)],
        vec![],
    );
    run(&mut w, 2);
    assert_eq!(w.reading(0), Reading::Detected(true));
    let body = w.products[0].body;
    let at = |z: Real| Vector::new(0.15, 0.95, z);
    w.world.bodies[body].set_translation(at(0.35 + 0.15 - 0.01), true);
    assert_eq!(
        w.reading(0),
        Reading::Detected(true),
        "its near face 10 mm inside the receiver's lens"
    );
    w.world.bodies[body].set_translation(at(0.32 + 0.15 + 0.01), true);
    w.world.step_with_events(&w.hooks, &());
    assert_eq!(w.reading(0), Reading::Detected(false), "past the receiver");
}

#[test]
fn a_laser_reads_the_distance_to_the_nearest_face_or_its_range() {
    let mut w = line(
        Products::carton(),
        vec![sensor(0.15, SensorKind::Laser { max_range: 4.0 })],
        vec![],
    );
    assert_eq!(w.reading(0), Reading::Distance(4.0), "clear");
    run(&mut w, 2);
    // The lens is 0.32 m from the centreline, the carton's near face 0.15 m: 0.17 m.
    let Reading::Distance(d) = w.reading(0) else {
        panic!()
    };
    assert!((d - 0.17).abs() < 0.001, "{d}");
    let body = w.products[0].body;
    w.world.bodies[body].set_translation(Vector::new(0.15, 0.95, 0.1), true);
    w.world.step_with_events(&w.hooks, &());
    let Reading::Distance(d) = w.reading(0) else {
        panic!()
    };
    assert!((d - 0.27).abs() < 0.002, "{d}");
}

#[test]
fn a_colour_sensor_reads_the_index_of_the_products_colour() {
    let products = Products {
        colors: vec![GREEN, [255, 255, 255], RED],
        ..Products::carton()
    };
    let color = SensorKind::Color {
        range: 0.5,
        colors: vec![RED, GREEN, BLUE],
        tolerance: 80,
    };
    let mut w = line(products, vec![sensor(2.0, color)], vec![]);
    let mut seen = Vec::new();
    for _ in 0..9000 {
        w.step();
        if let Reading::Color(c) = w.reading(0)
            && seen.last() != Some(&c)
        {
            seen.push(c);
        }
    }
    assert_eq!(
        seen,
        [0, 2, 0, 1, 0],
        "green, white (none), red, each between gaps"
    );
}

#[test]
fn an_unpainted_product_reads_no_colour() {
    let color = SensorKind::Color {
        range: 0.5,
        colors: vec![RED],
        tolerance: 255,
    };
    let mut w = line(Products::carton(), vec![sensor(0.15, color)], vec![]);
    run(&mut w, 2);
    assert_eq!(w.reading(0), Reading::Color(0));
}

/// Steps while `holds`, failing after 20 s of virtual time.
fn step_while(w: &mut World, holds: impl Fn(&World) -> bool) {
    while holds(w) {
        assert!(w.t_ns < 20_000_000_000, "still waiting after 20 s");
        w.step();
    }
}

fn scanner(at: Real) -> Scanner {
    Scanner {
        id: "scan".into(),
        belt: 0,
        at,
        depth: 0.5,
    }
}

#[test]
fn a_scanner_reads_each_product_once_with_its_code() {
    let products = Products {
        codes: vec![41, 42, 43],
        ..Products::carton()
    };
    let mut w = line(products, vec![], vec![scanner(2.0)]);
    let mut reads = Vec::new();
    let mut was = false;
    for ms in 0..8000 {
        w.step();
        let s = w.scanned(0);
        if s.read && !was {
            reads.push((ms, s.count, s.code));
        }
        was = s.read;
    }
    let codes: Vec<_> = reads.iter().map(|r| (r.1, r.2)).collect();
    assert_eq!(codes, [(1, 41), (2, 42), (3, 43)]);
    assert!(w.products.iter().take(3).all(|p| p.read));
}

#[test]
fn the_good_read_lasts_a_quarter_of_a_second_and_the_code_holds() {
    let products = Products {
        codes: vec![7],
        ..Products::carton()
    };
    let mut w = line(products, vec![], vec![scanner(2.0)]);
    step_while(&mut w, |w| !w.scanned(0).read);
    let read_at = w.t_ns;
    step_while(&mut w, |w| w.scanned(0).read);
    assert_eq!(w.t_ns - read_at, 250_000_000);
    assert_eq!(
        w.scanned(0),
        Scan {
            read: false,
            count: 1,
            code: 7
        }
    );
}

#[test]
fn a_product_held_under_the_scanner_is_read_once() {
    let mut w = line(Products::carton(), vec![], vec![scanner(2.0)]);
    step_while(&mut w, |w| w.scanned(0).count == 0);
    w.set_belt_running(0, false);
    run(&mut w, 3000);
    assert_eq!(w.scanned(0).count, 1);
    assert_eq!(w.scanned(0).code, 0, "a product without a code reads 0");
}

#[test]
fn products_closer_than_the_pulse_give_one_long_pulse_and_a_count_for_each() {
    // 0.1 m products every 0.1 s on a belt at 2 m/s: each is read about 0.1 s after the last, within its pulse.
    let mut w = World::new(Layout {
        belts: vec![belt("a", 0.0, 6.0, 2.0, true)],
        sources: vec![Source {
            products: Products {
                size: Vector::splat(0.1),
                ..Products::carton()
            },
            ..source(0, 1_000_000, 100_000_000, 2)
        }],
        scanners: vec![scanner(3.0)],
        ..Layout::default()
    });
    w.set_belt_running(0, true);
    let (mut rises, mut high_ms, mut reads) = (0, 0, Vec::new());
    let (mut was, mut count) = (false, 0);
    for ms in 0..4000 {
        w.step();
        let s = w.scanned(0);
        if s.count != count {
            reads.push(ms);
            count = s.count;
        }
        rises += usize::from(s.read && !was);
        high_ms += usize::from(s.read);
        was = s.read;
    }
    assert_eq!(w.scanned(0).count, 2);
    assert!(reads[1] - reads[0] < 250, "{reads:?}");
    assert_eq!(rises, 1, "one long pulse");
    assert_eq!(
        high_ms,
        reads[1] - reads[0] + 250,
        "from the first read to 250 ms after the second"
    );
}

/// `project` with a definition of `generator` (the diffuse sensor's, renamed, with `parameters`), and an instance of it
/// mounted on `host` at `at`.
fn mount_on(
    project: &mut twinova_project::Project,
    generator: &str,
    parameters: Vec<twinova_project::model::ParameterDef>,
    host: &str,
    at: f64,
) {
    use twinova_project::model::Connection;
    let family = generator.trim_end_matches("/1");
    let mut def = project.definitions["twinova.sensor_diffuse@1"].clone();
    (def.id, def.generator, def.parameters) = (family.into(), generator.into(), parameters);
    let key = def.key();
    project.definitions.insert(key.clone(), def);
    let id = family.trim_start_matches("twinova.").to_string();
    let mut instance = project
        .scene
        .instances
        .iter()
        .find(|i| i.id == "sensor_a")
        .unwrap()
        .clone();
    (instance.id, instance.definition) = (id.clone(), key);
    instance.signals.clear();
    project.scene.instances.push(instance);
    project.scene.connections.push(Connection::Mount {
        host: (host.into(), "rail".into()),
        device: (id, "mount".into()),
        position: [at, 0.0, 0.0],
        rotation: [0.0, 0.0, 0.0, 1.0],
    });
}

fn parameter(
    name: &str,
    ty: twinova_project::model::ParamType,
    default: twinova_project::model::ParamValue,
) -> twinova_project::model::ParameterDef {
    twinova_project::model::ParameterDef {
        name: name.into(),
        ty,
        unit: None,
        default,
        min: None,
        max: None,
    }
}

#[test]
fn the_layout_reads_each_sensor_family_and_the_scanner() {
    use twinova_project::model::{Item, ParamType, ParamValue};
    let mut project = super::super::tests::fixture("curve_line");
    let colors = ParamValue::List(vec![
        Item::Text("#ff0000".into()),
        Item::Text("#00ff7f".into()),
    ]);
    mount_on(
        &mut project,
        "twinova.sensor_through_beam/1",
        vec![],
        "belt_b",
        0.5,
    );
    mount_on(
        &mut project,
        "twinova.sensor_color/1",
        vec![
            parameter("colors", ParamType::ColorList, colors),
            parameter("tolerance", ParamType::DINT, ParamValue::Int(40)),
            parameter("range", ParamType::REAL, ParamValue::Real(0.4)),
        ],
        "curve_a",
        30.0,
    );
    mount_on(
        &mut project,
        "twinova.sensor_laser/1",
        vec![parameter(
            "max_range",
            ParamType::REAL,
            ParamValue::Real(3.0),
        )],
        "belt_c",
        1.0,
    );
    mount_on(
        &mut project,
        "twinova.scanner_barcode/1",
        vec![parameter(
            "read_depth",
            ParamType::REAL,
            ParamValue::Real(0.4),
        )],
        "belt_c",
        1.5,
    );
    let l = crate::layout::layout(&project).unwrap();
    let kinds: Vec<_> = l
        .sensors
        .iter()
        .map(|s| (s.id.as_str(), l.belts[s.belt].id.as_str(), s.at, &s.kind))
        .collect();
    assert_eq!(
        kinds,
        [
            (
                "sensor_a",
                "belt_a",
                2.75,
                &SensorKind::Diffuse { range: 0.5 }
            ),
            (
                "sensor_c",
                "belt_c",
                1.0,
                &SensorKind::Diffuse { range: 0.5 }
            ),
            (
                "sensor_color",
                "curve_a",
                30.0_f32.to_radians(),
                &SensorKind::Color {
                    range: 0.4,
                    colors: vec![RED, [0, 255, 127]],
                    tolerance: 40,
                }
            ),
            (
                "sensor_laser",
                "belt_c",
                1.0,
                &SensorKind::Laser { max_range: 3.0 }
            ),
            (
                "sensor_through_beam",
                "belt_b",
                0.5,
                &SensorKind::ThroughBeam
            ),
        ]
    );
    assert_eq!(
        l.scanners,
        [Scanner {
            id: "scanner_barcode".into(),
            belt: l.belt("belt_c").unwrap(),
            at: 1.5,
            depth: 0.4,
        }]
    );
    // A tolerance past 255 reads 255.
    let color = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == "sensor_color")
        .unwrap();
    color
        .parameters
        .insert("tolerance".into(), ParamValue::Int(300));
    let l = crate::layout::layout(&project).unwrap();
    assert!(matches!(
        l.sensors[2].kind,
        SensorKind::Color { tolerance: 255, .. }
    ));
}

#[test]
fn a_scanner_stands_over_a_straight_conveyor_and_a_sensor_on_a_conveyor() {
    let mut project = super::super::tests::fixture("curve_line");
    mount_on(
        &mut project,
        "twinova.scanner_barcode/1",
        vec![],
        "curve_a",
        30.0,
    );
    assert_eq!(
        crate::layout::layout(&project).unwrap_err(),
        "scanner_barcode is mounted on curve_a, a curve: a barcode scanner stands over a straight conveyor"
    );
    let mut project = super::super::tests::fixture("curve_line");
    mount_on(&mut project, "twinova.sensor_laser/1", vec![], "sink", 0.0);
    assert_eq!(
        crate::layout::layout(&project).unwrap_err(),
        "sensor_laser is mounted on sink, which is not a belt"
    );
}

/// A product is read once its centre is within the read window, half the read depth either side of the scan line, even
/// when its front crossed the line before.
#[test]
fn a_long_product_is_read_when_its_centre_enters_the_window() {
    let long = Products {
        size: Vector::new(0.6, 0.3, 0.3),
        ..Products::carton()
    };
    let mut w = line(
        long,
        vec![],
        vec![Scanner {
            depth: 0.2,
            ..scanner(2.0)
        }],
    );
    step_while(&mut w, |w| w.scanned(0).count == 0);
    let centre = w.cartons()[0].position[0];
    assert!(
        (centre - (2.0 - 0.1)).abs() < 0.002,
        "read with its centre at {centre}"
    );
}
