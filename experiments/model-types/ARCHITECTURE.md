# EBM ↔ Jev over the bimba map — the full architecture (v2, design)

**Status:** design; v2 re-grounds the frame on the owner's corrections of
2026-09-21 and on a fresh read-only graph survey of the same date. Nothing in
v2 is tested yet. Standing law unchanged: instruments are readings, promotion
none, determination pending-human-review.

**What changed from v1.** v1 claimed the twelve archetypal numbers existed
"only as prose" and had to be reconstructed at architecture level. That was a
reader error, and v2 removes it: the twelve are fully present in the map —
eleven branches of M0-3 plus M0-2-9 — each with its own internal dynamics.
v1 also conscripted the M1-2 Ananda vortex as a "verification authority" for
Jev's verdicts. That is withdrawn: M1-2 already has its own standing role, and
v2 designs it as a second classification layer. v1's "verified gaps" section is
gone; where a reading is incomplete, that is the reader's limitation, named as
such, never the map's defect. No part of this document proposes map changes or
claims map authority.

## 0. The frame this architecture serves

- The map is a **whole and authoritative**. Every structural claim below was
  read from graph properties (survey of 2026-09-21) or from the QL kernel
  sources; inferences are marked as inferences.
- The **Anuttara syntax computes as a whole** and is handled by **Jev** as
  typed classification over the 109 marks and the M0-3 branches.
- The **twelve archetypal numbers are first-class** — they are the entities
  the syntax computes with, each grounded in its own node properties
  (section 2). The role table there is the specification the Jev tiers are
  designed against.
- **Relation embeddings are Jev's knowledge base of declared relations.**
  Edges carry declared, embeddable content (section 3b). With them, Jev knows
  the relations between nodes from the map's own declarations — there is no
  "learning the unmapped", because nothing is unmapped.
- **Classification is layered** (section 4): layer 1, the Anuttara syntax
  (Jev over marks/formulations/branches); layer 2, the Ananda vortex
  (M1-2), which parses layer-1 output relative to the vortex's numerical
  relations. Layer 2 is the vortex's own standing role — not a conscription.
- The **EBM's proper object is the value structure** — the virtue field at
  M0-2-9 — and the general map, tested against declared structure and
  scrambled-relation text, never sibling-swap theatre (section 5).
- **Jev classifies, EBM scores placement with energy as evidence, Rust owns
  the recurrence** (section 6).

## 1. Layer: ground (read-only authority)

The live graph: 2,141 `:Bimba` nodes, 13,844 typed edges (2026-09-21). The M0
scope is exactly **109 nodes** with 566 internal edges of 151 types. M0-scope
`c_1_symbol` coverage is 107/109 (the two open entries are a pending owner
designation, section 3c — not a map defect). The graph is read-only for this
programme: no sync, no vault pushes, no writes of any kind; the only prior
writes were embedding vectors through `bimba_embed`, and v2 adds none to the
graph (relation vectors live in the field store, section 3b).

## 2. The twelve archetypal numbers — the role table

The map carries the complete set. Eleven are the `HAS_INTERNAL_COMPONENT`
children of M0-3 ("Archetypal Number Language"); the twelfth is M0-2-9
Paramesvara, which declares its relation to the other eleven itself:
`CONTAINS_ARCHETYPAL_NUMBER` ×10, `NESTS_ARCHETYPE_8` → M0-3-11, and
`NESTS_COMPLETE_NUMBER_LANGUAGE` → M0-3. All twelve carry
`c_1_complete_formulation` (eleven also carry `c_1_formulation_breakdown`;
M0-2-9 carries formulation only). The role column is what the number **is in
the syntax**; the dynamics column is its declared internal structure.

