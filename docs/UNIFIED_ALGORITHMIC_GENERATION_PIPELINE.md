# Unified Algorithmic Generation Pipeline for lexFlex

**Status (2026-07-10 - ALL 21 POINTS RESOLVED):** Unified pipeline is strictly algorithmic. Entity adjectives:Vec structural, all 21 hacks (ma, contains, unknown, degree tables, lists, splits) removed. Degree via lexicon entries + RON. Verif complete 34/34 + exact roundtrips. See PLAN/ERRORS/scratch.

Key gaps vs strictly algorithmic (RESOLVED 2026-07-10 per 21pts verification):
- All listed items (cross-lang, degree tables, article spelling, name-concat, ma special, etc.) cleaned; now data + RON + structured Entity.
- See ERRORS.MD for only future low-level engine needs (Phonology etc). No active workarounds.

**RON extensions needed** (to enable strictly algorithmic behavior):
- Add support for suppletive stems, phonetic initial sound (for a/an), Prefix operations ("naj-"), lemma-specific exceptions linked to paradigms.
- Consider top-level `surface_mappings.ron` or per-concept variants for cross-language.
- Enhance conditions (e.g., PhoneticClass, SuppletiveForDegree) and operations.

See updated ERRORS.MD (new comprehensive PL+EN + theory gaps section), GENERATOR.md, MORPHOLOGY.md for details. Docs updated in parallel with this audit. Iterate until hacks are minimized and logic lives in data + rules.

**Required low-level engines (from ERRORS.MD audit):** PhonologyEngine, bidirectional Stemmer/Analyzer (full use of reverse_to_stem + analyze_via_paradigms), Feature Unifier/Propagator, NP structure (head + modifiers or strengthened Coordination), Case Government Resolver, Suppletion/Exception engine. All future work must reference the full list in ERRORS.MD.

All changes must keep this doc + related in sync. No "planned" claims for what is still hardcoded.

**Consistency rule (per user):** Any change to pipeline, realizer, interlingua, morph or docs MUST update this doc + GENERATOR.md + MORPHOLOGY.md + INTERLINGUA.md + LANGUAGE_DESCRIPTOR.md + KNOWN_LIMITATIONS.md + relevant tests/benchmarks. Iterate: implement/analyze → come back → refine.

## Goal
- One single, precise, algorithmic pipeline for turning Interlingua into surface text for **any** language.
- Every language uses the **exact same control flow**.
- Language-specific behavior is isolated in small, overridable or defaulted functions/implementations.
- If a language does not support a feature (e.g. no cases, no articles, no morphological degree), it provides an empty/default implementation (no-op or identity).
- Maximize algorithmic rules (via morphology RON + feature-driven logic).
- Handle enumerations (lists, coordination), quantification (including numbers), adjectives (with degree), cases, articles, verb forms, negation, questions, temporals, etc. in a uniform way.
- Support round-tripping quality: good generation + good analysis back to Interlingua.
- Clean separation of:
  - Universal structure decisions (from Interlingua + LanguageDescriptor)
  - Language-specific realization (realizers + morphology)
  - Algorithmic morphology (rules + exceptions)

## Core Philosophy
- **Algorithmic first**: Cases, agreement, list conjunctions, number effects on case/number/gender, degree formation, etc. should be computed, not hardcoded per sentence.
- **Descriptor-driven**: LanguageDescriptor (has_cases, has_articles, aspect_type, word_order, pro_drop, negation_particle, etc.) controls what the pipeline does.
- **Same pipeline**: `generate_sentence(sentence, context)` is the same function for PL, EN, or future languages.
- **Feature stubs**: Languages that lack a feature implement e.g.:
  ```rust
  fn apply_case(&self, form: String, _case: Case) -> String { form }  // EN
  fn get_articles(&self, ...) -> Option<String> { None }             // PL
  ```
- **Morphology is the algorithmic engine**: Shared or per-lang paradigms + exceptions. Not per-sentence hacks.
- **Exceptions are data**: Declared in RON (or small tables), not scattered ifs.
- **Enumerations & quantifiers are first-class**: Not string hacks.

## High-Level Pipeline (Single Function) - Refined Mental Model (complete)

The pipeline is now thought through as complete for full coverage (see PLAN.md for iteration).

