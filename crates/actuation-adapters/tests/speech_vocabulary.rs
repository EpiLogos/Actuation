//! Actuation #94 vocabulary sync guard.
//!
//! The admission grammar in `actuation-adapters::speech` mirrors AIKit's
//! `aikit.model-modality/v1` vocabulary as identically-spelled kebab strings.
//! [`MIRRORED_MODALITY_VOCABULARY`] freezes that mirror; these tests pin the
//! grammar to the fixture in both directions, so a rename upstream (ai-kit
//! `crates/aikit-core/src/model_modality.rs`) or a local edit that adds,
//! drops or respells a term breaks here loudly instead of drifting silently.

use actuation_adapters::{
    validate_speech_constitution, SpeechSupport, AVAILABILITY_CONDITIONS, CREDENTIAL_CONDITIONS,
    INTERACTIONS, MIRRORED_MODALITY_VOCABULARY, MIRRORED_SUPPORT_STATES, MODALITIES, RECONNECTS,
    SPEECH_CONSTITUTION_VERSION, TRANSFORMS, TRANSPORTS,
};
use serde_json::{json, Value};

fn base_constitution(overrides: Value) -> Value {
    let mut v = json!({
        "schema": SPEECH_CONSTITUTION_VERSION,
        "constitution_ref": "constitution:vocab",
        "agent_ref": "nara:canonical",
        "agency_ref": "agency:nara",
        "world_binding_ref": "binding:nara",
        "agent_session_ref": "session:nara-1",
        "body_ref": "model-surface:vocab",
        "model_relation": {"schema": "actuation.instantiation/v1", "model_ref": "model:opaque",
            "inference_surface": {"contract_ref": "contract:opaque"}},
        "access_profile": {"schema": "actuation.instantiation/v1",
            "inference": {"allowed": ["invoke"]}, "control": {"allowed": []}, "interior": {"depth": "opaque"}},
        "input_modalities": ["text", "speech"],
        "output_modalities": ["text", "speech"],
        "interaction": {"request-response": {"state": "supported"}},
        "transport": "http",
        "connection": {"kind": "stateless", "reconnect": null},
        "interruption": {"state": "supported"},
        "provider_binding": {"provider_ref": "provider:vocab"},
        "provenance": {"source_refs": ["aikit:model-runtime:fixture@1"]},
        "resolved_at": "2026-09-18T09:00:00Z"
    });
    let base = v.as_object_mut().expect("fixture base is an object");
    for (k, override_value) in overrides.as_object().expect("object overrides") {
        base.insert(k.clone(), override_value.clone());
    }
    v
}

fn admit(constitution: Value) -> bool {
    validate_speech_constitution(&constitution).is_ok()
}

#[test]
fn the_frozen_vocabulary_is_exactly_the_union_of_the_admission_grammars() {
    let mut grammar: Vec<&str> = Vec::new();
    grammar.extend_from_slice(MODALITIES);
    grammar.extend_from_slice(TRANSFORMS);
    grammar.extend_from_slice(INTERACTIONS);
    grammar.extend_from_slice(TRANSPORTS);
    grammar.extend_from_slice(RECONNECTS);
    grammar.extend_from_slice(AVAILABILITY_CONDITIONS);
    grammar.extend_from_slice(CREDENTIAL_CONDITIONS);
    grammar.extend_from_slice(&[
        // The connection semantics kinds are carried by the same fixture.
        "stateless",
        "connected",
    ]);
    grammar.sort_unstable();
    grammar.dedup();
    let mut frozen = MIRRORED_MODALITY_VOCABULARY.to_vec();
    frozen.sort_unstable();
    assert_eq!(
        grammar, frozen,
        "the admission grammar and the frozen AIKit mirror have diverged"
    );
    // No duplicates hide inside any single category: sorting collapses a
    // list that carries the same term twice.
    for (name, list) in [
        ("MODALITIES", MODALITIES),
        ("TRANSFORMS", TRANSFORMS),
        ("INTERACTIONS", INTERACTIONS),
        ("TRANSPORTS", TRANSPORTS),
        ("RECONNECTS", RECONNECTS),
        ("MIRRORED_MODALITY_VOCABULARY", MIRRORED_MODALITY_VOCABULARY),
    ] {
        let mut sorted = list.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), list.len(), "{name} carries a duplicate term");
    }
}

