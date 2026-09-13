//! Native replay of the frozen migration corpora. Since the JavaScript
//! retirement (R11) this crate is the migration gate: it consumes the same
//! frozen Node-written corpora byte-for-byte and asserts the same drift laws
//! the retired `scripts/migration/*.mjs` executors asserted, against the same
//! native wire oracles (`target/debug/examples/*-oracle`). The corpora are
//! evidence and are never recaptured by a passing gate — every frozen file is
//! sha256-pinned and this gate only reads.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub const BASE_REVISION: &str = "1c862c6bf58478adf6842a090214906dd2337001";

pub struct Gate {
    pub root: PathBuf,
}

impl Gate {
    /// The repository root: the gate binary lives at `<root>/target/debug/`.
    pub fn from_exe() -> Self {
        let exe = std::env::current_exe().expect("gate executable path");
        // Support both `target/debug/actuation-migration-gate` and
        // `target/debug/deps/...` layouts.
        for ancestor in exe.ancestors().skip(1) {
            if ancestor
                .file_name()
                .map(|n| n == "debug" || n == "release")
                .unwrap_or(false)
            {
                let root = ancestor
                    .parent()
                    .and_then(Path::parent)
                    .map(Path::to_path_buf);
                if let Some(root) = root {
                    return Self { root };
                }
            }
        }
        panic!("gate must run from the workspace target directory; cannot locate repository root from {:?}", exe);
    }

    pub fn from_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn read(&self, rel: &str) -> Result<String, String> {
        std::fs::read_to_string(self.root.join(rel)).map_err(|e| format!("{rel} must read: {e}"))
    }

    fn parse(&self, rel: &str) -> Result<Value, String> {
        let text = self.read(rel)?;
        serde_json::from_str(&text).map_err(|e| format!("{rel} must parse: {e}"))
    }

    fn sha256(&self, rel: &str) -> Result<String, String> {
        let bytes =
            std::fs::read(self.root.join(rel)).map_err(|e| format!("{rel} must read: {e}"))?;
        Ok(format!("{:x}", Sha256::digest(&bytes)))
    }

    /// `expected  path` lines, whitespace separated.
    fn sha256sums(&self, rel: &str) -> Result<Vec<(String, String)>, String> {
        let text = self.read(rel)?;
        let mut out = vec![];
        for line in text.trim().split('\n') {
            let mut parts = line.split_whitespace();
            match (parts.next(), parts.next(), parts.next()) {
                (Some(digest), Some(path), None) => out.push((digest.to_owned(), path.to_owned())),
                _ => return Err(format!("{rel} must carry digest/path pairs")),
            }
        }
        Ok(out)
    }

    fn assert_frozen(&self, sums_rel: &str) -> Result<(), String> {
        for (expected, path) in self.sha256sums(sums_rel)? {
            let actual = self.sha256(&path)?;
            if actual != expected {
                return Err(format!("frozen file changed: {path}"));
            }
        }
        Ok(())
    }
}

/// Structural equality with the semantics of Node's `deepStrictEqual` over
/// parsed JSON: object key order is not semantic, array order is, numbers are
/// doubles (`1` and `1.0` parse equal; a `-0` cannot survive JSON).
pub fn json_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Number(x), Value::Number(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(fx), Some(fy)) => fx == fy,
            _ => x == y,
        },
        (Value::String(x), Value::String(y)) => x == y,
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(a, b)| json_eq(a, b))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|v2| json_eq(v, v2)))
        }
        _ => false,
    }
}

fn full_lower_hex(value: &Value, what: &str) -> Result<(), String> {
    let s = value
        .as_str()
        .filter(|s| {
            s.len() == 40
                && s.bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        })
        .ok_or_else(|| format!("{what} must be a full lowercase git revision"))?;
    let _ = s;
    Ok(())
}

// ---------------------------------------------------------------------------
// Frozen corpus integrity — ports of verify-corpus.mjs, verify-runtime-corpus.mjs,
// verify-stream-corpus.mjs, verify-adapter-corpus.mjs
// ---------------------------------------------------------------------------

