//! Roller conveyors and OIP's chain transfer (spec 2026-11-10-diverter-line-design.md). The fidelity checks follow
//! OIP's `src/sim/physicsFidelity.test.ts` at `1e7488c`: a surface carries at its speed, and a lift throws its load
//! v²/2g.

use super::tests::{belt, fixture, run, source};
use super::*;
use crate::layout::{Belt, Exit, Sink, Transfer, layout};

const ROLLER_GRIP: Real = 0.42;
/// OIP's default `raiseTime`, 0.15 s, as the chain transfer's block slews it: one millisecond of stroke per step.
const RAISE_MS: usize = 150;

fn roller(x: Real) -> Belt {
    Belt {
        grip: ROLLER_GRIP,
        ..belt("r", x, 2.0, 0.5, false)
    }
}

/// Belt A (3 m), roller R (2 m) with a transfer at its centre, belt B (2 m) into the normal sink, and the transfer's
/// side into the reject sink when `side_sink`; one carton every `interval_ms`.
fn line(side_sink: bool, cartons: i64, interval_ms: i64) -> World {
    let mut sinks = vec![Sink {
        id: "ok".into(),
        exit: Exit::End(2),
    }];
    if side_sink {
        sinks.push(Sink {
            id: "reject".into(),
            exit: Exit::Side(0),
        });
    }
    let mut w = World::new(Layout {
        belts: vec![
            belt("a", 0.0, 3.0, 0.5, false),
            roller(3.0),
            belt("b", 5.0, 2.0, 0.5, false),
        ],
        transfers: vec![Transfer {
            id: "t".into(),
            host: 1,
            at: 1.0,
            along: 0.66,
            across: 0.6,
            speed: 0.5,
            out_right: false,
            closed_exit: !side_sink,
        }],
        stoppers: vec![],
        pushers: vec![],
        turntables: vec![],
        sources: vec![source(0, 1_000_000, interval_ms * 1_000_000, cartons)],
        sinks,
        sensors: vec![],
        scanners: vec![],
    });
    for k in 0..3 {
        w.set_belt_running(k, true);
    }
    w.set_transfer(0, 0.0, false);
    w
}

/// The chain transfer's block, as Task 3's ST: `raised` at once, the lift one millisecond of stroke per step.
struct Lift {
    travel: usize,
}

impl Lift {
    fn step(&mut self, w: &mut World, raised: bool) {
        self.travel = if raised {
            (self.travel + 1).min(RAISE_MS)
        } else {
            self.travel.saturating_sub(1)
        };
        w.set_transfer(0, self.travel as Real / RAISE_MS as Real, raised);
        w.step();
    }
}

fn yaw_deg(c: &Carton) -> f32 {
    let [x, y, z, w] = c.rotation;
    (2.0 * (w * y + x * z))
        .atan2(1.0 - 2.0 * (y * y + z * z))
        .to_degrees()
}

/// Runs until the first carton's centre reaches `x`, within a minute of virtual time.
fn until_x(w: &mut World, x: f32) {
    for _ in 0..60_000 {
        if w.cartons().first().is_some_and(|c| c.position[0] >= x) {
            return;
        }
        w.step();
    }
    panic!("no carton reached x = {x}");
}

#[test]
fn a_roller_conveyor_carries_at_its_surface_speed() {
    let mut w = World::new(Layout {
        belts: vec![Belt {
            grip: ROLLER_GRIP,
            ..belt("r", 0.0, 4.0, 0.5, true)
        }],
        sources: vec![source(0, 1_000_000, 1_000_000_000_000, 1)],
        ..Layout::default()
    });
    w.set_belt_running(0, true);
    run(&mut w, 2000);
    let v = w.world.bodies[w.products[0].body].linvel();
    assert!((v.x - 0.5).abs() < 0.005 && v.z.abs() < 1e-3, "{v:?}");
}

#[test]
fn a_lowered_transfer_leaves_cartons_to_the_rollers() {
    let mut w = line(true, 3, 2000);
    run(&mut w, 20_000);
    assert_eq!((w.delivered(0), w.delivered(1)), (3, 0));
}

#[test]
fn a_raised_transfer_throws_its_load_v2_2g_and_carries_it_out_square() {
    let mut w = line(true, 1, 2000);
    let mut lift = Lift { travel: 0 };
    until_x(&mut w, 3.85);
    let (mut peak, mut yaw, mut side) = (0.0f32, 0.0f32, 0.0f32);
    for _ in 0..3000 {
        lift.step(&mut w, true);
        if let Some(c) = w.cartons().first() {
            peak = peak.max(c.position[1]);
            yaw = yaw.max(yaw_deg(c).abs());
            side = side.min(c.position[2]);
        }
    }
    // A transfer that does not carry to the right carries to the host's left, −z: the sink takes the carton when its
    // front crosses the left edge, its centre half a carton in.
    assert!(side < -0.1, "carried left: {side}");
    // At rest on the raised deck the carton's centre is at 0.8 − 0.03 + 0.095 + 0.15.
    let rest = 0.8 - TRANSFER_BELOW + TRANSFER_STROKE + CARTON_HALF;
    let v = TRANSFER_STROKE / (RAISE_MS as Real / 1000.0);
    let throw = v * v / (2.0 * 9.81);
    assert!(
        (peak - rest - throw).abs() < 0.004,
        "thrown {:.4} m, v²/2g is {throw:.4} m",
        peak - rest
    );
    assert!(yaw < 1.0, "{yaw}°");
    assert_eq!((w.delivered(0), w.delivered(1)), (0, 1));
}

