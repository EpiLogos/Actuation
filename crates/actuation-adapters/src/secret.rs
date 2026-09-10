//! Fingerprint-only observation of supplied secret-source declarations.
//! This module observes drift; it neither resolves secret values for a model
//! nor changes authored security policy or material placement.
use crate::wire::*;
use crate::{Error, Result};
use actuation_stream::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

pub const SECRET_DETECTION_VERSION: &str = "actuation.secret-detection/v1";
pub const SECRET_PROBE_KINDS: &[&str] = &["env", "file-pattern", "cli-presence", "vault-item"];
const FORBIDDEN: &[&str] = &[
    "value",
    "material",
    "secret_value",
    "secretvalue",
    "plaintext",
    "secret",
    "password",
    "token_value",
    "api_key",
    "apikey",
    "credential",
];
pub fn assert_fingerprint_only(v: &Value) -> Result<()> {
    match v {
        Value::Object(m) => {
            for (key, child) in m {
                if FORBIDDEN.contains(&key.to_ascii_lowercase().as_str()) {
                    return Err(Error::new("scan contains a forbidden value-shaped key"));
                }
                assert_fingerprint_only(child)?;
            }
        }
        Value::Array(a) => {
            for child in a {
                assert_fingerprint_only(child)?;
            }
        }
        _ => {}
    };
    Ok(())
}
fn check_source(v: &Value) -> Result<()> {
    object(v)?;
    exact(&v["schema"], SECRET_DETECTION_VERSION, "schema")?;
    exact(&v["document"], "descriptor", "document")?;
    text(&v["slug"], "slug")?;
    choice(
        &v["source_kind"],
        &[
            "op-item",
            "keychain-entry",
            "varlock-blob",
            "env-var",
            "plaintext-file",
        ],
        "source_kind",
    )?;
    strings(&v["ref_schemes"], "ref_schemes")?;
    let probes = object(&v["probe"])?;
    if probes.is_empty() {
        return Err(Error::new("secret descriptor must declare probes"));
    }
    for (kind, spec) in probes {
        if !SECRET_PROBE_KINDS.contains(&kind.as_str()) {
            return Err(Error::new("unsupported secret-source probe kind"));
        }
        object(spec)?;
        match kind.as_str() {
            "env" => {
                optional_strings(spec, "names")?;
                optional_text(spec, "name_pattern")?;
                if present(spec, "names").is_none() && present(spec, "name_pattern").is_none() {
                    return Err(Error::new("env probe needs names or name_pattern"));
                }
            }
            "file-pattern" => {
                strings(&spec["patterns"], "patterns")?;
                optional_strings(spec, "roots")?;
            }
            "cli-presence" => {
                strings(&spec["names"], "names")?;
            }
            "vault-item" => {
                text(&spec["item_ref"], "item_ref")?;
            }
            _ => unreachable!(),
        }
    }
    let p = &v["provenance"];
    object(p)?;
    text(&p["authored_by"], "authored_by")?;
    optional_strings(p, "source_refs")?;
    integer(&p["catalog_revision"], "catalog_revision")
}
document!(SecretSourceDescriptor, check_source);
impl SecretSourceDescriptor {
    pub fn slug(&self) -> &str {
        self.0["slug"].as_str().unwrap()
    }
}
fn check_secret_catalog(v: &Value) -> Result<()> {
    object(v)?;
    exact(&v["schema"], SECRET_DETECTION_VERSION, "schema")?;
    exact(&v["document"], "catalog", "document")?;
    integer(&v["catalog_revision"], "catalog_revision")?;
    let ds = list(&v["descriptors"], "descriptors")?;
    if ds.is_empty() {
        return Err(Error::new("secret-source catalogue cannot be empty"));
    }
    let mut seen = HashSet::new();
    for d in ds {
        check_source(d)?;
        if !seen.insert(&d["slug"]) {
            return Err(Error::new("duplicate secret-source slug"));
        }
    }
    assert_fingerprint_only(v)
}
document!(SecretSourceCatalog, check_secret_catalog);
impl SecretSourceCatalog {
    pub fn descriptors(&self) -> Result<Vec<SecretSourceDescriptor>> {
        self.0["descriptors"]
            .as_array()
            .unwrap()
            .iter()
            .cloned()
            .map(SecretSourceDescriptor::new)
            .collect()
    }
}
fn check_scan(v: &Value) -> Result<()> {
    object(v)?;
    exact(&v["schema"], SECRET_DETECTION_VERSION, "schema")?;
    exact(&v["document"], "scan", "document")?;
    text(&v["scan_ref"], "scan_ref")?;
    timestamp(&v["observed_at"], "observed_at")?;
    integer(&v["catalog_revision"], "catalog_revision")?;
    object(&v["scanner"])?;
    required_texts(&v["scanner"], &["implementation", "version"])?;
    let mut seen = HashSet::new();
    let mut violations = Vec::new();
    let mut unavailable = false;
    for e in list(&v["sources"], "sources")? {
        object(e)?;
        let slug = text(&e["slug"], "slug")?;
        exact(
            &e["source_ref"],
            &format!("secret-source/{slug}"),
            "source_ref",
        )?;
        if !seen.insert(slug) {
            return Err(Error::new("duplicate scan identity"));
        }
        let state = choice(
            &e["state"],
            &["verified", "violation", "absent", "unavailable"],
            "scan.state",
        )?;
        if state == "unavailable" {
            text(&e["unavailable_reason"], "unavailable_reason")?;
            unavailable = true;
        }
        if state == "violation" {
            choice(
                &e["violation_class"],
                &[
                    "stray-plaintext",
                    "uncentralised-env",
                    "legacy-env-ref",
                    "unknown",
                ],
                "violation_class",
            )?;
            text(&e["centralise_to"], "centralise_to")?;
            violations.push(json!(slug));
        } else if present(e, "violation_class").is_some() {
            return Err(Error::new("only violations carry a violation class"));
        }
        let empty = json!([]);
        let probes = list(present(e, "probes").unwrap_or(&empty), "probes")?;
        if state == "verified" && !probes.iter().any(|p| p["result"] == "pass") {
            return Err(Error::new("verified needs a passing probe"));
        }
        for p in probes {
            object(p)?;
            choice(&p["kind"], SECRET_PROBE_KINDS, "probe.kind")?;
            choice(&p["result"], &["pass", "fail"], "probe.result")?;
            optional_texts(p, &["spec", "detail"])?;
        }
        for item in list(present(e, "evidence").unwrap_or(&empty), "evidence")? {
            object(item)?;
            text(&item["where"], "evidence.where")?;
            fingerprint(&item["fingerprint_sha256"])?;
            if let Some(n) = present(item, "byte_length") {
                integer(n, "byte_length")?;
                if n.as_f64().unwrap() < 0.0 {
                    return Err(Error::new("byte length cannot be negative"));
                }
            }
        }
    }
    if present(v, "violations").unwrap_or(&json!([])) != &json!(violations) {
        return Err(Error::new(
            "violations must list exactly the violated source identities in order",
        ));
    }
    exact(
        &v["coverage"],
        if unavailable { "partial" } else { "complete" },
        "coverage",
    )?;
    optional_strings(v, "disclosure")?;
    assert_fingerprint_only(v)
}
document!(SecretScan, check_scan);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Fingerprint(String);
impl Fingerprint {
    pub fn of(bytes: &[u8]) -> Self {
        Self(format!("{:x}", Sha256::digest(bytes)))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<String> for Fingerprint {
    type Error = Error;
    fn try_from(s: String) -> Result<Self> {
        fingerprint(&json!(s))?;
        Ok(Self(s))
    }
}
impl From<Fingerprint> for String {
    fn from(s: Fingerprint) -> String {
        s.0
    }
}
/// No secret-value slot exists in an effect's evidence type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FingerprintEvidence {
    pub location: String,
    pub fingerprint: Fingerprint,
    pub byte_length: Option<u64>,
}
impl FingerprintEvidence {
    pub fn new(location: String, bytes: &[u8]) -> Self {
        Self {
            location,
            fingerprint: Fingerprint::of(bytes),
            byte_length: Some(bytes.len() as u64),
        }
    }
    fn wire(&self) -> Value {
        let mut v = json!({"where":self.location,"fingerprint_sha256":self.fingerprint});
        if let Some(n) = self.byte_length {
            v["byte_length"] = json!(n);
        }
        v
    }
}
#[derive(Clone, Debug, Default)]
pub struct SecretFinding {
    pub evidence: Vec<FingerprintEvidence>,
    pub found: bool,
    pub vault_item: Option<(Option<String>, Option<String>)>,
    pub truncated: bool,
    pub files_scanned: u64,
}
/// Effects return only metadata and fingerprints. An unreadable/truncated
/// location is unavailability, never a proof of absence.
pub trait SecretEffects {
    fn probe(&mut self, kind: &str, spec: &Value) -> Option<Result<SecretFinding>>;
}
#[derive(Clone, Debug)]
pub struct SecretScanOptions {
    pub roots: Vec<String>,
    pub scanner_implementation: String,
    pub scanner_version: String,
    pub scan_ref: String,
    pub observed_at: Timestamp,
}
impl SecretScanOptions {
    pub fn new(roots: Vec<String>) -> Result<Self> {
        let t = Timestamp::now()?;
        Ok(Self {
            roots,
            scanner_implementation: "actuation secret-scan".into(),
            scanner_version: "0.1.0".into(),
            scan_ref: format!(
                "secret-scan:actuation secret-scan:{}",
                t.unix_nanos() / 1_000_000
            ),
            observed_at: t,
        })
    }
}
const ENV_PATTERN: &str = "(KEY|TOKEN|SECRET|PASSWORD|PASS|CREDENTIAL)";
const FILE_PATTERNS: &[&str] = &[
    "client_secret*.json",
    "credentials*.json",
    "*.pem",
    "*.p12",
    "*.pfx",
    "id_rsa",
    "id_ed25519",
    "id_ecdsa",
];
const CENTRALISE: &str = "op://Central/central-security";
fn slugify(prefix: &str, name: &str) -> String {
    let mut base = String::new();
    for c in name.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            base.push(c);
        } else if !base.ends_with('-') {
            base.push('-');
        }
    }
    let base = base.trim_matches('-');
    let short = &Fingerprint::of(name.as_bytes()).0[..8];
    format!(
        "{prefix}-{}-{short}",
        base.chars().take(48).collect::<String>()
    )
}
fn run_declared(d: &SecretSourceDescriptor, effects: &mut dyn SecretEffects) -> Result<Value> {
    let mut probes = Vec::new();
    let mut evidence = Vec::new();
    let mut any_match = false;
    let mut unavailable = None;
    for (kind, spec) in object(&d.0["probe"])? {
        let mut p = json!({"kind":kind,"result":"pass","spec":serde_json::to_string(spec)?});
        match effects.probe(kind, spec) {
            None => {
                unavailable = Some(format!("no effect registered for probe kind {kind}"));
                p["result"] = json!("fail");
                p["detail"] = json!("effect unavailable");
            }
            Some(Err(error)) => {
                unavailable = Some(error.to_string());
                p["result"] = json!("fail");
                p["detail"] = json!(error.to_string());
            }
            Some(Ok(f)) => {
                if f.truncated {
                    unavailable = Some(format!(
                        "{kind} probe was incomplete after {} files",
                        f.files_scanned
                    ));
                    p["result"] = json!("fail");
                    p["detail"] = json!(unavailable);
                }
                match kind.as_str() {
                    "env" | "file-pattern" => {
                        any_match |= !f.evidence.is_empty();
                        evidence.extend(f.evidence.iter().map(FingerprintEvidence::wire));
                    }
                    "vault-item" => {
                        any_match = f.found;
                        if let Some((id, vault)) = f.vault_item.filter(|_| f.found) {
                            p["detail"] = json!(format!(
                                "item {} in vault {}",
                                id.as_deref().unwrap_or("?"),
                                vault.as_deref().unwrap_or("?")
                            ));
                        }
                    }
                    "cli-presence" => {
                        any_match = f.found;
                    }
                    _ => {}
                }
            }
        };
        probes.push(p);
    }
    let mut v =
        json!({"slug":d.slug(),"source_ref":format!("secret-source/{}",d.slug()),"probes":probes});
    if let Some(reason) = unavailable {
        v["state"] = json!("unavailable");
        v["unavailable_reason"] = json!(reason);
        if !evidence.is_empty() {
            v["evidence"] = json!(evidence);
        }
    } else if any_match {
        v["state"] = json!("verified");
        if !evidence.is_empty() {
            v["evidence"] = json!(evidence);
        }
    } else {
        v["state"] = json!("absent");
    }
    Ok(v)
}
fn unavailable(prefix: &str, name: &str, kind: &str, spec: &str, reason: String) -> Value {
    let slug = slugify(prefix, name);
    json!({"slug":slug,"source_ref":format!("secret-source/{slug}"),"state":"unavailable","unavailable_reason":reason,"probes":[{"kind":kind,"result":"fail","spec":spec}]})
}
fn effect(e: &mut dyn SecretEffects, kind: &str, spec: &Value) -> Result<SecretFinding> {
    e.probe(kind, spec)
        .unwrap_or_else(|| Err(Error::new(format!("no effect registered for {kind}"))))
}
pub fn scan_secret_sources(
    catalog: &SecretSourceCatalog,
    options: &SecretScanOptions,
    effects: &mut dyn SecretEffects,
) -> Result<SecretScan> {
    let descriptors = catalog.descriptors()?;
    let mut declared_env = HashSet::new();
    for d in &descriptors {
        if let Some(names) = present(&d.0["probe"]["env"], "names") {
            for n in strings(names, "names")? {
                declared_env.insert(n.to_owned());
            }
        }
    }
    let mut sources = descriptors
        .iter()
        .map(|d| run_declared(d, effects))
        .collect::<Result<Vec<_>>>()?;
    let declared_paths: HashSet<String> = sources
        .iter()
        .flat_map(|e| e["evidence"].as_array().into_iter().flatten())
        .filter_map(|i| {
            i["where"]
                .as_str()
                .filter(|s| s.starts_with('/'))
                .map(str::to_owned)
        })
        .collect();
    match effect(effects, "env", &json!({"name_pattern":ENV_PATTERN})) {
        Ok(f) => {
            for item in f.evidence {
                if declared_env.contains(&item.location) {
                    continue;
                }
                let slug = slugify("env", &item.location);
                sources.push(json!({"slug":slug,"source_ref":format!("secret-source/{slug}"),"state":"violation","violation_class":"uncentralised-env","centralise_to":CENTRALISE,"probes":[{"kind":"env","result":"pass","spec":format!("name_pattern {ENV_PATTERN}")}],"evidence":[item.wire()]}));
            }
        }
        Err(e) => sources.push(unavailable(
            "env",
            "env-discovery-unavailable",
            "env",
            &format!("name_pattern {ENV_PATTERN}"),
            e.to_string(),
        )),
    }
    match effect(
        effects,
        "file-pattern",
        &json!({"patterns":FILE_PATTERNS,"roots":options.roots}),
    ) {
        Err(e) => sources.push(unavailable(
            "file",
            "file-discovery-unavailable",
            "file-pattern",
            &FILE_PATTERNS.join(","),
            e.to_string(),
        )),
        Ok(f) if f.truncated => sources.push(unavailable(
            "file",
            "file-discovery-truncated",
            "file-pattern",
            &FILE_PATTERNS.join(","),
            format!(
                "file walk hit its {}-file budget before completing — discovery results incomplete",
                f.files_scanned
            ),
        )),
        Ok(f) => {
            for item in f.evidence {
                if declared_paths.contains(&item.location) {
                    continue;
                }
                let slug = slugify("file", &item.location);
                sources.push(json!({"slug":slug,"source_ref":format!("secret-source/{slug}"),"state":"violation","violation_class":"stray-plaintext","centralise_to":CENTRALISE,"probes":[{"kind":"file-pattern","result":"pass","spec":FILE_PATTERNS.join(",")}],"evidence":[item.wire()]}));
            }
        }
    }
    let violations: Vec<_> = sources
        .iter()
        .filter(|e| e["state"] == "violation")
        .map(|e| e["slug"].clone())
        .collect();
    let partial = sources.iter().any(|e| e["state"] == "unavailable");
    SecretScan::new(
        json!({"schema":SECRET_DETECTION_VERSION,"document":"scan","scan_ref":options.scan_ref,"observed_at":options.observed_at,"catalog_revision":catalog.0["catalog_revision"],"scanner":{"implementation":options.scanner_implementation,"version":options.scanner_version},"sources":sources,"violations":violations,"coverage":if partial {"partial"}else{"complete"},"disclosure":["Evidence is fingerprint-only (SHA-256 + byte length + location). No secret value is representable in this record."]}),
    )
}
