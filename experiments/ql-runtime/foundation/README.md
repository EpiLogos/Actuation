# QL Agent Runtime Foundation

This directory is the portable Foundation Freeze surface for issues #95–#100.

The boundary is intentionally small:

- `runtime-contract/` defines the harness-neutral `LoopRuntime`, `RuntimeHost`, observer and run-result seam.
- `classic-runtime/` holds the minimal ordinary recurrence control.
- `ql-core-runtime/` holds the Direct Core QL recurrence kernel. QL state stays private to that runtime.
- `optics/` records matched manifests, host events and runtime-semantic traces.
- `fixtures/` contains deterministic scripted hosts, policies and the Foundation Freeze fixture gate.
- `test/` proves the contract and both recurrence conditions with Node 22 `node:test`.

The foundation does not implement Pi, Pydantic AI, Native harness adapters, conjugation, recursive depth, full MEF runtime operations, or the later full Experiment Readiness profile. Those surfaces branch from the Foundation Freeze described by the development protocol.
