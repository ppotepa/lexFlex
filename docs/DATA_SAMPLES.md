# Data Samples - Example RON Files

This document provides complete examples of RON data files for lexFlex. Use these as templates for creating your own data.

---

## Concepts (data/concepts/concepts.ron)

```ron
// Master concept definitions (~500 for full system, 50 for MVP)

[
    // === ENTITIES ===
    Concept(
        id: "PERSON",
        frame_type: None,
        roles: ["Agent", "Recipient", "Experiencer"],
        inherent_features: FeatureBundle(
            animacy: Some(Animate),
            countability: Some(Count),
        ),
    ),
    Concept(
        id: "APPLE",
        frame_type: None,
        roles: ["Theme", "Patient"],
        inherent_features: FeatureBundle(
            gender: Some(Neuter),
            countability: Some(Count),
        ),
    ),
    Concept(
        id: "BOOK",
        frame_type: None,
        roles: ["Theme", "Patient"],
        inherent_features: FeatureBundle(
            gender: Some(Feminine),
            countability: Some(Count),
        ),
    ),
    Concept(
        id: "HOUSE",
        frame_type: None,
        roles: ["Location", "Theme"],
        inherent_features: FeatureBundle(
            gender: Some(Masculine),
            countability: Some(Count),
        ),
    ),
    Concept(
        id: "WATER",
        frame_type: None,
        roles: ["Theme", "Patient"],
        inherent_features: FeatureBundle(
            gender: Some(Neuter),
            countability: Some(Mass),
        ),
    ),
    Concept(
        id: "MOTHER",
        frame_type: None,
        roles: ["Agent", "Recipient"],
        inherent_features: FeatureBundle(
            gender: Some(Feminine),
            animacy: Some(Animate),
        ),
    ),
    Concept(
        id: "FATHER",
        frame_type: None,
        roles: ["Agent", "Recipient"],
        inherent_features: FeatureBundle(
            gender: Some(Masculine),
            animacy: Some(Animate),
        ),
    ),
    Concept(
        id: "FRIEND",
        frame_type: None,
        roles: ["Agent", "Recipient"],
        inherent_features: FeatureBundle(
            animacy: Some(Animate),
        ),
    ),
    Concept(
        id: "CHILD",
        frame_type: None,
        roles: ["Agent", "Recipient", "Theme"],
        inherent_features: FeatureBundle(
            gender: Some(Neuter),
            animacy: Some(Animate),
        ),
    ),
    Concept(
        id: "CAT",
        frame_type: None,
        roles: ["Agent", "Theme"],
        inherent_features: FeatureBundle(
            gender: Some(Masculine),
            animacy: Some(Animate),
        ),
    ),
    Concept(
        id: "DOG",
        frame_type: None,
        roles: ["Agent", "Theme"],
        inherent_features: FeatureBundle(
            gender: Some(Masculine),
            animacy: Some(Animate),
        ),
    ),
    Concept(
        id: "MILK",
        frame_type: None,
        roles: ["Theme", "Patient"],
        inherent_features: FeatureBundle(
            gender: Some(Neuter),
            countability: Some(Mass),
        ),
    ),
    Concept(
        id: "BREAD",
        frame_type: None,
        roles: ["Theme", "Patient"],
        inherent_features: FeatureBundle(
            gender: Some(Neuter),
            countability: Some(Mass),
        ),
    ),
    Concept(
        id: "CAR",
        frame_type: None,
        roles: ["Theme", "Instrument", "Location"],
        inherent_features: FeatureBundle(
            gender: Some(Neuter),
            countability: Some(Count),
        ),
    ),
    Concept(
        id: "CITY",
        frame_type: None,
        roles: ["Location", "Goal", "Source"],
        inherent_features: FeatureBundle(
            gender: Some(Feminine),
            countability: Some(Count),
        ),
    ),

    // === ACTIONS ===
    Concept(
        id: "GIVE",
        frame_type: Some("Transfer"),
        roles: ["Agent", "Recipient", "Theme"],
        inherent_features: FeatureBundle(
            tense: None,
            aspect: None,
        ),
    ),
    Concept(
        id: "TAKE",
        frame_type: Some("Transfer"),
        roles: ["Agent", "Theme", "Source"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "GO",
        frame_type: Some("Motion"),
        roles: ["Agent", "Goal", "Source"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "COME",
        frame_type: Some("Motion"),
        roles: ["Agent", "Goal"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "SEE",
        frame_type: Some("Perception"),
        roles: ["Experiencer", "Stimulus"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "HEAR",
        frame_type: Some("Perception"),
        roles: ["Experiencer", "Stimulus"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "THINK",
        frame_type: Some("Cognition"),
        roles: ["Experiencer", "Content"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "KNOW",
        frame_type: Some("Cognition"),
        roles: ["Experiencer", "Content"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "LOVE",
        frame_type: Some("Emotion"),
        roles: ["Experiencer", "Stimulus"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "HATE",
        frame_type: Some("Emotion"),
        roles: ["Experiencer", "Stimulus"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "EAT",
        frame_type: Some("Consumption"),
        roles: ["Agent", "Patient"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "DRINK",
        frame_type: Some("Consumption"),
        roles: ["Agent", "Patient"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "READ",
        frame_type: Some("Perception"),
        roles: ["Agent", "Theme"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "WRITE",
        frame_type: Some("Creation"),
        roles: ["Agent", "Theme", "Instrument"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "BUY",
        frame_type: Some("Transfer"),
        roles: ["Agent", "Theme", "Source"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "SELL",
        frame_type: Some("Transfer"),
        roles: ["Agent", "Theme", "Recipient"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "MAKE",
        frame_type: Some("Creation"),
        roles: ["Agent", "Theme", "Instrument"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "BREAK",
        frame_type: Some("Destruction"),
        roles: ["Agent", "Patient", "Instrument"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "SAY",
        frame_type: Some("Communication"),
        roles: ["Agent", "Addressee", "Message"],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "ASK",
        frame_type: Some("Communication"),
        roles: ["Agent", "Addressee", "Message"],
        inherent_features: FeatureBundle(),
    ),

    // === PROPERTIES ===
    Concept(
        id: "BIG",
        frame_type: None,
        roles: [],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "SMALL",
        frame_type: None,
        roles: [],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "GOOD",
        frame_type: None,
        roles: [],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "BAD",
        frame_type: None,
        roles: [],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "NEW",
        frame_type: None,
        roles: [],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "OLD",
        frame_type: None,
        roles: [],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "RED",
        frame_type: None,
        roles: [],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "BLUE",
        frame_type: None,
        roles: [],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "GREEN",
        frame_type: None,
        roles: [],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "HOT",
        frame_type: None,
        roles: [],
        inherent_features: FeatureBundle(),
    ),
    Concept(
        id: "COLD",
        frame_type: None,
        roles: [],
        inherent_features: FeatureBundle(),
    ),
]
```

