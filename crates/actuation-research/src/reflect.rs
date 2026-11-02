//! The Night (P′) readings, served natively. A reflect tool puts the MEF
//! register's machinery — each lens with its defined sub-slots and their
//! meanings — to the decision provider AIKit holds elected (`aikit decide`),
//! the worker's own disclosure riding the call, and returns the full giving
//! keyed to QL's lens and sublens references. A lens is read across the
//! whole of its machinery: every sub-slot is weighed on its own, and a
//! reading is the profile, never one winning cell. A square reads each of its
//! positions through that position's L-link pair. The subject is pre-lens
//! data; a reading is taken now, never an identity assigned. Nothing here
//! invents a reading: an unelected, unavailable or refusing provider is an
//! error the caller returns to the model as an ordinary tool result.
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

/// A sub-slot holds when its weight is above one half — strictly, as AIKit
/// reads a Noul: exactly one half is a coin flip, not a standing.
pub const HOLDS: f64 = 0.5;

/// The relational contract the readings are taken under (Series 1 round 12,
/// `relational-contract.min.txt`). With it the model read the cells as
/// relations to its own text rather than as content.
pub const RELATIONAL_CONTRACT: &str = "Relational contract: the P positions P0–P5 and their conjugates P0′–P5′ are purely relational responsibilities in the recurrence of work; they hold no inherent manifest content. The twelve MEF lenses are the refraction media of these positions: whatever content a position carries is read through a lens, and a lens reading is always of a position's responsibility — never of the thing itself. Readings inform; the work remains yours.";

/// What reaches the body's system prompt with the Night tools.
pub const PROMPT_GUIDELINES: [&str; 2] = [
    RELATIONAL_CONTRACT,
    "The Night reflect tools read any thought — the person's words, your own answer, a summary — through the MEF lenses. Take a reading when seeing what a thought carries would change what you say; the reading is yours to think and answer through.",
];

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

