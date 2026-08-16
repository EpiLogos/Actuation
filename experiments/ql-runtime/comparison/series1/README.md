# QL Agent Series 1 — live matched benchmark

This directory contains the first **real capability experiment**, not another semantic-conformance suite and not yet an automated leaderboard.

The question is ordinary:

> Holding model, exact prompt, starting files, capabilities, verification protocol, execution budget and host revision constant, how do Classic, Direct QL and Deep QL differ in the agent's ability to understand, act, recover, stop and fulfil intent?

Read `BENCHMARK-V0.1.md` for the normative benchmark design.

## First determination: human review

Series 1 v0.1 deliberately records:

```text
determination = pending-human-review
```

The first judgement comes from inspection of the prompt, model/tool behaviour, QL trace, final output/artifact and verification evidence. Scalar evals and blinded model judges are deferred until the behaviour of the three loop conditions has first been understood directly.

Every live record retains the complete model input/output, capability call/result trace, starting workspace, final workspace, objective verifier output, usage/cost and runtime semantic events. `render-review.mjs` produces a readable Markdown review bundle from the machine JSON.

## Conditions

```text
classic
ql-direct
ql-deep
```

Primary comparison is within-host. The same benchmark is then repeated through all three real host paths:

- `pi`: pinned `@earendil-works/pi-ai`;
- `pydantic-ai`: pinned Pydantic AI via the Python bridge;
- `native`: repo-owned minimal DeepSeek transport.

## Provider contract

Series 1 currently stipulates:

```text
provider   deepseek
candidate  deepseek-v4-flash
credential DEEPSEEK_API_KEY
```

`DEEPSEEK_API_KEY` is auto-discovered from the local environment. GitHub Actions uses the same name as a repository Actions secret. There is no experiment-specific API-key alias and no fixture fallback.

## Benchmark v0.1

The fixed exploratory set is:

- `S1-CODE-001` — code repair and verification;
- `S1-RESEARCH-001` — research over local files;
- `S1-EPISTEMIC-001` — understanding under incomplete evidence;
- `S1-SKILL-001` — procedural skill use with judgement;
- `S1-AGENCY-001` — stale-state diagnosis and narrow justified action;
- `S1-RESTRAINT-001` — trivial bounded request / anti-ceremony.

Each task has a frozen prompt, frozen starting workspace and frozen objective verification protocol. The verifier records observations; it does not decide which recurrence is better.

## Running locally

With `DEEPSEEK_API_KEY` already exported, run fail-closed preflight and then a benchmark task, for example:

```text
node preflight.mjs --live --host native --task S1-CODE-001
node run.mjs --host native --task S1-CODE-001 --repetitions 1 --max-steps 16 > run.json
node render-review.mjs run.json > review.md
```

Repeat the exact task through Pi and Pydantic AI after their pinned dependencies are installed.

## Running in GitHub Actions

The default branch exposes manual workflow **QL Series 1 Live**. It accepts a host (`all` by default), a task (`all` by default), repetitions (`1` for initial human review), a common max-step budget, and the candidate model (`deepseek-v4-flash`).

A full `all × all × 1` exploratory dispatch runs the six frozen tasks through all three host paths; inside each host/task it runs Classic, Direct QL and Deep QL from fresh identical workspaces. The workflow uploads both machine-readable JSON and human-readable Markdown review bundles.

GitHub cannot inherit credentials from a developer shell, so the repository needs the Actions secret `DEEPSEEK_API_KEY`.

No live run has occurred merely because this harness exists.