//! Bounded native observations. No shell interpolation, target installation,
//! model routing, credential resolution or mandatory daemon is involved.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self},
    io::{ErrorKind, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant, UNIX_EPOCH},
};

pub type ProbeResult<T> = std::result::Result<T, String>;
#[derive(Clone, Debug)]
pub struct FileObservation {
    pub is_directory: bool,
    pub modified_millis: Option<f64>,
    pub byte_length: Option<u64>,
}
#[derive(Clone, Debug)]
pub enum ServiceObservation {
    Present(String),
    Absent(String),
    Unverified(String),
}
impl ServiceObservation {
    pub fn detail(&self) -> &str {
        match self {
            Self::Present(s) | Self::Absent(s) | Self::Unverified(s) => s,
        }
    }
    pub fn is_present(&self) -> bool {
        matches!(self, Self::Present(_))
    }
}

/// Effects supply observed facts, not semantic Agent or permission identity.
/// Environment observations return marker NAMES only, never marker values.
/// Native defaults and controlled tests both enter the same detection engine.
pub trait ProbeEffects {
    fn expand_home(&mut self, path: &str) -> ProbeResult<String>;
    fn resolve_executable(&mut self, names: &[String]) -> ProbeResult<Option<String>>;
    fn stat(&mut self, path: &str) -> ProbeResult<Option<FileObservation>>;
    fn hash(&mut self, path: &str) -> ProbeResult<String>;
    fn directory_count(&mut self, path: &str) -> ProbeResult<Option<usize>>;
    fn markers(
        &mut self,
        names: &[String],
        environment: Option<&BTreeMap<String, String>>,
    ) -> ProbeResult<Vec<String>>;
    fn service(&mut self, spec: &Value) -> ProbeResult<ServiceObservation>;
    fn version(&mut self, path: &str, args: &[String]) -> ProbeResult<String>;
    fn http_json(&mut self, url: &str) -> ProbeResult<Value>;
}

