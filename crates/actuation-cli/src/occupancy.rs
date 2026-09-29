//! `actuation occupancy`: stable World Position occupancy and tenure
//! (O-I #65 / #220, Factory #195; World inhabitation contract v1 §2).
//!
//! The ledger is Actuation's (`actuation_stream::OccupancyStore`). This module
//! parses the verbs, renders results, and turns every refusal into three
//! parts: the fact (current state, naming the holder), the consequence (what
//! did or did not happen) and the action (the exact next lawful command).
//! Refusals exit 2 with `{ok:false,error:{code,fact,consequence,action}}` on
//! stdout under `--json`, and as three lines on stderr otherwise.
use crate::dispatch::{Command, Output};
use actuation_core::{
    is_blank_reference, AgencyRef, AgentRef, AgentSessionRef, Error, ExternalRef, OccupancyReading,
    OccupancyState, OccupantGenerationRef, Presence, Tenure, TenureEndKind, TenureKind,
    WorldPositionRef,
};
use actuation_stream::{
    format_unix_ms, ClaimExpectation, ClaimRequest, OccupancyError, OccupancyListing,
    OccupancyStore,
};
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

/// The exit status of every refusal, shared with the other semantic refusals
/// of this executable.
pub const REFUSAL_EXIT_CODE: i32 = 2;

struct RefusalBody {
    code: &'static str,
    fact: String,
    consequence: String,
    action: String,
    detail: Map<String, Value>,
}

/// One refusal, boxed so every `Result` carrying it stays small.
struct Refusal(Box<RefusalBody>);

impl Refusal {
    fn new(
        code: &'static str,
        fact: impl Into<String>,
        consequence: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self(Box::new(RefusalBody {
            code,
            fact: fact.into(),
            consequence: consequence.into(),
            action: action.into(),
            detail: Map::new(),
        }))
    }

    fn with(mut self, key: &str, value: impl serde::Serialize) -> Self {
        self.0.detail.insert(
            key.to_owned(),
            serde_json::to_value(value).unwrap_or(Value::Null),
        );
        self
    }

    fn render(self, json: bool) -> Output {
        let body = *self.0;
        if json {
            let mut error = Map::new();
            error.insert("code".into(), json!(body.code));
            error.insert("fact".into(), json!(body.fact));
            error.insert("consequence".into(), json!(body.consequence));
            error.insert("action".into(), json!(body.action));
            error.extend(body.detail);
            Output {
                code: REFUSAL_EXIT_CODE,
                stdout: serde_json::to_string_pretty(&json!({"ok": false, "error": error}))
                    .expect("refusal serialises"),
                stderr: String::new(),
            }
        } else {
            Output {
                code: REFUSAL_EXIT_CODE,
                stdout: String::new(),
                stderr: format!(
                    "actuation: {}\n  {}\n  {}",
                    body.fact, body.consequence, body.action
                ),
            }
        }
    }
}

type Refused<T> = std::result::Result<T, Refusal>;

fn usage_of(name: &str) -> &'static str {
    crate::dispatch::commands()
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.usage)
        .unwrap_or("actuation help")
}

fn usage_refusal(name: &str, fact: impl Into<String>) -> Refusal {
    Refusal::new(
        "occupancy.usage",
        fact,
        "Nothing was read or written.",
        format!("Usage: {}", usage_of(name)),
    )
}

/// Strict argv: every flag is declared, given once, and nothing is left
/// over — a silently dropped flag would alter the tenure it describes.
struct Parsed {
    values: BTreeMap<&'static str, String>,
    switches: BTreeSet<&'static str>,
}

