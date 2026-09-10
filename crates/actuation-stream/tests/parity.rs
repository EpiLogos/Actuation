mod support;
use serde_json::Value;
#[test]
fn every_frozen_generic_r4_case_matches_the_native_domain() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/migration/oracle.json")).unwrap();
    let prefixes = [
        "contracts/actuation-stream.mjs#",
        "contracts/activity.mjs#",
        "contracts/model-usage.mjs#",
        "contracts/request-correlation.mjs#",
    ];
    let rows: Vec<_> = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| {
            !r["operation"]
                .as_str()
                .unwrap()
                .starts_with("contracts/model-usage.mjs#modelUsageFrom")
                && prefixes
                    .iter()
                    .any(|p| r["operation"].as_str().unwrap().starts_with(p))
        })
        .collect();
    assert_eq!(rows.len(), 200, "missing frozen cases");
    let mut errors = Vec::new();
    for row in rows {
        let answer = support::pure_row(row);
        if answer["ok"] != row["expected"]["ok"]
            || (answer["ok"] == true && answer["value"] != row["expected"]["value"])
        {
            errors.push(serde_json::json!({"id":row["id"],"operation":row["operation"],"expected":row["expected"],"actual":answer}));
        }
    }
    assert!(
        errors.is_empty(),
        "{}",
        serde_json::to_string_pretty(&errors).unwrap()
    );
}
#[test]
fn frozen_jsonl_scenarios_preserve_actual_bytes_and_refusal_effects() {
    let corpus = support::corpus();
    let rows: Vec<_> = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| matches!(r["kind"].as_str(), Some("store" | "fold" | "filename")))
        .collect();
    assert_eq!(rows.len(), 21);
    for row in rows {
        let actual = support::scenario(row).unwrap_or_else(|e| panic!("{}: {e}", row["id"]));
        assert_eq!(
            support::normalized(actual),
            support::normalized(row["expected"].clone()),
            "{}",
            row["id"]
        );
    }
}

#[test]
fn standalone_original_node_stores_remain_readable_without_migration() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/migration/r4");
    let manifest: Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("manifest.json")).unwrap())
            .unwrap();
    for entry in manifest["stores"].as_array().unwrap() {
        let raw = std::fs::read_to_string(root.join(entry["path"].as_str().unwrap())).unwrap();
        let stream = actuation_stream::fold_stream_file(&raw).unwrap();
        assert_eq!(
            stream.fields().stream_ref.as_str(),
            entry["stream_ref"].as_str().unwrap()
        );
        assert_eq!(
            stream.fields().events.len() as u64,
            stream.fields().cursor.fields().last_sequence.get()
        );
    }
}
