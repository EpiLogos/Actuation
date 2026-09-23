//! World Position occupancy: which occupant generation holds a stable address,
//! and the tenure records that testify to it.
//!
//! Three things stay distinct here:
//!
//! - the **World Position** ([`WorldPositionRef`]) — a durable address defined
//!   in Central's ground. It survives every change of Agent, Agency,
//!   AgentSession, SessionSpace, model, harness and Workcell placement;
//! - the **occupant generation** ([`OccupantGenerationRef`]) — one tenure of
//!   that address, minted fresh for every occupancy and never reused. It is
//!   not an AgentSession: one session may hold several generations over time
//!   and a generation may outlive a session restart;
//! - the **occupant** — an existing Agent acting through an existing Agency,
//!   named by ref. Occupancy never mints an Agent identity.
//!
//! A World Position is also not an `AgenticLocus`: a locus is a participation
//! position inside one composition (docs/ACTUATION-RELATION.md §3), whereas a
//! Position is an address that outlives every composition occupying it.
//!
//! The current occupant is the single open tenure. Any other shape of the
//! record is an ambiguity, never resolved by picking the newest.
use crate::{AgencyRef, AgentRef, AgentSessionRef, Error, ExternalRef, Result, WorldPositionRef};
use serde::{Deserialize, Deserializer, Serialize};
use std::{fmt, str::FromStr};

pub const POSITION_TENURE_VERSION: &str = "actuation.position-tenure/v1";
pub const POSITION_OCCUPANCY_VERSION: &str = "actuation.position-occupancy/v1";
pub const OCCUPANT_GENERATION_PREFIX: &str = "actuation:generation:";

/// `actuation:generation:<uuid>` — the identity of one tenure. The UUID is a
/// canonical lowercase RFC 4122 rendering; anything else is refused, so a
/// generation can never be confused with a session, pane or process id.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct OccupantGenerationRef(String);

impl OccupantGenerationRef {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let uuid = value.strip_prefix(OCCUPANT_GENERATION_PREFIX).ok_or_else(|| {
            Error::new(format!(
                "occupant generation {value:?} must have the form {OCCUPANT_GENERATION_PREFIX}<uuid>"
            ))
        })?;
        if !is_canonical_uuid(uuid) {
            return Err(Error::new(format!(
                "occupant generation {value:?} does not carry a canonical lowercase uuid"
            )));
        }
        Ok(Self(value))
    }

    /// Render sixteen random bytes as a version-4 generation ref. The caller
    /// supplies the entropy: this crate performs no I/O.
    pub fn from_random_bytes(mut bytes: [u8; 16]) -> Self {
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        Self(format!(
            "{OCCUPANT_GENERATION_PREFIX}{}-{}-{}-{}-{}",
            &hex[0..8],
            &hex[8..12],
            &hex[12..16],
            &hex[16..20],
            &hex[20..32]
        ))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn uuid(&self) -> &str {
        &self.0[OCCUPANT_GENERATION_PREFIX.len()..]
    }
}

fn is_canonical_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => *byte == b'-',
            _ => byte.is_ascii_digit() || (b'a'..=b'f').contains(byte),
        })
}

impl fmt::Display for OccupantGenerationRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl AsRef<str> for OccupantGenerationRef {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl FromStr for OccupantGenerationRef {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self> {
        Self::new(value)
    }
}
impl<'de> Deserialize<'de> for OccupantGenerationRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// One occupant generation of one Position: its globally unique ref and its
/// per-Position ordinal (1 for the first tenure, then max + 1).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OccupantGeneration {
    pub generation_ref: OccupantGenerationRef,
    pub ordinal: u64,
}

macro_rules! word_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident => $word:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
        pub enum $name {
            $(#[serde(rename = $word)] $variant),+
        }
        impl $name {
            pub const WORDS: &'static [&'static str] = &[$($word),+];
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $word),+ }
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
        impl FromStr for $name {
            type Err = Error;
            fn from_str(value: &str) -> Result<Self> {
                match value {
                    $($word => Ok(Self::$variant),)+
                    other => Err(Error::new(format!(
                        concat!(stringify!($name), " {:?} is not one of {}"),
                        other,
                        Self::WORDS.join("|")
                    ))),
                }
            }
        }
    };
}

word_enum!(
    /// How a tenure began. `initial` is the first tenure a Position ever has;
    /// `handover` supersedes a live predecessor; `fresh` starts a new body on
    /// a Position that has held others before (or supersedes one without
    /// carrying its context); `adopt` binds an already-running body.
    TenureKind {
        Initial => "initial",
        Handover => "handover",
        Fresh => "fresh",
        Adopt => "adopt",
    }
);

word_enum!(
    /// How a tenure ended: released (the Position became vacant) or superseded
    /// (a successor tenure began in the same locked write).
    TenureEndKind {
        Released => "released",
        Superseded => "superseded",
    }
);

