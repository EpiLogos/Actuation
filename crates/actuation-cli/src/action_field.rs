//! Actuation's Action field, as the O:I Action registry holds it (`oi act`,
//! O-I #1002, `oi.owner-action-field/v1`).
//!
//! `actuation actions [--json]` lists the Actions; `actuation action run <id>
//! <json|->` runs one. An Action is a command of the one COMMANDS table: the
//! field names which commands are Actions and the mutation class each has;
//! its inputs are read from the command's own usage line, the line help and
//! the README carry, so the field cannot drift from what the command takes.
//! Running one makes the command's argv from the input, checked by name, and
//! runs it through the same dispatch in this process.
use crate::dispatch::{commands, execute, output, Command, CommandDescriptor, Output};
use actuation_core::Error;
use serde_json::{json, Map, Value};

pub const OWNER_ACTION_FIELD: &str = "oi.owner-action-field/v1";

/// The commands that are Actions, by name, with the mutation class each has.
const ACTIONS: &[(&str, &str)] = &[
    ("occupancy.claim", "locally-mutating"),
    ("occupancy.release", "locally-mutating"),
    ("occupancy.presence", "locally-mutating"),
    ("occupancy.verify", "read-only"),
    ("occupancy.read", "read-only"),
    ("occupancy.list", "read-only"),
    ("authority.issue", "locally-mutating"),
    ("authority.revoke", "locally-mutating"),
    ("authority.resolve", "read-only"),
    ("authority.list", "read-only"),
    ("stream.list", "read-only"),
    ("stream.replay", "read-only"),
    ("usage.list", "read-only"),
    ("harness.catalog", "read-only"),
    ("harness.detect", "read-only"),
    ("harness.self", "read-only"),
    ("harness.capability", "read-only"),
    ("system.read", "read-only"),
    ("verify", "read-only"),
];

#[derive(Debug, Clone, PartialEq)]
enum Part {
    /// `--flag <value>` (with its choices when the usage names them) or a
    /// bare `--flag`, which is a boolean.
    Flag {
        flag: String,
        valued: bool,
        choices: Vec<String>,
    },
    Positional,
    /// `file|-`: the command's JSON document, given on stdin.
    Document,
}

#[derive(Debug, Clone, PartialEq)]
struct Input {
    name: String,
    part: Part,
    required: bool,
}

/// The inputs a usage line takes after its route: required where it stands
/// outside brackets and is the only alternative; `--json` is the run's own.
fn usage_inputs(entry: &CommandDescriptor) -> Vec<Input> {
    let mut text = entry.usage.trim_start_matches("actuation").trim_start();
    for word in entry.route {
        text = text.trim_start().strip_prefix(word).unwrap_or(text);
    }
    // Segments: each bracket group is optional; the words between groups
    // are required.
    let mut segments: Vec<(Vec<String>, bool)> = Vec::new();
    let mut words = Vec::new();
    let mut rest = text.trim();
    while !rest.is_empty() {
        if let Some(inner) = rest.strip_prefix('[') {
            if !words.is_empty() {
                segments.push((std::mem::take(&mut words), false));
            }
            let end = inner.find(']').unwrap_or(inner.len());
            segments.push((
                inner[..end].split_whitespace().map(str::to_owned).collect(),
                true,
            ));
            rest = inner.get(end + 1..).unwrap_or("").trim_start();
        } else {
            let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
            words.push(rest[..end].to_owned());
            rest = rest[end..].trim_start();
        }
    }
    if !words.is_empty() {
        segments.push((words, false));
    }
    let mut inputs = Vec::new();
    for (words, optional) in segments {
        let alternatives = words
            .split(|word| word == "|")
            .map(<[String]>::to_vec)
            .collect::<Vec<_>>();
        let required = !optional && alternatives.len() == 1;
        for alternative in alternatives {
            let mut index = 0;
            while index < alternative.len() {
                let word = &alternative[index];
                if word == "--json" {
                    index += 1;
                } else if let Some(name) = word.strip_prefix("--") {
                    let value = alternative
                        .get(index + 1)
                        .filter(|next| !next.starts_with("--"));
                    let choices = value
                        .filter(|value| !value.starts_with('<'))
                        .map(|value| value.split('|').map(str::to_owned).collect())
                        .unwrap_or_default();
                    inputs.push(Input {
                        name: name.replace('-', "_"),
                        part: Part::Flag {
                            flag: word.clone(),
                            valued: value.is_some(),
                            choices,
                        },
                        required,
                    });
                    index += if value.is_some() { 2 } else { 1 };
                } else if word == "file|-" || word == "<file|->" {
                    inputs.push(Input {
                        name: "document".into(),
                        part: Part::Document,
                        required,
                    });
                    index += 1;
                } else if let Some(name) = word.strip_prefix('<').and_then(|w| w.strip_suffix('>'))
                {
                    inputs.push(Input {
                        name: name.replace('-', "_"),
                        part: Part::Positional,
                        required,
                    });
                    index += 1;
                } else {
                    index += 1;
                }
            }
        }
    }
    inputs
}

