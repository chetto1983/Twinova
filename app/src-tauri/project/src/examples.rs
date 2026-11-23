//! The example lines, bundled with the app and the agent (spec 2026-11-23-app-mcp-design.md, decision 5): the valid
//! fixtures of `tests/fixtures/valid`, so the examples a person opens and the corpus the tests read are one source. Each
//! embeds its own manifest, scene and program; the families it pins are the library's files, which its copies equal
//! (`library_tests.rs`), and `parse_project` ignores the ones its manifest does not list.

use serde::Serialize;

use crate::codec::{Files, parse_project};
use crate::library;
use crate::model::Project;

macro_rules! example {
    ($id:literal, $program:literal) => {
        (
            $id,
            [
                (
                    "twinova.json",
                    include_bytes!(concat!("../tests/fixtures/valid/", $id, "/twinova.json"))
                        .as_slice(),
                ),
                (
                    "scene.json",
                    include_bytes!(concat!("../tests/fixtures/valid/", $id, "/scene.json"))
                        .as_slice(),
                ),
                (
                    $program,
                    include_bytes!(concat!("../tests/fixtures/valid/", $id, "/", $program))
                        .as_slice(),
                ),
            ],
        )
    };
}

/// The example the app opens at start.
pub const DEMO: &str = "demo_m3";

/// An example's own files, by path: its manifest, its scene and its program.
type Own = [(&'static str, &'static [u8]); 3];

/// Every example by id, in the order the project menu lists them.
const EXAMPLES: [(&str, Own); 7] = [
    example!("demo_m3", "plc/Main.st"),
    example!("merge_line", "plc/Main.st"),
    example!("two_belts_diverter", "plc/Main.st"),
    example!("ladder_line", "plc/Main.ld"),
    example!("curve_line", "plc/Main.st"),
    example!("sort_line", "plc/Main.st"),
    example!("code_sort_line", "plc/Main.st"),
];

/// An example as the menu lists it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Example {
    pub id: &'static str,
    pub name: String,
}

/// The examples, in the menu's order.
pub fn examples() -> Vec<Example> {
    EXAMPLES
        .iter()
        .map(|(id, own)| Example {
            id,
            name: parsed(own).name,
        })
        .collect()
}

/// Example `id` as a project with no folder, or None when there is none by that id.
pub fn example(id: &str) -> Option<Project> {
    let (_, own) = EXAMPLES.iter().find(|(known, _)| *known == id)?;
    Some(parsed(own))
}

fn parsed(own: &[(&str, &[u8])]) -> Project {
    let mut files: Files = (library::files().into_iter())
        .map(|(path, text)| (path, text.as_bytes().to_vec()))
        .collect();
    files.extend(
        own.iter()
            .map(|(path, bytes)| (path.to_string(), bytes.to_vec())),
    );
    parse_project(&files)
        .expect("a bundled example is a valid project (examples_tests.rs)")
        .project
}

#[cfg(test)]
#[path = "examples_tests.rs"]
mod tests;
