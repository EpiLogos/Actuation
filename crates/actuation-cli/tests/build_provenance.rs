//! One real native binary retains its build source across foreign Git repos,
//! a directory with no repository, and a runtime PATH with no Git executable.
use std::process::Command;

fn reading(cwd: &std::path::Path, path: &std::path::Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_actuation"))
        .current_dir(cwd)
        .env("PATH", path)
        .env("ACTUATION_BUILD_SOURCE_REVISION", "f".repeat(40))
        .args(["capabilities", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    value["revision"]
        .as_str()
        .expect("native revision disclosure")
        .into()
}

#[test]
fn native_binary_reports_compiled_source_independent_of_caller_git_and_path() {
    let directory = tempfile::tempdir().unwrap();
    let empty_path = directory.path().join("no-executables");
    std::fs::create_dir(&empty_path).unwrap();
    let expected = env!("ACTUATION_SOURCE_REVISION");
    assert_eq!(reading(directory.path(), &empty_path), expected);
    for label in ["one", "two"] {
        let repository = directory.path().join(label);
        std::fs::create_dir(&repository).unwrap();
        assert!(Command::new("git")
            .current_dir(&repository)
            .args(["init", "--quiet"])
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .current_dir(&repository)
            .args([
                "-c",
                "user.name=Actuation test",
                "-c",
                "user.email=actuation-test@invalid.example",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "--quiet",
                "--allow-empty",
                "-m",
                label
            ])
            .status()
            .unwrap()
            .success());
        let runtime_path = std::env::var_os("PATH").unwrap_or_default();
        assert_eq!(
            reading(&repository, std::path::Path::new(&runtime_path)),
            expected
        );
        assert_eq!(reading(&repository, &empty_path), expected);
    }
}
