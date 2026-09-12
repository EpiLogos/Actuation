mod support;
use serde_json::{json, Value};
fn parity(prefixes: &[&str], expected: usize) {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/migration/oracle.json")).unwrap();
    let rows: Vec<_> = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| {
            prefixes
                .iter()
                .any(|p| r["operation"].as_str().unwrap().starts_with(p))
        })
        .collect();
    assert_eq!(rows.len(), expected, "missing frozen cases");
    let mut errors = vec![];
    for row in rows {
        let actual = support::pure_row(row);
        if actual["ok"] != row["expected"]["ok"]
            || (actual["ok"] == true && actual["value"] != row["expected"]["value"])
        {
            errors.push(json!({"id":row["id"],"operation":row["operation"],"expected":row["expected"],"actual":actual}));
        }
    }
    assert!(
        errors.is_empty(),
        "{}",
        serde_json::to_string_pretty(&errors).unwrap()
    );
}
#[test]
fn all_179_frozen_adapter_contract_cases_keep_wire_meaning() {
    parity(
        &[
            "contracts/harness-capability.mjs#",
            "contracts/harness-detection.mjs#",
            "contracts/instantiation.mjs#",
            "contracts/secret-detection.mjs#",
        ],
        179,
    );
}
#[test]
fn all_213_r4_cases_still_execute_with_native_usage_codecs_in_their_owner() {
    parity(
        &[
            "contracts/actuation-stream.mjs#",
            "contracts/activity.mjs#",
            "contracts/model-usage.mjs#",
            "contracts/request-correlation.mjs#",
        ],
        213,
    );
}
#[test]
fn frozen_catalog_probe_and_secret_effect_scenarios_match() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/migration/scenarios.json")).unwrap();
    let rows: Vec<_> = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| matches!(r["kind"].as_str(), Some("catalog" | "probe" | "secret")))
        .collect();
    assert_eq!(rows.len(), 13);
    for row in rows {
        let actual = support::scenario(row).unwrap_or_else(|e| panic!("{}: {e}", row["id"]));
        assert_eq!(
            support::stream::normalized(actual),
            support::stream::normalized(row["expected"].clone()),
            "{}",
            row["id"]
        );
    }
}
#[test]
fn r5_explicit_corrections_hold_natively_after_the_cutover() {
    // The R5 controlled regressions documented three detection bugs against
    // the served MJS sources; the executable law now lives here and is
    // asserted against the frozen scenarios verbatim.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../fixtures/migration/r5/corrections.json"
    ))
    .unwrap();
    assert_eq!(
        corpus["schema"],
        json!("actuation.r5-explicit-corrections/v1")
    );
    let rows: Vec<&Value> = corpus["cases"].as_array().unwrap().iter().collect();
    assert_eq!(rows.len(), 3);
    let answers: Vec<_> = rows
        .iter()
        .map(|row| {
            let value = support::scenario(row).unwrap_or_else(|e| panic!("{}: {e}", row["id"]));
            (row["id"].as_str().unwrap().to_owned(), value)
        })
        .collect();
    let answer = |id: &str| -> &Value {
        answers
            .iter()
            .find(|(name, _)| name == id)
            .map(|(_, value)| value)
            .expect("scenario answered")
    };

    // 1. Only an actual resolved executable is version-probed; configuration
    //    presence retains its explicit config-dir receipt. The absence detail
    //    must never become an executable receipt.
    let detection = answer("absence-detail-is-not-an-executable")["result"]["value"].clone();
    let harness = &detection["harnesses"][0];
    assert_eq!(harness["state"], json!("detected"));
    assert_eq!(harness["receipts"]["executable_is"], json!("config-dir"));
    assert_eq!(
        harness["receipts"]["executable"],
        json!("/home/oracle/.fixture")
    );
    assert!(
        !detection.to_string().contains("incorrectly-probed"),
        "an unresolved executable must not be version-probed"
    );

    // 2. A successful marker probe cannot make an incomplete presence probe
    //    prove absence: the harness is unavailable with that reason.
    let detection = answer("marker-success-is-not-presence-observation")["result"]["value"].clone();
    assert_eq!(detection["harnesses"][0]["state"], json!("unavailable"));
    assert_eq!(
        detection["harnesses"][0]["unavailable_reason"],
        json!(
            "presence observation incomplete; successful marker observation cannot prove absence"
        )
    );
    assert_eq!(detection["availability"], json!("partial"));

    // 3. Presence without a captureable same-run receipt is disclosed
    //    unavailable, never a fabricated probe or receipt.
    let detection = answer("service-without-a-receipt-is-degraded")["result"]["value"].clone();
    assert_eq!(detection["harnesses"][0]["state"], json!("unavailable"));
    assert_eq!(
        detection["harnesses"][0]["unavailable_reason"],
        json!("presence observed but no executable or configured-path receipt was captured")
    );
    assert!(detection["harnesses"][0]["receipts"].is_null());
}
