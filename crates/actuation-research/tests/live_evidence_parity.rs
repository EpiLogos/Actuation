//! Offline parity proofs over the recorded 2026-09-13 live-provider runs.
//!
//! The live run manifests are owner evidence and stay untracked. This suite
//! reads the tracked receipts under `fixtures/research/` — the recorded
//! digests, the recorded held-constant determination and the composed prompts
//! extracted from those runs — and asserts the Rust-native harness reproduces
//! them: the frozen JSON twins (tasks, conditions) and the Rust comparison
//! mechanics carry the same live behaviour the JavaScript runner recorded.
//! Nothing here performs a provider call.

use actuation_research::{comparison, evidence::stable_digest, execution, prime, tasks::Task};
use serde_json::{json, Value};
use std::path::PathBuf;

fn receipt(rel: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/research")
        .join(rel);
    serde_json::from_str(
        &std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("parity receipt {} must read: {e}", path.display())),
    )
    .unwrap_or_else(|e| panic!("parity receipt must parse: {e}"))
}

fn exact_digest(receipt: &Value, name: &str) -> String {
    let entry = &receipt["recorded_digests"][name];
    assert_eq!(
        entry["parity"],
        json!("exact"),
        "{name} is a runner-local identity, not a twin-parity digest"
    );
    entry["value"].as_str().expect("digest text").to_owned()
}

#[test]
fn series1_live_run_task_digests_reproduce_from_the_frozen_twin() {
    let receipt = receipt("series1/2026-09-13-glm-native-S1-RESTRAINT-001.json");
    let task = Task::get("S1-RESTRAINT-001").expect("frozen twin task");
    let candidate = task.candidate();
    assert_eq!(
        stable_digest(&candidate["prompt"]),
        exact_digest(&receipt, "prompt"),
        "prompt twin drifted from the recorded live run"
    );
    assert_eq!(
        stable_digest(&candidate["successConditions"]),
        exact_digest(&receipt, "success_constraints"),
        "success-constraints twin drifted from the recorded live run"
    );
    // The receipt is bound to the exact live manifest it was extracted from.
    let prefix = "experiments/ql-runtime/comparison/series1/runs/";
    assert!(
        receipt["source"]["path"]
            .as_str()
            .unwrap()
            .starts_with(prefix),
        "receipt source path moved"
    );
    assert_eq!(receipt["source"]["sha256"].as_str().unwrap().len(), 64);
}

#[test]
fn series1_live_run_held_constants_are_reproduced_by_the_rust_mechanics() {
    let receipt = receipt("series1/2026-09-13-glm-native-S1-RESTRAINT-001.json");
    let records = receipt["records"].as_array().expect("records array");
    assert_eq!(records.len(), 3, "classic/ql-direct/ql-deep repetition");
    let recomputed = comparison::compare_held_constant(records);
    assert_eq!(
        recomputed, receipt["recorded_held_constant"],
        "Rust held-constant mechanics diverge from the recorded live determination"
    );
    assert_eq!(recomputed["valid"], json!(true));
    assert_eq!(recomputed["mismatches"], json!([]));
}

#[test]
fn series1_mask_mapping_is_deterministic_and_condition_blinding() {
    let receipt = receipt("series1/2026-09-13-glm-native-S1-RESTRAINT-001.json");
    let manifest = json!({
        "benchmark_revision": receipt["source"]["benchmark_revision"],
        "host": {"id": "native"},
        "task": {"id": "S1-RESTRAINT-001"},
        "records": receipt["records"],
    });
    let first = comparison::mask_mapping(&manifest).expect("mask mapping");
    let second = comparison::mask_mapping(&manifest).expect("mask mapping");
    assert_eq!(first, second, "mask mapping must be deterministic");
    let row = &first["mapping"]["0"];
    let labels: Vec<&str> = row
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(labels.len(), 3);
    let blinded: Vec<String> = row
        .as_object()
        .unwrap()
        .values()
        .map(|v| v.as_str().expect("label text").to_owned())
        .collect();
    assert_eq!(blinded.len(), 3);
    let mut distinct = blinded.clone();
    distinct.sort();
    distinct.dedup();
    assert_eq!(
        distinct.len(),
        3,
        "each condition receives a distinct label"
    );
    for label in &blinded {
        assert!(
            label.starts_with("Candidate "),
            "review labels must not name the condition"
        );
    }
    assert_eq!(first["schema"], json!("ql-series1-mask-map/0.1"));
}

#[test]
fn prime_live_prompts_reproduce_from_the_json_twin_condition_and_task() {
    for (file, condition_id) in [
        ("prime/2026-09-13-glm-p0.json", "prime-native"),
        ("prime/2026-09-13-glm-p3.json", "prime-relational-return"),
    ] {
        let receipt = receipt(file);
        assert_eq!(receipt["condition_id"], json!(condition_id));
        let condition = prime::get_condition(condition_id).expect("twin condition");
        let task = json!({
            "prompt": receipt["task"]["prompt"],
            "successConditions": receipt["task"]["success_conditions"],
        });
        let composed = prime::condition_prompt(&condition, &task).expect("prompt composition");
        assert_eq!(
            composed,
            receipt["recorded_composed_prompt"]
                .as_str()
                .expect("recorded prompt"),
            "{condition_id} prompt composition drifted from the recorded live run"
        );
        // Receipt integrity: bound to the exact live run it was extracted from.
        assert_eq!(receipt["source"]["sha256"].as_str().unwrap().len(), 64);
    }
}

#[test]
fn prime_p3_live_claims_match_the_return_contract_condition() {
    let receipt = receipt("prime/2026-09-13-glm-p3.json");
    assert_eq!(receipt["condition_code"], json!("P3"));
    // The live relational-return run recorded a passing negative control: the
    // faculty was available and correctly not exercised.
    assert_eq!(
        receipt["claims"]["ql_relational_faculty_exercised"],
        json!(false)
    );
    assert_eq!(receipt["claims"]["live_prime_run"], json!(true));
    let condition = prime::get_condition("prime-relational-return").unwrap();
    assert_eq!(condition["returnContract"], json!(true));
    assert_eq!(condition["relational"], json!(true));
}

#[test]
fn native_capability_surface_is_the_recorded_four() {
    let receipt = receipt("series1/2026-09-13-glm-native-S1-RESTRAINT-001.json");
    let recorded = receipt["records"][0]["execution_status"]
        .as_str()
        .expect("execution status");
    assert_eq!(recorded, "completed", "the receipt records a completed run");
    // The four shared capabilities are the harness's whole capability surface.
    assert_eq!(
        execution::CAPABILITIES,
        &["list_files", "read_file", "write_file", "run_tests"]
    );
}
