# Typing Corpus Review Clarification — 2026-08-14

Status: governing clarification for the Deep QL development track and Experiment Readiness work.

This clarification supersedes the interpretation of `ql-agent-spec` §42 that made human adjudication of all 100 semantic-typing fixtures a development or Series 1 readiness blocker.

## Intent

The 100-act corpus is a shared, deterministic, inspectable semantic benchmark. Its purpose is to exercise positional typing, relation typing, ambiguity, reopening, carrier/function independence, replay, rendering, and cross-implementation comparison.

It is **not** a requirement that the product owner manually label or approve 100 synthetic acts before engineering may continue.

Human review is concentrated at the product level: the working runtime, operator behaviour, traces, renderers, comparison surfaces, and representative semantic cases are reviewed as a product. Individual corpus cases remain available for inspection, correction, or later adjudication when that is useful.

## Required evidence

Experiment Readiness requires that:

1. the shared corpus contains at least 100 deterministic, meaningful cases and satisfies the category/coverage minima in §42;
2. every case has stable identity, structural facts, an explicit benchmark reference, provenance, and ambiguity notes where applicable;
3. the same corpus is replayable by all three host profiles;
4. claimed and retrospective typings remain separately inspectable from the benchmark reference;
5. component agreement metrics remain available without collapsing them into one scalar score;
6. disagreement is evidence, not an automatic release blocker;
7. invalid or unsupported semantic claims fail explicitly;
8. the product-level renderer/comparator makes representative cases and disagreements human-inspectable.

## Human review

Human annotation MAY be added to any corpus case as an additional witness. It MUST NOT be fabricated or inferred from generated fixture labels.

A missing human witness is `not_reviewed`, not a conformance failure.

The product owner may review the corpus selectively, but Experiment Readiness does not require 100/100 manual adjudication.

## Relation to §42 and §43

For this development track, read `human reference type` in §42 as `benchmark reference type` unless an actual human witness is present.

Read the optional `claimed ↔ human` / `retrospective ↔ human` metrics in §43 as conditional metrics: report them only where real human witnesses exist. Always report claimed/retrospective agreement against the shared benchmark reference and implementation-to-implementation agreement where available.

This amendment does not weaken the Direct Core semantics, Deep operator requirements, or anti-collapse rules. It only removes a manual annotation gate that does not itself demonstrate product correctness.
