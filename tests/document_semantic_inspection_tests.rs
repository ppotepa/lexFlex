mod support;

use lexflex::core::interlingua::{
    ConceptId, Coordination, Entity, Frame, Polarity, Quantifier, Reference, Sentence,
    TemporalAnchor, TemporalReference, Utterance,
};
use lexflex::document::compilation::SemanticInspector;

fn inspect(utterance: Utterance) -> lexflex::document::compilation::SentenceSemanticInspection {
    SemanticInspector::inspect(&utterance)
}

#[test]
fn empty_utterance_reports_reasons() {
    let inspection = inspect(Utterance::new());
    assert_eq!(inspection.utterance_sentence_count, 0);
    assert_eq!(inspection.frame_count, 0);
    assert!(inspection.reasons.iter().any(|r| r == "empty utterance"));
}

#[test]
fn coordination_and_adjectives_are_counted_recursively() {
    let mut head = Entity::new(ConceptId::new("PERSON")).with_name("Ala");
    head.coordination = Some(Coordination {
        items: vec![
            Entity::new(ConceptId::new("PERSON")).with_name("Jan"),
            Entity::new(ConceptId::new("PERSON")).with_name("Ola"),
        ],
        conjunction: "i".into(),
    });
    head.adjectives = vec![Entity::new(ConceptId::new("ADJ")).with_name("miła")];

    let mut sentence = Sentence::new();
    sentence.frames.push(Frame::Motion {
        mover: head,
        source: None,
        goal: None,
        path: None,
        verb_concept: "GO".into(),
    });
    let inspection = inspect(Utterance::single_sentence(sentence));
    assert_eq!(inspection.frame_count, 1);
    assert_eq!(inspection.entity_count, 4);
    assert_eq!(inspection.named_entity_count, 4);
}

#[test]
fn reference_kinds_are_counted() {
    let direct = Entity::new(ConceptId::new("PERSON")).with_name("Ala");
    let mut unresolved = Entity::new(ConceptId::new("PERSON")).with_name("Ola");
    unresolved.reference = Reference::Unresolved;
    let mut anaphoric = Entity::new(ConceptId::new("PERSON")).with_name("Ela");
    anaphoric.reference = Reference::Anaphoric("Ala".into());
    let mut cataphoric = Entity::new(ConceptId::new("PERSON")).with_name("Iza");
    cataphoric.reference = Reference::Cataphoric("Ala".into());
    let mut deictic = Entity::new(ConceptId::new("PERSON")).with_name("To");
    deictic.reference = Reference::Deictic;
    let mut generic = Entity::new(ConceptId::new("PERSON"));
    generic.reference = Reference::Generic;
    let mut sentence = Sentence::new();
    sentence.frames.push(Frame::Transfer {
        agent: direct,
        recipient: unresolved,
        theme: anaphoric,
        verb_concept: "GIVE".into(),
    });
    sentence.frames.push(Frame::Motion {
        mover: cataphoric,
        source: Some(deictic),
        goal: Some(generic),
        path: None,
        verb_concept: "MOVE".into(),
    });
    let inspection = inspect(Utterance::single_sentence(sentence));
    assert_eq!(inspection.direct_reference_count, 1);
    assert_eq!(inspection.unresolved_reference_count, 1);
    assert_eq!(inspection.deferred_reference_count, 3);
}

#[test]
fn polarity_temporal_and_quantification_are_seen() {
    let mut sentence = Sentence::new();
    sentence.polarity = Polarity::Negative;
    sentence.temporal = Some(TemporalReference::Relative {
        offset_days: 1,
        anchor: TemporalAnchor::Now,
    });
    sentence.quantification = Some(Quantifier::Existential);
    sentence.frames.push(Frame::Existence {
        entity: Entity::new(ConceptId::new("THING")),
        location: None,
        verb_concept: "BE".into(),
    });
    let inspection = inspect(Utterance::single_sentence(sentence));
    assert!(inspection.has_negative_polarity);
    assert!(inspection.has_temporal_information);
    assert!(inspection.has_quantification);
}

