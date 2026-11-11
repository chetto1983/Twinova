use super::*;
use serde_json::json;
use std::path::Path;
use twinova_project::storage::open_folder;

fn sort_line() -> Project {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../project/tests/fixtures/valid/sort_line");
    open_folder(&dir).unwrap().opened.project
}

fn instance<'a>(p: &'a Project, id: &str) -> &'a Instance {
    p.scene.instances.iter().find(|i| i.id == id).unwrap()
}

#[test]
fn a_parameter_takes_a_value_of_its_type_or_its_default_back() {
    let mut p = sort_line();
    set_parameter(&mut p, "turntable", "rotation_speed", Some(&json!(45))).unwrap();
    set_parameter(&mut p, "stopper", "raise_time", Some(&json!(200_000_000))).unwrap();
    set_parameter(&mut p, "pusher", "out_right", Some(&json!(true))).unwrap();
    assert_eq!(
        instance(&p, "turntable").parameters["rotation_speed"],
        ParamValue::Real(45.0)
    );
    assert_eq!(
        instance(&p, "stopper").parameters["raise_time"],
        ParamValue::Int(200_000_000)
    );
    assert_eq!(
        instance(&p, "pusher").parameters["out_right"],
        ParamValue::Bool(true)
    );
    set_parameter(&mut p, "turntable", "rotation_speed", None).unwrap();
    assert!(
        !instance(&p, "turntable")
            .parameters
            .contains_key("rotation_speed")
    );
    let refusals = [
        ("nobody", "speed", json!(1), "no instance nobody"),
        (
            "belt_b",
            "wings",
            json!(1),
            "twinova.belt@1 has no parameter wings",
        ),
        (
            "belt_b",
            "speed",
            json!("fast"),
            "belt_b.speed is a REAL: \"fast\" is not one",
        ),
        (
            "stopper",
            "raise_time",
            json!(0.5),
            "stopper.raise_time is a TIME: 0.5 is not one",
        ),
        (
            "pusher",
            "out_right",
            json!(1),
            "pusher.out_right is a BOOL: 1 is not one",
        ),
    ];
    for (id, name, value, error) in refusals {
        assert_eq!(
            set_parameter(&mut p, id, name, Some(&value)),
            Err(error.into())
        );
    }
}

#[test]
fn a_signal_binds_and_unbinds_only_if_its_definition_has_it() {
    let mut p = sort_line();
    bind_signal(&mut p, "turntable", "enable", Some("%QX3.0")).unwrap();
    assert_eq!(instance(&p, "turntable").signals["enable"], "%QX3.0");
    bind_signal(&mut p, "turntable", "enable", None).unwrap();
    assert!(!instance(&p, "turntable").signals.contains_key("enable"));
    assert_eq!(
        bind_signal(&mut p, "turntable", "warp", Some("%QX3.0")),
        Err("twinova.turntable@1 has no signal warp".into())
    );
    assert_eq!(
        bind_signal(&mut p, "ghost", "run", None),
        Err("no instance ghost".into())
    );
}

