use crate::document::compilation::{
    ProvenanceOperation, ProvenanceOutcome, SentenceCompilation, SentenceProvenance,
    SentenceProvenanceBuilder,
};
use crate::document::DocumentSentence;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationFallbackKind {
    CopySource,
    Placeholder,
}

pub fn generation_success_provenance(
    compilation: &SentenceCompilation,
    sentence: &DocumentSentence,
    generator_id: &str,
) -> SentenceProvenance {
    let mut builder = SentenceProvenanceBuilder::from_compilation_for_generation(
        &compilation.provenance,
        generator_id,
    );
    builder.push(
        ProvenanceOperation::GenerationStarted,
        ProvenanceOutcome::Started,
        vec![sentence.content_span.clone()],
        BTreeMap::new(),
    );
    builder.push(
        ProvenanceOperation::GenerationSucceeded,
        ProvenanceOutcome::Succeeded,
        vec![sentence.content_span.clone()],
        BTreeMap::new(),
    );
    builder.finish()
}

pub fn generation_failure_fallback_provenance(
    compilation: &SentenceCompilation,
    sentence: &DocumentSentence,
    generator_id: &str,
    cause_code: &str,
    cause_message: &str,
    fallback: TranslationFallbackKind,
) -> SentenceProvenance {
    let mut builder = SentenceProvenanceBuilder::from_compilation_for_generation(
        &compilation.provenance,
        generator_id,
    );
    let span = sentence.content_span.clone();
    builder.push(
        ProvenanceOperation::GenerationStarted,
        ProvenanceOutcome::Started,
        vec![span.clone()],
        BTreeMap::new(),
    );
    builder.push(
        ProvenanceOperation::GenerationFailed,
        ProvenanceOutcome::Failed,
        vec![span.clone()],
        BTreeMap::from([
            ("cause".to_string(), cause_code.to_string()),
            ("message".to_string(), cause_message.to_string()),
            ("fallback".to_string(), format!("{fallback:?}")),
        ]),
    );
    builder.push(
        match fallback {
            TranslationFallbackKind::CopySource => ProvenanceOperation::SourceFallbackApplied,
            TranslationFallbackKind::Placeholder => ProvenanceOperation::PlaceholderApplied,
        },
        ProvenanceOutcome::Recovered,
        vec![span],
        BTreeMap::from([
            ("cause".to_string(), cause_code.to_string()),
            ("message".to_string(), cause_message.to_string()),
            ("fallback".to_string(), format!("{fallback:?}")),
        ]),
    );
    builder.finish()
}

pub fn generation_skipped_fallback_provenance(
    compilation: &SentenceCompilation,
    sentence: &DocumentSentence,
    generator_id: &str,
    cause_code: &str,
    cause_message: &str,
    fallback: TranslationFallbackKind,
) -> SentenceProvenance {
    let mut builder = SentenceProvenanceBuilder::from_compilation_for_generation(
        &compilation.provenance,
        generator_id,
    );
    let span = sentence.content_span.clone();
    builder.push(
        ProvenanceOperation::GenerationSkipped,
        ProvenanceOutcome::Skipped,
        vec![span.clone()],
        BTreeMap::from([
            ("cause".to_string(), cause_code.to_string()),
            ("message".to_string(), cause_message.to_string()),
            ("compilation_status".to_string(), format!("{:?}", compilation.status)),
            ("fallback".to_string(), format!("{fallback:?}")),
        ]),
    );
    builder.push(
        match fallback {
            TranslationFallbackKind::CopySource => ProvenanceOperation::SourceFallbackApplied,
            TranslationFallbackKind::Placeholder => ProvenanceOperation::PlaceholderApplied,
        },
        ProvenanceOutcome::Recovered,
        vec![span],
        BTreeMap::from([
            ("cause".to_string(), cause_code.to_string()),
            ("message".to_string(), cause_message.to_string()),
            ("compilation_status".to_string(), format!("{:?}", compilation.status)),
            ("fallback".to_string(), format!("{fallback:?}")),
        ]),
    );
    builder.finish()
}
