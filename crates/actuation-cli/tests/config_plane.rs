//! The owner side of the O:I configuration plane, exercised against the real
//! binary in a sandboxed HOME (never against live state).
//!
//! These tests are the C3B owner-contract proof: the contribution is a bare,
//! schema-conforming `oi.configuration-contribution/v1` document whose
//! subjects are all declared code; `config validate` answers truthfully
//! in-document; plan/apply/reset refuse with structured `oi.config-error/v1`
//! documents and non-zero exits; the authority constitution refuses as
//! `not_authorised` on its own law; and an executed idempotency key replays
//! as `no_op` naming the original receipt without re-execution.

use assert_cmd::cargo::cargo_bin;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// A sandbox HOME: the owner's receipt ledger (and anything else HOME-derived)
/// stays inside it, so no test ever touches live state.
fn sandbox() -> TempDir {
    TempDir::new().unwrap()
}

/// Run a config command against the real binary inside the sandbox. Every
/// config surface answers with a bare JSON document — success answer or
/// `oi.config-error/v1` — and its own exit status.
fn run(home: &Path, args: &[&str]) -> (i32, Value) {
    run_with_stdin(home, args, "")
}

fn run_with_stdin(home: &Path, args: &[&str], stdin: &str) -> (i32, Value) {
    let output = std::process::Command::new(cargo_bin("actuation"))
        .args(args)
        .env("HOME", home)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child
                .stdin
                .as_mut()
                .expect("piped stdin")
                .write_all(stdin.as_bytes())?;
            child.wait_with_output()
        })
        .expect("binary runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {stdout:?}"));
    (output.status.code().unwrap_or(-1), value)
}

/// Seed the owner-side receipt ledger with one executed receipt.
fn seed_receipt(home: &Path, receipt: &Value) {
    let root = home.join(".actuation");
    fs::create_dir_all(&root).unwrap();
    let mut existing = fs::read_to_string(root.join("config-receipts.jsonl")).unwrap_or_default();
    existing.push_str(&serde_json::to_string(receipt).unwrap());
    existing.push('\n');
    fs::write(root.join("config-receipts.jsonl"), existing).unwrap();
}

fn ledger_lines(home: &Path) -> usize {
    fs::read_to_string(home.join(".actuation/config-receipts.jsonl"))
        .map(|raw| raw.lines().filter(|line| !line.trim().is_empty()).count())
        .unwrap_or(0)
}

fn executed_receipt(changeset: &str, digest: Value, setting: &str, operation: &str) -> Value {
    json!({
        "schema": "oi.config-receipt/v1",
        "receipt_id": "actuation-config-receipt-1",
        "owner_ref": "actuation",
        "changeset_id": changeset,
        "plan_digest": digest,
        "setting_ref": setting,
        "scope": {"scope_kind": "machine", "scope_ref": null},
        "operation": operation,
        "outcome": "applied",
        "applied_at_unix_ms": 1,
        "native_ref": "actuation:config:receipts:1",
        "expected_effect": {"kind": "none", "summary": null, "ref": null},
        "original_receipt_id": null,
        "error": null,
    })
}

/// A well-formed plan shaped like one this owner would once have minted. The
/// owner mints none — which is exactly what the refusals below prove.
fn well_formed_plan(setting: &str) -> Value {
    json!({
        "schema": "oi.config-plan/v1",
        "plan_id": "actuation-plan-forged",
        "plan_digest": "abc123def4567890",
        "setting_ref": setting,
        "scope": {"scope_kind": "machine", "scope_ref": null},
        "changes": [{"summary": "a client-forged plan"}],
        "expected_effect": {"kind": "none", "summary": null, "ref": null},
    })
}

// ---------------------------------------------------------------------------
// The contribution document
// ---------------------------------------------------------------------------

