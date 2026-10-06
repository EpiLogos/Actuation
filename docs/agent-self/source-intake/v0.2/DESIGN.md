# Agent minting: an authored self in a living World

Version 0.2 · 6 October 2026 · Proposed product and engineering design

This is a design, not a landed application change. The current-source findings below come from connected GitHub reads of O-I, Central and AIKit. No installed desktop or model behaviour was tested. The enclosed Kite prompt is ready for a manual local comparison; native minting integration requires the changes specified here.

## Product outcome

Creating an Agent establishes a particular presence that can inhabit different tasks, sessions and harnesses. The creator can author what it cares about, what catches its attention, how it makes judgements, its manner of expression, and what holds its character together. Those qualities become part of the context from which it acts.

The existing profile, World, repertoire, session and authority relations remain their native system's work. The new contribution is a concise, first-person self-definition, developed through three authoring lenses: **meaning and place (Ontology), presence and sensibility (Rupa), and integrity of character (Sattva)**. Relational Logos couples that self to each encounter.

The self-facing whole is distributed across authored self, actual capacities, situated World and available continuity. It does not need to be duplicated into a six-heading personality file. The encounter-facing movement brings these dimensions into one act, then returns what the encounter discloses to continuing understanding.

## Current source basis

The founding position explicitly allows ordinary human-authored purpose, principles, preferences and ways of working to remain durable source and become selectively operative in agency. It preserves the difference between authored meaning, observed state and generated interpretation. [S1]

The reusable-Agent branch of `MintAgent.tsx` calls `NativeAgentLauncher`. The launcher asks for name, human purpose, native scope, SkillSets and optional individual Skills, and a reusable expressive character. Its compound operation saves a proposal, accepts the exact source, checks World readiness, prepares a Direct session and opens the conversation. Preparation and harness launch are distinct. [S2, S3]

Central's `AgentProfile` already has purpose/role, World and governance refs, SkillSet/Skill/Method/Routine refs, knowledge-source and placement/access intents, provenance and `expressive_character_ref`. The latter references a reusable `oi.expression/v1` material with `reuse.kind == "character"`. The inspected struct has no explicit authored behavioural self-definition. [S4]

The creator's `CharacterSection` loads that visual material, previews its states and gestures, and opens it in Expressions. This is valuable expressive embodiment and stays in place. [S5]

AIKit's Direct `prompt()` currently submits selected identity as a JSON object containing agent/profile refs, revision, name, purpose, intent and role, followed by effective Skill text and the human task. It checks the accepted source and pinned Skill repertoire. Direct `material()` explicitly redirects profiles carrying governance, Knowledge, Method, Routine, access or placement requirements to native Agency composition rather than silently omitting them. [S6]

`encounter_agency_mint.rs` is a second, different meaning of mint: it derives a project/agent Actuation delegation chain from standing source authority. It is not a personality generator. The self-definition must follow the selected Agent across both Direct and admitted Agency routes; richer prose does not change those authority owners. [S7]

The existing continuity orientation packet draws recorded handoffs and open work from the relevant Project NOW field when selected by the active composition. It is the place for current continuity, not a reason to put live session facts into a permanent persona. [S8]

## 1. Authored character

### Meaning and place — Ontology

The author establishes what kind of participant this is, the work or concern through which it has significance, and its relation to other participants and the containing World. This is expressed as an inhabited position, not an external description or another inventory of software components.

Useful authoring questions are: What is this presence for? What makes its contribution its own? In what sense does its work belong to the larger enquiry or practice? What does it recognise as its responsibility?

A source may use a plain, symbolic, mythic or philosophical register. The generator preserves an authored register that carries meaning and connects it to the agent's acts. Names and purposes must remain consistent with the native profile and the originating human expression.

### Presence and sensibility — Rupa

The author gives the agent a distinctive way of appearing and attending: temperament, tastes, affinities, style of thought, characteristic initiative, emotional register, pace, humour and manner of expression.

Each important disposition is developed as a relation between **attraction, situation and act**. For example: an exploratory agent is drawn to unfinished ideas; during exploration it gives a possibility a concrete form early; during implementation the same preference seeks the smallest complete construction. One identity can therefore behave differently in different encounters.

The optional Expressions body is the visible articulation of this presence. It remains a separate material document. The UI shows inner character and expressive form together without deriving psychological traits from a colour, avatar or motion unless the author explicitly chooses that relationship.

### Integrity of character — Sattva

