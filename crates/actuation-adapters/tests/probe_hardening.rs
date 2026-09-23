//! Probe hardening: every probe that invokes anything external ends inside a
//! hard wall-clock bound, and every failure class carries its named outcome
//! (`ok | credential-gated | unreachable | unsupported | timed-out | refused`)
//! instead of an error bubble or a silent pass. The regression class here is
//! real: gemini 0.29.5 with an expired oauth session hung 30-90s with empty
//! stdout/stderr on every init-bearing probe.

mod common;
mod support;
use actuation_adapters::*;
use common::*;
use serde_json::json;
use std::time::{Duration, Instant};

fn descriptor(slug: &str, probe: serde_json::Value) -> HarnessDescriptor {
    HarnessDescriptor::try_from(json!({
        "schema": "actuation.harness-detection/v1",
        "document": "descriptor",
        "slug": slug,
        "native_kind": "harness",
        "probe": probe,
        "provenance": {
            "authored_by": "probe-hardening acceptance fixture",
            "catalog_revision": 1
        }
    }))
    .expect("admitted descriptor")
}

fn version_record(entry: &serde_json::Value) -> &serde_json::Value {
    entry["probes"]
        .as_array()
        .expect("probe records")
        .iter()
        .find(|p| p["kind"] == "version")
        .expect("a version probe record is carried")
}

#[cfg(unix)]
#[test]
fn sleeping_version_probe_returns_timed_out_within_the_declared_bound() {
    // (a) The gemini hang class: the target produces no output and never
    // exits. The descriptor declares a 1s bound; the probe must end inside
    // it with the named outcome, and presence must stay detected.
    let tmp = tempfile::tempdir().unwrap();
    script(
        &tmp.path().join("bin/slow-harness"),
        "#!/bin/sh\nexec /bin/sleep 60\n",
    );
    let catalog = NativeCatalog::bundled().unwrap();
    let selected = vec![descriptor(
        "slow-harness",
        json!({"executable": {"names": ["slow-harness"], "timeout_ms": 1000}}),
    )];
    let mut o = options(&catalog);
    o.probe_versions = true;
    let started = Instant::now();
    let read = run_detection(&selected, &mut effects(tmp.path()), &o).unwrap();
    let elapsed = started.elapsed();
    let entry = &read.as_value()["harnesses"][0];

    assert_eq!(
        entry["state"], "detected",
        "a hung version probe must not unsee an installed binary"
    );
    assert_eq!(
        entry["receipts"]["executable"],
        json!(tmp
            .path()
            .join("bin/slow-harness")
            .to_string_lossy()
            .to_string()),
        "the same-run presence receipt survives the hang"
    );
    let record = version_record(entry);
    assert_eq!(record["result"], "fail");
    assert_eq!(record["outcome"], "timed-out");
    assert_eq!(
        record["detail"],
        json!("process observation timed out after 1s")
    );
    assert!(
        entry.get("version").is_none(),
        "no version may be fabricated from a hung probe"
    );
    let disclosure = read.as_value()["disclosure"].as_array().unwrap();
    assert!(
        disclosure.iter().any(|l| l
            .as_str()
            .unwrap()
            .contains("slow-harness: version probe timed-out")),
        "disclosure names the outcome: {disclosure:?}"
    );
    assert!(
        elapsed < Duration::from_secs(5),
        "the probe ended inside its declared bound, not after the target's own hang: {elapsed:?}"
    );
}

#[cfg(unix)]
#[test]
fn unspawnable_executable_returns_unreachable() {
    // (b) The binary resolves on PATH and is stat-executable, but the exec
    // itself fails (a shebang naming an interpreter that does not exist).
    // The probe names the class instead of bubbling a raw error.
    let tmp = tempfile::tempdir().unwrap();
    script(
        &tmp.path().join("bin/broken-harness"),
        "#!/nonexistent-interpreter\n",
    );
    let catalog = NativeCatalog::bundled().unwrap();
    let selected = vec![descriptor(
        "broken-harness",
        json!({"executable": {"names": ["broken-harness"]}}),
    )];
    let mut o = options(&catalog);
    o.probe_versions = true;
    let read = run_detection(&selected, &mut effects(tmp.path()), &o).unwrap();
    let entry = &read.as_value()["harnesses"][0];
    assert_eq!(entry["state"], "detected");
    let record = version_record(entry);
    assert_eq!(record["outcome"], "unreachable");
    assert!(
        record["detail"]
            .as_str()
            .unwrap()
            .starts_with("spawn failed"),
        "the spawn failure is the evidence: {}",
        record["detail"]
    );
}

