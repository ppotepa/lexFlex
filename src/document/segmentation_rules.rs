use crate::core::interlingua::LanguageId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentationRules {
    pub terminal_characters: BTreeSet<char>,
    pub closing_characters: BTreeSet<char>,
    pub decimal_separators: BTreeSet<char>,
    pub abbreviations: BTreeSet<String>,
}

impl SegmentationRules {
    pub fn for_language(language: &LanguageId) -> Self {
        let mut rules = Self::default();
        let abbreviations: &[&str] = match language.0.as_str() {
            "pl" => &["dr.", "prof.", "np.", "itd.", "itp.", "mgr.", "inż.", "ul."],
            "en" => &["mr.", "mrs.", "ms.", "dr.", "prof.", "e.g.", "i.e.", "etc."],
            _ => &[],
        };
        rules
            .abbreviations
            .extend(abbreviations.iter().map(|value| value.to_string()));
        rules
    }

    pub fn with_abbreviation(mut self, abbreviation: impl Into<String>) -> Self {
        self.abbreviations
            .insert(abbreviation.into().to_lowercase());
        self
    }
}

impl Default for SegmentationRules {
    fn default() -> Self {
        Self {
            terminal_characters: ['.', '!', '?', '…'].into_iter().collect(),
            closing_characters: ['"', '\'', '”', '’', '»', ')', ']', '}']
                .into_iter()
                .collect(),
            decimal_separators: ['.', ','].into_iter().collect(),
            abbreviations: BTreeSet::new(),
        }
    }
}
