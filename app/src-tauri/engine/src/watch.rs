//! Reading a runtime for online monitoring, and turning a user's text into a value (spec
//! 2026-11-13-online-monitoring-design.md, decisions 1 and 6). Both runtimes are truST harnesses: the line's PLC and the
//! field of the blocks. Reads need no debug control (measured 2026-11-13), so monitoring alone costs nothing per scan.

use trust_runtime::harness::{TestHarness, parse_debug_expression};
use trust_runtime::program_model::Expr;
use trust_runtime::value::{Value, format_user_value, normalize_assignment_for_target};

/// The runtime a watch reads, or a debugger serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scope {
    /// The line's PLC, the project's program.
    Plc,
    /// The field, the components' blocks.
    Field,
}

/// What the editor shows after a reference or a condition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Watch {
    /// A variable or an FB member, qualified by its program or its block's instance (`Main.carry.Q`, `F.belt_a.run`).
    Path(Scope, String),
    /// A condition's text, its references qualified as paths are.
    Condition(Scope, String),
}

/// A watch made ready for the run loop: a path kept, a condition parsed once.
pub(crate) enum Compiled {
    Path(Scope, String),
    Expr(Scope, Expr),
    /// A condition that does not parse: it shows nothing.
    Nothing,
}

/// `text` as an expression of `harness`'s program, with no side effects (`parse_debug_expression`).
pub(crate) fn parse(harness: &TestHarness, text: &str) -> Result<Expr, String> {
    let runtime = harness.runtime();
    let mut registry = runtime.registry().clone();
    parse_debug_expression(text, &mut registry, runtime.profile(), &[]).map_err(|e| e.to_string())
}

pub(crate) fn compile(harness: &TestHarness, watch: &Watch) -> Compiled {
    match watch {
        Watch::Path(scope, path) => Compiled::Path(*scope, path.clone()),
        Watch::Condition(scope, text) => {
            parse(harness, text).map_or(Compiled::Nothing, |expr| Compiled::Expr(*scope, expr))
        }
    }
}

/// A scalar value: an instance, a structure or an array has no single text to show.
pub(crate) fn scalar(value: Value) -> Option<Value> {
    match value {
        Value::Instance(_) | Value::Struct(_) | Value::Array(_) | Value::Null => None,
        other => Some(other),
    }
}

/// What `compiled` reads now in `harness`, as the editor shows it; `None` when it does not resolve.
pub(crate) fn read(harness: &mut TestHarness, compiled: &Compiled) -> Option<String> {
    let value = match compiled {
        Compiled::Path(_, path) => harness.get_output(path),
        Compiled::Expr(_, expr) => harness.runtime_mut().evaluate_expression(expr, None).ok(),
        Compiled::Nothing => None,
    };
    value.and_then(scalar).map(|v| format_user_value(&v))
}

/// The names of the members of the instance at `path`, as the runtime holds them: an FB's inputs, outputs and
/// locals, a standard block's included. Empty when `path` is not an instance.
pub(crate) fn members(harness: &TestHarness, path: &str) -> Vec<String> {
    let Some(Value::Instance(id)) = harness.get_output(path) else {
        return Vec::new();
    };
    let instance = harness.runtime().storage().get_instance(id);
    instance.map_or_else(Vec::new, |i| {
        i.variables.keys().map(ToString::to_string).collect()
    })
}

/// The ST type a typed literal names for `value`'s kind (IEC 61131-3 §6.3.3, `DINT#5`).
fn type_name(value: &Value) -> Option<&'static str> {
    Some(match value {
        Value::Bool(_) => "BOOL",
        Value::SInt(_) => "SINT",
        Value::Int(_) => "INT",
        Value::DInt(_) => "DINT",
        Value::LInt(_) => "LINT",
        Value::USInt(_) => "USINT",
        Value::UInt(_) => "UINT",
        Value::UDInt(_) => "UDINT",
        Value::ULInt(_) => "ULINT",
        Value::Real(_) => "REAL",
        Value::LReal(_) => "LREAL",
        Value::Byte(_) => "BYTE",
        Value::Word(_) => "WORD",
        Value::DWord(_) => "DWORD",
        Value::LWord(_) => "LWORD",
        _ => return None,
    })
}

fn same_kind(a: &Value, b: &Value) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

/// The value `text` gives a variable whose value is `current` now: the text is an ST literal or constant expression
/// (`FALSE`, `5`, `T#2S`, `16#FF`), converted as an assignment converts it, or read as a typed literal of the
/// variable's type when that alone fits (`5` for a REAL). A BOOL takes TRUE or FALSE: truST refuses `BOOL#1` (measured
/// 2026-11-13).
pub(crate) fn value_for(
    harness: &mut TestHarness,
    text: &str,
    current: &Value,
) -> Result<Value, String> {
    value_from(
        |text| {
            let expr = parse(harness, text).ok()?;
            harness.runtime_mut().evaluate_expression(&expr, None).ok()
        },
        text,
        current,
    )
}

/// `value_for`, with `evaluate` reading an expression wherever the value lives: the harness, or a paused snapshot
/// (debugger.rs).
pub(crate) fn value_from(
    mut evaluate: impl FnMut(&str) -> Option<Value>,
    text: &str,
    current: &Value,
) -> Result<Value, String> {
    let wanted = type_name(current).unwrap_or("a value of its type");
    let mut evaluate = |text: &str| -> Option<Value> {
        let value = normalize_assignment_for_target(current, evaluate(text)?);
        same_kind(&value, current).then_some(value)
    };
    let typed = || type_name(current).map(|ty| format!("{ty}#{}", text.trim()));
    evaluate(text)
        .or_else(|| typed().and_then(|t| evaluate(&t)))
        .ok_or_else(|| format!("'{}' is not {wanted}", text.trim()))
}

#[cfg(test)]
#[path = "watch_tests.rs"]
mod tests;
