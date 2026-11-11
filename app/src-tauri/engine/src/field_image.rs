//! Where each signal of the field's devices sits in the field's own I/O image (spec
//! 2026-11-10-curves-belt-options-design.md, amended in Task 4): a bound signal at the mirror of its controller
//! address, an unbound optional one at a private address past the last byte any bound signal uses in that area. A
//! block's `AT %I*` cannot be configured in `%M` ("VAR_CONFIG address area mismatch", truST `6405915`, measured), and
//! the line copies only bound addresses to and from the controller, so a private address is the field's alone.

use trust_runtime::value::Value;
use twinova_project::address::{Address, Area, Size};
use twinova_project::model::{Definition, Direction, Instance, ParamValue, SignalDef, SignalType};

use crate::field::{block_signals, mirror};

/// A device's signal and the field address it is configured at.
pub struct Placed<'a> {
    pub instance: &'a Instance,
    pub signal: &'a SignalDef,
    pub address: Address,
    /// The value an unbound optional signal starts with: its fallback parameter's, written once at Build.
    pub fallback: Option<Value>,
}

/// The area a block reads a signal from: what the controller outputs is the field's input.
fn area(direction: Direction) -> Area {
    match direction {
        Direction::PlcOutput => Area::Input,
        Direction::PlcInput => Area::Output,
    }
}

/// The value of the parameter `name` of `inst`, as its signal type travels in the image: a REAL as its bits.
fn fallback_value(inst: &Instance, def: &Definition, signal: &SignalDef) -> Option<Value> {
    let name = signal.fallback.as_ref()?;
    Some(match (signal.ty, inst.parameter(def, name)?) {
        (SignalType::BOOL, ParamValue::Bool(b)) => Value::Bool(*b),
        (SignalType::REAL, ParamValue::Real(x)) => Value::DWord((*x as f32).to_bits()),
        (SignalType::REAL, ParamValue::Int(n)) => Value::DWord((*n as f32).to_bits()),
        (_, ParamValue::Int(n)) => Value::DWord(*n as i32 as u32),
        (_, other) => unreachable!("the format checks a fallback's type: {other:?}"),
    })
}

/// Every signal of `devices` their blocks declare, as the field configures it: the bound ones, then the unbound
/// optional ones at private addresses. A required signal is always bound: the format refuses one that is not.
pub fn place<'a>(devices: &[(&'a Instance, &'a Definition)]) -> Result<Vec<Placed<'a>>, String> {
    let mut out = Vec::new();
    // The first byte past every bound signal, in the field's input and output areas.
    let mut free = [0u32; 2];
    let slot = |area: Area| usize::from(area == Area::Output);
    for (inst, def) in devices {
        for signal in block_signals(def) {
            if let Some(text) = inst.signals.get(&signal.name) {
                let address = Address::parse(&mirror(text)?)?;
                let end = address.span().1.div_ceil(8) as u32;
                free[slot(address.area)] = free[slot(address.area)].max(end);
                out.push(Placed {
                    instance: inst,
                    signal,
                    address,
                    fallback: None,
                });
            }
        }
    }
    for (inst, def) in devices {
        for signal in
            block_signals(def).filter(|s| s.optional && !inst.signals.contains_key(&s.name))
        {
            let area = area(signal.direction);
            let size = if signal.ty == SignalType::BOOL {
                Size::Bit
            } else {
                Size::Double
            };
            let byte = &mut free[slot(area)];
            let address = Address {
                area,
                size,
                byte: *byte,
                bit: 0,
            };
            *byte += if size == Size::Bit { 1 } else { 4 };
            let fallback = fallback_value(inst, def, signal);
            out.push(Placed {
                instance: inst,
                signal,
                address,
                fallback,
            });
        }
    }
    Ok(out)
}

#[cfg(test)]
#[path = "field_image_tests.rs"]
mod tests;
