use crate::token::TokenKind;

#[allow(dead_code)]
pub(crate) fn is_sentence_terminator(kind: &TokenKind) -> bool {
    matches!(kind, TokenKind::Punctuation)
}
