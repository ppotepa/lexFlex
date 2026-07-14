use crate::api::LexFlexAPI;
use crate::core::interlingua::LanguageId;
use crate::document::compilation::{
    DocumentCompilation, DocumentCompilationError, DocumentCompiler, LexFlexSentenceAnalyzer,
};
use crate::document::translation::{
    BestEffortDocumentTranslator, DocumentTranslation, DocumentTranslationError,
    DocumentTranslationOptions, LexFlexSentenceGenerator,
};
use crate::document::{
    Document, DocumentInput, DocumentSegmentationError, DocumentSegmentationOptions,
    DocumentSegmenter, LosslessDocumentSegmenter,
};

#[derive(Debug, thiserror::Error)]
pub enum DocumentServiceError {
    #[error("document segmentation failed: {0}")]
    Segmentation(#[from] DocumentSegmentationError),
    #[error("document compilation failed: {0}")]
    Compilation(#[from] DocumentCompilationError),
    #[error("document translation failed: {0}")]
    Translation(#[from] DocumentTranslationError),
}

pub struct DocumentService<'a> {
    api: &'a LexFlexAPI,
}

impl<'a> DocumentService<'a> {
    pub fn new(api: &'a LexFlexAPI) -> Self {
        Self { api }
    }

    pub fn segment(
        &self,
        input: &str,
        source_language: &str,
    ) -> Result<Document, DocumentServiceError> {
        self.segment_with_options(
            input,
            source_language,
            &DocumentSegmentationOptions::default(),
        )
    }

    pub fn segment_with_options(
        &self,
        input: &str,
        source_language: &str,
        options: &DocumentSegmentationOptions,
    ) -> Result<Document, DocumentServiceError> {
        let segmenter = LosslessDocumentSegmenter::default();
        Ok(segmenter.segment(
            DocumentInput {
                id: None,
                source_language: LanguageId::new(source_language),
                source: input.to_string(),
            },
            options,
        )?)
    }

    pub fn compile(
        &self,
        input: &str,
        source_language: &str,
    ) -> Result<DocumentCompilation, DocumentServiceError> {
        let document = self.segment(input, source_language)?;
        let analyzer = LexFlexSentenceAnalyzer::new(self.api);
        let compiler = DocumentCompiler::new(analyzer);
        compiler.compile(document).map_err(DocumentServiceError::Compilation)
    }

    pub fn translate_best_effort(
        &self,
        input: &str,
        source_language: &str,
        target_language: &str,
    ) -> Result<DocumentTranslation, DocumentServiceError> {
        self.translate_best_effort_with_options(
            input,
            source_language,
            target_language,
            &DocumentTranslationOptions::default(),
        )
    }

    pub fn translate_best_effort_with_options(
        &self,
        input: &str,
        source_language: &str,
        target_language: &str,
        options: &DocumentTranslationOptions,
    ) -> Result<DocumentTranslation, DocumentServiceError> {
        let compilation = self.compile(input, source_language)?;
        let generator = LexFlexSentenceGenerator::new(self.api);
        let translator =
            BestEffortDocumentTranslator::with_options(generator, options.clone());
        Ok(translator.translate(&compilation, LanguageId::new(target_language))?)
    }
}
