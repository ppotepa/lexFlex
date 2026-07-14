use lexflex::core::interlingua::{
    Aspect, ConceptId, Entity, Frame, Illocution, Polarity, Reference, Sentence, Tense,
    Utterance,
};
use lexflex_document_benchmark::canonical::canonicalize_utterance;

fn entity(name: &str, concept: &str, reference: Reference) -> Entity {
    let mut entity = Entity::new(ConceptId::new(concept));
    entity.name = Some(name.into());
    entity.reference = reference;
    entity
}

#[test]
fn canonicalization_preserves_frames_roles_and_constructions() {
    let mut first = Sentence::new();
    first.tense = Some(Tense::Past);
    first.aspect = Some(Aspect::Perfective);
    first.polarity = Polarity::Positive;
    first.frames = vec![Frame::Motion {
        mover: entity("Tomek", "PERSON", Reference::Direct),
        source: Some(entity("domu", "HOME", Reference::Direct)),
        goal: Some(entity("sklepu", "STORE", Reference::Direct)),
        path: None,
        verb_concept: "GO".into(),
    }];
    first.construction_concepts = vec![ConceptId::new("B"), ConceptId::new("A"), ConceptId::new("A")];

    let mut second = Sentence::new();
    second.polarity = Polarity::Negative;
    second.illocution = Illocution::Statement;
    second.frames = vec![Frame::Communication {
        speaker: entity("Anna", "PERSON", Reference::Anaphoric("Anna".into())),
        addressee: Some(entity("Iza", "PERSON", Reference::Direct)),
        message: entity("raport", "MESSAGE", Reference::Unresolved),
        verb_concept: "SAY".into(),
    }];

    let utterance = Utterance {
        sentences: vec![first, second],
        discourse: None,
        utterance_node_id: None,
    };

    let semantics = canonicalize_utterance(&utterance);

    assert_eq!(semantics.sentences.len(), 2);
    assert_eq!(semantics.sentences[0].frames.len(), 1);
    assert_eq!(semantics.sentences[0].frames[0].frame_type, "Motion");
    assert!(semantics.sentences[0].frames[0]
        .roles
        .iter()
        .any(|role| role.role == "Agent" && role.concept == "PERSON"));
    assert_eq!(semantics.sentences[1].polarity, "Negative");
    assert_eq!(semantics.sentences[0].constructions, vec!["A", "B"]);
    assert_eq!(semantics.unresolved_count, 1);
    assert_eq!(serde_json::to_string(&semantics).unwrap(), serde_json::to_string(&semantics).unwrap());
}
