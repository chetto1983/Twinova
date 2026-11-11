//! Twinova for coding agents (spec 2026-11-11-agent-server-design.md): the run engine served over the Model Context
//! Protocol with rmcp, on stdio and on streamable HTTP. A session owns a project and its engine; each HTTP session gets
//! its own.

use std::sync::{Arc, Mutex, MutexGuard};

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::model::{CallToolResult, ServerCapabilities, ServerConfig};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{ServerHandler, ServiceExt, tool_handler};
use serde::Serialize;

mod edit;
pub mod session;
mod tools_edit;
mod tools_run;

use session::Session;

/// The MCP server of one session.
#[derive(Clone)]
pub struct Agent {
    session: Arc<Mutex<Session>>,
    tool_router: ToolRouter<Self>,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            session: Arc::default(),
            tool_router: Self::run_router() + Self::edit_router(),
        }
    }
}

impl Agent {
    fn session(&self) -> Result<MutexGuard<'_, Session>, String> {
        self.session
            .lock()
            .map_err(|_| "a previous tool call panicked".to_string())
    }
}

/// A tool's answer as structured JSON content.
fn structured(value: &impl Serialize) -> Result<CallToolResult, String> {
    let value = serde_json::to_value(value).map_err(|e| e.to_string())?;
    Ok(CallToolResult::structured(value))
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Agent {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "Twinova's run engine. open_project builds a project folder at time zero; step and run_until advance \
             virtual time; snapshot and read show the line; press and block_sink drive it as a user does. describe \
             shows the project; the editing tools change it and take effect at build; save_project writes it.",
        )
    }
}

/// Serves one session on stdin and stdout until the client leaves.
pub async fn serve_stdio() -> Result<(), String> {
    let running = Agent::default()
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|e| e.to_string())?;
    running.waiting().await.map_err(|e| e.to_string())?;
    Ok(())
}

/// The streamable HTTP service at `/mcp`, a session of its own for each client.
pub fn http_router() -> axum::Router {
    let service = StreamableHttpService::new(
        || Ok(Agent::default()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default(),
    );
    axum::Router::new().nest_service("/mcp", service)
}
