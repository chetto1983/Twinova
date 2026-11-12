//! The line controller by address (spec 2026-11-09-component-behaviour-design.md, "The controller by address"):
//! truST's harness at the pinned commit, scanned with virtual time, its I/O read and written through the process
//! image as a real PLC's would be. Twinova never writes a program variable by name.

use trust_runtime::harness::{CompileSession, SourceFile, TestHarness};
use trust_runtime::io::{IoAddress, IoSize};
use trust_runtime::value::{Duration, Value};
use twinova_project::Project;
use twinova_project::model::SignalType;

use crate::locate::locate_in_files;
use crate::timing::ScanTiming;

/// The scan period of a program that declares no periodic `TASK`: M1's 10 ms.
pub const DEFAULT_SCAN_NS: i64 = 10_000_000;

/// A variable the program declares at an address. truST sizes the binding from the variable's type, not from the
/// address written in the source: `a AT %ID8 : BOOL` is a bit at byte 8 (measured on `6405915`, 2026-11-09).
struct Declared {
    address: IoAddress,
    name: String,
    type_name: String,
}

/// A direct address parsed once, with its text for messages: parsing one at every read and write of every millisecond
/// cost 5 µs of a line's 50 (spec 2026-11-10-solver-substeps-io-links-design.md, measured).
#[derive(Debug, Clone, PartialEq)]
pub struct Port {
    text: String,
    at: IoAddress,
}

impl Port {
    pub fn parse(address: &str) -> Result<Self, String> {
        let at = IoAddress::parse(address).map_err(|e| format!("{address}: {e}"))?;
        Ok(Self {
            text: address.to_string(),
            at,
        })
    }
}

pub struct Controller {
    timing: ScanTiming,
    harness: TestHarness,
    scan_ns: i64,
    declared: Vec<Declared>,
    /// Panel bits pressed since the last scan; each holds for the next scan only.
    pressed: Vec<String>,
}

fn st_type(ty: SignalType) -> &'static str {
    match ty {
        SignalType::BOOL => "BOOL",
        SignalType::DINT => "DINT",
        SignalType::REAL => "REAL",
    }
}

/// A compile error of the program, at its files and lines. `TestHarness` compiles unlabelled sources, so the failed
/// program is compiled again as truST's `CompileSession` with each file's path and labelled errors (truST
/// `host/harness/build.rs`, `source_label`), whose spans are in bytes of that file. A build that succeeds pays nothing.
fn located(project: &Project, error: &impl ToString) -> String {
    let files: Vec<SourceFile> = (project.program.iter())
        .map(|(path, text)| SourceFile::with_path(path.clone(), text.clone()))
        .collect();
    match CompileSession::from_sources(files)
        .label_errors(true)
        .build_runtime()
    {
        Err(labelled) => locate_in_files(&labelled.to_string(), &project.program),
        Ok(_) => error.to_string(),
    }
}

