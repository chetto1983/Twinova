//! Snapping over MCP (spec 2026-11-11-snapping-design.md, "The agent"): describe lists each instance's interfaces, a
//! move drops the joins it breaks, and join poses a part and joins it, or merges a spur onto a flank.

mod common;

use common::{fixture, ok, refused};
use serde_json::{Value, json};

fn has(described: &Value, connection: Value) -> bool {
    described["connections"]
        .as_array()
        .unwrap()
        .contains(&connection)
}

#[tokio::test(flavor = "multi_thread")]
async fn an_agent_snaps_a_belt_back_to_its_line() {
    let c = common::http().await.0;
    ok(
        &c,
        "open_project",
        json!({ "path": fixture("two_belts_diverter") }),
    )
    .await;
    let described = ok(&c, "describe", json!({})).await;
    let ends: Vec<&Value> = described["interfaces"]["belt_b"]["ends"]
        .as_array()
        .unwrap()
        .iter()
        .collect();
    assert_eq!(
        ends.iter().map(|e| e["name"].clone()).collect::<Vec<_>>(),
        [json!("in"), json!("out")]
    );
    assert_eq!(
        described["interfaces"]["roller_r"]["host"]["rail"]["kind"],
        "line"
    );
    let flow = json!({ "kind": "flow", "from": ["roller_r", "out"], "to": ["belt_b", "in"] });
    assert!(has(&described, flow.clone()));

    ok(
        &c,
        "move_instance",
        json!({ "id": "belt_b", "position": [9.0, 0.0, 9.0] }),
    )
    .await;
    assert!(
        !has(&ok(&c, "describe", json!({})).await, flow.clone()),
        "the move broke the join"
    );

    let joined = ok(
        &c,
        "join",
        json!({ "id": "belt_b", "end": "in", "peer": "roller_r", "peer_end": "out" }),
    )
    .await;
    assert_eq!(joined, json!({ "notice": null }));
    let described = ok(&c, "describe", json!({})).await;
    assert!(has(&described, flow));
    let belt_b = (described["instances"].as_array().unwrap().iter())
        .find(|i| i["id"] == "belt_b")
        .unwrap();
    let x = belt_b["position"][0].as_f64().unwrap();
    assert!((x - 5.17).abs() < 1e-6, "{belt_b}");
    let built = ok(&c, "build", json!({})).await;
    // The fixture's own warnings (feedback its program leaves unread) stay; the rejoined line builds.
    assert!(
        built["instances"].as_array().is_some_and(|i| !i.is_empty()),
        "{built}"
    );

    let refusal = refused(&c, "join", json!({ "id": "belt_b", "peer": "roller_r" })).await;
    assert_eq!(
        refusal,
        "give peer_end to join two ends, or at and side to merge onto a flank"
    );
    let refusal = refused(
        &c,
        "join",
        json!({ "id": "belt_b", "peer": "roller_r", "at": 1.0 }),
    )
    .await;
    assert_eq!(refusal, "give side, left or right, to merge");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_agent_merges_a_spur_onto_a_flank() {
    let c = common::http().await.0;
    ok(&c, "open_project", json!({ "path": fixture("merge_line") })).await;
    let merge = json!({ "kind": "merge", "from": ["belt_s", "out"], "host": ["belt_a", "rail"], "at": 1.5, "side": "left" });
    ok(
        &c,
        "join",
        json!({ "id": "belt_s", "peer": "belt_a", "at": 1.5, "side": "left", "gap": 0.04 }),
    )
    .await;
    let described = ok(&c, "describe", json!({})).await;
    assert!(has(&described, merge), "{}", described["connections"]);
    let back = json!({ "kind": "merge", "from": ["belt_s", "out"], "host": ["belt_a", "rail"], "at": 2.0, "side": "right" });
    ok(
        &c,
        "join",
        json!({ "id": "belt_s", "peer": "belt_a", "at": 2.0, "side": "right" }),
    )
    .await;
    assert!(has(&ok(&c, "describe", json!({})).await, back));
}