---

## Polish Lexicon (data/lexicons/pl/lexicon.ron)

```ron
// Polish lexicon (~500 words for full system, 100 for MVP)

Lexicon(
    entries: {
        // === NOUNS ===
        "Tomek": LexEntry(
            lemma: "Tomek",
            pos: "Noun",
            concept: "PERSON",
            features: FeatureBundle(
                gender: Some(Masculine),
                number: Some(Singular),
                animacy: Some(Animate),
                case: Some(Nominative),
            ),
        ),
        "Iza": LexEntry(
            lemma: "Iza",
            pos: "Noun",
            concept: "PERSON",
            features: FeatureBundle(
                gender: Some(Feminine),
                number: Some(Singular),
                animacy: Some(Animate),
                case: Some(Nominative),
            ),
        ),
        "jabłko": LexEntry(
            lemma: "jabłko",
            pos: "Noun",
            concept: "APPLE",
            features: FeatureBundle(
                gender: Some(Neuter),
                number: Some(Singular),
                countability: Some(Count),
                case: Some(Nominative),
            ),
        ),
        "książka": LexEntry(
            lemma: "książka",
            pos: "Noun",
            concept: "BOOK",
            features: FeatureBundle(
                gender: Some(Feminine),
                number: Some(Singular),
                countability: Some(Count),
                case: Some(Nominative),
            ),
        ),
        "dom": LexEntry(
            lemma: "dom",
            pos: "Noun",
            concept: "HOUSE",
            features: FeatureBundle(
                gender: Some(Masculine),
                number: Some(Singular),
                countability: Some(Count),
                case: Some(Nominative),
            ),
        ),
        "woda": LexEntry(
            lemma: "woda",
            pos: "Noun",
            concept: "WATER",
            features: FeatureBundle(
                gender: Some(Feminine),
                number: Some(Singular),
                countability: Some(Mass),
                case: Some(Nominative),
            ),
        ),
        "mama": LexEntry(
            lemma: "mama",
            pos: "Noun",
            concept: "MOTHER",
            features: FeatureBundle(
                gender: Some(Feminine),
                number: Some(Singular),
                animacy: Some(Animate),
                case: Some(Nominative),
            ),
        ),
        "tata": LexEntry(
            lemma: "tata",
            pos: "Noun",
            concept: "FATHER",
            features: FeatureBundle(
                gender: Some(Masculine),
                number: Some(Singular),
                animacy: Some(Animate),
                case: Some(Nominative),
            ),
        ),
        "kot": LexEntry(
            lemma: "kot",
            pos: "Noun",
            concept: "CAT",
            features: FeatureBundle(
                gender: Some(Masculine),
                number: Some(Singular),
                animacy: Some(Animate),
                case: Some(Nominative),
            ),
        ),
        "pies": LexEntry(
            lemma: "pies",
            pos: "Noun",
            concept: "DOG",
            features: FeatureBundle(
                gender: Some(Masculine),
                number: Some(Singular),
                animacy: Some(Animate),
                case: Some(Nominative),
            ),
        ),
        "mleko": LexEntry(
            lemma: "mleko",
            pos: "Noun",
            concept: "MILK",
            features: FeatureBundle(
                gender: Some(Neuter),
                number: Some(Singular),
                countability: Some(Mass),
                case: Some(Nominative),
            ),
        ),
        "chleb": LexEntry(
            lemma: "chleb",
            pos: "Noun",
            concept: "BREAD",
            features: FeatureBundle(
                gender: Some(Masculine),
                number: Some(Singular),
                countability: Some(Mass),
                case: Some(Nominative),
            ),
        ),
        "samochód": LexEntry(
            lemma: "samochód",
            pos: "Noun",
            concept: "CAR",
            features: FeatureBundle(
                gender: Some(Masculine),
                number: Some(Singular),
                countability: Some(Count),
                case: Some(Nominative),
            ),
        ),
        "miasto": LexEntry(
            lemma: "miasto",
            pos: "Noun",
            concept: "CITY",
            features: FeatureBundle(
                gender: Some(Neuter),
                number: Some(Singular),
                countability: Some(Count),
                case: Some(Nominative),
            ),
        ),

        // === VERBS ===
        "dać": LexEntry(
            lemma: "dać",
            pos: "Verb",
            concept: "GIVE",
            features: FeatureBundle(
                tense: None,
                aspect: Some(Perfective),
            ),
        ),
        "wziąć": LexEntry(
            lemma: "wziąć",
            pos: "Verb",
            concept: "TAKE",
            features: FeatureBundle(
                tense: None,
                aspect: Some(Perfective),
            ),
        ),
        "iść": LexEntry(
            lemma: "iść",
            pos: "Verb",
            concept: "GO",
            features: FeatureBundle(
                tense: None,
                aspect: Some(Imperfective),
            ),
        ),
        "przyjść": LexEntry(
            lemma: "przyjść",
            pos: "Verb",
            concept: "COME",
            features: FeatureBundle(
                tense: None,
                aspect: Some(Perfective),
            ),
        ),
        "widzieć": LexEntry(
            lemma: "widzieć",
            pos: "Verb",
            concept: "SEE",
            features: FeatureBundle(
                tense: None,
                aspect: Some(Imperfective),
            ),
        ),
        "słyszeć": LexEntry(
            lemma: "słyszeć",
            pos: "Verb",
            concept: "HEAR",
            features: FeatureBundle(
                tense: None,
                aspect: Some(Imperfective),
            ),
        ),
        "myśleć": LexEntry(
            lemma: "myśleć",
            pos: "Verb",
            concept: "THINK",
            features: FeatureBundle(
                tense: None,
                aspect: Some(Imperfective),
            ),
        ),
        "wiedzieć": LexEntry(
            lemma: "wiedzieć",
            pos: "Verb",
            concept: "KNOW",
            features: FeatureBundle(
                tense: None,
                aspect: Some(Imperfective),
            ),
        ),
        "kochać": LexEntry(
            lemma: "kochać",
            pos: "Verb",
            concept: "LOVE",
            features: FeatureBundle(
                tense: None,
                aspect: Some(Imperfective),
            ),
        ),
        "jeść": LexEntry(
            lemma: "jeść",
            pos: "Verb",
            concept: "EAT",
            features: FeatureBundle(
                tense: None,
                aspect: Some(Imperfective),
            ),
        ),
        "pić": LexEntry(
            lemma: "pić",
            pos: "Verb",
            concept: "DRINK",
            features: FeatureBundle(
                tense: None,
                aspect: Some(Imperfective),
            ),
        ),
        "czytać": LexEntry(
            lemma: "czytać",
            pos: "Verb",
            concept: "READ",
            features: FeatureBundle(
                tense: None,
                aspect: Some(Imperfective),
            ),
        ),
        "pisać": LexEntry(
            lemma: "pisać",
            pos: "Verb",
            concept: "WRITE",
            features: FeatureBundle(
                tense: None,
                aspect: Some(Imperfective),
            ),
        ),
        "kupić": LexEntry(
            lemma: "kupić",
            pos: "Verb",
            concept: "BUY",
            features: FeatureBundle(
                tense: None,
                aspect: Some(Perfective),
            ),
        ),
        "mówić": LexEntry(
            lemma: "mówić",
            pos: "Verb",
            concept: "SAY",
            features: FeatureBundle(
                tense: None,
                aspect: Some(Imperfective),
            ),
        ),

        // === ADJECTIVES ===
        "duży": LexEntry(
            lemma: "duży",
            pos: "Adjective",
            concept: "BIG",
            features: FeatureBundle(),
        ),
        "mały": LexEntry(
            lemma: "mały",
            pos: "Adjective",
            concept: "SMALL",
            features: FeatureBundle(),
        ),
        "dobry": LexEntry(
            lemma: "dobry",
            pos: "Adjective",
            concept: "GOOD",
            features: FeatureBundle(),
        ),
        "zły": LexEntry(
            lemma: "zły",
            pos: "Adjective",
            concept: "BAD",
            features: FeatureBundle(),
        ),
        "czerwony": LexEntry(
            lemma: "czerwony",
            pos: "Adjective",
            concept: "RED",
            features: FeatureBundle(),
        ),
        "niebieski": LexEntry(
            lemma: "niebieski",
            pos: "Adjective",
            concept: "BLUE",
            features: FeatureBundle(),
        ),
        "gorący": LexEntry(
            lemma: "gorący",
            pos: "Adjective",
            concept: "HOT",
            features: FeatureBundle(),
        ),
        "zimny": LexEntry(
            lemma: "zimny",
            pos: "Adjective",
            concept: "COLD",
            features: FeatureBundle(),
        ),
    },
)
```

