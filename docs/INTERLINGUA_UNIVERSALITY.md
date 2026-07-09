# Interlingua Universality Principle

## Core Principle

**Interlingua is a superset of all natural languages.** It contains ALL possible grammatical, semantic, and pragmatic features found across all languages. Each language uses only the subset relevant to it.

## Implications

### 1. FeatureBundle as Superset

FeatureBundle zawiera WSZYSTKIE możliwe features. Języki używają tylko tych, które są dla nich relevantne:

```rust
pub struct FeatureBundle {
    // Morphological (języki fleksyjne)
    pub gender: Option<Gender>,           // PL/DE/RU/FR: ważne, EN/ZH: None
    pub number: Option<Number>,           // wszystkie języki
    pub case: Option<Case>,               // PL/RU/DE/FI/HU: ważne, EN/ZH: None
    pub animacy: Option<Animacy>,         // PL/RU: ważne, inne: None
    
    // Verbal
    pub tense: Option<Tense>,             // EN/FR/DE: ważne, ZH: None
    pub aspect: Option<Aspect>,           // PL/RU/ZH: ważne
    pub mood: Option<Mood>,               // wszystkie języki
    pub evidentiality: Option<Evidentiality>, // TR/JA/KO: ważne, PL/EN: None
    
    // Syntactic
    pub voice: Option<Voice>,             // wszystkie języki
    pub polarity: Option<Polarity>,       // wszystkie języki
    
    // Pragmatic
    pub honorific_level: Option<HonorificLevel>, // JA/KO/JV: ważne, PL/EN: None
    
    // Classifiers (języki azjatyckie)
    pub classifier: Option<Classifier>,   // ZH/JA/TH/VN: ważne, PL/EN: None
}
```

**Przykład:**
- Polish: gender=Some(Masculine), case=Some(Nominative), tense=None, classifier=None
- Chinese: gender=None, case=None, tense=None, classifier=Some(General)
- Japanese: gender=None, case=Some(Nominative), evidentiality=Some(Direct), honorific_level=Some(Polite)
- Finnish: gender=None, case=Some(Inessive), tense=Some(Past), classifier=None

### 2. Case System as Universal Superset

Case enum zawiera WSZYSTKIE przypadki ze wszystkich języków:

```rust
pub enum Case {
    // Universal core (IE languages)
    Nominative,        // PL, DE, RU, LA, GR
    Genitive,          // PL, DE, RU, LA, GR
    Dative,            // PL, DE, RU, LA
    Accusative,        // PL, DE, RU, LA, GR
    
    // Extended (Slavic, Baltic)
    Instrumental,      // PL, RU, LT
    Locative,          // PL, RU
    Vocative,          // PL, CS, HR, LT, LA, GR
    Prepositional,     // RU
    
    // Finno-Ugric (Finnish, Hungarian)
    Partitive,         // FI: osittainen (partial)
    Inessive,          // FI: -ssa (in)
    Elative,           // FI: -sta (from)
    Illative,          // FI: -Vn (into)
    Adessive,          // FI: -lla (on)
    Ablative,          // FI: -lta (from)
    Allative,          // FI: -lle (to)
    Essive,            // FI: -na (as)
    Translative,       // FI: -ksi (becoming)
    Comitative,        // FI: -ne (with)
    Abessive,          // FI: -tta (without)
    
    // Ergative-absolutive languages
    Ergative,          // BASQUE, GEORGIAN
    Absolutive,        // BASQUE, GEORGIAN
    
    // Austronesian
    Oblique,           // TAGALOG
}
```

