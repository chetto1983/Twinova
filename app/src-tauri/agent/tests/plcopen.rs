//! PLCopen TC6 `<LD>` over MCP (spec 2026-11-13-ladder-editor-design.md, decision 7): what the app's PLC pane exports
//! and imports through the system dialogs, an agent does with paths. The 4.4a line's Main.ld is exported, edited outside
//! and imported over itself, imported again under another name, exported again to the same file, built, and run beside
//! it.

mod common;

use common::{fixture, ok, refused};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn an_agent_exports_a_ladder_program_and_imports_it_back() {
    let folder = tempfile::tempdir().unwrap();
    let exported = folder.path().join("Main.xml");
    let c = common::http().await.0;
    ok(
        &c,
        "open_project",
        json!({ "path": fixture("ladder_line") }),
    )
    .await;

    let written = ok(
        &c,
        "export_ladder",
        json!({ "file": "plc/Main.ld", "path": exported }),
    )
    .await;
    assert_eq!(written, json!(exported));
    let xml = std::fs::read_to_string(&exported).unwrap();
    assert!(
        xml.contains(r#"creationDateTime="1970-01-01T00:00:00""#),
        "{xml}"
    );
    assert!(xml.contains("<LD>"), "{xml}");
    assert_eq!(
        refused(
            &c,
            "export_ladder",
            json!({ "file": "plc/Pump.ld", "path": exported })
        )
        .await,
        "plc/Pump.ld is not a file of the program"
    );

    // edited in another tool and imported again, the file the program has takes the new text
    let edited = folder.path().join("Main-edited.xml");
    assert!(xml.contains("T#800MS"), "{xml}");
    std::fs::write(&edited, xml.replace("T#800MS", "T#900MS")).unwrap();
    assert_eq!(
        ok(&c, "import_ladder", json!({ "path": edited })).await,
        json!({ "replaced": ["plc/Main.ld"], "added": [], "notes": [] })
    );
    let text = ok(&c, "read_source", json!({ "file": "plc/Main.ld" })).await;
    assert!(
        text.as_str()
            .unwrap()
            .contains("deck_clear(IN, PT := T#900MS"),
        "{text}"
    );
    let nowhere = folder.path().join("no/such/folder.xml");
    assert!(
        refused(
            &c,
            "export_ladder",
            json!({ "file": "plc/Main.ld", "path": nowhere })
        )
        .await
        .starts_with(&format!("{}: ", nowhere.display()))
    );
    let missing = folder.path().join("none.xml");
    assert!(
        refused(&c, "import_ladder", json!({ "path": missing }))
            .await
            .starts_with(&format!("{}: ", missing.display()))
    );

    // the same program as a POU of another name, which runs beside Main
    let copy = folder.path().join("Copy.xml");
    let renamed = xml.replace(r#"name="Main""#, r#"name="Copy""#);
    std::fs::write(&copy, &renamed).unwrap();
    let imported = ok(&c, "import_ladder", json!({ "path": copy })).await;
    assert_eq!(imported["added"], json!(["plc/Copy.ld"]));
    assert_eq!(imported["replaced"], json!([]));
    let program = &ok(&c, "describe", json!({})).await["program"];
    assert_eq!(program, &json!(["plc/Main.ld", "plc/Copy.ld"]));
    // the round trip is semantic (spec, decision 7): read back, the program exports to the same file
    let again = folder.path().join("Copy-again.xml");
    ok(
        &c,
        "export_ladder",
        json!({ "file": "plc/Copy.ld", "path": again }),
    )
    .await;
    assert_eq!(std::fs::read_to_string(&again).unwrap(), renamed);
    ok(&c, "build", json!({})).await;
    ok(&c, "step", json!({ "ms": 2000 })).await;
    c.cancel().await.unwrap();
}
