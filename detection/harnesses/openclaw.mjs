// Declared harness descriptor. Presence is detection's job, never this file's.
// To incorporate a new harness: add one descriptor module here, then import it
// in ../catalog.mjs and bump CATALOG_REVISION.
export default {
  "schema": "actuation.harness-detection/v1",
  "document": "descriptor",
  "slug": "openclaw",
  "native_kind": "harness",
  "edition": "cli",
  "native_owner": "OpenClaw",
  "summary": "OpenClaw agent harness; agent config lives under ~/.openclaw",
  "probe": {
    "executable": {
      "names": [
        "openclaw"
      ],
      "version_args": [
        "--version"
      ]
    },
    "config-dir": {
      "path": "~/.openclaw"
    }
  },
  "facets": {
    "agents": {
      "path": "~/.openclaw/agents"
    },
    "config": {
      "path": "~/.openclaw/openclaw.json"
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