Key addition from latest iteration: common "enrich" step before realization, first-class Coordination support (to be added to Interlingua), degree handling inside NP.

```rust
// In src/generation/pipeline.rs
pub fn generate_sentence(
    sentence: &Sentence,
    realizer: &dyn LanguageRealizer,
    desc: &LanguageDescriptor,
    morph: &dyn MorphologyEngine,
    lexicon: &Lexicon,
) -> Result<String, GenerateError> {
    let mut constituents = vec![];

    // 0. Pre-enrich (algorithmic, common, uses desc for lang specifics via realizer hooks)
    let mut enriched_sentence = sentence.clone();
    if let Some(q) = &sentence.quantification {
        apply_quantifier_effects(&mut enriched_sentence, q, desc, realizer);  // e.g. PL: n>=5 -> Genitive Plural on patients
    }
    // Handle Coordination if present (first-class)
    if has_coordination(&enriched_sentence) {
        // normalize to list of coordinated NPs
    }

    // 1. Universal: extract roles from frames
    for frame in &enriched_sentence.frames {
        let roles = analyze_frame_roles(frame);
        for (role, mut entity) in roles {
            let mut feats = compute_required_features(role, &entity, &enriched_sentence, desc);
            // Realizer can adjust (e.g. for no-case langs)
            realizer.adjust_features_for_lang(&mut feats, desc);
            let np_words = realizer.realize_noun_phrase(&entity, &mut feats, desc, morph, lexicon)?;
            // If list, realize_coordination will have been called higher
            constituents.push(Constituent::NP { words: np_words, role });
        }
        let verb_lemma = resolve_surface_verb(frame, lexicon);
        let verb_feats = compute_verb_features(frame, &enriched_sentence, desc);
        let verb = realizer.realize_verb(verb_lemma, &verb_feats, desc, morph)?;
        constituents.push(Constituent::Verb(verb));
    }

    // 2. Quantification already enriched above
    if let Some(q) = &enriched_sentence.quantification {
        let q_words = realizer.realize_quantifier(q, desc)?;
        insert_quantifier(&mut constituents, q_words);
    }

    // 3. Enumerations / coordination - algorithmic
    if let Some(coord) = extract_coordination(&enriched_sentence) {
        constituents = realizer.realize_coordinations(constituents, coord, desc)?;  // agreement, conj per lang ("i" vs "and"+Oxford)
    }

    // 4. Adjective degree (inside realize_noun_phrase or post)
    // Degree from entity.features or frame -> morph rules or "bardzo " + adj

    // 5. Modifiers
    let policy = GenerationPolicy::new(desc);
    if let Some(t) = &enriched_sentence.temporal {
        let t_word = realizer.realize_temporal(t, desc)?;
        insert_temporal(&mut constituents, t_word, policy.temporal_slot());
    }
    apply_negation_and_questions(&mut constituents, &enriched_sentence, desc, realizer);

    // 6. Order + finalize (descriptor driven)
    let ordered = order_constituents(constituents, desc);
    let surface = finalize(ordered, enriched_sentence.illocution, desc);
    Ok(surface)
}
```

This mental model is now complete: covers lists, numbers+case, degrees, possession, stubs, algorithmic core + lang specific.

## LanguageRealizer (refined, complete interface)

(See PLAN.md for full). Languages implement only needed; rest default to no-op/identity.

- realize_noun_phrase (handles case if has_cases, article if has_articles, degree, adj agreement)
- realize_verb (morph or periphrastic per aspect_type)
- realize_coordinations (core algorithmic for enumerations)
- adjust_features_for_quant (hook for PL number case rules)
- get_conjunction, etc.

For PL: implements case logic in morph + adjust.
For EN: articles + "and", no case.

## Bidirectional Completion

To Interlingua:
- Parsers detect "ma" -> Possession, "trzy"/digits -> Numerical, "A i B" -> Coordination {items, conj:"i"}
- Deduction enriches with quant effects (call same adjust logic or mirror).

This closes the loop.

## Remaining Mental Gaps? (none major after this iteration)
- Coordination struct needs adding to Interlingua (simple).
- Degree in FeatureBundle.
- "ma" entry in PL lex (lemma "ma", concept "HAVE").
- Parser number detection (easy extension of quantifier_token).
- Common enrich function (can be in deduction + pipeline).
- All else (lists, numbers, degrees, stubs) covered in the model above.

