//! Where a file the user edits sits in a runtime file, for the debugger (spec 2026-11-13-breakpoints-hold-design.md):
//! - an ST file or a block maps its lines onto the runtime file's by an offset;
//! - a Ladder file maps them through its rungs (spec 2026-11-24-open-points-design.md, decision 2). Its runtime file
//!   is the ST it is lowered to, whose lines are not the file's.

use crate::locate::line_of;
use crate::program_files::Ladder;

/// How many lines `text` has, its last one counted though unterminated.
pub(crate) fn line_count(text: &str) -> u32 {
    u32::try_from(text.lines().count()).unwrap_or(u32::MAX)
}

/// A rung of a Ladder file: its lines in the file (1-based: its first comment's, its own first and its last) and its
/// generated lines in the runtime file (0-based rows: the first and the last).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Rung {
    pub from: u32,
    pub at: u32,
    pub to: u32,
    pub rows: (u32, u32),
}

/// A file the user edits, inside a runtime file: its path, the runtime file's id, its first line there (0-based) and
/// its number of lines; a Ladder file's rungs, through which its lines map instead.
#[derive(Debug, Clone)]
pub(crate) struct Place {
    pub path: String,
    pub file_id: u32,
    pub first_line: u32,
    pub lines: u32,
    pub rungs: Option<Vec<Rung>>,
}

impl Place {
    /// A runtime file that is the user's file, line for line: an ST file of the program.
    pub(crate) fn whole(path: &str, file_id: u32, text: &str) -> Self {
        Self {
            path: path.to_string(),
            file_id,
            first_line: 0,
            lines: line_count(text),
            rungs: None,
        }
    }

    /// A Ladder file whose lowered ST is runtime file `file_id`: each rung from where its text is to its generated
    /// lines.
    pub(crate) fn ladder(ladder: &Ladder, file_id: u32) -> Self {
        let (text, st) = (&ladder.text, &ladder.lowered.st);
        let line =
            |text: &str, offset: usize| u32::try_from(line_of(text, offset)).unwrap_or(u32::MAX);
        let spans =
            (twinova_ladder::parse_spans(text)).map_or_else(|_| Vec::new(), |(_, spans)| spans);
        let rungs = (ladder.lowered.rung_lines().into_iter())
            .filter_map(|(rung, first, last)| {
                let span = spans.get(rung)?;
                Some(Rung {
                    from: line(text, span.from),
                    at: line(text, span.at),
                    to: line(text, span.to.saturating_sub(1)),
                    rows: (line(st, first) - 1, line(st, last) - 1),
                })
            })
            .collect();
        Self {
            path: ladder.path.clone(),
            file_id,
            first_line: 0,
            lines: line_count(st),
            rungs: Some(rungs),
        }
    }

    /// Whether a stop in this place goes on stepping while it stays on its line: a Ladder file's, which is a rung's.
    pub(crate) fn steps_by_rung(&self) -> bool {
        self.rungs.is_some()
    }

    /// The runtime row a breakpoint on 1-based `line` searches from: a Ladder rung's first generated line, from any line
    /// of the rung or of its comment; none for a line outside every rung.
    pub(crate) fn row_of(&self, line: u32) -> Option<u32> {
        match &self.rungs {
            None => Some(self.first_line + line.checked_sub(1)?),
            Some(rungs) => (rungs.iter())
                .find(|r| (r.from..=r.to).contains(&line))
                .map(|r| r.rows.0),
        }
    }

    /// The 1-based line runtime `row` shows on in this place, a Ladder rung's own first line; none when the row is not
    /// this place's.
    pub(crate) fn line_at(&self, row: u32) -> Option<u32> {
        match &self.rungs {
            None => (self.first_line..self.first_line + self.lines)
                .contains(&row)
                .then(|| row - self.first_line + 1),
            Some(rungs) => (rungs.iter())
                .find(|r| (r.rows.0..=r.rows.1).contains(&row))
                .map(|r| r.at),
        }
    }
}

#[cfg(test)]
#[path = "debugger_places_tests.rs"]
mod tests;
