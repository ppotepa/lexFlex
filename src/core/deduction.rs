use crate::core::constructions::{
    self, ACCOMPANIMENT, AGE_IDIOM, DEMONSTRATIVE_REFERENCE, IDENTIFICATION,
};
use crate::core::graph::{self, EdgeKind};
use crate::core::interlingua::*;
use crate::core::ontology::Ontology;
use crate::core::temporal;
use crate::data::lexicon::Lexicon;
use crate::error::DeductionError;
use crate::core::unknown_concept::resolve_concept_for_unknown; // resolve real ConceptId for unknowns (A+B of unknown concept resolution)

pub struct DeductionContext<'a> {
    pub lexicon: &'a Lexicon,
    pub ontology: &'a Ontology,
    pub language: LanguageId,
}

impl<'a> DeductionContext<'a> {
    pub fn new(lexicon: &'a Lexicon, ontology: &'a Ontology, language: LanguageId) -> Self {
        Self { lexicon, ontology, language }
    }
}

pub fn deduce(
    mut utterance: Utterance,
    context: &DeductionContext,
) -> Result<Utterance, DeductionError> {
    for sentence in &mut utterance.sentences {
        apply_verb_frames(sentence, context)?;
        resolve_cases_and_roles(sentence)?;
        resolve_pronouns_within_sentence(sentence)?;
        anchor_temporals(sentence)?;
        validate_and_inherit_from_ontology(sentence, context.ontology)?;
        normalize_features(sentence)?;
    }
    Ok(utterance)
}

