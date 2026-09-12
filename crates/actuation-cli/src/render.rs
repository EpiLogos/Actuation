//! Human-readable renderers for every command output. These mirror the served
//! CLI's text format exactly; the frozen migration scenarios compare them
//! literally, so spacing and punctuation are part of the contract.
use serde_json::Value;

fn pad_end(text: &str, width: usize) -> String {
    format!("{:<width$}", text, width = width)
}

fn pad_start(text: &str, width: usize) -> String {
    format!("{:>width$}", text, width = width)
}

fn text(value: &Value) -> String {
    value.as_str().map(str::to_owned).unwrap_or_default()
}

pub fn capabilities(value: &Value) -> String {
    let contracts = value["native_contracts"]
        .as_object()
        .map(|entries| entries.values().map(text).collect::<Vec<_>>().join(", "))
        .unwrap_or_default();
    format!(
        "Actuation {}\ncommands: {}\ncontracts: {}\nrevision: {}",
        text(&value["version"]),
        value["commands"]
            .as_array()
            .map(|items| items.iter().map(text).collect::<Vec<_>>().join(", "))
            .unwrap_or_default(),
        contracts,
        text(&value["revision"]),
    )
}

pub fn contracts(contracts: &Value) -> String {
    contracts
        .as_object()
        .map(|entries| {
            entries
                .iter()
                .map(|(name, version)| format!("{name}\t{}", text(version)))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

pub fn agency(value: &Value) -> String {
    let metagency = &value["metagency"];
    let operations = if metagency["available"].as_bool() == Some(true) {
        metagency["operations"]
            .as_array()
            .map(|items| items.iter().map(text).collect::<Vec<_>>().join(", "))
            .unwrap_or_default()
    } else {
        "none".into()
    };
    format!(
        "Agency {}\nAgent: {}\nWorld: {}\nScope: {}\nRoot for scope: {}\nMetagency: {}",
        text(&value["agency_ref"]),
        text(&value["agent_ref"]),
        text(&value["world_ref"]),
        text(&value["scope_ref"]),
        if value["root_for_scope"].as_bool() == Some(true) {
            "yes"
        } else {
            "no"
        },
        operations,
    )
}

pub fn agency_actualisation(value: &Value) -> String {
    let binding = &value["differentiated_binding"];
    let return_relation = &value["return_relation"];
    let return_shows = return_relation["return_relation_ref"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| text(&return_relation["mode"]));
    format!(
        "Actualised {}\nAgent: {}\nWorld: {}\nDetermination: {} ({})\nAuthority: {}\nReturn: {}",
        text(&binding["agency_ref"]),
        text(&binding["agent_ref"]),
        text(&binding["world_ref"]),
        text(&value["determination"]["kind"]),
        text(&value["determination"]["determination_ref"]),
        text(&value["metagency"]["grant_ref"]),
        return_shows,
    )
}

pub fn realised(value: &Value) -> String {
    format!(
        "Realised Actuation {}\nActuation: {}\nAgency: {}\nRecurrence: {}\nObservation: {}",
        text(&value["realised_ref"]),
        text(&value["actuation_ref"]),
        text(&value["agency_ref"]),
        text(&value["recurrence"]),
        text(&value["observation"]["state"]),
    )
}

pub fn stream(value: &Value) -> String {
    format!(
        "ActuationStream {}\nActuation: {}\nAgency: {}\nState: {}\nEvents: {}",
        text(&value["stream_ref"]),
        text(&value["actuation_ref"]),
        text(&value["agency_ref"]),
        text(&value["lifecycle"]["state"]),
        value["events"].as_array().map(Vec::len).unwrap_or(0),
    )
}

pub fn stream_record(value: &Value) -> String {
    let event = &value["event"];
    let metadata = match event["metadata"].as_object() {
        Some(entries) => Value::Object(entries.clone()),
        None => Value::Object(Default::default()),
    };
    let boundary = if metadata["boundary"].is_null() {
        "unmapped (custom)".to_owned()
    } else {
        text(&metadata["boundary"])
    };
    let native_event = metadata["native_event"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| text(&event["kind"]));
    format!(
        "Recorded {} ({}) as {}\nStream: {}\nSequence: {}  Next: {}  State: {}",
        native_event,
        boundary,
        text(&event["event_ref"]),
        text(&value["stream_ref"]),
        event["sequence"],
        value["cursor"]["next_sequence"],
        text(&value["lifecycle"]["state"]),
    )
}

pub fn stream_replay(value: &Value) -> String {
    let lines = value["events"]
        .as_array()
        .map(|events| {
            events
                .iter()
                .map(|event| {
                    let observed_at = event["observed_at"].as_str().unwrap_or("");
                    let native = event["metadata"]["native_event"]
                        .as_str()
                        .map(|name| format!(" ({name})"))
                        .unwrap_or_default();
                    format!(
                        "  {}  {} {}  {}{}",
                        pad_start(&event["sequence"].to_string(), 4),
                        pad_end(&text(&event["kind"]), 14),
                        text(&event["event_ref"]),
                        observed_at,
                        native,
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let cursor = &value["cursor"];
    let more = if cursor["has_more"].as_bool() == Some(true) {
        " (more available)"
    } else {
        ""
    };
    let mut out = format!(
        "{}\ncursor: after {}, returned through {}{}",
        stream(value),
        cursor["after_sequence"],
        cursor["returned_through"],
        more,
    );
    if !lines.is_empty() {
        out.push('\n');
        out.push_str(&lines.join("\n"));
    }
    out
}

pub fn activity(value: &Value) -> String {
    format!(
        "Activity {}\n{}\n{} / {}\nSubject: {}\nOwner: {}",
        text(&value["activity_ref"]),
        text(&value["summary"]),
        text(&value["phase"]),
        text(&value["outcome"]),
        text(&value["subject_ref"]),
        text(&value["native_owner"]),
    )
}

pub fn usage(value: &Value) -> String {
    let model = &value["model"];
    let model_shows = model["name"]
        .as_str()
        .map(str::to_owned)
        .or_else(|| model["ref"].as_str().map(str::to_owned))
        .unwrap_or_else(|| text(&model["standing"]));
    let tokens = &value["tokens"];
    let input = tokens["input"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| tokens["input"].to_string());
    let input = if tokens["input"].is_null() {
        "not reported".into()
    } else {
        input
    };
    let output = tokens["output"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| tokens["output"].to_string());
    let output = if tokens["output"].is_null() {
        "not reported".into()
    } else {
        output
    };
    let cost = &value["cost"];
    let cost_shows = cost["amount"].to_string().replace('"', "");
    let cost_shows = if cost["amount"].is_null() {
        text(&cost["standing"])
    } else {
        cost_shows
    };
    format!(
        "Model usage {}\nInvocation: {}\nModel: {}\nTokens: {} in / {} out\nCost: {}",
        text(&value["usage_ref"]),
        text(&value["invocation_ref"]),
        model_shows,
        input,
        output,
        cost_shows,
    )
}

pub fn usage_record(value: &Value) -> String {
    let verb = if value["deduplicated"].as_bool() == Some(true) {
        "Replayed"
    } else {
        "Recorded"
    };
    format!(
        "{} {}\nStream: {}\nSequence: {}",
        verb,
        text(&value["event"]["model_usage"]["usage_ref"]),
        text(&value["stream_ref"]),
        value["event"]["sequence"],
    )
}

pub fn instantiation(value: &Value) -> String {
    let relation = &value["model_relation"];
    let placement = relation["material"]["placement"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| "unspecified".into());
    format!(
        "Instantiation receipt {}\nAgency: {}\nHarness: {}\nModel: {}\nPlacement: {}\nInference surface: {}",
        text(&value["actuation_ref"]),
        text(&value["agency_ref"]),
        value["harness_ref"]
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| "unattributed".into()),
        text(&relation["model_ref"]),
        placement,
        text(&relation["inference_surface"]["contract_ref"]),
    )
}

pub fn system(value: &Value) -> String {
    let settings = value["sections"]
        .as_array()
        .map(|sections| {
            sections
                .iter()
                .map(|section| section["settings"].as_array().map(Vec::len).unwrap_or(0))
                .sum::<usize>()
        })
        .unwrap_or(0);
    format!(
        "Actuation settings disclosure ({})\nProduct: {}  Revision: {}\nAvailability: {}\nSections: {}  Settings: {}  Actions: {}  Obligations: {}\nDigest: {}",
        text(&value["schema"]),
        text(&value["product_id"]),
        text(&value["contract_revision"]),
        text(&value["availability"]["state"]),
        value["sections"].as_array().map(Vec::len).unwrap_or(0),
        settings,
        value["actions"].as_array().map(Vec::len).unwrap_or(0),
        value["obligations"].as_array().map(Vec::len).unwrap_or(0),
        text(&value["owner"]["reading_digest"]),
    )
}

pub fn catalog(value: &Value) -> String {
    let lines = value["descriptors"]
        .as_array()
        .map(|descriptors| {
            descriptors
                .iter()
                .map(|descriptor| {
                    let probes = descriptor["probe"]
                        .as_object()
                        .map(|entries| entries.keys().cloned().collect::<Vec<_>>().join("+"))
                        .unwrap_or_default();
                    let facets = descriptor["facets"]
                        .as_object()
                        .map(|entries| entries.keys().cloned().collect::<Vec<_>>().join(","))
                        .unwrap_or_else(|| "-".into());
                    format!(
                        "  {} {} probes: {} facets: {}",
                        pad_end(&text(&descriptor["slug"]), 20),
                        pad_end(&text(&descriptor["native_kind"]), 15),
                        pad_end(&probes, 28),
                        facets,
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    format!(
        "Harness catalog (r{}, {} declared)\n{}",
        value["catalog_revision"],
        value["descriptors"].as_array().map(Vec::len).unwrap_or(0),
        lines,
    )
}

pub fn capability(value: &Value) -> String {
    let capability = &value["capability"];
    let events = capability["native_events"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .map(|event| {
                    let blocks = if event["can_block"].as_bool() == Some(true) {
                        "yes"
                    } else {
                        "no"
                    };
                    format!(
                        "  {} native: {}  blocks: {}  context: {}",
                        pad_end(&text(&event["event"]), 20),
                        text(&event["native_name"]),
                        blocks,
                        text(&event["context_channel"]),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut out = format!(
        "Capability {} (catalog r{})\nInjection: {} — {}\nBlocking: {}\nWake: {}\nInstall: {} ({})\nNative events:",
        text(&capability["harness_slug"]),
        capability["provenance"]["catalog_revision"],
        text(&capability["injection_channel"]["kind"]),
        text(&capability["injection_channel"]["mechanism"]),
        text(&capability["blocking_semantics"]["kind"]),
        text(&capability["wake_capability"]["kind"]),
        text(&capability["install_seam"]["config_path"]),
        text(&capability["install_seam"]["format"]),
    );
    for line in events {
        out.push('\n');
        out.push_str(&line);
    }
    out
}

pub fn capability_catalog(value: &Value) -> String {
    let lines = value["capabilities"]
        .as_array()
        .map(|capabilities| {
            capabilities
                .iter()
                .map(|capability| {
                    format!(
                        "  {} events: {}  blocking: {} wake: {}",
                        pad_end(&text(&capability["harness_slug"]), 20),
                        capability["native_events"]
                            .as_array()
                            .map(Vec::len)
                            .unwrap_or(0),
                        pad_end(&text(&capability["blocking_semantics"]["kind"]), 15),
                        text(&capability["wake_capability"]["kind"]),
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    format!(
        "Capability catalog (r{}, {} declared)\n{}",
        value["catalog_revision"],
        value["capabilities"].as_array().map(Vec::len).unwrap_or(0),
        lines,
    )
}

pub fn detection(record: &Value) -> String {
    let lines = record["harnesses"]
        .as_array()
        .map(|harnesses| {
            harnesses
                .iter()
                .map(|entry| {
                    let badge = match text(&entry["state"]).as_str() {
                        "detected" => entry["version"]
                            .as_str()
                            .map(str::to_owned)
                            .or_else(|| entry["receipts"]["executable"].as_str().map(str::to_owned))
                            .unwrap_or_default(),
                        "unavailable" => entry["unavailable_reason"]
                            .as_str()
                            .unwrap_or("")
                            .to_owned(),
                        _ => "not installed".into(),
                    };
                    format!(
                        "  {} {} {}",
                        pad_end(&text(&entry["slug"]), 18),
                        pad_end(&text(&entry["state"]), 14),
                        badge,
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let disclosure = record["disclosure"]
        .as_array()
        .filter(|lines| !lines.is_empty())
        .map(|lines| {
            format!(
                "\nDisclosure:\n{}",
                lines
                    .iter()
                    .map(|line| format!("  {}", text(line)))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        })
        .unwrap_or_default();
    format!(
        "Harness detection (catalog r{}, {})\n{}{}",
        record["catalog_revision"],
        text(&record["availability"]),
        lines.join("\n"),
        disclosure,
    )
}

pub fn harness_self(self_value: &Value) -> String {
    let lines = self_value["matched"]
        .as_array()
        .map(|matches| {
            matches
                .iter()
                .map(|matched| {
                    format!(
                        "  {} {}  markers: {}",
                        pad_end(&text(&matched["slug"]), 18),
                        text(&matched["harness_ref"]),
                        matched["markers"]
                            .as_array()
                            .map(|markers| markers.iter().map(text).collect::<Vec<_>>().join(", "))
                            .unwrap_or_default(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let resolved = &self_value["resolved"];
    let headline = if !resolved.is_null() {
        format!(
            "Running inside {} ({})",
            text(&resolved["harness_ref"]),
            resolved["markers"]
                .as_array()
                .map(|markers| markers.iter().map(text).collect::<Vec<_>>().join(", "))
                .unwrap_or_default(),
        )
    } else if self_value["ambiguity"].as_bool() == Some(true) {
        "Ambiguous harness identity — multiple markers matched; nested harnesses are real, the innermost is not guessed".into()
    } else {
        "No catalogued harness markers matched this environment".into()
    };
    let detection_states = &self_value["detection"]["states"];
    let cross_check = if !resolved.is_null() {
        let slug = text(&resolved["slug"]);
        let state = detection_states[&slug].clone();
        if state.as_str() != Some("detected") {
            format!(
                "\n  disclosure: {slug} is not detected on this machine (state {}) — marker identity and machine presence disagree",
                state,
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    let mut out = format!(
        "Harness self (catalog r{}, {})\n{}{}",
        self_value["catalog_revision"],
        text(&self_value["detection_ref"]),
        headline,
        cross_check,
    );
    if !lines.is_empty() {
        out.push('\n');
        out.push_str(&lines.join("\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn usage_renders_missing_evidence_as_not_reported() {
        let value = json!({
            "usage_ref": "model-usage:u1",
            "invocation_ref": "invocation:i1",
            "model": {"standing": "not-reported"},
            "tokens": {"input": null, "output": null},
            "cost": {"standing": "not-reported"}
        });
        assert_eq!(
            usage(&value),
            "Model usage model-usage:u1\nInvocation: invocation:i1\nModel: not-reported\nTokens: not reported in / not reported out\nCost: not-reported"
        );
    }

    #[test]
    fn numeric_tokens_render_without_quotes() {
        let value = json!({
            "usage_ref": "u", "invocation_ref": "i",
            "model": {"name": "m"},
            "tokens": {"input": 5, "output": 2},
            "cost": {"amount": 0.25}
        });
        let rendered = usage(&value);
        assert!(rendered.contains("Tokens: 5 in / 2 out"), "{rendered}");
        assert!(rendered.contains("Cost: 0.25"), "{rendered}");
    }
}