#[test]
fn contribution_is_a_bare_conforming_document() {
    let home = sandbox();
    let (code, value) = run(home.path(), &["config-contribution", "--json"]);

    assert_eq!(code, 0);
    // Bare: no envelope anywhere. The read/operability plane separation is
    // structural, not a convention.
    assert!(
        value.get("ok").is_none(),
        "config-contribution must be bare"
    );
    assert_eq!(value["schema"], "oi.configuration-contribution/v1");
    assert_eq!(
        value["contract_revision"],
        "configuration-plane/contribution.1"
    );
    assert_eq!(value["owner"]["owner_ref"], "actuation");
    assert_eq!(value["owner"]["owner_kind"], "product");
    assert_eq!(
        value["owner"]["contribution_command"],
        json!(["actuation", "config-contribution", "--json"])
    );
    assert_eq!(value["availability"]["state"], "available");
    assert_eq!(value["operations"]["transport"], "cli/v1");
    assert_eq!(value["operations"]["validate"]["availability"], "disclosed");
    for verb in ["plan", "apply", "reset"] {
        assert_eq!(
            value["operations"][verb]["availability"], "unavailable",
            "{verb} is honestly declared unavailable"
        );
    }
    // A contribution never carries native axes: no declared/effective/active
    // (the words may appear in prose; the axes must not appear as keys).
    let text = value.to_string();
    assert!(
        !text.contains("\"declared\""),
        "no disclosure axes in a contribution"
    );
    assert!(
        !text.contains("\"effective\""),
        "no disclosure axes in a contribution"
    );
    assert!(
        !text.contains("\"mutable\""),
        "mutability is carried by writable, not by the v2 axis"
    );

    let digest = value["owner"]["reading_digest"].as_str().unwrap();
    assert_eq!(digest.len(), 64, "reading_digest is sha256 hex");

    // Sections and setting_refs map structurally onto the v2 disclosure (§17).
    let sections: Vec<&str> = value["sections"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["id"].as_str().unwrap())
        .collect();
    assert_eq!(sections, vec!["agency", "authority", "activity", "return"]);
    let mut subjects = 0;
    for section in value["sections"].as_array().unwrap() {
        for setting in section["settings"].as_array().unwrap() {
            subjects += 1;
            assert_eq!(setting["section_ref"], section["id"]);
            let reference = setting["setting_ref"].as_str().unwrap();
            let parts: Vec<&str> = reference.split(':').collect();
            assert_eq!(
                parts.len(),
                3,
                "{reference} parses into exactly three parts"
            );
            assert_eq!(parts[0], "actuation", "{reference} carries this owner");
            assert_eq!(parts[1], section["id"].as_str().unwrap());
            assert_eq!(setting["writable"], json!(false), "{reference}");
            assert_eq!(setting["profileable"], json!(false), "{reference}");
            assert_eq!(
                setting["operations"]["validate"],
                json!(true),
                "{reference}"
            );
            for verb in ["plan", "apply", "reset"] {
                assert_eq!(setting["operations"][verb], json!(false), "{reference}");
            }
            assert!(
                setting["native_ref"].is_string(),
                "{reference} names its native source"
            );
        }
    }
    assert!(
        subjects >= 5,
        "the contribution must cover the declared constitutional subjects, found {subjects}"
    );
}

/// The contribution validates against the frozen C0 JSON Schema, vendored
/// verbatim from the O:I contract checkout.
#[test]
fn contribution_validates_against_the_frozen_schema() {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/oi.configuration-contribution-v1.schema.json");
    let schema_json: Value =
        serde_json::from_str(&fs::read_to_string(&schema_path).unwrap()).unwrap();
    let schema = jsonschema::validator_for(&schema_json).unwrap();

    let home = sandbox();
    let (_, value) = run(home.path(), &["config-contribution", "--json"]);
    let errors: Vec<String> = schema.iter_errors(&value).map(|e| format!("{e}")).collect();
    assert!(errors.is_empty(), "schema violations: {errors:?}");
}

