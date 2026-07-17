use thiserror::Error;

#[derive(Debug, Error)]
pub enum LanguageValidationError {
    #[error("{0}")]
    Issue(LanguageValidationIssue),
}

impl From<LanguageValidationIssue> for LanguageValidationError {
    fn from(value: LanguageValidationIssue) -> Self {
        Self::Issue(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageValidationIssue {
    DuplicateLexemeId { lexeme_id: String },
    LexemeLanguageMismatch {
        lexeme_id: String,
        lexeme_language: String,
        package_language: String,
    },
    LexemeNormalizedLemmaMismatch {
        lexeme_id: String,
        expected: String,
        found: String,
    },
    DuplicateSenseId { sense_id: String },
    UnknownLexemeIdInSense { sense_id: String, lexeme_id: String },
    UnknownConceptIdInSense {
        sense_id: String,
        concept_id: String,
    },
    UnknownEntityAnchor { sense_id: String, entity_id: String },
    UnknownConceptReference {
        sense_id: String,
        concept_id: String,
    },
    QueryVariableNotAllowed { sense_id: String, variable: String },
    BaseCategoryMustBeAtomicWhenValencyPresent { sense_id: String },
    UnknownAnchorConcept {
        sense_id: String,
        concept_id: String,
    },
    ValencyRequiresConceptAnchor { sense_id: String, entity_id: String },
    DuplicateValencyApplicationRank { sense_id: String, rank: u16 },
    UnknownValencyParameter {
        sense_id: String,
        concept_id: String,
        parameter_id: String,
    },
    ValencyTypeMismatch {
        sense_id: String,
        parameter_id: String,
        expected: String,
        actual: String,
    },
    MissingSurfaceRelation { sense_id: String, slot_id: String },
    MissingRequiredValency {
        sense_id: String,
        concept_id: String,
        parameter_id: String,
    },
    DuplicateFormId { form_id: String },
    UnknownLexemeIdInForm { form_id: String, lexeme_id: String },
    FormNormalizedMismatch {
        form_id: String,
        expected: String,
        found: String,
    },
    DuplicateParadigmId { paradigm_id: String },
    ParadigmLanguageMismatch {
        paradigm_id: String,
        paradigm_language: String,
        package_language: String,
    },
    ParadigmUnknownLexeme {
        paradigm_id: String,
        lexeme_id: String,
    },
    ParadigmUnknownForm {
        paradigm_id: String,
        form_id: String,
    },
    DuplicateParadigmForm {
        paradigm_id: String,
        form_id: String,
    },
    SenseAnchor { sense_id: String, message: String },
    SenseMeaning { sense_id: String, message: String },
    SenseCategory { sense_id: String, message: String },
    SenseSemanticType { sense_id: String, message: String },
    SenseValency { sense_id: String, message: String },
}

impl std::fmt::Display for LanguageValidationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateLexemeId { lexeme_id } => write!(f, "duplicate lexeme id: {lexeme_id}"),
            Self::LexemeLanguageMismatch {
                lexeme_id,
                lexeme_language,
                package_language,
            } => {
                write!(
                    f,
                    "lexeme {lexeme_id} language mismatch: {lexeme_language} != {package_language}"
                )
            }
            Self::LexemeNormalizedLemmaMismatch {
                lexeme_id,
                expected,
                found,
            } => {
                write!(
                    f,
                    "lexeme {lexeme_id} normalized lemma mismatch: expected {expected}, found {found}"
                )
            }
            Self::DuplicateSenseId { sense_id } => write!(f, "duplicate sense id: {sense_id}"),
            Self::UnknownLexemeIdInSense {
                sense_id,
                lexeme_id,
            } => write!(f, "sense {sense_id}: unknown lexeme id {lexeme_id}"),
            Self::UnknownConceptIdInSense {
                sense_id,
                concept_id,
            } => write!(f, "sense {sense_id}: unknown concept id {concept_id}"),
            Self::UnknownEntityAnchor {
                sense_id,
                entity_id,
            } => write!(f, "sense {sense_id}: unknown entity anchor: {entity_id}"),
            Self::UnknownConceptReference {
                sense_id,
                concept_id,
            } => {
                write!(
                    f,
                    "sense {sense_id}: unknown concept reference: {concept_id}"
                )
            }
            Self::QueryVariableNotAllowed { sense_id, variable } => {
                write!(
                    f,
                    "sense {sense_id}: query variable not allowed in declarative sense: {variable}"
                )
            }
            Self::BaseCategoryMustBeAtomicWhenValencyPresent { sense_id } => {
                write!(f, "sense {sense_id}: valency requires atomic base category")
            }
            Self::UnknownAnchorConcept {
                sense_id,
                concept_id,
            } => write!(f, "sense {sense_id}: unknown anchor concept: {concept_id}"),
            Self::ValencyRequiresConceptAnchor {
                sense_id,
                entity_id,
            } => {
                write!(
                    f,
                    "sense {sense_id}: valency requires concept anchor, found entity anchor: {entity_id}"
                )
            }
            Self::DuplicateValencyApplicationRank { sense_id, rank } => {
                write!(
                    f,
                    "sense {sense_id}: duplicate valency application rank: {rank}"
                )
            }
            Self::UnknownValencyParameter {
                sense_id,
                concept_id,
                parameter_id,
            } => {
                write!(
                    f,
                    "sense {sense_id}: unknown valency parameter {parameter_id} for concept {concept_id}"
                )
            }
            Self::ValencyTypeMismatch {
                sense_id,
                parameter_id,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "sense {sense_id}: valency parameter {parameter_id} type mismatch: expected {expected}, found {actual}"
                )
            }
            Self::MissingSurfaceRelation { sense_id, slot_id } => {
                write!(
                    f,
                    "sense {sense_id}: valency slot {slot_id} is missing surface relation"
                )
            }
            Self::MissingRequiredValency {
                sense_id,
                concept_id,
                parameter_id,
            } => {
                write!(
                    f,
                    "sense {sense_id}: required parameter {parameter_id} missing valency slot for concept {concept_id}"
                )
            }
            Self::DuplicateFormId { form_id } => write!(f, "duplicate form id: {form_id}"),
            Self::UnknownLexemeIdInForm { form_id, lexeme_id } => {
                write!(f, "form {form_id}: unknown lexeme id {lexeme_id}")
            }
            Self::FormNormalizedMismatch {
                form_id,
                expected,
                found,
            } => {
                write!(
                    f,
                    "form {form_id} normalized mismatch: expected {expected}, found {found}"
                )
            }
            Self::DuplicateParadigmId { paradigm_id } => {
                write!(f, "duplicate paradigm id: {paradigm_id}")
            }
            Self::ParadigmLanguageMismatch {
                paradigm_id,
                paradigm_language,
                package_language,
            } => {
                write!(
                    f,
                    "paradigm {paradigm_id} language mismatch: {paradigm_language} != {package_language}"
                )
            }
            Self::ParadigmUnknownLexeme {
                paradigm_id,
                lexeme_id,
            } => write!(f, "paradigm {paradigm_id}: unknown lexeme id {lexeme_id}"),
            Self::ParadigmUnknownForm {
                paradigm_id,
                form_id,
            } => {
                write!(
                    f,
                    "paradigm {paradigm_id}: references unknown top-level form {form_id}"
                )
            }
            Self::DuplicateParadigmForm {
                paradigm_id,
                form_id,
            } => {
                write!(
                    f,
                    "paradigm {paradigm_id} contains duplicate form reference {form_id}"
                )
            }
            Self::SenseAnchor { sense_id, message } => write!(f, "sense {sense_id}: {message}"),
            Self::SenseMeaning { sense_id, message } => write!(f, "sense {sense_id}: {message}"),
            Self::SenseCategory { sense_id, message } => write!(f, "sense {sense_id}: {message}"),
            Self::SenseSemanticType { sense_id, message } => {
                write!(f, "sense {sense_id}: {message}")
            }
            Self::SenseValency { sense_id, message } => write!(f, "sense {sense_id}: {message}"),
        }
    }
}
