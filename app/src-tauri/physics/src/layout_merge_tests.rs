use rapier3d::prelude::{Pose, Rotation, Vector};
use twinova_project::Project;
use twinova_project::model::{Connection, ParamValue, Side};

use super::meeting;
use crate::World;
use crate::layout::{Opening, layout};
use crate::world::tests::fixture;

fn instance<'a>(p: &'a mut Project, id: &str) -> &'a mut twinova_project::model::Instance {
    (p.scene.instances.iter_mut()).find(|i| i.id == id).unwrap()
}

fn merge(p: &mut Project) -> (&mut f64, &mut Side) {
    (p.scene.connections.iter_mut())
        .find_map(|c| match c {
            Connection::Merge { at, side, .. } => Some((at, side)),
            _ => None,
        })
        .unwrap()
}

/// The fixture's spur (written by a script, not by this code) meets the main belt's right flank square at 2 m: the
/// main belt's right guard opens over the spur's width there and the spur's exit is open.
#[test]
fn a_merge_opens_its_hosts_guard_and_its_spurs_exit() {
    let l = layout(&fixture("merge_line")).unwrap();
    let (host, spur) = (
        &l.belts[l.belt("belt_a").unwrap()],
        &l.belts[l.belt("belt_s").unwrap()],
    );
    assert_eq!(
        host.openings,
        [Opening {
            right: true,
            centre: 2.0,
            length: 0.6
        }]
    );
    assert!(!spur.closed_exit && host.guards == [true; 2]);
}

/// The throwaway probe's line, built through `merge` (spec acceptance 4): every carton the spur feeds reaches the main
/// belt's sink, upright; their turn on the main belt is measured, not bounded.
#[test]
fn cartons_merge_through_the_opening_upright() {
    let mut w = World::new(layout(&fixture("merge_line")).unwrap());
    for k in 0..w.belts() {
        w.set_belt_running(k, true);
    }
    let (mut tilt, mut turn): (f32, f32) = (0.0, 0.0);
    for _ in 0..20_000 {
        w.step();
        for c in w.cartons() {
            let r = Rotation::from_xyzw(c.rotation[0], c.rotation[1], c.rotation[2], c.rotation[3]);
            tilt = tilt.max((r * Vector::Y).angle_between(Vector::Y).to_degrees());
            if c.position[0] > 3.0 {
                let x = r * Vector::X;
                turn =
                    turn.max(((x.z.atan2(x.x).to_degrees() + 45.0).rem_euclid(90.0) - 45.0).abs());
            }
        }
    }
    println!(
        "merge_line: {} of {} delivered, tilt {tilt:.2}°, turn {turn:.1}°",
        w.delivered(0),
        w.created()
    );
    // A carton every 2 s takes about 8.5 s from the source to the sink: the rest are still on the decks, none fell.
    assert!(w.delivered(0) >= 6, "{} of {}", w.delivered(0), w.created());
    assert_eq!(w.created(), w.delivered(0) + w.cartons().len() as u64);
    assert!(
        w.cartons().iter().all(|c| c.position[1] > 0.8),
        "on the decks"
    );
    assert!(tilt < 1.0, "tilted {tilt}°");
}

/// A spur may stand off the flank up to OIP's 0.15 m contact tolerance.
#[test]
fn a_spur_may_stand_off_the_flank_within_reach() {
    let mut p = fixture("merge_line");
    instance(&mut p, "belt_s").position[2] += 0.15;
    assert!(layout(&p).is_ok());
    instance(&mut p, "belt_s").position[2] += 0.002;
    let error = layout(&p).unwrap_err();
    assert!(
        error.contains("0.152 m off the flank; a merge reaches 0.15 m"),
        "{error}"
    );
}

#[test]
fn a_merge_that_does_not_meet_its_flank_is_refused() {
    let refuse = |change: &dyn Fn(&mut Project), message: &str| {
        let mut p = fixture("merge_line");
        change(&mut p);
        let error = layout(&p).unwrap_err();
        assert!(error.starts_with("belt_s merges onto belt_a"), "{error}");
        assert!(error.contains(message), "{error}");
    };
    refuse(
        &|p| {
            // The fixture's quarter turn and 0.2° more, about the exit face's centre, which stays at 2 m on the flank.
            let turn = Rotation::from_rotation_y((90.2_f32).to_radians());
            let origin = Vector::new(2.0, 0.0, 0.3) - turn * Vector::new(2.085, 0.0, 0.0);
            let s = instance(p, "belt_s");
            s.rotation = [turn.x, turn.y, turn.z, turn.w].map(f64::from);
            s.position = [origin.x, 0.0, origin.z].map(f64::from);
        },
        "° from square to the flank",
    );
    refuse(
        &|p| instance(p, "belt_s").position[0] += 0.01,
        "its exit is at 2.010 m along the run, not 2 m",
    );
    refuse(
        &|p| instance(p, "belt_s").position[2] -= 0.01,
        "its exit is -0.010 m off the flank",
    );
    refuse(
        &|p| *merge(p).1 = Side::Left,
        "its exit is -0.600 m off the flank",
    );
    refuse(&|p| *merge(p).0 = 4.5, "at 4.5 m, outside its run of 4 m");
    let host = |name: &str, value: ParamValue| {
        let name = name.to_string();
        move |p: &mut Project| {
            instance(p, "belt_a")
                .parameters
                .insert(name.clone(), value.clone());
        }
    };
    refuse(
        &host("incline", ParamValue::Real(5.0)),
        "which is not level and square: a spur fits",
    );
    refuse(
        &|p| {
            twinova_project::edit::pin(p, "twinova.belt_curve@1").unwrap();
            instance(p, "belt_a").definition = "twinova.belt_curve@1".into();
        },
        "which is not a straight conveyor",
    );
}

