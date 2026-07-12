use serde::{Deserialize, Serialize};
use std::fmt;

// ─── Identifiers ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConceptId(pub String);

impl ConceptId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl fmt::Display for ConceptId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LanguageId(pub String);

impl LanguageId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl fmt::Display for LanguageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ─── Morphological Enums ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
    MasculinePersonal,
    MasculineAnimate,
    MasculineInanimate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Number {
    Singular,
    Plural,
    Dual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Person {
    First,
    Second,
    Third,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Animacy {
    Animate,
    Inanimate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Definiteness {
    Definite,
    Indefinite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Countability {
    Count,
    Mass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Concreteness {
    Concrete,
    Abstract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Case {
    Nominative,
    Genitive,
    Dative,
    Accusative,
    Instrumental,
    Locative,
    Vocative,
    Prepositional,
    Partitive,
    Inessive,
    Elative,
    Illative,
    Adessive,
    Ablative,
    Allative,
    Essive,
    Translative,
    Comitative,
    Abessive,
    Ergative,
    Absolutive,
    Oblique,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tense {
    Past,
    Present,
    Future,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Aspect {
    Perfective,
    Imperfective,
    Progressive,
    Habitual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mood {
    Indicative,
    Subjunctive,
    Imperative,
    Conditional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Voice {
    Active,
    Passive,
    Middle,
    Antipassive,
    Causative,
    Applicative,
    Reflexive,
    Reciprocal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Evidentiality {
    Direct,
    Reported,
    Inferred,
    Assumed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HonorificLevel {
    Plain,
    Polite,
    Honorific,
    Humble,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Classifier {
    General,
    Person,
    Animal,
    Flat,
    Long,
    Book,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Degree {
    Positive,
    Comparative,
    Superlative,
}

// ─── FeatureBundle ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FeatureBundle {
    pub gender: Option<Gender>,
    pub number: Option<Number>,
    pub case: Option<Case>,
    pub animacy: Option<Animacy>,
    pub person: Option<Person>,
    pub definiteness: Option<Definiteness>,
    pub countability: Option<Countability>,
    pub concreteness: Option<Concreteness>,
    pub tense: Option<Tense>,
    pub aspect: Option<Aspect>,
    pub mood: Option<Mood>,
    pub voice: Option<Voice>,
    pub evidentiality: Option<Evidentiality>,
    pub honorific_level: Option<HonorificLevel>,
    pub classifier: Option<Classifier>,
    pub degree: Option<Degree>,
    /// Phonetic / article hint from lexicon data (e.g. "vowel", "consonant", or first letter class).
    /// Enables descriptor/policy driven a/an (RESOLVED: no surface spelling logic).
    pub initial_sound: Option<String>,
    /// Suppletive stem for comparative (data-driven, e.g. special stem for positive base).
    pub suppletive_comparative: Option<String>,
    /// Suppletive stem for superlative.
    pub suppletive_superlative: Option<String>,
    /// Semantic role assigned by preposition (data-driven, e.g. Goal, Source, Location).
    /// Enables direct role assignment without case inference.
    pub semantic_role: Option<SemanticRole>,
}

// ─── Semantic Roles ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SemanticRole {
    Agent,
    Patient,
    Theme,
    Recipient,
    Experiencer,
    Stimulus,
    Source,
    Goal,
    Location,
    Instrument,
    Beneficiary,
    Topic,
    Creator,
    Created,
    Cognizer,
    Content,
    Speaker,
    Message,
}

// ─── Reference ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Reference {
    Direct,
    Anaphoric(String),
    Cataphoric(String),
    Deictic,
    Generic,
    Unresolved,
}

// ─── Coordination (for enumerations/lists) ───────────────────────────────────
// Defined here so Entity can reference it directly (first-class support).

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coordination {
    pub items: Vec<Entity>,
    pub conjunction: String,  // "i", "and", etc.
}

// ─── Entity ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    pub concept: ConceptId,
    pub name: Option<String>,
    pub features: FeatureBundle,
    pub reference: Reference,
    pub id: Option<EntityId>,
    pub coordination: Option<Coordination>,  // first-class lists/enumerations
    /// Adjectival modifiers for this NP head. Populated by parser grouping (structural, no concat of names per 21pts).
    /// This eliminates the need for split/concat (RESOLVED 21pts).
    pub adjectives: Vec<Entity>,
}

impl Entity {
    pub fn new(concept: ConceptId) -> Self {
        Self {
            concept,
            name: None,
            features: FeatureBundle::default(),
            reference: Reference::Direct,
            id: None,
            coordination: None,
            adjectives: vec![],
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn with_features(mut self, features: FeatureBundle) -> Self {
        self.features = features;
        self
    }

    pub fn with_coordination(mut self, coord: Coordination) -> Self {
        self.coordination = Some(coord);
        self
    }
}

// ─── Frames ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Frame {
    Transfer {
        agent: Entity,
        recipient: Entity,
        theme: Entity,
        verb_concept: String,
    },
    Motion {
        mover: Entity,
        source: Option<Entity>,
        goal: Option<Entity>,
        path: Option<Entity>,
        verb_concept: String,
    },
    Creation {
        creator: Entity,
        created: Entity,
        material: Option<Entity>,
        verb_concept: String,
    },
    Destruction {
        agent: Entity,
        patient: Entity,
        instrument: Option<Entity>,
        verb_concept: String,
    },
    Perception {
        experiencer: Entity,
        stimulus: Entity,
        verb_concept: String,
    },
    Cognition {
        cognizer: Entity,
        content: Entity,
        verb_concept: String,
    },
    Emotion {
        experiencer: Entity,
        stimulus: Entity,
        verb_concept: String,
    },
    Communication {
        speaker: Entity,
        addressee: Option<Entity>,
        message: Entity,
        verb_concept: String,
    },
    Statement {
        subject: Entity,
        property: Entity,
        verb_concept: String,
    },
    Existence {
        entity: Entity,
        location: Option<Entity>,
        verb_concept: String,
    },
    Possession {
        possessor: Entity,
        possessed: Entity,
        verb_concept: String,
    },
    Consumption {
        agent: Entity,
        patient: Entity,
        verb_concept: String,
    },
    Custom {
        name: String,
        roles: Vec<(SemanticRole, Entity)>,
    },
}

impl Frame {
    pub fn frame_type_name(&self) -> &str {
        match self {
            Frame::Transfer { .. } => "Transfer",
            Frame::Motion { .. } => "Motion",
            Frame::Creation { .. } => "Creation",
            Frame::Destruction { .. } => "Destruction",
            Frame::Perception { .. } => "Perception",
            Frame::Cognition { .. } => "Cognition",
            Frame::Emotion { .. } => "Emotion",
            Frame::Communication { .. } => "Communication",
            Frame::Statement { .. } => "Statement",
            Frame::Existence { .. } => "Existence",
            Frame::Possession { .. } => "Possession",
            Frame::Consumption { .. } => "Consumption",
            Frame::Custom { name, .. } => name,
        }
    }

    pub fn required_roles(&self) -> Vec<SemanticRole> {
        match self {
            Frame::Transfer { .. } => {
                vec![SemanticRole::Agent, SemanticRole::Recipient, SemanticRole::Theme]
            }
            Frame::Motion { .. } => vec![SemanticRole::Agent],
            Frame::Creation { .. } => {
                vec![SemanticRole::Creator, SemanticRole::Created]
            }
            Frame::Destruction { .. } => {
                vec![SemanticRole::Agent, SemanticRole::Patient]
            }
            Frame::Perception { .. } => {
                vec![SemanticRole::Experiencer, SemanticRole::Stimulus]
            }
            Frame::Cognition { .. } => {
                vec![SemanticRole::Cognizer, SemanticRole::Content]
            }
            Frame::Emotion { .. } => {
                vec![SemanticRole::Experiencer, SemanticRole::Stimulus]
            }
            Frame::Communication { .. } => {
                vec![SemanticRole::Speaker, SemanticRole::Message]
            }
            Frame::Statement { .. } => {
                vec![SemanticRole::Topic, SemanticRole::Theme]
            }
            Frame::Existence { .. } => vec![SemanticRole::Theme],
            Frame::Possession { .. } => {
                vec![SemanticRole::Agent, SemanticRole::Theme]
            }
            Frame::Consumption { .. } => {
                vec![SemanticRole::Agent, SemanticRole::Patient]
            }
            Frame::Custom { roles, .. } => roles.iter().map(|(r, _)| *r).collect(),
        }
    }

    pub fn entities(&self) -> Vec<&Entity> {
        match self {
            Frame::Transfer { agent, recipient, theme, .. } => {
                vec![agent, recipient, theme]
            }
            Frame::Motion { mover, source, goal, path, .. } => {
                let mut v = vec![mover];
                if let Some(s) = source { v.push(s); }
                if let Some(g) = goal { v.push(g); }
                if let Some(p) = path { v.push(p); }
                v
            }
            Frame::Creation { creator, created, material, .. } => {
                let mut v = vec![creator, created];
                if let Some(m) = material { v.push(m); }
                v
            }
            Frame::Destruction { agent, patient, instrument, .. } => {
                let mut v = vec![agent, patient];
                if let Some(i) = instrument { v.push(i); }
                v
            }
            Frame::Perception { experiencer, stimulus, .. } => vec![experiencer, stimulus],
            Frame::Cognition { cognizer, content, .. } => vec![cognizer, content],
            Frame::Emotion { experiencer, stimulus, .. } => vec![experiencer, stimulus],
            Frame::Communication { speaker, addressee, message, .. } => {
                let mut v = vec![speaker, message];
                if let Some(a) = addressee { v.push(a); }
                v
            }
            Frame::Statement { subject, property, .. } => vec![subject, property],
            Frame::Existence { entity, location, .. } => {
                let mut v = vec![entity];
                if let Some(l) = location { v.push(l); }
                v
            }
            Frame::Possession { possessor, possessed, .. } => vec![possessor, possessed],
            Frame::Consumption { agent, patient, .. } => vec![agent, patient],
            Frame::Custom { roles, .. } => roles.iter().map(|(_, e)| e).collect(),
        }
    }

    pub fn entities_mut(&mut self) -> Vec<&mut Entity> {
        match self {
            Frame::Transfer { agent, recipient, theme, .. } => {
                vec![agent, recipient, theme]
            }
            Frame::Motion { mover, source, goal, path, .. } => {
                let mut v = vec![mover];
                if let Some(s) = source { v.push(s); }
                if let Some(g) = goal { v.push(g); }
                if let Some(p) = path { v.push(p); }
                v
            }
            Frame::Creation { creator, created, material, .. } => {
                let mut v = vec![creator, created];
                if let Some(m) = material { v.push(m); }
                v
            }
            Frame::Destruction { agent, patient, instrument, .. } => {
                let mut v = vec![agent, patient];
                if let Some(i) = instrument { v.push(i); }
                v
            }
            Frame::Perception { experiencer, stimulus, .. } => vec![experiencer, stimulus],
            Frame::Cognition { cognizer, content, .. } => vec![cognizer, content],
            Frame::Emotion { experiencer, stimulus, .. } => vec![experiencer, stimulus],
            Frame::Communication { speaker, addressee, message, .. } => {
                let mut v = vec![speaker, message];
                if let Some(a) = addressee { v.push(a); }
                v
            }
            Frame::Statement { subject, property, .. } => vec![subject, property],
            Frame::Existence { entity, location, .. } => {
                let mut v = vec![entity];
                if let Some(l) = location { v.push(l); }
                v
            }
            Frame::Possession { possessor, possessed, .. } => vec![possessor, possessed],
            Frame::Consumption { agent, patient, .. } => vec![agent, patient],
            Frame::Custom { roles, .. } => roles.iter_mut().map(|(_, e)| e).collect(),
        }
    }
}

// ─── Polarity, Illocution, Modality ──────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Polarity {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Illocution {
    Statement,
    Question,
    Command,
    Exclamation,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Modality {
    Realis,
    Irrealis,
    Possibility,
    Necessity,
}

// ─── Temporal ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TemporalReference {
    Absolute { timestamp: String },
    Relative { offset_days: i64, anchor: TemporalAnchor },
    Deictic { word: String },
    Duration { days: i64 },
    Frequency { times: i32, period: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TemporalAnchor {
    Now,
    Past,
    Future,
}

// ─── Quantification ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Quantifier {
    Universal,
    Existential,
    NegatedExistential,
    Numerical(i32),
    Proportional(String),
}

// ─── Sentence ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sentence {
    pub frames: Vec<Frame>,
    pub tense: Option<Tense>,
    pub aspect: Option<Aspect>,
    pub polarity: Polarity,
    pub modality: Option<Modality>,
    pub illocution: Illocution,
    pub voice: Option<Voice>,
    pub reflexive: bool,
    pub temporal: Option<TemporalReference>,
    pub quantification: Option<Quantifier>,
    pub resolved_refs: Vec<(String, EntityId)>,
}

impl Sentence {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            tense: None,
            aspect: None,
            polarity: Polarity::Positive,
            modality: None,
            illocution: Illocution::Statement,
            voice: None,
            reflexive: false,
            temporal: None,
            quantification: None,
            resolved_refs: Vec::new(),
        }
    }
}

impl Default for Sentence {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Discourse (Feature v0.2+) ──────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Discourse {
    pub speaker: Option<Entity>,
    pub addressee: Option<Entity>,
    pub purpose: Option<String>,
    pub register: Option<String>,
}

// (Coordination moved earlier to allow Entity to reference it)

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Utterance {
    pub sentences: Vec<Sentence>,
    pub discourse: Option<Discourse>,
}

impl Utterance {
    pub fn new() -> Self {
        Self {
            sentences: Vec::new(),
            discourse: None,
        }
    }

    pub fn single_sentence(sentence: Sentence) -> Self {
        Self {
            sentences: vec![sentence],
            discourse: None,
        }
    }
}

impl Default for Utterance {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Interlingua (top-level) ─────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Interlingua {
    Natural(Utterance),
    MathExpression(String),
    LogicalProposition(String),
    ProgramStatement(String),
}

impl Interlingua {
    pub fn as_natural(&self) -> Option<&Utterance> {
        match self {
            Interlingua::Natural(u) => Some(u),
            _ => None,
        }
    }
}

// ─── Token types (for parsing pipeline) ──────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub form: String,
    pub lemma: Option<String>,
    pub pos: PartOfSpeech,
    pub features: FeatureBundle,
    pub span: (usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Adverb,
    Pronoun,
    Preposition,
    Conjunction,
    Determiner,
    Particle,
    Participle,
    Negation,
    Punctuation,
    Unknown,
}

// ─── MorphAnalysis ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct MorphAnalysis {
    pub token: Token,
    pub candidates: Vec<MorphCandidate>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MorphCandidate {
    pub lemma: String,
    pub pos: PartOfSpeech,
    pub features: FeatureBundle,
    pub paradigm: Option<String>,
}

// ─── Concept definition (for data loading) ───────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptDefinition {
    pub id: String,
    pub frame_type: Option<String>,
    pub roles: Vec<String>,
    pub inherent_features: FeatureBundle,
}
