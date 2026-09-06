// Declared harness capability. What claude-code IS for dispatch projections.
// Presence is detection's job; this file declares events, channels, blocking,
// wake and the install seams consumers must use. Detection descriptors live
// beside this directory (../harnesses/); slugs must stay aligned.
export default {
  "schema": "actuation.harness-capability/v1",
  "document": "capability",
  "harness_slug": "claude-code",
  "summary": "Hooks dispatched from the settings hooks map; stdin JSON in, stdout JSON/plain out; PreToolUse and UserPromptSubmit can deny-and-block; Stop can force continuation; no external wake.",
  "native_events": [
    {
      "event": "session-start",
      "native_name": "SessionStart",
      "transport": "settings-json-hooks-map",
      "can_block": false,
      "context_channel": "stdout-additional-context",
      "notes": "stdout (plain or JSON additionalContext) is added to the session's opening context; matcher values source/startup/resume."
    },
    {
      "event": "user-prompt-submit",
      "native_name": "UserPromptSubmit",
      "transport": "settings-json-hooks-map",
      "can_block": true,
      "context_channel": "stdout-additional-context",
      "notes": "exit 2 blocks the prompt; stdout (plain or JSON additionalContext) is added to the turn context."
    },
    {
      "event": "pre-tool-use",
      "native_name": "PreToolUse",
      "transport": "settings-json-hooks-map",
      "can_block": true,
      "context_channel": "stdout-additional-context",
      "notes": "exit 2 denies the tool call for this invocation; JSON hookSpecificOutcome.additionalContext is the pre-tool context channel (not stdout text)."
    },
    {
      "event": "post-tool-use",
      "native_name": "PostToolUse",
      "transport": "settings-json-hooks-map",
      "can_block": false,
      "context_channel": "stdout-additional-context",
      "notes": "runs after tool success; exit 2 shows stderr to the model (advisory), it cannot undo the tool result."
    },
    {
      "event": "stop",
      "native_name": "Stop",
      "transport": "settings-json-hooks-map",
      "can_block": true,
      "context_channel": "none",
      "notes": "JSON decision block forces the model to continue instead of stopping; stdout is not injected as context."
    },
    {
      "event": "session-end",
      "native_name": "SessionEnd",
      "transport": "settings-json-hooks-map",
      "can_block": false,
      "context_channel": "none",
      "notes": "cleanup only; the session is already terminating."
    },
    {
      "event": "pre-compact",
      "native_name": "PreCompact",
      "transport": "settings-json-hooks-map",
      "can_block": false,
      "context_channel": "stdout-additional-context",
      "notes": "stdout becomes a custom instruction carried into the compacted context."
    },
    {
      "event": "notification",
      "native_name": "Notification",
      "transport": "settings-json-hooks-map",
      "can_block": false,
      "context_channel": "none",
      "notes": "outbound: fires when the harness sends a notification (permission needed, idle prompt)."
    }
  ],
  "injection_channel": {
    "kind": "stdout-additional-context",
    "mechanism": "JSON hookSpecificOutcome.additionalContext on stdout for PreToolUse; plain stdout (or JSON additionalContext) added to context at session-start, user-prompt-submit and pre-compact; exit 2 stderr is the deny channel, never a context channel.",
    "notes": "Pre-tool context must travel hookSpecificOutcome.additionalContext; text printed to stdout outside the JSON outcome is not the pre-tool channel."
  },
  "blocking_semantics": {
    "kind": "deny-and-block",
    "notes": "PreToolUse and UserPromptSubmit exit 2 (or JSON decision block with reason) deny the operation; Stop JSON decision block forces continuation. The dispatcher's gate phase rides exit codes and JSON decisions."
  },
  "wake_capability": {
    "kind": "none",
    "notes": "hooks execute only inside a running session; nothing declared here wakes the harness from outside. Notification is the harness asking for attention, not accepting a wake."
  },
  "install_seam": {
    "config_path": "~/.claude/settings.json",
    "format": "json",
    "entry_shape": "hooks.<EventName>[] entries, each {matcher?, hooks: [{type: \"command\", command, timeout?}]}",
    "ownership_marker": "hook command resolves to the AIKit dispatch executable",
    "preserves_foreign_entries": true
  },
  "uninstall_seam": {
    "config_path": "~/.claude/settings.json",
    "format": "json",
    "entry_shape": "hooks.<EventName>[] entries whose command matches the ownership marker are removed; entries without the marker are untouched",
    "ownership_marker": "hook command resolves to the AIKit dispatch executable",
    "preserves_foreign_entries": true
  },
  "provenance": {
    "authored_by": "O:I capability descriptor programme (live settings/hooks observation on this machine + upstream hooks documentation, 2026-09-06)",
    "source_refs": [
      "survey:local-machine-2026-09-06",
      "upstream:claude-code-hooks-reference"
    ],
    "catalog_revision": 3
  }
};
