//! Actuation #94 conformance: speech/audio material constitution for
//! model-bearing Agency. Fixture-based only — no network, no keys, no audio
//! hardware.

use actuation_adapters::*;
use actuation_core::{AgencyRef, AgentRef, ExternalRef, Result};
use serde_json::{json, Value};

const V: &str = "actuation.speech-constitution/v1";

fn base_constitution(body: &str, overrides: Value) -> Value {
    let mut v = json!({
        "schema": V,
        "constitution_ref": format!("constitution:{body}"),
        "agent_ref": "nara:canonical",
        "agency_ref": "agency:nara",
        "world_binding_ref": "binding:nara",
        "agent_session_ref": "session:nara-1",
        "body_ref": format!("model-surface:{body}"),
        "body_revision": "rev-1",
        "harness_composition_ref": "composition:nara-body",
        "model_relation": {"schema": INSTANTIATION_VERSION, "model_ref": "model:opaque",
            "engine": {"implementation_ref": "engine:opaque", "provider_ref": "provider:opaque"},
            "material": {"binding_ref": "workcell:binding-1", "placement": "remote"},
            "inference_surface": {"contract_ref": "contract:opaque"}},
        "access_profile": {"schema": INSTANTIATION_VERSION,
            "inference": {"allowed": ["invoke", "stream", "tool-use"]},
            "control": {"allowed": []},
            "interior": {"depth": "opaque"}},
        "modality_contract_ref": "aikit:model-modality:fixture-surface-1",
        "modality_contract_revision": "rev-1",
        "input_modalities": ["text", "speech"],
        "output_modalities": ["text", "speech"],
        "transforms": {"speech-to-speech": {"state": "supported"}},
        "transform_role": "speech-to-speech",
        "interaction": {
            "request-response": {"state": "supported"},
            "streaming-input": {"state": "supported"},
            "streaming-output": {"state": "supported"},
            "full-duplex-realtime": {"state": "supported"},
            "structured-events": {"state": "supported"},
            "tool-requests": {"state": "supported"},
            "partial-transcripts": {"state": "supported"},
            "final-transcripts": {"state": "supported"},
            "vad-turn-detection": {"state": "supported"},
            "barge-in": {"state": "supported"}
        },
        "transport": "websocket",
        "connection": {"kind": "connected", "reconnect": "resumable"},
        "interruption": {"state": "supported"},
        "provider_binding": {"provider_ref": "provider:realtime",
            "provider_session_ref": "provider-session:abc",
            "transport_connection_ref": "connection:ws-1"},
        "provenance": {"source_refs": ["aikit:model-runtime:fixture@1"]},
        "resolved_at": "2026-09-17T09:00:00Z"
    });
    merge(&mut v, overrides);
    v
}

fn merge(base: &mut Value, overrides: Value) {
    match (base.as_object_mut(), overrides.as_object()) {
        (Some(base), Some(overrides)) => {
            for (k, v) in overrides {
                base.insert(k.clone(), v.clone());
            }
        }
        _ => unreachable!("fixture merge only merges objects"),
    }
}

fn demand_1_text_then_speech_with_explicit_change() -> Result<()> {
    // The same semantic Agent: text-only body first, speech body later, one
    // explicit constitution change between them.
    let text_only = SpeechConstitution::try_from(base_constitution(
        "text",
        json!({
            "input_modalities": ["text"],
            "output_modalities": ["text"],
            "transforms": null,
            "transform_role": null,
            "interaction": {
                "request-response": {"state": "supported"},
                "tool-requests": {"state": "supported"},
                "barge-in": {"state": "unsupported", "reason": "no speech output exists to interrupt"}
            },
            "transport": "http",
            "connection": {"kind": "stateless", "reconnect": null},
            "interruption": {"state": "unsupported", "reason": "no speech output exists to interrupt"},
            "provider_binding": {"provider_ref": "provider:text"}
        }),
    ))?;
    let speech = SpeechConstitution::try_from(base_constitution("realtime", json!({})))?;
    assert!(!text_only.speech_capable());
    assert!(speech.speech_capable());
    let change = SpeechConstitutionChange::record(
        "change:text-to-speech",
        text_only.clone(),
        speech.clone(),
        "a speech-capable body was resolved for the same Agent",
        vec![ExternalRef::new("evidence:aikit-resolution")?],
        "2026-09-17T09:01:00Z",
    )?;
    // One Agent on both sides, by admission.
    assert_eq!(change.identity().0, AgentRef::new("nara:canonical")?);
    assert_eq!(change.identity().1, AgencyRef::new("agency:nara")?);
    let d = change.delta();
    assert!(!d.speech_capable_before && d.speech_capable_after);
    assert!(d.body_changed);
    // A change that reminted identity is refused outright.
    let mut other_agent = base_constitution("realtime", json!({"agent_ref": "nara:other"}));
    other_agent["constitution_ref"] = json!("constitution:other");
    let other = SpeechConstitution::try_from(other_agent)?;
    assert!(SpeechConstitutionChange::record(
        "change:refused",
        text_only,
        other,
        "not allowed",
        vec![ExternalRef::new("evidence:x")?],
        "2026-09-17T09:02:00Z"
    )
    .is_err());
    Ok(())
}

