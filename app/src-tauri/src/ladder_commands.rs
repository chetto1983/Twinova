//! The Ladder editor's commands (spec 2026-11-13-ladder-editor-design.md). The editor holds a file as its tree, which
//! Rust reads from the text and prints back (the plan's decision 1), and asks for the file lowered as Build lowers it:
//! the BOOL holding each element's power, and the rungs that do not lower. A file goes to PLCopen TC6 and back
//! (`twinova_engine::plcopen`).

use serde::Serialize;
use trust_ide::stdlib_docs::{standard_fb_doc, standard_function_doc, standard_function_entries};
use twinova_engine::ladder_catalog::{CatalogEntry, catalog};
use twinova_engine::plcopen::{Imported, export_ladder, import_ladder};
use twinova_engine::program_files::ProgramFiles;
use twinova_ladder::{
    BoxPins, ElementId, LowerError, ParseError, Power, Program, parse, parse_spans, print,
};

#[tauri::command]
pub fn ladder_parse(text: String) -> Result<Program, ParseError> {
    parse(&text)
}

/// Each rung's own first line in `text`, from 1: where the drawing sets its breakpoint, the line the debugger answers
/// for it (spec 2026-11-24-open-points-design.md, decision 2).
#[tauri::command]
pub fn ladder_rung_lines(text: String) -> Result<Vec<u32>, ParseError> {
    let (_, spans) = parse_spans(&text)?;
    let line = |at: usize| u32::try_from(text[..at].matches('\n').count() + 1).unwrap_or(u32::MAX);
    Ok(spans.iter().map(|span| line(span.at)).collect())
}

#[tauri::command]
pub fn ladder_print(program: Program) -> String {
    print(&program)
}

/// A Ladder file lowered, as the editor draws it: each element's power, how each box sits on its rung, and the rungs
/// that do not lower.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Lowering {
    pub power: Vec<Power>,
    pub boxes: Vec<(ElementId, BoxPins)>,
    pub errors: Vec<LowerError>,
}

/// `file` of `program`, the editor's texts, lowered with the boxes resolved against the whole program.
#[tauri::command]
pub fn ladder_lowering(program: Vec<(String, String)>, file: String) -> Result<Lowering, String> {
    let lowered = ProgramFiles::ladder(&program, &file)?;
    Ok(Lowering {
        power: lowered.power,
        boxes: lowered.boxes,
        errors: lowered.errors,
    })
}

/// A box the catalogue offers, with the table of IEC 61131-3 Ed.3 its standard group is (truST's documentation of
/// it, `stdlib_docs.rs`): what the picker groups by. None for the project's own.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Catalogued {
    #[serde(flatten)]
    pub entry: CatalogEntry,
    pub table: Option<u32>,
}

/// The table a standard block's or function's documentation names: "… (IEC 61131-3 Ed.3, Table 46)." is 46, and
/// "Tables 22-27" is 22.
fn table_of(doc: &str) -> Option<u32> {
    let after = doc
        .split_once("Table")?
        .1
        .trim_start_matches('s')
        .trim_start();
    after
        .split(|c: char| !c.is_ascii_digit())
        .next()?
        .parse()
        .ok()
}

/// Every box the Ladder files of `program`, the editor's texts, may hold (`twinova_engine::ladder_catalog`).
#[tauri::command]
pub fn ladder_catalog(program: Vec<(String, String)>) -> Result<Vec<Catalogued>, String> {
    let names: Vec<&str> = standard_function_entries()
        .iter()
        .map(|e| e.name.as_str())
        .collect();
    let entries = catalog(&program, &names)?;
    Ok((entries.into_iter())
        .map(|entry| {
            let doc = match (entry.own, entry.block) {
                (true, _) => None,
                (false, true) => standard_fb_doc(&entry.name),
                (false, false) => standard_function_doc(&entry.name),
            };
            Catalogued {
                table: doc.and_then(table_of),
                entry,
            }
        })
        .collect())
}

/// Writes the Ladder file `file` of `program`, the editor's texts, to `target` as a PLCopen TC6 project made at
/// `created` (the webview's clock: the engine keeps none). The webview has no file access: the path is the save
/// dialog's, as for a project archive (`project_commands.rs`).
#[tauri::command]
pub fn ladder_export(
    program: Vec<(String, String)>,
    file: String,
    created: String,
    target: String,
) -> Result<(), String> {
    let xml = export_ladder(&program, &file, &created)?;
    std::fs::write(&target, xml).map_err(|e| format!("{target}: {e}"))
}

/// The Ladder programs of the PLCopen TC6 project in file `source`, as files of the PLC, and why the rest was left out.
#[tauri::command]
pub fn ladder_import(source: String) -> Result<Imported, String> {
    let xml = std::fs::read_to_string(&source).map_err(|e| format!("{source}: {e}"))?;
    import_ladder(&xml).map_err(|e| format!("{source}: {e}"))
}

#[cfg(test)]
#[path = "ladder_commands_tests.rs"]
mod tests;
