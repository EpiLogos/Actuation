# Authored Agent self: native integration specification

Version 0.3 | 7 October 2026 | Owner-commissioned work; this specification is a proposed implementation contract. Runtime and behavioural proof remain to be produced locally.

## Destination and preserved meaning

A person authors a particular Agent, meets that Agent through the application's ordinary creation and conversation surfaces, and can recognise the same presence across encounters and restarts. Its meaning and place, presence and sensibility (Rupa), and integrity of character (Sattva) become operative in what it notices, judges, expresses and learns. Relational Logos makes this self's encounter with the other a differentiated, whole-preserving act.

The self-facing whole includes authored orientation, actual World, repertoire, authority and continuity. Develop the present Agent-expression system; the old Rupa/Ontology/Frame Contract/Temporal/Capability/Sattva format is provenance, not a six-field schema to restore. The full six-position Logos and its interconnected 5-to-0 Return remain intact. Character is not reduced to an avatar, adjectives, job role or repeated first-person wording.

The concrete experiment uses exactly four peer Agent identities: Kite, Cairn, Lilt and Flint. They meet to become acquainted, not to perform a scripted debate, coding assignment, benchmark interview or division of labour. Factory governs the development and evidence; it does not dictate their conversational content.

## Authority and source map

Begin from O-I `docs/positions/FOUNDING-POSITIONS.md` (or its canonical successor), Actuation `docs/ACTUATION-CONSTITUTION.md`, and the retained [v0.2 design](source-intake/v0.2/DESIGN.md). The original eight files and hashes remain unchanged in `source-intake/v0.2`.

Current repository readings used for this plan:

| Owner | Source | What it establishes |
|---|---|---|
| Actuation | `docs/ACTUATION-CONSTITUTION.md`, blob `2c500560e8216184adf1ee47a394218282bb2f43` | enduring Agent, situated Agency, independent participants, bounded composition and attributable Return; repository is research home, not compulsory agent residence |
| O-I | `desktop/cradle/src/agency/NativeAgentLauncher.tsx`, blob `6d15fa1abd5dca1c927a97840bd1826427231f50`; `nativeAgent.ts`, blob `c88bfbdfdb1bb5e80c8509f41d29ac91e4904ecb` | native creator, held draft, SkillSet-first selection, visual character and staged source acceptance/preparation |
| O-I | `desktop/cradle/kernel/src/agent_definition.rs`, blob `ce8bcecb050ec9518de8f2655ca199f346ee6941` | Propose forwards purpose as intent to Central express; review/accept/prepare use exact native bases |
| Central | `ctrl/src/agent_profile.rs`, blob `ce79192877134118a273e5c43ede3bf3ce5ff318`; `agent_profile_actions.rs`, blob `0c57944e3923deecd93bbb10e2793a0752dfd115` | ref/intents profile, separate expressive character, express allocates profile/Agent proposal refs, explicit forwarding allowlist |
| AIKit | `docs/PRAXIS-ARCHITECTURE.md`, blob `8273fa1151c749192c1dc995257be15641b7ad60` | current sixfold assembled Agent reading; Intent, Skill, Method, Methodology, SkillSet, World and Return; carried is not loaded |
| Central | `docs/CAW-NATIVE-CONSUMER-INTERFACES.md` | native work policy, Workcell-root NOW, child allocation, revisions, source/destination, ordinary returns and receiving |
| Central | `docs/PROJECTCENTRAL-NOW.md`, blob `47e9c6e8b1d1ef4f4e81a9149c81357b209cbc82` | session-independent temporal ground and attributed returns; no synthetic Day lifecycle |
| Factory | `skills/factory-development/SKILL.md`, blob `85e83dc89a88de476b8bdbe8789f2b9fa200ccd0` | RunMap is Wayfinder, verification obligations, native NOW/prepared participant contexts, independent evidence and Recognition |

Repository base for publication: Actuation main `8a882c9851ffb9d85304714ee63d4ec3d0340a5b`. The v0.2 source ledger supplies the earlier AIKit Direct/Agency/continuity findings. These are investigation anchors, not claims about installed binaries. Resolve actual local checkout, source, build, installed and running versions before implementation. A changed path is followed to its owner successor; a stale observation is corrected explicitly.

## 1. Source contract and lifecycle — Central

Extend the existing AgentProfile with one optional self-definition binding, reusing native source-reference/revision/digest types wherever equivalent types exist:

```text
self_definition?
  source:             { reference, content_digest }
  relational_logos:   { reference, content_digest }
```

