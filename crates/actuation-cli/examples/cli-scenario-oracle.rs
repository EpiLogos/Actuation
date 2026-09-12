//! Scenario transport for the frozen `cli` kind of the migration corpus. Reads
//! `{id, kind, input}` rows on stdin, executes each through the real
//! `actuation` executable under the controlled environment the corpus was
//! captured with, and answers one `{id, value}` row per case. This is a
//! conformance transport only — the product never reads scenario fixtures.
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::process::Command;

fn main() -> std::io::Result<()> {
    let binary = std::env::var("ACTUATION_CLI_SCENARIO_BIN")
        .unwrap_or_else(|_| "target/debug/actuation".to_owned());
    let binary = std::path::PathBuf::from(&binary)
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(&binary));
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let row: Value = serde_json::from_str(&line).expect("scenario row must be JSON");
        let answer = match row["kind"].as_str() {
            Some("cli") => run_cli_scenario(&binary, &row["input"]),
            other => Err(format!("cli scenario runner cannot answer kind {other:?}")),
        };
        let value =
            answer.unwrap_or_else(|error| json!({"code": -1, "stdout": "", "stderr": error}));
        writeln!(stdout, "{}", json!({"id": row["id"], "value": value}))?;
    }
    stdout.flush()
}

fn run_cli_scenario(binary: &std::path::Path, input: &Value) -> Result<Value, String> {
    let directory = tempfile::tempdir().map_err(|e| e.to_string())?;
    let root = directory.path();

    let files = input["files"]
        .as_object()
        .map(|files| {
            files
                .iter()
                .map(|(name, text)| {
                    if name.contains('/') || name == ".." {
                        Err("fixture file must be a basename".to_owned())
                    } else {
                        std::fs::write(root.join(name), text.as_str().unwrap_or_default())
                            .map_err(|e| e.to_string())
                    }
                })
                .collect::<Result<Vec<_>, String>>()
        })
        .unwrap_or_else(|| Ok(Vec::new()))?;
    let _ = files;

    let store = root.to_string_lossy().into_owned();
    let argv: Vec<String> = input["argv"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .map(|arg| arg.as_str().unwrap_or_default().replace("$STORE", &store))
                .collect()
        })
        .ok_or("cli scenario requires argv")?;

    let stdin_text = input["stdin"].as_str().unwrap_or_default();
    let mut child = Command::new(binary)
        .args(&argv)
        .current_dir(root)
        .env_clear()
        .env("PATH", "")
        .env("HOME", root)
        .env("ACTUATION_STREAM_STORE", &store)
        .env("LANG", "C.UTF-8")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    // Supply the scenario's stdin, then close so the child sees EOF.
    child
        .stdin
        .take()
        .ok_or("child stdin unavailable")?
        .write_all(stdin_text.as_bytes())
        .map_err(|e| e.to_string())?;
    let run = child.wait_with_output().map_err(|e| e.to_string())?;

    let code = run.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&run.stdout).trim_end().to_owned();
    let stdout = match serde_json::from_str::<Value>(&stdout) {
        // Key order is not wire semantics; the revision claim is relative to
        // the serving checkout and normalised, exactly as when captured.
        Ok(Value::Object(mut parsed)) => {
            if parsed.contains_key("revision") {
                parsed.insert("revision".into(), json!("$REVISION"));
            }
            Value::Object(parsed)
        }
        _ => json!(stdout),
    };
    let stderr = String::from_utf8_lossy(&run.stderr).replace(&store, "$STORE");
    Ok(json!({"code": code, "stdout": stdout, "stderr": stderr}))
}
