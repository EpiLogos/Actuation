# Semantic typing benchmark

`corpus.js` deterministically materialises the shared 100-act §42 benchmark under the 2026-08-14 typing-corpus clarification.

The cases are meaningful semantic examples rather than numbered dummy acts. Every record has:

- a stable `QLT-###` identity;
- an act intent and carrier;
- a deterministic `benchmark_reference` with provenance;
- separately inspectable claimed and retrospective fixture witnesses;
- structural facts;
- ambiguity/category metadata;
- an optional `human_witness` field.

No generated value is represented as human judgement. `human_witness: null` means simply that nobody has manually adjudicated that case.

The benchmark is executable evidence for coverage, replay, metrics and cross-host comparison. Human review is product-level and may selectively inspect or annotate corpus cases when useful; 100/100 manual annotation is not a readiness gate.