/// Graph-native inference pass (runs after graph materialization in parsers).
pub fn apply_graph_inference(
    sentence: &mut Sentence,
    _lexicon: &Lexicon,
    ontology: &Ontology,
) -> Result<(), DeductionError> {
    // Fix any bad/unknown concepts on entities using the resolver in core deduction path.
    // This ensures even entities that reach graph inference get a real data ConceptId (baked/ron list).
    for frame in sentence.frames.iter_mut() {
        for entity in frame.entities_mut() {
            let c = &entity.concept.0;
            if c == "unknown" || c.is_empty() || c == "UNKNOWN" {
                let resolved = resolve_concept_for_unknown(_lexicon, entity.name.as_deref().unwrap_or(c), c, None, &[], None);
                entity.concept = resolved;
            }
        }
    }
    let Some(ref mut graph) = sentence.graph else {
        return Ok(());
    };
    graph.register_default_constructions();

    // Accompaniment paths → instrumental case + Location semantic role on IL entities
    let accomp_paths = graph.find_accompaniment_paths();
    for frame in &mut sentence.frames {
        if let Frame::Existence { location, .. } = frame {
            if let Some(loc) = location {
                if let Some(eid) = graph::find_entity_node_id(graph, loc) {
                    if graph.location_uses_instrumental(eid) || !accomp_paths.is_empty() {
                        loc.features.case = Some(Case::Instrumental);
                        loc.features.semantic_role = Some(SemanticRole::Location);
                    }
                }
            }
        }
    }

    // Concept-driven: entities whose concept is related to LIVE as typical accompaniment
    let live_related = graph.concept_related("LIVE", "typical_accompaniment");
    if !live_related.is_empty() {
        for frame in &mut sentence.frames {
            for entity in frame.entities_mut() {
                if live_related.iter().any(|c| c == &entity.concept) {
                    entity.features.semantic_role = Some(SemanticRole::Location);
                }
            }
        }
    }

    // Age idiom: YEAR + numerical quantification → AgeIdiom construction on graph
    if matches!(sentence.quantification, Some(Quantifier::Numerical(_))) {
        for frame in &mut sentence.frames {
            if let Frame::Possession { possessed, .. } = frame {
                if possessed.concept.0 == "YEAR" {
                    if possessed.adjectives.iter().all(|a| a.concept.0 != "OLD") {
                        if let Some(old_entry) = _lexicon
                            .lookup_by_form("old")
                            .or_else(|| _lexicon.lookup_concept("OLD"))
                        {
                            let mut adj = Entity::new(ConceptId::new(&old_entry.concept))
                                .with_name(&old_entry.lemma);
                            adj.features = old_entry.features.clone();
                            possessed.adjectives.push(adj);
                        }
                    }
                    if let Some(eid) = graph::find_entity_node_id(graph, possessed) {
                        constructions::attach_construction(
                            &mut sentence.construction_concepts,
                            graph,
                            eid,
                            AGE_IDIOM,
                        );
                    }
                }
            }
        }
    }

    let demonstrative_copula = graph
        .word_nodes()
        .any(|w| w.form.eq_ignore_ascii_case("to"));

    // Phase 4: attach concept-backed constructions for copula and demonstrative
    for frame in &mut sentence.frames {
        if let Frame::Existence { location, .. } = frame {
            if location.is_none() {
                for e in frame.entities_mut() {
                    if let Some(eid) = graph::find_entity_node_id(graph, e) {
                        constructions::attach_construction(
                            &mut sentence.construction_concepts,
                            graph,
                            eid,
                            IDENTIFICATION,
                        );
                        if demonstrative_copula {
                            constructions::attach_construction(
                                &mut sentence.construction_concepts,
                                graph,
                                eid,
                                DEMONSTRATIVE_REFERENCE,
                            );
                        }
                        break;
                    }
                }
            }
        }
        if let Frame::Statement { subject, property, verb_concept } = frame {
            if verb_concept == "BE" {
                // Identificational copula Statement case (e.g. plural 'To są' produces Statement)
                // attach to the main descriptive entity (the property or the noun one)
                let target = if property.adjectives.len() > 0 || property.concept.0 != "unknown" { property } else { subject };
                if let Some(eid) = graph::find_entity_node_id(graph, target) {
                    constructions::attach_construction(
                        &mut sentence.construction_concepts,
                        graph,
                        eid,
                        IDENTIFICATION,
                    );
                }
            }
        }
    }
    // For demonstrative: look for THIS in adjectives
    for frame in &mut sentence.frames {
        for entity in frame.entities_mut() {
            if entity.adjectives.iter().any(|a| a.concept.0 == "THIS") {
                if let Some(eid) = graph::find_entity_node_id(graph, entity) {
                    constructions::attach_construction(
                        &mut sentence.construction_concepts,
                        graph,
                        eid,
                        DEMONSTRATIVE_REFERENCE,
                    );
                }
            }
        }
    }

    // Accompaniment construction on graph paths
    if !accomp_paths.is_empty() {
        for path in &accomp_paths {
            if let Some(&eid) = path.last() {
                if graph.nodes.get(eid.0 as usize).map_or(false, |n| {
                    matches!(n, crate::core::graph::GraphNode::Entity(_))
                }) {
                    constructions::attach_construction(
                        &mut sentence.construction_concepts,
                        graph,
                        eid,
                        ACCOMPANIMENT,
                    );
                }
            }
        }
    }

    // Ontology inheritance still applies for features not set by graph
    for frame in &mut sentence.frames {
        for entity in frame.entities_mut() {
            ontology.inherit_features(entity);
        }
    }

    constructions::build_construction_tree(sentence);

    Ok(())
}

fn apply_verb_frames(
    sentence: &mut Sentence,
    context: &DeductionContext,
) -> Result<(), DeductionError> {
    if sentence.frames.is_empty() {
        return Ok(());
    }

    for frame in &mut sentence.frames {
        for entity in frame.entities_mut() {
            context.ontology.inherit_features(entity);
        }
        if let Frame::Consumption { patient, verb_concept, .. } = frame {
            let liquid = context.ontology.is_liquid(&patient.concept);
            if liquid && verb_concept.eq_ignore_ascii_case("EAT") {
                *verb_concept = "DRINK".to_string();
            }
        }
    }

    Ok(())
}