#[test]
fn demand_1_same_agent_text_only_then_speech_with_explicit_change() {
    demand_1_text_then_speech_with_explicit_change().unwrap();
}

#[test]
fn demand_2_cascade_and_native_s2s_produce_distinct_attributable_receipts() {
    let cascade = SpeechConstitution::try_from(base_constitution(
        "cascade",
        json!({
            "constitution_ref": "constitution:cascade",
            "model_relation": {"schema": INSTANTIATION_VERSION, "model_ref": "model:stt",
                "inference_surface": {"contract_ref": "contract:stt"}},
            "modality_contract_ref": "aikit:model-stage-runtime:fixture-cascade",
            "input_modalities": ["speech"],
            "output_modalities": ["speech"],
            "transforms": {"speech-to-text": {"state": "supported"}, "text-to-speech": {"state": "supported"}},
            "transform_role": null,
            "interaction": {
                "request-response": {"state": "supported"},
                "structured-events": {"state": "supported"},
                "final-transcripts": {"state": "supported"},
                "barge-in": {"state": "degraded", "reason": "barge-in only between cascade stages"},
                "tool-requests": {"state": "supported"}
            },
            "transport": "http",
            "connection": {"kind": "stateless", "reconnect": null},
            "interruption": {"state": "degraded", "reason": "barge-in only between cascade stages"},
            "provider_binding": {"provider_ref": "provider:cascade",
                "provider_session_ref": "provider-session:cascade"}
        }),
    ))
    .unwrap();
    let native = SpeechConstitution::try_from(base_constitution("realtime", json!({}))).unwrap();
    // Distinct, attributable receipts: different body, provider, transport
    // and capability truth, each carried by its own constitution ref.
    assert_ne!(cascade.constitution_ref(), native.constitution_ref());
    assert_ne!(cascade.body_ref().as_str(), native.body_ref().as_str());
    assert!(native.realtime_capable());
    assert!(!cascade.realtime_capable());
    assert_eq!(
        cascade.interruption_support(),
        SpeechSupport::Degraded {
            reason: "barge-in only between cascade stages".into()
        }
    );
    assert_eq!(native.interruption_support(), SpeechSupport::Supported);
    // A change receipt between them names exactly what moved.
    let change = SpeechConstitutionChange::record(
        "change:native-to-cascade",
        native,
        cascade.clone(),
        "realtime provider replaced by cascade composition",
        vec![ExternalRef::new("evidence:swap").unwrap()],
        "2026-09-17T09:03:00Z",
    )
    .unwrap();
    assert!(change
        .delta()
        .lost_interaction
        .contains(&"full-duplex-realtime".to_string()));
    // The cascade's stage transforms are visible in the after constitution.
    assert!(cascade.as_value()["transforms"]["speech-to-text"]["state"] == "supported");
}

