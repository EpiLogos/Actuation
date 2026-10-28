//! A comparison over a session's own task, with its agent as the body (O-I
//! #795; the O-I manual §19.1, The Launch panel: Repeat's Variable, Levels,
//! Repeats and Stop when). Every run here goes through the public research
//! application, the real RPC transport and a scripted agent process standing
//! at the process boundary; nothing is mocked inside Actuation.
use actuation_research::application::invoke;
use serde_json::{json, Value};
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

/// A scripted agent body speaking agent-jsonl: it reads back what it was
/// given and acts in its World, which is its cwd. At effort "high" it gets the
/// answer at once; at any other level it needs a second turn. With
/// ECHO_LEVEL=0 it is a body that ignores the level it was given.
const AGENT: &str = r#"#!/bin/sh
n=0
turns=0
level=null
model=
while IFS= read -r line; do
  n=$((n+1))
  id="actuation-prime-$n"
  case "$line" in
    *'"type":"preflight"'*)
      got=$(printf '%s' "$line" | sed -n 's/.*"level":\({[^}]*}\).*/\1/p')
      [ -n "$got" ] && level=$got
      model=$(printf '%s' "$line" | sed -n 's/.*"model":"\([^"]*\)".*/\1/p')
      echoed=$level
      [ "$ECHO_LEVEL" = "0" ] && echoed=null
      printf '{"type":"response","id":"%s","command":"preflight","success":true,"data":{"ready":true,"provider":"fixture","model":"%s","agent":"agent:session/s-795","level":%s}}\n' "$id" "$model" "$echoed"
      ;;
    *'"type":"turn"'*)
      turns=$((turns+1))
      case "$level" in
        *'"high"'*) answer=42 ;;
        *) if [ "$turns" -ge 2 ]; then answer=42; else answer=41; fi ;;
      esac
      printf '%s\n' "$answer" > answer.txt
      printf '{"type":"response","id":"%s","command":"turn","success":true,"data":{"status":"completed","output":"answer.txt now says %s","usage":{"total_tokens":5}}}\n' "$id" "$answer"
      ;;
    *'"type":"finalize"'*)
      printf '{"type":"response","id":"%s","command":"finalize","success":true,"data":{"turns":%s}}\n' "$id" "$turns"
      ;;
    *)
      printf '{"type":"response","id":"%s","success":false,"data":{}}\n' "$id"
      ;;
  esac
done
"#;

/// A one-shot model body that never says it is done: it writes 41, then 42,
/// then keeps reading. Its turn count lives beside it, outside the World.
const RESTLESS_MODEL: &str = r#"#!/bin/sh
cat >/dev/null
n=$(cat count 2>/dev/null || echo 0)
n=$((n+1))
echo "$n" > count
case "$n" in
  1) printf '{"content":"first try","capabilityCalls":[{"name":"write_file","args":{"path":"answer.txt","content":"41\\n"}}]}' ;;
  2) printf '{"content":"second try","capabilityCalls":[{"name":"write_file","args":{"path":"answer.txt","content":"42\\n"}}]}' ;;
  *) printf '{"content":"checking","capabilityCalls":[{"name":"read_file","args":{"path":"answer.txt"}}]}' ;;
esac
"#;

fn script(dir: &Path, name: &str, text: &str) -> String {
    let path = dir.join(name);
    fs::write(&path, text).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path.to_string_lossy().into_owned()
}

/// The session's own task: its id is the session's task ref, its prompt the
/// task's intent, its success conditions the Intent panel's, and its test the
/// check a Test pass reads.
fn own_task() -> Value {
    json!({
        "id": "task:session/s-795",
        "prompt": "Make answer.txt say 42.",
        "successConditions": ["answer.txt says 42"],
        "starting_workspace": {"answer.txt": "0\n"},
        "test": {"program": "/usr/bin/grep", "args": ["-qx", "42", "answer.txt"],
            "timeout_ms": 10000, "output_limit": 65536}
    })
}

