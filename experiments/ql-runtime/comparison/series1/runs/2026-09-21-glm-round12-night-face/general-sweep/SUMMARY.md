# MEF/jev general-corpus sweep — SUMMARY (2026-09-21)

Instrument: `jev-reflect.mjs` + `mef-giving.json` (canonical MEF register, 12 lenses x 6
sub-slots, meanings carried). Variant: `jev-reflect-nameonly.mjs` (identical file except
the full_text mode: sub-slot labels without meanings, machinery enumeration dropped from
the instructions). Corpus: 12 real positions (C1..C12), subject = statement + core move,
heading stripped. 7 calls per entry (full_text x2, nameonly x1, resonant x1, being /
becoming / knowing x1 each) = 84 calls. All 84 succeeded first-try (latency 0.69-1.64s,
no retries, no invalid JSON). Raw responses: `c<NN>-<mode>-<rep>.json` in this directory;
run record: `run-log.json`.

Method note: an initial parser read the whole corpus as C1's subject (the corpus has no
`---` separators between entries, only one at the top); those 7 files were deleted and
re-run against the corrected per-entry subject before any analysis. Every number below
comes from the corrected run.

Names are shortened below (e.g. "L1-3 Formal" = L1-3 Formal Cause; "L5-3' Sophia" =
L5-3' Sophia (σοφία)). "machinery" = full_text with meanings carried; "name-only" =
labels only.

---

## a) Per-entry readings

Format per entry: strongest sub-slot per lens under machinery full_text (rep 1; rep 2
agreed everywhere except where marked), the name-only pick where it differs (marked
`no:`), resonant top-3, and the three square readings' standouts (strongest slot per
question at its position).