impl Parsed {
    fn parse(
        name: &str,
        args: &[String],
        value_flags: &[&'static str],
        switch_flags: &[&'static str],
    ) -> Refused<Self> {
        let mut parsed = Self {
            values: BTreeMap::new(),
            switches: BTreeSet::new(),
        };
        let mut index = 0;
        while index < args.len() {
            let arg = args[index].as_str();
            if let Some(flag) = value_flags.iter().find(|flag| **flag == arg) {
                let value = args
                    .get(index + 1)
                    .filter(|value| !value.starts_with("--"))
                    .ok_or_else(|| usage_refusal(name, format!("{flag} requires a value.")))?;
                if parsed.values.insert(flag, value.clone()).is_some() {
                    return Err(usage_refusal(name, format!("{flag} was given twice.")));
                }
                index += 2;
            } else if let Some(flag) = switch_flags.iter().find(|flag| **flag == arg) {
                if !parsed.switches.insert(flag) {
                    return Err(usage_refusal(name, format!("{flag} was given twice.")));
                }
                index += 1;
            } else {
                return Err(usage_refusal(
                    name,
                    format!(
                        "Unexpected argument {arg:?} for {}.",
                        name.replace('.', " ")
                    ),
                ));
            }
        }
        Ok(parsed)
    }

    fn get(&self, flag: &str) -> Option<&str> {
        self.values.get(flag).map(String::as_str)
    }

    fn required(&self, name: &str, flag: &str) -> Refused<&str> {
        self.get(flag).ok_or_else(|| {
            usage_refusal(name, format!("{} requires {flag}.", name.replace('.', " ")))
        })
    }

    fn typed<T>(
        &self,
        name: &str,
        flag: &str,
        make: impl Fn(&str) -> actuation_core::Result<T>,
    ) -> Refused<Option<T>> {
        self.get(flag)
            .map(|raw| make(raw).map_err(|e| usage_refusal(name, format!("Invalid {flag}: {e}."))))
            .transpose()
    }
}

/// Quote a value for a copy-pasteable next command.
fn shell(value: &str) -> String {
    let plain = !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "@%+=:,./_-".contains(c));
    if plain {
        value.to_owned()
    } else {
        format!("'{}'", value.replace('\'', r"'\''"))
    }
}

/// Where the next command must look: the same store this one used.
#[derive(Clone)]
struct Scope {
    position: String,
    store_flag: Option<String>,
}

impl Scope {
    fn store_suffix(&self) -> String {
        self.store_flag
            .as_deref()
            .map(|store| format!(" --store {}", shell(store)))
            .unwrap_or_default()
    }
    fn read_command(&self) -> String {
        format!(
            "actuation occupancy read --position {}{}",
            shell(&self.position),
            self.store_suffix()
        )
    }
    fn release_command(&self, generation: &str) -> String {
        format!(
            "actuation occupancy release --position {} --generation {generation} --reason <why>{}",
            shell(&self.position),
            self.store_suffix()
        )
    }
    fn fresh_claim_command(&self, agent: &str, agency: &str) -> String {
        format!(
            "actuation occupancy claim --position {} --agent {} --agency {} --expect-vacant --reason <why>{}",
            shell(&self.position),
            shell(agent),
            shell(agency),
            self.store_suffix()
        )
    }
}

fn describe(tenure: &Tenure) -> String {
    format!(
        "generation {} (ordinal {}, agent {}, session {}, since {})",
        tenure.generation_ref,
        tenure.generation_ordinal,
        tenure.agent_ref,
        tenure
            .agent_session_ref
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "not named".into()),
        format_unix_ms(tenure.began_at_unix_ms)
    )
}

fn holder_sentence(reading: &OccupancyReading) -> String {
    match &reading.current {
        Some(current) => format!("{} is held by {}.", reading.position_ref, describe(current)),
        None => format!("{} is vacant.", reading.position_ref),
    }
}