fn entry_of(name: &str) -> Option<&'static CommandDescriptor> {
    commands().iter().find(|entry| entry.name == name)
}

fn descriptor(name: &str, mutation_class: &str) -> Value {
    let entry = entry_of(name).expect("every Action is a command of the table");
    let inputs = usage_inputs(entry)
        .into_iter()
        .map(|input| {
            let mut row = json!({"name": input.name, "required": input.required});
            match input.part {
                Part::Flag {
                    flag,
                    valued,
                    choices,
                } => {
                    row["type"] = json!(if valued { "string" } else { "boolean" });
                    row["flag"] = json!(flag);
                    if !choices.is_empty() {
                        row["choices"] = json!(choices);
                    }
                }
                Part::Positional => row["type"] = json!("string"),
                Part::Document => {
                    row["type"] = json!("object");
                    row["stdin"] = json!(true);
                }
            }
            row
        })
        .collect::<Vec<_>>();
    json!({
        "id": format!("actuation.{name}"),
        "title": entry.outcome,
        "description": entry.usage,
        "mutation_class": mutation_class,
        "inputs": inputs,
        "output": {"type": "actuation-json"},
        "route": entry.route,
        "availability": {"available": true, "reason": null},
    })
}

/// The field `actuation actions --json` prints.
pub fn field() -> Value {
    json!({
        "schema": OWNER_ACTION_FIELD,
        "owner": "actuation",
        "actions": ACTIONS.iter().map(|(name, class)| descriptor(name, class)).collect::<Vec<_>>(),
    })
}

/// The argv and stdin an Action's input makes: the route, its positionals in
/// order, its flags, `-` for its document, then `--json`. An input the
/// command does not take, a required one missing, a value of the wrong
/// shape or outside its choices is refused by name before anything runs.
fn route_argv(id: &str, input: &Value) -> Result<(Vec<String>, String), String> {
    let entry = id
        .strip_prefix("actuation.")
        .filter(|name| ACTIONS.iter().any(|(action, _)| action == name))
        .and_then(entry_of)
        .ok_or_else(|| {
            format!("`{id}` is not in Actuation's Action field; `actuation actions` lists it")
        })?;
    let given = input
        .as_object()
        .ok_or_else(|| format!("{id} takes one JSON object of named inputs"))?;
    let inputs = usage_inputs(entry);
    if let Some(name) = given
        .keys()
        .find(|name| !inputs.iter().any(|input| &input.name == *name))
    {
        return Err(format!("{id} takes no input `{name}`"));
    }
    let mut argv = entry
        .route
        .iter()
        .map(|word| word.to_string())
        .collect::<Vec<_>>();
    let (mut positionals, mut flags, mut stdin) = (Vec::new(), Vec::new(), String::new());
    for input in &inputs {
        let value = match given.get(&input.name) {
            None | Some(Value::Null) if input.required => {
                return Err(format!("{id} needs its input `{}`", input.name))
            }
            None | Some(Value::Null) => continue,
            Some(value) => value,
        };
        let text = || match value {
            Value::String(text) => Ok(text.clone()),
            Value::Number(number) => Ok(number.to_string()),
            _ => Err(format!("{id}'s input `{}` is one value", input.name)),
        };
        match &input.part {
            Part::Flag {
                flag,
                valued: false,
                ..
            } => match value {
                Value::Bool(true) => flags.push(flag.clone()),
                Value::Bool(false) => {}
                _ => return Err(format!("{id}'s input `{}` is true or false", input.name)),
            },
            Part::Flag { flag, choices, .. } => {
                let text = text()?;
                if !choices.is_empty() && !choices.contains(&text) {
                    return Err(format!(
                        "{id}'s input `{}` is one of {}",
                        input.name,
                        choices.join(", ")
                    ));
                }
                flags.extend([flag.clone(), text]);
            }
            Part::Positional => positionals.push(text()?),
            Part::Document => {
                if !value.is_object() {
                    return Err(format!("{id}'s input `document` is one JSON object"));
                }
                stdin = value.to_string();
            }
        }
    }
    argv.extend(positionals);
    argv.extend(flags);
    if !stdin.is_empty() {
        argv.push("-".into());
    }
    argv.push("--json".into());
    Ok((argv, stdin))
}

