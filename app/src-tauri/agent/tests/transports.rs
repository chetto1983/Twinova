//! The agent server end to end (spec 2026-11-11-agent-server-design.md, acceptance 1, 2 and 4): rmcp's own client on
//! stdio (the built binary as a child process) and on streamable HTTP (the router on a loopback port) runs the sort line
//! until every sink has counted; the fingerprint is the one the engine gives on its own for the same run; errors are
//! tool errors and the server keeps serving; two HTTP sessions keep two lines.

mod common;

use common::{Client, fixture, http, http_client, ok, refused};
use rmcp::ServiceExt;
use rmcp::model::ClientConfig;
use rmcp::transport::{ConfigureCommandExt, TokioChildProcess};
use serde_json::json;
use twinova_project::storage::open_folder;

async fn stdio() -> Client {
    let command =
        tokio::process::Command::new(env!("CARGO_BIN_EXE_twinova-agent")).configure(|_| {});
    let transport = TokioChildProcess::new(command).expect("the agent starts");
    ClientConfig::default()
        .serve(transport)
        .await
        .expect("an MCP session on stdio")
}

/// The engine on its own, run to `time_ms`: its fingerprint.
fn engine_fingerprint(time_ms: i64) -> String {
    let project = open_folder(&fixture("sort_line")).unwrap().opened.project;
    let (_, mut engine, _) = twinova_engine::build(&project).unwrap();
    while engine.time_ms() < time_ms {
        engine.step().unwrap();
    }
    format!("{:#018x}", engine.fingerprint())
}

/// Opens the sort line and runs it until every sink has counted; the virtual time then.
async fn sort_line_to_every_sink(client: &Client) -> i64 {
    let opened = ok(
        client,
        "open_project",
        json!({ "path": fixture("sort_line") }),
    )
    .await;
    assert_eq!(opened["warnings"], json!([]));
    let mut time = 0;
    for sink in ["sink_b", "sink_c", "sink_reject"] {
        let until = ok(
            client,
            "run_until",
            json!({ "sink": sink, "at_least": 1, "timeout_ms": 120_000 }),
        )
        .await;
        assert_eq!(until["held"], json!(true), "{sink}");
        time = until["state"]["time_ms"].as_i64().unwrap();
    }
    let counts = ok(
        client,
        "read",
        json!({ "signals": ["sink_b.count", "sink_c.count", "%ID12"] }),
    )
    .await;
    assert!(
        counts.as_object().unwrap().values().all(|v| v != "0"),
        "{counts}"
    );
    time
}

#[tokio::test(flavor = "multi_thread")]
async fn stdio_runs_the_sort_line_as_the_engine_does() {
    let client = stdio().await;
    let info = client.peer_info().expect("the server introduced itself");
    assert!(info.capabilities.tools.is_some(), "it serves tools");
    let instructions = info.instructions.as_deref().unwrap_or_default();
    assert!(
        instructions.starts_with("Twinova's run engine."),
        "{instructions}"
    );
    let tools = client.list_all_tools().await.unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    for tool in [
        "open_project",
        "step",
        "run_until",
        "snapshot",
        "read",
        "press",
        "block_sink",
        "fingerprint",
    ] {
        assert!(names.contains(&tool), "{tool} in {names:?}");
    }
    let time = sort_line_to_every_sink(&client).await;
    let fingerprint = ok(&client, "fingerprint", json!({})).await;
    assert_eq!(fingerprint["time_ms"], json!(time));
    assert_eq!(fingerprint["fingerprint"], json!(engine_fingerprint(time)));
    client.cancel().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn http_runs_the_sort_line_and_errors_leave_it_serving() {
    let (client, _) = http().await;
    assert_eq!(
        refused(&client, "step", json!({})).await,
        "open a project first"
    );
    let missing = refused(
        &client,
        "open_project",
        json!({ "path": "/nowhere/at/all" }),
    )
    .await;
    assert!(missing.contains("/nowhere/at/all"), "{missing}");
    let time = sort_line_to_every_sink(&client).await;
    assert_eq!(
        refused(&client, "read", json!({ "signals": ["nope"] })).await,
        "no signal named or addressed nope"
    );
    let bad = refused(
        &client,
        "run_until",
        json!({ "time_ms": 10, "sink": "sink_b", "timeout_ms": 10 }),
    )
    .await;
    assert!(bad.starts_with("give exactly one of"), "{bad}");
    let fingerprint = ok(&client, "fingerprint", json!({})).await;
    assert_eq!(fingerprint["fingerprint"], json!(engine_fingerprint(time)));
    client.cancel().await.unwrap();
}

/// Step, snapshot, press, block_sink, save and restore, reset, on one session; another session's line is its own.
#[tokio::test(flavor = "multi_thread")]
async fn two_http_sessions_keep_two_lines_and_the_run_tools_drive_one() {
    let (first, uri) = http().await;
    let second = http_client(&uri).await;
    ok(
        &first,
        "open_project",
        json!({ "path": fixture("sort_line") }),
    )
    .await;
    ok(
        &second,
        "open_project",
        json!({ "path": fixture("two_belts_diverter") }),
    )
    .await;
    let stepped = ok(&first, "step", json!({ "ms": 1000 })).await;
    assert_eq!(stepped["time_ms"], json!(1000));
    assert_eq!(
        ok(&second, "snapshot", json!({})).await["time_ms"],
        json!(0)
    );
    let id = ok(&first, "save_state", json!({})).await;
    ok(
        &first,
        "block_sink",
        json!({ "sink": "sink_b", "blocked": true }),
    )
    .await;
    ok(&first, "press", json!({ "address": "%IX9.0" })).await;
    let blocked = ok(&first, "step", json!({})).await;
    assert_eq!(blocked["time_ms"], json!(1010));
    assert_eq!(blocked["sinks"][0]["blocked"], json!(true));
    let with_cartons = ok(&first, "snapshot", json!({ "cartons": true })).await;
    assert!(!with_cartons["cartons"].as_array().unwrap().is_empty());
    assert!(
        ok(&first, "snapshot", json!({})).await["cartons"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let restored = ok(&first, "restore_state", json!({ "id": id })).await;
    assert_eq!(
        (
            restored["time_ms"].clone(),
            restored["sinks"][0]["blocked"].clone()
        ),
        (json!(1000), json!(false))
    );
    assert_eq!(
        refused(&first, "restore_state", json!({ "id": 9 })).await,
        "no saved state 9"
    );
    assert_eq!(ok(&first, "reset", json!({})).await["time_ms"], json!(0));
    let until = ok(
        &first,
        "run_until",
        json!({ "time_ms": 50, "timeout_ms": 1000 }),
    )
    .await;
    assert_eq!(
        (until["held"].clone(), until["state"]["time_ms"].clone()),
        (json!(true), json!(50))
    );
    let short = ok(
        &first,
        "run_until",
        json!({ "signal": "sink_b.count", "equals": "5", "timeout_ms": 100 }),
    )
    .await;
    assert_eq!(
        (short["held"].clone(), short["state"]["time_ms"].clone()),
        (json!(false), json!(150))
    );
    first.cancel().await.unwrap();
    second.cancel().await.unwrap();
}