fn resolve_cases_and_roles(sentence: &mut Sentence) -> Result<(), DeductionError> {
    // First pass: assign cases based on frame type and polarity
    for frame in &mut sentence.frames {
        match frame {
            Frame::Transfer { agent, recipient, theme, .. } => {
                agent.features.case = Some(Case::Nominative);
                recipient.features.case = Some(Case::Dative);
                if sentence.polarity == Polarity::Negative {
                    theme.features.case = Some(Case::Genitive);
                } else {
                    theme.features.case = Some(Case::Accusative);
                }
            }
            Frame::Motion { mover, source, goal, .. } => {
                mover.features.case = Some(Case::Nominative);
                if let Some(g) = goal {
                    g.features.case = Some(Case::Accusative);
                }
                if let Some(s) = source {
                    s.features.case = Some(Case::Genitive);
                }
            }
            Frame::Perception { experiencer, stimulus, .. } => {
                experiencer.features.case = Some(Case::Nominative);
                stimulus.features.case = Some(Case::Accusative);
            }
            Frame::Cognition { cognizer, content, .. } => {
                cognizer.features.case = Some(Case::Nominative);
                content.features.case = Some(Case::Accusative);
            }
            Frame::Emotion { experiencer, stimulus, .. } => {
                experiencer.features.case = Some(Case::Nominative);
                stimulus.features.case = Some(Case::Accusative);
            }
            Frame::Destruction { agent, patient, .. } => {
                agent.features.case = Some(Case::Nominative);
                if sentence.polarity == Polarity::Negative {
                    patient.features.case = Some(Case::Genitive);
                } else {
                    patient.features.case = Some(Case::Accusative);
                }
            }
            Frame::Consumption { agent, patient, .. } => {
                agent.features.case = Some(Case::Nominative);
                if sentence.polarity == Polarity::Negative {
                    patient.features.case = Some(Case::Genitive);
                } else {
                    patient.features.case = Some(Case::Accusative);
                }
            }
            Frame::Communication { speaker, addressee, message, .. } => {
                speaker.features.case = Some(Case::Nominative);
                if let Some(a) = addressee {
                    a.features.case = Some(Case::Dative);
                }
                message.features.case = Some(Case::Accusative);
            }
            Frame::Creation { creator, created, .. } => {
                creator.features.case = Some(Case::Nominative);
                if sentence.polarity == Polarity::Negative {
                    created.features.case = Some(Case::Genitive);
                } else {
                    created.features.case = Some(Case::Accusative);
                }
            }
            Frame::Statement { subject, property, .. } => {
                subject.features.case = Some(Case::Nominative);
                property.features.case = Some(Case::Nominative);
            }
            Frame::Existence { entity, .. } => {
                entity.features.case = Some(Case::Nominative);
            }
            Frame::Possession { possessor, possessed, .. } => {
                possessor.features.case = Some(Case::Nominative);
                possessed.features.case = Some(Case::Accusative);
            }
            Frame::Custom { .. } => {}
        }
    }

    // Second pass: resolve case ambiguity using ontology and verb frames
    resolve_case_ambiguity(sentence)?;

    Ok(())
}

fn resolve_case_ambiguity(sentence: &mut Sentence) -> Result<(), DeductionError> {
    // For each frame, check if entities have ambiguous cases and resolve them
    for frame in &mut sentence.frames {
        let entities = frame.entities_mut();
        for entity in entities {
            // If entity has no case assigned yet, try to infer it
            if entity.features.case.is_none() {
                // Use ontology to determine likely case
                if let Some(inferred_case) = infer_case_from_ontology(entity) {
                    entity.features.case = Some(inferred_case);
                }
            }
        }
    }
    Ok(())
}

fn infer_case_from_ontology(entity: &Entity) -> Option<Case> {
    // Default case inference based on entity features
    // This is a simplified version - in production, this would use verb frames and ontology
    if entity.features.animacy == Some(Animacy::Animate) {
        // Animate entities are more likely to be agents (Nominative)
        Some(Case::Nominative)
    } else {
        // Inanimate entities are more likely to be themes (Accusative)
        Some(Case::Accusative)
    }
}

