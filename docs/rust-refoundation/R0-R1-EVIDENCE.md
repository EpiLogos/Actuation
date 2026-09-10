# R0/R1 — PR A evidence

This is the reconciliation and executable oracle tranche of #58, not a claim that Rust is already the product.

The baseline is accepted main `1c862c6bf58478adf6842a090214906dd2337001`. The one-time source-locked capture was committed as `52806987db0157b9959ab4518d698ca74598c10e`; its workflow was 34433206816, capture job 102732856247, artifact 10135234984. The artifact ZIP SHA-256 is `ec9c5909e0457de72262f13348181bf40ffbbc32cda8de5118cfbb3f5429f4fb`.

The final committed corpus contains **599 public JSON cases across 57 operations**, **67 fresh-store/effect/CLI scenarios**, the **22-case historical Foundation gate**, and a **250-file exact baseline ledger including 120 research files**. This final capture is authoritative; earlier sandbox discovery counts in R0 describe a candidate, not the committed corpus. Every recorded expectation was replayed against uninstrumented original Node code. The source lock refuses modified original implementation files; the capture includes the original 228 native-plus-Prime assertions and 104 authored adversarial assertions.

Independent reinspection cloned the captured Git bundle at its exact SHA and passed corpus integrity, all 599 pure cases, all 67 scenarios, all 219 native tests, and served `verify --json` with 21 discovered suites and that exact revision. No provider or owner-machine evidence was inferred from this environment.

The first committed helper exposed an integration mistake in migration tooling: its `.test.mjs` suffix was seen by the original tracked-tree test inventory but was outside production native-suite directories. It was renamed `check-recorder.mjs` and remains explicitly executed by `node --test`. Neither the original suite-discovery test nor product implementation was weakened or modified.

The temporary source-study and one-time capture jobs are now removed. The accepted workflow has contents-read permission only, verifies frozen data and archives exact Git evidence. Capture is not a green-gate operation. The expected pre-capture missing-corpus failure is not acceptance evidence; PR A requires the subsequent read-only run to pass, followed by verification of merged main.

Research remains actively exercised: Foundation, Deep QL, Prime and epistemic tests run in the migration gate; the two genuine Python-native integration surfaces are syntax-checked separately. This is deterministic D evidence. Historical formal QL success is not current QL-owner conformance; Python syntax is not Pydantic/Prime provider execution. C, P, M and H are not claimed here.

R2 must begin only after PR #59 is verified, merged and its accepted main re-read and verified. It introduces the typed constitutional Rust kernel while leaving the original Node product intact. R3–R9 remain subsequent serial tranches; R10 / O:I #213 remains actual owner-machine and human proving.
