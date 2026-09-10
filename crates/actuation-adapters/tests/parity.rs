mod support;
use serde_json::{json, Value};
#[test]
fn all_observation_wire_cases_and_relocated_usage_codecs_preserve_the_frozen_contract() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/migration/oracle.json")).unwrap();
    let prefixes = [
        "contracts/harness-capability.mjs#",
        "contracts/harness-detection.mjs#",
        "contracts/instantiation.mjs#",
        "contracts/secret-detection.mjs#",
        "contracts/model-usage.mjs#modelUsageFrom",
    ];
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
    assert_eq!(
        rows.len(),
        192,
        "179 R5 and 13 relocated usage cases must remain executable"
    );
    let mut failures = Vec::new();
    for row in rows {
        let actual = support::pure_row(row);
        if actual["ok"] != row["expected"]["ok"]
            || (actual["ok"] == true && actual["value"] != row["expected"]["value"])
        {
            failures.push(json!({"id":row["id"],"operation":row["operation"],"actual":actual,"expected":row["expected"]}));
        }
    }
    assert!(
        failures.is_empty(),
        "{}",
        serde_json::to_string_pretty(&failures).unwrap()
    );
}
#[test]
fn frozen_target_catalogue_and_effect_calls_preserve_presence_absence_and_ambiguity() {
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
        let answer = support::scenario(row).unwrap_or_else(|e| panic!("{}: {e}", row["id"]));
        assert_eq!(
            support::normalized(answer),
            support::normalized(row["expected"].clone()),
            "{}",
            row["id"]
        );
    }
}

#[test]
fn pinned_heterogeneous_target_observations_match_original_effects() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../fixtures/migration/r5/observations.json"
    ))
    .unwrap();
    let rows = corpus["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 21);
    for row in rows {
        assert_eq!(
            support::normalized(support::scenario(row).unwrap()),
            support::normalized(row["expected"].clone()),
            "{}",
            row["id"]
        );
    }
}
#[test]
fn recorded_detector_defects_are_corrected_instead_of_recaptured_as_law() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../fixtures/migration/r5/observations.json"
    ))
    .unwrap();
    let rows = corpus["defects"].as_array().unwrap();
    assert_eq!(rows.len(), 3);
    for row in rows {
        let current = support::scenario(row).unwrap();
        let entry = &current["result"]["value"]["harnesses"][0];
        assert_eq!(current["result"]["ok"], true);
        assert_ne!(
            current, row["expected"],
            "a documented defect must not survive the refoundation"
        );
        match row["id"].as_str().unwrap() {
            "absence-detail-is-not-an-executable" => {
                assert_eq!(entry["state"], "detected");
                assert_eq!(entry["receipts"]["executable"], "/home/oracle/.partial");
                assert_eq!(entry["receipts"]["executable_is"], "config-dir");
                assert!(entry.get("version").is_none());
                assert!(!current["calls"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|c| c["effect"] == "versionProbe"
                        || c["effect"] == "hashProbe"
                        || c["args"][0] == "not found on PATH"));
            }
            "resolved-path-is-not-a-diagnostic" => {
                assert_eq!(entry["state"], "detected");
                assert_eq!(
                    entry["receipts"]["executable"],
                    "/specimen/not found but present"
                );
            }
            "marker-does-not-mask-failed-presence-probe" => {
                assert_eq!(entry["state"], "unavailable");
                assert_eq!(current["result"]["value"]["availability"], "partial");
            }
            _ => panic!("unreviewed correction"),
        }
    }
}
