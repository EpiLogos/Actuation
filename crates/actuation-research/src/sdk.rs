//! Persistent native SDK transport. Selection and SDK implementation remain
//! with the supplied body; Rust owns context, interpretation, bounds and evidence.
use crate::{
    evidence::candidate_boundary,
    execution::ModelBody,
    process::{ProcessSpec, RpcClient},
    Error, Result,
};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

const RESPONSE_SYSTEM: &str = "You are an execution model inside a controlled agent-loop experiment. Return exactly one JSON object and no prose outside it: {\"content\":\"assistant text\",\"capabilityCalls\":[{\"id\":\"optional\",\"name\":\"capability_name\",\"args\":{}}]}. Use capabilityCalls only when exterior work is needed. If no capability is needed, return an empty array.";
fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v[key]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| Error::new(format!("SDK {key} requires text")))
}
/// The same projection is used for Pi, Pydantic and DSH. No wrapper chooses a
/// different task, capabilities, control policy or response interpretation.
pub fn project(request: &Value) -> Result<Value> {
    candidate_boundary(request)?;
    let payload = &request["payload"];
    if let Some(control) = payload.get("series1Control") {
        return Ok(
            json!({"system":text(control,"system")?,"prompt":text(control,"prompt")?,
            "purpose":text(control,"purpose")?,"temperature":0}),
        );
    }
    let task = &request["request"];
    let mut prompt = json!({"task":task["input"],"success_conditions":task["successConditions"],
        "capabilities":[
            {"name":"list_files","args":{"path":"optional relative directory"}},
            {"name":"read_file","args":{"path":"relative file"}},
            {"name":"write_file","args":{"path":"relative file","content":"UTF-8 text"}},
            {"name":"run_tests","args":{"files":"optional relative test files"}}
        ],"available_capabilities":request["capabilities"]});
    let mut system = RESPONSE_SYSTEM.to_owned();
    let purpose = if let Some(act) = payload.get("ql_act").or_else(|| payload.get("qlAct")) {
        prompt["ql_act"] = act.clone();
        system.push_str(" Answer only the stated intent. A model-carried act does not itself execute a capability; return the needed difference rather than pretending a tool ran.");
        "ql-act"
    } else {
        prompt["history"] = payload.get("history").cloned().unwrap_or(json!([]));
        "classic-turn"
    };
    Ok(json!({"system":system,"prompt":prompt.to_string(),"purpose":purpose,"temperature":0}))
}
/// Preserve the inherited envelope extraction, while keeping the exact SDK
/// output and decode mode attributable. A parse failure is never an empty act.
pub fn normalize(data: &Value, control: bool) -> Result<Value> {
    let output = data["output"]
        .as_str()
        .ok_or_else(|| Error::new("SDK output requires text"))?;
    let (object, decoding) = match serde_json::from_str::<Value>(output.trim()) {
        Ok(v) if v.is_object() => (v, "json-object"),
        _ => {
            let start = output
                .find('{')
                .ok_or_else(|| Error::new("model returned no JSON object"))?;
            let end = output
                .rfind('}')
                .filter(|i| *i > start)
                .ok_or_else(|| Error::new("model returned incomplete JSON"))?;
            let v: Value = serde_json::from_str(&output[start..=end])
                .map_err(|_| Error::new("model returned invalid JSON"))?;
            if !v.is_object() {
                return Err(Error::new("model envelope requires an object"));
            }
            (v, "embedded-json-object")
        }
    };
    let mut result = json!({"content":"","capabilityCalls":[],"usage":data["usage"],
        "raw":{"sdk":data["raw"],"output":output,"decoding":decoding}});
    if control {
        result["control"] = object;
    } else {
        if let Some(content) = object.get("content") {
            if !content.is_string() {
                return Err(Error::new("model content requires text"));
            }
            result["content"] = content.clone();
        }
        let calls = match object.get("capabilityCalls") {
            None => vec![],
            Some(Value::Array(a)) if a.len() <= 10000 => a.clone(),
            _ => return Err(Error::new("model capabilityCalls requires a bounded array")),
        };
        let mut seen = std::collections::BTreeSet::new();
        let mut normalized = vec![];
        for (n, call) in calls.iter().enumerate() {
            let name = text(call, "name")?;
            let args = call.get("args").cloned().unwrap_or(json!({}));
            if !args.is_object() {
                return Err(Error::new("capability arguments require an object"));
            }
            let id = match call.get("id") {
                None => format!("call-{}", n + 1),
                Some(Value::String(s)) if !s.trim().is_empty() => s.clone(),
                _ => return Err(Error::new("capability id requires nonempty text")),
            };
            if !seen.insert(id.clone()) {
                return Err(Error::new("duplicate capability call identity"));
            }
            normalized.push(json!({"id":id,"name":name,"args":args}));
        }
        result["capabilityCalls"] = json!(normalized);
    }
    Ok(result)
}

