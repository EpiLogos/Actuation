//! Owner-issued local governing-authority records.
//!
//! A record is the durable governing side of a local actualisation: one
//! governing WorldBinding, one MetagencyGrant, one holder, one scope of
//! allowed worlds. Events are append-only per authority source and
//! revocation is an event, never a rewrite. The store folds events into the
//! current state and nothing more: it does not admit, actualise or dispatch.
//!
//! Persistence follows the same discipline as the stream and occupancy stores:
//! every read-check-append runs under a lock on the store directory (exclusive
//! to mutate, shared to read), every open uses `O_NOFOLLOW`, and every append
//! is `sync_data`'d — with the directory synced when a source is first created.
//! Two racing issuers therefore serialise, so issuance stays a compare-and-swap
//! and a crash cannot leave a torn line. The on-disk format is unchanged.

use actuation_core::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

fn io<T>(result: std::io::Result<T>) -> Result<T> {
    result.map_err(|e| Error::new(format!("authority store I/O failure: {e}")))
}

pub const AUTHORITY_EVENT_VERSION: &str = "actuation.authority-event/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum AuthorityEvent {
    Issue {
        schema: String,
        at_unix_seconds: u64,
        record: Value,
    },
    Revoke {
        schema: String,
        at_unix_seconds: u64,
        reason: String,
    },
}

