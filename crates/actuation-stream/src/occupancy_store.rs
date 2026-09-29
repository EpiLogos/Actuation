//! The durable World Position occupancy ledger.
//!
//! One append-only JSONL ledger per Position, named by the sha256 of its
//! `position_ref`, under `$ACTUATION_OCCUPANCY_STORE` (default
//! `~/.actuation/occupancy/`). Three events exist: `tenure_began`,
//! `tenure_ended` and `presence`. Every mutation reads, checks and appends
//! under one exclusive lock on the store directory, then syncs the ledger to
//! disk, so two racing claims can never both see a vacant Position.
//!
//! Derivation is strict. The current occupant is the single open tenure; two
//! open tenures are an ambiguity and an unreadable line is corruption. Both are
//! refusals naming what was found — a ledger is never repaired, truncated or
//! read newest-wins. History is never rewritten: an ended tenure stays in the
//! ledger as testimony and confers nothing on its successor.
use actuation_core::{
    AgencyRef, AgentRef, AgentSessionRef, ExternalRef, OccupancyReading, OccupancyState,
    OccupantGenerationRef, OccupantPresence, Presence, Tenure, TenureEndKind, TenureKind,
    TenureSchema, WorldPositionRef,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub const OCCUPANCY_LEDGER_VERSION: &str = "actuation.position-ledger/v1";
pub const OCCUPANCY_LISTING_VERSION: &str = "actuation.position-occupancy-listing/v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum LedgerSchema {
    #[serde(rename = "actuation.position-ledger/v1")]
    V1,
}

/// One line of a Position's ledger.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
pub enum LedgerEvent {
    TenureBegan {
        schema: LedgerSchema,
        position_ref: WorldPositionRef,
        at_unix_ms: u64,
        tenure: Box<Tenure>,
    },
    TenureEnded {
        schema: LedgerSchema,
        position_ref: WorldPositionRef,
        at_unix_ms: u64,
        generation_ref: OccupantGenerationRef,
        end_kind: TenureEndKind,
        reason: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        successor_generation_ref: Option<OccupantGenerationRef>,
    },
    Presence {
        schema: LedgerSchema,
        position_ref: WorldPositionRef,
        at_unix_ms: u64,
        generation_ref: OccupantGenerationRef,
        presence: Presence,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        attention: Option<String>,
    },
}

impl LedgerEvent {
    fn position_ref(&self) -> &WorldPositionRef {
        match self {
            Self::TenureBegan { position_ref, .. }
            | Self::TenureEnded { position_ref, .. }
            | Self::Presence { position_ref, .. } => position_ref,
        }
    }
    fn at_unix_ms(&self) -> u64 {
        match self {
            Self::TenureBegan { at_unix_ms, .. }
            | Self::TenureEnded { at_unix_ms, .. }
            | Self::Presence { at_unix_ms, .. } => *at_unix_ms,
        }
    }
}

/// What a claim expects to find. The expectation is checked inside the same
/// lock as the append, so it is the claimant's compare-and-swap.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClaimExpectation {
    /// No expectation stated: allowed only on a vacant Position.
    Unstated,
    /// `--expect-vacant`.
    Vacant,
    /// `--expect-generation`: the claimant supersedes exactly this occupant.
    Generation(OccupantGenerationRef),
}