---

## Polish Noun Paradigms (data/morphology/pl/noun_paradigms.ron)

```ron
// Polish noun declension paradigms

[
    // Paradigm 1: Neuter nouns ending in -o (e.g., "jabłko")
    MorphParadigm(
        name: "neuter_o",
        rules: [
            // Singular
            MorphRule(
                conditions: [CaseIs(Nominative), NumberIs(Singular)],
                operations: [],  // jabłko → jabłko
            ),
            MorphRule(
                conditions: [CaseIs(Genitive), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "o", to: "a" }],  // jabłko → jabłka
            ),
            MorphRule(
                conditions: [CaseIs(Dative), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "o", to: "u" }],  // jabłko → jabłku
            ),
            MorphRule(
                conditions: [CaseIs(Accusative), NumberIs(Singular)],
                operations: [],  // jabłko → jabłko (neuter ACC = NOM)
            ),
            MorphRule(
                conditions: [CaseIs(Instrumental), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "o", to: "iem" }],  // jabłko → jabłkiem
            ),
            MorphRule(
                conditions: [CaseIs(Locative), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "o", to: "u" }],  // jabłko → (o) jabłku
            ),
            // Plural
            MorphRule(
                conditions: [CaseIs(Nominative), NumberIs(Plural)],
                operations: [ReplaceSuffix { from: "o", to: "a" }],  // jabłko → jabłka
            ),
            MorphRule(
                conditions: [CaseIs(Genitive), NumberIs(Plural)],
                operations: [ReplaceSuffix { from: "o", to: "" }, AddSuffix("ek")],  // jabłko → jabłek
            ),
        ],
    ),

    // Paradigm 2: Feminine nouns ending in -a (e.g., "książka")
    MorphParadigm(
        name: "feminine_a",
        rules: [
            // Singular
            MorphRule(
                conditions: [CaseIs(Nominative), NumberIs(Singular)],
                operations: [],  // książka → książka
            ),
            MorphRule(
                conditions: [CaseIs(Genitive), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "a", to: "i" }],  // książka → książki
            ),
            MorphRule(
                conditions: [CaseIs(Dative), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "a", to: "e" }],  // książka → książce
            ),
            MorphRule(
                conditions: [CaseIs(Accusative), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "a", to: "ę" }],  // książka → książkę
            ),
            MorphRule(
                conditions: [CaseIs(Instrumental), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "a", to: "ą" }],  // książka → książką
            ),
            MorphRule(
                conditions: [CaseIs(Locative), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "a", to: "e" }],  // książka → (o) książce
            ),
            // Plural
            MorphRule(
                conditions: [CaseIs(Nominative), NumberIs(Plural)],
                operations: [ReplaceSuffix { from: "a", to: "i" }],  // książka → książki
            ),
        ],
    ),

    // Paradigm 3: Masculine nouns ending in consonant (e.g., "dom")
    MorphParadigm(
        name: "masculine_consonant",
        rules: [
            // Singular
            MorphRule(
                conditions: [CaseIs(Nominative), NumberIs(Singular)],
                operations: [],  // dom → dom
            ),
            MorphRule(
                conditions: [CaseIs(Genitive), NumberIs(Singular)],
                operations: [AddSuffix("u")],  // dom → domu
            ),
            MorphRule(
                conditions: [CaseIs(Dative), NumberIs(Singular)],
                operations: [AddSuffix("owi")],  // dom → domowi
            ),
            MorphRule(
                conditions: [CaseIs(Accusative), NumberIs(Singular)],
                operations: [],  // dom → dom (inanimate ACC = NOM)
            ),
            MorphRule(
                conditions: [CaseIs(Instrumental), NumberIs(Singular)],
                operations: [AddSuffix("em")],  // dom → domem
            ),
            MorphRule(
                conditions: [CaseIs(Locative), NumberIs(Singular)],
                operations: [AddSuffix("u")],  // dom → (o) domu
            ),
            // Plural
            MorphRule(
                conditions: [CaseIs(Nominative), NumberIs(Plural)],
                operations: [AddSuffix("y")],  // dom → domy
            ),
        ],
    ),
]
```

