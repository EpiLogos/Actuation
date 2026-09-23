use crate::{admission::*, effects::*, Error, NativeCatalog, Result};
use actuation_stream::Timestamp;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};
use std::time::Duration;

pub const DETECTOR_IMPLEMENTATION: &str = "actuation surface-probes";
pub const DETECTOR_VERSION: &str = "0.2.0";
#[derive(Clone, Debug)]
pub struct DetectionOptions {
    pub observed_at: Timestamp,
    pub probe_versions: bool,
    pub catalog_revision: i64,
}
impl DetectionOptions {
    pub fn now(catalog: &NativeCatalog) -> Result<Self> {
        Ok(Self {
            observed_at: Timestamp::now()?,
            probe_versions: false,
            catalog_revision: catalog.revision(),
        })
    }
}
fn probe(kind: &str, ok: bool, spec: Option<&str>, detail: Option<&str>) -> Value {
    let mut v = json!({"kind":kind,"result":if ok {"pass"} else {"fail"}});
    if let Some(s) = spec {
        v["spec"] = json!(s);
    }
    if let Some(s) = detail {
        v["detail"] = json!(s.chars().take(200).collect::<String>());
    }
    v
}
/// The named failure class a probe error carries, in the shared probe outcome
/// vocabulary (`ok | credential-gated | unreachable | unsupported | timed-out
/// | refused`). Only the bounded-observation refusals this engine produces
/// are classified; any other error stays a plain failure without a named
/// outcome.
pub fn probe_outcome(err: &str) -> Option<&'static str> {
    if err.contains("timed out") {
        Some("timed-out")
    } else if err.starts_with("spawn failed") {
        Some("unreachable")
    } else if err.starts_with("exit ") {
        Some("refused")
    } else if err == "unsupported service kind" {
        Some("unsupported")
    } else {
        None
    }
}
/// A failed probe record that names its outcome class when the error is one
/// the engine itself raises (bound refusal, spawn failure, target refusal,
/// unsupported observation). The plain `fail` result is preserved so the
/// pass/fail axis and the named outcome axis stay independent.
fn classified_probe(kind: &str, spec: Option<&str>, err: &str) -> Value {
    let mut v = probe(kind, false, spec, Some(err));
    if let Some(outcome) = probe_outcome(err) {
        v["outcome"] = json!(outcome);
    }
    v
}
fn spec_names(v: &Value, key: &str) -> Result<Vec<String>> {
    texts(&v[key])
}
/// The per-probe wall-clock bound: a declared `timeout_ms` override or the
/// engine default. The declaration rides the descriptor's own probe spec.
fn probe_bound(spec: Option<&Value>) -> Result<Duration> {
    let declared = spec.map(|s| &s["timeout_ms"]).filter(|v| !v.is_null());
    match declared {
        Some(ms) => Ok(Duration::from_millis(ms.as_u64().ok_or_else(|| {
            Error::new("probe timeout_ms must be a positive integer")
        })?)),
        None => Ok(DEFAULT_PROBE_BOUND),
    }
}

