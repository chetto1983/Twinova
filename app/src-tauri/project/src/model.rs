//! The document as stored, one type per file of the annex. Every struct refuses unknown fields: a field this build
//! does not know is a diagnostic, never silently dropped.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type Vec3 = [f64; 3];
/// A unit quaternion `[x, y, z, w]`, the glTF convention.
pub type Quat = [f64; 4];

/// A validated project: what `parse_project` returns and `encode_project` writes.
#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub name: String,
    pub app: String,
    pub scene: Scene,
    /// Pinned definitions, by `id@version`.
    pub definitions: BTreeMap<String, Definition>,
    /// ST sources, in the order the controller program is assembled: (path, text).
    pub program: Vec<(String, String)>,
    /// The behaviour block of each definition that has one, by `id@version`; stored as `components/<key>.st`.
    pub behaviours: BTreeMap<String, String>,
    /// Assets, by path (`assets/<sha256>.glb`).
    pub assets: BTreeMap<String, Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: String,
    pub version: u32,
    pub name: String,
    pub app: String,
    pub scene: String,
    pub components: Vec<String>,
    pub program: Vec<String>,
    pub files: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub id: String,
    pub version: u32,
    pub origin: Origin,
    pub generator: String,
    pub parameters: Vec<ParameterDef>,
    pub signals: Vec<SignalDef>,
    pub ports: Vec<PortDef>,
    pub nodes: Vec<Node>,
    pub joints: Vec<Joint>,
    pub behaviour: Option<Behaviour>,
}

impl Definition {
    /// The key a scene names it by, and the stem of its file.
    pub fn key(&self) -> String {
        format!("{}@{}", self.id, self.version)
    }

    /// Where its behaviour block is stored, when it has one.
    pub fn behaviour_path(&self) -> String {
        format!("components/{}.st", self.key())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Origin {
    pub app: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParamType {
    REAL,
    DINT,
    BOOL,
    /// Written as integer nanoseconds.
    TIME,
}

/// A parameter value as JSON holds it. Integers stay integers, so a TIME in ns or a DINT is exact.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    Bool(bool),
    Int(i64),
    Real(f64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterDef {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: ParamType,
    pub unit: Option<String>,
    pub default: ParamValue,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalType {
    BOOL,
    DINT,
    REAL,
}

/// Seen from the PLC: `PlcInput` is a component output that the program reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    PlcInput,
    PlcOutput,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignalDef {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: SignalType,
    pub direction: Direction,
    /// An optional signal may stay unbound: the field then gives it a private address and its block a default (spec
    /// 2026-11-10-curves-belt-options-design.md, decision 5). Written only when true, so format 1 files stay as they
    /// were.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub optional: bool,
    /// The parameter an unbound optional signal reads instead (spec, amended in Task 4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortKind {
    FlowIn,
    FlowOut,
    MountHost,
    MountDevice,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortDef {
    pub name: String,
    pub kind: PortKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub parent: Option<String>,
    pub position: Vec3,
    pub rotation: Quat,
    pub asset: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JointKind {
    Fixed,
    Revolute,
    Prismatic,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Joint {
    pub id: String,
    pub kind: JointKind,
    pub parent: String,
    pub child: String,
    pub axis: Vec3,
    pub limits: Option<[f64; 2]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Behaviour {
    pub function_block: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    pub instances: Vec<Instance>,
    pub groups: Vec<Group>,
    pub connections: Vec<Connection>,
    pub controller: Controller,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Instance {
    pub id: String,
    pub name: String,
    /// `id@version` of a pinned definition.
    pub definition: String,
    pub position: Vec3,
    pub rotation: Quat,
    /// Overrides only.
    pub parameters: BTreeMap<String, ParamValue>,
    /// Signal name to IEC direct address.
    pub signals: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub children: Vec<String>,
}

/// `[instance, port]`.
pub type Endpoint = (String, String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Connection {
    Flow {
        from: Endpoint,
        to: Endpoint,
    },
    Mount {
        host: Endpoint,
        device: Endpoint,
        position: Vec3,
        rotation: Quat,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Controller {
    /// The operator's buttons; a press holds its input bit for one controller scan.
    pub panel: Vec<PanelButton>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelButton {
    pub name: String,
    /// A Bit address in the input area, as the controller sees it.
    pub address: String,
}
