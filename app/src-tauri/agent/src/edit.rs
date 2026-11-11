//! Edits of a working project (spec 2026-11-11-agent-server-design.md, decision 4). Each checks the names it is given,
//! the instance, definition, parameter, signal or file, and the value's type against the parameter's, then changes the
//! project; the format's own checks (ports, addresses, ranges, ST) run at Build, as for a project edited on disk.

use serde_json::Value;
use twinova_project::Project;
use twinova_project::model::{Connection, Definition, Instance, ParamType, ParamValue, Quat, Vec3};

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

#[cfg(test)]
#[path = "edit_tests.rs"]
mod tests;
