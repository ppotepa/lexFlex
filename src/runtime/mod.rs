use serde::{Deserialize, Serialize};

use crate::api::LexFlexAPI;
use crate::document::knowledge::{
    DocumentKnowledgeExtraction, DocumentKnowledgeService, DocumentKnowledgeExtractionOptions,
};
use crate::document::temporal_discourse::{
    DocumentTemporalDiscourse, DocumentTemporalDiscourseOptions,
};
use crate::document::graph::DocumentGraph;
use crate::document::compilation::DocumentCompilation;
use crate::document::resolution::{DocumentEntityResolution, DocumentEntityResolutionOptions};
use crate::query::{DocumentAnswer, QueryInterlingua};
use crate::knowledge::{KnowledgeWorkspaceService, KnowledgeWorkspaceSnapshot, KnowledgeWorkspaceId};
use crate::runtime::bundle::SourceMetadata;
use std::collections::BTreeMap;

pub mod bundle;
pub use bundle::DocumentArtifactBundle;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentResourceBudget {
    pub max_source_bytes: usize,
    pub max_sentences: usize,
    pub max_frames: usize,
    pub max_claims: usize,
}

impl Default for DocumentResourceBudget {
    fn default() -> Self {
        Self {
            max_source_bytes: 1_000_000,
            max_sentences: 5_000,
            max_frames: 10_000,
            max_claims: 10_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexFlexRuntimeConfig {
    pub data_dir: String,
    pub offline: bool,
    pub budget: DocumentResourceBudget,
}

impl Default for LexFlexRuntimeConfig {
    fn default() -> Self {
        Self {
            data_dir: "data".into(),
            offline: true,
            budget: DocumentResourceBudget::default(),
        }
    }
}

pub struct LexFlexDocumentEngine {
    api: LexFlexAPI,
    workspace: KnowledgeWorkspaceService,
    config: LexFlexRuntimeConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum DocumentBundleError {
    #[error("source exceeds runtime budget")]
    SourceBudgetExceeded,
    #[error("document compilation failed: {0}")]
    Compilation(#[from] crate::document::service::DocumentServiceError),
    #[error("graph construction failed: {0}")]
    Graph(#[from] crate::document::graph::DocumentGraphServiceError),
    #[error("entity resolution failed: {0}")]
    Resolution(#[from] crate::document::resolution::DocumentEntityResolutionServiceError),
    #[error("temporal discourse failed: {0}")]
    Temporal(#[from] crate::document::temporal_discourse::DocumentTemporalDiscourseError),
    #[error("knowledge extraction failed: {0}")]
    Knowledge(#[from] crate::document::knowledge::DocumentKnowledgeError),
    #[error("artifact lineage invalid: {0}")]
    Lineage(String),
}

impl LexFlexDocumentEngine {
    pub fn parse(&self, input: &str, language: &str) -> Result<crate::core::interlingua::Interlingua, crate::error::LexFlexError> {
        self.api.parse(input, language)
    }
    pub fn translate(
        &self,
        input: &str,
        source_language: &str,
        target_language: &str,
    ) -> Result<String, crate::error::LexFlexError> {
        self.api.translate(input, source_language, target_language)
    }

    pub fn data_dir(&self) -> &str { &self.config.data_dir }
    pub fn config(&self) -> &LexFlexRuntimeConfig { &self.config }
    pub fn new(api: LexFlexAPI) -> Self {
        Self::with_config(api, LexFlexRuntimeConfig::default())
    }

    pub fn with_config(api: LexFlexAPI, config: LexFlexRuntimeConfig) -> Self {
        Self {
            api,
            workspace: KnowledgeWorkspaceService::default(),
            config,
        }
    }

    pub fn compile_document_bundle(
        &self,
        input: &str,
        source_language: &str,
    ) -> Result<DocumentCompilation, crate::document::service::DocumentServiceError> {
        self.api.compile_document(input, source_language)
    }

    pub fn ingest_document_bundle(
        &self,
        input: &str,
        source_language: &str,
    ) -> Result<DocumentArtifactBundle, DocumentBundleError> {
        let budget = &self.config.budget;
        if input.len() > budget.max_source_bytes {
            return Err(DocumentBundleError::SourceBudgetExceeded);
        }
        let compilation = self.api.compile_document(input, source_language)?;
        if compilation.summary.total_sentences > budget.max_sentences {
            return Err(DocumentBundleError::SourceBudgetExceeded);
        }
        let graph = self.api.build_document_graph(&compilation)?;
        if graph.summary.frame_occurrence_nodes > budget.max_frames {
            return Err(DocumentBundleError::SourceBudgetExceeded);
        }
        let resolution = self.api.resolve_document_graph(&graph)?;
        let temporal = self.api.resolve_document_temporal_discourse(
            &compilation,
            &graph,
            Some(&resolution),
        )?;
        let knowledge = self.api.extract_document_knowledge(
            &compilation,
            &graph,
            Some(&resolution),
            Some(&temporal),
        )?;
        if knowledge.summary.claims_total > budget.max_claims {
            return Err(DocumentBundleError::SourceBudgetExceeded);
        }
        let bundle_summary = format!("sentences={} graph_nodes={} claims={}", compilation.summary.total_sentences, graph.nodes.len(), knowledge.summary.claims_total);
        let artifact_checksums = BTreeMap::from([
            ("compilation".into(), compilation.compilation_sha256.clone()),
            ("graph".into(), graph.graph_sha256.clone()),
            ("resolution".into(), resolution.resolution_sha256.clone()),
            ("temporal_discourse".into(), temporal.temporal_discourse_sha256.clone()),
            ("knowledge".into(), knowledge.knowledge_sha256.clone()),
        ]);
        let mut bundle = DocumentArtifactBundle {
            source_document_id: compilation.document.id.to_string(),
            source_sha256: compilation.document.source_sha256.clone(),
            source_metadata: SourceMetadata { title: String::new(), language: source_language.into(), uri: None, revision: None },
            compilation,
            graph,
            entity_resolution: resolution,
            temporal_discourse: temporal,
            knowledge,
            summary: bundle_summary,
            artifact_checksums,
            complete: true,
            bundle_sha256: String::new(),
        };
        bundle.bundle_sha256 = bundle
            .recompute_hash()
            .map_err(|error| DocumentBundleError::Lineage(error.to_string()))?;
        bundle.validate_lineage().map_err(DocumentBundleError::Lineage)?;
        Ok(bundle)
    }

    pub fn ingest_document_bundle_with_metadata(
        &self,
        input: &str,
        source_language: &str,
        metadata: SourceMetadata,
    ) -> Result<DocumentArtifactBundle, DocumentBundleError> {
        let mut bundle = self.ingest_document_bundle(input, source_language)?;
        bundle.source_metadata = metadata;
        bundle.bundle_sha256 = bundle.recompute_hash().map_err(|error| DocumentBundleError::Lineage(error.to_string()))?;
        bundle.validate_lineage().map_err(DocumentBundleError::Lineage)?;
        Ok(bundle)
    }

    pub fn resolve_document(
        &self,
        graph: &DocumentGraph,
        options: &DocumentEntityResolutionOptions,
    ) -> Result<DocumentEntityResolution, crate::document::resolution::DocumentEntityResolutionServiceError> {
        self.api.resolve_document_graph_with_options(graph, options)
    }

    pub fn resolve_temporal_discourse(
        &self,
        compilation: &DocumentCompilation,
        graph: &DocumentGraph,
        resolution: Option<&DocumentEntityResolution>,
        options: &DocumentTemporalDiscourseOptions,
    ) -> Result<DocumentTemporalDiscourse, crate::document::temporal_discourse::DocumentTemporalDiscourseError> {
        self.api
            .resolve_document_temporal_discourse_with_options(compilation, graph, resolution, options)
    }

    pub fn extract_knowledge(
        &self,
        compilation: &DocumentCompilation,
        graph: &DocumentGraph,
        resolution: Option<&DocumentEntityResolution>,
        temporal: Option<&DocumentTemporalDiscourse>,
        options: &DocumentKnowledgeExtractionOptions,
    ) -> Result<DocumentKnowledgeExtraction, crate::document::knowledge::DocumentKnowledgeError> {
        DocumentKnowledgeService::with_options(options.clone()).extract(compilation, graph, resolution, temporal)
    }

    pub fn answer_document(
        &self,
        query: QueryInterlingua,
        knowledge: &DocumentKnowledgeExtraction,
    ) -> Result<DocumentAnswer, crate::query::QueryError> {
        crate::query::QueryService::answer_document_query(query, knowledge)
    }

    pub fn ingest_workspace_bundle(
        &mut self,
        bundle: crate::knowledge::DocumentArtifactBundleManifest,
    ) -> Result<KnowledgeWorkspaceSnapshot, crate::knowledge::KnowledgeWorkspaceError> {
        self.workspace.ingest_bundle(bundle)?;
        Ok(self.workspace.snapshot(KnowledgeWorkspaceId("workspace".into())))
    }

    pub fn health_report(&self) -> String {
        "ok".into()
    }
}
