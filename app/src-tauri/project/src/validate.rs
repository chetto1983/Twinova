//! Rules on names, paths, numbers and definitions (spec 2026-11-08-project-format-design.md, annex).

use crate::diagnostic::Diagnostic;
use crate::model::{Definition, ParamType, ParamValue, PortKind, Quat, SignalType, Vec3};
use std::collections::{BTreeMap, BTreeSet};

/// `^[a-z][a-z0-9_]*$`.
pub(crate) fn is_id(s: &str) -> bool {
    let mut bytes = s.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_lowercase())
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// A dotted definition id such as `twinova.belt`.
pub(crate) fn is_definition_id(s: &str) -> bool {
    s.split('.').all(is_id)
}

/// Why a project path is refused, if it is: it must stay inside the project folder on every platform.
pub(crate) fn path_problem(path: &str) -> Option<&'static str> {
    if path.is_empty() {
        Some("is empty")
    } else if path.starts_with('/') || path.contains('\\') || path.contains(':') {
        Some("must be relative, with '/' separators and no drive letter")
    } else if path
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        Some("must not contain empty, '.' or '..' parts")
    } else {
        None
    }
}

/// Paths that differ only in case, which collide on Windows.
pub(crate) fn case_collisions<'a>(
    paths: impl Iterator<Item = &'a String>,
) -> Vec<(String, String)> {
    let mut seen: BTreeMap<String, &String> = BTreeMap::new();
    let mut collisions = Vec::new();
    for path in paths {
        if let Some(first) = seen.insert(path.to_lowercase(), path) {
            collisions.push((first.clone(), path.clone()));
        }
    }
    collisions
}

pub(crate) fn finite(values: &[f64]) -> bool {
    values.iter().all(|v| v.is_finite())
}

pub(crate) fn pose_problem(position: &Vec3, rotation: &Quat) -> Option<String> {
    if !finite(position) || !finite(rotation) {
        return Some("position and rotation must be finite".into());
    }
    let norm = rotation.iter().map(|c| c * c).sum::<f64>().sqrt();
    (!(1.0 - 1e-6..=1.0 + 1e-6).contains(&norm))
        .then(|| format!("rotation must be a unit quaternion (its norm is {norm})"))
}

/// Checks a value against its parameter's type and range.
pub(crate) fn value_problem(
    ty: ParamType,
    value: ParamValue,
    min: Option<f64>,
    max: Option<f64>,
) -> Option<String> {
    let number = match (ty, value) {
        (ParamType::BOOL, ParamValue::Bool(_)) => return None,
        (ParamType::REAL, ParamValue::Real(v)) => v,
        (ParamType::REAL, ParamValue::Int(v)) => v as f64,
        (ParamType::DINT, ParamValue::Int(v)) if i32::try_from(v).is_ok() => v as f64,
        (ParamType::DINT, ParamValue::Int(_)) => return Some("is outside the DINT range".into()),
        (ParamType::TIME, ParamValue::Int(v)) if v >= 0 => v as f64,
        (ParamType::TIME, ParamValue::Int(_)) => {
            return Some("a TIME in ns cannot be negative".into());
        }
        (ty, _) => return Some(format!("is not a {ty:?} value")),
    };
    if !number.is_finite() {
        Some("must be finite".into())
    } else if min.is_some_and(|m| number < m) || max.is_some_and(|m| number > m) {
        Some(format!(
            "{number} is outside [{}, {}]",
            fmt_bound(min),
            fmt_bound(max)
        ))
    } else {
        None
    }
}

fn fmt_bound(bound: Option<f64>) -> String {
    bound.map_or("-".into(), |b| b.to_string())
}

/// Whether a parameter can stand for a signal: the same IEC type (a signal has no TIME).
fn same_type(parameter: ParamType, signal: SignalType) -> bool {
    matches!(
        (parameter, signal),
        (ParamType::BOOL, SignalType::BOOL)
            | (ParamType::DINT, SignalType::DINT)
            | (ParamType::REAL, SignalType::REAL)
    )
}

/// Names that occur more than once.
pub(crate) fn duplicates<'a>(names: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut seen = BTreeSet::new();
    names.filter(|n| !seen.insert(*n)).collect()
}

