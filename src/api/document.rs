use super::LexFlexAPI;
use crate::document::graph::{
    DocumentGraph, DocumentGraphBuildOptions, DocumentGraphService, DocumentGraphServiceError,
};
use crate::document::knowledge::{
    DocumentKnowledgeError, DocumentKnowledgeExtraction, DocumentKnowledgeExtractionOptions,
    DocumentKnowledgeService,
};
use crate::document::temporal_discourse::{
    DocumentTemporalDiscourse, DocumentTemporalDiscourseError, DocumentTemporalDiscourseOptions,
    DocumentTemporalDiscourseService,
};
use crate::document::{
    DocumentCompilation, DocumentEntityResolution, DocumentEntityResolutionOptions,
    DocumentEntityResolutionService, DocumentEntityResolutionServiceError, DocumentService,
    DocumentServiceError, DocumentTranslation, DocumentTranslationOptions,
};
use crate::query::{DocumentAnswer, QueryError, QueryInterlingua, QueryService};

impl LexFlexAPI {
    pub fn segment_document(
        &self,
        input: &str,
        source_language: &str,
    ) -> Result<crate::document::Document, DocumentServiceError> {
        DocumentService::new(self).segment(input, source_language)
    }

    pub fn compile_document(
        &self,
        input: &str,
        source_language: &str,
    ) -> Result<DocumentCompilation, DocumentServiceError> {
        DocumentService::new(self).compile(input, source_language)
    }

    pub fn translate_document_best_effort(
        &self,
        input: &str,
        source_language: &str,
        target_language: &str,
    ) -> Result<DocumentTranslation, DocumentServiceError> {
        DocumentService::new(self).translate_best_effort(input, source_language, target_language)
    }

    pub fn translate_document_best_effort_with_options(
        &self,
        input: &str,
        source_language: &str,
        target_language: &str,
        options: &DocumentTranslationOptions,
    ) -> Result<DocumentTranslation, DocumentServiceError> {
        DocumentService::new(self).translate_best_effort_with_options(
            input,
            source_language,
            target_language,
            options,
        )
    }

    pub fn translate_document_resolved(
        &self,
        input: &str,
        source_language: &str,
        target_language: &str,
    ) -> Result<DocumentTranslation, DocumentServiceError> {
        let _ = (input, source_language, target_language);
        Err(DocumentServiceError::Translation(crate::document::translation::DocumentTranslationError::UnsupportedResolvedRewrite))
    }

    pub fn build_document_graph(
        &self,
        compilation: &DocumentCompilation,
    ) -> Result<DocumentGraph, DocumentGraphServiceError> {
        DocumentGraphService::new(self).build_from_compilation(compilation)
    }

    pub fn build_document_graph_with_options(
        &self,
        compilation: &DocumentCompilation,
        options: &DocumentGraphBuildOptions,
    ) -> Result<DocumentGraph, DocumentGraphServiceError> {
        DocumentGraphService::new(self).build_from_compilation_with_options(compilation, options)
    }

    pub fn compile_document_graph(
        &self,
        input: &str,
        source_language: &str,
    ) -> Result<DocumentGraph, DocumentGraphServiceError> {
        DocumentGraphService::new(self).compile_and_build(input, source_language)
    }

    pub fn compile_document_graph_with_options(
        &self,
        input: &str,
        source_language: &str,
        options: &DocumentGraphBuildOptions,
    ) -> Result<DocumentGraph, DocumentGraphServiceError> {
        DocumentGraphService::new(self).compile_and_build_with_options(input, source_language, options)
    }

    pub fn document_graph_json(
        &self,
        input: &str,
        source_language: &str,
        pretty: bool,
    ) -> Result<String, DocumentGraphServiceError> {
        let graph = self.compile_document_graph(input, source_language)?;
        if pretty {
            Ok(graph.to_pretty_json()?)
        } else {
            Ok(graph.to_canonical_json()?)
        }
    }

    pub fn resolve_document_graph(
        &self,
        graph: &DocumentGraph,
    ) -> Result<DocumentEntityResolution, DocumentEntityResolutionServiceError> {
        DocumentEntityResolutionService::new(self).resolve_from_graph(graph)
    }

