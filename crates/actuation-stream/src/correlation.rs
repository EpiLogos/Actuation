use crate::{domain, Timestamp};
use crate::{Activity, ActuationStream, EventKind, StreamEvent};
use actuation_core::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashSet;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CorrelationSchema {
    #[serde(rename = "actuation.request-correlation/v1")]
    V1,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AuthorityDecisionDocument {
    #[serde(rename = "authority-decision")]
    AuthorityDecision,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AuthorityResolution {
    #[serde(rename = "allowed")]
    Allowed,
    #[serde(rename = "refused")]
    Refused,
    #[serde(rename = "pending")]
    Pending,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CorrelationState {
    #[serde(rename = "correlation-unavailable")]
    Unavailable,
    #[serde(rename = "unknown-identity")]
    UnknownIdentity,
    #[serde(rename = "no-recorded-activity")]
    NoRecordedActivity,
    #[serde(rename = "correlated")]
    Correlated,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuthorityDecisionFields {
    pub schema: CorrelationSchema,
    pub document: AuthorityDecisionDocument,
    pub decision_ref: ExternalRef,
    pub request_ref: RequestRef,
    pub decision: AuthorityResolution,
    pub basis: ExternalRef,
    pub determination_ref: DeterminationRef,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub bounds_refs: Slot<Vec<BoundsRef>>,
    pub decided_by: ExternalRef,
    pub observed_at: Timestamp,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub evidence_refs: Slot<Vec<ExternalRef>>,
    pub activity_refs: NonEmpty<ActivityRef>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    AuthorityDecision,
    AuthorityDecisionFields,
    [
        schema,
        document,
        decision_ref,
        request_ref,
        decision,
        basis,
        determination_ref,
        bounds_refs,
        decided_by,
        observed_at,
        evidence_refs,
        activity_refs
    ],
    |_: &Self| Ok(())
);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CorrelationCorpusFields {
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub authority_decisions: Slot<Vec<AuthorityDecision>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub activities: Slot<Vec<Activity>>,
    #[serde(default, skip_serializing_if = "Slot::is_absent")]
    pub streams: Slot<Vec<ActuationStream>>,
    #[serde(flatten)]
    pub extensions: Extensions,
}
domain!(
    CorrelationCorpus,
    CorrelationCorpusFields,
    [authority_decisions, activities, streams],
    |v: &Self| {
        if matches!(v.authority_decisions, Slot::Null)
            || matches!(v.activities, Slot::Null)
            || matches!(v.streams, Slot::Null)
        {
            Err(Error::new(
                "a supplied corpus side must be an array, not null",
            ))
        } else {
            Ok(())
        }
    }
);

fn unavailable() -> Value {
    json!({"available":false,"unavailable_reason":"corpus not supplied for this side; absence cannot be claimed"})
}
fn disposition(event: &StreamEvent) -> &'static str {
    if event.fields().kind == EventKind::Refusal {
        return "refused";
    }
    match event
        .fields()
        .metadata
        .value()
        .and_then(|m| m.get("permission_outcome"))
        .and_then(Value::as_str)
    {
        Some("granted") => "granted",
        Some("refused") => "refused",
        _ => "pending",
    }
}
fn event_order_key(stream_ref: &StreamRef, event: &StreamEvent) -> Vec<u16> {
    // Preserve the existing lexical key (including UTF-16 ordering), rather
    // than replacing it with inferred provider chronology.
    format!(
        "{}{}{:016}",
        event
            .fields()
            .observed_at
            .value()
            .map_or("", Timestamp::as_str),
        stream_ref,
        event.fields().sequence.get()
    )
    .encode_utf16()
    .collect()
}
impl CorrelationCorpus {
    pub fn unqueried() -> Self {
        Self::new(CorrelationCorpusFields {
            authority_decisions: Slot::Absent,
            activities: Slot::Absent,
            streams: Slot::Absent,
            extensions: Extensions::new(),
        })
        .expect("empty unqueried corpus is valid")
    }
    /// Read-only four-side join. Request/Activity identities are exact opaque
    /// equality keys; provider/process/World proximity cannot substitute.
    pub fn correlate(&self, request: &RequestRef) -> Value {
        let c = self.fields();
        let authority_available = c.authority_decisions.value().is_some();
        let activity_available = c.activities.value().is_some();
        let stream_available = c.streams.value().is_some();
        let decisions: Vec<_> = c
            .authority_decisions
            .value()
            .into_iter()
            .flatten()
            .filter(|d| &d.fields().request_ref == request)
            .collect();
        let mut ordered = decisions.clone();
        ordered.sort_by_key(|d| d.fields().observed_at.unix_nanos());
        let mut seen = HashSet::new();
        let named: Vec<_> = decisions
            .iter()
            .flat_map(|d| d.fields().activity_refs.as_slice())
            .filter(|r| seen.insert((*r).clone()))
            .cloned()
            .collect();
        let activities: Vec<_> = c
            .activities
            .value()
            .into_iter()
            .flatten()
            .filter(|a| named.contains(&a.fields().activity_ref))
            .collect();
        let mut events: Vec<_> = c
            .streams
            .value()
            .into_iter()
            .flatten()
            .flat_map(|s| {
                s.fields()
                    .events
                    .iter()
                    .filter(|e| {
                        matches!(e.fields().kind, EventKind::Permission | EventKind::Refusal)
                            && e.fields().resource_refs.value().is_some_and(|refs| {
                                refs.iter().any(|r| r.as_str() == request.as_str())
                            })
                    })
                    .map(move |e| (&s.fields().stream_ref, e))
            })
            .collect();
        events.sort_by_cached_key(|(s, e)| event_order_key(s, e));
        let referenced = !decisions.is_empty() || !events.is_empty();
        let state = if !authority_available && !activity_available && !stream_available {
            CorrelationState::Unavailable
        } else if !referenced {
            CorrelationState::UnknownIdentity
        } else if !activity_available {
            CorrelationState::Unavailable
        } else if activities.is_empty() {
            CorrelationState::NoRecordedActivity
        } else {
            CorrelationState::Correlated
        };
        let mut authority = if authority_available {
            json!({"available":true})
        } else {
            unavailable()
        };
        authority["decisions"] = crate::wire::json(&ordered);
        authority["decision_refs"] = crate::wire::json(
            &ordered
                .iter()
                .map(|d| &d.fields().decision_ref)
                .collect::<Vec<_>>(),
        );
        authority["resolution"] = ordered
            .last()
            .map(|d| crate::wire::json(&d.fields().decision))
            .unwrap_or(json!("none"));
        let mut activity_side = if activity_available {
            json!({"available":true})
        } else {
            unavailable()
        };
        let keys = [
            "activity_ref",
            "native_owner",
            "verb",
            "object",
            "summary",
            "phase",
            "outcome",
            "salience",
            "needs_attention",
            "started_at",
            "updated_at",
        ];
        activity_side["correlated"] = json!(activities
            .iter()
            .map(|a| {
                let value = crate::wire::json(a);
                let mut brief = serde_json::Map::new();
                for key in keys {
                    brief.insert(key.into(), value[key].clone());
                }
                Value::Object(brief)
            })
            .collect::<Vec<_>>());
        activity_side["activity_refs"] = crate::wire::json(
            &activities
                .iter()
                .map(|a| &a.fields().activity_ref)
                .collect::<Vec<_>>(),
        );
        activity_side["unrecorded_activity_refs"] = if activity_available {
            crate::wire::json(
                &named
                    .iter()
                    .filter(|r| !activities.iter().any(|a| &a.fields().activity_ref == *r))
                    .collect::<Vec<_>>(),
            )
        } else {
            json!([])
        };
        let attention_refs: Vec<_> = activities
            .iter()
            .filter(|a| a.needs_attention())
            .map(|a| &a.fields().activity_ref)
            .collect();
        let after = events
            .iter()
            .rposition(|(_, e)| disposition(e) != "pending")
            .map_or(0, |i| i + 1);
        let pending: Vec<_> = events
            .iter()
            .skip(after)
            .filter(|(_, e)| {
                e.fields().kind == EventKind::Permission && disposition(e) == "pending"
            })
            .map(|(_, e)| &e.fields().event_ref)
            .collect();
        let mut attention = json!({"available":activity_available && stream_available,"tracked":!attention_refs.is_empty()||!pending.is_empty(),"activity_refs":attention_refs,"pending_permission_event_refs":pending});
        if !(activity_available && stream_available) {
            attention["unavailable_reason"] = json!(
                "attention context derives from activities and streams; a missing side limits it"
            );
        }
        let mut permission = if stream_available {
            json!({"available":true})
        } else {
            unavailable()
        };
        permission["outcome"] = json!(events.last().map_or("none", |(_, e)| disposition(e)));
        permission["event_refs"] = crate::wire::json(
            &events
                .iter()
                .map(|(_, e)| &e.fields().event_ref)
                .collect::<Vec<_>>(),
        );
        json!({"schema":"actuation.request-correlation/v1","request_ref":request,"state":state,"authority":authority,"activities":activity_side,"attention":attention,"permission":permission})
    }
}
