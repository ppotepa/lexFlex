use crate::core::interlingua::*;
use crate::data::descriptor::LanguageDescriptor;
use crate::data::lexicon::Lexicon;
use crate::engines::pl::morphology::PolishMorphology;
use crate::error::GenerateError;

pub struct PolishGenerator {
    lexicon: Lexicon,
    morphology: PolishMorphology,
    descriptor: LanguageDescriptor,
}

impl PolishGenerator {
    pub fn new(
        lexicon: Lexicon,
        morphology: PolishMorphology,
        descriptor: LanguageDescriptor,
    ) -> Self {
        Self {
            lexicon,
            morphology,
            descriptor,
        }
    }

    pub fn generate(&self, utterance: &Utterance) -> Result<String, GenerateError> {
        let mut parts = Vec::new();

        for sentence in &utterance.sentences {
            let text = self.generate_sentence(sentence)?;
            parts.push(text);
        }

        Ok(parts.join(". "))
    }

    fn generate_sentence(&self, sentence: &Sentence) -> Result<String, GenerateError> {
        let mut words = Vec::new();

        if sentence.illocution == Illocution::Question {
            words.push("Czy".to_string());
        }

        // Handle passive voice
        if sentence.voice == Some(Voice::Passive) {
            words.extend(self.generate_passive_sentence(sentence)?);
        } else {
            for frame in &sentence.frames {
                let frame_words = self.generate_frame(frame, sentence)?;
                words.extend(frame_words);
            }
        }

        // Add quantification
        if let Some(ref quantifier) = sentence.quantification {
            let quant_word = match quantifier {
                Quantifier::Universal => "wszyscy",
                Quantifier::Existential => "niektórzy",
                Quantifier::NegatedExistential => "nikt",
                Quantifier::Numerical(n) => {
                    // For numerical quantifiers, we'd need to convert to words
                    // For now, use the number as a string
                    words.insert(1, n.to_string());
                    if let Some(ref temporal) = sentence.temporal {
                        let temporal_word = self.generate_temporal(temporal);
                        if let Some(w) = temporal_word {
                            words.push(w);
                        }
                    }
                    return self.finalize_sentence(words, sentence.illocution);
                }
                Quantifier::Proportional(prop) => {
                    match prop.as_str() {
                        "many" => "wielu",
                        "few" => "kilku",
                        "most" => "większość",
                        _ => "niektórzy",
                    }
                }
            };
            words.insert(0, quant_word.to_string());
        }

        if let Some(ref temporal) = sentence.temporal {
            let temporal_word = self.generate_temporal(temporal);
            if let Some(w) = temporal_word {
                words.push(w);
            }
        }

        self.finalize_sentence(words, sentence.illocution)
    }

    fn finalize_sentence(&self, words: Vec<String>, illocution: Illocution) -> Result<String, GenerateError> {
        let mut result = words.join(" ");

        match illocution {
            Illocution::Question => result.push('?'),
            Illocution::Exclamation => result.push('!'),
            _ => result.push('.'),
        }

        if let Some(first) = result.get_mut(0..1) {
            let upper = first.to_uppercase();
            result.replace_range(0..1, &upper);
        }

        Ok(result)
    }

    fn generate_frame(
        &self,
        frame: &Frame,
        sentence: &Sentence,
    ) -> Result<Vec<String>, GenerateError> {
        match frame {
            Frame::Transfer { agent, recipient, theme } => {
                self.generate_transfer(agent, recipient, theme, frame, sentence)
            }
            Frame::Motion { mover, source, goal, .. } => {
                self.generate_motion(mover, source, goal, sentence)
            }
            Frame::Perception { experiencer, stimulus } => {
                self.generate_two_role(frame, experiencer, stimulus, sentence)
            }
            Frame::Cognition { cognizer, content } => {
                self.generate_two_role(frame, cognizer, content, sentence)
            }
            Frame::Emotion { experiencer, stimulus } => {
                self.generate_two_role(frame, experiencer, stimulus, sentence)
            }
            Frame::Destruction { agent, patient, .. } => {
                self.generate_two_role(frame, agent, patient, sentence)
            }
            Frame::Consumption { agent, patient } => {
                self.generate_two_role(frame, agent, patient, sentence)
            }
            Frame::Communication { speaker, addressee, message } => {
                self.generate_communication(speaker, addressee.as_ref(), message, sentence)
            }
            Frame::Creation { creator, created, .. } => {
                self.generate_two_role(frame, creator, created, sentence)
            }
            Frame::Statement { subject, property } => {
                self.generate_statement(subject, property, sentence)
            }
            Frame::Existence { entity, location } => {
                self.generate_existence(entity, location.as_ref(), sentence)
            }
            Frame::Possession { possessor, possessed } => {
                self.generate_two_role(frame, possessor, possessed, sentence)
            }
            Frame::Custom { name: _, roles } => {
                let words: Vec<String> = roles.iter()
                    .map(|(_, e)| self.generate_entity_form(e, e.features.case))
                    .collect::<Result<_, _>>()?;
                Ok(words)
            }
        }
    }