pub(crate) fn definition(
    file: &str,
    def: &Definition,
    assets: &BTreeSet<String>,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut err = |path: String, message: String| out.push(Diagnostic::new(file, path, message));
    if !is_definition_id(&def.id) {
        err(
            "id".into(),
            format!("'{}' is not a dotted lowercase id", def.id),
        );
    }
    if file != format!("components/{}.json", def.key()) {
        err(
            String::new(),
            format!("must be stored as components/{}.json", def.key()),
        );
    }
    for (kind, names) in [
        (
            "parameter",
            def.parameters
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
        ),
        (
            "signal",
            def.signals.iter().map(|s| s.name.as_str()).collect(),
        ),
        ("port", def.ports.iter().map(|p| p.name.as_str()).collect()),
        ("node", def.nodes.iter().map(|n| n.id.as_str()).collect()),
        ("joint", def.joints.iter().map(|j| j.id.as_str()).collect()),
    ] {
        for name in names.iter().filter(|n| !is_id(n)) {
            err(format!("{kind} '{name}'"), "is not a lowercase id".into());
        }
        for name in duplicates(names.into_iter()) {
            err(format!("{kind} '{name}'"), "is declared twice".into());
        }
    }
    for p in &def.parameters {
        if let Some(problem) = value_problem(p.ty, p.default, p.min, p.max) {
            err(format!("parameters.{}.default", p.name), problem);
        }
        if p.min.zip(p.max).is_some_and(|(lo, hi)| lo > hi) {
            err(format!("parameters.{}", p.name), "min is above max".into());
        }
    }
    for s in &def.signals {
        let Some(fallback) = &s.fallback else {
            continue;
        };
        let path = format!("signals.{}", s.name);
        match def.parameters.iter().find(|p| &p.name == fallback) {
            _ if !s.optional => err(path, "only an optional signal has a fallback".into()),
            None => err(path, format!("fallback '{fallback}' is not a parameter")),
            Some(p) if !same_type(p.ty, s.ty) => err(
                path,
                format!("fallback '{fallback}' is {:?}, not {:?}", p.ty, s.ty),
            ),
            Some(_) => {}
        }
    }
    let nodes: BTreeMap<&str, Option<&str>> = def
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n.parent.as_deref()))
        .collect();
    if def.nodes.iter().filter(|n| n.parent.is_none()).count() != 1 {
        err(
            "nodes".into(),
            "must have exactly one root (parent: null)".into(),
        );
    }
    for n in &def.nodes {
        if let Some(problem) = pose_problem(&n.position, &n.rotation) {
            err(format!("nodes.{}", n.id), problem);
        }
        if n.parent.as_deref().is_some_and(|p| !nodes.contains_key(p)) {
            err(format!("nodes.{}.parent", n.id), "names no node".into());
        } else if in_cycle(&nodes, &n.id) {
            err(
                format!("nodes.{}.parent", n.id),
                "is part of a parent cycle".into(),
            );
        }
        if let Some(asset) = &n.asset
            && !assets.contains(asset)
        {
            err(
                format!("nodes.{}.asset", n.id),
                format!("'{asset}' is not a listed asset"),
            );
        }
    }
    for j in &def.joints {
        for (end, node) in [("parent", &j.parent), ("child", &j.child)] {
            if !nodes.contains_key(node.as_str()) {
                err(format!("joints.{}.{end}", j.id), "names no node".into());
            }
        }
        if !finite(&j.axis) || j.limits.is_some_and(|l| !finite(&l) || l[0] > l[1]) {
            err(
                format!("joints.{}", j.id),
                "axis and limits must be finite, with limits in order".into(),
            );
        }
    }
    out
}

fn in_cycle(parents: &BTreeMap<&str, Option<&str>>, start: &str) -> bool {
    let mut at = parents.get(start).copied().flatten();
    for _ in 0..parents.len() {
        match at {
            Some(node) if node == start => return true,
            Some(node) => at = parents.get(node).copied().flatten(),
            None => return false,
        }
    }
    false
}

/// The ports of a definition by name, for the scene's connection checks.
pub(crate) fn ports(def: &Definition) -> BTreeMap<&str, PortKind> {
    def.ports
        .iter()
        .map(|p| (p.name.as_str(), p.kind))
        .collect()
}

#[cfg(test)]
#[path = "validate_tests.rs"]
mod tests;

/// Whether an ST source declares `FUNCTION_BLOCK <name>`, ignoring case, comments and layout. truST checks the
/// source itself at Build; this only catches a behaviour file that does not hold the block its definition names.
pub(crate) fn declares_block(source: &str, name: &str) -> bool {
    let mut code = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(start) = rest.find("(*") {
        code.push_str(&rest[..start]);
        code.push(' ');
        rest = rest[start + 2..]
            .split_once("*)")
            .map_or("", |(_, after)| after);
    }
    code.push_str(rest);
    let words: Vec<&str> = code
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|w| !w.is_empty())
        .collect();
    words
        .windows(2)
        .any(|w| w[0].eq_ignore_ascii_case("FUNCTION_BLOCK") && w[1].eq_ignore_ascii_case(name))
}
