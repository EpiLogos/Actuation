//! Dispatch, help and process plumbing. Everything command-shaped (routes,
//! usage lines, handlers) lives in the COMMANDS table below; this module only
//! matches argv against that table and renders its consequences. Help text,
//! the capabilities listing and dispatch are all derived from the table, so a
//! command cannot exist in one representation and be missing from another.
use crate::surface::{cli_surface, ACTUATION_CLI_VERSION};
use crate::{authority, commands, occupancy};
use actuation_core::Error;
use serde_json::{json, Value};
use std::io::Read;

#[derive(Debug)]
pub struct Output {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Output {
    fn ok(stdout: String) -> Self {
        Self {
            code: 0,
            stdout,
            stderr: String::new(),
        }
    }
}

pub struct Command {
    pub args: Vec<String>,
    pub json: bool,
    pub stdin: String,
}

pub type Handler = fn(&Command) -> Result<Output, Error>;

pub struct CommandDescriptor {
    pub name: &'static str,
    pub route: &'static [&'static str],
    pub usage: &'static str,
    /// What the command is for, in one line: the outcome it produces.
    pub outcome: &'static str,
    pub input: bool,
    pub run: Handler,
}

macro_rules! command {
    ($name:literal, $route:expr, $usage:literal, $outcome:literal, $input:literal, $run:expr) => {
        CommandDescriptor {
            name: $name,
            route: $route,
            usage: $usage,
            outcome: $outcome,
            input: $input,
            run: $run,
        }
    };
}

pub fn commands() -> &'static [CommandDescriptor] {
    COMMANDS
}

/// Longest route first so operation routes win over their read-model parents.
fn route_order() -> Vec<usize> {
    let mut order: Vec<usize> = (0..COMMANDS.len()).collect();
    order.sort_by(|left, right| {
        COMMANDS[*right]
            .route
            .len()
            .cmp(&COMMANDS[*left].route.len())
    });
    order
}

