//! Kernel-native control arm. The QL-MEF vak composition contract
//! (`ql.vak-composition/v1`, invoked as `ql vak compose <request.json> --json`)
//! answers the loop's two control questions over the circuit's own positional
//! field: which office the returned difference belongs to (interpret-return)
//! and which office the next exterior act serves (next-act). The kernel admits
//! the supplied reading against its registry and positional law; it does not
//! infer semantics — the mapping from the difference's structural facts to a
//! Vāk operator is explicit below and recorded in the witness. A refused or
//! mismatched kernel call is an error (fail closed); control never degrades to
//! the model.
use crate::{
    evidence::{bytes_digest, stable_digest},
    process::ProcessSpec,
    relational::{Act, Circuit},
    Error, Result,
};
use serde_json::{json, Value};
use std::path::PathBuf;

pub const CONTRACT: &str = "ql.vak-composition/v1";
/// Frozen AIKit operative syntax acceptance the QL kernel validates against
/// (`ql_mef::vak_oi`); readings under any other revision are refused there.
const AIKIT_SYNTAX_VERSION: &str = "aikit.operative-resolve/v1";
const AIKIT_OWNER_REVISION: &str = "4e35f499c50b987551ab124b4432757973e823ae";
const READING_CONTRACT: &str = "vak-expression-reading-v1";
const CALLER: &str = "actuation:vak-control";

/// The kernel's own operator-to-position law (`VakRelationOp::position` in
/// ql-mef): the six Vāk relation operators are the six offices P0..P5. Mirrored
/// here as an explicit table; the fixture tests pin it to real kernel replies.
pub const GLYPH_POSITIONS: [(&str, u8); 6] =
    [("@#", 0), ("-", 1), ("+", 2), ("x", 3), ("/", 4), ("=", 5)];

pub fn glyph_position(glyph: &str) -> Option<u8> {
    GLYPH_POSITIONS
        .iter()
        .find(|(g, _)| *g == glyph)
        .map(|(_, p)| *p)
}

fn glyph_of(position: u8) -> &'static str {
    GLYPH_POSITIONS
        .iter()
        .find(|(_, p)| *p == position)
        .map(|(g, _)| *g)
        .unwrap_or("@#")
}

/// One member per office: a constellation refuses duplicate coordinates, so
/// live residues aggregate to their position and vacant offices stay explicit.
fn members(circuit: &Circuit) -> [bool; 6] {
    let mut realised = [false; 6];
    for r in &circuit.residues {
        if !r.invalidated {
            realised[r.position.min(5) as usize] = true;
        }
    }
    realised
}

fn frame() -> Value {
    json!({"id":"CF1","lens":"L0","basis":"chromatic","face":"direct","positions":"local"})
}

fn circuit_basis(circuit: &Circuit, run_id: &str, standing: &str, extra: &[String]) -> Value {
    let mut evidence = vec![format!("circuit:{}", circuit.id), format!("trace:{run_id}")];
    evidence.extend(extra.iter().cloned());
    json!({"caller":CALLER,"source":circuit.id,"revision":run_id,"standing":standing,"evidence":evidence})
}

fn circuit_whole(circuit: &Circuit, run_id: &str, last_return: &str) -> Value {
    let realised = members(circuit);
    let id = &circuit.id;
    let step = json!({"op":"whole","useRef":format!("circuit:{id}:state"),"subjectRef":format!("circuit:{id}"),
        "wholeRef":format!("anchor:circuit:{id}"),"category":"M",
        "groundRef":format!("ground:circuit:{id}"),"groundFace":"direct","frame":frame(),
        "basis":circuit_basis(circuit, run_id, "OBSERVED", &[]),
        "members":(0..6).map(|p|json!({"subjectRef":format!("circuit:{id}:p{p}{}", if realised[p as usize] {""} else {":vacant"}),
            "position":p,"face":"direct"})).collect::<Vec<_>>(),
        "sourceReturns":[{"fromRef":last_return,"anchorRef":format!("anchor:circuit:{id}"),
            "groundRef":format!("ground:circuit:{id}"),"face":"direct","kind":"own"}]});
    step
}

