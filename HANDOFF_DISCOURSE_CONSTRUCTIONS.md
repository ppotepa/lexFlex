# HANDOFF: Discourse-Aware Constructions + Context-Driven Interlingua (Persistent Subjects & Multi-Sentence Coherence)

**Date**: 2026-07-13  
**Context**: lexFlex project – Interlingua + LinguisticGraph + lightweight constructions work  
**Prepared for**: Next agent (any coding agent) continuing this thread  
**Priority**: High – core architectural evolution toward "IL as compilable AST with first-class discourse constructions"

---

## 1. Executive Summary – What Has Been Done So Far

### Recent Session Work (this conversation thread)
- **Adjective + Copula Hardcode Cleanup** (primary recent focus):
  - Removed multiple layers of `dbg.contains("są"/"niebiesk")`, CAT-prefer logic, name-suffix heuristics (`ends_with('e'/'ne'/'y')`), `"cats".to_string()` fallbacks, and manual `adj_form` collection in `src/engines/en/generator.rs`.
  - Generalized parser retain + safe lexicon-driven adj attachment in `src/engines/pl/parser.rs` (any adj concept via `lookup_by_form`/`lookup_concept` + pos=="Adjective", works for "niebieskie"+koty, "czerwone"+domy, etc.).
  - Force clean `Existence { location: None }` routing for copulas.
  - `generate_existence` loc=None path now uses `realize_noun_phrase` (so `.adjectives` are properly realized via concept lookup).
  - Lexicon already had inflected entries (`"niebieskie" → lemma:"niebieski", concept:"BLUE"`, similarly for RED). Generator + realize already did concept → EN lemma ("blue").
  - Result: 
    - "To są niebieskie koty." → "These are blue cats."
    - "To są czerwone domy." → "These are red houses."
    - Singulars and bare cases clean.
  - Tests updated in `tests/integration_test.rs` (stricter asserts for plural adj copula + IL grouped adjectives carrying BLUE/RED concepts).
  - Full `cargo test --workspace` (108+ tests) green before push.
  - Pushed as `fda782d`.

- **Lightweight Constructions** (earlier phases referenced in history):
  - `IdentificationalCopula`, `DemonstrativeNP`, `AgeIdiom`, `Accompaniment` registered in `LinguisticGraph`.
  - Attachment logic in `src/core/deduction.rs` (`apply_graph_inference`): adds `PartOfConstruction` self-edges on relevant Entities for Existence{loc=None} and Statement{BE}.
  - `has_construction(name)` predicate on graph (checks `EdgeKind::PartOfConstruction`).
  - Used in a few places (mainly AgeIdiom for word order in realizer/pipeline; some generator paths).
  - Constructions still **graph metadata only**, not first-class in IL.

- **Discourse / Multi-Sentence Context** (pre-existing + partially exercised):
  - `src/core/interlingua.rs`: `Utterance` has `discourse: Option<Discourse>`. `Discourse` has `entities_in_focus`, `recent_mentions`, `coref_edges`.
  - `src/core/context.rs`: `track_discourse()` (builds recent/focus/coref within Utterance), `link_cross_utterance_context()` (for DialogueGraph), `Corefers`/`ContinuesTopic`/`NextSentence`/`InFocus` edges.
  - `src/core/graph.rs`: `coreference_chain()`, `in_focus_entities()`, `recent_mention_entities()`, `EdgeKind` variants for discourse.
  - Existing tests in `tests/graph_tests.rs`: `test_multi_sentence_corefers_and_next_sentence`, `test_dialogue_continues_topic_edges`, `test_dialogue_graph_cross_utterance`, `test_translate_dialogue`.
  - These mostly verify **graph edges** for repeated explicit names (e.g. "Tomek ma kota. Tomek ma psa."). **Not yet** reliably propagating implicit/zero subjects or shaping IL `Frame`/`Entity` subjects from prior context.

- **General Architecture Health** (from this + prior work):
  - IL = `Utterance` → `Sentence[]` → `Frame[]` (Existence, Statement, Transfer, etc.) + `Entity` (concept, features, `adjectives: Vec<Entity>`, `coordination`).
  - Graph as relational layer (EvokesConcept, PartOfConstruction, Corefers, etc.).
  - Deduction enriches post-parse.
  - Generation = staged (policy + realizer + morphology). Recent cleanups moved toward data-driven paths.
  - Concepts are first-class (data/concepts/concepts.ron + lexicon concept links). Lexicon entries drive adj/noun forms via `lookup_concept`.
  - Many special cases reduced; graph/lexicon emphasized as source of truth.