/// Explicit observation environment. It intentionally implements neither Debug
/// nor Serialize: the private environment snapshot may contain credentials.
pub struct NativeEffects {
    pub(crate) home: Option<PathBuf>,
    pub(crate) cwd: PathBuf,
    pub(crate) search_path: Vec<PathBuf>,
    pub(crate) environment: BTreeMap<String, String>,
    pub(crate) timeout: Duration,
    http: ureq::Agent,
}
impl NativeEffects {
    pub fn from_environment() -> ProbeResult<Self> {
        Ok(Self::new(
            std::env::var_os("HOME").map(PathBuf::from),
            std::env::current_dir().map_err(|e| e.to_string())?,
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
                .filter(|p| !p.as_os_str().is_empty())
                .collect(),
            std::env::vars().collect(),
        ))
    }
    pub fn new(
        home: Option<PathBuf>,
        cwd: PathBuf,
        search_path: Vec<PathBuf>,
        environment: BTreeMap<String, String>,
    ) -> Self {
        let timeout = Duration::from_secs(5);
        let http = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(4)))
            .http_status_as_error(false)
            .max_redirects(0)
            .proxy(None)
            .build()
            .into();
        Self {
            home,
            cwd,
            search_path,
            environment,
            timeout,
            http,
        }
    }
    pub fn with_process_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    fn request(&self, url: &str) -> ProbeResult<ureq::http::Response<ureq::Body>> {
        // Never follow a redirect to a second endpoint, nor attach credentials
        // from an ambient proxy configuration. The declared endpoint is the
        // entire network authority of this read-only observation.
        let parsed: ureq::http::Uri = url
            .parse()
            .map_err(|_| "invalid observation endpoint".to_owned())?;
        if !matches!(parsed.scheme_str(), Some("http" | "https"))
            || parsed.authority().is_none()
            || parsed.authority().is_some_and(|a| a.as_str().contains('@'))
        {
            return Err("observation endpoint must be HTTP(S) without embedded credentials".into());
        }
        self.http.get(url).call().map_err(|e| match &e {
            ureq::Error::Io(io) if io.kind() == ErrorKind::ConnectionRefused => {
                "connection refused".to_owned()
            }
            _ => format!("endpoint observation failed: {e}"),
        })
    }
    pub(crate) fn process(
        &self,
        name: &str,
        args: &[String],
        limit: usize,
    ) -> ProbeResult<ProcessOutput> {
        bounded_process(
            Path::new(name),
            args,
            self.timeout,
            limit,
            &self.environment,
        )
    }
}
impl ProbeEffects for NativeEffects {
    fn expand_home(&mut self, path: &str) -> ProbeResult<String> {
        let p = if let Some(suffix) = path.strip_prefix('~') {
            self.home
                .as_ref()
                .ok_or_else(|| "home directory is unavailable".to_owned())?
                .join(suffix.trim_start_matches('/'))
        } else if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            self.cwd.join(path)
        };
        Ok(p.to_string_lossy().into_owned())
    }
    fn resolve_executable(&mut self, names: &[String]) -> ProbeResult<Option<String>> {
        for name in names {
            if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains('\0') {
                return Err("executable name must be a non-empty basename".into());
            }
            for parent in &self.search_path {
                let p = parent.join(name);
                let m = match fs::metadata(&p) {
                    Ok(m) => m,
                    Err(e) if e.kind() == ErrorKind::NotFound => continue,
                    Err(e) => return Err(format!("executable metadata unavailable: {e}")),
                };
                if !m.is_file() {
                    continue;
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if m.permissions().mode() & 0o111 == 0 {
                        continue;
                    }
                }
                return Ok(Some(p.to_string_lossy().into_owned()));
            }
        }
        Ok(None)
    }
    fn stat(&mut self, path: &str) -> ProbeResult<Option<FileObservation>> {
        match fs::metadata(path) {
            Ok(m) => Ok(Some(FileObservation {
                is_directory: m.is_dir(),
                modified_millis: m
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs_f64() * 1000.0),
                byte_length: Some(m.len()),
            })),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("metadata unavailable: {e}")),
        }
    }
    fn hash(&mut self, path: &str) -> ProbeResult<String> {
        // O_NONBLOCK prevents a substituted FIFO from hanging a metadata
        // probe. Executable symlinks remain valid; inspect the opened target.
        let mut options = fs::OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NONBLOCK);
        }
        let mut f = options.open(path).map_err(|e| e.to_string())?;
        let metadata = f.metadata().map_err(|e| e.to_string())?;
        if !metadata.is_file() {
            return Err("fingerprint requires a regular file".into());
        }
        const LIMIT: u64 = 1024 * 1024 * 1024;
        if metadata.len() > LIMIT {
            return Err("executable exceeds fingerprint budget".into());
        }
        let start = Instant::now();
        let mut h = Sha256::new();
        let mut b = [0_u8; 65536];
        let mut bytes = 0_u64;
        loop {
            let n = f.read(&mut b).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            bytes += n as u64;
            if bytes > LIMIT || start.elapsed() > self.timeout {
                return Err("executable fingerprint budget exhausted".into());
            }
            h.update(&b[..n]);
        }
        if bytes != metadata.len() {
            return Err("executable changed during fingerprint observation".into());
        }
        Ok(format!("{:x}", h.finalize()))
    }
    fn directory_count(&mut self, path: &str) -> ProbeResult<Option<usize>> {
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("directory observation unavailable: {e}")),
        };
        let start = Instant::now();
        let mut count = 0;
        for entry in entries {
            entry.map_err(|e| e.to_string())?;
            count += 1;
            if count > 1_000_000 || start.elapsed() > self.timeout {
                return Err("directory count incomplete: observation budget exhausted".into());
            }
        }
        Ok(Some(count))
    }
    fn markers(
        &mut self,
        names: &[String],
        environment: Option<&BTreeMap<String, String>>,
    ) -> ProbeResult<Vec<String>> {
        let env = environment.unwrap_or(&self.environment);
        Ok(names
            .iter()
            .filter(|n| env.get(*n).is_some_and(|v| !v.is_empty()))
            .cloned()
            .collect())
    }
    fn service(&mut self, spec: &Value) -> ProbeResult<ServiceObservation> {
        match spec["kind"].as_str() {
            Some("http") => {
                let url = spec["default_url"]
                    .as_str()
                    .or(spec["url"].as_str())
                    .ok_or_else(|| "http service probe requires default_url".to_owned())?;
                match self.request(url) {
                    Ok(response) => Ok(ServiceObservation::Present(format!(
                        "http {} from {url}",
                        response.status().as_u16()
                    ))),
                    Err(e) if e == "connection refused" => {
                        Ok(ServiceObservation::Absent(format!("no listener at {url}")))
                    }
                    Err(e) => Err(e),
                }
            }
            Some("daemon") => {
                let name = spec["name"]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| "daemon probe requires name".to_owned())?;
                let program = self
                    .resolve_executable(&["pgrep".into()])?
                    .ok_or_else(|| "pgrep unavailable".to_owned())?;
                let result = self.process(&program, &["-x".into(), name.into()], 16 * 1024)?;
                match result.code {
                    Some(0) => Ok(ServiceObservation::Present(format!(
                        "daemon {name} running"
                    ))),
                    Some(1) => Ok(ServiceObservation::Absent(format!(
                        "daemon {name} not running"
                    ))),
                    _ => Err(format!("pgrep exit {:?}", result.code)),
                }
            }
            _ => Err("unsupported service kind".into()),
        }
    }
    fn version(&mut self, path: &str, args: &[String]) -> ProbeResult<String> {
        let run = self.process(path, args, 64 * 1024)?;
        if run.code != Some(0) {
            return Err(format!("exit {:?}", run.code));
        }
        let stdout = String::from_utf8_lossy(&run.stdout);
        let line = stdout
            .trim()
            .lines()
            .next()
            .filter(|s| !s.is_empty())
            .unwrap_or("(no output)");
        Ok(line.chars().take(120).collect())
    }
    fn http_json(&mut self, url: &str) -> ProbeResult<Value> {
        let mut response = self.request(url)?;
        if response.status().as_u16() >= 400 {
            return Err(format!("http {}", response.status().as_u16()));
        }
        let bytes = response
            .body_mut()
            .with_config()
            .limit(4 * 1024 * 1024)
            .read_to_vec()
            .map_err(|e| format!("bounded inventory read failed: {e}"))?;
        serde_json::from_slice(&bytes).map_err(|_| "unparseable inventory JSON".into())
    }
}

