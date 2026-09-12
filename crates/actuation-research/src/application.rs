//! Thin research application routing. Product/research semantics remain in the
//! libraries; this descriptor also drives command discovery and help.
use crate::{
    comparison, evidence, execution,
    owner::OwnerInstrument,
    prime,
    prime_run::{self, PrimeRunRequest},
    process::ProcessSpec,
    records::EpistemicRecord,
    store::EpistemicStore,
    tasks::Task,
    world::World,
    Error, Result,
};
use actuation_core::ExternalRef;
use actuation_runtime::{CancellationToken, StreamRuntimeObserver};
use actuation_stream::{JsonlStreamStore, OpenStream, StreamStore};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::path::PathBuf;
pub const OPERATIONS: &[&str] = &[
    "capabilities",
    "record.validate",
    "record.persist",
    "record.read",
    "record.list",
    "task.get",
    "task.setup",
    "task.verify",
    "task.freeze",
    "owner.invoke",
    "faculty.invoke",
    "prime.return",
    "prime.run",
    "classic.run",
    "direct.run",
    "deep.run",
    "run",
    "readiness.run",
    "comparison.run",
    "comparison.held",
    "comparison.assess",
    "comparison.mask",
    "comparison.review",
];
fn decode<T: DeserializeOwned>(v: &Value) -> Result<T> {
    serde_json::from_value(v.clone())
        .map_err(|e| Error::new(format!("invalid research request: {e}")))
}
fn text(v: &Value, key: &str) -> Result<String> {
    v[key]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| Error::new(format!("{key} requires text")))
}
fn world(v: &Value) -> Result<World> {
    World::open(text(v, "world")?)
}
fn owner(v: &Value) -> Result<OwnerInstrument> {
    OwnerInstrument::bind(decode::<ProcessSpec>(&v["process"])?, &text(v, "revision")?)
}
fn observer(v: &Value) -> Result<execution::RecordingObserver> {
    if v.is_null() {
        return Ok(execution::RecordingObserver::default());
    }
    let store = JsonlStreamStore::new(text(v, "root")?)?;
    let opening: OpenStream = decode(&v["opening"])?;
    store.open(&opening)?;
    let sink = StreamRuntimeObserver::new(
        store,
        opening.stream_ref,
        opening.actuation_ref,
        decode(&v["attribution"])?,
    );
    Ok(execution::RecordingObserver::with_sink(Box::new(sink)))
}
fn secrets(spec: &ProcessSpec) -> Vec<String> {
    let pattern = regex::Regex::new("(?i)(key|token|secret|password|authorization)").unwrap();
    spec.environment
        .iter()
        .filter(|(k, _)| pattern.is_match(k))
        .map(|(_, v)| v.clone())
        .collect()
}
pub fn capabilities() -> Value {
    json!({"schema":"actuation.research-capabilities/v1","version":env!("CARGO_PKG_VERSION"),"operations":OPERATIONS,"tasks":Task::ids(),"prime_conditions":prime::conditions(),"formal_owner":"EpiLogos/QL-MEF","ql_owner_required_for_generic_research":false,"node_required_for_generic_research":false,"specimen_runtime_requirements":{"selected_javascript_tasks":"explicit Node executable","prime":"explicit admitted native Prime executable","python_fixture":"tests only"},"model_body_protocols":["one-shot-json","sdk-jsonl"],"acceptance_pending":["consumer harmonisation merge standing: O:I source contract and ai-kit CAW builds must reach their accepted mains","owner-machine acceptance against the O:I development field (the physical return tranche)"],"evidence_claims":{"provider":"not-assessed","owner_machine":false,"human_acceptance":false}})
}
pub fn invoke(v: &Value) -> Result<Value> {
    let op = text(v, "operation")?;
    match op.as_str() {
        "capabilities" => Ok(capabilities()),
        "record.validate" => Ok(EpistemicRecord::read(&v["record"])?.into_value()),
        "record.persist" => Ok(serde_json::to_value(
            EpistemicStore::open(text(v, "root")?)?
                .persist(EpistemicRecord::read(&v["record"])?)?,
        )
        .map_err(|e| Error::new(e.to_string()))?),
        "record.read" => Ok(EpistemicStore::open(text(v, "root")?)?
            .read(&text(v, "record_ref")?)?
            .into_value()),
        "record.list" => Ok(json!(EpistemicStore::open(text(v, "root")?)?.list()?)),
        "task.get" => Ok(Task::get(&text(v, "task_id")?)?.candidate()),
        "task.setup" => {
            let task = Task::get(&text(v, "task_id")?)?;
            let world = world(v)?;
            task.setup(&world)?;
            Ok(
                json!({"task_id":task.id(),"task_revision":task.revision(),"workspace":world.snapshot()?}),
            )
        }
        "task.verify" => {
            let task = Task::get(&text(v, "task_id")?)?;
            let world = world(v)?;
            let node = v["node"].as_str().map(PathBuf::from);
            task.verify(&world, &v["before"], &world.snapshot()?, node.as_deref())
        }
        "task.freeze" => Ok(comparison::freeze_task(&Task::get(&text(v, "task_id")?)?)),
        "owner.invoke" => owner(&v["owner"])?.invoke(v["request"].clone()),
        "readiness.run" => {
            let o = owner(&v["owner"])?;
            let node = v["node"].as_str().map(PathBuf::from);
            crate::readiness::run(&o, node.as_deref())
        }
        "faculty.invoke" => crate::faculty::invoke(
            &PathBuf::from(text(v, "configuration")?),
            &v["request"],
            v["trace_ref"].as_str(),
            v["declared_locus_ref"].as_str(),
        ),
        "prime.return" => evidence::prime_return(&v["return"], v["require_schema"] == true),
        "prime.run" => {
            let req: PrimeRunRequest = decode(&v["request"])?;
            let world = world(v)?;
            let owner = if v["owner"].is_null() {
                None
            } else {
                Some(owner(&v["owner"])?)
            };
            let mut observation = observer(&v["stream"])?;
            let result = prime_run::run_prime(&req, &world, owner.as_ref(), &mut observation)?;
            Ok(
                json!({"result":result,"events":observation.events,"durable_stream":observation.durable()}),
            )
        }
        "classic.run" | "direct.run" | "deep.run" | "run" => run(v),
        "comparison.run" => comparison::run(v),
        "comparison.held" => Ok(comparison::compare_held_constant(
            v["records"]
                .as_array()
                .ok_or_else(|| Error::new("records requires array"))?,
        )),
        "comparison.assess" => Ok(comparison::assess(&v["manifest"])),
        "comparison.mask" => comparison::mask_mapping(&v["manifest"]),
        "comparison.review" => Ok(
            json!({"text":comparison::render_review(&v["manifest"],v["masked"].as_bool().ok_or_else(||Error::new("masked must be explicit"))?,&decode::<Vec<String>>(&v.get("secrets").cloned().unwrap_or(json!([])))?)?}),
        ),
        _ => Err(Error::new("unsupported research operation")),
    }
}

