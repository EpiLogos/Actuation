# MEF/jev general-corpus sweep v3 — SUMMARY (2026-09-21)

Instrument: `jev-reflect.mjs` + `mef-giving.json` **v3** — the function-typed redesign.
Each lens now carries a `function` (`exhaustive`: L0, L2 — mapped across all six
sub-slots, "the distribution is the map"; `locate`: L3, L3', L4', L5 — subject placed
in its progression; `select`: L1, L4, L0', L1', L2', L5' — dominant member; every
returned cell carries the `function` field). Squares/resonant unchanged.

Corpus: the same 12 real positions (C1..C12). Per this round's instruction the
subject text is **markdown-stripped** (blockquote markers and emphasis removed,
quoted statement joined into one paragraph + the core-move paragraph); v2 fed the
raw `> ` text. Run: full_text ×3 (first run + 2 repeats) + resonant ×1 +
being/becoming/knowing ×1 = 7 calls × 12 = **84 calls, all succeeded first-try**
(wall 0.69–0.98 s, no retries, no invalid JSON). No name-only variant. Raw
responses: `c<NN>-<mode>-<rep>.json` here; run record `run-log.json`;
computed numbers: `computed-report.json`.

Methodology caveat: v3 changes two things at once (typed lens questions AND
stripped subjects), so v2→v3 deltas are the two effects confounded. The within-v3
comparisons (function type vs behaviour) are clean.

Names shortened ("L2-3 BOTH" = L2-3 BOTH (IS and IS-NOT); "L4.5' Insight" etc.).

---

## a) HEADLINE — L2 Logical as a map

The mapped form works. On the identical corpus, L2's mean top-slot mass fell from
**0.589 (v2 select-form) to 0.488 (v3 mapped-form)** and its mean entropy rose
**1.551 → 1.818 bits** — mass moved out of the single peak into the map. The maps
differ across entries in *shape*, not just peak. Rep-averaged maps (slot order =
register order: Ground · IS · IS-NOT · BOTH · NEITHER · SILENCE):

| entry | Ground | IS | IS-NOT | BOTH | NEITHER | SILENCE | shape in words |
|---|---|---|---|---|---|---|---|
| C1 Bentham | 0.13 | **0.78** | 0.01 | 0.05 | 0.01 | 0.01 | single-band assertion — value *is* a sum |
| C2 Smith | **0.31** | **0.33** | 0.03 | 0.23 | 0.07 | 0.03 | most dispersed; sits between ground and assertion |
| C3 Nozick | 0.20 | **0.60** | 0.09 | 0.08 | 0.02 | 0.02 | firm assertion + noticeable ground |
| C4 Burke | 0.24 | **0.34** | 0.08 | 0.28 | 0.04 | 0.02 | three-way: assertion / co-presence / ground |
| C5 Locke | 0.16 | **0.58** | 0.04 | 0.19 | 0.01 | 0.01 | assertion with BOTH secondary |
| C6 Marx | 0.11 | **0.64** | 0.09 | 0.14 | 0.01 | 0.01 | firm assertion |
| C7 Hardin | 0.24 | **0.48** | 0.01 | 0.25 | 0.01 | 0.01 | assertion + BOTH (ruin co-present with freedom) |
| C8 Kahneman | 0.12 | **0.52** | 0.01 | 0.33 | 0.01 | 0.01 | assertion + strong BOTH (two systems one mind) |
| C9 Whorf | 0.31 | **0.40** | 0.02 | 0.20 | 0.04 | 0.03 | dispersed, ground-heavy (flux before dissection) |
| C10 Frege | 0.31 | 0.23 | 0.04 | **0.36** | 0.04 | 0.02 | BOTH leads — the informative identity |
| C11 Freud | 0.15 | 0.19 | 0.08 | **0.53** | 0.03 | 0.02 | BOTH dominates — conscious/unconscious held together |
| C12 Ramsey | 0.22 | **0.34** | 0.20 | 0.09 | 0.11 | 0.03 | only entry with a real negation family |

**Verdict: informative, no longer flat.** Concrete contrasts:

- **Bentham vs Ramsey** — the brief's test case. C1 is a 0.78 spike on IS with the
  other five slots summing to only 0.21; C12's negation-and-void slots (IS-NOT 0.20 +
  NEITHER 0.11 + SILENCE 0.03 = 0.34) *equal* its IS peak (0.34). The map says:
  Bentham asserts, Ramsey theorises confidently *about* uncertainty — exactly the
  content difference. A peak-only reading showed neither.
- **Burke** — IS 0.34 / BOTH 0.28 / Ground 0.24: the dead-living-unborn partnership
  read simultaneously as assertion, as co-presence of what is and is not present,
  and as pre-logical ground. No single winner could carry that.
- **Smith vs Frege** — C2's argmax *flip-flops between reps* (Ground 0.32 / IS 0.32,
  rep1 picked Ground, reps 2–3 IS) which under a map regime is itself the honest
  signal of a tie; C10 is the only non-C11 entry where BOTH beats IS (0.36 > 0.23).