/// The folded current state of one authority source.
#[derive(Debug, Clone, Serialize)]
pub struct AuthorityState {
    pub record: Value,
    pub issued_at_unix_seconds: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked: Option<AuthorityRevocation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthorityRevocation {
    pub at_unix_seconds: u64,
    pub reason: String,
}

pub struct AuthorityStore {
    root: PathBuf,
}

impl AuthorityStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        if root.as_os_str().is_empty() {
            return Err(Error::new(
                "authority store root must be a non-empty directory path",
            ));
        }
        Ok(Self { root })
    }

    /// Same environment knob family as the stream store, with its own
    /// dedicated name so authority material never mixes with stream logs.
    pub fn from_environment(explicit: Option<&str>) -> Result<Self> {
        if let Some(root) = explicit {
            return Self::new(root);
        }
        if let Some(root) = std::env::var_os("ACTUATION_AUTHORITY_STORE") {
            return Self::new(PathBuf::from(root));
        }
        let home = std::env::var_os("HOME").ok_or_else(|| {
            Error::new("HOME is unavailable; supply an authority store explicitly")
        })?;
        Self::new(PathBuf::from(home).join(".actuation/authority"))
    }

    fn path(&self, authority_source_ref: &str) -> PathBuf {
        self.root.join(format!(
            "{}.jsonl",
            sanitize_source_ref(authority_source_ref)
        ))
    }

    /// The store-directory inode is the lock locus, exactly as the stream and
    /// occupancy stores do it: no lock file enters the public store directory,
    /// and an atomic append cannot invalidate a held lock. An exclusive lock
    /// serialises read-check-append so racing issuers see a consistent state;
    /// a shared lock lets concurrent readers fold in parallel. `None` means the
    /// store does not exist yet and a reader has nothing to lock.
    fn lock(&self, exclusive: bool) -> Result<Option<File>> {
        if exclusive {
            io(fs::create_dir_all(&self.root))?;
        } else if !self.root.exists() {
            return Ok(None);
        }
        let directory = io(File::open(&self.root))?;
        #[cfg(unix)]
        {
            if exclusive {
                io(directory.lock())?;
            } else {
                io(directory.lock_shared())?;
            }
        }
        #[cfg(not(unix))]
        {
            let _ = exclusive;
            return Err(Error::new(
                "authority store locking is not implemented on this target",
            ));
        }
        Ok(Some(directory))
    }

    /// Refuse to open through a symlink, so a source file can never be
    /// redirected out of the store directory.
    fn file_options() -> OpenOptions {
        let mut options = OpenOptions::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW);
        }
        options
    }

    /// Read a source's raw events. The caller holds the appropriate lock.
    fn read_events_unlocked(&self, authority_source_ref: &str) -> Result<Vec<AuthorityEvent>> {
        let path = self.path(authority_source_ref);
        let mut file = match Self::file_options().read(true).open(&path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(Error::new(format!(
                    "could not read authority source {}: {e}",
                    authority_source_ref
                )))
            }
        };
        let mut text = String::new();
        io(file.read_to_string(&mut text))?;
        let mut events = Vec::new();
        for (index, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let event: AuthorityEvent = serde_json::from_str(line).map_err(|e| {
                Error::new(format!(
                    "authority source {} line {} is not a valid event: {e}",
                    authority_source_ref,
                    index + 1
                ))
            })?;
            events.push(event);
        }
        Ok(events)
    }

    /// Append one event durably. The caller holds the exclusive lock and
    /// passes its directory handle so a newly created source can make its own
    /// directory entry durable too.
    fn append_event_unlocked(
        &self,
        authority_source_ref: &str,
        event: &AuthorityEvent,
        directory: &File,
    ) -> Result<()> {
        let path = self.path(authority_source_ref);
        // Under the exclusive lock this is race-free: a source that does not
        // exist yet is being created by this append, so its directory entry
        // must be synced as well, not only its bytes.
        let is_new = !path.exists();
        let mut file = Self::file_options()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| {
                Error::new(format!(
                    "could not open authority source {}: {e}",
                    authority_source_ref
                ))
            })?;
        let mut line = serde_json::to_string(event)
            .map_err(|e| Error::new(format!("could not encode authority event: {e}")))?;
        line.push('\n');
        file.write_all(line.as_bytes())
            .and_then(|_| file.sync_data())
            .map_err(|e| {
                Error::new(format!(
                    "could not write authority event for {authority_source_ref}: {e}"
                ))
            })?;
        if is_new {
            io(directory.sync_all())?;
        }
        Ok(())
    }

    /// Issue one governing record. A source that already has any state is
    /// refused: issuing again is re-founding authority, not bookkeeping.
    pub fn issue(
        &self,
        authority_source_ref: &str,
        at_unix_seconds: u64,
        record: Value,
    ) -> Result<AuthorityState> {
        let directory = self
            .lock(true)?
            .expect("an exclusive lock creates the store");
        if let Some(existing) = self.read_unlocked(authority_source_ref)? {
            return Err(Error::new(format!(
                "authority source {} already exists (issued at {}, revoked: {}); issue a new source ref instead",
                authority_source_ref,
                existing.issued_at_unix_seconds,
                existing.revoked.is_some()
            )));
        }
        let event = AuthorityEvent::Issue {
            schema: AUTHORITY_EVENT_VERSION.into(),
            at_unix_seconds,
            record,
        };
        self.append_event_unlocked(authority_source_ref, &event, &directory)?;
        self.read_unlocked(authority_source_ref)?
            .ok_or_else(|| Error::new("authority record disappeared after issue"))
    }

    /// Revoke a source. Revocation is permanent; a later issue needs a new
    /// source ref, so a stale cached record can never become valid again.
    pub fn revoke(
        &self,
        authority_source_ref: &str,
        at_unix_seconds: u64,
        reason: String,
    ) -> Result<AuthorityState> {
        let directory = self
            .lock(true)?
            .expect("an exclusive lock creates the store");
        let state = self.read_unlocked(authority_source_ref)?.ok_or_else(|| {
            Error::new(format!(
                "authority source {} is unknown; nothing to revoke",
                authority_source_ref
            ))
        })?;
        if state.revoked.is_some() {
            return Err(Error::new(format!(
                "authority source {} is already revoked",
                authority_source_ref
            )));
        }
        let event = AuthorityEvent::Revoke {
            schema: AUTHORITY_EVENT_VERSION.into(),
            at_unix_seconds,
            reason,
        };
        self.append_event_unlocked(authority_source_ref, &event, &directory)?;
        self.read_unlocked(authority_source_ref)?
            .ok_or_else(|| Error::new("authority record disappeared after revoke"))
    }

    /// Fold the source's events into its current state, or `None` when the
    /// source has no events at all.
    pub fn read(&self, authority_source_ref: &str) -> Result<Option<AuthorityState>> {
        let _lock = self.lock(false)?;
        self.read_unlocked(authority_source_ref)
    }

    /// The fold itself, under a lock the caller already holds.
    fn read_unlocked(&self, authority_source_ref: &str) -> Result<Option<AuthorityState>> {
        let mut record = None;
        let mut issued_at = None;
        let mut revoked = None;
        for event in self.read_events_unlocked(authority_source_ref)? {
            match event {
                AuthorityEvent::Issue {
                    at_unix_seconds,
                    record: issued,
                    ..
                } => {
                    record = Some(issued);
                    issued_at = Some(at_unix_seconds);
                }
                AuthorityEvent::Revoke {
                    at_unix_seconds,
                    reason,
                    ..
                } => {
                    revoked = Some(AuthorityRevocation {
                        at_unix_seconds,
                        reason,
                    });
                }
            }
        }
        Ok(match (record, issued_at) {
            (Some(record), Some(issued_at_unix_seconds)) => Some(AuthorityState {
                record,
                issued_at_unix_seconds,
                revoked,
            }),
            _ => None,
        })
    }
}

