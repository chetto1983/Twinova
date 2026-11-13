//! Navigation in the ST editor (spec 2026-11-13-completion-navigation-design.md, decision 1): a symbol's definition
//! and its references, from the analysis engine that already holds the program, the blocks and the field's
//! configuration (`analysis.rs`). Lines and columns are Monaco's: from 1, the columns in UTF-16 units.

use serde::Serialize;
use tauri::State;
use trust_wasm_analysis::{DefinitionRequest, Range, ReferencesRequest};

use crate::analysis::{AnalysisState, position};

/// A stretch of a file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Span {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

impl Span {
    fn of(file: String, range: &Range) -> Self {
        Span {
            file,
            line: range.start.line + 1,
            column: range.start.character + 1,
            end_line: range.end.line + 1,
            end_column: range.end.character + 1,
        }
    }
}

/// A use of a symbol, and whether it writes it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Reference {
    pub span: Span,
    pub write: bool,
}

/// Where the symbol at a place of `file` is declared, `text` being the file as the editor holds it now; `None` for a
/// symbol with no source, such as a standard FB, which truST answers with an empty range at 0:0 (measured on TON).
#[tauri::command]
pub fn definition(
    state: State<'_, AnalysisState>,
    file: String,
    text: String,
    line: u32,
    column: u32,
) -> Result<Option<Span>, String> {
    let request = DefinitionRequest {
        uri: file.clone(),
        position: position(line, column),
    };
    let found = state.on(&file, text, |engine| {
        engine.definition(request).map_err(|e| e.to_string())
    })?;
    Ok(found
        .filter(|d| d.range.start != d.range.end)
        .map(|d| Span::of(d.uri, &d.range)))
}

/// Every use of the symbol at a place of `file`, its declaration included, in the files the analysis holds.
#[tauri::command]
pub fn references(
    state: State<'_, AnalysisState>,
    file: String,
    text: String,
    line: u32,
    column: u32,
) -> Result<Vec<Reference>, String> {
    let request = ReferencesRequest {
        uri: file.clone(),
        position: position(line, column),
        include_declaration: Some(true),
    };
    let found = state.on(&file, text, |engine| {
        engine.references(request).map_err(|e| e.to_string())
    })?;
    Ok((found.into_iter())
        .map(|r| Reference {
            span: Span::of(r.uri, &r.range),
            write: r.is_write,
        })
        .collect())
}

#[cfg(test)]
#[path = "navigation_tests.rs"]
mod tests;
