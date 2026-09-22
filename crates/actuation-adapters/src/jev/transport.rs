use super::{Determination, SystemOneRequest, SystemOneResponse, SystemOneUsage, DOCUMENTED_INPUT_CEILING, ENDPOINT};
use crate::{Error, Result};
use actuation_core::{AgentRef, AgentSessionRef, AgencyRef, ExternalRef, RequestRef};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::time::{Duration, Instant};

/// Native identities are carried unchanged. External correlations can name the
/// existing NOW, Factory Commission/Run and prepared-context basis.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationAttribution {
    pub request_ref: RequestRef,
    pub agent_ref: AgentRef,
    pub agency_ref: AgencyRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_session_ref: Option<AgentSessionRef>,
    #[serde(default)]
    pub external_refs: Vec<ExternalRef>,
}

/// A tariff declaration, not an observed bill. Pin the model so an alias change
/// cannot silently change the spend basis. Current System One charges input
/// only; new billing semantics need an explicit new admission calculation.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JevPrice {
    pub model: String,
    pub input_microusd_per_million_tokens: u64,
    pub source_ref: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JevLimits {
    pub deadline_ms: u64,
    pub attempt_timeout_ms: u64,
    pub max_attempts: u32,
    pub retry_backoff_ms: u64,
    pub max_spend_microusd: u64,
    pub max_request_bytes: usize,
    pub max_response_bytes: usize,
    pub price: JevPrice,
}

impl JevLimits {
    pub fn validate(&self, request: &SystemOneRequest) -> Result<()> {
        if self.deadline_ms == 0 || self.deadline_ms > 120_000
            || self.attempt_timeout_ms == 0 || self.attempt_timeout_ms > self.deadline_ms
            || !(1..=4).contains(&self.max_attempts) || self.retry_backoff_ms > 10_000
            || self.max_request_bytes == 0 || self.max_request_bytes > 1_048_576
            || self.max_response_bytes == 0 || self.max_response_bytes > 4_194_304
            || self.max_spend_microusd == 0 || self.price.input_microusd_per_million_tokens == 0
            || self.price.source_ref.trim().is_empty()
        {
            return Err(Error::new("jev.limits.invalid: finite time, attempts, bytes and spend bounds are required"));
        }
        if self.price.model != request.model || request.model == "jev-latest" || request.model == "jev-preview" {
            return Err(Error::new("jev.limits.model: pin the requested model to its declared tariff version"));
        }
        if self.reservation()? > self.max_spend_microusd {
            return Err(Error::new("jev.budget.exhausted: budget cannot admit one request at the service token ceiling"));
        }
        Ok(())
    }
    pub fn estimated_cost(&self, input_tokens: u64) -> Result<u64> {
        let product = u128::from(input_tokens) * u128::from(self.price.input_microusd_per_million_tokens);
        u64::try_from(product.div_ceil(1_000_000)).map_err(|_| Error::new("jev.budget.overflow"))
    }
    pub fn reservation(&self) -> Result<u64> { self.estimated_cost(DOCUMENTED_INPUT_CEILING) }
}

/// Cancellation is checked before every attempt, throughout backoff and before
/// a response can be published. An in-flight synchronous HTTP call is bounded
/// by attempt_timeout_ms; a CLI host may also terminate/reap the native process.
/// Cancellation cannot assert that a request already sent was not billed.
#[derive(Clone, Default)]
pub struct JevCancellation(Arc<AtomicBool>);
impl JevCancellation {
    pub fn cancel(&self) { self.0.store(true, Ordering::Release); }
    pub fn is_cancelled(&self) -> bool { self.0.load(Ordering::Acquire) }
}

