use crate::document::{Document, DocumentSentence, DocumentStructureError, SourceSpan};

pub(crate) struct ValidatedSentenceContent<'a> {
    pub span: SourceSpan,
    pub text: &'a str,
    pub sha256: String,
}

pub(crate) fn validated_sentence_content<'a>(
    document: &'a Document,
    sentence: &DocumentSentence,
) -> Result<ValidatedSentenceContent<'a>, Vec<DocumentStructureError>> {
    let Some(span) = sentence.content_span.span else {
        return Err(vec![DocumentStructureError::NonExactSpan {
            id: sentence.id.as_str().to_string(),
            field: "content_span".to_string(),
        }]);
    };

    let text = span.slice(document.source()).map_err(|_| {
        vec![DocumentStructureError::InvalidSpan {
            id: sentence.id.as_str().to_string(),
            field: "content_span".to_string(),
        }]
    })?;

    Ok(ValidatedSentenceContent {
        span,
        text,
        sha256: crate::document::hash::sha256_text(text),
    })
}
