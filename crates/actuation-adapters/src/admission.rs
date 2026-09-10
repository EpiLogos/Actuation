//! Lossless public-wire admission. Extensible native descriptors retain foreign
//! fields and null/omission rather than narrowing an accepted /v1 wire shape.
//! Private record storage prevents mutation after admission. Executable probe
//! requests are separately checked at the effect boundary; a valid declaration
//! is not proof that the corresponding mechanism is executable or present.
use actuation_core::{is_blank_reference, Error, Result};
use actuation_stream::Timestamp;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashSet;

pub const HARNESS_DETECTION_VERSION: &str = "actuation.harness-detection/v1";
pub const HARNESS_CAPABILITY_VERSION: &str = "actuation.harness-capability/v1";
pub const INSTANTIATION_VERSION: &str = "actuation.instantiation/v1";
pub const LEGACY_MODEL_BEARING_SCHEMA: &str = "actuation.model-bearing/v1";
pub const SECRET_DETECTION_VERSION: &str = "actuation.secret-detection/v1";
pub const KNOWN_EVENTS: &[&str] = &[
    "session-start",
    "user-prompt-submit",
    "pre-tool-use",
    "post-tool-use",
    "stop",
    "session-end",
    "pre-compact",
    "notification",
    "custom",
];
const CHANNELS: &[&str] = &[
    "stdout-additional-context",
    "stdout-plain-text",
    "exit-code-payload",
    "none",
];
const PROBES: &[&str] = &["executable", "config-dir", "service", "env"];
const FACETS: &[&str] = &[
    "skills",
    "harness-compositions",
    "plugins",
    "hooks",
    "commands",
    "rules",
    "extensions",
    "agents",
    "settings",
    "config",
    "models",
];
const SECRET_PROBES: &[&str] = &["env", "file-pattern", "cli-presence", "vault-item"];
const FORBIDDEN: &[&str] = &[
    "value",
    "material",
    "secret_value",
    "secretvalue",
    "plaintext",
    "secret",
    "password",
    "token_value",
    "api_key",
    "apikey",
    "credential",
];