#[test]
fn an_unconnected_side_exit_holds_what_the_transfer_carries() {
    let mut w = line(false, 1, 2000);
    let mut lift = Lift { travel: 0 };
    until_x(&mut w, 3.85);
    for _ in 0..4000 {
        lift.step(&mut w, true);
    }
    let c = &w.cartons()[0];
    assert!(
        c.position[2] > -0.3 - CARTON_HALF - 0.03 && c.position[1] > 0.9,
        "held on the deck against the wall: {:?}",
        c.position
    );
}

#[test]
fn a_blocked_side_sink_holds_and_then_takes_the_carton() {
    let mut w = line(true, 1, 2000);
    w.set_sink_blocked(1, true);
    let mut lift = Lift { travel: 0 };
    until_x(&mut w, 3.85);
    for _ in 0..4000 {
        lift.step(&mut w, true);
    }
    assert_eq!((w.delivered(1), w.cartons().len()), (0, 1));
    w.set_sink_blocked(1, false);
    // At rest against the wall, the carton's front is on the edge; the chains push it over once the wall is gone.
    for _ in 0..500 {
        lift.step(&mut w, true);
    }
    assert_eq!((w.delivered(1), w.cartons().len()), (1, 0));
}

#[test]
fn no_carton_is_lost_or_duplicated_while_every_third_is_diverted() {
    let mut w = line(true, 0, 2000);
    let mut lift = Lift { travel: 0 };
    let (mut seen, mut raised, mut over) = (0u64, false, false);
    for _ in 0..60_000 {
        // A beam across the deck's centre, as the line's photocell: raise for every third carton, lower once it is
        // clear of the beam and the chains have topped out.
        let now = w
            .cartons()
            .iter()
            .any(|c| (c.position[0] - 4.0).abs() < CARTON_HALF && c.position[2] > -0.3);
        if now && !over {
            seen += 1;
            raised |= seen % 3 == 0;
        }
        if raised && !now && lift.travel == RAISE_MS {
            raised = false;
        }
        over = now;
        lift.step(&mut w, raised);
        let made = w.sources[0].made as u64;
        assert_eq!(w.created(), made);
        assert_eq!(
            made,
            w.delivered(0) + w.delivered(1) + w.cartons().len() as u64
        );
    }
    assert!(
        w.delivered(1) >= 9 && w.delivered(0) >= 2 * w.delivered(1) - 2,
        "{} / {}",
        w.delivered(0),
        w.delivered(1)
    );
}

#[test]
fn a_side_wall_runs_along_the_deck_and_a_carton_past_it() {
    let mut l = Layout {
        belts: vec![roller(3.0)],
        ..Layout::default()
    };
    l.transfers.push(Transfer {
        id: "t".into(),
        host: 0,
        at: 1.0,
        along: 0.66,
        across: 0.6,
        speed: 0.5,
        out_right: true,
        closed_exit: true,
    });
    let aabb = exit_wall(&l, Exit::Side(0)).unwrap().build().compute_aabb();
    let near = |a: f32, b: f32| (a - b).abs() < 1e-5;
    let (lo, hi) = (aabb.mins, aabb.maxs);
    assert!(
        near(lo.x, 3.52) && near(hi.x, 4.48),
        "along the deck and a carton past it: {lo:?} {hi:?}"
    );
    assert!(
        near(lo.z, 0.3) && near(hi.z, 0.32),
        "past the right edge: {lo:?} {hi:?}"
    );
    assert!(near(lo.y, 0.8), "from the rollers up: {lo:?}");
}

#[test]
fn the_line_fixture_lays_out_its_transfer_in_the_roller_and_its_two_exits() {
    let l = layout(&fixture("two_belts_diverter")).unwrap();
    let ids: Vec<&str> = l.belts.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(ids, ["belt_a", "belt_b", "roller_r"]);
    assert_eq!((l.belts[0].grip, l.belts[2].grip), (0.7, 0.42));
    assert_eq!(
        l.transfers,
        [Transfer {
            id: "chain_transfer".into(),
            host: 2,
            at: 1.0,
            along: 0.66,
            across: 0.6,
            speed: 0.5,
            out_right: false,
            closed_exit: false,
        }]
    );
    let exits: Vec<(&str, Exit)> = l.sinks.iter().map(|s| (s.id.as_str(), s.exit)).collect();
    assert_eq!(
        exits,
        [("sink_ok", Exit::End(1)), ("sink_reject", Exit::Side(0))]
    );
    assert_eq!((l.sensors[0].belt, l.sensors[0].at), (2, 1.0));
}

