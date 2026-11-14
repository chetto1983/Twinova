//! One rung lowered to ST, element by element (`lower.rs` holds the semantics). A power is the text of a BOOL: `TRUE`
//! at the left rail, else the hidden variable holding the power out of the element before.

use crate::lower::{Block, BoxPins, ElementId, Field, LowerError, Names, Power, Segment};
use crate::model::{CoilMode, ContactMode, Element, Exit, Pin, Rung};

const RAIL: &str = "TRUE";

/// Functions ST spells as operators: truST does not parse `AND(a, b)` or `MOD(a, b)` as calls (measured, spec).
const OPERATORS: [&str; 5] = ["AND", "OR", "XOR", "MOD", "NOT"];

/// The generated text of a rung, or of a line of the program's scaffolding, with what each part of it comes from.
#[derive(Debug, Default)]
pub(crate) struct Part {
    pub text: String,
    pub segments: Vec<Segment>,
    pub lines: Vec<(usize, ElementId)>,
    pub temps: Vec<String>,
    pub power: Vec<Power>,
    pub boxes: Vec<(ElementId, BoxPins)>,
}

impl Part {
    pub(crate) fn scaffold(text: &str, rung: usize) -> Part {
        Part {
            text: format!("{text}\n"),
            lines: vec![(
                0,
                ElementId {
                    rung,
                    path: Vec::new(),
                },
            )],
            ..Part::default()
        }
    }
}

pub(crate) struct RungLowerer<'a> {
    rung: usize,
    block: &'a dyn Fn(&str) -> Option<Block>,
    names: &'a mut Names,
    out: Part,
    line_start: usize,
}

type Lowering<T> = Result<T, LowerError>;

impl<'a> RungLowerer<'a> {
    pub(crate) fn new(
        rung: usize,
        block: &'a dyn Fn(&str) -> Option<Block>,
        names: &'a mut Names,
    ) -> Self {
        RungLowerer {
            rung,
            block,
            names,
            out: Part::default(),
            line_start: 0,
        }
    }

    fn id(&self, path: &[usize]) -> ElementId {
        ElementId {
            rung: self.rung,
            path: path.to_vec(),
        }
    }

    fn error(&self, path: &[usize], message: String) -> LowerError {
        LowerError {
            element: self.id(path),
            message,
        }
    }

    /// `jump_flag`: the flag the rung's jump sets, when it ends in one.
    pub(crate) fn rung(mut self, rung: &Rung, jump_flag: Option<&str>) -> Lowering<Part> {
        if let Some((path, what)) = first_blank(&rung.elements, &[]) {
            return Err(self.error(&path, format!("{what} yet: the ? on this rung")));
        }
        let power = self.series(&rung.elements, &[], RAIL.into(), true)?;
        match (&rung.exit, jump_flag) {
            (Some(Exit::Jump { .. }), Some(flag)) => self.put(&format!("{flag} := {power};")),
            (Some(Exit::Return), _) if power == RAIL => self.put("RETURN;"),
            (Some(Exit::Return), _) => self.put(&format!("IF {power} THEN RETURN; END_IF;")),
            _ => return Ok(self.out),
        }
        self.end_line(&[]);
        Ok(self.out)
    }

    // ---- writing the ST ----

    fn put(&mut self, text: &str) {
        self.out.text.push_str(text);
    }

    fn operand(&mut self, text: &str, path: &[usize], field: Field) {
        self.out.segments.push(Segment {
            at: self.out.text.len(),
            len: text.len(),
            element: self.id(path),
            field,
        });
        self.put(text);
    }

    fn end_line(&mut self, path: &[usize]) {
        self.out.text.push('\n');
        let id = self.id(path);
        self.out.lines.push((self.line_start, id));
        self.line_start = self.out.text.len();
    }

    fn temp(&mut self, stem: &'static str) -> String {
        let name = self.names.fresh(stem);
        self.out.temps.push(name.clone());
        name
    }