#[test]
fn demand_3_reconnect_and_provider_replacement_do_not_remint_agent_identity() {
    let before = SpeechConstitution::try_from(base_constitution(
        "realtime",
        json!({"provider_binding": {"provider_ref": "provider:realtime",
              "provider_session_ref": "provider-session:abc", "transport_connection_ref": "connection:ws-1"}}),
    ))
    .unwrap();
    let after = SpeechConstitution::try_from(base_constitution(
        "realtime-2",
        json!({
            "constitution_ref": "constitution:realtime-2",
            "provider_binding": {"provider_ref": "provider:other",
                "provider_session_ref": "provider-session:xyz",
                "transport_connection_ref": "connection:ws-9"},
            "resolved_at": "2026-09-17T09:10:00Z"
        }),
    ))
    .unwrap();
    let change = SpeechConstitutionChange::record(
        "change:reconnect",
        before.clone(),
        after.clone(),
        "transport connection rebuilt on a replacement provider",
        vec![ExternalRef::new("evidence:reconnect").unwrap()],
        "2026-09-17T09:11:00Z",
    )
    .unwrap();
    // Enduring identity survives; material facts moved.
    assert_eq!(change.identity().0, before.agent_ref());
    assert_eq!(change.identity().1, before.agency_ref());
    let d = change.delta();
    assert!(d.provider_changed && d.provider_session_changed && d.transport_connection_changed);
    assert_eq!(after.agent_session_ref(), before.agent_session_ref());
}

#[test]
fn demand_4_unsupported_interruption_remains_explicit_and_four_state() {
    // Unsupported, degraded and unknown are three different facts and none
    // of them behaves as supported.
    for (state, extra) in [
        (
            "unsupported",
            json!({"reason": "provider cannot cancel mid-utterance"}),
        ),
        ("degraded", json!({"reason": "cancel only between turns"})),
        (
            "unknown",
            json!({"reason": "no modality contract declared interruption"}),
        ),
    ] {
        let c = SpeechConstitution::try_from(base_constitution(
            "limited",
            json!({
                "constitution_ref": "constitution:limited",
                "interaction": {"request-response": {"state": "supported"},
                    "barge-in": {"state": state, "reason": extra["reason"]}},
                "interruption": {"state": state, "reason": extra["reason"]}
            }),
        ))
        .unwrap();
        assert_eq!(c.interruption_support().as_value()["state"], state);
        assert!(!c.supports_interruption());
        // The reason survives admission: the degradation is named, never
        // flattened away.
        assert_eq!(
            c.interruption_support().as_value()["reason"],
            extra["reason"]
        );
    }
    // A missing reason is refused: an unnamed degradation is a lie.
    let mut unnamed = base_constitution("limited", json!({"interruption": {"state": "degraded"}}));
    assert!(SpeechConstitution::try_from(unnamed.clone()).is_err());
    // Absence of a capability from the interaction map is a declared fact.
    unnamed["interaction"] = json!({"request-response": {"state": "supported"}});
    unnamed["interruption"] = json!({"state": "supported"});
    let c = SpeechConstitution::try_from(unnamed).unwrap();
    assert!(matches!(
        c.interaction_support("barge-in"),
        SpeechSupport::Unknown { .. }
    ));
}

