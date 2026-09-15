//! Actuation's Configuration Plane contribution (oi.configuration-contribution/v1,
//! contract revision configuration-plane/contribution.1, frozen by O-I
//! docs/cradle/09-CONFIGURATION-PLANE.md, #299 C0) and the owner-native
//! mutation transport (`actuation config validate|plan|apply|reset --json`).
//!
//! The law this module implements is Actuation's own, not the plane's: the
//! settings contributed here are declared code, not applied configuration —
//! the same fact `actuation system --json` discloses as `mutable: false` on
//! every subject. Contribution is discoverability, and discoverability is not
//! authority: no verb mutates Actuation state, no verb derives permission
//! from O:I's root position or from the caller's surface, and a mutation
//! request is refused with a structured owner error (`oi.config-error/v1`,
//! non-zero exit), never a silent success. The one authority-gated mutation
//! Actuation knows — metagency configure-agency over its own constitution —
//! is governed exactly as `agency actualise` governs determination: by
//! explicit per-request MetagencyGrant evidence, which the frozen transport
//! grammar carries no channel for.
//!
//! One subject table below drives the contribution document, the verbs and
//! the authority disposition, so the planes cannot drift: what the
//! contribution declares is exactly what the verbs enforce.
use crate::dispatch::{flag_value, Command, Output};
use crate::surface::ACTUATION_CLI_VERSION;
use crate::system::canonical_reading_body;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub const CONFIGURATION_CONTRIBUTION_VERSION: &str = "oi.configuration-contribution/v1";
pub const CONFIGURATION_CONTRACT_REVISION: &str = "configuration-plane/contribution.1";
pub const CONFIG_VALIDATION_VERSION: &str = "oi.config-validation/v1";
pub const CONFIG_PLAN_VERSION: &str = "oi.config-plan/v1";
pub const CONFIG_RECEIPT_VERSION: &str = "oi.config-receipt/v1";
pub const CONFIG_ERROR_VERSION: &str = "oi.config-error/v1";

const OWNER_REF: &str = "actuation";

/// The semantic refusal exit code, matching the house convention for
/// refusals (main.rs maps handler errors to 2). The configuration contract
/// requires only "non-zero"; the house code keeps one meaning for one thing.
pub const REFUSAL_EXIT_CODE: i32 = 2;

/// The subjects Actuation genuinely owns as configuration. Every one is a
/// declared constitutional or store-selection fact compiled into this binary
/// (the v2 disclosure's `declared` axes); none is applied runtime state, so
/// none is writable, profileable or sensitive, and none accepts plan/apply/
/// reset. Keys and section ids map structurally onto the v2 disclosure
/// (09 §17): section_ref ↔ sections[].id, setting_key ↔ settings[].key.
struct Subject {
    section_id: &'static str,
    key: &'static str,
    title: &'static str,
    description: &'static str,
    value_schema: Value,
    default_semantics: &'static str,
    default: Option<Value>,
    /// Source location in the owner's namespace (09 §2.2: a location, never
    /// a command string) — the same provenance paths the v2 disclosure uses.
    native_ref: &'static str,
    /// Whether the subject is part of Actuation's authority constitution, so
    /// that any hypothetical mutation would be a metagency configure-agency
    /// act under Actuation's own law (crates/actuation-core/src/agency.rs).
    authority_governed: bool,
}

fn enum_list(options: &[&str]) -> Value {
    json!({
        "type": "list",
        "items": {
            "type": "enum",
            "options": options.iter().map(|o| json!({"value": o})).collect::<Vec<_>>()
        }
    })
}

const DECLARED_CODE_EFFECT: &str =
    "apply is refused: this subject is declared code, not applied configuration";

/// The subject table is built once (its value schemas are runtime JSON) and
/// shared by the contribution builder and the verbs.
fn subjects() -> &'static [Subject] {
    static SUBJECTS: std::sync::OnceLock<Vec<Subject>> = std::sync::OnceLock::new();
    SUBJECTS.get_or_init(|| {
        vec![
            Subject {
                section_id: "agency",
                key: "agency.determination.kinds",
                title: "Determination kinds",
                description: "The closed vocabulary of determination kinds Actuation's agency contract admits. Declared in the contract, not selected at runtime.",
                value_schema: enum_list(&[
                    "self-differentiation",
                    "delegation",
                    "derivation",
                    "federation",
                ]),
                default_semantics: "constant",
                default: Some(json!([
                    "self-differentiation",
                    "delegation",
                    "derivation",
                    "federation"
                ])),
                native_ref: "crates/actuation-core/src/agency.rs",
                authority_governed: false,
            },
            Subject {
                section_id: "agency",
                key: "agency.world_binding.constraints",
                title: "WorldBinding constraint categories",
                description: "The constraint categories a WorldBinding may carry above technical root agency. Root position inside an Objective Internality implies no sovereignty over them.",
                value_schema: enum_list(&[
                    "human_authored_refs",
                    "security_policy_refs",
                    "evidence_refs",
                    "external_reality_refs",
                ]),
                default_semantics: "constant",
                default: Some(json!([
                    "human_authored_refs",
                    "security_policy_refs",
                    "evidence_refs",
                    "external_reality_refs"
                ])),
                native_ref: "crates/actuation-core/src/agency.rs",
                authority_governed: false,
            },
            Subject {
                section_id: "authority",
                key: "authority.metagency.operations",
                title: "Metagency operations",
                description: "The metagency operations Actuation's constitution names. Authority to exercise them is per-request MetagencyGrant evidence against the governing binding; discoverability here is not authority, and O:I root position confers no Actuation permission.",
                value_schema: enum_list(&[
                    "determine-agency",
                    "configure-agency",
                    "actualise-agency",
                    "reintegrate-return",
                ]),
                default_semantics: "constant",
                default: Some(json!([
                    "determine-agency",
                    "configure-agency",
                    "actualise-agency",
                    "reintegrate-return"
                ])),
                native_ref: "crates/actuation-core/src/agency.rs",
                authority_governed: true,
            },
            Subject {
                section_id: "authority",
                key: "authority.derivation.rule",
                title: "Derivation authority requirement",
                description: "The declared rule that derivation requires explicit actualise-agency authority and an explicitly actualised Agent identity. A constitutional declaration, not a tunable.",
                value_schema: json!({"type": "scalar"}),
                default_semantics: "constant",
                default: Some(json!(
                    "Derivation requires explicit actualise-agency authority and an explicitly actualised Agent identity"
                )),
                native_ref: "crates/actuation-runtime/src/actualisation.rs",
                authority_governed: true,
            },
            Subject {
                section_id: "authority",
                key: "authority.federation.rule",
                title: "Federation authority rule",
                description: "The declared rule that federation cannot silently carry determining authority. A constitutional declaration, not a tunable.",
                value_schema: json!({"type": "scalar"}),
                default_semantics: "constant",
                default: Some(json!(
                    "Federation cannot silently carry determining authority; use an explicit delegation"
                )),
                native_ref: "crates/actuation-core/src/agency.rs",
                authority_governed: true,
            },
            Subject {
                section_id: "activity",
                key: "stream.durable_store",
                title: "Durable stream store",
                description: "Where durable stream files live. The default root is derived from HOME at resolution time; the effective root is caller-supplied per invocation (--store <dir> or ACTUATION_STREAM_STORE). Actuation owns no persisted settings store, so there is no profile-writable root.",
                value_schema: json!({"type": "path", "format": "actuation-stream-store-root"}),
                default_semantics: "computed",
                default: None,
                native_ref: "crates/actuation-stream/src/store.rs",
                authority_governed: false,
            },
            Subject {
                section_id: "return",
                key: "return.modes",
                title: "Return policy modes",
                description: "The Return policy modes Actuation's contract admits: the closure vocabulary of the downward-authority/upward-return circuit.",
                value_schema: enum_list(&["required", "optional", "autonomous-termination"]),
                default_semantics: "constant",
                default: Some(json!(["required", "optional", "autonomous-termination"])),
                native_ref: "crates/actuation-core/src/agency.rs",
                authority_governed: false,
            },
        ]
    })
}

