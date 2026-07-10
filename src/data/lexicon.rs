use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::interlingua::{FeatureBundle, PartOfSpeech};

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
        self.form_to_lemma.insert(key.clone(), entry.lemma.clone());
        self.entries.insert(key, entry);
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
        self.entries.values().find(|e| e.concept == concept)
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