/// The structural facts of a returned difference, mapped explicitly onto the
/// Vāk operator whose office receives the difference. The rows restate the
/// loop's own worked semantics (material P1, effect P2, pattern P3, whole-
/// relative evaluation P4, delivered realisation P5) in kernel vocabulary.
fn difference_reading(
    circuit_id: &str,
    act: &Act,
    difference: &Value,
) -> (String, u8, &'static str, &'static str, Vec<String>) {
    let success = difference["operation_success"] == true;
    let kind = act.carrier["kind"].as_str().unwrap_or_default();
    let content = difference["raw_result"]["content"]
        .as_str()
        .unwrap_or_default();
    let facts = |carrier_fact: &str| {
        vec![
            format!("circuit:{circuit_id}"),
            difference["id"].as_str().unwrap_or_default().to_owned(),
            carrier_fact.to_owned(),
            format!("operation_success:{success}"),
        ]
    };
    let (glyph, position, field, self_other, fact): (&str, u8, &str, &str, String) = if !success {
        ("-", 1, "R#", "?-", "operation-failure".into())
    } else if kind == "model" {
        if content.trim().is_empty() {
            ("+", 2, "N#", "!?", "carrier:model:content-empty".into())
        } else {
            ("=", 5, "N#", "!?", "carrier:model:content-delivered".into())
        }
    } else if kind == "internal_control" {
        ("/", 4, "##", "!-", "carrier:internal_control".into())
    } else {
        let name = act.carrier["name"].as_str().unwrap_or_default();
        match name {
            "write_file" => ("+", 2, "R#", "?-", "carrier:capability:write_file".into()),
            "read_file" | "list_files" => {
                ("-", 1, "R#", "?-", format!("carrier:capability:{name}"))
            }
            "run_tests" => ("/", 4, "R#", "?-", "carrier:capability:run_tests".into()),
            _ => ("x", 3, "R#", "?-", format!("carrier:capability:{name}")),
        }
    };
    let mut evidence = facts(&fact);
    evidence.push(format!("glyph:{glyph}"));
    (glyph.to_owned(), position, field, self_other, evidence)
}

fn reading_language(
    circuit: &Circuit,
    act_ref: &str,
    glyph: &str,
    position: u8,
    field: &str,
    self_other: &str,
) -> Value {
    let id = &circuit.id;
    json!({"acceptedSyntaxRevision":AIKIT_OWNER_REVISION,"nativeNodeRef":act_ref,
        "selfOther":self_other,"field":field,"interpreter":CALLER,"expectedGround":null,
        "general":{"syntaxVersion":AIKIT_SYNTAX_VERSION,"ownerRevision":AIKIT_OWNER_REVISION,
            "resolvePathIdentity":format!("actuation://ql/{id}/{act_ref}"),
            "rendered":format!("vak-control reading of returned difference {act_ref}"),
            "fullVakRendering":format!("{glyph} returned-difference of {act_ref}"),
            "evidence":[format!("actuation:vak-control:{id}"),act_ref]},
        "reading":{"contract":READING_CONTRACT,"operator":glyph,"horizon":format!("@{position}"),
            "subjects":[{"native":format!("circuit:{id}")}],"relationRefs":[],"complementRefs":[],
            "worldRef":null,"projectRef":null,"focusRef":null,"expectedReturn":null,
            "standing":"DERIVED","evidence":[format!("actuation:vak-control:{id}"),act_ref]}})
}

/// Kernel-admitted reading of a returned difference: the destination office is
/// the admitted operator's own position in the kernel's law.
#[derive(Clone, Debug)]
pub struct VakReading {
    pub destination: u8,
    pub operator: String,
    pub request_digest: String,
    pub result_digest: String,
    pub source_revision: String,
    pub harmonic_pitch: Value,
    pub focus_interval: Value,
}
impl VakReading {
    pub fn kernel_evidence(&self) -> Value {
        json!({"operator":self.operator,"destination":format!("P{}",self.destination),
            "source_revision":self.source_revision,"harmonic_pitch":self.harmonic_pitch,
            "focus_interval":self.focus_interval,"result_digest":self.result_digest})
    }
}

