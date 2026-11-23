use super::*;
use std::path::PathBuf;

const AGENT: &str = "/usr/bin/twinova-agent";

fn desktop(config: Option<&str>) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("Claude")).unwrap();
    let file = dir.path().join("Claude/claude_desktop_config.json");
    if let Some(text) = config {
        std::fs::write(&file, text).unwrap();
    }
    (dir, file)
}

fn read(file: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap()
}

#[test]
fn claude_desktop_gets_the_agent_beside_its_other_servers() {
    let (dir, file) = desktop(Some(
        r#"{"mcpServers":{"other":{"command":"x"}},"theme":"dark"}"#,
    ));
    let said = claude_desktop(dir.path(), Path::new(AGENT)).unwrap();
    assert!(
        said.ends_with("quit Claude Desktop and start it again"),
        "{said}"
    );
    assert_eq!(
        read(&file),
        json!({
            "mcpServers": { "other": { "command": "x" }, "twinova": { "command": AGENT, "args": [] } },
            "theme": "dark",
        })
    );
    claude_desktop(dir.path(), Path::new("/opt/new/twinova-agent")).unwrap();
    assert_eq!(
        read(&file)["mcpServers"]["twinova"]["command"],
        "/opt/new/twinova-agent",
        "setup again takes the new path"
    );
}

#[test]
fn claude_desktop_s_file_is_made_when_it_has_none() {
    let (dir, file) = desktop(None);
    claude_desktop(dir.path(), Path::new(AGENT)).unwrap();
    assert_eq!(
        read(&file),
        json!({ "mcpServers": { "twinova": { "command": AGENT, "args": [] } } })
    );
}

#[test]
fn a_file_that_is_not_a_configuration_is_left_as_it_is() {
    for broken in ["{ not json", "[1, 2]", r#"{"mcpServers": 3}"#] {
        let (dir, file) = desktop(Some(broken));
        let error = claude_desktop(dir.path(), Path::new(AGENT)).unwrap_err();
        assert!(error.starts_with("Claude Desktop: "), "{error}");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), broken);
    }
}

#[test]
fn without_claude_desktop_s_folder_nothing_is_written() {
    let dir = tempfile::tempdir().unwrap();
    let error = claude_desktop(dir.path(), Path::new(AGENT)).unwrap_err();
    assert!(
        error.starts_with("Claude Desktop: not installed"),
        "{error}"
    );
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[test]
fn claude_code_that_is_not_installed_is_said_so() {
    let error = claude_code("/no/such/claude", Path::new(AGENT)).unwrap_err();
    assert!(error.contains("is Claude Code installed?"), "{error}");
}

/// A stand-in for Claude Code's command: it logs its arguments, and fails `add` when `fail` is set.
#[cfg(unix)]
fn fake_claude(dir: &Path, fail: bool) -> String {
    use std::os::unix::fs::PermissionsExt;
    let script = dir.join("claude");
    let log = dir.join("log");
    let failing = if fail {
        "[ \"$2\" = add ] && { echo 'already broken' >&2; exit 1; }"
    } else {
        ""
    };
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\necho \"$*\" >> '{}'\n{failing}\nexit 0\n",
            log.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    script.display().to_string()
}

#[cfg(unix)]
#[test]
fn claude_code_gets_the_agent_for_every_project_replacing_an_older_entry() {
    let dir = tempfile::tempdir().unwrap();
    let said = claude_code(&fake_claude(dir.path(), false), Path::new(AGENT)).unwrap();
    assert_eq!(
        said,
        format!("Claude Code: twinova added for every project, running {AGENT}")
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("log")).unwrap(),
        format!("mcp remove --scope user twinova\nmcp add --scope user twinova -- {AGENT}\n")
    );
    let error = claude_code(&fake_claude(dir.path(), true), Path::new(AGENT)).unwrap_err();
    let claude = dir.path().join("claude");
    assert_eq!(
        error,
        format!(
            "Claude Code: `{} mcp add` failed: already broken",
            claude.display()
        )
    );
}
