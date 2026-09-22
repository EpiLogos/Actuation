//! Product launcher for the Epi-Logos Prime-QL acting body.
//!
//! The launcher owns no model selection: AIKit appends the resolved native
//! provider/model to this command. It materialises the Actuation-owned Prime
//! constitution around that resolved model, pins the QL owner revision, and
//! then runs Prime's native JSONL RPC as the encounter process.

use actuation_core::{Error, Result};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Debug, Clone)]
pub struct EpiProviderArgs {
    pub prime_bin: PathBuf,
    pub ql_bin: PathBuf,
    pub ql_revision: String,
    pub skill_path: PathBuf,
    pub research_bin: PathBuf,
    pub faculty_config: PathBuf,
    pub ql_root: Option<PathBuf>,
    pub provider: String,
    pub model: String,
    pub agent_session: Option<String>,
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
    args.get(index + 1).filter(|value| !value.trim().is_empty()).cloned()
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
    let canonical = std::fs::canonicalize(&path)
        .map_err(|_| Error::new(format!("{label} is unavailable")))?;
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
        let provider = value(args, "--provider")?;
        let model = value(args, "--model")?;
        if provider.starts_with('-')
            || model.starts_with('-')
            || provider.len() > 256
            || model.len() > 1024
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
            research_bin: file(
                PathBuf::from(value(args, "--research-bin")?),
                "Actuation research binary",
            )?,
            faculty_config: file(
                PathBuf::from(value(args, "--faculty-config")?),
                "Actuation faculty configuration",
            )?,
            ql_root,
            provider,
            model,
            agent_session: optional(args, "--agent-session"),
        })
    }
}

pub fn run(args: EpiProviderArgs) -> Result<i32> {
    let cwd = std::env::current_dir()
        .map_err(|_| Error::new("Prime-QL provider has no working directory"))?;
    let cwd = std::fs::canonicalize(cwd)
        .map_err(|_| Error::new("Prime-QL provider working directory is unavailable"))?;

    let mut command = Command::new(&args.prime_bin);
    command
        .args([
            "--mode",
            "rpc",
            "--cwd",
            cwd.to_string_lossy().as_ref(),
            "--no-extensions",
            "--no-prompt-templates",
            "--no-context-files",
            "--no-skills",
            "--provider",
            &args.provider,
            "--model",
            &args.model,
            "--skill",
            args.skill_path.to_string_lossy().as_ref(),
        ])
        .env("RLM_MAX_DEPTH", "2")
        .env("DO_NOT_TRACK", "1")
        .env("QL_BIN", &args.ql_bin)
        .env("QL_OWNER_REVISION", &args.ql_revision)
        .env("ACTUATION_RESEARCH_BIN", &args.research_bin)
        .env("ACTUATION_RESEARCH_FACULTY_CONFIG", &args.faculty_config)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    if let Some(root) = &args.ql_root {
        command.env("QL_MEF_ROOT", root);
    }
    if let Some(agent_session) = &args.agent_session {
        if agent_session.len() > 256 || agent_session.trim().is_empty() {
            return Err(Error::new("--agent-session is invalid"));
        }
        command
            .env("ACTUATION_RESEARCH_TRACE_REF", agent_session)
            .env("ACTUATION_RESEARCH_LOCUS_REF", agent_session);
    }

    let status = command
        .status()
        .map_err(|_| Error::new("Prime-QL provider process could not start"))?;
    Ok(status.code().unwrap_or(1))
}

pub fn usage() -> &'static str {
    "actuation-epi-prime --prime-bin <abs> --ql-bin <abs> --ql-revision <sha> \
--skill-path <abs> --research-bin <abs> --faculty-config <abs> \
[--ql-root <abs>] [--agent-session <ref>] --provider <native> --model <id>"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ql_revision_is_exact_not_a_label() {
        let args = vec![
            "--prime-bin".into(),
            "/bin/true".into(),
            "--ql-bin".into(),
            "/bin/true".into(),
            "--ql-revision".into(),
            "short".into(),
            "--skill-path".into(),
            "/tmp".into(),
            "--research-bin".into(),
            "/bin/true".into(),
            "--faculty-config".into(),
            "/bin/true".into(),
            "--provider".into(),
            "p".into(),
            "--model".into(),
            "m".into(),
        ];
        let error = EpiProviderArgs::parse(&args).unwrap_err();
        assert!(error.to_string().contains("40-hex"));
    }
}
