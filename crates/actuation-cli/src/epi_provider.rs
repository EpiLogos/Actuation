//! Product launcher for the Epi-Logos Prime-QL acting body.
//!
//! The launcher owns no model selection: AIKit appends the resolved native
//! provider/model to this command. It materialises the Actuation-owned Prime
//! constitution around that resolved model, pins the QL owner revision, and
//! then runs Prime's native JSONL RPC as the encounter process.
//!
//! The shared Epi/QL extension is loaded explicitly (`-e`), because discovery is
//! disabled and Prime then loads only explicit extensions. Prime swallows
//! extension load errors in RPC mode, so a missing or drifted QL binding cannot
//! be left to the extension: the launcher verifies the binding first and refuses
//! to start Prime without it.

use actuation_core::{Error, Result};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

const BINDING_SCHEMA: &str = "actuation.prime-faculty-installation/v1";
const OWNER_REQUIRED: [&str; 5] = [
    "extensions/ql-agent.ts",
    "extensions/ql-event-context.ts",
    "ql-agent-contracts-v1.schema.json",
    "provenance.json",
    "package.json",
];

#[derive(Debug, Clone)]
pub struct EpiProviderArgs {
    pub prime_bin: PathBuf,
    pub ql_bin: PathBuf,
    pub ql_revision: String,
    pub skill_path: PathBuf,
    /// The packaged binding extension (`ql-faculty-bindings.ts`, from EpiLogos/Epi-Prime), loaded with `-e`.
    pub extension: PathBuf,
    /// The binding file (`actuation.prime-faculty-installation/v1`) the
    /// extension reads; verified before Prime starts.
    pub installation: PathBuf,
    pub research_bin: PathBuf,
    pub faculty_config: PathBuf,
    pub ql_root: Option<PathBuf>,
    pub aikit_bin: Option<PathBuf>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub agent_session: Option<String>,
    pub central_ctrl_bin: Option<PathBuf>,
    pub central_root: Option<PathBuf>,
    pub central_project: Option<String>,
    pub child_message_dir: Option<PathBuf>,
}

fn value(args: &[String], name: &str) -> Result<String> {
    let index = args
        .iter()
        .position(|value| value == name)
        .ok_or_else(|| Error::new(format!("missing {name}")))?;
    args.get(index + 1)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .ok_or_else(|| Error::new(format!("{name} requires a value")))
}

fn optional(args: &[String], name: &str) -> Option<String> {
    let index = args.iter().position(|value| value == name)?;
    args.get(index + 1)
        .filter(|value| !value.trim().is_empty())
        .cloned()
}

fn file(path: PathBuf, label: &str) -> Result<PathBuf> {
    if !path.is_absolute() {
        return Err(Error::new(format!("{label} must be an absolute path")));
    }
    let canonical =
        std::fs::canonicalize(&path).map_err(|_| Error::new(format!("{label} is unavailable")))?;
    if !canonical.is_file() {
        return Err(Error::new(format!("{label} must be a regular file")));
    }
    Ok(canonical)
}

fn directory(path: PathBuf, label: &str) -> Result<PathBuf> {
    if !path.is_absolute() {
        return Err(Error::new(format!("{label} must be an absolute path")));
    }
    let canonical =
        std::fs::canonicalize(&path).map_err(|_| Error::new(format!("{label} is unavailable")))?;
    if !canonical.is_dir() {
        return Err(Error::new(format!("{label} must be a directory")));
    }
    Ok(canonical)
}

