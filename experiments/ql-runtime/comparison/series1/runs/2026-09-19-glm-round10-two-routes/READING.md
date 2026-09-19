# Series 1 round 10 — the two routes (2026-09-19)

> **The 4/6 distinction is behavioral, not just theory. Route one (the
> explicate four, tagged) runs at better-than-classic economy with the office
> ledger free — but with no return condition, its one failure ended with a
> good synthesis delivered as chat and the required artifact never made.
> Route two's forced close is what carries determination. And QL in the
> surrounding material alone changed nothing measurable: the lawful order on
> this corpus is demanded by the tasks themselves.**

```text
benchmark_revision   series1-round5-accept-2026-09-17 (arm E, same frozen corpus)
                     series1-round5-accept-office-overlay-2026-09-19 (arm F, named variant:
                     task material overlaid with office-language text, tools untouched)
host                 rust-native-acceptance, release binaries at 3fa0108 (single experiment
                     tree per TESTING-PROTOCOL.md)
arms                 E ql-tagged (route one: four tagged tools, classic-shaped ending)
                     F classic + office-language material overlay (the propagation probe)
design               INTENT-2026-09-19-two-routes.md (owner's two-route position)
```

## The matrix

| arm | result | calls | tokens | wall |
|-----|--------|------:|-------:|-----:|
| E tagged four (route one) | 4/5 checks + SKILL completed-but-artifact-missing | 20 | 25,278 | 511 s (+100 s variance rerun) |
| F classic + office material | 5/5 | 19 | 24,111 | 321 s |
| classic control (round 5) | 5/5 | 21 | 26,579 | 356 s |
| A toolset 4+2 (round 9) | 5/5 | 23 | 37,809 | 411 s |

Baselines as before: vak 38/192.1k · compressed 48/228.9k · ql-direct
73/392.9k · ql-deep 107/715.3k.

## Answers

1. **Route one is the cheapest arm ever recorded — and it shows why route two
   exists.** The tagged four beat classic's economy (20 vs 21 calls, 25.3k vs
   26.6k tokens) while the office ledger records the trail for free (CODE:
   `frame → material×5 → effect → evaluation`). But route one advertises no
   return condition, and its one failure is exactly the predicted one: on
   SKILL the model read every note, then delivered a well-formed, correctly
   sourced handoff **as bare chat** — `deliverable.md` never existed, and the
   objective gate failed the trial. In arm A the forced `close` materialised
   the artifact and passed. The explicate four alone do not carry
   determination; the loop verbs do.
2. **Propagation: material language alone changed nothing measurable.** Arm F
   ran the same five tasks with office-language text woven into the task
   material (SKILL.md reworded, header comments, a "how to work these"
   note per evidence task; the exact overlay rides this run as
   `overlay-office-language.json`) and no law in the tools. Call sequences
   are near-identical to the classic control task by task (SKILL: survey →
   read skill → read request → read notes → write deliverable in all
   three arms); economy sits in the same band, marginally below classic. The
   honest reading: on this corpus the lawful order (material before effect,
   evaluation before finishing) is demanded by the tasks themselves — QL in
   and amongst other things adds no behavior beyond what the work already
   requires. One candidate signal, honestly n=1: F's SKILL materialised the
   deliverable where route one's did not; the office-voiced SKILL.md names
   the deliverable as the effect to make. Repetition would settle it; this
   run does not.
3. **Jev refinement (classification thread, branch
   `agent/jev-mef-lenses-2026-09-18`, commit 35c9276):** the settling rule —
   a return lands where it settles, not as the kind of act that produced it —
   took toolset-row agreement from 11/30 to **30/30** against the by-tagged
   landings, held write_file at P2 2/2, and lifted classic rows 46/75 →
   63/75. Honest caveats: run_tests returns are unsampled (the P4 boundary of
   the rule is unverified); landing-P5 classic rows now all read P1 (two
   previously-correct P5 answers lost); 14 duplicate rows make the 84%
   directional, not a stable point estimate.
4. **Situate, strengthened:** zero situate calls in 22/22 round-9 trials (all
   four arms, plus variance reruns). The return reading happens through the
   tool offices themselves; whether situate keeps a place in the 4+2 form is
   an owner question, recorded not settled.

## Variance and environment, disclosed

- Arm E SKILL first attempt died on `model specimen returned invalid JSON`
  (the same class as round-9 arms B/C); preserved as
  `round10-tagged-S1-SKILL-001-variance-fail-invalid-json.json` and retried
  clean at the loop level — the retry then failed the objective gate for the
  behavioral reason above, which is a real result, not a variance.
- **The volume hit ENOSPC again mid-session** (third time this week).
  ~1.5 GiB of clearly-regenerable caches were cleared on the spot (cargo
  incremental, puppeteer, uv, this checkout's debug artifacts) to finish the
  runs; 5.0 GiB free at close. The actuation world itself is small (~720 MB
  total). The large consumers are outside this line's paths — named for the
  owner: `~/Library/Application Support/Claude` ≈ 15 GiB, `~/.cache/huggingface`
  ≈ 2.7 GiB, `~/.cache/codex-runtimes` ≈ 1.6 GiB — and something external
  consumed several GiB during this session without any local run asking for
  it.

Determination remains `pending-human-review` throughout; the review pass now
covers seven matched conditions across rounds 5–10 plus the two routes.
