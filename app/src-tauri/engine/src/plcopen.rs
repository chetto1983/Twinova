//! A Ladder file as a PLCopen TC6 project, and the Ladder programs of one as files (spec 2026-11-13-ladder-editor-design.md,
//! decision 7, Task 5). The rungs are OIP's `<LD>` (`twinova_ladder::{ld_body, read_ld_body}`), the variables the
//! POU's `<interface>` (`plcopen_interface.rs`).

use serde::Serialize;
use twinova_ladder::{declarations_only, esc, ld_body, parse, program_name, read_ld_body};

use crate::plcopen_interface::{declarations_text, interface_xml, named};
use crate::program_files::ProgramFiles;

/// The Ladder files a TC6 project holds, and why each POU that is not one was left out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Imported {
    pub files: Vec<(String, String)>,
    pub notes: Vec<String>,
}

/// The Ladder file `path` of `program` as a TC6 project; `created` is its `creationDateTime`. Its boxes are resolved as
/// Build resolves them, and a rung that does not lower fails the export.
pub fn export_ladder(
    program: &[(String, String)],
    path: &str,
    created: &str,
) -> Result<String, String> {
    let (_, text) = (program.iter())
        .find(|(p, _)| p == path)
        .ok_or_else(|| format!("{path} is not a file of the program"))?;
    let name = program_name(path)?;
    let lowered = ProgramFiles::ladder(program, path)?;
    if let Some(error) = lowered.errors.first() {
        return Err(format!(
            "{path}, rung {}: {}",
            error.element.rung + 1,
            error.message
        ));
    }
    let rungs = parse(text).map_err(|e| format!("{path}: {}", e.message))?;
    let declarations =
        declarations_only(name, text).map_err(|e| format!("{path}: {}", e.message))?;
    let interface = interface_xml(&declarations, "        ").map_err(|e| format!("{path}: {e}"))?;
    let body = ld_body(&rungs, &lowered.boxes, "            ")?;
    Ok(format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<project xmlns="http://www.plcopen.org/xml/tc6_0201" xmlns:xhtml="http://www.w3.org/1999/xhtml">
  <fileHeader companyName="Twinova" productName="Twinova" productVersion="{version}" creationDateTime="{created}"/>
  <contentHeader name="{name}">
    <coordinateInfo>
      <fbd><scaling x="1" y="1"/></fbd>
      <ld><scaling x="1" y="1"/></ld>
      <sfc><scaling x="1" y="1"/></sfc>
    </coordinateInfo>
  </contentHeader>
  <types>
    <dataTypes/>
    <pous>
      <pou name="{name}" pouType="program">
{interface}
        <body>
          <LD>
{body}
          </LD>
        </body>
      </pou>
    </pous>
  </types>
  <instances>
    <configurations/>
  </instances>
</project>
"#,
        version = env!("CARGO_PKG_VERSION"),
        created = esc(created),
        name = esc(name),
    ))
}

/// The program POUs of TC6 project `xml` whose body is Ladder, each as `plc/<name>.ld`.
pub fn import_ladder(xml: &str) -> Result<Imported, String> {
    let document =
        roxmltree::Document::parse(xml).map_err(|e| format!("the file is not XML: {e}"))?;
    let root = document.root_element();
    if root.tag_name().name() != "project" {
        return Err("the file is not a PLCopen project: its root is not <project>".into());
    }
    let mut imported = Imported {
        files: Vec::new(),
        notes: Vec::new(),
    };
    for pou in root.descendants().filter(|n| n.tag_name().name() == "pou") {
        let name = pou.attribute("name").unwrap_or_default();
        match ladder_file(pou, name) {
            Ok(text) => imported.files.push((format!("plc/{name}.ld"), text)),
            Err(note) => imported.notes.push(note),
        }
    }
    Ok(imported)
}

/// POU `name` as a Ladder file's text, or why it is not one.
fn ladder_file(pou: roxmltree::Node, name: &str) -> Result<String, String> {
    let Some(ld) = named(pou, "body").and_then(|b| named(b, "LD")) else {
        return Err(format!("POU '{name}' is not Ladder, so it was left out"));
    };
    let kind = pou.attribute("pouType").unwrap_or_default();
    if kind != "program" {
        return Err(format!(
            "POU '{name}' is a {kind}: a Ladder file here is a program, so it was left out"
        ));
    }
    program_name(&format!("{name}.ld"))
        .map_err(|_| format!("POU '{name}': its name is not an identifier"))?;
    let declarations = match named(pou, "interface") {
        Some(interface) => {
            declarations_text(interface).map_err(|why| format!("POU '{name}': {why}"))?
        }
        None => String::new(),
    };
    let rungs = read_ld_body(ld, name)?;
    let text = format!("{declarations}\n{rungs}");
    parse(&text).map_err(|e| {
        format!(
            "POU '{name}' reads back as a Ladder file that does not: {}",
            e.message
        )
    })?;
    Ok(text)
}

#[cfg(test)]
#[path = "plcopen_tests.rs"]
mod tests;