/// The kernel-named next act: a capability carrier for the leading office's
/// faculty, or a closure request when no exterior act remains.
#[derive(Clone, Debug)]
pub struct VakNextAct {
    pub closure: bool,
    pub carrier: Option<Value>,
    pub intent: String,
    pub lead_operator: String,
    pub allowed_offices: Vec<String>,
    pub request_digest: String,
    pub result_digest: String,
    pub source_revision: String,
}
impl VakNextAct {
    pub fn kernel_evidence(&self) -> Value {
        json!({"lead_operator":self.lead_operator,"allowed_offices":self.allowed_offices,
            "closure":self.closure,"carrier":self.carrier,
            "source_revision":self.source_revision,"result_digest":self.result_digest})
    }
}

pub struct VakControl {
    spec: ProcessSpec,
    scratch: tempfile::TempDir,
    binary_digest: String,
    source_revision: String,
}

impl VakControl {
    /// Bind one exact `ql` binary. The bind probe runs a real minimal
    /// composition through the supplied process and requires the
    /// ql.vak-composition/v1 envelope back before any control turn is trusted.
    pub fn bind(program: PathBuf) -> Result<Self> {
        if !program.is_absolute() || !program.is_file() {
            return Err(Error::new(
                "vak control requires an absolute path to a regular ql binary",
            ));
        }
        let scratch = tempfile::tempdir()
            .map_err(|e| Error::new(format!("vak control scratch unavailable: {e}")))?;
        let request_path = scratch.path().join("request.json");
        let spec = ProcessSpec {
            program,
            args: vec![
                "vak".into(),
                "compose".into(),
                request_path.to_string_lossy().into_owned(),
                "--json".into(),
            ],
            cwd: scratch.path().to_owned(),
            environment: Default::default(),
            timeout_ms: 10_000,
            output_limit: 16 * 1024 * 1024,
        };
        let control =
            Self {
                binary_digest: bytes_digest(&std::fs::read(&spec.program).map_err(|_| {
                    Error::new("vak control instrument is unavailable or unreadable")
                })?),
                spec,
                scratch,
                source_revision: String::new(),
            };
        let probe = Self::bind_probe_request();
        let response = control.call(&probe)?;
        let whole = response["results"]
            .as_array()
            .and_then(|r| r.first())
            .ok_or_else(|| Error::new("vak control bind probe returned no results"))?;
        if whole["op"] != "whole"
            || whole["result"]["useRef"] != json!("circuit:vak-bind-probe:state")
        {
            return Err(Error::new(
                "vak control bind probe did not admit the probe circuit",
            ));
        }
        Ok(control)
    }
    pub fn basis(&self) -> Value {
        json!({"contract":CONTRACT,"program":self.spec.program,"instrument_sha256":self.binary_digest,
            "source_revision":self.source_revision})
    }
    /// The minimal composition every bound instrument must admit before use.
    pub fn bind_probe_request() -> Value {
        let id = "vak-bind-probe";
        let run_id = "trace:bind-probe";
        let circuit = Circuit {
            id: id.into(),
            parent_id: None,
            depth: 0,
            face: "direct".into(),
            frame: json!({"id":format!("{id}:frame")}),
            active_position: 0,
            closure_state: "open".into(),
            residues: vec![crate::relational::Residue {
                id: format!("{id}:res:frame"),
                position: 0,
                kind: "frame".into(),
                value: json!({"id":format!("{id}:frame")}),
                provenance: json!({"probe":true}),
                invalidated: false,
            }],
            trajectory: vec![],
            children: vec![],
            conjugates: vec![],
            success_state: Value::Null,
        };
        let whole = circuit_whole(&circuit, run_id, &format!("circuit:{id}:origin"));
        json!({"contract":CONTRACT,"steps":[whole]})
    }
    fn call(&self, request: &Value) -> Result<Value> {
        if bytes_digest(
            &std::fs::read(&self.spec.program)
                .map_err(|_| Error::new("vak control instrument is unavailable or unreadable"))?,
        ) != self.binary_digest
        {
            return Err(Error::new("bound vak control instrument bytes changed"));
        }
        std::fs::write(
            self.scratch.path().join("request.json"),
            serde_json::to_vec(request).map_err(|e| Error::new(e.to_string()))?,
        )
        .map_err(|e| Error::new(format!("vak control request not written: {}", e.kind())))?;
        let r = self.spec.run(&[])?;
        if r.code != Some(0) {
            let why: String = r.stderr.chars().take(512).collect();
            return Err(Error::new(format!(
                "vak composition kernel refused (exit {:?}): {}; no fallback to model control",
                r.code, why
            )));
        }
        let v: Value = serde_json::from_str(&r.stdout)
            .map_err(|_| Error::new("vak composition kernel returned invalid JSON"))?;
        if v["contract"] != CONTRACT || !v["results"].is_array() {
            return Err(Error::new(
                "vak composition kernel reply is not a ql.vak-composition/v1 envelope",
            ));
        }
        if v["sourceRevision"].as_str().map(str::is_empty) != Some(false) {
            return Err(Error::new(
                "vak composition kernel reply omits its source revision",
            ));
        }
        Ok(v)
    }
    fn expected_ops(response: &Value, ops: &[&str]) -> Result<()> {
        let actual = response["results"]
            .as_array()
            .ok_or_else(|| Error::new("vak composition envelope has no results"))?;
        if actual.len() != ops.len()
            || actual
                .iter()
                .zip(ops)
                .any(|(r, op)| r["op"].as_str() != Some(op))
        {
            return Err(Error::new(format!(
                "vak composition result sequence does not match the requested {ops:?} steps"
            )));
        }
        Ok(())
    }