/// The digest is a function of the reading, never of the clock (07 §4.5).
#[test]
fn the_reading_digest_is_not_the_clock() {
    let home = sandbox();
    let (_, a) = run(home.path(), &["config-contribution", "--json"]);
    std::thread::sleep(std::time::Duration::from_millis(5));
    let (_, b) = run(home.path(), &["config-contribution", "--json"]);
    assert_ne!(
        a["owner"]["disclosed_at_unix_ms"],
        b["owner"]["disclosed_at_unix_ms"]
    );
    assert_eq!(a["owner"]["reading_digest"], b["owner"]["reading_digest"]);
}

// ---------------------------------------------------------------------------
// The four verbs
// ---------------------------------------------------------------------------

#[test]
fn validate_answers_truthfully_in_document() {
    let home = sandbox();
    // A well-formed value for a declared-code subject is still not settable:
    // the owner answers in-document, exit 0, with the not_writable violation.
    let (code, answer) = run(
        home.path(),
        &[
            "config",
            "validate",
            "--json",
            "--setting",
            "actuation:return:return.modes",
            "--value",
            "[\"required\"]",
        ],
    );
    assert_eq!(code, 0, "a validation answer is an answer: {answer}");
    assert_eq!(answer["schema"], "oi.config-validation/v1");
    assert_eq!(answer["valid"], json!(false));
    assert_eq!(
        answer["scope"],
        json!({"scope_kind": "machine", "scope_ref": null})
    );
    let codes: Vec<&str> = answer["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["code"].as_str().unwrap())
        .collect();
    assert!(codes.contains(&"not_writable"), "{codes:?}");

    // A wrong-shaped value adds the owner's value-contract violation.
    let (code, answer) = run(
        home.path(),
        &[
            "config",
            "validate",
            "--json",
            "--setting",
            "actuation:return:return.modes",
            "--value",
            "\"required\"",
        ],
    );
    assert_eq!(code, 0);
    let codes: Vec<&str> = answer["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["code"].as_str().unwrap())
        .collect();
    assert!(codes.contains(&"invalid_value"), "{codes:?}");
    assert!(codes.contains(&"not_writable"), "{codes:?}");

    // --value-file - reads the value from stdin.
    let (code, answer) = run_with_stdin(
        home.path(),
        &[
            "config",
            "validate",
            "--json",
            "--setting",
            "actuation:agency:agency.determination.kinds",
            "--value-file",
            "-",
        ],
        "[\"delegation\"]",
    );
    assert_eq!(code, 0, "{answer}");
    assert_eq!(
        answer["setting_ref"],
        "actuation:agency:agency.determination.kinds"
    );
}

#[test]
fn the_authority_constitution_refuses_on_its_own_law() {
    let home = sandbox();

    // plan on a constitution subject: not_authorised, whatever the value.
    let (code, error) = run(
        home.path(),
        &[
            "config",
            "plan",
            "--json",
            "--setting",
            "actuation:authority:authority.derivation.rule",
            "--value",
            "\"rewritten\"",
        ],
    );
    assert_ne!(code, 0, "authority refusal must exit non-zero");
    assert_eq!(error["schema"], "oi.config-error/v1");
    assert_eq!(error["error_code"], "not_authorised");
    assert_eq!(
        error["setting_ref"],
        "actuation:authority:authority.derivation.rule"
    );
    let message = error["message"].as_str().unwrap();
    assert!(message.contains("configure-agency"), "{message}");
    assert!(
        message.contains("discoverability is not authority"),
        "{message}"
    );

    // reset on the metagency vocabulary: the same authority law.
    let (code, error) = run(
        home.path(),
        &[
            "config",
            "reset",
            "--json",
            "--setting",
            "actuation:authority:authority.metagency.operations",
        ],
    );
    assert_ne!(code, 0);
    assert_eq!(error["error_code"], "not_authorised", "{error}");

    // apply with a well-formed forged plan: authority precedes mechanics.
    let (code, error) = run_with_stdin(
        home.path(),
        &[
            "config",
            "apply",
            "--json",
            "--plan-file",
            "-",
            "--changeset",
            "cs-auth-1",
        ],
        &serde_json::to_string(&well_formed_plan(
            "actuation:authority:authority.federation.rule",
        ))
        .unwrap(),
    );
    assert_ne!(code, 0);
    assert_eq!(error["error_code"], "not_authorised", "{error}");
}

#[test]
fn declared_subjects_refuse_writes_with_structured_errors() {
    let home = sandbox();

    let cases: Vec<(Vec<&str>, Value, &str)> = vec![
        (
            vec![
                "config",
                "plan",
                "--json",
                "--setting",
                "actuation:return:return.modes",
                "--value",
                "[\"optional\"]",
            ],
            json!("unsupported_setting"),
            "plan on a declared subject",
        ),
        (
            vec![
                "config",
                "reset",
                "--json",
                "--setting",
                "actuation:activity:stream.durable_store",
            ],
            json!("unsupported_setting"),
            "reset on a declared subject",
        ),
        (
            vec![
                "config",
                "plan",
                "--json",
                "--setting",
                "actuation:nope:not.a.thing",
                "--value",
                "1",
            ],
            json!("unsupported_setting"),
            "unknown setting",
        ),
        (
            vec![
                "config",
                "validate",
                "--json",
                "--setting",
                "actuation:return:return.modes",
                "--scope",
                "galaxy:x",
                "--value",
                "[]",
            ],
            json!("unknown_scope_kind"),
            "unknown scope kind",
        ),
        (
            vec![
                "config",
                "plan",
                "--json",
                "--setting",
                "actuation:return:return.modes",
                "--scope",
                "project:epilogos/o-i",
                "--value",
                "[]",
            ],
            json!("unsupported_scope"),
            "declared subjects are machine-scoped",
        ),
        (
            vec![
                "config",
                "validate",
                "--json",
                "--setting",
                "actuation:return:return.modes",
                "--value",
                "[broken",
            ],
            json!("invalid_value"),
            "malformed value JSON",
        ),
        (
            vec!["config", "validate", "--json", "--value", "1"],
            json!("unsupported_setting"),
            "missing --setting",
        ),
        (
            vec!["config", "apply", "--json"],
            json!("validation_failed"),
            "apply without a plan",
        ),
    ];
    for (args, expected_code, what) in cases {
        let (code, error) = run(home.path(), &args);
        assert_ne!(code, 0, "{what} must exit non-zero");
        assert_eq!(error["schema"], "oi.config-error/v1", "{what}");
        assert_eq!(error["error_code"], expected_code, "{what}: {error}");
    }

    // A well-formed forged plan under a fresh changeset: the owner never
    // minted it, and the digest was not issued here.
    let (code, error) = run_with_stdin(
        home.path(),
        &[
            "config",
            "apply",
            "--json",
            "--plan-file",
            "-",
            "--changeset",
            "cs-fresh-1",
        ],
        &serde_json::to_string(&well_formed_plan("actuation:activity:stream.durable_store"))
            .unwrap(),
    );
    assert_ne!(code, 0);
    assert_eq!(error["error_code"], "validation_failed", "{error}");
    assert_eq!(
        error["setting_ref"],
        "actuation:activity:stream.durable_store"
    );

    // A plan in a foreign schema is not interpreted.
    let mut foreign = well_formed_plan("actuation:activity:stream.durable_store");
    foreign["schema"] = json!("oi.config-plan/v9");
    let (code, error) = run_with_stdin(
        home.path(),
        &["config", "apply", "--json", "--plan-file", "-"],
        &serde_json::to_string(&foreign).unwrap(),
    );
    assert_ne!(code, 0);
    assert_eq!(error["error_code"], "unsupported_schema", "{error}");

    // A malformed changeset id can never anchor a receipt.
    let (code, error) = run_with_stdin(
        home.path(),
        &[
            "config",
            "apply",
            "--json",
            "--plan-file",
            "-",
            "--changeset",
            "not-a-changeset",
        ],
        &serde_json::to_string(&well_formed_plan("actuation:activity:stream.durable_store"))
            .unwrap(),
    );
    assert_ne!(code, 0);
    assert_eq!(error["error_code"], "validation_failed", "{error}");
}

#[test]
fn an_executed_apply_key_replays_as_no_op_without_reexecution() {
    let home = sandbox();
    let setting = "actuation:activity:stream.durable_store";
    seed_receipt(
        home.path(),
        &executed_receipt("cs-replay-1", json!("abc123def4567890"), setting, "apply"),
    );
    let plan = well_formed_plan(setting);
    let plan_path = home.path().join("plan.json");
    fs::write(&plan_path, serde_json::to_string_pretty(&plan).unwrap()).unwrap();

    // Replay under the executed key: exit 0, no_op, the original receipt
    // named, the ledger untouched.
    let (code, receipt) = run(
        home.path(),
        &[
            "config",
            "apply",
            "--json",
            "--plan-file",
            plan_path.to_str().unwrap(),
            "--changeset",
            "cs-replay-1",
        ],
    );
    assert_eq!(code, 0, "a replay is a no_op answer: {receipt}");
    assert_eq!(receipt["schema"], "oi.config-receipt/v1");
    assert_eq!(receipt["outcome"], "no_op");
    assert_eq!(receipt["operation"], "apply");
    assert_eq!(receipt["original_receipt_id"], "actuation-config-receipt-1");
    assert_ne!(receipt["receipt_id"], "actuation-config-receipt-1");
    assert_eq!(receipt["setting_ref"], setting);
    assert_eq!(receipt["plan_digest"], plan["plan_digest"]);
    assert_eq!(ledger_lines(home.path()), 1, "the replay never re-executes");

    // The same plan under a fresh changeset was never executed: refused.
    let (code, error) = run(
        home.path(),
        &[
            "config",
            "apply",
            "--json",
            "--plan-file",
            plan_path.to_str().unwrap(),
            "--changeset",
            "cs-fresh-2",
        ],
    );
    assert_ne!(code, 0);
    assert_eq!(error["error_code"], "validation_failed", "{error}");
    assert_eq!(ledger_lines(home.path()), 1, "a refusal records nothing");
}

#[test]
fn an_executed_reset_key_replays_and_a_fresh_reset_refuses() {
    let home = sandbox();
    let setting = "actuation:return:return.modes";
    seed_receipt(
        home.path(),
        &executed_receipt("cs-reset-1", Value::Null, setting, "reset"),
    );

    let (code, receipt) = run(
        home.path(),
        &[
            "config",
            "reset",
            "--json",
            "--setting",
            setting,
            "--changeset",
            "cs-reset-1",
        ],
    );
    assert_eq!(code, 0, "{receipt}");
    assert_eq!(receipt["outcome"], "no_op");
    assert_eq!(receipt["operation"], "reset");
    assert_eq!(receipt["original_receipt_id"], "actuation-config-receipt-1");

    let (code, error) = run(
        home.path(),
        &[
            "config",
            "reset",
            "--json",
            "--setting",
            setting,
            "--changeset",
            "cs-reset-2",
        ],
    );
    assert_ne!(code, 0);
    assert_eq!(error["error_code"], "unsupported_setting", "{error}");
}

#[test]
fn plan_files_can_arrive_by_stdin_and_the_refusals_still_structure() {
    let home = sandbox();
    // --plan-file - with a valid plan and no changeset: the grammar allows
    // the optional changeset; the owner still refuses (never minted a plan).
    let (code, error) = run_with_stdin(
        home.path(),
        &["config", "apply", "--json", "--plan-file", "-"],
        &serde_json::to_string(&well_formed_plan("actuation:return:return.modes")).unwrap(),
    );
    assert_ne!(code, 0);
    assert_eq!(error["schema"], "oi.config-error/v1");
    assert_eq!(error["error_code"], "validation_failed", "{error}");

    // --plan-file - with stdin that is not JSON.
    let (code, error) = run_with_stdin(
        home.path(),
        &["config", "apply", "--json", "--plan-file", "-"],
        "not json at all",
    );
    assert_ne!(code, 0);
    assert_eq!(error["error_code"], "validation_failed", "{error}");
    assert!(
        error["message"]
            .as_str()
            .unwrap()
            .contains("not valid JSON"),
        "{error}"
    );
}