static COMMANDS: &[CommandDescriptor] = &[
    command!("capabilities", &["capabilities"], "actuation capabilities [--json]", "Operator discovery: the descriptors context-selected agents read", false, commands::capabilities),
    command!("contract.list", &["contract", "list"], "actuation contract list [--json]", "List the served contract routes", false, commands::contract_list),
    command!("agency.read", &["agency"], "actuation agency [file|-] [--json]", "The current agency reading for authorised agency work", true, commands::agency_read),
    command!("agency.actualise", &["agency", "actualise"], "actuation agency actualise <file|-> [--schema] [--json]", "Actualise a composed agency request (normally called by the composed work path)", true, commands::agency_actualise),
    command!("realised.read", &["realised"], "actuation realised [file|-] [--json]", "Read what execution actually realised", true, commands::realised_read),
    command!("authority.issue", &["authority", "issue"], "actuation authority issue [--store <dir>] [--now <ts>] [file|-] [--json]", "Issue an explicit authority grant; never implicit in save, project, prepare or start", true, authority::authority_issue),
    command!("authority.resolve", &["authority", "resolve"], "actuation authority resolve [--store <dir>] [--now <ts>] [file|-] [--json]", "Resolve an authority question against its source", true, authority::authority_resolve),
    command!("authority.revoke", &["authority", "revoke"], "actuation authority revoke <authority_source_ref> [--reason <text>] [--store <dir>] [--now <ts>] [--json]", "Revoke an authority explicitly", false, authority::authority_revoke),
    command!("occupancy.claim", &["occupancy", "claim"], "actuation occupancy claim --position <ref> --agent <ref> --agency <ref> [--agent-session <ref>] [--session-space <ref>] [--harness-composition <ref>] [--model <ref>] [--workcell <ref>] [--gateway-address <addr>] --reason <text> [--expect-vacant | --expect-generation <generation>] [--kind initial|handover|fresh|adopt] [--store <dir>] [--json]", "Claim a Position tenure (initial, handover, fresh or adopt) through the owning launcher", false, occupancy::claim),
    command!("occupancy.release", &["occupancy", "release"], "actuation occupancy release --position <ref> --generation <generation> --reason <text> [--store <dir>] [--json]", "Release a Position tenure explicitly", false, occupancy::release),
    command!("occupancy.verify", &["occupancy", "verify"], "actuation occupancy verify --position <ref> --generation <generation> [--store <dir>] [--json]", "Verify a tenure against its expected generation", false, occupancy::verify),
    command!("occupancy.presence", &["occupancy", "presence"], "actuation occupancy presence --position <ref> --generation <generation> --presence active|idle|away|offline [--attention <text>] [--store <dir>] [--json]", "Protocol presence write; explicit operator diagnosis stays available", false, occupancy::presence),
    command!("occupancy.read", &["occupancy", "read"], "actuation occupancy read --position <ref> [--store <dir>] [--json]", "Read the current participation at one Position", false, occupancy::read),
    command!("occupancy.list", &["occupancy", "list"], "actuation occupancy list [--store <dir>] [--json]", "List tenures for participation and recovery", false, occupancy::list),
    command!("stream.read", &["stream"], "actuation stream [file|-] [--json]", "Read an execution trajectory", true, commands::stream_read),
    command!("stream.open", &["stream", "open"], "actuation stream open [--store <dir>] [file|-] [--json]", "Open a stream record (protocol lifecycle, not the real action itself)", true, commands::stream_open),
    command!("stream.record", &["stream", "record"], "actuation stream record [--store <dir>] [file|-] [--json]", "Append an event to a stream (telemetry, not a substitute for the action)", true, commands::stream_record),
    command!("stream.replay", &["stream", "replay"], "actuation stream replay <stream_ref> [--after <n>] [--limit <n>] [--store <dir>] [--json]", "Replay a trajectory after a point", false, commands::stream_replay),
    command!("stream.close", &["stream", "close"], "actuation stream close <stream_ref> [--state closed|interrupted|cancelled] [--ended-at <ts>] [--store <dir>] [--json]", "Close a stream with its true terminal state", false, commands::stream_close),
    command!("activity.read", &["activity"], "actuation activity [file|-] [--json]", "Read aggregated activity evidence", true, commands::activity_read),
    command!("usage.read", &["usage"], "actuation usage [file|-] [--json]", "Read usage evidence", true, commands::usage_read),
    command!("stream.usage", &["stream", "usage"], "actuation stream usage [--store <dir>] [file|-] [--json] [adapter: claude-code-transcript|codex-exec-jsonl|observation]", "Ingest an adapter transcript into the usage read model", true, commands::stream_usage),
    command!("instantiation.read", &["instantiation"], "actuation instantiation [file|-] [--json]", "Read the instantiation read model", true, commands::instantiation_read),
    command!("instantiation.record", &["instantiation", "record"], "actuation instantiation record [--allow-unattributed] [--out <file>] [file|-] [--json]", "Record a supplied instantiation; it cannot become independent observation", true, commands::instantiation_record),
    command!("harness.catalog", &["harness", "catalog"], "actuation harness catalog [--json]", "Declare what this product can detect", false, commands::harness_catalog),
    command!("harness.detect", &["harness", "detect"], "actuation harness detect [--only <slugs>] [--versions] [--json]", "Prove which harnesses exist on this machine", false, commands::harness_detect),
    command!("harness.self", &["harness", "self"], "actuation harness self [--json]", "Identify which harness this process runs inside", false, commands::harness_self),
    command!("harness.capability", &["harness", "capability"], "actuation harness capability [<slug>] [--json]", "Declare the dispatch-relevant harness capabilities", false, commands::harness_capability),
    command!("harness.capability.validate", &["harness", "capability", "validate"], "actuation harness capability validate <file|-> [--json]", "Operator intake: validate a harness capability extension", true, commands::harness_capability_validate),
    command!("system.read", &["system"], "actuation system [--json]", "Owner system disclosure", false, commands::system_read),
    command!("config.contribution", &["config-contribution"], "actuation config-contribution [--json]", "Protocol: the owner configuration contribution document", false, commands::config_contribution),
    command!("config.contribution.capability", &["config-contribution", "capability"], "actuation config-contribution capability <file|-> [--json]", "Protocol intake for a capability contribution", true, commands::config_contribution_capability),
    command!("config.validate", &["config", "validate"], "actuation config validate [--json] [--setting <setting_ref>] [--scope <compact>] [--value <json> | --value-file <path|->]", "Validate settings without applying", false, commands::config_validate),
    command!("config.plan", &["config", "plan"], "actuation config plan [--json] [--setting <setting_ref>] [--scope <compact>] [--value <json> | --value-file <path|->]", "Plan settings changes without applying", false, commands::config_plan),
    command!("config.apply", &["config", "apply"], "actuation config apply [--json] [--plan-file <path|->] [--changeset <id>]", "Apply an owner settings plan", false, commands::config_apply),
    command!("config.reset", &["config", "reset"], "actuation config reset [--json] [--setting <setting_ref>] [--scope <compact>] [--changeset <id>]", "Reset settings through the owner transport", false, commands::config_reset),
    command!("verify", &["verify"], "actuation verify [--json]", "Everyday diagnostic: verify the served surface and read models", false, commands::verify),
];

