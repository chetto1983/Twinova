//! The in-app ST editor's analysis (spec 2026-11-12-st-editor-design.md): truST's language frontend at v0.39.0 reads
//! the program, the blocks and the field's generated configuration as one project, as VS Code's workspace does (decision
//! 6), and answers diagnostics, hover and completion. The diagnostics are filtered as Twinova's `trust-lsp.toml` filters
//! VS Code's, and the false E202 of a block variable the configuration places are left out (decision 3). Build stays the
//! referee: the runtime compiles with the frontend at its pin.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use serde::Serialize;
use tauri::State;
use trust_wasm_analysis::{
    BrowserAnalysisEngine, CompletionRequest, DiagnosticItem, DocumentInput, HoverRequest, Position,
};
use twinova_engine::field::{config_source, placed_variables};
use twinova_project::Project;

use crate::lsp_files::FIELD;
use crate::project_commands::ProjectState;

/// The analysis engine and the documents it holds.
#[derive(Default)]
pub struct Analysis {
    engine: BrowserAnalysisEngine,
    documents: Vec<DocumentInput>,
}

#[derive(Default)]
pub struct AnalysisState(Mutex<Analysis>);

impl Analysis {
    fn replace(&mut self, documents: Vec<DocumentInput>) -> Result<(), String> {
        (self.engine.replace_documents(documents.clone())).map_err(|e| e.to_string())?;
        self.documents = documents;
        Ok(())
    }

    /// Takes `text` as `file`'s, when it changed since the last analysis: hover and completion answer on what the editor
    /// holds now, not on the texts of the analysis debounced 300 ms behind the typing (decision 6).
    fn with_text(&mut self, file: &str, text: String) -> Result<(), String> {
        let mut documents = self.documents.clone();
        let Some(document) = documents.iter_mut().find(|d| d.uri == file) else {
            return Err(format!("{file} is not analysed"));
        };
        if document.text == text {
            return Ok(());
        }
        document.text = text;
        self.replace(documents)
    }
}

/// A diagnostic as the editor shows it: 1-based lines, and columns in UTF-16 units from 1, as Monaco counts them (the
/// engine counts both from 0, the column in UTF-16 units: `trust-wasm-analysis`, `offset_to_position`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Problem {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

/// The nondeterminism warnings Twinova's `trust-lsp.toml` turns off in VS Code (`lsp_files.rs`).
const HIDDEN: [&str; 2] = ["W010", "W011"];
const UNMAPPED: &str = "E202";

/// The file of a behaviour block, as the project folder holds it.
fn block_file(key: &str) -> String {
    format!("components/{key}.st")
}

/// The diagnostics the editor shows of `file`: none of `HIDDEN`, and no E202 on a variable of `placed`, the block
/// variables the field's configuration places.
fn shown(diagnostics: Vec<DiagnosticItem>, text: &str, placed: &BTreeSet<String>) -> Vec<Problem> {
    let named = |d: &DiagnosticItem| text.get(d.span.start as usize..d.span.end as usize);
    (diagnostics.into_iter())
        .filter(|d| !HIDDEN.contains(&d.code.as_str()))
        .filter(|d| d.code != UNMAPPED || !named(d).is_some_and(|name| placed.contains(name)))
        .map(|d| Problem {
            line: d.range.start.line + 1,
            column: d.range.start.character + 1,
            end_line: d.range.end.line + 1,
            end_column: d.range.end.character + 1,
            severity: d.severity,
            code: d.code,
            message: d.message,
        })
        .collect()
}

/// Analyses `project`'s sources: each shown file's problems, by path.
pub fn analyze_project(
    analysis: &mut Analysis,
    project: &Project,
) -> Result<BTreeMap<String, Vec<Problem>>, String> {
    let shown_files: Vec<(String, String)> = (project.program.iter().cloned())
        .chain(
            project
                .behaviours
                .iter()
                .map(|(k, t)| (block_file(k), t.clone())),
        )
        .collect();
    let mut documents: Vec<DocumentInput> = (shown_files.iter())
        .map(|(uri, text)| DocumentInput {
            uri: uri.clone(),
            text: text.clone(),
        })
        .collect();
    if let Some(config) = config_source(project)? {
        documents.push(DocumentInput {
            uri: FIELD.into(),
            text: config,
        });
    }
    analysis.replace(documents)?;
    let placed = placed_variables(project);
    let none = BTreeSet::new();
    let placed_in = |file: &str| {
        let key = file
            .strip_prefix("components/")
            .and_then(|f| f.strip_suffix(".st"));
        key.and_then(|k| placed.get(k)).unwrap_or(&none)
    };
    (shown_files.iter())
        .map(|(file, text)| {
            let found = (analysis.engine.diagnostics(file)).map_err(|e| e.to_string())?;
            Ok((file.clone(), shown(found, text, placed_in(file))))
        })
        .collect()
}

/// The engine's position of the editor's 1-based `line` and `column`.
fn position(line: u32, column: u32) -> Position {
    Position {
        line: line.saturating_sub(1),
        character: column.saturating_sub(1),
    }
}

/// A completion as the editor offers it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Completion {
    pub label: String,
    pub kind: String,
    pub detail: Option<String>,
    pub insert_text: String,
}

/// The current project's diagnostics with the editor's texts (decision 6).
#[tauri::command]
pub fn analyze(
    state: State<'_, AnalysisState>,
    projects: State<'_, ProjectState>,
    program: Vec<(String, String)>,
    behaviours: BTreeMap<String, String>,
) -> Result<BTreeMap<String, Vec<Problem>>, String> {
    let project = Project {
        program,
        behaviours,
        ..projects.project()?
    };
    let mut analysis = state.0.lock().map_err(|_| "analysis lock poisoned")?;
    analyze_project(&mut analysis, &project)
}

/// What a symbol of `file` is, in Markdown, `text` being the file as the editor holds it now.
#[tauri::command]
pub fn hover(
    state: State<'_, AnalysisState>,
    file: String,
    text: String,
    line: u32,
    column: u32,
) -> Result<Option<String>, String> {
    let mut analysis = state.0.lock().map_err(|_| "analysis lock poisoned")?;
    analysis.with_text(&file, text)?;
    let request = HoverRequest {
        uri: file,
        position: position(line, column),
    };
    Ok((analysis.engine)
        .hover(request)
        .map_err(|e| e.to_string())?
        .map(|h| h.contents))
}

/// What may be typed at a place of `file`, `text` being the file as the editor holds it now.
#[tauri::command]
pub fn completion(
    state: State<'_, AnalysisState>,
    file: String,
    text: String,
    line: u32,
    column: u32,
) -> Result<Vec<Completion>, String> {
    let mut analysis = state.0.lock().map_err(|_| "analysis lock poisoned")?;
    analysis.with_text(&file, text)?;
    let request = CompletionRequest {
        uri: file,
        position: position(line, column),
        limit: None,
    };
    let items = (analysis.engine.completion(request)).map_err(|e| e.to_string())?;
    Ok(items
        .into_iter()
        .map(|c| Completion {
            insert_text: c.insert_text.unwrap_or_else(|| c.label.clone()),
            label: c.label,
            kind: c.kind,
            detail: c.detail,
        })
        .collect())
}

#[cfg(test)]
#[path = "analysis_tests.rs"]
mod tests;
