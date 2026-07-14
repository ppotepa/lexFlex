use super::analyzer::{
    DocumentSentenceAnalyzer, SentenceAnalysisError, SentenceAnalysisInput,
};
use super::hash::compilation_hash;
use super::invariants::validated_sentence_content;
use super::inspect::SemanticInspector;
use super::model::{
    classify_sentence_status, summarize_compilation, DocumentCompilation,
    DocumentCompilationOptions, SentenceCompilation, SentenceProcessingStatus,
    SentenceSemanticInspection,
};
use super::provenance::{
    ProvenanceOperation, ProvenanceOutcome, SentenceProvenanceBuilder,
};
use super::validation::DocumentCompilationValidator;
use super::error::DocumentCompilationError;
use crate::core::interlingua::Utterance;
use crate::document::{
    Document, DocumentDiagnosticCollector, DocumentDiagnosticSeverity,
    DocumentDiagnosticSpec, DocumentReconstructor, DocumentSentence,
    DocumentStage, DocumentValidator, LocatedSpan, DOC_ANALYSIS_EMPTY_CONCEPT,
    DOC_ANALYSIS_EMPTY_UTTERANCE, DOC_ANALYSIS_MULTI_SENTENCE_OUTPUT,
    DOC_ANALYSIS_NO_FRAME, DOC_ANALYSIS_PANIC, DOC_ANALYSIS_UNRESOLVED_REFERENCE,
    DOC_ANALYSIS_DEFERRED_REFERENCE,
};
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

pub struct DocumentCompiler<A> {
    analyzer: A,
    options: DocumentCompilationOptions,
}