pub(crate) fn require(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(Error::new(message))
    }
}
pub(crate) fn object(v: &Value) -> Result<&serde_json::Map<String, Value>> {
    v.as_object()
        .ok_or_else(|| Error::new("expected a record object"))
}
pub(crate) fn text(v: &Value) -> Result<&str> {
    v.as_str()
        .filter(|s| !is_blank_reference(s))
        .ok_or_else(|| Error::new("expected non-empty text"))
}
pub(crate) fn texts(v: &Value) -> Result<Vec<String>> {
    array(v)?
        .iter()
        .map(|v| text(v).map(str::to_owned))
        .collect()
}
pub(crate) fn array(v: &Value) -> Result<&Vec<Value>> {
    v.as_array().ok_or_else(|| Error::new("expected an array"))
}
pub(crate) fn optional_text(v: &Value) -> Result<()> {
    if !v.is_null() {
        text(v)?;
    }
    Ok(())
}
fn optional_texts(v: &Value) -> Result<()> {
    if !v.is_null() {
        texts(v)?;
    }
    Ok(())
}
pub(crate) fn one(v: &Value, values: &[&str]) -> Result<()> {
    require(
        v.as_str().is_some_and(|s| values.contains(&s)),
        &format!("expected one of {}", values.join(", ")),
    )
}
fn integer(v: &Value) -> bool {
    v.as_f64()
        .is_some_and(|n| n.is_finite() && n.fract() == 0.0)
}
fn nonnegative_integer(v: &Value) -> bool {
    integer(v) && v.as_f64().is_some_and(|n| n >= 0.0)
}
fn date(v: &Value) -> Result<()> {
    Timestamp::new(text(v)?)?;
    Ok(())
}
fn header(v: &Value, schema: &str, document: Option<&str>) -> Result<()> {
    object(v)?;
    require(v["schema"] == schema, "wrong schema")?;
    if let Some(d) = document {
        require(v["document"] == d, "wrong document kind")?;
    }
    Ok(())
}
fn reference_fields(v: &Value, required: &[&str], optional: &[&str]) -> Result<()> {
    for key in required {
        text(&v[*key])?;
    }
    for key in optional {
        optional_text(&v[*key])?;
    }
    Ok(())
}
fn probe_records(v: &Value, kinds: &[&str]) -> Result<()> {
    if v.is_null() {
        return Ok(());
    }
    for p in array(v)? {
        object(p)?;
        one(&p["kind"], kinds)?;
        one(&p["result"], &["pass", "fail"])?;
        optional_text(&p["spec"])?;
        optional_text(&p["detail"])?;
    }
    Ok(())
}
fn fingerprint(v: &Value) -> Result<()> {
    require(
        v.as_str().is_some_and(|s| {
            s.len() == 64
                && s.bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        }),
        "expected lowercase SHA-256 fingerprint",
    )
}
pub(crate) fn no_value_keys(v: &Value) -> Result<()> {
    match v {
        Value::Object(m) => {
            for (k, v) in m {
                require(
                    !FORBIDDEN.contains(&k.to_lowercase().as_str()),
                    "secret evidence contains a forbidden value-shaped key",
                )?;
                no_value_keys(v)?;
            }
        }
        Value::Array(a) => {
            for v in a {
                no_value_keys(v)?;
            }
        }
        _ => (),
    }
    Ok(())
}
macro_rules! wire_record {
    ($name:ident, $validate:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name(Value);
        impl $name {
            pub fn as_value(&self) -> &Value {
                &self.0
            }
            pub fn into_value(self) -> Value {
                self.0
            }
        }
        impl TryFrom<Value> for $name {
            type Error = Error;
            fn try_from(v: Value) -> Result<Self> {
                $validate(&v)?;
                Ok(Self(v))
            }
        }
        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
                self.0.serialize(s)
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
                Self::try_from(Value::deserialize(d)?).map_err(serde::de::Error::custom)
            }
        }
    };
}
wire_record!(HarnessDescriptor, validate_harness_descriptor);
wire_record!(HarnessCapability, validate_harness_capability);
wire_record!(HarnessCatalog, validate_harness_catalog);
wire_record!(HarnessDetection, validate_harness_detection);
wire_record!(HarnessSelf, validate_harness_self);
wire_record!(ModelRelation, validate_model_relation);
wire_record!(ModelAccessProfile, validate_model_access_profile);
wire_record!(InstantiationReceipt, validate_instantiation_receipt);
wire_record!(SecretSourceDescriptor, validate_secret_source_descriptor);
wire_record!(SecretSourceCatalog, validate_secret_source_catalog);
wire_record!(SecretScan, validate_secret_scan);

