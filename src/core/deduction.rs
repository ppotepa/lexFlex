use crate::core::interlingua::*;
use crate::core::ontology::Ontology;
use crate::core::temporal;
use crate::data::lexicon::Lexicon;
use crate::error::DeductionError;

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
    let person_match = match (pronoun.features.person, candidate.features.person) {
        (Some(pp), Some(cp)) => pp == cp,
        (None, _) | (_, None) => true,
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
