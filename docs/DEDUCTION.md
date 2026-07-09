# Deduction Engine — Resolving Ambiguity and Completing Meaning

**Status:** Core component of v0.1 (used by all natural language engines)

The Deduction Engine runs **after surface parsing** and **before** the final Interlingua representation is produced. Its job is to resolve ambiguities and fill in implicit information using linguistic knowledge (verb frames, ontology, grammar rules).

## Purpose

Surface parse produces **partial / ambiguous** structures. Deduction turns them into **complete, language-neutral Interlingua**.

Example:
- Surface: `jabłko` (case = unknown)
- After deduction: `jabłko` as Theme in ACC case (because "dać" requires agent:NOM + recipient:DAT + theme:ACC)

## When Deduction Runs

```
Surface Parse (engine) 
    → Deduction (core + engine rules)
        → Full InterlinguaNode
            → Capability Check
                → Generation
```

Deduction is **always** executed during `to_interlingua()` for natural languages in v0.1.

## Responsibilities in v0.1

| Responsibility                    | Scope in v0.1                  | Example |
|-----------------------------------|--------------------------------|---------|
| Case disambiguation               | Full                           | `jabłko` → ACC via verb subcat |
| Basic pronoun resolution          | Within single sentence         | Reflexives → subject of clause |
| Temporal anchoring                | Deictic expressions            | `wczoraj` → relative to "now" |
| Ontology type validation          | Full                           | Reject "kamień zjadł jabłko" |
| Feature inheritance               | Full                           | APPLE inherits from FRUIT → FOOD |
| Aspect / tense normalization      | Full                           | Perfective + Past → completed event |
| Polarity & modality propagation   | Full                           | Negation scope |

**Out of scope in v0.1:**
- Multi-sentence coreference
- Discourse salience / topic tracking
- Full quantifier scope resolution across clauses
- Ellipsis resolution across sentences

## Parser vs Deduction: Clear Boundaries

### Design Principle

**"Parser should be as simple as possible, Deduction should be as powerful as possible."**

The Parser's job is to do **minimal structural analysis** - just enough to identify the basic components (tokens, POS tags, morphological features). The Deduction Engine's job is to do **all semantic reasoning** - resolving ambiguities, assigning roles, validating types.

**Why this matters:**
- Parser is language-specific (different for each language)
- Deduction is mostly language-independent (core logic is shared)
- Simple parser = easier to implement and maintain
- Powerful deduction = better semantic understanding

### What Belongs in Parser

| Task | Why Parser | Example |
|------|-----------|---------|
| **Tokenization** | Language-specific rules | Split "Tomek dał" into tokens |
| **POS tagging** | Language-specific morphology | "dał" → Verb |
| **Morphological analysis** | Language-specific inflection | "dał" → lemma: "dać", tense: Past |
| **Identify main verb** | Basic syntactic structure | Find "dał" as main verb |
| **Collect NPs** | Basic phrase structure | Find "Tomek", "jabłko", "Izie" |

**Parser should NOT:**
- ❌ Assign semantic roles (Agent, Theme, etc.)
- ❌ Resolve case ambiguities
- ❌ Validate semantic types
- ❌ Handle pronoun resolution
- ❌ Make any semantic decisions

### What Belongs in Deduction

| Task | Why Deduction | Example |
|------|--------------|---------|
| **Verb frame matching** | Requires semantic knowledge | "dać" → Transfer frame |
| **Role assignment** | Requires frame + case knowledge | Tomek (NOM) → Agent |
| **Case resolution** | Requires frame + ontology | "jabłko" → ACC (not NOM) |
| **Pronoun resolution** | Requires context + binding theory | "się" → subject |
| **Temporal anchoring** | Requires world knowledge | "wczoraj" → relative to now |
| **Ontology validation** | Requires type hierarchy | Reject "kamień zjadł" |
| **Feature inheritance** | Requires ontology | APPLE → FRUIT → FOOD |

**Deduction should:**
- ✅ Make all semantic decisions
- ✅ Resolve all ambiguities
- ✅ Provide complete, unambiguous InterlinguaNode
- ✅ Validate semantic types
- ✅ Handle pronoun resolution (within sentence)

### Decision Table

