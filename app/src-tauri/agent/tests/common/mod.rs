//! What the agent's end-to-end tests share: rmcp's client on streamable HTTP, the tool calls and the fixtures. Each
//! test binary uses part of it.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, CallToolResult, ClientConfig};
use rmcp::service::{RoleClient, RunningService};
use rmcp::transport::StreamableHttpClientTransport;
use serde_json::Value;

pub type Client = RunningService<RoleClient, ClientConfig>;

pub fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../project/tests/fixtures/valid")
        .join(name)
}

pub async fn call(client: &Client, tool: &str, arguments: Value) -> CallToolResult {
    let Value::Object(arguments) = arguments else {
        panic!("arguments are an object")
    };
    let params = CallToolRequestParams::new(tool.to_string()).with_arguments(arguments);
    client
        .call_tool(params)
        .await
        .expect("the call reaches the server")
}

/// A tool's structured answer, or the panic of a tool error with its message.
pub async fn ok(client: &Client, tool: &str, arguments: Value) -> Value {
    let result = call(client, tool, arguments).await;
    assert_ne!(result.is_error, Some(true), "{tool}: {:?}", result.content);
    result.structured_content.expect("a structured answer")
}

/// A tool error's message.
pub async fn refused(client: &Client, tool: &str, arguments: Value) -> String {
    let result = call(client, tool, arguments).await;
    assert_eq!(result.is_error, Some(true), "{tool} succeeded");
    result.content[0]
        .as_text()
        .expect("a text message")
        .text
        .clone()
}

/// The HTTP router on a loopback port of its own, and a client of it.
pub async fn http() -> (Client, String) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let uri = format!("http://{}/mcp", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, twinova_agent::http_router()).await });
    (http_client(&uri).await, uri)
}

pub async fn http_client(uri: &str) -> Client {
    let transport = StreamableHttpClientTransport::from_uri(uri.to_string());
    ClientConfig::default()
        .serve(transport)
        .await
        .expect("an MCP session on HTTP")
}
