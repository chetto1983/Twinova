//! Edits of a working project, shared by the app's editor and the agent (specs 2026-11-11-agent-server-design.md,
//! decision 4, and 2026-11-11-editor-basics-design.md). Each checks the names it is given, the instance, definition,
//! parameter, signal or file, and the value's type against the parameter's, then changes the project; the format's own
//! checks (ports, addresses, ranges, ST) run at Build, as for a project edited on disk.

use serde_json::Value;

use crate::Project;
use crate::library;
use crate::model::{
    Connection, Definition, Endpoint, Instance, ParamType, ParamValue, PortKind, Quat, Vec3,
};

fn instance_mut<'a>(project: &'a mut Project, id: &str) -> Result<&'a mut Instance, String> {
    (project.scene.instances.iter_mut())
        .find(|i| i.id == id)
        .ok_or_else(|| format!("no instance {id}"))
}

fn definition<'a>(project: &'a Project, key: &str) -> Result<&'a Definition, String> {
    project
        .definitions
        .get(key)
        .ok_or_else(|| format!("no definition {key}"))
}

/// The definition of instance `id`.
fn definition_of(project: &Project, id: &str) -> Result<Definition, String> {
    let instance = (project.scene.instances.iter()).find(|i| i.id == id);
    let key = &instance
        .ok_or_else(|| format!("no instance {id}"))?
        .definition;
    definition(project, key).cloned()
}

/// `value` as a parameter of type `ty`: a REAL from any number, a DINT or a TIME (ns) from an integer, a BOOL from a
/// boolean, a list from an array of integers or texts. The format checks ranges and colours at Build.
fn param_value(ty: ParamType, value: &Value) -> Option<ParamValue> {
    match (ty, value) {
        (ParamType::REAL, Value::Number(n)) => n.as_f64().map(ParamValue::Real),
        (ParamType::DINT | ParamType::TIME, Value::Number(n)) => n.as_i64().map(ParamValue::Int),
        (ParamType::BOOL, Value::Bool(b)) => Some(ParamValue::Bool(*b)),
        (ParamType::DintList | ParamType::ColorList, Value::Array(_)) => {
            serde_json::from_value(value.clone()).ok()
        }
        _ => None,
    }
}

/// Sets a parameter's value on an instance, or with `None` takes it back to the definition's default.
pub fn set_parameter(
    project: &mut Project,
    id: &str,
    name: &str,
    value: Option<&Value>,
) -> Result<(), String> {
    let def = definition_of(project, id)?;
    let param = (def.parameters.iter())
        .find(|p| p.name == name)
        .ok_or_else(|| format!("{} has no parameter {name}", def.key()))?;
    let instance = instance_mut(project, id)?;
    match value {
        None => {
            instance.parameters.remove(name);
        }
        Some(v) => {
            let ty = param.ty;
            let value = param_value(ty, v)
                .ok_or_else(|| format!("{id}.{name} is a {}: {v} is not one", ty.name()))?;
            instance.parameters.insert(name.into(), value);
        }
    }
    Ok(())
}

/// Binds a signal of an instance to an address, or with `None` unbinds it.
pub fn bind_signal(
    project: &mut Project,
    id: &str,
    signal: &str,
    address: Option<&str>,
) -> Result<(), String> {
    let def = definition_of(project, id)?;
    if !def.signals.iter().any(|s| s.name == signal) {
        return Err(format!("{} has no signal {signal}", def.key()));
    }
    let instance = instance_mut(project, id)?;
    match address {
        None => instance.signals.remove(signal),
        Some(a) => instance.signals.insert(signal.into(), a.into()),
    };
    Ok(())
}

/// Adds an instance of a pinned definition, under an id no other instance has.
pub fn add_instance(project: &mut Project, instance: Instance) -> Result<(), String> {
    definition(project, &instance.definition)?;
    if project.scene.instances.iter().any(|i| i.id == instance.id) {
        return Err(format!("an instance {} already exists", instance.id));
    }
    project.scene.instances.push(instance);
    Ok(())
}

