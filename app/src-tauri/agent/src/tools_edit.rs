//! The editing tools (spec 2026-11-11-agent-server-design.md, decision 4; user, 2026-11-11: full scene editing). They
//! change the session's working project; `build` checks it as the format does on open and replaces the running line,
//! which stays as it was if Build fails, as in the app.

use std::path::{Path, PathBuf};

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};
use twinova_physics::interfaces_view::interfaces_view;
use twinova_physics::snap;
use twinova_project::model::{Connection, Instance, Quat, Side, Vec3};
use twinova_project::storage::save_folder;
use twinova_project::{Project, encode_project, parse_project};
use twinova_project::{edit, library};

use crate::{Agent, structured};

#[derive(Deserialize, JsonSchema)]
pub struct SetParameter {
    pub instance: String,
    pub name: String,
    /// A number for REAL, an integer for DINT and TIME (ns), a boolean for BOOL; null for the definition's default.
    pub value: Option<Value>,
}

#[derive(Deserialize, JsonSchema)]
pub struct BindSignal {
    pub instance: String,
    pub signal: String,
    /// An IEC direct address (`%IX0.0`, `%QD8`); null to unbind it.
    pub address: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct AddInstance {
    pub id: String,
    /// `id@version`: a definition the project pins, or a family of the library, which is pinned (see describe).
    pub definition: String,
    /// Shown in the app; the id when absent.
    pub name: Option<String>,
    /// Metres, `[x, y, z]`, y up.
    pub position: Vec3,
    /// A unit quaternion `[x, y, z, w]`; no turn when absent.
    pub rotation: Option<Quat>,
    /// Parameter overrides, as `set_parameter` takes them.
    pub parameters: Option<serde_json::Map<String, Value>>,
    /// Signal name to address.
    pub signals: Option<std::collections::BTreeMap<String, String>>,
}

#[derive(Deserialize, JsonSchema)]
pub struct MoveInstance {
    pub id: String,
    pub position: Vec3,
    pub rotation: Option<Quat>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Join {
    /// The part that moves.
    pub id: String,
    /// Its end that meets the other: `in`, `out`, or a turntable's `left` or `right`. A spur merges by its `out`.
    pub end: Option<String>,
    /// The part it joins, which stays where it is.
    pub peer: String,
    /// The peer's end to face. Leave it out to merge onto the peer's flank instead, with `at` and `side`.
    pub peer_end: Option<String>,
    /// To merge: how far along the peer's run, in metres.
    pub at: Option<f64>,
    /// To merge: the peer's `left` or `right` flank, looking along its flow.
    pub side: Option<String>,
    /// Metres between the two faces, 0 to 0.15; 0 when absent.
    pub gap: Option<f64>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Id {
    pub id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ConnectionArgs {
    /// As stored in scene.json: `{"kind": "flow", "from": [instance, port], "to": [instance, port]}`,
    /// `{"kind": "mount", "host": [instance, "rail"], "device": [instance, "mount"], "position": [at, 0, 0],
    /// "rotation": [0, 0, 0, 1]}` or `{"kind": "merge", "from": [spur, "out"], "host": [instance, "rail"], "at": m,
    /// "side": "left" | "right"}`.
    pub connection: Value,
}

#[derive(Deserialize, JsonSchema)]
pub struct ReadSource {
    /// `plc/<file>.st` or `components/<id@version>.st`.
    pub file: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct WriteSource {
    pub file: String,
    pub text: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct SaveProject {
    /// A folder; the one the project was opened from when absent.
    pub path: Option<String>,
}

fn connection(args: Value) -> Result<Connection, String> {
    serde_json::from_value(args).map_err(|e| format!("not a connection: {e}"))
}

/// What `describe` answers: the working project as an agent edits it.
fn describe(p: &Project) -> Value {
    let definitions: Vec<Value> = (p.definitions.values())
        .map(|d| {
            json!({
                "key": d.key(),
                "generator": d.generator,
                "parameters": d.parameters,
                "signals": d.signals,
                "ports": d.ports,
                "behaviour": d.behaviour.as_ref().map(|_| d.behaviour_path()),
            })
        })
        .collect();
    let library: Vec<Value> = (library::families().iter())
        .map(|f| json!({ "key": f.definition.key(), "group": f.group }))
        .collect();
    json!({
        "name": p.name,
        "instances": p.scene.instances,
        "connections": p.scene.connections,
        "panel": p.scene.controller.panel,
        "definitions": definitions,
        "library": library,
        "interfaces": interfaces_view(p),
        "program": p.program.iter().map(|(path, _)| path).collect::<Vec<_>>(),
    })
}

#[tool_router(router = edit_router, vis = "pub(crate)")]
impl Agent {
    #[tool(
        description = "The working project as the editing tools change it: instances (definition, pose, \
                          parameters, signals), connections, panel buttons, definitions (their parameters, signals, \
                          ports and behaviour file), the library's families add_instance can place, program files, \
                          and each instance's interfaces for join: its ends and, on a conveyor, its rail (frames in \
                          the instance's own frame)."
    )]
    async fn describe(&self) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        structured(&describe(session.working()?))
    }

    #[tool(
        description = "Set a parameter of an instance (null: back to the definition's default). Takes effect at \
                          build."
    )]
    async fn set_parameter(
        &self,
        Parameters(a): Parameters<SetParameter>,
    ) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        edit::set_parameter(session.working()?, &a.instance, &a.name, a.value.as_ref())?;
        structured(&a.instance)
    }

