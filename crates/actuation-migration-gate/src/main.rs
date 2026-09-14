//! The native migration gate entry point. `corpus` replays the frozen-corpus
//! integrity laws (the migration-oracle job); `parity` replays every frozen
//! parity and scenario corpus against the native wire oracles and writes the
//! same evidence files the retired JavaScript executors captured.

use actuation_migration_gate::{write_evidence, Gate};
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("help");
    let gate = Gate::from_exe();
    let exit = match command {
        "corpus" => corpus(&gate),
        "parity" => parity(&gate, &args[1..]),
        other => {
            eprintln!(
                "usage: actuation-migration-gate corpus | parity [--evidence DIR] [--only NAME]...\nunknown command {other}"
            );
            2
        }
    };
    std::process::exit(exit);
}

fn corpus(gate: &Gate) -> i32 {
    let checks = [
        ("corpus", gate.corpus_integrity()),
        ("runtime", gate.runtime_corpus_integrity()),
        ("stream", gate.stream_corpus_integrity()),
        ("adapter", gate.adapter_corpus_integrity()),
    ];
    let mut aggregate = serde_json::Map::new();
    for (name, result) in checks {
        match result {
            Ok(value) => {
                aggregate.insert(name.into(), value);
            }
            Err(error) => {
                eprintln!("frozen corpus integrity failed ({name}): {error}");
                return 1;
            }
        }
    }
    let mut summary = serde_json::Map::new();
    for (_, value) in aggregate {
        for (key, item) in value.as_object().expect("object summaries") {
            if key != "schema" && key != "gate" {
                summary.insert(key.clone(), item.clone());
            }
        }
    }
    summary.insert(
        "schema".into(),
        serde_json::json!("actuation.corpus-integrity/v1"),
    );
    summary.insert(
        "gate".into(),
        serde_json::json!("native-rust (R11; supersedes the JavaScript executors)"),
    );
    println!("{}", serde_json::Value::Object(summary));
    0
}

fn parity(gate: &Gate, args: &[String]) -> i32 {
    let mut evidence = PathBuf::from("native-evidence");
    let mut only: Vec<String> = vec![];
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--evidence" => {
                evidence = PathBuf::from(iter.next().expect("--evidence requires a value"))
            }
            "--only" => only.push(iter.next().expect("--only requires a value").clone()),
            other => {
                eprintln!("unknown parity option {other}");
                return 2;
            }
        }
    }
    if let Err(e) = std::fs::create_dir_all(&evidence) {
        eprintln!("evidence directory must create: {e}");
        return 1;
    }
    let examples = actuation_migration_gate::oracle_dir();
    let oracle = |name: &str| {
        let path: &Path = &examples.join(name);
        if !path.exists() {
            eprintln!(
                "native wire oracle {name} is missing at {}; run `cargo build --locked --examples --bins` first",
                path.display()
            );
            std::process::exit(2);
        }
        path.to_path_buf()
    };
    std::env::set_current_dir(&gate.root).expect("gate must run from the repository root");
    let run = gate_run(gate, &evidence, &only, &examples, oracle);
    match run {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}

fn gate_run(
    gate: &Gate,
    evidence: &Path,
    only: &[String],
    examples: &Path,
    oracle: impl Fn(&str) -> PathBuf,
) -> Result<(), String> {
    let wanted = |name: &str| only.is_empty() || only.iter().any(|n| n == name);
    // Pure parity phases against the frozen oracle corpus.
    let phases: &[(&str, &str)] = &[
        ("R2", "constitutional-oracle"),
        ("R3", "runtime-oracle"),
        ("R4", "stream-oracle"),
        ("R5", "adapter-oracle"),
        ("R6", "research-oracle"),
    ];
    for (phase, example) in phases {
        if wanted(phase) {
            let command = oracle(example);
            let summary = gate.parity_phase(phase, &[command.as_path()])?;
            write_evidence(&evidence.join(format!("{phase}-parity.json")), &summary)?;
            println!("{summary}");
        }
    }
    // Effect/store scenario families through the store and observation oracles.
    if wanted("R4-scenario") {
        let command = oracle("store-oracle");
        let summary = gate.scenario_parity(
            "fixtures/migration/scenarios.json",
            &["store", "fold", "filename"],
            &[command.as_path()],
        )?;
        write_evidence(&evidence.join("R4-scenario-parity.json"), &summary)?;
        println!("{summary}");
    }
    if wanted("R5-scenario") {
        let command = oracle("observation-oracle");
        let summary = gate.scenario_parity(
            "fixtures/migration/scenarios.json",
            &["catalog", "probe", "secret"],
            &[command.as_path()],
        )?;
        write_evidence(&evidence.join("R5-scenario-parity.json"), &summary)?;
        println!("{summary}");
    }
    // The R7 line: the served CLI itself answers the frozen command-surface
    // scenarios (through the cli scenario transport, as before).
    if wanted("R7-cli") {
        let command = oracle("cli-scenario-oracle");
        let summary = gate.scenario_parity(
            "fixtures/migration/scenarios.json",
            &["cli"],
            &[command.as_path()],
        )?;
        write_evidence(&evidence.join("R7-cli-scenario-parity.json"), &summary)?;
        println!("{summary}");
    }
    // Frozen per-corpus integrity for the runtime, stream and adapter corpora.
    if wanted("R3-corpus") {
        let summary = gate.runtime_corpus_integrity()?;
        write_evidence(&evidence.join("R3-corpus-integrity.json"), &summary)?;
        println!("{summary}");
    }
    if wanted("R4-corpus") {
        let summary = gate.stream_corpus_integrity()?;
        write_evidence(&evidence.join("R4-corpus-integrity.json"), &summary)?;
        println!("{summary}");
    }
    if wanted("R5-corpus") {
        let summary = gate.adapter_corpus_integrity()?;
        write_evidence(&evidence.join("R5-corpus-integrity.json"), &summary)?;
        println!("{summary}");
    }
    let _ = examples;
    Ok(())
}
