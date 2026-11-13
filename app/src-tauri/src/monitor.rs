//! Online monitoring in the ST editor (spec 2026-11-13-online-monitoring-design.md, decisions 4 to 7): the lines in
//! view become the paths and conditions the engine reads after each scan, and the user's writes and forces reach the
//! PLC. The references come from truST's frontend at v0.39.0 (`trust_ide::inline_values`), on the files of the built
//! project, so a value always belongs to the program that runs: an edited tab is not monitored until it is built.
//!
//! An FB member is shown whole (`entered.Q`): the inline values give its instance only, so the path climbs the syntax
//! tree's `FieldExpr`s. A condition is the first expression of an IF, ELSIF or WHILE, or the last of a REPEAT's UNTIL:
//! truST's parser has no node of its own for it (measured 2026-11-13). Its references are qualified as paths are, since
//! truST evaluates without a program scope. A node's range takes in the white space around it, so positions use the
//! range of its tokens.

use std::collections::BTreeSet;

use serde::Serialize;
use tauri::State;
use text_size::{TextRange, TextSize};
use trust_hir::project::{Project as Frontend, SourceKey};
use trust_ide::inline_values::{InlineValueScope, InlineValueTarget, inline_value_data};
use trust_syntax::parser::parse;
use trust_syntax::syntax::{SyntaxKind, SyntaxNode};
use twinova_engine::controller::Target;
use twinova_engine::engine::Engine;
use twinova_engine::watch::{Scope, Watch};

use crate::analysis::sources;
use crate::commands::{AppState, with_engine};
use crate::declarations::declarations;

/// Where a value is drawn and what it shows.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Spot {
    /// The 1-based line, and the 1-based UTF-16 column after which the value is drawn.
    pub line: u32,
    pub column: u32,
    /// A condition's TRUE or FALSE, or a reference's value.
    pub condition: bool,
    /// What a click writes or forces: a program's variable or a member of one of its instances; a block's file is
    /// read-only. A reference climbs through members only, so an indexed array reads and targets the whole array.
    pub target: Option<Target>,
}

/// How a file's references are qualified for the runtime.
enum Owner<'a> {
    /// A program file: each reference by its program's instance, `Main.count`.
    Program,
    /// A block's file, for one device's instance in the field: `F.belt_a.motor`.
    Device(&'a str),
}

/// The 1-based line and UTF-16 column just after byte `offset` of `text`.
fn position(text: &str, offset: usize) -> (u32, u32) {
    let before = &text[..offset];
    let line_start = before.rfind('\n').map_or(0, |n| n + 1);
    let line = before.matches('\n').count() as u32 + 1;
    let column = before[line_start..].encode_utf16().count() as u32 + 1;
    (line, column)
}

/// The byte range of 1-based lines `first` to `last` of `text`, both whole.
fn lines(text: &str, first: u32, last: u32) -> TextRange {
    let mut starts = std::iter::once(0).chain(text.match_indices('\n').map(|(n, _)| n + 1));
    let start = starts
        .nth(first.saturating_sub(1) as usize)
        .unwrap_or(text.len());
    let end = (text.match_indices('\n').map(|(n, _)| n))
        .nth(last.saturating_sub(1) as usize)
        .unwrap_or(text.len());
    let at = |n: usize| TextSize::from(n as u32);
    TextRange::new(at(start), at(end.max(start)))
}

/// The reference at `range`, climbed through the `FieldExpr`s it is the base of: `entered` becomes `entered.Q`.
fn whole(root: &SyntaxNode, range: TextRange) -> Option<SyntaxNode> {
    let token = root.covering_element(range).into_token()?;
    let mut node = token.parent()?;
    while let Some(parent) = node.parent().filter(|p| p.kind() == SyntaxKind::FieldExpr) {
        let is_base = parent.first_child().is_some_and(|base| base == node);
        if !is_base {
            break;
        }
        node = parent;
    }
    Some(node)
}

fn compact(node: &SyntaxNode) -> String {
    node.text().to_string().split_whitespace().collect()
}

