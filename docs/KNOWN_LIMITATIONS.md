# Known Limitations — lexFlex v0.1

**Note (2026-07-10):** Core LanguageDescriptor influence (has_articles, aspect_type, pro_drop, negation, temporal placement), RON-based morphology in parser/generator, and centralized verb_concept resolution via resolver+lexicon are now implemented and wired symmetrically in PL/EN generators per docs/ (LANGUAGE_DESCRIPTOR.md, GENERATOR.md). 

**Major remaining low-level limitations (full exhaustive list with code sites in lexFlex/ERRORS.MD — "Full Cross-Language (PL + EN) Non-Algorithmic Elements + Linguistic Theory Gaps Audit"):** surface contains/replaces for lemmas across generators/parsers/pipeline, crude degree stemmers, simplistic spelling-based articles, name-concat NP, missing PhonologyEngine, incomplete virile/clitics/government/suppletion/"być", ad-hoc feature propagation, duplicated normalization. All docs (including this one) updated in parallel. See ERRORS.MD for required engines (Phonology, Analyzer, Unifier, NP structure, etc.) and RON proposals. Remaining limitations below are accurate for v0.1 scope but the linguistic engineering debt is detailed in ERRORS.MD.

This document lists known limitations and constraints of lexFlex v0.1. Understanding these limitations is crucial for setting realistic expectations and planning future improvements.

## Scope Limitations

### Supported Language Pairs

**v0.1:** Polish ↔ English only

**Limitations:**
- No support for other Slavic languages (Czech, Slovak, Russian)
- No support for Germanic languages (German, Dutch)
- No support for Romance languages (French, Spanish, Italian)
- No support for Asian languages (Chinese, Japanese, Korean)

**Workaround:** None in v0.1. Wait for v0.2+ which will add language plugin system.

### Supported Sentence Types

**v0.1:** Simple declarative sentences only

**Supported:**
- Simple SVO sentences: "Tomek dał jabłko Izie"
- Sentences with temporal modifiers: "Tomek dał jabłko Izie wczoraj"
- Negative sentences: "Tomek nie dał jabłka Izie"
- Simple questions (basic): "Czy Tomek dał jabłko Izie?"

**Not Supported:**
- ❌ Complex sentences with subordinate clauses
- ❌ Relative clauses: "Tomek, który dał jabłko Izie, jest moim bratem"
- ❌ Conditional sentences: "Gdyby Tomek dał jabłko Izie, byłaby szczęśliwa"
- ❌ Reported speech: "Powiedział, że Tomek dał jabłko Izie"
- ❌ Imperative mood: "Daj jabłko Izie!"
- ❌ Exclamations: "Jakie piękne jabłko!"

**Workaround:** Break complex sentences into simple sentences before translation.

### Supported Semantic Frames

**v0.1:** 5 core frames

**Supported:**
- ✅ Transfer: give, send, hand
- ✅ Motion: go, come, walk
- ✅ Perception: see, hear, notice
- ✅ Consumption: eat, drink
- ✅ Destruction: break, destroy

**Not Supported:**
- ❌ Creation: make, build, create
- ❌ Communication: say, tell, ask
- ❌ Cognition: think, know, believe
- ❌ Emotion: love, hate, fear
- ❌ Social: marry, divorce, meet
- ❌ Possession: have, own, possess
- ❌ Location: be, stay, live

**Workaround:** Use Transfer frame as approximation (e.g., "Tomek ma książkę" → "Tomek otrzymał książkę").

## Morphological Limitations

**Progress (2026-07-10 iteration, see UNIFIED pipeline doc):** Cases algorithmic via RON rules + common pipeline. Degree/comparatives and full exceptions still aspirational (in design/docs but not wired in code). Lists/enumerations and number effects on case partially in design, need implementation for full algorithmic coverage. Consistency two-way with parser/deduction is a focus of current iteration.

### Polish Morphology

**Supported (algorithmic where possible):**
- ✅ 4 noun paradigms (neuter -o, feminine -a, masculine consonant, masculine animate) — via rules
- ✅ 2 verb paradigms (-ać, -eć) — via rules
- ✅ Basic case inflection (7 cases) — algorithmic in morph + pipeline
- ✅ Aspect pairs (perfective/imperfective)
- ✅ Tense inflection (past, present, future)

**Not Supported / Partial:**
- ❌ Irregular nouns: "człowiek" → "ludzie" (plural) — exceptions planned but not wired
- ❌ Irregular verbs: "być", "mieć", "iść"
- ❌ Comparative/superlative adjectives: "dobry" → "lepszy" → "najlepszy" (design only)
- ❌ Adverbs from adjectives: "szybki" → "szybko"
- ❌ Numerals and quantifiers with case effects: "trzy książki", "30 jabłek" (Numerical exists, case adjustment in design)
- ❌ Participles: "dający", "dany"
- ❌ Gerunds: "dając", "dawszy"
- ❌ Full lists/enumerations algorithmic (basic "i" in some places)

**Impact:** Sentences with irregular forms will fail or produce incorrect output.

**Workaround:** 
- Avoid irregular forms
- Use regular synonyms when possible
- Pre-process text to normalize irregular forms

