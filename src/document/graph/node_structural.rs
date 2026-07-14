use crate::core::interlingua::LanguageId;
use crate::document::id::{DocumentBlockId, DocumentId, ParagraphId, SentenceId};
use crate::document::span::LocatedSpan;
use serde::{Deserialize, Serialize};

use super::model::DocumentBlockGraphKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentRootNode {
    pub id: super::id::GraphNodeId,
    pub document_id: DocumentId,
    pub source_language: LanguageId,
    pub source_sha256: String,
    pub source_len_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentBlockGraphNode {
    pub id: super::id::GraphNodeId,
    pub block_id: DocumentBlockId,
    pub ordinal: usize,
    pub kind: DocumentBlockGraphKind,
    pub span: LocatedSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentParagraphGraphNode {
    pub id: super::id::GraphNodeId,
    pub paragraph_id: ParagraphId,
    pub ordinal: usize,
    pub block_id: DocumentBlockId,
    pub span: LocatedSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSourceSentenceNode {
    pub id: super::id::GraphNodeId,
    pub sentence_id: SentenceId,
    pub paragraph_id: ParagraphId,
    pub document_ordinal: usize,
    pub paragraph_ordinal: usize,
    pub raw_span: LocatedSpan,
    pub content_span: LocatedSpan,
    pub compilation_status: crate::document::compilation::SentenceProcessingStatus,
    pub semantic_sentence_count: usize,
    pub diagnostic_codes: Vec<String>,
}
