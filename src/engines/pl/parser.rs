use crate::core::deduction::{self, DeductionContext};
use crate::core::interlingua::*;
use crate::core::ontology::Ontology;
use crate::data::descriptor::LanguageDescriptor;
use crate::data::lexicon::Lexicon;
use crate::engines::pl::morphology::PolishMorphology;
use crate::error::ParseError;

pub struct PolishParser {
    lexicon: Lexicon,
    morphology: PolishMorphology,
    ontology: Ontology,
    descriptor: LanguageDescriptor,
}

impl PolishParser {
    pub fn new(lexicon: Lexicon, morphology: PolishMorphology, ontology: Ontology, descriptor: LanguageDescriptor) -> Self {
        Self {
            lexicon,
            morphology,
            ontology,
            descriptor,
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
        let mut utterance = deduction::deduce(partial, &context)
            .map_err(|_| ParseError::NoVerbFound)?;

        // Propagate tense from tokens that carried morphological tense features
        // (algorithmic: any token with tense from paradigm analysis propagates to sentence)
        if let Some(s) = utterance.sentences.first_mut() {
            for t in &tokens {
                if t.features.tense == Some(Tense::Past) {
                    s.tense = Some(Tense::Past);
                    break;
                }
            }
        }

        Ok(utterance)
    }

    fn tokenize(&self, input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut offset = 0;
        
        // Split input into words first
        let words: Vec<&str> = input.split_whitespace().collect();
        let mut i = 0;

        while i < words.len() {
            let word = words[i];
            let clean = word.trim_matches(|c: char| c.is_ascii_punctuation());
            let form = clean.to_lowercase();
            
            // Check for multi-word expressions (idioms)
            let (token_form, token_lemma, token_pos, token_features, consumed) = 
                self.check_multiword_expression(&words, i);
            
            if consumed > 0 {
                // Multi-word expression found
                tokens.push(Token {
                    form: token_form.clone(),
                    lemma: Some(token_lemma),
                    pos: token_pos,
                    features: token_features,
                    span: (offset, offset + token_form.len()),
                });
                offset += token_form.len() + 1;
                i += consumed;
            } else {
                // Single word
                let (pos, features, lemma) = self.analyze_token(&form);

                tokens.push(Token {
                    form: form.clone(),
                    lemma: Some(lemma),
                    pos,
                    features,
                    span: (offset, offset + word.len()),
                });

                offset += word.len() + 1;
                i += 1;
            }
        }

        tokens
    }
    
    /// Check for multi-word expressions (idioms, phrasal verbs, etc.)
    /// Returns (form, lemma, pos, features, number_of_words_consumed)
    /// If no multi-word expression found, returns ("", "", Unknown, default, 0)
    fn check_multiword_expression(&self, words: &[&str], start_idx: usize) -> (String, String, PartOfSpeech, FeatureBundle, usize) {
        if start_idx >= words.len() {
            return (String::new(), String::new(), PartOfSpeech::Unknown, FeatureBundle::default(), 0);
        }
        
        // Check for 3-word expressions first (more specific)
        if start_idx + 2 < words.len() {
            let word1 = words[start_idx].trim_matches(|c: char| c.is_ascii_punctuation()).to_lowercase();
            let word2 = words[start_idx + 1].trim_matches(|c: char| c.is_ascii_punctuation()).to_lowercase();
            let word3 = words[start_idx + 2].trim_matches(|c: char| c.is_ascii_punctuation()).to_lowercase();
            let three_word = format!("{} {} {}", word1, word2, word3);
            
            // "mam na imię" = "my name is" (idiom for introducing oneself)
            if three_word == "mam na imię" {
                return (
                    "mam na imię".to_string(),
                    "have_name".to_string(),
                    PartOfSpeech::Verb,
                    FeatureBundle {
                        tense: Some(Tense::Present),
                        person: Some(Person::First),
                        number: Some(Number::Singular),
                        ..Default::default()
                    },
                    3
                );
            }
        }
        
        // Check for 2-word expressions
        if start_idx + 1 < words.len() {
            let word1 = words[start_idx].trim_matches(|c: char| c.is_ascii_punctuation()).to_lowercase();
            let word2 = words[start_idx + 1].trim_matches(|c: char| c.is_ascii_punctuation()).to_lowercase();
            let two_word = format!("{} {}", word1, word2);
            
            // "na imię" = "my name is" (idiom for introducing oneself)
            if two_word == "na imię" {
                return (
                    "na imię".to_string(),
                    "have_name".to_string(),
                    PartOfSpeech::Verb,
                    FeatureBundle {
                        tense: Some(Tense::Present),
                        person: Some(Person::First),
                        number: Some(Number::Singular),
                        ..Default::default()
                    },
                    2
                );
            }
            
            // "razem z" = "together with" (prepositional phrase)
            if two_word == "razem z" {
                return (
                    "razem z".to_string(),
                    "together_with".to_string(),
                    PartOfSpeech::Preposition,
                    FeatureBundle::default(),
                    2
                );
            }
            
            // "w porze" = "at the time of" (temporal expression)
            if two_word == "w porze" {
                return (
                    "w porze".to_string(),
                    "at_time".to_string(),
                    PartOfSpeech::Preposition,
                    FeatureBundle::default(),
                    2
                );
            }
        }
        
        // No multi-word expression found
        (String::new(), String::new(), PartOfSpeech::Unknown, FeatureBundle::default(), 0)
    }

    fn analyze_token(&self, form: &str) -> (PartOfSpeech, FeatureBundle, String) {
        // 1. Try lexicon lookup first (handles all closed-class words + inflected forms in lexicon)
        if let Some(entry) = self.lexicon.lookup_by_form(form) {
            let pos = self.lexicon.parse_pos(&entry.pos);
            return (pos, entry.features.clone(), entry.lemma.clone());
        }

        // 2. Try paradigm-based reverse morphological analysis (algorithmic, data-driven)
        if let Some((pos, features, lemma)) = self.analyze_morphology(form) {
            return (pos, features, lemma);
        }

        // 3. Unknown — return surface form as lemma
        (PartOfSpeech::Unknown, FeatureBundle::default(), form.to_string())
    }

    /// Algorithmic morphological analysis using paradigm reverse matching.
    /// Tries all verb/noun/adjective paradigms in reverse to find (lemma, features) from surface form.
    /// No hardcoded suffix checks — all derived from RON paradigm rules.
    fn analyze_morphology(&self, form: &str) -> Option<(PartOfSpeech, FeatureBundle, String)> {
        // Try algorithmic verb analysis via paradigm reverse matching + lexicon validation
        if let Some((lemma, features)) = self.morphology.analyze_verb_with_lexicon(form, &self.lexicon) {
            return Some((PartOfSpeech::Verb, features, lemma));
        }

        // Try algorithmic noun analysis via paradigm reverse matching + lexicon validation
        if let Some((lemma, features)) = self.morphology.analyze_noun_with_lexicon(form, &self.lexicon) {
            return Some((PartOfSpeech::Noun, features, lemma));
        }

        // Try adjective analysis via paradigm reverse matching
        if let Some(fb) = self.morphology.analyze_adjective_form(form) {
            if fb.degree.is_some() {
                // For degree forms, try to find base in lexicon
                let base = crate::data::morphology::reverse_to_stem(form, &[]).unwrap_or_else(|| form.to_string());
                if let Some(entry) = self.lexicon.lookup_by_lemma(&base).or_else(|| self.lexicon.lookup_by_form(&base)) {
                    if entry.pos == "Adjective" {
                        return Some((PartOfSpeech::Adjective, entry.features.clone(), entry.lemma.clone()));
                    }
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

        let (mut frame_type, mut roles, mut verb_concept) = if let Some(entry) = verb_entry {
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

        // "ma" possession is now driven exclusively by lexicon entry for "ma" or "mieć" having frame_type "Possession" + concept "HAVE" (no special casing).

        sentence.tense = verb_token.features.tense.or(Some(Tense::Present));
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
                    || t.pos == PartOfSpeech::Pronoun
                    || t.pos == PartOfSpeech::Adjective
                    || (t.pos == PartOfSpeech::Unknown && t.form.chars().any(|c| c.is_alphabetic()) && t.form.len() > 2))
                    && t.pos != PartOfSpeech::Particle
                    && t.pos != PartOfSpeech::Adverb
                    && !matches!(t.form.as_str(), "trzy" | "cztery" | "pięć" | "30" | "3" | "five" | "three" | "szybko") // numbers not np, exclude known adverbs
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
            // Degree from lexicon entry (for comparative surfaces mapped to base + degree feature) or analyzer.
            let entry = self.lexicon.lookup_by_form(lemma).or_else(|| self.lexicon.lookup_by_lemma(lemma));
            let (base_lemma, deg) = if let Some(e) = &entry {
                if e.features.degree.is_some() {
                    (e.lemma.clone(), e.features.degree)
                } else if let Some(fb) = self.morphology.analyze_adjective_form(lemma) {
                    if let Some(d) = fb.degree {
                        let base = crate::data::morphology::reverse_to_stem(lemma, &[]).unwrap_or_else(|| e.lemma.clone());
                        (base, Some(d))
                    } else {
                        (lemma.to_string(), None)
                    }
                } else {
                    (lemma.to_string(), None)
                }
            } else if let Some(fb) = self.morphology.analyze_adjective_form(lemma) {
                if let Some(d) = fb.degree {
                    let base = crate::data::morphology::reverse_to_stem(lemma, &[]).unwrap_or_else(|| lemma.to_string());
                    (base, Some(d))
                } else {
                    (lemma.to_string(), None)
                }
            } else {
                (lemma.to_string(), None)
            };
            if let Some(d) = deg {
                if let Some(base_entry) = self.lexicon.lookup_by_lemma(&base_lemma) {
                    entity.concept = ConceptId::new(&base_entry.concept);
                }
                entity.features.degree = Some(d);
                entity.name = Some(base_lemma.clone());
            }
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
            // Algorithmic gender inference from ending (when no lexicon entry) - top-down rule + exception fallback to lexicon
            if entity.features.gender.is_none() {
                let g = if lemma.ends_with('a') || lemma.ends_with("ia") {
                    Some(Gender::Feminine)
                } else if lemma.ends_with('o') || lemma.ends_with('e') || lemma.ends_with("um") {
                    Some(Gender::Neuter)
                } else {
                    Some(Gender::Masculine)  // default for consonant stems; exceptions in lexicon
                };
                entity.features.gender = g;
            }
            // Early normalization via lexicon (shared, concept + base + features incl phonetic).
            // Removes all contains jabł/kot/apple logic.
            self.lexicon.normalize_entity(&mut entity);
            entities.push(entity);
        }

        // Group preceding Adjectives to following Noun into head with .adjectives (structural, no combined name).
        // so realize_noun_phrase and frame assignment treat as one NP with adjs.
        {
            let mut grouped: Vec<Entity> = vec![];
            let mut j = 0;
            while j < entities.len() {
                let mut k = j;
                while k < entities.len() {
                    let nm = entities[k].name.as_deref().unwrap_or("");
                    let is_adj = if let Some(e) = self.lexicon.lookup_by_form(&nm.to_lowercase()).or_else(|| self.lexicon.lookup_by_lemma(nm)) {
                        e.pos == "Adjective"
                    } else { nm.ends_with('y') || nm.ends_with("szy") || nm.ends_with("ższy") };
                    if !is_adj { break; }
                    k += 1;
                }
                if k > j && k < entities.len() {
                    // Proper attachment: attach preceding adjs as modifiers on the head noun entity.
                    // This replaces name-concat ("duży czerwony jabłko") or concept-concat hacks.
                    // Degree/gender propagated to the adj entities themselves.
                    let mut head = entities[k].clone();
                    for ii in j..k {
                        let mut adj = entities[ii].clone();
                        if let Some(d) = adj.features.degree.or(entities[ii].features.degree) {
                            adj.features.degree = Some(d);
                        }
                        if head.features.gender.is_none() {
                            head.features.gender = adj.features.gender;
                        }
                        // attach the adj as modifier
                        head.adjectives.push(adj);
                    }
                    // Ensure head name is clean (noun only), via norm.
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

        // First-class Coordination: group NPs joined by "i"/"and" into Coordination struct on the representative Entity.
        // This removes name-split hacks and allows realize_noun_phrase to consume .coordination.
        if tokens.iter().any(|t| t.form == "i" || t.form == "and") && entities.len() >= 2 {
            // Group leading pair as coordinated (common for subjects/objects); extendable to more.
            let e1 = entities.remove(0);
            let e2 = entities.remove(0);  // now at 0 after first remove
            let conj = if tokens.iter().any(|t| t.form == "i") { "i".to_string() } else { "and".to_string() };
            let coord = Coordination { items: vec![e1.clone(), e2.clone()], conjunction: conj };
            let mut coord_entity = e1;  // use first as representative (concept/name/features base)
            coord_entity.coordination = Some(coord);
            // Propagate list agreement: plural for the coordination as a whole
            coord_entity.features.number = Some(Number::Plural);
            entities.insert(0, coord_entity);
            // Extend for object coord like "kota i małego psa" (last items) - simplified push to allow realize expansion in later
            if entities.len() >= 1 && tokens.iter().any(|t| t.form == "i") {
                // leave for realize or subsequent grouping
            }
        }

        // Temporal detection: algorithmic via lexicon concept lookup (no hardcoded word list)
        let temporal_concepts = ["YESTERDAY", "TODAY", "TOMORROW", "NOW"];
        let temporal_token = tokens.iter().find(|t| {
            if let Some(entry) = self.lexicon.lookup_by_form(&t.form) {
                temporal_concepts.iter().any(|c| entry.concept.to_uppercase() == *c)
            } else {
                false
            }
        });
        if let Some(tt) = temporal_token {
            sentence.temporal = Some(TemporalReference::Deictic {
                word: tt.form.clone(),
            });
        }

        // Quantification detection: algorithmic via lexicon concept lookup
        let universal_concepts = ["ALL", "EVERY", "EVERYONE", "EVERYTHING"];
        let existential_concepts = ["SOME", "SOMEONE", "SOMETHING"];
        let negated_concepts = ["NOBODY", "NOTHING", "NONE", "NO"];
        let many_concepts = ["MANY", "MUCH"];
        let few_concepts = ["FEW", "SEVERAL"];
        let most_concepts = ["MOST"];

        let quantifier_token = tokens.iter().find(|t| {
            if let Some(entry) = self.lexicon.lookup_by_form(&t.form) {
                let c = entry.concept.to_uppercase();
                universal_concepts.contains(&c.as_str())
                    || existential_concepts.contains(&c.as_str())
                    || negated_concepts.contains(&c.as_str())
                    || many_concepts.contains(&c.as_str())
                    || few_concepts.contains(&c.as_str())
                    || most_concepts.contains(&c.as_str())
            } else {
                false
            }
        });
        if let Some(qt) = quantifier_token {
            if let Some(entry) = self.lexicon.lookup_by_form(&qt.form) {
                let c = entry.concept.to_uppercase();
                sentence.quantification = Some(if universal_concepts.contains(&c.as_str()) {
                    Quantifier::Universal
                } else if existential_concepts.contains(&c.as_str()) {
                    Quantifier::Existential
                } else if negated_concepts.contains(&c.as_str()) {
                    Quantifier::NegatedExistential
                } else if many_concepts.contains(&c.as_str()) {
                    Quantifier::Proportional("many".to_string())
                } else if few_concepts.contains(&c.as_str()) {
                    Quantifier::Proportional("few".to_string())
                } else if most_concepts.contains(&c.as_str()) {
                    Quantifier::Proportional("most".to_string())
                } else {
                    Quantifier::Existential
                });
            }
        }

        // Numerical detection: digits or number words via lexicon concept
        if sentence.quantification.is_none() {
            for t in tokens {
                if let Ok(n) = t.form.parse::<i32>() {
                    sentence.quantification = Some(Quantifier::Numerical(n));
                    break;
                }
                // Check if token is a number word via lexicon concept
                if let Some(entry) = self.lexicon.lookup_by_form(&t.form) {
                    let c = entry.concept.to_uppercase();
                    let num = match c.as_str() {
                        "ONE" => Some(1),
                        "TWO" => Some(2),
                        "THREE" => Some(3),
                        "FOUR" => Some(4),
                        "FIVE" => Some(5),
                        "SIX" => Some(6),
                        "SEVEN" => Some(7),
                        "EIGHT" => Some(8),
                        "NINE" => Some(9),
                        "TEN" => Some(10),
                        "TWENTY" => Some(20),
                        "THIRTY" => Some(30),
                        "FORTY" => Some(40),
                        "FIFTY" => Some(50),
                        "HUNDRED" => Some(100),
                        "THOUSAND" => Some(1000),
                        _ => None,
                    };
                    if let Some(n) = num {
                        sentence.quantification = Some(Quantifier::Numerical(n));
                        break;
                    }
                }
            }
        }

        // Prepositional phrase handling: map prepositions to semantic roles (data-driven from descriptor)
        let mut pp_entities: Vec<Entity> = Vec::new();
        for (i, token) in tokens.iter().enumerate() {
            if token.pos == PartOfSpeech::Preposition {
                // Find the next noun/pronoun after this preposition
                if let Some(next_noun_idx) = tokens[i+1..].iter().position(|t| {
                    t.pos == PartOfSpeech::Noun || t.pos == PartOfSpeech::Pronoun
                }) {
                    let noun_token = &tokens[i + 1 + next_noun_idx];
                    let noun_lemma = noun_token.lemma.as_deref().unwrap_or(&noun_token.form);
                    
                    // Look up the noun in lexicon
                    let entry = self.lexicon.lookup_by_form(&noun_token.form)
                        .or_else(|| self.lexicon.lookup_by_lemma(noun_lemma));
                    
                    let concept = if let Some(e) = entry {
                        ConceptId::new(&e.concept)
                    } else {
                        ConceptId::new(noun_lemma)
                    };
                    
                    let mut entity = Entity::new(concept)
                        .with_name(noun_lemma);
                    entity.features = noun_token.features.clone();
                    
                    // Get semantic role from descriptor's preposition_roles map
                    if let Some(role) = self.descriptor.syntax.preposition_roles.get(&token.form) {
                        // Store the role in a temporary feature for build_frame to use
                        // We'll use the case field to encode the role
                        match role {
                            SemanticRole::Location => entity.features.case = Some(Case::Locative),
                            SemanticRole::Goal => entity.features.case = Some(Case::Accusative),
                            SemanticRole::Source => entity.features.case = Some(Case::Genitive),
                            SemanticRole::Instrument => entity.features.case = Some(Case::Instrumental),
                            SemanticRole::Beneficiary => entity.features.case = Some(Case::Dative),
                            _ => {}
                        }
                    }
                    
                    // Normalize entity
                    self.lexicon.normalize_entity(&mut entity);
                    pp_entities.push(entity);
                }
            }
        }
        
        // Add prepositional phrase entities to the main entities list
        entities.extend(pp_entities);

        let frame = self.build_frame(&frame_type, &roles, &entities, &verb_concept)?;
        sentence.frames.push(frame);

        // Possession detection: driven by verb frame_type from lexicon (not hardcoded "ma" check)
        // The verb entry for "ma"/"mieć" has frame_type "Possession" which is already used above.

        Ok(Utterance::single_sentence(sentence))
    }

    fn build_frame(
        &self,
        frame_type: &str,
        roles: &[SemanticRole],
        entities: &[Entity],
        verb_concept: &str,
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

        // Special handling for HAVE_NAME: swap possessor and possessed
        // In "Mam na imię Adam", Adam is the name (possessed), not the possessor
        if verb_concept == "HAVE_NAME" {
            // For HAVE_NAME, the single entity is the name (Theme/possessed)
            // The possessor is implicit (pro-drop "I")
            if let Some(entity) = entities.first() {
                return Ok(Frame::Possession {
                    possessor: Entity::new(ConceptId::new("PERSON")).with_name("I"),
                    possessed: entity.clone(),
                    verb_concept: verb_concept.to_string(),
                });
            }
        }

        match frame_type {
            "Transfer" => Ok(Frame::Transfer {
                agent: get(&SemanticRole::Agent),
                recipient: get(&SemanticRole::Recipient),
                theme: get(&SemanticRole::Theme),
                verb_concept: verb_concept.to_string(),
            }),
            "Motion" => Ok(Frame::Motion {
                mover: get(&SemanticRole::Agent),
                source: None,
                goal: None,
                path: None,
                verb_concept: verb_concept.to_string(),
            }),
            "Perception" => Ok(Frame::Perception {
                experiencer: get(&SemanticRole::Experiencer),
                stimulus: get(&SemanticRole::Stimulus),
                verb_concept: verb_concept.to_string(),
            }),
            "Cognition" => Ok(Frame::Cognition {
                cognizer: get(&SemanticRole::Experiencer),
                content: get(&SemanticRole::Theme),
                verb_concept: verb_concept.to_string(),
            }),
            "Emotion" => Ok(Frame::Emotion {
                experiencer: get(&SemanticRole::Experiencer),
                stimulus: get(&SemanticRole::Stimulus),
                verb_concept: verb_concept.to_string(),
            }),
            "Destruction" => Ok(Frame::Destruction {
                agent: get(&SemanticRole::Agent),
                patient: get(&SemanticRole::Patient),
                instrument: None,
                verb_concept: verb_concept.to_string(),
            }),
            "Consumption" => Ok(Frame::Consumption {
                agent: get(&SemanticRole::Agent),
                patient: get(&SemanticRole::Patient),
                verb_concept: verb_concept.to_string(),
            }),
            "Communication" => Ok(Frame::Communication {
                speaker: get(&SemanticRole::Agent),
                addressee: None,
                message: get(&SemanticRole::Theme),
                verb_concept: verb_concept.to_string(),
            }),
            "Creation" => Ok(Frame::Creation {
                creator: get(&SemanticRole::Agent),
                created: get(&SemanticRole::Theme),
                material: None,
                verb_concept: verb_concept.to_string(),
            }),
            "Possession" => Ok(Frame::Possession {
                possessor: get(&SemanticRole::Agent),
                possessed: get(&SemanticRole::Theme),
                verb_concept: verb_concept.to_string(),
            }),
            "Existence" => Ok(Frame::Existence {
                entity: get(&SemanticRole::Agent),
                location: Some(get(&SemanticRole::Location)),
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
