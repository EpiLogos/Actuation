# Pi matched runtime experiment — issue 101

Active baseline over the frozen Foundation. The Pi host boundary is constant while each run selects either the classic or ql-core runtime.

Upstream pin: earendil-works/pi at 9d2ec7ffabe927bfad2214c1cee25b6632a78dcf. The inspected upstream surfaces are packages/agent/src/agent.ts and packages/coding-agent/src/core/agent-session-runtime.ts.

The shared host adapter contains no QL position, relation, closure, or re-entry logic. The QL directory contains deterministic task policy; all QL recurrence semantics remain in the unchanged Foundation ql-core runtime. Classic uses the unchanged Foundation classic runtime.

The baseline smoke holds task, model, capability, environment and start-state inputs constant; records host and runtime events separately; checks Classic semantic status is not_applicable; checks positive QL closure precedes re-entry; and proves stored records replay without the live host.

Deep QL operators remain out of scope until later convergence at issue 109.
