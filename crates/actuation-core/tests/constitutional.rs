mod support;
use actuation_core::*;
use serde_json::{json, Value};

fn corpus() -> Value {
    serde_json::from_str(include_str!("../../../fixtures/migration/oracle.json")).unwrap()
}
fn seed(operation: &str) -> Value {
    corpus()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| {
            row["operation"] == format!("contracts/agency.mjs#{operation}")
                && row["expected"]["ok"] == true
        })
        .unwrap()["args"][0]
        .clone()
}
fn binding() -> WorldBinding {
    WorldBinding::new(WorldBindingFields::new(
        WorldBindingRef::new("binding:ordinary").unwrap(),
        AgentRef::new("agent:independent").unwrap(),
        AgencyRef::new("agency:ordinary").unwrap(),
        WorldRef::new("directory:/work/ordinary").unwrap(),
        ScopeRef::new("scope:ordinary").unwrap(),
    ))
    .unwrap()
}

#[test]
fn frozen_constitutional_oracle_is_total_and_semantically_identical() {
    let corpus = corpus();
    let cases: Vec<_> = corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| {
            row["operation"]
                .as_str()
                .unwrap()
                .starts_with("contracts/agency.mjs#")
        })
        .collect();
    assert_eq!(
        cases.len(),
        119,
        "frozen constitutional coverage must not silently shrink"
    );
    for row in cases {
        let actual = support::evaluate(
            row["operation"].as_str().unwrap(),
            row["args"].as_array().unwrap(),
        );
        assert_eq!(
            actual.is_ok(),
            row["expected"]["ok"].as_bool().unwrap(),
            "{}: {actual:?}",
            row["id"]
        );
        if let Ok(value) = actual {
            assert_eq!(value, row["expected"]["value"], "{}", row["id"]);
        }
    }
}

#[test]
fn direct_constitutional_composition_requires_no_factory_or_body() {
    let binding = binding();
    let identity = binding.identity();
    let reading = AgenticComposition::new(binding).read();
    assert_eq!(reading.agent_ref, identity.agent);
    assert_eq!(reading.agency_ref, identity.agency);
    assert!(!reading.root_for_scope);
    assert!(!reading.metagency.available);
    let json = serde_json::to_value(reading).unwrap();
    for name in [
        "run_ref",
        "journey_ref",
        "agent_session_ref",
        "provider_ref",
        "caller_ref",
    ] {
        assert!(json.get(name).is_none(), "{name} was fabricated");
    }
}

#[test]
fn rootness_is_a_world_scope_relation_not_an_agent_species() {
    let binding = binding();
    let fields = binding.fields();
    let root = RootScope::new(RootScopeFields::new(
        fields.scope_ref.clone(),
        fields.world_ref.clone(),
    ))
    .unwrap();
    assert!(binding.is_root_for(&root));
    let other_world = RootScope::new(RootScopeFields::new(
        fields.scope_ref.clone(),
        WorldRef::new("world:other").unwrap(),
    ))
    .unwrap();
    assert!(!binding.is_root_for(&other_world));
    let other_scope = RootScope::new(RootScopeFields::new(
        ScopeRef::new("scope:other").unwrap(),
        fields.world_ref.clone(),
    ))
    .unwrap();
    assert!(!binding.is_root_for(&other_scope));
    assert_eq!(
        binding.identity(),
        AgenticComposition::new(binding.clone())
            .with_root_scope(root)
            .binding()
            .identity()
    );
}

#[test]
fn optional_absent_null_and_value_are_losslessly_distinct() {
    let base = serde_json::to_value(binding()).unwrap();
    for value in [None, Some(Value::Null), Some(json!("purpose:authored"))] {
        let mut wire = base.clone();
        if let Some(value) = value {
            wire["purpose_ref"] = value;
        }
        let admitted: WorldBinding = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(admitted).unwrap(), wire);
    }
}

#[test]
fn extensions_are_preserved_but_cannot_shadow_admitted_identity() {
    let mut fields = binding().into_fields();
    fields.extensions.insert(
        "owner_reading".into(),
        json!({"unknown": null, "difference": ["a", "b"]}),
    );
    let admitted = WorldBinding::new(fields).unwrap();
    let roundtrip: WorldBinding =
        serde_json::from_value(serde_json::to_value(&admitted).unwrap()).unwrap();
    assert_eq!(roundtrip, admitted);
    let mut malicious = admitted.into_fields();
    malicious
        .extensions
        .insert("agent_ref".into(), json!("agent:imposter"));
    assert!(WorldBinding::new(malicious).is_err());
}

