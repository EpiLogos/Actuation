use crate::{LoopEvent, RuntimeObserver};
use actuation_core::*;
use actuation_stream::*;
use serde_json::json;

/// Preserve actual loop callbacks in the canonical ActuationStream. The loop's
/// opaque run_id is an execution trace correlation, never a Factory RunRef.
/// This observer neither creates an Agency nor provisions a host/body.
///
/// Each supplied callback is disclosed portable runtime material, retained
/// verbatim under metadata.runtime_event. Provider-private material is not
/// acquired by this port. Hosts must supply only material they may disclose.
/// Failure to persist means no evidence reference is returned to the runtime.
pub struct StreamRuntimeObserver<S, C = fn() -> Result<Timestamp>> {
    store: S,
    stream_ref: StreamRef,
    actuation_ref: ActuationRef,
    actor: Attribution,
    clock: C,
}
impl<S: StreamStore> StreamRuntimeObserver<S> {
    pub fn new(
        store: S,
        stream_ref: StreamRef,
        actuation_ref: ActuationRef,
        actor: Attribution,
    ) -> Self {
        Self {
            store,
            stream_ref,
            actuation_ref,
            actor,
            clock: Timestamp::now,
        }
    }
}
impl<S: StreamStore, C: FnMut() -> Result<Timestamp> + Send> StreamRuntimeObserver<S, C> {
    /// The clock witnesses reception of the callback, not provider timing.
    pub fn with_clock(
        store: S,
        stream_ref: StreamRef,
        actuation_ref: ActuationRef,
        actor: Attribution,
        clock: C,
    ) -> Self {
        Self {
            store,
            stream_ref,
            actuation_ref,
            actor,
            clock,
        }
    }
    pub fn store(&self) -> &S {
        &self.store
    }
}
impl<S: StreamStore, C: FnMut() -> Result<Timestamp> + Send> RuntimeObserver
    for StreamRuntimeObserver<S, C>
{
    fn emit(&mut self, event: &LoopEvent) -> Result<ExternalRef> {
        let stream = self.store.load(&self.stream_ref)?;
        if stream.fields().actuation_ref != self.actuation_ref {
            return Err(Error::new(
                "runtime observer cannot substitute Actuation identity",
            ));
        }
        let event_ref = EventRef::new(event.event_id.clone())?;
        let portable = StreamEvent::new(StreamEventFields {
            event_ref: event_ref.clone(),
            sequence: stream.fields().cursor.fields().next_sequence,
            kind: EventKind::ExecutionEvent,
            custom_kind: Slot::Absent,
            observed_at: Slot::Value((self.clock)()?),
            actor: Slot::Value(self.actor.clone()),
            execution_ref: Slot::Value(event.run_id.clone()),
            surface_ref: Slot::Absent,
            return_ref: Slot::Absent,
            native_trace_ref: Slot::Absent,
            resource_refs: Slot::Absent,
            evidence_refs: Slot::Absent,
            disclosure: Slot::Value(Disclosure::Portable),
            content: Slot::Absent,
            model_usage: Slot::Absent,
            metadata: Slot::Value(
                json!({"runtime_event":event,"timestamp_basis":"observer-reception"})
                    .as_object()
                    .expect("object")
                    .clone(),
            ),
            extensions: Extensions::new(),
        })?;
        // Store append checks the cursor again under its write lock. A competing
        // writer yields a visible refusal; no callback is acknowledged then lost.
        self.store.append(&self.stream_ref, portable)?;
        ExternalRef::new(event_ref.as_str())
    }
}
