//! `actuation occupancy` through the real executable on an isolated store.
//!
//! World inhabitation contract v1 §2: a stable World Position is occupied by
//! one tenure at a time. A second claim is refused naming the holder; a
//! handover names the current generation and supersedes it in one locked
//! write, after which the predecessor can no longer verify, report presence or
//! release; racing claims on a vacant Position produce exactly one occupant;
//! two open tenures are an ambiguity, never newest-wins; and the Position
//! outlives every change of Agent, Agency, session, model, harness and
//! Workcell across its tenures.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const POSITION: &str = "central:position:project:O-I:factory-guardian";

struct Store {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

impl Store {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = dir.path().join("occupancy");
        Self { _dir: dir, root }
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_actuation"));
        command
            .arg("occupancy")
            .args(args)
            .env("ACTUATION_OCCUPANCY_STORE", &self.root)
            // Never let a missing variable fall through to a real home store.
            .env("HOME", self._dir.path());
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().expect("actuation runs")
    }

    /// Run with `--json`; return (exit code, parsed stdout).
    fn json(&self, args: &[&str]) -> (i32, Value) {
        let mut all = args.to_vec();
        all.push("--json");
        let output = self.run(&all);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "{all:?} did not print JSON ({e}); stdout={stdout} stderr={}",
                String::from_utf8_lossy(&output.stderr)
            )
        });
        (output.status.code().expect("exit code"), value)
    }

    fn ok(&self, args: &[&str]) -> Value {
        let (code, value) = self.json(args);
        assert_eq!(code, 0, "{args:?} refused: {value:#}");
        value
    }

    fn refused(&self, args: &[&str], code: &str) -> Value {
        let (exit, value) = self.json(args);
        assert_eq!(exit, 2, "{args:?} should refuse: {value:#}");
        assert_eq!(value["ok"], json!(false), "{value:#}");
        assert_eq!(value["error"]["code"], json!(code), "{value:#}");
        for part in ["fact", "consequence", "action"] {
            assert!(
                value["error"][part]
                    .as_str()
                    .is_some_and(|text| !text.is_empty()),
                "refusal lacks its {part}: {value:#}"
            );
        }
        value["error"].clone()
    }

    fn ledger(&self, position: &str) -> PathBuf {
        use sha2::{Digest, Sha256};
        self.root
            .join(format!("{:x}.jsonl", Sha256::digest(position.as_bytes())))
    }
}

fn lines(path: &Path) -> Vec<String> {
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect()
}

fn generation_of(value: &Value) -> String {
    value["generation"]["generation_ref"]
        .as_str()
        .expect("generation ref")
        .to_owned()
}

fn is_generation_ref(value: &str) -> bool {
    let Some(uuid) = value.strip_prefix("actuation:generation:") else {
        return false;
    };
    uuid.len() == 36
        && uuid.chars().enumerate().all(|(i, c)| match i {
            8 | 13 | 18 | 23 => c == '-',
            _ => c.is_ascii_hexdigit() && !c.is_ascii_uppercase(),
        })
}

fn claim<'a>(agent: &'a str, reason: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "claim",
        "--position",
        POSITION,
        "--agent",
        agent,
        "--agency",
        "agency:factory-guardian",
        "--reason",
        reason,
    ];
    args.extend_from_slice(extra);
    args
}

#[test]
fn an_initial_claim_opens_generation_one_and_stamps_the_body_environment() {
    let store = Store::new();
    let claimed = store.ok(&claim(
        "agent/factory-guardian",
        "first occupancy",
        &["--agent-session", "session:claude-1", "--expect-vacant"],
    ));
    let generation = generation_of(&claimed);
    assert!(is_generation_ref(&generation), "{generation}");
    assert_eq!(claimed["generation"]["ordinal"], json!(1));
    assert_eq!(
        claimed["tenure"]["schema"],
        json!("actuation.position-tenure/v1")
    );
    assert_eq!(claimed["tenure"]["kind"], json!("initial"));
    assert_eq!(claimed["tenure"]["reason"], json!("first occupancy"));
    assert!(claimed["tenure"]
        .get("predecessor_generation_ref")
        .is_none());
    assert_eq!(claimed["env"]["OI_POSITION_REF"], json!(POSITION));
    assert_eq!(claimed["env"]["OI_OCCUPANT_GENERATION"], json!(generation));

    let reading = store.ok(&["read", "--position", POSITION]);
    assert_eq!(reading["schema"], json!("actuation.position-occupancy/v1"));
    assert_eq!(reading["state"], json!("occupied"));
    assert_eq!(reading["current"]["generation_ref"], json!(generation));
    assert!(reading.get("predecessor").is_none());
    assert_eq!(reading["generations"].as_array().unwrap().len(), 1);

    let verified = store.ok(&[
        "verify",
        "--position",
        POSITION,
        "--generation",
        &generation,
    ]);
    assert_eq!(verified["current"]["generation_ref"], json!(generation));

    let presence = store.ok(&[
        "presence",
        "--position",
        POSITION,
        "--generation",
        &generation,
        "--presence",
        "active",
        "--attention",
        "reviewing Factory #195",
    ]);
    assert_eq!(presence["presence"]["presence"], json!("active"));
    let reading = store.ok(&["read", "--position", POSITION]);
    assert_eq!(
        reading["presence"]["attention"],
        json!("reviewing Factory #195")
    );
}

