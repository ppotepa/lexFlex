use super::*;

impl EnglishParser {
    pub(super) fn definition_question(
        &self,
        tokens: &[Token],
        entities: &[Entity],
        verb_concept: &str,
    ) -> Option<QuestionSemantics> {
        if verb_concept != "BE" || entities.len() != 1 {
            return None;
        }
        let kind = tokens
            .iter()
            .find_map(|token| {
                self.lexicon
                    .lookup_by_form(&token.form)
                    .and_then(|entry| question_kind_from_concept(&ConceptId::new(&entry.concept)))
            })
            .or_else(|| {
                tokens
                    .first()
                    .is_some_and(|token| token.form.eq_ignore_ascii_case("is"))
                    .then_some(QuestionKind::YesNo)
            })?;
        if matches!(kind, QuestionKind::Where | QuestionKind::When | QuestionKind::HowMany | QuestionKind::Why | QuestionKind::How) {
            return None;
        }
        Some(QuestionSemantics::definition(kind, entities[0].clone()))
    }

    pub(super) fn question_semantics(
        &self,
        tokens: &[Token],
        entities: &[Entity],
        verb_concept: &str,
    ) -> Option<QuestionSemantics> {
        let kind = tokens
            .iter()
            .find_map(|token| {
                self.lexicon
                    .lookup_by_form(&token.form.to_lowercase())
                    .and_then(|entry| question_kind_from_concept(&ConceptId::new(&entry.concept)))
            })
            .or_else(|| {
                tokens
                    .first()
                    .is_some_and(|token| token.form.eq_ignore_ascii_case("is"))
                    .then_some(QuestionKind::YesNo)
            })
            .unwrap_or(QuestionKind::What);
        if verb_concept == "BE"
            && tokens.iter().any(|token| {
                self.lexicon
                    .lookup_by_form(&token.form.to_lowercase())
                    .is_some_and(|entry| entry.concept == "CAPITAL")
            })
        {
            let object = self.capital_object(tokens, entities)?;
            return Some(QuestionSemantics::relation(
                kind,
                ConceptId::new("CAPITAL_OF"),
                object,
                QueryProjection::Subject,
            ));
        }
        if (matches!(verb_concept, "LOCATED_IN") || matches!(kind, QuestionKind::Where))
            && !entities.is_empty()
        {
            return Some(QuestionSemantics::relation_from_subject(
                kind,
                ConceptId::new("LOCATED_IN"),
                entities[0].clone(),
                QueryProjection::Object,
            ));
        }
        if matches!(verb_concept, "BORN_IN") {
            let object = entities.last().cloned().or_else(|| {
                tokens
                    .iter()
                    .rev()
                    .find(|token| token.form.chars().any(char::is_alphabetic))
                    .map(|token| Entity::new(ConceptId::new("ENTITY")).with_name(&token.form))
            })?;
            return Some(QuestionSemantics::relation(
                kind,
                ConceptId::new("BORN_IN"),
                object,
                QueryProjection::Subject,
            ));
        }
        if tokens.iter().any(|token| {
            self.lexicon
                .lookup_by_form(&token.form.to_lowercase())
                .is_some_and(|entry| entry.concept == "POPULATION")
        }) {
            let subject = entities.last().cloned()?;
            return Some(QuestionSemantics::relation_from_subject(
                kind,
                ConceptId::new("POPULATION"),
                subject,
                QueryProjection::Object,
            ));
        }
        self.definition_question(tokens, entities, verb_concept)
    }

    pub(super) fn capital_relation(
        &self,
        tokens: &[Token],
        entities: &[Entity],
        pre_verb: &[bool],
        verb_concept: &str,
    ) -> Option<Frame> {
        if verb_concept != "BE"
            || !tokens.iter().any(|token| {
                self.lexicon
                    .lookup_by_form(&token.form)
                    .is_some_and(|entry| entry.concept == "CAPITAL")
            })
        {
            return None;
        }
        let subject = entities
            .iter()
            .find(|entity| entity.concept.0 != "CAPITAL")
            .cloned()
            .or_else(|| self.capital_subject(tokens))
            .or_else(|| {
                entities
                    .iter()
                    .zip(pre_verb)
                    .find(|(_, before)| **before)
                    .map(|(entity, _)| entity)
                    .cloned()
            })?;
        let object = self.capital_object(tokens, entities)?;
        Some(Frame::Custom {
            name: "CAPITAL_OF".to_string(),
            roles: vec![(SemanticRole::Topic, subject), (SemanticRole::Location, object)],
        })
    }

    fn capital_object(&self, tokens: &[Token], entities: &[Entity]) -> Option<Entity> {
        let capital_index = tokens.iter().position(|token| {
            self.lexicon
                .lookup_by_form(&token.form)
                .is_some_and(|entry| entry.concept == "CAPITAL")
        })?;
        let token = if let Some(of_index) = tokens
            .iter()
            .enumerate()
            .skip(capital_index + 1)
            .find_map(|(index, token)| token.form.eq_ignore_ascii_case("of").then_some(index))
        {
            tokens.iter().skip(of_index + 1).find(|token| {
                !matches!(token.form.to_lowercase().as_str(), "the" | "a" | "an")
                    && token.form.chars().any(char::is_alphabetic)
            })?
        } else {
            tokens.iter().skip(capital_index + 1).find(|token| {
                !matches!(
                    token.form.to_lowercase().as_str(),
                    "of" | "the" | "and" | "most" | "largest" | "city"
                ) && token.form.chars().any(char::is_alphabetic)
            })?
        };
        entities
            .iter()
            .find(|entity| {
                entity
                    .name
                    .as_deref()
                    .is_some_and(|name| name.eq_ignore_ascii_case(&token.form))
            })
            .cloned()
            .or_else(|| {
                let concept = self
                    .lexicon
                    .lookup_by_form(&token.form)
                    .map(|entry| entry.concept.clone())
                    .unwrap_or_else(|| "ENTITY".to_string());
                Some(Entity::new(ConceptId::new(&concept)).with_name(&token.form))
            })
    }

    fn capital_subject(&self, tokens: &[Token]) -> Option<Entity> {
        let capital_index = tokens.iter().position(|token| {
            self.lexicon
                .lookup_by_form(&token.form)
                .is_some_and(|entry| entry.concept == "CAPITAL")
        })?;
        let token = tokens[..capital_index].iter().rev().find(|token| {
            !matches!(token.form.to_lowercase().as_str(), "the" | "is" | "a" | "an")
                && token.form.chars().any(char::is_alphabetic)
        })?;
        Some(Entity::new(ConceptId::new("ENTITY")).with_name(&token.form))
    }
}
