# E-MT-3 — World formation over the loop's own trace

JEPA/EB-JEPA attacks the constitution of a predictive world in representation
space: predict abstract state, not sensory detail, and ask what invariants must
survive transformation for two moments to belong to the same world. This
experiment points that question at our own agent recurrence: the actuation loop's
event traces are the environment.

## Data (declared honestly: currently toy volume)

- Retained Series-1 run records:
  `experiments/ql-runtime/comparison/series1/runs/2026-09-13-glm/*.json` and
  `2026-09-13-smoke-restraint/` — each record carries
  `records[].record.events`, the QL semantic event stream
  (`ql-agent/0.1` envelope: event type, face, ql position, payload, witness), plus
  execution vs semantic status.
- Prime recursive runs: `experiments/ql-runtime/prime/runs/2026-09-13/*.json`
  (conditions P0–P5 of the prime harness).
- Fifteen-odd retained runs is not a training corpus. The first real gate on this
  experiment is trace volume, which E-MT-4's composed loop runs would produce as a
  by-product. No JEPA result should be claimed from the current trace set.

## Method

1. **Trace → sequence**: export each run's event stream as a sequence of
   (event-type, ql position, carrier, capability-or-model marker, outcome) tokens.
   The exporter from run-record JSON is the first implementation task; it is the
   same exporter E-MT-2's decision-context replay needs.
2. **Latents**: embed event tokens into a small latent space; the world-formation
   question becomes: which latent structure survives transitions (act → project →
   absorb → interpret → transition) across runs and tasks?
3. **Predictive JEPA**: predict the next event's latent from the history latent
   plus the action (the capability call), in the EB-JEPA pattern
   (representation-level prediction, energy-based compatibility for candidate
   continuations) rather than token reconstruction.
4. **Readouts**:
   - next-event prediction vs an n-gram baseline on held-out runs (a JEPA that
     cannot beat n-grams on our own traces has not found a world there);
   - energy scoring of candidate continuations — does the learned field rank the
     events real runs actually took below corrupted continuations (same
     corruption-operator discipline as `../ebm/`);
   - later, action-conditioned planning: choose the next capability call by
     planning over latents, then check against what live runs show — a
     capability-effect claim, which by loop-programme law requires live runs and
     human review, never fixture evidence.

## Reference implementation

[facebookresearch/eb_jepa](https://github.com/facebookresearch/eb_jepa)
(Apache-2.0; paper [arXiv:2602.03604](https://arxiv.org/abs/2602.03604)) —
`examples/ac_video_jepa` (action-conditioned world model + planning, reported 97%
planning success on its Two Rooms environment) is the pattern to imitate,
transplanted from grid-navigation events to loop events.

Bootstrap: `uv sync` in a checkout; Python 3.12. Compute honesty: shipped configs
target a single CUDA H100; this Mac's MPS backend and the Omarchy host's NVIDIA
status are **unverified** — verify before any training claim. The Two Rooms-scale
models are small, so CPU/MPS feasibility is plausible but unevidenced.