/// One application path admits the supplied body and condition. Alias commands
/// do not each grow their own host, World, evidence or provider implementation.
pub fn run(v: &Value) -> Result<Value> {
    use execution::{ModelBody, RunMode};
    let mode = match v["operation"].as_str() {
        Some("classic.run") => RunMode::Classic,
        Some("direct.run") => RunMode::Direct,
        Some("deep.run") => RunMode::Deep,
        _ => RunMode::parse(&text(v, "condition")?)?,
    };
    let task = Task::get(&text(v, "task_id")?)?;
    let trace = ExternalRef::new(text(v, "trace_ref")?)?;
    let world = world(v)?;
    let spec: ProcessSpec = decode(&v["body"]["process"])?;
    spec.validate()?;
    let secret_values = secrets(&spec);
    let fixture = v["body"]["fixture_provider"]
        .as_bool()
        .ok_or_else(|| Error::new("fixture_provider standing must be explicit"))?;
    if v["body"].get("protocol").is_some_and(|p| !p.is_string()) {
        return Err(Error::new("model body protocol must be text"));
    }
    let mut configuration = v["body"]["configuration"].clone();
    if v["body"]["protocol"] == "sdk-jsonl" {
        if !configuration.is_object() {
            return Err(Error::new("SDK configuration requires an object"));
        }
        if configuration
            .get("trace_ref")
            .is_some_and(|x| x != &json!(trace))
        {
            return Err(Error::new("SDK trace must name this research run"));
        }
        if let Some(path) = configuration.get("world") {
            let path = path
                .as_str()
                .ok_or_else(|| Error::new("SDK World requires a path"))?;
            if PathBuf::from(path).canonicalize().ok().as_deref() != Some(world.root()) {
                return Err(Error::new(
                    "SDK World must name this admitted research World",
                ));
            }
        }
        configuration["trace_ref"] = json!(trace);
        configuration["world"] = json!(world.root());
    }
    let body: Box<dyn ModelBody> = match v["body"]["protocol"].as_str() {
        None | Some("one-shot-json") => Box::new(execution::ProcessModelBody {
            process: spec,
            source_basis: v["body"]["source_basis"].clone(),
            fixture,
        }),
        Some("sdk-jsonl") => Box::new(crate::sdk::NativeSdkBody::new(
            spec,
            configuration,
            v["body"]["source_basis"].clone(),
            fixture,
            v["body"]["session_timeout_ms"]
                .as_u64()
                .ok_or_else(|| Error::new("SDK session timeout required"))?,
        )?),
        _ => return Err(Error::new("unsupported model body protocol")),
    };
    let steps = v["max_steps"]
        .as_u64()
        .filter(|n| (1..=10000).contains(n))
        .ok_or_else(|| Error::new("max_steps requires 1..10000"))?;
    let limits = if v["limits"].is_null() {
        crate::relational::Limits {
            max_steps: steps,
            ..Default::default()
        }
    } else {
        let l: crate::relational::Limits = decode(&v["limits"])?;
        if l.max_steps != steps {
            return Err(Error::new(
                "shared max_steps and relational limits disagree",
            ));
        }
        l
    };
    limits.validate()?;
    let bound_owner = if mode == RunMode::Classic {
        None
    } else {
        Some(owner(&v["owner"])?)
    };
    let calls = v["max_calls"]
        .as_u64()
        .ok_or_else(|| Error::new("max_calls required"))?;
    let mut host = execution::ResearchHost::new(
        body,
        world,
        v["node"].as_str().map(PathBuf::from),
        calls,
        secret_values,
    )?;
    let mut observation = observer(&v["stream"])?;
    let result = execution::run_task(
        &task,
        trace,
        mode,
        limits,
        bound_owner
            .as_ref()
            .map(|o| o as &dyn crate::relational::FormalOwner),
        &mut host,
        &mut observation,
        &CancellationToken::default(),
    )?;
    Ok(json!({"result":result,"events":observation.events,"durable_stream":observation.durable()}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_disclose_the_whole_surface() {
        let caps = capabilities();
        assert_eq!(caps["schema"], json!("actuation.research-capabilities/v1"));
        assert_eq!(caps["version"], json!(env!("CARGO_PKG_VERSION")));
        assert_eq!(caps["formal_owner"], json!("EpiLogos/QL-MEF"));
        assert_eq!(caps["ql_owner_required_for_generic_research"], json!(false));
        let ops = caps["operations"].as_array().unwrap();
        assert_eq!(ops.len(), OPERATIONS.len());
        for required in [
            "record.validate",
            "classic.run",
            "deep.run",
            "prime.run",
            "comparison.review",
        ] {
            assert!(ops.iter().any(|o| o == required), "{required}");
        }
        // The disclosed tasks and conditions are exactly the frozen catalogues.
        assert_eq!(caps["tasks"].as_array().unwrap().len(), 8);
        assert_eq!(
            caps["prime_conditions"]["prime-native"]["code"],
            json!("P0")
        );
        assert_eq!(caps["evidence_claims"]["provider"], json!("not-assessed"));
        assert_eq!(caps["evidence_claims"]["human_acceptance"], json!(false));
    }

    #[test]
    fn invoke_routes_record_operations_over_a_dedicated_store() {
        let dir = tempfile::tempdir().unwrap();
        let record = crate::records::EpistemicRecord::read(&json!({
            "schema": crate::records::EPISTEMIC_RECORD_SCHEMA,
            "record_kind": "EpistemicCorpus",
            "record_ref": "record:routed/1",
            "recorded_at": "2026-09-11T00:00:00Z",
            "provenance": {"kind":"source","actor_ref":null,"generator_ref":null,"source_refs":["source:1"],"derivation_refs":[]},
            "access": {
                "behavioural": {"state":"available","method_refs":["m:1"],"evidence_refs":["e:1"]},
                "output_state": {"state":"available","method_refs":["m:1"],"evidence_refs":["e:1"]},
                "internal_read": {"state":"unavailable","reason":"none","method_refs":[],"evidence_refs":[]},
                "internal_write": {"state":"unavailable","reason":"none","method_refs":[],"evidence_refs":[]},
                "causal": {"state":"unavailable","reason":"none","method_refs":[],"evidence_refs":[]},
                "learning": {"state":"not-assessed","reason":"not attempted","method_refs":[],"evidence_refs":[]}
            },
            "subject_refs": ["subject:1"],
            "model_ref": null,
            "checkpoint_ref": null,
            "method_refs": [],
            "coordinate_refs": [],
            "derivation_refs": [],
            "evidence_refs": [],
            "payload": {"corpus_ref":"corpus:1","revision_ref":"rev:1","item_refs":["item:1"]}
        }))
        .unwrap();
        let root = dir.path().to_string_lossy().into_owned();
        let out =
            invoke(&json!({"operation":"record.persist","root":root,"record":record.as_value()}))
                .unwrap();
        assert_eq!(out["deduplicated"], json!(false));
        let out =
            invoke(&json!({"operation":"record.read","root":root,"record_ref":"record:routed/1"}))
                .unwrap();
        assert_eq!(out["record_ref"], json!("record:routed/1"));
        let out = invoke(&json!({"operation":"record.list","root":root})).unwrap();
        assert_eq!(out.as_array().unwrap().len(), 1);
        invoke(&json!({"operation":"record.validate","record":record.as_value()})).unwrap();

        let dir2 = tempfile::tempdir().unwrap();
        let world = dir2.path().join("world");
        std::fs::create_dir(&world).unwrap();
        let out = invoke(&json!({
            "operation":"task.setup","task_id":"S1-RESTRAINT-001","world":world.to_string_lossy()
        }))
        .unwrap();
        assert_eq!(out["task_id"], json!("S1-RESTRAINT-001"));
        assert!(out["workspace"]["fact.txt"].is_string());
        assert!(invoke(&json!({"operation":"mystery.op"})).is_err());
        assert!(invoke(&json!({"operation":"task.get","task_id":"absent"})).is_err());
    }
}
