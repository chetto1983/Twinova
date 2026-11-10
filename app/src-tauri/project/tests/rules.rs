//! One rule at a time: each case edits one detail of the valid reference line, rehashes it, and expects its
//! diagnostic. The corpus covers the same rules from files on disk; this table reaches every branch.

use serde_json::{Value, json};
use std::path::Path;
use twinova_project::codec::{MANIFEST, sha256};
use twinova_project::{Files, parse_project};

fn reference() -> Files {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/valid/two_belts_diverter");
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(root.join(MANIFEST)).unwrap()).unwrap();
    let listed = manifest["files"].as_object().unwrap().keys().cloned();
    std::iter::once(MANIFEST.to_string())
        .chain(listed)
        .map(|path| {
            let bytes = std::fs::read(root.join(&path)).unwrap();
            (path, bytes)
        })
        .collect()
}

fn edit(files: &mut Files, path: &str, f: impl FnOnce(&mut Value)) {
    let mut value: Value = serde_json::from_slice(&files[path]).unwrap();
    f(&mut value);
    files.insert(path.into(), serde_json::to_vec(&value).unwrap());
}

/// Recomputes the manifest's hashes after an edit, as a correct save would; a manifest that is no longer JSON is
/// left as the case made it.
fn rehash(files: &mut Files) {
    if serde_json::from_slice::<Value>(&files[MANIFEST]).is_err() {
        return;
    }
    let hashes: Vec<(String, String)> = files
        .iter()
        .filter(|(p, _)| *p != MANIFEST)
        .map(|(p, b)| (p.clone(), sha256(b)))
        .collect();
    edit(files, MANIFEST, |m| {
        for (path, hash) in hashes {
            if m["files"].get(&path).is_some() {
                m["files"][&path] = json!(hash);
            }
        }
    });
}

fn expect(case: &str, message: &str, f: impl FnOnce(&mut Files)) {
    let mut files = reference();
    f(&mut files);
    rehash(&mut files);
    let text: Vec<String> = parse_project(&files)
        .expect_err(case)
        .iter()
        .map(ToString::to_string)
        .collect();
    assert!(
        text.iter().any(|d| d.contains(message)),
        "{case}: expected '{message}' in {text:#?}"
    );
}

const BELT: &str = "components/twinova.belt@1.json";

fn instance<'a>(scene: &'a mut Value, id: &str) -> &'a mut Value {
    scene["instances"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|i| i["id"] == id)
        .unwrap()
}

#[test]
fn the_reference_line_is_valid() {
    let mut files = reference();
    rehash(&mut files);
    let opened = parse_project(&files).unwrap();
    assert!(opened.changed_outside.is_empty());
}

#[test]
fn manifest_rules() {
    expect("format", "format: must be 'twinova-project'", |f| {
        edit(f, MANIFEST, |m| m["format"] = json!("other"))
    });
    expect("version 0", "version: must be 1", |f| {
        edit(f, MANIFEST, |m| m["version"] = json!(0))
    });
    expect("lists itself", "must not list twinova.json itself", |f| {
        edit(f, MANIFEST, |m| m["files"]["twinova.json"] = json!("0"))
    });
    expect(
        "missing file",
        "'plc/Extra.st' is listed but missing",
        |f| edit(f, MANIFEST, |m| m["files"]["plc/Extra.st"] = json!("0")),
    );
    expect(
        "scene not listed",
        "scene: 'other.json' is not listed in files",
        |f| edit(f, MANIFEST, |m| m["scene"] = json!("other.json")),
    );
    expect(
        "program not listed",
        "program: 'plc/Other.st' is not listed",
        |f| edit(f, MANIFEST, |m| m["program"] = json!(["plc/Other.st"])),
    );
    expect("absolute path", "'/abs.st' must be relative", |f| {
        edit(f, MANIFEST, |m| m["files"]["/abs.st"] = json!("0"))
    });
    expect("not JSON", "twinova.json: line 1", |f| {
        f.insert(MANIFEST.into(), b"{".to_vec());
    });
}

