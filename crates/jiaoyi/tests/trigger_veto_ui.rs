use serde_json::json;

#[test]
fn numeric_distance_keeps_every_gate_visible_and_escaped() {
    let reasons=[
        "Vetoed: 5d/20d macro drift opposes impulse (5d: -2.790%, 20d: -6.870%)",
        "Vetoed: peer ETHUSD < 24 bps or opposing (missing or stale quote/preparation); mandatory at dispatch",
        "Blocked: pair reservation held by sibling symbol NAS100",
        "Armed: waiting 50%–61.8% pullback + 8% turn (ref: 7398.6, extreme: 7450)",
        "Invalidated: retraced > 61.8% without turn confirmation",
        "Blocked: <img src=x onerror=window.bad=true>",
    ];
    let triggers: serde_json::Map<String, serde_json::Value> = (0..25)
        .map(|i| {
            (
                format!("TEST{i:02}"),
                json!({"at":"2026-11-09T11:00:00Z","distance_half_widths":-0.25,
        "entry_gates":[{"code":"fixture","message":reasons[i%reasons.len()]}]}),
            )
        })
        .collect();
    let health = json!({"strategy3":[{"account_id":49236051,"environment":"demo","trigger_distance":triggers}]});
    let html = jiaoyi::status_page::page(
        &health,
        &json!({}),
        &json!({}),
        "2026-11-09T11:00:00Z".parse().unwrap(),
    );
    assert!(html.contains("Entry state / gate"));
    assert_eq!(html.matches("class=\"entry-gate\"").count(), 25);
    assert_eq!(html.matches(">-0.25</td>").count(), 25);
    assert!(html.contains("5d: -2.790%, 20d: -6.870%"));
    assert!(html.contains("peer ETHUSD &lt; 24 bps"));
    assert!(html.contains("pair reservation held by sibling symbol NAS100"));
    assert!(html.contains("Armed: waiting 50%–61.8%"));
    assert!(html.contains("Invalidated: retraced &gt; 61.8%"));
    assert!(html.contains("&lt;img src=x onerror=window.bad=true&gt;"));
    assert!(!html.contains("<img src=x"));
    if let Ok(path) = std::env::var("JIAOYI_UI_FIXTURE_PATH") {
        std::fs::write(path, html).unwrap();
    }
}
