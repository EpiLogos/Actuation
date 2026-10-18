//! A basis-keyed memo of the two expensive executable observations — the
//! fingerprint (`sha256`) and the version probe — kept in Actuation's own state
//! (`~/.actuation/probe-cache.json`). A detection run hashes every detected
//! harness executable (Claude Code's alone is ~240 MB) and, with `--versions`,
//! spawns each one; neither answer changes while the executable is the same
//! file. An entry answers only while the executable's device, inode, size and
//! modification time are exactly those it was observed at; anything else
//! re-probes. A probe that timed out or could not reach its target is never
//! kept — a transient bound is not an observation of the binary — while a
//! version the target refused (`exit N`) is, because the same bytes refuse the
//! same way. A cache file that cannot be read is ignored and rewritten; one that
//! cannot be written costs nothing but the next run's probes.
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub const PROBE_CACHE_SCHEMA: &str = "actuation.probe-cache/v1";
/// More executables than any catalog declares; the oldest-keyed entries beyond
/// it are dropped at save.
const MAX_ENTRIES: usize = 512;

/// What the executable file is, as the filesystem says: the basis an entry is
/// valid for. Two observations of the same basis are observations of the same
/// bytes for every purpose detection has.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutableBasis {
    pub dev: u64,
    pub ino: u64,
    pub size: u64,
    pub mtime_ns: i128,
}

