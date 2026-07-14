use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Pl,
    En,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Pl => write!(f, "pl"),
            Language::En => write!(f, "en"),
        }
    }
}

impl Language {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pl" | "polish" | "pl-pl" => Some(Language::Pl),
            "en" | "english" | "en-us" | "en-gb" => Some(Language::En),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeductionResult {
    pub word: String,
    pub lang: Language,
    pub surface: SurfaceInfo,
    pub semantics: SemanticInfo,
    pub best_concept: Option<ConceptMatch>,
    pub confidence: f32,
    pub sources_used: Vec<String>,

    // === New deducer fields for full goal ===
    /// When we decide a new (or better) main IL concept is warranted, this carries the full proposal.
    /// Takes precedence for "create main Interlingua concept" when present.
    pub concept_proposal: Option<ProposedConcept>,
    /// Structured bilingual (and multi-form) lexicon proposals for both EN and PL.
    pub lexicon_proposals: Vec<LexiconProposal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SurfaceInfo {
    pub lemma: Option<String>,
    pub pos: Option<String>,
    pub gender: Option<String>,
    pub number: Option<String>,
    pub case: Option<String>,
    pub animacy: Option<String>,
    pub tense: Option<String>,
    pub other_features: Vec<String>,
    pub example_sentences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SemanticInfo {
    pub definitions: Vec<String>,
    pub is_a: Vec<String>,           // From ConceptNet / WordNet - crucial for concept inference
    pub synonyms: Vec<String>,
    pub related: Vec<String>,
    pub hypernyms: Vec<String>,
    pub usage_examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptMatch {
    pub concept_id: String,          // e.g. "PERSON", "LIVE", "WIFE", "YEAR"
    pub confidence: f32,
    pub reason: String,
    pub matched_relations: Vec<String>,
}

/// Structured proposal for a (possibly new) main Interlingua concept.
/// When best_concept is None or weak but evidence is rich, we may propose a brand new
/// ConceptDefinition (with id, roles, features, optional parent) that can be appended
/// to data/concepts/concepts.ron.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedConcept {
    /// The ConceptId we propose (e.g. "QUOKKA", "SWEET_TASTE", or reuse of existing like "SWEET_ADJ").
    pub concept_id: String,
    /// The definition ready to be turned into a RON fragment for concepts.ron.
    pub definition: ProposedConceptDef,
    pub confidence: f32,
    pub reason: String,
    /// Suggested parent for is_a hierarchy (if we can infer one from relations).
    pub parent_suggestion: Option<String>,
    pub matched_relations: Vec<String>,
}

/// Minimal serializable definition that produces valid entries for concepts.ron
/// (compatible with main crate's ConceptDefinition + loader).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProposedConceptDef {
    pub id: String,
    pub frame_type: Option<String>,
    pub roles: Vec<String>,
    /// We store a compact representation; the emitter will turn this into proper RON
    /// for inherent_features (e.g. "(gender: Some(Neuter), countability: Some(Count))").
    pub inherent_features_ron: String,
}

/// Convenience: a lexicon entry proposal (structured + raw RON for easy append).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexiconProposal {
    pub lang: Language,
    /// The key used in lexicon.ron (the surface or form).
    pub key: String,
    /// Raw RON line ready to append (e.g. `("słodkie" , (lemma: "słodki", ...))` )
    pub ron_line: String,
    pub pos: Option<String>,
    pub concept_id: String,
}
