use crate::wire::*;
use crate::{
    DetectionReport, Error, Fingerprint, HarnessDescriptor, HarnessSelf, Result,
    HARNESS_DETECTION_VERSION,
};
use actuation_stream::Timestamp;
use serde_json::{json, Map, Value};
use std::collections::HashSet;

/// Absence is the result of a completed measurement. A failed mechanism is
/// unavailable, not a measurement that found nothing.
#[derive(Clone, Debug, PartialEq)]
pub enum Observation<T> {
    Present(T),
    Absent,
    Unavailable(String),
}
#[derive(Clone, Debug, PartialEq)]
pub struct FileObservation {
    pub is_directory: bool,
    pub size: Option<u64>,
    pub modified_millis: Option<f64>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ServiceObservation {
    Present(String),
    Absent(String),
    Unverified(String),
    Unavailable(String),
}
impl ServiceObservation {
    pub fn is_present(&self) -> bool {
        matches!(self, Self::Present(_))
    }
    fn detail(&self) -> &str {
        match self {
            Self::Present(s) | Self::Absent(s) | Self::Unverified(s) | Self::Unavailable(s) => s,
        }
    }
}

/// Effects return measured facts, never receipt-shaped guesses. Provisioning,
/// model choice and authority are not operations of this observation port.
pub trait DetectionEffects {
    fn expand_home(&mut self, path: &str) -> Result<String>;
    fn resolve_executable(&mut self, names: &[String]) -> Observation<String>;
    fn stat(&mut self, path: &str) -> Observation<FileObservation>;
    fn fingerprint(&mut self, path: &str) -> Result<Fingerprint>;
    fn directory_count(&mut self, path: &str) -> Observation<u64>;
    fn version(&mut self, path: &str, args: &[String]) -> Result<String>;
    fn service(&mut self, _spec: &Value) -> ServiceObservation {
        ServiceObservation::Unverified(
            "service probe not implemented; declared, not verified".into(),
        )
    }
    /// None is an unsupported faculty, not an empty provider catalogue.
    fn http_json(&mut self, _url: &str) -> Option<Result<Value>> {
        None
    }
    fn environment_markers(&mut self, _names: &[String]) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}
#[derive(Clone, Debug)]
pub struct DetectionOptions {
    pub observed_at: Timestamp,
    pub probe_versions: bool,
    pub catalog_revision: Value,
}
impl DetectionOptions {
    pub fn new(catalog_revision: Value) -> Result<Self> {
        integer(&catalog_revision, "catalog_revision")?;
        Ok(Self {
            observed_at: Timestamp::now()?,
            probe_versions: false,
            catalog_revision,
        })
    }
}
fn probe(kind: &str, passed: bool, spec: Option<&str>, detail: Option<&str>) -> Value {
    let mut value = json!({"kind":kind,"result":if passed {"pass"}else{"fail"}});
    if let Some(s) = spec {
        value["spec"] = json!(s);
    }
    if let Some(s) = detail {
        value["detail"] = json!(s.chars().take(200).collect::<String>());
    }
    value
}
struct MeasuredBody {
    probes: Vec<Value>,
    executable: Option<String>,
    wanted: Option<String>,
    config_present: bool,
    service: Option<ServiceObservation>,
}
fn measure(d: &HarnessDescriptor, e: &mut impl DetectionEffects) -> Result<MeasuredBody> {
    let spec = d.probes();
    let wanted = spec
        .get("config-dir")
        .map(|v| text(&v["path"], "config-dir.path").and_then(|s| e.expand_home(s)))
        .transpose()?;
    let mut m = MeasuredBody {
        probes: Vec::new(),
        executable: None,
        wanted,
        config_present: false,
        service: None,
    };
    if let Some(x) = spec.get("executable") {
        let names = owned_strings(&x["names"], "executable.names")?;
        let label = names.join(" ");
        let (passed, detail) = match e.resolve_executable(&names) {
            Observation::Present(path) => {
                text(&json!(path), "resolved executable")?;
                m.executable = Some(path.clone());
                (true, path)
            }
            Observation::Absent => (true, "not found on PATH".into()),
            Observation::Unavailable(reason) => (false, reason),
        };
        m.probes
            .push(probe("executable", passed, Some(&label), Some(&detail)));
    }
    if let Some(wanted) = &m.wanted {
        let (passed, detail) = match e.stat(wanted) {
            Observation::Present(_) => {
                m.config_present = true;
                (true, format!("exists at {wanted}"))
            }
            Observation::Absent => (true, "not found".into()),
            Observation::Unavailable(reason) => (false, reason),
        };
        m.probes.push(probe(
            "config-dir",
            passed,
            spec["config-dir"]["path"].as_str(),
            Some(&detail),
        ));
    }
    if let Some(s) = spec.get("service") {
        let observed = e.service(s);
        m.probes.push(probe(
            "service",
            !matches!(observed, ServiceObservation::Unavailable(_)),
            Some(s["kind"].as_str().unwrap_or("service")),
            Some(observed.detail()),
        ));
        m.service = Some(observed);
    }
    if let Some(s) = spec.get("env") {
        let names = owned_strings(&s["any_of"], "env.any_of")?;
        let (passed, detail) = match e.environment_markers(&names) {
            Ok(markers) => (
                true,
                if markers.is_empty() {
                    "no marker set".into()
                } else {
                    markers
                        .iter()
                        .map(|n| format!("{n} set"))
                        .collect::<Vec<_>>()
                        .join(", ")
                },
            ),
            Err(error) => (false, error.to_string()),
        };
        m.probes
            .push(probe("env", passed, Some(&names.join(" ")), Some(&detail)));
    }
    Ok(m)
}
fn receipts(m: &MeasuredBody, e: &mut impl DetectionEffects) -> Value {
    let mut r = json!({});
    if let Some(path) = &m.executable {
        r["executable"] = json!(path);
        if let Ok(digest) = e.fingerprint(path) {
            r["sha256"] = json!(digest.as_str());
        }
        if let Observation::Present(s) = e.stat(path) {
            if !s.is_directory {
                if let Some(n) = s.modified_millis.filter(|n| n.is_finite()) {
                    let rounded = (n + 0.5).floor();
                    r["mtime"] = if rounded >= i64::MIN as f64 && rounded < i64::MAX as f64 {
                        json!(rounded as i64)
                    } else {
                        json!(rounded)
                    };
                }
                if let Some(n) = s.size {
                    r["size"] = json!(n);
                }
            }
        }
    } else if let Some(path) = &m.wanted {
        if let Observation::Present(_) = e.stat(path) {
            r["executable"] = json!(path);
            r["executable_is"] = json!("config-dir");
        }
    }
    r
}
fn inventory(
    d: &HarnessDescriptor,
    declared: &Value,
    m: &MeasuredBody,
    e: &mut impl DetectionEffects,
    notes: &mut Vec<String>,
    now: &Timestamp,
) -> Result<Value> {
    if !m
        .service
        .as_ref()
        .is_some_and(ServiceObservation::is_present)
    {
        let reason=m.service.as_ref().map_or_else(||"declared service inventory not read: no service probe ran".into(),|s|format!("declared service inventory not read: service probe did not prove a live endpoint ({})",s.detail()));
        return Ok(json!({"inventory_unavailable_reason":reason}));
    }
    let service = &d.probes()["service"];
    let base = text(
        present(service, "default_url").unwrap_or(&service["url"]),
        "service URL",
    )?;
    let route = text(&declared["route"], "inventory.route")?;
    let url = format!("{}{route}", base.strip_suffix('/').unwrap_or(base));
    let Some(answer) = e.http_json(&url) else {
        return Ok(
            json!({"inventory_unavailable_reason":format!("declared service inventory not read: no http-json probe effect available for {url}")}),
        );
    };
    let answer = match answer {
        Ok(a) => a,
        Err(err) => {
            notes.push(format!("{}: model inventory read failed ({err})", d.slug()));
            return Ok(
                json!({"inventory_unavailable_reason":format!("inventory read from {url} failed: {err}")}),
            );
        }
    };
    let collection = text(&declared["collection"], "inventory.collection")?;
    let Some(items) = answer.get(collection).and_then(Value::as_array) else {
        return Ok(
            json!({"inventory_unavailable_reason":format!("inventory read from {url} returned no \"{collection}\" array")}),
        );
    };
    let id_field = text(&declared["id_field"], "id_field")?;
    let aliases = present(declared, "also_id_fields")
        .map(|v| owned_strings(v, "also_id_fields"))
        .transpose()?
        .unwrap_or_default();
    let details = present(declared, "detail_fields")
        .map(|v| owned_strings(v, "detail_fields"))
        .transpose()?
        .unwrap_or_default();
    let mut seen = HashSet::new();
    let mut records = Vec::new();
    for item in items {
        let Some(id) = item
            .get(id_field)
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
        else {
            continue;
        };
        if !seen.insert(id) {
            continue;
        }
        let mut record = json!({"id":id});
        let mut also_seen = HashSet::new();
        let also: Vec<_> = aliases
            .iter()
            .filter_map(|f| item.get(f).and_then(Value::as_str))
            .filter(|s| !s.trim().is_empty() && *s != id && also_seen.insert(*s))
            .collect();
        if !also.is_empty() {
            record["also_known_as"] = json!(also);
        }
        for f in &details {
            // A provider detail cannot overwrite the identity field selected
            // by the descriptor. It remains detail, not a second identity.
            if f == "id" || f == "also_known_as" {
                continue;
            }
            if let Some(v) = item.get(f).filter(|v| v.is_string() || v.is_number()) {
                record[f] = v.clone();
            }
        }
        records.push(record);
    }
    Ok(
        json!({"inventory":records,"inventory_receipt":{"kind":declared["kind"],"source":url,"observed_at":now,"item_count":records.len()}}),
    )
}
fn facets(
    d: &HarnessDescriptor,
    m: &MeasuredBody,
    e: &mut impl DetectionEffects,
    notes: &mut Vec<String>,
    now: &Timestamp,
) -> Result<Vec<Value>> {
    let mut result = Vec::new();
    for (kind, f) in d.facets().into_iter().flatten() {
        let path = text(&f["path"], "facet.path")?;
        let expanded = e.expand_home(path)?;
        match e.stat(&expanded) {
            Observation::Present(s) => {
                let mut entry = json!({"kind":kind,"path":path,"exists":true});
                if s.is_directory {
                    match e.directory_count(&expanded) {
                        Observation::Present(count) => entry["count"] = json!(count),
                        Observation::Unavailable(reason) => {
                            notes.push(format!("{}: {kind} count unavailable ({reason})", d.slug()))
                        }
                        Observation::Absent => {}
                    }
                }
                if let Some(i) = present(f, "inventory") {
                    entry
                        .as_object_mut()
                        .unwrap()
                        .extend(object(&inventory(d, i, m, e, notes, now)?)?.clone());
                }
                result.push(entry);
            }
            Observation::Unavailable(reason) => notes.push(format!(
                "{}: {kind} observation unavailable ({reason})",
                d.slug()
            )),
            Observation::Absent => {}
        }
    }
    Ok(result)
}
/// One observation pass. `probe_versions` is deliberately opt-in because it
/// executes target code; plain presence detection never starts a harness.
pub fn detect(
    descriptors: &[HarnessDescriptor],
    effects: &mut impl DetectionEffects,
    options: &DetectionOptions,
) -> Result<DetectionReport> {
    integer(&options.catalog_revision, "catalog_revision")?;
    let mut entries = Vec::new();
    let mut notes = Vec::new();
    for d in descriptors {
        let m = measure(d, effects)?;
        let has_presence = m.executable.is_some()
            || m.config_present
            || m.service
                .as_ref()
                .is_some_and(ServiceObservation::is_present);
        // A marker read cannot turn a failed presence measurement into absence.
        let unavailable = (!has_presence
            && m.probes
                .iter()
                .any(|p| p["result"] == "fail" && p["kind"] != "env"))
            || (!m.probes.iter().any(|p| p["result"] == "pass")
                && m.probes.iter().any(|p| p["result"] == "fail"));
        let state = if unavailable {
            "unavailable"
        } else if has_presence {
            "detected"
        } else {
            "not-installed"
        };
        let mut entry = json!({"slug":d.slug(),"harness_ref":format!("harness/{}",d.slug()),"native_kind":d.native_kind(),"state":state,"probes":m.probes});
        if unavailable {
            entry["unavailable_reason"] = json!("all probes failed; could not run");
        }
        if has_presence && !unavailable {
            let r = receipts(&m, effects);
            // /v1 has no service-only receipt variant. Keep this limitation
            // explicit rather than inventing an executable from a status line.
            if r.as_object().unwrap().is_empty() {
                return Err(Error::new("service was observed but /v1 has no executable/config-dir receipt for this target"));
            }
            entry["receipts"] = r;
            let observed = facets(d, &m, effects, &mut notes, &options.observed_at)?;
            if !observed.is_empty() {
                entry["facets"] = json!(observed);
            }
            if options.probe_versions {
                if let Some(path) = &m.executable {
                    let args = present(&d.probes()["executable"], "version_args")
                        .map(|v| owned_strings(v, "version_args"))
                        .transpose()?
                        .unwrap_or_else(|| vec!["--version".into()]);
                    match effects.version(path, &args) {
                        Ok(v) => entry["version"] = json!(v),
                        Err(err) => {
                            notes.push(format!("{}: version probe failed ({err})", d.slug()))
                        }
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
    let mut record = json!({"schema":HARNESS_DETECTION_VERSION,"document":"detection","detection_ref":format!("detection:{}",options.observed_at.as_str()),"observed_at":options.observed_at,"catalog_revision":options.catalog_revision,"detector":{"implementation":"actuation surface-probes","version":"0.1.0"},"availability":if entries.iter().any(|e|e["state"]=="unavailable"){"partial"}else{"complete"},"harnesses":entries,"absent":absent});
    if !notes.is_empty() {
        record["disclosure"] = json!(notes);
    }
    DetectionReport::new(record)
}
/// Native context markers identify only the harness context. They never mint
/// Agent/Agency identity or acquire determining authority from process facts.
pub fn observe_self(
    descriptors: &[HarnessDescriptor],
    effects: &mut impl DetectionEffects,
    options: &DetectionOptions,
) -> Result<HarnessSelf> {
    let mut matched = Vec::new();
    for d in descriptors {
        let Some(env) = d.probes().get("env") else {
            continue;
        };
        let names = owned_strings(&env["any_of"], "env.any_of")?;
        if let Ok(markers) = effects.environment_markers(&names) {
            if !markers.is_empty() {
                matched.push(json!({"slug":d.slug(),"harness_ref":format!("harness/{}",d.slug()),"markers":markers}));
            }
        }
    }
    let mut ordinary = options.clone();
    ordinary.probe_versions = false;
    let detection = detect(descriptors, effects, &ordinary)?;
    let states: Map<String, Value> = detection
        .entries()
        .iter()
        .map(|e| (e["slug"].as_str().unwrap().into(), e["state"].clone()))
        .collect();
    HarnessSelf::new(
        json!({"schema":HARNESS_DETECTION_VERSION,"document":"self","self_ref":format!("self:{}",options.observed_at.as_str()),"observed_at":options.observed_at,"catalog_revision":options.catalog_revision,"resolved":if matched.len()==1{matched[0].clone()}else{Value::Null},"ambiguity":matched.len()>1,"matched":matched,"detection_ref":detection.reference(),"detection":{"states":states}}),
    )
}
