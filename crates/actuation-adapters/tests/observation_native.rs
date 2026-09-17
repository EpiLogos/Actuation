mod common;
mod support;
use actuation_adapters::{effects::*, *};
use common::*;
use serde_json::json;
use std::{
    collections::BTreeMap,
    fs, thread,
    time::{Duration, Instant},
};
#[test]
fn declarative_catalog_is_extensible_without_generic_executable_changes() {
    let original = NativeCatalog::bundled().unwrap();
    assert_eq!(original.revision(), 9);
    assert_eq!(original.descriptors().len(), 12);
    assert_eq!(original.capabilities().len(), 4);
    assert_eq!(original.capability_gaps().len(), 8);
    for slug in ["claude-code", "codex", "pi", "ollama", "zcode"] {
        assert!(original.descriptor(slug).is_some());
    }
    assert_eq!(
        original.descriptor("ollama").unwrap().native_kind(),
        "model-provider"
    );
    assert!(
        original.capability("pi").is_some(),
        "the pi descriptor is authored from admission evidence, not guessed"
    );
    assert!(original.capability_gap("pi").is_none());
    assert!(
        original.capability("gemini").is_none(),
        "an unauthored capability is declared as a gap, not guessed"
    );
    assert!(original.capability_gap("gemini").is_some());
    for d in original.descriptors() {
        let slug = d.slug();
        assert!(
            original.capability(slug).is_some() || original.capability_gap(slug).is_some(),
            "every declared descriptor closes: {slug}"
        );
    }
    let mut v = catalog_value();
    let mut target = v["descriptors"][0].clone();
    target["slug"] = json!("controlled-new-body");
    target["probe"] = json!({"executable":{"names":["controlled"]}});
    v["descriptors"]
        .as_array_mut()
        .unwrap()
        .push(target.clone());
    // Extensibility holds under the closure law: a new descriptor declares a
    // gap until a capability descriptor is authored for it.
    v["capability_gaps"].as_array_mut().unwrap().push(json!({
        "schema": "actuation.harness-capability-gap/v1",
        "document": "capability-gap",
        "harness_slug": "controlled-new-body",
        "reason": "acceptance fixture: freshly declared target, capability not authored",
        "evidence_refs": ["fixture:declarative-extensibility"],
        "provenance": {
            "authored_by": "acceptance fixture",
            "source_refs": ["fixture:declarative-extensibility"]
        }
    }));
    let extended = NativeCatalog::from_json(&v.to_string()).unwrap();
    assert!(extended.descriptor("controlled-new-body").is_some());
    assert!(extended.capability_gap("controlled-new-body").is_some());
    assert!(extended.select(&["unknown".into()]).is_err());
    v["descriptors"].as_array_mut().unwrap().push(target);
    assert!(NativeCatalog::from_json(&v.to_string()).is_err());
}
#[test]
fn capability_closure_is_structural_not_conventional() {
    let gap_fixture = |slug: &str| {
        json!({
            "schema": "actuation.harness-capability-gap/v1",
            "document": "capability-gap",
            "harness_slug": slug,
            "reason": "acceptance fixture: capability withdrawn without replacement",
            "evidence_refs": ["fixture:capability-closure-acceptance"],
            "provenance": {
                "authored_by": "acceptance fixture",
                "source_refs": ["fixture:capability-closure-acceptance"]
            }
        })
    };
    // (a) Removing a descriptor's capability without a gap declaration fails
    // the load, naming the slug.
    let mut v = catalog_value();
    v["capabilities"]
        .as_array_mut()
        .unwrap()
        .retain(|c| c["harness_slug"] != json!("zcode"));
    let err = NativeCatalog::from_json(&v.to_string())
        .expect_err("an unclosed catalog must fail to load")
        .to_string();
    assert!(
        err.contains("zcode") && err.contains("neither"),
        "load error must name the slug: {err}"
    );

    // (b) A well-formed gap closes the catalog: it loads, the slug is
    // answered by a gap rather than a capability, and coverage is exact.
    v["capability_gaps"]
        .as_array_mut()
        .unwrap()
        .push(gap_fixture("zcode"));
    let closed = NativeCatalog::from_json(&v.to_string()).expect("a gapped catalog must load");
    assert!(closed.capability("zcode").is_none());
    let gap = closed.capability_gap("zcode").expect("gap declared");
    assert_eq!(
        gap.as_value()["reason"],
        json!("acceptance fixture: capability withdrawn without replacement")
    );
    assert_eq!(
        closed.capabilities().len() + closed.capability_gaps().len(),
        closed.descriptors().len()
    );

    // (c) Malformed gap declarations fail the load: undeclared target,
    // shadowed capability, missing evidence.
    let mut undeclared = catalog_value();
    undeclared["capability_gaps"]
        .as_array_mut()
        .unwrap()
        .push(gap_fixture("never-declared"));
    assert!(NativeCatalog::from_json(&undeclared.to_string()).is_err());

    let mut shadowed = catalog_value();
    shadowed["capability_gaps"]
        .as_array_mut()
        .unwrap()
        .push(gap_fixture("claude-code"));
    assert!(NativeCatalog::from_json(&shadowed.to_string()).is_err());

    let mut unevidenced = catalog_value();
    let mut bare = gap_fixture("codex");
    bare["evidence_refs"] = json!([]);
    unevidenced["capability_gaps"]
        .as_array_mut()
        .unwrap()
        .push(bare);
    assert!(NativeCatalog::from_json(&unevidenced.to_string()).is_err());

    // (d) A catalog without the gaps array cannot hide an open descriptor.
    let mut legacy = catalog_value();
    legacy["capability_gaps"].as_array_mut().unwrap().clear();
    let err = NativeCatalog::from_json(&legacy.to_string())
        .expect_err("a catalog with open descriptors must fail")
        .to_string();
    assert!(
        err.contains("neither"),
        "closure error names the failure: {err}"
    );
}
#[test]
fn mcp_config_facet_names_declared_servers_and_redacts_secret_flag_values() {
    let catalog = NativeCatalog::bundled().unwrap();
    let mut d = catalog.descriptor("openclaw").unwrap().as_value().clone();
    d["probe"] = json!({"config-dir": {"path": "~/.openclaw"}});
    d["facets"] = json!({
        "mcp-config": {
            "path": "~/.openclaw/mcp.json",
            "inventory": {"kind": "mcp-servers", "source": "file", "collection": "mcpServers"}
        }
    });
    let mut effects = support::FixtureEffects::new(json!({
        "statProbe": {"value": {"exists": true, "isDir": true}},
        "readTextFile": {"value": {"ok": true, "text": "{\"mcpServers\":{\"bimba-mcp\":{\"command\":\"docker\",\"args\":[\"exec\",\"-i\",\"epi-bimba-mcp\",\"node\",\"dist/index.js\"]},\"linear-server\":{\"command\":\"npx\",\"args\":[\"-y\",\"mcp-remote\",\"https://mcp.linear.app/mcp\"]},\"secretive\":{\"command\":\"srv\",\"args\":[\"--password\",\"hunter2\",\"--token=abc123\",\"--port\",\"9\"]},\"remote\":{\"url\":\"https://mcp.example/mcp\"}}}"}}
    }));
    let read = run_detection(
        &[HarnessDescriptor::try_from(d).unwrap()],
        &mut effects,
        &options(&catalog),
    )
    .unwrap();
    let entry = &read.as_value()["harnesses"][0];
    assert_eq!(entry["state"], "detected");
    let facet = entry["facets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["kind"] == "mcp-config")
        .expect("mcp-config facet observed")
        .clone();
    let inventory = facet["inventory"].as_array().unwrap();
    assert_eq!(inventory.len(), 4);
    let bimba = inventory.iter().find(|i| i["id"] == "bimba-mcp").unwrap();
    assert_eq!(
        bimba["command"],
        "docker exec -i epi-bimba-mcp node dist/index.js"
    );
    let secretive = inventory.iter().find(|i| i["id"] == "secretive").unwrap();
    let command = secretive["command"].as_str().unwrap();
    assert!(
        command.contains("[redacted-flag] [redacted-value]"),
        "{command}"
    );
    assert!(command.contains("--token=[redacted]"), "{command}");
    assert!(command.contains("--port 9"), "{command}");
    assert!(
        !command.contains("hunter2")
            && !command.contains("abc123")
            && !command.contains("--password"),
        "{command}"
    );
    assert_eq!(facet["inventory_receipt"]["item_count"], 4);
    assert_eq!(facet["inventory_receipt"]["source"], "~/.openclaw/mcp.json");
    assert!(
        !effects.calls.iter().any(|c| c["effect"] == "envProbe"),
        "file inventory never reads environment values"
    );
}

#[test]
fn absence_and_existing_configuration_never_become_a_fake_executable_or_version_call() {
    let catalog = NativeCatalog::bundled().unwrap();
    let mut effects = support::FixtureEffects::new(
        json!({"statProbe":{"value":{"exists":true,"isDir":true}},"versionProbe":{"value":{"ok":true,"version":"must-not-be-called"}}}),
    );
    let mut o = options(&catalog);
    o.probe_versions = true;
    let read = run_detection(
        &catalog.select(&["claude-code".into()]).unwrap(),
        &mut effects,
        &o,
    )
    .unwrap();
    let entry = &read.as_value()["harnesses"][0];
    assert_eq!(entry["state"], "detected");
    assert_eq!(entry["receipts"]["executable"], "/home/oracle/.claude");
    assert_eq!(entry["receipts"]["executable_is"], "config-dir");
    assert!(entry.get("version").is_none());
    assert!(!effects.calls.iter().any(|c| c["effect"] == "versionProbe"));
    assert!(!read.as_value().to_string().contains("must-not-be-called"));
}
#[test]
fn a_successful_environment_marker_does_not_turn_failed_presence_probes_into_absence() {
    let catalog = NativeCatalog::bundled().unwrap();
    let mut d = catalog.descriptor("codex").unwrap().as_value().clone();
    d["probe"] = json!({"executable":{"names":["codex"]},"env":{"any_of":["CODEX_MARKER"]}});
    let mut effects = support::FixtureEffects::new(
        json!({"resolveExecutable":{"value":{"error":"controlled IO refusal"}},"envProbe":{"value":{"ok":true,"matched":{"CODEX_MARKER":true}}}}),
    );
    let read = run_detection(
        &[HarnessDescriptor::try_from(d).unwrap()],
        &mut effects,
        &options(&catalog),
    )
    .unwrap();
    assert_eq!(read.as_value()["harnesses"][0]["state"], "unavailable");
    assert_eq!(read.as_value()["availability"], "partial");
    assert_eq!(read.as_value()["absent"], json!([]));
}
#[test]
fn endpoint_presence_without_a_captureable_receipt_is_disclosed_not_fabricated() {
    let catalog = NativeCatalog::bundled().unwrap();
    let mut d = catalog.descriptor("ollama").unwrap().as_value().clone();
    d["probe"] = json!({"service":{"kind":"http","default_url":"http://127.0.0.1:1"}});
    d.as_object_mut().unwrap().remove("facets");
    let mut effects = support::FixtureEffects::new(
        json!({"serviceProbe":{"value":{"ok":true,"detail":"http 200 from http://127.0.0.1:1"}}}),
    );
    let read = run_detection(
        &[HarnessDescriptor::try_from(d).unwrap()],
        &mut effects,
        &options(&catalog),
    )
    .unwrap();
    let entry = &read.as_value()["harnesses"][0];
    assert_eq!(entry["state"], "unavailable");
    assert!(entry["receipts"].is_null());
    assert_eq!(read.as_value()["absent"], json!([]));
}
#[test]
fn native_file_observation_distinguishes_missing_home_file_and_configuration() {
    let tmp = tempfile::tempdir().unwrap();
    let mut e = effects(tmp.path());
    assert!(e
        .stat(tmp.path().join("absent").to_str().unwrap())
        .unwrap()
        .is_none());
    fs::create_dir_all(tmp.path().join(".claude/skills")).unwrap();
    fs::write(tmp.path().join(".claude/skills/one"), "a skill").unwrap();
    fs::write(tmp.path().join(".claude/skills/two"), "another").unwrap();
    let catalog = NativeCatalog::bundled().unwrap();
    let read = run_detection(
        &catalog
            .select(&[
                "claude-code".into(),
                "codex".into(),
                "pi".into(),
                "zcode".into(),
            ])
            .unwrap(),
        &mut e,
        &options(&catalog),
    )
    .unwrap();
    let rows = read.as_value()["harnesses"].as_array().unwrap();
    assert_eq!(rows[0]["state"], "detected");
    assert_eq!(rows[0]["facets"][0]["count"], 2);
    assert!(rows[0]["facets"][0]["inventory"].is_null());
    for row in &rows[1..] {
        assert_eq!(row["state"], "not-installed");
    }
    let mut no_home = NativeEffects::new(None, tmp.path().into(), vec![], BTreeMap::new());
    assert!(no_home.expand_home("~/.claude").is_err());
    for key in ["agent_ref", "agency_ref", "model_ref", "authority_ref"] {
        assert!(!read.as_value().to_string().contains(key));
    }
}
#[test]
fn optional_self_markers_are_names_and_ambiguity_never_picks_an_identity() {
    let tmp = tempfile::tempdir().unwrap();
    let catalog = NativeCatalog::bundled().unwrap();
    let mut first = catalog.descriptor("codex").unwrap().as_value().clone();
    let mut second = catalog.descriptor("pi").unwrap().as_value().clone();
    first["probe"] = json!({"executable":{"names":["codex"]},"env":{"any_of":["FIRST"]}});
    second["probe"] = json!({"executable":{"names":["pi"]},"env":{"any_of":["SECOND"]}});
    let env = BTreeMap::from([
        ("FIRST".into(), "private-marker-value-1".into()),
        ("SECOND".into(), "private-marker-value-2".into()),
    ]);
    let mut e = NativeEffects::new(
        Some(tmp.path().into()),
        tmp.path().into(),
        vec![],
        env.clone(),
    );
    let read = resolve_self(
        &[
            HarnessDescriptor::try_from(first).unwrap(),
            HarnessDescriptor::try_from(second).unwrap(),
        ],
        &mut e,
        Some(&env),
        &options(&catalog),
    )
    .unwrap();
    assert_eq!(read.as_value()["ambiguity"], true);
    assert!(read.as_value()["resolved"].is_null());
    assert!(!read.as_value().to_string().contains("private-marker-value"));
    assert_eq!(
        read.as_value()["detection"]["states"]["codex"],
        "not-installed"
    );
}
#[test]
fn native_http_inventory_observes_the_declared_service_and_preserves_provider_native_ids() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join(".ollama/models")).unwrap();
    let catalog = NativeCatalog::bundled().unwrap();
    let (url,peer)=http_peer(vec![(200,"{}".into()),(200,json!({"models":[{"model":"model:one","name":"alias","digest":"digest","size":42,"prompt":"NEVER-CAPTURE"},{"model":"model:one","name":"duplicate"},{"model":"model:two","name":"model:two","size":99}]}).to_string())]);
    let mut d = catalog.descriptor("ollama").unwrap().as_value().clone();
    d["probe"]["service"]["default_url"] = json!(url);
    let read = run_detection(
        &[HarnessDescriptor::try_from(d).unwrap()],
        &mut effects(tmp.path()),
        &options(&catalog),
    )
    .unwrap();
    let entry = &read.as_value()["harnesses"][0];
    assert_eq!(entry["native_kind"], "model-provider");
    let facet = &entry["facets"][0];
    assert_eq!(facet["inventory"].as_array().unwrap().len(), 2);
    assert_eq!(facet["inventory"][0]["also_known_as"], json!(["alias"]));
    assert_eq!(
        facet["inventory_receipt"]["source"],
        format!("{url}/api/tags")
    );
    assert_eq!(facet["inventory_receipt"]["item_count"], 2);
    assert!(!read.as_value().to_string().contains("NEVER-CAPTURE"));
    let requests = peer.join().unwrap();
    assert!(requests[0].starts_with("GET / HTTP"));
    assert!(requests[1].starts_with("GET /api/tags HTTP"));
    assert!(requests
        .iter()
        .all(|r| !r.to_lowercase().contains("authorization")));
}
#[test]
fn malformed_inventory_stays_unavailable_instead_of_an_empty_provider_offering() {
    for (status, body) in [(500, "secret-error-body"), (200, "not-json"), (200, "{}")] {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join(".ollama/models")).unwrap();
        let catalog = NativeCatalog::bundled().unwrap();
        let (url, peer) = http_peer(vec![(200, "{}".into()), (status, body.into())]);
        let mut d = catalog.descriptor("ollama").unwrap().as_value().clone();
        d["probe"]["service"]["default_url"] = json!(url);
        let read = run_detection(
            &[HarnessDescriptor::try_from(d).unwrap()],
            &mut effects(tmp.path()),
            &options(&catalog),
        )
        .unwrap();
        let facet = &read.as_value()["harnesses"][0]["facets"][0];
        assert!(facet.get("inventory").is_none());
        assert!(facet["inventory_unavailable_reason"].is_string());
        assert!(!read.as_value().to_string().contains("secret-error-body"));
        peer.join().unwrap();
    }
}
#[test]
fn absent_native_service_prevents_inventory_calls() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join(".ollama/models")).unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    let catalog = NativeCatalog::bundled().unwrap();
    let mut d = catalog.descriptor("ollama").unwrap().as_value().clone();
    d["probe"]["service"]["default_url"] = json!(url);
    let read = run_detection(
        &[HarnessDescriptor::try_from(d).unwrap()],
        &mut effects(tmp.path()),
        &options(&catalog),
    )
    .unwrap();
    let facet = &read.as_value()["harnesses"][0]["facets"][0];
    assert!(facet.get("inventory").is_none());
    assert!(facet["inventory_unavailable_reason"]
        .as_str()
        .unwrap()
        .contains("did not prove"));
}
#[test]
fn native_observation_endpoints_refuse_non_http_and_embedded_credentials() {
    let tmp = tempfile::tempdir().unwrap();
    let mut e = effects(tmp.path());
    for url in [
        "file:///etc/passwd",
        "http://user:secret@127.0.0.1/",
        "not-an-endpoint",
    ] {
        let error = e.http_json(url).unwrap_err();
        assert!(!error.contains("secret"));
    }
}
#[cfg(unix)]
#[test]
fn native_version_probing_is_explicit_and_plain_detection_cannot_execute_targets() {
    let tmp = tempfile::tempdir().unwrap();
    let marker = tmp.path().join("ran");
    script(
        &tmp.path().join("bin/claude"),
        &format!(
            "#!/bin/sh\nprintf ran > '{}'\nprintf 'controlled-v1\\n'\n",
            marker.display()
        ),
    );
    let catalog = NativeCatalog::bundled().unwrap();
    let selected = catalog.select(&["claude-code".into()]).unwrap();
    let read = run_detection(&selected, &mut effects(tmp.path()), &options(&catalog)).unwrap();
    assert_eq!(read.as_value()["harnesses"][0]["state"], "detected");
    assert!(!marker.exists());
    let mut o = options(&catalog);
    o.probe_versions = true;
    let read = run_detection(&selected, &mut effects(tmp.path()), &o).unwrap();
    assert!(marker.exists());
    assert_eq!(read.as_value()["harnesses"][0]["version"], "controlled-v1");
    assert!(read.as_value()["harnesses"][0]["receipts"]["sha256"].is_string());
}
#[cfg(unix)]
#[test]
fn native_process_time_and_output_budgets_kill_the_owned_probe_group() {
    let tmp = tempfile::tempdir().unwrap();
    let marker = tmp.path().join("descendant-wrote");
    let program = tmp.path().join("slow");
    script(
        &program,
        &format!(
            "#!/bin/sh\n(/bin/sleep 0.35; printf late > '{}') &\nwait\n",
            marker.display()
        ),
    );
    let mut e = effects(tmp.path()).with_process_timeout(Duration::from_millis(50));
    let start = Instant::now();
    assert!(e
        .version(program.to_str().unwrap(), &[])
        .unwrap_err()
        .contains("timed out"));
    assert!(start.elapsed() < Duration::from_secs(2));
    thread::sleep(Duration::from_millis(450));
    assert!(!marker.exists(), "owned descendant survived timeout");
    script(&program,"#!/bin/sh\nwhile :; do printf '012345678901234567890123456789012345678901234567890123456789'; done\n");
    let mut e = effects(tmp.path());
    assert!(e
        .version(program.to_str().unwrap(), &[])
        .unwrap_err()
        .contains("output budget"));
}
#[test]
fn catalogue_boundary_uses_capability_provenance_not_the_later_whole_catalog_revision() {
    use actuation_stream::{
        BoundaryCatalogue, BoundaryOccurrence, JsonlStreamStore, OpenStream, StreamStore,
    };
    let catalog = NativeCatalog::bundled().unwrap();
    let declared =
        &catalog.capability("claude-code").unwrap().as_value()["provenance"]["catalog_revision"];
    let b = catalog.boundary("claude-code", "PreToolUse").unwrap();
    assert_eq!(serde_json::to_value(b.catalog_revision).unwrap(), *declared);
    let tmp = tempfile::tempdir().unwrap();
    let store = JsonlStreamStore::new(tmp.path()).unwrap();
    let open:OpenStream=serde_json::from_value(json!({"stream_ref":"stream:adapter","actuation_ref":"actuation:adapter","agency_ref":"agency:adapter","agent_session_ref":"session:external","world_binding_ref":"world:one","started_at":"2026-09-10T12:00:00Z"})).unwrap();
    store.open(&open).unwrap();
    let occurrence:BoundaryOccurrence=serde_json::from_value(json!({"stream_ref":"stream:adapter","harness":"claude-code","native_event":"PreToolUse","event_ref":"event:one","observed_at":"2026-09-10T12:00:01Z"})).unwrap();
    let recorded = store.record_boundary(occurrence, &catalog).unwrap();
    let wire = serde_json::to_value(recorded).unwrap();
    assert!(wire.to_string().contains("PreToolUse"));
    assert!(catalog.boundary("pi", "guessed").is_err());
}