#[derive(Clone, Debug)]
pub struct ClaimRequest {
    pub position_ref: WorldPositionRef,
    pub agent_ref: AgentRef,
    pub agency_ref: AgencyRef,
    pub agent_session_ref: Option<AgentSessionRef>,
    pub session_space_ref: Option<ExternalRef>,
    pub harness_composition_ref: Option<ExternalRef>,
    pub model_ref: Option<ExternalRef>,
    pub workcell_ref: Option<ExternalRef>,
    pub gateway_address: Option<String>,
    pub reason: String,
    pub expectation: ClaimExpectation,
    pub kind: Option<TenureKind>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ClaimOutcome {
    pub tenure: Tenure,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superseded: Option<Tenure>,
    pub occupancy: OccupancyReading,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReleaseOutcome {
    pub tenure: Tenure,
    /// The occupancy after the release. Absent only when a release repaired
    /// part of an ambiguous ledger and more than one tenure is still open;
    /// those are listed in `still_open`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occupancy: Option<OccupancyReading>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub still_open: Vec<Tenure>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ListedPosition {
    pub position_ref: WorldPositionRef,
    pub state: OccupancyState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<Tenure>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<OccupantPresence>,
    pub generation_count: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct InvalidLedger {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_ref: Option<WorldPositionRef>,
    pub code: &'static str,
    pub error: String,
}

/// `actuation.position-occupancy-listing/v1`: every ledger in the store,
/// uncapped. A ledger that cannot be derived is listed under `invalid` with
/// its refusal code rather than dropped.
#[derive(Clone, Debug, Serialize)]
pub struct OccupancyListing {
    pub schema: &'static str,
    pub store: String,
    pub positions: Vec<ListedPosition>,
    pub invalid: Vec<InvalidLedger>,
}

/// Every way an occupancy operation can refuse. Each carries the state it
/// found, so a caller can name the current holder and the next lawful step.
#[derive(Clone, Debug)]
pub enum OccupancyError {
    /// The Position is occupied and the claim did not name its generation.
    Occupied { reading: Box<OccupancyReading> },
    /// `--expect-generation` named something other than the current occupant.
    StaleExpectation {
        expected: OccupantGenerationRef,
        reading: Box<OccupancyReading>,
    },
    /// The generation held this Position once and no longer does.
    Superseded {
        generation: Box<Tenure>,
        reading: Box<OccupancyReading>,
    },
    /// The generation never held this Position.
    UnknownGeneration {
        generation: OccupantGenerationRef,
        reading: Box<OccupancyReading>,
    },
    /// The requested tenure kind contradicts the Position's history.
    KindInconsistent {
        kind: TenureKind,
        detail: String,
        reading: Box<OccupancyReading>,
    },
    /// More than one tenure is open. Nothing picks a winner.
    Ambiguous {
        position_ref: WorldPositionRef,
        path: PathBuf,
        open: Vec<Tenure>,
    },
    /// The ledger cannot be read as a valid event history.
    Corrupt {
        position_ref: Option<WorldPositionRef>,
        path: PathBuf,
        line: Option<usize>,
        detail: String,
    },
    /// The request itself violates a record invariant.
    Invalid { detail: String },
    /// The store could not be read or written. `outcome_unknown` is true when
    /// a write had started: it may or may not have landed.
    Store {
        path: PathBuf,
        detail: String,
        outcome_unknown: bool,
    },
}

impl OccupancyError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Occupied { .. } => "occupancy.occupied",
            Self::StaleExpectation { .. } => "occupancy.stale_expectation",
            Self::Superseded { .. } => "occupancy.superseded",
            Self::UnknownGeneration { .. } => "occupancy.unknown_generation",
            Self::KindInconsistent { .. } => "occupancy.kind_inconsistent",
            Self::Ambiguous { .. } => "occupancy.ambiguous",
            Self::Corrupt { .. } => "occupancy.corrupt",
            Self::Invalid { .. } => "occupancy.invalid",
            Self::Store {
                outcome_unknown: true,
                ..
            } => "occupancy.outcome_unknown",
            Self::Store { .. } => "occupancy.store_unavailable",
        }
    }
}

impl std::fmt::Display for OccupancyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Occupied { reading } => write!(
                f,
                "{} is occupied by {}",
                reading.position_ref,
                reading
                    .current
                    .as_ref()
                    .map(|t| t.generation_ref.to_string())
                    .unwrap_or_default()
            ),
            Self::StaleExpectation { expected, reading } => write!(
                f,
                "expected {expected} to hold {} but it does not",
                reading.position_ref
            ),
            Self::Superseded { generation, .. } => write!(
                f,
                "{} no longer holds {}",
                generation.generation_ref, generation.position_ref
            ),
            Self::UnknownGeneration {
                generation,
                reading,
            } => write!(f, "{generation} never held {}", reading.position_ref),
            Self::KindInconsistent { detail, .. } | Self::Invalid { detail } => f.write_str(detail),
            Self::Ambiguous {
                position_ref, open, ..
            } => write!(f, "{position_ref} has {} open tenures", open.len()),
            Self::Corrupt { path, detail, .. } => write!(f, "{}: {detail}", path.display()),
            Self::Store { path, detail, .. } => write!(f, "{}: {detail}", path.display()),
        }
    }
}

impl std::error::Error for OccupancyError {}

type Outcome<T> = std::result::Result<T, OccupancyError>;

/// The folded content of one ledger, before the single-open-tenure law is
/// applied. Tenures are in append order, which the fold proves is ordinal
/// order.
#[derive(Clone, Debug, Default)]
struct Folded {
    position_ref: Option<WorldPositionRef>,
    tenures: Vec<Tenure>,
    presence: BTreeMap<OccupantGenerationRef, OccupantPresence>,
    /// Superseded generation → the successor its end names.
    successors: BTreeMap<OccupantGenerationRef, OccupantGenerationRef>,
    last_at_unix_ms: u64,
    ends_with_newline: bool,
    exists: bool,
}