/// `node`'s range without the white space and comments at its ends.
fn trimmed(node: &SyntaxNode) -> TextRange {
    let mut tokens = (node.descendants_with_tokens())
        .filter_map(|e| e.into_token())
        .filter(|t| !t.kind().is_trivia())
        .map(|t| t.text_range());
    let first = tokens.next().unwrap_or_else(|| node.text_range());
    let last = tokens.last().unwrap_or(first);
    first.cover(last)
}

/// The condition of a statement that has one: the first expression of an IF, ELSIF or WHILE, the last of a REPEAT.
fn condition_of(node: &SyntaxNode) -> Option<SyntaxNode> {
    let mut expressions = node.children().filter(|c| c.kind().is_expression_node());
    match node.kind() {
        SyntaxKind::IfStmt | SyntaxKind::ElsifBranch | SyntaxKind::WhileStmt => expressions.next(),
        SyntaxKind::RepeatStmt => expressions.last(),
        _ => None,
    }
}

/// The prefix that makes `target` a runtime path, or `None` when this file's references cannot be read.
fn qualifier(owner: &Owner, target: &InlineValueTarget) -> Option<(Scope, String)> {
    match (owner, target.scope) {
        (Owner::Program, InlineValueScope::Local) => {
            Some((Scope::Plc, format!("{}.", target.owner.as_ref()?)))
        }
        (Owner::Program, InlineValueScope::Global | InlineValueScope::Retain) => {
            Some((Scope::Plc, String::new()))
        }
        (Owner::Device(device), InlineValueScope::Local) => {
            Some((Scope::Field, format!("F.{device}.")))
        }
        (Owner::Device(_), _) => None,
    }
}

/// What lines `first` to `last` of `file` show, in drawing order: each reference's value once per line, and each
/// condition's TRUE or FALSE. `files` are the built project's, as `analysis::sources` gives them.
pub fn spots(
    files: &[(String, String)],
    file: &str,
    first: u32,
    last: u32,
    device: Option<&str>,
) -> Vec<(Spot, Watch)> {
    let Some((_, text)) = files.iter().find(|(f, _)| f == file) else {
        return Vec::new();
    };
    let owner = match (file.starts_with("components/"), device) {
        (false, _) => Owner::Program,
        (true, Some(device)) => Owner::Device(device),
        (true, None) => return Vec::new(),
    };
    let mut frontend = Frontend::new();
    for (path, source) in files {
        frontend.set_source_text(SourceKey::from_virtual(path.clone()), source.clone());
    }
    let id = frontend.file_id_for_key(&SourceKey::from_virtual(file.to_string()));
    let range = lines(text, first, last);
    let targets = id.map_or_else(Vec::new, |id| {
        inline_value_data(frontend.database(), id, range).targets
    });
    let root = parse(text).syntax();
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for target in &targets {
        let (Some((scope, prefix)), Some(node)) =
            (qualifier(&owner, target), whole(&root, target.range))
        else {
            continue;
        };
        let path = compact(&node);
        let (line, column) = position(text, usize::from(trimmed(&node).end()));
        if !seen.insert((line, path.clone())) {
            continue;
        }
        let own = matches!(owner, Owner::Program);
        let program = target.owner.as_deref().unwrap_or_default();
        let spot = Spot {
            line,
            column,
            condition: false,
            target: own.then(|| Target::variable(program, &path)),
        };
        out.push((spot, Watch::Path(scope, format!("{prefix}{path}"))));
    }
    for (spot, watch) in conditions(&root, text, range, &targets, &owner) {
        // A condition that is one reference (`IF entered.Q THEN`) is that reference's value, coloured as a condition.
        let same = |(s, w): &&mut (Spot, Watch)| match (w, &watch) {
            (Watch::Path(_, p), Watch::Condition(_, c)) => {
                (s.line, s.column) == (spot.line, spot.column) && p == c
            }
            _ => false,
        };
        match out.iter_mut().find(same) {
            Some((path, _)) => path.condition = true,
            None => out.push((spot, watch)),
        }
    }
    out.sort_by_key(|(spot, _)| (spot.line, spot.column));
    out
}

