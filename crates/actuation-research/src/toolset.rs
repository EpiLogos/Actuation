//! The toolset paradigm: the QL form carried by the agent's tools, not
//! explained to the model.
//!
//! The owner's correction (2026-09-18): the earlier conditions framed the
//! loop as a *task for the model* — control turns in whittled English asking
//! it to reason about positions. The paradigm framing instead organises the
//! praxis itself: the agent's toolset IS the QL form. Six tools — the four
//! world capabilities plus the loop's own two verbs (`situate`, `close`) —
//! whose descriptions name the office each serves in the Px/Px′ format. The
//! model needs no relational system prompt and no control turns: the offices
//! are implicit in what the tools already do, and the engine records the
//! circuit (residues, transitions, determination, closure) from what the
//! tools actually do. The return condition is a tool that must run.
//!
//! Closure-check options (env `QL_CLOSE_CHECK`): `self` (default — the
//! model's own synthesis in the close call, gated by the task's objective
//! checks), `model` (one separate control-style model turn evaluates the
//! synthesis against the success conditions first). Jev is not a closure
//! option: its role is fast classification — the reflect instrument behind
//! the Night (P′) tools, and the batch classification harnesses in the
//! model-types thread.
use crate::tasks::Task;
use crate::{Error, Result};
use actuation_runtime::{
    dispatch_host_carrier, CancellationToken, Carrier, LoopEvent, LoopRequest, RuntimeHost,
    RuntimeObserver,
};
use serde_json::{json, Map, Value};

/// The founding toolset: four world capabilities plus the loop's own two
/// verbs. Each description carries the office it serves in the Px/Px′
/// format — the law is implicit in the toolset, never explained in prose.
pub const TOOLSET_TOOLS: &[(&str, &str)] = &[
    (
        "list_files",
        "See what the workspace holds. This takes in the field as given material (P1); what comes back reads as discovery (P1').",
    ),
    (
        "read_file",
        "Read one file's content into the work as material and evidence (P1). What comes back reads as the given (P1').",
    ),
    (
        "write_file",
        "Transform the ground: create or change one file (P2). What comes back reads as the effect made (P2').",
    ),
    (
        "run_tests",
        "Evaluate the built form against the whole: run the workspace test suite (P4). What comes back reads as whole-relative evaluation (P4').",
    ),
    (
        "situate",
        "The return reading: state where the work now stands. Positions: P0 frame, P1 material, P2 effect, P3 form, P4 evaluation, P5 determination; a prime marks the return reading of a position (P1'). Args: {\"position\": \"P2\"}.",
    ),
    (
        "close",
        "The return condition: when the bounded request is realised, close the work at determination (P5); what passes reads as the return (P5'). Args: {\"synthesis\": string} — what is actually realised, in plain text. The workspace checks run on close.",
    ),
];

/// The capability supply presented to the model: name plus description pairs.
pub fn toolset_capability_supply() -> Value {
    Value::Array(
        TOOLSET_TOOLS
            .iter()
            .map(|(name, description)| json!({"name": name, "description": description}))
            .collect(),
    )
}

/// Route one: the pure explicate four — the world tools with their office
/// law, no loop verbs. The work ends the way classic ends.
pub fn tagged_tools() -> &'static [(&'static str, &'static str)] {
    &TOOLSET_TOOLS[..4]
}

pub fn tagged_capability_supply() -> Value {
    Value::Array(
        tagged_tools()
            .iter()
            .map(|(name, description)| json!({"name": name, "description": description}))
            .collect(),
    )
}

/// Which of the two routes the loop runs: the full return loop (4+2) or the
/// tagged explicate four.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    ReturnLoop,
    Tagged,
}
impl Route {
    pub fn name(self) -> &'static str {
        match self {
            Self::ReturnLoop => "ql-toolset",
            Self::Tagged => "ql-tagged",
        }
    }
}