#[test]
fn instances_are_added_moved_and_removed_with_their_connections() {
    let mut p = sort_line();
    let belt = Instance {
        id: "belt_d".into(),
        name: "Belt D".into(),
        definition: "twinova.belt@1".into(),
        position: [4.45, 0.0, -2.45],
        rotation: [0.0, 0.0, 0.0, 1.0],
        parameters: Default::default(),
        signals: Default::default(),
    };
    add_instance(&mut p, belt.clone()).unwrap();
    assert_eq!(
        add_instance(&mut p, belt.clone()),
        Err("an instance belt_d already exists".into())
    );
    let unknown = Instance {
        id: "x".into(),
        definition: "twinova.robot@1".into(),
        ..belt
    };
    assert_eq!(
        add_instance(&mut p, unknown),
        Err("no definition twinova.robot@1".into())
    );
    move_instance(&mut p, "belt_d", [1.0, 2.0, 3.0], None).unwrap();
    assert_eq!(instance(&p, "belt_d").position, [1.0, 2.0, 3.0]);
    move_instance(
        &mut p,
        "belt_d",
        [1.0, 2.0, 3.0],
        Some([0.0, 1.0, 0.0, 0.0]),
    )
    .unwrap();
    assert_eq!(instance(&p, "belt_d").rotation, [0.0, 1.0, 0.0, 0.0]);
    assert_eq!(
        move_instance(&mut p, "ghost", [0.0; 3], None),
        Err("no instance ghost".into())
    );
    let flow = Connection::Flow {
        from: ("belt_c".into(), "out".into()),
        to: ("belt_d".into(), "in".into()),
    };
    connect(&mut p, flow.clone()).unwrap();
    assert_eq!(
        connect(&mut p, flow.clone()),
        Err("that connection already exists".into())
    );
    let dangling = Connection::Mount {
        host: ("ghost".into(), "rail".into()),
        device: ("s_in".into(), "mount".into()),
        position: [0.0; 3],
        rotation: [0.0, 0.0, 0.0, 1.0],
    };
    assert_eq!(
        connect(&mut p, dangling.clone()),
        Err("no instance ghost".into())
    );
    assert_eq!(
        disconnect(&mut p, &dangling),
        Err("no such connection".into())
    );
    disconnect(&mut p, &flow).unwrap();
    connect(&mut p, flow).unwrap();
    let before = p.scene.connections.len();
    remove_instance(&mut p, "belt_c").unwrap();
    // belt C's two flows (into its sink, and the one to belt D) go with it.
    assert_eq!(p.scene.connections.len(), before - 2);
    assert!(p.scene.connections.iter().all(|c| !touches(c, "belt_c")));
    remove_instance(&mut p, "s_turn").unwrap();
    assert!(
        p.scene.connections.iter().all(|c| !touches(c, "s_turn")),
        "its mount too"
    );
    // A host takes the devices' mounts on it with it: roller A carries the stopper and its photocell.
    remove_instance(&mut p, "roller_a").unwrap();
    let mounted = |device: &str| {
        (p.scene.connections.iter())
            .any(|c| matches!(c, Connection::Mount { device: d, .. } if d.0 == device))
    };
    assert!(!mounted("stopper") && !mounted("s_stop") && mounted("pusher"));
    assert_eq!(
        remove_instance(&mut p, "belt_c"),
        Err("no instance belt_c".into())
    );
}

#[test]
fn a_group_forgets_a_removed_instance() {
    let mut p = sort_line();
    p.scene.groups.push(twinova_project::model::Group {
        id: "g".into(),
        name: "G".into(),
        position: [0.0; 3],
        rotation: [0.0, 0.0, 0.0, 1.0],
        children: vec!["belt_in".into(), "s_in".into()],
    });
    remove_instance(&mut p, "belt_in").unwrap();
    assert_eq!(p.scene.groups[0].children, ["s_in"]);
}

#[test]
fn sources_are_the_program_files_and_the_definitions_behaviours() {
    let mut p = sort_line();
    assert!(
        read_source(&p, "plc/Main.st")
            .unwrap()
            .starts_with("(* The 4.4c sort line")
    );
    let pusher = read_source(&p, "components/twinova.pusher@1.st").unwrap();
    assert!(pusher.contains("FUNCTION_BLOCK PusherBehaviour"));
    write_source(&mut p, "plc/Main.st", "PROGRAM Main\nEND_PROGRAM\n").unwrap();
    write_source(&mut p, "components/twinova.pusher@1.st", "(* empty *)").unwrap();
    assert_eq!(
        read_source(&p, "plc/Main.st").unwrap(),
        "PROGRAM Main\nEND_PROGRAM\n"
    );
    assert_eq!(p.behaviours["twinova.pusher@1"], "(* empty *)");
    for file in [
        "plc/Other.st",
        "components/twinova.sink@1.st",
        "components/x",
        "Main.st",
    ] {
        assert_eq!(read_source(&p, file), Err(format!("no source {file}")));
        assert_eq!(
            write_source(&mut p, file, ""),
            Err(format!("no source {file}"))
        );
    }
}
