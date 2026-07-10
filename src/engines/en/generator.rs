use crate::core::interlingua::*;
use crate::data::descriptor::LanguageDescriptor;
use crate::data::lexicon::Lexicon;
use crate::engines::en::morphology::EnglishMorphology;
use crate::error::GenerateError;

pub struct EnglishGenerator {
    lexicon: Lexicon,
    morphology: EnglishMorphology,
    descriptor: LanguageDescriptor,
}

impl EnglishGenerator {
    pub fn new(
        lexicon: Lexicon,
        morphology: EnglishMorphology,
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

        // Handle passive voice
        if sentence.voice == Some(Voice::Passive) {
            words = self.generate_passive_sentence(sentence)?;
        } else {
            for frame in &sentence.frames {
                let frame_words = self.generate_frame(frame, sentence)?;
                words.extend(frame_words);
            }

            // Add quantification
            if let Some(ref quantifier) = sentence.quantification {
                let quant_word = match quantifier {
                    Quantifier::Universal => "all",
                    Quantifier::Existential => "some",
                    Quantifier::NegatedExistential => "none",
                    Quantifier::Numerical(n) => {
                        words.insert(1, n.to_string());
                        if let Some(ref temporal) = sentence.temporal {
                            if let Some(w) = self.generate_temporal(temporal) {
                                words.push(w);
                            }
                        }
                        return self.finalize_sentence(words, sentence.illocution, sentence.polarity);
                    }
                    Quantifier::Proportional(prop) => prop.as_str(),
                };
                words.insert(0, quant_word.to_string());
            }

            if let Some(ref temporal) = sentence.temporal {
                if let Some(w) = self.generate_temporal(temporal) {
                    words.push(w);
                }
            }
        }

        self.finalize_sentence(words, sentence.illocution, sentence.polarity)
    }

    fn generate_passive_sentence(&self, sentence: &Sentence) -> Result<Vec<String>, GenerateError> {
        let mut words = Vec::new();

        for frame in &sentence.frames {
            // Extract theme/patient as the new subject and determine the verb
            let (theme, agent, verb_lemma) = match frame {
                Frame::Transfer { agent, recipient: _, theme } => (theme.clone(), Some(agent.clone()), "give"),
                Frame::Motion { mover, .. } => (mover.clone(), None, "go"),
                Frame::Perception { experiencer, stimulus } => (stimulus.clone(), Some(experiencer.clone()), "see"),
                Frame::Cognition { cognizer, content } => (content.clone(), Some(cognizer.clone()), "think"),
                Frame::Destruction { agent, patient, .. } => (patient.clone(), Some(agent.clone()), "break"),
                Frame::Consumption { agent, patient } => (patient.clone(), Some(agent.clone()), "eat"),
                Frame::Emotion { experiencer, stimulus } => (stimulus.clone(), Some(experiencer.clone()), "love"),
                Frame::Communication { speaker, addressee: _, message } => (message.clone(), Some(speaker.clone()), "say"),
                Frame::Creation { creator, created, .. } => (created.clone(), Some(creator.clone()), "make"),
                Frame::Statement { subject, property: _ } => (subject.clone(), None, "be"),
                Frame::Existence { entity, .. } => (entity.clone(), None, "exist"),
                Frame::Possession { possessor, possessed } => (possessed.clone(), Some(possessor.clone()), "have"),
                Frame::Custom { .. } => return Ok(words),
            };

            // Generate theme as subject
            let theme_str = self.generate_entity_form(&theme, true)?;
            words.push(theme_str);

            // Add "be" auxiliary (past tense for now)
            let be_form = if theme.features.number == Some(Number::Plural) {
                "were"
            } else {
                "was"
            };
            words.push(be_form.to_string());

            // Add past participle
            let participle = self.find_past_participle(verb_lemma);
            words.push(participle);

            // Add "by" phrase for agent
            if let Some(agent_entity) = agent {
                words.push("by".to_string());
                let agent_str = self.generate_entity_form(&agent_entity, true)?;
                words.push(agent_str);
            }
        }

        if let Some(ref temporal) = sentence.temporal {
            if let Some(w) = self.generate_temporal(temporal) {
                words.push(w);
            }
        }

        Ok(words)
    }

    fn find_past_participle(&self, lemma: &str) -> String {
        // Irregular past participles
        match lemma {
            "give" => "given".to_string(),
            "eat" => "eaten".to_string(),
            "see" => "seen".to_string(),
            "drink" => "drunk".to_string(),
            "make" => "made".to_string(),
            "take" => "taken".to_string(),
            "buy" => "bought".to_string(),
            "break" => "broken".to_string(),
            "love" => "loved".to_string(),
            "think" => "thought".to_string(),
            "know" => "known".to_string(),
            "hear" => "heard".to_string(),
            "read" => "read".to_string(),
            "write" => "written".to_string(),
            "have" => "had".to_string(),
            "be" => "been".to_string(),
            _ => format!("{}ed", lemma),
        }
    }

