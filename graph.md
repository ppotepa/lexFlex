# Linguistic Graph Design & Exhaustive Implementation Plan

**Project**: lexFlex — Universal Interlingua for Polish ↔ English (and future languages)  
**Date**: 2026-07-12  
**Status**: Planning Document  
**Goal**: Transform the current token + IL model into a rich, directed, multi-layer **Linguistic Graph** that enables:
- Structural, direction-aware navigation and pattern matching
- Better grammatical construction deduction (e.g. `[Verb, complementizer, Subject]`)
- Robust text-level context (entity tracking, coreference, focus, discourse relations)
- Preparation for multi-utterance dialogue / chat
- **Full genericity** (language-neutral Interlingua core for PL and EN)
- **Plastic lexical abilities** (data-driven extension of concepts, relations, and lexical mappings with minimal code changes)

Everything must remain **graph-native**. Avoid surface lemma checks (`"mieszkać"`, `"to"`) in core logic. Use:
- Semantic `ConceptId` (LIVE, BE, HAVE_AGE, MOTION, WIFE, YEAR, ...)
- Grammatical categories (Infinitive, Finite, Perfective, Instrumental, ...)
- Typed directed edges and subgraph patterns

---

## 1. Vision & Principles

### 1.1 Core Vision
The entire linguistic object (surface words → syntax → semantics → discourse) is represented as one coherent, queryable, directed graph (or layered graphs with cross-layer links).

- **Surface Layer**: Raw words/tokens with linear order.
- **Syntactic Layer**: Phrases, heads, dependencies.
- **Semantic Layer**: Current Interlingua (Frames, Entities, roles) materialized as nodes + edges.
- **Discourse / Context Layer**: Entity tracking, coreference, focus, speaker/addressee, utterance history, topic chains.
- **Lexical / Conceptual Layer**: Concepts as first-class nodes with relations (`is_a`, `typical_accompaniment_of`, `can_be_location_for`, etc.).

Directionality (`next`/`prev`, `head`→`dependent`, `realizes`→, `corefers`↔) enables:
- Forward/backward construction extraction.
- Reliable deduction of grammatical patterns.
- Rich context building across sentences and utterances.

### 1.2 Non-Negotiable Principles (Interlingua PL/EN)
- **Interlingua remains the semantic source of truth**. The graph augments it — does not replace the meaning representation.
- **Genericity**: All core logic (deduction, context resolution, construction matching, generation decisions) operates on `ConceptId` + features + graph topology. Language-specific surface forms live only in:
  - Parsers (input → graph)
  - Realizers/Generators (graph + IL → output)
  - Lexicon data files
- **Lexical Plasticity**: Adding a new word, sense, or construction should require changes **only in data** (`data/lexicons/`, `data/concepts/`, `data/ontology/`) + possibly descriptor entries. No new `if verb_lemma == "..."` in pipeline or generators.
- **Bidirectional & Traceable**: Every surface word links to the semantic entities it realizes. Traces record the exact graph paths + decisions.
- **Symmetry for PL ↔ EN**: The same graph model and query primitives are used for both languages. Language-specific behavior is driven by `LanguageDescriptor` + lexicon + morphology data.
- **Incremental & Backward-Compatible**: Existing `Interlingua` (RON) serialization must continue to work. Graph is an additional rich view (can be serialized separately or embedded).

---

## 2. Current State & Gaps

### 2.1 What Exists Today
- `src/core/interlingua.rs`:
  - `Token` (parser-only, form/lemma/pos/features/span)
  - `Entity` (concept, name, features, reference, optional `coordination`, adjectives)
  - `Frame` enum (Existence, Possession, Motion, Transfer, ...)
  - `Sentence`, `Utterance`, `Discourse` (very basic: speaker/addressee/purpose/register)
  - `Interlingua::Natural(Utterance)`
