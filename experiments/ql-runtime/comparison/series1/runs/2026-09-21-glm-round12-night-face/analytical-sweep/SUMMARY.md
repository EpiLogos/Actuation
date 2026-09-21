# MEF/jev analytical sweep — summary (2026-09-21)

Corpus: `experiments/ql-runtime/comparison/series1/corpus/mef-general-corpus.md` (C1-C12).
Chain per entry: `jev-reflect.mjs` mode `full_text` (jev-latest via Typesafe) → `reflect-analyse.mjs`
(glm-5.3-flash via ZAI), the jev reading object passed as silent scaffolding.
24 calls total (12 + 12), 200 ms apart, one retry on transport failure.
Runner: `/tmp/analytical-sweep-run.mjs` (C1-C10, interrupted externally after C10), resumed with
`/tmp/analytical-sweep-resume.mjs` (C11 reusing its saved reading; C12 full chain).

## (a) One-line verdict per analysis

- **C1 Bentham** — Stands alone; a close rhetorical reading of the two sentences ("a coronation"), catches that aggregation is latent not performed, the diarchy-under-one-governance problem and the missing accountant; slightly more aphoristic in its own voice than the rest.
- **C2 Smith** — Outstanding; detects the two-book splice and the consequential ellipses, proves decentralization from the cast list, and shows the hand figure undoing the demonstration.
- **C3 Nozick** — Outstanding; "whatever" scope, genealogy replacing shape, the rectification parenthesis as Trojan horse, the liberty equivocation, and the hidden pattern in provenance-space.
- **C4 Burke** — Stands alone; consumables vs accumulables recovers the cut inference, the contract-term strain (Paine), and the independent judgment that the corpus gloss ("in trust") domesticates Burke's stranger claim.
- **C5 Locke** — Stands alone; the "therefore" spanning three unstated ladders, the "can" modal, "property" ambiguity, tacit-consent dissolution and the who-judges problem; the longest and most systematic.
- **C6 Marx** — Stands alone; notices the gloss imports base/superstructure vocabulary the excerpt never uses, the mirrored-syntax reversal that stays inside the Hegelian frame, and the effect-as-engine tension between the two sentences.
- **C7 Hardin** — Outstanding; "believes in" as the legitimation insight, res nullius vs governed commons, license/liberty and best-interest equivocations, and the self-refutation (a society that can hold a creed can write a rule).
- **C8 Kahneman** — Stands alone; well-judged for a précis object: "predictable" as load-bearing, the post-rationalization vs signature-failures tension, interface faults vs single addresses, the reification slip; the critique set is the most standard of the twelve.
- **C9 Whorf** — Stands alone; the cut negative claim, kaleidoscope quietly conceding structure to the flux, the self-application problem, comparison circularity, bilinguals, and the amputated "largely".
- **C10 Frege** — Outstanding; the concessive level-jump, "body" as load-bearing word, the missing cognitive-value premise, and four precise critiques of the corpus's own caption (carries-more vs carry-differently).
- **C11 Freud** — Outstanding; the aptness-of-error abduction, "actively" as the entire novelty, the censor regress, immunization, repetition-without-meaning, and the consolation reading of the no-randomness claim.
- **C12 Ramsey** — Stands alone; the Dutch-book derivation stated correctly and compactly, "across" and "will accept" as load-bearing, finite-vs-countable overshoot, statics without updating, and frequency returning via representation.

Every analysis is grounded in the work's own sentences, quotes where it matters, and engages the
entry's known core move (usually by locating it in, or measuring the excerpt against, the fuller
argument). The map's influence shows only through what each essay attends to — weight given to
process over outcome, to how a claim is performed rather than only what it asserts — never through
map vocabulary.

## (b) Relation violations

**0 of 12.** No analysis mentions lens, slot, the map, the reading, or probabilities in the
map-referential sense. String-scan hits, all adjudicated benign:
C2 "Its own scaffolding" (the work's own presuppositions); C7 "on the map" (Hardin's
destination metaphor); C9/C10 "register" (linguistic/stylistic register); C10 "the reading the
modal argument put under strain" (a reading of Frege's caption, not the jev reading);
C11 "the sentence has no slot for them" (Freud's sentence, not the map); C12 "probability/
probabilities" (the work's subject matter, 14 occurrences, all about Ramsey).

## (c) Strongest and weakest

**Strongest: C2 (Smith).** It is the only analysis that reads the corpus entry as an artefact —
identifying the Book I/Book IV splice and showing each ellipsis deleting something the argument
needs — and then turns that into the decisive critique: "The absence of a center from the scene of
the proof is the proof of decentralization," followed by "The argument against design is delivered
in the idiom of design." It also recovers the cut hedge ("frequently") and the excised beggar,
so it both confirms and exceeds the known core move.

Runners-up: C3 (Nozick) for the rectification-parenthesis and liberty-equivocation analysis; C7
(Hardin) for the verdict that the two sentences "enact the tragedy they warn against."

**Weakest: C1 (Bentham)** — relative only; it is still a good essay. Its weakness is register:
it substitutes aphorism for argument more often than its siblings ("They are not an argument.
They are a coronation"), and its miss-list (the hidden third sovereign, the diarchy, the is/ought
collapse) is the most predictable of the twelve against the standard critical literature. Its
genuine finds — "no accountant" for the summing, "as well as" as the load-bearing conjunction,
the "two" as the smallest number that makes weighing possible — keep it well above failure.
Honorable mention of a thin object: C8 analyses a précis rather than a primary text, so its
critique set (clustering, reification, missing fiction-disclaimer) is the closest to the standard
dual-process literature, though its interface-faults point is its own.

## (d) Transport failures

- C2 analyse, attempt 1: runner-level timeout (360 s, generation never completed) — retry succeeded (327 s).
- C5 analyse, attempt 1: same — retry succeeded (273 s).
- Run interruption (not a model failure): the background runner was stopped externally after C10,
  killing the in-flight C11 analyse; the resume run reused C11's saved reading and both C11 and C12
  completed on first attempts.
- No jev call failed (12/12 first-attempt, all under 1 s). Final tally: 24/24 calls produced output;
  0 entries lost.

Analyse latencies ranged 202-347 s; model output 7.3-12.8 kB per essay.
