use crate::core::context;
use crate::core::deduction::{self, DeductionContext};
use crate::core::graph::{self, EdgeKind, GraphNode, LinguisticGraph, TrackedEntity};
use crate::core::interlingua::*;
use crate::core::ontology::Ontology;
use crate::data::descriptor::LanguageDescriptor;
use crate::data::lexicon::Lexicon;
use crate::data::morphology::analyze_morph;
use crate::engines::en::morphology::EnglishMorphology;
use crate::error::ParseError;
use crate::core::unknown_concept::{resolve_concept_for_unknown, is_entity_candidate_token};

pub struct EnglishParser {
    lexicon: Lexicon,
    _morphology: EnglishMorphology,
    ontology: Ontology,
    descriptor: LanguageDescriptor,
    concept_ids: Vec<String>,
}

impl EnglishParser {
    pub fn new(lexicon: Lexicon, morphology: EnglishMorphology, ontology: Ontology, descriptor: LanguageDescriptor, concept_ids: Vec<String>) -> Self {
        Self {
            lexicon,
            _morphology: morphology,
            ontology,
            descriptor,
            concept_ids,
        }
    }

    pub fn parse(&self, input: &str) -> Result<Utterance, ParseError> {
        if input.trim().is_empty() {
            return Err(ParseError::EmptyInput);
        }

        let sentence_parts = context::split_sentence_boundaries(input);
        if sentence_parts.len() > 1 {
            let mut all_sentences = Vec::new();
            for part in sentence_parts {
                if let Ok(utt) = self.parse_single(&part) {
                    all_sentences.extend(utt.sentences);
                }
            }
            if all_sentences.is_empty() {
                return Err(ParseError::NoVerbFound);
            }
            let mut utterance = Utterance {
                sentences: all_sentences,
                discourse: None,
                utterance_node_id: None,
            };
            context::track_discourse(&mut utterance);
            return Ok(utterance);
        }

        let mut utterance = self.parse_single(input)?;
        context::track_discourse(&mut utterance);
        Ok(utterance)
    }

    fn parse_single(&self, input: &str) -> Result<Utterance, ParseError> {
        let tokens = self.tokenize(input);
        let partial = self.build_partial_structure(&tokens)?;

        let context = DeductionContext::new(
            &self.lexicon,
            &self.ontology,
            LanguageId::new("en"),
        );
        deduction::deduce(partial, &context).map_err(|_| ParseError::NoVerbFound)
    }

    fn tokenize(&self, input: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut offset = 0;

        for word in input.split_whitespace() {
            let clean = word.trim_matches(|c: char| c.is_ascii_punctuation());
            let form_lower = clean.to_lowercase();

            let (pos, features, lemma) = self.analyze_token(&form_lower);

            tokens.push(Token {
                form: clean.to_string(),
                lemma: Some(lemma),
                pos,
                features,
                span: (offset, offset + word.len()),
                word_node_id: None,
            });

            offset += word.len() + 1;
        }

        tokens
    }