fn how_it_ended(tenure: &Tenure, reading: &OccupancyReading) -> String {
    let at = tenure
        .ended_at_unix_ms
        .map(format_unix_ms)
        .unwrap_or_default();
    match tenure.end_kind {
        Some(TenureEndKind::Superseded) => {
            let successor = reading
                .generations
                .iter()
                .find(|next| {
                    next.predecessor_generation_ref.as_ref() == Some(&tenure.generation_ref)
                })
                .map(|next| {
                    format!(
                        "generation {} (ordinal {})",
                        next.generation_ref, next.generation_ordinal
                    )
                })
                .unwrap_or_else(|| "a successor".into());
            format!("superseded by {successor} at {at}")
        }
        _ => format!(
            "released at {at} ({})",
            tenure.end_reason.as_deref().unwrap_or("no reason recorded")
        ),
    }
}

/// The three parts for every store refusal. `retry` renders the claim that
/// would be lawful against the state found, when the verb was a claim.
fn refusal_for(
    error: OccupancyError,
    verb: &str,
    scope: &Scope,
    retry: Option<&dyn Fn(ClaimRetry) -> String>,
) -> Refusal {
    let code = error.code();
    let nothing = "Nothing was written.";
    match error {
        OccupancyError::Occupied { reading } => {
            let current = reading.current.clone().expect("occupied has a current tenure");
            let action = retry
                .map(|retry| {
                    format!(
                        "To supersede that occupant deliberately, run: {}",
                        retry(ClaimRetry::Supersede(current.generation_ref.to_string()))
                    )
                })
                .unwrap_or_else(|| format!("Read the Position: {}", scope.read_command()));
            Refusal::new(
                code,
                holder_sentence(&reading),
                "Nothing was written; a claim that does not name the current occupant with --expect-generation cannot supersede it.",
                action,
            )
            .with("position_ref", &reading.position_ref)
            .with("current", &current)
        }
        OccupancyError::StaleExpectation { expected, reading } => {
            let expected_was = match reading
                .generations
                .iter()
                .find(|t| t.generation_ref == expected)
            {
                Some(tenure) => format!(
                    "{expected} (ordinal {}) was {}",
                    tenure.generation_ordinal,
                    how_it_ended(tenure, &reading)
                ),
                None => format!("{expected} has never held it"),
            };
            let (fact, action) = match (&reading.current, retry) {
                (Some(current), Some(retry)) => (
                    format!("{} Expected {expected}, but {expected_was}.", holder_sentence(&reading)),
                    format!(
                        "Re-read with `{}`; to supersede the current occupant deliberately, run: {}",
                        scope.read_command(),
                        retry(ClaimRetry::Supersede(current.generation_ref.to_string()))
                    ),
                ),
                (None, Some(retry)) => (
                    format!("{} Expected {expected}, but {expected_was}.", holder_sentence(&reading)),
                    format!("To occupy the vacant Position, run: {}", retry(ClaimRetry::Vacant)),
                ),
                (_, None) => (holder_sentence(&reading), scope.read_command()),
            };
            let mut refusal = Refusal::new(
                code,
                fact,
                "Nothing was written; a claim against a stale expectation never supersedes anyone.",
                action,
            )
            .with("position_ref", &reading.position_ref)
            .with("expected_generation_ref", &expected);
            if let Some(current) = &reading.current {
                refusal = refusal.with("current", current);
            }
            refusal
        }
        OccupancyError::Superseded { generation, reading } => {
            let consequence = match verb {
                "verify" => format!(
                    "Generation {} is not the current occupant and holds no authority at {}; nothing was written.",
                    generation.generation_ref, reading.position_ref
                ),
                "presence" => "Nothing was written; only the current occupant reports presence.".into(),
                _ => "Nothing was written; only the current occupant can release the Position.".into(),
            };
            let action = match &reading.current {
                Some(_) => format!(
                    "Stop acting as {}. Read the current occupancy: {}",
                    reading.position_ref,
                    scope.read_command()
                ),
                None => format!(
                    "To occupy {} again, claim a fresh tenure: {}",
                    reading.position_ref,
                    scope.fresh_claim_command(generation.agent_ref.as_str(), generation.agency_ref.as_str())
                ),
            };
            let mut refusal = Refusal::new(
                code,
                format!(
                    "Generation {} (ordinal {}) no longer holds {}: it was {}. {}",
                    generation.generation_ref,
                    generation.generation_ordinal,
                    reading.position_ref,
                    how_it_ended(&generation, &reading),
                    holder_sentence(&reading)
                ),
                consequence,
                action,
            )
            .with("position_ref", &reading.position_ref)
            .with("generation", &generation);
            if let Some(current) = &reading.current {
                refusal = refusal.with("current", current);
            }
            refusal
        }
        OccupancyError::UnknownGeneration { generation, reading } => {
            let mut refusal = Refusal::new(
                code,
                format!(
                    "{generation} has never held {}. {}",
                    reading.position_ref,
                    holder_sentence(&reading)
                ),
                if verb == "verify" {
                    format!("{generation} holds no authority at {}; nothing was written.", reading.position_ref)
                } else {
                    nothing.into()
                },
                format!(
                    "Read the Position and use the generation it names: {}",
                    scope.read_command()
                ),
            )
            .with("position_ref", &reading.position_ref)
            .with("generation_ref", &generation);
            if let Some(current) = &reading.current {
                refusal = refusal.with("current", current);
            }
            refusal
        }
        OccupancyError::KindInconsistent { kind, detail, reading } => Refusal::new(
            code,
            format!("{detail}. {}", holder_sentence(&reading)),
            nothing,
            match retry {
                Some(retry) => format!(
                    "Re-run without --kind {kind} to take the kind the history implies: {}",
                    retry(ClaimRetry::WithoutKind)
                ),
                None => scope.read_command(),
            },
        )
        .with("position_ref", &reading.position_ref),
        OccupancyError::Ambiguous { position_ref, path, open } => {
            let candidates = open
                .iter()
                .map(describe)
                .collect::<Vec<_>>()
                .join("; ");
            let generations = open
                .iter()
                .map(|t| t.generation_ref.to_string())
                .collect::<Vec<_>>()
                .join(" | ");
            Refusal::new(
                code,
                format!(
                    "{position_ref} has {} open tenures in {} ({candidates}); the newest is never taken as the occupant.",
                    open.len(),
                    path.display()
                ),
                "Nothing was written; the Position has no determinable occupant until only one tenure is open.",
                format!(
                    "Decide which generation truly holds {position_ref}, then end each other one with: {}",
                    scope.release_command(&format!("<{generations}>"))
                ),
            )
            .with("position_ref", &position_ref)
            .with("candidates", &open)
        }
        OccupancyError::Corrupt { path, line, detail, .. } => Refusal::new(
            code,
            format!(
                "The occupancy ledger of {} at {} is unreadable: {detail}.",
                scope.position,
                path.display()
            ),
            "Nothing was written and nothing was repaired; the occupancy of this Position cannot be determined.",
            format!(
                "Inspect {} (append-only evidence: keep a copy of any line you set aside), then re-run: {}",
                path.display(),
                scope.read_command()
            ),
        )
        .with("path", path.display().to_string())
        .with("line", line),
        OccupancyError::Invalid { detail } => {
            usage_refusal(&format!("occupancy.{verb}"), format!("{detail}."))
        }
        OccupancyError::Store {
            path,
            detail,
            outcome_unknown: true,
        } => Refusal::new(
            code,
            format!("Writing the occupancy ledger at {} failed: {detail}.", path.display()),
            "The outcome is UNKNOWN: the write may or may not have landed.",
            format!("Re-read before retrying: {}", scope.read_command()),
        ),
        OccupancyError::Store { path, detail, .. } => Refusal::new(
            code,
            format!("The occupancy store at {} is unavailable: {detail}.", path.display()),
            nothing,
            "Check the store directory (--store <dir> or ACTUATION_OCCUPANCY_STORE), then re-run the command.",
        ),
    }
}