    fn resolved(
        &mut self,
        path: &[usize],
        function: bool,
        type_name: &str,
        input: Option<&str>,
        output: &str,
    ) {
        let element = self.id(path);
        self.out.boxes.push((
            element,
            BoxPins {
                function,
                type_name: type_name.into(),
                input: input.map(Into::into),
                output: output.into(),
            },
        ));
    }

    fn record(&mut self, path: &[usize], variable: &str) {
        let element = self.id(path);
        self.out.power.push(Power {
            element,
            variable: variable.into(),
        });
    }

    /// `power AND …`, with the rail's `TRUE` left out.
    fn and(&mut self, power: &str) {
        if power != RAIL {
            self.put(power);
            self.put(" AND ");
        }
    }

    // ---- the elements ----

    /// `rail`: nothing stands between the series and the left rail, the only place a function may start a rung.
    fn series(
        &mut self,
        elements: &[Element],
        prefix: &[usize],
        mut power: String,
        mut rail: bool,
    ) -> Lowering<String> {
        for (index, element) in elements.iter().enumerate() {
            let path = [prefix, &[index]].concat();
            power = match element {
                Element::Contact { mode, operand } => {
                    self.contact(*mode, operand.as_deref().unwrap_or_default(), &path, &power)
                }
                Element::Coil { mode, operand } => {
                    self.coil(*mode, operand.as_deref().unwrap_or_default(), &path, &power);
                    power
                }
                Element::Branch { paths } => self.branch(paths, &path, &power, rail)?,
                Element::Box { head, pins } => self.boxed(
                    head.as_deref().unwrap_or_default(),
                    pins,
                    &path,
                    &power,
                    rail,
                )?,
            };
            self.record(&path, &power);
            rail = false;
        }
        Ok(power)
    }

    fn contact(&mut self, mode: ContactMode, x: &str, path: &[usize], power: &str) -> String {
        let out = self.temp("LD_P");
        // a P or N contact passes power for one evaluation after its variable changed, and its memory follows the
        // variable every time the element is evaluated, whatever its left link does (IEC 61131-3, 8.2.3)
        let memory =
            matches!(mode, ContactMode::Rising | ContactMode::Falling).then(|| self.temp("LD_M"));
        self.put(&format!("{out} := "));
        self.and(power);
        if matches!(mode, ContactMode::Closed | ContactMode::Falling) {
            self.put("NOT ");
        }
        self.operand(x, path, Field::Operand);
        match (&memory, mode) {
            (Some(m), ContactMode::Rising) => self.put(&format!(" AND NOT {m};")),
            (Some(m), _) => self.put(&format!(" AND {m};")),
            (None, _) => self.put(";"),
        }
        self.end_line(path);
        if let Some(m) = memory {
            self.put(&format!("{m} := "));
            self.operand(x, path, Field::Operand);
            self.put(";");
            self.end_line(path);
        }
        out
    }

    /// A coil passes its power on unchanged.
    fn coil(&mut self, mode: CoilMode, x: &str, path: &[usize], power: &str) {
        match mode {
            CoilMode::Normal | CoilMode::Negated => {
                self.operand(x, path, Field::Operand);
                let not = if mode == CoilMode::Negated {
                    "NOT "
                } else {
                    ""
                };
                self.put(&format!(" := {not}{power};"));
            }
            CoilMode::Set | CoilMode::Reset => {
                let value = if mode == CoilMode::Set {
                    "TRUE"
                } else {
                    "FALSE"
                };
                self.guarded(power, |me| {
                    me.operand(x, path, Field::Operand);
                    me.put(&format!(" := {value};"));
                });
            }
            CoilMode::Rising | CoilMode::Falling => {
                // TRUE for one evaluation after its power changed
                let memory = self.temp("LD_M");
                self.operand(x, path, Field::Operand);
                if mode == CoilMode::Rising {
                    self.put(&format!(" := {power} AND NOT {memory};"));
                } else {
                    self.put(&format!(" := NOT {power} AND {memory};"));
                }
                self.end_line(path);
                self.put(&format!("{memory} := {power};"));
            }
        }
        self.end_line(path);
    }