    fn generate_passive_sentence(&self, sentence: &Sentence) -> Result<Vec<String>, GenerateError> {
        let mut words = Vec::new();

        for frame in &sentence.frames {
            // Extract theme/patient as the new subject and determine the verb
            let (theme, agent, verb_lemma) = match frame {
                Frame::Transfer { agent, recipient: _, theme } => (theme.clone(), Some(agent.clone()), "dać"),
                Frame::Motion { mover, .. } => (mover.clone(), None, "iść"),
                Frame::Perception { experiencer, stimulus } => (stimulus.clone(), Some(experiencer.clone()), "widzieć"),
                Frame::Cognition { cognizer, content } => (content.clone(), Some(cognizer.clone()), "myśleć"),
                Frame::Destruction { agent, patient, .. } => (patient.clone(), Some(agent.clone()), "zniszczyć"),
                Frame::Consumption { agent, patient } => (patient.clone(), Some(agent.clone()), "jeść"),
                Frame::Emotion { experiencer, stimulus } => (stimulus.clone(), Some(experiencer.clone()), "kochać"),
                Frame::Communication { speaker, addressee: _, message } => (message.clone(), Some(speaker.clone()), "mówić"),
                Frame::Creation { creator, created, .. } => (created.clone(), Some(creator.clone()), "zrobić"),
                Frame::Statement { subject, property: _ } => (subject.clone(), None, "być"),
                Frame::Existence { entity, .. } => (entity.clone(), None, "istnieć"),
                Frame::Possession { possessor, possessed } => (possessed.clone(), Some(possessor.clone()), "mieć"),
                Frame::Custom { .. } => return Ok(words),
            };

            // Generate theme as subject (NOM case)
            let theme_str = self.generate_entity_form(&theme, Some(Case::Nominative))?;
            words.push(theme_str);

            // Add passive auxiliary (został for past perfective, był for past imperfective)
            let aux = if sentence.aspect == Some(Aspect::Perfective) {
                match theme.features.gender {
                    Some(Gender::Masculine) => "został",
                    Some(Gender::Feminine) => "została",
                    Some(Gender::Neuter) => "zostało",
                    _ => "został",
                }
            } else {
                match theme.features.gender {
                    Some(Gender::Masculine) => "był",
                    Some(Gender::Feminine) => "była",
                    Some(Gender::Neuter) => "było",
                    _ => "był",
                }
            };
            words.push(aux.to_string());

            // Add passive participle
            let participle = self.find_passive_participle_pl(verb_lemma, theme.features.gender);
            words.push(participle);

            // Add "przez" phrase for agent
            if let Some(agent_entity) = agent {
                words.push("przez".to_string());
                let agent_str = self.generate_entity_form(&agent_entity, Some(Case::Accusative))?;
                words.push(agent_str);
            }
        }

        Ok(words)
    }

    fn find_passive_participle_pl(&self, lemma: &str, gender: Option<Gender>) -> String {
        // Common passive participles in Polish
        let base = match lemma {
            "dać" => "dan",
            "widzieć" => "widzian",
            "jeść" => "jedzon",
            "pić" => "pit",
            "czytać" => "czytan",
            "kochać" => "kochan",
            "kupić" => "kupion",
            "zrobić" => "zrobion",
            "mówić" => "mówion",
            "myśleć" => "myślan",
            _ => return format!("{}ny", lemma),
        };

        // Add gender ending
        match gender {
            Some(Gender::Masculine) => base.to_string(),
            Some(Gender::Feminine) => format!("{}a", base),
            Some(Gender::Neuter) => format!("{}e", base),
            _ => base.to_string(),
        }
    }

    fn generate_transfer(
        &self,
        agent: &Entity,
        recipient: &Entity,
        theme: &Entity,
        frame: &Frame,
        _sentence: &Sentence,
    ) -> Result<Vec<String>, GenerateError> {
        let verb_lemma = self.find_verb_for_frame(frame)?;

        let agent_form = self.generate_entity_form(agent, Some(Case::Nominative))?;
        let verb_form = self.generate_verb_form(
            &verb_lemma,
            _sentence.tense,
            _sentence.aspect,
            Some(Person::Third),
            Some(Number::Singular),
            agent.features.gender,
        )?;
        let theme_case = if _sentence.polarity == Polarity::Negative {
            Some(Case::Genitive)
        } else {
            Some(Case::Accusative)
        };
        let theme_form = self.generate_entity_form(theme, theme_case)?;
        let recipient_form = self.generate_entity_form(recipient, Some(Case::Dative))?;

        let mut words = vec![agent_form, verb_form];

        if _sentence.polarity == Polarity::Negative {
            words.insert(2, "nie".to_string());
        }

        words.push(theme_form);
        words.push(recipient_form);

        Ok(words)
    }

