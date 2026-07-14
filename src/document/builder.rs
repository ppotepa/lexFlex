use crate::document::id::{DocumentBlockId, DocumentId, DocumentIdFactory, ParagraphId, SentenceId};
use crate::document::model::{
    Document, DocumentBlock, DocumentBlockKind, DocumentInput, DocumentSentence, Paragraph,
};
use crate::document::reconstruction::DocumentReconstructor;
use crate::document::span::{LocatedSpan, SourceSpan, SpanPrecision};
use crate::document::validation::{DocumentStructureError, DocumentValidator};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentBuildError {
    UnknownParagraph { id: String },
    MissingSpan,
    NonExactSpan,
    EmptyBlockSpan,
    NonContiguousBlock {
        expected_start: usize,
        actual_start: usize,
    },
    SpanOutOfBounds {
        start: usize,
        end: usize,
        source_len: usize,
    },
    InvalidSpan { field: String, message: String },
    EmptySentenceContent,
    SentenceOutsideParagraph { paragraph_id: String },
    ContentOutsideRaw,
    NonContiguousSentence {
        paragraph_id: String,
        expected_start: usize,
        actual_start: usize,
    },
    DuplicateSentenceId { id: String },
}

impl std::fmt::Display for DocumentBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownParagraph { id } => write!(f, "unknown paragraph: {id}"),
            Self::MissingSpan => write!(f, "block span is missing"),
            Self::NonExactSpan => write!(f, "block span must be exact"),
            Self::EmptyBlockSpan => write!(f, "block span must not be empty"),
            Self::NonContiguousBlock {
                expected_start,
                actual_start,
            } => write!(
                f,
                "block span is not contiguous: expected start {expected_start}, got {actual_start}"
            ),
            Self::SpanOutOfBounds {
                start,
                end,
                source_len,
            } => write!(
                f,
                "block span {start}..{end} exceeds source length {source_len}"
            ),
            Self::InvalidSpan { field, message } => {
                write!(f, "invalid span for {field}: {message}")
            }
            Self::EmptySentenceContent => write!(f, "sentence content must not be empty"),
            Self::SentenceOutsideParagraph { paragraph_id } => {
                write!(f, "sentence span is outside paragraph {paragraph_id}")
            }
            Self::ContentOutsideRaw => write!(f, "sentence content is outside its raw span"),
            Self::NonContiguousSentence {
                paragraph_id,
                expected_start,
                actual_start,
            } => write!(
                f,
                "sentence in paragraph {paragraph_id} is not contiguous: expected start {expected_start}, got {actual_start}"
            ),
            Self::DuplicateSentenceId { id } => {
                write!(f, "duplicate generated sentence identifier: {id}")
            }
        }
    }
}

impl std::error::Error for DocumentBuildError {}

pub struct DocumentBuilder {
    document: Document,
}

impl DocumentBuilder {
    pub fn new(input: DocumentInput) -> Self {
        Self {
            document: Document::new(input),
        }
    }

    pub(crate) fn resume_validated(
        document: Document,
    ) -> Result<Self, Vec<DocumentStructureError>> {
        DocumentValidator::validate(&document)?;
        DocumentReconstructor::verify_reconstruction_only(&document)?;
        Ok(Self { document })
    }

    #[allow(dead_code)]
    pub(crate) fn from_document(
        document: Document,
    ) -> Result<Self, Vec<DocumentStructureError>> {
        Self::resume_validated(document)
    }

    pub fn source(&self) -> &str {
        &self.document.source
    }

    pub fn document_id(&self) -> &DocumentId {
        &self.document.id
    }

    pub fn next_expected_block_start(&self) -> usize {
        self.document
            .block_order
            .last()
            .and_then(|id| self.document.blocks.get(id))
            .and_then(|block| block.span.span)
            .map_or(0, |span| span.end)
    }

    pub fn push_paragraph(
        &mut self,
        span: LocatedSpan,
    ) -> Result<ParagraphId, DocumentBuildError> {
        self.validate_append_span(&span)?;
        let block_ordinal = self.document.block_order.len();
        let paragraph_ordinal = self.document.paragraphs.len();
        let block_id = DocumentIdFactory::block(&self.document.id, block_ordinal);
        let paragraph_id = DocumentIdFactory::paragraph(&self.document.id, paragraph_ordinal);
        if self.document.blocks.contains_key(&block_id)
            || self.document.paragraphs.contains_key(&paragraph_id)
        {
            return Err(DocumentBuildError::InvalidSpan {
                field: "paragraph.id".into(),
                message: "duplicate generated identifier".into(),
            });
        }
        let block = DocumentBlock {
            id: block_id.clone(),
            ordinal: block_ordinal,
            kind: DocumentBlockKind::Paragraph(paragraph_id.clone()),
            span: span.clone(),
        };
        let paragraph = Paragraph {
            id: paragraph_id.clone(),
            block_id: block_id.clone(),
            ordinal: paragraph_ordinal,
            span,
            sentence_order: Vec::new(),
        };
        self.document.block_order.push(block_id.clone());
        self.document.blocks.insert(block_id, block);
        self.document
            .paragraphs
            .insert(paragraph_id.clone(), paragraph);
        Ok(paragraph_id)
    }

    pub fn push_preserved_whitespace(
        &mut self,
        span: LocatedSpan,
    ) -> Result<DocumentBlockId, DocumentBuildError> {
        self.push_block_internal(DocumentBlockKind::PreservedWhitespace, span)
    }

