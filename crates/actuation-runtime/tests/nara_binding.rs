//! Actuation #91 conformance: canonical Nara bound to native speech-capable
//! Agency bodies with Epii delegation. Fixture-based only — no network, no
//! keys, no audio hardware.

use actuation_adapters::{
    SpeechConstitution, SpeechToolDecision, SpeechToolRequest, SPEECH_CONSTITUTION_VERSION,
    SPEECH_TOOL_DECISION_VERSION,
};
use actuation_core::{
    ActuationRef, AgencyRef, AgentSessionRef, EventRef, Extensions, ExternalRef, Slot, StreamRef,
};
use actuation_runtime::{
    shared_world_guard, NaraBinding, QL_EPII_ENRICHMENT, QL_NARA_DIALOGUE_CONTEXT,
};
use actuation_stream::{
    ActivityDescription, ActivityOutcome, ActivityPhase, Attribution, AttributionFields, Count,
    EventKind, OpenStream, Salience, StreamEvent, StreamEventFields, Timestamp,
};
use serde_json::{json, Value};

fn constitution(
    body: &str,
    agent_session: &str,
    speech: bool,
    interruption: Value,
) -> SpeechConstitution {
    let (input, output, interaction, transport, connection) = if speech {
        (
            json!(["text", "speech"]),
            json!(["text", "speech"]),
            json!({
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
            }),
            json!("websocket"),
            json!({"kind": "connected", "reconnect": "resumable"}),
        )
    } else {
        (
            json!(["text"]),
            json!(["text"]),
            json!({
                "request-response": {"state": "supported"},
                "tool-requests": {"state": "supported"},
                "barge-in": {"state": "unsupported", "reason": "no speech output exists to interrupt"}
            }),
            json!("http"),
            json!({"kind": "stateless", "reconnect": null}),
        )
    };
    SpeechConstitution::try_from(json!({
        "schema": SPEECH_CONSTITUTION_VERSION,
        "constitution_ref": format!("constitution:{body}"),
        "agent_ref": "nara:canonical",
        "agency_ref": "agency:nara",
        "world_binding_ref": "binding:shared-world",
        "agent_session_ref": agent_session,
        "body_ref": format!("model-surface:{body}"),
        "body_revision": "rev-1",
        "model_relation": {"schema": "actuation.instantiation/v1", "model_ref": format!("model:{body}"),
            "inference_surface": {"contract_ref": format!("contract:{body}")}},
        "access_profile": {"schema": "actuation.instantiation/v1",
            "inference": {"allowed": ["invoke", "stream"]}, "control": {"allowed": []}, "interior": {"depth": "opaque"}},
        "modality_contract_ref": format!("aikit:model-modality:{body}"),
        "input_modalities": input,
        "output_modalities": output,
        "interaction": interaction,
        "transport": transport,
        "connection": connection,
        "interruption": interruption,
        "provider_binding": {"provider_ref": format!("provider:{body}"),
            "provider_session_ref": format!("provider-session:{body}"),
            "transport_connection_ref": format!("connection:{body}")},
        "provenance": {"source_refs": ["aikit:model-runtime:fixture@1"]},
        "resolved_at": "2026-09-17T09:00:00Z"
    }))
    .expect("fixture constitution must be valid")
}

fn context(nara: &str, session: &str) -> Value {
    json!({
        "schema": QL_NARA_DIALOGUE_CONTEXT,
        "context_ref": "context:encounter-1",
        "nara_ref": nara,
        "subject_ref": "subject:frank",
        "agent_session_ref": session,
        "coordinate_ref": "M2-5-9:C0",
        "expression_ref": "expression:1",
        "expression_revision": "rev-7",
        "profile_ref": "profile:1",
        "profile_revision": "rev-2",
        "active_m_focus": "m4",
        "disclosed": [{"ref_id": "source:1", "revision": "rev-1", "standing": "verified",
            "disclosure": "khora-entry", "disclosed_via_ref": "disclosure:1"}],
        "available_action_refs": ["action:read-coordinate"],
        "expressive_act": {
            "expressive_act_ref": "expressive-act:9",
            "phase": "active",
            "basis_expression_revision": "rev-7",
            "speech_turn_ref": "response:1"
        }
    })
}