    fn generate_motion(
        &self,
        mover: &Entity,
        source: &Option<Entity>,
        goal: &Option<Entity>,
        sentence: &Sentence,
    ) -> Result<Vec<String>, GenerateError> {
        let verb_lemma = "iść";
        let mover_form = self.generate_entity_form(mover, Some(Case::Nominative))?;
        let verb_form = self.generate_verb_form(
            verb_lemma,
            sentence.tense,
            sentence.aspect,
            Some(Person::Third),
            Some(Number::Singular),
            mover.features.gender,
        )?;

        let mut words = vec![mover_form, verb_form];

        if let Some(ref g) = goal {
            words.push(format!("do {}", self.generate_entity_form(g, Some(Case::Genitive))?));
        }

        if let Some(ref s) = source {
            words.push(format!("z {}", self.generate_entity_form(s, Some(Case::Genitive))?));
        }

        Ok(words)
    }

    fn generate_two_role(
        &self,
        frame: &Frame,
        subject: &Entity,
        object: &Entity,
        sentence: &Sentence,
    ) -> Result<Vec<String>, GenerateError> {
        let verb_lemma = self.find_verb_for_frame(frame)?;

        let subject_form = self.generate_entity_form(subject, Some(Case::Nominative))?;
        let verb_form = self.generate_verb_form(
            &verb_lemma,
            sentence.tense,
            sentence.aspect,
            Some(Person::Third),
            Some(Number::Singular),
            subject.features.gender,
        )?;
        let object_case = if sentence.polarity == Polarity::Negative {
            Some(Case::Genitive)
        } else {
            Some(Case::Accusative)
        };
        let object_form = self.generate_entity_form(object, object_case)?;

        let mut words = vec![subject_form, verb_form];

        if sentence.polarity == Polarity::Negative {
            words.push("nie".to_string());
        }

        words.push(object_form);

        Ok(words)
    }

    fn generate_communication(
        &self,
        speaker: &Entity,
        addressee: Option<&Entity>,
        message: &Entity,
        sentence: &Sentence,
    ) -> Result<Vec<String>, GenerateError> {
        let verb_lemma = "mówić";
        let speaker_form = self.generate_entity_form(speaker, Some(Case::Nominative))?;
        let verb_form = self.generate_verb_form(
            verb_lemma,
            sentence.tense,
            sentence.aspect,
            Some(Person::Third),
            Some(Number::Singular),
            speaker.features.gender,
        )?;
        let message_form = self.generate_entity_form(message, Some(Case::Accusative))?;

        let mut words = vec![speaker_form, verb_form];

        if let Some(addr) = addressee {
            let addr_form = self.generate_entity_form(addr, Some(Case::Dative))?;
            words.push(format!("do {}", addr_form));
        }

        words.push(message_form);

        Ok(words)
    }

    fn generate_statement(
        &self,
        subject: &Entity,
        property: &Entity,
        _sentence: &Sentence,
    ) -> Result<Vec<String>, GenerateError> {
        let subject_form = self.generate_entity_form(subject, Some(Case::Nominative))?;
        let property_form = self.generate_entity_form(property, Some(Case::Nominative))?;

        let verb_form = "to";

        Ok(vec![subject_form, verb_form.to_string(), property_form])
    }

    fn generate_existence(
        &self,
        entity: &Entity,
        location: Option<&Entity>,
        sentence: &Sentence,
    ) -> Result<Vec<String>, GenerateError> {
        let entity_form = self.generate_entity_form(entity, Some(Case::Nominative))?;
        let verb_form = match sentence.tense {
            Some(Tense::Past) => match entity.features.gender {
                Some(Gender::Masculine) | Some(Gender::MasculinePersonal)
                | Some(Gender::MasculineAnimate) | Some(Gender::MasculineInanimate) => "był",
                Some(Gender::Feminine) => "była",
                Some(Gender::Neuter) => "było",
                _ => "był",
            },
            _ => match entity.features.number {
                Some(Number::Plural) => "są",
                _ => "jest",
            },
        };

        let mut words = vec![verb_form.to_string(), entity_form];

        if let Some(loc) = location {
            words.push(format!("w {}", self.generate_entity_form(loc, Some(Case::Locative))?));
        }

        Ok(words)
    }

