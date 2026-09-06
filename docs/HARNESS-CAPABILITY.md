# Harness capability descriptors

`actuation.harness-capability/v1` declares **what a harness is** for the
products that dispatch into it: the native lifecycle events it provides, the
channel additional context travels through, what a hook can block, whether the
harness can be woken from outside, and the install/uninstall seams a dispatch
projection must use.

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
  provenance            authored_by + source_refs
```

## Laws the contract carries

- **Ownership.** Actuation owns harness facts. A capability descriptor is the
  only channel through which those facts reach consumers; nothing about a
  declared harness is reassigned or re-discovered downstream.
- **Honesty.** Absence is declared as absence. codex, for example, declares
  `injection_channel: none` and `wake_capability: none` on the surface observed
  in 2026-09: hooks and notify invoke programs, they do not inject context, and
  nothing wakes the harness. Consumers must treat declared absence as truth —
  never print to stdout and hope, never pretend a wake.
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
`codex` (4 events, no injection channel, no wake), `zcode` (8 events,
deny-and-block, additional-context channel, plugin seam).

Read models: `actuation harness capability` (catalog) and
`actuation harness capability <slug>` (one descriptor, human or `--json`).
