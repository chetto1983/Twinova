//! A Ladder file's declarations as a PLCopen TC6 POU's `<interface>`, and back (spec 2026-11-13-ladder-editor-design.md,
//! Task 5, the user's choice). They are read with truST's ST parser; what TC6 has no place for is refused by name.
//!
//! A `VAR` block is a TC6 section (`localVars`, `inputVars`…), its qualifiers attributes (`retain`, `constant`…). A
//! variable keeps its name, its `AT` address, its type (TC6's elementary types, `<derived>` for any other name, `<array>`),
//! its initial value (`<simpleValue>`, `<arrayValue>`) and a comment written in it (`<documentation>`). The tests are
//! `plcopen_tests.rs`'s, through a whole project.

use roxmltree::Node;
use trust_syntax::{SyntaxKind, SyntaxNode};
use twinova_ladder::esc;

/// The ST keyword of each section, and its TC6 element.
const SECTIONS: [(SyntaxKind, &str, &str); 7] = [
    (SyntaxKind::KwVar, "VAR", "localVars"),
    (SyntaxKind::KwVarInput, "VAR_INPUT", "inputVars"),
    (SyntaxKind::KwVarOutput, "VAR_OUTPUT", "outputVars"),
    (SyntaxKind::KwVarInOut, "VAR_IN_OUT", "inOutVars"),
    (SyntaxKind::KwVarTemp, "VAR_TEMP", "tempVars"),
    (SyntaxKind::KwVarExternal, "VAR_EXTERNAL", "externalVars"),
    (SyntaxKind::KwVarGlobal, "VAR_GLOBAL", "globalVars"),
];

/// A section's qualifiers, and their TC6 attributes.
const QUALIFIERS: [(SyntaxKind, &str, &str); 4] = [
    (SyntaxKind::KwRetain, "RETAIN", "retain"),
    (SyntaxKind::KwNonRetain, "NON_RETAIN", "nonretain"),
    (SyntaxKind::KwConstant, "CONSTANT", "constant"),
    (SyntaxKind::KwPersistent, "PERSISTENT", "persistent"),
];

/// TC6's elementary types (tc6_0201.xsd, `elementaryTypes`), as ST spells them, and the element TC6 gives each where it
/// differs.
const ELEMENTARY: [(&str, &str); 23] = [
    ("BOOL", "BOOL"),
    ("BYTE", "BYTE"),
    ("WORD", "WORD"),
    ("DWORD", "DWORD"),
    ("LWORD", "LWORD"),
    ("SINT", "SINT"),
    ("INT", "INT"),
    ("DINT", "DINT"),
    ("LINT", "LINT"),
    ("USINT", "USINT"),
    ("UINT", "UINT"),
    ("UDINT", "UDINT"),
    ("ULINT", "ULINT"),
    ("REAL", "REAL"),
    ("LREAL", "LREAL"),
    ("TIME", "TIME"),
    ("DATE", "DATE"),
    ("DT", "DT"),
    ("DATE_AND_TIME", "DT"),
    ("TOD", "TOD"),
    ("TIME_OF_DAY", "TOD"),
    ("STRING", "string"),
    ("WSTRING", "wstring"),
];

fn kinds(node: &SyntaxNode) -> impl Iterator<Item = SyntaxKind> + '_ {
    node.children_with_tokens().map(|e| e.kind())
}

fn trimmed(node: &SyntaxNode) -> String {
    node.text().to_string().trim().to_string()
}

/// The `<interface>` of the program `declarations` declares (`twinova_ladder::declarations_only`), one line per tag.
pub(crate) fn interface_xml(declarations: &str, indent: &str) -> Result<String, String> {
    let parse = trust_syntax::parser::parse(declarations);
    if let Some(error) = parse.errors().first() {
        // `declarations` keeps the Ladder file's lines (`declarations_only`)
        let line = declarations[..usize::from(error.range.start())]
            .matches('\n')
            .count()
            + 1;
        return Err(format!("line {line}: {}", error.message));
    }
    let mut lines = vec![format!("{indent}<interface>")];
    for block in parse
        .syntax()
        .descendants()
        .filter(|n| n.kind() == SyntaxKind::VarBlock)
    {
        let keyword = kinds(&block)
            .find(|k| !k.is_trivia())
            .unwrap_or(SyntaxKind::KwVar);
        let (_, _, section) = (SECTIONS.iter()).find(|(k, _, _)| *k == keyword).expect(
            "a Ladder file's declarations are the sections its grammar reads, each one TC6 has",
        );
        let attributes: String = (QUALIFIERS.iter())
            .filter(|(k, _, _)| kinds(&block).any(|b| b == *k))
            .map(|(_, _, attribute)| format!(" {attribute}=\"true\""))
            .collect();
        lines.push(format!("{indent}  <{section}{attributes}>"));
        for declaration in block.children().filter(|n| n.kind() == SyntaxKind::VarDecl) {
            variables(&declaration, &format!("{indent}    "), &mut lines)?;
        }
        lines.push(format!("{indent}  </{section}>"));
    }
    lines.push(format!("{indent}</interface>"));
    Ok(lines.join("\n"))
}

