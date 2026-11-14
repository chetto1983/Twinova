//! What a Ladder box may be (plan 2026-11-13-ladder-editor.md, Task 6; the user's choice, 2026-11-14, of truST's own
//! lists over one written for Twinova): the function blocks of the compiled program, the standard library's and the
//! project's, with their pins; the project's functions; and the standard functions the frontend names, their inputs
//! from the runtime's standard library. The program is compiled as Build resolves its boxes (`program_files.rs`).

use serde::Serialize;
use trust_hir::symbols::ParamDirection;
use trust_runtime::Runtime;
use trust_runtime::program_model::Param;
use trust_runtime::stdlib::StdParams;
use trust_runtime::stdlib::fbs::standard_function_blocks;

use crate::program_files::{declarations_compiled, parsed};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CatalogPin {
    pub name: String,
    /// its declared type; empty for a standard function's, which the call decides (ADD of INTs is an INT)
    pub type_name: String,
    pub bool: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CatalogEntry {
    pub name: String,
    /// a function block, whose box is an instance of it; else a function
    pub block: bool,
    /// declared in the project, rather than the standard library
    pub own: bool,
    pub inputs: Vec<CatalogPin>,
    pub outputs: Vec<CatalogPin>,
    /// takes more inputs on the stem of its last (ADD: IN3, IN4…)
    pub variadic: bool,
}

/// Every box `program`'s Ladder files may hold: the blocks first, then the project's functions, then the standard
/// functions named in `standard_functions` that the runtime has, each list in truST's order.
pub fn catalog(
    program: &[(String, String)],
    standard_functions: &[&str],
) -> Result<Vec<CatalogEntry>, String> {
    let ladders = parsed(program)?;
    let runtime = declarations_compiled(program, &ladders)
        .ok_or("the program does not compile, so its blocks are not known")?;
    let standard: Vec<String> = (standard_function_blocks().into_iter())
        .map(|fb| fb.name.to_ascii_uppercase())
        .collect();
    let pin = |p: &Param| typed(&runtime, &p.name, p.type_id);
    let of = |params: &[Param], direction: ParamDirection| -> Vec<CatalogPin> {
        params
            .iter()
            .filter(|p| p.direction == direction)
            .map(pin)
            .collect()
    };
    let blocks = runtime.function_blocks().values().map(|fb| CatalogEntry {
        name: fb.name.to_string(),
        block: true,
        own: !standard.contains(&fb.name.to_ascii_uppercase()),
        inputs: of(&fb.params, ParamDirection::In),
        outputs: of(&fb.params, ParamDirection::Out),
        variadic: false,
    });
    let functions = runtime.functions().values().map(|f| CatalogEntry {
        name: f.name.to_string(),
        block: false,
        own: true,
        inputs: of(&f.params, ParamDirection::In),
        outputs: std::iter::once(typed(&runtime, "OUT", f.return_type))
            .chain(of(&f.params, ParamDirection::Out))
            .collect(),
        variadic: false,
    });
    let library = (standard_functions.iter()).filter_map(|name| {
        let found = runtime.stdlib().get(name)?;
        let untyped = |name: &str| CatalogPin {
            name: name.to_string(),
            type_name: String::new(),
            bool: false,
        };
        let (inputs, variadic): (Vec<CatalogPin>, bool) = match found.params {
            StdParams::Fixed(names) => (names.iter().map(|n| untyped(n)).collect(), false),
            StdParams::Variadic {
                fixed,
                prefix,
                start,
                min,
            } => {
                let numbered = (*start..start + min).map(|i| format!("{prefix}{i}"));
                let names = fixed.iter().map(ToString::to_string).chain(numbered);
                (names.map(|n| untyped(&n)).collect(), true)
            }
        };
        Some(CatalogEntry {
            name: name.to_string(),
            block: false,
            own: false,
            inputs,
            outputs: vec![untyped("OUT")],
            variadic,
        })
    });
    Ok(blocks.chain(functions).chain(library).collect())
}

fn typed(runtime: &Runtime, name: &str, type_id: trust_hir::TypeId) -> CatalogPin {
    let type_name = runtime
        .registry()
        .type_name(type_id)
        .map(|t| t.to_string())
        .unwrap_or_default();
    CatalogPin {
        name: name.to_string(),
        bool: type_name == "BOOL",
        type_name,
    }
}

#[cfg(test)]
#[path = "ladder_catalog_tests.rs"]
mod tests;
