//! Owner-issued local governing-authority records.
//!
//! A record is the durable governing side of a local actualisation: one
//! governing WorldBinding, one MetagencyGrant, one holder, one scope of
//! allowed worlds. Events are append-only per authority source and
//! revocation is an event, never a rewrite. The store folds events into the
//! current state and nothing more: it does not admit, actualise or dispatch.
//!
//! A source is the literal `authority_source_ref` its Issue record names.
//! Each source has its own history at `sources-v2/<hex>/events.jsonl`, where
//! `<hex>` is every byte of the ref, so no two refs can share a file. A store
//! written before V2 kept one `<sanitised ref>.jsonl` per source at the root;
//! those files are read and revoked where they lie and never moved, and a ref
//! that maps to one without being the ref its record names is another source.
//! The layout is documented in `docs/LOCAL-AUTHORITY-STORAGE.md`.
//!
//! Persistence follows the same discipline as the stream and occupancy stores:
//! every read-check-append runs under a lock on the store directory (exclusive
//! to mutate, shared to read), every path component is opened relative to a
//! held directory without following symlinks, and every append is
//! `sync_data`'d — with the parent directory synced when an entry is first
//! created. Two racing issuers therefore serialise, so issuance stays a
//! compare-and-swap and a crash cannot leave a torn line.

use actuation_core::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

fn io<T>(result: std::io::Result<T>) -> Result<T> {
    result.map_err(|e| Error::new(format!("authority store I/O failure: {e}")))
}

pub const AUTHORITY_EVENT_VERSION: &str = "actuation.authority-event/v1";
pub const AUTHORITY_LISTING_VERSION: &str = "actuation.local-authority-listing/v1";

const V2_DIRECTORY: &str = "sources-v2";
const V2_EVENTS: &str = "events.jsonl";
const LEGACY_SUFFIX: &str = ".jsonl";
/// Hex characters per V2 directory name: 90 bytes of the ref, inside every
/// filesystem's 255-byte name limit.
const V2_COMPONENT: usize = 180;

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

/// One issued source in a listing: the source ref its own record names, and
/// the folded state exactly as `read` gives it.
#[derive(Debug, Clone, Serialize)]
pub struct ListedAuthority {
    pub authority_source_ref: String,
    #[serde(flatten)]
    pub state: AuthorityState,
}

/// A source file that cannot be folded, or whose record does not name the
/// source its file holds. Listed with the reason, never dropped.
/// `authority_source_ref` is the source the file is attributed to when one is
/// known; for a pre-V2 file filed under another source's key, the one its
/// record names.
#[derive(Debug, Clone, Serialize)]
pub struct InvalidAuthoritySource {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority_source_ref: Option<String>,
    pub error: String,
}

/// `actuation.local-authority-listing/v1`: every source in the store,
/// uncapped, revoked ones included (a revocation is part of the record).
#[derive(Debug, Clone, Serialize)]
pub struct AuthorityListing {
    pub schema: &'static str,
    pub store: String,
    pub authorities: Vec<ListedAuthority>,
    pub invalid: Vec<InvalidAuthoritySource>,
}

/// The history found where a source lives names a different source. The
/// store's error carries this as its typed source, so a caller can refuse it
/// as an identity mismatch rather than as an unreadable store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceIdentityMismatch {
    pub requested: String,
    pub stored: String,
}

impl std::fmt::Display for SourceIdentityMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the history stored for authority source {} names {}",
            self.requested, self.stored
        )
    }
}

impl std::error::Error for SourceIdentityMismatch {}

fn mismatch(requested: &str, stored: &str) -> Error {
    let mismatch = SourceIdentityMismatch {
        requested: requested.to_owned(),
        stored: stored.to_owned(),
    };
    Error::new(mismatch.to_string()).with_source(mismatch)
}

/// Where a source's history lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layout {
    V2,
    Legacy,
}

struct Found {
    state: AuthorityState,
    layout: Layout,
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

    /// The V2 history file of a literal source, for display. Every operation
    /// reaches it through held directory descriptors, never by this path.
    fn v2_path(&self, authority_source_ref: &str) -> PathBuf {
        let mut path = self.root.join(V2_DIRECTORY);
        path.extend(v2_components(authority_source_ref));
        path.join(V2_EVENTS)
    }

    /// The store-directory inode is the lock locus, exactly as the stream and
    /// occupancy stores do it: no lock file enters the public store directory,
    /// and an atomic append cannot invalidate a held lock. An exclusive lock
    /// serialises read-check-append so racing issuers see a consistent state;
    /// a shared lock lets concurrent readers fold in parallel. `None` means the
    /// store does not exist yet and a reader has nothing to lock. The handle
    /// returned is the root every path below is opened from.
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

