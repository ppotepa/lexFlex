use crate::api::response::{EngineDiagnostic, EngineResponse};
use crate::error::EngineErrorCode;
use crate::knowledge::snapshot::KnowledgeSnapshotError;
use crate::runtime::evidence_error_mapping::{evidence_error_code, EvidenceOrigin};
use crate::runtime::EngineError;
use lexflex_lingua::runtime::RuntimeError;
use lexflex_lingua::solve::{GoalCanonicalizationError, SolveError, UnifyError};
use lexflex_model::{AssertionCatalogError, AssertionError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProgramOrigin {
    RequestProgram,
    TextLowering,
}

pub(crate) fn solve_error_response(error: SolveError) -> EngineResponse {
    let code = match &error {
        SolveError::Goal(_)
        | SolveError::GoalCanonicalization(GoalCanonicalizationError::Validation(_)) => {
            EngineErrorCode::InvalidGoal
        }
        SolveError::Unify(UnifyError::DepthLimitExceeded { .. }) => EngineErrorCode::RuntimeBudget,
        SolveError::Unify(UnifyError::CandidateType(_)) | SolveError::AssertionIntegrity(_) => {
            EngineErrorCode::Integrity
        }
        SolveError::Unify(UnifyError::BoundScopeUnderflow)
        | SolveError::Unify(UnifyError::UnknownVariableType(_)) => {
            EngineErrorCode::InternalInvariant
        }
        SolveError::CanonicalHash(_)
        | SolveError::GoalCanonicalization(GoalCanonicalizationError::CanonicalHash(_))
        | SolveError::GoalCanonicalization(GoalCanonicalizationError::Normalization(_)) => {
            EngineErrorCode::Canonicalization
        }
    };
    EngineResponse::Error {
        code,
        message: error.to_string(),
        diagnostics: Vec::new(),
    }
}

pub(crate) fn request_program_error_response(error: EngineError) -> EngineResponse {
    engine_error_response(error, ProgramOrigin::RequestProgram)
}

pub(crate) fn text_lowering_error_response(error: EngineError) -> EngineResponse {
    engine_error_response(error, ProgramOrigin::TextLowering)
}

pub(crate) fn engine_error_response(error: EngineError, origin: ProgramOrigin) -> EngineResponse {
    match error {
        EngineError::Solve(error) => solve_error_response(error),
        EngineError::Runtime(ref runtime) => EngineResponse::Error {
            code: runtime_error_code(runtime, origin),
            message: error.to_string(),
            diagnostics: Vec::new(),
        },
        EngineError::ModelLoad(_) => EngineResponse::Error {
            code: EngineErrorCode::ModelError,
            message: error.to_string(),
            diagnostics: Vec::new(),
        },
        EngineError::Compile(_) => EngineResponse::Error {
            code: EngineErrorCode::InvalidProgram,
            message: error.to_string(),
            diagnostics: Vec::new(),
        },
        EngineError::Normalize(_) => EngineResponse::Error {
            code: EngineErrorCode::Canonicalization,
            message: error.to_string(),
            diagnostics: Vec::new(),
        },
    }
}

pub(crate) fn incoming_knowledge_error_response(error: KnowledgeSnapshotError) -> EngineResponse {
    let code = match &error {
        KnowledgeSnapshotError::Assertion { source, .. } => match source {
            AssertionError::Evidence(error) => evidence_error_code(error, EvidenceOrigin::Incoming),
            AssertionError::Normalization(_) => EngineErrorCode::InvalidAssertion,
            AssertionError::CanonicalHash(_) => EngineErrorCode::Canonicalization,
            AssertionError::HashMismatch { .. }
            | AssertionError::IdMismatch { .. }
            | AssertionError::ExpressionNotNormalized { .. } => EngineErrorCode::Integrity,
        },
        KnowledgeSnapshotError::AssertionCatalog { source, .. } => assertion_catalog_code(source),
        KnowledgeSnapshotError::CanonicalHash(_) => EngineErrorCode::Canonicalization,
        KnowledgeSnapshotError::AssertionKeyMismatch { .. }
        | KnowledgeSnapshotError::SnapshotHashMismatch { .. } => EngineErrorCode::Integrity,
        KnowledgeSnapshotError::MissingAssertion { .. } => EngineErrorCode::InternalInvariant,
    };
    EngineResponse::Error {
        code,
        message: error.to_string(),
        diagnostics: Vec::new(),
    }
}

pub(crate) fn stored_knowledge_error_response(error: KnowledgeSnapshotError) -> EngineResponse {
    let code = match &error {
        KnowledgeSnapshotError::CanonicalHash(_)
        | KnowledgeSnapshotError::AssertionKeyMismatch { .. }
        | KnowledgeSnapshotError::SnapshotHashMismatch { .. } => EngineErrorCode::Integrity,
        KnowledgeSnapshotError::Assertion { source, .. } => match source {
            AssertionError::Evidence(error) => evidence_error_code(error, EvidenceOrigin::Stored),
            _ => EngineErrorCode::Integrity,
        },
        KnowledgeSnapshotError::AssertionCatalog { source, .. } => match source {
            AssertionCatalogError::Integrity(AssertionError::Evidence(error)) => {
                evidence_error_code(error, EvidenceOrigin::Stored)
            }
            _ => EngineErrorCode::Integrity,
        },
        KnowledgeSnapshotError::MissingAssertion { .. } => EngineErrorCode::InternalInvariant,
    };
    EngineResponse::Error {
        code,
        message: error.to_string(),
        diagnostics: Vec::new(),
    }
}

pub(crate) fn missing_assertion_response(
    assertion_id: lexflex_model::AssertionId,
) -> EngineResponse {
    EngineResponse::Error {
        code: EngineErrorCode::InternalInvariant,
        message: format!("missing indexed assertion: {assertion_id}"),
        diagnostics: vec![EngineDiagnostic::MissingAssertion { assertion_id }],
    }
}

fn assertion_catalog_code(error: &AssertionCatalogError) -> EngineErrorCode {
    match error {
        AssertionCatalogError::Integrity(AssertionError::Evidence(error)) => {
            evidence_error_code(error, EvidenceOrigin::Incoming)
        }
        AssertionCatalogError::Integrity(AssertionError::CanonicalHash(_)) => {
            EngineErrorCode::Canonicalization
        }
        AssertionCatalogError::Integrity(_) | AssertionCatalogError::NonBoolean(_) => {
            EngineErrorCode::InvalidAssertion
        }
        AssertionCatalogError::Type(_) => EngineErrorCode::InvalidAssertion,
    }
}

fn runtime_error_code(error: &RuntimeError, origin: ProgramOrigin) -> EngineErrorCode {
    match error {
        RuntimeError::BudgetExceeded
        | RuntimeError::ExpansionDepthExceeded { .. }
        | RuntimeError::ExpansionCountExceeded { .. }
        | RuntimeError::EnvironmentBindingLimitExceeded { .. } => EngineErrorCode::RuntimeBudget,
        RuntimeError::UnknownFunction(_)
        | RuntimeError::UnknownLocal(_)
        | RuntimeError::MissingArgument(_)
        | RuntimeError::MissingSelfSubject { .. }
        | RuntimeError::ExpectedClosure
        | RuntimeError::ExpectedSemantic
        | RuntimeError::ExpansionCycle { .. } => match origin {
            ProgramOrigin::RequestProgram => EngineErrorCode::InvalidProgram,
            ProgramOrigin::TextLowering => EngineErrorCode::InternalInvariant,
        },
        RuntimeError::Normalization(_) => EngineErrorCode::Canonicalization,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexflex_lingua::FunctionId;

    #[test]
    fn runtime_error_variants_map_by_phase() {
        assert_eq!(
            runtime_error_code(&RuntimeError::BudgetExceeded, ProgramOrigin::RequestProgram),
            EngineErrorCode::RuntimeBudget
        );
        assert_eq!(
            runtime_error_code(
                &RuntimeError::UnknownFunction(FunctionId::new_unchecked("missing")),
                ProgramOrigin::RequestProgram
            ),
            EngineErrorCode::InvalidProgram
        );
        assert_eq!(
            runtime_error_code(
                &RuntimeError::UnknownFunction(FunctionId::new_unchecked("missing")),
                ProgramOrigin::TextLowering
            ),
            EngineErrorCode::InternalInvariant
        );
    }
}