- Parsers (PL/EN): build `Token` → `PartialStructure` → deduction → full IL. Tokens are discarded.
- Generation: `pipeline.rs` (frame → words using realizer), `realizer.rs`, language-specific generators.
- `src/engines/policy.rs`: `resolve_surface_verb` (prefers `verb_concept`).
- Data-driven core: `data/lexicons/{pl,en}/*.ron`, `concepts.ron`, `ontology.ron`, paradigms, descriptors.
- Benchmark: produces RON IL + `TraceStep` (stage/decision/reason) per sentence. Saved under `results/runs/...-trace/`.
- Existing docs: `INTERLINGUA.md`, `DISCOURSE.md`, `DIALOGUE.md`, `ARCHITECTURE.md`, `IMPLEMENTATION_ROADMAP.md`.

### 2.2 Critical Gaps
- No persistent word-level nodes after parsing.
- No directed navigation (`find.next_subject()`, `find_near()`, construction patterns).
- Tokens disappear → hard to correlate surface decisions with IL.
- Context is almost non-existent beyond single-sentence IL.
- `Discourse` is a stub (v0.2+).
- Logic sometimes still leans on surface lemmas or ad-hoc checks (historical "has"→"have", special cases).
- Traces are useful but not true graph traversals.
- Lexicon is flat lists; no rich concept-relation graph.
- No first-class representation of grammatical constructions as subgraphs.
- No entity tracking / coreference across sentences inside an utterance.
- Limited preparation for multi-utterance context.

---

## 3. Proposed Graph Model

### 3.1 Layers (stacked, with cross-links)

```
Discourse / Context Layer
  ↑ corefers, in_focus, continues_topic, speaker_turn
Semantic Layer (Interlingua materialized)
  ↑ realizes, has_role, verb_concept
Syntactic Layer
  ↑ syntactic_head, dependent, phrase_type
Surface Layer (WordNode)
  ←→ next / prev (directed linear order)
Lexical / Conceptual Layer (orthogonal)
  (Concept nodes + relations: is_a, typical_role, etc.)
```

All layers live in one `TextGraph` (or `LinguisticGraph`) with typed nodes and directed edges. Cross-layer edges (`realizes`, `syntactic_head_of`) connect them.

### 3.2 Node Types (exhaustive)

```rust
// Core node enum (use enum or trait object + downcast, or separate arenas)
enum GraphNode {
    Word(WordNode),
    Phrase(PhraseNode),
    Clause(ClauseNode),
    Sentence(SentenceNode),
    Entity(EntityNode),           // materialization of current Entity
    Frame(FrameNode),             // materialization of Frame + verb_concept
    Concept(ConceptNode),         // from data/concepts + ontology
    Utterance(UtteranceNode),
    Discourse(DiscourseNode),
    // future: Turn, Dialogue, etc.
}

struct WordNode {
    id: NodeId,
    form: String,
    lemma: String,
    pos: PartOfSpeech,
    features: FeatureBundle,      // grammatical features (tense, case, number, infinitive?, ...)
    span: (usize, usize),
    // links populated during parsing/generation
}

struct EntityNode {
    id: NodeId,
    concept: ConceptId,           // "LIVE", "WIFE", "YEAR", ...
    features: FeatureBundle,
    name: Option<String>,
    reference: Reference,
    // back-links to realizing WordNodes + syntactic phrases
}

struct FrameNode {
    id: NodeId,
    kind: FrameKind,              // Existence, Possession, ...
    verb_concept: ConceptId,      // "LIVE", "BE", "HAVE_AGE", ...
    roles: Vec<(SemanticRole, EntityNodeId)>,
    // ...
}
```

### 3.3 Directed Edge Types (exhaustive, typed)

Use a rich `Edge` struct or enum of edge kinds with direction.

Important directed edges:

**Linear / Surface**
- `Next` / `Prev` (word-to-word, phrase-to-phrase)

**Syntactic**
- `SyntacticHead` (dependent → head)
- `Dependent` (head → dependent)
- `ModifierOf`
- `ComplementOf`

**Semantic / Role**
- `Realizes` (Word/Phrase → Entity)
- `HasRole` (Entity → Frame with role label)
- `AgentOf`, `PatientOf`, `LocationOf`, `AccompanimentOf`, ... (derived or explicit)