fn resolve_pronouns_within_sentence(sentence: &mut Sentence) -> Result<(), DeductionError> {
    for frame in &mut sentence.frames {
        // Collect all entities in the frame for antecedent lookup
        let all_entities: Vec<Entity> = frame.entities().iter().map(|e| (*e).clone()).collect();
        
        // Find the subject/agent for reflexive binding
        let subject = match frame {
            Frame::Transfer { agent, .. } => Some(agent.clone()),
            Frame::Motion { mover, .. } => Some(mover.clone()),
            Frame::Perception { experiencer, .. } => Some(experiencer.clone()),
            Frame::Cognition { cognizer, .. } => Some(cognizer.clone()),
            Frame::Emotion { experiencer, .. } => Some(experiencer.clone()),
            Frame::Destruction { agent, .. } => Some(agent.clone()),
            Frame::Consumption { agent, .. } => Some(agent.clone()),
            Frame::Communication { speaker, .. } => Some(speaker.clone()),
            Frame::Creation { creator, .. } => Some(creator.clone()),
            Frame::Statement { subject, .. } => Some(subject.clone()),
            Frame::Existence { entity, .. } => Some(entity.clone()),
            Frame::Possession { possessor, .. } => Some(possessor.clone()),
            Frame::Custom { .. } => None,
        };

        // Resolve pronouns in each entity
        for entity in frame.entities_mut() {
            resolve_pronoun_entity(entity, &subject, &all_entities)?;
        }
    }
    Ok(())
}

fn resolve_pronoun_entity(
    entity: &mut Entity,
    subject: &Option<Entity>,
    all_entities: &[Entity],
) -> Result<(), DeductionError> {
    let name = entity.name.as_deref().unwrap_or("");
    let concept = &entity.concept.0;
    
    // Check if this is a reflexive pronoun
    let is_reflexive = is_reflexive_pronoun(name, concept);
    if is_reflexive {
        if let Some(ref subj) = subject {
            // Bind reflexive to subject
            entity.concept = subj.concept.clone();
            entity.name = subj.name.clone();
            entity.features.gender = subj.features.gender;
            entity.features.number = subj.features.number;
            entity.features.person = subj.features.person;
            entity.reference = Reference::Anaphoric(subj.concept.0.clone());
        }
        return Ok(());
    }
    
    // Check if this is a personal pronoun
    let is_pronoun = is_personal_pronoun(name, concept);
    if is_pronoun {
        // Try to find antecedent by matching gender/number
        if let Some(antecedent) = find_antecedent(entity, all_entities, subject) {
            entity.concept = antecedent.concept.clone();
            if let Some(ref name) = antecedent.name {
                entity.name = Some(name.clone());
            }
            entity.reference = Reference::Anaphoric(antecedent.concept.0.clone());
        }
    }
    
    Ok(())
}

fn is_reflexive_pronoun(name: &str, concept: &str) -> bool {
    let name_lower = name.to_lowercase();
    matches!(
        name_lower.as_str(),
        "się" | "sobie" | "siebie" | "myself" | "yourself" | "himself" 
        | "herself" | "itself" | "ourselves" | "yourselves" | "themselves"
    ) || concept == "REFLEXIVE"
}

fn is_personal_pronoun(name: &str, concept: &str) -> bool {
    let name_lower = name.to_lowercase();
    // Skip dummy subjects used for existential/copular sentences (e.g., "it is bright")
    if concept == "DUMMY_SUBJECT" { return false; }
    matches!(
        name_lower.as_str(),
        "on" | "ona" | "ono" | "oni" | "one" | "ja" | "ty" | "my" | "wy"
        | "i" | "you" | "he" | "she" | "it" | "we" | "they" | "me" | "him" 
        | "her" | "us" | "them"
    ) || concept == "PRONOUN"
}

