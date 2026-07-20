use crate::api::response::EngineResponse;
use crate::error::EngineErrorCode;
use crate::runtime::error_mapping::text_lowering_error_response;
use crate::runtime::formal_result::FormalExpressionError;

pub(crate) fn analysis_error_response(
    error: crate::api::text::TextAnalysisError,
    diagnostics: Vec<crate::api::response::EngineDiagnostic>,
) -> EngineResponse {
    let code = match error {
        crate::api::text::TextAnalysisError::CanonicalHash(_) => EngineErrorCode::Canonicalization,
        crate::api::text::TextAnalysisError::HashMismatch { .. } => EngineErrorCode::Integrity,
        _ => EngineErrorCode::InternalInvariant,
    };
    EngineResponse::Error {
        code,
        message: error.to_string(),
        diagnostics,
    }
}

pub(crate) fn formal_error_response(error: FormalExpressionError) -> EngineResponse {
    match error {
        FormalExpressionError::CanonicalHash(error) => EngineResponse::Error {
            code: EngineErrorCode::Canonicalization,
            message: error.to_string(),
            diagnostics: Vec::new(),
        },
        FormalExpressionError::Analysis(error) => {
            let code = match error {
                crate::api::text::TextAnalysisError::CanonicalHash(_) => {
                    EngineErrorCode::Canonicalization
                }
                _ => EngineErrorCode::InternalInvariant,
            };
            EngineResponse::Error {
                code,
                message: error.to_string(),
                diagnostics: Vec::new(),
            }
        }
        FormalExpressionError::Lingua(error) => text_lowering_error_response(error),
        FormalExpressionError::NonBooleanAssertion(found) => EngineResponse::Error {
            code: EngineErrorCode::InvalidAssertion,
            message: FormalExpressionError::NonBooleanAssertion(found).to_string(),
            diagnostics: Vec::new(),
        },
        FormalExpressionError::NonBooleanGoal(found) => EngineResponse::Error {
            code: EngineErrorCode::InvalidGoal,
            message: FormalExpressionError::NonBooleanGoal(found).to_string(),
            diagnostics: Vec::new(),
        },
    }
}