impl Gate {
    /// Port of `verify-corpus.mjs` (the migration-oracle job) — plus the
    /// recorder law: the retired JS capture tooling is gone, so "never
    /// recapture in a passing gate" is enforced structurally: every frozen
    /// corpus is byte-pinned and this gate is read-only over it.
    pub fn corpus_integrity(&self) -> Result<Value, String> {
        let oracle = self.parse("fixtures/migration/oracle.json")?;
        let scenarios = self.parse("fixtures/migration/scenarios.json")?;
        let ledger = self.parse("docs/rust-refoundation/ledger.json")?;
        if oracle["source_revision"] != json!(BASE_REVISION) {
            return Err("oracle source_revision drifted from the locked base revision".into());
        }
        if scenarios["source_revision"] != json!(BASE_REVISION) {
            return Err("scenario source_revision drifted from the locked base revision".into());
        }
        if ledger["source_revision"] != json!(BASE_REVISION) {
            return Err("ledger source_revision drifted from the locked base revision".into());
        }
        let cases = oracle["cases"]
            .as_array()
            .ok_or("oracle cases must be an array")?;
        if cases.len() < 500 {
            return Err("missing substantive public oracle".into());
        }
        if oracle["coverage"].as_object().map(|c| c.len()).unwrap_or(0) < 50 {
            return Err("public operation coverage shrank".into());
        }
        let scenario_cases = scenarios["cases"]
            .as_array()
            .ok_or("scenario cases must be an array")?;
        if scenario_cases.len() < 60 {
            return Err("missing effect/store/CLI oracle".into());
        }
        let entries = ledger["entries"]
            .as_array()
            .ok_or("ledger entries must be an array")?;
        if entries.len() < 200 {
            return Err("missing exact whole-tree ledger".into());
        }
        for (corpus, cases) in [("oracle", cases), ("scenarios", scenario_cases)] {
            let ids: std::collections::BTreeSet<&str> = cases
                .iter()
                .map(|row| row["id"].as_str().ok_or("case id must be text"))
                .collect::<Result<_, _>>()?;
            if ids.len() != cases.len() {
                return Err(format!("duplicate case identity in {corpus}"));
            }
        }
        // The frozen families are historical identities of the recorded
        // oracle operations (the named JavaScript sources are retired; the
        // operations live on as native oracle dispatch).
        for prefix in [
            "contracts/agency.mjs#",
            "contracts/agency-actualisation.mjs#",
            "contracts/realised-actuation.mjs#",
            "contracts/actuation-stream.mjs#",
            "contracts/activity.mjs#",
            "contracts/model-usage.mjs#",
            "contracts/instantiation.mjs#",
            "contracts/harness-detection.mjs#",
            "contracts/harness-capability.mjs#",
            "contracts/request-correlation.mjs#",
            "contracts/secret-detection.mjs#",
            "experiments/epistemic-cultivation/",
            "experiments/ql-runtime/prime/",
        ] {
            if !cases.iter().any(|row| {
                row["operation"]
                    .as_str()
                    .is_some_and(|o| o.starts_with(prefix))
            }) {
                return Err(format!("unrepresented family {prefix}"));
            }
        }
        for kind in [
            "catalog", "probe", "secret", "fold", "filename", "store", "cli",
        ] {
            if !scenario_cases.iter().any(|row| row["kind"] == json!(kind)) {
                return Err(format!("unrepresented scenario kind {kind}"));
            }
        }
        for row in entries {
            full_lower_hex(&row["source_blob"], "ledger source_blob")?;
            for field in ["target", "reason", "phase", "disposition"] {
                if row[field].as_str().unwrap_or("").is_empty() {
                    return Err(format!("ledger entry missing {field}"));
                }
            }
            if row["path"]
                .as_str()
                .unwrap_or("")
                .starts_with("experiments/")
                && !"ABCD".contains(row["classification"].as_str().unwrap_or(""))
            {
                return Err(format!(
                    "unclassified experiment {}",
                    row["path"].as_str().unwrap_or("")
                ));
            }
        }
        self.assert_frozen("fixtures/migration/SHA256SUMS")?;
        Ok(json!({
            "schema": "actuation.corpus-integrity/v1",
            "source_revision": BASE_REVISION,
            "pure_cases": cases.len(),
            "scenario_cases": scenario_cases.len(),
            "source_files": entries.len(),
            "evidence_class": "D",
            "status": "ok",
            "gate": "native-rust (R11; supersedes the JavaScript executors)"
        }))
    }

