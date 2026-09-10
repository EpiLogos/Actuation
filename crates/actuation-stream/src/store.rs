use crate::*;
use actuation_core::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

fn io<T>(result: std::io::Result<T>) -> Result<T> {
    result.map_err(|e| Error::new(e.to_string()))
}
fn slot<T>(value: Option<T>) -> Slot<T> {
    value.map(Slot::Value).unwrap_or(Slot::Absent)
}

#[derive(Clone, Debug, Deserialize)]
pub struct OpenStream {
    pub stream_ref: StreamRef,
    pub actuation_ref: ActuationRef,
    pub agency_ref: AgencyRef,
    pub agent_session_ref: AgentSessionRef,
    #[serde(default)]
    pub world_binding_ref: Option<WorldBindingRef>,
    #[serde(default)]
    pub provenance: Option<Vec<ExternalRef>>,
    #[serde(default)]
    pub started_at: Option<Timestamp>,
}
impl OpenStream {
    pub fn empty(&self) -> Result<ActuationStream> {
        ActuationStream::new(ActuationStreamFields {
            schema: StreamSchema::V1,
            stream_ref: self.stream_ref.clone(),
            actuation_ref: self.actuation_ref.clone(),
            agency_ref: self.agency_ref.clone(),
            agent_session_ref: self.agent_session_ref.clone(),
            world_binding_ref: slot(self.world_binding_ref.clone()),
            participating_loci: Slot::Absent,
            surface_refs: Slot::Absent,
            provenance: slot(self.provenance.clone()),
            lifecycle: StreamLifecycle::new(StreamLifecycleFields {
                state: StreamState::Open,
                started_at: slot(self.started_at.clone()),
                ended_at: Slot::Absent,
                extensions: Extensions::new(),
            })?,
            cursor: StreamCursor::new(StreamCursorFields {
                last_sequence: Count::ZERO,
                next_sequence: Count::ONE,
                extensions: Extensions::new(),
            })?,
            events: Vec::new(),
            extensions: Extensions::new(),
        })
    }
}
/// An existing stream can be checked against a partial supplied identity. These
/// hints never rewrite its World or history; opening a new stream needs all
/// three actual identity refs, exactly as the existing public operation did.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct IdentityHint {
    #[serde(default)]
    pub actuation_ref: Option<ActuationRef>,
    #[serde(default)]
    pub agency_ref: Option<AgencyRef>,
    #[serde(default)]
    pub agent_session_ref: Option<AgentSessionRef>,
    #[serde(default)]
    pub world_binding_ref: Option<WorldBindingRef>,
    #[serde(default)]
    pub provenance: Option<Vec<ExternalRef>>,
    #[serde(default)]
    pub started_at: Option<Timestamp>,
}
impl IdentityHint {
    fn check(&self, stream: &ActuationStream) -> Result<()> {
        let s = stream.fields();
        if self
            .actuation_ref
            .as_ref()
            .is_some_and(|r| r != &s.actuation_ref)
            || self.agency_ref.as_ref().is_some_and(|r| r != &s.agency_ref)
            || self
                .agent_session_ref
                .as_ref()
                .is_some_and(|r| r != &s.agent_session_ref)
        {
            Err(Error::new(
                "refusing to reuse a stream under different Actuation, Agency or Session identity",
            ))
        } else {
            Ok(())
        }
    }
    fn opening(&self, stream_ref: StreamRef) -> Result<OpenStream> {
        Ok(OpenStream {
            stream_ref,
            actuation_ref: self
                .actuation_ref
                .clone()
                .ok_or_else(|| Error::new("opening requires actuation_ref"))?,
            agency_ref: self
                .agency_ref
                .clone()
                .ok_or_else(|| Error::new("opening requires agency_ref"))?,
            agent_session_ref: self
                .agent_session_ref
                .clone()
                .ok_or_else(|| Error::new("opening requires agent_session_ref"))?,
            world_binding_ref: self.world_binding_ref.clone(),
            provenance: self.provenance.clone(),
            started_at: self.started_at.clone(),
        })
    }
}
impl From<&OpenStream> for IdentityHint {
    fn from(o: &OpenStream) -> Self {
        Self {
            actuation_ref: Some(o.actuation_ref.clone()),
            agency_ref: Some(o.agency_ref.clone()),
            agent_session_ref: Some(o.agent_session_ref.clone()),
            world_binding_ref: o.world_binding_ref.clone(),
            provenance: o.provenance.clone(),
            started_at: o.started_at.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NativeBoundary {
    pub harness: ExternalRef,
    pub native_event: ExternalRef,
    pub boundary: Option<ExternalRef>,
    pub catalog_revision: Option<Count>,
}
/// An observation catalogue supplies the native-to-portable reading; it does
/// not supply acting authority. The native catalogue implementation belongs to
/// actuation-adapters, never to the generic stream domain.
pub trait BoundaryCatalogue {
    fn boundary(&self, harness: &str, native_event: &str) -> Result<NativeBoundary>;
}
#[derive(Clone, Debug, Deserialize)]
pub struct BoundaryOccurrence {
    pub stream_ref: StreamRef,
    pub harness: ExternalRef,
    pub native_event: ExternalRef,
    #[serde(default)]
    pub identity: Option<IdentityHint>,
    #[serde(default)]
    pub event_ref: Option<EventRef>,
    #[serde(default)]
    pub observed_at: Option<Timestamp>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub native_trace_ref: Option<ExternalRef>,
    #[serde(default)]
    pub metadata: Option<JsonObject>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct UsageOccurrence {
    pub stream_ref: StreamRef,
    pub observation: ModelUsageObservation,
    #[serde(default)]
    pub identity: Option<IdentityHint>,
    #[serde(default)]
    pub event_ref: Option<EventRef>,
}
#[derive(Clone, Debug, Serialize)]
pub struct OccurrenceReceipt {
    pub stream_ref: StreamRef,
    pub event: StreamEvent,
    pub cursor: StreamCursor,
    pub lifecycle: StreamLifecycle,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deduplicated: Option<bool>,
}
impl OccurrenceReceipt {
    fn new(stream: &ActuationStream, event: StreamEvent, deduplicated: Option<bool>) -> Self {
        Self {
            stream_ref: stream.fields().stream_ref.clone(),
            event,
            cursor: stream.fields().cursor.clone(),
            lifecycle: stream.fields().lifecycle.clone(),
            deduplicated,
        }
    }
}

/// Persistence owns bytes and sequencing. It does not decide the caller's
/// determination, provider/body selection, or Factory Recognition.
pub trait StreamStore: Send + Sync {
    fn open(&self, opening: &OpenStream) -> Result<ActuationStream>;
    fn load(&self, stream_ref: &StreamRef) -> Result<ActuationStream>;
    fn append(&self, stream_ref: &StreamRef, event: StreamEvent) -> Result<ActuationStream>;
    fn close(
        &self,
        stream_ref: &StreamRef,
        state: TerminalState,
        ended_at: Timestamp,
    ) -> Result<ActuationStream>;
}

pub fn stream_file_name(stream_ref: &StreamRef) -> String {
    let mut name = String::new();
    for byte in stream_ref.as_str().bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&byte) {
            name.push(byte as char);
        } else {
            use std::fmt::Write;
            write!(&mut name, "%{byte:02X}").expect("writing to String");
        }
    }
    name.push_str(".jsonl");
    name
}
/// Reconstruct only from real lines. Exactly one final newline is formatting;
/// interior holes, duplicate refs, invalid JSON and sequence gaps are refused.
pub fn fold_stream_file(raw: &str) -> Result<ActuationStream> {
    let mut lines: Vec<_> = raw.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    let first = lines
        .first()
        .ok_or_else(|| Error::new("durable ActuationStream file is empty"))?;
    let mut header: Value = serde_json::from_str(first)
        .map_err(|e| Error::new(format!("invalid durable stream header: {e}")))?;
    crate::wire::object(&header)?;
    let mut events = Vec::new();
    for (position, line) in lines.iter().enumerate().skip(1) {
        let value: Value = serde_json::from_str(line).map_err(|e| {
            Error::new(format!(
                "torn or invalid event at position {}: {e}; tail is not silently dropped",
                position + 1
            ))
        })?;
        let event = StreamEvent::try_from(value.clone()).map_err(|e| {
            Error::new(format!(
                "event at position {} violates the portable contract: {e}",
                position + 1
            ))
        })?;
        event.assert_sequence(Count::new(position as u64)?)?;
        events.push(value);
    }
    header["cursor"] = json!({"last_sequence":events.len(),"next_sequence":events.len()+1});
    header["events"] = Value::Array(events);
    ActuationStream::try_from(header)
}
fn encoded_stream(stream: &ActuationStream) -> Result<String> {
    let mut raw = serde_json::to_string(&stream.header())?;
    raw.push('\n');
    for event in &stream.fields().events {
        raw.push_str(&serde_json::to_string(event)?);
        raw.push('\n');
    }
    Ok(raw)
}
fn event_identity() -> Result<EventRef> {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b).map_err(|e| Error::new(e.to_string()))?;
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    EventRef::new(format!("actuation:event:{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",b[0],b[1],b[2],b[3],b[4],b[5],b[6],b[7],b[8],b[9],b[10],b[11],b[12],b[13],b[14],b[15]))
}

#[derive(Clone, Debug)]
pub struct JsonlStreamStore {
    root: PathBuf,
}
impl JsonlStreamStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        if is_blank_reference(&root.to_string_lossy()) {
            return Err(Error::new(
                "stream store root must be a non-empty directory path",
            ));
        }
        Ok(Self { root })
    }
    pub fn from_environment(explicit: Option<&str>) -> Result<Self> {
        if let Some(root) = explicit {
            return Self::new(root);
        }
        if let Some(root) = std::env::var_os("ACTUATION_STREAM_STORE") {
            return Self::new(PathBuf::from(root));
        }
        let home = std::env::var_os("HOME")
            .ok_or_else(|| Error::new("HOME is unavailable; supply a stream store explicitly"))?;
        Self::new(PathBuf::from(home).join(".actuation/streams"))
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn path(&self, stream_ref: &StreamRef) -> PathBuf {
        self.root.join(stream_file_name(stream_ref))
    }
    fn lock(&self, create: bool, exclusive: bool) -> Result<File> {
        if create {
            io(fs::create_dir_all(&self.root))?;
        }
        let directory = io(File::open(&self.root))?;
        // Current release targets are Unix. The stable directory inode is the
        // lock locus: atomic header rename cannot invalidate the writer lock,
        // and no extra files are introduced into the public store directory.
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
                "native JSONL locking is not implemented on this target",
            ));
        }
        Ok(directory)
    }
    fn file_options(&self) -> OpenOptions {
        let mut options = OpenOptions::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW);
        }
        options
    }
    fn read_unlocked(&self, stream_ref: &StreamRef) -> Result<Option<(ActuationStream, String)>> {
        let path = self.path(stream_ref);
        let mut file = match self.file_options().read(true).open(&path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => {
                return Err(Error::new(format!(
                    "cannot read stream {}: {e}",
                    path.display()
                )))
            }
        };
        let mut raw = String::new();
        io(file.read_to_string(&mut raw))?;
        let stream = fold_stream_file(&raw)?;
        if &stream.fields().stream_ref != stream_ref {
            return Err(Error::new("durable filename and stream_ref disagree"));
        }
        Ok(Some((stream, raw)))
    }
    fn required_unlocked(&self, stream_ref: &StreamRef) -> Result<(ActuationStream, String)> {
        self.read_unlocked(stream_ref)?.ok_or_else(|| {
            Error::new(format!(
                "no durable ActuationStream named {stream_ref} under {}",
                self.root.display()
            ))
        })
    }
    fn rewrite_unlocked(&self, stream_ref: &StreamRef, raw: &str, directory: &File) -> Result<()> {
        let mut temporary = io(tempfile::NamedTempFile::new_in(&self.root))?;
        io(temporary.write_all(raw.as_bytes()))?;
        io(temporary.as_file().sync_all())?;
        temporary
            .persist(self.path(stream_ref))
            .map_err(|e| Error::new(e.to_string()))?;
        io(directory.sync_all())
    }
    fn open_unlocked(&self, opening: &OpenStream, directory: &File) -> Result<ActuationStream> {
        if let Some((existing, _)) = self.read_unlocked(&opening.stream_ref)? {
            IdentityHint::from(opening).check(&existing)?;
            return Ok(existing);
        }
        let stream = opening.empty()?;
        self.rewrite_unlocked(&opening.stream_ref, &encoded_stream(&stream)?, directory)?;
        Ok(stream)
    }
    fn ensure_unlocked(
        &self,
        stream_ref: &StreamRef,
        hint: Option<&IdentityHint>,
        directory: &File,
    ) -> Result<(ActuationStream, String)> {
        if let Some((stream, raw)) = self.read_unlocked(stream_ref)? {
            if let Some(hint) = hint {
                hint.check(&stream)?;
            }
            return Ok((stream, raw));
        }
        let opening = hint
            .ok_or_else(|| {
                Error::new(
                    "stream does not exist; supply its identity to open it with this occurrence",
                )
            })?
            .opening(stream_ref.clone())?;
        let stream = self.open_unlocked(&opening, directory)?;
        let raw = encoded_stream(&stream)?;
        Ok((stream, raw))
    }
    fn append_unlocked(
        &self,
        stream: &ActuationStream,
        raw: &str,
        event: StreamEvent,
    ) -> Result<ActuationStream> {
        let next = stream.append(event.clone())?;
        // A valid final JSON record without a newline is readable under the
        // existing fold law. Add a separator before the new record, never
        // concatenate two JSON objects or repair an invalid/torn tail.
        let mut line = if raw.ends_with('\n') {
            String::new()
        } else {
            String::from("\n")
        };
        line.push_str(&serde_json::to_string(&event)?);
        line.push('\n');
        let mut file = io(self
            .file_options()
            .append(true)
            .open(self.path(&stream.fields().stream_ref)))?;
        io(file.write_all(line.as_bytes()))?;
        io(file.sync_data())?;
        Ok(next)
    }
    pub fn replay(&self, stream_ref: &StreamRef, request: PageRequest) -> Result<StreamPage> {
        Ok(self.load(stream_ref)?.read(request))
    }
    pub fn record_boundary(
        &self,
        occurrence: BoundaryOccurrence,
        catalogue: &dyn BoundaryCatalogue,
    ) -> Result<OccurrenceReceipt> {
        let declaration = catalogue.boundary(
            occurrence.harness.as_str(),
            occurrence.native_event.as_str(),
        )?;
        if declaration.harness != occurrence.harness
            || declaration.native_event != occurrence.native_event
        {
            return Err(Error::new(
                "boundary catalogue returned a different native event identity",
            ));
        }
        let mut metadata = json!({"harness":declaration.harness,"native_event":declaration.native_event,"boundary":declaration.boundary,"catalog_revision":declaration.catalog_revision});
        if let Some(extra) = occurrence.metadata {
            for (key, value) in extra {
                // Preserve the existing wire operation's caller-annotation
                // precedence. Opaque metadata never binds semantic identity or
                // grants authority; those checks use the admitted refs/ports.
                metadata[key] = value;
            }
        }
        let directory = self.lock(occurrence.identity.is_some(), true)?;
        let (stream, raw) = self.ensure_unlocked(
            &occurrence.stream_ref,
            occurrence.identity.as_ref(),
            &directory,
        )?;
        let event = StreamEvent::new(StreamEventFields {
            event_ref: occurrence
                .event_ref
                .map(Ok)
                .unwrap_or_else(event_identity)?,
            sequence: stream.fields().cursor.fields().next_sequence,
            kind: EventKind::HarnessEvent,
            custom_kind: Slot::Absent,
            observed_at: slot(occurrence.observed_at),
            actor: Slot::Absent,
            surface_ref: Slot::Absent,
            execution_ref: Slot::Absent,
            return_ref: Slot::Absent,
            native_trace_ref: slot(occurrence.native_trace_ref),
            resource_refs: Slot::Absent,
            evidence_refs: Slot::Absent,
            disclosure: Slot::Absent,
            content: slot(occurrence.content),
            metadata: Slot::Value(metadata.as_object().expect("metadata object").clone()),
            model_usage: Slot::Absent,
            extensions: Extensions::new(),
        })?;
        let next = self.append_unlocked(&stream, &raw, event.clone())?;
        Ok(OccurrenceReceipt::new(&next, event, None))
    }
    pub fn record_usage(&self, occurrence: UsageOccurrence) -> Result<OccurrenceReceipt> {
        let directory = self.lock(occurrence.identity.is_some(), true)?;
        let (stream, raw) = self.ensure_unlocked(
            &occurrence.stream_ref,
            occurrence.identity.as_ref(),
            &directory,
        )?;
        let usage = &occurrence.observation;
        let u = usage.fields();
        let s = stream.fields();
        let c = u.correlation.fields();
        if s.actuation_ref != u.actuation_ref
            || c.agency_ref.value().is_some_and(|r| r != &s.agency_ref)
            || c.agent_session_ref
                .value()
                .is_some_and(|r| r != &s.agent_session_ref)
        {
            return Err(Error::new(
                "usage correlation does not match the actual stream identity",
            ));
        }
        if let Some(existing) = stream.usage(&u.usage_ref) {
            if existing.fields().model_usage.value() != Some(usage) {
                return Err(Error::new(
                    "usage_ref was replayed with conflicting evidence",
                ));
            }
            return Ok(OccurrenceReceipt::new(
                &stream,
                existing.clone(),
                Some(true),
            ));
        }
        let provenance = u.provenance.fields();
        let event = StreamEvent::new(StreamEventFields {
            event_ref: occurrence
                .event_ref
                .map(Ok)
                .unwrap_or_else(event_identity)?,
            sequence: s.cursor.fields().next_sequence,
            kind: EventKind::ModelUsage,
            custom_kind: Slot::Absent,
            observed_at: Slot::Value(provenance.observed_at.clone()),
            native_trace_ref: Slot::Value(provenance.raw_evidence_refs.first().clone()),
            evidence_refs: Slot::Value(provenance.raw_evidence_refs.as_slice().to_vec()),
            disclosure: Slot::Value(Disclosure::Portable),
            model_usage: Slot::Value(usage.clone()),
            actor: Slot::Absent,
            surface_ref: Slot::Absent,
            execution_ref: Slot::Absent,
            return_ref: Slot::Absent,
            resource_refs: Slot::Absent,
            content: Slot::Absent,
            metadata: Slot::Absent,
            extensions: Extensions::new(),
        })?;
        let next = self.append_unlocked(&stream, &raw, event.clone())?;
        Ok(OccurrenceReceipt::new(&next, event, Some(false)))
    }
}
impl StreamStore for JsonlStreamStore {
    fn open(&self, opening: &OpenStream) -> Result<ActuationStream> {
        let directory = self.lock(true, true)?;
        self.open_unlocked(opening, &directory)
    }
    fn load(&self, stream_ref: &StreamRef) -> Result<ActuationStream> {
        let _directory = self.lock(false, false)?;
        Ok(self.required_unlocked(stream_ref)?.0)
    }
    fn append(&self, stream_ref: &StreamRef, event: StreamEvent) -> Result<ActuationStream> {
        let _directory = self.lock(false, true)?;
        let (stream, raw) = self.required_unlocked(stream_ref)?;
        self.append_unlocked(&stream, &raw, event)
    }
    fn close(
        &self,
        stream_ref: &StreamRef,
        state: TerminalState,
        ended_at: Timestamp,
    ) -> Result<ActuationStream> {
        let directory = self.lock(false, true)?;
        let (stream, raw) = self.required_unlocked(stream_ref)?;
        let closed = stream.close(state, ended_at)?;
        // Lifecycle changes only the header. Preserve existing event bytes,
        // including native extension member spelling/order, across the rename.
        let mut rewritten = serde_json::to_string(&closed.header())?;
        rewritten.push('\n');
        if let Some((_, events)) = raw.split_once('\n') {
            rewritten.push_str(events);
        }
        self.rewrite_unlocked(stream_ref, &rewritten, &directory)?;
        Ok(closed)
    }
}