#[test]
fn demand_5_speech_model_tool_requests_can_be_refused_before_effect() {
    let constitution =
        SpeechConstitution::try_from(base_constitution("realtime", json!({}))).unwrap();
    let request = |constitution_ref: &str, action: Value| {
        SpeechToolRequest::try_from(json!({
            "schema": SPEECH_TOOL_DECISION_VERSION,
            "request_ref": "request:spoke-1",
            "constitution_ref": constitution_ref,
            "agent_session_ref": "session:nara-1",
            "proposed_action_ref": action,
            "payload_refs": ["ref:spoken-target"],
            "requested_at": "2026-09-17T09:20:00Z"
        }))
        .unwrap()
    };
    // 1. The body carries the channel, but available is not authorised:
    //    an action absent from the explicit allowed list is refused.
    let decision = adjudicate_speech_tool_request(
        "decision:1",
        &constitution,
        request("constitution:realtime", json!("action:not-listed")),
        &[ExternalRef::new("action:allowed-one").unwrap()],
        &[],
        "owner",
        "2026-09-17T09:20:01Z",
    )
    .unwrap();
    assert!(!decision.is_authorised());
    assert_eq!(decision.as_value()["resolution"]["stage"], "unauthorised");
    assert!(decision
        .record_execution(
            "execution:x",
            vec![ExternalRef::new("evidence:x").unwrap()],
            "2026-09-17T09:20:02Z"
        )
        .is_err());
    // 2. Explicitly denied is refused even though the channel works.
    let denied = adjudicate_speech_tool_request(
        "decision:2",
        &constitution,
        request("constitution:realtime", json!("action:denied-one")),
        &[ExternalRef::new("action:denied-one").unwrap()],
        &[ExternalRef::new("action:denied-one").unwrap()],
        "owner",
        "2026-09-17T09:20:03Z",
    )
    .unwrap();
    assert_eq!(denied.as_value()["resolution"]["stage"], "denied");
    // 3. No usable channel: refused regardless of the lists.
    let mut no_channel = base_constitution(
        "text",
        json!({"interaction": {"request-response": {"state": "supported"},
              "tool-requests": {"state": "unsupported", "reason": "the surface carries no tool channel"}}}),
    );
    no_channel["constitution_ref"] = json!("constitution:text");
    let no_channel = SpeechConstitution::try_from(no_channel).unwrap();
    let channel_refused = adjudicate_speech_tool_request(
        "decision:3",
        &no_channel,
        request("constitution:text", json!("action:allowed-one")),
        &[ExternalRef::new("action:allowed-one").unwrap()],
        &[],
        "owner",
        "2026-09-17T09:20:04Z",
    )
    .unwrap();
    assert_eq!(channel_refused.as_value()["resolution"]["stage"], "channel");
    // 4. Explicit authorisation exists — and execution is still a separate
    //    receipt that cites it. Authorised != executed.
    let allowed = adjudicate_speech_tool_request(
        "decision:4",
        &constitution,
        request("constitution:realtime", json!("action:allowed-one")),
        &[ExternalRef::new("action:allowed-one").unwrap()],
        &[],
        "owner",
        "2026-09-17T09:20:05Z",
    )
    .unwrap();
    assert!(allowed.is_authorised());
    let execution = allowed
        .record_execution(
            "execution:1",
            vec![ExternalRef::new("evidence:ran").unwrap()],
            "2026-09-17T09:20:06Z",
        )
        .unwrap();
    assert_eq!(execution.as_value()["action_ref"], "action:allowed-one");
    // The authorisation itself carries no execution.
    assert!(allowed.as_value()["execution"].is_null());
}

#[test]
fn demand_6_modality_facts_stay_material_conditions_not_consumer_semantics() {
    // The vocabulary carries no consumer ontology: no Nara, no dialogue, no
    // desktop. Only modality/transform/interaction/transport names.
    let consumer_words = ["nara", "dialogue", "expression", "epii", "desktop"];
    for word in INTERACTIONS
        .iter()
        .chain(MODALITIES)
        .chain(TRANSFORMS)
        .chain(TRANSPORTS)
        .chain(RECONNECTS)
    {
        for consumer in consumer_words {
            assert!(
                !word.contains(consumer),
                "generic vocabulary leaked consumer semantics: {word}"
            );
        }
    }
    // Provider spellings stay provenance refs, never the contract identity.
    let c = SpeechConstitution::try_from(base_constitution(
        "realtime",
        json!({"provider_binding": {"provider_ref": "provider:realtime",
              "provider_session_ref": "provider-session:abc",
              "facts": {"voice": "alloy", "native_model": "gpt-realtime"}}}),
    ))
    .unwrap();
    assert_eq!(
        c.provider_session_ref().unwrap().as_str(),
        "provider-session:abc"
    );
    // Secret-shaped keys are refused at admission.
    let mut leak = base_constitution("realtime", json!({}));
    leak["provider_binding"]["api_key"] = json!("must-not-travel");
    assert!(SpeechConstitution::try_from(leak).is_err());
}

