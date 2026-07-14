use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentProfile {
    pub schema_version: u32,
    pub id: String,
    pub display_name: String,
    pub source_language: String,
    pub target_language: String,
    pub blocking_direction: bool,
    pub input: DocumentInputContract,
    pub length_tiers: Vec<DocumentLengthTier>,
    pub capabilities: Vec<DocumentCapabilityRequirement>,
    pub quality_gates: DocumentQualityGates,
    pub error_severity: Vec<DocumentErrorSeverityRule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentInputContract {
    pub format: DocumentInputFormat,
    pub encoding: String,
    pub preserve_paragraphs: bool,
    pub paragraph_separator: ParagraphSeparator,
    pub max_bytes: Option<usize>,
    pub allow_crlf: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentInputFormat {
    PlainText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParagraphSeparator {
    BlankLine,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentLengthTier {
    pub id: String,
    pub min_words: Option<usize>,
    pub max_words: Option<usize>,
    pub min_sentences: Option<usize>,
    pub max_sentences: Option<usize>,
    pub min_paragraphs: Option<usize>,
    pub max_paragraphs: Option<usize>,
    pub blocking: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentCapabilityStatus {
    Required,
    Tracked,
    Deferred,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentCapabilityRequirement {
    pub id: String,
    pub name: String,
    pub status: DocumentCapabilityStatus,
    pub blocking: bool,
    pub corpus_tag: String,
    pub minimum_cases: usize,
    pub target_chapter: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentQualityGates {
    pub require_deterministic_output: bool,
    pub require_non_empty_output: bool,
    pub max_fatal_errors: usize,
    pub required_name_preservation: ThresholdRatio,
    pub required_number_preservation: ThresholdRatio,
    pub required_negation_preservation: ThresholdRatio,
    pub required_reference_accuracy: ThresholdRatio,
    pub required_zero_anaphora_accuracy: ThresholdRatio,
    pub required_frame_signature_recall: ThresholdRatio,
    pub required_glossary_compliance: ThresholdRatio,
    pub required_paragraph_preservation: ThresholdRatio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThresholdRatio {
    milli: u16,
}

impl ThresholdRatio {
    pub const ZERO: Self = Self { milli: 0 };
    pub const ONE: Self = Self { milli: 1000 };

    pub const fn from_milli(milli: u16) -> Self {
        Self { milli }
    }

    pub const fn milli(self) -> u16 {
        self.milli
    }

    pub fn as_decimal_string(self) -> String {
        format!("{}.{:03}", self.milli / 1000, self.milli % 1000)
    }
}

impl Serialize for ThresholdRatio {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(f64::from(self.milli) / 1000.0)
    }
}

impl<'de> Deserialize<'de> for ThresholdRatio {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        if !value.is_finite() || value < 0.0 {
            return Err(serde::de::Error::custom("threshold must be a finite non-negative number"));
        }
        let milli = (value * 1000.0).round();
        if !(0.0..=(u16::MAX as f64)).contains(&milli) {
            return Err(serde::de::Error::custom("threshold milli value out of range"));
        }
        Ok(Self { milli: milli as u16 })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentErrorSeverity {
    Fatal,
    Major,
    Minor,
    Informational,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentErrorSeverityRule {
    pub category: String,
    pub severity: DocumentErrorSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentProfileValidationError {
    UnsupportedSchemaVersion { found: u32 },
    EmptyId,
    SameSourceAndTargetLanguage,
    DuplicateCapabilityId { id: String },
    DuplicateCapabilityTag { tag: String },
    RequiredCapabilityNotBlocking { id: String },
    DeferredCapabilityBlocking { id: String },
    InvalidThreshold { field: String, value_milli: u16 },
    InvalidLengthRange { tier: String, field: String },
    NoBlockingLengthTier,
    MissingErrorCategory { category: String },
}

impl std::fmt::Display for DocumentProfileValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion { found } => {
                write!(f, "unsupported schema version: {found}")
            }
            Self::EmptyId => write!(f, "profile id must not be empty"),
            Self::SameSourceAndTargetLanguage => {
                write!(f, "source and target languages must differ")
            }
            Self::DuplicateCapabilityId { id } => write!(f, "duplicate capability id: {id}"),
            Self::DuplicateCapabilityTag { tag } => write!(f, "duplicate capability tag: {tag}"),
            Self::RequiredCapabilityNotBlocking { id } => {
                write!(f, "required capability must be blocking: {id}")
            }
            Self::DeferredCapabilityBlocking { id } => {
                write!(f, "deferred capability must not be blocking: {id}")
            }
            Self::InvalidThreshold { field, value_milli } => {
                write!(f, "invalid threshold {field}={}.{:03}", value_milli / 1000, value_milli % 1000)
            }
            Self::InvalidLengthRange { tier, field } => {
                write!(f, "invalid length range in {tier}.{field}")
            }
            Self::NoBlockingLengthTier => write!(f, "at least one blocking length tier required"),
            Self::MissingErrorCategory { category } => {
                write!(f, "missing error category: {category}")
            }
        }
    }
}

impl std::error::Error for DocumentProfileValidationError {}

impl DocumentProfile {
    pub fn validate(&self) -> Result<(), Vec<DocumentProfileValidationError>> {
    let mut errors = Vec::new();

        if self.schema_version != 1 {
            errors.push(DocumentProfileValidationError::UnsupportedSchemaVersion {
                found: self.schema_version,
            });
        }
        if self.id.trim().is_empty() {
            errors.push(DocumentProfileValidationError::EmptyId);
        }
        if self.source_language == self.target_language {
            errors.push(DocumentProfileValidationError::SameSourceAndTargetLanguage);
        }

        let mut seen_ids = BTreeSet::new();
        let mut seen_tags = BTreeSet::new();
        for cap in &self.capabilities {
            if !seen_ids.insert(cap.id.clone()) {
                errors.push(DocumentProfileValidationError::DuplicateCapabilityId {
                    id: cap.id.clone(),
                });
            }
            if !seen_tags.insert(cap.corpus_tag.clone()) {
                errors.push(DocumentProfileValidationError::DuplicateCapabilityTag {
                    tag: cap.corpus_tag.clone(),
                });
            }
            match cap.status {
                DocumentCapabilityStatus::Required if !cap.blocking => {
                    errors.push(DocumentProfileValidationError::RequiredCapabilityNotBlocking {
                        id: cap.id.clone(),
                    });
                }
                DocumentCapabilityStatus::Deferred if cap.blocking => {
                    errors.push(DocumentProfileValidationError::DeferredCapabilityBlocking {
                        id: cap.id.clone(),
                    });
                }
                _ => {}
            }
        }

        for (field, value) in threshold_fields(&self.quality_gates) {
            if value.milli() > ThresholdRatio::ONE.milli() {
                errors.push(DocumentProfileValidationError::InvalidThreshold {
                    field: field.to_string(),
                    value_milli: value.milli(),
                });
            }
        }

        let mut has_blocking_tier = false;
        for tier in &self.length_tiers {
            if tier.blocking {
                has_blocking_tier = true;
            }
            validate_length_range(tier, "min_words", tier.min_words, tier.max_words, &mut errors);
            validate_length_range(
                tier,
                "min_sentences",
                tier.min_sentences,
                tier.max_sentences,
                &mut errors,
            );
            validate_length_range(
                tier,
                "min_paragraphs",
                tier.min_paragraphs,
                tier.max_paragraphs,
                &mut errors,
            );
        }
        if !has_blocking_tier {
            errors.push(DocumentProfileValidationError::NoBlockingLengthTier);
        }

        let required_categories = [
            "panic",
            "empty_output",
            "missing_paragraph",
            "hallucinated_critical_fact",
            "lost_negation",
            "changed_number",
            "role_swap",
            "wrong_coreference",
            "wrong_tense",
            "wrong_frame",
            "terminology_inconsistent",
            "article",
            "agreement",
            "word_order",
            "punctuation",
        ];
        let seen_categories: BTreeSet<_> = self
            .error_severity
            .iter()
            .map(|rule| rule.category.clone())
            .collect();
        for category in required_categories {
            if !seen_categories.contains(category) {
                errors.push(DocumentProfileValidationError::MissingErrorCategory {
                    category: category.to_string(),
                });
            }
        }

        sort_errors(&mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn capability(&self, id: &str) -> Option<&DocumentCapabilityRequirement> {
        self.capabilities.iter().find(|cap| cap.id == id)
    }

    pub fn blocking_capabilities(
        &self,
    ) -> impl Iterator<Item = &DocumentCapabilityRequirement> {
        self.capabilities.iter().filter(|cap| cap.blocking)
    }

    pub fn length_tier(&self, id: &str) -> Option<&DocumentLengthTier> {
        self.length_tiers.iter().find(|tier| tier.id == id)
    }
}

fn threshold_fields(gates: &DocumentQualityGates) -> [(&'static str, ThresholdRatio); 8] {
    [
        ("required_name_preservation", gates.required_name_preservation),
        ("required_number_preservation", gates.required_number_preservation),
        ("required_negation_preservation", gates.required_negation_preservation),
        ("required_reference_accuracy", gates.required_reference_accuracy),
        ("required_zero_anaphora_accuracy", gates.required_zero_anaphora_accuracy),
        ("required_frame_signature_recall", gates.required_frame_signature_recall),
        ("required_glossary_compliance", gates.required_glossary_compliance),
        (
            "required_paragraph_preservation",
            gates.required_paragraph_preservation,
        ),
    ]
}

fn validate_length_range(
    tier: &DocumentLengthTier,
    field: &'static str,
    min: Option<usize>,
    max: Option<usize>,
    errors: &mut Vec<DocumentProfileValidationError>,
) {
    if let (Some(min), Some(max)) = (min, max) {
        if min > max {
            errors.push(DocumentProfileValidationError::InvalidLengthRange {
                tier: tier.id.clone(),
                field: field.to_string(),
            });
        }
    }
}

fn sort_errors(errors: &mut [DocumentProfileValidationError]) {
    errors.sort_by(|a, b| error_key(a).cmp(&error_key(b)));
}

fn error_key(err: &DocumentProfileValidationError) -> (u8, String, String) {
    match err {
        DocumentProfileValidationError::UnsupportedSchemaVersion { found } => {
            (0, found.to_string(), String::new())
        }
        DocumentProfileValidationError::EmptyId => (1, String::new(), String::new()),
        DocumentProfileValidationError::SameSourceAndTargetLanguage => {
            (2, String::new(), String::new())
        }
        DocumentProfileValidationError::DuplicateCapabilityId { id } => {
            (3, id.clone(), String::new())
        }
        DocumentProfileValidationError::DuplicateCapabilityTag { tag } => {
            (4, tag.clone(), String::new())
        }
        DocumentProfileValidationError::RequiredCapabilityNotBlocking { id } => {
            (5, id.clone(), String::new())
        }
        DocumentProfileValidationError::DeferredCapabilityBlocking { id } => {
            (6, id.clone(), String::new())
        }
        DocumentProfileValidationError::InvalidThreshold { field, value_milli } => {
            (7, field.clone(), value_milli.to_string())
        }
        DocumentProfileValidationError::InvalidLengthRange { tier, field } => {
            (8, tier.clone(), field.clone())
        }
        DocumentProfileValidationError::NoBlockingLengthTier => {
            (9, String::new(), String::new())
        }
        DocumentProfileValidationError::MissingErrorCategory { category } => {
            (10, category.clone(), String::new())
        }
    }
}