/// Each condition in `range`, its references qualified in place.
fn conditions(
    root: &SyntaxNode,
    text: &str,
    range: TextRange,
    targets: &[InlineValueTarget],
    owner: &Owner,
) -> Vec<(Spot, Watch)> {
    let within = |r: TextRange| range.contains_range(r);
    (root.descendants())
        .filter_map(|n| condition_of(&n))
        .filter(|n| within(trimmed(n)))
        .filter_map(|node| {
            let span = trimmed(&node);
            let mut condition = text[span].to_string();
            let mut scope = Scope::Plc;
            let inside = targets.iter().filter(|t| span.contains_range(t.range));
            // From the last reference back, so each earlier insertion point stays where it was.
            for target in inside.collect::<Vec<_>>().into_iter().rev() {
                let (s, prefix) = qualifier(owner, target)?;
                scope = s;
                let at = usize::from(target.range.start() - span.start());
                condition.insert_str(at, &prefix);
            }
            let (line, column) = position(text, usize::from(span.end()));
            let spot = Spot {
                line,
                column,
                condition: true,
                target: None,
            };
            Some((
                spot,
                Watch::Condition(
                    scope,
                    condition.split_whitespace().collect::<Vec<_>>().join(" "),
                ),
            ))
        })
        .collect()
}

/// A row of the variables table that the engine reads: a declaration, or a member of an unfolded instance.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Row {
    /// The declaration's name, or `instance.member`.
    pub name: String,
    /// The declaration's index in `declarations::declarations`, for a declaration's row.
    pub declaration: Option<usize>,
    /// What writing or forcing the row changes; none in a block's file, which is read-only.
    pub target: Option<Target>,
    /// Whether the row is an instance, which unfolds into its members.
    pub instance: bool,
}

/// What the editor monitors, kept for a held line (spec 2026-11-13-breakpoints-hold-design.md, decision 6): its watches
/// in the values' order, and the table's rows with theirs, which need the engine to be listed again.
#[derive(Default)]
pub(crate) struct Monitored {
    watches: Vec<Watch>,
    table: Vec<(Row, Watch)>,
}

/// What the editor shows of a file while monitoring: the spots in the code, then the table's rows, the values coming
/// in that order.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Watched {
    pub spots: Vec<Spot>,
    pub rows: Vec<Row>,
}

/// The table's rows of `text` with their watches: each declaration, and the members of each instance in `unfolded`,
/// read from the runtime (`Engine::members`), so a standard block's members come too.
fn rows(engine: &mut Engine, text: &str, owner: &Owner, unfolded: &[String]) -> Vec<(Row, Watch)> {
    let mut out = Vec::new();
    for (index, decl) in declarations(text).into_iter().enumerate() {
        let (scope, prefix) = match owner {
            Owner::Program => (Scope::Plc, format!("{}.", decl.pou)),
            Owner::Device(device) => (Scope::Field, format!("F.{device}.")),
        };
        let target =
            |name: &str| matches!(owner, Owner::Program).then(|| Target::variable(&decl.pou, name));
        let path = format!("{prefix}{}", decl.name);
        let members = engine.members(scope, &path);
        let row = Row {
            target: target(&decl.name).filter(|_| members.is_empty()),
            name: decl.name.clone(),
            declaration: Some(index),
            instance: !members.is_empty(),
        };
        out.push((row, Watch::Path(scope, path.clone())));
        if !unfolded.contains(&decl.name) {
            continue;
        }
        for member in members {
            let name = format!("{}.{member}", decl.name);
            let row = Row {
                target: target(&name),
                name,
                declaration: None,
                instance: false,
            };
            out.push((row, Watch::Path(scope, format!("{path}.{member}"))));
        }
    }
    out
}

