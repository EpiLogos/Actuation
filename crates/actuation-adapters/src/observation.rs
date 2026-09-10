use crate::wire::*;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;

pub const HARNESS_DETECTION_VERSION: &str = "actuation.harness-detection/v1";
pub const FACET_KINDS: &[&str] = &[
    "skills",
    "harness-compositions",
    "plugins",
    "hooks",
    "commands",
    "rules",
    "extensions",
    "agents",
    "settings",
    "config",
    "models",
];
pub const PROBE_KINDS: &[&str] = &["executable", "config-dir", "service", "env"];
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Presence {
    Detected,
    Unavailable,
    NotInstalled,
}

fn check_descriptor(v: &Value) -> Result<()> {
    object(v)?;
    exact(&v["schema"], HARNESS_DETECTION_VERSION, "schema")?;
    required_texts(v, &["slug", "native_kind"])?;
    optional_texts(v, &["edition", "native_owner", "summary"])?;
    optional_strings(v, "aliases")?;
    let probes = object(&v["probe"])?;
    if probes.is_empty() {
        return Err(Error::new("descriptor must declare a probe"));
    }
    for (kind, spec) in probes {
        if !PROBE_KINDS.contains(&kind.as_str()) {
            return Err(Error::new("unsupported probe kind"));
        }
        object(spec)?;
    }
    if let Some(env) = present(&v["probe"], "env") {
        if strings(&env["any_of"], "env.any_of")?.is_empty() {
            return Err(Error::new("env marker list cannot be empty"));
        }
    }
    if let Some(facets) = present(v, "facets") {
        for (kind, f) in object(facets)? {
            if !FACET_KINDS.contains(&kind.as_str()) {
                return Err(Error::new("unsupported facet kind"));
            }
            object(f)?;
            text(&f["path"], "facet.path")?;
            if let Some(i) = present(f, "inventory") {
                object(i)?;
                exact(&i["kind"], "http-json", "inventory.kind")?;
                exact(&i["from"], "service", "inventory.from")?;
                if present(&v["probe"], "service").is_none() {
                    return Err(Error::new(
                        "inventory needs the descriptor's own service probe",
                    ));
                }
                let route = text(&i["route"], "inventory.route")?;
                if !route.starts_with('/') {
                    return Err(Error::new("inventory route must be service-relative"));
                }
                required_texts(i, &["collection", "id_field"])?;
                optional_strings(i, "also_id_fields")?;
                optional_strings(i, "detail_fields")?;
            }
        }
    }
    let p = &v["provenance"];
    object(p)?;
    text(&p["authored_by"], "authored_by")?;
    optional_strings(p, "source_refs")?;
    optional_text(p, "accepted_by")?;
    integer(&p["catalog_revision"], "catalog_revision")
}
document!(HarnessDescriptor, check_descriptor);
impl HarnessDescriptor {
    pub fn slug(&self) -> &str {
        self.0["slug"].as_str().unwrap()
    }
    pub fn native_kind(&self) -> &str {
        self.0["native_kind"].as_str().unwrap()
    }
    pub fn probes(&self) -> &serde_json::Map<String, Value> {
        self.0["probe"].as_object().unwrap()
    }
    pub fn facets(&self) -> Option<&serde_json::Map<String, Value>> {
        self.0["facets"].as_object()
    }
}
fn check_inventory(f: &Value) -> Result<()> {
    let Some(items) = present(f, "inventory") else {
        if present(f, "inventory_receipt").is_some() {
            return Err(Error::new(
                "inventory receipt requires an observed inventory",
            ));
        }
        return optional_text(f, "inventory_unavailable_reason");
    };
    if present(f, "inventory_unavailable_reason").is_some() {
        return Err(Error::new("inventory cannot be observed and unavailable"));
    }
    let items = list(items, "inventory")?;
    let mut seen = HashSet::new();
    for i in items {
        object(i)?;
        let id = text(&i["id"], "inventory.id")?;
        if !seen.insert(id) {
            return Err(Error::new("duplicate inventory identity"));
        }
        optional_strings(i, "also_known_as")?;
    }
    let r = &f["inventory_receipt"];
    object(r)?;
    exact(&r["kind"], "http-json", "inventory_receipt.kind")?;
    text(&r["source"], "inventory_receipt.source")?;
    timestamp(&r["observed_at"], "inventory_receipt.observed_at")?;
    if r["item_count"].as_f64() != Some(items.len() as f64) {
        return Err(Error::new(
            "inventory receipt count does not match observation",
        ));
    }
    Ok(())
}
fn optional_array<'a>(v: &'a Value, key: &str) -> Result<&'a [Value]> {
    match present(v, key) {
        None => Ok(&[]),
        Some(v) => Ok(list(v, key)?),
    }
}
fn check_detection(v: &Value) -> Result<()> {
    object(v)?;
    exact(&v["schema"], HARNESS_DETECTION_VERSION, "schema")?;
    text(&v["detection_ref"], "detection_ref")?;
    timestamp(&v["observed_at"], "observed_at")?;
    integer(&v["catalog_revision"], "catalog_revision")?;
    let d = &v["detector"];
    object(d)?;
    required_texts(d, &["implementation", "version"])?;
    let mut seen = HashSet::new();
    let mut absent = Vec::new();
    let mut unavailable = false;
    for e in list(&v["harnesses"], "harnesses")? {
        object(e)?;
        let slug = text(&e["slug"], "slug")?;
        exact(&e["harness_ref"], &format!("harness/{slug}"), "harness_ref")?;
        optional_texts(e, &["native_kind", "version"])?;
        if !seen.insert(slug) {
            return Err(Error::new("duplicate observed harness slug"));
        }
        let state: Presence = serde_json::from_value(e["state"].clone())?;
        let probes = optional_array(e, "probes")?;
        let facets = optional_array(e, "facets")?;
        match state {
            Presence::Unavailable => {
                text(&e["unavailable_reason"], "unavailable_reason")?;
                unavailable = true;
            }
            Presence::Detected => {
                if !probes.iter().any(|p| p["result"] == "pass") {
                    return Err(Error::new("detected requires a passing probe"));
                }
                let r = &e["receipts"];
                object(r)?;
                text(&r["executable"], "receipts.executable")?;
                if let Some(h) = present(r, "sha256") {
                    fingerprint(h)?;
                }
            }
            Presence::NotInstalled => {
                if e["probes"].as_array().is_some_and(|a| a.is_empty())
                    || facets.iter().any(|f| truthy(&f["exists"]))
                {
                    return Err(Error::new(
                        "absence requires probes and cannot carry observed facets",
                    ));
                }
                absent.push(json!(slug));
            }
        }
        for f in facets {
            object(f)?;
            choice(&f["kind"], FACET_KINDS, "facet.kind")?;
            text(&f["path"], "facet.path")?;
            boolean(&f["exists"], "facet.exists")?;
            check_inventory(f)?;
        }
        for p in probes {
            object(p)?;
            choice(&p["kind"], PROBE_KINDS, "probe.kind")?;
            choice(&p["result"], &["pass", "fail"], "probe.result")?;
            optional_texts(p, &["spec", "detail"])?;
        }
    }
    if present(v, "absent").unwrap_or(&json!([])) != &json!(absent) {
        return Err(Error::new(
            "absent must list exactly the absent slugs in order",
        ));
    }
    exact(
        &v["availability"],
        if unavailable { "partial" } else { "complete" },
        "availability",
    )?;
    optional_strings(v, "disclosure")
}
document!(DetectionReport, check_detection);
impl DetectionReport {
    pub fn entries(&self) -> &[Value] {
        self.0["harnesses"].as_array().unwrap()
    }
    pub fn entry(&self, slug: &str) -> Option<&Value> {
        self.entries().iter().find(|e| e["slug"] == slug)
    }
    pub fn presence(&self, slug: &str) -> Option<Presence> {
        self.entry(slug)
            .and_then(|e| serde_json::from_value(e["state"].clone()).ok())
    }
    pub fn reference(&self) -> &str {
        self.0["detection_ref"].as_str().unwrap()
    }
}
fn check_self(v: &Value) -> Result<()> {
    object(v)?;
    exact(&v["schema"], HARNESS_DETECTION_VERSION, "schema")?;
    exact(&v["document"], "self", "document")?;
    text(&v["self_ref"], "self_ref")?;
    timestamp(&v["observed_at"], "observed_at")?;
    integer(&v["catalog_revision"], "catalog_revision")?;
    let matched = list(&v["matched"], "matched")?;
    for m in matched {
        object(m)?;
        let slug = text(&m["slug"], "slug")?;
        exact(&m["harness_ref"], &format!("harness/{slug}"), "harness_ref")?;
        strings(&m["markers"], "markers")?;
    }
    if matched.len() == 1 {
        if v["resolved"]["slug"] != matched[0]["slug"]
            || v["resolved"]["harness_ref"] != matched[0]["harness_ref"]
        {
            return Err(Error::new("self resolution must name the single match"));
        }
    } else if present(v, "resolved").is_some() {
        return Err(Error::new("self resolution requires exactly one match"));
    }
    boolean(&v["ambiguity"], "ambiguity")?;
    text(&v["detection_ref"], "detection_ref")?;
    // The legacy /v1 reader permits an object or array here. Native production
    // always emits the explicit same-run state map; do not silently break old readers.
    if !v["detection"].is_object() && !v["detection"].is_array() {
        return Err(Error::new("self requires a detection cross-check"));
    }
    Ok(())
}
document!(HarnessSelf, check_self);
impl HarnessSelf {
    pub fn resolved_slug(&self) -> Option<&str> {
        self.0["resolved"]["slug"].as_str()
    }
    pub fn ambiguous(&self) -> bool {
        self.0["ambiguity"].as_bool().unwrap()
    }
}
fn check_catalog(v: &Value) -> Result<()> {
    object(v)?;
    exact(&v["schema"], HARNESS_DETECTION_VERSION, "schema")?;
    exact(&v["document"], "catalog", "document")?;
    integer(&v["catalog_revision"], "catalog_revision")?;
    let descriptors = list(&v["descriptors"], "descriptors")?;
    if descriptors.is_empty() {
        return Err(Error::new("catalogue must not be empty"));
    }
    let mut seen = HashSet::new();
    for d in descriptors {
        check_descriptor(d)?;
        if !seen.insert(&d["slug"]) {
            return Err(Error::new("duplicate catalogue slug"));
        }
    }
    Ok(())
}
document!(HarnessCatalogDocument, check_catalog);
