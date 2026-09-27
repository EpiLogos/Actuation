//! Bounded process adapter over the native Nara binding. The pipe owner admits
//! QL context and supplies transport evidence; this actor performs no audio I/O.
use crate::dispatch::{Command, Output};
use actuation_adapters::SpeechConstitution;
use actuation_core::{Error, ExternalRef, Result};
use actuation_runtime::NaraBinding;
use actuation_stream::Timestamp;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::io::{BufRead, Read, Write};

const MAX_LINE: usize = 256 * 1024;
const MAX_REPLY: usize = 1024 * 1024;
const MAX_REQUESTS: usize = 65536;
const RESPONSE_SCHEMA: &str = "actuation.nara-session-response/v1";

#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
enum Operation {
    Constitute {
        constitution: Value,
        dialogue_context: Value,
        allowed_action_refs: Vec<String>,
        denied_action_refs: Vec<String>,
    },
    Read,
    Context {
        dialogue_context: Value,
    },
    Listen,
    Response {
        response_ref: String,
    },
    Complete {
        response_ref: String,
    },
    Interrupt {
        interruption_ref: String,
        response_ref: String,
        reason: String,
        evidence_refs: Vec<String>,
        at: String,
        effect: TransportEffect,
    },
    Reconnect {
        constitution: Value,
        dialogue_context: Value,
        change_ref: String,
        reason: String,
        evidence_refs: Vec<String>,
        at: String,
    },
    Close,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum TransportEffect {
    PlaybackStopped,
    ProviderCancelled,
}

fn bounded_text(value: &str, field: &str, maximum: usize) -> Result<()> {
    if value.trim().is_empty() || value.len() > maximum || value.chars().any(char::is_control) {
        return Err(Error::new(format!(
            "{field} must be nonempty, bounded text without control characters"
        )));
    }
    Ok(())
}

fn reference(value: String) -> Result<ExternalRef> {
    bounded_text(&value, "reference", 4096)?;
    ExternalRef::new(value)
}

fn references(values: Vec<String>, required: bool) -> Result<Vec<ExternalRef>> {
    if values.len() > 128 || (required && values.is_empty()) {
        return Err(Error::new(
            "expected at most 128 references and required evidence must be nonempty",
        ));
    }
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .map(|value| {
            if !seen.insert(value.clone()) {
                return Err(Error::new("duplicate reference"));
            }
            reference(value)
        })
        .collect()
}

fn current_response(binding: &NaraBinding, response_ref: &str) -> Result<()> {
    bounded_text(response_ref, "response_ref", 4096)?;
    if binding.session().read()["in_flight_response_ref"].as_str() != Some(response_ref) {
        return Err(Error::new(
            "response_ref does not name the current native response",
        ));
    }
    Ok(())
}

fn apply(state: &mut Option<NaraBinding>, operation: Operation) -> Result<(Value, bool)> {
    if let Operation::Constitute {
        constitution,
        dialogue_context,
        allowed_action_refs,
        denied_action_refs,
    } = operation
    {
        if state.is_some() {
            return Err(Error::new("this actor is already constituted"));
        }
        let allowed = references(allowed_action_refs, false)?;
        let denied = references(denied_action_refs, false)?;
        if allowed.iter().any(|value| denied.contains(value)) {
            return Err(Error::new("an action cannot be both allowed and denied"));
        }
        *state = Some(NaraBinding::constitute(
            SpeechConstitution::try_from(constitution)?,
            dialogue_context,
            allowed,
            denied,
        )?);
        return Ok((json!({}), false));
    }
    if matches!(operation, Operation::Close) {
        return Ok((
            json!({"actor_closed": true, "canonical_agent_session_destroyed": false,
            "provider_cancellation_performed": false}),
            true,
        ));
    }
    let binding = state
        .as_mut()
        .ok_or_else(|| Error::new("constitute the actor before using its binding"))?;
    let receipt = match operation {
        Operation::Read => json!({}),
        Operation::Context { dialogue_context } => {
            binding.update_context(dialogue_context)?;
            json!({})
        }
        Operation::Listen => {
            binding.session_mut().begin_listening()?;
            json!({})
        }
        Operation::Response { response_ref } => {
            binding
                .session_mut()
                .begin_response(reference(response_ref)?)?;
            json!({})
        }
        Operation::Complete { response_ref } => {
            current_response(binding, &response_ref)?;
            binding.session_mut().complete_response()?;
            json!({})
        }
        Operation::Interrupt {
            interruption_ref,
            response_ref,
            reason,
            evidence_refs,
            at,
            effect,
        } => {
            reference(interruption_ref.clone())?;
            bounded_text(&reason, "reason", 4096)?;
            Timestamp::new(at.clone())?;
            current_response(binding, &response_ref)?;
            let evidence = references(evidence_refs, true)?;
            let receipt = binding.interrupt(interruption_ref, reason, &at)?;
            // Native cancellation describes this local speech turn. Transport
            // effects are attributed to the pipe owner, never inferred from it.
            let (effect_name, scope, provider_cancelled) = match effect {
                TransportEffect::PlaybackStopped => ("playback-stopped", "audio-playback", false),
                TransportEffect::ProviderCancelled => {
                    ("provider-cancelled", "provider-response", true)
                }
            };
            json!({"receipt": receipt.as_value(), "transport_effect": {
                "effect": effect_name, "effect_scope": scope, "response_ref": response_ref,
                "evidence_refs": evidence, "at": at, "provider_cancelled": provider_cancelled,
                "standing": "caller-reported-native-effect"
            }})
        }
        Operation::Reconnect {
            constitution,
            dialogue_context,
            change_ref,
            reason,
            evidence_refs,
            at,
        } => {
            reference(change_ref.clone())?;
            bounded_text(&reason, "reason", 4096)?;
            Timestamp::new(at.clone())?;
            let next = SpeechConstitution::try_from(constitution)?;
            if next.agent_session_ref() != binding.session().agent_session_ref() {
                return Err(Error::new("this pipe belongs to one canonical AgentSession; open a new actor for a different session"));
            }
            let receipt = binding.reconnect(
                change_ref,
                next,
                dialogue_context,
                reason,
                references(evidence_refs, true)?,
                &at,
            )?;
            json!({"receipt": receipt.as_value()})
        }
        Operation::Constitute { .. } | Operation::Close => unreachable!(),
    };
    Ok((receipt, false))
}

fn error(request_ref: Option<&str>, message: impl ToString) -> Value {
    json!({"schema": RESPONSE_SCHEMA, "request_ref": request_ref, "ok": false, "error": message.to_string()})
}

fn write_reply(writer: &mut impl Write, reply: &Value) -> Result<()> {
    serde_json::to_writer(&mut *writer, reply).map_err(|e| Error::new(e.to_string()))?;
    writer
        .write_all(b"\n")
        .and_then(|()| writer.flush())
        .map_err(|e| Error::new(e.to_string()))
}

fn run(reader: &mut impl BufRead, writer: &mut impl Write) -> Result<i32> {
    let mut state = None;
    let mut seen = BTreeSet::<[u8; 32]>::new();
    loop {
        let mut line = Vec::new();
        let count = reader
            .by_ref()
            .take((MAX_LINE + 1) as u64)
            .read_until(b'\n', &mut line)
            .map_err(|e| Error::new(e.to_string()))?;
        if count == 0 {
            return Ok(0);
        }
        if count > MAX_LINE || line.last() != Some(&b'\n') || seen.len() >= MAX_REQUESTS {
            write_reply(writer, &error(None, "actor line/request limit reached, or EOF before newline; reopen the ephemeral actor"))?;
            return Ok(2);
        }
        let mut value: Value = match serde_json::from_slice(&line) {
            Ok(value) => value,
            Err(_) => {
                write_reply(writer, &error(None, "invalid JSON request"))?;
                continue;
            }
        };
        let request_ref = value["request_ref"].as_str().map(str::to_owned);
        let valid_ref = request_ref
            .as_deref()
            .filter(|v| bounded_text(v, "request_ref", 256).is_ok());
        let result = (|| -> Result<(Option<NaraBinding>, Value, bool)> {
            let request_ref = valid_ref.ok_or_else(|| Error::new("invalid request_ref"))?;
            if value["schema"] != "actuation.nara-session-request/v1" {
                return Err(Error::new("expected actuation.nara-session-request/v1"));
            }
            if !seen.insert(Sha256::digest(request_ref.as_bytes()).into()) {
                return Err(Error::new("request_ref was already used; read the binding before deciding the next operation"));
            }
            let object = value
                .as_object_mut()
                .ok_or_else(|| Error::new("expected request object"))?;
            object.remove("schema");
            object.remove("request_ref");
            let operation = serde_json::from_value(value).map_err(|e| Error::new(e.to_string()))?;
            let mut candidate = state.clone();
            let (extra, close) = apply(&mut candidate, operation)?;
            let mut reply = json!({"schema": RESPONSE_SCHEMA, "request_ref": request_ref, "ok": true,
                "reading": candidate.as_ref().map(NaraBinding::read)});
            reply
                .as_object_mut()
                .unwrap()
                .extend(extra.as_object().unwrap().clone());
            if serde_json::to_vec(&reply)
                .map_err(|e| Error::new(e.to_string()))?
                .len()
                > MAX_REPLY
            {
                return Err(Error::new("native reply exceeds actor output bound"));
            }
            Ok((candidate, reply, close))
        })();
        match result {
            Ok((candidate, reply, close)) => {
                state = candidate;
                write_reply(writer, &reply)?;
                if close {
                    return Ok(0);
                }
            }
            Err(reason) => write_reply(writer, &error(valid_ref, reason))?,
        }
    }
}

pub fn serve(command: &Command) -> Result<Output> {
    if !command.args.is_empty() {
        return Err(Error::new(
            "usage: actuation nara serve (bounded JSON lines on stdin)",
        ));
    }
    let code = run(&mut std::io::stdin().lock(), &mut std::io::stdout().lock())?;
    Ok(Output {
        code,
        stdout: String::new(),
        stderr: String::new(),
    })
}
