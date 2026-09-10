#[path = "../tests/support/mod.rs"]
mod support;
use std::io::{self, BufRead};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for line in io::stdin().lock().lines() {
        let row: serde_json::Value = serde_json::from_str(&line?)?;
        println!("{}", support::pure_row(&row));
    }
    Ok(())
}