**Mapping do języków:**
- Polish: używa Nominative, Genitive, Dative, Accusative, Instrumental, Locative, Vocative
- Finnish: używa Nominative, Genitive, Partitive, Inessive, Elative, Illative, itd.
- English: używa tylko Nominative, Accusative (I/me, he/him), Genitive ('s)
- Chinese: case=None (używa word order i prepositions)

### 3. Temporal System as Universal Superset

Nie wszystkie języki mają tense. Interlingua używa kombinacji tense, aspect, i temporal reference.

See [INTERLINGUA.md](./INTERLINGUA.md#sentence) for the canonical Sentence definition, which includes:
- `tense: Option<Tense>` - Some(Past) dla EN/PL, None dla ZH
- `aspect: Option<Aspect>` - wszystkie języki
- `mood: Option<Mood>` - wszystkie języki
- `evidentiality: Option<Evidentiality>` - TR/JA/KO: ważne
- `temporal: Option<TemporalReference>` - uniwersalne

**Przykład:**
- English: "He gave" → tense=Some(Past), aspect=None, temporal=None
- Polish: "Dał" → tense=Some(Past), aspect=Some(Perfective), temporal=None
- Chinese: "他给了" (tā gěi le) → tense=None, aspect=Some(Perfective), temporal=None
- Turkish: "Verdi" → tense=Some(Past), aspect=None, evidentiality=Some(Direct)
- Japanese: "上げた" (ageta) → tense=Some(Past), aspect=Some(Perfective), evidentiality=Some(Direct)

### 4. Word Order Flexibility

Różne języki mają różne word order constraints.

See [LANGUAGE_DESCRIPTOR.md](./LANGUAGE_DESCRIPTOR.md#descriptor-structure) for the canonical LanguageDescriptor definition, which includes:
- `word_order: WordOrder` (SVO, SOV, VSO, VOS, OVS, OSV, Free)
- `word_order_flexibility: WordOrderFlexibility` (Strict, V2, Flexible, Free)

**Przykład:**
- English: SVO, Strict → "John gave Mary the apple" (nie: "Mary John gave the apple")
- Polish: SVO, Flexible → "Jan dał Marii jabłko" / "Marii Jan dał jabłko" (zależy od focus)
- German: SOV (subordinate), V2 (main) → "Ich glaube, dass Johann der Maria den Apfel gab"
- Japanese: SOV, Strict → "ジョンがメアリーにりんごをあげた" (John ga Mary ni ringo o ageta)
- Arabic: VSO, Flexible → "أعطى يوحنا مريم التفاحة" (aʿṭā yūḥannā maryam al-tuffāḥa)

### 5. Topic-Comment Structure

Niektóre języki są topic-prominent. Sentence (zdefiniowane w [INTERLINGUA.md](./INTERLINGUA.md#sentence)) zawiera pola topic-comment:
- `topic: Option<EntityRef>` - what the sentence is about
- `topic_marker: Option<TopicMarker>` - how topic is marked

```rust
pub enum TopicMarker {
    Explicit(String),   // JP: は (wa), KO: 은/는 (eun/neun)
    Implicit,           // PL: word order (topic first)
}
```

**Przykład:**
- Japanese: "ジョンはメアリーにりんごをあげた" (John WA Mary ni ringo o ageta)
  - topic=John, topic_marker=Explicit("は")
  - comment: gave Mary apple
- Polish: "Jan dał Marii jabłko"
  - topic=Jan, topic_marker=Implicit
  - comment: dał Marii jabłko

### 6. Classifiers

Niektóre języki używają classifiers. FeatureBundle (zdefiniowane w [INTERLINGUA.md](./INTERLINGUA.md#featurebundle)) zawiera:
- `classifier: Option<Classifier>` - ZH/JA/TH/VN: ważne, PL/EN: None

```rust
pub enum Classifier {
    General,        // ZH: 个 (ge), JA: つ (tsu)
    Person,         // ZH: 人 (rén), JA: 人 (nin)
    Animal,         // JA: 匹 (hiki) - małe zwierzęta
    Flat,           // JA: 枚 (mai) - płaskie obiekty
    Long,           // JA: 本 (hon) - długie obiekty
    Book,           // JA: 冊 (satsu) - książki
    Custom(String), // language-specific
}
```

**Przykład:**
- Chinese: "三个人" (sān gè rén) → "three CLF person" → classifier=Some(General)
- Japanese: "三人の学生" (san nin no gakusei) → "three CLF person student" → classifier=Some(Person)
- Polish: "trzech studentów" → classifier=None

### 7. Honorifics

Niektóre języki mają rozbudowane systemy honorifics. Discourse (zdefiniowane w [DISCOURSE.md](./DISCOURSE.md#discourse-model)) zawiera pole `honorific_level: Option<HonorificLevel>` które jest ważne dla JA/KO/JV, ale ignorowane dla PL/EN.

See [DISCOURSE.md](./DISCOURSE.md#discourse-model) for the canonical `HonorificLevel` enum definition.

**Przykład:**
- Polish: "Czy Pan mógłby..." → honorific_level=Some(Polite)
- English: "Could you please..." → honorific_level=Some(Polite)
- Japanese: "お食べになりますか" (o-tabe-ni-narimasu ka) → honorific_level=Some(Honorific)
- Javanese: ngoko vs madya vs krama → honorific_level=Some(Krama)

## Implementation Guidelines

### 1. FeatureBundle jest zawsze kompletny

Nawet jeśli język nie używa danego feature, pole istnieje (jako None):

```rust
// Polish noun
FeatureBundle {
    gender: Some(Masculine),
    number: Some(Singular),
    case: Some(Nominative),
    animacy: Some(Animate),
    tense: None,                    // nouns nie mają tense
    aspect: None,                   // nouns nie mają aspect
    evidentiality: None,            // PL nie ma evidentiality
    honorific_level: None,          // PL nie ma honorifics
    classifier: None,               // PL nie ma classifiers
}

// Chinese noun
FeatureBundle {
    gender: None,                   // ZH nie ma grammatical gender
    number: Some(Singular),
    case: None,                     // ZH nie ma cases
    animacy: None,                  // ZH nie ma animacy
    tense: None,
    aspect: None,
    evidentiality: None,
    honorific_level: None,
    classifier: Some(General),      // ZH ma classifiers!
}
```

### 2. Każdy język definiuje swój subset

LanguageDescriptor (zdefiniowane w [LANGUAGE_DESCRIPTOR.md](./LANGUAGE_DESCRIPTOR.md#descriptor-structure)) mówi, które features są używane przez dany język.

**Przykład:**
- Polish: używa Gender, Number, Case, Animacy, Tense, Aspect, Mood, Voice; ignoruje Evidentiality, HonorificLevel, Classifier
- Japanese: używa Number, Case, Tense, Aspect, Evidentiality, HonorificLevel; ignoruje Gender, Animacy, Classifier
- Chinese: używa Number, Aspect, Classifier; ignoruje Gender, Case, Tense, HonorificLevel

### 3. Translation zachowuje wszystkie features

Podczas tłumaczenia PL→ZH, zachowujemy WSZYSTKIE features z PL, nawet jeśli ZH ich nie używa:

```rust
// Input (PL): "Dał jej jabłko"
// Interlingua:
Utterance {
    sentences: [Sentence {
        frames: [Frame::Transfer {
            agent: Entity { features: FeatureBundle { gender: Some(Masculine), ... } },
            recipient: Entity { features: FeatureBundle { gender: Some(Feminine), ... } },
            theme: Entity { features: FeatureBundle { gender: Some(Neuter), ... } },
        }],
        tense: Some(Past),
        aspect: Some(Perfective),
    }],
}

// Output (ZH): "他给了她苹果" (tā gěi le tā píngguǒ)
// Generator ZH ignoruje gender, ale zachowuje tense/aspect
// tense=None (ZH nie ma tense), aspect=Some(Perfective)
```

## Summary

Interlingua jest **uniwersalnym supersetem** wszystkich języków:
- FeatureBundle zawiera WSZYSTKIE możliwe features
- Case enum zawiera WSZYSTKIE przypadki ze wszystkich języków
- Temporal system obsługuje tense, aspect, evidentiality
- Word order jest konfigurowalny per język
- Topic-comment, classifiers, honorifics są opcjonalne

Każdy język używa tylko swojego subsetu, ale Interlingua może reprezentować WSZYSTKO.
