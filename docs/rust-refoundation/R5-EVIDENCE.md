# R5 source and evidence provenance

R5 began from accepted/reverified R4 main
`0ac4bbb47cc37a7ac3c1db42b88a39ffd6bb5b30`. Its independently tested source
was tree `5dfe4fcfa3f8a4701251f73870ff04d04a51dca4`, transported as a bounded,
SHA-256-checked source capsule because the authoring environment has no direct
Git push. The importer authors no code and is not a parallel agent.

The decoded source hash was
`e7eca501123740ea1b01daa76c4bc5a08628134ff2e5a0e29fe2fda9742e7de2`.
Every transport Git tree was compared independently with the locally computed
tree before publication. Source admission and all tests passed in workflow
**34509457408**, job **102979679569**. Exact clean-source artifact **10165201648**
has ZIP SHA-256
`3bb662c4fd2ac395389f7e94c96f8a98c91d016434a07a127eb1f82622c25088`.
The returned Git bundle was independently loaded and its source tree compared.

During the tranche, non-migration PR #64 was accepted on main at
`80432d7dba7e8c909e8871352625df524e92cfc8`. It bundles the existing Node entry
for the current single-file installation contract. R5 preserves all three of
its changed files exactly: `bin/actuation`, `scripts/bundle.mjs`, and
`scripts/entry.mjs`. Its accepted-main source bundle (artifact **10164768015**,
workflow **34508425744**) was retrieved and checked. This is preservation of an
accepted installation repair, not an early Rust cutover or a new Node product.
The integrated native-source tree before this evidence note is
`85663efac5ec200a358da18c4b47c2c43e9518e6`; the full native gate, original Node
suite, served verify, pure/scenario parity and Skills were rerun against it.

The final PR consists of readable Rust, catalogue data, immutable supplemental
regressions and documentation. Its temporary writable importer and encoded
capsule are absent. Final read-only PR CI and merged-main CI remain the
acceptance authorities and must be checked on their exact SHAs before the
next serial tranche begins.

Native execution comprises 84 integration tests and one identity compile-fail
doctest; 119/44/213/179 R2/R3/R4/R5 pure cases; 29 original-loop cases; 21 durable
store scenarios; 13 observation scenarios; and 57 fresh-file interoperability
assertions through 27 Rust processes. Controlled native filesystem, process,
HTTP and vault-shaped responses are D evidence. Source-pinned Node/Rust
comparison is limited cross-implementation conformance. No real provider P,
owner-machine M, human H, R9 or R10/S7 acceptance is claimed by these tests.