impl Controller {
    /// Compiles the project's program and checks it against the scene. The variable the program declares at a scene
    /// signal or a panel button must have that signal's type (BOOL for a button), or Build fails. An address the
    /// program does not declare is a warning, because a real line may leave a signal unused.
    pub fn build(project: &Project) -> Result<(Self, Vec<String>), String> {
        let sources: Vec<&str> = project
            .program
            .iter()
            .map(|(_, text)| text.as_str())
            .collect();
        let harness = TestHarness::from_sources(&sources).map_err(|e| located(project, &e))?;
        let scan_ns = scan_period(&harness)?;
        let runtime = harness.runtime();
        let declared = runtime
            .io()
            .bindings()
            .iter()
            .map(|b| Declared {
                address: b.address.clone(),
                name: b.display_name.as_deref().unwrap_or("?").to_string(),
                type_name: b
                    .value_type
                    .and_then(|id| runtime.registry().type_name(id))
                    .map_or_else(String::new, |t| t.to_string()),
            })
            .collect();
        let controller = Self {
            timing: ScanTiming::default(),
            harness,
            scan_ns,
            declared,
            pressed: Vec::new(),
        };
        let scene = &project.scene;
        let signals = scene.instances.iter().flat_map(|i| {
            let def = &project.definitions[&i.definition];
            i.signals.iter().map(move |(name, address)| {
                let ty = def
                    .signals
                    .iter()
                    .find(|s| &s.name == name)
                    .map(|s| st_type(s.ty));
                (format!("{}.{name}", i.id), address, ty.unwrap_or("BOOL"))
            })
        });
        let buttons = scene
            .controller
            .panel
            .iter()
            .map(|b| (format!("the button '{}'", b.name), &b.address, "BOOL"));
        let (mut warnings, mut errors) = (Vec::new(), Vec::new());
        for (what, address, ty) in signals.chain(buttons) {
            match controller.at(address) {
                None => warnings.push(format!(
                    "{what} at {address}: the program does not declare it"
                )),
                Some(d) if d.type_name != ty => errors.push(format!(
                    "{what} at {address} is {ty}, but the program declares {} : {} there",
                    d.name, d.type_name
                )),
                Some(_) => {}
            }
        }
        if errors.is_empty() {
            Ok((controller, warnings))
        } else {
            Err(errors.join("\n"))
        }
    }

    /// Replaces the program with `project`'s while the line runs (spec 2026-11-12-st-vscode-design.md, decision 1):
    /// checked first as `build` checks it, then swapped by truST's reload, a warm restart that keeps the RETAIN
    /// variables and virtual time and reinitialises the rest (`TestHarness::reload_sources`, measured 2026-11-12). The
    /// reload starts from an empty process image, so the inputs and outputs are copied over: they hold until the new
    /// program's first scan, as a PLC's do through an online change. A pressed button stays pressed for that scan. On
    /// an error nothing changes. Returns Build's warnings for the new program.
    pub fn reload(&mut self, project: &Project) -> Result<Vec<String>, String> {
        let (checked, warnings) = Self::build(project)?;
        let io = self.harness.runtime().io();
        let images = [("I", io.inputs().to_vec()), ("Q", io.outputs().to_vec())];
        let sources: Vec<&str> = project.program.iter().map(|(_, t)| t.as_str()).collect();
        (self.harness.reload_sources(&sources)).map_err(|e| located(project, &e))?;
        self.scan_ns = checked.scan_ns;
        self.timing = ScanTiming::default();
        self.declared = checked.declared;
        for (area, bytes) in images {
            // The new image reaches as far as the new program declares: a byte past it is one it does not declare.
            let io = self.harness.runtime().io();
            let room = if area == "I" {
                io.inputs()
            } else {
                io.outputs()
            }
            .len();
            for (byte, value) in bytes.into_iter().take(room).enumerate() {
                self.write_at(
                    &Port::parse(&format!("%{area}B{byte}"))?,
                    Value::Byte(value),
                )?;
            }
        }
        Ok(warnings)
    }

    /// Writes zero at input `port` inside the image; past it, the image reads zero already (measured 2026-11-09). A bit
    /// is cleared alone: the other bits of its byte may be declared.
    pub fn clear_input(&mut self, port: &Port) -> Result<(), String> {
        let end = port
            .at
            .flat_byte_end()
            .map_err(|e| format!("{}: {e}", port.text))?;
        let room = self.harness.runtime().io().inputs().len();
        match end {
            Some(end) if end as usize <= room => {
                if matches!(port.at.size, IoSize::Bit) {
                    return self.write_at(port, Value::Bool(false));
                }
                (port.at.byte..end).try_for_each(|b| self.write(&format!("%IB{b}"), Value::Byte(0)))
            }
            _ => Ok(()),
        }
    }

    /// The variable the program declares where `address` starts: the same area and byte, and the same bit for a bit
    /// address.
    fn at(&self, address: &str) -> Option<&Declared> {
        let a = IoAddress::parse(address).ok()?;
        self.declared.iter().find(|d| {
            d.address.area == a.area
                && d.address.byte == a.byte
                && (a.size != IoSize::Bit || d.address.bit == a.bit)
        })
    }

