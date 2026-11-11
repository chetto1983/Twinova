//! List parameters (spec 2026-11-11-sensors-products-design.md, "The families"): a project with a DINT_LIST and a
//! COLOR_LIST, as default and as override, encodes to canonical files and parses back equal, whatever the items.

use proptest::prelude::*;
use std::path::Path;
use twinova_project::model::{Item, ParamType, ParamValue, ParameterDef};
use twinova_project::storage::open_folder;
use twinova_project::{Project, encode_project, parse_project};

fn reference() -> Project {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/valid/two_belts_diverter");
    open_folder(&root).unwrap().opened.project
}

fn list_param(name: &str, ty: ParamType, default: Vec<Item>) -> ParameterDef {
    ParameterDef {
        name: name.into(),
        ty,
        unit: None,
        default: ParamValue::List(default),
        min: None,
        max: None,
    }
}

/// The reference line with the source's definition given both list parameters, and an instance overriding them.
fn with_lists(codes: Vec<i32>, colors: Vec<[u8; 3]>) -> Project {
    let mut project = reference();
    let key = project
        .scene
        .instances
        .iter()
        .find(|i| i.id == "source")
        .unwrap()
        .definition
        .clone();
    let def = project.definitions.get_mut(&key).unwrap();
    def.parameters
        .push(list_param("codes", ParamType::DintList, vec![Item::Int(7)]));
    def.parameters
        .push(list_param("colors", ParamType::ColorList, vec![]));
    let source = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == "source")
        .unwrap();
    let codes = codes.into_iter().map(|c| Item::Int(c.into())).collect();
    let colors = colors
        .iter()
        .map(|[r, g, b]| Item::Text(format!("#{r:02x}{g:02x}{b:02x}")))
        .collect();
    source
        .parameters
        .insert("codes".into(), ParamValue::List(codes));
    source
        .parameters
        .insert("colors".into(), ParamValue::List(colors));
    project
}

proptest! {
    #[test]
    fn list_parameters_round_trip(
        codes in prop::collection::vec(any::<i32>(), 0..6),
        colors in prop::collection::vec(any::<[u8; 3]>(), 0..6),
    ) {
        let project = with_lists(codes, colors);
        let files = encode_project(&project);
        let back = parse_project(&files).expect("a valid project").project;
        prop_assert_eq!(&back, &project);
        prop_assert_eq!(encode_project(&back), files);
    }
}

#[test]
fn a_list_where_a_scalar_belongs_is_refused_with_its_place() {
    let mut project = with_lists(vec![1], vec![[255, 0, 0]]);
    let source = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == "source")
        .unwrap();
    source
        .parameters
        .insert("max_count".into(), ParamValue::List(vec![Item::Int(1)]));
    source.parameters.insert(
        "colors".into(),
        ParamValue::List(vec![Item::Text("#FF0000".into())]),
    );
    let errors = parse_project(&encode_project(&project)).unwrap_err();
    let messages: Vec<String> = errors
        .iter()
        .map(|d| format!("{}: {}", d.path, d.message))
        .collect();
    assert!(
        messages
            .iter()
            .any(|m| m.ends_with("parameters.max_count: is not a DINT value")),
        "{messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|m| m.ends_with("parameters.colors: item 0 is not a lowercase #rrggbb colour")),
        "{messages:?}"
    );
}