#[test]
fn demand_7_provenance_explains_which_body_heard_and_spoke() {
    let c = SpeechConstitution::try_from(base_constitution("realtime", json!({}))).unwrap();
    let read = c.as_value();
    // Enough provenance to answer "which body heard/spoke at this turn":
    // the body, its revision, the provider session and connection that
    // actually served it, the AIKit resolution it came from, and when.
    assert_eq!(read["body_ref"], "model-surface:realtime");
    assert_eq!(read["body_revision"], "rev-1");
    assert_eq!(
        read["provider_binding"]["provider_session_ref"],
        "provider-session:abc"
    );
    assert_eq!(
        read["provider_binding"]["transport_connection_ref"],
        "connection:ws-1"
    );
    assert_eq!(
        read["modality_contract_ref"],
        "aikit:model-modality:fixture-surface-1"
    );
    assert_eq!(
        read["provenance"]["source_refs"][0],
        "aikit:model-runtime:fixture@1"
    );
    assert_eq!(read["resolved_at"], "2026-09-17T09:00:00Z");
    assert_eq!(
        aikit_resolution_ref("fixture", "1"),
        "aikit:model-runtime:fixture@1"
    );
}

#[test]
fn identity_roles_are_distinct_at_admission() {
    // The body is not the Agent, the Agency or the session; the provider
    // session is not the body.
    for conflict in [
        json!({"body_ref": "nara:canonical"}),
        json!({"agent_session_ref": "model-surface:realtime"}),
        json!({"provider_binding": {"provider_session_ref": "model-surface:realtime"}}),
    ] {
        assert!(
            SpeechConstitution::try_from(base_constitution("realtime", conflict)).is_err(),
            "identity collision must be refused"
        );
    }
}

#[test]
fn a_text_only_constitution_is_valid_and_unavailable_bodies_cannot_hide() {
    let text_only = SpeechConstitution::try_from(base_constitution(
        "text",
        json!({
            "input_modalities": ["text"],
            "output_modalities": ["text"],
            "transforms": null,
            "interaction": {"request-response": {"state": "supported"}},
            "transport": "http",
            "connection": {"kind": "stateless", "reconnect": null},
            "interruption": {"state": "unsupported", "reason": "no speech output exists to interrupt"},
            "provider_binding": {"provider_ref": "provider:text"}
        }),
    ))
    .unwrap();
    assert!(!text_only.speech_capable());
    // An unavailable body is admitted as a fact — but it names its condition.
    let down = SpeechConstitution::try_from(base_constitution(
        "down",
        json!({"conditions": [{"condition": "unavailable", "reason": "provider region outage"}]}),
    ))
    .unwrap();
    assert!(!down.body_usable());
}