/// The Night (P′) face: six reflect tools, the conjugates of the Day six.
/// Each call carries the model's own disclosure; the jev reflect instrument
/// decides alongside it; results return the full MEF giving, never bare
/// codes. The descriptions name each tool's office — the law stays in the
/// toolset, never in prose to the model.
pub const NIGHT_TOOLS: [(&str, &str); 6] = [
    (
        "reflect_full_text",
        "The Night reading of a presented whole: how the whole text stands across all twelve lenses at their six articulations — the complete giving returned in full. Pass your own reading of the text as your disclosure; the instrument decides alongside it. Args: {\"subject\": text, \"own_disclosure\": optional string}.",
    ),
    (
        "reflect_right_frame",
        "The one lens most useful for reading this subject now — the frame that, taken up, does the most work. Returns the selected lens in its full giving with the complete distribution. Your candidate frame may ride the call as disclosure. Args: {\"subject\": string, \"own_disclosure\": optional string}.",
    ),
    (
        "reflect_being",
        "The subject read through Square A (its titles: Being, Becoming, Knowing) — three paired crossings of the base alignment, each read through all twelve lenses, returned in full. Args: {\"subject\": string, \"own_disclosure\": optional string}.",
    ),
    (
        "reflect_becoming",
        "The subject read through Square B (its titles: Articulation, Encounter, Becoming) — three paired crossings of the offset alignment, each read through all twelve lenses, returned in full. Args: {\"subject\": string, \"own_disclosure\": optional string}.",
    ),
    (
        "reflect_knowing",
        "The subject read through Square C (its titles: Text/Texture, and two held open) — three paired crossings of the mirror alignment, each read through all twelve lenses, returned in full. Args: {\"subject\": string, \"own_disclosure\": optional string}.",
    ),
    (
        "reflect_resonant_frames",
        "The ranked relation among frames for this subject — the full ordering of which frames resonate and how strongly, to name the ones worth taking deeper. Args: {\"subject\": string, \"own_disclosure\": optional string}.",
    ),
];

/// The full P+P′ supply: the Day six plus the Night six (`ql-twelve`).
pub fn night_capability_supply() -> Value {
    Value::Array(
        TOOLSET_TOOLS
            .iter()
            .chain(NIGHT_TOOLS.iter())
            .map(|(name, description)| json!({"name": name, "description": description}))
            .collect(),
    )
}

/// The explicate eight: the four world tools plus the Night middle (three
/// square readings and the mediator) — no return verbs (`ql-eight`).
pub fn eight_capability_supply() -> Value {
    Value::Array(
        TOOLSET_TOOLS
            .iter()
            .take(4)
            .chain(NIGHT_TOOLS.iter().skip(2))
            .map(|(name, description)| json!({"name": name, "description": description}))
            .collect(),
    )
}

/// The static office law: which office a world tool serves when it runs.
/// The two loop verbs are settled by their own calls, not by this map.
pub fn tool_office(name: &str) -> Option<u8> {
    match name {
        "list_files" | "read_file" => Some(1),
        "write_file" => Some(2),
        "run_tests" => Some(4),
        _ => None,
    }
}

fn position_of(spec: &Value) -> Result<(u8, bool)> {
    let raw = spec
        .as_str()
        .ok_or_else(|| Error::new("situate requires a position string"))?;
    let (base, prime) = match raw.strip_suffix('\'') {
        Some(base) => (base, true),
        None => (raw, false),
    };
    let digit = base
        .strip_prefix('P')
        .and_then(|d| d.parse::<u8>().ok())
        .ok_or_else(|| Error::new("situate position must be P0..P5 (optionally P-prime)"))?;
    if digit > 5 {
        return Err(Error::new("situate position must be P0..P5"));
    }
    Ok((digit, prime))
}

/// Everything the loop needs beyond the host: the task (for objective checks
/// at close), the world (for the closing snapshot), the starting workspace
/// snapshot, and the request wire.
pub struct ToolsetContext<'a> {
    pub request: &'a LoopRequest,
    pub task: Task,
    pub world: crate::world::World,
    pub node: Option<std::path::PathBuf>,
    pub before: Value,
    /// Which route the loop runs: the full return loop or the tagged four.
    pub route: Route,
    /// The Night (P′) tools ride the supply when true (`ql-eight`,
    /// `ql-twelve`).
    pub night: bool,
    /// `self` (default) or `model`.
    pub close_check: CloseCheck,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseCheck {
    Self_,
    Model,
}
impl CloseCheck {
    pub fn from_env() -> Self {
        match std::env::var("QL_CLOSE_CHECK").as_deref() {
            Ok("model") => Self::Model,
            _ => Self::Self_,
        }
    }
}

