// Declared harness descriptor. Presence is detection's job, never this file's.
// To incorporate a new harness: add one descriptor module here, then import it
// in ../catalog.mjs and bump CATALOG_REVISION.
export default {
  "schema": "actuation.harness-detection/v1",
  "document": "descriptor",
  "slug": "gemini",
  "native_kind": "harness",
  "edition": "cli",
  "native_owner": "Google",
  "summary": "Gemini CLI agent harness; antigravity ships inside its config tree",
  "probe": {
    "executable": {
      "names": [
        "gemini"
      ],
      "version_args": [
        "--version"
      ]
    },
    "config-dir": {
      "path": "~/.gemini"
    }
  },
  "facets": {
    "extensions": {
      "path": "~/.gemini/extensions"
    },
    "settings": {
      "path": "~/.gemini/settings.json"
    },
    "agents": {
      "path": "~/.gemini/antigravity"
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