pub struct NativeSdkBody {
    spec: ProcessSpec,
    configuration: Value,
    source_basis: Value,
    fixture: bool,
    budget: Duration,
    started: Option<Instant>,
    client: Option<RpcClient>,
    preflight: Value,
    failures: Vec<Value>,
    finished: Option<Value>,
    protocol: &'static str,
    /// An agent body acting in its own loop (`turn`), rather than answering
    /// Actuation's turns with capability calls (`complete`).
    acting: bool,
}
/// The protocol of an agent body: an agent — a session's own, as its owner
/// supplies it — speaking the SDK JSONL protocol plus `turn`. Under the
/// classic condition it acts in its World in its own loop, one `turn` at a
/// time; under a QL condition it answers Actuation's turns (`complete`), as
/// every SDK body does, so the condition is how its loop recurs.
pub const AGENT_PROTOCOL: &str = "agent-jsonl";
pub const SDK_PROTOCOL: &str = "sdk-jsonl";
impl NativeSdkBody {
    pub fn new(
        spec: ProcessSpec,
        configuration: Value,
        source_basis: Value,
        fixture: bool,
        session_timeout_ms: u64,
    ) -> Result<Self> {
        spec.validate()?;
        text(&configuration, "provider")?;
        text(&configuration, "model")?;
        if !(1..=3_600_000).contains(&session_timeout_ms) {
            return Err(Error::new("SDK session_timeout_ms requires 1..3600000"));
        }
        if let Some(level) = configuration.get("level") {
            read_level(level)?;
        }
        Ok(Self {
            spec,
            configuration,
            source_basis,
            fixture,
            budget: Duration::from_millis(session_timeout_ms),
            started: None,
            client: None,
            preflight: Value::Null,
            failures: vec![],
            finished: None,
            protocol: SDK_PROTOCOL,
            acting: false,
        })
    }
    /// An agent body: the configuration names the agent (`agent`), and the
    /// body must read it back at preflight, with its provider, model and any
    /// level it was given. `acting` is the classic condition's own loop.
    pub fn agent(
        spec: ProcessSpec,
        configuration: Value,
        source_basis: Value,
        fixture: bool,
        session_timeout_ms: u64,
        acting: bool,
    ) -> Result<Self> {
        text(&configuration, "agent")?;
        let mut body = Self::new(
            spec,
            configuration,
            source_basis,
            fixture,
            session_timeout_ms,
        )?;
        body.protocol = AGENT_PROTOCOL;
        body.acting = acting;
        Ok(body)
    }
    fn remaining(&self) -> Result<Duration> {
        self.budget
            .checked_sub(self.started.map_or(Duration::ZERO, |s| s.elapsed()))
            .filter(|d| !d.is_zero())
            .ok_or_else(|| Error::new("SDK session budget exhausted"))
    }
    fn exchange(&mut self, command: Value) -> Result<Value> {
        let time = self.remaining()?;
        let expected = command["type"].clone();
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| Error::new("SDK has not started"))?;
        let reply = client.exchange(command, time)?;
        if reply["command"] != expected {
            return Err(client
                .finish_after_error(Error::new("SDK reply requires exact operation correlation")));
        }
        if reply["success"] != true {
            return Err(Error::new("SDK refused operation; response retained"));
        }
        if !reply["data"].is_object() {
            return Err(Error::new("SDK data requires an object"));
        }
        Ok(reply["data"].clone())
    }
    fn prepare(&mut self) -> Result<()> {
        if self.preflight["ready"] == true {
            return Ok(());
        }
        if self.started.is_some() {
            return Err(Error::new(
                "SDK preflight already failed; no implicit retry",
            ));
        }
        self.started = Some(Instant::now());
        self.client = Some(RpcClient::start(self.spec.clone())?);
        self.preflight =
            self.exchange(json!({"type":"preflight","configuration":self.configuration}))?;
        if self.preflight["ready"] != true {
            return Err(Error::new("SDK readiness is not established"));
        }
        for field in ["provider", "model"] {
            if self.preflight[field] != self.configuration[field] {
                self.preflight["ready"] = json!(false);
                return Err(Error::new(
                    "SDK preflight changed the supplied provider or model",
                ));
            }
        }
        // The agent, and a level a comparison varies, are the body's to apply:
        // it reads them back, or the run does not claim them.
        for field in ["agent", "level"] {
            let required = field == "level" || self.protocol == AGENT_PROTOCOL;
            if required
                && self.configuration.get(field).is_some()
                && self.preflight[field] != self.configuration[field]
            {
                self.preflight["ready"] = json!(false);
                return Err(Error::new(format!(
                    "the body did not read back the {field} it was given: {}",
                    self.configuration[field]
                )));
            }
        }
        Ok(())
    }
    /// One turn of an acting agent: the task the first time, then what the
    /// run tells it when it finished a turn before its stop condition held.
    fn turn(&mut self, request: &Value) -> Result<Value> {
        candidate_boundary(request)?;
        let iteration = request["payload"]["iteration"].as_u64().unwrap_or(1);
        let prompt = if iteration <= 1 {
            let task = &request["request"];
            let mut text = task["input"]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| task["input"].to_string());
            let conditions = task["successConditions"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>();
            if !conditions.is_empty() {
                text.push_str("\n\nSuccess conditions:");
                for c in conditions {
                    text.push_str("\n- ");
                    text.push_str(c);
                }
            }
            text
        } else {
            request["payload"]["history"]
                .as_array()
                .and_then(|h| h.iter().rev().find(|m| m["role"] == "user"))
                .map(|m| {
                    m["content"]
                        .as_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| m["content"].to_string())
                })
                .unwrap_or_else(|| "Continue the task.".to_owned())
        };
        self.prepare()?;
        let data =
            self.exchange(json!({"type":"turn","turn":{"iteration":iteration,"prompt":prompt}}))?;
        let status = data["status"].as_str().unwrap_or("unknown");
        if status != "completed" {
            return Err(Error::new(format!(
                "the agent's turn ended {status}, not completed"
            )));
        }
        let output = data["output"]
            .as_str()
            .ok_or_else(|| Error::new("the agent's turn output requires text"))?;
        // The agent acted itself; there is nothing for the run to execute.
        Ok(
            json!({"content":output,"capabilityCalls":[],"usage":data["usage"],
            "raw":{"agent":data["raw"],"output":output,"decoding":"agent-turn"}}),
        )
    }
}

