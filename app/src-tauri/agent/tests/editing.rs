//! Scene editing over MCP (spec 2026-11-11-agent-server-design.md, acceptance 3): an agent extends the sort line with a
//! belt after belt C, retargets its sink, speeds the turntable up, rewrites the program for the new belt, builds, runs,
//! and saves; the saved folder reopens and runs the same. Broken edits fail at build with the format's or the
//! compiler's message, and the line runs on.

use std::f64::consts::FRAC_1_SQRT_2;
use std::path::Path;

use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, CallToolResult, ClientConfig};
use rmcp::service::{RoleClient, RunningService};
use rmcp::transport::StreamableHttpClientTransport;
use serde_json::{Value, json};
use twinova_project::storage::open_folder;

type Client = RunningService<RoleClient, ClientConfig>;

async fn call(client: &Client, tool: &str, arguments: Value) -> CallToolResult {
    let Value::Object(arguments) = arguments else {
        panic!("arguments are an object")
    };
    let params = CallToolRequestParams::new(tool.to_string()).with_arguments(arguments);
    client
        .call_tool(params)
        .await
        .expect("the call reaches the server")
}

async fn ok(client: &Client, tool: &str, arguments: Value) -> Value {
    let result = call(client, tool, arguments).await;
    assert_ne!(result.is_error, Some(true), "{tool}: {:?}", result.content);
    result.structured_content.expect("a structured answer")
}

async fn refused(client: &Client, tool: &str, arguments: Value) -> String {
    let result = call(client, tool, arguments).await;
    assert_eq!(result.is_error, Some(true), "{tool} succeeded");
    result.content[0]
        .as_text()
        .expect("a text message")
        .text
        .clone()
}

async fn client() -> Client {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let uri = format!("http://{}/mcp", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, twinova_agent::http_router()).await });
    let transport = StreamableHttpClientTransport::from_uri(uri);
    ClientConfig::default()
        .serve(transport)
        .await
        .expect("an MCP session")
}

/// Belt C runs along −z from (4.45, 0, −0.45) for 2 m: belt D continues it for 2 m, and the turned sink moves to its end.
const QUARTER_LEFT: [f64; 4] = [0.0, FRAC_1_SQRT_2, 0.0, FRAC_1_SQRT_2];

fn flow(from: (&str, &str), to: (&str, &str)) -> Value {
    json!({ "connection": { "kind": "flow", "from": [from.0, from.1], "to": [to.0, to.1] } })
}

/// The program, belt D running beside belt C.
fn program_with_belt_d(program: &str) -> String {
    program
        .replace(
            "    belt_c_running AT %IX1.4 : BOOL;\n",
            "    belt_c_running AT %IX1.4 : BOOL;\n    belt_d_run AT %QX0.7 : BOOL;\n    belt_d_running AT %IX1.5 : BOOL;\n",
        )
        .replace("belt_c_run := TRUE;\n", "belt_c_run := TRUE;\nbelt_d_run := TRUE;\n")
}

