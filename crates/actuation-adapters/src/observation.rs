use crate::{admission::*, effects::*, Error, NativeCatalog, Result};
use actuation_stream::Timestamp;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};

pub const DETECTOR_IMPLEMENTATION: &str = "actuation surface-probes";
pub const DETECTOR_VERSION: &str = "0.1.0";
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
fn spec_names(v: &Value, key: &str) -> Result<Vec<String>> {
    texts(&v[key])
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
                Err(e) => probes.push(probe("executable", false, Some(&spec), Some(&e))),
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
                    Err(e) => probes.push(probe("config-dir", false, Some(path), Some(&e))),
                },
                Err(e) => probes.push(probe("config-dir", false, Some(path), Some(e))),
            }
        }
        if let Some(s) = p.get("service") {
            let kind = s["kind"].as_str().unwrap_or("service");
            match effects.service(s) {
                Ok(reading) => {
                    service_live = reading.is_present();
                    probes.push(probe("service", true, Some(kind), Some(reading.detail())));
                }
                Err(e) => probes.push(probe("service", false, Some(kind), Some(&e))),
            }
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
                Err(e) => probes.push(probe("env", false, Some(&spec), Some(&e))),
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
                    match effects.version(path, &args) {
                        Ok(version) => entry["version"] = json!(version),
                        Err(reason) => disclosure
                            .push(format!("{}: version probe failed ({reason})", d.slug())),
                    }
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
                let inventory = observe_inventory(
                    d,
                    &spec["inventory"],
                    probes,
                    live,
                    effects,
                    disclosure,
                    options,
                )?;
                for (k, v) in object(&inventory)? {
                    entry[k] = v.clone();
                }
            }
            out.push(entry);
        }
    }
    Ok(out)
}
fn observe_inventory(
    d: &HarnessDescriptor,
    declared: &Value,
    probes: &[Value],
    live: bool,
    effects: &mut dyn ProbeEffects,
    disclosure: &mut Vec<String>,
    options: &DetectionOptions,
) -> Result<Value> {
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
