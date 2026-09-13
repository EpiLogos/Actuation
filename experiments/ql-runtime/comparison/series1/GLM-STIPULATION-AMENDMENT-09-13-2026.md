# Series 1 v0.1 — GLM candidate stipulation amendment

Status: normative Series 1 amendment, 2026-09-13
Amends: `BENCHMARK-V0.1.md` §provider contract

## What changed

The candidate stipulation moves from `deepseek / deepseek-v4-flash / DEEPSEEK_API_KEY` to:

```text
provider   zai
candidate  glm-5.3-flash
credential ZAI_API_KEY
```

The native transport endpoint is `https://api.z.ai/api/coding/paas/v4`. The stipulation
check remains fail-closed: a provider override that does not equal the stipulation is an
error, and there is no fixture fallback.

## Why

The live programme now runs through two harness surfaces — the native LoopRuntime host and
the Prime recursive research harness (`experiments/ql-runtime/prime/`, carrier issue
EpiLogos/Actuation#26). Prime natively serves `zai/glm-5.3-flash`, and the owner directed
the campaign onto the GLM key. One candidate across all surfaces keeps the held-constant
law meaningful for cross-surface comparison; running each surface on a different model
would confound host effects with model effects.

## Standing of prior records

The 2026-09-13 deepseek smoke (`runs/2026-09-13-smoke-restraint/`) remains a retained
exploratory record under determination `pending-human-review`. It is not invalidated by
this amendment; it is simply not comparable with GLM-era runs and must not be pooled with
them.

## Consequences

- The benchmark, task, runner and review revisions all advance because the stipulation is
  part of the frozen profile. Comparison validity is per-revision as before: only runs
  produced from the same frozen profile count as matched sets.
- The DSH maximal-reference amendment (#139/#140, parked) names the DeepSeek provider
  route for its host lane. When that lane is activated it must either restate its provider
  contract against this amendment or run as a deliberately separate-model lane.
- `DEEPSEEK_API_KEY` remains in the redaction scan alongside `ZAI_API_KEY` so historical
  evidence stays safely reproducible.