| Decision | Parser | Deduction | Rationale |
|----------|--------|-----------|-----------|
| Identify verb | ✅ | ❌ | Basic syntactic structure |
| Identify nouns | ✅ | ❌ | Basic phrase structure |
| Morphological analysis | ✅ | ❌ | Language-specific inflection |
| Assign semantic roles | ❌ | ✅ | Requires frame knowledge |
| Resolve case ambiguity | ❌ | ✅ | Requires frame + ontology |
| Validate semantic types | ❌ | ✅ | Requires ontology |
| Resolve pronouns | ❌ | ✅ | Requires binding theory |
| Anchor temporals | ❌ | ✅ | Requires world knowledge |
| Handle negation scope | ❌ | ✅ | Requires semantic reasoning |

### Example: Where Does Logic Belong?

**Example 1: Identifying the main verb**
```
Input: "Tomek dał jabłko Izie"

Parser:
  - Tokenize: ["Tomek", "dał", "jabłko", "Izie"]
  - POS tag: Tomek(Noun), dał(Verb), jabłko(Noun), Izie(Noun)
  - Identify main verb: "dał" (first verb in sentence)
  
Deduction:
  - Look up "dać" in lexicon → Transfer frame
  - Assign roles based on frame
```

**Why Parser?** Identifying the main verb is basic syntactic structure. It doesn't require semantic knowledge.

**Example 2: Resolving case ambiguity**
```
Input: "Tomek dał jabłko Izie"

Parser:
  - Morphological analysis: "jabłko" → lemma: "jabłko", case: **ambiguous** (NOM or ACC)
  
Deduction:
  - Look up "dać" → Transfer frame requires [Agent:NOM, Recipient:DAT, Theme:ACC]
  - Tomek is NOM → Agent ✓
  - Izie is DAT → Recipient ✓
  - jabłko must be ACC → Theme ✓ (not NOM)
```

**Why Deduction?** Resolving case ambiguity requires knowledge of verb frames and semantic roles. Parser doesn't have this knowledge.

**Example 3: Pronoun resolution**
```
Input: "Tomek widzi się"

Parser:
  - Identify pronoun: "się" (reflexive)
  
Deduction:
  - Apply binding theory: reflexive must bind to subject
  - Subject is "Tomek"
  - Resolve "się" → Tomek
```

**Why Deduction?** Pronoun resolution requires binding theory and semantic reasoning. Parser just identifies the pronoun.

### When to Move Logic from Parser to Deduction

**Move to Deduction when:**
- Logic requires semantic knowledge (frames, roles, ontology)
- Logic requires world knowledge (temporal anchoring, common sense)
- Logic requires cross-sentence context (coreference, discourse)
- Logic is language-independent (shared across languages)

**Keep in Parser when:**
- Logic is purely syntactic (tokenization, POS tagging)
- Logic is language-specific (morphological rules)
- Logic doesn't require semantic knowledge
- Logic is simple and fast

### Example: Refactoring from Parser to Deduction

**Before (wrong):**
```rust
// Parser doing semantic work
fn parse(&self, input: &str) -> Result<Utterance, ParseError> {
    let tokens = self.tokenize(input);
    let verb = self.find_main_verb(&tokens)?;
    
    // ❌ Parser shouldn't do this!
    let frame = self.assign_semantic_roles(&verb, &tokens)?;
    
    Ok(Utterance { frames: vec![frame] })
}
```

**After (correct):**
```rust
// Parser does minimal work
fn parse(&self, input: &str) -> Result<PartialUtterance, ParseError> {
    let tokens = self.tokenize(input);
    let verb = self.find_main_verb(&tokens)?;
    let nps = self.collect_noun_phrases(&tokens);
    
    // ✅ Just return partial structure
    Ok(PartialUtterance { verb, nps })
}

// Deduction does semantic work
fn deduce(partial: PartialUtterance, context: &DeductionContext) -> Result<Utterance, DeductionError> {
    // ✅ Assign semantic roles here
    let frame = assign_semantic_roles(&partial.verb, &partial.nps, context)?;
    
    Ok(Utterance { frames: vec![frame] })
}
```

### Testing the Boundary

**Test 1: Can Parser work without semantic knowledge?**
- ✅ Yes - Parser should only need morphological rules and basic syntax
- ❌ No - If Parser needs frames/roles, logic belongs in Deduction

**Test 2: Can Deduction work without language-specific rules?**
- ✅ Yes - Deduction should use language-independent semantic rules
- ❌ No - If Deduction needs language-specific morphology, logic belongs in Parser

