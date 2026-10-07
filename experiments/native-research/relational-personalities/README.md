# Four-agent acquaintance

A native, bounded observation of Kite, Cairn, Lilt and Flint becoming acquainted after the authored-self feature is integrated.

Read [PROTOCOL.md](PROTOCOL.md), [experiment.json](experiment.json), and the [integration specification](../../../docs/agent-self/INTEGRATION-SPEC.md). Begin local execution from [LOCAL-EXECUTION.md](../../../docs/agent-self/LOCAL-EXECUTION.md).

`personas/` contains four candidate self-sources; `ROOM.md` is the common participant brief. These are not registered identities. The native Agent-expression path must allocate and accept actual definitions and bind their source pins before sessions start. The shared Logos lives at `skills/author-agent-self/references/RELATIONAL-LOGOS.md`.

`experiment.json` is an experimental input, not an owner Run/NOW/session schema. Runtime refs and private evidence belong to the real Factory/NOW/Actuation/AIKit records. `check.py` checks source integrity and input contracts; `analyse.py` calculates clearly labelled derived summaries from an independently annotated evidence projection. Neither script launches agents or establishes native authority.

From this directory: `python3 check.py` and `python3 -m unittest discover -s tests -v`. The local implementer must build the thin native experiment adapter described in the protocol and register its actual invocation in the Factory RunMap. A generic API role-play is not a substitute.