/// A source ref names one file. Only the safe subset of characters survives;
/// a ref outside it still works, but several may share one file, so issue
/// and read go through the typed ref parse before they reach the store.
fn sanitize_source_ref(authority_source_ref: &str) -> String {
    authority_source_ref
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store(tag: &str) -> (tempfile::TempDir, AuthorityStore) {
        let dir = tempfile::tempdir().unwrap();
        let store =
            AuthorityStore::new(dir.path().join(tag)).expect("store root is a real directory");
        (dir, store)
    }

    #[test]
    fn issue_then_read_folds_the_record_state() {
        let (_dir, store) = temp_store("issue-read");
        let record = serde_json::json!({"holder": "human:owner"});
        let state = store
            .issue("authority-source:main", 1726300000, record.clone())
            .unwrap();
        assert_eq!(state.record, record);
        assert_eq!(state.issued_at_unix_seconds, 1726300000);
        assert!(state.revoked.is_none());
        let reread = store.read("authority-source:main").unwrap().unwrap();
        assert_eq!(reread.record, record);
    }

    #[test]
    fn a_second_issue_on_the_same_source_is_refused() {
        let (_dir, store) = temp_store("double-issue");
        store
            .issue(
                "authority-source:main",
                1726300000,
                serde_json::json!({"a": 1}),
            )
            .unwrap();
        let again = store.issue(
            "authority-source:main",
            1726300001,
            serde_json::json!({"a": 2}),
        );
        assert!(again.is_err(), "re-issuing an existing source must fail");
    }

    #[test]
    fn revoke_is_append_only_and_survives_reread() {
        let (_dir, store) = temp_store("revoke");
        store
            .issue(
                "authority-source:main",
                1726300000,
                serde_json::json!({"a": 1}),
            )
            .unwrap();
        let state = store
            .revoke(
                "authority-source:main",
                1726300005,
                "owner withdrawal".into(),
            )
            .unwrap();
        assert_eq!(state.revoked.as_ref().unwrap().reason, "owner withdrawal");
        let reread = store.read("authority-source:main").unwrap().unwrap();
        assert!(reread.revoked.is_some(), "revocation must survive a reread");
        let second = store.revoke("authority-source:main", 1726300006, "again".into());
        assert!(second.is_err(), "a revoked source cannot be revoked again");
    }

    #[test]
    fn an_unknown_source_reads_as_absent_never_as_an_error() {
        let (_dir, store) = temp_store("unknown");
        assert!(store.read("authority-source:none").unwrap().is_none());
    }

    /// A store written by the pre-lock implementation is one JSON object per
    /// line: issue, then optionally revoke, with a trailing newline. The
    /// hardening must not change that on-disk format, so bytes in that exact
    /// shape must still fold correctly.
    #[test]
    fn a_store_written_in_the_existing_format_still_loads() {
        let (_dir, store) = temp_store("legacy");
        let path = store.path("authority-source:main");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let bytes = concat!(
            r#"{"op":"issue","schema":"actuation.authority-event/v1","at_unix_seconds":1726300000,"record":{"holder":"human:owner"}}"#,
            "\n",
            r#"{"op":"revoke","schema":"actuation.authority-event/v1","at_unix_seconds":1726300005,"reason":"owner withdrawal"}"#,
            "\n",
        );
        std::fs::write(&path, bytes).unwrap();
        let state = store.read("authority-source:main").unwrap().unwrap();
        assert_eq!(state.record, serde_json::json!({"holder": "human:owner"}));
        assert_eq!(state.issued_at_unix_seconds, 1726300000);
        let revocation = state.revoked.expect("the revoke line folds in");
        assert_eq!(revocation.at_unix_seconds, 1726300005);
        assert_eq!(revocation.reason, "owner withdrawal");
    }

    /// A record this crate writes today is re-read after the hardening: format
    /// stability holds across a write-then-read on the current code path.
    #[test]
    fn a_freshly_issued_record_reads_back_unchanged() {
        let (_dir, store) = temp_store("round-trip");
        let record = serde_json::json!({"holder": "human:owner", "scope": ["world:one"]});
        store
            .issue("authority-source:main", 1726300000, record.clone())
            .unwrap();
        // Re-read through a fresh store handle over the same directory: the
        // durable bytes, not an in-memory cache, are what answers.
        let reopened = AuthorityStore::new(store.root.clone()).unwrap();
        let state = reopened.read("authority-source:main").unwrap().unwrap();
        assert_eq!(state.record, record);
        assert_eq!(state.issued_at_unix_seconds, 1726300000);
        assert!(state.revoked.is_none());
    }

    fn shared_store() -> (tempfile::TempDir, std::sync::Arc<AuthorityStore>) {
        let dir = tempfile::tempdir().unwrap();
        let store = std::sync::Arc::new(
            AuthorityStore::new(dir.path().join("authority"))
                .expect("store root is a real directory"),
        );
        (dir, store)
    }

    fn intact_lines(store: &AuthorityStore, source: &str) -> Vec<AuthorityEvent> {
        let raw = std::fs::read_to_string(store.path(source)).unwrap();
        raw.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                serde_json::from_str::<AuthorityEvent>(line)
                    .unwrap_or_else(|e| panic!("torn or interleaved line {line:?}: {e}"))
            })
            .collect()
    }

    /// N threads each issue a distinct source at once. The directory lock
    /// serialises the appends, so every source ends up with exactly one intact
    /// event line and no torn or interleaved writes.
    #[test]
    fn concurrent_issuance_of_distinct_sources_writes_every_record_intact() {
        let (_dir, store) = shared_store();
        let count = 24u64;
        let mut handles = Vec::new();
        for i in 0..count {
            let store = std::sync::Arc::clone(&store);
            handles.push(std::thread::spawn(move || {
                let source = format!("authority-source:racer-{i}");
                store
                    .issue(&source, 1726300000 + i, serde_json::json!({ "holder": i }))
                    .expect("each distinct source issues once");
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }
        for i in 0..count {
            let source = format!("authority-source:racer-{i}");
            let state = store.read(&source).unwrap().expect("source exists");
            assert_eq!(state.record, serde_json::json!({ "holder": i }));
            assert!(state.revoked.is_none());
            let lines = intact_lines(&store, &source);
            assert_eq!(lines.len(), 1, "exactly one issue line per source");
        }
    }

    /// N threads race to issue the *same* source. The lock makes read-check-
    /// append a compare-and-swap: exactly one issuance wins, the rest are
    /// refused, and a single intact record is on disk.
    #[test]
    fn concurrent_issuance_of_one_source_admits_exactly_one() {
        let (_dir, store) = shared_store();
        let count = 24u64;
        let mut handles = Vec::new();
        for i in 0..count {
            let store = std::sync::Arc::clone(&store);
            handles.push(std::thread::spawn(move || {
                store
                    .issue(
                        "authority-source:contended",
                        1726300000 + i,
                        serde_json::json!({ "racer": i }),
                    )
                    .is_ok()
            }));
        }
        let wins = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .filter(|issued| *issued)
            .count();
        assert_eq!(
            wins, 1,
            "the directory lock makes read-check-append a compare-and-swap"
        );
        let state = store.read("authority-source:contended").unwrap().unwrap();
        assert!(state.revoked.is_none());
        let lines = intact_lines(&store, "authority-source:contended");
        assert_eq!(lines.len(), 1, "only the winning issue is on disk");
    }

    /// O_NOFOLLOW must refuse to open a source file that is a symlink, so a
    /// planted link cannot redirect an issuance out of the store directory.
    #[cfg(unix)]
    #[test]
    fn a_symlinked_source_file_is_refused_not_followed() {
        let (dir, store) = temp_store("nofollow");
        std::fs::create_dir_all(&store.root).unwrap();
        let target = dir.path().join("outside-the-store.jsonl");
        std::fs::write(&target, "").unwrap();
        let link = store.path("authority-source:link");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let result = store.issue(
            "authority-source:link",
            1726300000,
            serde_json::json!({"a": 1}),
        );
        assert!(
            result.is_err(),
            "O_NOFOLLOW must refuse a symlinked source file rather than write through it"
        );
        // The link's target must not have been written through.
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "");
    }
}