**Overall**: Significant progress on removing surface hacks and making basic copula + adj cases work via concepts. Discourse infrastructure exists but is not yet **driving IL construction or construction selection** for cross-sentence coherence.

---

## 2. The Goal (User's Explicit Vision)

From the conversation:

> "konstrukcje z INTERLINGUA to byly definiowalne concepty"
> "kazda wypowiedz nawet kilku zdaniowa ma kontekst"
> "pojaw ia sie temat podmioty itp itd, to wszystko powinno pomagac w dedukcji"
> "konstrukcje jezykowe powinny miec szybkie tak jakbysmy drzewo expression troche konstruowali w c#"
> "z interlingua powinno dac sie kompilowac zdania na dany jezyk"
> Concrete requirement: "skoro w pierwszym zdaniu mówiliśmy o Tomku, to w ostatnim, nawet gdy nie wspominamy jego imienia to nadal jest podmiotem tej wypowiedzi."

**Desired Outcome**:
- Multi-sentence `Utterance` with rich **discourse context**.
- Context (topic, focus, prior entities) feeds **deduction** and **IL population**.
- Later sentences can have **implicit/zero/pronominal subjects** that resolve to the same persistent Entity (Tomek) in the semantic `Frame`.
- Linguistic **constructions are first-class definable Concepts** (e.g. `TOPIC_CONTINUATION`, `ZERO_ANAPHORA`, `CONTINUING_AGENT`, `IDENTIFICATION`).
- IL is a **composable tree/AST** (constructions wrap/compose Frames/Entities).
- Generation is explicitly a **compilation** process from this IL (front-end parse/deduce → middle context resolution → back-end target lowering/realization).
- Result: coherent complex discourses, better deduction, fewer hardcodes.

This builds directly on the "lightweight constructions" work and the "graph as source of truth" discipline.

---

## 3. Current Architecture Snapshot (Key Pieces)

### Core Types (`src/core/interlingua.rs`)
- `Utterance { sentences: Vec<Sentence>, discourse: Option<Discourse>, ... }`
- `Sentence { frames: Vec<Frame>, graph: Option<LinguisticGraph>, resolved_refs, ... }`
- `Discourse { entities_in_focus, recent_mentions, coref_edges, speaker/addressee, ... }`
- `Entity { concept: ConceptId, name, features, adjectives: Vec<Entity>, coordination, reference, ... }`
- `Frame` variants (Existence {entity, location}, Statement {subject, property}, etc.)
- No native `Construction` type in IL yet.

### Graph Layer (`src/core/graph.rs`)
- `LinguisticGraph { nodes, edges, constructions: HashMap<String, ConstructionPattern> }`
- `EdgeKind::PartOfConstruction(String)`, `Corefers`, `ContinuesTopic`, `NextSentence`, `InFocus`, `RecentMention`, `EvokesConcept`, etc.
- `has_construction(name)`, `register_default_constructions()`, `coreference_chain()`.
- Constructions currently attached as self-edges on Entities.

### Context & Deduction
- `src/core/context.rs`: `track_discourse()`, `link_cross_utterance_context()`, edge population.
- `src/core/deduction.rs`: `apply_graph_inference` (registers defaults + attaches PartOfConstruction for copula/demonstrative/age).
- Context mostly builds **graph metadata**; limited back-propagation into `Frame` subjects or construction Concepts.

### Parsers / Generation
- PL parser: grouping, retain for copula/THIS, post-fix attach (recently generalized).
- EN generator/pipeline: now leans on `Existence {loc:None}` + `realize_noun_phrase` for copulas.
- Lexicon-driven concept lookup for forms.

### Data
- `data/concepts/concepts.ron` (no discourse/construction Concepts yet).
- Lexicons have adjs/nouns with concepts.

### Tests
- Graph tests cover basic multi-sentence coref/NextSentence/ContinuesTopic (mostly explicit repeats).
- Integration tests cover copula precision + constructions (has + output).

**Gap Summary**: Discourse edges exist. Persistent subject resolution into IL Frames + construction-as-Concept selection does **not** yet happen reliably for implicit cases.

---

