use crate::{records::EpistemicRecord, world::World, Error, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};
pub fn filename(reference: &str) -> String {
    let mut result = String::new();
    for b in reference.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) {
            result.push(b as char)
        } else {
            result.push_str(&format!("%{b:02X}"))
        }
    }
    result.push_str(".json");
    result
}
#[derive(Clone, Debug)]
pub struct EpistemicStore {
    world: World,
}
#[derive(Debug, Serialize)]
pub struct PersistedRecord {
    pub record: EpistemicRecord,
    pub deduplicated: bool,
    pub pathname: PathBuf,
}
impl EpistemicStore {
    /// The caller supplies the dedicated store directory. Creating ancestors is
    /// a caller operation, not implicit authority over another existing World.
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            world: World::open(root)?,
        })
    }
    pub fn read(&self, reference: &str) -> Result<EpistemicRecord> {
        let bytes = self.world.read(&filename(reference))?;
        let record = EpistemicRecord::read(
            &serde_json::from_slice(&bytes)
                .map_err(|_| Error::new("stored record is invalid JSON"))?,
        )?;
        if record.record_ref() != reference {
            return Err(Error::new("stored record identity mismatch"));
        }
        Ok(record)
    }
    pub fn persist(&self, record: EpistemicRecord) -> Result<PersistedRecord> {
        let name = filename(record.record_ref());
        let pathname = self.world.root().join(&name);
        let mut bytes =
            serde_json::to_vec_pretty(&record).map_err(|e| Error::new(e.to_string()))?;
        bytes.push(b'\n');
        if self.world.write(&name, &bytes, true)? {
            return Ok(PersistedRecord {
                record,
                deduplicated: false,
                pathname,
            });
        }
        let existing = self.read(record.record_ref())?;
        if existing != record {
            return Err(Error::new("conflicting epistemic record_ref"));
        }
        Ok(PersistedRecord {
            record: existing,
            deduplicated: true,
            pathname,
        })
    }
    pub fn list(&self) -> Result<Vec<EpistemicRecord>> {
        let mut records = Vec::new();
        for entry in self.world.list(".")? {
            let name = entry["name"].as_str().unwrap();
            let Some(stem) = name.strip_suffix(".json") else {
                continue;
            };
            let mut bytes = Vec::new();
            let mut iter = stem.bytes();
            while let Some(b) = iter.next() {
                if b == b'%' {
                    let a = iter.next().and_then(|b| (b as char).to_digit(16));
                    let c = iter.next().and_then(|b| (b as char).to_digit(16));
                    match (a, c) {
                        (Some(a), Some(c)) => bytes.push((a * 16 + c) as u8),
                        _ => return Err(Error::new("malformed encoded record ref")),
                    }
                } else {
                    bytes.push(b)
                }
            }
            let reference = String::from_utf8(bytes)
                .map_err(|_| Error::new("invalid UTF8 record reference"))?;
            if filename(&reference) != name {
                return Err(Error::new("noncanonical record filename"));
            }
            records.push(self.read(&reference)?);
        }
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn corpus_record(reference: &str, revision: &str) -> EpistemicRecord {
        EpistemicRecord::read(&json!({
            "schema": crate::records::EPISTEMIC_RECORD_SCHEMA,
            "record_kind": "EpistemicCorpus",
            "record_ref": reference,
            "recorded_at": "2026-09-11T00:00:00Z",
            "provenance": {"kind":"source","actor_ref":null,"generator_ref":null,"source_refs":["source:1"],"derivation_refs":[]},
            "access": {
                "behavioural": {"state":"available","method_refs":["m:1"],"evidence_refs":["e:1"]},
                "output_state": {"state":"available","method_refs":["m:1"],"evidence_refs":["e:1"]},
                "internal_read": {"state":"unavailable","reason":"no interior access","method_refs":[],"evidence_refs":[]},
                "internal_write": {"state":"unavailable","reason":"no interior access","method_refs":[],"evidence_refs":[]},
                "causal": {"state":"unavailable","reason":"not established","method_refs":[],"evidence_refs":[]},
                "learning": {"state":"not-assessed","reason":"assessment not attempted","method_refs":[],"evidence_refs":[]}
            },
            "subject_refs": ["subject:1"],
            "model_ref": null,
            "checkpoint_ref": null,
            "method_refs": [],
            "coordinate_refs": [],
            "derivation_refs": [],
            "evidence_refs": [],
            "payload": {"corpus_ref":"corpus:1","revision_ref":revision,"item_refs":["item:1"]}
        }))
        .expect("valid corpus record")
    }

    #[test]
    fn filename_encodes_portably_and_round_trips() {
        assert_eq!(filename("plain-ref.1"), "plain-ref.1.json");
        assert_eq!(filename("a/b:c"), "a%2Fb%3Ac.json");
        for (reference, name) in [("x", "x.json"), ("a b", "a%20b.json")] {
            assert_eq!(filename(reference), name);
            let mut decoded = Vec::new();
            let stem = name.strip_suffix(".json").unwrap();
            let mut chars = stem.bytes();
            while let Some(b) = chars.next() {
                if b == b'%' {
                    let a = chars.next().unwrap();
                    let c = chars.next().unwrap();
                    decoded.push(
                        ((a as char).to_digit(16).unwrap() * 16 + (c as char).to_digit(16).unwrap())
                            as u8,
                    );
                } else {
                    decoded.push(b);
                }
            }
            assert_eq!(String::from_utf8(decoded).unwrap(), reference);
        }
    }

    #[test]
    fn persist_read_list_and_dedup_round_trip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = EpistemicStore::open(dir.path()).expect("store");
        let record = corpus_record("record:round/1", "rev:1");
        let persisted = store.persist(record.clone()).expect("persist");
        assert!(!persisted.deduplicated);
        assert_eq!(store.read("record:round/1").expect("read"), record);
        let again = store.persist(record.clone()).expect("repersist");
        assert!(again.deduplicated);
        let listed = store.list().expect("list");
        assert_eq!(listed, vec![record]);
    }

    #[test]
    fn conflicting_content_under_one_reference_is_refused() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = EpistemicStore::open(dir.path()).expect("store");
        store
            .persist(corpus_record("record:conflict/1", "rev:1"))
            .expect("persist");
        let conflicting = corpus_record("record:conflict/1", "rev:2");
        assert!(store.persist(conflicting).is_err());
    }

    #[test]
    fn stored_identity_mismatch_is_refused() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = EpistemicStore::open(dir.path()).expect("store");
        store
            .persist(corpus_record("record:identity/1", "rev:1"))
            .expect("persist");
        let path = dir.path().join("record%3Aidentity%2F1.json");
        let mut bytes = std::fs::read(&path).expect("stored bytes");
        let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        v["record_ref"] = json!("record:other");
        bytes = serde_json::to_vec_pretty(&v).unwrap();
        std::fs::write(&path, bytes).unwrap();
        assert!(store.read("record:identity/1").is_err());
    }
}
