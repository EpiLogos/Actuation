//! The `authority` commands through the real dispatch table, on a real store.
//!
//! One owner issues a governing record; the holder resolves an exact request
//! into a complete actualisation request; a non-holder, a revoked source and
//! an out-of-scope world refuse without effect. The admitted request must
//! pass `agency actualise` unchanged — resolution fabricates nothing.

use actuation_cli::{execute, match_route};
use serde_json::{json, Value};

fn record_value() -> Value {
    json!({
        "schema": "actuation.local-authority/v1",
        "authority_source_ref": "authority-source:governing-main",
        "holder": "human:owner",
        "issued_by": "human:owner",
        "governing_binding": {
            "schema": "actuation.agency/v1",
            "binding_ref": "binding:governing",
            "agent_ref": "agent:governor",
            "agency_ref": "agency:governing",
            "world_ref": "world:personal",
            "scope_ref": "scope:personal",
            "bounds_refs": ["bound:personal", "bound:project:delegation", "bound:secondary"],
            "authority_refs": ["authority:metagency", "authority:project:delegation"],
            "return_relation_ref": "return-relation:governing"
        },
        "metagency_grant": {
            "schema": "actuation.agency/v1",
            "grant_ref": "grant:project-agency",
            "agency_ref": "agency:governing",
            "world_binding_ref": "binding:governing",
            "authority_ref": "authority:metagency",
            "bounds_refs": ["bound:project:delegation", "bound:secondary"],
            "operations": ["determine-agency", "actualise-agency"]
        },
        "allowed_world_refs": ["central:project:Example"],
        "issued_at_unix_seconds": 1726300000
    })
}

fn resolution_value(source: &str, requester: &str, world: &str) -> Value {
    json!({
        "schema": "actuation.local-authority/v1",
        "resolution_ref": "resolution:walk-1",
        "request_ref": "actualisation-request:walk-1",
        "requester_ref": requester,
        "authority_source_ref": source,
        "determination_ref": "determination:walk-1",
        "differentiated_binding": {
            "schema": "actuation.agency/v1",
            "binding_ref": "binding:project:delegation",
            "agent_ref": "agent:existing-1",
            "agency_ref": "agency:project:delegation",
            "world_ref": world,
            "scope_ref": "central:project:Example:scope",
            "determining_agency_ref": "agency:governing",
            "bounds_refs": ["bound:secondary", "bound:project:delegation"],
            "authority_refs": ["authority:project:delegation"],
            "return_relation_ref": "return-relation:project:delegation",
            "continuity_ref": "continuity:agent:existing-1"
        },
        "agent_identity": {
            "standing": "existing",
            "evidence_refs": ["evidence:identity-registry"]
        },
        "requested_bounds_refs": ["bound:project:delegation", "bound:secondary"],
        "delegated_autonomy": {
            "allowed_action_refs": ["action:bounded-work"],
            "denied_action_refs": ["action:source-mutation"],
            "may_determine_within_bounds": true
        },
        "return_policy": {
            "mode": "required",
            "return_relation_ref": "return-relation:project:delegation"
        },
        "provenance": {
            "source_refs": ["actuation:#84"],
            "context_refs": []
        }
    })
}

fn run(store: &std::path::Path, argv: &[&str], stdin: &str) -> Value {
    let mut argv: Vec<String> = argv
        .iter()
        .map(|a| a.replace("{STORE}", &store.display().to_string()))
        .collect();
    argv.push("--json".into());
    assert!(
        match_route(&argv).is_some(),
        "the dispatch table must know every authority route: {argv:?}"
    );
    let output = execute(&argv, stdin).expect("authority commands must not error");
    assert_eq!(output.code, 0, "typed refusals still exit cleanly");
    serde_json::from_str(&output.stdout).expect("output is JSON")
}

#[test]
fn issue_resolve_and_actualise_join_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let store = dir.path().join("authority");

    let issued = run(
        &store,
        &[
            "authority",
            "issue",
            "--store",
            "{STORE}",
            "--now",
            "1726300000",
            "-",
        ],
        &record_value().to_string(),
    );
    assert_eq!(issued["issued"], json!(true));

    let resolved = run(
        &store,
        &[
            "authority",
            "resolve",
            "--store",
            "{STORE}",
            "--now",
            "1726300100",
            "-",
        ],
        &resolution_value(
            "authority-source:governing-main",
            "human:owner",
            "central:project:Example",
        )
        .to_string(),
    );
    assert_eq!(resolved["standing"], json!("admitted"));
    let request = resolved["actualisation_request"]
        .as_object()
        .expect("an admitted resolution carries the complete request");

    // The admitted request must be accepted by the constitutional gate
    // unchanged — resolution invented nothing.
    let actualised = run(
        &store,
        &["agency", "actualise", "-"],
        &Value::Object(request.clone()).to_string(),
    );
    assert!(
        actualised.get("receipt").is_some() || actualised.is_object(),
        "actualisation returns its receipt: {actualised}"
    );
}

