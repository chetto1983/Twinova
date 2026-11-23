//! The example lines over MCP (spec 2026-11-23-app-mcp-design.md, decision 5): the headless agent lists the examples
//! it carries, opens one with no folder, runs it, and saves it only to a path it is given.

mod common;

use common::{ok, refused};
use serde_json::json;
use twinova_project::storage::open_folder;

#[tokio::test(flavor = "multi_thread")]
async fn an_example_opens_runs_and_saves_only_to_a_path() {
    let c = common::http().await.0;
    let listed = ok(&c, "list_examples", json!({})).await;
    assert_eq!(listed.as_array().unwrap().len(), 7);
    assert_eq!(listed[0], json!({ "id": "demo_m3", "name": "M3 demo" }));
    assert_eq!(
        refused(&c, "open_example", json!({ "id": "nope" })).await,
        "no example nope: list_examples names them"
    );
    let opened = ok(&c, "open_example", json!({ "id": "ladder_line" })).await;
    assert_eq!(
        opened["name"],
        json!("Two belts and a chain transfer, in Ladder")
    );
    assert_eq!(opened["instances"].as_array().unwrap().len(), 10);
    assert_eq!(
        ok(&c, "step", json!({ "ms": 100 })).await["time_ms"],
        json!(100)
    );
    assert_eq!(
        refused(&c, "save_project", json!({})).await,
        "the project has no folder yet: name a path"
    );
    let saved = tempfile::tempdir().unwrap();
    ok(&c, "save_project", json!({ "path": saved.path() })).await;
    assert_eq!(
        open_folder(saved.path()).unwrap().opened.project,
        twinova_project::examples::example("ladder_line").unwrap()
    );
    c.cancel().await.unwrap();
}