enum ClaimRetry {
    Supersede(String),
    Vacant,
    WithoutKind,
}

fn store(scope: &Scope, verb: &str) -> Refused<OccupancyStore> {
    OccupancyStore::from_environment(scope.store_flag.as_deref()).map_err(|e| {
        Refusal::new(
            "occupancy.store_unavailable",
            format!("No occupancy store could be selected: {e}."),
            "Nothing was read or written.",
            format!(
                "Name one with --store <dir> or ACTUATION_OCCUPANCY_STORE, then re-run: {}",
                usage_of(&format!("occupancy.{verb}"))
            ),
        )
    })
}

fn position(name: &str, parsed: &Parsed) -> Refused<(WorldPositionRef, Scope)> {
    let raw = parsed.required(name, "--position")?;
    let position = WorldPositionRef::new(raw)
        .map_err(|e| usage_refusal(name, format!("Invalid --position: {e}.")))?;
    Ok((
        position,
        Scope {
            position: raw.to_owned(),
            store_flag: parsed.get("--store").map(str::to_owned),
        },
    ))
}

fn generation(name: &str, parsed: &Parsed) -> Refused<OccupantGenerationRef> {
    let raw = parsed.required(name, "--generation")?;
    OccupantGenerationRef::new(raw)
        .map_err(|e| usage_refusal(name, format!("Invalid --generation: {e}.")))
}

