use super::*;

// ─── Construction tree (IL composable AST) ───────────────────────────────────

/// A linguistic construction wrapping an inner predication frame.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstructionInstance {
    pub construction_concept: ConceptId,
    pub inner: Frame,
}

impl ConstructionInstance {
    pub fn new(concept: ConceptId, inner: Frame) -> Self {
        Self {
            construction_concept: concept,
            inner,
        }
    }
}

// ─── Sentence ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sentence {
    /// Construction tree nodes (primary IL structure for compilation).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constructions: Vec<ConstructionInstance>,
    pub frames: Vec<Frame>,
    pub tense: Option<Tense>,
    pub aspect: Option<Aspect>,
    pub polarity: Polarity,
    pub modality: Option<Modality>,
    pub illocution: Illocution,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub question: Option<QuestionSemantics>,
    pub voice: Option<Voice>,
    pub reflexive: bool,
    pub temporal: Option<TemporalReference>,
    pub quantification: Option<Quantifier>,
    pub resolved_refs: Vec<(String, EntityId)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph: Option<crate::core::graph::LinguisticGraph>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sentence_node_id: Option<NodeId>,
    /// Discourse construction concepts attached during cross-sentence resolution.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub construction_concepts: Vec<ConceptId>,
}

impl Sentence {
    pub fn new() -> Self {
        Self {
            constructions: Vec::new(),
            frames: Vec::new(),
            tense: None,
            aspect: None,
            polarity: Polarity::Positive,
            modality: None,
            illocution: Illocution::Statement,
            question: None,
            voice: None,
            reflexive: false,
            temporal: None,
            quantification: None,
            resolved_refs: Vec::new(),
            graph: None,
            sentence_node_id: None,
            construction_concepts: Vec::new(),
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
    #[serde(default)]
    pub entities_in_focus: Vec<NodeId>,
    #[serde(default)]
    pub recent_mentions: Vec<(NodeId, usize)>,
    #[serde(default)]
    pub coref_edges: Vec<EdgeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utterance_node_id: Option<NodeId>,
    /// Most recent salient subject/topic entity node for continuing discourse.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_topic: Option<NodeId>,
}

impl Default for Discourse {
    fn default() -> Self {
        Self {
            speaker: None,
            addressee: None,
            purpose: None,
            register: None,
            entities_in_focus: Vec::new(),
            recent_mentions: Vec::new(),
            coref_edges: Vec::new(),
            utterance_node_id: None,
            current_topic: None,
        }
    }
}

// (Coordination moved earlier to allow Entity to reference it)

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Utterance {
    pub sentences: Vec<Sentence>,
    pub discourse: Option<Discourse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utterance_node_id: Option<NodeId>,
}

impl Utterance {
    pub fn new() -> Self {
        Self {
            sentences: Vec::new(),
            discourse: None,
            utterance_node_id: None,
        }
    }

    pub fn single_sentence(sentence: Sentence) -> Self {
        Self {
            sentences: vec![sentence],
            discourse: None,
            utterance_node_id: None,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub word_node_id: Option<NodeId>,
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