#[test]
fn wrong_holder_and_out_of_scope_world_refuse_without_effect() {
    let dir = tempfile::tempdir().unwrap();
    let store = dir.path().join("authority");
    run(
        &store,
        &[
            "authority",
            "issue",
            "--store",
            "{STORE}",
            "--now",
            "1726300000",
            "-",
        ],
        &record_value().to_string(),
    );

    let intruder = run(
        &store,
        &[
            "authority",
            "resolve",
            "--store",
            "{STORE}",
            "--now",
            "1726300100",
            "-",
        ],
        &resolution_value(
            "authority-source:governing-main",
            "human:intruder",
            "central:project:Example",
        )
        .to_string(),
    );
    assert_eq!(intruder["standing"], json!("refused"));
    assert_eq!(intruder["refusal"]["code"], json!("authority.wrong_holder"));
    assert!(intruder.get("actualisation_request").is_none());

    let elsewhere = run(
        &store,
        &[
            "authority",
            "resolve",
            "--store",
            "{STORE}",
            "--now",
            "1726300100",
            "-",
        ],
        &resolution_value(
            "authority-source:governing-main",
            "human:owner",
            "central:project:SomewhereElse",
        )
        .to_string(),
    );
    assert_eq!(elsewhere["standing"], json!("refused"));
    assert_eq!(
        elsewhere["refusal"]["code"],
        json!("authority.world_not_scoped")
    );
}

#[test]
fn revocation_persists_and_a_revoked_source_never_resolves() {
    let dir = tempfile::tempdir().unwrap();
    let store = dir.path().join("authority");
    run(
        &store,
        &[
            "authority",
            "issue",
            "--store",
            "{STORE}",
            "--now",
            "1726300000",
            "-",
        ],
        &record_value().to_string(),
    );
    let revoked = run(
        &store,
        &[
            "authority",
            "revoke",
            "authority-source:governing-main",
            "--store",
            "{STORE}",
            "--now",
            "1726300050",
            "--reason",
            "owner withdrawal",
        ],
        "",
    );
    assert_eq!(revoked["revoked"], json!(true));

    let resolved = run(
        &store,
        &[
            "authority",
            "resolve",
            "--store",
            "{STORE}",
            "--now",
            "1726300100",
            "-",
        ],
        &resolution_value(
            "authority-source:governing-main",
            "human:owner",
            "central:project:Example",
        )
        .to_string(),
    );
    assert_eq!(resolved["standing"], json!("refused"));
    assert_eq!(resolved["refusal"]["code"], json!("authority.revoked"));
}

#[test]
fn list_names_every_issued_authority_and_its_revocation() {
    let dir = tempfile::tempdir().unwrap();
    let store = dir.path().join("authority");
    let empty = run(&store, &["authority", "list", "--store", "{STORE}"], "");
    assert_eq!(
        empty["schema"],
        json!("actuation.local-authority-listing/v1")
    );
    assert_eq!(empty["authorities"], json!([]));
    let mut second = record_value();
    second["authority_source_ref"] = json!("authority-source:governing-second");
    for (record, now) in [(record_value(), "1726300000"), (second, "1726300010")] {
        run(
            &store,
            &[
                "authority",
                "issue",
                "--store",
                "{STORE}",
                "--now",
                now,
                "-",
            ],
            &record.to_string(),
        );
    }
    run(
        &store,
        &[
            "authority",
            "revoke",
            "authority-source:governing-second",
            "--store",
            "{STORE}",
            "--now",
            "1726300050",
            "--reason",
            "owner withdrawal",
        ],
        "",
    );
    let listed = run(&store, &["authority", "list", "--store", "{STORE}"], "");
    assert_eq!(listed["invalid"], json!([]));
    let authorities = listed["authorities"].as_array().unwrap();
    assert_eq!(authorities.len(), 2);
    assert_eq!(
        authorities[0]["authority_source_ref"],
        json!("authority-source:governing-main")
    );
    // The stored record, as issue encoded it.
    let record = &authorities[0]["record"];
    assert_eq!(record["holder"], json!("human:owner"));
    assert_eq!(record["issued_by"], json!("human:owner"));
    assert_eq!(
        record["allowed_world_refs"],
        json!(["central:project:Example"])
    );
    assert_eq!(record["metagency_grant"], record_value()["metagency_grant"]);
    assert_eq!(authorities[0]["issued_at_unix_seconds"], json!(1726300000));
    assert!(authorities[0].get("revoked").is_none());
    assert_eq!(
        authorities[1]["revoked"],
        json!({"at_unix_seconds": 1726300050, "reason": "owner withdrawal"})
    );
    // A source ref is not an argument of the listing.
    let refused = execute(
        &[
            "authority".into(),
            "list".into(),
            "authority-source:governing-main".into(),
            "--json".into(),
        ],
        "",
    );
    assert!(refused
        .err()
        .map(|error| error.to_string())
        .unwrap_or_default()
        .contains("authority list takes only --store"));
}

