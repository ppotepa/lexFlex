use crate::core::interlingua::LanguageId;
use crate::document::id::{DocumentBlockId, DocumentId, DocumentIdFactory, ParagraphId, SentenceId};
use crate::document::reconstruction::DocumentReconstructor;
use crate::document::span::LocatedSpan;
use crate::document::validation::DocumentStructureError;
use crate::document::validation::DocumentValidator;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub source_language: LanguageId,
    pub source: String,
    pub source_sha256: String,
    pub block_order: Vec<DocumentBlockId>,
    pub blocks: BTreeMap<DocumentBlockId, DocumentBlock>,
    pub paragraphs: BTreeMap<ParagraphId, Paragraph>,
    pub sentences: BTreeMap<SentenceId, DocumentSentence>,
}

impl Document {
    pub fn new(input: DocumentInput) -> Self {
        let id = input
            .id
            .unwrap_or_else(|| DocumentIdFactory::from_source(&input.source_language.0, &input.source));
        let source_sha256 = sha256_bytes(input.source.as_bytes());
        Self {
            id,
            source_language: input.source_language,
            source: input.source,
            source_sha256,
            block_order: Vec::new(),
            blocks: BTreeMap::new(),
            paragraphs: BTreeMap::new(),
            sentences: BTreeMap::new(),
        }
    }

    pub fn id(&self) -> &DocumentId {
        &self.id
    }

    pub fn source_language(&self) -> &LanguageId {
        &self.source_language
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn source_sha256(&self) -> &str {
        &self.source_sha256
    }

    pub fn block_order(&self) -> &[DocumentBlockId] {
        &self.block_order
    }

    pub fn blocks(&self) -> &BTreeMap<DocumentBlockId, DocumentBlock> {
        &self.blocks
    }

    pub fn paragraphs(&self) -> &BTreeMap<ParagraphId, Paragraph> {
        &self.paragraphs
    }

    pub fn sentences(&self) -> &BTreeMap<SentenceId, DocumentSentence> {
        &self.sentences
    }

    pub fn block(&self, id: &DocumentBlockId) -> Option<&DocumentBlock> {
        self.blocks.get(id)
    }

    pub fn paragraph(&self, id: &ParagraphId) -> Option<&Paragraph> {
        self.paragraphs.get(id)
    }

    pub fn sentence(&self, id: &SentenceId) -> Option<&DocumentSentence> {
        self.sentences.get(id)
    }

    pub fn source_slice(&self, span: &LocatedSpan) -> Option<&str> {
        span.span.as_ref().and_then(|span| span.slice(&self.source).ok())
    }

    pub fn sentence_text(&self, id: &SentenceId) -> Option<&str> {
        self.sentences.get(id).and_then(|sentence| self.source_slice(&sentence.content_span))
    }

    pub fn sentence_raw_text(&self, id: &SentenceId) -> Option<&str> {
        self.sentences.get(id).and_then(|sentence| self.source_slice(&sentence.raw_span))
    }

    pub fn paragraph_text(&self, id: &ParagraphId) -> Option<&str> {
        self.paragraphs.get(id).and_then(|paragraph| self.source_slice(&paragraph.span))
    }

    pub fn ordered_paragraphs(&self) -> Vec<&Paragraph> {
        let mut paragraphs = self.paragraphs.values().collect::<Vec<_>>();
        paragraphs.sort_by(|a, b| a.ordinal.cmp(&b.ordinal).then_with(|| a.id.cmp(&b.id)));
        paragraphs
    }

    pub fn ordered_sentences(&self) -> Vec<&DocumentSentence> {
        let mut sentences = self.sentences.values().collect::<Vec<_>>();
        sentences.sort_by(|a, b| a.document_ordinal.cmp(&b.document_ordinal).then_with(|| a.id.cmp(&b.id)));
        sentences
    }

    pub fn validate_structure(&self) -> Result<(), Vec<DocumentStructureError>> {
        DocumentValidator::validate(self)
    }

    pub fn reconstruct(&self) -> Result<String, Vec<DocumentStructureError>> {
        DocumentReconstructor::reconstruct(self)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentBlock {
    pub id: DocumentBlockId,
    pub ordinal: usize,
    pub kind: DocumentBlockKind,
    pub span: LocatedSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentBlockKind {
    Paragraph(ParagraphId),
    PreservedWhitespace,
    PreservedRaw,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paragraph {
    pub id: ParagraphId,
    pub block_id: DocumentBlockId,
    pub ordinal: usize,
    pub span: LocatedSpan,
    pub sentence_order: Vec<SentenceId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSentence {
    pub id: SentenceId,
    pub paragraph_id: ParagraphId,
    pub document_ordinal: usize,
    pub paragraph_ordinal: usize,
    pub raw_span: LocatedSpan,
    pub content_span: LocatedSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentInput {
    pub id: Option<DocumentId>,
    pub source_language: LanguageId,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SentenceSegmentationMode {
    Conservative,
    ExistingEngineCompatible,
}

impl Default for SentenceSegmentationMode {
    fn default() -> Self {
        Self::Conservative
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSegmentationOptions {
    pub preserve_empty_blocks: bool,
    pub trim_document_edges: bool,
    pub sentence_mode: SentenceSegmentationMode,
}

impl Default for DocumentSegmentationOptions {
    fn default() -> Self {
        Self {
            preserve_empty_blocks: true,
            trim_document_edges: false,
            sentence_mode: SentenceSegmentationMode::Conservative,
        }
    }
}

pub(crate) fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
