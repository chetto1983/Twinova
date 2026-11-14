//! A Ladder file's variables in the table (plan 2026-11-13-ladder-editor.md, decision 2): its declarations are an ST
//! `VAR` block at its head, so the table reads and edits them as the program they make (`declarations_only`), and the
//! rungs after them stay as they are.

use twinova_ladder::{declarations_only, declarations_prefix, parse, program_name};

use crate::declarations::{Change, Declaration, declarations, edit};

const END: &str = "\nEND_PROGRAM\n";

/// Whether `file` is a Ladder file, whose table goes through this module.
pub(crate) fn is_ladder(file: &str) -> bool {
    file.ends_with(".ld")
}

/// The program `file`'s declarations make, its name's prefix, and the text after the declarations: the rungs.
fn split<'a>(file: &str, text: &'a str) -> Result<(String, String, &'a str), String> {
    let name = program_name(file)?;
    let program = parse(text).map_err(|e| e.message)?;
    let head = text.len() - text.trim_start().len() + program.declarations.len();
    let only = declarations_only(name, text).map_err(|e| e.message)?;
    Ok((only, declarations_prefix(name), &text[head..]))
}

/// The table's rows of Ladder file `file`; none while it does not read.
pub(crate) fn rows(file: &str, text: &str) -> Vec<Declaration> {
    split(file, text).map_or_else(|_| Vec::new(), |(only, _, _)| declarations(&only))
}

/// Ladder file `file` after an edit of its table: its declarations edited, its rungs as they were.
pub(crate) fn edited(file: &str, text: &str, change: &Change) -> Result<String, String> {
    let (only, prefix, rungs) = split(file, text)?;
    let changed = edit(&only, change)?;
    let head = (changed.strip_prefix(prefix.as_str()))
        .and_then(|rest| rest.strip_suffix(END))
        .ok_or("the edit did not stay among the declarations")?;
    let joint = if head.is_empty() || head.ends_with('\n') || rungs.starts_with('\n') {
        ""
    } else {
        "\n"
    };
    Ok(format!("{head}{joint}{rungs}"))
}

#[cfg(test)]
#[path = "ladder_table_tests.rs"]
mod tests;
