# R2 — Constitutional Rust kernel evidence

PR #60 is serial PR B of #58, based on accepted main `e21cf7540ea67cfb9f8746c60823d2aea0b3880a`. R0/R1 main was re-read and its successful push workflow 34433457494 and exact-main source bundle were inspected before this branch began.

## Native candidate

Source tree `8e80074bd44dd60755862e16e236362795a0a80b` contains the typed `actuation-core` workspace, normal native domain operations, tests, read-only dual-platform native CI and R2 implementation notes. No original Node product source or public schema changed. A deterministic source capsule identified every changed path, original blob and exact replacement bytes; its decoded SHA-256 is `0c6cb2cf3a8448a90528c3673617d9635676bd410603eb5f67024f36a85258d1`.

The single acting agent authored and exercised this tree in the network-isolated Linux authoring environment. Formatting, locked all-target Clippy with warnings denied, 18 native integration tests, one compile-fail identity doctest and 119 constitutional Node-to-Rust JSONL parity cases passed. The original product independently passed 219 native tests, the full 599-case public JSON oracle and all 67 state/effect/CLI scenarios.

## CI re-execution and publication

Workflow 34434898546 / job 102737820804 imported only the hash- and tree-checked candidate, removed the capsule/import workflow and repeated all those native, Node, corpus and Skill checks successfully. Its later Git publication step was refused because the workflow token lacked permission to update workflow files. This is a publication failure, **not an all-green workflow** and not main acceptance.

The exact source tree was available through the connected repository API and was published using that authorised write surface. Artifact 10135810250 preserved the candidate Git bundle, archive and test output; archive ZIP SHA-256 is `3505d46c55a29ad3e27b2976ec73ae8bf70a2e0edc823928ad7df19430544dfa`. The bundled candidate commit is `bae7f12368b40dd8ca01c318fe05985dbe7c2a2c`, tree `8e80074bd44dd60755862e16e236362795a0a80b`; independent local diff against the authored tree was empty. That candidate commit was not itself accepted main.

This final publication contains readable source plus this evidence note. The temporary source capsule and writable importer are absent. Only subsequent read-only checks on the published PR source, including both Linux and macOS native jobs, can authorize merge; merged main must then be re-read and verified before R3.

All reported execution is deterministic D evidence. There is no P/M/H claim, provider selection, material allocation, actualisation runtime, physical actuation or human Recognition in R2. The original Node product remains served until R7. The immutable R0 ledger remains a source baseline; R2 acceptance is accounted here rather than mutating its frozen hashes.
