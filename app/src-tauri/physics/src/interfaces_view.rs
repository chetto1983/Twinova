//! Each instance's interfaces as the webview and the agent receive them (spec 2026-11-11-snapping-design.md, "The view"): its ends
//! and rail in its own frame, which the view composes with the pose it draws the instance at; a mounted device's side
//! exit in its host's frame (`on`).

use std::collections::BTreeMap;

use glamx::DPose3;
use serde::Serialize;
use twinova_project::Project;

use crate::interfaces::{Direction, Interfaces, Rail, interfaces};

/// A frame: a point and a unit quaternion `[x, y, z, w]`, its x pointing out.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FrameView {
    pub position: [f64; 3],
    pub rotation: [f64; 4],
}

impl From<DPose3> for FrameView {
    fn from(p: DPose3) -> Self {
        let q = p.rotation;
        Self {
            position: p.translation.to_array(),
            rotation: [q.x, q.y, q.z, q.w],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectionView {
    In,
    Out,
    Either,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EndView {
    pub name: String,
    pub direction: DirectionView,
    pub frame: FrameView,
    /// The host whose frame a mounted device's side exit is in.
    pub on: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RailView {
    Line {
        start: FrameView,
        length: f64,
    },
    Arc {
        centre: [f64; 3],
        start: [f64; 3],
        sweep: f64,
        right: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HostView {
    pub rail: RailView,
    pub width: f64,
    pub accepts: Vec<String>,
    pub spurs: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InterfacesView {
    pub ends: Vec<EndView>,
    pub host: Option<HostView>,
}

fn view_of(i: Interfaces) -> InterfacesView {
    InterfacesView {
        ends: (i.ends.into_iter())
            .map(|e| EndView {
                name: e.name.into(),
                direction: match e.direction {
                    Direction::In => DirectionView::In,
                    Direction::Out => DirectionView::Out,
                    Direction::Either => DirectionView::Either,
                },
                frame: e.frame.into(),
                on: e.on,
            })
            .collect(),
        host: i.host.map(|h| HostView {
            rail: match h.rail {
                Rail::Line { start, length } => RailView::Line {
                    start: start.into(),
                    length,
                },
                Rail::Arc {
                    centre,
                    start,
                    sweep,
                    right,
                } => RailView::Arc {
                    centre: centre.to_array(),
                    start: start.to_array(),
                    sweep,
                    right,
                },
            },
            width: h.width,
            accepts: h.accepts.into_iter().map(String::from).collect(),
            spurs: h.spurs,
        }),
    }
}

/// Every instance's interfaces, by id.
pub fn interfaces_view(project: &Project) -> BTreeMap<String, InterfacesView> {
    (interfaces(project).into_iter())
        .map(|(id, i)| (id, view_of(i)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::tests::fixture;

    /// The view carries every instance's ends and rail, as the interfaces give them, in plain numbers.
    #[test]
    fn the_view_carries_each_instances_ends_and_rail() {
        let all = interfaces_view(&fixture("sort_line"));
        assert_eq!(all.len(), fixture("sort_line").scene.instances.len());
        let t = &all["turntable"];
        let names: Vec<(&str, &DirectionView)> = t
            .ends
            .iter()
            .map(|e| (e.name.as_str(), &e.direction))
            .collect();
        assert_eq!(
            names,
            [
                ("in", &DirectionView::In),
                ("out", &DirectionView::Out),
                ("left", &DirectionView::Either),
                ("right", &DirectionView::Either)
            ]
        );
        let host = t.host.as_ref().unwrap();
        assert!((host.width - 0.6).abs() < 1e-6 && !host.spurs);
        let RailView::Line { start, length } = &host.rail else {
            panic!("{:?}", host.rail)
        };
        assert!((length - 0.9).abs() < 1e-6 && (start.position[0] + 0.45).abs() < 1e-6);
        assert_eq!(all["pusher"].ends[0].on.as_deref(), Some("belt_b"));
        let curves = interfaces_view(&fixture("curve_line"));
        let RailView::Arc {
            centre,
            sweep,
            right,
            ..
        } = &curves["curve_b"].host.as_ref().unwrap().rail
        else {
            panic!()
        };
        assert!(
            *right
                && (sweep - std::f64::consts::FRAC_PI_2).abs() < 1e-6
                && (centre[2] - 0.9).abs() < 1e-6
        );
        let exit = &curves["belt_a"].ends[1].frame;
        assert!((exit.position[0] - 3.085).abs() < 1e-6 && (exit.rotation[3] - 1.0).abs() < 1e-6);
    }
}