**Test 3: Is the logic shared across languages?**
- ✅ Yes - Logic belongs in Deduction (shared core)
- ❌ No - Logic belongs in Parser (language-specific)

## Deduction Pipeline (v0.1)

```rust
pub struct DeductionContext {
    /// Current time for temporal anchoring
    pub current_time: Timestamp,
    
    /// Language being processed
    pub language: LanguageId,
    
    /// Lexicon for verb subcategorization
    pub lexicon: &SubLexicon,
    
    /// Ontology for type validation
    pub ontology: &Ontology,
    
    /// Feature v0.2+: discourse context (None in MVP v0.1)
    pub discourse: Option<&Discourse>,
}

pub fn deduce(
    mut utterance: Utterance,
    language: &LanguageId,
    context: &DeductionContext,
) -> Result<Utterance, DeductionError> {

    for sentence in &mut utterance.sentences {
        // 1. Verb frame matching (subcategorization)
        apply_verb_frames(sentence)?;

        // 2. Case & role resolution
        resolve_cases_and_roles(sentence)?;

        // 3. Pronoun resolution (within sentence only)
        resolve_pronouns_within_sentence(sentence)?;

        // 4. Temporal anchoring
        anchor_temporals(sentence, context.current_time)?;

        // 5. Ontology validation + feature inheritance
        validate_and_inherit_from_ontology(sentence)?;

        // 6. Normalize features (aspect, polarity, etc.)
        normalize_features(sentence)?;
    }

    Ok(utterance)
}
```

## 1. Verb Frame Matching (Subcategorization)

Every verb in the lexicon has an associated **frame** (from `concepts.ron`).

```ron
"GIVE": Concept(
    id: "GIVE",
    frame_type: Some("Transfer"),
    roles: ["Agent", "Recipient", "Theme"],
    ...
)
```

### Detailed Algorithm