pub struct ToolsetOutcome {
    pub report: Value,
    pub evidence_refs: Vec<actuation_core::ExternalRef>,
}

/// The toolset loop: a classic-shaped tool loop whose tools carry the QL
/// form. The model drives; the engine records the circuit from tool usage;
/// the return condition is terminal and check-gated.
#[allow(clippy::drop_non_drop, dropping_copy_types)]
pub async fn run_toolset(
    ctx: ToolsetContext<'_>,
    host: &mut dyn RuntimeHost,
    observer: &mut dyn RuntimeObserver,
    cancellation: &CancellationToken,
) -> ToolsetOutcome {
    let trace = ctx.request.trace_ref().clone();
    let circuit_id = format!("{trace}:c0");
    let prompt = ctx.request.wire()["input"].clone();
    let mut history = vec![json!({"role": "user", "content": prompt})];
    let mut residues: Vec<Value> = vec![json!({
        "id": format!("{circuit_id}:res:frame"), "position": 0, "kind": "frame",
        "value": ctx.request.wire()["input"], "invalidated": false})];
    let mut transitions: Vec<Value> = vec![];
    let mut active_position: u8 = 0;
    let mut sequence: u64 = 0;
    let mut evidence: Vec<actuation_core::ExternalRef> = Vec::new();
    let mut model_calls: u64 = 0;
    let mut capability_calls: u64 = 0;
    let mut iterations: u64 = 0;
    let mut premature_deliveries: u64 = 0;
    let mut night_calls: u64 = 0;
    let mut status = "failed";
    let mut error: Option<String> = None;
    let mut outcome: Value = Value::Null;
    let mut closure: Value = Value::Null;
    let mut determination: Value = Value::Null;
    let mut closed_via_close = false;
    let runtime_name = if ctx.night {
        if ctx.route == Route::ReturnLoop {
            "ql-twelve"
        } else {
            "ql-eight"
        }
    } else {
        ctx.route.name()
    };
    let max_steps = ctx.request.wire()["maxSteps"].as_u64().unwrap_or(64);

    let mut record = |event_type: &str, mut payload: Value| {
        if let Some(obj) = payload.as_object_mut() {
            obj.entry("circuit_id").or_insert(json!(circuit_id));
        }
        let event = LoopEvent {
            channel: "runtime-semantic".into(),
            event_id: format!("{trace}:toolset:{sequence}"),
            event_type: event_type.into(),
            run_id: trace.clone(),
            sequence,
            runtime: runtime_name.into(),
            payload,
        };
        // Persistence failures cannot abort the ledger; the sink's own
        // failure flag (CheckedObserver) still fails the run at the seam.
        if let Ok(evidence_line) = observer.emit(&event) {
            evidence.push(evidence_line);
        }
        sequence += 1;
    };
    record(
        "run_started",
        json!({"task_id": ctx.request.wire()["taskId"], "paradigm": "toolset"}),
    );
    record(
        "circuit_started",
        json!({"frame": ctx.request.wire()["input"], "active_position": 0}),
    );
    let settle = |residues: &mut Vec<Value>,
                  transitions: &mut Vec<Value>,
                  active: &mut u8,
                  position: u8,
                  kind: &str,
                  value: Value,
                  by: &str| {
        residues.push(json!({
            "id": format!("{circuit_id}:res:{}", residues.len()),
            "position": position, "kind": kind, "value": value, "invalidated": false}));
        transitions.push(json!({"from": *active, "to": position, "by": by}));
        *active = position;
    };

    while iterations < max_steps {
        if cancellation.requested() {
            status = "cancelled";
            break;
        }
        iterations += 1;
        let model = match dispatch_host_carrier(
            host,
            &Carrier::Model,
            ctx.request,
            cancellation,
            json!({"history": history, "iteration": iterations})
                .as_object()
                .expect("object")
                .clone(),
        )
        .await
        {
            Ok(value) => value,
            Err(e) => {
                status = "failed";
                error = Some(e.message().to_string());
                record(
                    "run_failed",
                    json!({"iteration": iterations, "error": e.message()}),
                );
                break;
            }
        };
        model_calls += 1;
        let mut message = json!({"role": "assistant"});
        if let Some(fields) = model.as_object() {
            message
                .as_object_mut()
                .expect("object")
                .extend(fields.clone());
        }
        history.push(message);
        record(
            "act_created",
            json!({"act_id": format!("{circuit_id}:act:{}", iterations),
                   "carrier": "model",
                   "capability_calls": model.get("capabilityCalls").cloned().unwrap_or(json!([]))}),
        );
        let calls = model
            .get("capabilityCalls")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if calls.is_empty() {
            // Route one ends the way classic ends: bare content is a
            // delivery. The ledger stays; no return condition exists to run.
            if ctx.route != Route::ReturnLoop {
                status = "completed";
                outcome = model.get("content").cloned().unwrap_or(Value::Null);
                record(
                    "run_completed",
                    json!({"outcome": outcome, "via": "bare-content"}),
                );
                break;
            }
            // The return condition is a tool that must run: bare content
            // cannot close the work. Repair-nudge twice, then fail honestly.
            premature_deliveries += 1;
            if premature_deliveries > 2 {
                status = "failed";
                error = Some(
                    "the return condition never ran: the model delivered content without calling close"
                        .into(),
                );
                record(
                    "run_failed",
                    json!({"reason": "return-condition-never-ran"}),
                );
                break;
            }
            history.push(json!({"role": "user", "content":
                "The work cannot end with content alone. When the bounded request is realised, run the close tool with {\"synthesis\": ...}; otherwise continue with the tools."}));
            continue;
        }
        let mut ended = false;
        for call in &calls {
            let name = call["name"].as_str().unwrap_or_default().to_owned();
            let args = call.get("args").cloned().unwrap_or(json!({}));
            if ctx.route == Route::Tagged && matches!(name.as_str(), "close" | "situate") {
                // Route one advertises neither verb; refuse rather than
                // dispatch a capability the workspace does not have.
                history.push(json!({"role": "capability", "name": name,
                    "result": {"ok": false, "error": "no such tool in the tagged route"}}));
                continue;
            }
            if ctx.night && name.starts_with("reflect_") {
                // A Night tool: the reading is served by the jev reflect
                // instrument with the model's own disclosure riding the
                // call. It records as a residue without moving the active
                // position — a reading is not a position change. An
                // unavailable instrument is an ordinary tool error, not a
                // trial failure.
                night_calls += 1;
                let subject = args
                    .get("subject")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let own = args.get("own_disclosure").and_then(Value::as_str);
                let instrument_mode = match name.as_str() {
                    "reflect_full_text" => "full_text",
                    "reflect_right_frame" => "right_frame",
                    "reflect_being" => "being",
                    "reflect_becoming" => "becoming",
                    "reflect_knowing" => "knowing",
                    "reflect_resonant_frames" => "resonant",
                    _ => "unknown",
                };
                let reading = if subject.trim().is_empty() {
                    Err(Error::new(
                        "{name} requires a subject".replace("{name}", &name),
                    ))
                } else if instrument_mode == "unknown" {
                    Err(Error::new(format!("no reflect instrument for {name}")))
                } else {
                    run_reflect(instrument_mode, subject, own)
                };
                match reading {
                    Ok(reading) => {
                        residues.push(json!({
                            "id": format!("{circuit_id}:res:{}", residues.len()),
                            "position": active_position, "kind": "night-reading",
                            "value": reading, "invalidated": false}));
                        history
                            .push(json!({"role": "capability", "name": name, "result": reading}));
                        record(
                            "night_reading_taken",
                            json!({
                            "tool": name,
                            "subject_chars": reading["subject_chars"],
                            "latency_ms": reading["latency_ms"]}),
                        );
                    }
                    Err(e) => {
                        history.push(json!({"role": "capability", "name": name,
                            "result": {"ok": false, "error": e.to_string()}}));
                        record("night_reading_failed", json!({"error": e.to_string()}));
                    }
                }
                continue;
            }
            if name == "close" {
                let synthesis = args
                    .get("synthesis")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                determination = json!({
                    "synthesis": synthesis,
                    "requested_outcome": "close",
                    "via": "close tool"});
                record(
                    "determination_proposed",
                    json!({"determination": determination}),
                );
                let after = match ctx.world.verify_root().and_then(|()| ctx.world.snapshot()) {
                    Ok(after) => after,
                    Err(e) => {
                        status = "failed";
                        error = Some(format!("closing snapshot unavailable: {e}"));
                        record(
                            "run_failed",
                            json!({"error": error.clone().unwrap_or_default()}),
                        );
                        ended = true;
                        break;
                    }
                };
                let verification = ctx
                    .task
                    .verify(&ctx.world, &ctx.before, &after, ctx.node.as_deref())
                    .unwrap_or_else(|e| {
                        json!({"status": "failed", "error": e.to_string(), "objective_checks_pass": false})
                    });
                let mut checks = verification["objective_checks_pass"] == true;
                let mut refusal: Option<String> = None;
                match ctx.close_check {
                    CloseCheck::Model => {
                        let check = dispatch_host_carrier(
                            host,
                            &Carrier::Model,
                            ctx.request,
                            cancellation,
                            json!({"series1Control": {
                                "purpose": "ql-close-check",
                                "system": "You evaluate one proposed closure. Return exactly one JSON object: {\"verdict\": \"close\"|\"reopen\", \"rationale\": string}. Reopen when the synthesis does not actually realise the success conditions.",
                                "prompt": json!({"success_conditions": ctx.request.wire()["successConditions"],
                                                 "synthesis": synthesis}).to_string(),
                                "circuit_ref": circuit_id}})
                                .as_object()
                                .expect("object")
                                .clone(),
                        )
                        .await;
                        model_calls += 1;
                        let evaluated = match check {
                            Ok(v) => v.get("control").cloned().unwrap_or(Value::Null),
                            Err(e) => json!({"verdict": "reopen", "rationale": e.message()}),
                        };
                        if evaluated["verdict"] == json!("reopen") {
                            checks = false;
                            refusal = Some(format!(
                                "model close-check refused the synthesis: {}",
                                evaluated["rationale"].as_str().unwrap_or_default()
                            ));
                            record("closure_refused", json!({"evaluation": evaluated}));
                        } else {
                            record("closure_evaluated", json!({"evaluation": evaluated}));
                        }
                    }
                    CloseCheck::Self_ => {}
                }
                let accepted = checks;
                closure = json!({
                    "success_state": if accepted {"true"} else {"false"},
                    "inspection": {"objective_checks_pass": checks},
                    "refusal": refusal,
                    "closed_at_position": 5});
                settle(
                    &mut residues,
                    &mut transitions,
                    &mut active_position,
                    5,
                    "determination",
                    determination.clone(),
                    "close",
                );
                if accepted {
                    status = "completed";
                    closed_via_close = true;
                    outcome = json!(synthesis);
                    record("circuit_closed", json!({"closed_at_position": 5}));
                    record("run_completed", json!({"outcome": outcome}));
                } else {
                    status = "failed";
                    error = Some(refusal.clone().unwrap_or_else(|| {
                        "the return condition ran but the workspace checks do not pass".into()
                    }));
                    record("closure_refused", json!({"verification": verification}));
                    record("run_failed", json!({"reason": "closure-refused"}));
                }
                ended = true;
                break;
            }
            if name == "situate" {
                match position_of(args.get("position").unwrap_or(&Value::Null)) {
                    Ok((position, prime)) => {
                        settle(
                            &mut residues,
                            &mut transitions,
                            &mut active_position,
                            position,
                            "return-reading",
                            json!({"claimed": args.get("position").cloned().unwrap_or(Value::Null),
                                   "prime": prime}),
                            "situate",
                        );
                        history.push(json!({"role": "capability", "name": "situate",
                            "result": {"ok": true, "position": args.get("position").cloned().unwrap_or(Value::Null)}}));
                        record(
                            "return_interpreted",
                            json!({"by": "situate tool", "position": position, "prime": prime}),
                        );
                    }
                    Err(e) => {
                        history.push(json!({"role": "capability", "name": "situate",
                            "result": {"ok": false, "error": e.to_string()}}));
                    }
                }
                continue;
            }
            // A world tool: dispatch, then settle at its office by law.
            let carrier = match Carrier::from_legacy(
                &json!({"kind": "capability", "name": name, "args": args}),
            ) {
                Ok(c) => c,
                Err(e) => {
                    status = "failed";
                    error = Some(e.to_string());
                    record("run_failed", json!({"error": e.to_string()}));
                    ended = true;
                    break;
                }
            };
            let result =
                dispatch_host_carrier(host, &carrier, ctx.request, cancellation, Map::new()).await;
            let result = result.unwrap_or_else(|e| json!({"ok": false, "error": e.message()}));
            capability_calls += 1;
            history.push(json!({"role": "capability", "name": name, "result": result}));
            if let Some(position) = tool_office(&name) {
                let kind = match position {
                    1 => "material",
                    2 => "effect",
                    4 => "evaluation",
                    _ => "residue",
                };
                settle(
                    &mut residues,
                    &mut transitions,
                    &mut active_position,
                    position,
                    kind,
                    result,
                    &name,
                );
                record(
                    "return_interpreted",
                    json!({"by": "tool office law", "tool": name, "position": position}),
                );
            }
        }
        if ended {
            break;
        }
    }
    if status != "completed" && status != "failed" && status != "cancelled" {
        status = "exhausted";
        error = Some(if ctx.route == Route::ReturnLoop {
            format!("max_steps {max_steps} ran out before the return condition closed the work")
        } else {
            format!("max_steps {max_steps} ran out before the work was delivered")
        });
        record("run_exhausted", json!({"max_steps": max_steps}));
    }
    drop(record);
    drop(settle);
    let report = json!({
        "status": status,
        "runtime": runtime_name,
        "runtimeVersion": if ctx.night { "0.1.0-night" } else if ctx.route == Route::ReturnLoop { "0.1.0-toolset" } else { "0.1.0-tagged" },
        "trace_ref": trace,
        "iterations": iterations,
        "model_calls": model_calls,
        "capability_calls": capability_calls,
        "night_calls": night_calls,
        "outcome": outcome,
        "error": error,
        "history": history,
        "circuits": [{
            "id": circuit_id,
            "face": "direct",
            "active_position": active_position,
            "closure_state": if closed_via_close {"closed"} else {"open"},
            "residues": residues,
            "trajectory": transitions,
        }],
        "closure": closure,
        "determination": determination,
        "paradigm": if ctx.route == Route::ReturnLoop { "toolset" } else { "tagged" },
    });
    ToolsetOutcome {
        report,
        evidence_refs: evidence,
    }
}

