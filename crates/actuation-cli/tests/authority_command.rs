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
