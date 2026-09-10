# R3 — Actualisation and realised runtime

Serial PR C of #58, following R2 PR #60 and accepted main `05e440f4628bbc40eca30f2cabb869a76e99ddd9`. Its push workflow 34435129965 passed on Linux/macOS and the original Node/research gates. Exact-main source artifact 10135881458 (ZIP SHA-256 `073feaeccafad19d2d86500069fb87616d54122ef4b0242b549aaf562ee9a430`) was re-read, checked out and independently reverified before R3 began.

## One actuality, not a second harness

`actuation-runtime` uses `actuation-core`'s identity, WorldBinding, bounds, determination and Return types. It introduces no session host, model selector, provider registry, process daemon, Workcell allocator or QL kernel. Its Rust futures are executor-neutral; a caller supplies the actual host and observer.

`ActualisationRequest::admit` executes strict holder/binding/authority/bounds/identity admission and returns an `Actualisation`, not a launched process. Its typed `ActualisationLineage` requires a unique complete ordered ancestry; this does not narrow the core's separate aggregate reading. The receipt explicitly reports materialisation, Factory Recognition and source mutation as not performed. Context and participation remain provenance, never replacement grants. `admit_return` checks exact determination and Agency lineage before any later synthesis.

`RealisedActuation` admits acting recurrence plus attributable observation. Body/session/process/model/material references remain opaque and preserve absent/null/extension wire law. Its continuity delta changes only the actual facts compared: replacing all five body facts does not mint a new Agent, Agency or WorldBinding. A model's mere availability is not an acting receipt.

`Actuator<D: LocusDriver>` admits reports against the exact binding and prior actuation identity. Managed and discovered modes have the same semantic grammar. Discovered mode does not implicitly launch a body. Beginning twice, silently reopening a terminal actuation and replacing identity during continuation are refused. Unsupported controls leave standing unchanged; an interrupt/cancel/terminate success needs the corresponding observed post-state. A driver error is not a success. No capabilities are inferred from the product's ability to name a method.

`collect_return` takes only a supplied, attributable new Return. It preserves difference and exact supplied correlations. Collection cannot make it received, recognised or materially applied. Historical reading of already-received Returns remains available through the unchanged public contract; this fresh-return port does not rewrite that contract.

## Generic runtime extraction

The A-disposition source is `foundation/runtime-contract/index.js` and `foundation/classic-runtime/index.js`, not the historical QL formal kernel. Their generic host/carrier, registry, bounded loop, follow-up, tool failure, cancellation and observer mechanics now live in this native crate.

`RuntimeHost` provides model, capability, external-input and context encounters. Typed `Carrier` dispatch has no QL vocabulary; opaque host payloads are retained, but cannot shadow the actual request/cancellation object. `RuntimeRegistry` preserves declaration order and rejects duplicate/missing runtimes. `ClassicRuntime` performs ordinary model→capability→follow-up recurrence with distinct completed/failed/cancelled/exhausted outcomes. Cancellation is cooperative acknowledgement, never a fabricated process kill. `RuntimeObserver` returns refs for evidence it actually accepted; refusal fails the operation.

`LoopLocusDriver` makes the ordinary loop directly usable as managed agency in a supplied WorldBinding. It neither resolves nor instantiates a harness. A plain `file:///project` World plus an acting host suffices; AIKit and Factory are not prerequisites. Optional richer external body refs use the identical domain. The bounded driver records a failed/exhausted execution truthfully and cannot claim acting model actuality when no model return was observed. Native process interrupt/cancel/terminate and automatic structured Return synthesis remain explicitly unsupported by that driver. A suitable target can implement those faculties through `LocusDriver` with real evidence.

The older generic JS remains temporarily callable by the still-Node product/research programme. It is A migration residue, **not** a C language exemption or abandoned research. R6 migrates its research consumers; R7 retires the ordinary Node product. QL's active deeper programmes remain required and unchanged in this tranche.

## Executable evidence

All 44 frozen realised/actualisation cases match the Node oracle. A separate immutable 29-case source-locked extraction corpus captures the original generic loop's exact model/tool/human/context calls, events and result; it includes zero/multiple tools, failed-tool recovery, false/empty follow-up, cancellation, exhaustion, host failures, carrier dispatch and registry integrity. It is generated once from unchanged baseline source, hash-checked, and compiled into native tests; passing CI never recaptures it. Original R1 expectations and the frozen source ledger are untouched.

Sixteen runtime tests exercise that corpus, all realised/actualisation cases, Direct/rich body fixtures, managed/discovered parity, unsupported lifecycle, identity/observation refusal, Return attribution/Recognition separation and actualisation authority failures. Together with core this is 34 native integration tests and one compile-fail identity doctest. Source-tree formatting, locked all-target Clippy with warnings denied, native tests and both R2/R3 parity transports run on the existing Linux/macOS CI matrix. The original 219 native Node tests and full 599/67 oracle remain active, with Foundation/Deep/Prime/epistemic checks unchanged.

These are deterministic D observations. The test host and test driver are explicit fixtures, not real provider or owner-machine evidence. Accepted main and final CI run IDs must be recorded after merge. R4 cannot begin until this PR is accepted and its resulting main is re-read and verified.