fn reason(name: &str, parsed: &Parsed) -> Refused<String> {
    let raw = parsed.required(name, "--reason")?;
    if is_blank_reference(raw) {
        return Err(usage_refusal(name, "--reason must state why."));
    }
    Ok(raw.to_owned())
}

fn respond(result: Refused<Value>, json: bool, human: fn(&Value) -> String) -> Output {
    match result {
        Ok(value) => Output {
            code: 0,
            stdout: if json {
                serde_json::to_string_pretty(&value).expect("result serialises")
            } else {
                human(&value)
            },
            stderr: String::new(),
        },
        Err(refusal) => refusal.render(json),
    }
}

const CLAIM_VALUES: &[&str] = &[
    "--position",
    "--agent",
    "--agency",
    "--agent-session",
    "--session-space",
    "--harness-composition",
    "--model",
    "--workcell",
    "--gateway-address",
    "--reason",
    "--expect-generation",
    "--kind",
    "--store",
];

/// The flags that describe the occupant, in usage order, for rebuilding the
/// exact next claim.
const CLAIM_DESCRIPTIVE: &[&str] = &[
    "--position",
    "--agent",
    "--agency",
    "--agent-session",
    "--session-space",
    "--harness-composition",
    "--model",
    "--workcell",
    "--gateway-address",
    "--reason",
];

fn claim_command(parsed: &Parsed, retry: ClaimRetry) -> String {
    let mut words = vec!["actuation occupancy claim".to_owned()];
    for flag in CLAIM_DESCRIPTIVE {
        if let Some(value) = parsed.get(flag) {
            words.push(format!("{flag} {}", shell(value)));
        }
    }
    match &retry {
        ClaimRetry::Supersede(generation) => {
            words.push(format!("--expect-generation {generation}"))
        }
        ClaimRetry::Vacant => words.push("--expect-vacant".into()),
        ClaimRetry::WithoutKind => {
            if parsed.switches.contains("--expect-vacant") {
                words.push("--expect-vacant".into());
            }
            if let Some(expected) = parsed.get("--expect-generation") {
                words.push(format!("--expect-generation {expected}"));
            }
        }
    }
    if !matches!(retry, ClaimRetry::WithoutKind) {
        if let Some(kind) = parsed.get("--kind") {
            words.push(format!("--kind {kind}"));
        }
    }
    if let Some(store) = parsed.get("--store") {
        words.push(format!("--store {}", shell(store)));
    }
    words.join(" ")
}

