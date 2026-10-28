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

/// What a comparison can vary from one take to the next (O-I manual §19.1,
/// The Launch panel, Repeat's Variable). `condition` is Actuation's own: how
/// the run's loop recurs. Every other variable is a setting of the supplied
/// body, carried to it as its configuration's `level` and confirmed by it.
pub const VARIABLES: &[&str] = &[
    "condition",
    "model",
    "effort",
    "skillset",
    "budget",
    "autonomy",
];

/// The condition that ends each run (O-I manual §19.1, Repeat's Stop when).
/// It is read at each turn's boundary, after the run's first turn: before the
/// body is asked again, and when the body finishes a turn with nothing more
/// to do (then, until it holds, the body is told to continue). A turn in
/// flight finishes within the body's own bound; no turn starts once the
/// condition holds. Absent, a run ends as it always has: at its runtime's
/// own end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopWhen {
    /// Task done: the run ends when the body finishes the task.
    Done,
    /// Turn cap: the run goes on until the body has taken this many turns.
    Turns(u64),
    /// Time cap: the run goes on until this many milliseconds have passed.
    Time(u64),
    /// Test pass: the run goes on until the task's objective check passes.
    Pass,
}
impl StopWhen {
    pub const WORDS: &'static [&'static str] = &["done", "turns", "time", "pass"];
    /// Read `{"when": done|turns|time|pass, "turns"?, "time_ms"?}`; null is
    /// no stop condition. A Turn cap without its number is the run's
    /// `max_steps`; a Time cap without its number is the body's session time
    /// (`time_default`), else it is refused.
    pub fn resolve(
        value: &Value,
        max_steps: Option<u64>,
        time_default: Option<u64>,
    ) -> Result<Option<Self>> {
        if value.is_null() {
            return Ok(None);
        }
        let fields = value
            .as_object()
            .ok_or_else(|| Error::new("stop requires an object {when, turns?, time_ms?}"))?;
        if let Some(other) = fields
            .keys()
            .find(|k| !["when", "turns", "time_ms"].contains(&k.as_str()))
        {
            return Err(Error::new(format!(
                "stop has no field {other}; it takes when, turns and time_ms"
            )));
        }
        let when = value["when"].as_str().unwrap_or_default();
        let number =
            |key: &str| -> Result<Option<u64>> {
                match value.get(key) {
                    None | Some(Value::Null) => Ok(None),
                    Some(v) => v.as_u64().filter(|n| *n > 0).map(Some).ok_or_else(|| {
                        Error::new(format!("stop {key} requires a positive integer"))
                    }),
                }
            };
        let (turns, time) = (number("turns")?, number("time_ms")?);
        let stray = |key: &str| Error::new(format!("stop {key} belongs to another stop condition"));
        let stop = match when {
            "done" | "pass" => {
                if turns.is_some() {
                    return Err(stray("turns"));
                }
                if time.is_some() {
                    return Err(stray("time_ms"));
                }
                if when == "done" {
                    Self::Done
                } else {
                    Self::Pass
                }
            }
            "turns" => {
                if time.is_some() {
                    return Err(stray("time_ms"));
                }
                let cap = turns.or(max_steps).ok_or_else(|| {
                    Error::new("a Turn cap needs its number of turns or the run's max_steps")
                })?;
                if max_steps.is_some_and(|steps| cap > steps) {
                    return Err(Error::new("a Turn cap cannot exceed the run's max_steps"));
                }
                Self::Turns(cap)
            }
            "time" => {
                if turns.is_some() {
                    return Err(stray("turns"));
                }
                let cap = time.or(time_default).ok_or_else(|| {
                    Error::new("a Time cap needs its time_ms or the body's session time")
                })?;
                if cap > 3_600_000 {
                    return Err(Error::new("a Time cap is at most 3600000 ms"));
                }
                Self::Time(cap)
            }
            _ => return Err(Error::new("stop when is one of done, turns, time and pass")),
        };
        Ok(Some(stop))
    }
    pub fn value(&self) -> Value {
        match self {
            Self::Done => json!({"when":"done"}),
            Self::Turns(n) => json!({"when":"turns","turns":n}),
            Self::Time(ms) => json!({"when":"time","time_ms":ms}),
            Self::Pass => json!({"when":"pass"}),
        }
    }
}