impl EpiProviderArgs {
    pub fn parse(args: &[String]) -> Result<Self> {
        let ql_revision = value(args, "--ql-revision")?;
        if ql_revision.len() != 40
            || !ql_revision
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(Error::new(
                "--ql-revision must be a lowercase 40-hex revision",
            ));
        }
        let provider = optional(args, "--provider");
        let model = optional(args, "--model");
        if provider.is_some() != model.is_some() {
            return Err(Error::new(
                "--provider and --model must be supplied together or both omitted",
            ));
        }
        if provider
            .as_ref()
            .is_some_and(|provider| provider.starts_with('-') || provider.len() > 256)
            || model
                .as_ref()
                .is_some_and(|model| model.starts_with('-') || model.len() > 1024)
        {
            return Err(Error::new(
                "provider/model identifiers are invalid or unbounded",
            ));
        }
        let ql_root = optional(args, "--ql-root")
            .map(PathBuf::from)
            .map(|path| directory(path, "QL source root"))
            .transpose()?;
        Ok(Self {
            prime_bin: file(PathBuf::from(value(args, "--prime-bin")?), "Prime binary")?,
            ql_bin: file(PathBuf::from(value(args, "--ql-bin")?), "QL binary")?,
            ql_revision,
            skill_path: directory(
                PathBuf::from(value(args, "--skill-path")?),
                "QL relational skill",
            )?,
            extension: file(
                PathBuf::from(value(args, "--extension")?),
                "Epi/QL binding extension",
            )?,
            installation: file(
                PathBuf::from(value(args, "--installation")?),
                "QL binding (installation) file",
            )?,
            research_bin: file(
                PathBuf::from(value(args, "--research-bin")?),
                "Actuation research binary",
            )?,
            faculty_config: file(
                PathBuf::from(value(args, "--faculty-config")?),
                "Actuation faculty configuration",
            )?,
            ql_root,
            aikit_bin: optional(args, "--aikit-bin")
                .map(PathBuf::from)
                .map(|path| file(path, "AIKit binary"))
                .transpose()?,
            provider,
            model,
            agent_session: optional(args, "--agent-session"),
            central_ctrl_bin: optional(args, "--central-ctrl-bin")
                .map(PathBuf::from)
                .map(|path| file(path, "Central ctrl binary"))
                .transpose()?,
            central_root: optional(args, "--central-root")
                .map(PathBuf::from)
                .map(|path| directory(path, "Central root"))
                .transpose()?,
            central_project: optional(args, "--central-project"),
            child_message_dir: optional(args, "--child-message-dir")
                .map(PathBuf::from)
                .map(|path| directory(path, "child message directory"))
                .transpose()?,
        })
    }
}

fn sha256_hex(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).map_err(|_| {
        Error::new(format!(
            "QL binding member {} is unreadable",
            path.display()
        ))
    })?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

/// Verify the QL binding exactly as the binding extension does (schema, absolute
/// binary/configuration, the owner extension's whole source closure by SHA-256,
/// entry point and path containment). A failure is an error that stops the
/// launch: a body presented as Epi-Logos Prime-QL never starts without QL.
pub fn verify_binding(installation: &Path) -> Result<()> {
    let refuse = |why: &str| {
        Error::new(format!(
            "QL binding refused: {why} ({})",
            installation.display()
        ))
    };
    let bytes = std::fs::read(installation).map_err(|_| refuse("unreadable"))?;
    if bytes.len() > 65536 {
        return Err(refuse("larger than the binding bound"));
    }
    let binding: Value = serde_json::from_slice(&bytes).map_err(|_| refuse("not JSON"))?;
    if binding["schema"] != BINDING_SCHEMA {
        return Err(refuse("wrong schema"));
    }
    for key in ["binary", "configuration"] {
        if !binding[key]
            .as_str()
            .is_some_and(|p| Path::new(p).is_absolute())
        {
            return Err(refuse(&format!("{key} is not an absolute path")));
        }
    }
    let owner = &binding["owner_extension"];
    let root = owner["root"]
        .as_str()
        .filter(|p| Path::new(p).is_absolute())
        .ok_or_else(|| refuse("owner extension root is not an absolute path"))?;
    let root =
        std::fs::canonicalize(root).map_err(|_| refuse("owner extension root is missing"))?;
    let module = owner["module"]
        .as_str()
        .ok_or_else(|| refuse("owner extension has no module"))?;
    let digest_ok = |d: &Value| {
        d.as_str()
            .is_some_and(|d| d.len() == 64 && d.bytes().all(|b| b.is_ascii_hexdigit()))
    };
    let sources = owner["source_files"]
        .as_object()
        .filter(|m| !m.is_empty() && m.len() <= 64)
        .ok_or_else(|| refuse("owner source closure is absent or unbounded"))?;
    for required in OWNER_REQUIRED {
        if !sources.contains_key(required) {
            return Err(refuse(&format!("owner source closure lacks {required}")));
        }
    }
    let mut total = 0u64;
    for (name, expected) in sources {
        let relative = Path::new(name);
        if name.is_empty()
            || relative.is_absolute()
            || relative
                .components()
                .any(|c| matches!(c, Component::ParentDir))
            || !digest_ok(expected)
        {
            return Err(refuse(&format!("owner source member {name} is invalid")));
        }
        let path = std::fs::canonicalize(root.join(relative))
            .map_err(|_| refuse(&format!("owner source member {name} is missing")))?;
        let meta =
            std::fs::metadata(&path).map_err(|_| refuse(&format!("{name} is unreadable")))?;
        total += meta.len();
        if !path.starts_with(&root)
            || !meta.is_file()
            || meta.len() > 1_048_576
            || total > 2_097_152
        {
            return Err(refuse(&format!(
                "owner source member {name} is outside its bounds"
            )));
        }
        if expected.as_str() != Some(sha256_hex(&path)?.as_str()) {
            return Err(refuse(&format!(
                "{name} differs from its installed source digest"
            )));
        }
    }
    let module =
        std::fs::canonicalize(module).map_err(|_| refuse("owner entry module is missing"))?;
    if module != root.join("extensions/ql-agent.ts")
        || owner["sha256"] != sources["extensions/ql-agent.ts"]
    {
        return Err(refuse(
            "owner entry is not extensions/ql-agent.ts of the verified closure",
        ));
    }
    Ok(())
}

