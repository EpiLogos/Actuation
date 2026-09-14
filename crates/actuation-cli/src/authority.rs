//! The local governing-authority join (Actuation #84; O:I #220 lane A).
//!
//! `agency actualise` admits a supplied relation, so the missing step was who
//! establishes that relation for a local caller. The owner issues one
//! governing record naming its holder and the worlds it scopes; resolution
//! assembles the actualisation request from that record and the caller's
//! exact request, and refuses without effect at every failed scope relation.
//! A copied example grant or an accepted JSON payload is never evidence of
//! the caller's authority: only the stored record is.

use crate::dispatch::{flag_value, output, positional, read_json_input, Command, Output};
use actuation_core::{
    BoundsRef, Determination, ExternalRef, MetagencyGrant, WorldBinding, WorldRef,
};
use actuation_runtime::{ActualisationProvenance, ActualisationRequest, IdentityEvidence};
use actuation_stream::AuthorityStore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const LOCAL_AUTHORITY_VERSION: &str = "actuation.local-authority/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalAuthoritySchema {
    #[serde(rename = "actuation.local-authority/v1")]
    V1,
}

/// The durable governing side of a local actualisation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoverningAuthorityRecord {
    pub schema: LocalAuthoritySchema,
    pub authority_source_ref: String,
    /// The one requester this record answers for. Nobody else resolves.
    pub holder: ExternalRef,
    pub issued_by: ExternalRef,
    pub governing_binding: WorldBinding,
    pub metagency_grant: MetagencyGrant,
    /// The worlds this authority may actualise into. Empty is refused at
    /// issue: an authority scoped to no world is not an authority.
    pub allowed_world_refs: Vec<WorldRef>,
    pub issued_at_unix_seconds: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_unix_seconds: Option<u64>,
    #[serde(default)]
    pub provenance: Vec<String>,
}

