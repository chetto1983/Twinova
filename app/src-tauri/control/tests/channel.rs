//! The control channel end to end over the loopback (spec 2026-11-23-app-mcp-design.md, decision 1 and test plan): a
//! client with the launch's token is answered in order; a wrong or missing token is refused and the connection closed;
//! a busy port is skipped; the file is written for its user only and found by the client.

use std::sync::Arc;

use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use twinova_control::{Client, ControlFile, Handler, Reply, bind, serve, token};

/// A handler that answers `echo` with its parameters and refuses anything else, as the app refuses a command it lacks.
fn echo() -> Handler {
    Arc::new(|method: String, params: Value| {
        Box::pin(async move {
            match method.as_str() {
                "echo" => Ok(params),
                other => Err(format!("no command {other}")),
            }
        })
    })
}

/// An app serving on a free port of a range of its own, and its port and token.
async fn app() -> (u16, String) {
    let listener = bind(0..=0).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let token = token().unwrap();
    tokio::spawn(serve(listener, token.clone(), echo()));
    (port, token)
}

#[tokio::test]
async fn a_client_with_the_token_is_answered_in_order() {
    let (port, token) = app().await;
    let mut client = Client::connect(port, &token).await.unwrap();
    assert_eq!(
        client.call("echo", json!({ "a": 1 })).await,
        Ok(json!({ "a": 1 }))
    );
    assert_eq!(client.call("echo", json!([2])).await, Ok(json!([2])));
    assert_eq!(
        client.call("build", json!({})).await,
        Err("no command build".to_string())
    );
    assert_eq!(client.call("echo", Value::Null).await, Ok(Value::Null));
}

#[tokio::test]
async fn a_wrong_or_missing_token_is_refused_and_the_connection_closed() {
    let (port, token) = app().await;
    let other = token.replace(&token[..1], if &token[..1] == "a" { "b" } else { "a" });
    assert_eq!(
        Client::connect(port, &other).await.err(),
        Some("wrong token".to_string())
    );
    // a request in place of the hello: refused, and nothing more is read
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    stream
        .write_all(b"{\"id\":1,\"method\":\"echo\",\"params\":7}\n{\"id\":2,\"method\":\"echo\"}\n")
        .await
        .unwrap();
    let mut lines = BufReader::new(stream).lines();
    let first: Reply = serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap();
    assert_eq!(first, Reply::of(0, Err("wrong token".into())));
    assert_eq!(lines.next_line().await.unwrap(), None);
}

#[tokio::test]
async fn a_line_that_is_no_request_is_answered_and_the_connection_goes_on() {
    let (port, token) = app().await;
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let hello = format!(
        "{{\"token\":\"{token}\"}}\nnot json\n{{\"id\":3,\"method\":\"echo\",\"params\":1}}\n"
    );
    stream.write_all(hello.as_bytes()).await.unwrap();
    let mut lines = BufReader::new(stream).lines();
    let mut next = async || -> Reply {
        serde_json::from_str(&lines.next_line().await.unwrap().unwrap()).unwrap()
    };
    assert_eq!(next().await, Reply::of(0, Ok(json!({ "app": "twinova" }))));
    assert!(next().await.error.unwrap().starts_with("not a request: "));
    assert_eq!(next().await, Reply::of(3, Ok(json!(1))));
}

#[tokio::test]
async fn a_busy_port_is_skipped_and_none_free_is_an_error() {
    let taken = bind(0..=0).await.unwrap();
    let busy = taken.local_addr().unwrap().port();
    let free = bind(0..=0).await.unwrap().local_addr().unwrap().port();
    // the free one is let go, then the range starts at the busy one
    let next = bind(busy..=busy).await;
    assert!(next.is_err(), "the busy port was bound twice");
    let skipped = bind(busy..=free)
        .await
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    assert_ne!(skipped, busy);
    assert!(bind(busy..=busy).await.is_err());
    #[allow(clippy::reversed_empty_ranges)]
    let none = bind(1..=0).await;
    assert_eq!(none.unwrap_err().to_string(), "no port to try");
}

#[tokio::test]
async fn the_file_is_its_user_s_and_the_client_finds_the_app_by_it() {
    let (port, token) = app().await;
    let data = tempfile::tempdir().unwrap();
    let file = ControlFile {
        port,
        token: token.clone(),
        pid: 7,
    };
    let written = file.write(data.path()).unwrap();
    assert_eq!(written, data.path().join("control/current.json"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&written).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }
    assert_eq!(ControlFile::read(data.path()).unwrap(), file);
    let mut client = Client::discover(data.path()).await.unwrap();
    assert_eq!(
        client.call("echo", json!("found")).await,
        Ok(json!("found"))
    );

    // an app that quits removes its own file, never a newer app's
    let newer = ControlFile {
        pid: 8,
        ..file.clone()
    };
    newer.write(data.path()).unwrap();
    file.remove(data.path());
    assert_eq!(ControlFile::read(data.path()).unwrap(), newer);
    newer.remove(data.path());
    let missing = Client::discover(data.path()).await.err().unwrap();
    assert!(missing.starts_with("no open Twinova app (no "), "{missing}");
}

#[tokio::test]
async fn an_app_that_is_gone_is_said_so() {
    let port = bind(0..=0).await.unwrap().local_addr().unwrap().port();
    let gone = Client::connect(port, "x").await.err().unwrap();
    assert!(
        gone.starts_with(&format!("no Twinova app on port {port}: ")),
        "{gone}"
    );
}

#[test]
fn tokens_are_256_bits_and_differ() {
    let (a, b) = (token().unwrap(), token().unwrap());
    assert_eq!(a.len(), 64);
    assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    assert_ne!(a, b);
}
