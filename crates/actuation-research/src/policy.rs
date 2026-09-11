//! Model-driven research policy. Model/body selection remains with the supplier;
//! every control turn uses Actuation's existing RuntimeHost. Formal validity is
//! checked by the Engine's bound QL owner, not inferred from the controller.
use crate::{evidence::candidate_boundary, execution::block_on, relational::*, Error, Result};
use actuation_runtime::{HostCall, RuntimeHost};
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

const GUIDE:&str="Positions are responsibilities, not chronological stages: P0 initiating intent and operative frame; P1 material, evidence and givens; P2 effect, operation and transformation; P3 form, pattern and implementation; P4 whole-relative evaluation, context and adequacy; P5 candidate determination and synthesis. The carrier does not determine the semantic destination. Do not force a six-step path.";
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
        json!({"purpose":purpose,"system":format!("{instruction}\nReturn exactly one JSON object, without prose outside it."),"prompt":payload.to_string(),"circuit_ref":cx.circuit.id}),
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
pub struct ModelPolicy {
    pub mode: Mode,
    depth_used: BTreeSet<String>,
    last_context: Option<(
        actuation_runtime::LoopRequest,
        actuation_runtime::CancellationToken,
    )>,
}
impl ModelPolicy {
    pub fn new(mode: Mode) -> Self {
        Self {
            mode,
            depth_used: BTreeSet::new(),
            last_context: None,
        }
    }
}
impl Policy for ModelPolicy {
    fn next_act(
        &mut self,
        cx: &PolicyContext<'_>,
        host: &mut dyn RuntimeHost,
    ) -> Result<Option<Act>> {
        self.last_context = Some((cx.request.clone(), cx.cancellation.clone()));
        let d=control(cx,host,"ql-next-act",&format!("{GUIDE} Choose an exterior act: carrier model, capability, internal_control, environment or human when supplied. In Deep mode at P4, request deep_operator=depth only when a bounded local whole genuinely warrants independent execution. Do not use depth ceremonially."),
            json!({"mode":self.mode,"circuit":cx.circuit.compact(),"task":cx.request.wire()["input"],"capabilities":cx.circuit.frame["available_capabilities"]}))?;
        let (carrier, nested) = if d["deep_operator"] == "depth" {
            if self.mode != Mode::Deep
                || cx.circuit.active_position != 4
                || !self.depth_used.insert(cx.circuit.id.clone())
            {
                return Err(Error::new(
                    "depth not available at this circuit aperture or already used",
                ));
            }
            let aperture=control(cx,host,"ql-depth-aperture","State one bounded local whole whose independent resolution would materially improve the parent evaluation. Return local_whole_intent, selected_residue_refs, success_conditions, capabilities; do not request access absent from the parent.",json!({"circuit":cx.circuit.compact()}))?;
            (
                json!({"kind":"child_circuit"}),
                Some(NestedRequest {
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
            )
        } else {
            (
                d.get("carrier")
                    .cloned()
                    .ok_or_else(|| Error::new("controller must choose a supplied carrier"))?,
                None,
            )
        };
        Ok(Some(Act {
            source_position: Some(cx.circuit.active_position),
            intent: d["intent"].clone(),
            carrier,
            input_residue_refs: strings(&d["input_residue_refs"])?,
            nested,
            metadata: json!({"model_control":true,"claimed_position":d["claimed_position"]}),
        }))
    }
    fn interpret(
        &mut self,
        cx: &PolicyContext<'_>,
        act: &Act,
        difference: &Value,
        host: &mut dyn RuntimeHost,
    ) -> Result<Interpretation> {
        let d=control(cx,host,"ql-interpret-return",&format!("{GUIDE} Interpret the returned difference. Choose one explicit destination P0..P5 and explain why. Preserve operation failure and ambiguity. Return destination, rationale, semantic_summary and optional ambiguity."),
            json!({"circuit":cx.circuit.compact(),"act":act,"difference":difference}))?;
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
        let d=control(cx,host,"ql-propose-determination","Synthesize what is actually realised relative to the initiating intent. Return synthesis, requested_outcome close|reopen|conjugate, evidence_refs, unresolved_refs, and any genuinely known claimed_subject and claimed_state. Conjugation is optional in Deep mode; do not substitute process stopping for closure.",json!({"mode":self.mode,"circuit":cx.circuit.compact()}))?;
        let requested = match text(&d, "requested_outcome")?.as_str() {
            "close" => Outcome::Close,
            "reopen" => Outcome::Reopen,
            "conjugate" if self.mode == Mode::Deep => Outcome::Conjugate,
            _ => return Err(Error::new("unsupported or missing determination outcome")),
        };
        let nested = if matches!(requested, Outcome::Conjugate) {
            let s=control(cx,host,"ql-conjugate-scope","Select a fresh inspection packet: scope whole|current_position, selected_residue_refs, optional pairing_modulation with owner fields family, pair_index, degree and projection_side for D2. Do not invoke a modulation merely because one exists. The new context will not inherit the persuasive direct transcript.",json!({"circuit":cx.circuit.compact(),"determination":d}))?;
            Some(NestedRequest {
                intent: json!({"initiating_intent":retained_context(&cx.circuit.frame["initiating_intent"]),"candidate_outcome":d["synthesis"],"office":"fresh inverse/critical assessment"}),
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
        Ok(Some(Determination {
            synthesis: d["synthesis"].clone(),
            requested_outcome: requested,
            claimed_subject: d["claimed_subject"].as_str().map(str::to_owned),
            claimed_state: d["claimed_state"].as_str().map(str::to_owned),
            evidence_refs: strings(&d["evidence_refs"])?,
            evaluation_refs: cx
                .circuit
                .residues
                .iter()
                .filter(|r| r.kind == "evaluation" && !r.invalidated)
                .map(|r| r.id.clone())
                .collect(),
            unresolved_refs: strings(&d["unresolved_refs"])?,
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
        let v=control(cx,host,"ql-evaluate-closure","Compare initiating Frame/P0, current whole-relative Evaluation/P4 and Determination/P5. Current inspection is evidence; a supplied/model claim is not inspection. Return status close|reopen. Reopen requires a distinct destination P0..P4. Close requires explicit task_success true|false|unknown and rationale; missing objective evidence remains unknown.",json!({"frame":retained_context(&cx.circuit.frame),"determination":d,"inspection":inspection,"evaluations":cx.circuit.residues.iter().filter(|r|r.kind=="evaluation"&&!r.invalidated).collect::<Vec<_>>()}))?;
        match v["status"].as_str() {
            Some("close") => Ok(Verdict::Close {
                task_success: text(&v, "task_success")?,
                rationale: v["rationale"].clone(),
            }),
            Some("reopen") => Ok(Verdict::Reopen {
                destination: position(&v["destination"])?,
                rationale: v["rationale"].clone(),
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
        let mut d=control(&cx,host,"ql-conjugate-reintegration","Read only the attributable typed result of the completed fresh review. Return status confirm|qualify|reopen|invalidate, rationale and target_position P0..P4 for reopening. A child's termination alone is not a positive review.",json!({"direct_circuit_ref":parent.id,"child_summary":summary}))?;
        if matches!(d["status"].as_str(), Some("reopen" | "invalidate")) {
            d["target_position"] = json!(position(&d["target_position"])?);
        }
        Ok(d)
    }
}
