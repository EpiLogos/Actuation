//! The Night (P′) readings, served natively. A reflect tool puts the MEF
//! register's machinery — each lens with its defined sub-slots and their
//! meanings — to the decision provider AIKit holds elected (`aikit decide`),
//! the worker's own disclosure riding the call, and returns the full giving
//! keyed to QL's lens and sublens references. The subject is pre-lens data;
//! a reading is taken now, never an identity assigned. Nothing here invents a
//! reading: an unelected, unavailable or refusing provider is an error the
//! caller returns to the model as an ordinary tool result.
use crate::{process::ProcessSpec, toolset::NIGHT_TOOLS, Error, Result};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;

pub const READING_SCHEMA: &str = "actuation.night-reading/v1";
pub const TOOLS_SCHEMA: &str = "actuation.night-tools/v1";

/// Lens coverage per reading, from the register's lens sets: a full_text
/// reading covers all twelve; each square covers its own four.
pub const COVERAGE_FULL: [&str; 12] = [
    "L0", "L0'", "L1", "L1'", "L2", "L2'", "L3", "L3'", "L4", "L4'", "L5", "L5'",
];
pub const COVERAGE_BEING: [&str; 4] = ["L0", "L0'", "L5", "L5'"];
pub const COVERAGE_BECOMING: [&str; 4] = ["L1", "L1'", "L4", "L4'"];
pub const COVERAGE_KNOWING: [&str; 4] = ["L2", "L2'", "L3", "L3'"];
/// A frame is selected only after adequate coverage: half the registry.
pub const COVERAGE_REQUIRED: usize = 6;

const MODES: [&str; 6] = [
    "full_text",
    "right_frame",
    "being",
    "becoming",
    "knowing",
    "resonant",
];

/// The carried register: QL's canonical MEF lens register with QL's refs.
fn register() -> &'static Value {
    static REGISTER: OnceLock<Value> = OnceLock::new();
    REGISTER.get_or_init(|| {
        serde_json::from_str(include_str!("mef-lens-register.json"))
            .expect("the carried MEF lens register is JSON")
    })
}

fn lenses() -> &'static [Value] {
    register()["lenses"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
}

fn lens(id: &str) -> Option<&'static Value> {
    lenses().iter().find(|l| l["id"] == id)
}

fn str_of<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or_default()
}

fn sublenses(l: &Value) -> &[Value] {
    l["sublens"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
}

/// The instrument mode each Night tool runs.
pub fn mode_of(tool: &str) -> Option<&'static str> {
    match tool {
        "reflect_full_text" => Some("full_text"),
        "reflect_right_frame" => Some("right_frame"),
        "reflect_being" => Some("being"),
        "reflect_becoming" => Some("becoming"),
        "reflect_knowing" => Some("knowing"),
        "reflect_resonant_frames" => Some("resonant"),
        _ => None,
    }
}

/// The lenses a reading in this mode has run.
pub fn covers(mode: &str) -> &'static [&'static str] {
    match mode {
        "full_text" => &COVERAGE_FULL,
        "being" => &COVERAGE_BEING,
        "becoming" => &COVERAGE_BECOMING,
        "knowing" => &COVERAGE_KNOWING,
        _ => &[],
    }
}

/// The coverage gate: a single frame is selected only after the lenses have
/// been adequately read — selection without coverage is the
/// name-without-machinery failure the register warns of.
pub fn refusal(mode: &str, covered: usize) -> Option<String> {
    (mode == "right_frame" && covered < COVERAGE_REQUIRED).then(|| {
        format!(
            "reflect_right_frame requires adequate coverage first: only {covered} of {} lenses have been read this work. Take a reflect_full_text reading, or square readings covering the registry, before selecting a frame.",
            COVERAGE_FULL.len()
        )
    })
}

