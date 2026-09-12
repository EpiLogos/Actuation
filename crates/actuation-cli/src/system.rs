//! Actuation's Wave 5 System contribution: the native settings disclosure
//! document (oi.product-settings-disclosure/v2, wave-5/system.1).
//!
//! This module is a READ-ONLY projection. It never mutates state, never
//! fabricates a live value, and never reaches past the owner seam. It derives
//! every fact from the command surface (`surface.rs`), the contract version
//! constants, and one live harness-detection pass — the same read-only probes
//! `actuation harness detect` runs. There is no settings database, no
//! duplicate action catalogue and no duplicate provider registry here: the
//! actions are a disclosure-grade projection of the single command table,
//! annotated with the seam metadata (availability / exposure / authority) the
//! command table does not carry.
use crate::surface::{
    cli_surface, ACTIVITY_VERSION, ACTUATION_CLI_VERSION, ACTUATION_STREAM_VERSION,
    AGENCY_CONTRACT_VERSION, SYSTEM_DISCLOSURE_CONTRACT_REVISION, SYSTEM_DISCLOSURE_VERSION,
};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

const PRODUCT_ID: &str = "actuation";
const OWNER_REF: &str = "actuation:cli";
const READING_COMMAND: &[&str] = &["actuation", "system", "--json"];

// provenance.path is a location, not a command (§4.6). Each owner-namespace
// ref names the real native source file the value comes from; the command that
// reads it belongs in native_path, never in provenance.path.
const PATH_SURFACE: &str = "crates/actuation-cli/src/surface.rs";
const PATH_COMMANDS: &str = "crates/actuation-cli/src/dispatch.rs";
const PATH_SYSTEM: &str = "crates/actuation-cli/src/system.rs";
const PATH_AGENCY: &str = "crates/actuation-core/src/agency.rs";
const PATH_AGENCY_ACTUALISATION: &str = "crates/actuation-runtime/src/actualisation.rs";
const PATH_CATALOG: &str = "catalog/targets.json";
const PATH_DETECT: &str = "crates/actuation-adapters/src/observation.rs";
const PATH_SELF: &str = "crates/actuation-adapters/src/observation.rs";
const PATH_STREAM_STORE: &str = "crates/actuation-stream/src/store.rs";

// Authority is never inferred from UI location or root-agent identity. The
// read/verify actions below require no authority (they are read-only and the
// owner self-authorises them); the single actualising action names its exact
// MetagencyGrant authority and is refused without it.
fn no_authority() -> Value {
    json!({"requires": [], "granted_by": "actuation:cli", "evidence_ref": null})
}

fn provenance(owner_ref: &str, path: &str, observed_at_ms: i64) -> Value {
    json!({"owner_ref": owner_ref, "path": path, "observed_at_unix_ms": observed_at_ms})
}

fn axis(value: Value, prov: Value) -> Value {
    json!({"value": value, "provenance": prov})
}

fn staged_axis(prov: Value) -> Value {
    json!({"value": null, "provenance": prov, "stage_ref": null, "stage_state": "none"})
}

fn expected_effect() -> Value {
    json!({
        "summary": "No stage prepared; this setting is declared code, not applied configuration.",
        "ref": null
    })
}

fn drift(state: &str) -> Value {
    json!({"state": state, "between": ["declared", "effective"], "remediation_action_ref": null})
}

/// §4.5: the canonical reading body is the whole descriptor with every
/// *_unix_ms field zeroed and reading_digest itself held null. Hashing that
/// body makes the digest a function of the reading, never of the clock.
pub fn canonical_reading_body(descriptor: &Value) -> Value {
    fn zero_clock(node: &mut Value) {
        match node {
            Value::Array(items) => {
                for item in items {
                    zero_clock(item);
                }
            }
            Value::Object(entries) => {
                for (key, value) in entries.iter_mut() {
                    if key.ends_with("_unix_ms") {
                        *value = json!(0);
                    } else {
                        zero_clock(value);
                    }
                }
            }
            _ => {}
        }
    }
    let mut body = descriptor.clone();
    zero_clock(&mut body);
    body["owner"]["reading_digest"] = Value::Null;
    body
}

