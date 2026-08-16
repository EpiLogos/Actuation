# Runtime contract

The shared contract standardises only the seam required to replace recurrence beneath one host.

A `LoopRuntime` has `id`, `version` and `run(request, host, observer, signal)`. A `RuntimeHost` supplies exterior operations through `callModel`, `executeCapability`, `receiveExternalInput` and `readContext`. A `RuntimeObserver` accepts events.

The contract deliberately contains no QL positions, relations, residues, conjugation packets, child circuits, framework message types, graph nodes or session ontology. Runtime-specific state belongs to the selected runtime.

Run results distinguish `completed`, `failed`, `cancelled` and `exhausted`.
