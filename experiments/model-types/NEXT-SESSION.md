# NEXT SESSION — model-types experiments: context, run, report

You are picking up the model-type experiment programme in
`experiments/model-types/` of the Actuation repo. Your job, in order:
(1) get context (30 minutes of reading, listed below), (2) go straight into
running the experiments, (3) come back with clean, well-explained, detailed
reporting per the contract at the end. The owner has said plainly: "I have no
idea what these numbers even mean in context" — every number you report must
be explained in plain language, with its meaning in practice. No naked
decimals.

## Hard rules (do not violate)

- The bimba Neo4j graph is authoritative and data-complete. **Do not touch
  `bimba_sync`. Do not push anything from the vault or datasets into the
  graph.** Graph writes happen only through `bimba_embed`.
- Embedding defaults: **3072 dims, `models/gemini-embedding-2`** (already set
  in the bimba `.env`; the literal name "gemini-embedding-002" does not exist
  — 404).
- All readings are instrument output: `semantic-stochastic`/`research`
  standing, `promotion: none`, determination stays `pending-human-review`.
  A fixture result never counts as a live claim.
- Seeds stay fixed (20260917 probe, 20260918 head) so runs are comparable.
- Nothing is committed in Actuation; the working tree carries everything.

## Context to read first (in this order)

1. `experiments/model-types/README.md` — the programme: five faculties, five
   experiments, the pairing law (LLM owns semantics, Jev owns typed
   classification across QL positions and MEF lenses, EBM reads the bimba
   field, Mercury generates, GLM reflects).
2. `experiments/model-types/ebm/README.md` — the EBM field experiment, all
   phases, the corrected content story, current numbers.
3. `experiments/model-types/jev/README.md` + `jev/emt2-report.md` — the Jev
   typed-classification and loop-refereeing results.
4. `experiments/model-types/specimens.json` — specimen register.
5. ProjectCentral NOW handoffs (latest three, dated 2026-09-18/19) for state.
6. `experiments/ql-runtime/comparison/series1/BENCHMARK-V0.1.md` — the
   evidence law you inherit.

## Infra state (verify before running anything)

- Graph: bolt `100.92.62.101:7687`, auth `neo4j`/`bimba` (password ignored —
  server runs auth=none), database `neo4j`. Expected: 2,141 `:Bimba` nodes,
  13,844 relationships, ~1,992 nodes carrying `n.embedding` (3072-dim
  JSON-string, model `models/gemini-embedding-2`).
- If bolt refuses: on the Omarchy host (ssh frank@100.92.62.101), run
  `cd ~/vm-migration/bimba-project && docker compose up -d` and wait for the
  neo4j database to reach `online` (this stack is durable and pinned to the
  map volume; verified 2026-09-19). Do not start anything else.
- Python env: `/tmp/mt-venv` may be gone after reboot — recreate:
  `python3 -m venv /tmp/mt-venv && /tmp/mt-venv/bin/pip install neo4j numpy`.
- Embed driver: `/tmp/embed-bimba.mjs` — if /tmp was wiped, its logic is
  documented in ebm/README.md Phase 1.5
  (MCP stdio client, `bimba_embed` with `texts` + `store_for` uuid lists,
  50/batch, `versionNegotiation: {mode:'auto'}`, dimensions 3072).

## Experiments you can run immediately

All in `experiments/model-types/ebm/` unless noted:

1. **Field probe (full graph):**
   `/tmp/mt-venv/bin/python field_probe.py --samples 2000 --seed 20260917`
2. **Field probe (M0 focus):** same + `--root M0`
3. **Learned energy head:**
   `/tmp/mt-venv/bin/python train_energy_head.py` (writes weights + evidence)
4. **Jev typed classification (needs nothing new):**
   `cd ../jev && python3 build_states.py --limit 100 && node harness.mjs --limit 200`
   (credential is in the macOS login keychain; harness fails closed without it)
5. **Jev loop refereeing (E-MT-2):**
   `node harness.mjs --states emt2/states.jsonl --truth emt2/truth.jsonl --out-dir emt2/runs`

## Current baseline (explain every future number against this)

- Field probe, full graph: different-root corruption combined **0.939**;
  same-root **0.751** (content/semantic potential alone); sibling **0.557**;
  direction-reversal **0.500** for all pair potentials.