#[test]
fn admission_accepts_every_mirrored_term_in_its_own_category() {
    // Modalities: each term is admissible as an input and an output modality.
    for &m in MODALITIES {
        assert!(
            admit(base_constitution(json!({
                "input_modalities": [m], "output_modalities": [m],
                "transforms": null, "transform_role": null,
            }))),
            "modality {m} must be admitted"
        );
    }
    // Transforms: each term is admissible on an acoustic body.
    for &t in TRANSFORMS {
        assert!(
            admit(base_constitution(json!({
                "transforms": {t: {"state": "supported"}},
                "transform_role": t,
            }))),
            "transform {t} must be admitted"
        );
    }
    // Interactions: each term is admissible as a declared capability.
    for &i in INTERACTIONS {
        assert!(
            admit(base_constitution(json!({
                "interaction": {i: {"state": "supported"}},
            }))),
            "interaction {i} must be admitted"
        );
    }
    // Transports: each term is admissible with stateless semantics.
    for &t in TRANSPORTS {
        assert!(
            admit(base_constitution(json!({"transport": t}))),
            "transport {t} must be admitted"
        );
    }
    // Reconnects: each term is admissible on a connected transport.
    for &r in RECONNECTS {
        assert!(
            admit(base_constitution(json!({
                "connection": {"kind": "connected", "reconnect": r},
            }))),
            "reconnect support {r} must be admitted"
        );
    }
    // Connection kinds: both terms are admissible.
    assert!(admit(base_constitution(json!({
        "connection": {"kind": "stateless", "reconnect": null},
    }))));
    assert!(admit(base_constitution(json!({
        "connection": {"kind": "connected", "reconnect": "resumable"},
    }))));
    // Availability conditions: each recorded state is admissible with its
    // named reason ("available" is the absence of a condition, so it is not
    // a recorded fact and is not part of the grammar).
    for &a in AVAILABILITY_CONDITIONS {
        assert!(
            admit(base_constitution(json!({
                "conditions": [{"condition": a, "reason": "the contract requires a named reason"}],
            }))),
            "availability condition {a} must be admitted"
        );
    }
    // Credential conditions: each state with exactly its own fields.
    for (c, extra) in [
        ("not-required", json!({})),
        ("required", json!({"hint": "bind ZAI_API_KEY"})),
        (
            "satisfied",
            json!({"hint": "bind ZAI_API_KEY", "binding_ref": "secret-ref:zai"}),
        ),
    ] {
        assert!(
            CREDENTIAL_CONDITIONS.contains(&c),
            "credential condition {c} is missing from the grammar"
        );
        let mut condition = json!({"condition": c});
        for (k, v) in extra.as_object().expect("object extra") {
            condition[k.clone()] = v.clone();
        }
        assert!(
            admit(base_constitution(
                json!({"credential_condition": condition})
            )),
            "credential condition {c} must be admitted"
        );
    }
}

#[test]
fn admission_refuses_an_invented_term_in_every_category() {
    let invented = [
        json!({"input_modalities": ["braille"], "output_modalities": ["text"]}),
        json!({"input_modalities": ["text"], "output_modalities": ["smell"]}),
        json!({"transforms": {"smell-to-text": {"state": "supported"}}}),
        json!({"transform_role": "smell-to-text"}),
        json!({"interaction": {"smell-requests": {"state": "supported"}}}),
        json!({"transport": "carrier-pigeon"}),
        json!({"connection": {"kind": "smoky", "reconnect": null}}),
        json!({"connection": {"kind": "connected", "reconnect": "auto-magic"}}),
        json!({"conditions": [{"condition": "gone", "reason": "invented"}]}),
        json!({"credential_condition": {"condition": "smoky"}}),
        // A half-stated credential fact is refused, not guessed at.
        json!({"credential_condition": {"condition": "required"}}),
        json!({"credential_condition": {"condition": "satisfied", "hint": "h"}}),
        json!({"credential_condition": {"condition": "not-required", "hint": "h"}}),
    ];
    for case in invented {
        let constitution = base_constitution(case.clone());
        assert!(
            !admit(constitution),
            "an invented vocabulary term must be refused: {case}"
        );
    }
}

#[test]
fn the_support_states_are_exactly_the_four_mirrored_answers() {
    for state in MIRRORED_SUPPORT_STATES {
        let value = if *state == "supported" {
            json!({"state": state})
        } else {
            json!({"state": state, "reason": "the contract requires a named reason"})
        };
        let support = SpeechSupport::from_value(&value)
            .unwrap_or_else(|e| panic!("support state {state} must be admitted: {e}"));
        assert_eq!(support.as_value()["state"], *state);
    }
    // An invented state is refused; "proven" (the internal name for it) is
    // not part of the wire vocabulary.
    assert!(SpeechSupport::from_value(&json!({"state": "proven"})).is_err());
    assert!(SpeechSupport::from_value(&json!({"state": "supported-if-needed"})).is_err());
    // And the four admitted states are exactly the fixture, in both
    // directions.
    for state in ["supported", "degraded", "unsupported", "unknown"] {
        assert!(
            MIRRORED_SUPPORT_STATES.contains(&state),
            "support state {state} is missing from the frozen fixture"
        );
    }
    assert_eq!(MIRRORED_SUPPORT_STATES.len(), 4);
}
