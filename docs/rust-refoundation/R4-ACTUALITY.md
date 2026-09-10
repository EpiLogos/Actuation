# R4 — Attributable actuality and durable continuity

Serial PR D of Actuation #58. Basis: accepted and reverified main
`6398b23e0972e85d61295ddc1cfcbb83989dc00b` (R3 / #61), tree
`801bd048e84c494b1c617b580bd46026e092c5cf`.

Before R4 began, merged-main Rust refoundation workflow `34437406382`
and documentation workflow `34437406407` were inspected as successful.
Artifact `10136670878` (ZIP SHA-256
`5e846c93d8439e8b20309f92cf4b96f906075ff14edd965b9acde4384f4f252f`)
was independently unpacked as its exact Git bundle and reverified. No other
Actuation migration PR was open. No R5 implementation begins before this
tranche is verified, merged, and its new main re-read and verified.

## What this changes

Actuation now owns the native library through which an actual acting relation
can preserve its attributable unfolding. A Stream is not an AgentSession,
a Factory Run, a Workcell log, or a UI conversation. Its own identity and
ordered events let those distinct owner relations encounter the same actual
difference without manufacturing common ancestry.

`actuation-stream` depends on the constitutional core. `actuation-runtime`
consumes that Stream surface, not vice versa. The core acquires no I/O, clock,
provider or peer-product dependency. No daemon, model selection, ACP host,
material provisioning or Recognition service has been introduced.

The admitted native domain includes Stream/Event, Lifecycle/Cursor, Activity,
Attribution/Trace, model-usage observations and request/authority correlations.
These are immutable validated records with nominal refs and closed enums for
closed vocabularies. Supplied absence, explicit null and known values remain
distinct. Open wire extensions retain the accepted validator behavior; they
cannot shadow typed fields. The unchanged JSON schemas are not retrospectively
rewritten to erase pre-existing schema/validator differences.

`StreamStore` is the persistence port. `JsonlStreamStore` implements the current
format. `ActuationStreamJournal` provides in-memory replay/subscription over
the same canonical stream. `StreamRuntimeObserver` is the production connection
from the ordinary acting loop: it preserves each disclosed callback under
`metadata.runtime_event` and returns an evidence ref only after a successful
append. The callback's opaque `run_id` remains an execution-trace ref, not a
Factory RunRef. Its timestamp is explicitly observer reception, not a claim
about provider timing. Unsupported/unrecorded callbacks cannot become evidence.

Activity projects observed semantic unfolding, including native occurrences
without a canonical Action. Direct activity has no invented Plan/Journey/Run.
When those refs are supplied, they stay separate correlations. Return events
preserve an explicit Return ref and raw attributable difference; neither stream
closure nor Activity completion performs Recognition or source mutation.

Correlation reads precisely the corpus provided. An omitted side is unqueried;
a supplied empty side is queried but empty; explicit null is not a substitute.
The native four-sided read preserves unknown caller state, authority-decision
ordering, Activity joins and pending-permission disposition without granting
new authority.

## Persistence law and compatibility

No new on-disk format or schema version was introduced. The filename remains
ECMAScript-compatible percent encoding of `stream_ref`, followed by `.jsonl`.
The first line holds identity/lifecycle; event lines own ordering. Header cursor
and embedded events are derived state and are recomputed from the actual lines.
Torn JSON, holes, gaps, duplicate event refs and invalid portable material fail
closed. Reads and refusals do not repair or truncate files.

Native append preserves prior bytes and synchronizes event material. Closing
atomically replaces the header while retaining the original event-tail bytes.
Open is identity-checked/idempotent. Identical usage replay is idempotent even
after closure; conflicting reuse of a usage ref is refused and does not reopen
the act. Known usage Agency/Session/Actuation correlations must match the stream.

On the supported Unix targets, native operations lock the stable store-directory
inode, which survives the lifecycle rename. This gives independently opened
native writers one contiguous sequence without adding lock files to the public
store. The original Node writer does not participate in these locks: the
cross-implementation proof is deliberately **serial interoperability**, not a
claim about concurrently mixing Node and Rust writers.

Two store safeguards are explicit: a requested filename cannot substitute a
file with another embedded stream identity, and stream-file symlinks are
refused. The configured root may still be an ordinary user-selected directory.
These safeguards do not create Agent identity from filesystem proximity.

One native append correction follows directly from the accepted fold law:
a complete last JSON record without a final newline is readable. The old Node
append concatenates the next object to it and then cannot fold its own output;
this was reproduced against the locked source in an injected temporary store.
Rust retains the existing bytes and inserts a separator before appending. It
never does this for an invalid/torn record. This is not a reported production
incident, a new file format, or an automatic corruption-repair operation.

The public boundary-recording operation retains its existing opaque caller
metadata precedence. Metadata does not supply semantic identity or authority.
The observation-catalogue port is separately checked for the requested target
and native-event identity. Production catalogue data remains R5 work; test
catalogues consume pinned R1 descriptor facts.

## Migration destinations and remaining ownership

| Accepted source behavior | Native destination in this tranche |
| --- | --- |
| Stream validation, append, read/cursor, close | `actuation-stream`: Stream/Event/Lifecycle/Cursor |
| In-memory replay/subscription | `ActuationStreamJournal` |
| Existing JSONL lifecycle, occurrence and usage store | `StreamStore`, `JsonlStreamStore` |
| Activity validation and projections | `Activity`, `ActivityDescription`, `Attribution`, `ActivityTrace` |
| Model-usage wire admission and dedup facts | Native usage records and `JsonlStreamStore::record_usage` |
| Request/authority/Activity/Stream joining | `AuthorityDecision`, `CorrelationCorpus` |
| Actual ordinary-loop event persistence | `actuation-runtime::StreamRuntimeObserver` |

The frozen R1 R4 selection contains two native-usage codecs as well as the usage
contract. Their pure Claude/Codex translations are temporarily in
`actuation-stream::usage_native` to prove that complete selection here. R5 moves
this module to `actuation-adapters`; it must not leave duplicate codec owners.
No provider or host is executed by either codec.

Original MJS product files are unchanged and remain served until R7. They are
migration oracles/pending cutover, **not** specimen-native exemptions. Both
research fields remain active and untouched; R6, not this tranche, performs
their complete A/B/C/D consumer refoundation. QL stays number-neutral at this
layer and formally owned by QL-MEF.

## Executed candidate verification

The authoring environment exercised:

- formatting, locked workspace/all-target Clippy with warnings denied;
- 59 native integration tests plus one compile-fail identity doctest;
- 119 R2 and 44 R3 public parity cases, plus 29 original loop extraction cases;
- all 213 frozen R4 public contract cases and 21 frozen store/fold/filename scenarios;
- eight exact Node-created JSONL snapshots, extracted from immutable R1 data;
- 57 fresh interoperation assertions over 27 Rust processes: Node writes/Rust
  reads, Rust writes/Node reads, both extending each other's files, portable
  replay, terminal/dedup refusal, and corruption preserving evidence;
- independent native writer concurrency and adversarial identity/order/disclosure;
- original Node 219 native tests, all 599 pure/67 scenario cases, 84 research
  tests and native Skills verification.

`verify-native.sh` runs these native and cross-reader gates on maintained
read-only Linux/macOS CI. `verify-stream-corpus.mjs` verifies exact byte
extraction and checksums against R1, not a recaptured expected result. The
original R1 ledger/corpus hashes are unchanged. Test-oracle example executables
are transport adapters only; product semantics reside in the native libraries.

These are D evidence. They do not prove a live provider, the owner's machine,
GUI/human acceptance or physical research success. Final PR checks and the
subsequent main-push verification must be recorded before beginning R5.
