# R10 — owner-machine physical return (Development Field S7)

Basis: accepted Actuation main `90041d5f81331b636893d772c3288c062ad0e13b`
(PR #73; consumer harmonisation merged as EpiLogos/O-I#245/#252/#254 and
EpiLogos/ai-kit#300 — the last open R8 items closed with O-I#252, whose joined
Development Field witness drives this same served binary). Machine record for
the physical return demanded by issue #58 and fed into EpiLogos/O-I#213
(Development Field S7). Executed on the owner workstation (macOS arm64),
2026-09-12, against the installed coherent artifact — not CI.

## Release

- Tag `actuation-v0.2.0` cut at `90041d5f81331b636893d772c3288c062ad0e13b`
  per `.oi/product.json` release law (github-releases, actuation-v tags,
  sha256 checksums, GitHub artifact attestation).
- Artifacts produced by the prelocal-build workflow (run 34687734592) at that
  exact revision:
  - `actuation-0.2.0-90041d5f8133-macos-arm64.tar.gz`
    sha256 `6de02fd2e656bcdc18f31b6bd2e71af82ef03ae527442358747cd939e714a4ea`
  - `actuation-0.2.0-90041d5f8133-linux-x86_64.tar.gz`
    sha256 `9b71cf2ab70d1e7374e06d26aa9a3ff955d47129c2ee4f3cc3860e69f3b005e1`
- Both checksums re-verified locally after download; both artifact
  attestations verified (`gh attestation verify`, exit 0).
- Clean-directory serve proof (no repository, no Node, stripped
  `env -i PATH=/usr/bin:/bin`): `--version` = `actuation 0.2.0`;
  `verify --json` ok with the full compiled-in suite;
  `source-commit.txt` = the exact source commit.

## Installation/body

- The stale pre-R7 Node launchers on this machine were `npm link` symlinks
  (`~/.npm-global/bin/actuation` → `../lib/node_modules/actuation/bin/actuation`
  → the worktree's removed `bin/`; `~/.npm-global/lib/node_modules/actuation`
  → the worktree itself). Both dangling symlinks removed; no Node actuation
  remains in any install root or on PATH.
- The release artifact is installed at
  `~/.local/share/oi/installs/actuation/actuation-v0.2.0/` with an
  `install-receipt.json` naming the tag, source revision, sha256 and
  attestation standing, and registered with O:I
  (`oi register actuation --executable … --root …`).
- `oi where actuation --json`: executable = the installed artifact,
  expected_revision = `90041d5f81331b636893d772c3288c062ad0e13b`,
  observed binary sha256 `b90e67bc1a68d6e229754e8e8da35ec2c663c0b53611a11514e3606a2a587799`.
- `oi actuation --version` → `actuation 0.2.0`; `oi actuation verify --json`
  → ok, 15/15 deterministic self-verification checks, dispatched through the
  O:I front door with no Node on PATH and the working directory outside any
  repository.