- Field probe, M0 focus: same-root **0.780**, sibling **0.697** (semantic).
- Learned head (held-out): different-root **0.985**, same-root **0.871**,
  direction **0.807**, sibling **0.539**; in M0 the head LOSES to plain
  cosine (0.719 vs 0.790; 0.608 vs 0.673).
- Jev vs the loop's recorded decisions: carrier choice 90.5%, determination
  outcome 90.0%, intent-already-achieved 95%, next-position 61.9% (chance
  16.7%); latency p50 ~240-270ms.

## Remainders — the actual work

**R1 — sibling-level separation (the main open frontier).** "Sibling"
corruption replaces a relation's target with a node that shares its parent
coordinate (e.g. real edge M0-1→M0-1-(0/1) corrupted to M0-1→M0-2). These are
hard because same-parent concepts are semantically adjacent by design. Current
content-embedding separation is ~0.56 on the full graph. Ideas to try, in
order: (a) pair-difference features for the head (embed(a)−embed(b), not just
each side separately); (b) relation-type-aware energy — the graph has 1,378
live relation types; condition the energy on the type; (c) hard-negative
mining (train on the sibling negatives the current model scores wrongly);
(d) per-root heads (M0-only head vs global). Report each idea's effect
separately.

**R2 — head redesign.** The small learned head loses to raw cosine in M0. It
must consume the semantic channel directly (not as one scalar), more capacity,
and output calibrated probabilities (not just ranking) so the loop gate can
threshold it. Compare head vs cosine vs combined explicitly in your report.

**R3 — direction at path level.** Direction sensitivity (0.807) exists only in
the learned head via asymmetric node features. Extend the state from a pair
(a→b) to a short traversal (a→b→c) and measure whether path-level energies
separate real traversals from corrupted ones — this is what a loop gate would
actually score.

**R4 — E-MT-2 next-position re-measure.** The 61.9% result is partly an
artifact: the question omitted the loop's mechanical state (per-position
allowance budgets, closure routing). Fix the exporter
(`jev/export_decision_contexts.py`) to include `allowance_state` (the field
exists in the catalogue's decision_context and was left empty), re-run, and
re-measure on the semantic remainder only. Mechanical transitions should stay
in Rust, not be asked of any model.

**R5 — composed-loop gate wiring (design first).** Before any Rust: write the
design for scoring candidate acts in the field at the `ql-next-act` acceptance
point (typed refusal with energy attached as evidence, mirroring the
allowance-schedule pattern). `policy.rs` is in-flight code — the design doc
lands in `experiments/model-types/hybrid-loop/`, code goes on a branch via
proposal, never silently.

**R6 — vendor-claim validation.** Jev's speed/cost claims are recorded
unverified in specimens.json except latency (measured p50 238ms). If you run
E-MT-4 conditions, measure Mercury's actual tok/s and cost from usage fields —
vendor claims until then.

## Reporting contract (this is the deliverable's shape)

For EVERY experiment you report:

1. **What was asked** — one plain sentence ("can the field's energy tell a
   real relation of the map from a corrupted one?").
2. **How it was tested** — what was corrupted and why that corruption is a
   fair stand-in for "incoherent" (one sentence per operator: different-root,
   same-root, sibling, direction-reversed).
3. **The number, translated** — give the score, then its meaning in practice:
   an AUC of 0.75 means "pick a real relation and a corrupted one at random;
   the field ranks the real one as more viable 3 times in 4". Always give the
   coin-flip baseline (0.5) and the previous best next to it.
4. **What it means for the theory** — one paragraph: does the bimba map, read
   as an energy field, show usable structure here? What changed vs the last
   run, and what do you attribute the change to?
5. **What remains** — the next concrete step, named.

Worked example of the required style: "Sibling test scored 0.557 (coin flip
0.500, last run 0.559). In practice: when the field is shown one real relation
and one relation whose target was swapped for a node under the same parent
coordinate, it ranks the real one as more viable about 56 times in 100 —
barely better than chance. Reading: the map's content embeddings distinguish
concepts across the map well (0.939) but cannot yet tell neighbouring siblings
apart; this is the gap between 'the field has geometry' and 'the field can
adjudicate fine-grained placement'."

If a number cannot be explained in that style, you do not understand it yet —
investigate before reporting. Detail is wanted; jargon without translation is
not.