    /// Port of `verify-runtime-corpus.mjs` (R3).
    pub fn runtime_corpus_integrity(&self) -> Result<Value, String> {
        let rel = "fixtures/migration/runtime.json";
        let sums = self.sha256sums("fixtures/migration/R3-SHA256SUMS")?;
        match sums.as_slice() {
            [(digest, path)] if path.as_str() == rel => {
                if self.sha256(rel)? != digest.as_str() {
                    return Err("frozen runtime extraction corpus changed".into());
                }
            }
            _ => return Err("R3 checksum manifest must pin the runtime corpus".into()),
        }
        let corpus = self.parse(rel)?;
        if corpus["source_revision"] != json!("1c862c6bf58478adf6842a090214906dd2337001") {
            return Err("runtime corpus source_revision drifted".into());
        }
        if corpus["schema"] != json!("actuation.runtime-extraction/v1") {
            return Err("runtime corpus schema drifted".into());
        }
        let cases = corpus["cases"].as_array().ok_or("cases array")?;
        if cases.len() != 29 {
            return Err(format!(
                "runtime corpus must hold 29 cases, has {}",
                cases.len()
            ));
        }
        let ids: std::collections::BTreeSet<_> = cases
            .iter()
            .map(|c| c["id"].as_str().ok_or("case id must be text"))
            .collect::<Result<_, _>>()?;
        if ids.len() != 29 {
            return Err("runtime corpus duplicate case identity".into());
        }
        Ok(json!({
            "schema": "actuation.runtime-corpus-integrity/v1",
            "cases": 29, "status": "ok", "evidence_class": "D"
        }))
    }

    /// Port of `verify-stream-corpus.mjs` (R4).
    pub fn stream_corpus_integrity(&self) -> Result<Value, String> {
        let manifest = self.parse("fixtures/migration/r4/manifest.json")?;
        let scenarios = self.parse("fixtures/migration/scenarios.json")?;
        if manifest["schema"] != json!("actuation.r4-frozen-stores/v1") {
            return Err("r4 manifest schema drifted".into());
        }
        if manifest["source_revision"] != scenarios["source_revision"] {
            return Err("r4 manifest source_revision drifted from the scenario corpus".into());
        }
        let stores_dir = self.root.join("fixtures/migration/r4/stores");
        let mut on_disk: Vec<String> = std::fs::read_dir(&stores_dir)
            .map_err(|e| format!("r4 stores directory must read: {e}"))?
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        on_disk.sort();
        let stores = manifest["stores"]
            .as_array()
            .ok_or("r4 stores must be an array")?;
        if stores.len() != 8 {
            return Err(format!(
                "r4 manifest must hold 8 stores, has {}",
                stores.len()
            ));
        }
        let mut expected: Vec<String> = stores
            .iter()
            .map(|s| {
                s["path"]
                    .as_str()
                    .map(|p| p.trim_start_matches("stores/").to_owned())
                    .ok_or("store path must be text")
            })
            .collect::<Result<_, _>>()?;
        expected.sort();
        if on_disk != expected {
            return Err("extracted r4 store set drifted from the manifest".into());
        }
        for entry in stores {
            let path = entry["path"].as_str().ok_or("store path must be text")?;
            let rel = format!("fixtures/migration/r4/{path}");
            let name = path.strip_prefix("stores/").unwrap_or("");
            if ![
                name.len() == 8,
                name.ends_with(".jsonl"),
                name[..2].bytes().all(|b| b.is_ascii_digit()),
                name.as_bytes()[2] == b'.',
            ]
            .into_iter()
            .all(|ok| ok)
            {
                return Err(format!("r4 store path shape drifted: {path}"));
            }
            let raw = self.read(&rel)?;
            let source_case = entry["source_case"]
                .as_str()
                .ok_or("source_case must be text")?;
            let source_step = &entry["source_step"];
            let original_name = entry["original_name"]
                .as_str()
                .ok_or("original_name must be text")?;
            let case = scenarios["cases"]
                .as_array()
                .ok_or("scenario cases array")?
                .iter()
                .find(|c| c["id"] == json!(source_case))
                .ok_or_else(|| format!("r4 source case {source_case} absent from scenarios"))?;
            // The recorded step indexes the expected result array (JS semantics).
            let step_value = match source_step {
                Value::Number(n) => case["expected"]
                    .as_array()
                    .and_then(|a| n.as_u64().and_then(|i| a.get(i as usize))),
                _ => case["expected"].get(source_step.as_str().unwrap_or("")),
            }
            .ok_or("r4 source_step must index a recorded expectation")?;
            let original = &step_value["files"][original_name];
            if original.as_str() != Some(raw.as_str()) {
                return Err(format!("extracted snapshot must be exact R1 bytes: {path}"));
            }
            if self.sha256(&rel)? != entry["sha256"].as_str().unwrap_or("") {
                return Err(format!("r4 snapshot digest drifted: {path}"));
            }
            let first = raw.split('\n').next().unwrap_or("");
            let first: Value = serde_json::from_str(first)
                .map_err(|e| format!("r4 first line must parse: {e}"))?;
            if first["stream_ref"] != entry["stream_ref"] {
                return Err(format!("r4 stream_ref drifted: {path}"));
            }
        }
        Ok(json!({
            "schema": "actuation.stream-corpus-integrity/v1",
            "snapshots": stores.len(), "status": "ok", "evidence_class": "D"
        }))
    }

