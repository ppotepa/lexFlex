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

        // Detect clause boundaries and split into clause groups
        let clause_groups = self.split_into_clauses(&tokens);

        if clause_groups.len() <= 1 {
            // Single clause — proceed as before
            let partial = self.build_partial_structure(&tokens)?;
            let context = DeductionContext::new(&self.lexicon, &self.ontology, LanguageId::new("pl"));
            let mut utterance = deduction::deduce(partial, &context)
                .map_err(|_| ParseError::NoVerbFound)?;

            // Propagate tense from tokens
            if let Some(s) = utterance.sentences.first_mut() {
                for t in &tokens {
                    if t.features.tense == Some(Tense::Past) {
                        s.tense = Some(Tense::Past);
                        break;
                    }
                }
            }
            return Ok(utterance);
        }

        // Multiple clauses — parse each separately
        let mut all_sentences = vec![];
        for clause_tokens in &clause_groups {
            if clause_tokens.is_empty() { continue; }
            // Skip if no verb in this clause
            if !clause_tokens.iter().any(|t| t.pos == PartOfSpeech::Verb) { continue; }

            match self.build_partial_structure(clause_tokens) {
                Ok(partial) => {
                    let context = DeductionContext::new(&self.lexicon, &self.ontology, LanguageId::new("pl"));
                    if let Ok(mut utterance) = deduction::deduce(partial, &context) {
                        // Propagate tense
                        if let Some(s) = utterance.sentences.first_mut() {
                            for t in clause_tokens.iter() {
                                if t.features.tense == Some(Tense::Past) {
                                    s.tense = Some(Tense::Past);
                                    break;
                                }
                            }
                        }
                        all_sentences.extend(utterance.sentences);
                    }
                }
                Err(_) => continue,
            }
        }

        if all_sentences.is_empty() {
            return Err(ParseError::NoVerbFound);
        }

        Ok(Utterance { sentences: all_sentences, discourse: None })
    }

    /// Split tokens into clause groups based on clause boundary conjunctions.
    /// Returns a vec of token vectors, one per clause.
    fn split_into_clauses(&self, tokens: &[Token]) -> Vec<Vec<Token>> {
        // Conjunctions that always mark clause boundaries
        let always_boundary = ["ale", "a", "że", "bo", "ponieważ", "jeśli", "jeżeli", "gdy", "kiedy", "dlatego"];
        // Conjunctions that mark clause boundaries only when there's a verb on both sides
        let sometimes_boundary = ["i", "oraz", "lub", "albo"];

        let mut boundaries: Vec<usize> = vec![];

        for (i, token) in tokens.iter().enumerate() {
            if always_boundary.contains(&token.form.as_str()) {
                boundaries.push(i);
            } else if sometimes_boundary.contains(&token.form.as_str()) {
                let verb_before = tokens[..i].iter().any(|t| t.pos == PartOfSpeech::Verb);
                let verb_after = tokens[i+1..].iter().any(|t| t.pos == PartOfSpeech::Verb);
                if verb_before && verb_after {
                    boundaries.push(i);
                }
            }
        }

        if boundaries.is_empty() {
            return vec![tokens.to_vec()];
        }

        let mut groups: Vec<Vec<Token>> = vec![];
        let mut start = 0;

        for &boundary in &boundaries {
            if boundary > start {
                groups.push(tokens[start..boundary].to_vec());
            }
            start = boundary + 1;
        }

        if start < tokens.len() {
            groups.push(tokens[start..].to_vec());
        }

        groups
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
            
            // Reflexive verbs with "się"
            if two_word.ends_with(" się") {
                let verb_form = two_word.trim_end_matches(" się");
                // Check if this verb form exists in lexicon as reflexive
                if let Some(entry) = self.lexicon.lookup_by_form(&two_word) {
                    eprintln!("DEBUG: Found reflexive verb '{}' in lexicon, features: {:?}", two_word, entry.features);
                    let pos = self.lexicon.parse_pos(&entry.pos);
                    return (
                        two_word.clone(),
                        entry.lemma.clone(),
                        pos,
                        entry.features.clone(),
                        2
                    );
                }
                // Check if base verb exists (without "się")
                if let Some(entry) = self.lexicon.lookup_by_form(verb_form) {
                    eprintln!("DEBUG: Found base verb '{}' in lexicon, features: {:?}", verb_form, entry.features);
                    let pos = self.lexicon.parse_pos(&entry.pos);
                    return (
                        two_word.clone(),
                        entry.lemma.clone(),
                        pos,
                        entry.features.clone(),
                        2
                    );
                }
                eprintln!("DEBUG: No lexicon entry for '{}' or '{}'", two_word, verb_form);
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
        eprintln!("DEBUG: verb_token.form={}, verb_token.features.person={:?}", verb_token.form, verb_token.features.person);
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

        // Pro-drop detection: if verb has 1st/2nd person and no explicit pronoun, add implicit subject
        let verb_person = verb_token.features.person;
        let verb_number = verb_token.features.number;
        eprintln!("DEBUG: verb_person={:?}, verb_number={:?}", verb_person, verb_number);
        let has_explicit_subject = tokens.iter().any(|t| {
            t.pos == PartOfSpeech::Pronoun && t.features.person == verb_person
        });
        eprintln!("DEBUG: has_explicit_subject={}", has_explicit_subject);

        let implicit_subject = if !has_explicit_subject && verb_person.is_some() {
            let (pronoun_name, pronoun_person) = match (verb_person, verb_number) {
                (Some(Person::First), Some(Number::Singular)) => (Some("I"), Some(Person::First)),
                (Some(Person::First), Some(Number::Plural)) => (Some("we"), Some(Person::First)),
                (Some(Person::Second), Some(Number::Singular)) => (Some("you"), Some(Person::Second)),
                (Some(Person::Second), Some(Number::Plural)) => (Some("you"), Some(Person::Second)),
                _ => (None, None),
            };
            eprintln!("DEBUG: pronoun_name={:?}, pronoun_person={:?}", pronoun_name, pronoun_person);
            pronoun_name.map(|name| {
                let mut entity = Entity::new(ConceptId::new("PERSON"))
                    .with_name(name);
                entity.features.person = pronoun_person;
                entity.features.number = verb_number;
                eprintln!("DEBUG: Created implicit subject: {:?}", entity.name);
                entity
            })
        } else {
            None
        };

        let negation = tokens.iter().any(|t| t.pos == PartOfSpeech::Negation);
        if negation {
            sentence.polarity = Polarity::Negative;
        }

        let question = tokens.iter().any(|t| t.form == "czy");
        if question {
            sentence.illocution = Illocution::Question;
        }

        // Detect reflexive: "się" token OR reflexive verb lemma
        let has_reflexive = tokens.iter().any(|t| t.form == "się")
            || verb_lemma.ends_with(" się")
            || matches!(verb_concept.as_str(), "DRESS_ONESELF" | "WASH_ONESELF" | "COMB_ONESELF");
        if has_reflexive {
            sentence.reflexive = true;
        }

        // Detect passive voice: być/zostać + passive participle
        let has_passive_aux = tokens.iter().any(|t| {
            matches!(t.lemma.as_deref(), Some("być") | Some("zostać"))
        });
        let has_passive_participle = tokens.iter().any(|t| {
            if t.pos == PartOfSpeech::Participle { return true; }
            // Suffix heuristic: only if token is NOT an adjective in the lexicon
            if t.form.ends_with("ny") || t.form.ends_with("na") || t.form.ends_with("ne")
                || t.form.ends_with("ty") || t.form.ends_with("ta") || t.form.ends_with("te")
            {
                // Check if it's actually an adjective
                let entry = self.lexicon.lookup_by_form(&t.form)
                    .or_else(|| self.lexicon.lookup_by_lemma(t.lemma.as_deref().unwrap_or(&t.form)));
                if let Some(e) = entry {
                    return e.pos == "Participle";
                }
                return true; // unknown form with passive suffix — assume participle
            }
            false
        });
        if has_passive_aux && has_passive_participle {
            sentence.voice = Some(Voice::Passive);
        }

        let np_tokens: Vec<(usize, &Token)> = tokens
            .iter()
            .enumerate()
            .filter(|(idx, t)| {
                // Exclude tokens that are part of prepositional phrases
                let is_after_preposition = if *idx > 0 {
                    tokens[*idx - 1].pos == PartOfSpeech::Preposition
                } else {
                    false
                };

                (t.pos == PartOfSpeech::Noun
                    || t.pos == PartOfSpeech::Pronoun
                    || t.pos == PartOfSpeech::Adjective
                    || (t.pos == PartOfSpeech::Unknown && t.form.chars().any(|c| c.is_alphabetic()) && t.form.len() > 2))
                    && t.pos != PartOfSpeech::Particle
                    && t.pos != PartOfSpeech::Adverb
                    && t.pos != PartOfSpeech::Conjunction  // exclude conjunctions (i, oraz, and)
                    && t.form != "się"  // exclude reflexive marker
                    && !matches!(t.form.as_str(), "trzy" | "cztery" | "pięć" | "30" | "3" | "five" | "three" | "szybko") // numbers not np, exclude known adverbs
                    && !is_after_preposition // exclude nouns after prepositions (they're handled in pp_entities)
            })
            .map(|(idx, t)| (idx, t))
            .collect();

        let mut entities: Vec<Entity> = Vec::new();
        
        // Track which entities are before vs after the verb (for coordination grouping)
        let mut entity_pre_verb: Vec<bool> = Vec::new();

        // Add implicit subject (pro-drop) if detected
        if let Some(subject) = implicit_subject {
            entities.push(subject);
            entity_pre_verb.push(true);
        }

        for (np_idx, np) in &np_tokens {
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
            entity_pre_verb.push(*np_idx < verb_idx);
            entities.push(entity);
        }

        // Group preceding Adjectives to following Noun into head with .adjectives (structural, no combined name).
        // so realize_noun_phrase and frame assignment treat as one NP with adjs.
        {
            let mut grouped: Vec<Entity> = vec![];
            let mut grouped_pre_verb: Vec<bool> = vec![];
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
                    grouped_pre_verb.push(entity_pre_verb[k]);
                    j = k + 1;
                } else {
                    grouped.push(entities[j].clone());
                    grouped_pre_verb.push(entity_pre_verb[j]);
                    j += 1;
                }
            }
            entities = grouped;
            entity_pre_verb = grouped_pre_verb;
        }

        // First-class Coordination: group NPs joined by conjunctions into Coordination struct.
        // Split by verb position so pre-verb entities (subjects) and post-verb entities (objects)
        // are coordinated separately — "Tomek i Iza ma jabłko" → subject=[Tomek+Iza], object=[jabłko].
        let coord_conjunctions = ["i", "oraz", "and", "albo", "lub", "or", ","];
        let has_coordination = tokens.iter().any(|t| coord_conjunctions.contains(&t.form.as_str()));
        if has_coordination && entities.len() >= 2 {
            // Split entities by verb position
            let mut pre_verb_ents: Vec<Entity> = vec![];
            let mut post_verb_ents: Vec<Entity> = vec![];
            for (i, entity) in entities.drain(..).enumerate() {
                if entity_pre_verb[i] {
                    pre_verb_ents.push(entity);
                } else {
                    post_verb_ents.push(entity);
                }
            }

            // Find conjunction tokens split by verb position
            let conj_tokens: Vec<(usize, &str)> = tokens.iter().enumerate()
                .filter(|(_, t)| coord_conjunctions.contains(&t.form.as_str()))
                .map(|(i, t)| (i, t.form.as_str()))
                .collect();

            let pre_verb_conj = conj_tokens.iter()
                .find(|(i, _)| *i < verb_idx)
                .map(|(_, c)| c.to_string());
            let post_verb_conj = conj_tokens.iter()
                .find(|(i, _)| *i >= verb_idx)
                .map(|(_, c)| c.to_string());

            // Coordinate pre-verb entities (subjects) if 2+ and conjunction exists before verb
            if pre_verb_ents.len() >= 2 {
                if let Some(c) = pre_verb_conj {
                    let first = pre_verb_ents.remove(0);
                    let items: Vec<Entity> = std::iter::once(first.clone()).chain(pre_verb_ents.into_iter()).collect();
                    let coord = Coordination { items, conjunction: c };
                    let mut coord_entity = first;
                    coord_entity.coordination = Some(coord);
                    coord_entity.features.number = Some(Number::Plural);
                    pre_verb_ents = vec![coord_entity];
                }
            }

            // Coordinate post-verb entities (objects) if 2+ and conjunction exists after verb
            if post_verb_ents.len() >= 2 {
                if let Some(c) = post_verb_conj {
                    let first = post_verb_ents.remove(0);
                    let items: Vec<Entity> = std::iter::once(first.clone()).chain(post_verb_ents.into_iter()).collect();
                    let coord = Coordination { items, conjunction: c };
                    let mut coord_entity = first;
                    coord_entity.coordination = Some(coord);
                    post_verb_ents = vec![coord_entity];
                }
            }

            // Reassemble: pre-verb (subjects) first, then post-verb (objects)
            entities.extend(pre_verb_ents);
            entities.extend(post_verb_ents);
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
                // Find the next noun/pronoun after this preposition, skipping adjectives
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
                        // Store the semantic role directly in entity features
                        entity.features.semantic_role = Some(*role);
                        // Also set the case field for backward compatibility
                        match role {
                            SemanticRole::Location => entity.features.case = Some(Case::Locative),
                            SemanticRole::Goal => entity.features.case = Some(Case::Accusative),
                            SemanticRole::Source => entity.features.case = Some(Case::Genitive),
                            SemanticRole::Instrument => entity.features.case = Some(Case::Instrumental),
                            SemanticRole::Beneficiary => entity.features.case = Some(Case::Dative),
                            _ => {}
                        }
                    }

                    // Collect adjectives before the noun as modifiers
                    for j in (i+1)..(i+1+next_noun_idx) {
                        if tokens[j].pos == PartOfSpeech::Adjective {
                            let adj_lemma = tokens[j].lemma.as_deref().unwrap_or(&tokens[j].form);
                            let adj_entry = self.lexicon.lookup_by_form(&tokens[j].form)
                                .or_else(|| self.lexicon.lookup_by_lemma(adj_lemma));

                            if let Some(adj_e) = adj_entry {
                                let mut adj_entity = Entity::new(ConceptId::new(&adj_e.concept))
                                    .with_name(adj_lemma);
                                adj_entity.features = tokens[j].features.clone();
                                entity.adjectives.push(adj_entity);
                            }
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

        // Pass 0: assign entities with explicit person (pro-drop subjects)
        // Person 1st/2nd → Agent/Experiencer (subject of the verb)
        let mut unassigned: Vec<Entity> = Vec::new();
        for entity in entities {
            if let Some(person) = entity.features.person {
                if person == Person::First || person == Person::Second {
                    // Try to assign to Agent or Experiencer
                    if roles.contains(&SemanticRole::Agent) {
                        if let Some(idx) = roles.iter().position(|r| *r == SemanticRole::Agent) {
                            if assigned[idx].is_none() {
                                assigned[idx] = Some(entity.clone());
                                continue;
                            }
                        }
                    }
                    if roles.contains(&SemanticRole::Experiencer) {
                        if let Some(idx) = roles.iter().position(|r| *r == SemanticRole::Experiencer) {
                            if assigned[idx].is_none() {
                                assigned[idx] = Some(entity.clone());
                                continue;
                            }
                        }
                    }
                }
            }
            unassigned.push(entity.clone());
        }

        // Pass 1: assign entities with explicit semantic role or case markings
        // Map semantic role or case to available role in the frame
        let mut still_unassigned: Vec<Entity> = Vec::new();
        for entity in unassigned {
            // First, check if entity has an explicit semantic role from preposition
            if let Some(role) = entity.features.semantic_role {
                if roles.contains(&role) {
                    if let Some(idx) = roles.iter().position(|r| *r == role) {
                        if assigned[idx].is_none() {
                            assigned[idx] = Some(entity.clone());
                            continue;
                        }
                    }
                }
            }
            
            // Fall back to case-based role inference
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
            still_unassigned.push(entity.clone());
        }

        // Pass 2: assign remaining entities using animacy heuristics
        // Animate entities prefer Agent/Experiencer, inanimate prefer Theme/Patient
        for entity in &still_unassigned {
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

        // Special handling for HAVE_NAME: the name is the last non-pro-drop entity
        // In "Mam na imię Adam", entities = [pro-drop "I", "Adam"] → Adam is the name
        if verb_concept == "HAVE_NAME" {
            let name_entity = entities.iter()
                .rev()
                .find(|e| e.name.as_deref() != Some("I") || e.features.person.is_none())
                .or(entities.last());
            if let Some(entity) = name_entity {
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
                source: if get(&SemanticRole::Source).concept.0 != "unknown" { Some(get(&SemanticRole::Source)) } else { None },
                goal: if get(&SemanticRole::Goal).concept.0 != "unknown" { Some(get(&SemanticRole::Goal)) } else { None },
                path: None,
                verb_concept: verb_concept.to_string(),
            }),
            "Perception" => Ok(Frame::Perception {
                experiencer: get(&SemanticRole::Experiencer),
                stimulus: get(&SemanticRole::Stimulus),
                verb_concept: verb_concept.to_string(),
            }),
            "Cognition" => {
                // Try multiple role combinations for flexibility
                let cognizer = if roles.contains(&SemanticRole::Cognizer) {
                    get(&SemanticRole::Cognizer)
                } else if roles.contains(&SemanticRole::Experiencer) {
                    get(&SemanticRole::Experiencer)
                } else if roles.contains(&SemanticRole::Agent) {
                    get(&SemanticRole::Agent)
                } else {
                    Entity::new(ConceptId::new("unknown"))
                };
                
                let content = if roles.contains(&SemanticRole::Content) {
                    get(&SemanticRole::Content)
                } else if roles.contains(&SemanticRole::Theme) {
                    get(&SemanticRole::Theme)
                } else {
                    Entity::new(ConceptId::new("unknown"))
                };
                
                Ok(Frame::Cognition {
                    cognizer,
                    content,
                    verb_concept: verb_concept.to_string(),
                })
            },
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
            "Existence" => {
                // Get entity from Agent or Theme role (different verbs use different roles)
                let mut theme = get(&SemanticRole::Theme);
                if theme.concept.0 == "unknown" {
                    theme = get(&SemanticRole::Agent);
                }
                let loc = get(&SemanticRole::Location);
                // Helper: check if an entity name is an adjective in the lexicon
                let is_adj_entity = |name: &str| -> bool {
                    self.lexicon.lookup_by_form(name)
                        .or_else(|| self.lexicon.lookup_by_lemma(name))
                        .map_or(false, |e| e.pos == "Adjective")
                };
                // If "location" is actually a predicate adjective, all entities are predicate adjectives
                // → "jest jasny, czysty i wygodny" = "It is bright, clean and comfortable"
                if loc.concept.0 != "unknown" {
                    let loc_name = loc.name.as_deref().unwrap_or("");
                    if is_adj_entity(loc_name) {
                        // Collect all adjective entities as the property
                        let all_adj: Vec<Entity> = entities.iter()
                            .filter(|e| is_adj_entity(e.name.as_deref().unwrap_or("")))
                            .cloned()
                            .collect();
                        let property = if all_adj.len() >= 2 {
                            let first = all_adj[0].clone();
                            let coord = Coordination { items: all_adj, conjunction: "i".to_string() };
                            let mut ce = first;
                            ce.coordination = Some(coord);
                            ce
                        } else if all_adj.len() == 1 {
                            all_adj[0].clone()
                        } else {
                            loc.clone()
                        };
                        // Use non-adjective entities as subject, "it" if none
                        let non_adj: Vec<Entity> = entities.iter()
                            .filter(|e| !is_adj_entity(e.name.as_deref().unwrap_or("")))
                            .cloned()
                            .collect();
                        let subject = if non_adj.is_empty() {
                            Entity::new(ConceptId::new("DUMMY_SUBJECT")).with_name("it")
                        } else if non_adj.len() == 1 {
                            non_adj[0].clone()
                        } else {
                            let first = non_adj[0].clone();
                            let coord = Coordination { items: non_adj, conjunction: "i".to_string() };
                            let mut ce = first;
                            ce.coordination = Some(coord);
                            ce
                        };
                        return Ok(Frame::Statement {
                            subject,
                            property,
                            verb_concept: verb_concept.to_string(),
                        });
                    }
                }
                // If entity itself is an adjective concept and no location, treat as Statement
                let theme_name = theme.name.as_deref().unwrap_or("");
                if is_adj_entity(theme_name) && loc.concept.0 == "unknown" {
                    return Ok(Frame::Statement {
                        subject: Entity::new(ConceptId::new("DUMMY_SUBJECT")).with_name("it"),
                        property: theme,
                        verb_concept: verb_concept.to_string(),
                    });
                }
                Ok(Frame::Existence {
                    entity: theme,
                    location: if loc.concept.0 != "unknown" { Some(loc) } else { None },
                    verb_concept: verb_concept.to_string(),
                })
            },
            "Custom" => {
                // Custom frame for reflexive verbs and other special cases
                let agent = get(&SemanticRole::Agent);
                Ok(Frame::Custom {
                    name: verb_concept.to_string(),
                    roles: vec![(SemanticRole::Agent, agent)],
                })
            },
            other => {
                Ok(Frame::Statement {
                    subject: entities.first().cloned().unwrap_or(Entity::new(ConceptId::new("unknown"))),
                    property: entities.get(1).cloned().unwrap_or(Entity::new(ConceptId::new("unknown"))),
                    verb_concept: verb_concept.to_string(),
                })
            },
        }
    }
}

fn parse_role_str(s: &str) -> Option<SemanticRole> {
    crate::core::utils::parse_role_str(s)
}
