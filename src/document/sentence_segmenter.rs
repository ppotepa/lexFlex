use crate::document::builder::DocumentBuilder;
use crate::document::model::{Document, DocumentInput, DocumentSegmentationOptions};
use crate::document::segmentation_rules::SegmentationRules;
use crate::document::segmenter::{
    DocumentSegmentationError, DocumentSegmenter, LosslessParagraphSegmenter,
};
use crate::document::span::{LocatedSpan, SourceSpan};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SentenceBoundaryKind {
    TerminalPunctuation,
    EndOfParagraph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SentenceRange {
    pub raw_span: SourceSpan,
    pub content_span: SourceSpan,
    pub boundary_kind: SentenceBoundaryKind,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SentenceSegmentationError {
    #[error("paragraph span is invalid: {0}")]
    InvalidParagraphSpan(String),
    #[error("sentence ranges do not partition paragraph {start}..{end}")]
    InvalidPartition { start: usize, end: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentenceRangeSegmenter {
    rules: SegmentationRules,
}

impl SentenceRangeSegmenter {
    pub fn new(rules: SegmentationRules) -> Self {
        Self { rules }
    }

    pub fn rules(&self) -> &SegmentationRules {
        &self.rules
    }

    pub fn segment(
        &self,
        source: &str,
        paragraph_span: SourceSpan,
    ) -> Result<Vec<SentenceRange>, SentenceSegmentationError> {
        self.segment_paragraph(
            source,
            paragraph_span,
            crate::document::SentenceSegmentationMode::Conservative,
            &self.rules,
        )
    }

    pub fn segment_paragraph(
        &self,
        source: &str,
        paragraph_span: SourceSpan,
        mode: crate::document::SentenceSegmentationMode,
        rules: &SegmentationRules,
    ) -> Result<Vec<SentenceRange>, SentenceSegmentationError> {
        paragraph_span.validate_for(source).map_err(|error| {
            SentenceSegmentationError::InvalidParagraphSpan(error.to_string())
        })?;
        let rules = match mode {
            crate::document::SentenceSegmentationMode::Conservative => rules.clone(),
            crate::document::SentenceSegmentationMode::ExistingEngineCompatible => {
                let mut compatible = rules.clone();
                compatible.abbreviations.clear();
                compatible
            }
        };
        let mut ranges = Vec::new();
        let mut raw_start = paragraph_span.start;
        let mut content_start = skip_whitespace(source, raw_start, paragraph_span.end);
        let mut cursor = content_start;

        while cursor < paragraph_span.end {
            let (character, next) = char_at(source, cursor).expect("cursor is a UTF-8 boundary");
            if rules.terminal_characters.contains(&character)
                && self.is_sentence_terminal(source, cursor, character, paragraph_span, &rules)
            {
                let content_end = self.consume_terminal_and_closers(source, cursor, paragraph_span.end);
                let next_content = skip_whitespace(source, content_end, paragraph_span.end);
                let raw_end = if next_content < paragraph_span.end {
                    next_content
                } else {
                    paragraph_span.end
                };
                ranges.push(SentenceRange {
                    raw_span: SourceSpan { start: raw_start, end: raw_end },
                    content_span: SourceSpan { start: content_start, end: content_end },
                    boundary_kind: SentenceBoundaryKind::TerminalPunctuation,
                });
                raw_start = raw_end;
                content_start = next_content;
                cursor = next_content;
            } else {
                cursor = next;
            }
        }

        if content_start < paragraph_span.end {
            let content_end = trim_whitespace_end(source, content_start, paragraph_span.end);
            if content_start < content_end {
                ranges.push(SentenceRange {
                    raw_span: SourceSpan { start: raw_start, end: paragraph_span.end },
                    content_span: SourceSpan { start: content_start, end: content_end },
                    boundary_kind: SentenceBoundaryKind::EndOfParagraph,
                });
            }
        }
        validate_partition(paragraph_span, &ranges)?;
        Ok(ranges)
    }

    fn is_sentence_terminal(
        &self,
        source: &str,
        offset: usize,
        character: char,
        paragraph: SourceSpan,
        rules: &SegmentationRules,
    ) -> bool {
        if character != '.' {
            return true;
        }
        let next_offset = offset + character.len_utf8();
        if source[next_offset..paragraph.end].starts_with('.') {
            return true;
        }
        let previous = previous_char(source, paragraph.start, offset);
        let next = char_at_before(source, next_offset, paragraph.end);
        if previous.is_some_and(char::is_alphanumeric)
            && next.is_some_and(char::is_alphanumeric)
        {
            return false;
        }
        let token = token_ending_at(source, paragraph.start, next_offset).to_lowercase();
        if rules.abbreviations.contains(&token) {
            return false;
        }
        let stem = token.trim_end_matches('.');
        if stem.chars().count() == 1
            && stem.chars().all(char::is_alphabetic)
            && is_initial_continuation(source, next_offset, paragraph.end)
        {
            return false;
        }
        true
    }

    fn consume_terminal_and_closers(&self, source: &str, start: usize, end: usize) -> usize {
        let mut cursor = start;
        while let Some((character, next)) = char_at_before_with_end(source, cursor, end) {
            if self.rules.terminal_characters.contains(&character) {
                cursor = next;
            } else {
                break;
            }
        }
        while let Some((character, next)) = char_at_before_with_end(source, cursor, end) {
            if self.rules.closing_characters.contains(&character) {
                cursor = next;
            } else {
                break;
            }
        }
        cursor
    }
}

impl Default for SentenceRangeSegmenter {
    fn default() -> Self {
        Self::new(SegmentationRules::default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct LosslessDocumentSegmenter;

impl DocumentSegmenter for LosslessDocumentSegmenter {
    fn segment(
        &self,
        input: DocumentInput,
        options: &DocumentSegmentationOptions,
    ) -> Result<Document, DocumentSegmentationError> {
        let paragraph_document = LosslessParagraphSegmenter.segment(input, options)?;
        let rules = SegmentationRules::for_language(paragraph_document.source_language());
        let range_segmenter = SentenceRangeSegmenter::new(rules);
        let paragraphs = paragraph_document
            .ordered_paragraphs()
            .into_iter()
            .map(|paragraph| (paragraph.id.clone(), paragraph.span.span))
            .collect::<Vec<_>>();
        let mut builder = DocumentBuilder::resume_validated(paragraph_document)
            .map_err(|errors| DocumentSegmentationError::InvalidStructure { errors })?;
        for (paragraph_id, span) in paragraphs {
            let span = span.ok_or_else(|| {
                DocumentSegmentationError::InternalInvariant(format!(
                    "paragraph {paragraph_id} has no exact span"
                ))
            })?;
            for range in range_segmenter.segment_paragraph(
                builder.source(),
                span,
                options.sentence_mode,
                range_segmenter.rules(),
            )? {
                builder.push_sentence(
                    paragraph_id.clone(),
                    LocatedSpan::exact(range.raw_span),
                    LocatedSpan::exact(range.content_span),
                )?;
            }
        }
        builder
            .finish()
            .map_err(|errors| DocumentSegmentationError::InvalidStructure { errors })
    }
}

fn char_at(source: &str, offset: usize) -> Option<(char, usize)> {
    let character = source.get(offset..)?.chars().next()?;
    Some((character, offset + character.len_utf8()))
}

fn char_at_before(source: &str, offset: usize, end: usize) -> Option<char> {
    (offset < end).then(|| source[offset..end].chars().next()).flatten()
}

fn char_at_before_with_end(source: &str, offset: usize, end: usize) -> Option<(char, usize)> {
    if offset >= end {
        None
    } else {
        char_at(source, offset).filter(|(_, next)| *next <= end)
    }
}

fn previous_char(source: &str, start: usize, offset: usize) -> Option<char> {
    (start < offset).then(|| source[start..offset].chars().next_back()).flatten()
}

fn skip_whitespace(source: &str, mut offset: usize, end: usize) -> usize {
    while let Some((character, next)) = char_at_before_with_end(source, offset, end) {
        if !character.is_whitespace() {
            break;
        }
        offset = next;
    }
    offset
}

fn trim_whitespace_end(source: &str, start: usize, mut end: usize) -> usize {
    while start < end {
        let Some(character) = source[start..end].chars().next_back() else { break };
        if !character.is_whitespace() {
            break;
        }
        end -= character.len_utf8();
    }
    end
}

fn is_initial_continuation(source: &str, start: usize, end: usize) -> bool {
    let token_start = skip_whitespace(source, start, end);
    let mut cursor = token_start;
    let mut letters = 0;
    while let Some((character, next)) = char_at_before_with_end(source, cursor, end) {
        if !character.is_alphabetic() {
            break;
        }
        letters += 1;
        cursor = next;
    }
    letters > 1 || (letters == 1 && char_at_before(source, cursor, end) == Some('.'))
}

fn token_ending_at(source: &str, start: usize, end: usize) -> &str {
    let mut token_start = end;
    for (offset, character) in source[start..end].char_indices().rev() {
        if character.is_whitespace() || matches!(character, '(' | '[' | '{' | '"' | '“' | '„') {
            break;
        }
        token_start = start + offset;
    }
    &source[token_start..end]
}

fn validate_partition(
    paragraph: SourceSpan,
    ranges: &[SentenceRange],
) -> Result<(), SentenceSegmentationError> {
    if ranges.is_empty() {
        return Err(SentenceSegmentationError::InvalidPartition {
            start: paragraph.start,
            end: paragraph.end,
        });
    }
    let mut cursor = paragraph.start;
    for range in ranges {
        if range.raw_span.start != cursor
            || range.raw_span.is_empty()
            || !range.raw_span.contains_span(&range.content_span)
            || range.content_span.is_empty()
        {
            return Err(SentenceSegmentationError::InvalidPartition {
                start: paragraph.start,
                end: paragraph.end,
            });
        }
        cursor = range.raw_span.end;
    }
    if cursor != paragraph.end {
        return Err(SentenceSegmentationError::InvalidPartition {
            start: paragraph.start,
            end: paragraph.end,
        });
    }
    Ok(())
}
