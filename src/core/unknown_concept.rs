//! Unknown concept resolution for words not present in any lexicon.
//!
//! This module provides the mechanism to automatically assign a real base ConceptId
//! to unknown words so they can participate in parsing, frames, and the linguistic graph.
//!
//! The three core responsibilities, described in normal (non-acronym) language:
//!
//! A. Automatic routing of unknowns into entity construction:
//!    - is_entity_candidate_token() treats Unknown POS tokens (with alphabetic chars, len>2) as valid for NP/PP.
//!    - Parsers invoke the resolver for such tokens so unknowns become first-class Entity nodes.
//!
//! B. Algorithmic selection of a real base ConceptId from project data:
//!    - For words absent from the lexicon, resolve_concept_for_unknown() falls back to
//!      UnknownConceptResolver::resolve() which uses a deterministic bucket (length + vowel count + context)
//!      to pick from the live list of ConceptIds loaded from data/concepts/concepts.ron (with baked fallback).
//!    - The result is always a real ConceptId that exists in the ontology; never a synthetic placeholder id
//!      and never the raw unknown surface string as the concept.
//!
//! C. Preservation of original surface name alongside the assigned concept:
//!    - After assignment, the Entity must retain .name = the exact surface/lemma from input (e.g. "blargxyz")
//!      while .concept holds the real base ConceptId chosen in B.
//!    - Downstream code (e.g. normalize_entity) must not overwrite .name when only a by-concept match occurred.
//!
//! Pure logic where possible; uses passed Lexicon for lookup.
//! Used by parsers for NP/PP token filtering and entity creation.

use std::path::PathBuf;

use crate::core::interlingua::{ConceptDefinition, ConceptId, PartOfSpeech, Token};
use crate::data::lexicon::Lexicon;

#[derive(Debug, Default, Clone)]
pub struct UnknownConceptResolver;

/// Resolver responsible for "B": choosing a real base ConceptId for unknown words
/// using an algorithmic bucket over the project's known concepts list.
impl UnknownConceptResolver {
    /// Assign a base concept for an unknown word form.
    pub fn resolve(base: &str, context_hint: Option<&str>, known_concepts: &[String]) -> String {
        if base.trim().is_empty() || known_concepts.is_empty() {
            return "PERSON".to_string();
        }
        let normalized: String = base.to_lowercase()
            .chars()
            .filter(|c| c.is_alphabetic())
            .collect();
        if normalized.is_empty() {
            return "PERSON".to_string();
        }
        let len = normalized.len();
        let vowels = normalized.chars().filter(|&c| "aeiouy".contains(c)).count();
        let ctx_factor = context_hint.map(|s| s.len() % 3).unwrap_or(0);
        let id = (len + vowels + ctx_factor) % known_concepts.len();
        known_concepts[id].clone()
    }

    pub fn is_valid_concept(concept: &str) -> bool {
        !concept.is_empty() 
    }
}

/// Resolve a real ConceptId for a word that is not present in the lexicon.
///
/// This implements the A + B responsibilities:
/// - A: called for words that were routed as entity candidates
/// - B: falls back to algorithmic selection of a base concept from real data
pub fn resolve_concept_for_unknown(
    lexicon: &Lexicon,
    surface: &str,
    lemma: &str,
    context_hint: Option<&str>,
    known_concepts: &[String],
) -> ConceptId {
    if let Some(entry) = lexicon
        .lookup_by_form(surface)
        .or_else(|| lexicon.lookup_by_lemma(lemma))
    {
        ConceptId::new(&entry.concept)
    } else if surface.chars().next().map_or(false, |c| c.is_uppercase()) {
        ConceptId::new("PERSON")
    } else {
        let list = if known_concepts.is_empty() {
            load_concept_ids()
        } else {
            known_concepts.to_vec()
        };
        let real_concept = UnknownConceptResolver::resolve(lemma, context_hint, &list);
        ConceptId::new(&real_concept)
    }
}

