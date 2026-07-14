use lexflex::api::LexFlexAPI;
use lexflex::chat::semantic::SemanticConversationMemory;

fn memory() -> SemanticConversationMemory {
    let api = LexFlexAPI::builder()
        .data_dir("data")
        .build()
        .expect("API should build");
    SemanticConversationMemory::new(api)
}

#[test]
fn semantic_memory_answers_definition_question_in_english() {
    let mut memory = memory();
    assert_eq!(memory.observe("Paris is a city.", "en").unwrap(), 1);
    let answer = memory.answer("What is Paris?", "en").unwrap().expect("answer");
    assert!(answer.text.to_lowercase().contains("city"));
    assert_eq!(answer.evidence.len(), 1);
}

#[test]
fn semantic_memory_answers_definition_question_in_polish() {
    let mut memory = memory();
    assert_eq!(memory.observe("Paryż jest miastem.", "pl").unwrap(), 1);
    let answer = memory.answer("Co to jest Paryż?", "pl").unwrap().expect("answer");
    assert!(answer.text.to_lowercase().contains("miasto"));
}

#[test]
fn semantic_memory_answers_relation_question_in_english() {
    let mut memory = memory();
    assert_eq!(memory.observe("Paris is the capital of France.", "en").unwrap(), 1);
    let answer = memory
        .answer("What is the capital of France?", "en")
        .unwrap()
        .expect("answer");
    assert_eq!(answer.text, "Paris");
    assert_eq!(answer.evidence.len(), 1);
}

#[test]
fn semantic_memory_answers_relation_question_in_polish() {
    let mut memory = memory();
    assert_eq!(memory.observe("Paryż jest stolicą Francji.", "pl").unwrap(), 1);
    let answer = memory
        .answer("Jaka jest stolica Francji?", "pl")
        .unwrap()
        .expect("answer");
    assert_eq!(answer.text, "Paryż");
}

#[test]
fn semantic_trace_exposes_interlingua_question_and_frames() {
    let memory = memory();
    let trace = memory
        .inspect("What is the capital of France?", "en")
        .expect("trace");
    assert_eq!(trace.parse_status, "ok");
    assert_eq!(trace.sentence_count, 1);
    assert!(trace.sentences[0].contains("frames="));
    assert!(trace.question.expect("question").contains("CAPITAL_OF"));
}
