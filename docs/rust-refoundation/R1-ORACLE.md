# R1 — Executable compatibility oracle

The authority is committed fixture data from source `1c862c6bf58478adf6842a090214906dd2337001`, not a freshly generated expectation. `SHA256SUMS` freezes public JSON cases, state/effect/CLI scenarios, the exact source ledger and historical Foundation gate. Capture refuses changed original source and refuses overwriting an existing output.

```sh
node --test scripts/migration/recorder.test.mjs
node scripts/migration/verify-corpus.mjs
node scripts/migration/parity.mjs
node scripts/migration/scenario-parity.mjs
```

The pure runner discovers each committed operation, checks total ordered responses, success/refusal, and deep semantic JSON. It refuses zero cases. Phase filters select R2–R6. A Rust test executable may consume JSONL requests `{id,operation,args}` and return `{id,ok,value|error}`:

```sh
node scripts/migration/parity.mjs --phase R2 -- target/debug/examples/constitutional-oracle
```

That temporary transport is not an Actuation service contract. Domain operations must remain library code, not hardcoded fixture responses.

State/effect scenarios use fresh temporary stores or isolated CLI invocations and explicit fake-effect replies. Authored assertions constrain capture itself. The store captures literal header/event lines; replay asserts unchanged-on-refusal/dedup and append-only history, alongside semantic cross-language comparison. R4 additionally exercises real files written in both runtime directions. Probe scenarios record exactly which effects ran; plain detection must not run opt-in version probes. Caller zero/one/multiple marker cases preserve unknown and ambiguity. Secret scenarios expose fingerprint/location only. CLI tests compare exit status and successful output; parser wording is diagnostic, but exit 2 must use recognisable `actuation: ` stderr.

Only declared nondeterministic metadata is normalised: CLI revision is checked by native exact-build self-certification instead; secret scan clock/ref is metadata, not proof of a live scan. JSON object member order is not a contract. Arrays, refs, missing versus null, returned evidence and cursor order are compared. Verify's implementation-specific suite filenames will change at native cutover; installed verification must still discover/execute a substantive non-empty native suite, with exact revision and truthful failures.

The recorder observes explicitly selected public synchronous JSON functions while retaining original test assertions. It does not turn mocks into live evidence. Its own tests check result identity, input mutation, thrown identity and asynchronous pass-through. Generic product, Prime, Foundation, Deep and epistemic tests remain active during migration. Historical QL formal fixtures preserve their original scope; current formal ownership is QL-MEF.

Capture is a deliberate one-time R1 development action, never part of passing normal CI. Any later fixture change requires an explicit reviewed semantic reason and preserved old evidence. The reproduced config-only detector receipt bug is recorded in R0 and corrected explicitly in R5 rather than silently blessed by recapture.
