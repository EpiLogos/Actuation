# EBM over the bimba grounding field

The owner's question this experiment answers: does an energy-based treatment of the
bimba map produce a useful geometry of viable versus incoherent states? An energy
here is compatibility with declared or learned structure — **low energy is not
value, correctness, or worth**; it is no wiser than the structure it is computed
from.

## The field

- **State**: a directed relation `(a, r, b)` between two bimba coordinate nodes.
  Later phases extend states to traversals and to loop-proposed transitions.
- **Grounding field**: the live graph at the bimba-portable Neo4j
  (`bolt://100.92.62.101:7687`, auth as configured in `Work/epi/bimba-portable/.env`;
  ~2,141 `:Bimba` nodes, ~11,774 relations as of 2026-09-11).
- **Structure comes from the coordinates, not from relation-type declarations.**
  The `bimba_schema` relation-type vocabulary is out of line with the live graph —
  zero of its `POSk_*` types appear among the 1,378 live types (owner-confirmed
  2026-09-17). The operative schema is the coordinate grammar itself: the probe
  mirrors `bimba-portable/src/coordinates/parser.ts` (type letter `C|P|M|S|T|L`,
  multi-digit segments, `-`/`.` descent, canonical context frames, prime marker,
  legacy `#` → M family) and reads structure from coordinate sections/segments:
  - `E_lineage` — 0 for ancestor/descendant or identical coordinates; a flat 1.5
    across coordinate families; otherwise `1/(1+LCP)` by longest common prefix.
    These weights are the probe's declared convention, stated in every evidence
    file — not owner-declared legality.
  - `E_semantic` — `1 − cos(emb_a, emb_b)` over in-graph embedding vectors when
    present; otherwise a lexical-token overlap fallback, which is
    **fixture-grade** and labelled as such in every output.
- **Readout**: separation quality (AUC of P(real energy < corrupted energy); 0.5
  is no separation), per corruption operator and per potential, plus mean
  energies.

## Corruption operators (the degradation ladder)

1. `endpoint-rewire` — target replaced with a random node of a *different
   coordinate root* (type + first segment). The coarse test.
2. `hard-rewire` — target replaced with a random node of the *same root*.
3. `sibling-rewire` — target replaced with a random *sibling* (same type and
   parent segments, different final segment). Structure is nearly invariant
   here; this is where semantic compatibility and later the learned head are
   tested.
4. `direction-reversal` — endpoints swapped: `(b, r, a)`. Pair potentials are
   direction-blind by construction, so this operator marks exactly the gap a
   direction-sensitive learned potential must close.

All sampling is seeded; graph-capture digests are recorded in each evidence file.

## Phase-1 live results

### Embedding the field (2026-09-18)

