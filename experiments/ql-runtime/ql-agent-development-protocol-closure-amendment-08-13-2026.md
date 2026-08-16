# QL Agent Development Protocol — Closure Amendment

**Date:** 2026-08-13  
**Companion to:** `ql-agent-development-protocol-08-13-2026.md` and `LOOP-CLOSURE-CLARIFICATION.md`

This amendment changes development factoring, not QL semantics.

## Foundation tightening before F2

Before Foundation Freeze is treated as the branch point for #101–#104:

1. Preserve the existing Host / Loop Runtime seam.
2. Within the foundation, factor mechanically shared recurrent carrier/driver behaviour from selectable loop logic where doing so is clean and does not erase host-native differences.
3. Keep `classic` and `ql-core` as the two selectable experimental conditions. QL still changes interpretation, recurrence, determination, closure and re-entry; it is not reduced to a stop predicate.
4. Make `Run != Closure` explicit in common optics. A Run is execution/chronology; QL closure is a positive semantic determination and may depend on evidence produced by one or more Runs or exterior encounters.
5. Extend deterministic closure fixtures with:
   - a current verification result whose subject/state matches the candidate being determined;
   - a negative stale or subject/state-mismatched verification case.
6. Re-run the deterministic foundation gate against the exact state proposed as the common branch point and pin that state in the foundation manifest.

## MEF / QL kernel relation

The L3 concrescence, L3-prime chronology, L1 causal-disclosure and L4-prime knowledge-work readings are semantic refractions of the underlying runtime architecture. They inform design but do not become dependencies of the generic runtime contract.

The QL Agent experiments and the QL/MEF kernel may mature independently and later integrate: the kernel should be able to project and refract the successful agent architecture and traces without redefining the underlying technical primitives.