#[test]
fn a_merge_from_what_is_not_a_conveyor_is_refused() {
    let mut p = fixture("merge_line");
    for c in &mut p.scene.connections {
        if let Connection::Merge { from, .. } = c {
            from.0 = "source".into();
        }
        if let Connection::Flow { from, .. } = c
            && from.0 == "source"
        {
            from.0 = "belt_s".into();
        }
    }
    let error = layout(&p).unwrap_err();
    assert!(
        error.contains("source merges onto belt_a, but is not a conveyor"),
        "{error}"
    );
}

/// Two spurs on one side of a host may not overlap; on its two sides they may face each other.
#[test]
fn spurs_on_one_side_of_a_host_do_not_overlap() {
    let mut p = fixture("merge_line");
    let mut second = p
        .scene
        .instances
        .iter()
        .find(|i| i.id == "belt_s")
        .unwrap()
        .clone();
    second.id = "belt_t".into();
    second.position[0] = 2.5;
    p.scene.instances.push(second);
    p.scene.connections.push(Connection::Merge {
        from: ("belt_t".into(), "out".into()),
        host: ("belt_a".into(), "rail".into()),
        at: 2.5,
        side: Side::Right,
    });
    let error = layout(&p).unwrap_err();
    assert!(
        error.contains("belt_t merges onto belt_a over belt_s's merge on its Right side"),
        "{error}"
    );
    let t = instance(&mut p, "belt_t");
    t.position = [2.7, 0.0, 2.385];
    let c = p.scene.connections.last_mut().unwrap();
    if let Connection::Merge { at, .. } = c {
        *at = 2.7;
    }
    let l = layout(&p).unwrap();
    assert_eq!(l.belts[l.belt("belt_a").unwrap()].openings.len(), 2);
    let pose = Pose::from_parts(
        Vector::new(2.0, 0.0, -2.385),
        Rotation::from_rotation_y(-std::f32::consts::FRAC_PI_2),
    );
    let t = instance(&mut p, "belt_t");
    t.position = [pose.translation.x, 0.0, pose.translation.z].map(f64::from);
    t.rotation = [
        pose.rotation.x,
        pose.rotation.y,
        pose.rotation.z,
        pose.rotation.w,
    ]
    .map(f64::from);
    if let Connection::Merge { at, side, .. } = p.scene.connections.last_mut().unwrap() {
        (*at, *side) = (2.0, Side::Left);
    }
    assert!(layout(&p).is_ok(), "{:?}", layout(&p).err());
}

/// The tolerances are inclusive: an exit exactly 1 mm from its `at` still meets the flank. The exit is built by hand,
/// a frame with no turn, so its position is exact and the next check, squareness, is the one that refuses it.
#[test]
fn a_merge_meets_its_flank_within_a_millimetre_of_its_distance() {
    let host = crate::world::tests::belt("h", 0.0, 4.0, 0.5, false);
    let exit = |x: f32| Pose::from_translation(Vector::new(x, 0.8, 0.3));
    let error = meeting(&host, exit(0.001), 0.0, true).unwrap();
    assert!(error.contains("from square"), "{error}");
    let error = meeting(&host, exit(0.0011), 0.0, true).unwrap();
    assert!(error.contains("along the run"), "{error}");
}

/// Two spurs whose openings touch end to end do not overlap.
#[test]
fn spurs_that_touch_do_not_overlap() {
    let mut p = fixture("merge_line");
    instance(&mut p, "belt_s")
        .parameters
        .insert("width".into(), ParamValue::Real(0.5));
    let mut second = p
        .scene
        .instances
        .iter()
        .find(|i| i.id == "belt_s")
        .unwrap()
        .clone();
    second.id = "belt_t".into();
    second.position[0] = 2.5;
    p.scene.instances.push(second);
    p.scene.connections.push(Connection::Merge {
        from: ("belt_t".into(), "out".into()),
        host: ("belt_a".into(), "rail".into()),
        at: 2.5,
        side: Side::Right,
    });
    let l = layout(&p).unwrap();
    assert_eq!(l.belts[l.belt("belt_a").unwrap()].openings.len(), 2);
}