fn variables(
    declaration: &SyntaxNode,
    indent: &str,
    lines: &mut Vec<String>,
) -> Result<(), String> {
    let names: Vec<String> = (declaration.children())
        .filter(|n| n.kind() == SyntaxKind::Name)
        .map(|n| trimmed(&n))
        .collect();
    let tokens: Vec<_> = declaration
        .children_with_tokens()
        .filter_map(|e| e.into_token())
        .collect();
    let address = (tokens.iter())
        .find(|t| t.kind() == SyntaxKind::DirectAddress)
        .map(|t| format!(" address=\"{}\"", esc(t.text())));
    let comment = (tokens.iter())
        .find(|t| t.kind() == SyntaxKind::BlockComment)
        .map(|t| {
            t.text()
                .trim_start_matches("(*")
                .trim_end_matches("*)")
                .trim()
                .to_string()
        });
    let type_ref = (declaration.children())
        .find(|n| n.kind() == SyntaxKind::TypeRef)
        .ok_or_else(|| format!("{} has no type", names.join(", ")))?;
    let initial = (declaration.children())
        .skip_while(|n| n.kind() != SyntaxKind::TypeRef)
        .nth(1);
    for name in &names {
        lines.push(format!(
            "{indent}<variable name=\"{}\"{}>",
            esc(name),
            address.as_deref().unwrap_or_default()
        ));
        lines.push(format!(
            "{indent}  <type>{}</type>",
            type_xml(&type_ref, name)?
        ));
        if let Some(value) = &initial {
            lines.push(format!(
                "{indent}  <initialValue>{}</initialValue>",
                value_xml(value, name)?
            ));
        }
        if let Some(text) = &comment {
            lines.push(format!(
                "{indent}  <documentation><xhtml:p>{}</xhtml:p></documentation>",
                esc(text)
            ));
        }
        lines.push(format!("{indent}</variable>"));
    }
    Ok(())
}

fn type_xml(type_ref: &SyntaxNode, name: &str) -> Result<String, String> {
    let text = trimmed(type_ref);
    if let Some((_, element)) = ELEMENTARY
        .iter()
        .find(|(st, _)| st.eq_ignore_ascii_case(&text))
    {
        return Ok(format!("<{element}/>"));
    }
    let only: Vec<SyntaxNode> = type_ref.children().collect();
    match &only[..] {
        [named] if named.kind() == SyntaxKind::Name => {
            Ok(format!("<derived name=\"{}\"/>", esc(&trimmed(named))))
        }
        [array] if array.kind() == SyntaxKind::ArrayType => {
            let mut xml = String::from("<array>");
            for range in array
                .children()
                .filter(|n| n.kind() == SyntaxKind::Subrange)
            {
                let bounds: Vec<String> = range.children().map(|b| trimmed(&b)).collect();
                let [lower, upper] = &bounds[..] else {
                    return Err(format!(
                        "{name}'s array bounds {} are not a range, lower..upper",
                        trimmed(&range)
                    ));
                };
                xml.push_str(&format!(
                    "<dimension lower=\"{}\" upper=\"{}\"/>",
                    esc(lower),
                    esc(upper)
                ));
            }
            let base = (array.children())
                .find(|n| n.kind() == SyntaxKind::TypeRef)
                .ok_or_else(|| format!("{name}'s array has no element type"))?;
            xml.push_str(&format!(
                "<baseType>{}</baseType></array>",
                type_xml(&base, name)?
            ));
            Ok(xml)
        }
        _ => Err(format!(
            "{name}'s type {text} is not one PLCopen is written with here"
        )),
    }
}

fn value_xml(value: &SyntaxNode, name: &str) -> Result<String, String> {
    match value.kind() {
        SyntaxKind::ArrayInitializer => {
            let items: String = (value.children())
                .map(|item| {
                    format!(
                        "<value><simpleValue value=\"{}\"/></value>",
                        esc(&trimmed(&item))
                    )
                })
                .collect();
            Ok(format!("<arrayValue>{items}</arrayValue>"))
        }
        SyntaxKind::InitializerList => Err(format!(
            "{name}'s initial value {} is a structure, which PLCopen is not written with here",
            trimmed(value)
        )),
        _ => Ok(format!("<simpleValue value=\"{}\"/>", esc(&trimmed(value)))),
    }
}