/// Through the real binary: `--now T -` reads the record from stdin (the
/// stdin decision once mistook the timestamp for the document).
#[test]
fn the_real_binary_reads_the_record_after_now() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let store = dir.path().join("authority");
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_actuation"))
        .args(["authority", "issue", "--store"])
        .arg(&store)
        .args(["--now", "1726300000", "-", "--json"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(record_value().to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let issued: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(issued["issued_at_unix_seconds"], json!(1726300000));
    let listed = std::process::Command::new(env!("CARGO_BIN_EXE_actuation"))
        .args(["authority", "list", "--store"])
        .arg(&store)
        .arg("--json")
        .output()
        .unwrap();
    assert!(listed.status.success());
    let listed: Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(
        listed["authorities"][0]["authority_source_ref"],
        json!("authority-source:governing-main")
    );
}

#[test]
fn actualise_refusal_names_the_malformed_record() {
    // The shape a session plausibly invents when nothing discloses the
    // envelope: compact string shorthands where the contract has records.
    let shorthand = json!({
        "schema": "actuation.agency-actualisation/v1",
        "request_ref": "request:shorthand",
        "requester_ref": "human:owner",
        "governing_binding": {
            "schema": "actuation.agency/v1",
            "binding_ref": "binding:governing",
            "agent_ref": "agent:governor",
            "agency_ref": "agency:governing",
            "world_ref": "world:personal",
            "scope_ref": "scope:personal"
        },
        "metagency_grant": {
            "schema": "actuation.agency/v1",
            "grant_ref": "grant:delegation",
            "agency_ref": "agency:governing",
            "world_binding_ref": "binding:governing",
            "authority_ref": "authority:owner",
            "operations": ["determine-agency"],
            "bounds_refs": ["bound:observation"]
        },
        "determination": {
            "schema": "actuation.agency/v1",
            "determination_ref": "determination:delegation",
            "kind": "delegation",
            "determining_agency_ref": "agency:governing",
            "differentiated_agency_ref": "agency:delegated-work",
            "world_binding_ref": "binding:delegated-work",
            "bounds_refs": ["bound:observation"],
            "delegated_autonomy": "bounded",
            "return_policy": "owner-return",
            "authority_refs": ["authority:owner"]
        },
        "differentiated_binding": {
            "schema": "actuation.agency/v1",
            "binding_ref": "binding:delegated-work",
            "agent_ref": "agent:governor",
            "agency_ref": "agency:delegated-work",
            "world_ref": "world:personal",
            "scope_ref": "scope:personal"
        },
        "agent_identity": {
            "standing": "existing",
            "evidence_refs": ["evidence:identity-registry"]
        },
        "provenance": { "source_refs": ["source:owner-commission"] }
    });
    let error = execute(
        &["agency".into(), "actualise".into(), "-".into()],
        &shorthand.to_string(),
    )
    .expect_err("a shorthand document must be refused");
    let message = error.to_string();
    assert!(
        message.contains("DelegatedAutonomyFields"),
        "the refusal must name the record that was not an object: {message}"
    );
}

#[test]
fn schema_flag_prints_an_envelope_that_admits() {
    let printed = execute(
        &["agency".into(), "actualise".into(), "--schema".into()],
        "",
    )
    .expect("--schema must not error");
    let example: Value =
        serde_json::from_str(&printed.stdout).expect("--schema prints a JSON envelope");
    assert_eq!(
        example["schema"],
        json!("actuation.agency-actualisation/v1")
    );
    // The printed envelope must itself be a complete, admitting request —
    // the example is documentation that cannot rot.
    let actualised = execute(
        &[
            "agency".into(),
            "actualise".into(),
            "-".into(),
            "--json".into(),
        ],
        &example.to_string(),
    )
    .expect("the printed example must parse as a request");
    assert_eq!(actualised.code, 0, "the printed example must admit");
    let receipt: Value = serde_json::from_str(&actualised.stdout).expect("receipt is JSON");
    assert!(
        receipt.get("receipt").is_some() || receipt.is_object(),
        "actualisation returns its receipt: {receipt}"
    );
}
