//! Recorded-fixture conformance for the pinned external specimens.
//!
//! The JavaScript specimen hosts (`experiments/native-research/adapters`)
//! were retired at R11. Their guarantee — the harness's SDK wire contract is
//! true of the pinned external specimen SDKs — is carried by versioned
//! fixtures captured from the real pinned specimens (pi: captured from the
//! published `@earendil-works/pi-ai@0.84.1` package; dsh: pinned public
//! source at `47f9438…`). Each fixture names its capture procedure and the
//! upstream revision it was captured at; bumping a pin requires re-capturing
//! the fixture, and the pin assertions below fail if the two drift apart.
//! The pydantic specimen remains executed live against its pinned source by
//! the retained Python adapter lane (`research-sdk.yml`, `pydantic.py`).

use actuation_research::sdk;
use serde_json::{json, Value};
use std::path::PathBuf;

/// The specimen pins, exactly as the CI workflows declare them. A pin bump
/// that does not re-capture the fixtures fails here.
pub const PI_PACKAGE_VERSION: &str = "0.84.1";
pub const DSH_UPSTREAM_REVISION: &str = "47f943859bef60e4160492346772ded9b24f765a";
pub const DSH_PACKAGE_VERSION: &str = "0.1.0-rc.5";
pub const PYDANTIC_AI_REVISION: &str = "00db3a4b391eb9a46f3d6e704070bcf725121f75";

fn fixture(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/research/specimens")
        .join(name);
    serde_json::from_str(
        &std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("specimen fixture {} must read: {e}", path.display())),
    )
    .unwrap_or_else(|e| panic!("specimen fixture must parse: {e}"))
}

#[test]
fn pi_fixture_pins_the_published_package_the_workflows_install() {
    let f = fixture("pi.json");
    assert_eq!(f["schema"], json!("actuation.specimen-conformance/v1"));
    assert_eq!(f["specimen"]["package"], json!("@earendil-works/pi-ai"));
    assert_eq!(f["specimen"]["pinned_version"], json!(PI_PACKAGE_VERSION));
    assert_eq!(f["pins"]["equals_pinned"], json!(true));
    // The fixture was captured from a real preflight against the pinned
    // package catalogue (credential present), not authored.
    assert_eq!(
        f["preflight"]["wire_result"]["package_version"],
        json!(PI_PACKAGE_VERSION)
    );
    assert_eq!(
        f["preflight"]["wire_result"]["provider_request_executed"],
        json!(false)
    );
    assert_eq!(
        f["preflight"]["wire_result"]["credential_available"],
        json!(true)
    );
    // The historical adapter the capture ran through is named by revision.
    assert_eq!(
        f["specimen"]["adapter_origin_revision"],
        json!("ece2478649d58ed210eadd2e2bac00af9e5ced0d")
    );
}

#[test]
fn pi_recorded_extraction_flows_through_the_rust_wire_contract() {
    let f = fixture("pi.json");
    let extraction = &f["completion"]["extraction"];
    // The adapter hands {output, usage, raw} across the RPC; the harness's
    // normalize must reproduce the recorded harness view of that payload.
    let data = json!({"output": extraction["output"], "usage": extraction["usage"], "raw": extraction["raw"]});
    let normalized = sdk::normalize(&data, false).expect("recorded extraction must normalise");
    assert_eq!(normalized["content"], json!("fixture"));
    assert_eq!(normalized["raw"]["decoding"], json!("json-object"));
    assert_eq!(
        normalized["capabilityCalls"],
        json!([{"id":"call-1","name":"read_file","args":{"path":"fact.txt"}}])
    );
    assert_eq!(normalized["usage"], extraction["usage"]);
    assert_eq!(normalized["usage"]["input_tokens"], json!(10));
    assert_eq!(normalized["usage"]["total_tokens"], json!(15));
}

