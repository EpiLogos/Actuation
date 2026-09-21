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

#[test]
fn the_actuation_now_horizon_represents_every_reconciled_live_owner_carrier() {
    let agents_dir = format!("{ROOT}/ProjectCentral/now/agents");
    // The horizon under test is the committed register. Live session returns
    // that exist only in a working tree belong to that day's open field and
    // move through a day close, so untracked files are out of scope here.
    let tracked = std::process::Command::new("git")
        .args(["ls-files", "ProjectCentral/now/agents"])
        .current_dir(ROOT)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter(|line| line.ends_with(".json"))
                .map(|line| format!("{ROOT}/{line}"))
                .collect::<Vec<_>>()
        });
    let paths: Vec<String> = match tracked {
        Some(paths) if !paths.is_empty() => paths,
        _ => std::fs::read_dir(&agents_dir)
            .expect("the NOW agents directory must be readable")
            .flatten()
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .map(|entry| entry.path().to_string_lossy().into_owned())
            .collect(),
    };
    let mut records: Vec<Value> = paths
        .iter()
        .map(|path| {
            let raw = std::fs::read_to_string(path).expect("record must be readable");
            serde_json::from_str(&raw).expect("record must be JSON")
        })
        .collect();
    let mut live: Vec<(String, String)> = records
        .iter()
        .filter(|record| {
            matches!(
                record["status"].as_str(),
                Some("active") | Some("waiting") | Some("carried")
            )
        })
        .map(|record| {
            (
                record["id"].as_str().unwrap().to_owned(),
                record["status"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    live.sort_by(|left, right| left.0.cmp(&right.0));
    records.retain(|record| {
        matches!(
            record["status"].as_str(),
            Some("active") | Some("waiting") | Some("carried")
        )
    });

    // Floor reconciled at the research-line merge (PR #102, 2026-09-21,
    // main merged at 4ba531d): the deep-conjugate-allowance and model-types
    // session returns for 2026-09-13..21 landed live between day closes, so
    // the horizon is reconciled to them here. Statuses move only through a
    // day close (projectcentral.now.rollover); the next Actuation day close
    // carries these records with their day lineage, and this list is
    // reconciled again at that boundary.
    assert_eq!(
        live,
        [
            (
                "actuation-wayfinder-programme-2026-09-09".to_owned(),
                "carried".to_owned()
            ),
            (
                "anuttara-syntax-computes-jev-round-2026-09-20".to_owned(),
                "active".to_owned()
            ),
            (
                "architecture-v2-corrected-frame-role-table-2026-09-21".to_owned(),
                "active".to_owned()
            ),
            (
                "bimba-embedded-1777-ml-nodes-after-2026-09-18".to_owned(),
                "active".to_owned()
            ),
            (
                "bimba-fixes-committed-keys-rotated-phase2-2026-09-18".to_owned(),
                "active".to_owned()
            ),
            (
                "branch-hygiene-report-remains-scheduler-owned-2026-09-09".to_owned(),
                "carried".to_owned()
            ),
            (
                "classic-repaired-and-verified-live-relational-2026-09-13".to_owned(),
                "active".to_owned()
            ),
            (
                "clean-state-landed-pr-82-merged-2026-09-14".to_owned(),
                "active".to_owned()
            ),
            (
                "clean-state-reached-r12-pr-82-2026-09-14".to_owned(),
                "active".to_owned()
            ),
            (
                "defects-fixed-and-full-glm-matched-2026-09-13".to_owned(),
                "active".to_owned()
            ),
            (
                "docker-situation-fixed-graph-served-from-2026-09-19".to_owned(),
                "active".to_owned()
            ),
            (
                "embedding-starvation-found-and-fixed-graph-2026-09-18".to_owned(),
                "active".to_owned()
            ),
            (
                "emt3-jevvak-chain-run-live-kernel-2026-09-18".to_owned(),
                "active".to_owned()
            ),
            (
                "emt4-fromvslanding-reconciliation-mef-lens-2026-09-18".to_owned(),
                "active".to_owned()
            ),
            (
                "epistemic-cultivation-research-ground-and-record-2026-09-09".to_owned(),
                "carried".to_owned()
            ),
            (
                "full-architecture-designed-map-verified-questions-posed-2026-09-20".to_owned(),
                "active".to_owned()
            ),
            (
                "full-campaign-dispatched-three-parallel-tracks-2026-09-15".to_owned(),
                "active".to_owned()
            ),
            (
                "full-rust-move-complete-pr-83-2026-09-14".to_owned(),
                "active".to_owned()
            ),
            (
                "handoff-queue-executed-conjugate-allowance-2026-09-17".to_owned(),
                "active".to_owned()
            ),
            (
                "jev-lens-tool-round11-zero-invocations-2026-09-20".to_owned(),
                "active".to_owned()
            ),
            (
                "m0-symbol-granularity-and-amplify-loop-live-2026-09-19".to_owned(),
                "active".to_owned()
            ),
            (
                "mjs-retirement-commissioned-and-halted-on-2026-09-13".to_owned(),
                "active".to_owned()
            ),
            (
                "modeltype-corrections-landed-coordinateled-ebm-2026-09-17".to_owned(),
                "active".to_owned()
            ),
            (
                "modeltype-experiment-class-set-up-in-2026-09-17".to_owned(),
                "active".to_owned()
            ),
            (
                "native-agent-harness-direction-and-research-merge-2026-09-21".to_owned(),
                "active".to_owned()
            ),
            (
                "native-corpus-final-classicqldirect-11-of-2026-09-17".to_owned(),
                "active".to_owned()
            ),
            (
                "night-face-to-spec-round12-invitation-answered-2026-09-21".to_owned(),
                "active".to_owned()
            ),
            (
                "owner-corrections-applied-sync-untouched-2026-09-19".to_owned(),
                "active".to_owned()
            ),
            (
                "pr-h-r8r9-harmonisation-and-prelocal-2026-09-12".to_owned(),
                "active".to_owned()
            ),
            (
                "prime-campaign-live-on-glm-p0p3-2026-09-13".to_owned(),
                "active".to_owned()
            ),
            (
                "prime-physical-relational-campaign-awaits-a-2026-09-09".to_owned(),
                "carried".to_owned()
            ),
            (
                "public-determination-and-agency-actualisation-operation-2026-09-09".to_owned(),
                "carried".to_owned()
            ),
            (
                "ql-form-validity-floor-and-concrescence-instrument-2026-09-19".to_owned(),
                "active".to_owned()
            ),
            (
                "r11-acceptance-evidence-native-ql-circuit-2026-09-17".to_owned(),
                "active".to_owned()
            ),
            (
                "r6-dispositions-recorded-and-branch-gates-2026-09-12".to_owned(),
                "carried".to_owned()
            ),
            (
                "r6-head-0dfa401-independently-verified-and-2026-09-11".to_owned(),
                "carried".to_owned()
            ),
            (
                "r6-merged-to-main-r7-unblocked-2026-09-12".to_owned(),
                "carried".to_owned()
            ),
            (
                "r6-native-research-crate-published-tested-2026-09-11".to_owned(),
                "carried".to_owned()
            ),
            (
                "r7-executed-native-cli-cutover-merged-2026-09-12".to_owned(),
                "carried".to_owned()
            ),
            (
                "reinspect-codex-stop-event-capability-evidence-2026-09-09".to_owned(),
                "carried".to_owned()
            ),
            (
                "relation-embeddings-m0-m1-live-tests-2026-09-21".to_owned(),
                "active".to_owned()
            ),
            (
                "round-3-stagedloop-corpus-complete-classic-2026-09-13".to_owned(),
                "active".to_owned()
            ),
            (
                "round-4-complete-first-live-native-2026-09-15".to_owned(),
                "active".to_owned()
            ),
            (
                "round-5-corpus-complete-correctness-invariant-2026-09-17".to_owned(),
                "active".to_owned()
            ),
            (
                "round-9-toolset-matrix-complete-implicit-2026-09-18".to_owned(),
                "active".to_owned()
            ),
            (
                "series-1-experiment-thread-activated-live-2026-09-13".to_owned(),
                "active".to_owned()
            ),
            (
                "session-handoff-harness-is-settled-the-2026-09-17".to_owned(),
                "active".to_owned()
            ),
            (
                "sibling-frontier-raised-and-head-beats-cosine-in-m0-jev-remeasured-2026-09-19"
                    .to_owned(),
                "active".to_owned()
            ),
            (
                "supply-actuation-intent-and-grant-integration-handoff-2026-09-09".to_owned(),
                "carried".to_owned()
            ),
            (
                "toolset-paradigm-landed-qltoolset-condition-2026-09-18".to_owned(),
                "active".to_owned()
            ),
            (
                "track-a-complete-qlmef-185-merged-2026-09-15".to_owned(),
                "active".to_owned()
            ),
            (
                "two-routes-round10-tagged-propagation-jev-refined-2026-09-19".to_owned(),
                "active".to_owned()
            ),
            (
                "vak-arm-ran-the-corpus-live-2026-09-18".to_owned(),
                "active".to_owned()
            ),
            (
                "wayfinder-58-closed-native-rust-product-2026-09-12".to_owned(),
                "active".to_owned()
            ),
            (
                "wayfinder-58-finishup-done-on-this-2026-09-12".to_owned(),
                "active".to_owned()
            ),
        ],
        "the live NOW horizon must be exactly the reconciled carrier set"
    );

    for record in &records {
        assert_eq!(
            record["schema"],
            serde_json::json!("central.project-now.handoff/v1")
        );
        assert_eq!(
            record["provenance"],
            serde_json::json!("agent-authored-bounded-return")
        );
        assert!(!record["actor"].is_null());
        assert!(!record["subject"].is_null());
        let result = record["result"].as_str().unwrap();
        assert!(
            result.contains("Next condition:")
                || result.contains("Waiting condition:")
                || result.contains("waiting on the next scheduled inspection"),
            "live records must name their continuation: {result}"
        );
        assert!(
            record["evidence_refs"]
                .as_array()
                .map(Vec::len)
                .unwrap_or(0)
                > 0,
            "live records must carry evidence"
        );
    }
}
