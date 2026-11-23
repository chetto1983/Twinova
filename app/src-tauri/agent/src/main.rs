//! `twinova-agent`: MCP on stdio, attached to the open app when there is one and headless otherwise; `--headless` never
//! attaches, `--connect <port>` attaches to the app on that port; `--http <address>` serves the headless session on
//! streamable HTTP at `/mcp` (specs 2026-11-11-agent-server-design.md, decision 2, and 2026-11-23-app-mcp-design.md,
//! decision 4).

use std::process::ExitCode;

/// The loopback address and port HTTP takes when `--http` gives none.
const DEFAULT_HTTP: &str = "127.0.0.1:7650";

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [] => attached_or_headless().await,
        [flag] if flag == "--headless" => twinova_agent::serve_stdio().await,
        [flag, port] if flag == "--connect" => connect(port).await,
        [flag] if flag == "--http" => serve_http(DEFAULT_HTTP).await,
        [flag, address] if flag == "--http" => serve_http(address).await,
        _ => Err(format!(
            "usage: twinova-agent [--headless | --connect <port> | --http [address, default {DEFAULT_HTTP}]]"
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

/// The open app found through its file, or the headless session when there is none.
async fn attached_or_headless() -> Result<(), String> {
    let found = match twinova_control::file::app_data_dir() {
        Some(data) => twinova_control::Client::discover(&data).await,
        None => Err("no data folder on this system".into()),
    };
    match found {
        Ok(client) => {
            eprintln!("twinova-agent: attached to the open Twinova app");
            twinova_agent::attached::serve_stdio(client).await
        }
        Err(why) => {
            eprintln!("twinova-agent: headless ({why})");
            twinova_agent::serve_stdio().await
        }
    }
}

/// The app on `port`, with the token its file holds.
async fn connect(port: &str) -> Result<(), String> {
    let port: u16 = port.parse().map_err(|_| format!("{port} is not a port"))?;
    let data = twinova_control::file::app_data_dir().ok_or("no data folder on this system")?;
    let file = twinova_control::ControlFile::read(&data)
        .map_err(|e| format!("no token for port {port}: {e}"))?;
    let client = twinova_control::Client::connect(port, &file.token).await?;
    twinova_agent::attached::serve_stdio(client).await
}