pub fn claim(command: &Command) -> Result<Output, Error> {
    const NAME: &str = "occupancy.claim";
    let run = || -> Refused<Value> {
        let parsed = Parsed::parse(NAME, &command.args, CLAIM_VALUES, &["--expect-vacant"])?;
        let (position_ref, scope) = position(NAME, &parsed)?;
        let agent_ref = AgentRef::new(parsed.required(NAME, "--agent")?)
            .map_err(|e| usage_refusal(NAME, format!("Invalid --agent: {e}.")))?;
        let agency_ref = AgencyRef::new(parsed.required(NAME, "--agency")?)
            .map_err(|e| usage_refusal(NAME, format!("Invalid --agency: {e}.")))?;
        let reason = reason(NAME, &parsed)?;
        let expectation = match (
            parsed.switches.contains("--expect-vacant"),
            parsed.typed(NAME, "--expect-generation", |raw: &str| {
                OccupantGenerationRef::new(raw)
            })?,
        ) {
            (true, Some(_)) => {
                return Err(usage_refusal(
                    NAME,
                    "--expect-vacant and --expect-generation contradict each other.",
                ))
            }
            (true, None) => ClaimExpectation::Vacant,
            (false, Some(generation)) => ClaimExpectation::Generation(generation),
            (false, None) => ClaimExpectation::Unstated,
        };
        let gateway_address = parsed.get("--gateway-address").map(str::to_owned);
        if gateway_address.as_deref().is_some_and(is_blank_reference) {
            return Err(usage_refusal(NAME, "--gateway-address must not be blank."));
        }
        let request = ClaimRequest {
            position_ref: position_ref.clone(),
            agent_ref,
            agency_ref,
            agent_session_ref: parsed.typed(NAME, "--agent-session", |raw: &str| {
                AgentSessionRef::new(raw)
            })?,
            session_space_ref: parsed
                .typed(NAME, "--session-space", |raw: &str| ExternalRef::new(raw))?,
            harness_composition_ref: parsed.typed(NAME, "--harness-composition", |raw: &str| {
                ExternalRef::new(raw)
            })?,
            model_ref: parsed.typed(NAME, "--model", |raw: &str| ExternalRef::new(raw))?,
            workcell_ref: parsed.typed(NAME, "--workcell", |raw: &str| ExternalRef::new(raw))?,
            gateway_address,
            reason,
            expectation,
            kind: parsed.typed(NAME, "--kind", |raw: &str| raw.parse::<TenureKind>())?,
        };
        let store = store(&scope, "claim")?;
        let retry = |retry: ClaimRetry| claim_command(&parsed, retry);
        let outcome = store
            .claim(request)
            .map_err(|error| refusal_for(error, "claim", &scope, Some(&retry)))?;
        let mut value = json!({
            "ok": true,
            "verb": "claim",
            "position_ref": position_ref,
            "generation": outcome.tenure.generation(),
            "tenure": outcome.tenure,
            "env": {
                "OI_POSITION_REF": position_ref,
                "OI_OCCUPANT_GENERATION": outcome.tenure.generation_ref,
            },
            "occupancy": outcome.occupancy,
        });
        if let Some(superseded) = outcome.superseded {
            value["superseded"] = json!(superseded);
        }
        Ok(value)
    };
    Ok(respond(run(), command.json, render_claim))
}

pub fn release(command: &Command) -> Result<Output, Error> {
    const NAME: &str = "occupancy.release";
    let run = || -> Refused<Value> {
        let parsed = Parsed::parse(
            NAME,
            &command.args,
            &["--position", "--generation", "--reason", "--store"],
            &[],
        )?;
        let (position_ref, scope) = position(NAME, &parsed)?;
        let generation = generation(NAME, &parsed)?;
        let reason = reason(NAME, &parsed)?;
        let store = store(&scope, "release")?;
        let outcome = store
            .release(&position_ref, &generation, &reason)
            .map_err(|error| refusal_for(error, "release", &scope, None))?;
        let mut value = json!({
            "ok": true,
            "verb": "release",
            "position_ref": position_ref,
            "generation": outcome.tenure.generation(),
            "tenure": outcome.tenure,
        });
        match outcome.occupancy {
            Some(reading) => value["occupancy"] = json!(reading),
            None => value["still_open"] = json!(outcome.still_open),
        }
        Ok(value)
    };
    Ok(respond(run(), command.json, render_release))
}