    #[tool(
        description = "Bind a signal of an instance to an IEC address, or unbind it with null. Takes effect at \
                          build."
    )]
    async fn bind_signal(
        &self,
        Parameters(a): Parameters<BindSignal>,
    ) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        edit::bind_signal(
            session.working()?,
            &a.instance,
            &a.signal,
            a.address.as_deref(),
        )?;
        structured(&a.instance)
    }

    #[tool(
        description = "Add an instance of a pinned definition or of a library family (pinned then), with its pose, \
                          parameters and signals."
    )]
    async fn add_instance(
        &self,
        Parameters(a): Parameters<AddInstance>,
    ) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        let working = session.working()?;
        // On a copy, so a refused parameter leaves neither the instance nor a newly pinned family behind.
        let mut project = working.clone();
        edit::pin(&mut project, &a.definition)?;
        let instance = Instance {
            name: a.name.unwrap_or_else(|| a.id.clone()),
            id: a.id.clone(),
            definition: a.definition,
            position: a.position,
            rotation: a.rotation.unwrap_or([0.0, 0.0, 0.0, 1.0]),
            parameters: Default::default(),
            signals: a.signals.unwrap_or_default(),
        };
        edit::add_instance(&mut project, instance)?;
        for (name, value) in a.parameters.unwrap_or_default() {
            edit::set_parameter(&mut project, &a.id, &name, Some(&value))?;
        }
        *working = project;
        structured(&a.id)
    }

    #[tool(description = "Move an instance, and turn it when rotation is given.")]
    async fn move_instance(
        &self,
        Parameters(a): Parameters<MoveInstance>,
    ) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        let working = session.working()?;
        edit::move_instance(working, &a.id, a.position, a.rotation)?;
        snap::moved(working, &a.id);
        structured(&a.id)
    }

    #[tool(
        description = "Snap a part to another (see describe's interfaces): turn and move `id` so that its `end` \
                          faces the peer's `peer_end` across `gap` metres and join them with a flow (a conveyor also \
                          takes the height that meets the other); or, with `at` and `side` instead of `peer_end`, \
                          merge spur `id` square onto the peer's flank. Returns what else it changed."
    )]
    async fn join(&self, Parameters(a): Parameters<Join>) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        let working = session.working()?;
        let gap = a.gap.unwrap_or(0.0);
        let notice = match (a.peer_end, a.at) {
            (Some(peer_end), None) => {
                let end = a.end.ok_or("give the moving part's end")?;
                snap::join(working, (&a.id, &end), (&a.peer, &peer_end), gap)?
            }
            (None, Some(at)) => {
                let side = match a.side.as_deref() {
                    Some("left") => Side::Left,
                    Some("right") => Side::Right,
                    _ => return Err("give side, left or right, to merge".into()),
                };
                snap::merge(working, &a.id, (&a.peer, at, side), gap)?
            }
            _ => {
                return Err(
                    "give peer_end to join two ends, or at and side to merge onto a flank".into(),
                );
            }
        };
        structured(&json!({ "notice": notice }))
    }

    #[tool(description = "Remove an instance with its connections.")]
    async fn remove_instance(
        &self,
        Parameters(a): Parameters<Id>,
    ) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        edit::remove_instance(session.working()?, &a.id)?;
        structured(&a.id)
    }

    #[tool(description = "Add a flow or mount connection, given as scene.json stores it.")]
    async fn connect(
        &self,
        Parameters(a): Parameters<ConnectionArgs>,
    ) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        edit::connect(session.working()?, connection(a.connection)?)?;
        structured(&true)
    }

    #[tool(
        description = "Remove a connection, given exactly as scene.json stores it (see describe)."
    )]
    async fn disconnect(
        &self,
        Parameters(a): Parameters<ConnectionArgs>,
    ) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        edit::disconnect(session.working()?, &connection(a.connection)?)?;
        structured(&true)
    }

    #[tool(
        description = "The text of a program file (plc/…) or of a definition's behaviour (components/…st)."
    )]
    async fn read_source(
        &self,
        Parameters(a): Parameters<ReadSource>,
    ) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        structured(&edit::read_source(session.working()?, &a.file)?)
    }

    #[tool(
        description = "Replace the text of a program file or a definition's behaviour. Compiled at build."
    )]
    async fn write_source(
        &self,
        Parameters(a): Parameters<WriteSource>,
    ) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        edit::write_source(session.working()?, &a.file, &a.text)?;
        structured(&a.file)
    }

    #[tool(
        description = "Check the working project as the format does on open, compile its ST, and replace the \
                          running line at time zero; on failure the line runs on and the errors are returned."
    )]
    async fn build(&self) -> Result<CallToolResult, String> {
        structured(&self.session()?.build()?)
    }

    #[tool(
        description = "Save the working project to a folder (the one it was opened from by default), once it \
                          passes the format's checks."
    )]
    async fn save_project(
        &self,
        Parameters(a): Parameters<SaveProject>,
    ) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        let folder: PathBuf = match a.path {
            Some(p) => p.into(),
            None => session.folder.clone().ok_or("open a project first")?,
        };
        let checked = parse_project(&encode_project(session.working()?))
            .map_err(|d| twinova_engine::diagnostics(&d))?
            .project;
        save_folder(Path::new(&folder), &checked).map_err(|e| e.to_string())?;
        structured(&folder)
    }
}
