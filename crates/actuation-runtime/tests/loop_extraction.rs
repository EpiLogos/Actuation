#[path = "support/runtime.rs"]
mod runtime;
use actuation_core::*;
use actuation_runtime::*;
use runtime::{block_on, ScriptedHost, Witness};
use serde_json::{json, Value};

#[test]
fn generic_runtime_extraction_matches_original_node_host_calls_events_and_results() {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/migration/runtime.json")).unwrap();
    assert_eq!(corpus["schema"], "actuation.runtime-extraction/v1");
    let cases = corpus["cases"].as_array().unwrap();
    assert!(cases.len() >= 29);
    for row in cases {
        assert_eq!(
            runtime::evaluate(row).unwrap(),
            row["expected"],
            "{}",
            row["id"]
        );
    }
}
fn binding() -> WorldBinding {
    WorldBinding::new(WorldBindingFields::new(
        WorldBindingRef::new("binding:project").unwrap(),
        AgentRef::new("agent:direct").unwrap(),
        AgencyRef::new("agency:direct").unwrap(),
        WorldRef::new("file:///project").unwrap(),
        ScopeRef::new("scope:directory").unwrap(),
    ))
    .unwrap()
}
#[test]
fn directory_bound_acting_model_is_a_complete_valid_direct_locus_without_aikit_or_factory() {
    let request =
        LoopRequest::from_legacy(json!({"taskId":"direct","input":"edit within project"})).unwrap();
    let driver = LoopLocusDriver::new(
        ScriptedHost::new(&json!({"models":[{"content":"done"}]})),
        Witness::default(),
        request,
        RealisedRef::new("realised:direct").unwrap(),
        ActuationRef::new("actuation:direct").unwrap(),
    );
    let mut actuator = Actuator::new(binding(), DrivingMode::Managed, driver);
    assert_eq!(actuator.phase(), LocusPhase::Unobserved);
    let result = block_on(actuator.begin()).unwrap();
    assert!(matches!(result, DriverOutcome::Observed { .. }));
    let current = actuator.current().unwrap();
    assert_eq!(current.realised.identity(), binding().identity());
    assert!(current.realised.fields().body.is_absent());
    assert_eq!(
        current.realised.fields().observation.fields().state,
        ObservationState::Observed
    );
    assert_eq!(
        actuator.driver().execution().unwrap().report.status,
        LoopStatus::Completed
    );
    assert_eq!(actuator.driver().execution().unwrap().report.model_calls, 1);
    assert_eq!(actuator.driver().observer().events.len(), 2);
    assert!(block_on(actuator.begin()).is_err());
    assert!(matches!(
        block_on(actuator.collect_return()).unwrap(),
        DriverOutcome::Unsupported { .. }
    ));
}
#[test]
fn model_availability_failure_cannot_become_an_acting_receipt() {
    let request = LoopRequest::from_legacy(json!({"taskId":"inert"})).unwrap();
    let driver = LoopLocusDriver::new(
        ScriptedHost::new(&json!({"models":[{"throw":"unavailable"}]})),
        Witness::default(),
        request,
        RealisedRef::new("realised:inert").unwrap(),
        ActuationRef::new("actuation:inert").unwrap(),
    );
    let mut actuator = Actuator::new(binding(), DrivingMode::Managed, driver);
    assert!(matches!(
        block_on(actuator.begin()).unwrap(),
        DriverOutcome::Unsupported { .. }
    ));
    assert_eq!(actuator.phase(), LocusPhase::Unobserved);
    assert_eq!(
        actuator.driver().execution().unwrap().report.status,
        LoopStatus::Failed
    );
}
#[test]
fn refusing_observer_does_not_produce_success_or_fabricated_evidence() {
    let request = LoopRequest::from_legacy(json!({"taskId":"refused"})).unwrap();
    let mut host = ScriptedHost::new(&json!({"models":[{"content":"done"}]}));
    let mut witness = Witness {
        events: vec![],
        refuse: true,
    };
    assert!(block_on(ClassicRuntime.run(
        &request,
        &mut host,
        &mut witness,
        &CancellationToken::default()
    ))
    .is_err());
    assert!(host.calls.is_empty());
}
#[test]
fn cancellation_is_a_request_until_the_loop_observes_it() {
    let request = LoopRequest::from_legacy(json!({"taskId":"cancel"})).unwrap();
    let mut host = ScriptedHost::new(&json!({"models":[{"content":"unused"}]}));
    let mut witness = Witness::default();
    let cancellation = CancellationToken::default();
    cancellation.request();
    let execution =
        block_on(ClassicRuntime.run(&request, &mut host, &mut witness, &cancellation)).unwrap();
    assert_eq!(execution.report.status, LoopStatus::Cancelled);
    assert_eq!(execution.report.model_calls, 0);
    assert!(host.calls.is_empty());
}
