//! The field runtime (spec 2026-11-09-component-behaviour-design.md): a second truST runtime that simulates the
//! devices whose definitions have an ST behaviour, beside the line controller. It sees the controller's outputs as
//! its inputs and writes the controller's inputs, at the mirrored address (`%QX0.0` ↔ `%IX0.0`), and it scans every
//! millisecond. Twinova generates its program and configuration from the project at Build; nothing of it is stored.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use trust_runtime::harness::TestHarness;
use trust_runtime::io::IoAddress;
use trust_runtime::memory::InstanceId;
use trust_runtime::value::{Duration, Value};
use twinova_project::Project;
use twinova_project::address::{Address, Area};
use twinova_project::model::{
    Definition, Direction, Instance, ParamType, ParamValue, SignalDef, SignalType,
};

use crate::field_image::place;

/// The field's cycle, fixed (user, 2026-11-09).
pub const FIELD_CYCLE_NS: i64 = 1_000_000;

/// The instance of the generated `PROGRAM Field`.
const PROGRAM: &str = "F";

/// The variables a family's native model reads from its behaviour block, by generator.
fn natives(generator: &str) -> &'static [(&'static str, &'static str)] {
    match generator {
        "twinova.belt/1"
        | "twinova.roller/1"
        | "twinova.belt_curve/1"
        | "twinova.roller_curve/1" => &[("surface_speed", "REAL")],
        "twinova.chain_transfer/1" => &[("lift", "REAL"), ("raised", "BOOL")],
        _ => &[],
    }
}

/// The signals a family's native model writes itself (drive.rs): its block does not declare them, so the field neither
/// places nor copies them (spec 2026-11-10-run-pace-native-feedback-design.md, decision 2).
const DRIVE_FEEDBACK: [&str; 3] = ["velocity", "encoder", "pulse"];

/// The signals of `def` its behaviour block declares: all but those its native model writes.
pub(crate) fn block_signals(def: &Definition) -> impl Iterator<Item = &SignalDef> {
    let native: &[&str] = match def.generator.as_str() {
        "twinova.belt/1"
        | "twinova.roller/1"
        | "twinova.belt_curve/1"
        | "twinova.roller_curve/1" => &DRIVE_FEEDBACK,
        _ => &[],
    };
    def.signals
        .iter()
        .filter(move |s| !native.contains(&s.name.as_str()))
}

pub struct Field {
    harness: TestHarness,
    /// Each behaving instance's block, by instance id, resolved at Build. A read by path parses the path every time,
    /// 31 % of a run when the belts' `motor` was read so every millisecond (spec, "Amended after the acceptance").
    blocks: BTreeMap<String, InstanceId>,
    /// The controller's output addresses the field reads, as the controller names them.
    commands: Vec<String>,
    /// The controller's input addresses the field writes.
    feedback: Vec<String>,
    /// The field's own address of each command and feedback, by the controller's: its mirror, parsed at Build.
    /// Mirroring and parsing one at every copy of every millisecond cost 3 µs of a line's 50 (spec
    /// 2026-11-10-solver-substeps-io-links-design.md, measured).
    mirrors: BTreeMap<String, IoAddress>,
}

/// The same byte and bit seen from the other runtime: input and output swap.
pub fn mirror(address: &str) -> Result<String, String> {
    let mut a = Address::parse(address)?;
    a.area = match a.area {
        Area::Input => Area::Output,
        Area::Output => Area::Input,
        Area::Memory => return Err(format!("{address} is not an input or an output")),
    };
    Ok(a.to_string())
}

/// The field's address of the command or feedback the controller has at `address`.
fn mirrored<'a>(
    mirrors: &'a BTreeMap<String, IoAddress>,
    address: &str,
) -> Result<&'a IoAddress, String> {
    mirrors
        .get(address)
        .ok_or_else(|| format!("{address}: the field has no signal there"))
}

pub(crate) fn behaving(project: &Project) -> Vec<(&Instance, &Definition)> {
    let mut out: Vec<_> = project
        .scene
        .instances
        .iter()
        .filter_map(|i| project.definitions.get(&i.definition).map(|d| (i, d)))
        .filter(|(_, d)| d.behaviour.is_some())
        .collect();
    out.sort_by(|a, b| a.0.id.cmp(&b.0.id));
    out
}

fn st_type(ty: SignalType) -> &'static str {
    match ty {
        SignalType::BOOL => "BOOL",
        SignalType::DINT => "DINT",
        SignalType::REAL => "REAL",
    }
}

/// Where each behaviour block sits in the generated source: (start byte, end byte, file).
type Spans = Vec<(usize, usize, String)>;