word_enum!(
    /// Presence of the current occupant, as it reports itself.
    Presence {
        Active => "active",
        Idle => "idle",
        Away => "away",
        Offline => "offline",
    }
);

word_enum!(
    /// Derived from the tenure record: exactly one open tenure is `occupied`,
    /// none is `vacant`. More than one is an ambiguity refusal, not a state.
    OccupancyState {
        Occupied => "occupied",
        Vacant => "vacant",
    }
);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TenureSchema {
    #[serde(rename = "actuation.position-tenure/v1")]
    V1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum OccupancySchema {
    #[serde(rename = "actuation.position-occupancy/v1")]
    V1,
}

/// `actuation.position-tenure/v1`: one occupant generation's hold on one
/// Position. Open while `ended_at_unix_ms` is absent.
///
/// A predecessor's tenure stays readable as testimony. It grants its
/// successor no authority: the successor's standing comes from its own claim.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tenure {
    pub schema: TenureSchema,
    pub position_ref: WorldPositionRef,
    pub generation_ref: OccupantGenerationRef,
    pub generation_ordinal: u64,
    pub kind: TenureKind,
    pub agent_ref: AgentRef,
    pub agency_ref: AgencyRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_session_ref: Option<AgentSessionRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_space_ref: Option<ExternalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness_composition_ref: Option<ExternalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model_ref: Option<ExternalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workcell_ref: Option<ExternalRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gateway_address: Option<String>,
    pub began_at_unix_ms: u64,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predecessor_generation_ref: Option<OccupantGenerationRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_at_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_kind: Option<TenureEndKind>,
    /// Why the tenure ended: the release reason, or the successor's claim
    /// reason for a supersession. Present exactly when the tenure has ended.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_reason: Option<String>,
}

impl Tenure {
    pub fn is_open(&self) -> bool {
        self.ended_at_unix_ms.is_none()
    }

    pub fn generation(&self) -> OccupantGeneration {
        OccupantGeneration {
            generation_ref: self.generation_ref.clone(),
            ordinal: self.generation_ordinal,
        }
    }

    /// The record's own invariants: a real ordinal, a stated reason, a
    /// non-empty gateway address when one is named, and end facts that are
    /// all present or all absent.
    pub fn check(&self) -> Result<()> {
        if self.generation_ordinal == 0 {
            return Err(Error::new("tenure generation_ordinal starts at 1"));
        }
        if crate::is_blank_reference(&self.reason) {
            return Err(Error::new("tenure reason must be stated"));
        }
        if self
            .gateway_address
            .as_deref()
            .is_some_and(crate::is_blank_reference)
        {
            return Err(Error::new(
                "tenure gateway_address, when named, is non-empty",
            ));
        }
        if self.predecessor_generation_ref.as_ref() == Some(&self.generation_ref) {
            return Err(Error::new("a tenure cannot be its own predecessor"));
        }
        let ended = [
            self.ended_at_unix_ms.is_some(),
            self.end_kind.is_some(),
            self.end_reason.is_some(),
        ];
        if ended.iter().any(|x| *x) && !ended.iter().all(|x| *x) {
            return Err(Error::new(
                "tenure end facts (ended_at_unix_ms, end_kind, end_reason) are all present or all absent",
            ));
        }
        if self
            .ended_at_unix_ms
            .is_some_and(|ended| ended < self.began_at_unix_ms)
        {
            return Err(Error::new("a tenure cannot end before it began"));
        }
        Ok(())
    }
}

/// The latest presence the current occupant reported about itself.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OccupantPresence {
    pub generation_ref: OccupantGenerationRef,
    pub presence: Presence,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attention: Option<String>,
    pub at_unix_ms: u64,
}

/// `actuation.position-occupancy/v1`: the derived occupancy of one Position.
///
/// `current` is the single open tenure (present exactly when `occupied`).
/// `predecessor` is the tenure held immediately before the current one, or
/// the last one held when the Position is vacant. `generations` is the whole
/// tenure history in ordinal order, uncapped.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OccupancyReading {
    pub schema: OccupancySchema,
    pub position_ref: WorldPositionRef,
    pub state: OccupancyState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<Tenure>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predecessor: Option<Tenure>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence: Option<OccupantPresence>,
    pub generations: Vec<Tenure>,
}