    fn analyze_token(&self, form: &str) -> (PartOfSpeech, FeatureBundle, String) {
        match form {
            "with" | "in" | "on" | "at" | "from" | "to" => {
                return (PartOfSpeech::Preposition, FeatureBundle::default(), form.to_string());
            }
            "and" => return (PartOfSpeech::Conjunction, FeatureBundle::default(), "and".to_string()),
            "not" | "n't" => return (PartOfSpeech::Negation, FeatureBundle::default(), "not".to_string()),
            "a" | "an" | "the" => {
                let def = if form.eq("the") { Definiteness::Definite } else { Definiteness::Indefinite };
                return (PartOfSpeech::Determiner, FeatureBundle { definiteness: Some(def), ..Default::default() }, form.to_string());
            }
            _ => {}
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

        // Detect questions (does/do/did / ? ). Sets illocution so PL generator emits "Czy ... ?"
        let is_question = tokens.iter().any(|t| {
            let f = t.form.to_lowercase();
            f == "did" || f == "does" || f == "do" || f == "?"
        }) || tokens.first().map_or(false, |t| t.form.eq_ignore_ascii_case("did") || t.form.eq_ignore_ascii_case("does") || t.form.eq_ignore_ascii_case("do"));
        if is_question {
            sentence.illocution = Illocution::Question;
        }

        // Find main verb - skip auxiliary "do/does/did" in questions
        let verb_idx = if is_question && tokens.first().map_or(false, |t| {
            let f = t.form.to_lowercase();
            f == "did" || f == "does" || f == "do"
        }) {
            // Skip first token (auxiliary) and find next verb
            tokens.iter().skip(1).position(|t| t.pos == PartOfSpeech::Verb).map(|i| i + 1)
        } else {
            tokens.iter().position(|t| t.pos == PartOfSpeech::Verb)
        }.ok_or(ParseError::NoVerbFound)?;

        let verb_token = &tokens[verb_idx];
        let verb_lemma = verb_token.lemma.as_deref().unwrap_or(&verb_token.form);

        let verb_entry = self
            .lexicon
            .lookup_by_form(&verb_token.form)
            .or_else(|| self.lexicon.lookup_by_form(verb_lemma))
            .or_else(|| self.lexicon.lookup_by_lemma(verb_lemma));

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

        let pp_span = Self::pp_span_indices(tokens);
        let np_tokens: Vec<(usize, &Token)> = tokens
            .iter()
            .enumerate()
            .filter(|(idx, t)| {
                let f = t.form.to_lowercase();
                !pp_span.contains(idx)
                    && t.pos != PartOfSpeech::Conjunction
                    && t.pos != PartOfSpeech::Preposition
                    && !matches!(f.as_str(), "years" | "year" | "old")
                    && f.parse::<i32>().is_err()
                    && is_entity_candidate_token(t)
                    && t.pos != PartOfSpeech::Particle
            })
            .collect();

        let mut entities: Vec<Entity> = Vec::new();
        let mut entity_pre_verb: Vec<bool> = Vec::new();
        for (i, (np_idx, np)) in np_tokens.iter().enumerate() {
            let surface = &np.form;
            let lookup = surface.to_lowercase();
            let lemma = np.lemma.as_deref().unwrap_or(&lookup);
            let entry = self.lexicon.lookup_by_form(&lookup)
                .or_else(|| self.lexicon.lookup_by_lemma(lemma));

            let concept = resolve_concept_for_unknown(&self.lexicon, surface, lemma, None, &self.concept_ids, Some("en"));

            // Preserve original surface form as .name for all entity candidates (strengthen AC1)
            let name = surface.clone();

            let mut entity = Entity::new(concept)
                .with_name(&name);
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
                if entity.features.degree.is_none() {
                    if let Some((stem, fb)) = analyze_morph(nm, &[], &self.lexicon) {
                        if fb.degree == Some(Degree::Comparative) {
                            entity.name = Some(stem);
                            entity.features.degree = fb.degree;
                        }
                    }
                }
            }

            entity_pre_verb.push(*np_idx < verb_idx);
            entities.push(entity);
        }

        // Group preceding Adjectives to following Noun (for comparative + noun, e.g. big red apple)
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
                    } else { nm.len() > 2 && (nm.as_bytes()[nm.len()-2] == b'e' || nm.as_bytes()[nm.len()-1] == b'y' || nm.ends_with("ous") || nm.ends_with("ful") || nm.ends_with("al")) };
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

