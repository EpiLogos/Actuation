//! Comparison/review mechanics remain outside candidate execution. A matched
//! set is not a result, and a human review packet is not human Recognition.
use crate::{
    evidence::{bytes_digest, sanitize, stable_digest},
    tasks::Task,
    world::World,
    Error, Result,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
pub const CONDITIONS: &[&str] = &["classic", "ql-direct", "ql-deep"];
pub const HELD_CONSTANTS: &[&str] = &[
    "prompt",
    "success_constraints",
    "start_state",
    "model",
    "capabilities",
    "verification",
    "budget",
    "host_revision",
    "host_composition",
    "network_policy",
    "benchmark_revision",
    "task_revision",
    "runner_revision",
    "review_contract_revision",
];
fn selected<'a>(r: &'a Value, field: &str) -> &'a Value {
    match field {
        "prompt" => &r["prompt_digest"],
        "success_constraints" => &r["success_constraints_digest"],
        "start_state" => &r["start_state_digest"],
        "model" => &r["model"],
        "capabilities" => r
            .get("capability_contract_digest")
            .filter(|v| !v.is_null())
            .unwrap_or(&r["capability_digest"]),
        "verification" => &r["verification_protocol_digest"],
        "budget" => &r["execution_budget_digest"],
        "host_revision" => r
            .get("host_revision")
            .filter(|v| !v.is_null())
            .unwrap_or(&r["host"]["revision"]),
        "host_composition" => &r["host_composition_fingerprint"],
        "network_policy" => &r["network_policy_digest"],
        _ => &r[field],
    }
}
pub fn compare_held_constant(records: &[Value]) -> Value {
    let mut result = json!({});
    let mut mismatches = vec![];
    for field in HELD_CONSTANTS {
        let distinct = records
            .iter()
            .map(|r| stable_digest(selected(r, field)))
            .collect::<BTreeSet<_>>();
        // The non-DSH ordinary host has no composition fingerprint. All other
        // absent values are missing evidence, not a successfully held constant.
        let missing = records.iter().any(|r| {
            selected(r, field).is_null()
                && (*field != "host_composition" || r["host"]["id"] == "dsh")
        });
        let held = distinct.len() == 1 && !missing && !records.is_empty();
        result[field] = json!(held);
        if !held {
            mismatches.push(
                json!({"field":field,"distinct_values":distinct.len(),"missing_evidence":missing}),
            );
        }
    }
    result["valid"] = json!(mismatches.is_empty());
    result["mismatches"] = json!(mismatches);
    result
}
/// Deterministic labels are evidence-local, not a claim to cryptographic
/// blinding. The mapping is returned separately and excluded from Pass A.
pub fn mask_mapping(manifest: &Value) -> Result<Value> {
    let records = manifest["records"]
        .as_array()
        .ok_or_else(|| Error::new("comparison records required"))?;
    let mut groups: BTreeMap<u64, Vec<&Value>> = BTreeMap::new();
    for r in records {
        let n = r["repetition"]
            .as_u64()
            .ok_or_else(|| Error::new("nonnegative repetition required"))?;
        if !CONDITIONS.contains(&r["condition"].as_str().unwrap_or("")) {
            return Err(Error::new("unrecognised comparison condition"));
        }
        groups.entry(n).or_default().push(r);
    }
    let mut mapping = json!({});
    for (repetition, mut group) in groups {
        let benchmark = manifest["benchmark_revision"]
            .as_str()
            .or(manifest["benchmark"].as_str())
            .unwrap_or("unknown");
        let seed = format!(
            "{benchmark}:{}:{}:{repetition}",
            manifest["host"]["id"].as_str().unwrap_or("unknown"),
            manifest["task"]["id"].as_str().unwrap_or("unknown")
        );
        group.sort_by_key(|r| {
            stable_digest(&json!(format!(
                "{seed}:{}",
                r["condition"].as_str().unwrap()
            )))
        });
        let mut row = json!({});
        for (i, r) in group.into_iter().enumerate() {
            let condition = r["condition"].as_str().unwrap();
            if row.get(condition).is_some() {
                return Err(Error::new("duplicate condition in one repetition"));
            }
            row[condition] = json!(format!("Candidate {}", char::from(b'A' + i as u8)));
        }
        mapping[repetition.to_string()] = row;
    }
    Ok(
        json!({"schema":"ql-series1-mask-map/0.1","benchmark_revision":manifest["benchmark_revision"],"host":manifest["host"]["id"],"task":manifest["task"]["id"],"mapping":mapping}),
    )
}
pub fn assess(manifest: &Value) -> Value {
    let Some(records) = manifest["records"].as_array() else {
        return json!({"valid":false,"reasons":["comparison records required"],"records":[]});
    };
    let held = compare_held_constant(records);
    let mut reasons = vec![];
    if manifest["schema"] != "ql-series1-run/0.3" {
        reasons.push("unsupported comparison manifest schema".to_owned());
    }
    if manifest["determination"] != "pending-human-review" {
        reasons.push("research does not supply human determination".into());
    }
    for field in HELD_CONSTANTS {
        if held[field] != true {
            reasons.push(format!("held constant missing/mismatch: {field}"));
        }
    }
    if records.is_empty() {
        reasons.push("no executed comparison records".into());
    }
    let mut completeness = vec![];
    let mut repeats: BTreeMap<u64, BTreeSet<String>> = BTreeMap::new();
    for record in records {
        let mut missing = vec![];
        for key in [
            "prompt",
            "success_conditions",
            "starting_workspace",
            "final_workspace",
            "verification",
            "outcome",
            "model",
            "host_revision",
            "benchmark_revision",
            "task_revision",
            "runner_revision",
            "review_contract_revision",
            "elapsed_ms",
            "model_calls",
            "capability_calls",
            "total_tokens",
        ] {
            let value = if key == "host_revision" {
                selected(record, key)
            } else {
                &record[key]
            };
            if value.is_null() {
                missing.push(key.to_owned());
            }
        }
        if record["fixture_provider"] == true
            || record["provider_mode"] == "fixture"
            || manifest["fixture_provider"] == true
            || manifest["provider_mode"] != "live"
        {
            missing.push("real provider evidence (fixture/unknown is not live comparison)".into());
        }
        let condition = record["condition"].as_str().unwrap_or("unknown");
        if let Some(n) = record["repetition"].as_u64() {
            let g = repeats.entry(n).or_default();
            if !CONDITIONS.contains(&condition) || !g.insert(condition.into()) {
                missing.push("unique admitted condition for repetition".into());
            }
        } else {
            missing.push("repetition".into());
        }
        if let Some(events) = record["record"]["events"].as_array() {
            for (index, event) in events.iter().enumerate() {
                if event["record_index"].as_u64() != Some(index as u64) {
                    missing.push(format!("record.events[{index}].record_index"));
                }
            }
            for kind in ["model", "capability"] {
                let requests = events
                    .iter()
                    .filter(|e| {
                        e["channel"] == "host" && e["event_type"] == format!("{kind}_requested")
                    })
                    .count();
                let returns = events
                    .iter()
                    .filter(|e| {
                        e["channel"] == "host" && e["event_type"] == format!("{kind}_returned")
                    })
                    .count();
                if requests != returns {
                    missing.push(format!("balanced {kind} request/return chronology"));
                }
            }
        } else {
            missing.push("record.events".into());
        }
        for m in &missing {
            reasons.push(format!("{condition} r{} missing {m}", record["repetition"]));
        }
        completeness.push(json!({"condition":condition,"repetition":record["repetition"],"complete":missing.is_empty(),"missing":missing}));
    }
    for (n, conditions) in repeats {
        if conditions.len() != CONDITIONS.len() {
            reasons.push(format!(
                "repetition {n} lacks a complete three-condition matched set"
            ));
        }
    }
    reasons.sort();
    reasons.dedup();
    json!({"valid":reasons.is_empty(),"reasons":reasons,"records":completeness,"held_constant":held,"human_acceptance":false})
}
pub fn fingerprint_workspace(world: &World) -> Result<Value> {
    let snapshot = world.snapshot()?;
    let files = snapshot
        .as_object()
        .unwrap()
        .iter()
        .map(|(p, v)| {
            let bytes = v.as_str().unwrap().as_bytes();
            json!({"path":p,"bytes":bytes.len(),"sha256":bytes_digest(bytes)})
        })
        .collect::<Vec<_>>();
    Ok(json!({"digest":stable_digest(&json!(files)),"files":files}))
}
pub fn freeze_task(task: &Task) -> Value {
    let mut files = task
        .start()
        .as_object()
        .unwrap()
        .iter()
        .map(|(path, value)| {
            let bytes = value.as_str().unwrap().as_bytes();
            json!({"path":path,"bytes":bytes.len(),"sha256":bytes_digest(bytes)})
        })
        .collect::<Vec<_>>();
    files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let candidate = task.candidate();
    let verifier = bytes_digest(include_bytes!("tasks.rs"));
    json!({"id":task.id(),"category":candidate["category"],"task_revision":stable_digest(&json!({"authored_task":task.revision(),"native_verifier_sha256":verifier,"human_reference_digest":stable_digest(&task.human_reference())})),"prompt_digest":stable_digest(&candidate["prompt"]),"success_constraints_digest":stable_digest(&candidate["successConditions"]),"starting_workspace_digest":stable_digest(&json!(files)),"starting_workspace_files":files,"verification_protocol_digest":stable_digest(&json!({"text":candidate["verificationProtocol"],"native_verifier_sha256":verifier})),"review_reference_digest":stable_digest(&task.human_reference())})
}
fn fence(value: &Value) -> String {
    let text = value
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| serde_json::to_string_pretty(value).unwrap());
    // A source containing fences remains quoted data rather than breaking out.
    let longest = text
        .lines()
        .filter_map(|l| {
            let n = l.chars().take_while(|c| *c == '~').count();
            (n > 0).then_some(n)
        })
        .max()
        .unwrap_or(0);
    let marker = "~".repeat(3.max(longest + 1));
    format!("{marker}\n{text}\n{marker}\n")
}
/// Pass A excludes controller identities, mappings and human reference answers;
/// it does not falsify a candidate's own ordinary outcome to hide prose clues.
pub fn render_review(manifest: &Value, masked: bool, secrets: &[String]) -> Result<String> {
    let safe = sanitize(manifest, secrets);
    let assessment = assess(&safe);
    let mapping = mask_mapping(&safe)?;
    let status = if assessment["valid"] == true {
        "eligible for human comparison; no result inferred"
    } else {
        "INVALID / INCOMPLETE: do not infer a comparison result"
    };
    let mut out=format!("# Series 1 — Pass {}\n\n{status}\n\nDetermination: pending-human-review\n\n## Exact benchmark prompt\n\n{}\n## Success constraints\n\n{}\n",if masked{"A: condition-masked"}else{"B: unmasked"},fence(&safe["review"]["prompt"]),fence(&safe["review"]["success_conditions"]));
    if !masked {
        out.push_str(&format!(
            "## Assessment\n\n{}\n## Human-only source/reference anchors\n\n{}\n",
            fence(&assessment),
            fence(&safe["review"]["human_reference"])
        ));
    }
    for r in safe["records"]
        .as_array()
        .ok_or_else(|| Error::new("comparison records required"))?
    {
        let rep = r["repetition"]
            .as_u64()
            .ok_or_else(|| Error::new("repetition required"))?
            .to_string();
        let condition = r["condition"]
            .as_str()
            .ok_or_else(|| Error::new("condition required"))?;
        let label = if masked {
            mapping["mapping"][&rep][condition]
                .as_str()
                .unwrap_or("Candidate ?")
        } else {
            condition
        };
        out.push_str(&format!("## {label} — repetition {rep}\n\n### Final outcome\n\n{}\n### Objective verification\n\n{}\n### Complete starting workspace\n\n{}\n### Complete final workspace\n\n{}\n",fence(&r["outcome"]),fence(&r["verification"]),fence(&r["starting_workspace"]),fence(&r["final_workspace"])));
        if masked {
            out.push_str("### Ordinary host chronology\n\n");
            for e in r["record"]["events"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|e| e["channel"] == "host")
            {
                match e["event_type"].as_str(){
                    Some("capability_requested")=>out.push_str(&fence(&json!({"event":"capability_requested","name":e["payload"]["name"],"args":e["payload"]["args"]}))),
                    Some("capability_returned")=>out.push_str(&fence(&json!({"event":"capability_returned","name":e["payload"]["name"],"ok":e["payload"]["ok"],"result":e["payload"]["result"],"error":e["payload"]["error"]}))),
                    Some("model_requested")=>out.push_str("Model requested. Controller payload reserved for Pass B.\n\n"),
                    Some("model_returned")=>out.push_str("Model returned. Controller envelope reserved for Pass B.\n\n"),_=>{}
                }
            }
        } else {
            out.push_str(&format!(
                "### Native/controller trace\n\n{}\n### Host-native evidence\n\n{}\n",
                fence(&r["record"]["events"]),
                fence(&r["host_native_evidence"])
            ));
        }
    }
    out.push_str("\nCounts and operator activity are not quality scores. Human interpretation and acceptance remain open.\n");
    Ok(out)
}