impl Folded {
    fn tenure(&self, generation: &OccupantGenerationRef) -> Option<&Tenure> {
        self.tenures
            .iter()
            .find(|tenure| &tenure.generation_ref == generation)
    }
    fn open(&self) -> Vec<&Tenure> {
        self.tenures.iter().filter(|t| t.is_open()).collect()
    }
    fn next_ordinal(&self) -> u64 {
        self.tenures
            .iter()
            .map(|t| t.generation_ordinal)
            .max()
            .unwrap_or(0)
            + 1
    }
    fn reading(&self, position_ref: &WorldPositionRef, path: &Path) -> Outcome<OccupancyReading> {
        OccupancyReading::derive(position_ref.clone(), self.tenures.clone(), |generation| {
            self.presence.get(generation).cloned()
        })
        .map_err(|open| OccupancyError::Ambiguous {
            position_ref: position_ref.clone(),
            path: path.to_path_buf(),
            open,
        })
    }
}

fn fold(raw: &str, expected: Option<&WorldPositionRef>, path: &Path) -> Outcome<Folded> {
    let corrupt = |line: Option<usize>, position: Option<&WorldPositionRef>, detail: String| {
        OccupancyError::Corrupt {
            position_ref: position.cloned(),
            path: path.to_path_buf(),
            line,
            detail,
        }
    };
    let mut folded = Folded {
        position_ref: expected.cloned(),
        ends_with_newline: raw.is_empty() || raw.ends_with('\n'),
        exists: true,
        ..Folded::default()
    };
    for (index, line) in raw.split('\n').enumerate() {
        let number = index + 1;
        if line.trim().is_empty() {
            continue;
        }
        let event: LedgerEvent = serde_json::from_str(line).map_err(|e| {
            corrupt(
                Some(number),
                folded.position_ref.as_ref(),
                format!("line {number} is not a valid occupancy event ({e}); the ledger is never read past a bad line"),
            )
        })?;
        match &folded.position_ref {
            Some(position) if position != event.position_ref() => {
                return Err(corrupt(
                    Some(number),
                    Some(position),
                    format!(
                        "line {number} records Position {} inside the ledger of {position}",
                        event.position_ref()
                    ),
                ))
            }
            Some(_) => {}
            None => folded.position_ref = Some(event.position_ref().clone()),
        }
        let position = folded.position_ref.clone();
        let position = position.as_ref();
        if event.at_unix_ms() < folded.last_at_unix_ms {
            return Err(corrupt(
                Some(number),
                position,
                format!("line {number} is dated before the line that precedes it"),
            ));
        }
        folded.last_at_unix_ms = event.at_unix_ms();
        match event {
            LedgerEvent::TenureBegan {
                position_ref,
                at_unix_ms,
                tenure,
                ..
            } => {
                let fault = if tenure.position_ref != position_ref {
                    Some("its tenure names a different Position".to_owned())
                } else if !tenure.is_open() {
                    Some("a tenure cannot begin already ended".to_owned())
                } else if tenure.began_at_unix_ms != at_unix_ms {
                    Some("its tenure's began_at_unix_ms disagrees with the event time".to_owned())
                } else if folded.tenure(&tenure.generation_ref).is_some() {
                    Some(format!(
                        "generation {} begins a second time",
                        tenure.generation_ref
                    ))
                } else if tenure.generation_ordinal != folded.next_ordinal() {
                    Some(format!(
                        "generation ordinal {} is not max + 1 ({})",
                        tenure.generation_ordinal,
                        folded.next_ordinal()
                    ))
                } else if let Err(e) = tenure.check() {
                    Some(e.to_string())
                } else if let Some(predecessor) = &tenure.predecessor_generation_ref {
                    match folded.tenure(predecessor) {
                        Some(ended)
                            if ended.end_kind == Some(TenureEndKind::Superseded)
                                && ended.ended_at_unix_ms == Some(at_unix_ms) =>
                        {
                            None
                        }
                        _ => Some(format!(
                            "its predecessor {predecessor} was not superseded in the same write"
                        )),
                    }
                } else {
                    None
                };
                if let Some(fault) = fault {
                    return Err(corrupt(
                        Some(number),
                        position,
                        format!("line {number} tenure_began is invalid: {fault}"),
                    ));
                }
                folded.tenures.push(*tenure);
            }
            LedgerEvent::TenureEnded {
                at_unix_ms,
                generation_ref,
                end_kind,
                reason,
                successor_generation_ref,
                ..
            } => {
                let successor_ok = match end_kind {
                    TenureEndKind::Released => successor_generation_ref.is_none(),
                    TenureEndKind::Superseded => successor_generation_ref.is_some(),
                };
                let Some(tenure) = folded
                    .tenures
                    .iter_mut()
                    .find(|t| t.generation_ref == generation_ref)
                else {
                    return Err(corrupt(
                        Some(number),
                        position,
                        format!("line {number} ends {generation_ref}, which never began"),
                    ));
                };
                if !tenure.is_open() {
                    return Err(corrupt(
                        Some(number),
                        position,
                        format!("line {number} ends {generation_ref} a second time"),
                    ));
                }
                if !successor_ok {
                    return Err(corrupt(
                        Some(number),
                        position,
                        format!(
                            "line {number}: a {end_kind} end {} a successor generation",
                            if end_kind == TenureEndKind::Released {
                                "cannot name"
                            } else {
                                "must name"
                            }
                        ),
                    ));
                }
                tenure.ended_at_unix_ms = Some(at_unix_ms);
                tenure.end_kind = Some(end_kind);
                tenure.end_reason = Some(reason);
                if let Some(successor) = successor_generation_ref {
                    folded.successors.insert(generation_ref.clone(), successor);
                }
                if let Err(e) = tenure.check() {
                    return Err(corrupt(
                        Some(number),
                        position,
                        format!("line {number} tenure_ended is invalid: {e}"),
                    ));
                }
            }
            LedgerEvent::Presence {
                at_unix_ms,
                generation_ref,
                presence,
                attention,
                ..
            } => {
                if !folded
                    .tenure(&generation_ref)
                    .is_some_and(|tenure| tenure.is_open())
                {
                    return Err(corrupt(
                        Some(number),
                        position,
                        format!(
                            "line {number} records presence for {generation_ref}, which does not hold the Position"
                        ),
                    ));
                }
                folded.presence.insert(
                    generation_ref.clone(),
                    OccupantPresence {
                        generation_ref,
                        presence,
                        attention,
                        at_unix_ms,
                    },
                );
            }
        }
    }
    // A supersession is two lines in one write. A successor that never began
    // (or began without naming this predecessor) is a torn handover: report
    // it, never guess which side holds.
    for (predecessor, successor) in &folded.successors {
        let joined = folded
            .tenure(successor)
            .is_some_and(|next| next.predecessor_generation_ref.as_ref() == Some(predecessor));
        if !joined {
            return Err(corrupt(
                None,
                folded.position_ref.as_ref(),
                format!(
                    "{predecessor} is recorded as superseded by {successor}, which never began as its successor (torn handover)"
                ),
            ));
        }
    }
    Ok(folded)
}

