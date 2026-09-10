//! Fingerprint-only read-only observations of supplied secret-source surfaces.
//! Central owns secret references, governance and credential resolution. These
//! effects observe existing material; they neither acquire credentials for an
//! act nor move them. Source values never leave the native effect boundary.
use crate::{admission::*, effects::*, Error, NativeCatalog, Result};
use actuation_stream::Timestamp;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Read,
    path::Path,
};

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
const CENTRALISE_TO: &str = "op://Central/central-security";
const SKIPPED: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "dist",
    "build",
    ".next",
    "__pycache__",
    ".venv",
    "venv",
];

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FingerprintEvidence {
    #[serde(rename = "where")]
    pub location: String,
    pub fingerprint_sha256: String,
    pub byte_length: u64,
}
#[derive(Clone, Debug, Default)]
pub struct SecretObservation {
    pub present: bool,
    pub evidence: Vec<FingerprintEvidence>,
    pub detail: Option<String>,
    pub truncated: bool,
    pub files_scanned: u64,
}
pub trait SecretEffects {
    fn expand_root(&mut self, path: &str) -> ProbeResult<String>;
    fn inspect(&mut self, kind: &str, specification: &Value) -> ProbeResult<SecretObservation>;
}
impl SecretEffects for NativeEffects {
    fn expand_root(&mut self, path: &str) -> ProbeResult<String> {
        self.expand_home(path)
    }
    fn inspect(&mut self, kind: &str, spec: &Value) -> ProbeResult<SecretObservation> {
        match kind {
            "env" => {
                let names: Vec<_> = if let Some(a) = spec["names"].as_array() {
                    a.iter()
                        .map(|v| {
                            v.as_str()
                                .ok_or_else(|| "invalid environment name".to_owned())
                                .map(str::to_owned)
                        })
                        .collect::<ProbeResult<_>>()?
                } else {
                    let pattern = Regex::new(
                        spec["name_pattern"]
                            .as_str()
                            .ok_or_else(|| "missing environment name pattern".to_owned())?,
                    )
                    .map_err(|_| "unsupported or invalid environment name pattern".to_owned())?;
                    self.environment
                        .keys()
                        .filter(|k| pattern.is_match(k))
                        .cloned()
                        .collect()
                };
                let mut names = names;
                names.sort();
                names.dedup();
                let evidence: Vec<_> = names
                    .iter()
                    .filter_map(|name| {
                        self.environment
                            .get(name)
                            .filter(|v| !v.is_empty())
                            .map(|secret| FingerprintEvidence {
                                location: name.clone(),
                                fingerprint_sha256: format!(
                                    "{:x}",
                                    Sha256::digest(secret.as_bytes())
                                ),
                                byte_length: secret.len() as u64,
                            })
                    })
                    .collect();
                Ok(SecretObservation {
                    present: !evidence.is_empty(),
                    evidence,
                    ..Default::default()
                })
            }
            "file-pattern" => self.fingerprint_files(spec),
            "cli-presence" => {
                let names = crate::admission::texts(&spec["names"]).map_err(|e| e.to_string())?;
                let Some(path) = self.resolve_executable(&names)? else {
                    return Ok(SecretObservation::default());
                };
                // Version failure cannot erase a separately observed executable.
                let _version = self.version(&path, &["--version".into()]);
                Ok(SecretObservation {
                    present: true,
                    ..Default::default()
                })
            }
            "vault-item" => {
                let item_ref = spec["item_ref"]
                    .as_str()
                    .filter(|v| !v.is_empty())
                    .ok_or_else(|| "missing vault item ref".to_owned())?;
                let program = self
                    .resolve_executable(&["op".into()])?
                    .ok_or_else(|| "vault metadata mechanism is unavailable".to_owned())?;
                let run = self.process(
                    &program,
                    &[
                        "item".into(),
                        "get".into(),
                        item_ref.into(),
                        "--format".into(),
                        "json".into(),
                    ],
                    1024 * 1024,
                )?;
                if run.code != Some(0) {
                    let stderr = String::from_utf8_lossy(&run.stderr).to_lowercase();
                    if stderr.contains("not found") && !stderr.contains("not signed in") {
                        return Ok(SecretObservation::default());
                    }
                    return Err(format!(
                        "vault metadata unavailable (exit {:?}); no credential material captured",
                        run.code
                    ));
                }
                let v: Value = serde_json::from_slice(&run.stdout)
                    .map_err(|_| "invalid vault metadata JSON".to_owned())?;
                // Re-emit only the two pre-existing metadata facts needed by
                // the receipt. Field contents, provider stderr and values are
                // deliberately impossible to copy through this result type.
                Ok(SecretObservation {
                    present: true,
                    detail: Some(format!(
                        "item {} in vault {}",
                        v["id"].as_str().unwrap_or("?"),
                        v["vault"]["name"].as_str().unwrap_or("?")
                    )),
                    ..Default::default()
                })
            }
            _ => Err(format!("no effect registered for probe kind {kind}")),
        }
    }
}
impl NativeEffects {
    fn fingerprint_files(&mut self, spec: &Value) -> ProbeResult<SecretObservation> {
        let patterns = texts(&spec["patterns"]).map_err(|e| e.to_string())?;
        let patterns: Vec<_> = patterns
            .iter()
            .map(|p| {
                Regex::new(&format!(
                    "^{}$",
                    p.split('*')
                        .map(regex::escape)
                        .collect::<Vec<_>>()
                        .join(".*")
                ))
                .expect("escaped glob")
            })
            .collect();
        let roots = if spec["roots"].is_null() {
            vec!["~".into()]
        } else {
            texts(&spec["roots"]).map_err(|e| e.to_string())?
        };
        let budget = spec["max_files"].as_u64().unwrap_or(500_000);
        let mut result = SecretObservation::default();
        for root in roots {
            let root = self.expand_home(&root)?;
            fingerprint_walk(Path::new(&root), 0, budget, &patterns, &mut result)?;
        }
        result.evidence.sort_by(|a, b| a.location.cmp(&b.location));
        result.present = !result.evidence.is_empty();
        Ok(result)
    }
}
fn fingerprint_walk(
    root: &Path,
    depth: u8,
    budget: u64,
    patterns: &[Regex],
    result: &mut SecretObservation,
) -> ProbeResult<()> {
    if result.files_scanned >= budget || depth > 6 {
        result.truncated = true;
        return Ok(());
    }
    let entries = match fs::read_dir(root) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            return Err(format!(
                "file discovery cannot read a directory: {}",
                e.kind()
            ))
        }
    };
    let mut entries = entries
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| format!("file discovery could not complete: {}", e.kind()))?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        if result.files_scanned >= budget {
            result.truncated = true;
            break;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        if ty.is_dir() {
            if !SKIPPED.contains(&name.as_str()) {
                fingerprint_walk(&entry.path(), depth + 1, budget, patterns, result)?;
            }
            continue;
        }
        result.files_scanned += 1;
        if !ty.is_file() || !patterns.iter().any(|p| p.is_match(&name)) {
            continue;
        }
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let mut f = options.open(entry.path()).map_err(|e| {
            format!(
                "matching secret-source file cannot be observed: {}",
                e.kind()
            )
        })?;
        let m = f.metadata().map_err(|e| e.to_string())?;
        if !m.is_file() {
            return Err("matching file changed kind during observation".into());
        }
        // Bound a single material read as well as the directory walk.
        if m.len() > 64 * 1024 * 1024 {
            return Err("secret-source file exceeds 64 MiB fingerprint budget".into());
        }
        let mut h = Sha256::new();
        let mut buf = [0u8; 65536];
        let mut bytes = 0u64;
        loop {
            let n = f
                .read(&mut buf)
                .map_err(|e| format!("secret fingerprint unavailable: {}", e.kind()))?;
            if n == 0 {
                break;
            }
            bytes += n as u64;
            if bytes > 64 * 1024 * 1024 {
                return Err("secret-source file grew beyond fingerprint budget".into());
            }
            h.update(&buf[..n]);
        }
        if bytes != m.len() {
            return Err("secret-source file changed during fingerprint observation".into());
        }
        result.evidence.push(FingerprintEvidence {
            location: entry.path().to_string_lossy().into_owned(),
            fingerprint_sha256: format!("{:x}", h.finalize()),
            byte_length: bytes,
        });
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct ScanOptions {
    pub roots: Vec<String>,
    pub observed_at: Timestamp,
    pub scanner_implementation: String,
    pub scanner_version: String,
}
impl ScanOptions {
    pub fn now(roots: Vec<String>) -> Result<Self> {
        Ok(Self {
            roots,
            observed_at: Timestamp::now()?,
            scanner_implementation: "actuation secret-scan".into(),
            scanner_version: "0.1.0".into(),
        })
    }
}
fn slugify(prefix: &str, name: &str) -> String {
    let re = Regex::new("[^a-z0-9]+").expect("constant regex");
    let lower = name.to_lowercase();
    let cleaned = re.replace_all(&lower, "-");
    let cleaned = cleaned.trim_matches('-');
    let short = format!("{:x}", Sha256::digest(name.as_bytes()));
    format!(
        "{prefix}-{}-{}",
        cleaned.chars().take(48).collect::<String>(),
        &short[..8]
    )
}
fn declared(descriptor: &Value, effects: &mut dyn SecretEffects) -> Result<Value> {
    let mut probes = Vec::new();
    let mut evidence = Vec::new();
    let mut unavailable = None;
    let mut any = false;
    for (kind, spec) in object(&descriptor["probe"])? {
        let mut probe = json!({"kind":kind,"result":"pass","spec":serde_json::to_string(spec)?});
        match effects.inspect(kind, spec) {
            Err(e) => {
                probe["result"] = json!("fail");
                probe["detail"] = json!(e);
                unavailable = Some(e);
            }
            Ok(reading) => {
                any |= reading.present;
                evidence.extend(reading.evidence);
                if let Some(detail) = reading.detail {
                    probe["detail"] = json!(detail);
                }
                if reading.truncated {
                    unavailable = Some(format!(
                        "declared file walk incomplete after {} files",
                        reading.files_scanned
                    ));
                }
            }
        }
        probes.push(probe);
    }
    let slug = text(&descriptor["slug"])?;
    let mut out = json!({"slug":slug,"source_ref":format!("secret-source/{slug}"),"probes":probes});
    if let Some(reason) = unavailable {
        out["state"] = json!("unavailable");
        out["unavailable_reason"] = json!(reason);
    } else if any {
        out["state"] = json!("verified");
        if !evidence.is_empty() {
            out["evidence"] = json!(evidence);
        }
    } else {
        out["state"] = json!("absent");
    }
    Ok(out)
}
fn unavailable_entry(prefix: &str, label: &str, kind: &str, spec: &str, reason: String) -> Value {
    let slug = slugify(prefix, label);
    json!({"slug":slug,"source_ref":format!("secret-source/{slug}"),"state":"unavailable","unavailable_reason":reason,"probes":[{"kind":kind,"result":"fail","spec":spec}]})
}
fn violation(
    prefix: &str,
    kind: &str,
    spec: &str,
    class: &str,
    evidence: FingerprintEvidence,
) -> Value {
    let slug = slugify(prefix, &evidence.location);
    json!({"slug":slug,"source_ref":format!("secret-source/{slug}"),"state":"violation","violation_class":class,"centralise_to":CENTRALISE_TO,"probes":[{"kind":kind,"result":"pass","spec":spec}],"evidence":[evidence]})
}
/// Run the supplied source/catalogue observation. Returned suggestions are
/// correlations to Central's authority, never a migration or secret write.
pub fn scan_secret_sources(
    catalog: &NativeCatalog,
    effects: &mut dyn SecretEffects,
    options: &ScanOptions,
) -> Result<SecretScan> {
    let catalog = catalog.secret_sources().as_value();
    let roots: Vec<_> = options
        .roots
        .iter()
        .map(|p| effects.expand_root(p).map_err(Error::new))
        .collect::<Result<_>>()?;
    let descriptors = array(&catalog["descriptors"])?;
    let mut declared_env = HashSet::new();
    for d in descriptors {
        if let Some(names) = d["probe"]["env"]["names"].as_array() {
            for name in names {
                declared_env.insert(text(name)?.to_owned());
            }
        }
    }
    let mut sources = descriptors
        .iter()
        .map(|d| declared(d, effects))
        .collect::<Result<Vec<_>>>()?;
    let declared_paths: HashSet<_> = sources
        .iter()
        .flat_map(|s| s["evidence"].as_array().into_iter().flatten())
        .filter_map(|e| {
            e["where"]
                .as_str()
                .filter(|p| p.starts_with('/'))
                .map(str::to_owned)
        })
        .collect();
    let env_spec = format!("name_pattern {ENV_PATTERN}");
    match effects.inspect("env", &json!({"name_pattern":ENV_PATTERN})) {
        Ok(reading) => {
            for e in reading.evidence {
                if !declared_env.contains(&e.location) {
                    sources.push(violation("env", "env", &env_spec, "uncentralised-env", e));
                }
            }
        }
        Err(e) => sources.push(unavailable_entry(
            "env",
            "env-discovery-unavailable",
            "env",
            &env_spec,
            e,
        )),
    }
    let file_spec = FILE_PATTERNS.join(",");
    match effects.inspect(
        "file-pattern",
        &json!({"patterns":FILE_PATTERNS,"roots":roots}),
    ) {
        Err(e) => sources.push(unavailable_entry(
            "file",
            "file-discovery-unavailable",
            "file-pattern",
            &file_spec,
            e,
        )),
        Ok(reading) if reading.truncated => sources.push(unavailable_entry(
            "file",
            "file-discovery-truncated",
            "file-pattern",
            &file_spec,
            format!(
                "file walk hit its {}-file budget before completing — discovery results incomplete",
                reading.files_scanned
            ),
        )),
        Ok(reading) => {
            for e in reading.evidence {
                if !declared_paths.contains(&e.location) {
                    sources.push(violation(
                        "file",
                        "file-pattern",
                        &file_spec,
                        "stray-plaintext",
                        e,
                    ));
                }
            }
        }
    }
    let violations: Vec<_> = sources
        .iter()
        .filter(|e| e["state"] == "violation")
        .map(|e| e["slug"].clone())
        .collect();
    let partial = sources.iter().any(|e| e["state"] == "unavailable");
    let millis = options.observed_at.unix_nanos() / 1_000_000;
    SecretScan::try_from(
        json!({"schema":SECRET_DETECTION_VERSION,"document":"scan","scan_ref":format!("secret-scan:{}:{millis}",options.scanner_implementation),"observed_at":options.observed_at.as_str(),"catalog_revision":catalog["catalog_revision"],"scanner":{"implementation":options.scanner_implementation,"version":options.scanner_version},"sources":sources,"violations":violations,"coverage":if partial{"partial"}else{"complete"},"disclosure":["Evidence is fingerprint-only (SHA-256 + byte length + location). No secret value is representable in this record."]}),
    )
}
