//! `twinova-agent setup` (spec 2026-11-23-app-mcp-design.md, decision 6): registers this binary with the MCP clients a
//! person has, by their documented routes:
//! - Claude Code through its own command, `claude mcp add --scope user` (https://code.claude.com/docs/en/mcp: user scope
//!   is "available to you across all projects"); `add` refuses a name that exists (measured 2026-11-23: "MCP server
//!   twinova already exists in user config", exit 1), so the entry is removed first and setup can run again;
//! - Claude Desktop through `claude_desktop_config.json` in its folder of the system's configuration folder
//!   (https://modelcontextprotocol.io/docs/develop/connect-local-servers), its other servers kept; it reads the file at
//!   start only.

use std::path::Path;
use std::process::Command;

use serde_json::{Map, Value, json};

/// The name the clients list the agent by.
pub const NAME: &str = "twinova";

/// What setup did for one client, in a sentence for the person.
pub type Outcome = Result<String, String>;

/// Registers `agent` with Claude Code, through the program `claude` names (`claude` itself, but for the tests).
pub fn claude_code(claude: &str, agent: &Path) -> Outcome {
    let agent = agent.display().to_string();
    let _ = Command::new(claude)
        .args(["mcp", "remove", "--scope", "user", NAME])
        .output();
    let added = Command::new(claude)
        .args(["mcp", "add", "--scope", "user", NAME, "--", &agent])
        .output()
        .map_err(|e| {
            format!("Claude Code: `{claude}` did not start ({e}); is Claude Code installed?")
        })?;
    if added.status.success() {
        Ok(format!(
            "Claude Code: {NAME} added for every project, running {agent}"
        ))
    } else {
        Err(format!(
            "Claude Code: `{claude} mcp add` failed: {}",
            String::from_utf8_lossy(&added.stderr).trim()
        ))
    }
}

/// Registers `agent` in Claude Desktop's configuration under `config_dir` (the system's configuration folder), when
/// Claude Desktop has its folder there.
pub fn claude_desktop(config_dir: &Path, agent: &Path) -> Outcome {
    let folder = config_dir.join("Claude");
    if !folder.is_dir() {
        return Err(format!(
            "Claude Desktop: not installed ({} not found)",
            folder.display()
        ));
    }
    let file = folder.join("claude_desktop_config.json");
    let mut config = match std::fs::read_to_string(&file) {
        Ok(text) => serde_json::from_str::<Value>(&text).map_err(|e| {
            format!(
                "Claude Desktop: {} is not JSON ({e}); left as it is",
                file.display()
            )
        })?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => json!({}),
        Err(e) => return Err(format!("Claude Desktop: {}: {e}", file.display())),
    };
    let servers = (config.as_object_mut())
        .ok_or_else(|| {
            format!(
                "Claude Desktop: {} is not a JSON object; left as it is",
                file.display()
            )
        })?
        .entry("mcpServers")
        .or_insert_with(|| Value::Object(Map::new()));
    let servers = (servers.as_object_mut()).ok_or_else(|| {
        format!(
            "Claude Desktop: mcpServers in {} is not an object",
            file.display()
        )
    })?;
    servers.insert(NAME.into(), json!({ "command": agent, "args": [] }));
    let text = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&file, text + "\n")
        .map_err(|e| format!("Claude Desktop: {}: {e}", file.display()))?;
    Ok(format!(
        "Claude Desktop: {NAME} added to {}; quit Claude Desktop and start it again",
        file.display()
    ))
}

#[cfg(test)]
#[path = "setup_tests.rs"]
mod tests;