#[test]
fn file_level_rules() {
    expect("program not UTF-8", "plc/Main.st: is not UTF-8 text", |f| {
        f.insert("plc/Main.st".into(), vec![0xff, 0xfe]);
    });
    expect("scene not JSON", "scene.json: line 1", |f| {
        f.insert("scene.json".into(), b"[".to_vec());
    });
    expect(
        "definition not JSON",
        "components/twinova.belt@1.json: line 1",
        |f| {
            f.insert(BELT.into(), b"nope".to_vec());
        },
    );
    expect(
        "pinned twice",
        "pins a definition another file already pins",
        |f| {
            f.insert("components/copy.json".into(), f[BELT].clone());
            edit(f, MANIFEST, |m| {
                m["components"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!("components/copy.json"));
                m["files"]["components/copy.json"] = json!("0");
            });
        },
    );
    const BELT_ST: &str = "components/twinova.belt@1.st";
    expect(
        "behaviour not listed",
        "'components/twinova.belt@1.st', the behaviour of twinova.belt@1, is not listed",
        |f| {
            edit(f, MANIFEST, |m| {
                m["files"].as_object_mut().unwrap().remove(BELT_ST);
            })
        },
    );
    expect(
        "behaviour without a block",
        "does not declare FUNCTION_BLOCK BeltBehaviour, the behaviour twinova.belt@1 names",
        |f| {
            f.insert(
                BELT_ST.into(),
                b"(* FUNCTION_BLOCK BeltBehaviour *)\nFUNCTION_BLOCK Other\nEND_FUNCTION_BLOCK\n"
                    .to_vec(),
            );
        },
    );
    expect(
        "behaviour not UTF-8",
        "components/twinova.belt@1.st: is not UTF-8 text",
        |f| {
            f.insert(BELT_ST.into(), vec![0xff]);
        },
    );
    expect(
        "behaviour file without a behaviour",
        "is listed, but twinova.belt@1 has no behaviour",
        |f| edit(f, BELT, |d| d["behaviour"] = Value::Null),
    );
    expect("asset named wrongly", "named after the SHA-256", |f| {
        f.insert("assets/wrong.glb".into(), b"glb".to_vec());
        edit(f, MANIFEST, |m| m["files"]["assets/wrong.glb"] = json!("0"));
    });
}

#[test]
fn definition_rules() {
    expect("bad id", "is not a dotted lowercase id", |f| {
        edit(f, BELT, |d| d["id"] = json!("Twinova.Belt"))
    });
    expect(
        "stored elsewhere",
        "must be stored as components/twinova.belt@2.json",
        |f| edit(f, BELT, |d| d["version"] = json!(2)),
    );
    expect(
        "bad name",
        "parameter 'Length': is not a lowercase id",
        |f| edit(f, BELT, |d| d["parameters"][0]["name"] = json!("Length")),
    );
    expect("duplicate signal", "signal 'run': is declared twice", |f| {
        edit(f, BELT, |d| d["signals"][1]["name"] = json!("run"))
    });
    expect("default out of range", "parameters.length.default", |f| {
        edit(f, BELT, |d| d["parameters"][0]["default"] = json!(99.0))
    });
    expect("min above max", "min is above max", |f| {
        edit(f, BELT, |d| d["parameters"][0]["min"] = json!(60.0))
    });
    expect("two roots", "exactly one root", |f| {
        edit(f, BELT, |d| {
            let root = d["nodes"][0].clone();
            d["nodes"].as_array_mut().unwrap().push(root);
        })
    });
    expect("missing parent", "nodes.leaf.parent: names no node", |f| {
        edit(f, BELT, |d| {
            let mut leaf = d["nodes"][0].clone();
            leaf["id"] = json!("leaf");
            leaf["parent"] = json!("nowhere");
            d["nodes"].as_array_mut().unwrap().push(leaf);
        })
    });
    expect(
        "bad node pose",
        "nodes.root: rotation must be a unit quaternion",
        |f| {
            edit(f, BELT, |d| {
                d["nodes"][0]["rotation"] = json!([0.0, 0.0, 0.0, 2.0])
            })
        },
    );
    expect(
        "asset not listed",
        "nodes.root.asset: 'assets/x.glb' is not a listed asset",
        |f| edit(f, BELT, |d| d["nodes"][0]["asset"] = json!("assets/x.glb")),
    );
    let joint = |parent: &str, limits: Value| json!({ "id": "j", "kind": "revolute", "parent": parent, "child": "root", "axis": [0.0, 0.0, 1.0], "limits": limits });
    expect("joint to no node", "joints.j.parent: names no node", |f| {
        edit(f, BELT, |d| {
            d["joints"] = json!([joint("ghost", Value::Null)])
        })
    });
    expect("joint limits reversed", "limits in order", |f| {
        edit(f, BELT, |d| {
            d["joints"] = json!([joint("root", json!([1.0, -1.0]))])
        })
    });
}

