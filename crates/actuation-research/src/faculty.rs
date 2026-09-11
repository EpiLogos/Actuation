//! Prime's inherited faculty is native Actuation infrastructure over an exact
//! QL owner. Python only transports calls. No QL algebra or source clone lives here.
use crate::{
    evidence::{bytes_digest, stable_digest},
    owner::OwnerInstrument,
    process::ProcessSpec,
    world::World,
    Error, Result,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
const SOURCE_FILES: &[&str] = &[
    "docs/wiki-structural-contract-v2.md",
    "docs/sources/ql-musical-derivation-v3.md",
    "docs/music/PRE-M-MUSICAL-DERIVATION-v1.md",
    "crates/ql-mef/src/music.rs",
];
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerBinding {
    pub process: ProcessSpec,
    pub revision: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceBinding {
    pub root: PathBuf,
    pub git: PathBuf,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacultyConfig {
    pub schema: String,
    pub owner: OwnerBinding,
    pub harmonic_enabled: bool,
    #[serde(default)]
    pub source: Option<SourceBinding>,
    #[serde(default)]
    pub evidence_root: Option<PathBuf>,
}
impl FacultyConfig {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.is_absolute() {
            return Err(Error::new("faculty configuration must be an absolute path"));
        }
        let meta = std::fs::symlink_metadata(path)
            .map_err(|_| Error::new("faculty configuration unavailable"))?;
        if !meta.is_file() || meta.file_type().is_symlink() {
            return Err(Error::new("faculty configuration must be a regular file"));
        }
        let mut bytes = Vec::new();
        File::open(path)
            .map_err(|_| Error::new("faculty configuration unavailable"))?
            .take(1_048_577)
            .read_to_end(&mut bytes)
            .map_err(|_| Error::new("faculty configuration read failed"))?;
        if bytes.len() > 1_048_576 {
            return Err(Error::new("faculty configuration exceeds bound"));
        }
        let c: Self = serde_json::from_slice(&bytes)
            .map_err(|e| Error::new(format!("invalid faculty configuration: {e}")))?;
        if c.schema != "actuation.prime-faculty/v1" {
            return Err(Error::new("wrong faculty configuration schema"));
        }
        if let Some(root) = &c.evidence_root {
            World::open(root)?;
        }
        if c.source
            .as_ref()
            .is_some_and(|s| !s.root.is_absolute() || !s.git.is_absolute())
        {
            return Err(Error::new(
                "faculty source root and Git executable require absolute paths",
            ));
        }
        Ok(c)
    }
    pub fn bind(&self) -> Result<OwnerInstrument> {
        OwnerInstrument::bind(self.owner.process.clone(), &self.owner.revision)
    }
    fn source_file(&self, path: &str) -> Result<Value> {
        if !SOURCE_FILES.contains(&path) {
            return Err(Error::new("source is outside the admitted faculty surface"));
        }
        let source = self.source.as_ref().ok_or_else(|| {
            Error::new("this faculty operation requires a supplied QL source repository")
        })?;
        let world = World::open(&source.root)?;
        let spec = ProcessSpec {
            program: source.git.clone(),
            args: vec![
                "--no-pager".into(),
                "--no-replace-objects".into(),
                "-C".into(),
                world.root().to_string_lossy().into_owned(),
                "cat-file".into(),
                "blob".into(),
                format!("{}:{path}", self.owner.revision),
            ],
            cwd: world.root().into(),
            environment: BTreeMap::new(),
            timeout_ms: 10000,
            output_limit: 8 * 1024 * 1024,
        };
        let response = spec.run(&[])?;
        world.verify_root()?;
        if response.code != Some(0) {
            return Err(Error::new(
                "source file is absent from the exact locked Git object",
            ));
        }
        Ok(
            json!({"path":path,"revision":self.owner.revision,"content":response.stdout,
            "content_sha256":bytes_digest(response.stdout.as_bytes()),"source":"locked-git-object-not-working-tree"}),
        )
    }
}
fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v[key]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| Error::new(format!("faculty {key} requires text")))
}
fn invoke_owner(config: &FacultyConfig, owner: &OwnerInstrument, request: &Value) -> Result<Value> {
    let op = text(request, "operation")?;
    let cli = |args: Value| -> Result<Value> {
        Ok(owner.invoke(json!({"operation":"cli","arguments":args}))?["result"].clone())
    };
    match op {
        "capabilities" => cli(json!(["capabilities"])),
        "kernel-apply" => cli(json!([
            "kernel",
            "apply",
            text(request, "operator")?,
            text(request, "address")?
        ])),
        "mef-lenses" => cli(json!(["mef", "lenses"])),
        "context-frames" => cli(json!(["context-frame", "list"])),
        "vak-locate" => cli(json!(["vak", "locate", text(request, "vak_ref")?])),
        "negotiate" => cli(json!([
            "service",
            "negotiate",
            text(request, "service_operation")?
        ])),
        "wiki-refract" => Ok(owner
            .invoke(json!({"operation":"wiki-refraction","request":request["request"]}))?
            ["result"]
            .clone()),
        "source-state" => Ok(
            json!({"ql_mef_root":config.source.as_ref().map(|s|&s.root),"revision":config.owner.revision,
            "harmonic_enabled":config.harmonic_enabled,"owner_basis":owner.basis(),"standing":"source-locked-owner"}),
        ),
        "constellation-contract" => config.source_file(SOURCE_FILES[0]),
        "harmonic-search" => {
            let query = text(request, "query")?;
            let max = request
                .get("max_matches")
                .cloned()
                .unwrap_or(json!(8))
                .as_u64()
                .filter(|n| (1..=128).contains(n))
                .ok_or_else(|| Error::new("max_matches requires 1..128"))?
                as usize;
            let mut matches = vec![];
            let mut sources = vec![];
            for path in &SOURCE_FILES[1..if config.harmonic_enabled { 4 } else { 3 }] {
                let source = config.source_file(path)?;
                let lines = source["content"]
                    .as_str()
                    .expect("source text")
                    .lines()
                    .collect::<Vec<_>>();
                for (index, line) in lines.iter().enumerate() {
                    if line.to_lowercase().contains(&query.to_lowercase()) {
                        matches.push(json!({"path":path,"line":index+1,
                            "excerpt":lines[index.saturating_sub(1)..(index+2).min(lines.len())].join("\n")}));
                        if matches.len() >= max {
                            break;
                        }
                    }
                }
                sources.push(json!({"path":path,"content_sha256":source["content_sha256"]}));
                if matches.len() >= max {
                    break;
                }
            }
            Ok(
                json!({"query":query,"revision":config.owner.revision,"harmonic_enabled":config.harmonic_enabled,
                "matches":matches,"sources":sources,"source":"locked-git-objects-not-working-tree"}),
            )
        }
        "harmonic-snapshot" => {
            if !config.harmonic_enabled {
                return Err(Error::new(
                    "harmonic faculty is not admitted in this configuration",
                ));
            }
            let basis = request.get("basis").cloned().unwrap_or(json!("chromatic"));
            let v = owner.invoke(json!({"operation":"harmonic-snapshot","basis":basis}))?["result"]
                .clone();
            // Keep the installed faculty's public field names; every value is
            // obtained from the owner, not recomputed by the language adapter.
            Ok(
                json!({"basis":v["basis"],"direct_helix":v["direct_helix"],"conjugate_helix":v["conjugate_helix"],
                "lens_anchor_pitches":v["lens_anchor_pitches"],"A_direct_deltas":v["pair_intervals"]["A"],
                "B_direct_deltas":v["pair_intervals"]["B"],"C_direct_deltas":v["pair_intervals"]["C"],
                "D1_cross_deltas":v["cross_intervals"]["same-position"],"D2_transform_deltas":v["cross_intervals"]["transform"],
                "D2_require_deltas":v["cross_intervals"]["require"],"D2_complete_deltas":v["cross_intervals"]["complete"],
                "mode_tonic_count":v["mode_tonic_instances"],"revision":config.owner.revision,"standing":"source-locked-owner"}),
            )
        }
        _ => Err(Error::new("unsupported inherited faculty operation")),
    }
}
/// A receipt records actual native invocation, including refusal. It is not a
/// provider attestation or an authenticated identification of the caller Agency.
pub fn invoke(
    path: &Path,
    request: &Value,
    trace_ref: Option<&str>,
    declared_locus_ref: Option<&str>,
) -> Result<Value> {
    let config = FacultyConfig::load(path)?;
    if config.evidence_root.is_some() && trace_ref.is_none_or(|s| s.trim().is_empty()) {
        return Err(Error::new(
            "recorded faculty calls require an explicit trace_ref",
        ));
    }
    crate::evidence::candidate_boundary(request)?;
    let owner = config.bind()?;
    let result = invoke_owner(&config, &owner, request);
    let response = match &result {
        Ok(v) => v.clone(),
        Err(e) => json!({"error":e.to_string()}),
    };
    let receipt = json!({"schema":"actuation.prime-ql-operation/v1","operation":text(request,"operation")?,
        "trace_ref":trace_ref,"declared_locus_ref":declared_locus_ref,"caller_identity_standing":"supplied-not-authenticated",
        "process_id":std::process::id(),"sequence":SEQUENCE.fetch_add(1,Ordering::Relaxed),
        "observed_unix_nanos":SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_|Error::new("clock precedes epoch"))?.as_nanos().to_string(),
        "ql_mef_revision":config.owner.revision,"owner_basis":owner.basis(),"request_digest":stable_digest(request),
        "response_digest":stable_digest(&response),"harmonic_enabled":config.harmonic_enabled,"success":result.is_ok(),
        "error":result.as_ref().err().map(|e|e.to_string()),"provider_evidence":"not-assessed","human_acceptance":false});
    let evidence_ref = if let Some(root) = &config.evidence_root {
        let world = World::open(root)?;
        let name = format!("{}.json", stable_digest(&receipt));
        world.write(&name, receipt.to_string().as_bytes(), true)?;
        Some(world.root().join(name))
    } else {
        None
    };
    Ok(
        json!({"schema":"actuation.prime-faculty-result/v1","success":result.is_ok(),"result":response,
        "receipt":receipt,"evidence_ref":evidence_ref}),
    )
}
/// Read only correctly named native receipt objects. Content-addressed files
/// detect accidental alteration; they are not a signature or a sandbox claim.
pub fn receipts(root: &Path) -> Result<BTreeMap<String, Value>> {
    let world = World::open(root)?;
    let mut receipts = BTreeMap::new();
    for entry in world.list(".")? {
        let name = entry["name"]
            .as_str()
            .ok_or_else(|| Error::new("invalid World directory entry"))?;
        if !name.ends_with(".json") {
            continue;
        }
        let v: Value = serde_json::from_slice(&world.read(name)?)
            .map_err(|_| Error::new("invalid faculty receipt JSON"))?;
        if v["schema"] != "actuation.prime-ql-operation/v1"
            || name != format!("{}.json", stable_digest(&v))
        {
            return Err(Error::new("faculty receipt identity/content mismatch"));
        }
        receipts.insert(name.to_owned(), v);
    }
    Ok(receipts)
}