**Construction / Pattern**
- `PartOfConstruction` (e.g. "AccompanimentWithList", "AgeIdiom", "MotionGoal")
- `MatchesPattern` (points to a named ConstructionPattern)

**Discourse / Context**
- `Corefers` (bidirectional or directed with strength)
- `Introduces` (first mention)
- `InFocus`
- `RecentMention`
- `ContinuesTopic`
- `Answers`, `Elaborates`, `Contrasts` (between utterances)

**Lexical**
- `EvokesConcept` (Word → Concept)
- `ConceptRelation` (`is_a`, `part_of`, `typical_with`, `can_be_location_for_LIVE`, ...)

**Generation / Trace**
- `ChosenForRealization`
- `DecisionEdge` (links decision to the graph elements that influenced it)

### 3.4 Grammatical Categories (first-class, not strings)

Store in `FeatureBundle` or dedicated enums on nodes:

- `VerbForm`: Finite, Infinitive, Participle, Gerund, ...
- `Mood`, `Aspect` (perfective/imperfective), `Tense`, `Voice`
- `Case` (already rich), `Number`, `Person`, `Gender` (incl. virile), `Animacy`
- `Definiteness`, `Countability`, etc.

Construction patterns are recognized by **topology + node features + concept**, e.g.:

Path pattern for accompaniment after "z"/"razem z":
```
Word(lemma~="z" | concept=ACCOMPANIMENT) --Next--> Word(pos=Noun|Pronoun) --Coordination--> [Entity(concept in {WIFE, DAUGHTER, DOG, PERSON}) ...]
```

---

## 4. Lexical Plasticity & Genericity Strategy

- **Concepts are primary keys**. All `verb_concept`, `concept` fields stay as `ConceptId`.
- Lexicon entries map `(language, form/lemma) → (ConceptId, default features, paradigm)`.
- New words/senses = new entries in `data/lexicons/{pl,en}/lexicon.ron` + possibly `concepts.ron`.
- New relations between concepts go into `data/ontology/ontology.ron` or a new `concept_relations.ron`.
- Grammar constructions can be registered declaratively (future: pattern definitions in data).
- Language descriptors (`data/descriptors/pl.ron`, `en.ron`) declare which features/cases/preposition_roles they support.
- Parsers and generators read descriptors + graph topology + concepts.

This gives **plasticity**: adding "współpracownik" as COLLEAGUE or a new age idiom variant requires data only.

---

## 5. Context Building for Text & Utterances

`Utterance` becomes `UtteranceNode` that owns/points to its `SentenceNode`s.

Within one utterance:
- Run entity tracking after deduction: link anaphors, maintain "recent entities" list per semantic type.
- Build `Corefers` edges.
- Maintain focus stack (last mentioned salient entities).

Across utterances (future chat):
- `DialogueGraph` containing sequence of `UtteranceNode`s.
- Cross-utterance `Corefers`, `ContinuesTopic`, `SpeakerTurn` edges.
- Persistent `DiscourseState` (shared entities, relationship, register).

`Discourse` struct will be greatly expanded and become part of the graph.

---

## 6. Query / Navigation API (examples of what we want)

High-level fluent or method-based API on `TextGraph` / `SentenceGraph` / `WordNode`:

```rust
// Direction-aware
word.next(1).find_subject();
word.sentence().find_preposition("z").following_coordination();
word.find_in_construction("Accompaniment");

// Semantic + grammatical
sentence.find_verbs()
    .where_concept_in(["LIVE", "RESIDE"])
    .where_feature(VerbForm::Finite)
    .with_location_accompaniment();

// Context
graph.recent_entities_of_type("PERSON", window=3);
entity.coreference_chain();
sentence.in_focus_entities();

// Construction extraction
graph.find_paths()
    .starting_with_verb()
    .then_complementizer_or_preposition()
    .then_subject_or_coordination();
```

These queries power deduction, generation choices, and rich traces.

---

## 7. Traces & Analysis

- Every `TraceStep` records the set of `NodeId`s and `EdgeId`s involved.
- Benchmark can export full graph (or subgraph) + decisions in RON or a dedicated format.
- Goal: "w pełni wynioskować" — be able to walk the exact graph paths that led to a surface form or IL structure.