pub fn verify(command: &Command) -> Result<Output, Error> {
    const NAME: &str = "occupancy.verify";
    let run = || -> Refused<Value> {
        let parsed = Parsed::parse(
            NAME,
            &command.args,
            &["--position", "--generation", "--store"],
            &[],
        )?;
        let (position_ref, scope) = position(NAME, &parsed)?;
        let generation = generation(NAME, &parsed)?;
        let store = store(&scope, "verify")?;
        let current = store
            .verify(&position_ref, &generation)
            .map_err(|error| refusal_for(error, "verify", &scope, None))?;
        Ok(json!({
            "ok": true,
            "verb": "verify",
            "position_ref": position_ref,
            "generation": current.generation(),
            "current": current,
        }))
    };
    Ok(respond(run(), command.json, render_verify))
}

pub fn presence(command: &Command) -> Result<Output, Error> {
    const NAME: &str = "occupancy.presence";
    let run = || -> Refused<Value> {
        let parsed = Parsed::parse(
            NAME,
            &command.args,
            &[
                "--position",
                "--generation",
                "--presence",
                "--attention",
                "--store",
            ],
            &[],
        )?;
        let (position_ref, scope) = position(NAME, &parsed)?;
        let generation = generation(NAME, &parsed)?;
        let presence = parsed
            .typed(NAME, "--presence", |raw: &str| raw.parse::<Presence>())?
            .ok_or_else(|| usage_refusal(NAME, "occupancy presence requires --presence."))?;
        let attention = parsed.get("--attention").map(str::to_owned);
        let store = store(&scope, "presence")?;
        let recorded = store
            .presence(&position_ref, &generation, presence, attention)
            .map_err(|error| refusal_for(error, "presence", &scope, None))?;
        Ok(json!({
            "ok": true,
            "verb": "presence",
            "position_ref": position_ref,
            "presence": recorded,
        }))
    };
    Ok(respond(run(), command.json, render_presence))
}

pub fn read(command: &Command) -> Result<Output, Error> {
    const NAME: &str = "occupancy.read";
    let run = || -> Refused<Value> {
        let parsed = Parsed::parse(NAME, &command.args, &["--position", "--store"], &[])?;
        let (position_ref, scope) = position(NAME, &parsed)?;
        let store = store(&scope, "read")?;
        let reading = store
            .read(&position_ref)
            .map_err(|error| refusal_for(error, "read", &scope, None))?;
        Ok(json!(reading))
    };
    Ok(respond(run(), command.json, render_read))
}

pub fn list(command: &Command) -> Result<Output, Error> {
    const NAME: &str = "occupancy.list";
    let run = || -> Refused<Value> {
        let parsed = Parsed::parse(NAME, &command.args, &["--store"], &[])?;
        let scope = Scope {
            position: String::new(),
            store_flag: parsed.get("--store").map(str::to_owned),
        };
        let store = store(&scope, "list")?;
        let listing: OccupancyListing = store.list().map_err(|error| {
            let mut refusal = refusal_for(error, "list", &scope, None);
            refusal.0.action = format!(
                "Check the store directory, then re-run: actuation occupancy list{}",
                scope.store_suffix()
            );
            refusal
        })?;
        Ok(json!(listing))
    };
    Ok(respond(run(), command.json, render_list))
}

// ---- human renderings ----

fn text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => "not named".into(),
        other => other.to_string(),
    }
}

fn tenure_line(tenure: &Value) -> String {
    format!(
        "generation {} (ordinal {}, kind {}) — agent {} via agency {}, session {}, since {}",
        text(&tenure["generation_ref"]),
        text(&tenure["generation_ordinal"]),
        text(&tenure["kind"]),
        text(&tenure["agent_ref"]),
        text(&tenure["agency_ref"]),
        text(&tenure["agent_session_ref"]),
        tenure["began_at_unix_ms"]
            .as_u64()
            .map(format_unix_ms)
            .unwrap_or_default()
    )
}

