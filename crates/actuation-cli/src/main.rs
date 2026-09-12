//! Process plumbing: read stdin exactly when the matched command needs it,
//! write the output envelope, and map semantic refusals to exit status 2.
use std::io::Write;

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let stdin = if actuation_cli::dispatch::command_needs_stdin(&argv) {
        actuation_cli::dispatch::read_stdin()
    } else {
        String::new()
    };
    let code = match actuation_cli::execute(&argv, &stdin) {
        Ok(output) => {
            if !output.stdout.is_empty() {
                println!("{}", output.stdout);
            }
            if !output.stderr.is_empty() {
                eprintln!("{}", output.stderr);
            }
            output.code
        }
        Err(error) => {
            eprintln!("actuation: {error}");
            2
        }
    };
    let _ = std::io::stdout().flush();
    std::process::exit(code);
}