/// Intentionally neither Debug nor Serialize. Only a previously resolved
/// native credential is accepted; this adapter never searches keychains,
/// another machine, or arbitrary environment variables for a secret.
pub struct JevCredential(String);
impl JevCredential {
    pub fn resolved(value: String) -> Result<Self> {
        if value.trim().is_empty() || value.len() > 16_384 || value.chars().any(char::is_control) {
            return Err(Error::new("jev.credential.invalid"));
        }
        Ok(Self(value))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum JevStanding { Completed, Failed, Cancelled, TimedOut, BudgetExhausted }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JevFailure {
    pub code: String,
    /// No raw provider error body, request state or credential is retained.
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JevAttempt {
    pub ordinal: u32,
    pub elapsed_ms: u64,
    pub reserved_cost_microusd: u64,
    pub http_status: Option<u16>,
    pub actual_model: Option<String>,
    pub usage: Option<SystemOneUsage>,
    pub estimated_cost_microusd: Option<u64>,
    /// A sent request whose response is unknown is not a free attempt.
    pub effect_uncertain: bool,
    pub failure: Option<JevFailure>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JevInvocationReceipt {
    pub schema: String,
    pub attribution: InvocationAttribution,
    pub request_digest: String,
    pub requested_model: String,
    pub standing: JevStanding,
    pub elapsed_ms: u64,
    pub limits: JevLimits,
    pub attempts: Vec<JevAttempt>,
    pub total_reserved_cost_microusd: u64,
    /// Absent when any attempt's usage/model/tariff relation is unknown.
    pub total_estimated_cost_microusd: Option<u64>,
    pub failure: Option<JevFailure>,
}

#[derive(Clone, Debug, Serialize)]
pub struct JevEvaluation {
    pub receipt: JevInvocationReceipt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<Determination>,
}

fn failure(code: &str, reason: impl Into<String>) -> JevFailure {
    JevFailure { code: code.to_owned(), reason: reason.into() }
}
fn elapsed(start: Instant) -> u64 { start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64 }

pub struct JevClient {
    endpoint: String,
    credential: JevCredential,
}
impl JevClient {
    pub fn new(credential: JevCredential) -> Self { Self { endpoint: ENDPOINT.into(), credential } }

    /// Explicit controlled-provider endpoint, restricted to literal loopback.
    /// Production egress stays on the official HTTPS endpoint; no redirects,
    /// URL credentials or ambient proxy can widen that authority.
    pub fn controlled_loopback(endpoint: &str, credential: JevCredential) -> Result<Self> {
        let uri: ureq::http::Uri = endpoint.parse().map_err(|_| Error::new("jev.endpoint.invalid"))?;
        if uri.scheme_str() != Some("http") || !matches!(uri.host(), Some("127.0.0.1" | "[::1]" | "::1"))
            || uri.port_u16().is_none() || uri.authority().is_some_and(|a| a.as_str().contains('@'))
            || uri.path() != "/v1/systemone" || uri.query().is_some()
        {
            return Err(Error::new("jev.endpoint.refused: controlled endpoint must be literal loopback /v1/systemone"));
        }
        Ok(Self { endpoint: endpoint.to_owned(), credential })
    }

    /// General typed invocation. The caller owns admission/disclosure and
    /// persists the returned receipt in its native Activity/Return boundary.
    pub fn evaluate(&self, request: &SystemOneRequest, attribution: InvocationAttribution,
        limits: JevLimits, cancellation: &JevCancellation) -> Result<JevEvaluation>
    {
        request.validate()?;
        limits.validate(request)?;
        let bytes = serde_json::to_vec(request)?;
        if bytes.len() > limits.max_request_bytes { return Err(Error::new("jev.request.too_large")); }
        let reservation = limits.reservation()?;
        let start = Instant::now();
        let deadline = Duration::from_millis(limits.deadline_ms);
        let mut evaluation = JevEvaluation {
            receipt: JevInvocationReceipt {
                schema: "actuation.jev-invocation/v1".into(), attribution,
                request_digest: format!("sha256:{:x}", Sha256::digest(&bytes)),
                requested_model: request.model.clone(), standing: JevStanding::Failed,
                elapsed_ms: 0, limits: limits.clone(), attempts: Vec::new(),
                total_reserved_cost_microusd: 0, total_estimated_cost_microusd: None, failure: None,
            }, response: None,
        };
        for ordinal in 1..=limits.max_attempts {
            if cancellation.is_cancelled() {
                evaluation.receipt.standing = JevStanding::Cancelled;
                evaluation.receipt.failure = Some(failure("jev.cancelled", "invocation cancelled"));
                break;
            }
            let Some(remaining) = deadline.checked_sub(start.elapsed()).filter(|d| !d.is_zero()) else {
                evaluation.receipt.standing = JevStanding::TimedOut;
                evaluation.receipt.failure = Some(failure("jev.deadline", "total invocation deadline reached"));
                break;
            };
            let Some(reserved) = evaluation.receipt.total_reserved_cost_microusd.checked_add(reservation)
                .filter(|value| *value <= limits.max_spend_microusd) else {
                evaluation.receipt.standing = JevStanding::BudgetExhausted;
                evaluation.receipt.failure = Some(failure("jev.budget.exhausted", "no further attempt admitted"));
                break;
            };
            evaluation.receipt.total_reserved_cost_microusd = reserved;
            let attempt_start = Instant::now();
            let timeout = remaining.min(Duration::from_millis(limits.attempt_timeout_ms));
            let agent: ureq::Agent = ureq::Agent::config_builder()
                .timeout_global(Some(timeout)).timeout_connect(Some(timeout.min(Duration::from_secs(5))))
                .http_status_as_error(false).max_redirects(0).proxy(None).build().into();
            let mut attempt = JevAttempt { ordinal, elapsed_ms: 0, reserved_cost_microusd: reservation,
                http_status: None, actual_model: None, usage: None, estimated_cost_microusd: None,
                effect_uncertain: true, failure: None };
            let mut retry_after = Duration::ZERO;
            let mut retry = false;
            match agent.post(&self.endpoint).header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", self.credential.0)).send(bytes.as_slice())
            {
                Err(error) => {
                    let timed_out = matches!(error, ureq::Error::Timeout(_));
                    evaluation.receipt.standing = if timed_out { JevStanding::TimedOut } else { JevStanding::Failed };
                    attempt.failure = Some(failure(if timed_out { "jev.timeout" } else { "jev.transport" },
                        "provider response unavailable; remote processing or cost may be uncertain"));
                    // A transport failure may follow a sent POST. Do not
                    // automatically replay an ambiguous invocation.
                }
                Ok(mut response) => {
                    let status = response.status().as_u16();
                    attempt.http_status = Some(status);
                    retry_after = response.headers().get("retry-after").and_then(|h| h.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok()).map(Duration::from_secs).unwrap_or_default();
                    match response.body_mut().with_config().limit(limits.max_response_bytes as u64).read_to_vec() {
                        Err(_) => { attempt.failure = Some(failure("jev.response.unavailable",
                            "provider body exceeded its bound, timed out or could not be read")); }
                        Ok(body) => {
                            let parsed = actuation_core::system_one::unique_json(&body);
                            if let Ok(value) = &parsed {
                                attempt.actual_model = value.get("model").and_then(Value::as_str).map(str::to_owned);
                                attempt.usage = value.get("usage").and_then(|u| serde_json::from_value(u.clone()).ok());
                                if attempt.actual_model.as_deref() == Some(limits.price.model.as_str()) {
                                    attempt.estimated_cost_microusd = attempt.usage.as_ref()
                                        .map(|u| limits.estimated_cost(u.input_tokens)).transpose()?;
                                }
                            }
                            if status == 200 {
                                let decoded = parsed.map_err(|_| Error::new("jev.answer.json: invalid response JSON"))
                                    .and_then(|v| serde_json::from_value::<SystemOneResponse>(v)
                                        .map_err(|_| Error::new("jev.answer.shape: required typed response fields are missing or invalid")))
                                    .and_then(|value| request.validate_response(value));
                                match decoded {
                                    Ok(value) if value.response().model != limits.price.model => {
                                        attempt.failure = Some(failure("jev.model.changed", "returned model differs from the admitted tariff/model version"));
                                    }
                                    Ok(value) if value.response().usage.input_tokens > DOCUMENTED_INPUT_CEILING => {
                                        attempt.failure = Some(failure("jev.budget.basis_violated", "reported usage exceeds the admitted service ceiling"));
                                    }
                                    Ok(value) => {
                                        attempt.effect_uncertain = false;
                                        evaluation.response = Some(value);
                                        evaluation.receipt.standing = JevStanding::Completed;
                                    }
                                    Err(error) => { attempt.failure = Some(failure("jev.answer.invalid", error.to_string())); }
                                }
                            } else {
                                attempt.failure = Some(failure("jev.http", format!("provider returned HTTP {status}")));
                                // Only explicit rate-limit/overload responses are
                                // retryable, and each consumes a reservation.
                                retry = matches!(status, 429 | 529);
                            }
                        }
                    }
                }
            }
            attempt.elapsed_ms = elapsed(attempt_start);
            evaluation.receipt.failure = attempt.failure.clone();
            evaluation.receipt.attempts.push(attempt);
            if cancellation.is_cancelled() {
                evaluation.response = None;
                evaluation.receipt.standing = JevStanding::Cancelled;
                evaluation.receipt.failure = Some(failure("jev.cancelled", "cancelled before publishing the returned determination"));
                break;
            }
            if start.elapsed() >= deadline {
                evaluation.response = None;
                evaluation.receipt.standing = JevStanding::TimedOut;
                evaluation.receipt.failure = Some(failure("jev.deadline", "late result discarded after invocation deadline"));
                break;
            }
            if evaluation.response.is_some() || !retry || ordinal == limits.max_attempts { break; }
            let backoff = Duration::from_millis(limits.retry_backoff_ms.saturating_mul(1u64 << (ordinal - 1))).max(retry_after);
            let until = start.elapsed().saturating_add(backoff).min(deadline);
            while start.elapsed() < until && !cancellation.is_cancelled() {
                std::thread::sleep(until.saturating_sub(start.elapsed()).min(Duration::from_millis(20)));
            }
        }
        evaluation.receipt.elapsed_ms = elapsed(start);
        evaluation.receipt.total_estimated_cost_microusd = evaluation.receipt.attempts.iter()
            .try_fold(0u64, |total, attempt| total.checked_add(attempt.estimated_cost_microusd?));
        if evaluation.receipt.standing != JevStanding::Completed { evaluation.response = None; }
        Ok(evaluation)
    }
}
