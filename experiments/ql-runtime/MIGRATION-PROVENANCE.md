# Migration provenance

This QL Agent Runtime experiment corpus was graduated from `EpiLogos/agent-system-design` into Actuation.

## Source lines

- source Wayfinder: `EpiLogos/agent-system-design#94`
- Deep integration PR: `EpiLogos/agent-system-design#130`
- Deep source branch: `ql/deep-runtime`
- pinned Deep source head: `a654c62f68b82236061986d9215b23257fe53b17`
- Factory `main` observed during migration: `94c72f534cb51410865de0af138feb488b13e999`
- PR/main merge base: `1928107d926083d997d3b1b87ad5f3236bd863a9`
- former path: `ql-agent-experiments/`
- Actuation path: `experiments/ql-runtime/`

The experiment body is the **complete `ql-agent-experiments/` tree from the Deep PR head**, because that line is 119 commits ahead of the common base and contains the corrected Deep formal/runtime work. Factory `main` had diverged by two later commits (`9fa7e85d…`, `94c72f53…`), both confined to the default-branch Series 1 live dispatcher. Their DeepSeek-native, matched-human-review dispatcher changes were reconciled into Actuation's adapted `.github/workflows/ql-series1-live.yml` rather than replacing the newer Deep experiment body with the older `main` tree.

The Actuation DSH workflows additionally carry a compatibility bootstrap for the pinned public DeepSeek Harness source `47f943859bef60e4160492346772ded9b24f765a`: that source declares `pnpm@11.7.0`, whose tarball was unavailable from the public npm registry during migration, while its lockfile is v9. The source SHA remains pinned; Actuation uses pinned public `pnpm@10.15.1` only to bootstrap that lockfile.

## Evidence boundary

At migration:

```text
structural / conformance evidence     present
live Series 1 capability runs         0
capability-effect determination       unclaimed
```

Migration does not upgrade that evidence status. Historical review and commit provenance remain in Factory issue #94 and PR #130. New canonical experiment development proceeds in Actuation after migration acceptance.