    fn finalize_sentence(&self, mut words: Vec<String>, illocution: Illocution, polarity: Polarity) -> Result<String, GenerateError> {
        // Handle do-support for questions and negation
        let is_question = illocution == Illocution::Question;
        let is_negative = polarity == Polarity::Negative;

        if is_question && is_negative {
            // "Did X not Y?"
            words.insert(0, "Did".to_string());
            if words.len() > 2 {
                words.insert(2, "not".to_string());
            }
        } else if is_question {
            // "Did X Y?"
            words.insert(0, "Did".to_string());
        } else if is_negative {
            // "X did not Y"
            if words.len() > 1 {
                words.insert(1, "did".to_string());
                words.insert(2, "not".to_string());
            } else {
                words.insert(0, "did".to_string());
                words.insert(1, "not".to_string());
            }
        }

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
            Frame::Custom { .. } => Ok(Vec::new()),
        }
    }

    fn generate_transfer(
        &self,
        agent: &Entity,
        recipient: &Entity,
        theme: &Entity,
        frame: &Frame,
        sentence: &Sentence,
    ) -> Result<Vec<String>, GenerateError> {
        let verb_lemma = self.find_verb_for_frame(frame)?;

        let agent_form = self.generate_entity_form(agent, false)?;
        // Use base form for questions and negations (do-support)
        let verb_form = if sentence.polarity == Polarity::Negative || sentence.illocution == Illocution::Question {
            verb_lemma.to_string()
        } else {
            self.morphology.inflect_verb(
                &verb_lemma,
                sentence.tense.unwrap_or(Tense::Present),
                Some(Person::Third),
                Some(Number::Singular),
            )?
        };
        let theme_form = self.generate_entity_form(theme, true)?;
        let recipient_form = self.generate_entity_form(recipient, false)?;

        let mut words = vec![agent_form, verb_form];
        words.push(theme_form);
        words.push("to".to_string());
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
        let verb_lemma = "go";
        let mover_form = self.generate_entity_form(mover, false)?;
        let verb_form = self.morphology.inflect_verb(
            verb_lemma,
            sentence.tense.unwrap_or(Tense::Present),
            Some(Person::Third),
            Some(Number::Singular),
        )?;

        let mut words = vec![mover_form, verb_form];

        if let Some(ref g) = goal {
            let goal_form = self.generate_entity_form(g, false)?;
            words.push("to".to_string());
            words.push(goal_form);
        }

        if let Some(ref s) = source {
            let source_form = self.generate_entity_form(s, false)?;
            words.push("from".to_string());
            words.push(source_form);
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

        let subject_form = self.generate_entity_form(subject, false)?;
        // Use base form for questions and negations (do-support)
        let verb_form = if sentence.polarity == Polarity::Negative || sentence.illocution == Illocution::Question {
            verb_lemma.to_string()
        } else {
            self.morphology.inflect_verb(
                &verb_lemma,
                sentence.tense.unwrap_or(Tense::Present),
                Some(Person::Third),
                Some(Number::Singular),
            )?
        };
        let object_form = self.generate_entity_form(object, true)?;

        let mut words = vec![subject_form, verb_form];
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
        let verb_lemma = "say";
        let speaker_form = self.generate_entity_form(speaker, false)?;
        let verb_form = self.morphology.inflect_verb(
            verb_lemma,
            sentence.tense.unwrap_or(Tense::Present),
            Some(Person::Third),
            Some(Number::Singular),
        )?;
        let message_form = self.generate_entity_form(message, true)?;

        let mut words = vec![speaker_form, verb_form];

        if let Some(addr) = addressee {
            let addr_form = self.generate_entity_form(addr, false)?;
            words.push("to".to_string());
            words.push(addr_form);
        }

        words.push(message_form);

        Ok(words)
    }

    fn generate_statement(
        &self,
        subject: &Entity,
        property: &Entity,
        sentence: &Sentence,
    ) -> Result<Vec<String>, GenerateError> {
        let subject_form = self.generate_entity_form(subject, false)?;
        let property_form = self.generate_entity_form(property, false)?;

        let verb = match sentence.tense {
            Some(Tense::Past) => "was",
            _ => {
                if subject.features.number == Some(Number::Plural) {
                    "are"
                } else {
                    "is"
                }
            }
        };

        Ok(vec![subject_form, verb.to_string(), property_form])
    }

    fn generate_existence(
        &self,
        entity: &Entity,
        location: Option<&Entity>,
        sentence: &Sentence,
    ) -> Result<Vec<String>, GenerateError> {
        let entity_form = self.generate_entity_form(entity, true)?;

        let verb = match sentence.tense {
            Some(Tense::Past) => {
                if entity.features.number == Some(Number::Plural) {
                    "were"
                } else {
                    "was"
                }
            }
            _ => {
                if entity.features.number == Some(Number::Plural) {
                    "are"
                } else {
                    "is"
                }
            }
        };

        let mut words = vec![verb.to_string(), entity_form];

        if let Some(loc) = location {
            let loc_form = self.generate_entity_form(loc, false)?;
            words.push("in".to_string());
            words.push(loc_form);
        }

        Ok(words)
    }

    fn generate_entity_form(
        &self,
        entity: &Entity,
        needs_article: bool,
    ) -> Result<String, GenerateError> {
        if let Some(ref name) = entity.name {
            // Check if this is a proper noun (starts with uppercase)
            if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                return Ok(name.clone());
            }
            
            let by_name = self.lexicon.lookup_by_form(&name.to_lowercase())
                .or_else(|| self.lexicon.lookup_by_lemma(name));
            if let Some(e) = by_name {
                let number = entity.features.number.unwrap_or(Number::Singular);
                let noun_form = self.morphology.inflect_noun(&e.lemma, number)?;
                if needs_article && number == Number::Singular {
                    let is_definite = entity.features.definiteness == Some(Definiteness::Definite);
                    if is_definite {
                        return Ok(format!("the {}", noun_form));
                    }
                    let is_countable = e.features.countability != Some(Countability::Mass);
                    if is_countable {
                        let article = if self.starts_with_vowel_sound(&noun_form) { "an" } else { "a" };
                        return Ok(format!("{} {}", article, noun_form));
                    }
                }
                return Ok(noun_form);
            }
        }

        let lemma = if let Some(entry) = self.lexicon.lookup_concept(&entity.concept.0) {
            entry.lemma.clone()
        } else {
            entity.concept.0.to_lowercase()
        };

        let number = entity.features.number.unwrap_or(Number::Singular);
        let noun_form = self.morphology.inflect_noun(&lemma, number)?;

        if needs_article && number == Number::Singular {
            let is_definite = entity.features.definiteness == Some(Definiteness::Definite);
            if is_definite {
                return Ok(format!("the {}", noun_form));
            }
            let entry = self.lexicon.lookup_by_form(&lemma)
                .or_else(|| self.lexicon.lookup_by_lemma(&lemma));
            let is_countable = entry
                .map(|e| e.features.countability != Some(Countability::Mass))
                .unwrap_or(true);
            if is_countable {
                let article = if self.starts_with_vowel_sound(&noun_form) { "an" } else { "a" };
                return Ok(format!("{} {}", article, noun_form));
            }
        }

        Ok(noun_form)
    }

    fn find_verb_for_frame(&self, frame: &Frame) -> Result<String, GenerateError> {
        let concept_name = match frame {
            Frame::Transfer { .. } => "GIVE",
            Frame::Motion { .. } => "GO",
            Frame::Perception { .. } => "SEE",
            Frame::Cognition { .. } => "THINK",
            Frame::Emotion { .. } => "LOVE",
            Frame::Destruction { .. } => "BREAK",
            Frame::Consumption { patient, .. } => {
                // Check if patient is a liquid (mass noun with liquid concept)
                let is_liquid = patient.features.countability == Some(Countability::Mass)
                    && matches!(
                        patient.concept.0.as_str(),
                        "WATER" | "MILK" | "JUICE" | "COFFEE" | "TEA" | "BEER" | "WINE"
                    );
                if is_liquid {
                    "DRINK"
                } else {
                    "EAT"
                }
            }
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
            language: "en".to_string(),
        })
    }

    fn generate_temporal(&self, temporal: &TemporalReference) -> Option<String> {
        match temporal {
            TemporalReference::Deictic { word } => {
                let en = match word.as_str() {
                    "wczoraj" => "yesterday",
                    "dzisiaj" | "teraz" => "today",
                    "jutro" => "tomorrow",
                    _ => word.as_str(),
                };
                Some(en.to_string())
            }
            TemporalReference::Relative { offset_days, .. } => match *offset_days {
                -1 => Some("yesterday".to_string()),
                0 => Some("today".to_string()),
                1 => Some("tomorrow".to_string()),
                _ => None,
            },
            _ => None,
        }
    }

    fn starts_with_vowel_sound(&self, word: &str) -> bool {
        matches!(
            word.chars().next().map(|c| c.to_ascii_lowercase()),
            Some('a') | Some('e') | Some('i') | Some('o') | Some('u')
        )
    }
}