fn realtime_nara() -> NaraBinding {
    NaraBinding::constitute(
        constitution(
            "realtime",
            "session:nara-1",
            true,
            json!({"state": "supported"}),
        ),
        context("nara:canonical", "session:nara-1"),
        vec![ExternalRef::new("action:read-coordinate").unwrap()],
        vec![ExternalRef::new("action:delete-world").unwrap()],
    )
    .unwrap()
}

/// Demand 1 + 2: one Nara across two speech-body materialisations; native
/// realtime and composed cascade both inhabit the same semantic Nara.
#[test]
fn one_nara_across_realtime_and_cascade_materialisations() {
    let realtime = realtime_nara();
    assert!(realtime.session().constitution().realtime_capable());
    let mut cascade = realtime.clone();
    let change = cascade
        .reconnect(
            "change:rt-to-cascade",
            constitution(
                "cascade",
                "session:nara-1",
                true,
                json!({"state": "degraded", "reason": "barge-in only between cascade stages"}),
            ),
            context("nara:canonical", "session:nara-1"),
            "realtime provider lost; cascade fallback",
            vec![ExternalRef::new("evidence:outage").unwrap()],
            "2026-09-17T09:06:00Z",
        )
        .unwrap();
    // The same Agent identity on both sides of the change, by admission.
    assert_eq!(change.identity().0.as_str(), "nara:canonical");
    assert_eq!(cascade.nara_ref(), "nara:canonical");
    assert_eq!(cascade.agent_ref(), realtime.agent_ref());
    assert_eq!(
        cascade.session().agency_ref(),
        realtime.session().agency_ref()
    );
    assert_eq!(
        cascade.session().agent_session_ref(),
        realtime.session().agent_session_ref()
    );
    // Distinct material truth, retained for provenance.
    assert_ne!(
        cascade.session().constitution().body_ref(),
        realtime.session().constitution().body_ref()
    );
}

/// Demand 3: an updated Expression/deictic context does not remint the
/// session or the Nara.
#[test]
fn context_updates_do_not_remint_the_session() {
    let mut nara = realtime_nara();
    let before = (nara.agent_ref(), nara.session().agent_session_ref());
    let mut next = context("nara:canonical", "session:nara-1");
    next["expression_revision"] = json!("rev-8");
    next["pointed_ref"] = json!("source:1");
    nara.update_context(next).unwrap();
    assert_eq!(before.0, nara.agent_ref());
    assert_eq!(before.1, nara.session().agent_session_ref());
}

