//! The speech/Nara surface verbs (Actuation #94/#91 follow-up).
//!
//! Document-in, document-out: each verb admits a supplied document through
//! the real library gate, runs one lane operation, and prints the native
//! receipt document it produced. No server process, no network, no audio
//! material — the desktop consumes these shapes exactly as the libraries
//! define them, and a session state is reconstituted only through the same
//! validated phase transitions a live session would take.

use crate::dispatch::{output, read_json_input, Command, Output};
use actuation_adapters::{adjudicate_speech_tool_request, SpeechConstitution, SpeechToolRequest};
use actuation_core::{ExternalRef, Result};
use actuation_runtime::{read_dialogue_context, NaraBinding, SpeechSession};
use serde_json::Value;

/// The supplied constitution document, admitted.
fn constitution_of(input: &Value) -> Result<SpeechConstitution> {
    SpeechConstitution::try_from(input["constitution"].clone())
        .map_err(|e| actuation_core::Error::new(format!("constitution: {e}")))
}

/// Reconstitute the document-described session state through the same
/// validated phase transitions a live session takes. A state the session
/// could not actually be in is refused, not approximated.
fn constituted_session(input: &Value) -> Result<SpeechSession> {
    let mut session = SpeechSession::constitute(constitution_of(input)?)?;
    let state = &input["session_state"];
    require_object(state)?;
    if !state.is_null() {
        match state["phase"].as_str() {
            None | Some("idle") => {}
            Some("listening") => session.begin_listening()?,
            Some("speaking") => {}
            Some(other) => {
                return Err(actuation_core::Error::new(format!(
                    "session_state phase {other} cannot be described into existence; the CLI reconstitutes idle, listening or speaking states only"
                )))
            }
        }
        if let Some(response) = state["in_flight_response_ref"].as_str() {
            session.begin_response(ExternalRef::new(response)?)?;
        }
        if let Some(executed) = state["executed_refs"].as_array() {
            for result in executed {
                let result = result.as_str().ok_or_else(|| {
                    actuation_core::Error::new("session_state executed_refs must be strings")
                })?;
                session.commit_result(ExternalRef::new(result)?)?;
            }
        }
    }
    Ok(session)
}

fn require_object(v: &Value) -> Result<()> {
    if v.is_null() || v.is_object() {
        Ok(())
    } else {
        Err(actuation_core::Error::new(
            "session_state must be an object when supplied",
        ))
    }
}

fn field(input: &Value, key: &str) -> Result<String> {
    input[key]
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| actuation_core::Error::new(format!("{key} is required")))
}

/// Admit a speech constitution. The receipt is the admitted document itself,
/// lossless.
pub fn speech_constitution(command: &Command) -> Result<Output> {
    let input = read_json_input(command.args.first().map(String::as_str), &command.stdin)?;
    let constitution = SpeechConstitution::try_from(input.clone())
        .map_err(|e| actuation_core::Error::new(format!("not an admissible constitution: {e}")))?;
    output(
        constitution.as_value().clone(),
        command.json,
        crate::render::speech_constitution,
    )
}

/// Run the authority gate on one speech-model tool request and print the
/// decision receipt. Authority comes only from the supplied explicit
/// allowed/denied lists; available capability is never permission.
pub fn speech_decision(command: &Command) -> Result<Output> {
    let input = read_json_input(command.args.first().map(String::as_str), &command.stdin)?;
    let constitution = constitution_of(&input)?;
    let request = SpeechToolRequest::try_from(input["request"].clone())
        .map_err(|e| actuation_core::Error::new(format!("request: {e}")))?;
    let external_refs = |key: &str| -> Result<Vec<ExternalRef>> {
        input[key]
            .as_array()
            .map(|refs| {
                refs.iter()
                    .map(|r| ExternalRef::new(r.as_str().unwrap_or_default()))
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()
            .map(|v| v.unwrap_or_default())
    };
    let decision = adjudicate_speech_tool_request(
        field(&input, "decision_ref")?,
        &constitution,
        request,
        &external_refs("allowed_action_refs")?,
        &external_refs("denied_action_refs")?,
        field(&input, "decided_by")?,
        &field(&input, "decided_at")?,
    )?;
    output(
        decision.as_value().clone(),
        command.json,
        crate::render::speech_decision,
    )
}

/// Run an interruption against a document-described session state and print
/// the receipt. Where the body cannot cancel, the receipt says so and
/// nothing else changes — degraded behaviour is recorded, never faked.
pub fn speech_interrupt(command: &Command) -> Result<Output> {
    let input = read_json_input(command.args.first().map(String::as_str), &command.stdin)?;
    let mut session = constituted_session(&input)?;
    let receipt = session.interrupt(
        field(&input, "interruption_ref")?,
        field(&input, "reason")?,
        &field(&input, "at")?,
    );
    output(
        receipt.as_value().clone(),
        command.json,
        crate::render::speech_interrupt,
    )
}

/// Print the session read model for a (possibly state-described) session.
pub fn speech_session(command: &Command) -> Result<Output> {
    let input = read_json_input(command.args.first().map(String::as_str), &command.stdin)?;
    let session = constituted_session(&input)?;
    output(session.read(), command.json, crate::render::speech_session)
}

/// Admit a caller-supplied QL dialogue context against its constitution and
/// print the identity-checked reading.
pub fn nara_context(command: &Command) -> Result<Output> {
    let input = read_json_input(command.args.first().map(String::as_str), &command.stdin)?;
    let constitution = constitution_of(&input)?;
    let reading = read_dialogue_context(&input["context"], &constitution)
        .map_err(|e| actuation_core::Error::new(format!("context: {e}")))?;
    output(
        reading.read_model(),
        command.json,
        crate::render::nara_context,
    )
}

/// The {constitution, context} every Nara verb binds on.
fn nara_binding(input: &Value) -> Result<NaraBinding> {
    let constitution = constitution_of(input)?;
    NaraBinding::constitute(constitution, input["context"].clone(), vec![], vec![])
        .map_err(|e| actuation_core::Error::new(format!("nara binding: {e}")))
}

/// Record a structured Nara→Epii delegation and print its receipt. The
/// delegation stays foreground Nara; application is never implied.
pub fn nara_delegate(command: &Command) -> Result<Output> {
    let input = read_json_input(command.args.first().map(String::as_str), &command.stdin)?;
    let mut nara = nara_binding(&input)?;
    let receipt = nara.delegate_to_epii(
        input["delegation"].clone(),
        field(&input, "delegation_receipt_ref")?,
        &field(&input, "at")?,
    )?;
    output(receipt, command.json, crate::render::nara_delegate)
}

/// Receive an Epii enrichment on a recorded delegation and print its
/// receipt. `applied` is structurally false in the receipt: a current result
/// is proposed only, a stale one retained-not-applied.
pub fn nara_enrichment(command: &Command) -> Result<Output> {
    let input = read_json_input(command.args.first().map(String::as_str), &command.stdin)?;
    let mut nara = nara_binding(&input)?;
    nara.delegate_to_epii(
        input["delegation"].clone(),
        field(&input, "delegation_receipt_ref")?,
        &field(&input, "delegated_at")?,
    )?;
    let receipt = nara.receive_enrichment(
        input["enrichment"].clone(),
        field(&input, "enrichment_receipt_ref")?,
        &field(&input, "at")?,
    )?;
    output(receipt, command.json, crate::render::nara_enrichment)
}
