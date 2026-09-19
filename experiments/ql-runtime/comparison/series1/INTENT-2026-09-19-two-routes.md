# Series 1 session intent — two routes (2026-09-19)

Owner's design position for this session, stated plainly. Round 9 left the
toolset arm as the strongest QL condition; the owner's correction gives the
line its fuller shape instead of another isolated arm.

## The design position

- A referee like jev is not needed on the small closures. RESTRAINT closed at
  classic's call economy; gating that with a classifier is spend without a
  question. The closing decision belongs to the loop the model is running.
- The model in fact has **two loops**: the basic work loop (classic-shaped,
  four world tools) and the QL task loop (the full return loop). Give the four
  basic tools a clean #1–#4 QL landing, just like the full sixfold. The two
  routes then hold the QL 4/6 distinction:
  - **Route one — the pure explicate four.** classic-shaped, but each tool's
    description names the office its return settles in (P1, P1, P2, P4).
    The work ends the way classic ends. QL is present as tags, not as a
    second loop.
  - **Route two — the 4+2 implicate-inclusive form.** The four tagged tools
    plus situate (the return reading) and close (the return condition).
    For deeper uses.
- Route two is the experimentation branch: the options for how the closing is
  checked (self / model / jev) live there, not on route one.
- Jev gets refined for specific work, and its real home is the MEF
  classification thread — reading across the full lens matrix at once, not
  gate-keeping tiny closures.

## The stipulations as they stand (recovered, not re-derived)

The six tool descriptions are the only law the model ever sees
(`crates/actuation-research/src/toolset.rs`, `TOOLSET_TOOLS`):

```text
list_files   "See what the workspace holds. This takes in the field as given
              material (P1); what comes back reads as discovery (P1')."
read_file    "Read one file's content into the work as material and evidence
              (P1). What comes back reads as the given (P1')."
write_file   "Transform the ground: create or change one file (P2). What comes
              back reads as the effect made (P2')."
run_tests    "Evaluate the built form against the whole: run the workspace test
              suite (P4). What comes back reads as whole-relative evaluation
              (P4')."
situate      "The return reading: state where the work now stands. Positions:
              P0 frame, P1 material, P2 effect, P3 form, P4 evaluation, P5
              determination; a prime marks the return reading of a position
              (P1'). Args: {"position": "P2"}."
close        "The return condition: when the bounded request is realised,
              close the work at determination (P5); what passes reads as the
              return (P5'). Args: {"synthesis": string} — what is actually
              realised, in plain text. The workspace checks run on close."
```

The classic capability set is the four names only
(`execution.rs` `CAPABILITIES`) with no office law. The jev office criteria
(classification thread, `OFFICE_QUESTION`): P0 initiating intent and operative
frame · P1 material, evidence and givens · P2 effect, operation and
transformation · P3 form, pattern and implementation · P4 whole-relative
evaluation, context and adequacy · P5 candidate determination and synthesis.

## Work in scope this session

1. **Route one as a condition: `ql-tagged`.** The four world tools carry the
   same office descriptions as the toolset; the loop ends like classic (bare
   content may end the work; no situate, no close); the office ledger still
   records the trail from what the tools do. Questions it answers: do the tags
   alone change anything against classic (arm E), and what exactly does the
   return loop add or cost (route one vs route two).
2. **Arm E — `ql-tagged` across the corpus.** Expectation to test, not
   assume: classic's economy with a ledger record classic lacks.
3. **`QL_WORKSPACE_OVERLAY` knob** (route-two/experimentation enabler):
   overlay `starting_workspace` entries at `Task` construction, so `start()`,
   `setup()`, `verify()` and the revision digest all see the variant and every
   frozen check stays internally consistent. Task material is compile-bound
   (`tasks.rs` `include_str!`), so this is the smallest honest seam; node-level
   overlay is not a path (the binary pins the world against the embedded
   start).
4. **Arm F — the propagation probe.** The owner's actual question: does QL
   bake in through surrounding material, not just the toolset — "QL in and
   amongst other things" rather than "QL explained". Run: classic condition
   (no tool law anywhere) + office-language task material via the overlay.
   The material speaks with the offices in its working voice; it does not
   explain a framework (that was arm B, and it changed nothing). Read out:
   post-tagged trail shape (P1-before-P2 discipline, evaluation before
   ending) against the classic control (round 5) and arm A (law in tools).
5. **Jev refinement** (classification thread, branch
   `agent/jev-mef-lenses-2026-09-18`): the settling rule — a return lands
   where it settles, not as the kind of act that produced it; successful
   read/list returns land P1. Rerun the ~105-act sample; report the delta
   including overcorrection. Landed this session before this doc was
   committed: toolset rows 11/30 → 30/30, writes held at P2 2/2, classic
   rows 46/75 → 63/75; honest caveats — run_tests returns unsampled (P4
   boundary unverified), landing-P5 classic rows now all read P1 (2
   previously-correct P5 answers lost), 14 duplicate rows make the 84%
   directional rather than a stable point estimate. Commit 35c9276.
6. **Situate finding** (measured this session from the round-9 records):
   zero situate calls in 22/22 valid trials across all four arms — the return
   reading happened through the tool offices themselves. Recorded for the
   owner's review; situate's place in the 4+2 form is an owner question, not
   settled by this session.

## Protocol (unchanged)

One experiment tree (`~/.cache/actuation/acceptance`); no new worktrees.
Runs by the recipe with a fresh `BASE` per arm; outputs land beside the tree
in cache, arms land in the repo under `runs/<date-glm-roundN-...>/` with a
plain `READING.md`. Disk checked before builds (the volume filled twice on
2026-09-17). Spine commits on `techne/deep-conjugate-allowance`; the jev lane
commits on its own branch in the single tree. Nothing pushed, nothing merged —
branch, pushes and the human-review pass over all rounds are the owner's.
Determination stays `pending-human-review` throughout.
