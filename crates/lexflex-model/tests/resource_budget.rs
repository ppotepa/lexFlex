use lexflex_model::ResourceBudget;

#[test]
fn default_resource_budget_is_valid_and_shared() {
    let budget = ResourceBudget::default();
    assert!(budget.validate().is_ok());
    assert_eq!(budget.max_text_bytes, budget.max_document_bytes);
    assert!(budget.max_generation_bytes > 0);
}

#[test]
fn zero_resource_limit_is_rejected() {
    let budget = ResourceBudget {
        max_json_bytes: 0,
        ..ResourceBudget::default()
    };
    assert_eq!(
        budget.validate(),
        Err("resource budget limits must be non-zero")
    );
}
