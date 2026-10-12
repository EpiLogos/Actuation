---
name: epi-field
description: Gives Prime Agencies operative access to the Anuttara field — embed accounts into the shared 3072-d space, retrieve declared relations, compute accounts through the full mark table (Jev), read virtue presence/balance against the ninefold, score relation placement, and draw drafts toward a virtue balance. Use when an account should be read against the map's declared structure, when a virtue balance matters to tone or judgement, or when energetic evidence should accompany a reading. Read-only on the canon.
---

# Epi field faculty

This is an **operative faculty**: the resonance-and-classification instrument
verified in `Work/Actuation/experiments/model-types/FRONTIER-ARCHITECTURE.md`.
Low resonance means compatibility with declared structure — never value,
never worth. Every reading is instrument output (`semantic-stochastic`,
promotion none); where classification and field disagree, record the
disagreement for review.

Import the Python-backed skill:

```python
import epi_field
```

Available calls:

```python
await epi_field.capabilities()
vec = await epi_field.embed_account("<account text>")
await epi_field.retrieve(vec, k=3)                  # nearest declared relations + marks (verbatim text)
await epi_field.virtue_resonance(vec)               # nine-position profile
await epi_field.relation_placement(a, "<relation text>", b)   # energy; lower is more compatible
await epi_field.syntax_compute("<account>")          # which mark does this compute through (Jev, full mark table)
await epi_field.virtue_presence("<account>")         # presence/balance + field resonance, COUPLED in one record
await epi_field.draw_align(draft, "Wisdom", situation="...")  # the alignment draw: numeric verdict steers the redraft
await epi_field.shadow_receipt({...})                # Return acts carry their reading as evidence; never a refusal
await epi_field.verify_battery()                     # T1 presence + T3 placement, through these very calls
```

Honesty law: the reading is evidence attached to the act, never the
determination; the graph is read-only here (embeddings are additive and
sanctioned elsewhere; this module never writes). Refresh the field store with
`node field_refresh.mjs` when the map moves.
