#[allow(dead_code)]
#[path = "support/runtime.rs"]
mod runtime;
use actuation_core::*;
use actuation_runtime::*;
use runtime::block_on;
use serde_json::{json, Value};

fn seed(operation: &str) -> Value {
    let corpus: Value =
        serde_json::from_str(include_str!("../../../fixtures/migration/oracle.json")).unwrap();
    corpus["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["operation"] == operation && r["expected"]["ok"] == true)
        .unwrap()["args"][0]
        .clone()
}
fn binding() -> WorldBinding {
    WorldBinding::new(WorldBindingFields::new(
        WorldBindingRef::new("binding:direct").unwrap(),
        AgentRef::new("agent:direct").unwrap(),
        AgencyRef::new("agency:direct").unwrap(),
        WorldRef::new("file:///project").unwrap(),
        ScopeRef::new("scope:directory").unwrap(),
    ))
    .unwrap()
}
fn realisation() -> RealisedActuation {
    serde_json::from_value(json!({"schema":"actuation.realised/v1","realised_ref":"realised:direct","actuation_ref":"actuation:direct","agent_ref":"agent:direct","agency_ref":"agency:direct","world_binding_ref":"binding:direct",
        "loop":{"recurrence":"event-driven","acting":true},"observation":{"state":"observed","evidence_refs":["D:injected-driver-boundary"]},
        "body":{"harness_ref":"aikit:body:1","session_ref":"aikit:session:1","process_ref":"process:1","model_condition_ref":"model:1","material_binding_ref":"workcell:binding:1"}})).unwrap()
}
fn report(phase: LocusPhase) -> DriverOutcome<LocusReport> {
    DriverOutcome::Observed {
        value: LocusReport {
            realised: realisation(),
            phase,
        },
        evidence_refs: NonEmpty::one(ExternalRef::new("D:fixture-driver").unwrap()),
    }
}
struct TestDriver {
    next: DriverOutcome<LocusReport>,
    begin_calls: usize,
    control: bool,
    returned: Option<Return>,
}
impl TestDriver {
    fn new() -> Self {
        Self {
            next: report(LocusPhase::Acting),
            begin_calls: 0,
            control: false,
            returned: None,
        }
    }
}
impl LocusDriver for TestDriver {
    fn observe<'a>(
        &'a mut self,
        _: &'a WorldBinding,
    ) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        Box::pin(async move { Ok(self.next.clone()) })
    }
    fn begin<'a>(&'a mut self, _: &'a WorldBinding) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        self.begin_calls += 1;
        Box::pin(async move { Ok(self.next.clone()) })
    }
    fn resume<'a>(&'a mut self, _: &'a WorldBinding) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        Box::pin(async move { Ok(self.next.clone()) })
    }
    fn interrupt<'a>(
        &'a mut self,
        _: &'a WorldBinding,
    ) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        Box::pin(async move {
            Ok(if self.control {
                self.next.clone()
            } else {
                DriverOutcome::unsupported("native interrupt unavailable")
            })
        })
    }
    fn cancel<'a>(&'a mut self, _: &'a WorldBinding) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        Box::pin(async move {
            Ok(if self.control {
                self.next.clone()
            } else {
                DriverOutcome::unsupported("native cancel unavailable")
            })
        })
    }
    fn terminate<'a>(
        &'a mut self,
        _: &'a WorldBinding,
    ) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        Box::pin(async move {
            Ok(if self.control {
                self.next.clone()
            } else {
                DriverOutcome::unsupported("native terminate unavailable")
            })
        })
    }
    fn collect_return<'a>(
        &'a mut self,
        _: &'a WorldBinding,
    ) -> PortFuture<'a, DriverOutcome<Return>> {
        Box::pin(async move {
            Ok(self
                .returned
                .clone()
                .map(|value| DriverOutcome::Observed {
                    value,
                    evidence_refs: NonEmpty::one(ExternalRef::new("D:returned-boundary").unwrap()),
                })
                .unwrap_or_else(|| DriverOutcome::unsupported("no Return")))
        })
    }
}
#[test]
fn managed_and_discovered_share_grammar_without_discovered_launch() {
    let mut managed = Actuator::new(binding(), DrivingMode::Managed, TestDriver::new());
    let mut discovered = Actuator::new(binding(), DrivingMode::Discovered, TestDriver::new());
    assert!(matches!(
        block_on(discovered.begin()).unwrap(),
        DriverOutcome::Unsupported { .. }
    ));
    assert_eq!(discovered.driver().begin_calls, 0);
    block_on(managed.begin()).unwrap();
    block_on(discovered.observe()).unwrap();
    assert_eq!(managed.current(), discovered.current());
    assert_eq!(
        managed
            .current()
            .unwrap()
            .realised
            .fields()
            .body
            .value()
            .unwrap()
            .fields()
            .material_binding_ref
            .value()
            .unwrap()
            .as_str(),
        "workcell:binding:1"
    );
    assert_eq!(managed.binding().identity(), binding().identity());
}
#[test]
fn body_relocation_preserves_identity_and_missing_differs_from_null() {
    let first = realisation();
    let mut wire = serde_json::to_value(&first).unwrap();
    for key in [
        "harness_ref",
        "session_ref",
        "process_ref",
        "model_condition_ref",
        "material_binding_ref",
    ] {
        wire["body"][key] = json!(format!("external:{key}:replacement"));
    }
    let changed: RealisedActuation = serde_json::from_value(wire.clone()).unwrap();
    let delta = first.continuity_to(&changed);
    assert!(
        delta.same_agent && delta.same_agency && delta.same_world_binding && delta.same_actuation
    );
    assert!(
        delta.harness_changed
            && delta.session_changed
            && delta.process_changed
            && delta.model_condition_changed
            && delta.material_binding_changed
    );
    wire["body"] = json!({});
    let absent: RealisedActuation = serde_json::from_value(wire.clone()).unwrap();
    wire["body"]["process_ref"] = Value::Null;
    let null: RealisedActuation = serde_json::from_value(wire).unwrap();
    assert!(absent.continuity_to(&null).process_changed);
}
#[test]
fn unsupported_controls_do_not_change_observed_state() {
    let mut locus = Actuator::new(binding(), DrivingMode::Discovered, TestDriver::new());
    block_on(locus.observe()).unwrap();
    assert!(matches!(
        block_on(locus.interrupt()).unwrap(),
        DriverOutcome::Unsupported { .. }
    ));
    assert!(matches!(
        block_on(locus.cancel()).unwrap(),
        DriverOutcome::Unsupported { .. }
    ));
    assert!(matches!(
        block_on(locus.terminate()).unwrap(),
        DriverOutcome::Unsupported { .. }
    ));
    assert_eq!(locus.phase(), LocusPhase::Acting);
}
#[test]
fn lifecycle_claim_requires_matching_observed_post_state() {
    for phase in [
        LocusPhase::Acting,
        LocusPhase::Interrupted,
        LocusPhase::Cancelled,
        LocusPhase::Terminated,
    ] {
        let mut driver = TestDriver::new();
        driver.next = report(phase);
        driver.control = true;
        let mut locus = Actuator::new(binding(), DrivingMode::Managed, driver);
        if phase == LocusPhase::Acting {
            block_on(locus.begin()).unwrap();
            assert!(block_on(locus.interrupt()).is_err());
            assert!(block_on(locus.cancel()).is_err());
            assert!(block_on(locus.terminate()).is_err());
        } else {
            block_on(locus.observe()).unwrap();
            if phase == LocusPhase::Interrupted {
                assert!(block_on(locus.interrupt()).is_ok());
            } else {
                assert!(block_on(locus.resume()).is_err());
            }
        }
    }
}
#[test]
fn driver_identity_or_unavailable_claim_is_refused() {
    for (key, value) in [
        ("agent_ref", json!("agent:imposter")),
        ("agency_ref", json!("agency:imposter")),
        ("world_binding_ref", json!("binding:imposter")),
        ("observation", json!({"state":"unavailable"})),
    ] {
        let mut wire = serde_json::to_value(realisation()).unwrap();
        wire[key] = value;
        let mut driver = TestDriver::new();
        driver.next = DriverOutcome::Observed {
            value: LocusReport {
                realised: serde_json::from_value(wire).unwrap(),
                phase: LocusPhase::Acting,
            },
            evidence_refs: NonEmpty::one(ExternalRef::new("D:insufficient").unwrap()),
        };
        let mut locus = Actuator::new(binding(), DrivingMode::Managed, driver);
        assert!(block_on(locus.begin()).is_err(), "{key}");
        assert_eq!(locus.phase(), LocusPhase::Unobserved);
    }
}
#[test]
fn return_collection_retains_difference_and_does_not_recognise() {
    let mut wire = seed("contracts/agency.mjs#validateReturn");
    wire["from_agency_ref"] = json!("agency:direct");
    wire["provenance"] = json!({"agency_lineage_refs":["agency:parent","agency:direct"],"agent_refs":["agent:direct"],"world_binding_refs":["binding:direct"],"actuation_refs":["actuation:direct"]});
    wire["received"] = json!(false);
    wire["recognition_state"] = json!("pending");
    wire["world_mutation_state"] = json!("not-applied");
    let returned: Return = serde_json::from_value(wire.clone()).unwrap();
    let mut driver = TestDriver::new();
    driver.returned = Some(returned.clone());
    let mut locus = Actuator::new(binding(), DrivingMode::Discovered, driver);
    block_on(locus.observe()).unwrap();
    let DriverOutcome::Observed { value, .. } = block_on(locus.collect_return()).unwrap() else {
        panic!("must observe supplied Return")
    };
    assert_eq!(value, returned);
    assert_eq!(value.standing(), ReturnStanding::Offered);
    for key in ["agent_refs", "world_binding_refs", "actuation_refs"] {
        let mut wrong = wire.clone();
        wrong["provenance"][key] = json!(["ref:unrelated"]);
        let mut driver = TestDriver::new();
        driver.returned = Some(serde_json::from_value(wrong).unwrap());
        let mut locus = Actuator::new(binding(), DrivingMode::Discovered, driver);
        block_on(locus.observe()).unwrap();
        assert!(block_on(locus.collect_return()).is_err(), "{key}");
    }
}
#[test]
fn receiving_an_act_is_not_full_authority_admission() {
    let valid = seed("contracts/agency-actualisation.mjs#actualiseAgency");
    let admitted = serde_json::from_value::<ActualisationRequest>(valid.clone())
        .unwrap()
        .admit()
        .unwrap();
    assert_eq!(
        admitted.receipt()["effects"],
        json!({"semantic_relation":"actualised","materialisation":"not-performed","factory_recognition":"not-performed","source_mutation":"not-performed"})
    );
    for (pointer, value) in [
        ("/metagency_grant/agency_ref", json!("agency:other")),
        ("/metagency_grant/world_binding_ref", json!("binding:other")),
        (
            "/metagency_grant/authority_ref",
            json!("authority:invented"),
        ),
        ("/metagency_grant/bounds_refs", json!(["bounds:invented"])),
        ("/governing_binding/bounds_refs", json!([])),
        (
            "/differentiated_binding/authority_refs",
            json!(["authority:invented"]),
        ),
        (
            "/differentiated_binding/bounds_refs",
            json!(["bounds:invented"]),
        ),
        (
            "/differentiated_binding/return_relation_ref",
            json!("return:invented"),
        ),
        ("/agent_identity/evidence_refs", json!([])),
        ("/determination/parent_determination_ref", Value::Null),
    ] {
        let mut wrong = valid.clone();
        // Insert optional root-parent as well as overwrite existing fields.
        let (parent, key) = pointer.rsplit_once('/').unwrap();
        wrong
            .pointer_mut(parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert(key.into(), value);
        let parsed = serde_json::from_value::<ActualisationRequest>(wrong);
        assert!(
            parsed
                .and_then(|r| r.admit().map_err(serde::de::Error::custom))
                .is_err(),
            "{pointer}"
        );
    }
}
#[test]
fn actualisation_return_must_retain_exact_lineage_before_synthesis() {
    let admitted = serde_json::from_value::<ActualisationRequest>(seed(
        "contracts/agency-actualisation.mjs#actualiseAgency",
    ))
    .unwrap()
    .admit()
    .unwrap();
    let d = admitted.request().fields().determination.fields();
    let mut wire = seed("contracts/agency.mjs#validateReturn");
    wire["determination_ref"] = json!(d.determination_ref);
    wire["from_agency_ref"] = json!(d.differentiated_agency_ref);
    wire["to_agency_ref"] = json!(d.determining_agency_ref);
    wire["provenance"] =
        json!({"agency_lineage_refs":[d.determining_agency_ref,d.differentiated_agency_ref]});
    let valid: Return = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(admitted.admit_return(valid.clone()).unwrap(), valid);
    wire["determination_ref"] = json!("determination:other");
    assert!(admitted
        .admit_return(serde_json::from_value(wire).unwrap())
        .is_err());
}

struct SequenceDriver(std::collections::VecDeque<LocusReport>);
impl LocusDriver for SequenceDriver {
    fn observe<'a>(
        &'a mut self,
        _: &'a WorldBinding,
    ) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        let value = self.0.pop_front();
        Box::pin(async move {
            Ok(DriverOutcome::Observed {
                value: value.ok_or_else(|| Error::new("no observation"))?,
                evidence_refs: NonEmpty::one(ExternalRef::new("D:ordered-driver-observation")?),
            })
        })
    }
    fn begin<'a>(&'a mut self, b: &'a WorldBinding) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        self.observe(b)
    }
    fn resume<'a>(&'a mut self, b: &'a WorldBinding) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        self.observe(b)
    }
    fn interrupt<'a>(
        &'a mut self,
        b: &'a WorldBinding,
    ) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        self.observe(b)
    }
    fn cancel<'a>(&'a mut self, b: &'a WorldBinding) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        self.observe(b)
    }
}
#[test]
fn observed_relocation_interrupt_resume_cancel_preserve_identity_and_refuse_reopening() {
    let original = realisation();
    let mut body = serde_json::to_value(&original).unwrap();
    body["body"]["process_ref"] = json!("external:new-process");
    body["body"]["material_binding_ref"] = json!("workcell:new-material");
    let relocated: RealisedActuation = serde_json::from_value(body).unwrap();
    let mut reports = vec![LocusReport {
        realised: original,
        phase: LocusPhase::Acting,
    }];
    reports.extend(
        [
            LocusPhase::Acting,
            LocusPhase::Interrupted,
            LocusPhase::Acting,
            LocusPhase::Cancelled,
            LocusPhase::Acting,
        ]
        .map(|phase| LocusReport {
            realised: relocated.clone(),
            phase,
        }),
    );
    let mut actuator = Actuator::new(
        binding(),
        DrivingMode::Managed,
        SequenceDriver(reports.into()),
    );
    block_on(actuator.begin()).unwrap();
    block_on(actuator.observe()).unwrap();
    assert_eq!(actuator.current().unwrap().realised, relocated);
    block_on(actuator.interrupt()).unwrap();
    assert_eq!(actuator.phase(), LocusPhase::Interrupted);
    block_on(actuator.resume()).unwrap();
    assert_eq!(actuator.phase(), LocusPhase::Acting);
    block_on(actuator.cancel()).unwrap();
    assert_eq!(actuator.phase(), LocusPhase::Cancelled);
    assert!(block_on(actuator.observe()).is_err());
    assert_eq!(actuator.phase(), LocusPhase::Cancelled);
    assert_eq!(
        actuator.current().unwrap().realised.identity(),
        binding().identity()
    );
}
#[test]
fn unrelated_actuation_cannot_replace_a_continuation() {
    let first = realisation();
    let mut wrong = serde_json::to_value(&first).unwrap();
    wrong["actuation_ref"] = json!("actuation:imposter");
    let reports = vec![
        LocusReport {
            realised: first.clone(),
            phase: LocusPhase::Acting,
        },
        LocusReport {
            realised: serde_json::from_value(wrong).unwrap(),
            phase: LocusPhase::Acting,
        },
    ];
    let mut actuator = Actuator::new(
        binding(),
        DrivingMode::Discovered,
        SequenceDriver(reports.into()),
    );
    block_on(actuator.observe()).unwrap();
    assert!(block_on(actuator.observe()).is_err());
    assert_eq!(actuator.current().unwrap().realised, first);
}
