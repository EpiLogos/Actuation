//! The native verification gate. `actuation verify` runs this compiled-in
//! suite: deterministic checks over the shipped product's own application
//! surfaces. The suite discovers nothing at runtime and depends on no external
//! runtime — an empty suite can never pass, because the table itself is
//! asserted non-empty and every entry must report ok.
use crate::dispatch::{execute, git_revision};
use actuation_core::StreamRef;
use actuation_stream::UsageOccurrence;
use actuation_stream::{JsonlStreamStore, StreamStore};
use serde_json::{json, Value};

type Check = (&'static str, fn() -> std::result::Result<(), String>);

/// One frozen durable store from the R4 wire corpus (Node-written bytes,
/// sha256-published in fixtures/migration/r4/manifest.json). The shipped
/// binary carries the historical wire oracle: if the native reader ever stops
/// reading what the Node product wrote, verification fails here.
const FROZEN_STORE_01: &str = include_str!("../../../fixtures/migration/r4/stores/01.jsonl");
const FROZEN_STORE_01_SHA256: &str =
    "1009dfc27f2e2ed5aacefe63552f7b4598d2cb920ab0f509cc2264542b616483";

pub struct VerifyReceipt {
    pub status: &'static str,
    pub tests: Vec<&'static str>,
    pub failure: Option<String>,
}

pub fn suite() -> Vec<Check> {
    vec![
        ("surface/self-description", check_surface_self_description),
        ("dispatch/help-and-routing", check_help_and_routing),
        ("dispatch/refusals-fail-closed", check_refusals_fail_closed),
        ("agency/read-model-golden", check_agency_golden),
        ("agency/actualise-receipt-golden", check_actualise_golden),
        ("realised/read-model-golden", check_realised_golden),
        ("stream/read-model-golden", check_stream_golden),
        ("activity/read-model-golden", check_activity_golden),
        ("usage/adapter-and-read-golden", check_usage_golden),
        (
            "instantiation/legacy-read-golden",
            check_instantiation_golden,
        ),
        (
            "stream/durable-store-lifecycle",
            check_durable_store_lifecycle,
        ),
        (
            "stream/frozen-wire-oracle-interop",
            check_frozen_wire_oracle,
        ),
        ("harness/catalog-declared", check_harness_catalog),
        ("system/disclosure-digest-stable", check_disclosure_digest),
        (
            "verify/receipt-names-the-suite",
            check_receipt_names_the_suite,
        ),
    ]
}

pub fn run() -> VerifyReceipt {
    let tests = suite();
    if tests.is_empty() {
        return VerifyReceipt {
            status: "failed",
            tests: Vec::new(),
            failure: Some(
                "no deterministic tests selected; the verification gate refuses to pass empty"
                    .to_owned(),
            ),
        };
    }
    for (name, check) in &tests {
        if let Err(failure) = check() {
            return VerifyReceipt {
                status: "failed",
                tests: tests.iter().map(|(name, _)| *name).collect(),
                failure: Some(format!("{name}: {failure}")),
            };
        }
    }
    VerifyReceipt {
        status: "ok",
        tests: tests.iter().map(|(name, _)| *name).collect(),
        failure: None,
    }
}

fn run_cli(argv: &[&str], stdin: &str) -> std::result::Result<Value, String> {
    let args: Vec<String> = argv.iter().map(|s| s.to_string()).collect();
    let output = execute(&args, stdin).map_err(|e| format!("{argv:?}: {e}"))?;
    if output.code != 0 {
        return Err(format!(
            "{argv:?}: exit {} stderr {}",
            output.code, output.stderr
        ));
    }
    serde_json::from_str(&output.stdout).map_err(|e| format!("{argv:?}: stdout not JSON: {e}"))
}

fn run_cli_raw(argv: &[&str], stdin: &str) -> std::result::Result<String, String> {
    let args: Vec<String> = argv.iter().map(|s| s.to_string()).collect();
    let output = execute(&args, stdin).map_err(|e| format!("{argv:?}: {e}"))?;
    Ok(output.stdout)
}

fn check_surface_self_description() -> std::result::Result<(), String> {
    let value = run_cli(&["capabilities", "--json"], "")?;
    if value["contract"] != json!("actuation.cli/v1") {
        return Err(format!("contract drift: {}", value["contract"]));
    }
    if value["executable"] != json!("actuation") || value["product"] != json!("actuation") {
        return Err("product/executable drift".into());
    }
    let commands = value["commands"]
        .as_array()
        .ok_or("capabilities must declare the command table")?;
    if commands.len() < 21 {
        return Err(format!(
            "command surface shrank: {} entries",
            commands.len()
        ));
    }
    let revision = value["revision"].as_str().ok_or("revision must be text")?;
    if revision != "unknown" && !(7..=40).contains(&revision.len()) {
        return Err(format!("revision is neither unknown nor a sha: {revision}"));
    }
    if run_cli_raw(&["--version"], "")?.trim()
        != format!("actuation {}", crate::surface::ACTUATION_CLI_VERSION)
    {
        return Err("--version drifted".into());
    }
    Ok(())
}

fn check_help_and_routing() -> std::result::Result<(), String> {
    let help = run_cli_raw(&["help"], "")?;
    if !help.starts_with("Actuation ") || !help.contains("Usage:") {
        return Err("help text lost its shape".into());
    }
    for route in [
        "actuation capabilities",
        "actuation stream replay",
        "actuation verify",
    ] {
        if !help.contains(route) {
            return Err(format!("help no longer documents `{route}`"));
        }
    }
    Ok(())
}

fn check_refusals_fail_closed() -> std::result::Result<(), String> {
    let args: Vec<String> = vec!["invented".into()];
    let error = execute(&args, "").map_err(|e| e.to_string());
    match error {
        Err(e) if e.to_string().contains("unknown command") => Ok(()),
        Err(e) => Err(format!("unexpected refusal: {e}")),
        Ok(_) => Err("an unknown command must fail, not fall through".into()),
    }
}

fn check_agency_golden() -> std::result::Result<(), String> {
    let value = run_cli(
        &["agency", "-", "--json"],
        r#"{"binding":{"schema":"actuation.agency/v1","binding_ref":"world-binding:verify","agent_ref":"agent:verify","agency_ref":"agency:verify","world_ref":"world:verify","scope_ref":"scope:verify"},"root_scope":{"schema":"actuation.agency/v1","scope_ref":"scope:verify","enclosing_world_ref":"world:verify"}}"#,
    )?;
    if value["agency_ref"] != json!("agency:verify")
        || value["root_for_scope"] != json!(true)
        || value["metagency"]["available"] != json!(false)
    {
        return Err(format!("agency read model drifted: {value}"));
    }
    Ok(())
}

fn check_actualise_golden() -> std::result::Result<(), String> {
    // Frozen wire document from the migration corpus (scenarios.json,
    // cli-actualise-json): the served delegation request and its receipt law.
    const GOLDEN: &str = r#"{"schema":"actuation.agency-actualisation/v1","request_ref":"actualisation-request:delegation","requester_ref":"human:owner","governing_binding":{"schema":"actuation.agency/v1","binding_ref":"binding:governing","agent_ref":"agent:governor","agency_ref":"agency:governing","world_ref":"world:personal","scope_ref":"scope:personal","bounds_refs":["bound:personal","bound:project:delegation","bound:secondary"],"authority_refs":["authority:metagency","authority:project:delegation"],"return_relation_ref":"return-relation:governing"},"metagency_grant":{"schema":"actuation.agency/v1","grant_ref":"grant:project-agency","agency_ref":"agency:governing","world_binding_ref":"binding:governing","authority_ref":"authority:metagency","bounds_refs":["bound:project:delegation","bound:secondary"],"operations":["determine-agency","actualise-agency"]},"determination":{"schema":"actuation.agency/v1","determination_ref":"determination:delegation","kind":"delegation","determining_agency_ref":"agency:governing","differentiated_agency_ref":"agency:project:delegation","world_binding_ref":"binding:project:delegation","bounds_refs":["bound:project:delegation","bound:secondary"],"authority_refs":["authority:project:delegation"],"delegated_autonomy":{"allowed_action_refs":["action:bounded-work"],"denied_action_refs":["action:source-mutation"],"may_determine_within_bounds":true},"return_policy":{"mode":"required","return_relation_ref":"return-relation:project:delegation"}},"differentiated_binding":{"schema":"actuation.agency/v1","binding_ref":"binding:project:delegation","agent_ref":"agent:existing-1","agency_ref":"agency:project:delegation","world_ref":"central:project:Example","scope_ref":"central:project:Example:scope","determining_agency_ref":"agency:governing","bounds_refs":["bound:secondary","bound:project:delegation"],"authority_refs":["authority:project:delegation"],"return_relation_ref":"return-relation:project:delegation","continuity_ref":"continuity:agent:existing-1"},"agent_identity":{"standing":"existing","evidence_refs":["evidence:identity-registry"]},"provenance":{"source_refs":["actuation:#44"],"context_refs":[]}}"#;
    let value = run_cli(&["agency", "actualise", "-", "--json"], GOLDEN)?;
    if value["determination"]["kind"] != json!("delegation")
        || value["differentiated_binding"]["agency_ref"] != json!("agency:project:delegation")
        || value["metagency"]["grant_ref"] != json!("grant:project-agency")
    {
        return Err(format!("actualisation receipt drifted: {value}"));
    }
    Ok(())
}

fn check_realised_golden() -> std::result::Result<(), String> {
    let value = run_cli(
        &["realised", "-", "--json"],
        r#"{"schema":"actuation.realised/v1","realised_ref":"realised/verify/1","actuation_ref":"actuation:verify","agent_ref":"agent:verify","agency_ref":"agency:verify","world_binding_ref":"world:verify","loop":{"recurrence":"turn-based","acting":true,"entrypoint_ref":"native:verify","observed_faculties":["filesystem"]},"body":{"model_condition_ref":"actuation:model-bearing/verify"},"observation":{"state":"observed","evidence_refs":["evidence:verify"]}}"#,
    )?;
    if value["realised_ref"] != json!("realised/verify/1")
        || value["observation"]["state"] != json!("observed")
    {
        return Err(format!("realised read model drifted: {value}"));
    }
    Ok(())
}

fn check_stream_golden() -> std::result::Result<(), String> {
    let value = run_cli(
        &["stream", "-", "--json"],
        r#"{"schema":"actuation.stream/v1","stream_ref":"stream:verify","actuation_ref":"actuation:verify","agency_ref":"agency:verify","agent_session_ref":"session:verify","lifecycle":{"state":"open","started_at":"2026-09-12T00:00:00Z"},"cursor":{"last_sequence":0,"next_sequence":1},"events":[]}"#,
    )?;
    if value["stream_ref"] != json!("stream:verify") || value["lifecycle"]["state"] != json!("open")
    {
        return Err(format!("stream read model drifted: {value}"));
    }
    Ok(())
}

fn check_activity_golden() -> std::result::Result<(), String> {
    let stream = r#"{"schema":"actuation.stream/v1","stream_ref":"stream:verify-activity","actuation_ref":"actuation:verify","agency_ref":"agency:verify","agent_session_ref":"session:verify","lifecycle":{"state":"open","started_at":"2026-09-12T00:00:00Z"},"cursor":{"last_sequence":2,"next_sequence":3},"events":[{"event_ref":"event:1","sequence":1,"kind":"tool-request","observed_at":"2026-09-12T00:00:01Z","actor":{"locus_ref":"locus:builder","agency_ref":"agency:verify","agent_ref":"agent:builder"},"disclosure":"portable","native_trace_ref":"trace:1","resource_refs":["action:verify"]},{"event_ref":"event:2","sequence":2,"kind":"tool-result","observed_at":"2026-09-12T00:00:02Z","actor":{"locus_ref":"locus:builder","agency_ref":"agency:verify","agent_ref":"agent:builder"},"disclosure":"portable","native_trace_ref":"trace:1","resource_refs":["result:verify:1"]}]}"#;
    let activity = json!({
        "activityRef": "activity:verify",
        "subjectRef": "subject:verify",
        "nativeOwner": "actuation",
        "actionRef": "action:verify",
        "invocationRef": "invocation:verify",
        "resultRef": "result:verify:1",
        "verb": "verified",
        "object": "verify suite",
        "summary": "The compiled-in verification suite exercised the product.",
        "evidenceRefs": ["evidence:verify"],
    });
    let description: actuation_stream::ActivityDescription =
        serde_json::from_value(activity).map_err(|e| e.to_string())?;
    let activity = actuation_stream::Activity::from_stream(
        &actuation_stream::ActuationStream::try_from(
            serde_json::from_str::<Value>(stream).expect("golden stream"),
        )
        .map_err(|e| e.to_string())?,
        description,
    )
    .map_err(|e| e.to_string())?;
    let value = run_cli(
        &["activity", "-", "--json"],
        &serde_json::to_string(&activity).map_err(|e| e.to_string())?,
    )?;
    if value["activity_ref"] != json!("activity:verify") {
        return Err(format!("activity read model drifted: {value}"));
    }
    Ok(())
}

fn check_usage_golden() -> std::result::Result<(), String> {
    let observation = actuation_adapters::usage::from_claude_code_transcript(
        &json!({
            "type": "assistant",
            "sessionId": "session-verify",
            "timestamp": "2026-09-12T00:00:00Z",
            "message": {"id": "message-verify", "model": "claude-fable-5", "stop_reason": "end_turn", "usage": {"input_tokens": 5, "output_tokens": 2}}
        }),
        &json!({"actuation_ref": "actuation:verify", "native_trace_ref": "trace:verify"}),
    )
    .map_err(|e| e.to_string())?;
    let observed = serde_json::to_value(&observation).map_err(|e| e.to_string())?;
    let value = run_cli(
        &["usage", "-", "--json"],
        &serde_json::to_string(&observation).map_err(|e| e.to_string())?,
    )?;
    if value["usage_ref"] != observed["usage_ref"] {
        return Err("usage read model drifted".into());
    }
    Ok(())
}

fn check_instantiation_golden() -> std::result::Result<(), String> {
    let receipt = r#"{"schema":"actuation.model-bearing/v1","actuation_ref":"actuation:verify","agency_ref":"agency:verify","world_binding_ref":"world:verify","model_relation":{"schema":"actuation.model-bearing/v1","model_ref":"model:verify","material":{"placement":"local"},"inference_surface":{"contract_ref":"contract:verify/v1"}},"access_profile":{"schema":"actuation.model-bearing/v1","inference":{"allowed":["invoke"],"denied":[]},"control":{"allowed":[],"denied":["acquire"]},"interior":{"depth":"behavioral","allowed":[],"denied":[]}},"observed_at":"2026-09-12T00:00:00Z"}"#;
    let value = run_cli(&["instantiation", "-", "--json"], receipt)?;
    if value["schema"] != json!("actuation.instantiation/v1") {
        return Err(format!("legacy model-bearing read drifted: {value}"));
    }
    Ok(())
}

fn scratch_dir(name: &str) -> std::result::Result<std::path::PathBuf, String> {
    let dir = std::env::temp_dir().join(format!("actuation-verify-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn check_durable_store_lifecycle() -> std::result::Result<(), String> {
    let dir = scratch_dir("store")?;
    let store = JsonlStreamStore::new(dir.clone()).map_err(|e| e.to_string())?;
    let opening: actuation_stream::OpenStream = serde_json::from_value(json!({
        "stream_ref": "actuation:stream:verify",
        "actuation_ref": "actuation:verify",
        "agency_ref": "agency:verify",
        "agent_session_ref": "session:verify",
        "started_at": "2026-09-12T00:00:00Z"
    }))
    .map_err(|e| e.to_string())?;
    store.open(&opening).map_err(|e| e.to_string())?;
    // The observation is produced by the same adapter path the CLI serves, so
    // the lifecycle check exercises translate → validate → record faithfully.
    let observation = actuation_adapters::usage::from_claude_code_transcript(
        &json!({
            "type": "assistant",
            "sessionId": "session-verify",
            "timestamp": "2026-09-12T00:00:00Z",
            "message": {"id": "message-verify", "model": "claude-fable-5", "stop_reason": "end_turn", "usage": {"input_tokens": 5, "output_tokens": 2}}
        }),
        &json!({"actuation_ref": "actuation:verify", "native_trace_ref": "trace:verify"}),
    )
    .map_err(|e| e.to_string())?;
    let receipt = store
        .record_usage(UsageOccurrence {
            stream_ref: StreamRef::new("actuation:stream:verify").map_err(|e| e.to_string())?,
            observation,
            identity: None,
            event_ref: None,
        })
        .map_err(|e| e.to_string())?;
    let receipt = serde_json::to_value(receipt).map_err(|e| e.to_string())?;
    if receipt["event"]["sequence"] != json!(1) {
        return Err("first usage observation must take sequence 1".into());
    }
    let replay = store
        .replay(
            &serde_json::from_value(json!("actuation:stream:verify")).map_err(|e| e.to_string())?,
            actuation_stream::PageRequest::default(),
        )
        .map_err(|e| e.to_string())?;
    if replay.events.len() != 1 {
        return Err("replay must return the recorded usage event".into());
    }
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

fn sha256_hex(raw: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn check_frozen_wire_oracle() -> std::result::Result<(), String> {
    if sha256_hex(FROZEN_STORE_01) != FROZEN_STORE_01_SHA256 {
        return Err("the embedded frozen wire store no longer matches its published sha256".into());
    }
    let dir = scratch_dir("frozen")?;
    let store = JsonlStreamStore::new(dir.clone()).map_err(|e| e.to_string())?;
    let stream_ref: StreamRef =
        serde_json::from_value(json!("stream:oracle/雪 !'()*")).map_err(|e| e.to_string())?;
    std::fs::write(store.path(&stream_ref), FROZEN_STORE_01).map_err(|e| e.to_string())?;
    let stream = store.load(&stream_ref).map_err(|e| e.to_string())?;
    let reading = serde_json::to_value(stream.read(actuation_stream::PageRequest::default()))
        .map_err(|e| e.to_string())?;
    if reading["lifecycle"]["state"] != json!("open") {
        return Err(format!("frozen store lifecycle drifted: {reading}"));
    }
    let _ = std::fs::remove_dir_all(&dir);
    Ok(())
}

fn check_harness_catalog() -> std::result::Result<(), String> {
    let catalog = actuation_adapters::NativeCatalog::bundled().map_err(|e| e.to_string())?;
    if catalog.revision() < 1 || catalog.descriptors().is_empty() {
        return Err("bundled catalog must declare targets".into());
    }
    if catalog.capability("zcode").is_none() {
        return Err("bundled catalog lost the zcode capability".into());
    }
    Ok(())
}

fn check_disclosure_digest() -> std::result::Result<(), String> {
    use crate::system::{build_system_disclosure, canonical_reading_body};
    let detection = json!({
        "schema": "actuation.harness-detection/v1",
        "catalog_revision": 6,
        "availability": "complete",
        "harnesses": [{"slug": "zcode", "harness_ref": "harness/zcode", "state": "not-installed"}]
    });
    let self_value = json!({"resolved": null, "ambiguity": false});
    let a = build_system_disclosure(&detection, &self_value, 1_000);
    let b = build_system_disclosure(&detection, &self_value, 2_000);
    if a["owner"]["reading_digest"] != b["owner"]["reading_digest"] {
        return Err("disclosure digest changed with the clock".into());
    }
    if canonical_reading_body(&a)["owner"]["reading_digest"] != Value::Null {
        return Err("canonical body must hold the digest null".into());
    }
    if a["actions"].as_array().map(Vec::len).unwrap_or(0) < 10 {
        return Err("disclosure lost its action projection".into());
    }
    Ok(())
}

fn check_receipt_names_the_suite() -> std::result::Result<(), String> {
    let names: Vec<&'static str> = suite().iter().map(|(name, _)| *name).collect();
    if names.len() < 10 {
        return Err(format!("suite shrank: {} checks", names.len()));
    }
    let unique: std::collections::HashSet<&&str> = names.iter().collect();
    if unique.len() != names.len() {
        return Err("suite check names must be unique".into());
    }
    if git_revision().is_empty() {
        return Err("revision claim must resolve".into());
    }
    Ok(())
}