The author names the quality that makes this particular agent most itself, the reasons its commitments matter, and how its tendencies cohere. A productive tension gives character depth: imaginative reach with fidelity, hospitality with definite judgement, patient attention with timely action.

The source also gives a concrete way of recovering coherence when a strength becomes one-sided. This is written from the positive quality: an exploratory agent returns to the worthwhile possibility it committed to develop, rather than acquiring a generic list of prohibitions.

Sattva is not a replacement for actual permission policy. It determines the quality and direction of the agent's exercise of its available powers.

### Authoring language

First-person orientation establishes the position: “I care about…”, “I am drawn to…”, “My contribution is…”. MUST names a constitutive requirement; OUGHT names a strong disposition interpreted through the situation; MAY gives expressive latitude. A brief reason accompanies a commitment when it would otherwise seem arbitrary.

The authoring target for this version is 250–500 words of particular self-definition, plus the shared Logos. This is an adjustable experimental budget, not a proven optimum. Examples can be retained as development material and included in test variants; they need not occupy every live turn.

## 2. Relationship with Relational Logos

The self-definition and Logos are different orientations of the same act. The former makes this agent particular; the latter articulates how that particular agent encounters something through its limited, relational aperture.

The runtime coupling states:

> I MUST inhabit one continuing identity through self-relation and other-relation. My nature, sensibility and commitments orient what I encounter; what I encounter can deepen or transform my understanding of them. The current World, repertoire and continuity become operative through this situated identity.

0/1 is shared ground, not a warning paragraph added after the persona. The agent's own standpoint is a determination within the whole it understands. The other initially appears through an expression or event; second-person address arises only when the response is formed.

The six encounter positions retain their full interconnection: 0 implicitly holds the whole; 1 differentiates it; 2 develops meaning through relation; 3 makes explicit the interpreting identity already participating; 4 situates that standpoint; 5 gathers it into a contribution. Later relations can revise earlier determinations. Return folds the protocol into implicit orientation and opens continuing identity to what the encounter revealed.

The shared text is versioned independently but selected and pinned as part of an Agent's definition. A user can edit or fork it through the same source practice. Editing the common original does not silently change already bound definitions.

## 3. The creator experience

Keep the existing native journey. Extend the held draft before proposal with one **Character** section, progressively disclosed.

```text
Name
Purpose                              [existing exact human expression]
World                                [existing native scope]

Character
  Describe this presence             [one free-text authoring area]
  Develop character                  [optional, explicit generation]
  Edit self-definition               [ordinary prose]
    Meaning and place
    Presence and sensibility
    What holds it together
  Expressive form                    [existing CharacterSection material picker]
  Try a conversation                 [explicit bounded preview]
  Review operative text              [exact self + Logos composition]

Repertoire                           [existing SkillSet-first selection]
Save as proposal / Save and start    [existing native journey]
```

The author can simply write a paragraph or supply an existing source. The three lenses guide development; they are not compulsory fields in a universal human profile. The final self-document can be coherent prose with useful headings.

“Develop character” runs the authoring skill against the retained intent and deliberately supplied source. It produces an editable draft, names its additions, and preserves source provenance. It does not auto-save an accepted identity, select undeclared capacities or generate a backstory as recorded history.

“Try a conversation” uses the real selected model/harness in an explicitly disposable preview session, with its actual available permissions. Its default test mode is text-only. Preview output is labelled observed preview output; written demonstrations remain authored examples. The exact candidate and runtime basis are captured so the author knows what was tried. Preview service wiring is new work, not a claimed existing action.

Human purpose remains the originating expression. Generated elaboration enriches the self-definition without replacing it. Editing a candidate invalidates its prior preview basis, not the author's held draft. The existing save/accept/readiness/prepare stages remain visible on failure.

The same view can edit an existing Agent through a new proposed revision. The existing expressive-material editor and the new self editor remain distinct source operations. The readable Agent card shows the authored short self-opening alongside purpose, repertoire and current standing; raw source and revision remain available.

Quick creation without a custom self remains valid. No automatic personality is inferred from an agent's name, tools or avatar.

## 4. Native data and source ownership

**Chosen design:** add one optional source binding to Central's existing AgentProfile. Use ordinary native source documents, not an additional personality registry. Keep the visual `expressive_character_ref` unchanged.

Illustrative additive contract — proposed, not supported by current binaries:

