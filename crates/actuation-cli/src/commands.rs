//! The command handlers. Each one stays thin: product semantics remain in the
//! actuation-* libraries; this layer translates argv, stdin and output
//! envelopes around them.
use crate::dispatch::{
    capabilities_listing, flag_value, git_revision, output, positional, read_json_input,
    remove_flag, Command, Output,
};
use crate::render;
use crate::surface::{cli_surface, ACTUATION_CLI_CONTRACT, ACTUATION_CLI_VERSION};
use crate::system::build_system_disclosure;
use crate::verify;
use actuation_adapters::{
    attach_detection_evidence, effects::NativeEffects, usage, DetectionOptions, HarnessCapability,
    InstantiationReceipt, NativeCatalog, HARNESS_CAPABILITY_VERSION,
};
use actuation_core::{agency_reading, Error, Result, StreamRef};
use actuation_runtime::{ActualisationRequest, RealisedActuation};
use actuation_stream::{
    Activity, ActuationStream, BoundaryOccurrence, Count, JsonlStreamStore, ModelUsageObservation,
    OpenStream, PageRequest, StreamStore, TerminalState, Timestamp, UsageOccurrence,
};
use serde_json::{json, Value};

pub fn capabilities(command: &Command) -> Result<Output> {
    output(
        capabilities_listing(git_revision()),
        command.json,
        render::capabilities,
    )
}

pub fn contract_list(command: &Command) -> Result<Output> {
    output(
        cli_surface()["native_contracts"].clone(),
        command.json,
        render::contracts,
    )
}

pub fn agency_read(command: &Command) -> Result<Output> {
    let input = read_json_input(command.args.first().map(String::as_str), &command.stdin)?;
    output(
        serde_json::to_value(agency_reading(input)?).map_err(|e| Error::new(e.to_string()))?,
        command.json,
        render::agency,
    )
}

pub fn agency_actualise(command: &Command) -> Result<Output> {
    let input = read_json_input(command.args.first().map(String::as_str), &command.stdin)?;
    let request: ActualisationRequest = serde_json::from_value(input)
        .map_err(|e| Error::new(format!("invalid agency actualisation request: {e}")))?;
    output(
        request.admit()?.receipt(),
        command.json,
        render::agency_actualisation,
    )
}

pub fn realised_read(command: &Command) -> Result<Output> {
    let input = read_json_input(command.args.first().map(String::as_str), &command.stdin)?;
    let realised: RealisedActuation = serde_json::from_value(input)
        .map_err(|e| Error::new(format!("invalid realised actuation: {e}")))?;
    output(realised.reading(), command.json, render::realised)
}

pub fn stream_read(command: &Command) -> Result<Output> {
    let input = read_json_input(command.args.first().map(String::as_str), &command.stdin)?;
    let reading = ActuationStream::try_from(input)?.read(PageRequest::default());
    output(
        serde_json::to_value(reading).map_err(|e| Error::new(e.to_string()))?,
        command.json,
        render::stream,
    )
}

fn store(explicit: Option<String>) -> Result<JsonlStreamStore> {
    JsonlStreamStore::from_environment(explicit.as_deref())
}

fn json_error<T: std::fmt::Display>(context: &str, error: T) -> Error {
    Error::new(format!("{context}: {error}"))
}

pub fn stream_open(command: &Command) -> Result<Output> {
    let mut args = command.args.clone();
    let root = flag_value(&mut args, "--store")?;
    let input = read_json_input(positional(&args).as_deref(), &command.stdin)?;
    let opening: OpenStream =
        serde_json::from_value(input).map_err(|e| json_error("invalid stream opening", e))?;
    let reading = store(root)?.open(&opening)?.read(PageRequest::default());
    output(
        serde_json::to_value(reading).map_err(|e| Error::new(e.to_string()))?,
        command.json,
        render::stream,
    )
}

pub fn stream_record(command: &Command) -> Result<Output> {
    let mut args = command.args.clone();
    let root = flag_value(&mut args, "--store")?;
    let input = read_json_input(positional(&args).as_deref(), &command.stdin)?;
    let occurrence: BoundaryOccurrence =
        serde_json::from_value(input).map_err(|e| json_error("invalid boundary occurrence", e))?;
    let catalog = catalog()?;
    let receipt = store(root)?.record_boundary(occurrence, &catalog)?;
    output(
        serde_json::to_value(receipt).map_err(|e| Error::new(e.to_string()))?,
        command.json,
        render::stream_record,
    )
}

