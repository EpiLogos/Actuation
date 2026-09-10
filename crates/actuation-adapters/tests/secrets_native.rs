mod common;
use actuation_adapters::{effects::*, secrets::*, *};
use common::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs};
fn controlled_catalog(probe: Value) -> NativeCatalog {
    let mut v = catalog_value();
    let mut d = v["secret_sources"]["descriptors"][1].clone();
    d["slug"] = json!("controlled-source");
    d["probe"] = probe;
    v["secret_sources"]["descriptors"] = json!([d]);
    NativeCatalog::from_json(&v.to_string()).unwrap()
}
fn scan_options(root: &std::path::Path) -> ScanOptions {
    ScanOptions {
        roots: vec![root.to_string_lossy().into_owned()],
        observed_at: serde_json::from_value(json!("2026-09-10T12:00:00Z")).unwrap(),
        scanner_implementation: "controlled-native-proof".into(),
        scanner_version: "1".into(),
    }
}
#[test]
fn native_environment_scan_emits_fingerprints_not_values_and_never_resolves_credentials() {
    let tmp = tempfile::tempdir().unwrap();
    let sentinel = "private-credential-λ-do-not-capture";
    let env = BTreeMap::from([
        ("DECLARED_API_KEY".into(), sentinel.into()),
        (
            "UNDECLARED_TOKEN".into(),
            "another-private-credential".into(),
        ),
    ]);
    let mut e = NativeEffects::new(Some(tmp.path().into()), tmp.path().into(), vec![], env);
    let catalog = controlled_catalog(json!({"env":{"names":["DECLARED_API_KEY"]}}));
    let scan = scan_secret_sources(&catalog, &mut e, &scan_options(tmp.path())).unwrap();
    let wire = scan.as_value();
    assert_eq!(wire["coverage"], "complete");
    assert_eq!(wire["sources"][0]["state"], "verified");
    let evidence = &wire["sources"][0]["evidence"][0];
    assert_eq!(
        evidence["fingerprint_sha256"],
        format!("{:x}", Sha256::digest(sentinel.as_bytes()))
    );
    assert_eq!(evidence["byte_length"], sentinel.len());
    assert_eq!(wire["violations"].as_array().unwrap().len(), 1);
    assert!(!wire.to_string().contains(sentinel));
    assert!(!wire.to_string().contains("another-private-credential"));
    assert!(wire["sources"]
        .as_array()
        .unwrap()
        .iter()
        .all(|s| s["state"] != "absent"));
}
#[test]
fn native_fingerprint_walk_covers_hidden_files_and_explicitly_discloses_truncation() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join(".hidden")).unwrap();
    fs::create_dir_all(tmp.path().join("node_modules")).unwrap();
    fs::write(
        tmp.path().join(".hidden/credentials.json"),
        "secret material",
    )
    .unwrap();
    fs::write(
        tmp.path().join("node_modules/credentials.json"),
        "not part of declared walk",
    )
    .unwrap();
    let mut e = effects(tmp.path());
    let spec = json!({"patterns":["credentials*.json"],"roots":[tmp.path()],"max_files":10});
    let read = e.inspect("file-pattern", &spec).unwrap();
    assert!(read.present);
    assert!(!read.truncated);
    assert_eq!(read.evidence.len(), 1);
    assert_eq!(read.evidence[0].byte_length, 15);
    assert!(read.evidence[0].location.contains(".hidden"));
    let mut tight = spec.clone();
    tight["max_files"] = json!(0);
    let read = e.inspect("file-pattern", &tight).unwrap();
    assert!(read.truncated);
    assert_eq!(read.files_scanned, 0);
    let catalog = controlled_catalog(
        json!({"env":{"names":["DECLARED_API_KEY"]},"file-pattern":{"patterns":["credentials*.json"],"roots":[tmp.path()],"max_files":0}}),
    );
    let mut e = NativeEffects::new(
        Some(tmp.path().into()),
        tmp.path().into(),
        vec![],
        BTreeMap::from([(
            "DECLARED_API_KEY".into(),
            "present-but-not-total-coverage".into(),
        )]),
    );
    let scan = scan_secret_sources(&catalog, &mut e, &scan_options(tmp.path())).unwrap();
    assert_eq!(scan.as_value()["sources"][0]["state"], "unavailable");
    assert_eq!(scan.as_value()["coverage"], "partial");
}
#[cfg(unix)]
#[test]
fn native_secret_walk_does_not_follow_symlinks_or_read_special_files() {
    use std::os::unix::fs::symlink;
    let tmp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("real"), "outside-private-material").unwrap();
    symlink(
        outside.path().join("real"),
        tmp.path().join("credentials.json"),
    )
    .unwrap();
    symlink(outside.path(), tmp.path().join("hidden-directory")).unwrap();
    let read = effects(tmp.path())
        .inspect(
            "file-pattern",
            &json!({"patterns":["credentials*.json"],"roots":[tmp.path()]}),
        )
        .unwrap();
    assert!(!read.present);
    assert!(read.evidence.is_empty());
}
#[test]
fn fingerprint_limits_return_unavailability_not_partial_content_or_a_fake_hash() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("credentials.json");
    let f = fs::File::create(&path).unwrap();
    f.set_len(64 * 1024 * 1024 + 1).unwrap();
    let error = effects(tmp.path())
        .inspect(
            "file-pattern",
            &json!({"patterns":["credentials*.json"],"roots":[tmp.path()]}),
        )
        .unwrap_err();
    assert!(error.contains("fingerprint budget"));
    let catalog = controlled_catalog(
        json!({"file-pattern":{"patterns":["credentials*.json"],"roots":[tmp.path()]}}),
    );
    let scan = scan_secret_sources(
        &catalog,
        &mut effects(tmp.path()),
        &scan_options(tmp.path()),
    )
    .unwrap();
    assert_eq!(scan.as_value()["coverage"], "partial");
    assert_eq!(scan.as_value()["sources"][0]["state"], "unavailable");
    assert!(scan.as_value()["sources"][0]["evidence"].is_null());
}
#[cfg(unix)]
#[test]
fn native_vault_effect_keeps_only_declared_metadata_and_discloses_authentication_failure() {
    let tmp = tempfile::tempdir().unwrap();
    let op = tmp.path().join("bin/op");
    script(&op,"#!/bin/sh\nprintf '%s' '{\"id\":\"controlled-id\",\"vault\":{\"name\":\"controlled-vault\"},\"fields\":[{\"value\":\"NEVER-EMIT-PRIVATE\"}]}'\n");
    let read = effects(tmp.path())
        .inspect("vault-item", &json!({"item_ref":"op://controlled/item"}))
        .unwrap();
    assert!(read.present);
    assert_eq!(
        read.detail.as_deref(),
        Some("item controlled-id in vault controlled-vault")
    );
    assert!(read.evidence.is_empty());
    let catalog = controlled_catalog(json!({"vault-item":{"item_ref":"op://controlled/item"}}));
    let scan = scan_secret_sources(
        &catalog,
        &mut effects(tmp.path()),
        &scan_options(tmp.path()),
    )
    .unwrap();
    assert!(!scan.as_value().to_string().contains("NEVER-EMIT-PRIVATE"));
    script(
        &op,
        "#!/bin/sh\nprintf 'not signed in: PRIVATE-AUTH-TEXT' >&2\nexit 1\n",
    );
    let error = effects(tmp.path())
        .inspect("vault-item", &json!({"item_ref":"op://controlled/item"}))
        .unwrap_err();
    assert!(error.contains("unavailable"));
    assert!(!error.contains("PRIVATE-AUTH-TEXT"));
    let scan = scan_secret_sources(
        &catalog,
        &mut effects(tmp.path()),
        &scan_options(tmp.path()),
    )
    .unwrap();
    assert_eq!(scan.as_value()["sources"][0]["state"], "unavailable");
    script(&op, "#!/bin/sh\nprintf 'item not found' >&2\nexit 1\n");
    assert!(
        !effects(tmp.path())
            .inspect("vault-item", &json!({"item_ref":"op://controlled/item"}))
            .unwrap()
            .present
    );
}
#[test]
fn failed_declared_probe_cannot_be_overruled_by_another_positive_probe() {
    struct Partial;
    impl SecretEffects for Partial {
        fn expand_root(&mut self, p: &str) -> ProbeResult<String> {
            Ok(p.into())
        }
        fn inspect(&mut self, k: &str, _: &Value) -> ProbeResult<SecretObservation> {
            if k == "vault-item" {
                Err("controlled denied access".into())
            } else {
                Ok(SecretObservation {
                    present: true,
                    ..Default::default()
                })
            }
        }
    }
    let tmp = tempfile::tempdir().unwrap();
    let c = controlled_catalog(
        json!({"env":{"names":["DECLARED_API_KEY"]},"vault-item":{"item_ref":"op://controlled/item"}}),
    );
    let scan = scan_secret_sources(&c, &mut Partial, &scan_options(tmp.path())).unwrap();
    assert_eq!(scan.as_value()["sources"][0]["state"], "unavailable");
    assert_eq!(scan.as_value()["coverage"], "partial");
}