/// The exact Prime command line and the environment added to it. Separate from
/// `run` so the flags that matter are testable without starting Prime.
pub fn prime_command_spec(
    args: &EpiProviderArgs,
    cwd: &Path,
) -> (Vec<String>, Vec<(String, String)>) {
    let text = |p: &Path| p.to_string_lossy().into_owned();
    let mut argv: Vec<String> = [
        "--mode",
        "rpc",
        "--cwd",
        &text(cwd),
        // Discovery is off; the shared QL extension is the explicit exception.
        "--no-extensions",
        "-e",
        &text(&args.extension),
        "--no-prompt-templates",
        "--no-context-files",
        "--no-skills",
        "--skill",
        &text(&args.skill_path),
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect();
    if let (Some(provider), Some(model)) = (&args.provider, &args.model) {
        argv.extend([
            "--provider".into(),
            provider.clone(),
            "--model".into(),
            model.clone(),
        ]);
    }
    let mut env = vec![
        ("RLM_MAX_DEPTH".to_owned(), "2".to_owned()),
        ("DO_NOT_TRACK".to_owned(), "1".to_owned()),
        ("QL_BIN".to_owned(), text(&args.ql_bin)),
        ("QL_OWNER_REVISION".to_owned(), args.ql_revision.clone()),
        (
            "ACTUATION_RESEARCH_BIN".to_owned(),
            text(&args.research_bin),
        ),
        (
            "ACTUATION_RESEARCH_FACULTY_CONFIG".to_owned(),
            text(&args.faculty_config),
        ),
        (
            "ACTUATION_RESEARCH_INSTALLATION".to_owned(),
            text(&args.installation),
        ),
    ];
    // The QL workspace builds `ql` and `ql-wiki-refraction` side by side. When the sibling exists the skill runs it directly instead of
    // `cargo run`-ing the QL checkout at call time (which compiles whatever state that checkout is in, mid-turn).
    if let Some(sibling) = args
        .ql_bin
        .parent()
        .map(|dir| dir.join("ql-wiki-refraction"))
        .filter(|path| path.is_file())
    {
        env.push(("QL_WIKI_REFRACTION_BIN".into(), text(&sibling)));
    }
    if let Some(root) = &args.ql_root {
        env.push(("QL_MEF_ROOT".into(), text(root)));
    }
    if let Some(dir) = &args.child_message_dir {
        env.push(("ACTUATION_CHILD_MESSAGE_DIR".into(), text(dir)));
    }
    if let Some(aikit) = &args.aikit_bin {
        env.push(("AIKIT_BIN".into(), text(aikit)));
    }
    if let Some(ctrl) = &args.central_ctrl_bin {
        env.push(("CENTRAL_CTRL_BIN".into(), text(ctrl)));
    }
    if let Some(root) = &args.central_root {
        env.push(("CENTRAL_ROOT".into(), text(root)));
    }
    if let Some(project) = &args.central_project {
        env.push(("CENTRAL_PROJECT".into(), project.clone()));
    }
    if let Some(agent_session) = &args.agent_session {
        env.push(("ACTUATION_RESEARCH_TRACE_REF".into(), agent_session.clone()));
        env.push(("ACTUATION_RESEARCH_LOCUS_REF".into(), agent_session.clone()));
    }
    (argv, env)
}

pub fn run(args: EpiProviderArgs) -> Result<i32> {
    let cwd = std::env::current_dir()
        .map_err(|_| Error::new("Prime-QL provider has no working directory"))?;
    let cwd = std::fs::canonicalize(cwd)
        .map_err(|_| Error::new("Prime-QL provider working directory is unavailable"))?;
    if args.central_ctrl_bin.is_some() != args.central_root.is_some() {
        return Err(Error::new(
            "--central-ctrl-bin and --central-root must be supplied together or both omitted",
        ));
    }
    if let Some(project) = &args.central_project {
        if project.trim().is_empty() || project.len() > 256 || project.chars().any(char::is_control)
        {
            return Err(Error::new("--central-project is invalid"));
        }
    }
    if let Some(agent_session) = &args.agent_session {
        if agent_session.len() > 256 || agent_session.trim().is_empty() {
            return Err(Error::new("--agent-session is invalid"));
        }
    }
    // Fail closed: no verified QL binding, no Prime-QL body.
    verify_binding(&args.installation)?;
    let (argv, env) = prime_command_spec(&args, &cwd);
    let mut command = Command::new(&args.prime_bin);
    command
        .args(&argv)
        .envs(env)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let status = command
        .status()
        .map_err(|_| Error::new("Prime-QL provider process could not start"))?;
    Ok(status.code().unwrap_or(1))
}

pub fn usage() -> &'static str {
    "actuation-epi-prime --prime-bin <abs> --ql-bin <abs> --ql-revision <sha> \
--skill-path <abs> --extension <abs> --installation <abs> \
--research-bin <abs> --faculty-config <abs> \
[--ql-root <abs>] [--aikit-bin <abs>] [--agent-session <ref>] \
[--central-ctrl-bin <abs> --central-root <abs> [--central-project <key>]] \
[--child-message-dir <abs>] [--provider <native> --model <id>]"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_and_model_are_one_explicit_override() {
        let args = vec![
            "--prime-bin".into(),
            "/usr/bin/true".into(),
            "--ql-bin".into(),
            "/usr/bin/true".into(),
            "--ql-revision".into(),
            "0123456789abcdef0123456789abcdef01234567".into(),
            "--skill-path".into(),
            "/tmp".into(),
            "--research-bin".into(),
            "/usr/bin/true".into(),
            "--faculty-config".into(),
            "/usr/bin/true".into(),
            "--provider".into(),
            "p".into(),
        ];
        let error = EpiProviderArgs::parse(&args).unwrap_err();
        assert!(error.to_string().contains("supplied together"));
    }

    #[test]
    fn ql_revision_is_exact_not_a_label() {
        let args = vec![
            "--prime-bin".into(),
            "/usr/bin/true".into(),
            "--ql-bin".into(),
            "/usr/bin/true".into(),
            "--ql-revision".into(),
            "short".into(),
            "--skill-path".into(),
            "/tmp".into(),
            "--research-bin".into(),
            "/usr/bin/true".into(),
            "--faculty-config".into(),
            "/usr/bin/true".into(),
            "--provider".into(),
            "p".into(),
            "--model".into(),
            "m".into(),
        ];
        let error = EpiProviderArgs::parse(&args).unwrap_err();
        assert!(error.to_string().contains("40-hex"));
    }

    use std::os::unix::fs::PermissionsExt;

    fn digest(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    /// A real binding over a real owner closure on disk.
    fn binding_dir(dir: &Path) -> PathBuf {
        let root = dir.join("owner");
        let mut sources = serde_json::Map::new();
        for name in OWNER_REQUIRED {
            let path = root.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            let body = format!("member {name}");
            std::fs::write(&path, &body).unwrap();
            sources.insert(name.into(), Value::String(digest(body.as_bytes())));
        }
        let binding = serde_json::json!({
            "schema": BINDING_SCHEMA,
            "binary": "/usr/bin/true", "configuration": "/usr/bin/true",
            "owner_extension": {
                "root": root, "module": root.join("extensions/ql-agent.ts"),
                "sha256": sources["extensions/ql-agent.ts"], "source_files": sources
            }
        });
        let path = dir.join("current.json");
        std::fs::write(&path, serde_json::to_vec(&binding).unwrap()).unwrap();
        path
    }

    fn launch_args(dir: &Path, prime: &Path, installation: &Path) -> Vec<String> {
        let extension = dir.join("ql-faculty-bindings.ts");
        std::fs::write(&extension, "export default () => {}").unwrap();
        let skill = dir.join("skill");
        std::fs::create_dir_all(&skill).unwrap();
        [
            ("--prime-bin", prime.display().to_string()),
            ("--ql-bin", "/usr/bin/true".into()),
            (
                "--ql-revision",
                "0123456789abcdef0123456789abcdef01234567".into(),
            ),
            ("--skill-path", skill.display().to_string()),
            ("--extension", extension.display().to_string()),
            ("--installation", installation.display().to_string()),
            ("--research-bin", "/usr/bin/true".into()),
            ("--faculty-config", "/usr/bin/true".into()),
        ]
        .into_iter()
        .flat_map(|(k, v)| [k.to_owned(), v])
        .collect()
    }

    #[test]
    fn the_shared_extension_is_loaded_explicitly_while_discovery_stays_off() {
        let dir = tempfile::tempdir().unwrap();
        let installation = binding_dir(dir.path());
        let args = EpiProviderArgs::parse(&launch_args(
            dir.path(),
            Path::new("/usr/bin/true"),
            &installation,
        ))
        .unwrap();
        let (argv, env) = prime_command_spec(&args, Path::new("/work"));
        let discovery_off = argv.iter().position(|a| a == "--no-extensions").unwrap();
        let explicit = argv.iter().position(|a| a == "-e").expect("an explicit -e");
        assert!(explicit > discovery_off, "-e follows --no-extensions");
        assert_eq!(argv[explicit + 1], args.extension.to_string_lossy());
        assert!(env
            .iter()
            .any(|(k, v)| k == "ACTUATION_RESEARCH_INSTALLATION"
                && v == &installation.canonicalize().unwrap().to_string_lossy()));
    }

    #[test]
    fn a_built_wiki_refraction_binary_beside_ql_is_handed_to_the_skill() {
        let dir = tempfile::tempdir().unwrap();
        let installation = binding_dir(dir.path());
        let bin = dir.path().join("qlbin");
        std::fs::create_dir_all(&bin).unwrap();
        let ql = bin.join("ql");
        std::fs::write(&ql, "#!/bin/sh\n").unwrap();
        let mut args = launch_args(dir.path(), Path::new("/usr/bin/true"), &installation);
        let at = args.iter().position(|a| a == "--ql-bin").unwrap();
        args[at + 1] = ql.display().to_string();
        let parsed = EpiProviderArgs::parse(&args).unwrap();
        let (_, env) = prime_command_spec(&parsed, Path::new("/work"));
        assert!(
            !env.iter().any(|(k, _)| k == "QL_WIKI_REFRACTION_BIN"),
            "no sibling, nothing named"
        );
        let sibling = bin.join("ql-wiki-refraction");
        std::fs::write(&sibling, "#!/bin/sh\n").unwrap();
        let (_, env) = prime_command_spec(&parsed, Path::new("/work"));
        let sibling = sibling.canonicalize().unwrap();
        assert!(env
            .iter()
            .any(|(k, v)| k == "QL_WIKI_REFRACTION_BIN" && v == &sibling.to_string_lossy()));
    }

    #[test]
    fn a_missing_extension_or_binding_argument_is_refused_at_parse() {
        let dir = tempfile::tempdir().unwrap();
        let installation = binding_dir(dir.path());
        for flag in ["--extension", "--installation"] {
            let mut args = launch_args(dir.path(), Path::new("/usr/bin/true"), &installation);
            let at = args.iter().position(|a| a == flag).unwrap();
            args.drain(at..at + 2);
            let error = EpiProviderArgs::parse(&args).unwrap_err();
            assert!(error.to_string().contains(flag), "{error}");
        }
    }

    #[test]
    fn a_binding_is_verified_by_schema_closure_and_digest() {
        let dir = tempfile::tempdir().unwrap();
        let installation = binding_dir(dir.path());
        verify_binding(&installation).unwrap();

        // One changed byte in the owner source.
        let member = dir.path().join("owner/extensions/ql-agent.ts");
        std::fs::write(&member, "tampered").unwrap();
        assert!(verify_binding(&installation)
            .unwrap_err()
            .to_string()
            .contains("differs"));
        std::fs::write(&member, "member extensions/ql-agent.ts").unwrap();
        verify_binding(&installation).unwrap();

        // A member the binding claims but disk lacks.
        std::fs::remove_file(dir.path().join("owner/provenance.json")).unwrap();
        assert!(verify_binding(&installation)
            .unwrap_err()
            .to_string()
            .contains("missing"));
        std::fs::write(
            dir.path().join("owner/provenance.json"),
            "member provenance.json",
        )
        .unwrap();

        // Schema, closure membership, path escape and relative paths.
        let good: Value = serde_json::from_slice(&std::fs::read(&installation).unwrap()).unwrap();
        let mutate = |f: &dyn Fn(&mut Value)| {
            let mut value = good.clone();
            f(&mut value);
            let path = dir.path().join("mutated.json");
            std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
            verify_binding(&path).unwrap_err().to_string()
        };
        assert!(mutate(&|v| v["schema"] = "other".into()).contains("wrong schema"));
        assert!(mutate(&|v| v["binary"] = "relative/bin".into()).contains("absolute"));
        assert!(mutate(&|v| {
            v["owner_extension"]["source_files"]
                .as_object_mut()
                .unwrap()
                .remove("package.json");
        })
        .contains("lacks package.json"));
        assert!(mutate(&|v| {
            v["owner_extension"]["source_files"]["../outside"] = "a".repeat(64).into();
        })
        .contains("invalid"));
    }

    #[test]
    fn the_launcher_never_starts_prime_without_a_verified_binding() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join("PRIME-STARTED");
        let prime = dir.path().join("prime");
        std::fs::write(
            &prime,
            format!("#!/bin/sh\ntouch \"{}\"\n", marker.display()),
        )
        .unwrap();
        std::fs::set_permissions(&prime, std::fs::Permissions::from_mode(0o755)).unwrap();
        let installation = binding_dir(dir.path());
        let args = launch_args(dir.path(), &prime, &installation);

        // Drift: Prime must not start.
        std::fs::write(dir.path().join("owner/extensions/ql-agent.ts"), "tampered").unwrap();
        let error = run(EpiProviderArgs::parse(&args).unwrap()).unwrap_err();
        assert!(error.to_string().contains("QL binding refused"), "{error}");
        assert!(!marker.exists(), "Prime started despite a drifted binding");

        // A verified binding starts it.
        std::fs::write(
            dir.path().join("owner/extensions/ql-agent.ts"),
            "member extensions/ql-agent.ts",
        )
        .unwrap();
        assert_eq!(run(EpiProviderArgs::parse(&args).unwrap()).unwrap(), 0);
        assert!(
            marker.exists(),
            "Prime did not start with a verified binding"
        );
    }
}