pub fn stream_replay(command: &Command) -> Result<Output> {
    let mut args = command.args.clone();
    let root = flag_value(&mut args, "--store")?;
    let after = flag_value(&mut args, "--after")?;
    let limit = flag_value(&mut args, "--limit")?;
    let stream_ref =
        positional(&args).ok_or_else(|| Error::new("stream replay requires a stream_ref"))?;
    let after_sequence: u64 = match after {
        None => 0,
        Some(raw) => raw
            .parse()
            .map_err(|_| Error::new("--after and --limit must be non-negative integers"))?,
    };
    let limit: Option<u64> = match limit {
        None => None,
        Some(raw) => Some(raw.parse::<u64>())
            .transpose()
            .map_err(|_| Error::new("--after and --limit must be non-negative integers"))?,
    };
    let request = PageRequest {
        after_sequence: Count::new(after_sequence)
            .map_err(|_| Error::new("--after and --limit must be non-negative integers"))?,
        limit: limit
            .map(Count::new)
            .transpose()
            .map_err(|_| Error::new("--after and --limit must be non-negative integers"))?,
    };
    let stream_ref: StreamRef =
        StreamRef::new(&stream_ref).map_err(|e| Error::new(format!("invalid stream_ref: {e}")))?;
    let page = store(root)?.replay(&stream_ref, request)?;
    output(
        serde_json::to_value(page).map_err(|e| Error::new(e.to_string()))?,
        command.json,
        render::stream_replay,
    )
}

pub fn stream_close(command: &Command) -> Result<Output> {
    let mut args = command.args.clone();
    let root = flag_value(&mut args, "--store")?;
    let state = flag_value(&mut args, "--state")?;
    let ended_at = flag_value(&mut args, "--ended-at")?;
    let stream_ref =
        positional(&args).ok_or_else(|| Error::new("stream close requires a stream_ref"))?;
    let state: TerminalState = match state {
        None => TerminalState::Closed,
        Some(raw) => {
            serde_json::from_value(json!(raw)).map_err(|e| json_error("invalid stream state", e))?
        }
    };
    let ended = match ended_at {
        None => Timestamp::now()?,
        Some(raw) => serde_json::from_value(json!(raw))
            .map_err(|e| json_error("invalid --ended-at timestamp", e))?,
    };
    let stream_ref: StreamRef =
        StreamRef::new(&stream_ref).map_err(|e| Error::new(format!("invalid stream_ref: {e}")))?;
    let reading = store(root)?
        .close(&stream_ref, state, ended)?
        .read(PageRequest::default());
    output(
        serde_json::to_value(reading).map_err(|e| Error::new(e.to_string()))?,
        command.json,
        render::stream,
    )
}

pub fn activity_read(command: &Command) -> Result<Output> {
    let input = read_json_input(command.args.first().map(String::as_str), &command.stdin)?;
    output(
        serde_json::to_value(Activity::try_from(input)?).map_err(|e| Error::new(e.to_string()))?,
        command.json,
        render::activity,
    )
}

pub fn usage_read(command: &Command) -> Result<Output> {
    let input = read_json_input(command.args.first().map(String::as_str), &command.stdin)?;
    output(
        serde_json::to_value(ModelUsageObservation::try_from(input)?)
            .map_err(|e| Error::new(e.to_string()))?,
        command.json,
        render::usage,
    )
}

