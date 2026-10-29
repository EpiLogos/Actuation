use std::path::{Path, PathBuf};
use std::process::Command;

fn source_git(manifest: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .current_dir(manifest)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_INDEX_FILE")
        .args(args)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn full_revision(value: &str) -> bool {
    value.len() == 40 && lowercase_hex(value)
}

fn lowercase_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn watch_git_path(manifest: &Path, name: &str) {
    if let Some(path) = source_git(manifest, &["rev-parse", "--git-path", name]) {
        let path = PathBuf::from(path);
        let path = if path.is_absolute() {
            path
        } else {
            manifest.join(path)
        };
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

fn main() {
    println!("cargo:rerun-if-env-changed=ACTUATION_BUILD_SOURCE_REVISION");
    println!("cargo:rerun-if-env-changed=SUITE_BUILD_REVISION");
    println!("cargo:rerun-if-changed=build.rs");
    let manifest =
        PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo declares its source"));
    watch_git_path(&manifest, "HEAD");
    if let Some(reference) = source_git(&manifest, &["symbolic-ref", "--quiet", "HEAD"]) {
        watch_git_path(&manifest, &reference);
    }
    let revision = match std::env::var("ACTUATION_BUILD_SOURCE_REVISION") {
        Ok(explicit) => {
            assert!(
                full_revision(&explicit),
                "ACTUATION_BUILD_SOURCE_REVISION must be the full lowercase source Git revision"
            );
            explicit
        }
        Err(std::env::VarError::NotUnicode(_)) => {
            panic!("ACTUATION_BUILD_SOURCE_REVISION is not UTF-8")
        }
        // `oi update` builds a `git archive` export with no `.git`, and names
        // the cut it exported in SUITE_BUILD_REVISION (its short form).
        Err(std::env::VarError::NotPresent) => std::env::var("SUITE_BUILD_REVISION")
            .ok()
            .filter(|revision| (7..=40).contains(&revision.len()) && lowercase_hex(revision))
            .or_else(|| {
                source_git(&manifest, &["rev-parse", "HEAD"])
                    .filter(|revision| full_revision(revision))
            })
            .unwrap_or_else(|| "unknown".into()),
    };
    println!("cargo:rustc-env=ACTUATION_SOURCE_REVISION={revision}");
}