#[test]
fn a_transfer_needs_a_roller_conveyor_to_fit_in() {
    use twinova_project::model::Connection;
    let mut project = fixture("two_belts_diverter");
    for c in &mut project.scene.connections {
        if let Connection::Mount { host, device, .. } = c
            && device.0 == "chain_transfer"
        {
            host.0 = "belt_a".into();
        }
    }
    assert_eq!(
        layout(&project).unwrap_err(),
        "chain_transfer is mounted on belt_a, which is not a roller conveyor"
    );
    project
        .scene
        .connections
        .retain(|c| !matches!(c, Connection::Mount { device, .. } if device.0 == "chain_transfer"));
    assert_eq!(
        layout(&project).unwrap_err(),
        "chain_transfer is not mounted"
    );
}

#[test]
fn a_sink_must_take_from_a_conveyor_a_transfer_or_a_pusher() {
    use twinova_project::model::Connection;
    let mut project = fixture("two_belts_diverter");
    for c in &mut project.scene.connections {
        if let Connection::Flow { from, to } = c
            && to.0 == "sink_reject"
        {
            from.0 = "source".into();
        }
    }
    assert_eq!(
        layout(&project).unwrap_err(),
        "sink_reject.in is connected to source, which is not a conveyor, a chain transfer or a pusher"
    );
}

#[test]
fn a_line_with_a_transfer_gives_equal_bits_on_every_platform() {
    let fingerprint = || {
        let mut w = line(true, 3, 1500);
        let mut lift = Lift { travel: 0 };
        // Cartons reach the deck at about 7.7, 9.2 and 10.7 s: the first two arrive while it is raised and leave by
        // the side; the third passes on the rollers and is still on the line at 12 s.
        for ms in 0..12_000 {
            lift.step(&mut w, (7_500..9_500).contains(&ms));
        }
        assert_eq!((w.delivered(1), w.cartons().len()), (2, 1));
        w.fingerprint()
    };
    let first = fingerprint();
    assert_eq!(first, fingerprint());
    // Pinned on Linux x86-64 (2026-11-10, Coulomb friction, one solver iteration), equal in dev and release; the
    // windows-2022 CI job runs this test too.
    // Re-pinned 2026-11-11 for 4.4d: the fingerprint covers each product's size, colour, code and read mark, and the
    // carton's mass is set (2.7 kg) instead of following from its rounded core (1.76 kg).
    assert_eq!(first, 0x021c_4252_434d_2f94, "{first:#018x}");
}

/// Two transfers on one roller conveyor, the second carrying to the right: only the raised one takes the carton, at
/// its own speed, and each answers to its own index.
#[test]
fn each_transfer_answers_for_its_own_deck_and_side() {
    let transfer = |id: &str, at: Real, out_right: bool| Transfer {
        id: id.into(),
        host: 0,
        at,
        along: 0.66,
        across: 0.6,
        speed: 0.5,
        out_right,
        closed_exit: false,
    };
    let mut w = World::new(Layout {
        belts: vec![Belt {
            grip: ROLLER_GRIP,
            ..belt("r", 0.0, 3.0, 0.5, false)
        }],
        transfers: vec![transfer("first", 0.8, false), transfer("second", 2.0, true)],
        stoppers: vec![],
        pushers: vec![],
        turntables: vec![],
        sources: vec![source(0, 1_000_000, 1_000_000_000_000, 1)],
        sinks: vec![
            Sink {
                id: "left".into(),
                exit: Exit::Side(0),
            },
            Sink {
                id: "right".into(),
                exit: Exit::Side(1),
            },
        ],
        sensors: vec![],
        scanners: vec![],
    });
    assert_eq!((w.transfers(), w.transfer_id(1)), (2, "second"));
    w.set_transfer(1, 0.25, true);
    assert_eq!((w.transfer_lift(0), w.transfer_lift(1)), (0.0, 0.25));
    w.set_belt_running(0, true);
    w.set_transfer(0, 0.0, false);
    until_x(&mut w, 1.85);
    let mut travel = 0;
    // The carry speed once the carton has settled to it, two thirds across (its front leaves at z = 0.15): it climbs
    // through the lift's throw and overshoots to 0.51 before settling (measured 2026-11-10, Coulomb friction).
    let mut lateral = None;
    for _ in 0..3000 {
        travel = (travel + 1).min(RAISE_MS);
        w.set_transfer(1, travel as Real / RAISE_MS as Real, true);
        w.step();
        if let Some(c) = w.products.first() {
            let body = &w.world.bodies[c.body];
            if lateral.is_none() && body.translation().z >= 0.1 {
                lateral = Some(body.linvel().z);
            }
        }
    }
    let lateral = lateral.expect("carried right");
    assert_eq!((w.delivered(0), w.delivered(1)), (0, 1));
    assert!(
        (lateral - 0.5).abs() < 0.01,
        "carried right at the transfer's speed: {lateral}"
    );
}