---

## 8. Exhaustive Phased Implementation Plan (Hyper-Detailed with Symbols & File Paths)

This section is the **navigable implementation bible**. Every bullet contains:
- Exact file path
- Function / struct / field / method symbols
- Suggested insertion points or new code signatures
- PL vs EN differences
- How it enables graph navigation (`word.next()`, `find_paths()`, etc.)

Use Ctrl+F / search on symbols like `WordNode`, `build_partial_structure`, `Realizes` to jump.

### Phase 0 — Foundations, IDs, Graph Skeleton & Audit (1-2 weeks)

**0.1 Audit surface hacks (search these exact strings)**
- Search `src/generation/pipeline.rs` for:
  - `if verb_lemma == "have" || verb_lemma == "ma"`
  - `if verb_concept == "HAVE_AGE"`
  - `let use_with = loc.concept.0 == "PERSON"`
  - `if tokens.iter().any(|t| t.form.contains("żoną"))` (should already be gone)
- Search `src/engines/en/generator.rs:490` (around `if let Some(ref coord) = entity.coordination`)
- Search `src/engines/policy.rs:87` `pub fn resolve_surface_verb`
- Search `src/engines/pl/parser.rs:361` `fn build_partial_structure`
- Search `src/engines/en/parser.rs` for any `lemma ==` or `form.contains`

**0.2 Core ID types (add to `src/core/interlingua.rs`)**
```rust
// src/core/interlingua.rs: after ConceptId and EntityId
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    Next, Prev,
    SyntacticHead, Dependent,
    Realizes, HasRole(SemanticRole),
    Corefers, Introduces,
    InFocus, RecentMention,
    EvokesConcept,
    PartOfConstruction(String),
    // ... more
}
```

**0.3 New module skeleton**
- Create `src/core/graph.rs`
  - `pub struct LinguisticGraph { nodes: Vec<GraphNode>, edges: Vec<Edge> }`
  - `pub struct Edge { id: EdgeId, from: NodeId, to: NodeId, kind: EdgeKind }`
  - `impl LinguisticGraph { pub fn add_word(&mut self, w: WordNode) -> NodeId { ... } }`

**0.4 Wire basic graph into existing types**
- `src/core/interlingua.rs:579` `pub struct Sentence` → add:
  ```rust
  pub graph: Option<LinguisticGraph>,           // or separate surface_graph
  pub sentence_node_id: Option<NodeId>,
  ```
- `src/core/interlingua.rs:630` `pub struct Utterance` → add `utterance_node_id: Option<NodeId>`
- `src/core/mod.rs` → `pub use graph::{LinguisticGraph, WordNode, ...};`

**0.5 Update docs & plan**
- `graph.md` (this file)
- `goal/plan.md`
- `docs/ARCHITECTURE.md`, `docs/INTERLINGUA.md`, `docs/DISCOURSE.md`, `docs/IMPLEMENTATION_ROADMAP.md`

**Key symbols to introduce in Phase 0**:
- `NodeId`, `EdgeId`, `EdgeKind`
- `LinguisticGraph`
- `Sentence.graph: Option<LinguisticGraph>`

---

### Phase 1 — Surface Graph: WordNode + Directed Linear Order (3-4 weeks)

**Goal**: Every surface token becomes a first-class `WordNode` with `next`/`prev`. Enables `word.next(1)`, `sentence.find_words()`.

**1.1 Define WordNode (src/core/graph.rs)**
```rust
#[derive(Debug, Clone)]
pub struct WordNode {
    pub id: NodeId,
    pub form: String,
    pub lemma: String,
    pub pos: PartOfSpeech,
    pub features: FeatureBundle,   // reuse from interlingua.rs:186
    pub span: (usize, usize),
    pub next: Option<NodeId>,
    pub prev: Option<NodeId>,
    pub sentence_id: Option<NodeId>,   // link to SentenceNode
    pub evokes: Option<ConceptId>,     // will be filled later
}
```