---

## Polish Verb Paradigms (data/morphology/pl/verb_paradigms.ron)

```ron
// Polish verb conjugation paradigms

[
    // Paradigm 1: Verbs ending in -ać (e.g., "czytać" - to read)
    MorphParadigm(
        name: "verb_ac",
        rules: [
            // Present tense
            MorphRule(
                conditions: [TenseIs(Present), PersonIs(First), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "ać", to: "am" }],  // czytać → czytam
            ),
            MorphRule(
                conditions: [TenseIs(Present), PersonIs(Second), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "ać", to: "asz" }],  // czytać → czytasz
            ),
            MorphRule(
                conditions: [TenseIs(Present), PersonIs(Third), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "ać", to: "a" }],  // czytać → czyta
            ),
            MorphRule(
                conditions: [TenseIs(Present), PersonIs(First), NumberIs(Plural)],
                operations: [ReplaceSuffix { from: "ać", to: "amy" }],  // czytać → czytamy
            ),
            MorphRule(
                conditions: [TenseIs(Present), PersonIs(Second), NumberIs(Plural)],
                operations: [ReplaceSuffix { from: "ać", to: "acie" }],  // czytać → czytacie
            ),
            MorphRule(
                conditions: [TenseIs(Present), PersonIs(Third), NumberIs(Plural)],
                operations: [ReplaceSuffix { from: "ać", to: "ają" }],  // czytać → czytają
            ),
            // Past tense (simplified - masculine form)
            MorphRule(
                conditions: [TenseIs(Past), PersonIs(Third), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "ać", to: "ał" }],  // czytać → czytał
            ),
        ],
    ),

    // Paradigm 2: Irregular verb "dać" (to give)
    MorphParadigm(
        name: "verb_dac_irregular",
        rules: [
            // Present tense (actually future for perfective)
            MorphRule(
                conditions: [TenseIs(Present), PersonIs(First), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "ać", to: "am" }],  // dać → dam
            ),
            MorphRule(
                conditions: [TenseIs(Present), PersonIs(Second), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "ać", to: "asz" }],  // dać → dasz
            ),
            MorphRule(
                conditions: [TenseIs(Present), PersonIs(Third), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "ać", to: "a" }],  // dać → da
            ),
            // Past tense
            MorphRule(
                conditions: [TenseIs(Past), PersonIs(Third), NumberIs(Singular)],
                operations: [ReplaceSuffix { from: "ać", to: "ał" }],  // dać → dał
            ),
        ],
    ),
]
```

