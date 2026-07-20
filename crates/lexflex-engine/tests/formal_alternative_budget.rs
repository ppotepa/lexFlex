use lexflex_engine::runtime::{TextRuntimeBudget, TextRuntimeBudgetError};

#[test]
fn text_runtime_budget_rejects_zero_derivations() {
    assert_eq!(
        TextRuntimeBudget::new(0, 1),
        Err(TextRuntimeBudgetError::ZeroFormalDerivations)
    );
}

#[test]
fn text_runtime_budget_rejects_zero_alternatives() {
    assert_eq!(
        TextRuntimeBudget::new(1, 0),
        Err(TextRuntimeBudgetError::ZeroFormalAlternatives)
    );
}

#[test]
fn text_runtime_budget_exposes_valid_limits() {
    let budget = TextRuntimeBudget::new(7, 3).expect("valid budget");

    assert_eq!(budget.max_formal_derivations(), 7);
    assert_eq!(budget.max_formal_alternatives(), 3);
}
