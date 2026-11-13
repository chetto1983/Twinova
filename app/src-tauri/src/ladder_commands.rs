//! The Ladder editor's commands (spec 2026-11-13-ladder-editor-design.md). The editor holds a file as its tree, which
//! Rust reads from the text and prints back (the plan's decision 1), and asks for the file lowered as Build lowers it:
//! the BOOL holding each element's power, and the rungs that do not lower.

use serde::Serialize;
use twinova_engine::program_files::ProgramFiles;
use twinova_ladder::{LowerError, ParseError, Power, Program, parse, print};

#[tauri::command]
pub fn ladder_parse(text: String) -> Result<Program, ParseError> {
    parse(&text)
}

#[tauri::command]
pub fn ladder_print(program: Program) -> String {
    print(&program)
}

/// A Ladder file lowered, as the editor draws it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Lowering {
    pub power: Vec<Power>,
    pub errors: Vec<LowerError>,
}

/// `file` of `program`, the editor's texts, lowered with the boxes resolved against the whole program.
#[tauri::command]
pub fn ladder_lowering(program: Vec<(String, String)>, file: String) -> Result<Lowering, String> {
    let lowered = ProgramFiles::ladder(&program, &file)?;
    Ok(Lowering {
        power: lowered.power,
        errors: lowered.errors,
    })
}

#[cfg(test)]
#[path = "ladder_commands_tests.rs"]
mod tests;
