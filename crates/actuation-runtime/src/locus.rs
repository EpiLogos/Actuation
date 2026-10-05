use crate::*;
use actuation_core::*;
use serde::{Deserialize, Serialize};

/// A tighter condition on an already-authorised turn. This wire value does not
/// authenticate a Factory principal, an epoch or a Source. Factory retains its
/// own total-clock/current-intent completion guard, including preparation time.
pub const NATIVE_TURN_BOUND_VERSION: &str = "actuation.native-turn-bound/v1";
pub const NATIVE_TURN_DEADLINE_CAPABILITY: &str = "actuation.native-turn-deadline/cooperative/v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NativeTurnBoundVersion {
    #[serde(rename = "actuation.native-turn-bound/v1")]
    V1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeTurnStopRequirement {
    CooperativeTurn,
    OwnedProcess,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactoryTotalBudgetBasis {
    pub original_attempt_ref: String,
    pub run_ref: String,
    pub unit_ref: String,
    pub execution_ref: String,
    pub budget_source_ref: String,
    pub budget_source_revision: String,
    pub budget_source_digest: String,
    pub intent_ref: String,
    /// A native Factory dispatch-clock observation, forwarded as provenance.
    /// The host does not promote this number into its own clock authority.
    pub dispatch_started_unix_ms: u64,
    pub total_duration_ms: u64,
    pub total_deadline_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTurnBound {
    pub schema: NativeTurnBoundVersion,
    pub agent_ref: String,
    pub agency_ref: String,
    pub world_binding_ref: String,
    pub task_ref: String,
    pub delivery_ref: String,
    pub expected_binding_revision: String,
    pub total: FactoryTotalBudgetBasis,
    pub remaining_ms: u64,
    pub remaining_observation_ref: String,
    pub required_stop: NativeTurnStopRequirement,
}

impl NativeTurnBound {
    pub fn validate(&self) -> Result<()> {
        let refs = [
            &self.agent_ref,
            &self.agency_ref,
            &self.world_binding_ref,
            &self.task_ref,
            &self.delivery_ref,
            &self.expected_binding_revision,
            &self.total.original_attempt_ref,
            &self.total.run_ref,
            &self.total.unit_ref,
            &self.total.execution_ref,
            &self.total.budget_source_ref,
            &self.total.budget_source_revision,
            &self.total.budget_source_digest,
            &self.total.intent_ref,
            &self.remaining_observation_ref,
        ];
        if refs
            .iter()
            .any(|value| value.trim().is_empty() || value.len() > 1024)
            || self.remaining_ms == 0
            || self.remaining_ms > self.total.total_duration_ms
            || self
                .total
                .dispatch_started_unix_ms
                .checked_add(self.total.total_duration_ms)
                != Some(self.total.total_deadline_unix_ms)
        {
            return Err(Error::new(
                "native turn condition is incomplete or inconsistent",
            ));
        }
        Ok(())
    }

    /// Canonical condition basis: UTF-8 JSON of this fixed-order nested array.
    /// No maps, optional fields, current admission or digest is in this basis.
    /// Consumers use `blake3-v1:` + lowercase BLAKE3 hex of these exact bytes.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>> {
        self.validate()?;
        Ok(serde_json::to_vec(&serde_json::json!([
            NATIVE_TURN_BOUND_VERSION,
            self.agent_ref,
            self.agency_ref,
            self.world_binding_ref,
            self.task_ref,
            self.delivery_ref,
            self.expected_binding_revision,
            [
                self.total.original_attempt_ref,
                self.total.run_ref,
                self.total.unit_ref,
                self.total.execution_ref,
                self.total.budget_source_ref,
                self.total.budget_source_revision,
                self.total.budget_source_digest,
                self.total.intent_ref,
                self.total.dispatch_started_unix_ms,
                self.total.total_duration_ms,
                self.total.total_deadline_unix_ms
            ],
            self.remaining_ms,
            self.remaining_observation_ref,
            self.required_stop
        ]))?)
    }
}

/// Transport identities fence a single native operation. They are not Agent,
/// Agency or World identities and cannot allocate or replace any of them.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTurnSelector {
    pub agent_session: String,
    pub native_session_id: String,
    pub connection_generation: String,
    pub delivery_ref: String,
    pub prompt_token: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTurnAdmission {
    pub condition_digest: String,
    pub owner_clock_incarnation: String,
    pub owner_admitted_unix_ms: u64,
    pub owner_admitted_tick_ms: u64,
    pub owner_deadline_tick_ms: u64,
    pub remaining_ms: u64,
    /// Always false: the host admits a forwarded restrictive condition under
    /// its existing authority, not a purported Factory total-clock proof.
    pub factory_total_verified: bool,
}

/// A genuine live owner clock, never restored from an epoch after restart.
/// The caller supplies its existing owner generation, not a semantic World.
pub struct NativeTurnClock {
    incarnation: String,
    started: std::time::Instant,
}

impl NativeTurnClock {
    pub fn new(incarnation: String) -> Result<Self> {
        if incarnation.trim().is_empty() || incarnation.len() > 1024 {
            return Err(Error::new(
                "native owner clock requires its actual incarnation",
            ));
        }
        Ok(Self {
            incarnation,
            started: std::time::Instant::now(),
        })
    }

    fn tick_ms(&self) -> Result<u64> {
        u64::try_from(self.started.elapsed().as_millis())
            .map_err(|e| Error::new("native owner clock overflow").with_source(e))
    }

    pub fn admit(&self, bound: &NativeTurnBound, digest: String) -> Result<NativeTurnAdmission> {
        bound.validate()?;
        if bound.required_stop != NativeTurnStopRequirement::CooperativeTurn {
            return Err(Error::new(
                "this turn port cannot retire a shared provider process",
            ));
        }
        validate_condition_digest(&digest)?;
        let tick = self.tick_ms()?;
        let remaining = bound.remaining_ms;
        let deadline = tick
            .checked_add(remaining)
            .ok_or_else(|| Error::new("native owner deadline overflow"))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| Error::new("native owner wall clock unavailable").with_source(e))?;
        Ok(NativeTurnAdmission {
            condition_digest: digest,
            owner_clock_incarnation: self.incarnation.clone(),
            owner_admitted_unix_ms: u64::try_from(now.as_millis())
                .map_err(|e| Error::new("native owner wall clock overflow").with_source(e))?,
            owner_admitted_tick_ms: tick,
            owner_deadline_tick_ms: deadline,
            remaining_ms: remaining,
            factory_total_verified: false,
        })
    }

    pub fn remaining(&self, admission: &NativeTurnAdmission) -> Result<std::time::Duration> {
        if admission.owner_clock_incarnation != self.incarnation
            || admission.factory_total_verified
            || admission.remaining_ms == 0
            || admission
                .owner_admitted_tick_ms
                .checked_add(admission.remaining_ms)
                != Some(admission.owner_deadline_tick_ms)
        {
            return Err(Error::new(
                "native turn live clock continuity is unavailable",
            ));
        }
        validate_condition_digest(&admission.condition_digest)?;
        let tick = self.tick_ms()?;
        if tick < admission.owner_admitted_tick_ms {
            return Err(Error::new("native turn clock observation is inconsistent"));
        }
        Ok(std::time::Duration::from_millis(
            admission.owner_deadline_tick_ms.saturating_sub(tick),
        ))
    }
}