### English Morphology

**Supported:**
- ✅ Regular plural nouns: "book" → "books"
- ✅ Regular verb conjugation: "give" → "gives", "gave"
- ✅ Basic tense forms (simple past, present, future)
- ✅ Progressive aspect: "is giving"

**Not Supported:**
- ❌ Irregular verbs: "go" → "went" (not "goed") — many covered via lexicon
- ❌ Irregular plurals: "child" → "children"
- ❌ Perfect aspect: "has given"
- ❌ Modal verbs: "can give", "must give"

**Impact:** English output may be grammatically incorrect for irregular forms.

**Workaround:**
- Use regular verb forms when possible
- Post-process output to fix common irregularities
- Add irregular forms to lexicon manually

## Syntactic Limitations

### Word Order

**Supported:**
- ✅ SVO order (Subject-Verb-Object)
- ✅ Basic adverb placement (temporal at end)
- ✅ Polish flexible word order (with constraints)

**Not Supported:**
- ❌ Question word order: "What did Tomek give?"
- ❌ Topicalization: "Jabłko dał Tomek Izie"
- ❌ Focus movement: "To Izie dał Tomek jabłko"
- ❌ Inversion: "Never have I seen..."
- ❌ Verb-second (V2) in German

**Impact:** Questions and emphasized sentences will have incorrect word order.

**Workaround:**
- Convert questions to declarative form before translation
- Avoid topicalization and focus movement
- Use SVO order consistently

### Coordination

**Supported:**
- ✅ Simple coordination: "Tomek i Iza"
- ✅ Sentence coordination: "Tomek dał jabłko i Iza dała książkę"

**Not Supported:**
- ❌ Gapping: "Tomek dał jabłko, a Iza książkę"
- ❌ Right node raising: "Tomek lubi i Iza kocha jabłka"
- ❌ Correlative coordination: "Zarówno Tomek, jak i Iza"

**Impact:** Coordinated sentences may be incomplete or incorrect.

**Workaround:**
- Expand gapped coordinations to full sentences
- Avoid complex coordination patterns

## Semantic Limitations

### Pronoun Resolution

**Supported:**
- ✅ Reflexive pronouns within sentence: "Tomek widzi się"
- ✅ 1st/2nd person pronouns: "ja", "ty"
- ✅ 3rd person pronouns (simple): "on", "ona", "ono"

**Not Supported:**
- ❌ Cross-sentence coreference: "Tomek przyszedł. On dał jabłko."
- ❌ Ambiguous pronouns: "Tomek i Marek rozmawiali. On dał jabłko."
- ❌ Cataphora: "Zanim on przyszedł, Tomek dał jabłko."
- ❌ Discourse deixis: "To było wczoraj."

**Impact:** Pronouns may be resolved incorrectly or left unresolved.

**Workaround:**
- Replace pronouns with explicit nouns before translation
- Keep sentences short to minimize pronoun use
- Use proper names instead of pronouns

### Temporal Expressions

**Supported:**
- ✅ Simple temporal adverbs: "wczoraj", "dzisiaj", "jutro"
- ✅ Basic tense markers

**Not Supported:**
- ❌ Complex temporal expressions: "w zeszły wtorek", "za trzy dni"
- ❌ Relative temporal: "kiedy przyszedł", "zanim dał"
- ❌ Duration: "przez godzinę", "od wczoraj"
- ❌ Frequency: "codziennie", "co tydzień"

**Impact:** Complex temporal expressions will be ignored or translated incorrectly.

**Workaround:**
- Normalize temporal expressions to simple forms before translation
- Avoid relative temporal expressions
- Use explicit dates instead of relative references

### Quantification

**Supported:**
- ✅ Implicit universal: "Tomek dał jabłko Izie" (specific entities)
- ✅ Explicit quantifiers (universal, existential, negated, proportional, numerical): "wszyscy jadł jabłko", "nikt widział", "some drank milk" etc. (see tests and benchmark)

**Not Supported:**
- ❌ Numerals with agreement in complex NPs: "trzy książki" (basic numerical supported via quant)
- ❌ Full scope ambiguity across clauses

**Impact:** Most simple quantified sentences now work; complex scoping may need workarounds.

**Workaround:**
- For advanced scope, break sentences.

## Generation Limitations

### Naturalness

**Supported:**
- ✅ Grammatically correct sentences
- ✅ Basic word order
- ✅ Correct morphological forms

**Not Supported:**
- ❌ Natural-sounding output (may be stiff)
- ❌ Idiomatic expressions: "dać komuś znać" → "let someone know"
- ❌ Collocations: "mocna kawa" → "strong coffee"
- ❌ Register/style variation: formal vs informal
- ❌ Discourse markers: "no", "well", "you know"

**Impact:** Output may be grammatically correct but sound unnatural or robotic.

**Workaround:**
- Use post-processing with LLM to improve naturalness (v0.2+)
- Accept slightly stiff output in v0.1
- Focus on correctness over naturalness

### Articles (English)

**Supported:**
- ✅ Basic article selection: "a" vs "the"
- ✅ First mention vs subsequent mention