    /// Interpret-return: bind the circuit's positional field, attach the
    /// supplied derived reading of the returned difference as a new immutable
    /// use of the same whole, and read the admitted result back. The kernel
    /// names the office; nothing here falls back to the model.
    pub fn interpret_return(
        &self,
        circuit: &Circuit,
        act: &Act,
        difference: &Value,
        run_id: &str,
    ) -> Result<VakReading> {
        let act_ref = difference["id"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| Error::new("vak interpret requires the returned difference id"))?;
        let (glyph, position, field, self_other, evidence) =
            difference_reading(&circuit.id, act, difference);
        let into = format!(
            "circuit:{}:interpret:{}",
            circuit.id,
            circuit.trajectory.len()
        );
        let request = json!({"contract":CONTRACT,"steps":[
            circuit_whole(circuit, run_id, &last_return_ref(circuit)),
            {"op":"interpret","from":format!("circuit:{}:state",circuit.id),"into":into,
             "language":reading_language(circuit, act_ref, &glyph, position, field, self_other),
             "basis":{"caller":CALLER,"source":act_ref,"revision":run_id,"standing":"DERIVED",
                      "evidence":&evidence[..evidence.len()-1]}},
            {"op":"read","useRef":into,"lens":"L0"}]});
        let response = self.call(&request)?;
        Self::expected_ops(&response, &["whole", "interpret", "read"])?;
        let admitted = response["results"][1]["result"]["language"]["reading"]["operator"]
            .as_str()
            .ok_or_else(|| Error::new("vak interpret result carries no admitted operator"))?;
        if admitted != glyph {
            return Err(Error::new(format!(
                "vak interpret admitted operator {admitted} instead of the requested {glyph}"
            )));
        }
        let echoed = response["results"][2]["result"]["language"]["reading"]["operator"]
            .as_str()
            .ok_or_else(|| Error::new("vak read result carries no attached operator"))?;
        if echoed != glyph {
            return Err(Error::new(format!(
                "vak read echoed operator {echoed} instead of the admitted {glyph}"
            )));
        }
        let destination = glyph_position(admitted)
            .ok_or_else(|| Error::new(format!("vak operator {admitted} names no office P0..P5")))?;
        if destination != position {
            return Err(Error::new(format!(
                "vak reading maps operator {glyph} to P{destination}, not the mapped P{position}"
            )));
        }
        Ok(VakReading {
            destination,
            operator: glyph.to_owned(),
            request_digest: stable_digest(&request),
            result_digest: stable_digest(&response),
            source_revision: response["sourceRevision"]
                .as_str()
                .unwrap_or_default()
                .into(),
            harmonic_pitch: response["results"][2]["result"]["harmonicPitch"].clone(),
            focus_interval: response["results"][2]["result"]["focusInterval"].clone(),
        })
    }

