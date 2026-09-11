use crate::{value::*, Error, Result};
use actuation_stream::Timestamp;
use serde::Serialize;
use serde_json::{json, Value};
pub const EPISTEMIC_RECORD_SCHEMA: &str = "actuation.epistemic-record/v0";
pub const RECORD_KINDS: &[&str] = &[
    "EpistemicCorpus",
    "EpistemicAnnotation",
    "DisclosureTrace",
    "InteriorObservation",
    "InteriorIntervention",
    "CultivationRun",
    "EpistemicEvaluation",
    "StructuralFinding",
];
pub const ACCESS_KINDS: &[&str] = &[
    "behavioural",
    "output_state",
    "internal_read",
    "internal_write",
    "causal",
    "learning",
];
/// Only validated construction is public. Access assertions remain declarations,
/// not a claim by the store that a provider or model interior was exercised.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct EpistemicRecord(Value);
impl EpistemicRecord {
    pub fn read(input: &Value) -> Result<Self> {
        exact(
            input,
            &[
                "schema",
                "record_kind",
                "record_ref",
                "recorded_at",
                "provenance",
                "access",
                "subject_refs",
                "model_ref",
                "checkpoint_ref",
                "method_refs",
                "coordinate_refs",
                "derivation_refs",
                "evidence_refs",
                "payload",
            ],
            "record",
        )?;
        if input["schema"] != EPISTEMIC_RECORD_SCHEMA {
            return Err(Error::new("wrong epistemic record schema"));
        }
        let kind = one_of(&input["record_kind"], RECORD_KINDS, "record_kind")?;
        let date = text(&input["recorded_at"], "recorded_at")?;
        if !date.ends_with('Z') {
            return Err(Error::new("recorded_at requires UTC Z"));
        }
        Timestamp::new(date.clone())?;
        let mut record = json!({"schema":EPISTEMIC_RECORD_SCHEMA,"record_kind":kind,"record_ref":text(&input["record_ref"],"record_ref")?,"recorded_at":date,"provenance":provenance(&input["provenance"] )?,"access":access(&input["access"] )?,"subject_refs":refs(&input["subject_refs"],true,"subject_refs")?,"method_refs":refs(&input["method_refs"],false,"method_refs")?,"coordinate_refs":refs(&input["coordinate_refs"],false,"coordinate_refs")?,"derivation_refs":refs(&input["derivation_refs"],false,"derivation_refs")?,"evidence_refs":refs(&input["evidence_refs"],false,"evidence_refs")?,"payload":payload(&kind,&input["payload"])?});
        optional_text(input, &mut record, "model_ref")?;
        optional_text(input, &mut record, "checkpoint_ref")?;
        if !record["checkpoint_ref"].is_null() && record["model_ref"].is_null() {
            return Err(Error::new("checkpoint_ref requires model_ref"));
        }
        if [
            "InteriorObservation",
            "InteriorIntervention",
            "CultivationRun",
        ]
        .contains(&kind.as_str())
            && (record["model_ref"].is_null()
                || record["checkpoint_ref"].is_null()
                || record["method_refs"].as_array().unwrap().is_empty()
                || record["coordinate_refs"].as_array().unwrap().is_empty())
        {
            return Err(Error::new(
                "interior/cultivation record requires model, checkpoint, methods and coordinates",
            ));
        }
        if kind == "InteriorObservation"
            && record["access"]["internal_read"]["state"] != "available"
        {
            return Err(Error::new(
                "InteriorObservation requires evidenced internal_read access",
            ));
        }
        if kind == "InteriorIntervention"
            && record["access"]["internal_write"]["state"] != "available"
            && record["access"]["causal"]["state"] != "available"
        {
            return Err(Error::new(
                "InteriorIntervention requires evidenced internal_write or causal access",
            ));
        }
        if kind == "StructuralFinding"
            && ["supported", "refuted"]
                .contains(&record["payload"]["finding_state"].as_str().unwrap_or(""))
            && record["evidence_refs"].as_array().unwrap().is_empty()
        {
            return Err(Error::new("supported/refuted finding requires evidence"));
        }
        Ok(Self(record))
    }
    pub fn as_value(&self) -> &Value {
        &self.0
    }
    pub fn into_value(self) -> Value {
        self.0
    }
    pub fn record_ref(&self) -> &str {
        self.0["record_ref"].as_str().expect("validated record")
    }
}
fn provenance(input: &Value) -> Result<Value> {
    exact(
        input,
        &[
            "kind",
            "actor_ref",
            "generator_ref",
            "source_refs",
            "derivation_refs",
        ],
        "provenance",
    )?;
    let kind = one_of(
        &input["kind"],
        &["source", "human", "agent", "transformed", "synthetic"],
        "provenance.kind",
    )?;
    let mut r = json!({"kind":kind,"source_refs":refs(&input["source_refs"],false,"source_refs")?,"derivation_refs":refs(&input["derivation_refs"],false,"derivation_refs")?});
    optional_text(input, &mut r, "actor_ref")?;
    optional_text(input, &mut r, "generator_ref")?;
    let source = r["source_refs"].as_array().unwrap().is_empty();
    if (kind == "source" && source)
        || (["human", "agent"].contains(&kind.as_str()) && r["actor_ref"].is_null())
        || (kind == "transformed"
            && (source || r["derivation_refs"].as_array().unwrap().is_empty()))
        || (kind == "synthetic" && r["generator_ref"].is_null())
    {
        return Err(Error::new("provenance is missing its required attribution"));
    }
    Ok(r)
}
fn access(input: &Value) -> Result<Value> {
    exact(input, ACCESS_KINDS, "access")?;
    let mut r = json!({});
    for kind in ACCESS_KINDS {
        let v = &input[kind];
        exact(
            v,
            &["state", "reason", "method_refs", "evidence_refs"],
            kind,
        )?;
        let state = one_of(
            &v["state"],
            &["available", "unavailable", "not-assessed"],
            "access state",
        )?;
        let mut entry = json!({"state":state,"method_refs":refs(&v["method_refs"],false,"access.method_refs")?,"evidence_refs":refs(&v["evidence_refs"],false,"access.evidence_refs")?});
        optional_text(v, &mut entry, "reason")?;
        let m = entry["method_refs"].as_array().unwrap().is_empty();
        let e = entry["evidence_refs"].as_array().unwrap().is_empty();
        if (state == "available" && (m || e))
            || (state != "available" && entry["reason"].is_null())
            || (state == "not-assessed" && (!m || !e))
        {
            return Err(Error::new(format!(
                "unwarranted access declaration: {kind}"
            )));
        }
        r[kind] = entry;
    }
    Ok(r)
}
fn payload(kind: &str, input: &Value) -> Result<Value> {
    let fields: &[&str] = match kind {
        "EpistemicCorpus" => &["corpus_ref", "revision_ref", "item_refs"],
        "EpistemicAnnotation" => &[
            "annotation_ref",
            "annotation_kind",
            "content_ref",
            "source_span_refs",
        ],
        "DisclosureTrace" => &["trace_ref", "condition_ref", "step_refs"],
        "InteriorObservation" => &["observation_ref", "observed_quantity", "coordinate_ref"],
        "InteriorIntervention" => &["intervention_ref", "operation", "before_ref", "after_ref"],
        "CultivationRun" => &[
            "run_ref",
            "input_corpus_ref",
            "condition_ref",
            "result_state_ref",
        ],
        "EpistemicEvaluation" => &["evaluation_ref", "criteria_refs", "evaluation_state"],
        "StructuralFinding" => &["finding_ref", "statement", "finding_state"],
        _ => return Err(Error::new("unknown payload kind")),
    };
    exact(input, fields, "payload")?;
    let mut r = json!({});
    for field in fields {
        r[field] = if field.ends_with("_refs") {
            refs(&input[field], true, field)?
        } else {
            json!(text(&input[field], field)?)
        };
    }
    if kind == "EpistemicEvaluation" {
        one_of(
            &r["evaluation_state"],
            &["declared", "completed", "inconclusive"],
            "evaluation_state",
        )?;
    }
    if kind == "StructuralFinding" {
        one_of(
            &r["finding_state"],
            &["hypothesis", "supported", "refuted", "inconclusive"],
            "finding_state",
        )?;
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Map, Value};

    fn access_entry(state: &str) -> Value {
        match state {
            "available" => {
                json!({"state":state,"method_refs":["method:1"],"evidence_refs":["evidence:1"]})
            }
            "not-assessed" => {
                json!({"state":state,"reason":"assessment not attempted","method_refs":[],"evidence_refs":[]})
            }
            _ => {
                json!({"state":state,"reason":"not admitted in this environment","method_refs":[],"evidence_refs":[]})
            }
        }
    }
    fn access_all(state: &str) -> Value {
        let mut m = Map::new();
        for k in ACCESS_KINDS {
            m.insert((*k).to_owned(), access_entry(state));
        }
        Value::Object(m)
    }
    fn base_record(kind: &str) -> Value {
        let payload = match kind {
            "EpistemicCorpus" => {
                json!({"corpus_ref":"corpus:1","revision_ref":"rev:1","item_refs":["item:1"]})
            }
            "InteriorObservation" => {
                json!({"observation_ref":"obs:1","observed_quantity":"hidden-state-delta","coordinate_ref":"coord:1"})
            }
            "InteriorIntervention" => {
                json!({"intervention_ref":"int:1","operation":"write","before_ref":"state:before","after_ref":"state:after"})
            }
            "StructuralFinding" => {
                json!({"finding_ref":"finding:1","statement":"the relation holds","finding_state":"hypothesis"})
            }
            _ => {
                json!({"run_ref":"run:1","input_corpus_ref":"corpus:1","condition_ref":"cond:1","result_state_ref":"state:1"})
            }
        };
        json!({
            "schema": EPISTEMIC_RECORD_SCHEMA,
            "record_kind": kind,
            "record_ref": format!("record:{kind}-1"),
            "recorded_at": "2026-09-11T00:00:00Z",
            "provenance": {"kind":"source","actor_ref":null,"generator_ref":null,"source_refs":["source:1"],"derivation_refs":[]},
            "access": access_all("available"),
            "subject_refs": ["subject:1"],
            "model_ref": null,
            "checkpoint_ref": null,
            "method_refs": [],
            "coordinate_refs": [],
            "derivation_refs": [],
            "evidence_refs": [],
            "payload": payload
        })
    }

    #[test]
    fn valid_corpus_record_canonicalises_deterministically() {
        let input = base_record("EpistemicCorpus");
        let record = EpistemicRecord::read(&input).expect("valid record");
        // The canonical form omits absent optional fields rather than nulls.
        assert!(record.as_value().get("model_ref").is_none());
        assert!(record.as_value()["provenance"].get("actor_ref").is_none());
        // Reading the canonical form again is a fixed point.
        let canonical = record.as_value().clone();
        assert_eq!(
            EpistemicRecord::read(&canonical).unwrap().as_value(),
            &canonical
        );
        assert_eq!(record.clone().into_value(), canonical);
        assert_eq!(record.record_ref(), "record:EpistemicCorpus-1");
    }

    #[test]
    fn undeclared_or_misfielded_records_are_refused() {
        let mut v = base_record("EpistemicCorpus");
        v["schema"] = json!("other.schema/v0");
        assert!(EpistemicRecord::read(&v).is_err());
        let mut v = base_record("EpistemicCorpus");
        v["extra"] = json!(1);
        assert!(EpistemicRecord::read(&v).is_err());
        let mut v = base_record("EpistemicCorpus");
        v["record_kind"] = json!("NotAKind");
        assert!(EpistemicRecord::read(&v).is_err());
        let mut v = base_record("EpistemicCorpus");
        v["recorded_at"] = json!("2026-09-11T00:00:00+00:00");
        assert!(EpistemicRecord::read(&v).is_err());
    }

    #[test]
    fn checkpoint_requires_model() {
        let mut v = base_record("EpistemicCorpus");
        v["checkpoint_ref"] = json!("checkpoint:1");
        assert!(EpistemicRecord::read(&v).is_err());
        v["model_ref"] = json!("model:1");
        assert!(EpistemicRecord::read(&v).is_ok());
    }

    #[test]
    fn interior_records_demand_evidenced_interior_access() {
        let mut v = base_record("InteriorObservation");
        v["model_ref"] = json!("model:1");
        v["checkpoint_ref"] = json!("checkpoint:1");
        v["method_refs"] = json!(["method:obs"]);
        v["coordinate_refs"] = json!(["coord:1"]);
        v["access"] = access_all("unavailable");
        assert!(EpistemicRecord::read(&v).is_err());
        let mut ok = v.clone();
        let mut access = access_all("unavailable");
        access["internal_read"] = access_entry("available");
        ok["access"] = access;
        assert!(EpistemicRecord::read(&ok).is_ok());

        let mut w = base_record("InteriorIntervention");
        w["model_ref"] = json!("model:1");
        w["checkpoint_ref"] = json!("checkpoint:1");
        w["method_refs"] = json!(["method:write"]);
        w["coordinate_refs"] = json!(["coord:1"]);
        w["access"] = access_all("unavailable");
        assert!(EpistemicRecord::read(&w).is_err());
        let mut ok = w.clone();
        let mut access = access_all("unavailable");
        access["causal"] = access_entry("available");
        ok["access"] = access;
        assert!(EpistemicRecord::read(&ok).is_ok());
    }

    #[test]
    fn supported_findings_require_evidence() {
        let mut v = base_record("StructuralFinding");
        v["payload"]["finding_state"] = json!("supported");
        assert!(EpistemicRecord::read(&v).is_err());
        v["evidence_refs"] = json!(["evidence:1"]);
        assert!(EpistemicRecord::read(&v).is_ok());
        let mut h = base_record("StructuralFinding");
        h["payload"]["finding_state"] = json!("hypothesis");
        assert!(EpistemicRecord::read(&h).is_ok());
    }

    #[test]
    fn access_declarations_must_be_warranted() {
        let mut v = base_record("EpistemicCorpus");
        let mut access = access_all("available");
        access["behavioural"]["evidence_refs"] = json!([]);
        v["access"] = access.clone();
        assert!(EpistemicRecord::read(&v).is_err());
        access["behavioural"] = json!({"state":"unavailable","method_refs":[],"evidence_refs":[]});
        v["access"] = access.clone();
        assert!(EpistemicRecord::read(&v).is_err());
        // "not-assessed" must not claim a basis it never assessed.
        access["behavioural"] = json!({"state":"not-assessed","reason":"not attempted","method_refs":["method:1"],"evidence_refs":[]});
        v["access"] = access.clone();
        assert!(EpistemicRecord::read(&v).is_err());
        // The lawful not-assessed form: a reason and no claimed basis.
        access["behavioural"] = json!({"state":"not-assessed","reason":"not attempted","method_refs":[],"evidence_refs":[]});
        v["access"] = access;
        assert!(EpistemicRecord::read(&v).is_ok());
    }

    #[test]
    fn provenance_requires_its_attribution() {
        let cases = [
            (
                json!({"kind":"source","actor_ref":null,"generator_ref":null,"source_refs":[],"derivation_refs":[]}),
                false,
            ),
            (
                json!({"kind":"agent","actor_ref":null,"generator_ref":null,"source_refs":[],"derivation_refs":[]}),
                false,
            ),
            (
                json!({"kind":"agent","actor_ref":"agent:1","generator_ref":null,"source_refs":[],"derivation_refs":[]}),
                true,
            ),
            (
                json!({"kind":"human","actor_ref":null,"generator_ref":null,"source_refs":[],"derivation_refs":[]}),
                false,
            ),
            (
                json!({"kind":"synthetic","actor_ref":null,"generator_ref":null,"source_refs":[],"derivation_refs":[]}),
                false,
            ),
            (
                json!({"kind":"synthetic","actor_ref":null,"generator_ref":"gen:1","source_refs":[],"derivation_refs":[]}),
                true,
            ),
            (
                json!({"kind":"transformed","actor_ref":null,"generator_ref":null,"source_refs":[],"derivation_refs":[]}),
                false,
            ),
            (
                json!({"kind":"transformed","actor_ref":null,"generator_ref":null,"source_refs":["source:1"],"derivation_refs":["record:1"]}),
                true,
            ),
        ];
        for (provenance, ok) in cases {
            let mut v = base_record("EpistemicCorpus");
            v["provenance"] = provenance;
            assert_eq!(EpistemicRecord::read(&v).is_ok(), ok);
        }
    }

    #[test]
    fn payload_fields_are_exact_per_kind() {
        let mut v = base_record("EpistemicCorpus");
        v["payload"]["unexpected"] = json!(1);
        assert!(EpistemicRecord::read(&v).is_err());
        let mut v = base_record("EpistemicCorpus");
        v["payload"]["item_refs"] = json!([]);
        assert!(EpistemicRecord::read(&v).is_err());
        let mut v = base_record("EpistemicCorpus");
        v["payload"]["corpus_ref"] = json!("");
        assert!(EpistemicRecord::read(&v).is_err());
    }
}
