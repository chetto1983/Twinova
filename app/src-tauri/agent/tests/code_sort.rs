//! The agent drives the 4.4d line (spec 2026-11-11-sensors-products-design.md, acceptance 8): it runs the code and
//! colour sort line until the reject sink counts, reads the scanner's and the colour sensor's signals, and sees each
//! product's size and colour in the snapshot.

mod common;

use common::{fixture, http, ok};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn an_agent_runs_the_code_sort_line_and_reads_its_sensors() {
    let (c, _) = http().await;
    let opened = ok(
        &c,
        "open_project",
        json!({ "path": fixture("code_sort_line") }),
    )
    .await;
    assert_eq!(opened["warnings"], json!([]));
    let until = ok(
        &c,
        "run_until",
        json!({ "sink": "sink_reject", "at_least": 1, "timeout_ms": 60_000 }),
    )
    .await;
    assert_eq!(until["held"], json!(true));
    // The first blue carton is rejected: the scanner read the three cartons before it on belt B, and its code was 3.
    let read = ok(
        &c,
        "read",
        json!({ "signals": ["scan_b.count", "scan_b.code", "s_color.color", "s_laser.distance"] }),
    )
    .await;
    assert_eq!(read["scan_b.code"], json!("3"), "{read}");
    let count: u32 = read["scan_b.count"].as_str().unwrap().parse().unwrap();
    assert!(count >= 2, "{read}");
    let snapshot = ok(&c, "snapshot", json!({ "cartons": true })).await;
    let cartons = snapshot["cartons"].as_array().unwrap();
    assert!(!cartons.is_empty());
    for carton in cartons {
        // The size travels as the physics' f32: 0.3 m within its rounding.
        let size: Vec<f64> = serde_json::from_value(carton["size"].clone()).unwrap();
        assert!(size.iter().all(|s| (s - 0.3).abs() < 1e-6), "{carton}");
        assert!(
            [json!([255, 0, 0]), json!([0, 255, 0]), json!([0, 0, 255])].contains(&carton["color"]),
            "{carton}"
        );
    }
    c.cancel().await.unwrap();
}
