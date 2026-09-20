# Series 1 round 11 — the lens reading as a tool (2026-09-20)

> **Given the lens-reading tool — a live MEF reading of any subject it names,
> served by jev — the model never called it. Zero calls in five clean trials.
> Availability alone does nothing. The tools that got used in this line were
> the ones the work's structure demanded; an un-demanded cognitive tool is
> idle capacity, not a window.**

```text
benchmark_revision   series1-round5-accept-2026-09-17 (same frozen corpus)
host                 rust-native-acceptance, release binaries at d04dbc2
condition            ql-lens — the founding six tools plus lens_reading; the
                     reading is served by the jev classification instrument
                     (jev-lens-reading.mjs, question wording verbatim from the
                     classification thread's LENS_QUESTION); jev's closure-gate
                     role is retired this round (QL_CLOSE_CHECK is self|model)
```

## The matrix row

| arm | result | calls | tokens | lens calls |
|-----|--------|------:|-------:|-----------:|
| ql-lens (six + lens_reading) | 5/5 | 22 | 41,401 | **0** |

(Reference: arm A toolset, same loop without the lens tool: 23 calls /
37,809 tokens. The lens condition costs nothing and adds nothing yet —
the +3.6k tokens are the longer tool roster in every turn's envelope.)

## What happened

- The model read the tool's description every turn for 22 turns across five
  tasks and never once named a subject. The reading instrument itself works
  (live smoke: a subject in, a full twelve-lens distribution out in ~670 ms;
  the smoke reading landed on the standing L1 + L4′ pair the classification
  thread measures).
- This extends the round-9 situate result from a curiosity to a pattern:
  **capacity without invitation stays idle.** list/read/write/tests get used
  because the tasks demand them; close got used because the loop requires
  it; situate and lens_reading sit untouched because nothing in the work's
  flow ever asks for them.
- The design implication for the cognitive-toolset idea: if a lens reading is
  to happen, something in the work must want it — an office whose procedure
  includes taking a reading (the way close's procedure includes the checks),
  or material that references it. "A tool exists" is not an invitation.

## Variance, disclosed

- The model endpoint had a bad window during the first pass: RESTRAINT hung
  twice at ~213 s and SKILL once ("model specimen failed", empty stderr —
  the adapter was killed mid-hang, no HTTP error ever surfaced). Both failed
  records are preserved (`*-variance-fail-endpoint.json`); solo reruns of
  both tasks completed cleanly, checks passing. A direct reproduction of the
  failing call succeeded in 3 s minutes later.
- One probe mishap that is tooling, not experiment: a stdin-capturing wrapper
  around the body adapter was first written as a shell script and died
  instantly (the adapter is spawned as `node <file>`); rewritten in JS it
  captured normally.

Determination remains `pending-human-review`. The open question this round
hands the owner: what counts as an invitation — does the reading become part
of an office the work flows through, part of the material, or part of close?