```rust
fn apply_verb_frames(
    sentence: &mut Sentence,
    context: &DeductionContext,
) -> Result<(), DeductionError> {
    
    // Step 1: Identify the main verb
    let verb_token = sentence.tokens.iter()
        .find(|t| t.pos == PartOfSpeech::Verb)
        .ok_or(DeductionError::NoVerbFound)?;
    
    // Step 2: Look up verb in lexicon to get frame
    let verb_entry = context.lexicon.lookup_verb(&verb_token.lemma)
        .ok_or(DeductionError::UnknownVerb { lemma: verb_token.lemma.clone() })?;
    
    let frame_template = verb_entry.frame_template;
    // Example: Frame::Transfer { agent: None, recipient: None, theme: None }
    
    // Step 3: Collect all noun phrases (NPs) and prepositional phrases (PPs)
    let np_candidates: Vec<&MorphAnalysis> = sentence.tokens.iter()
        .filter(|t| t.pos == PartOfSpeech::Noun || t.pos == PartOfSpeech::Pronoun)
        .collect();
    
    // Step 4: Assign NPs to frame roles
    let mut remaining_roles = frame_template.required_roles.clone();
    let mut role_assignments: Vec<(SemanticRole, &MorphAnalysis)> = Vec::new();
    
    // Priority 1: Explicit case assignment
    for np in &np_candidates {
        if let Some(case) = np.features.case {
            let matching_role = find_role_for_case(case, &remaining_roles, context);
            if let Some(role) = matching_role {
                role_assignments.push((role, np));
                remaining_roles.retain(|r| r != &role);
            }
        }
    }
    
    // Priority 2: Ontology constraints (animacy, etc.)
    for np in &np_candidates {
        if role_assignments.iter().any(|(_, n)| *n == np) {
            continue; // Already assigned
        }
        
        let animacy = np.features.animacy;
        for role in &remaining_roles {
            if ontology_allows(role, animacy, context.ontology) {
                role_assignments.push((*role, np));
                remaining_roles.retain(|r| r != role);
                break;
            }
        }
    }
    
    // Priority 3: Position-based heuristics
    // Subject (first NP) → Agent, Object (second NP) → Theme, etc.
    if !remaining_roles.is_empty() {
        let unassigned_nps: Vec<&&MorphAnalysis> = np_candidates.iter()
            .filter(|np| !role_assignments.iter().any(|(_, n)| *n == **np))
            .collect();
        
        for (i, role) in remaining_roles.iter().enumerate() {
            if let Some(np) = unassigned_nps.get(i) {
                role_assignments.push((*role, *np));
            }
        }
    }
    
    // Step 5: Build the Frame
    let frame = build_frame_from_assignments(frame_template, role_assignments)?;
    sentence.frames.push(frame);
    
    Ok(())
}

fn find_role_for_case(
    case: Case,
    available_roles: &[SemanticRole],
    context: &DeductionContext,
) -> Option<SemanticRole> {
    // Polish case-to-role mapping
    match case {
        Case::Nominative => {
            if available_roles.contains(&SemanticRole::Agent) {
                Some(SemanticRole::Agent)
            } else {
                None
            }
        }
        Case::Accusative => {
            if available_roles.contains(&SemanticRole::Theme) {
                Some(SemanticRole::Theme)
            } else if available_roles.contains(&SemanticRole::Patient) {
                Some(SemanticRole::Patient)
            } else {
                None
            }
        }
        Case::Dative => {
            if available_roles.contains(&SemanticRole::Recipient) {
                Some(SemanticRole::Recipient)
            } else if available_roles.contains(&SemanticRole::Experiencer) {
                Some(SemanticRole::Experiencer)
            } else {
                None
            }
        }
        Case::Instrumental => {
            if available_roles.contains(&SemanticRole::Instrument) {
                Some(SemanticRole::Instrument)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn ontology_allows(
    role: &SemanticRole,
    animacy: Option<Animacy>,
    ontology: &Ontology,
) -> bool {
    match role {
        SemanticRole::Agent | SemanticRole::Experiencer => {
            // Agents and experiencers must be animate
            animacy == Some(Animacy::Animate)
        }
        SemanticRole::Theme | SemanticRole::Patient => {
            // Themes and patients can be anything
            true
        }
        SemanticRole::Recipient => {
            // Recipients are usually animate
            animacy == Some(Animacy::Animate)
        }
        _ => true,
    }
}

fn build_frame_from_assignments(
    template: FrameTemplate,
    assignments: Vec<(SemanticRole, &MorphAnalysis)>,
) -> Result<Frame, DeductionError> {
    // Check if all required roles are filled
    for required_role in &template.required_roles {
        if !assignments.iter().any(|(r, _)| r == required_role) {
            return Err(DeductionError::MissingRequiredRole {
                role: *required_role,
                frame_type: template.frame_type.clone(),
            });
        }
    }
    
    // Build the frame
    match template.frame_type.as_str() {
        "Transfer" => {
            let agent = assignments.iter()
                .find(|(r, _)| *r == SemanticRole::Agent)
                .map(|(_, np)| np.to_entity())
                .unwrap();
            
            let recipient = assignments.iter()
                .find(|(r, _)| *r == SemanticRole::Recipient)
                .map(|(_, np)| np.to_entity())
                .unwrap();
            
            let theme = assignments.iter()
                .find(|(r, _)| *r == SemanticRole::Theme)
                .map(|(_, np)| np.to_entity())
                .unwrap();
            
            Ok(Frame::Transfer { agent, recipient, theme })
        }
        "Motion" => {
            let agent = assignments.iter()
                .find(|(r, _)| *r == SemanticRole::Agent)
                .map(|(_, np)| np.to_entity())
                .unwrap();
            
            let goal = assignments.iter()
                .find(|(r, _)| *r == SemanticRole::Goal)
                .map(|(_, np)| np.to_entity());
            
            let source = assignments.iter()
                .find(|(r, _)| *r == SemanticRole::Source)
                .map(|(_, np)| np.to_entity());
            
            Ok(Frame::Motion { agent, goal, source })
        }
        _ => Err(DeductionError::UnknownFrameType {
            frame_type: template.frame_type.clone(),
        }),
    }
}
```

**Example walkthrough:**
```
"Tomek dał jabłko Izie"

Step 1: Identify verb → "dał" (lemma: "dać")

Step 2: Look up frame → Transfer { agent, recipient, theme }

Step 3: Collect NPs:
  - Tomek (NOM, Masc, Animate)
  - jabłko (unknown case, Neuter, Inanimate)
  - Izie (DAT, Fem, Animate)

Step 4: Assign roles:
  Priority 1 (explicit case):
    - Tomek (NOM) → Agent ✓
    - Izie (DAT) → Recipient ✓
    - jabłko (unknown) → skip
  
  Priority 2 (ontology):
    - jabłko (Inanimate) → Theme ✓ (Theme allows anything)
  
  Result: All roles filled!

Step 5: Build Frame::Transfer {
    agent: Entity { concept: "person", name: "Tomek", ... },
    recipient: Entity { concept: "person", name: "Iza", ... },
    theme: Entity { concept: "apple", ... }
}
```