#[test]
fn a_second_claim_is_refused_naming_the_holder_and_the_next_lawful_command() {
    let store = Store::new();
    let first = store.ok(&claim(
        "agent/factory-guardian",
        "first",
        &["--agent-session", "session:claude-1"],
    ));
    let holder = generation_of(&first);
    let ledger = store.ledger(POSITION);
    let before = lines(&ledger);

    for expectation in [&[][..], &["--expect-vacant"][..]] {
        let error = store.refused(
            &claim("agent/intruder", "take it", expectation),
            "occupancy.occupied",
        );
        let fact = error["fact"].as_str().unwrap();
        for named in [
            holder.as_str(),
            "ordinal 1",
            "agent/factory-guardian",
            "session:claude-1",
            "since ",
        ] {
            assert!(fact.contains(named), "fact does not name {named}: {fact}");
        }
        assert!(error["consequence"]
            .as_str()
            .unwrap()
            .starts_with("Nothing was written"));
        let action = error["action"].as_str().unwrap();
        assert!(
            action.contains(&format!("--expect-generation {holder}"))
                && action.contains("actuation occupancy claim --position"),
            "{action}"
        );
        assert_eq!(error["current"]["generation_ref"], json!(holder));
    }
    assert_eq!(lines(&ledger), before, "a refused claim must not write");

    // Human mode: three lines on stderr, nothing on stdout, exit 2.
    let output = store.run(&claim("agent/intruder", "take it", &[]));
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let parts: Vec<&str> = stderr.trim_end().lines().collect();
    assert_eq!(parts.len(), 3, "{stderr}");
    assert!(parts[0].starts_with("actuation: ") && parts[0].contains(&holder));
    assert!(parts[1].starts_with("  Nothing was written"));
    assert!(parts[2].contains("--expect-generation"));
}

#[test]
fn a_handover_supersedes_the_predecessor_which_can_no_longer_act() {
    let store = Store::new();
    let first = generation_of(&store.ok(&claim("agent/factory-guardian", "first", &[])));
    let handed = store.ok(&claim(
        "agent/factory-guardian",
        "context wall",
        &["--expect-generation", &first],
    ));
    let second = generation_of(&handed);
    assert_ne!(first, second);
    assert_eq!(handed["tenure"]["kind"], json!("handover"));
    assert_eq!(handed["generation"]["ordinal"], json!(2));
    assert_eq!(handed["tenure"]["predecessor_generation_ref"], json!(first));
    assert_eq!(handed["superseded"]["generation_ref"], json!(first));
    assert_eq!(handed["superseded"]["end_kind"], json!("superseded"));
    assert_eq!(handed["superseded"]["end_reason"], json!("context wall"));
    assert_eq!(
        handed["superseded"]["ended_at_unix_ms"], handed["tenure"]["began_at_unix_ms"],
        "the supersession and the successor are one write"
    );

    let ledger_before = lines(&store.ledger(POSITION));
    for verb in [
        vec!["verify", "--position", POSITION, "--generation", &first],
        vec![
            "presence",
            "--position",
            POSITION,
            "--generation",
            &first,
            "--presence",
            "active",
        ],
        vec![
            "release",
            "--position",
            POSITION,
            "--generation",
            &first,
            "--reason",
            "leaving",
        ],
    ] {
        let error = store.refused(&verb, "occupancy.superseded");
        let fact = error["fact"].as_str().unwrap();
        assert!(fact.contains(&first) && fact.contains(&second), "{fact}");
        assert!(fact.contains("superseded by"), "{fact}");
        assert_eq!(error["current"]["generation_ref"], json!(second));
        assert!(error["action"]
            .as_str()
            .unwrap()
            .contains("actuation occupancy read --position"));
    }
    assert_eq!(lines(&store.ledger(POSITION)), ledger_before);

    // A claim against the stale expectation never supersedes the successor.
    let error = store.refused(
        &claim(
            "agent/factory-guardian",
            "again",
            &["--expect-generation", &first],
        ),
        "occupancy.stale_expectation",
    );
    assert!(error["fact"].as_str().unwrap().contains(&second));
    assert!(error["action"]
        .as_str()
        .unwrap()
        .contains(&format!("--expect-generation {second}")));

    let verified = store.ok(&["verify", "--position", POSITION, "--generation", &second]);
    assert_eq!(verified["generation"]["ordinal"], json!(2));

    let reading = store.ok(&["read", "--position", POSITION]);
    assert_eq!(reading["current"]["generation_ref"], json!(second));
    assert_eq!(reading["predecessor"]["generation_ref"], json!(first));
    assert_eq!(reading["predecessor"]["end_kind"], json!("superseded"));
}

