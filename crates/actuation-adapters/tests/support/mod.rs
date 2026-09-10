#![allow(dead_code)]
// Temporary conformance transport, not a product operation or test runtime.
// The implementation under test is the ordinary native library below.
use actuation_adapters::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
#[path = "../../../actuation-stream/tests/support/mod.rs"]
pub mod stream;
fn admitted<T: DeserializeOwned + Serialize>(v: Value) -> Result<Value> {
    Ok(serde_json::to_value(serde_json::from_value::<T>(v)?)?)
}
fn arg(args: &[Value], n: usize) -> Value {
    args.get(n).cloned().unwrap_or(Value::Null)
}
fn options(args: &[Value], n: usize) -> Value {
    args.get(n).cloned().unwrap_or(json!({}))
}
pub fn evaluate(op: &str, args: &[Value]) -> Result<Value> {
    let v = arg(args, 0);
    match op {
        "contracts/harness-capability.mjs#harnessCapability"
        | "contracts/harness-capability.mjs#validateHarnessCapability" => {
            admitted::<HarnessCapability>(v)
        }
        "contracts/harness-detection.mjs#harnessDescriptor"
        | "contracts/harness-detection.mjs#validateHarnessDescriptor" => {
            admitted::<HarnessDescriptor>(v)
        }
        "contracts/harness-detection.mjs#harnessDetection"
        | "contracts/harness-detection.mjs#validateHarnessDetection" => {
            admitted::<HarnessDetection>(v)
        }
        "contracts/harness-detection.mjs#harnessSelf"
        | "contracts/harness-detection.mjs#validateHarnessSelf" => admitted::<HarnessSelf>(v),
        "contracts/harness-detection.mjs#harnessCatalog"
        | "contracts/harness-detection.mjs#validateHarnessCatalog" => admitted::<HarnessCatalog>(v),
        "contracts/instantiation.mjs#validateModelRelation" => admitted::<ModelRelation>(v),
        "contracts/instantiation.mjs#validateModelAccessProfile" => {
            admitted::<ModelAccessProfile>(v)
        }
        "contracts/instantiation.mjs#validateActuationReceipt" => {
            admitted::<InstantiationReceipt>(v)
        }
        "contracts/instantiation.mjs#instantiationReceipt" => {
            Ok(InstantiationReceipt::read(v)?.into_value())
        }
        "contracts/instantiation.mjs#attachDetectionEvidence" => {
            attach_detection_evidence(&v, &arg(args, 1))
        }
        "contracts/secret-detection.mjs#secretScan"
        | "contracts/secret-detection.mjs#validateSecretScan" => admitted::<SecretScan>(v),
        "contracts/secret-detection.mjs#secretSourceCatalog"
        | "contracts/secret-detection.mjs#validateSecretSourceCatalog" => {
            admitted::<SecretSourceCatalog>(v)
        }
        "contracts/secret-detection.mjs#secretSourceDescriptor"
        | "contracts/secret-detection.mjs#validateSecretSourceDescriptor" => {
            admitted::<SecretSourceDescriptor>(v)
        }
        "contracts/model-usage.mjs#modelUsageFromClaudeCodeTranscript" => Ok(serde_json::to_value(
            usage::from_claude_code_transcript(&v, &options(args, 1))?,
        )?),
        "contracts/model-usage.mjs#modelUsageFromCodexExecEvents" => Ok(serde_json::to_value(
            usage::from_codex_exec_events(&v, &options(args, 1))?,
        )?),
        _ => stream::evaluate(op, args),
    }
}
pub fn pure_row(row: &Value) -> Value {
    let mut result = stream::caught(evaluate(
        row["operation"].as_str().unwrap_or(""),
        row["args"].as_array().map(Vec::as_slice).unwrap_or(&[]),
    ));
    result["id"] = row["id"].clone();
    result
}
use actuation_adapters::{effects::*, secrets::*};
use std::collections::BTreeMap;
pub struct FixtureEffects {
    pub spec: Value,
    pub calls: Vec<Value>,
}
impl FixtureEffects {
    pub fn new(spec: Value) -> Self {
        Self {
            spec,
            calls: vec![],
        }
    }
    fn call(&mut self, name: &str, args: Vec<Value>, default: Value) -> Value {
        self.calls.push(json!({"effect":name,"args":args}));
        let key = serde_json::to_string(args.first().unwrap_or(&Value::Null)).unwrap();
        self.spec[name]["by_argument"]
            .get(&key)
            .or(self.spec[name].get("value"))
            .cloned()
            .unwrap_or(default)
    }
    fn omitted(&self, name: &str) -> bool {
        self.spec["omit"]
            .as_array()
            .is_some_and(|a| a.iter().any(|v| v == name))
    }
}
impl ProbeEffects for FixtureEffects {
    fn expand_home(&mut self, p: &str) -> ProbeResult<String> {
        Ok(self
            .call(
                "expandHome",
                vec![json!(p)],
                json!(p.replacen('~', "/home/oracle", 1)),
            )
            .as_str()
            .unwrap()
            .to_owned())
    }
    fn resolve_executable(&mut self, n: &[String]) -> ProbeResult<Option<String>> {
        let v = self.call(
            "resolveExecutable",
            vec![json!(n)],
            json!({"found":false,"path":null}),
        );
        if let Some(e) = v["error"].as_str() {
            return Err(e.into());
        }
        Ok(if v["found"] == true {
            Some(v["path"].as_str().unwrap().into())
        } else {
            None
        })
    }
    fn stat(&mut self, p: &str) -> ProbeResult<Option<FileObservation>> {
        let v = self.call("statProbe", vec![json!(p)], json!({"exists":false}));
        if let Some(e) = v["error"].as_str() {
            return Err(e.into());
        }
        Ok(if v["exists"] == true {
            Some(FileObservation {
                is_directory: v["isDir"] == true,
                modified_millis: v["mtimeMs"].as_f64(),
                byte_length: v["size"].as_u64(),
            })
        } else {
            None
        })
    }
    fn hash(&mut self, p: &str) -> ProbeResult<String> {
        let v = self.call("hashProbe", vec![json!(p)], json!({"ok":false}));
        if v["ok"] == true {
            Ok(v["sha256"].as_str().unwrap().into())
        } else {
            Err("unavailable".into())
        }
    }
    fn directory_count(&mut self, p: &str) -> ProbeResult<Option<usize>> {
        let v = self.call("dirCountProbe", vec![json!(p)], json!({"exists":false}));
        Ok(if v["exists"] == true {
            Some(v["count"].as_u64().unwrap() as usize)
        } else {
            None
        })
    }
    fn markers(
        &mut self,
        n: &[String],
        env: Option<&BTreeMap<String, String>>,
    ) -> ProbeResult<Vec<String>> {
        if self.omitted("envProbe") {
            return Ok(vec![]);
        }
        let mut args = vec![json!(n)];
        if let Some(e) = env {
            args.push(json!(e));
        }
        let v = self.call("envProbe", args, json!({"ok":true,"matched":{}}));
        if v["ok"] != true {
            return Err(v["error"].as_str().unwrap_or("env failed").into());
        }
        Ok(v["matched"].as_object().unwrap().keys().cloned().collect())
    }
    fn service(&mut self, s: &Value) -> ProbeResult<ServiceObservation> {
        if self.omitted("serviceProbe") {
            return Ok(ServiceObservation::Unverified(
                "service probe not implemented; declared, not verified".into(),
            ));
        }
        let v = self.call(
            "serviceProbe",
            vec![s.clone()],
            json!({"ok":true,"detail":"declared, not verified"}),
        );
        if v["ok"] == false {
            return Err(v["detail"]
                .as_str()
                .or(v["reason"].as_str())
                .unwrap_or("service failed")
                .into());
        }
        Ok(service_observation_from_detail(
            v["detail"].as_str().unwrap_or(""),
        ))
    }
    fn version(&mut self, p: &str, a: &[String]) -> ProbeResult<String> {
        let v = self.call(
            "versionProbe",
            vec![json!(p), json!(a)],
            json!({"ok":false,"reason":"not supplied"}),
        );
        if v["ok"] == true {
            Ok(v["version"].as_str().unwrap().into())
        } else {
            Err(v["reason"].as_str().unwrap_or("no reason captured").into())
        }
    }
    fn http_json(&mut self, u: &str) -> ProbeResult<Value> {
        let v = self.call(
            "httpJsonProbe",
            vec![json!(u)],
            json!({"ok":false,"reason":"not supplied"}),
        );
        if v["ok"] == true {
            Ok(v["body"].clone())
        } else {
            Err(v["reason"].as_str().unwrap_or("no reason captured").into())
        }
    }
}
impl SecretEffects for FixtureEffects {
    fn expand_root(&mut self, p: &str) -> ProbeResult<String> {
        Ok(p.into())
    }
    fn inspect(&mut self, kind: &str, spec: &Value) -> ProbeResult<SecretObservation> {
        let default = match kind {
            "env" => json!({"ok":true,"matched":{}}),
            "file-pattern" => json!({"ok":true,"matched":[]}),
            _ => json!({"ok":true,"found":false}),
        };
        let v = self.call(kind, vec![spec.clone()], default);
        if v["ok"] != true {
            return Err(v["reason"].as_str().unwrap_or("unavailable").into());
        }
        let mut evidence = Vec::new();
        if kind == "env" {
            for (k, v) in v["matched"].as_object().into_iter().flatten() {
                evidence.push(FingerprintEvidence {
                    location: k.clone(),
                    fingerprint_sha256: v["fingerprint_sha256"].as_str().unwrap().into(),
                    byte_length: v["byte_length"].as_u64().unwrap(),
                });
            }
        }
        if kind == "file-pattern" {
            for v in v["matched"].as_array().into_iter().flatten() {
                evidence.push(FingerprintEvidence {
                    location: v["path"].as_str().unwrap().into(),
                    fingerprint_sha256: v["fingerprint_sha256"].as_str().unwrap().into(),
                    byte_length: v["byte_length"].as_u64().unwrap(),
                });
            }
        }
        let present = !evidence.is_empty() || v["found"] == true;
        let detail = if kind == "vault-item" && present {
            Some(format!(
                "item {} in vault {}",
                v["item_id"].as_str().unwrap_or("?"),
                v["vault"].as_str().unwrap_or("?")
            ))
        } else {
            None
        };
        Ok(SecretObservation {
            present,
            evidence,
            detail,
            truncated: v["truncated"] == true,
            files_scanned: v["files_scanned"].as_u64().unwrap_or(0),
        })
    }
}
pub fn scenario(row: &Value) -> Result<Value> {
    let catalog = NativeCatalog::bundled()?;
    match row["kind"].as_str() {
        Some("catalog") => Ok(
            json!({"revision":catalog.revision(),"harnesses":catalog.descriptors(),"capabilities":catalog.capabilities(),"secret_sources":catalog.secret_sources()}),
        ),
        Some("probe") => {
            let input = &row["input"];
            let mut effects = FixtureEffects::new(input["effects"].clone());
            let descriptors = if let Some(v) = input.get("descriptors") {
                serde_json::from_value(v.clone())?
            } else {
                let slugs: Vec<String> = input
                    .get("slugs")
                    .map(|v| serde_json::from_value(v.clone()))
                    .transpose()?
                    .unwrap_or_default();
                catalog.select(&slugs)?
            };
            let options = DetectionOptions {
                observed_at: actuation_stream::Timestamp::new(input["now"].as_str().unwrap())?,
                probe_versions: input["versions"] == true,
                catalog_revision: catalog.revision(),
            };
            let result = if input["mode"] == "self" {
                resolve_self(&descriptors, &mut effects, Some(&BTreeMap::new()), &options)
                    .map(|r| r.into_value())
            } else {
                run_detection(&descriptors, &mut effects, &options).map(|r| r.into_value())
            };
            Ok(json!({"result":stream::caught(result),"calls":effects.calls}))
        }
        Some("secret") => {
            let mut effects = FixtureEffects::new(row["input"]["effects"].clone());
            let options = ScanOptions {
                roots: vec!["/oracle/project".into()],
                observed_at: actuation_stream::Timestamp::new("2026-09-10T00:00:00.000Z")?,
                scanner_implementation: "actuation secret-scan".into(),
                scanner_version: "0.1.0".into(),
            };
            let mut value = scan_secret_sources(&catalog, &mut effects, &options)?.into_value();
            value["scan_ref"] = json!("secret-scan:$CLOCK");
            value["observed_at"] = json!("$CLOCK");
            Ok(json!({"value":value,"calls":effects.calls}))
        }
        _ => Err(Error::new("not an adapter scenario")),
    }
}

// Original scripted effects use readable service strings. The native engine
// receives typed presence instead; this parser belongs only to oracle transport.
fn service_observation_from_detail(detail: &str) -> ServiceObservation {
    if regex::Regex::new(r"^(http \d{3} from |daemon .+ running$)")
        .unwrap()
        .is_match(detail)
    {
        ServiceObservation::Present(detail.into())
    } else {
        ServiceObservation::Unverified(detail.into())
    }
}