    /// Port of `verify-adapter-corpus.mjs` (R5).
    pub fn adapter_corpus_integrity(&self) -> Result<Value, String> {
        let catalog = self.parse("catalog/targets.json")?;
        if catalog["schema"] != json!("actuation.native-catalog/v1") {
            return Err("catalog schema drifted".into());
        }
        let revision = catalog["catalog_revision"]
            .as_u64()
            .ok_or("catalog_revision")?;
        if revision < 1 {
            return Err("catalog revision must be at least 1".into());
        }
        if catalog["descriptors"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0)
            < 1
        {
            return Err("catalog must describe at least one target".into());
        }
        if catalog["capabilities"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0)
            < 1
        {
            return Err("catalog must declare at least one capability".into());
        }
        let corrections = self.parse("fixtures/migration/r5/corrections.json")?;
        if corrections["schema"] != json!("actuation.r5-explicit-corrections/v1") {
            return Err("r5 corrections schema drifted".into());
        }
        let cases = corrections["cases"].as_array().ok_or("corrections cases")?;
        if cases.len() != 3 {
            return Err("r5 corrections must hold 3 cases".into());
        }
        if corrections["source_blobs"]
            .as_object()
            .map(|b| b.len())
            .unwrap_or(0)
            != 3
        {
            return Err("r5 corrections must pin its 3 original sources".into());
        }
        for row in cases {
            if row["correction"].as_str().unwrap_or("").is_empty() {
                return Err("r5 correction rows must carry their correction".into());
            }
        }
        self.assert_frozen("fixtures/migration/r5/SHA256SUMS")?;
        Ok(json!({
            "schema": "actuation.adapter-corpus-integrity/v1",
            "catalog_revision": revision,
            "targets": catalog["descriptors"].as_array().map(|a| a.len()).unwrap_or(0),
            "capabilities": catalog["capabilities"].as_array().map(|a| a.len()).unwrap_or(0),
            "explicit_corrections": cases.len(),
            "status": "ok", "evidence_class": "D"
        }))
    }
}

// ---------------------------------------------------------------------------
// Parity replay — port of parity.mjs and scenario-parity.mjs (native oracle path)
// ---------------------------------------------------------------------------

pub const PHASES: &[(&str, &[&str])] = &[
    ("R2", &["contracts/agency.mjs"]),
    (
        "R3",
        &[
            "contracts/agency-actualisation.mjs",
            "contracts/realised-actuation.mjs",
        ],
    ),
    (
        "R4",
        &[
            "contracts/actuation-stream.mjs",
            "contracts/activity.mjs",
            "contracts/model-usage.mjs",
            "contracts/request-correlation.mjs",
        ],
    ),
    (
        "R5",
        &[
            "contracts/harness-capability.mjs",
            "contracts/harness-detection.mjs",
            "contracts/instantiation.mjs",
            "contracts/secret-detection.mjs",
        ],
    ),
    ("R6", &["experiments/"]),
];

