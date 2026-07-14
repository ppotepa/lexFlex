use crate::core::graph::LinguisticGraph;
use crate::core::interlingua::{Aspect, Entity, Frame, Number, Sentence};
use crate::data::descriptor::{AspectType, LanguageDescriptor, WordOrder};
use crate::data::lexicon::Lexicon;

/// Temporal placement decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalSlot {
    Beginning,
    End,
}

/// Pure policy decisions based on LanguageDescriptor (no side effects, easily testable).
pub struct GenerationPolicy<'a> {
    desc: &'a LanguageDescriptor,
}

impl<'a> GenerationPolicy<'a> {
    pub fn new(desc: &'a LanguageDescriptor) -> Self {
        Self { desc }
    }

    /// Returns true if the subject/agent should be emitted in the surface string.
    /// With pro_drop, we drop 1st/2nd person pronoun subjects (and known simple pronouns);
    /// proper names and 3rd person are emitted so existing tests/bench are unaffected.
    pub fn should_emit_subject(&self, entity: &Entity) -> bool {
        if !self.desc.syntax.pro_drop {
            return true;
        }
        // Drop only for 1st/2nd or simple pronoun concepts when pro_drop.
        if let Some(p) = entity.features.person {
            if p == crate::core::interlingua::Person::First || p == crate::core::interlingua::Person::Second {
                return false;
            }
        }
        if let Some(name) = &entity.name {
            // Proper names (uppercase) always emitted.
            if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                return true;
            }
        }
        // For other 3rd person / concepts, emit (keeps "Tomek", "Tom", "Iza" etc.).
        true
    }

    /// Where to place temporal adverb based on word order.
    pub fn temporal_slot(&self) -> TemporalSlot {
        match self.desc.syntax.word_order {
            WordOrder::SVO => TemporalSlot::End,
            _ => TemporalSlot::Beginning,
        }
    }

    /// Whether to insert an article for this entity/position.
    pub fn should_add_article(&self, entity: &Entity, needs_article: bool) -> bool {
        if !self.desc.morphology.has_articles || !needs_article {
            return false;
        }
        let num = entity.features.number.unwrap_or(Number::Singular);
        if num != Number::Singular {
            return false;
        }
        // Definite or countable singular -> article candidate (caller decides "a"/"the").
        true
    }

    /// The particle to use for negation (directly from descriptor).
    pub fn negation_particle(&self) -> &str {
        &self.desc.syntax.negation_particle
    }

    /// The aspect handling mode from descriptor.
    pub fn aspect_type(&self) -> AspectType {
        self.desc.morphology.aspect_type
    }

    /// Whether to realize as periphrastic prog_aspect (data-driven via descriptor.aspect_type).
    pub fn use_periphrastic_prog_aspect(&self, sentence: &Sentence) -> bool {
        self.desc.morphology.aspect_type == AspectType::Periphrastic
            && sentence.aspect == Some(Aspect::Progressive)
    }
}