## 2. Case & Role Resolution

This is the most important step for Polish in v0.1.

### Detailed Algorithm

```rust
fn resolve_cases_and_roles(
    sentence: &mut Sentence,
    context: &DeductionContext,
) -> Result<(), DeductionError> {
    
    // For each frame in the sentence
    for frame in &mut sentence.frames {
        match frame {
            Frame::Transfer { agent, recipient, theme } => {
                // Resolve case for each role
                
                // Agent → Nominative (always)
                agent.features.case = Some(Case::Nominative);
                
                // Recipient → Dative (for Transfer frame)
                recipient.features.case = Some(Case::Dative);
                
                // Theme → Accusative (positive) or Genitive (negative)
                if sentence.polarity == Polarity::Negative {
                    theme.features.case = Some(Case::Genitive);
                } else {
                    theme.features.case = Some(Case::Accusative);
                }
            }
            
            Frame::Motion { agent, goal, source } => {
                agent.features.case = Some(Case::Nominative);
                
                if let Some(g) = goal {
                    g.features.case = Some(Case::Accusative);
                }
                
                if let Some(s) = source {
                    s.features.case = Some(Case::Genitive);
                }
            }
            
            Frame::Perception { experiencer, stimulus } => {
                experiencer.features.case = Some(Case::Nominative);
                stimulus.features.case = Some(Case::Accusative);
            }
            
            _ => {
                // Other frames handled similarly
            }
        }
    }
    
    // Validate with ontology
    for frame in &sentence.frames {
        validate_frame_with_ontology(frame, context.ontology)?;
    }
    
    Ok(())
}

fn validate_frame_with_ontology(
    frame: &Frame,
    ontology: &Ontology,
) -> Result<(), DeductionError> {
    match frame {
        Frame::Transfer { agent, recipient, theme } => {
            // Agent must be animate
            if agent.features.animacy != Some(Animacy::Animate) {
                return Err(DeductionError::SemanticTypeViolation {
                    role: SemanticRole::Agent,
                    expected: ConceptId("animate_entity"),
                    found: agent.concept.clone(),
                });
            }
            
            // Recipient must be animate
            if recipient.features.animacy != Some(Animacy::Animate) {
                return Err(DeductionError::SemanticTypeViolation {
                    role: SemanticRole::Recipient,
                    expected: ConceptId("animate_entity"),
                    found: recipient.concept.clone(),
                });
            }
            
            // Theme can be anything (no validation needed)
        }
        
        Frame::Perception { experiencer, stimulus } => {
            // Experiencer must be animate
            if experiencer.features.animacy != Some(Animacy::Animate) {
                return Err(DeductionError::SemanticTypeViolation {
                    role: SemanticRole::Experiencer,
                    expected: ConceptId("animate_entity"),
                    found: experiencer.concept.clone(),
                });
            }
            
            // Stimulus can be anything
        }
        
        _ => {}
    }
    
    Ok(())
}
```

### Case Resolution Rules (Polish)

**Rules (priority order):**

1. **Explicit case on the word wins**
   - If morphological analysis already determined the case, use it
   - Example: "Izie" is unambiguously Dative

2. **Verb subcat frame + position → assign case**
   - Each verb frame specifies required cases for each role
   - Example: Transfer frame requires NOM (Agent), DAT (Recipient), ACC (Theme)

3. **Ontology constraints**
   - Some roles have semantic restrictions
   - Example: Agent must be Animate, Recipient must be Animate

4. **Default heuristics**
   - Subject (first NP) → Nominative
   - Direct object (second NP) → Accusative
   - Indirect object (third NP) → Dative

5. **Polarity affects case**
   - Negative sentences: Accusative → Genitive
   - Example: "nie dał jabłka" (not "jabłko")

### Example: Complex Case Resolution

