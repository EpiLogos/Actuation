use crate::{domain, Count, JsonObject, Timestamp};
use actuation_core::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum UsageSchema {
    #[serde(rename = "actuation.model-usage/v1")]
    V1,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EvidenceStanding {
    #[serde(rename = "provider-reported")]
    ProviderReported,
    #[serde(rename = "observed")]
    Observed,
    #[serde(rename = "normalized-from-native")]
    NormalizedFromNative,
    #[serde(rename = "derived")]
    Derived,
    #[serde(rename = "estimated")]
    Estimated,
    #[serde(rename = "unavailable")]
    Unavailable,
    #[serde(rename = "not-reported")]
    NotReported,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum UsageOutcomeState {
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "partial")]
    Partial,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "interrupted")]
    Interrupted,
    #[serde(rename = "unknown")]
    Unknown,
}
impl EvidenceStanding {
    pub fn is_available(self) -> bool {
        !matches!(self, Self::Unavailable | Self::NotReported)
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UsageIdentityFields {
    pub standing: EvidenceStanding,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    #[serde(rename = "ref")]
    pub identity_ref: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub name: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub revision: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub variant: Slot<ExternalRef>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(UsageIdentity, UsageIdentityFields, [standing, ref, name, revision, variant], |v: &Self| {
    let has = v.identity_ref.value().is_some() || v.name.value().is_some();
    let any = has || v.revision.value().is_some() || v.variant.value().is_some();
    if (v.standing.is_available() && !has) || (!v.standing.is_available() && any) {
        Err(Error::new("identity fields and observation standing disagree"))
    } else { Ok(()) }
});

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UsageCorrelationFields {
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub activity_ref: Slot<ActivityRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub agent_ref: Slot<AgentRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub agency_ref: Slot<AgencyRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub agent_session_ref: Slot<AgentSessionRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub harness_ref: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub body_ref: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub native_session_ref: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub external_refs: Slot<Vec<ExternalRef>>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    UsageCorrelation,
    UsageCorrelationFields,
    [
        activity_ref,
        agent_ref,
        agency_ref,
        agent_session_ref,
        harness_ref,
        body_ref,
        native_session_ref,
        external_refs
    ],
    |_: &Self| Ok(())
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TokenUsageFields {
    pub standing: EvidenceStanding,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub input: Slot<Count>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub output: Slot<Count>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    TokenUsage,
    TokenUsageFields,
    [standing, input, output],
    |v: &Self| {
        if !v.standing.is_available() && (v.input.value().is_some() || v.output.value().is_some()) {
            Err(Error::new("unavailable usage must not carry counts"))
        } else {
            Ok(())
        }
    }
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CacheUsageFields {
    pub standing: EvidenceStanding,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub read_input: Slot<Count>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub creation_input: Slot<Count>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub output: Slot<Count>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    CacheUsage,
    CacheUsageFields,
    [standing, read_input, creation_input, output],
    |v: &Self| {
        if !v.standing.is_available()
            && (v.read_input.value().is_some()
                || v.creation_input.value().is_some()
                || v.output.value().is_some())
        {
            Err(Error::new("unavailable usage must not carry counts"))
        } else {
            Ok(())
        }
    }
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UsageClassFields {
    pub class: ExternalRef,
    pub quantity: Count,
    pub unit: ExternalRef,
    pub standing: EvidenceStanding,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    UsageClass,
    UsageClassFields,
    [class, quantity, unit, standing],
    |_: &Self| Ok(())
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LatencyFields {
    pub standing: EvidenceStanding,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub milliseconds: Slot<serde_json::Number>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    Latency,
    LatencyFields,
    [standing, milliseconds],
    |v: &Self| {
        if v.standing.is_available() {
            if !v.milliseconds.value().is_some_and(crate::wire::nonnegative) {
                return Err(Error::new(
                    "reported latency requires non-negative finite milliseconds",
                ));
            }
        } else if v.milliseconds.value().is_some() {
            return Err(Error::new("unknown latency must not carry milliseconds"));
        }
        Ok(())
    }
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UsageTimingFields {
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub started_at: Slot<Timestamp>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub completed_at: Slot<Timestamp>,
    pub latency: Latency,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    UsageTiming,
    UsageTimingFields,
    [started_at, completed_at, latency],
    |v: &Self| {
        if let (Some(start), Some(end)) = (v.started_at.value(), v.completed_at.value()) {
            if end.unix_nanos() < start.unix_nanos() {
                return Err(Error::new("completion must not precede start"));
            }
        }
        Ok(())
    }
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PricingBasisFields {
    pub source_ref: ExternalRef,
    pub revision: ExternalRef,
    pub effective_at: Timestamp,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    PricingBasis,
    PricingBasisFields,
    [source_ref, revision, effective_at],
    |_: &Self| Ok(())
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UsageCostFields {
    pub standing: EvidenceStanding,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub amount: Slot<serde_json::Number>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub currency: Slot<ExternalRef>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub pricing_basis: Slot<PricingBasis>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    UsageCost,
    UsageCostFields,
    [standing, amount, currency, pricing_basis],
    |v: &Self| {
        if !v.standing.is_available() {
            if v.amount.value().is_some()
                || v.currency.value().is_some()
                || v.pricing_basis.value().is_some()
            {
                return Err(Error::new("unknown cost must not carry monetary fields"));
            }
        } else {
            if !v.amount.value().is_some_and(crate::wire::nonnegative)
                || v.currency.value().is_none()
            {
                return Err(Error::new("reported cost requires amount and currency"));
            }
            if (v.standing == EvidenceStanding::Derived) != v.pricing_basis.value().is_some() {
                return Err(Error::new(
                    "only derived cost requires an exact pricing basis",
                ));
            }
        }
        Ok(())
    }
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UsageOutcomeFields {
    pub state: UsageOutcomeState,
    pub standing: EvidenceStanding,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub reason: Slot<ExternalRef>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    UsageOutcome,
    UsageOutcomeFields,
    [state, standing, reason],
    |_: &Self| Ok(())
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UsageProvenanceFields {
    pub reporter_ref: ExternalRef,
    pub native_event_ref: ExternalRef,
    pub native_schema: ExternalRef,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub native_request_ref: Slot<ExternalRef>,
    pub observed_at: Timestamp,
    pub raw_evidence_refs: NonEmpty<ExternalRef>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    UsageProvenance,
    UsageProvenanceFields,
    [
        reporter_ref,
        native_event_ref,
        native_schema,
        native_request_ref,
        observed_at,
        raw_evidence_refs
    ],
    |_: &Self| Ok(())
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ModelUsageObservationFields {
    pub schema: UsageSchema,
    pub usage_ref: ExternalRef,
    pub actuation_ref: ActuationRef,
    pub invocation_ref: InvocationRef,
    pub correlation: UsageCorrelation,
    pub provider: UsageIdentity,
    pub model: UsageIdentity,
    pub tokens: TokenUsage,
    pub cache: CacheUsage,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub usage_classes: Slot<Vec<UsageClass>>,
    pub timing: UsageTiming,
    pub cost: UsageCost,
    pub outcome: UsageOutcome,
    pub provenance: UsageProvenance,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub provider_facts: Slot<JsonObject>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    ModelUsageObservation,
    ModelUsageObservationFields,
    [
        schema,
        usage_ref,
        actuation_ref,
        invocation_ref,
        correlation,
        provider,
        model,
        tokens,
        cache,
        usage_classes,
        timing,
        cost,
        outcome,
        provenance,
        provider_facts
    ],
    |v: &Self| {
        if let Some(facts) = v.provider_facts.value() {
            for (key, value) in facts {
                if !["service_tier", "speed", "inference_geo"].contains(&key.as_str())
                    || value.is_object()
                    || value.is_array()
                {
                    return Err(Error::new(
                        "provider_facts admits only bounded scalar native facts",
                    ));
                }
            }
        }
        Ok(())
    }
);