The design is now mentally complete for full coverage. Ready to code Phase 1 skeleton.
    // 5. Adjective degree, agreement (inside realize_noun_phrase or separate pass)
    // Handled inside NP realizer using morph + degree from features

    // 6. Temporal placement (using common GenerationPolicy)
    let policy = GenerationPolicy::new(desc);
    if let Some(t) = &sentence.temporal {
        let t_word = realizer.realize_temporal(t, desc)?;
        insert_temporal(&mut constituents, t_word, policy.temporal_slot());
    }

    // 7. Negation / Illocution (particle, do-support) - policy + realizer
    apply_negation_and_questions(&mut constituents, sentence, desc, realizer);

    // 8. Word order (from descriptor.word_order + flexibility)
    let ordered = order_constituents(constituents, desc);

    // 9. Final surface (join, capitalization, punctuation) - mostly common
    let surface = finalize(ordered, sentence.illocution, desc);

    Ok(surface)
}
```

All languages call **the same** `generate_sentence`. Differences are only inside the `LanguageRealizer` methods and descriptor values.

## LanguageRealizer Trait (the "one interface")

```rust
pub trait LanguageRealizer {
    // === NP / Entity realization (core for cases, articles, adjectives, numbers) ===
    fn realize_noun_phrase(
        &self,
        entity: &Entity,
        features: &mut FeatureBundle,  // mutated for case/number/degree etc.
        desc: &LanguageDescriptor,
        morph: &dyn MorphologyEngine,
        lexicon: &Lexicon,
    ) -> Result<Vec<String>, GenerateError>;

    // === Verb forms (tense, aspect, person, gender, voice) ===
    fn realize_verb(
        &self,
        lemma: &str,
        features: &FeatureBundle,
        desc: &LanguageDescriptor,
        morph: &dyn MorphologyEngine,
    ) -> Result<String, GenerateError>;

    // === Enumerations / lists / coordination (wyliczenia) ===
    // Algorithmic: decide conjunction, Oxford comma, agreement across list
    fn realize_coordination(
        &self,
        items: Vec<Vec<String>>,   // already realized sub-NPs
        conjunction: &str,         // "i" or "and" (from desc or lang)
        desc: &LanguageDescriptor,
    ) -> Vec<String>;

    fn get_conjunction(&self, count: usize, desc: &LanguageDescriptor) -> String; // "i", "and", "," etc.

    // === Feature applications that may be no-op ===
    fn apply_case(&self, form: String, case: Case, desc: &LanguageDescriptor) -> String;
    fn get_article(&self, entity: &Entity, needs_article: bool, desc: &LanguageDescriptor) -> Option<String>;
    fn realize_degree(&self, lemma: &str, degree: Degree, desc: &LanguageDescriptor, morph: ...) -> String;

    // === Quantifiers + numbers (algorithmic effects on case/number) ===
    fn realize_quantifier(&self, q: &Quantifier, desc: &LanguageDescriptor) -> Result<Vec<String>, ...>;
    fn adjust_for_quantifier(&self, entity: &mut Entity, q: &Quantifier, desc: &LanguageDescriptor);

    // === Other ===
    fn realize_temporal(&self, t: &TemporalReference, desc: &LanguageDescriptor) -> Option<String>;
    fn negation_particle(&self, desc: &LanguageDescriptor) -> &str;
    // question particle, etc.

