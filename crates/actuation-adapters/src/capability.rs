use crate::wire::*;
use crate::{Error, Result};
use serde_json::Value;
use std::collections::HashSet;

pub const HARNESS_CAPABILITY_VERSION: &str = "actuation.harness-capability/v1";
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

fn seam(v: &Value) -> Result<()> {
    object(v)?;
    required_texts(v, &["config_path", "entry_shape", "ownership_marker"])?;
    choice(&v["format"], &["json", "jsonc", "toml"], "seam.format")?;
    if v["preserves_foreign_entries"] != true {
        return Err(Error::new("seam must preserve foreign entries"));
    }
    Ok(())
}
fn model_dispatch(v: &Value) -> Result<()> {
    object(v)?;
    match choice(
        &v["kind"],
        &["native-provider-binding", "none"],
        "model_dispatch.kind",
    )? {
        "native-provider-binding" => {
            let providers = list(&v["providers"], "providers")?;
            if providers.is_empty() {
                return Err(Error::new("native provider binding requires providers"));
            }
            for p in providers {
                object(p)?;
                text(&p["provider_ref"], "provider_ref")?;
                let selector = &p["selector"];
                object(selector)?;
                choice(
                    &selector["kind"],
                    &["config-key", "cli-flag", "env-var"],
                    "selector.kind",
                )?;
                text(&selector["name"], "selector.name")?;
                let c = &p["credential"];
                object(c)?;
                if boolean(&c["required"], "credential.required")? {
                    text(&c["hint"], "credential.hint")?;
                } else if present(c, "hint").is_some() {
                    return Err(Error::new(
                        "credential-free binding cannot assert a credential hint",
                    ));
                }
                optional_text(p, "notes")?;
            }
        }
        _ if present(v, "providers").is_some() => {
            return Err(Error::new("no-dispatch declaration cannot carry providers"))
        }
        _ => {}
    }
    optional_text(v, "notes")
}
fn check_capability(v: &Value) -> Result<()> {
    object(v)?;
    exact(&v["schema"], HARNESS_CAPABILITY_VERSION, "schema")?;
    exact(&v["document"], "capability", "document")?;
    text(&v["harness_slug"], "harness_slug")?;
    optional_text(v, "summary")?;
    let mut seen = HashSet::new();
    for e in list(&v["native_events"], "native_events")? {
        object(e)?;
        let kind = choice(&e["event"], KNOWN_EVENTS, "event")?;
        if kind != "custom" && !seen.insert(kind) {
            return Err(Error::new("duplicate standard native event"));
        }
        required_texts(e, &["native_name", "transport"])?;
        boolean(&e["can_block"], "can_block")?;
        choice(&e["context_channel"], CHANNELS, "context_channel")?;
        optional_text(e, "notes")?;
    }
    let c = &v["injection_channel"];
    object(c)?;
    choice(&c["kind"], CHANNELS, "injection_channel.kind")?;
    text(&c["mechanism"], "mechanism")?;
    optional_text(c, "notes")?;
    let b = &v["blocking_semantics"];
    object(b)?;
    choice(
        &b["kind"],
        &["deny-and-block", "advisory-only", "none"],
        "blocking.kind",
    )?;
    optional_text(b, "notes")?;
    let w = &v["wake_capability"];
    object(w)?;
    let kind = choice(
        &w["kind"],
        &["immediate-wake", "next-event", "none"],
        "wake.kind",
    )?;
    optional_text(w, "notes")?;
    if kind == "immediate-wake" {
        text(&w["notes"], "immediate wake listener evidence")?;
    }
    seam(&v["install_seam"])?;
    seam(&v["uninstall_seam"])?;
    if let Some(d) = present(v, "model_dispatch") {
        model_dispatch(d)?;
    }
    let p = &v["provenance"];
    object(p)?;
    text(&p["authored_by"], "authored_by")?;
    if strings(&p["source_refs"], "source_refs")?.is_empty() {
        return Err(Error::new("capability needs authored provenance"));
    }
    Ok(())
}
document!(HarnessCapability, check_capability);
impl HarnessCapability {
    pub fn slug(&self) -> &str {
        self.0["harness_slug"].as_str().unwrap()
    }
    pub fn declared_events(&self) -> &[Value] {
        self.0["native_events"].as_array().unwrap()
    }
    pub fn declares_event(&self, event: &str) -> bool {
        self.declared_events().iter().any(|e| e["event"] == event)
    }
    /// Mechanism and credential requirements, never a provider model list or selection.
    pub fn model_dispatch(&self) -> Option<&Value> {
        present(&self.0, "model_dispatch")
    }
}