pub fn validate_harness_descriptor(v: &Value) -> Result<()> {
    header(v, HARNESS_DETECTION_VERSION, None)?;
    reference_fields(
        v,
        &["slug", "native_kind"],
        &["edition", "native_owner", "summary"],
    )?;
    optional_texts(&v["aliases"])?;
    let probes = object(&v["probe"])?;
    require(!probes.is_empty(), "at least one probe must be declared")?;
    for (k, p) in probes {
        require(PROBES.contains(&k.as_str()), "unsupported probe kind")?;
        object(p)?;
    }
    if !v["probe"]["env"].is_null() {
        require(
            !texts(&v["probe"]["env"]["any_of"])?.is_empty(),
            "environment probe requires marker names",
        )?;
    }
    if !v["facets"].is_null() {
        for (k, f) in object(&v["facets"])? {
            require(FACETS.contains(&k.as_str()), "unsupported facet kind")?;
            object(f)?;
            text(&f["path"])?;
            if !f["inventory"].is_null() {
                let i = &f["inventory"];
                object(i)?;
                one(&i["kind"], &["http-json"])?;
                one(&i["from"], &["service"])?;
                require(
                    !v["probe"]["service"].is_null(),
                    "inventory requires the descriptor's own service probe",
                )?;
                require(
                    text(&i["route"])?.starts_with('/'),
                    "inventory route must remain a path on the declared endpoint",
                )?;
                reference_fields(i, &["collection", "id_field"], &[])?;
                optional_texts(&i["also_id_fields"])?;
                optional_texts(&i["detail_fields"])?;
            }
        }
    }
    let p = &v["provenance"];
    object(p)?;
    text(&p["authored_by"])?;
    optional_texts(&p["source_refs"])?;
    optional_text(&p["accepted_by"])?;
    require(
        integer(&p["catalog_revision"]),
        "catalog revision must be an integer",
    )
}
fn validate_seam(v: &Value) -> Result<()> {
    object(v)?;
    reference_fields(v, &["config_path", "entry_shape", "ownership_marker"], &[])?;
    one(&v["format"], &["json", "jsonc", "toml"])?;
    require(
        v["preserves_foreign_entries"] == true,
        "native seams must preserve foreign entries",
    )
}
pub fn validate_harness_capability(v: &Value) -> Result<()> {
    header(v, HARNESS_CAPABILITY_VERSION, Some("capability"))?;
    text(&v["harness_slug"])?;
    optional_text(&v["summary"])?;
    let mut seen = HashSet::new();
    for e in array(&v["native_events"])? {
        object(e)?;
        one(&e["event"], KNOWN_EVENTS)?;
        if e["event"] != "custom" {
            require(seen.insert(text(&e["event"])?), "duplicate native event")?;
        }
        reference_fields(e, &["native_name", "transport"], &["notes"])?;
        require(e["can_block"].is_boolean(), "can_block must be boolean")?;
        one(&e["context_channel"], CHANNELS)?;
    }
    let i = &v["injection_channel"];
    object(i)?;
    one(&i["kind"], CHANNELS)?;
    text(&i["mechanism"])?;
    optional_text(&i["notes"])?;
    let b = &v["blocking_semantics"];
    object(b)?;
    one(&b["kind"], &["deny-and-block", "advisory-only", "none"])?;
    optional_text(&b["notes"])?;
    let w = &v["wake_capability"];
    object(w)?;
    one(&w["kind"], &["immediate-wake", "next-event", "none"])?;
    optional_text(&w["notes"])?;
    if w["kind"] == "immediate-wake" {
        text(&w["notes"])?;
    }
    validate_seam(&v["install_seam"])?;
    validate_seam(&v["uninstall_seam"])?;
    if !v["model_dispatch"].is_null() {
        let d = &v["model_dispatch"];
        object(d)?;
        one(&d["kind"], &["native-provider-binding", "none"])?;
        optional_text(&d["notes"])?;
        if d["kind"] == "none" {
            require(
                d["providers"].is_null(),
                "absent model dispatch cannot carry providers",
            )?;
        } else {
            let providers = array(&d["providers"])?;
            require(!providers.is_empty(), "native binding requires a provider")?;
            for p in providers {
                object(p)?;
                text(&p["provider_ref"])?;
                optional_text(&p["notes"])?;
                let s = &p["selector"];
                object(s)?;
                one(&s["kind"], &["config-key", "cli-flag", "env-var"])?;
                text(&s["name"])?;
                let c = &p["credential"];
                object(c)?;
                require(
                    c["required"].is_boolean(),
                    "credential.required must be boolean",
                )?;
                if c["required"] == true {
                    text(&c["hint"])?;
                } else {
                    require(
                        c["hint"].is_null(),
                        "credential-free path cannot require a credential hint",
                    )?;
                }
            }
        }
    }
    let p = &v["provenance"];
    object(p)?;
    text(&p["authored_by"])?;
    require(
        !texts(&p["source_refs"])?.is_empty(),
        "capability requires source provenance",
    )
}
pub fn validate_harness_catalog(v: &Value) -> Result<()> {
    header(v, HARNESS_DETECTION_VERSION, Some("catalog"))?;
    require(
        integer(&v["catalog_revision"]),
        "catalog revision must be integer",
    )?;
    let a = array(&v["descriptors"])?;
    require(!a.is_empty(), "catalog must not be empty")?;
    let mut seen = HashSet::new();
    for d in a {
        validate_harness_descriptor(d)?;
        require(seen.insert(text(&d["slug"])?), "duplicate descriptor slug")?;
    }
    Ok(())
}
fn validate_inventory(f: &Value) -> Result<()> {
    if f["inventory"].is_null() {
        require(
            f["inventory_receipt"].is_null(),
            "receipt cannot accompany absent inventory",
        )?;
        optional_text(&f["inventory_unavailable_reason"])?;
        return Ok(());
    }
    require(
        f["inventory_unavailable_reason"].is_null(),
        "inventory cannot be observed and unavailable",
    )?;
    let items = array(&f["inventory"])?;
    let mut seen = HashSet::new();
    for item in items {
        object(item)?;
        require(
            seen.insert(text(&item["id"])?),
            "duplicate provider-native identity",
        )?;
        optional_texts(&item["also_known_as"])?;
    }
    let r = &f["inventory_receipt"];
    object(r)?;
    one(&r["kind"], &["http-json"])?;
    text(&r["source"])?;
    date(&r["observed_at"])?;
    require(
        r["item_count"].as_f64() == Some(items.len() as f64),
        "inventory receipt count disagrees with observation",
    )
}
pub fn validate_harness_detection(v: &Value) -> Result<()> {
    // The accepted constructor does not constrain `document` here; retain the
    // existing public validation semantics instead of narrowing them silently.
    header(v, HARNESS_DETECTION_VERSION, None)?;
    text(&v["detection_ref"])?;
    date(&v["observed_at"])?;
    require(
        integer(&v["catalog_revision"]),
        "catalog revision must be integer",
    )?;
    object(&v["detector"])?;
    reference_fields(&v["detector"], &["implementation", "version"], &[])?;
    let mut seen = HashSet::new();
    let mut absent = Vec::new();
    let mut unavailable = false;
    for e in array(&v["harnesses"])? {
        object(e)?;
        let slug = text(&e["slug"])?;
        require(seen.insert(slug), "duplicate observed slug")?;
        require(
            e["harness_ref"] == format!("harness/{slug}"),
            "harness ref does not match its slug",
        )?;
        reference_fields(e, &[], &["native_kind", "version"])?;
        one(&e["state"], &["detected", "unavailable", "not-installed"])?;
        if e["state"] == "unavailable" {
            text(&e["unavailable_reason"])?;
            unavailable = true;
        }
        if e["state"] == "detected" {
            require(
                array(&e["probes"])?.iter().any(|p| p["result"] == "pass"),
                "detected requires same-run probe evidence",
            )?;
            object(&e["receipts"])?;
            text(&e["receipts"]["executable"])?;
            if !e["receipts"]["sha256"].is_null() {
                fingerprint(&e["receipts"]["sha256"])?;
            }
        }
        if e["state"] == "not-installed" {
            absent.push(Value::String(slug.to_owned()));
            if let Some(a) = e["probes"].as_array() {
                require(!a.is_empty(), "absence requires a probe")?;
            }
            if let Some(a) = e["facets"].as_array() {
                require(
                    !a.iter().any(|f| f["exists"] == true),
                    "absent target cannot have observed facets",
                )?;
            }
        }
        if !e["facets"].is_null() {
            for f in array(&e["facets"])? {
                object(f)?;
                one(&f["kind"], FACETS)?;
                text(&f["path"])?;
                require(f["exists"].is_boolean(), "facet existence must be boolean")?;
                validate_inventory(f)?;
            }
        }
        probe_records(&e["probes"], PROBES)?;
    }
    let empty = Value::Array(vec![]);
    let declared = v.get("absent").filter(|v| !v.is_null()).unwrap_or(&empty);
    require(
        *declared == Value::Array(absent),
        "absence list must exactly preserve catalog order",
    )?;
    require(
        v["availability"] == if unavailable { "partial" } else { "complete" },
        "availability does not reflect unavailable probes",
    )?;
    optional_texts(&v["disclosure"])
}
pub fn validate_harness_self(v: &Value) -> Result<()> {
    header(v, HARNESS_DETECTION_VERSION, Some("self"))?;
    text(&v["self_ref"])?;
    date(&v["observed_at"])?;
    require(
        integer(&v["catalog_revision"]),
        "catalog revision must be integer",
    )?;
    let a = array(&v["matched"])?;
    for m in a {
        object(m)?;
        let slug = text(&m["slug"])?;
        require(
            m["harness_ref"] == format!("harness/{slug}"),
            "self harness ref mismatch",
        )?;
        texts(&m["markers"])?;
    }
    if a.len() == 1 {
        require(
            !v["resolved"].is_null()
                && v["resolved"]["slug"] == a[0]["slug"]
                && v["resolved"]["harness_ref"] == a[0]["harness_ref"],
            "self must name its single match",
        )?;
    } else {
        require(
            v["resolved"].is_null(),
            "ambiguous or absent self cannot resolve a target",
        )?;
    }
    require(v["ambiguity"].is_boolean(), "ambiguity must be boolean")?;
    text(&v["detection_ref"])?;
    require(
        v["detection"].is_object() || v["detection"].is_array(),
        "self requires same-run detection cross-check",
    )
}
fn facts(v: &Value) -> Result<()> {
    if !v.is_null() {
        for (k, value) in object(v)? {
            text(&Value::String(k.clone()))?;
            require(
                !value.is_object() && !value.is_array(),
                "provider facts must be scalars",
            )?;
        }
    }
    Ok(())
}
pub fn validate_model_relation(v: &Value) -> Result<()> {
    header(v, INSTANTIATION_VERSION, None)?;
    text(&v["model_ref"])?;
    optional_text(&v["variant_ref"])?;
    if !v["engine"].is_null() {
        let e = &v["engine"];
        object(e)?;
        reference_fields(e, &[], &["implementation_ref", "provider_ref"])?;
        facts(&e["facts"])?;
    }
    if !v["material"].is_null() {
        let m = &v["material"];
        object(m)?;
        optional_text(&m["binding_ref"])?;
        if !m["placement"].is_null() {
            one(
                &m["placement"],
                &["local", "remote", "distributed", "opaque"],
            )?;
        }
        facts(&m["facts"])?;
    }
    let s = &v["inference_surface"];
    object(s)?;
    text(&s["contract_ref"])?;
    optional_text(&s["binding_ref"])?;
    facts(&s["facts"])
}
pub fn validate_model_access_profile(v: &Value) -> Result<()> {
    header(v, INSTANTIATION_VERSION, None)?;
    for k in ["inference", "control"] {
        object(&v[k])?;
        texts(&v[k]["allowed"])?;
        optional_texts(&v[k]["denied"])?;
    }
    let i = &v["interior"];
    object(i)?;
    one(
        &i["depth"],
        &[
            "opaque",
            "behavioral",
            "outputs",
            "state-read",
            "state-write",
            "causal-intervention",
            "learning",
        ],
    )?;
    optional_texts(&i["allowed"])?;
    optional_texts(&i["denied"])
}
pub fn validate_instantiation_receipt(v: &Value) -> Result<()> {
    header(v, INSTANTIATION_VERSION, None)?;
    reference_fields(
        v,
        &["actuation_ref", "agency_ref", "world_binding_ref"],
        &[
            "harness_ref",
            "harness_composition_ref",
            "agent_session_ref",
            "return_ref",
        ],
    )?;
    validate_model_relation(&v["model_relation"])?;
    validate_model_access_profile(&v["access_profile"])?;
    optional_texts(&v["bounds_refs"])?;
    optional_texts(&v["evidence_refs"])?;
    if !v["experiment"].is_null() {
        let e = &v["experiment"];
        object(e)?;
        optional_texts(&e["held_constant_refs"])?;
        if !e["variables"].is_null() {
            for x in array(&e["variables"])? {
                object(x)?;
                text(&x["name"])?;
                optional_text(&x["value_ref"])?;
                facts(&x["facts"])?;
            }
        }
    }
    if !v["observed_at"].is_null() {
        date(&v["observed_at"])?;
    }
    Ok(())
}
pub fn validate_secret_source_descriptor(v: &Value) -> Result<()> {
    header(v, SECRET_DETECTION_VERSION, Some("descriptor"))?;
    text(&v["slug"])?;
    one(
        &v["source_kind"],
        &[
            "op-item",
            "keychain-entry",
            "varlock-blob",
            "env-var",
            "plaintext-file",
        ],
    )?;
    texts(&v["ref_schemes"])?;
    let probes = object(&v["probe"])?;
    require(!probes.is_empty(), "secret descriptor needs a probe")?;
    for (k, p) in probes {
        require(
            SECRET_PROBES.contains(&k.as_str()),
            "unsupported secret probe kind",
        )?;
        object(p)?;
    }
    let env = &v["probe"]["env"];
    if !env.is_null() {
        optional_texts(&env["names"])?;
        optional_text(&env["name_pattern"])?;
        require(
            !env["names"].is_null() || !env["name_pattern"].is_null(),
            "env probe requires names or name pattern",
        )?;
    }
    let files = &v["probe"]["file-pattern"];
    if !files.is_null() {
        texts(&files["patterns"])?;
        optional_texts(&files["roots"])?;
    }
    let cli = &v["probe"]["cli-presence"];
    if !cli.is_null() {
        texts(&cli["names"])?;
    }
    let vault = &v["probe"]["vault-item"];
    if !vault.is_null() {
        text(&vault["item_ref"])?;
    }
    let p = &v["provenance"];
    object(p)?;
    text(&p["authored_by"])?;
    optional_texts(&p["source_refs"])?;
    require(
        integer(&p["catalog_revision"]),
        "secret catalog revision must be integer",
    )
}
pub fn validate_secret_source_catalog(v: &Value) -> Result<()> {
    header(v, SECRET_DETECTION_VERSION, Some("catalog"))?;
    require(
        integer(&v["catalog_revision"]),
        "secret catalog revision must be integer",
    )?;
    let a = array(&v["descriptors"])?;
    require(!a.is_empty(), "secret source catalog cannot be empty")?;
    let mut seen = HashSet::new();
    for d in a {
        validate_secret_source_descriptor(d)?;
        require(
            seen.insert(text(&d["slug"])?),
            "duplicate secret source slug",
        )?;
    }
    no_value_keys(v)
}
pub fn validate_secret_scan(v: &Value) -> Result<()> {
    header(v, SECRET_DETECTION_VERSION, Some("scan"))?;
    text(&v["scan_ref"])?;
    date(&v["observed_at"])?;
    require(
        integer(&v["catalog_revision"]),
        "secret catalog revision must be integer",
    )?;
    object(&v["scanner"])?;
    reference_fields(&v["scanner"], &["implementation", "version"], &[])?;
    let mut seen = HashSet::new();
    let mut violations = Vec::new();
    let mut unavailable = false;
    for e in array(&v["sources"])? {
        object(e)?;
        let slug = text(&e["slug"])?;
        require(seen.insert(slug), "duplicate scanned source")?;
        require(
            e["source_ref"] == format!("secret-source/{slug}"),
            "secret source ref mismatch",
        )?;
        one(
            &e["state"],
            &["verified", "violation", "absent", "unavailable"],
        )?;
        if e["state"] == "unavailable" {
            text(&e["unavailable_reason"])?;
            unavailable = true;
        }
        if e["state"] == "violation" {
            one(
                &e["violation_class"],
                &[
                    "stray-plaintext",
                    "uncentralised-env",
                    "legacy-env-ref",
                    "unknown",
                ],
            )?;
            text(&e["centralise_to"])?;
            violations.push(Value::String(slug.to_owned()));
        } else {
            require(
                e["violation_class"].is_null(),
                "only violations carry a violation class",
            )?;
        }
        if e["state"] == "verified" {
            require(
                array(&e["probes"])?.iter().any(|p| p["result"] == "pass"),
                "verified source requires a passing probe",
            )?;
        }
        probe_records(&e["probes"], SECRET_PROBES)?;
        if !e["evidence"].is_null() {
            for i in array(&e["evidence"])? {
                object(i)?;
                text(&i["where"])?;
                fingerprint(&i["fingerprint_sha256"])?;
                if !i["byte_length"].is_null() {
                    require(
                        nonnegative_integer(&i["byte_length"]),
                        "evidence byte length must be a nonnegative integer",
                    )?;
                }
            }
        }
    }
    let empty = Value::Array(vec![]);
    let supplied = v
        .get("violations")
        .filter(|v| !v.is_null())
        .unwrap_or(&empty);
    require(
        *supplied == Value::Array(violations),
        "violation list must match source order",
    )?;
    require(
        v["coverage"] == if unavailable { "partial" } else { "complete" },
        "scan coverage must reflect unavailable probes",
    )?;
    optional_texts(&v["disclosure"])?;
    no_value_keys(v)
}
