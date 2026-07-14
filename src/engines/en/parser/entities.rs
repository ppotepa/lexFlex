use super::*;

impl EnglishParser {
    pub(super) fn pp_span_indices(tokens: &[Token]) -> std::collections::HashSet<usize> {
        let mut span = std::collections::HashSet::new();
        for (i, token) in tokens.iter().enumerate() {
            if token.pos != PartOfSpeech::Preposition {
                continue;
            }
            let prep = token.form.to_lowercase();
            if !matches!(prep.as_str(), "with" | "in" | "on" | "at") {
                continue;
            }
            span.insert(i);
            let mut k = i + 1;
            while k < tokens.len() {
                let t = &tokens[k];
                if t.pos == PartOfSpeech::Verb {
                    break;
                }
                if t.pos == PartOfSpeech::Preposition && k > i + 1 {
                    break;
                }
                if matches!(
                    t.pos,
                    PartOfSpeech::Noun
                        | PartOfSpeech::Pronoun
                        | PartOfSpeech::Adjective
                        | PartOfSpeech::Determiner
                ) {
                    span.insert(k);
                } else if !matches!(t.form.as_str(), "," | "and" | "or") {
                    break;
                }
                k += 1;
            }
        }
        span
    }

    pub(super) fn token_to_entity(&self, token: &Token) -> Entity {
        let surface = &token.form;
        let lookup = surface.to_lowercase();
        let lemma = token.lemma.as_deref().unwrap_or(&lookup);
        let entry = self
            .lexicon
            .lookup_by_form(&lookup)
            .or_else(|| self.lexicon.lookup_by_lemma(lemma));
        let concept = resolve_concept_for_unknown(
            &self.lexicon,
            surface,
            lemma,
            None,
            &self.concept_ids,
            Some("en"),
        );
        let name = surface.clone();
        let mut entity = Entity::new(concept).with_name(&name);
        entity.features = token.features.clone();
        if let Some(e) = entry {
            if entity.features.gender.is_none() {
                entity.features.gender = e.features.gender;
            }
            if entity.features.animacy.is_none() {
                entity.features.animacy = e.features.animacy;
            }
        }
        if entity.features.number.is_none() {
            entity.features.number = Some(Number::Singular);
        }
        self.lexicon.normalize_entity(&mut entity);
        entity
    }

    pub(super) fn extract_pp_entities(&self, tokens: &[Token]) -> Vec<Entity> {
        let mut pp_entities: Vec<Entity> = Vec::new();
        for (i, token) in tokens.iter().enumerate() {
            if token.pos != PartOfSpeech::Preposition {
                continue;
            }
            let prep = token.form.to_lowercase();
            let is_with = prep == "with";
            let mut collected: Vec<Entity> = Vec::new();
            let mut k = i + 1;
            while k < tokens.len() {
                let t = &tokens[k];
                if t.pos == PartOfSpeech::Verb {
                    break;
                }
                if t.pos == PartOfSpeech::Preposition && k > i + 1 {
                    break;
                }
                if t.pos == PartOfSpeech::Determiner {
                    k += 1;
                    continue;
                }
                if is_entity_candidate_token(t) {
                    let mut ent = self.token_to_entity(t);
                    let is_person_context = self.ontology.is_animate_entity(&ent);
                    if is_with && is_person_context {
                        ent.features.semantic_role = Some(SemanticRole::Location);
                        ent.features.case = Some(Case::Instrumental);
                    } else if let Some(role) = self.descriptor.syntax.preposition_roles.get(&prep)
                    {
                        ent.features.semantic_role = Some(*role);
                        match role {
                            SemanticRole::Location => ent.features.case = Some(Case::Locative),
                            SemanticRole::Goal => ent.features.case = Some(Case::Accusative),
                            SemanticRole::Source => ent.features.case = Some(Case::Genitive),
                            _ => {}
                        }
                    }
                    collected.push(ent);
                } else if t.pos == PartOfSpeech::Adjective {
                    if let Some(last) = collected.last_mut() {
                        let adj = self.token_to_entity(t);
                        last.adjectives.push(adj);
                    }
                } else if !matches!(t.form.as_str(), "," | "and" | "or") {
                    break;
                }
                k += 1;
            }
            if collected.is_empty() {
                continue;
            }
            if collected.len() > 1 {
                let conj = if tokens.iter().any(|tt| tt.form == "and") {
                    "and".to_string()
                } else {
                    "or".to_string()
                };
                let coord = Coordination {
                    items: collected.clone(),
                    conjunction: conj,
                };
                let mut coord_ent = collected[0].clone();
                coord_ent.coordination = Some(coord);
                coord_ent.features.number = Some(Number::Plural);
                pp_entities.push(coord_ent);
            } else if let Some(entity) = collected.into_iter().next() {
                pp_entities.push(entity);
            }
        }
        pp_entities
    }

    pub(super) fn try_age_idiom_frame(
        &self,
        tokens: &[Token],
        entities: &[Entity],
        verb_concept: &str,
    ) -> Option<Frame> {
        if verb_concept != "BE" {
            return None;
        }
        let has_year = tokens.iter().any(|t| {
            self.lexicon
                .lookup_by_form(&t.form.to_lowercase())
                .is_some_and(|e| e.concept == "YEAR")
        });
        let has_old = tokens.iter().any(|t| {
            self.lexicon
                .lookup_by_form(&t.form.to_lowercase())
                .is_some_and(|e| e.concept == "OLD")
        });
        let has_number = tokens
            .iter()
            .any(|t| self.lexicon.cardinal_from_token(t).is_some());
        if !has_year || !has_number || !has_old {
            return None;
        }
        let possessor = entities.first()?.clone();
        let year_entry = tokens.iter().find_map(|t| {
            self.lexicon
                .lookup_by_form(&t.form.to_lowercase())
                .filter(|e| e.concept == "YEAR")
        })?;
        let mut possessed =
            Entity::new(ConceptId::new(&year_entry.concept)).with_name(&year_entry.lemma);
        possessed.features.number = Some(Number::Plural);
        if let Some(old_entry) = self
            .lexicon
            .lookup_by_form("old")
            .or_else(|| self.lexicon.lookup_concept("OLD"))
        {
            let mut adj =
                Entity::new(ConceptId::new(&old_entry.concept)).with_name(&old_entry.lemma);
            adj.features = old_entry.features.clone();
            possessed.adjectives.push(adj);
        }
        Some(Frame::Possession {
            possessor,
            possessed,
            verb_concept: "BE".to_string(),
        })
    }
}