fn validate_condition_digest(digest: &str) -> Result<()> {
    let Some(hex) = digest.strip_prefix("blake3-v1:") else {
        return Err(Error::new(
            "native condition digest algorithm is unsupported",
        ));
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(Error::new("native condition digest is malformed"));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NativeTurnObservedState {
    Active,
    CancelRequested,
    Completed,
    Cancelled,
    Failed,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTurnObservation {
    pub selector: NativeTurnSelector,
    pub state: NativeTurnObservedState,
    pub event_cursor: Option<u64>,
    pub process_retirement_observed: bool,
}

/// Native adapter/journal ownership stays with the existing host. These methods
/// must revalidate the complete selector before adapter mutation and transport.
pub trait NativeTurnStopPort: Send + Sync {
    fn observe_exact(&self, selector: &NativeTurnSelector) -> Result<NativeTurnObservation>;
    fn request_cancel_exact(
        &self,
        selector: &NativeTurnSelector,
        condition_digest: &str,
    ) -> Result<NativeTurnObservation>;
}

/// This step is driven by the existing owner's event drain, not a new runner or
/// detached timer. It never turns elapsed time into observed cancellation.
pub struct NativeTurnDeadline {
    pub selector: NativeTurnSelector,
    pub admission: NativeTurnAdmission,
    requested: bool,
}

#[cfg(test)]
mod native_turn_clock_tests {
    use super::*;

    fn restriction(duration: u64) -> NativeTurnBound {
        NativeTurnBound {
            schema: NativeTurnBoundVersion::V1,
            agent_ref: "agent:clock-test".into(),
            agency_ref: "agency:clock-test".into(),
            world_binding_ref: "binding:clock-test".into(),
            task_ref: "task:clock-test".into(),
            delivery_ref: "delivery:clock-test".into(),
            expected_binding_revision: "revision:clock-test".into(),
            total: FactoryTotalBudgetBasis {
                original_attempt_ref: "attempt:clock-test".into(),
                run_ref: "run:clock-test".into(),
                unit_ref: "unit:clock-test".into(),
                execution_ref: "execution:clock-test".into(),
                budget_source_ref: "source:clock-test".into(),
                budget_source_revision: "revision:clock-test".into(),
                budget_source_digest: "digest:clock-test".into(),
                intent_ref: "intent:clock-test".into(),
                dispatch_started_unix_ms: 1,
                total_duration_ms: duration,
                total_deadline_unix_ms: 1 + duration,
            },
            remaining_ms: duration,
            remaining_observation_ref: "observation:clock-test".into(),
            required_stop: NativeTurnStopRequirement::CooperativeTurn,
        }
    }

    #[test]
    fn actual_native_clock_elapsed_time_is_not_replenished_and_restart_refuses() {
        // This exercises a real monotonic/wall clock, not a Factory/native
        // provider proof. Opaque pure inputs do not assert semantic authority.
        let clock = NativeTurnClock::new("owner:first".into()).unwrap();
        let digest = format!("blake3-v1:{}", "0".repeat(64));
        let admission = clock.admit(&restriction(10), digest).unwrap();
        assert!(!admission.factory_total_verified);
        std::thread::sleep(std::time::Duration::from_millis(15));
        assert!(clock.remaining(&admission).unwrap().is_zero());
        let restarted = NativeTurnClock::new("owner:restarted".into()).unwrap();
        assert!(restarted.remaining(&admission).is_err());
    }

    #[test]
    fn strict_native_bound_wire_and_canonical_basis_preserve_every_condition() {
        let bound = restriction(10);
        let mut value = serde_json::to_value(&bound).unwrap();
        value["caller_principal"] = serde_json::json!("human");
        assert!(serde_json::from_value::<NativeTurnBound>(value).is_err());
        assert!(serde_json::from_value::<NativeTurnBound>(serde_json::Value::Null).is_err());
        let expected = br#"["actuation.native-turn-bound/v1","agent:clock-test","agency:clock-test","binding:clock-test","task:clock-test","delivery:clock-test","revision:clock-test",["attempt:clock-test","run:clock-test","unit:clock-test","execution:clock-test","source:clock-test","revision:clock-test","digest:clock-test","intent:clock-test",1,10,11],10,"observation:clock-test","cooperative-turn"]"#;
        assert_eq!(bound.canonical_bytes().unwrap(), expected);
        let mut narrower = bound.clone();
        narrower.remaining_ms = 9;
        assert_ne!(
            bound.canonical_bytes().unwrap(),
            narrower.canonical_bytes().unwrap()
        );
        let mut changed_source = bound.clone();
        changed_source.total.budget_source_revision = "another revision".into();
        assert_ne!(
            bound.canonical_bytes().unwrap(),
            changed_source.canonical_bytes().unwrap()
        );
        let round_trip: NativeTurnBound =
            serde_json::from_slice(&serde_json::to_vec(&bound).unwrap()).unwrap();
        assert_eq!(round_trip, bound);
    }

    #[test]
    fn pure_restriction_cannot_grant_shared_process_retirement_or_unknown_clock() {
        let clock = NativeTurnClock::new("owner:clock".into()).unwrap();
        let digest = format!("blake3-v1:{}", "0".repeat(64));
        let mut bound = restriction(10);
        bound.required_stop = NativeTurnStopRequirement::OwnedProcess;
        assert!(clock.admit(&bound, digest.clone()).is_err());
        bound.required_stop = NativeTurnStopRequirement::CooperativeTurn;
        bound.total.total_deadline_unix_ms += 1;
        assert!(clock.admit(&bound, digest).is_err());
    }
}

impl NativeTurnDeadline {
    pub fn new(selector: NativeTurnSelector, admission: NativeTurnAdmission) -> Result<Self> {
        if [
            &selector.agent_session,
            &selector.native_session_id,
            &selector.connection_generation,
            &selector.delivery_ref,
            &selector.prompt_token,
        ]
        .iter()
        .any(|s| s.trim().is_empty() || s.len() > 1024)
        {
            return Err(Error::new("native turn selector is incomplete"));
        }
        validate_condition_digest(&admission.condition_digest)?;
        Ok(Self {
            selector,
            admission,
            requested: false,
        })
    }

    pub fn step(
        &mut self,
        clock: &NativeTurnClock,
        port: &dyn NativeTurnStopPort,
    ) -> Result<NativeTurnObservation> {
        let observed = port.observe_exact(&self.selector)?;
        if observed.selector != self.selector || observed.process_retirement_observed {
            return Err(Error::new(
                "turn port returned an inconsistent ownership observation",
            ));
        }
        if !matches!(observed.state, NativeTurnObservedState::Active) || self.requested {
            return Ok(observed);
        }
        if !clock.remaining(&self.admission)?.is_zero() {
            return Ok(observed);
        }
        // Retain the once-only request fact before a possibly uncertain write.
        // An error is not permission to resend; actual state remains observable.
        self.requested = true;
        let requested =
            port.request_cancel_exact(&self.selector, &self.admission.condition_digest)?;
        if requested.selector != self.selector || requested.process_retirement_observed {
            return Err(Error::new(
                "turn port returned an inconsistent cancellation observation",
            ));
        }
        Ok(requested)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DrivingMode {
    Managed,
    Discovered,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocusPhase {
    Unobserved,
    Acting,
    Waiting,
    Interrupted,
    Cancelled,
    Terminated,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocusReport {
    pub realised: RealisedActuation,
    pub phase: LocusPhase,
}
/// A refusal to pretend is an ordinary portable outcome, not a fabricated
/// successful cancellation, trace, Return or observed body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DriverOutcome<T> {
    Observed {
        value: T,
        evidence_refs: NonEmpty<ExternalRef>,
    },
    Unsupported {
        reason: String,
    },
}
impl<T> DriverOutcome<T> {
    pub fn unsupported(reason: impl Into<String>) -> Self {
        Self::Unsupported {
            reason: reason.into(),
        }
    }
}
/// A target may implement only observation. Default lifecycle faculties remain
/// explicitly unsupported. This port does not allocate or resolve any body.
pub trait LocusDriver: Send {
    fn observe<'a>(
        &'a mut self,
        binding: &'a WorldBinding,
    ) -> PortFuture<'a, DriverOutcome<LocusReport>>;
    fn begin<'a>(
        &'a mut self,
        _binding: &'a WorldBinding,
    ) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        Box::pin(async {
            Ok(DriverOutcome::unsupported(
                "begin is unsupported by this target",
            ))
        })
    }
    fn resume<'a>(
        &'a mut self,
        _binding: &'a WorldBinding,
    ) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        Box::pin(async {
            Ok(DriverOutcome::unsupported(
                "resume is unsupported by this target",
            ))
        })
    }
    fn interrupt<'a>(
        &'a mut self,
        _binding: &'a WorldBinding,
    ) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        Box::pin(async {
            Ok(DriverOutcome::unsupported(
                "interrupt is unsupported by this target",
            ))
        })
    }
    fn cancel<'a>(
        &'a mut self,
        _binding: &'a WorldBinding,
    ) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        Box::pin(async {
            Ok(DriverOutcome::unsupported(
                "cancel is unsupported by this target",
            ))
        })
    }
    fn terminate<'a>(
        &'a mut self,
        _binding: &'a WorldBinding,
    ) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        Box::pin(async {
            Ok(DriverOutcome::unsupported(
                "terminate is unsupported by this target",
            ))
        })
    }
    fn collect_return<'a>(
        &'a mut self,
        _binding: &'a WorldBinding,
    ) -> PortFuture<'a, DriverOutcome<Return>> {
        Box::pin(async {
            Ok(DriverOutcome::unsupported(
                "no attributable Return has been supplied",
            ))
        })
    }
}
/// Managed/discovered are modes of encounter with the same actual agency.
/// Neither mode mints identity, authority or provider-specific capabilities.
pub struct Actuator<D> {
    binding: WorldBinding,
    mode: DrivingMode,
    driver: D,
    current: Option<LocusReport>,
}
impl<D: LocusDriver> Actuator<D> {
    pub fn new(binding: WorldBinding, mode: DrivingMode, driver: D) -> Self {
        Self {
            binding,
            mode,
            driver,
            current: None,
        }
    }
    pub fn binding(&self) -> &WorldBinding {
        &self.binding
    }
    pub fn current(&self) -> Option<&LocusReport> {
        self.current.as_ref()
    }
    pub fn phase(&self) -> LocusPhase {
        self.current
            .as_ref()
            .map(|r| r.phase)
            .unwrap_or(LocusPhase::Unobserved)
    }
    pub fn mode(&self) -> DrivingMode {
        self.mode
    }
    pub fn driver(&self) -> &D {
        &self.driver
    }
    fn record(
        &mut self,
        outcome: DriverOutcome<LocusReport>,
    ) -> Result<DriverOutcome<LocusReport>> {
        if let DriverOutcome::Observed { value, .. } = &outcome {
            if !value.realised.belongs_to(&self.binding) {
                return Err(Error::new(
                    "driver observation cannot replace Agent, Agency or WorldBinding identity",
                ));
            }
            if value.phase == LocusPhase::Unobserved
                || value.realised.fields().observation.fields().state
                    == ObservationState::Unavailable
            {
                return Err(Error::new(
                    "unavailable or unobserved actuality cannot be promoted by a driver envelope",
                ));
            }
            if let Some(current) = &self.current {
                if !current
                    .realised
                    .continuity_to(&value.realised)
                    .same_actuation
                {
                    return Err(Error::new(
                        "continuation cannot silently replace actuation identity",
                    ));
                }
                if matches!(
                    current.phase,
                    LocusPhase::Cancelled | LocusPhase::Terminated
                ) && !matches!(value.phase, LocusPhase::Cancelled | LocusPhase::Terminated)
                {
                    return Err(Error::new("terminal actuation cannot silently resume"));
                }
            }
            self.current = Some(value.clone());
        }
        Ok(outcome)
    }
    pub async fn observe(&mut self) -> Result<DriverOutcome<LocusReport>> {
        let outcome = self.driver.observe(&self.binding).await?;
        self.record(outcome)
    }
    pub async fn begin(&mut self) -> Result<DriverOutcome<LocusReport>> {
        if self.mode == DrivingMode::Discovered {
            return Ok(DriverOutcome::unsupported(
                "an already-existing body is observed, not implicitly launched",
            ));
        }
        if self.current.is_some() {
            return Err(Error::new("an observed locus cannot be begun again"));
        }
        let outcome = self.driver.begin(&self.binding).await?;
        self.record(outcome)
    }
    pub async fn resume(&mut self) -> Result<DriverOutcome<LocusReport>> {
        if !matches!(self.phase(), LocusPhase::Waiting | LocusPhase::Interrupted) {
            return Err(Error::new("only a waiting or interrupted locus may resume"));
        }
        let outcome = self.driver.resume(&self.binding).await?;
        self.record(outcome)
    }
    fn require_active(&self) -> Result<()> {
        if matches!(
            self.phase(),
            LocusPhase::Acting | LocusPhase::Waiting | LocusPhase::Interrupted
        ) {
            Ok(())
        } else {
            Err(Error::new(
                "lifecycle control requires observed nonterminal actuality",
            ))
        }
    }
    pub async fn interrupt(&mut self) -> Result<DriverOutcome<LocusReport>> {
        self.require_active()?;
        let outcome = self.driver.interrupt(&self.binding).await?;
        if matches!(&outcome, DriverOutcome::Observed { value, .. } if value.phase != LocusPhase::Interrupted)
        {
            return Err(Error::new(
                "interrupt requires an observed interrupted state",
            ));
        }
        self.record(outcome)
    }
    pub async fn cancel(&mut self) -> Result<DriverOutcome<LocusReport>> {
        self.require_active()?;
        let outcome = self.driver.cancel(&self.binding).await?;
        if matches!(&outcome, DriverOutcome::Observed { value, .. } if value.phase != LocusPhase::Cancelled)
        {
            return Err(Error::new("cancel requires an observed cancelled state"));
        }
        self.record(outcome)
    }
    pub async fn terminate(&mut self) -> Result<DriverOutcome<LocusReport>> {
        self.require_active()?;
        let outcome = self.driver.terminate(&self.binding).await?;
        if matches!(&outcome, DriverOutcome::Observed { value, .. } if value.phase != LocusPhase::Terminated)
        {
            return Err(Error::new(
                "terminate requires an observed terminated state",
            ));
        }
        self.record(outcome)
    }
    pub async fn collect_return(&mut self) -> Result<DriverOutcome<Return>> {
        if self.current.is_none() {
            return Err(Error::new(
                "no observed locus from which to collect a Return",
            ));
        }
        let outcome = self.driver.collect_return(&self.binding).await?;
        if let DriverOutcome::Observed { value, .. } = &outcome {
            let r = value.fields();
            if r.from_agency_ref != self.binding.fields().agency_ref
                || !r
                    .provenance
                    .fields()
                    .agency_lineage_refs
                    .as_slice()
                    .contains(&self.binding.fields().agency_ref)
            {
                return Err(Error::new("Return must retain observed Agency provenance"));
            }
            let provenance = r.provenance.fields();
            let current = self.current.as_ref().expect("observation checked");
            let mismatch = provenance.agent_refs.value().is_some_and(|refs| {
                !refs.is_empty() && !refs.contains(&self.binding.fields().agent_ref)
            }) || provenance.world_binding_refs.value().is_some_and(|refs| {
                !refs.is_empty() && !refs.contains(&self.binding.fields().binding_ref)
            }) || provenance.actuation_refs.value().is_some_and(|refs| {
                !refs.is_empty() && !refs.contains(&current.realised.fields().actuation_ref)
            }) || current
                .realised
                .fields()
                .return_ref
                .value()
                .is_some_and(|reference| reference != &r.return_ref);
            if mismatch {
                return Err(Error::new(
                    "supplied Return correlations contradict the observed locus",
                ));
            }
            if r.received
                || r.recognition_state != RecognitionState::Pending
                || r.world_mutation_state != MutationState::NotApplied
            {
                return Err(Error::new(
                    "collecting a new Return cannot decide reception, Recognition or mutation",
                ));
            }
            if let Some(parent) = self.binding.fields().determining_agency_ref.value() {
                if &r.to_agency_ref != parent {
                    return Err(Error::new(
                        "Return cannot be redirected away from its determining Agency",
                    ));
                }
            }
        }
        Ok(outcome)
    }
}