/// Moves an instance, turning it too when `rotation` is given.
pub fn move_instance(
    project: &mut Project,
    id: &str,
    position: Vec3,
    rotation: Option<Quat>,
) -> Result<(), String> {
    let instance = instance_mut(project, id)?;
    instance.position = position;
    if let Some(r) = rotation {
        instance.rotation = r;
    }
    Ok(())
}

/// Whether `connection` names instance `id` at either end.
fn touches(connection: &Connection, id: &str) -> bool {
    match connection {
        Connection::Flow { from, to } => from.0 == id || to.0 == id,
        Connection::Mount { host, device, .. } => host.0 == id || device.0 == id,
    }
}

/// Removes an instance, its connections and its place in any group.
pub fn remove_instance(project: &mut Project, id: &str) -> Result<(), String> {
    instance_mut(project, id)?;
    let scene = &mut project.scene;
    scene.instances.retain(|i| i.id != id);
    scene.connections.retain(|c| !touches(c, id));
    for group in &mut scene.groups {
        group.children.retain(|c| c != id);
    }
    Ok(())
}

/// The instances a connection joins.
fn ends(connection: &Connection) -> [&str; 2] {
    match connection {
        Connection::Flow { from, to } => [&from.0, &to.0],
        Connection::Mount { host, device, .. } => [&host.0, &device.0],
    }
}

/// Adds a flow or mount connection between existing instances; the same connection twice is refused.
pub fn connect(project: &mut Project, connection: Connection) -> Result<(), String> {
    for id in ends(&connection) {
        definition_of(project, id)?;
    }
    if project.scene.connections.contains(&connection) {
        return Err("that connection already exists".into());
    }
    project.scene.connections.push(connection);
    Ok(())
}

/// Removes a connection given exactly as it is stored.
pub fn disconnect(project: &mut Project, connection: &Connection) -> Result<(), String> {
    let before = project.scene.connections.len();
    project.scene.connections.retain(|c| c != connection);
    if project.scene.connections.len() == before {
        return Err("no such connection".into());
    }
    Ok(())
}

/// The definition key of a behaviour file, `components/<key>.st`.
fn behaviour_key(file: &str) -> Option<&str> {
    file.strip_prefix("components/")?.strip_suffix(".st")
}

/// The text of a program file (`plc/…`) or of a definition's behaviour (`components/<key>.st`).
pub fn read_source(project: &Project, file: &str) -> Result<String, String> {
    let program = project.program.iter().find(|(path, _)| path == file);
    let behaviour = behaviour_key(file).and_then(|k| project.behaviours.get(k));
    (program.map(|(_, text)| text).or(behaviour))
        .cloned()
        .ok_or_else(|| format!("no source {file}"))
}

/// Replaces the text of an existing program file or behaviour.
pub fn write_source(project: &mut Project, file: &str, text: &str) -> Result<(), String> {
    if let Some((_, old)) = project.program.iter_mut().find(|(path, _)| path == file) {
        *old = text.into();
        return Ok(());
    }
    let key = behaviour_key(file).filter(|k| project.behaviours.contains_key(*k));
    let key = key.ok_or_else(|| format!("no source {file}"))?.to_string();
    project.behaviours.insert(key, text.into());
    Ok(())
}

/// Pins a library family in the project, its definition and its block, unless the project already pins that key: a
/// pinned copy is kept as it is, and upgrading it is explicit (spec 2026-11-08-component-model-design.md).
pub fn pin(project: &mut Project, key: &str) -> Result<(), String> {
    if project.definitions.contains_key(key) {
        return Ok(());
    }
    let family = library::family(key).ok_or_else(|| format!("no family {key} in the library"))?;
    if let Some(block) = family.behaviour {
        project.behaviours.insert(key.into(), block.into());
    }
    project.definitions.insert(key.into(), family.definition);
    Ok(())
}

/// Places a family at a pose, pinning it if the project does not have it yet: an instance with no overrides and no
/// signal bound, named and id'd after the family with the first free number (`belt_1`, `belt_2`, …). Its id.
pub fn place(
    project: &mut Project,
    key: &str,
    position: Vec3,
    rotation: Quat,
) -> Result<String, String> {
    pin(project, key)?;
    let short = key.split('@').next().unwrap_or(key);
    let short = short.rsplit('.').next().unwrap_or(short);
    let taken = |id: &str| project.scene.instances.iter().any(|i| i.id == id);
    let id = (1..)
        .map(|n| format!("{short}_{n}"))
        .find(|id| !taken(id))
        .expect("a free id");
    add_instance(
        project,
        Instance {
            id: id.clone(),
            name: id.clone(),
            definition: key.into(),
            position,
            rotation,
            parameters: Default::default(),
            signals: Default::default(),
        },
    )?;
    Ok(id)
}

