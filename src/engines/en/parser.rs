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

        let (frame_type, roles, verb_concept) = if let Some(entry) = verb_entry {
            if let Some(ref ft) = entry.frame_type {
                let parsed_roles: Vec<SemanticRole> = entry.roles.iter()
                    .filter_map(|r| parse_role_str(r))
                    .collect();
                (ft.clone(), parsed_roles, entry.concept.clone())
            } else {
                ("Statement".to_string(), vec![SemanticRole::Topic, SemanticRole::Theme], entry.concept.clone())
            }
        } else {
            ("Statement".to_string(), vec![SemanticRole::Topic, SemanticRole::Theme], "BE".to_string())
        };

        sentence.tense = verb_token.features.tense.or(Some(Tense::Present));
        sentence.aspect = verb_token.features.aspect;

        let negation = tokens.iter().any(|t| t.pos == PartOfSpeech::Negation);
        if negation {
            sentence.polarity = Polarity::Negative;
        }

        // Detect questions (does/do/did / ? ). Sets illocution so PL generator emits "Czy ... ?"
        let is_question = tokens.iter().any(|t| {
            let f = t.form.to_lowercase();
            f == "did" || f == "does" || f == "do" || f == "?"
        }) || tokens.first().map_or(false, |t| t.form.eq_ignore_ascii_case("did") || t.form.eq_ignore_ascii_case("does") || t.form.eq_ignore_ascii_case("do"));
        if is_question {
            sentence.illocution = Illocution::Question;
            if tokens.iter().any(|t| t.form.eq_ignore_ascii_case("did")) {
                sentence.tense = Some(Tense::Past);
            }
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
                    || t.pos == PartOfSpeech::Pronoun
                    || t.pos == PartOfSpeech::Adjective)
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

            // Early normalization via lexicon (shared, kills apple/kot contains).
            // Degree recovery is data-driven via lexicon entry features when present.
            // General fallback for regular comparative forms.
            self.lexicon.normalize_entity(&mut entity);
            if let Some(nm) = &entity.name {
                if entity.features.degree.is_none() && nm.ends_with("er") && nm.len() > 3 {
                    let base = nm.trim_end_matches("er").to_string();
                    entity.name = Some(base);
                    entity.features.degree = Some(Degree::Comparative);
                }
            }

            entities.push(entity);
        }

        // Group preceding Adjectives to following Noun (for comparative + noun, e.g. big red apple)
        {
            let mut grouped: Vec<Entity> = vec![];
            let mut j = 0;
            while j < entities.len() {
                let mut k = j;
                while k < entities.len() {
                    let nm = entities[k].name.as_deref().unwrap_or("");
                    let is_adj = if let Some(e) = self.lexicon.lookup_by_form(&nm.to_lowercase()).or_else(|| self.lexicon.lookup_by_lemma(nm)) {
                        e.pos == "Adjective"
                    } else { nm.ends_with("er") || nm.ends_with('y') };
                    if !is_adj { break; }
                    k += 1;
                }
                if k > j && k < entities.len() {
                    // Proper attachment: attach preceding adjs as modifiers on the head noun entity.
                    // Structural .adjectives attachment (RESOLVED 21pts: no concat of names).
                    let mut head = entities[k].clone();
                    for ii in j..k {
                        let mut adj = entities[ii].clone();
                        if let Some(d) = adj.features.degree.or(entities[ii].features.degree) {
                            adj.features.degree = Some(d);
                        }
                        if head.features.gender.is_none() {
                            head.features.gender = adj.features.gender;
                        }
                        head.adjectives.push(adj);
                    }
                    // Do not concat name for adjs; keep clean noun name via norm. Adjs are in .adjectives.
                    self.lexicon.normalize_entity(&mut head);
                    grouped.push(head);
                    j = k + 1;
                } else {
                    grouped.push(entities[j].clone());
                    j += 1;
                }
            }
            entities = grouped;
        }

        // debug for degree attach
        // First-class Coordination: group NPs joined by "and"/"i" into Coordination struct on the representative Entity.
        if tokens.iter().any(|t| t.form == "and" || t.form == "i") && entities.len() >= 2 {
            let e1 = entities.remove(0);
            let e2 = entities.remove(0);
            let conj = if tokens.iter().any(|t| t.form == "and") { "and".to_string() } else { "i".to_string() };
            let coord = Coordination { items: vec![e1.clone(), e2.clone()], conjunction: conj };
            let mut coord_entity = e1;
            coord_entity.coordination = Some(coord);
            coord_entity.features.number = Some(Number::Plural);
            entities.insert(0, coord_entity);
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

        // detect numerical
        if sentence.quantification.is_none() {
            for t in tokens {
                if let Ok(n) = t.form.parse::<i32>() {
                    sentence.quantification = Some(Quantifier::Numerical(n));
                    break;
                }
                let num = match t.form.as_str() {
                    "three" => Some(3),
                    "four" => Some(4),
                    "five" => Some(5),
                    "ten" => Some(10),
                    "twenty" => Some(20),
                    "thirty" => Some(30),
                    _ => None,
                };
                if let Some(n) = num {
                    sentence.quantification = Some(Quantifier::Numerical(n));
                    break;
                }
            }
        }

        let frame = self.build_frame(&frame_type, &roles, &entities, &verb_concept)?;
        sentence.frames.push(frame);

        Ok(Utterance::single_sentence(sentence))
    }

    fn build_frame(
        &self,
        frame_type: &str,
        _roles: &[SemanticRole],
        entities: &[Entity],
        verb_concept: &str,
    ) -> Result<Frame, ParseError> {
        // English SVO: first NP = Agent, second NP = Theme/Patient, third NP = Recipient
        // For Transfer: Agent V Theme (to Recipient)
        let agent = entities.first().cloned().unwrap_or(Entity::new(ConceptId::new("unknown")));
        let theme = entities.get(1).cloned().unwrap_or(Entity::new(ConceptId::new("unknown")));
        let recipient = entities.get(2).cloned();

        match frame_type {
            "Possession" => Ok(Frame::Possession {
                possessor: agent,
                possessed: theme,
                verb_concept: verb_concept.to_string(),
            }),
            "Transfer" => Ok(Frame::Transfer {
                agent,
                recipient: recipient.unwrap_or(Entity::new(ConceptId::new("unknown"))),
                theme,
                verb_concept: verb_concept.to_string(),
            }),
            "Motion" => Ok(Frame::Motion {
                mover: agent,
                source: None,
                goal: entities.get(1).cloned(),
                path: None,
                verb_concept: verb_concept.to_string(),
            }),
            "Perception" => Ok(Frame::Perception {
                experiencer: agent,
                stimulus: theme,
                verb_concept: verb_concept.to_string(),
            }),
            "Cognition" => Ok(Frame::Cognition {
                cognizer: agent,
                content: theme,
                verb_concept: verb_concept.to_string(),
            }),
            "Emotion" => Ok(Frame::Emotion {
                experiencer: agent,
                stimulus: theme,
                verb_concept: verb_concept.to_string(),
            }),
            "Destruction" => Ok(Frame::Destruction {
                agent,
                patient: theme,
                instrument: None,
                verb_concept: verb_concept.to_string(),
            }),
            "Consumption" => Ok(Frame::Consumption {
                agent,
                patient: theme,
                verb_concept: verb_concept.to_string(),
            }),
            "Communication" => Ok(Frame::Communication {
                speaker: agent,
                addressee: recipient,
                message: theme,
                verb_concept: verb_concept.to_string(),
            }),
            "Creation" => Ok(Frame::Creation {
                creator: agent,
                created: theme,
                material: None,
                verb_concept: verb_concept.to_string(),
            }),
            _ => Ok(Frame::Statement {
                subject: entities.first().cloned().unwrap_or(Entity::new(ConceptId::new("unknown"))),
                property: entities.get(1).cloned().unwrap_or(Entity::new(ConceptId::new("unknown"))),
                verb_concept: verb_concept.to_string(),
            }),
        }
    }
}

fn parse_role_str(s: &str) -> Option<SemanticRole> {
    crate::core::utils::parse_role_str(s)
}
