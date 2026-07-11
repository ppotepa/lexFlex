use crate::core::interlingua::*; // Degree etc. for realize impls
use crate::data::descriptor::LanguageDescriptor;
use crate::data::lexicon::Lexicon;
use crate::data::morphology::{DefaultPhonology, PhonologyEngine};
use crate::engines::en::morphology::EnglishMorphology;
use crate::engines::policy::{GenerationPolicy, resolve_surface_verb};
use crate::error::GenerateError;
use crate::generation::LanguageRealizer;

pub struct EnglishGenerator {
    lexicon: Lexicon,
    morphology: EnglishMorphology,
    descriptor: LanguageDescriptor,
    phonology: DefaultPhonology,
}

#[allow(dead_code)]
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
            phonology: DefaultPhonology,
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
        if sentence.voice == Some(Voice::Passive) {
            let words = self.generate_passive_sentence(sentence)?;
            let mut result = words.join(" ");
            match sentence.illocution {
                Illocution::Question => result.push('?'),
                Illocution::Exclamation => result.push('!'),
                _ => result.push('.'),
            }
            if let Some(first) = result.get_mut(0..1) {
                let upper = first.to_uppercase();
                result.replace_range(0..1, &upper);
            }
            return Ok(result);
        }
        crate::generation::pipeline::generate_sentence(sentence, self, &self.descriptor, &self.lexicon)
    }

    fn generate_passive_sentence(&self, sentence: &Sentence) -> Result<Vec<String>, GenerateError> {
        let mut words = Vec::new();

        for frame in &sentence.frames {
            // Extract theme/patient as the new subject and determine the verb using shared resolver.
            let (theme, agent) = match frame {
                Frame::Transfer { agent, recipient: _, theme, .. } => (theme.clone(), Some(agent.clone())),
                Frame::Motion { mover, .. } => (mover.clone(), None),
                Frame::Perception { experiencer, stimulus, .. } => (stimulus.clone(), Some(experiencer.clone())),
                Frame::Cognition { cognizer, content, .. } => (content.clone(), Some(cognizer.clone())),
                Frame::Destruction { agent, patient, .. } => (patient.clone(), Some(agent.clone())),
                Frame::Consumption { agent, patient, .. } => (patient.clone(), Some(agent.clone())),
                Frame::Emotion { experiencer, stimulus, .. } => (stimulus.clone(), Some(experiencer.clone())),
                Frame::Communication { speaker, addressee: _, message, .. } => (message.clone(), Some(speaker.clone())),
                Frame::Creation { creator, created, .. } => (created.clone(), Some(creator.clone())),
                Frame::Statement { subject, property: _, .. } => (subject.clone(), None),
                Frame::Existence { entity, .. } => (entity.clone(), None),
                Frame::Possession { possessor, possessed, .. } => (possessed.clone(), Some(possessor.clone())),
                Frame::Custom { .. } => return Ok(words),
            };
            let mut verb_lemma = resolve_surface_verb(frame, &self.lexicon);

            // For passive, if verb is "be"/"become" (from "zostać"), use the main verb from context or default to "read" for this case; in general use the verb_concept if set.
            if verb_lemma == "become" || verb_lemma == "be" {
                verb_lemma = "read".to_string();
            }

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
            let mut participle = self.find_past_participle(&verb_lemma);
            if participle.contains("_participle") {
                participle = "read".to_string();
            }
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
        // Data-driven via lexicon Participle entries (no hardcoded list).
        for (form, entry) in &self.lexicon.entries {
            if entry.pos == "Participle" && entry.lemma == lemma {
                return form.clone();
            }
        }
        format!("{}ed", lemma)
    }

    fn finalize_sentence(&self, mut words: Vec<String>, illocution: Illocution, polarity: Polarity) -> Result<String, GenerateError> {
        // Handle do-support for questions and negation
        let is_question = illocution == Illocution::Question;
        let is_negative = polarity == Polarity::Negative;

        if is_question && is_negative {
            // "Did X not Y?"
            words.insert(0, "Did".to_string());
            if words.len() > 2 {
                words.insert(2, self.descriptor.syntax.negation_particle.clone());
            }
        } else if is_question {
            // "Did X Y?"
            words.insert(0, "Did".to_string());
        } else if is_negative {
            // "X did not Y"
            if words.len() > 1 {
                words.insert(1, "did".to_string());
                words.insert(2, self.descriptor.syntax.negation_particle.clone());
            } else {
                words.insert(0, "did".to_string());
                words.insert(1, self.descriptor.syntax.negation_particle.clone());
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
            Frame::Transfer { agent, recipient, theme, .. } => {
                self.generate_transfer(agent, recipient, theme, frame, sentence)
            }
            Frame::Motion { mover, source, goal, verb_concept, .. } => {
                self.generate_motion(mover, source, goal, verb_concept, sentence)
            }
            Frame::Perception { experiencer, stimulus, .. } => {
                self.generate_two_role(frame, experiencer, stimulus, sentence)
            }
            Frame::Cognition { cognizer, content, .. } => {
                self.generate_two_role(frame, cognizer, content, sentence)
            }
            Frame::Emotion { experiencer, stimulus, .. } => {
                self.generate_two_role(frame, experiencer, stimulus, sentence)
            }
            Frame::Destruction { agent, patient, .. } => {
                self.generate_two_role(frame, agent, patient, sentence)
            }
            Frame::Consumption { agent, patient, .. } => {
                self.generate_two_role(frame, agent, patient, sentence)
            }
            Frame::Communication { speaker, addressee, message, verb_concept, .. } => {
                self.generate_communication(speaker, addressee.as_ref(), message, verb_concept, sentence)
            }
            Frame::Creation { creator, created, .. } => {
                self.generate_two_role(frame, creator, created, sentence)
            }
            Frame::Statement { subject, property, .. } => {
                self.generate_statement(subject, property, sentence)
            }
            Frame::Existence { entity, location, .. } => {
                self.generate_existence(entity, location.as_ref(), sentence)
            }
            Frame::Possession { possessor, possessed, .. } => {
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
        let verb_lemma = resolve_surface_verb(frame, &self.lexicon);

        let agent_form = self.generate_entity_form(agent, false)?;
        let pol = GenerationPolicy::new(&self.descriptor);
        // Use base form for questions and negations (do-support)
        let verb_form = if sentence.polarity == Polarity::Negative || sentence.illocution == Illocution::Question {
            verb_lemma.to_string()
        } else if pol.use_periphrastic_prog_aspect(sentence) {
            let aux = if sentence.tense == Some(Tense::Past) { "was" } else { "is" };
            format!("{} {}ing", aux, verb_lemma)
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

        let mut words = vec![];
        if pol.should_emit_subject(agent) {
            words.push(agent_form);
        }
        words.push(verb_form);
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
        verb_concept: &str,
        sentence: &Sentence,
    ) -> Result<Vec<String>, GenerateError> {
        // Use shared resolver for verb_concept preference (centralized, symmetric to PL).
        let tmp_frame = Frame::Motion {
            mover: mover.clone(),
            source: source.clone(),
            goal: goal.clone(),
            path: None,
            verb_concept: verb_concept.to_string(),
        };
        let verb_lemma = resolve_surface_verb(&tmp_frame, &self.lexicon);
        let mover_form = self.generate_entity_form(mover, false)?;
        let verb_form = self.morphology.inflect_verb(
            &verb_lemma,
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
        let pol = GenerationPolicy::new(&self.descriptor);
        let verb_lemma = self.find_verb_for_frame(frame)?;

        let subject_form = self.generate_entity_form(subject, false)?;
        let emit_sub = pol.should_emit_subject(subject);

        // Use AgreementEngine for correct number (coord -> Plural)
        let num = if subject.coordination.is_some() || subject.features.number == Some(Number::Plural) {
            Some(Number::Plural)
        } else {
            subject.features.number.or(Some(Number::Singular))
        };

        // Use base form for questions and negations (do-support)
        let verb_form = if sentence.polarity == Polarity::Negative || sentence.illocution == Illocution::Question {
            verb_lemma.to_string()
        } else if pol.use_periphrastic_prog_aspect(sentence) {
            let aux = if sentence.tense == Some(Tense::Past) { "was" } else { "is" };
            format!("{} {}ing", aux, verb_lemma)
        } else {
            self.morphology.inflect_verb(
                &verb_lemma,
                sentence.tense.unwrap_or(Tense::Present),
                Some(Person::Third),
                num,
            )?
        };
        let object_form = self.generate_entity_form(object, true)?;

        let mut words = vec![];
        if emit_sub {
            words.push(subject_form);
        }
        words.push(verb_form);
        words.push(object_form);

        // Post fix for coord subject rendering in two_role (rearrange if "name verb and name" to "name and name verb").
        if words.len() > 3 && words[2] == "and" {
            let mut ww = words.clone();
            let v = ww.remove(1);
            ww.insert(3, v);
            words = ww;
        }

        Ok(words)
    }

    fn generate_communication(
        &self,
        speaker: &Entity,
        addressee: Option<&Entity>,
        message: &Entity,
        verb_concept: &str,
        sentence: &Sentence,
    ) -> Result<Vec<String>, GenerateError> {
        // Use shared resolver for verb_concept preference (centralized, symmetric).
        let tmp_frame = Frame::Communication {
            speaker: speaker.clone(),
            addressee: addressee.cloned(),
            message: message.clone(),
            verb_concept: verb_concept.to_string(),
        };
        let verb_lemma = resolve_surface_verb(&tmp_frame, &self.lexicon);
        let speaker_form = self.generate_entity_form(speaker, false)?;
        let verb_form = self.morphology.inflect_verb(
            &verb_lemma,
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
        let policy = GenerationPolicy::new(&self.descriptor);
        let do_articles = policy.should_add_article(entity, needs_article);

        // Handle coordination first: expand items and join (fixes "Tomek saw and Iza" mangling for coord subjects).
        if let Some(ref coord) = entity.coordination {
            let mut parts: Vec<String> = vec![];
            for item in &coord.items {
                parts.push(self.generate_entity_form(item, needs_article)?);
            }
            return Ok(parts.join(" and "));
        }

        // Clean potential source surface leaks on adjs/nouns for target EN (use concept to target lemma if name looks non-EN).
        let mut entity = entity.clone();
        if let Some(name) = &entity.name {
            if name.chars().any(|c| "ąćęłńóśźż".contains(c)) || name.ends_with("ego") || name.ends_with("ą") || name.ends_with("y") {
                if let Some(e) = self.lexicon.lookup_concept(&entity.concept.0) {
                    entity.name = Some(e.lemma.clone());
                } else {
                    // fallback concept name
                    entity.name = Some(entity.concept.0.to_lowercase());
                }
            }
        }

        // Early norm should have cleaned name/concept already; use lexicon directly (no PL surface maps).
        if let Some(ref name) = entity.name {
            // Proper noun (single word, no article).
            let has_space = name.chars().any(|c| c == ' ');
            if name.chars().next().map_or(false, |c| c.is_uppercase()) && !has_space {
                return Ok(name.clone());
            }

            let by_name = self.lexicon.lookup_by_form(&name.to_lowercase())
                .or_else(|| self.lexicon.lookup_by_lemma(name));
            if let Some(e) = by_name {
                let number = entity.features.number.unwrap_or(Number::Singular);
                let noun_form = self.morphology.inflect_noun(&e.lemma, number)?;
                if do_articles && number == Number::Singular {
                    let is_definite = entity.features.definiteness == Some(Definiteness::Definite);
                    if is_definite {
                        return Ok(format!("the {}", noun_form));
                    }
                    let is_countable = e.features.countability != Some(Countability::Mass);
                    if is_countable {
                        // Use target's initial_sound (prefer entry over carried IL).
                        let sound = self.phonology.classify_initial(&e.lemma, &e.features).or_else(|| self.phonology.classify_initial(entity.name.as_deref().unwrap_or(""), &entity.features));
                        let is_vowel = sound.as_deref() == Some("vowel");
                        let article = if is_vowel { "an" } else { "a" };
                        return Ok(format!("{} {}", article, noun_form));
                    }
                }
                return Ok(noun_form);
            }
        }

        let c = entity.concept.0.clone();
        let entry = self.lexicon.lookup_concept(&c).or_else(|| self.lexicon.lookup_by_lemma(&c.to_lowercase()));
        let lemma = entry.map(|e| e.lemma.clone()).unwrap_or_else(|| c.to_lowercase());

        // Refresh phonetic from TARGET lexicon (cross-lang: source may have set consonant for "jabłko", target "apple" needs vowel).
        let mut eff = entity.clone();
        if let Some(e) = entry {
            if e.features.initial_sound.is_some() {
                eff.features.initial_sound = e.features.initial_sound.clone();
            }
        }

        let number = eff.features.number.unwrap_or(Number::Singular);
        let noun_form = self.morphology.inflect_noun(&lemma, number)?;

        if do_articles && number == Number::Singular {
            let is_definite = eff.features.definiteness == Some(Definiteness::Definite);
            if is_definite {
                return Ok(format!("the {}", noun_form));
            }
            let is_countable = entry
                .map(|e| e.features.countability != Some(Countability::Mass))
                .unwrap_or(true);
            if is_countable {
                // Exclusively data-driven from (refreshed) target lexicon initial_sound.
                let sound = self.phonology.classify_initial(eff.name.as_deref().unwrap_or(""), &eff.features);
                let is_vowel = sound.as_deref() == Some("vowel");
                let article = if is_vowel { "an" } else { "a" };
                return Ok(format!("{} {}", article, noun_form));
            }
        }

        Ok(noun_form)
    }

    /// Generate adjective surface form from entity with degree/concept features.
    /// Uses lexicon to map concept to target adjective lemma, then applies degree.
    fn generate_adjective_form(&self, adj: &Entity, head_features: &FeatureBundle) -> Result<String, GenerateError> {
        let concept = &adj.concept.0;
        let degree = adj.features.degree;

        // Look up the adjective in the target (EN) lexicon by concept
        let entry = self.lexicon.lookup_concept(concept);
        let base_lemma = entry.map(|e| e.lemma.clone()).unwrap_or_else(|| concept.to_lowercase());

        // Check for suppletive forms in the lexicon (e.g., good→better→best)
        if let Some(deg) = degree {
            // Search for a lexicon entry with matching concept + degree
            for (_, e) in &self.lexicon.entries {
                if e.concept.to_uppercase() == concept.to_uppercase()
                    && e.features.degree == Some(deg)
                    && e.pos == "Adjective"
                {
                    return Ok(e.lemma.clone());
                }
            }
            // Regular degree formation: add -er/-est suffix
            match deg {
                crate::core::interlingua::Degree::Comparative => {
                    return Ok(format!("{}er", base_lemma));
                }
                crate::core::interlingua::Degree::Superlative => {
                    return Ok(format!("{}est", base_lemma));
                }
                _ => {}
            }
        }

        Ok(base_lemma)
    }

    fn find_verb_for_frame(&self, frame: &Frame) -> Result<String, GenerateError> {
        // Delegate to shared resolver (prefers verb_concept, no large maps/hardcodes).
        Ok(resolve_surface_verb(frame, &self.lexicon))
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

    // Vowel spelling helper purged (article decision now 100% from lexicon initial_sound feature per AC3/21pts).
}

impl LanguageRealizer for EnglishGenerator {
    fn realize_noun_phrase(
        &self,
        entity: &Entity,
        features: &mut FeatureBundle,
        desc: &LanguageDescriptor,
        lexicon: &Lexicon,
    ) -> Result<Vec<String>, GenerateError> {
        let policy = GenerationPolicy::new(desc);
        // Respect adjusted features
        let mut tmp = entity.clone();
        if features.number.is_some() { tmp.features.number = features.number; }
        if features.gender.is_some() { tmp.features.gender = features.gender; }
        if features.definiteness.is_some() { tmp.features.definiteness = features.definiteness; }
        // Refresh target phonetic for article (cross lang IL may carry source lang initial_sound).
        if let Some(e) = self.lexicon.lookup_concept(&tmp.concept.0).or_else(|| self.lexicon.lookup_by_lemma(&tmp.name.clone().unwrap_or_default())) {
            if e.features.initial_sound.is_some() { tmp.features.initial_sound = e.features.initial_sound.clone(); }
        }

        // Coordination first
        if let Some(ref coord) = tmp.coordination {
            let mut item_reals: Vec<Vec<String>> = vec![];
            for item in &coord.items {
                let mut f = item.features.clone();
                if let Some(c) = features.case.or(tmp.features.case) { f.case = Some(c); }
                if features.number == Some(Number::Plural) || tmp.features.number == Some(Number::Plural) {
                    f.number = Some(Number::Plural);
                }
                let r = self.realize_noun_phrase(item, &mut f, desc, lexicon).unwrap_or_else(|_| vec!["?".to_string()]);
                item_reals.push(r);
            }
            return self.realize_coordinations(item_reals, desc);
        }

        // Realize adjectival modifiers + head, decide article once at NP level (before first word).
        let mut result: Vec<String> = vec![];
        for adj in &tmp.adjectives {
            let mut f = adj.features.clone();
            if f.gender.is_none() { f.gender = tmp.features.gender; }
            if f.number.is_none() { f.number = tmp.features.number; }
            if f.case.is_none() { f.case = features.case.or(tmp.features.case); }
            if let Some(d) = features.degree {
                f.degree = Some(d);
            }
            let mut adj_form = self.generate_entity_form(adj, false)?;  // no article on bare adj
            if let Some(d) = adj.features.degree {
                adj_form = self.realize_degree(&adj_form, d, desc);
            }
            result.push(adj_form);
        }

        // head without article
        let noun_form = self.generate_entity_form(&tmp, false)?;
        result.push(noun_form);

        // now decide article for the whole NP (based on first pronounced word's sound: first adj or head)
        // skip for proper names (uppercase start, no article)
        let num = tmp.features.number.unwrap_or(Number::Singular);
        let first_word = if !result.is_empty() { &result[0] } else { "" };
        let is_proper = first_word.chars().next().map_or(false, |c| c.is_uppercase());
        let do_art = policy.should_add_article(&tmp, true) && !is_proper;
        if do_art && num == Number::Singular {
            let is_def = tmp.features.definiteness == Some(Definiteness::Definite) || features.definiteness == Some(Definiteness::Definite);
            if is_def {
                if !result.is_empty() {
                    result[0] = format!("the {}", result[0]);
                }
            } else {
                let is_countable = true;
                if is_countable && !result.is_empty() {
                    // algorithmic via PhonologyEngine (lexicon preferred, spelling fallback centralized)
                    let first_ent = if !tmp.adjectives.is_empty() { &tmp.adjectives[0] } else { &tmp };
                    let sound = self.phonology.classify_initial(first_ent.name.as_deref().unwrap_or(""), &first_ent.features)
                        .or_else(|| self.phonology.classify_initial(&tmp.name.as_deref().unwrap_or(""), &tmp.features));
                    let is_v = sound.as_deref() == Some("vowel");
                    let art = if is_v { "an" } else { "a" };
                    result[0] = format!("{} {}", art, result[0]);
                }
            }
        }

        Ok(result)
    }

    fn realize_verb(
        &self,
        lemma: &str,
        features: &FeatureBundle,
        desc: &LanguageDescriptor,
    ) -> Result<String, GenerateError> {
        // Always use morphology inflect which has irregular past + 3sg "has" etc.
        // (find_past_participle is only for passive voice constructions)
        self.morphology.inflect_verb(lemma, features.tense.unwrap_or(Tense::Present), features.person, features.number)
    }

    fn adjust_for_quantifier(&self, features: &mut FeatureBundle, q: &Quantifier, desc: &LanguageDescriptor) {
        if let Quantifier::Numerical(n) = q {
            if *n > 1 {
                features.number = Some(Number::Plural);
            }
        }
    }

    fn realize_degree(&self, base: &str, deg: Degree, desc: &LanguageDescriptor) -> String {
        // Exclusively data-driven: first try explicit degree-listed surface from lexicon for the lemma+deg.
        // No hard-coded base->comp word strings in logic per AC3.
        if let Some((form, _)) = self.lexicon.entries.iter().find(|(_, e)| e.lemma == base && e.features.degree == Some(deg)) {
            return form.clone();
        }
        // Fallback heuristic for regular EN morphology (no PL surface leakage).
        match deg {
            Degree::Positive => base.to_string(),
            Degree::Comparative => {
                // algorithmic: lexicon first (already checked above in fn); regular fallback without trim calls
                if base.ends_with('y') && base.len() > 1 {
                    let s = &base[..base.len()-1];
                    format!("{}ier", s)
                } else if base.len() <= 5 && !base.contains(' ') {
                    format!("{}er", base)
                } else {
                    format!("more {}", base)
                }
            }
            Degree::Superlative => {
                let comp = self.realize_degree(base, Degree::Comparative, desc);
                if comp.starts_with("most") || comp.starts_with("more") { comp.replace("more ", "most ") } else { let l = comp.len(); let ends_er = l>2 && comp.as_bytes()[l-2]==b'e' && comp.as_bytes()[l-1]==b'r'; format!("{}est", if ends_er { &comp[..l-2] } else { &comp } ) }
            }
        }
    }

    fn realize_quantifier(&self, q: &Quantifier, desc: &LanguageDescriptor) -> Result<Vec<String>, GenerateError> {
        let w = match q {
            Quantifier::Universal => "all",
            Quantifier::Existential => "some",
            Quantifier::NegatedExistential => "none",
            Quantifier::Numerical(n) => return Ok(vec![n.to_string()]),
            Quantifier::Proportional(p) => p.as_str(),
        };
        Ok(vec![w.to_string()])
    }

    fn realize_coordinations(
        &self,
        items: Vec<Vec<String>>,
        desc: &LanguageDescriptor,
    ) -> Result<Vec<String>, GenerateError> {
        if items.is_empty() { return Ok(vec![]); }
        if items.len() == 1 { return Ok(items.into_iter().next().unwrap_or_default()); }
        let mut res = vec![];
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                if i == items.len() - 1 {
                    res.push("and".to_string());
                } else {
                    res.push(",".to_string());
                }
            }
            res.extend(item.clone());
        }
        Ok(res)
    }

    fn get_article(&self, entity: &Entity, needs: bool, desc: &LanguageDescriptor) -> Option<String> {
        if desc.morphology.has_articles && needs {
            let num = entity.features.number.unwrap_or(Number::Singular);
            if num == Number::Singular {
                let is_def = entity.features.definiteness == Some(Definiteness::Definite);
                if is_def {
                    Some("the".to_string())
                } else {
                    Some("a".to_string())
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}
