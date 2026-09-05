// Declared harness descriptor. Presence is detection's job, never this file's.
// To incorporate a new harness: add one descriptor module here, then import it
// in ../catalog.mjs and bump CATALOG_REVISION.
export default {
  "schema": "actuation.harness-detection/v1",
  "document": "descriptor",
  "slug": "zcode",
  "native_kind": "harness",
  "edition": "client",
  "native_owner": "ZCode",
  "summary": "zcode agent harness client",
  "probe": {
    "config-dir": {
      "path": "~/.zcode"
    },
    "env": {
      "any_of": [
        "ZCODE_APP_VERSION",
        "ZCODE_ENV"
      ]
    }
  },
  "facets": {
    "plugins": {
      "path": "~/.zcode/cli/plugins"
    }
  },
  "provenance": {
    "authored_by": "O:I detection programme (agent survey of this machine, 2026-09-05)",
    "source_refs": [
      "survey:local-machine-2026-09-05"
    ],
    "catalog_revision": 2
  }
};