/// The program, the configuration and the blocks they use, in one source, with where each block sits.
fn generate(project: &Project) -> Result<(String, Spans), String> {
    let devices = behaving(project);
    let mut out = String::new();
    let mut spans = Spans::new();
    let blocks: BTreeSet<&str> = devices.iter().map(|(i, _)| i.definition.as_str()).collect();
    for key in blocks {
        let start = out.len();
        out.push_str(&project.behaviours[key]);
        spans.push((start, out.len(), format!("components/{key}.st")));
        out.push('\n');
    }
    out.push_str("PROGRAM Field\nVAR\n");
    for (inst, def) in &devices {
        let block = &def.behaviour.as_ref().expect("filtered").function_block;
        let _ = writeln!(out, "    {} : {block};", inst.id);
    }
    out.push_str("END_VAR\n");
    for (inst, _) in &devices {
        let _ = writeln!(out, "{}();", inst.id);
    }
    out.push_str("END_PROGRAM\n\nCONFIGURATION Twinova\nRESOURCE Devices ON PLC\n");
    out.push_str("    TASK Cycle (INTERVAL := T#1ms, PRIORITY := 1);\n");
    let _ = writeln!(out, "    PROGRAM {PROGRAM} WITH Cycle : Field;");
    out.push_str("END_RESOURCE\n");
    let mut config = String::new();
    for placed in place(&devices)? {
        let _ = writeln!(
            config,
            "    {PROGRAM}.{}.{} AT {} : {};",
            placed.instance.id,
            placed.signal.name,
            placed.address,
            st_type(placed.signal.ty)
        );
    }
    if !config.is_empty() {
        out.push_str("VAR_CONFIG\n");
        out.push_str(&config);
        out.push_str("END_VAR\n");
    }
    out.push_str("END_CONFIGURATION\n");
    Ok((out, spans))
}

/// truST's compile messages locate a problem as `at <start>..<end>`, in bytes of the compiled source (measured on
/// `6405915`, 2026-11-09; `CompileError` carries only its message). The field compiles one generated source, so each
/// location is rewritten as the behaviour file and line it falls in.
fn locate(message: &str, text: &str, spans: &Spans) -> String {
    let mut out = String::new();
    let mut rest = message;
    while let Some(i) = rest.find("at ") {
        let (before, after) = rest.split_at(i + 3);
        out.push_str(before);
        let digits = |s: &str| s.bytes().take_while(u8::is_ascii_digit).count();
        let start_len = digits(after);
        let end_len = after[start_len..].strip_prefix("..").map_or(0, digits);
        if start_len == 0 || end_len == 0 {
            rest = after;
            continue;
        }
        let start: usize = after[..start_len].parse().expect("digits");
        match spans.iter().find(|(a, b, _)| (*a..*b).contains(&start)) {
            Some((a, _, file)) => {
                let line = text[*a..start].matches('\n').count() + 1;
                let _ = write!(out, "{file}:{line}");
            }
            None => out.push_str("the generated field program"),
        }
        rest = &after[start_len + 2 + end_len..];
    }
    out.push_str(rest);
    out
}

fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Bool(_) => "BOOL",
        Value::DInt(_) => "DINT",
        Value::Real(_) => "REAL",
        Value::Time(_) => "TIME",
        _ => "another type",
    }
}

fn param_value(ty: ParamType, value: ParamValue) -> Value {
    match (ty, value) {
        (_, ParamValue::Bool(b)) => Value::Bool(b),
        (ParamType::TIME, ParamValue::Int(ns)) => Value::Time(Duration::from_nanos(ns)),
        (ParamType::TIME, ParamValue::Real(ns)) => Value::Time(Duration::from_nanos(ns as i64)),
        (ParamType::REAL, ParamValue::Int(n)) => Value::Real(n as f32),
        (ParamType::REAL, ParamValue::Real(x)) => Value::Real(x as f32),
        (_, ParamValue::Int(n)) => Value::DInt(n as i32),
        (_, ParamValue::Real(x)) => Value::DInt(x as i32),
    }
}

fn param_type(ty: ParamType) -> &'static str {
    match ty {
        ParamType::REAL => "REAL",
        ParamType::DINT => "DINT",
        ParamType::BOOL => "BOOL",
        ParamType::TIME => "TIME",
    }
}

/// The block of each device, from the program instance down: the generated program declares both, so a field that
/// compiled has them.
fn resolve_blocks(
    harness: &TestHarness,
    devices: &[(&Instance, &Definition)],
) -> BTreeMap<String, InstanceId> {
    let storage = harness.runtime().storage();
    let instance = |value: Option<&Value>, name: &str| match value {
        Some(Value::Instance(id)) => *id,
        other => panic!("the generated field declares {name} as an instance, found {other:?}"),
    };
    let program = instance(storage.get_global(PROGRAM), PROGRAM);
    devices
        .iter()
        .map(|(inst, _)| {
            let block = instance(storage.get_instance_var(program, &inst.id), &inst.id);
            (inst.id.clone(), block)
        })
        .collect()
}