#[test]
fn object_records_refuse_positional_arrays_and_scalar_shapes() {
    for invalid in [
        Value::Null,
        json!(false),
        json!([]),
        json!(["actuation.agency/v1", "scope:a", "world:a"]),
        json!(12),
    ] {
        assert!(serde_json::from_value::<RootScope>(invalid).is_err());
    }
    let mut wire = serde_json::to_value(binding()).unwrap();
    wire["constraints"] = json!([]);
    assert!(serde_json::from_value::<WorldBinding>(wire).is_err());
}

#[test]
fn reference_admission_preserves_published_whitespace_and_opaque_content() {
    for empty in ["", " \t\r\n", "\u{feff}", "\u{2000}\u{2029}"] {
        assert!(AgentRef::new(empty).is_err());
    }
    for opaque in ["  not:a:path  ", "agent:雪/../x", "\u{0085}", "0"] {
        assert_eq!(AgentRef::new(opaque).unwrap().as_str(), opaque);
    }
    assert!(serde_json::from_value::<AgentRef>(json!(42)).is_err());
}

#[test]
fn federation_never_silently_grants_authority() {
    let mut wire = seed("validateDetermination");
    wire["kind"] = json!("federation");
    for authorities in [json!([]), Value::Null] {
        wire["authority_refs"] = authorities;
        assert!(serde_json::from_value::<Determination>(wire.clone()).is_ok());
    }
    wire["authority_refs"] = json!(["authority:smuggled"]);
    assert!(serde_json::from_value::<Determination>(wire).is_err());
}

#[test]
fn four_determinations_share_grammar_without_collapsing_their_meaning() {
    let base = seed("validateDetermination");
    for (name, kind) in [
        (
            "self-differentiation",
            DeterminationKind::SelfDifferentiation,
        ),
        ("delegation", DeterminationKind::Delegation),
        ("derivation", DeterminationKind::Derivation),
        ("federation", DeterminationKind::Federation),
    ] {
        let mut wire = base.clone();
        wire["kind"] = json!(name);
        wire.as_object_mut().unwrap().remove("authority_refs");
        let admitted: Determination = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(admitted.fields().kind, kind);
        assert_eq!(serde_json::to_value(admitted).unwrap(), wire);
    }
}

#[test]
fn delegated_action_absence_is_unknown_and_explicit_denial_wins() {
    let autonomy: DelegatedAutonomy = serde_json::from_value(json!({
        "may_determine_within_bounds": true,
        "allowed_action_refs": ["action:read", "action:denied"],
        "denied_action_refs": ["action:denied"]
    }))
    .unwrap();
    assert_eq!(
        autonomy.action_standing(&ExternalRef::new("action:read").unwrap()),
        ActionStanding::Allowed
    );
    assert_eq!(
        autonomy.action_standing(&ExternalRef::new("action:denied").unwrap()),
        ActionStanding::Denied
    );
    assert_eq!(
        autonomy.action_standing(&ExternalRef::new("action:unknown").unwrap()),
        ActionStanding::Unspecified
    );
}

#[test]
fn return_path_or_explicit_autonomous_termination_is_required() {
    for mode in ["required", "optional"] {
        assert!(serde_json::from_value::<ReturnPolicy>(json!({"mode": mode})).is_err());
        assert!(serde_json::from_value::<ReturnPolicy>(
            json!({"mode": mode, "return_relation_ref": "return:relation"})
        )
        .is_ok());
    }
    assert!(
        serde_json::from_value::<ReturnPolicy>(json!({"mode": "autonomous-termination"})).is_ok()
    );
}

