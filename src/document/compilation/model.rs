use crate::core::interlingua::Utterance;
use crate::document::{Document, DocumentDiagnostic, SentenceId, SentenceProvenance};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SentenceProcessingStatus {
    Pending,
    Resolved,
    Partial,
    Unresolved,
    Failed,
}

impl SentenceProcessingStatus {
    pub fn is_terminal(self) -> bool {
        !matches!(self, Self::Pending)
    }

    pub fn is_usable(self) -> bool {
        matches!(self, Self::Resolved | Self::Partial)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SentenceSemanticInspection {
    pub utterance_sentence_count: usize,
    pub frame_count: usize,
    pub entity_count: usize,
    pub named_entity_count: usize,
    pub unresolved_reference_count: usize,
    pub deferred_reference_count: usize,
    pub direct_reference_count: usize,
    pub empty_concept_count: usize,
    pub empty_name_count: usize,
    pub has_positive_polarity: bool,
    pub has_negative_polarity: bool,
    pub has_temporal_information: bool,
    pub has_quantification: bool,
    pub reasons: Vec<String>,
}

impl SentenceSemanticInspection {
    pub fn has_usable_frames(&self) -> bool {
        self.frame_count > 0
    }

    pub fn has_hard_uncertainty(&self) -> bool {
        self.unresolved_reference_count > 0 || self.empty_concept_count > 0
    }

    pub fn has_deferred_uncertainty(&self) -> bool {
        self.deferred_reference_count > 0 || self.utterance_sentence_count != 1
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SentenceCompilation {
    pub sentence_id: SentenceId,
    pub status: SentenceProcessingStatus,
    pub semantics: Option<Utterance>,
    pub inspection: SentenceSemanticInspection,
    pub diagnostics: Vec<DocumentDiagnostic>,
    pub provenance: SentenceProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DocumentCompilationSummary {
    pub total_sentences: usize,
    pub pending: usize,
    pub resolved: usize,
    pub partial: usize,
    pub unresolved: usize,
    pub failed: usize,
    pub usable: usize,
    pub diagnostics_info: usize,
    pub diagnostics_warning: usize,
    pub diagnostics_error: usize,
    pub diagnostics_fatal: usize,
}

impl DocumentCompilationSummary {
    pub fn terminal_count(&self) -> usize {
        self.resolved + self.partial + self.unresolved + self.failed
    }

    pub fn coverage_ratio(&self) -> f64 {
        if self.total_sentences == 0 {
            1.0
        } else {
            self.terminal_count() as f64 / self.total_sentences as f64
        }
    }

    pub fn usable_ratio(&self) -> f64 {
        if self.total_sentences == 0 {
            1.0
        } else {
            self.usable as f64 / self.total_sentences as f64
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentCompilation {
    pub document: Document,
    pub sentence_results: BTreeMap<SentenceId, SentenceCompilation>,
    pub diagnostics: Vec<DocumentDiagnostic>,
    pub summary: DocumentCompilationSummary,
    pub compilation_sha256: String,
}

impl DocumentCompilation {
    pub fn result(&self, sentence_id: &SentenceId) -> Option<&SentenceCompilation> {
        self.sentence_results.get(sentence_id)
    }

    pub fn ordered_results(&self) -> Vec<&SentenceCompilation> {
        self.document
            .ordered_sentences()
            .into_iter()
            .filter_map(|sentence| self.sentence_results.get(&sentence.id))
            .collect()
    }

    pub fn has_pending_results(&self) -> bool {
        self.sentence_results
            .values()
            .any(|result| result.status == SentenceProcessingStatus::Pending)
    }

    pub fn silent_drop_count(&self) -> usize {
        self.document
            .sentences()
            .keys()
            .filter(|id| !self.sentence_results.contains_key(*id))
            .count()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentCompilationOptions {
    pub catch_panics: bool,
}

impl Default for DocumentCompilationOptions {
    fn default() -> Self {
        Self { catch_panics: true }
    }
}

pub fn classify_sentence_status(
    inspection: &SentenceSemanticInspection,
    analyzer_succeeded: bool,
) -> SentenceProcessingStatus {
    if !analyzer_succeeded {
        return SentenceProcessingStatus::Failed;
    }
    if inspection.utterance_sentence_count == 0 || inspection.frame_count == 0 {
        return SentenceProcessingStatus::Unresolved;
    }
    if inspection.has_hard_uncertainty() || inspection.has_deferred_uncertainty() {
        return SentenceProcessingStatus::Partial;
    }
    SentenceProcessingStatus::Resolved
}

pub fn summarize_compilation(
    document: &Document,
    sentence_results: &BTreeMap<SentenceId, SentenceCompilation>,
    diagnostics: &[DocumentDiagnostic],
) -> DocumentCompilationSummary {
    let mut summary = DocumentCompilationSummary {
        total_sentences: document.sentences().len(),
        ..Default::default()
    };
    for result in sentence_results.values() {
        match result.status {
            SentenceProcessingStatus::Pending => summary.pending += 1,
            SentenceProcessingStatus::Resolved => summary.resolved += 1,
            SentenceProcessingStatus::Partial => summary.partial += 1,
            SentenceProcessingStatus::Unresolved => summary.unresolved += 1,
            SentenceProcessingStatus::Failed => summary.failed += 1,
        }
        if result.status.is_usable() {
            summary.usable += 1;
        }
    }
    for diagnostic in diagnostics {
        match diagnostic.severity {
            crate::document::DocumentDiagnosticSeverity::Info => summary.diagnostics_info += 1,
            crate::document::DocumentDiagnosticSeverity::Warning => {
                summary.diagnostics_warning += 1
            }
            crate::document::DocumentDiagnosticSeverity::Error => summary.diagnostics_error += 1,
            crate::document::DocumentDiagnosticSeverity::Fatal => summary.diagnostics_fatal += 1,
        }
    }
    summary
}