Both pins are exact UTF-8 source bytes with algorithm-tagged digests. Keep `expressive_character_ref` as the visual Expressions body; source temperament is distinct from visual form. Skills, methods, routines, governance, placement and authority keep their existing owners. The author chooses an ordinary source home; there is no global personality registry or required runtime filename.

The binding MUST survive `agent-profile.express`, `propose`, `save`, `read`, `review`, acceptance, roster, projected profile and session selection. Update Central's explicit express forwarding allowlist as well as its typed struct and schema. Derive native Agent/Profile refs through express; seed names and paths are not semantic identity. A retry after an uncertain express result first resolves the original outcome through available owner evidence. Add idempotent owner support when missing; never mint replacements merely because a response timed out.

Review MUST show the complete particular self and Logos at their exact source basis. Acceptance covers the profile binding and both pinned sources through Central's current authenticated acceptance path. Generated authorship remains generated even when accepted. Neither an experiment plan nor an authoring skill fabricates human Recognition or privilege. This commission authorises work and these four proposed test personalities; use existing native authority, or produce the one actual review request when a protected action requires it.

At acceptance, preparation and each subsequent context submission, resolve the pinned source under current source-readability rules. Source resolution and submitted bytes MUST be the same immutable retained bytes, avoiding check-then-reread races. A missing/withheld source, digest mismatch, stale acceptance or unsupported populated binding is a named non-success before inference. An explicit new profile revision and re-acceptance adopts changed source. Existing sessions retain their exact basis or pause for re-preparation; they never silently take the latest file. Ordinary profiles without a self remain usable.

Limits follow the current native source boundary and are declared to the author. A word-budget warning never silently truncates accepted source. Known conflicting declarations of name/purpose become reviewable differences. A source file cannot acquire tool or execution grants by containing imperatives.

## 2. Operative context — AIKit

Create or extend ONE typed identity-source resolver/compositor used by both Direct Agent sessions and admitted Agency composition. Inspect the real `central_agent_profile` and `actor_composition` adapters and current successors of `direct_agent_session.rs`, `encounter_agency_mint.rs` and `orientation_packet.rs`. These are owner entry points, not permission to duplicate their implementations in Actuation.

The deterministic composition carries:

```text
native host law and actual authority
selected Agent identity and accepted source basis
particular self + selected Logos, marked as operative first-person orientation
resolved World / repertoire / participant-specific continuity
present encounter and native messages with their real speakers
```

The `I` denotes the selected enduring Agent. It is neither a model brand nor a temporary role nor the author of a retrieved passage. Preserve the particular self's and Logos's exact wording. Host adaptation uses the harness's actual supported standing-instruction/context channel. A turn-payload-only adapter declares that delivery mode; it cannot claim system-message loading. Context receipt fields extend the existing native evidence object with Agent, profile/acceptance revision, self/Logos pins, compositor version, actual delivery mode, target session and submission outcome.

The self is not smuggled into a Knowledge source or a Skill body solely to get it loaded. Direct's current richer-context guard must gain explicit supported identity-source composition, not an exception that drops unrelated requirements. SkillSet membership, selected source, submission and observed behaviour remain distinct facts.

Restart, compaction and rematerialisation rehydrate the same selected identity through this component. Children receive their own selected self; parent presence never counts as child delivery. A child inheriting the same Agent versus a different Agent remains an explicit native relation. Role overlays do not silently rewrite the standing self.

## 3. Creator, native expression and authoring practice — O-I / Actuation

Extend the current `MintAgent -> NativeAgentLauncher -> NativeAgentController -> agent_definition` path. The Character area provides: free prose/source selection; Develop character; edit complete self; retain current expressive-material picker; real disposable Try a conversation; inspect exact operative text. Existing name/purpose/scope/repertoire selection and Save as proposal / Save and start stay one native journey.

The same native expression operation is callable by an authorised agent using the CLI/Action route. The four peers MUST be created there, not as hand-authored fake registry records or one model impersonating four speakers. Bind exact source pins before review. Temporary preview sessions do not count as the final four recognised definitions.

`skills/author-agent-self/SKILL.md` is the Actuation-owned authoring practice. Register it through the current native Skill source/capsule mechanism, its existing conformance declarations and the appropriate repertoire. It is a procedure invoked by the authoring/implementation agent; it is not automatically loaded into every peer. Its shared Logos reference is a packaged source asset, not a second governance store. Record public source/revision and materialisation receipts. Prove actual invocation and resulting native proposal rather than only discovery.

