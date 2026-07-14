use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use lexflex::core::interlingua::{
    ConceptId, Entity, Frame, LanguageId, Reference, Sentence, Utterance,
};
use lexflex::document::compilation::{
    DocumentSentenceAnalyzer, SentenceAnalysisError, SentenceAnalysisInput,
};
use lexflex::document::{
    Document, DocumentInput, DocumentSegmentationOptions, DocumentSegmenter,
    LosslessDocumentSegmenter,
};

pub mod document_corpus;
pub mod document_translation;

pub use document_corpus::*;
pub use document_translation::*;


pub fn segmented(source: &str) -> Document {
    LosslessDocumentSegmenter
        .segment(
            DocumentInput {
                id: None,
                source_language: LanguageId::new("pl"),
                source: source.to_string(),
            },
            &DocumentSegmentationOptions::default(),
        )
        .expect("document segmentation")
}

pub fn simple_resolved_utterance() -> Utterance {
    let mut sentence = Sentence::new();
    sentence.frames.push(Frame::Motion {
        mover: Entity::new(ConceptId::new("PERSON")).with_name("Ala"),
        source: None,
        goal: None,
        path: None,
        verb_concept: "GO".into(),
    });
    Utterance::single_sentence(sentence)
}

pub fn partial_utterance() -> Utterance {
    let mut sentence = Sentence::new();
    let mut mover = Entity::new(ConceptId::new("PERSON")).with_name("Ala");
    mover.reference = Reference::Unresolved;
    sentence.frames.push(Frame::Motion {
        mover,
        source: None,
        goal: None,
        path: None,
        verb_concept: "GO".into(),
    });
    Utterance::single_sentence(sentence)
}

pub struct AlwaysSuccessAnalyzer;

impl DocumentSentenceAnalyzer for AlwaysSuccessAnalyzer {
    fn analyzer_id(&self) -> &'static str {
        "always-success"
    }

    fn analyze(
        &self,
        _input: SentenceAnalysisInput<'_>,
    ) -> Result<Utterance, SentenceAnalysisError> {
        Ok(simple_resolved_utterance())
    }
}

pub struct SelectiveFailureAnalyzer {
    pub fail_ordinals: BTreeSet<usize>,
}

impl DocumentSentenceAnalyzer for SelectiveFailureAnalyzer {
    fn analyzer_id(&self) -> &'static str {
        "selective-failure"
    }

    fn analyze(
        &self,
        input: SentenceAnalysisInput<'_>,
    ) -> Result<Utterance, SentenceAnalysisError> {
        if self.fail_ordinals.contains(&input.sentence.document_ordinal) {
            return Err(SentenceAnalysisError {
                sentence_id: input.sentence.id.clone(),
                code: "TEST_ANALYSIS_FAILURE".into(),
                message: "intentional failure".into(),
                cause: None,
            });
        }
        Ok(simple_resolved_utterance())
    }
}

pub struct PanicAnalyzer;

impl DocumentSentenceAnalyzer for PanicAnalyzer {
    fn analyzer_id(&self) -> &'static str {
        "panic"
    }

    fn analyze(
        &self,
        _input: SentenceAnalysisInput<'_>,
    ) -> Result<Utterance, SentenceAnalysisError> {
        panic!("intentional analyzer panic")
    }
}

pub struct UnresolvedAnalyzer;

impl DocumentSentenceAnalyzer for UnresolvedAnalyzer {
    fn analyzer_id(&self) -> &'static str {
        "unresolved"
    }

    fn analyze(
        &self,
        _input: SentenceAnalysisInput<'_>,
    ) -> Result<Utterance, SentenceAnalysisError> {
        Ok(Utterance::new())
    }
}

pub struct PartialAnalyzer;

impl DocumentSentenceAnalyzer for PartialAnalyzer {
    fn analyzer_id(&self) -> &'static str {
        "partial"
    }

    fn analyze(
        &self,
        _input: SentenceAnalysisInput<'_>,
    ) -> Result<Utterance, SentenceAnalysisError> {
        Ok(partial_utterance())
    }
}

pub struct RecordingAnalyzer {
    pub texts: Arc<Mutex<Vec<String>>>,
}

impl DocumentSentenceAnalyzer for RecordingAnalyzer {
    fn analyzer_id(&self) -> &'static str {
        "recording"
    }

    fn analyze(
        &self,
        input: SentenceAnalysisInput<'_>,
    ) -> Result<Utterance, SentenceAnalysisError> {
        self.texts.lock().unwrap().push(input.text.to_string());
        Ok(simple_resolved_utterance())
    }
}

pub fn recording_analyzer() -> (RecordingAnalyzer, Arc<Mutex<Vec<String>>>) {
    let texts = Arc::new(Mutex::new(Vec::new()));
    (
        RecordingAnalyzer {
            texts: texts.clone(),
        },
        texts,
    )
}
