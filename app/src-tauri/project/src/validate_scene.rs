//! Rules on the scene: identities, overrides, addresses and connections (spec 2026-11-08-project-format-design.md,
//! annex, `scene.json`).

use crate::address::{Address, Area, Size};
use crate::diagnostic::Diagnostic;
use crate::model::{Connection, Definition, Direction, Endpoint, PortKind, Scene, SignalType};
use crate::validate::{duplicates, is_id, ports, pose_problem, value_problem};
use std::collections::{BTreeMap, BTreeSet};

const FILE: &str = "scene.json";

pub(crate) fn scene(scene: &Scene, definitions: &BTreeMap<String, Definition>) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut err = |path: String, message: String| out.push(Diagnostic::new(FILE, path, message));
    let ids = scene
        .instances
        .iter()
        .map(|i| i.id.as_str())
        .chain(scene.groups.iter().map(|g| g.id.as_str()));
    for id in ids.clone().filter(|id| !is_id(id)) {
        err(format!("'{id}'"), "is not a lowercase id".into());
    }
    for id in duplicates(ids) {
        err(
            format!("'{id}'"),
            "is used by two instances or groups".into(),
        );
    }
    if !is_id(&scene.controller.program_instance) {
        err(
            "controller.program_instance".into(),
            "is not a lowercase id".into(),
        );
    }
    let mut bound: Vec<(String, Address)> = Vec::new();
    for inst in &scene.instances {
        let at = |what: &str| format!("instances.{}.{what}", inst.id);
        if let Some(problem) = pose_problem(&inst.position, &inst.rotation) {
            err(at("pose"), problem);
        }
        let Some(def) = definitions.get(&inst.definition) else {
            err(
                at("definition"),
                format!(
                    "'{}' is not a pinned definition of this project",
                    inst.definition
                ),
            );
            continue;
        };
        for (name, value) in &inst.parameters {
            match def.parameters.iter().find(|p| &p.name == name) {
                None => err(
                    at(&format!("parameters.{name}")),
                    format!("{} has no such parameter", def.key()),
                ),
                Some(p) => {
                    if let Some(problem) = value_problem(p.ty, *value, p.min, p.max) {
                        err(at(&format!("parameters.{name}")), problem);
                    }
                }
            }
        }
        for (name, text) in &inst.signals {
            let path = at(&format!("signals.{name}"));
            let Some(signal) = def.signals.iter().find(|s| &s.name == name) else {
                err(path, format!("{} has no such signal", def.key()));
                continue;
            };
            match Address::parse(text) {
                Err(problem) => err(path, problem),
                Ok(address) => {
                    if let Some(problem) = binding_problem(signal.ty, signal.direction, address) {
                        err(path.clone(), problem);
                    }
                    if let Some((other, _)) = bound.iter().find(|(_, a)| a.overlaps(address)) {
                        err(
                            path.clone(),
                            format!("{address} overlaps the address of {other}"),
                        );
                    }
                    bound.push((format!("{}.{name}", inst.id), address));
                }
            }
        }
    }
    let instances: BTreeMap<&str, &Definition> = scene
        .instances
        .iter()
        .filter_map(|i| definitions.get(&i.definition).map(|d| (i.id.as_str(), d)))
        .collect();
    let mut grouped = BTreeSet::new();
    for g in &scene.groups {
        if let Some(problem) = pose_problem(&g.position, &g.rotation) {
            err(format!("groups.{}.pose", g.id), problem);
        }
        for child in &g.children {
            if !scene.instances.iter().any(|i| &i.id == child) {
                err(
                    format!("groups.{}.children", g.id),
                    format!("'{child}' is not an instance"),
                );
            } else if !grouped.insert(child.as_str()) {
                err(
                    format!("groups.{}.children", g.id),
                    format!("'{child}' already belongs to a group"),
                );
            }
        }
    }
    let mut peers: BTreeMap<&Endpoint, usize> = BTreeMap::new();
    for (i, c) in scene.connections.iter().enumerate() {
        let (a, a_kind, b, b_kind) = match c {
            Connection::Flow { from, to } => (from, PortKind::FlowOut, to, PortKind::FlowIn),
            Connection::Mount {
                host,
                device,
                position,
                rotation,
            } => {
                if let Some(problem) = pose_problem(position, rotation) {
                    err(format!("connections[{i}]"), problem);
                }
                (host, PortKind::MountHost, device, PortKind::MountDevice)
            }
        };
        // A host may carry several devices (two sensors on one belt); a flow end and a device have one peer.
        let host = matches!(c, Connection::Mount { .. });
        for (end, kind, single) in [(a, a_kind, !host), (b, b_kind, true)] {
            match instances
                .get(end.0.as_str())
                .map(|d| ports(d).get(end.1.as_str()).copied())
            {
                None => err(
                    format!("connections[{i}]"),
                    format!("'{}' is not an instance", end.0),
                ),
                Some(None) => err(
                    format!("connections[{i}]"),
                    format!("'{}' has no port '{}'", end.0, end.1),
                ),
                Some(Some(k)) if k != kind => err(
                    format!("connections[{i}]"),
                    format!("{}.{} is a {k:?} port, not {kind:?}", end.0, end.1),
                ),
                Some(Some(_)) => {}
            }
            if single {
                *peers.entry(end).or_default() += 1;
            }
        }
    }
    for (end, n) in peers.into_iter().filter(|(_, n)| *n > 1) {
        err(
            "connections".into(),
            format!(
                "{}.{} has {n} peers; each endpoint has at most one",
                end.0, end.1
            ),
        );
    }
    out
}

/// The address must fit the type and the direction: X for BOOL, D for DINT and REAL; %I for a PLC input, %Q for a
/// PLC output. truST accepts a mismatch silently.
fn binding_problem(ty: SignalType, direction: Direction, address: Address) -> Option<String> {
    let size = match ty {
        SignalType::BOOL => Size::Bit,
        SignalType::DINT | SignalType::REAL => Size::Double,
    };
    let area = match direction {
        Direction::PlcInput => Area::Input,
        Direction::PlcOutput => Area::Output,
    };
    if address.size != size {
        Some(format!(
            "a {ty:?} signal needs a {size:?} address, not {address}"
        ))
    } else if address.area != area {
        Some(format!(
            "a {direction:?} signal needs a {area:?} address, not {address}"
        ))
    } else {
        None
    }
}