## 4. Proposed Architecture (Updated for User's Requirements)

```
Utterance (the "program" / compilation unit)
├── Discourse (first-class context)
│   ├── topic_chain: Vec<EntityId>          // persistent current topic(s)
│   ├── focus_stack / entities_in_focus
│   ├── entity_registry (persistent identities + salience)
│   ├── history_of_constructions: Vec<ConceptId>
│   └── coref / anaphora links
│
└── Sentence[]
    └── ConstructionInstance (now a Concept + tree node)
        ├── construction_concept: ConceptId   // e.g. TOPIC_CONTINUATION, ZERO_ANAPHORA_SUBJECT
        ├── roles / parameters (Entities, features)
        └── inner: Frame or Predication
            └── Entity (can be "linked" or "continuation" reference)
                └── reference: Anaphoric { antecedent: EntityId } | Direct
```

**Flow (Compilation View)**:
1. Parse → partial IL + initial graph.
2. Deduction (context-aware):
   - Track Discourse (update topic, focus, recent).
   - Resolve implicit subjects via `Current Topic` / focus → link or inject Entity into Frame.
   - Select/attach Construction Concepts (as first-class).
   - Materialize rich edges (ContinuesTopic, Corefers, PartOfConstruction on Concepts).
3. LinguisticGraph as **IR** (queryable for "what is the continuing subject?", "which construction applies?").
4. Compilation pipeline:
   - Lower IL tree (apply discourse constructions).
   - Target-specific phases (lexicon by Concept, morphology, linearization guided by construction).
   - Output surface.

**Benefits**:
- "Tomek..." → later "Kupił" has the same Entity as subject in IL.
- Construction choice (e.g. continuing vs shift) driven by context.
- Extensible: new discourse constructions defined in concepts.ron.
- IL remains the single source for "compilation" to any language.

---

## 5. Phased Implementation Plan (Recommended for Next Agent)

### Phase 0: Stabilize & Baseline (1-2 days)
- Run full verification (rtk cargo test, key translates with Tomek-style multi-sentence).
- Capture current behavior for "Tomek poszedł. Kupił mleko." (IL + output + graph edges).
- Read key files (see section 7).
- Add a failing test case that asserts persistent subject (e.g. second sentence Frame has subject concept matching first + linked reference).

### Phase 1: Make Constructions First-Class Concepts
- Add discourse/construction Concepts to `data/concepts/concepts.ron`:
  - `TOPIC_CONTINUATION`, `ZERO_ANAPHORA`, `CONTINUING_AGENT`, `TOPIC_SHIFT`, `IDENTIFICATION` (enhance existing), `DEMONSTRATIVE_REFERENCE`, etc.
  - Give them roles where useful.
- Introduce minimal `Construction` modeling in Interlingua (either new struct or `Frame` variant + `construction: Option<ConceptId>` on Entity/Frame for starters).
- Update deduction to attach via ConceptId (not just string).

### Phase 2: Discourse-Driven Entity Propagation
- Enhance `Discourse` (add `current_topic: Option<EntityId>`, `topic_stack`, better persistence).
- In `track_discourse` + new deduction step:
  - After processing sentence N-1, record salient subject as continuing topic.
  - For sentence N: if implicit subject / missing agent/theme and context has continuing topic → resolve and inject/link the Entity (set concept + reference + id).
- Support zero anaphora / pro-drop cases (Polish "Kupił" should get Tomek subject in IL).
- Use `ContinuesTopic` / `Corefers` more proactively to shape IL, not just annotate graph.

### Phase 3: Construction Selection in Deduction
- In `apply_graph_inference` (or new `resolve_discourse_constructions`):
  - Based on Discourse state (is this a continuation of prior topic? is subject implicit?) → choose and attach Construction Concept.
  - Example: continuing topic + BE without location → `TOPIC_CONTINUATION` + `IDENTIFICATION`.
- Make `has_construction` work with ConceptIds or keep string facade + back it by Concepts.

### Phase 4: IL Tree + Compilation Mindset
- Treat generation as compilation:
  - Rename/refactor entry points toward `compile(utterance, target)`.
  - In pipeline/generator: consult active Construction Concepts + Discourse for decisions (word order, pronoun vs zero, article choice, etc.).
  - Ensure `realize_noun_phrase` / entity form respects linked/continued entities.
