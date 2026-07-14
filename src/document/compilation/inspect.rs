use crate::core::interlingua::{Entity, Polarity, Reference, Utterance};
use crate::document::compilation::model::SentenceSemanticInspection;

pub struct SemanticInspector;

impl SemanticInspector {
    pub fn inspect(utterance: &Utterance) -> SentenceSemanticInspection {
        let mut inspection = SentenceSemanticInspection {
            utterance_sentence_count: utterance.sentences.len(),
            ..Default::default()
        };
        for sentence in &utterance.sentences {
            for frame in &sentence.frames {
                inspection.frame_count += 1;
                for entity in frame.entities() {
                    inspect_entity(entity, &mut inspection);
                }
            }
            match sentence.polarity {
                Polarity::Positive => inspection.has_positive_polarity = true,
                Polarity::Negative => inspection.has_negative_polarity = true,
            }
            if sentence.temporal.is_some() {
                inspection.has_temporal_information = true;
            }
            if sentence.quantification.is_some() {
                inspection.has_quantification = true;
            }
        }
        if inspection.utterance_sentence_count == 0 {
            inspection.reasons.push("empty utterance".to_string());
        }
        if inspection.frame_count == 0 {
            inspection.reasons.push("no semantic frames".to_string());
        }
        if inspection.unresolved_reference_count > 0 {
            inspection
                .reasons
                .push(format!("{} unresolved references", inspection.unresolved_reference_count));
        }
        if inspection.deferred_reference_count > 0 {
            inspection
                .reasons
                .push(format!("{} deferred references", inspection.deferred_reference_count));
        }
        if inspection.empty_concept_count > 0 {
            inspection
                .reasons
                .push(format!("{} empty concepts", inspection.empty_concept_count));
        }
        inspection.reasons.sort();
        inspection.reasons.dedup();
        inspection
    }
}

fn inspect_entity(entity: &Entity, inspection: &mut SentenceSemanticInspection) {
    inspection.entity_count += 1;
    if entity.concept.0.trim().is_empty() {
        inspection.empty_concept_count += 1;
    }
    match entity.name.as_deref() {
        Some(name) if !name.trim().is_empty() => inspection.named_entity_count += 1,
        Some(_) => inspection.empty_name_count += 1,
        None => {}
    }
    match &entity.reference {
        Reference::Direct => inspection.direct_reference_count += 1,
        Reference::Unresolved => inspection.unresolved_reference_count += 1,
        Reference::Anaphoric(_) | Reference::Cataphoric(_) | Reference::Deictic => {
            inspection.deferred_reference_count += 1
        }
        Reference::Generic => {}
    }

    if let Some(coordination) = &entity.coordination {
        for item in &coordination.items {
            inspect_entity(item, inspection);
        }
    }

    for adjective in &entity.adjectives {
        inspect_entity(adjective, inspection);
    }
}
