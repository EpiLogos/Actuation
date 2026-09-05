// Declared harness descriptor. Presence is detection's job, never this file's.
// To incorporate a new harness: add one descriptor module here, then import it
// in ../catalog.mjs and bump CATALOG_REVISION.
export default {
  "schema": "actuation.harness-detection/v1",
  "document": "descriptor",
  "slug": "grok-bot",
  "native_kind": "harness",
  "edition": "cli+service",
  "native_owner": "xAI",
  "summary": "grok-bot harness; runs as a background daemon, version probe may prompt the keychain",
  "aliases": [
    "gbot"
  ],
  "probe": {
    "executable": {
      "names": [
        "grok-bot",
        "gbot"
      ],
      "version_args": [
        "--version"
      ]
    },
    "config-dir": {
      "path": "~/.grokbot"
    },
    "service": {
      "kind": "daemon",
      "name": "grok-bot"
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
