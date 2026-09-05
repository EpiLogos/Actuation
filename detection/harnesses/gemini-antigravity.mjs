// Declared harness descriptor. Presence is detection's job, never this file's.
// To incorporate a new harness: add one descriptor module here, then import it
// in ../catalog.mjs and bump CATALOG_REVISION.
export default {
  "schema": "actuation.harness-detection/v1",
  "document": "descriptor",
  "slug": "gemini-antigravity",
  "native_kind": "harness",
  "edition": "ide",
  "native_owner": "Google",
  "summary": "Antigravity agent (own-slug decision pending; detected inside the gemini config tree)",
  "aliases": [
    "antigravity"
  ],
  "probe": {
    "config-dir": {
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