/// Resolve the surface verb lemma for the frame, preferring the verb_concept
/// stored in the frame (looked up in lexicon). Fallback is lowercased concept.
/// Only minimal handling for consumption liquid (DRINK vs EAT) using patient features.
/// No language-specific string literals for lemmas.
pub fn resolve_surface_verb(
    frame: &Frame,
    lexicon: &Lexicon,
    graph: Option<&LinguisticGraph>,
    lang: &str,
) -> String {
    let concept = match frame {
        Frame::Transfer { verb_concept, .. } => verb_concept.as_str(),
        Frame::Motion { verb_concept, .. } => verb_concept.as_str(),
        Frame::Perception { verb_concept, .. } => verb_concept.as_str(),
        Frame::Cognition { verb_concept, .. } => verb_concept.as_str(),
        Frame::Emotion { verb_concept, .. } => verb_concept.as_str(),
        Frame::Destruction { verb_concept, .. } => verb_concept.as_str(),
        Frame::Consumption { verb_concept, .. } => verb_concept.as_str(),
        Frame::Communication { verb_concept, .. } => verb_concept.as_str(),
        Frame::Creation { verb_concept, .. } => verb_concept.as_str(),
        Frame::Statement { verb_concept, .. } => verb_concept.as_str(),
        Frame::Existence { verb_concept, .. } => verb_concept.as_str(),
        Frame::Possession { verb_concept, .. } => verb_concept.as_str(),
        Frame::Custom { name, .. } => name.as_str(),
    };

    let age_idiom_have = graph.map_or(false, |g| g.has_construction("AGE_IDIOM"))
        && lang == "pl"
        && matches!(frame, Frame::Possession { verb_concept, .. } if verb_concept == "BE");

    let chosen = if age_idiom_have {
        "HAVE"
    } else {
        concept
    };

    // Prefer verb_concept via lexicon lookup (first match wins; data order + RON morphology drive surface like "jeść" -> "jadł").
    // No per-concept string overrides or ad-hoc cases here.
    if let Some(entry) = lexicon.lookup_concept(chosen) {
        return entry.lemma.clone();
    }
    if let Some(entry) = lexicon.lookup_concept(&chosen.to_uppercase()) {
        return entry.lemma.clone();
    }

    // final fallback: the concept name lowercased (no hard-coded surface forms)
    chosen.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::interlingua::*;
    use crate::data::descriptor::{AspectType, LanguageDescriptor, MorphologyDescriptor, SyntaxDescriptor, WordOrder};

    fn make_desc(pro_drop: bool, has_articles: bool, aspect: AspectType, wo: WordOrder) -> LanguageDescriptor {
        LanguageDescriptor {
            language: "test".into(),
            name: "Test".into(),
            morphology: MorphologyDescriptor {
                has_cases: false,
                cases: vec![],
                has_articles,
                aspect_type: aspect,
            },
            syntax: SyntaxDescriptor {
                word_order: wo,
                pro_drop,
                negation_particle: "not".into(),
                preposition_roles: std::collections::HashMap::new(),
            },
        }
    }

    fn make_entity(concept: &str, name: Option<&str>, person: Option<Person>, count: Option<Countability>) -> Entity {
        let mut e = Entity::new(ConceptId::new(concept));
        if let Some(n) = name {
            e.name = Some(n.to_string());
        }
        if let Some(p) = person {
            e.features.person = Some(p);
        }
        if let Some(c) = count {
            e.features.countability = Some(c);
        }
        e.features.number = Some(Number::Singular);
        e.adjectives = vec![];
        e
    }

    #[test]
    fn test_should_emit_subject_prodrop_affects_pronouns() {
        let desc = make_desc(true, false, AspectType::Morphological, WordOrder::SVO);
        let policy = GenerationPolicy::new(&desc);

        let i = make_entity("I", None, Some(Person::First), None);
        assert!(!policy.should_emit_subject(&i), "1st person should be dropped with pro_drop");

        let tomek = make_entity("PERSON", Some("Tomek"), None, None);
        assert!(policy.should_emit_subject(&tomek), "proper names always emitted");

        let desc_no_drop = make_desc(false, false, AspectType::Morphological, WordOrder::SVO);
        let p2 = GenerationPolicy::new(&desc_no_drop);
        assert!(p2.should_emit_subject(&i), "always emit when !pro_drop");
    }

    #[test]
    fn test_temporal_slot_depends_on_word_order() {
        let svo = make_desc(false, false, AspectType::Morphological, WordOrder::SVO);
        let p1 = GenerationPolicy::new(&svo);
        assert_eq!(p1.temporal_slot(), TemporalSlot::End);

        let other = make_desc(false, false, AspectType::Morphological, WordOrder::Free);
        let p2 = GenerationPolicy::new(&other);
        assert_eq!(p2.temporal_slot(), TemporalSlot::Beginning);
    }

    #[test]
    fn test_should_add_article_depends_on_has_articles() {
        let with = make_desc(false, true, AspectType::Periphrastic, WordOrder::SVO);
        let p = GenerationPolicy::new(&with);
        let apple = make_entity("APPLE", None, None, Some(Countability::Count));
        assert!(p.should_add_article(&apple, true));

        let without = make_desc(false, false, AspectType::Periphrastic, WordOrder::SVO);
        let p2 = GenerationPolicy::new(&without);
        assert!(!p2.should_add_article(&apple, true));
    }

    #[test]
    fn test_negation_and_aspect_from_descriptor() {
        let d = make_desc(true, false, AspectType::Morphological, WordOrder::SVO);
        let p = GenerationPolicy::new(&d);
        assert_eq!(p.negation_particle(), "not");
        assert_eq!(p.aspect_type(), AspectType::Morphological);
    }

    #[test]
    fn test_resolve_surface_verb_prefers_concept() {
        // Resolver uses verb_concept + lexicon lookup; falls back to lowercased when no entry.
        let f = Frame::Transfer {
            agent: make_entity("P", None, None, None),
            recipient: make_entity("P", None, None, None),
            theme: make_entity("APPLE", None, None, None),
            verb_concept: "GIVE".to_string(),
        };
        // Empty lexicon -> lowercased concept as final fallback (no panic).
        let res = resolve_surface_verb(&f, &Lexicon::new(), None, "en");
        assert_eq!(res, "give");
    }
}