impl<A> DocumentCompiler<A>
where
    A: DocumentSentenceAnalyzer,
{
    pub fn new(analyzer: A) -> Self {
        Self {
            analyzer,
            options: DocumentCompilationOptions::default(),
        }
    }

    pub fn with_options(analyzer: A, options: DocumentCompilationOptions) -> Self {
        Self { analyzer, options }
    }

    pub fn compile(&self, document: Document) -> Result<DocumentCompilation, DocumentCompilationError> {
        DocumentValidator::validate(&document).map_err(|errors| {
            DocumentCompilationError::InvalidDocument { errors }
        })?;
        DocumentReconstructor::verify_reconstruction_only(&document).map_err(|errors| {
            DocumentCompilationError::Reconstruction { errors }
        })?;

        let mut sentence_results = BTreeMap::new();
        let mut collector = DocumentDiagnosticCollector::new(document.id.clone());

        for sentence in document.ordered_sentences() {
            let result = self.compile_sentence(&document, sentence, &mut collector)?;
            sentence_results.insert(sentence.id.clone(), result);
        }

        let diagnostics = collector.into_sorted();
        let summary = summarize_compilation(&document, &sentence_results, &diagnostics);
        let mut compilation = DocumentCompilation {
            document,
            sentence_results,
            diagnostics,
            summary,
            compilation_sha256: String::new(),
        };
        compilation.compilation_sha256 = compilation_hash(&compilation)?;
        DocumentCompilationValidator::validate(&compilation).map_err(|errors| {
            DocumentCompilationError::InvalidCompilation { errors }
        })?;
        Ok(compilation)
    }

    fn compile_sentence(
        &self,
        document: &Document,
        sentence: &DocumentSentence,
        collector: &mut DocumentDiagnosticCollector,
    ) -> Result<SentenceCompilation, DocumentCompilationError> {
        let content = validated_sentence_content(document, sentence)
            .map_err(|errors| DocumentCompilationError::InvalidDocument { errors })?;
        let mut provenance = SentenceProvenanceBuilder::for_analysis(
            sentence.id.clone(),
            content.span,
            content.sha256,
            self.analyzer.analyzer_id(),
        );
        provenance.push(
            ProvenanceOperation::SentenceSelected,
            ProvenanceOutcome::Started,
            vec![LocatedSpan::exact(content.span)],
            BTreeMap::new(),
        );
        provenance.push(
            ProvenanceOperation::AnalysisStarted,
            ProvenanceOutcome::Started,
            vec![LocatedSpan::exact(content.span)],
            BTreeMap::new(),
        );
        let outcome = self.run_analyzer(SentenceAnalysisInput {
            document,
            sentence,
            text: content.text,
            content_span: content.span,
            source_language: document.source_language(),
        });
        match outcome {
            AnalyzerOutcome::Success(utterance) => {
                provenance.push(
                    ProvenanceOperation::AnalysisSucceeded,
                    ProvenanceOutcome::Succeeded,
                    vec![LocatedSpan::exact(content.span)],
                    BTreeMap::new(),
                );
                let inspection = SemanticInspector::inspect(&utterance);
                emit_semantic_diagnostics(sentence, &inspection, collector);
                provenance.push(
                    ProvenanceOperation::SemanticInspected,
                    ProvenanceOutcome::Succeeded,
                    vec![LocatedSpan::exact(content.span)],
                    BTreeMap::from([
                        ("frame_count".to_string(), inspection.frame_count.to_string()),
                        ("entity_count".to_string(), inspection.entity_count.to_string()),
                        (
                            "unresolved_reference_count".to_string(),
                            inspection.unresolved_reference_count.to_string(),
                        ),
                        (
                            "deferred_reference_count".to_string(),
                            inspection.deferred_reference_count.to_string(),
                        ),
                    ]),
                );
                let status = classify_sentence_status(&inspection, true);
                provenance.push(
                    ProvenanceOperation::StatusClassified,
                    ProvenanceOutcome::Succeeded,
                    vec![LocatedSpan::exact(content.span)],
                    BTreeMap::from([("status".to_string(), format!("{status:?}"))]),
                );
                Ok(SentenceCompilation {
                    sentence_id: sentence.id.clone(),
                    status,
                    semantics: Some(utterance),
                    inspection,
                    diagnostics: diagnostics_for_sentence(collector, &sentence.id),
                    provenance: provenance.finish(),
                })
            }
            AnalyzerOutcome::Error(error) => failed_sentence_with_error(
                sentence,
                collector,
                provenance,
                error,
            ),
            AnalyzerOutcome::Panic(message) => {
                provenance.push(
                    ProvenanceOperation::AnalysisPanicked,
                    ProvenanceOutcome::Failed,
                    vec![LocatedSpan::exact(content.span)],
                    BTreeMap::from([("panic".to_string(), message.clone())]),
                );
                provenance.push(
                    ProvenanceOperation::StatusClassified,
                    ProvenanceOutcome::Succeeded,
                    vec![LocatedSpan::exact(content.span)],
                    BTreeMap::from([
                        ("status".to_string(), "Failed".to_string()),
                        ("reason".to_string(), "analyzer_panic".to_string()),
                    ]),
                );
                collector.push_spec(DocumentDiagnosticSpec {
                    stage: DocumentStage::Parsing,
                    severity: DocumentDiagnosticSeverity::Error,
                    code: DOC_ANALYSIS_PANIC.to_string(),
                    message: "sentence analyzer panicked".to_string(),
                    span: LocatedSpan::exact(content.span),
                    sentence_id: Some(sentence.id.clone()),
                    cause: Some(message),
                    recoverable: true,
                });
                Ok(SentenceCompilation {
                    sentence_id: sentence.id.clone(),
                    status: SentenceProcessingStatus::Failed,
                    semantics: None,
                    inspection: SentenceSemanticInspection::default(),
                    diagnostics: diagnostics_for_sentence(collector, &sentence.id),
                    provenance: provenance.finish(),
                })
            }
        }
    }

    fn run_analyzer(&self, input: SentenceAnalysisInput<'_>) -> AnalyzerOutcome {
        if !self.options.catch_panics {
            return match self.analyzer.analyze(input) {
                Ok(value) => AnalyzerOutcome::Success(value),
                Err(error) => AnalyzerOutcome::Error(error),
            };
        }
        match catch_unwind(AssertUnwindSafe(|| self.analyzer.analyze(input))) {
            Ok(Ok(value)) => AnalyzerOutcome::Success(value),
            Ok(Err(error)) => AnalyzerOutcome::Error(error),
            Err(payload) => AnalyzerOutcome::Panic(panic_message(payload)),
        }
    }
}

enum AnalyzerOutcome {
    Success(Utterance),
    Error(SentenceAnalysisError),
    Panic(String),
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    "non-string panic payload".to_string()
}

