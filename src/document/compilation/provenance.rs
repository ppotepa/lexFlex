use crate::document::id::{DocumentIdFactory, ProvenanceId, SentenceId};
use crate::document::span::{LocatedSpan, SourceSpan};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProvenanceOperation {
    SentenceSelected,
    AnalysisStarted,
    AnalysisSucceeded,
    AnalysisFailed,
    AnalysisPanicked,
    SemanticInspected,
    StatusClassified,
    GenerationStarted,
    GenerationSucceeded,
    GenerationFailed,
    GenerationSkipped,
    SourceFallbackApplied,
    PlaceholderApplied,
    OutputAssembled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProvenanceOutcome {
    Started,
    Succeeded,
    Failed,
    Skipped,
    Recovered,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceStep {
    pub id: ProvenanceId,
    pub operation: ProvenanceOperation,
    pub outcome: ProvenanceOutcome,
    pub input_spans: Vec<LocatedSpan>,
    pub details: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SentenceProvenance {
    pub sentence_id: SentenceId,
    pub source_span: SourceSpan,
    pub source_sha256: String,
    #[serde(default)]
    pub analyzer_id: Option<String>,
    #[serde(default)]
    pub generator_id: Option<String>,
    pub steps: Vec<ProvenanceStep>,
}

pub struct SentenceProvenanceBuilder {
    provenance: SentenceProvenance,
    ordinal: usize,
}

impl SentenceProvenanceBuilder {
    pub fn for_analysis(
        sentence_id: SentenceId,
        source_span: SourceSpan,
        source_sha256: String,
        analyzer_id: impl Into<String>,
    ) -> Self {
        Self {
            provenance: SentenceProvenance {
                sentence_id,
                source_span,
                source_sha256,
                analyzer_id: Some(analyzer_id.into()),
                generator_id: None,
                steps: Vec::new(),
            },
            ordinal: 0,
        }
    }

    pub fn from_compilation_for_generation(
        source: &SentenceProvenance,
        generator_id: impl Into<String>,
    ) -> Self {
        Self {
            provenance: SentenceProvenance {
                sentence_id: source.sentence_id.clone(),
                source_span: source.source_span,
                source_sha256: source.source_sha256.clone(),
                analyzer_id: source.analyzer_id.clone(),
                generator_id: Some(generator_id.into()),
                steps: source.steps.clone(),
            },
            ordinal: source.steps.len(),
        }
    }

    pub fn push(
        &mut self,
        operation: ProvenanceOperation,
        outcome: ProvenanceOutcome,
        input_spans: Vec<LocatedSpan>,
        details: BTreeMap<String, String>,
    ) -> ProvenanceId {
        let id = DocumentIdFactory::sentence_provenance(&self.provenance.sentence_id, self.ordinal);
        self.ordinal += 1;
        self.provenance.steps.push(ProvenanceStep {
            id: id.clone(),
            operation,
            outcome,
            input_spans,
            details,
        });
        id
    }

    pub fn finish(self) -> SentenceProvenance {
        self.provenance
    }
}