fn label(tool: &str) -> &'static str {
    match tool {
        "reflect_full_text" => "Reflect the whole text",
        "reflect_right_frame" => "Reflect the right frame",
        "reflect_being" => "Reflect through Square A",
        "reflect_becoming" => "Reflect through Square B",
        "reflect_knowing" => "Reflect through Square C",
        _ => "Reflect the resonant frames",
    }
}

/// The Night tool supply as a body registers it: Actuation's six reflect
/// tools, their descriptions as the toolset gives them, each executed by
/// the `reflect.read` operation.
pub fn tools() -> Value {
    let parameters = json!({
        "type": "object", "additionalProperties": false, "required": ["subject"],
        "properties": {
            "subject": {"type": "string", "minLength": 1},
            "own_disclosure": {"type": "string"}
        }
    });
    json!({
        "schema": TOOLS_SCHEMA,
        "operation": "reflect.read",
        "coverage_required": COVERAGE_REQUIRED,
        "tools": NIGHT_TOOLS.iter().map(|(name, description)| json!({
            "name": name, "label": label(name), "description": description,
            "parameters": parameters,
        })).collect::<Vec<_>>(),
    })
}

fn slot_line(s: &Value) -> String {
    format!(
        "{} {} — {}",
        str_of(s, "slot"),
        str_of(s, "label"),
        str_of(s, "meaning")
    )
}

fn criterion(s: &Value) -> String {
    format!("{} {}", str_of(s, "slot"), str_of(s, "label"))
}

fn machinery(l: &Value) -> String {
    sublenses(l)
        .iter()
        .map(slot_line)
        .collect::<Vec<_>>()
        .join("; ")
}

fn lens_criteria() -> Value {
    Value::Object(
        lenses()
            .iter()
            .map(|l| {
                let labels: Vec<&str> = sublenses(l).iter().map(|s| str_of(s, "label")).collect();
                let line = format!(
                    "{} {} ({}): {}",
                    str_of(l, "id"),
                    str_of(l, "name"),
                    str_of(l, "ground"),
                    labels.join(" · ")
                );
                (str_of(l, "id").to_owned(), Value::String(line))
            })
            .collect(),
    )
}

fn slot_criteria(l: &Value) -> Value {
    Value::Object(
        sublenses(l)
            .iter()
            .map(|s| (criterion(s), Value::String(slot_line(s))))
            .collect(),
    )
}

fn square_key(mode: &str) -> &'static str {
    match mode {
        "being" => "A",
        "becoming" => "B",
        _ => "C",
    }
}