/// sha256 of the ref, lowercase hex: the ledger's file stem.
pub fn position_ledger_stem(position_ref: &WorldPositionRef) -> String {
    format!("{:x}", Sha256::digest(position_ref.as_str().as_bytes()))
}

/// RFC 3339 rendering of a unix-millisecond instant, for human-facing text.
pub fn format_unix_ms(at_unix_ms: u64) -> String {
    time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(at_unix_ms) * 1_000_000)
        .ok()
        .and_then(|at| {
            at.format(&time::format_description::well_known::Rfc3339)
                .ok()
        })
        .unwrap_or_else(|| format!("{at_unix_ms}ms"))
}

fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or_default()
}

fn mint_generation(folded: &Folded) -> Outcome<OccupantGenerationRef> {
    loop {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).map_err(|e| OccupancyError::Store {
            path: PathBuf::new(),
            detail: format!("no entropy to mint an occupant generation: {e}"),
            outcome_unknown: false,
        })?;
        let generation = OccupantGenerationRef::from_random_bytes(bytes);
        if folded.tenure(&generation).is_none() {
            return Ok(generation);
        }
    }
}

#[derive(Clone, Debug)]
pub struct OccupancyStore {
    root: PathBuf,
}

impl OccupancyStore {
    pub fn new(root: impl Into<PathBuf>) -> actuation_core::Result<Self> {
        let root = root.into();
        if actuation_core::is_blank_reference(&root.to_string_lossy()) {
            return Err(actuation_core::Error::new(
                "occupancy store root must be a non-empty directory path",
            ));
        }
        Ok(Self { root })
    }

