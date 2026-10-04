//! Bounded owned specimen processes. This is a research execution boundary,
//! not Workcell hosting or AIKit provider selection. No implicit shell/env.
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashSet},
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
    thread,
    time::{Duration, Instant},
};
fn io(e: std::io::Error) -> Error {
    Error::new(format!("research process: {}", e.kind())).with_source(e)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessSpec {
    pub program: PathBuf,
    #[serde(default)]
    pub args: Vec<String>,
    pub cwd: PathBuf,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
    pub timeout_ms: u64,
    /// Requests that omit an explicit bound get the documented maximum
    /// (64 MiB), matching what recursive Prime runs need; validate() still
    /// rejects anything above it.
    #[serde(default = "default_output_limit")]
    pub output_limit: usize,
}
fn default_output_limit() -> usize {
    64 * 1024 * 1024
}
#[derive(Clone, Debug, Serialize)]
pub struct ProcessResult {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}
#[derive(Clone, Default, Debug, Serialize)]
pub struct ProcessObservation {
    pub phase: &'static str,
    pub spawned: bool,
    pub direct_child_reaped: bool,
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
    pub signal_forbidden: bool,
    pub term_signal_attempted: bool,
    pub kill_signal_attempted: bool,
    pub direct_kill_attempted: bool,
    pub group_absent: bool,
    pub unreaped_owner_observed_before_signal: bool,
    pub retirement_deadline_exhausted: bool,
    pub reply_observed: bool,
    pub stdout_observed_bytes: Option<u64>,
    pub stderr_observed_bytes: Option<u64>,
    pub captured_stdout_bytes: usize,
    pub captured_stderr_bytes: usize,
    pub capture_truncated: bool,
    pub capture_read_failed: bool,
}
#[derive(Clone)]
struct ObservedCause {
    phase: &'static str,
    error: Error,
}
#[derive(Clone, Default)]
struct Retirement {
    observation: ProcessObservation,
    errors: Vec<ObservedCause>,
    // ESRCH is observed group absence, not evidence of a failed direct wait.
    // Retain the original cause separately; never reconstruct errno from text.
    absence: Vec<ObservedCause>,
}
impl Retirement {
    fn clean(&self) -> bool {
        self.observation.direct_child_reaped && self.errors.is_empty()
    }
}
/// Private byte access is explicit. Neither Debug nor public JSON emits it.
/// Capture is a bounded observation of held regular files, not pipe EOF or a
/// hard while-running disk/RSS limit. Escaped descendants are not excluded.
pub struct NativeProcessFailure {
    primary: Error,
    secondary: Vec<ObservedCause>,
    absence: Vec<ObservedCause>,
    observation: ProcessObservation,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}
impl NativeProcessFailure {
    pub fn observation(&self) -> &ProcessObservation {
        &self.observation
    }
    pub fn primary(&self) -> &Error {
        &self.primary
    }
    pub fn secondary(&self) -> impl Iterator<Item = (&'static str, &Error)> {
        self.secondary.iter().map(|c| (c.phase, &c.error))
    }
    pub fn observed_group_absence(&self) -> impl Iterator<Item = (&'static str, &Error)> {
        self.absence.iter().map(|c| (c.phase, &c.error))
    }
    pub fn private_capture(&self) -> (&[u8], &[u8]) {
        (&self.stdout, &self.stderr)
    }
}
impl std::fmt::Debug for NativeProcessFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeProcessFailure")
            .field("observation", &self.observation)
            .field("secondary_count", &self.secondary.len())
            .field("observed_absence_count", &self.absence.len())
            .finish()
    }
}
impl std::fmt::Display for NativeProcessFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "native process {} failure", self.observation.phase)
    }
}
impl std::error::Error for NativeProcessFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.primary)
    }
}
fn actual_io<'a>(mut e: &'a (dyn std::error::Error + 'static)) -> Option<&'a std::io::Error> {
    for _ in 0..32 {
        if let Some(io) = e.downcast_ref::<std::io::Error>() {
            return Some(io);
        }
        e = e.source()?;
    }
    None
}
fn io_fact(e: &Error) -> Value {
    match actual_io(e) {
        Some(io) => json!({"kind":format!("{:?}",io.kind()),"raw_os_error":io.raw_os_error()}),
        None => Value::Null,
    }
}
fn native_failure_in<'a>(
    mut error: &'a (dyn std::error::Error + 'static),
) -> Option<&'a NativeProcessFailure> {
    for _ in 0..32 {
        if let Some(failure) = error.downcast_ref::<NativeProcessFailure>() {
            return Some(failure);
        }
        error = error.source()?;
    }
    None
}
/// Selected owner-only error facts. Exact scalar types prevent accidental
/// copying of a reply/request/document Value or the private capture. Causal
/// projection admits at most32 entries/links, with actual counts disclosed.
pub fn failure_details(error: &Error) -> Option<Value> {
    const CAUSE_LIMIT: usize = 32;
    let supplemental_count = error.secondary_sources().count();
    let supplemental = error.secondary_sources().take(CAUSE_LIMIT).map(|e| {
        actual_io(e).map(|io| json!({"kind":format!("{:?}",io.kind()),"raw_os_error":io.raw_os_error()}))
    }).collect::<Vec<_>>();
    // A later publisher/read failure can retain the process failure as an
    // actual supplemental source beside the original semantic cause. Do not
    // lose that owner observation, or label it the outer operation's primary.
    let (failure, attachment) = match native_failure_in(error) {
        Some(failure) => (failure, "primary_source"),
        None => (error.secondary_sources().take(CAUSE_LIMIT).find_map(|e| native_failure_in(e))?, "supplemental_source"),
    };
    Some(json!({
        "attachment":attachment,
        "observation":failure.observation,
        "primary_io":io_fact(&failure.primary),
        "cause_projection_limit":CAUSE_LIMIT,
        "supplemental_count":supplemental_count,
        "secondary_count":failure.secondary.len(),
        "observed_group_absence_count":failure.absence.len(),
        "cause_projection_truncated":supplemental_count > CAUSE_LIMIT
            || failure.secondary.len() > CAUSE_LIMIT || failure.absence.len() > CAUSE_LIMIT,
        "supplemental_io":supplemental,
        "secondary":failure.secondary.iter().take(CAUSE_LIMIT).map(|c|
            json!({"phase":c.phase,"io":io_fact(&c.error)})).collect::<Vec<_>>(),
        "observed_group_absence":failure.absence.iter().take(CAUSE_LIMIT).map(|c|
            json!({"phase":c.phase,"io":io_fact(&c.error)})).collect::<Vec<_>>()
    }))
}
#[cfg(all(test, unix))]
type RetirementCheckpoint = Option<Box<dyn FnOnce(rustix::process::Pid)>>;
#[cfg(all(test, unix))]
thread_local! {
    static RETIREMENT_CHECKPOINT: std::cell::RefCell<RetirementCheckpoint> =
        std::cell::RefCell::new(None);
}
#[cfg(all(test, unix))]
pub(crate) fn retirement_checkpoint(f: impl FnOnce(rustix::process::Pid) + 'static) {
    RETIREMENT_CHECKPOINT.with(|slot| {
        assert!(slot.borrow().is_none(), "checkpoint already admitted");
        *slot.borrow_mut() = Some(Box::new(f));
    });
}
struct OwnedChild {
    child: Child,
    termination_grace: Duration,
    observation: ProcessObservation,
    retired: Option<std::sync::Arc<Retirement>>,
}
impl OwnedChild {
    fn new(child: Child) -> Self {
        Self {
            child,
            termination_grace: Duration::ZERO,
            observation: ProcessObservation { spawned: true, phase: "running", ..Default::default() },
            retired: None,
        }
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn peek(&mut self) -> Result<bool> {
        use rustix::process::{waitid, Pid, WaitId, WaitIdOptions};
        let pid = Pid::from_raw(self.child.id() as i32)
            .ok_or_else(|| Error::new("owned child identity unavailable"))?;
        match waitid(WaitId::Pid(pid), WaitIdOptions::EXITED | WaitIdOptions::NOHANG | WaitIdOptions::NOWAIT) {
            Ok(Some(status)) => {
                self.observation.exit_code = status.exit_status();
                self.observation.signal = status.terminating_signal();
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(e) => {
                self.observation.signal_forbidden = true;
                Err(io(e.into()))
            }
        }
    }
    #[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]
    fn peek(&mut self) -> Result<bool> {
        self.observation.signal_forbidden = true;
        Err(Error::new("safe unreaped process observation is unavailable on this platform"))
    }
    #[cfg(not(unix))]
    fn peek(&mut self) -> Result<bool> {
        match self.child.try_wait() {
            Ok(Some(status)) => {
                self.observation.direct_child_reaped = true;
                self.observation.exit_code = status.code();
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(e) => {
                self.observation.signal_forbidden = true;
                Err(io(e))
            }
        }
    }
    #[cfg(unix)]
    fn held_identity(&mut self) -> Result<rustix::process::Pid> {
        if self.observation.signal_forbidden || self.observation.direct_child_reaped {
            return Err(Error::new("numeric signal is unavailable after lost or reaped ownership"));
        }
        self.peek()?;
        let pid = rustix::process::Pid::from_raw(self.child.id() as i32)
            .ok_or_else(|| Error::new("owned child identity unavailable"))?;
        match rustix::process::getpgid(Some(pid)) {
            Ok(group) if group == pid => {
                self.observation.unreaped_owner_observed_before_signal = true;
                Ok(pid)
            },
            Ok(_) => {
                self.observation.signal_forbidden = true;
                Err(Error::new("owned child process group changed"))
            }
            Err(e) => {
                self.observation.signal_forbidden = true;
                Err(io(e.into()))
            }
        }
    }
    #[cfg(unix)]
    fn group_signal(&mut self, signal: rustix::process::Signal, phase: &'static str, r: &mut Retirement) {
        let pid = match self.held_identity() {
            Ok(pid) => pid,
            Err(error) => {
                self.observation.signal_forbidden = true;
                r.errors.push(ObservedCause { phase: "owner_identity", error });
                return;
            }
        };
        if signal == rustix::process::Signal::TERM {
            self.observation.term_signal_attempted = true;
        } else {
            self.observation.kill_signal_attempted = true;
        }
        match rustix::process::kill_process_group(pid, signal) {
            Ok(()) => {},
            Err(e) if e == rustix::io::Errno::SRCH => {
                self.observation.group_absent = true;
                r.absence.push(ObservedCause { phase, error: io(e.into()) });
            }
            Err(e) => r.errors.push(ObservedCause { phase, error: io(e.into()) }),
        }
    }
    fn retire(&mut self) -> std::sync::Arc<Retirement> {
        if let Some(r) = &self.retired {
            return r.clone();
        }
        #[cfg(all(test, unix))]
        RETIREMENT_CHECKPOINT.with(|slot| {
            let checkpoint = slot.borrow_mut().take();
            if let Some(checkpoint) = checkpoint {
                if let Some(pid) = rustix::process::Pid::from_raw(self.child.id() as i32) {
                    checkpoint(pid);
                }
            }
        });
        let mut r = Retirement::default();
        self.observation.phase = "retirement";
        #[cfg(unix)]
        {
            if !self.termination_grace.is_zero() {
                self.group_signal(rustix::process::Signal::TERM, "group_term", &mut r);
                let grace_end = Instant::now() + self.termination_grace;
                while !self.observation.signal_forbidden && Instant::now() < grace_end {
                    match self.peek() {
                        Ok(true) => break,
                        Ok(false) => thread::sleep(Duration::from_millis(5)),
                        Err(error) => {
                            r.errors.push(ObservedCause { phase: "grace_wait", error });
                            break;
                        }
                    }
                }
            }
            if !self.observation.signal_forbidden {
                self.group_signal(rustix::process::Signal::KILL, "group_kill", &mut r);
            }
            // A failed group signal may leave the owned leader alive. Any
            // direct fallback still needs the same unreaped native witness.
            if !r.errors.is_empty() && !self.observation.signal_forbidden {
                match self.held_identity() {
                    Ok(_) => {
                        self.observation.direct_kill_attempted = true;
                        if let Err(e) = self.child.kill() {
                            r.errors.push(ObservedCause { phase: "direct_kill", error: io(e) });
                        }
                    }
                    Err(error) => r.errors.push(ObservedCause { phase: "owner_identity", error }),
                }
            }
        }
        #[cfg(not(unix))]
        if !self.observation.direct_child_reaped && !self.observation.signal_forbidden {
            match self.peek() {
                Ok(false) => {
                    self.observation.direct_kill_attempted = true;
                    if let Err(e) = self.child.kill() {
                        r.errors.push(ObservedCause { phase: "direct_kill", error: io(e) });
                    }
                }
                Ok(true) => {},
                Err(error) => r.errors.push(ObservedCause { phase: "owner_identity", error }),
            }
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        while !self.observation.direct_child_reaped {
            match self.child.try_wait() {
                Ok(Some(status)) => {
                    self.observation.direct_child_reaped = true;
                    self.observation.exit_code = status.code();
                    #[cfg(unix)]
                    {
                        use std::os::unix::process::ExitStatusExt;
                        self.observation.signal = status.signal();
                    }
                    break;
                }
                Ok(None) => {},
                Err(e) => {
                    self.observation.signal_forbidden = true;
                    r.errors.push(ObservedCause { phase: "reap", error: io(e) });
                    break;
                }
            }
            if Instant::now() >= deadline {
                self.observation.retirement_deadline_exhausted = true;
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        r.observation = self.observation.clone();
        let r = std::sync::Arc::new(r);
        self.retired = Some(r.clone());
        r
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        // Once attempted, Drop cannot retry a numeric signal or blocking wait.
        let _ = self.retire();
    }
}
fn capture_file(file: &File, cap: usize) -> (Vec<u8>, Option<std::io::Error>) {
    let mut bytes = Vec::new();
    let mut block = [0_u8; 8192];
    while bytes.len() < cap {
        let width = block.len().min(cap - bytes.len());
        #[cfg(unix)]
        let n = {
            use std::os::unix::fs::FileExt;
            file.read_at(&mut block[..width], bytes.len() as u64)
        };
        #[cfg(not(unix))]
        let n = {
            use std::os::windows::fs::FileExt;
            file.seek_read(&mut block[..width], bytes.len() as u64)
        };
        let n = match n {
            Ok(n) => n,
            Err(error) => return (bytes, Some(error)),
        };
        if n == 0 { break; }
        bytes.extend_from_slice(&block[..n]);
    }
    (bytes, None)
}
fn process_failure(primary: Error, retirement: &Retirement, out: &File, err: &File, limit: usize, phase: &'static str, reply_observed: bool) -> Error {
    let mut observation = retirement.observation.clone();
    observation.phase = phase;
    observation.reply_observed = reply_observed;
    let mut secondary = retirement.errors.clone();
    let mut observe = |file: &File, field: &mut Option<u64>| {
        match file.metadata() {
            Ok(m) => *field = Some(m.len()),
            Err(e) => secondary.push(ObservedCause { phase: "capture_metadata", error: io(e) }),
        }
    };
    observe(out, &mut observation.stdout_observed_bytes);
    observe(err, &mut observation.stderr_observed_bytes);
    // Private partial evidence has an explicit one-MiB aggregate profile.
    // Truncation is disclosed and never admitted as a successful JSON result.
    let cap = limit.min(1024 * 1024);
    let (stdout, stdout_error) = capture_file(out, cap);
    let (stderr, stderr_error) = capture_file(err, cap.saturating_sub(stdout.len()));
    observation.capture_read_failed = stdout_error.is_some() || stderr_error.is_some();
    if let Some(e) = stdout_error {
        secondary.push(ObservedCause { phase: "capture_stdout", error: io(e) });
    }
    if let Some(e) = stderr_error {
        secondary.push(ObservedCause { phase: "capture_stderr", error: io(e) });
    }
    observation.captured_stdout_bytes = stdout.len();
    observation.captured_stderr_bytes = stderr.len();
    observation.capture_truncated = observation.stdout_observed_bytes
        .is_some_and(|n| n > stdout.len() as u64)
        || observation.stderr_observed_bytes.is_some_and(|n| n > stderr.len() as u64);
    let message = primary.to_string();
    Error::new(message).with_source(NativeProcessFailure {
        primary, secondary, absence: retirement.absence.clone(), observation, stdout, stderr,
    })
}
impl ProcessSpec {
    pub fn validate(&self) -> Result<()> {
        if !self.program.is_absolute() || !self.cwd.is_absolute() {
            return Err(Error::new(
                "specimen program and cwd must be explicit absolute paths",
            ));
        }
        if self.timeout_ms == 0
            || self.timeout_ms > 3_600_000
            || self.output_limit == 0
            || self.output_limit > 64 * 1024 * 1024
        {
            return Err(Error::new("invalid process time/output bound"));
        }
        Ok(())
    }
    fn command(&self) -> Result<Command> {
        self.validate()?;
        #[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]
        {
            Err(Error::new("safe unreaped process observation is unavailable on this platform"))
        }
        #[cfg(not(all(unix, not(any(target_os = "linux", target_os = "macos")))))]
        {
            let mut c = Command::new(&self.program);
            c.args(&self.args)
                .current_dir(&self.cwd)
                .env_clear()
                .envs(&self.environment);
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                c.process_group(0);
            }
            Ok(c)
        }
    }
    pub fn run(&self, input: &[u8]) -> Result<ProcessResult> {
        self.run_with_termination_grace(input, Duration::ZERO)
    }
    /// Explicit decision instruments may own a separately grouped native client.
    /// Let their TERM handler reap it before the final group KILL. Ordinary
    /// process/RPC callers retain their existing immediate cleanup policy.
    pub fn run_with_termination_grace(
        &self,
        input: &[u8],
        termination_grace: Duration,
    ) -> Result<ProcessResult> {
        if termination_grace > Duration::from_secs(3) {
            return Err(Error::new("process termination grace exceeds bound"));
        }
        if input.len() > 16 * 1024 * 1024 {
            return Err(Error::new("specimen input exceeds bound"));
        }
        let mut stdin = tempfile::tempfile().map_err(io)?;
        stdin.write_all(input).map_err(io)?;
        stdin.seek(SeekFrom::Start(0)).map_err(io)?;
        let mut stdout = tempfile::tempfile().map_err(io)?;
        let mut stderr = tempfile::tempfile().map_err(io)?;
        let mut c = self.command()?;
        c.stdin(stdin)
            .stdout(stdout.try_clone().map_err(io)?)
            .stderr(stderr.try_clone().map_err(io)?);
        let spawned = c.spawn().map_err(|e| process_failure(
            io(e), &Retirement::default(), &stdout, &stderr, self.output_limit, "spawn", false,
        ))?;
        let mut child = OwnedChild::new(spawned);
        child.termination_grace = termination_grace;
        let start = Instant::now();
        let outcome = (|| -> Result<()> {
            loop {
                check_size(&stdout, &stderr, self.output_limit)?;
                // WNOWAIT keeps the original owner until intended group effects.
                if child.peek()? { return Ok(()); }
                if start.elapsed() >= Duration::from_millis(self.timeout_ms) {
                    return Err(Error::new("specimen process timed out"));
                }
                thread::sleep(Duration::from_millis(5));
            }
        })();
        let retirement = child.retire();
        if let Err(primary) = outcome {
            return Err(process_failure(primary, &retirement, &stdout, &stderr,
                self.output_limit, "execution", false));
        }
        if !retirement.clean() {
            return Err(process_failure(Error::new("specimen process retirement unconfirmed"),
                &retirement, &stdout, &stderr, self.output_limit, "retirement", false));
        }
        let result = (|| {
            check_size(&stdout, &stderr, self.output_limit)?;
            Ok(ProcessResult {
                code: retirement.observation.exit_code,
                stdout: read_final(&mut stdout, self.output_limit)?,
                stderr: read_final(&mut stderr, self.output_limit)?,
            })
        })();
        result.map_err(|primary| process_failure(primary, &retirement, &stdout, &stderr,
            self.output_limit, "capture", false))
    }
}
fn check_size(out: &File, err: &File, limit: usize) -> Result<()> {
    if out
        .metadata()
        .map_err(io)?
        .len()
        .saturating_add(err.metadata().map_err(io)?.len())
        > limit as u64
    {
        Err(Error::new("specimen output budget exhausted"))
    } else {
        Ok(())
    }
}
fn read_final(f: &mut File, limit: usize) -> Result<String> {
    f.seek(SeekFrom::Start(0)).map_err(io)?;
    let mut b = Vec::new();
    f.take(limit as u64 + 1).read_to_end(&mut b).map_err(io)?;
    if b.len() > limit {
        return Err(Error::new("specimen output exceeds bound"));
    }
    String::from_utf8(b).map_err(|e| Error::new("specimen output is not UTF8").with_source(e.utf8_error()))
}
/// Serial JSONL RPC retains unsolicited records while matching the exact reply.
/// File-backed reads use read_at, never changing the child's output offset.
/// A nonblocking input pipe makes even a non-reading child obey the deadline.
pub struct RpcClient {
    child: OwnedChild,
    stdin: ChildStdin,
    stdout: File,
    stderr: File,
    spec: ProcessSpec,
    offset: u64,
    buffer: Vec<u8>,
    records: Vec<Value>,
    sequence: u64,
    stopped: bool,
    refinement_sent: bool,
    response_ids: HashSet<String>,
    reply_observed: bool,
    finish_result: Option<Result<()>>,
}
impl RpcClient {
    pub fn start(spec: ProcessSpec) -> Result<Self> {
        let stdout = tempfile::tempfile().map_err(io)?;
        let stderr = tempfile::tempfile().map_err(io)?;
        let mut c = spec.command()?;
        c.stdin(Stdio::piped())
            .stdout(stdout.try_clone().map_err(io)?)
            .stderr(stderr.try_clone().map_err(io)?);
        let spawned = c.spawn().map_err(|e| process_failure(
            io(e), &Retirement::default(), &stdout, &stderr, spec.output_limit, "spawn", false,
        ))?;
        let mut child = OwnedChild::new(spawned);
        let setup = (|| {
            let stdin = child.child.stdin.take()
                .ok_or_else(|| Error::new("RPC stdin unavailable"))?;
            let flags = rustix::fs::fcntl_getfl(&stdin).map_err(|e| io(e.into()))?;
            rustix::fs::fcntl_setfl(&stdin, flags | rustix::fs::OFlags::NONBLOCK)
                .map_err(|e| io(e.into()))?;
            Ok(stdin)
        })();
        let stdin = match setup {
            Ok(stdin) => stdin,
            Err(primary) => {
                let retirement = child.retire();
                return Err(process_failure(primary, &retirement, &stdout, &stderr,
                    spec.output_limit, "rpc_setup", false));
            }
        };
        Ok(Self {
            child,
            stdin,
            stdout,
            stderr,
            spec,
            offset: 0,
            buffer: vec![],
            records: vec![],
            sequence: 0,
            stopped: false,
            refinement_sent: false,
            response_ids: HashSet::new(),
            reply_observed: false,
            finish_result: None,
        })
    }
    pub fn refinement_attempted(&self) -> bool {
        self.refinement_sent
    }
    pub fn stderr_text(&mut self) -> Result<String> {
        read_final(&mut self.stderr, self.spec.output_limit)
    }
    pub fn records(&self) -> &[Value] {
        &self.records
    }
    fn consume(&mut self) -> Result<Vec<Value>> {
        check_size(&self.stdout, &self.stderr, self.spec.output_limit)?;
        let mut fresh = Vec::new();
        let mut b = [0; 8192];
        loop {
            // A still-running or escaped writer cannot make retirement capture
            // an unbounded read. These are owner capture limits, not pipe EOF.
            check_size(&self.stdout, &self.stderr, self.spec.output_limit)?;
            if self.offset > self.spec.output_limit as u64 {
                return Err(Error::new("RPC observed byte budget exhausted"));
            }
            #[cfg(unix)]
            let n = {
                use std::os::unix::fs::FileExt;
                self.stdout.read_at(&mut b, self.offset).map_err(io)?
            };
            #[cfg(not(unix))]
            let n = {
                use std::os::windows::fs::FileExt;
                self.stdout.seek_read(&mut b, self.offset).map_err(io)?
            };
            if n == 0 {
                break;
            }
            if self.offset.saturating_add(n as u64) > self.spec.output_limit as u64 {
                return Err(Error::new("RPC observed byte budget exhausted"));
            }
            self.offset += n as u64;
            self.buffer.extend_from_slice(&b[..n]);
            while let Some(end) = self.buffer.iter().position(|b| *b == b'\n') {
                let mut line = self.buffer.drain(..=end).collect::<Vec<_>>();
                line.pop();
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                if line.is_empty() {
                    continue;
                }
                let v=serde_json::from_slice::<Value>(&line).unwrap_or_else(|_|json!({"type":"client_parse_error","line":String::from_utf8_lossy(&line),"error":"invalid JSONL record"}));
                self.records.push(v.clone());
                fresh.push(v);
            }
            if self.buffer.len() > 1024 * 1024 {
                return Err(Error::new("RPC frame exceeds bound"));
            }
        }
        // Keep raw records before refusing protocol ambiguity. A duplicate that
        // arrives later also poisons the next operation rather than being reused.
        for record in &fresh {
            if record["type"] == "response" {
                if let Some(id) = record["id"].as_str() {
                    if !self.response_ids.insert(id.to_owned()) {
                        return Err(Error::new("duplicate RPC response identity"));
                    }
                }
            }
        }
        Ok(fresh)
    }
    /// A transport-valid refusal is data for SDK sessions: the caller may still
    /// request final evidence. Framing, identity and timeout failures stop the
    /// process. Prime's request() keeps its existing stop-on-refusal behaviour.
    pub fn exchange(&mut self, command: Value, timeout: Duration) -> Result<Value> {
        match self.request_inner(command, timeout) {
            Ok(reply) => { self.reply_observed = true; Ok(reply) },
            Err(primary) => Err(self.finish_after_error(primary)),
        }
    }
    pub fn request(&mut self, command: Value, timeout: Duration) -> Result<Value> {
        let reply = self.exchange(command, timeout)?;
        if reply["success"] != true {
            return Err(self.finish_after_error(Error::new(
                "specimen refused RPC request; retained in raw record",
            )));
        }
        Ok(reply)
    }
    fn request_inner(&mut self, mut command: Value, timeout: Duration) -> Result<Value> {
        if self.stopped {
            return Err(Error::new("RPC has stopped"));
        }
        if !command.is_object() || !command["type"].is_string() {
            return Err(Error::new("RPC requires command type"));
        }
        self.sequence += 1;
        let id = format!("actuation-prime-{}", self.sequence);
        if !command["id"].is_null() {
            return Err(Error::new("RPC request identities are client-owned"));
        }
        command["id"] = json!(id);
        let mut bytes =
            serde_json::to_vec(&command).map_err(|_| Error::new("RPC command encode failed"))?;
        bytes.push(b'\n');
        if bytes.len() > 1024 * 1024 {
            return Err(Error::new("RPC request exceeds bound"));
        }
        let start = Instant::now();
        let deadline = timeout.min(Duration::from_millis(self.spec.timeout_ms));
        let mut written = 0;
        loop {
            if start.elapsed() >= deadline {
                return Err(Error::new("RPC request timed out"));
            }
            if written < bytes.len() {
                match self.stdin.write(&bytes[written..]) {
                    Ok(0) => {
                                return Err(Error::new("RPC input closed"));
                    }
                    Ok(n) => written += n,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => {
                                return Err(io(e));
                    }
                }
            }
            for v in self.consume()? {
                if v["type"] == "response" && v["id"] == id {
                    if written != bytes.len()
                        || (v.get("command").is_some() && v["command"] != command["type"])
                    {
                        return Err(Error::new(
                            "RPC reply command does not match the dispatched operation",
                        ));
                    }
                    if !v["success"].is_boolean() {
                        return Err(Error::new("RPC reply has unknown success state"));
                    }
                    return Ok(v);
                }
            }
            if self.child.peek()? {
                return Err(Error::new("RPC exited before a correlated response"));
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
    pub fn state(&mut self) -> Result<Value> {
        Ok(self.request(json!({"type":"get_state"}), Duration::from_secs(60))?["data"].clone())
    }
    pub fn wait_idle(&mut self, timeout: Duration) -> Result<Value> {
        let start = Instant::now();
        let mut busy = false;
        let mut idle_answers = 0;
        loop {
            let remaining = timeout
                .checked_sub(start.elapsed())
                .ok_or_else(|| Error::new("Prime idle deadline exhausted"))?;
            let state = self.request(
                json!({"type":"get_state"}),
                remaining.min(Duration::from_secs(60)),
            )?["data"]
                .clone();
            let streaming = state["isStreaming"]
                .as_bool()
                .ok_or_else(|| Error::new("Prime streaming state is unknown"))?;
            let unfinished = if state["unfinishedActionCount"].is_null() {
                0
            } else {
                state["unfinishedActionCount"]
                    .as_u64()
                    .ok_or_else(|| Error::new("invalid unfinishedActionCount"))?
            };
            if streaming || unfinished > 0 {
                busy = true;
                idle_answers = 0;
            } else if busy {
                return Ok(state);
            } else {
                let remaining = timeout
                    .checked_sub(start.elapsed())
                    .ok_or_else(|| Error::new("Prime idle deadline exhausted"))?;
                let v = self.request(
                    json!({"type":"get_last_assistant_text"}),
                    remaining.min(Duration::from_secs(60)),
                )?;
                if v["data"]["text"].as_str().is_some_and(|s| !s.is_empty()) {
                    idle_answers += 1
                } else {
                    idle_answers = 0;
                }
                if idle_answers >= 2 {
                    return Ok(state);
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
    /// The one-shot is consumed before sending, including timeout/failure: a
    /// missing reply is not permission to perform a second consequential pass.
    pub fn refine_once(
        &mut self,
        authorised: bool,
        trajectory_completed: bool,
        instructions: &str,
    ) -> Result<Value> {
        self.refine_once_bounded(
            authorised,
            trajectory_completed,
            instructions,
            Duration::from_secs(600),
        )
    }
    /// The caller's remaining run budget also bounds the consequential pass.
    pub fn refine_once_bounded(
        &mut self,
        authorised: bool,
        trajectory_completed: bool,
        instructions: &str,
        timeout: Duration,
    ) -> Result<Value> {
        if !authorised || !trajectory_completed || self.refinement_sent {
            return Err(Error::new(
                "refinement requires completed trajectory and unspent explicit authority",
            ));
        }
        self.refinement_sent = true;
        Ok(self.request(
            json!({"type":"refine","instructions":instructions,"global":false}),
            timeout,
        )?["data"]
            .clone())
    }
    pub fn retirement_observation(&self) -> Option<ProcessObservation> {
        self.child.retired.as_ref().map(|r| {
            let mut observation = r.observation.clone();
            observation.reply_observed = self.reply_observed;
            observation
        })
    }
    fn finish_with_primary(&mut self, primary: Option<Error>) -> Result<()> {
        if let Some(result) = &self.finish_result {
            return result.clone();
        }
        self.stopped = true;
        let mut retirement = (*self.child.retire()).clone();
        let late = self.consume().err();
        let primary = match (primary, late) {
            (Some(primary), Some(error)) => {
                retirement.errors.push(ObservedCause { phase: "late_rpc_capture", error });
                Some(primary)
            }
            (Some(primary), None) => Some(primary),
            (None, Some(error)) => Some(error),
            (None, None) if !retirement.clean() => Some(Error::new("RPC retirement unconfirmed")),
            (None, None) => None,
        };
        let result = match primary {
            Some(primary) => Err(process_failure(primary, &retirement, &self.stdout,
                &self.stderr, self.spec.output_limit, "rpc_finish", self.reply_observed)),
            None => Ok(()),
        };
        self.finish_result = Some(result.clone());
        result
    }
    pub(crate) fn finish_after_error(&mut self, error: Error) -> Error {
        // A primary semantic failure remains a failure even on clean retirement.
        self.finish_with_primary(Some(error.clone())).err().unwrap_or(error)
    }
    /// Acknowledgement requires actual reaping and retained cleanup/capture truth.
    pub fn finish(&mut self) -> Result<()> {
        self.finish_with_primary(None)
    }
    /// Compatibility only: bounded best effort is not a clean-retirement receipt.
    pub fn stop(&mut self) {
        let _ = self.finish();
    }
}
impl Drop for RpcClient {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn spec(program: &str, args: &[&str], timeout_ms: u64, output_limit: usize) -> ProcessSpec {
        ProcessSpec {
            program: program.into(),
            args: args.iter().map(|s| s.to_string()).collect(),
            cwd: std::env::temp_dir(),
            environment: Default::default(),
            timeout_ms,
            output_limit,
        }
    }

    #[test]
    fn validate_requires_explicit_paths_and_bounded_time_and_output() {
        let mut s = spec("/bin/cat", &[], 1_000, 1_000);
        assert!(s.validate().is_ok());
        s.program = "cat".into();
        assert!(s.validate().is_err());
        s = spec("/bin/cat", &[], 1_000, 1_000);
        s.cwd = "relative".into();
        assert!(s.validate().is_err());
        s = spec("/bin/cat", &[], 0, 1_000);
        assert!(s.validate().is_err());
        s = spec("/bin/cat", &[], 3_600_001, 1_000);
        assert!(s.validate().is_err());
        s = spec("/bin/cat", &[], 1_000, 0);
        assert!(s.validate().is_err());
        s = spec("/bin/cat", &[], 1_000, 64 * 1024 * 1024 + 1);
        assert!(s.validate().is_err());
    }

    #[test]
    fn run_carries_input_output_exit_code_and_a_clean_environment() {
        let out = spec("/bin/cat", &[], 10_000, 1 << 20)
            .run(b"payload-bytes")
            .expect("cat echoes");
        assert_eq!(out.code, Some(0));
        assert_eq!(out.stdout, "payload-bytes");

        let mut s = spec("/usr/bin/env", &[], 10_000, 1 << 20);
        s.environment.insert("RESEARCH_PROBE".into(), "1".into());
        let out = s.run(b"").expect("env runs");
        assert!(out.stdout.contains("RESEARCH_PROBE=1"));
        assert!(
            !out.stdout.contains("PATH="),
            "the specimen environment must be cleared, not inherited"
        );

        let out = spec("/usr/bin/false", &[], 10_000, 1_000).run(b"").unwrap();
        assert_eq!(out.code, Some(1));
    }

    #[test]
    fn output_limit_defaults_to_the_documented_maximum_when_a_request_omits_it() {
        let spec: ProcessSpec = serde_json::from_value(json!({
            "program": "/bin/cat",
            "args": [],
            "cwd": std::env::temp_dir().to_string_lossy(),
            "environment": {},
            "timeout_ms": 1_000
        }))
        .expect("request without an explicit output limit");
        assert_eq!(spec.output_limit, 64 * 1024 * 1024);
        assert!(spec.validate().is_ok());
        // An explicit bound is never overwritten by the default.
        let explicit: ProcessSpec = serde_json::from_value(json!({
            "program": "/bin/cat",
            "args": [],
            "cwd": std::env::temp_dir().to_string_lossy(),
            "environment": {},
            "timeout_ms": 1_000,
            "output_limit": 1_000
        }))
        .expect("request with an explicit output limit");
        assert_eq!(explicit.output_limit, 1_000);
    }

    #[test]
    fn run_enforces_time_and_output_bounds() {
        let err = spec("/bin/sleep", &["5"], 200, 1_000).run(b"").unwrap_err();
        assert!(err.to_string().contains("timed out"));
        let big = vec![b'x'; 4_000];
        let err = spec("/bin/cat", &[], 10_000, 1_000).run(&big).unwrap_err();
        assert!(err.to_string().contains("budget") || err.to_string().contains("bound"));
    }

    fn responder(script: &str) -> (tempfile::TempDir, ProcessSpec) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("responder.sh");
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        let s = ProcessSpec {
            program: path,
            args: vec![],
            cwd: dir.path().to_owned(),
            environment: Default::default(),
            timeout_ms: 10_000,
            output_limit: 1 << 20,
        };
        (dir, s)
    }

    const STATE_OK: &str = r#"{"type":"response","id":"actuation-prime-1","command":"get_state","success":true,"data":{"isStreaming":false}}"#;

    #[test]
    fn rpc_client_correlates_one_request() {
        let (_dir, s) = responder(&format!(
            "#!/bin/sh\nprintf '%s\\n' '{STATE_OK}'\nwhile read -r line; do :; done\n"
        ));
        let mut client = RpcClient::start(s).unwrap();
        let state = client.state().expect("correlated state");
        assert_eq!(state["isStreaming"], serde_json::json!(false));
        assert_eq!(client.records().len(), 1);
        client.stop();
        assert!(client.stderr_text().is_ok());
    }

    #[test]
    fn rpc_client_refuses_duplicate_response_identity() {
        // Both records leave in one buffered write after the request, so the
        // first consume deterministically sees the duplicated identity.
        let (_dir, s) = responder(&format!(
            "#!/bin/sh\nread -r request\nprintf '%s\\n%s\\n' '{STATE_OK}' '{STATE_OK}'\nwhile read -r line; do :; done\n"
        ));
        let mut client = RpcClient::start(s).unwrap();
        let err = client.state().unwrap_err();
        assert!(err.to_string().contains("duplicate"));
    }

    #[test]
    fn rpc_client_stops_on_exit_without_a_correlated_response() {
        // The responder consumes the request and exits without replying, so the
        // transport ends deterministically before any correlated response.
        let (_dir, s) = responder("#!/bin/sh\nread -r request\nexit 0\n");
        let mut client = RpcClient::start(s).unwrap();
        let err = client
            .request(
                serde_json::json!({"type":"get_state"}),
                std::time::Duration::from_secs(5),
            )
            .unwrap_err();
        assert!(
            err.to_string().contains("correlated response")
                || err.to_string().contains("timed out")
        );
    }

    #[test]
    fn rpc_client_treats_a_transport_refusal_as_data_then_stops() {
        let refusal = r#"{"type":"response","id":"actuation-prime-1","command":"get_state","success":false,"data":{}}"#;
        let (_dir, s) = responder(&format!(
            "#!/bin/sh\nprintf '%s\\n' '{refusal}'\nwhile read -r line; do :; done\n"
        ));
        let mut client = RpcClient::start(s).unwrap();
        let err = client
            .request(
                serde_json::json!({"type":"get_state"}),
                std::time::Duration::from_secs(5),
            )
            .unwrap_err();
        assert!(err.to_string().contains("refused RPC request"));
        // Raw records survive the refusal for evidence collection.
        assert_eq!(client.records().len(), 1);
    }
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
pub(crate) mod native_retirement_tests {
    use super::*;
    use std::{
        fs::{self, OpenOptions},
        os::unix::fs::{MetadataExt, PermissionsExt},
        path::{Path, PathBuf},
    };

    /// A native test fixture is retained before any child effect. Unwind or
    /// unknown retirement leaves the exact directory for owner inspection.
    pub(crate) struct Fixture {
        pub(crate) root: PathBuf,
        identity: (u64, u64),
    }
    impl Fixture {
        pub(crate) fn new() -> Self {
            let parent = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../ProjectCentral/now/tmp/actuation-process-native");
            fs::create_dir_all(&parent).expect("native fixture parent");
            let parent = fs::canonicalize(parent).expect("physical native scratch");
            assert!(fs::symlink_metadata(&parent).unwrap().is_dir());
            let root = tempfile::Builder::new()
                .prefix("owned-")
                .tempdir_in(&parent)
                .expect("exclusive native fixture")
                .keep();
            let metadata = fs::symlink_metadata(&root).unwrap();
            let identity = (metadata.dev(), metadata.ino());
            fs::write(
                root.join("custody.json"),
                serde_json::to_vec(&json!({
                    "root":root,"device":identity.0,"inode":identity.1,
                    "disposition":"retained-before-effects",
                    "semantic_world_identity_inferred":false
                })).unwrap(),
            ).unwrap();
            Self { root, identity }
        }
        pub(crate) fn script(&self, body: &str, timeout: u64, limit: usize) -> ProcessSpec {
            let path = self.root.join("specimen.sh");
            fs::write(&path, body).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
            ProcessSpec {
                program: path,
                args: Vec::new(),
                cwd: self.root.clone(),
                environment: BTreeMap::new(),
                timeout_ms: timeout,
                output_limit: limit,
            }
        }
        pub(crate) fn record(&self, facts: Value) {
            fs::write(self.root.join("actual-result.json"), serde_json::to_vec(&facts).unwrap())
                .expect("retain actual native result");
        }
        pub(crate) fn dispose_after_known_retirement(self) {
            let metadata = fs::symlink_metadata(&self.root).expect("current fixture affiliation");
            assert!(metadata.is_dir() && !metadata.file_type().is_symlink());
            assert_eq!((metadata.dev(), metadata.ino()), self.identity);
            fs::remove_dir_all(&self.root).unwrap_or_else(|error| {
                panic!("owned fixture cleanup failed at {}: {error}", self.root.display())
            });
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            // No automatic recursive removal after unknown/unwound lifecycle.
            if self.root.exists() {
                eprintln!("native fixture retained for owner inspection: {}", self.root.display());
            }
        }
    }
    pub(crate) fn failure(error: &Error) -> &NativeProcessFailure {
        let mut source: &(dyn std::error::Error + 'static) = error;
        for _ in 0..32 {
            if let Some(failure) = source.downcast_ref::<NativeProcessFailure>() {
                return failure;
            }
            source = source.source().expect("typed native process source");
        }
        panic!("native process cause chain exceeded profile");
    }
    pub(crate) fn external_reap(pid: rustix::process::Pid) {
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            match rustix::process::waitpid(Some(pid), rustix::process::WaitOptions::NOHANG) {
                Ok(Some((observed, _))) => {
                    assert_eq!(observed, pid);
                    return;
                }
                Ok(None) => {}
                Err(error) => panic!("actual external wait failed: {error}"),
            }
            assert!(Instant::now() < deadline, "actual child did not terminate before external reap");
            thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn native_timeout_retains_private_bytes_and_actual_reaped_owner() {
        let fixture = Fixture::new();
        let spec = fixture.script(
            "#!/bin/sh\nprintf 'PRIVATE-TIMEOUT-CANARY'\nprintf 'PRIVATE-ERR-CANARY' >&2\nexec /bin/sleep 5\n",
            100, 4096,
        );
        let started = Instant::now();
        let error = spec.run(b"PRIVATE-INPUT-CANARY").unwrap_err();
        let failure = failure(&error);
        assert!(started.elapsed() < Duration::from_secs(4));
        assert!(error.to_string().contains("timed out"));
        assert!(failure.observation().direct_child_reaped);
        assert!(failure.observation().kill_signal_attempted);
        assert!(failure.observation().unreaped_owner_observed_before_signal);
        assert!(!failure.observation().signal_forbidden);
        assert!(!failure.observation().retirement_deadline_exhausted);
        assert!(failure.secondary().next().is_none());
        assert_eq!(failure.private_capture().0, b"PRIVATE-TIMEOUT-CANARY");
        assert_eq!(failure.private_capture().1, b"PRIVATE-ERR-CANARY");
        let public = failure_details(&error).unwrap();
        assert!(public["primary_io"].is_null());
        assert!(!public.to_string().contains("PRIVATE"));
        assert!(!format!("{error:?} {failure:?}").contains("PRIVATE"));
        fixture.record(public);
        fixture.dispose_after_known_retirement();
    }

    #[test]
    fn native_budget_refuses_successful_truncation_and_discloses_actual_counts() {
        let fixture = Fixture::new();
        let mut spec = fixture.script("#!/bin/sh\nexec /bin/cat\n", 2000, 32);
        let error = spec.run(&[b'x'; 8192]).unwrap_err();
        let failure = failure(&error);
        assert!(failure.observation().direct_child_reaped);
        assert!(failure.observation().stdout_observed_bytes.unwrap() >= 8192);
        assert_eq!(failure.private_capture().0.len(), 32);
        assert!(failure.observation().capture_truncated);
        assert!(actual_io(failure.primary()).is_none(), "budget is not fictitious IO");
        fixture.record(failure_details(&error).unwrap());
        spec.output_limit = 16384;
        let positive = spec.run(b"actual-restored-echo").unwrap();
        assert_eq!(positive.code, Some(0));
        assert_eq!(positive.stdout, "actual-restored-echo");
        fixture.dispose_after_known_retirement();
    }

    #[test]
    fn native_spawn_absence_and_real_nonroot_eacces_keep_original_io() {
        assert!(!rustix::process::geteuid().is_root(), "genuine EACCES prerequisite: nonroot");
        let fixture = Fixture::new();
        let mut spec = fixture.script("#!/bin/sh\nprintf restored\n", 1000, 1024);
        let executable = spec.program.clone();
        spec.program = fixture.root.join("genuinely-absent-executable");
        let missing = spec.run(b"").unwrap_err();
        let absent = failure(&missing);
        assert_eq!(actual_io(absent.primary()).unwrap().kind(), std::io::ErrorKind::NotFound);
        assert!(actual_io(absent.primary()).unwrap().raw_os_error().is_some());
        assert!(!absent.observation().spawned);
        assert!(!absent.observation().kill_signal_attempted);
        spec.program = executable.clone();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o600)).unwrap();
        let denied = spec.run(b"").unwrap_err();
        let original = actual_io(failure(&denied).primary()).unwrap();
        assert_eq!(original.kind(), std::io::ErrorKind::PermissionDenied);
        assert_eq!(original.raw_os_error(), Some(rustix::io::Errno::ACCESS.raw_os_error()));
        fixture.record(failure_details(&denied).unwrap());
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
        let restored = spec.run(b"").unwrap();
        assert_eq!(restored.code, Some(0));
        assert_eq!(restored.stdout, "restored");
        fixture.dispose_after_known_retirement();
    }

    #[test]
    fn native_invalid_utf8_keeps_actual_error_and_private_capture() {
        let fixture = Fixture::new();
        let spec = fixture.script("#!/bin/sh\nprintf '\\377PRIVATE-UTF8-CANARY'\n", 1000, 1024);
        let error = spec.run(b"").unwrap_err();
        let failure = failure(&error);
        let primary = std::error::Error::source(failure.primary()).unwrap();
        assert!(primary.downcast_ref::<std::str::Utf8Error>().is_some());
        assert_eq!(failure.observation().phase, "capture");
        assert!(failure.observation().direct_child_reaped);
        assert_eq!(failure.private_capture().0[0], 255);
        assert!(!format!("{error:?} {failure:?}").contains("CANARY"));
        assert!(!failure_details(&error).unwrap().to_string().contains("CANARY"));
        fixture.dispose_after_known_retirement();
    }

    #[test]
    fn native_immediate_group_signal_uses_unreaped_terminal_owner_before_wait() {
        let fixture = Fixture::new();
        let spec = fixture.script("#!/bin/sh\nexit 0\n", 1000, 1024);
        let mut child = OwnedChild::new(spec.command().unwrap().spawn().unwrap());
        let deadline = Instant::now() + Duration::from_secs(1);
        while !child.peek().unwrap() {
            if Instant::now() >= deadline {
                let actual = child.retire();
                fixture.record(json!(actual.observation));
                panic!("actual child did not reach terminal observation");
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert!(!child.observation.direct_child_reaped);
        let pid = rustix::process::Pid::from_raw(child.child.id() as i32).unwrap();
        let actual = child.retire();
        assert!(actual.clean());
        assert!(actual.observation.kill_signal_attempted);
        assert!(actual.observation.unreaped_owner_observed_before_signal);
        assert_eq!(actual.observation.exit_code, Some(0));
        assert_eq!(rustix::process::waitpid(Some(pid), rustix::process::WaitOptions::NOHANG)
            .unwrap_err(), rustix::io::Errno::CHILD);
        fixture.record(json!(actual.observation));
        drop(child);
        fixture.dispose_after_known_retirement();
    }

    #[test]
    fn native_external_reap_suppresses_later_numeric_signal_and_drop_retry() {
        let fixture = Fixture::new();
        let spec = fixture.script("#!/bin/sh\nexit 0\n", 1000, 1024);
        let mut child = OwnedChild::new(spec.command().unwrap().spawn().unwrap());
        let pid = rustix::process::Pid::from_raw(child.child.id() as i32).unwrap();
        external_reap(pid);
        let first = child.retire();
        assert!(first.observation.signal_forbidden);
        assert!(!first.observation.direct_child_reaped);
        assert!(!first.observation.term_signal_attempted);
        assert!(!first.observation.kill_signal_attempted);
        assert!(!first.observation.direct_kill_attempted);
        assert!(first.errors.iter().any(|cause|
            actual_io(&cause.error).is_some_and(|io|
                io.raw_os_error() == Some(rustix::io::Errno::CHILD.raw_os_error()))));
        let repeated = child.retire();
        assert!(std::sync::Arc::ptr_eq(&first, &repeated));
        fixture.record(json!({"actual_retirement":first.observation,
            "external_wait_observed":true,"fixture_disposition":"retained-owner-unavailable"}));
        drop(child); // memoized retirement: no new numeric effect
        // Owner unavailable remains a retained fixture, even though this test's
        // independent actual wait reaped the original child.
    }

    #[test]
    fn native_term_grace_preserves_real_handler_client_reaping() {
        let fixture = Fixture::new();
        let spec = fixture.script(
            "#!/bin/sh\n/bin/sleep 5 &\nclient=$!\ntrap 'kill \"$client\"; wait \"$client\"; printf actual-client-reaped > client-reaped; exit 0' TERM\nprintf ready\nwait \"$client\"\n",
            150, 4096,
        );
        let error = spec.run_with_termination_grace(b"", Duration::from_millis(2500)).unwrap_err();
        let actual = failure(&error);
        assert!(actual.observation().term_signal_attempted);
        assert!(actual.observation().unreaped_owner_observed_before_signal);
        assert!(actual.observation().direct_child_reaped);
        assert_eq!(actual.observation().exit_code, Some(0));
        assert_eq!(fs::read(fixture.root.join("client-reaped")).unwrap(), b"actual-client-reaped");
        assert!(actual.secondary().next().is_none());
        fixture.record(failure_details(&error).unwrap());
        fixture.dispose_after_known_retirement();
    }

    #[test]
    fn rpc_timeout_retains_original_failure_and_actual_late_record() {
        let fixture = Fixture::new();
        let spec = fixture.script(
            "#!/bin/sh\nread -r request\nprintf '%s\\n' '{\"type\":\"native_progress\",\"body\":\"PRIVATE-LATE-CANARY\"}'\nexec /bin/sleep 5\n",
            1000, 4096,
        );
        let mut client = RpcClient::start(spec).unwrap();
        let error = client.request(json!({"type":"get_state"}), Duration::from_millis(100))
            .unwrap_err();
        let actual = failure(&error);
        assert!(actual.primary().to_string().contains("timed out"));
        assert!(actual.observation().direct_child_reaped);
        assert!(!actual.observation().reply_observed);
        assert_eq!(client.records()[0]["type"], "native_progress");
        assert_eq!(client.records()[0]["body"], "PRIVATE-LATE-CANARY");
        assert!(!failure_details(&error).unwrap().to_string().contains("PRIVATE"));
        let again = client.finish().unwrap_err();
        assert_eq!(again.to_string(), error.to_string());
        fixture.record(failure_details(&error).unwrap());
        drop(client);
        fixture.dispose_after_known_retirement();
    }

    #[test]
    fn rpc_completed_reply_survives_actual_external_reap_finish_refusal() {
        let fixture = Fixture::new();
        let spec = fixture.script(
            "#!/bin/sh\nread -r request\nprintf '%s\\n' '{\"type\":\"response\",\"id\":\"actuation-prime-1\",\"command\":\"get_state\",\"success\":true,\"data\":{\"token\":\"PRIVATE-REPLY-CANARY\"}}'\nexit 0\n",
            1000, 4096,
        );
        let mut client = RpcClient::start(spec).unwrap();
        let reply = client.exchange(json!({"type":"get_state"}), Duration::from_secs(1)).unwrap();
        assert_eq!(reply["data"]["token"], "PRIVATE-REPLY-CANARY");
        retirement_checkpoint(external_reap);
        let error = client.finish().unwrap_err();
        let actual = failure(&error);
        assert!(actual.observation().reply_observed);
        assert!(actual.observation().signal_forbidden);
        assert!(!actual.observation().kill_signal_attempted);
        assert_eq!(client.records()[0], reply);
        assert!(actual.secondary().any(|(_,error)| actual_io(error)
            .is_some_and(|io| io.raw_os_error() == Some(rustix::io::Errno::CHILD.raw_os_error()))));
        assert!(!failure_details(&error).unwrap().to_string().contains("PRIVATE"));
        fixture.record(failure_details(&error).unwrap());
        drop(client);
        // Unknown owner retirement retains actual completed reply and fixture.
    }

    #[test]
    fn rpc_actual_read_and_external_wait_failures_retain_both_io_causes() {
        let fixture = Fixture::new();
        let spec = fixture.script(
            "#!/bin/sh\nread -r request\nprintf '%s\\n' '{\"type\":\"response\",\"id\":\"actuation-prime-1\",\"command\":\"get_state\",\"success\":true,\"data\":{}}'\nexit 0\n",
            1000, 4096,
        );
        let mut client = RpcClient::start(spec).unwrap();
        client.exchange(json!({"type":"get_state"}), Duration::from_secs(1)).unwrap();
        let write_only = OpenOptions::new().write(true).create_new(true)
            .open(fixture.root.join("actual-write-only-capture")).unwrap();
        client.stdout = write_only;
        retirement_checkpoint(external_reap);
        let error = client.finish().unwrap_err();
        let actual = failure(&error);
        assert_eq!(actual_io(actual.primary()).unwrap().raw_os_error(),
            Some(rustix::io::Errno::BADF.raw_os_error()));
        assert!(actual.secondary().any(|(_,error)| actual_io(error)
            .is_some_and(|io| io.raw_os_error() == Some(rustix::io::Errno::CHILD.raw_os_error()))));
        assert!(actual.observation().capture_read_failed);
        let facts = failure_details(&error).unwrap();
        assert_eq!(facts["primary_io"]["raw_os_error"], rustix::io::Errno::BADF.raw_os_error());
        assert!(facts["secondary"].as_array().unwrap().len() >= 2);
        let actual_prior = File::open(fixture.root.join("actual-prior-absent-member"))
            .unwrap_err();
        assert_eq!(actual_prior.kind(), std::io::ErrorKind::NotFound);
        let combined = io(actual_prior).with_secondary_source(error.clone());
        assert_eq!(actual_io(&combined).unwrap().kind(), std::io::ErrorKind::NotFound);
        let supplemental = failure_details(&combined).unwrap();
        assert_eq!(supplemental["attachment"], "supplemental_source");
        assert_eq!(supplemental["primary_io"]["raw_os_error"],
            rustix::io::Errno::BADF.raw_os_error());
        assert_eq!(supplemental["observation"], facts["observation"]);
        assert!(!format!("{combined:?}").contains("actual-prior-absent-member"));
        fixture.record(json!({"native_failure":facts,"retained_supplemental":supplemental}));
        drop(client);
        // Retain unknown native retirement; no arbitrary kill/wait retry.
    }
}