#[test]
fn the_speech_body_names_its_gap_instead_of_passing_as_a_text_agent() {
    // A satisfied, fully capable body reads present.
    let present = SpeechConstitution::try_from(base_constitution(
        "realtime",
        json!({"credential_condition": {"condition": "satisfied", "hint": "bind ZAI_API_KEY",
              "binding_ref": "secret-ref:zai"}}),
    ))
    .unwrap();
    let body = present.speech_body();
    assert_eq!(body["state"], "present");
    assert_eq!(body["text_capable"], true);
    assert_eq!(body["speech_capable"], true);
    assert_eq!(body["realtime_capable"], true);
    assert_eq!(body["credential_condition"]["condition"], "satisfied");
    assert!(body["reason"].is_null(), "a present body names no gap");

    // none-supplied: a text-only body — text-capable, speech body absent.
    let text_only = SpeechConstitution::try_from(base_constitution(
        "text",
        json!({
            "input_modalities": ["text"],
            "output_modalities": ["text"],
            "transforms": null,
            "transform_role": null,
            "interaction": {"request-response": {"state": "supported"}},
            "transport": "http",
            "connection": {"kind": "stateless", "reconnect": null},
            "interruption": {"state": "unsupported", "reason": "no speech output exists to interrupt"},
            "provider_binding": {"provider_ref": "provider:text"}
        }),
    ))
    .unwrap();
    let body = text_only.speech_body();
    assert_eq!(body["state"], "absent");
    assert_eq!(body["reason"], "none-supplied");
    assert!(body["detail"].is_null());
    assert_eq!(body["text_capable"], true);
    assert_eq!(body["speech_capable"], false);

    // credential-gated: a speech body is declared but its credential is
    // required and unbound — the visible pre-key state.
    let gated = SpeechConstitution::try_from(base_constitution(
        "realtime",
        json!({"credential_condition": {"condition": "required", "hint": "bind ZAI_API_KEY"}}),
    ))
    .unwrap();
    let body = gated.speech_body();
    assert_eq!(body["state"], "absent");
    assert_eq!(body["reason"], "credential-gated");
    assert_eq!(body["detail"], "bind ZAI_API_KEY");
    // Capability and availability are two different facts; both stay honest
    // on the wire.
    assert_eq!(body["speech_capable"], true);

    // degraded: a recorded availability condition reduces the body.
    let degraded = SpeechConstitution::try_from(base_constitution(
        "realtime",
        json!({"conditions": [{"condition": "degraded", "reason": "elevation limits barge-in"}]}),
    ))
    .unwrap();
    let body = degraded.speech_body();
    assert_eq!(body["state"], "absent");
    assert_eq!(body["reason"], "degraded");
    assert_eq!(body["detail"], "elevation limits barge-in");

    // unavailable: a recorded condition forbids the body outright, and the
    // conditions stay on the wire unflattened.
    let down = SpeechConstitution::try_from(base_constitution(
        "down",
        json!({
            "constitution_ref": "constitution:down",
            "conditions": [{"condition": "unavailable", "reason": "provider region outage"}]
        }),
    ))
    .unwrap();
    let body = down.speech_body();
    assert_eq!(body["state"], "absent");
    assert_eq!(body["reason"], "unavailable");
    assert_eq!(body["detail"], "provider region outage");
    assert_eq!(body["conditions"][0]["condition"], "unavailable");

    // Priority: a body that declares no acoustic modality names none-supplied
    // even when a credential gate is also carried.
    let both = SpeechConstitution::try_from(base_constitution(
        "text",
        json!({
            "input_modalities": ["text"],
            "output_modalities": ["text"],
            "transforms": null,
            "transform_role": null,
            "interaction": {"request-response": {"state": "supported"}},
            "transport": "http",
            "connection": {"kind": "stateless", "reconnect": null},
            "interruption": {"state": "unsupported", "reason": "no speech output"},
            "provider_binding": {"provider_ref": "provider:text"},
            "credential_condition": {"condition": "required", "hint": "bind ZAI_API_KEY"}
        }),
    ))
    .unwrap();
    assert_eq!(both.speech_body()["reason"], "none-supplied");

    // The credential fact is condition/hint/refs only: the exact key
    // "credential" stays a refused value-shaped key.
    let mut leak = base_constitution("realtime", json!({}));
    leak["credential"] = json!("must-not-travel");
    assert!(SpeechConstitution::try_from(leak).is_err());
}

#[test]
fn instantiation_receipts_carry_a_correlated_speech_constitution() {
    let constitution =
        SpeechConstitution::try_from(base_constitution("realtime", json!({}))).unwrap();
    let receipt = json!({
        "schema": INSTANTIATION_VERSION,
        "actuation_ref": "actuation:nara-1",
        "agency_ref": "agency:nara",
        "world_binding_ref": "binding:nara",
        "agent_session_ref": "session:nara-1",
        "model_relation": {"schema": INSTANTIATION_VERSION, "model_ref": "model:opaque",
            "inference_surface": {"contract_ref": "contract:opaque"}},
        "access_profile": {"schema": INSTANTIATION_VERSION,
            "inference": {"allowed": ["invoke"]}, "control": {"allowed": []}, "interior": {"depth": "opaque"}}
    });
    let attached =
        InstantiationReceipt::read(attach_speech_constitution(&receipt, &constitution).unwrap())
            .unwrap();
    assert_eq!(
        attached.speech_constitution().unwrap().body_ref().as_str(),
        "model-surface:realtime"
    );
    // A foreign session's body cannot ride this receipt.
    let mut foreign = base_constitution("realtime", json!({}));
    foreign["agent_session_ref"] = json!("session:other");
    let foreign = SpeechConstitution::try_from(foreign).unwrap();
    assert!(attach_speech_constitution(&receipt, &foreign).is_err());
    // A receipt-level mismatch is refused at read admission too.
    let mut mismatched = receipt.clone();
    mismatched["speech_constitution"] = foreign.as_value().clone();
    assert!(InstantiationReceipt::read(mismatched).is_err());
}