Honest caveat: at peak level the catuṣkoṭi gravity persists — IS still takes the
argmax in 10/12 (BOTH in C10 and C11). What changed is that the *peak no longer
carries the reading*: assertion strength ranges from C1's 0.78 to C10's 0.23, and
pairwise map distance spans 0.13 (C2–C9, twin dispersed maps) to 1.19 (C1–C11).
All 12 maps are pairwise distinct.

## b) Stability

Per-lens strongest-slot agreement across full_text repeats (12 lenses × 12 entries
= 144; instrument's own `strongest` field):

- **rep1 = rep2: 138/144 = 95.8%** (v2 baseline recomputed from its raw files on
  the same field: 140/144 = 97.2% — confirmed, so the typed forms cost ~1.4 pp).
- **all three reps agree: 137/144 = 95.1%**.

Disagreement sites (all sibling or near-tie flips, never a leap):

| entry | lens (fn) | rep1 / rep2 (/ rep3) | note |
|---|---|---|---|
| C2 | L2 (exhaustive) | Ground / IS / IS | a true 0.32/0.32 tie — the flip *is* the signal |
| C6 | L3' (locate) | Spring / Spirit / Spring | Marx: emergence vs the Spirit in history |
| C6 | L5' (select) | Dynamis / Arche / Dynamis | |
| C7 | L0' (select) | Three / Two / Three | |
| C10 | L3' (locate) | Spring / Spirit / Spring | **the only site that also flipped in v2** |
| C12 | L5 (locate) | Madhyamā / Mātṛkā / Mātṛkā | 0.34/0.33 near-tie |
| C4 | L5' (select) | Sophia / Sophia / Epi-Logos | rep3 only |

Per-lens: L3' is the only lens below 11/12 (10/12 — both its flips are the same
Spring↔Spirit wobble); L2, L5', L5, L0' sit at 11/12; all others 12/12. Mapping
did not destabilise the exhaustive lenses beyond the one genuine tie; 5 of the 6
rep1-rep2 flips sit in locate/select lenses, and the one recurring instability
across instrument versions is Frege's L3' (Spring↔Spirit, unresolved in both v2
and v3).

## c) Functional difference visibility

Rep-averaged concentration by function type (top1 = mean max slot mass; entropy
in bits, 6-slot max 2.585; adjacent/farther = mean mass ±1 slot from the peak vs
beyond, meaningful for the ordered lenses):

| type | lenses | cells | top1 | entropy |
|---|---|---|---|---|
| exhaustive | L0, L2 | 72 | **0.456** | **1.971** |
| locate | L3, L5, L3', L4' | 144 | 0.682 | 1.264 |
| select | L1, L4, L0', L1', L2', L5' | 216 | 0.680 | 1.165 |

Three findings:

1. **The exhaustive lenses are now the flattest, most structured cells in the
   instrument** — exactly as designed. L0 maps are genuinely multi-slot
   (C6 Marx: Why 0.40 + Where/When/Why-for 0.27; C2 Smith: How 0.48 +
   Where/When 0.18 + Why-so 0.16; C11 Freud: What 0.45 with How and Ground
   trailing).
2. **The locate family is internally heterogeneous, not uniformly "spread".**
   Aggregate, locate ≈ select in concentration (0.682 vs 0.680). Per lens:
   - **L5 reads as a true progression where content calls**: C12 Ramsey spreads
     Madhyamā 0.34 / Mātṛkā 0.33 / Vaikharī 0.27 — the whole articulation band
     (mental structuring → acted bets → letters as measuring power, for a theory
     that measures belief by bets); C9 Whorf Madhyamā 0.54 + Vaikharī 0.42,
     adjacent mass, little beyond. L5 per-lens: 0.227 adjacent vs 0.086 farther —
     the mass hugs the progression's neighbourhood.
   - **L3' is the most spread lens in the instrument** (top1 0.471, entropy 1.89):
     C4 Burke = Spirit 0.62 + Life 0.28 (Geist moving through history + the full
     cycle renewed — the partnership across generations).
   - **L3 stays pick-like** (top1 0.727; C11 Concrescent Desire 0.79), and
     **L4' has become the most concentrated lens in the entire instrument**
     (top1 0.840, adjacent mass 0.017).
3. **Select stays concentrated as designed** (L1 0.747, L1' 0.787) — no regress,
   e.g. C3's L1-0 Svātantrya 0.85 and C5's L1-4 Final Cause 0.96 are as sharp as
   v2.

The honest negative: L4' (Scientific) under its new locate instruction answers
**Insight 12/12** — *up* from 8/12 under v2's select form. Asked where the subject
"sits" on a Questions→…→Insight ladder, the model always answers at the top of
the ladder. The locate typing sharpened L4' into a pure endpoint-detector; as a
progression reading it is worse than before.

## d) Sharpest readings and misses (against the known core moves)

Sharpest:

1. **C11 Freud** — the whole stack coheres: L2 map BOTH 0.53 (conscious and
   unconscious co-present — the dialetheia of the divided mind, now in the main
   reading), L3-0 Concrescent Desire 0.79 (desire/lack as the motor), resonant
   L3 Processual 0.86 (corpus's strongest frame mass, again), L1-0' Introversion
   0.98 (toward-unconscious-pole), L5-1' Apokalypsis 0.53 (the hidden unveiled in
   symptoms and dreams), L3-4' Winter 0.32 with Spring/Summer trailing (dormancy
   and return spread across the cycle — the locate form showing its spread).