The M/L scope (1,777 nodes: M 1,691 + L 86 — the coordinate families this
programme's field work runs on) is now embedded in-graph via `bimba_embed`
(`SEMANTIC_SIMILARITY`, 768 dims, model `gemini-embedding-001`; verified 1,777/1,777
with `n.embedding` present). Getting there required fixing **three real defects in
bimba-portable's embedder**, which is why no vectors existed in-graph before:

1. `callGeminiAPI` posted a `requests` array to the singular `:embedContent`
   endpoint ("Unknown name requests") — the batch endpoint is
   `:batchEmbedContents`.
2. The URL double-prefixed `models/` while the batch body requires the prefix
   ("unexpected model name format").
3. The store Cypher matched `{uuid: …}`, but Bimba nodes carry `c_2_uuid` —
   every write silently matched nothing and reported success.

Also: `text-embedding-004` is no longer served to this key; `gemini-embedding-001`
is the working model (env `GEMINI_EMBEDDING_MODEL=models/gemini-embedding-001`).
The valid Gemini credential lives in `legacy-python-stack/.env`; the
`GEMINI_API_KEY` exported from `~/.zshrc` is **dead** ("API key not valid") and
should be rotated or removed. These fixes are uncommitted in `Work/epi/bimba-portable`
(not a git repo) — flag them for the upstream sync policy.

### Full graph (seed 20260917, 2,000 pairs/operator, embedding readout)

| Operator | `E_lineage` | `E_semantic` | combined |
|---|---|---|---|
| endpoint-rewire | 0.883 | 0.775 | **0.902** |
| hard-rewire | 0.639 | **0.679** | 0.659 |
| sibling-rewire | 0.517 | 0.539 | 0.523 |
| direction-reversal | 0.5 | 0.5 | 0.5 |

With real embeddings the semantic potential now **outperforms** lineage at the
hard levels (same-root and sibling), which the lexical fallback could not do.
Unit weights are visibly suboptimal (combined < semantic at hard-rewire) —
weight learning belongs to Phase 2.

### M0 focus (owner hypothesis: densest region for values, metaphysical dynamics, condensed relation)

| Operator | `E_lineage` | `E_semantic` | combined |
|---|---|---|---|
| endpoint-rewire | — vacuous inside a single root — | — | — |
| hard-rewire | 0.696 | 0.724 | **0.743** |
| sibling-rewire | 0.622 | 0.667 | **0.673** |
| direction-reversal | 0.5 | 0.5 | 0.5 |

**Confirmed.** Every comparable operator separates more strongly inside M0 than
on the full graph; the sibling level moves from near-blind (0.523) to clearly
separated (0.673). The densest, most condensed metaphysical region has the
strongest field geometry, and semantic compatibility carries it. Same-root
negatives inside M0 are drawn from the M0-internal relation set (566 edges over
~111 nodes), so cross-region comparisons are directional, not exact.

Evidence: `evidence/field-probe-20260918-021435.json` (full graph),
`evidence/field-probe-20260918-021455.json` (M0 focus) — digest-pinned,
promotion none, reading class research.

## Phase 2.5 — embedding the graph's own content (corrected 2026-09-19)

Correction of an earlier wrong claim: a first pass reported "the graph carries
no content". That survey checked legacy unnamespaced property names
(`description`, `keyPrinciples`, …) on one content-bare straggler node and
generalised. The real survey (`survey_graph.py`) shows the current map stores
its content in the coordinate-tagged schema — 3,026 distinct keys:
`c_1_description` on 1,691 nodes, `c_0_core_nature` 997, `c_1_key_principles`
412, `c_5_resonances` 339, the `m_3_5_*` matrix keys ~360 each, QL metadata on
nearly all — mean ~1,670 chars of text per node. The map was complete all
along; the readings were not.

**Embedding v3 (current): composed from the graph's own properties**
(`compose_from_graph.py`): the common essential set (coordinate, name,
description, QL/family/layer/subsystem metadata) plus every available
content-family key, bounded 6,000 chars. 1,992 M/L nodes embedded at **3072
dims with `gemini-embedding-2`** (the API has no "gemini-embedding-002";
`-2` is the successor to 001). Provenance:
`evidence/embed-text-provenance.json`.

Separation ladder (AUC that a real relation scores lower-energy than a
corrupted one; 0.5 = coin flip), same seeds throughout:

| Operator | name-only 768 (09-18) | graph-native 3072 (09-19) |
|---|---|---|
| full graph, endpoint-rewire | 0.902 combined | **0.939 combined** (semantic alone 0.923) |
| full graph, hard-rewire | 0.659 | **0.751 semantic** |
| full graph, sibling-rewire | 0.523 | 0.557 semantic |
| M0, hard-rewire | 0.809 | **0.780 semantic** |
| M0, sibling-rewire | 0.707 | 0.697 semantic |

Head retrained (held-out, same protocol): endpoint **0.985**, hard-rewire
**0.871**, direction-reversal **0.807** (pair-symmetric potentials are pinned
at 0.500 there); sibling-rewire 0.539 — still the open level. In M0 the raw
cosine still beats the small head (0.790 vs 0.719; 0.673 vs 0.608): the head's
features predate rich embeddings; redesign is next-phase work.

**Infra note (2026-09-19):** the graph went offline mid-session. The serving
container had been removed and its storage was not recoverable from any named
volume; the map itself is intact in the 2026-09-14 snapshot volume and is now
served by a durable compose stack (`~/vm-migration/bimba-project` on the
Omarchy host, pinned to the named volume). Embeddings written before 09-19
were lost with that container and have been regenerated from graph content.
Sync was never invoked; nothing was pushed into the graph from the vault.

Evidence: `survey` output embedded in `evidence/`, latest
`field-probe-*.json` (full + M0), `energy-head-*.json`,
`energy-head-weights.npz`. Standing law unchanged: readings are
`semantic-stochastic`/`research`, never promoted; low energy is
compatibility, nothing else.

## Phase 2.6 — head redesign + sibling frontier (2026-09-19, R1/R2)

`train_energy_head_v2.py` runs a rung ladder where each rung adds exactly one
idea to the Phase-2 head, so each idea's effect is measured separately; seeds
unchanged (probe 20260917, head 20260918), split unchanged (edge digest mod 5).
Rungs: `base` (phase-2 replication) → `semantic_channel` (PCA-64 projections of
e_a, e_b, e_a−e_b, e_a⊙e_b + cosine fed directly — R1a/R2's "consume the
semantic channel") → `typed` (+128-dim type hash + incident-type profile
hashes + log-degree — R1b) → `capacity` (2×128 — R2) → `hardneg` (mined
sibling train negatives ×3, low-LR fine-tune — R1c); plus `m0only` (R1d:
M0-trained head) and `v2b` (mining applied to the best rung, `typed` 1×32 —
`train_energy_head_v2b.py`).

Held-out separation AUC (sibling-rewire; coin flip 0.500; phase-2 best learned
0.539 full / cosine 0.790-ish M0-era baselines in parentheses where relevant):

| rung | full sibling | M0 sibling | note |
|---|---|---|---|
| base 1×32 | 0.519 | 0.585 | replicates phase 2 (0.539 / loses to cosine in M0) |
| + semantic channel | 0.580 | **0.728** | first head to beat raw cosine in M0 (cos 0.717) |
| + typed conditioning | **0.619** | 0.666 | best full-graph sibling; direction 0.876, hard 0.923 |
| + capacity 2×128 | 0.602 | 0.632 | capacity hurts — 1×32 generalises better |
| + mining (on 2×128) | 0.614 | 0.648 | mostly fixed calibration (ECE 0.144 → 0.043) |
| + mining (on typed 1×32, `v2b`) | **0.654** | **0.711** | recommended configuration |

Recommended head (v2b: semantic channel + typed conditioning, 1×32, sibling
mining) — full ladder, held-out:

| operator | full graph | M0 | cosine (full / M0) |
|---|---|---|---|
| endpoint-rewire | **0.987** | — vacuous | 0.926 / — |
| hard-rewire | **0.922** | **0.843** | 0.767 / 0.811 |
| sibling-rewire | **0.654** | **0.711** | 0.547 / 0.657 |
| direction-reversal | 0.848 | 0.657 | 0.500 / 0.500 (pinned) |

Calibration of the v2b head on the sibling operator: Brier 0.229, ECE(10)
0.048 — probabilities a loop gate can threshold (Youden τ ≈ 0.54 on held-out;
per-operator thresholds in the evidence file).

Per-idea reading (each rung vs the one before, sibling operator):

- **R1a direct semantic channel: the single biggest win.** Full sibling
  +0.06; M0 sibling +0.14 and the first head configuration to beat raw cosine
  inside M0. The phase-2 head's one-scalar cosine feature was the binding
  constraint, exactly as R2 suspected.
- **R1b relation-type conditioning: the biggest full-graph win.** +0.04
  sibling, +0.08 direction (0.799 → 0.876), +0.02 hard-rewire over the
  semantic-channel rung. The live relation vocabulary and each endpoint's
  incident-type neighbourhood carry real structure.
- **R1c hard-negative mining: real but second-order** (+0.035 on the best
  rung, +0.012 on 2×128) and the main calibration fix (ECE 0.144 → 0.043 on
  2×128; 0.048 on v2b).
- **R1d per-root head: no.** The M0-only head loses to the global head on the
  same M0 held-out (sibling 0.587 vs 0.632; direction 0.444 — below coin
  flip) — ~453 M0 train edges starve it. Keep the global head.
- **R2 capacity: answered negatively.** 2×128 loses to 1×32 everywhere that
  matters (sibling, M0). The redesign's win came from features, not size.

Honest limits: M0 held-out is 121 edges — differences within ±0.05 there are
noise; negatives are synthetic corruptions (AUC on the ladder, not on real
loop transitions); calibration describes agreement with the real-vs-corrupted
label, not with human judgement.

Evidence: `evidence/energy-head-v2-20260919-191342.json` (ladder),
`evidence/energy-head-v2b-*.json` (recommended config),
`evidence/energy-head-v2-weights.npz`. Phase-2.6b script:
`train_energy_head_v2.py`, `train_energy_head_v2b.py`.

## Phase 3-preview — path-level energies (2026-09-19, R3)

`path_probe.py` extends the state from a pair to a 2-hop traversal
a→b→c (consecutive real edges) — the shape a loop gate would actually score.
Corruption operators: the pair ladder applied to the last hop, plus
`mid-rewire` (middle node replaced — the traversal no longer exists) and
path-level direction-reversal (c→b→a). All scorers are pairwise-summed
hop energies, so per-hop direction blindness still applies except to the
head's directed features. Evidence: `evidence/path-probe-*.json`.

## Phase 2 — learned energy head (2026-09-18, `train_energy_head.py`)

A 1-hidden-layer (32-unit) energy head, numpy-only, trained with BCE on real
edges (positives) against the corruption ladder (negatives, drawn from **train
edges only**), evaluated on held-out edges (split by edge digest, mod 5).
Features: the Phase-1 potentials, plus directed structure — per-direction
ancestor flags, depths, family/root one-hots, prime flags, and a 32-dim signed
hash of the live relation type. Separation AUC (P(real energy < corrupted)),
held-out:

| Operator | learned | best baseline | Δ |
|---|---|---|---|
| endpoint-rewire | **0.977** | 0.898 (combined) | +0.079 |
| hard-rewire | **0.841** | 0.704 (semantic) | +0.137 |
| sibling-rewire | 0.522 | 0.539 (semantic) | **−0.017** |
| direction-reversal | **0.732** | 0.500 (all) | +0.232 |

**M0 focus** (self-contained: corruption pools restricted to M0's ~111 nodes and
123 held-out edges): hard-rewire learned 0.720 ≈ combined 0.721; sibling 0.572
vs combined 0.629; direction 0.593 vs 0.5; endpoint vacuous (n=0).

Reading:

- **Direction blindness is broken** — 0.732 on the full graph where every
  pair-symmetric potential is pinned at 0.500 by construction. The graph's
  directed structure is real and learnable (the head uses per-direction
  ancestor flags and per-side depths/families to see it).
- **The coarse and same-root levels are solved** (0.977 / 0.841) — the head
  learns the weighting the fixed unit-sum combined potential got wrong.
- **Sibling-level discrimination remains the open frontier** — the head does
  not beat the best baseline there (0.522 vs 0.539; in M0 the baselines still
  win outright). Same-parent candidates are semantically and structurally
  close; cracking that needs richer features than this head has (better
  embeddings, typed/traversal features, more data), not more epochs.

Artifact: `evidence/energy-head-weights.npz`;
evidence `evidence/energy-head-20260918-030352.json` (digest-pinned,
`semantic-stochastic`, promotion none). Phase 3 (loop gate) consumes the head
by scoring candidate acts/transitions; refusals typed, energy attached as
evidence.

## Phases

- **Phase 1 — evaluative probe (implemented, `field_probe.py`, no training):**
  runs live against the real graph; no API keys needed. `--root M0` restricts
  the field to a coordinate family (used for the focus result above).
- **Phase 1.5 — embedding readout (done 2026-09-18):** 1,777 M/L nodes embedded
  in-graph via the fixed `bimba_embed`; the probe now uses real cosine
  compatibility. Re-embed command for new/changed nodes: run the MCP stdio
  driver pattern against `bimba_embed` with `texts` + `store_for` uuid lists
  (50/batch), env `GEMINI_EMBEDDING_MODEL=models/gemini-embedding-001` and
  `BIMBA_MCP_PERMISSIONS=bimba:read,bimba:write` for the process.
- **Phase 2 — learned energy head:** train a small energy model on the graph's
  real relations with the corruption ladder as negatives, targeting exactly the
  gaps Phase 1 measured: direction sensitivity, potential weights, and sibling
  separation outside M0; the EB-JEPA energy treatment is the reference
  architecture. A learned reading is `semantic-stochastic`, never promoted.
- **Phase 3 — loop gate (E-MT-4):** the composed loop scores candidate acts and
  transitions in this field; refusals are typed, with the energy attached as
  evidence, mirroring the allowance-schedule typed-refusal pattern in
  `policy.rs`.

## Running

```bash
python3 -m venv /tmp/mt-venv && /tmp/mt-venv/bin/pip install neo4j
/tmp/mt-venv/bin/python field_probe.py \
  --uri bolt://100.92.62.101:7687 --database neo4j \
  --samples 2000 --seed 20260917 --out evidence
# focused:
/tmp/mt-venv/bin/python field_probe.py --root M0 --samples 2000 --seed 20260917
```
