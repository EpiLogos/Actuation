// Declared harness descriptor. Presence is detection's job, never this file's.
// To incorporate a new harness: add one descriptor module here, then import it
// in ../catalog.mjs and bump CATALOG_REVISION.
export default {
  "schema": "actuation.harness-detection/v1",
  "document": "descriptor",
  "slug": "hermes-acp",
  "native_kind": "harness",
  "edition": "acp-bridge",
  "native_owner": "Nous Research",
  "summary": "Hermes ACP bridge; shares the hermes config tree",
  "probe": {
    "executable": {
      "names": [
        "hermes-acp"
      ],
      "version_args": [
        "--version"
      ]
    },
    "config-dir": {
      "path": "~/.hermes"
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
