// Declared harness descriptor. Presence is detection's job, never this file's.
// To incorporate a new harness: add one descriptor module here, then import it
// in ../catalog.mjs and bump CATALOG_REVISION.
export default {
  "schema": "actuation.harness-detection/v1",
  "document": "descriptor",
  "slug": "codex",
  "native_kind": "harness",
  "edition": "cli",
  "native_owner": "OpenAI",
  "summary": "Codex CLI agent harness",
  "probe": {
    "executable": {
      "names": [
        "codex"
      ],
      "version_args": [
        "--version"
      ]
    },
    "config-dir": {
      "path": "~/.codex"
    }
  },
  "facets": {
    "skills": {
      "path": "~/.codex/skills"
    },
    "rules": {
      "path": "~/.codex/rules"
    },
    "config": {
      "path": "~/.codex/config.toml"
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