- `~/.config/oi/catalogue.json` re-adopted from the current surfaces
  (post-#245/#254 contract); the O:I cli suite is deterministic on this
  machine again after the re-adopt.

## Direct actuation (real harness, real model turn)

- Real supported harnesses observed on this machine by the installed binary
  (`harness detect --json`, catalog r6): claude-code (executable + config +
  100 skills/9 plugins/8 commands/11 hooks), codex (executable + config),
  zcode (config + env), ollama (executable present; service honestly absent —
  no listener at 11434), gemini/pi/hermes/grok-bot/openclaw per catalog.
- One real bounded model turn executed in a persistent project World
  (`~/.local/share/actuation/s7-world-2026-09-12`, world.md authored) through
  the owner's configured codex harness (`codex exec`, read-only sandbox):
  returned exactly `S7_DIRECT_ACTUATION_OK` (10,240 tokens). Real output
  preserved at `evidence/direct-turn-stdout.txt`
  (sha256 `8b0f0e5e93354e1f0f8754895934299612afc2783edf8fcfff413731b8b150a5`).
- The turn is recorded as a real stream observation
  (`stream:s7:direct-codex:1`, event `event:s7:direct:codex-turn`, content =
  the output digest, metadata turn_result) under the catalog-declared codex
  session-start boundary; the Activity
  (`activity:s7:direct:codex-turn`) validated through the served binary with
  `run_ref`/`journey_ref`/`plan_ref` **absent** — direct agency with zero
  Factory ancestry, physically.
- claude-code was also attempted first and refused honestly: "OAuth session
  expired and could not be refreshed" — recorded as the owner's provider
  condition, not worked around.

## Factory-correlated development

- The identical real turn re-read with supplied Factory correlations
  (`activity:s7:factory-correlated:codex-turn`, plan/journey/run refs
  `plan:s7:development-field` / `journey:s7:development-field` /
  `run:s7:development-field`): identity laws hold on the wire —
  `run_ref != actuation_ref`, `run_ref != agent_session_ref`,
  `actuation_ref != agent_session_ref`; the correlations name no new
  Actuation identity.
- Return recognition boundary physically read back through the served agency
  read: `return:s7:developer` — received true, `recognition_state` pending,
  `world_mutation_state` not-applied.
- The full joined chain (Factory Commission/Journey/Run → AIKit exact Git
  basis → Workcell material Development World → real execution → Activity/
  Return → Factory evidence/candidate/recognition boundary, identity laws
  intact) is proven deterministically by the joined conformance specimen
  (O:I #252, green in CI at this exact cut including Actuation `90041d5`).
  The on-machine pieces executed above are its Actuation-side physical half;
  the Workcell `workcell:local` material-host leg was not separately
  exercised on this machine (Workcell pin unchanged; its hosting laws are
  CI-proven).

## Existing/external harness

- The codex exec run above is an externally started native harness process:
  it was observed (detection receipt with executable sha256, config state)
  and correlated only where actually known; no O:I/Factory caller ancestry
  was fabricated — the direct Activity carries none.

## Relocation/degradation

- Two real material roots (`root-a`, `root-b`) each holding the preserved
  artifact; the served binary genuinely executed inside each
  (`harness detect`, exit 0 per root); realised receipts validated per root
  with per-root material bindings; continuity across the move computed as
  same_agent/same_agency/same_world_binding/same_actuation = true with
  `material_binding_changed` = true — semantic identity survives material
  relocation because a new body is not an input to identity.
- Degradation: `root-b` removed; executing the body there fails honestly
  (exit 1, no fabricated observation); the durable store replays truthfully
  and independently of the material removal.

## Stream/persistence

- Real durable store at `~/.local/share/actuation/streams/s7-2026-09-12/`
  (append-only JSONL, one file per stream): open → record → replay → close
  across separate served processes (actual process-restart persistence);
  raw bytes on disk inspected (header line + event line).
- Torn-tail law proven on a damaged copy: `replay` refuses loudly —
  "torn or invalid event at position 2 … tail is not silently dropped" —
  never silently repaired.
- Note: the earlier claude-code `SessionStart` record in
  `stream:s7:physical-return:1` is a store-mechanics proof (the recording
  path and its refusal laws); it is not a claim that the claude-code harness
  emitted a hook callback at that moment.

## Research readiness

- The QL/Prime and epistemic-cultivation programmes resolve through the
  accepted Rust substrate: `actuation-research` is part of the locked
  workspace (82/82 lib tests green locally at this tree; workspace tests
  green in CI at this main), the frozen specimen-native oracles replay
  through the native gate's R6 line (44 parity cases; full native gate green
  on this machine before every Actuation PR today), and the research
  capabilities disclosure names the remaining acceptance horizon as
  owner-machine only (this record).
- The live-provider QL experiments (deep-runtime/series1-live) were not
  exercised; their standing is unchanged and explicitly not claimed here.

## Evidence standings

```text
D  deterministic: served verify suite 15/15 (installed artifact);
   O:I cli suite deterministic after catalogue re-adopt; frozen corpora
   replayed natively (native gate green per PR: R2 119, R3 44, R4 213,
   R5 179, R6 44 parity + scenarios + 33/33 cli scenarios)
C  cross-product: joined conformance specimen green in CI at this exact cut
   (O:I #252), including the six-product source-built dispatch; ai-kit CAW
   delivery + task-dispatch material phase green on merged main (ai-kit#300)
P  provider: codex real turn in the S7 World (S7_DIRECT_ACTUATION_OK,
   output digest preserved); claude-code turn blocked by expired OAuth
   session (owner condition, honestly refused)
M  owner-machine: release artifact install + registration + dispatch;
   clean-directory serve proof; stream persistence/restart/torn-tail;
   relocation continuity + honest degradation; real harness detection
   receipts (claude-code, codex, zcode, ollama-absent)
H  human: none recorded — the owner has not walked or accepted; S7/TUI/Desktop
   human EX remains with the owner
```

## Not claimed

No human acceptance; no claude-code model turn (provider session expired);
no on-machine Workcell material-host leg beyond the real local roots above;
no whole six-product active suite receipt (the mainline candidate remains a
source composition; `suite active` is null on this ground — advancing it is
the S0 whole-suite horizon, not Actuation's R10 scope).

## Findings for the owner (machine/integration faults, not product defects)

1. The desktop flow tooling creates empty flow placeholder files directly in
   `ProjectCentral/now/flows/` (three today: 10:51, 11:37, 11:45). Two were
   registered byte-exact via PRs #74/#75; while any untracked flow exists,
   the O:I current-main install law (`oi dev install`) correctly refuses the
   dirty worktree. The desktop's flow lifecycle and the developer-source
   install law need one integration decision (e.g. desktop flows staged in
   `.central/` until day-close, or auto-registered into the register on
   creation).
2. The claude-code OAuth session is expired on this machine; every harness
   observation is otherwise healthy. Refreshing it restores the second
   real-harness path.
3. The whole-suite active receipt is unmaterialised on this ground; single
   product installs resolve through composition registration. Advancing the
   coherent six-product suite install is the remaining S0 horizon.
