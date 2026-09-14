//! Gateway admission and invocation authority.
//!
//! The gateway never infers authority from presence. A subject may attach to a
//! stream only through an explicit grant, and a controller Agency may invoke
//! another Agency only through an explicit invocation grant naming the mode.
//! Everything not granted is refused.

use crate::wire::POLICY_SCHEMA;
use actuation_core::{AgencyRef, AgentRef, Error, ExternalRef, LocusRef, Result, StreamRef};
use serde::Deserialize;

/// A granted Surface attachment: what this subject may encounter, and the
/// exact attribution the gateway will stamp on its behalf. A connector subject
/// speaks for an external participant; an agent subject speaks as an
/// attributed locus of the granted Agency.
#[derive(Clone, Debug, Deserialize)]
pub struct AttachGrant {
    pub subject: String,
    pub stream_ref: StreamRef,
    pub role: GrantRole,
    pub surface_ref: Option<ExternalRef>,
    pub participant_ref: Option<ExternalRef>,
    pub agency_ref: Option<AgencyRef>,
    pub agent_ref: Option<AgentRef>,
    pub locus_ref: Option<LocusRef>,
    #[serde(default)]
    pub may_invoke: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GrantRole {
    Connector,
    Agent,
}

impl AttachGrant {
    pub fn actor_is_supplied(&self) -> bool {
        self.agency_ref.is_some() || self.agent_ref.is_some() || self.locus_ref.is_some()
    }
}

/// One co-internal invocation relation. Modes are qualitative and explicit:
/// a one-shot communique, a deliberate contribution to an existing session,
/// or a delegation that expects a Return. Absence of a grant is refusal.
#[derive(Clone, Debug, Deserialize)]
pub struct InvokeGrant {
    pub controller_agency_ref: AgencyRef,
    pub target_agency_ref: AgencyRef,
    pub modes: Vec<InvocationMode>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InvocationMode {
    Communique,
    SessionContribution,
    Delegation,
}
impl InvocationMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Communique => "communique",
            Self::SessionContribution => "session-contribution",
            Self::Delegation => "delegation",
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct GatewayPolicy {
    #[serde(default)]
    pub attach: Vec<AttachGrant>,
    #[serde(default)]
    pub invoke: Vec<InvokeGrant>,
}

impl GatewayPolicy {
    pub fn admitted(schema: &str) -> Result<()> {
        if schema == POLICY_SCHEMA {
            Ok(())
        } else {
            Err(Error::new(format!(
                "expected gateway policy schema {POLICY_SCHEMA}, found {schema}"
            )))
        }
    }

    pub fn attach_grant(&self, subject: &str, stream_ref: &StreamRef) -> Option<&AttachGrant> {
        self.attach
            .iter()
            .find(|grant| grant.subject == subject && &grant.stream_ref == stream_ref)
    }

    pub fn agent_grant(&self, subject: &str, stream_ref: &StreamRef) -> Result<&AttachGrant> {
        let grant = self.attach_grant(subject, stream_ref).ok_or_else(|| {
            Error::new(format!(
                "subject {subject} is not granted attach on {stream_ref}"
            ))
        })?;
        if grant.role != GrantRole::Agent {
            return Err(Error::new(format!(
                "subject {subject} is not granted an agent locus on {stream_ref}"
            )));
        }
        if !grant.actor_is_supplied() {
            return Err(Error::new(format!(
                "agent grant for {subject} must name the attributed Agency, Agent or locus"
            )));
        }
        Ok(grant)
    }

    pub fn invocation(
        &self,
        controller: &AgencyRef,
        target: &AgencyRef,
        mode: InvocationMode,
    ) -> Result<()> {
        let granted = self.invoke.iter().find(|grant| {
            &grant.controller_agency_ref == controller && &grant.target_agency_ref == target
        });
        match granted {
            Some(grant) if grant.modes.contains(&mode) => Ok(()),
            Some(_) => Err(Error::new(format!(
                "invocation mode {} from {controller} to {target} is not granted",
                mode.as_str()
            ))),
            None => Err(Error::new(format!(
                "invocation from {controller} to {target} is not granted"
            ))),
        }
    }

    /// Readable streams for one subject: only what its own grants name.
    pub fn visible_streams(&self, subject: &str) -> Vec<StreamRef> {
        self.attach
            .iter()
            .filter(|grant| grant.subject == subject)
            .map(|grant| grant.stream_ref.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(value: serde_json::Value) -> GatewayPolicy {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn grants_are_exact_and_absence_denies() {
        let policy = policy(serde_json::json!({
            "schema": POLICY_SCHEMA,
            "attach": [{"subject":"connector:cli","role":"connector","stream_ref":"stream:one",
                        "surface_ref":"surface:cli","participant_ref":"participant:cli"}],
            "invoke": [{"controller_agency_ref":"agency:ctl","target_agency_ref":"agency:worker",
                        "modes":["delegation"]}]
        }));
        GatewayPolicy::admitted(POLICY_SCHEMA).unwrap();
        let one = StreamRef::new("stream:one").unwrap();
        assert!(policy.attach_grant("connector:cli", &one).is_some());
        assert!(policy.attach_grant("connector:other", &one).is_none());
        assert!(policy
            .attach_grant("connector:cli", &StreamRef::new("stream:two").unwrap())
            .is_none());
        // An exact delegation grant does not widen to other modes or agencies.
        policy
            .invocation(
                &AgencyRef::new("agency:ctl").unwrap(),
                &AgencyRef::new("agency:worker").unwrap(),
                InvocationMode::Delegation,
            )
            .unwrap();
        assert!(policy
            .invocation(
                &AgencyRef::new("agency:ctl").unwrap(),
                &AgencyRef::new("agency:worker").unwrap(),
                InvocationMode::Communique,
            )
            .is_err());
        assert!(policy
            .invocation(
                &AgencyRef::new("agency:ctl").unwrap(),
                &AgencyRef::new("agency:intruder").unwrap(),
                InvocationMode::Delegation,
            )
            .is_err());
        assert!(policy.visible_streams("connector:cli") == vec![one]);
        assert!(policy.visible_streams("nobody").is_empty());
    }

    #[test]
    fn agent_grants_require_a_supplied_locus() {
        let policy = policy(serde_json::json!({
            "attach": [
                {"subject":"agent:bare","role":"agent","stream_ref":"stream:one"},
                {"subject":"agent:full","role":"agent","stream_ref":"stream:one",
                 "agency_ref":"agency:worker","agent_ref":"agent:worker","locus_ref":"locus:worker"}
            ]
        }));
        let one = StreamRef::new("stream:one").unwrap();
        assert!(policy.agent_grant("agent:bare", &one).is_err());
        assert!(policy.agent_grant("agent:full", &one).is_ok());
        assert!(policy.agent_grant("nobody", &one).is_err());
    }

    #[test]
    fn policy_schema_is_checked_not_assumed() {
        assert!(GatewayPolicy::admitted("actuation.gateway-policy/v2").is_err());
    }
}