    /// `explicit`, then `ACTUATION_OCCUPANCY_STORE`, then
    /// `~/.actuation/occupancy` — the same knob family as the stream and
    /// authority stores, under its own name so the ledgers never mix.
    pub fn from_environment(explicit: Option<&str>) -> actuation_core::Result<Self> {
        if let Some(root) = explicit {
            return Self::new(root);
        }
        if let Some(root) = std::env::var_os("ACTUATION_OCCUPANCY_STORE") {
            return Self::new(PathBuf::from(root));
        }
        let home = std::env::var_os("HOME").ok_or_else(|| {
            actuation_core::Error::new("HOME is unavailable; supply an occupancy store explicitly")
        })?;
        Self::new(PathBuf::from(home).join(".actuation/occupancy"))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn ledger_path(&self, position_ref: &WorldPositionRef) -> PathBuf {
        self.root
            .join(format!("{}.jsonl", position_ledger_stem(position_ref)))
    }

    fn store_error(&self, path: &Path, detail: impl std::fmt::Display) -> OccupancyError {
        OccupancyError::Store {
            path: path.to_path_buf(),
            detail: detail.to_string(),
            outcome_unknown: false,
        }
    }

    /// The directory inode is the lock locus, exactly as the stream store
    /// does it: no lock files enter the public store. `None` means the store
    /// does not exist yet and a reader has nothing to lock.
    fn lock(&self, exclusive: bool) -> Outcome<Option<File>> {
        if exclusive {
            fs::create_dir_all(&self.root).map_err(|e| self.store_error(&self.root, e))?;
        } else if !self.root.exists() {
            return Ok(None);
        }
        let directory = File::open(&self.root).map_err(|e| self.store_error(&self.root, e))?;
        #[cfg(unix)]
        {
            let locked = if exclusive {
                directory.lock()
            } else {
                directory.lock_shared()
            };
            locked.map_err(|e| self.store_error(&self.root, format!("could not lock: {e}")))?;
        }
        #[cfg(not(unix))]
        {
            return Err(self.store_error(
                &self.root,
                "occupancy ledger locking is not implemented on this target",
            ));
        }
        Ok(Some(directory))
    }

    fn file_options() -> OpenOptions {
        let mut options = OpenOptions::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW);
        }
        options
    }

    fn load_path(&self, path: &Path, expected: Option<&WorldPositionRef>) -> Outcome<Folded> {
        let mut raw = String::new();
        match Self::file_options().read(true).open(path) {
            Ok(mut file) => {
                file.read_to_string(&mut raw)
                    .map_err(|e| self.store_error(path, e))?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Folded {
                    position_ref: expected.cloned(),
                    ends_with_newline: true,
                    ..Folded::default()
                })
            }
            Err(e) => return Err(self.store_error(path, e)),
        }
        fold(&raw, expected, path)
    }

    fn load(&self, position_ref: &WorldPositionRef) -> Outcome<(PathBuf, Folded)> {
        let path = self.ledger_path(position_ref);
        let folded = self.load_path(&path, Some(position_ref))?;
        Ok((path, folded))
    }

    fn append(
        &self,
        path: &Path,
        folded: &Folded,
        events: &[LedgerEvent],
        directory: &File,
    ) -> Outcome<()> {
        let mut buffer = if folded.ends_with_newline {
            String::new()
        } else {
            // A valid final line without its newline stays readable; the
            // separator keeps the next record on its own line.
            String::from("\n")
        };
        for event in events {
            let line = serde_json::to_string(event)
                .map_err(|e| self.store_error(path, format!("could not encode event: {e}")))?;
            buffer.push_str(&line);
            buffer.push('\n');
        }
        let mut file = Self::file_options()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| self.store_error(path, e))?;
        let unknown = |e: std::io::Error| OccupancyError::Store {
            path: path.to_path_buf(),
            detail: e.to_string(),
            outcome_unknown: true,
        };
        file.write_all(buffer.as_bytes()).map_err(unknown)?;
        file.sync_data().map_err(unknown)?;
        if !folded.exists {
            directory.sync_all().map_err(unknown)?;
        }
        Ok(())
    }

    /// Event time never runs backwards inside a ledger, even when the clock
    /// does: a skewed clock must not make an honest supersession unreadable.
    fn event_time(folded: &Folded) -> u64 {
        now_unix_ms().max(folded.last_at_unix_ms)
    }

    pub fn read(&self, position_ref: &WorldPositionRef) -> Outcome<OccupancyReading> {
        let _lock = self.lock(false)?;
        let (path, folded) = self.load(position_ref)?;
        folded.reading(position_ref, &path)
    }

    /// Every ledger in the store, uncapped, in `position_ref` order.
    pub fn list(&self) -> Outcome<OccupancyListing> {
        let mut listing = OccupancyListing {
            schema: OCCUPANCY_LISTING_VERSION,
            store: self.root.display().to_string(),
            positions: Vec::new(),
            invalid: Vec::new(),
        };
        let Some(_lock) = self.lock(false)? else {
            return Ok(listing);
        };
        let entries = fs::read_dir(&self.root).map_err(|e| self.store_error(&self.root, e))?;
        let mut paths = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| self.store_error(&self.root, e))?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "jsonl") {
                paths.push(path);
            }
        }
        paths.sort();
        for path in paths {
            let shown = path.display().to_string();
            let folded = match self.load_path(&path, None) {
                Ok(folded) => folded,
                Err(error) => {
                    let position_ref = match &error {
                        OccupancyError::Corrupt { position_ref, .. } => position_ref.clone(),
                        _ => None,
                    };
                    listing.invalid.push(InvalidLedger {
                        path: shown,
                        position_ref,
                        code: error.code(),
                        error: error.to_string(),
                    });
                    continue;
                }
            };
            let Some(position_ref) = folded.position_ref.clone() else {
                listing.invalid.push(InvalidLedger {
                    path: shown,
                    position_ref: None,
                    code: "occupancy.corrupt",
                    error: "ledger holds no events, so it names no Position".into(),
                });
                continue;
            };
            let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned());
            if stem.as_deref() != Some(position_ledger_stem(&position_ref).as_str()) {
                listing.invalid.push(InvalidLedger {
                    path: shown,
                    position_ref: Some(position_ref),
                    code: "occupancy.corrupt",
                    error: "ledger file name is not the digest of the Position it records".into(),
                });
                continue;
            }
            match folded.reading(&position_ref, &path) {
                Ok(reading) => listing.positions.push(ListedPosition {
                    position_ref,
                    state: reading.state,
                    generation_count: reading.generations.len(),
                    current: reading.current,
                    presence: reading.presence,
                }),
                Err(error) => listing.invalid.push(InvalidLedger {
                    path: shown,
                    position_ref: Some(position_ref),
                    code: error.code(),
                    error: error.to_string(),
                }),
            }
        }
        listing
            .positions
            .sort_by(|a, b| a.position_ref.cmp(&b.position_ref));
        Ok(listing)
    }

    /// Open a tenure. A vacant Position admits an unstated or vacant
    /// expectation; an occupied one admits only its current generation, and
    /// then the predecessor is superseded in the same locked write.
    pub fn claim(&self, request: ClaimRequest) -> Outcome<ClaimOutcome> {
        let directory = self
            .lock(true)?
            .expect("an exclusive lock creates the store");
        let (path, folded) = self.load(&request.position_ref)?;
        let open = folded.open();
        let predecessor = match (&request.expectation, open.as_slice()) {
            (_, [_, _, ..]) => {
                return Err(OccupancyError::Ambiguous {
                    position_ref: request.position_ref.clone(),
                    path,
                    open: open.into_iter().cloned().collect(),
                })
            }
            (ClaimExpectation::Unstated | ClaimExpectation::Vacant, []) => None,
            (ClaimExpectation::Unstated | ClaimExpectation::Vacant, [_]) => {
                return Err(OccupancyError::Occupied {
                    reading: Box::new(folded.reading(&request.position_ref, &path)?),
                })
            }
            (ClaimExpectation::Generation(expected), [current])
                if &current.generation_ref == expected =>
            {
                Some((*current).clone())
            }
            (ClaimExpectation::Generation(expected), _) => {
                return Err(OccupancyError::StaleExpectation {
                    expected: expected.clone(),
                    reading: Box::new(folded.reading(&request.position_ref, &path)?),
                })
            }
        };
        let has_history = !folded.tenures.is_empty();
        let kind = match (request.kind, &predecessor) {
            (Some(TenureKind::Initial), _) if has_history => {
                return Err(OccupancyError::KindInconsistent {
                    kind: TenureKind::Initial,
                    detail: format!(
                        "--kind initial names a first occupancy, but {} has held {} generation(s)",
                        request.position_ref,
                        folded.tenures.len()
                    ),
                    reading: Box::new(folded.reading(&request.position_ref, &path)?),
                })
            }
            (Some(TenureKind::Handover), None) => {
                return Err(OccupancyError::KindInconsistent {
                    kind: TenureKind::Handover,
                    detail: format!(
                        "--kind handover needs a live occupant to hand over from, but {} is vacant",
                        request.position_ref
                    ),
                    reading: Box::new(folded.reading(&request.position_ref, &path)?),
                })
            }
            (Some(kind), _) => kind,
            (None, Some(_)) => TenureKind::Handover,
            (None, None) if has_history => TenureKind::Fresh,
            (None, None) => TenureKind::Initial,
        };
        let at = Self::event_time(&folded);
        let generation_ref = mint_generation(&folded)?;
        let tenure = Tenure {
            schema: TenureSchema::V1,
            position_ref: request.position_ref.clone(),
            generation_ref: generation_ref.clone(),
            generation_ordinal: folded.next_ordinal(),
            kind,
            agent_ref: request.agent_ref,
            agency_ref: request.agency_ref,
            agent_session_ref: request.agent_session_ref,
            session_space_ref: request.session_space_ref,
            harness_composition_ref: request.harness_composition_ref,
            model_ref: request.model_ref,
            workcell_ref: request.workcell_ref,
            gateway_address: request.gateway_address,
            began_at_unix_ms: at,
            reason: request.reason.clone(),
            predecessor_generation_ref: predecessor.as_ref().map(|p| p.generation_ref.clone()),
            ended_at_unix_ms: None,
            end_kind: None,
            end_reason: None,
        };
        tenure.check().map_err(|e| OccupancyError::Invalid {
            detail: e.to_string(),
        })?;
        let mut events = Vec::new();
        if let Some(predecessor) = &predecessor {
            events.push(LedgerEvent::TenureEnded {
                schema: LedgerSchema::V1,
                position_ref: request.position_ref.clone(),
                at_unix_ms: at,
                generation_ref: predecessor.generation_ref.clone(),
                end_kind: TenureEndKind::Superseded,
                reason: request.reason.clone(),
                successor_generation_ref: Some(generation_ref.clone()),
            });
        }
        events.push(LedgerEvent::TenureBegan {
            schema: LedgerSchema::V1,
            position_ref: request.position_ref.clone(),
            at_unix_ms: at,
            tenure: Box::new(tenure.clone()),
        });
        self.append(&path, &folded, &events, &directory)?;
        let (path, after) = self.load(&request.position_ref)?;
        let occupancy = after.reading(&request.position_ref, &path)?;
        let superseded = predecessor.and_then(|p| after.tenure(&p.generation_ref).cloned());
        Ok(ClaimOutcome {
            tenure,
            superseded,
            occupancy,
        })
    }

    /// Resolve `generation` against the ledger. Current → its open tenure.
    /// Anything else is the matching refusal.
    fn current_of(
        folded: &Folded,
        position_ref: &WorldPositionRef,
        path: &Path,
        generation: &OccupantGenerationRef,
    ) -> Outcome<Tenure> {
        let reading = Box::new(folded.reading(position_ref, path)?);
        match folded.tenure(generation) {
            Some(tenure) if tenure.is_open() => Ok(tenure.clone()),
            Some(tenure) => Err(OccupancyError::Superseded {
                generation: Box::new(tenure.clone()),
                reading,
            }),
            None => Err(OccupancyError::UnknownGeneration {
                generation: generation.clone(),
                reading,
            }),
        }
    }

    /// End the current tenure; the Position becomes vacant. On an ambiguous
    /// ledger this is also the repair: the caller names which open tenure
    /// ends, and nothing is chosen for them.
    pub fn release(
        &self,
        position_ref: &WorldPositionRef,
        generation: &OccupantGenerationRef,
        reason: &str,
    ) -> Outcome<ReleaseOutcome> {
        if actuation_core::is_blank_reference(reason) {
            return Err(OccupancyError::Invalid {
                detail: "a release must state its reason".into(),
            });
        }
        let directory = self
            .lock(true)?
            .expect("an exclusive lock creates the store");
        let (path, folded) = self.load(position_ref)?;
        let ambiguous_member = folded.open().len() > 1
            && folded
                .tenure(generation)
                .is_some_and(|tenure| tenure.is_open());
        if !ambiguous_member {
            Self::current_of(&folded, position_ref, &path, generation)?;
        }
        let event = LedgerEvent::TenureEnded {
            schema: LedgerSchema::V1,
            position_ref: position_ref.clone(),
            at_unix_ms: Self::event_time(&folded),
            generation_ref: generation.clone(),
            end_kind: TenureEndKind::Released,
            reason: reason.to_owned(),
            successor_generation_ref: None,
        };
        self.append(&path, &folded, &[event], &directory)?;
        let (path, after) = self.load(position_ref)?;
        let tenure = after
            .tenure(generation)
            .cloned()
            .expect("the released tenure is in the ledger");
        match after.reading(position_ref, &path) {
            Ok(reading) => Ok(ReleaseOutcome {
                tenure,
                occupancy: Some(reading),
                still_open: Vec::new(),
            }),
            Err(OccupancyError::Ambiguous { open, .. }) => Ok(ReleaseOutcome {
                tenure,
                occupancy: None,
                still_open: open,
            }),
            Err(other) => Err(other),
        }
    }

    /// The current tenure of `generation`, or the refusal that says why it is
    /// not current. Read-only.
    pub fn verify(
        &self,
        position_ref: &WorldPositionRef,
        generation: &OccupantGenerationRef,
    ) -> Outcome<Tenure> {
        let _lock = self.lock(false)?;
        let (path, folded) = self.load(position_ref)?;
        Self::current_of(&folded, position_ref, &path, generation)
    }

    /// Record presence for the current occupant only.
    pub fn presence(
        &self,
        position_ref: &WorldPositionRef,
        generation: &OccupantGenerationRef,
        presence: Presence,
        attention: Option<String>,
    ) -> Outcome<OccupantPresence> {
        let directory = self
            .lock(true)?
            .expect("an exclusive lock creates the store");
        let (path, folded) = self.load(position_ref)?;
        Self::current_of(&folded, position_ref, &path, generation)?;
        let at = Self::event_time(&folded);
        let event = LedgerEvent::Presence {
            schema: LedgerSchema::V1,
            position_ref: position_ref.clone(),
            at_unix_ms: at,
            generation_ref: generation.clone(),
            presence,
            attention: attention.clone(),
        };
        self.append(&path, &folded, &[event], &directory)?;
        Ok(OccupantPresence {
            generation_ref: generation.clone(),
            presence,
            attention,
            at_unix_ms: at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn position() -> WorldPositionRef {
        WorldPositionRef::new("central:position:project:O-I:factory-guardian").unwrap()
    }

    fn request(agent: &str, expectation: ClaimExpectation) -> ClaimRequest {
        ClaimRequest {
            position_ref: position(),
            agent_ref: AgentRef::new(agent).unwrap(),
            agency_ref: AgencyRef::new("agency:guardian").unwrap(),
            agent_session_ref: None,
            session_space_ref: None,
            harness_composition_ref: None,
            model_ref: None,
            workcell_ref: None,
            gateway_address: None,
            reason: "test".into(),
            expectation,
            kind: None,
        }
    }

    fn store() -> (tempfile::TempDir, OccupancyStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = OccupancyStore::new(dir.path().join("occupancy")).unwrap();
        (dir, store)
    }

    #[test]
    fn ledger_events_round_trip_and_refuse_unknown_keys() {
        let event = LedgerEvent::Presence {
            schema: LedgerSchema::V1,
            position_ref: position(),
            at_unix_ms: 5,
            generation_ref: OccupantGenerationRef::from_random_bytes([1; 16]),
            presence: Presence::Idle,
            attention: None,
        };
        let line = serde_json::to_value(&event).unwrap();
        assert_eq!(line["event"], "presence");
        assert_eq!(line["schema"], OCCUPANCY_LEDGER_VERSION);
        assert_eq!(
            serde_json::from_value::<LedgerEvent>(line.clone()).unwrap(),
            event
        );
        let mut extra = line;
        extra["extra"] = serde_json::json!(true);
        assert!(serde_json::from_value::<LedgerEvent>(extra).is_err());
    }

    #[test]
    fn a_reader_never_creates_the_store() {
        let (_dir, store) = store();
        let reading = store.read(&position()).unwrap();
        assert_eq!(reading.state, OccupancyState::Vacant);
        assert!(reading.generations.is_empty());
        assert!(store.list().unwrap().positions.is_empty());
        assert!(!store.root().exists());
    }

    #[test]
    fn claim_supersede_release_and_fresh_claim_follow_the_ordinals() {
        let (_dir, store) = store();
        let first = store
            .claim(request("agent/a", ClaimExpectation::Unstated))
            .unwrap();
        assert_eq!(first.tenure.kind, TenureKind::Initial);
        assert_eq!(first.tenure.generation_ordinal, 1);
        let second = store
            .claim(request(
                "agent/b",
                ClaimExpectation::Generation(first.tenure.generation_ref.clone()),
            ))
            .unwrap();
        assert_eq!(second.tenure.kind, TenureKind::Handover);
        assert_eq!(
            second.superseded.as_ref().unwrap().end_kind,
            Some(TenureEndKind::Superseded)
        );
        assert!(matches!(
            store.verify(&position(), &first.tenure.generation_ref),
            Err(OccupancyError::Superseded { .. })
        ));
        store
            .release(&position(), &second.tenure.generation_ref, "done")
            .unwrap();
        let third = store
            .claim(request("agent/c", ClaimExpectation::Vacant))
            .unwrap();
        assert_eq!(third.tenure.kind, TenureKind::Fresh);
        assert_eq!(third.tenure.generation_ordinal, 3);
        assert!(third.tenure.predecessor_generation_ref.is_none());
        assert_eq!(third.occupancy.generations.len(), 3);
        assert_eq!(
            third.occupancy.predecessor.unwrap().generation_ref,
            second.tenure.generation_ref
        );
    }

    #[test]
    fn a_torn_handover_is_corruption_not_a_vacancy() {
        let (_dir, store) = store();
        let first = store
            .claim(request("agent/a", ClaimExpectation::Unstated))
            .unwrap();
        let path = store.ledger_path(&position());
        let torn = LedgerEvent::TenureEnded {
            schema: LedgerSchema::V1,
            position_ref: position(),
            at_unix_ms: first.tenure.began_at_unix_ms + 1,
            generation_ref: first.tenure.generation_ref.clone(),
            end_kind: TenureEndKind::Superseded,
            reason: "crash".into(),
            successor_generation_ref: Some(OccupantGenerationRef::from_random_bytes([9; 16])),
        };
        let mut file = OpenOptions::new().append(true).open(&path).unwrap();
        writeln!(file, "{}", serde_json::to_string(&torn).unwrap()).unwrap();
        assert!(matches!(
            store.read(&position()),
            Err(OccupancyError::Corrupt { .. })
        ));
    }
}