    /// Next-act: admit the vacant offices through the kernel's CPF exclusion,
    /// walk the direct face from the active position to the next vacant office,
    /// and let the admitted operator name the faculty. Offices whose faculty
    /// needs authored content refuse with a typed error; a carrier is never
    /// invented.
    pub fn next_act(&self, circuit: &Circuit, run_id: &str) -> Result<VakNextAct> {
        let realised = members(circuit);
        let vacant: Vec<u8> = (0..6).filter(|p| !realised[*p as usize]).collect();
        if vacant.is_empty() {
            // Every office carries a live residue: no exterior act remains and
            // the loop law routes to determination. No kernel turn is needed to
            // see an empty vacancy set, so the witness states the rule itself.
            return Ok(VakNextAct {
                closure: true,
                carrier: None,
                intent: "All six offices are realised; route to determination.".into(),
                lead_operator: String::new(),
                allowed_offices: vec![],
                request_digest: String::new(),
                result_digest: String::new(),
                source_revision: self.source_revision.clone(),
            });
        }
        let active = circuit.active_position.min(5);
        let candidates: Vec<&str> = vacant.iter().map(|p| glyph_of(*p)).collect();
        let vacant_offices = vacant
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let request = json!({"contract":CONTRACT,"steps":[
            circuit_whole(circuit, run_id, &last_return_ref(circuit)),
            {"op":"enter","id":"ctx","useRef":format!("circuit:{}:state",circuit.id),
             "categoryGround":{"position":active,"face":"direct"}},
            {"op":"cpf","context":"ctx","into":format!("circuit:{}:cpf",circuit.id),
             "face":"direct","operators":candidates,
             "basis":circuit_basis(circuit, run_id, "OBSERVED",
                                   &[format!("vacant-offices:{vacant_offices}")])}]});
        let response = self.call(&request)?;
        Self::expected_ops(&response, &["whole", "enter", "cpf"])?;
        let allowed: Vec<String> = response["results"][2]["result"]["allowedOperators"]
            .as_array()
            .ok_or_else(|| Error::new("vak cpf result carries no admitted operator set"))?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| Error::new("vak admitted operator is not a glyph"))
            })
            .collect::<Result<_>>()?;
        // The kernel may only narrow the supplied vacancy set, never widen it.
        if allowed
            .iter()
            .any(|g| glyph_position(g).is_none_or(|p| !vacant.contains(&p)))
        {
            return Err(Error::new(format!(
                "vak cpf admitted offices outside the supplied vacancy set: {allowed:?}"
            )));
        }
        // Direct-face walk from the active office to the next vacant one.
        let lead_position = (1..=6)
            .map(|step| (active + step) % 6)
            .find(|p| vacant.contains(p))
            .ok_or_else(|| Error::new("vak next-act found no vacant office to walk to"))?;
        let lead = glyph_of(lead_position);
        if !allowed.contains(&lead.to_owned()) {
            return Err(Error::new(format!(
                "vak cpf did not admit the walked office {lead} at P{lead_position}"
            )));
        }
        let id = &circuit.id;
        let (closure, carrier, intent) = match lead {
            "-" => (
                false,
                Some(json!({"kind":"capability","name":"list_files","args":{"path":"."}})),
                format!("Serve the Distinguish faculty at P{lead_position}: take in the given material."),
            ),
            "x" | "/" => (
                false,
                Some(json!({"kind":"capability","name":"run_tests","args":{"files":[]}})),
                format!("Serve the faculty at P{lead_position}: relate the built form to its checks."),
            ),
            "=" => (
                true,
                None,
                format!("The walk reached Express at P{lead_position}: route to determination."),
            ),
            other => {
                let name = match other {
                    "@" => "Potential",
                    "+" => "Affirm",
                    _ => "unknown",
                };
                return Err(Error::new(format!(
                    "vak next-act named the {name} faculty ({other}) at office P{lead_position}; \
                     carrying it requires authored content no vak operation supplies — \
                     refusing to invent a carrier"
                )));
            }
        };
        Ok(VakNextAct {
            closure,
            carrier,
            intent,
            lead_operator: lead.to_owned(),
            allowed_offices: allowed,
            request_digest: stable_digest(&request),
            result_digest: stable_digest(&response),
            source_revision: response["sourceRevision"]
                .as_str()
                .unwrap_or_default()
                .into(),
        })
    }
}

