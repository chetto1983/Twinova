//! Breakpoints that hold the line (spec 2026-11-13-breakpoints-hold-design.md). At a stop, truST's hook blocks the
//! scanning thread until it is resumed (`host/debug/control/hook.rs`), so the engine, the plant and virtual time stand:
//! a PLC in HOLD. A `Debugger` therefore works without the engine, which that thread holds while it is stopped: it
//! resolves lines from statement locations copied at attach, and reads values from the paused snapshot.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use trust_runtime::debug::{
    DebugBreakpoint, DebugControl, SourceLocation, location_to_line_col,
    resolve_breakpoint_location,
};
use trust_runtime::harness::{TestHarness, parse_debug_expression};
use trust_runtime::value::format_user_value;

use crate::watch::scalar;

/// Where the program is held: a file of the program, a 1-based line, and the call stack, innermost first.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Hold {
    pub file: String,
    pub line: u32,
    pub frames: Vec<(String, u32)>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Resume {
    Continue,
    Over,
    Into,
    Out,
}

/// An expression of the program read at the stop, formatted as the editor shows it.
type Reader = dyn Fn(&DebugControl, &str) -> Option<String> + Send + Sync;

#[derive(Clone)]
pub struct Debugger {
    control: DebugControl,
    /// The program's files in Build's order: a file's place is its id in the runtime.
    files: Vec<(String, String)>,
    statements: Vec<Vec<SourceLocation>>,
    /// The user's breakpoints by file, as lines; Run to cursor adds one for a single stop.
    lines: Arc<Mutex<BTreeMap<usize, Vec<u32>>>>,
    cursor: Arc<Mutex<Option<usize>>>,
    read: Arc<Reader>,
}

impl Debugger {
    /// The debugger of `harness`'s program, made of `files` in Build's order; `control` is attached to it.
    pub(crate) fn new(
        control: DebugControl,
        harness: &TestHarness,
        files: Vec<(String, String)>,
    ) -> Self {
        let runtime = harness.runtime();
        let statements = (0..files.len())
            .map(|id| {
                let id = u32::try_from(id).unwrap_or(u32::MAX);
                runtime
                    .statement_locations(id)
                    .map_or_else(Vec::new, <[_]>::to_vec)
            })
            .collect();
        let registry = Mutex::new(runtime.registry().clone());
        let (profile, stdlib) = (runtime.profile(), runtime.stdlib().clone());
        let read = move |control: &DebugControl, text: &str| {
            let mut registry = registry.lock().ok()?;
            let expr = parse_debug_expression(text, &mut registry, profile, &[]).ok()?;
            let value = control.with_snapshot(|snapshot| {
                snapshot.evaluate_expression(&expr, &registry, Some(&stdlib), &profile, None)
            })?;
            value.ok().and_then(scalar).map(|v| format_user_value(&v))
        };
        Self {
            control,
            files,
            statements,
            lines: Arc::default(),
            cursor: Arc::default(),
            read: Arc::new(read),
        }
    }

    fn file(&self, path: &str) -> Result<usize, String> {
        (self.files.iter().position(|(p, _)| p == path))
            .ok_or_else(|| format!("{path} is not a file of the program"))
    }

    /// The statement on 1-based `line` of file `id`, or the nearest one when the line has none
    /// (`resolve_breakpoint_location`, searched from the line's start).
    fn statement(&self, id: usize, line: u32) -> Option<SourceLocation> {
        let file_id = u32::try_from(id).ok()?;
        let (text, statements) = (&self.files[id].1, &self.statements[id]);
        resolve_breakpoint_location(text, file_id, statements, line.checked_sub(1)?, 0)
    }

    fn line_of(&self, location: &SourceLocation) -> Option<(String, u32)> {
        let (path, text) = self.files.get(usize::try_from(location.file_id).ok()?)?;
        Some((path.clone(), location_to_line_col(text, location).0 + 1))
    }

    /// Arms file `id`'s breakpoints at `lines`, each moved to its statement's line, and gives those lines.
    fn arm(&self, id: usize, lines: &[u32]) -> Vec<u32> {
        let mut resolved: Vec<(u32, SourceLocation)> = (lines.iter())
            .filter_map(|&line| self.statement(id, line))
            .filter_map(|loc| Some((self.line_of(&loc)?.1, loc)))
            .collect();
        resolved.sort_by_key(|(line, _)| *line);
        resolved.dedup_by_key(|(line, _)| *line);
        let breakpoints = resolved
            .iter()
            .map(|(_, loc)| DebugBreakpoint::new(*loc))
            .collect();
        self.control
            .set_breakpoints_for_file(u32::try_from(id).unwrap_or(u32::MAX), breakpoints);
        resolved.into_iter().map(|(line, _)| line).collect()
    }

    /// Sets `path`'s breakpoints to `lines`, each moved to its statement's line; a line without one is dropped. Gives
    /// the lines set.
    pub fn set(&self, path: &str, lines: &[u32]) -> Result<Vec<u32>, String> {
        let id = self.file(path)?;
        let set = self.arm(id, lines);
        self.lines
            .lock()
            .map_err(|e| e.to_string())?
            .insert(id, set.clone());
        Ok(set)
    }

    /// Runs to `line` of `path`: a breakpoint for one stop, then the user's breakpoints as they were.
    pub fn run_to(&self, path: &str, line: u32) -> Result<(), String> {
        let id = self.file(path)?;
        self.statement(id, line)
            .ok_or_else(|| format!("line {line} of {path} has no statement"))?;
        let mut lines = self.user_lines(id);
        lines.push(line);
        self.arm(id, &lines);
        *self.cursor.lock().map_err(|e| e.to_string())? = Some(id);
        self.control.continue_run();
        Ok(())
    }

    fn user_lines(&self, id: usize) -> Vec<u32> {
        (self.lines.lock().ok())
            .and_then(|lines| lines.get(&id).cloned())
            .unwrap_or_default()
    }

    /// Where the program is held, or `None` while it runs. A stop after Run to cursor gives back the user's
    /// breakpoints.
    pub fn held(&self) -> Option<Hold> {
        if !self.control.is_paused() {
            return None;
        }
        if let Some(id) = self.cursor.lock().ok().and_then(|mut c| c.take()) {
            self.arm(id, &self.user_lines(id));
        }
        let (file, line) = self.line_of(&self.control.last_location()?)?;
        let mut frames: Vec<_> = self.control.frame_locations().into_iter().collect();
        frames.sort_by_key(|(id, _)| std::cmp::Reverse(id.0));
        let frames = frames
            .iter()
            .filter_map(|(_, loc)| self.line_of(loc))
            .collect();
        Some(Hold { file, line, frames })
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
            .map(|text| paused.then(|| (self.read)(&self.control, text)).flatten())
            .collect()
    }

    /// No breakpoint left and the scan running: what Reset, Build, Restore and Pause do first (decision 9).
    pub fn let_go(&self) {
        self.control.clear_breakpoints();
        if let Ok(mut lines) = self.lines.lock() {
            lines.clear();
        }
        self.control.continue_run();
    }
}

#[cfg(test)]
#[path = "debugger_tests.rs"]
mod tests;