**1.2 Token integration**
- `src/core/interlingua.rs:679` `pub struct Token` → add:
  ```rust
  pub word_node_id: Option<NodeId>,
  ```
- Keep `Token` for parser internal use, but immediately promote to `WordNode`.

**1.3 PL Parser changes (`src/engines/pl/parser.rs`)**
- `fn tokenize` (line 133):
  - After creating each `Token`, immediately create `WordNode` and store `word_node_id`.
  - After the while loop, wire `next`/`prev`:
    ```rust
    for i in 0..words.len()-1 {
        if let (Some(id1), Some(id2)) = (words[i].word_node_id, words[i+1].word_node_id) {
            graph.add_edge(id1, id2, EdgeKind::Next);
            graph.add_edge(id2, id1, EdgeKind::Prev);
        }
    }
    ```
- `fn build_partial_structure` (line 361):
  - Accept `&mut LinguisticGraph`
  - Create `SentenceNode { id: ..., words: vec![...] }`
  - `sentence.graph = Some(graph.clone());`

**1.4 EN Parser symmetry (`src/engines/en/parser.rs`)**
- Mirror the exact changes in `tokenize` and `build_partial_structure`.
- Ensure `FeatureBundle` population uses the same `analyze_token` path for consistency.

**1.5 Sentence / Utterance updates**
- `src/core/interlingua.rs:579` `Sentence`:
  ```rust
  pub words: Vec<WordNode>,           // convenience view
  pub sentence_node: Option<SentenceNode>,
  ```
- Add helper impls in `graph.rs`:
  ```rust
  impl WordNode {
      pub fn next<'a>(&self, graph: &'a LinguisticGraph) -> Option<&'a WordNode> { ... }
      pub fn prev<'a>(&self, graph: &'a LinguisticGraph) -> Option<&'a WordNode> { ... }
      pub fn sentence<'a>(&self, graph: &'a LinguisticGraph) -> Option<&'a SentenceNode> { ... }
  }
  ```

**1.6 Benchmark & Trace integration**
- `src/bin/benchmark.rs:99` (after `let il_result = ...`):
  - Serialize `graph` (or at least the `WordNode`s + Next edges) into the trace RON.
  - New trace section: `SURFACE_GRAPH: ...`

**PL vs EN specifics**:
- PL must handle "razem z" as single preposition token (already in `check_multiword_expression`).
- EN has no cases, but still needs `WordNode` for uniformity.

**New symbols introduced**:
- `WordNode { id, form, lemma, next, prev, evokes }`
- `SentenceNode`
- `WordNode::next()`, `find_next_subject()` (stub for later)
- `LinguisticGraph::add_edge(from, to, EdgeKind::Next)`

**Tests to add**:
- In `src/engines/pl/parser.rs` tests: assert `word.next.is_some()`
- `tests/integration_test.rs`: `test_surface_graph_linear_order`

---

### Phase 2 — Syntactic + Semantic Layer (EntityNode, FrameNode, Realizes edges) (4-5 weeks)

**2.1 New node types in `src/core/graph.rs`**
```rust
pub struct EntityNode {
    pub id: NodeId,
    pub concept: ConceptId,           // "LIVE", "WIFE", "YEAR"
    pub features: FeatureBundle,
    pub name: Option<String>,
    pub realizing_words: Vec<NodeId>, // back-links
}

pub struct FrameNode {
    pub id: NodeId,
    pub kind: String,                 // "Existence", "Possession"
    pub verb_concept: ConceptId,      // "BE", "LIVE"
    pub roles: Vec<(SemanticRole, NodeId)>,
}
```

**2.2 Wire during parsing (key functions)**
- `src/engines/pl/parser.rs:918` `fn build_frame(...)`:
  - After `let frame = self.build_frame(...)`
  - For every `Entity` created:
    ```rust
    let enode = EntityNode { concept: e.concept, ... };
    let eid = graph.add_entity(enode);
    // For each word that contributed to this entity
    graph.add_edge(word_id, eid, EdgeKind::Realizes);
    ```
- Same in `src/engines/en/parser.rs:347` `build_frame`

