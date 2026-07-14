use crate::document::id::{DiagnosticId, DocumentId, DocumentIdFactory, SentenceId};
use crate::document::span::LocatedSpan;
use serde::{Deserialize, Serialize};

pub const DOC_SEGMENT_EMPTY_PARAGRAPH: &str = "DOC_SEGMENT_EMPTY_PARAGRAPH";
pub const DOC_SEGMENT_UNCLOSED_QUOTE: &str = "DOC_SEGMENT_UNCLOSED_QUOTE";
pub const DOC_PARSE_FAILED: &str = "DOC_PARSE_FAILED";
pub const DOC_TRANSLATE_FAILED: &str = "DOC_TRANSLATE_FAILED";
pub const DOC_SPAN_INVALID: &str = "DOC_SPAN_INVALID";
pub const DOC_RECONSTRUCTION_MISMATCH: &str = "DOC_RECONSTRUCTION_MISMATCH";
pub const DOC_GRAPH_UNLINKED_MENTION: &str = "DOC_GRAPH_UNLINKED_MENTION";
pub const DOC_ANALYSIS_PANIC: &str = "DOC_ANALYSIS_PANIC";
pub const DOC_ANALYSIS_NON_NATURAL: &str = "DOC_ANALYSIS_NON_NATURAL";
pub const DOC_ANALYSIS_EMPTY_UTTERANCE: &str = "DOC_ANALYSIS_EMPTY_UTTERANCE";
pub const DOC_ANALYSIS_NO_FRAME: &str = "DOC_ANALYSIS_NO_FRAME";
pub const DOC_ANALYSIS_MULTI_SENTENCE_OUTPUT: &str = "DOC_ANALYSIS_MULTI_SENTENCE_OUTPUT";
pub const DOC_ANALYSIS_UNRESOLVED_REFERENCE: &str = "DOC_ANALYSIS_UNRESOLVED_REFERENCE";
pub const DOC_ANALYSIS_DEFERRED_REFERENCE: &str = "DOC_ANALYSIS_DEFERRED_REFERENCE";
pub const DOC_ANALYSIS_EMPTY_CONCEPT: &str = "DOC_ANALYSIS_EMPTY_CONCEPT";
pub const DOC_COMPILATION_MISSING_RESULT: &str = "DOC_COMPILATION_MISSING_RESULT";
pub const DOC_COMPILATION_UNEXPECTED_RESULT: &str = "DOC_COMPILATION_UNEXPECTED_RESULT";
pub const DOC_COMPILATION_PENDING_RESULT: &str = "DOC_COMPILATION_PENDING_RESULT";
pub const DOC_COMPILATION_SUMMARY_MISMATCH: &str = "DOC_COMPILATION_SUMMARY_MISMATCH";
pub const DOC_TRANSLATION_EMPTY_OUTPUT: &str = "DOC_TRANSLATION_EMPTY_OUTPUT";
pub const DOC_TRANSLATION_SOURCE_FALLBACK: &str = "DOC_TRANSLATION_SOURCE_FALLBACK";
pub const DOC_TRANSLATION_PLACEHOLDER: &str = "DOC_TRANSLATION_PLACEHOLDER";
pub const DOC_TRANSLATION_ASSEMBLY_MISMATCH: &str = "DOC_TRANSLATION_ASSEMBLY_MISMATCH";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DocumentStage {
    Segmentation,
    Parsing,
    Deduction,
    Compilation,
    Generation,
    Reconstruction,
    GraphBuild,
    Validation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DocumentDiagnosticSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentDiagnostic {
    pub id: DiagnosticId,
    pub code: String,
    pub stage: DocumentStage,
    pub severity: DocumentDiagnosticSeverity,
    pub message: String,
    pub span: LocatedSpan,
    pub sentence_id: Option<SentenceId>,
    pub cause: Option<String>,
    pub recoverable: bool,
}

#[derive(Debug, Clone)]
pub struct DocumentDiagnosticCollector {
    document_id: DocumentId,
    diagnostics: Vec<DocumentDiagnostic>,
}

impl DocumentDiagnosticCollector {
    pub fn new(document_id: DocumentId) -> Self {
        Self {
            document_id,
            diagnostics: Vec::new(),
        }
    }

    pub fn push(
        &mut self,
        stage: DocumentStage,
        severity: DocumentDiagnosticSeverity,
        code: impl Into<String>,
        message: impl Into<String>,
        span: LocatedSpan,
        sentence_id: Option<SentenceId>,
        cause: Option<String>,
        recoverable: bool,
    ) -> DiagnosticId {
        let id = DocumentIdFactory::diagnostic(&self.document_id, self.diagnostics.len());
        self.diagnostics.push(DocumentDiagnostic {
            id: id.clone(),
            code: code.into(),
            stage,
            severity,
            message: message.into(),
            span,
            sentence_id,
            cause,
            recoverable,
        });
        id
    }

    pub fn push_spec(&mut self, spec: DocumentDiagnosticSpec) -> DiagnosticId {
        self.push(
            spec.stage,
            spec.severity,
            spec.code,
            spec.message,
            spec.span,
            spec.sentence_id,
            spec.cause,
            spec.recoverable,
        )
    }

    pub fn diagnostics(&self) -> &[DocumentDiagnostic] {
        &self.diagnostics
    }

    pub fn into_sorted(mut self) -> Vec<DocumentDiagnostic> {
        self.diagnostics.sort_by(|a, b| {
            (
                a.stage,
                a.severity,
                a.code.as_str(),
                a.id.as_str(),
            )
                .cmp(&(
                    b.stage,
                    b.severity,
                    b.code.as_str(),
                    b.id.as_str(),
                ))
        });
        self.diagnostics
    }
}

#[derive(Debug, Clone)]
pub struct DocumentDiagnosticSpec {
    pub stage: DocumentStage,
    pub severity: DocumentDiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub span: LocatedSpan,
    pub sentence_id: Option<SentenceId>,
    pub cause: Option<String>,
    pub recoverable: bool,
}
