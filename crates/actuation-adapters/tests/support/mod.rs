#![allow(dead_code)]
// Temporary oracle transport. Only this test layer understands old JS effect
// records, operation strings and fixture calls. Production takes typed ports.
#[path = "../../../actuation-stream/tests/support/mod.rs"]
mod stream;
use actuation_adapters::*;
use actuation_stream::Timestamp;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
fn encode<T: Serialize>(v: T) -> Result<Value> {
    Ok(serde_json::to_value(v)?)
}
fn admitted<T: Serialize + DeserializeOwned>(v: Value) -> Result<Value> {
    encode(serde_json::from_value::<T>(v)?)
}
fn arg(args: &[Value], n: usize) -> Value {
    args.get(n).cloned().unwrap_or(Value::Null)
}
fn usage_options(args: &[Value]) -> Value {
    args.get(1).cloned().unwrap_or(json!({}))
}
pub fn evaluate(operation: &str, args: &[Value]) -> Result<Value> {
    let first = arg(args, 0);
    match operation {
        "contracts/harness-capability.mjs#harnessCapability"
        | "contracts/harness-capability.mjs#validateHarnessCapability" => {
            admitted::<HarnessCapability>(first)
        }
        "contracts/harness-detection.mjs#harnessDescriptor"
        | "contracts/harness-detection.mjs#validateHarnessDescriptor" => {
            admitted::<HarnessDescriptor>(first)
        }
        "contracts/harness-detection.mjs#harnessDetection"
        | "contracts/harness-detection.mjs#validateHarnessDetection" => {
            admitted::<DetectionReport>(first)
        }
        "contracts/harness-detection.mjs#harnessSelf"
        | "contracts/harness-detection.mjs#validateHarnessSelf" => admitted::<HarnessSelf>(first),
        "contracts/harness-detection.mjs#harnessCatalog"
        | "contracts/harness-detection.mjs#validateHarnessCatalog" => {
            admitted::<HarnessCatalogDocument>(first)
        }
        "contracts/instantiation.mjs#validateModelRelation" => admitted::<ModelRelation>(first),
        "contracts/instantiation.mjs#validateModelAccessProfile" => {
            admitted::<ModelAccessProfile>(first)
        }
        "contracts/instantiation.mjs#validateActuationReceipt" => {
            admitted::<InstantiationReceipt>(first)
        }
        "contracts/instantiation.mjs#instantiationReceipt" => {
            encode(InstantiationReceipt::read(first)?)
        }
        "contracts/instantiation.mjs#attachDetectionEvidence" => {
            let receipt = InstantiationReceipt::new(first)?;
            if receipt
                .as_value()
                .get("harness_ref")
                .is_none_or(Value::is_null)
            {
                encode(receipt.mark_unattributed()?)
            } else {
                encode(receipt.attach_detection(&DetectionReport::new(arg(args, 1))?)?)
            }
        }
        "contracts/secret-detection.mjs#secretSourceDescriptor"
        | "contracts/secret-detection.mjs#validateSecretSourceDescriptor" => {
            admitted::<SecretSourceDescriptor>(first)
        }
        "contracts/secret-detection.mjs#secretSourceCatalog"
        | "contracts/secret-detection.mjs#validateSecretSourceCatalog" => {
            admitted::<SecretSourceCatalog>(first)
        }
        "contracts/secret-detection.mjs#secretScan"
        | "contracts/secret-detection.mjs#validateSecretScan" => admitted::<SecretScan>(first),
        "contracts/model-usage.mjs#modelUsageFromClaudeCodeTranscript" => encode(
            usage::from_claude_code_transcript(&first, &usage_options(args))?,
        ),
        "contracts/model-usage.mjs#modelUsageFromCodexExecEvents" => {
            encode(usage::from_codex_exec_events(&first, &usage_options(args))?)
        }
        _ => stream::evaluate(operation, args),
    }
}
pub fn caught(r: Result<Value>) -> Value {
    match r {
        Ok(v) => json!({"ok":true,"value":v}),
        Err(e) => json!({"ok":false,"error":{"name":"TypeError","message":e.to_string()}}),
    }
}
pub fn pure_row(row: &Value) -> Value {
    let mut result = caught(match (row["operation"].as_str(), row["args"].as_array()) {
        (Some(o), Some(a)) => evaluate(o, a),
        _ => Err(Error::new("invalid oracle request")),
    });
    result["id"] = row["id"].clone();
    result
}
pub struct FixtureEffects {
    pub spec: Value,
    pub calls: Vec<Value>,
    pub self_reads: usize,
}
impl FixtureEffects {
    pub fn new(spec: Value) -> Self {
        Self {
            spec,
            calls: Vec::new(),
            self_reads: 0,
        }
    }
    fn omitted(&self, name: &str) -> bool {
        self.spec["omit"]
            .as_array()
            .is_some_and(|a| a.iter().any(|n| n == name))
    }
    fn call(&mut self, name: &str, args: Value, default: Value) -> Value {
        self.calls.push(json!({"effect":name,"args":args}));
        let key = serde_json::to_string(&args[0]).unwrap();
        self.spec[name]["by_argument"]
            .get(&key)
            .or_else(|| self.spec[name].get("value"))
            .cloned()
            .unwrap_or(default)
    }
}
fn stat(v: Value) -> Observation<FileObservation> {
    if let Some(error) = v["error"].as_str() {
        Observation::Unavailable(error.into())
    } else if v["exists"] == true {
        Observation::Present(FileObservation {
            is_directory: v["isDir"] == true,
            size: v["size"].as_u64(),
            modified_millis: v["mtimeMs"].as_f64(),
        })
    } else {
        Observation::Absent
    }
}
impl DetectionEffects for FixtureEffects {
    fn expand_home(&mut self, path: &str) -> Result<String> {
        let v = self.call(
            "expandHome",
            json!([path]),
            json!(path.replacen('~', "/home/oracle", 1)),
        );
        v.as_str()
            .map(str::to_owned)
            .ok_or_else(|| Error::new("invalid expandHome fixture"))
    }
    fn resolve_executable(&mut self, names: &[String]) -> Observation<String> {
        let v = self.call(
            "resolveExecutable",
            json!([names]),
            json!({"found":false,"path":null}),
        );
        if let Some(e) = v["error"].as_str() {
            Observation::Unavailable(e.into())
        } else if v["found"] == true {
            v["path"]
                .as_str()
                .map(|s| Observation::Present(s.into()))
                .unwrap_or_else(|| Observation::Unavailable("missing executable path".into()))
        } else {
            Observation::Absent
        }
    }
    fn stat(&mut self, path: &str) -> Observation<FileObservation> {
        stat(self.call("statProbe", json!([path]), json!({"exists":false})))
    }
    fn fingerprint(&mut self, path: &str) -> Result<Fingerprint> {
        let v = self.call("hashProbe", json!([path]), json!({"ok":false}));
        if v["ok"] == true {
            serde_json::from_value(v["sha256"].clone()).map_err(Into::into)
        } else {
            Err(Error::new("hash unavailable"))
        }
    }
    fn directory_count(&mut self, path: &str) -> Observation<u64> {
        let v = self.call("dirCountProbe", json!([path]), json!({"exists":false}));
        if v["exists"] == true {
            Observation::Present(v["count"].as_u64().unwrap())
        } else if let Some(e) = v["error"].as_str() {
            Observation::Unavailable(e.into())
        } else {
            Observation::Absent
        }
    }
    fn version(&mut self, path: &str, args: &[String]) -> Result<String> {
        let v = self.call(
            "versionProbe",
            json!([path, args]),
            json!({"ok":false,"reason":"not supplied"}),
        );
        if v["ok"] == true {
            Ok(v["version"].as_str().unwrap().into())
        } else {
            Err(Error::new(
                v["reason"].as_str().unwrap_or("no reason captured"),
            ))
        }
    }
    fn service(&mut self, spec: &Value) -> ServiceObservation {
        if self.omitted("serviceProbe") {
            return ServiceObservation::Unverified(
                "service probe not implemented; declared, not verified".into(),
            );
        }
        let v = self.call(
            "serviceProbe",
            json!([spec]),
            json!({"ok":true,"detail":"declared, not verified"}),
        );
        let detail = v["detail"]
            .as_str()
            .or_else(|| v["reason"].as_str())
            .unwrap_or("")
            .to_owned();
        if v["ok"] == false {
            ServiceObservation::Unavailable(detail)
        } else if regex::Regex::new(r"^(http \d{3} from |daemon .+ running$)")
            .unwrap()
            .is_match(&detail)
        {
            ServiceObservation::Present(detail)
        } else {
            ServiceObservation::Unverified(detail)
        }
    }
    fn http_json(&mut self, url: &str) -> Option<Result<Value>> {
        if self.omitted("httpJsonProbe") {
            return None;
        }
        let v = self.call(
            "httpJsonProbe",
            json!([url]),
            json!({"ok":false,"reason":"not supplied"}),
        );
        Some(if v["ok"] == true {
            Ok(v["body"].clone())
        } else {
            Err(Error::new(
                v["reason"].as_str().unwrap_or("no reason captured"),
            ))
        })
    }
    fn environment_markers(&mut self, names: &[String]) -> Result<Vec<String>> {
        if self.omitted("envProbe") {
            return Ok(Vec::new());
        }
        // Legacy self passed the explicitly isolated {} environment to its
        // effect. The native port already owns that environment; retain this
        // incidental test call spelling without putting JS args in production.
        let args = if self.self_reads > 0 {
            self.self_reads -= 1;
            json!([names, {}])
        } else {
            json!([names])
        };
        let v = self.call("envProbe", args, json!({"ok":true,"matched":{}}));
        if v["ok"] != true {
            return Err(Error::new(
                v["error"].as_str().unwrap_or("env probe failed"),
            ));
        }
        Ok(v["matched"]
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default())
    }
}
impl SecretEffects for FixtureEffects {
    fn probe(&mut self, kind: &str, spec: &Value) -> Option<Result<SecretFinding>> {
        if self.omitted(kind) {
            return None;
        }
        let default = match kind {
            "env" => json!({"ok":true,"matched":{}}),
            "file-pattern" => json!({"ok":true,"matched":[]}),
            _ => json!({"ok":true,"found":false}),
        };
        let v = self.call(kind, json!([spec]), default);
        Some((|| {
            if v["ok"] != true {
                return Err(Error::new(v["reason"].as_str().unwrap_or("probe failed")));
            }
            let mut f = SecretFinding {
                found: v["found"] == true,
                truncated: v["truncated"] == true,
                files_scanned: v["files_scanned"].as_u64().unwrap_or(0),
                ..SecretFinding::default()
            };
            if kind == "vault-item" && f.found {
                f.vault_item = Some((
                    v["item_id"].as_str().map(str::to_owned),
                    v["vault"].as_str().map(str::to_owned),
                ));
            }
            let mut evidence = |location: &str, item: &Value| -> Result<()> {
                f.evidence.push(FingerprintEvidence {
                    location: location.into(),
                    fingerprint: serde_json::from_value(item["fingerprint_sha256"].clone())?,
                    byte_length: item["byte_length"].as_u64(),
                });
                Ok(())
            };
            if kind == "env" {
                for (name, item) in v["matched"].as_object().into_iter().flatten() {
                    evidence(name, item)?;
                }
            }
            if kind == "file-pattern" {
                for item in v["matched"].as_array().into_iter().flatten() {
                    evidence(
                        item["path"]
                            .as_str()
                            .ok_or_else(|| Error::new("missing fixture path"))?,
                        item,
                    )?;
                }
            }
            Ok(f)
        })())
    }
}
pub fn scenario(row: &Value) -> Result<Value> {
    let catalog = Catalog::embedded()?;
    match row["kind"].as_str() {
        Some("catalog") => Ok(
            json!({"revision":catalog.revision(),"harnesses":catalog.descriptors(),"capabilities":catalog.capabilities(),"secret_sources":catalog.secret_catalog()}),
        ),
        Some("probe") => {
            let input = &row["input"];
            let descriptors: Vec<HarnessDescriptor> = if let Some(ds) = input.get("descriptors") {
                serde_json::from_value(ds.clone())?
            } else {
                catalog
                    .descriptors()
                    .iter()
                    .filter(|d| {
                        input["slugs"]
                            .as_array()
                            .is_none_or(|slugs| slugs.iter().any(|s| s == d.slug()))
                    })
                    .cloned()
                    .collect()
            };
            let mut effects =
                FixtureEffects::new(input.get("effects").cloned().unwrap_or(json!({})));
            let options = DetectionOptions {
                observed_at: serde_json::from_value(input["now"].clone())?,
                probe_versions: input["versions"] == true,
                catalog_revision: catalog.revision().clone(),
            };
            let result = if input["mode"] == "self" {
                effects.self_reads = descriptors
                    .iter()
                    .filter(|d| d.probes().contains_key("env"))
                    .count();
                observe_self(&descriptors, &mut effects, &options).and_then(encode)
            } else {
                detect(&descriptors, &mut effects, &options).and_then(encode)
            };
            Ok(json!({"result":caught(result),"calls":effects.calls}))
        }
        Some("secret") => {
            let mut effects = FixtureEffects::new(row["input"]["effects"].clone());
            let mut options = SecretScanOptions::new(vec!["/oracle/project".into()])?;
            options.observed_at = Timestamp::new("2026-09-10T00:00:00.000Z")?;
            let mut value =
                scan_secret_sources(catalog.secret_catalog(), &options, &mut effects)?.into_value();
            value["scan_ref"] = json!("secret-scan:$CLOCK");
            value["observed_at"] = json!("$CLOCK");
            Ok(json!({"value":value,"calls":effects.calls}))
        }
        _ => stream::scenario(row),
    }
}
pub fn normalized(v: Value) -> Value {
    stream::normalized(v)
}