/// A square's day lenses (register order) and day positions: register rule
/// 4, lens-pairs over position-pairs.
fn square_cells(key: &str) -> (Vec<&'static Value>, Vec<&'static str>) {
    let square = &register()["squares"][key];
    let pair_ids: Vec<&str> = square["lens_pairs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_array)
        .flatten()
        .filter_map(Value::as_str)
        .filter(|id| !id.contains('\''))
        .collect();
    let day = lenses()
        .iter()
        .filter(|l| pair_ids.contains(&str_of(l, "id")))
        .collect();
    let positions = square["positions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|p| !p.contains('\''))
        .collect();
    (day, positions)
}

/// The typed questions a mode puts to the provider, and the reading's shape.
/// Machinery first: every question carries the lens's sub-slots with their
/// authored meanings — a lens named must run its machinery.
pub fn questions(mode: &str, with_disclosure: bool) -> Result<(String, Map<String, Value>)> {
    let mut questions = Map::new();
    let shape = match mode {
        "full_text" => {
            for l in lenses() {
                questions.insert(str_of(l, "id").to_owned(), json!({
                    "type": "choice",
                    "instructions": format!("Apply {} ({}) to the subject as a presented whole. Its machinery: {}. Apply it as it is defined: state where the subject stands at each sub-slot — the distribution is the statement, covering all of them.", str_of(l, "name"), str_of(l, "ground"), machinery(l)),
                    "criteria": slot_criteria(l),
                }));
            }
            "machinery applied as given: each lens run across its defined sub-slots, the distribution stating where the subject stands at each".to_owned()
        }
        "right_frame" => {
            let weigh = if with_disclosure {
                " The worker's own candidate reading rides with this request; weigh it but decide independently."
            } else {
                ""
            };
            questions.insert("right_frame".into(), json!({
                "type": "choice",
                "instructions": format!("Which single lens is most useful for reading this subject now — the frame that, taken up, does the most work? The lens is a reading taken now, never an identity assigned to the subject.{weigh}"),
                "criteria": lens_criteria(),
            }));
            "single-lens selection (12-lens distribution, machinery summarised)".to_owned()
        }
        "resonant" => {
            let weigh = if with_disclosure {
                " The worker's own candidate frames ride with this request; weigh them but decide independently."
            } else {
                ""
            };
            questions.insert("resonant_frames".into(), json!({
                "type": "choice",
                "instructions": format!("Which lens is most strongly resonant with this subject as a whole — the first among the frames in relation? The full ranking is taken from the returned distribution.{weigh}"),
                "criteria": lens_criteria(),
            }));
            "ranked frame relation (full ordering, topk=12)".to_owned()
        }
        "being" | "becoming" | "knowing" => {
            let key = square_key(mode);
            let square = &register()["squares"][key];
            let (day, positions) = square_cells(key);
            for l in &day {
                for p in &positions {
                    let office = str_of(&register()["positions"], p);
                    questions.insert(format!("{}@{p}", str_of(l, "id")), json!({
                        "type": "choice",
                        "instructions": format!("Reading the subject's {office} through {} {} ({}). Its machinery: {}. At which sub-slot does it refract the subject's {office}?", str_of(l, "id"), str_of(l, "name"), str_of(l, "ground"), machinery(l)),
                        "criteria": slot_criteria(l),
                    }));
                }
            }
            let titles: Vec<&str> = square["titles"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect();
            let pairs: Vec<String> = square["lens_pairs"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|p| {
                    format!(
                        "{}↔{}",
                        p[0].as_str().unwrap_or_default(),
                        p[1].as_str().unwrap_or_default()
                    )
                })
                .collect();
            let all_positions: Vec<&str> = square["positions"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect();
            format!(
                "square {key} ({}): {} over {}",
                titles.join(" / "),
                pairs.join(", "),
                all_positions.join("/")
            )
        }
        other => return Err(Error::new(format!("no reflect reading for mode {other}"))),
    };
    Ok((shape, questions))
}

/// The strongest answer: the provider's choice, else the distribution's peak.
fn strongest(answer: &Value) -> Option<String> {
    answer["choice"].as_str().map(str::to_owned).or_else(|| {
        answer["probabilities"]
            .as_object()?
            .iter()
            .filter_map(|(k, v)| v.as_f64().map(|m| (k, m)))
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(k, _)| k.clone())
    })
}

/// The QL sublens reference of a criterion the provider chose.
fn sublens_ref(l: &Value, chosen: Option<&str>) -> Value {
    chosen
        .and_then(|c| sublenses(l).iter().find(|s| criterion(s) == c))
        .map(|s| s["ref"].clone())
        .unwrap_or(Value::Null)
}

fn cell(l: &Value, answer: &Value) -> Value {
    let chosen = strongest(answer);
    json!({
        "lens": l["id"], "ref": l["ref"], "name": l["name"], "ground": l["ground"],
        "strongest": chosen, "strongest_ref": sublens_ref(l, chosen.as_deref()),
        "distribution": answer["probabilities"], "machinery": l["sublens"],
    })
}

/// The full giving of the provider's answers, in the mode's shape.
pub fn shape_reading(mode: &str, shape: String, answers: &Value) -> Value {
    match mode {
        "full_text" => json!({
            "shape": shape,
            "cells": lenses().iter().map(|l| cell(l, &answers[str_of(l, "id")])).collect::<Vec<_>>(),
        }),
        "right_frame" | "resonant" => {
            let answer = &answers[if mode == "right_frame" {
                "right_frame"
            } else {
                "resonant_frames"
            }];
            let mut ranked: Vec<(&String, f64)> = answer["probabilities"]
                .as_object()
                .into_iter()
                .flatten()
                .filter_map(|(k, v)| v.as_f64().map(|m| (k, m)))
                .collect();
            ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
            let selected = strongest(answer);
            json!({
                "shape": shape,
                "selected": selected,
                "selected_ref": selected.as_deref().and_then(lens).map(|l| l["ref"].clone()),
                "ranking": ranked.iter().map(|(id, mass)| {
                    let l = lens(id);
                    json!({"lens": id, "ref": l.map(|l| &l["ref"]), "name": l.map(|l| &l["name"]), "mass": mass})
                }).collect::<Vec<_>>(),
                "rationale": answer["rationale"],
            })
        }
        _ => {
            let key = square_key(mode);
            let square = &register()["squares"][key];
            let (day, positions) = square_cells(key);
            let mut readings = Vec::new();
            for l in &day {
                for p in &positions {
                    let mut reading = cell(l, &answers[format!("{}@{p}", str_of(l, "id"))]);
                    reading["at"] = json!(p);
                    readings.push(reading);
                }
            }
            json!({
                "shape": shape,
                "square": {"key": key, "titles": square["titles"], "lens_pairs": square["lens_pairs"], "positions": square["positions"]},
                "readings": readings,
            })
        }
    }
}

/// The decision provider AIKit holds elected, reached through its own CLI.
struct Decide {
    program: PathBuf,
    environment: BTreeMap<String, String>,
}

impl Decide {
    /// `AIKIT_BIN` when the launcher bound one, else `aikit` on `PATH`. The
    /// CLI resolves the election from its own state, so its home and the
    /// asking session pass through; nothing else of this environment does.
    fn from_env() -> Result<Self> {
        let path = std::env::var("PATH").unwrap_or_default();
        let program = match std::env::var("AIKIT_BIN").ok().filter(|p| !p.is_empty()) {
            Some(bin) => PathBuf::from(bin),
            None => std::env::split_paths(&path)
                .map(|dir| dir.join("aikit"))
                .find(|candidate| candidate.is_file())
                .ok_or_else(|| Error::new("the reflect readings need aikit (set AIKIT_BIN or put aikit on PATH) to reach the elected decision provider"))?,
        };
        let environment = [
            "PATH",
            "HOME",
            "TMPDIR",
            "AIKIT_HOME",
            "AIKIT_SESSION_ID",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_STATE_HOME",
        ]
        .into_iter()
        .filter_map(|key| std::env::var(key).ok().map(|v| (key.to_owned(), v)))
        .collect();
        Ok(Self {
            program,
            environment,
        })
    }

    fn run(&self, args: &[&str], timeout_ms: u64) -> Result<Value> {
        let scratch = tempfile::tempdir()
            .map_err(|e| Error::new(format!("reflect scratch unavailable: {e}")))?;
        let spec = ProcessSpec {
            program: self.program.clone(),
            args: args.iter().map(|a| (*a).to_owned()).collect(),
            cwd: scratch.path().to_owned(),
            environment: self.environment.clone(),
            timeout_ms,
            output_limit: 4 * 1024 * 1024,
        };
        let r = spec.run(b"")?;
        let receipt: Value = serde_json::from_str(&r.stdout).map_err(|_| {
            let why: String = r.stderr.chars().take(300).collect();
            Error::new(format!(
                "aikit decide returned no receipt (exit {:?}): {why}",
                r.code
            ))
        })?;
        if receipt["ok"] != true {
            let why = receipt["error"]["message"].as_str().unwrap_or("refused");
            return Err(Error::new(format!("aikit decide refused: {why}")));
        }
        Ok(receipt["data"].clone())
    }

    /// The elected model, as the provider's own status reports it.
    fn status(&self) -> Result<Value> {
        self.run(&["decide", "status", "--json"], 20_000)
    }

    fn invoke(&self, request: &Value) -> Result<Value> {
        let scratch = tempfile::tempdir()
            .map_err(|e| Error::new(format!("reflect scratch unavailable: {e}")))?;
        let file = scratch.path().join("request.json");
        std::fs::write(
            &file,
            serde_json::to_vec(request).map_err(|e| Error::new(e.to_string()))?,
        )
        .map_err(|e| Error::new(format!("reflect request not written: {e}")))?;
        let file = file.to_string_lossy().into_owned();
        let data = self.run(
            &["decide", "invoke", "--json", "--request-file", &file],
            150_000,
        )?;
        if data["outcome"] != "completed" {
            return Err(Error::new(format!(
                "the decision provider did not complete the reading: {}",
                data["failure"]
            )));
        }
        Ok(data)
    }
}

/// One Night reading for one tool call (`reflect.read`): `{tool | mode,
/// subject, own_disclosure?, coverage?}` where `coverage` lists the lenses
/// this work has already read, so the frame selector's gate holds across a
/// body's calls. Returns the full giving with the lenses it covers.
pub fn read(request: &Value) -> Result<Value> {
    let tool = request["tool"].as_str();
    let mode = match (tool, request["mode"].as_str()) {
        (Some(tool), _) => {
            mode_of(tool).ok_or_else(|| Error::new(format!("no reflect instrument for {tool}")))?
        }
        (None, Some(mode)) => MODES
            .iter()
            .find(|m| **m == mode)
            .copied()
            .ok_or_else(|| Error::new(format!("no reflect reading for mode {mode}")))?,
        (None, None) => return Err(Error::new("reflect.read requires a tool or a mode")),
    };
    let covered: Vec<&str> = request["coverage"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|lens| COVERAGE_FULL.contains(lens))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    if let Some(notice) = refusal(mode, covered.len()) {
        return Err(Error::new(notice));
    }
    let subject = request["subject"].as_str().unwrap_or_default().trim();
    let own = request["own_disclosure"]
        .as_str()
        .filter(|s| !s.trim().is_empty());
    read_with(&Decide::from_env()?, mode, subject, own, tool)
}

/// The same reading for the toolset harness, which keeps its own coverage.
pub fn read_mode(mode: &str, subject: &str, own_disclosure: Option<&str>) -> Result<Value> {
    read_with(
        &Decide::from_env()?,
        mode,
        subject.trim(),
        own_disclosure,
        None,
    )
}

fn read_with(
    decide: &Decide,
    mode: &str,
    subject: &str,
    own: Option<&str>,
    tool: Option<&str>,
) -> Result<Value> {
    if subject.is_empty() {
        return Err(Error::new("a reflect reading requires a subject"));
    }
    let (shape, questions) = questions(mode, own.is_some())?;
    let status = decide.status()?;
    let model = status["selected_model"].as_str().ok_or_else(|| {
        Error::new("no decision model is elected (aikit decide status names none)")
    })?;
    let mut state = json!({"kind": format!("reflect-{mode}"), "subject": subject});
    if let Some(own) = own {
        state["own_disclosure"] = json!(own);
    }
    let started = Instant::now();
    let data = decide.invoke(&json!({"model": model, "state": state, "questions": questions}))?;
    Ok(json!({
        "schema": READING_SCHEMA,
        "tool": tool,
        "mode": mode,
        "subject_chars": subject.chars().count(),
        "reading": shape_reading(mode, shape, &data["answer"]["answers"]),
        "own_disclosure": own,
        "covers": covers(mode),
        "latency_ms": started.elapsed().as_millis() as u64,
        "provider": {
            "mode": status["mode"], "placement": status["placement"], "model": model,
            "invocation_ref": data["invocation_ref"], "standing": data["standing"],
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn the_carried_register_is_qls_twelve_lenses_at_six_articulations() {
        let ids: Vec<&str> = lenses().iter().map(|l| str_of(l, "id")).collect();
        assert_eq!(ids.len(), 12);
        for id in COVERAGE_FULL {
            assert!(ids.contains(&id), "{id} is in the register");
        }
        for l in lenses() {
            let id = str_of(l, "id");
            assert_eq!(str_of(l, "ref"), format!("mef:lens:{id}@1"));
            assert_eq!(sublenses(l).len(), 6, "{id} has six sub-slots");
            for (i, s) in sublenses(l).iter().enumerate() {
                assert_eq!(str_of(s, "ref"), format!("mef:sublens:{id}.{i}@1"));
                assert!(
                    !str_of(s, "meaning").is_empty(),
                    "{id}.{i} carries its meaning"
                );
            }
        }
        // The owner's 28 October ruling (EpiLogos/O-I #707), as QL's register reads.
        let l0: Vec<&str> = sublenses(lens("L0").unwrap())
            .iter()
            .map(|s| str_of(s, "label"))
            .collect();
        assert_eq!(
            &l0[3..],
            [
                "Who/Which/Whereby (./?)",
                "Where/When/Whither",
                "Why-for/Why-not (./?)"
            ]
        );
        // The T levels are the L4′ sub-slots.
        let t: Vec<&str> = sublenses(lens("L4'").unwrap())
            .iter()
            .map(|s| str_of(s, "label"))
            .collect();
        assert_eq!(
            t,
            [
                "Questions",
                "Traces",
                "Challenges",
                "Patterns",
                "Discovery",
                "Insight"
            ]
        );
    }

    #[test]
    fn each_mode_runs_its_machinery() {
        let (_, full) = questions("full_text", false).unwrap();
        assert_eq!(full.len(), 12, "every lens, as its machinery defines");
        for (id, q) in &full {
            let criteria = q["criteria"].as_object().unwrap();
            assert_eq!(criteria.len(), 6, "{id}: all six sub-slots");
            assert!(
                criteria
                    .values()
                    .all(|c| c.as_str().unwrap().contains(" — ")),
                "{id}: meanings carried"
            );
        }
        for (mode, cells) in [
            ("being", ["L0@P0", "L0@P5", "L5@P0", "L5@P5"]),
            ("becoming", ["L1@P1", "L1@P4", "L4@P1", "L4@P4"]),
            ("knowing", ["L2@P2", "L2@P3", "L3@P2", "L3@P3"]),
        ] {
            let (shape, q) = questions(mode, false).unwrap();
            assert_eq!(
                q.keys().map(String::as_str).collect::<Vec<_>>(),
                cells,
                "{mode}: lens-pairs over position-pairs"
            );
            assert!(shape.starts_with(&format!("square {}", square_key(mode))));
        }
        for (mode, key) in [
            ("right_frame", "right_frame"),
            ("resonant", "resonant_frames"),
        ] {
            let (_, q) = questions(mode, true).unwrap();
            assert_eq!(q[key]["criteria"].as_object().unwrap().len(), 12);
            assert!(q[key]["instructions"]
                .as_str()
                .unwrap()
                .contains("decide independently"));
        }
        assert!(questions("tarot", false).is_err());
    }

    #[test]
    fn the_frame_selector_waits_for_coverage() {
        assert!(refusal("right_frame", 4).is_some());
        assert!(refusal("right_frame", COVERAGE_REQUIRED).is_none());
        assert!(refusal("full_text", 0).is_none());
        let refused = read(&json!({"tool": "reflect_right_frame", "subject": "s", "coverage": ["L0", "L0", "L5", "nonsense"]})).unwrap_err();
        assert!(refused.to_string().contains("only 2 of 12"), "{refused}");
    }

    #[test]
    fn the_tool_supply_is_the_toolsets_night_six() {
        let supply = tools();
        let names: Vec<&str> = supply["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            names,
            NIGHT_TOOLS.iter().map(|(n, _)| *n).collect::<Vec<_>>()
        );
        for tool in supply["tools"].as_array().unwrap() {
            assert!(mode_of(tool["name"].as_str().unwrap()).is_some());
            assert_eq!(tool["parameters"]["required"], json!(["subject"]));
        }
    }

    #[test]
    fn the_pinned_body_supply_is_this_supply() {
        let pinned: Value = serde_json::from_str(include_str!(
            "../../../distribution/pi-ql-native/skills/ql-night-reflect/night-tools.json"
        ))
        .unwrap();
        assert_eq!(
            pinned,
            tools(),
            "re-render night-tools.json from reflect.tools"
        );
    }

    /// A stand-in for `aikit` that answers status and returns a fixed answer.
    fn fake_decide(answers: Value) -> (tempfile::TempDir, Decide) {
        let dir = tempfile::tempdir().unwrap();
        let program = dir.path().join("aikit");
        let status = json!({"ok": true, "data": {"selected_model": "kev-latest", "mode": "managed-local", "placement": "127.0.0.1:8019"}});
        let invoked = json!({"ok": true, "data": {"outcome": "completed", "invocation_ref": "invocation/jev/x", "standing": "local-protocol", "answer": {"answers": answers}}});
        std::fs::write(dir.path().join("status.json"), status.to_string()).unwrap();
        std::fs::write(dir.path().join("invoke.json"), invoked.to_string()).unwrap();
        let here = dir.path().display();
        std::fs::write(&program, format!("#!/bin/sh\ncat \"{here}/$2.json\"\n")).unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
        (
            dir,
            Decide {
                program,
                environment: BTreeMap::new(),
            },
        )
    }

    #[test]
    fn a_full_text_reading_returns_each_cell_at_its_ql_reference() {
        let (_dir, decide) = fake_decide(
            json!({"L4'": {"choice": "L4.5' Insight", "probabilities": {"L4.5' Insight": 0.4, "L4.0' Questions": 0.1}}, "L1": {"probabilities": {"L1-2 Efficient Cause": 0.5, "L1-1 Material Cause": 0.2}}}),
        );
        let r = read_with(
            &decide,
            "full_text",
            "Frege: sense and reference",
            Some("a P3 distinction"),
            Some("reflect_full_text"),
        )
        .unwrap();
        assert_eq!(r["schema"], READING_SCHEMA);
        assert_eq!(r["covers"].as_array().unwrap().len(), 12);
        assert_eq!(r["provider"]["model"], "kev-latest");
        let cells = r["reading"]["cells"].as_array().unwrap();
        let t = cells.iter().find(|c| c["lens"] == "L4'").unwrap();
        assert_eq!(t["strongest_ref"], "mef:sublens:L4'.5@1");
        let causal = cells.iter().find(|c| c["lens"] == "L1").unwrap();
        assert_eq!(
            causal["strongest"], "L1-2 Efficient Cause",
            "no choice: the distribution's peak"
        );
        assert_eq!(causal["strongest_ref"], "mef:sublens:L1.2@1");
        let unread = cells.iter().find(|c| c["lens"] == "L0").unwrap();
        assert!(
            unread["strongest"].is_null(),
            "an unanswered lens stays unread, never filled"
        );
    }

    #[test]
    fn a_square_reading_names_each_crossing() {
        let (_dir, decide) = fake_decide(json!({"L4@P1": {"choice": "L4.1 Geworfenheit"}}));
        let r = read_with(&decide, "becoming", "s", None, None).unwrap();
        let readings = r["reading"]["readings"].as_array().unwrap();
        assert_eq!(readings.len(), 4);
        let hit = readings
            .iter()
            .find(|x| x["lens"] == "L4" && x["at"] == "P1")
            .unwrap();
        assert_eq!(hit["strongest_ref"], "mef:sublens:L4.1@1");
        assert_eq!(r["covers"], json!(COVERAGE_BECOMING));
    }
}
