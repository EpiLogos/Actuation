use crate::*;
use std::{collections::BTreeMap, sync::mpsc};

#[derive(Clone, Debug)]
pub struct JournalOccurrence {
    pub event: StreamEvent,
    pub snapshot: StreamPage,
}
/// In-memory replay/subscription to the same admitted stream, not a daemon or
/// a second persistent event source. Dropped subscriptions lose no canonical
/// history: a later subscription can replay from its own cursor.
pub struct ActuationStreamJournal {
    stream: ActuationStream,
    subscribers: BTreeMap<u64, mpsc::Sender<JournalOccurrence>>,
    next_subscription: u64,
}
impl ActuationStreamJournal {
    pub fn new(stream: ActuationStream) -> Self {
        Self {
            stream,
            subscribers: BTreeMap::new(),
            next_subscription: 0,
        }
    }
    pub fn snapshot(&self, request: PageRequest) -> StreamPage {
        self.stream.read(request)
    }
    pub fn stream(&self) -> &ActuationStream {
        &self.stream
    }
    pub fn append(&mut self, event: StreamEvent) -> Result<StreamEvent> {
        self.stream = self.stream.append(event.clone())?;
        let item = JournalOccurrence {
            event: event.clone(),
            snapshot: self.snapshot(PageRequest::default()),
        };
        self.subscribers.retain(|_, s| s.send(item.clone()).is_ok());
        Ok(event)
    }
    pub fn close(&mut self, state: TerminalState, ended_at: Timestamp) -> Result<StreamPage> {
        self.stream = self.stream.close(state, ended_at)?;
        Ok(self.snapshot(PageRequest::default()))
    }
    pub fn subscribe(
        &mut self,
        after_sequence: Option<Count>,
        replay: bool,
    ) -> Result<(u64, mpsc::Receiver<JournalOccurrence>)> {
        let after = after_sequence.unwrap_or(self.stream.fields().cursor.fields().last_sequence);
        let (tx, rx) = mpsc::channel();
        if replay {
            let snapshot = self.snapshot(PageRequest::default());
            for event in self
                .stream
                .fields()
                .events
                .iter()
                .filter(|e| e.fields().sequence > after)
            {
                tx.send(JournalOccurrence {
                    event: event.clone(),
                    snapshot: snapshot.clone(),
                })
                .map_err(|e| Error::new(e.to_string()))?;
            }
        }
        let id = self.next_subscription;
        self.next_subscription = self
            .next_subscription
            .checked_add(1)
            .ok_or_else(|| Error::new("subscription identity space exhausted"))?;
        self.subscribers.insert(id, tx);
        Ok((id, rx))
    }
    pub fn unsubscribe(&mut self, id: u64) -> bool {
        self.subscribers.remove(&id).is_some()
    }
}
