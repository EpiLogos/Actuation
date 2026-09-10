use crate::{Error, Result};
use std::io::Read;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::time::{Duration, Instant};

/// A bounded native operation, not a session host or execution arrangement.
/// Commands are supplied as executable/argument vectors, never a shell string.
#[derive(Clone, Copy, Debug)]
pub struct ProcessBounds {
    pub timeout: Duration,
    pub output_bytes_per_stream: usize,
}
impl Default for ProcessBounds {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            output_bytes_per_stream: 4 * 1024 * 1024,
        }
    }
}
/// Transient native bytes. Deliberately not serializable: each observation
/// adapter decides which facts are admissible in a public receipt.
pub struct NativeOutput {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}
fn read_pipe(
    mut pipe: impl Read + Send + 'static,
    limit: usize,
    exceeded: Arc<AtomicBool>,
) -> mpsc::Receiver<std::io::Result<Vec<u8>>> {
    let (send, receive) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let result = (|| {
            let mut output = Vec::new();
            let mut buffer = [0u8; 8192];
            loop {
                let length = pipe.read(&mut buffer)?;
                if length == 0 {
                    return Ok(output);
                }
                let available = limit.saturating_sub(output.len());
                output.extend_from_slice(&buffer[..length.min(available)]);
                if length > available {
                    exceeded.store(true, Ordering::Release);
                }
            }
        })();
        let _ = send.send(result);
    });
    receive
}
fn stop_group(child: &mut Child) {
    #[cfg(unix)]
    if let Some(pid) = rustix::process::Pid::from_raw(child.id() as i32) {
        let _ = rustix::process::kill_process_group(pid, rustix::process::Signal::KILL);
    }
    let _ = child.kill();
    // Reap after the process has actually exited. No unbounded wait on a
    // descendant retaining a pipe or on a platform's unsupported control.
    let deadline = Instant::now() + Duration::from_millis(500);
    while Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(_)) | Err(_) => break,
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
        }
    }
}
pub fn run_bounded(command: &mut Command, bounds: ProcessBounds) -> Result<NativeOutput> {
    if bounds.timeout.is_zero() || bounds.output_bytes_per_stream == 0 {
        return Err(Error::new("native process bounds must be positive"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|e| Error::new(format!("native process could not start: {e}")))?;
    let exceeded = Arc::new(AtomicBool::new(false));
    let stdout = read_pipe(
        child.stdout.take().unwrap(),
        bounds.output_bytes_per_stream,
        exceeded.clone(),
    );
    let stderr = read_pipe(
        child.stderr.take().unwrap(),
        bounds.output_bytes_per_stream,
        exceeded.clone(),
    );
    let deadline = Instant::now() + bounds.timeout;
    let status = loop {
        if exceeded.load(Ordering::Acquire) {
            stop_group(&mut child);
            return Err(Error::new("native process output limit exceeded"));
        }
        if Instant::now() >= deadline {
            stop_group(&mut child);
            return Err(Error::new("native process timeout"));
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(e) => {
                stop_group(&mut child);
                return Err(Error::new(format!(
                    "native process observation failed: {e}"
                )));
            }
        }
    };
    let receive = |rx: mpsc::Receiver<std::io::Result<Vec<u8>>>| -> Result<Vec<u8>> {
        rx.recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .map_err(|_| Error::new("native process pipe did not close within its deadline"))?
            .map_err(|e| Error::new(format!("native process pipe could not be read: {e}")))
    };
    let result = receive(stdout).and_then(|out| receive(stderr).map(|err| (out, err)));
    let (stdout, stderr) = match result {
        Ok(result) => result,
        Err(error) => {
            stop_group(&mut child);
            return Err(error);
        }
    };
    if exceeded.load(Ordering::Acquire) {
        return Err(Error::new("native process output limit exceeded"));
    }
    Ok(NativeOutput {
        status,
        stdout,
        stderr,
    })
}
