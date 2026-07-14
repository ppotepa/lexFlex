use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceSpanError {
    InvalidBounds { start: usize, end: usize },
    OutOfBounds { end: usize, len: usize },
    NotUtf8Boundary { offset: usize },
}

impl std::fmt::Display for SourceSpanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidBounds { start, end } => {
                write!(f, "invalid span bounds: start={start} end={end}")
            }
            Self::OutOfBounds { end, len } => write!(f, "span exceeds text length: end={end} len={len}"),
            Self::NotUtf8Boundary { offset } => write!(f, "offset is not a UTF-8 boundary: {offset}"),
        }
    }
}

impl std::error::Error for SourceSpanError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

impl SourceSpan {
    pub fn new(start: usize, end: usize) -> Result<Self, SourceSpanError> {
        if start > end {
            return Err(SourceSpanError::InvalidBounds { start, end });
        }
        Ok(Self { start, end })
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn contains(&self, offset: usize) -> bool {
        (self.start..self.end).contains(&offset)
    }

    pub fn contains_span(&self, other: &Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn try_union(&self, other: &Self, text: &str) -> Result<Self, SourceSpanError> {
        self.validate_for(text)?;
        other.validate_for(text)?;
        let span = self.union(other);
        span.validate_for(text)?;
        Ok(span)
    }

    pub fn shift(&self, offset: isize) -> Option<Self> {
        let start = self.start.checked_add_signed(offset)?;
        let end = self.end.checked_add_signed(offset)?;
        (start <= end).then_some(Self { start, end })
    }

    pub fn validate_for(&self, text: &str) -> Result<(), SourceSpanError> {
        if self.start > self.end {
            return Err(SourceSpanError::InvalidBounds {
                start: self.start,
                end: self.end,
            });
        }
        if self.end > text.len() {
            return Err(SourceSpanError::OutOfBounds {
                end: self.end,
                len: text.len(),
            });
        }
        if !text.is_char_boundary(self.start) {
            return Err(SourceSpanError::NotUtf8Boundary { offset: self.start });
        }
        if !text.is_char_boundary(self.end) {
            return Err(SourceSpanError::NotUtf8Boundary { offset: self.end });
        }
        Ok(())
    }

    pub fn slice<'a>(&self, text: &'a str) -> Result<&'a str, SourceSpanError> {
        self.validate_for(text)?;
        Ok(&text[self.start..self.end])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanPrecision {
    Exact,
    SentenceScoped,
    ParagraphScoped,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocatedSpan {
    pub span: Option<SourceSpan>,
    pub precision: SpanPrecision,
}

impl LocatedSpan {
    pub fn exact(span: SourceSpan) -> Self {
        Self {
            span: Some(span),
            precision: SpanPrecision::Exact,
        }
    }

    pub fn sentence_scoped(span: Option<SourceSpan>) -> Self {
        Self {
            span,
            precision: SpanPrecision::SentenceScoped,
        }
    }

    pub fn paragraph_scoped(span: Option<SourceSpan>) -> Self {
        Self {
            span,
            precision: SpanPrecision::ParagraphScoped,
        }
    }

    pub fn unknown() -> Self {
        Self {
            span: None,
            precision: SpanPrecision::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineIndex {
    pub line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![0];
        let bytes = text.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            match bytes[index] {
                b'\n' => {
                    index += 1;
                    line_starts.push(index);
                }
                b'\r' => {
                    index += 1;
                    if index < bytes.len() && bytes[index] == b'\n' {
                        index += 1;
                    }
                    line_starts.push(index);
                }
                _ => index += 1,
            }
        }
        Self { line_starts }
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    pub fn line_span(&self, text: &str, line: usize) -> Option<SourceSpan> {
        if line == 0 || line > self.line_starts.len() {
            return None;
        }
        let start = self.line_starts[line - 1];
        let end = self.line_starts.get(line).copied().unwrap_or(text.len());
        SourceSpan::new(start, end).ok()
    }

    pub fn line_column_at(&self, text: &str, offset: usize) -> Option<LineColumn> {
        if offset > text.len() || !text.is_char_boundary(offset) {
            return None;
        }
        let line_index = self
            .line_starts
            .iter()
            .enumerate()
            .rev()
            .find(|(_, start)| **start <= offset)?
            .0;
        let line_start = self.line_starts[line_index];
        let line_end = self.line_starts.get(line_index + 1).copied().unwrap_or(text.len());
        let mut column = 1;
        for (byte_index, ch) in text[line_start..line_end].char_indices() {
            if line_start + byte_index == offset {
                return Some(LineColumn {
                    line: line_index + 1,
                    column,
                });
            }
            column += 1;
            if line_start + byte_index > offset {
                break;
            }
            let _ = ch;
        }
        if offset == line_end {
            Some(LineColumn {
                line: line_index + 1,
                column,
            })
        } else {
            None
        }
    }
}
