//! Undo and redo of a working project (spec 2026-11-11-editor-basics-design.md): each edit records the whole project as
//! it was, so undoing gives back the scene and the definitions before it (OIP keeps whole documents too). A failed edit
//! records nothing; a new edit clears what could be redone.
//!
//! The ST sources are not undone: they are edited in the editor and enter the project at Build, so undoing a scene edit
//! keeps the program and every block's current text. A block the earlier project pins and the current one does not
//! comes back with the definition.

use crate::Project;

/// How many edits can be undone.
pub const DEPTH: usize = 100;

#[derive(Debug, Default)]
pub struct History {
    undo: Vec<Project>,
    redo: Vec<Project>,
}

impl History {
    /// Applies `edit` to `project`, and records the project before it if the edit succeeds; a failed edit leaves the
    /// project and the history as they were.
    pub fn apply<T>(
        &mut self,
        project: &mut Project,
        edit: impl FnOnce(&mut Project) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut next = project.clone();
        let value = edit(&mut next)?;
        self.undo.push(std::mem::replace(project, next));
        if self.undo.len() > DEPTH {
            self.undo.remove(0);
        }
        self.redo.clear();
        Ok(value)
    }

    /// Back to the project before the last edit; false when there is none.
    pub fn undo(&mut self, project: &mut Project) -> bool {
        Self::swap(&mut self.undo, &mut self.redo, project)
    }

    /// Forward to the project the last undo left; false when there is none.
    pub fn redo(&mut self, project: &mut Project) -> bool {
        Self::swap(&mut self.redo, &mut self.undo, project)
    }

    fn swap(from: &mut Vec<Project>, to: &mut Vec<Project>, project: &mut Project) -> bool {
        let Some(mut other) = from.pop() else {
            return false;
        };
        other.program = project.program.clone();
        for (key, text) in &mut other.behaviours {
            if let Some(current) = project.behaviours.get(key) {
                text.clone_from(current);
            }
        }
        to.push(std::mem::replace(project, other));
        true
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Forgets every step: another project was opened.
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit;

    fn project() -> Project {
        let dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/valid/demo_m3");
        crate::storage::open_folder(&dir).unwrap().opened.project
    }

    fn rename(name: &str) -> impl FnOnce(&mut Project) -> Result<(), String> + '_ {
        move |p| edit::set_name(p, "belt_a", name)
    }

    #[test]
    fn undo_gives_back_each_earlier_project_and_redo_the_later_ones() {
        let (mut p, mut h) = (project(), History::default());
        let mut states = vec![p.clone()];
        for name in ["one", "two", "three"] {
            h.apply(&mut p, rename(name)).unwrap();
            states.push(p.clone());
        }
        assert!(h.can_undo() && !h.can_redo());
        for k in (0..3).rev() {
            assert!(h.undo(&mut p));
            assert_eq!(p, states[k]);
        }
        assert!(!h.undo(&mut p), "nothing before the first edit");
        assert_eq!(p, states[0]);
        for state in &states[1..] {
            assert!(h.redo(&mut p));
            assert_eq!(&p, state);
        }
        assert!(!h.redo(&mut p));
    }

    #[test]
    fn a_new_edit_clears_redo_and_a_failed_one_records_nothing() {
        let (mut p, mut h) = (project(), History::default());
        h.apply(&mut p, rename("one")).unwrap();
        h.undo(&mut p);
        assert!(h.can_redo());
        h.apply(&mut p, rename("two")).unwrap();
        assert!(!h.can_redo());
        let before = p.clone();
        assert!(h.apply(&mut p, rename("  ")).is_err());
        assert_eq!(p, before);
        assert!(h.undo(&mut p));
        assert!(!h.can_undo(), "the failed edit was not recorded");
    }

    #[test]
    fn undo_and_redo_keep_the_current_st_sources() {
        let (mut p, mut h) = (project(), History::default());
        h.apply(&mut p, |p| edit::remove_instance(p, "belt_a"))
            .unwrap();
        // A Build since put new sources in: undoing the removal keeps them.
        p.program[0].1 = "PROGRAM Main (* built *) END_PROGRAM".into();
        p.behaviours
            .insert("twinova.belt@1".into(), "(* belt, edited *)".into());
        h.undo(&mut p);
        assert!(p.scene.instances.iter().any(|i| i.id == "belt_a"));
        assert_eq!(p.program[0].1, "PROGRAM Main (* built *) END_PROGRAM");
        assert_eq!(p.behaviours["twinova.belt@1"], "(* belt, edited *)");
        h.redo(&mut p);
        assert_eq!(p.program[0].1, "PROGRAM Main (* built *) END_PROGRAM");
        assert_eq!(p.behaviours["twinova.belt@1"], "(* belt, edited *)");
    }

    #[test]
    fn a_block_comes_back_with_the_definition_an_undo_restores() {
        let (mut p, mut h) = (project(), History::default());
        let block = p.behaviours["twinova.belt@1"].clone();
        h.apply(&mut p, |p| {
            p.definitions.remove("twinova.belt@1");
            p.behaviours.remove("twinova.belt@1");
            Ok(())
        })
        .unwrap();
        h.undo(&mut p);
        assert_eq!(p.behaviours["twinova.belt@1"], block);
    }

    #[test]
    fn the_depth_is_bounded_and_clear_forgets_everything() {
        let (mut p, mut h) = (project(), History::default());
        for n in 0..DEPTH + 5 {
            h.apply(&mut p, |p| edit::set_name(p, "belt_a", &format!("n{n}")))
                .unwrap();
        }
        let undone = (0..2 * DEPTH).take_while(|_| h.undo(&mut p)).count();
        assert_eq!(undone, DEPTH);
        assert_eq!(
            p.scene
                .instances
                .iter()
                .find(|i| i.id == "belt_a")
                .unwrap()
                .name,
            "n4"
        );
        h.clear();
        assert!(!h.can_undo() && !h.can_redo());
    }
}