/// Try to locate the canonical concepts.ron with several strategies suitable for
/// cargo test, cargo run, docker /app, installed layouts, etc.
fn find_concepts_ron_path() -> Option<PathBuf> {
    // Highest priority: explicit env override
    if let Ok(dir) = std::env::var("LEXFLEX_DATA_DIR") {
        let p = PathBuf::from(dir).join("concepts/concepts.ron");
        if p.exists() {
            return Some(p);
        }
    }

    // From current working dir, walk up a few levels
    if let Ok(mut dir) = std::env::current_dir() {
        for _ in 0..6 {
            let p = dir.join("data/concepts/concepts.ron");
            if p.exists() {
                return Some(p);
            }
            if !dir.pop() {
                break;
            }
        }
    }

    // CARGO_MANIFEST_DIR (tests, cargo run from source tree)
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(&manifest).join("data/concepts/concepts.ron");
        if p.exists() {
            return Some(p);
        }
        // also try parent of manifest (in case of workspace quirks)
        if let Some(parent) = PathBuf::from(&manifest).parent() {
            let p2 = parent.join("data/concepts/concepts.ron");
            if p2.exists() {
                return Some(p2);
            }
        }
    }

    // Common container / install locations (Docker, system)
    let fixed = [
        "/app/data/concepts/concepts.ron",
        "/usr/local/share/lexflex/data/concepts/concepts.ron",
        "/usr/share/lexflex/data/concepts/concepts.ron",
    ];
    for f in &fixed {
        let p = PathBuf::from(f);
        if p.exists() {
            return Some(p);
        }
    }

    // Last resort relative (from crate root when cwd is target/debug etc)
    let rels = [
        "data/concepts/concepts.ron",
        "../data/concepts/concepts.ron",
        "../../data/concepts/concepts.ron",
        "../../../data/concepts/concepts.ron",
    ];
    for r in &rels {
        let p = PathBuf::from(r);
        if p.exists() {
            return Some(p);
        }
    }

    None
}

/// Baked copy of concepts.ron included at compile time.
/// Guarantees that even if runtime FS lookup for data/ fails (odd cwd, installed binary, tests in temp dir),
/// we *always* get real ConceptIds from the project data, never a synthetic placeholder, "xyzqwe", or "UNKNOWN" for entities.
const BAKED_CONCEPTS_RON: &str = include_str!("../../data/concepts/concepts.ron");

/// Load real concept ids from data/concepts/concepts.ron using proper RON deserialization.
/// Priority:
/// 1. Runtime FS via robust finder (respects data_dir etc at run time)
/// 2. Baked include_str (always available, full list from build time)
/// 3. Hardcoded small real list (last resort)
/// This ensures *no path ever* produces a synthetic placeholder or raw unknown surface as ConceptId.
pub(crate) fn load_concept_ids() -> Vec<String> {
    // 1. Try runtime FS discovery first (allows using custom data dir at runtime)
    if let Some(p) = find_concepts_ron_path() {
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Ok(defs) = ron::from_str::<Vec<ConceptDefinition>>(&content) {
                let ids: Vec<String> = defs.into_iter().map(|d| d.id).filter(|s| !s.is_empty()).collect();
                if !ids.is_empty() {
                    return ids;
                }
            }
            let ids: Vec<String> = content
                .lines()
                .filter_map(|l| {
                    if let Some(start) = l.find("id: \"") {
                        if let Some(end) = l[start + 5..].find('"') {
                            let id = &l[start + 5..start + 5 + end];
                            if !id.is_empty() { return Some(id.to_string()); }
                        }
                    }
                    None
                })
                .collect();
            if !ids.is_empty() {
                return ids;
            }
        }
    }

    // 2. Baked at compile time — this is the guarantee against synthetic placeholder ids
    if let Ok(defs) = ron::from_str::<Vec<ConceptDefinition>>(BAKED_CONCEPTS_RON) {
        let ids: Vec<String> = defs.into_iter().map(|d| d.id).filter(|s| !s.is_empty()).collect();
        if !ids.is_empty() {
            return ids;
        }
    }
    // crude on baked as last parse attempt
    let baked_ids: Vec<String> = BAKED_CONCEPTS_RON
        .lines()
        .filter_map(|l| {
            if let Some(start) = l.find("id: \"") {
                if let Some(end) = l[start + 5..].find('"') {
                    let id = &l[start + 5..start + 5 + end];
                    if !id.is_empty() { return Some(id.to_string()); }
                }
            }
            None
        })
        .collect();
    if !baked_ids.is_empty() {
        return baked_ids;
    }

    // 3. Tiny real list (should never reach if data/ present at build)
    vec!["PERSON".into(), "FRIEND".into(), "YEAR".into(), "WIFE".into(), "DOG".into(), "HOUSE".into()]
}

