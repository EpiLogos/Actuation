//! `actuation stream list` through the real binary, on a real store: the
//! streams one Agent session keeps, for the routed `stream replay` and
//! `usage` readings to project.
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

#[test]
fn a_sessions_streams_are_listed_from_the_store() {
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
    let (ok, all, stderr) = actuation(&["stream", "list", "--store", store, "--json"], None);
    assert!(ok, "{stderr}");
    assert_eq!(all["schema"], "actuation.stream-listing/v1");
    assert_eq!(all["streams"].as_array().unwrap().len(), 2);
    assert_eq!(all["invalid"], json!([]));

    let (ok, one, _) = actuation(
        &[
            "stream",
            "list",
            "--session",
            "session:b",
            "--store",
            store,
            "--json",
        ],
        None,
    );
    assert!(ok);
    assert_eq!(one["agent_session_ref"], "session:b");
    let streams = one["streams"].as_array().unwrap();
    assert_eq!(streams.len(), 1);
    assert_eq!(streams[0]["stream_ref"], "stream:two");
    assert_eq!(streams[0]["lifecycle"]["state"], "open");
    assert_eq!(streams[0]["event_count"], 0);

    let (ok, _, stderr) = actuation(&["stream", "list", "stream:one", "--json"], None);
    assert!(!ok);
    assert!(
        stderr.contains("stream list takes only --session and --store"),
        "{stderr}"
    );
    let (ok, absent, _) = actuation(
        &[
            "stream",
            "list",
            "--store",
            dir.path().join("none").to_str().unwrap(),
            "--json",
        ],
        None,
    );
    assert!(ok);
    assert_eq!(absent["streams"], json!([]));
}