#[tokio::test(flavor = "multi_thread")]
async fn an_agent_extends_the_sort_line_builds_runs_and_saves_it() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../project/tests/fixtures/valid/sort_line");
    let saved = tempfile::tempdir().unwrap();
    let c = client().await;
    assert_eq!(
        refused(&c, "describe", json!({})).await,
        "open a project first"
    );
    ok(&c, "open_project", json!({ "path": fixture })).await;
    let described = ok(&c, "describe", json!({})).await;
    assert_eq!(described["program"], json!(["plc/Main.st"]));
    let belt = (described["definitions"].as_array().unwrap().iter())
        .find(|d| d["key"] == "twinova.belt@1")
        .unwrap();
    assert_eq!(belt["behaviour"], json!("components/twinova.belt@1.st"));

    let add = json!({
        "id": "belt_d", "definition": "twinova.belt@1", "position": [4.45, 0.0, -2.45], "rotation": QUARTER_LEFT,
        "parameters": { "length": 2.0 }, "signals": { "run": "%QX0.7", "running": "%IX1.5" },
    });
    ok(&c, "add_instance", add).await;
    ok(&c, "disconnect", flow(("belt_c", "out"), ("sink_c", "in"))).await;
    ok(&c, "connect", flow(("belt_c", "out"), ("belt_d", "in"))).await;
    ok(&c, "connect", flow(("belt_d", "out"), ("sink_c", "in"))).await;
    ok(
        &c,
        "move_instance",
        json!({ "id": "sink_c", "position": [4.45, 0.0, -4.45] }),
    )
    .await;
    ok(
        &c,
        "set_parameter",
        json!({ "instance": "turntable", "name": "rotation_speed", "value": 180.0 }),
    )
    .await;
    let program = ok(&c, "read_source", json!({ "file": "plc/Main.st" })).await;
    let program = program_with_belt_d(program.as_str().unwrap());
    ok(
        &c,
        "write_source",
        json!({ "file": "plc/Main.st", "text": program }),
    )
    .await;
    let built = ok(&c, "build", json!({})).await;
    assert_eq!(built["warnings"], json!([]));
    assert!(
        built["instances"]
            .as_array()
            .unwrap()
            .contains(&json!(["belt_d", "twinova.belt@1"]))
    );

    let until = ok(
        &c,
        "run_until",
        json!({ "sink": "sink_c", "at_least": 1, "timeout_ms": 120_000 }),
    )
    .await;
    assert_eq!(until["held"], json!(true));
    let time = until["state"]["time_ms"].as_i64().unwrap();
    let fingerprint = ok(&c, "fingerprint", json!({})).await["fingerprint"].clone();

    ok(&c, "save_project", json!({ "path": saved.path() })).await;
    let reopened = open_folder(saved.path()).unwrap().opened.project;
    let (_, mut engine, warnings) = twinova_engine::build(&reopened).unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    while engine.time_ms() < time {
        engine.step().unwrap();
    }
    assert_eq!(
        json!(format!("{:#018x}", engine.fingerprint())),
        fingerprint,
        "the saved line runs the same"
    );

    // A broken program fails Build with the compiler's message, and the line runs on from where it was. truST gives
    // the controller's error as a byte span of the program, without the file (the field's are located per file).
    ok(
        &c,
        "write_source",
        json!({ "file": "plc/Main.st", "text": "PROGRAM Main\nVAR x : ; END_VAR\nEND_PROGRAM\n" }),
    )
    .await;
    assert_eq!(
        refused(&c, "build", json!({})).await,
        "expected type at 21..22"
    );
    let after = ok(&c, "step", json!({})).await;
    assert!(after["time_ms"].as_i64().unwrap() > time, "the line ran on");
    ok(
        &c,
        "write_source",
        json!({ "file": "plc/Main.st", "text": program }),
    )
    .await;
    // A connection to a port the definition does not have fails Build with the format's message.
    ok(&c, "connect", flow(("belt_d", "nowhere"), ("sink_b", "in"))).await;
    let error = refused(&c, "build", json!({})).await;
    assert!(error.contains("nowhere"), "{error}");
    let error = refused(&c, "save_project", json!({ "path": saved.path() })).await;
    assert!(
        error.contains("nowhere"),
        "an invalid project is not saved: {error}"
    );
    ok(
        &c,
        "disconnect",
        flow(("belt_d", "nowhere"), ("sink_b", "in")),
    )
    .await;
    ok(&c, "build", json!({})).await;
    // Refusals of the editing tools themselves.
    let refusals = [
        (
            "set_parameter",
            json!({ "instance": "belt_d", "name": "length", "value": "long" }),
        ),
        (
            "bind_signal",
            json!({ "instance": "belt_d", "signal": "warp", "address": "%QX9.0" }),
        ),
        ("connect", json!({ "connection": { "kind": "glue" } })),
        ("remove_instance", json!({ "id": "ghost" })),
        ("read_source", json!({ "file": "plc/Nope.st" })),
    ];
    for (tool, arguments) in refusals {
        refused(&c, tool, arguments).await;
    }
    ok(
        &c,
        "bind_signal",
        json!({ "instance": "turntable", "signal": "enable", "address": null }),
    )
    .await;
    ok(&c, "remove_instance", json!({ "id": "s_push" })).await;
    let bad = json!({ "id": "x", "definition": "twinova.belt@1", "position": [0, 0, 0], "parameters": { "speed": true } });
    refused(&c, "add_instance", bad).await;
    let ids = ok(&c, "describe", json!({})).await["instances"].clone();
    assert!(
        ids.as_array()
            .unwrap()
            .iter()
            .all(|i| i["id"] != "x" && i["id"] != "s_push"),
        "{ids}"
    );
    c.cancel().await.unwrap();
}

/// save_project without a path writes back to the folder the project was opened from.
#[tokio::test(flavor = "multi_thread")]
async fn save_project_writes_back_to_its_own_folder() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../project/tests/fixtures/valid/demo_m3");
    let copy = tempfile::tempdir().unwrap();
    for entry in std::fs::read_dir(&fixture).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            std::fs::copy(entry.path(), copy.path().join(entry.file_name())).unwrap();
        }
    }
    for dir in ["components", "plc"] {
        std::fs::create_dir_all(copy.path().join(dir)).unwrap();
        for entry in std::fs::read_dir(fixture.join(dir)).unwrap() {
            let entry = entry.unwrap();
            std::fs::copy(entry.path(), copy.path().join(dir).join(entry.file_name())).unwrap();
        }
    }
    let c = client().await;
    assert_eq!(
        refused(&c, "save_project", json!({})).await,
        "open a project first"
    );
    ok(&c, "open_project", json!({ "path": copy.path() })).await;
    ok(
        &c,
        "set_parameter",
        json!({ "instance": "belt_a", "name": "speed", "value": 0.25 }),
    )
    .await;
    let folder = ok(&c, "save_project", json!({})).await;
    assert_eq!(folder, json!(copy.path()));
    let reopened = open_folder(copy.path()).unwrap().opened.project;
    let belt = reopened
        .scene
        .instances
        .iter()
        .find(|i| i.id == "belt_a")
        .unwrap();
    assert_eq!(
        belt.parameters["speed"],
        twinova_project::model::ParamValue::Real(0.25)
    );
    c.cancel().await.unwrap();
}