pub fn match_route(args: &[String]) -> Option<(&'static CommandDescriptor, Vec<String>)> {
    for index in route_order() {
        let entry = &COMMANDS[index];
        if entry
            .route
            .iter()
            .enumerate()
            .all(|(position, word)| args.get(position).map(String::as_str) == Some(*word))
        {
            return Some((entry, args[entry.route.len()..].to_vec()));
        }
    }
    None
}

/// The one dispatch entry point. Errors are semantic refusals: the caller
/// renders them as `actuation: <message>` with exit status 2.
pub fn execute(argv: &[String], stdin: &str) -> Result<Output, Error> {
    let args = strip_flag(argv, "--json");
    let json = argv.contains(&"--json".to_string());
    let command = args.first().cloned();

    if command.is_none()
        || ["help", "--help", "-h"].contains(&command.as_deref().unwrap_or_default())
    {
        return Ok(Output::ok(help_text()));
    }
    if ["--version", "version"].contains(&command.as_deref().unwrap_or_default()) {
        return Ok(Output::ok(format!("actuation {ACTUATION_CLI_VERSION}")));
    }
    let (entry, rest) = match_route(&args).ok_or_else(|| {
        if command.as_deref() == Some("harness") {
            Error::new(format!(
                "unknown harness subcommand {}; expected catalog, detect, self or capability",
                args.get(1).cloned().unwrap_or_else(|| "(none)".into())
            ))
        } else if command.as_deref() == Some("occupancy") {
            Error::new(format!(
                "unknown occupancy subcommand {}; expected claim, release, verify, presence, read or list",
                args.get(1).cloned().unwrap_or_else(|| "(none)".into())
            ))
        } else if command.as_deref() == Some("config") {
            Error::new(format!(
                "unknown config subcommand {}; expected validate, plan, apply or reset",
                args.get(1).cloned().unwrap_or_else(|| "(none)".into())
            ))
        } else {
            Error::new(format!(
                "unknown command {}; run actuation help",
                command.unwrap_or_default()
            ))
        }
    })?;
    (entry.run)(&Command {
        args: rest,
        json,
        stdin: stdin.to_owned(),
    })
}

pub fn help_text() -> String {
    // Every row states what it is for, not only how it is spelled. The usage
    // strings stay the table's own (the README law pins them); the outcome
    // lines are the same law's outcome half.
    let rows = COMMANDS
        .iter()
        .map(|entry| format!("  {}\n      {}", entry.usage, entry.outcome))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Actuation {ACTUATION_CLI_VERSION}\n\nUsage:\n  actuation --version\n{rows}\n\nAuthority is never implicit: issuing, revoking, claiming, releasing and actualising are explicit operator or launcher acts, and the stream lifecycle records evidence rather than performing the corresponding real action."
    )
}

