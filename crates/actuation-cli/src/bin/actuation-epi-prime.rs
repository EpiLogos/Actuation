use actuation_cli::epi_provider::{self, EpiProviderArgs};

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|value| value == "--help" || value == "-h") {
        println!("{}", epi_provider::usage());
        return;
    }
    match EpiProviderArgs::parse(&args).and_then(epi_provider::run) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("actuation-epi-prime: {error}");
            std::process::exit(2);
        }
    }
}
