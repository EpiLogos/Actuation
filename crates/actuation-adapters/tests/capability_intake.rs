//! The public intake law for harness capability descriptors (O:I #113 A1).
//! The gemini specimen contributed by the uncoached external author was
//! received and landed at catalog r15 (receipt
//! capability-contribution:gemini:f1af4031414a), and the openclaw specimen
//! followed at r17 (receipt capability-contribution:openclaw:307591449de6),
//! so against the shipped catalog each landed specimen now answers the
//! coverage-closure shadowing refusal — the law working as designed. The
//! minting law that produced those receipts is proven against the
//! reconstructed pre-landing catalog; a fabricated event kind is refused by
//! the closed vocabulary; a descriptor that would shadow a declared
//! capability fails coverage closure; an unknown slug fails slug alignment.
use actuation_adapters::*;
use serde_json::{json, Value};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn gemini_specimen() -> Value {
    let path = format!("{ROOT}/fixtures/capability-contributions/gemini.harness-capability.json");
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"));
    serde_json::from_str(&raw).expect("the gemini specimen is JSON")
}

fn openclaw_specimen() -> Value {
    let path = format!("{ROOT}/fixtures/capability-contributions/openclaw.harness-capability.json");
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"));
    serde_json::from_str(&raw).expect("the openclaw specimen is JSON")
}

/// The declared gemini capability gap exactly as catalog r14 shipped it —
/// the gap the landed receipt withdrew.
const R14_GEMINI_GAP: &str = r#"{
  "schema": "actuation.harness-capability-gap/v1",
  "document": "capability-gap",
  "harness_slug": "gemini",
  "summary": "Gemini CLI agent harness; adapter consumes the detection record, descriptor not authored",
  "reason": "No capability descriptor authored at diagnosis (2026-09-14): the gemini CLI's native event/blocking surface has not been observed into descriptor form. AIKit's adapter cites the detection record for discovery (ai-kit#186 round-2 migration) and records the undeclared capability state honestly; authoring the descriptor is Actuation-side work that had not happened.",
  "evidence_refs": [
    "aikit:clients/gemini.rs (ai-kit#186 round-2 migration, commit 971e8fa)",
    "survey:local-machine-2026-09-05",
    "diagnosis:harness-capability-coverage-2026-09-14"
  ],
  "provenance": {
    "authored_by": "O:I capability-coverage closure (catalog r7), 2026-09-15",
    "source_refs": [
      "diagnosis:harness-capability-coverage-2026-09-14"
    ],
    "catalog_revision": 7
  }
}"#;

/// The shipped catalog as a raw value (the bytes the binary bundles).
fn bundled_catalog_value() -> Value {
    let path = format!("{ROOT}/catalog/targets.json");
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"));
    serde_json::from_str(&raw).expect("the shipped catalog is JSON")
}

/// The catalog as it stood when the gemini receipt was minted: the shipped
/// catalog with the landed gemini capability withdrawn and the r14 declared
/// gap restored.
fn pre_landing_catalog() -> NativeCatalog {
    let mut value = bundled_catalog_value();
    let capabilities = value["capabilities"].as_array_mut().unwrap();
    capabilities.retain(|c| c["harness_slug"] != json!("gemini"));
    value["capability_gaps"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::from_str(R14_GEMINI_GAP).expect("the r14 gap parses"));
    let raw = serde_json::to_string(&value).unwrap();
    NativeCatalog::from_json(&raw).expect("the pre-landing catalog admits")
}