/// Demand 4: interruption yields attributable Activity — projected through a
/// real ActuationStream — and the correlated O:I ExpressiveAct hold/cancel.
#[test]
fn interruption_yields_attributable_activity_and_o_i_correlation() {
    let mut nara = realtime_nara();
    nara.session_mut()
        .begin_response(ExternalRef::new("response:1").unwrap())
        .unwrap();
    nara.session_mut()
        .commit_result(ExternalRef::new("result:executed-1").unwrap())
        .unwrap();
    let receipt = nara
        .interrupt(
            "interruption:1",
            "user spoke over the response",
            "2026-09-17T09:30:00Z",
        )
        .unwrap();
    assert!(receipt.cancelled());
    assert_eq!(
        receipt.as_value()["interruption_receipt"]["response_ref"],
        "response:1"
    );
    assert_eq!(
        receipt.as_value()["interruption_receipt"]["executed_refs"],
        json!(["result:executed-1"])
    );
    assert_eq!(receipt.expressive_act_ref(), Some("expressive-act:9"));
    assert_eq!(
        receipt.as_value()["expressive_act"]["disposition"],
        "hold-and-cancel-pending-choreography"
    );
    assert_eq!(receipt.as_value()["session_destroyed"], false);

    // The same interruption rides a real stream as an Interruption event and
    // projects to an attributable Activity: actor, phase Interrupted,
    // outcome Cancelled, the receipt as evidence.
    let stream = OpenStream {
        stream_ref: StreamRef::new("stream:nara-1").unwrap(),
        actuation_ref: ActuationRef::new("actuation:nara-1").unwrap(),
        agency_ref: AgencyRef::new("agency:nara").unwrap(),
        agent_session_ref: AgentSessionRef::new("session:nara-1").unwrap(),
        world_binding_ref: Some(
            actuation_core::WorldBindingRef::new("binding:shared-world").unwrap(),
        ),
        provenance: None,
        started_at: Some(Timestamp::new("2026-09-17T09:29:00Z").unwrap()),
    }
    .empty()
    .unwrap();
    let actor = || {
        Attribution::new(AttributionFields {
            locus_ref: Slot::Absent,
            agency_ref: Slot::Value(AgencyRef::new("agency:nara").unwrap()),
            agent_ref: Slot::Value(actuation_core::AgentRef::new("nara:canonical").unwrap()),
            participant_ref: Slot::Absent,
            extensions: Extensions::new(),
        })
        .unwrap()
    };
    let event = |sequence: u64, kind: EventKind, evidence: Vec<ExternalRef>| {
        StreamEvent::new(StreamEventFields {
            event_ref: EventRef::new(format!("event:{sequence}")).unwrap(),
            sequence: Count::new(sequence).unwrap(),
            kind,
            custom_kind: Slot::Absent,
            observed_at: Slot::Value(Timestamp::new("2026-09-17T09:30:00Z").unwrap()),
            actor: Slot::Value(actor()),
            surface_ref: Slot::Absent,
            execution_ref: Slot::Absent,
            return_ref: Slot::Absent,
            native_trace_ref: Slot::Absent,
            resource_refs: Slot::Absent,
            evidence_refs: Slot::Value(
                evidence
                    .iter()
                    .map(|e| e.as_str().to_owned())
                    .map(|s| ExternalRef::new(s).unwrap())
                    .collect(),
            ),
            disclosure: Slot::Absent,
            content: Slot::Absent,
            metadata: Slot::Absent,
            model_usage: Slot::Absent,
            extensions: Extensions::new(),
        })
        .unwrap()
    };
    let stream = stream
        .append(event(1, EventKind::ToolRequest, vec![]))
        .unwrap()
        .append(event(2, EventKind::ToolResult, vec![]))
        .unwrap()
        .append(event(
            3,
            EventKind::Interruption,
            vec![ExternalRef::new("interruption:1").unwrap()],
        ))
        .unwrap();
    let activity = actuation_stream::Activity::from_event(
        &stream,
        &EventRef::new("event:3").unwrap(),
        ActivityDescription {
            activity_ref: actuation_core::ActivityRef::new("activity:interruption-1").unwrap(),
            subject_ref: ExternalRef::new("response:1").unwrap(),
            native_owner: ExternalRef::new("nara:canonical").unwrap(),
            verb: ExternalRef::new("interrupted").unwrap(),
            object: ExternalRef::new("response:1").unwrap(),
            summary: ExternalRef::new("speech interruption cancelled the in-flight Nara response")
                .unwrap(),
            salience: Salience::Important,
            needs_attention: false,
            action_ref: Slot::Absent,
            invocation_ref: Slot::Absent,
            plan_ref: None,
            journey_ref: None,
            run_ref: None,
            result_ref: Slot::Value(ExternalRef::new("interruption:1").unwrap()),
            return_ref: Slot::Absent,
            evidence_refs: vec![ExternalRef::new("interruption:1").unwrap()],
            outcome: Slot::Value(ActivityOutcome::Cancelled),
            metadata: Slot::Absent,
        },
        ActivityPhase::Interrupted,
    )
    .unwrap();
    assert_eq!(activity.fields().phase, ActivityPhase::Interrupted);
    assert_eq!(
        activity
            .fields()
            .actor
            .fields()
            .agent_ref
            .value()
            .unwrap()
            .as_str(),
        "nara:canonical"
    );
    assert_eq!(activity.fields().object.as_str(), "response:1");
}

