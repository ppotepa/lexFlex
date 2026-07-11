use crate::core::interlingua::{Degree, Entity, FeatureBundle, Quantifier, TemporalReference};
use crate::data::descriptor::LanguageDescriptor;
use crate::data::lexicon::Lexicon;
use crate::error::GenerateError;

/// LanguageRealizer isolates language-specific realization.
/// Default implementations are no-ops or identities for unsupported features (e.g. no cases in EN).
pub trait LanguageRealizer {
    fn realize_noun_phrase(
        &self,
        entity: &Entity,
        features: &mut FeatureBundle,
        desc: &LanguageDescriptor,
        lexicon: &Lexicon,
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
        // Default algorithmic: use provided conjunction, map to target lang
        if items.is_empty() { return Ok(vec![]); }
        if items.len() == 1 { return Ok(items.into_iter().next().unwrap_or_default()); }

        // Map source conjunction to target
        let target_conj = match conjunction {
            "i" | "oraz" | "and" => if desc.language == "pl" { "i" } else { "and" },
            "albo" | "lub" | "or" => if desc.language == "pl" { "albo" } else { "or" },
            "," => ",",
            _ => if desc.language == "pl" { "i" } else { "and" },
        };

        let mut res = vec![];
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                if target_conj == "," {
                    // Comma-separated list: Oxford comma before last item (EN) or just "i" (PL)
                    if i == items.len() - 1 {
                        if desc.language != "pl" {
                            res.push(",".to_string());
                        }
                        res.push(if desc.language == "pl" { "i" } else { "and" }.to_string());
                    } else {
                        res.push(",".to_string());
                    }
                } else if items.len() > 2 && desc.language != "pl" {
                    // Oxford comma for 3+ items in EN: "A, B, and C"
                    if i == items.len() - 1 {
                        res.push(",".to_string());
                        res.push(target_conj.to_string());
                    } else {
                        res.push(",".to_string());
                    }
                } else {
                    // 2 items or PL: just the conjunction
                    res.push(target_conj.to_string());
                }
            }
            res.extend(item.clone());
        }
        Ok(res)
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