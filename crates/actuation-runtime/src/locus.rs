use crate::*;
use actuation_core::*;
use serde::{Deserialize, Serialize};

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