    /// `IF power THEN … END_IF;` around what `body` writes, or the body alone at the rail.
    fn guarded(&mut self, power: &str, body: impl FnOnce(&mut Self)) {
        if power == RAIL {
            body(self);
        } else {
            self.put(&format!("IF {power} THEN "));
            body(self);
            self.put(" END_IF;");
        }
    }

    /// Every path starts from the branch's power; the branch's power out is their OR.
    fn branch(
        &mut self,
        paths: &[Vec<Element>],
        path: &[usize],
        power: &str,
        rail: bool,
    ) -> Lowering<String> {
        let mut outs = vec![String::new(); paths.len()];
        for index in branch_order(paths) {
            let prefix = [path, &[index]].concat();
            outs[index] = self.series(&paths[index], &prefix, power.into(), rail)?;
        }
        let mut distinct: Vec<String> = Vec::new();
        for out in outs {
            if !distinct.contains(&out) {
                distinct.push(out);
            }
        }
        if distinct.iter().any(|o| o == RAIL) {
            return Ok(RAIL.into());
        }
        if let [only] = &distinct[..] {
            return Ok(only.clone());
        }
        let out = self.temp("LD_P");
        self.put(&format!("{out} := {};", distinct.join(" OR ")));
        self.end_line(path);
        Ok(out)
    }

    fn boxed(
        &mut self,
        head: &str,
        pins: &[Pin],
        path: &[usize],
        power: &str,
        rail: bool,
    ) -> Lowering<String> {
        if pins
            .iter()
            .any(|p| p.value.is_some() && p.name.eq_ignore_ascii_case("EN"))
        {
            return Err(self.error(
                path,
                "EN is the pin the rung feeds: write it bare, or leave it out".into(),
            ));
        }
        let operator = OPERATORS.iter().any(|o| o.eq_ignore_ascii_case(head));
        match if operator {
            Some(Block::Function)
        } else {
            (self.block)(head)
        } {
            Some(Block::Function) => self.function(head, pins, path, power, rail),
            Some(Block::Instance {
                type_name,
                inputs,
                outputs,
            }) => self.instance(
                head,
                pins,
                path,
                power,
                Some((&type_name, &inputs, &outputs)),
            ),
            None => self.instance(head, pins, path, power, None),
        }
    }
}

#[path = "lower_boxes.rs"]
mod boxes;

/// The order a branch's paths run in. IEC leaves it open (8.2.3), so this is OIP's rule: the paths holding a contact or
/// a box first, top to bottom, then the paths of coils only. So in `|A| [(X), |X| (Y)]` the contact examines X as it
/// was, while in `|A| [|A| (X), |X| (Y)]` it sees the new X.
pub(crate) fn branch_order(paths: &[Vec<Element>]) -> Vec<usize> {
    let coils_only =
        |p: &Vec<Element>| !p.is_empty() && p.iter().all(|e| matches!(e, Element::Coil { .. }));
    let (coils, others): (Vec<usize>, Vec<usize>) =
        (0..paths.len()).partition(|&i| coils_only(&paths[i]));
    [others, coils].concat()
}

/// The first operand of `elements` still blank (`?`), with what it lacks, or None.
fn first_blank(elements: &[Element], prefix: &[usize]) -> Option<(Vec<usize>, String)> {
    for (index, element) in elements.iter().enumerate() {
        let path = [prefix, &[index]].concat();
        let lacks = match element {
            Element::Branch { paths } => {
                let found = paths
                    .iter()
                    .enumerate()
                    .find_map(|(i, p)| first_blank(p, &[&path[..], &[i]].concat()));
                if found.is_some() {
                    return found;
                }
                None
            }
            Element::Contact { operand: None, .. } => {
                Some("This contact has no variable".to_string())
            }
            Element::Coil { operand: None, .. } => Some("This coil has no variable".to_string()),
            Element::Box { head: None, .. } => Some("This block has no name".to_string()),
            Element::Box {
                head: Some(head),
                pins,
            } => pins
                .iter()
                .find(|p| p.value.as_deref() == Some("?"))
                .map(|p| format!("{head}'s {} has no value", p.name)),
            _ => None,
        };
        if let Some(lacks) = lacks {
            return Some((path, lacks));
        }
    }
    None
}
