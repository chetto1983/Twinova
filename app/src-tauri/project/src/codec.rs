//! Between a project and its files: `parse_project` validates everything before returning a project, and
//! `encode_project` writes canonical bytes, so the same project always gives the same files and hashes.

use crate::diagnostic::Diagnostic;
use crate::model::{Definition, Manifest, Project, Scene};
use crate::{FORMAT_VERSION, validate, validate_scene};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

/// A project's files by relative path, as a folder or an archive holds them.
pub type Files = BTreeMap<String, Vec<u8>>;

pub const MANIFEST: &str = "twinova.json";
pub const FORMAT: &str = "twinova-project";

pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// A project as opened, with the files whose bytes no longer match the manifest's hash.
#[derive(Debug, Clone, PartialEq)]
pub struct Opened {
    pub project: Project,
    /// Changed outside Twinova (git, an editor, a copy), validated like any file. An interrupted save is recognised by
    /// the save marker, not by these hashes (spec 2026-11-08-project-format-design.md, decision 2).
    pub changed_outside: Vec<String>,
}

/// Reads and validates a whole project. Files the manifest does not list are ignored.
pub fn parse_project(files: &Files) -> Result<Opened, Vec<Diagnostic>> {
    let manifest: Manifest = match files.get(MANIFEST) {
        None => {
            return Err(vec![Diagnostic::new(
                MANIFEST,
                "",
                "is missing: this folder is not a Twinova project",
            )]);
        }
        Some(bytes) => match json(MANIFEST, bytes) {
            Ok(m) => m,
            Err(d) => return Err(vec![version_problem(bytes).unwrap_or(d)]),
        },
    };
    let mut diags = manifest_problems(&manifest, files);
    if !diags.is_empty() {
        return Err(diags);
    }
    let changed_outside: Vec<String> = manifest
        .files
        .iter()
        .filter(|(path, hash)| {
            files
                .get(*path)
                .is_some_and(|bytes| sha256(bytes) != **hash)
        })
        .map(|(path, _)| path.clone())
        .collect();
    let listed = |path: &str| files.get(path).map(Vec::as_slice).unwrap_or_default();
    let assets: BTreeMap<String, Vec<u8>> = manifest
        .files
        .keys()
        .filter(|p| p.starts_with("assets/"))
        .map(|p| (p.clone(), listed(p).to_vec()))
        .collect();
    for (path, bytes) in &assets {
        if path
            .strip_prefix("assets/")
            .and_then(|n| n.split_once('.'))
            .map(|(stem, _)| stem)
            != Some(&sha256(bytes))
        {
            diags.push(Diagnostic::new(
                path,
                "",
                "an asset must be named after the SHA-256 of its bytes",
            ));
        }
    }
    let asset_names: BTreeSet<String> = assets.keys().cloned().collect();
    let mut definitions = BTreeMap::new();
    for path in &manifest.components {
        match json::<Definition>(path, listed(path)) {
            Err(d) => diags.push(d),
            Ok(def) => {
                diags.extend(validate::definition(path, &def, &asset_names));
                if definitions.insert(def.key(), def).is_some() {
                    diags.push(Diagnostic::new(
                        path,
                        "",
                        "pins a definition another file already pins",
                    ));
                }
            }
        }
    }
    let scene = match json::<Scene>(&manifest.scene, listed(&manifest.scene)) {
        Err(d) => {
            diags.push(d);
            None
        }
        Ok(scene) => Some(scene),
    };
    if let Some(scene) = &scene {
        diags.extend(validate_scene::scene(scene, &definitions));
    }
    let mut program = Vec::new();
    for path in &manifest.program {
        match text(path, listed(path)) {
            Ok(text) => program.push((path.clone(), text)),
            Err(d) => diags.push(d),
        }
    }
    let mut behaviours = BTreeMap::new();
    for def in definitions.values() {
        let path = def.behaviour_path();
        match (&def.behaviour, manifest.files.contains_key(&path)) {
            (None, false) => {}
            (None, true) => diags.push(Diagnostic::new(
                &path,
                "",
                format!("is listed, but {} has no behaviour", def.key()),
            )),
            (Some(_), false) => diags.push(Diagnostic::new(
                MANIFEST,
                "files",
                format!("'{path}', the behaviour of {}, is not listed", def.key()),
            )),
            (Some(behaviour), true) => match text(&path, listed(&path)) {
                Err(d) => diags.push(d),
                Ok(source) => {
                    if !validate::declares_block(&source, &behaviour.function_block) {
                        diags.push(Diagnostic::new(
                            &path,
                            "",
                            format!(
                                "does not declare FUNCTION_BLOCK {}, the behaviour {} names",
                                behaviour.function_block,
                                def.key()
                            ),
                        ));
                    }
                    behaviours.insert(def.key(), source);
                }
            },
        }
    }
    match scene {
        Some(scene) if diags.is_empty() => Ok(Opened {
            project: Project {
                name: manifest.name,
                app: manifest.app,
                scene,
                definitions,
                program,
                behaviours,
                assets,
            },
            changed_outside,
        }),
        _ => Err(diags),
    }
}

