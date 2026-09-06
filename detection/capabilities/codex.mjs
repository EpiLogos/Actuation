// Declared harness capability. What codex IS for dispatch projections.
// Grounded in codex-cli 0.153.0 as observed on this machine: a per-project
// hooks.json (session_start / pre_tool_use / post_tool_use state entries
// under [hooks.state]) plus the config.toml notify program that fires at
// turn completion. Injection, blocking and wake are declared as the honest
// absences they are today.
export default {
  "schema": "actuation.harness-capability/v1",
  "document": "capability",
  "harness_slug": "codex",
  "summary": "Per-project hooks.json covers session_start / pre_tool_use / post_tool_use; config.toml notify invokes an external program at turn completion; no declared context-injection channel, no deny-and-block, no harness wake.",
  "native_events": [
    {
      "event": "session-start",
      "native_name": "session_start",
      "transport": "hooks-json-file",
      "can_block": false,
      "context_channel": "none",
      "notes": "hook entry observed as .codex/hooks.json state ([hooks.state.\"<path>:session_start:0:0\"]); no context-injection behaviour is declared for it."
    },
    {
      "event": "pre-tool-use",
      "native_name": "pre_tool_use",
      "transport": "hooks-json-file",
      "can_block": false,
      "context_channel": "none",
      "notes": "observed entry shape as for session_start; no deny semantics declared."
    },
    {
      "event": "post-tool-use",
      "native_name": "post_tool_use",
      "transport": "hooks-json-file",
      "can_block": false,
      "context_channel": "none",
      "notes": "observed entry shape as for session_start."
    },
    {
      "event": "notification",
      "native_name": "notify: agent-turn-complete",
      "transport": "toml-notify",
      "can_block": false,
      "context_channel": "none",
      "notes": "config.toml notify = [<program>, ...args] invoked at turn completion; outbound only. This is the closest native boundary to stop."
    }
  ],
  "injection_channel": {
    "kind": "none",
    "mechanism": "no context-injection channel is declared on codex-cli 0.153.0: hooks and notify are observed as invocations, not as context providers. Projections must treat injected content as unavailable rather than print to stdout and hope.",
    "notes": "re-declare this descriptor when a codex release grows an additional-context channel."
  },
  "blocking_semantics": {
    "kind": "none",
    "notes": "no deny-and-block channel is declared; a hook cannot veto a tool call on this surface."
  },
  "wake_capability": {
    "kind": "none",
    "notes": "notify wakes the listener program at turn completion, never the harness itself. A relay sender must not pretend codex can be woken."
  },
  "install_seam": {
    "config_path": ".codex/hooks.json",
    "format": "json",
    "entry_shape": "per-event hook command entries (session_start / pre_tool_use / post_tool_use); notify listener configured separately in config.toml",
    "ownership_marker": "hook command resolves to the AIKit dispatch executable",
    "preserves_foreign_entries": true
  },
  "uninstall_seam": {
    "config_path": ".codex/hooks.json",
    "format": "json",
    "entry_shape": "entries whose command matches the ownership marker are removed; foreign entries and config.toml notify are untouched",
    "ownership_marker": "hook command resolves to the AIKit dispatch executable",
    "preserves_foreign_entries": true
  },
  "provenance": {
    "authored_by": "O:I capability descriptor programme (codex-cli 0.153.0 observed on this machine: config.toml notify + [hooks.state] entries, 2026-09-06)",
    "source_refs": [
      "survey:local-machine-2026-09-06",
      "upstream:openai-codex-cli-0.153.0"
    ],
    "catalog_revision": 3
  }
};