impl OccupancyReading {
    /// Derive a reading from a complete tenure history (ordinal order) and
    /// the latest presence per generation. Refuses — with the open tenures
    /// named — unless at most one tenure is open.
    pub fn derive(
        position_ref: WorldPositionRef,
        mut generations: Vec<Tenure>,
        presence_of: impl Fn(&OccupantGenerationRef) -> Option<OccupantPresence>,
    ) -> std::result::Result<Self, Vec<Tenure>> {
        generations.sort_by_key(|tenure| tenure.generation_ordinal);
        let open: Vec<&Tenure> = generations.iter().filter(|t| t.is_open()).collect();
        if open.len() > 1 {
            return Err(open.into_iter().cloned().collect());
        }
        let current = open.first().map(|tenure| (*tenure).clone());
        let predecessor = match &current {
            Some(current) => generations
                .iter()
                .filter(|t| t.generation_ordinal < current.generation_ordinal)
                .max_by_key(|t| t.generation_ordinal)
                .cloned(),
            None => generations.last().cloned(),
        };
        let presence = current
            .as_ref()
            .and_then(|current| presence_of(&current.generation_ref));
        Ok(Self {
            schema: OccupancySchema::V1,
            position_ref,
            state: if current.is_some() {
                OccupancyState::Occupied
            } else {
                OccupancyState::Vacant
            },
            current,
            predecessor,
            presence,
            generations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generation(n: u8) -> OccupantGenerationRef {
        OccupantGenerationRef::from_random_bytes([n; 16])
    }

    fn tenure(ordinal: u64, open: bool) -> Tenure {
        Tenure {
            schema: TenureSchema::V1,
            position_ref: WorldPositionRef::new("central:position:project:O-I:guardian").unwrap(),
            generation_ref: generation(ordinal as u8),
            generation_ordinal: ordinal,
            kind: if ordinal == 1 {
                TenureKind::Initial
            } else {
                TenureKind::Handover
            },
            agent_ref: AgentRef::new("agent/guardian").unwrap(),
            agency_ref: AgencyRef::new("agency:guardian").unwrap(),
            agent_session_ref: None,
            session_space_ref: None,
            harness_composition_ref: None,
            model_ref: None,
            workcell_ref: None,
            gateway_address: None,
            began_at_unix_ms: ordinal * 10,
            reason: "test".into(),
            predecessor_generation_ref: None,
            ended_at_unix_ms: (!open).then_some(ordinal * 10 + 5),
            end_kind: (!open).then_some(TenureEndKind::Superseded),
            end_reason: (!open).then(|| "next".to_owned()),
        }
    }

    #[test]
    fn generation_refs_are_canonical_v4_uuids_under_the_actuation_prefix() {
        let minted = OccupantGenerationRef::from_random_bytes([0xff; 16]);
        assert_eq!(
            minted.as_str(),
            "actuation:generation:ffffffff-ffff-4fff-bfff-ffffffffffff"
        );
        assert_eq!(OccupantGenerationRef::new(minted.as_str()).unwrap(), minted);
        for bad in [
            "",
            "ffffffff-ffff-4fff-bfff-ffffffffffff",
            "actuation:generation:FFFFFFFF-FFFF-4FFF-BFFF-FFFFFFFFFFFF",
            "actuation:generation:not-a-uuid",
            "actuation:session:ffffffff-ffff-4fff-bfff-ffffffffffff",
        ] {
            assert!(OccupantGenerationRef::new(bad).is_err(), "{bad} accepted");
        }
    }

    #[test]
    fn the_current_occupant_is_the_single_open_tenure_and_never_the_newest() {
        let reading = OccupancyReading::derive(
            WorldPositionRef::new("p").unwrap(),
            vec![tenure(2, true), tenure(1, false)],
            |_| None,
        )
        .unwrap();
        assert_eq!(reading.state, OccupancyState::Occupied);
        assert_eq!(reading.current.as_ref().unwrap().generation_ordinal, 2);
        assert_eq!(reading.predecessor.as_ref().unwrap().generation_ordinal, 1);
        assert_eq!(
            reading
                .generations
                .iter()
                .map(|t| t.generation_ordinal)
                .collect::<Vec<_>>(),
            vec![1, 2]
        );

        let open_both = OccupancyReading::derive(
            WorldPositionRef::new("p").unwrap(),
            vec![tenure(1, true), tenure(2, true)],
            |_| None,
        )
        .unwrap_err();
        assert_eq!(open_both.len(), 2, "two open tenures are an ambiguity");

        let vacant = OccupancyReading::derive(
            WorldPositionRef::new("p").unwrap(),
            vec![tenure(1, false), tenure(2, false)],
            |_| None,
        )
        .unwrap();
        assert_eq!(vacant.state, OccupancyState::Vacant);
        assert!(vacant.current.is_none());
        assert_eq!(vacant.predecessor.unwrap().generation_ordinal, 2);
    }

    #[test]
    fn tenure_end_facts_travel_together() {
        let mut broken = tenure(1, false);
        broken.end_reason = None;
        assert!(broken.check().is_err());
        assert!(tenure(1, false).check().is_ok());
        assert!(tenure(1, true).check().is_ok());
        let json = serde_json::to_value(tenure(1, true)).unwrap();
        assert_eq!(json["schema"], "actuation.position-tenure/v1");
        assert!(json.get("ended_at_unix_ms").is_none(), "absent is omitted");
        let mut unknown = json.clone();
        unknown["surprise"] = serde_json::json!(1);
        assert!(serde_json::from_value::<Tenure>(unknown).is_err());
    }
}