        let is_coord_token =
            |t: &Token| t.pos == PartOfSpeech::Conjunction || t.form == ",";
        let has_coordination = tokens.iter().any(is_coord_token);
        let has_non_pp_coordination = has_coordination
            && !tokens
                .iter()
                .enumerate()
                .filter(|(_, t)| is_coord_token(t))
                .all(|(i, _)| pp_span.contains(&i));
        if has_non_pp_coordination && entities.len() >= 2 {
            let mut pre_verb_ents: Vec<Entity> = vec![];
            let mut post_verb_ents: Vec<Entity> = vec![];
            for (i, entity) in entities.drain(..).enumerate() {
                if entity_pre_verb[i] {
                    pre_verb_ents.push(entity);
                } else {
                    post_verb_ents.push(entity);
                }
            }
            let conj_tokens: Vec<(usize, &str)> = tokens
                .iter()
                .enumerate()
                .filter(|(_, t)| is_coord_token(t))
                .map(|(i, t)| (i, t.form.as_str()))
                .collect();
            let pre_verb_conj = conj_tokens
                .iter()
                .find(|(i, _)| *i < verb_idx)
                .map(|(_, c)| c.to_string());
            let post_verb_conj = conj_tokens
                .iter()
                .find(|(i, _)| *i >= verb_idx)
                .map(|(_, c)| c.to_string());
            if pre_verb_ents.len() >= 2 {
                if let Some(c) = pre_verb_conj {
                    let first = pre_verb_ents.remove(0);
                    let items: Vec<Entity> =
                        std::iter::once(first.clone()).chain(pre_verb_ents.into_iter()).collect();
                    let coord = Coordination {
                        items,
                        conjunction: c,
                    };
                    let mut coord_entity = first;
                    coord_entity.coordination = Some(coord);
                    coord_entity.features.number = Some(Number::Plural);
                    pre_verb_ents = vec![coord_entity];
                }
            }
            if post_verb_ents.len() >= 2 {
                if let Some(c) = post_verb_conj {
                    let first = post_verb_ents.remove(0);
                    let items: Vec<Entity> =
                        std::iter::once(first.clone()).chain(post_verb_ents.into_iter()).collect();
                    let coord = Coordination {
                        items,
                        conjunction: c,
                    };
                    let mut coord_entity = first;
                    coord_entity.coordination = Some(coord);
                    post_verb_ents = vec![coord_entity];
                }
            }
            entities.extend(pre_verb_ents);
            entities.extend(post_verb_ents);
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
                if let Some(n) = self.lexicon.cardinal_from_token(t) {
                    sentence.quantification = Some(Quantifier::Numerical(n));
                    break;
                }
            }
        }

        let pp_entities = self.extract_pp_entities(tokens);
        let mut frame_entities = entities.clone();
        frame_entities.extend(pp_entities);

        let frame = if let Some(age) = self.try_age_idiom_frame(tokens, &frame_entities, &verb_concept) {
            age
        } else {
            self.build_frame(&frame_type, &roles, &frame_entities, &verb_concept)?
        };
        sentence.frames.push(frame.clone());

        let (mut graph, word_ids) = LinguisticGraph::from_tokens(tokens);
        // Fully populate evokes for words from lexicon or resolver (symmetric to PL, including unknowns)
        for (i, &wid) in word_ids.iter().enumerate() {
            if let Some(tok) = tokens.get(i) {
                let lookup = tok.form.to_lowercase();
                let lemma = tok.lemma.as_deref().unwrap_or(&lookup);
                if let Some(entry) = self.lexicon.lookup_by_form(&lookup).or_else(|| self.lexicon.lookup_by_lemma(lemma)) {
                    if let Some(GraphNode::Word(w)) = graph.nodes.get_mut(wid.0 as usize) {
                        w.evokes = Some(ConceptId::new(&entry.concept));
                    }
                    graph.add_edge(wid, wid, EdgeKind::EvokesConcept);
                } else {
                    let concept = resolve_concept_for_unknown(&self.lexicon, &tok.form, lemma, None, &self.concept_ids, Some("en"));
                    if let Some(GraphNode::Word(w)) = graph.nodes.get_mut(wid.0 as usize) {
                        w.evokes = Some(concept);
                    }
                    graph.add_edge(wid, wid, EdgeKind::EvokesConcept);
                }
            }
        }
        let tracked: Vec<TrackedEntity> = graph::collect_frame_entities(&frame)
            .into_iter()
            .map(|e| TrackedEntity::new(e.clone(), graph::match_entity_to_tokens(&e, tokens)))
            .collect();
        graph.materialize_semantic(&frame, &tracked, &word_ids, Some(verb_idx));
        graph.materialize_phrases(tokens, &word_ids);
        graph.attach_concept_layer(&self.lexicon, &self.ontology);

        // Populate full node types (Sentence etc) symmetric to PL for multi-sentence
        let sent_node_id = graph.alloc_node_id();
        graph.nodes.push(GraphNode::Sentence(crate::core::graph::SentenceNode {
            id: sent_node_id,
            frame_id: sentence.frames.last().and_then(|_| None),
        }));
        let utt_node_id = graph.alloc_node_id();
        graph.nodes.push(GraphNode::Utterance(crate::core::graph::UtteranceNode {
            id: utt_node_id,
            sentence_ids: vec![sent_node_id],
            discourse: None,
        }));
        let disc_id = graph.alloc_node_id();
        graph.nodes.push(GraphNode::Discourse(crate::core::graph::DiscourseNode {
            id: disc_id,
            speaker: None,
            addressee: None,
            entities: vec![],
        }));
        if tokens.len() > 0 {
            let cl_id = graph.alloc_node_id();
            graph.nodes.push(GraphNode::Clause(crate::core::graph::ClauseNode {
                id: cl_id,
                sentence_id: Some(sent_node_id),
                words: word_ids.clone(),
            }));
        }

        sentence.graph = Some(graph);
        deduction::apply_graph_inference(&mut sentence, &self.lexicon, &self.ontology)
            .map_err(|_| ParseError::NoVerbFound)?;

        Ok(Utterance::single_sentence(sentence))
    }

    fn pp_span_indices(tokens: &[Token]) -> std::collections::HashSet<usize> {
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
                } else if matches!(t.form.as_str(), "," | "and" | "or") {
                } else {
                    break;
                }
                k += 1;
            }
        }
        span
    }

    fn token_to_entity(&self, token: &Token) -> Entity {
        let surface = &token.form;
        let lookup = surface.to_lowercase();
        let lemma = token.lemma.as_deref().unwrap_or(&lookup);
        let entry = self
            .lexicon
            .lookup_by_form(&lookup)
            .or_else(|| self.lexicon.lookup_by_lemma(lemma));
        let concept = resolve_concept_for_unknown(&self.lexicon, surface, lemma, None, &self.concept_ids, Some("en"));
        // Preserve surface (AC1)
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

    fn extract_pp_entities(&self, tokens: &[Token]) -> Vec<Entity> {
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
                    let is_person_context =
                        self.ontology.is_animate_entity(&ent);
                    if is_with && is_person_context {
                        ent.features.semantic_role = Some(SemanticRole::Location);
                        ent.features.case = Some(Case::Instrumental);
                    } else if let Some(role) = self.descriptor.syntax.preposition_roles.get(&prep) {
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
                } else if matches!(t.form.as_str(), "," | "and" | "or") {
                } else {
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
            } else {
                pp_entities.push(collected.into_iter().next().unwrap());
            }
        }
        pp_entities
    }

    fn try_age_idiom_frame(
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
                .map_or(false, |e| e.concept == "YEAR")
        });
        let has_old = tokens.iter().any(|t| {
            self.lexicon
                .lookup_by_form(&t.form.to_lowercase())
                .map_or(false, |e| e.concept == "OLD")
        });
        let has_number = tokens.iter().any(|t| self.lexicon.cardinal_from_token(t).is_some());
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
            let mut adj = Entity::new(ConceptId::new(&old_entry.concept)).with_name(&old_entry.lemma);
            adj.features = old_entry.features.clone();
            possessed.adjectives.push(adj);
        }
        Some(Frame::Possession {
            possessor,
            possessed,
            verb_concept: "BE".to_string(),
        })
    }

    fn build_frame(
        &self,
        frame_type: &str,
        roles: &[SemanticRole],
        entities: &[Entity],
        verb_concept: &str,
    ) -> Result<Frame, ParseError> {
        Ok(crate::core::frame_builder::build_frame_from_roles(
            frame_type,
            roles,
            entities,
            verb_concept,
            &self.lexicon,
        ))
    }
}

fn parse_role_str(s: &str) -> Option<SemanticRole> {
    crate::core::utils::parse_role_str(s)
}
