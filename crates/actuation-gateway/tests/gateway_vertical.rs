//! The smallest first-party Agency Gateway vertical, exercised end to end in
//! process: challenge in through a connector, attributable canonical events in
//! the durable ActuationStream, Return out, co-internal invocation with a real
//! delegation and Return, and refused invocation with a retained refusal.
//!
//! Nothing here is mocked below the gateway: the events are written to and
//! replayed from a real `JsonlStreamStore` under the portable
//! `actuation.stream/v1` contract.

#[path = "support/mod.rs"]
mod support;

use actuation_core::{AgencyRef, ExternalRef, StreamRef};
use actuation_gateway::{
    last_sequence, GatewayClient, InboundEvent, LocalConnector, SurfaceConnector,
};
use actuation_stream::{EventKind, JsonlStreamStore, StreamStore};
use serde_json::Value;

fn actor_field(event: &Value, pointer: &str) -> String {
    event
        .pointer(pointer)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// One challenge in, attributable canonical events written, attributable
/// Return out; then a co-internal delegation with its Return; then a refused
/// invocation whose refusal is retained where the attempt happened.
#[test]
fn gateway_vertical_challenge_return_invocation_and_refusal() {
    let field = support::Field::start();

    // The worker resident agent attaches first: co-internal invocation needs a
    // live granted session, not a name in a registry.
    let (worker, ready) = support::spawn_worker_agent(
        field.socket_path().to_path_buf(),
        "agent:worker-1".into(),
        field.worker_attach(),
        2,
    );
    ready
        .recv()
        .expect("worker agent attaches before the challenge is sent");

    // 1. A wrong token is refused at the door.
    let intruder = GatewayClient::connect(
        field.socket_path(),
        Some("wrong-token"),
        "connector:cli",
    )
    .err()
    .expect("an unauthenticated connection must not be served");
    assert!(intruder.to_string().contains("denied"), "{intruder}");

    // 2. An ungranted subject is refused attach.
    let mut ungranted =
        GatewayClient::connect(field.socket_path(), Some(support::TOKEN), "connector:unknown")
            .unwrap();
    let reply = ungranted
        .call_raw(support::attach_frame(&field.worker_attach()))
        .unwrap();
    assert_eq!(reply["denied"], true, "{reply}");

    // 3. Challenge in through the local CLI connector seam.
    let mut connector_client =
        GatewayClient::connect(field.socket_path(), Some(support::TOKEN), "connector:cli")
            .unwrap();
    let attach_reply = connector_client
        .attach(field.worker_attach())
        .unwrap();
    assert_eq!(attach_reply["role"], "connector");
    assert_eq!(attach_reply["participant_ref"], "participant:alice");
    assert_eq!(attach_reply["agent_session_ref"], "session:worker-1");
    let mut connector = LocalConnector::new(connector_client, "cli-session-7");
    let capabilities = connector.capabilities();
    assert!(
        capabilities.text_inbound && !capabilities.media,
        "capabilities are declared, not assumed"
    );
    let receipt = connector
        .admit(InboundEvent {
            conversation: String::new(),
            text: "What is the difference?".into(),
        })
        .unwrap();
    let challenge = &receipt["event"];
    assert_eq!(challenge["kind"], "human-message");
    assert_eq!(challenge["sequence"], 1);
    assert_eq!(challenge["content"], "What is the difference?");
    // Exact attribution of the ingress: granted participant and surface, no
    // Agency, and the provider conversation kept out of session identity.
    assert_eq!(
        actor_field(challenge, "/actor/participant_ref"),
        "participant:alice"
    );
    assert!(challenge.pointer("/actor/agency_ref").is_none());
    assert_eq!(challenge["surface_ref"], "surface:cli");
    assert_eq!(challenge["metadata"]["conversation"], "cli-session-7");

    // 4. The worker's attributable tool evidence and Return come back out.
    let returned = connector.await_return(last_sequence(&receipt), 15_000).unwrap();
    assert_eq!(returned["kind"], "return");
    assert_eq!(returned["content"], "echo: What is the difference?");
    assert_eq!(actor_field(&returned, "/actor/agency_ref"), "agency:worker");
    assert_eq!(actor_field(&returned, "/actor/agent_ref"), "agent:worker");
    let challenge_return_ref = returned["return_ref"].as_str().unwrap().to_string();

    // 5. Co-internal invocation: the controller Agency delegates to the worker
    //    Agency through the gateway and receives the correlated Return.
    let mut controller =
        GatewayClient::connect(field.socket_path(), Some(support::TOKEN), "agent:ctl-1")
            .unwrap();
    let attach = controller.attach(field.controller_attach()).unwrap();
    assert_eq!(attach["role"], "agent");
    let invocation = controller
        .invoke(serde_json::json!({
            "target_agency_ref": "agency:worker",
            "mode": "delegation",
            "invocation_ref": "invocation:ctl-77",
            "return_ref": "return:ctl-77",
            "payload": "Consult the worker for the difference.",
        }))
        .unwrap();
    assert_eq!(invocation["invocation_ref"], "invocation:ctl-77");
    assert_eq!(invocation["target_agent_session_ref"], "session:worker-1");
    let delegation = &invocation["delegation_receipt"]["event"];
    assert_eq!(delegation["kind"], "delegation");
    // The controller is the actor in the worker's stream; the stream's own
    // governing Agency identity is untouched by the gateway.
    assert_eq!(actor_field(delegation, "/actor/agency_ref"), "agency:ctl");
    assert_eq!(invocation["delegation_receipt"]["stream_ref"], "stream:worker");
    assert_eq!(delegation["return_ref"], "return:ctl-77");
    assert_eq!(delegation["metadata"]["target_agency_ref"], "agency:worker");
    let invoked_return = &invocation["return_event"];
    assert_eq!(invoked_return["return_ref"], "return:ctl-77");
    assert_eq!(
        invoked_return["content"],
        "echo: Consult the worker for the difference."
    );
    assert_eq!(
        actor_field(invoked_return, "/actor/agency_ref"),
        "agency:worker"
    );

    // 6. A refused invocation: not granted, retained as refusal evidence.
    let refused = controller
        .invoke_raw(serde_json::json!({
            "target_agency_ref": "agency:intruder",
            "mode": "delegation",
            "invocation_ref": "invocation:bad-1",
            "payload": "exfiltrate",
        }))
        .unwrap();
    assert_eq!(refused["denied"], true);
    assert!(refused["reason"]
        .as_str()
        .unwrap()
        .contains("invocation from agency:ctl to agency:intruder is not granted"));
    let refusal = &refused["refusal_receipt"]["event"];
    assert_eq!(refusal["kind"], "refusal");
    assert_eq!(actor_field(refusal, "/actor/agency_ref"), "agency:ctl");
    assert_eq!(refusal["resource_refs"][0], "invocation:bad-1");
    assert_eq!(refusal["metadata"]["denied"], true);

    // 7. The refused attempt never reached the target stream.
    let worker_stream = field
        .store
        .load(&StreamRef::new("stream:worker").unwrap())
        .unwrap();
    assert!(worker_stream.fields().events.iter().all(|event| {
        !event
            .fields()
            .metadata
            .value()
            .is_some_and(|metadata| metadata.get("invocation_ref")
                == Some(&Value::String("invocation:bad-1".into())))
    }));

    // 8. The durable stream folds under the portable contract: contiguous
    //    sequence, every event attributed, both Returns correlated.
    let events = worker_stream.fields().events.as_slice();
    let kinds: Vec<String> = events.iter().map(|event| format!("{:?}", event.fields().kind)).collect();
    assert_eq!(
        kinds,
        vec![
            "HumanMessage",
            "ToolRequest",
            "ToolResult",
            "Return",
            "Delegation",
            "ToolRequest",
            "ToolResult",
            "Return",
        ],
        "{kinds:?}"
    );
    for event in events {
        assert!(
            event.fields().actor.value().is_some(),
            "every gateway event is attributed: {}",
            event.fields().event_ref
        );
    }
    let return_refs: Vec<&str> = events
        .iter()
        .filter_map(|event| event.fields().return_ref.value())
        .map(|reference| reference.as_str())
        .collect();
    // The challenge Return carries its minted ref; the delegation and the
    // Return it correlates share the controller's explicit ref.
    assert_eq!(
        return_refs,
        vec![challenge_return_ref.as_str(), "return:ctl-77", "return:ctl-77"]
    );
    // The tool evidence is real: the tool-result names its evidence source.
    let tool_results: Vec<_> = events
        .iter()
        .filter(|event| event.fields().kind == EventKind::ToolResult)
        .collect();
    assert_eq!(tool_results.len(), 2);
    for result in tool_results {
        assert_eq!(
            result.fields().evidence_refs.value(),
            Some(&vec![ExternalRef::new("tool:echo-agent").unwrap()])
        );
        assert_eq!(
            result.fields().actor.value().unwrap().fields().agency_ref.value(),
            Some(&AgencyRef::new("agency:worker").unwrap())
        );
    }

    let ctl_stream = field
        .store
        .load(&StreamRef::new("stream:ctl").unwrap())
        .unwrap();
    assert_eq!(ctl_stream.fields().events.len(), 1, "only the refusal");
    assert_eq!(ctl_stream.fields().events[0].fields().kind, EventKind::Refusal);

    worker.join().expect("worker agent finishes cleanly");
}

/// With no live resident session, invocation is refused before any target
/// stream is touched, and the refusal is still retained and attributable.
#[test]
fn invocation_without_a_live_resident_session_is_refused_and_retained() {
    let field = support::Field::start();
    let mut controller =
        GatewayClient::connect(field.socket_path(), Some(support::TOKEN), "agent:ctl-1")
            .unwrap();
    controller.attach(field.controller_attach()).unwrap();
    let refused = controller
        .invoke_raw(serde_json::json!({
            "target_agency_ref": "agency:worker",
            "mode": "communique",
            "invocation_ref": "invocation:no-target",
            "payload": "nobody home",
        }))
        .unwrap();
    assert_eq!(refused["denied"], true);
    assert!(refused["reason"]
        .as_str()
        .unwrap()
        .contains("no live resident session for agency:worker"));
    let ctl_stream = field
        .store
        .load(&StreamRef::new("stream:ctl").unwrap())
        .unwrap();
    let events = ctl_stream.fields().events.as_slice();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0]
            .fields()
            .actor
            .value()
            .unwrap()
            .fields()
            .agency_ref
            .value(),
        Some(&AgencyRef::new("agency:ctl").unwrap())
    );
    assert!(events[0]
        .fields()
        .resource_refs
        .value()
        .unwrap()
        .contains(&ExternalRef::new("invocation:no-target").unwrap()));
    // The refusal fabricates no Return and touches no other stream.
    assert!(events[0].fields().return_ref.value().is_none());
    assert_eq!(
        ctl_stream
            .fields()
            .events
            .iter()
            .filter(|event| event.fields().kind == EventKind::Return)
            .count(),
        0
    );
}