pub(crate) struct ProcessOutput {
    pub code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
/// File-backed capture bounds elapsed time even when a descendant inherits an
/// output descriptor. Unix probes own a fresh process group and reap their
/// direct child. Output budgets are observed during execution, not after an
/// unbounded allocation. No captured provider error text is logged here.
pub(crate) fn bounded_process(
    program: &Path,
    args: &[String],
    timeout: Duration,
    limit: usize,
    environment: &BTreeMap<String, String>,
) -> ProbeResult<ProcessOutput> {
    let mut stdout = tempfile::tempfile()
        .map_err(|e| format!("cannot create private probe capture: {}", e.kind()))?;
    let mut stderr = tempfile::tempfile()
        .map_err(|e| format!("cannot create private probe capture: {}", e.kind()))?;
    let mut command = Command::new(program);
    command
        .args(args)
        .env_clear()
        .envs(environment)
        .stdin(Stdio::null())
        .stdout(stdout.try_clone().map_err(|e| e.to_string())?)
        .stderr(stderr.try_clone().map_err(|e| e.to_string())?);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("spawn failed: {}", e.kind()))?;
    let group = rustix::process::Pid::from_raw(child.id() as i32);
    let start = Instant::now();
    let status = loop {
        let bytes = stdout
            .metadata()
            .map(|m| m.len())
            .unwrap_or(u64::MAX)
            .max(stderr.metadata().map(|m| m.len()).unwrap_or(u64::MAX));
        if bytes > limit as u64 {
            break Err("process observation exceeded output budget".to_owned());
        }
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Err(e) => break Err(format!("process observation failed: {}", e.kind())),
            Ok(None) if start.elapsed() >= timeout => {
                break Err("process observation timed out".into())
            }
            _ => thread::sleep(Duration::from_millis(5)),
        }
    };
    #[cfg(unix)]
    if let Some(group) = group {
        let _ = rustix::process::kill_process_group(group, rustix::process::Signal::KILL);
    }
    let _ = child.kill();
    let _ = child.wait();
    let status = status?;
    stdout.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    stderr.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    let mut err = Vec::new();
    stdout
        .take(limit as u64 + 1)
        .read_to_end(&mut out)
        .map_err(|e| format!("stdout read failed: {}", e.kind()))?;
    stderr
        .take(limit as u64 + 1)
        .read_to_end(&mut err)
        .map_err(|e| format!("stderr read failed: {}", e.kind()))?;
    if out.len() > limit || err.len() > limit {
        return Err("process observation exceeded output budget".into());
    }
    Ok(ProcessOutput {
        code: status.code(),
        stdout: out,
        stderr: err,
    })
}
