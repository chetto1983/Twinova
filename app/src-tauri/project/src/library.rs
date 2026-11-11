//! Twinova's component library (spec 2026-11-11-editor-basics-design.md, decision 2): one definition and its behaviour
//! block per family, stored in `app/src-tauri/library/components/` under the format's own file names and embedded here.
//! Placing a family pins a copy of it in the project (`edit::place`); the fixtures copy it too.

use crate::model::Definition;

/// Where a family is listed in the catalogue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Group {
    Conveyors,
    Devices,
    Sensors,
    Flow,
}

/// A family of the library: its definition, its behaviour block if it has one, and its catalogue group.
#[derive(Debug, Clone, PartialEq)]
pub struct Family {
    pub definition: Definition,
    pub behaviour: Option<&'static str>,
    pub group: Group,
}

macro_rules! component {
    ($key:literal) => {
        include_str!(concat!("../../library/components/", $key, ".json"))
    };
}

macro_rules! block {
    ($key:literal) => {
        Some(include_str!(concat!(
            "../../library/components/",
            $key,
            ".st"
        )))
    };
}

/// Every family, in catalogue order: (definition JSON, behaviour block, group).
const FAMILIES: [(&str, Option<&str>, Group); 15] = [
    (
        component!("twinova.belt@1"),
        block!("twinova.belt@1"),
        Group::Conveyors,
    ),
    (
        component!("twinova.roller@1"),
        block!("twinova.roller@1"),
        Group::Conveyors,
    ),
    (
        component!("twinova.belt_curve@1"),
        block!("twinova.belt_curve@1"),
        Group::Conveyors,
    ),
    (
        component!("twinova.roller_curve@1"),
        block!("twinova.roller_curve@1"),
        Group::Conveyors,
    ),
    (
        component!("twinova.turntable@1"),
        block!("twinova.turntable@1"),
        Group::Conveyors,
    ),
    (
        component!("twinova.chain_transfer@1"),
        block!("twinova.chain_transfer@1"),
        Group::Devices,
    ),
    (
        component!("twinova.stopper@1"),
        block!("twinova.stopper@1"),
        Group::Devices,
    ),
    (
        component!("twinova.pusher@1"),
        block!("twinova.pusher@1"),
        Group::Devices,
    ),
    (component!("twinova.sensor_diffuse@1"), None, Group::Sensors),
    (
        component!("twinova.sensor_through_beam@1"),
        None,
        Group::Sensors,
    ),
    (component!("twinova.sensor_color@1"), None, Group::Sensors),
    (component!("twinova.sensor_laser@1"), None, Group::Sensors),
    (
        component!("twinova.scanner_barcode@1"),
        None,
        Group::Sensors,
    ),
    (component!("twinova.source@1"), None, Group::Flow),
    (component!("twinova.sink@1"), None, Group::Flow),
];

/// The library's families, in catalogue order. The library's test checks every one parses and validates, so a broken
/// file fails the build's tests, not a user's placing.
pub fn families() -> Vec<Family> {
    FAMILIES
        .iter()
        .map(|(json, behaviour, group)| Family {
            definition: serde_json::from_str(json)
                .expect("a library definition parses (library_tests.rs)"),
            behaviour: *behaviour,
            group: *group,
        })
        .collect()
}

/// The family pinned as `key` (`id@version`), if the library has it.
pub fn family(key: &str) -> Option<Family> {
    families().into_iter().find(|f| f.definition.key() == key)
}

#[cfg(test)]
#[path = "library_tests.rs"]
mod tests;