/// stdin is read exactly when an input command has no positional file, or its
/// first positional is "-", so `actuation stream usage --store <dir> -` reads
/// stdin even though the marker follows value flags. The config verbs take no
/// positional document; they read stdin exactly when their file flag is the
/// stdin marker (`--value-file -`, `--plan-file -`).
pub fn command_needs_stdin(argv: &[String]) -> bool {
    let args: Vec<String> = argv
        .iter()
        .filter(|arg| arg.as_str() != "--json")
        .cloned()
        .collect();
    let Some((entry, rest)) = match_route(&args) else {
        return false;
    };
    if entry.input {
        let value_flags = ["--store", "--out"];
        let mut positional = Vec::new();
        let mut index = 0;
        while index < rest.len() {
            let value = &rest[index];
            if value_flags.contains(&value.as_str()) {
                index += 1;
            } else if !value.starts_with("--") {
                positional.push(value.clone());
            }
            index += 1;
        }
        return positional.is_empty() || positional[0] == "-";
    }
    let stdin_markers: &[(&str, &str)] = &[
        ("config.validate", "--value-file"),
        ("config.plan", "--value-file"),
        ("config.apply", "--plan-file"),
    ];
    stdin_markers
        .iter()
        .filter(|(name, _)| *name == entry.name)
        .any(|(_, flag)| {
            rest.iter()
                .position(|arg| arg == flag)
                .and_then(|index| rest.get(index + 1))
                .map(|value| value == "-")
                .unwrap_or(false)
        })
}

pub fn read_stdin() -> String {
    let mut buffer = String::new();
    let _ = std::io::stdin().read_to_string(&mut buffer);
    buffer
}

/// Truthful revision claim: resolved from git at read time, never transcribed.
/// A binary outside a checkout reports "unknown" rather than a stale sha.
pub fn git_revision() -> String {
    let run = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output();
    let sha = match run {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_owned()
        }
        _ => String::new(),
    };
    if is_sha(&sha) {
        sha
    } else {
        "unknown".to_owned()
    }
}

