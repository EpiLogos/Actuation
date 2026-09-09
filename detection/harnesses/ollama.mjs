// Declared harness descriptor. Presence is detection's job, never this file's.
// To incorporate a new harness: add one descriptor module here, then import it
// in ../catalog.mjs and bump CATALOG_REVISION.
export default {
  "schema": "actuation.harness-detection/v1",
  "document": "descriptor",
  "slug": "ollama",
  "native_kind": "model-provider",
  "edition": "cli+service",
  "native_owner": "Ollama",
  "summary": "local model runtime and provider; detected for model binding, not agency",
  "probe": {
    "executable": {
      "names": [
        "ollama"
      ],
      "version_args": [
        "--version"
      ]
    },
    "config-dir": {
      "path": "~/.ollama"
    },
    "service": {
      "kind": "http",
      "default_url": "http://127.0.0.1:11434"
    }
  },
  "facets": {
    "models": {
      "path": "~/.ollama/models",
      "inventory": {
        "kind": "http-json",
        "from": "service",
        "route": "/api/tags",
        "collection": "models",
        "id_field": "model",
        "also_id_fields": [
          "name"
        ],
        "detail_fields": [
          "digest",
          "size",
          "modified_at"
        ]
      }
    }
  },
  "provenance": {
    "authored_by": "O:I detection programme (agent survey of this machine, 2026-09-05)",
    "source_refs": [
      "survey:local-machine-2026-09-05",
      "https://github.com/ollama/ollama/blob/main/docs/api.md"
    ],
    "catalog_revision": 5
  }
};
