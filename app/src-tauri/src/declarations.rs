//! The variables table (spec 2026-11-13-online-monitoring-design.md, decisions 9 and 10): a POU's declarations read
//! from truST's syntax tree, and each edit of the table turned into a text edit of the tab. A declaration's own line is
//! written again, nothing around it reformatted; a rename is truST's (`trust_ide::rename::rename_checked`), every
//! reference included. An edit that adds parse errors is refused. The Comment column is the comment after a
//! declaration's `;`; a comment above it stays in the text, outside the row.

use serde::{Deserialize, Serialize};
use text_size::{TextRange, TextSize};
use trust_hir::project::{Project as Frontend, SourceKey};
use trust_ide::rename::rename_checked;
use trust_syntax::parser::parse;
use trust_syntax::syntax::{SyntaxKind, SyntaxNode, SyntaxToken};

/// A row of the table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Declaration {
    /// The POU it belongs to: a program or a function block.
    pub pou: String,
    pub name: String,
    /// The block's keyword: `VAR`, `VAR_INPUT`, `VAR_OUTPUT`, `VAR_IN_OUT`, `VAR_TEMP`, `VAR_GLOBAL`, `VAR_EXTERNAL`.
    pub class: String,
    /// `RETAIN`, `CONSTANT`, `PERSISTENT` or `NON_RETAIN`, as the block declares it.
    pub qualifier: Option<String>,
    pub type_name: String,
    pub address: Option<String>,
    pub initial: Option<String>,
    pub comment: Option<String>,
}

/// An edit of the table: a row by its index in `declarations`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum Change {
    Add {
        row: Declaration,
    },
    Delete {
        index: usize,
    },
    /// Any field but the name: a rename goes through `rename`.
    Update {
        index: usize,
        row: Declaration,
    },
}

const POUS: [SyntaxKind; 2] = [SyntaxKind::Program, SyntaxKind::FunctionBlock];

fn tokens(node: &SyntaxNode) -> impl Iterator<Item = SyntaxToken> {
    node.descendants_with_tokens()
        .filter_map(|e| e.into_token())
}

fn significant(node: &SyntaxNode) -> impl Iterator<Item = SyntaxToken> {
    tokens(node).filter(|t| !t.kind().is_trivia())
}

/// A node's words, without the comments and white space the parser attaches to it.
fn words(node: &SyntaxNode) -> String {
    significant(node).map(|t| t.text().to_string()).collect()
}

fn pou_name(pou: &SyntaxNode) -> String {
    let name = pou.children().find(|c| c.kind() == SyntaxKind::Name);
    name.map(|n| words(&n)).unwrap_or_default()
}

/// A block's keyword and qualifier: the words before its first declaration.
fn class_of(block: &SyntaxNode) -> (String, Option<String>) {
    let words: Vec<String> = (block.children_with_tokens())
        .take_while(|e| e.kind() != SyntaxKind::VarDecl)
        .filter_map(|e| e.into_token())
        .filter(|t| !t.kind().is_trivia())
        .map(|t| t.text().to_uppercase())
        .collect();
    let class = words.first().cloned().unwrap_or_else(|| "VAR".into());
    (class, words.get(1).cloned())
}

fn text_between(text: &str, from: TextSize, to: TextSize) -> Option<String> {
    let found = text[usize::from(from)..usize::from(to)].trim();
    (!found.is_empty()).then(|| found.to_string())
}

fn comment_text(token: &SyntaxToken) -> String {
    let raw = token.text();
    let inner = raw
        .strip_prefix("(*")
        .and_then(|r| r.strip_suffix("*)"))
        .or_else(|| raw.strip_prefix("//"))
        .unwrap_or(raw);
    inner.trim().to_string()
}

/// The comment written after a declaration's `;` on its own line.
fn trailing_comment(decl: &SyntaxNode) -> Option<SyntaxToken> {
    let mut after = tokens(decl)
        .skip_while(|t| t.kind() != SyntaxKind::Semicolon)
        .skip(1);
    after.find(|t| matches!(t.kind(), SyntaxKind::BlockComment | SyntaxKind::LineComment))
}

fn read(decl: &SyntaxNode, text: &str, pou: &str, block: &SyntaxNode) -> Declaration {
    let (class, qualifier) = class_of(block);
    let names: Vec<String> = (decl.children())
        .filter(|c| c.kind() == SyntaxKind::Name)
        .map(|n| words(&n))
        .collect();
    let all: Vec<SyntaxToken> = significant(decl).collect();
    let find = |kind| all.iter().find(|t| t.kind() == kind);
    let type_ref = decl.children().find(|c| c.kind() == SyntaxKind::TypeRef);
    let initial = match (find(SyntaxKind::Assign), find(SyntaxKind::Semicolon)) {
        (Some(assign), Some(end)) => {
            text_between(text, assign.text_range().end(), end.text_range().start())
        }
        _ => None,
    };
    Declaration {
        pou: pou.to_string(),
        name: names.join(", "),
        class,
        qualifier,
        type_name: type_ref.map_or_else(String::new, |t| t.text().to_string().trim().to_string()),
        address: find(SyntaxKind::DirectAddress).map(|t| t.text().to_string()),
        initial,
        comment: trailing_comment(decl).map(|t| comment_text(&t)),
    }
}

