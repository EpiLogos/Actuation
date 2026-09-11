//! Temporary JSONL migration transport. Product semantics live in the library.
use actuation_research::{evidence, prime, records::EpistemicRecord, Error, Result};
use serde_json::{json, Value};
use std::io::{self, BufRead};
fn invoke(v: &Value) -> Result<Value> {
    let a = &v["args"];
    let string = |n: usize| {
        a[n].as_str()
            .ok_or_else(|| Error::new("string argument required"))
    };
    match v["operation"]
        .as_str()
        .and_then(|s| s.split_once('#'))
        .map(|(_, s)| s)
    {
        Some("validateEpistemicRecord") => Ok(EpistemicRecord::read(&a[0])?.into_value()),
        Some("conditionPrompt") => Ok(json!(prime::condition_prompt(&a[0], &a[1])?)),
        Some("getPrimeCondition") => prime::get_condition(string(0)?),
        Some("extractPrimeFamily") => Ok(evidence::extract_prime_family(
            a[0].as_array()
                .ok_or_else(|| Error::new("records required"))?,
        )),
        Some("stableDigest") => Ok(json!(evidence::stable_digest(&a[0]))),
        Some("assertPrimeReturn") => evidence::prime_return(&a[0], true),
        Some("createPrimeReturn") => evidence::prime_return(&a[0], false),
        Some("validateSourceLock") => Ok(prime::SourceLock::read(&a[0])?.as_value().clone()),
        Some("classifyQlRevision") => Ok(json!(prime::SourceLock::read(&a[0])?.classify(
            string(1)?,
            a[2]["harmonicEnabled"] == true,
            a[2]["sourceDirty"] == true
        )?)),
        _ => Err(Error::new("unsupported oracle operation")),
    }
}
fn main() {
    for line in io::stdin().lock().lines() {
        let v: Value = serde_json::from_str(&line.expect("input")).expect("JSON");
        let response = match invoke(&v) {
            Ok(value) => json!({"id":v["id"],"ok":true,"value":value}),
            Err(e) => json!({"id":v["id"],"ok":false,"error":{"message":e.to_string()}}),
        };
        println!("{response}");
    }
}