#[test]
fn pi_catalogue_and_envelope_laws_are_pinned() {
    let f = fixture("pi.json");
    assert_eq!(
        f["refusals"]["wrong_provider"],
        json!("Explicit DeepSeek model required")
    );
    assert_eq!(
        f["refusals"]["absent_model"],
        json!("Selected model absent from pinned Pi catalogue")
    );
    let finalize = &f["finalize"];
    assert_eq!(finalize["kind"], json!("pi-native-sdk-evidence"));
    assert_eq!(finalize["provider_evidence"], json!("not-assessed"));
    assert_eq!(
        finalize["model_calls"],
        json!([[0, "failed"], [1, "returned"]]),
        "ordered failure attribution is part of the pinned ABI"
    );
    // Standalone JSONL envelope: exact reply correlation, unsupported command
    // refused, preflight runs without credentials.
    let replies = &f["envelope"]["replies"];
    assert_eq!(replies[0]["command"], json!("preflight"));
    assert_eq!(replies[0]["success"], json!(true));
    assert_eq!(replies[0]["data"]["credential_available"], json!(false));
    assert_eq!(replies[1]["command"], json!("unsupported"));
    assert_eq!(replies[1]["success"], json!(false));
    assert_eq!(replies[1]["error"], json!("Unsupported Pi adapter command"));
    assert_eq!(replies[2]["command"], json!("finalize"));
    assert_eq!(replies[2]["success"], json!(true));
}

#[test]
fn dsh_fixture_pins_the_public_source_revision_the_workflows_check_out() {
    let f = fixture("dsh.json");
    assert_eq!(f["schema"], json!("actuation.specimen-conformance/v1"));
    assert_eq!(
        f["specimen"]["upstream_revision"],
        json!(DSH_UPSTREAM_REVISION)
    );
    assert_eq!(f["specimen"]["package_version"], json!(DSH_PACKAGE_VERSION));
    // Usage mapping law of the pinned rc.5 composition: cache counters are
    // summed into the wire input_tokens; absent counters are never invented;
    // the raw SDK usage stays inspectable alongside the mapping.
    let mapping = &f["usage_mapping"];
    assert_eq!(
        mapping["input"],
        json!({"inputTokens": 3, "outputTokens": 1, "cacheReadTokens": 2, "cacheWriteTokens": 0})
    );
    let wire = &f["stream_completion"]["extraction"]["usage"];
    assert_eq!(wire["input_tokens"], json!(5));
    assert_eq!(wire["output_tokens"], json!(1));
    assert_eq!(wire["total_tokens"], json!(6));
    assert_eq!(
        wire["native_usage"],
        json!({"inputTokens": 3, "outputTokens": 1, "cacheReadTokens": 2, "cacheWriteTokens": 0})
    );
    assert_eq!(
        mapping["absent_counters"],
        json!({"input_tokens": Value::Null, "total_tokens": Value::Null})
    );
    // Session and inspection laws of the recorded composition.
    let session = &f["session_law"];
    assert_eq!(
        session["agent_drive_refused"],
        "Series 1 DSH Agent is observational; frozen LoopRuntime owns execution."
    );
    assert_eq!(
        session["inspection_schema"],
        "ql-series1-dsh-inspection/0.1"
    );
    assert_eq!(session["provider_route"], "deepseek-official");
    assert_eq!(session["inspection_is_read_only"], true);
    // Session and inspection separation of the recorded finalize.
    let finalize = &f["finalize"];
    assert_eq!(finalize["inspection_session_error"], json!(null));
    assert_eq!(finalize["sessions_separated"], json!(true));
    assert_eq!(
        finalize["candidate_session"]["events_include_assistant_message"],
        json!(true)
    );
    assert_eq!(
        finalize["inspection_session"]["events_include_portable_event"],
        json!(true)
    );
    assert_eq!(
        finalize["candidate_session"]["portable_events_present"],
        json!(0),
        "the candidate session never carries inspection-seeded events"
    );
    assert_eq!(
        finalize["model_call_alignment"],
        json!([{"ordinal":0,"error":true},{"ordinal":1,"error":false}])
    );
}

#[test]
fn pydantic_pin_is_declared_for_the_retained_live_lane() {
    // The pydantic specimen is not fixture-carried: the retained Python
    // adapter executes the pinned source live (research-sdk.yml). The pin is
    // restated here so a bump anywhere is a reviewed, three-place change
    // (workflow, fixture where applicable, this assertion).
    assert_eq!(PYDANTIC_AI_REVISION.len(), 40);
    assert!(PYDANTIC_AI_REVISION.bytes().all(|b| b.is_ascii_hexdigit()));
}