fn last_return_ref(circuit: &Circuit) -> String {
    circuit
        .trajectory
        .last()
        .and_then(|t| t["interpretation_ref"].as_str())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("circuit:{}:origin", circuit.id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relational::Residue;

    fn fixture_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../experiments/native-research/vak-control-fixtures")
    }
    fn fixture(name: &str) -> Value {
        serde_json::from_str(
            &std::fs::read_to_string(fixture_dir().join(name))
                .unwrap_or_else(|e| panic!("fixture {name} unavailable: {e}")),
        )
        .unwrap()
    }
    /// A circuit shaped like the captured fixtures: trace:t1 with the frame at
    /// P0, one completed act whose return is resident at P1, one material
    /// residue, active position P1.
    fn fixture_circuit() -> Circuit {
        let residue = |n: usize, p: u8, kind: &str| Residue {
            id: format!("trace:t1:c0:res:{n}"),
            position: p,
            kind: kind.into(),
            value: json!({"fixture":true}),
            provenance: json!({"fixture":true}),
            invalidated: false,
        };
        Circuit {
            id: "trace:t1:c0".into(),
            parent_id: None,
            depth: 0,
            face: "direct".into(),
            frame: json!({"id":"trace:t1:c0:frame","initiating_intent":"fixture intent"}),
            active_position: 1,
            closure_state: "open".into(),
            residues: vec![residue(0, 0, "frame"), residue(1, 1, "material")],
            trajectory: vec![json!({"id":"trace:t1:c0:transition:0","from":0,"to":1,
                "interpretation_ref":"trace:t1:c0:act:0:return"})],
            children: vec![],
            conjugates: vec![],
            success_state: Value::Null,
        }
    }
    const RUN_ID: &str = "trace:t1";

    #[test]
    fn glyph_table_matches_the_kernel_operator_positions() {
        for (glyph, position) in GLYPH_POSITIONS {
            assert_eq!(glyph_position(glyph), Some(position));
        }
        assert_eq!(glyph_position("%"), None);
        assert_eq!(glyph_of(5), "=");
    }

    #[test]
    fn interpret_request_matches_the_captured_affirm_fixture() {
        let circuit = fixture_circuit();
        let act = Act {
            source_position: Some(1),
            intent: json!("write the deliverable"),
            carrier: json!({"kind":"capability","name":"write_file","args":{"path":"deliverable.md"}}),
            input_residue_refs: vec![],
            nested: None,
            metadata: Value::Null,
        };
        let difference = json!({"id":"trace:t1:c0:act:1:return","act_id":"trace:t1:c0:act:1",
            "raw_result":{"path":"deliverable.md"},"operation_success":true});
        let reading = difference_reading(&circuit.id, &act, &difference);
        assert_eq!(reading.0, "+");
        assert_eq!(reading.1, 2);
        let (_, position, field, self_other, evidence) = reading;
        let request = json!({"contract":CONTRACT,"steps":[
            circuit_whole(&circuit, RUN_ID, &last_return_ref(&circuit)),
            {"op":"interpret","from":"circuit:trace:t1:c0:state","into":"circuit:trace:t1:c0:interpret:1",
             "language":reading_language(&circuit, "trace:t1:c0:act:1:return", "+", position, field, self_other),
             "basis":{"caller":CALLER,"source":"trace:t1:c0:act:1:return","revision":RUN_ID,"standing":"DERIVED",
                      "evidence":&evidence[..evidence.len()-1]}},
            {"op":"read","useRef":"circuit:trace:t1:c0:interpret:1","lens":"L0"}]});
        let fixture = fixture("interpret-return-affirm.request.json");
        assert_eq!(
            request, fixture,
            "the built request must equal the real captured request"
        );
    }

    #[test]
    fn interpret_mapping_rows_cover_the_loop_semantics() {
        let circuit = fixture_circuit();
        let act_for = |carrier: Value| Act {
            source_position: Some(1),
            intent: json!("fixture"),
            carrier,
            input_residue_refs: vec![],
            nested: None,
            metadata: Value::Null,
        };
        let diff = |success: bool, content: &str| {
            json!({"id":"trace:t1:c0:act:1:return","operation_success":success,
                "raw_result":{"content":content}})
        };
        let (glyph, position, ..) = difference_reading(
            &circuit.id,
            &act_for(json!({"kind":"capability","name":"write_file"})),
            &diff(true, ""),
        );
        assert_eq!((glyph.as_str(), position), ("+", 2));
        let (glyph, position, ..) = difference_reading(
            &circuit.id,
            &act_for(json!({"kind":"capability","name":"read_file"})),
            &diff(true, ""),
        );
        assert_eq!((glyph.as_str(), position), ("-", 1));
        let (glyph, position, ..) = difference_reading(
            &circuit.id,
            &act_for(json!({"kind":"capability","name":"list_files"})),
            &diff(true, ""),
        );
        assert_eq!((glyph.as_str(), position), ("-", 1));
        let (glyph, position, ..) = difference_reading(
            &circuit.id,
            &act_for(json!({"kind":"capability","name":"run_tests"})),
            &diff(true, ""),
        );
        assert_eq!((glyph.as_str(), position), ("/", 4));
        let (glyph, position, ..) = difference_reading(
            &circuit.id,
            &act_for(json!({"kind":"model"})),
            &diff(true, "the answer"),
        );
        assert_eq!((glyph.as_str(), position), ("=", 5));
        let (glyph, position, ..) = difference_reading(
            &circuit.id,
            &act_for(json!({"kind":"model"})),
            &diff(true, "  "),
        );
        assert_eq!((glyph.as_str(), position), ("+", 2));
        let (glyph, position, ..) = difference_reading(
            &circuit.id,
            &act_for(json!({"kind":"internal_control","name":"hold"})),
            &diff(true, ""),
        );
        assert_eq!((glyph.as_str(), position), ("/", 4));
        let (glyph, position, ..) = difference_reading(
            &circuit.id,
            &act_for(json!({"kind":"capability","name":"write_file"})),
            &diff(false, ""),
        );
        assert_eq!(
            (glyph.as_str(), position),
            ("-", 1),
            "failure returns to material"
        );
    }

    #[test]
    fn interpret_parser_reads_the_real_affirm_response() {
        // The parser is exercised through a bound control against a scripted
        // instrument in execution.rs; here the destination law is pinned
        // against the captured kernel replies.
        let response = fixture("interpret-return-affirm.response.json");
        let admitted = response["results"][1]["result"]["language"]["reading"]["operator"]
            .as_str()
            .unwrap();
        assert_eq!(glyph_position(admitted), Some(2));
        assert_eq!(response["contract"], json!(CONTRACT));
        assert!(!response["sourceRevision"].as_str().unwrap().is_empty());
        for name in [
            "interpret-return-distinguish",
            "interpret-return-contextualise",
            "interpret-return-express",
        ] {
            let response = fixture(&format!("{name}.response.json"));
            let admitted = response["results"][1]["result"]["language"]["reading"]["operator"]
                .as_str()
                .unwrap();
            assert!(
                glyph_position(admitted).is_some(),
                "{name} admits an office"
            );
        }
        let express = fixture("interpret-return-express.response.json");
        assert_eq!(
            glyph_position(
                express["results"][1]["result"]["language"]["reading"]["operator"]
                    .as_str()
                    .unwrap()
            ),
            Some(5)
        );
    }

    #[test]
    fn next_act_request_matches_the_captured_material_fixture() {
        let circuit = fixture_circuit();
        let vacant: Vec<u8> = (0..6).filter(|p| p != &0 && p != &1).collect();
        let candidates: Vec<&str> = vacant.iter().map(|p| glyph_of(*p)).collect();
        let request = json!({"contract":CONTRACT,"steps":[
            circuit_whole(&circuit, RUN_ID, "trace:t1:c0:act:0:return"),
            {"op":"enter","id":"ctx","useRef":"circuit:trace:t1:c0:state",
             "categoryGround":{"position":1,"face":"direct"}},
            {"op":"cpf","context":"ctx","into":"circuit:trace:t1:c0:cpf","face":"direct",
             "operators":candidates,
             "basis":circuit_basis(&circuit, RUN_ID, "OBSERVED",
                                   &["vacant-offices:2,3,4,5".to_owned()])}]});
        assert_eq!(
            request,
            fixture("next-act-material.request.json"),
            "the built next-act request must equal the real captured request"
        );
    }

    #[test]
    fn next_act_parser_walks_to_the_first_admitted_vacant_office() {
        let response = fixture("next-act-material.response.json");
        let allowed: Vec<String> = response["results"][2]["result"]["allowedOperators"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_owned())
            .collect();
        assert_eq!(allowed, ["+", "x", "/", "="]);
        // Active P1: the direct-face walk reaches P2 first, named by "+".
        let vacant: Vec<u8> = vec![2, 3, 4, 5];
        let lead = (1..=6)
            .map(|step| (1 + step) % 6)
            .find(|p| vacant.contains(p))
            .unwrap();
        assert_eq!(lead, 2);
        assert_eq!(glyph_of(lead), "+");
    }

    #[test]
    fn bind_probe_request_matches_the_captured_probe_fixture() {
        let probe = VakControl::bind_probe_request();
        assert_eq!(probe, fixture("bind-probe.request.json"));
        let response = fixture("bind-probe.response.json");
        assert_eq!(response["results"][0]["op"], json!("whole"));
        assert_eq!(
            response["results"][0]["result"]["useRef"],
            json!("circuit:vak-bind-probe:state")
        );
    }

    #[test]
    fn empty_vacancy_yields_closure_without_a_kernel_turn() {
        // next_act's all-realised rule is pure: no instrument is involved. The
        // branch is pinned here through the members computation.
        let mut circuit = fixture_circuit();
        for p in 2..=5u8 {
            circuit.residues.push(Residue {
                id: format!("trace:t1:c0:res:{p}"),
                position: p,
                kind: "frame".into(),
                value: json!({}),
                provenance: json!({}),
                invalidated: false,
            });
        }
        let realised = members(&circuit);
        assert!(realised.iter().all(|r| *r), "all offices realised");
        let vacant: Vec<u8> = (0..6).filter(|p| !realised[*p as usize]).collect();
        assert!(vacant.is_empty());
    }

    #[test]
    fn a_relative_or_missing_instrument_is_refused_at_bind() {
        assert!(VakControl::bind(PathBuf::from("relative/ql")).is_err());
        assert!(VakControl::bind(PathBuf::from("/nonexistent/ql")).is_err());
    }
}
