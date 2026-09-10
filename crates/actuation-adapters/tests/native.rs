#![cfg(unix)]
use actuation_adapters::*;
use actuation_stream::{BoundaryCatalogue, Timestamp};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

fn environment(root: &Path) -> NativeEffects {
    NativeEffects::in_environment(
        root.to_owned(),
        root.to_owned(),
        OsString::from(root),
        BTreeMap::new(),
    )
}
fn executable(root: &Path, name: &str, body: &str) -> String {
    let path = root.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    path.to_string_lossy().into_owned()
}
fn descriptor(slug: &str, probe: Value) -> HarnessDescriptor {
    HarnessDescriptor::new(json!({"schema":"actuation.harness-detection/v1","document":"descriptor","slug":slug,"native_kind":"harness","probe":probe,"provenance":{"authored_by":"controlled native test","catalog_revision":1}})).unwrap()
}
fn options(versions: bool) -> DetectionOptions {
    DetectionOptions {
        observed_at: Timestamp::new("2026-09-10T00:00:00.000Z").unwrap(),
        probe_versions: versions,
        catalog_revision: json!(6),
    }
}
fn http_server(replies: Vec<(String, u16, String)>) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    listener.set_nonblocking(true).unwrap();
    let thread = std::thread::spawn(move || {
        for (expected, status, body) in replies {
            let deadline = Instant::now() + Duration::from_secs(5);
            let mut stream = loop {
                match listener.accept() {
                    Ok((s, _)) => break s,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            Instant::now() < deadline,
                            "expected native HTTP operation was disconnected"
                        );
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => panic!("{e}"),
                }
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut request = Vec::new();
            let mut buf = [0; 1024];
            while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                let n = stream.read(&mut buf).unwrap();
                assert!(n > 0);
                request.extend_from_slice(&buf[..n]);
                assert!(request.len() <= 8192);
            }
            assert!(String::from_utf8_lossy(&request)
                .starts_with(&format!("GET {expected} HTTP/1.1\r\n")));
            let response=format!("HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
            let _ = stream.write_all(response.as_bytes());
        }
    });
    (address, thread)
}

