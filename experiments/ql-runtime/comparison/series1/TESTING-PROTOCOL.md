# Series 1 testing protocol — one tree, one recipe (2026-09-18)

Owner's rule after the disk filled twice in one day: every agent building a
worktree per small task filled the volume with duplicate build trees. All
experiment work runs through **one worktree and one recipe**.

## The tree

```text
~/.cache/actuation/acceptance      THE experiment worktree (Actuation repo)
```

- Corpus runs execute the release binaries built in this tree; nothing else
  builds.
- Thread work (classification harnesses, corpus extraction, analysis) works
  in this same tree on its own branch, **node/Python level only** — no cargo
  builds, no branch switching while runs are live, no touching another
  thread's paths.
- Run outputs land beside the tree, not inside it:
  `~/.cache/actuation/roundN-<arm>-experiments/`.
- A retired worktree's branch keeps its commits; tip SHAs are recorded in
  the project register before removal.

## The recipe

```bash
cd ~/.cache/actuation
TASKS="S1-CODE-001,S1-EPISTEMIC-001,S1-RESEARCH-001,S1-RESTRAINT-001,S1-SKILL-001" \
CONDS=ql-toolset \
BASE=/Users/admin/.cache/actuation/round9-toolset-experiments \
node acceptance-run.mjs
```

Environment selects the arm: `CONDS` (classic | ql-direct | ql-deep |
ql-toolset), `QL_CLOSE_CHECK` (self | model | jev), `QL_SYSTEM_FILE` (a
system-prompt file appended to the envelope — the implicit-vs-explicit A/B),
`QL_VAK_CONTROL` + `QL_VAK_BIN` (kernel control arm). The runner skips
non-empty task dirs, so an arm rerun needs a fresh `BASE`.

Arms land in the repo under
`experiments/ql-runtime/comparison/series1/runs/<date-glm-...>/` with a
plain READING.md once a full corpus completes.