fn reading_digest(descriptor: &Value) -> String {
    let body = canonical_reading_body(descriptor);
    let encoded = serde_json::to_string(&body).expect("canonical body serialises");
    let mut hasher = Sha256::new();
    hasher.update(encoded.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// §4.7: availability is probed, not asserted. Owner-level availability is
/// derived from the live detection pass; a partial detection (a probe that
/// failed, not a clean absence) degrades the reading.
pub fn derive_availability(detection: &Value) -> Value {
    if detection["availability"] == "complete" {
        json!({"state": "available", "reason": null})
    } else {
        json!({
            "state": "degraded",
            "reason": "harness detection is partial: at least one probe failed (see degradations)"
        })
    }
}

/// §4.3: per-subject degradations are separate from owner-level availability,
/// and are derived from the same live detection pass, never a hardcoded list.
/// A harness whose probes failed is unavailable with a reason; a clean absence
/// (not-installed) is not a degradation of this owner.
pub fn derive_degradations(detection: &Value) -> Value {
    let harnesses = detection["harnesses"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    Value::Array(
        harnesses
            .iter()
            .filter(|entry| entry["state"] == "unavailable")
            .map(|entry| {
                json!({
                    "subject_ref": entry["harness_ref"],
                    "state": "unavailable",
                    "reason": entry["unavailable_reason"]
                        .as_str()
                        .unwrap_or("all probes failed; could not run"),
                    "native_error": null,
                })
            })
            .collect(),
    )
}

/// The durable stream store's active state is a real filesystem observation:
/// whether the effective default root exists, and how many streams in it are
/// genuinely open. Openness is read from each stream's own lifecycle state —
/// the header line of its durable file — never inferred from the presence of
/// a `.jsonl` file. The authored default root lives in the declared axis; the
/// env-overridden root is the effective axis.
fn default_stream_store_state() -> Value {
    let default_root = match home_streams() {
        Some(path) => path,
        None => {
            return json!({
                "root": Value::Null,
                "exists": null,
                "in_use": false,
                "open_streams": 0,
                "closed_streams": 0,
                "reason": "default stream store root is not observable on this machine",
            });
        }
    };
    let root = default_root.to_string_lossy().into_owned();
    let base = |exists: Value| json!({"root": root, "exists": exists, "in_use": Value::Null, "open_streams": 0, "closed_streams": 0});
    let mut value = match std::fs::metadata(&default_root) {
        Ok(meta) => base(json!(meta.is_dir())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let mut absent = base(json!(false));
            absent["in_use"] = json!(false);
            return absent;
        }
        Err(_) => {
            let mut obscured = base(Value::Null);
            obscured["reason"] =
                json!("default stream store root is not observable on this machine");
            return obscured;
        }
    };
    let entries = match std::fs::read_dir(&default_root) {
        Ok(entries) => entries,
        Err(error) => {
            value["in_use"] = Value::Null;
            value["reason"] = json!(format!(
                "default stream store root exists but is not readable: {error}"
            ));
            return value;
        }
    };
    let mut names: Vec<std::path::PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|ext| ext == "jsonl").unwrap_or(false))
        .collect();
    names.sort();
    // Each `.jsonl` file is one durable stream. Its lifecycle.state is the
    // only truth about whether it is open: count genuinely open streams, and
    // treat a stream whose lifecycle cannot be determined as neither open nor
    // closed, naming it instead of guessing.
    let mut undetermined: Vec<String> = Vec::new();
    let (mut open, mut closed) = (0usize, 0usize);
    for path in names {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let reading = std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|raw| {
                raw.lines()
                    .next()
                    .ok_or_else(|| "stream file is empty".to_owned())
                    .and_then(|header| {
                        serde_json::from_str::<Value>(header)
                            .map_err(|e| e.to_string())
                            .and_then(|stream| {
                                stream["lifecycle"]["state"]
                                    .as_str()
                                    .map(str::to_owned)
                                    .ok_or_else(|| {
                                        "stream lifecycle state is unreadable".to_owned()
                                    })
                            })
                    })
            });
        match reading {
            Ok(state) if state == "open" => open += 1,
            Ok(_) => closed += 1,
            Err(reason) => undetermined.push(format!("{name} ({reason})")),
        }
    }
    value["open_streams"] = json!(open);
    value["closed_streams"] = json!(closed);
    value["in_use"] = json!(open > 0);
    if !undetermined.is_empty() {
        value["undetermined_streams"] = json!(undetermined.len());
        value["reason"] = json!(format!(
            "{} stream(s) lifecycle undetermined; counted as neither open nor closed: {}",
            undetermined.len(),
            undetermined.join("; ")
        ));
    }
    value
}

