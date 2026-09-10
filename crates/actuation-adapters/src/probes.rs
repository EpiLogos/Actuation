use crate::wire::*;
use crate::{
    run_bounded, DetectionEffects, Error, FileObservation, Fingerprint, FingerprintEvidence,
    NativeOutput, Observation, ProcessBounds, Result, SecretEffects, SecretFinding,
    ServiceObservation,
};
use regex::Regex;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::ffi::OsString;
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, UNIX_EPOCH};

/// One explicit process environment, not an AIKit profile or credential
/// resolver. Values are private and never included in Debug or probe results.
pub struct NativeEffects {
    home: PathBuf,
    cwd: PathBuf,
    search_path: OsString,
    environment: BTreeMap<String, String>,
    pub process_bounds: ProcessBounds,
    pub http_timeout: Duration,
    pub max_http_bytes: u64,
    pub max_files: u64,
    pub max_fingerprint_bytes: u64,
    pub max_depth: usize,
}
impl NativeEffects {
    pub fn snapshot() -> Result<Self> {
        let home = std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
            Error::new("HOME is unavailable; supply an explicit observation environment")
        })?;
        let cwd = std::env::current_dir().map_err(|e| Error::new(e.to_string()))?;
        Ok(Self::in_environment(
            home,
            cwd,
            std::env::var_os("PATH").unwrap_or_default(),
            std::env::vars_os()
                .filter_map(|(k, v)| Some((k.into_string().ok()?, v.into_string().ok()?)))
                .collect(),
        ))
    }
    pub fn in_environment(
        home: PathBuf,
        cwd: PathBuf,
        search_path: OsString,
        environment: BTreeMap<String, String>,
    ) -> Self {
        Self {
            home,
            cwd,
            search_path,
            environment,
            process_bounds: ProcessBounds::default(),
            http_timeout: Duration::from_secs(4),
            max_http_bytes: 4 * 1024 * 1024,
            max_files: 500_000,
            max_fingerprint_bytes: 1024 * 1024 * 1024,
            max_depth: 6,
        }
    }
    fn expanded(&self, path: &str) -> PathBuf {
        if let Some(suffix) = path.strip_prefix('~') {
            self.home.join(suffix.trim_start_matches('/'))
        } else {
            let p = Path::new(path);
            if p.is_absolute() {
                p.to_owned()
            } else {
                self.cwd.join(p)
            }
        }
    }
    fn locate(&self, names: &[String]) -> Observation<String> {
        let mut inaccessible = None;
        for name in names {
            // Descriptor executable names are basenames. Absolute commands
            // belong to an explicitly supplied body, not implicit PATH lookup.
            if name.is_empty()
                || Path::new(name).components().count() != 1
                || name == "."
                || name == ".."
            {
                return Observation::Unavailable("executable name must be a basename".into());
            }
            for directory in
                std::env::split_paths(&self.search_path).filter(|p| !p.as_os_str().is_empty())
            {
                let candidate = if directory.is_absolute() {
                    directory.join(name)
                } else {
                    self.cwd.join(directory).join(name)
                };
                match fs::metadata(&candidate) {
                    Ok(metadata) if metadata.is_file() => {
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            if metadata.permissions().mode() & 0o111 == 0 {
                                continue;
                            }
                        }
                        return Observation::Present(candidate.to_string_lossy().into_owned());
                    }
                    Ok(_) => {}
                    Err(e) if e.kind() == ErrorKind::NotFound => {}
                    Err(e) => inaccessible = Some(format!("executable lookup incomplete: {e}")),
                }
            }
        }
        inaccessible.map_or(Observation::Absent, Observation::Unavailable)
    }
    fn command(&self, path: &str, args: &[String]) -> Command {
        let mut command = Command::new(path);
        command
            .args(args)
            .current_dir(&self.cwd)
            .env_clear()
            .envs(&self.environment)
            .env("HOME", &self.home)
            .env("PATH", &self.search_path);
        command
    }
    fn run(&self, path: &str, args: &[String]) -> Result<NativeOutput> {
        run_bounded(&mut self.command(path, args), self.process_bounds)
    }
    fn agent(&self) -> ureq::Agent {
        ureq::Agent::config_builder()
            .timeout_global(Some(self.http_timeout))
            .max_redirects(0)
            .http_status_as_error(false)
            .proxy(None)
            .build()
            .into()
    }
    fn checked_url(&self, address: &str) -> Result<url::Url> {
        let parsed = url::Url::parse(address).map_err(|_| Error::new("service URL is invalid"))?;
        if !["http", "https"].contains(&parsed.scheme())
            || parsed.host_str().is_none()
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.fragment().is_some()
        {
            return Err(Error::new("service observation requires an HTTP(S) URL without inline credentials or fragments"));
        }
        Ok(parsed)
    }
    fn digest_file(&self, path: &Path, follow_final_link: bool) -> Result<(Fingerprint, u64)> {
        // No raw file data leaves this function. Incremental hashing avoids
        // loading a large executable or secret-bearing file into a receipt.
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            let mut flags = rustix::fs::OFlags::NONBLOCK;
            if !follow_final_link {
                flags |= rustix::fs::OFlags::NOFOLLOW;
            }
            options.custom_flags(flags.bits() as i32);
        }
        let mut file = options
            .open(path)
            .map_err(|e| Error::new(format!("fingerprint source unavailable: {e}")))?;
        let before = file.metadata().map_err(|e| Error::new(e.to_string()))?;
        if !before.is_file() || before.len() > self.max_fingerprint_bytes {
            return Err(Error::new(
                "fingerprint source is not a regular file within its byte bound",
            ));
        }
        let mut hasher = Sha256::new();
        let mut buffer = [0; 65536];
        let mut total = 0u64;
        loop {
            let count = file
                .read(&mut buffer)
                .map_err(|e| Error::new(format!("fingerprint read failed: {e}")))?;
            if count == 0 {
                break;
            }
            total = total
                .checked_add(count as u64)
                .ok_or_else(|| Error::new("fingerprint byte count overflow"))?;
            if total > self.max_fingerprint_bytes {
                return Err(Error::new("fingerprint source exceeded its byte bound"));
            }
            hasher.update(&buffer[..count]);
        }
        Ok((
            Fingerprint::try_from(format!("{:x}", hasher.finalize()))?,
            total,
        ))
    }
    fn env_fingerprints(&self, spec: &Value) -> Result<SecretFinding> {
        let mut names = if let Some(v) = present(spec, "names") {
            owned_strings(v, "env.names")?
        } else {
            let regex = Regex::new(text(&spec["name_pattern"], "env.name_pattern")?)
                .map_err(|_| Error::new("environment-name pattern is invalid"))?;
            self.environment
                .keys()
                .filter(|n| regex.is_match(n))
                .cloned()
                .collect()
        };
        names.sort();
        names.dedup();
        let evidence = names
            .into_iter()
            .filter_map(|name| {
                self.environment
                    .get(&name)
                    .filter(|v| !v.is_empty())
                    .map(|v| FingerprintEvidence::new(name, v.as_bytes()))
            })
            .collect();
        Ok(SecretFinding {
            evidence,
            ..SecretFinding::default()
        })
    }
    fn file_fingerprints(&self, spec: &Value) -> Result<SecretFinding> {
        let roots = present(spec, "roots")
            .map(|v| owned_strings(v, "file roots"))
            .transpose()?
            .unwrap_or_else(|| vec!["~".into()]);
        let patterns = owned_strings(&spec["patterns"], "file patterns")?
            .into_iter()
            .map(|p| {
                Regex::new(&format!(
                    "^{}$",
                    p.split('*')
                        .map(regex::escape)
                        .collect::<Vec<_>>()
                        .join(".*")
                ))
            })
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|_| Error::new("file pattern is invalid"))?;
        let maximum = present(spec, "max_files")
            .map(|v| {
                v.as_u64()
                    .ok_or_else(|| Error::new("max_files must be a nonnegative integer"))
            })
            .transpose()?
            .unwrap_or(self.max_files)
            .min(self.max_files);
        let skipped: HashSet<&str> = [
            "node_modules",
            ".git",
            "target",
            "dist",
            "build",
            ".next",
            "__pycache__",
            ".venv",
            "venv",
        ]
        .into_iter()
        .collect();
        let mut pending: Vec<_> = roots
            .iter()
            .rev()
            .map(|p| (self.expanded(p), 0usize))
            .collect();
        let mut result = SecretFinding::default();
        let mut seen = HashSet::new();
        while let Some((directory, depth)) = pending.pop() {
            if depth > self.max_depth {
                result.truncated = true;
                continue;
            }
            let metadata = match fs::symlink_metadata(&directory) {
                Ok(m) => m,
                Err(e) if e.kind() == ErrorKind::NotFound => continue,
                Err(e) => return Err(Error::new(format!("file scan root unavailable: {e}"))),
            };
            if metadata.file_type().is_symlink() {
                return Err(Error::new(
                    "file scan refuses a symlink directory; supply its owned root explicitly",
                ));
            }
            if !metadata.is_dir() {
                return Err(Error::new("file scan root is not a directory"));
            }
            let canonical = fs::canonicalize(&directory).map_err(|e| Error::new(e.to_string()))?;
            if !seen.insert(canonical) {
                continue;
            }
            let mut entries = fs::read_dir(&directory)
                .map_err(|e| Error::new(format!("file scan directory unavailable: {e}")))?
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e| Error::new(e.to_string()))?;
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                if result.files_scanned >= maximum {
                    result.truncated = true;
                    break;
                }
                let kind = entry
                    .file_type()
                    .map_err(|e| Error::new(format!("file scan metadata unavailable: {e}")))?;
                let name = entry.file_name().to_string_lossy().into_owned();
                if kind.is_dir() {
                    if !skipped.contains(name.as_str()) {
                        pending.push((entry.path(), depth + 1));
                    }
                    continue;
                }
                result.files_scanned += 1;
                if !patterns.iter().any(|r| r.is_match(&name)) {
                    continue;
                }
                if kind.is_symlink() {
                    return Err(Error::new(
                        "matching secret-source path is a symlink; no target material was read",
                    ));
                }
                if !kind.is_file() {
                    continue;
                }
                let path = entry.path();
                let (fingerprint, length) = self.digest_file(&path, false)?;
                result.evidence.push(FingerprintEvidence {
                    location: path.to_string_lossy().into_owned(),
                    fingerprint,
                    byte_length: Some(length),
                });
            }
            if result.files_scanned >= maximum {
                if !pending.is_empty() {
                    result.truncated = true;
                }
                break;
            }
        }
        result.evidence.sort_by(|a, b| a.location.cmp(&b.location));
        result.evidence.dedup_by(|a, b| a.location == b.location);
        Ok(result)
    }
    fn vault_item(&self, spec: &Value) -> Result<SecretFinding> {
        let path = match self.locate(&["op".into()]) {
            Observation::Present(p) => p,
            Observation::Absent => {
                return Err(Error::new(
                    "vault observation unavailable: op is not installed",
                ))
            }
            Observation::Unavailable(e) => return Err(Error::new(e)),
        };
        let item_ref = text(&spec["item_ref"], "item_ref")?;
        let run = self.run(
            &path,
            &[
                "item".into(),
                "get".into(),
                item_ref.into(),
                "--format".into(),
                "json".into(),
            ],
        )?;
        if !run.status.success() {
            // Unauthenticated is not absent. Do not echo CLI output: it may
            // contain credentials or user material. A bounded explicit item-
            // missing diagnostic is the only supported negative observation.
            let message = String::from_utf8_lossy(&run.stderr).to_ascii_lowercase();
            if message.contains("item not found") && !message.contains("not signed in") {
                return Ok(SecretFinding::default());
            }
            return Err(Error::new(format!(
                "vault observation failed (status {:?}); identity/access remains unknown",
                run.status.code()
            )));
        }
        let item: Value = serde_json::from_slice(&run.stdout)
            .map_err(|_| Error::new("vault returned invalid JSON; no material was retained"))?;
        object(&item)?;
        Ok(SecretFinding {
            found: true,
            vault_item: Some((
                item["id"].as_str().map(str::to_owned),
                item["vault"]["name"].as_str().map(str::to_owned),
            )),
            ..SecretFinding::default()
        })
    }
}
impl DetectionEffects for NativeEffects {
    fn expand_home(&mut self, path: &str) -> Result<String> {
        Ok(self.expanded(path).to_string_lossy().into_owned())
    }
    fn resolve_executable(&mut self, names: &[String]) -> Observation<String> {
        self.locate(names)
    }
    fn stat(&mut self, path: &str) -> Observation<FileObservation> {
        match fs::metadata(path) {
            Ok(info) => Observation::Present(FileObservation {
                is_directory: info.is_dir(),
                size: Some(info.len()),
                modified_millis: info
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs_f64() * 1000.0),
            }),
            Err(e) if e.kind() == ErrorKind::NotFound => Observation::Absent,
            Err(e) => Observation::Unavailable(format!("filesystem observation failed: {e}")),
        }
    }
    fn fingerprint(&mut self, path: &str) -> Result<Fingerprint> {
        self.digest_file(Path::new(path), true).map(|(d, _)| d)
    }
    fn directory_count(&mut self, path: &str) -> Observation<u64> {
        match fs::read_dir(path) {
            Err(e) if e.kind() == ErrorKind::NotFound => Observation::Absent,
            Err(e) => Observation::Unavailable(format!("directory count failed: {e}")),
            Ok(mut entries) => match entries.try_fold(0u64, |n, e| e.map(|_| n + 1)) {
                Ok(n) => Observation::Present(n),
                Err(e) => Observation::Unavailable(format!("directory count incomplete: {e}")),
            },
        }
    }
    fn version(&mut self, path: &str, args: &[String]) -> Result<String> {
        let run = self.run(path, args)?;
        if !run.status.success() {
            return Err(Error::new(format!(
                "native version probe exited {:?}",
                run.status.code()
            )));
        }
        let version = String::from_utf8_lossy(&run.stdout);
        Ok(version
            .trim()
            .lines()
            .next()
            .unwrap_or("(no output)")
            .chars()
            .take(120)
            .collect())
    }
    fn environment_markers(&mut self, names: &[String]) -> Result<Vec<String>> {
        Ok(names
            .iter()
            .filter(|n| self.environment.get(*n).is_some_and(|v| !v.is_empty()))
            .cloned()
            .collect())
    }
    fn service(&mut self, spec: &Value) -> ServiceObservation {
        match spec["kind"].as_str() {
            Some("http") => {
                let address = present(spec, "default_url")
                    .or_else(|| present(spec, "url"))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let address = match self.checked_url(address) {
                    Ok(a) => a,
                    Err(e) => return ServiceObservation::Unavailable(e.to_string()),
                };
                match self.agent().get(address.as_str()).call() {
                    Ok(response) => ServiceObservation::Present(format!(
                        "http {} from {address}",
                        response.status().as_u16()
                    )),
                    Err(ureq::Error::Io(e)) if e.kind() == ErrorKind::ConnectionRefused => {
                        ServiceObservation::Absent(format!("no listener at {address}"))
                    }
                    Err(_) => ServiceObservation::Unavailable(
                        "HTTP service observation could not complete".into(),
                    ),
                }
            }
            Some("daemon") => {
                let name = match text(&spec["name"], "daemon.name") {
                    Ok(n) => n,
                    Err(e) => return ServiceObservation::Unavailable(e.to_string()),
                };
                let path = match self.locate(&["pgrep".into()]) {
                    Observation::Present(p) => p,
                    _ => {
                        return ServiceObservation::Unavailable(
                            "daemon observation unavailable: pgrep is not available".into(),
                        )
                    }
                };
                match self.run(&path, &["-x".into(), name.into()]) {
                    Ok(run) if run.status.success() => {
                        ServiceObservation::Present(format!("daemon {name} running"))
                    }
                    Ok(run) if run.status.code() == Some(1) => {
                        ServiceObservation::Absent(format!("daemon {name} not running"))
                    }
                    _ => ServiceObservation::Unavailable(
                        "daemon observation could not complete".into(),
                    ),
                }
            }
            _ => ServiceObservation::Unavailable("unsupported service probe kind".into()),
        }
    }
    fn http_json(&mut self, address: &str) -> Option<Result<Value>> {
        Some((|| {
            let url = self.checked_url(address)?;
            let mut response = self
                .agent()
                .get(url.as_str())
                .call()
                .map_err(|_| Error::new("HTTP inventory read could not complete"))?;
            if !response.status().is_success() {
                return Err(Error::new(format!(
                    "HTTP inventory status {}",
                    response.status().as_u16()
                )));
            }
            let bytes = response
                .body_mut()
                .with_config()
                .limit(self.max_http_bytes)
                .read_to_vec()
                .map_err(|_| {
                    Error::new("HTTP inventory body exceeded its bounds or could not be read")
                })?;
            serde_json::from_slice(&bytes)
                .map_err(|_| Error::new("HTTP inventory returned invalid JSON"))
        })())
    }
}
impl SecretEffects for NativeEffects {
    fn probe(&mut self, kind: &str, spec: &Value) -> Option<Result<SecretFinding>> {
        Some(match kind {
            "env" => self.env_fingerprints(spec),
            "file-pattern" => self.file_fingerprints(spec),
            "vault-item" => self.vault_item(spec),
            "cli-presence" => {
                owned_strings(&spec["names"], "cli.names").and_then(|names| {
                    match self.locate(&names) {
                        Observation::Present(path) => {
                            let _ = self.version(&path, &["--version".into()]);
                            Ok(SecretFinding {
                                found: true,
                                ..SecretFinding::default()
                            })
                        }
                        Observation::Absent => Ok(SecretFinding::default()),
                        Observation::Unavailable(reason) => Err(Error::new(reason)),
                    }
                })
            }
            _ => return None,
        })
    }
}