### C1. Utilitarianism — Bentham
| lens | strongest |
|---|---|
| L0 | L0.1 What (definitional) |
| L1 | L1-4 Final Cause |
| L2 | L2-1 IS |
| L3 | L3-5 Satisfaction / Perishing (no: L3-3 Eternal Objects) |
| L4 | L4.4 Besorge |
| L5 | L5-5 Mātṛkā 0.85 (no: L5-4 Vaikharī) |
| L0' | L0-1' Two |
| L1' | L1-3' Thinking |
| L2' | L2-5' Salt (no: L2-0' Aether) |
| L3' | L3-1' Spring (no: L3-4' Winter) |
| L4' | L4.3' Patterns |
| L5' | L5-3' Sophia (no: L5-5' Epi-Logos) |

Resonant top-3: L1 Causal 0.36 · L4' Scientific 0.23 · L0' Archetypal-Numerical 0.11.
being: L0@P0=L0.0 Why; L0@P5=L0.1 What; L5@P0=L5-5 Mātṛkā; L5@P5=L5-5 Mātṛkā.
becoming: L1@P1=L1-4 Final; L1@P4=L1-4 Final; L4@P1=L4.1 Geworfenheit; L4@P4=L4.0 Sein.
knowing: L2@P2=L2-1 IS; L2@P3=L2-1 IS; L3@P2=L3-5 Satisfaction; L3@P3=L3-5 Satisfaction.

### C2. The invisible hand — Smith
| lens | strongest |
|---|---|
| L0 | L0.2 How (operational) |
| L1 | L1-3 Formal Cause (no: L1-2 Efficient) |
| L2 | L2-1 IS (no: L2-4 NEITHER) |
| L3 | L3-4 Community Integration |
| L4 | L4.5 Gelassenheit (no: L4.4 Besorge) |
| L5 | L5-2 Paśyantī (rep2 flipped to L5-3 Madhyamā) |
| L0' | L0-2' Three (no: L0-4' Five) |
| L1' | L1-4' Intuition |
| L2' | L2-0' Aether |
| L3' | L3-0' Spirit (Geist) |
| L4' | L4.5' Insight (no: L4.3' Patterns) |
| L5' | L5-3' Sophia 0.91 (no: L5-5' Epi-Logos) |

Resonant top-3: L1 Causal 0.51 · L3 Processual 0.28 · L4' Scientific 0.09.
being: L0@P0=L0.2 How; L0@P5=L0.2 How; L5@P0=L5-3 Madhyamā; L5@P5=L5-3 Madhyamā.
becoming: L1@P1=L1-3 Formal; L1@P4=L1-4 Final; L4@P1=L4.5 Gelassenheit; L4@P4=L4.5 Gelassenheit.
knowing: L2@P2=L2-3 BOTH; L2@P3=L2-3 BOTH; L3@P2=L3-4 Community; L3@P3=L3-4 Community.

### C3. Entitlement theory — Nozick
| lens | strongest |
|---|---|
| L0 | L0.0 Why (presuppositional frame) (no: L0.5 Why-so/Why-not) |
| L1 | L1-0 Svātantrya (essential freedom causation) |
| L2 | L2-1 IS |
| L3 | L3-1 Actual Occasion |
| L4 | L4.5 Gelassenheit 0.71 (no: L4.3 Zeit) |
| L5 | L5-4 Vaikharī |
| L0' | L0-1' Two (no: L0-0' One) |
| L1' | L1-3' Thinking |
| L2' | L2-3' Air (no: L2-0' Aether) |
| L3' | L3-2' Summer (no: L3-0' Spirit) |
| L4' | L4.5' Insight (no: L4.3' Patterns) |
| L5' | L5-0' Arche (no: L5-5' Epi-Logos) |

Resonant top-3: L1 Causal 0.61 · L2 Logical 0.19 · L4' Scientific 0.07.
being: L0@P0=L0.0 Why; L0@P5=L0.0 Why; L5@P0=L5-4 Vaikharī; L5@P5=L5-4 Vaikharī.
becoming: L1@P1=L1-0 Svātantrya; L1@P4=L1-0 Svātantrya; L4@P1=L4.5 Gelassenheit; L4@P4=L4.5 Gelassenheit.
knowing: L2@P2=L2-2 IS-NOT; L2@P3=L2-2 IS-NOT; L3@P2=L3-4 Community; L3@P3=L3-4 Community.

### C4. The partnership of generations — Burke
| lens | strongest |
|---|---|
| L0 | L0.4 Where/When/Why-for (contextual) (no: L0.3 Whom/Which/When) |
| L1 | L1-4 Final Cause |
| L2 | L2-1 IS (no: L2-2 IS-NOT) |
| L3 | L3-4 Community Integration |
| L4 | L4.3 Zeit (temporality) |
| L5 | L5-2 Paśyantī (no: L5-1 Parā Vāk) |
| L0' | L0-2' Three |
| L1' | L1-4' Intuition |
| L2' | L2-0' Aether |
| L3' | L3-0' Spirit (Geist) |
| L4' | L4.5' Insight |
| L5' | L5-3' Sophia (no: L5-5' Epi-Logos) |

Resonant top-3: L3' Chronological 0.66 · L4 Phenomenological 0.19 · L0 Quaternal 0.05.
being: L0@P0=L0.4 Where/When/Why-for; L0@P5=L0.4 Where/When/Why-for; L5@P0=L5-2 Paśyantī; L5@P5=L5-3 Madhyamā.
becoming: L1@P1=L1-4 Final; L1@P4=L1-4 Final; L4@P1=L4.3 Zeit; L4@P4=L4.3 Zeit.
knowing: L2@P2=L2-3 BOTH; L2@P3=L2-3 BOTH; L3@P2=L3-4 Community; L3@P3=L3-4 Community.

### C5. Government by consent — Locke
| lens | strongest |
|---|---|
| L0 | L0.4 Where/When/Why-for |
| L1 | L1-4 Final Cause |
| L2 | L2-1 IS |
| L3 | L3-4 Community Integration |
| L4 | L4.4 Besorge |
| L5 | L5-4 Vaikharī |
| L0' | L0-2' Three (no: L0-1' Two) |
| L1' | L1-3' Thinking 0.93 (no: L1-4' Intuition) |
| L2' | L2-3' Air (rep2 flipped to L2-5' Salt; no: L2-0' Aether) |
| L3' | L3-1' Spring (no: L3-0' Spirit) |
| L4' | L4.5' Insight |
| L5' | L5-3' Sophia (no: L5-5' Epi-Logos) |

Resonant top-3: L1 Causal 0.89 · L0 Quaternal 0.05 · L5' Divine Logos 0.02.
being: L0@P0=L0.0 Why; L0@P5=L0.4 Where/When/Why-for; L5@P0=L5-3 Madhyamā; L5@P5=L5-4 Vaikharī.
becoming: L1@P1=L1-4 Final; L1@P4=L1-4 Final; L4@P1=L4.4 Besorge; L4@P4=L4.4 Besorge.
knowing: L2@P2=L2-2 IS-NOT; L2@P3=L2-1 IS; L3@P2=L3-4 Community; L3@P3=L3-4 Community.

### C6. Historical materialism — Marx
| lens | strongest |
|---|---|
| L0 | L0.0 Why |
| L1 | L1-2 Efficient Cause 0.65 (no: L1-1 Material Cause 0.64) |
| L2 | L2-1 IS |
| L3 | L3-4 Community Integration 0.89 (no: L3-1 Actual Occasion) |
| L4 | L4.1 Geworfenheit (thrownness) |
| L5 | L5-4 Vaikharī |
| L0' | L0-2' Three (no: L0-3' Four) |
| L1' | L1-3' Thinking |
| L2' | L2-1' Earth (only Earth pick in the corpus) |
| L3' | L3-0' Spirit (Geist) (no: L3-3' Autumn) |
| L4' | L4.3' Patterns |
| L5' | L5-2' Dynamis (creative power) |

Resonant top-3: L1 Causal 0.63 · L3' Chronological 0.32 · L0 Quaternal 0.02.
being: L0@P0=L0.0 Why; L0@P5=L0.0 Why; L5@P0=L5-4 Vaikharī; L5@P5=L5-4 Vaikharī.
becoming: L1@P1=L1-2 Efficient; L1@P4=L1-2 Efficient; L4@P1=L4.1 Geworfenheit; L4@P4=L4.1 Geworfenheit.
knowing: L2@P2=L2-1 IS; L2@P3=L2-1 IS; L3@P2=L3-4 Community; L3@P3=L3-4 Community.

### C7. The tragedy of the commons — Hardin
| lens | strongest |
|---|---|
| L0 | L0.5 Why-so/Why-not (tie 0.25 with L0.0 Why) (no: L0.2 How) |
| L1 | L1-3 Formal Cause 0.64 (no: L1-2 Efficient) |
| L2 | L2-1 IS |
| L3 | L3-4 Community Integration |
| L4 | L4.4 Besorge |
| L5 | L5-4 Vaikharī (rep2 flipped to L5-3 Madhyamā) |
| L0' | L0-2' Three |
| L1' | L1-3' Thinking (no: L1-4' Intuition) |
| L2' | L2-4' Fire (no: L2-3' Air) |
| L3' | L3-3' Autumn 0.71 (no: L3-0' Spirit) |
| L4' | L4.5' Insight |
| L5' | L5-3' Sophia (no: L5-5' Epi-Logos) |

Resonant top-3: L1 Causal 0.67 · L4' Scientific 0.16 · L2 Logical 0.07.
being: L0@P0=L0.0 Why; L0@P5=L0.2 How; L5@P0=L5-4 Vaikharī; L5@P5=L5-4 Vaikharī.
becoming: L1@P1=L1-3 Formal; L1@P4=L1-3 Formal; L4@P1=L4.4 Besorge; L4@P4=L4.4 Besorge.
knowing: L2@P2=L2-3 BOTH; L2@P3=L2-3 BOTH; L3@P2=L3-4 Community; L3@P3=L3-4 Community.

### C8. Two systems of thinking — Kahneman
| lens | strongest |
|---|---|
| L0 | L0.2 How |
| L1 | L1-3 Formal Cause 0.95 (no: L1-2 Efficient 0.75) |
| L2 | L2-1 IS 0.77 (no: L2-3 BOTH) |
| L3 | L3-1 Actual Occasion (no: L3-3 Eternal Objects) |
| L4 | L4.2 Dasein |
| L5 | L5-3 Madhyamā |
| L0' | L0-1' Two |
| L1' | L1-3' Thinking |
| L2' | L2-3' Air |
| L3' | L3-2' Summer (no: L3-0' Spirit) |
| L4' | L4.3' Patterns |
| L5' | L5-3' Sophia 0.86 (no: L5-5' Epi-Logos) |

Resonant top-3: L4' Scientific 0.46 · L3 Processual 0.23 · L1 Causal 0.11 (the only
Scientific-topped entry in the corpus).
being: L0@P0=L0.2 How; L0@P5=L0.2 How; L5@P0=L5-3 Madhyamā; L5@P5=L5-3 Madhyamā.
becoming: L1@P1=L1-3 Formal; L1@P4=L1-3 Formal; L4@P1=L4.2 Dasein; L4@P4=L4.2 Dasein.
knowing: L2@P2=L2-3 BOTH; L2@P3=L2-1 IS; L3@P2=L3-1 Actual Occasion; L3@P3=L3-1 Actual Occasion.

### C9. Linguistic relativity — Whorf
| lens | strongest |
|---|---|
| L0 | L0.2 How |
| L1 | L1-3 Formal Cause |
| L2 | L2-1 IS |
| L3 | L3-2 Ingression (eternal objects entering occasions) |
| L4 | L4.1 Geworfenheit |
| L5 | L5-3 Madhyamā 0.74 (no: L5-4 Vaikharī) |
| L0' | L0-2' Three |
| L1' | L1-4' Intuition |
| L2' | L2-3' Air (intellect, communication) |
| L3' | L3-0' Spirit (Geist) |
| L4' | L4.3' Patterns |
| L5' | L5-3' Sophia (no: L5-5' Epi-Logos) |

Resonant top-3: L5 Para Vāk 0.48 · L4 Phenomenological 0.28 · L2 Logical 0.05 (the only
speech-lens-topped entry — for the language entry).
being: L0@P0=L0.0 Why; L0@P5=L0.2 How; L5@P0=L5-3 Madhyamā; L5@P5=L5-3 Madhyamā.
becoming: L1@P1=L1-3 Formal; L1@P4=L1-3 Formal; L4@P1=L4.1 Geworfenheit; L4@P4=L4.1 Geworfenheit.
knowing: L2@P2=L2-0 Tetralemmaic Ground; L2@P3=L2-0 Tetralemmaic Ground; L3@P2=L3-2 Ingression; L3@P3=L3-2 Ingression.

### C10. Sense and reference — Frege
| lens | strongest |
|---|---|
| L0 | L0.1 What (no: L0.2 How) |
| L1 | L1-3 Formal Cause |
| L2 | L2-3 BOTH 0.46 (no: L2-1 IS) |
| L3 | L3-3 Eternal Objects (no: L3-1 Actual Occasion) |
| L4 | L4.2 Dasein (no: L4.0 Sein) |
| L5 | L5-3 Madhyamā |
| L0' | L0-1' Two |
| L1' | L1-3' Thinking (no: L1-4' Intuition) |
| L2' | L2-3' Air 0.88 (no: L2-0' Aether) |
| L3' | L3-0' Spirit (rep2 flipped to L3-1' Spring) |
| L4' | L4.5' Insight |
| L5' | L5-3' Sophia (no: L5-5' Epi-Logos) |

Resonant top-3: L2 Logical 0.81 · L4 Phenomenological 0.05 · L0 Quaternal 0.03 (the
strongest lens concentration in the corpus — for the logic entry).
being: L0@P0=L0.1 What; L0@P5=L0.1 What; L5@P0=L5-3 Madhyamā; L5@P5=L5-3 Madhyamā.
becoming: L1@P1=L1-3 Formal; L1@P4=L1-3 Formal; L4@P1=L4.0 Sein; L4@P4=L4.0 Sein.
knowing: L2@P2=L2-3 BOTH; L2@P3=L2-3 BOTH; L3@P2=L3-3 Eternal Objects; L3@P3=L3-3 Eternal Objects.

### C11. The unconscious — Freud
| lens | strongest |
|---|---|
| L0 | L0.2 How |
| L1 | L1-3 Formal Cause 0.73 (no: L1-5 Icchā Śakti / Will) |
| L2 | L2-3 BOTH 0.49 (no: L2-1 IS) |
| L3 | L3-0 Concrescent Desire (creative urge / lack) |
| L4 | L4.2 Dasein |
| L5 | L5-3 Madhyamā 0.93 (no: L5-2 Paśyantī) |
| L0' | L0-1' Two |
| L1' | L1-0' Introversion 0.97 (toward-unconscious-pole; no: L1-4' Intuition) |
| L2' | L2-0' Aether |
| L3' | L3-4' Winter 0.76 (no: L3-0' Spirit) |
| L4' | L4.5' Insight (no: L4.3' Patterns) |
| L5' | L5-1' Apokalypsis 0.48 (unveiling; no: L5-5' Epi-Logos) |

Resonant top-3: L3 Processual 0.83 · L1 Causal 0.13 · L0 Quaternal 0.02 (the strongest
single resonant mass in the corpus).
being: L0@P0=L0.0 Why; L0@P5=L0.1 What; L5@P0=L5-3 Madhyamā; L5@P5=L5-3 Madhyamā.
becoming: L1@P1=L1-3 Formal; L1@P4=L1-3 Formal; L4@P1=L4.2 Dasein; L4@P4=L4.2 Dasein.
knowing: L2@P2=L2-3 BOTH; L2@P3=L2-2 IS-NOT; L3@P2=L3-0 Concrescent Desire; L3@P3=L3-0 Concrescent Desire.

### C12. Probability as degree of belief — Ramsey
| lens | strongest |
|---|---|
| L0 | L0.1 What (no: L0.2 How) |
| L1 | L1-3 Formal Cause (no: L1-5 Icchā Śakti / Will 0.71) |
| L2 | L2-2 IS-NOT (uncertainty) |
| L3 | L3-1 Actual Occasion |
| L4 | L4.4 Besorge (concern, engaged action) |
| L5 | L5-3 Madhyamā (near-tie 0.43/0.41 with L5-5 Mātṛkā; no: L5-2 Paśyantī) |
| L0' | L0-1' Two |
| L1' | L1-3' Thinking |
| L2' | L2-3' Air (no: L2-0' Aether) |
| L3' | L3-2' Summer (no: L3-0' Spirit) |
| L4' | L4.5' Insight |
| L5' | L5-3' Sophia (no: L5-5' Epi-Logos) |

Resonant top-3: L2 Logical 0.62 · L4' Scientific 0.17 · L1 Causal 0.09.
being: L0@P0=L0.1 What; L0@P5=L0.1 What; L5@P0=L5-3 Madhyamā; L5@P5=L5-5 Mātṛkā.
becoming: L1@P1=L1-3 Formal; L1@P4=L1-3 Formal; L4@P1=L4.4 Besorge; L4@P4=L4.4 Besorge.
knowing: L2@P2=L2-2 IS-NOT; L2@P3=L2-2 IS-NOT; L3@P2=L3-1 Actual Occasion; L3@P3=L3-1 Actual Occasion.

---

## b) Stability (machinery full_text, rep 1 vs rep 2)

Per entry (lenses agreeing on strongest sub-slot, of 12):
C1 12, C2 11, C3 12, C4 12, C5 11, C6 12, C7 11, C8 12, C9 12, C10 11, C11 12, C12 12.

**Overall: 140/144 = 97.2%.** All four disagreements were flips to an adjacent or
sibling slot, never a leap: C2 and C7 L5 (Paśyantī ↔ Madhyamā; Vaikharī ↔ Madhyamā),
C5 L2' (Air ↔ Salt), C10 L3' (Spirit ↔ Spring). The instrument is effectively
deterministic at the strongest-slot level for this corpus.

## c) Machinery vs name-only

Per entry (lenses picking the SAME strongest slot under both forms, of 12):
C1 7, C2 6, C3 5, C4 8, C5 7, C6 8, C7 6, C8 7, C9 10, C10 5, C11 5, C12 6.

**Overall: 80/144 = 55.6%.** Running the machinery vs naming the labels are genuinely
different acts — the register's claim is confirmed, and the divergence is not noise:

- The day lenses mostly agree (L0 12/12, L2 8/12, L3 8/12, L4 8/12, L1 6/12, L5 6/12).
- The night lenses collapse without meanings: under name-only, **L5' answers Epi-Logos
  12/12** (machinery varied: Sophia x9, Arche, Dynamis, Apokalypsis), **L3' answers
  Spirit (Geist) 10/12**, and L2' drifts to Aether in 5/12 (machinery gave five
  different elements). Stripped of meanings, the model gravitates to the grandest
  label in the set; the meanings are what force the reading through the subject.

Concrete examples, judged against the known core moves:

1. **C8 (Kahneman) L1** — machinery: Formal Cause 0.95; name-only: Efficient Cause
   0.75. The core move is architectural ("split the mind's work between automatic and
   controlled processes, and locate specific errors in specific systems") — Formal
   Cause (pattern, architecture, organizing principle) is the right reading; name-only
   slid to the surface verb "processes". **Machinery clearly better.**
2. **C11 (Freud) L5'** — machinery: Apokalypsis 0.48 (revelation/unveiling); name-only:
   Epi-Logos 0.45 (the label-attraction default). Symptoms, slips and dreams as the
   *return/unveiling* of a hidden process is precisely Apokalypsis. **Machinery
   better**, and it is the machinery reading that varies per-entry here at all.
3. **C12 (Ramsey) L1** — machinery: Formal Cause 0.66 (axioms derived from coherence —
   a formal-structural move); name-only: Icchā Śakti/Will 0.71 (keys on the surface
   phrase "agent's dispositions to act"). The core move is deriving the logic from
   consistency under betting. **Machinery better**, though name-only has a partial
   excuse.
4. One honest counter-example: **C2 (Smith) L2** — machinery: IS 0.43; name-only:
   NEITHER 0.35. The invisible hand — coordination with no coordinator — sits naturally
   on the catuṣkoṭi's NEITHER (void, no-thing that acts). **Name-only's pick is the
   more apt reading here**, whether by genuine reading or lexical luck on "without
   intention or design".
5. A wash worth recording: **C6 (Marx) L1** — machinery: Efficient Cause 0.65
   (forces/relations contradiction as the motor); name-only: Material Cause 0.64 (the
   determining base). The two forms each land on one half of Marx's two-part claim;
   neither is wrong.

## d) Differentiation

**Verdict: the twelve works get distinguishable signatures, but coarsely.**
Every entry used 12 distinct slots (one per lens — never two lenses on one slot), no
(lens, distribution) pair ever repeated exactly across entries, and the resonant lens
tops spread over 7 distinct lenses (L1 x6, L2 x2, L3, L3', L4', L5 x1 each). But at the
full 12-slot signature level the granularity is genre-shaped, not work-shaped:

- **Sharpest separation: C6-C11 (Marx vs Freud): 0/12 shared slots** — every one of
  the twelve lenses separates them (Efficient vs Formal; Geworfenheit vs Dasein;
  Community vs Concrescent Desire; Earth vs Aether; Spirit vs Winter; Thinking vs
  Introversion; Dynamis vs Apokalypsis). Also strongly separated: C6-C12 (1/12),
  C5-C11 (1/12), C1-C11 (1/12), C7-C11 (2/12) — Freud's entry is the corpus's most
  distinctive reader.
- **Most alike: C2-C4 (Smith vs Burke): 9/12** — they differ only in the day row
  (Formal vs Final; How vs Where/When/Why-for; Gelassenheit vs Zeit) and agree on *all
  six night lenses* (IS, Community, Paśyantī, Three, Intuition, Aether, Spirit,
  Insight, Sophia). Two 18th-century social-philosophy texts read nearly identically on
  the night side. Next: C5-C7 and C8-C12 and C10-C12 (8/12 each) — the political/
  economic cluster (Locke-Hardin) and the mind/logic cluster (Kahneman-Ramsey,
  Frege-Ramsey) respectively.

So: resonant and a few load-bearing slots separate the corpus well (L2 Logical 0.81
mass for Frege; L3 0.83 for Freud; L5 Para Vāk for Whorf; L4' Scientific for Kahneman;
L3' Chronological 0.66 for Burke), but within the social-political block the frames
converge on L1 Causal and the night row is near-uniform for same-genre pairs.

## e) Correspondence with known content (one line per entry)

- **C1 Bentham**: hit — L1-4 Final Cause (pleasure/pain as the declared end of action),
  L5-5 Mātṛkā 0.85 ("measuring power") for "value is a measurable sum", L3-5
  Satisfaction for the hedonic sum; resonant Causal + Scientific (measurement).
- **C2 Smith**: partial hit — the unintended-coordination move is caught by L3-4
  Community Integration (prehensions forming a society, no design) and the becoming
  square's split L1@P1 Formal / L1@P4 Final (self-interest as means, public benefit as
  end); but the headline L1-3 Formal Cause is generic, and knowing's L2-3 BOTH (private
  vice / public virtue) only appears in the square, not the main full_text pick (IS).
- **C3 Nozick**: hit — L1-0 Svātantrya (essential-freedom causation) for
  entitlement-from-free-choice, L4.5 Gelassenheit 0.71 (letting-be — no imposed
  pattern), knowing L2-2 IS-NOT (the pattern negated); the strongest single entry for
  "liberty upsets patterns".
- **C4 Burke**: hit — resonant L3' Chronological 0.66 for the partnership across
  generations, L4.3 Zeit, L0.4 Where/When/Why-for (contextual, why-for = holding in
  trust), L0-2' Three (dead/living/unborn as a triad).
- **C5 Locke**: hit — L1-4 Final Cause for the "great and chief end … preservation of
  property" (a literally final-cause argument), resonant L1 at 0.89 (the corpus's
  highest Causal mass).
- **C6 Marx**: hit — L1-2 Efficient Cause (the forces/relations contradiction as motor
  of change), L2-1' Earth (the corpus's only Earth pick — the material base), L4.1
  Geworfenheit (thrownness into material conditions), L3-4 Community (social
  existence determines consciousness).
- **C7 Hardin**: soft hit — L1-3 Formal Cause 0.64 fits (the commons as a payoff
  structure that aggregates individual rationality into ruin) and L3-4 Community for
  the aggregation, but the reading stays structural; "mutual coercion, mutually agreed
  upon" — the remedy half of the core move — never surfaces as a strong slot, and L0
  ended in a 0.25/0.25 tie.
- **C8 Kahneman**: hit — L1-3 Formal Cause 0.95 (two-system architecture), L0-1' Two,
  the corpus's only L4'-Scientific-topped resonant (0.46) for a cognitive-science
  claim, knowing L3-1 Actual Occasion for discrete fast judgments.
- **C9 Whorf**: hit — resonant L5 Para Vāk 0.48 (the speech lens, unique in the corpus,
  for the language entry), L5-3 Madhyamā (mental speech — intermediate structuring),
  L3-2 Ingression (linguistic categories entering perception), knowing L2-0
  Tetralemmaic Ground for the "kaleidoscopic flux" before linguistic dissection.
- **C10 Frege**: hit — resonant L2 Logical 0.81 (the corpus's sharpest lens
  concentration, for the logic paper), L2-3 BOTH (informative identity: same body, two
  senses — is and is-not held together), L3-3 Eternal Objects (Sinne as pure potentials
  of a third realm), L0-1' Two.
- **C11 Freud**: hit — resonant L3 Processual 0.83 (the corpus's strongest single
  mass), L3-0 Concrescent Desire (desire/lack as the motor), L1-0' Introversion 0.97
  (toward-unconscious-pole — dead-on), L5-1' Apokalypsis (the hidden unveiled in
  symptoms and dreams), L3-4' Winter (dormancy) 0.76.
- **C12 Ramsey**: hit — L2-2 IS-NOT (uncertainty as its subject matter), L4.4 Besorge
  (pragmatic concern — belief measured by dispositions to act), L1-3 Formal (axioms
  from coherence), resonant L2 0.62.

Score: 10 clean hits, C2 partial, C7 soft; no outright misses at the entry level. The
lens-level misses are different: see (f).

## f) Anomalies

1. **Night-lens label collapse (the main finding of the name-only arm)**: without
   meanings, L5' → Epi-Logos 12/12, L3' → Spirit (Geist) 10/12, L2' → Aether 5/12.
   The empty grand label wins; meanings are load-bearing, not decoration.
2. **Default-slot gravity inside several lenses (machinery form)**: L2-1 IS x9/12
   (the Logical lens barely differentiates — only Frege BOTH, Ramsey IS-NOT move off
   it), L5-3' Sophia x9/12, L4.5' Insight x8/12, L1-3' Thinking x8/12, L1-3 Formal
   Cause x7/12. For these lenses a "strongest" answer often carries little
   information; the genuinely discriminating lenses are L0, L3, L4, L5 (day) and L2',
   L3' (night).
3. **Resonant lumpiness**: L1 Causal tops 6 of 12 entries — all the social/political/
   economic ones. At frame level the corpus splits into "Causal social argument" vs
   everything else, and only the masses (0.89 Locke, 0.36 Bentham) distinguish within
   the block.
4. **Near-ties / degenerate cells**: C7 L0 ended 0.25/0.25 (Why vs Why-so/Why-not,
   choice picked arbitrarily); C12 L5 0.43/0.41; C1 L3 0.34/0.32. Mean top-probability
   share 0.635 across all 144 machinery cells; no zero-mass or flat distributions; no
   identical distributions repeated anywhere.
5. **Sharpness is bimodal**: several cells are near-determined (0.93-0.97: C5 L1',
   C11 L1' Introversion, C11 L5 Madhyamā, C8 L1 Formal, C2 L5' Sophia 0.91), while
   others are coin-flips — treat any strongest-slot below ~0.5 mass as provisional.
6. **API**: none — 84/84 clean, 0.69-1.64s per call, no retries needed.

## Files

- Raw responses: `c01..c12-{full_text-1,full_text-2,nameonly-1,resonant-1,being-1,becoming-1,knowing-1}.json`
- Instruments: `jev-reflect-nameonly.mjs` + `mef-giving.json` (name-only arm; differs
  from the original only in the full_text branch), original at
  `/Users/admin/.cache/actuation/jev-reflect.mjs`
- `corpus-meta.json` (parsed subjects), `run-log.json` (per-call record),
  `analyze.mjs` (computed report generator), `run-sweep.mjs` (driver)
