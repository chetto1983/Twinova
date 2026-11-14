//! What may be typed as a Ladder element's operand (plan 2026-11-13-ladder-editor.md, Task 6; the user's choice of
//! truST's completion over a list of its own): truST is asked on the program the file's declarations make, the operand
//! written as a statement of its body, so it offers what ST offers there: the file's variables, the globals, a block
//! instance's members after its dot.

use tauri::State;
use twinova_ladder::{declarations_only, program_name};

use crate::analysis::{AnalysisState, Completion, completion};
use crate::ladder_table::END;

/// The candidates for `typed`, the start of an operand in Ladder file `file` whose text is `text`.
#[tauri::command]
pub fn ladder_completion(
    state: State<'_, AnalysisState>,
    file: String,
    text: String,
    typed: String,
) -> Result<Vec<Completion>, String> {
    let only = declarations_only(program_name(&file)?, &text).map_err(|e| e.message)?;
    let head = only
        .strip_suffix(END)
        .ok_or("the declarations do not end the program")?;
    let line = u32::try_from(head.lines().count() + 1).map_err(|e| e.to_string())?;
    let column = u32::try_from(typed.encode_utf16().count() + 1).map_err(|e| e.to_string())?;
    let offered = completion(
        state.clone(),
        file.clone(),
        format!("{head}\n{typed}{END}"),
        line,
        column,
    );
    // the file goes back to what the frontend reads of it, its declarations alone (`analysis.rs` `frontend_text`)
    state.on(&file, only, |_| Ok(()))?;
    offered
}

#[cfg(test)]
#[path = "ladder_operand_tests.rs"]
mod tests;
