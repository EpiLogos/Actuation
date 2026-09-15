//! Owner-issued local governing-authority records.
//!
//! A record is the durable governing side of a local actualisation: one
//! governing WorldBinding, one MetagencyGrant, one holder, one scope of
//! allowed worlds. Events are append-only per authority source and
//! revocation is an event, never a rewrite. The store folds events into the
//! current state and nothing more: it does not admit, actualise or dispatch.

use actuation_core::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
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

    fn read_events(&self, authority_source_ref: &str) -> Result<Vec<AuthorityEvent>> {
        let path = self.path(authority_source_ref);
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(Error::new(format!(
                    "could not read authority source {}: {e}",
                    authority_source_ref
                )))
            }
        };
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

    fn append_event(&self, authority_source_ref: &str, event: &AuthorityEvent) -> Result<()> {
        io(std::fs::create_dir_all(&self.root))?;
        let path = self.path(authority_source_ref);
        let mut file = OpenOptions::new()
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
            .and_then(|_| file.flush())
            .map_err(|e| {
                Error::new(format!(
                    "could not write authority event for {authority_source_ref}: {e}"
                ))
            })?;
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
        if let Some(existing) = self.read(authority_source_ref)? {
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
        self.append_event(authority_source_ref, &event)?;
        self.read(authority_source_ref)?
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
        let state = self.read(authority_source_ref)?.ok_or_else(|| {
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
        self.append_event(authority_source_ref, &event)?;
        self.read(authority_source_ref)?
            .ok_or_else(|| Error::new("authority record disappeared after revoke"))
    }

    /// Fold the source's events into its current state, or `None` when the
    /// source has no events at all.
    pub fn read(&self, authority_source_ref: &str) -> Result<Option<AuthorityState>> {
        let mut record = None;
        let mut issued_at = None;
        let mut revoked = None;
        for event in self.read_events(authority_source_ref)? {
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
}
