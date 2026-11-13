//! The user's writes and forces on the line's PLC (spec 2026-11-13-online-monitoring-design.md, decisions 2, 3 and 6),
//! through truST's debug control. It is attached at the first write or force and stays until Build builds a new PLC:
//! it cannot be detached, and while attached the VM copies its storage before every statement, 3 to 6 times a scan's
//! cost (measured 2026-11-13).

use std::collections::BTreeMap;

use trust_runtime::debug::DebugControl;
use trust_runtime::memory::InstanceId;
use trust_runtime::value::{Value, format_user_value};

use super::{Controller, Port};
use crate::watch::value_for;

/// What the user writes or forces.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Target {
    /// A variable of a program, its own: truST discards a force or a write on an instance's member (`carry.PT`) without
    /// a word (measured 2026-11-13), so Twinova refuses one.
    Variable { program: String, path: String },
    /// An address of the process image, `%IX0.6` or `%QX0.1`.
    Address { address: String },
}

impl Target {
    pub fn variable(program: &str, path: &str) -> Self {
        Self::Variable {
            program: program.to_string(),
            path: path.to_string(),
        }
    }

    pub fn address(address: &str) -> Self {
        Self::Address {
            address: address.to_string(),
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Variable { program, path } => write!(f, "{program}.{path}"),
            Self::Address { address } => f.write_str(address),
        }
    }
}

/// The forces in effect, as the Watch pane lists them.
pub(super) type Forces = BTreeMap<Target, Value>;

impl Controller {
    /// The debug control, attached now if it was not.
    fn debug(&mut self) -> DebugControl {
        let runtime = self.harness.runtime_mut();
        runtime
            .debug_control()
            .unwrap_or_else(|| runtime.enable_debug())
    }

    /// Whether the debug control is attached: from the first write or force to the next Build.
    pub fn debugged(&self) -> bool {
        self.harness.runtime().debug_control().is_some()
    }

    fn instance(&self, program: &str) -> Result<InstanceId, String> {
        match self.harness.get_output(program) {
            Some(Value::Instance(id)) => Ok(id),
            _ => Err(format!("{program} is not a program of the PLC")),
        }
    }

    /// The value `target` holds now, which sets the type a written text must have.
    fn current(&self, target: &Target) -> Result<Value, String> {
        match target {
            Target::Variable { path, .. } if path.contains('.') => Err(format!(
                "{target} is a member of an instance: only a program's own variables can be written or forced"
            )),
            Target::Variable { program, path } => {
                let found = self.harness.get_output(&format!("{program}.{path}"));
                match found {
                    Some(Value::Instance(_) | Value::Struct(_) | Value::Array(_)) => {
                        Err(format!("{target} has no single value to write"))
                    }
                    Some(value) => Ok(value),
                    None => Err(format!("{target} is not a variable of the PLC")),
                }
            }
            Target::Address { address } => self.read(address),
        }
    }

    /// The value `text` gives `target`, typed as the target is.
    pub fn value_for(&mut self, target: &Target, text: &str) -> Result<Value, String> {
        let current = self.current(target)?;
        value_for(&mut self.harness, text, &current)
    }

    /// Holds `target` at `value`, at every scan, until released. An address is held in the process image: an input
    /// against the plant, an output against the program.
    pub fn force(&mut self, target: &Target, value: Value) -> Result<(), String> {
        let refused = |e| format!("{target}: {e:?}");
        match target {
            Target::Variable { program, path } => {
                let id = self.instance(program)?;
                self.debug()
                    .try_force_instance(id, path.as_str(), value.clone())
                    .map_err(refused)?;
            }
            Target::Address { address } => {
                let port = Port::parse(address)?;
                self.debug()
                    .try_force_io(port.at, value.clone())
                    .map_err(refused)?;
            }
        }
        self.forces.insert(target.clone(), value);
        Ok(())
    }

    /// Gives `target` back to the program, or to the plant.
    pub fn release(&mut self, target: &Target) -> Result<(), String> {
        if self.forces.remove(target).is_none() {
            return Err(format!("{target} is not forced"));
        }
        match target {
            Target::Variable { program, path } => {
                let id = self.instance(program)?;
                self.debug().release_instance(id, path);
            }
            Target::Address { address } => self.debug().release_io(&Port::parse(address)?.at),
        }
        Ok(())
    }

    /// Writes `value` into a program variable once, at the next scan; the program may overwrite it. An address is not
    /// written: the plant writes an input every millisecond and the program an output every scan.
    pub fn write_once(&mut self, target: &Target, value: Value) -> Result<(), String> {
        let Target::Variable { program, path } = target else {
            return Err(format!("{target} is an address: force it instead"));
        };
        let id = self.instance(program)?;
        self.debug()
            .try_enqueue_instance_write(id, path.as_str(), value)
            .map_err(|e| format!("{target}: {e:?}"))
    }

    /// Each force in effect and its value as the editor shows it.
    pub fn forces(&self) -> Vec<(Target, String)> {
        (self.forces.iter())
            .map(|(target, value)| (target.clone(), format_user_value(value)))
            .collect()
    }
}
