//! One conveyor, from its family and parameters: its pose, size, guards and arc. The line's layout and the editor's
//! interfaces both build their conveyors here (spec 2026-11-11-snapping-design.md, "One geometry").

use rapier3d::prelude::{Pose, Real, Vector};
use twinova_project::model::{Definition, Instance};

use super::{Arc, BELT_FACE, BELT_GRIP, Belt, TURNTABLE_LANE, degrees, inclined, param, pose};
use crate::path::arc_length;

/// The conveyor `inst` is, with its exit closed (the layout opens it where a flow leaves it), or none if its family
/// is not a conveyor.
pub(crate) fn conveyor(inst: &Instance, def: &Definition) -> Option<Belt> {
    let real = |name| param(inst, def, name) as Real;
    let (width, height) = (real("width"), real("height"));
    let (grip, arc) = match def.generator.as_str() {
        "twinova.belt/1" => (BELT_GRIP, None),
        "twinova.roller/1" | "twinova.turntable/1" => (real("friction"), None),
        "twinova.belt_curve/1" | "twinova.roller_curve/1" => {
            let arc = Arc {
                inner_radius: real("inner_radius"),
                sweep: degrees(inst, def, "sweep"),
                right: real("turn_right") != 0.0,
            };
            (real("friction"), Some(arc))
        }
        _ => return None,
    };
    let turntable = def.generator == "twinova.turntable/1";
    let (pose, length) = if turntable {
        let half = TURNTABLE_LANE * width / 2.0;
        (
            pose(inst) * Pose::from_translation(Vector::new(-half, 0.0, 0.0)),
            2.0 * half,
        )
    } else {
        let length = arc.map_or_else(|| real("length"), |a| arc_length(&a, width));
        (
            inclined(pose(inst), length, height, degrees(inst, def, "incline")),
            length,
        )
    };
    Some(Belt {
        id: inst.id.clone(),
        pose,
        length,
        width,
        height,
        speed: real("speed"),
        grip,
        // A turntable's ends have no wall: its deck turns them (OIP has none).
        closed_exit: !turntable,
        arc,
        guards: [real("left_guard") != 0.0, real("right_guard") != 0.0],
        openings: Vec::new(),
        skew: degrees(inst, def, "skew"),
        turntable,
        face: if def.generator == "twinova.belt/1" {
            BELT_FACE
        } else {
            0.0
        },
    })
}