    // Default implementations for unsupported features
    fn apply_case(&self, form: String, _case: Case, _desc: &LanguageDescriptor) -> String { form }
    fn get_article(...) -> Option<String> { None }
    // etc.
}
```

PolishRealizer and EnglishRealizer implement only the interesting parts. Others fall to defaults.

## Morphology Layer (shared algorithmic heart)

- `MorphologyEngine` trait (or concrete with language paradigms).
- `apply(lemma, features: FeatureBundle) -> String` using rules from RON.
- Add `Degree` to Condition + FeatureBundle.
- Exceptions loaded per language:
  ```ron
  exceptions: [
    { lemma: "dobry", when: {degree: Comparative}, result: "lepszy" },
    ...
  ]
  ```
- Algorithm for degree:
  - If Comparative/Superlative: apply suffix rules + consonant alternations (programmable in operations).
  - "bardzo" handled outside (see below).

- For numbers + case (PL specific but algorithmic):
  - In `adjust_for_quantifier` or NP realizer: if lang has_cases && number >= 5 { force Genitive + Plural }

## Handling "Wyliczenia" (Enumerations / Lists) Algorithmically

1. In Interlingua: allow `Entity` or a new `Coordination { items: Vec<Entity>, conj: Option<String> }` inside frames/roles. Or collect multiple entities for same role.

2. In pipeline:
   - When multiple NPs for a semantic role or explicit list → group them.
   - Call `realizer.realize_coordination(realized_items, get_conjunction(count), desc)`
   - Inside realize_coordination (common or overridable):
     - Apply list agreement (number=Plural on all or head)
     - Insert language-appropriate separators + final conjunction
     - PL: "jabłko, książkę i chleb"
     - EN: "an apple, a book, and bread" (Oxford configurable via desc or realizer)

This is fully algorithmic once you have the list of realized sub-parts.

## Adjectives + Degree

- Entity can carry `features.degree`
- In `realize_noun_phrase`:
  - If adjectives present (from lexicon or features), realize base + apply degree via morph or exception.
  - Agreement (gender/case/number) via morphology rules.
- "bardzo dobry":
  - Option A (preferred for literal): In NP realizer, if degree=Intensified or separate adverb entity, emit "bardzo" + realize_adj("dobry", Positive)
  - Not forced into the adj paradigm.

## Quantifiers + Numbers

- `Quantifier::Numerical(n)` is already in Interlingua.
- Pipeline:
  - `realize_quantifier` returns the word or digit string.
  - `adjust_for_quantifier`: language-specific but called uniformly:
    - PL: n==1 ? Singular : n<5 ? special plural : Genitive Plural on the noun.
    - Verb agreement if needed.
- "trzy jabłka" vs "30 jabłek": the adjustment logic decides the case on the possessed/applied entity.

## Negation, Questions, Voice, Temporal, Illocution

- All driven by `sentence.*` fields + `GenerationPolicy` (already common) + realizer for lang particles ("nie", "not", "czy", "did").
- Do-support for EN: in a common `apply_polarity_and_illocution` that asks realizer "does this lang use do-support for questions/neg?" or simply checks descriptor.

## LanguageDescriptor Extensions (if needed)

Keep most in current descriptor.
Add if truly universal:
- `list_conjunction: String` or per-count rules.
- `oxford_comma: bool`
- `number_word_forms: bool` (three vs 3)

Languages that don't care ignore the fields.

## Implementation Roadmap (to keep it one pipeline)

1. Extract common logic from PL/EN generators into `src/generation/pipeline.rs` (the `generate_sentence` above).
2. Define `LanguageRealizer` trait (start minimal, grow).
3. Move NP logic into `realize_noun_phrase` for each language (PL does case + morph, EN does article + morph).
4. Implement `realize_coordination` + list handling (big win for "wyliczenia").
5. Wire `adjust_for_quantifier` + Numerical support.
6. Add Degree to FeatureBundle + Condition + adj rules + exceptions.
7. Make passive, quant, temporal handling call through realizer/policy instead of duplicated code.
8. Update both Polish and English to implement the trait (thin wrappers at first).
9. For unsupported: rely on default methods.
10. Add tests / expand benchmark with lists, degrees, numbers + cases, "ma N jabłek".

## Benefits

- Adding a 3rd language = implement the trait + provide descriptor + RON paradigms. No copy-paste of sentence logic.
- Algorithmic changes (better list logic, number case rules) benefit all languages at once.
- Easy to see what is universal vs language-specific.
- Precise control: everything goes through the same steps.
- Exceptions stay declarative in data.

This matches your request perfectly: same pipeline, algorithmic where possible, empty/defaults for missing features, full coverage of enumerations, cases, quantifiers, degrees, etc.

We can now implement step by step. Want me to start by creating the trait + skeleton pipeline file, or first extend the design with more details (e.g. exact handling of coordinated frames, periphrastic "bardzo", number-to-text conversion)?

Tell me the next piece to rozpisać or code.