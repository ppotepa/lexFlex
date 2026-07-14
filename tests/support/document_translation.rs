use lexflex::document::translation::{
    DocumentSentenceGenerator, SentenceGenerationError, SentenceGenerationInput,
};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

pub struct PrefixGenerator {
    pub prefix: String,
}

impl DocumentSentenceGenerator for PrefixGenerator {
    fn generator_id(&self) -> &'static str {
        "prefix-generator"
    }

    fn generate(&self, input: SentenceGenerationInput<'_>) -> Result<String, SentenceGenerationError> {
        Ok(format!("{}{}", self.prefix, input.sentence_id))
    }
}

pub struct SelectiveFailureGenerator {
    pub fail_concepts: BTreeSet<String>,
}

impl DocumentSentenceGenerator for SelectiveFailureGenerator {
    fn generator_id(&self) -> &'static str {
        "selective-failure-generator"
    }

    fn generate(&self, input: SentenceGenerationInput<'_>) -> Result<String, SentenceGenerationError> {
        if self.fail_concepts.contains(input.sentence_id.as_str()) {
            return Err(SentenceGenerationError {
                sentence_id: input.sentence_id.clone(),
                code: "TEST_GENERATION_FAILURE".to_string(),
                message: "intentional generation failure".to_string(),
                cause: None,
            });
        }
        Ok(format!("generated-{}", input.sentence_id))
    }
}

pub struct EmptyGenerator;

impl DocumentSentenceGenerator for EmptyGenerator {
    fn generator_id(&self) -> &'static str {
        "empty-generator"
    }

    fn generate(&self, _input: SentenceGenerationInput<'_>) -> Result<String, SentenceGenerationError> {
        Ok(String::new())
    }
}

pub struct RecordingGenerator {
    pub calls: Arc<Mutex<Vec<String>>>,
}

impl DocumentSentenceGenerator for RecordingGenerator {
    fn generator_id(&self) -> &'static str {
        "recording-generator"
    }

    fn generate(&self, input: SentenceGenerationInput<'_>) -> Result<String, SentenceGenerationError> {
        self.calls.lock().unwrap().push(input.sentence_id.as_str().to_string());
        Ok(format!("generated-{}", input.sentence_id))
    }
}

pub fn recording_generator() -> (RecordingGenerator, Arc<Mutex<Vec<String>>>) {
    let calls = Arc::new(Mutex::new(Vec::new()));
    (
        RecordingGenerator {
            calls: calls.clone(),
        },
        calls,
    )
}
