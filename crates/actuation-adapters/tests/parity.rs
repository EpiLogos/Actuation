mod support;
use serde_json::{json, Value};
fn parity(prefixes: &[&str], expected: usize) {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/migration/oracle.json")).unwrap();
    let rows: Vec<_> = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| {
            prefixes
                .iter()
                .any(|p| r["operation"].as_str().unwrap().starts_with(p))
        })
        .collect();
    assert_eq!(rows.len(), expected, "missing frozen cases");
    let mut errors = vec![];
    for row in rows {
        let actual = support::pure_row(row);
        if actual["ok"] != row["expected"]["ok"]
            || (actual["ok"] == true && actual["value"] != row["expected"]["value"])
        {
            errors.push(json!({"id":row["id"],"operation":row["operation"],"expected":row["expected"],"actual":actual}));
        }
    }
    assert!(
        errors.is_empty(),
        "{}",
        serde_json::to_string_pretty(&errors).unwrap()
    );
}
#[test]
fn all_179_frozen_adapter_contract_cases_keep_wire_meaning() {
    parity(
        &[
            "contracts/harness-capability.mjs#",
            "contracts/harness-detection.mjs#",
            "contracts/instantiation.mjs#",
            "contracts/secret-detection.mjs#",
        ],
        179,
    );
}
#[test]
fn all_213_r4_cases_still_execute_with_native_usage_codecs_in_their_owner() {
    parity(
        &[
            "contracts/actuation-stream.mjs#",
            "contracts/activity.mjs#",
            "contracts/model-usage.mjs#",
            "contracts/request-correlation.mjs#",
        ],
        213,
    );
}
#[test]
fn frozen_catalog_probe_and_secret_effect_scenarios_match() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/migration/scenarios.json")).unwrap();
    let rows: Vec<_> = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| matches!(r["kind"].as_str(), Some("catalog" | "probe" | "secret")))
        .collect();
    assert_eq!(rows.len(), 13);
    for row in rows {
        let actual = support::scenario(row).unwrap_or_else(|e| panic!("{}: {e}", row["id"]));
        assert_eq!(
            support::stream::normalized(actual),
            support::stream::normalized(row["expected"].clone()),
            "{}",
            row["id"]
        );
    }
}