/// `actuation actions [--json]`.
pub fn actions(command: &Command) -> Result<Output, Error> {
    output(field(), command.json, |field| {
        field["actions"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|action| {
                format!(
                    "  {:<32} {:<18} {}",
                    action["id"].as_str().unwrap_or_default(),
                    action["mutation_class"].as_str().unwrap_or_default(),
                    action["title"].as_str().unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    })
}

/// `actuation action run <id> <json|->`: the Action's command, run through
/// the one dispatch in this process.
pub fn run(command: &Command) -> Result<Output, Error> {
    let id = command
        .args
        .first()
        .ok_or_else(|| Error::new("usage: actuation action run <id> <json|->"))?;
    let raw = match command.args.get(1).map(String::as_str) {
        None => "{}",
        Some("-") => command.stdin.as_str(),
        Some(raw) => raw,
    };
    let input = if raw.trim().is_empty() {
        Value::Object(Map::new())
    } else {
        serde_json::from_str::<Value>(raw)
            .map_err(|error| Error::new(format!("{id} takes one JSON object: {error}")))?
    };
    let (argv, stdin) = route_argv(id, &input).map_err(Error::new)?;
    execute(&argv, &stdin)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input<'a>(action: &'a Value, name: &str) -> &'a Value {
        action["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|input| input["name"] == name)
            .unwrap_or_else(|| panic!("{} takes {name}", action["id"]))
    }

    #[test]
    fn the_field_reads_each_actions_inputs_from_its_usage_line() {
        let field = field();
        assert_eq!(
            (field["schema"].as_str(), field["owner"].as_str()),
            (Some(OWNER_ACTION_FIELD), Some("actuation"))
        );
        let actions = field["actions"].as_array().unwrap();
        assert_eq!(actions.len(), ACTIONS.len());
        let claim = actions
            .iter()
            .find(|action| action["id"] == "actuation.occupancy.claim")
            .unwrap();
        for required in ["position", "agent", "agency", "reason"] {
            assert_eq!(input(claim, required)["required"], true, "{required}");
        }
        assert_eq!(input(claim, "agent_session")["required"], false);
        assert_eq!(input(claim, "expect_vacant")["type"], "boolean");
        assert_eq!(
            input(claim, "expect_vacant")["required"],
            false,
            "one of two alternatives"
        );
        assert_eq!(input(claim, "expect_generation")["type"], "string");
        assert_eq!(
            input(claim, "kind")["choices"],
            json!(["initial", "handover", "fresh", "adopt"])
        );
        assert!(claim["inputs"]
            .as_array()
            .unwrap()
            .iter()
            .all(|input| input["name"] != "json"));
        let revoke = actions
            .iter()
            .find(|action| action["id"] == "actuation.authority.revoke")
            .unwrap();
        assert_eq!(
            revoke["inputs"][0],
            json!({"name": "authority_source_ref", "required": true, "type": "string"})
        );
        let issue = actions
            .iter()
            .find(|action| action["id"] == "actuation.authority.issue")
            .unwrap();
        assert_eq!(input(issue, "document")["stdin"], true);
        assert_eq!(input(issue, "document")["required"], false);
        let capability = actions
            .iter()
            .find(|action| action["id"] == "actuation.harness.capability")
            .unwrap();
        assert_eq!(input(capability, "slug")["required"], false);
    }

    #[test]
    fn an_input_makes_the_commands_argv_and_is_refused_by_name() {
        let (argv, stdin) = route_argv(
            "actuation.occupancy.presence",
            &json!({"position": "position:a", "generation": 3, "presence": "idle", "store": "/s"}),
        )
        .unwrap();
        assert_eq!(
            argv,
            [
                "occupancy",
                "presence",
                "--position",
                "position:a",
                "--generation",
                "3",
                "--presence",
                "idle",
                "--store",
                "/s",
                "--json"
            ]
        );
        assert!(stdin.is_empty());
        let (argv, stdin) = route_argv(
            "actuation.authority.issue",
            &json!({"store": "/s", "document": {"authority_source_ref": "a"}}),
        )
        .unwrap();
        assert_eq!(argv, ["authority", "issue", "--store", "/s", "-", "--json"]);
        assert_eq!(stdin, r#"{"authority_source_ref":"a"}"#);
        for (id, input, words) in [
            (
                "actuation.occupancy.read",
                json!({"position": "p", "colour": "red"}),
                "takes no input `colour`",
            ),
            (
                "actuation.occupancy.read",
                json!({}),
                "needs its input `position`",
            ),
            (
                "actuation.occupancy.presence",
                json!({"position": "p", "generation": 1, "presence": "asleep"}),
                "is one of active, idle, away, offline",
            ),
            (
                "actuation.harness.detect",
                json!({"versions": "yes"}),
                "true or false",
            ),
            (
                "actuation.authority.issue",
                json!({"document": "x"}),
                "is one JSON object",
            ),
            ("actuation.occupancy.list", json!([]), "one JSON object"),
            (
                "actuation.stream.open",
                json!({}),
                "not in Actuation's Action field",
            ),
        ] {
            let refused = route_argv(id, &input).unwrap_err();
            assert!(refused.contains(words), "{id}: {refused}");
        }
    }

    #[test]
    fn a_position_is_claimed_read_and_released_through_the_field() {
        let store =
            std::env::temp_dir().join(format!("actuation-action-field-{}", std::process::id()));
        let store = store.to_string_lossy().into_owned();
        let run_action = |id: &str, input: Value| -> Value {
            let ran = run(&Command {
                args: vec![id.into(), "-".into()],
                json: true,
                stdin: input.to_string(),
            })
            .unwrap_or_else(|error| panic!("{id} refused: {error}"));
            assert_eq!(ran.code, 0, "{id}: {}", ran.stderr);
            serde_json::from_str(&ran.stdout).unwrap()
        };
        let claimed = run_action(
            "actuation.occupancy.claim",
            json!({"position": "position:action-field", "agent": "agent:field", "agency": "agency:field",
                   "reason": "the field's own test", "expect_vacant": true, "store": store}),
        );
        let generation = claimed["generation"]["generation_ref"]
            .as_str()
            .unwrap_or_else(|| panic!("the claim names its generation: {claimed}"))
            .to_owned();
        let read = run_action(
            "actuation.occupancy.read",
            json!({"position": "position:action-field", "store": store}),
        );
        assert!(read.to_string().contains("agent:field"), "{read}");
        run_action(
            "actuation.occupancy.release",
            json!({"position": "position:action-field", "generation": generation, "reason": "done", "store": store}),
        );
        let _ = std::fs::remove_dir_all(&store);
    }
}
