---
name: ql-relational
description: Gives Prime root and child Agencies executable access to the source-locked QL/MEF kernel, MEF/Context-Frame/VĀK readings, QL Wiki refraction, constellation/Return source, and optional source-locked harmonic development. Use when relational structure can materially change investigation, delegation, evidence or Return.
---

# QL relational faculty

This is an **operative faculty**, not a vocabulary pack.

The subject remains the subject. Use QL/MEF to disclose relations of the current task, source, Wiki object, Agency or returned difference; do not rename client objects into QL nouns.

Import the Python-backed skill:

```python
import ql_relational
```

Available calls:

```python
await ql_relational.capabilities()
await ql_relational.kernel_apply("conjugate-address", "qladdr:sixfold@1/direct/P2/d0")
await ql_relational.ananda_m1_2({"schema": "ql.m1.engine/v1", "config": {"event_ref": "...", "subject_coordinate": "#1", "selected_coordinate": "#1-2-0", "revision": "0", "cycle": "0", "tick12": 0, "family": 0, "row12": 0, "col12": 0, "flowering_substage": 0, "lens12": 0, "context_frame": 1, "basis": "chromatic"}})
await ql_relational.mef_lenses()
await ql_relational.context_frames()
await ql_relational.vak_locate("<vak-ref>")
await ql_relational.negotiate("refract")
await ql_relational.wiki_refract(request_dict)
await ql_relational.constellation_contract()
await ql_relational.harmonic_search("3:3", max_matches=8)
await ql_relational.harmonic_snapshot("chromatic")
await ql_relational.spawn_child_cheapest(
    "Inspect this bounded local whole and return the material difference.",
    name="bounded-review",
    use_type="agent-child",
)
handoff = await ql_relational.central_now_handover(
    "bounded contribution",
    "What changed, what remains, and the next native action.",
    actor="prime-child/bounded-review",
    source_refs=["source:..."],
    evidence_refs=["evidence:..."],
    work_refs=[{"repo":"EpiLogos/O-I","branch":"feature/...","worktree_path":"/actual/path"}],
)
await ql_relational.central_now_handoff_read(handoff["data"]["handoff"]["id"])
ql_relational.return_envelope(...)
await ql_relational.agent_message.send("CHILD_QL_OK", receiver_role="parent")
```

## Child-to-parent message

`agent_message.send(text, receiver_role="parent")` delivers one bounded text
message to the parent's encounter channel. The encounter adapter supplies
each session its own message directory; the send is one JSON record there,
correlated to this session's locus digest like every faculty receipt. The
channel carries words, never effects. When the adapter supplied no
directory the send refuses with its reason - report that refusal honestly
in your final answer instead of claiming a send.
```

## Acting relation

For a differentiated child Agency:

1. receive the bounded task **and its relation to the parent task**;
2. inspect the actual local sources first;
3. use QL/MEF/Wiki operations only where they disclose a consequential relation, complement, conjugate, partial whole, Context Frame, traversal, absence, tension or evidence demand;
4. preserve the native source and revision of what was read;
5. return the result **plus what difference it makes to the parent determination**;
6. retain unresolved or contradictory returned difference instead of flattening it.

Prime children inherit this installed Skill from their parent runtime. A child may itself recurse when the live Prime depth configuration admits it.

## Child model selection

Use `spawn_child_cheapest(...)` when a differentiated child is useful and the
task is compatible with cost-first routing. The Skill asks the installed AIKit
binary to resolve the current Project's **CHEAPEST_ELIGIBLE** roster policy.
It then asks Prime's live `rlm.find_models()` catalogue to confirm the exact
provider/native model pair before calling the native child runtime. A missing or
ambiguous match refuses; the Skill never hard-codes a remembered cheap model.

The returned object distinguishes requested policy, AIKit-resolved canonical
Model/route and the Prime-observed child handle/model. Admission is not the
child's answer. Results still arrive through Prime's normal child messaging or
files. The child's private session path is represented only by a SHA-256 digest
for correlation with native faculty receipts.


## NOW handover and worker replacement

When a useful worker must be replaced, use Central's own bounded NOW return
instead of transferring the parent transcript or inventing a handoff file.

`central_now_handover(...)` calls the installed `ctrl` owner through
`projectcentral.now.return`. It requires `CENTRAL_CTRL_BIN` and
`CENTRAL_ROOT`; `CENTRAL_PROJECT` supplies the default Project key. The
record keeps the worker/session ref, source/evidence/preserve refs, and exact
repo/branch/optional-worktree lane claim. Central owns its lifecycle and DAY
rollover.

A replacement worker uses `central_now_handoff_read(id)` to re-read that exact
record from Central's current NOW inspection and continues from the referenced
source/evidence/next action. These calls are optional: absence of Central makes
the continuation faculty unavailable and never blocks ordinary QL work.

Do not put private transcript text, credentials or protected material in the
handoff merely to make it look complete. Keep it pithy: returned difference,
remaining uncertainty, exact refs and the next native action.

## Wiki / constellation

The deterministic Wiki structural contract is the common structural floor. `constellation_contract()` returns that source verbatim with its QL-MEF revision so you can reason from the current A/B/C, D1-D3, whole-anchor, constellation and Return canon instead of inventing another sixfold.

`wiki_refract()` calls the native `ql-wiki-refraction` binary with a `ql-mef/wiki-refraction/v1` request.

## Harmonic field

`harmonic_search()` does not reimplement the musical/harmonic system in Actuation. It searches the exact source-locked QL-MEF checkout:

- accepted source derivation on main; and
- when `QL_PRIME_HARMONIC=1`, the source-locked accepted QL-MEF main containing the executable pre-M music implementation first carried by the historical #81 head.

`harmonic_snapshot()` additionally compiles a temporary path-dependent Rust probe against the exact accepted-main checkout and executes `derive_pre_m_music()` to return the helices, lens anchors, A/B/C and D cross-interval fields, and 84-landscape cardinality as operative data.

The returned path, line and excerpt are evidence. The original #81 pull request closed unmerged, but its head is an ancestor of accepted main through the recorded integration revision; the lock preserves both facts instead of calling accepted code current development.

## Return

Use `return_envelope()` when a child finding changes the parent determination. Its experimental schema is `actuation.prime-return/v0` and carries:

- subject/ref;
- relation to parent;
- determination;
- result;
- returned difference;
- evidence refs;
- QL reading refs;
- unresolved relations;
- next relations;
- provenance.

This envelope is experiment evidence over Actuation Return. It is not a new product ontology.

Every executable call appends a compact provenance/evidence record to `QL_RELATIONAL_EVIDENCE_LOG` when that environment variable is present.
