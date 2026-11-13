//! What a breakpoint can be besides a stop (spec 2026-11-13-breakpoints-complete-design.md, decisions 1 to 3): a
//! condition, a hit count, or a log message that never stops, built into truST's `DebugBreakpoint`.

use serde::{Deserialize, Serialize};
use trust_runtime::debug::{DebugBreakpoint, HitCondition, LogFragment, SourceLocation};
use trust_runtime::program_model::Expr;

/// A breakpoint as the editor sets it: its line, and optionally a condition, a hit count and a log message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Breakpoint {
    pub line: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hit: Option<Hit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log: Option<String>,
}

/// Stops when the breakpoint's hits compare so with `count`: truST's three; CODESYS's "a multiple of" is not one.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Hit {
    pub op: HitOp,
    pub count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HitOp {
    Equal,
    AtLeast,
    GreaterThan,
}

impl Hit {
    pub(crate) fn condition(self) -> HitCondition {
        match self.op {
            HitOp::Equal => HitCondition::Equal(self.count),
            HitOp::AtLeast => HitCondition::AtLeast(self.count),
            HitOp::GreaterThan => HitCondition::GreaterThan(self.count),
        }
    }
}

/// A line a logpoint wrote, where it is.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Log {
    pub file: String,
    pub line: u32,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Piece<'a> {
    Text(&'a str),
    Expr(&'a str),
}

/// A log message's text and `{expressions}`, in order.
pub(crate) fn pieces(message: &str) -> Result<Vec<Piece<'_>>, String> {
    let mut out = Vec::new();
    let mut rest = message;
    while !rest.is_empty() {
        let open = rest.find('{');
        let close = rest.find('}');
        match (open, close) {
            (None, None) => {
                out.push(Piece::Text(rest));
                break;
            }
            (Some(o), Some(c)) if o < c => {
                if o > 0 {
                    out.push(Piece::Text(&rest[..o]));
                }
                let expr = rest[o + 1..c].trim();
                if expr.is_empty() {
                    return Err("an empty {} in the message".into());
                }
                out.push(Piece::Expr(expr));
                rest = &rest[c + 1..];
            }
            (Some(_), None) => return Err("'{' without '}' in the message".into()),
            _ => return Err("'}' without '{' in the message".into()),
        }
    }
    Ok(out)
}

/// Around each logged expression: its value through `TO_STRING`, then bare. Control characters a message's text does
/// not hold.
const START: char = '\u{2}';
const BARE: char = '\u{3}';
const END: char = '\u{4}';

/// truST's breakpoint for `breakpoint` at `location`, its condition and each `{expression}` parsed by `parse`. An
/// expression is logged twice, through `TO_STRING` and bare, and `message` keeps the first unless it is an error: a
/// number logs as `1`, not `DInt(1)`, and a TIME, which `TO_STRING` refuses, as truST shows it (decision 3, measured).
pub(crate) fn compile(
    breakpoint: &Breakpoint,
    location: SourceLocation,
    parse: &mut dyn FnMut(&str) -> Result<Expr, String>,
) -> Result<DebugBreakpoint, String> {
    let at = breakpoint.line;
    let mut compiled = DebugBreakpoint::new(location);
    if let Some(text) = &breakpoint.condition {
        let expr = parse(text).map_err(|e| format!("line {at}: the condition '{text}' {e}"))?;
        compiled.condition = Some(expr);
    }
    compiled.hit_condition = breakpoint.hit.map(Hit::condition);
    if let Some(message) = &breakpoint.log {
        let mut fragments = Vec::new();
        for piece in pieces(message).map_err(|e| format!("line {at}: {e}"))? {
            match piece {
                Piece::Text(text) => {
                    fragments.push(LogFragment::Text(text.replace([START, BARE, END], "")))
                }
                Piece::Expr(text) => {
                    let mut parsed = |source: &str| {
                        parse(source).map_err(|e| format!("line {at}: the expression '{text}' {e}"))
                    };
                    let bare = parsed(text)?;
                    let as_text = parsed(&format!("TO_STRING({text})"))?;
                    fragments.extend([
                        LogFragment::Text(START.into()),
                        LogFragment::Expr(as_text),
                        LogFragment::Text(BARE.into()),
                        LogFragment::Expr(bare),
                        LogFragment::Text(END.into()),
                    ]);
                }
            }
        }
        compiled.log_message = Some(fragments);
    }
    Ok(compiled)
}

/// A logged line as the user reads it: each expression's `TO_STRING` text, or its bare value where that is an error.
pub(crate) fn message(logged: &str) -> String {
    let mut out = String::new();
    let mut rest = logged;
    while let Some(start) = rest.find(START) {
        out.push_str(&rest[..start]);
        let inner = &rest[start + START.len_utf8()..];
        let end = inner.find(END).unwrap_or(inner.len());
        let (text, bare) = inner[..end].split_once(BARE).unwrap_or((&inner[..end], ""));
        out.push_str(if text.starts_with("<error") {
            bare
        } else {
            text
        });
        rest = inner.get(end + END.len_utf8()..).unwrap_or("");
    }
    out.push_str(rest);
    out
}
