//! The agent attached to the open app (spec 2026-11-23-app-mcp-design.md, decision 4, after SolidCraft's design): its
//! tools run the window's command registry through the control channel (crate `twinova-control`), so the person sees
//! every change as if they had clicked. `list_commands` and `describe_command` name the registry's commands and their
//! parameters, `execute` runs one, `batch` runs several and undoes the scene's edits of a batch that fails, `state`
//! reads the window, `screenshot` shows it.

use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, ServerCapabilities, ServerConfig};
use rmcp::{ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use twinova_control::Client;

use crate::structured;

/// The MCP server attached to one open app.
#[derive(Clone)]
pub struct Attached {
    client: Arc<Mutex<Client>>,
    tool_router: ToolRouter<Self>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Describe {
    /// A command's name, as list_commands gives it.
    pub name: String,
}

#[derive(Deserialize, Serialize, JsonSchema, Clone)]
pub struct Execute {
    /// A command's name, as list_commands gives it.
    pub command: String,
    /// Its parameters, as describe_command's schema says; none when absent.
    #[serde(default)]
    pub params: Option<serde_json::Map<String, Value>>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Batch {
    /// The commands, in order; the first that fails stops the batch.
    pub commands: Vec<Execute>,
    /// Undo the scene's edits the batch made when a command fails; true when absent.
    #[serde(default)]
    pub rollback: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub struct Screenshot {
    /// `main` (the default) or `plc`, the PLC pane's own window.
    #[serde(default)]
    pub window: Option<String>,
}

/// What the app's `screenshot` answers: the window as drawn, as a base64 PNG.
#[derive(Deserialize)]
struct Picture {
    png: String,
    width: u32,
    height: u32,
}

impl Attached {
    pub fn new(client: Client) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
            tool_router: Self::attached_router(),
        }
    }

    async fn call(&self, method: &str, params: Value) -> Result<Value, String> {
        self.client.lock().await.call(method, params).await
    }

    async fn run(&self, step: &Execute) -> Result<Value, String> {
        let params = step.params.clone().map_or(json!({}), Value::Object);
        self.call(&step.command, params).await
    }
}

#[tool_router(router = attached_router, vis = "pub(crate)")]
impl Attached {
    #[tool(
        description = "The open app's commands: every action of its window, by name, with a sentence each."
    )]
    async fn list_commands(&self) -> Result<CallToolResult, String> {
        structured(&self.call("list_commands", json!({})).await?)
    }

    #[tool(description = "A command's sentence and its parameters as a JSON Schema.")]
    async fn describe_command(
        &self,
        Parameters(a): Parameters<Describe>,
    ) -> Result<CallToolResult, String> {
        structured(
            &self
                .call("describe_command", json!({ "name": a.name }))
                .await?,
        )
    }

    #[tool(
        description = "Run one of the app's commands with its parameters, as the window's own button does: the person \
                          sees the change. Answers what the command answers; a refused command is an error with the \
                          window's reason."
    )]
    async fn execute(&self, Parameters(a): Parameters<Execute>) -> Result<CallToolResult, String> {
        structured(&self.run(&a).await?)
    }

    #[tool(
        description = "Run commands in order. The first that fails stops the batch; unless rollback is false, the \
                          scene's edits the batch made are undone. Answers each command's answer, or which one failed \
                          and why."
    )]
    async fn batch(&self, Parameters(a): Parameters<Batch>) -> Result<CallToolResult, String> {
        let mut answers = Vec::new();
        for (index, step) in a.commands.iter().enumerate() {
            match self.run(step).await {
                Ok(answer) => answers.push(answer),
                Err(error) => {
                    let edits = a.commands[..index]
                        .iter()
                        .filter(|s| s.command == "edit")
                        .count();
                    let undone = if a.rollback.unwrap_or(true) {
                        self.undo(edits).await
                    } else {
                        0
                    };
                    return Err(json!({
                        "failed": { "index": index, "command": step.command, "error": error },
                        "answers": answers,
                        "undone": undone,
                    })
                    .to_string());
                }
            }
        }
        structured(&answers)
    }

    #[tool(
        description = "The window's state: the project and its instances, the build, the line's snapshot (time, \
                          sinks, devices, signals), the sources and whether each has edits not built, notices, errors."
    )]
    async fn state(&self) -> Result<CallToolResult, String> {
        structured(&self.call("state", json!({})).await?)
    }

    #[tool(
        description = "A picture of the app's window as it is drawn now, as a PNG: `main` or `plc`."
    )]
    async fn screenshot(
        &self,
        Parameters(a): Parameters<Screenshot>,
    ) -> Result<CallToolResult, String> {
        let window = a.window.unwrap_or_else(|| "main".into());
        let answer = self.call("screenshot", json!({ "window": window })).await?;
        let picture: Picture =
            serde_json::from_value(answer).map_err(|e| format!("not a picture: {e}"))?;
        Ok(CallToolResult::success(vec![
            ContentBlock::image(picture.png, "image/png"),
            ContentBlock::text(format!(
                "{window} window, {} × {}",
                picture.width, picture.height
            )),
        ]))
    }
}

impl Attached {
    /// Undoes `edits` scene edits, as many as the app lets; how many it undid.
    async fn undo(&self, edits: usize) -> usize {
        let mut undone = 0;
        while undone < edits && self.call("undo", json!({})).await.is_ok() {
            undone += 1;
        }
        undone
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Attached {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "Twinova's open app: the window a person sees. list_commands names every action of the window and \
             describe_command gives its parameters; execute runs one as the window's button does, batch several. \
             state reads the window; screenshot shows it.",
        )
    }
}

/// Serves the agent attached to the app `client` reaches, on stdin and stdout, until the client leaves.
pub async fn serve_stdio(client: Client) -> Result<(), String> {
    let running = Attached::new(client)
        .serve(rmcp::transport::stdio())
        .await
        .map_err(|e| e.to_string())?;
    running.waiting().await.map_err(|e| e.to_string())?;
    Ok(())
}