/// Monitors lines `first` to `last` of `file`, a block's file for `device`, and its variables table with the instances
/// in `unfolded` unfolded: the engine reads them after every scan from now on, and the Run loop sends their values as
/// `monitor`, the spots' first, then the rows'.
#[tauri::command]
pub fn watch_range(
    state: State<'_, AppState>,
    file: String,
    first: u32,
    last: u32,
    device: Option<String>,
    unfolded: Vec<String>,
) -> Result<Watched, String> {
    let project = state.built_project()?.ok_or("build the project first")?;
    let files = sources(&project)?;
    let found = spots(&files, &file, first, last, device.as_deref());
    let text = files
        .iter()
        .find(|(f, _)| *f == file)
        .map(|(_, t)| t.clone());
    let owner = match (file.starts_with("components/"), device.as_deref()) {
        (false, _) => Some(Owner::Program),
        (true, Some(device)) => Some(Owner::Device(device)),
        (true, None) => None,
    };
    let mut monitored = state.monitored.lock().map_err(|e| e.to_string())?;
    // While held, the engine is in its scan: the spots are found in the text and the table stays as it was.
    let held = state.held();
    let table = match held {
        Some(_) => monitored.table.clone(),
        None => with_engine(&state, |e| {
            Ok(match (text, owner) {
                (Some(text), Some(owner)) => rows(e, &text, &owner, &unfolded),
                _ => Vec::new(),
            })
        })?,
    };
    let watches: Vec<Watch> = (found.iter().map(|(_, w)| w.clone()))
        .chain(table.iter().map(|(_, w)| w.clone()))
        .collect();
    if held.is_none() {
        with_engine(&state, |e| {
            e.watch(&watches);
            Ok(())
        })?;
    }
    let rows = table.iter().map(|(r, _)| r.clone()).collect();
    *monitored = Monitored { watches, table };
    Ok(Watched {
        spots: found.into_iter().map(|(s, _)| s).collect(),
        rows,
    })
}

/// Monitoring off: the engine reads nothing.
#[tauri::command]
pub fn unwatch(state: State<'_, AppState>) -> Result<(), String> {
    *state.monitored.lock().map_err(|e| e.to_string())? = Monitored::default();
    if state.held().is_some() {
        return Ok(());
    }
    with_engine(&state, |e| {
        e.watch(&[]);
        Ok(())
    })
}

/// The monitored values now, for a paused or stepped line, or as the held statement sees them: only the held runtime's,
/// the PLC's or the field's, since the other's is not stopped at a statement.
#[tauri::command]
pub fn watched(state: State<'_, AppState>) -> Result<Vec<Option<String>>, String> {
    let Some((held, debugger)) = state.held() else {
        return with_engine(&state, |e| Ok(e.watched()));
    };
    let monitored = state.monitored.lock().map_err(|e| e.to_string())?;
    Ok((monitored.watches.iter())
        .map(|w| match w {
            Watch::Path(scope, text) | Watch::Condition(scope, text) if *scope == held => {
                debugger.read(std::slice::from_ref(text)).pop().flatten()
            }
            _ => None,
        })
        .collect())
}

/// A force in effect, as the Watch pane lists it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Force {
    pub target: Target,
    pub label: String,
    pub value: String,
}

fn listed(forces: Vec<(Target, String)>) -> Vec<Force> {
    (forces.into_iter())
        .map(|(target, value)| Force {
            label: target.to_string(),
            target,
            value,
        })
        .collect()
}

#[tauri::command]
pub fn forces(state: State<'_, AppState>) -> Result<Vec<Force>, String> {
    with_engine(&state, |e| Ok(listed(e.forces())))
}

/// Holds `target` at `text`'s value; returns the forces in effect.
#[tauri::command]
pub fn force(
    state: State<'_, AppState>,
    target: Target,
    text: String,
) -> Result<Vec<Force>, String> {
    with_engine(&state, |e| {
        e.force(target, &text)?;
        Ok(listed(e.forces()))
    })
}

#[tauri::command]
pub fn release(state: State<'_, AppState>, target: Target) -> Result<Vec<Force>, String> {
    with_engine(&state, |e| {
        e.release(target)?;
        Ok(listed(e.forces()))
    })
}

/// Writes `text` into a program variable once, at the next scan.
#[tauri::command]
pub fn write_value(state: State<'_, AppState>, target: Target, text: String) -> Result<(), String> {
    // While the PLC is held, its debugger writes and the engine logs it at its next millisecond (spec
    // 2026-11-13-breakpoints-complete-design.md, decisions 5 and 6).
    if let (Some((Scope::Plc, debugger)), Target::Variable { program, path }) =
        (state.held(), &target)
    {
        return debugger.write(&format!("{program}.{path}"), &text);
    }
    with_engine(&state, |e| e.write(target, &text))
}

#[cfg(test)]
#[path = "monitor_tests.rs"]
mod tests;