#[test]
fn scene_rules() {
    let scene = |f: &mut Files, g: fn(&mut Value)| edit(f, "scene.json", g);
    expect("bad instance id", "'Belt_A': is not a lowercase id", |f| {
        scene(f, |s| instance(s, "belt_a")["id"] = json!("Belt_A"))
    });
    let button = |name: &str, address: &str| json!({ "name": name, "address": address });
    expect("unnamed button", "controller.panel[0]: needs a name", |f| {
        edit(f, "scene.json", |s| {
            s["controller"]["panel"] = json!([button(" ", "%IX20.0")])
        })
    });
    expect(
        "button on an output",
        "a button needs an input bit (%IX), not %QX20.0",
        |f| {
            edit(f, "scene.json", |s| {
                s["controller"]["panel"] = json!([button("Start", "%QX20.0")])
            })
        },
    );
    expect(
        "button on a word",
        "a button needs an input bit (%IX), not %IW20",
        |f| {
            edit(f, "scene.json", |s| {
                s["controller"]["panel"] = json!([button("Start", "%IW20")])
            })
        },
    );
    expect("button address", "controller.panel[0]: ", |f| {
        edit(f, "scene.json", |s| {
            s["controller"]["panel"] = json!([button("Start", "IX9")])
        })
    });
    expect(
        "button over a signal",
        "%IX0.0 overlaps the address of sensor_a.detected",
        |f| {
            edit(f, "scene.json", |s| {
                s["controller"]["panel"] = json!([button("Start", "%IX0.0")])
            })
        },
    );
    expect(
        "two buttons at one bit",
        "%IX20.0 overlaps the address of the button 'Start'",
        |f| {
            edit(f, "scene.json", |s| {
                s["controller"]["panel"] =
                    json!([button("Start", "%IX20.0"), button("Stop", "%IX20.0")])
            })
        },
    );
    expect(
        "two buttons, one name",
        "two buttons are named 'Start'",
        |f| {
            edit(f, "scene.json", |s| {
                s["controller"]["panel"] =
                    json!([button("Start", "%IX20.0"), button("Start", "%IX20.1")])
            })
        },
    );
    expect(
        "bad instance pose",
        "instances.belt_a.pose: rotation must be a unit quaternion",
        |f| {
            scene(f, |s| {
                instance(s, "belt_a")["rotation"] = json!([0.0, 0.0, 0.0, 2.0])
            })
        },
    );
    expect(
        "unknown definition",
        "'twinova.belt@9' is not a pinned definition",
        |f| {
            scene(f, |s| {
                instance(s, "belt_a")["definition"] = json!("twinova.belt@9")
            })
        },
    );
    expect(
        "override of the wrong type",
        "parameters.length: is not a REAL value",
        |f| {
            scene(f, |s| {
                instance(s, "belt_a")["parameters"]["length"] = json!(true)
            })
        },
    );
    expect(
        "unknown signal",
        "signals.horn: twinova.belt@1 has no such signal",
        |f| {
            scene(f, |s| {
                instance(s, "belt_a")["signals"]["horn"] = json!("%QX1.0")
            })
        },
    );
    expect(
        "not an address",
        "'Q0.0' is not an IEC direct address",
        |f| {
            scene(f, |s| {
                instance(s, "belt_a")["signals"]["run"] = json!("Q0.0")
            })
        },
    );
    expect(
        "group child unknown",
        "groups.infeed.children: 'ghost' is not an instance",
        |f| scene(f, |s| s["groups"][0]["children"] = json!(["ghost"])),
    );
    expect(
        "child in two groups",
        "'belt_a' already belongs to a group",
        |f| {
            scene(f, |s| {
                let mut second = s["groups"][0].clone();
                second["id"] = json!("other");
                second["children"] = json!(["belt_a"]);
                s["groups"].as_array_mut().unwrap().push(second);
            })
        },
    );
    expect("bad group pose", "groups.infeed.pose", |f| {
        scene(f, |s| {
            s["groups"][0]["rotation"] = json!([0.0, 0.0, 0.0, 3.0])
        })
    });
    expect(
        "connection to no instance",
        "'ghost' is not an instance",
        |f| scene(f, |s| s["connections"][0]["from"] = json!(["ghost", "out"])),
    );
    expect(
        "connection to no port",
        "'source' has no port 'side'",
        |f| {
            scene(f, |s| {
                s["connections"][0]["from"] = json!(["source", "side"])
            })
        },
    );
    expect(
        "wrong port kind",
        "belt_a.in is a FlowIn port, not FlowOut",
        |f| scene(f, |s| s["connections"][1]["from"] = json!(["belt_a", "in"])),
    );
    expect(
        "bad mount pose",
        "connections[5]: rotation must be a unit quaternion",
        |f| {
            scene(f, |s| {
                s["connections"][5]["rotation"] = json!([0.0, 0.0, 0.0, 0.5])
            })
        },
    );
}