    /// The directory holding a source's V2 history, opened component by
    /// component from the locked root. `create` makes missing components,
    /// syncing each parent so the new entry is durable before anything is
    /// published beneath it; without it a missing component means the source
    /// has no V2 history.
    fn v2_directory(root: &File, authority_source_ref: &str, create: bool) -> Result<Option<File>> {
        let mut current: Option<File> = None;
        for name in
            std::iter::once(V2_DIRECTORY.to_owned()).chain(v2_components(authority_source_ref))
        {
            let parent = current.as_ref().unwrap_or(root);
            let next = match fd::open_directory(parent, &name) {
                Ok(next) => next,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound && create => {
                    match fd::make_directory(parent, &name) {
                        Ok(()) => io(parent.sync_all())?,
                        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                        Err(e) => return io(Err(e)),
                    }
                    io(fd::open_directory(parent, &name))?
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(e) => {
                    return Err(Error::new(format!(
                        "could not open the V2 directory of authority source {authority_source_ref}: {e}"
                    )))
                }
            };
            current = Some(next);
        }
        Ok(current)
    }

    /// The raw bytes of one history file, or `None` when it does not exist.
    fn read_at(directory: &File, name: &str, shown: &str) -> Result<Option<String>> {
        let mut file = match fd::open_file(directory, name, fd::Access::Read) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => {
                return Err(Error::new(format!(
                    "could not read authority history {shown}: {e}"
                )))
            }
        };
        let mut text = String::new();
        io(file.read_to_string(&mut text))?;
        Ok(Some(text))
    }

    /// Fold one history into its state, checking it as it goes: one Issue
    /// first, naming its source; at most one Revoke after it; the v1 event
    /// schema throughout; every line complete. A history with no events is
    /// `None`; anything else that breaks a rule refuses, and nothing is
    /// folded around it.
    fn fold(text: &str) -> Result<Option<AuthorityState>> {
        if !text.is_empty() && !text.ends_with('\n') {
            return Err(Error::new("the history's final line is incomplete"));
        }
        let mut state: Option<AuthorityState> = None;
        for (index, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let line_number = index + 1;
            let event: AuthorityEvent = serde_json::from_str(line)
                .map_err(|e| Error::new(format!("line {line_number} is not a valid event: {e}")))?;
            let (AuthorityEvent::Issue { schema, .. } | AuthorityEvent::Revoke { schema, .. }) =
                &event;
            if schema != AUTHORITY_EVENT_VERSION {
                return Err(Error::new(format!(
                    "line {line_number} has event schema {schema}, not {AUTHORITY_EVENT_VERSION}"
                )));
            }
            match event {
                AuthorityEvent::Issue {
                    at_unix_seconds,
                    record,
                    ..
                } => {
                    if state.is_some() {
                        return Err(Error::new(format!(
                            "line {line_number} is a second Issue; a source is issued once"
                        )));
                    }
                    if record_source(&record).is_none() {
                        return Err(Error::new("the Issue record names no authority_source_ref"));
                    }
                    state = Some(AuthorityState {
                        record,
                        issued_at_unix_seconds: at_unix_seconds,
                        revoked: None,
                    });
                }
                AuthorityEvent::Revoke {
                    at_unix_seconds,
                    reason,
                    ..
                } => {
                    let issued = state.as_mut().ok_or_else(|| {
                        Error::new(format!("line {line_number} revokes before any Issue"))
                    })?;
                    if issued.revoked.is_some() {
                        return Err(Error::new(format!(
                            "line {line_number} is a second Revoke; revocation is permanent"
                        )));
                    }
                    issued.revoked = Some(AuthorityRevocation {
                        at_unix_seconds,
                        reason,
                    });
                }
            }
        }
        Ok(state)
    }

    /// Find a source's history in either layout, under a lock the caller
    /// holds. The V2 history must name the source; the pre-V2 file under its
    /// key belongs to it only if its record names it, belongs to another
    /// source if that source shares the key, and otherwise is misfiled. Two
    /// histories for one source are a conflict: both are kept, neither folds.
    fn find_unlocked(&self, root: &File, authority_source_ref: &str) -> Result<Option<Found>> {
        let within = |e: Error| Error::new(format!("authority source {authority_source_ref}: {e}"));
        let v2 = match Self::v2_directory(root, authority_source_ref, false)? {
            Some(directory) => {
                let shown = self.v2_path(authority_source_ref).display().to_string();
                match Self::read_at(&directory, V2_EVENTS, &shown)? {
                    Some(text) => Self::fold(&text).map_err(within)?,
                    None => None,
                }
            }
            None => None,
        };
        if let Some(state) = &v2 {
            let named = record_source(&state.record).unwrap_or_default();
            if named != authority_source_ref {
                return Err(mismatch(authority_source_ref, named));
            }
        }
        let legacy_name = legacy_file_name(authority_source_ref);
        let shown = self.root.join(&legacy_name).display().to_string();
        let legacy = match Self::read_at(root, &legacy_name, &shown) {
            Ok(Some(text)) => match Self::fold(&text).map_err(within)? {
                Some(state) => {
                    let named = record_source(&state.record).unwrap_or_default();
                    if named == authority_source_ref {
                        Some(state)
                    } else if legacy_key(named) == legacy_key(authority_source_ref) {
                        None
                    } else {
                        return Err(mismatch(authority_source_ref, named));
                    }
                }
                None => None,
            },
            Ok(None) => None,
            // A key longer than a filename was never writable before V2, so
            // there is no pre-V2 file to find; the V2 layout has no such limit.
            Err(_) if legacy_name.len() > 255 => None,
            Err(e) => return Err(e),
        };
        match (v2, legacy) {
            (Some(_), Some(_)) => Err(within(Error::new(format!(
                "two histories exist, {} and {shown}; both are kept and neither is folded",
                self.v2_path(authority_source_ref).display()
            )))),
            (Some(state), None) => Ok(Some(Found {
                state,
                layout: Layout::V2,
            })),
            (None, Some(state)) => Ok(Some(Found {
                state,
                layout: Layout::Legacy,
            })),
            (None, None) => Ok(None),
        }
    }

