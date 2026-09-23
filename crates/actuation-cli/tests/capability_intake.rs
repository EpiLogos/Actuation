//! The CLI faces of the capability-descriptor intake route (O:I #113 A1):
//! `harness capability validate` answers with named diagnostics and exit
//! codes; `config-contribution capability` receives a gap-filling descriptor
//! with a receipt the owner lands into the bundled catalog.
use actuation_adapters::NativeCatalog;
use actuation_cli::dispatch::{execute, Output};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
const SPECIMEN: &str = "fixtures/capability-contributions/gemini.harness-capability.json";

fn specimen_path() -> String {
    format!("{ROOT}/{SPECIMEN}")
}

fn specimen_bytes() -> String {
    std::fs::read_to_string(specimen_path()).expect("the gemini specimen ships as a fixture")
}

fn run(argv: &[&str], stdin: &str) -> Output {
    let args: Vec<String> = argv.iter().map(|s| s.to_string()).collect();
    execute(&args, stdin).expect("the intake routes execute or refuse, never panic")
}

fn body(output: &Output) -> Value {
    serde_json::from_str(&output.stdout).expect("intake answers are JSON documents")
}

#[test]
fn validate_answers_exit_zero_with_named_passing_checks_on_the_specimen() {
    let output = run(
        &[
            "harness",
            "capability",
            "validate",
            &specimen_path(),
            "--json",
        ],
        "",
    );
    assert_eq!(output.code, 0, "{}", output.stdout);
    let document = body(&output);
    assert_eq!(
        document["schema"],
        json!("actuation.harness-capability-validation/v1")
    );
    assert_eq!(document["valid"], json!(true));
    assert_eq!(document["harness_slug"], json!("gemini"));
    let checks: Vec<&str> = document["checks"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|c| c["check"].as_str())
        .collect();
    assert_eq!(
        checks,
        vec!["schema-admission", "slug-alignment", "coverage-closure"]
    );
}

#[test]
fn validate_answers_exit_one_with_the_failing_check_named() {
    let dir = std::env::temp_dir().join(format!("actuation-intake-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("fabricated.json");
    std::fs::write(
        &path,
        r#"{"schema":"actuation.harness-capability/v1","document":"capability","harness_slug":"gemini","native_events":[{"event":"teleport","native_name":"Teleport","transport":"hooks-json-file","can_block":false,"context_channel":"none"}],"injection_channel":{"kind":"none","mechanism":"none"},"blocking_semantics":{"kind":"none"},"wake_capability":{"kind":"none"},"install_seam":{"config_path":"x","format":"json","entry_shape":"x","ownership_marker":"x","preserves_foreign_entries":true},"uninstall_seam":{"config_path":"x","format":"json","entry_shape":"x","ownership_marker":"x","preserves_foreign_entries":true},"provenance":{"authored_by":"negative fixture","source_refs":["fixture:fabricated"]}}"#,
    )
    .unwrap();
    let output = run(
        &[
            "harness",
            "capability",
            "validate",
            path.to_str().unwrap(),
            "--json",
        ],
        "",
    );
    assert_eq!(
        output.code, 1,
        "a refused document exits 1: {}",
        output.stdout
    );
    let document = body(&output);
    assert_eq!(document["valid"], json!(false));
    let failed = document["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["result"] == json!("fail"))
        .expect("the failing check is named");
    assert_eq!(failed["check"], json!("schema-admission"));
    assert!(
        failed["detail"]
            .as_str()
            .unwrap()
            .contains("expected one of"),
        "{}",
        failed["detail"]
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn the_contribution_face_mints_a_receipt_the_owner_can_land() {
    let output = run(
        &[
            "config-contribution",
            "capability",
            &specimen_path(),
            "--json",
        ],
        "",
    );
    assert_eq!(output.code, 0, "{}", output.stdout);
    let receipt = body(&output);
    assert_eq!(
        receipt["schema"],
        json!("actuation.capability-contribution/v1")
    );
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
    // The source digest is the sha256 of the contributed bytes.
    let mut digest = Sha256::new();
    digest.update(specimen_bytes().as_bytes());
    assert_eq!(
        receipt["source"]["sha256"],
        json!(format!("{:x}", digest.finalize()))
    );
    // The landing edit names the owner's merge into the bundled catalog.
    let catalog = NativeCatalog::bundled().unwrap();
    assert_eq!(
        receipt["landing"]["current_revision"],
        json!(catalog.revision())
    );
    assert_eq!(
        receipt["landing"]["landed_revision"],
        json!(catalog.revision() + 1)
    );
    // Provenance travels with the descriptor unchanged.
    assert!(
        receipt["capability"]["provenance"]["authored_by"]
            .as_str()
            .unwrap()
            .contains("oi65-l6-author"),
        "{}",
        receipt["capability"]["provenance"]["authored_by"]
    );
}

#[test]
fn the_contribution_face_refuses_a_descriptor_that_shadows_a_declared_capability() {
    let catalog = NativeCatalog::bundled().unwrap();
    let declared = catalog.capability("pi").unwrap();
    let raw = serde_json::to_string(declared.as_value()).unwrap();
    let output = run(&["config-contribution", "capability", "-", "--json"], &raw);
    assert_eq!(output.code, 1, "{}", output.stdout);
    let document = body(&output);
    assert_eq!(
        document["schema"],
        json!("actuation.harness-capability-validation/v1")
    );
    assert_eq!(document["valid"], json!(false));
    let failed = document["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["result"] == json!("fail"))
        .unwrap();
    assert_eq!(failed["check"], json!("coverage-closure"));
    assert!(
        failed["detail"].as_str().unwrap().contains("owner edit"),
        "{}",
        failed["detail"]
    );
}

#[test]
fn validate_reads_stdin_when_the_positional_is_the_stdin_marker() {
    let output = run(
        &["harness", "capability", "validate", "-", "--json"],
        &specimen_bytes(),
    );
    assert_eq!(output.code, 0, "{}", output.stdout);
    assert_eq!(body(&output)["valid"], json!(true));
}

#[test]
fn a_non_json_document_is_a_handler_refusal_not_a_validation_answer() {
    let args: Vec<String> = ["harness", "capability", "validate", "-"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let error = execute(&args, "not json at all").expect_err("non-JSON input is a refusal");
    assert!(error.to_string().contains("not valid JSON"), "{error}");
}

#[test]
fn the_human_render_names_the_verdict_and_the_checks() {
    let output = run(
        &["harness", "capability", "validate", "-"],
        &specimen_bytes(),
    );
    assert_eq!(output.code, 0);
    assert!(output.stdout.contains("valid"), "{}", output.stdout);
    assert!(
        output.stdout.contains("schema-admission"),
        "{}",
        output.stdout
    );
}
