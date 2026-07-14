use crate::model::{
    CanonicalBestEffortTranslation, CanonicalDocumentArtifacts, CanonicalDocumentCompilation,
    CanonicalDocumentGraphArtifacts, CanonicalDocumentStructure, CanonicalStatusCounts,
};
use lexflex::core::interlingua::LanguageId;
use lexflex::document::compilation::{
    DocumentCompilation, DocumentCompilationValidator, SentenceProcessingStatus,
};
use lexflex::document::translation::{
    translation_output_hash, DocumentTranslation, DocumentTranslationValidator,
};
use lexflex::document::{
    DocumentInput, DocumentSegmentationOptions, LosslessDocumentSegmenter,
    DocumentReconstructor, DocumentSegmenter,
};
use std::collections::BTreeMap;

pub fn canonicalize_document_artifacts(
    compilation: &DocumentCompilation,
    translation: Option<&DocumentTranslation>,
    deterministic: bool,
    graph: Option<&CanonicalDocumentGraphArtifacts>,
) -> CanonicalDocumentArtifacts {
    let structure = CanonicalDocumentStructure {
        document_id: compilation.document.id().as_str().to_string(),
        source_sha256: compilation.document.source_sha256().to_string(),
        block_count: compilation.document.blocks().len(),
        paragraph_count: compilation.document.paragraphs().len(),
        sentence_count: compilation.document.sentences().len(),
        reconstruction_exact: DocumentReconstructor::verify_lossless(&compilation.document).is_ok(),
    };
    let compilation_artifact = CanonicalDocumentCompilation {
        result_count: compilation.sentence_results.len(),
        silent_drop_count: compilation.silent_drop_count(),
        status_counts: status_counts(compilation),
        diagnostics_by_code: diagnostics_by_code(&compilation.diagnostics),
        compilation_sha256: compilation.compilation_sha256.clone(),
        valid: DocumentCompilationValidator::validate(compilation).is_ok(),
    };
    let best_effort = translation.map(|translation| CanonicalBestEffortTranslation {
        output_sha256: translation.output_sha256.clone(),
        translation_sha256: translation.translation_sha256.clone(),
        output_non_empty: !translation.output.trim().is_empty(),
        sentence_result_count: translation.sentence_results.len(),
        translated: translation.summary.translated,
        source_fallback: translation.summary.source_fallback,
        placeholder: translation.summary.placeholder,
        paragraph_count: paragraph_count(&translation.output, &translation.target_language),
        deterministic,
        valid: DocumentTranslationValidator::validate(compilation, translation).is_ok()
            && translation_output_hash(&translation.output) == translation.output_sha256,
    });
    CanonicalDocumentArtifacts {
        structure,
        compilation: compilation_artifact,
        best_effort,
        graph: graph.cloned(),
    }
}

fn status_counts(compilation: &DocumentCompilation) -> CanonicalStatusCounts {
    let mut counts = CanonicalStatusCounts {
        pending: 0,
        resolved: 0,
        partial: 0,
        unresolved: 0,
        failed: 0,
    };
    for result in compilation.sentence_results.values() {
        match result.status {
            SentenceProcessingStatus::Pending => counts.pending += 1,
            SentenceProcessingStatus::Resolved => counts.resolved += 1,
            SentenceProcessingStatus::Partial => counts.partial += 1,
            SentenceProcessingStatus::Unresolved => counts.unresolved += 1,
            SentenceProcessingStatus::Failed => counts.failed += 1,
        }
    }
    counts
}

fn diagnostics_by_code(
    diagnostics: &[lexflex::document::DocumentDiagnostic],
) -> BTreeMap<String, usize> {
    let mut map = BTreeMap::new();
    for diagnostic in diagnostics {
        *map.entry(diagnostic.code.clone()).or_insert(0) += 1;
    }
    map
}

fn paragraph_count(text: &str, language: &LanguageId) -> usize {
    let segmenter = LosslessDocumentSegmenter::default();
    segmenter
        .segment(
            DocumentInput {
                id: None,
                source_language: language.clone(),
                source: text.to_string(),
            },
            &DocumentSegmentationOptions::default(),
        )
        .map(|document| document.paragraphs().len())
        .unwrap_or(0)
}
