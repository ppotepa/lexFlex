use lexflex::api::LexFlexAPI;
use lexflex::core::interlingua::{Interlingua, QueryProjection, QuestionKind, QueryTerm};

fn api() -> LexFlexAPI {
    LexFlexAPI::builder().data_dir("data").build().expect("API should build")
}

#[test]
fn english_definition_question_has_language_neutral_semantics() {
    let il = api().parse("What is Paris?", "en").expect("question should parse");
    let Interlingua::Natural(utterance) = il else { panic!("expected natural interlingua") };
    let question = utterance.sentences[0].question.as_ref().expect("question semantics");
    assert_eq!(question.kind, QuestionKind::What);
    assert_eq!(question.proposition.predicate.0, "IS_A");
    assert_eq!(question.projection, QueryProjection::Concept);
    assert!(matches!(question.proposition.object, Some(QueryTerm::Variable(_))));
}

#[test]
fn polish_definition_question_has_same_semantic_shape() {
    let il = api().parse("Co to jest Paryż?", "pl").expect("question should parse");
    let Interlingua::Natural(utterance) = il else { panic!("expected natural interlingua") };
    let question = utterance.sentences[0].question.as_ref().expect("question semantics");
    assert_eq!(question.kind, QuestionKind::What);
    assert_eq!(question.proposition.predicate.0, "IS_A");
    assert_eq!(question.projection, QueryProjection::Concept);
}
