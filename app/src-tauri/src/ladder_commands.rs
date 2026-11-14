//! The Ladder editor's commands (spec 2026-11-13-ladder-editor-design.md). The editor holds a file as its tree, which
//! Rust reads from the text and prints back (the plan's decision 1), and asks for the file lowered as Build lowers it:
//! the BOOL holding each element's power, and the rungs that do not lower. A file goes to PLCopen TC6 and back
//! (`twinova_engine::plcopen`).

use serde::Serialize;
use twinova_engine::plcopen::{Imported, export_ladder, import_ladder};
use twinova_engine::program_files::ProgramFiles;
use twinova_ladder::{BoxPins, ElementId, LowerError, ParseError, Power, Program, parse, print};

#[tauri::command]
pub fn ladder_parse(text: String) -> Result<Program, ParseError> {
    parse(&text)
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

/// The Ladder file `file` of `program`, the editor's texts, as a PLCopen TC6 project written at `created` (the
/// webview's clock: the engine keeps none).
#[tauri::command]
pub fn ladder_export(
    program: Vec<(String, String)>,
    file: String,
    created: String,
) -> Result<String, String> {
    export_ladder(&program, &file, &created)
}

/// The Ladder programs of PLCopen TC6 project `xml`, as files of the PLC, and why the rest was left out.
#[tauri::command]
pub fn ladder_import(xml: String) -> Result<Imported, String> {
    import_ladder(&xml)
}

#[cfg(test)]
#[path = "ladder_commands_tests.rs"]
mod tests;
