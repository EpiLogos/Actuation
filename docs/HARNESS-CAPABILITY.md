---
Register: episteme
Standing: architecture-contract
---

# Harness capability descriptors

`actuation.harness-capability/v1` declares **what a harness is** for the
products that dispatch into it: the native lifecycle events it provides, the
channel additional context travels through, what a hook can block, whether the
harness can be woken from outside, the install/uninstall seams a dispatch
projection must use, and (optionally) which model providers it natively
dispatches to.

It exists because detection and dispatch are different knowledge and must not
be rediscovered per product. Detection (`harness-detection-v1`) proves *that a
harness exists here*, with same-run receipts. Capability declares *what that
harness accepts* — one authority, versioned, owned by Actuation. Consumers
(AIKit installers, doctor, relay) read the descriptor; none of them re-derives
harness facts, and none of them may special-case a harness the descriptor
already declares.

## The shape

```text
capability
  harness_slug          aligned with the detection catalog slug
  native_events[]       event | native_name | transport | can_block | context_channel
  injection_channel     kind + mechanism — how additional context actually travels
  blocking_semantics    deny-and-block | advisory-only | none
  wake_capability       immediate-wake | next-event | none
  install_seam          config_path, format (json | jsonc | toml |
                        skill-tree), entry_shape, ownership_marker
  uninstall_seam        same shape; must preserve foreign entries
  model_dispatch        optional; kind (native-provider-binding | none)
                        providers[] when binding: provider_ref,
                        selector (config-key | cli-flag | env-var + name),
                        credential (required + hint when required)
  provenance            authored_by + source_refs

capability_gap         declared absence, catalog sibling of capabilities[]
  harness_slug          aligned with the detection catalog slug; must not
                        shadow a declared capability
  summary               optional; what the harness is, in one line
  reason                why no descriptor is authored — true, specific, dated
  evidence_refs[]       what the reason rests on (detection receipts,
                        admission records, the coverage diagnosis)
  provenance            authored_by + source_refs
```

## Laws the contract carries

- **Ownership.** Actuation owns harness facts. A capability descriptor is the
  only channel through which those facts reach consumers; nothing about a
  declared harness is reassigned or re-discovered downstream.
- **Honesty.** Absence is declared as absence. codex, for example, declares
  `injection_channel: none` and `wake_capability: none` on the surface observed
  in 2026-09: hooks and notify invoke programs, they do not inject context, and
  nothing wakes the harness. The same law covers `model_dispatch`: a
  descriptor that cannot evidence a provider binding declares `kind: none`
  rather than guessing one. Consumers must treat declared absence as truth —
  never print to stdout and hope, never pretend a wake, never invent a
  provider.
- **Reversibility.** Every seam must declare `preserves_foreign_entries: true`.
  Uninstall removes only entries matching the ownership marker; native config
  the projection does not own is untouched.
- **Wake honesty.** `immediate-wake` requires notes naming the listener that
  makes it true. Relay transports read this field; optimism is not a transport.
- **Coverage closure.** Every detection descriptor in the catalog MUST carry
  either a capability descriptor or a declared capability gap. Detection
  without capability standing is an open question wearing the clothes of a
  settled one: consumers can see the harness but must guess — or silently
  forgo — what dispatch into it accepts. A gap is a first-class declaration,
  not a footnote: a gap with a true reason beats a descriptor with guessed
  facts. Filling a gap means authoring the descriptor against observed
  evidence under the citation discipline below, never by relaxing the
  loader. The catalog loader rejects a descriptor that declares neither —
  the converse of its existing alignment check, naming the slug — and the
  shipped `verify` suite checks closure over all descriptors, not a spot
  sample. The closure law was commissioned by the owner on 2026-09-14,
  executed via coordinator.

## Catalog alignment

Capability descriptors live in `catalog/targets.json` (`schema
actuation.native-catalog/v1`, loaded natively by
`crates/actuation-adapters::NativeCatalog::bundled()`), aligned slug-for-slug
with the harness descriptors in the same document and validated by the
adapters crate's tests and the shipped binary's `verify` suite. Declared
today:
`claude-code` (8 events, deny-and-block, additional-context channel),
`codex` (the 12 hook events codex 0.155.1 ships — the shipped draft-07 output
schemas decide the truth: PostCompact and Interrupt are declared, PreToolUse
permissionDecision deny vetoes the tool call, and Stop, UserPromptSubmit and
SubagentStop carry codex's own decision:block + required-reason channels —
additional-context channel on five events, environment-scoped
`$CODEX_HOME/hooks.json` seam), `zcode` (seven native
events — no PreCompact, no Notification — deny-and-block, additional-context
channel, config-json seam with the plugin seam documented alongside), `pi`
(skill-tree seam).
Catalog r6 adds optional `model_dispatch` on those three: `claude-code`
binds `provider:anthropic`, `codex` binds `provider:openai`, and `zcode`
declares `kind: none` because no provider binding is evidenced in this
repo. The block names how a model is selected on the harness's own surface;
it does not enumerate a provider's model catalogue.