/// What a caller may ask resolution for: everything except the governing
/// side, which only the stored record supplies.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalResolutionRequest {
    pub schema: LocalAuthoritySchema,
    pub resolution_ref: ExternalRef,
    pub request_ref: actuation_core::RequestRef,
    pub requester_ref: ExternalRef,
    pub authority_source_ref: String,
    pub determination_ref: actuation_core::DeterminationRef,
    pub differentiated_binding: WorldBinding,
    pub agent_identity: IdentityEvidence,
    pub requested_bounds_refs: Vec<BoundsRef>,
    pub delegated_autonomy: actuation_core::DelegatedAutonomy,
    pub return_policy: actuation_core::ReturnPolicy,
    pub provenance: ActualisationProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocalResolutionStanding {
    Admitted,
    Refused,
    Unavailable,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthorityRefusal {
    pub code: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalResolutionOutcome {
    pub schema: &'static str,
    pub standing: LocalResolutionStanding,
    pub resolution_ref: String,
    pub authority_source_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<AuthorityRefusal>,
    /// The complete assembled request for `agency actualise`, present only
    /// when the resolution admitted. Nothing here has actualised: admission
    /// remains the semantic gate, and execution the further real one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actualisation_request: Option<Value>,
    pub provenance: Vec<String>,
}

fn refused(
    request: &LocalResolutionRequest,
    code: &str,
    reason: impl std::fmt::Display,
) -> LocalResolutionOutcome {
    LocalResolutionOutcome {
        schema: LOCAL_AUTHORITY_VERSION,
        standing: LocalResolutionStanding::Refused,
        resolution_ref: request.resolution_ref.as_str().to_owned(),
        authority_source_ref: request.authority_source_ref.clone(),
        refusal: Some(AuthorityRefusal {
            code: code.to_owned(),
            reason: reason.to_string(),
        }),
        actualisation_request: None,
        provenance: Vec::new(),
    }
}

fn unavailable(
    request: &LocalResolutionRequest,
    code: &str,
    reason: impl std::fmt::Display,
) -> LocalResolutionOutcome {
    LocalResolutionOutcome {
        schema: LOCAL_AUTHORITY_VERSION,
        standing: LocalResolutionStanding::Unavailable,
        resolution_ref: request.resolution_ref.as_str().to_owned(),
        authority_source_ref: request.authority_source_ref.clone(),
        refusal: Some(AuthorityRefusal {
            code: code.to_owned(),
            reason: reason.to_string(),
        }),
        actualisation_request: None,
        provenance: Vec::new(),
    }
}

fn same_unique_set<T: Ord>(left: &[T], right: &[T]) -> bool {
    let mut left_set: Vec<&T> = left.iter().collect();
    left_set.sort();
    left_set.dedup();
    let mut right_set: Vec<&T> = right.iter().collect();
    right_set.sort();
    right_set.dedup();
    !left.is_empty() && left.len() == right.len() && left_set == right_set
}

/// Resolve the caller's request against the stored governing record.
///
/// Every failed relation refuses without effect. A derived (new) Agent
/// identity additionally needs the actualise-agency operation on the grant.
pub fn resolve_local_authority(
    request: &LocalResolutionRequest,
    store: &AuthorityStore,
    now_unix_seconds: u64,
) -> LocalResolutionOutcome {
    let state = match store.read(&request.authority_source_ref) {
        Ok(Some(state)) => state,
        Ok(None) => {
            return unavailable(
                request,
                "authority.unknown_source",
                format!(
                    "no governing authority record exists for {}",
                    request.authority_source_ref
                ),
            )
        }
        Err(error) => return unavailable(request, "authority.source_unavailable", error),
    };
    let record: GoverningAuthorityRecord = match serde_json::from_value(state.record.clone()) {
        Ok(record) => record,
        Err(error) => {
            return unavailable(
                request,
                "authority.source_invalid",
                format!("stored authority record is not a valid v1 record: {error}"),
            )
        }
    };
    if let Some(revocation) = &state.revoked {
        return refused(
            request,
            "authority.revoked",
            format!(
                "governing authority was revoked at {} ({})",
                revocation.at_unix_seconds, revocation.reason
            ),
        );
    }
    if record
        .expires_at_unix_seconds
        .is_some_and(|expires| expires <= now_unix_seconds)
    {
        return refused(
            request,
            "authority.expired",
            format!(
                "governing authority expired at {}",
                record.expires_at_unix_seconds.unwrap_or_default()
            ),
        );
    }
    if request.requester_ref != record.holder {
        return refused(
            request,
            "authority.wrong_holder",
            format!(
                "{} is not the holder of {}; only {} resolves it",
                request.requester_ref, request.authority_source_ref, record.holder
            ),
        );
    }
    if record.allowed_world_refs.is_empty()
        || !record
            .allowed_world_refs
            .contains(&request.differentiated_binding.fields().world_ref)
    {
        return refused(
            request,
            "authority.world_not_scoped",
            format!(
                "world {} is not in the authority's allowed worlds",
                request.differentiated_binding.fields().world_ref
            ),
        );
    }
    let grant = record.metagency_grant.fields();
    use actuation_core::MetagencyOperation::*;
    let needs = match request.agent_identity.standing {
        actuation_runtime::IdentityStanding::Actualised => vec![DetermineAgency, ActualiseAgency],
        actuation_runtime::IdentityStanding::Existing => vec![DetermineAgency],
    };
    for operation in &needs {
        if !grant.operations.as_slice().contains(operation) {
            return refused(
                request,
                "authority.operation_not_granted",
                format!("grant does not authorise {operation:?}"),
            );
        }
    }
    let grant_bounds: &[BoundsRef] = grant.bounds_refs.value().map(Vec::as_slice).unwrap_or(&[]);
    if request.requested_bounds_refs.is_empty()
        || request
            .requested_bounds_refs
            .iter()
            .any(|bound| !grant_bounds.contains(bound))
    {
        return refused(
            request,
            "authority.bounds_exceeded",
            "requested bounds are empty or exceed the grant bounds",
        );
    }
    let child = request.differentiated_binding.fields();
    if !same_unique_set(
        child.bounds_refs.value().map(Vec::as_slice).unwrap_or(&[]),
        &request.requested_bounds_refs,
    ) {
        return refused(
            request,
            "authority.bounds_mismatch",
            "the differentiated binding's bounds must be exactly the requested bounds",
        );
    }
    let derivation =
        request.agent_identity.standing == actuation_runtime::IdentityStanding::Actualised;
    let determination = match serde_json::from_value::<Determination>(json!({
        "schema": "actuation.agency/v1",
        "determination_ref": request.determination_ref,
        "kind": if derivation { "derivation" } else { "delegation" },
        "determining_agency_ref": grant.agency_ref,
        "differentiated_agency_ref": child.agency_ref,
        "world_binding_ref": child.binding_ref,
        "bounds_refs": request.requested_bounds_refs,
        "delegated_autonomy": request.delegated_autonomy,
        "return_policy": request.return_policy,
        "authority_refs": child.authority_refs,
    })) {
        Ok(determination) => determination,
        Err(error) => {
            return refused(
                request,
                "authority.determination_invalid",
                format!("the resolved determination is not valid: {error}"),
            )
        }
    };
    let governing_binding = match serde_json::to_value(&record.governing_binding) {
        Ok(value) => value,
        Err(error) => {
            return unavailable(
                request,
                "authority.source_unavailable",
                format!("stored governing binding could not be re-encoded: {error}"),
            )
        }
    };
    let metagency_grant = match serde_json::to_value(&record.metagency_grant) {
        Ok(value) => value,
        Err(error) => {
            return unavailable(
                request,
                "authority.source_unavailable",
                format!("stored governing grant could not be re-encoded: {error}"),
            )
        }
    };
    let assembled = match serde_json::from_value::<ActualisationRequest>(json!({
        "schema": "actuation.agency-actualisation/v1",
        "request_ref": request.request_ref,
        "requester_ref": request.requester_ref,
        "governing_binding": governing_binding,
        "metagency_grant": metagency_grant,
        "determination": determination,
        "differentiated_binding": request.differentiated_binding,
        "agent_identity": request.agent_identity,
        "provenance": request.provenance,
    })) {
        Ok(assembled) => assembled,
        Err(error) => {
            return refused(
                request,
                "authority.request_invalid",
                format!("the assembled actualisation request is not valid: {error}"),
            )
        }
    };
    if let Err(error) = assembled.admit() {
        return refused(
            request,
            "authority.admission_refused",
            format!("the assembled relation fails constitutional admission: {error}"),
        );
    }
    let request_value = serde_json::to_value(&assembled).unwrap_or(Value::Null);
    LocalResolutionOutcome {
        schema: LOCAL_AUTHORITY_VERSION,
        standing: LocalResolutionStanding::Admitted,
        resolution_ref: request.resolution_ref.as_str().to_owned(),
        authority_source_ref: request.authority_source_ref.clone(),
        refusal: None,
        actualisation_request: Some(request_value),
        provenance: vec![
            format!(
                "resolved against governing authority {}",
                record.authority_source_ref
            ),
            format!("holder {}", record.holder),
            format!(
                "operations used: determine-agency{}",
                if derivation { ", actualise-agency" } else { "" }
            ),
        ],
    }
}

fn now_unix_seconds(explicit: Option<&str>) -> Result<u64, actuation_core::Error> {
    match explicit {
        Some(raw) => raw.parse().map_err(|_| {
            actuation_core::Error::new("--now must be a non-negative integer unix timestamp")
        }),
        None => Ok(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or_default()),
    }
}

fn store_from_flag(explicit: Option<String>) -> Result<AuthorityStore, actuation_core::Error> {
    AuthorityStore::from_environment(explicit.as_deref())
}

pub fn authority_issue(command: &Command) -> Result<Output, actuation_core::Error> {
    let mut args = command.args.clone();
    let store_flag = flag_value(&mut args, "--store")?;
    let now = flag_value(&mut args, "--now")?;
    let input = read_json_input(positional(&args).as_deref(), &command.stdin)?;
    let record: GoverningAuthorityRecord = serde_json::from_value(input).map_err(|e| {
        actuation_core::Error::new(format!("invalid governing authority record: {e}"))
    })?;
    if record.allowed_world_refs.is_empty() {
        return Err(actuation_core::Error::new(
            "an authority scoped to no world is not an authority; name allowed_world_refs",
        ));
    }
    let store = store_from_flag(store_flag)?;
    let at = now_unix_seconds(now.as_deref())?;
    let value = serde_json::to_value(&record)
        .map_err(|e| actuation_core::Error::new(format!("record could not be encoded: {e}")))?;
    let state = store.issue(&record.authority_source_ref, at, value)?;
    output(
        json!({
            "schema": LOCAL_AUTHORITY_VERSION,
            "issued": true,
            "authority_source_ref": record.authority_source_ref,
            "holder": record.holder,
            "issued_at_unix_seconds": state.issued_at_unix_seconds,
        }),
        command.json,
        render_local_authority_issue,
    )
}

pub fn authority_resolve(command: &Command) -> Result<Output, actuation_core::Error> {
    let mut args = command.args.clone();
    let store_flag = flag_value(&mut args, "--store")?;
    let now = flag_value(&mut args, "--now")?;
    let input = read_json_input(positional(&args).as_deref(), &command.stdin)?;
    let request: LocalResolutionRequest = serde_json::from_value(input).map_err(|e| {
        actuation_core::Error::new(format!("invalid local resolution request: {e}"))
    })?;
    let store = store_from_flag(store_flag)?;
    let at = now_unix_seconds(now.as_deref())?;
    let outcome = resolve_local_authority(&request, &store, at);
    output(
        serde_json::to_value(&outcome).map_err(|e| {
            actuation_core::Error::new(format!("outcome could not be encoded: {e}"))
        })?,
        command.json,
        render_local_authority_resolve,
    )
}

pub fn authority_revoke(command: &Command) -> Result<Output, actuation_core::Error> {
    let mut args = command.args.clone();
    let store_flag = flag_value(&mut args, "--store")?;
    let now = flag_value(&mut args, "--now")?;
    let reason =
        flag_value(&mut args, "--reason")?.unwrap_or_else(|| "owner withdrawal".to_owned());
    let source_ref = positional(&args).ok_or_else(|| {
        actuation_core::Error::new("authority revoke requires an authority source ref")
    })?;
    let store = store_from_flag(store_flag)?;
    let at = now_unix_seconds(now.as_deref())?;
    let state = store.revoke(&source_ref, at, reason)?;
    output(
        json!({
            "schema": LOCAL_AUTHORITY_VERSION,
            "revoked": true,
            "authority_source_ref": source_ref,
            "revoked_at_unix_seconds": state.revoked.as_ref().map(|r| r.at_unix_seconds),
        }),
        command.json,
        render_local_authority_revoke,
    )
}

fn render_local_authority_issue(value: &Value) -> String {
    format!(
        "issued governing authority {} (holder {})",
        value["authority_source_ref"].as_str().unwrap_or("?"),
        value["holder"].as_str().unwrap_or("?")
    )
}

fn render_local_authority_resolve(value: &Value) -> String {
    let standing = value["standing"].as_str().unwrap_or("?");
    let mut text = format!(
        "resolution {}: {standing}",
        value["resolution_ref"].as_str().unwrap_or("?")
    );
    if let Some(code) = value["refusal"]["code"].as_str() {
        text.push_str(&format!(
            " — {code}: {}",
            value["refusal"]["reason"].as_str().unwrap_or("")
        ));
    }
    if value["actualisation_request"].is_object() {
        text.push_str(" — pipe the request into `actuation agency actualise`");
    }
    text
}

fn render_local_authority_revoke(value: &Value) -> String {
    format!(
        "revoked governing authority {}",
        value["authority_source_ref"].as_str().unwrap_or("?")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use actuation_core::AgencyRef;
    use actuation_runtime::IdentityStanding;

    fn valid_record_value() -> Value {
        serde_json::json!({
            "schema": "actuation.local-authority/v1",
            "authority_source_ref": "authority-source:governing-main",
            "holder": "human:owner",
            "issued_by": "human:owner",
            "governing_binding": {
                "schema": "actuation.agency/v1",
                "binding_ref": "binding:governing",
                "agent_ref": "agent:governor",
                "agency_ref": "agency:governing",
                "world_ref": "world:personal",
                "scope_ref": "scope:personal",
                "bounds_refs": ["bound:personal", "bound:project:delegation", "bound:secondary"],
                "authority_refs": ["authority:metagency", "authority:project:delegation"],
                "return_relation_ref": "return-relation:governing"
            },
            "metagency_grant": {
                "schema": "actuation.agency/v1",
                "grant_ref": "grant:project-agency",
                "agency_ref": "agency:governing",
                "world_binding_ref": "binding:governing",
                "authority_ref": "authority:metagency",
                "bounds_refs": ["bound:project:delegation", "bound:secondary"],
                "operations": ["determine-agency", "actualise-agency"]
            },
            "allowed_world_refs": ["central:project:Example"],
            "issued_at_unix_seconds": 1726300000
        })
    }

    fn resolution_request() -> LocalResolutionRequest {
        serde_json::from_value(json!({
            "schema": "actuation.local-authority/v1",
            "resolution_ref": "resolution:walk-1",
            "request_ref": "actualisation-request:walk-1",
            "requester_ref": "human:owner",
            "authority_source_ref": "authority-source:governing-main",
            "determination_ref": "determination:walk-1",
            "differentiated_binding": {
                "schema": "actuation.agency/v1",
                "binding_ref": "binding:project:delegation",
                "agent_ref": "agent:existing-1",
                "agency_ref": "agency:project:delegation",
                "world_ref": "central:project:Example",
                "scope_ref": "central:project:Example:scope",
                "determining_agency_ref": "agency:governing",
                "bounds_refs": ["bound:secondary", "bound:project:delegation"],
                "authority_refs": ["authority:project:delegation"],
                "return_relation_ref": "return-relation:project:delegation",
                "continuity_ref": "continuity:agent:existing-1"
            },
            "agent_identity": {
                "standing": "existing",
                "evidence_refs": ["evidence:identity-registry"]
            },
            "requested_bounds_refs": ["bound:project:delegation", "bound:secondary"],
            "delegated_autonomy": {
                "allowed_action_refs": ["action:bounded-work"],
                "denied_action_refs": ["action:source-mutation"],
                "may_determine_within_bounds": true
            },
            "return_policy": {
                "mode": "required",
                "return_relation_ref": "return-relation:project:delegation"
            },
            "provenance": {
                "source_refs": ["actuation:#84"],
                "context_refs": []
            }
        }))
        .unwrap()
    }

    fn store_with_record() -> (tempfile::TempDir, AuthorityStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = AuthorityStore::new(dir.path().join("authority")).unwrap();
        store
            .issue(
                "authority-source:governing-main",
                1726300000,
                valid_record_value(),
            )
            .unwrap();
        (dir, store)
    }

    #[test]
    fn the_holder_resolves_a_full_admissible_request() {
        let (_dir, store) = store_with_record();
        let outcome = resolve_local_authority(&resolution_request(), &store, 1726300100);
        assert_eq!(outcome.standing, LocalResolutionStanding::Admitted);
        let assembled = outcome
            .actualisation_request
            .expect("admitted resolution carries the request");
        let admitted = serde_json::from_value::<ActualisationRequest>(assembled)
            .unwrap()
            .admit()
            .expect("the assembled request must pass constitutional admission");
        assert_eq!(
            admitted.binding().fields().agency_ref,
            AgencyRef::new("agency:project:delegation").unwrap()
        );
    }

    #[test]
    fn a_non_holder_is_refused_before_any_scope_is_read() {
        let (_dir, store) = store_with_record();
        let mut request = resolution_request();
        request.requester_ref = ExternalRef::new("human:intruder").unwrap();
        let outcome = resolve_local_authority(&request, &store, 1726300100);
        assert_eq!(
            outcome.refusal.as_ref().unwrap().code,
            "authority.wrong_holder"
        );
        assert!(outcome.actualisation_request.is_none());
    }

    #[test]
    fn bounds_beyond_the_grant_are_refused() {
        let (_dir, store) = store_with_record();
        let mut request = resolution_request();
        request.requested_bounds_refs = vec![BoundsRef::new("bound:world-dominion").unwrap()];
        let outcome = resolve_local_authority(&request, &store, 1726300100);
        assert_eq!(
            outcome.refusal.as_ref().unwrap().code,
            "authority.bounds_exceeded"
        );
    }

    #[test]
    fn a_world_outside_the_scoped_set_is_refused() {
        let (_dir, store) = store_with_record();
        let mut request = resolution_request();
        let mut binding = request.differentiated_binding.fields().clone();
        binding.world_ref = WorldRef::new("central:project:Other").unwrap();
        request.differentiated_binding = WorldBinding::new(binding).unwrap();
        let outcome = resolve_local_authority(&request, &store, 1726300100);
        assert_eq!(
            outcome.refusal.as_ref().unwrap().code,
            "authority.world_not_scoped"
        );
    }

    #[test]
    fn revoked_and_expired_and_unknown_sources_never_admit() {
        let (dir, store) = store_with_record();
        store
            .revoke(
                "authority-source:governing-main",
                1726300050,
                "owner withdrawal".into(),
            )
            .unwrap();
        let outcome = resolve_local_authority(&resolution_request(), &store, 1726300100);
        assert_eq!(outcome.refusal.as_ref().unwrap().code, "authority.revoked");

        let fresh = AuthorityStore::new(dir.path().join("fresh")).unwrap();
        let mut record_value = valid_record_value();
        record_value["authority_source_ref"] = json!("authority-source:stale");
        record_value["expires_at_unix_seconds"] = json!(1726300005);
        fresh
            .issue("authority-source:stale", 1726300000, record_value)
            .unwrap();
        let mut request = resolution_request();
        request.authority_source_ref = "authority-source:stale".into();
        let outcome = resolve_local_authority(&request, &fresh, 1726300100);
        assert_eq!(outcome.refusal.as_ref().unwrap().code, "authority.expired");

        let mut request = resolution_request();
        request.authority_source_ref = "authority-source:missing".into();
        let outcome = resolve_local_authority(&request, &fresh, 1726300100);
        assert_eq!(outcome.standing, LocalResolutionStanding::Unavailable);
        assert_eq!(
            outcome.refusal.as_ref().unwrap().code,
            "authority.unknown_source"
        );
    }

    #[test]
    fn a_derived_identity_needs_the_actualise_operation_on_the_grant() {
        let (_dir, store) = store_with_record();
        let mut request = resolution_request();
        request.agent_identity.standing = IdentityStanding::Actualised;
        let with_both = resolve_local_authority(&request, &store, 1726300100);
        assert_eq!(
            with_both.standing,
            LocalResolutionStanding::Admitted,
            "the fixture grant carries both operations"
        );

        let (dir2, store2) = store_with_record();
        let mut narrowed = valid_record_value();
        narrowed["metagency_grant"]["operations"] = serde_json::json!(["determine-agency"]);
        store2
            .issue("authority-source:no-actualise", 1726300000, narrowed)
            .unwrap();
        let mut request = resolution_request();
        request.authority_source_ref = "authority-source:no-actualise".into();
        request.agent_identity.standing = IdentityStanding::Actualised;
        let outcome = resolve_local_authority(&request, &store2, 1726300100);
        assert_eq!(
            outcome.refusal.as_ref().unwrap().code,
            "authority.operation_not_granted"
        );
        drop(dir2);
    }
}