/// Execute comparisons serially through the public application. Each condition
/// gets a new supplied-body instance and an exclusive World. An error occupies
/// its original trial position; it never shifts attribution onto a later result.
pub fn run(request: &Value) -> Result<Value> {
    run_with(request, crate::application::run)
}
pub fn run_with<F>(request: &Value, mut execute: F) -> Result<Value>
where
    F: FnMut(&Value) -> Result<Value>,
{
    let repetitions = request["repetitions"]
        .as_u64()
        .filter(|n| (1..=32).contains(n))
        .ok_or_else(|| Error::new("comparison repetitions requires 1..32"))?;
    let condition_values = request
        .get("conditions")
        .cloned()
        .unwrap_or(json!(CONDITIONS));
    let conditions = condition_values
        .as_array()
        .filter(|a| !a.is_empty() && a.len() <= 3)
        .ok_or_else(|| {
            Error::new("comparison conditions requires 1..3 distinct native conditions")
        })?;
    let mut selected = BTreeSet::new();
    for c in conditions {
        let name = c
            .as_str()
            .filter(|s| CONDITIONS.contains(s))
            .ok_or_else(|| Error::new("comparison condition is not admitted"))?;
        if !selected.insert(name) {
            return Err(Error::new("duplicate comparison condition"));
        }
    }
    let common = &request["run"];
    if !common.is_object() || common.get("world").is_some() || common.get("stream").is_some() {
        return Err(Error::new(
            "comparison run requires common parameters, not a shared World or Stream",
        ));
    }
    let trace = request["trace_ref"]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| Error::new("comparison trace_ref required"))?;
    actuation_core::ExternalRef::new(trace)?;
    let task = Task::get(
        common["task_id"]
            .as_str()
            .ok_or_else(|| Error::new("comparison task_id required"))?,
    )?;
    let root = World::open(
        request["root"]
            .as_str()
            .ok_or_else(|| Error::new("comparison root required"))?,
    )?;
    if !root.list(".")?.is_empty() {
        return Err(Error::new("comparison requires an empty dedicated root"));
    }
    let count = repetitions as usize * conditions.len();
    let streams = request.get("streams").and_then(Value::as_array);
    if request.get("streams").is_some() && streams.is_none_or(|s| s.len() != count) {
        return Err(Error::new(
            "explicit trial streams must match the complete trial count",
        ));
    }
    // The exclusive plan is also the concurrency admission record. A second
    // caller cannot race the empty check and adopt this comparison's directories.
    let plan = json!({"schema":"actuation.comparison-plan/v1","trace_ref":trace,
        "task_id":task.id(),"task_revision":task.revision(),"conditions":conditions,
        "repetitions":repetitions,"standing":"execution plan, not evidence"});
    if !root.write("plan.json", plan.to_string().as_bytes(), true)? {
        return Err(Error::new("comparison root was concurrently reserved"));
    }
    let candidate = task.candidate();
    let source = &request["basis"];
    let secret_values = common["body"]["process"]["environment"]
        .as_object()
        .map(|env| {
            let key = regex::Regex::new("(?i)(key|token|secret|password|authorization)").unwrap();
            env.iter()
                .filter(|(k, _)| key.is_match(k))
                .filter_map(|(_, v)| v.as_str().map(str::to_owned))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut records = vec![];
    for repetition in 0..repetitions {
        for condition in conditions {
            let name = condition.as_str().expect("admitted condition");
            let ordinal = records.len();
            let trial_name = format!("trial-{ordinal:03}-{name}");
            let trial = root.create_child(&trial_name)?;
            let world = trial.create_child("world")?;
            let trial_ref = format!("{trace}:trial:{ordinal}");
            let mut input = common.clone();
            input["operation"] = json!("run");
            input["condition"] = condition.clone();
            input["trace_ref"] = json!(trial_ref);
            input["world"] = json!(world.root());
            if let Some(streams) = streams {
                input["stream"] = streams[ordinal].clone();
            }
            let begun = std::time::Instant::now();
            let output = match execute(&input) {
                Ok(v) => v,
                Err(e) => {
                    json!({"result":{"schema":"actuation.research-run/v1","runtime":condition,
                    "trace_ref":trial_ref,"status":"failed","error":e.to_string(),
                    "human_acceptance":false,"provider_evidence":"not-assessed"},"events":[],"durable_stream":false})
                }
            };
            let r = &output["result"];
            // Outcomes may fail or remain unknown; common parameters come from
            // the supplied plan, never inferred from a neighbouring successful run.
            let model = match (
                common["body"]["configuration"]["provider"].as_str(),
                common["body"]["configuration"]["model"].as_str(),
            ) {
                (Some(p), Some(m)) if !p.trim().is_empty() && !m.trim().is_empty() => {
                    json!({"provider":p,"model":m})
                }
                _ => Value::Null,
            };
            let mut record = json!({"condition":condition,"repetition":repetition,"ordinal":ordinal,
                "trace_ref":trial_ref,"status":r["status"],"error":r["error"],
                "prompt":candidate["prompt"],"success_conditions":candidate["successConditions"],
                "starting_workspace":r["workspace"]["before"],"final_workspace":r["workspace"]["after"],
                "verification":r["verification"],"outcome":r["execution"],
                "model":model,"fixture_provider":common["body"]["fixture_provider"],
                "provider_mode":if common["body"]["fixture_provider"]==true{"fixture"}else{"supplied-body"},
                "host":{"id":source["host_id"],"revision":source["host_revision"]},
                "host_revision":source["host_revision"],"host_composition_fingerprint":common["body"]["configuration"]["composition_fingerprint"],
                "benchmark_revision":source["benchmark_revision"],"task_revision":task.revision(),
                "runner_revision":source["runner_revision"],"review_contract_revision":source["review_contract_revision"],
                "prompt_digest":stable_digest(&candidate["prompt"]),
                "success_constraints_digest":stable_digest(&candidate["successConditions"]),
                "start_state_digest":stable_digest(task.start()),
                "capability_contract_digest":stable_digest(&json!(crate::execution::CAPABILITIES)),
                "verification_protocol_digest":stable_digest(&candidate["verificationProtocol"]),
                "execution_budget_digest":stable_digest(&json!({"max_steps":common["max_steps"],
                    "max_calls":common["max_calls"],"limits":common["limits"],
                    "session_timeout_ms":common["body"]["session_timeout_ms"]})),
                "network_policy_digest":source["network_policy_digest"],
                "elapsed_ms":begun.elapsed().as_millis(),"model_calls":r["model_calls"],
                "capability_calls":r["capability_calls"],"total_tokens":observed_total_tokens(r),
                "receipt_path":format!("{trial_name}/receipt.json"),"record":output,
                "human_acceptance":false,"provider_evidence":"not-assessed"});
            record = sanitize(&record, &secret_values);
            trial.write("receipt.json", record.to_string().as_bytes(), true)?;
            records.push(record);
        }
    }
    let manifest = json!({"schema":"ql-series1-run/0.3","trace_ref":trace,"task":candidate,
        "benchmark_revision":source["benchmark_revision"],"host":{"id":source["host_id"],"revision":source["host_revision"]},
        "determination":"pending-human-review","provider_mode":if common["body"]["fixture_provider"]==true{"fixture"}else{"supplied-body"},
        "fixture_provider":common["body"]["fixture_provider"],"records":records,
        "held_constants":compare_held_constant(&records),"provider_evidence":"not-assessed","human_acceptance":false});
    root.write("manifest.json", manifest.to_string().as_bytes(), true)?;
    Ok(manifest)
}
fn observed_total_tokens(run: &Value) -> Value {
    let Some(observations) = run["observations"].as_array() else {
        return Value::Null;
    };
    let mut total = 0_u64;
    let mut observed = false;
    for event in observations
        .iter()
        .filter(|o| o["event_type"] == "model_returned")
    {
        let Some(n) = event["value"]["result"]["usage"]["total_tokens"].as_u64() else {
            return Value::Null;
        };
        let Some(sum) = total.checked_add(n) else {
            return Value::Null;
        };
        total = sum;
        observed = true;
    }
    if observed {
        json!(total)
    } else {
        Value::Null
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(condition: &str, repetition: u64, model: &str) -> Value {
        json!({
            "condition": condition,
            "repetition": repetition,
            "prompt_digest": "digest(prompt)",
            "success_constraints_digest": "digest(constraints)",
            "start_state_digest": "digest(start)",
            "model": {"provider":"p","model":model},
            "capability_contract_digest": "digest(capabilities)",
            "verification_protocol_digest": "digest(verification)",
            "execution_budget_digest": "digest(budget)",
            "host_revision": "94591372975f45f7f42f35cbabbccfaf6936355e",
            "host_composition_fingerprint": Value::Null,
            "network_policy_digest": "digest(network)",
            "benchmark_revision": "bench-1",
            "task_revision": "task-1",
            "runner_revision": "runner-1",
            "review_contract_revision": "review-1",
            "host": {"id":"ordinary","revision":"94591372975f45f7f42f35cbabbccfaf6936355e"}
        })
    }

    #[test]
    fn held_constants_pass_a_matched_set_and_fail_a_mismatch() {
        let records = vec![
            record("classic", 0, "m"),
            record("ql-direct", 0, "m"),
            record("ql-deep", 0, "m"),
        ];
        let held = compare_held_constant(&records);
        assert_eq!(held["valid"], json!(true));
        for field in HELD_CONSTANTS {
            assert_eq!(held[*field], json!(true), "{field}");
        }
        let records = vec![record("classic", 0, "m-a"), record("ql-direct", 0, "m-b")];
        let held = compare_held_constant(&records);
        assert_eq!(held["valid"], json!(false));
        assert_eq!(held["model"], json!(false));
        assert_eq!(held["mismatches"].as_array().unwrap().len(), 1);
        // Empty records hold nothing.
        assert_eq!(compare_held_constant(&[])["valid"], json!(false));
    }

    #[test]
    fn mask_mapping_labels_deterministically_without_duplicates() {
        let manifest = json!({
            "benchmark_revision": "bench-1",
            "host": {"id":"ordinary"},
            "task": {"id":"S1-CODE-001"},
            "records": [
                {"condition":"classic","repetition":0},
                {"condition":"ql-direct","repetition":0},
                {"condition":"ql-deep","repetition":0},
                {"condition":"classic","repetition":1},
                {"condition":"ql-direct","repetition":1},
                {"condition":"ql-deep","repetition":1}
            ]
        });
        let a = mask_mapping(&manifest).unwrap();
        let b = mask_mapping(&manifest).unwrap();
        assert_eq!(a, b, "the mask must be deterministic");
        assert_eq!(a["schema"], json!("ql-series1-mask-map/0.1"));
        let row0 = &a["mapping"]["0"];
        let labels: Vec<&str> = ["classic", "ql-direct", "ql-deep"]
            .iter()
            .map(|c| row0[*c].as_str().unwrap())
            .collect();
        let mut sorted = labels.clone();
        sorted.sort();
        assert_eq!(sorted, vec!["Candidate A", "Candidate B", "Candidate C"]);

        let mut manifest = manifest.clone();
        manifest["records"][0]["condition"] = json!("classic");
        manifest["records"][1]["condition"] = json!("classic");
        assert!(mask_mapping(&manifest).is_err(), "duplicate condition");
        let bad = json!({"records":[{"condition":"mystery","repetition":0}]});
        assert!(mask_mapping(&bad).is_err(), "unadmitted condition");
    }

    #[test]
    fn assess_refuses_an_incomplete_manifest_and_never_accepts_for_humans() {
        let empty = json!({"schema":"ql-series1-run/0.3","determination":"pending-human-review","provider_mode":"live","fixture_provider":false,"records":[]});
        let out = assess(&empty);
        assert_eq!(out["valid"], json!(false));
        assert_eq!(out["human_acceptance"], json!(false));
        assert!(out["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r.as_str().unwrap().contains("no executed")));

        let missing_outcome = json!({
            "schema":"ql-series1-run/0.3","determination":"pending-human-review","provider_mode":"live","fixture_provider":false,
            "host":{"id":"ordinary","revision":"rev"},
            "records":[{"condition":"classic","repetition":0,"prompt":"p","success_conditions":["s"],
                "starting_workspace":{},"final_workspace":{},"verification":{},"outcome":"done",
                "model":{"provider":"p","model":"m"},"fixture_provider":false,"provider_mode":"supplied-body",
                "host_revision":"rev","benchmark_revision":"b","task_revision":"t","runner_revision":"r",
                "review_contract_revision":"rc","elapsed_ms":1,"model_calls":1,"capability_calls":0,
                "record":{"events":[]}}]
        });
        let out = assess(&missing_outcome);
        assert_eq!(out["valid"], json!(false));
        let reasons = out["reasons"].as_array().unwrap().to_vec();
        // A single record is not a three-condition matched set, and token totals
        // were never observed.
        assert!(reasons
            .iter()
            .any(|r| r.as_str().unwrap().contains("matched set")));
        assert!(reasons
            .iter()
            .any(|r| r.as_str().unwrap().contains("total_tokens")));

        // A fixture-provider comparison is never live evidence.
        let mut fixture = missing_outcome.clone();
        fixture["records"][0]["fixture_provider"] = json!(true);
        let out = assess(&fixture);
        assert!(out["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r.as_str().unwrap().contains("fixture")));
    }

    #[test]
    fn freeze_task_is_deterministic_and_separates_human_digests() {
        let task = Task::get("S1-CODE-001").unwrap();
        let a = freeze_task(&task);
        let b = freeze_task(&task);
        assert_eq!(a, b, "freezing is deterministic");
        assert_eq!(a["id"], json!("S1-CODE-001"));
        assert!(!a["task_revision"].as_str().unwrap().is_empty());
        assert!(!a["review_reference_digest"].as_str().unwrap().is_empty());
        let files = a["starting_workspace_files"].as_array().unwrap();
        let paths: Vec<&str> = files.iter().map(|f| f["path"].as_str().unwrap()).collect();
        let mut sorted = paths.clone();
        sorted.sort();
        assert_eq!(paths, sorted);
    }

    #[test]
    fn run_with_executes_each_trial_in_its_own_world_and_records_failures() {
        let root_dir = tempfile::tempdir().unwrap();
        let root = World::open(root_dir.path()).unwrap();
        let request = json!({
            "repetitions": 1,
            "trace_ref": "trace:test:comparison-1",
            "root": root.root().to_string_lossy(),
            "basis": {"host_id":"ordinary","host_revision":"rev-1","benchmark_revision":"bench-1","runner_revision":"run-1","review_contract_revision":"rc-1","network_policy_digest":"np"},
            "run": {
                "task_id": "S1-SKILL-001",
                "max_steps": 4,
                "max_calls": 4,
                "body": {"fixture_provider": true,
                    "configuration": {"provider":"fixture","model":"m","composition_fingerprint":"fp-1"},
                    "session_timeout_ms": 1000,
                    "process": {
                    "program":"/bin/true","args":[],"cwd":"/tmp","environment":{},"timeout_ms":1000,"output_limit":1000}}
            }
        });
        let mut calls = 0;
        let manifest = run_with(&request, |_input| {
            calls += 1;
            let world_path = _input["world"].as_str().unwrap();
            let world = World::open(world_path).unwrap();
            // Each trial receives its own dedicated, still-empty World below
            // its trial directory; task setup belongs to the real application.
            assert!(world.list(".").unwrap().is_empty());
            assert!(world_path.contains("trial-"));
            Ok(json!({
                "result": {"schema":"actuation.research-run/v1","status":"completed",
                    "workspace":{"before":{},"after":{}},"verification":{"objective_checks_pass":true},
                    "execution":{"status":"completed"},"model_calls":1,"capability_calls":0,
                    "observations":[{"event_type":"model_returned","value":{"result":{"usage":{"total_tokens":7}}}}]},
                "events": [],
                "durable_stream": false
            }))
        })
        .expect("comparison executes");
        assert_eq!(calls, 3, "one trial per admitted condition");
        let records = manifest["records"].as_array().unwrap();
        assert_eq!(records.len(), 3);
        for r in records {
            assert_eq!(r["status"], json!("completed"));
            assert_eq!(r["total_tokens"], json!(7));
            assert_eq!(r["fixture_provider"], json!(true));
            assert_eq!(r["human_acceptance"], json!(false));
        }
        assert_eq!(manifest["held_constants"]["valid"], json!(true));
        assert_eq!(manifest["determination"], json!("pending-human-review"));
        // The root holds the plan, the manifest and one directory per trial.
        let entries: Vec<String> = root
            .list(".")
            .unwrap()
            .iter()
            .map(|e| e["name"].as_str().unwrap().to_owned())
            .collect();
        assert!(entries.iter().any(|n| n == "plan.json"));
        assert!(entries.iter().any(|n| n == "manifest.json"));
        assert_eq!(entries.len(), 5);

        // An executor error occupies its own trial position and fails noisily.
        let root_dir2 = tempfile::tempdir().unwrap();
        let root2 = World::open(root_dir2.path()).unwrap();
        let mut request = request.clone();
        request["root"] = json!(root2.root().to_string_lossy());
        let manifest = run_with(&request, |input| {
            if input["condition"] == json!("ql-direct") {
                Err(Error::new("specimen exploded"))
            } else {
                Ok(json!({"result":{"status":"completed","workspace":{},"verification":{},"execution":{},"observations":[]},"events":[]}))
            }
        })
        .unwrap();
        let failed = manifest["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["condition"] == json!("ql-direct"))
            .unwrap();
        assert_eq!(failed["status"], json!("failed"));
        assert!(failed["error"].as_str().unwrap().contains("exploded"));
    }

    #[test]
    fn comparison_rejects_shared_roots_and_bad_plans() {
        let root_dir = tempfile::tempdir().unwrap();
        let root = World::open(root_dir.path()).unwrap();
        let base = json!({
            "repetitions": 1,
            "trace_ref": "trace:test:comparison-2",
            "root": root.root().to_string_lossy(),
            "run": {"task_id":"S1-SKILL-001","max_steps":1,"max_calls":1,
                "body":{"fixture_provider":true}}
        });
        // A shared World inside the common parameters is refused.
        let mut shared = base.clone();
        shared["run"]["world"] = json!("/tmp/shared");
        assert!(run_with(&shared, |_| Ok(json!({}))).is_err());
        // Duplicated conditions are refused.
        let mut dup = base.clone();
        dup["conditions"] = json!(["classic", "classic"]);
        assert!(run_with(&dup, |_| Ok(json!({}))).is_err());
        // Zero repetitions are refused.
        let mut zero = base.clone();
        zero["repetitions"] = json!(0);
        assert!(run_with(&zero, |_| Ok(json!({}))).is_err());
        // A non-empty comparison root is refused.
        root.write("occupied", b"x", false).unwrap();
        assert!(run_with(&base, |_| Ok(json!({}))).is_err());
    }
}
