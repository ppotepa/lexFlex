use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentIdError {
    Empty,
    TooLong { bytes: usize },
    InvalidCharacter { index: usize, character: char },
}

impl fmt::Display for DocumentIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "identifier must not be empty"),
            Self::TooLong { bytes } => write!(f, "identifier exceeds limit: {bytes}"),
            Self::InvalidCharacter { index, character } => {
                write!(f, "invalid identifier character at {index}: {character:?}")
            }
        }
    }
}

impl std::error::Error for DocumentIdError {}

macro_rules! document_id_type {
    ($name:ident, $max_bytes:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            pub fn new(id: &str) -> Result<Self, DocumentIdError> {
                validate_identifier(id, $max_bytes)?;
                Ok(Self(id.to_string()))
            }

            fn unchecked(id: String) -> Self {
                Self(id)
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = DocumentIdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::new(s)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

document_id_type!(DocumentId, 96);
document_id_type!(DocumentBlockId, 128);
document_id_type!(ParagraphId, 128);
document_id_type!(SentenceId, 128);
document_id_type!(DiagnosticId, 128);
document_id_type!(ProvenanceId, 128);

pub struct DocumentIdFactory;

impl DocumentIdFactory {
    pub fn from_source(language: &str, source: &str) -> DocumentId {
        let mut hasher = sha2::Sha256::new();
        use sha2::Digest;
        hasher.update(language.trim().as_bytes());
        hasher.update([0u8]);
        hasher.update(source.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        generated(
            format!("doc-{}-{}", sanitize(language), &hash[..32]),
            DocumentId::unchecked,
            96,
        )
    }

    pub fn block(document_id: &DocumentId, ordinal: usize) -> DocumentBlockId {
        generated(
            format!("{}:b{:04}", document_id.as_str(), ordinal),
            DocumentBlockId::unchecked,
            128,
        )
    }

    pub fn paragraph(document_id: &DocumentId, ordinal: usize) -> ParagraphId {
        generated(
            format!("{}:p{:04}", document_id.as_str(), ordinal),
            ParagraphId::unchecked,
            128,
        )
    }

    pub fn sentence(
        document_id: &DocumentId,
        paragraph_ordinal: usize,
        sentence_ordinal: usize,
    ) -> SentenceId {
        generated(
            format!(
                "{}:p{:04}:s{:04}",
                document_id.as_str(),
                paragraph_ordinal,
                sentence_ordinal
            ),
            SentenceId::unchecked,
            128,
        )
    }

    pub fn diagnostic(document_id: &DocumentId, ordinal: usize) -> DiagnosticId {
        generated(
            format!("{}:d{:04}", document_id.as_str(), ordinal),
            DiagnosticId::unchecked,
            128,
        )
    }

    pub fn provenance(document_id: &DocumentId, ordinal: usize) -> ProvenanceId {
        generated(
            format!("{}:pr{:04}", document_id.as_str(), ordinal),
            ProvenanceId::unchecked,
            128,
        )
    }

    pub fn sentence_provenance(sentence_id: &SentenceId, ordinal: usize) -> ProvenanceId {
        generated(
            format!("{}:prov:{ordinal:04}", sentence_id.as_str()),
            ProvenanceId::unchecked,
            128,
        )
    }
}

fn sanitize(language: &str) -> String {
    language
        .trim()
        .chars()
        .map(|ch| if valid_char(ch) { ch } else { '-' })
        .collect()
}

fn generated<T>(value: String, constructor: impl FnOnce(String) -> T, max_bytes: usize) -> T {
    debug_assert!(validate_identifier(&value, max_bytes).is_ok());
    constructor(value)
}

fn validate_identifier(id: &str, max_bytes: usize) -> Result<(), DocumentIdError> {
    if id.is_empty() {
        return Err(DocumentIdError::Empty);
    }
    let bytes = id.len();
    if bytes > max_bytes {
        return Err(DocumentIdError::TooLong { bytes });
    }
    for (index, character) in id.chars().enumerate() {
        if !valid_char(character) {
            return Err(DocumentIdError::InvalidCharacter { index, character });
        }
    }
    Ok(())
}

fn valid_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | ':' | '-')
}