/// Demand 5: unsupported interruption is explicit — recorded refusal, the
/// response continues, nothing is faked, and where the AIKit truth says
/// degraded the session behaviour matches it.
#[test]
fn unsupported_and_degraded_interruptions_behave_like_their_truth() {
    // Unsupported: refusal, response continues.
    let mut nara = NaraBinding::constitute(
        constitution(
            "batch",
            "session:nara-1",
            true,
            json!({"state": "unsupported", "reason": "provider cannot cancel mid-utterance"}),
        ),
        context("nara:canonical", "session:nara-1"),
        vec![],
        vec![],
    )
    .unwrap();
    nara.session_mut()
        .begin_response(ExternalRef::new("response:2").unwrap())
        .unwrap();
    let receipt = nara
        .interrupt("interruption:2", "stop", "2026-09-17T09:31:00Z")
        .unwrap();
    assert!(!receipt.cancelled());
    assert!(receipt.as_value()["interruption_receipt"]["refusal_reason"]
        .as_str()
        .unwrap()
        .contains("does not support interruption"));
    assert!(nara.session().read()["in_flight_response_ref"] == "response:2");
    // Degraded: also refused, with the provider's own reason carried.
    let mut degraded = NaraBinding::constitute(
        constitution(
            "cascade",
            "session:nara-1",
            true,
            json!({"state": "degraded", "reason": "cancel only between cascade stages"}),
        ),
        context("nara:canonical", "session:nara-1"),
        vec![],
        vec![],
    )
    .unwrap();
    degraded
        .session_mut()
        .begin_response(ExternalRef::new("response:3").unwrap())
        .unwrap();
    let receipt = degraded
        .interrupt("interruption:3", "stop", "2026-09-17T09:32:00Z")
        .unwrap();
    assert!(!receipt.cancelled());
    assert!(receipt.as_value()["interruption_receipt"]["refusal_reason"]
        .as_str()
        .unwrap()
        .contains("cancel only between cascade stages"));
}

/// Demand 6: a speech tool request is refused before effect; an authorised
/// one still separates authorisation from execution.
#[test]
fn speech_tool_requests_are_gated_before_effect() {
    let mut nara = realtime_nara();
    let request = |action: Option<&str>| {
        SpeechToolRequest::try_from(json!({
            "schema": SPEECH_TOOL_DECISION_VERSION,
            "request_ref": "request:spoken-1",
            "constitution_ref": "constitution:realtime",
            "agent_session_ref": "session:nara-1",
            "proposed_action_ref": action,
            "payload_refs": ["ref:spoken-target"],
            "requested_at": "2026-09-17T09:33:00Z"
        }))
        .unwrap()
    };
    let unauthorised = nara
        .adjudicate_tool_request(
            "decision:1",
            request(Some("action:not-in-the-list")),
            "owner",
            "2026-09-17T09:33:01Z",
        )
        .unwrap();
    assert!(!unauthorised.is_authorised());
    let denied = nara
        .adjudicate_tool_request(
            "decision:2",
            request(Some("action:delete-world")),
            "owner",
            "2026-09-17T09:33:02Z",
        )
        .unwrap();
    assert!(!denied.is_authorised());
    let allowed = nara
        .adjudicate_tool_request(
            "decision:3",
            request(Some("action:read-coordinate")),
            "owner",
            "2026-09-17T09:33:03Z",
        )
        .unwrap();
    assert!(allowed.is_authorised());
    assert!(allowed.as_value()["execution"].is_null());
    assert!(unauthorised
        .record_execution(
            "execution:x",
            vec![ExternalRef::new("evidence:x").unwrap()],
            "2026-09-17T09:33:04Z"
        )
        .is_err());
    assert_eq!(nara.decisions().len(), 3);
    let _: Option<&SpeechToolDecision> = nara.decisions().first();
}

/// Demand 7 + 8: Nara delegates a bounded task to Epii and remains
/// foreground; a late result is retained but never auto-applied.
#[test]
fn delegation_stays_structured_and_late_results_never_apply() {
    let mut nara = realtime_nara();
    let delegation = json!({
        "schema": "ql.nara-epii-delegation/v1",
        "delegation_ref": "delegation:1",
        "nara_ref": "nara:canonical",
        "epii_session_ref": "session:epii-1",
        "basis": {
            "context_ref": "context:encounter-1",
            "expression_ref": "expression:1",
            "expression_revision": "rev-7",
            "profile_ref": "profile:1",
            "profile_revision": "rev-2",
            "coordinate_ref": "M2-5-9:C0",
            "bimba_registry_revision": "registry-3"
        },
        "brief": "resolve the relation under focus",
        "scope_refs": ["source:1"],
        "delegated_at_unix_ms": 1,
        "state": "delegated"
    });
    let receipt = nara
        .delegate_to_epii(delegation, "delegation-receipt:1", "2026-09-17T09:34:00Z")
        .unwrap();
    assert_eq!(receipt["foreground_agent"], "nara:canonical");
    assert_ne!(receipt["epii_session_ref"], receipt["nara_agent_ref"]);
    assert_eq!(receipt["basis_current_at_receipt"], true);

    let enrichment = json!({
        "schema": QL_EPII_ENRICHMENT,
        "enrichment_ref": "enrichment:1",
        "delegation_ref": "delegation:1",
        "basis_context_ref": "context:encounter-1",
        "basis_expression_revision": "rev-7",
        "proposed_focus_refs": ["source:1"],
        "coordinate_refs": [],
        "summary": "the relation resolves as ..."
    });
    // Current result: proposed only. `applied: false` is structural.
    let fresh = nara
        .receive_enrichment(
            enrichment.clone(),
            "enrichment-receipt:1",
            "2026-09-17T09:35:00Z",
        )
        .unwrap();
    assert_eq!(fresh["standing"], "proposed-only");
    assert_eq!(fresh["applied"], false);
    // Late result (encounter moved to rev-9): retained, never applied.
    let mut moved = nara.clone();
    let mut next = context("nara:canonical", "session:nara-1");
    next["expression_revision"] = json!("rev-9");
    moved.update_context(next).unwrap();
    let late = moved
        .receive_enrichment(enrichment, "enrichment-receipt:2", "2026-09-17T09:36:00Z")
        .unwrap();
    assert_eq!(late["standing"], "retained-not-applied");
    assert_eq!(late["applied"], false);
}

