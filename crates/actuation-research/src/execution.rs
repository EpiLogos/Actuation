//! Native Actuation runtime composition for research. A model body is supplied
//! by its owner; this module never chooses providers, models or Harnesses.
use crate::{
    evidence::{candidate_boundary, sanitize},
    process::ProcessSpec,
    tasks,
    world::World,
    Error, Result,
};
use actuation_core::ExternalRef;
use actuation_runtime::*;
use serde_json::{json, Value};
use std::{
    future::Future,
    path::PathBuf,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};
/// The current adapters are synchronous/bounded. This executor also permits a
/// supplied asynchronous library host to wake the thread without busy polling.
pub fn block_on<F: Future>(future: F) -> F::Output {
    struct ThreadWake(std::thread::Thread);
    impl Wake for ThreadWake {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }
    let w = Waker::from(Arc::new(ThreadWake(std::thread::current())));
    let mut cx = Context::from_waker(&w);
    let mut future = std::pin::pin!(future);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => std::thread::park(),
        }
    }
}
/// Optional canonical sink is invoked before acknowledging the event. Without
/// it these refs belong to the retained research receipt, NOT a durable stream.
#[derive(Default)]
pub struct RecordingObserver {
    pub events: Vec<LoopEvent>,
    sink: Option<Box<dyn RuntimeObserver>>,
}
impl RecordingObserver {
    pub fn with_sink(sink: Box<dyn RuntimeObserver>) -> Self {
        Self {
            events: vec![],
            sink: Some(sink),
        }
    }
    pub fn durable(&self) -> bool {
        self.sink.is_some()
    }
}
impl RuntimeObserver for RecordingObserver {
    fn emit(&mut self, event: &LoopEvent) -> Result<ExternalRef> {
        let r = if let Some(s) = self.sink.as_mut() {
            s.emit(event)?
        } else {
            ExternalRef::new(event.event_id.clone())?
        };
        self.events.push(event.clone());
        Ok(r)
    }
}
pub fn observe(
    observer: &mut dyn RuntimeObserver,
    trace: &ExternalRef,
    runtime: &str,
    sequence: &mut u64,
    event_type: &str,
    payload: Value,
) -> Result<ExternalRef> {
    let e = LoopEvent {
        channel: "runtime".into(),
        event_id: format!("{trace}:{runtime}:{}", *sequence),
        event_type: event_type.into(),
        run_id: trace.clone(),
        sequence: *sequence,
        runtime: runtime.into(),
        payload,
    };
    let r = observer.emit(&e)?;
    *sequence += 1;
    Ok(r)
}
/// A language-native specimen returns content/tool calls or structured control.
/// Tests supply a deterministic specimen; a fixture is never promoted to P.
pub trait ModelBody: Send {
    fn complete(&mut self, request: &Value) -> Result<Value>;
    fn basis(&self) -> Value;
    /// Finish a supplied body without losing the run when evidence collection
    /// fails. Unsupported session evidence is not fabricated as a successful run.
    fn finish(&mut self, _inspection: &Value) -> Value {
        json!({"status":"not-applicable","session_evidence":null})
    }
}
impl<T: ModelBody + ?Sized> ModelBody for Box<T> {
    fn complete(&mut self, request: &Value) -> Result<Value> {
        (**self).complete(request)
    }
    fn basis(&self) -> Value {
        (**self).basis()
    }
    fn finish(&mut self, inspection: &Value) -> Value {
        (**self).finish(inspection)
    }
}
#[derive(Clone, Debug)]
pub struct ProcessModelBody {
    pub process: ProcessSpec,
    pub source_basis: Value,
    pub fixture: bool,
}
impl ModelBody for ProcessModelBody {
    fn complete(&mut self, request: &Value) -> Result<Value> {
        candidate_boundary(request)?;
        let r = self.process.run(request.to_string().as_bytes())?;
        if r.code != Some(0) {
            return Err(Error::new("supplied model specimen failed"));
        }
        let value: Value = serde_json::from_str(&r.stdout)
            .map_err(|_| Error::new("model specimen returned invalid JSON"))?;
        if !value.is_object() {
            return Err(Error::new("model specimen response must be an object"));
        }
        Ok(value)
    }
    fn basis(&self) -> Value {
        json!({"source":self.source_basis,"fixture_provider":self.fixture,"provider_mode":if self.fixture{"fixture"}else{"supplied-body"}})
    }
}
pub const CAPABILITIES: &[&str] = &["list_files", "read_file", "write_file", "run_tests"];
pub struct ResearchHost<B: ModelBody> {
    pub body: B,
    pub world: World,
    pub node: Option<PathBuf>,
    pub observations: Vec<Value>,
    pub model_calls: u64,
    pub capability_calls: u64,
    max_calls: u64,
    secrets: Vec<String>,
}
impl<B: ModelBody> ResearchHost<B> {
    pub fn new(
        body: B,
        world: World,
        node: Option<PathBuf>,
        max_calls: u64,
        secrets: Vec<String>,
    ) -> Result<Self> {
        if max_calls == 0 || max_calls > 10000 {
            return Err(Error::new("invalid research host call budget"));
        }
        Ok(Self {
            body,
            world,
            node,
            observations: vec![],
            model_calls: 0,
            capability_calls: 0,
            max_calls,
            secrets,
        })
    }
    fn budget(&self, cancel: &CancellationToken) -> Result<()> {
        if cancel.requested() {
            return Err(Error::new("research cancelled"));
        }
        if self.model_calls + self.capability_calls >= self.max_calls {
            return Err(Error::new("research host call budget exhausted"));
        }
        self.world.verify_root()
    }
    fn record(&mut self, kind: &str, value: Value) {
        self.observations
            .push(json!({"event_type":kind,"value":sanitize(&value,&self.secrets)}));
    }
    fn model(&mut self, call: HostCall) -> Result<Value> {
        self.budget(&call.cancellation)?;
        let payload = json!({"request":call.request.wire(),"payload":call.payload,"capabilities":CAPABILITIES});
        candidate_boundary(&payload)?;
        self.model_calls += 1;
        self.record("model_requested", payload.clone());
        let result = self.body.complete(&payload);
        match &result {
            Ok(v) => self.record("model_returned", json!({"ok":true,"result":v})),
            Err(e) => self.record("model_returned", json!({"ok":false,"error":e.to_string()})),
        };
        self.world.verify_root()?;
        result
    }
    fn capability(&mut self, call: HostCall) -> Result<Value> {
        self.budget(&call.cancellation)?;
        self.capability_calls += 1;
        let name = call
            .payload
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::new("capability name required"))?;
        let args = call.payload.get("args").cloned().unwrap_or(json!({}));
        self.record("capability_requested", json!({"name":name,"args":args}));
        let result = (|| match name {
            "list_files" => Ok(json!(self
                .world
                .list(args["path"].as_str().unwrap_or("."))?)),
            "read_file" => Ok(json!(self.world.text(
                args["path"]
                    .as_str()
                    .ok_or_else(|| Error::new("read_file path required"))?
            )?)),
            "write_file" => {
                let path = args["path"]
                    .as_str()
                    .ok_or_else(|| Error::new("write_file path required"))?;
                let content = args["content"]
                    .as_str()
                    .ok_or_else(|| Error::new("write_file content required"))?;
                self.world.write(path, content.as_bytes(), false)?;
                Ok(json!({"path":path,"written":true}))
            }
            "run_tests" => {
                let files = match args.get("files") {
                    None | Some(Value::Null) => vec![],
                    Some(Value::Array(a)) => a
                        .iter()
                        .map(|v| {
                            v.as_str()
                                .map(str::to_owned)
                                .ok_or_else(|| Error::new("test file requires string"))
                        })
                        .collect::<Result<Vec<_>>>()?,
                    _ => return Err(Error::new("files requires array")),
                };
                tasks::run_tests(&self.world, self.node.as_deref(), &files)
            }
            _ => Err(Error::new(
                "capability is not supplied by this research World",
            )),
        })();
        self.record(
            "capability_returned",
            match &result {
                Ok(v) => json!({"name":name,"ok":true,"result":v}),
                Err(e) => json!({"name":name,"ok":false,"error":e.to_string()}),
            },
        );
        result
    }
}
impl<B: ModelBody> RuntimeHost for ResearchHost<B> {
    fn call_model<'a>(&'a mut self, call: HostCall) -> HostFuture<'a> {
        Box::pin(async move { self.model(call).map_err(Into::into) })
    }
    fn execute_capability<'a>(&'a mut self, call: HostCall) -> HostFuture<'a> {
        Box::pin(async move { self.capability(call).map_err(Into::into) })
    }
    fn receive_external_input<'a>(&'a mut self, _call: HostCall) -> HostFuture<'a> {
        Box::pin(async { Ok(Value::Null) })
    }
    fn read_context<'a>(&'a mut self, call: HostCall) -> HostFuture<'a> {
        Box::pin(async move {
            self.world.verify_root()?;
            Ok(json!({"workspace":self.world.snapshot()?,"input":call.payload.get("input")}))
        })
    }
}

