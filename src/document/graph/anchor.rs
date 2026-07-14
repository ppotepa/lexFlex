use crate::core::graph::LinguisticGraph;
use crate::core::interlingua::PartOfSpeech;
use crate::document::model::{Document, DocumentSentence};
use crate::document::span::{LocatedSpan, SourceSpan};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MentionAnchor {
    Exact { span: SourceSpan },
    Discontinuous { spans: Vec<SourceSpan>, envelope: SourceSpan },
    SentenceScoped { span: SourceSpan },
    Unknown,
}

impl MentionAnchor {
    pub fn primary_span(&self) -> Option<SourceSpan> {
        match self {
            Self::Exact { span } => Some(*span),
            Self::Discontinuous { envelope, .. } => Some(*envelope),
            Self::SentenceScoped { span } => Some(*span),
            Self::Unknown => None,
        }
    }

    pub fn spans(&self) -> Vec<SourceSpan> {
        match self {
            Self::Exact { span } => vec![*span],
            Self::Discontinuous { spans, .. } => spans.clone(),
            Self::SentenceScoped { span } => vec![*span],
            Self::Unknown => vec![],
        }
    }

    pub fn is_exact(&self) -> bool {
        matches!(self, Self::Exact { .. })
    }

    pub fn is_sentence_scoped(&self) -> bool {
        matches!(self, Self::SentenceScoped { .. })
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MentionAnchorSource {
    LinguisticRealizingWords,
    ParentMentionInherited,
    SentenceFallbackNoGraph,
    SentenceFallbackNoFrameBinding,
    SentenceFallbackNoEntityBinding,
    SentenceFallbackInvalidLocalSpan,
    SentenceFallbackNoRealizingWords,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MentionAnchorError {
    MissingSentenceSpan,
    InvalidLocalBounds,
    LocalOutOfBounds,
    LocalNotUtf8Boundary,
    AbsoluteOverflow,
    AbsoluteOutOfBounds,
    AbsoluteNotContained,
}

pub fn local_word_span_to_document(
    document: &Document,
    sentence: &DocumentSentence,
    local: (usize, usize),
) -> Result<SourceSpan, MentionAnchorError> {
    let Some(sentence_span) = sentence.content_span.span else {
        return Err(MentionAnchorError::MissingSentenceSpan);
    };
    let text = document
        .sentence_text(&sentence.id)
        .ok_or(MentionAnchorError::MissingSentenceSpan)?;
    if local.0 > local.1 {
        return Err(MentionAnchorError::InvalidLocalBounds);
    }
    if local.1 > text.len() {
        return Err(MentionAnchorError::LocalOutOfBounds);
    }
    if !text.is_char_boundary(local.0) || !text.is_char_boundary(local.1) {
        return Err(MentionAnchorError::LocalNotUtf8Boundary);
    }
    let start = sentence_span
        .start
        .checked_add(local.0)
        .ok_or(MentionAnchorError::AbsoluteOverflow)?;
    let end = sentence_span
        .start
        .checked_add(local.1)
        .ok_or(MentionAnchorError::AbsoluteOverflow)?;
    let span = SourceSpan::new(start, end).map_err(|_| MentionAnchorError::AbsoluteOutOfBounds)?;
    if !sentence_span.contains_span(&span) {
        return Err(MentionAnchorError::AbsoluteNotContained);
    }
    Ok(span)
}

pub fn anchor_from_realizing_words(
    document: &Document,
    sentence: &DocumentSentence,
    graph: &LinguisticGraph,
    entity_node_id: crate::core::interlingua::NodeId,
) -> Result<MentionAnchor, MentionAnchorError> {
    let mut spans = graph
        .realizing_words_for_entity(entity_node_id)
        .into_iter()
        .filter(|word| word.pos != PartOfSpeech::Punctuation)
        .map(|word| local_word_span_to_document(document, sentence, word.span))
        .collect::<Result<Vec<_>, _>>()?;
    spans.sort_by_key(|span| (span.start, span.end));
    spans.dedup();
    if spans.is_empty() {
        return Err(MentionAnchorError::MissingSentenceSpan);
    }
    if spans.len() == 1 {
        return Ok(MentionAnchor::Exact { span: spans[0] });
    }
    let envelope = spans.iter().copied().reduce(|a, b| a.union(&b)).unwrap();
    Ok(MentionAnchor::Discontinuous { spans, envelope })
}

pub fn sentence_scoped_anchor(sentence: &DocumentSentence) -> MentionAnchor {
    sentence
        .content_span
        .span
        .map(|span| MentionAnchor::SentenceScoped { span })
        .unwrap_or(MentionAnchor::Unknown)
}

#[allow(dead_code)]
pub fn located_to_source(located: &LocatedSpan) -> Option<SourceSpan> {
    located.span
}