#[test]
fn a_host_carries_several_devices_but_a_device_has_one_host() {
    let mut files = reference();
    edit(&mut files, "scene.json", |s| {
        s["connections"][6]["host"] = json!(["belt_a", "rail"]);
    });
    rehash(&mut files);
    parse_project(&files).expect("two sensors on belt_a's rail");
    expect("device mounted twice", "sensor_a.mount has 2 peers", |f| {
        edit(f, "scene.json", |s| {
            let again = s["connections"][5].clone();
            s["connections"].as_array_mut().unwrap().push(again);
        })
    });
}

#[test]
fn edge_values_that_are_valid() {
    let mut files = reference();
    edit(&mut files, BELT, |d| {
        d["parameters"][1]["min"] = json!(0.6);
        d["parameters"][1]["max"] = json!(0.6);
        d["joints"] = json!([
            { "id": "swing", "kind": "revolute", "parent": "root", "child": "root", "axis": [0.0, 0.0, 1.0], "limits": [-1.0, 1.0] },
            { "id": "pinned", "kind": "prismatic", "parent": "root", "child": "root", "axis": [1.0, 0.0, 0.0], "limits": [0.5, 0.5] },
            { "id": "weld", "kind": "fixed", "parent": "root", "child": "root", "axis": [0.0, 1.0, 0.0], "limits": null }
        ]);
    });
    rehash(&mut files);
    parse_project(&files).expect("min == max, a valid axis, and equal or absent limits");
}

#[test]
fn a_newer_manifest_with_new_fields_says_it_is_newer() {
    expect(
        "newer with a new field",
        "saved by a newer Twinova (format 2)",
        |f| {
            edit(f, MANIFEST, |m| {
                m["version"] = json!(2);
                m["signature"] = json!("a field format 2 added");
            })
        },
    );
    expect(
        "current with an unknown field",
        "unknown field `signature`",
        |f| edit(f, MANIFEST, |m| m["signature"] = json!("x")),
    );
    expect(
        "older with an unknown field",
        "unknown field `signature`",
        |f| {
            edit(f, MANIFEST, |m| {
                m["version"] = json!(0);
                m["signature"] = json!("x");
            })
        },
    );
}

#[test]
fn an_optional_signal_may_stay_unbound_and_a_required_one_may_not() {
    let unbind = |f: &mut Files| {
        edit(f, "scene.json", |s| {
            instance(s, "belt_a")["signals"]
                .as_object_mut()
                .unwrap()
                .remove("running");
        })
    };
    expect(
        "required signal unbound",
        "instances.belt_a.signals.running: twinova.belt@1 needs it bound to an address",
        unbind,
    );
    let mut files = reference();
    unbind(&mut files);
    edit(&mut files, BELT, |d| {
        d["signals"][1]["optional"] = json!(true)
    });
    rehash(&mut files);
    parse_project(&files).expect("an optional signal left unbound");
}
