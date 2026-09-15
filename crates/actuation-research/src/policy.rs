//! Model-driven research policy. Model/body selection remains with the supplier;
//! every control turn uses Actuation's existing RuntimeHost. Formal validity is
//! checked by the Engine's bound QL owner, not inferred from the controller.
//!
//! Behavioural parity with the staged JS loop (PR #79 branch, merged as #81 and
//! advanced on 2026-09-13): the Relational Logos standing system prompt, the
//! owner's conjugacy law with the P′ face executing on every Deep closure,
//! per-position allowances with typed refusal, typed goal/exclusion stipulations
//! with closure verdicts, never-close-on-empty synthesis, and closure-request
//! carriers routed to determination.
use crate::{evidence::candidate_boundary, execution::block_on, relational::*, Error, Result};
use actuation_runtime::{HostCall, RuntimeHost};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

/// The QL agent's standing system prompt, supplied by the owner on 2026-09-13.
const RELATIONAL_SYSTEM: &str = include_str!("ql-relational-system-prompt.md");

/// Conjugacy law (owner, 2026-09-13): P and P′ are not distinct circuits but
/// directional views on the same psychoid #0–#5 — the outward walk and the
/// return reading of one field. A model looking back from P5 toward ground is
/// therefore already operating in the conjugate direction.
const CONJUGATE_DIRECTION_LAW: &str = "Positions P0..P5 are directional views on one field. The outward reading (P) runs ground toward determination; the return reading (P-prime) is the same positions seen looking back from a later position toward ground. Looking back from a determination toward the frame is already conjugate operation, not a different circuit.";

/// Per-position allowance schedule, declared with the frame (kernel shape:
/// lawful refusal and explicit allowance, never a silent stop). Acts are the
/// loop-native currency; token consumption is metered in the run record.
pub fn default_allowance_schedule() -> AllowanceSchedule {
    AllowanceSchedule::new([
        ("P0", 2),
        ("P1", 6),
        ("P2", 5),
        ("P3", 4),
        ("P4", 4),
        ("P5", 3),
    ])
}

/// Research-shaped work is naturally material-heavy: round 3 (2026-09-13)
/// consumed the P1 allowance exactly on both local-research runs before
/// closing directly P1→P5. The schedule is task-shape aware like every other
/// frame stipulation.
pub fn schedule_for_category(category: &str) -> AllowanceSchedule {
    if category == "local-research" {
        AllowanceSchedule::new([
            ("P0", 2),
            ("P1", 10),
            ("P2", 5),
            ("P3", 4),
            ("P4", 4),
            ("P5", 3),
        ])
    } else {
        default_allowance_schedule()
    }
}

const GRACE_ALLOWANCE: u64 = 2;