#[test]
fn release_leaves_the_position_vacant_and_a_fresh_claim_takes_the_next_ordinal() {
    let store = Store::new();
    let first = generation_of(&store.ok(&claim("agent/factory-guardian", "first", &[])));
    let second = generation_of(&store.ok(&claim(
        "agent/factory-guardian",
        "handover",
        &["--expect-generation", &first],
    )));
    let released = store.ok(&[
        "release",
        "--position",
        POSITION,
        "--generation",
        &second,
        "--reason",
        "work finished",
    ]);
    assert_eq!(released["tenure"]["end_kind"], json!("released"));
    assert_eq!(released["tenure"]["end_reason"], json!("work finished"));
    assert_eq!(released["occupancy"]["state"], json!("vacant"));

    let error = store.refused(
        &["verify", "--position", POSITION, "--generation", &second],
        "occupancy.superseded",
    );
    assert!(error["fact"].as_str().unwrap().contains("released"));
    assert!(error["action"]
        .as_str()
        .unwrap()
        .contains("--expect-vacant"));

    // A stale expectation against a vacant Position points at --expect-vacant.
    let error = store.refused(
        &claim(
            "agent/factory-guardian",
            "back",
            &["--expect-generation", &second],
        ),
        "occupancy.stale_expectation",
    );
    assert!(error["action"]
        .as_str()
        .unwrap()
        .contains("--expect-vacant"));

    let fresh = store.ok(&claim(
        "agent/factory-guardian",
        "new day",
        &["--expect-vacant"],
    ));
    let third = generation_of(&fresh);
    assert!(is_generation_ref(&third));
    assert!(
        third != first && third != second,
        "a new occupant gets a new uuid"
    );
    assert_eq!(fresh["generation"]["ordinal"], json!(3));
    assert_eq!(fresh["tenure"]["kind"], json!("fresh"));
    assert!(fresh["tenure"].get("predecessor_generation_ref").is_none());
    assert_eq!(
        fresh["occupancy"]["predecessor"]["generation_ref"],
        json!(second)
    );
    assert_eq!(
        fresh["occupancy"]["generations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["generation_ordinal"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
}

#[test]
fn racing_claims_on_a_vacant_position_produce_exactly_one_occupant() {
    let store = Store::new();
    let racers: Vec<_> = (0..16)
        .map(|n| {
            let agent = format!("agent/racer-{n}");
            store
                .command(&[
                    "claim",
                    "--position",
                    POSITION,
                    "--agent",
                    &agent,
                    "--agency",
                    "agency:race",
                    "--reason",
                    "race",
                    "--expect-vacant",
                    "--json",
                ])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .expect("spawn racer")
        })
        .collect();
    let mut winners = 0;
    let mut refused = 0;
    for racer in racers {
        let output = racer.wait_with_output().expect("racer finishes");
        let value: Value = serde_json::from_slice(&output.stdout).expect("racer prints JSON");
        match output.status.code() {
            Some(0) => winners += 1,
            Some(2) => {
                assert_eq!(
                    value["error"]["code"],
                    json!("occupancy.occupied"),
                    "{value:#}"
                );
                refused += 1;
            }
            other => panic!("unexpected exit {other:?}: {value:#}"),
        }
    }
    assert_eq!((winners, refused), (1, 15));
    let began = lines(&store.ledger(POSITION))
        .iter()
        .filter(|line| line.contains("\"tenure_began\""))
        .count();
    assert_eq!(began, 1, "exactly one tenure may begin");
    let reading = store.ok(&["read", "--position", POSITION]);
    assert_eq!(reading["generations"].as_array().unwrap().len(), 1);
}

#[test]
fn two_open_tenures_are_an_ambiguity_refusal_and_never_newest_wins() {
    let store = Store::new();
    let claimed = store.ok(&claim("agent/factory-guardian", "first", &[]));
    let first = generation_of(&claimed);
    // Forge a second open tenure, as a writer bypassing the lock would.
    let mut forged = claimed["tenure"].clone();
    let intruder = "actuation:generation:00000000-0000-4000-8000-000000000002";
    forged["generation_ref"] = json!(intruder);
    forged["generation_ordinal"] = json!(2);
    forged["agent_ref"] = json!("agent/intruder");
    let at = forged["began_at_unix_ms"].as_u64().unwrap() + 1;
    forged["began_at_unix_ms"] = json!(at);
    let line = json!({
        "event": "tenure_began",
        "schema": "actuation.position-ledger/v1",
        "position_ref": POSITION,
        "at_unix_ms": at,
        "tenure": forged,
    });
    let ledger = store.ledger(POSITION);
    let mut raw = std::fs::read_to_string(&ledger).unwrap();
    raw.push_str(&format!("{line}\n"));
    std::fs::write(&ledger, raw).unwrap();

    let error = store.refused(&["read", "--position", POSITION], "occupancy.ambiguous");
    let candidates = error["candidates"].as_array().unwrap();
    assert_eq!(candidates.len(), 2);
    assert!(error["fact"]
        .as_str()
        .unwrap()
        .contains("newest is never taken"));
    assert!(error["action"]
        .as_str()
        .unwrap()
        .contains("actuation occupancy release"));
    for verb in [
        vec!["verify", "--position", POSITION, "--generation", intruder],
        vec!["verify", "--position", POSITION, "--generation", &first],
        vec![
            "presence",
            "--position",
            POSITION,
            "--generation",
            intruder,
            "--presence",
            "idle",
        ],
    ] {
        store.refused(&verb, "occupancy.ambiguous");
    }
    store.refused(
        &claim("agent/other", "x", &["--expect-generation", intruder]),
        "occupancy.ambiguous",
    );
    let listed = store.ok(&["list"]);
    assert!(listed["positions"].as_array().unwrap().is_empty());
    assert_eq!(listed["invalid"][0]["code"], json!("occupancy.ambiguous"));
    assert_eq!(listed["invalid"][0]["position_ref"], json!(POSITION));

    // The lawful repair: the caller names which open tenure ends.
    let repaired = store.ok(&[
        "release",
        "--position",
        POSITION,
        "--generation",
        intruder,
        "--reason",
        "forged tenure",
    ]);
    assert_eq!(
        repaired["occupancy"]["current"]["generation_ref"],
        json!(first)
    );

    // An unparseable line is corruption, named by line, and nothing reads past it.
    let mut raw = std::fs::read_to_string(&ledger).unwrap();
    raw.push_str("{not json\n");
    std::fs::write(&ledger, raw).unwrap();
    let error = store.refused(&["read", "--position", POSITION], "occupancy.corrupt");
    assert_eq!(error["line"], json!(4));
    store.refused(
        &claim("agent/other", "x", &["--expect-generation", &first]),
        "occupancy.corrupt",
    );
}

#[test]
fn list_is_uncapped() {
    let store = Store::new();
    for n in 0..150 {
        let position = format!("central:position:project:O-I:seat-{n:03}");
        store.ok(&[
            "claim",
            "--position",
            &position,
            "--agent",
            "agent/any",
            "--agency",
            "agency:any",
            "--reason",
            "fill",
        ]);
    }
    let listed = store.ok(&["list"]);
    assert_eq!(
        listed["schema"],
        json!("actuation.position-occupancy-listing/v1")
    );
    let positions = listed["positions"].as_array().unwrap();
    assert_eq!(positions.len(), 150);
    assert!(listed["invalid"].as_array().unwrap().is_empty());
    assert_eq!(
        positions[149]["position_ref"],
        json!("central:position:project:O-I:seat-149")
    );
    assert!(positions.iter().all(|p| p["state"] == json!("occupied")));
}

#[test]
fn the_position_survives_every_change_of_occupant_body_and_placement() {
    let store = Store::new();
    let body = |n: usize| -> Vec<String> {
        vec![
            "--agent".into(),
            format!("agent/occupant-{n}"),
            "--agency".into(),
            format!("agency:occupant-{n}"),
            "--agent-session".into(),
            format!("session:{n}"),
            "--session-space".into(),
            format!("session-space:{n}"),
            "--harness-composition".into(),
            format!("harness:{n}"),
            "--model".into(),
            format!("model:{n}"),
            "--workcell".into(),
            format!("workcell:{n}"),
            "--gateway-address".into(),
            format!("gateway://{n}"),
        ]
    };
    let claim_with = |n: usize, expectation: &[&str]| -> Value {
        let body = body(n);
        let mut args: Vec<&str> = vec!["claim", "--position", POSITION, "--reason", "occupy"];
        args.extend(body.iter().map(String::as_str));
        args.extend_from_slice(expectation);
        store.ok(&args)
    };
    let one = generation_of(&claim_with(1, &[]));
    let two = generation_of(&claim_with(2, &["--expect-generation", &one]));
    store.ok(&[
        "release",
        "--position",
        POSITION,
        "--generation",
        &two,
        "--reason",
        "moving workcell",
    ]);
    claim_with(3, &["--expect-vacant", "--kind", "adopt"]);

    let reading = store.ok(&["read", "--position", POSITION]);
    assert_eq!(reading["position_ref"], json!(POSITION));
    let generations = reading["generations"].as_array().unwrap();
    assert_eq!(generations.len(), 3);
    for (index, tenure) in generations.iter().enumerate() {
        let n = index + 1;
        assert_eq!(tenure["position_ref"], json!(POSITION), "same address");
        assert_eq!(tenure["generation_ordinal"], json!(n));
        for (field, expected) in [
            ("agent_ref", format!("agent/occupant-{n}")),
            ("agency_ref", format!("agency:occupant-{n}")),
            ("agent_session_ref", format!("session:{n}")),
            ("session_space_ref", format!("session-space:{n}")),
            ("harness_composition_ref", format!("harness:{n}")),
            ("model_ref", format!("model:{n}")),
            ("workcell_ref", format!("workcell:{n}")),
            ("gateway_address", format!("gateway://{n}")),
        ] {
            assert_eq!(tenure[field], json!(expected), "{field} of tenure {n}");
        }
    }
    let kinds: Vec<&str> = generations
        .iter()
        .map(|t| t["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, ["initial", "handover", "adopt"]);
    let refs: std::collections::BTreeSet<&str> = generations
        .iter()
        .map(|t| t["generation_ref"].as_str().unwrap())
        .collect();
    assert_eq!(refs.len(), 3, "every tenure has its own generation");
}

#[test]
fn malformed_requests_refuse_in_three_parts_before_touching_the_store() {
    let store = Store::new();
    let no_reason = store.refused(
        &[
            "claim",
            "--position",
            POSITION,
            "--agent",
            "a",
            "--agency",
            "g",
        ],
        "occupancy.usage",
    );
    assert!(no_reason["fact"].as_str().unwrap().contains("--reason"));
    assert!(no_reason["action"]
        .as_str()
        .unwrap()
        .contains("actuation occupancy claim"));
    store.refused(
        &claim(
            "a",
            "r",
            &[
                "--expect-vacant",
                "--expect-generation",
                "actuation:generation:00000000-0000-4000-8000-000000000001",
            ],
        ),
        "occupancy.usage",
    );
    store.refused(
        &claim("a", "r", &["--agentsession", "s"]),
        "occupancy.usage",
    );
    store.refused(&claim("a", "   ", &[]), "occupancy.usage");
    store.refused(
        &[
            "verify",
            "--position",
            POSITION,
            "--generation",
            "session:1",
        ],
        "occupancy.usage",
    );
    store.refused(
        &[
            "release",
            "--position",
            POSITION,
            "--generation",
            "actuation:generation:00000000-0000-4000-8000-000000000001",
        ],
        "occupancy.usage",
    );
    assert!(
        !store.root.exists(),
        "no refused request may create the store"
    );

    // Kinds must tell the truth about the history.
    store.refused(
        &claim("a", "r", &["--kind", "handover"]),
        "occupancy.kind_inconsistent",
    );
    let first = generation_of(&store.ok(&claim("a", "r", &[])));
    store.refused(
        &claim(
            "a",
            "r",
            &["--expect-generation", &first, "--kind", "initial"],
        ),
        "occupancy.kind_inconsistent",
    );
    // An unknown generation is never mistaken for a superseded one.
    let error = store.refused(
        &[
            "verify",
            "--position",
            POSITION,
            "--generation",
            "actuation:generation:00000000-0000-4000-8000-00000000000f",
        ],
        "occupancy.unknown_generation",
    );
    assert!(error["fact"].as_str().unwrap().contains("has never held"));
}
