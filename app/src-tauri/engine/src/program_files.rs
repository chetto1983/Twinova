//! The program as truST compiles it (spec 2026-11-13-ladder-editor-design.md, decision 2): an ST file as written, a
//! Ladder file (`.ld`) lowered to a PROGRAM named after it, which runs after the ST files' programs (measured, Task 1).
//!
//! Lowering a box needs to know what its head names. truST tells: the ST files are compiled once with, for each Ladder
//! file, a PROGRAM holding only its declarations, and the compiled runtime holds every program's variables with their
//! types, the function blocks (the standard library's and the project's) with their pins, and the functions.

use trust_hir::symbols::ParamDirection;
use trust_runtime::Runtime;
use trust_runtime::harness::TestHarness;
use twinova_ladder::{
    Block, BlockPin, Lowered, Origin, declarations_only, lower, parse, program_name,
};

use crate::locate::line_of;

/// A Ladder file read: its path, its text, its program's name, its rungs and its declarations alone.
type Parsed<'a> = (
    &'a String,
    &'a String,
    &'a str,
    twinova_ladder::Program,
    String,
);

struct Ladder {
    path: String,
    text: String,
    lowered: Lowered,
}

/// The files truST compiles, in the project's order, and the Ladder files they were lowered from.
pub struct ProgramFiles {
    pub files: Vec<(String, String)>,
    ladders: Vec<Ladder>,
}

fn is_ladder(path: &str) -> bool {
    path.ends_with(".ld")
}

impl ProgramFiles {
    /// `program` lowered; a Ladder file that does not read, or a rung that does not lower, fails the whole program
    /// with each problem on a line of its own.
    pub fn of(program: &[(String, String)]) -> Result<Self, String> {
        if !program.iter().any(|(path, _)| is_ladder(path)) {
            return Ok(Self {
                files: program.to_vec(),
                ladders: Vec::new(),
            });
        }
        let ladders = lowered(program)?;
        let problems: Vec<String> = (ladders.iter())
            .flat_map(|l| {
                (l.lowered.errors.iter())
                    .map(|e| format!("{}, rung {}: {}", l.path, e.element.rung + 1, e.message))
            })
            .collect();
        if !problems.is_empty() {
            return Err(problems.join("\n"));
        }
        let files = (program.iter())
            .map(
                |(path, text)| match ladders.iter().find(|l| l.path == *path) {
                    Some(ladder) => (path.clone(), ladder.lowered.st.clone()),
                    None => (path.clone(), text.clone()),
                },
            )
            .collect();
        Ok(Self { files, ladders })
    }

    /// The Ladder file `path` of `program` lowered, its rungs that do not lower reported in it rather than failing it:
    /// what the editor draws and marks.
    pub fn ladder(program: &[(String, String)], path: &str) -> Result<Lowered, String> {
        (lowered(program)?.into_iter())
            .find(|l| l.path == path)
            .map(|l| l.lowered)
            .ok_or_else(|| format!("{path} is not a Ladder file of the program"))
    }

    /// Where a byte of the compiled file `path` comes from, as a person reads it: `plc/Main.st:12`, or
    /// `plc/Pump.ld, rung 3` (a rung's text has no lines of its own in the editor). None for a file it did not compile.
    pub(crate) fn placer(&self, path: &str) -> Option<impl Fn(usize) -> String + '_> {
        let (path, text) = self.files.iter().find(|(p, _)| p == path)?;
        let ladder = self.ladders.iter().find(|l| l.path == *path);
        Some(move |start: usize| {
            let Some(ladder) = ladder else {
                return format!("{path}:{}", line_of(text, start));
            };
            match ladder.lowered.origin(start) {
                Some(Origin::Declarations(offset)) => {
                    let leading = ladder.text.len() - ladder.text.trim_start().len();
                    format!("{path}:{}", line_of(&ladder.text, leading + offset))
                }
                Some(Origin::Element(element, _)) => format!("{path}, rung {}", element.rung + 1),
                // the PROGRAM's own lines and its hidden variables
                None => path.clone(),
            }
        })
    }
}

/// Every Ladder file of `program` lowered, each box resolved by `declarations_compiled`; a file that does not read
/// fails them all.
fn lowered(program: &[(String, String)]) -> Result<Vec<Ladder>, String> {
    let mut parsed = Vec::new();
    for (path, text) in program.iter().filter(|(path, _)| is_ladder(path)) {
        let name = program_name(path)?;
        let unreadable = |e: twinova_ladder::ParseError| {
            format!("{path}:{}: {}", line_of(text, e.offset), e.message)
        };
        let ladder = parse(text).map_err(unreadable)?;
        let declarations = declarations_only(name, text).map_err(unreadable)?;
        parsed.push((path, text, name, ladder, declarations));
    }
    let resolver = declarations_compiled(program, &parsed);
    let ladders = (parsed.iter())
        .map(|(path, text, name, ladder, _)| Ladder {
            path: (*path).clone(),
            text: (*text).clone(),
            lowered: lower(ladder, name, &|head| {
                resolver.as_ref().and_then(|r| block(r, name, head))
            }),
        })
        .collect();
    Ok(ladders)
}

/// The ST files and each Ladder file's declarations, compiled. None when they do not compile: every box is then
/// lowered by its bare pins, and the program's own compile reports the problem.
fn declarations_compiled(program: &[(String, String)], ladders: &[Parsed]) -> Option<Runtime> {
    let sources: Vec<&str> = (program.iter())
        .filter(|(path, _)| !is_ladder(path))
        .map(|(_, text)| text.as_str())
        .chain(ladders.iter().map(|(_, _, _, _, only)| only.as_str()))
        .collect();
    TestHarness::from_sources(&sources)
        .ok()
        .map(TestHarness::into_runtime)
}

/// What `head` names in program `program`: a variable whose type is a function block, a function of the project or
/// of the standard library, or nothing known.
fn block(runtime: &Runtime, program: &str, head: &str) -> Option<Block> {
    let vars = (runtime.programs().iter())
        .find(|(name, _)| name.eq_ignore_ascii_case(program))
        .map(|(_, p)| &p.vars)?;
    if let Some(var) = vars.iter().find(|v| v.name.eq_ignore_ascii_case(head)) {
        let registry = runtime.registry();
        let type_name = registry.type_name(var.type_id)?;
        let fb = runtime
            .function_blocks()
            .get(type_name.to_ascii_uppercase().as_str())?;
        let pins = |direction: ParamDirection| {
            (fb.params.iter())
                .filter(|p| p.direction == direction)
                .map(|p| BlockPin {
                    name: p.name.to_string(),
                    bool: registry.type_name(p.type_id).as_deref() == Some("BOOL"),
                })
                .collect()
        };
        return Some(Block::Instance {
            type_name: fb.name.to_string(),
            inputs: pins(ParamDirection::In),
            outputs: pins(ParamDirection::Out),
        });
    }
    let function = runtime
        .functions()
        .contains_key(head.to_ascii_uppercase().as_str())
        || runtime.stdlib().get(head).is_some();
    function.then_some(Block::Function)
}

#[cfg(test)]
#[path = "program_files_tests.rs"]
pub(crate) mod tests;
