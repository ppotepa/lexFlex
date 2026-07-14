use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentCorpusManifest {
    pub schema_version: u32,
    pub corpus_id: String,
    pub profile_path: String,
    pub cases: Vec<DocumentCaseRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentCaseRef {
    pub id: String,
    pub split: CorpusSplit,
    pub directory: String,
    pub enabled: bool,
    pub blocking: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorpusSplit {
    Dev,
    Regression,
    Holdout,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentCase {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub source_language: String,
    pub target_language: String,
    pub length_tier: String,
    pub domain: String,
    pub difficulty: u8,
    pub tags: Vec<String>,
    pub source_file: String,
    pub reference_files: Vec<String>,
    pub expectations_file: String,
    pub glossary_file: String,
    pub known_limitations: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlossaryConstraint {
    pub id: String,
    pub source: String,
    pub preferred_target: String,
    pub allowed_targets: Vec<String>,
    pub forbidden_targets: Vec<String>,
    pub case_sensitive: bool,
    pub minimum_occurrences: usize,
    pub consistency_required: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentExpectations {
    pub expected_paragraph_count: Option<usize>,
    pub expected_min_sentence_count: Option<usize>,
    pub invariants: Vec<DocumentInvariant>,
    pub allowed_failures: Vec<KnownFailure>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentInvariant {
    PreserveName {
        source: String,
        accepted_targets: Vec<String>,
        minimum_occurrences: usize,
    },
    PreserveNumber {
        source_surface: String,
        normalized_value: String,
        accepted_targets: Vec<String>,
    },
    PreserveNegation {
        sentence_hint: usize,
        predicate_concept: Option<String>,
    },
    RequireFrame {
        frame_type: String,
        verb_concept: Option<String>,
        minimum_occurrences: usize,
    },
    RequireRoleBinding {
        frame_type: String,
        role: String,
        concept: String,
        name: Option<String>,
        minimum_occurrences: usize,
    },
    RequireReference {
        mention: String,
        antecedent_name: String,
        target_forms: Vec<String>,
    },
    RequireTemporal {
        kind: String,
        target_forms: Vec<String>,
    },
    RequireParagraphCount {
        count: usize,
    },
    RequireOutputContains {
        any_of: Vec<String>,
        case_sensitive: bool,
    },
    ForbidOutputContains {
        any_of: Vec<String>,
        case_sensitive: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnownFailure {
    pub id: String,
    pub category: DocumentErrorCategory,
    pub description: String,
    pub expected_until_chapter: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentErrorCategory {
    Segmentation,
    UnknownToken,
    Morphology,
    Syntax,
    Frame,
    SemanticRole,
    Coreference,
    ZeroAnaphora,
    Temporal,
    Negation,
    Quantifier,
    LexicalChoice,
    Article,
    Agreement,
    WordOrder,
    Terminology,
    Formatting,
    Omission,
    Hallucination,
    Runtime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentMetrics {
    pub technical: BTreeMap<String, MetricValue>,
    pub structure: BTreeMap<String, MetricValue>,
    pub semantics: BTreeMap<String, MetricValue>,
    pub glossary: BTreeMap<String, MetricValue>,
    pub expectations: BTreeMap<String, MetricValue>,
    pub reference: BTreeMap<String, MetricValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricValue {
    pub value: Option<f64>,
    pub numerator: Option<f64>,
    pub denominator: Option<f64>,
    pub status: MetricStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricStatus {
    Exact,
    Approximate,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpectationResult {
    pub invariant_index: usize,
    pub invariant_type: String,
    pub passed: bool,
    pub known_failure: Option<KnownFailure>,
    pub severity: String,
    pub message: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentCaseRun {
    pub case_id: String,
    pub split: CorpusSplit,
    pub source: String,
    pub output: Option<String>,
    pub source_semantics: Option<CanonicalDocumentSemantics>,
    pub target_semantics: Option<CanonicalDocumentSemantics>,
    pub metrics: DocumentMetrics,
    pub expectations: Vec<ExpectationResult>,
    pub glossary_results: Vec<ExpectationResult>,
    pub deterministic: bool,
    pub runtime_ms: u128,
    pub error_categories: Vec<String>,
    #[serde(default)]
    pub provenance: Option<CaseProvenance>,
    #[serde(default)]
    pub document_artifacts: Option<CanonicalDocumentArtifacts>,
    #[serde(default)]
    pub document_graph_artifacts: Option<CanonicalDocumentGraphArtifacts>,
    #[serde(default)]
    pub document_resolution_artifacts: Option<CanonicalDocumentResolutionArtifacts>,
    #[serde(skip_serializing, skip_deserializing, default)]
    pub document_compilation: Option<lexflex::document::compilation::DocumentCompilation>,
    #[serde(skip_serializing, skip_deserializing, default)]
    pub document_translation: Option<lexflex::document::translation::DocumentTranslation>,
    #[serde(skip_serializing, skip_deserializing, default)]
    pub document_graph: Option<lexflex::document::graph::DocumentGraph>,
    #[serde(skip_serializing, skip_deserializing, default)]
    pub document_resolution: Option<lexflex::document::resolution::DocumentEntityResolution>,
    #[serde(default)]
    pub document_error_categories: Vec<String>,
    #[serde(default)]
    pub best_effort_output: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentRun {
    pub run_id: String,
    pub corpus_id: String,
    pub profile_id: String,
    pub git_commit: Option<String>,
    pub cases: Vec<DocumentCaseRun>,
    pub summary: RunSummary,
    #[serde(default)]
    pub provenance: Option<RunProvenance>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunSummary {
    pub total_cases: usize,
    pub success_cases: usize,
    pub fatal_cases: usize,
    pub major_cases: usize,
    pub minor_cases: usize,
    pub missing_manual_reviews: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaselineComparison {
    pub baseline_run_id: String,
    pub candidate_run_id: String,
    pub entries: Vec<BaselineComparisonEntry>,
    pub critical_regressions: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunProvenance {
    pub schema_version: u32,
    pub benchmark_version: String,
    pub corpus_sha256: String,
    pub manifest_sha256: String,
    pub profile_sha256: String,
    pub command: Vec<String>,
    #[serde(default)]
    pub selection: RunSelectionProvenance,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseProvenance {
    pub source_sha256: String,
    pub reference_sha256: Vec<String>,
    pub metadata_sha256: String,
    pub expectations_sha256: String,
    pub glossary_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunSelectionProvenance {
    pub repeat_count: usize,
    pub split: Option<CorpusSplit>,
    pub case_filter: Vec<String>,
    pub all_enabled_cases: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalDocumentArtifacts {
    pub structure: CanonicalDocumentStructure,
    pub compilation: CanonicalDocumentCompilation,
    pub best_effort: Option<CanonicalBestEffortTranslation>,
    pub graph: Option<CanonicalDocumentGraphArtifacts>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalDocumentStructure {
    pub document_id: String,
    pub source_sha256: String,
    pub block_count: usize,
    pub paragraph_count: usize,
    pub sentence_count: usize,
    pub reconstruction_exact: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalStatusCounts {
    pub pending: usize,
    pub resolved: usize,
    pub partial: usize,
    pub unresolved: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalDocumentCompilation {
    pub result_count: usize,
    pub silent_drop_count: usize,
    pub status_counts: CanonicalStatusCounts,
    pub diagnostics_by_code: BTreeMap<String, usize>,
    pub compilation_sha256: String,
    pub valid: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalBestEffortTranslation {
    pub output_sha256: String,
    pub translation_sha256: String,
    pub output_non_empty: bool,
    pub sentence_result_count: usize,
    pub translated: usize,
    pub source_fallback: usize,
    pub placeholder: usize,
    pub paragraph_count: usize,
    pub deterministic: bool,
    pub valid: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalDocumentGraphArtifacts {
    pub schema_version: u32,
    pub algorithm_version: u32,
    pub graph_sha256: String,
    pub build_success: bool,
    pub valid: bool,
    pub deterministic: bool,
    pub nodes_total: usize,
    pub edges_total: usize,
    pub block_nodes: usize,
    pub paragraph_nodes: usize,
    pub source_sentence_nodes: usize,
    pub semantic_sentence_nodes: usize,
    pub frame_occurrence_nodes: usize,
    pub mention_nodes: usize,
    pub unresolved_fragment_nodes: usize,
    pub dangling_edges: usize,
    pub fatal_graph_diagnostics: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalDocumentResolutionArtifacts {
    pub schema_version: u32,
    pub algorithm_version: u32,
    pub resolution_sha256: String,
    pub valid: bool,
    pub deterministic: bool,
    pub mentions_total: usize,
    pub synthetic_mentions_total: usize,
    pub decisions_total: usize,
    pub clusters_total: usize,
    pub accepted: usize,
    pub hard_accepted: usize,
    pub ambiguous: usize,
    pub deferred: usize,
    pub unresolved: usize,
    pub excluded: usize,
    pub diagnostics_info: usize,
    pub diagnostics_warning: usize,
    pub diagnostics_error: usize,
    pub diagnostics_fatal: usize,
}

impl Default for RunSelectionProvenance {
    fn default() -> Self {
        Self {
            repeat_count: 0,
            split: None,
            case_filter: Vec::new(),
            all_enabled_cases: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BaselineComparisonEntry {
    pub case_id: String,
    pub status: ComparisonStatus,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonStatus {
    Improved,
    Regressed,
    Unchanged,
    Added,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalDocumentSemantics {
    pub sentences: Vec<CanonicalSentence>,
    pub entities: Vec<CanonicalEntityMention>,
    pub unresolved_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalSentence {
    pub index: usize,
    pub frames: Vec<CanonicalFrame>,
    pub tense: Option<String>,
    pub aspect: Option<String>,
    pub polarity: String,
    pub modality: Option<String>,
    pub illocution: String,
    pub voice: Option<String>,
    pub temporal: Option<String>,
    pub quantification: Option<String>,
    pub constructions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalFrame {
    pub frame_type: String,
    pub verb_concept: String,
    pub roles: Vec<CanonicalRole>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalRole {
    pub role: String,
    pub concept: String,
    pub name: Option<String>,
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanonicalEntityMention {
    pub concept: String,
    pub name: Option<String>,
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityCoverageDeficit {
    pub tag: String,
    pub found: usize,
    pub required: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityCoverage {
    pub counts_by_tag: BTreeMap<String, usize>,
    pub missing_required_tags: Vec<String>,
    pub below_minimum: Vec<CapabilityCoverageDeficit>,
}