fn element_kids<'a, 'i>(node: Node<'a, 'i>) -> impl Iterator<Item = Node<'a, 'i>> {
    node.children().filter(Node::is_element)
}

/// `node`'s first child element called `name`.
pub(crate) fn named<'a, 'i>(node: Node<'a, 'i>, name: &str) -> Option<Node<'a, 'i>> {
    element_kids(node).find(|n| n.tag_name().name() == name)
}

/// The VAR blocks of a TC6 `<interface>`, as a Ladder file's declarations.
pub(crate) fn declarations_text(interface: Node) -> Result<String, String> {
    let mut out = String::new();
    for section in element_kids(interface) {
        let tag = section.tag_name().name();
        let Some((_, keyword, _)) = SECTIONS.iter().find(|(_, _, s)| *s == tag) else {
            return Err(format!(
                "an interface's <{tag}>, which a Ladder program here has no section for"
            ));
        };
        let qualifiers: String = (QUALIFIERS.iter())
            .filter(|(_, _, attribute)| section.attribute(*attribute) == Some("true"))
            .map(|(_, st, _)| format!(" {st}"))
            .collect();
        out.push_str(&format!("{keyword}{qualifiers}\n"));
        for variable in element_kids(section).filter(|n| n.tag_name().name() == "variable") {
            out.push_str(&format!("    {}\n", variable_text(variable)?));
        }
        out.push_str("END_VAR\n");
    }
    Ok(out)
}

fn variable_text(variable: Node) -> Result<String, String> {
    let name = variable.attribute("name").unwrap_or_default();
    let address = variable
        .attribute("address")
        .map_or(String::new(), |a| format!(" AT {a}"));
    let ty = named(variable, "type")
        .and_then(|t| element_kids(t).next())
        .ok_or_else(|| format!("variable '{name}' has no type"))?;
    let mut text = format!("{name}{address} : {}", type_text(ty, name)?);
    if let Some(initial) = named(variable, "initialValue").and_then(|i| element_kids(i).next()) {
        text.push_str(&format!(" := {}", value_text(initial, name)?));
    }
    text.push(';');
    let doc: String = (named(variable, "documentation").into_iter())
        .flat_map(|d| d.descendants())
        .filter_map(|n| n.text().filter(|_| n.is_text()))
        .collect();
    if !doc.trim().is_empty() {
        text.push_str(&format!(" (* {} *)", doc.trim().replace("*)", "* )")));
    }
    Ok(text)
}

fn type_text(ty: Node, name: &str) -> Result<String, String> {
    let tag = ty.tag_name().name();
    // the first spelling of an element is ST's short one: DT, TOD
    if let Some((st, _)) = ELEMENTARY.iter().find(|(_, element)| *element == tag) {
        return Ok((*st).to_string());
    }
    match tag {
        "derived" => Ok(ty.attribute("name").unwrap_or_default().to_string()),
        "array" => {
            let dimensions: Vec<String> = (element_kids(ty))
                .filter(|d| d.tag_name().name() == "dimension")
                .map(|d| {
                    format!(
                        "{}..{}",
                        d.attribute("lower").unwrap_or("0"),
                        d.attribute("upper").unwrap_or("0")
                    )
                })
                .collect();
            let base = named(ty, "baseType")
                .and_then(|b| element_kids(b).next())
                .ok_or_else(|| format!("variable '{name}''s array has no base type"))?;
            Ok(format!(
                "ARRAY[{}] OF {}",
                dimensions.join(", "),
                type_text(base, name)?
            ))
        }
        other => Err(format!(
            "variable '{name}' is of a <{other}> type, which a Ladder program here is not read with"
        )),
    }
}

fn value_text(value: Node, name: &str) -> Result<String, String> {
    match value.tag_name().name() {
        "simpleValue" => Ok(value.attribute("value").unwrap_or_default().to_string()),
        "arrayValue" => {
            let items: Result<Vec<String>, String> = (element_kids(value))
                .filter_map(|v| element_kids(v).next())
                .map(|v| value_text(v, name))
                .collect();
            Ok(format!("[{}]", items?.join(", ")))
        }
        "structValue" => {
            let fields: Result<Vec<String>, String> = (element_kids(value))
                .map(|f| {
                    let inner = element_kids(f)
                        .next()
                        .ok_or_else(|| format!("variable '{name}' has an empty field"))?;
                    Ok(format!(
                        "{} := {}",
                        f.attribute("member").unwrap_or_default(),
                        value_text(inner, name)?
                    ))
                })
                .collect();
            Ok(format!("({})", fields?.join(", ")))
        }
        other => Err(format!(
            "variable '{name}''s initial value is a <{other}>, which is not read here"
        )),
    }
}
