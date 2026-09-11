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
    Error::new(format!("research process: {}", e.kind()))
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
    pub output_limit: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct ProcessResult {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}
struct OwnedChild {
    child: Child,
    stopped: bool,
}
impl OwnedChild {
    fn new(child: Child) -> Self {
        Self {
            child,
            stopped: false,
        }
    }
}
impl OwnedChild {
    fn stop(&mut self) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        #[cfg(unix)]
        if let Some(p) = rustix::process::Pid::from_raw(self.child.id() as i32) {
            let _ = rustix::process::kill_process_group(p, rustix::process::Signal::KILL);
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        self.stop();
    }
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
    pub fn run(&self, input: &[u8]) -> Result<ProcessResult> {
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
        let mut child = OwnedChild::new(c.spawn().map_err(io)?);
        let start = Instant::now();
        let status = loop {
            check_size(&stdout, &stderr, self.output_limit)?;
            if let Some(s) = child.child.try_wait().map_err(io)? {
                break s;
            }
            if start.elapsed() >= Duration::from_millis(self.timeout_ms) {
                return Err(Error::new("specimen process timed out"));
            }
            thread::sleep(Duration::from_millis(5));
        };
        child.stop();
        check_size(&stdout, &stderr, self.output_limit)?;
        Ok(ProcessResult {
            code: status.code(),
            stdout: read_final(&mut stdout, self.output_limit)?,
            stderr: read_final(&mut stderr, self.output_limit)?,
        })
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
    String::from_utf8(b).map_err(|_| Error::new("specimen output is not UTF8"))
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
}
impl RpcClient {
    pub fn start(spec: ProcessSpec) -> Result<Self> {
        let stdout = tempfile::tempfile().map_err(io)?;
        let stderr = tempfile::tempfile().map_err(io)?;
        let mut c = spec.command()?;
        c.stdin(Stdio::piped())
            .stdout(stdout.try_clone().map_err(io)?)
            .stderr(stderr.try_clone().map_err(io)?);
        let mut child = OwnedChild::new(c.spawn().map_err(io)?);
        let stdin = child
            .child
            .stdin
            .take()
            .ok_or_else(|| Error::new("RPC stdin unavailable"))?;
        let flags =
            rustix::fs::fcntl_getfl(&stdin).map_err(|_| Error::new("cannot inspect RPC pipe"))?;
        rustix::fs::fcntl_setfl(&stdin, flags | rustix::fs::OFlags::NONBLOCK)
            .map_err(|_| Error::new("cannot bound RPC pipe"))?;
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
        let result = self.request_inner(command, timeout);
        if result.is_err() {
            self.stop();
        }
        result
    }
    pub fn request(&mut self, command: Value, timeout: Duration) -> Result<Value> {
        let result = self.exchange(command, timeout).and_then(|reply| {
            if reply["success"] != true {
                Err(Error::new(
                    "specimen refused RPC request; retained in raw record",
                ))
            } else {
                Ok(reply)
            }
        });
        if result.is_err() {
            self.stop();
        }
        result
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
                self.stop();
                return Err(Error::new("RPC request timed out"));
            }
            if written < bytes.len() {
                match self.stdin.write(&bytes[written..]) {
                    Ok(0) => {
                        self.stop();
                        return Err(Error::new("RPC input closed"));
                    }
                    Ok(n) => written += n,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(e) => {
                        self.stop();
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
            if self.child.child.try_wait().map_err(io)?.is_some() {
                self.stop();
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
    pub fn stop(&mut self) {
        if !self.stopped {
            self.child.stop();
            self.stopped = true;
            let _ = self.consume();
        }
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
        let (_dir, s) = responder("#!/bin/sh\nexit 0\n");
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