**2.3 Role edges**
- In `build_frame` (both languages), after role assignment:
  ```rust
  graph.add_edge(entity_id, frame_id, EdgeKind::HasRole(role));
  ```

**2.4 Coordination materialization**
- `src/core/interlingua.rs:256` `pub struct Coordination`
- When `entity.coordination.is_some()`:
  - Create `CoordinationNode`
  - `EdgeKind::CoordinatesWith` between item `EntityNode`s

**2.5 Phrase layer (light)**
- Create `PhraseNode { id, phrase_type: "NP" | "PP" | "VP", head: NodeId, members: Vec<NodeId> }`
- Populate from existing grouping logic in `build_partial_structure` (around line 620 in pl parser for coordination).

**2.6 Update generation to read graph**
- `src/generation/pipeline.rs:461` (Existence location block):
  ```rust
  // instead of heuristic
  if let Some(word) = loc.realizing_words.first() {
      if graph.get_word(*word).features.case == Some(Case::Instrumental) { "with" }
  }
  ```
- `src/engines/en/generator.rs:710` (realize_noun_phrase coord branch) → query graph for full construction.

**Key symbols to search/edit**:
- `build_frame` (pl:918, en:~347)
- `Entity { concept, features, coordination }`
- `Frame::Existence { location, verb_concept }`
- `resolve_surface_verb` (policy.rs:87)
- `generate_frame` (pipeline.rs:164)

---

### Phase 3 — Discourse & Text Context Layer (4-6 weeks)

**3.1 Expand Discourse**
- `src/core/interlingua.rs:620` `pub struct Discourse` → add:
  ```rust
  pub entities_in_focus: Vec<NodeId>,
  pub recent_mentions: Vec<(NodeId, usize)>, // entity, recency
  pub coref_edges: Vec<EdgeId>,
  pub utterance_node_id: Option<NodeId>,
  ```

**3.2 Entity tracking pass (new file)**
- Create `src/core/context.rs`:
  ```rust
  pub fn track_entities(utt: &mut Utterance, graph: &mut LinguisticGraph) {
      // walk sentences, build Corefers edges using name + features + recency
  }
  ```

**3.3 Cross-sentence links**
- After `build_partial_structure` for each sentence:
  - `graph.add_edge(prev_sentence_id, current_sentence_id, EdgeKind::NextSentence);`

**3.4 Focus & topic**
- `SentenceNode` gets `focus_entity: Option<NodeId>`
- Update after every `Frame` that introduces a salient `Entity`.

**Future utterance support**:
- `src/core/graph.rs`:
  ```rust
  pub struct DialogueGraph { utterances: Vec<NodeId>, ... }
  pub fn add_utterance(&mut self, u: UtteranceNode) { ... resolve_cross_utterance_coref(...) }
  ```

---

### Phase 4 — Lexical/Conceptual Layer + Plasticity (3-4 weeks)

**4.1 ConceptNode**
- `src/core/graph.rs`:
  ```rust
  pub struct ConceptNode {
      pub id: NodeId,
      pub concept: ConceptId,   // "LIVE"
      pub relations: Vec<(String, ConceptId)>, // "typical_accompaniment" -> "PERSON"
  }
  ```

**4.2 Loading**
- `src/data/loader.rs` + `src/data/lexicon.rs`:
  - Load `concepts.ron` into `ConceptNode`s
  - During `WordNode` creation: `word.evokes = lookup_concept(lemma)`

**4.3 Queries become data-driven**
- Replace any hard-coded lists in `pipeline.rs` / `deduction.rs` with:
  ```rust
  graph.concept("LIVE").related("can_have_location")
  ```

---

### Phase 5 — Query API, Construction Patterns, Rich Traces (4 weeks)

**5.1 Query primitives (src/core/graph.rs)**
```rust
impl LinguisticGraph {
    pub fn find_words(&self) -> WordIterator { ... }
    pub fn find_paths(&self) -> PathBuilder { ... }
    pub fn word(&self, id: NodeId) -> Option<&WordNode> { ... }
}

pub struct PathBuilder { ... }
impl PathBuilder {
    pub fn starting_with_verb(self) -> Self { ... }
    pub fn then_complementizer(self) -> Self { ... }
    pub fn matching_concept(self, c: &str) -> Self { ... }
}
```