#[test]
fn catalogue_is_extensible_without_a_new_agent_species() {
    let c = Catalog::embedded().unwrap();
    assert_eq!(c.descriptors().len(), 12);
    assert_eq!(c.capabilities().len(), 3);
    let mut h = c.harness_document().as_value().clone();
    h["descriptors"]
        .as_array_mut()
        .unwrap()
        .push(descriptor("new-body", json!({"executable":{"names":["new-body"]}})).into_value());
    let extended = Catalog::from_documents(
        h,
        c.capability_document().clone(),
        c.secret_catalog().as_value().clone(),
    )
    .unwrap();
    assert!(extended.by_slug("new-body").is_some());
    assert!(extended.capability_by_slug("new-body").is_none());
    assert!(extended.by_slug("ollama").unwrap().native_kind() == "model-provider");
}
#[test]
fn catalogue_refuses_revision_drift_and_undeclared_capability_identity() {
    let c = Catalog::embedded().unwrap();
    let mut caps = c.capability_document().clone();
    caps["catalog_revision"] = json!(7);
    assert!(Catalog::from_documents(
        c.harness_document().as_value().clone(),
        caps,
        c.secret_catalog().as_value().clone()
    )
    .is_err());
    let mut caps = c.capability_document().clone();
    caps["capabilities"][0]["harness_slug"] = json!("unowned");
    assert!(Catalog::from_documents(
        c.harness_document().as_value().clone(),
        caps,
        c.secret_catalog().as_value().clone()
    )
    .is_err());
}
#[test]
fn boundary_translation_uses_the_native_catalogue_and_refuses_undeclared_events() {
    let c = Catalog::embedded().unwrap();
    let cap = c.capability_by_slug("claude-code").unwrap();
    let event = &cap.declared_events()[0];
    let boundary = c
        .boundary("claude-code", event["native_name"].as_str().unwrap())
        .unwrap();
    assert!(boundary.boundary.is_some());
    assert!(c.boundary("claude-code", "invented-event").is_err());
    assert!(c.boundary("pi", "invented-event").is_err());
}
#[test]
fn absent_binary_and_present_config_are_not_an_executable_to_version() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join(".body")).unwrap();
    let d = descriptor(
        "body",
        json!({"executable":{"names":["missing"]},"config-dir":{"path":"~/.body"}}),
    );
    let r = detect(&[d], &mut environment(root.path()), &options(true)).unwrap();
    let e = r.entry("body").unwrap();
    assert_eq!(e["receipts"]["executable_is"], "config-dir");
    assert_eq!(e["state"], "detected");
    assert!(e.get("version").is_none());
    assert!(r.as_value().get("disclosure").is_none());
}
#[test]
fn a_path_containing_diagnostic_words_is_still_measured_presence() {
    let root = tempfile::tempdir().unwrap();
    executable(root.path(), "not found", "printf 'actual-version\\n'");
    let d = descriptor("body", json!({"executable":{"names":["not found"]}}));
    let r = detect(&[d], &mut environment(root.path()), &options(true)).unwrap();
    assert_eq!(r.presence("body"), Some(Presence::Detected));
    assert_eq!(r.entry("body").unwrap()["version"], "actual-version");
}
#[test]
fn ordinary_detection_does_not_execute_a_declared_harness() {
    let root = tempfile::tempdir().unwrap();
    executable(
        root.path(),
        "body",
        "printf ran > invoked\nprintf 'body-version\\n'",
    );
    let d = descriptor("body", json!({"executable":{"names":["body"]}}));
    let r = detect(
        std::slice::from_ref(&d),
        &mut environment(root.path()),
        &options(false),
    )
    .unwrap();
    assert_eq!(r.presence("body"), Some(Presence::Detected));
    assert!(!root.path().join("invoked").exists());
    assert!(r.entry("body").unwrap()["receipts"]["sha256"]
        .as_str()
        .is_some());
    let r = detect(&[d], &mut environment(root.path()), &options(true)).unwrap();
    assert_eq!(r.entry("body").unwrap()["version"], "body-version");
    assert!(root.path().join("invoked").exists());
}
#[test]
fn version_refusal_does_not_fabricate_absence_or_echo_process_material() {
    let root = tempfile::tempdir().unwrap();
    executable(
        root.path(),
        "body",
        "printf 'credential-canary' >&2; exit 9",
    );
    let d = descriptor("body", json!({"executable":{"names":["body"]}}));
    let r = detect(&[d], &mut environment(root.path()), &options(true)).unwrap();
    assert_eq!(r.presence("body"), Some(Presence::Detected));
    assert!(r.entry("body").unwrap().get("version").is_none());
    assert!(r.as_value()["disclosure"].as_array().is_some());
    assert!(!serde_json::to_string(&r)
        .unwrap()
        .contains("credential-canary"));
}
#[test]
fn filesystem_errors_are_not_completed_negative_measurements() {
    let root = tempfile::tempdir().unwrap();
    let link = root.path().join("loop");
    symlink(&link, &link).unwrap();
    let mut e = environment(root.path());
    assert!(matches!(
        e.stat(link.to_str().unwrap()),
        Observation::Unavailable(_)
    ));
    assert!(matches!(
        e.stat(root.path().join("missing").to_str().unwrap()),
        Observation::Absent
    ));
    assert!(matches!(
        e.resolve_executable(&["../outside".into()]),
        Observation::Unavailable(_)
    ));
}
#[test]
fn markers_are_context_evidence_not_agent_identity_or_binary_presence() {
    let root = tempfile::tempdir().unwrap();
    let mut env = BTreeMap::new();
    env.insert("CLAUDECODE".into(), "private-marker-value".into());
    env.insert("CODEX_CI".into(), "other-value".into());
    let mut e = NativeEffects::in_environment(
        root.path().into(),
        root.path().into(),
        root.path().into(),
        env,
    );
    let ds = vec![
        descriptor("a", json!({"env":{"any_of":["CLAUDECODE"]}})),
        descriptor("b", json!({"env":{"any_of":["CODEX_CI"]}})),
    ];
    let r = observe_self(&ds, &mut e, &options(false)).unwrap();
    assert!(r.ambiguous());
    assert!(r.resolved_slug().is_none());
    let rendered = serde_json::to_string(&r).unwrap();
    assert!(!rendered.contains("private-marker-value"));
    assert!(!rendered.contains("other-value"));
    assert!(!rendered.contains("agent_ref"));
    assert_eq!(r.as_value()["detection"]["states"]["a"], "not-installed");
}
#[test]
fn local_http_exchange_observes_names_not_directory_counts_or_model_selection() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(".ollama/models")).unwrap();
    fs::write(
        root.path().join(".ollama/models/unrelated"),
        "not a model name",
    )
    .unwrap();
    let (address,server)=http_server(vec![("/".into(),200,"{}".into()),("/api/tags".into(),200,json!({"models":[{"model":"native-a","name":"alias-a","size":4,"private":"not-retained"},{"model":"native-a","size":6},{"model":"native-b","digest":"d"}]}).to_string())]);
    let mut d = Catalog::embedded()
        .unwrap()
        .by_slug("ollama")
        .unwrap()
        .as_value()
        .clone();
    d["probe"]["service"]["default_url"] = json!(address);
    let r = detect(
        &[HarnessDescriptor::new(d).unwrap()],
        &mut environment(root.path()),
        &options(false),
    )
    .unwrap();
    server.join().unwrap();
    let f = &r.entry("ollama").unwrap()["facets"][0];
    assert_eq!(f["count"], 1);
    assert_eq!(f["inventory_receipt"]["item_count"], 2);
    assert_eq!(f["inventory"][0]["id"], "native-a");
    assert_eq!(f["inventory"][0]["also_known_as"], json!(["alias-a"]));
    assert!(!serde_json::to_string(&r).unwrap().contains("not-retained"));
    assert!(!serde_json::to_string(&r).unwrap().contains("unrelated"));
}
#[test]
fn inventory_refusal_is_not_an_empty_offering() {
    let root = tempfile::tempdir().unwrap();
    let (address, server) = http_server(vec![(
        "/models".into(),
        403,
        "access refused with sensitive body".into(),
    )]);
    let result = environment(root.path())
        .http_json(&format!("{address}/models"))
        .unwrap();
    server.join().unwrap();
    assert!(result.is_err());
    assert!(!result.unwrap_err().to_string().contains("sensitive"));
}
#[test]
fn malformed_and_oversized_http_bodies_fail_with_no_retained_payload() {
    for (body, limit) in [
        ("invalid sensitive body".to_owned(), 1024u64),
        ("x".repeat(8192), 32),
    ] {
        let root = tempfile::tempdir().unwrap();
        let (address, server) = http_server(vec![("/".into(), 200, body)]);
        let mut e = environment(root.path());
        e.max_http_bytes = limit;
        let result = e.http_json(&address).unwrap();
        server.join().unwrap();
        assert!(result.is_err());
    }
}
#[test]
fn inline_credentials_and_unsupported_service_kinds_are_not_probed() {
    let root = tempfile::tempdir().unwrap();
    let mut e = environment(root.path());
    let r = e
        .http_json("https://user:credential-canary@example.test/models")
        .unwrap();
    assert!(r.is_err());
    assert!(!r.unwrap_err().to_string().contains("credential-canary"));
    assert!(matches!(
        e.service(&json!({"kind":"unimplemented"})),
        ServiceObservation::Unavailable(_)
    ));
}
#[test]
fn process_output_limits_and_timeouts_are_executable_not_json_checks() {
    let root = tempfile::tempdir().unwrap();
    let noisy = executable(
        root.path(),
        "noisy",
        "while :; do printf '012345678901234567890123456789'; done",
    );
    let sleeping = executable(root.path(), "sleeping", "/bin/sleep 10");
    let bounds = ProcessBounds {
        timeout: Duration::from_millis(150),
        output_bytes_per_stream: 128,
    };
    for path in [noisy, sleeping] {
        let start = Instant::now();
        let result = run_bounded(Command::new(&path).current_dir(root.path()), bounds);
        assert!(result.is_err());
        assert!(start.elapsed() < Duration::from_secs(2));
    }
}
#[test]
fn inherited_pipes_and_descendants_cannot_hang_a_completed_probe() {
    let root = tempfile::tempdir().unwrap();
    let path = executable(root.path(), "descendant", "/bin/sleep 10 &\nexit 0");
    let start = Instant::now();
    let result = run_bounded(
        &mut Command::new(path),
        ProcessBounds {
            timeout: Duration::from_millis(150),
            output_bytes_per_stream: 128,
        },
    );
    assert!(result.is_err());
    assert!(start.elapsed() < Duration::from_secs(2));
}
#[test]
fn native_env_and_file_effects_return_only_fingerprints() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join(".hidden")).unwrap();
    fs::create_dir(root.path().join("node_modules")).unwrap();
    fs::write(
        root.path().join(".hidden/credentials.json"),
        "sensitive-file-canary",
    )
    .unwrap();
    fs::write(
        root.path().join("node_modules/credentials.json"),
        "excluded",
    )
    .unwrap();
    let mut env = BTreeMap::new();
    env.insert("TEST_TOKEN".into(), "sensitive-env-canary".into());
    env.insert("BORING".into(), "not-selected".into());
    let mut e = NativeEffects::in_environment(
        root.path().into(),
        root.path().into(),
        root.path().into(),
        env,
    );
    let f = e
        .probe("env", &json!({"name_pattern":"TOKEN"}))
        .unwrap()
        .unwrap();
    assert_eq!(f.evidence.len(), 1);
    assert_eq!(
        f.evidence[0].fingerprint,
        Fingerprint::of(b"sensitive-env-canary")
    );
    assert!(!format!("{f:?}").contains("sensitive-env-canary"));
    let f = e
        .probe(
            "file-pattern",
            &json!({"patterns":["credentials*.json"],"roots":["~"]}),
        )
        .unwrap()
        .unwrap();
    assert_eq!(f.evidence.len(), 1);
    assert!(f.evidence[0].location.contains(".hidden"));
    assert!(!format!("{f:?}").contains("sensitive-file-canary"));
}
#[test]
fn bounded_or_unreadable_secret_walk_never_reports_clean_absence() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a"), "a").unwrap();
    fs::write(root.path().join("b"), "b").unwrap();
    let mut e = environment(root.path());
    e.max_files = 1;
    let c = Catalog::embedded().unwrap();
    let o = SecretScanOptions::new(vec![root.path().to_string_lossy().into_owned()]).unwrap();
    let r = scan_secret_sources(c.secret_catalog(), &o, &mut e).unwrap();
    assert_eq!(r.as_value()["coverage"], "partial");
    assert!(r.as_value()["sources"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["state"] == "unavailable"
            && s["unavailable_reason"]
                .as_str()
                .unwrap_or("")
                .contains("budget")));
    let file = root.path().join("a");
    assert!(e
        .probe("file-pattern", &json!({"patterns":["*"],"roots":[file]}))
        .unwrap()
        .is_err());
}
#[test]
fn a_partial_declared_secret_source_stays_partial_even_when_a_match_was_seen() {
    struct Partial;
    impl SecretEffects for Partial {
        fn probe(&mut self, kind: &str, _: &Value) -> Option<Result<SecretFinding>> {
            Some(Ok(if kind == "file-pattern" {
                SecretFinding {
                    truncated: true,
                    files_scanned: 2,
                    evidence: vec![FingerprintEvidence::new("/owned/.env".into(), b"canary")],
                    ..SecretFinding::default()
                }
            } else {
                SecretFinding::default()
            }))
        }
    }
    let c = Catalog::embedded().unwrap();
    let r = scan_secret_sources(
        c.secret_catalog(),
        &SecretScanOptions::new(vec!["/owned".into()]).unwrap(),
        &mut Partial,
    )
    .unwrap();
    assert_eq!(r.as_value()["coverage"], "partial");
    let declared = r.as_value()["sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["slug"] == "varlock-env-files")
        .unwrap();
    assert_eq!(declared["state"], "unavailable");
}
#[test]
fn secret_walk_does_not_follow_symlinks_outside_its_supplied_root() {
    let root = tempfile::tempdir().unwrap();
    let foreign = tempfile::tempdir().unwrap();
    fs::write(foreign.path().join("credentials.json"), "foreign").unwrap();
    symlink(
        foreign.path().join("credentials.json"),
        root.path().join("credentials.json"),
    )
    .unwrap();
    assert!(environment(root.path())
        .probe(
            "file-pattern",
            &json!({"patterns":["credentials*.json"],"roots":["~"]})
        )
        .unwrap()
        .is_err());
}
#[test]
fn vault_failure_is_unknown_access_not_absent_item_and_never_echoes_material() {
    let root = tempfile::tempdir().unwrap();
    executable(
        root.path(),
        "op",
        "printf 'not signed in sensitive-secret-canary' >&2; exit 1",
    );
    let mut e = environment(root.path());
    let r = e
        .probe("vault-item", &json!({"item_ref":"op://test/item"}))
        .unwrap();
    assert!(r.is_err());
    assert!(!r
        .unwrap_err()
        .to_string()
        .contains("sensitive-secret-canary"));
}
#[test]
fn vault_success_discards_value_bearing_fields_at_the_effect_boundary() {
    let root = tempfile::tempdir().unwrap();
    let raw = json!({"id":"item-1","vault":{"name":"test"},"fields":[{"value":"sensitive-secret-canary"}]});
    executable(root.path(), "op", &format!("printf '%s' '{}'", raw));
    let r = environment(root.path())
        .probe("vault-item", &json!({"item_ref":"op://test/item"}))
        .unwrap()
        .unwrap();
    assert!(r.found);
    assert_eq!(
        r.vault_item,
        Some((Some("item-1".into()), Some("test".into())))
    );
    assert!(!format!("{r:?}").contains("sensitive-secret-canary"));
}
#[test]
fn strict_receipt_admission_does_not_manufacture_agency_from_a_provider() {
    let source: Value =
        serde_json::from_str(include_str!("../../../fixtures/migration/oracle.json")).unwrap();
    let valid = source["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| {
            r["operation"] == "contracts/instantiation.mjs#validateActuationReceipt"
                && r["expected"]["ok"] == true
        })
        .unwrap()["args"][0]
        .clone();
    for field in ["agency_ref", "world_binding_ref", "actuation_ref"] {
        let mut v = valid.clone();
        v.as_object_mut().unwrap().remove(field);
        assert!(InstantiationReceipt::new(v).is_err());
    }
    let original = InstantiationReceipt::new(valid.clone()).unwrap();
    let mut moved = valid;
    moved["model_relation"]["material"] =
        json!({"binding_ref":"workcell:elsewhere","placement":"remote"});
    moved["model_relation"]["model_ref"] = json!("provider-native:replacement");
    let moved = InstantiationReceipt::new(moved).unwrap();
    assert_eq!(original.agency_ref(), moved.agency_ref());
    assert_eq!(original.world_binding_ref(), moved.world_binding_ref());
    assert_eq!(original.actuation_ref(), moved.actuation_ref());
}
#[test]
fn model_access_reading_does_not_grant_control_or_override_explicit_denial() {
    let p=ModelAccessProfile::new(json!({"schema":"actuation.instantiation/v1","inference":{"allowed":["generate"],"denied":["generate"]},"control":{"allowed":[]},"interior":{"depth":"opaque"}})).unwrap();
    assert!(!p.records_inference("generate"));
    assert!(!p.records_inference("change-weights"));
}

#[test]
fn file_fingerprints_have_an_actual_read_bound() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("credentials.json"), "longer-than-limit").unwrap();
    let mut e = environment(root.path());
    e.max_fingerprint_bytes = 4;
    assert!(e
        .probe(
            "file-pattern",
            &json!({"patterns":["credentials*.json"],"roots":["~"]})
        )
        .unwrap()
        .is_err());
}
#[test]
fn unsupported_or_stalled_http_operation_returns_unavailability_within_bound() {
    let root = tempfile::tempdir().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let mut e = environment(root.path());
    e.http_timeout = Duration::from_millis(100);
    let start = Instant::now();
    assert!(e.http_json(&address).unwrap().is_err());
    assert!(start.elapsed() < Duration::from_secs(2));
    drop(listener);
}
