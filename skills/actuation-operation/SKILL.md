---
name: actuation-operation
description: Operate Actuation's world-bound Agent/Agency, determination, metagency and Return contracts without confusing procedural competence with authority.
---

# Actuation operation

Use this Skill when an authorised actor needs to inspect or request Actuation operations for an Agent or Agency in a world-bound scope.

## Contract metadata

- Semantic ref: `actuation:operator`
- Native owner: `EpiLogos/Actuation`
- Authoritative source: this `SKILL.md`; projections are copies, never source
- Contract version: `actuation.agency/v1`
- Public contract: `crates/actuation-core/src/agency.rs` (native `actuation.agency/v1` types) and `schemas/actuation.v0.schema.json`
- Verification: `cargo test --workspace` (runs every native suite) and `actuation verify --json` (the served binary certifies itself), plus `bash scripts/verify-native-skills.sh`
- Risk class: authority-sensitive; inspection is not mutation

## Invariants

Keep these distinctions explicit:

```text
Agent != Agency
Agency != RootAgency subtype
WorldBinding != authority grant
Skill available != Capability granted
Capability available != Action authorised
valid contract object != authorised actuation
Return received != Return recognised
Return recognised != automatic world mutation
```

`RootAgency` is a positional relation: a `WorldBinding` is root only when `isRootAgency(binding, rootScope)` proves that its scope and enclosing world coincide. Never infer root position from a SkillSet, profile, model, harness, name or UI location.

## Inputs

Obtain references to the relevant Agent, Agency, world, scope and purpose; the current `WorldBinding`; any `RootScope`; applicable `MetagencyGrant` records; determination lineage; and Return records. Treat bounds, authority refs, human-authored constraints, security-policy refs, evidence refs and external-reality refs as part of the operating world, not decoration.

## Procedure

1. **Inspect the binding.** Admit it through the native `WorldBinding` type in `crates/actuation-core/src/agency.rs` (or read it through `actuation agency`). Read `agent_ref`, `agency_ref`, `world_ref`, `scope_ref`, bounds, authority refs and constraints. Do not rewrite the Agent as the Agency: Agency is the situated determination of an Agent for an act.
2. **Establish root position only from reality.** If a `RootScope` is available, admit it and use `WorldBinding::is_root_for`. A false result means non-root for that scope. Absence of a `RootScope` means root is not established.
3. **Explain metagency rather than assuming it.** Admit each `MetagencyGrant` through the native type. The only portable metagency operations in v1 are `determine-agency`, `configure-agency`, `actualise-agency`, and `reintegrate-return`. Report only operations backed by grants for this Agency and binding.
4. **Form a determination through the public contract.** Admit a `Determination` for `self-differentiation`, `delegation`, `derivation` or `federation`. Preserve non-empty bounds and the declared return policy. Federation does not silently carry determining authority; use explicit delegation when authority is actually granted.
5. **Check recursive lineage.** Before an Agency determines another Agency, admit the `DeterminationLineage`. Downward determination is valid only when the parent determination permits determination within bounds.
6. **Request actualisation through the authority-bearing application surface.** A validated `Determination` describes a legal relation; it does not itself execute or authorise native Actions. Submit the complete request through `actuation agency actualise [file|-] [--json]`, which admits it via `ActualisationRequest::admit` in `crates/actuation-runtime/src/actualisation.rs`. The operation requires the exact governing and differentiated `WorldBinding` records, an explicit matching `MetagencyGrant`, determination lineage, bounded Agent-identity evidence and Return relation. This Skill targets that operation but cannot manufacture authority: source visibility, available Skills or Capabilities, communication, AgentSet membership and Factory Journey participation do not replace the grant.
7. **Receive attributable difference.** Admit each `Return` through the native type. Preserve determination, agency lineage, difference, artifact, Claim, Evidence and material/external provenance refs.
8. **Recognise before mutation.** A Return may be pending, recognised or rejected. World mutation may be `applied` only for an explicitly recognised Return. Do not convert repeated success, benchmark fitness or agent preference into recognition.
9. **Use the read model for explanation.** `agency_reading` (served as `actuation agency`) is the stable inspection view for root position, metagency grants, determination relations, Returns and constraints. Present missing or withheld authority as missing; never invent health or permission.

## Outputs

Produce or return validated contract objects and an explanation containing: operative Agency/world/scope, root status if established, applicable metagency operations, active bounds, determination/return lineage, unresolved authority checks, and verification evidence. An actualisation receipt establishes the semantic relation only; its `effects` truthfully report that materialisation, Factory recognition and source mutation were not performed.

## Verification

Run the repository-owned contract tests. A representative acceptance must demonstrate at least one fail-closed case (for example federation carrying authority, an unauthorised recursive determination, or world mutation before Return recognition) as well as a valid read model.

## Example decision rule

If an Agent has this Skill and can read a `WorldBinding`, but no `MetagencyGrant` authorises `determine-agency`, the correct result is: **the Agent understands how determination works but may not perform it**.
