//! Optional research executable; not the R7 public Actuation CLI cutover.
use actuation_research::{application, Error, Result};
use serde_json::{json, Value};
use std::io::{self, Read};
fn request() -> Result<Value> {
    let mut bytes = vec![];
    io::stdin()
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::new("research input read failed"))?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(Error::new("research input exceeds bound"));
    }
    let v: Value =
        serde_json::from_slice(&bytes).map_err(|_| Error::new("invalid research request JSON"))?;
    application::invoke(&v)
}
fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let result = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["--version"] => {
            println!("actuation-research {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        ["help"] | ["--help"] => {
            println!("actuation-research [--json]\nRead one JSON operation from stdin and return one JSON result.\n\nOperations:\n{}",application::OPERATIONS.join("\n"));
            return;
        }
        ["capabilities"] | ["capabilities", "--json"] => Ok(application::capabilities()),
        [] | ["--json"] => request(),
        _ => Err(Error::new("unknown research command; use help")),
    };
    match result {
        Ok(v) => println!("{v}"),
        Err(e) => {
            println!(
                "{}",
                json!({"schema":"actuation.research-error/v1","error":e.to_string()})
            );
            std::process::exit(1);
        }
    }
}