**5.2 Construction registry**
- `src/core/graph.rs`:
  ```rust
  pub fn register_construction(&mut self, name: &str, pattern: PatternDef);
  pub fn find_construction(&self, word: NodeId, name: &str) -> Option<Vec<NodeId>>;
  ```

**5.3 Trace enhancement**
- `src/generation/pipeline.rs:171` (TRACE push):
  ```rust
  TRACE.with(|t| t.borrow_mut().push(TraceStep {
      ...
      involved_nodes: vec![word_id, entity_id],
  }));
  ```

**5.4 Benchmark output**
- `src/bin/benchmark.rs`: dump `graph` as RON under `SURFACE_NODES`, `EDGES`, `CONSTRUCTIONS`.

---

### Phase 6 — Generation Walks the Graph (3-4 weeks)

**Key edits**:
- `src/generation/pipeline.rs:420` (Existence arm) → use graph to decide "with" vs "in"
- `src/engines/en/generator.rs:490` and `710` (coord handling) → query `graph.find_coordination(entity_id)`
- Remove remaining string hacks in `realizer.rs` and generators.
- Ensure `realize_noun_phrase` receives graph context for better decisions.

---

### Phase 7 — Multi-Utterance & Polish (4+ weeks)

- Implement `DialogueGraph`
- Update `src/api.rs`, `src/translator.rs` to accept `Vec<Utterance>`
- Full context carry-over in `src/core/context.rs`
- Update benchmark for multi-sentence inputs
- Performance: indices on `NodeId`, avoid cloning whole graph

---

### Phase 8 — Documentation, Cleanup, Migration

- Update every `docs/*.md` with "Graph" sections
- Add examples in `docs/END_TO_END_EXAMPLE.md` showing `word.next().realizes().concept`
- Remove deprecated special cases
- Add `#[deprecated]` on old ad-hoc methods

---

## Navigation Table (Fast Jump)

| What you want to find                  | Symbol / Path                                      | File |
|----------------------------------------|----------------------------------------------------|------|
| Surface word definition                | `struct WordNode`                                  | `src/core/graph.rs` |
| Token promotion                        | `Token.word_node_id`                               | `src/core/interlingua.rs:679` |
| Main parsing entry                     | `fn parse` / `fn build_partial_structure`          | `src/engines/pl/parser.rs:27,361` |
| Role assignment + Entity creation      | `fn build_frame`                                   | `src/engines/pl/parser.rs:918` |
| Verb resolution (replace later)        | `resolve_surface_verb`                             | `src/engines/policy.rs:87` |
| Frame processing                       | `fn generate_frame`                                | `src/generation/pipeline.rs:164` |
| Location preposition decision          | Existence arm in `generate_frame`                  | `src/generation/pipeline.rs:420` |
| Coordination realization               | `if let Some(ref coord)`                           | `src/engines/en/generator.rs:490,710` |
| Discourse stub                         | `struct Discourse`                                 | `src/core/interlingua.rs:620` |
| Concept loading                        | `loader::load_concepts`                            | `src/data/loader.rs` |
| Trace step                             | `struct TraceStep { involved_nodes }`              | `src/generation/pipeline.rs:10` |

Search the codebase for these exact strings to locate edit points quickly.

This level of symbol + path detail should make navigation trivial. Update this section as you implement.
3. Ensure "I live with a wife, a daughter, and a dog." and age idioms continue to work (or improve) purely via graph + concepts.
4. Update English/Polish morphology and realizer to consult graph features when available.

### Phase 7 — Multi-Utterance / Dialogue Preparation & Polish (ongoing)
1. Implement `DialogueGraph` skeleton.
2. Add utterance-level context carry-over (shared entities, focus persistence).
3. Update `src/api.rs` / translator to accept sequences of utterances.
4. Extend benchmark to support multi-utterance inputs.
5. Full test coverage + golden files for context scenarios.

