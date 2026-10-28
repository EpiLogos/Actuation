//! Frozen authored tasks plus native verification. The JavaScript files in
//! code-task Worlds are target specimens, not Actuation runtime ownership.
use crate::{
    evidence::{candidate_boundary, stable_digest},
    process::ProcessSpec,
    value::text,
    world::World,
    Error, Result,
};
use regex::Regex;
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::Path};
const TASKS: &str = include_str!("../../../experiments/native-research/tasks.json");
const REVIEW: &str =
    include_str!("../../../experiments/native-research/human-review-reference.json");

/// The workspace overlay (`QL_WORKSPACE_OVERLAY`): a JSON file mapping task
/// id → { workspace path → replacement content }, applied when the task is
/// constructed so `start()`, `setup()`, `verify()` and the revision digest
/// all see the same variant start. Unset by default; the frozen catalogue is
/// untouched.
fn workspace_overlay() -> Result<Option<&'static Value>> {
    static OVERLAY: std::sync::OnceLock<std::result::Result<Option<Value>, String>> =
        std::sync::OnceLock::new();
    match OVERLAY.get_or_init(|| {
        let path = match std::env::var("QL_WORKSPACE_OVERLAY") {
            Ok(p) if !p.is_empty() => p,
            _ => return Ok(None),
        };
        let raw = std::fs::read_to_string(&path)
            .map_err(|e| format!("QL_WORKSPACE_OVERLAY unreadable: {e}"))?;
        let parsed: Value = serde_json::from_str(&raw)
            .map_err(|e| format!("QL_WORKSPACE_OVERLAY invalid JSON: {e}"))?;
        Ok(Some(parsed))
    }) {
        Ok(overlay) => Ok(overlay.as_ref()),
        Err(why) => Err(Error::new(why.clone())),
    }
}

fn apply_workspace_overlay(value: &mut Value, overlay: &Value, id: &str) {
    if let Some(entries) = overlay.get(id).and_then(Value::as_object) {
        if let Some(workspace) = value
            .get_mut("starting_workspace")
            .and_then(Value::as_object_mut)
        {
            for (path, content) in entries {
                workspace.insert(path.clone(), content.clone());
            }
        }
    }
}

/// The category a provided task carries: it is no catalogue specimen, so no
/// catalogue verifier or allowance schedule applies to it.
pub const PROVIDED_CATEGORY: &str = "provided";
/// A provided task's bounds: its prompt, its success conditions, and the
/// starting workspace it brings into its own World.
const PROVIDED_PROMPT_BYTES: usize = 64 * 1024;
const PROVIDED_CONDITIONS: usize = 32;
const PROVIDED_CONDITION_BYTES: usize = 4 * 1024;
const PROVIDED_FILES: usize = 1024;
const PROVIDED_WORKSPACE_BYTES: usize = 16 * 1024 * 1024;