fn is_sha(value: &str) -> bool {
    (7..=40).contains(&value.len())
        && value.chars().all(|c| c.is_ascii_hexdigit())
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

// ---- argv helpers shared by the handlers ----

pub fn remove_flag(args: &mut Vec<String>, flag: &str) -> bool {
    let Some(index) = args.iter().position(|arg| arg == flag) else {
        return false;
    };
    args.remove(index);
    true
}

fn strip_flag(argv: &[String], flag: &str) -> Vec<String> {
    let mut args = argv.to_vec();
    remove_flag(&mut args, flag);
    args
}

pub fn flag_value(args: &mut Vec<String>, flag: &str) -> Result<Option<String>, Error> {
    let Some(index) = args.iter().position(|arg| arg == flag) else {
        return Ok(None);
    };
    let value = args
        .get(index + 1)
        .cloned()
        .ok_or_else(|| Error::new(format!("{flag} requires a value")))?;
    if value.starts_with("--") {
        return Err(Error::new(format!("{flag} requires a value")));
    }
    args.drain(index..=index + 1);
    Ok(Some(value))
}

pub fn positional(args: &[String]) -> Option<String> {
    args.iter().find(|arg| !arg.starts_with("--")).cloned()
}

pub fn read_json_input(path: Option<&str>, stdin: &str) -> Result<Value, Error> {
    let path = path.unwrap_or("-");
    let text = if path == "-" {
        stdin.to_owned()
    } else {
        std::fs::read_to_string(path)
            .map_err(|e| Error::new(format!("no JSON input supplied for {path}: {e}")))?
    };
    if text.trim().is_empty() {
        return Err(Error::new(format!("no JSON input supplied for {path}")));
    }
    serde_json::from_str(&text).map_err(|e| Error::new(format!("invalid JSON input: {e}")))
}

pub fn output(value: Value, json: bool, human: fn(&Value) -> String) -> Result<Output, Error> {
    Ok(if json {
        Output::ok(serde_json::to_string_pretty(&value).map_err(|e| Error::new(e.to_string()))?)
    } else {
        Output::ok(human(&value))
    })
}

pub fn capabilities_listing(revision: String) -> Value {
    let mut value = cli_surface();
    let entries = value.as_object_mut().expect("surface is an object");
    entries.insert(
        "commands".into(),
        json!(COMMANDS.iter().map(|entry| entry.name).collect::<Vec<_>>()),
    );
    entries.insert("revision".into(), json!(revision));
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn longest_route_wins_over_read_model_parents() {
        let (entry, _) = match_route(&argv(&["instantiation", "record", "-", "--json"])).unwrap();
        assert_eq!(entry.name, "instantiation.record");
        let (entry, _) = match_route(&argv(&["instantiation", "-", "--json"])).unwrap();
        assert_eq!(entry.name, "instantiation.read");
    }

    #[test]
    fn every_command_matches_its_own_route() {
        for entry in commands() {
            let (matched, _) = match_route(&argv(entry.route))
                .unwrap_or_else(|| panic!("{} does not match its own route", entry.name));
            assert_eq!(matched.name, entry.name);
        }
    }

    fn closing_for(open: char) -> char {
        if open == '[' {
            ']'
        } else {
            '>'
        }
    }

    #[test]
    fn help_documents_exactly_the_declared_routes() {
        let help = execute(&argv(&["help"]), "").unwrap();
        let mut routes = Vec::new();
        for line in help.stdout.split('\n') {
            if !line.starts_with("  actuation ") {
                continue;
            }
            // Strip bracketed groups entirely, as the served test does: an
            // optional part of a usage line never names a route word.
            let mut cleaned = String::new();
            let mut depth: Option<char> = None;
            for character in line.chars() {
                match depth {
                    Some(open) if character == closing_for(open) => depth = None,
                    Some(_) => {}
                    None if character == '[' || character == '<' => depth = Some(character),
                    None => cleaned.push(character),
                }
            }
            let cleaned = cleaned
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>()
                .join(" ");
            // The route is the leading words; required flags that follow
            // (`--position <ref>`) are arguments, not route words.
            let words: Vec<&str> = cleaned
                .split(' ')
                .skip(1)
                .take_while(|w| !w.starts_with("--"))
                .collect();
            if words.is_empty() {
                continue;
            }
            routes.push(words.join(" "));
        }
        let declared: std::collections::HashSet<String> = commands()
            .iter()
            .map(|entry| entry.route.join(" "))
            .collect();
        assert_eq!(
            std::collections::HashSet::from_iter(routes),
            declared,
            "help usage lines and the command table have diverged"
        );
    }

    #[test]
    fn stdin_needed_for_value_flag_then_marker() {
        assert!(command_needs_stdin(&argv(&[
            "stream", "usage", "--store", "/tmp/x", "-", "--json"
        ])));
        assert!(!command_needs_stdin(&argv(&[
            "stream", "replay", "s:1", "--store", "/tmp/x"
        ])));
        assert!(!command_needs_stdin(&argv(&["capabilities", "--json"])));
        // The config verbs read stdin exactly when their file flag carries
        // the stdin marker; an inline --value never touches stdin.
        assert!(command_needs_stdin(&argv(&[
            "config",
            "validate",
            "--value-file",
            "-",
            "--setting",
            "actuation:return:return.modes",
        ])));
        assert!(command_needs_stdin(&argv(&[
            "config",
            "apply",
            "--plan-file",
            "-",
        ])));
        assert!(!command_needs_stdin(&argv(&[
            "config",
            "apply",
            "--plan-file",
            "/tmp/plan.json",
        ])));
        assert!(!command_needs_stdin(&argv(&[
            "config",
            "validate",
            "--value",
            "[]",
            "--value-file",
            "/tmp/v.json",
        ])));
    }

    #[test]
    fn bare_config_names_its_subcommands() {
        let error = execute(&argv(&["config"]), "").unwrap_err().to_string();
        assert!(
            error.contains("expected validate, plan, apply or reset"),
            "{error}"
        );
        let error = execute(&argv(&["config", "teleport"]), "")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("expected validate, plan, apply or reset"),
            "{error}"
        );
    }

    #[test]
    fn bare_harness_names_its_subcommands() {
        let error = execute(&argv(&["harness"]), "").unwrap_err().to_string();
        assert!(
            error.contains("expected catalog, detect, self or capability"),
            "{error}"
        );
        let error = execute(&argv(&["harness", "teleport"]), "")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("expected catalog, detect, self or capability"),
            "{error}"
        );
    }

    #[test]
    fn bare_occupancy_names_its_subcommands() {
        let error = execute(&argv(&["occupancy"]), "").unwrap_err().to_string();
        assert!(
            error.contains("expected claim, release, verify, presence, read or list"),
            "{error}"
        );
    }

    #[test]
    fn unknown_command_fails_rather_than_fabricating_a_fallback() {
        let error = execute(&argv(&["unknown"]), "").unwrap_err().to_string();
        assert!(error.contains("unknown command"), "{error}");
    }
}