#[test]
fn refused_version_probe_names_refused() {
    // The target ran and exited non-zero: it refused the probe.
    let catalog = NativeCatalog::bundled().unwrap();
    let selected = vec![descriptor(
        "fixture",
        json!({"executable": {"names": ["fixture"]}}),
    )];
    let mut fixture = support::FixtureEffects::new(json!({
        "resolveExecutable": {"value": {"found": true, "path": "/fixture/harness"}},
        "statProbe": {"value": {"exists": true, "isDir": false, "size": 4, "mtimeMs": 1.0}},
        "hashProbe": {"value": {"ok": true, "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}},
        "versionProbe": {"value": {"ok": false, "reason": "exit 1"}}
    }));
    let mut o = options(&catalog);
    o.probe_versions = true;
    let read = run_detection(&selected, &mut fixture, &o).unwrap();
    let entry = &read.as_value()["harnesses"][0];
    assert_eq!(entry["state"], "detected");
    let record = version_record(entry);
    assert_eq!(record["outcome"], "refused");
    let disclosure = read.as_value()["disclosure"].as_array().unwrap();
    assert!(
        disclosure
            .iter()
            .any(|l| l.as_str().unwrap() == "fixture: version probe refused (exit 1)"),
        "{disclosure:?}"
    );
}

#[test]
fn unsupported_service_observation_names_unsupported() {
    // A probe that cannot apply to the declared target is unsupported, not a
    // generic failure.
    let catalog = NativeCatalog::bundled().unwrap();
    let selected = vec![descriptor(
        "fixture",
        json!({"service": {"kind": "telepathy"}}),
    )];
    let mut fixture = support::FixtureEffects::new(json!({
        "serviceProbe": {"value": {"ok": false, "detail": "unsupported service kind"}}
    }));
    let read = run_detection(&selected, &mut fixture, &options(&catalog)).unwrap();
    let entry = &read.as_value()["harnesses"][0];
    let record = entry["probes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["kind"] == "service")
        .unwrap();
    assert_eq!(record["result"], "fail");
    assert_eq!(record["outcome"], "unsupported");
}

#[test]
fn credential_file_presence_flips_the_disclosure() {
    // (d) A declared credential signal is surfaced as credential-gated on
    // presence. The file is stat'ed, never read: no content may leak into
    // the detection record.
    let tmp = tempfile::tempdir().unwrap();
    let catalog = NativeCatalog::bundled().unwrap();
    let mut selected_value = json!([descriptor(
        "gated-harness",
        json!({"executable": {"names": ["gated-harness"]}}),
    )
    .as_value()
    .clone()]);
    selected_value[0]["credential"] = json!({"path": "~/auth/credentials.json"});

    let selected = vec![HarnessDescriptor::try_from(selected_value[0].clone()).expect("admitted")];

    // Absent credential: the outcome is named absent, and disclosure is silent.
    let read = run_detection(&selected, &mut effects(tmp.path()), &options(&catalog)).unwrap();
    let entry = &read.as_value()["harnesses"][0];
    assert_eq!(entry["credential"]["outcome"], "absent");
    assert_eq!(
        entry["credential"]["path"],
        json!("~/auth/credentials.json")
    );
    assert!(read.as_value().get("disclosure").is_none());

    // Present credential: credential-gated in the entry and the disclosure,
    // and the file's contents never appear anywhere in the record.
    std::fs::create_dir_all(tmp.path().join("auth")).unwrap();
    std::fs::write(
        tmp.path().join("auth/credentials.json"),
        "{\"token\": \"pretend-secret-value\"}",
    )
    .unwrap();
    let read = run_detection(&selected, &mut effects(tmp.path()), &options(&catalog)).unwrap();
    let entry = &read.as_value()["harnesses"][0];
    assert_eq!(entry["credential"]["outcome"], "credential-gated");
    let disclosure = read.as_value()["disclosure"].as_array().unwrap();
    assert!(
        disclosure.iter().any(|l| l
            .as_str()
            .unwrap()
            .contains("gated-harness: credential-gated (~/auth/credentials.json present")),
        "{disclosure:?}"
    );
    let record = read.as_value().to_string();
    assert!(
        !record.contains("pretend-secret-value"),
        "presence only: the credential content must never be read or rendered"
    );
}

#[test]
fn declared_probe_bounds_and_credential_declarations_are_validated() {
    let catalog = NativeCatalog::bundled().unwrap();
    let base = |probe: serde_json::Value| {
        json!({
            "schema": "actuation.harness-detection/v1",
            "document": "descriptor",
            "slug": "fixture",
            "native_kind": "harness",
            "probe": probe,
            "provenance": {"authored_by": "probe-hardening acceptance fixture", "catalog_revision": 1}
        })
    };
    let ok = base(json!({"executable": {"names": ["x"], "timeout_ms": 1000}}));
    assert!(HarnessDescriptor::try_from(ok).is_ok());
    for bad in [0, -5, 600_001] {
        let v = base(json!({"executable": {"names": ["x"], "timeout_ms": bad}}));
        let err = HarnessDescriptor::try_from(v).unwrap_err().to_string();
        assert!(
            err.contains("timeout_ms"),
            "a bound outside the declared range is refused: {err}"
        );
    }
    let mut with_credential = base(json!({"executable": {"names": ["x"]}}));
    with_credential["credential"] = json!({"path": "~/.fixture/auth.json"});
    assert!(HarnessDescriptor::try_from(with_credential).is_ok());
    let mut credential_without_path = base(json!({"executable": {"names": ["x"]}}));
    credential_without_path["credential"] = json!({"hint": "no path declared"});
    assert!(HarnessDescriptor::try_from(credential_without_path).is_err());

    // The bundled catalog's own gemini declaration admits with the credential
    // presence signal, and the read model renders the documented state.
    let gemini = catalog.descriptor("gemini").unwrap().as_value().clone();
    assert_eq!(
        gemini["credential"]["path"],
        json!("~/.gemini/oauth_creds.json"),
        "the real hang case carries a cheap presence signal"
    );
}

#[test]
fn a_version_probe_record_admits_and_a_bad_outcome_does_not() {
    // The detection wire accepts the version probe record and the credential
    // block; an unknown outcome class is refused at admission.
    let now = "2026-09-10T00:00:00.000Z";
    let harness = |probes: serde_json::Value, credential: serde_json::Value| {
        json!({
            "schema": "actuation.harness-detection/v1",
            "document": "detection",
            "detection_ref": format!("detection:{now}"),
            "observed_at": now,
            "catalog_revision": 16,
            "detector": {"implementation": "actuation surface-probes", "version": "0.2.0"},
            "harnesses": [{
                "slug": "fixture",
                "harness_ref": "harness/fixture",
                "native_kind": "harness",
                "state": "detected",
                "probes": probes,
                "credential": credential,
                "receipts": {"executable": "/fixture/harness"}
            }],
            "absent": [],
            "availability": "complete"
        })
    };
    let good = harness(
        json!([
            {"kind": "executable", "result": "pass", "spec": "fixture", "detail": "/fixture/harness"},
            {"kind": "version", "result": "fail", "outcome": "timed-out", "spec": "--version", "detail": "process observation timed out after 10s"}
        ]),
        json!({"path": "~/.fixture/auth.json", "outcome": "credential-gated"}),
    );
    assert!(HarnessDetection::try_from(good).is_ok());

    let bad_outcome = harness(
        json!([
            {"kind": "executable", "result": "pass", "spec": "fixture", "detail": "/fixture/harness"},
            {"kind": "version", "result": "fail", "outcome": "exploded", "spec": "--version", "detail": "x"}
        ]),
        json!({"path": "~/.fixture/auth.json", "outcome": "absent"}),
    );
    let err = HarnessDetection::try_from(bad_outcome)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("expected one of"),
        "only the shared outcome vocabulary admits: {err}"
    );
}
