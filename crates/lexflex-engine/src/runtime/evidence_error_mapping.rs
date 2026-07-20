use crate::api::response::EngineResponse;
use crate::error::EngineErrorCode;
use lexflex_model::EvidenceError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EvidenceOrigin {
    Incoming,
    Stored,
    GeneratedText,
}

pub(crate) fn evidence_error_response(
    error: EvidenceError,
    origin: EvidenceOrigin,
) -> EngineResponse {
    let code = match origin {
        EvidenceOrigin::Incoming => match error {
            EvidenceError::CanonicalHash(_) => EngineErrorCode::Canonicalization,
            _ => EngineErrorCode::InvalidEvidence,
        },
        EvidenceOrigin::Stored => match error {
            EvidenceError::CanonicalHash(_)
            | EvidenceError::InvalidSpan { .. }
            | EvidenceError::IdMismatch { .. }
            | EvidenceError::EmptySourceId
            | EvidenceError::KeyMismatch { .. }
            | EvidenceError::ConflictingEvidence(_) => EngineErrorCode::Integrity,
        },
        EvidenceOrigin::GeneratedText => match error {
            EvidenceError::CanonicalHash(_) => EngineErrorCode::Canonicalization,
            _ => EngineErrorCode::InternalInvariant,
        },
    };
    EngineResponse::Error {
        code,
        message: error.to_string(),
        diagnostics: Vec::new(),
    }
}

pub(crate) fn evidence_error_code(
    error: &lexflex_model::EvidenceError,
    origin: EvidenceOrigin,
) -> EngineErrorCode {
    match origin {
        EvidenceOrigin::Incoming => match error {
            lexflex_model::EvidenceError::CanonicalHash(_) => EngineErrorCode::Canonicalization,
            _ => EngineErrorCode::InvalidEvidence,
        },
        EvidenceOrigin::Stored => EngineErrorCode::Integrity,
        EvidenceOrigin::GeneratedText => match error {
            lexflex_model::EvidenceError::CanonicalHash(_) => EngineErrorCode::Canonicalization,
            _ => EngineErrorCode::InternalInvariant,
        },
    }
}