// A declared/effective/active triple over a static contract fact: the value is
// authored in code and validated live, but nothing is materialised at runtime
// (Actuation performs no materialisation by design). All three axes are still
// present — the agreement (and the empty active) is itself the information.
fn contract_setting(
    key: &str,
    title: &str,
    kind: &str,
    declared_value: Value,
    declared_path: &str,
    effective_owner: &str,
    observed_at_ms: i64,
) -> Value {
    let declared = axis(
        declared_value.clone(),
        provenance(OWNER_REF, declared_path, observed_at_ms),
    );
    let effective = axis(
        declared_value,
        provenance(effective_owner, PATH_SURFACE, observed_at_ms),
    );
    let active = axis(
        Value::Null,
        provenance("actuation:cli:system", PATH_SYSTEM, observed_at_ms),
    );
    setting(key, title, kind, declared, effective, active)
}

fn vocabulary_setting(
    key: &str,
    title: &str,
    declared_value: Value,
    declared_path: &str,
    effective_owner: &str,
    observed_at_ms: i64,
) -> Value {
    let declared = axis(
        declared_value.clone(),
        provenance(OWNER_REF, declared_path, observed_at_ms),
    );
    let effective = axis(
        declared_value,
        provenance(effective_owner, PATH_AGENCY, observed_at_ms),
    );
    let active = axis(
        json!([]),
        provenance("actuation:cli:system", PATH_SYSTEM, observed_at_ms),
    );
    setting(key, title, "table", declared, effective, active)
}

fn setting(
    key: &str,
    title: &str,
    kind: &str,
    declared: Value,
    effective: Value,
    active: Value,
) -> Value {
    json!({
        "key": key,
        "title": title,
        "kind": kind,
        "axes": {
            "declared": declared,
            "effective": effective,
            "active": active,
            "staged": null,
            "expected_effect": null,
        },
        "mutable": false,
        "native_path": "actuation (read-only projection; no mutation disclosed)",
        "bootstrap": false,
        "drift": drift("none"),
    })
}

fn finish_setting(setting: Value, declared_path: &str, observed_at_ms: i64) -> Value {
    let mut value = setting;
    value["axes"]["staged"] = staged_axis(provenance(OWNER_REF, declared_path, observed_at_ms));
    value["axes"]["expected_effect"] = expected_effect();
    value
}

fn headless() -> Value {
    json!({"ui": false, "agent": true, "headless": true})
}

fn native_only() -> Value {
    json!({"ui": false, "agent": false, "headless": true})
}

fn action(input: Value) -> Value {
    json!({
        "action_ref": input["action_ref"],
        "title": input["title"],
        "args": input["args"],
        "availability": input["availability"],
        "unavailable_reason": input["unavailable_reason"],
        "subject_kinds": input["subject_kinds"],
        "authority": input["authority"],
        "exposure": input["exposure"],
        "explain": {"ref": null, "command": null},
        "history": {"ref": null, "command": null},
    })
}

fn read_action(action_ref: &str, title: &str, args: Value, subject_kinds: Value) -> Value {
    action(json!({
        "action_ref": action_ref,
        "title": title,
        "args": args,
        "availability": "disclosed",
        "unavailable_reason": null,
        "subject_kinds": subject_kinds,
        "authority": no_authority(),
        "exposure": headless(),
    }))
}

