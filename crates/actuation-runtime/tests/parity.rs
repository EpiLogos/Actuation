mod support;
use serde_json::Value;
#[test]
fn every_frozen_realised_and_actualisation_case_has_native_parity() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/migration/oracle.json")).unwrap();
    let mut count = 0;
    for row in corpus["cases"].as_array().unwrap().iter().filter(|r| {
        let operation = r["operation"].as_str().unwrap();
        operation.starts_with("contracts/agency-actualisation.mjs#")
            || operation.starts_with("contracts/realised-actuation.mjs#")
    }) {
        count += 1;
        let result = support::evaluate(
            row["operation"].as_str().unwrap(),
            row["args"].as_array().unwrap(),
        );
        assert_eq!(
            result.is_ok(),
            row["expected"]["ok"].as_bool().unwrap(),
            "{}: {result:?}",
            row["id"]
        );
        if let Ok(actual) = result {
            assert_eq!(actual, row["expected"]["value"], "{}", row["id"]);
        }
    }
    assert!(count >= 30, "a nonempty substantive corpus must run");
}
