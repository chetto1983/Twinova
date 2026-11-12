//! The run tools (spec 2026-11-11-agent-server-design.md, decision 3): open and build a project, advance virtual time,
//! read the line and drive what a user drives, keep and restore states. Virtual time advances only when a tool asks:
//! there is no wall clock here.

use std::path::Path;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use twinova_engine::engine::{Engine, Snapshot};
use twinova_project::sources::Kind;

use crate::{Agent, structured};

#[derive(Deserialize, JsonSchema)]
pub struct OpenProject {
    /// The project's folder.
    pub path: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct Step {
    /// Virtual milliseconds to run, in whole controller scans; one scan when absent.
    pub ms: Option<i64>,
}

/// What `run_until` waits for: exactly one of a signal's value, a sink's count, or a virtual time.
#[derive(Deserialize, JsonSchema)]
pub struct RunUntil {
    /// A signal by name (`instance.signal`) or by address, as the I/O panel shows it.
    pub signal: Option<String>,
    /// The value `signal` must read, as the I/O panel shows it (`TRUE`, `30`, `0.5`).
    pub equals: Option<String>,
    /// A sink by id.
    pub sink: Option<String>,
    /// The count `sink` must reach.
    pub at_least: Option<u64>,
    /// A virtual time to reach, in ms.
    pub time_ms: Option<i64>,
    /// Gives up after this many virtual milliseconds.
    pub timeout_ms: i64,
}

#[derive(Deserialize, JsonSchema)]
pub struct SnapshotArgs {
    /// Include every carton's pose (left out unless asked: a line can hold hundreds).
    pub cartons: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Read {
    /// Signals by name (`instance.signal`) or by address.
    pub signals: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Press {
    /// A panel button's address: true for the next scan.
    pub address: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct BlockSink {
    /// A sink by id.
    pub sink: String,
    pub blocked: bool,
}

#[derive(Deserialize, JsonSchema)]
pub struct ChangeProgram {
    /// The line program's files as (path, text), in order, as the project lists them (`plc/Main.st`, …).
    pub program: Vec<(String, String)>,
}

/// An online change's or a reload's answer: what was taken, Build's warnings, and the line then.
#[derive(Serialize)]
struct Changed {
    /// `program` (online, the plant went on), `behaviours` (rebuilt at time zero), or null (nothing changed).
    taken: Option<Kind>,
    warnings: Vec<String>,
    state: Snapshot,
}

#[derive(Deserialize, JsonSchema)]
pub struct RestoreState {
    /// The id `save_state` gave.
    pub id: usize,
}

/// `run_until`'s answer: whether the condition held, and the line then.
#[derive(Serialize)]
struct Until {
    held: bool,
    state: Snapshot,
}

/// What `run_until` waits for, checked.
enum Condition {
    Signal(String, String),
    Sink(String, u64),
    Time(i64),
}

impl RunUntil {
    fn condition(&self) -> Result<Condition, String> {
        match (
            &self.signal,
            &self.equals,
            &self.sink,
            self.at_least,
            self.time_ms,
        ) {
            (Some(s), Some(v), None, None, None) => Ok(Condition::Signal(s.clone(), v.clone())),
            (None, None, Some(s), Some(n), None) => Ok(Condition::Sink(s.clone(), n)),
            (None, None, None, None, Some(t)) => Ok(Condition::Time(t)),
            _ => {
                Err("give exactly one of: signal and equals, sink and at_least, or time_ms".into())
            }
        }
    }
}

/// `snapshot` without its cartons: the line's state as a tool answers it.
fn state(engine: &Engine) -> Snapshot {
    Snapshot {
        cartons: Vec::new(),
        ..engine.snapshot()
    }
}

/// The value the I/O panel shows for a signal named or addressed `signal`.
fn signal(snapshot: &Snapshot, signal: &str) -> Result<String, String> {
    (snapshot.signals.iter())
        .find(|s| s.name == signal || s.address == signal)
        .map(|s| s.value.clone())
        .ok_or_else(|| format!("no signal named or addressed {signal}"))
}

fn sink(snapshot: &Snapshot, sink: &str) -> Result<(usize, u64), String> {
    (snapshot.sinks.iter().enumerate())
        .find(|(_, s)| s.id == sink)
        .map(|(k, s)| (k, s.delivered))
        .ok_or_else(|| format!("no sink {sink}"))
}

impl Condition {
    fn holds(&self, snapshot: &Snapshot) -> Result<bool, String> {
        Ok(match self {
            Condition::Signal(name, value) => signal(snapshot, name)? == *value,
            Condition::Sink(id, n) => sink(snapshot, id)?.1 >= *n,
            Condition::Time(t) => snapshot.time_ms >= *t,
        })
    }
}

#[tool_router(router = run_router, vis = "pub(crate)")]
impl Agent {
    #[tool(
        description = "Open a Twinova project folder and build it at time zero. Answers its name, its instances \
                          (id, definition) and Build's warnings; a project that does not open or build is an error."
    )]
    async fn open_project(
        &self,
        Parameters(a): Parameters<OpenProject>,
    ) -> Result<CallToolResult, String> {
        let opened = self.session()?.open(Path::new(&a.path))?;
        structured(&opened)
    }

    #[tool(
        description = "Run virtual time by whole controller scans (one scan without `ms`) and answer the line's \
                          state: time, sinks, moving devices, conveyors' travel and every signal."
    )]
    async fn step(&self, Parameters(a): Parameters<Step>) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        let engine = session.engine()?;
        let target = engine.time_ms() + a.ms.unwrap_or(0);
        engine.step()?;
        while engine.time_ms() < target {
            engine.step()?;
        }
        structured(&state(engine))
    }

    #[tool(
        description = "Run scan by scan until a condition holds: a signal reads a value, a sink counts at least n, \
                          or a virtual time is reached; or until `timeout_ms` of virtual time has passed. Answers \
                          whether it held and the line's state then."
    )]
    async fn run_until(
        &self,
        Parameters(a): Parameters<RunUntil>,
    ) -> Result<CallToolResult, String> {
        let condition = a.condition()?;
        let mut session = self.session()?;
        let engine = session.engine()?;
        let end = engine.time_ms() + a.timeout_ms;
        let mut now = state(engine);
        let mut held = condition.holds(&now)?;
        while !held && now.time_ms < end {
            engine.step()?;
            now = state(engine);
            held = condition.holds(&now)?;
        }
        structured(&Until { held, state: now })
    }

    #[tool(description = "The line's state, with every carton's pose when `cartons` is true.")]
    async fn snapshot(
        &self,
        Parameters(a): Parameters<SnapshotArgs>,
    ) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        let engine = session.engine()?;
        let snapshot = if a.cartons.unwrap_or(false) {
            engine.snapshot()
        } else {
            state(engine)
        };
        structured(&snapshot)
    }

    #[tool(
        description = "Read signals by name (`instance.signal`) or address, as the I/O panel shows them."
    )]
    async fn read(&self, Parameters(a): Parameters<Read>) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        let now = state(session.engine()?);
        let values = (a.signals.iter())
            .map(|s| signal(&now, s).map(|v| (s.clone(), v)))
            .collect::<Result<std::collections::BTreeMap<_, _>, _>>()?;
        structured(&values)
    }

    #[tool(description = "Press a panel button: its input reads true for the next scan.")]
    async fn press(&self, Parameters(a): Parameters<Press>) -> Result<CallToolResult, String> {
        self.session()?.engine()?.press(&a.address);
        structured(&a.address)
    }

    #[tool(
        description = "Block or free a sink: a blocked sink takes nothing and holds products at its exit."
    )]
    async fn block_sink(
        &self,
        Parameters(a): Parameters<BlockSink>,
    ) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        let engine = session.engine()?;
        let (k, _) = sink(&state(engine), &a.sink)?;
        engine.set_sink_blocked(k, a.blocked);
        structured(&a.blocked)
    }

    #[tool(
        description = "Change the line program online, as a PLC download in RUN: the PLC takes it between two scans \
                          with a warm restart (RETAIN variables and time kept, the rest reinitialised; the outputs \
                          hold until its first scan) while the plant goes on. A program Build would refuse is an \
                          error and changes nothing. Answers Build's warnings and the line's state."
    )]
    async fn change_program(
        &self,
        Parameters(a): Parameters<ChangeProgram>,
    ) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        let warnings = session.change_program(a.program)?;
        let state = state(session.engine()?);
        structured(&Changed {
            taken: Some(Kind::Program),
            warnings,
            state,
        })
    }

    #[tool(
        description = "Take the ST sources as the project's folder holds them now, as the app does when they are \
                          saved outside it (VS Code): a changed program goes online (see change_program), a changed \
                          behaviour rebuilds the line at time zero. Answers what was taken (null when the files equal \
                          the running sources), Build's warnings and the line's state."
    )]
    async fn reload_sources(&self) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        let reloaded = session.reload_sources()?;
        let state = state(session.engine()?);
        let (taken, warnings) = reloaded.map_or((None, Vec::new()), |r| (Some(r.kind), r.warnings));
        structured(&Changed {
            taken,
            warnings,
            state,
        })
    }

    #[tool(description = "Keep the line's current state; answers its id for restore_state.")]
    async fn save_state(&self) -> Result<CallToolResult, String> {
        structured(&self.session()?.save_state()?)
    }

    #[tool(
        description = "Bring the line back to a state save_state kept, replayed exactly from time zero."
    )]
    async fn restore_state(
        &self,
        Parameters(a): Parameters<RestoreState>,
    ) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        session.restore_state(a.id)?;
        structured(&state(session.engine()?))
    }

    #[tool(description = "The last built project again at time zero.")]
    async fn reset(&self) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        session.reset()?;
        structured(&state(session.engine()?))
    }

    #[tool(
        description = "The physics world's fingerprint: equal for equal runs, on every platform."
    )]
    async fn fingerprint(&self) -> Result<CallToolResult, String> {
        let mut session = self.session()?;
        let engine = session.engine()?;
        structured(&serde_json::json!({
            "time_ms": engine.time_ms(),
            "fingerprint": format!("{:#018x}", engine.fingerprint()),
        }))
    }
}
