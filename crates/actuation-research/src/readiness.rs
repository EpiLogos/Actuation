//! Executable deterministic review of the eight retained QLDR scenarios.
//! This creates controlled temporary Worlds and uses real filesystem/JSONL
//! operations. Its scripted controller is explicitly not a provider experiment.
use crate::{
    evidence::stable_digest,
    execution::{ModelBody, RecordingObserver, ResearchHost},
    relational::*,
    tasks,
    world::World,
    Error, Result,
};
use actuation_core::ExternalRef;
use actuation_runtime::{CancellationToken, RuntimeHost, StreamRuntimeObserver};
use actuation_stream::{JsonlStreamStore, OpenStream, StreamStore};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, VecDeque},
    path::{Path, PathBuf},
};
const CATALOG: &str =
    include_str!("../../../experiments/ql-runtime/deep-ql/convergence/dry-runs.json");
struct FixtureBody;
impl ModelBody for FixtureBody {
    fn complete(&mut self, _: &Value) -> Result<Value> {
        Ok(json!({"content":"controlled structural-readiness response"}))
    }
    fn basis(&self) -> Value {
        json!({"fixture_provider":true,"provider_evidence":false})
    }
}
#[derive(Clone)]
struct Step {
    act: Act,
    to: u8,
}
fn step(to: u8) -> Step {
    Step {
        act: Act {
            source_position: None,
            intent: json!("retained readiness scenario"),
            carrier: json!({"kind":"internal_control","input":{"standing":"scripted-structural-fixture"}}),
            input_residue_refs: vec![],
            nested: None,
            metadata: Value::Null,
        },
        to,
    }
}
fn tool(to: u8, name: &str, args: Value) -> Step {
    let mut s = step(to);
    s.act.carrier = json!({"kind":"capability","name":name,"args":args});
    s
}
fn nested_request(intent: Value, caps: Vec<String>) -> NestedRequest {
    NestedRequest {
        intent,
        selected_residue_refs: vec![],
        success_conditions: vec![json!("return the attributable local result")],
        capabilities: caps,
        scope: json!("whole"),
        extension: None,
        modulation: None,
    }
}
fn label(c: &Circuit) -> &str {
    if c.face == "conjugate" {
        "conjugate"
    } else if c.parent_id.is_some() {
        "child"
    } else {
        "root"
    }
}
struct FixturePolicy {
    id: String,
    steps: BTreeMap<String, VecDeque<Step>>,
    current: BTreeMap<String, Step>,
    determinations: usize,
    conjugate_saw_failed_tests: bool,
    initial_state: String,
}
impl Policy for FixturePolicy {
    fn next_act(&mut self, cx: &PolicyContext<'_>, _: &mut dyn RuntimeHost) -> Result<Option<Act>> {
        let s = self
            .steps
            .get_mut(label(cx.circuit))
            .and_then(VecDeque::pop_front);
        if let Some(s) = &s {
            self.current.insert(cx.circuit.id.clone(), s.clone());
        }
        Ok(s.map(|s| s.act))
    }
    fn interpret(
        &mut self,
        cx: &PolicyContext<'_>,
        act: &Act,
        difference: &Value,
        _: &mut dyn RuntimeHost,
    ) -> Result<Interpretation> {
        if cx.circuit.face == "conjugate"
            && act.carrier["name"] == "run_tests"
            && difference["raw_result"]["ok"] == false
        {
            self.conjugate_saw_failed_tests = true;
        }
        let destination = self.current[&cx.circuit.id].to;
        Ok(Interpretation {
            destination,
            rationale: json!("explicit source-locked scenario transition"),
            witness: json!({"scenario":self.id,"operation_success":difference["operation_success"],"ambiguous_requirement":self.id=="QLDR-005"}),
            create: vec![
                json!({"position":destination,"value":difference,"provenance":{"scenario":self.id,"scripted_interpretation":true}}),
            ],
            revise: vec![],
            invalidate: vec![],
        })
    }
    fn determine(
        &mut self,
        cx: &PolicyContext<'_>,
        _: &mut dyn RuntimeHost,
    ) -> Result<Option<Determination>> {
        let root = cx.circuit.parent_id.is_none();
        if root {
            self.determinations += 1;
        }
        let conjugate = root && self.id == "QLDR-008" && self.determinations == 1;
        Ok(Some(Determination{synthesis:json!({"scenario":self.id,"conjugate_saw_failed_tests":self.conjugate_saw_failed_tests,"standing":"scripted determination"}),
            requested_outcome:if conjugate{Outcome::Conjugate}else{Outcome::Close},
            claimed_subject:if root&&self.id=="QLDR-006"{Some(self.id.clone())}else{None},
            claimed_state:if root&&self.id=="QLDR-006"&&self.determinations==1{Some(self.initial_state.clone())}else{None},
            evidence_refs:vec![],evaluation_refs:cx.circuit.residues.iter().filter(|r|r.kind=="evaluation"&&!r.invalidated).map(|r|r.id.clone()).collect(),unresolved_refs:vec![],
            nested:conjugate.then(||nested_request(json!({"question":"Does the actual current source satisfy the stated sum contract?","candidate":"sum is correct"}),vec!["read_file".into(),"run_tests".into()]))}))
    }
    fn evaluate_closure(
        &mut self,
        cx: &PolicyContext<'_>,
        _: &Determination,
        inspection: Option<&Inspection>,
        _: &mut dyn RuntimeHost,
    ) -> Result<Verdict> {
        if self.id == "QLDR-004" && cx.circuit.parent_id.is_none() && self.determinations == 1 {
            return Ok(Verdict::Reopen {
                destination: 1,
                rationale: json!(
                    "the failed operation requires current material, not a successful exit claim"
                ),
            });
        }
        Ok(Verdict::Close {
            task_success: if inspection.is_some_and(|i| i.objective_checks_pass) {
                "true"
            } else {
                "unknown"
            }
            .into(),
            rationale: json!("controlled review verdict; not human acceptance"),
        })
    }
    fn conjugate_delta(
        &mut self,
        _: &Circuit,
        _: &Value,
        _: &mut dyn RuntimeHost,
    ) -> Result<Value> {
        if self.conjugate_saw_failed_tests {
            Ok(
                json!({"status":"reopen","target_position":3,"reason":"actual tests in the fresh context found a form defect"}),
            )
        } else {
            Err(Error::new(
                "readiness defect was not observed by the fresh conjugate execution",
            ))
        }
    }
}
struct CurrentWorldInspection {
    id: String,
    world: World,
    node: Option<PathBuf>,
    expected: BTreeMap<String, String>,
    protected: BTreeMap<String, String>,
    test_files: Vec<String>,
}
impl Inspector for CurrentWorldInspection {
    fn inspect(&mut self, c: &Circuit, _: &Determination) -> Result<Option<Inspection>> {
        if c.parent_id.is_some() || self.expected.is_empty() && self.test_files.is_empty() {
            return Ok(None);
        }
        let after = self.world.snapshot()?;
        let expected = self.expected.iter().all(|(p, s)| after[p] == json!(s));
        let sources = self.protected.iter().all(|(p, s)| after[p] == json!(s));
        let tests = if self.test_files.is_empty() {
            None
        } else {
            Some(tasks::run_tests(
                &self.world,
                self.node.as_deref(),
                &self.test_files,
            )?)
        };
        let pass = expected && sources && tests.as_ref().is_none_or(|t| t["ok"] == true);
        Ok(Some(Inspection {
            subject_ref: self.id.clone(),
            state_digest: stable_digest(&after),
            objective_checks_pass: pass,
            evidence: json!({"protocol":"current-world-and-protected-source-checks","expected_artifacts":expected,"protected_sources":sources,"tests":tests,"standing":"D"}),
        }))
    }
}
fn source(world: &World, name: &str, value: &str) -> Result<()> {
    world.write(name, value.as_bytes(), true).map(|_| ())
}
fn run_case(scenario: &Value, owner: &dyn FormalOwner, node: Option<&Path>) -> Result<Value> {
    let id = scenario["id"]
        .as_str()
        .ok_or_else(|| Error::new("readiness scenario id missing"))?;
    let directory = tempfile::tempdir().map_err(|e| Error::new(e.to_string()))?;
    let world = World::open(directory.path())?;
    let mut expected = BTreeMap::new();
    let mut protected = BTreeMap::new();
    let mut tests = vec![];
    let mut plans: BTreeMap<String, VecDeque<Step>> = BTreeMap::new();
    let root = match id {
        "QLDR-001" => vec![step(3), step(4), step(5)],
        "QLDR-002" | "QLDR-008" => {
            let code = "export function sum(a,b) { return a - b; }\n";
            let fixed = "export function sum(a,b) { return a + b; }\n";
            let check="import {test} from 'node:test'; import assert from 'node:assert/strict'; import {sum} from './sum.js'; test('current sum contract',()=>assert.equal(sum(3,2),5));\n";
            source(&world, "sum.js", code)?;
            source(&world, "package.json", "{\"type\":\"module\"}\n")?;
            source(&world, "sum.test.js", check)?;
            expected.insert("sum.js".into(), fixed.into());
            protected.insert("sum.test.js".into(), check.into());
            protected.insert("package.json".into(), "{\"type\":\"module\"}\n".into());
            tests.push("sum.test.js".into());
            if id == "QLDR-008" {
                plans.insert(
                    "conjugate".into(),
                    vec![
                        tool(1, "read_file", json!({"path":"sum.js"})),
                        tool(4, "run_tests", json!({"files":["sum.test.js"]})),
                        step(5),
                    ]
                    .into(),
                );
                vec![
                    step(4),
                    step(5),
                    tool(4, "write_file", json!({"path":"sum.js","content":fixed})),
                    step(5),
                ]
            } else {
                vec![
                    tool(1, "read_file", json!({"path":"sum.js"})),
                    tool(2, "write_file", json!({"path":"sum.js","content":fixed})),
                    tool(4, "run_tests", json!({"files":["sum.test.js"]})),
                    step(5),
                ]
            }
        }
        "QLDR-003" => {
            let helper = "export function normalize(x) { return x.trim().toLowerCase(); }\n";
            let index="import {normalize} from './helper.js'; export function key(x) { return normalize(x); }\n";
            let check="import {test} from 'node:test'; import assert from 'node:assert/strict'; import {key} from './index.js'; test('cross-file contract',()=>assert.equal(key(' A '),'a'));\n";
            source(
                &world,
                "helper.js",
                "export function normalize(x) { return x; }\n",
            )?;
            source(&world, "index.js", "export function key(x) { return x; }\n")?;
            source(&world, "package.json", "{\"type\":\"module\"}\n")?;
            source(&world, "index.test.js", check)?;
            expected.insert("helper.js".into(), helper.into());
            expected.insert("index.js".into(), index.into());
            protected.insert("index.test.js".into(), check.into());
            tests.push("index.test.js".into());
            vec![
                tool(1, "list_files", json!({})),
                step(3),
                tool(
                    2,
                    "write_file",
                    json!({"path":"helper.js","content":helper}),
                ),
                tool(3, "write_file", json!({"path":"index.js","content":index})),
                tool(4, "run_tests", json!({"files":["index.test.js"]})),
                step(5),
            ]
        }
        "QLDR-004" => {
            source(&world, "current.txt", "current material\n")?;
            expected.insert("current.txt".into(), "current material\n".into());
            vec![
                step(2),
                tool(4, "read_file", json!({"path":"missing.txt"})),
                step(5),
                tool(4, "read_file", json!({"path":"current.txt"})),
                step(5),
            ]
        }
        "QLDR-005" => vec![step(4), step(0), step(3), step(4), step(5)],
        "QLDR-006" => {
            source(&world, "evidence-a.txt", "before\n")?;
            source(&world, "evidence-b.txt", "independent current evidence\n")?;
            expected.insert("evidence-a.txt".into(), "after\n".into());
            protected.insert(
                "evidence-b.txt".into(),
                "independent current evidence\n".into(),
            );
            vec![
                tool(1, "read_file", json!({"path":"evidence-a.txt"})),
                tool(
                    4,
                    "write_file",
                    json!({"path":"evidence-a.txt","content":"after\n"}),
                ),
                step(5),
                tool(4, "read_file", json!({"path":"evidence-b.txt"})),
                step(5),
            ]
        }
        "QLDR-007" => {
            source(
                &world,
                "compatibility.json",
                "{\"wire_version\":1,\"compatible\":true}\n",
            )?;
            protected.insert(
                "compatibility.json".into(),
                "{\"wire_version\":1,\"compatible\":true}\n".into(),
            );
            expected = protected.clone();
            let mut nested = step(4);
            nested.act.carrier = json!({"kind":"child_circuit"});
            nested.act.nested = Some(nested_request(
                json!("Read the actual compatibility record and retain only its bounded finding"),
                vec!["read_file".into()],
            ));
            plans.insert(
                "child".into(),
                vec![
                    tool(1, "read_file", json!({"path":"compatibility.json"})),
                    step(4),
                    step(5),
                ]
                .into(),
            );
            vec![step(4), nested, step(5)]
        }
        _ => {
            return Err(Error::new(
                "readiness scenario has no native implementation",
            ))
        }
    };
    if !tests.is_empty() && node.is_none() {
        return Err(Error::new(
            "the selected readiness code specimen requires an explicit Node executable",
        ));
    }
    let before = world.snapshot()?;
    plans.insert("root".into(), root.into());
    let mut policy = FixturePolicy {
        id: id.into(),
        steps: plans,
        current: BTreeMap::new(),
        determinations: 0,
        conjugate_saw_failed_tests: false,
        initial_state: stable_digest(&before),
    };
    let mut inspector = CurrentWorldInspection {
        id: id.into(),
        world: World::open(directory.path())?,
        node: node.map(Path::to_path_buf),
        expected,
        protected,
        test_files: tests,
    };
    let mut host = ResearchHost::new(FixtureBody, world, node.map(Path::to_path_buf), 128, vec![])?;
    let stream_directory = tempfile::tempdir().map_err(|e| Error::new(e.to_string()))?;
    let store = JsonlStreamStore::new(stream_directory.path())?;
    let opening:OpenStream=serde_json::from_value(json!({"agent_session_ref":format!("session:fixture:{id}"),"stream_ref":format!("stream:fixture:{id}"),"actuation_ref":format!("act:fixture:{id}"),"agency_ref":"agency:readiness-fixture","world_binding_ref":format!("binding:fixture:{id}")})).map_err(|e|Error::new(e.to_string()))?;
    store.open(&opening)?;
    let attribution = serde_json::from_value(
        json!({"agent_ref":"agent:readiness-fixture","agency_ref":"agency:readiness-fixture"}),
    )
    .map_err(|e| Error::new(e.to_string()))?;
    let sink = StreamRuntimeObserver::new(
        store.clone(),
        opening.stream_ref.clone(),
        opening.actuation_ref.clone(),
        attribution,
    );
    let mut observer = RecordingObserver::with_sink(Box::new(sink));
    let caps = crate::execution::CAPABILITIES
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    let frame = json!({"id":id,"initiating_intent":scenario["prompt"],"success_conditions":scenario["review"],"available_capabilities":caps,"provenance":{"scenario":id,"evidence_class":"controlled-structural-readiness"}});
    let result = Engine::new(
        owner,
        &mut host,
        &mut policy,
        &mut inspector,
        &mut observer,
        Mode::Deep,
        ExternalRef::new(format!("trace:readiness:{id}"))?,
        CancellationToken::default(),
        Limits {
            max_steps: 40,
            max_depth: 2,
            max_contexts: 4,
            max_trace_bytes: 8 * 1024 * 1024,
            max_reentries: 0,
        },
        caps,
        vec![],
        vec![],
    )?
    .run(frame)?;
    let event_types = observer
        .events
        .iter()
        .map(|e| e.event_type.clone())
        .collect::<Vec<_>>();
    let has = |name: &str| event_types.iter().any(|t| t == name);
    let root = result
        .circuits
        .iter()
        .find(|c| c.parent_id.is_none())
        .ok_or_else(|| Error::new("readiness run omitted its parent circuit"))?;
    let specific = match id {
        "QLDR-001" => root.trajectory.len() == 3 && root.trajectory[0]["relation"] == "R03",
        "QLDR-002" | "QLDR-003" => result
            .closure
            .as_ref()
            .is_some_and(|v| v["inspection"]["objective_checks_pass"] == true),
        "QLDR-004" => {
            has("circuit_reopened")
                && observer.events.iter().any(|e| {
                    e.event_type == "return_received"
                        && e.payload["research"]["returned"]["operation_success"] == false
                })
        }
        "QLDR-005" => root.trajectory.iter().any(|t| t["relation"] == "R40"),
        "QLDR-006" => {
            has("closure_refused") && has("circuit_reopened") && policy.determinations == 2
        }
        "QLDR-007" => {
            has("child_started")
                && has("child_reintegrated")
                && result
                    .circuits
                    .iter()
                    .any(|c| c.parent_id.is_some() && c.closure_state == "closed")
        }
        "QLDR-008" => {
            has("conjugate_started")
                && has("conjugate_delta")
                && root.trajectory.iter().any(|t| t["relation"] == "R53")
                && policy.conjugate_saw_failed_tests
        }
        _ => false,
    };
    let persisted = store.load(&opening.stream_ref)?;
    let replayed = JsonlStreamStore::new(store.root())?.load(&opening.stream_ref)?;
    let persistence =
        persisted == replayed && persisted.fields().events.len() == result.evidence_refs.len();
    let passed = result.status == "completed" && specific && persistence;
    Ok(
        json!({"scenario":scenario,"passed":passed,"structural_claims_exercised":specific,"execution":result,
        "events":observer.events,"event_types":event_types,"world":{"before":before,"after":host.world.snapshot()?},
        "host_observations":host.observations,"jsonl_replay_identical":persistence,"stream_event_count":persisted.fields().events.len(),
        "evidence_class":"deterministic-structural-conformance","fixture_model":true,"provider_evidence":false,"owner_machine_evidence":false,"human_acceptance":false}),
    )
}
pub fn run(owner: &dyn FormalOwner, node: Option<&Path>) -> Result<Value> {
    let catalog: Value = serde_json::from_str(CATALOG).map_err(|e| Error::new(e.to_string()))?;
    let cases = catalog["cases"]
        .as_array()
        .ok_or_else(|| Error::new("readiness catalog missing"))?;
    if cases.len() != 8 {
        return Err(Error::new(
            "retained readiness catalog does not contain all eight scenarios",
        ));
    }
    let mut records = vec![];
    for c in cases {
        records.push(run_case(c, owner, node)?);
    }
    let passed = records.iter().filter(|r| r["passed"] == true).count();
    Ok(
        json!({"schema":"actuation.native-ql-readiness/v1","scenario_catalog_sha256":crate::evidence::bytes_digest(CATALOG.as_bytes()),
        "owner_basis":owner.basis(),"required":cases.len(),"passed":passed,"failed":cases.len()-passed,
        "structural_ready":passed==cases.len(),"capability_effect_evidence_ready":false,"records":records,
        "evidence_class":"deterministic-structural-conformance","provider_evidence":false,"owner_machine_evidence":false,"human_acceptance":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relational::FormalOwner;

    struct ScriptedOwner;
    impl FormalOwner for ScriptedOwner {
        fn invoke(&self, request: Value) -> Result<Value> {
            match request["operation"].as_str() {
                Some("vocabulary") => Ok(json!({"result":{
                    "positions":[{"position":0},{"position":1},{"position":2},
                                 {"position":3},{"position":4},{"position":5}],
                    "faces":["direct","conjugate"]}})),
                Some("classify-relation") => Ok(json!({"result":[]})),
                other => Err(Error::new(format!("scripted owner lacks {other:?}"))),
            }
        }
        fn basis(&self) -> Value {
            json!({"repository":"scripted","revision":"0".repeat(40)})
        }
    }

    fn node_path() -> Option<PathBuf> {
        std::env::var_os("NODE")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| {
                std::env::var_os("PATH").map(|paths| {
                    std::env::split_paths(&paths)
                        .map(|dir| dir.join("node"))
                        .find(|p| p.is_file())
                })?
            })
    }

    #[test]
    fn readiness_without_node_names_the_missing_executable() {
        let err = run(&ScriptedOwner, None).unwrap_err();
        assert!(err.to_string().contains("Node executable"));
    }

    #[test]
    fn all_eight_scenarios_pass_against_the_scripted_owner() {
        let Some(node) = node_path() else {
            eprintln!("skipping: no Node executable on PATH");
            return;
        };
        let out = run(&ScriptedOwner, Some(&node)).expect("readiness run");
        assert_eq!(out["schema"], json!("actuation.native-ql-readiness/v1"));
        assert_eq!(out["required"], json!(8));
        assert_eq!(out["failed"], json!(0));
        assert_eq!(out["structural_ready"], json!(true));
        for r in out["records"].as_array().unwrap() {
            let id = r["scenario"]["id"].as_str().unwrap().to_owned();
            assert_eq!(r["passed"], json!(true), "scenario {id}");
            assert_eq!(r["fixture_model"], json!(true));
            assert_eq!(r["provider_evidence"], json!(false));
            assert_eq!(r["human_acceptance"], json!(false));
            assert_eq!(r["jsonl_replay_identical"], json!(true), "{id}");
        }
        // Each structural claim is exercised specifically, not vacuously.
        let specific: Vec<bool> = out["records"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["structural_claims_exercised"] == json!(true))
            .collect();
        assert!(specific.iter().all(|s| *s));
        let catalog_sha = out["scenario_catalog_sha256"].as_str().unwrap();
        assert_eq!(catalog_sha.len(), 64);
        assert!(catalog_sha
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
    }
}
