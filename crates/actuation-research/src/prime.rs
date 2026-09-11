use crate::{value::*, Error, Result};
use serde_json::{json, Value};
const CONDITIONS: &str = include_str!("../../../experiments/ql-runtime/prime/conditions.json");
/// The provenance envelope keeps the JSON derivation of conditions.mjs explicit;
/// lookups and the capabilities projection see only the condition map itself.
pub fn conditions() -> Value {
    let v: Value = serde_json::from_str(CONDITIONS).expect("checked condition catalogue");
    if v["schema"] != "actuation.prime-conditions/v1" || !v["conditions"].is_object() {
        panic!("condition catalogue lost its schema or conditions object");
    }
    v["conditions"].clone()
}
pub fn get_condition(id: &str) -> Result<Value> {
    conditions()
        .get(id)
        .cloned()
        .ok_or_else(|| Error::new(format!("unknown Prime condition {id}")))
}
pub fn condition_prompt(c: &Value, task: &Value) -> Result<String> {
    let mut lines=vec![format!("You are the root Agency for Actuation experiment {}.",text(&c["code"],"condition code")?),"Work directly on the supplied task and leave the workspace in the requested state.".into(),"Prime child agents are differentiated acting loci, not role-play labels. Create them only when the task genuinely benefits from differentiated work.".into()];
    if c["relational"] == true {
        lines.extend([
        "A Python-backed ql_relational faculty is available to you and is inherited by Prime child agents.",
        "Use QL/MEF/Wiki operations where they disclose an operationally useful relation. Do not decorate prose with QL names in place of actual operations.",
        "When you create a child, tell it to situate its bounded task in relation to the parent task, use the same relational faculty where useful, preserve source/evidence provenance, and return material difference rather than a generic summary."].map(str::to_owned));
    }
    if c["returnContract"] == true {
        lines.extend([
        "When a child return materially affects the parent determination, preserve it using the Actuation Prime Return envelope. The ql_relational.return_envelope(...) helper can construct it.",
        "Reconstitution must retain returned difference and unresolved relations before synthesis; do not flatten conflicting child returns merely to reach agreement."].map(str::to_owned));
    }
    if c["code"] == "P4" {
        lines.push("Descendant recursion is part of this condition: if a child discovers a genuinely subordinate unresolved relation and the current Prime runtime admits another depth, it may create a child of its own. Do not manufacture grandchildren to satisfy the experiment.".into());
    }
    if c["continual"] == true {
        lines.push("This run is authorised for one explicit Continual Harness refinement after task execution. Do not call /refine yourself; the experiment driver performs and records the single refinement from the completed trajectory.".into());
    }
    lines.extend([
        "".into(),
        "TASK".into(),
        text(&task["prompt"], "task.prompt")?,
        "".into(),
        "SUCCESS CONDITIONS".into(),
    ]);
    let success = task["successConditions"]
        .as_array()
        .ok_or_else(|| Error::new("task successConditions requires an array"))?;
    for s in success {
        lines.push(format!("- {}", text(s, "success condition")?));
    }
    Ok(lines.join("\n"))
}
pub fn full_revision(v: &Value, name: &str) -> Result<String> {
    let s = text(v, name)?;
    if s.len() != 40
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(Error::new(format!(
            "{name} requires full lowercase Git revision"
        )));
    }
    Ok(s)
}
#[derive(Clone, Debug)]
pub struct SourceLock(Value);
impl SourceLock {
    pub fn read(v: &Value) -> Result<Self> {
        if v["schema"] != "actuation.prime-source-lock/v1" {
            return Err(Error::new("wrong Prime source-lock schema"));
        }
        for (value, name) in [
            (&v["actuation"]["base_revision"], "actuation.base_revision"),
            (
                &v["prime_agent"]["release_revision"],
                "prime_agent.release_revision",
            ),
            (
                &v["prime_agent"]["observed_main_revision"],
                "prime_agent.observed_main_revision",
            ),
            (
                &v["ql_mef"]["accepted_main_revision"],
                "ql_mef.accepted_main_revision",
            ),
            (
                &v["ql_mef"]["harmonic_research"]["revision"],
                "harmonic.revision",
            ),
            (
                &v["ql_mef"]["harmonic_research"]["accepted_via_revision"],
                "harmonic.accepted_via_revision",
            ),
        ] {
            full_revision(value, name)?;
        }
        let tag = v["prime_agent"]["release"].as_str().unwrap_or("");
        if !regex::Regex::new(r"^v\d+\.\d+\.\d+$")
            .unwrap()
            .is_match(tag)
        {
            return Err(Error::new("Prime requires exact semantic release tag"));
        }
        if v["ql_mef"]["harmonic_research"]["standing"] != "accepted-main-history-carrier"
            || v["ql_mef"]["harmonic_research"]["pull_request_state"] != "closed-unmerged"
        {
            return Err(Error::new(
                "harmonic historical provenance must remain explicit",
            ));
        }
        Ok(Self(v.clone()))
    }
    pub fn as_value(&self) -> &Value {
        &self.0
    }
    pub fn classify(&self, revision: &str, harmonic: bool, dirty: bool) -> Result<&'static str> {
        full_revision(&json!(revision), "observed QL revision")?;
        Ok(if dirty {
            "explicit-drift"
        } else if self.0["ql_mef"]["accepted_main_revision"] == revision {
            if harmonic {
                "accepted-main-harmonic"
            } else {
                "accepted-main"
            }
        } else if self.0["ql_mef"]["harmonic_research"]["revision"] == revision {
            "historical-harmonic-head"
        } else {
            "explicit-drift"
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCK_FIXTURE: &str =
        include_str!("../../../experiments/native-research/prime-source-lock.json");

    #[test]
    fn condition_catalogue_matches_the_frozen_javascript_source() {
        let conditions = conditions();
        let expected = [
            ("prime-native", "P0", false, 1, false, false, false),
            ("prime-relational", "P2", true, 1, false, true, false),
            ("prime-relational-return", "P3", true, 1, true, true, false),
            ("prime-recursive-field", "P4", true, 2, true, true, false),
            ("prime-continual", "P5", true, 2, true, true, true),
        ];
        let mjs = include_str!("../../../experiments/ql-runtime/prime/conditions.mjs");
        for (id, code, relational, max_depth, return_contract, recursive, continual) in expected {
            let c = get_condition(id).expect("catalogued condition");
            assert_eq!(c["code"], json!(code), "{id}");
            assert_eq!(c["relational"], json!(relational), "{id}");
            assert_eq!(c["maxDepth"], json!(max_depth), "{id}");
            assert_eq!(c["returnContract"], json!(return_contract), "{id}");
            assert_eq!(c["recursive"], json!(recursive), "{id}");
            assert_eq!(c["continual"], json!(continual), "{id}");
            // Every condition name in the frozen JavaScript source is present.
            assert!(
                mjs.contains(&format!("'{id}'")),
                "{id} missing from mjs source"
            );
        }
        assert_eq!(
            conditions.as_object().unwrap().len(),
            expected.len(),
            "catalogue must not carry conditions the JavaScript source does not"
        );
        assert!(get_condition("prime-absent").is_err());
    }

    #[test]
    fn condition_prompt_composes_the_shared_contract() {
        let task =
            json!({"prompt":"fix the index","successConditions":["tests pass","exports kept"]});
        let prompt = condition_prompt(&get_condition("prime-native").unwrap(), &task).unwrap();
        assert!(prompt.contains("experiment P0"));
        assert!(prompt.contains("TASK\nfix the index"));
        assert!(prompt.contains("SUCCESS CONDITIONS"));
        assert!(prompt.contains("- tests pass"));
        assert!(!prompt.contains("relational faculty"));

        let relational =
            condition_prompt(&get_condition("prime-relational").unwrap(), &task).unwrap();
        assert!(relational.contains("ql_relational faculty is available"));
        assert!(!relational.contains("Prime Return envelope"));

        let return_contract =
            condition_prompt(&get_condition("prime-relational-return").unwrap(), &task).unwrap();
        assert!(return_contract.contains("Prime Return envelope"));

        let deep =
            condition_prompt(&get_condition("prime-recursive-field").unwrap(), &task).unwrap();
        assert!(deep.contains("Descendant recursion is part of this condition"));

        let continual =
            condition_prompt(&get_condition("prime-continual").unwrap(), &task).unwrap();
        assert!(continual.contains("one explicit Continual Harness refinement"));

        let bad_task = json!({"prompt":"x","successConditions":"not-an-array"});
        assert!(condition_prompt(&get_condition("prime-native").unwrap(), &bad_task).is_err());
    }

    #[test]
    fn full_revision_accepts_only_full_lowercase_hex() {
        let good = "e753efc91f62b5b2af09e0a852c5063e366eccbe";
        assert_eq!(full_revision(&json!(good), "r").unwrap(), good);
        for bad in [
            "",
            "e753EFC91F62B5B2AF09E0A852C5063E366ECCBE",
            "e753efc91f62b5b2af09e0a852c5063e366eccb",
            "e753efc91f62b5b2af09e0a852c5063e366eccbee",
            "g753efc91f62b5b2af09e0a852c5063e366eccbe",
            "20260911",
        ] {
            assert!(full_revision(&json!(bad), "r").is_err());
        }
        assert!(full_revision(&json!(42), "r").is_err());
    }

    fn valid_lock() -> Value {
        serde_json::from_str(LOCK_FIXTURE).expect("fixture parses")
    }

    #[test]
    fn source_lock_fixture_is_admitted_and_classifies_as_authored() {
        let lock = SourceLock::read(&valid_lock()).expect("fixture is a valid lock");
        assert_eq!(
            lock.classify("e753efc91f62b5b2af09e0a852c5063e366eccbe", false, false)
                .unwrap(),
            "accepted-main"
        );
        assert_eq!(
            lock.classify("e753efc91f62b5b2af09e0a852c5063e366eccbe", true, false)
                .unwrap(),
            "accepted-main-harmonic"
        );
        assert_eq!(
            lock.classify("42d36ed75fd9cf8a70bcbabc5dca766cc51b6811", true, false)
                .unwrap(),
            "historical-harmonic-head"
        );
        assert_eq!(
            lock.classify("94591372975f45f7f42f35cbabbccfaf6936355e", false, false)
                .unwrap(),
            "explicit-drift"
        );
        assert_eq!(
            lock.classify("e753efc91f62b5b2af09e0a852c5063e366eccbe", false, true)
                .unwrap(),
            "explicit-drift"
        );
    }

    #[test]
    fn source_lock_requires_explicit_provenance_fields() {
        let cases = [
            |v: &mut Value| v["schema"] = json!("actuation.prime-source-lock/v0"),
            |v: &mut Value| v["prime_agent"]["release"] = json!("0.9.4"),
            |v: &mut Value| v["ql_mef"]["harmonic_research"]["standing"] = json!("something-else"),
            |v: &mut Value| {
                v["ql_mef"]["harmonic_research"]["pull_request_state"] = json!("merged")
            },
            |v: &mut Value| v["actuation"]["base_revision"] = json!("short"),
        ];
        for change in cases {
            let mut v = valid_lock();
            change(&mut v);
            assert!(SourceLock::read(&v).is_err());
        }
        assert!(SourceLock::read(&valid_lock()).is_ok());
    }
}
