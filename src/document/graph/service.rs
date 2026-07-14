use crate::api::LexFlexAPI;
use crate::document::compilation::DocumentCompilation;

use super::compatibility::DefaultEntityCompatibility;
use super::{DocumentGraph, DocumentGraphBuildOptions, DocumentGraphBuilder, DocumentGraphServiceError};

pub struct DocumentGraphService<'a> {
    api: &'a LexFlexAPI,
}

impl<'a> DocumentGraphService<'a> {
    pub fn new(api: &'a LexFlexAPI) -> Self {
        Self { api }
    }

    pub fn build_from_compilation(&self, compilation: &DocumentCompilation) -> Result<DocumentGraph, DocumentGraphServiceError> {
        DocumentGraphBuilder::default()
            .build(compilation)
            .map_err(DocumentGraphServiceError::Build)
    }

    pub fn build_from_compilation_with_options(
        &self,
        compilation: &DocumentCompilation,
        options: &DocumentGraphBuildOptions,
    ) -> Result<DocumentGraph, DocumentGraphServiceError> {
        DocumentGraphBuilder::<DefaultEntityCompatibility>::with_options(options.clone())
            .build(compilation)
            .map_err(DocumentGraphServiceError::Build)
    }

    pub fn compile_and_build(
        &self,
        input: &str,
        source_language: &str,
    ) -> Result<DocumentGraph, DocumentGraphServiceError> {
        let compilation = self.api.compile_document(input, source_language)?;
        self.build_from_compilation(&compilation)
    }

    pub fn compile_and_build_with_options(
        &self,
        input: &str,
        source_language: &str,
        options: &DocumentGraphBuildOptions,
    ) -> Result<DocumentGraph, DocumentGraphServiceError> {
        let compilation = self.api.compile_document(input, source_language)?;
        self.build_from_compilation_with_options(&compilation, options)
    }

    pub fn compile_and_build_json(
        &self,
        input: &str,
        source_language: &str,
    ) -> Result<String, DocumentGraphServiceError> {
        let graph = self.compile_and_build(input, source_language)?;
        Ok(graph.to_canonical_json()?)
    }
}