/// The day position whose responsibility a lens refracts: the register's
/// L-links give each lens to exactly one position.
fn refracts(id: &str) -> Value {
    register()["p_links"]
        .as_object()
        .into_iter()
        .flatten()
        .find(|(_, link)| link.as_array().is_some_and(|l| l.iter().any(|x| x == id)))
        .map_or(
            Value::Null,
            |(p, _)| json!({"at": p, "office": register()["positions"][p]}),
        )
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
/// tools, their descriptions as the toolset gives them, the relational
/// contract as their prompt guideline, each executed by the `reflect.read`
/// operation.
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
            "parameters": parameters, "prompt_guidelines": PROMPT_GUIDELINES,
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

fn machinery(l: &Value) -> String {
    sublenses(l)
        .iter()
        .map(slot_line)
        .collect::<Vec<_>>()
        .join("; ")
}

fn lens_line(l: &Value) -> String {
    let labels: Vec<&str> = sublenses(l).iter().map(|s| str_of(s, "label")).collect();
    format!(
        "{} {} ({}): {}",
        str_of(l, "id"),
        str_of(l, "name"),
        str_of(l, "ground"),
        labels.join(" · ")
    )
}

fn lens_criteria() -> Value {
    Value::Object(
        lenses()
            .iter()
            .map(|l| (str_of(l, "id").to_owned(), Value::String(lens_line(l))))
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

/// A square's positions, each with its L-link pair (register §2): Square A
/// reads P0 through L0↔L5′ and P5 through L5↔L0′, and so on for B and C.
/// Every lens of the square is read once, at the position it links.
fn square_links(key: &str) -> Vec<(&'static str, Vec<&'static Value>)> {
    register()["squares"][key]["positions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|p| !p.contains('\''))
        .map(|p| {
            let link = register()["p_links"][p]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|id| id.as_str().and_then(lens))
                .collect();
            (p, link)
        })
        .collect()
}

/// One sub-slot judged on its own: a lens is read across all six of its
/// sub-slots, and several can hold at once, so each is a Noul, never a
/// choice among them.
fn slot_question(l: &Value, s: &Value, reading: &str) -> Value {
    json!({
        "type": "noul",
        "instructions": format!("{reading} through {} {} ({}). Its machinery, six sub-slots: {}. Does it stand at {}? Several sub-slots can hold at once; judge this one alone.", str_of(l, "id"), str_of(l, "name"), str_of(l, "ground"), machinery(l), slot_line(s)),
    })
}

fn slot_key(l: &Value, at: Option<&str>, s: &Value) -> String {
    match at {
        Some(p) => format!("{}@{p}|{}", str_of(l, "id"), str_of(s, "slot")),
        None => format!("{}|{}", str_of(l, "id"), str_of(s, "slot")),
    }
}

fn square_shape(key: &str) -> String {
    let square = &register()["squares"][key];
    let titles: Vec<&str> = square["titles"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let links: Vec<String> = square_links(key)
        .iter()
        .map(|(p, link)| {
            let ids: Vec<&str> = link.iter().map(|l| str_of(l, "id")).collect();
            format!(
                "{p} {} through {}",
                str_of(&register()["positions"], p),
                ids.join("↔")
            )
        })
        .collect();
    format!(
        "square {key} ({}): {}",
        titles.join(" / "),
        links.join("; ")
    )
}

/// The typed questions a mode puts to the provider, and the reading's shape.
/// Machinery first: every question carries the lens's sub-slots with their
/// authored meanings — a lens named must run its machinery.
pub fn questions(mode: &str, with_disclosure: bool) -> Result<(String, Map<String, Value>)> {
    let mut questions = Map::new();
    let shape = match mode {
        "full_text" => {
            for l in lenses() {
                for s in sublenses(l) {
                    questions.insert(
                        slot_key(l, None, s),
                        slot_question(l, s, "Read the subject as a presented whole"),
                    );
                }
            }
            "each lens run across all six of its sub-slots, each sub-slot weighed on its own"
                .to_owned()
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
            for l in lenses() {
                questions.insert(str_of(l, "id").to_owned(), json!({
                    "type": "noul",
                    "instructions": format!("Is {} a frame in play for this subject as a whole — a lens whose reading bears on it now? Judge this lens alone; any number of lenses can be in play.{weigh}", lens_line(l)),
                }));
            }
            "every lens weighed on its own as a frame in play, ranked".to_owned()
        }
        "being" | "becoming" | "knowing" => {
            let key = square_key(mode);
            for (p, link) in square_links(key) {
                let office = str_of(&register()["positions"], p);
                let reading = format!("Read the subject's {office} ({p})");
                for l in link {
                    for s in sublenses(l) {
                        questions.insert(slot_key(l, Some(p), s), slot_question(l, s, &reading));
                    }
                }
            }
            square_shape(key)
        }
        other => return Err(Error::new(format!("no reflect reading for mode {other}"))),
    };
    Ok((shape, questions))
}

/// A lens read across its six sub-slots: each sub-slot's weight (the Noul,
/// null when the provider left it unanswered) and the sub-slots that hold.
fn profile(l: &Value, at: Option<&str>, answers: &Value) -> Value {
    let slots: Vec<Value> = sublenses(l)
        .iter()
        .map(|s| {
            json!({
                "slot": s["slot"], "ref": s["ref"], "label": s["label"], "meaning": s["meaning"],
                "weight": answers[slot_key(l, at, s)]["noul"].as_f64(),
            })
        })
        .collect();
    let holds: Vec<Value> = slots
        .iter()
        .filter(|s| s["weight"].as_f64().is_some_and(|w| w > HOLDS))
        .map(|s| s["ref"].clone())
        .collect();
    json!({
        "lens": l["id"], "ref": l["ref"], "name": l["name"], "ground": l["ground"],
        "refracts": refracts(str_of(l, "id")), "slots": slots, "holds": holds,
    })
}

/// The full giving of the provider's answers, in the mode's shape.
pub fn shape_reading(mode: &str, shape: String, answers: &Value) -> Value {
    match mode {
        "full_text" => json!({
            "shape": shape,
            "lenses": lenses().iter().map(|l| profile(l, None, answers)).collect::<Vec<_>>(),
        }),
        "right_frame" => {
            let answer = &answers["right_frame"];
            let mut ranked: Vec<(&String, f64)> = answer["probabilities"]
                .as_object()
                .into_iter()
                .flatten()
                .filter_map(|(k, v)| v.as_f64().map(|m| (k, m)))
                .collect();
            ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
            let selected = answer["choice"]
                .as_str()
                .or(ranked.first().map(|(id, _)| id.as_str()));
            json!({
                "shape": shape,
                "selected": selected,
                "selected_ref": selected.and_then(lens).map(|l| l["ref"].clone()),
                "ranking": ranked.iter().map(|(id, mass)| {
                    let l = lens(id);
                    json!({"lens": id, "ref": l.map(|l| &l["ref"]), "name": l.map(|l| &l["name"]), "mass": mass})
                }).collect::<Vec<_>>(),
                "rationale": answer["rationale"],
            })
        }
        "resonant" => {
            let mut frames: Vec<Value> = lenses()
                .iter()
                .map(|l| json!({"lens": l["id"], "ref": l["ref"], "name": l["name"], "weight": answers[str_of(l, "id")]["noul"].as_f64()}))
                .collect();
            frames.sort_by(|a, b| {
                let w = |v: &Value| v["weight"].as_f64().unwrap_or(-1.0);
                w(b).total_cmp(&w(a))
            });
            let in_play: Vec<Value> = frames
                .iter()
                .filter(|f| f["weight"].as_f64().is_some_and(|w| w > HOLDS))
                .map(|f| f["lens"].clone())
                .collect();
            json!({"shape": shape, "frames": frames, "in_play": in_play})
        }
        _ => {
            let key = square_key(mode);
            let square = &register()["squares"][key];
            let positions: Vec<Value> = square_links(key)
                .into_iter()
                .map(|(p, link)| json!({
                    "at": p,
                    "office": register()["positions"][p],
                    "link": link.iter().map(|l| &l["id"]).collect::<Vec<_>>(),
                    "lenses": link.iter().map(|l| profile(l, Some(p), answers)).collect::<Vec<_>>(),
                }))
                .collect();
            json!({
                "shape": shape,
                "square": {"key": key, "titles": square["titles"], "lens_pairs": square["lens_pairs"], "positions": square["positions"]},
                "positions": positions,
            })
        }
    }
}

fn weight_text(w: &Value) -> String {
    w.as_f64()
        .map_or_else(|| "unread".to_owned(), |w| format!("{w:.2}"))
}

/// One lens's profile as the reader sees through it: whose responsibility the
/// lens refracts, then all six sub-slots with what each means, the ones that
/// hold first, strongest first.
fn profile_block(p: &Value) -> String {
    let mut slots: Vec<&Value> = p["slots"].as_array().into_iter().flatten().collect();
    slots.sort_by(|a, b| {
        let w = |v: &Value| v["weight"].as_f64().unwrap_or(-1.0);
        w(b).total_cmp(&w(a))
    });
    let weight = |s: &&Value| s["weight"].as_f64();
    let holding: Vec<&Value> = slots
        .iter()
        .copied()
        .filter(|s| weight(s).is_some_and(|w| w > HOLDS))
        .collect();
    let rest: Vec<&Value> = slots
        .iter()
        .copied()
        .filter(|s| weight(s).is_some_and(|w| w <= HOLDS))
        .collect();
    let unread: Vec<&Value> = slots
        .iter()
        .copied()
        .filter(|s| weight(s).is_none())
        .collect();
    let list = |group: &[&Value]| -> String {
        group
            .iter()
            .map(|s| {
                let w = s["weight"]
                    .as_f64()
                    .map_or_else(String::new, |w| format!(" {w:.2}"));
                format!(
                    "{} {}{w}: {}",
                    str_of(s, "slot"),
                    str_of(s, "label"),
                    str_of(s, "meaning")
                )
            })
            .collect::<Vec<_>>()
            .join(" · ")
    };
    let mut out = vec![format!(
        "{} {} ({}), refracting {}, {}",
        str_of(p, "lens"),
        str_of(p, "name"),
        str_of(p, "ground"),
        str_of(&p["refracts"], "at"),
        str_of(&p["refracts"], "office")
    )];
    if !holding.is_empty() {
        out.push(format!("  holds — {}", list(&holding)));
    }
    if !rest.is_empty() {
        out.push(format!("  below one half — {}", list(&rest)));
    }
    if !unread.is_empty() {
        out.push(format!("  unread — {}", list(&unread)));
    }
    out.join("\n")
}

/// One lens weighed as a whole frame, with the position it refracts.
fn frame_text(f: &Value, weight: &Value) -> String {
    let lens = str_of(f, "lens");
    format!(
        "{lens} {} (refracting {}) {}",
        str_of(f, "name"),
        str_of(&refracts(lens), "at"),
        weight_text(weight)
    )
}

/// The reading as the model reads it: the profiles in plain lines, every
/// weight shown. The receipt keeps the same reading as data.
pub fn giving(mode: &str, reading: &Value) -> String {
    let blocks = |v: &Value| -> Vec<String> {
        v.as_array()
            .into_iter()
            .flatten()
            .map(profile_block)
            .collect()
    };
    match mode {
        "full_text" => format!(
            "The subject read through the twelve lenses. Each lens refracts one position's responsibility and is read across its six sub-slots; a sub-slot holds above 0.50, and several can hold at once.\n{}",
            blocks(&reading["lenses"]).join("\n")
        ),
        "right_frame" => {
            let ranking: Vec<String> = reading["ranking"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|r| frame_text(r, &r["mass"]))
                .collect();
            format!(
                "The right frame now: {}. The ranking: {}.",
                reading["selected"].as_str().unwrap_or("none"),
                ranking.join(" · ")
            )
        }
        "resonant" => {
            let frames: Vec<String> = reading["frames"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|f| frame_text(f, &f["weight"]))
                .collect();
            format!(
                "The twelve lenses, each weighed on its own as a frame in play (in play above 0.50): {}.",
                frames.join(" · ")
            )
        }
        _ => {
            let mut out = vec![format!(
                "{}; a sub-slot holds above 0.50, and several can hold at once.",
                str_of(reading, "shape")
            )];
            for p in reading["positions"].as_array().into_iter().flatten() {
                out.extend(blocks(&p["lenses"]));
            }
            out.join("\n")
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
    let reading = shape_reading(mode, shape, &data["answer"]["answers"]);
    Ok(json!({
        "schema": READING_SCHEMA,
        "tool": tool,
        "mode": mode,
        "subject_chars": subject.chars().count(),
        "reading": reading,
        "giving": giving(mode, &reading),
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
        assert_eq!(full.len(), 72, "every lens across all six sub-slots");
        for (key, q) in &full {
            assert_eq!(q["type"], "noul", "{key}: each sub-slot weighed on its own");
            let (id, slot) = key.split_once('|').unwrap();
            let l = lens(id).unwrap();
            let s = sublenses(l)
                .iter()
                .find(|s| str_of(s, "slot") == slot)
                .unwrap();
            let text = q["instructions"].as_str().unwrap();
            assert!(text.contains(&machinery(l)), "{key}: the lens's machinery");
            assert!(
                text.contains(&format!("Does it stand at {}?", slot_line(s))),
                "{key}: this sub-slot, with its meaning"
            );
        }
        assert!(full.contains_key("L4'|L4.5'"));
        // A square reads each of its positions through that position's
        // L-link pair (register §2): every lens it covers is read, the night
        // lenses included, and nothing outside the links.
        for (mode, cells) in [
            ("being", ["L0@P0", "L5'@P0", "L5@P5", "L0'@P5"]),
            ("becoming", ["L1@P1", "L4'@P1", "L4@P4", "L1'@P4"]),
            ("knowing", ["L2@P2", "L3'@P2", "L3@P3", "L2'@P3"]),
        ] {
            let (shape, q) = questions(mode, false).unwrap();
            assert_eq!(q.len(), 24, "{mode}: four lenses across six sub-slots");
            let read: std::collections::BTreeSet<&str> =
                q.keys().map(|k| k.split('|').next().unwrap()).collect();
            assert_eq!(read, cells.into_iter().collect(), "{mode}: the L-links");
            let lenses_read: std::collections::BTreeSet<&str> =
                cells.iter().map(|c| c.split('@').next().unwrap()).collect();
            assert_eq!(
                lenses_read,
                covers(mode).iter().copied().collect(),
                "{mode}: a square covers exactly the lenses it reads"
            );
            assert!(shape.starts_with(&format!("square {}", square_key(mode))));
        }
        let (_, resonant) = questions("resonant", true).unwrap();
        assert_eq!(resonant.len(), 12);
        assert!(resonant.values().all(|q| q["type"] == "noul"
            && q["instructions"]
                .as_str()
                .unwrap()
                .contains("decide independently")));
        let (_, right) = questions("right_frame", true).unwrap();
        assert_eq!(
            right["right_frame"]["type"], "choice",
            "the right frame is one lens"
        );
        assert_eq!(
            right["right_frame"]["criteria"].as_object().unwrap().len(),
            12
        );
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
    fn a_full_text_reading_is_each_lens_profile() {
        let (_dir, decide) = fake_decide(json!({
            "L4'|L4.5'": {"noul": 0.81}, "L4'|L4.0'": {"noul": 0.62}, "L4'|L4.3'": {"noul": 0.5},
            "L1|L1-2": {"noul": 0.2}
        }));
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
        let lenses = r["reading"]["lenses"].as_array().unwrap();
        let t = lenses.iter().find(|c| c["lens"] == "L4'").unwrap();
        assert_eq!(t["slots"].as_array().unwrap().len(), 6);
        assert_eq!(
            t["holds"],
            json!(["mef:sublens:L4'.0@1", "mef:sublens:L4'.5@1"]),
            "two sub-slots hold at once; exactly one half does not"
        );
        let causal = lenses.iter().find(|c| c["lens"] == "L1").unwrap();
        assert_eq!(causal["slots"][2]["weight"], 0.2);
        assert_eq!(causal["holds"], json!([]));
        let unread = lenses.iter().find(|c| c["lens"] == "L0").unwrap();
        assert!(
            unread["slots"]
                .as_array()
                .unwrap()
                .iter()
                .all(|s| s["weight"].is_null()),
            "an unanswered lens stays unread, never filled"
        );
        assert_eq!(
            t["refracts"],
            json!({"at": "P1", "office": "material, evidence and givens"})
        );
        let giving = r["giving"].as_str().unwrap();
        assert!(
            giving.contains("L4' Scientific (Kuhn / knowledge work, Earth), refracting P1, material, evidence and givens"),
            "each lens says whose responsibility it reads: {giving}"
        );
        assert!(
            giving.contains("holds — L4.5' Insight 0.81: synthesis crystallization · L4.0' Questions 0.62: inquiry initiation\n"),
            "what holds carries its meaning, strongest first: {giving}"
        );
        assert!(
            giving.contains("below one half — L4.3' Patterns 0.50: structure recognition\n"),
            "the rest of the lens stays in the giving: {giving}"
        );
        assert!(
            giving.contains("L1-2 Efficient Cause 0.20: agency, force, transformation"),
            "{giving}"
        );
        assert!(
            giving.contains("  unread — L0.0 Why (./?): presuppositional frame · "),
            "an unanswered sub-slot is unread, not below one half: {giving}"
        );
    }

    #[test]
    fn a_square_reading_reads_each_position_through_its_link() {
        let (_dir, decide) =
            fake_decide(json!({"L4'@P1|L4.1'": {"noul": 0.7}, "L4@P4|L4.1": {"noul": 0.9}}));
        let r = read_with(&decide, "becoming", "s", None, None).unwrap();
        let positions = r["reading"]["positions"].as_array().unwrap();
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0]["at"], "P1");
        assert_eq!(positions[0]["link"], json!(["L1", "L4'"]));
        let night = &positions[0]["lenses"][1];
        assert_eq!(night["lens"], "L4'", "the night lens of the pair is read");
        assert_eq!(night["holds"], json!(["mef:sublens:L4'.1@1"]));
        assert_eq!(positions[1]["link"], json!(["L4", "L1'"]));
        assert_eq!(r["covers"], json!(COVERAGE_BECOMING));
        let giving = r["giving"].as_str().unwrap();
        assert!(
            giving.contains("L4' Scientific (Kuhn / knowledge work, Earth), refracting P1, material, evidence and givens\n  holds — L4.1' Traces 0.70: evidence collection"),
            "{giving}"
        );
        assert!(
            giving.contains("L1' Phenomenal (Jung's psychic functions, Water), refracting P4, "),
            "{giving}"
        );
    }

    #[test]
    fn resonant_frames_weigh_every_lens_and_rank_them() {
        let (_dir, decide) =
            fake_decide(json!({"L3": {"noul": 0.4}, "L1'": {"noul": 0.8}, "L0": {"noul": 0.6}}));
        let r = read_with(&decide, "resonant", "s", None, None).unwrap();
        let frames = r["reading"]["frames"].as_array().unwrap();
        assert_eq!(frames.len(), 12);
        let order: Vec<&str> = frames[..3]
            .iter()
            .map(|f| f["lens"].as_str().unwrap())
            .collect();
        assert_eq!(order, ["L1'", "L0", "L3"]);
        assert_eq!(r["reading"]["in_play"], json!(["L1'", "L0"]));
        assert!(frames[11]["weight"].is_null());
        let giving = r["giving"].as_str().unwrap();
        assert!(
            giving.contains(
                "L1' Phenomenal (refracting P4) 0.80 · L0 Quaternal (refracting P0) 0.60"
            ),
            "{giving}"
        );
    }

    #[test]
    fn the_supply_carries_the_relational_contract_to_the_body() {
        for tool in tools()["tools"].as_array().unwrap() {
            assert_eq!(tool["prompt_guidelines"][0], RELATIONAL_CONTRACT);
            let description = tool["description"].as_str().unwrap();
            assert!(!description.contains("Args:"), "{description}");
        }
    }
}
