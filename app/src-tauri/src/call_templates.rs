//! A call's parameters as a template, and Auto Declare's proposal (spec 2026-11-13-completion-navigation-design.md,
//! decisions 3 and 4), from truST's frontend on the editor's sources. Lines and columns are Monaco's.

use serde::Serialize;
use text_size::TextSize;
use trust_hir::symbols::{ParamDirection, SymbolKind};
use trust_ide::signature_help::call_signature_info;
use trust_ide::util::{find_enclosing_pou, scope_at_position};
use trust_syntax::parser::parse;
use trust_syntax::syntax::SyntaxKind;

use crate::frontend::{frontend_of, id_of, text_of};
use crate::positions::offset;

/// The parameters of a call, as Monaco snippets: every one, and the mandatory ones only.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Template {
    pub all: String,
    pub required: String,
}

/// What Auto Declare proposes for an undeclared name: its POU and, when the code tells it, its type.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Proposal {
    pub name: String,
    pub pou: String,
    pub type_name: Option<String>,
}

/// The template of the call whose `(` was just typed before the place in `file`; `None` outside a call.
///
/// Mandatory follows IEC 61131-3: an FB's in-outs, and every input and in-out of a function (a standard one included,
/// which truST knows by its signature only). Outputs are never mandatory.
#[tauri::command]
pub fn call_template(
    files: Vec<(String, String)>,
    file: String,
    line: u32,
    column: u32,
) -> Result<Option<Template>, String> {
    let (frontend, ids) = frontend_of(&files);
    let text = text_of(&files, &file)?;
    let id = id_of(&files, &ids, &file).ok_or("no such file")?;
    let at = offset(text, line, column).ok_or("no such place")?;
    let db = frontend.database();
    let Some(signature) = call_signature_info(db, id, TextSize::from(at as u32)) else {
        return Ok(None);
    };
    let symbols = db.file_symbols_with_project(id);
    let function = (symbols.lookup(&signature.name))
        .and_then(|s| symbols.get(s))
        .is_none_or(|s| matches!(s.kind, SymbolKind::Function { .. }));
    let mandatory = |direction: ParamDirection| match direction {
        ParamDirection::InOut => true,
        ParamDirection::In => function,
        ParamDirection::Out => false,
    };
    let snippet = |only_mandatory: bool| {
        (signature.params.iter())
            .filter(|p| !only_mandatory || mandatory(p.direction))
            .enumerate()
            .map(|(n, p)| {
                let op = match p.direction {
                    ParamDirection::Out => "=>",
                    ParamDirection::In | ParamDirection::InOut => ":=",
                };
                format!("{} {op} ${{{}}}", p.name, n + 1)
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    Ok(Some(Template {
        all: snippet(false),
        required: snippet(true),
    }))
}

/// Auto Declare's proposal for the name starting at a place of `file`; `None` where no name starts.
///
/// The type is the other side's, when the name is assigned: a literal's own (an integer is INT, as trust-lsp's quick fix
/// proposes), or a declared variable's. Anything else proposes none.
#[tauri::command]
pub fn declare_proposal(
    files: Vec<(String, String)>,
    file: String,
    line: u32,
    column: u32,
) -> Result<Option<Proposal>, String> {
    let (frontend, ids) = frontend_of(&files);
    let text = text_of(&files, &file)?;
    let id = id_of(&files, &ids, &file).ok_or("no such file")?;
    let at = TextSize::from(offset(text, line, column).ok_or("no such place")? as u32);
    let root = parse(text).syntax();
    let Some(token) = (root.token_at_offset(at).right_biased())
        .filter(|t| t.kind() == SyntaxKind::Ident && t.text_range().start() == at)
    else {
        return Ok(None);
    };
    let pou = find_enclosing_pou(&root, at)
        .and_then(|p| p.children().find(|c| c.kind() == SyntaxKind::Name))
        .map(|n| n.text().to_string().trim().to_string())
        .unwrap_or_default();
    let assigned = (token.parent_ancestors())
        .find(|n| n.kind() == SyntaxKind::AssignStmt)
        .and_then(|assign| {
            let mut sides = assign.children().filter(|c| c.kind().is_expression_node());
            let (left, right) = (sides.next()?, sides.next()?);
            left.text_range().contains(at).then_some(right)
        });
    let type_name = assigned.and_then(|right| match right.kind() {
        SyntaxKind::Literal => literal_type(right.text().to_string().trim()),
        SyntaxKind::NameRef => {
            let db = frontend.database();
            let symbols = db.file_symbols_with_project(id);
            let name = right.text().to_string();
            let scope = scope_at_position(&symbols, &root, right.text_range().start());
            let symbol = symbols.get(symbols.resolve(name.trim(), scope)?)?;
            symbols.type_name(symbol.type_id).map(|t| t.to_string())
        }
        _ => None,
    });
    Ok(Some(Proposal {
        name: token.text().to_string(),
        pou,
        type_name,
    }))
}

/// The type a literal has on its own: a string's, a typed literal's prefix, an integer's INT, a real's REAL.
fn literal_type(literal: &str) -> Option<String> {
    let upper = literal.to_ascii_uppercase();
    if upper == "TRUE" || upper == "FALSE" {
        return Some("BOOL".into());
    }
    if let Some((prefix, _)) = upper.split_once('#') {
        return Some(
            match prefix {
                "T" | "TIME" => "TIME",
                "LT" | "LTIME" => "LTIME",
                p if p.chars().all(|c| c.is_ascii_digit()) => "INT",
                p => p,
            }
            .to_string(),
        );
    }
    let plain = upper.replace('_', "");
    let kind = match upper.chars().next() {
        Some('\'') => "STRING",
        Some('"') => "WSTRING",
        _ if plain.parse::<i64>().is_ok() => "INT",
        _ if plain.parse::<f64>().is_ok() => "REAL",
        _ => return None,
    };
    Some(kind.into())
}

#[cfg(test)]
#[path = "call_templates_tests.rs"]
mod tests;
