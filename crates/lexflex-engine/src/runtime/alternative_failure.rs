use crate::api::response::EngineResponse;
use crate::runtime::formal_error_mapping::formal_error_response;
use crate::runtime::formal_result::FormalExpressionError;
use crate::EngineError;

pub(crate) enum AlternativeFailure {
    Rejected { message: String },
    Fatal { response: EngineResponse },
}

pub(crate) fn classify_alternative_failure(error: FormalExpressionError) -> AlternativeFailure {
    match error {
        FormalExpressionError::NonBooleanAssertion(_)
        | FormalExpressionError::NonBooleanGoal(_) => AlternativeFailure::Rejected {
            message: error.to_string(),
        },
        FormalExpressionError::Lingua(EngineError::Compile(_)) => AlternativeFailure::Rejected {
            message: error.to_string(),
        },
        fatal => AlternativeFailure::Fatal {
            response: formal_error_response(fatal),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::EngineErrorCode;
    use lexflex_lingua::runtime::RuntimeError;
    use lexflex_model::SemanticType;

    #[test]
    fn non_boolean_candidate_is_rejected() {
        assert!(matches!(
            classify_alternative_failure(FormalExpressionError::NonBooleanAssertion(
                SemanticType::Boolean
            )),
            AlternativeFailure::Rejected { .. }
        ));
    }

    #[test]
    fn runtime_budget_candidate_is_fatal() {
        let AlternativeFailure::Fatal { response } = classify_alternative_failure(
            FormalExpressionError::Lingua(EngineError::Runtime(RuntimeError::BudgetExceeded)),
        ) else {
            panic!("runtime budget must be fatal");
        };
        assert!(matches!(
            response,
            EngineResponse::Error {
                code: EngineErrorCode::RuntimeBudget,
                ..
            }
        ));
    }
}
