use crate::core::deduction::{self, DeductionContext};
use crate::core::interlingua::*;
use crate::core::ontology::Ontology;
use crate::data::lexicon::Lexicon;
use crate::engines::en::morphology::EnglishMorphology;
use crate::error::ParseError;

pub struct EnglishParser {
    lexicon: Lexicon,
    _morphology: EnglishMorphology,
    ontology: Ontology,
}

impl EnglishParser {
    pub fn new(lexicon: Lexicon, morphology: EnglishMorphology, ontology: Ontology) -> Self {
        Self {
            lexicon,
            _morphology: morphology,
            ontology,
        }
    }

    pub fn parse(&self, input: &str) -> Result<Utterance, ParseError> {
        if input.trim().is_empty() {
            return Err(ParseError::EmptyInput);
        }

        let tokens = self.tokenize(input);
        let partial = self.build_partial_structure(&tokens)?;

        let context = DeductionContext::new(
            &self.lexicon,
            &self.ontology,
            LanguageId::new("en"),
        );
        let utterance = deduction::deduce(partial, &context)
            .map_err(|_| ParseError::NoVerbFound)?;

        Ok(utterance)
    }

    fn tokenize(&self, input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut offset = 0;

        for word in input.split_whitespace() {
            let clean = word.trim_matches(|c: char| c.is_ascii_punctuation());
            let form = clean.to_lowercase();

            let (pos, features, lemma) = self.analyze_token(&form);

            tokens.push(Token {
                form: form.clone(),
                lemma: Some(lemma),
                pos,
                features,
                span: (offset, offset + word.len()),
            });

            offset += word.len() + 1;
        }

        tokens
    }

    fn analyze_token(&self, form: &str) -> (PartOfSpeech, FeatureBundle, String) {
        if form == "not" || form == "n't" {
            return (PartOfSpeech::Negation, FeatureBundle::default(), "not".to_string());
        }
        if form == "a" || form == "an" || form == "the" {
            let def = if form == "the" {
                Definiteness::Definite
            } else {
                Definiteness::Indefinite
            };
            return (
                PartOfSpeech::Determiner,
                FeatureBundle {
                    definiteness: Some(def),
                    ..Default::default()
                },
                form.to_string(),
            );
        }

        if let Some(entry) = self.lexicon.lookup_by_form(form) {
            let pos = self.lexicon.parse_pos(&entry.pos);
            return (pos, entry.features.clone(), entry.lemma.clone());
        }

        if form.ends_with("s") && !form.ends_with("ss") {
            let stem = &form[..form.len() - 1];
            if let Some(entry) = self.lexicon.lookup_by_form(stem) {
                let pos = self.lexicon.parse_pos(&entry.pos);
                return (pos, entry.features.clone(), entry.lemma.clone());
            }
        }

        if form.ends_with("ed") {
            let stem = &form[..form.len() - 2];
            if let Some(entry) = self.lexicon.lookup_by_form(stem) {
                if entry.pos == "Verb" {
                    return (
                        PartOfSpeech::Verb,
                        FeatureBundle {
                            tense: Some(Tense::Past),
                            ..entry.features.clone()
                        },
                        entry.lemma.clone(),
                    );
                }
            }
        }

        (PartOfSpeech::Unknown, FeatureBundle::default(), form.to_string())
    }