/// Sanitize before any durable sink receives model/specimen output. This is
/// explicit configured-secret protection, not a claim to discover all secrets.
pub struct RedactingObserver<'a> {
    pub inner: &'a mut dyn RuntimeObserver,
    pub secrets: &'a [String],
}
impl RuntimeObserver for RedactingObserver<'_> {
    fn emit(&mut self, event: &LoopEvent) -> Result<ExternalRef> {
        let mut clean = event.clone();
        clean.payload = sanitize(&clean.payload, self.secrets);
        self.inner.emit(&clean)
    }
}
/// Conditions share one executable lifecycle, one World and one model/capability
/// host. Only the recurrence differs. None of these conditions selects a model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunMode {
    Classic,
    Direct,
    Deep,
}
impl RunMode {
    pub fn parse(name: &str) -> Result<Self> {
        match name {
            "classic" => Ok(Self::Classic),
            "ql-direct" => Ok(Self::Direct),
            "ql-deep" => Ok(Self::Deep),
            _ => Err(Error::new("unknown research condition")),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Direct => "ql-direct",
            Self::Deep => "ql-deep",
        }
    }
}
struct CheckedObserver<'a> {
    events: Vec<LoopEvent>,
    inner: &'a mut dyn RuntimeObserver,
    failed: bool,
}
impl RuntimeObserver for CheckedObserver<'_> {
    fn emit(&mut self, event: &LoopEvent) -> Result<ExternalRef> {
        let result = self.inner.emit(event);
        self.failed |= result.is_err();
        if result.is_ok() {
            self.events.push(event.clone());
        }
        result
    }
}
#[allow(clippy::too_many_arguments)]
pub fn run_task<B: ModelBody>(
    task: &tasks::Task,
    trace: ExternalRef,
    mode: RunMode,
    limits: crate::relational::Limits,
    owner: Option<&dyn crate::relational::FormalOwner>,
    host: &mut ResearchHost<B>,
    observer: &mut dyn RuntimeObserver,
    cancellation: &CancellationToken,
) -> Result<Value> {
    use crate::relational::{Engine, Mode, TaskInspector};
    limits.validate()?;
    if mode != RunMode::Classic && owner.is_none() {
        return Err(Error::new(
            "relational execution requires an explicitly bound QL owner",
        ));
    }
    let candidate = task.candidate();
    let request = LoopRequest::from_legacy(
        json!({"id":task.id(),"taskId":task.id(),"runId":trace,"input":candidate["prompt"],
            "successConditions":candidate["successConditions"],"maxSteps":limits.max_steps}),
    )?;
    task.setup(&host.world)?;
    let before = host.world.snapshot()?;
    let secrets = host.secrets.clone();
    let started = std::time::Instant::now();
    let mut checked = CheckedObserver {
        events: vec![],
        inner: observer,
        failed: false,
    };
    let mut redacted = RedactingObserver {
        inner: &mut checked,
        secrets: &secrets,
    };
    let execution: Result<Value> = (|| {
        if mode == RunMode::Classic {
            let run = block_on(ClassicRuntime.run(&request, host, &mut redacted, cancellation))?;
            return Ok(json!({"report":run.report,"evidence_refs":run.evidence_refs}));
        }
        let native_mode = if mode == RunMode::Direct {
            Mode::Direct
        } else {
            Mode::Deep
        };
        let mut policy = crate::policy::ModelPolicy::new(native_mode);
        let mut inspector = TaskInspector {
            task: task.clone(),
            world: host.world.clone(),
            node: host.node.clone(),
        };
        let frame = json!({"id":task.id(),"initiating_intent":candidate["prompt"],
            "success_conditions":candidate["successConditions"],"available_capabilities":CAPABILITIES,
            "working_context":{"task_revision":task.revision()}});
        let run = Engine::new(
            owner.expect("admitted owner"),
            host,
            &mut policy,
            &mut inspector,
            &mut redacted,
            native_mode,
            trace.clone(),
            cancellation.clone(),
            limits,
            CAPABILITIES.iter().map(|s| s.to_string()).collect(),
            vec![],
            secrets.clone(),
        )?
        .run(frame)?;
        Ok(json!({"evidence_refs":run.evidence_refs,"report":run}))
    })();
    let sink_failed = checked.failed;
    let execution = match execution {
        Ok(v) => v,
        Err(e) => json!({"report":{"status":"failed","error":e.to_string()},"evidence_refs":[]}),
    };
    // Even an exterior failure is followed by independent inspection. A World
    // that cannot be read is unknown/failed, never replaced with a stale snapshot.
    let (after, verification) = match host
        .world
        .verify_root()
        .and_then(|()| host.world.snapshot())
    {
        Ok(after) => {
            let v = task.verify(&host.world,&before,&after,host.node.as_deref())
                .unwrap_or_else(|e|json!({"status":"failed","error":e.to_string(),"objective_checks_pass":false}));
            (after, v)
        }
        Err(e) => (
            Value::Null,
            json!({"status":"failed","error":e.to_string(),"objective_checks_pass":false}),
        ),
    };
    let mut inspection = crate::inspection::project(&checked.events, trace.as_ref(), mode.name());
    inspection["verification"] = verification.clone();
    inspection["execution_status"] = execution["report"]["status"].clone();
    inspection["after_state_digest"] = if after.is_null() {
        Value::Null
    } else {
        json!(crate::evidence::stable_digest(&after))
    };
    // Host observations have their own order; do not fabricate cross-owner indices.
    inspection["host_observations"] = json!(host.observations);
    let body_evidence = host.body.finish(&sanitize(&inspection, &secrets));
    // A broken canonical sink cannot acknowledge the run as persisted. Body
    // cleanup and evidence collection nevertheless happen before returning error.
    if sink_failed {
        return Err(Error::new(
            "research event persistence failed; run is not acknowledged",
        ));
    }
    let status = if after.is_null() || body_evidence["status"] == "failed" {
        json!("failed")
    } else {
        execution["report"]["status"].clone()
    };
    let record = json!({"schema":"actuation.research-run/v1","runtime":mode.name(),"status":status,
        "task":candidate,"task_revision":task.revision(),"trace_ref":trace,"model_body":host.body.basis(),
        "workspace":{"before":before,"after":after},"execution":execution,"body_evidence":body_evidence,
        "observations":host.observations,"verification":verification,"model_calls":host.model_calls,
        "capability_calls":host.capability_calls,"elapsed_ms":started.elapsed().as_millis(),
        "budget":{"limits":limits,"max_calls":host.max_calls},
        "evidence_standing":"D-unless-separately-exercised-and-attested","provider_evidence":"not-assessed",
        "human_acceptance":false,"factory_ancestry":"not-supplied"});
    Ok(sanitize(&record, &secrets))
}
/// Backwards-compatible library entry, using the same lifecycle as Direct/Deep.
pub fn run_classic<B: ModelBody>(
    task: &tasks::Task,
    trace: ExternalRef,
    max_steps: std::num::NonZeroU64,
    host: &mut ResearchHost<B>,
    observer: &mut dyn RuntimeObserver,
    cancellation: &CancellationToken,
) -> Result<Value> {
    run_task(
        task,
        trace,
        RunMode::Classic,
        crate::relational::Limits {
            max_steps: max_steps.get(),
            ..Default::default()
        },
        None,
        host,
        observer,
        cancellation,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::ProcessSpec;
    use crate::relational::{FormalOwner, Limits};
    use serde_json::{json, Value};
    use std::collections::VecDeque;

    struct ScriptedBody {
        control: bool,
        turns: VecDeque<Value>,
        fallback: Value,
    }
    impl ScriptedBody {
        fn classic(turns: Vec<Value>) -> Self {
            Self {
                control: false,
                turns: turns.into(),
                fallback: json!({"content":"done","capabilityCalls":[]}),
            }
        }
    }
    impl ModelBody for ScriptedBody {
        fn complete(&mut self, request: &Value) -> Result<Value> {
            candidate_boundary(request)?;
            if self.control {
                assert!(request["payload"].get("series1Control").is_some());
            } else {
                assert!(request["payload"].get("series1Control").is_none());
            }
            Ok(self.turns.pop_front().unwrap_or(self.fallback.clone()))
        }
        fn basis(&self) -> Value {
            json!({"fixture_provider":true,"provider_mode":"fixture"})
        }
    }

    /// Answers each purpose of the ModelPolicy control loop in order.
    struct ControlLoopBody {
        wrote: bool,
        interpreted: u32,
    }
    impl ModelBody for ControlLoopBody {
        fn complete(&mut self, request: &Value) -> Result<Value> {
            let purpose = request["payload"]["series1Control"]["purpose"]
                .as_str()
                .unwrap_or("")
                .to_owned();
            let control = match purpose.as_str() {
                "ql-next-act" => {
                    if self.wrote {
                        json!({"carrier":{"kind":"internal_control","input":{"standing":"scripted-hold"}},"intent":"hold","input_residue_refs":[]})
                    } else {
                        self.wrote = true;
                        json!({"carrier":{"kind":"capability","name":"write_file","args":{"path":"deliverable.md","content":"the deliverable"}},"intent":"deliver","input_residue_refs":[]})
                    }
                }
                "ql-interpret-return" => {
                    self.interpreted += 1;
                    let destination = if self.interpreted >= 2 { "P5" } else { "P4" };
                    json!({"destination":destination,"rationale":"scripted transition","semantic_summary":"difference retained","claimed_position":null,"ambiguity":null})
                }
                "ql-propose-determination" => {
                    json!({"synthesis":{"result":"delivered"},"requested_outcome":"close","evidence_refs":[],"unresolved_refs":[]})
                }
                "ql-evaluate-closure" => {
                    json!({"status":"close","task_success":"true","rationale":"objective checks passed"})
                }
                other => return Err(Error::new(format!("unexpected control purpose {other:?}"))),
            };
            Ok(json!({"control":control}))
        }
        fn basis(&self) -> Value {
            json!({"fixture_provider":true,"provider_mode":"fixture"})
        }
    }

    /// A scripted formal owner: the owner's vocabulary admits positions 0..=5,
    /// and every relation classification yields an empty owner reading.
    struct ScriptedOwner;
    impl FormalOwner for ScriptedOwner {
        fn invoke(&self, request: Value) -> Result<Value> {
            match request["operation"].as_str() {
                Some("vocabulary") => Ok(json!({"result":{
                    "positions":[{"position":0},{"position":1},{"position":2},
                                 {"position":3},{"position":4},{"position":5}],
                    "faces":["direct","conjugate"]}})),
                Some("classify-relation") => {
                    assert!(request["from"].is_u64() && request["to"].is_u64());
                    Ok(json!({"result":[]}))
                }
                other => Err(Error::new(format!("scripted owner lacks {other:?}"))),
            }
        }
        fn basis(&self) -> Value {
            json!({"repository":"scripted","revision":"0".repeat(40)})
        }
    }

    fn temp_world() -> (tempfile::TempDir, World) {
        let dir = tempfile::tempdir().unwrap();
        let world = World::open(dir.path()).unwrap();
        (dir, world)
    }

    #[test]
    fn run_mode_admits_only_the_three_native_conditions() {
        for (name, mode) in [
            ("classic", RunMode::Classic),
            ("ql-direct", RunMode::Direct),
            ("ql-deep", RunMode::Deep),
        ] {
            assert_eq!(RunMode::parse(name).unwrap(), mode);
            assert_eq!(mode.name(), name);
        }
        assert!(RunMode::parse("ql-shallow").is_err());
        assert!(RunMode::parse("Classic").is_err());
    }

    #[test]
    fn research_host_requires_a_positive_call_budget() {
        let (_dir, world) = temp_world();
        let body = ScriptedBody::classic(vec![]);
        assert!(ResearchHost::new(body, world, None, 0, vec![]).is_err());
    }

    #[test]
    fn block_on_returns_a_ready_value() {
        assert_eq!(block_on(async { 21 * 2 }), 42);
    }

    #[test]
    fn classic_mode_completes_a_skill_task_through_capabilities() {
        let task = crate::tasks::Task::get("S1-SKILL-001").unwrap();
        let (_dir, world) = temp_world();
        let body = ScriptedBody::classic(vec![json!({
            "content":"writing the deliverable",
            "capabilityCalls":[{"name":"write_file","args":{"path":"deliverable.md","content":"the deliverable"}}]
        })]);
        let mut host = ResearchHost::new(body, world, None, 64, vec![]).unwrap();
        let mut observer = RecordingObserver::default();
        let result = run_task(
            &task,
            ExternalRef::new("trace:test:classic-1").unwrap(),
            RunMode::Classic,
            Limits::default(),
            None,
            &mut host,
            &mut observer,
            &CancellationToken::default(),
        )
        .expect("classic run completes");
        assert_eq!(result["schema"], json!("actuation.research-run/v1"));
        assert_eq!(result["runtime"], json!("classic"));
        assert_eq!(result["verification"]["objective_checks_pass"], json!(true));
        assert_eq!(result["model_calls"], json!(2));
        assert_eq!(result["capability_calls"], json!(1));
        assert_eq!(
            result["workspace"]["after"]["deliverable.md"],
            json!("the deliverable")
        );
        assert!(!observer.events.is_empty());
        assert!(!host.observations.is_empty());
        assert_eq!(result["provider_evidence"], json!("not-assessed"));
        assert_eq!(result["human_acceptance"], json!(false));
        // The run never asks for more than the scripted call budget.
        assert!(host.model_calls + host.capability_calls <= 64);
    }

    #[test]
    fn direct_mode_requires_the_bound_owner_and_drives_the_control_loop() {
        let task = crate::tasks::Task::get("S1-SKILL-001").unwrap();
        let (_dir, world) = temp_world();
        let body = ScriptedBody::classic(vec![]);
        let mut host = ResearchHost::new(body, world, None, 64, vec![]).unwrap();
        let mut observer = RecordingObserver::default();
        // Direct/Deep without a bound owner is refused outright.
        let err = run_task(
            &task,
            ExternalRef::new("trace:test:direct-refused").unwrap(),
            RunMode::Direct,
            Limits::default(),
            None,
            &mut host,
            &mut observer,
            &CancellationToken::default(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("explicitly bound QL owner"));

        let (_dir2, world2) = temp_world();
        // The control body answers each purpose of the Direct control loop:
        // write once, hold, then evaluate and close at the determination.
        let body = ControlLoopBody {
            wrote: false,
            interpreted: 0,
        };
        let mut host2 = ResearchHost::new(body, world2, None, 64, vec![]).unwrap();
        let mut observer2 = RecordingObserver::default();
        let owner = ScriptedOwner;
        let result = run_task(
            &task,
            ExternalRef::new("trace:test:direct-1").unwrap(),
            RunMode::Direct,
            Limits::default(),
            Some(&owner),
            &mut host2,
            &mut observer2,
            &CancellationToken::default(),
        )
        .expect("direct run completes");
        assert_eq!(result["runtime"], json!("ql-direct"));
        assert_eq!(result["verification"]["objective_checks_pass"], json!(true));
        assert_eq!(
            result["workspace"]["after"]["deliverable.md"],
            json!("the deliverable")
        );
        assert!(result["owner_basis"].is_null() || result["body_evidence"].is_object());
        assert!(!observer2.events.is_empty());
    }

    #[test]
    fn sanitized_secrets_never_reach_observations() {
        let task = crate::tasks::Task::get("S1-SKILL-001").unwrap();
        let (_dir, world) = temp_world();
        let body = ScriptedBody::classic(vec![json!({
            "content":"token is sk-999",
            "capabilityCalls":[{"name":"write_file","args":{"path":"deliverable.md","content":"token sk-999 inside"}}]
        })]);
        let mut host = ResearchHost::new(body, world, None, 64, vec!["sk-999".to_owned()]).unwrap();
        let mut observer = RecordingObserver::default();
        let result = run_task(
            &task,
            ExternalRef::new("trace:test:secret-1").unwrap(),
            RunMode::Classic,
            Limits::default(),
            None,
            &mut host,
            &mut observer,
            &CancellationToken::default(),
        )
        .expect("run completes");
        let serialized = serde_json::to_string(&[&result, &json!(host.observations)]).unwrap();
        assert!(!serialized.contains("sk-999"));
        assert!(serialized.contains("[REDACTED]"));
    }

    #[test]
    fn a_relative_specimen_spec_is_refused_before_any_process() {
        let spec: ProcessSpec = serde_json::from_value(json!({
            "program":"./relative-agent","args":[],"cwd":"/tmp",
            "environment":{},"timeout_ms":1000,"output_limit":1000
        }))
        .unwrap();
        let err = spec.validate().unwrap_err();
        assert!(err.to_string().contains("absolute"));
    }
}
