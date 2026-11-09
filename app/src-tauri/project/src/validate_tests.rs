use super::*;
use crate::model::ParamType::{BOOL, DINT, REAL, TIME};
use crate::model::ParamValue::{Bool, Int, Real};

#[test]
fn ids_are_lowercase_words_and_definition_ids_are_dotted() {
    for ok in ["a", "belt_a", "x9"] {
        assert!(is_id(ok), "{ok}");
    }
    for bad in ["", "A", "9a", "_a", "belt-a", "belt a", "bélt"] {
        assert!(!is_id(bad), "{bad}");
    }
    assert!(is_definition_id("twinova.belt"));
    assert!(
        !is_definition_id("twinova..belt")
            && !is_definition_id("Twinova.belt")
            && !is_definition_id("")
    );
}

#[test]
fn paths_stay_inside_the_project_on_every_platform() {
    for ok in ["scene.json", "plc/Main.st", "assets/ab.glb"] {
        assert_eq!(path_problem(ok), None, "{ok}");
    }
    for bad in [
        "",
        "/etc/passwd",
        "C:/x",
        "plc\\Main.st",
        "../x",
        "plc/../x",
        "./x",
        "plc//x",
        "plc/",
    ] {
        assert!(path_problem(bad).is_some(), "{bad}");
    }
}

#[test]
fn paths_that_differ_only_in_case_collide() {
    let paths: Vec<String> = ["plc/Main.st", "plc/MAIN.st", "scene.json"]
        .map(String::from)
        .into();
    assert_eq!(
        case_collisions(paths.iter()),
        [("plc/Main.st".to_string(), "plc/MAIN.st".to_string())]
    );
    assert!(case_collisions(["a", "b"].map(String::from).iter()).is_empty());
}

#[test]
fn poses_must_be_finite_with_a_unit_quaternion() {
    let id = [0.0, 0.0, 0.0, 1.0];
    assert_eq!(pose_problem(&[1.0, 2.0, 3.0], &id), None);
    assert_eq!(pose_problem(&[0.0; 3], &[0.0, 0.0, 0.6, 0.8]), None);
    assert!(pose_problem(&[f64::NAN, 0.0, 0.0], &id).is_some());
    assert!(pose_problem(&[0.0; 3], &[0.0, 0.0, 0.0, f64::INFINITY]).is_some());
    assert!(
        pose_problem(&[0.0; 3], &[0.0, 0.0, 0.0, 1.00001])
            .unwrap()
            .contains("unit quaternion")
    );
    assert!(pose_problem(&[0.0; 3], &[0.0, 0.0, 0.0, 0.99999]).is_some());
    assert_eq!(
        pose_problem(&[0.0; 3], &[0.0, 0.0, 0.0, 1.0000005]),
        None,
        "within 1e-6"
    );
    assert_eq!(
        pose_problem(&[0.0; 3], &[0.0, 0.0, 0.0, 0.9999995]),
        None,
        "within 1e-6"
    );
}

#[test]
fn values_must_match_the_type_and_the_range() {
    assert_eq!(value_problem(BOOL, Bool(true), None, None), None);
    assert_eq!(value_problem(REAL, Real(0.5), Some(0.0), Some(1.0)), None);
    assert_eq!(
        value_problem(REAL, Int(1), Some(0.0), Some(1.0)),
        None,
        "an integer is an exact REAL"
    );
    assert_eq!(
        value_problem(REAL, Real(0.0), Some(0.0), Some(1.0)),
        None,
        "bounds are inclusive"
    );
    assert_eq!(value_problem(REAL, Real(1.0), Some(0.0), Some(1.0)), None);
    assert!(
        value_problem(REAL, Real(1.5), Some(0.0), Some(1.0))
            .unwrap()
            .contains("outside [0, 1]")
    );
    assert!(
        value_problem(REAL, Real(-0.5), Some(0.0), None)
            .unwrap()
            .contains("outside [0, -]")
    );
    assert!(
        value_problem(REAL, Real(f64::NAN), None, None)
            .unwrap()
            .contains("finite")
    );
    assert!(
        value_problem(REAL, Bool(true), None, None)
            .unwrap()
            .contains("not a REAL")
    );
    assert_eq!(
        value_problem(DINT, Int(i64::from(i32::MAX)), None, None),
        None
    );
    assert!(
        value_problem(DINT, Int(i64::from(i32::MAX) + 1), None, None)
            .unwrap()
            .contains("DINT range")
    );
    assert!(value_problem(DINT, Int(i64::from(i32::MIN) - 1), None, None).is_some());
    assert!(
        value_problem(DINT, Real(1.0), None, None)
            .unwrap()
            .contains("not a DINT")
    );
    assert_eq!(value_problem(TIME, Int(0), None, None), None);
    assert!(
        value_problem(TIME, Int(-1), None, None)
            .unwrap()
            .contains("negative")
    );
    assert!(value_problem(BOOL, Int(1), None, None).is_some());
}

#[test]
fn duplicates_are_reported_once_per_repeat() {
    assert_eq!(duplicates(["a", "b", "a", "a"].into_iter()), ["a", "a"]);
    assert!(duplicates(["a", "b"].into_iter()).is_empty());
}

#[test]
fn a_cycle_is_found_from_any_of_its_nodes_only() {
    let parents: BTreeMap<&str, Option<&str>> = [
        ("root", None),
        ("a", Some("b")),
        ("b", Some("a")),
        ("c", Some("a")),
        ("d", Some("root")),
    ]
    .into();
    assert!(in_cycle(&parents, "a") && in_cycle(&parents, "b"));
    assert!(
        !in_cycle(&parents, "c"),
        "c hangs off the cycle without being in it"
    );
    assert!(!in_cycle(&parents, "d") && !in_cycle(&parents, "root"));
}

#[test]
fn a_block_is_found_outside_comments_and_in_any_case() {
    assert!(declares_block(
        "function_block Wanted\nEND_FUNCTION_BLOCK",
        "WANTED"
    ));
    assert!(declares_block(
        "VAR x : BOOL; END_VAR (*c*) FUNCTION_BLOCK Wanted (* tail *)",
        "Wanted"
    ));
    assert!(!declares_block(
        "(* FUNCTION_BLOCK Wanted *) FUNCTION_BLOCK Other",
        "Wanted"
    ));
    assert!(!declares_block("FUNCTION_BLOCK WantedToo", "Wanted"));
    assert!(!declares_block(
        "FUNCTION_BLOCK (* unclosed Wanted",
        "Wanted"
    ));
}