fn failed_sentence_with_error(
    sentence: &DocumentSentence,
    collector: &mut DocumentDiagnosticCollector,
    mut provenance: SentenceProvenanceBuilder,
    error: SentenceAnalysisError,
) -> Result<SentenceCompilation, DocumentCompilationError> {
    let span = sentence.content_span.clone();
    provenance.push(
        ProvenanceOperation::AnalysisFailed,
        ProvenanceOutcome::Failed,
        vec![span.clone()],
        BTreeMap::from([
            ("code".to_string(), error.code.clone()),
            ("message".to_string(), error.message.clone()),
        ]),
    );
    provenance.push(
        ProvenanceOperation::StatusClassified,
        ProvenanceOutcome::Succeeded,
        vec![span.clone()],
        BTreeMap::from([
            ("status".to_string(), "Failed".to_string()),
            ("reason".to_string(), "analyzer_error".to_string()),
        ]),
    );
    collector.push_spec(DocumentDiagnosticSpec {
        stage: DocumentStage::Parsing,
        severity: DocumentDiagnosticSeverity::Error,
        code: error.code.clone(),
        message: error.message.clone(),
        span,
        sentence_id: Some(sentence.id.clone()),
        cause: error.cause,
        recoverable: true,
    });
    Ok(SentenceCompilation {
        sentence_id: sentence.id.clone(),
        status: SentenceProcessingStatus::Failed,
        semantics: None,
        inspection: SentenceSemanticInspection::default(),
        diagnostics: diagnostics_for_sentence(collector, &sentence.id),
        provenance: provenance.finish(),
    })
}

fn emit_semantic_diagnostics(
    sentence: &DocumentSentence,
    inspection: &SentenceSemanticInspection,
    collector: &mut DocumentDiagnosticCollector,
) {
    let span = sentence.content_span.clone();
    if inspection.utterance_sentence_count == 0 {
        collector.push_spec(DocumentDiagnosticSpec {
            stage: DocumentStage::Compilation,
            severity: DocumentDiagnosticSeverity::Error,
            code: DOC_ANALYSIS_EMPTY_UTTERANCE.to_string(),
            message: "analysis produced an empty utterance".to_string(),
            span: span.clone(),
            sentence_id: Some(sentence.id.clone()),
            cause: None,
            recoverable: true,
        });
    }
    if inspection.frame_count == 0 {
        collector.push_spec(DocumentDiagnosticSpec {
            stage: DocumentStage::Compilation,
            severity: DocumentDiagnosticSeverity::Warning,
            code: DOC_ANALYSIS_NO_FRAME.to_string(),
            message: "analysis produced no semantic frame".to_string(),
            span: span.clone(),
            sentence_id: Some(sentence.id.clone()),
            cause: None,
            recoverable: true,
        });
    }
    if inspection.utterance_sentence_count > 1 {
        collector.push_spec(DocumentDiagnosticSpec {
            stage: DocumentStage::Compilation,
            severity: DocumentDiagnosticSeverity::Warning,
            code: DOC_ANALYSIS_MULTI_SENTENCE_OUTPUT.to_string(),
            message: format!(
                "one document sentence produced {} IL sentences",
                inspection.utterance_sentence_count
            ),
            span: span.clone(),
            sentence_id: Some(sentence.id.clone()),
            cause: None,
            recoverable: true,
        });
    }
    if inspection.unresolved_reference_count > 0 {
        collector.push_spec(DocumentDiagnosticSpec {
            stage: DocumentStage::Deduction,
            severity: DocumentDiagnosticSeverity::Warning,
            code: DOC_ANALYSIS_UNRESOLVED_REFERENCE.to_string(),
            message: format!(
                "{} unresolved references remain",
                inspection.unresolved_reference_count
            ),
            span: span.clone(),
            sentence_id: Some(sentence.id.clone()),
            cause: None,
            recoverable: true,
        });
    }
    if inspection.deferred_reference_count > 0 {
        collector.push_spec(DocumentDiagnosticSpec {
            stage: DocumentStage::Deduction,
            severity: DocumentDiagnosticSeverity::Info,
            code: DOC_ANALYSIS_DEFERRED_REFERENCE.to_string(),
            message: format!(
                "{} references require document-level resolution",
                inspection.deferred_reference_count
            ),
            span: span.clone(),
            sentence_id: Some(sentence.id.clone()),
            cause: None,
            recoverable: true,
        });
    }
    if inspection.empty_concept_count > 0 {
        collector.push_spec(DocumentDiagnosticSpec {
            stage: DocumentStage::Compilation,
            severity: DocumentDiagnosticSeverity::Warning,
            code: DOC_ANALYSIS_EMPTY_CONCEPT.to_string(),
            message: format!("{} entities contain an empty concept", inspection.empty_concept_count),
            span,
            sentence_id: Some(sentence.id.clone()),
            cause: None,
            recoverable: true,
        });
    }
}

fn diagnostics_for_sentence(
    collector: &DocumentDiagnosticCollector,
    sentence_id: &crate::document::SentenceId,
) -> Vec<crate::document::DocumentDiagnostic> {
    collector
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.sentence_id.as_ref() == Some(sentence_id))
        .cloned()
        .collect()
}
