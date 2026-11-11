use super::*;
use crate::model::Item;
use crate::storage::open_folder;
use serde_json::json;
use std::path::Path;

fn sort_line() -> Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/valid/sort_line");
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
fn a_list_parameter_takes_an_array_of_its_items() {
    let mut p = sort_line();
    set_parameter(&mut p, "source", "codes", Some(&json!([3, 1]))).unwrap();
    set_parameter(&mut p, "source", "colors", Some(&json!(["#ff0000"]))).unwrap();
    let parameters = &instance(&p, "source").parameters;
    assert_eq!(
        parameters["codes"],
        ParamValue::List(vec![Item::Int(3), Item::Int(1)])
    );
    assert_eq!(
        parameters["colors"],
        ParamValue::List(vec![Item::Text("#ff0000".into())])
    );
    for (name, value, error) in [
        (
            "codes",
            json!(3),
            "source.codes is a DINT_LIST: 3 is not one",
        ),
        (
            "codes",
            json!([1.5]),
            "source.codes is a DINT_LIST: [1.5] is not one",
        ),
        (
            "colors",
            json!({"red": 1}),
            "source.colors is a COLOR_LIST: {\"red\":1} is not one",
        ),
    ] {
        assert_eq!(
            set_parameter(&mut p, "source", name, Some(&value)),
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
    p.scene.groups.push(crate::model::Group {
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

const AT: Vec3 = [0.0, 0.0, 0.0];
const NO_TURN: Quat = [0.0, 0.0, 0.0, 1.0];

/// The sort line with no instance, connection or definition: a scene to build from the library.
fn empty() -> Project {
    let mut p = sort_line();
    p.scene.instances.clear();
    p.scene.connections.clear();
    p.scene.groups.clear();
    p.definitions.clear();
    p.behaviours.clear();
    p
}

#[test]
fn placing_pins_the_family_once_and_numbers_its_instances() {
    let mut p = empty();
    assert_eq!(
        place(&mut p, "twinova.belt@1", [1.0, 0.0, 2.0], NO_TURN),
        Ok("belt_1".into())
    );
    assert_eq!(
        place(&mut p, "twinova.belt@1", AT, NO_TURN),
        Ok("belt_2".into())
    );
    assert_eq!(
        place(&mut p, "twinova.sink@1", AT, NO_TURN),
        Ok("sink_1".into())
    );
    let belt = &p.scene.instances[0];
    assert_eq!(
        (belt.name.as_str(), belt.position),
        ("belt_1", [1.0, 0.0, 2.0])
    );
    assert!(belt.parameters.is_empty() && belt.signals.is_empty());
    let library = crate::library::family("twinova.belt@1").unwrap();
    assert_eq!(p.definitions["twinova.belt@1"], library.definition);
    assert_eq!(p.behaviours["twinova.belt@1"], library.behaviour.unwrap());
    assert!(
        !p.behaviours.contains_key("twinova.sink@1"),
        "a family without a block pins none"
    );
    // A gap left by a removed instance is filled first.
    remove_instance(&mut p, "belt_1").unwrap();
    assert_eq!(
        place(&mut p, "twinova.belt@1", AT, NO_TURN),
        Ok("belt_1".into())
    );
    assert_eq!(
        place(&mut p, "twinova.belt@9", AT, NO_TURN),
        Err("no family twinova.belt@9 in the library".into())
    );
}

#[test]
fn placing_a_pinned_family_keeps_the_projects_copy() {
    let mut p = sort_line();
    p.definitions.get_mut("twinova.belt@1").unwrap().parameters[0].default = ParamValue::Real(9.0);
    let pinned = p.definitions["twinova.belt@1"].clone();
    p.behaviours
        .insert("twinova.belt@1".into(), "(* edited *)".into());
    place(&mut p, "twinova.belt@1", AT, NO_TURN).unwrap();
    assert_eq!(p.definitions["twinova.belt@1"], pinned);
    assert_eq!(p.behaviours["twinova.belt@1"], "(* edited *)");
}

#[test]
fn a_name_is_trimmed_and_cannot_be_empty() {
    let mut p = sort_line();
    set_name(&mut p, "belt_b", "  Belt B, outfeed ").unwrap();
    assert_eq!(instance(&p, "belt_b").name, "Belt B, outfeed");
    assert_eq!(
        set_name(&mut p, "belt_b", " "),
        Err("a name cannot be empty".into())
    );
    assert_eq!(
        set_name(&mut p, "ghost", "x"),
        Err("no instance ghost".into())
    );
}

fn mounts_of<'a>(p: &'a Project, device: &str) -> Vec<&'a Connection> {
    (p.scene.connections.iter())
        .filter(|c| matches!(c, Connection::Mount { device: d, .. } if d.0 == device))
        .collect()
}

#[test]
fn a_device_mounts_on_one_host_at_a_distance_and_unmounts() {
    let mut p = sort_line();
    set_mount(&mut p, "s_push", Some(("belt_c", 0.75))).unwrap();
    assert_eq!(
        mounts_of(&p, "s_push"),
        [&Connection::Mount {
            host: ("belt_c".into(), "rail".into()),
            device: ("s_push".into(), "mount".into()),
            position: [0.75, 0.0, 0.0],
            rotation: NO_TURN,
        }]
    );
    set_mount(&mut p, "s_push", None).unwrap();
    assert!(mounts_of(&p, "s_push").is_empty());
    assert_eq!(
        mounts_of(&p, "s_in").len(),
        1,
        "other devices keep their mounts"
    );
    assert_eq!(
        set_mount(&mut p, "s_push", Some(("sink_b", 1.0))),
        Err("sink_b (twinova.sink@1) has no MountHost port".into())
    );
    assert_eq!(
        set_mount(&mut p, "belt_b", Some(("belt_c", 1.0))),
        Err("belt_b (twinova.belt@1) has no MountDevice port".into())
    );
    assert!(set_mount(&mut p, "s_push", Some(("belt_c", f64::NAN))).is_err());
}

fn flows(p: &Project) -> Vec<(String, String)> {
    (p.scene.connections.iter())
        .filter_map(|c| match c {
            Connection::Flow { from, to } => Some((
                format!("{}.{}", from.0, from.1),
                format!("{}.{}", to.0, to.1),
            )),
            Connection::Mount { .. } => None,
        })
        .collect()
}

#[test]
fn a_flow_port_has_one_peer_set_from_either_end() {
    let mut p = sort_line();
    // belt_c.out fed sink_c.in: from the sink's end, belt_b takes its place, and belt_b's old peer, sink_b, is freed.
    set_flow(&mut p, "sink_c", "in", Some(("belt_b", "out"))).unwrap();
    let all = flows(&p);
    assert!(
        all.contains(&("belt_b.out".into(), "sink_c.in".into())),
        "{all:?}"
    );
    assert!(
        all.contains(&("belt_in.out".into(), "roller_a.in".into())),
        "the other flows stay: {all:?}"
    );
    assert!(
        !all.iter()
            .any(|(from, to)| from == "belt_c.out" || to == "sink_b.in"),
        "{all:?}"
    );
    set_flow(&mut p, "belt_c", "out", Some(("sink_b", "in"))).unwrap();
    assert!(flows(&p).contains(&("belt_c.out".into(), "sink_b.in".into())));
    set_flow(&mut p, "belt_c", "out", None).unwrap();
    assert!(!flows(&p).iter().any(|(from, _)| from == "belt_c.out"));
    for (id, port, peer, error) in [
        (
            "belt_c",
            "out",
            Some(("belt_b", "out")),
            "belt_c.out cannot flow to belt_b.out",
        ),
        ("belt_c", "rail", None, "belt_c.rail is not a flow port"),
        ("belt_c", "side", None, "twinova.belt@1 has no port side"),
        ("belt_c", "out", Some(("ghost", "in")), "no instance ghost"),
    ] {
        assert_eq!(
            set_flow(&mut p, id, port, peer),
            Err(error.into()),
            "{id}.{port}"
        );
    }
}