/// A comparison's level, as a body is given it: `{variable, value}`, the
/// variable one Actuation names and the value text of 1..64 characters.
pub fn read_level(level: &Value) -> Result<(&str, &str)> {
    let variable = level["variable"]
        .as_str()
        .filter(|v| crate::execution::VARIABLES.contains(v))
        .ok_or_else(|| Error::new("a level names one of the comparison variables"))?;
    let value = level["value"]
        .as_str()
        .filter(|v| !v.trim().is_empty() && v.chars().count() <= 64)
        .ok_or_else(|| Error::new("a level's value is text of 1..64 characters"))?;
    if level.as_object().is_some_and(|m| m.len() != 2) {
        return Err(Error::new("a level is exactly {variable, value}"));
    }
    Ok((variable, value))
}
impl ModelBody for NativeSdkBody {
    fn basis(&self) -> Value {
        // Configuration may contain additional native options; this record
        // exposes selection, not credentials or an assertion of provider use.
        let mut basis = json!({"protocol":self.protocol,"source":self.source_basis,"fixture_provider":self.fixture,
            "provider":self.configuration["provider"],"model":self.configuration["model"],
            "provider_mode":if self.fixture{"fixture"}else{"supplied-body"},"preflight":self.preflight});
        for field in ["agent", "level"] {
            if let Some(v) = self.configuration.get(field) {
                basis[field] = v.clone();
            }
        }
        if self.protocol == AGENT_PROTOCOL {
            basis["acting"] = json!(self.acting);
        }
        basis
    }
    fn complete(&mut self, request: &Value) -> Result<Value> {
        if self.finished.is_some() {
            return Err(Error::new("SDK session already finalized"));
        }
        let payload = &request["payload"];
        if self.acting
            && payload.get("series1Control").is_none()
            && payload.get("qlAct").is_none()
            && payload.get("ql_act").is_none()
        {
            let result = self.turn(request);
            if let Err(e) = &result {
                self.failures
                    .push(json!({"operation":"turn","error":e.to_string()}));
            }
            return result;
        }
        let result = (|| {
            let projected = project(request)?;
            self.prepare()?;
            let data = self.exchange(json!({"type":"complete","request":projected}))?;
            normalize(&data, request["payload"].get("series1Control").is_some())
        })();
        if let Err(e) = &result {
            self.failures.push(json!({"error":e.to_string()}));
        }
        result
    }
    fn finish(&mut self, inspection: &Value) -> Value {
        if let Some(done) = &self.finished {
            return done.clone();
        }
        let (status, finalization) = if self.client.is_none() {
            ("not-started", Value::Null)
        } else {
            match self.exchange(json!({"type":"finalize","inspection":inspection})) {
                Ok(v) => ("completed", v),
                Err(e) => {
                    self.failures
                        .push(json!({"operation":"finalize","error":e.to_string()}));
                    ("failed", json!({"error":e.to_string()}))
                }
            }
        };
        let (records, retirement, retirement_error) = if let Some(client) = &mut self.client {
            let finish = client.finish();
            let error = finish.err();
            let facts = error.as_ref().and_then(crate::process::failure_details);
            let retirement = match facts {
                Some(facts) => facts,
                None => json!({"observation":client.retirement_observation()}),
            };
            (json!(client.records()), retirement, error)
        } else {
            (json!([]), Value::Null, None)
        };
        if let Some(error) = &retirement_error {
            self.failures
                .push(json!({"operation":"native_finish","error":error.to_string()}));
        }
        // A completed semantic reply remains data even when physical retirement
        // fails. The top-level status must not acknowledge clean completion.
        let receipt = json!({"schema":"actuation.sdk-session/v1",
            "status":if retirement_error.is_some(){"failed"}else{status},
            "finalization_status":status,"process_retirement":retirement,
            "preflight":self.preflight,"responses":records,"failures":self.failures,
            "finalization":finalization,"provider_evidence":"not-assessed","human_acceptance":false});
        self.finished = Some(receipt.clone());
        receipt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classic_request() -> Value {
        json!({
            "request": {"input":"fix the index","successConditions":["tests pass"]},
            "payload": {},
            "capabilities": ["list_files","read_file","write_file","run_tests"]
        })
    }

    #[test]
    fn project_builds_the_shared_classic_prompt() {
        let out = project(&classic_request()).expect("projection");
        assert!(out["system"]
            .as_str()
            .unwrap()
            .starts_with("You are an execution model"));
        let prompt: Value =
            serde_json::from_str(out["prompt"].as_str().unwrap()).expect("prompt is JSON text");
        assert_eq!(prompt["task"], json!("fix the index"));
        assert_eq!(prompt["success_conditions"], json!(["tests pass"]));
        assert_eq!(prompt["capabilities"].as_array().unwrap().len(), 4);
        assert_eq!(out["purpose"], json!("classic-turn"));
        assert_eq!(out["temperature"], json!(0));
    }

    #[test]
    fn project_honours_control_and_ql_act_payloads() {
        let mut request = classic_request();
        request["payload"]["series1Control"] =
            json!({"system":"be exact","prompt":"choose an act","purpose":"ql-next-act"});
        let out = project(&request).unwrap();
        assert_eq!(out["purpose"], json!("ql-next-act"));
        assert_eq!(out["system"], json!("be exact"));
        assert_eq!(out["prompt"], json!("choose an act"));

        let mut request = classic_request();
        request["payload"]["ql_act"] = json!({"operator":"apply"});
        let out = project(&request).unwrap();
        assert_eq!(out["purpose"], json!("ql-act"));
        let prompt: Value = serde_json::from_str(out["prompt"].as_str().unwrap()).unwrap();
        assert_eq!(prompt["ql_act"], json!({"operator":"apply"}));
    }

    #[test]
    fn project_refuses_review_only_material() {
        let mut request = classic_request();
        request["expected_answer"] = json!("leaked");
        assert!(project(&request).is_err());
    }

    #[test]
    fn normalize_decides_the_decoding_and_normalizes_calls() {
        let plain = json!({"output":"{\"content\":\"done\",\"capabilityCalls\":[]}"});
        let out = normalize(&plain, false).unwrap();
        assert_eq!(out["content"], json!("done"));
        assert_eq!(out["raw"]["decoding"], json!("json-object"));

        let wrapped = json!({"output":"Here you are: {\"content\":\"ok\",\"capabilityCalls\":[{\"name\":\"write_file\",\"args\":{\"path\":\"a.md\"}}]} as requested"});
        let out = normalize(&wrapped, false).unwrap();
        assert_eq!(out["raw"]["decoding"], json!("embedded-json-object"));
        assert_eq!(out["capabilityCalls"][0]["id"], json!("call-1"));
        assert_eq!(out["capabilityCalls"][0]["name"], json!("write_file"));
        assert_eq!(out["usage"], Value::Null);

        let control = json!({"output":"{\"destination\":\"P4\"}","raw":"ctx"});
        let out = normalize(&control, true).unwrap();
        assert_eq!(out["control"]["destination"], json!("P4"));
        assert_eq!(out["raw"]["decoding"], json!("json-object"));
    }

    #[test]
    fn normalize_refuses_degenerate_envelopes() {
        assert!(normalize(&json!({}), false).is_err());
        assert!(normalize(&json!({"output":"no object here"}), false).is_err());
        assert!(normalize(&json!({"output":"{broken"}), false).is_err());
        assert!(normalize(&json!({"output":"{\"content\":42}"}), false).is_err());
        assert!(normalize(&json!({"output":"{\"capabilityCalls\":\"all\"}"}), false).is_err());
        assert!(normalize(
            &json!({"output":"{\"capabilityCalls\":[{\"name\":\"x\",\"args\":\"bad\"}]}"}),
            false
        )
        .is_err());
        assert!(normalize(
            &json!({"output":"{\"capabilityCalls\":[{\"id\":\"dup\"},{\"id\":\"dup\"}]}"}),
            false
        )
        .is_err());
        assert!(
            normalize(&json!("{\"content\":\"ok\"} trailing"), false).is_err(),
            "non-object root must be refused"
        );
    }

    #[test]
    fn sdk_body_bounds_are_checked_before_any_process_starts() {
        let spec = ProcessSpec {
            program: "/bin/true".into(),
            args: vec![],
            cwd: std::env::temp_dir(),
            environment: Default::default(),
            timeout_ms: 1_000,
            output_limit: 1 << 20,
        };
        let cfg = json!({"provider":"fixture","model":"m"});
        assert!(NativeSdkBody::new(spec.clone(), cfg.clone(), json!({}), true, 0).is_err());
        assert!(NativeSdkBody::new(spec.clone(), cfg.clone(), json!({}), true, 3_600_001).is_err());
        assert!(NativeSdkBody::new(
            spec.clone(),
            json!({"provider":"","model":"m"}),
            json!({}),
            true,
            1_000
        )
        .is_err());
        let mut relative = spec.clone();
        relative.program = "true".into();
        assert!(NativeSdkBody::new(relative, cfg.clone(), json!({}), true, 1_000).is_err());
        assert!(NativeSdkBody::new(spec, cfg, json!({}), true, 1_000).is_ok());
    }
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
mod native_finish_tests {
    use super::*;
    use crate::process::{
        native_retirement_tests::{external_reap, Fixture},
        retirement_checkpoint,
    };

    fn body(fixture: &Fixture) -> NativeSdkBody {
        let spec = fixture.script(
            r#"#!/bin/sh
read -r request
printf '%s\n' '{"type":"response","id":"actuation-prime-1","command":"preflight","success":true,"data":{"ready":true,"provider":"actual-test-specimen","model":"actual-test-specimen"}}'
read -r request
printf '%s\n' '{"type":"response","id":"actuation-prime-2","command":"complete","success":true,"data":{"output":"{\"content\":\"actual-native-completion\",\"capabilityCalls\":[]}"}}'
read -r request
printf '%s\n' '{"type":"response","id":"actuation-prime-3","command":"finalize","success":true,"data":{"retained_semantic_data":"PRIVATE-FINALIZATION-CANARY"}}'
exit 0
"#,
            1000, 4096,
        );
        let mut body = NativeSdkBody::new(
            spec,
            json!({"provider":"actual-test-specimen","model":"actual-test-specimen"}),
            json!({"source":"owned-native-fixture"}),
            true,
            2000,
        )
        .unwrap();
        let completion = body.complete(&json!({
            "request":{"input":"actual controlled native completion","successConditions":["reply observed"]},
            "payload":{},"capabilities":["read_file"]
        })).unwrap();
        assert_eq!(completion["content"], "actual-native-completion");
        assert_eq!(body.preflight["ready"], true);
        body
    }

    #[test]
    fn sdk_completed_native_finalize_keeps_data_and_known_retirement() {
        let fixture = Fixture::new();
        let mut body = body(&fixture);
        let receipt = body.finish(&json!({"observed":"actual-owned-fixture"}));
        assert_eq!(receipt["status"], "completed");
        assert_eq!(receipt["finalization_status"], "completed");
        assert_eq!(
            receipt["finalization"]["retained_semantic_data"],
            "PRIVATE-FINALIZATION-CANARY"
        );
        assert_eq!(
            receipt["process_retirement"]["observation"]["direct_child_reaped"],
            true
        );
        assert_eq!(
            receipt["process_retirement"]["observation"]["reply_observed"],
            true
        );
        assert_eq!(body.finish(&json!({"different":"not-replayed"})), receipt);
        assert_eq!(receipt["responses"].as_array().unwrap().len(), 3);
        fixture.record(json!({"status":receipt["status"],
            "observation":receipt["process_retirement"]["observation"]}));
        drop(body);
        fixture.dispose_after_known_retirement();
    }

    #[test]
    fn sdk_completed_native_finalize_does_not_acknowledge_lost_retirement() {
        let fixture = Fixture::new();
        let mut body = body(&fixture);
        retirement_checkpoint(external_reap);
        let receipt = body.finish(&json!({"observed":"actual-owned-fixture"}));
        assert_eq!(receipt["status"], "failed");
        assert_eq!(receipt["finalization_status"], "completed");
        assert_eq!(
            receipt["finalization"]["retained_semantic_data"],
            "PRIVATE-FINALIZATION-CANARY"
        );
        let facts = &receipt["process_retirement"];
        assert_eq!(facts["observation"]["reply_observed"], true);
        assert_eq!(facts["observation"]["signal_forbidden"], true);
        assert_eq!(facts["observation"]["direct_child_reaped"], false);
        assert_eq!(facts["observation"]["kill_signal_attempted"], false);
        assert!(facts["secondary"]
            .as_array()
            .unwrap()
            .iter()
            .any(|cause| cause["io"]["raw_os_error"] == rustix::io::Errno::CHILD.raw_os_error()));
        assert!(!facts.to_string().contains("PRIVATE"));
        assert_eq!(receipt["responses"].as_array().unwrap().len(), 3);
        assert_eq!(body.finish(&json!({"different":"no-replay"})), receipt);
        fixture.record(facts.clone());
        drop(body);
        // Unknown native owner retirement retains the already-created fixture.
    }
}