fn render_claim(value: &Value) -> String {
    let mut lines = vec![format!(
        "claimed {}: {}",
        text(&value["position_ref"]),
        tenure_line(&value["tenure"])
    )];
    if value["superseded"].is_object() {
        lines.push(format!("superseded: {}", tenure_line(&value["superseded"])));
    }
    lines.push(format!(
        "env: OI_POSITION_REF={} OI_OCCUPANT_GENERATION={}",
        shell(&text(&value["env"]["OI_POSITION_REF"])),
        text(&value["env"]["OI_OCCUPANT_GENERATION"])
    ));
    lines.join("\n")
}

fn render_release(value: &Value) -> String {
    let mut line = format!(
        "released {} from generation {} (ordinal {})",
        text(&value["position_ref"]),
        text(&value["generation"]["generation_ref"]),
        text(&value["generation"]["ordinal"])
    );
    if let Some(open) = value["still_open"].as_array() {
        line.push_str(&format!(
            "; {} tenures are still open, so the Position remains ambiguous",
            open.len()
        ));
    } else {
        line.push_str(&format!(
            "; it is now {}",
            text(&value["occupancy"]["state"])
        ));
    }
    line
}

fn render_verify(value: &Value) -> String {
    format!(
        "current occupant of {}: {}",
        text(&value["position_ref"]),
        tenure_line(&value["current"])
    )
}

fn render_presence(value: &Value) -> String {
    let presence = &value["presence"];
    let mut line = format!(
        "{}: generation {} is {}",
        text(&value["position_ref"]),
        text(&presence["generation_ref"]),
        text(&presence["presence"])
    );
    if let Some(attention) = presence["attention"].as_str() {
        line.push_str(&format!(" (attention: {attention})"));
    }
    line
}

fn render_read(value: &Value) -> String {
    let position = text(&value["position_ref"]);
    let count = value["generations"].as_array().map(Vec::len).unwrap_or(0);
    let mut lines = Vec::new();
    if value["state"] == json!(OccupancyState::Occupied.as_str()) {
        lines.push(format!(
            "{position}: occupied by {}",
            tenure_line(&value["current"])
        ));
        if let Some(presence) = value["presence"].as_object() {
            lines.push(format!(
                "  presence: {}{}",
                text(&presence["presence"]),
                presence
                    .get("attention")
                    .and_then(Value::as_str)
                    .map(|a| format!(" (attention: {a})"))
                    .unwrap_or_default()
            ));
        }
    } else {
        lines.push(format!("{position}: vacant"));
    }
    if value["predecessor"].is_object() {
        lines.push(format!(
            "  predecessor: {}, ended {}",
            tenure_line(&value["predecessor"]),
            text(&value["predecessor"]["end_kind"])
        ));
    }
    lines.push(format!("  generations: {count}"));
    lines.join("\n")
}

fn render_list(value: &Value) -> String {
    let mut lines = Vec::new();
    let positions = value["positions"].as_array().cloned().unwrap_or_default();
    for entry in &positions {
        if entry["current"].is_object() {
            lines.push(format!(
                "{}  occupied  {}",
                text(&entry["position_ref"]),
                tenure_line(&entry["current"])
            ));
        } else {
            lines.push(format!(
                "{}  vacant  ({} generations)",
                text(&entry["position_ref"]),
                text(&entry["generation_count"])
            ));
        }
    }
    let invalid = value["invalid"].as_array().cloned().unwrap_or_default();
    for entry in &invalid {
        lines.push(format!(
            "invalid  {}  {}: {}",
            text(&entry["path"]),
            text(&entry["code"]),
            text(&entry["error"])
        ));
    }
    lines.push(format!(
        "{} positions, {} invalid ledgers in {}",
        positions.len(),
        invalid.len(),
        text(&value["store"])
    ));
    lines.join("\n")
}