### Phase 8 — Documentation, Migration, Cleanup
- Update all `docs/*.md` (especially INTERLINGUA, DISCOURSE, DIALOGUE, ARCHITECTURE, IMPLEMENTATION_ROADMAP).
- Migration guide for any breaking changes to `Interlingua` structs (minimize them).
- Performance audit (graph size for 50-sentence Adam text).
- Remove dead code and old special cases.
- Add `cargo test --features graph` or similar if needed.

---

## 9. Data File Changes

- `data/lexicons/pl/lexicon.ron` & `en/lexicon.ron` — add any missing explicit inflected forms or richer annotations if needed for graph features.
- `data/concepts/concepts.ron` — ensure all needed concepts exist.
- `data/ontology/ontology.ron` — add concept hierarchy and new relations.
- `data/descriptors/pl.ron`, `en.ron` — possibly add `construction_roles` or `graph_features` section later.
- New (optional): `data/graph_patterns.ron` for declarative construction definitions.

---

## 10. Testing & Verification Strategy

- **Unit**: Parser produces correct WordNodes + next edges; construction matchers.
- **Integration**: Existing tests + new graph-aware tests (`graph_has_coord_for_accompaniment`, `age_produces_correct_nodes`, `coref_chain_across_sentences`).
- **Golden traces**: Every benchmark run saves the graph snapshot (or diffable subset) under `results/runs/...`.
- **Property-based**: "For any sentence, number of WordNodes == number of tokens".
- **Regression**: All 44+ current tests must pass after each phase.
- **Manual**: Re-run Adam narrative; inspect new rich traces for full "w pełni wynioskować" capability.
- **Symmetry**: PL→IL graph and EN→IL graph must allow equivalent queries for the same meaning.

---

## 11. Risks, Mitigations, and Non-Goals

**Risks**:
- Graph mutation during generation (keep graph mostly immutable after parsing; build generation views).
- Performance on very long texts (use indices, avoid full traversals when simple iteration suffices).
- Over-engineering (start minimal: Word + next + realizes + basic context).
- RON serialization of graph (use indices + separate node/edge arrays).

**Mitigations**:
- Strict layering and clear ownership.
- Keep classic IL structs as the stable public API.
- Heavy use of `#[cfg(test)]` and incremental rollout.

**Non-Goals (for v1 of graph)**:
- Full petgraph or external graph DB.
- Visual graph rendering.
- Complete dialogue management (only the substrate).
- Replacing all morphology with graph rules.

---

## 12. Summary of Files to Create / Modify (Master List)

**New**:
- `graph.md`
- `src/core/graph.rs`
- `src/core/context.rs` (or discourse tracking)
- Possibly `src/core/graph_query.rs`

**Heavily modified**:
- `src/core/interlingua.rs`
- `src/engines/pl/parser.rs`
- `src/engines/en/parser.rs`
- `src/core/deduction.rs`
- `src/generation/pipeline.rs`
- `src/generation/realizer.rs`
- `src/engines/{pl,en}/generator.rs`
- `src/engines/policy.rs`
- `src/bin/benchmark.rs`
- `src/data/loader.rs`, `src/data/lexicon.rs`
- `tests/integration_test.rs`
- Multiple `docs/*.md` and `goal/plan.md`

**Data**:
- Various `.ron` files (mostly additive)

---

## 13. Estimated Timeline (rough, one focused developer)

- Phase 0: 1-2 weeks
- Phase 1: 3-4 weeks
- Phase 2: 4-5 weeks
- Phase 3: 4-6 weeks
- Phase 4: 3-4 weeks
- Phase 5-6: 7-8 weeks
- Phase 7-8: 4-6 weeks

**Total to usable rich graph + context for utterances**: 6-9 months of focused work, or faster with parallel work on independent phases (e.g. lexical layer can proceed in parallel after Phase 1).

Start with Phase 0 + Phase 1 — they deliver immediate value for analysis and set the foundation for everything else.

---

This document is the master plan. Update it as implementation progresses. Every change should be traceable back to a section here.

**Next immediate action**: Implement Phase 0, then start wiring `WordNode` + `next` edges in both parsers.

End of plan.