/// Every declaration of `text`'s programs and function blocks, in the order written, with its node and block.
fn walk(text: &str) -> Vec<(Declaration, SyntaxNode)> {
    let root = parse(text).syntax();
    let pous = root.descendants().filter(|n| POUS.contains(&n.kind()));
    pous.flat_map(|pou| {
        let name = pou_name(&pou);
        let blocks: Vec<SyntaxNode> = pou
            .children()
            .filter(|c| c.kind() == SyntaxKind::VarBlock)
            .collect();
        blocks
            .into_iter()
            .flat_map(|block| {
                let decls: Vec<SyntaxNode> = block
                    .children()
                    .filter(|c| c.kind() == SyntaxKind::VarDecl)
                    .collect();
                decls
                    .into_iter()
                    .map(|d| (read(&d, text, &name, &block), d))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    })
    .collect()
}

pub fn declarations(text: &str) -> Vec<Declaration> {
    walk(text).into_iter().map(|(d, _)| d).collect()
}

/// A row as its declaration's line: `name [AT address] : type [:= value]; [(* comment *)]`.
fn line_of(row: &Declaration) -> String {
    let mut line = row.name.clone();
    if let Some(address) = &row.address {
        line += &format!(" AT {address}");
    }
    line += &format!(" : {}", row.type_name.trim());
    if let Some(value) = row.initial.as_deref().filter(|v| !v.trim().is_empty()) {
        line += &format!(" := {}", value.trim());
    }
    line.push(';');
    if let Some(comment) = row.comment.as_deref().filter(|c| !c.trim().is_empty()) {
        line += &format!(" (* {} *)", comment.trim().replace("*)", "* )"));
    }
    line
}

/// A declaration's span: from its first word to the end of its `;` or of the comment after it.
fn span_of(decl: &SyntaxNode) -> TextRange {
    let first = significant(decl)
        .next()
        .map_or_else(|| decl.text_range(), |t| t.text_range());
    let last = trailing_comment(decl)
        .or_else(|| significant(decl).last())
        .map_or(first, |t| t.text_range());
    first.cover(last)
}

fn line_start(text: &str, at: usize) -> usize {
    text[..at].rfind('\n').map_or(0, |n| n + 1)
}

/// The whole line(s) of `range`, its newline included.
fn whole_lines(text: &str, range: TextRange) -> std::ops::Range<usize> {
    let end = usize::from(range.end());
    let end = text[end..].find('\n').map_or(text.len(), |n| end + n + 1);
    line_start(text, usize::from(range.start()))..end
}

/// Where a row of `class` and `qualifier` in `pou` goes: before its block's `END_VAR`, with the block's indentation, or
/// in a new block after the POU's last one (after its name when it has none).
fn insertion(
    text: &str,
    pou: &str,
    class: &str,
    qualifier: Option<&str>,
) -> Result<(usize, String, bool), String> {
    let root = parse(text).syntax();
    let node = (root.descendants())
        .find(|n| POUS.contains(&n.kind()) && pou_name(n) == pou)
        .ok_or_else(|| format!("{pou} is not a program or a function block of this file"))?;
    let blocks: Vec<SyntaxNode> = node
        .children()
        .filter(|c| c.kind() == SyntaxKind::VarBlock)
        .collect();
    let wanted = |b: &&SyntaxNode| {
        let (c, q) = class_of(b);
        c == class && q.as_deref() == qualifier
    };
    if let Some(block) = blocks.iter().find(wanted) {
        let end = significant(block).last().ok_or("an empty block")?;
        let at = line_start(text, usize::from(end.text_range().start()));
        let indent = (block
            .children()
            .filter(|c| c.kind() == SyntaxKind::VarDecl)
            .last())
        .map(|d| {
            let start = usize::from(span_of(&d).start());
            text[line_start(text, start)..start].to_string()
        })
        .unwrap_or_else(|| "    ".into());
        return Ok((at, indent, false));
    }
    // A node's range takes in the white space after it, so the last word's end is where it stops.
    let last_word = |n: &SyntaxNode| significant(n).last().map(|t| t.text_range().end());
    let name = node.children().find(|c| c.kind() == SyntaxKind::Name);
    let after = blocks.last().or(name.as_ref()).and_then(last_word);
    let end = usize::from(after.ok_or("a POU without a name")?);
    let at = text[end..].find('\n').map_or(text.len(), |n| end + n + 1);
    Ok((at, "    ".into(), true))
}

/// `text` with `row` added at its place.
fn add(text: &str, row: &Declaration) -> Result<String, String> {
    let (at, indent, new_block) = insertion(text, &row.pou, &row.class, row.qualifier.as_deref())?;
    let line = format!("{indent}{}\n", line_of(row));
    let block = if new_block {
        let head = [
            row.class.as_str(),
            row.qualifier.as_deref().unwrap_or_default(),
        ]
        .join(" ");
        format!("{}\n{line}END_VAR\n", head.trim())
    } else {
        line
    };
    Ok(format!("{}{block}{}", &text[..at], &text[at..]))
}

/// The text of `change` applied to `text`, refused when it adds parse errors.
pub fn edit(text: &str, change: &Change) -> Result<String, String> {
    let rows = walk(text);
    let row_at = |i: usize| rows.get(i).ok_or_else(|| format!("no declaration {i}"));
    let edited = match change {
        Change::Add { row } => add(text, row)?,
        Change::Delete { index } => {
            let lines = whole_lines(text, span_of(&row_at(*index)?.1));
            format!("{}{}", &text[..lines.start], &text[lines.end..])
        }
        Change::Update { index, row } => {
            let (old, node) = row_at(*index)?;
            let row = Declaration {
                name: old.name.clone(),
                ..row.clone()
            };
            if (&row.pou, &row.class, &row.qualifier) == (&old.pou, &old.class, &old.qualifier) {
                let span = span_of(node);
                let (start, end) = (usize::from(span.start()), usize::from(span.end()));
                format!("{}{}{}", &text[..start], line_of(&row), &text[end..])
            } else {
                let lines = whole_lines(text, span_of(node));
                add(
                    &format!("{}{}", &text[..lines.start], &text[lines.end..]),
                    &row,
                )?
            }
        }
    };
    let errors = |t: &str| parse(t).errors().len();
    if errors(&edited) > errors(text) {
        return Err("the edit would not parse".into());
    }
    Ok(edited)
}

/// Renames declaration `index` of `file` to `new_name` in every file of `files` that refers to it, through truST's
/// rename. Returns each changed file with its new text.
pub fn rename(
    files: &[(String, String)],
    file: &str,
    index: usize,
    new_name: &str,
) -> Result<Vec<(String, String)>, String> {
    let (_, text) = files
        .iter()
        .find(|(f, _)| f == file)
        .ok_or("no such file")?;
    let rows = walk(text);
    let (_, decl) = rows
        .get(index)
        .ok_or_else(|| format!("no declaration {index}"))?;
    let name = decl
        .children()
        .find(|c| c.kind() == SyntaxKind::Name)
        .ok_or("no name")?;
    let at = significant(&name)
        .next()
        .ok_or("no name")?
        .text_range()
        .start();
    let mut frontend = Frontend::new();
    let ids: Vec<_> = (files.iter())
        .map(|(path, source)| {
            frontend.set_source_text(SourceKey::from_virtual(path.clone()), source.clone())
        })
        .collect();
    let id = ids[files
        .iter()
        .position(|(f, _)| f == file)
        .expect("found above")];
    let result =
        rename_checked(frontend.database(), id, at, new_name).map_err(|e| format!("{e:?}"))?;
    let mut changed = Vec::new();
    for ((path, source), file_id) in files.iter().zip(&ids) {
        let Some(edits) = result.edits.get(file_id) else {
            continue;
        };
        let mut edits = edits.clone();
        edits.sort_by_key(|e| std::cmp::Reverse(e.range.start()));
        let mut out = source.clone();
        for e in edits {
            out.replace_range(
                usize::from(e.range.start())..usize::from(e.range.end()),
                &e.new_text,
            );
        }
        changed.push((path.clone(), out));
    }
    Ok(changed)
}

/// The table's rows of a tab's text.
#[tauri::command]
pub fn declarations_of(text: String) -> Vec<Declaration> {
    declarations(&text)
}

/// A tab's text after an edit of its table.
#[tauri::command]
pub fn edit_declaration(text: String, change: Change) -> Result<String, String> {
    edit(&text, &change)
}

/// The tabs' texts a rename changes, from every source the editor holds.
#[tauri::command]
pub fn rename_declaration(
    files: Vec<(String, String)>,
    file: String,
    index: usize,
    new_name: String,
) -> Result<Vec<(String, String)>, String> {
    rename(&files, &file, index, &new_name)
}

#[cfg(test)]
#[path = "declarations_tests.rs"]
mod tests;
