//! `harness detect --versions` must actually probe versions.
//!
//! The flag was parsed and discarded, so the CLI silently emitted a detection
//! record with no `version` for any detected harness — the version half of the
//! census was unreachable from the command surface even though the detection
//! engine has implemented it (and is covered for it at the adapter level).
//! This test stands a fixture harness on PATH so the probe is hermetic: it
//! proves the flag reaches the options, not that any particular harness is
//! installed on the machine.

use assert_cmd::cargo::cargo_bin;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[cfg(unix)]
fn fixture(path: &Path, body: &str) {
    use std::os::unix::fs::PermissionsExt;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture directory");
    }
    fs::write(path, body).expect("fixture script");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("fixture mode");
}

#[cfg(unix)]
fn detect(dir: &Path, extra: &[&str]) -> Value {
    let path = format!(
        "{}:{}",
        dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut args = vec!["harness", "detect", "--json", "--only", "codex"];
    args.extend_from_slice(extra);
    let output = std::process::Command::new(cargo_bin("actuation"))
        .args(&args)
        .env("PATH", path)
        .output()
        .expect("detection runs");
    assert!(
        output.status.success(),
        "detection failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("detection emits JSON")
}

#[cfg(unix)]
#[test]
fn versions_flag_probes_the_declared_version_args_and_plain_detection_does_not() {
    let tmp = tempfile::tempdir().expect("temp dir");
    fixture(
        &tmp.path().join("codex"),
        "#!/bin/sh\nprintf '9.9.9-fixture\\n'\n",
    );

    let plain = detect(tmp.path(), &[]);
    let harness = &plain["harnesses"][0];
    assert_eq!(harness["state"], "detected");
    assert!(
        harness.get("version").is_none(),
        "an unrequested version probe must stay absent: {harness}"
    );

    let probed = detect(tmp.path(), &["--versions"]);
    let harness = &probed["harnesses"][0];
    assert_eq!(harness["state"], "detected");
    assert_eq!(
        harness["version"], "9.9.9-fixture",
        "the declared version_args must be executed when --versions is asked for"
    );
}