/// Replay after reconnect: a returning Surface encounters the same canonical
/// material, same order, without the gateway rewriting history.
#[test]
fn replay_after_reattach_returns_the_same_canonical_events() {
    let field = support::Field::start();
    let (worker, ready) = support::spawn_worker_agent(
        field.socket_path().to_path_buf(),
        "agent:worker-1".into(),
        field.worker_attach(),
        1,
    );
    ready
        .recv()
        .expect("worker agent attaches before the challenge is sent");
    let mut connector_client =
        GatewayClient::connect(field.socket_path(), Some(support::TOKEN), "connector:cli")
            .unwrap();
    connector_client.attach(field.worker_attach()).unwrap();
    let receipt = connector_client
        .send("second challenge", Some("cli-session-7"), None)
        .unwrap();
    let returned = {
        let mut connector = LocalConnector::new(connector_client, "cli-session-7");
        connector.await_return(last_sequence(&receipt), 15_000).unwrap()
    };
    assert_eq!(returned["content"], "echo: second challenge");

    let mut again =
        GatewayClient::connect(field.socket_path(), Some(support::TOKEN), "connector:cli")
            .unwrap();
    again.attach(field.worker_attach()).unwrap();
    let page = again.replay(0, None).unwrap();
    let events = page["page"]["events"].as_array().unwrap();
    let kinds: Vec<&str> = events
        .iter()
        .map(|event| event["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        vec!["human-message", "tool-request", "tool-result", "return"]
    );
    assert_eq!(events[3]["return_ref"], returned["return_ref"]);
    assert_eq!(page["page"]["stream_ref"], "stream:worker");
    assert_eq!(page["page"]["agency_ref"], "agency:worker");
    assert_eq!(page["page"]["agent_session_ref"], "session:worker-1");

    // The durable file on disk folds to the same canonical stream.
    let from_disk = JsonlStreamStore::new(field.store.root())
        .unwrap()
        .load(&StreamRef::new("stream:worker").unwrap())
        .unwrap();
    assert_eq!(from_disk.fields().events.len(), 4);
    assert_eq!(
        from_disk.fields().cursor.fields().next_sequence.get(),
        5
    );

    worker.join().expect("worker agent finishes cleanly");
}