#[test]
fn return_progress_admits_exactly_the_five_constitutional_standings() {
    let base = seed("validateReturn");
    let mut accepted = 0;
    for received in [false, true] {
        for recognition in ["pending", "recognised", "rejected"] {
            for mutation in ["not-applied", "applied"] {
                let mut wire = base.clone();
                wire["received"] = json!(received);
                wire["recognition_state"] = json!(recognition);
                wire["world_mutation_state"] = json!(mutation);
                let valid = (received || recognition == "pending")
                    && (mutation != "applied" || recognition == "recognised");
                let result = serde_json::from_value::<Return>(wire.clone());
                assert_eq!(result.is_ok(), valid);
                if let Ok(returned) = result {
                    accepted += 1;
                    assert_eq!(serde_json::to_value(returned).unwrap(), wire);
                }
            }
        }
    }
    assert_eq!(accepted, 5);
}

#[test]
fn reception_preserves_raw_difference_and_does_not_recognise_it() {
    let mut wire = seed("validateReturn");
    wire["received"] = json!(false);
    wire["recognition_state"] = json!("pending");
    wire["world_mutation_state"] = json!("not-applied");
    wire["difference_refs"] = json!([
        "difference:dissent",
        "difference:failure",
        "difference:result"
    ]);
    let returned: Return = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(returned.standing(), ReturnStanding::Offered);
    let received = returned.receive();
    assert_eq!(received.standing(), ReturnStanding::Received);
    wire["received"] = json!(true);
    assert_eq!(serde_json::to_value(received).unwrap(), wire);
}

#[test]
fn return_provenance_never_invents_factory_or_caller_ancestry() {
    let wire = json!({"agency_lineage_refs": ["agency:direct"], "external_source_refs": null});
    let admitted: ReturnProvenance = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(serde_json::to_value(admitted).unwrap(), wire);
    assert!(
        serde_json::from_value::<ReturnProvenance>(json!({"agency_lineage_refs": []})).is_err()
    );
}

#[test]
fn metagency_operation_reading_is_not_an_implicit_scope_grant() {
    let grant: MetagencyGrant = serde_json::from_value(seed("validateMetagencyGrant")).unwrap();
    assert!(grant.includes(MetagencyOperation::DetermineAgency));
    assert!(!grant.includes(MetagencyOperation::ReintegrateReturn));
    let unrelated = AgenticComposition::new(binding())
        .with_grants(vec![grant])
        .read();
    assert!(!unrelated.metagency.available);
    assert!(unrelated.metagency.operations.is_empty());
}

#[test]
fn recursive_aggregate_retains_order_and_requires_real_parent_autonomy() {
    let mut parent = seed("validateDetermination");
    parent["determination_ref"] = json!("determination:parent");
    parent
        .as_object_mut()
        .unwrap()
        .remove("parent_determination_ref");
    parent["differentiated_agency_ref"] = json!("agency:middle");
    parent["delegated_autonomy"]["may_determine_within_bounds"] = json!(true);
    let mut child = parent.clone();
    child["determination_ref"] = json!("determination:child");
    child["parent_determination_ref"] = parent["determination_ref"].clone();
    child["determining_agency_ref"] = parent["differentiated_agency_ref"].clone();
    child["differentiated_agency_ref"] = json!("agency:leaf");
    let unordered = json!([child.clone(), parent.clone()]);
    let reading: DeterminationLineage = serde_json::from_value(unordered.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(reading).unwrap(),
        unordered,
        "aggregate reading is not actualisation admission"
    );
    assert!(serde_json::from_value::<DeterminationLineage>(json!([child.clone()])).is_err());
    parent["delegated_autonomy"]["may_determine_within_bounds"] = json!(false);
    assert!(serde_json::from_value::<DeterminationLineage>(json!([parent, child])).is_err());
}

#[test]
fn empty_lineage_and_unbounded_determination_cannot_be_admitted() {
    assert!(serde_json::from_value::<DeterminationLineage>(json!([])).is_err());
    let mut wire = seed("validateDetermination");
    wire["bounds_refs"] = json!([]);
    assert!(serde_json::from_value::<Determination>(wire).is_err());
}

#[test]
fn explicit_identity_comparison_is_independent_of_material_conditions() {
    let identity = binding().identity();
    assert!(identity.is_continuation_of(&identity.clone()));
    let changed = AgencyIdentity {
        agent: AgentRef::new("agent:explicitly-different").unwrap(),
        agency: identity.agency.clone(),
    };
    assert!(!identity.is_continuation_of(&changed));
}