/// The canonical Actions. The five states are deliberately kept distinct and
/// named: exists (a command is in the table), selected (a surface picks it for
/// a subject — not a fact this owner asserts), exposed (exposure.ui/agent/
/// headless), authorised (authority.requires/granted_by), invoked (a receipt
/// returns). This list is a projection of the command table, not a second
/// catalogue: the test suite asserts every disclosed action_ref matches a
/// command name.
fn canonical_actions() -> Vec<Value> {
    let read = |action_ref: &str, title: &str, subject_kinds: Value| {
        read_action(action_ref, title, json!([]), subject_kinds)
    };
    let document_args = json!([{"name": "document", "kind": "json"}]);
    vec![
        read(
            "capabilities",
            "Read the CLI surface and native contract versions",
            json!(["actuation.cli"]),
        ),
        read(
            "contract.list",
            "List native contract versions",
            json!(["actuation.contract"]),
        ),
        read(
            "harness.catalog",
            "Declare the harness detection catalog",
            json!(["actuation.harness"]),
        ),
        read(
            "harness.detect",
            "Prove harness presence live on this machine",
            json!(["actuation.harness"]),
        ),
        read(
            "harness.self",
            "Identify the harness this process runs inside",
            json!(["actuation.harness"]),
        ),
        read(
            "verify",
            "Run the native verification gate (deterministic product checks)",
            json!(["actuation.verification"]),
        ),
        read(
            "agency.read",
            "Project an Agency read model",
            json!(["actuation.agency"]),
        ),
        read(
            "realised.read",
            "Project a realised-actuation read model",
            json!(["actuation.realised"]),
        ),
        read(
            "stream.read",
            "Project an ActuationStream read model",
            json!(["actuation.stream"]),
        ),
        read(
            "activity.read",
            "Validate and project an Activity read model",
            json!(["actuation.activity"]),
        ),
        action(json!({
            "action_ref": "instantiation.read",
            "title": "Validate an instantiation receipt",
            "args": document_args,
            "availability": "disclosed",
            "unavailable_reason": null,
            "subject_kinds": ["actuation.instantiation"],
            "authority": no_authority(),
            "exposure": headless(),
        })),
        action(json!({
            "action_ref": "agency.actualise",
            "title": "Actualise a determination as a semantic relation",
            "args": document_args,
            "availability": "unavailable",
            "unavailable_reason": "Native CLI operation; Actuation discloses no O:I intent/invoke seam this wave and performs no materialisation (receipt effect: materialisation not-performed).",
            "subject_kinds": ["actuation.agency", "actuation.determination"],
            "authority": {
                "requires": ["metagency-grant:determine-agency", "metagency-grant:actualise-agency (derivation only)"],
                "granted_by": "the governing Agency's exact MetagencyGrant (authority is never derived from caller identity or surface position)",
                "evidence_ref": PATH_AGENCY_ACTUALISATION,
            },
            "exposure": native_only(),
        })),
        action(json!({
            "action_ref": "stream.durable",
            "title": "Durable stream open/record/replay/close",
            "args": [{"name": "store", "kind": "path"}, {"name": "document", "kind": "json"}],
            "availability": "unavailable",
            "unavailable_reason": "Native CLI operation; no intent/invoke seam disclosed. Store is caller-supplied (--store <dir>); there is no product-owned live stream registry.",
            "subject_kinds": ["actuation.stream"],
            "authority": no_authority(),
            "exposure": native_only(),
        })),
        action(json!({
            "action_ref": "instantiation.record",
            "title": "Append a bound instantiation receipt (JSONL)",
            "args": [{"name": "document", "kind": "json"}, {"name": "out", "kind": "path"}],
            "availability": "unavailable",
            "unavailable_reason": "Native CLI operation; no intent/invoke seam disclosed. Writes caller-supplied JSONL only.",
            "subject_kinds": ["actuation.instantiation"],
            "authority": no_authority(),
            "exposure": native_only(),
        })),
        action(json!({
            "action_ref": "stream.usage",
            "title": "Record a model-usage observation into a durable stream",
            "args": [{"name": "adapter", "kind": "string"}, {"name": "document", "kind": "json"}],
            "availability": "unavailable",
            "unavailable_reason": "Native CLI operation; no intent/invoke seam disclosed.",
            "subject_kinds": ["actuation.model-usage"],
            "authority": no_authority(),
            "exposure": native_only(),
        })),
        action(json!({
            "action_ref": "ecology.read",
            "title": "Read the live actor ecology (instantiated/realised actuations)",
            "args": [],
            "availability": "missing_native_obligation",
            "unavailable_reason": null,
            "subject_kinds": ["actuation.ecology"],
            "authority": no_authority(),
            "exposure": {"ui": false, "agent": false, "headless": false},
        })),
        action(json!({
            "action_ref": "attach",
            "title": "Attach an observer to a live actor/session",
            "args": [{"name": "actor_ref", "kind": "string"}],
            "availability": "missing_native_obligation",
            "unavailable_reason": null,
            "subject_kinds": ["actuation.ecology"],
            "authority": no_authority(),
            "exposure": {"ui": false, "agent": false, "headless": false},
        })),
    ]
}

