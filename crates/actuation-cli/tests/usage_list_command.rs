//! `actuation usage list` and `actuation realised continuity` through the real
//! binary: the usage records one Agent session's streams keep, with their
//! totals, for the routed `usage` reading and the Ledger; and the continuity
//! of two realised receipts, which until now only the library could compute.
use serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Stdio};

fn actuation(args: &[&str], stdin: Option<&Value>) -> (bool, Value, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_actuation"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    if let Some(body) = stdin {
        input.write_all(body.to_string().as_bytes()).unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    (
        output.status.success(),
        serde_json::from_str(&stdout).unwrap_or(Value::Null),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// One normalized usage observation for `stream usage`'s `observation`
/// adapter: tokens counted, cost as given.
fn observation(stream: &str, session: &str, n: u32, cost: Value) -> Value {
    json!({
        "adapter": "observation",
        "stream_ref": stream,
        "event_ref": format!("event:{stream}:{n}"),
        "native_event": {
            "schema": "actuation.model-usage/v1",
            "usage_ref": format!("model-usage:{stream}:{n}"),
            "actuation_ref": "act:direct",
            "invocation_ref": format!("invocation:{n}"),
            "correlation": {"agency_ref": "agency:direct", "agent_session_ref": session,
                "harness_ref": "harness:claude-code"},
            "provider": {"standing": "not-reported"},
            "model": {"standing": "normalized-from-native", "name": "claude-fable-5"},
            "tokens": {"standing": "normalized-from-native", "input": 10 * n, "output": n},
            "cache": {"standing": "not-reported"},
            "timing": {"latency": {"standing": "not-reported"}},
            "cost": cost,
            "outcome": {"state": "completed", "standing": "normalized-from-native"},
            "provenance": {"reporter_ref": "harness:claude-code",
                "native_event_ref": format!("claude-code:message:{n}"),
                "native_schema": "claude-code.transcript/assistant-message",
                "observed_at": "2026-09-08T20:00:00Z",
                "raw_evidence_refs": [format!("trace:line-{n}")]}
        }
    })
}

#[test]
fn a_sessions_usage_records_and_their_totals_come_from_the_store() {
    let dir = tempfile::tempdir().unwrap();
    let store = dir.path().join("streams");
    let store = store.to_str().unwrap();
    for (stream, session) in [("stream:one", "session:a"), ("stream:two", "session:b")] {
        let (ok, _, stderr) = actuation(
            &["stream", "open", "--store", store, "-", "--json"],
            Some(&json!({"stream_ref": stream, "actuation_ref": "act:direct",
                "agency_ref": "agency:direct", "agent_session_ref": session})),
        );
        assert!(ok, "{stderr}");
    }
    let usd = json!({"standing": "provider-reported", "amount": 0.5, "currency": "USD"});
    for (stream, session, n, cost) in [
        ("stream:one", "session:a", 1, usd.clone()),
        (
            "stream:one",
            "session:a",
            2,
            json!({"standing": "not-reported"}),
        ),
        ("stream:two", "session:b", 3, usd),
    ] {
        let (ok, _, stderr) = actuation(
            &["stream", "usage", "--store", store, "-", "--json"],
            Some(&observation(stream, session, n, cost)),
        );
        assert!(ok, "{stderr}");
    }

    let (ok, a, stderr) = actuation(
        &[
            "usage",
            "list",
            "--session",
            "session:a",
            "--store",
            store,
            "--json",
        ],
        None,
    );
    assert!(ok, "{stderr}");
    assert_eq!(a["schema"], "actuation.usage-listing/v1");
    assert_eq!(a["agent_session_ref"], "session:a");
    assert_eq!(a["records"].as_array().unwrap().len(), 2);
    assert_eq!(a["records"][1]["sequence"], 2);
    assert_eq!(
        a["totals"],
        json!({"records": 2, "tokens_input": {"sum": 30, "records": 2},
            "tokens_output": {"sum": 3, "records": 2}, "tokens_not_reported": 0,
            "cost": [{"currency": "USD", "amount": 0.5, "records": 1,
                "standings": ["provider-reported"]}],
            "cost_not_reported": 1})
    );
    // A listed record is the one the routed `usage` reading takes.
    let (ok, read, stderr) = actuation(&["usage", "-", "--json"], Some(&a["records"][0]["usage"]));
    assert!(ok, "{stderr}");
    assert_eq!(read, a["records"][0]["usage"]);

    let (ok, all, _) = actuation(&["usage", "list", "--store", store, "--json"], None);
    assert!(ok);
    assert_eq!(all["totals"]["records"], 3);
    assert!(all.get("agent_session_ref").is_none());

    let (ok, _, stderr) = actuation(&["usage", "list", "stream:one", "--json"], None);
    assert!(!ok);
    assert!(
        stderr.contains("usage list takes only --session and --store"),
        "{stderr}"
    );
    let (ok, absent, _) = actuation(
        &[
            "usage",
            "list",
            "--store",
            dir.path().join("none").to_str().unwrap(),
            "--json",
        ],
        None,
    );
    assert!(ok);
    assert_eq!(absent["records"], json!([]));
    assert_eq!(absent["totals"]["records"], 0);
    assert!(
        !dir.path().join("none").exists(),
        "a listing creates nothing"
    );
}

fn realised(body: Value) -> Value {
    json!({"schema": "actuation.realised/v1", "realised_ref": "realised:direct",
        "actuation_ref": "actuation:direct", "agent_ref": "agent:direct", "agency_ref": "agency:direct",
        "world_binding_ref": "binding:direct", "loop": {"recurrence": "event-driven", "acting": true},
        "observation": {"state": "observed", "evidence_refs": ["evidence:driver"]}, "body": body})
}

#[test]
fn continuity_names_a_changed_body_under_the_same_identity() {
    let previous = realised(
        json!({"harness_ref": "harness:claude-code", "session_ref": "session:1",
        "model_condition_ref": "model:1"}),
    );
    let mut next = realised(
        json!({"harness_ref": "harness:codex", "session_ref": "session:1",
        "model_condition_ref": "model:1"}),
    );
    next["realised_ref"] = json!("realised:later");
    let (ok, reading, stderr) = actuation(
        &["realised", "continuity", "-", "--json"],
        Some(&json!({"previous": previous, "next": next})),
    );
    assert!(ok, "{stderr}");
    assert_eq!(reading["schema"], "actuation.realised-continuity/v1");
    assert_eq!(reading["previous_realised_ref"], "realised:direct");
    assert_eq!(reading["next_realised_ref"], "realised:later");
    assert_eq!(
        reading["delta"],
        json!({"same_agent": true, "same_agency": true, "same_world_binding": true,
            "same_actuation": true, "harness_changed": true, "session_changed": false,
            "process_changed": false, "model_condition_changed": false,
            "material_binding_changed": false, "recurrence_changed": false})
    );

    let mut other = next.clone();
    other["agent_ref"] = json!("agent:other");
    let (ok, reading, _) = actuation(
        &["realised", "continuity", "--json"],
        Some(&json!({"previous": previous, "next": other})),
    );
    assert!(ok);
    assert_eq!(reading["delta"]["same_agent"], false);

    let (ok, _, stderr) = actuation(
        &["realised", "continuity", "-", "--json"],
        Some(&json!({"previous": previous})),
    );
    assert!(!ok);
    assert!(
        stderr.contains("invalid realised continuity input"),
        "{stderr}"
    );
}
