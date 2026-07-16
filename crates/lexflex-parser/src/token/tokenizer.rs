use crate::diagnostic::ParseError;
use crate::input::{ClauseMode, ParseInput};
use lexflex_model::SourceSpan;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    pub id: String,
    pub surface: String,
    pub normalized: String,
    pub span: SourceSpan,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenKind {
    Word,
    Punctuation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizationResult {
    pub tokens: Vec<Token>,
    pub mode: ClauseMode,
}

pub fn normalize_surface(surface: &str) -> String {
    surface
        .chars()
        .flat_map(char::to_lowercase)
        .map(|character| if character == '’' { '\'' } else { character })
        .collect()
}

pub fn tokenize(input: &ParseInput) -> Result<TokenizationResult, ParseError> {
    let mut tokens = Vec::new();
    let mut start = None;
    for (index, character) in input.text.char_indices() {
        if character.is_alphanumeric() || matches!(character, '\'' | '’' | '-') {
            start.get_or_insert(index);
            continue;
        }
        if let Some(begin) = start.take() {
            push_word(&mut tokens, &input.text, begin, index)?;
        }
        if is_punctuation(character) {
            let end = index + character.len_utf8();
            tokens.push(Token {
                id: format!("tok:{}", tokens.len()),
                surface: character.to_string(),
                normalized: character.to_string(),
                span: SourceSpan {
                    start: index as u64,
                    end: end as u64,
                },
                kind: TokenKind::Punctuation,
            });
        } else if !character.is_whitespace() {
            return Err(ParseError::UnsupportedCharacter { character, index });
        }
    }
    if let Some(begin) = start {
        push_word(&mut tokens, &input.text, begin, input.text.len())?;
    }
    if tokens.is_empty() {
        return Err(ParseError::EmptyInput);
    }
    let mode = match tokens.last().map(|token| token.surface.as_str()) {
        Some("?") => ClauseMode::Interrogative,
        _ => ClauseMode::Declarative,
    };
    Ok(TokenizationResult { tokens, mode })
}

fn push_word(
    tokens: &mut Vec<Token>,
    text: &str,
    start: usize,
    end: usize,
) -> Result<(), ParseError> {
    let surface = text[start..end].to_string();
    tokens.push(Token {
        id: format!("tok:{}", tokens.len()),
        normalized: normalize_surface(&surface),
        surface,
        span: SourceSpan {
            start: start as u64,
            end: end as u64,
        },
        kind: TokenKind::Word,
    });
    Ok(())
}

fn is_punctuation(character: char) -> bool {
    matches!(character, '.' | '?' | '!' | ',' | ':' | ';')
}
