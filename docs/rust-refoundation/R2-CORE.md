# R2 — Constitutional Rust core

PR B of #58 starts from accepted/reverified R0/R1 main `e21cf7540ea67cfb9f8746c60823d2aea0b3880a` (#59). Its merged-main read-only verification is workflow 34433457494; the exact source bundle and native/research/oracle logs are artifact 10135317440. R3 has not begun in this tranche.

## The domain, rather than the old file layout

`actuation-core` is the only workspace member at R2. It owns World-relative identity, bindings, positional root, explicit metagency grants, four determination kinds, delegated autonomy, Return policy/provenance and constitutional composition. It depends only on serde/serde_json and the Rust standard library. There is no provider, async executor, filesystem operation, QL dependency or other suite implementation in this crate.

Agent, Agency, WorldBinding, Scope, Actuation, Session, Run and the other correlation refs have nominal types. A Session cannot be supplied as an Agent by an accidental assignment. Ref admission does not normalise strings or invent addresses. The immutable admitted records contain typed fields, not a private JSON validator catalogue: callers can build typed drafts, admit them, read positional/root and authority relations, inspect autonomy, compose attributable Returns, and record reception through the same domain API used by the compatibility surface.

`AgenticComposition::new(binding)` is the ordinary one-binding constitutional case. It does not require AIKit, Factory ancestry or a privileged Agent species. Richer grant, determination and Return relations are added to that same composition. A grant's operation list is a reading, not proof that its World/bounds/holder are admissible; R3 actualisation separately checks the complete act.

Return has only five admitted joint standings: offered, received, rejected, recognised and re-entered. `receive()` records reception and preserves every attributable difference; it never recognises a candidate, mutates an external World or infers a caller. Denial wins over an explicit allowed-action list, while an unlisted action remains unspecified rather than silently permitted.

## Compatibility without a weaker domain

The original `actuation.agency/v1` schema and optional-field meanings remain unchanged. `Slot<T>` distinguishes omitted/null/present values; open-object extensions survive round-trip. Typed draft admission rejects extension keys that try to shadow declared identity or authority fields. Required refs, non-empty sequences, enum alternatives and constitutional cross-field checks cannot be bypassed by deserializing an admitted record.

The public aggregate lineage reading remains distinct from a complete unique contiguous actualisation path. Its accepted ordering behaviour is preserved; R3 must impose the stricter authorisation-path law at the actualisation boundary rather than narrowing the old aggregate wire contract.

The temporary JSONL oracle executable delegates every operation to these ordinary native domain APIs. It is not a product service and contains no fixture-to-output lookup. Static golden expectations are compiled into the Rust integration tests, so the native tests do not require Node or a source checkout to validate their selected corpus.

## Exercised evidence

In the isolated authoring environment, locked Rust compilation, formatting and all-target clippy passed. Eighteen native constitutional tests and one compile-fail identity test passed; the integration gate exercises all 119 frozen constitutional cases. The separate original-Node-to-Rust JSONL runner also passed all 119 cases. Native tests include the complete Return-state truth table, malformed object/array shapes, federation refusal, autonomy/lineage failures, absent/null preservation, extension-shadow attacks, opaque Unicode refs and Direct/no-inferred-Factory readings.

The final PR gate additionally runs these native tests and parity on the existing Linux x86_64/macOS arm64 suite matrix while keeping the complete Node/research oracle active. The ordinary product is still Node at this tranche. No provider, owner-machine or human evidence is claimed. The native packages remain unpublished; this refoundation makes no new licensing grant.

After verified merge, re-read and verify main before R3: realised actuality and the harness-neutral managed/discovered acting relation.