    pub fn push_preserved_raw(
        &mut self,
        span: LocatedSpan,
    ) -> Result<DocumentBlockId, DocumentBuildError> {
        self.push_block_internal(DocumentBlockKind::PreservedRaw, span)
    }

    fn push_block_internal(
        &mut self,
        kind: DocumentBlockKind,
        span: LocatedSpan,
    ) -> Result<DocumentBlockId, DocumentBuildError> {
        self.validate_append_span(&span)?;
        let ordinal = self.document.block_order.len();
        let id = DocumentIdFactory::block(&self.document.id, ordinal);
        if self.document.blocks.contains_key(&id) {
            return Err(DocumentBuildError::InvalidSpan {
                field: "block.id".into(),
                message: "duplicate generated identifier".into(),
            });
        }
        let block = DocumentBlock {
            id: id.clone(),
            ordinal,
            kind,
            span,
        };
        self.document.block_order.push(id.clone());
        self.document.blocks.insert(id.clone(), block);
        Ok(id)
    }

    fn validate_append_span(&self, located: &LocatedSpan) -> Result<SourceSpan, DocumentBuildError> {
        let span = located.span.ok_or(DocumentBuildError::MissingSpan)?;
        if located.precision != SpanPrecision::Exact {
            return Err(DocumentBuildError::NonExactSpan);
        }
        if span.is_empty() {
            return Err(DocumentBuildError::EmptyBlockSpan);
        }
        if span.end > self.document.source.len() || span.start > span.end {
            return Err(DocumentBuildError::SpanOutOfBounds {
                start: span.start,
                end: span.end,
                source_len: self.document.source.len(),
            });
        }
        span.validate_for(&self.document.source)
            .map_err(|error| DocumentBuildError::InvalidSpan {
                field: "block.span".into(),
                message: error.to_string(),
            })?;
        let expected_start = self.next_expected_block_start();
        if span.start != expected_start {
            return Err(DocumentBuildError::NonContiguousBlock {
                expected_start,
                actual_start: span.start,
            });
        }
        Ok(span)
    }

    pub fn push_sentence(
        &mut self,
        paragraph_id: ParagraphId,
        raw_span: LocatedSpan,
        content_span: LocatedSpan,
    ) -> Result<SentenceId, DocumentBuildError> {
        validate_structural_span(&self.document.source, "sentence.raw_span", &raw_span)?;
        validate_structural_span(
            &self.document.source,
            "sentence.content_span",
            &content_span,
        )?;
        let raw = raw_span.span.expect("validated structural span has a value");
        let content = content_span
            .span
            .expect("validated structural span has a value");
        if content.is_empty()
            || content
                .slice(&self.document.source)
                .map_or(true, |text| text.trim().is_empty())
        {
            return Err(DocumentBuildError::EmptySentenceContent);
        }
        if !raw.contains_span(&content) {
            return Err(DocumentBuildError::ContentOutsideRaw);
        }
        let paragraph = self
            .document
            .paragraphs
            .get(&paragraph_id)
            .ok_or_else(|| DocumentBuildError::UnknownParagraph {
                id: paragraph_id.as_str().to_string(),
            })?;
        let paragraph_span = paragraph
            .span
            .span
            .expect("builder paragraphs always have exact spans");
        if !paragraph_span.contains_span(&raw) {
            return Err(DocumentBuildError::SentenceOutsideParagraph {
                paragraph_id: paragraph_id.to_string(),
            });
        }
        let expected_start = paragraph
            .sentence_order
            .last()
            .and_then(|id| self.document.sentences.get(id))
            .and_then(|sentence| sentence.raw_span.span)
            .map_or(paragraph_span.start, |span| span.end);
        if raw.start != expected_start {
            return Err(DocumentBuildError::NonContiguousSentence {
                paragraph_id: paragraph_id.to_string(),
                expected_start,
                actual_start: raw.start,
            });
        }
        let sentence_id = DocumentIdFactory::sentence(
            &self.document.id,
            paragraph.ordinal,
            paragraph.sentence_order.len(),
        );
        if self.document.sentences.contains_key(&sentence_id) {
            return Err(DocumentBuildError::DuplicateSentenceId {
                id: sentence_id.to_string(),
            });
        }
        let sentence = DocumentSentence {
            id: sentence_id.clone(),
            paragraph_id: paragraph_id.clone(),
            document_ordinal: self.document.sentences.len(),
            paragraph_ordinal: paragraph.sentence_order.len(),
            raw_span,
            content_span,
        };
        let paragraph = self
            .document
            .paragraphs
            .get_mut(&paragraph_id)
            .expect("paragraph was checked before mutation");
        paragraph.sentence_order.push(sentence_id.clone());
        self.document.sentences.insert(sentence_id.clone(), sentence);
        Ok(sentence_id)
    }

    pub fn finish(self) -> Result<Document, Vec<DocumentStructureError>> {
        DocumentValidator::validate(&self.document)?;
        DocumentReconstructor::verify_reconstruction_only(&self.document)?;
        Ok(self.document)
    }
}

fn validate_structural_span(
    source: &str,
    field: &str,
    span: &LocatedSpan,
) -> Result<(), DocumentBuildError> {
    let raw = span.span.ok_or_else(|| DocumentBuildError::InvalidSpan {
        field: field.to_string(),
        message: "missing span".into(),
    })?;
    if span.precision != SpanPrecision::Exact {
        return Err(DocumentBuildError::InvalidSpan {
            field: field.to_string(),
            message: "span precision must be exact".into(),
        });
    }
    raw.validate_for(source)
        .map_err(|error| DocumentBuildError::InvalidSpan {
            field: field.to_string(),
            message: error.to_string(),
        })
}