pub fn stream_usage(command: &Command) -> Result<Output> {
    let mut args = command.args.clone();
    let root = flag_value(&mut args, "--store")?;
    let input = read_json_input(positional(&args).as_deref(), &command.stdin)?;
    let adapters = ["claude-code-transcript", "codex-exec-jsonl", "observation"];
    let adapter = input["adapter"].as_str().ok_or_else(|| {
        Error::new(format!(
            "stream usage adapter must be one of {}",
            adapters.join(", ")
        ))
    })?;
    let observation = match adapter {
        "claude-code-transcript" => {
            usage::from_claude_code_transcript(&input["native_event"], &input["correlation"])?
        }
        "codex-exec-jsonl" => {
            usage::from_codex_exec_events(&input["native_events"], &input["correlation"])?
        }
        // "observation" trusts its caller to have already normalized the
        // evidence; every other adapter still owns the native-format
        // translation. Either way the result is validated before it ever
        // reaches the durable store.
        "observation" => ModelUsageObservation::try_from(input["native_event"].clone())?,
        other => {
            return Err(Error::new(format!(
                "stream usage adapter must be one of {} (got {other})",
                adapters.join(", ")
            )))
        }
    };
    let occurrence = UsageOccurrence {
        stream_ref: serde_json::from_value(input["stream_ref"].clone())
            .map_err(|e| json_error("invalid stream_ref", e))?,
        observation,
        identity: serde_json::from_value(input["identity"].clone())
            .map_err(|e| json_error("invalid identity", e))?,
        event_ref: serde_json::from_value(input["event_ref"].clone())
            .map_err(|e| json_error("invalid event_ref", e))?,
    };
    let receipt = store(root)?.record_usage(occurrence)?;
    output(
        serde_json::to_value(receipt).map_err(|e| Error::new(e.to_string()))?,
        command.json,
        render::usage_record,
    )
}

pub fn instantiation_read(command: &Command) -> Result<Output> {
    let input = read_json_input(command.args.first().map(String::as_str), &command.stdin)?;
    output(
        InstantiationReceipt::read(input)?.into_value(),
        command.json,
        render::instantiation,
    )
}

pub fn instantiation_record(command: &Command) -> Result<Output> {
    let mut args = command.args.clone();
    let allow_unattributed = remove_flag(&mut args, "--allow-unattributed");
    let out_path = flag_value(&mut args, "--out")?;
    let input = read_json_input(positional(&args).as_deref(), &command.stdin)?;
    let receipt = InstantiationReceipt::read(input)?;
    if receipt.as_value()["harness_ref"].is_null() && !allow_unattributed {
        return Err(Error::new(
            "receipt has no harness_ref; pass --allow-unattributed to record an unattributed instantiation",
        ));
    }
    let detection = run_detection(&[])?;
    let bound = attach_detection_evidence(receipt.as_value(), &detection)?;
    if let Some(out) = &out_path {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(out)
            .map_err(|e| Error::new(format!("cannot append to {out}: {e}")))?;
        writeln!(
            file,
            "{}",
            serde_json::to_string(&bound).map_err(|e| Error::new(e.to_string()))?
        )
        .map_err(|e| Error::new(e.to_string()))?;
    }
    if command.json {
        return Ok(Output {
            code: 0,
            stdout: serde_json::to_string_pretty(&bound).map_err(|e| Error::new(e.to_string()))?,
            stderr: String::new(),
        });
    }
    let line = if bound["unattributed"].as_bool() == Some(true) {
        "unattributed instantiation (no harness binding claimed)".to_owned()
    } else {
        format!(
            "bound to {} via {}",
            bound["harness_ref"].as_str().unwrap_or_default(),
            bound["detection_ref"].as_str().unwrap_or_default(),
        )
    };
    let persisted = out_path
        .map(|path| format!("; appended to {path}"))
        .unwrap_or_default();
    Ok(Output {
        code: 0,
        stdout: format!(
            "Instantiation recorded: {line}; model {}{persisted}",
            bound["model_relation"]["model_ref"]
                .as_str()
                .unwrap_or_default(),
        ),
        stderr: String::new(),
    })
}

fn catalog() -> Result<NativeCatalog> {
    NativeCatalog::bundled()
}

fn run_detection(selected: &[String]) -> Result<Value> {
    let catalog = catalog()?;
    let descriptors = catalog.select(selected).map_err(|_| {
        Error::new(format!(
            "unknown harness slug(s): {}; catalog r{} declares: {}",
            selected.join(", "),
            catalog.revision(),
            catalog
                .descriptors()
                .iter()
                .map(|d| d.slug())
                .collect::<Vec<_>>()
                .join(", "),
        ))
    })?;
    let mut effects = NativeEffects::from_environment()
        .map_err(|e| Error::new(format!("harness probes unavailable: {e}")))?;
    let options = DetectionOptions::now(&catalog)?;
    let record = actuation_adapters::run_detection(&descriptors, &mut effects, &options)?;
    Ok(record.as_value().clone())
}