impl ExecutableBasis {
    /// The basis of the file `path` resolves to (symlinks followed), or None
    /// where the platform or the file gives none — then nothing is cached.
    pub fn of(path: &Path) -> Option<Self> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let m = fs::metadata(path).ok()?;
            if !m.is_file() {
                return None;
            }
            Some(Self {
                dev: m.dev(),
                ino: m.ino(),
                size: m.size(),
                mtime_ns: i128::from(m.mtime()) * 1_000_000_000 + i128::from(m.mtime_nsec()),
            })
        }
        #[cfg(not(unix))]
        {
            let _ = path;
            None
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Entry {
    basis: ExecutableBasis,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sha256: Option<String>,
    /// Version outcomes by their argv (joined with U+001F): the first stdout
    /// line, or the target's refusal.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    versions: BTreeMap<String, VersionOutcome>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", content = "text", rename_all = "kebab-case")]
enum VersionOutcome {
    Answered(String),
    Refused(String),
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CacheFile {
    schema: String,
    entries: BTreeMap<String, Entry>,
}

/// The memo, bound to the file it loads from and saves to.
#[derive(Debug)]
pub struct ProbeCache {
    path: PathBuf,
    file: CacheFile,
    dirty: bool,
}

impl ProbeCache {
    /// Load the memo at `path`; an absent, unreadable or foreign-schema file is
    /// an empty memo that will replace it at save.
    pub fn load(path: PathBuf) -> Self {
        let file = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<CacheFile>(&bytes).ok())
            .filter(|file| file.schema == PROBE_CACHE_SCHEMA)
            .unwrap_or_default();
        Self {
            path,
            file,
            dirty: false,
        }
    }

    fn key(path: &Path) -> String {
        fs::canonicalize(path)
            .unwrap_or_else(|_| path.to_path_buf())
            .to_string_lossy()
            .into_owned()
    }

    fn args_key(args: &[String]) -> String {
        args.join("\u{1f}")
    }

    fn valid(&self, key: &str, basis: &ExecutableBasis) -> Option<&Entry> {
        self.file
            .entries
            .get(key)
            .filter(|entry| &entry.basis == basis)
    }

    /// The entry for `key` at `basis`, a fresh one when the file changed.
    fn entry_mut(&mut self, key: String, basis: ExecutableBasis) -> &mut Entry {
        self.dirty = true;
        let entry = self.file.entries.entry(key).or_insert_with(|| Entry {
            basis: basis.clone(),
            sha256: None,
            versions: BTreeMap::new(),
        });
        if entry.basis != basis {
            *entry = Entry {
                basis,
                sha256: None,
                versions: BTreeMap::new(),
            };
        }
        entry
    }

    /// The executable's fingerprint, from the memo when its basis still holds,
    /// else from `observe` (kept on success).
    pub fn fingerprint(
        &mut self,
        path: &str,
        observe: impl FnOnce() -> Result<String, String>,
    ) -> Result<String, String> {
        let file = Path::new(path);
        let Some(basis) = ExecutableBasis::of(file) else {
            return observe();
        };
        let key = Self::key(file);
        if let Some(sha) = self
            .valid(&key, &basis)
            .and_then(|entry| entry.sha256.clone())
        {
            return Ok(sha);
        }
        let sha = observe()?;
        // The bytes hashed must be the bytes the basis names: a file replaced
        // during the read is not remembered.
        if ExecutableBasis::of(file).as_ref() == Some(&basis) {
            self.entry_mut(key, basis).sha256 = Some(sha.clone());
        }
        Ok(sha)
    }

    /// The executable's version answer for `args`, from the memo when its basis
    /// still holds, else from `observe`. An answer and a refusal (`exit N`) are
    /// kept; any other failure (timed out, unreachable) is not.
    pub fn version(
        &mut self,
        path: &str,
        args: &[String],
        observe: impl FnOnce() -> Result<String, String>,
    ) -> Result<String, String> {
        let file = Path::new(path);
        let Some(basis) = ExecutableBasis::of(file) else {
            return observe();
        };
        let key = Self::key(file);
        let args_key = Self::args_key(args);
        if let Some(outcome) = self
            .valid(&key, &basis)
            .and_then(|entry| entry.versions.get(&args_key).cloned())
        {
            return match outcome {
                VersionOutcome::Answered(text) => Ok(text),
                VersionOutcome::Refused(text) => Err(text),
            };
        }
        let result = observe();
        let kept = match &result {
            Ok(text) => Some(VersionOutcome::Answered(text.clone())),
            Err(reason) if is_refusal(reason) => Some(VersionOutcome::Refused(reason.clone())),
            Err(_) => None,
        };
        if let Some(outcome) = kept {
            if ExecutableBasis::of(file).as_ref() == Some(&basis) {
                self.entry_mut(key, basis)
                    .versions
                    .insert(args_key, outcome);
            }
        }
        result
    }

    /// Write the memo back when it changed: entries whose executable is gone
    /// or changed are dropped, the newest MAX_ENTRIES kept, the file replaced
    /// atomically. Failure is silent by design (see the module note).
    pub fn save(&mut self) {
        if !self.dirty {
            return;
        }
        self.file.schema = PROBE_CACHE_SCHEMA.into();
        self.file.entries.retain(|key, entry| {
            ExecutableBasis::of(Path::new(key)).as_ref() == Some(&entry.basis)
        });
        while self.file.entries.len() > MAX_ENTRIES {
            let Some(oldest) = self
                .file
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.basis.mtime_ns)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            self.file.entries.remove(&oldest);
        }
        let Ok(bytes) = serde_json::to_vec_pretty(&self.file) else {
            return;
        };
        let Some(dir) = self.path.parent() else {
            return;
        };
        if fs::create_dir_all(dir).is_err() {
            return;
        }
        let tmp = dir.join(format!(".probe-cache.{}.tmp", std::process::id()));
        if fs::write(&tmp, bytes).is_ok() && fs::rename(&tmp, &self.path).is_ok() {
            self.dirty = false;
        } else {
            let _ = fs::remove_file(&tmp);
        }
    }
}

/// The version probe's own refusal spelling (effects.rs `version`): the target
/// ran and exited non-zero.
fn is_refusal(reason: &str) -> bool {
    reason.starts_with("exit ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[test]
    fn a_second_observation_of_the_same_file_is_answered_from_the_memo() {
        let dir = tempfile::tempdir().unwrap();
        let exe = script(dir.path(), "tool", "echo 1.0");
        let mut cache = ProbeCache::load(dir.path().join("cache.json"));
        let path = exe.to_str().unwrap();
        let args = vec!["--version".to_owned()];
        let mut calls = 0;
        for _ in 0..3 {
            let answer = cache.version(path, &args, || {
                calls += 1;
                Ok("tool 1.0".into())
            });
            assert_eq!(answer, Ok("tool 1.0".into()));
        }
        assert_eq!(calls, 1);
        let mut hashes = 0;
        for _ in 0..2 {
            cache
                .fingerprint(path, || {
                    hashes += 1;
                    Ok("abc".into())
                })
                .unwrap();
        }
        assert_eq!(hashes, 1);
    }

    #[test]
    fn a_changed_executable_is_observed_again_and_the_memo_survives_a_reload() {
        let dir = tempfile::tempdir().unwrap();
        let exe = script(dir.path(), "tool", "echo 1.0");
        let store = dir.path().join("state/probe-cache.json");
        let path = exe.to_str().unwrap().to_owned();
        let args = vec!["--version".to_owned()];
        let mut cache = ProbeCache::load(store.clone());
        cache
            .version(&path, &args, || Ok("tool 1.0".into()))
            .unwrap();
        cache.save();
        assert!(store.exists());

        // A reload answers without observing.
        let mut reloaded = ProbeCache::load(store.clone());
        let answer = reloaded.version(&path, &args, || panic!("the memo should answer"));
        assert_eq!(answer, Ok("tool 1.0".into()));

        // Replacing the file (new size, new mtime, maybe a new inode) re-probes.
        std::thread::sleep(std::time::Duration::from_millis(20));
        script(dir.path(), "tool", "echo 2.0 with a longer body");
        let mut seen = false;
        let answer = reloaded.version(&path, &args, || {
            seen = true;
            Ok("tool 2.0".into())
        });
        assert!(seen);
        assert_eq!(answer, Ok("tool 2.0".into()));
    }

    #[test]
    fn a_refusal_is_kept_and_a_timeout_is_not() {
        let dir = tempfile::tempdir().unwrap();
        let exe = script(dir.path(), "tool", "exit 3");
        let path = exe.to_str().unwrap();
        let args = vec!["--version".to_owned()];
        let mut cache = ProbeCache::load(dir.path().join("cache.json"));
        assert_eq!(
            cache.version(path, &args, || Err("exit 3".into())),
            Err("exit 3".into())
        );
        assert_eq!(
            cache.version(path, &args, || panic!("a refusal is remembered")),
            Err("exit 3".into())
        );
        let other = vec!["-V".to_owned()];
        assert!(cache
            .version(path, &other, || Err("timed out after 10s".into()))
            .is_err());
        let mut again = false;
        let _ = cache.version(path, &other, || {
            again = true;
            Err("timed out after 10s".into())
        });
        assert!(again, "a timeout is never remembered");
    }

    #[test]
    fn an_unreadable_memo_is_ignored_and_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let exe = script(dir.path(), "tool", "echo 1.0");
        let store = dir.path().join("cache.json");
        fs::write(&store, b"{ not json").unwrap();
        let mut cache = ProbeCache::load(store.clone());
        let path = exe.to_str().unwrap();
        cache
            .version(path, &["--version".into()], || Ok("tool 1.0".into()))
            .unwrap();
        cache.save();
        let file: CacheFile = serde_json::from_slice(&fs::read(&store).unwrap()).unwrap();
        assert_eq!(file.schema, PROBE_CACHE_SCHEMA);
        assert_eq!(file.entries.len(), 1);
    }

    #[test]
    fn a_removed_executable_leaves_the_memo_at_save() {
        let dir = tempfile::tempdir().unwrap();
        let exe = script(dir.path(), "tool", "echo 1.0");
        let store = dir.path().join("cache.json");
        let mut cache = ProbeCache::load(store.clone());
        cache
            .fingerprint(exe.to_str().unwrap(), || Ok("abc".into()))
            .unwrap();
        fs::remove_file(&exe).unwrap();
        cache.save();
        let file: CacheFile = serde_json::from_slice(&fs::read(&store).unwrap()).unwrap();
        assert!(file.entries.is_empty());
    }
}