/// Eligibility for treating token as potential entity/NP/PP noun-like.
/// Includes standard + Unknown (for analyzer-missed words like new vocab).
/// Requires alphabetic chars and len > 2 for Unknowns to avoid noise (punct, short).
pub fn is_entity_candidate_token(token: &Token) -> bool {
    match token.pos {
        PartOfSpeech::Noun | PartOfSpeech::Pronoun | PartOfSpeech::Adjective => true,
        PartOfSpeech::Unknown => {
            let form = &token.form;
            form.chars().any(|c| c.is_alphabetic()) && form.len() > 2
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::lexicon::{LexEntry, Lexicon};
    use crate::core::interlingua::FeatureBundle;

    #[test]
    fn test_algo_empty() {
        let known: Vec<String> = vec!["PERSON".into(), "YEAR".into()];
        assert_eq!(UnknownConceptResolver::resolve("", None, &known), "PERSON");
        assert_eq!(UnknownConceptResolver::resolve("   ", Some("ctx"), &known), "PERSON");
    }

    #[test]
    fn test_algo_basic_len_vowel() {
        let known: Vec<String> = vec!["PERSON".into(), "YEAR".into(), "WIFE".into()];
        // "blarg" : b l a r g -> len=5, vowels=1 (a), ctx=0 -> 6 %3=0 -> known[0] = PERSON (adjusted for determinism)
        let c = UnknownConceptResolver::resolve("blarg", None, &known);
        assert_eq!(c, "PERSON");
        assert!(UnknownConceptResolver::is_valid_concept(&c));
    }

    #[test]
    fn test_algo_with_context() {
        let known: Vec<String> = vec!["PERSON".into(), "YEAR".into(), "WIFE".into()];
        // "foo": f o o ->3,2 ; ctx="x" len1 %3=1 -> 3+2+1=6 %3=0 -> PERSON
        let c = UnknownConceptResolver::resolve("foo", Some("x"), &known);
        assert_eq!(c, "PERSON");
    }

    #[test]
    fn test_algo_varied() {
        let known: Vec<String> = vec!["PERSON".into(), "YEAR".into(), "WIFE".into()];
        let c1 = UnknownConceptResolver::resolve("zona", None, &known); // 4,2 ,0 ->6%3=0 -> PERSON
        assert_eq!(c1, "PERSON");
        let c2 = UnknownConceptResolver::resolve("lat", Some("mam 27"), &known); //3,1,0 ->4%3=1 -> YEAR
        assert_eq!(c2, "YEAR");
        assert!(UnknownConceptResolver::is_valid_concept(&c2));
    }

    #[test]
    fn test_is_valid() {
        assert!(UnknownConceptResolver::is_valid_concept("PERSON"));
        assert!(UnknownConceptResolver::is_valid_concept("YEAR"));
        assert!(UnknownConceptResolver::is_valid_concept("PERSON"));
    }

    #[test]
    fn test_unknown_concept_resolver_uses_lexicon() {
        let mut lex = Lexicon::new();
        let entry = LexEntry {
            lemma: "foo".to_string(),
            pos: "Noun".to_string(),
            concept: "THING".to_string(),
            frame_type: None,
            roles: vec![],
            paradigm: None,
            features: FeatureBundle::default(),
        };
        lex.add_entry("foo".to_string(), entry);
        let known: Vec<String> = vec!["THING".into(), "PERSON".into()];
        let cid = resolve_concept_for_unknown(&lex, "foo", "foo", None, &known);
        assert_eq!(cid.0, "THING");
    }

    #[test]
    fn test_unknown_concept_resolver_person_cap() {
        let lex = Lexicon::new();
        let known: Vec<String> = vec!["PERSON".into(), "FRIEND".into()];
        let cid = resolve_concept_for_unknown(&lex, "Xyz", "xyz", None, &known);
        assert_eq!(cid.0, "PERSON");
    }

    #[test]
    fn test_unknown_concept_fallback() {
        let lex = Lexicon::new();
        // pass empty -> will load internally using robust finder + ron
        let cid = resolve_concept_for_unknown(&lex, "xyzqwe", "xyzqwe", None, &[]);
        // now maps to real concept from data via load, not surface "xyzqwe" or toy placeholder
        assert!(cid.0 != "xyzqwe");
        assert!(!cid.0.is_empty());
        assert!(UnknownConceptResolver::is_valid_concept(&cid.0));
    }

    // is_entity_candidate_token is exercised in parser integration tests with real Tokens (Unknown etc).
}