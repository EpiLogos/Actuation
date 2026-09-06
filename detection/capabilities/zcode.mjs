// Declared harness capability. What zcode IS for dispatch projections.
// Grounded in zcode's own shipped hook documentation (the zcode-guide
// plugin's diagnosing-hooks skill): exactly seven native events, plugin
// hooks via hooks/hooks.json, and configuration-file hooks under
// ~/.zcode/cli/config.json that stay disabled until hooks.enabled: true.
// Notification and PreCompact are NOT supported — a deliberate difference
// from claude-code that consumers must read from here, never assume away.
export default {
  "schema": "actuation.harness-capability/v1",
  "document": "capability",
  "harness_slug": "zcode",
  "summary": "Seven native hook events (SessionStart, UserPromptSubmit, PreToolUse, PermissionRequest, PostToolUse, PostToolUseFailure, Stop); stdout JSON additionalContext; exit 2 denies pre-tool and permission boundaries; configuration-file hooks need hooks.enabled, plugin hooks auto-enable; no PreCompact, no Notification, no external wake.",
  "native_events": [
    {
      "event": "session-start",
      "native_name": "SessionStart",
      "transport": "config-json-hooks-map",
      "can_block": false,
      "context_channel": "stdout-additional-context",
      "notes": "matcher value is the session mode: startup, resume, clear or compact."
    },
    {
      "event": "user-prompt-submit",
      "native_name": "UserPromptSubmit",
      "transport": "config-json-hooks-map",
      "can_block": true,
      "context_channel": "stdout-additional-context",
      "notes": "matcher value is the prompt text; stdout JSON additionalContext joins the turn."
    },
    {
      "event": "pre-tool-use",
      "native_name": "PreToolUse",
      "transport": "config-json-hooks-map",
      "can_block": true,
      "context_channel": "stdout-additional-context",
      "notes": "exit 2 is a deny for this invocation; the hook may also return a permission decision of allow/ask/deny. additionalContext is injected."
    },
    {
      "event": "post-tool-use",
      "native_name": "PostToolUse",
      "transport": "config-json-hooks-map",
      "can_block": false,
      "context_channel": "stdout-additional-context",
      "notes": "runs after tool success; advisory."
    },
    {
      "event": "stop",
      "native_name": "Stop",
      "transport": "config-json-hooks-map",
      "can_block": true,
      "context_channel": "none",
      "notes": "the hook may request continuation, up to three times."
    },
    {
      "event": "custom",
      "native_name": "PermissionRequest",
      "transport": "config-json-hooks-map",
      "can_block": true,
      "context_channel": "exit-code-payload",
      "notes": "no AIKit boundary kind; exit 2 denies the permission request. Declared so consumers see the whole native surface."
    },
    {
      "event": "custom",
      "native_name": "PostToolUseFailure",
      "transport": "config-json-hooks-map",
      "can_block": false,
      "context_channel": "none",
      "notes": "no AIKit boundary kind; fires after a failed tool call."
    }
  ],
  "injection_channel": {
    "kind": "stdout-additional-context",
    "mechanism": "hook stdout is parsed as a strict JSON schema; additionalContext is injected into the conversation. Exit codes carry the deny channel: 0 pass, 2 block, other non-zero error.",
    "notes": "the schema is strict — any extra key fails validation, so a projection must emit exactly the admitted fields."
  },
  "blocking_semantics": {
    "kind": "deny-and-block",
    "notes": "PreToolUse and PermissionRequest accept exit-2 denies; Stop accepts bounded continuation requests."
  },
  "wake_capability": {
    "kind": "none",
    "notes": "hooks run inside a live session only; there is no Notification event and no external wake channel."
  },
  "install_seam": {
    "config_path": "~/.zcode/cli/config.json",
    "format": "json",
    "entry_shape": "top-level hooks block: { enabled: true, events: { <Event>: [ { matcher?, hooks: [{type: \"command\", command, ...}] } ] } }; configuration-file hooks stay disabled until hooks.enabled is true",
    "ownership_marker": "hook command resolves to the AIKit dispatch executable",
    "preserves_foreign_entries": true
  },
  "uninstall_seam": {
    "config_path": "~/.zcode/cli/config.json",
    "format": "json",
    "entry_shape": "entries whose command matches the ownership marker are removed and empty event keys pruned; foreign hooks and every other top-level key are untouched",
    "ownership_marker": "hook command resolves to the AIKit dispatch executable",
    "preserves_foreign_entries": true
  },
  "provenance": {
    "authored_by": "O:I capability descriptor programme (zcode hook grammar read from the shipped zcode-guide plugin's diagnosing-hooks documentation; config surfaces observed on this machine, 2026-09-06)",
    "source_refs": [
      "survey:local-machine-2026-09-06",
      "upstream:zcode-guide-diagnosing-hooks",
      "correction:rev1-claimed-claude-grammar-with-precompact-notification;the-shipped-event-list-says-otherwise"
    ],
    "catalog_revision": 4
  }
};