```
Input: "Student dał książkę profesorowi"

Morphological analysis:
  - Student (NOM, Masc, Animate)
  - dał (Verb, Past, Perf)
  - książkę (ACC, Fem, Inanimate)
  - profesorowi (DAT, Masc, Animate)

Case resolution:
  1. Explicit cases already determined by morphology
  2. Verify against frame requirements:
     - Transfer frame: Agent(NOM), Recipient(DAT), Theme(ACC)
     - Student (NOM) → Agent ✓
     - profesorowi (DAT) → Recipient ✓
     - książkę (ACC) → Theme ✓
  3. Ontology validation:
     - Agent (Student) is Animate ✓
     - Recipient (profesor) is Animate ✓
     - Theme (książka) can be anything ✓

Result: Valid frame assignment
```

### Handling Ambiguity

```
Input: "Jabłko leży na stole"

Morphological analysis:
  - Jabłko (NOM/ACC, Neuter, Inanimate) - AMBIGUOUS
  - leży (Verb, Present, Imperf)
  - na (Preposition)
  - stole (LOC, Masc, Inanimate)

Case resolution:
  1. "jabłko" is ambiguous (NOM or ACC)
  2. Look up verb "leżeć" (lie/be located):
     - Frame: Location { theme, location }
     - Theme requires NOM or ACC
     - Location requires LOC
  3. Position heuristic:
     - First NP → likely Theme (subject)
     - "stole" (LOC) → Location
  4. Assign:
     - jabłko (NOM) → Theme
     - stole (LOC) → Location

Result: Frame::Location { theme: jabłko, location: stół }
```

## 3. Pronoun Resolution (Within Sentence)

In v0.1 we only resolve pronouns **inside one sentence**.

**Simplified rules:**
- Reflexive (`się`, `sobie`, `himself`) → bind to subject of the same clause.
- 1st/2nd person → map to speaker/addressee (if known in context).
- 3rd person → most recent matching NP in the sentence (subject > object preference).

Full cross-sentence resolution is moved to v0.2+ (requires `Discourse`).

## 4. Temporal Anchoring

```rust
fn anchor_temporals(sentence: &mut Sentence, now: Timestamp) {
    if let Some(TemporalReference::Deictic { word, .. }) = &sentence.temporal {
        match word.as_str() {
            "wczoraj" | "yesterday" => {
                sentence.temporal = Some(TemporalReference::Relative {
                    offset: Duration::days(1),
                    anchor: TemporalAnchor::Now,
                });
            }
            // ... other cases
        }
    }
}
```

## 5. Ontology Validation + Inheritance

Before accepting the IL, we run:

```rust
ontology.validate_semantic_types(&frame)?;
ontology.inherit_features(&mut entity)?;
```

This catches semantic anomalies early (e.g. inanimate agent of "jeść").

## Error Handling in Deduction

```rust
pub enum DeductionError {
    /// Cannot assign case/role even with all rules
    UnresolvableCase { token: String, possible_roles: Vec<SemanticRole> },
    
    /// Ontology type violation
    SemanticTypeViolation { role: SemanticRole, expected: ConceptId, found: ConceptId },
    
    /// Ambiguous pronoun with no clear antecedent in sentence
    AmbiguousPronoun { pronoun: String },
}
```

In `best_effort` mode, some deduction errors become warnings instead of hard failures.

## Deduction State

During deduction, we track:

```rust
struct DeductionState {
    // Mutable during deduction, frozen after
    unresolved_pronouns: Vec<(String, FeatureBundle)>,
    ambiguous_cases: Vec<(String, Vec<Case>)>,
    
    // Filled during deduction
    resolved_refs: Vec<(String, EntityRef)>,
    frame_assignments: Vec<Frame>,
    
    // Final
    sentences: Vec<Sentence>,
}
```

After deduction completes, `unresolved_pronouns` and `ambiguous_cases` should be empty. If not, they are marked as `Unresolved` in the final Interlingua — the generation engine can then request clarification or make a best-effort choice.

See [INTERLINGUA.md](./INTERLINGUA.md#deduction-state) for more details.

## Integration Points

| Component          | How it uses Deduction                  |
|--------------------|----------------------------------------|
| `PolishEngine`     | Calls `core::deduction::deduce()` after surface parse |
| `EnglishEngine`    | Lighter version (mainly temporal + quantifier normalization) |
| `UniversalTranslator` | Runs deduction as part of `to_interlingua()` |
| `Ontology`         | Provides type constraints and inheritance |

## Future (v0.2+)

- Full discourse-based coreference resolver
- Quantifier scope resolution with heuristics + context
- Ellipsis reconstruction
- Speech act / intent inference (post-deduction)