/// Observe an existing native condition. A result describes the observation
/// source and its limits, never a selected route, authority or a new Agent.
pub fn run_detection(
    descriptors: &[HarnessDescriptor],
    effects: &mut dyn ProbeEffects,
    options: &DetectionOptions,
) -> Result<HarnessDetection> {
    let mut entries = Vec::new();
    let mut disclosure = Vec::<String>::new();
    for d in descriptors {
        let p = d.probe_plan();
        let mut probes = Vec::new();
        let mut executable = None;
        let mut config_present = false;
        let mut service_live = false;
        let wanted = match p.get("config-dir") {
            Some(s) => Some(effects.expand_home(text(&s["path"])?)),
            None => None,
        };
        if let Some(s) = p.get("executable") {
            let names = spec_names(s, "names")?;
            let spec = names.join(" ");
            match effects.resolve_executable(&names) {
                Ok(Some(path)) => {
                    probes.push(probe("executable", true, Some(&spec), Some(&path)));
                    executable = Some(path);
                }
                Ok(None) => probes.push(probe(
                    "executable",
                    true,
                    Some(&spec),
                    Some("not found on PATH"),
                )),
                Err(e) => probes.push(classified_probe("executable", Some(&spec), &e)),
            }
        }
        if let Some(s) = p.get("config-dir") {
            let path = text(&s["path"])?;
            match wanted.as_ref().expect("configured path") {
                Ok(full) => match effects.stat(full) {
                    Ok(observed) => {
                        config_present = observed.is_some();
                        probes.push(probe(
                            "config-dir",
                            true,
                            Some(path),
                            Some(&if config_present {
                                format!("exists at {full}")
                            } else {
                                "not found".into()
                            }),
                        ));
                    }
                    Err(e) => probes.push(classified_probe("config-dir", Some(path), &e)),
                },
                Err(e) => probes.push(probe("config-dir", false, Some(path), Some(e))),
            }
        }
        if let Some(s) = p.get("service") {
            let kind = s["kind"].as_str().unwrap_or("service");
            effects.set_probe_bound(probe_bound(Some(s))?);
            match effects.service(s) {
                Ok(reading) => {
                    service_live = reading.is_present();
                    probes.push(probe("service", true, Some(kind), Some(reading.detail())));
                }
                Err(e) => probes.push(classified_probe("service", Some(kind), &e)),
            }
            effects.set_probe_bound(DEFAULT_PROBE_BOUND);
        }
        if let Some(s) = p.get("env") {
            let names = spec_names(s, "any_of")?;
            let spec = names.join(" ");
            match effects.markers(&names, None) {
                Ok(names) => {
                    let detail = if names.is_empty() {
                        "no marker set".to_owned()
                    } else {
                        names
                            .iter()
                            .map(|n| format!("{n} set"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    };
                    probes.push(probe("env", true, Some(&spec), Some(&detail)));
                }
                Err(e) => probes.push(classified_probe("env", Some(&spec), &e)),
            }
        }
        // A declared credential signal is a cheap presence stat, never a read:
        // surfacing it in disclosure spares callers from discovering a
        // credential gate by hanging on an external invocation.
        let mut credential = None;
        if let Some(spec) = d.as_value()["credential"].as_object() {
            let declared = text(&spec["path"])?.to_owned();
            let observed = effects
                .expand_home(&declared)
                .and_then(|full| effects.stat(&full).map(|o| (declared.clone(), o)));
            match observed {
                Ok((declared, Some(_))) => {
                    credential = Some(json!({"path": declared, "outcome": "credential-gated"}));
                    disclosure.push(format!(
                        "{}: credential-gated ({declared} present; presence only, contents never read)",
                        d.slug()
                    ));
                }
                Ok((declared, None)) => {
                    credential = Some(json!({"path": declared, "outcome": "absent"}));
                }
                Err(e) => disclosure.push(format!(
                    "{}: credential presence not observable ({e})",
                    d.slug()
                )),
            }
        }
        let any_pass = probes.iter().any(|p| p["result"] == "pass");
        let any_fail = probes.iter().any(|p| p["result"] == "fail");
        let present = executable.is_some() || config_present || service_live;
        let presence_failed = probes
            .iter()
            .any(|p| p["result"] == "fail" && p["kind"] != "env");
        let state = if (!any_pass && any_fail) || (!present && presence_failed) {
            "unavailable"
        } else if present {
            "detected"
        } else {
            "not-installed"
        };
        let mut entry = json!({"slug":d.slug(),"harness_ref":format!("harness/{}",d.slug()),"native_kind":d.native_kind(),"state":state,"probes":probes});
        if let Some(c) = credential {
            entry["credential"] = c;
        }
        if state == "unavailable" {
            entry["unavailable_reason"] = json!(if !any_pass {
                "all probes failed; could not run"
            } else {
                "presence observation incomplete; successful marker observation cannot prove absence"
            });
        }
        if state == "detected" {
            let mut receipts = json!({});
            // Preserve the actual resolution separately from its readable probe
            // detail. A passing ABSENCE probe must never become an executable.
            effects.set_probe_bound(probe_bound(p.get("executable"))?);
            if let Some(path) = &executable {
                receipts["executable"] = json!(path);
                if let Ok(hash) = effects.hash(path) {
                    receipts["sha256"] = json!(hash);
                }
                if let Ok(Some(stat)) = effects.stat(path) {
                    if !stat.is_directory {
                        if let Some(mtime) = stat.modified_millis {
                            receipts["mtime"] = json!(mtime.round() as i64);
                        }
                        if let Some(size) = stat.byte_length {
                            receipts["size"] = json!(size);
                        }
                    }
                }
            } else if let Some(Ok(path)) = &wanted {
                if matches!(effects.stat(path), Ok(Some(_))) {
                    receipts["executable"] = json!(path);
                    receipts["executable_is"] = json!("config-dir");
                }
            }
            effects.set_probe_bound(DEFAULT_PROBE_BOUND);
            let facets =
                observe_facets(d, &probes, service_live, effects, &mut disclosure, options)?;
            if !facets.is_empty() {
                entry["facets"] = json!(facets);
            }
            if receipts.as_object().expect("object").is_empty() {
                // No executable receipt can truthfully be supplied. This is a
                // degraded observation, not proof of absence or a fake path.
                entry["state"] = json!("unavailable");
                entry["unavailable_reason"] = json!(
                    "presence observed but no executable or configured-path receipt was captured"
                );
                entry.as_object_mut().expect("object").remove("facets");
            } else {
                entry["receipts"] = receipts;
                if options.probe_versions && executable.is_some() {
                    let path = executable.as_deref().expect("known executable");
                    let args = p
                        .get("executable")
                        .and_then(|s| s.get("version_args"))
                        .filter(|v| !v.is_null())
                        .map(texts)
                        .transpose()?
                        .unwrap_or_else(|| vec!["--version".into()]);
                    let spec = p.get("executable");
                    effects.set_probe_bound(probe_bound(spec)?);
                    match effects.version(path, &args) {
                        Ok(version) => entry["version"] = json!(version),
                        Err(reason) => {
                            // A version failure is a named probe outcome, not
                            // an error bubble: the record carries the class
                            // (timed-out, unreachable, refused) and the bound
                            // that elapsed.
                            let record =
                                classified_probe("version", Some(&args.join(" ")), &reason);
                            entry["probes"]
                                .as_array_mut()
                                .expect("admitted probes array")
                                .push(record);
                            let line = probe_outcome(&reason)
                                .map(|token| format!("version probe {token} ({reason})"))
                                .unwrap_or_else(|| format!("version probe failed ({reason})"));
                            disclosure.push(format!("{}: {line}", d.slug()));
                        }
                    }
                    effects.set_probe_bound(DEFAULT_PROBE_BOUND);
                }
            }
        }
        entries.push(entry);
    }
    let absent: Vec<_> = entries
        .iter()
        .filter(|e| e["state"] == "not-installed")
        .map(|e| e["slug"].clone())
        .collect();
    let partial = entries.iter().any(|e| e["state"] == "unavailable");
    let now = options.observed_at.as_str();
    let mut out = json!({"schema":HARNESS_DETECTION_VERSION,"document":"detection","detection_ref":format!("detection:{now}"),"observed_at":now,"catalog_revision":options.catalog_revision,"detector":{"implementation":DETECTOR_IMPLEMENTATION,"version":DETECTOR_VERSION},"harnesses":entries,"absent":absent,"availability":if partial {"partial"} else {"complete"}});
    if !disclosure.is_empty() {
        out["disclosure"] = json!(disclosure);
    }
    HarnessDetection::try_from(out)
}
fn observe_facets(
    d: &HarnessDescriptor,
    probes: &[Value],
    live: bool,
    effects: &mut dyn ProbeEffects,
    disclosure: &mut Vec<String>,
    options: &DetectionOptions,
) -> Result<Vec<Value>> {
    let mut out = Vec::new();
    if let Some(facets) = d.as_value()["facets"].as_object() {
        for (kind, spec) in facets {
            let declared = text(&spec["path"])?;
            let Ok(path) = effects.expand_home(declared) else {
                continue;
            };
            let Ok(Some(stat)) = effects.stat(&path) else {
                continue;
            };
            let mut entry = json!({"kind":kind,"path":declared,"exists":true});
            if stat.is_directory {
                if let Ok(Some(count)) = effects.directory_count(&path) {
                    entry["count"] = json!(count);
                }
            }
            if !spec["inventory"].is_null() {
                let context = InventoryContext {
                    descriptor: d,
                    facet_path: declared,
                    expanded_path: &path,
                    probes,
                    live,
                };
                let inventory =
                    observe_inventory(context, &spec["inventory"], effects, disclosure, options)?;
                for (k, v) in object(&inventory)? {
                    entry[k] = v.clone();
                }
            }
            out.push(entry);
        }
    }
    Ok(out)
}
/// One MCP server spec reduced to an evidence-bearing, secret-free summary:
/// the launch command (or remote URL), with secret-flag arguments redacted —
/// including the value a bare secret flag introduces — and environment blocks
/// never read.
fn mcp_server_summary(spec: &Value) -> Option<String> {
    const SECRET_FLAGS: &[&str] = &[
        "password",
        "token",
        "api-key",
        "apikey",
        "secret",
        "credential",
        "auth",
    ];
    let is_secret_flag = |arg: &str| -> bool {
        let lower = arg.to_lowercase();
        SECRET_FLAGS
            .iter()
            .any(|flag| lower == format!("--{flag}") || lower == format!("-{flag}"))
    };
    let redact_inline = |arg: &str| -> String {
        let lower = arg.to_lowercase();
        for flag in SECRET_FLAGS {
            if lower.starts_with(&format!("--{flag}=")) || lower.starts_with(&format!("{flag}=")) {
                return format!("{}=[redacted]", arg.split('=').next().unwrap_or(arg));
            }
        }
        arg.to_owned()
    };
    if let Some(command) = spec["command"].as_str() {
        let args: Vec<&str> = spec["args"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        let mut summary = redact_inline(command);
        let mut redact_next_value = false;
        for arg in args {
            summary.push(' ');
            if redact_next_value {
                summary.push_str("[redacted-value]");
                redact_next_value = false;
            } else if is_secret_flag(arg) {
                summary.push_str("[redacted-flag]");
                redact_next_value = true;
            } else {
                summary.push_str(&redact_inline(arg));
            }
        }
        Some(summary.chars().take(200).collect())
    } else {
        spec["url"]
            .as_str()
            .or(spec["endpoint"].as_str())
            .map(|u| u.chars().take(200).collect())
    }
}
/// A file-declared inventory reads the JSON document at the facet path and
/// names the entries under its dotted `collection` path. Read failures are
/// disclosure, never an empty inventory.
fn observe_file_inventory(
    facet_path: &str,
    expanded_path: &str,
    declared: &Value,
    effects: &mut dyn ProbeEffects,
    disclosure: &mut Vec<String>,
    options: &DetectionOptions,
) -> Result<Value> {
    let unavailable = |reason: String| json!({"inventory_unavailable_reason": format!("declared file inventory not read: {reason}")});
    let contents = match effects.read_text_file(expanded_path) {
        Ok(c) => c,
        Err(reason) => {
            disclosure.push(format!("mcp-config inventory read failed ({reason})"));
            return Ok(unavailable(reason));
        }
    };
    let parsed: Value = match serde_json::from_str(&contents) {
        Ok(v) => v,
        Err(reason) => {
            let reason = format!("unparseable inventory JSON: {reason}");
            disclosure.push(reason.clone());
            return Ok(unavailable(reason));
        }
    };
    let collection = text(&declared["collection"])?;
    let mut cursor = &parsed;
    for segment in collection.split('.') {
        cursor = &cursor[segment];
    }
    let Some(entries) = cursor.as_object() else {
        let reason = format!("no \"{collection}\" object in {facet_path}");
        disclosure.push(reason.clone());
        return Ok(unavailable(reason));
    };
    let mut inventory = Vec::new();
    for (name, spec) in entries {
        let mut item = json!({"id": name});
        if let Some(summary) = mcp_server_summary(spec) {
            item["command"] = json!(summary);
        }
        inventory.push(item);
    }
    Ok(
        json!({"inventory":inventory,"inventory_receipt":{"kind":declared["kind"],"source":facet_path,"observed_at":options.observed_at.as_str(),"item_count":inventory.len()}}),
    )
}
/// The descriptor-scoped facts one facet observation needs, bundled so the
/// observation functions keep a readable argument list.
struct InventoryContext<'a> {
    descriptor: &'a HarnessDescriptor,
    facet_path: &'a str,
    expanded_path: &'a str,
    probes: &'a [Value],
    live: bool,
}

fn observe_inventory(
    context: InventoryContext<'_>,
    declared: &Value,
    effects: &mut dyn ProbeEffects,
    disclosure: &mut Vec<String>,
    options: &DetectionOptions,
) -> Result<Value> {
    let InventoryContext {
        descriptor: d,
        facet_path,
        expanded_path,
        probes,
        live,
    } = context;
    if declared["source"] == "file" {
        return observe_file_inventory(
            facet_path,
            expanded_path,
            declared,
            effects,
            disclosure,
            options,
        );
    }
    if !live {
        let reason = if let Some(service) = probes.iter().find(|p| p["kind"] == "service") {
            format!("declared service inventory not read: service probe did not prove a live endpoint ({})", service["detail"].as_str().or(service["result"].as_str()).unwrap_or("unknown"))
        } else {
            "declared service inventory not read: no service probe ran".into()
        };
        return Ok(json!({"inventory_unavailable_reason":reason}));
    }
    let service = &d.as_value()["probe"]["service"];
    let base = service["default_url"]
        .as_str()
        .or(service["url"].as_str())
        .ok_or_else(|| Error::new("inventory has no declared service URL"))?;
    let url = format!(
        "{}{}",
        base.strip_suffix('/').unwrap_or(base),
        text(&declared["route"])?
    );
    let answer = match effects.http_json(&url) {
        Ok(v) => v,
        Err(reason) => {
            disclosure.push(format!(
                "{}: model inventory read failed ({reason})",
                d.slug()
            ));
            return Ok(
                json!({"inventory_unavailable_reason":format!("inventory read from {url} failed: {reason}")}),
            );
        }
    };
    let collection = text(&declared["collection"])?;
    let Some(items) = answer[collection].as_array() else {
        return Ok(
            json!({"inventory_unavailable_reason":format!("inventory read from {url} returned no \"{collection}\" array")}),
        );
    };
    let id_field = text(&declared["id_field"])?;
    let aliases = if declared["also_id_fields"].is_null() {
        vec![]
    } else {
        texts(&declared["also_id_fields"])?
    };
    let details = if declared["detail_fields"].is_null() {
        vec![]
    } else {
        texts(&declared["detail_fields"])?
    };
    let mut seen = HashSet::new();
    let mut inventory = Vec::new();
    for item in items {
        if !item.is_object() {
            continue;
        }
        let Ok(id) = text(&item[id_field]) else {
            continue;
        };
        if !seen.insert(id) {
            continue;
        }
        let mut entry = json!({"id":id});
        let mut names = Vec::<String>::new();
        for a in &aliases {
            if let Ok(value) = text(&item[a]) {
                if value != id && !names.iter().any(|n| n == value) {
                    names.push(value.to_owned());
                }
            }
        }
        if !names.is_empty() {
            entry["also_known_as"] = json!(names);
        }
        for k in &details {
            if item[k].is_string() || item[k].is_number() {
                entry[k] = item[k].clone();
            }
        }
        inventory.push(entry);
    }
    Ok(
        json!({"inventory":inventory,"inventory_receipt":{"kind":declared["kind"],"source":url,"observed_at":options.observed_at.as_str(),"item_count":inventory.len()}}),
    )
}

pub fn resolve_self(
    descriptors: &[HarnessDescriptor],
    effects: &mut dyn ProbeEffects,
    environment: Option<&BTreeMap<String, String>>,
    options: &DetectionOptions,
) -> Result<HarnessSelf> {
    let mut matched = Vec::new();
    for d in descriptors {
        let Some(spec) = d.probe_plan().get("env") else {
            continue;
        };
        let names = texts(&spec["any_of"])?;
        if let Ok(markers) = effects.markers(&names, environment) {
            if !markers.is_empty() {
                matched.push(json!({"slug":d.slug(),"harness_ref":format!("harness/{}",d.slug()),"markers":markers}));
            }
        }
    }
    let detection = run_detection(
        descriptors,
        effects,
        &DetectionOptions {
            probe_versions: false,
            ..options.clone()
        },
    )?;
    let resolved = if matched.len() == 1 {
        matched[0].clone()
    } else {
        Value::Null
    };
    let states: serde_json::Map<_, _> = detection.as_value()["harnesses"]
        .as_array()
        .expect("admitted")
        .iter()
        .map(|e| {
            (
                e["slug"].as_str().expect("admitted").to_owned(),
                e["state"].clone(),
            )
        })
        .collect();
    let now = options.observed_at.as_str();
    HarnessSelf::try_from(
        json!({"schema":HARNESS_DETECTION_VERSION,"document":"self","self_ref":format!("self:{now}"),"observed_at":now,"catalog_revision":options.catalog_revision,"matched":matched,"resolved":resolved,"ambiguity":matched.len()>1,"detection_ref":detection.as_value()["detection_ref"],"detection":{"states":states}}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_outcomes_cover_the_engine_refusal_classes() {
        // The gemini 0.29.5 expired-oauth class: an init-bearing binary that
        // produces no output and no exit inside the bound.
        assert_eq!(
            probe_outcome("process observation timed out after 10s"),
            Some("timed-out")
        );
        assert_eq!(
            probe_outcome("observation timed out after 4s"),
            Some("timed-out")
        );
        assert_eq!(
            probe_outcome("executable fingerprint timed out after 10s"),
            Some("timed-out")
        );
        assert_eq!(
            probe_outcome("directory count timed out after 10s"),
            Some("timed-out")
        );
        assert_eq!(
            probe_outcome("spawn failed: PermissionDenied"),
            Some("unreachable")
        );
        assert_eq!(
            probe_outcome("spawn failed: ExecFormatError"),
            Some("unreachable")
        );
        assert_eq!(probe_outcome("exit 1"), Some("refused"));
        assert_eq!(probe_outcome("exit signal"), Some("refused"));
        assert_eq!(
            probe_outcome("unsupported service kind"),
            Some("unsupported")
        );
        assert_eq!(probe_outcome("metadata unavailable: EIO"), None);
        assert_eq!(probe_outcome("pgrep exit Some(2)"), None);
        // A refused connection is an absence observation, not a refused probe.
        assert_eq!(probe_outcome("connection refused"), None);
    }

    #[test]
    fn classified_failures_carry_the_outcome_and_plain_failures_do_not() {
        let timed_out = classified_probe(
            "version",
            Some("--version"),
            "process observation timed out after 10s",
        );
        assert_eq!(timed_out["result"], json!("fail"));
        assert_eq!(timed_out["outcome"], json!("timed-out"));
        assert_eq!(
            timed_out["detail"],
            json!("process observation timed out after 10s")
        );
        let generic = classified_probe("executable", Some("fixture"), "metadata unavailable: EIO");
        assert_eq!(generic["result"], json!("fail"));
        assert!(generic.get("outcome").is_none());
    }
}
