//! The faculty receipt bridge, end to end and offline.
//!
//! A relational Prime run hands its specimen the research binary, the faculty
//! configuration and the owning trace through the environment. The specimen's
//! faculty operations file content-addressed native receipts through
//! `faculty.invoke`, and `run_prime`'s claims collector reads them back. This
//! suite scripts the specimen (a Prime RPC responder) and the QL owner
//! instrument (an exact-revision responder) so the whole bridge runs without
//! a provider call, a real Prime executable or a real QL checkout.

use actuation_research::{
    execution::RecordingObserver,
    owner::OwnerInstrument,
    prime_run::{run_prime, PrimeRunRequest},
    process::ProcessSpec,
    world::World,
};
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::PathBuf};

const LOCK_FIXTURE: &str =
    include_str!("../../../experiments/native-research/prime-source-lock.json");

fn owner_script(revision: &str) -> String {
    format!(
        r#"#!/bin/sh
request=$(cat)
operation=$(printf '%s' "$request" | /usr/bin/sed -n 's/.*"operation":"\([^"]*\)".*/\1/p')
if [ "$operation" = "capabilities" ]; then
  printf '{{"schema":"actuation.ql-owner-operation/v1","owner_repository":"EpiLogos/QL-MEF","owner_revision":"{revision}","operation":"capabilities","result":{{"formal_owner":"EpiLogos/QL-MEF","positions":6}}}}\n'
elif [ "$operation" = "source-state" ]; then
  printf '{{"schema":"actuation.ql-owner-operation/v1","owner_repository":"EpiLogos/QL-MEF","owner_revision":"{revision}","operation":"source-state","result":{{"standing":"scripted"}}}}\n'
else
  exit 1
fi
"#
    )
}

fn prime_script() -> &'static str {
    // The scripted Prime answers the RPC protocol the driver speaks and, on
    // the prompt turn, files one faculty receipt through the research binary
    // exactly as the ql_relational skill does (same env contract, same
    // faculty.invoke payload shape).
    r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  printf 'v0.9.4\n'
  exit 0
fi
while IFS= read -r line; do
  id=$(printf '%s' "$line" | /usr/bin/sed -n 's/.*"id":"\([^"]*\)".*/\1/p')
  type=$(printf '%s' "$line" | /usr/bin/sed -n 's/.*"type":"\([^"]*\)".*/\1/p')
  case "$type" in
    get_state)
      printf '{"type":"response","id":"%s","command":"get_state","success":true,"data":{"isStreaming":false}}\n' "$id"
      ;;
    prompt)
      printf '{"type":"response","id":"%s","command":"prompt","success":true,"data":{}}\n' "$id"
      printf '{"operation":"faculty.invoke","configuration":"%s","request":{"operation":"source-state"},"trace_ref":"%s","declared_locus_ref":"prime-test-root"}\n' "$ACTUATION_RESEARCH_FACULTY_CONFIG" "$ACTUATION_RESEARCH_TRACE_REF" \
        | "$ACTUATION_RESEARCH_BIN" >/dev/null 2>&1
      ;;
    get_last_assistant_text)
      printf '{"type":"response","id":"%s","command":"get_last_assistant_text","success":true,"data":{"text":"done"}}\n' "$id"
      ;;
    get_messages)
      printf '{"type":"response","id":"%s","command":"get_messages","success":true,"data":{"messages":[]}}\n' "$id"
      ;;
    get_session_stats)
      printf '{"type":"response","id":"%s","command":"get_session_stats","success":true,"data":{}}\n' "$id"
      ;;
  esac
done
"#
}

fn executable(dir: &std::path::Path, name: &str, script: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    std::fs::write(&path, script).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn spec(program: PathBuf, cwd: &std::path::Path) -> ProcessSpec {
    ProcessSpec {
        program,
        args: vec![],
        cwd: cwd.to_owned(),
        environment: BTreeMap::new(),
        timeout_ms: 60_000,
        output_limit: 1 << 20,
    }
}

#[test]
fn prime_scripted_faculty_receipts_reach_the_run_claims() {
    let lock: Value = serde_json::from_str(LOCK_FIXTURE).expect("lock fixture parses");
    let accepted = lock["ql_mef"]["accepted_main_revision"]
        .as_str()
        .expect("accepted revision")
        .to_owned();

    let base = tempfile::tempdir().expect("base dir");
    let evidence = base.path().join("evidence");
    std::fs::create_dir(&evidence).expect("evidence world");
    let skill = base.path().join("skill");
    std::fs::create_dir_all(skill.join("src").join("ql_relational")).expect("skill tree");
    std::fs::write(
        skill.join("src").join("ql_relational").join("__init__.py"),
        b"",
    )
    .unwrap();

    let owner_program = executable(base.path(), "owner-instrument", &owner_script(&accepted));
    let owner_spec = spec(owner_program.clone(), base.path());
    let owner = OwnerInstrument::bind(owner_spec.clone(), &accepted).expect("owner binds");

    let faculty_config = base.path().join("faculty.json");
    std::fs::write(
        &faculty_config,
        json!({
            "schema": "actuation.prime-faculty/v1",
            "owner": {"process": owner_spec, "revision": accepted},
            "harmonic_enabled": false,
            "evidence_root": evidence.to_string_lossy(),
        })
        .to_string(),
    )
    .expect("faculty configuration");

    let research_binary = PathBuf::from(env!("CARGO_BIN_EXE_actuation-research"));
    let prime_program = executable(base.path(), "prime-fake", prime_script());
    let world_dir = tempfile::tempdir().expect("run world");
    let world = World::open(world_dir.path()).expect("run world opens");

    let request = PrimeRunRequest {
        trace_ref: actuation_core::ExternalRef::new("trace:test:prime-receipt-bridge").unwrap(),
        task_id: "S1-RESTRAINT-001".into(),
        condition: "prime-relational".into(),
        source_lock: lock,
        prime: spec(prime_program, base.path()),
        provider: "fixture".into(),
        model: "scripted".into(),
        fixture_provider: true,
        allow_refinement: false,
        node: None,
        skill_path: Some(skill),
        research_binary: Some(research_binary),
        faculty_config: Some(faculty_config),
    };
    let mut observer = RecordingObserver::default();
    let record = run_prime(&request, &world, Some(&owner), &mut observer)
        .expect("the scripted relational run completes");

    assert_eq!(record["execution_status"], json!("completed"), "{record}");
    assert_eq!(
        record["claims"]["ql_relational_faculty_exercised"],
        json!(true),
        "the native receipt must drive the claim, not the specimen's own log"
    );
    let operations = record["claims"]["relational_operations"]
        .as_array()
        .expect("relational operations claim");
    assert!(
        operations.iter().any(|o| *o == json!("source-state")),
        "{operations:?}"
    );
    let receipts = record["faculty"]["receipts"]
        .as_array()
        .expect("collected receipts");
    assert_eq!(receipts.len(), 1, "{receipts:?}");
    assert_eq!(receipts[0]["operation"], json!("source-state"));
    assert_eq!(receipts[0]["success"], json!(true));
    assert_eq!(receipts[0]["declared_locus_ref"], json!("prime-test-root"));
    assert_eq!(
        receipts[0]["trace_ref"],
        json!("trace:test:prime-receipt-bridge")
    );
}