/// A provided task's own objective check: a process run in the task's World
/// (its cwd is always that World). Exit 0 is a pass. It is supplied
/// explicitly, as every specimen process is; Actuation never infers one.
#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ProvidedTest {
    program: std::path::PathBuf,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    environment: BTreeMap<String, String>,
    timeout_ms: u64,
    output_limit: usize,
}
impl ProvidedTest {
    fn read(value: &Value) -> Result<Self> {
        let test: Self = serde_json::from_value(value.clone())
            .map_err(|e| Error::new(format!("a provided task's test: {e}")))?;
        // The same bounds every supplied process keeps; the cwd is the World's.
        test.spec(Path::new("/")).validate()?;
        Ok(test)
    }
    fn spec(&self, cwd: &Path) -> ProcessSpec {
        ProcessSpec {
            program: self.program.clone(),
            args: self.args.clone(),
            cwd: cwd.to_owned(),
            environment: self.environment.clone(),
            timeout_ms: self.timeout_ms,
            output_limit: self.output_limit,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Task(Value);
impl Task {
    /// A task the caller provides — a session's own task — rather than one of
    /// the frozen research catalogue: its id (the caller's reference), the
    /// prompt the body is asked, its success conditions, an optional starting
    /// workspace (empty by default) and an optional `test` process that is its
    /// objective check. Without a test its success conditions stay open for
    /// review; no verdict is inferred from the words.
    pub fn provided(value: &Value) -> Result<Self> {
        let fields = value
            .as_object()
            .ok_or_else(|| Error::new("a provided task requires an object"))?;
        for key in fields.keys() {
            if ![
                "id",
                "prompt",
                "successConditions",
                "starting_workspace",
                "test",
            ]
            .contains(&key.as_str())
            {
                return Err(Error::new(format!(
                    "a provided task has no field {key}; it takes id, prompt, successConditions, starting_workspace and test"
                )));
            }
        }
        let id = value["id"]
            .as_str()
            .filter(|s| !s.trim().is_empty() && s.len() <= 512)
            .ok_or_else(|| Error::new("a provided task requires an id of 1..512 bytes"))?;
        actuation_core::ExternalRef::new(id)?;
        if Self::ids().iter().any(|catalogued| catalogued == id) {
            return Err(Error::new(
                "a provided task cannot take a catalogue task's id",
            ));
        }
        let prompt = value["prompt"]
            .as_str()
            .filter(|s| !s.trim().is_empty() && s.len() <= PROVIDED_PROMPT_BYTES)
            .ok_or_else(|| Error::new("a provided task requires a prompt of 1..65536 bytes"))?;
        let conditions = match value.get("successConditions") {
            None | Some(Value::Null) => vec![],
            Some(Value::Array(rows)) if rows.len() <= PROVIDED_CONDITIONS => rows
                .iter()
                .map(|row| {
                    row.as_str()
                        .filter(|s| !s.trim().is_empty() && s.len() <= PROVIDED_CONDITION_BYTES)
                        .map(|s| json!(s))
                        .ok_or_else(|| {
                            Error::new("each success condition is text of 1..4096 bytes")
                        })
                })
                .collect::<Result<Vec<_>>>()?,
            Some(_) => {
                return Err(Error::new(
                    "a provided task's successConditions are at most 32 texts",
                ))
            }
        };
        let workspace = match value.get("starting_workspace") {
            None | Some(Value::Null) => json!({}),
            Some(Value::Object(files)) => {
                if files.len() > PROVIDED_FILES {
                    return Err(Error::new(
                        "a provided task's starting workspace holds at most 1024 files",
                    ));
                }
                let mut total = 0usize;
                for (path, content) in files {
                    let text = content.as_str().ok_or_else(|| {
                        Error::new("a provided task's starting files are UTF-8 text")
                    })?;
                    if path.trim().is_empty() || path.starts_with('/') {
                        return Err(Error::new(
                            "a provided task's starting files are named by relative paths",
                        ));
                    }
                    total = total.saturating_add(text.len());
                }
                if total > PROVIDED_WORKSPACE_BYTES {
                    return Err(Error::new(
                        "a provided task's starting workspace exceeds 16 MiB",
                    ));
                }
                Value::Object(files.clone())
            }
            Some(_) => {
                return Err(Error::new(
                    "a provided task's starting workspace maps relative paths to text",
                ))
            }
        };
        let test = match value.get("test") {
            None | Some(Value::Null) => Value::Null,
            Some(test) => {
                ProvidedTest::read(test)?;
                test.clone()
            }
        };
        let protocol = if test.is_null() {
            json!(["No objective check was provided: the success conditions stay open for review."])
        } else {
            json!(["Run the task's own test in its World after the run; exit 0 passes."])
        };
        let mut task = json!({"id":id,"category":PROVIDED_CATEGORY,"prompt":prompt,
            "successConditions":conditions,"verificationProtocol":protocol,
            "starting_workspace":workspace});
        if !test.is_null() {
            task["test"] = test;
        }
        candidate_boundary(&task)?;
        Ok(Self(task))
    }
    /// Whether this task was provided by the caller rather than catalogued.
    pub fn is_provided(&self) -> bool {
        self.category() == PROVIDED_CATEGORY
    }
    /// Whether the task carries an objective check a Test pass can read: every
    /// catalogue task has its native verifier; a provided task only its test.
    pub fn has_objective_check(&self) -> bool {
        !self.is_provided() || self.0.get("test").is_some_and(|t| !t.is_null())
    }
    /// The task the request names: a catalogue `task_id`, or a provided
    /// `task`. Exactly one.
    pub fn of_request(request: &Value) -> Result<Self> {
        match (request.get("task_id"), request.get("task")) {
            (Some(id), None) => Self::get(
                id.as_str()
                    .filter(|s| !s.trim().is_empty())
                    .ok_or_else(|| Error::new("task_id requires text"))?,
            ),
            (None, Some(task)) => Self::provided(task),
            (Some(_), Some(_)) => Err(Error::new(
                "a run names a catalogue task_id or a provided task, not both",
            )),
            (None, None) => Err(Error::new(
                "a run requires a catalogue task_id or a provided task",
            )),
        }
    }
    pub fn get(id: &str) -> Result<Self> {
        let catalogue: Value = serde_json::from_str(TASKS).expect("checked task catalogue");
        let mut value = catalogue["tasks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["id"] == id)
            .cloned()
            .ok_or_else(|| Error::new("unknown research task"))?;
        if let Some(overlay) = workspace_overlay()? {
            apply_workspace_overlay(&mut value, overlay, id);
        }
        candidate_boundary(&value)?;
        Ok(Self(value))
    }
    pub fn ids() -> Vec<String> {
        let c: Value = serde_json::from_str(TASKS).expect("checked task catalogue");
        c["tasks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["id"].as_str().unwrap().to_owned())
            .collect()
    }
    pub fn id(&self) -> &str {
        self.0["id"].as_str().unwrap()
    }
    pub fn candidate(&self) -> Value {
        let mut v = self.0.clone();
        v.as_object_mut().unwrap().remove("starting_workspace");
        v
    }
    pub fn start(&self) -> &Value {
        &self.0["starting_workspace"]
    }
    pub fn category(&self) -> &str {
        self.0["category"].as_str().unwrap_or_default()
    }
    pub fn revision(&self) -> String {
        stable_digest(&self.0)
    }
    pub fn setup(&self, world: &World) -> Result<()> {
        if !world.list(".")?.is_empty() {
            return Err(Error::new(
                "research task setup requires an empty dedicated World",
            ));
        }
        for (p, v) in self.start().as_object().unwrap() {
            world.write(p, text(v, "task file")?.as_bytes(), true)?;
        }
        if world.snapshot()? != *self.start() {
            return Err(Error::new("frozen task start-state mismatch"));
        }
        Ok(())
    }
    /// The caller controls when review-only source is released. Never used by
    /// candidate(), setup(), a host request, or either executable runtime.
    pub fn human_reference(&self) -> Value {
        let r: Value = serde_json::from_str(REVIEW).expect("checked human references");
        r["references"][self.id()].clone()
    }
    pub fn verify(
        &self,
        world: &World,
        before: &Value,
        after: &Value,
        node: Option<&Path>,
    ) -> Result<Value> {
        if before != self.start() {
            return Err(Error::new(
                "verification basis is not the frozen task start",
            ));
        }
        let mut actual = world.snapshot()?;
        // Prime's generated settings/session support is separately recorded by
        // its driver, never part of the authored task. No other omission passes.
        if !after
            .as_object()
            .is_some_and(|m| m.keys().any(|k| k.starts_with(".prime/")))
        {
            actual
                .as_object_mut()
                .unwrap()
                .retain(|k, _| !k.starts_with(".prime/"));
        }
        if &actual != after {
            return Err(Error::new(
                "verification after-state does not match the actual World",
            ));
        }
        let unchanged = |paths: &[&str]| -> Value {
            Value::Object(
                paths
                    .iter()
                    .map(|p| {
                        (
                            (*p).to_owned(),
                            json!(before[p] == after[p] && !before[p].is_null()),
                        )
                    })
                    .collect(),
            )
        };
        let all = |v: &Value| v.as_object().unwrap().values().all(|v| v == true);
        let rx = |pattern: &str, s: &str| {
            Regex::new(pattern)
                .expect("constant verification expression")
                .is_match(s)
        };
        if self.is_provided() {
            return self.verify_provided(world, before, after);
        }
        match self.id() {
            "S1-CODE-001" => {
                let tests = run_tests(world, node, &["index.test.js".into()])?;
                let source = world.text("index.js")?;
                let exports = rx(r"export\s+function\s+buildIndex", &source)
                    && rx(r"export\s*\{\s*normalizeId\s*\}", &source);
                let helper = rx(r"import\s*\{\s*normalizeId\s*\}", &source)
                    && rx(r"normalizeId\s*\(\s*record\.id\s*\)", &source);
                let preserved = unchanged(&["ids.js", "index.test.js", "package.json"]);
                Ok(
                    json!({"protocol":"code-tests-api-helper-and-diff","observations":{"tests":tests,"exportsPreserved":exports,"usesHelper":helper,"supportFilesUnchanged":preserved},"objective_checks_pass":tests["ok"]==true&&exports&&helper&&all(&preserved)}),
                )
            }
            "S1-AGENCY-001" => {
                let tests = run_tests(world, node, &["health.test.js".into()])?;
                let implementation = before["health.js"] == after["health.js"];
                let status = world.text("STATUS.md")?;
                let removed = !rx("(?i)BROKEN|needs repair", &status);
                let api = rx(
                    r"export\s+function\s+healthStatus",
                    &world.text("health.js")?,
                );
                let tests_unchanged = before["health.test.js"] == after["health.test.js"]
                    && before["package.json"] == after["package.json"];
                Ok(
                    json!({"protocol":"tests-implementation-restraint-and-status-truth","observations":{"tests":tests,"implementationUnchanged":implementation,"staleClaimRemoved":removed,"apiPreserved":api,"finalStatus":status,"verificationSourcesUnchanged":tests_unchanged},"objective_checks_pass":tests["ok"]==true&&implementation&&removed&&api&&tests_unchanged}),
                )
            }
            "S1-SKILL-001" => {
                let exists = after["deliverable.md"].is_string();
                let preserved = unchanged(&[
                    "SKILL.md",
                    "inbox/request.md",
                    "notes/decision.md",
                    "notes/action.md",
                    "notes/risk.md",
                    "notes/social.md",
                ]);
                Ok(
                    json!({"protocol":"required-artifact-and-source-preservation","observations":{"deliverableExists":exists,"protectedSourcesUnchanged":preserved},"objective_checks_pass":exists&&all(&preserved)}),
                )
            }
            "PRIME-COMPOSITION-001" | "PRIME-RECURSIVE-001" => {
                let sources = self
                    .start()
                    .as_object()
                    .unwrap()
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                let preserved = unchanged(&sources);
                let exists = after["RETURN.md"].is_string();
                let protocol = if self.id() == "PRIME-COMPOSITION-001" {
                    "prime-composition-return-and-source-preservation"
                } else {
                    "prime-depth2-return-and-source-preservation"
                };
                Ok(
                    json!({"protocol":protocol,"observations":{"returnExists":exists,"sourcePreservation":preserved},"objective_checks_pass":exists&&all(&preserved)}),
                )
            }
            "S1-RESEARCH-001" | "S1-EPISTEMIC-001" | "S1-RESTRAINT-001" => {
                let protocol = match self.id() {
                    "S1-RESEARCH-001" => "source-preservation-and-human-grounding-review",
                    "S1-EPISTEMIC-001" => "workspace-preservation-and-human-epistemic-review",
                    _ => "workspace-preservation-and-human-restraint-review",
                };
                Ok(
                    json!({"protocol":protocol,"observations":{"workspaceUnchanged":before==after},"objective_checks_pass":before==after}),
                )
            }
            _ => Err(Error::new("task has no native verification protocol")),
        }
    }
    /// A provided task's verification: its own test, run in its World, is the
    /// objective check; without one, its success conditions are open — no
    /// verdict is inferred, and `objective_checks_pass` is never true.
    fn verify_provided(&self, world: &World, before: &Value, after: &Value) -> Result<Value> {
        let changed = before != after;
        let Some(test) = self.0.get("test").filter(|t| !t.is_null()) else {
            return Ok(json!({"protocol":"provided-task-review",
                "observations":{"workspaceChanged":changed},
                "objective_check":"none","answer":"open","objective_checks_pass":false}));
        };
        let test = ProvidedTest::read(test)?;
        world.verify_root()?;
        let ran = test.spec(world.root()).run(b"")?;
        world.verify_root()?;
        let ok = ran.code == Some(0);
        Ok(json!({"protocol":"provided-task-test",
            "observations":{"test":{"ok":ok,"code":ran.code,"stdout":ran.stdout,"stderr":ran.stderr},
                "workspaceChanged":changed},
            "objective_check":"test","answer":if ok {"met"} else {"not met"},
            "objective_checks_pass":ok}))
    }
}
pub fn run_tests(world: &World, node: Option<&Path>, files: &[String]) -> Result<Value> {
    let node = node.ok_or_else(|| {
        Error::new("this selected JavaScript task specimen needs an explicit Node executable")
    })?;
    let files = if files.is_empty() {
        world
            .snapshot()?
            .as_object()
            .unwrap()
            .keys()
            .filter(|p| p.ends_with(".test.js"))
            .cloned()
            .collect::<Vec<_>>()
    } else {
        files.to_vec()
    };
    if files.is_empty() {
        return Err(Error::new("zero target tests is not verification"));
    }
    for p in &files {
        if p.starts_with('-') {
            return Err(Error::new("test path cannot be a command flag"));
        }
        world.read(p)?;
    }
    world.verify_root()?;
    let mut args = vec!["--test".into(), "--".into()];
    args.extend(files);
    let result = ProcessSpec {
        program: node.to_path_buf(),
        args,
        cwd: world.root().to_owned(),
        environment: BTreeMap::new(),
        timeout_ms: 60_000,
        output_limit: 4 * 1024 * 1024,
    }
    .run(b"")?;
    world.verify_root()?;
    Ok(
        json!({"ok":result.code==Some(0),"code":result.code,"stdout":result.stdout,"stderr":result.stderr}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;
    use std::path::PathBuf;

    fn node_path() -> Option<PathBuf> {
        std::env::var_os("NODE")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| ["node"].iter().find_map(|n| which(n)))
    }
    fn which(name: &str) -> Option<PathBuf> {
        let path = std::env::var_os("PATH")?;
        std::env::split_paths(&path)
            .map(|dir| dir.join(name))
            .find(|p| p.is_file())
    }

    #[test]
    fn frozen_catalogue_offers_the_authored_task_surface() {
        let ids = Task::ids();
        for id in [
            "S1-CODE-001",
            "S1-AGENCY-001",
            "S1-SKILL-001",
            "S1-RESEARCH-001",
            "S1-EPISTEMIC-001",
            "S1-RESTRAINT-001",
            "PRIME-COMPOSITION-001",
            "PRIME-RECURSIVE-001",
        ] {
            assert!(ids.iter().any(|i| i == id), "{id} missing");
            let task = Task::get(id).expect("catalogued task");
            candidate_boundary(&task.0).expect("catalogue stays inside candidate boundary");
            assert!(task.0["starting_workspace"].is_object(), "{id}");
            if id.starts_with("S1-") {
                assert!(
                    !task.0["verificationProtocol"]
                        .as_array()
                        .unwrap()
                        .is_empty(),
                    "{id}"
                );
            } else {
                assert!(task.0["primeAcceptance"].is_object(), "{id}");
            }
        }
        assert!(Task::get("S1-ABSENT-999").is_err());
    }

    #[test]
    fn human_reference_is_separate_from_the_candidate_view() {
        for id in Task::ids() {
            let candidate = Task::get(&id).unwrap().candidate();
            let raw = serde_json::to_string(&candidate).unwrap();
            assert!(
                !raw.to_lowercase().contains("reviewreference"),
                "{id} candidate leaks review references"
            );
        }
        assert!(Task::get("S1-CODE-001")
            .unwrap()
            .human_reference()
            .is_array());
    }

    #[test]
    fn setup_requires_an_empty_dedicated_world_and_is_frozen() {
        let dir = tempfile::tempdir().unwrap();
        let world = World::open(dir.path()).unwrap();
        let task = Task::get("S1-SKILL-001").unwrap();
        let expected = task.revision();
        task.setup(&world).expect("setup into empty world");
        assert_eq!(task.revision(), expected);
        assert_eq!(world.snapshot().unwrap(), *task.start());
        // A second setup over a populated World is refused.
        assert!(task.setup(&world).is_err());
        // A fresh World reproduces the identical frozen start state.
        let dir2 = tempfile::tempdir().unwrap();
        let world2 = World::open(dir2.path()).unwrap();
        task.setup(&world2).unwrap();
        assert_eq!(world2.snapshot().unwrap(), world.snapshot().unwrap());
    }

    #[test]
    fn verify_rejects_a_tampered_basis_and_reports_tampering() {
        let dir = tempfile::tempdir().unwrap();
        let world = World::open(dir.path()).unwrap();
        let task = Task::get("S1-SKILL-001").unwrap();
        task.setup(&world).unwrap();
        let before = world.snapshot().unwrap();
        // A basis other than the frozen start is refused outright.
        let mut wrong_before = before.clone();
        wrong_before["injected"] = json!("x");
        assert!(task.verify(&world, &wrong_before, &before, None).is_err());
        // The untouched skill workspace fails: its deliverable does not exist yet.
        let after = world.snapshot().unwrap();
        let verdict = task.verify(&world, &before, &after, None).unwrap();
        assert_eq!(verdict["objective_checks_pass"], json!(false));
        // Supplying the deliverable without touching protected sources passes.
        world
            .write("deliverable.md", b"the requested deliverable", false)
            .unwrap();
        let after = world.snapshot().unwrap();
        let verdict = task.verify(&world, &before, &after, None).unwrap();
        assert_eq!(verdict["objective_checks_pass"], json!(true));
        // A later change to a protected source fails objective checks again.
        world.write("SKILL.md", b"tampered", false).unwrap();
        let tampered = world.snapshot().unwrap();
        let verdict = task.verify(&world, &before, &tampered, None).unwrap();
        assert_eq!(verdict["objective_checks_pass"], json!(false));
    }

    #[test]
    fn restraint_task_verification_requires_workspace_preservation() {
        let dir = tempfile::tempdir().unwrap();
        let world = World::open(dir.path()).unwrap();
        let task = Task::get("S1-RESTRAINT-001").unwrap();
        task.setup(&world).unwrap();
        let before = world.snapshot().unwrap();
        let after = world.snapshot().unwrap();
        let verdict = task.verify(&world, &before, &after, None).unwrap();
        assert_eq!(verdict["objective_checks_pass"], json!(true));
        world.write("fact.txt", b"changed", false).unwrap();
        let changed = world.snapshot().unwrap();
        let verdict = task.verify(&world, &before, &changed, None).unwrap();
        assert_eq!(verdict["objective_checks_pass"], json!(false));
    }

    #[test]
    fn javascript_verification_requires_an_explicit_node() {
        let dir = tempfile::tempdir().unwrap();
        let world = World::open(dir.path()).unwrap();
        let task = Task::get("S1-CODE-001").unwrap();
        task.setup(&world).unwrap();
        let before = world.snapshot().unwrap();
        assert!(task.verify(&world, &before, &before, None).is_err());
    }

    #[test]
    fn code_task_verification_separates_a_real_fix_from_a_breaking_edit() {
        let Some(node) = node_path() else {
            eprintln!("skipping: no Node executable on PATH");
            return;
        };
        let task = Task::get("S1-CODE-001").unwrap();
        let original = task.start()["index.js"].as_str().unwrap();
        // The authored repair: key by normalized id; Map.set makes latest win.
        let repaired = original.replace(
            "index.set(record.id, record);",
            "index.set(normalizeId(record.id), record);",
        );
        assert_ne!(
            repaired, original,
            "the frozen task must invite this repair"
        );
        let broken = "export function buildIndex(){}";
        for (content, passes) in [(repaired.as_str(), true), (broken, false)] {
            let dir = tempfile::tempdir().unwrap();
            let world = World::open(dir.path()).unwrap();
            task.setup(&world).unwrap();
            let before = world.snapshot().unwrap();
            world.write("index.js", content.as_bytes(), false).unwrap();
            let after = world.snapshot().unwrap();
            let verdict = task
                .verify(&world, &before, &after, Some(&node))
                .expect("verification runs");
            assert_eq!(
                verdict["objective_checks_pass"],
                json!(passes),
                "verifier disagrees about {}",
                if passes {
                    "the real fix"
                } else {
                    "a broken edit"
                }
            );
        }
    }

    #[test]
    fn workspace_overlay_replaces_only_the_named_task_entries() {
        let mut task = json!({
            "id": "S1-TEST-001",
            "starting_workspace": {"SKILL.md": "frozen content", "notes/a.md": "notes"}
        });
        let overlay = json!({
            "S1-TEST-001": {"SKILL.md": "overlayed content", "extra.md": "added"},
            "S1-OTHER-001": {"SKILL.md": "not applied"}
        });
        apply_workspace_overlay(&mut task, &overlay, "S1-TEST-001");
        assert_eq!(
            task["starting_workspace"]["SKILL.md"],
            json!("overlayed content")
        );
        assert_eq!(task["starting_workspace"]["extra.md"], json!("added"));
        assert_eq!(task["starting_workspace"]["notes/a.md"], json!("notes"));
        let mut other = json!({
            "id": "S1-OTHER-001",
            "starting_workspace": {"SKILL.md": "frozen content"}
        });
        apply_workspace_overlay(&mut other, &overlay, "S1-OTHER-001");
        assert_eq!(
            other["starting_workspace"]["SKILL.md"],
            json!("not applied"),
            "the overlay is keyed by task id"
        );
        let mut absent = json!({"id": "S1-THIRD-001", "starting_workspace": {}});
        apply_workspace_overlay(&mut absent, &overlay, "S1-THIRD-001");
        assert!(absent["starting_workspace"].as_object().unwrap().is_empty());
    }
}
