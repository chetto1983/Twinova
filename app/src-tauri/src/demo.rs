//! The M3 demo as a project, bundled with the app (spec 2026-11-09-component-behaviour-design.md, "The M3 demo becomes a
//! project"). Its files are the `demo_m3` fixture's, so the demo the app opens and the corpus the tests read are one
//! source.

use twinova_project::{Files, Project, parse_project};

macro_rules! demo_files {
    ($($path:literal),* $(,)?) => {
        [$((
            $path,
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/project/tests/fixtures/valid/demo_m3/", $path))
                .as_slice(),
        )),*]
    };
}

const FILES: [(&str, &[u8]); 8] = demo_files![
    "twinova.json",
    "scene.json",
    "plc/Main.st",
    "components/twinova.belt@1.json",
    "components/twinova.belt@1.st",
    "components/twinova.sensor_diffuse@1.json",
    "components/twinova.sink@1.json",
    "components/twinova.source@1.json",
];

/// The demo, as the app opens it at start.
pub fn demo() -> Project {
    let files: Files = FILES
        .iter()
        .map(|(path, bytes)| (path.to_string(), bytes.to_vec()))
        .collect();
    parse_project(&files)
        .expect("the bundled demo is a valid project (tests/corpus.rs)")
        .project
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use twinova_project::storage::open_folder;

    #[test]
    fn the_bundled_demo_is_the_demo_m3_fixture() {
        let dir =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("project/tests/fixtures/valid/demo_m3");
        assert_eq!(demo(), open_folder(&dir).unwrap().opened.project);
    }
}