    fn build_partial_structure(&self, tokens: &[Token]) -> Result<Utterance, ParseError> {
        let mut sentence = Sentence::new();

        let verb_idx = tokens
            .iter()
            .position(|t| t.pos == PartOfSpeech::Verb)
            .ok_or(ParseError::NoVerbFound)?;

        let verb_token = &tokens[verb_idx];
        let verb_lemma = verb_token.lemma.as_deref().unwrap_or(&verb_token.form);

        let verb_entry = self.lexicon.lookup_by_lemma(verb_lemma)
            .or_else(|| self.lexicon.lookup_by_form(verb_lemma));

        let (frame_type, roles) = if let Some(entry) = verb_entry {
            if let Some(ref ft) = entry.frame_type {
                let parsed_roles: Vec<SemanticRole> = entry.roles.iter()
                    .filter_map(|r| parse_role_str(r))
                    .collect();
                (ft.clone(), parsed_roles)
            } else {
                ("Statement".to_string(), vec![SemanticRole::Topic, SemanticRole::Theme])
            }
        } else {
            ("Statement".to_string(), vec![SemanticRole::Topic, SemanticRole::Theme])
        };

        sentence.tense = verb_token.features.tense.or(Some(Tense::Present));
        sentence.aspect = verb_token.features.aspect;

        let negation = tokens.iter().any(|t| t.pos == PartOfSpeech::Negation);
        if negation {
            sentence.polarity = Polarity::Negative;
        }

        // Detect passive voice: be + past participle
        let has_be_aux = tokens.iter().any(|t| {
            matches!(t.lemma.as_deref(), Some("be"))
        });
        let has_past_participle = tokens.iter().any(|t| {
            t.pos == PartOfSpeech::Participle
        });
        if has_be_aux && has_past_participle {
            sentence.voice = Some(Voice::Passive);
        }

        let np_tokens: Vec<&Token> = tokens
            .iter()
            .filter(|t| {
                (t.pos == PartOfSpeech::Noun
                    || t.pos == PartOfSpeech::Pronoun)
                    && t.pos != PartOfSpeech::Particle
            })
            .collect();

        let mut entities: Vec<Entity> = Vec::new();
        for (i, np) in np_tokens.iter().enumerate() {
            let lemma = np.lemma.as_deref().unwrap_or(&np.form);
            let entry = self.lexicon.lookup_by_form(&np.form)
                .or_else(|| self.lexicon.lookup_by_lemma(lemma));

            let concept = if let Some(e) = entry {
                ConceptId::new(&e.concept)
            } else {
                ConceptId::new(lemma)
            };

            let mut entity = Entity::new(concept)
                .with_name(lemma);
            entity.features = np.features.clone();
            if let Some(e) = entry {
                if entity.features.gender.is_none() {
                    entity.features.gender = e.features.gender;
                }
                if entity.features.animacy.is_none() {
                    entity.features.animacy = e.features.animacy;
                }
            }
            entity.features.number = Some(Number::Singular);

            let prev_token = if i > 0 {
                tokens.iter().find(|t| t.pos == PartOfSpeech::Determiner)
            } else {
                None
            };
            if let Some(det) = prev_token {
                entity.features.definiteness = det.features.definiteness;
            }

            entities.push(entity);
        }

        // Detect quantification
        let quantifier_token = tokens.iter().find(|t| {
            matches!(t.form.as_str(),
                "all" | "every" | "everyone" | "everything" |  // universal
                "some" | "someone" | "something" |  // existential
                "no" | "none" | "nobody" | "nothing" |  // negated existential
                "many" | "much" |  // many/much
                "few" | "several" |  // few/several
                "most"  // most
            )
        });
        if let Some(qt) = quantifier_token {
            sentence.quantification = Some(match qt.form.as_str() {
                "all" | "every" | "everyone" | "everything" => Quantifier::Universal,
                "some" | "someone" | "something" => Quantifier::Existential,
                "no" | "none" | "nobody" | "nothing" => Quantifier::NegatedExistential,
                "many" | "much" => Quantifier::Proportional("many".to_string()),
                "few" | "several" => Quantifier::Proportional("few".to_string()),
                "most" => Quantifier::Proportional("most".to_string()),
                _ => Quantifier::Existential,
            });
        }

        let frame = self.build_frame(&frame_type, &roles, &entities)?;
        sentence.frames.push(frame);

        Ok(Utterance::single_sentence(sentence))
    }

    fn build_frame(
        &self,
        frame_type: &str,
        _roles: &[SemanticRole],
        entities: &[Entity],
    ) -> Result<Frame, ParseError> {
        // English SVO: first NP = Agent, second NP = Theme/Patient, third NP = Recipient
        // For Transfer: Agent V Theme (to Recipient)
        let agent = entities.first().cloned().unwrap_or(Entity::new(ConceptId::new("unknown")));
        let theme = entities.get(1).cloned().unwrap_or(Entity::new(ConceptId::new("unknown")));
        let recipient = entities.get(2).cloned();

        match frame_type {
            "Transfer" => Ok(Frame::Transfer {
                agent,
                recipient: recipient.unwrap_or(Entity::new(ConceptId::new("unknown"))),
                theme,
            }),
            "Motion" => Ok(Frame::Motion {
                mover: agent,
                source: None,
                goal: entities.get(1).cloned(),
                path: None,
            }),
            "Perception" => Ok(Frame::Perception {
                experiencer: agent,
                stimulus: theme,
            }),
            "Cognition" => Ok(Frame::Cognition {
                cognizer: agent,
                content: theme,
            }),
            "Emotion" => Ok(Frame::Emotion {
                experiencer: agent,
                stimulus: theme,
            }),
            "Destruction" => Ok(Frame::Destruction {
                agent,
                patient: theme,
                instrument: None,
            }),
            "Consumption" => Ok(Frame::Consumption {
                agent,
                patient: theme,
            }),
            "Communication" => Ok(Frame::Communication {
                speaker: agent,
                addressee: recipient,
                message: theme,
            }),
            "Creation" => Ok(Frame::Creation {
                creator: agent,
                created: theme,
                material: None,
            }),
            _ => Ok(Frame::Statement {
                subject: entities.first().cloned().unwrap_or(Entity::new(ConceptId::new("unknown"))),
                property: entities.get(1).cloned().unwrap_or(Entity::new(ConceptId::new("unknown"))),
            }),
        }
    }
}

fn parse_role_str(s: &str) -> Option<SemanticRole> {
    crate::core::utils::parse_role_str(s)
}