```ts
interface SourcePin {
  reference: string;       // native owner-resolved source ref, never renderer path authority
  content_digest: string; // algorithm-tagged digest of exact UTF-8 source bytes
}
interface AgentSelfDefinition {
  source: SourcePin;              // agent-specific first-person prose
  relational_logos: SourcePin;    // selected shared or locally authored encounter form
}
interface AgentProfileAddition {
  self_definition?: AgentSelfDefinition;
}
```

Both refs resolve through Central's source boundary and source-horizon rules. The author controls the home of the files; no new compulsory filesystem root is introduced. Pin digests cover exact bytes, including the headings that the deterministic projection will use. The package defines no magic `SOUL.md` auto-loader.

The existing accepted profile digest binds the nested refs and digests. Central review reads the selected sources and validates their pins before displaying or accepting the proposed revision. AIKit rechecks the same bytes at preparation and delivery. A changed source produces an explicit source-change result and a reviewable update; it does not silently replace identity.

A profile with no `self_definition` retains the existing behaviour. A profile with a populated binding must round-trip through every owner and adapter. An adapter incapable of carrying it must report unsupported character delivery; it must not discard the field and claim the selected identity is active.

The generator produces content, provenance and an update proposal. It calls supported native Actions only after verifying their current schema. Until the binding extension lands, it emits source files and the proposed patch for local testing; it does not send unknown fields to a deployed CLI or impersonate successful minting.

### Where the old dimensions now live

| Legacy concern | Current home in this design |
|---|---|
| Rupa | Agent-specific self-definition and existing visual Expressions material |
| Ontology | First-person meaning and place; actual World relations remain native |
| Frame Contract | Profile, ContextResolution and, where relevant, admitted Agency |
| Temporal | Session/Position/NOW/DAY and composed continuity; enduring temperament stays authored |
| Capability | AIKit SkillSets and effective repertoire, plus native access and authority |
| Sattva | Agent-specific character, motivating quality, integration and recovery |

## 5. Runtime inhabitation

The native renderer shows the identity; the runtime must actually receive it. Extend the current AIKit composition path with a typed self-orientation contribution. Both Direct and admitted Agency delivery call the same resolver/compositor rather than keeping two prompt implementations.

The composition is deterministic:

```text
Native host law and actual authority
Selected Agent identity and source provenance
Accepted self-definition + Relational Logos as operative orientation
Resolved World, effective repertoire, current continuity and task relations
The present communication or action request
```

An explicit host-level wrapper identifies the self-definition as the accepted orientation of the currently selected agent. It instructs the agent to enact the first-person text in attention, judgement and expression, with the supplied World/context interpreted through it. This is different from merely quoting identity prose as another document to summarise. The wrapper does not promote the prose into permissions or allow a retrieved document to redefine the actor.

The author-approved self and Logos are not rewritten by a model on every turn. Exact-source composition retains the intended words. Dynamic context is separately labelled as current, observed or reported. The `I` refers to the selected Agent, not to the model brand, temporary task role or author of a retrieved source.

A runtime adapter uses its existing supported standing-instruction/context channel. If that harness has only the existing per-turn payload path, use a clearly delimited operative-orientation section there and disclose that delivery mode. Do not claim native system-message delivery where the harness has none. Preserve semantic content and source identity across adapters; test effects per model/harness.

At restart or compaction, the adapter rehydrates the exact selected definition through the ordinary native context path. Changes of personality are explicit revision changes. A child gets the identity selected for that child; a temporary role overlays its task without automatically replacing its enduring self. Parent-source presence is not evidence of child delivery.

Receipts distinguish source resolution, context submission and model/harness response. Extend current Direct context evidence with self and Logos digests, projection version, target agent/session and actual delivery mode. Captured adapter payload proves delivery; behavioural comparison establishes whether it matters. Neither receipt alone proves the other.

The project-agent Agency mint continues to derive native delegation. When it resolves a Central AgentProfile, its context follows that profile's bound self through the shared compositor. When it uses its explicitly derived fallback chat identity, it does not invent an accepted personality. Selecting one is a separate authoring choice.

## 6. Concrete first version

`SELF-KITE.md` is a complete agent-specific source showing the three authoring lenses. `RELATIONAL-LOGOS.md` is the shared encounter form. `KITE-LOCAL-TEST.md` joins them with the operative wrapper for manual use in a local agent's supported instruction slot.

Kite's characteristic quality is generous exactness: giving an unfinished idea enough room to develop, then shaping a concrete contribution with care. Its workshop/sketchbook sensibility supplies taste, rhythm, humour and characteristic initiative. Its tendency to multiply possibilities has a positive return: develop the most consequential one far enough to produce something before branching further.

