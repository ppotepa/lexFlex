use crate::core::graph::LinguisticGraph;
use crate::core::interlingua::{
    Case, Degree, Entity, FeatureBundle, Number, Quantifier, TemporalReference,
};
use crate::data::descriptor::LanguageDescriptor;
use crate::data::lexicon::Lexicon;
use crate::error::GenerateError;

/// Apply AgeIdiom construction features from graph before NP realization.
pub fn adjust_age_idiom_entity(
    entity: &mut Entity,
    features: &mut FeatureBundle,
    graph: Option<&LinguisticGraph>,
    lexicon: &Lexicon,
    lang: &str,
) {
    if !graph.map_or(false, |g| g.has_construction("AgeIdiom")) || entity.concept.0 != "YEAR" {
        return;
    }
    if lang == "pl" {
        entity.adjectives.clear();
        let case = Case::Genitive;
        entity.features.case = Some(case);
        features.case = Some(case);
        entity.features.number = Some(Number::Plural);
        features.number = Some(Number::Plural);
        let lemma = lexicon
            .lookup_concept("YEAR")
            .map(|e| e.lemma.clone())
            .or_else(|| entity.name.clone())
            .unwrap_or_default();
        if let Some(surface) = lexicon.lookup_inflected_surface(&lemma, case, Number::Plural) {
            entity.name = Some(surface);
        }
    } else {
        entity.features.number = Some(Number::Plural);
        features.number = Some(Number::Plural);
        if let Some(entry) = lexicon.lookup_concept("YEAR") {
            entity.name = Some(entry.lemma.clone());
        }
    }
}

/// LanguageRealizer isolates language-specific realization.
/// Default implementations are no-ops or identities for unsupported features (e.g. no cases in EN).
pub trait LanguageRealizer {
    fn realize_noun_phrase(
        &self,
        entity: &Entity,
        features: &mut FeatureBundle,
        desc: &LanguageDescriptor,
        lexicon: &Lexicon,
        graph: Option<&LinguisticGraph>,
    ) -> Result<Vec<String>, GenerateError>;

    fn realize_verb(
        &self,
        lemma: &str,
        features: &FeatureBundle,
        desc: &LanguageDescriptor,
    ) -> Result<String, GenerateError>;

    fn realize_coordinations(
        &self,
        items: Vec<Vec<String>>,
        conjunction: &str,
        desc: &LanguageDescriptor,
    ) -> Result<Vec<String>, GenerateError> {
        // Default algorithmic: build a single phrase token for the coordinated list.
        // This prevents spaced punctuation like "wife , daughter" when top-level result is words.join(" ").
        if items.is_empty() { return Ok(vec![]); }
        if items.len() == 1 { return Ok(items.into_iter().next().unwrap_or_default()); }

        // Map source conjunction to target
        let target_conj = match conjunction {
            "i" | "oraz" | "and" => if desc.language == "pl" { "i" } else { "and" },
            "albo" | "lub" | "or" => if desc.language == "pl" { "albo" } else { "or" },
            "," => ",",
            _ => if desc.language == "pl" { "i" } else { "and" },
        };

        // First, stringify each sub-item (may be "a wife" etc)
        let phrases: Vec<String> = items.into_iter().map(|it| it.join(" ")).collect();

        // Build the list phrase with proper separators (no lone punct tokens)
        let list_str = if target_conj == "," || (phrases.len() > 2 && desc.language != "pl") {
            // Oxford style for EN: "A, B, and C"
            if phrases.len() > 1 {
                let last = phrases.last().unwrap();
                let prefix = &phrases[..phrases.len()-1];
                if desc.language == "pl" {
                    format!("{} i {}", prefix.join(", "), last)
                } else {
                    format!("{}, and {}", prefix.join(", "), last)
                }
            } else {
                phrases.join(" ")
            }
        } else {
            // simple "A and B"
            let last = phrases.last().unwrap();
            let prefix = &phrases[..phrases.len()-1];
            format!("{} {} {}", prefix.join(" "), target_conj, last)  // note: caller may adjust
        };

        Ok(vec![list_str])
    }

    fn adjust_for_quantifier(&self, features: &mut FeatureBundle, q: &Quantifier, desc: &LanguageDescriptor) {
        // default no-op; PL impl will set case for Numerical
    }

    fn realize_quantifier(&self, q: &Quantifier, desc: &LanguageDescriptor) -> Result<Vec<String>, GenerateError> {
        match q {
            Quantifier::Universal => Ok(vec!["all".to_string()]), // stub
            Quantifier::Existential => Ok(vec!["some".to_string()]),
            Quantifier::NegatedExistential => Ok(vec!["none".to_string()]),
            Quantifier::Numerical(n) => Ok(vec![n.to_string()]),
            Quantifier::Proportional(p) => Ok(vec![p.clone()]),
        }
    }

    fn realize_degree(&self, base: &str, _deg: Degree, _desc: &LanguageDescriptor) -> String {
        // stub; real impl in morph
        base.to_string()
    }

    // stubs for unsupported
    fn apply_case(&self, form: String, _case: crate::core::interlingua::Case, _desc: &LanguageDescriptor) -> String {
        form
    }

    fn get_article(&self, _entity: &Entity, _needs: bool, _desc: &LanguageDescriptor) -> Option<String> {
        None
    }

    fn realize_temporal(&self, temporal: &TemporalReference, desc: &LanguageDescriptor) -> Option<String> {
        match temporal {
            TemporalReference::Deictic { word } => {
                // Map common deictics to target lang surface
                let mapped = match (desc.language.as_str(), word.as_str()) {
                    ("en", "wczoraj") => "yesterday",
                    ("en", "dzisiaj") => "today",
                    ("en", "jutro") => "tomorrow",
                    ("en", "teraz") => "now",
                    ("pl", "yesterday") => "wczoraj",
                    ("pl", "today") => "dzisiaj",
                    ("pl", "tomorrow") => "jutro",
                    ("pl", "now") => "teraz",
                    _ => word.as_str(),
                };
                Some(mapped.to_string())
            }
            TemporalReference::Relative { offset_days, .. } => {
                let word = match (desc.language.as_str(), *offset_days) {
                    ("en", -1) => "yesterday",
                    ("en", 0) => "today",
                    ("en", 1) => "tomorrow",
                    ("pl", -1) => "wczoraj",
                    ("pl", 0) => "dzisiaj",
                    ("pl", 1) => "jutro",
                    _ => return None,
                };
                Some(word.to_string())
            }
            _ => None,
        }
    }

    fn negation_particle<'a>(&self, desc: &'a LanguageDescriptor) -> Option<&'a str> {
        Some(desc.syntax.negation_particle.as_str())
    }

    fn question_particle<'a>(&self, desc: &'a LanguageDescriptor) -> Option<&'a str> {
        if desc.language == "pl" { Some("Czy") } else { Some("Does") }  // caller adjusts to Do/Did based on tense/number
    }
}