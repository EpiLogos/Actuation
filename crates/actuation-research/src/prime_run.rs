//! First-class Prime experiment execution over the native process, evidence,
//! World and Actuation observer seams. Prime remains a supplied native body.
use crate::{
    evidence::{extract_prime_family, sanitize, stable_digest},
    execution::observe,
    owner::OwnerInstrument,
    prime::{condition_prompt, get_condition, SourceLock},
    process::{ProcessSpec, RpcClient},
    tasks::Task,
    world::World,
    Error, Result,
};
use actuation_core::ExternalRef;
use actuation_runtime::RuntimeObserver;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrimeRunRequest {
    pub trace_ref: ExternalRef,
    pub task_id: String,
    pub condition: String,
    pub source_lock: Value,
    pub prime: ProcessSpec,
    pub provider: String,
    pub model: String,
    pub fixture_provider: bool,
    pub allow_refinement: bool,
    #[serde(default)]
    pub node: Option<PathBuf>,
    #[serde(default)]
    pub skill_path: Option<PathBuf>,
    #[serde(default)]
    pub research_binary: Option<PathBuf>,
    #[serde(default)]
    pub faculty_config: Option<PathBuf>,
}
const REFINEMENT:&str="Using only evidence from the completed trajectory, retain one small reusable improvement that would help a future similar task. Prefer a focused memory, supplemental prompt note, skill description, or subagent specification. Preserve uncertainty and do not generalise beyond the evidence.";
fn task_snapshot(world: &World) -> Result<Value> {
    let mut v = world.snapshot()?;
    v.as_object_mut()
        .unwrap()
        .retain(|k, _| !k.starts_with(".prime/"));
    Ok(v)
}
/// Call with a dedicated empty World. No existing personal Control is touched.
/// `owner` is required for relational conditions; not a local formal fallback.
pub fn run_prime(
    request: &PrimeRunRequest,
    world: &World,
    owner: Option<&OwnerInstrument>,
    observer: &mut dyn RuntimeObserver,
) -> Result<Value> {
    request.prime.validate()?;
    let task = Task::get(&request.task_id)?;
    let condition = get_condition(&request.condition)?;
    let lock = SourceLock::read(&request.source_lock)?;
    if condition["continual"].as_bool() != Some(request.allow_refinement) {
        return Err(Error::new(
            "refinement authority must exactly match the chosen research condition",
        ));
    }
    if request.provider.trim().is_empty() || request.model.trim().is_empty() {
        return Err(Error::new(
            "Prime model/provider must be explicitly supplied, not resolved here",
        ));
    }
    let expected = lock.as_value()["prime_agent"]["release"]
        .as_str()
        .unwrap()
        .trim_start_matches('v');
    let mut probe = request.prime.clone();
    probe.cwd = world.root().to_owned();
    probe.args.push("--version".into());
    probe.timeout_ms = probe.timeout_ms.min(10_000);
    let version = probe.run(b"")?;
    let observed = if version.stdout.trim().is_empty() {
        version.stderr.trim()
    } else {
        version.stdout.trim()
    };
    let matches = regex::Regex::new(&format!(r"(?:^|\s)v?{}(?:$|\s)", regex::escape(expected)))
        .unwrap()
        .is_match(observed);
    if version.code != Some(0) || !matches {
        return Err(Error::new(
            "Prime executable version does not match the admitted source lock",
        ));
    }
    let ql = if condition["relational"] == true {
        let owner = owner.ok_or_else(|| {
            Error::new("relational condition requires the supplied QL owner instrument")
        })?;
        let basis = owner.basis();
        if basis["revision"] != lock.as_value()["ql_mef"]["accepted_main_revision"] {
            return Err(Error::new(
                "QL instrument does not match the active source lock",
            ));
        }
        for path in [
            &request.skill_path,
            &request.research_binary,
            &request.faculty_config,
        ] {
            if !path.as_ref().is_some_and(|p| p.is_absolute() && p.exists()) {
                return Err(Error::new("relational condition requires explicit existing skill, native faculty binary and configuration"));
            }
        }
        owner.invoke(json!({"operation":"capabilities"}))?;
        basis
    } else {
        json!({"status":"not-used"})
    };
    let faculty = if condition["relational"] == true {
        let config = crate::faculty::FacultyConfig::load(request.faculty_config.as_ref().unwrap())?;
        if config.bind()?.basis() != ql {
            return Err(Error::new(
                "inherited faculty does not use the driver-bound QL owner",
            ));
        }
        Some(config)
    } else {
        None
    };
    let prior_faculty = faculty
        .as_ref()
        .and_then(|c| c.evidence_root.as_ref())
        .map(|root| crate::faculty::receipts(root))
        .transpose()?
        .unwrap_or_default();
    // Reject prefix arguments that would override the driver's protocol/World.
    if request.prime.args.iter().any(|a| {
        [
            "--mode",
            "--cwd",
            "--session-dir",
            "--skill",
            "--provider",
            "--model",
        ]
        .iter()
        .any(|p| a == p || a.starts_with(&format!("{p}=")))
    }) {
        return Err(Error::new(
            "Prime launcher prefix cannot override research-owned arguments",
        ));
    }
    task.setup(world)?;
    let before = world.snapshot()?;
    let settings = json!({"autoRefine":{"enabled":false}});
    world.write(
        ".prime/agent/settings.json",
        format!("{settings}\n").as_bytes(),
        true,
    )?;
    let sessions =
        tempfile::tempdir().map_err(|_| Error::new("cannot establish isolated Prime sessions"))?;
    let mut process = request.prime.clone();
    process.cwd = world.root().to_owned();
    process.args.extend([
        "--mode".into(),
        "rpc".into(),
        "--cwd".into(),
        world.root().to_string_lossy().into_owned(),
        "--no-extensions".into(),
        "--no-prompt-templates".into(),
        "--no-context-files".into(),
        "--no-skills".into(),
        "--offline".into(),
        "--provider".into(),
        request.provider.clone(),
        "--model".into(),
        request.model.clone(),
        "--session-dir".into(),
        sessions.path().to_string_lossy().into_owned(),
    ]);
    process
        .environment
        .insert("RLM_MAX_DEPTH".into(), condition["maxDepth"].to_string());
    process
        .environment
        .insert("DO_NOT_TRACK".into(), "1".into());
    if condition["relational"] == true {
        process.args.extend([
            "--skill".into(),
            request
                .skill_path
                .as_ref()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ]);
        process.environment.insert(
            "ACTUATION_RESEARCH_BIN".into(),
            request
                .research_binary
                .as_ref()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        );
        process.environment.insert(
            "ACTUATION_RESEARCH_FACULTY_CONFIG".into(),
            request
                .faculty_config
                .as_ref()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        );
        process.environment.insert(
            "ACTUATION_RESEARCH_TRACE_REF".into(),
            request.trace_ref.to_string(),
        );
    }
    let secrets = request
        .prime
        .environment
        .iter()
        .filter(|(k, _)| {
            regex::Regex::new("(?i)(key|token|secret|password|authorization)")
                .unwrap()
                .is_match(k)
        })
        .map(|(_, v)| v.clone())
        .collect::<Vec<_>>();
    let mut sequence = 0;
    let mut evidence = vec![];
    let runtime = "prime-research";
    evidence.push(observe(observer,&request.trace_ref,runtime,&mut sequence,"prime_launch_requested",json!({"condition":condition,"source_lock_digest":stable_digest(lock.as_value()),"ql_owner":ql,"fixture_provider":request.fixture_provider}))?);
    world.verify_root()?;
    let mut client = RpcClient::start(process)?;
    let start = Instant::now();
    let total = Duration::from_millis(request.prime.timeout_ms);
    let remaining = || {
        total
            .checked_sub(start.elapsed())
            .ok_or_else(|| Error::new("Prime research execution deadline exhausted"))
    };
    let mut after_task = Value::Null;
    let mut after_refinement = Value::Null;
    let mut verification = Value::Null;
    let mut refinement_verification = Value::Null;
    let mut output = Value::Null;
    let mut messages = Value::Null;
    let mut stats = Value::Null;
    let mut final_state = Value::Null;
    let mut refinement = Value::Null;
    let execution = (|| -> Result<()> {
        let initial = client.request(json!({"type":"get_state"}), remaining()?)?["data"].clone();
        if initial["isStreaming"] != false {
            return Err(Error::new(
                "new Prime session must disclose an idle initial state",
            ));
        }
        evidence.push(observe(
            observer,
            &request.trace_ref,
            runtime,
            &mut sequence,
            "prime_started",
            sanitize(
                &json!({"observed_version":observed,"initial_state":initial}),
                &secrets,
            ),
        )?);
        let prompt = condition_prompt(&condition, &task.candidate())?;
        client.request(json!({"type":"prompt","message":prompt}), remaining()?)?;
        evidence.push(observe(
            observer,
            &request.trace_ref,
            runtime,
            &mut sequence,
            "prime_prompt_acknowledged",
            json!({"prompt_digest":stable_digest(&json!(prompt))}),
        )?);
        final_state = client.wait_idle(remaining()?)?;
        output = client.request(json!({"type":"get_last_assistant_text"}), remaining()?)?["data"]
            ["text"]
            .clone();
        messages = client.request(json!({"type":"get_messages"}), remaining()?)?["data"]
            ["messages"]
            .clone();
        stats = client.request(json!({"type":"get_session_stats"}), remaining()?)?["data"].clone();
        world.verify_root()?;
        after_task = task_snapshot(world)?;
        verification = task.verify(world, &before, &after_task, request.node.as_deref())?;
        evidence.push(observe(observer,&request.trace_ref,runtime,&mut sequence,"prime_trajectory_collected",sanitize(&json!({"outcome":output,"verification":verification,"family":extract_prime_family(client.records())}),&secrets))?);
        if request.allow_refinement {
            remaining()?;
            evidence.push(observe(observer,&request.trace_ref,runtime,&mut sequence,"prime_refinement_requested",json!({"authority":"explicit-selected-P5-condition","max_passes":1,"trajectory_completed":true}))?);
            refinement = client.refine_once_bounded(true, true, REFINEMENT, remaining()?)?;
            evidence.push(observe(
                observer,
                &request.trace_ref,
                runtime,
                &mut sequence,
                "prime_refinement_returned",
                sanitize(&refinement, &secrets),
            )?);
            after_refinement = task_snapshot(world)?;
            refinement_verification =
                task.verify(world, &before, &after_refinement, request.node.as_deref())?;
        }
        let control: Value = serde_json::from_str(&world.text(".prime/agent/settings.json")?)
            .map_err(|_| Error::new("Prime project settings became unreadable"))?;
        if control["autoRefine"]["enabled"] != false {
            return Err(Error::new(
                "Prime automatic refinement suppression changed during execution",
            ));
        }
        Ok(())
    })();
    client.stop();
    world.verify_root()?;
    let family = extract_prime_family(client.records());
    let acceptance = task.candidate()["primeAcceptance"].clone();
    let acceptance_result = if acceptance.is_object() {
        json!({"required":acceptance,"observed_child_loci":family["child_nodes"].as_array().unwrap().len(),"observed_nested_edges":family["nested_edges"].as_array().unwrap().len(),"pass":family["child_nodes"].as_array().unwrap().len() as u64>=acceptance["minChildLoci"].as_u64().unwrap_or(u64::MAX)&&(acceptance["requireNestedChild"]!=true||!family["nested_edges"].as_array().unwrap().is_empty())})
    } else {
        Value::Null
    };
    let faculty_result = (|| -> Result<Option<Vec<Value>>> {
        let Some(root) = faculty.as_ref().and_then(|c| c.evidence_root.as_ref()) else {
            return Ok(None);
        };
        let current = crate::faculty::receipts(root)?;
        for (name, prior) in &prior_faculty {
            if current.get(name) != Some(prior) {
                return Err(Error::new(
                    "previous faculty evidence changed during Prime execution",
                ));
            }
        }
        let mut retained = vec![];
        for (name, receipt) in current {
            if prior_faculty.contains_key(&name) {
                continue;
            }
            if receipt["trace_ref"] != request.trace_ref.to_string() || receipt["owner_basis"] != ql
            {
                return Err(Error::new(
                    "new faculty receipt belongs to another trace or owner",
                ));
            }
            retained.push(receipt);
        }
        Ok(Some(retained))
    })();
    let faculty_error = faculty_result.as_ref().err().map(|e| e.to_string());
    let faculty_records = faculty_result.ok().flatten();
    let faculty_exercised = if condition["relational"] != true {
        json!(false)
    } else {
        faculty_records
            .as_ref()
            .map(|rows| json!(rows.iter().any(|r| r["success"] == true)))
            .unwrap_or(Value::Null)
    };
    let status = if execution.is_ok() && faculty_error.is_none() {
        "completed"
    } else {
        "failed"
    };
    let error = execution
        .err()
        .map(|e| e.to_string())
        .or_else(|| faculty_error.clone());
    evidence.push(observe(
        observer,
        &request.trace_ref,
        runtime,
        &mut sequence,
        if error.is_some() {
            "prime_run_failed"
        } else {
            "prime_run_completed"
        },
        sanitize(
            &json!({"error":error,"refinement_attempted":client.refinement_attempted()}),
            &secrets,
        ),
    )?);
    let record = json!({"schema":"actuation.prime-recursive-experiment/v1","execution_status":status,"error":error,"condition":condition,"task":task.candidate(),"source":{"lock":lock.as_value(),"ql_owner":ql,"task_revision":task.revision()},"prime":{"observed_version":observed,"provider":request.provider,"model":request.model,"selection_standing":"supplied-not-resolved-by-Actuation","final_state":final_state,"session_stats":stats,"messages":messages,"requested_rlm_max_depth":condition["maxDepth"],"family":family,"prime_acceptance":acceptance_result,"rpc_records":client.records(),"stderr":client.stderr_text()?},"workspace":{"before":before,"after":after_task,"after_refinement":after_refinement,"excluded_generated_prefix":".prime/"},"faculty":{"receipts":faculty_records,"collection_error":faculty_error,"standing":"native-invocation-receipts; caller-locus-labels-not-authenticated"},"outcome":output,"verification":verification,"continual_refinement":refinement,"refinement_verification":refinement_verification,"evidence_refs":evidence,"claims":{"fixture_provider":request.fixture_provider,"live_prime_run":if request.fixture_provider{json!(false)}else{Value::Null},"prime_body_executed":true,"ql_relational_faculty_exercised":faculty_exercised,"observed_child_loci":family["child_nodes"].as_array().unwrap().len(),"observed_lineage_edges":family["edges"].as_array().unwrap().len(),"observed_nested_child_edges":family["nested_edges"].as_array().unwrap().len(),"continual_refinement_invoked":client.refinement_attempted(),"provider_evidence":"not-assessed","owner_machine_evidence":false,"human_acceptance":false}});
    Ok(sanitize(&record, &secrets))
}