/// A run's stop condition, armed once its task is set up in its World.
struct Stopping {
    when: StopWhen,
    task: tasks::Task,
    before: Value,
    started: std::time::Instant,
    /// The host's call count at the last Test pass reading, and that reading:
    /// nothing has acted since, so it is not run again.
    checked: Option<(u64, Value)>,
    /// Why the run was ended by its stop condition, once it was.
    met: Option<Value>,
}
const CONTINUE: &str = "Continue the task.";
/// A malformed turn is a host-protocol event, not a model verdict: re-ask
/// with the failure made explicit rather than failing the run, and re-ask an
/// empty model-carried act once before accepting the empty return as the
/// interpreted difference. Applied uniformly to every condition; repair
/// counts and failed attempts stay on the record.
pub struct RepairingBody<B: ModelBody> {
    pub inner: B,
    pub max_repairs: usize,
}
impl<B: ModelBody> RepairingBody<B> {
    pub fn new(inner: B) -> Self {
        Self {
            inner,
            max_repairs: 2,
        }
    }
    fn is_parse_error(error: &Error) -> bool {
        let m = error.to_string();
        m.contains("invalid JSON")
            || m.contains("did not return a JSON object")
            || m.contains("must be an object")
    }
    fn repair_request(request: &Value, note: &str) -> Value {
        let mut repaired = request.clone();
        // The host wraps the call as {request, payload, capabilities}: the
        // control surface lives under payload.series1Control.
        if let Some(control) = repaired
            .get_mut("payload")
            .and_then(|payload| payload.get_mut("series1Control"))
        {
            let system = control["system"].as_str().unwrap_or_default().to_owned();
            control["system"] = json!(format!("{system}\n\n{note}"));
        }
        repaired
    }
}
impl<B: ModelBody> ModelBody for RepairingBody<B> {
    fn complete(&mut self, request: &Value) -> Result<Value> {
        let mut failed_attempts: Vec<Value> = Vec::new();
        let mut response = self.inner.complete(request);
        for attempt in 0..=self.max_repairs {
            match &response {
                Err(e) if Self::is_parse_error(e) && attempt < self.max_repairs => {
                    failed_attempts.push(json!({"error":e.to_string()}));
                    response = self.inner.complete(&Self::repair_request(
                        request,
                        "Your previous response was not a single valid JSON object. Return exactly one JSON object and no prose outside it. If you need to reason, do it silently and return only the JSON object.",
                    ));
                }
                Ok(value)
                    if request
                        .get("payload")
                        .and_then(|p| p.get("qlAct"))
                        .is_some()
                        && value.get("control").map(Value::is_null).unwrap_or(true)
                        && value
                            .get("capabilityCalls")
                            .and_then(Value::as_array)
                            .map(Vec::is_empty)
                            .unwrap_or(true)
                        && value
                            .get("content")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .map(str::is_empty)
                            .unwrap_or(true)
                        && attempt < 1 =>
                {
                    // Empty model-carried act: no difference to interpret.
                    response = self.inner.complete(&Self::repair_request(
                        request,
                        "Your previous response was an empty object. Perform the stated intent now and return exactly one JSON object with non-empty \"content\".",
                    ));
                    if let Ok(v) = &mut response {
                        v["empty_content_retry"] = json!(true);
                    }
                }
                _ => break,
            }
        }
        if let Ok(value) = &mut response {
            value["repairs"] = json!(failed_attempts.len());
            if !failed_attempts.is_empty() {
                value["failed_attempts"] = json!(failed_attempts);
            }
        }
        response
    }
    fn basis(&self) -> Value {
        let mut basis = self.inner.basis();
        basis["turn_repairs"] =
            json!({"malformed_reask_max":self.max_repairs,"empty_act_reask":true});
        basis
    }
    fn finish(&mut self, inspection: &Value) -> Value {
        self.inner.finish(inspection)
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
            // Host-side failures (disk pressure, a missing runtime) otherwise
            // surface with zero diagnostics. Carry the head of the specimen's
            // stderr into the error the run records.
            let head = r.stderr.chars().take(2048).collect::<String>();
            let detail = head.trim();
            return Err(Error::new(if detail.is_empty() {
                "supplied model specimen failed (no stderr captured)".to_owned()
            } else {
                format!("supplied model specimen failed: {detail}")
            }));
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
    /// Capability supply override (e.g. the toolset paradigm's six described
    /// tools). Absent: the default four-name list.
    capability_supply: Option<Value>,
    max_calls: u64,
    secrets: Vec<String>,
    stop: Option<StopWhen>,
    stopping: Option<Stopping>,
}
impl<B: ModelBody> ResearchHost<B> {
    /// Replace the capability supply the model sees on every model call.
    pub fn set_capability_supply(&mut self, supply: Value) {
        self.capability_supply = Some(supply);
    }
    /// The condition that ends the run (Repeat's Stop when). It is armed when
    /// the run sets its task up, and read at each turn's boundary.
    pub fn set_stop(&mut self, stop: StopWhen) {
        self.stop = Some(stop);
    }
    fn arm_stop(&mut self, task: &tasks::Task, before: &Value) -> Result<()> {
        let Some(when) = self.stop else {
            return Ok(());
        };
        if when == StopWhen::Pass && !task.has_objective_check() {
            return Err(Error::new(
                "Test pass needs the task's objective check, and this task has none",
            ));
        }
        let started = std::time::Instant::now();
        self.stopping = Some(Stopping {
            when,
            task: task.clone(),
            before: before.clone(),
            started,
            checked: None,
            met: None,
        });
        Ok(())
    }
    /// The task's objective check as the World stands, read again only when
    /// something has acted since the last reading.
    fn test_reading(&mut self) -> Result<Value> {
        let calls = self.model_calls + self.capability_calls;
        let stopping = self.stopping.as_mut().expect("armed");
        if let Some((at, reading)) = &stopping.checked {
            if *at == calls {
                return Ok(reading.clone());
            }
        }
        self.world.verify_root()?;
        let after = self.world.snapshot()?;
        let reading = stopping
            .task
            .verify(&self.world, &stopping.before, &after, self.node.as_deref())
            .unwrap_or_else(
                |e| json!({"status":"failed","error":e.to_string(),"objective_checks_pass":false}),
            );
        stopping.checked = Some((calls, reading.clone()));
        Ok(reading)
    }
    /// Whether the run's stop condition holds now; `finished` is the body's
    /// own finish (a turn with nothing more to do). Records why, once.
    fn stop_reached(&mut self, finished: bool) -> Result<Option<String>> {
        let Some((when, started, met)) = self
            .stopping
            .as_ref()
            .map(|s| (s.when, s.started, s.met.clone()))
        else {
            return Ok(None);
        };
        if let Some(met) = met {
            return Ok(met["reason"].as_str().map(str::to_owned));
        }
        // A run always takes its first turn.
        if self.model_calls == 0 {
            return Ok(None);
        }
        let reason = match when {
            StopWhen::Done => finished.then(|| "the body finished the task".to_owned()),
            StopWhen::Turns(cap) => {
                (self.model_calls >= cap).then(|| format!("its turn cap of {cap} turns"))
            }
            StopWhen::Time(ms) => (started.elapsed().as_millis() >= u128::from(ms))
                .then(|| format!("its time cap of {ms} ms")),
            StopWhen::Pass => (self.test_reading()?["objective_checks_pass"] == true)
                .then(|| "its test passes".to_owned()),
        };
        if let Some(reason) = &reason {
            let stopping = self.stopping.as_mut().expect("armed");
            stopping.met = Some(json!({"reason":reason,"at_turn":self.model_calls,
                "elapsed_ms":stopping.started.elapsed().as_millis()}));
        }
        Ok(reason)
    }
    /// What the body is told when it finishes a turn before its stop
    /// condition holds.
    fn continuation(&mut self) -> Result<String> {
        let pass = self
            .stopping
            .as_ref()
            .is_some_and(|s| s.when == StopWhen::Pass);
        if !pass {
            return Ok(CONTINUE.to_owned());
        }
        let reading = self.test_reading()?;
        let test = &reading["observations"]["test"];
        let output = [test["stdout"].as_str(), test["stderr"].as_str()]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("\n");
        let head = output.trim().chars().take(2000).collect::<String>();
        let mut text =
            "The task's own check does not pass yet. Continue the task until it does.".to_owned();
        if !head.is_empty() {
            text.push_str("\n\nThe check said:\n");
            text.push_str(&head);
        }
        Ok(sanitize(&json!(text), &self.secrets)
            .as_str()
            .unwrap_or(CONTINUE)
            .to_owned())
    }
    /// The run's stop condition as it ended: whether it was met, and why. Read
    /// from the facts after the run where the run ended on its own.
    fn stop_outcome(&self, execution_status: &Value, verification: &Value) -> Option<Value> {
        let stopping = self.stopping.as_ref()?;
        let mut outcome = stopping.when.value();
        let met = if let Some(met) = &stopping.met {
            outcome["reason"] = met["reason"].clone();
            outcome["at_turn"] = met["at_turn"].clone();
            outcome["elapsed_ms"] = met["elapsed_ms"].clone();
            true
        } else {
            // A run that failed or was cut off did not reach its condition,
            // whatever its turns or time came to.
            let ended = execution_status == "completed" || execution_status == "exhausted";
            let elapsed = stopping.started.elapsed().as_millis();
            let (met, reason) = match stopping.when {
                StopWhen::Done => (
                    execution_status == "completed",
                    "the body finished the task".to_owned(),
                ),
                StopWhen::Turns(cap) => (
                    self.model_calls >= cap,
                    format!("its turn cap of {cap} turns"),
                ),
                StopWhen::Time(ms) => (
                    elapsed >= u128::from(ms),
                    format!("its time cap of {ms} ms"),
                ),
                StopWhen::Pass => (
                    verification["objective_checks_pass"] == true,
                    "its test passes".to_owned(),
                ),
            };
            let met = met && ended;
            if met {
                outcome["reason"] = json!(reason);
                outcome["at_turn"] = json!(self.model_calls);
                outcome["elapsed_ms"] = json!(elapsed);
            }
            met
        };
        outcome["met"] = json!(met);
        Some(outcome)
    }
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
            capability_supply: None,
            max_calls,
            secrets,
            stop: None,
            stopping: None,
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
        let capabilities = self
            .capability_supply
            .clone()
            .unwrap_or_else(|| json!(CAPABILITIES));
        let payload = json!({"request":call.request.wire(),"payload":call.payload,"capabilities":capabilities});
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
        Box::pin(async move {
            // The stop condition is read before the body is asked again: a
            // run whose condition holds takes no further turn.
            if let Some(reason) = self.stop_reached(false)? {
                call.cancellation.request();
                return Err(HostError::Aborted(format!("stop when met: {reason}")));
            }
            self.model(call).map_err(Into::into)
        })
    }
    fn execute_capability<'a>(&'a mut self, call: HostCall) -> HostFuture<'a> {
        Box::pin(async move { self.capability(call).map_err(Into::into) })
    }
    fn receive_external_input<'a>(&'a mut self, call: HostCall) -> HostFuture<'a> {
        Box::pin(async move {
            // The body finished a turn with nothing more to do. Without a stop
            // condition, or once it holds, the run ends there; otherwise the
            // body is told to continue.
            if self.stopping.is_none() || call.payload.get("kind") != Some(&json!("follow_up")) {
                return Ok(Value::Null);
            }
            if self.stop_reached(true)?.is_some() {
                return Ok(Value::Null);
            }
            Ok(json!(self.continuation()?))
        })
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
    Toolset,
    Tagged,
    Eight,
    Twelve,
}
impl RunMode {
    pub fn parse(name: &str) -> Result<Self> {
        match name {
            "classic" => Ok(Self::Classic),
            "ql-direct" => Ok(Self::Direct),
            "ql-deep" => Ok(Self::Deep),
            "ql-toolset" => Ok(Self::Toolset),
            "ql-tagged" => Ok(Self::Tagged),
            "ql-eight" => Ok(Self::Eight),
            "ql-twelve" => Ok(Self::Twelve),
            _ => Err(Error::new("unknown research condition")),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Direct => "ql-direct",
            Self::Deep => "ql-deep",
            Self::Toolset => "ql-toolset",
            Self::Tagged => "ql-tagged",
            Self::Eight => "ql-eight",
            Self::Twelve => "ql-twelve",
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
/// Which authority answers the loop's control turns. Selection is explicit:
/// the environment at the run seam (QL_COMPRESSED_CONTROL, QL_VAK_CONTROL plus
/// QL_VAK_BIN), never a silent default inside the loop. Model control remains
/// the unchanged baseline.
pub enum ControlArm {
    /// Free English model control — the original lane.
    Model,
    /// Hand-ruled compression of interpret-return.
    Compressed,
    /// Kernel-native vak composition control over the QL-MEF `ql vak compose`
    /// instrument named by QL_VAK_BIN.
    Vak(crate::vak_control::VakControl),
}
impl ControlArm {
    pub fn from_env() -> Result<Self> {
        let compressed = std::env::var("QL_COMPRESSED_CONTROL").as_deref() == Ok("1");
        let vak = std::env::var("QL_VAK_CONTROL").as_deref() == Ok("1");
        if compressed && vak {
            return Err(Error::new(
                "QL_COMPRESSED_CONTROL and QL_VAK_CONTROL select different control arms; enable exactly one",
            ));
        }
        if !vak {
            return Ok(if compressed {
                Self::Compressed
            } else {
                Self::Model
            });
        }
        let program = std::env::var("QL_VAK_BIN")
            .ok()
            .map(std::path::PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
            .ok_or_else(|| {
                Error::new("QL_VAK_CONTROL=1 requires an explicit QL_VAK_BIN path to the ql binary")
            })?;
        Ok(Self::Vak(crate::vak_control::VakControl::bind(program)?))
    }
    /// Run-record disclosure of the armed control lane. Absent for the
    /// unchanged model lane so ordinary runs keep their record shape.
    fn disclosure(&self) -> Option<Value> {
        match self {
            Self::Model => None,
            Self::Compressed => Some(json!({"compressed_control":true,"vak_control":null})),
            Self::Vak(v) => Some(json!({"compressed_control":false,"vak_control":v.basis()})),
        }
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
    run_task_with_control(
        task,
        trace,
        mode,
        limits,
        owner,
        host,
        observer,
        cancellation,
        ControlArm::from_env()?,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn run_task_with_control<B: ModelBody>(
    task: &tasks::Task,
    trace: ExternalRef,
    mode: RunMode,
    limits: crate::relational::Limits,
    owner: Option<&dyn crate::relational::FormalOwner>,
    host: &mut ResearchHost<B>,
    observer: &mut dyn RuntimeObserver,
    cancellation: &CancellationToken,
    arm: ControlArm,
) -> Result<Value> {
    use crate::relational::{Engine, Mode, TaskInspector};
    limits.validate()?;
    if matches!(mode, RunMode::Direct | RunMode::Deep) && owner.is_none() {
        return Err(Error::new(
            "relational execution requires an explicitly bound QL owner",
        ));
    }
    let disclosure = arm.disclosure();
    let candidate = task.candidate();
    let request = LoopRequest::from_legacy(
        json!({"id":task.id(),"taskId":task.id(),"runId":trace,"input":candidate["prompt"],
            "successConditions":candidate["successConditions"],"maxSteps":limits.max_steps}),
    )?;
    task.setup(&host.world)?;
    let before = host.world.snapshot()?;
    host.arm_stop(task, &before)?;
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
        if matches!(
            mode,
            RunMode::Toolset | RunMode::Tagged | RunMode::Eight | RunMode::Twelve
        ) {
            // The toolset family: described tools replace the default
            // capability list, and the loop owns the circuit ledger. The
            // return loop carries all six tools; route one carries the
            // tagged four and ends the way classic ends; the lens condition
            // adds the cognitive lens-reading tool.
            let (supply, route, night) = match mode {
                // Route one: the tagged explicate four, ends like classic.
                RunMode::Tagged => (
                    crate::toolset::tagged_capability_supply(),
                    crate::toolset::Route::Tagged,
                    false,
                ),
                // The explicate eight: P-world four + Night middle, ends
                // like classic (no return verbs).
                RunMode::Eight => (
                    crate::toolset::eight_capability_supply(),
                    crate::toolset::Route::Tagged,
                    true,
                ),
                // The full P+P' twelve: both return conditions ride.
                RunMode::Twelve => (
                    crate::toolset::night_capability_supply(),
                    crate::toolset::Route::ReturnLoop,
                    true,
                ),
                _ => (
                    crate::toolset::toolset_capability_supply(),
                    crate::toolset::Route::ReturnLoop,
                    false,
                ),
            };
            host.set_capability_supply(supply);
            let outcome = block_on(crate::toolset::run_toolset(
                crate::toolset::ToolsetContext {
                    request: &request,
                    task: task.clone(),
                    world: host.world.clone(),
                    node: host.node.clone(),
                    before: before.clone(),
                    route,
                    night,
                    close_check: crate::toolset::CloseCheck::from_env(),
                },
                host,
                &mut redacted,
                cancellation,
            ));
            return Ok(json!({"report":outcome.report,"evidence_refs":outcome.evidence_refs}));
        }
        let native_mode = if mode == RunMode::Direct {
            Mode::Direct
        } else {
            Mode::Deep
        };
        let mut policy = crate::policy::ModelPolicy::with_schedule(
            native_mode,
            crate::policy::schedule_for_category(task.category()),
        );
        match arm {
            ControlArm::Model => {}
            ControlArm::Compressed => policy = policy.with_compressed_control(),
            ControlArm::Vak(vak) => policy = policy.with_vak_control(vak),
        }
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
    // A run its stop condition ended stands completed: it ran to the
    // condition its plan named. The runtime's own word stays in `execution`.
    let stop = host.stop_outcome(&execution["report"]["status"], &verification);
    let status = if after.is_null() || body_evidence["status"] == "failed" {
        json!("failed")
    } else if stop.as_ref().is_some_and(|s| s["met"] == true) {
        json!("completed")
    } else {
        execution["report"]["status"].clone()
    };
    let mut record = json!({"schema":"actuation.research-run/v1","runtime":mode.name(),"status":status,
        "task":candidate,"task_revision":task.revision(),"trace_ref":trace,"model_body":host.body.basis(),
        "workspace":{"before":before,"after":after},"execution":execution,"body_evidence":body_evidence,
        "observations":host.observations,"verification":verification,"model_calls":host.model_calls,
        "capability_calls":host.capability_calls,"elapsed_ms":started.elapsed().as_millis(),
        "budget":{"limits":limits,"max_calls":host.max_calls},
        "evidence_standing":"D-unless-separately-exercised-and-attested","provider_evidence":"not-assessed",
        "human_acceptance":false,"factory_ancestry":"not-supplied"});
    if let (Some(m), Some(d)) = (record.as_object_mut(), disclosure) {
        m.insert("control".into(), d);
    }
    if let (Some(m), Some(stop)) = (record.as_object_mut(), stop) {
        m.insert("stop".into(), stop);
    }
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

    #[test]
    fn stop_when_reads_its_condition_and_fills_only_the_caps_it_may() {
        assert_eq!(
            StopWhen::resolve(&Value::Null, Some(4), None).unwrap(),
            None
        );
        let read = |v: Value| StopWhen::resolve(&v, Some(4), Some(9000));
        assert_eq!(read(json!({"when":"done"})).unwrap(), Some(StopWhen::Done));
        assert_eq!(read(json!({"when":"pass"})).unwrap(), Some(StopWhen::Pass));
        assert_eq!(
            read(json!({"when":"turns"})).unwrap(),
            Some(StopWhen::Turns(4))
        );
        assert_eq!(
            read(json!({"when":"turns","turns":2})).unwrap(),
            Some(StopWhen::Turns(2))
        );
        assert_eq!(
            read(json!({"when":"time"})).unwrap(),
            Some(StopWhen::Time(9000))
        );
        assert_eq!(
            read(json!({"when":"time","time_ms":5})).unwrap(),
            Some(StopWhen::Time(5))
        );
        for bad in [
            json!({"when":"turns","turns":5}),
            json!({"when":"turns","turns":0}),
            json!({"when":"done","turns":2}),
            json!({"when":"pass","time_ms":2}),
            json!({"when":"time","time_ms":3_600_001}),
            json!({"when":"later"}),
            json!({"when":"done","until":"later"}),
            json!("done"),
        ] {
            assert!(read(bad.clone()).is_err(), "{bad}");
        }
        assert!(StopWhen::resolve(&json!({"when":"time"}), Some(4), None).is_err());
        for stop in [
            StopWhen::Done,
            StopWhen::Turns(3),
            StopWhen::Time(7),
            StopWhen::Pass,
        ] {
            assert_eq!(
                read(stop.value()).unwrap(),
                Some(stop),
                "{stop:?} round-trips"
            );
        }
    }

    fn temp_world() -> (tempfile::TempDir, World) {
        let dir = tempfile::tempdir().unwrap();
        let world = World::open(dir.path()).unwrap();
        (dir, world)
    }

    #[test]
    fn toolset_mode_carries_the_ql_form_in_the_toolset_and_closes_through_the_return_condition() {
        let task = crate::tasks::Task::get("S1-SKILL-001").unwrap();
        let (_dir, world) = temp_world();
        let body = ScriptedBody::classic(vec![
            json!({"content":"seeing the field",
                   "capabilityCalls":[{"name":"read_file","args":{"path":"SKILL.md"}}]}),
            json!({"content":"the skill is absorbed; the effect comes next",
                   "capabilityCalls":[{"name":"situate","args":{"position":"P2"}}]}),
            json!({"content":"making the effect",
                   "capabilityCalls":[{"name":"write_file","args":{"path":"deliverable.md","content":"the deliverable"}}]}),
            json!({"content":"the bounded request is realised",
                   "capabilityCalls":[{"name":"close","args":{"synthesis":"deliverable.md written from SKILL.md"}}]}),
        ]);
        let mut host = ResearchHost::new(body, world, None, 64, vec![]).unwrap();
        let mut observer = RecordingObserver::default();
        let result = run_task(
            &task,
            ExternalRef::new("trace:test:toolset-1").unwrap(),
            RunMode::Toolset,
            Limits::default(),
            None,
            &mut host,
            &mut observer,
            &CancellationToken::default(),
        )
        .expect("toolset run completes");
        assert_eq!(result["runtime"], json!("ql-toolset"));
        assert_eq!(result["status"], json!("completed"));
        assert_eq!(result["verification"]["objective_checks_pass"], json!(true));
        // The model saw the six described tools, not the four bare names.
        let seen = result["observations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["event_type"] == json!("model_requested"))
            .and_then(|o| o["value"]["capabilities"].as_array())
            .unwrap();
        assert_eq!(seen.len(), 6);
        assert!(
            seen[4]["description"].as_str().unwrap().contains("P"),
            "situate carries the office law"
        );
        // situate and close are loop verbs: they never reach the world.
        assert_eq!(result["model_calls"], json!(4));
        assert_eq!(result["capability_calls"], json!(2));
        let report = &result["execution"]["report"];
        let circuit = &report["circuits"][0];
        assert_eq!(circuit["closure_state"], json!("closed"));
        let positions: Vec<u64> = circuit["residues"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["position"].as_u64().unwrap())
            .collect();
        assert_eq!(
            positions,
            vec![0, 1, 2, 2, 5],
            "frame, material, effect, return-reading, determination"
        );
        assert_eq!(report["closure"]["success_state"], json!("true"));
    }

    #[test]
    fn toolset_mode_refuses_to_complete_without_the_return_condition() {
        let task = crate::tasks::Task::get("S1-SKILL-001").unwrap();
        let (_dir, world) = temp_world();
        // Bare content three times: the loop nudges twice, then fails honestly.
        let body =
            ScriptedBody::classic(vec![json!({"content":"the answer","capabilityCalls":[]})]);
        let mut host = ResearchHost::new(body, world, None, 64, vec![]).unwrap();
        let mut observer = RecordingObserver::default();
        let result = run_task(
            &task,
            ExternalRef::new("trace:test:toolset-2").unwrap(),
            RunMode::Toolset,
            Limits::default(),
            None,
            &mut host,
            &mut observer,
            &CancellationToken::default(),
        )
        .expect("toolset run resolves");
        assert_eq!(result["status"], json!("failed"));
        assert!(result["execution"]["report"]["error"]
            .as_str()
            .unwrap()
            .contains("return condition never ran"));
    }

    #[test]
    fn toolset_mode_close_without_work_fails_the_objective_gate() {
        let task = crate::tasks::Task::get("S1-SKILL-001").unwrap();
        let (_dir, world) = temp_world();
        let body = ScriptedBody::classic(vec![json!({"content":"closing immediately",
                   "capabilityCalls":[{"name":"close","args":{"synthesis":"nothing was needed"}}]})]);
        let mut host = ResearchHost::new(body, world, None, 64, vec![]).unwrap();
        let mut observer = RecordingObserver::default();
        let result = run_task(
            &task,
            ExternalRef::new("trace:test:toolset-3").unwrap(),
            RunMode::Toolset,
            Limits::default(),
            None,
            &mut host,
            &mut observer,
            &CancellationToken::default(),
        )
        .expect("toolset run resolves");
        assert_eq!(result["status"], json!("failed"));
        assert_eq!(
            result["execution"]["report"]["closure"]["success_state"],
            json!("false")
        );
        assert_eq!(
            result["verification"]["objective_checks_pass"],
            json!(false)
        );
    }

    #[test]
    fn tagged_route_carries_the_explicate_four_and_ends_like_classic() {
        let task = crate::tasks::Task::get("S1-SKILL-001").unwrap();
        let (_dir, world) = temp_world();
        let body = ScriptedBody::classic(vec![
            json!({"content":"seeing the field",
                   "capabilityCalls":[{"name":"read_file","args":{"path":"SKILL.md"}}]}),
            json!({"content":"the deliverable, written from the material in hand",
                   "capabilityCalls":[{"name":"write_file","args":{"path":"deliverable.md","content":"the deliverable"}}]}),
            json!({"content":"the bounded request is realised from the material in hand"}),
        ]);
        let mut host = ResearchHost::new(body, world, None, 64, vec![]).unwrap();
        let mut observer = RecordingObserver::default();
        let result = run_task(
            &task,
            ExternalRef::new("trace:test:tagged-1").unwrap(),
            RunMode::Tagged,
            Limits::default(),
            None,
            &mut host,
            &mut observer,
            &CancellationToken::default(),
        )
        .expect("tagged run completes");
        assert_eq!(result["runtime"], json!("ql-tagged"));
        assert_eq!(result["status"], json!("completed"));
        assert_eq!(result["verification"]["objective_checks_pass"], json!(true));
        // The model saw the tagged four, never the loop verbs.
        let seen = result["observations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["event_type"] == json!("model_requested"))
            .and_then(|o| o["value"]["capabilities"].as_array())
            .unwrap();
        assert_eq!(seen.len(), 4);
        assert!(
            seen.iter()
                .all(|c| c["description"].as_str().unwrap().contains("(P")),
            "every tagged tool carries its office law"
        );
        let report = &result["execution"]["report"];
        assert_eq!(report["paradigm"], json!("tagged"));
        let circuit = &report["circuits"][0];
        // No return condition ran: the circuit stays open, the ledger stays.
        assert_eq!(circuit["closure_state"], json!("open"));
        let positions: Vec<u64> = circuit["residues"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["position"].as_u64().unwrap())
            .collect();
        assert_eq!(positions, vec![0, 1, 2], "frame, material, effect");
    }

    #[test]
    fn tagged_route_refuses_the_loop_verbs_and_still_delivers() {
        let task = crate::tasks::Task::get("S1-SKILL-001").unwrap();
        let (_dir, world) = temp_world();
        let body = ScriptedBody::classic(vec![
            json!({"content":"trying the verb",
                   "capabilityCalls":[{"name":"close","args":{"synthesis":"not here"}}]}),
            json!({"content":"then without it",
                   "capabilityCalls":[{"name":"write_file","args":{"path":"deliverable.md","content":"the deliverable"}}]}),
            json!({"content":"delivered"}),
        ]);
        let mut host = ResearchHost::new(body, world, None, 64, vec![]).unwrap();
        let mut observer = RecordingObserver::default();
        let result = run_task(
            &task,
            ExternalRef::new("trace:test:tagged-2").unwrap(),
            RunMode::Tagged,
            Limits::default(),
            None,
            &mut host,
            &mut observer,
            &CancellationToken::default(),
        )
        .expect("tagged run resolves");
        assert_eq!(result["status"], json!("completed"));
        assert_eq!(result["model_calls"], json!(3));
        assert_eq!(result["capability_calls"], json!(1));
        assert_eq!(
            result["execution"]["report"]["circuits"][0]["closure_state"],
            json!("open")
        );
    }

    #[test]
    fn twelve_condition_serves_the_night_face_and_records_full_readings() {
        let script_dir = tempfile::tempdir().unwrap();
        let script = script_dir.path().join("fake-reflect.sh");
        std::fs::write(
            &script,
            "#!/bin/sh\nprintf '%s' '{\"mode\":\"resonant\",\"reading\":{\"shape\":\"ranked\",\"selected\":\"L2\",\"ranking\":[{\"lens\":\"L2\",\"name\":\"Logical\",\"mass\":0.5}]},\"own_disclosure\":\"my candidate\",\"latency_ms\":1}'\n",
        )
        .unwrap();
        std::fs::set_permissions(&script, std::os::unix::fs::PermissionsExt::from_mode(0o755))
            .unwrap();
        std::env::set_var("QL_REFLECT_BIN", &script);
        let task = crate::tasks::Task::get("S1-SKILL-001").unwrap();
        let (_dir, world) = temp_world();
        let body = ScriptedBody::classic(vec![
            json!({"content":"reading the subject's frame",
                   "capabilityCalls":[{"name":"reflect_resonant_frames","args":{"subject":"the handoff work as it stands","own_disclosure":"my candidate: it reads Logical"}}]}),
            json!({"content":"making the effect",
                   "capabilityCalls":[{"name":"write_file","args":{"path":"deliverable.md","content":"the deliverable"}}]}),
            json!({"content":"the bounded request is realised",
                   "capabilityCalls":[{"name":"close","args":{"synthesis":"deliverable.md written from SKILL.md"}}]}),
        ]);
        let mut host = ResearchHost::new(body, world, None, 64, vec![]).unwrap();
        let mut observer = RecordingObserver::default();
        let result = run_task(
            &task,
            ExternalRef::new("trace:test:twelve-1").unwrap(),
            RunMode::Twelve,
            Limits::default(),
            None,
            &mut host,
            &mut observer,
            &CancellationToken::default(),
        )
        .expect("twelve run completes");
        assert_eq!(result["runtime"], json!("ql-twelve"));
        assert_eq!(result["status"], json!("completed"));
        // The model saw the full P+P' twelve.
        let seen = result["observations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["event_type"] == json!("model_requested"))
            .and_then(|o| o["value"]["capabilities"].as_array())
            .unwrap();
        assert_eq!(seen.len(), 12);
        assert_eq!(seen[6]["name"], json!("reflect_full_text"));
        assert_eq!(seen[11]["name"], json!("reflect_resonant_frames"));
        let report = &result["execution"]["report"];
        assert_eq!(report["night_calls"], json!(1));
        let kinds: Vec<&str> = report["circuits"][0]["residues"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|r| r["kind"].as_str())
            .collect();
        assert!(
            kinds.contains(&"night-reading"),
            "the reading rides the circuit record"
        );
        assert_eq!(result["verification"]["objective_checks_pass"], json!(true));
    }

    #[test]
    fn eight_condition_is_explicate_with_the_night_middle_and_ends_like_classic() {
        let script_dir = tempfile::tempdir().unwrap();
        let script = script_dir.path().join("fake-reflect-eight.sh");
        std::fs::write(
            &script,
            "#!/bin/sh\nprintf '%s' '{\"mode\":\"being\",\"reading\":{\"shape\":\"square A\"},\"latency_ms\":1}'\n",
        )
        .unwrap();
        std::fs::set_permissions(&script, std::os::unix::fs::PermissionsExt::from_mode(0o755))
            .unwrap();
        std::env::set_var("QL_REFLECT_BIN", &script);
        let task = crate::tasks::Task::get("S1-SKILL-001").unwrap();
        let (_dir, world) = temp_world();
        let body = ScriptedBody::classic(vec![
            json!({"content":"reading through square A",
                   "capabilityCalls":[{"name":"reflect_being","args":{"subject":"the work as it stands"}}]}),
            json!({"content":"making the effect",
                   "capabilityCalls":[{"name":"write_file","args":{"path":"deliverable.md","content":"the deliverable"}}]}),
            json!({"content":"the bounded request is realised from the material in hand"}),
        ]);
        let mut host = ResearchHost::new(body, world, None, 64, vec![]).unwrap();
        let mut observer = RecordingObserver::default();
        let result = run_task(
            &task,
            ExternalRef::new("trace:test:eight-1").unwrap(),
            RunMode::Eight,
            Limits::default(),
            None,
            &mut host,
            &mut observer,
            &CancellationToken::default(),
        )
        .expect("eight run completes");
        assert_eq!(result["runtime"], json!("ql-eight"));
        assert_eq!(result["status"], json!("completed"));
        let seen = result["observations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|o| o["event_type"] == json!("model_requested"))
            .and_then(|o| o["value"]["capabilities"].as_array())
            .unwrap();
        assert_eq!(seen.len(), 8);
        let names: Vec<&str> = seen.iter().map(|c| c["name"].as_str().unwrap()).collect();
        assert!(!names.contains(&"situate") && !names.contains(&"close"));
        assert!(!names.contains(&"reflect_full_text"));
        assert!(names.contains(&"reflect_being") && names.contains(&"reflect_resonant_frames"));
        let report = &result["execution"]["report"];
        assert_eq!(report["night_calls"], json!(1));
        assert_eq!(report["circuits"][0]["closure_state"], json!("open"));
    }

    #[test]
    fn right_frame_is_gated_on_coverage_and_then_admitted() {
        let script_dir = tempfile::tempdir().unwrap();
        let script = script_dir.path().join("fake-reflect-gate.sh");
        std::fs::write(
            &script,
            "#!/bin/sh\nprintf '%s' '{\"mode\":\"x\",\"reading\":{\"shape\":\"fake\"},\"latency_ms\":1}'\n",
        )
        .unwrap();
        std::fs::set_permissions(&script, std::os::unix::fs::PermissionsExt::from_mode(0o755))
            .unwrap();
        std::env::set_var("QL_REFLECT_BIN", &script);
        let task = crate::tasks::Task::get("S1-SKILL-001").unwrap();
        let (_dir, world) = temp_world();
        let body = ScriptedBody::classic(vec![
            // Frame selection before any coverage: refused as a tool error.
            json!({"content":"selecting a frame first",
                   "capabilityCalls":[{"name":"reflect_right_frame","args":{"subject":"the work"}}]}),
            // Coverage: the full reading.
            json!({"content":"taking the full reading",
                   "capabilityCalls":[{"name":"reflect_full_text","args":{"subject":"the work as a whole"}}]}),
            // Now the selection is admitted.
            json!({"content":"selecting the frame",
                   "capabilityCalls":[{"name":"reflect_right_frame","args":{"subject":"the work"}}]}),
            json!({"content":"making the effect",
                   "capabilityCalls":[{"name":"write_file","args":{"path":"deliverable.md","content":"the deliverable"}}]}),
            json!({"content":"the bounded request is realised",
                   "capabilityCalls":[{"name":"close","args":{"synthesis":"deliverable.md written from SKILL.md"}}]}),
        ]);
        let mut host = ResearchHost::new(body, world, None, 64, vec![]).unwrap();
        let mut observer = RecordingObserver::default();
        let result = run_task(
            &task,
            ExternalRef::new("trace:test:gate-1").unwrap(),
            RunMode::Twelve,
            Limits::default(),
            None,
            &mut host,
            &mut observer,
            &CancellationToken::default(),
        )
        .expect("gated run completes");
        assert_eq!(result["status"], json!("completed"));
        let history = &result["execution"]["report"]["history"];
        let entries = history.as_array().unwrap();
        // The first right_frame call was refused; the retried one returned a reading.
        let first = entries
            .iter()
            .find(|h| h["role"] == json!("capability") && h["name"] == json!("reflect_right_frame"))
            .expect("the first right_frame call is on the record");
        assert!(
            first["result"]["error"].is_string(),
            "the pre-coverage selection is refused with a reason"
        );
        let admitted = entries
            .iter()
            .filter(|h| {
                h["role"] == json!("capability") && h["name"] == json!("reflect_right_frame")
            })
            .any(|h| h["result"]["error"].is_null());
        assert!(admitted, "the post-coverage selection is admitted");
        // Refusals are not readings: the counter counts what was taken.
        assert_eq!(result["execution"]["report"]["night_calls"], json!(2));
        assert_eq!(result["verification"]["objective_checks_pass"], json!(true));
    }

    #[test]
    fn run_mode_admits_the_native_conditions() {
        for (name, mode) in [
            ("classic", RunMode::Classic),
            ("ql-direct", RunMode::Direct),
            ("ql-deep", RunMode::Deep),
            ("ql-toolset", RunMode::Toolset),
            ("ql-tagged", RunMode::Tagged),
            ("ql-eight", RunMode::Eight),
            ("ql-twelve", RunMode::Twelve),
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

    #[test]
    fn a_failed_specimen_carries_the_head_of_its_stderr() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let mut body = ProcessModelBody {
            process: ProcessSpec {
                program: dir.path().join("failing-specimen.sh"),
                args: vec![],
                cwd: dir.path().to_owned(),
                environment: Default::default(),
                timeout_ms: 10_000,
                output_limit: 1 << 20,
            },
            source_basis: json!({}),
            fixture: false,
        };
        std::fs::write(
            &body.process.program,
            b"#!/bin/sh\ncat >/dev/null\necho 'disk pressure on host' >&2\nexit 1\n",
        )
        .unwrap();
        std::fs::set_permissions(
            &body.process.program,
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        let err = body.complete(&json!({"payload":{}})).unwrap_err();
        let message = err.to_string();
        assert!(
            message.contains("supplied model specimen failed"),
            "{message}"
        );
        assert!(message.contains("disk pressure on host"), "{message}");

        // A silent failure still says what happened instead of bare tone.
        std::fs::write(
            &body.process.program,
            b"#!/bin/sh\ncat >/dev/null\nexit 1\n",
        )
        .unwrap();
        let err = body.complete(&json!({"payload":{}})).unwrap_err();
        assert!(err.to_string().contains("no stderr captured"));
    }
}
