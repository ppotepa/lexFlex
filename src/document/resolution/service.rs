use crate::api::LexFlexAPI;
use crate::document::graph::{DocumentGraph, DocumentGraphBuildOptions, DocumentGraphService};

use super::error::DocumentEntityResolutionServiceError;
use super::model::DocumentEntityResolution;
use super::options::DocumentEntityResolutionOptions;
use super::resolver::DocumentEntityResolver;

pub struct DocumentEntityResolutionService<'a> {
    api: &'a LexFlexAPI,
}

impl<'a> DocumentEntityResolutionService<'a> {
    pub fn new(api: &'a LexFlexAPI) -> Self {
        Self { api }
    }

    pub fn resolve_from_graph(
        &self,
        graph: &DocumentGraph,
    ) -> Result<DocumentEntityResolution, DocumentEntityResolutionServiceError> {
        DocumentEntityResolver::default()
            .resolve(graph)
            .map_err(DocumentEntityResolutionServiceError::Resolution)
    }

    pub fn resolve_from_graph_with_options(
        &self,
        graph: &DocumentGraph,
        options: &DocumentEntityResolutionOptions,
    ) -> Result<DocumentEntityResolution, DocumentEntityResolutionServiceError> {
        DocumentEntityResolver::with_options(options.clone())
            .resolve(graph)
            .map_err(DocumentEntityResolutionServiceError::Resolution)
    }

    pub fn compile_and_resolve(
        &self,
        input: &str,
        source_language: &str,
    ) -> Result<DocumentEntityResolution, DocumentEntityResolutionServiceError> {
        let graph = DocumentGraphService::new(self.api).compile_and_build(input, source_language)?;
        self.resolve_from_graph(&graph)
    }

    pub fn compile_and_resolve_with_options(
        &self,
        input: &str,
        source_language: &str,
        options: &DocumentEntityResolutionOptions,
    ) -> Result<DocumentEntityResolution, DocumentEntityResolutionServiceError> {
        let graph = DocumentGraphService::new(self.api).compile_and_build_with_options(
            input,
            source_language,
            &DocumentGraphBuildOptions::default(),
        )?;
        self.resolve_from_graph_with_options(&graph, options)
    }

    pub fn resolve_json(
        &self,
        input: &str,
        source_language: &str,
        pretty: bool,
    ) -> Result<String, DocumentEntityResolutionServiceError> {
        let resolution = self.compile_and_resolve(input, source_language)?;
        if pretty {
            Ok(resolution.to_pretty_json()?)
        } else {
            Ok(resolution.to_canonical_json()?)
        }
    }
}