pub fn harness_catalog(command: &Command) -> Result<Output> {
    let document = catalog()?.read();
    output(document.as_value().clone(), command.json, render::catalog)
}

pub fn harness_detect(command: &Command) -> Result<Output> {
    let mut args = command.args.clone();
    let _probe_versions = remove_flag(&mut args, "--versions");
    let only = flag_value(&mut args, "--only")?;
    let wanted: Vec<String> = match only {
        Some(raw) => raw
            .split(',')
            .map(|slug| slug.trim().to_owned())
            .filter(|slug| !slug.is_empty())
            .collect(),
        None => Vec::new(),
    };
    let record = run_detection(&wanted)?;
    output(record, command.json, render::detection)
}

pub fn harness_self(command: &Command) -> Result<Output> {
    let catalog = catalog()?;
    let mut effects = NativeEffects::from_environment()
        .map_err(|e| Error::new(format!("harness probes unavailable: {e}")))?;
    let options = DetectionOptions::now(&catalog)?;
    let self_value =
        actuation_adapters::resolve_self(catalog.descriptors(), &mut effects, None, &options)?;
    output(
        self_value.as_value().clone(),
        command.json,
        render::harness_self,
    )
}

pub fn harness_capability(command: &Command) -> Result<Output> {
    let slug = positional(&command.args);
    let catalog = catalog()?;
    match slug {
        Some(slug) => {
            let capability: &HarnessCapability = catalog.capability(&slug).ok_or_else(|| {
                Error::new(format!(
                    "no capability descriptor declared for harness {slug}; declared: {}",
                    catalog
                        .capabilities()
                        .iter()
                        .map(|c| c.as_value()["harness_slug"].as_str().unwrap_or_default())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            })?;
            output(
                json!({
                    "schema": HARNESS_CAPABILITY_VERSION,
                    "document": "capability-read-model",
                    "capability": capability.as_value(),
                }),
                command.json,
                render::capability,
            )
        }
        None => output(
            json!({
                "schema": HARNESS_CAPABILITY_VERSION,
                "document": "capability-catalog",
                "catalog_revision": catalog.revision(),
                "capabilities": catalog.capabilities(),
            }),
            command.json,
            render::capability_catalog,
        ),
    }
}

pub fn system_read(command: &Command) -> Result<Output> {
    let catalog = catalog()?;
    let mut effects = NativeEffects::from_environment()
        .map_err(|e| Error::new(format!("harness probes unavailable: {e}")))?;
    let options = DetectionOptions::now(&catalog)?;
    let detection =
        actuation_adapters::run_detection(catalog.descriptors(), &mut effects, &options)?;
    let self_value =
        actuation_adapters::resolve_self(catalog.descriptors(), &mut effects, None, &options)?;
    let disclosure =
        build_system_disclosure(detection.as_value(), self_value.as_value(), now_unix_ms());
    output(disclosure, command.json, render::system)
}

fn now_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

pub fn verify(command: &Command) -> Result<Output> {
    let receipt = verify::run();
    let json_body = json!({
        "contract": ACTUATION_CLI_CONTRACT,
        "product": "actuation",
        "version": ACTUATION_CLI_VERSION,
        "revision": git_revision(),
        "status": receipt.status,
        "tests": receipt.tests,
    });
    if receipt.status != "ok" {
        return Ok(Output {
            code: 1,
            stdout: if command.json {
                serde_json::to_string_pretty(&json_body).map_err(|e| Error::new(e.to_string()))?
            } else {
                "Actuation native verification: failed".into()
            },
            stderr: receipt
                .failure
                .unwrap_or_else(|| "native verification failed".into()),
        });
    }
    Ok(Output {
        code: 0,
        stdout: if command.json {
            serde_json::to_string_pretty(&json_body).map_err(|e| Error::new(e.to_string()))?
        } else {
            format!(
                "Actuation native verification: ok ({} checks)",
                receipt.tests.len()
            )
        },
        stderr: String::new(),
    })
}
