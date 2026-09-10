use crate::PortFuture;
use actuation_core::{Error, ExternalRef, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::{
    collections::BTreeMap,
    future::Future,
    num::NonZeroU64,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

/// Cancellation is a request, not a claim to have killed a provider process.
/// A host must observe it or explicitly report its inability to interrupt.
#[derive(Clone, Default, Debug)]
pub struct CancellationToken(Arc<AtomicBool>);
impl CancellationToken {
    pub fn request(&self) {
        self.0.store(true, Ordering::SeqCst);
    }
    pub fn requested(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostError {
    Aborted(String),
    Failed(String),
}
impl HostError {
    pub fn message(&self) -> &str {
        match self {
            Self::Aborted(s) | Self::Failed(s) => s,
        }
    }
}
impl From<Error> for HostError {
    fn from(value: Error) -> Self {
        Self::Failed(value.to_string())
    }
}
pub type HostFuture<'a> =
    Pin<Box<dyn Future<Output = std::result::Result<Value, HostError>> + Send + 'a>>;

/// A task payload remains opaque. This reference identifies execution trace,
/// not a Factory Run or an Agency. The compatibility view uses legacy runId.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoopRequest {
    wire: Value,
    task: ExternalRef,
    trace: ExternalRef,
    max_steps: NonZeroU64,
}
impl LoopRequest {
    pub fn new(task: ExternalRef, trace: ExternalRef, input: Value, max_steps: NonZeroU64) -> Self {
        Self {
            wire: json!({"id": task, "taskId": task, "runId": trace, "input": input, "maxSteps": max_steps.get(), "successConditions": []}),
            task,
            trace,
            max_steps,
        }
    }
    pub fn from_legacy(mut wire: Value) -> Result<Self> {
        let object = wire
            .as_object_mut()
            .ok_or_else(|| Error::new("loop request must be an object"))?;
        let id = object
            .get("id")
            .filter(|v| !v.is_null())
            .or_else(|| object.get("taskId").filter(|v| !v.is_null()))
            .cloned()
            .unwrap_or(json!("task"));
        let task = object
            .get("taskId")
            .filter(|v| !v.is_null())
            .cloned()
            .unwrap_or(id.clone());
        let task_ref = ExternalRef::new(
            task.as_str()
                .ok_or_else(|| Error::new("taskId must name a task"))?,
        )?;
        let trace = object
            .get("runId")
            .filter(|v| !v.is_null())
            .cloned()
            .unwrap_or_else(|| json!(format!("run:{task_ref}:classic")));
        let trace_ref = ExternalRef::new(
            trace
                .as_str()
                .ok_or_else(|| Error::new("runId must name a trace"))?,
        )?;
        let max = object
            .get("maxSteps")
            .and_then(Value::as_u64)
            .filter(|n| *n > 0 && *n <= 9007199254740991)
            .unwrap_or(64);
        object.insert("id".into(), id);
        object.insert("taskId".into(), task);
        let input = object.get("input").cloned().unwrap_or(Value::Null);
        object.insert("input".into(), input);
        let conditions = object
            .get("successConditions")
            .filter(|v| v.is_array())
            .cloned()
            .unwrap_or(json!([]));
        object.insert("successConditions".into(), conditions);
        object.insert("maxSteps".into(), json!(max));
        Ok(Self {
            wire,
            task: task_ref,
            trace: trace_ref,
            max_steps: NonZeroU64::new(max).expect("positive default"),
        })
    }
    pub fn wire(&self) -> &Value {
        &self.wire
    }
    pub fn task_ref(&self) -> &ExternalRef {
        &self.task
    }
    pub fn trace_ref(&self) -> &ExternalRef {
        &self.trace
    }
    pub fn max_steps(&self) -> NonZeroU64 {
        self.max_steps
    }
}

#[derive(Clone, Debug)]
pub struct HostCall {
    pub request: LoopRequest,
    pub payload: Map<String, Value>,
    pub cancellation: CancellationToken,
}
/// All carriers cross one host seam. The owner decides whether/how the actual
/// tool/model/human/context exists; the runtime cannot manufacture availability.
pub trait RuntimeHost: Send {
    fn call_model<'a>(&'a mut self, call: HostCall) -> HostFuture<'a>;
    fn execute_capability<'a>(&'a mut self, call: HostCall) -> HostFuture<'a>;
    fn receive_external_input<'a>(&'a mut self, call: HostCall) -> HostFuture<'a>;
    fn read_context<'a>(&'a mut self, call: HostCall) -> HostFuture<'a>;
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Carrier {
    Model,
    Capability { name: ExternalRef, args: Value },
    Human { input_kind: Option<String> },
    Environment { kind: ContextCarrier, input: Value },
    InternalControl { input: Value },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextCarrier {
    Environment,
    Artifact,
    ExternalEvaluator,
}
impl Carrier {
    pub fn from_legacy(value: &Value) -> Result<Self> {
        let kind = value
            .get("kind")
            .filter(|v| !v.is_null())
            .and_then(Value::as_str)
            .unwrap_or("model");
        let optional = |key| value.get(key).filter(|v| !v.is_null()).cloned();
        match kind {
            "model" => Ok(Self::Model),
            "tool" | "capability" => Ok(Self::Capability {
                name: ExternalRef::new(
                    value["name"]
                        .as_str()
                        .ok_or_else(|| Error::new("capability must name a supplied faculty"))?,
                )?,
                args: optional("args").unwrap_or(json!({})),
            }),
            "human" => Ok(Self::Human {
                input_kind: optional("inputKind")
                    .map(|v| {
                        v.as_str()
                            .map(String::from)
                            .ok_or_else(|| Error::new("human input kind must be a string"))
                    })
                    .transpose()?,
            }),
            "environment" | "artifact" | "external_evaluator" => Ok(Self::Environment {
                kind: match kind {
                    "environment" => ContextCarrier::Environment,
                    "artifact" => ContextCarrier::Artifact,
                    _ => ContextCarrier::ExternalEvaluator,
                },
                input: optional("input").unwrap_or(Value::Null),
            }),
            "internal_control" => Ok(Self::InternalControl {
                input: optional("input").unwrap_or(Value::Null),
            }),
            _ => Err(Error::new(format!(
                "Unsupported shared carrier kind '{kind}'."
            ))),
        }
    }
}
pub async fn dispatch_host_carrier(
    host: &mut dyn RuntimeHost,
    carrier: &Carrier,
    request: &LoopRequest,
    cancellation: &CancellationToken,
    mut payload: Map<String, Value>,
) -> std::result::Result<Value, HostError> {
    // Runtime-owned request/signal values cannot be shadowed by arbitrary payload.
    payload.remove("request");
    payload.remove("signal");
    match carrier {
        Carrier::Model => {
            host.call_model(HostCall {
                request: request.clone(),
                payload,
                cancellation: cancellation.clone(),
            })
            .await
        }
        Carrier::Capability { name, args } => {
            payload.insert("name".into(), json!(name));
            payload.insert("args".into(), args.clone());
            host.execute_capability(HostCall {
                request: request.clone(),
                payload,
                cancellation: cancellation.clone(),
            })
            .await
        }
        Carrier::Human { input_kind } => {
            let kind = input_kind
                .clone()
                .map(Value::String)
                .or_else(|| payload.get("kind").filter(|v| !v.is_null()).cloned())
                .unwrap_or(json!("external_input"));
            payload.insert("kind".into(), kind);
            host.receive_external_input(HostCall {
                request: request.clone(),
                payload,
                cancellation: cancellation.clone(),
            })
            .await
        }
        Carrier::Environment { kind, input } => {
            payload.insert("kind".into(), json!(kind));
            payload.insert("input".into(), input.clone());
            host.read_context(HostCall {
                request: request.clone(),
                payload,
                cancellation: cancellation.clone(),
            })
            .await
        }
        Carrier::InternalControl { input } => Ok(input.clone()),
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LoopEvent {
    pub channel: String,
    pub event_id: String,
    pub event_type: String,
    pub run_id: ExternalRef,
    pub sequence: u64,
    pub runtime: String,
    pub payload: Value,
}
/// An observer returns the reference of the evidence it really accepted. A
/// failed observer fails the operation; evidence is not fabricated by the loop.
pub trait RuntimeObserver: Send {
    fn emit(&mut self, event: &LoopEvent) -> Result<ExternalRef>;
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LoopStatus {
    Completed,
    Failed,
    Cancelled,
    Exhausted,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LoopReport {
    pub status: LoopStatus,
    pub runtime: String,
    #[serde(rename = "runtimeVersion")]
    pub runtime_version: String,
    #[serde(rename = "runId")]
    pub trace_ref: ExternalRef,
    pub iterations: u64,
    #[serde(rename = "modelCalls")]
    pub model_calls: u64,
    #[serde(rename = "capabilityCalls")]
    pub capability_calls: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub history: Vec<Value>,
}
#[derive(Clone, Debug)]
pub struct LoopExecution {
    pub report: LoopReport,
    pub evidence_refs: Vec<ExternalRef>,
}
pub trait LoopRuntime: Send {
    fn id(&self) -> &str;
    fn version(&self) -> &str;
    fn run<'a>(
        &'a mut self,
        request: &'a LoopRequest,
        host: &'a mut dyn RuntimeHost,
        observer: &'a mut dyn RuntimeObserver,
        cancellation: &'a CancellationToken,
    ) -> PortFuture<'a, LoopExecution>;
}
#[derive(Default)]
pub struct RuntimeRegistry {
    runtimes: Vec<Box<dyn LoopRuntime>>,
    indexes: BTreeMap<String, usize>,
}
impl RuntimeRegistry {
    pub fn register(&mut self, runtime: Box<dyn LoopRuntime>) -> Result<()> {
        if runtime.id().is_empty() || runtime.version().is_empty() {
            return Err(Error::new("runtime requires an id and version"));
        }
        if self.indexes.contains_key(runtime.id()) {
            return Err(Error::new(format!(
                "Runtime '{}' is already registered.",
                runtime.id()
            )));
        }
        self.indexes
            .insert(runtime.id().into(), self.runtimes.len());
        self.runtimes.push(runtime);
        Ok(())
    }
    pub fn list(&self) -> Value {
        json!(self
            .runtimes
            .iter()
            .map(|r| json!({"id": r.id(), "version": r.version()}))
            .collect::<Vec<_>>())
    }
    pub fn get_mut(&mut self, id: &str) -> Result<&mut Box<dyn LoopRuntime>> {
        let index = *self
            .indexes
            .get(id)
            .ok_or_else(|| Error::new(format!("Unknown runtime '{id}'.")))?;
        Ok(&mut self.runtimes[index])
    }
}
#[derive(Default)]
pub struct ClassicRuntime;
impl LoopRuntime for ClassicRuntime {
    fn id(&self) -> &str {
        "classic"
    }
    fn version(&self) -> &str {
        "0.1.0-foundation"
    }
    fn run<'a>(
        &'a mut self,
        request: &'a LoopRequest,
        host: &'a mut dyn RuntimeHost,
        observer: &'a mut dyn RuntimeObserver,
        cancellation: &'a CancellationToken,
    ) -> PortFuture<'a, LoopExecution> {
        Box::pin(async move {
            let mut report = LoopReport {
                status: LoopStatus::Exhausted,
                runtime: self.id().into(),
                runtime_version: self.version().into(),
                trace_ref: request.trace_ref().clone(),
                iterations: 0,
                model_calls: 0,
                capability_calls: 0,
                outcome: None,
                error: None,
                history: vec![json!({"role": "user", "content": request.wire()["input"]})],
            };
            let mut sequence = 0;
            let mut evidence = Vec::new();
            let mut emit = |event_type: &str, payload: Value| -> Result<()> {
                let event = LoopEvent {
                    channel: "runtime".into(),
                    event_id: format!("{}:classic:{sequence}", request.trace_ref()),
                    event_type: event_type.into(),
                    run_id: request.trace_ref().clone(),
                    sequence,
                    runtime: "classic".into(),
                    payload,
                };
                evidence.push(observer.emit(&event)?);
                sequence += 1;
                Ok(())
            };
            emit("run_started", json!({"task_id": request.task_ref()}))?;
            while report.iterations < request.max_steps().get() {
                if cancellation.requested() {
                    report.status = LoopStatus::Cancelled;
                    emit("run_cancelled", json!({"iteration": report.iterations}))?;
                    return Ok(LoopExecution {
                        report,
                        evidence_refs: evidence,
                    });
                }
                report.iterations += 1;
                let model = dispatch_host_carrier(
                    host,
                    &Carrier::Model,
                    request,
                    cancellation,
                    json!({"history": report.history, "iteration": report.iterations})
                        .as_object()
                        .expect("object")
                        .clone(),
                )
                .await;
                let model = match model {
                    Ok(value) => value,
                    Err(error) => {
                        let aborted =
                            cancellation.requested() || matches!(error, HostError::Aborted(_));
                        if aborted {
                            report.status = LoopStatus::Cancelled;
                            emit("run_cancelled", json!({"iteration": report.iterations}))?;
                        } else {
                            report.status = LoopStatus::Failed;
                            report.error = Some(error.message().into());
                            emit(
                                "run_failed",
                                json!({"iteration": report.iterations, "error": error.message()}),
                            )?;
                        }
                        return Ok(LoopExecution {
                            report,
                            evidence_refs: evidence,
                        });
                    }
                };
                report.model_calls += 1;
                let mut message = json!({"role": "assistant"});
                if let Some(fields) = model.as_object() {
                    message
                        .as_object_mut()
                        .expect("object")
                        .extend(fields.clone());
                }
                report.history.push(message);
                if let Some(calls) = model
                    .get("capabilityCalls")
                    .and_then(Value::as_array)
                    .filter(|calls| !calls.is_empty())
                {
                    for call in calls {
                        if cancellation.requested() {
                            report.status = LoopStatus::Cancelled;
                            emit(
                                "run_cancelled",
                                json!({"iteration": report.iterations, "during": "capability"}),
                            )?;
                            return Ok(LoopExecution {
                                report,
                                evidence_refs: evidence,
                            });
                        }
                        let carrier = Carrier::from_legacy(
                            &json!({"kind": "capability", "name": call["name"], "args": call["args"]}),
                        );
                        let result = match carrier {
                            Ok(carrier) => {
                                dispatch_host_carrier(
                                    host,
                                    &carrier,
                                    request,
                                    cancellation,
                                    Map::new(),
                                )
                                .await
                            }
                            Err(error) => Err(HostError::from(error)),
                        };
                        let result = result
                            .unwrap_or_else(|error| json!({"ok": false, "error": error.message()}));
                        report.capability_calls += 1;
                        let mut message = json!({"role": "capability", "callId": call.get("id").filter(|v| !v.is_null()).unwrap_or(&Value::Null), "result": result});
                        if let Some(name) = call.get("name") {
                            message["name"] = name.clone();
                        }
                        report.history.push(message);
                    }
                    continue;
                }
                let follow_up = dispatch_host_carrier(
                    host,
                    &Carrier::Human {
                        input_kind: Some("follow_up".into()),
                    },
                    request,
                    cancellation,
                    json!({"history": report.history})
                        .as_object()
                        .expect("object")
                        .clone(),
                )
                .await;
                match follow_up {
                    Ok(value) if !value.is_null() => {
                        report
                            .history
                            .push(json!({"role": "user", "content": value}));
                        continue;
                    }
                    Ok(_) => {}
                    Err(error) => {
                        let aborted =
                            cancellation.requested() || matches!(error, HostError::Aborted(_));
                        if aborted {
                            report.status = LoopStatus::Cancelled;
                            emit("run_cancelled", json!({"iteration": report.iterations}))?;
                        } else {
                            report.status = LoopStatus::Failed;
                            report.error = Some(error.message().into());
                            emit(
                                "run_failed",
                                json!({"iteration": report.iterations, "error": error.message()}),
                            )?;
                        }
                        return Ok(LoopExecution {
                            report,
                            evidence_refs: evidence,
                        });
                    }
                }
                let outcome = model
                    .get("content")
                    .filter(|v| !v.is_null())
                    .cloned()
                    .unwrap_or(model);
                report.status = LoopStatus::Completed;
                report.outcome = Some(outcome.clone());
                emit(
                    "run_completed",
                    json!({"iteration": report.iterations, "outcome": outcome}),
                )?;
                return Ok(LoopExecution {
                    report,
                    evidence_refs: evidence,
                });
            }
            emit(
                "run_exhausted",
                json!({"max_steps": request.max_steps().get()}),
            )?;
            Ok(LoopExecution {
                report,
                evidence_refs: evidence,
            })
        })
    }
}