    /// Append one event durably to the source's history in `layout`. The
    /// caller holds the exclusive lock and passes the locked root.
    fn append_unlocked(
        &self,
        root: &File,
        authority_source_ref: &str,
        layout: Layout,
        event: &AuthorityEvent,
    ) -> Result<()> {
        let owned;
        let (directory, name) = match layout {
            Layout::V2 => {
                owned = Self::v2_directory(root, authority_source_ref, true)?
                    .ok_or_else(|| Error::new("the V2 directory was not created"))?;
                (&owned, V2_EVENTS.to_owned())
            }
            Layout::Legacy => (root, legacy_file_name(authority_source_ref)),
        };
        let cannot_open = |e: std::io::Error| {
            Error::new(format!(
                "could not open authority source {authority_source_ref}: {e}"
            ))
        };
        // A V2 history is created by its first append, and under the
        // exclusive lock that is race-free, so its new directory entry is
        // synced as well as its bytes. A pre-V2 file is only ever appended to.
        let created = match layout {
            Layout::V2 => match fd::open_file(directory, &name, fd::Access::CreateNew) {
                Ok(file) => Some(file),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => None,
                Err(e) => return Err(cannot_open(e)),
            },
            Layout::Legacy => None,
        };
        let is_new = created.is_some();
        let mut file = match created {
            Some(file) => file,
            None => fd::open_file(directory, &name, fd::Access::Append).map_err(cannot_open)?,
        };
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

    /// Issue one governing record. The record must name the source it is
    /// issued under. A source that already has any state, in either layout,
    /// is refused: issuing again is re-founding authority, not bookkeeping.
    pub fn issue(
        &self,
        authority_source_ref: &str,
        at_unix_seconds: u64,
        record: Value,
    ) -> Result<AuthorityState> {
        if authority_source_ref.is_empty() {
            return Err(Error::new("an authority source ref must be non-empty"));
        }
        match record_source(&record) {
            Some(named) if named == authority_source_ref => {}
            named => {
                return Err(Error::new(format!(
                    "the record names authority source {}, not {authority_source_ref} it is issued under",
                    named.unwrap_or("none")
                )))
            }
        }
        let root = self
            .lock(true)?
            .expect("an exclusive lock creates the store");
        if let Some(existing) = self.find_unlocked(&root, authority_source_ref)? {
            return Err(Error::new(format!(
                "authority source {} already exists (issued at {}, revoked: {}); issue a new source ref instead",
                authority_source_ref,
                existing.state.issued_at_unix_seconds,
                existing.state.revoked.is_some()
            )));
        }
        let event = AuthorityEvent::Issue {
            schema: AUTHORITY_EVENT_VERSION.into(),
            at_unix_seconds,
            record,
        };
        self.append_unlocked(&root, authority_source_ref, Layout::V2, &event)?;
        self.find_unlocked(&root, authority_source_ref)?
            .map(|found| found.state)
            .ok_or_else(|| Error::new("authority record disappeared after issue"))
    }

    /// Revoke a source, appending to its history wherever it lies. Revocation
    /// is permanent; a later issue needs a new source ref, so a stale cached
    /// record can never become valid again.
    pub fn revoke(
        &self,
        authority_source_ref: &str,
        at_unix_seconds: u64,
        reason: String,
    ) -> Result<AuthorityState> {
        let root = self
            .lock(true)?
            .expect("an exclusive lock creates the store");
        let found = self
            .find_unlocked(&root, authority_source_ref)?
            .ok_or_else(|| {
                Error::new(format!(
                    "authority source {} is unknown; nothing to revoke",
                    authority_source_ref
                ))
            })?;
        if found.state.revoked.is_some() {
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
        self.append_unlocked(&root, authority_source_ref, found.layout, &event)?;
        self.find_unlocked(&root, authority_source_ref)?
            .map(|found| found.state)
            .ok_or_else(|| Error::new("authority record disappeared after revoke"))
    }

    /// Fold the source's events into its current state, or `None` when the
    /// source has no history at all.
    pub fn read(&self, authority_source_ref: &str) -> Result<Option<AuthorityState>> {
        let Some(root) = self.lock(false)? else {
            return Ok(None);
        };
        Ok(self
            .find_unlocked(&root, authority_source_ref)?
            .map(|found| found.state))
    }

    /// Every source in the store, both layouts, uncapped, in source-ref
    /// order. Each source folds exactly as `read` folds it; a file that cannot
    /// be folded, holds no issue, is not where its record's source lives, or
    /// is one of two histories for one source is listed under `invalid` with
    /// its reason. An absent store lists nothing.
    pub fn list(&self) -> Result<AuthorityListing> {
        let mut listing = AuthorityListing {
            schema: AUTHORITY_LISTING_VERSION,
            store: self.root.display().to_string(),
            authorities: Vec::new(),
            invalid: Vec::new(),
        };
        let Some(root) = self.lock(false)? else {
            return Ok(listing);
        };
        // Each source found, with the files that hold it.
        let mut sources: BTreeMap<String, Vec<String>> = BTreeMap::new();

        let mut names = Vec::new();
        for entry in io(fs::read_dir(&self.root))? {
            let name = io(entry)?.file_name().to_string_lossy().into_owned();
            if name.ends_with(LEGACY_SUFFIX) {
                names.push(name);
            }
        }
        names.sort();
        for name in names {
            let shown = self.root.join(&name).display().to_string();
            let key = &name[..name.len() - LEGACY_SUFFIX.len()];
            let state = match Self::read_at(&root, &name, &shown)
                .and_then(|text| Self::fold(&text.unwrap_or_default()))
            {
                Ok(Some(state)) => state,
                Ok(None) => {
                    listing.invalid.push(InvalidAuthoritySource {
                        path: shown,
                        authority_source_ref: None,
                        error: "the source file holds no issue, so it names no authority".into(),
                    });
                    continue;
                }
                Err(error) => {
                    listing.invalid.push(InvalidAuthoritySource {
                        path: shown,
                        authority_source_ref: None,
                        error: error.to_string(),
                    });
                    continue;
                }
            };
            let named = record_source(&state.record).unwrap_or_default().to_owned();
            if legacy_key(&named) == key {
                sources.entry(named).or_default().push(shown);
            } else {
                listing.invalid.push(InvalidAuthoritySource {
                    path: shown,
                    authority_source_ref: Some(named),
                    error: "the record names a source that is not the one its file holds".into(),
                });
            }
        }

        match fd::open_directory(&root, V2_DIRECTORY) {
            Ok(directory) => self.walk_v2(
                directory,
                &mut Vec::new(),
                &mut sources,
                &mut listing.invalid,
            ),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => listing.invalid.push(InvalidAuthoritySource {
                path: self.root.join(V2_DIRECTORY).display().to_string(),
                authority_source_ref: None,
                error: format!("the V2 directory cannot be opened: {e}"),
            }),
        }

        for (source, paths) in sources {
            let error = match self.find_unlocked(&root, &source) {
                Ok(Some(found)) => {
                    listing.authorities.push(ListedAuthority {
                        authority_source_ref: source,
                        state: found.state,
                    });
                    continue;
                }
                Ok(None) => "the source file holds no issue, so it names no authority".to_owned(),
                Err(error) => error.to_string(),
            };
            for path in paths {
                listing.invalid.push(InvalidAuthoritySource {
                    path,
                    authority_source_ref: Some(source.clone()),
                    error: error.clone(),
                });
            }
        }
        Ok(listing)
    }

    /// Collect every V2 history beneath `directory`, whose path from
    /// `sources-v2/` is `components`. A history whose path is not the
    /// canonical encoding of a ref is invalid; an entry that cannot be opened
    /// without following a link is invalid; any other stray file is ignored.
    fn walk_v2(
        &self,
        directory: File,
        components: &mut Vec<String>,
        sources: &mut BTreeMap<String, Vec<String>>,
        invalid: &mut Vec<InvalidAuthoritySource>,
    ) {
        let shown = |components: &[String], name: &str| {
            let mut path = self.root.join(V2_DIRECTORY);
            path.extend(components);
            path.join(name).display().to_string()
        };
        let names = match fd::entry_names(&directory) {
            Ok(names) => names,
            Err(e) => {
                invalid.push(InvalidAuthoritySource {
                    path: shown(components, ""),
                    authority_source_ref: None,
                    error: format!("the V2 directory cannot be read: {e}"),
                });
                return;
            }
        };
        for name in names {
            if name == V2_EVENTS {
                match decode_v2(components) {
                    Some(source) => sources
                        .entry(source)
                        .or_default()
                        .push(shown(components, &name)),
                    None => invalid.push(InvalidAuthoritySource {
                        path: shown(components, &name),
                        authority_source_ref: None,
                        error: "the V2 path is not the encoding of a source ref".into(),
                    }),
                }
                continue;
            }
            match fd::open_directory(&directory, &name) {
                Ok(child) => {
                    components.push(name);
                    self.walk_v2(child, components, sources, invalid);
                    components.pop();
                }
                Err(e) if e.raw_os_error() == Some(fd::NOT_A_DIRECTORY) => {}
                Err(e) => invalid.push(InvalidAuthoritySource {
                    path: shown(components, &name),
                    authority_source_ref: None,
                    error: format!("refused, not opened without following links: {e}"),
                }),
            }
        }
    }
}

/// The `authority_source_ref` an Issue record names.
fn record_source(record: &Value) -> Option<&str> {
    record.get("authority_source_ref").and_then(Value::as_str)
}

/// A ref's V2 directory names: its UTF-8 bytes as lowercase hex, in pieces of
/// at most `V2_COMPONENT` characters. Lowercase hex survives case-insensitive
/// filesystems, which the pre-V2 key did not.
fn v2_components(authority_source_ref: &str) -> Vec<String> {
    let hex: String = authority_source_ref
        .bytes()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    hex.as_bytes()
        .chunks(V2_COMPONENT)
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect()
}

/// The ref a V2 path encodes, when it is exactly that ref's canonical path.
fn decode_v2(components: &[String]) -> Option<String> {
    let hex = components.concat();
    if hex.len() % 2 != 0 {
        return None;
    }
    let bytes = (0..hex.len())
        .step_by(2)
        .map(|at| {
            let pair = hex.get(at..at + 2)?;
            if pair
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                u8::from_str_radix(pair, 16).ok()
            } else {
                None
            }
        })
        .collect::<Option<Vec<u8>>>()?;
    let source = String::from_utf8(bytes).ok()?;
    (v2_components(&source) == components).then_some(source)
}

/// The pre-V2 file key. Only the safe subset of characters survived it, so
/// several refs share one key; V2 never writes under it, and a file under it
/// belongs to the ref its own record names.
fn legacy_key(authority_source_ref: &str) -> String {
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

fn legacy_file_name(authority_source_ref: &str) -> String {
    format!("{}{LEGACY_SUFFIX}", legacy_key(authority_source_ref))
}

/// Descriptor-relative opens: each name is one component opened from a held
/// directory, never following a symlink, so neither a planted link nor a
/// path longer than the system's whole-path limit can redirect or refuse a
/// source.
#[cfg(unix)]
mod fd {
    use rustix::fs::{self, Mode, OFlags};
    use std::fs::File;
    use std::io::Result;

    pub const NOT_A_DIRECTORY: i32 = libc::ENOTDIR;

    #[derive(Clone, Copy)]
    pub enum Access {
        Read,
        Append,
        CreateNew,
    }

    fn open(directory: &File, name: &str, flags: OFlags) -> Result<File> {
        fs::openat(
            directory,
            name,
            flags | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::RUSR | Mode::WUSR,
        )
        .map(File::from)
        .map_err(Into::into)
    }

    pub fn open_directory(directory: &File, name: &str) -> Result<File> {
        open(directory, name, OFlags::RDONLY | OFlags::DIRECTORY)
    }

    pub fn open_file(directory: &File, name: &str, access: Access) -> Result<File> {
        let flags = match access {
            Access::Read => OFlags::RDONLY,
            Access::Append => OFlags::WRONLY | OFlags::APPEND,
            Access::CreateNew => OFlags::WRONLY | OFlags::APPEND | OFlags::CREATE | OFlags::EXCL,
        };
        open(directory, name, flags)
    }

    pub fn make_directory(directory: &File, name: &str) -> Result<()> {
        fs::mkdirat(directory, name, Mode::RWXU).map_err(Into::into)
    }

    /// The entry names of a directory, sorted, without `.` and `..`.
    pub fn entry_names(directory: &File) -> Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in fs::Dir::read_from(directory)? {
            let name = entry?.file_name().to_string_lossy().into_owned();
            if name != "." && name != ".." {
                names.push(name);
            }
        }
        names.sort();
        Ok(names)
    }
}

#[cfg(not(unix))]
mod fd {
    use std::fs::File;
    use std::io::{Error, Result};