In the creator, one meaningful edit invalidates the prior preview basis. Unknown save/accept/prepare outcomes retain the draft and original correlation. Previews, observed native launch and staged failures are visible through existing components; no decorative success state substitutes for the owner response. Test actual browser/native controls, including no-self compatibility and visual-character round trip.

## 4. Factory and NOW — one undertaking, different apertures

Use the installed root/Project guardian, existing Factory development Methodology, Wayfinder and documentation Methodology. Publish this undertaking as a native commissioned Run with a RunMap. The logical units below map to native refs, not a second task database:

| Unit | Work / native owner | Depends on | Closure evidence |
|---|---|---|---|
| I0 | inspect current work, basis, native contracts and existing related Run | none | exact source/build/install/running cut, real Run/NOW and owner map |
| I1 | source binding and exact review/acceptance / Central | I0 | save/read/express/CAS/revocation and race tests |
| I2 | shared identity compositor and Direct/Agency delivery / AIKit | I1 | captured real adapter inputs, unsupported/stale tests and restart |
| I3 | native mint editor, generation, preview and operative review / O-I | I1 | actual mint UI walk plus Action parity |
| I4 | authoring Skill and persona intake / Actuation + AIKit registration | I0 | invoked native authoring route and source provenance |
| I5 | independent joined proof and four-peer native admission | I2,I3,I4 | all engineering E obligations in experiment protocol |
| I6 | free acquaintance and continuity encounter | I5 | four native sessions, original messages, source/load/delivery receipts |
| I7 | independent analysis, Return, release and human reading | I6 | metrics with denominators, raw evidence refs, no orphan processes |

One integrator owns the whole. Subagents can work on disjoint owner files; an independent verifier does not certify its own implementation. Implementation/verification agents are not experimental participants.

Recover the actual Workcell-root NOW, then allocate a bounded undertaking child and participant children using native policy/revision checks before starting work. The four peers receive four distinct child NOWs, actual Agency/AgentSession/placement refs and one shared encounter relation. Maintain a separate assessor aperture. Use each allocation's returned writable destination and source refs; never infer T paths from names/hashes. Scope participants to their own admitted self, their own continuity and the shared encounter. Evaluator instructions, peer private source files and implementation evidence MUST be withheld, not merely left unmentioned. If current source/material policy cannot enforce this, implement the missing native join before claiming an uncontaminated trial.

The meeting's existing ActuationStream/Communique/AIKit conversation is the message owner; a shared Flow is a human-readable projection over those refs. Factory RunMap is the developmental plan, Central NOW is temporal continuity, Actuation retains Agency/Return semantics, AIKit runs/discloses the sessions, Workcell owns their material place. An experiment manifest and analysis JSON are derived test artifacts, not new canonical stores for these identities.

## 5. Integration verification (engineering, not personality scores)

Implement and independently run all E1-E10 in the experiment protocol. Specifically: test missing/different self source; changed Logos; withheld source; forged/stale acceptance; source edit between read and submit; unsupported old adapter; Direct versus Agency parity; no-self backward compatibility; child source isolation; restart/compaction and native message de-duplication. Removing the compositor or native message handler must break the appropriate test.

Run native suites at every changed owner and the appropriate existing O-I #65/#220 story obligations. Use current names and evidence grades; add a bounded subcase, not a rival acceptance programme. Keep tests, deployed source, installed binary and real provider observations separately recorded.

## 6. Experimental implementation and success

The complete encounter, roster, budgets, disclosure, metrics and comparative design are in `experiments/native-research/relational-personalities/PROTOCOL.md` and `experiment.json`. Build the experiment adapter as a thin client of current native APIs. It may prepare test inputs, subscribe, schedule bounded opportunities, capture evidence and stop; it may not become a persona loader, message broker, identity allocator or alternate scheduler.

The first release condition is engineering E1-E10 plus the primary four-agent acquaintance and restart encounter with attributable evidence. A surprising, quiet, awkward or weakly differentiated conversation is a valid experimental result, not a reason to script stronger personalities into the transcript. Behavioural superiority is a separate comparative claim requiring controlled cohorts. Baselines and ablations are predeclared but not launched without the Run's real resource budget.

The actual final Return names: source changes and accepted pins; changed owner commits; real mint/acceptance/session/NOW/stream refs; captured delivery mode; original chronological conversation; events that support or challenge the hypothesis; numerical measures with missing opportunities marked; and the exact cleanup/retention state. Source improvement follows review. Do not rewrite a personality mid-episode to manufacture success.