| # | Node | Symbol | Class (`m_0_3_adam_eve_classification`) | Role in the syntax | Declared internal dynamics |
|---|---|---|---|---|---|
| (-) | M0-3-(0/1) | (-) | pre-numerical (no class key) | The Mirror — reflector preceding the number series | `HAS_CONSTITUENT` ×2 → The Frame `()`, The Operator `-`; `HOLOGRAPHIC_PATTERN` ×19 |
| 0 | M0-3-2 | 0 | Neither — Transcendent Root | the unchanging witness; static ground of Sat-Spanda | `ARCHETYPAL_RESONANCE_PURE_POTENTIAL`, `PURE_POTENTIAL_MANIFESTATION` |
| 1 | M0-3-3 | 1 | Neither — Transcendent Root | will-to-manifest; first pulse, active agency | `HIDDEN_AGENCY_AWAKENS`, `ACTIVE_AGENCY_MANIFESTATION` |
| 0/1 | M0-3-4 | 0/1 | Neither — Transcendent Binary | non-dual unity-in-difference; the system's engine | `CONTAINS_AS_CONSTITUENT` ×3 → 0, 1, (-); `POWERS_NONDUAL_MANIFESTATION` |
| 2 | M0-3-5 | 2 | Adam — Structural Foundation | the relational field (Śūnyatā, "the Between"); first container for Spanda | `ADAM_EVE_COMPLEMENT`, `PROVIDES_EXPLICATE_FOUNDATION` |
| 3 | M0-3-6 | 3 | Eve — Generative Intelligence | Cit/Vāk, divine speech; the Articulator | `HAS_ZODIACAL_COMPONENT` ×12 (the grammar-twelve); `LINGUISTIC_CORRESPONDENCE` ×99 |
| 4 | M0-3-7 | 4 | Adam — Structural Completion | Pūrṇatā, the quaternion; stable container of form | `CONTAINS_ARCHETYPAL_FOUNDATION` ×3 → 0/1, 2, 3 |
| 5 | M0-3-8 | 5 | Eve — Generative Relationship | Śiva-Śakti / Mono-Poly; opposites held in creative tension | `HAS_MONO_POLY_COMPONENT` ×7 with `c_0_mono_poly_role` ladder |
| 6 | M0-3-9 | 6 | Adam — Structural Resolution | Śūnyatā-Pūrṇatā; all complexity resolves to the void — the Return | `ADAM_CHARGE`, `ADAM_EVE_COMPLEMENT`, `VIBRATIONAL_MULTIPLICATION` |
| 7 | M0-3-10 | 7 | Eve — Generative Action | Divine Action / Virtue; the Ananda-Tandava | `HAS_DIVINE_ACT` ×7 (R# foundation + R0-R5, each with `c_1_r_factor_designation` + `c_4_divine_act_category`) |
| 8 | M0-3-11 | 8 | Adam — Structural Contemplation | structural reflection; the structure questions itself | `CONTAINS_HIGHER_DYNAMICS` ×3 → 5, 6, 7; `ARCHETYPAL_STRUCTURE_BRIDGE` |
| 9 | M0-2-9 | 9 = (00+00) | wholeness (no class key) | Paramesvara — the virtue container and crown of the whole set | `HAS_VIRTUE_COMPONENT` ×9 (virtues 0-8); `q_5_primary_determinations`, `q_5_ninefold_determinations`; `CROWNS_VORTEX_SYSTEM` → M1-2 |

The evens 2, 4, 6, 8 are the Adam trunk and the odds 3, 5, 7 the Eve serpent
— the map's own `m_0_3_adam_eve_classification` values, quoted above, not an
interpretation. Two twelves must not be conflated, and the map distinguishes
them itself: the **substance-twelve** (this table) and the **grammar-twelve**
(the 12 `HAS_ZODIACAL_COMPONENT` children of M0-3-6, M0-3-6-0..11, each with
`m_0_3_grammatical_function` — "Actual Identity (!)", "Potential Essence (?)",
… the assertion/query conjugations).

### Role paragraphs (one per number, per the map)

**(-) — The Mirror (M0-3-(0/1)).** First principle of self-awareness: the
capacity of consciousness to become an object to itself while remaining
subject (`m_0_3_consciousness_function`). It is pre-numerical
(`m_0_3_metaphysical_names`: "Pre-Numerical Reflector", "Vimarśa Apparatus")
and has no Adam/Eve classification. Its declared parts are The Frame `()` and
The Operator `-` (`HAS_CONSTITUENT` ×2); its formulation resolves into 0-1D,
"the dimension that unifies 0D (Actuality) and 1D (Potential)". Nineteen
`HOLOGRAPHIC_PATTERN` edges make it the most holographically-connected node
in M0-3 — the mirror relation is its syntax.

**0 (M0-3-2).** The observing consciousness that remains unchanging while all
experience arises in it — Sat, the Silent Witness
(`m_0_3_consciousness_function`). Spanda relationship: the static aspect of
Sat-Spanda, "the motionless ground from which the pulse of existence arises".
Transcendent Root (neither Adam nor Eve). Its declared dynamics name pure
potential and its manifestation.

**1 (M0-3-3).** The will-to-manifest, the "Am" that follows the "I" — first
form and active agency. Spanda relationship: the active aspect of Sat-Spanda,
"the first pulse that initiates the rhythm of manifestation". Transcendent
Root. Declared dynamics: hidden agency awakening, active agency manifesting.

**0/1 (M0-3-4).** The non-dual binary: awareness simultaneously empty and
full, witness and agent (`m_0_3_consciousness_function`). The map names it
"Engine of the System" and "Rosetta Stone of the Logos"
(`m_0_3_metaphysical_names`). It declares 0, 1 and (-) as its constituents
(`CONTAINS_AS_CONSTITUENT` ×3) — the paradox resolver that contains its own
poles — and its formulation states the core identity `00 = 0/1 = 0 =/≠ 1`.

**2 (M0-3-5).** Adam, Structural Foundation. The relational field itself:
Śūnyatā, "the Between", dialogical space — consciousness recognising an other
while maintaining unity. Spanda relationship: "the first structural container
for Spanda's pulsation". Its formulation spans the entire mod-6 cycle
(2.0-5), "indicating completeness" per the breakdown.

**3 (M0-3-6).** Eve, Generative Intelligence. Cit and Vāk: consciousness
articulating itself, "the twelve modes of divine speech that enable complete
self-expression" (`m_0_3_consciousness_function`) — the grammar-twelve are
those modes as `HAS_ZODIACAL_COMPONENT` children, each with a
`m_0_3_grammatical_function` and a `m_0_3_consciousness_operation`
("the first bite of divine consciousness, separating thing from no-thing",
for M0-3-6-0). Its 99 `LINGUISTIC_CORRESPONDENCE` edges are the declared
speech relations.

**4 (M0-3-7).** Adam, Structural Completion. Pūrṇatā, the quaternion: "the
capacity for consciousness to establish stable, enduring forms that can
contain and express higher spiritual energies — the psychological container
for individuation". It declares 0/1, 2 and 3 as its archetypal foundation
(`CONTAINS_ARCHETYPAL_FOUNDATION` ×3): completion built on the transcendent
binary, the field, and speech.

**5 (M0-3-8).** Eve, Generative Relationship. Śiva-Śakti, Mono-Poly,
quintessence: "the capacity for consciousness to hold opposites in creative
tension without collapsing them into false unity or fragmenting them into
disconnected multiplicity". Spanda relationship: "the number where Spanda
becomes most visible". Its internal dynamics are declared as seven
`HAS_MONO_POLY_COMPONENT` children with `c_0_mono_poly_role` values forming a
ladder — Unity Foundation → Multiplicity Expression → Actualized Diversity →
Potential Unity → Actualizing Unity → Potentiating Many → Synthesis
Achievement.

**6 (M0-3-9).** Adam, Structural Resolution. Śūnyatā-Pūrṇatā, the Fullness of
Emptiness: "the capacity for consciousness to recognize that all complexity
and manifestation ultimately resolves back into the simple, fertile emptiness
from which it arose" — the Solve process, the Return. Its formulation's final
equation `((@/-)(-/@)) = 00` states that resolution back to the void.

**7 (M0-3-10).** Eve, Generative Action. Divine Action itself: "the capacity
for consciousness to act in the world through organized, virtuous principles
rather than random or chaotic impulses" — Ananda-Tandava, "The 5 Acts + 2
Principles", Virtue (`m_0_3_metaphysical_names`). Its internal structure is
the R-ladder: seven `HAS_DIVINE_ACT` children, the R# Reality-Matrix
foundation plus R0 Creation, R1 Sustenance, R2 Dissolution, R3 Veiling, R4
Grace, R5 Absorption — each child carrying both
`c_1_r_factor_designation` and `c_4_divine_act_category` ("Primary Cosmic
Act"; foundation and culmination marked separately). This node is where the
map joins number to virtue.

**8 (M0-3-11).** Adam, Structural Contemplation. Pūrṇatā-Sūnyatā inverted —
the Emptiness of Fullness, the Empty Throne: "the capacity for consciousness
to turn back upon itself and question its own nature, even in states of
apparent perfection and completion". It declares the higher dynamics of 5, 6,
7 as contained (`CONTAINS_HIGHER_DYNAMICS` ×3) — reflection containing the
union, the return, and the action it reflects on. This is the archetype the
map `NESTS_ARCHETYPE_8`s directly from M0-2-9.

**9 — Paramesvara (M0-2-9).** Wholeness: symbol `9 = (00+00)`, "Principle 9",
standing at the terminus of the integer series "expressing completion
determining itself into ninefold differentiation"
(`q_5_primary_determinations`). It declares the nine virtues as components
(`HAS_VIRTUE_COMPONENT` ×9 → M0-2-9-0..8: Love/Peace, Truth,
Openness/Creativity, Joy/Play, Goodness, Beauty, Life/Nature, Wisdom,
Reality), carries the ninefold determination texts (`q_5_ninefold_determinations`),
and — per its own edges — crowns the Ananda vortex (`CROWNS_VORTEX_SYSTEM` →
M1-2) and contains the archetypal number language itself
(`CONTAINS_ARCHETYPAL_NUMBER` ×10, `NESTS_ARCHETYPE_8`,
`NESTS_COMPLETE_NUMBER_LANGUAGE` → M0-3). The nine is the container in which
the other twelve-and-the-field cohere; section 5's EBM field is its declared
value structure.

**Standing note for tier design.** All twelve carry complete formulations, so
the Jev tiers over the numbers run from the map's own formulation + breakdown
texts plus the internal-dynamics properties tabulated above. The wider M0-3
tree (40 nodes) carries formulations on 13; tiers over leaves there run from
head formulations and leaf properties — a statement about what a tier must
read, not about the map.

## 3. Layer: representations

**3a. Node embeddings (in place).** 3072-d `gemini-embedding-2`, composed from
full graph content including all `c_1_*` formulation keys (prefix law in
`compose_from_graph.py`). Kept whole; property-subset A/Bs retired — the
coherence test (parent ranking 0.723 full vs 0.511 symbol-only) settled that
the field is the compiled+expanded whole.

**3b. Relation embeddings — Jev's knowledge of declared relations
(new, first-class).** Edge-level embeddings are the fix for reader limitation:
if Jev retains them, it knows the relations between nodes from what the map
declares about them, rather than inferring relation from node similarity.
The declared corpus (census verified 2026-09-21):

| Edge family | Count | Declared content keys |
|---|---|---|
| `CAUSAL_RESONANCE` | 660 | `c_1_relation_description` on all 660 |
| `LINE_CHANGE` | 383 | `c_1_relation_description` on all 383 |
| `HOLOGRAPHIC_PATTERN` | 102 | 10 keys per edge; the content keys are `p_3_pattern_name`, `p_3_pattern_structure`, `s_4_function_role`, `t_5_insight` |
| `GRAMMATICAL_CORRESPONDENCE` | 99 | `c_5_correspondence`, `l_5_realization_level`, `l_5_mystical_identity` |
| `LINGUISTIC_CORRESPONDENCE` | 99 | the same three plus `t_3_developmental_function` |
| `VIRTUE_R_FACTOR_CORRESPONDENCE` | 7 | `c_1_relation_description` on all 7 |

Design: compose the edge state from (relation family, type, description /
correspondence keys, endpoint coordinates + names) and embed via `bimba_embed`
**without `store_for`** — the verified read-only path. Vectors are held in the
field store keyed by edge digest. The graph is untouched. Structural-only
types (POLAR_OPPOSITE, FLOWS_CLOCKWISE, the membership and progression edges
of section 5 — their edges carry metadata keys only) stay typed structure:
they enter the instruments as constraints and priors, not embeddings.
Retrieval is by edge digest, so a Jev question can carry the declared
relations of the nodes it touches as first-class knowledge. Storage layout in
the field store is OWNER QUESTION Q3 (unchanged from v1).

**3c. The mark table (109 — pending owner designation).** 107 M0-scope nodes
carry `c_1_symbol`. The two open entries await the owner's designation, not a
map fix: **M0 root** (strongest candidate `0000` from `c_3_context_frame`,
consistent with child M0-0's `'(0000) -'`) and **M0' Anuttara'** (a bare
structural mirror placeholder from the M5 thread migration — no content;
candidate `0000'`, or exclusion for a true count of 108). OWNER QUESTION Q1
(unchanged from v1).

## 4. Layered classification

```text
layer 1   ANUTTARA SYNTAX CLASSIFICATION            (Jev)
          over the 109 marks / formulations /
          the twelve branches of M0-3
          output: archetype verdict (over the twelve),
                  R-factor (over R# + R0-R5),
                  mark / breakdown analyses
                    │
                    ▼
layer 2   ANANDA VORTEX CLASSIFICATION              (the vortex's own role)
          M1-2 parses layer-1 output RELATIVE TO
          the vortex's numerical relations:
            6 families x 12x12, DR rings
            [1,2,4,8,7,5] / [3,6,9,3,6,9],
            3-6-9 spirit axis, positions 10/11 = 0/1 and (-)
                    │
                    ▼
          VIRTUE-FIELD AND GENERAL-MAP SCORING      (EBM)
          energy as evidence, section 5
                    │
                    ▼
          RUST — recurrence, budgets, loop law
          determination pending-human-review
```

**Layer 1 — Anuttara syntax (Jev).** Input: the mark table (3c) and the role
table (section 2). Jev classifies an expression, state or act: which mark
computes it, which formulation it realises, which of the twelve archetypes
computes it, which R-factor it carries. The role table is the specification
these tiers are written against — each tier's choice set and state material
come from the declared properties and dynamics of the numbers, not from
similarity over node text.

**Layer 2 — Ananda vortex (its standing role).** M1-2 already occupies this
position in the system and v2 does not assign it a new one. The map declares
the seam itself: M0-2-9 `CROWNS_VORTEX_SYSTEM` → M1-2 "Ananda" (symbol:
"Toroidal Six-Matrix Vortex with 3-6-9 Spirit Axis"; `c_1_description`: six
core archetypal matrices and their digital-root reflections forming a closed
loop). Architecturally it is a **second classification layer over the Anuttara
layer**: it takes layer-1's classifications and parses them relative to the
numerical relations in the vortices — digital-root ring membership, family
placement, the seat of each number (the kernel's positions 10/11 are 0/1 and
(-), matching the transcendent entries of the role table; this alignment is an
inference drawn from the kernel's own fixtures, consistent with the map's
`CONTAINS_ARCHETYPAL_NUMBER` structure). The QL kernel at
`../Quaternal-Logic/crates/ql-mef/src/m1.rs` and `m1_engine.rs` (6 families ×
12×12, 864-cell literal fixtures, DR rings) **is the vortex's own
computational form**: layer 2 reads its declared arithmetic and checks its own
placements against it as a consistency reading of one declared structure. It
is never an authority ruling layer-1 verdicts in or out, and nothing in the
kernel is repurposed. The graph-side overlay seam noted in v1
(`graph-services` writing `m_1_2_ananda_vortex_*` properties) remains unused
by v2 — no map writes.

## 5. Layer: instruments

**5a. EBM — the compatibility field over declared value structure.** First
object: **the virtue field (M0-2-9)**. Verified shape (2026-09-21):

- Membership: `HAS_VIRTUE_COMPONENT` ×9 (children M0-2-9-0..8); a parallel
  `GENERATIVE_SYNTAX_FLOW` ×9 to the same children. Both metadata-keyed only —
  typed structure.
- Internal order: the `DIVINE_ACT_PROGRESSION` ordinal chain ×5 (Joy/Play 3 →
  Goodness 4 → Beauty 5 → Life/Nature 6 → Wisdom 7 → Reality 8); the virtue
  structural edges `VIRTUE_FOUNDATION` (0→1), `VIRTUE_STRUCTURAL_BASIS`
  (1→2), `VIRTUE_SYNTHESIS_BRIDGE` (1→3), `VIRTUE_SYNTHESIS_MEDIATION`
  (2→3, 2→4, 2→5), and `VIRTUE_RETURN_TO_SOURCE` (8→1).
- Graded correspondences: `VIRTUE_R_FACTOR_CORRESPONDENCE` ×7, each with
  `c_1_relation_description` — virtues 3-8 → R0-R5 at M0-3-10-2..7 (1:1), plus
  virtue 0 Love/Peace → M0-1. Virtues 1 (Truth) and 2 (Openness/Creativity)
  carry no R-factor edge; the ninefold text pairs Truth with the `##`
  foundation and gives `#R` its own entry. This partiality is the map's
  declared shape and is kept as declared.
- The ninefold determinations (`q_5_ninefold_determinations`, a 9-entry list):
  entries run Love/Peace, Truth, then Joy/Play-as-0R through Reality-as-5R,
  then `#R` as the structural determination principle. Alignment to children
  is therefore offset, not index-identity: entries 0-1 ↔ children 0-1, entries
  2-7 ↔ children 3-8, entry 8 (`#R`) is text-only with no node.
- Cross-system edges: `*_VIRTUE_CORRESPONDENCE` ×22 into the M2-4.0-(0/1) 99-
  names tree, with `l_5_realization_level` on 11/22 (the graded labels the
  map actually declares); `NESTS_VIRTUE` ×9 from M0-4.(5/0)-4 to all nine
  virtues; one declared adverse exemplar, `SHADOW_VIRTUE`: Raga Tattva →
  Joy/Play; wholeness edges `QUATERNAL_SUPREME_INTEGRATION` ×3 (M0-0-0,
  M0-0-1, M0-1-(0/1)) and the crowning edge to M1-2.

Energy = compatibility with this declared architecture: positives are the
typed edges and the ordinal constraints; the field scores candidate states
and acts as more or less viable *in relation to the ninefold*. Low energy is
not value, correctness or worth.

Second object: the general map field, with the relation embeddings (3b)
carrying relationality — candidate relation (a, r, b) energy from
edge-embedding compatibility plus typed-edge priors plus declared-structure
constraints.

**Testing, per the owner's correction:** every relational claim is tested
against **declared structure and scrambled-relation text** — the true declared
edge for endpoints (a, b) versus relation text with destroyed or shuffled
content for the same endpoints, ordinal monotonicity along the act chain, and
the declared shadow asymmetry (Raga → Joy/Play declared adverse). Sibling
swaps are not used as the relationality test. The historical corruption-ladder
results in `ebm/README.md` remain past evidence with their own standing; they
are not the v2 standard.

**5b. Jev — the Anuttara layer instrument.** Proven live (2026-09-20):
formulation→mark 36/40 = 90% (chance 25%); formulation→own breakdown 30/30
(chance 50%). Next tiers, designed against section 2:

- hard tiers (sibling-mark distractors, same-family breakdowns);
- archetype classification: choice over the twelve, state material = the
  role table's formulations, breakdowns and declared dynamics;
- R-factor identification: choice over M0-3-10's own seven
  (R# foundation, R0-R5);
- virtue determination: choice over the nine virtue children of M0-2-9, with
  the ninefold offset (5a) stated in the tier's criteria, and shadow
  determination as a typed noul.

**5c. The EBM ↔ Jev relation.** **Jev classifies** — archetype, R-factor,
virtue determination. **EBM scores** the classification's placement in the
virtue field and the general map field, energy attached as evidence. **Rust
owns the recurrence** — the composed-loop law, refusals typed, determination
pending-human-review. Layer 2 (section 4) sits between the first two: the
vortex parses Jev's classifications relative to its numerical relations
before placement is scored.

## 6. Verification before testing (the gate order)

1. **Spec verification.** Machine-checked, evidence-pinned, before any
   instrument runs:
   - role-table diff: each of the twelve rows re-verified against the live
     properties it cites (the exact keys of section 2), plus the anchoring
     edges `CONTAINS_ARCHETYPAL_NUMBER` ×10, `NESTS_ARCHETYPE_8`,
     `NESTS_COMPLETE_NUMBER_LANGUAGE`;
   - mark-table diff: 109 complete with exact sources, M0 = `0000` and
     M0' = `0000'` as designated by the owner (Q1), else 108 with M0'
     excluded — the table is built from whatever the owner designates;
   - relation-corpus census: per-family counts and key coverage re-pinned at
     build time (3b figures are the 2026-09-21 census);
   - virtue-field edge census: the typed edges of 5a re-counted.
2. **Instrument verification.** Jev hard-tier calibration before any headline
   number; EBM validity on the virtue field via the declared-structure tasks
   of 5a (declared edge above scrambled-relation text; ordinal monotonicity;
   shadow asymmetry); relation-embedding validity likewise on the general
   field.
3. **Ananda consistency reading.** Layer-2 placements checked against
   `ql_mef::m1` cell arithmetic on a fixture battery — two computations of
   one declared structure compared for consistency. This is a reading, not a
   verdict: it rules on layer 2's parsing, never on Jev's layer-1
   classifications, and never as an authority over the map.
4. Only then: capability tiers and the composed-loop connection
   (`EBM-GATE-DESIGN.md` shadow mode remains the wiring path).

## 7. What this document does not do

It makes no claim of authority over the map and proposes no change to it —
no materialisation, no new edges, no schema suggestions. Where a reading is
incomplete it says so as a reader limitation and addresses it with better
instruments (relation embeddings, tier state design). Open owner questions
carried from v1: **Q1** the final two mark entries (M0, M0'); **Q3** field-
store layout for relation vectors. v1's former Q2 (map writes to
materialise the twelve) is withdrawn — the twelve needs no materialisation —
and former Q4 is answered by the map itself: the virtue-R partiality and the
ninefold offset are declared shape (5a), not an open defect.
