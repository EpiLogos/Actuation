//! Direct/Deep research recurrence over Actuation's shared carrier and observer.
//! Positions and relation classifications come from the pinned QL owner. The
//! residue responsibilities and closure protocol below are the authored research
//! apparatus, not a second implementation of QL's formal algebra.
use crate::{
    evidence::{candidate_boundary, sanitize, stable_digest},
    execution::block_on,
    owner::OwnerInstrument,
    Error, Result,
};
use actuation_core::ExternalRef;
use actuation_runtime::{
    dispatch_host_carrier, CancellationToken, Carrier, HostError, LoopEvent, LoopRequest,
    RuntimeHost, RuntimeObserver,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub trait FormalOwner {
    fn invoke(&self, request: Value) -> Result<Value>;
    fn basis(&self) -> Value;
}
impl FormalOwner for OwnerInstrument {
    fn invoke(&self, request: Value) -> Result<Value> {
        OwnerInstrument::invoke(self, request)
    }
    fn basis(&self) -> Value {
        OwnerInstrument::basis(self)
    }
}
/// Research responsibilities are explicit source data. Formal validity is
/// admitted by the owner's vocabulary, never by an independently coded modulus.
#[derive(Clone, Debug)]
pub struct Profile {
    pub positions: BTreeSet<u8>,
    pub roles: BTreeMap<u8, String>,
    pub faces: BTreeSet<String>,
    pub basis: Value,
}
impl Profile {
    pub fn bind(owner: &dyn FormalOwner) -> Result<Self> {
        let v = owner.invoke(json!({"operation":"vocabulary"}))?;
        let positions = v["result"]["positions"]
            .as_array()
            .ok_or_else(|| Error::new("QL owner vocabulary has no positions"))?
            .iter()
            .map(|p| {
                p["position"]
                    .as_u64()
                    .and_then(|n| u8::try_from(n).ok())
                    .ok_or_else(|| Error::new("invalid owner position"))
            })
            .collect::<Result<BTreeSet<_>>>()?;
        let faces = v["result"]["faces"]
            .as_array()
            .ok_or_else(|| Error::new("QL owner vocabulary has no faces"))?
            .iter()
            .map(|f| {
                f.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| Error::new("invalid owner face"))
            })
            .collect::<Result<BTreeSet<_>>>()?;
        let roles: BTreeMap<u8, String> = serde_json::from_str(include_str!(
            "../../../experiments/native-research/relational-roles.json"
        ))
        .map_err(|e| Error::new(e.to_string()))?;
        if positions != roles.keys().copied().collect()
            || !faces.contains("direct")
            || !faces.contains("conjugate")
        {
            return Err(Error::new(
                "research responsibilities do not match the bound owner vocabulary",
            ));
        }
        Ok(Self {
            positions,
            roles,
            faces,
            basis: owner.basis(),
        })
    }
    fn position(&self, p: u8) -> Result<()> {
        if self.positions.contains(&p) {
            Ok(())
        } else {
            Err(Error::new("position is not admitted by the bound QL owner"))
        }
    }
    pub fn role(&self, p: u8) -> Result<&str> {
        self.roles
            .get(&p)
            .map(String::as_str)
            .ok_or_else(|| Error::new("unknown research responsibility"))
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    Direct,
    Deep,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub max_steps: u64,
    pub max_depth: u32,
    pub max_contexts: u64,
    pub max_trace_bytes: usize,
    pub max_reentries: u64,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_steps: 64,
            max_depth: 2,
            max_contexts: 8,
            max_trace_bytes: 8 * 1024 * 1024,
            max_reentries: 0,
        }
    }
}
impl Limits {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.max_steps == 0
            || self.max_steps > 10000
            || self.max_depth > 16
            || self.max_contexts == 0
            || self.max_contexts > 128
            || self.max_trace_bytes < 1024
            || self.max_trace_bytes > 64 * 1024 * 1024
            || self.max_reentries > 32
        {
            return Err(Error::new("invalid shared research limits"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Residue {
    pub id: String,
    pub position: u8,
    pub kind: String,
    pub value: Value,
    pub provenance: Value,
    #[serde(default)]
    pub invalidated: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Circuit {
    pub id: String,
    pub parent_id: Option<String>,
    pub depth: u32,
    pub face: String,
    pub frame: Value,
    pub active_position: u8,
    pub closure_state: String,
    pub residues: Vec<Residue>,
    pub trajectory: Vec<Value>,
    pub children: Vec<String>,
    pub conjugates: Vec<String>,
    pub success_state: Value,
}
impl Circuit {
    fn new(
        id: String,
        parent_id: Option<String>,
        depth: u32,
        face: &str,
        frame: Value,
    ) -> Result<Self> {
        candidate_boundary(&frame)?;
        if !frame.is_object()
            || frame["id"].as_str().is_none_or(|s| s.trim().is_empty())
            || frame.get("initiating_intent").is_none_or(Value::is_null)
        {
            return Err(Error::new(
                "independent circuit requires an identified frame and initiating intent",
            ));
        }
        Ok(Self {
            id,
            parent_id,
            depth,
            face: face.into(),
            frame,
            active_position: 0,
            closure_state: "open".into(),
            residues: vec![],
            trajectory: vec![],
            children: vec![],
            conjugates: vec![],
            success_state: json!({"operation":"unknown","task":"unknown","circuit":"unknown","harmonic":"unknown"}),
        })
    }
    pub fn compact(&self) -> Value {
        json!({"id":self.id,"parent_id":self.parent_id,"depth":self.depth,"face":self.face,
            "frame":retained_context(&self.frame),"active_position":self.active_position,
            "residues":self.residues.iter().filter(|r|!r.invalidated).map(|r|json!({"id":r.id,"kind":r.kind,"position":r.position,"value":retained_context(&r.value)})).collect::<Vec<_>>(),
            "trajectory":self.trajectory.iter().map(|t|json!({"from":t["from"],"to":t["to"],"relation":t["relation"]})).collect::<Vec<_>>()})
    }
    fn selected(&self, refs: &[String]) -> Result<Vec<Value>> {
        let mut seen = BTreeSet::new();
        refs.iter().map(|id| {
            if !seen.insert(id) { return Err(Error::new("duplicate selected residue reference")); }
            let r=self.residues.iter().find(|r| &r.id==id && !r.invalidated)
                .ok_or_else(||Error::new("selected residue is absent, foreign, or invalidated"))?;
            Ok(json!({"id":r.id,"kind":r.kind,"position":r.position,"value":retained_context(&r.value),"provenance":r.provenance}))
        }).collect()
    }
}
/// Remove known session/transcript containers, preserving other authored frame
/// extensions. Selected retained information is not the parent's full transcript.
pub fn retained_context(v: &Value) -> Value {
    match v {
        Value::Object(m) => Value::Object(
            m.iter()
                .filter(|(k, _)| {
                    !matches!(
                        k.as_str(),
                        "transcript"
                            | "messages"
                            | "chat_history"
                            | "model_history"
                            | "session"
                            | "session_id"
                            | "raw_events"
                            | "raw_transcript"
                    )
                })
                .map(|(k, v)| (k.clone(), retained_context(v)))
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().map(retained_context).collect()),
        _ => v.clone(),
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NestedRequest {
    pub intent: Value,
    #[serde(default)]
    pub selected_residue_refs: Vec<String>,
    #[serde(default)]
    pub success_conditions: Vec<Value>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default = "whole")]
    pub scope: Value,
    #[serde(default)]
    pub extension: Option<String>,
    #[serde(default)]
    pub modulation: Option<Value>,
}
fn whole() -> Value {
    json!("whole")
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Act {
    #[serde(default)]
    pub source_position: Option<u8>,
    #[serde(default)]
    pub intent: Value,
    pub carrier: Value,
    #[serde(default)]
    pub input_residue_refs: Vec<String>,
    #[serde(default)]
    pub nested: Option<NestedRequest>,
    #[serde(default)]
    pub metadata: Value,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interpretation {
    pub destination: u8,
    #[serde(default)]
    pub rationale: Value,
    #[serde(default)]
    pub witness: Value,
    #[serde(default)]
    pub create: Vec<Value>,
    #[serde(default)]
    pub revise: Vec<Value>,
    #[serde(default)]
    pub invalidate: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Close,
    Reopen,
    Conjugate,
    Depth,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Determination {
    pub synthesis: Value,
    pub requested_outcome: Outcome,
    #[serde(default)]
    pub claimed_subject: Option<String>,
    #[serde(default)]
    pub claimed_state: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub evaluation_refs: Vec<String>,
    #[serde(default)]
    pub unresolved_refs: Vec<String>,
    #[serde(default)]
    pub nested: Option<NestedRequest>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum Verdict {
    Close {
        task_success: String,
        #[serde(default)]
        rationale: Value,
    },
    Reopen {
        destination: u8,
        #[serde(default)]
        rationale: Value,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inspection {
    pub subject_ref: String,
    pub state_digest: String,
    pub objective_checks_pass: bool,
    pub evidence: Value,
}
pub trait Inspector {
    /// None means no objective task evidence, never a successful check.
    fn inspect(
        &mut self,
        circuit: &Circuit,
        determination: &Determination,
    ) -> Result<Option<Inspection>>;
}
pub struct NoInspection;
impl Inspector for NoInspection {
    fn inspect(&mut self, _: &Circuit, _: &Determination) -> Result<Option<Inspection>> {
        Ok(None)
    }
}
pub struct PolicyContext<'a> {
    pub circuit: &'a Circuit,
    pub request: &'a LoopRequest,
    pub cancellation: &'a CancellationToken,
}
pub trait Policy {
    fn next_act(
        &mut self,
        cx: &PolicyContext<'_>,
        host: &mut dyn RuntimeHost,
    ) -> Result<Option<Act>>;
    fn establish_difference(
        &mut self,
        _cx: &PolicyContext<'_>,
        returned: &Value,
        _host: &mut dyn RuntimeHost,
    ) -> Result<Value> {
        Ok(returned.clone())
    }
    fn interpret(
        &mut self,
        cx: &PolicyContext<'_>,
        act: &Act,
        difference: &Value,
        host: &mut dyn RuntimeHost,
    ) -> Result<Interpretation>;
    fn determine(
        &mut self,
        cx: &PolicyContext<'_>,
        host: &mut dyn RuntimeHost,
    ) -> Result<Option<Determination>>;
    fn evaluate_closure(
        &mut self,
        cx: &PolicyContext<'_>,
        d: &Determination,
        inspection: Option<&Inspection>,
        host: &mut dyn RuntimeHost,
    ) -> Result<Verdict>;
    fn conjugate_delta(
        &mut self,
        _parent: &Circuit,
        _summary: &Value,
        _host: &mut dyn RuntimeHost,
    ) -> Result<Value> {
        Err(Error::new("conjugate reintegration policy is not supplied"))
    }
    fn reentry_delta(&mut self, _circuit: &Circuit, d: &Determination) -> Result<Value> {
        Ok(json!({"unresolved_refs":d.unresolved_refs}))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunResult {
    pub schema: String,
    pub status: String,
    pub runtime: Mode,
    pub trace_ref: ExternalRef,
    pub outcome: Value,
    pub circuits: Vec<Circuit>,
    pub closure: Option<Value>,
    pub reentry: Option<Value>,
    pub evidence_refs: Vec<ExternalRef>,
    pub owner_basis: Value,
    pub steps: u64,
    pub error: Option<String>,
    pub provider_evidence: String,
    pub owner_machine_evidence: bool,
    pub human_acceptance: bool,
}
#[derive(Clone)]
struct Completion {
    status: String,
    outcome: Value,
    closure: Option<Value>,
    reentry: Option<Value>,
}
impl Completion {
    fn stopped(status: &str) -> Self {
        Self {
            status: status.into(),
            outcome: Value::Null,
            closure: None,
            reentry: None,
        }
    }
}
pub struct Engine<'a> {
    formal: &'a dyn FormalOwner,
    profile: Profile,
    host: &'a mut dyn RuntimeHost,
    policy: &'a mut dyn Policy,
    inspector: &'a mut dyn Inspector,
    observer: &'a mut dyn RuntimeObserver,
    mode: Mode,
    trace: ExternalRef,
    cancellation: CancellationToken,
    limits: Limits,
    capabilities: BTreeSet<String>,
    extensions: BTreeSet<String>,
    secrets: Vec<String>,
    steps: u64,
    contexts: u64,
    sequence: u64,
    trace_bytes: usize,
    sink_failed: bool,
    refs: Vec<ExternalRef>,
    circuits: Vec<Circuit>,
    started: bool,
}
impl<'a> Engine<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        formal: &'a dyn FormalOwner,
        host: &'a mut dyn RuntimeHost,
        policy: &'a mut dyn Policy,
        inspector: &'a mut dyn Inspector,
        observer: &'a mut dyn RuntimeObserver,
        mode: Mode,
        trace: ExternalRef,
        cancellation: CancellationToken,
        limits: Limits,
        capabilities: Vec<String>,
        extensions: Vec<String>,
        secrets: Vec<String>,
    ) -> Result<Self> {
        limits.validate()?;
        let profile = Profile::bind(formal)?;
        Ok(Self {
            formal,
            profile,
            host,
            policy,
            inspector,
            observer,
            mode,
            trace,
            cancellation,
            limits,
            capabilities: capabilities.into_iter().collect(),
            extensions: extensions.into_iter().collect(),
            secrets,
            steps: 0,
            contexts: 0,
            sequence: 0,
            trace_bytes: 0,
            sink_failed: false,
            refs: vec![],
            circuits: vec![],
            started: false,
        })
    }
    fn emit(&mut self, c: &Circuit, kind: &str, payload: Value) -> Result<()> {
        let payload = sanitize(
            &json!({"circuit_id":c.id,"parent_circuit_id":c.parent_id,"face":c.face,
            "active_position":c.active_position,"research":payload}),
            &self.secrets,
        );
        let event = LoopEvent {
            channel: "runtime-semantic".into(),
            event_id: format!("{}:relational:{}", self.trace, self.sequence),
            event_type: kind.into(),
            run_id: self.trace.clone(),
            sequence: self.sequence,
            runtime: match self.mode {
                Mode::Direct => "ql-direct",
                Mode::Deep => "ql-deep",
            }
            .into(),
            payload,
        };
        let bytes = serde_json::to_vec(&event)
            .map_err(|e| Error::new(e.to_string()))?
            .len();
        if bytes > self.limits.max_trace_bytes.saturating_sub(self.trace_bytes) {
            self.sink_failed = true;
            return Err(Error::new(
                "research chronology exceeds its retained byte budget",
            ));
        }
        match self.observer.emit(&event) {
            Ok(r) => {
                self.refs.push(r);
                self.sequence += 1;
                self.trace_bytes += bytes;
                Ok(())
            }
            Err(e) => {
                self.sink_failed = true;
                Err(e)
            }
        }
    }
    fn request(&self, c: &Circuit) -> Result<LoopRequest> {
        LoopRequest::from_legacy(
            json!({"id":c.frame["id"],"taskId":c.frame["id"],"runId":self.trace,
            "input":retained_context(&c.frame["initiating_intent"]),"successConditions":c.frame["success_conditions"],
            "operativeScope":c.frame["operative_scope"],"capabilities":c.frame["available_capabilities"],
            "maxSteps":self.limits.max_steps}),
        )
    }
    fn add_residue(
        &self,
        c: &mut Circuit,
        p: u8,
        kind: Option<&str>,
        value: Value,
        provenance: Value,
    ) -> Result<Residue> {
        self.profile.position(p)?;
        let kind = kind.unwrap_or(self.profile.role(p)?);
        if !self.profile.roles.values().any(|r| r == kind) {
            return Err(Error::new("unrecognised research residue kind"));
        }
        let r = Residue {
            id: format!("{}:res:{}", c.id, c.residues.len()),
            position: p,
            kind: kind.into(),
            value,
            provenance,
            invalidated: false,
        };
        c.residues.push(r.clone());
        Ok(r)
    }
    fn relation(&self, from: u8, to: u8) -> Result<Value> {
        self.profile.position(from)?;
        self.profile.position(to)?;
        let reading = self
            .formal
            .invoke(json!({"operation":"classify-relation","from":from,"to":to}))?;
        if !reading["result"].is_array() {
            return Err(Error::new("QL owner relation result is not an array"));
        }
        // Rij is a trace address. Family membership is exclusively the owner's
        // result, including empty or multiple memberships; ambiguity survives.
        Ok(json!({"id":format!("R{from}{to}"),"owner_reading":reading}))
    }
    fn transition(&mut self, c: &mut Circuit, i: &Interpretation, return_id: &str) -> Result<()> {
        let relation = self.relation(c.active_position, i.destination)?;
        let mut updated = c.clone();
        let mut creates = vec![];
        for item in &i.create {
            let p = match item.get("position") {
                None => i.destination,
                Some(v) => v
                    .as_u64()
                    .and_then(|n| u8::try_from(n).ok())
                    .ok_or_else(|| Error::new("residue position requires an owner position"))?,
            };
            let r = self.add_residue(
                &mut updated,
                p,
                item["kind"].as_str(),
                item["value"].clone(),
                item.get("provenance")
                    .cloned()
                    .unwrap_or(json!({"return_ref":return_id})),
            )?;
            creates.push(r);
        }
        for item in &i.revise {
            let r = updated
                .residues
                .iter_mut()
                .find(|r| Some(r.id.as_str()) == item["id"].as_str() && !r.invalidated)
                .ok_or_else(|| Error::new("cannot revise absent or invalidated residue"))?;
            r.value = item["value"].clone();
            if let Some(p) = item.get("provenance") {
                r.provenance = p.clone();
            }
        }
        for id in &i.invalidate {
            let r = updated
                .residues
                .iter_mut()
                .find(|r| &r.id == id && !r.invalidated)
                .ok_or_else(|| {
                    Error::new("cannot invalidate absent or already invalidated residue")
                })?;
            r.invalidated = true;
        }
        let transition = json!({"id":format!("{}:transition:{}",c.id,c.trajectory.len()),"from":c.active_position,
            "to":i.destination,"relation":relation["id"],"formal_owner_reading":relation["owner_reading"],
            "interpretation_ref":return_id,"rationale":i.rationale,"witness_state":i.witness,
            "created_residue_refs":creates.iter().map(|r|&r.id).collect::<Vec<_>>(),
            "revised_residue_refs":i.revise.iter().map(|v|v["id"].clone()).collect::<Vec<_>>(),"invalidated_residue_refs":i.invalidate});
        // Validate the full delta before emitting or mutating the circuit.
        for r in &creates {
            self.emit(c, "residue_created", json!({"residue":r}))?;
        }
        for item in &i.revise {
            self.emit(c, "residue_revised", item.clone())?;
        }
        for id in &i.invalidate {
            self.emit(c, "residue_invalidated", json!({"residue_id":id}))?;
        }
        self.emit(c, "transition", json!({"transition":transition}))?;
        updated.trajectory.push(transition);
        updated.active_position = i.destination;
        *c = updated;
        Ok(())
    }
    fn reopen(
        &mut self,
        c: &mut Circuit,
        destination: u8,
        why: Value,
        reference: &str,
    ) -> Result<()> {
        if c.closure_state != "open" || c.active_position != 5 || destination == 5 {
            return Err(Error::new(
                "reopening requires an open P5 circuit and a distinct P0-P4 destination",
            ));
        }
        self.profile.position(destination)?;
        self.emit(
            c,
            "circuit_reopened",
            json!({"destination":destination,"reason":why,"determination_ref":reference}),
        )?;
        self.transition(
            c,
            &Interpretation {
                destination,
                rationale: why,
                witness: json!({"closure_status":"reopen"}),
                create: vec![],
                revise: vec![],
                invalidate: vec![],
            },
            reference,
        )
    }
    fn nested(
        &mut self,
        parent: &mut Circuit,
        n: &NestedRequest,
        conjugate: bool,
    ) -> Result<Value> {
        if self.mode != Mode::Deep {
            return Err(Error::new(
                "Direct research does not provide recursive or conjugate execution",
            ));
        }
        if parent.closure_state != "open" {
            return Err(Error::new(
                "a closed circuit cannot spawn or be retroactively reopened",
            ));
        }
        if conjugate && parent.active_position != 5 {
            return Err(Error::new("conjugate review requires a candidate at P5"));
        }
        if !conjugate
            && parent.active_position != 4
            && !n
                .extension
                .as_ref()
                .is_some_and(|e| e.starts_with("ql.") && self.extensions.contains(e))
        {
            return Err(Error::new(
                "depth requires the P4 aperture or an explicitly admitted extension",
            ));
        }
        let depth = parent.depth + u32::from(!conjugate);
        if depth > self.limits.max_depth || self.contexts >= self.limits.max_contexts {
            return Err(Error::new("nested research context/depth budget exhausted"));
        }
        let selected = parent.selected(&n.selected_residue_refs)?;
        let parent_caps = parent.frame["available_capabilities"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        if n.capabilities
            .iter()
            .any(|k| !self.capabilities.contains(k) || !parent_caps.contains(&json!(k)))
        {
            return Err(Error::new(
                "a nested context cannot increase parent capability authority",
            ));
        }
        if n.intent.is_null() {
            return Err(Error::new("nested whole requires its own intent"));
        }
        let scope = if conjugate {
            match &n.scope {
                Value::String(s) if s == "whole" || s == "current_position" => n.scope.clone(),
                Value::Object(m) if m.len() == 1 && m.contains_key("position") => {
                    let p = m["position"]
                        .as_u64()
                        .and_then(|n| u8::try_from(n).ok())
                        .ok_or_else(|| Error::new("invalid conjugate position scope"))?;
                    self.profile.position(p)?;
                    n.scope.clone()
                }
                Value::Object(m) if m.len() == 1 && m["relation"].is_array() => {
                    let ps = m["relation"].as_array().unwrap();
                    if ps.len() != 2 {
                        return Err(Error::new("relation scope requires two positions"));
                    }
                    let parse = |v: &Value| {
                        v.as_u64()
                            .and_then(|n| u8::try_from(n).ok())
                            .ok_or_else(|| Error::new("invalid relation scope"))
                    };
                    self.relation(parse(&ps[0])?, parse(&ps[1])?)?;
                    n.scope.clone()
                }
                _ => return Err(Error::new("unsupported conjugate scope")),
            }
        } else {
            json!({"parent_position":parent.active_position})
        };
        let modulation = if let Some(m) = &n.modulation {
            if !conjugate {
                return Err(Error::new(
                    "pairing modulation belongs to explicit conjugate inspection",
                ));
            }
            let mut m = m
                .as_object()
                .cloned()
                .ok_or_else(|| Error::new("modulation request requires object"))?;
            m.insert("operation".into(), json!("modulation"));
            self.formal.invoke(Value::Object(m))?
        } else {
            Value::Null
        };
        let id = format!(
            "{}:{}:{}",
            parent.id,
            if conjugate { "conjugate" } else { "child" },
            self.contexts
        );
        let packet = json!({"parent_circuit_ref":parent.id,"source_position":parent.active_position,"scope":scope,
            "initiating_intent":retained_context(&n.intent),"selected_residues":selected,"pairing_modulation":modulation,
            "provenance":{"reconstructed_context":true,"complete_direct_transcript_inherited":false}});
        let frame = json!({"id":format!("{id}:frame"),"initiating_intent":retained_context(&n.intent),
            "operative_scope":scope,"constraints":[],"available_capabilities":n.capabilities,
            "success_conditions":n.success_conditions,"inherited_delta":null,"selected_context":packet,
            "provenance":{"parent_circuit_ref":parent.id,"parent_position":parent.active_position,"independent_frame":true}});
        let mut child = Circuit::new(
            id.clone(),
            Some(parent.id.clone()),
            depth,
            if conjugate { "conjugate" } else { "direct" },
            frame,
        )?;
        self.contexts += 1;
        if conjugate {
            parent.conjugates.push(id.clone());
        } else {
            parent.children.push(id.clone());
        }
        self.emit(
            &child,
            if conjugate {
                "conjugate_started"
            } else {
                "child_started"
            },
            json!({"frame":child.frame,"packet":packet}),
        )?;
        let completed = match self.run_circuit(&mut child) {
            Ok(r) => r,
            Err(e) if !self.sink_failed => {
                self.emit(&child, "circuit_failed", json!({"error":e.to_string()}))?;
                Completion::stopped("failed")
            }
            Err(e) => return Err(e),
        };
        let summary = json!({"child_circuit":child.id,"parent_circuit":parent.id,"child_intent":n.intent,
            "status":completed.status,"closure_ref":completed.closure.as_ref().map(|v|&v["id"]),
            "returned_delta":{"synthesis":retained_context(&completed.outcome),"success_state":child.success_state},
            "relevant_residue_refs":child.residues.iter().filter(|r|!r.invalidated).map(|r|&r.id).collect::<Vec<_>>(),
            "typed_summary_only":true,"transcript_required":false});
        self.emit(
            &child,
            if conjugate {
                "conjugate_completed"
            } else {
                "child_completed"
            },
            json!({"summary":summary}),
        )?;
        self.circuits.push(child);
        self.emit(
            parent,
            if conjugate {
                "conjugate_returned"
            } else {
                "child_reintegrated"
            },
            json!({"summary":summary}),
        )?;
        Ok(summary)
    }
    fn close(
        &mut self,
        c: &mut Circuit,
        d: &Determination,
        did: &str,
        verdict: &Verdict,
        inspection: Option<&Inspection>,
    ) -> Result<Completion> {
        let Verdict::Close { task_success, .. } = verdict else {
            return Err(Error::new("positive closure requires a close verdict"));
        };
        if c.active_position != 5 || c.closure_state != "open" {
            return Err(Error::new("positive closure requires an open P5 circuit"));
        }
        if !matches!(task_success.as_str(), "true" | "false" | "unknown") {
            return Err(Error::new("invalid task success standing"));
        }
        let success = if inspection.is_some_and(|i| i.objective_checks_pass) {
            task_success.as_str()
        } else if task_success == "true" {
            "unknown"
        } else {
            task_success.as_str()
        };
        let closure = json!({"id":format!("{}:closure",c.id),"circuit_id":c.id,"determination_ref":did,
            "frame_ref":c.frame["id"],"evaluation_refs":d.evaluation_refs,"evidence_refs":d.evidence_refs,
            "success_state":{"operation":c.success_state["operation"],"task":success,"circuit":"true","harmonic":"unknown"},
            "closed_at_position":5,"inspection":inspection,"human_acceptance":false});
        self.emit(c, "circuit_closed", json!({"closure":closure}))?;
        c.closure_state = "closed".into();
        c.success_state = closure["success_state"].clone();
        // No delta or new context is constructed before the close was persisted.
        let proposed = self.policy.reentry_delta(c, d)?;
        let mut delta = proposed
            .as_object()
            .cloned()
            .ok_or_else(|| Error::new("reentry delta requires object"))?;
        delta.insert("id".into(), json!(format!("{}:reentry-delta", c.id)));
        delta.insert("source_circuit".into(), json!(c.id));
        delta
            .entry("revised_success_conditions")
            .or_insert(c.frame["success_conditions"].clone());
        delta
            .entry("unresolved_refs")
            .or_insert(json!(d.unresolved_refs));
        let mut frame = retained_context(&c.frame);
        frame["id"] = json!(format!("{}+", c.frame["id"].as_str().unwrap()));
        frame["inherited_delta"] = delta["id"].clone();
        frame["success_conditions"] = delta["revised_success_conditions"].clone();
        let reentry = json!({"prior_circuit":c.id,"closure_ref":closure["id"],"delta_ref":delta["id"],
            "delta":delta,"renewed_frame":frame});
        self.emit(c, "reentry_created", reentry.clone())?;
        Ok(Completion {
            status: "completed".into(),
            outcome: d.synthesis.clone(),
            closure: Some(closure),
            reentry: Some(reentry),
        })
    }
    fn run_circuit(&mut self, c: &mut Circuit) -> Result<Completion> {
        self.emit(
            c,
            "circuit_started",
            json!({"frame":c.frame,"depth":c.depth}),
        )?;
        let frame = self.add_residue(
            c,
            0,
            Some("frame"),
            c.frame.clone(),
            c.frame["provenance"].clone(),
        )?;
        self.emit(
            c,
            "frame_established",
            json!({"frame":c.frame,"residue_ref":frame.id}),
        )?;
        while self.steps < self.limits.max_steps {
            if self.cancellation.requested() {
                return Ok(Completion::stopped("cancelled"));
            }
            let request = self.request(c)?;
            // Retain chronology before consequential policy/host calls.
            self.emit(c, "policy_requested", json!({"operation":"next_act"}))?;
            let cx = PolicyContext {
                circuit: c,
                request: &request,
                cancellation: &self.cancellation,
            };
            let Some(act) = self.policy.next_act(&cx, self.host)? else {
                return Ok(Completion::stopped("exhausted"));
            };
            if act.source_position.is_some_and(|p| p != c.active_position) {
                return Err(Error::new(
                    "act source does not match active circuit position",
                ));
            }
            c.selected(&act.input_residue_refs)?;
            let aid = format!("{}:act:{}", c.id, self.steps);
            self.emit(c, "act_created", json!({"act_id":aid,"act":act}))?;
            self.emit(c,"projection",json!({"act_id":aid,"phase":"0/1","carrier":act.carrier,"context_refs":act.input_residue_refs}))?;
            self.steps += 1;
            let result = if act.carrier["kind"] == "child_circuit" {
                self.nested(
                    c,
                    act.nested
                        .as_ref()
                        .ok_or_else(|| Error::new("child carrier requires scoped request"))?,
                    false,
                )
            } else {
                let carrier = Carrier::from_legacy(&act.carrier)?;
                if let Carrier::Capability { name, .. } = &carrier {
                    let permitted = c.frame["available_capabilities"]
                        .as_array()
                        .is_some_and(|a| a.contains(&json!(name.to_string())));
                    if !self.capabilities.contains(&name.to_string()) || !permitted {
                        return Err(Error::new(
                            "capability is not authorised in this circuit frame",
                        ));
                    }
                }
                let payload = Map::from_iter([(
                    "ql_act".into(),
                    json!({"id":aid,"circuit":c.compact(),"act":act}),
                )]);
                match block_on(dispatch_host_carrier(
                    self.host,
                    &carrier,
                    &request,
                    &self.cancellation,
                    payload,
                )) {
                    Ok(v) => Ok(v),
                    Err(HostError::Aborted(_)) => return Ok(Completion::stopped("cancelled")),
                    Err(e) => Err(Error::new(e.message())),
                }
            };
            if self.sink_failed {
                return Err(Error::new("nested observer persistence failed"));
            }
            let succeeded = result.as_ref().is_ok_and(|v| {
                !(act.carrier["kind"] == "child_circuit" && v["status"] != "completed")
            });
            let raw = match result {
                Ok(v) => v,
                Err(e) => json!({"error":e.to_string()}),
            };
            c.success_state["operation"] = json!(if succeeded { "true" } else { "false" });
            let rid = format!("{aid}:return");
            let returned = json!({"id":rid,"act_id":aid,"phase":"1/0","raw_result":raw,"operation_success":succeeded});
            self.emit(c, "return_received", json!({"returned":returned}))?;
            let cx = PolicyContext {
                circuit: c,
                request: &request,
                cancellation: &self.cancellation,
            };
            let difference = self
                .policy
                .establish_difference(&cx, &returned, self.host)?;
            self.emit(
                c,
                "difference_established",
                json!({"return_ref":rid,"difference":difference}),
            )?;
            let cx = PolicyContext {
                circuit: c,
                request: &request,
                cancellation: &self.cancellation,
            };
            let i = self.policy.interpret(&cx, &act, &difference, self.host)?;
            self.emit(
                c,
                "return_interpreted",
                json!({"return_ref":rid,"difference":difference,"interpretation":i}),
            )?;
            self.transition(c, &i, &rid)?;
            if c.active_position != 5 {
                continue;
            }
            let cx = PolicyContext {
                circuit: c,
                request: &request,
                cancellation: &self.cancellation,
            };
            let Some(d) = self.policy.determine(&cx, self.host)? else {
                continue;
            };
            let did = format!("{}:determination:{}", c.id, self.steps);
            for id in &d.evaluation_refs {
                if !c
                    .residues
                    .iter()
                    .any(|r| &r.id == id && !r.invalidated && r.kind == "evaluation")
                {
                    return Err(Error::new(
                        "determination cites absent or invalidated evaluation",
                    ));
                }
            }
            let r = self.add_residue(
                c,
                5,
                Some("determination"),
                serde_json::to_value(&d).unwrap(),
                json!({"return_ref":rid}),
            )?;
            self.emit(c, "residue_created", json!({"residue":r}))?;
            self.emit(
                c,
                "determination_proposed",
                json!({"id":did,"determination":d}),
            )?;
            if matches!(d.requested_outcome, Outcome::Depth) {
                return Err(Error::new(
                    "depth must be entered from its explicit aperture, not a synthetic P5 closure",
                ));
            }
            if matches!(d.requested_outcome, Outcome::Conjugate) {
                let summary = self.nested(
                    c,
                    d.nested.as_ref().ok_or_else(|| {
                        Error::new("conjugate determination requires a selected fresh context")
                    })?,
                    true,
                )?;
                if summary["status"] != "completed" {
                    self.reopen(c, 4, json!({"conjugate_incomplete":summary}), &did)?;
                    continue;
                }
                let delta = self.policy.conjugate_delta(c, &summary, self.host)?;
                self.emit(c, "conjugate_delta", delta.clone())?;
                match delta["status"].as_str() {
                    Some("reopen" | "invalidate") => {
                        let p = delta["target_position"]
                            .as_u64()
                            .and_then(|n| u8::try_from(n).ok())
                            .ok_or_else(|| {
                                Error::new("conjugate reopening requires an explicit target")
                            })?;
                        self.reopen(c, p, delta, &did)?;
                        continue;
                    }
                    Some("confirm" | "qualify") => {}
                    _ => return Err(Error::new("unrecognised conjugate delta status")),
                }
            }
            let inspection = self.inspector.inspect(c, &d)?;
            let cx = PolicyContext {
                circuit: c,
                request: &request,
                cancellation: &self.cancellation,
            };
            let mut verdict =
                self.policy
                    .evaluate_closure(&cx, &d, inspection.as_ref(), self.host)?;
            if matches!(verdict, Verdict::Close { .. }) {
                if matches!(d.requested_outcome, Outcome::Reopen) {
                    return Err(Error::new(
                        "a reopening determination cannot silently receive a close verdict",
                    ));
                }
                if let Some(facts) = &inspection {
                    let subject_matches = d
                        .claimed_subject
                        .as_ref()
                        .is_none_or(|s| s == &facts.subject_ref);
                    let state_matches = d
                        .claimed_state
                        .as_ref()
                        .is_none_or(|s| s == &facts.state_digest);
                    if !facts.objective_checks_pass || !subject_matches || !state_matches {
                        self.emit(c,"closure_refused",json!({"inspection":facts,"subject_matches":subject_matches,"state_matches":state_matches}))?;
                        verdict = Verdict::Reopen {
                            destination: 4,
                            rationale: json!(
                                "current evidence does not warrant this determination"
                            ),
                        };
                    }
                } else if d.claimed_subject.is_some() || d.claimed_state.is_some() {
                    verdict = Verdict::Reopen {
                        destination: 4,
                        rationale: json!("claimed subject/state has no current inspection"),
                    };
                }
            }
            self.emit(
                c,
                "closure_evaluated",
                json!({"determination_ref":did,"verdict":verdict,"inspection":inspection}),
            )?;
            match verdict {
                Verdict::Reopen {
                    destination,
                    rationale,
                } => self.reopen(c, destination, rationale, &did)?,
                Verdict::Close { .. } => {
                    return self.close(c, &d, &did, &verdict, inspection.as_ref())
                }
            }
        }
        Ok(Completion::stopped("exhausted"))
    }
    pub fn run(&mut self, frame: Value) -> Result<RunResult> {
        if self.started {
            return Err(Error::new(
                "engine already consumed; resume requires an explicit new invocation",
            ));
        }
        self.started = true;
        let mut c = Circuit::new(format!("{}:c0", self.trace), None, 0, "direct", frame)?;
        self.contexts = 1;
        if c.frame["available_capabilities"]
            .as_array()
            .is_some_and(|a| {
                a.iter()
                    .any(|v| v.as_str().is_none_or(|s| !self.capabilities.contains(s)))
            })
        {
            return Err(Error::new(
                "root frame exceeds explicitly supplied capabilities",
            ));
        }
        self.emit(
            &c,
            "run_started",
            json!({"owner_basis":self.profile.basis,"mode":self.mode,"limits":self.limits}),
        )?;
        let mut reentries = 0;
        let mut error = None;
        let result = loop {
            let completion = match self.run_circuit(&mut c) {
                Ok(v) => v,
                Err(e) if !self.sink_failed => {
                    error = Some(e.to_string());
                    self.emit(&c, "circuit_failed", json!({"error":error}))?;
                    Completion::stopped("failed")
                }
                Err(e) => return Err(e),
            };
            self.circuits.push(c.clone());
            if completion.status != "completed" || reentries >= self.limits.max_reentries {
                break completion;
            }
            if self.contexts >= self.limits.max_contexts {
                break Completion::stopped("exhausted");
            }
            reentries += 1;
            self.contexts += 1;
            let renewed = completion
                .reentry
                .as_ref()
                .ok_or_else(|| Error::new("closed circuit lacks a retained reentry"))?
                ["renewed_frame"]
                .clone();
            c = Circuit::new(
                format!("{}:reentry:{reentries}", self.trace),
                None,
                0,
                "direct",
                renewed,
            )?;
            self.emit(
                &c,
                "reentry_started",
                json!({"prior_closure_ref":completion.closure.as_ref().map(|v|&v["id"])}),
            )?;
        };
        self.emit(
            &c,
            &format!("run_{}", result.status),
            json!({"steps":self.steps,"error":error}),
        )?;
        Ok(RunResult {
            schema: "actuation.relational-research-run/v1".into(),
            status: result.status,
            runtime: self.mode,
            trace_ref: self.trace.clone(),
            outcome: sanitize(&result.outcome, &self.secrets),
            circuits: serde_json::from_value(sanitize(&json!(self.circuits), &self.secrets))
                .map_err(|e| Error::new(e.to_string()))?,
            closure: result.closure.map(|v| sanitize(&v, &self.secrets)),
            reentry: result.reentry.map(|v| sanitize(&v, &self.secrets)),
            evidence_refs: self.refs.clone(),
            owner_basis: self.formal.basis(),
            steps: self.steps,
            error,
            provider_evidence: "not-assessed".into(),
            owner_machine_evidence: false,
            human_acceptance: false,
        })
    }
}
/// Current World state is hashed at the time of the closure check. A model's
/// remembered successful test is not a substitute for this inspection.
pub struct TaskInspector {
    pub task: crate::tasks::Task,
    pub world: crate::world::World,
    pub node: Option<std::path::PathBuf>,
}
impl Inspector for TaskInspector {
    fn inspect(&mut self, c: &Circuit, _d: &Determination) -> Result<Option<Inspection>> {
        if c.parent_id.is_some() {
            return Ok(None);
        }
        let after = self.world.snapshot()?;
        let evidence =
            self.task
                .verify(&self.world, self.task.start(), &after, self.node.as_deref())?;
        Ok(Some(Inspection {
            subject_ref: self.task.id().into(),
            state_digest: stable_digest(&after),
            objective_checks_pass: evidence["objective_checks_pass"] == true,
            evidence,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ScriptedVocabulary;
    impl FormalOwner for ScriptedVocabulary {
        fn invoke(&self, request: Value) -> Result<Value> {
            match request["operation"].as_str() {
                Some("vocabulary") => Ok(json!({"result":{
                    "positions":[{"position":0},{"position":1},{"position":2},
                                 {"position":3},{"position":4},{"position":5}],
                    "faces":["direct","conjugate"]}})),
                other => Err(Error::new(format!("scripted owner lacks {other:?}"))),
            }
        }
        fn basis(&self) -> Value {
            json!({"repository":"scripted","revision":"0".repeat(40)})
        }
    }

    #[test]
    fn limits_bound_the_shared_research_apparatus() {
        assert!(Limits::default().validate().is_ok());
        let rejected = [
            Limits {
                max_steps: 0,
                ..Default::default()
            },
            Limits {
                max_steps: 10_001,
                ..Default::default()
            },
            Limits {
                max_depth: 17,
                ..Default::default()
            },
            Limits {
                max_contexts: 0,
                ..Default::default()
            },
            Limits {
                max_contexts: 129,
                ..Default::default()
            },
            Limits {
                max_trace_bytes: 1023,
                ..Default::default()
            },
            Limits {
                max_trace_bytes: 64 * 1024 * 1024 + 1,
                ..Default::default()
            },
            Limits {
                max_reentries: 33,
                ..Default::default()
            },
        ];
        for limits in rejected {
            assert!(limits.validate().is_err());
        }
    }

    #[test]
    fn retained_context_strips_session_containers_only() {
        let v = json!({
            "transcript": ["drop"],
            "messages": ["drop"],
            "session_id": "drop",
            "authored_extension": {"chat_history":"drop","keep":"kept"},
            "keep": "kept"
        });
        let out = retained_context(&v);
        assert!(out.get("transcript").is_none());
        assert!(out.get("messages").is_none());
        assert!(out.get("session_id").is_none());
        assert_eq!(out["keep"], json!("kept"));
        assert_eq!(out["authored_extension"]["keep"], json!("kept"));
        assert!(out["authored_extension"].get("chat_history").is_none());
        // Scalars and arrays survive untouched apart from nested stripping.
        assert_eq!(retained_context(&json!("scalar")), json!("scalar"));
        assert_eq!(retained_context(&json!([{"session":1}])), json!([{}]));
    }

    #[test]
    fn profile_binds_only_to_a_matching_owner_vocabulary() {
        let owner = ScriptedVocabulary;
        let profile = Profile::bind(&owner).expect("scripted owner admits positions 0..5");
        for p in 0..=5u8 {
            assert!(profile.role(p).is_ok(), "role for P{p}");
        }
        assert_eq!(profile.role(0).unwrap(), "frame");
        assert_eq!(profile.role(4).unwrap(), "evaluation");
        assert!(profile.position(6).is_err(), "P6 is not admitted");
        assert!(profile.role(6).is_err());
    }
}
