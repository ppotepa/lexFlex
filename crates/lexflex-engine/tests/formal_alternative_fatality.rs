use lexflex_engine::runtime::{TextRuntimeBudget, TextRuntimeBudgetError};

#[test]
fn fatality_contract_keeps_budget_validation_distinct_from_rejection() {
    assert_eq!(
        TextRuntimeBudget::new(0, 1),
        Err(TextRuntimeBudgetError::ZeroFormalDerivations)
    );
    assert_eq!(
        TextRuntimeBudget::new(1, 0),
        Err(TextRuntimeBudgetError::ZeroFormalAlternatives)
    );
}