const SECTION_TITLES: &[(&str, &str)] = &[
    ("agency", "Agent / Agency / WorldBinding"),
    ("authority", "Authority / Metagency"),
    ("activity", "Activity / ActuationStream"),
    ("return", "Return"),
];

fn subject(setting_ref: &str) -> Option<&'static Subject> {
    subjects()
        .iter()
        .find(|subject| full_ref(subject) == setting_ref)
}

fn full_ref(subject: &Subject) -> String {
    format!("{OWNER_REF}:{}:{}", subject.section_id, subject.key)
}

fn effect() -> Value {
    json!({"kind": "none", "summary": DECLARED_CODE_EFFECT, "ref": null})
}

/// The 07 §4.5 digest convention, shared with the v2 disclosure: sha256 over
/// the document with every `*_unix_ms` field zeroed and `reading_digest` null.
fn reading_digest(document: &Value) -> String {
    let body = canonical_reading_body(document);
    let encoded = serde_json::to_string(&body).expect("canonical body serialises");
    let mut hasher = Sha256::new();
    hasher.update(encoded.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// The contribution document: bare `oi.configuration-contribution/v1` on
/// stdout. `now_ms` is injectable for hermetic tests.
pub fn build_contribution(now_ms: i64) -> Value {
    let sections: Vec<Value> = SECTION_TITLES
        .iter()
        .map(|(section_id, title)| {
            let settings: Vec<Value> = subjects()
                .iter()
                .filter(|subject| subject.section_id == *section_id)
                .map(contribution_setting)
                .collect();
            json!({"id": section_id, "title": title, "settings": settings})
        })
        .collect();

    let unavailable =
        |reason: &'static str| json!({"availability": "unavailable", "reason": reason});
    let mut document = json!({
        "schema": CONFIGURATION_CONTRIBUTION_VERSION,
        "contract_revision": CONFIGURATION_CONTRACT_REVISION,
        "owner": {
            "owner_ref": OWNER_REF,
            "owner_kind": "product",
            "owner_version": ACTUATION_CLI_VERSION,
            "contribution_command": ["actuation", "config-contribution", "--json"],
            "disclosed_at_unix_ms": now_ms,
            "reading_digest": Value::Null,
            "reading_digest_covers": "07 §4.5 convention",
        },
        "about": "Actuation's declared agency constitution and stream store selection: the determination vocabulary, WorldBinding constraints, metagency authority rules, Return modes and durable-store default the product owns as code. Every subject is declared, none is applied; Actuation performs no settings mutation, and discovery here confers no authority.",
        "sections": sections,
        "operations": {
            "transport": "cli/v1",
            "validate": {"availability": "disclosed", "reason": null},
            "plan": unavailable("Actuation mints no change plans: its settings are declared code and it performs no settings mutation"),
            "apply": unavailable("Actuation executes no settings changes: its settings are declared code and it performs no settings mutation"),
            "reset": unavailable("Declared settings have no applied state to reset: the default is the declaration itself"),
        },
        "availability": {"state": "available", "reason": null},
        "degradations": [],
        "obligations": [
            "Actuation performs no settings mutation this revision: every contributed subject is declared code (system --json discloses mutable:false); plan/apply/reset are refused with structured owner errors, never silently successful.",
            "Authority is per-request MetagencyGrant evidence against the exact governing binding (see agency actualise). Discoverability through this contribution is not authority; O:I root position confers no Actuation permission.",
            "The durable stream store is caller-supplied (--store <dir> / ACTUATION_STREAM_STORE); Actuation owns no persisted settings store that a profile could write.",
        ],
    });
    document["owner"]["reading_digest"] = json!(reading_digest(&document));
    document
}

fn contribution_setting(subject: &Subject) -> Value {
    let mut setting = json!({
        "setting_ref": full_ref(subject),
        "section_ref": subject.section_id,
        "title": subject.title,
        "description": subject.description,
        "value_schema": subject.value_schema,
        "allowed_scopes": [{ "scope_kind": "machine", "scope_ref": null }],
        "writable": false,
        "profileable": false,
        "sensitive": false,
        "default_semantics": subject.default_semantics,
        "effect": effect(),
        "operations": {"validate": true, "plan": false, "apply": false, "reset": false},
        "native_ref": subject.native_ref,
    });
    if let Some(default) = &subject.default {
        setting["default"] = default.clone();
    }
    setting
}

// ---- verb plumbing ----

/// setting_ref grammar (09 §3): owner ':' section ':' dotted key, exactly
/// three components, lowercase product/section tokens.
fn parse_setting_ref(raw: &str) -> Result<(), String> {
    let parts: Vec<&str> = raw.split(':').collect();
    if parts.len() != 3 {
        return Err(format!(
            "setting_ref must parse into exactly three ':'-separated components, got {raw}"
        ));
    }
    let ok_token = |token: &str| {
        !token.is_empty()
            && token
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            && token
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    };
    let ok_key_part = |part: &str| {
        !part.is_empty()
            && part
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            && part
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
    };
    if !ok_token(parts[0]) || !ok_token(parts[1]) {
        return Err(format!(
            "setting_ref owner/section tokens are invalid: {raw}"
        ));
    }
    if parts[2].split('.').any(|part| !ok_key_part(part)) {
        return Err(format!("setting_ref key is invalid: {raw}"));
    }
    Ok(())
}

const SINGULAR_SCOPE_KINDS: &[&str] = &["world", "ground", "machine"];

const KNOWN_SCOPE_KINDS: &[&str] = &[
    "world",
    "ground",
    "project",
    "machine",
    "workcell",
    "agency",
    "agent",
    "session-space",
    "agent-session",
    "provider",
    "connector-relation",
    "invocation",
];

/// Compact scope form (CLI only, never wire): "<kind>:<ref>", ref omitted
/// for singular kinds. Absent --scope defaults to this owner's addressing
/// ground: this machine.
fn parse_scope(raw: Option<&str>) -> Result<Value, String> {
    let Some(raw) = raw else {
        return Ok(json!({"scope_kind": "machine", "scope_ref": Value::Null}));
    };
    let (kind, scope_ref) = match raw.split_once(':') {
        Some((kind, reference)) => {
            if reference.is_empty() {
                return Err(format!("scope {kind} carries an empty scope_ref"));
            }
            (kind, Value::String(reference.to_owned()))
        }
        None => {
            if !SINGULAR_SCOPE_KINDS.contains(&raw) {
                return Err(format!(
                    "scope kind {raw} requires a scope_ref in compact form kind:ref"
                ));
            }
            (raw, Value::Null)
        }
    };
    if !KNOWN_SCOPE_KINDS.contains(&kind) {
        return Err(format!("unknown scope kind {kind}"));
    }
    Ok(json!({"scope_kind": kind, "scope_ref": scope_ref}))
}

/// Every contributed subject is machine-scoped declared code, so the only
/// scope in any subject's allowed_scopes is the singular machine. Addressing
/// at any other declared-kind scope is unsupported_scope — never a fallback.
fn scope_supported(subject: &Subject, scope: &Value) -> bool {
    let _ = subject;
    scope["scope_kind"] == json!("machine") && scope["scope_ref"].is_null()
}

fn config_error(
    code: &str,
    message: &str,
    setting_ref: Option<&str>,
    scope_kind: Option<&str>,
) -> Output {
    let mut error = json!({
        "schema": CONFIG_ERROR_VERSION,
        "error_code": code,
        "message": message,
        "retryable": false,
        "detail_ref": null,
    });
    error["setting_ref"] = setting_ref.map(Value::from).unwrap_or(Value::Null);
    error["scope_kind"] = scope_kind.map(Value::from).unwrap_or(Value::Null);
    Output {
        code: REFUSAL_EXIT_CODE,
        stdout: serde_json::to_string_pretty(&error).expect("error document serialises"),
        stderr: String::new(),
    }
}

fn output_document(document: Value) -> Output {
    Output {
        code: 0,
        stdout: serde_json::to_string_pretty(&document).expect("document serialises"),
        stderr: String::new(),
    }
}

fn now_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// The owner-native authority disposition for a requested settings mutation.
/// Actuation's own law decides, exactly as its native Actions do: the
/// authority-section subjects are part of the constitution, and mutating a
/// constitution is a metagency configure-agency act — never a configuration
/// write, and never derivable from discoverability or surface position.
fn refusal_for(subject: &Subject, setting_ref: &str, scope: &Value) -> Output {
    if subject.authority_governed {
        config_error(
            "not_authorised",
            "mutation of Actuation's authority constitution is a metagency configure-agency act: it requires explicit configure-agency authority evidenced per-request against the exact governing binding (as agency actualise requires determine-agency). The configuration transport carries no grant channel, discoverability is not authority, and O:I root position confers no Actuation permission; Actuation additionally performs no settings mutation this revision.",
            Some(setting_ref),
            scope["scope_kind"].as_str(),
        )
    } else {
        config_error(
            "unsupported_setting",
            "this subject is declared code, not applied configuration: Actuation performs no settings mutation this revision (see the contribution's obligations and system --json mutable:false)",
            Some(setting_ref),
            scope["scope_kind"].as_str(),
        )
    }
}

/// Value-contract check against the subject's declared value_schema. This is
/// a rendering/validation hint (09 §2.3); the owner's native validation
/// remains authoritative — which, for a non-writable subject, always adds
/// the not_writable violation.
fn value_violations(subject: &Subject, value: &Value) -> Vec<Value> {
    let mut violations: Vec<Value> = Vec::new();
    let schema = &subject.value_schema;
    match schema["type"].as_str() {
        Some("scalar") | Some("path") => {
            if !value.is_string() {
                violations.push(violation("invalid_value", "value must be a string"));
            }
        }
        Some("boolean") => {
            if !value.is_boolean() {
                violations.push(violation("invalid_value", "value must be a boolean"));
            }
        }
        Some("integer") | Some("number") => {
            if !value.is_number() {
                violations.push(violation("invalid_value", "value must be a number"));
            }
        }
        Some("list") => match value.as_array() {
            None => violations.push(violation("invalid_value", "value must be an array")),
            Some(items) => {
                let options: Vec<&Value> = schema["items"]["options"]
                    .as_array()
                    .map(|options| options.iter().map(|option| &option["value"]).collect())
                    .unwrap_or_default();
                for item in items {
                    if !options.contains(&item) {
                        violations.push(violation(
                            "invalid_value",
                            "list item is not one of the declared options",
                        ));
                    }
                }
            }
        },
        _ => violations.push(violation(
            "invalid_value",
            "subject value kind is not validated by this owner",
        )),
    }
    // The one owner-native validation answer that always applies: the
    // subject accepts no mutation, so no value can be validly set.
    violations.push(violation("not_writable", DECLARED_CODE_EFFECT));
    violations
}

fn violation(code: &str, message: &str) -> Value {
    json!({"code": code, "message": message, "path": Value::Null})
}

/// flag_value with any argv failure shaped as a structured config error, so
/// the verb handlers can surface every refusal as oi.config-error/v1. A
/// malformed invocation is a caller fault, so it is validation_failed, never
/// internal (nothing inside the owner failed).
fn flag(args: &mut Vec<String>, name: &str) -> Result<Option<String>, Output> {
    flag_value(args, name).map_err(|error| {
        config_error(
            "validation_failed",
            &format!("argument error: {error}"),
            None,
            None,
        )
    })
}

fn requested_value(args: &mut Vec<String>, stdin: &str) -> Result<Value, Output> {
    let inline = flag(args, "--value")?;
    let file = flag(args, "--value-file")?;
    let parse = |text: String, origin: String| {
        serde_json::from_str::<Value>(&text).map_err(|error| {
            config_error(
                "invalid_value",
                &format!("{origin} is not valid JSON: {error}"),
                None,
                None,
            )
        })
    };
    match (inline, file) {
        (Some(raw), None) => parse(raw, "--value".to_owned()),
        (None, Some(path)) => {
            let text = if path == "-" {
                Ok(stdin.to_owned())
            } else {
                std::fs::read_to_string(&path).map_err(|error| {
                    config_error(
                        "invalid_value",
                        &format!("cannot read --value-file {path}: {error}"),
                        None,
                        None,
                    )
                })
            }?;
            parse(text, format!("--value-file {path}"))
        }
        (None, None) => Err(config_error(
            "invalid_value",
            "one of --value <json> or --value-file <path|-> is required",
            None,
            None,
        )),
        (Some(_), Some(_)) => Err(config_error(
            "invalid_value",
            "pass either --value or --value-file, not both",
            None,
            None,
        )),
    }
}

/// Shared addressing for validate/plan/reset: resolve --setting and --scope,
/// refusing with the frozen error codes when the request does not address a
/// contributed subject at a supported scope.
fn address(args: &mut Vec<String>) -> Result<(&'static Subject, String, Value), Output> {
    let setting_ref = flag(args, "--setting")?.ok_or_else(|| {
        config_error(
            "unsupported_setting",
            "--setting <setting_ref> is required",
            None,
            None,
        )
    })?;
    if let Err(reason) = parse_setting_ref(&setting_ref) {
        return Err(config_error(
            "unsupported_setting",
            &reason,
            Some(&setting_ref),
            None,
        ));
    }
    let scope_raw = flag(args, "--scope")?;
    let scope = parse_scope(scope_raw.as_deref()).map_err(|reason| {
        if reason.starts_with("unknown scope kind") {
            config_error(
                "unknown_scope_kind",
                &reason,
                Some(&setting_ref),
                reason.strip_prefix("unknown scope kind "),
            )
        } else {
            config_error("unsupported_scope", &reason, Some(&setting_ref), None)
        }
    })?;
    let Some(resolved) = subject(&setting_ref) else {
        return Err(config_error(
            "unsupported_setting",
            &format!(
                "{setting_ref} is unknown to Actuation's contribution; declared subjects are listed by actuation config-contribution --json"
            ),
            Some(&setting_ref),
            scope["scope_kind"].as_str(),
        ));
    };
    if !scope_supported(resolved, &scope) {
        return Err(config_error(
            "unsupported_scope",
            &format!(
                "{} does not address subjects at scope {}:{ }; Actuation's declared subjects are machine-scoped",
                setting_ref,
                scope["scope_kind"].as_str().unwrap_or_default(),
                scope["scope_ref"].as_str().unwrap_or_default(),
            ),
            Some(&setting_ref),
            scope["scope_kind"].as_str(),
        ));
    }
    Ok((resolved, setting_ref, scope))
}

/// Every configuration refusal is a structured `oi.config-error/v1`
/// document on stdout with a non-zero exit; success documents exit 0. The
/// verb bodies below return `Err(error_document)` for refusals and
/// `Ok(document)` for answers, and the handlers give each shape its exit.
fn finished(outcome: Result<Value, Output>) -> Result<Output, actuation_core::Error> {
    Ok(match outcome {
        Ok(document) => output_document(document),
        Err(refusal) => refusal,
    })
}

/// `client-minted` changeset identity grammar (09 §8): `cs-` plus a globally
/// unique suffix of letters, digits, underscores or hyphens. A receipt can
/// only be minted under a well-formed changeset id.
fn changeset_id_ok(changeset: &str) -> bool {
    changeset.len() > "cs-".len()
        && changeset.starts_with("cs-")
        && changeset["cs-".len()..]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// `config validate --json --setting <ref> [--scope <compact>] (--value <json> | --value-file <path|->)`
///
/// A truthful owner validation answer exits 0 even when `valid` is false —
/// the document is the answer. Addressing failures (unknown setting, unknown
/// or unsupported scope) are structured errors with a non-zero exit.
pub fn config_validate(command: &Command) -> Result<Output, actuation_core::Error> {
    let mut args = command.args.clone();
    finished((|| -> Result<Value, Output> {
        let (resolved, setting_ref, scope) = address(&mut args)?;
        let value = requested_value(&mut args, &command.stdin)?;
        if !args.is_empty() {
            return Err(config_error(
                "validation_failed",
                &format!(
                    "unexpected arguments for config validate: {}",
                    args.join(" ")
                ),
                None,
                None,
            ));
        }
        let violations = value_violations(resolved, &value);
        Ok(json!({
            "schema": CONFIG_VALIDATION_VERSION,
            "setting_ref": setting_ref,
            "scope": scope,
            "valid": violations.is_empty(),
            "violations": violations,
            "expected_effect": effect(),
        }))
    })())
}

/// `config plan --json --setting <ref> [--scope <compact>] (--value <json> | --value-file <path|->)`
///
/// Actuation mints no change plans: every plan request is refused with a
/// structured owner error. The refusal passes through the owner's authority
/// disposition for the addressed subject first, so an authority-governed
/// subject is refused as not_authorised regardless of the value offered.
pub fn config_plan(command: &Command) -> Result<Output, actuation_core::Error> {
    let mut args = command.args.clone();
    finished((|| -> Result<Value, Output> {
        let (resolved, setting_ref, scope) = address(&mut args)?;
        requested_value(&mut args, &command.stdin)?;
        if !args.is_empty() {
            return Err(config_error(
                "validation_failed",
                &format!("unexpected arguments for config plan: {}", args.join(" ")),
                None,
                None,
            ));
        }
        Err(refusal_for(resolved, &setting_ref, &scope))
    })())
}

/// `config apply --json (--plan-file <path|->) [--changeset <id>]`
///
/// The owner-side idempotency key (09 §9) is consulted first: an executed key
/// replays as outcome no_op naming the original receipt, without
/// re-execution. No plan is ever owner-minted — the contribution declares
/// apply unavailable — so a well-formed plan for a non-governed subject is
/// still refused as validation_failed: the presented plan_digest was not
/// issued by this owner. The authority disposition of the plan's subject is
/// honoured before anything mechanical, and a plan whose schema is not
/// oi.config-plan/v1 is refused as unsupported_schema.
pub fn config_apply(command: &Command) -> Result<Output, actuation_core::Error> {
    let mut args = command.args.clone();
    finished((|| -> Result<Value, Output> {
        let plan_path = flag(&mut args, "--plan-file")?;
        let changeset = flag(&mut args, "--changeset")?;
        if !args.is_empty() {
            return Err(config_error(
                "validation_failed",
                &format!("unexpected arguments for config apply: {}", args.join(" ")),
                None,
                None,
            ));
        }
        let Some(plan_path) = plan_path else {
            return Err(config_error(
                "validation_failed",
                "config apply requires --plan-file <path|-> carrying an owner-minted plan; Actuation mints no plans because it performs no settings mutation",
                None,
                None,
            ));
        };
        let plan_text = if plan_path == "-" {
            Ok(command.stdin.to_owned())
        } else {
            std::fs::read_to_string(&plan_path).map_err(|error| {
                config_error(
                    "validation_failed",
                    &format!("cannot read --plan-file {plan_path}: {error}"),
                    None,
                    None,
                )
            })
        }?;
        let plan: Value = serde_json::from_str(&plan_text).map_err(|error| {
            config_error(
                "validation_failed",
                &format!("--plan-file is not valid JSON: {error}"),
                None,
                None,
            )
        })?;
        if plan["schema"] != json!(CONFIG_PLAN_VERSION) {
            return Err(config_error(
                "unsupported_schema",
                &format!(
                    "a plan must carry schema {}; this owner cannot interpret {}",
                    CONFIG_PLAN_VERSION,
                    plan["schema"].as_str().unwrap_or("(absent)")
                ),
                None,
                None,
            ));
        }
        let setting_ref = plan["setting_ref"].as_str().unwrap_or_default().to_owned();
        let scope = plan["scope"].clone();
        let plan_digest = plan["plan_digest"].as_str().map(str::to_owned);
        if let Some(resolved) = subject(&setting_ref) {
            // The authority disposition precedes everything mechanical: a
            // constitution-governed subject is not_authorised however well
            // the plan is formed.
            if resolved.authority_governed {
                return Err(refusal_for(resolved, &setting_ref, &scope));
            }
        } else if setting_ref.is_empty() {
            return Err(config_error(
                "validation_failed",
                "the plan carries no setting_ref; this owner cannot interpret it",
                None,
                scope["scope_kind"].as_str(),
            ));
        } else {
            return Err(config_error(
                "unsupported_setting",
                &format!(
                    "{setting_ref} is unknown to Actuation's contribution; declared subjects are listed by actuation config-contribution --json"
                ),
                Some(&setting_ref),
                scope["scope_kind"].as_str(),
            ));
        }
        if let Some(changeset) = &changeset {
            if !changeset_id_ok(changeset) {
                return Err(config_error(
                    "validation_failed",
                    &format!(
                        "--changeset must be a client-minted `cs-<suffix>` id (09 §8), got {changeset}"
                    ),
                    Some(&setting_ref),
                    scope["scope_kind"].as_str(),
                ));
            }
            let key = IdempotencyKey {
                changeset_id: changeset.clone(),
                setting_ref: setting_ref.clone(),
                scope: scope.clone(),
                plan_digest: plan_digest.clone(),
            };
            if let Some(original) =
                ReceiptLedger::from_home().and_then(|ledger| ledger.original_receipt(&key))
            {
                return Ok(no_op_receipt(&key, original, now_unix_ms()));
            }
        }
        Err(config_error(
            "validation_failed",
            &format!(
                "no owner-minted plan: Actuation mints no change plans because it performs no settings mutation; plan_digest {} was not issued by this owner",
                plan_digest.as_deref().unwrap_or("(absent)")
            ),
            Some(&setting_ref),
            scope["scope_kind"].as_str(),
        ))
    })())
}

/// `config reset --json --setting <ref> [--scope <compact>] [--changeset <id>]`
///
/// Declared settings have no applied state to reset — the default is the
/// declaration itself — so every reset request is refused after the owner's
/// authority disposition for the subject is honoured. An executed reset key
/// still replays as no_op under the idempotency contract.
pub fn config_reset(command: &Command) -> Result<Output, actuation_core::Error> {
    let mut args = command.args.clone();
    finished((|| -> Result<Value, Output> {
        let (resolved, setting_ref, scope) = address(&mut args)?;
        let changeset = flag(&mut args, "--changeset")?;
        if !args.is_empty() {
            return Err(config_error(
                "validation_failed",
                &format!("unexpected arguments for config reset: {}", args.join(" ")),
                None,
                None,
            ));
        }
        if let Some(changeset) = &changeset {
            if !changeset_id_ok(changeset) {
                return Err(config_error(
                    "validation_failed",
                    &format!(
                        "--changeset must be a client-minted `cs-<suffix>` id (09 §8), got {changeset}"
                    ),
                    Some(&setting_ref),
                    scope["scope_kind"].as_str(),
                ));
            }
            let key = IdempotencyKey {
                changeset_id: changeset.clone(),
                setting_ref: setting_ref.clone(),
                scope: scope.clone(),
                plan_digest: None,
            };
            if let Some(original) =
                ReceiptLedger::from_home().and_then(|ledger| ledger.original_receipt(&key))
            {
                return Ok(no_op_receipt(&key, original, now_unix_ms()));
            }
        }
        Err(refusal_for(resolved, &setting_ref, &scope))
    })())
}

/// The idempotency ledger: the owner-side record of executed configuration
/// receipts (09 §9). The key is exactly the frozen one — (owner_ref,
/// changeset_id, setting_ref, scope, plan_digest); a replay of an executed
/// key MUST return outcome no_op naming the original receipt, and the owner
/// must not re-execute. The ledger is appended only when a configuration
/// operation actually executes — which, with every subject declared code, is
/// never on the CLI path today; the enforcement is nonetheless real, is
/// proven end-to-end in the sandbox tests, and lives under the owner's own
/// home (`~/.actuation/config-receipts.jsonl`) if a revision ever executes.
pub struct ReceiptLedger {
    root: std::path::PathBuf,
}

/// The frozen idempotency key (09 §9). The owner part is this owner by
/// construction; the operation is not part of the key — the original
/// receipt's own operation is what a replay echoes.
pub struct IdempotencyKey {
    pub changeset_id: String,
    pub setting_ref: String,
    pub scope: Value,
    pub plan_digest: Option<String>,
}

impl ReceiptLedger {
    pub fn from_home() -> Option<Self> {
        std::env::var_os("HOME").map(|home| Self {
            root: std::path::Path::new(&home).join(".actuation/config-receipts.jsonl"),
        })
    }

    pub fn at_root(root: std::path::PathBuf) -> Self {
        Self { root }
    }

    fn key_matches(entry: &Value, key: &IdempotencyKey) -> bool {
        entry["owner_ref"] == json!(OWNER_REF)
            && entry["changeset_id"] == json!(key.changeset_id)
            && entry["setting_ref"] == json!(key.setting_ref)
            && entry["scope"] == key.scope
            && entry["plan_digest"]
                == key
                    .plan_digest
                    .clone()
                    .map(Value::from)
                    .unwrap_or(Value::Null)
            && entry["outcome"] == json!("applied")
    }

    /// The executed receipt for this key, if any: the full entry, so the
    /// replay can echo the original's operation, native ref and effect.
    pub fn original_receipt(&self, key: &IdempotencyKey) -> Option<Value> {
        let raw = std::fs::read_to_string(&self.root).ok()?;
        for line in raw.lines() {
            let Ok(entry) = serde_json::from_str::<Value>(line) else {
                continue;
            };
            if Self::key_matches(&entry, key) {
                return Some(entry);
            }
        }
        None
    }

    pub fn record(&self, receipt: &Value) -> std::io::Result<()> {
        use std::io::Write;
        if let Some(parent) = self.root.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.root)?;
        writeln!(
            file,
            "{}",
            serde_json::to_string(receipt)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?
        )
    }

    pub fn len(&self) -> usize {
        std::fs::read_to_string(&self.root)
            .map(|raw| raw.lines().filter(|line| !line.trim().is_empty()).count())
            .unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn key_digest(key: &IdempotencyKey) -> String {
    let mut hasher = Sha256::new();
    hasher.update(OWNER_REF.as_bytes());
    hasher.update(key.changeset_id.as_bytes());
    hasher.update(key.setting_ref.as_bytes());
    hasher.update(key.scope.to_string().as_bytes());
    hasher.update(key.plan_digest.as_deref().unwrap_or("").as_bytes());
    format!("{:x}", hasher.finalize())
}

/// The no_op replay receipt (09 §9): a fresh owner-minted id for the replay
/// itself, the original's operation/native ref/effect echoed, outcome no_op,
/// and original_receipt_id naming the executed receipt.
fn no_op_receipt(key: &IdempotencyKey, original: Value, now_ms: i64) -> Value {
    json!({
        "schema": CONFIG_RECEIPT_VERSION,
        "receipt_id": format!("actuation-config-replay-{}", &key_digest(key)[..16]),
        "owner_ref": OWNER_REF,
        "changeset_id": key.changeset_id,
        "plan_digest": key.plan_digest,
        "setting_ref": key.setting_ref,
        "scope": key.scope,
        "operation": original["operation"],
        "outcome": "no_op",
        "applied_at_unix_ms": now_ms,
        "native_ref": original["native_ref"],
        "expected_effect": original["expected_effect"],
        "original_receipt_id": original["receipt_id"],
        "error": null,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::execute;

    fn argv(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn contribution_carries_only_declared_non_writable_subjects() {
        let document = build_contribution(1_000);
        assert_eq!(
            document["schema"],
            json!(CONFIGURATION_CONTRIBUTION_VERSION)
        );
        assert_eq!(
            document["contract_revision"],
            json!(CONFIGURATION_CONTRACT_REVISION)
        );
        assert_eq!(document["owner"]["owner_ref"], json!("actuation"));
        assert_eq!(document["owner"]["owner_kind"], json!("product"));
        for section in document["sections"].as_array().unwrap() {
            for setting in section["settings"].as_array().unwrap() {
                assert_eq!(
                    setting["writable"],
                    json!(false),
                    "{}",
                    setting["setting_ref"]
                );
                assert_eq!(setting["profileable"], json!(false));
                assert_eq!(setting["sensitive"], json!(false));
                assert_eq!(setting["operations"]["validate"], json!(true));
                assert_eq!(setting["operations"]["plan"], json!(false));
                assert_eq!(setting["operations"]["apply"], json!(false));
                assert_eq!(setting["operations"]["reset"], json!(false));
            }
        }
        // The digest is a function of the reading, not the clock (07 §4.5).
        let later = build_contribution(2_000);
        assert_eq!(
            document["owner"]["reading_digest"],
            later["owner"]["reading_digest"]
        );
        assert_ne!(
            document["owner"]["disclosed_at_unix_ms"],
            later["owner"]["disclosed_at_unix_ms"]
        );
    }

    #[test]
    fn setting_refs_parse_by_the_frozen_grammar() {
        assert!(parse_setting_ref("actuation:agency:agency.determination.kinds").is_ok());
        assert!(parse_setting_ref("actuation:agency").is_err());
        assert!(parse_setting_ref("actuation:agency:a:b").is_err());
        assert!(parse_setting_ref("Actuation:agency:rule").is_err());
        assert!(parse_setting_ref("actuation:agency:Bad_Key").is_err());
    }

    #[test]
    fn compact_scopes_resolve_with_singular_default() {
        let scope = parse_scope(None).unwrap();
        assert_eq!(
            scope,
            json!({"scope_kind": "machine", "scope_ref": Value::Null})
        );
        assert!(parse_scope(Some("machine")).is_ok());
        assert!(parse_scope(Some("project:epilogos/o-i")).is_ok());
        assert!(
            parse_scope(Some("project")).is_err(),
            "non-singular kinds need a ref"
        );
        assert!(
            parse_scope(Some("galaxy:x")).is_err(),
            "unknown kinds are refused"
        );
    }

    #[test]
    fn validate_answers_truthfully_and_addresses_explicitly() {
        let ok = execute(
            &argv(&[
                "config",
                "validate",
                "--json",
                "--setting",
                "actuation:return:return.modes",
                "--value",
                "[\"required\"]",
            ]),
            "",
        )
        .unwrap();
        assert_eq!(ok.code, 0, "a validation answer is an answer");
        let document: Value = serde_json::from_str(&ok.stdout).unwrap();
        assert_eq!(document["schema"], json!(CONFIG_VALIDATION_VERSION));
        assert_eq!(document["valid"], json!(false));
        let codes: Vec<&str> = document["violations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v["code"].as_str().unwrap())
            .collect();
        assert!(codes.contains(&"not_writable"));

        let unknown = execute(
            &argv(&[
                "config",
                "validate",
                "--json",
                "--setting",
                "actuation:nope:not.a.thing",
                "--value",
                "1",
            ]),
            "",
        )
        .unwrap();
        assert_eq!(unknown.code, REFUSAL_EXIT_CODE);
        let error: Value = serde_json::from_str(&unknown.stdout).unwrap();
        assert_eq!(error["schema"], json!(CONFIG_ERROR_VERSION));
        assert_eq!(error["error_code"], json!("unsupported_setting"));
    }

    #[test]
    fn plan_refuses_with_the_authority_law_on_constitution_subjects() {
        let refused = execute(
            &argv(&[
                "config",
                "plan",
                "--json",
                "--setting",
                "actuation:authority:authority.derivation.rule",
                "--value",
                "\"rewritten\"",
            ]),
            "",
        )
        .unwrap();
        assert_eq!(refused.code, REFUSAL_EXIT_CODE);
        let error: Value = serde_json::from_str(&refused.stdout).unwrap();
        assert_eq!(error["error_code"], json!("not_authorised"));
        let message = error["message"].as_str().unwrap();
        assert!(message.contains("configure-agency"), "{message}");
        assert!(
            message.contains("discoverability is not authority"),
            "{message}"
        );

        let plain = execute(
            &argv(&[
                "config",
                "plan",
                "--json",
                "--setting",
                "actuation:return:return.modes",
                "--value",
                "[\"optional\"]",
            ]),
            "",
        )
        .unwrap();
        assert_eq!(plain.code, REFUSAL_EXIT_CODE);
        let error: Value = serde_json::from_str(&plain.stdout).unwrap();
        assert_eq!(error["error_code"], json!("unsupported_setting"));
    }

    #[test]
    fn ledger_replays_executed_keys_as_no_op_without_reexecution() {
        let sandbox = tempfile::tempdir().unwrap();
        let ledger = ReceiptLedger::at_root(sandbox.path().join("receipts.jsonl"));
        assert!(ledger.is_empty());
        let key = IdempotencyKey {
            changeset_id: "cs-test-replay-1".to_owned(),
            setting_ref: "actuation:activity:stream.durable_store".to_owned(),
            scope: json!({"scope_kind": "machine", "scope_ref": Value::Null}),
            plan_digest: Some("abc123".to_owned()),
        };
        assert!(ledger.original_receipt(&key).is_none());
        ledger
            .record(&json!({
                "schema": CONFIG_RECEIPT_VERSION,
                "receipt_id": "actuation-config-receipt-1",
                "owner_ref": "actuation",
                "changeset_id": key.changeset_id,
                "plan_digest": key.plan_digest,
                "setting_ref": key.setting_ref,
                "scope": key.scope,
                "operation": "apply",
                "outcome": "applied",
                "applied_at_unix_ms": 1,
                "native_ref": "actuation:config:receipts:1",
                "expected_effect": {"kind": "none", "summary": null, "ref": null},
                "original_receipt_id": null,
                "error": null,
            }))
            .unwrap();
        let original = ledger.original_receipt(&key).unwrap();
        assert_eq!(original["receipt_id"], json!("actuation-config-receipt-1"));
        let replay = no_op_receipt(&key, original, 2);
        assert_eq!(replay["schema"], json!(CONFIG_RECEIPT_VERSION));
        assert_eq!(replay["outcome"], json!("no_op"));
        assert_eq!(replay["operation"], json!("apply"));
        assert_eq!(
            replay["original_receipt_id"],
            json!("actuation-config-receipt-1")
        );
        assert_eq!(replay["native_ref"], json!("actuation:config:receipts:1"));
        assert_eq!(ledger.len(), 1, "the replay did not re-execute or append");

        // A different changeset is a different key: no replay, no receipt.
        let other = IdempotencyKey {
            changeset_id: "cs-other-2".to_owned(),
            ..key
        };
        assert!(ledger.original_receipt(&other).is_none());
    }

    fn executed_entry(changeset: &str, digest: &str, setting: &str) -> Value {
        json!({
            "schema": CONFIG_RECEIPT_VERSION,
            "receipt_id": "actuation-config-receipt-9",
            "owner_ref": "actuation",
            "changeset_id": changeset,
            "plan_digest": digest,
            "setting_ref": setting,
            "scope": {"scope_kind": "machine", "scope_ref": null},
            "operation": "apply",
            "outcome": "applied",
            "applied_at_unix_ms": 1,
            "native_ref": "actuation:config:receipts:9",
            "expected_effect": {"kind": "none", "summary": null, "ref": null},
            "original_receipt_id": null,
            "error": null,
        })
    }

    #[test]
    fn executed_entry_keys_only_match_their_own_key() {
        let sandbox = tempfile::tempdir().unwrap();
        let ledger = ReceiptLedger::at_root(sandbox.path().join("receipts.jsonl"));
        let setting = "actuation:activity:stream.durable_store";
        ledger
            .record(&executed_entry("cs-key-1", "abc123def456", setting))
            .unwrap();
        let hit = |changeset: &str, digest: Option<&str>, subject: &str| {
            ledger
                .original_receipt(&IdempotencyKey {
                    changeset_id: changeset.to_owned(),
                    setting_ref: subject.to_owned(),
                    scope: json!({"scope_kind": "machine", "scope_ref": Value::Null}),
                    plan_digest: digest.map(str::to_owned),
                })
                .is_some()
        };
        assert!(hit("cs-key-1", Some("abc123def456"), setting));
        assert!(
            !hit("cs-key-2", Some("abc123def456"), setting),
            "changeset is in the key"
        );
        assert!(
            !hit("cs-key-1", Some("other"), setting),
            "plan_digest is in the key"
        );
        assert!(
            !hit("cs-key-1", None, setting),
            "an absent digest is its own key value"
        );
        assert!(
            !hit(
                "cs-key-1",
                Some("abc123def456"),
                "actuation:return:return.modes"
            ),
            "setting_ref is in the key"
        );
        assert!(
            !hit("cs-key-1", Some("abc123def456"), "actuation:nope:other.key"),
            "an unknown subject never replays"
        );
    }

    #[test]
    fn apply_honours_authority_and_schema_before_anything_mechanical() {
        let governed_plan = |schema: &str, setting: &str| {
            json!({
                "schema": schema,
                "plan_id": "forged",
                "plan_digest": "deadbeef",
                "setting_ref": setting,
                "scope": {"scope_kind": "machine", "scope_ref": null},
                "changes": [{"summary": "forged"}],
                "expected_effect": {"kind": "none", "summary": null, "ref": null},
            })
        };
        // A constitution-governed subject is not_authorised however well the
        // plan is formed: authority precedes mechanics.
        let refused = config_apply(&Command {
            args: argv(&["--plan-file", "-", "--changeset", "cs-auth-1"]),
            json: true,
            stdin: serde_json::to_string(&governed_plan(
                CONFIG_PLAN_VERSION,
                "actuation:authority:authority.federation.rule",
            ))
            .unwrap(),
        })
        .unwrap();
        assert_eq!(refused.code, REFUSAL_EXIT_CODE);
        let error: Value = serde_json::from_str(&refused.stdout).unwrap();
        assert_eq!(error["error_code"], json!("not_authorised"));

        // A plan in a foreign schema is unsupported_schema, never interpreted.
        let refused = config_apply(&Command {
            args: argv(&["--plan-file", "-"]),
            json: true,
            stdin: serde_json::to_string(&governed_plan(
                "oi.config-plan/v9",
                "actuation:return:return.modes",
            ))
            .unwrap(),
        })
        .unwrap();
        assert_eq!(refused.code, REFUSAL_EXIT_CODE);
        let error: Value = serde_json::from_str(&refused.stdout).unwrap();
        assert_eq!(error["error_code"], json!("unsupported_schema"));

        // A well-formed plan naming an unknown subject is unsupported_setting.
        let refused = config_apply(&Command {
            args: argv(&["--plan-file", "-", "--changeset", "cs-unknown-1"]),
            json: true,
            stdin: serde_json::to_string(&governed_plan(
                CONFIG_PLAN_VERSION,
                "actuation:return:no.such.key",
            ))
            .unwrap(),
        })
        .unwrap();
        assert_eq!(refused.code, REFUSAL_EXIT_CODE);
        let error: Value = serde_json::from_str(&refused.stdout).unwrap();
        assert_eq!(error["error_code"], json!("unsupported_setting"));

        // A malformed changeset id can never anchor a receipt.
        let refused = config_apply(&Command {
            args: argv(&["--plan-file", "-", "--changeset", "not-a-changeset"]),
            json: true,
            stdin: serde_json::to_string(&governed_plan(
                CONFIG_PLAN_VERSION,
                "actuation:return:return.modes",
            ))
            .unwrap(),
        })
        .unwrap();
        assert_eq!(refused.code, REFUSAL_EXIT_CODE);
        let error: Value = serde_json::from_str(&refused.stdout).unwrap();
        assert_eq!(error["error_code"], json!("validation_failed"));
    }
}