/// Writes a project as files. The manifest lists every other file with its SHA-256.
pub fn encode_project(project: &Project) -> Files {
    let mut files = Files::new();
    files.insert("scene.json".into(), canonical(&project.scene));
    let mut components = Vec::new();
    for def in project.definitions.values() {
        let path = format!("components/{}.json", def.key());
        files.insert(path.clone(), canonical(def));
        components.push(path);
    }
    for (path, text) in &project.program {
        files.insert(path.clone(), text.clone().into_bytes());
    }
    for (key, source) in &project.behaviours {
        files.insert(format!("components/{key}.st"), source.clone().into_bytes());
    }
    for (path, bytes) in &project.assets {
        files.insert(path.clone(), bytes.clone());
    }
    let manifest = Manifest {
        format: FORMAT.into(),
        version: FORMAT_VERSION,
        name: project.name.clone(),
        app: project.app.clone(),
        scene: "scene.json".into(),
        components,
        program: project
            .program
            .iter()
            .map(|(path, _)| path.clone())
            .collect(),
        files: files
            .iter()
            .map(|(path, bytes)| (path.clone(), sha256(bytes)))
            .collect(),
    };
    files.insert(MANIFEST.into(), canonical(&manifest));
    files
}

/// Pretty JSON with object keys sorted and a final newline, whatever serde_json's map order is built with.
pub fn canonical<T: Serialize>(value: &T) -> Vec<u8> {
    fn sorted(value: Value) -> Value {
        match value {
            Value::Object(map) => {
                let entries: BTreeMap<String, Value> =
                    map.into_iter().map(|(k, v)| (k, sorted(v))).collect();
                Value::Object(entries.into_iter().collect())
            }
            Value::Array(items) => Value::Array(items.into_iter().map(sorted).collect()),
            other => other,
        }
    }
    let value = sorted(serde_json::to_value(value).expect("the model serialises"));
    let mut bytes = serde_json::to_vec_pretty(&value).expect("a JSON value serialises");
    bytes.push(b'\n');
    bytes
}

fn text(file: &str, bytes: &[u8]) -> Result<String, Diagnostic> {
    String::from_utf8(bytes.to_vec()).map_err(|_| Diagnostic::new(file, "", "is not UTF-8 text"))
}

fn json<T: DeserializeOwned>(file: &str, bytes: &[u8]) -> Result<T, Diagnostic> {
    serde_json::from_slice(bytes)
        .map_err(|e| Diagnostic::new(file, format!("line {}", e.line()), e.to_string()))
}

/// A manifest that does not parse may still be a newer format: say so rather than report its unknown fields.
fn version_problem(bytes: &[u8]) -> Option<Diagnostic> {
    let version = serde_json::from_slice::<Value>(bytes)
        .ok()?
        .get("version")?
        .as_u64()?;
    (version > u64::from(FORMAT_VERSION)).then(|| newer(version))
}

fn newer(version: u64) -> Diagnostic {
    Diagnostic::new(
        MANIFEST,
        "version",
        format!(
            "saved by a newer Twinova (format {version}); this build reads format {FORMAT_VERSION}"
        ),
    )
}

fn manifest_problems(m: &Manifest, files: &Files) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut err = |path: &str, message: String| out.push(Diagnostic::new(MANIFEST, path, message));
    if m.format != FORMAT {
        err("format", format!("must be '{FORMAT}'"));
    }
    if u64::from(m.version) > u64::from(FORMAT_VERSION) {
        out.push(newer(m.version.into()));
        return out;
    }
    if m.version != FORMAT_VERSION {
        err("version", format!("must be {FORMAT_VERSION}"));
    }
    for path in m.files.keys() {
        if path == MANIFEST {
            err("files", format!("must not list {MANIFEST} itself"));
        } else if let Some(problem) = validate::path_problem(path) {
            err("files", format!("'{path}' {problem}"));
        }
    }
    for (a, b) in validate::case_collisions(m.files.keys()) {
        err(
            "files",
            format!("'{a}' and '{b}' differ only in case, and collide on Windows"),
        );
    }
    let referenced = std::iter::once(("scene", &m.scene))
        .chain(m.components.iter().map(|p| ("components", p)))
        .chain(m.program.iter().map(|p| ("program", p)));
    for (field, path) in referenced {
        if !m.files.contains_key(path) {
            err(field, format!("'{path}' is not listed in files"));
        }
    }
    for path in m.files.keys().filter(|path| !files.contains_key(*path)) {
        err("files", format!("'{path}' is listed but missing"));
    }
    out
}