fn agent_body(program: &str, environment: Value) -> Value {
    json!({
        "protocol": "agent-jsonl",
        "fixture_provider": true,
        "session_timeout_ms": 60000,
        "configuration": {"provider": "fixture", "model": "m-1", "agent": "agent:session/s-795"},
        "process": {"program": program, "args": [], "cwd": "/tmp",
            "environment": environment, "timeout_ms": 60000, "output_limit": 1048576}
    })
}

fn path_only() -> Value {
    json!({"PATH": "/usr/bin:/bin"})
}

fn comparison(root: &Path, trace: &str, body: Value, extra: Value) -> Value {
    let mut request = json!({
        "operation": "comparison.run",
        "trace_ref": trace,
        "root": root.to_string_lossy(),
        "repetitions": 1,
        "basis": {"host_id": "ordinary", "host_revision": "rev-1", "benchmark_revision": "own-task",
            "runner_revision": "run-1", "review_contract_revision": "rc-1", "network_policy_digest": "np"},
        "run": {"task": own_task(), "max_steps": 4, "max_calls": 16, "body": body}
    });
    for (key, value) in extra.as_object().unwrap() {
        request[key] = value.clone();
    }
    request
}

fn takes(manifest: &Value) -> &Vec<Value> {
    manifest["records"].as_array().expect("records")
}