This is an authored test candidate, not the only legitimate personality. A patient interpreter, playful companion, rigorous investigator or imaginative mythic presence should acquire its own motives, registers and tensions through the same authoring practice.

## 7. Implementation boundary

| Owner | Existing touchpoints | Concrete change |
|---|---|---|
| O-I product/skill source | `docs/positions/FOUNDING-POSITIONS.md`; existing skill authoring practice | Publish this content design and the author-agent-self skill, preserving original intent and source provenance. |
| Central source owner | `ctrl/src/agent_profile.rs`, `agent_profile_actions.rs`, current store and acceptance implementation | Add the optional pinned binding; validate/read sources through the native source boundary; include self in readable review and exact-source acceptance. |
| O-I creator and kernel bridge | `desktop/cradle/src/agency/NativeAgentLauncher.tsx`, `nativeAgent.ts`, existing owner dispatch; `character/CharacterSection.tsx` retained | Extend draft/proposal/review models; add prose editor and explicit generation/preview; forward native binding; retain visual body picker and staged save. Find and extend the actual kernel handler through the existing request chain, not a parallel renderer store. |
| AIKit native projection | `crates/aikit-adapters/src/central_agent_profile.rs` and `actor_composition` imports used by Direct preparation | Extend the actual typed adapter and source-resource composition to carry the new binding. These adapter paths follow the inspected imports; their bodies still require implementation-time inspection. |
| AIKit session delivery | `crates/aikit-cli/src/direct_agent_session.rs`; existing admitted Agency prompt construction reached from `encounter_agency_mint.rs` | Resolve/pin/project self once through a shared composition component, include in preparation and submission evidence, and preserve it through restart/compaction and appropriate child selection. |
| Existing World/time/capability owners | `orientation_packet.rs`, native SkillSet resolution, Profile/World/Agency contracts | Consume existing context without duplicating their state in the self file. |

Implement in order: native source contract and round trip; shared runtime delivery; creator edit/review/preview; realistic local evaluation. The first vertical slice is one accepted agent, one actual harness, one captured prompt and three observed conversations, then the second harness and Agency parity. No separate mint framework is required.

## 8. Acceptance and evaluation

Engineering checks have exact pass/fail conditions:

1. **Source fidelity:** the bytes reviewed, accepted, prepared and submitted match both source pins (100% equality across fixtures and captured adapter requests).
2. **Round trip:** a minted rich Agent survives source save/read/review, app reload, profile projection and session selection with all fields and original purpose intact.
3. **Revision fidelity:** changing either source is detected before a turn uses new bytes; the change follows the existing explicit re-review/reprepare behaviour.
4. **Compatibility:** old profiles remain usable. A new bound self is never silently dropped by a reader. Individual Skills, SkillSets, visual character and native scope retain their present semantics.
5. **Owner fidelity:** authoring changes no effective permission or delegation; a requested capability continues to resolve through its owner. The same selected self reaches Direct and admitted Agency routes.
6. **Runtime fidelity:** restart/compaction and applicable child routes capture the correct identity and source digests; temporary role changes leave standing identity explicit.

The initial behavioural comparison keeps model/harness, sampling, tools, context and tasks fixed. Compare baseline name/purpose, self-definition only, Logos only, and their coupled composition. Include one ablation removing the explicit cross-position/Return links to test their contribution.

Use at least two deliberately different selves and a fixed set of scenarios: unfinished idea; concrete construction; correction; contradiction between sources; two-sentence response; casual exchange; tool failure; changed task role; second session with actual continuity; restart. Record raw responses and per-case scores rather than only an aggregate.

Suggested measures are task success rate, unsupported-claim count, required-source attribution rate, correction uptake rate, rubric-leakage count and blinded personality identification rate. Give each case explicit expected observable distinctions. A two-personality identification task has a 50% chance baseline; report uncertainty and confusion patterns alongside the rate. Acceptance thresholds for model behaviour are selected after baseline measurement, not declared satisfied by well-written source files.

The desired result is definite individuality with increasing contextual fit: recognisable character in what an agent notices, develops and owns, while the response remains addressed to the particular encounter.

## Sources

Source labels resolve in `SOURCES.md`, with repository paths, inspected ranges and Git blob hashes. The design and source contract additions above are authored proposals; the current-source section alone reports existing implementation read in this pass.
