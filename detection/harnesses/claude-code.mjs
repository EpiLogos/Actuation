// Declared harness descriptor. Presence is detection's job, never this file's.
// To incorporate a new harness: add one descriptor module here, then import it
// in ../catalog.mjs and bump CATALOG_REVISION.
export default {
  "schema": "actuation.harness-detection/v1",
  "document": "descriptor",
  "slug": "claude-code",
  "native_kind": "harness",
  "edition": "cli",
  "native_owner": "Anthropic",
  "summary": "Claude Code agent harness",
  "aliases": [
    "claude"
  ],
  "probe": {
    "executable": {
      "names": [
        "claude"
      ],
      "version_args": [
        "--version"
      ]
    },
    "config-dir": {
      "path": "~/.claude"
    }
  },
  "facets": {
    "skills": {
      "path": "~/.claude/skills"
    },
    "plugins": {
      "path": "~/.claude/plugins"
    },
    "commands": {
      "path": "~/.claude/commands"
    },
    "hooks": {
      "path": "~/.claude/hooks"
    },
    "settings": {
      "path": "~/.claude/settings.json"
    }
  },
  "provenance": {
    "authored_by": "O:I detection programme (agent survey of this machine, 2026-09-05)",
    "source_refs": [
      "survey:local-machine-2026-09-05"
    ],
    "catalog_revision": 1
  }
};
