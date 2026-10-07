# Publication check — 7 October 2026

The source-intake files were compared to the supplied package by Git blob SHA and byte length after remote creation. All eight match, including DESIGN.md (046031ac6a5433064142626b07c2a252b26f5f3e), KITE-LOCAL-TEST.md (c0e5c5cf6c52d5135b1b3309419910c2afb4648c), MANIFEST.json (63d50a47515814aaf936fbdf866d44f5b3f0f285), README.md (1d2beb9dac5449dfdca2cf07d7f6f05e61791221), RELATIONAL-LOGOS.md (83a5961d98e8aab4d130923f0f2d40d4e9375ccd), SELF-KITE.md (3df48106c6fd6e56d0d6be5ab0135f181021f842), SKILL.md (3b3f98e7ccb0bf0f374d9e20a9489c0c0df53d41), SOURCES.md (e73576450b981158252a1a0300d452f2e23e340f).

Executed against the local publication copy from experiments/native-research/relational-personalities:

- python3 check.py — exit 0; original source hashes and four source pins/plan consistency passed.
- python3 -m unittest discover -s tests -v — exit 0; 13 tests passed. These exercise ratios/null denominators, four identities, explicit event/sender/reply relations, duplicate and uncited evidence rejection, and nonfinite usage rejection.

The tests are synthetic unit cases for analysis arithmetic, not four native Agents or model results. No runtime integration, installation, authenticated Agent mint, Factory Run, NOW allocation or live conversation was executed in this publication. Those are the local commission in LOCAL-EXECUTION.md.