---

## English Lexicon (data/lexicons/en/lexicon.ron)

```ron
// English lexicon (~500 words for full system, 100 for MVP)

Lexicon(
    entries: {
        // === NOUNS ===
        "apple": LexEntry(
            lemma: "apple",
            pos: "Noun",
            concept: "APPLE",
            features: FeatureBundle(
                number: Some(Singular),
                countability: Some(Count),
            ),
        ),
        "book": LexEntry(
            lemma: "book",
            pos: "Noun",
            concept: "BOOK",
            features: FeatureBundle(
                number: Some(Singular),
                countability: Some(Count),
            ),
        ),
        "house": LexEntry(
            lemma: "house",
            pos: "Noun",
            concept: "HOUSE",
            features: FeatureBundle(
                number: Some(Singular),
                countability: Some(Count),
            ),
        ),
        "water": LexEntry(
            lemma: "water",
            pos: "Noun",
            concept: "WATER",
            features: FeatureBundle(
                number: Some(Singular),
                countability: Some(Mass),
            ),
        ),
        "milk": LexEntry(
            lemma: "milk",
            pos: "Noun",
            concept: "MILK",
            features: FeatureBundle(
                number: Some(Singular),
                countability: Some(Mass),
            ),
        ),
        "cat": LexEntry(
            lemma: "cat",
            pos: "Noun",
            concept: "CAT",
            features: FeatureBundle(
                number: Some(Singular),
                countability: Some(Count),
            ),
        ),
        "dog": LexEntry(
            lemma: "dog",
            pos: "Noun",
            concept: "DOG",
            features: FeatureBundle(
                number: Some(Singular),
                countability: Some(Count),
            ),
        ),

        // === VERBS ===
        "give": LexEntry(
            lemma: "give",
            pos: "Verb",
            concept: "GIVE",
            features: FeatureBundle(
                tense: None,
                aspect: None,
            ),
        ),
        "take": LexEntry(
            lemma: "take",
            pos: "Verb",
            concept: "TAKE",
            features: FeatureBundle(
                tense: None,
                aspect: None,
            ),
        ),
        "go": LexEntry(
            lemma: "go",
            pos: "Verb",
            concept: "GO",
            features: FeatureBundle(
                tense: None,
                aspect: None,
            ),
        ),
        "see": LexEntry(
            lemma: "see",
            pos: "Verb",
            concept: "SEE",
            features: FeatureBundle(
                tense: None,
                aspect: None,
            ),
        ),
        "eat": LexEntry(
            lemma: "eat",
            pos: "Verb",
            concept: "EAT",
            features: FeatureBundle(
                tense: None,
                aspect: None,
            ),
        ),
        "drink": LexEntry(
            lemma: "drink",
            pos: "Verb",
            concept: "DRINK",
            features: FeatureBundle(
                tense: None,
                aspect: None,
            ),
        ),
        "read": LexEntry(
            lemma: "read",
            pos: "Verb",
            concept: "READ",
            features: FeatureBundle(
                tense: None,
                aspect: None,
            ),
        ),

        // === ADJECTIVES ===
        "big": LexEntry(
            lemma: "big",
            pos: "Adjective",
            concept: "BIG",
            features: FeatureBundle(),
        ),
        "small": LexEntry(
            lemma: "small",
            pos: "Adjective",
            concept: "SMALL",
            features: FeatureBundle(),
        ),
        "good": LexEntry(
            lemma: "good",
            pos: "Adjective",
            concept: "GOOD",
            features: FeatureBundle(),
        ),
        "red": LexEntry(
            lemma: "red",
            pos: "Adjective",
            concept: "RED",
            features: FeatureBundle(),
        ),
    },
)
```

---

## Usage

These files should be placed in the `data/` directory:

```
data/
├── concepts/
│   └── concepts.ron          # 50 concepts (expand to 500)
├── lexicons/
│   ├── pl/
│   │   └── lexicon.ron       # 50 Polish words (expand to 500)
│   └── en/
│       └── lexicon.ron       # 30 English words (expand to 500)
└── morphology/
    └── pl/
        ├── noun_paradigms.ron  # 3 noun paradigms
        └── verb_paradigms.ron  # 2 verb paradigms
```

Load them using `DataLoader`:

```rust
let loader = DataLoader::new("data");
let concepts = loader.load_concepts()?;
let pl_lexicon = loader.load_lexicon("pl")?;
let pl_noun_paradigms = loader.load_morphology("pl", "noun")?;
```
