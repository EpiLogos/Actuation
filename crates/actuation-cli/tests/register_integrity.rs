//! Native port of ProjectCentral/tests/register.integrity.mjs (removed at the
//! R7 cutover). The checked-in project register's ownership floor and the live
//! NOW horizon are asserted here, so a register fault fails the same pull
//! request that caused it. Central authored-source ownership stays in the
//! register; this test only reads it.
use serde_json::Value;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn read_json(relative: &str) -> Value {
    let path = format!("{ROOT}/{relative}");
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{relative} is not JSON: {e}"))
}

#[test]
fn the_checked_in_projectcentral_register_has_its_full_ownership_floor() {
    let manifest = read_json("ProjectCentral/project.json");
    assert_eq!(
        manifest,
        serde_json::json!({
            "human_source": "ProjectCentral/user",
            "project_id": "Actuation",
            "schema": "central.project/v1",
            "wiki": {
                "profile": "okf-wiki/v1",
                "source": "ProjectCentral/agents/wiki/wiki.json",
            },
        })
    );

    let wiki = read_json(manifest["wiki"]["source"].as_str().unwrap());
    let project_space = wiki["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|object| {
            object["object"] == "space" && object["ref"] == "central:wiki:project:Actuation"
        })
        .expect("the Actuation WikiSpace must be present");
    assert_eq!(
        project_space["parent_space_refs"],
        serde_json::json!(["central:wiki:root"])
    );

    for required in [
        "ProjectCentral/agents/governance/repo-content.md",
        "ProjectCentral/agents/governance/repo-structure.md",
    ] {
        assert!(
            std::path::Path::new(&format!("{ROOT}/{required}")).exists(),
            "{required} must exist"
        );
    }

    let policy = read_json("ProjectCentral/now/policy.json");
    assert_eq!(
        policy["schema"],
        serde_json::json!("central.project-now.policy/v1")
    );
    assert_eq!(
        policy["carry_statuses"],
        serde_json::json!(["active", "waiting", "carried"])
    );
    assert_eq!(
        policy["remove_statuses"],
        serde_json::json!(["resolved", "expired", "promoted"])
    );
    assert_eq!(
        policy["human_scratch_cleanup"],
        serde_json::json!("human-owned-manual")
    );
    assert_eq!(
        policy["day_boundary"],
        serde_json::json!("caller-supplied-local-civil-date")
    );

    let promotions = read_json("ProjectCentral/now/promotions.json");
    assert_eq!(
        promotions,
        serde_json::json!({
            "schema": "central.project-now.promotions/v1",
            "entries": [],
        })
    );

    let completion = read_json(
        "ProjectCentral/now/agents/complete-actuation-projectcentral-now-register-2026-09-09.json",
    );
    assert_eq!(
        completion["schema"],
        serde_json::json!("central.project-now.handoff/v1")
    );
    assert_eq!(
        completion["provenance"],
        serde_json::json!("agent-authored-bounded-return")
    );
    assert_eq!(completion["status"], serde_json::json!("resolved"));
    assert!(!completion["actor"].is_null());
    assert!(completion["source_refs"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r == "ProjectCentral/project.json"));
    assert!(completion["evidence_refs"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r == "https://github.com/EpiLogos/Actuation/issues/1"));
    assert!(completion["preserve_refs"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r == "https://github.com/EpiLogos/Actuation/pull/41"));
    assert!(
        completion["result"]
            .as_str()
            .unwrap()
            .contains("Structural initialization only"),
        "completion result must state its standing"
    );
}
