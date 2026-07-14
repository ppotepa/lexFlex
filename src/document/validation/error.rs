use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentStructureError {
    BlockOrderMismatch,
    MissingBlock { id: String },
    DuplicateBlockId { id: String },
    BlockOrdinalMismatch { id: String },
    UnreferencedBlock { id: String },
    MissingParagraph { id: String },
    ParagraphBlockMismatch { paragraph_id: String },
    ParagraphOrdinalMismatch { id: String },
    DuplicateParagraphReference { id: String },
    UnreferencedParagraph { id: String },
    MissingSentence { id: String },
    DuplicateSentenceId { id: String },
    SentenceParagraphMismatch { sentence_id: String },
    SentenceOrdinalMismatch { id: String },
    DuplicateDocumentSentenceOrdinal { ordinal: usize },
    DocumentSentenceOrdinalMismatch { id: String, expected: usize, actual: usize },
    Gap { start: usize, end: usize },
    Overlap { left_end: usize, right_start: usize },
    CoverageMismatch { end: usize, len: usize },
    EmptyBlockSpan { id: String },
    SpanOutsideParent { id: String, parent: String },
    ContentOutsideRaw { id: String },
    EmptySentenceContent { id: String },
    SentenceGapInParagraph { paragraph_id: String, start: usize, end: usize },
    SentenceOverlapInParagraph { paragraph_id: String, left_end: usize, right_start: usize },
    SentenceCoverageMismatch { paragraph_id: String, expected_end: usize, actual_end: usize },
    ParagraphBlockSpanMismatch { id: String },
    PreservedWhitespaceContainsContent { id: String },
    NonExactSpan { id: String, field: String },
    InvalidSpan { id: String, field: String },
    UnreferencedSentence { id: String },
    SourceHashMismatch,
}

impl std::fmt::Display for DocumentStructureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BlockOrderMismatch => write!(f, "block order does not match block map"),
            Self::MissingBlock { id } => write!(f, "missing block: {id}"),
            Self::DuplicateBlockId { id } => write!(f, "duplicate block id: {id}"),
            Self::BlockOrdinalMismatch { id } => write!(f, "block ordinal mismatch: {id}"),
            Self::UnreferencedBlock { id } => write!(f, "unreferenced block: {id}"),
            Self::MissingParagraph { id } => write!(f, "missing paragraph: {id}"),
            Self::ParagraphBlockMismatch { paragraph_id } => write!(f, "paragraph block mismatch: {paragraph_id}"),
            Self::ParagraphOrdinalMismatch { id } => write!(f, "paragraph ordinal mismatch: {id}"),
            Self::DuplicateParagraphReference { id } => write!(f, "duplicate paragraph reference: {id}"),
            Self::UnreferencedParagraph { id } => write!(f, "unreferenced paragraph: {id}"),
            Self::MissingSentence { id } => write!(f, "missing sentence: {id}"),
            Self::DuplicateSentenceId { id } => write!(f, "duplicate sentence id: {id}"),
            Self::SentenceParagraphMismatch { sentence_id } => write!(f, "sentence paragraph mismatch: {sentence_id}"),
            Self::SentenceOrdinalMismatch { id } => write!(f, "sentence ordinal mismatch: {id}"),
            Self::DuplicateDocumentSentenceOrdinal { ordinal } => write!(f, "duplicate document sentence ordinal: {ordinal}"),
            Self::DocumentSentenceOrdinalMismatch { id, expected, actual } => write!(f, "document sentence ordinal mismatch: {id} expected={expected} actual={actual}"),
            Self::Gap { start, end } => write!(f, "gap detected: {start}..{end}"),
            Self::Overlap { left_end, right_start } => write!(f, "overlap detected: {left_end} > {right_start}"),
            Self::CoverageMismatch { end, len } => write!(f, "coverage mismatch: end={end} len={len}"),
            Self::EmptyBlockSpan { id } => write!(f, "empty block span: {id}"),
            Self::SpanOutsideParent { id, parent } => write!(f, "span outside parent: {id} in {parent}"),
            Self::ContentOutsideRaw { id } => write!(f, "content outside raw span: {id}"),
            Self::EmptySentenceContent { id } => write!(f, "empty sentence content: {id}"),
            Self::SentenceGapInParagraph { paragraph_id, start, end } => write!(f, "sentence gap in paragraph {paragraph_id}: {start}..{end}"),
            Self::SentenceOverlapInParagraph { paragraph_id, left_end, right_start } => write!(f, "sentence overlap in paragraph {paragraph_id}: {left_end} > {right_start}"),
            Self::SentenceCoverageMismatch { paragraph_id, expected_end, actual_end } => write!(f, "sentence coverage mismatch in paragraph {paragraph_id}: expected={expected_end} actual={actual_end}"),
            Self::ParagraphBlockSpanMismatch { id } => write!(f, "paragraph/block span mismatch: {id}"),
            Self::PreservedWhitespaceContainsContent { id } => write!(f, "preserved whitespace contains content: {id}"),
            Self::NonExactSpan { id, field } => write!(f, "non-exact span: {id}.{field}"),
            Self::InvalidSpan { id, field } => write!(f, "invalid span: {id}.{field}"),
            Self::UnreferencedSentence { id } => write!(f, "unreferenced sentence: {id}"),
            Self::SourceHashMismatch => write!(f, "source hash mismatch"),
        }
    }
}

impl std::error::Error for DocumentStructureError {}

pub(super) fn sort_errors(errors: &mut [DocumentStructureError]) {
    errors.sort_by_key(|error| format!("{error:?}"));
}