2. **C12 Ramsey** — L4.4 Besorge (belief measured by dispositions to act), the L5
   articulation-band spread quoted above, the only L2 map with a real negation
   family (uncertainty as subject), L1-3 Formal (axioms from coherence), resonant
   L2 0.59, knowing square L2@P2 = IS-NOT. The v3 map adds what the v2 peak
   (bare IS-NOT) could not: a confident theory *about* the not-is.
3. **C4 Burke** — L4.3 Zeit 0.99 (the corpus's highest day-lens mass: temporality
   for the partnership across time), resonant L3' Chronological 0.64, L0-2' Three
   0.88 (dead/living/unborn as triad), L3-4 Community 0.95, and the three-band L2
   map. Near-perfect on every layer.

Misses and weak spots:

1. **L4' Insight 12/12** (above) — the Scientific lens no longer separates
   anything; its v2 disagreement capacity (Patterns ×4) was real signal and the
   locate form destroyed it. If kept typed, this lens's locate wording needs
   revision ("which stage of the inquiry does the subject occupy?" rather than
   "where does it sit").
2. **C7 Hardin's remedy half** — "mutual coercion, mutually agreed upon" still
   never surfaces: L3-4 Community 0.92 and L5-4 Vaikharī 0.87 read the
   aggregation, L1 Formal 0.49 the payoff structure, but the coercive-solution
   move stays invisible; L0's map is thin mush (top slot 0.32). Same partial as
   v2.
3. **C3 Nozick L3-4 Community Integration 0.60** — a soft miss: for the
   individualist entitlement theory the processual lens picks *society-forming*,
   when the argument's engine is L1-0 Svātantrya (0.85, correct) and the knowing
   square's IS-NOT (the imposed pattern negated). v2 picked Actual Occasion here;
   neither reading catches "liberty upsets patterns" at L3.
4. **L5' Sophia 9/12** — unchanged from v2: the untyped night default still
   gravitates to Sophia (only Arche for Nozick, Dynamis for Marx, Apokalypsis for
   Freud break it). The owner's pending night-lens typing is still the fix.
5. **Resonant lumpiness persists**: L1 Causal tops all six social/political/
   economic entries (C5 Locke at 0.89 the extreme); at frame level the corpus
   still splits into "Causal social argument" vs everything else.

## e) Anomalies

1. **A saturated cell**: C8 Kahneman L0-1' Two = **1.00** (perfect distribution;
   v2 had 0.95). "Two systems" as pure duality — the instrument can be fully
   certain; treat saturated cells as ceiling, not error.
2. **Stability dip is real but marginal** (95.8% vs 97.2%) and concentrated at
   near-ties; the only recurring flip site across instrument versions is C10 L3'
   Spring↔Spirit — Frege genuinely unsettles the Chronological lens.
3. **C2 L2's rep1 Ground → rep2/3 IS flip at 0.32/0.32**: under the exhaustive
   regime an argmax flip at a tie is expected behaviour of the map, not noise;
   peak-level stability metrics will systematically under-count such entries.
4. **Endpoint gravity from locate typing** (L4' 12/12 Insight, up from 8/12) —
   the one clearly counterproductive effect of the redesign; see (c)/(d).
5. **Night-lens default gravity unchanged** (L5' Sophia 9/12, L1' Thinking 8/12,
   L0' Two/Three splits) — the function typing did not touch the untyped night
   defaults, as expected pending the owner's typing.
6. **API**: 84/84 clean, wall 0.69–0.98 s, no retries, no invalid JSON.

## Files

- Raw responses: `c01..c12-{full_text-1,full_text-2,full_text-3,resonant-1,being-1,becoming-1,knowing-1}.json`
- `corpus-meta.json` (parsed, markdown-stripped subjects), `run-log.json`
  (note: written by the resumed second invocation — done=12, skipped=72; combined
  across both invocations 84/84 ok), `computed-report.json` (all tables above,
  machine-readable), `analyze.mjs` (report generator), `run-sweep.mjs` (driver,
  resumable)
- v2 baseline: `/Users/admin/.cache/actuation/analysis/general-sweep/` (raw files
  re-read for the stability recomputation; v2 not modified)
- Instrument: `/Users/admin/.cache/actuation/jev-reflect.mjs` +
  `/Users/admin/.cache/actuation/mef-giving.json` (v3, not modified by this sweep)

Nothing committed anywhere; analysis artifacts only, in this directory.
