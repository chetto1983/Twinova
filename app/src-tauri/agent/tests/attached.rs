//! The agent attached to an open app (spec 2026-11-23-app-mcp-design.md, decision 4): a real MCP client drives the
//! agent, which reaches an app through the real control channel. The app here is a stand-in for the window's command
//! registry: it counts the scene's edits, refuses what it does not know, and answers a picture; the window's own
//! commands are tested in the app's end-to-end run (`e2e/mcp.mjs`).

mod common;

use std::sync::{Arc, Mutex};

use common::{ok, refused};
use rmcp::ServiceExt;
use rmcp::model::ClientConfig;
use serde_json::{Value, json};
use twinova_agent::attached::Attached;
use twinova_control::{Client, Handler, bind, serve, token};

/// The stand-in app: its edits, as the undo stack counts them.
fn app(edits: Arc<Mutex<u32>>) -> Handler {
    Arc::new(move |method: String, params: Value| {
        let edits = Arc::clone(&edits);
        Box::pin(async move {
            let mut count = edits.lock().unwrap();
            match method.as_str() {
                "list_commands" => Ok(json!([{ "name": "edit", "description": "one edit" }])),
                "describe_command" => {
                    Ok(json!({ "name": params["name"], "params": { "type": "object" } }))
                }
                "state" => Ok(json!({ "edits": *count })),
                "edit" if params["op"]["op"] == "bad" => Err("unknown instance bad".into()),
                "edit" => {
                    *count += 1;
                    Ok(Value::Null)
                }
                "undo" if *count == 0 => Err("nothing to undo".into()),
                "undo" => {
                    *count -= 1;
                    Ok(Value::Null)
                }
                "screenshot" => Ok(
                    json!({ "png": "iVBORw0KGgo=", "width": 2, "height": 1, "window": params["window"] }),
                ),
                other => Err(format!("no command {other}: list_commands names them")),
            }
        })
    })
}

/// An MCP client of the agent attached to a stand-in app, and that app's edit count.
async fn attached() -> (common::Client, Arc<Mutex<u32>>) {
    let edits = Arc::new(Mutex::new(0));
    let listener = bind(0..=0).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let token = token().unwrap();
    tokio::spawn(serve(listener, token.clone(), app(Arc::clone(&edits))));
    let agent = Attached::new(Client::connect(port, &token).await.unwrap());
    let (server_side, client_side) = tokio::io::duplex(1 << 16);
    tokio::spawn(async move {
        let running = agent.serve(server_side).await.unwrap();
        let _ = running.waiting().await;
    });
    (
        ClientConfig::default().serve(client_side).await.unwrap(),
        edits,
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn the_agent_names_describes_and_runs_the_app_s_commands() {
    let (c, edits) = attached().await;
    let info = c.peer_info().expect("the server's handshake");
    assert!(info.capabilities.tools.is_some(), "it serves tools");
    let instructions = info.instructions.clone().unwrap_or_default();
    assert!(
        instructions.starts_with("Twinova's open app: the window a person sees."),
        "{instructions}"
    );
    assert_eq!(
        ok(&c, "list_commands", json!({})).await,
        json!([{ "name": "edit", "description": "one edit" }])
    );
    assert_eq!(
        ok(&c, "describe_command", json!({ "name": "edit" })).await,
        json!({ "name": "edit", "params": { "type": "object" } })
    );
    ok(
        &c,
        "execute",
        json!({ "command": "edit", "params": { "op": { "op": "place" } } }),
    )
    .await;
    assert_eq!(*edits.lock().unwrap(), 1);
    assert_eq!(ok(&c, "state", json!({})).await, json!({ "edits": 1 }));
    assert_eq!(
        refused(&c, "execute", json!({ "command": "fly" })).await,
        "no command fly: list_commands names them"
    );
    c.cancel().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_batch_that_fails_undoes_its_edits_unless_told_not_to() {
    let (c, edits) = attached().await;
    let place = json!({ "command": "edit", "params": { "op": { "op": "place" } } });
    let bad = json!({ "command": "edit", "params": { "op": { "op": "bad" } } });
    assert_eq!(
        ok(
            &c,
            "batch",
            json!({ "commands": [place, { "command": "state" }] })
        )
        .await,
        json!([null, { "edits": 1 }])
    );
    let failed: Value = serde_json::from_str(
        &refused(
            &c,
            "batch",
            json!({ "commands": [place, place, { "command": "state" }, bad] }),
        )
        .await,
    )
    .unwrap();
    assert_eq!(
        failed,
        json!({
            "failed": { "index": 3, "command": "edit", "error": "unknown instance bad" },
            "answers": [null, null, { "edits": 3 }],
            "undone": 2,
        })
    );
    assert_eq!(
        *edits.lock().unwrap(),
        1,
        "the batch's two edits undone, the first batch's kept"
    );
    let kept: Value = serde_json::from_str(
        &refused(
            &c,
            "batch",
            json!({ "commands": [place, bad], "rollback": false }),
        )
        .await,
    )
    .unwrap();
    assert_eq!(kept["undone"], json!(0));
    assert_eq!(*edits.lock().unwrap(), 2);
    c.cancel().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_screenshot_comes_back_as_a_png_image() {
    let (c, _) = attached().await;
    let result = common::call(&c, "screenshot", json!({})).await;
    assert_ne!(result.is_error, Some(true), "{:?}", result.content);
    let image = result.content[0].as_image().expect("an image");
    assert_eq!(
        (image.data.as_str(), image.mime_type.as_str()),
        ("iVBORw0KGgo=", "image/png")
    );
    assert_eq!(
        result.content[1].as_text().unwrap().text,
        "main window, 2 × 1"
    );
    let plc = common::call(&c, "screenshot", json!({ "window": "plc" })).await;
    assert_eq!(plc.content[1].as_text().unwrap().text, "plc window, 2 × 1");
    c.cancel().await.unwrap();
}

/// The built binary with no argument finds the open app through its file and serves it on stdio. The file is in a
/// data folder of the test's own: on Linux `dirs` reads `XDG_DATA_HOME`, so the person's own app is never reached.
#[cfg(target_os = "linux")]
#[tokio::test(flavor = "multi_thread")]
async fn the_binary_finds_the_open_app_through_its_file_and_serves_it_on_stdio() {
    use rmcp::transport::{ConfigureCommandExt, TokioChildProcess};
    let listener = bind(0..=0).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let token = token().unwrap();
    tokio::spawn(serve(listener, token.clone(), app(Arc::new(Mutex::new(0)))));
    let data = tempfile::tempdir().unwrap();
    let file = twinova_control::ControlFile {
        port,
        token,
        pid: std::process::id(),
    };
    file.write(&data.path().join(twinova_control::file::IDENTIFIER))
        .unwrap();
    let agent = tokio::process::Command::new(env!("CARGO_BIN_EXE_twinova-agent")).configure(|c| {
        c.env("XDG_DATA_HOME", data.path());
    });
    let c = ClientConfig::default()
        .serve(TokioChildProcess::new(agent).expect("the agent starts"))
        .await
        .expect("an MCP session on stdio");
    assert_eq!(
        ok(&c, "list_commands", json!({})).await,
        json!([{ "name": "edit", "description": "one edit" }])
    );
    c.cancel().await.unwrap();
}
