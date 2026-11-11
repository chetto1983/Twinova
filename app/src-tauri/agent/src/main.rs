//! `twinova-agent`: MCP on stdio, or `twinova-agent --http <address>` on streamable HTTP at `/mcp` (spec
//! 2026-11-11-agent-server-design.md, decision 2).

use std::process::ExitCode;

/// The loopback address and port HTTP takes when `--http` gives none.
const DEFAULT_HTTP: &str = "127.0.0.1:7650";

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [] => twinova_agent::serve_stdio().await,
        [flag] if flag == "--http" => serve_http(DEFAULT_HTTP).await,
        [flag, address] if flag == "--http" => serve_http(address).await,
        _ => Err(format!(
            "usage: twinova-agent [--http [address, default {DEFAULT_HTTP}]]"
        )),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("twinova-agent: {e}");
            ExitCode::FAILURE
        }
    }
}

async fn serve_http(address: &str) -> Result<(), String> {
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|e| format!("{address}: {e}"))?;
    let bound = listener.local_addr().map_err(|e| e.to_string())?;
    eprintln!("twinova-agent: MCP on http://{bound}/mcp");
    axum::serve(listener, twinova_agent::http_router())
        .await
        .map_err(|e| e.to_string())
}
