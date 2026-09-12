//! The README documents the served surface: every command usage line in the
//! descriptor table must appear in the README, so the product cannot grow a
//! command its documentation does not name (and vice versa is caught by the
//! frozen scenarios). This is the native continuation of the same law the
//! served Node CLI enforced on itself before the cutover.
use actuation_cli::dispatch::commands;

#[test]
fn readme_documents_every_command_the_table_declares() {
    let readme = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../README.md"))
        .expect("README must exist");
    for entry in commands() {
        assert!(
            readme.contains(entry.usage),
            "`{}` missing from README",
            entry.usage
        );
    }
}
