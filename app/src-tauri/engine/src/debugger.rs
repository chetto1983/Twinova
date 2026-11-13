//! Breakpoints that hold the line (specs 2026-11-13-breakpoints-hold-design.md and -breakpoints-complete-design.md).
//! At a stop, truST's hook blocks the scanning thread until it is resumed (`host/debug/control/hook.rs`), so the engine,
//! the plant and virtual time stand: a PLC in HOLD. A `Debugger` therefore works without the engine, which that thread
//! holds while it is stopped: it resolves lines from statement locations copied at attach, reads values from the paused
//! snapshot, and hands a write made while held to the engine through a queue.
//!
//! One debugger serves one runtime: the PLC's, whose files are each a runtime file, or the field's, whose blocks sit in
//! one generated source at their spans (field.rs).

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use trust_runtime::debug::{
    DebugBreakpoint, DebugControl, SourceLocation, location_to_line_col,
    resolve_breakpoint_location,
};
use trust_runtime::harness::{TestHarness, parse_debug_expression};
use trust_runtime::memory::InstanceId;
use trust_runtime::value::{Value, format_user_value};

use crate::controller::Target;
pub use crate::debugger_kinds::{Breakpoint, Hit, HitOp, Log};
use crate::debugger_kinds::{compile, message};
use crate::watch::{scalar, value_from};