- Strengthen `Sentence` / `Frame` to carry explicit continuing subject when context provides it.

### Phase 5: Testing, Examples, Polish
- Add rich test: multi-sentence with zero anaphora ("Tomek poszedł do sklepu. Kupił mleko i wyszedł.").
  - Assert in IL: all Frames have subject Entity linked to same antecedent (or same id).
  - Assert correct EN output with "he" or maintained reference.
  - Assert correct construction Concepts attached.
- Update docs (DISCOURSE.md, INTERLINGUA.md, DEDUCTION.md).
- Exercise with dialogue_sample / benchmarks.
- Ensure no regression on prior copula/adj work.

**Cross-cutting**:
- Always use rtk for shell, CodeGraph for locating symbols before edits.
- Prefer data-driven (concepts.ron + lexicon) over hardcode.
- Keep graph as rich IR but make IL the primary "source of truth" for semantics + constructions.
- Update plan/checklists if a goal/plan.md or similar exists in your session.

---

## 6. Open Questions & Risks
- Should `Construction` be a top-level IL item (parallel to Frame) or a modifier on Frames/Entities?
- How deep should context propagate? (Only subjects? Full role resolution? Construction choice only?)
- Performance: Discourse tracking on very long dialogues?
- Backward compat: Existing graph edges vs new Concept-based constructions.
- How much of "expression tree" do we expose in public API vs keep internal?
- Integration with existing `resolved_refs` and `reference: Anaphoric`.

Discuss with user if blocked.

---

## 7. Key Files & Starting Points (for Next Agent)

**Core**:
- `src/core/interlingua.rs` – IL structs (add to Discourse/Entity/Sentence/Frame here).
- `src/core/graph.rs` – EdgeKinds, has_construction, coref helpers, register_...
- `src/core/context.rs` – track_discourse, link_..., recent_entities_of_type.
- `src/core/deduction.rs` – apply_graph_inference + construction attachment.

**Parsers/Gen**:
- `src/engines/pl/parser.rs` (grouping, retain, frame building for copula).
- `src/engines/en/generator.rs` + `src/generation/pipeline.rs` (realize, loc=None handling, construction checks).
- `src/engines/policy.rs`, realizer.

**Data & Tests**:
- `data/concepts/concepts.ron`
- `tests/graph_tests.rs` (extend multi-sentence tests).
- `tests/integration_test.rs` (precision copula + constructions).
- `docs/DISCOURSE.md`, `docs/INTERLINGUA.md`, `docs/DIALOGUE.md`.

**Other**:
- `src/core/mod.rs`, api.rs, translator.rs for entry points.
- Use CodeGraph queries: `codegraph query "Discourse"`, `codegraph query "has_construction"`, `codegraph callers "track_discourse"`, etc.

**Commands** (always prefix with rtk where applicable):
- `rtk cargo test --workspace`
- `rtk cargo run --quiet --bin lexflex -- translate -f pl -t en 'Tomek poszedł. Kupił mleko.'`
- `codegraph sync && codegraph query "..."`

---

## 8. Verification Criteria (for "done")

- Multi-sentence input with implicit subject produces IL where later Frames have subject Entity whose concept + name (or reference) matches the antecedent from sentence 1.
- Correct construction Concept(s) attached (visible via graph or new IL field).
- Output is coherent ("Tomek went... He bought..." or appropriate zero/pronoun in target).
- No new hardcodes for specific names.
- Existing copula/adj cases + single-sentence tests still pass.
- New tests + updated docs.

---

## 9. Recommended First Actions for Next Agent

1. `rtk cargo test --workspace` + run a few Tomek-style translates to establish baseline.
2. Read this handoff + `docs/DISCOURSE.md` + `docs/INTERLINGUA.md`.
3. Use CodeGraph + rg to locate discourse attachment points.
4. Add the failing "persistent subject" test case first (TDD style).
5. Start with Phase 1 + Phase 2 (Concepts + entity propagation).
6. Keep changes small, data-driven, and reversible. Capture evidence in `results/scratch/` or `/tmp/...` if following prior patterns.
7. When ready, run full verification and update this handoff or create follow-up notes.

**Contact / Context**: Refer back to this file and the conversation history for the "constructions as Concepts + IL as compilable tree" vision.

Good luck – this is a high-leverage architectural improvement that directly addresses long-standing coherence and special-case reduction goals.

---

*End of Handoff*