/// Demand 9: two Naras on one projected world keep separate Agency, session
/// and context; co-presence is not shared authority.
#[test]
fn two_naras_on_one_world_stay_separate() {
    let a = realtime_nara();
    let mut collision_context = context("nara:canonical", "session:nara-1");
    collision_context["context_ref"] = json!("context:encounter-2");
    let collision = NaraBinding::constitute(
        constitution(
            "realtime",
            "session:nara-1",
            true,
            json!({"state": "supported"}),
        ),
        collision_context,
        vec![],
        vec![],
    )
    .unwrap();
    assert!(shared_world_guard(&a, &collision).is_err());

    let mut b_value = constitution(
        "realtime",
        "session:nara-b",
        true,
        json!({"state": "supported"}),
    )
    .into_value();
    b_value["agent_ref"] = json!("nara:b");
    b_value["agency_ref"] = json!("agency:nara-b");
    b_value["constitution_ref"] = json!("constitution:nara-b");
    let mut b_context = context("nara:b", "session:nara-b");
    b_context["context_ref"] = json!("context:b-1");
    let b = NaraBinding::constitute(
        SpeechConstitution::try_from(b_value).unwrap(),
        b_context,
        vec![],
        vec![],
    )
    .unwrap();
    shared_world_guard(&a, &b).unwrap();
    assert_ne!(a.agent_ref(), b.agent_ref());
    assert_ne!(a.session().agency_ref(), b.session().agency_ref());
    assert_ne!(
        a.session().agent_session_ref(),
        b.session().agent_session_ref()
    );
}

/// Demand 10: speech failure leaves the text Nara embodiment usable on the
/// same Agent/Agency/session identity.
#[test]
fn speech_failure_falls_back_to_a_usable_text_body() {
    let mut nara = realtime_nara();
    let change = nara
        .reconnect(
            "change:speech-failure",
            constitution(
                "text",
                "session:nara-1",
                false,
                json!({"state": "unsupported", "reason": "no speech output exists to interrupt"}),
            ),
            context("nara:canonical", "session:nara-1"),
            "speech body failed; text-only fallback",
            vec![ExternalRef::new("evidence:speech-failure").unwrap()],
            "2026-09-17T09:40:00Z",
        )
        .unwrap();
    assert_eq!(change.identity().0.as_str(), "nara:canonical");
    assert!(!nara.session().constitution().speech_capable());
    // Speech turns refuse honestly.
    assert!(nara.session_mut().begin_listening().is_err());
    // Text dialogue continues on the same Nara.
    nara.session_mut()
        .begin_response(ExternalRef::new("response:text-1").unwrap())
        .unwrap();
    nara.session_mut().complete_response().unwrap();
    assert_eq!(nara.nara_ref(), "nara:canonical");
    // And the interrupted-in-flight law never confused text with speech:
    // interruption on a text-only body is a recorded unsupported fact.
    nara.session_mut()
        .begin_response(ExternalRef::new("response:text-2").unwrap())
        .unwrap();
    let receipt = nara
        .interrupt("interruption:text-1", "stop", "2026-09-17T09:41:00Z")
        .unwrap();
    assert!(!receipt.cancelled());
}
