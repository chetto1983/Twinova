//! Where in the project a truST compile error is. truST locates a problem as `at <start>..<end>`, in bytes of the
//! source it compiled (measured on `6405915`, 2026-11-09; `CompileError` carries only its message); Build rewrites
//! each location as a file and line, `at plc/Main.st:12`, or a Ladder file's rung, `at plc/Pump.ld, rung 3`.

use crate::program_files::ProgramFiles;

/// `message` with each `at <start>..<end>` rewritten by `at(start)`; text that only looks like one stays.
pub(crate) fn rewrite_spans(message: &str, at: impl Fn(usize) -> String) -> String {
    let mut out = String::new();
    let mut rest = message;
    while let Some(i) = rest.find("at ") {
        let (before, after) = rest.split_at(i + 3);
        out.push_str(before);
        let digits = |s: &str| s.bytes().take_while(u8::is_ascii_digit).count();
        let start_len = digits(after);
        let end_len = after[start_len..].strip_prefix("..").map_or(0, digits);
        if start_len == 0 || end_len == 0 {
            rest = after;
            continue;
        }
        out.push_str(&at(after[..start_len].parse().expect("digits")));
        rest = &after[start_len + 2 + end_len..];
    }
    out.push_str(rest);
    out
}

/// The line of byte `offset` in `text`, from 1.
pub(crate) fn line_of(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())].matches('\n').count() + 1
}

/// truST's message for program files it compiled with their paths (`<path>: <problem>` a line): each span rewritten
/// as the place it comes from (`ProgramFiles::placer`), the label then dropped; a line with no span keeps its label.
pub(crate) fn locate_in_files(message: &str, program: &ProgramFiles) -> String {
    let line = |l: &str| {
        let Some((label, problem)) = l.split_once(": ") else {
            return l.to_string();
        };
        let Some(place) = program.placer(label) else {
            return l.to_string();
        };
        let located = rewrite_spans(problem, place);
        if located == problem {
            l.to_string()
        } else {
            located
        }
    };
    message.lines().map(line).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
#[path = "locate_tests.rs"]
mod tests;