/// Renames an instance; its id, which connections and signals name, stays.
pub fn set_name(project: &mut Project, id: &str, name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("a name cannot be empty".into());
    }
    instance_mut(project, id)?.name = name.trim().into();
    Ok(())
}

/// The name of instance `id`'s first port of `kind`.
fn port_of(project: &Project, id: &str, kind: PortKind) -> Result<String, String> {
    let def = definition_of(project, id)?;
    let port = def.ports.iter().find(|p| p.kind == kind);
    port.map(|p| p.name.clone())
        .ok_or_else(|| format!("{id} ({}) has no {kind:?} port", def.key()))
}

/// Mounts a device on a host at a distance along it (an angle in degrees on a curve), or with `None` unmounts it; a
/// device has one host, so a mount replaces the one it had.
pub fn set_mount(
    project: &mut Project,
    device: &str,
    on: Option<(&str, f64)>,
) -> Result<(), String> {
    let device_port = port_of(project, device, PortKind::MountDevice)?;
    let mount = match on {
        Some((host, at)) => {
            if !at.is_finite() {
                return Err(format!("{at} is not a distance along {host}"));
            }
            let host_port = port_of(project, host, PortKind::MountHost)?;
            Some(Connection::Mount {
                host: (host.into(), host_port),
                device: (device.into(), device_port.clone()),
                position: [at, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
            })
        }
        None => None,
    };
    project.scene.connections.retain(|c| {
        !matches!(c, Connection::Mount { device: d, .. } if d.0 == device && d.1 == device_port)
    });
    project.scene.connections.extend(mount);
    Ok(())
}

/// The flow connection at a port, either end.
fn flow_at(connection: &Connection, end: &Endpoint) -> bool {
    matches!(connection, Connection::Flow { from, to } if from == end || to == end)
}

/// Sets what a flow port feeds or is fed by, or with `None` leaves it unconnected. A flow end has one peer, so the
/// connections either end had are replaced.
pub fn set_flow(
    project: &mut Project,
    id: &str,
    port: &str,
    peer: Option<(&str, &str)>,
) -> Result<(), String> {
    let kind_of = |project: &Project, id: &str, port: &str| -> Result<PortKind, String> {
        let def = definition_of(project, id)?;
        let found = def.ports.iter().find(|p| p.name == port);
        found
            .map(|p| p.kind)
            .ok_or_else(|| format!("{} has no port {port}", def.key()))
    };
    let here: Endpoint = (id.into(), port.into());
    let kind = kind_of(project, id, port)?;
    if !matches!(kind, PortKind::FlowIn | PortKind::FlowOut) {
        return Err(format!("{id}.{port} is not a flow port"));
    }
    let flow = match peer {
        None => None,
        Some((peer_id, peer_port)) => {
            let there: Endpoint = (peer_id.into(), peer_port.into());
            match (kind, kind_of(project, peer_id, peer_port)?) {
                (PortKind::FlowOut, PortKind::FlowIn) => Some((here.clone(), there)),
                (PortKind::FlowIn, PortKind::FlowOut) => Some((there, here.clone())),
                _ => return Err(format!("{id}.{port} cannot flow to {peer_id}.{peer_port}")),
            }
        }
    };
    let ends: Vec<Endpoint> = std::iter::once(here)
        .chain(
            flow.iter()
                .flat_map(|(from, to)| [from.clone(), to.clone()]),
        )
        .collect();
    (project.scene.connections).retain(|c| !ends.iter().any(|end| flow_at(c, end)));
    if let Some((from, to)) = flow {
        project
            .scene
            .connections
            .push(Connection::Flow { from, to });
    }
    Ok(())
}

#[cfg(test)]
#[path = "edit_tests.rs"]
mod tests;
