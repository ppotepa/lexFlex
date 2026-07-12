use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::interlingua::{Case, FeatureBundle, Number, PartOfSpeech};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexEntry {
    pub lemma: String,
    pub pos: String,
    pub concept: String,
    pub frame_type: Option<String>,
    pub roles: Vec<String>,
    pub paradigm: Option<String>,
    pub features: FeatureBundle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lexicon {
    pub entries: HashMap<String, LexEntry>,
    pub form_to_lemma: HashMap<String, String>,
}

impl Lexicon {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            form_to_lemma: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, key: String, entry: LexEntry) {
        let lower_key = key.to_lowercase();
        self.form_to_lemma.insert(lower_key.clone(), entry.lemma.clone());
        self.entries.insert(lower_key, entry);
    }

    pub fn lookup_by_form(&self, form: &str) -> Option<&LexEntry> {
        let lower = form.to_lowercase();
        self.entries.get(&lower).or_else(|| {
            self.form_to_lemma
                .get(&lower)
                .and_then(|lemma| self.entries.get(lemma))
        })
    }

    pub fn lookup_by_lemma(&self, lemma: &str) -> Option<&LexEntry> {
        self.entries.values().find(|e| e.lemma == lemma)
    }

    pub fn lookup_concept(&self, concept: &str) -> Option<&LexEntry> {
        let c = concept.to_uppercase();
        self.entries.values().find(|e| e.concept.to_uppercase() == c)
    }

    /// Find a pre-registered surface form for lemma + case + number (irregular inflections in lexicon).
    pub fn lookup_inflected_surface(&self, lemma: &str, case: Case, number: Number) -> Option<String> {
        self.entries
            .iter()
            .find(|(_, e)| {
                e.lemma == lemma
                    && e.features.case == Some(case)
                    && e.features.number == Some(number)
            })
            .map(|(form, _)| form.clone())
    }

    /// Early normalization using base form / concept lookup.
    /// Resolves to canonical concept + lemma from lexicon; propagates features (incl new initial_sound).
    /// RESOLVED: early normalization + concept/lemma lookup (no ad-hoc per-word surface patches remain in logic).
    pub fn normalize_entity(&self, entity: &mut crate::core::interlingua::Entity) {
        use crate::core::interlingua::ConceptId;
        let candidate = entity.name.as_deref().unwrap_or(&entity.concept.0).to_lowercase();
        let by_form = self.lookup_by_form(&candidate);
        let by_lemma = self.lookup_by_lemma(&candidate);
        let by_concept = self.lookup_concept(&entity.concept.0);
        let is_proper = entity.name.as_ref().map_or(false, |n| n.chars().next().map_or(false, |c| c.is_uppercase()));
        if is_proper && by_form.is_none() && by_lemma.is_none() {
            // Keep unattested proper names (Tom, Anna); inflected surfaces (Warszawie) still map via form entry.
            return;
        }
        if let Some(e) = by_form.or(by_lemma).or(by_concept) {
            entity.concept = ConceptId::new(&e.concept);
            entity.name = Some(e.lemma.clone());
            let f = &mut entity.features;
            if e.features.gender.is_some() { f.gender = e.features.gender; }
            if e.features.number.is_some() { f.number = e.features.number; }
            if e.features.animacy.is_some() { f.animacy = e.features.animacy; }
            if f.countability.is_none() { f.countability = e.features.countability; }
            if f.initial_sound.is_none() { f.initial_sound = e.features.initial_sound.clone(); }
            if f.definiteness.is_none() { f.definiteness = e.features.definiteness; }
            if f.degree.is_none() { f.degree = e.features.degree; }
            if f.suppletive_comparative.is_none() { f.suppletive_comparative = e.features.suppletive_comparative.clone(); }
            if f.suppletive_superlative.is_none() { f.suppletive_superlative = e.features.suppletive_superlative.clone(); }
        }
    }

    pub fn parse_pos(&self, pos_str: &str) -> PartOfSpeech {
        match pos_str {
            "Noun" => PartOfSpeech::Noun,
            "Verb" => PartOfSpeech::Verb,
            "Adjective" => PartOfSpeech::Adjective,
            "Adverb" => PartOfSpeech::Adverb,
            "Pronoun" => PartOfSpeech::Pronoun,
            "Preposition" => PartOfSpeech::Preposition,
            "Conjunction" => PartOfSpeech::Conjunction,
            "Determiner" => PartOfSpeech::Determiner,
            "Particle" => PartOfSpeech::Particle,
            "Participle" => PartOfSpeech::Participle,
            "Negation" => PartOfSpeech::Negation,
            _ => PartOfSpeech::Unknown,
        }
    }
}

impl Default for Lexicon {
    fn default() -> Self {
        Self::new()
    }
}