fn result_of(document: &Value, check: &str) -> String {
    document["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["check"] == json!(check))
        .expect("the check is named in the document")["result"]
        .as_str()
        .expect("admitted result")
        .to_owned()
}

#[test]
fn the_landed_gemini_specimen_now_answers_the_shadowing_refusal() {
    let catalog = NativeCatalog::bundled().unwrap();
    assert!(
        catalog.capability("gemini").is_some(),
        "the contribution landed at r15"
    );
    let document = validate_capability_document(&catalog, &gemini_specimen());
    assert_eq!(document["valid"], json!(false), "{document}");
    assert_eq!(result_of(&document, "schema-admission"), "pass");
    assert_eq!(result_of(&document, "slug-alignment"), "pass");
    assert_eq!(result_of(&document, "coverage-closure"), "fail");
    let detail = document["checks"][2]["detail"].as_str().unwrap();
    assert!(
        detail.contains("already carries a declared capability"),
        "the refusal names the landing: {detail}"
    );
    let raw = serde_json::to_string(&gemini_specimen()).unwrap();
    assert!(
        capability_contribution_receipt(
            &catalog,
            &gemini_specimen(),
            "fixtures/capability-contributions/gemini.harness-capability.json",
            &raw,
            1_000,
        )
        .is_err(),
        "a landed contribution is received no longer"
    );
}

#[test]
fn the_landed_openclaw_specimen_now_answers_the_shadowing_refusal() {
    let catalog = NativeCatalog::bundled().unwrap();
    assert!(
        catalog.capability("openclaw").is_some(),
        "the contribution landed at r17"
    );
    let document = validate_capability_document(&catalog, &openclaw_specimen());
    assert_eq!(document["valid"], json!(false), "{document}");
    assert_eq!(result_of(&document, "schema-admission"), "pass");
    assert_eq!(result_of(&document, "slug-alignment"), "pass");
    assert_eq!(result_of(&document, "coverage-closure"), "fail");
    let detail = document["checks"][2]["detail"].as_str().unwrap();
    assert!(
        detail.contains("already carries a declared capability"),
        "the refusal names the landing: {detail}"
    );
    let raw = serde_json::to_string(&openclaw_specimen()).unwrap();
    assert!(
        capability_contribution_receipt(
            &catalog,
            &openclaw_specimen(),
            "fixtures/capability-contributions/openclaw.harness-capability.json",
            &raw,
            1_000,
        )
        .is_err(),
        "a landed contribution is received no longer"
    );
}

#[test]
fn the_minting_law_that_landed_gemini_still_holds() {
    let catalog = pre_landing_catalog();
    let document = validate_capability_document(&catalog, &gemini_specimen());
    assert_eq!(document["valid"], json!(true), "{document}");
    assert_eq!(document["harness_slug"], json!("gemini"));
    for check in ["schema-admission", "slug-alignment", "coverage-closure"] {
        assert_eq!(result_of(&document, check), "pass", "{check} must pass");
    }
    // The receipt the contribution face mints carries the bytes' digest, the
    // unchanged descriptor, and the owner landing edit — the receipt the
    // owner landed at r15.
    let raw = serde_json::to_string(&gemini_specimen()).unwrap();
    let receipt = capability_contribution_receipt(
        &catalog,
        &gemini_specimen(),
        "fixtures/capability-contributions/gemini.harness-capability.json",
        &raw,
        1_000,
    )
    .expect("a validated specimen is received");
    assert_eq!(receipt["status"], json!("received"));
    assert_eq!(receipt["harness_slug"], json!("gemini"));
    assert!(
        receipt["contribution_ref"]
            .as_str()
            .unwrap()
            .starts_with("capability-contribution:gemini:"),
        "{}",
        receipt["contribution_ref"]
    );
    assert_eq!(
        receipt["landing"]["current_revision"],
        json!(catalog.revision())
    );
    assert_eq!(
        receipt["landing"]["landed_revision"],
        json!(catalog.revision() + 1)
    );
    assert!(
        receipt["landing"]["edit"]
            .as_str()
            .unwrap()
            .contains("capability_gaps -= gemini"),
        "{}",
        receipt["landing"]["edit"]
    );
    // The descriptor travels unchanged.
    assert_eq!(receipt["capability"], gemini_specimen());
}

#[test]
fn a_fabricated_event_kind_is_refused_by_the_closed_vocabulary() {
    let catalog = NativeCatalog::bundled().unwrap();
    let mut fabricated = gemini_specimen();
    fabricated["native_events"][0]["event"] = json!("teleport");
    fabricated["native_events"][0]["native_name"] = json!("Teleport");
    let document = validate_capability_document(&catalog, &fabricated);
    assert_eq!(document["valid"], json!(false), "{document}");
    assert_eq!(result_of(&document, "schema-admission"), "fail");
    let detail = document["checks"][0]["detail"].as_str().unwrap();
    assert!(
        detail.contains("expected one of"),
        "the refusal names the closed event vocabulary: {detail}"
    );
    assert!(capability_contribution_receipt(&catalog, &fabricated, "stdin", "{}", 0).is_err());
}

#[test]
fn a_descriptor_shadowing_a_declared_capability_fails_coverage_closure() {
    let catalog = NativeCatalog::bundled().unwrap();
    let declared = catalog.capability("pi").expect("pi is declared").as_value();
    let document = validate_capability_document(&catalog, declared);
    assert_eq!(document["valid"], json!(false), "{document}");
    assert_eq!(result_of(&document, "coverage-closure"), "fail");
    let detail = document["checks"][2]["detail"].as_str().unwrap();
    assert!(
        detail.contains("already carries a declared capability"),
        "the refusal names the shadowing: {detail}"
    );
    assert!(
        detail.contains("owner edit"),
        "the refusal points at the correction discipline: {detail}"
    );
}

#[test]
fn an_unknown_slug_fails_slug_alignment() {
    let catalog = NativeCatalog::bundled().unwrap();
    let mut stranger = gemini_specimen();
    stranger["harness_slug"] = json!("invented-harness");
    let document = validate_capability_document(&catalog, &stranger);
    assert_eq!(document["valid"], json!(false), "{document}");
    assert_eq!(result_of(&document, "slug-alignment"), "fail");
    let detail = document["checks"][1]["detail"].as_str().unwrap();
    assert!(
        detail.contains("names no detection descriptor"),
        "the refusal names the misalignment: {detail}"
    );
}
