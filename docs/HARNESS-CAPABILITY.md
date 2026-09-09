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
  install_seam          config_path, format, entry_shape, ownership_marker
  uninstall_seam        same shape; must preserve foreign entries
  model_dispatch        optional; kind (native-provider-binding | none)
                        providers[] when binding: provider_ref,
                        selector (config-key | cli-flag | env-var + name),
                        credential (required + hint when required)
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

## Catalog alignment

Capability descriptors live in `detection/capabilities/<slug>.mjs`, aligned
slug-for-slug with `detection/harnesses/<slug>.mjs`, and are validated against
the detection catalog by `detection/capabilities.test.mjs`. Declared today:
`claude-code` (8 events, deny-and-block, additional-context channel),
`codex` (4 events, no injection channel, no wake), `zcode` (seven native
events — no PreCompact, no Notification — deny-and-block, additional-context
channel, config-json seam with the plugin seam documented alongside).
Catalog r6 adds optional `model_dispatch` on those three: `claude-code`
binds `provider:anthropic`, `codex` binds `provider:openai`, and `zcode`
declares `kind: none` because no provider binding is evidenced in this
repo. The block names how a model is selected on the harness's own surface;
it does not enumerate a provider's model catalogue.

Read models: `actuation harness capability` (catalog) and
`actuation harness capability <slug>` (one descriptor, human or `--json`).

## Correction discipline

Descriptors are corrected against the real product surface, never defended:
zcode rev 1 claimed claude's eight-event grammar; zcode's own shipped
diagnosing-hooks documentation says exactly seven events (no PreCompact, no
Notification) — rev 2 corrects the descriptor and records the correction in
its provenance. Consumers must read the event set from the descriptor, which
is the entire point of owning it in one place.
