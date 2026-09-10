#[path = "../tests/support/mod.rs"]
mod support;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = io::BufWriter::new(io::stdout().lock());
    for line in io::stdin().lock().lines() {
        let request: Value = serde_json::from_str(&line?)?;
        let operation = request["operation"].as_str().ok_or("missing operation")?;
        let args = request["args"].as_array().ok_or("missing args")?;
        let response = match support::evaluate(operation, args) {
            Ok(value) => json!({"id": request["id"], "ok": true, "value": value}),
            Err(error) => {
                json!({"id": request["id"], "ok": false, "error": {"name": "TypeError", "message": error.to_string()}})
            }
        };
        serde_json::to_writer(&mut output, &response)?;
        writeln!(output)?;
    }
    Ok(())
}