/// Build the Wave 5 System disclosure for Actuation. Read-only: it projects
/// the frozen descriptor over the live harness-detection and
/// self-identification readings the CLI already runs. `now_ms` is injectable
/// for hermetic tests.
pub fn build_system_disclosure(detection: &Value, self_value: &Value, now_ms: i64) -> Value {
    let observed_at_ms = now_ms;
    let surface = cli_surface();
    let contracts = surface["native_contracts"].clone();

    let declared_count = detection["harnesses"].as_array().map(Vec::len).unwrap_or(0);
    let detected_count = detection["harnesses"]
        .as_array()
        .map(|harnesses| {
            harnesses
                .iter()
                .filter(|entry| entry["state"] == "detected")
                .count()
        })
        .unwrap_or(0);
    let slugs = |state: &str| -> Vec<Value> {
        detection["harnesses"]
            .as_array()
            .map(|harnesses| {
                harnesses
                    .iter()
                    .filter(|entry| entry["state"] == state)
                    .map(|entry| entry["slug"].clone())
                    .collect()
            })
            .unwrap_or_default()
    };
    let states: Map<String, Value> = detection["harnesses"]
        .as_array()
        .map(|harnesses| {
            harnesses
                .iter()
                .filter_map(|entry| {
                    Some((entry["slug"].as_str()?.to_owned(), entry["state"].clone()))
                })
                .collect()
        })
        .unwrap_or_default();

    let resolved = &self_value["resolved"];
    let running_in = if resolved.is_null() {
        Value::Null
    } else {
        json!({
            "slug": resolved["slug"],
            "harness_ref": resolved["harness_ref"],
        })
    };

    let agency_contract_path = PATH_AGENCY;
    let sections = vec![
        {
            let settings = vec![
                finish_setting(
                    contract_setting(
                        "agency.contract",
                        "Agency contract",
                        "scalar",
                        json!(AGENCY_CONTRACT_VERSION),
                        agency_contract_path,
                        "actuation:cli:capabilities",
                        observed_at_ms,
                    ),
                    agency_contract_path,
                    observed_at_ms,
                ),
                finish_setting(
                    vocabulary_setting(
                        "agency.determination.kinds",
                        "Determination kinds",
                        json!([
                            "self-differentiation",
                            "delegation",
                            "derivation",
                            "federation"
                        ]),
                        agency_contract_path,
                        "actuation:cli:agency",
                        observed_at_ms,
                    ),
                    agency_contract_path,
                    observed_at_ms,
                ),
                finish_setting(
                    vocabulary_setting(
                        "agency.world_binding.constraints",
                        "WorldBinding constraint categories",
                        json!([
                            "human_authored_refs",
                            "security_policy_refs",
                            "evidence_refs",
                            "external_reality_refs"
                        ]),
                        agency_contract_path,
                        "actuation:cli:agency",
                        observed_at_ms,
                    ),
                    agency_contract_path,
                    observed_at_ms,
                ),
            ];
            json!({"id": "agency", "title": "Agent / Agency / WorldBinding", "settings": settings})
        },
        {
            let settings = vec![
                finish_setting(
                    vocabulary_setting("authority.metagency.operations", "Metagency operations", json!(["determine-agency", "configure-agency", "actualise-agency", "reintegrate-return"]), agency_contract_path, "actuation:cli:agency", observed_at_ms),
                    agency_contract_path,
                    observed_at_ms,
                ),
                finish_setting(
                    contract_setting(
                        "authority.derivation.rule",
                        "Derivation authority requirement",
                        "scalar",
                        json!("Derivation requires explicit actualise-agency authority and an explicitly actualised Agent identity"),
                        PATH_AGENCY_ACTUALISATION,
                        "actuation:cli:agency-actualisation",
                        observed_at_ms,
                    ),
                    PATH_AGENCY_ACTUALISATION,
                    observed_at_ms,
                ),
                finish_setting(
                    contract_setting(
                        "authority.federation.rule",
                        "Federation authority rule",
                        "scalar",
                        json!("Federation cannot silently carry determining authority; use an explicit delegation"),
                        agency_contract_path,
                        "actuation:cli:agency",
                        observed_at_ms,
                    ),
                    agency_contract_path,
                    observed_at_ms,
                ),
            ];
            json!({"id": "authority", "title": "Authority / Metagency", "settings": settings})
        },
        {
            let harness_axes = json!({
                "declared": axis(
                    json!({"catalog_revision": detection["catalog_revision"], "descriptor_count": declared_count, "slugs": slugs_all(detection)}),
                    provenance("actuation:harness:catalog", PATH_CATALOG, observed_at_ms),
                ),
                "effective": axis(
                    json!({
                        "states": states,
                        "detected": slugs("detected"),
                        "unavailable": slugs("unavailable"),
                        "not_installed": slugs("not-installed"),
                    }),
                    provenance("actuation:harness:detect", PATH_DETECT, observed_at_ms),
                ),
                "active": axis(
                    json!({"running_in": running_in, "ambiguity": self_value["ambiguity"]}),
                    provenance("actuation:harness:self", PATH_SELF, observed_at_ms),
                ),
            });
            let settings = vec![
                {
                    let mut setting_value = setting(
                        "availability.contracts",
                        "Native contract surface",
                        "table",
                        axis(
                            contracts.clone(),
                            provenance(OWNER_REF, PATH_SURFACE, observed_at_ms),
                        ),
                        axis(
                            contracts.clone(),
                            provenance("actuation:cli:capabilities", PATH_SURFACE, observed_at_ms),
                        ),
                        axis(
                            contracts.clone(),
                            provenance("actuation:cli:system", PATH_SYSTEM, observed_at_ms),
                        ),
                    );
                    setting_value["axes"]["staged"] =
                        staged_axis(provenance(OWNER_REF, PATH_SURFACE, observed_at_ms));
                    setting_value["axes"]["expected_effect"] = expected_effect();
                    setting_value["native_path"] =
                        json!("actuation (read-only projection; no mutation disclosed)");
                    setting_value
                },
                {
                    let mut setting_value = setting(
                        "availability.harness.catalog",
                        "Harness detection catalog vs live detection",
                        "table",
                        harness_axes["declared"].clone(),
                        harness_axes["effective"].clone(),
                        harness_axes["active"].clone(),
                    );
                    setting_value["axes"]["staged"] = staged_axis(provenance(
                        "actuation:harness:catalog",
                        PATH_CATALOG,
                        observed_at_ms,
                    ));
                    setting_value["axes"]["expected_effect"] = expected_effect();
                    setting_value["native_path"] =
                        json!("actuation harness detect (no mutation disclosed)");
                    setting_value["drift"] = if detected_count == declared_count {
                        drift("none")
                    } else {
                        drift("diverged")
                    };
                    setting_value
                },
                {
                    let verification_command =
                        json!({"present": true, "command": ["actuation", "verify", "--json"]});
                    let mut setting_value = setting(
                        "availability.verification",
                        "Verification gate",
                        "presence",
                        axis(
                            verification_command.clone(),
                            provenance(OWNER_REF, PATH_COMMANDS, observed_at_ms),
                        ),
                        axis(
                            verification_command.clone(),
                            provenance("actuation:cli:capabilities", PATH_COMMANDS, observed_at_ms),
                        ),
                        axis(
                            json!({"running": false, "last_run": null}),
                            provenance("actuation:cli:system", PATH_SYSTEM, observed_at_ms),
                        ),
                    );
                    setting_value["axes"]["staged"] =
                        staged_axis(provenance(OWNER_REF, PATH_COMMANDS, observed_at_ms));
                    setting_value["axes"]["expected_effect"] = expected_effect();
                    setting_value["native_path"] = json!("actuation verify");
                    setting_value
                },
            ];
            json!({"id": "availability", "title": "Actual actuation availability", "settings": settings})
        },
        {
            let settings = vec![
                finish_setting(
                    contract_setting(
                        "activity.contract",
                        "Activity contract",
                        "scalar",
                        json!(ACTIVITY_VERSION),
                        "crates/actuation-stream/src/activity.rs",
                        "actuation:cli:capabilities",
                        observed_at_ms,
                    ),
                    "crates/actuation-stream/src/activity.rs",
                    observed_at_ms,
                ),
                finish_setting(
                    contract_setting(
                        "stream.contract",
                        "ActuationStream contract",
                        "scalar",
                        json!(ACTUATION_STREAM_VERSION),
                        "crates/actuation-stream/src/stream.rs",
                        "actuation:cli:capabilities",
                        observed_at_ms,
                    ),
                    "crates/actuation-stream/src/stream.rs",
                    observed_at_ms,
                ),
                {
                    let mut setting_value = setting(
                        "stream.durable_store",
                        "Durable stream store",
                        "presence",
                        axis(
                            json!({"default_root": default_root_display()}),
                            provenance(OWNER_REF, PATH_STREAM_STORE, observed_at_ms),
                        ),
                        axis(
                            json!({"root": effective_store_root()}),
                            provenance(
                                "actuation:cli:capabilities",
                                PATH_STREAM_STORE,
                                observed_at_ms,
                            ),
                        ),
                        axis(
                            default_stream_store_state(),
                            provenance("actuation:cli:system", PATH_SYSTEM, observed_at_ms),
                        ),
                    );
                    setting_value["axes"]["staged"] =
                        staged_axis(provenance(OWNER_REF, PATH_STREAM_STORE, observed_at_ms));
                    setting_value["axes"]["expected_effect"] = expected_effect();
                    setting_value["native_path"] = json!("actuation stream open --store <dir>");
                    setting_value
                },
            ];
            json!({"id": "activity", "title": "Activity / ActuationStream", "settings": settings})
        },
        {
            let settings = vec![
                finish_setting(
                    contract_setting(
                        "return.contract",
                        "Return contract",
                        "scalar",
                        json!(AGENCY_CONTRACT_VERSION),
                        agency_contract_path,
                        "actuation:cli:capabilities",
                        observed_at_ms,
                    ),
                    agency_contract_path,
                    observed_at_ms,
                ),
                finish_setting(
                    vocabulary_setting(
                        "return.modes",
                        "Return policy modes",
                        json!(["required", "optional", "autonomous-termination"]),
                        agency_contract_path,
                        "actuation:cli:agency",
                        observed_at_ms,
                    ),
                    agency_contract_path,
                    observed_at_ms,
                ),
            ];
            json!({"id": "return", "title": "Return", "settings": settings})
        },
    ];

    let obligations = [
        "ecology.read — no product-owned live registry of instantiated/realised actuations; receipts are caller-supplied documents, not a live ledger.",
        "attach — no native operation binds an observer to a live actor/session.",
        "Actuation discloses no kernel intent/invoke engagement seam; mutating operations (agency actualise, stream lifecycle, instantiation record, stream usage) are native CLI only and are not mountable through the O:I System surface this wave.",
        "Factory<->Actuation discovery/intent/authority seam — NOT owned by either track; returned for the serialized owner decision (do not invent). Actuation's actualise/realised receipts deliberately perform no Factory recognition and no source mutation; who owns the materialisation hand-off is undecided.",
    ];

    let mut descriptor = json!({
        "schema": SYSTEM_DISCLOSURE_VERSION,
        "product_id": PRODUCT_ID,
        "contract_revision": SYSTEM_DISCLOSURE_CONTRACT_REVISION,
        "disclosed_at_unix_ms": observed_at_ms,
        "owner": {
            "owner_id": PRODUCT_ID,
            "owner_ref": OWNER_REF,
            "owner_version": ACTUATION_CLI_VERSION,
            "reading_command": READING_COMMAND,
            "reading_digest": Value::Null,
            "reading_digest_covers": "whole descriptor, every *_unix_ms field zeroed, owner.reading_digest null",
            "observed_at_unix_ms": observed_at_ms,
        },
        "about": "Actuation is the constitution and management of technological agency: it validates and projects Agent/Agency/WorldBinding, determination, bounds, authority and Return contracts, and proves harness presence live on this machine. It performs no agency materialisation and discloses no O:I engagement seam; mutating operations are native CLI only.",
        "sections": sections,
        "actions": canonical_actions(),
        "availability": derive_availability(detection),
        "degradations": derive_degradations(detection),
        "obligations": obligations,
    });

    // The canonical reading digest is the sha256 of the canonical body — the
    // whole descriptor with every *_unix_ms field zeroed and reading_digest
    // held null — so two readings of an unchanged world produce the same
    // digest and a changed digest means a changed reading, never a changed
    // clock (§4.5).
    descriptor["owner"]["reading_digest"] = json!(reading_digest(&descriptor));
    descriptor
}

