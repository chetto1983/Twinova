//! Build: a project checked as the format checks it on open, then its engine at time zero. On failure nothing changes
//! for the caller, which keeps its running line (spec 2026-11-09-component-behaviour-design.md, "Build").

use twinova_project::{Diagnostic, Project};
use twinova_project::{encode_project, parse_project};

use crate::engine::Engine;

/// The format's diagnostics, one a line.
pub fn diagnostics(found: &[Diagnostic]) -> String {
    found
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

/// `project` as the format reads it back, its engine at time zero, and Build's warnings.
pub fn build(project: &Project) -> Result<(Project, Engine, Vec<String>), String> {
    let checked = parse_project(&encode_project(project))
        .map_err(|d| diagnostics(&d))?
        .project;
    let (engine, warnings) = Engine::new(&checked)?;
    Ok((checked, engine, warnings))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use twinova_project::storage::open_folder;

    fn demo() -> Project {
        let dir =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../project/tests/fixtures/valid/demo_m3");
        open_folder(&dir).unwrap().opened.project
    }

    /// The demo builds with its warning; a project the format refuses fails with each diagnostic on its own line.
    #[test]
    fn build_checks_a_project_as_the_format_does_on_open() {
        let (project, engine, warnings) = build(&demo()).unwrap();
        assert_eq!((project, engine.time_ms()), (demo(), 0));
        assert_eq!(
            warnings,
            ["sink.count at %ID4: the program does not declare it"]
        );
        let mut broken = demo();
        for instance in &mut broken.scene.instances {
            for address in instance.signals.values_mut() {
                *address = "nowhere".into();
            }
        }
        let Err(error) = build(&broken) else {
            panic!("a project with bad addresses built")
        };
        let lines: Vec<&str> = error.lines().collect();
        assert!(lines.len() > 1, "{error}");
        assert!(
            lines
                .iter()
                .all(|l| l.contains("scene.json") && l.contains("nowhere")),
            "{error}"
        );
    }
}