    pub const NOT_A_DIRECTORY: i32 = 0;

    #[derive(Clone, Copy)]
    pub enum Access {
        Read,
        Append,
        CreateNew,
    }

    fn unsupported<T>() -> Result<T> {
        Err(Error::other(
            "descriptor-relative authority storage is not implemented on this target",
        ))
    }

    pub fn open_directory(_: &File, _: &str) -> Result<File> {
        unsupported()
    }

    pub fn open_file(_: &File, _: &str, _: Access) -> Result<File> {
        unsupported()
    }

    pub fn make_directory(_: &File, _: &str) -> Result<()> {
        unsupported()
    }

    pub fn entry_names(_: &File) -> Result<Vec<String>> {
        unsupported()
    }
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
        let record = serde_json::json!({"authority_source_ref": "authority-source:main", "holder": "human:owner"});
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
                serde_json::json!({"authority_source_ref": "authority-source:main", "a": 1}),
            )
            .unwrap();
        let again = store.issue(
            "authority-source:main",
            1726300001,
            serde_json::json!({"authority_source_ref": "authority-source:main", "a": 2}),
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
                serde_json::json!({"authority_source_ref": "authority-source:main", "a": 1}),
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

    /// A store written before V2 is one JSON object per line in
    /// `<sanitised ref>.jsonl` at the root: issue, then optionally revoke,
    /// with a trailing newline, the record naming its source as the CLI
    /// always wrote it. Bytes in that exact shape must still fold, in place.
    #[test]
    fn a_store_written_in_the_existing_format_still_loads() {
        let (_dir, store) = temp_store("legacy");
        let path = store.root.join("authority-source_main.jsonl");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let bytes = concat!(
            r#"{"op":"issue","schema":"actuation.authority-event/v1","at_unix_seconds":1726300000,"record":{"authority_source_ref":"authority-source:main","holder":"human:owner"}}"#,
            "\n",
            r#"{"op":"revoke","schema":"actuation.authority-event/v1","at_unix_seconds":1726300005,"reason":"owner withdrawal"}"#,
            "\n",
        );
        std::fs::write(&path, bytes).unwrap();
        let state = store.read("authority-source:main").unwrap().unwrap();
        assert_eq!(
            state.record,
            serde_json::json!({"authority_source_ref": "authority-source:main", "holder": "human:owner"})
        );
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
        let record = serde_json::json!({"authority_source_ref": "authority-source:main", "holder": "human:owner", "scope": ["world:one"]});
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
        let raw = std::fs::read_to_string(store.v2_path(source)).unwrap();
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
                    .issue(
                        &source,
                        1726300000 + i,
                        serde_json::json!({ "authority_source_ref": source, "holder": i }),
                    )
                    .expect("each distinct source issues once");
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }
        for i in 0..count {
            let source = format!("authority-source:racer-{i}");
            let state = store.read(&source).unwrap().expect("source exists");
            assert_eq!(
                state.record,
                serde_json::json!({ "authority_source_ref": source, "holder": i })
            );
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
                        serde_json::json!({ "authority_source_ref": "authority-source:contended", "racer": i }),
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

    #[test]
    fn list_folds_every_source_revoked_ones_included() {
        let (_dir, store) = temp_store("list");
        assert!(
            store.list().unwrap().authorities.is_empty(),
            "an absent store lists nothing"
        );
        store
            .issue(
                "authority-source:b",
                1726300001,
                serde_json::json!({"authority_source_ref": "authority-source:b", "holder": "human:owner"}),
            )
            .unwrap();
        store
            .issue(
                "authority-source:a",
                1726300000,
                serde_json::json!({"authority_source_ref": "authority-source:a", "holder": "human:owner"}),
            )
            .unwrap();
        store
            .revoke("authority-source:b", 1726300009, "owner withdrawal".into())
            .unwrap();
        let listing = store.list().unwrap();
        assert_eq!(listing.schema, AUTHORITY_LISTING_VERSION);
        assert!(listing.invalid.is_empty(), "{:?}", listing.invalid);
        let refs: Vec<&str> = listing
            .authorities
            .iter()
            .map(|listed| listed.authority_source_ref.as_str())
            .collect();
        assert_eq!(refs, ["authority-source:a", "authority-source:b"]);
        assert!(listing.authorities[0].state.revoked.is_none());
        let revoked = listing.authorities[1].state.revoked.as_ref().unwrap();
        assert_eq!(revoked.reason, "owner withdrawal");
        // Each listed state is exactly what `read` folds.
        let read = store.read("authority-source:b").unwrap().unwrap();
        assert_eq!(
            serde_json::to_value(&listing.authorities[1].state).unwrap(),
            serde_json::to_value(&read).unwrap()
        );
    }

    #[test]
    fn list_names_a_source_it_cannot_fold_instead_of_dropping_it() {
        let (_dir, store) = temp_store("list-invalid");
        store
            .issue(
                "authority-source:good",
                1726300000,
                serde_json::json!({"authority_source_ref": "authority-source:good"}),
            )
            .unwrap();
        std::fs::write(store.root.join("torn.jsonl"), "{not json\n").unwrap();
        // A pre-V2 record whose own ref is not the source its file holds.
        std::fs::write(
            store.root.join("authority-source_here.jsonl"),
            history("authority-source:elsewhere", false),
        )
        .unwrap();
        std::fs::write(store.root.join("notes.txt"), "not a source").unwrap();
        let listing = store.list().unwrap();
        assert_eq!(listing.authorities.len(), 1);
        assert_eq!(
            listing.authorities[0].authority_source_ref,
            "authority-source:good"
        );
        assert_eq!(listing.invalid.len(), 2, "{:?}", listing.invalid);
        let named: Vec<Option<&str>> = listing
            .invalid
            .iter()
            .map(|invalid| invalid.authority_source_ref.as_deref())
            .collect();
        assert!(named.contains(&Some("authority-source:elsewhere")));
        assert!(named.contains(&None));
    }

    /// O_NOFOLLOW must refuse to open a source file that is a symlink, in
    /// either layout, so a planted link cannot redirect an issuance out of
    /// the store directory.
    #[cfg(unix)]
    #[test]
    fn a_symlinked_source_file_is_refused_not_followed() {
        let (dir, store) = temp_store("nofollow");
        let target = dir.path().join("outside-the-store.jsonl");
        std::fs::write(&target, "").unwrap();
        let v2_link = store.v2_path("authority-source:link");
        std::fs::create_dir_all(v2_link.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&target, &v2_link).unwrap();
        let legacy_link = store.root.join("authority-source_old.jsonl");
        std::os::unix::fs::symlink(&target, &legacy_link).unwrap();
        for source in ["authority-source:link", "authority-source:old"] {
            let result = store.issue(source, 1726300000, record_for(source));
            assert!(
                result.is_err(),
                "O_NOFOLLOW must refuse a symlinked source file rather than write through it"
            );
        }
        // The link's target must not have been written through.
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "");
    }

    // The layouts are spelled out here, not taken from the store, so these
    // tests name the on-disk contract itself.

    /// A file the pre-V2 binary wrote: the sanitised key at the store root.
    fn legacy_file(root: &std::path::Path, key: &str) -> PathBuf {
        root.join(format!("{key}.jsonl"))
    }

    /// The V2 file of a literal source: its UTF-8 bytes as lowercase hex,
    /// in components of at most 180 characters, under `sources-v2/`.
    fn v2_file(root: &std::path::Path, source: &str) -> PathBuf {
        let hex: String = source.bytes().map(|byte| format!("{byte:02x}")).collect();
        let mut path = root.join("sources-v2");
        for chunk in hex.as_bytes().chunks(180) {
            path.push(std::str::from_utf8(chunk).unwrap());
        }
        path.join("events.jsonl")
    }

    /// The record shape the CLI issues: it always names its own source.
    fn record_for(source: &str) -> Value {
        serde_json::json!({
            "authority_source_ref": source,
            "holder": "human:owner",
            "retained_unknown": {"native": "metadata"}
        })
    }

    /// A history byte-for-byte as the pre-V2 binary writes it.
    fn history(source: &str, revoked: bool) -> String {
        let mut bytes = serde_json::to_string(&AuthorityEvent::Issue {
            schema: AUTHORITY_EVENT_VERSION.into(),
            at_unix_seconds: 10,
            record: record_for(source),
        })
        .unwrap()
            + "\n";
        if revoked {
            bytes += &serde_json::to_string(&AuthorityEvent::Revoke {
                schema: AUTHORITY_EVENT_VERSION.into(),
                at_unix_seconds: 15,
                reason: "owner withdrawal".into(),
            })
            .unwrap();
            bytes += "\n";
        }
        bytes
    }

    fn put(path: &std::path::Path, bytes: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    const COLON: &str = "authority-source:identity";
    const SLASH: &str = "authority-source/identity";
    const UNDERSCORE: &str = "authority-source_identity";

    /// Three literal refs that the pre-V2 key folds into one file are three
    /// authorities: each issues, reads and revokes as itself.
    #[test]
    fn refs_that_share_a_legacy_key_are_each_kept() {
        let (_dir, store) = temp_store("colliding");
        for source in [COLON, SLASH, UNDERSCORE] {
            store.issue(source, 10, record_for(source)).unwrap();
        }
        store.revoke(SLASH, 20, "slash only".into()).unwrap();
        for source in [COLON, SLASH, UNDERSCORE] {
            let state = store.read(source).unwrap().unwrap();
            assert_eq!(state.record, record_for(source));
            assert_eq!(state.revoked.is_some(), source == SLASH, "{source}");
        }
        assert!(!legacy_file(&store.root, UNDERSCORE).exists());
    }

    /// A store the pre-V2 binary wrote reads in place, under the literal ref
    /// its record names and no other; its revocation appends in place, and a
    /// ref that shares its key issues beside it.
    #[test]
    fn a_legacy_store_reads_in_place_under_its_own_ref_only() {
        let (_dir, store) = temp_store("legacy-in-place");
        let legacy = legacy_file(&store.root, UNDERSCORE);
        let bytes = history(COLON, false);
        put(&legacy, &bytes);

        assert_eq!(
            store.read(COLON).unwrap().unwrap().record,
            record_for(COLON)
        );
        assert!(
            store.read(SLASH).unwrap().is_none(),
            "an alias reads nothing"
        );
        assert!(store.revoke(SLASH, 20, "alias".into()).is_err());
        assert_eq!(std::fs::read_to_string(&legacy).unwrap(), bytes);

        store.revoke(COLON, 20, "owner withdrawal".into()).unwrap();
        let revoked = std::fs::read_to_string(&legacy).unwrap();
        assert!(revoked.starts_with(&bytes) && revoked.lines().count() == 2);
        assert!(!v2_file(&store.root, COLON).exists(), "read in place");

        store.issue(SLASH, 30, record_for(SLASH)).unwrap();
        assert!(store.read(SLASH).unwrap().unwrap().revoked.is_none());
        assert!(store.read(COLON).unwrap().unwrap().revoked.is_some());
        assert_eq!(std::fs::read_to_string(&legacy).unwrap(), revoked);
    }

    /// `list` walks the legacy files and the V2 tree, and each listed state
    /// is exactly what `read` folds.
    #[test]
    fn list_walks_the_legacy_files_and_the_v2_tree() {
        let (_dir, store) = temp_store("list-layouts");
        put(&legacy_file(&store.root, UNDERSCORE), &history(COLON, true));
        store.issue(SLASH, 30, record_for(SLASH)).unwrap();
        let long = format!("authority-source:{}", "long".repeat(60));
        store.issue(&long, 40, record_for(&long)).unwrap();
        let listing = store.list().unwrap();
        assert!(listing.invalid.is_empty(), "{:?}", listing.invalid);
        let refs: Vec<&str> = listing
            .authorities
            .iter()
            .map(|listed| listed.authority_source_ref.as_str())
            .collect();
        assert_eq!(refs, [SLASH, COLON, long.as_str()]);
        for listed in &listing.authorities {
            let read = store.read(&listed.authority_source_ref).unwrap().unwrap();
            assert_eq!(
                serde_json::to_value(&listed.state).unwrap(),
                serde_json::to_value(&read).unwrap()
            );
        }
    }

    /// A history found where one source lives but naming another is refused,
    /// not admitted under the requested ref, and is never rewritten.
    #[test]
    fn a_history_naming_another_source_is_refused() {
        let (_dir, store) = temp_store("identity");
        let elsewhere = "authority-source:elsewhere";
        assert!(store.issue(COLON, 10, record_for(elsewhere)).is_err());

        // Pre-V2 store API misuse: issued under one ref, naming another.
        let misfiled = legacy_file(&store.root, UNDERSCORE);
        put(&misfiled, &history(elsewhere, false));
        let error = store.read(COLON).unwrap_err();
        assert_eq!(
            std::error::Error::source(&error)
                .and_then(|source| source.downcast_ref::<SourceIdentityMismatch>()),
            Some(&SourceIdentityMismatch {
                requested: COLON.into(),
                stored: elsewhere.into(),
            }),
            "the refusal carries its typed cause"
        );
        assert!(store.revoke(COLON, 20, "must not append".into()).is_err());
        assert_eq!(
            std::fs::read_to_string(&misfiled).unwrap(),
            history(elsewhere, false)
        );

        // A V2 file whose bytes name another source.
        let tampered = v2_file(&store.root, "authority-source:second");
        put(&tampered, &history(elsewhere, false));
        assert!(store.read("authority-source:second").is_err());
        assert_eq!(
            std::fs::read_to_string(&tampered).unwrap(),
            history(elsewhere, false)
        );
    }

    /// Unattributable, duplicated, torn, out-of-order or unknown-schema
    /// histories refuse without rewrite; nothing is folded around them.
    #[test]
    fn a_malformed_history_refuses_without_rewrite() {
        let valid = history(COLON, false);
        let revoke = concat!(
            r#"{"op":"revoke","schema":"actuation.authority-event/v1","at_unix_seconds":15,"reason":"x"}"#,
            "\n"
        );
        let cases: [String; 6] = [
            concat!(
                r#"{"op":"issue","schema":"actuation.authority-event/v1","at_unix_seconds":10,"record":{"holder":"human:owner"}}"#,
                "\n"
            )
            .into(),
            valid.clone() + &valid,
            valid.trim_end().into(),
            revoke.into(),
            valid.replace(AUTHORITY_EVENT_VERSION, "actuation.authority-event/v9"),
            valid.clone() + revoke + revoke,
        ];
        for (index, bytes) in cases.iter().enumerate() {
            let (_dir, store) = temp_store(&format!("malformed-{index}"));
            let legacy = legacy_file(&store.root, UNDERSCORE);
            put(&legacy, bytes);
            assert!(store.read(COLON).is_err(), "case {index} must refuse");
            assert!(store.revoke(COLON, 20, "must not append".into()).is_err());
            assert!(store.issue(COLON, 30, record_for(COLON)).is_err());
            assert_eq!(std::fs::read_to_string(&legacy).unwrap(), *bytes);
            assert!(!store.root.join("sources-v2").exists(), "case {index}");
        }
    }

    /// Two histories for one literal source are a conflict: both are kept
    /// byte for byte, every operation on that source refuses, and the listing
    /// names both files.
    #[test]
    fn two_histories_for_one_source_refuse_and_keep_both() {
        let (_dir, store) = temp_store("conflict");
        let legacy = legacy_file(&store.root, UNDERSCORE);
        let v2 = v2_file(&store.root, COLON);
        put(&legacy, &history(COLON, false));
        put(&v2, &history(COLON, true));
        assert!(store.read(COLON).is_err());
        assert!(store.issue(COLON, 30, record_for(COLON)).is_err());
        assert!(store.revoke(COLON, 30, "x".into()).is_err());
        assert_eq!(
            std::fs::read_to_string(&legacy).unwrap(),
            history(COLON, false)
        );
        assert_eq!(std::fs::read_to_string(&v2).unwrap(), history(COLON, true));
        let listing = store.list().unwrap();
        assert!(listing.authorities.is_empty());
        assert_eq!(listing.invalid.len(), 2, "{:?}", listing.invalid);
    }

    /// A crash between creating a V2 file and writing its Issue leaves it
    /// empty. An empty history holds no authority, so the source reads as
    /// absent and the retried issue lands in the same file.
    #[test]
    fn an_empty_v2_file_reads_absent_and_issue_completes_it() {
        let (_dir, store) = temp_store("empty-v2");
        let v2 = v2_file(&store.root, COLON);
        put(&v2, "");
        assert!(store.read(COLON).unwrap().is_none());
        store.issue(COLON, 10, record_for(COLON)).unwrap();
        assert_eq!(std::fs::read_to_string(&v2).unwrap(), history(COLON, false));
    }

    /// Unicode and refs longer than any filename keep their literal identity:
    /// nothing is truncated or normalised, and a fresh handle reads them back.
    #[test]
    fn unicode_and_long_refs_keep_their_literal_identity() {
        let (_dir, store) = temp_store("long-refs");
        let unicode = "authority-source:α/β|γ";
        for source in [
            unicode.to_owned(),
            format!("authority-source:{}", "long".repeat(150)),
            format!("authority-source:{}", "long".repeat(1500)),
        ] {
            store.issue(&source, 10, record_for(&source)).unwrap();
            let reopened = AuthorityStore::new(&store.root).unwrap();
            assert_eq!(
                reopened.read(&source).unwrap().unwrap().record,
                record_for(&source)
            );
            reopened.revoke(&source, 20, "long revoked".into()).unwrap();
            let state = AuthorityStore::new(&store.root)
                .unwrap()
                .read(&source)
                .unwrap()
                .unwrap();
            assert_eq!(state.revoked.unwrap().reason, "long revoked");
        }
        let unicode_bytes = std::fs::read_to_string(v2_file(&store.root, unicode)).unwrap();
        assert!(unicode_bytes.starts_with(&history(unicode, false)));
        assert_eq!(unicode_bytes.lines().count(), 2);
        assert_eq!(store.list().unwrap().authorities.len(), 3);
    }

    /// The nested layout adds directories a planted symlink could redirect;
    /// every component is opened without following links.
    #[cfg(unix)]
    #[test]
    fn a_symlinked_v2_directory_cannot_redirect_authority_bytes() {
        let (dir, store) = temp_store("v2-nofollow");
        std::fs::create_dir_all(&store.root).unwrap();
        let outside = dir.path().join("outside");
        std::fs::create_dir(&outside).unwrap();
        std::os::unix::fs::symlink(&outside, store.root.join("sources-v2")).unwrap();
        assert!(store.issue(COLON, 10, record_for(COLON)).is_err());
        assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 0);
    }
}
