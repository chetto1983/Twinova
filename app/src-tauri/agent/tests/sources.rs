//! Sources saved outside Twinova, over MCP (spec 2026-11-12-st-vscode-design.md, decisions 1, 2 and 9): an agent runs
//! the diverter line from a folder, the program is saved there as VS Code saves it, and reload_sources takes it online
//! while the line runs on; a behaviour saved there rebuilds the line from zero; change_program refuses what Build would.

mod common;

use std::fs;

use common::{fixture, ok, refused};
use serde_json::json;
use twinova_project::storage::{open_folder, save_folder};

async fn signal(c: &common::Client, name: &str) -> String {
    let read = ok(c, "read", json!({ "signals": [name] })).await;
    read[name].as_str().unwrap().to_string()
}

#[tokio::test(flavor = "multi_thread")]
async fn sources_saved_in_the_folder_reach_the_running_line() {
    let project = open_folder(&fixture("two_belts_diverter"))
        .unwrap()
        .opened
        .project;
    let dir = tempfile::tempdir().unwrap();
    save_folder(dir.path(), &project).unwrap();
    let main = dir.path().join("plc/Main.st");
    let text = fs::read_to_string(&main).unwrap();
    let c = common::http().await.0;
    assert_eq!(
        refused(&c, "reload_sources", json!({})).await,
        "open a project first"
    );
    ok(&c, "open_project", json!({ "path": dir.path() })).await;
    ok(
        &c,
        "run_until",
        json!({ "time_ms": 3_000, "timeout_ms": 5_000 }),
    )
    .await;
    assert_eq!(signal(&c, "belt_b.run").await, "TRUE");

    let unchanged = ok(&c, "reload_sources", json!({})).await;
    assert_eq!(unchanged["taken"], json!(null));

    fs::write(
        &main,
        text.replace("belt_b_run := TRUE;", "belt_b_run := FALSE;"),
    )
    .unwrap();
    let online = ok(&c, "reload_sources", json!({})).await;
    assert_eq!(online["taken"], "program");
    assert_eq!(online["state"]["time_ms"], 3_000, "the line goes on");
    ok(&c, "step", json!({})).await;
    assert_eq!(signal(&c, "belt_b.run").await, "FALSE");
    assert_eq!(
        ok(&c, "reload_sources", json!({})).await["taken"],
        json!(null)
    );
    assert_eq!(ok(&c, "reset", json!({})).await["time_ms"], 0);
    ok(&c, "step", json!({})).await;
    assert_eq!(
        signal(&c, "belt_b.run").await,
        "FALSE",
        "reset starts from the new program"
    );

    let broken = json!({ "program": [["plc/Main.st", "PROGRAM Main\nx := ;\nEND_PROGRAM\n"]] });
    assert!(
        refused(&c, "change_program", broken)
            .await
            .contains("plc/Main.st")
    );
    let restored = json!({ "program": [["plc/Main.st", text.clone()]] });
    assert_eq!(ok(&c, "change_program", restored).await["taken"], "program");
    ok(&c, "step", json!({})).await;
    assert_eq!(signal(&c, "belt_b.run").await, "TRUE");

    ok(
        &c,
        "run_until",
        json!({ "time_ms": 2_000, "timeout_ms": 5_000 }),
    )
    .await;
    fs::write(&main, &text).unwrap();
    let belt = dir.path().join("components/twinova.belt@1.st");
    let block = fs::read_to_string(&belt).unwrap();
    fs::write(
        &belt,
        block.replace("(* Twinova belt conveyor", "(* Our belt conveyor"),
    )
    .unwrap();
    let rebuilt = ok(&c, "reload_sources", json!({})).await;
    assert_eq!(rebuilt["taken"], "behaviours");
    assert_eq!(
        rebuilt["state"]["time_ms"], 0,
        "a behaviour rebuilds from zero"
    );
}