#[test]
fn a_repeat_over_a_sessions_own_task_runs_with_its_agent_as_the_body() {
    let bin = tempfile::tempdir().unwrap();
    let agent = script(bin.path(), "agent.sh", AGENT);
    let root = tempfile::tempdir().unwrap();
    let manifest = invoke(&comparison(
        root.path(),
        "takes:s-795:effort",
        agent_body(&agent, path_only()),
        json!({"repetitions": 2, "variable": "effort", "levels": ["low", "high"],
            "stop": {"when": "pass"}}),
    ))
    .expect("the comparison runs");

    assert_eq!(manifest["variable"], "effort");
    assert_eq!(manifest["levels"], json!(["low", "high"]));
    assert_eq!(
        manifest["conditions"],
        json!(["classic"]),
        "the held condition"
    );
    assert_eq!(manifest["stop"], json!({"when": "pass"}));
    assert_eq!(manifest["task"]["id"], "task:session/s-795");
    let records = takes(&manifest);
    assert_eq!(records.len(), 4, "two levels, twice each");
    let order: Vec<(u64, &str)> = records
        .iter()
        .map(|r| {
            (
                r["repetition"].as_u64().unwrap(),
                r["level"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(order, [(0, "low"), (0, "high"), (1, "low"), (1, "high")]);
    for take in records {
        let level = take["level"].as_str().unwrap();
        assert_eq!(take["variable"], "effort");
        assert_eq!(take["condition"], "classic");
        assert_eq!(take["agent"], "agent:session/s-795");
        assert_eq!(take["status"], "completed", "{level}: {}", take["error"]);
        assert_eq!(
            take["verification"]["objective_checks_pass"], true,
            "{level}"
        );
        assert_eq!(take["stop"]["when"], "pass");
        assert_eq!(take["stop"]["met"], true, "{level}");
        assert_eq!(take["stop"]["reason"], "its test passes");
        // The agent acted in its own World: Actuation executed nothing for it.
        assert_eq!(take["capability_calls"], 0);
        // Low needed the second turn the stop condition asked of it; high
        // was stopped at its first.
        let turns = if level == "high" { 1 } else { 2 };
        assert_eq!(take["model_calls"], turns, "{level}");
        assert_eq!(take["total_tokens"], 5 * turns, "{level}");
        assert_eq!(take["final_workspace"]["answer.txt"], "42\n");
        // The body read its level back, and the record says so.
        let basis = &take["record"]["result"]["model_body"];
        assert_eq!(basis["protocol"], "agent-jsonl");
        assert_eq!(basis["acting"], true);
        assert_eq!(
            basis["level"],
            json!({"variable": "effort", "value": level})
        );
        assert_eq!(
            basis["preflight"]["level"],
            json!({"variable": "effort", "value": level})
        );
        assert_eq!(
            take["record"]["result"]["body_evidence"]["status"],
            "completed"
        );
        let receipt: Value = serde_json::from_slice(
            &fs::read(root.path().join(take["receipt_path"].as_str().unwrap())).unwrap(),
        )
        .unwrap();
        assert_eq!(receipt["level"], level);
    }
    // The second turn the low level took was asked by the stop condition.
    let low = &records[0]["record"]["result"]["observations"];
    let asked: Vec<&Value> = low
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["event_type"] == "model_requested")
        .collect();
    assert_eq!(asked.len(), 2);
    let history = asked[1]["value"]["payload"]["history"].as_array().unwrap();
    let told = history.last().unwrap()["content"].as_str().unwrap();
    assert!(told.contains("does not pass yet"), "{told}");
    // Everything else held: the model, the condition, the body's own setup.
    let held = &manifest["held_constants"];
    assert_eq!(held["valid"], true, "{held}");
    assert!(held["held"]
        .as_array()
        .unwrap()
        .contains(&json!("condition")));
    assert!(held["held"]
        .as_array()
        .unwrap()
        .contains(&json!("body_configuration")));
    // The takes are grouped by their level for review.
    let mask = invoke(&json!({"operation": "comparison.mask", "manifest": manifest})).unwrap();
    let row = mask["mapping"]["0"].as_object().unwrap();
    assert_eq!(row.len(), 2);
    assert!(row.contains_key("low") && row.contains_key("high"));
    let assessed =
        invoke(&json!({"operation": "comparison.assess", "manifest": manifest})).unwrap();
    assert!(
        !assessed["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r.as_str().unwrap().contains("matched set")),
        "each repetition holds both levels: {assessed}"
    );
    assert_eq!(assessed["human_acceptance"], false);
    // The plan, the manifest and one directory per take stand in the root.
    let plan: Value =
        serde_json::from_slice(&fs::read(root.path().join("plan.json")).unwrap()).unwrap();
    assert_eq!(plan["variable"], "effort");
    assert_eq!(plan["levels"], json!(["low", "high"]));
    assert!(root.path().join("manifest.json").is_file());
}

#[test]
fn stop_when_decides_where_each_take_ends() {
    let bin = tempfile::tempdir().unwrap();
    let agent = script(bin.path(), "agent.sh", AGENT);
    let run = |stop: Value, trace: &str| {
        let root = tempfile::tempdir().unwrap();
        let manifest = invoke(&comparison(
            root.path(),
            trace,
            agent_body(&agent, path_only()),
            json!({"variable": "effort", "levels": ["low"], "stop": stop}),
        ))
        .expect("the comparison runs");
        takes(&manifest)[0].clone()
    };
    // Task done: the run ends at the agent's own finish, check or no check.
    let done = run(json!({"when": "done"}), "takes:s-795:done");
    assert_eq!(done["model_calls"], 1);
    assert_eq!(done["status"], "completed");
    assert_eq!(done["stop"]["met"], true);
    assert_eq!(done["stop"]["reason"], "the body finished the task");
    assert_eq!(done["verification"]["objective_checks_pass"], false);
    assert_eq!(done["final_workspace"]["answer.txt"], "41\n");
    // Turn cap: the run goes on to its cap.
    let turns = run(json!({"when": "turns", "turns": 3}), "takes:s-795:turns");
    assert_eq!(turns["model_calls"], 3);
    assert_eq!(
        turns["stop"],
        json!({"when": "turns", "turns": 3, "reason": "its turn cap of 3 turns",
        "at_turn": 3, "elapsed_ms": turns["stop"]["elapsed_ms"], "met": true})
    );
    // A Turn cap names no number: the run's own max_steps.
    let steps = run(json!({"when": "turns"}), "takes:s-795:steps");
    assert_eq!(steps["model_calls"], 4);
    assert_eq!(steps["stop"]["turns"], 4);
    // Time cap: a cap that has passed by the first turn's end ends the take.
    let time = run(json!({"when": "time", "time_ms": 1}), "takes:s-795:time");
    assert_eq!(time["model_calls"], 1);
    assert_eq!(time["stop"]["met"], true);
    assert_eq!(time["stop"]["reason"], "its time cap of 1 ms");
}

#[test]
fn a_model_variable_reaches_the_body_and_only_the_model_varies() {
    let bin = tempfile::tempdir().unwrap();
    let agent = script(bin.path(), "agent.sh", AGENT);
    let root = tempfile::tempdir().unwrap();
    let manifest = invoke(&comparison(
        root.path(),
        "takes:s-795:model",
        agent_body(&agent, path_only()),
        json!({"variable": "model", "levels": ["m-a", "m-b"], "stop": {"when": "done"}}),
    ))
    .unwrap();
    let models: Vec<&Value> = takes(&manifest)
        .iter()
        .map(|r| &r["model"]["model"])
        .collect();
    assert_eq!(models, [&json!("m-a"), &json!("m-b")]);
    for take in takes(&manifest) {
        assert_eq!(take["status"], "completed", "{}", take["error"]);
        assert_eq!(
            take["record"]["result"]["model_body"]["preflight"]["model"],
            take["level"]
        );
    }
    let held = &manifest["held_constants"];
    assert_eq!(held["valid"], true, "{held}");
    assert!(!held["held"].as_array().unwrap().contains(&json!("model")));
}

#[test]
fn a_body_that_does_not_read_back_its_level_makes_no_take_of_it() {
    let bin = tempfile::tempdir().unwrap();
    let agent = script(bin.path(), "agent.sh", AGENT);
    let root = tempfile::tempdir().unwrap();
    let manifest = invoke(&comparison(
        root.path(),
        "takes:s-795:deaf",
        agent_body(&agent, json!({"PATH": "/usr/bin:/bin", "ECHO_LEVEL": "0"})),
        json!({"variable": "autonomy", "levels": ["review"], "stop": {"when": "done"}}),
    ))
    .unwrap();
    let take = &takes(&manifest)[0];
    assert_eq!(take["status"], "failed");
    let said = take["record"]["result"]["execution"]["report"]["error"]
        .as_str()
        .unwrap_or_default();
    assert!(said.contains("did not read back the level"), "{take}");
    // It never acted: the World is as the task started it.
    assert_eq!(take["final_workspace"]["answer.txt"], "0\n");
}

#[test]
fn plans_a_comparison_cannot_run_are_refused_before_any_take() {
    let bin = tempfile::tempdir().unwrap();
    let agent = script(bin.path(), "agent.sh", AGENT);
    let one_shot = json!({"fixture_provider": true, "configuration": {"provider": "fixture", "model": "m-1"},
        "process": {"program": agent, "args": [], "cwd": "/tmp", "environment": {}, "timeout_ms": 1000, "output_limit": 1000}});
    let mut untested = own_task();
    untested.as_object_mut().unwrap().remove("test");
    let mut stray = own_task();
    stray["expected"] = json!("42");
    let cases: Vec<(&str, Value, Value, &str)> = vec![
        (
            "unknown variable",
            agent_body(&agent, path_only()),
            json!({"variable": "temperature", "levels": ["0.1"]}),
            "comparison variable is one of",
        ),
        (
            "no levels",
            agent_body(&agent, path_only()),
            json!({"variable": "effort"}),
            "names its levels",
        ),
        (
            "four levels",
            agent_body(&agent, path_only()),
            json!({"variable": "effort", "levels": ["low", "medium", "high", "max"]}),
            "1..3 distinct",
        ),
        (
            "conditions with another variable",
            agent_body(&agent, path_only()),
            json!({"variable": "effort", "levels": ["low"], "conditions": ["classic"]}),
            "conditions are the condition's levels",
        ),
        (
            "a body that cannot carry a level",
            one_shot.clone(),
            json!({"variable": "effort", "levels": ["low"]}),
            "applies it and reads it back",
        ),
        (
            "an unadmitted condition",
            agent_body(&agent, path_only()),
            json!({"levels": ["classic", "ql-nine"]}),
            "not admitted: ql-nine",
        ),
        (
            "a turn cap past max_steps",
            agent_body(&agent, path_only()),
            json!({"variable": "effort", "levels": ["low"], "stop": {"when": "turns", "turns": 9}}),
            "cannot exceed",
        ),
        (
            "a time cap with no time",
            one_shot.clone(),
            json!({"levels": ["classic"], "stop": {"when": "time"}}),
            "Time cap needs",
        ),
        (
            "an unknown stop",
            agent_body(&agent, path_only()),
            json!({"variable": "effort", "levels": ["low"], "stop": {"when": "soon"}}),
            "stop when is one of",
        ),
    ];
    for (case, body, extra, words) in cases {
        let root = tempfile::tempdir().unwrap();
        let error = invoke(&comparison(root.path(), "takes:s-795:bad", body, extra))
            .expect_err(case)
            .to_string();
        assert!(error.contains(words), "{case}: {error}");
        assert!(
            fs::read_dir(root.path()).unwrap().next().is_none(),
            "{case}: nothing was reserved"
        );
    }
    // A Test pass needs a check the task brings; a provided task is its own shape.
    for (case, task, words) in [
        (
            "test pass with no test",
            untested,
            "Test pass needs the task's objective check",
        ),
        (
            "a field a provided task does not take",
            stray,
            "has no field expected",
        ),
    ] {
        let root = tempfile::tempdir().unwrap();
        let mut request = comparison(
            root.path(),
            "takes:s-795:task",
            agent_body(&agent, path_only()),
            json!({"variable": "effort", "levels": ["low"], "stop": {"when": "pass"}}),
        );
        request["run"]["task"] = task;
        let error = invoke(&request).expect_err(case).to_string();
        assert!(error.contains(words), "{case}: {error}");
    }
    // One task: a catalogue id or a provided task, never both.
    let root = tempfile::tempdir().unwrap();
    let mut both = comparison(
        root.path(),
        "takes:s-795:both",
        agent_body(&agent, path_only()),
        json!({"variable": "effort", "levels": ["low"]}),
    );
    both["run"]["task_id"] = json!("S1-SKILL-001");
    let error = invoke(&both).expect_err("both").to_string();
    assert!(error.contains("not both"), "{error}");
}

#[test]
fn test_pass_ends_a_model_bodys_run_before_its_next_turn() {
    let bin = tempfile::tempdir().unwrap();
    let model = script(bin.path(), "model.sh", RESTLESS_MODEL);
    let run = |stop: Value| {
        let state = tempfile::tempdir().unwrap();
        let world = tempfile::tempdir().unwrap();
        let mut request = json!({
            "operation": "run", "condition": "classic", "trace_ref": "trace:s-795:restless",
            "world": world.path().to_string_lossy(), "task": own_task(),
            "max_steps": 6, "max_calls": 32,
            "body": {"fixture_provider": true, "process": {"program": model, "args": [],
                "cwd": state.path().to_string_lossy(), "environment": path_only(),
                "timeout_ms": 10000, "output_limit": 65536}}
        });
        if !stop.is_null() {
            request["stop"] = stop;
        }
        invoke(&request).expect("the run runs")["result"].clone()
    };
    // Without a stop condition the body never finishes: the run is exhausted.
    let restless = run(Value::Null);
    assert_eq!(restless["model_calls"], 6);
    assert_eq!(restless["status"], "exhausted");
    assert!(restless.get("stop").is_none());
    // With Test pass it ends before the turn after the one that made it pass.
    let stopped = run(json!({"when": "pass"}));
    assert_eq!(stopped["model_calls"], 2);
    assert_eq!(stopped["status"], "completed");
    assert_eq!(stopped["stop"]["met"], true);
    assert_eq!(stopped["stop"]["at_turn"], 2);
    assert_eq!(stopped["verification"]["objective_checks_pass"], true);
    assert_eq!(stopped["verification"]["protocol"], "provided-task-test");
}

#[test]
fn the_research_application_discloses_what_a_comparison_can_vary() {
    let caps = invoke(&json!({"operation": "capabilities"})).unwrap();
    assert_eq!(
        caps["comparison"]["variables"],
        json!([
            "condition",
            "model",
            "effort",
            "skillset",
            "budget",
            "autonomy"
        ])
    );
    assert_eq!(
        caps["comparison"]["stop_when"],
        json!(["done", "turns", "time", "pass"])
    );
    assert_eq!(caps["comparison"]["provided_tasks"], true);
    assert!(caps["model_body_protocols"]
        .as_array()
        .unwrap()
        .contains(&json!("agent-jsonl")));
}