Read models: `actuation harness capability` (catalog) and
`actuation harness capability <slug>` (one descriptor, human or `--json`).

## Public intake

The contract's intake promise has a route. An outside author — any agent or
person who can observe a harness — can author a descriptor for a declared
capability gap, check it against the law, and hand it to the owner for
landing, without touching this repository and without a plugin system:

```text
actuation harness capability validate <file|->
actuation config-contribution capability <file|-> [--json]
```

`validate` checks a descriptor document against the capability schema and the
catalog laws and answers with a named validation document
(`actuation.harness-capability-validation/v1`): `schema-admission` (the closed
event vocabulary, reversible seams, wake honesty, provenance),
`slug-alignment` (the slug names a detection descriptor in the shipped
catalog), and `coverage-closure` (a contribution fills a declared capability
gap). Exit codes carry one meaning each: 0 valid, 1 refused with the named
checks on stdout, 2 for a handler refusal (unreadable or non-JSON input).

`config-contribution capability` is the intake face of the same law: when the
checks pass it mints a receipt (`actuation.capability-contribution/v1`) that
holds the descriptor with its provenance and the contributed bytes' sha256,
and states the landing edit — capabilities += the descriptor, the gap
withdrawn, the catalog revision advanced. Actuation performs no catalog
mutation: the bundled catalog is compiled into the binary, and landing the
contribution remains the owner's edit to `catalog/targets.json` under the
correction discipline, made after the owner has reviewed the declared facts
against their cited sources. The gemini descriptor authored from public
surfaces during the 2026-09-22 adapter-acceptance campaign
(`fixtures/capability-contributions/gemini.harness-capability.json`) is the
worked specimen: its receipt (`capability-contribution:gemini:f1af4031414a`)
was the route's first landing — catalog r15 applies its edit verbatim. The
specimen's provenance travels unchanged (it names the revision its
normalisation was recorded at); re-running the intake on the landed specimen
now answers the coverage-closure refusal, since gemini carries a declared
capability — the same law that refused a shadow before the landing existed.

A contribution fills a gap. Correcting an already-declared capability — the
ordinary maintenance this document records below — is the owner's own edit,
and intake says so rather than accepting a shadow.

Catalog r7 closed the coverage gap the other direction: the nine descriptors
that carried no capability standing received declared capability gaps, the
loader learned the converse of its alignment check, and the shipped verify
suite stopped spot-checking one harness and started looping over all of them.
Catalog r8 filled the first gap from evidence: `pi` is authored from the
AIKit admission (ai-kit#186, closed 2026-09-08, riding the 2026-09-06
detection receipt — sha256 unchanged on the 2026-09-15 re-detect) and live
observation of pi 0.84.4 on this machine: additional context travels as
skill capsules projected into `~/.pi/agent/skills` (global) and `.pi/skills`
(project-relative) and read at session start — the `skill-tree` seam — with
no observed stdout/JSON context channel, no deny-and-block, no wake, and no
single provider binding (pi is provider-plural by design). Eight gaps
remain, each naming its true reason: `gemini` and `gemini-antigravity`,
`grok-bot`, `openclaw` and `kimi` have AIKit admissions that ride detection
records or partial projection evidence with no observed event/blocking
grammar; `hermes` and `hermes-acp` have no AIKit adapter at diagnosis;
`ollama` is detected as a model-provider for model binding, not agency
dispatch. Catalog r13/r14 redeclared `codex` on codex 0.155.1's own embedded
draft-07 hook schemas (events and blocking channels read from the shipped
schemas, never from brand similarity) and on the environment-scoped seam the
dispatch projection actually writes — the same correction discipline this
route serves. Catalog r15 landed the intake route's first contribution: the
gemini capability descriptor received through `config-contribution
capability` (the campaign specimen above) applied its receipt's edit — the
r7-declared gemini gap withdrawn, gemini carrying a real account instead:
eleven declared native events, a stdout-additional-context injection
channel, deny-and-block, no wake. Thirteen gaps remain declared, each naming
its true reason.

## Correction discipline

Descriptors are corrected against the real product surface, never defended:
zcode rev 1 claimed claude's eight-event grammar; zcode's own shipped
diagnosing-hooks documentation says exactly seven events (no PreCompact, no
Notification) — rev 2 corrects the descriptor and records the correction in
its provenance. Consumers must read the event set from the descriptor, which
is the entire point of owning it in one place.