    /// Whether the program declares a variable at `address`.
    pub fn declares(&self, address: &str) -> bool {
        self.at(address).is_some()
    }

    /// The program's scan period (ROADMAP 3.4).
    pub fn scan_ns(&self) -> i64 {
        self.scan_ns
    }

    pub fn timing(&self) -> ScanTiming {
        ScanTiming {
            interval_ms: self.scan_ns as f64 / 1_000_000.0,
            ..self.timing.clone()
        }
    }

    /// `address` parsed, when the program declares a variable there: what the line writes every millisecond, resolved
    /// once at Build (spec 2026-11-10-solver-substeps-io-links-design.md, decision 2).
    pub fn declared_at(&self, address: &str) -> Option<Port> {
        self.at(address)?;
        Port::parse(address).ok()
    }

    /// Writes the input image at `address`, read at the next scan.
    pub fn write(&mut self, address: &str, value: Value) -> Result<(), String> {
        self.write_at(&Port::parse(address)?, value)
    }

    pub fn write_at(&mut self, port: &Port, value: Value) -> Result<(), String> {
        self.harness
            .runtime_mut()
            .write_io(&port.at, value)
            .map_err(|e| format!("{}: {e}", port.text))
    }

    /// Reads the image at `address`: an output the last scan wrote, or an input as last written. truST reads an
    /// address outside the image as zero (measured 2026-11-09).
    pub fn read(&self, address: &str) -> Result<Value, String> {
        self.read_at(&Port::parse(address)?)
    }

    pub fn read_at(&self, port: &Port) -> Result<Value, String> {
        self.harness
            .runtime()
            .io()
            .read(&port.at)
            .map_err(|e| format!("{}: {e}", port.text))
    }

    /// A panel button: its bit is true for the next scan only. A bit the program does not declare is ignored; Build
    /// warned about it.
    pub fn press(&mut self, address: &str) {
        if self.declares(address) {
            self.pressed.push(address.to_string());
        }
    }

    /// One scan: the pressed bits, then virtual time advanced by the scan period, then one cycle.
    pub fn scan(&mut self) -> Result<(), String> {
        let pressed = std::mem::take(&mut self.pressed);
        for address in &pressed {
            self.write(address, Value::Bool(true))?;
        }
        self.harness
            .advance_time(Duration::from_nanos(self.scan_ns));
        let started = std::time::Instant::now();
        let cycle = self.harness.cycle();
        self.timing.record(started.elapsed());
        for address in &pressed {
            self.write(address, Value::Bool(false))?;
        }
        cycle.errors.first().map_or(Ok(()), |e| Err(e.to_string()))
    }

    /// A variable of the program by name, for the UI (M3's `jam`).
    pub fn variable(&self, name: &str) -> Option<Value> {
        self.harness.get_output(name)
    }
}

/// The period at which the program is scanned (ROADMAP 3.4): the greatest common divisor of the intervals of its
/// periodic `TASK`s, so that every task falls on a scan, or `DEFAULT_SCAN_NS` when it declares none. At each scan
/// truST runs only the tasks whose interval has elapsed (measured on the pinned truST, 2026-11-08).
pub(crate) fn scan_period(harness: &TestHarness) -> Result<i64, String> {
    let tasks = harness.runtime().tasks();
    if let Some(task) = tasks.iter().find(|t| t.single.is_some()) {
        return Err(format!(
            "TASK {} is started by SINGLE; only periodic tasks (INTERVAL) are supported",
            task.name
        ));
    }
    let gcd = |a: i64, b: i64| {
        let (mut a, mut b) = (a, b);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        a
    };
    Ok(tasks
        .iter()
        .map(|t| t.interval.as_nanos())
        .filter(|ns| *ns > 0)
        .reduce(gcd)
        .unwrap_or(DEFAULT_SCAN_NS))
}

#[cfg(test)]
#[path = "controller_tests.rs"]
mod tests;
