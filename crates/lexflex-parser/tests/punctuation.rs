use lexflex_model::LanguageId;
use lexflex_parser::{tokenize, ParseInput};

#[test]
fn question_mark_sets_interrogative_mode() {
    let input = ParseInput {
        source_id: "test:punctuation".into(),
        language: LanguageId::new("en").expect("language"),
        text: "What is the capital of France?".into(),
    };
    let tokenization = tokenize(&input).expect("tokenize");
    assert!(matches!(
        tokenization.mode,
        lexflex_parser::ClauseMode::Interrogative
    ));
}