impl Field {
    /// Builds the field of `project`, or `None` when no component has a behaviour. A block cannot compile alone,
    /// because its `AT %I*` needs the configuration (E202, measured), so the whole field compiles at once and each
    /// error is located in its behaviour file.
    pub fn build(project: &Project) -> Result<Option<Self>, String> {
        let devices = behaving(project);
        if devices.is_empty() {
            return Ok(None);
        }
        let (text, spans) = generate(project)?;
        let mut harness = TestHarness::from_source(&text)
            .map_err(|e| format!("field runtime: {}", locate(&e.to_string(), &text, &spans)))?;
        for placed in place(&devices)? {
            if let Some(value) = placed.fallback {
                let address = placed.address.to_string();
                harness.set_direct_input(&address, value).map_err(|e| {
                    format!(
                        "{}.{} at {address}: {e}",
                        placed.instance.id, placed.signal.name
                    )
                })?;
            }
        }
        let (mut commands, mut feedback) = (Vec::new(), Vec::new());
        for (inst, def) in &devices {
            let block = &def.behaviour.as_ref().expect("filtered").function_block;
            let path = |name: &str| format!("{PROGRAM}.{}.{name}", inst.id);
            for (name, ty) in natives(&def.generator) {
                match harness.get_output(&path(name)) {
                    Some(v) if type_name(&v) == *ty => {}
                    found => {
                        return Err(format!(
                            "components/{}.st: {block} must declare {name} : {ty}, which the native {} uses{}",
                            def.key(),
                            def.generator,
                            found.map_or(String::new(), |v| format!(" (it is {})", type_name(&v)))
                        ));
                    }
                }
            }
            for p in &def.parameters {
                let Some(current) = harness.get_output(&path(&p.name)) else {
                    continue;
                };
                if type_name(&current) != param_type(p.ty) {
                    return Err(format!(
                        "components/{}.st: {block}.{} must be {} like the parameter, not {}",
                        def.key(),
                        p.name,
                        param_type(p.ty),
                        type_name(&current)
                    ));
                }
                let value = inst.parameters.get(&p.name).copied().unwrap_or(p.default);
                harness
                    .try_set_input(&path(&p.name), param_value(p.ty, value))
                    .map_err(|e| format!("{}.{}: {e}", inst.id, p.name))?;
            }
            for signal in block_signals(def) {
                if let Some(address) = inst.signals.get(&signal.name) {
                    match signal.direction {
                        Direction::PlcOutput => commands.push(address.clone()),
                        Direction::PlcInput => feedback.push(address.clone()),
                    }
                }
            }
        }
        let blocks = resolve_blocks(&harness, &devices);
        let mut mirrors = BTreeMap::new();
        for address in commands.iter().chain(&feedback) {
            let at = IoAddress::parse(&mirror(address)?).map_err(|e| format!("{address}: {e}"))?;
            mirrors.insert(address.clone(), at);
        }
        Ok(Some(Self {
            harness,
            blocks,
            commands,
            feedback,
            mirrors,
        }))
    }

    pub fn commands(&self) -> &[String] {
        &self.commands
    }

    pub fn feedback(&self) -> &[String] {
        &self.feedback
    }

    /// Writes a command the controller output at `address` (its `%Q`), into the field's mirrored input.
    pub fn set_command(&mut self, address: &str, value: Value) -> Result<(), String> {
        let at = mirrored(&self.mirrors, address)?;
        self.harness
            .runtime_mut()
            .write_io(at, value)
            .map_err(|e| format!("{address}: {e}"))
    }

    /// The value the field gives the controller input at `address` (its `%I`).
    pub fn feedback_value(&self, address: &str) -> Result<Value, String> {
        let at = mirrored(&self.mirrors, address)?;
        self.harness
            .runtime()
            .io()
            .read(at)
            .map_err(|e| format!("{address}: {e}"))
    }

    /// A block variable the native model reads, such as a belt's `motor`.
    pub fn native(&self, instance: &str, name: &str) -> Option<Value> {
        let block = *self.blocks.get(instance)?;
        self.harness
            .runtime()
            .storage()
            .get_instance_var(block, name)
            .cloned()
    }

    /// One millisecond of virtual time, then one cycle.
    pub fn cycle(&mut self) -> Result<(), String> {
        self.harness
            .advance_time(Duration::from_nanos(FIELD_CYCLE_NS));
        let cycle = self.harness.cycle();
        match cycle.errors.first() {
            Some(error) => Err(format!("field runtime: {error}")),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
#[path = "field_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "field_drive_tests.rs"]
mod drive_tests;