    pub fn resolve_document_graph_with_options(
        &self,
        graph: &DocumentGraph,
        options: &DocumentEntityResolutionOptions,
    ) -> Result<DocumentEntityResolution, DocumentEntityResolutionServiceError> {
        DocumentEntityResolutionService::new(self).resolve_from_graph_with_options(graph, options)
    }

    pub fn compile_resolved_document(
        &self,
        input: &str,
        source_language: &str,
    ) -> Result<DocumentEntityResolution, DocumentEntityResolutionServiceError> {
        DocumentEntityResolutionService::new(self).compile_and_resolve(input, source_language)
    }

    pub fn compile_resolved_document_with_options(
        &self,
        input: &str,
        source_language: &str,
        options: &DocumentEntityResolutionOptions,
    ) -> Result<DocumentEntityResolution, DocumentEntityResolutionServiceError> {
        DocumentEntityResolutionService::new(self).compile_and_resolve_with_options(
            input,
            source_language,
            options,
        )
    }

    pub fn resolve_document_temporal_discourse(
        &self,
        compilation: &DocumentCompilation,
        graph: &DocumentGraph,
        resolution: Option<&DocumentEntityResolution>,
    ) -> Result<DocumentTemporalDiscourse, DocumentTemporalDiscourseError> {
        DocumentTemporalDiscourseService::new(self).resolve(compilation, graph, resolution)
    }

    pub fn resolve_document_temporal_discourse_with_options(
        &self,
        compilation: &DocumentCompilation,
        graph: &DocumentGraph,
        resolution: Option<&DocumentEntityResolution>,
        options: &DocumentTemporalDiscourseOptions,
    ) -> Result<DocumentTemporalDiscourse, DocumentTemporalDiscourseError> {
        DocumentTemporalDiscourseService::with_options(options.clone())
            .resolve(compilation, graph, resolution)
    }

    pub fn compile_document_temporal_discourse(
        &self,
        input: &str,
        source_language: &str,
    ) -> Result<DocumentTemporalDiscourse, DocumentTemporalDiscourseError> {
        let compilation = self.compile_document(input, source_language)?;
        let graph = self.build_document_graph(&compilation)?;
        let resolution = self.resolve_document_graph(&graph)?;
        self.resolve_document_temporal_discourse(&compilation, &graph, Some(&resolution))
    }

    pub fn compile_document_temporal_discourse_with_options(
        &self,
        input: &str,
        source_language: &str,
        options: &DocumentTemporalDiscourseOptions,
    ) -> Result<DocumentTemporalDiscourse, DocumentTemporalDiscourseError> {
        let compilation = self.compile_document(input, source_language)?;
        let graph = self.build_document_graph(&compilation)?;
        let resolution = self.resolve_document_graph(&graph)?;
        self.resolve_document_temporal_discourse_with_options(
            &compilation,
            &graph,
            Some(&resolution),
            options,
        )
    }

    pub fn document_resolution_json(
        &self,
        input: &str,
        source_language: &str,
        pretty: bool,
    ) -> Result<String, DocumentEntityResolutionServiceError> {
        DocumentEntityResolutionService::new(self).resolve_json(input, source_language, pretty)
    }

    pub fn extract_document_knowledge(
        &self,
        compilation: &DocumentCompilation,
        graph: &DocumentGraph,
        resolution: Option<&DocumentEntityResolution>,
        temporal: Option<&DocumentTemporalDiscourse>,
    ) -> Result<DocumentKnowledgeExtraction, DocumentKnowledgeError> {
        DocumentKnowledgeService::default().extract(compilation, graph, resolution, temporal)
    }

    pub fn extract_document_knowledge_with_options(
        &self,
        compilation: &DocumentCompilation,
        graph: &DocumentGraph,
        resolution: Option<&DocumentEntityResolution>,
        temporal: Option<&DocumentTemporalDiscourse>,
        options: &DocumentKnowledgeExtractionOptions,
    ) -> Result<DocumentKnowledgeExtraction, DocumentKnowledgeError> {
        DocumentKnowledgeService::with_options(options.clone())
            .extract(compilation, graph, resolution, temporal)
    }

    pub fn answer_document_query(
        &self,
        query: QueryInterlingua,
        knowledge: &DocumentKnowledgeExtraction,
    ) -> Result<DocumentAnswer, QueryError> {
        QueryService::answer_document_query(query, knowledge)
    }
}
