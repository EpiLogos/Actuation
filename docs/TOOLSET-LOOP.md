# The toolset loop (`ql-toolset`) — design

Owner's correction, 2026-09-18: the QL loop has been framed as a *task for
the model* — control turns in whittled English asking it to reason about
positions it cannot see. The correction: make the QL form **a paradigm for
the praxis itself**. The agent's toolset carries the law; nothing needs
explaining to the model; the smallest change to the agent setup is the
toolset it already uses.

## The founding set — six tools

The four world capabilities keep their names and gain descriptions that name
their office in the Px/Px′ format; the loop's own two verbs become tools.

```text
list_files   P1  take in the field; the return reads as discovery (P1')
read_file    P1  receive material and evidence; the return reads as the given (P1')
write_file   P2  transform the ground; the return reads as the effect made (P2')
run_tests    P4  whole-relative evaluation of the built form (P4')
situate      the return reading: where the work now stands (P0..P5, prime allowed)
close        the return condition: the bounded request is realised (P5; checks run)
```

The descriptions are the only law the model ever sees (source of truth:
`toolset::TOOLSET_TOOLS`). There is no relational system prompt, no control
turns, no whittled English. A with/without-prompt A/B remains possible
without touching this condition because there is nothing QL-flavoured in the
system envelope to begin with — the A/B belongs to the other arms.

## Engine

`RunMode::Toolset` runs a classic-shaped tool loop (the model returns
`{content, capabilityCalls}`) and records the QL circuit from what the tools
do:

- **Office ledger.** Each world tool settles its return at the office the
  static law names (`tool_office`): read/list → P1 material, write → P2
  effect, tests → P4 evaluation. Each settlement is a residue plus a
  transition (`{"from", "to", "by"}`).
- **`situate` is the model's return reading.** It moves the active position
  and records a `return-reading` residue with the claimed P-value (prime
  form accepted). A malformed claim is returned to the model as a tool
  error, not a crash.
- **`close` is the return condition.** Bare content cannot end the work —
  the loop repair-nudges twice and then fails honestly ("return condition
  never ran"). `close` carries the synthesis, runs the task's objective
  checks against a fresh workspace snapshot, and closes the circuit only if
  they pass; otherwise the trial fails with the refusal on the record.
- **Close-check options** (`QL_CLOSE_CHECK`): `self` (default — objective
  gate only), `model` (one separate control-style turn judges the synthesis
  against the success conditions first), `jev` — a named seam refusing
  until the jev+vak thread wires its classifier in.

The record rides the existing `ql-series1-run/0.3` machinery: same held
constants, same workspace verification, same `records[]` shape, with the
circuit, residues, transitions and closure riding `report` — so the corpus
readings and the human-review pass work across arms unchanged.

## Comparison discipline

`CONDITIONS` admits `ql-toolset`; a comparison run still names 1..3
conditions explicitly, and the default set without an explicit request stays
the original trio. The matched-set completeness check now measures each
repetition against the run's *named* conditions, not the global list.

## Relationship to the other arms

```text
classic      no QL structure, cheapest            (21 calls / 26.6k tokens)
ql-direct    model control in whittled English    (73 / 392.9k)
compressed   interpret ruled in code              (48 / 228.9k)
vak          kernel names offices, model authors  (38 / 192.1k)
ql-deep      direct + bounded conjugate return    (107 / 715.3k)
ql-toolset   the law IS the toolset; no control turns at all   (this design)
```

The toolset arm and the vak arm probe the same question from opposite ends:
vak keeps a controller and removes the prose; toolset removes the controller
and puts the law in the model's hands directly. Whether either buys anything
beyond economy is what the pending human-review pass judges.

## Known reconciliation note

The jev+vak classification thread measured that `source_position` (where an
act was issued *from*) and return classification (where a return *lands*)
disagree even between the corpus's own incumbent interpreter and the code
table — the toolset arm sidesteps the ambiguity structurally: a tool's
office is where its return settles, by definition in the ledger.