#[derive(Clone, Debug)]
pub struct AllowanceSchedule(BTreeMap<String, u64>);
impl AllowanceSchedule {
    pub fn new<const N: usize>(entries: [(&'static str, u64); N]) -> Self {
        Self(BTreeMap::from_iter(
            entries.into_iter().map(|(k, v)| (k.to_owned(), v)),
        ))
    }
    fn limit(&self, position: &str) -> u64 {
        self.0.get(position).copied().unwrap_or(8)
    }
    fn value(&self) -> Value {
        Value::Object(self.0.iter().map(|(k, v)| (k.clone(), json!(v))).collect())
    }
}

const CLOSURE_CONTROL_NAMES: [&str; 7] = [
    "close",
    "stop",
    "finalize",
    "finish",
    "complete",
    "end",
    "propose_closure",
];

/// Task conditions become typed frame-carried constraint bindings (the
/// FullVakBinding pattern — a readable scope does not authorise an action).
pub fn classify_stipulations(conditions: &Value) -> Value {
    let empty = Vec::new();
    let list = conditions.as_array().unwrap_or(&empty);
    Value::Array(
        list.iter()
            .enumerate()
            .map(|(i, v)| {
                let text = v.as_str().unwrap_or_default();
                let exclusion = {
                    let t = text.trim().to_ascii_lowercase();
                    ["do not", "never", "don't", "avoid", "without"]
                        .iter()
                        .any(|prefix| t.starts_with(prefix))
                };
                json!({"id":format!("S{}", i+1), "text":text, "kind":if exclusion {"exclusion"} else {"goal"}})
            })
            .collect(),
    )
}

fn violated_exclusions(stipulations: &Value, verdicts: &Value) -> Vec<String> {
    let mut out = vec![];
    if let Some(list) = stipulations.as_array() {
        for s in list {
            if s["kind"] != "exclusion" {
                continue;
            }
            if let Some(verdicts) = verdicts.as_array() {
                for v in verdicts {
                    if v["id"] == s["id"] && v["verdict"] == "violated" {
                        out.push(s["id"].as_str().unwrap_or_default().to_owned());
                    }
                }
            }
        }
    }
    out
}

fn text(v: &Value, key: &str) -> Result<String> {
    v[key]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| Error::new(format!("controller {key} requires text")))
}
fn position(v: &Value) -> Result<u8> {
    if let Some(n) = v.as_u64().and_then(|n| u8::try_from(n).ok()) {
        return Ok(n);
    }
    v.as_str()
        .and_then(|s| s.strip_prefix('P'))
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Error::new("controller position must be explicit"))
}
fn strings(v: &Value) -> Result<Vec<String>> {
    if v.is_null() {
        return Ok(vec![]);
    }
    v.as_array()
        .ok_or_else(|| Error::new("controller refs require array"))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| Error::new("controller ref requires string"))
        })
        .collect()
}
fn control(
    cx: &PolicyContext<'_>,
    host: &mut dyn RuntimeHost,
    purpose: &str,
    instruction: &str,
    payload: Value,
) -> Result<Value> {
    candidate_boundary(&payload)?;
    let payload = Map::from_iter([(
        "series1Control".into(),
        // Parity: every QL control turn runs under the QL agent's standing
        // relational protocol, with the turn-specific instruction composed on
        // top; classic keeps its plain envelope protocol.
        json!({"purpose":purpose,"system":format!("{RELATIONAL_SYSTEM}\n\n---\n\n{instruction}\nReturn exactly one JSON object, without prose outside it."),"prompt":payload.to_string(),"circuit_ref":cx.circuit.id}),
    )]);
    let response = block_on(host.call_model(HostCall {
        request: cx.request.clone(),
        payload,
        cancellation: cx.cancellation.clone(),
    }))
    .map_err(|e| Error::new(e.message()))?;
    let object = response["control"].as_object().cloned().ok_or_else(|| {
        Error::new(format!(
            "control turn {purpose} did not return a structured control object"
        ))
    })?;
    Ok(Value::Object(object))
}

/// The controller's carrier decision, accepted in both the documented object
/// form and the flat form models naturally return. Shape leniency only: an
/// unknown carrier kind still fails closed.
fn normalise_carrier(decision: &Value) -> Result<Value> {
    let raw = decision
        .get("carrier")
        .cloned()
        .unwrap_or_else(|| json!({"kind":"model"}));
    let (kind, name, args, input) = if raw.is_string() {
        (
            raw.as_str().unwrap_or_default().to_owned(),
            decision
                .get("capability")
                .or_else(|| decision.get("name"))
                .or_else(|| decision.get("tool"))
                .cloned(),
            decision.get("args").cloned().unwrap_or(json!({})),
            decision.get("input").cloned(),
        )
    } else {
        (
            raw["kind"].as_str().unwrap_or_default().to_owned(),
            raw.get("name")
                .or_else(|| raw.get("capability"))
                .or_else(|| raw.get("tool"))
                .cloned(),
            raw.get("args").cloned().unwrap_or(json!({})),
            raw.get("input").cloned(),
        )
    };
    Ok(match kind.as_str() {
        "model" | "" => json!({"kind":"model"}),
        "internal_control" => {
            json!({"kind":"internal_control","name":name,"input":input.unwrap_or_else(|| args.clone())})
        }
        "capability" | "tool" => json!({"kind":"capability","name":name,"args":args}),
        other => {
            return Err(Error::new(format!(
                "QL controller selected unsupported carrier '{other}'"
            )))
        }
    })
}

