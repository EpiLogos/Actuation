# Loop and Closure Clarification

Status: normative clarification for the QL agent specification and development protocol.

## Runtime factoring

The host supplies model, capability, context and external I/O mechanics. The runtime owns recurrence. Within that seam, implementations should share mechanically identical loop-driver behaviour where practical and vary the loop logic that interprets returns, selects recurrence, evaluates completion and retains re-entry state.

Classic and QL remain selectable runtime conditions. QL is not merely a stop predicate: interpreted return, contextual Rij movement, typed residue, explicit P5 determination, typed P5 continuation, positive closure and retained re-entry remain QL semantics.

## Run is not Closure

A Run is an execution and observation unit. It records chronology, calls, effects, evidence and nested work. Closure is the semantic determination that the operative whole has attained sufficient determinacy relative to its initiating intent and governing conditions.

Therefore Run termination, process exit, tool exhaustion and a completion claim are not by themselves closure. Closure may depend on evidence produced by one or more runs or exterior encounters. Run optics should represent execution status separately from closure status.

## P5 and positive closure

P5 produces a candidate determination. R51/R52/R53/R54/R50 mean that this still-open process continues toward a disclosed insufficiency. It has not closed and then reopened. Only a positive QLClosure closes the circuit. ReentryDelta then carries retained difference into possible P0+.

## Causal disclosure

Causal disclosure names the process by which the conditions warranting a completion determination become sufficiently explicit, current and related to the operative subject that closure can be established. It is intentionally distinct from the established phrase causal closure.

A useful software invariant is:

Completion Claim must cite Current Verification Results whose Subject matches the state being claimed complete.

Where verification is relevant, stale evidence or evidence for a different subject/state must not by itself warrant closure. This is a concrete P4 to P5 integrity condition, not a requirement that every task use CI or mechanical tests.

## MEF interpretation without dependency

The existing MEF material makes L3 Processual the natural refraction of the closure-bearing recurrence as concrescence: inherited datum plus novel encounter, integration, candidate determination, satisfaction, and retained datum for what follows. L3-prime reveals actual chronology; L1 reveals causal constitution and causal disclosure; L4-prime reveals knowledge-work and evidential activity.

This alignment does not make the QL agent runtime depend on an MEF service. Ordinary technical primitives should exist independently. QL form and laws inform their design; a QL/MEF kernel can later project and refract those primitives as semantically loaded meta-intelligence. A successful QL agent is a natural integration target, not a dependency of the QL/MEF kernel.

## Experiment consequences

The A/B experiment should hold host and shareable loop mechanics steady while varying loop logic. Run manifests should separate execution and closure state and link closure to the evidence or runs on which it depends. Deterministic fixtures should cover current subject-matched verification and stale or mismatched verification as a negative case.
