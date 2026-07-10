use crate::core::deduction::{self, DeductionContext};
use crate::core::interlingua::*;
use crate::core::ontology::Ontology;
use crate::data::lexicon::Lexicon;
use crate::engines::pl::morphology::PolishMorphology;
use crate::error::ParseError;

pub struct PolishParser {
    lexicon: Lexicon,
    morphology: PolishMorphology,
    ontology: Ontology,
}

impl PolishParser {
    pub fn new(lexicon: Lexicon, morphology: PolishMorphology, ontology: Ontology) -> Self {
        Self {
            lexicon,
            morphology,
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
            LanguageId::new("pl"),
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
        if form == "nie" {
            return (PartOfSpeech::Negation, FeatureBundle::default(), "nie".to_string());
        }
        if form == "czy" {
            return (PartOfSpeech::Particle, FeatureBundle::default(), "czy".to_string());
        }

        // Try lexicon lookup first
        if let Some(entry) = self.lexicon.lookup_by_form(form) {
            let pos = self.lexicon.parse_pos(&entry.pos);
            return (pos, entry.features.clone(), entry.lemma.clone());
        }

        if let Some(entry) = self.lexicon.lookup_by_form(&form.to_lowercase()) {
            let pos = self.lexicon.parse_pos(&entry.pos);
            return (pos, entry.features.clone(), entry.lemma.clone());
        }

        // Try morphological analysis for unknown forms
        if let Some((pos, features, lemma)) = self.analyze_morphology(form) {
            return (pos, features, lemma);
        }

        (PartOfSpeech::Unknown, FeatureBundle::default(), form.to_string())
    }

    fn analyze_morphology(&self, form: &str) -> Option<(PartOfSpeech, FeatureBundle, String)> {
        // Try to analyze as past tense verb (ending in -ł, -ła, -ło, -li, -ły)
        if form.ends_with("ł") || form.ends_with("ła") || form.ends_with("ło") || form.ends_with("li") || form.ends_with("ły") {
            let (gender, number) = if form.ends_with("ła") {
                (Some(Gender::Feminine), Some(Number::Singular))
            } else if form.ends_with("ło") {
                (Some(Gender::Neuter), Some(Number::Singular))
            } else if form.ends_with("li") {
                (None, Some(Number::Plural))
            } else if form.ends_with("ły") {
                (None, Some(Number::Plural))
            } else {
                (Some(Gender::Masculine), Some(Number::Singular))
            };

            // Try to find base form by removing past tense ending
            let possible_lemmas = vec![
                form.trim_end_matches("ła").to_string() + "ć",
                form.trim_end_matches("ło").to_string() + "ć",
                form.trim_end_matches("li").to_string() + "ć",
                form.trim_end_matches("ły").to_string() + "ć",
                form.trim_end_matches("ł").to_string() + "ć",
            ];

            for lemma in possible_lemmas {
                if let Some(entry) = self.lexicon.lookup_by_lemma(&lemma) {
                    let mut features = entry.features.clone();
                    features.tense = Some(Tense::Past);
                    features.gender = gender;
                    features.number = number;
                    return Some((PartOfSpeech::Verb, features, lemma));
                }
            }
        }

        // Try to analyze as present tense verb (ending in -e, -esz, -emy, -ecie, -ą)
        if form.ends_with("e") || form.ends_with("esz") || form.ends_with("emy") || form.ends_with("ecie") || form.ends_with("ą") {
            let (person, number) = if form.ends_with("e") {
                (Some(Person::First), Some(Number::Singular))
            } else if form.ends_with("esz") {
                (Some(Person::Second), Some(Number::Singular))
            } else if form.ends_with("emy") {
                (Some(Person::First), Some(Number::Plural))
            } else if form.ends_with("ecie") {
                (Some(Person::Second), Some(Number::Plural))
            } else if form.ends_with("ą") {
                (Some(Person::Third), Some(Number::Plural))
            } else {
                (Some(Person::Third), Some(Number::Singular))
            };

            // Try to find base form
            let possible_lemmas = vec![
                form.trim_end_matches("e").to_string() + "ć",
                form.trim_end_matches("esz").to_string() + "ć",
                form.trim_end_matches("emy").to_string() + "ć",
                form.trim_end_matches("ecie").to_string() + "ć",
                form.trim_end_matches("ą").to_string() + "ć",
            ];

            for lemma in possible_lemmas {
                if let Some(entry) = self.lexicon.lookup_by_lemma(&lemma) {
                    let mut features = entry.features.clone();
                    features.tense = Some(Tense::Present);
                    features.person = person;
                    features.number = number;
                    return Some((PartOfSpeech::Verb, features, lemma));
                }
            }
        }

        None
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

        sentence.tense = verb_token.features.tense.or(Some(Tense::Past));
        sentence.aspect = verb_token.features.aspect;

        let negation = tokens.iter().any(|t| t.pos == PartOfSpeech::Negation);
        if negation {
            sentence.polarity = Polarity::Negative;
        }

        let question = tokens.iter().any(|t| t.form == "czy");
        if question {
            sentence.illocution = Illocution::Question;
        }

        // Detect passive voice: być/zostać + passive participle
        let has_passive_aux = tokens.iter().any(|t| {
            matches!(t.lemma.as_deref(), Some("być") | Some("zostać"))
        });
        let has_passive_participle = tokens.iter().any(|t| {
            t.pos == PartOfSpeech::Participle
                || t.form.ends_with("ny") || t.form.ends_with("na") || t.form.ends_with("ne")
                || t.form.ends_with("ty") || t.form.ends_with("ta") || t.form.ends_with("te")
        });
        if has_passive_aux && has_passive_participle {
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
        for np in &np_tokens {
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
                if entity.features.number.is_none() {
                    entity.features.number = e.features.number;
                }
            }
            if entity.features.number.is_none() {
                entity.features.number = Some(Number::Singular);
            }
            entities.push(entity);
        }

        let temporal_token = tokens.iter().find(|t| {
            matches!(t.form.as_str(),
                "wczoraj" | "dzisiaj" | "jutro" | "teraz" |
                "yesterday" | "today" | "tomorrow" | "now"
            )
        });
        if let Some(tt) = temporal_token {
            sentence.temporal = Some(TemporalReference::Deictic {
                word: tt.form.clone(),
            });
        }

        // Detect quantification
        let quantifier_token = tokens.iter().find(|t| {
            matches!(t.form.as_str(),
                "wszyscy" | "każdy" | "wszystko" |  // all, every
                "niektórzy" | "niektóre" | "coś" |  // some
                "nikt" | "nic" |  // none, nothing
                "wiele" | "wielu" | "dużo" |  // many, much
                "mało" | "kilka" |  // few, several
                "większość"  // most
            )
        });
        if let Some(qt) = quantifier_token {
            sentence.quantification = Some(match qt.form.as_str() {
                "wszyscy" | "każdy" | "wszystko" => Quantifier::Universal,
                "niektórzy" | "niektóre" | "coś" => Quantifier::Existential,
                "nikt" | "nic" => Quantifier::NegatedExistential,
                "wiele" | "wielu" | "dużo" => Quantifier::Proportional("many".to_string()),
                "mało" | "kilka" => Quantifier::Proportional("few".to_string()),
                "większość" => Quantifier::Proportional("most".to_string()),
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
        roles: &[SemanticRole],
        entities: &[Entity],
    ) -> Result<Frame, ParseError> {
        let mut assigned: Vec<Option<Entity>> = vec![None; roles.len()];

        // Pass 1: assign entities with explicit case markings
        // Map case to role, checking which roles are available in the frame
        let mut unassigned: Vec<Entity> = Vec::new();
        for entity in entities {
            if let Some(c) = entity.features.case {
                // Try to map case to available role
                let target_role = match c {
                    Case::Nominative => {
                        // NOM → Agent (primary) or Experiencer (if Agent not available)
                        if roles.contains(&SemanticRole::Agent) {
                            Some(SemanticRole::Agent)
                        } else if roles.contains(&SemanticRole::Experiencer) {
                            Some(SemanticRole::Experiencer)
                        } else {
                            None
                        }
                    }
                    Case::Dative => {
                        // DAT → Recipient
                        if roles.contains(&SemanticRole::Recipient) {
                            Some(SemanticRole::Recipient)
                        } else {
                            None
                        }
                    }
                    Case::Accusative => {
                        // ACC → Theme (primary) or Patient (if Theme not available)
                        if roles.contains(&SemanticRole::Theme) {
                            Some(SemanticRole::Theme)
                        } else if roles.contains(&SemanticRole::Patient) {
                            Some(SemanticRole::Patient)
                        } else {
                            None
                        }
                    }
                    Case::Genitive => {
                        // GEN → Patient (in negative contexts) or Source
                        if roles.contains(&SemanticRole::Patient) && entity.features.case == Some(Case::Genitive) {
                            Some(SemanticRole::Patient)
                        } else if roles.contains(&SemanticRole::Source) {
                            Some(SemanticRole::Source)
                        } else {
                            None
                        }
                    }
                    Case::Instrumental => {
                        // INST → Instrument
                        if roles.contains(&SemanticRole::Instrument) {
                            Some(SemanticRole::Instrument)
                        } else {
                            None
                        }
                    }
                    Case::Locative => {
                        // LOC → Location
                        if roles.contains(&SemanticRole::Location) {
                            Some(SemanticRole::Location)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                
                if let Some(role) = target_role {
                    if let Some(idx) = roles.iter().position(|r| *r == role) {
                        if assigned[idx].is_none() {
                            assigned[idx] = Some(entity.clone());
                            continue;
                        }
                    }
                }
            }
            unassigned.push(entity.clone());
        }

        // Pass 2: assign remaining entities using animacy heuristics
        // Animate entities prefer Agent/Experiencer, inanimate prefer Theme/Patient
        for entity in &unassigned {
            let is_animate = entity.features.animacy == Some(Animacy::Animate);
            
            if is_animate {
                // Animate entity: try Agent first, then Experiencer
                if let Some(idx) = roles.iter().position(|r| *r == SemanticRole::Agent) {
                    if assigned[idx].is_none() {
                        assigned[idx] = Some(entity.clone());
                        continue;
                    }
                }
                if let Some(idx) = roles.iter().position(|r| *r == SemanticRole::Experiencer) {
                    if assigned[idx].is_none() {
                        assigned[idx] = Some(entity.clone());
                        continue;
                    }
                }
            } else {
                // Inanimate entity: try Theme first, then Patient
                if let Some(idx) = roles.iter().position(|r| *r == SemanticRole::Theme) {
                    if assigned[idx].is_none() {
                        assigned[idx] = Some(entity.clone());
                        continue;
                    }
                }
                if let Some(idx) = roles.iter().position(|r| *r == SemanticRole::Patient) {
                    if assigned[idx].is_none() {
                        assigned[idx] = Some(entity.clone());
                        continue;
                    }
                }
            }
            
            // Fallback: try Recipient for animate, any slot for inanimate
            if is_animate {
                if let Some(idx) = roles.iter().position(|r| *r == SemanticRole::Recipient) {
                    if assigned[idx].is_none() {
                        assigned[idx] = Some(entity.clone());
                        continue;
                    }
                }
            }
            
            // Final fallback: any empty slot
            for slot in assigned.iter_mut() {
                if slot.is_none() {
                    *slot = Some(entity.clone());
                    break;
                }
            }
        }

        let get = |role: &SemanticRole| -> Entity {
            if let Some(idx) = roles.iter().position(|r| r == role) {
                if let Some(Some(e)) = assigned.get(idx) {
                    return e.clone();
                }
            }
            Entity::new(ConceptId::new("unknown"))
        };

        match frame_type {
            "Transfer" => Ok(Frame::Transfer {
                agent: get(&SemanticRole::Agent),
                recipient: get(&SemanticRole::Recipient),
                theme: get(&SemanticRole::Theme),
            }),
            "Motion" => Ok(Frame::Motion {
                mover: get(&SemanticRole::Agent),
                source: None,
                goal: None,
                path: None,
            }),
            "Perception" => Ok(Frame::Perception {
                experiencer: get(&SemanticRole::Experiencer),
                stimulus: get(&SemanticRole::Stimulus),
            }),
            "Cognition" => Ok(Frame::Cognition {
                cognizer: get(&SemanticRole::Experiencer),
                content: get(&SemanticRole::Theme),
            }),
            "Emotion" => Ok(Frame::Emotion {
                experiencer: get(&SemanticRole::Experiencer),
                stimulus: get(&SemanticRole::Stimulus),
            }),
            "Destruction" => Ok(Frame::Destruction {
                agent: get(&SemanticRole::Agent),
                patient: get(&SemanticRole::Patient),
                instrument: None,
            }),
            "Consumption" => Ok(Frame::Consumption {
                agent: get(&SemanticRole::Agent),
                patient: get(&SemanticRole::Patient),
            }),
            "Communication" => Ok(Frame::Communication {
                speaker: get(&SemanticRole::Agent),
                addressee: None,
                message: get(&SemanticRole::Theme),
            }),
            "Creation" => Ok(Frame::Creation {
                creator: get(&SemanticRole::Agent),
                created: get(&SemanticRole::Theme),
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