fn is_closure_control_carrier(carrier: &Value) -> bool {
    carrier["kind"] == "internal_control"
        && carrier
            .get("name")
            .and_then(Value::as_str)
            .map(|n| CLOSURE_CONTROL_NAMES.contains(&n.trim().to_ascii_lowercase().as_str()))
            .unwrap_or(false)
}

pub struct ModelPolicy {
    pub mode: Mode,
    pub schedule: AllowanceSchedule,
    /// Compressed-intent control: interpret-return is decided by the loop's
    /// own law in code — no model call — with the rule and basis recorded in
    /// the witness. The model keeps acts, determination and closure.
    pub compressed_control: bool,
    depth_used: BTreeSet<String>,
    acts_by_position: BTreeMap<String, u64>,
    grace_used: BTreeSet<String>,
    last_context: Option<(
        actuation_runtime::LoopRequest,
        actuation_runtime::CancellationToken,
    )>,
}
impl ModelPolicy {
    pub fn new(mode: Mode) -> Self {
        Self::with_schedule(mode, default_allowance_schedule())
    }
    pub fn with_schedule(mode: Mode, schedule: AllowanceSchedule) -> Self {
        Self {
            mode,
            schedule,
            compressed_control: false,
            depth_used: BTreeSet::new(),
            acts_by_position: BTreeMap::new(),
            grace_used: BTreeSet::new(),
            last_context: None,
        }
    }
    pub fn with_compressed_control(mut self) -> Self {
        self.compressed_control = true;
        self
    }
    /// Allowance is per-position and frame-carried: consumed by acts at the
    /// active position, restated in every control payload, refused with a
    /// typed event on overrun (one recorded grace extension per position).
    fn allowance(&mut self, position: &str) -> (bool, Value) {
        let scheduled = self.schedule.limit(position);
        let consumed = self.acts_by_position.get(position).copied().unwrap_or(0);
        let grace_used = self.grace_used.contains(position);
        let limit = if grace_used {
            scheduled + GRACE_ALLOWANCE
        } else {
            scheduled
        };
        if consumed >= limit {
            if !grace_used {
                self.grace_used.insert(position.to_owned());
            }
            (
                true,
                json!({"position":position,"consumed":consumed,"scheduled":scheduled,
                       "grace_extension":if grace_used {0} else {GRACE_ALLOWANCE}}),
            )
        } else {
            self.acts_by_position
                .insert(position.to_owned(), consumed + 1);
            (false, Value::Null)
        }
    }
    fn consumption(&self) -> Value {
        Value::Object(
            self.acts_by_position
                .iter()
                .map(|(k, v)| (k.clone(), json!(v)))
                .collect(),
        )
    }
}
impl Policy for ModelPolicy {
    fn next_act(
        &mut self,
        cx: &PolicyContext<'_>,
        host: &mut dyn RuntimeHost,
    ) -> Result<Option<Act>> {
        self.last_context = Some((cx.request.clone(), cx.cancellation.clone()));
        let active = format!("P{}", cx.circuit.active_position);
        let stipulations = classify_stipulations(&cx.request.wire()["successConditions"]);
        let (exhausted, refusal) = self.allowance(&active);
        if exhausted {
            // Typed refusal: no model call, a recorded overrun event, and the
            // loop moves to determination through the closure-request path.
            return Ok(Some(Act {
                source_position: Some(cx.circuit.active_position),
                intent: json!(format!(
                    "Allowance at {active} exhausted; routed to determination by the allowance schedule."
                )),
                carrier: json!({"kind":"internal_control","name":"close","input":null}),
                input_residue_refs: vec![],
                nested: None,
                metadata: json!({
                    "closure_request":true,
                    "controller_rationale":"Typed allowance refusal: the position budget is spent; determination must decide whether the realisable intent is achieved.",
                    "allowance_refusal":refusal
                }),
            }));
        }
        let d=control(cx,host,"ql-next-act",&format!("You are controlling a QL-native agent recurrence. Positions are responsibilities, not chronological stages: P0 initiating intent and operative frame; P1 material, evidence and givens; P2 effect, operation and transformation; P3 form, pattern and implementation; P4 whole-relative evaluation, context and adequacy; P5 candidate determination and synthesis. {CONJUGATE_DIRECTION_LAW} Choose the next exterior act appropriate to the currently active position. Return exactly one JSON object of the form {{\"intent\": string, \"carrier\": {{\"kind\": \"model\"|\"capability\"|\"internal_control\", \"name\": <capability id, required when kind is \"capability\">, \"args\": object}}, \"claimed_relation\": string|null, \"rationale\": string}}. The \"internal_control\" kind is only a closure request: use {{\"kind\": \"internal_control\", \"name\": \"close\", \"args\": {{\"reason\": string}}}} when the realisable intent is already achieved and no exterior act remains — do not repeat equivalent acts. Stipulations of kind \"exclusion\" forbid the entire class of action including creating new artifacts: check the carrier choice against every exclusion stipulation before returning. In Deep mode, only at P4, you may add \"deep_operator\": \"depth\" when a genuinely local whole — a sub-question whose independent resolution would materially change the evaluation, resolvable without the parent's transcript — warrants independent treatment at the lemniscate point; depth at #4 is the nesting entry, not a ceremony. Do not force a six-step path."),
            json!({"mode":self.mode,"task":cx.request.wire()["input"],"stipulations":stipulations,
                   "success_conditions":cx.request.wire()["successConditions"],
                   "capabilities":cx.circuit.frame["available_capabilities"],
                   "circuit":cx.circuit.compact(),
                   "budget":{"max_steps":cx.request.wire()["maxSteps"],"allowance":{
                       "schedule":self.schedule.value(),"consumed":self.consumption(),
                       "active_position":active}}}))?;
        if d["deep_operator"] == "depth" {
            if self.mode != Mode::Deep
                || cx.circuit.active_position != 4
                || !self.depth_used.insert(cx.circuit.id.clone())
            {
                return Err(Error::new(
                    "depth not available at this circuit aperture or already used",
                ));
            }
            let aperture=control(cx,host,"ql-depth-aperture","State one bounded local whole whose independent resolution would materially improve the parent evaluation. Return local_whole_intent, selected_residue_refs, success_conditions, capabilities; do not request access absent from the parent.",json!({"circuit":cx.circuit.compact()}))?;
            return Ok(Some(Act {
                source_position: Some(cx.circuit.active_position),
                intent: aperture["local_whole_intent"].clone(),
                carrier: json!({"kind":"child_circuit"}),
                input_residue_refs: vec![],
                nested: Some(NestedRequest {
                    intent: aperture["local_whole_intent"].clone(),
                    selected_residue_refs: strings(&aperture["selected_residue_refs"])?,
                    success_conditions: aperture["success_conditions"]
                        .as_array()
                        .cloned()
                        .unwrap_or_default(),
                    capabilities: strings(&aperture["capabilities"])?,
                    scope: json!("whole"),
                    extension: None,
                    modulation: None,
                }),
                metadata: json!({"deep_operator":"depth","model_control":true}),
            }));
        }
        let carrier = normalise_carrier(&d)?;
        let closure_request = is_closure_control_carrier(&carrier);
        Ok(Some(Act {
            source_position: Some(cx.circuit.active_position),
            intent: if d["intent"].is_null() {
                json!(format!(
                    "Advance the {active} responsibility for the initiating intent."
                ))
            } else {
                d["intent"].clone()
            },
            carrier: carrier.clone(),
            input_residue_refs: strings(&d["input_residue_refs"])?,
            nested: None,
            metadata: {
                let mut metadata = json!({
                    "model_control":true,
                    "claimed_position":d["claimed_position"],
                    "controller_rationale":d["rationale"]
                });
                if closure_request {
                    metadata["closure_request"] = json!(true);
                }
                metadata
            },
        }))
    }
    fn interpret(
        &mut self,
        cx: &PolicyContext<'_>,
        act: &Act,
        difference: &Value,
        host: &mut dyn RuntimeHost,
    ) -> Result<Interpretation> {
        if self.compressed_control && act.metadata["closure_request"] != true {
            // The loop's own law decides the destination in code: delivered
            // realisation with non-empty content is P5; a successful read of
            // unprocessed evidence is P1; a successful mutation or operation
            // is P2; a failed operation returns to P1 with its failure. The
            // model keeps acts, determination and closure.
            let success = difference["operation_success"] == true;
            let content = difference["raw_result"]["content"]
                .as_str()
                .unwrap_or_default();
            let delivered = act.carrier["kind"] == "model" && !content.trim().is_empty();
            let destination = if !success {
                1
            } else if delivered {
                5
            } else if act.carrier["kind"] == "capability" {
                let name = act.carrier["name"].as_str().unwrap_or_default();
                if name == "write_file" {
                    2
                } else {
                    1
                }
            } else {
                2
            };
            return Ok(Interpretation {
                destination,
                rationale: json!(format!(
                    "compressed interpret rule: {}",
                    if !success {
                        "failed operation returns to material"
                    } else if delivered {
                        "delivered realisation of the intent"
                    } else if destination == 2 {
                        "successful mutation is an effect"
                    } else {
                        "successful read of unprocessed evidence is material"
                    }
                )),
                witness: json!({"compressed_control":{"rule_applied":true,"carrier":act.carrier["kind"],"operation_success":success},"observed_position":destination}),
                create: vec![
                    json!({"position":destination,"value":{"difference":difference},"provenance":{"compressed_interpretation":true}}),
                ],
                revise: vec![],
                invalidate: vec![],
            });
        }
        if act.metadata["closure_request"] == true {
            // The controller stated the realisable intent is achieved (or the
            // allowance schedule refused further acts): route straight to
            // determination. Only the P5 propose/evaluate path may establish
            // positive closure.
            let refusal = act.metadata.get("allowance_refusal").cloned();
            return Ok(Interpretation {
                destination: 5,
                rationale: act.metadata["controller_rationale"].clone(),
                witness: {
                    let mut witness = json!({
                        "claimed_position":"P5","observed_position":"P5","closure_request":true,
                        "operation_success":difference["operation_success"]
                    });
                    if let Some(refusal) = refusal {
                        witness["allowance_refusal"] = refusal;
                    }
                    witness
                },
                create: vec![],
                revise: vec![],
                invalidate: vec![],
            });
        }
        let d=control(cx,host,"ql-interpret-return","Interpret the returned difference for the current QL whole. The carrier does NOT determine semantic destination. Worked examples: a successful read of unprocessed evidence belongs at P1 even if the act claimed otherwise; a delivered realisation of the intent belongs at P5; a partial tool result still in use belongs at P2; a model or pattern worth keeping belongs at P3; a whole-relative check belongs at P4. Return exactly one JSON object of the form {\"destination\": \"P0\"|\"P1\"|\"P2\"|\"P3\"|\"P4\"|\"P5\", \"semantic_summary\": string, \"claimed_position\": \"P0\"..\"P5\"|null, \"ambiguity\": string|null, \"rationale\": string}. Choose exactly one destination and explain why. Preserve genuine failure or ambiguity rather than pretending success.",
            json!({"circuit":cx.circuit.compact(),"act":act,"difference":difference,"success_conditions":cx.request.wire()["successConditions"]}))?;
        let destination = position(&d["destination"])?;
        Ok(Interpretation {
            destination,
            rationale: d["rationale"].clone(),
            witness: json!({"model_claimed":d["claimed_position"],"ambiguity":d["ambiguity"],"operation_success":difference["operation_success"]}),
            create: vec![
                json!({"position":destination,"value":{"difference":difference,"semantic_summary":d["semantic_summary"]},"provenance":{"model_interpretation":true}}),
            ],
            revise: vec![],
            invalidate: vec![],
        })
    }
    fn determine(
        &mut self,
        cx: &PolicyContext<'_>,
        host: &mut dyn RuntimeHost,
    ) -> Result<Option<Determination>> {
        let deep_note = if self.mode == Mode::Deep {
            " The backward reading of this determination through the conjugate direction is performed at closure as part of the lane; you need not request it."
        } else {
            ""
        };
        let d=control(cx,host,"ql-propose-determination",&format!("The active responsibility is P5: candidate determination. {CONJUGATE_DIRECTION_LAW} Synthesize what is actually realised relative to the initiating intent and success conditions; reading back from this determination toward the frame is the return direction of the same field. Return exactly one JSON object of the form {{\"synthesis\": string (the realised outcome in plain text; never empty), \"requested_outcome\": \"close\"|\"reopen\", \"claimed_adequacy\": \"adequate\"|\"partial\"|\"inadequate\"|\"unknown\", \"claimed_subject\": string (exactly the task id, or null — free-text subjects cannot be verified), \"evidence_refs\": string[], \"unresolved_refs\": string[]}}. Never fabricate a claimed_state: the workspace digest is measured by the inspector, not asserted by you.{deep_note}"),
            json!({"mode":self.mode,"stipulations":classify_stipulations(&cx.request.wire()["successConditions"]),
                   "success_conditions":cx.request.wire()["successConditions"],"circuit":cx.circuit.compact()}))?;
        let synthesis = ["synthesis", "answer", "content"]
            .iter()
            .find_map(|k| d[k].as_str())
            .unwrap_or_default()
            .to_owned();
        let empty_synthesis = synthesis.trim().is_empty();
        // Closure is a positive determination: it may not close on an empty
        // synthesis. Deep mode always takes the conjugate return — the P′
        // face executes on every closure evaluation per the owner's law.
        let requested = match d["requested_outcome"].as_str() {
            Some("reopen") => Outcome::Reopen,
            _ if self.mode == Mode::Deep => Outcome::Conjugate,
            Some("close") if !empty_synthesis => Outcome::Close,
            _ => Outcome::Reopen,
        };
        let mut unresolved = strings(&d["unresolved_refs"])?;
        if empty_synthesis && !matches!(requested, Outcome::Reopen) {
            unresolved.push("determination-synthesis-empty".into());
        }
        let nested = if matches!(requested, Outcome::Conjugate) {
            let s=control(cx,host,"ql-conjugate-scope","Select a fresh inspection packet for the backward reading: scope whole|current_position, selected_residue_refs, optional pairing_modulation with owner fields family, pair_index, degree and projection_side for D2. Do not invoke a modulation merely because one exists. The new context will not inherit the persuasive direct transcript.",json!({"circuit":cx.circuit.compact(),"determination":d,"synthesis":synthesis}))?;
            Some(NestedRequest {
                intent: json!({"initiating_intent":retained_context(&cx.circuit.frame["initiating_intent"]),"candidate_outcome":synthesis,"office":"return-direction backward reading of the determination"}),
                selected_residue_refs: strings(&s["selected_residue_refs"])?,
                success_conditions: cx.circuit.frame["success_conditions"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default(),
                capabilities: strings(&s["capabilities"])?,
                scope: s["scope"].clone(),
                extension: None,
                modulation: s
                    .get("pairing_modulation")
                    .filter(|v| !v.is_null())
                    .cloned(),
            })
        } else {
            None
        };
        // The engine's positive-closure gate compares claimed_subject against
        // the task id byte-for-byte and claimed_state against the measured
        // workspace digest. The policy holds the task id and cannot compute
        // the digest, so the claim is normalised to the id or dropped rather
        // than letting free-text prose fail the gate.
        let task_id = cx
            .request
            .wire()
            .get("taskId")
            .and_then(Value::as_str)
            .map(str::to_owned);
        let claimed_subject = d["claimed_subject"]
            .as_str()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| task_id.clone().unwrap_or_else(|| s.to_owned()));
        Ok(Some(Determination {
            synthesis: json!(synthesis),
            requested_outcome: requested,
            claimed_subject,
            claimed_state: None,
            evidence_refs: strings(&d["evidence_refs"])?,
            evaluation_refs: cx
                .circuit
                .residues
                .iter()
                .filter(|r| r.kind == "evaluation" && !r.invalidated)
                .map(|r| r.id.clone())
                .collect(),
            unresolved_refs: unresolved,
            nested,
        }))
    }
    fn evaluate_closure(
        &mut self,
        cx: &PolicyContext<'_>,
        d: &Determination,
        inspection: Option<&Inspection>,
        host: &mut dyn RuntimeHost,
    ) -> Result<Verdict> {
        let stipulations = classify_stipulations(&cx.request.wire()["successConditions"]);
        let v=control(cx,host,"ql-evaluate-closure","Evaluate positive QL closure. Do not equate no pending tool call with task completion. Compare initiating Frame/P0, current whole-relative Evaluation/P4 and Determination/P5 — the return direction reads these back from the determination toward the ground; in Deep mode this backward reading is conjugate operation and the supplied inspection is its result. Current inspection is evidence; a supplied/model claim is not inspection. Every stipulation must receive an explicit verdict. Return exactly one JSON object: {\"status\": \"close\"|\"reopen\", \"destination\": \"P0\"..\"P4\" (on reopen), \"task_success\": true|false|unknown, \"stipulation_verdicts\": [{\"id\": string, \"verdict\": \"met\"|\"violated\"|\"untestable\", \"evidence\": string}], \"rationale\": string}. Exclusion stipulations forbid the entire class of action including creating new artifacts.",
            json!({"stipulations":stipulations,"success_conditions":cx.request.wire()["successConditions"],
                   "frame":retained_context(&cx.circuit.frame),"determination":d,"inspection":inspection,
                   "evaluations":cx.circuit.residues.iter().filter(|r|r.kind=="evaluation"&&!r.invalidated).collect::<Vec<_>>()}))?;
        // Engine law: a determination that requested reopening cannot
        // silently receive a close verdict. The reopen plays out as further
        // acts and a fresh determination; the closure evaluator is not asked
        // to overturn it.
        if matches!(d.requested_outcome, Outcome::Reopen) {
            return Ok(Verdict::Reopen {
                destination: 4,
                rationale: json!("determination requested reopening; a fresh determination must follow further acts"),
            });
        }
        let verdicts = v.get("stipulation_verdicts").cloned().unwrap_or(json!([]));
        let violated = violated_exclusions(&stipulations, &verdicts);
        match v["status"].as_str() {
            Some("close") => {
                // A violated exclusion forbids positive success: the
                // determination may close, but it closes as failed and the
                // violation stays on the record.
                let mut rationale = v["rationale"].as_str().unwrap_or_default().to_owned();
                let task_success = if violated.is_empty() {
                    text(&v, "task_success").unwrap_or_else(|_| "unknown".into())
                } else {
                    if !rationale.is_empty() {
                        rationale.push(' ');
                    }
                    rationale.push_str(&format!(
                        "Exclusion stipulations violated: {}.",
                        violated.join(", ")
                    ));
                    "false".to_owned()
                };
                Ok(Verdict::Close {
                    task_success,
                    rationale: json!(rationale),
                })
            }
            Some("reopen") => Ok(Verdict::Reopen {
                destination: if violated.is_empty() {
                    let destination = position(&v["destination"]).unwrap_or(4);
                    if destination == 5 {
                        4
                    } else {
                        destination
                    }
                } else {
                    4
                },
                rationale: json!(format!(
                    "{}{}",
                    v["rationale"].as_str().unwrap_or_default(),
                    if violated.is_empty() {
                        String::new()
                    } else {
                        format!(" Exclusion stipulations violated: {}.", violated.join(", "))
                    }
                )),
            }),
            _ => Err(Error::new(
                "controller closure verdict is not close or reopen",
            )),
        }
    }
    fn conjugate_delta(
        &mut self,
        parent: &Circuit,
        summary: &Value,
        host: &mut dyn RuntimeHost,
    ) -> Result<Value> {
        let (request, cancellation) = self
            .last_context
            .as_ref()
            .ok_or_else(|| Error::new("missing supplied control context"))?;
        // Rebuild a parent request: last_context may belong to the completed
        // child. Never mislabel a returning child controller as the parent.
        let mut wire = request.wire().clone();
        wire["input"] = parent.frame["initiating_intent"].clone();
        wire["id"] = parent.frame["id"].clone();
        wire["taskId"] = parent.frame["id"].clone();
        let request = actuation_runtime::LoopRequest::from_legacy(wire)?;
        let cx = PolicyContext {
            circuit: parent,
            request: &request,
            cancellation,
        };
        let mut d=control(&cx,host,"ql-conjugate-reintegration","Read only the attributable typed result of the completed fresh backward review. Return status confirm|qualify|reopen|invalidate, rationale and target_position P0..P4 for reopening. A child's termination alone is not a positive review.",json!({"direct_circuit_ref":parent.id,"child_summary":summary}))?;
        if matches!(d["status"].as_str(), Some("reopen" | "invalidate")) {
            d["target_position"] = json!(position(&d["target_position"])?);
        }
        Ok(d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stipulations_classify_goal_and_exclusion_kinds() {
        let bindings = classify_stipulations(&json!([
            "Deliver a one-page note answering all four questions.",
            "Do not modify any file.",
            "Never import outside knowledge."
        ]));
        let kinds: Vec<&str> = bindings
            .as_array()
            .unwrap()
            .iter()
            .map(|b| b["kind"].as_str().unwrap())
            .collect();
        assert_eq!(kinds, ["goal", "exclusion", "exclusion"]);
        assert_eq!(bindings[1]["id"], "S2");
    }

    #[test]
    fn violated_exclusions_are_detected_by_id() {
        let stipulations =
            classify_stipulations(&json!(["Answer the questions.", "Do not modify any file."]));
        let verdicts = json!([
            {"id": "S1", "verdict": "met"},
            {"id": "S2", "verdict": "violated", "evidence": "wrote research-note.md"}
        ]);
        assert_eq!(violated_exclusions(&stipulations, &verdicts), ["S2"]);
    }

    #[test]
    fn research_category_scales_the_material_allowance() {
        let research = schedule_for_category("local-research");
        let default = default_allowance_schedule();
        assert_eq!(research.limit("P1"), 10);
        assert_eq!(default.limit("P1"), 6);
        assert_eq!(research.limit("P4"), default.limit("P4"));
    }

    #[test]
    fn carrier_decisions_accept_documented_and_flat_shapes() {
        let flat = normalise_carrier(&json!({
            "carrier": "capability", "capability": "read_file", "args": {"path": "fact.txt"}
        }))
        .unwrap();
        assert_eq!(flat["kind"], "capability");
        assert_eq!(flat["name"], "read_file");
        let documented = normalise_carrier(&json!({
            "carrier": {"kind": "internal_control", "name": "close", "args": {"reason": "done"}}
        }))
        .unwrap();
        assert!(is_closure_control_carrier(&documented));
        let model = normalise_carrier(&json!({"intent": "answer"})).unwrap();
        assert_eq!(model["kind"], "model");
        assert!(normalise_carrier(&json!({"carrier": "teleport"})).is_err());
    }

    #[test]
    fn allowance_grants_one_recorded_grace_then_binds() {
        let mut policy =
            ModelPolicy::with_schedule(Mode::Direct, AllowanceSchedule::new([("P1", 2)]));
        let (first, refusal) = policy.allowance("P1");
        assert!(!first && refusal.is_null());
        let _ = policy.allowance("P1");
        let (third, refusal) = policy.allowance("P1");
        assert!(third, "third act at the wall is refused");
        assert_eq!(refusal["grace_extension"], 2);
        let (fourth, _) = policy.allowance("P1");
        assert!(!fourth, "grace act runs");
        let _ = policy.allowance("P1");
        let (sixth, refusal) = policy.allowance("P1");
        assert!(sixth);
        assert_eq!(refusal["grace_extension"], 0);
    }
}