/// A useful native managed driver for an already supplied acting host. No
/// harness install, provider selection, session hosting, or material allocation
/// happens here. A project directory can remain the caller's World reference.
pub struct LoopLocusDriver<H, O> {
    host: H,
    observer: O,
    runtime: Box<dyn LoopRuntime>,
    request: LoopRequest,
    realised_ref: RealisedRef,
    actuation_ref: ActuationRef,
    body: Slot<BodyRelation>,
    cancellation: CancellationToken,
    execution: Option<LoopExecution>,
    observed: Option<DriverOutcome<LocusReport>>,
}
impl<H: RuntimeHost, O: RuntimeObserver> LoopLocusDriver<H, O> {
    pub fn new(
        host: H,
        observer: O,
        request: LoopRequest,
        realised_ref: RealisedRef,
        actuation_ref: ActuationRef,
    ) -> Self {
        Self {
            host,
            observer,
            runtime: Box::new(ClassicRuntime),
            request,
            realised_ref,
            actuation_ref,
            body: Slot::Absent,
            cancellation: CancellationToken::default(),
            execution: None,
            observed: None,
        }
    }
    pub fn with_runtime(mut self, runtime: Box<dyn LoopRuntime>) -> Self {
        self.runtime = runtime;
        self
    }
    pub fn with_body(mut self, body: BodyRelation) -> Self {
        self.body = Slot::Value(body);
        self
    }
    pub fn cancellation(&self) -> CancellationToken {
        self.cancellation.clone()
    }
    pub fn execution(&self) -> Option<&LoopExecution> {
        self.execution.as_ref()
    }
    pub fn observer(&self) -> &O {
        &self.observer
    }
    async fn run(&mut self, binding: &WorldBinding) -> Result<DriverOutcome<LocusReport>> {
        if self.execution.is_some() {
            return Err(Error::new(
                "completed bounded execution cannot be silently re-run",
            ));
        }
        let execution = self
            .runtime
            .run(
                &self.request,
                &mut self.host,
                &mut self.observer,
                &self.cancellation,
            )
            .await?;
        let witness = execution.evidence_refs.clone();
        let model_calls = execution.report.model_calls;
        let phase = match execution.report.status {
            LoopStatus::Cancelled => LocusPhase::Cancelled,
            _ => LocusPhase::Terminated,
        };
        self.execution = Some(execution);
        if model_calls == 0 {
            return Ok(DriverOutcome::unsupported("execution has no witnessed model return; acting actuality cannot be inferred from availability"));
        }
        let evidence = NonEmpty::new(witness)?;
        let realised = RealisedActuation::new(RealisedActuationFields {
            schema: RealisedSchema::V1,
            realised_ref: self.realised_ref.clone(),
            actuation_ref: self.actuation_ref.clone(),
            agent_ref: binding.fields().agent_ref.clone(),
            agency_ref: binding.fields().agency_ref.clone(),
            world_binding_ref: binding.fields().binding_ref.clone(),
            loop_facts: LoopFacts::new(LoopFactsFields {
                recurrence: Recurrence::TurnBased,
                acting: true,
                entrypoint_ref: Slot::Absent,
                observed_faculties: Slot::Value(vec![ExternalRef::new("model-call")?]),
                extensions: Extensions::new(),
            })?,
            body: self.body.clone(),
            participating_loci: Slot::Absent,
            stream_ref: Slot::Absent,
            return_ref: Slot::Absent,
            observation: Observation::new(ObservationFields {
                state: ObservationState::Observed,
                evidence_refs: Slot::Value(evidence.as_slice().to_vec()),
                unsupported_faculties: Slot::Value(vec![
                    ExternalRef::new("native-process-interrupt")?,
                    ExternalRef::new("native-process-cancel")?,
                    ExternalRef::new("native-process-terminate")?,
                ]),
                degraded_faculties: Slot::Absent,
                extensions: Extensions::new(),
            })?,
            lifecycle: Slot::Absent,
            extensions: Extensions::new(),
        })?;
        let outcome = DriverOutcome::Observed {
            value: LocusReport { realised, phase },
            evidence_refs: evidence,
        };
        self.observed = Some(outcome.clone());
        Ok(outcome)
    }
}
impl<H: RuntimeHost, O: RuntimeObserver> LocusDriver for LoopLocusDriver<H, O> {
    fn begin<'a>(
        &'a mut self,
        binding: &'a WorldBinding,
    ) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        Box::pin(async move { self.run(binding).await })
    }
    fn observe<'a>(
        &'a mut self,
        _binding: &'a WorldBinding,
    ) -> PortFuture<'a, DriverOutcome<LocusReport>> {
        Box::pin(async move {
            Ok(self.observed.clone().unwrap_or_else(|| {
                DriverOutcome::unsupported("no managed-loop actuality has yet been observed")
            }))
        })
    }
}
