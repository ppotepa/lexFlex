use lexflex::core::interlingua::{
    Aspect, ConceptId, Entity, Frame, Illocution, Polarity, Reference, Sentence, Tense,
    Utterance,
};
use lexflex_document_benchmark::canonical::canonicalize_utterance;
use lexflex_document_benchmark::metrics::{build_metrics, MetricInputs};
use lexflex_document_benchmark::model::{CanonicalDocumentSemantics, DocumentMetrics};

fn entity(name: &str, concept: &str, reference: Reference) -> Entity {
    let mut entity = Entity::new(ConceptId::new(concept));
    entity.name = Some(name.into());
    entity.reference = reference;
    entity
}

fn semantics(frames: Vec<Frame>, polarity: Polarity) -> CanonicalDocumentSemantics {
    let mut sentence = Sentence::new();
    sentence.frames = frames;
    sentence.polarity = polarity;
    sentence.tense = Some(Tense::Past);
    sentence.aspect = Some(Aspect::Perfective);
    sentence.illocution = Illocution::Statement;
    canonicalize_utterance(&Utterance::single_sentence(sentence))
}

fn multi_sentence_semantics() -> CanonicalDocumentSemantics {
    let mut first = Sentence::new();
    first.frames = vec![Frame::Motion {
        mover: entity("Tomek", "PERSON", Reference::Direct),
        source: None,
        goal: None,
        path: None,
        verb_concept: "GO".into(),
    }];
    first.polarity = Polarity::Positive;
    first.tense = Some(Tense::Past);
    first.aspect = Some(Aspect::Perfective);
    first.illocution = Illocution::Statement;

    let mut second = Sentence::new();
    second.frames = vec![Frame::Communication {
        speaker: entity("Anna", "PERSON", Reference::Direct),
        addressee: None,
        message: entity("raport", "MESSAGE", Reference::Direct),
        verb_concept: "SAY".into(),
    }];
    second.polarity = Polarity::Positive;
    second.tense = Some(Tense::Past);
    second.aspect = Some(Aspect::Perfective);
    second.illocution = Illocution::Statement;

    canonicalize_utterance(&Utterance {
        sentences: vec![first, second],
        discourse: None,
        utterance_node_id: None,
    })
}

fn build(
    source: &str,
    output: Option<&str>,
    source_semantics: Option<&CanonicalDocumentSemantics>,
    target_semantics: Option<&CanonicalDocumentSemantics>,
    translation_error: bool,
) -> DocumentMetrics {
    let references = vec!["Tomek went home.".to_string()];
    build_metrics(MetricInputs {
        source,
        output,
        source_semantics,
        target_semantics,
        deterministic: true,
        translation_error,
        references: &references,
        glossary_results: &[],
        expectation_results: &[],
        runtime_ms: 7,
    })
}

#[test]
fn translation_success_requires_non_empty_output_and_no_error() {
    let empty_error = build("", Some(""), None, None, true);
    assert_eq!(empty_error.technical["translation_success"].value, Some(0.0));
    let empty_ok = build("", Some(""), None, None, false);
    assert_eq!(empty_ok.technical["output_non_empty"].value, Some(0.0));
    assert_eq!(empty_ok.technical["translation_success"].value, Some(0.0));
    let non_empty = build("A. B.", Some("A. B."), None, None, false);
    assert_eq!(non_empty.technical["translation_success"].value, Some(1.0));
}

#[test]
fn parse_target_success_is_zero_without_target_semantics() {
    let metrics = build("A. B.", Some("A. B."), None, None, false);
    assert_eq!(metrics.technical["parse_target_success"].value, Some(0.0));
}

#[test]
fn structure_metrics_reflect_paragraph_retention_and_loss() {
    let source = "A.\n\nB.";
    let preserved = build(source, Some(source), None, None, false);
    assert_eq!(preserved.structure["paragraph_preservation"].value, Some(1.0));
    let flattened = build(source, Some("A. B."), None, None, false);
    assert!(flattened.structure["paragraph_preservation"].value.unwrap() < 1.0);
}

#[test]
fn sentence_retention_and_semantics_drop_when_text_is_lost() {
    let source_semantics = multi_sentence_semantics();
    let target_semantics = semantics(
        vec![Frame::Motion {
            mover: entity("Tomek", "PERSON", Reference::Direct),
            source: None,
            goal: None,
            path: None,
            verb_concept: "GO".into(),
        }],
        Polarity::Negative,
    );
    let metrics = build(
        "A. B.",
        Some("A."),
        Some(&source_semantics),
        Some(&target_semantics),
        false,
    );
    assert!(metrics.structure["sentence_retention"].value.unwrap() < 1.0);
    assert!(metrics.semantics["polarity_preservation"].value.unwrap() < 1.0);
    assert!(metrics.semantics["frame_recall"].value.unwrap() < 1.0);
    assert!(metrics.semantics["role_recall"].value.unwrap() < 1.0);
}

#[test]
fn zero_denominator_never_becomes_nan() {
    let metrics = build("", Some(""), None, None, false);
    for map in [
        &metrics.technical,
        &metrics.structure,
        &metrics.semantics,
        &metrics.glossary,
        &metrics.expectations,
        &metrics.reference,
    ] {
        for metric in map.values() {
            if let Some(value) = metric.value {
                assert!(!value.is_nan());
            }
            if let Some(value) = metric.numerator {
                assert!(!value.is_nan());
            }
            if let Some(value) = metric.denominator {
                assert!(!value.is_nan());
            }
        }
    }
}

#[test]
fn metric_maps_are_deterministic() {
    let a = build("A. B.", Some("A. B."), None, None, false);
    let b = build("A. B.", Some("A. B."), None, None, false);
    assert_eq!(a, b);
}
