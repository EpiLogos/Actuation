mod support;
use serde_json::Value;
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