/// Run the reflect instrument (QL_REFLECT_BIN) for one Night tool call. The
/// instrument is a process, like the owner instrument: request JSON on
/// stdin, one JSON reading on stdout, fail-closed.
fn run_reflect(mode: &str, subject: &str, own_disclosure: Option<&str>) -> Result<Value> {
    let program = std::env::var("QL_REFLECT_BIN")
        .ok()
        .filter(|p| !p.is_empty())
        .ok_or_else(|| {
            Error::new(
                "the reflect tools require QL_REFLECT_BIN (path to the jev reflect instrument)",
            )
        })?;
    let scratch =
        tempfile::tempdir().map_err(|e| Error::new(format!("reflect scratch unavailable: {e}")))?;
    // The instrument reads one JSON request on stdin.
    let input = serde_json::to_vec(&json!({
        "mode": mode,
        "subject": subject,
        "own_disclosure": own_disclosure,
    }))
    .map_err(|e| Error::new(format!("reflect request not built: {e}")))?;
    // The instrument is a Node script resolved through PATH: inherit the
    // host's PATH (an empty environment leaves `env node` unresolvable).
    let environment = [("PATH".to_owned(), std::env::var("PATH").unwrap_or_default())]
        .into_iter()
        .collect();
    let spec = crate::process::ProcessSpec {
        program: std::path::PathBuf::from(program),
        args: vec![],
        cwd: scratch.path().to_owned(),
        environment,
        timeout_ms: 30_000,
        output_limit: 4 * 1024 * 1024,
    };
    let r = spec.run(&input)?;
    if r.code != Some(0) {
        let why: String = r.stderr.chars().take(300).collect();
        return Err(Error::new(format!("instrument exited {:?}: {why}", r.code)));
    }
    serde_json::from_str(&r.stdout)
        .map_err(|_| Error::new("reflect instrument returned invalid JSON"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn founding_toolset_is_six_tools_with_office_law_in_descriptions() {
        assert_eq!(
            TOOLSET_TOOLS.len(),
            6,
            "the founding set: 4 world + 2 loop verbs"
        );
        let names: Vec<&str> = TOOLSET_TOOLS.iter().map(|(n, _)| *n).collect();
        for world in ["list_files", "read_file", "write_file", "run_tests"] {
            assert!(names.contains(&world), "{world} must stay in the set");
        }
        assert!(
            names.contains(&"situate"),
            "the return reading must be a tool"
        );
        assert!(
            names.contains(&"close"),
            "the return condition must be a tool"
        );
        for (name, description) in TOOLSET_TOOLS {
            assert!(
                description.contains("(P"),
                "{name}'s description must carry the Px/Px′ office law"
            );
        }
    }

    #[test]
    fn tagged_route_is_the_explicate_four_with_their_office_law() {
        assert_eq!(tagged_tools().len(), 4, "route one: the four world tools");
        let names: Vec<&str> = tagged_tools().iter().map(|(n, _)| *n).collect();
        assert_eq!(
            names,
            vec!["list_files", "read_file", "write_file", "run_tests"]
        );
        for (name, description) in tagged_tools() {
            assert!(
                description.contains("(P"),
                "{name}'s description must carry the Px/Px′ office law"
            );
        }
    }

    #[test]
    fn night_face_is_six_reflect_tools_and_the_conditions_split_p_and_p_prime() {
        assert_eq!(NIGHT_TOOLS.len(), 6, "the Night face: six reflect tools");
        let names: Vec<&str> = NIGHT_TOOLS.iter().map(|(n, _)| *n).collect();
        assert_eq!(
            names,
            vec![
                "reflect_full_text",
                "reflect_right_frame",
                "reflect_being",
                "reflect_becoming",
                "reflect_knowing",
                "reflect_resonant_frames",
            ]
        );
        let twelve = night_capability_supply();
        assert_eq!(twelve.as_array().unwrap().len(), 12, "P six + P' six");
        let eight = eight_capability_supply();
        let eight_names: Vec<&str> = eight
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["name"].as_str().unwrap())
            .collect();
        assert_eq!(eight_names.len(), 8, "P-world four + Night middle four");
        assert!(
            !eight_names.contains(&"situate"),
            "the eight is explicate: no return reading"
        );
        assert!(
            !eight_names.contains(&"close"),
            "the eight is explicate: no return condition"
        );
        assert!(
            !eight_names.contains(&"reflect_full_text"),
            "full_text is an implicate pole"
        );
        assert!(
            !eight_names.contains(&"reflect_right_frame"),
            "right_frame is an implicate pole"
        );
        assert!(
            eight_names.contains(&"reflect_resonant_frames"),
            "the mediator is explicate"
        );
    }

    #[test]
    fn tool_office_maps_the_world_tools_only() {
        assert_eq!(tool_office("read_file"), Some(1));
        assert_eq!(tool_office("list_files"), Some(1));
        assert_eq!(tool_office("write_file"), Some(2));
        assert_eq!(tool_office("run_tests"), Some(4));
        assert_eq!(tool_office("situate"), None);
        assert_eq!(tool_office("close"), None);
    }

    #[test]
    fn positions_accept_direct_and_prime_forms_only() {
        assert_eq!(position_of(&json!("P2")).unwrap(), (2, false));
        assert_eq!(position_of(&json!("P1'")).unwrap(), (1, true));
        assert!(position_of(&json!("P6")).is_err());
        assert!(position_of(&json!("material")).is_err());
        assert!(position_of(&json!(2)).is_err());
    }
}