fn slugs_all(detection: &Value) -> Vec<Value> {
    detection["harnesses"]
        .as_array()
        .map(|harnesses| {
            harnesses
                .iter()
                .map(|entry| entry["slug"].clone())
                .collect()
        })
        .unwrap_or_default()
}

fn home_streams() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(|home| std::path::Path::new(&home).join(".actuation/streams"))
}

fn default_root_display() -> Value {
    match home_streams() {
        Some(path) => json!(path.to_string_lossy().into_owned()),
        None => Value::Null,
    }
}

fn effective_store_root() -> Value {
    match std::env::var_os("ACTUATION_STREAM_STORE") {
        Some(root) => json!(root.to_string_lossy().into_owned()),
        None => default_root_display(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detection_fixture() -> Value {
        json!({
            "schema": "actuation.harness-detection/v1",
            "document": "detection",
            "detection_ref": "detection:1",
            "observed_at": "2026-09-12T00:00:00Z",
            "catalog_revision": 6,
            "availability": "complete",
            "harnesses": [
                {"slug": "zcode", "harness_ref": "harness/zcode", "state": "not-installed"},
                {"slug": "codex", "harness_ref": "harness/codex", "state": "not-installed"}
            ],
            "disclosure": []
        })
    }

    fn self_fixture() -> Value {
        json!({
            "schema": "actuation.harness-detection/v1",
            "document": "self",
            "resolved": null,
            "ambiguity": false,
            "matched": [],
            "detection_ref": "detection:1",
            "detection": {"states": {"zcode": "not-installed", "codex": "not-installed"}}
        })
    }

    #[test]
    fn the_digest_is_a_function_of_the_reading_not_the_clock() {
        let a = build_system_disclosure(&detection_fixture(), &self_fixture(), 1_000);
        let b = build_system_disclosure(&detection_fixture(), &self_fixture(), 2_000);
        assert_ne!(a["disclosed_at_unix_ms"], b["disclosed_at_unix_ms"]);
        assert_eq!(a["owner"]["reading_digest"], b["owner"]["reading_digest"]);
        let body = canonical_reading_body(&a);
        assert_eq!(body["disclosed_at_unix_ms"], json!(0));
        assert_eq!(body["owner"]["observed_at_unix_ms"], json!(0));
        assert_eq!(body["owner"]["reading_digest"], Value::Null);
        assert_eq!(
            a["owner"]["reading_digest"],
            json!(reading_digest(&a)),
            "digest must recompute from the canonical body"
        );
    }

    #[test]
    fn every_setting_exposes_all_five_axes() {
        let value = build_system_disclosure(&detection_fixture(), &self_fixture(), 1);
        let mut count = 0;
        for section in value["sections"].as_array().unwrap() {
            assert!(!section["id"].as_str().unwrap_or_default().is_empty());
            for setting in section["settings"].as_array().unwrap() {
                count += 1;
                assert!(!setting["key"].as_str().unwrap_or_default().is_empty());
                for name in [
                    "declared",
                    "effective",
                    "active",
                    "staged",
                    "expected_effect",
                ] {
                    assert!(
                        setting["axes"].get(name).is_some(),
                        "{} missing axis {name}",
                        setting["key"]
                    );
                }
                assert!(setting["axes"]["staged"].get("stage_state").is_some());
                assert!(setting["axes"]["expected_effect"].get("summary").is_some());
                assert!(setting["mutable"].is_boolean());
                assert!(!setting["native_path"]
                    .as_str()
                    .unwrap_or_default()
                    .is_empty());
                assert!(setting["bootstrap"].is_boolean());
            }
        }
        assert!(
            count >= 8,
            "the disclosure must cover the required subjects, not a stub"
        );
    }

    #[test]
    fn required_subjects_are_present() {
        let value = build_system_disclosure(&detection_fixture(), &self_fixture(), 1);
        let keys: Vec<&str> = value["sections"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|section| section["settings"].as_array().unwrap().iter())
            .filter_map(|setting| setting["key"].as_str())
            .collect();
        for key in [
            "agency.contract",
            "agency.determination.kinds",
            "authority.metagency.operations",
            "authority.derivation.rule",
            "authority.federation.rule",
            "availability.harness.catalog",
            "availability.verification",
            "activity.contract",
            "stream.contract",
            "stream.durable_store",
            "return.contract",
            "return.modes",
        ] {
            assert!(
                keys.contains(&key),
                "required subject setting {key} missing"
            );
        }
    }

    #[test]
    fn the_five_action_states_never_collapse() {
        let value = build_system_disclosure(&detection_fixture(), &self_fixture(), 1);
        let actualise = value["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|action| action["action_ref"] == "agency.actualise")
            .expect("actualise action must be disclosed");
        assert_eq!(actualise["exposure"]["headless"], json!(true));
        assert_eq!(actualise["exposure"]["ui"], json!(false));
        assert!(
            !actualise["authority"]["requires"]
                .as_array()
                .unwrap()
                .is_empty(),
            "actualise authority must not be empty (exposed must not imply authorised)"
        );
        assert!(actualise["authority"]["granted_by"]
            .as_str()
            .unwrap()
            .contains("MetagencyGrant"));
        assert!(actualise.get("selected").is_none());
        assert!(actualise.get("invoked").is_none());
        assert!(
            !value.to_string().contains("\"selected\""),
            "selected is not an owner fact anywhere in the descriptor"
        );
        for action in value["actions"].as_array().unwrap() {
            let required = action["authority"]["requires"].as_array().unwrap();
            for requirement in required {
                assert!(
                    !requirement
                        .as_str()
                        .unwrap()
                        .to_lowercase()
                        .contains("root"),
                    "authority.requires must not name a root identity: {requirement}"
                );
            }
        }
    }
}
