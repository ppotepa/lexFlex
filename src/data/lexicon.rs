use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::interlingua::{Case, FeatureBundle, Number, PartOfSpeech, Token};

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

    /// Lookup longest matching multi-word lexicon entry starting at `start_idx` (3- then 2-word).
    pub fn lookup_multiword(
        &self,
        words: &[&str],
        start_idx: usize,
    ) -> Option<(LexEntry, String, usize)> {
        if start_idx >= words.len() {
            return None;
        }
        for len in (2..=3).rev() {
            if start_idx + len > words.len() {
                continue;
            }
            let phrase: String = words[start_idx..start_idx + len]
                .iter()
                .map(|w| w.trim_matches(|c: char| c.is_ascii_punctuation()).to_lowercase())
                .collect::<Vec<_>>()
                .join(" ");
            if let Some(entry) = self.lookup_by_form(&phrase) {
                let surface: String = words[start_idx..start_idx + len]
                    .iter()
                    .map(|w| w.trim_matches(|c: char| c.is_ascii_punctuation()))
                    .collect::<Vec<_>>()
                    .join(" ");
                return Some((entry.clone(), surface, len));
            }
        }
        None
    }

    /// Parse cardinal value from a token using lexicon number concepts (ONE, TWENTY, …).
    pub fn cardinal_from_token(&self, token: &Token) -> Option<i32> {
        if let Ok(n) = token.form.parse::<i32>() {
            return Some(n);
        }
        let entry = self.lookup_by_form(&token.form.to_lowercase())?;
        match entry.concept.to_uppercase().as_str() {
            "ONE" => Some(1),
            "TWO" => Some(2),
            "THREE" => Some(3),
            "FOUR" => Some(4),
            "FIVE" => Some(5),
            "SIX" => Some(6),
            "SEVEN" => Some(7),
            "EIGHT" => Some(8),
            "NINE" => Some(9),
            "TEN" => Some(10),
            "TWENTY" => Some(20),
            "THIRTY" => Some(30),
            "FORTY" => Some(40),
            "FIFTY" => Some(50),
            "HUNDRED" => Some(100),
            "THOUSAND" => Some(1000),
            _ => None,
        }
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
        let by_form = self.lookup_by_form(&candidate).filter(|e| {
            if entity.features.person.is_some() && e.pos == "Conjunction" {
                return false;
            }
            true
        });
        let by_lemma = self.lookup_by_lemma(&candidate);
        let by_concept = self.lookup_concept(&entity.concept.0);
        let lemma_is_proper = |lemma: &str| {
            lemma.chars().next().map_or(false, |c| c.is_uppercase())
        };
        let is_proper = entity.name.as_ref().map_or(false, |n| {
            n.chars().next().map_or(false, |c| c.is_uppercase())
        }) || by_form.as_ref().map_or(false, |e| lemma_is_proper(&e.lemma))
            || by_lemma.as_ref().map_or(false, |e| lemma_is_proper(&e.lemma));
        if is_proper && by_form.is_none() && by_lemma.is_none() {
            // Keep unattested proper names (Tom, Anna); inflected surfaces (Warszawie) still map via form entry.
            return;
        }
        if let Some(e) = by_form.or(by_lemma) {
            entity.concept = ConceptId::new(&e.concept);
            entity.name = Some(e.lemma.clone());
            let f = &mut entity.features;
            if let Some(bf) = by_form.as_ref() {
                if bf.features.case.is_some() {
                    f.case = bf.features.case;
                }
                if bf.features.number.is_some() && bf.features.case.is_some() {
                    f.number = bf.features.number;
                }
            }
            if e.features.gender.is_some() && f.gender.is_none() {
                f.gender = e.features.gender;
            }
            if e.features.number.is_some() && f.number.is_none() {
                f.number = e.features.number;
            }
            if e.features.animacy.is_some() && f.animacy.is_none() {
                f.animacy = e.features.animacy;
            }
            if f.countability.is_none() { f.countability = e.features.countability; }
            if f.initial_sound.is_none() { f.initial_sound = e.features.initial_sound.clone(); }
            if f.definiteness.is_none() { f.definiteness = e.features.definiteness; }
            if f.degree.is_none() { f.degree = e.features.degree; }
            if f.suppletive_comparative.is_none() { f.suppletive_comparative = e.features.suppletive_comparative.clone(); }
            if f.suppletive_superlative.is_none() { f.suppletive_superlative = e.features.suppletive_superlative.clone(); }
        } else if let Some(e) = by_concept {
            entity.concept = ConceptId::new(&e.concept);
            let f = &mut entity.features;
            if e.features.gender.is_some() && f.gender.is_none() {
                f.gender = e.features.gender;
            }
            if e.features.number.is_some() && f.number.is_none() {
                f.number = e.features.number;
            }
            if e.features.animacy.is_some() && f.animacy.is_none() {
                f.animacy = e.features.animacy;
            }
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
