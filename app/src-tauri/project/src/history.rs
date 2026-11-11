//! Undo and redo of a working project (spec 2026-11-11-editor-basics-design.md): each edit records the whole project as
//! it was, so undoing gives back exactly the project before it (OIP keeps whole documents too). A failed edit records
//! nothing; a new edit clears what could be redone.

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
        match from.pop() {
            Some(earlier) => {
                to.push(std::mem::replace(project, earlier));
                true
            }
            None => false,
        }
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