/// Where the program is held: a file, a 1-based line, the call stack innermost first, and in a block, the device whose
/// instance it is.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Hold {
    pub file: String,
    pub line: u32,
    pub frames: Vec<(String, u32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Resume {
    Continue,
    Over,
    Into,
    Out,
}

/// Writes made while held, for the engine to log at its next millisecond (spec decision 6).
pub(crate) type HeldWrites = Arc<Mutex<Vec<(Target, Value)>>>;

/// A file the user edits, inside a runtime file: its path, the runtime file's id, its first line there (0-based) and
/// its number of lines.
#[derive(Debug, Clone)]
pub(crate) struct Place {
    pub path: String,
    pub file_id: u32,
    pub first_line: u32,
    pub lines: u32,
}

/// A breakpoint as the user set it, and as truST takes it.
type Built = (Breakpoint, DebugBreakpoint);

/// An expression of the program read at the stop.
type Reader = dyn Fn(&DebugControl, &str) -> Option<Value> + Send + Sync;
type Parser = dyn Fn(&str) -> Result<trust_runtime::program_model::Expr, String> + Send + Sync;

#[derive(Clone)]
pub struct Debugger {
    control: DebugControl,
    places: Vec<Place>,
    /// Each runtime file's text and statements, by id.
    texts: Vec<String>,
    statements: Vec<Vec<SourceLocation>>,
    /// The field's devices and their blocks' instances; none for the PLC.
    devices: Vec<(String, InstanceId)>,
    /// The user's breakpoints by place, as set and as built for truST; Run to cursor adds one for a single stop.
    breakpoints: Arc<Mutex<BTreeMap<usize, Vec<Built>>>>,
    cursor: Arc<Mutex<Option<u32>>>,
    read: Arc<Reader>,
    parse: Arc<Parser>,
    /// The PLC's: where a write made while held goes. A block is read-only, so the field has none.
    writes: Option<HeldWrites>,
}

impl Debugger {
    /// The debugger of `harness`'s runtime, whose files are `texts` by id; `control` is attached to it.
    pub(crate) fn new(
        control: DebugControl,
        harness: &TestHarness,
        texts: Vec<String>,
        places: Vec<Place>,
        devices: Vec<(String, InstanceId)>,
        writes: Option<HeldWrites>,
    ) -> Self {
        let runtime = harness.runtime();
        let statements = (0..texts.len())
            .map(|id| {
                let id = u32::try_from(id).unwrap_or(u32::MAX);
                runtime
                    .statement_locations(id)
                    .map_or_else(Vec::new, <[_]>::to_vec)
            })
            .collect();
        let registry = Arc::new(Mutex::new(runtime.registry().clone()));
        let (profile, stdlib) = (runtime.profile(), runtime.stdlib().clone());
        let parsing = registry.clone();
        let parse = move |text: &str| {
            let mut registry = parsing.lock().map_err(|e| e.to_string())?;
            parse_debug_expression(text, &mut registry, profile, &[]).map_err(|e| e.to_string())
        };
        let read = move |control: &DebugControl, text: &str| {
            let mut registry = registry.lock().ok()?;
            let expr = parse_debug_expression(text, &mut registry, profile, &[]).ok()?;
            let value = control.with_snapshot(|snapshot| {
                snapshot.evaluate_expression(&expr, &registry, Some(&stdlib), &profile, None)
            })?;
            value.ok()
        };
        Self {
            control,
            places,
            texts,
            statements,
            devices,
            breakpoints: Arc::default(),
            cursor: Arc::default(),
            read: Arc::new(read),
            parse: Arc::new(parse),
            writes,
        }
    }

    /// The places of a runtime whose every file is one place: the PLC's.
    pub(crate) fn whole_files(files: &[(String, String)]) -> Vec<Place> {
        (files.iter().enumerate())
            .map(|(id, (path, text))| Place {
                path: path.clone(),
                file_id: u32::try_from(id).unwrap_or(u32::MAX),
                first_line: 0,
                lines: u32::try_from(text.lines().count()).unwrap_or(u32::MAX),
            })
            .collect()
    }

    fn place(&self, path: &str) -> Result<usize, String> {
        (self.places.iter().position(|p| p.path == path)).ok_or_else(|| {
            let runtime = if self.writes.is_some() {
                "program"
            } else {
                "field"
            };
            format!("{path} is not a file of the {runtime}")
        })
    }

    /// The statement on 1-based `line` of place `at`, or the nearest one in it when the line has none
    /// (`resolve_breakpoint_location`, searched from the line's start).
    fn statement(&self, at: usize, line: u32) -> Option<SourceLocation> {
        let place = &self.places[at];
        let row = place.first_line + line.checked_sub(1)?;
        let (id, text) = (place.file_id as usize, &self.texts[place.file_id as usize]);
        let found = resolve_breakpoint_location(text, place.file_id, &self.statements[id], row, 0)?;
        (self.line_of(&found)?.0 == place.path).then_some(found)
    }

    fn line_of(&self, location: &SourceLocation) -> Option<(String, u32)> {
        let text = self.texts.get(usize::try_from(location.file_id).ok()?)?;
        let row = location_to_line_col(text, location).0;
        let place = (self.places.iter()).find(|p| {
            p.file_id == location.file_id && (p.first_line..p.first_line + p.lines).contains(&row)
        })?;
        Some((place.path.clone(), row - place.first_line + 1))
    }

    /// Arms runtime file `file_id` with every place's breakpoints in it, and `extra`.
    fn arm(&self, file_id: u32, extra: Option<DebugBreakpoint>) {
        let kept = self
            .breakpoints
            .lock()
            .map(|b| b.clone())
            .unwrap_or_default();
        let mut armed: Vec<DebugBreakpoint> = (kept.iter())
            .filter(|(at, _)| self.places[**at].file_id == file_id)
            .flat_map(|(_, set)| set.iter().map(|(_, built)| built.clone()))
            .collect();
        armed.extend(extra);
        self.control.set_breakpoints_for_file(file_id, armed);
    }

    /// Sets `path`'s breakpoints, each moved to its statement's line, a line without one dropped, and gives them as
    /// set.
    pub fn set(&self, path: &str, breakpoints: &[Breakpoint]) -> Result<Vec<Breakpoint>, String> {
        let at = self.place(path)?;
        let mut set: Vec<Built> = Vec::new();
        for breakpoint in breakpoints {
            let Some(location) = self.statement(at, breakpoint.line) else {
                continue;
            };
            let line = self.line_of(&location).map_or(breakpoint.line, |(_, l)| l);
            if set.iter().any(|(b, _)| b.line == line) {
                continue;
            }
            let moved = Breakpoint {
                line,
                ..breakpoint.clone()
            };
            let built = compile(&moved, location, &mut |t| (self.parse)(t))?;
            set.push((moved, built));
        }
        set.sort_by_key(|(b, _)| b.line);
        let shown = set.iter().map(|(b, _)| b.clone()).collect();
        self.breakpoints
            .lock()
            .map_err(|e| e.to_string())?
            .insert(at, set);
        self.arm(self.places[at].file_id, None);
        Ok(shown)
    }

    /// Runs to `line` of `path`: a breakpoint for one stop, then the user's breakpoints as they were.
    pub fn run_to(&self, path: &str, line: u32) -> Result<(), String> {
        let at = self.place(path)?;
        let location = (self.statement(at, line))
            .ok_or_else(|| format!("line {line} of {path} has no statement"))?;
        let file_id = self.places[at].file_id;
        self.arm(file_id, Some(DebugBreakpoint::new(location)));
        *self.cursor.lock().map_err(|e| e.to_string())? = Some(file_id);
        self.control.continue_run();
        Ok(())
    }

    /// Where the program is held, or `None` while it runs. A stop after Run to cursor gives back the user's
    /// breakpoints.
    pub fn held(&self) -> Option<Hold> {
        if !self.control.is_paused() {
            return None;
        }
        if let Some(file_id) = self.cursor.lock().ok().and_then(|mut c| c.take()) {
            self.arm(file_id, None);
        }
        let (file, line) = self.line_of(&self.control.last_location()?)?;
        let mut frames: Vec<_> = self.control.frame_locations().into_iter().collect();
        frames.sort_by_key(|(id, _)| std::cmp::Reverse(id.0));
        let frames = frames
            .iter()
            .filter_map(|(_, loc)| self.line_of(loc))
            .collect();
        Some(Hold {
            file,
            line,
            frames,
            device: self.device(),
        })
    }

    /// The device whose block instance the stopped frame runs in.
    fn device(&self) -> Option<String> {
        let instance = self.control.with_snapshot(|snapshot| {
            snapshot
                .storage
                .current_frame()
                .and_then(|frame| frame.instance_id)
        })??;
        (self.devices.iter())
            .find(|(_, id)| *id == instance)
            .map(|(device, _)| device.clone())
    }

    pub fn resume(&self, how: Resume) {
        match how {
            Resume::Continue => self.control.continue_run(),
            Resume::Over => self.control.step_over(),
            Resume::Into => self.control.step(),
            Resume::Out => self.control.step_out(),
        }
    }

    /// Each of `texts`, an expression of the program, as the stopped statement sees it; `None` while it runs, and for
    /// what does not resolve or has no single value.
    pub fn read(&self, texts: &[String]) -> Vec<Option<String>> {
        // The snapshot outlives its stop: while running it would show the last stop's values.
        let paused = self.control.is_paused();
        (texts.iter())
            .map(|text| {
                let value = paused.then(|| (self.read)(&self.control, text)).flatten();
                value.and_then(scalar).map(|v| format_user_value(&v))
            })
            .collect()
    }

    /// Writes `text` to the program variable at `path` (`Main.count`, `Main.carry.PT`) while held (decision 5). The
    /// value shows in the snapshot at once and reaches the program at the next cycle, as truST queues it; the engine
    /// logs it at its next millisecond, where a replay gives the same trajectory (measured).
    pub fn write(&self, path: &str, text: &str) -> Result<(), String> {
        let writes = self.writes.as_ref().ok_or("a block is read-only")?;
        if !self.control.is_paused() {
            return Err("the PLC is not held".into());
        }
        let unknown = || format!("{path} is not a variable of the PLC");
        let (owner, name) = path.rsplit_once('.').ok_or_else(unknown)?;
        let (program, member) = path.split_once('.').ok_or_else(unknown)?;
        let Some(Value::Instance(id)) = (self.read)(&self.control, owner) else {
            return Err(unknown());
        };
        let current = (self.control)
            .with_snapshot(|s| s.storage.get_instance_var(id, name).cloned())
            .flatten()
            .ok_or_else(unknown)?;
        let current =
            scalar(current).ok_or_else(|| format!("{path} has no single value to write"))?;
        let value = value_from(|t| (self.read)(&self.control, t), text, &current)?;
        (self.control)
            .try_enqueue_instance_write(id, name, value.clone())
            .map_err(|e| format!("{path}: {e:?}"))?;
        (self.control).with_snapshot(|s| s.storage.set_instance_var(id, name, value.clone()));
        writes
            .lock()
            .map_err(|e| e.to_string())?
            .push((Target::variable(program, member), value));
        Ok(())
    }

    /// The lines logpoints wrote since the last call, where they are.
    pub fn logs(&self) -> Vec<Log> {
        (self.control.drain_logs().into_iter())
            .filter_map(|log| {
                let (file, line) = self.line_of(&log.location?)?;
                Some(Log {
                    file,
                    line,
                    message: message(&log.message),
                })
            })
            .collect()
    }

    /// No breakpoint left and the scan running: what Reset, Build, Restore and Pause do first (decision 9).
    pub fn let_go(&self) {
        self.control.clear_breakpoints();
        if let Ok(mut breakpoints) = self.breakpoints.lock() {
            breakpoints.clear();
        }
        self.control.continue_run();
    }
}

#[cfg(test)]
#[path = "debugger_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "debugger_kinds_tests.rs"]
mod kinds_tests;