    fn generate_entity_form(
        &self,
        entity: &Entity,
        case: Option<Case>,
    ) -> Result<String, GenerateError> {
        // First try to find by name in this lexicon
        if let Some(ref name) = entity.name {
            let lex_entry = self.lexicon.lookup_by_form(&name.to_lowercase())
                .or_else(|| self.lexicon.lookup_by_lemma(name));

            if let Some(entry) = lex_entry {
                if entry.pos == "Noun" {
                    let target_case = case.unwrap_or(Case::Nominative);
                    let number = entity.features.number.unwrap_or(Number::Singular);
                    let gender = entity.features.gender.or(entry.features.gender);

                    if target_case == Case::Nominative && number == Number::Singular {
                        return Ok(self.capitalize_first(&entry.lemma));
                    }

                    return self.morphology.inflect_noun(
                        &entry.lemma,
                        target_case,
                        number,
                        gender,
                        entry.paradigm.as_deref(),
                    );
                }
            }

            // Proper name not in target lexicon — use as-is
            if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                return Ok(name.clone());
            }
        }

        // Fall back to concept lookup
        let concept_entry = self.lexicon.lookup_concept(&entity.concept.0);
        if let Some(entry) = concept_entry {
            let target_case = case.unwrap_or(Case::Nominative);
            let number = entity.features.number.unwrap_or(Number::Singular);
            let gender = entity.features.gender.or(entry.features.gender);

            if target_case == Case::Nominative && number == Number::Singular {
                return Ok(self.capitalize_first(&entry.lemma));
            }

            return self.morphology.inflect_noun(
                &entry.lemma,
                target_case,
                number,
                gender,
                entry.paradigm.as_deref(),
            );
        }

        let concept_str = entity.concept.0.to_lowercase();
        Ok(self.capitalize_first(&concept_str))
    }

    fn generate_verb_form(
        &self,
        lemma: &str,
        tense: Option<Tense>,
        _aspect: Option<Aspect>,
        person: Option<Person>,
        number: Option<Number>,
        gender: Option<Gender>,
    ) -> Result<String, GenerateError> {
        let t = tense.unwrap_or(Tense::Present);

        let entry = self.lexicon.lookup_by_lemma(lemma)
            .or_else(|| self.lexicon.lookup_by_form(lemma));
        let paradigm = entry.and_then(|e| e.paradigm.clone());

        match self.morphology.inflect_verb(
            lemma,
            t,
            person,
            number,
            gender,
            paradigm.as_deref(),
        ) {
            Ok(form) => Ok(form),
            Err(_) => {
                // Fallback to lemma if inflection fails
                Ok(lemma.to_string())
            }
        }
    }

    fn find_verb_for_frame(&self, frame: &Frame) -> Result<String, GenerateError> {
        let concept_name = match frame {
            Frame::Transfer { .. } => "GIVE",
            Frame::Motion { .. } => "GO",
            Frame::Perception { stimulus, .. } => {
                // Check stimulus concept to determine if it's reading or seeing
                match stimulus.concept.0.as_str() {
                    "READ" | "BOOK" | "NEWSPAPER" | "MAGAZINE" => "READ",
                    _ => "SEE",
                }
            }
            Frame::Cognition { .. } => "THINK",
            Frame::Emotion { .. } => "LOVE",
            Frame::Destruction { .. } => "BREAK",
            Frame::Consumption { .. } => "EAT",
            Frame::Communication { .. } => "SAY",
            Frame::Creation { .. } => "MAKE",
            Frame::Statement { .. } => "BE",
            Frame::Existence { .. } => "BE",
            Frame::Possession { .. } => "HAVE",
            Frame::Custom { name, .. } => name,
        };

        if let Some(entry) = self.lexicon.lookup_concept(concept_name) {
            return Ok(entry.lemma.clone());
        }

        Err(GenerateError::NoLexemeForConcept {
            concept: concept_name.to_string(),
            language: "pl".to_string(),
        })
    }

    fn generate_temporal(&self, temporal: &TemporalReference) -> Option<String> {
        match temporal {
            TemporalReference::Deictic { word } => Some(word.clone()),
            TemporalReference::Relative { offset_days, .. } => match *offset_days {
                -1 => Some("wczoraj".to_string()),
                0 => Some("dzisiaj".to_string()),
                1 => Some("jutro".to_string()),
                _ => None,
            },
            _ => None,
        }
    }

    fn capitalize_first(&self, s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().to_string() + chars.as_str(),
        }
    }
}
