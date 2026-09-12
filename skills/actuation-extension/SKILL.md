---
name: actuation-extension
description: Extend Actuation's Agent/Agency and actuation contracts through versioned public seams with conformance, provenance and explicit authority boundaries.
---

# Actuation extension

Use this Skill when changing or adding a native Actuation capability, Action, actuation relation, provider-facing adapter or contract version.

## Contract metadata

- Semantic ref: `actuation:extension-developer`
- Native owner: `EpiLogos/Actuation`
- Public foundation: `crates/actuation-core/src/agency.rs`, `schemas/actuation.v0.schema.json`, `docs/ACTUATION-CONSTITUTION.md`, `docs/ACTUATION-RELATION.md`
- Verification: `cargo test --workspace` (runs every native suite) and repository workflows
- Risk class: contract/authority-sensitive

## Extension law

Extend the public ontology; do not edit generated/projected runtime state and call that a new Actuation contract. Keep generic Actuation number-neutral and harness-neutral. `AgenticComposition` must not collapse into AIKit `HarnessComposition`, Factory `ExecutionDisposition`, O:I `SharedField`, or Workcell process/service topology.

## Procedure

1. State the deficiency as a Claim and identify the public contract or adapter seam that owns it.
2. Decide whether the change is additive within `actuation.agency/v1` or requires a new explicit version. Do not silently change the meaning of an accepted field.
3. Preserve the `Agent`/`Agency` distinction, positional Root Agency, explicit bounds and authority refs, determination kinds, delegated-autonomy rules and attributable Return lineage.
4. If adding a metagency operation, define its authority semantics first; add it to schema/runtime validation only when the operation can fail closed without a grant.
5. If adding an adapter, translate into the portable contract at the boundary. Do not make one harness, model, provider or QL profile the generic ontology.
6. Add positive and adversarial contract fixtures/tests. At minimum prove malformed refs fail, authority cannot be smuggled through federation or composition, and Return-driven world mutation still requires explicit recognition.
7. Run `cargo test --workspace` and the repository-native CI workflows (the frozen parity corpora replay the accepted wire meaning against the native oracles). Record the exact source revision as evidence.
8. Submit the proposed source revision for native-owner review. Promotion is explicit; projected copies, successful runs and benchmark wins never promote themselves.

## Representative specimen

`crates/actuation-core/src/agency.rs` and its tests are the executable specimen for extending the portable contract: they construct world bindings, root scopes, grants, determinations and Returns through the native validated constructors, with the frozen parity corpus (`fixtures/migration/oracle.json`) preserving the accepted wire meaning. Follow that pattern for a small extension before wiring any UI or harness-specific presentation.

## Self-improvement route

```text
observed deficiency
  -> Factory Claim / Run
  -> proposed Actuation source revision
  -> contract + adversarial evidence
  -> Actuation-owner review / Recognition
  -> explicit promotion or rejection
```

A Factory Run may propose and test this change. It does not receive arbitrary repository authority from doing so.