fn find_antecedent(
    pronoun: &Entity,
    all_entities: &[Entity],
    subject: &Option<Entity>,
) -> Option<Entity> {
    // For personal pronouns, prefer subject as antecedent if features match
    if let Some(ref subj) = subject {
        if features_match(pronoun, subj) && !is_pronoun_entity(subj) {
            return Some(subj.clone());
        }
    }
    
    // Otherwise, find the most recent matching entity
    for entity in all_entities.iter().rev() {
        if !is_pronoun_entity(entity) && features_match(pronoun, entity) {
            return Some(entity.clone());
        }
    }
    
    None
}

fn features_match(pronoun: &Entity, candidate: &Entity) -> bool {
    // Check gender compatibility
    let gender_match = match (pronoun.features.gender, candidate.features.gender) {
        (Some(pg), Some(cg)) => pg == cg,
        (None, _) | (_, None) => true, // If pronoun has no gender specified, any candidate works
    };

    // Check number compatibility
    let number_match = match (pronoun.features.number, candidate.features.number) {
        (Some(pn), Some(cn)) => pn == cn,
        (None, _) | (_, None) => true,
    };

    // Check person compatibility (3rd person pronouns refer to 3rd person entities)
    // 1st/2nd person pronouns should NOT match 3rd person entities
    let person_match = match (pronoun.features.person, candidate.features.person) {
        (Some(Person::First), Some(Person::First)) => true,
        (Some(Person::Second), Some(Person::Second)) => true,
        (Some(Person::Third), Some(Person::Third)) => true,
        (Some(Person::First), _) | (Some(Person::Second), _) => false, // 1st/2nd person pronouns don't refer to 3rd person entities
        (None, _) | (_, None) => true,
        _ => true,
    };

    gender_match && number_match && person_match
}

fn is_pronoun_entity(entity: &Entity) -> bool {
    let name = entity.name.as_deref().unwrap_or("");
    let concept = &entity.concept.0;
    is_personal_pronoun(name, concept) || is_reflexive_pronoun(name, concept)
}

fn anchor_temporals(sentence: &mut Sentence) -> Result<(), DeductionError> {
    if let Some(TemporalReference::Deictic { ref word }) = sentence.temporal {
        if let Some(resolved) = temporal::resolve_deictic(word) {
            sentence.temporal = Some(resolved);
        }
    }
    Ok(())
}

fn validate_and_inherit_from_ontology(
    sentence: &Sentence,
    ontology: &Ontology,
) -> Result<(), DeductionError> {
    for frame in &sentence.frames {
        ontology.validate_semantic_types(frame)?;
    }
    Ok(())
}

fn normalize_features(sentence: &mut Sentence) -> Result<(), DeductionError> {
    // Normalize tense/aspect combinations
    // Ensure consistent feature representation across the sentence
    
    // If sentence has tense but no aspect, default to imperfective for ongoing actions
    if sentence.tense.is_some() && sentence.aspect.is_none() {
        sentence.aspect = Some(Aspect::Imperfective);
    }
    
    // If sentence has aspect but no tense, default to present
    if sentence.aspect.is_some() && sentence.tense.is_none() {
        sentence.tense = Some(Tense::Present);
    }
    
    // Normalize modality - default to realis (factual/declarative) if not specified
    if sentence.modality.is_none() {
        sentence.modality = Some(Modality::Realis);
    }
    
    Ok(())
}

pub fn build_frame_from_concept(
    concept_id: &str,
    lexicon: &Lexicon,
) -> Option<(String, Vec<SemanticRole>)> {
    let entry = lexicon.entries.get(concept_id)?;
    let frame_type = entry.frame_type.clone()?;
    let roles: Vec<SemanticRole> = entry
        .roles
        .iter()
        .filter_map(|r| parse_role(r))
        .collect();
    Some((frame_type, roles))
}

fn parse_role(s: &str) -> Option<SemanticRole> {
    crate::core::utils::parse_role_str(s)
}