impl Gate {
    fn oracle_cases(&self, phase: &str) -> Result<Vec<Value>, String> {
        let prefixes: &[&str] = PHASES
            .iter()
            .find(|(name, _)| *name == phase)
            .ok_or_else(|| format!("unknown phase {phase}"))?
            .1;
        let corpus = self.parse("fixtures/migration/oracle.json")?;
        if corpus["schema"] != json!("actuation.migration-oracle/v1") {
            return Err("oracle corpus schema drifted".into());
        }
        let rows: Vec<Value> = corpus["cases"]
            .as_array()
            .ok_or("oracle cases must be an array")?
            .iter()
            .filter(|row| {
                let op = row["operation"].as_str().unwrap_or("");
                prefixes.iter().any(|p| op.starts_with(p))
            })
            .cloned()
            .collect();
        if rows.is_empty() {
            return Err(format!("zero {phase} cases is not conformance"));
        }
        Ok(rows)
    }

    fn feed_oracle(command: &[&Path], rows: &[Value]) -> Result<Vec<Value>, String> {
        let mut child = Command::new(command[0])
            .args(&command[1..])
            .current_dir(
                std::env::var("ACTUATION_GATE_ROOT")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from(".")),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("native oracle process failed to start: {e}"))?;
        // Feed on a side thread: the oracle answers while it reads, so its
        // stdout pipe must be drained while we write or both sides deadlock.
        let mut stdin = child.stdin.take().ok_or("oracle stdin unavailable")?;
        let owned_rows = rows.to_vec();
        let feeder = std::thread::spawn(move || {
            for row in &owned_rows {
                if serde_json::to_writer(
                    &mut stdin,
                    &json!({"id": row["id"], "operation": row["operation"], "args": row["args"]}),
                )
                .is_err()
                {
                    return;
                }
                if stdin.write_all(b"\n").is_err() {
                    return;
                }
            }
        });
        let output = child
            .wait_with_output()
            .map_err(|e| format!("native oracle process failed: {e}"))?;
        let _ = feeder.join();
        if !output.status.success() {
            return Err(format!(
                "native oracle process failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let mut results: Vec<Value> = vec![];
        for line in String::from_utf8_lossy(&output.stdout).trim().split('\n') {
            if line.is_empty() {
                continue;
            }
            results.push(
                serde_json::from_str(line).map_err(|e| format!("oracle answer must parse: {e}"))?,
            );
        }
        Ok(results)
    }

    /// Port of `parity.mjs --phase X -- <oracle>`: replay the phase's frozen
    /// cases through the native wire oracle and compare with the recorded
    /// expectations.
    pub fn parity_phase(&self, phase: &str, command: &[&Path]) -> Result<Value, String> {
        let rows = self.oracle_cases(phase)?;
        let results = Self::feed_oracle(command, &rows)?;
        if results.len() != rows.len() {
            return Err("oracle must answer every case exactly once".into());
        }
        let mut failures = vec![];
        for (row, actual) in rows.iter().zip(&results) {
            let mut fail = |error: String| {
                failures.push(json!({
                    "operation": row["operation"], "id": row["id"], "error": error,
                    "expected": row["expected"], "actual": actual,
                }))
            };
            if actual["id"] != row["id"] {
                fail("response identity/order drift".into());
                continue;
            }
            if actual["ok"] != row["expected"]["ok"] {
                fail("accept/refuse drift".into());
                continue;
            }
            if actual["ok"] == json!(true) && !json_eq(&actual["value"], &row["expected"]["value"])
            {
                fail("semantic JSON drift".into());
            }
        }
        let failed = failures.len();
        let summary = json!({
            "schema": "actuation.migration-parity/v1",
            "phase": phase, "cases": rows.len(), "failed": failed,
            "status": if failed == 0 { "ok" } else { "failed" },
            "evidence_class": "D",
        });
        if failed > 0 {
            return Err(format!(
                "{phase} parity failures:\n{}",
                serde_json::to_string_pretty(&failures).unwrap_or_default()
            ));
        }
        Ok(summary)
    }

    /// Port of `scenario-parity.mjs <path> <selection> -- <oracle>`.
    pub fn scenario_parity(
        &self,
        corpus_rel: &str,
        selection: &[&str],
        command: &[&Path],
    ) -> Result<Value, String> {
        let corpus = self.parse(corpus_rel)?;
        if corpus["schema"] != json!("actuation.migration-scenarios/v1") {
            return Err("scenario corpus schema drifted".into());
        }
        let rows: Vec<Value> = corpus["cases"]
            .as_array()
            .ok_or("scenario cases must be an array")?
            .iter()
            .filter(|row| {
                selection.is_empty() || selection.contains(&row["kind"].as_str().unwrap_or(""))
            })
            .cloned()
            .collect();
        if rows.is_empty() {
            return Err("zero selected scenarios is not conformance".into());
        }
        let mut child = Command::new(command[0])
            .args(&command[1..])
            .current_dir(&self.root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("native scenario command failed to start: {e}"))?;
        {
            let mut stdin = child.stdin.take().ok_or("scenario stdin unavailable")?;
            for row in &rows {
                serde_json::to_writer(
                    &mut stdin,
                    &json!({"id": row["id"], "kind": row["kind"], "input": row["input"]}),
                )
                .map_err(|e| format!("scenario input failed: {e}"))?;
                stdin
                    .write_all(b"\n")
                    .map_err(|e| format!("scenario input failed: {e}"))?;
            }
        }
        let output = child
            .wait_with_output()
            .map_err(|e| format!("native scenario command failed: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "native scenario command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let mut actual: Vec<Value> = vec![];
        for line in String::from_utf8_lossy(&output.stdout).trim().split('\n') {
            if line.is_empty() {
                continue;
            }
            actual.push(
                serde_json::from_str(line)
                    .map_err(|e| format!("scenario answer must parse: {e}"))?,
            );
        }
        if actual.len() != rows.len() {
            return Err("scenario results must be total and ordered".into());
        }
        let mut failed = 0;
        for (row, answer) in rows.iter().zip(&actual) {
            if answer["id"] != row["id"] || !compare_scenario(&answer["value"], &row["expected"]) {
                failed += 1;
                eprintln!("{} scenario drift", row["id"].as_str().unwrap_or("?"));
            }
        }
        let summary = json!({
            "schema": "actuation.scenario-parity/v1",
            "cases": rows.len(), "failed": failed,
            "status": if failed == 0 { "ok" } else { "failed" },
            "evidence_class": "D",
        });
        if failed > 0 {
            return Err(format!("{failed} scenarios drifted"));
        }
        Ok(summary)
    }
}

/// Port of `compareScenario`: normalise both sides, then structural equality.
pub fn compare_scenario(actual: &Value, expected: &Value) -> bool {
    json_eq(&normalise_scenario(actual), &normalise_scenario(expected))
}

fn normalise_scenario(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(normalise_scenario).collect()),
        Value::Object(map) => {
            // A refused operation reduces to its refusal; the diagnostic text
            // is not wire semantics.
            if map.get("ok").map(|v| v == &json!(false)).unwrap_or(false)
                && map.contains_key("error")
            {
                return json!({"ok": false});
            }
            let mut out = serde_json::Map::new();
            for (key, item) in map {
                if key == "stderr" {
                    if let Some(text) = item.as_str() {
                        // A code-2 CLI refusal must be recognisable on stderr;
                        // its diagnostic text is runner-local.
                        if map
                            .get("code")
                            .map(|c| c.as_i64() == Some(2))
                            .unwrap_or(false)
                        {
                            if !text.starts_with("actuation: ") {
                                return json!({"__unrecognisable_cli_refusal__": text});
                            }
                            out.insert("stderr".into(), json!("actuation: $DIAGNOSTIC\n"));
                            continue;
                        }
                    }
                }
                if key == "files" {
                    // JSON member order inside stored lines is not wire
                    // semantics; parse each stored line when it is JSON.
                    if let Some(files) = item.as_object() {
                        let parsed: serde_json::Map<String, Value> = files
                            .iter()
                            .map(|(name, raw)| {
                                let content = raw.as_str().unwrap_or_default();
                                let parsed_line: Value = content
                                    .split('\n')
                                    .map(|line| {
                                        serde_json::from_str::<Value>(line).unwrap_or(json!(line))
                                    })
                                    .collect::<Vec<_>>()
                                    .into();
                                (name.clone(), parsed_line)
                            })
                            .collect();
                        out.insert("files".into(), Value::Object(parsed));
                        continue;
                    }
                }
                out.insert(key.clone(), normalise_scenario(item));
            }
            Value::Object(out)
        }
        other => other.clone(),
    }
}

/// Where the native wire oracles live: `<target>/<profile>/examples`.
pub fn oracle_dir() -> PathBuf {
    let exe = std::env::current_exe().expect("gate executable path");
    for ancestor in exe.ancestors().skip(1) {
        if ancestor
            .file_name()
            .map(|n| n == "debug" || n == "release")
            .unwrap_or(false)
        {
            return ancestor.join("examples");
        }
    }
    panic!("cannot locate the examples directory from {:?}", exe);
}

/// Write `value` as one JSON document (evidence files are exactly what the
/// retired `tee`-based script captured).
pub fn write_evidence(path: &Path, value: &Value) -> Result<(), String> {
    let file =
        std::fs::File::create(path).map_err(|e| format!("{} must create: {e}", path.display()))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, value)
        .map_err(|e| format!("{} must serialise: {e}", path.display()))?;
    writeln!(writer).map_err(|e| format!("{} must write: {e}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_eq_matches_deep_strict_equal_over_parsed_json() {
        assert!(json_eq(&json!(1), &json!(1.0)), "JS numbers are doubles");
        assert!(json_eq(
            &json!({"a":1,"b":[2,"x"]}),
            &json!({"b":[2,"x"],"a":1})
        ));
        assert!(
            !json_eq(&json!([1, 2]), &json!([2, 1])),
            "array order is semantic"
        );
        assert!(!json_eq(&json!({"a":1}), &json!({"a":"1"})));
        assert!(
            !json_eq(&json!({"a":1}), &json!({"a":1,"b":null})),
            "present null is not absent"
        );
        assert!(json_eq(&json!(null), &json!(null)));
        assert!(!json_eq(&json!(true), &json!(1)));
    }

    #[test]
    fn scenario_comparison_reduces_refusals_and_cli_diagnostics() {
        let expected = json!({"ok": false, "error": {"name": "TypeError", "message": "anything"}});
        let actual = json!({"ok": false, "error": {"name": "RangeError", "message": "different"}});
        assert!(compare_scenario(&actual, &expected));

        let cli_expected =
            json!({"code": 2, "stdout": "", "stderr": "actuation: original diagnostic\n"});
        let cli_actual =
            json!({"code": 2, "stdout": "", "stderr": "actuation: another diagnostic\n"});
        assert!(compare_scenario(&cli_actual, &cli_expected));
        let unrecognisable = json!({"code": 2, "stdout": "", "stderr": "panic: boom"});
        assert!(!compare_scenario(&unrecognisable, &cli_expected));

        // Stored JSONL member order is not wire semantics.
        let stored_expected = json!({"files": {"s.jsonl": "{\"a\":1,\"b\":2}\n"}});
        let stored_actual = json!({"files": {"s.jsonl": "{\"b\":2,\"a\":1}\n"}});
        assert!(compare_scenario(&stored_actual, &stored_expected));
        // Non-JSON stored lines stay raw.
        let raw = json!({"files": {"note.txt": "hello\n"}});
        assert!(compare_scenario(&raw, &raw));
    }

    #[test]
    fn phase_filtering_reproduces_the_recorded_family_law() {
        let gate = Gate::from_root(env!("CARGO_MANIFEST_DIR").to_owned() + "/../..");
        for (phase, expected_cases) in [
            ("R2", 119usize),
            ("R3", 44),
            ("R4", 213),
            ("R5", 179),
            ("R6", 44),
        ] {
            let rows = gate.oracle_cases(phase).expect("phase rows");
            assert_eq!(
                rows.len(),
                expected_cases,
                "{phase} case count is part of the frozen law"
            );
        }
        assert!(gate.oracle_cases("R9").is_err());
    }
}