**Not Supported:**
- ❌ Generic vs specific: "Dogs are loyal" vs "The dog is loyal"
- ❌ Mass nouns: "water" (no article) vs "a water" (bottle)
- ❌ Idiomatic article use: "go to school" vs "go to the school"

**Impact:** English articles may be incorrect in some contexts.

**Workaround:**
- Post-process output to fix article errors
- Use simple sentences where article choice is clear
- Add article rules to generator incrementally

## Performance Limitations

### Processing Speed

**v0.1:** ~100ms per simple sentence

**Limitations:**
- Morphology lookup: O(n) where n = number of paradigms
- Deduction: O(n²) where n = number of NPs in sentence
- Generation: O(n) where n = number of roles in frame

**Impact:** Complex sentences with many NPs may be slow.

**Workaround:**
- Keep sentences short (< 20 words)
- Limit number of NPs per sentence (< 5)
- Cache morphology lookups

### Memory Usage

**v0.1:** ~50MB for full system

**Components:**
- Ontology: ~10MB
- Lexicons (PL + EN): ~20MB
- Morphology rules: ~5MB
- Runtime structures: ~15MB

**Impact:** Not suitable for memory-constrained environments.

**Workaround:**
- Load only required language components
- Use streaming for large texts
- Consider v0.2+ with optimized data structures

## Error Handling Limitations

### Error Recovery

**Supported:**
- ✅ Unknown words (with suggestions)
- ✅ Ambiguous morphology (choose first)
- ✅ Missing lexical entries (use concept name)
- ✅ Semantic type violations (warn and continue)

**Not Supported:**
- ❌ Automatic error correction
- ❌ Intelligent fallback strategies
- ❌ User feedback integration
- ❌ Learning from errors

**Impact:** Errors may produce suboptimal output without user awareness.

**Workaround:**
- Use best-effort mode with warnings
- Review warnings and fix input manually
- Pre-validate input before translation

## Testing Limitations

### Test Coverage

**v0.1:** ~80% code coverage

**Not Covered:**
- ❌ Edge cases with irregular forms
- ❌ Complex error scenarios
- ❌ Performance edge cases
- ❌ Concurrent usage

**Impact:** Some bugs may not be caught in testing.

**Workaround:**
- Add tests for discovered bugs
- Use property-based testing for robustness
- Monitor production usage for issues

## Comparison with Other Systems

### vs Rule-Based MT (Apertium, Moses)

**lexFlex advantages:**
- ✅ Semantic understanding (frames, roles)
- ✅ Better handling of word order variation
- ✅ Explicit semantic representation

**lexFlex disadvantages:**
- ❌ Much smaller vocabulary (~500 concepts vs millions)
- ❌ Less mature morphological analyzers
- ❌ No statistical components

### vs Neural MT (Google Translate, DeepL)

**lexFlex advantages:**
- ✅ Explicit semantic representation
- ✅ Better control over translation process
- ✅ Easier to debug and extend
- ✅ Lower resource requirements

**lexFlex disadvantages:**
- ❌ Much lower quality output
- ❌ Limited vocabulary and grammar
- ❌ No learning from data
- ❌ Requires manual rule creation

## Roadmap for Improvements

### v0.2 (Planned)
- Add 5 more semantic frames (Creation, Communication, Cognition, Emotion, Social)
- Support irregular verbs and nouns
- Add cross-sentence coreference
- Support simple questions and imperatives
- Add 3 more languages (German, French, Spanish)

### v0.3 (Planned)
- Support complex sentences (subordinate clauses)

- Add discourse markers
- Improve naturalness with optional LLM post-processing

### v1.0 (Target)
- Full support for all major sentence types
- Complete morphological coverage
- 10+ languages
- Quality approaching commercial MT systems
- Production-ready performance

## When to Use lexFlex v0.1

### Good Use Cases
- ✅ Simple sentence translation (PL ↔ EN)
- ✅ Learning about MT architecture
- ✅ Research on semantic representation
- ✅ Prototyping MT systems
- ✅ Educational purposes

### Poor Use Cases
- ❌ Production translation services
- ❌ Complex document translation
- ❌ Real-time conversation translation
- ❌ Literary translation
- ❌ Technical/scientific translation

## Conclusion

lexFlex v0.1 is a **research prototype** demonstrating a semantic-based approach to machine translation. It has significant limitations compared to commercial MT systems, but provides a solid foundation for future improvements.

**Key takeaways:**
1. Use for simple sentences only
2. Expect grammatically correct but possibly stiff output
3. Be aware of morphological and syntactic limitations
4. Use error handling modes appropriately
5. Plan for v0.2+ for production use

## References

- [IMPLEMENTATION_GUIDE.md](./IMPLEMENTATION_GUIDE.md) - Implementation details
- [ERROR_HANDLING_GUIDE.md](./ERROR_HANDLING_GUIDE.md) - Error handling strategies
- [IMPLEMENTATION_ROADMAP.md](./IMPLEMENTATION_ROADMAP.md) - Future improvements
- [GENERATOR.md](./GENERATOR.md) - Generator limitations
- [DEDUCTION.md](./DEDUCTION.md) - Deduction limitations
