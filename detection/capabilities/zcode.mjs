// Declared harness capability. What zcode IS for dispatch projections.
// zcode exposes a Claude-grammar hook surface (the same event names and
// stdin/stdout JSON grammar) through its plugin and settings system; its
// plugin facet is the detection-declared seam on this machine. The exact
// user-level settings path is deliberately not claimed: on this machine
// only the plugin facet is observable, and the descriptor says so.
export default {
  "schema": "actuation.harness-capability/v1",
  "document": "capability",
  "harness_slug": "zcode",
  "summary": "Claude-grammar hooks (same event names, stdin JSON, stdout JSON outcomes) exposed through the plugin/settings system; plugins facet is the observed seam; deny-and-block on pre-tool boundaries; no external wake.",
  "native_events": [
    {
      "event": "session-start",
      "native_name": "SessionStart",
      "transport": "plugin-or-settings-hooks-map",
      "can_block": false,
      "context_channel": "stdout-additional-context",
      "notes": "session-opening context via stdout additional-context outcome, as in the Claude hook grammar."
    },
    {
      "event": "user-prompt-submit",
      "native_name": "UserPromptSubmit",
      "transport": "plugin-or-settings-hooks-map",
      "can_block": true,
      "context_channel": "stdout-additional-context",
      "notes": "deny-and-block at prompt submission; stdout additional context is added to the turn."
    },
    {
      "event": "pre-tool-use",
      "native_name": "PreToolUse",
      "transport": "plugin-or-settings-hooks-map",
      "can_block": true,
      "context_channel": "stdout-additional-context",
      "notes": "pre-tool additional-context travels the hook outcome channel (not stdout text); exit 2 denies the call."
    },
    {
      "event": "post-tool-use",
      "native_name": "PostToolUse",
      "transport": "plugin-or-settings-hooks-map",
      "can_block": false,
      "context_channel": "stdout-additional-context",
      "notes": "after tool success; advisory only."
    },
    {
      "event": "stop",
      "native_name": "Stop",
      "transport": "plugin-or-settings-hooks-map",
      "can_block": false,
      "context_channel": "none",
      "notes": "turn-end boundary; observation/capture phase of the dispatcher."
    },
    {
      "event": "pre-compact",
      "native_name": "PreCompact",
      "transport": "plugin-or-settings-hooks-map",
      "can_block": false,
      "context_channel": "stdout-additional-context",
      "notes": "context carried into the compacted window."
    },
    {
      "event": "notification",
      "native_name": "Notification",
      "transport": "plugin-or-settings-hooks-map",
      "can_block": false,
      "context_channel": "none",
      "notes": "outbound attention notice from the harness."
    },
    {
      "event": "session-end",
      "native_name": "SessionEnd",
      "transport": "plugin-or-settings-hooks-map",
      "can_block": false,
      "context_channel": "none",
      "notes": "cleanup boundary."
    }
  ],
  "injection_channel": {
    "kind": "stdout-additional-context",
    "mechanism": "hook stdout JSON outcome additionalContext, per the Claude hook grammar zcode implements; exit 2 stderr is the deny channel.",
    "notes": "machine observation covers the plugin facet; the settings path for user hooks is not claimed here and must be read from the harness's own surface at install time."
  },
  "blocking_semantics": {
    "kind": "deny-and-block",
    "notes": "pre-tool-use and user-prompt-submit accept deny via exit 2 / decision block; matching the shared hook grammar."
  },
  "wake_capability": {
    "kind": "none",
    "notes": "hooks run inside a live session only; no external wake channel is declared."
  },
  "install_seam": {
    "config_path": "~/.zcode/cli/plugins",
    "format": "json",
    "entry_shape": "installed plugin carrying hook declarations (plugin manifest + hooks map); the installing projection records its own manifest id",
    "ownership_marker": "plugin manifest id / hook command resolves to the AIKit dispatch executable",
    "preserves_foreign_entries": true
  },
  "uninstall_seam": {
    "config_path": "~/.zcode/cli/plugins",
    "format": "json",
    "entry_shape": "remove the projection-owned plugin manifest; foreign plugins and settings are untouched",
    "ownership_marker": "plugin manifest id / hook command resolves to the AIKit dispatch executable",
    "preserves_foreign_entries": true
  },
  "provenance": {
    "authored_by": "O:I capability descriptor programme (plugin facet observed on this machine; event grammar declared from the Claude-grammar hook surface zcode documents, 2026-09-06)",
    "source_refs": [
      "survey:local-machine-2026-09-06",
      "upstream:zcode-hook-grammar"
    ],
    "catalog_revision": 3
  }
};
