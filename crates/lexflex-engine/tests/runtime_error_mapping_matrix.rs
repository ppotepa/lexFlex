use lexflex_engine::EngineErrorCode;

#[test]
fn runtime_mapping_matrix_target_is_executable() {
    let cases = [
        (EngineErrorCode::RuntimeBudget, "RuntimeBudget"),
        (EngineErrorCode::InvalidProgram, "InvalidProgram"),
        (EngineErrorCode::Canonicalization, "Canonicalization"),
        (EngineErrorCode::Integrity, "Integrity"),
        (EngineErrorCode::InternalInvariant, "InternalInvariant"),
    ];
    for (code, expected) in cases {
        assert_eq!(
            serde_json::to_string(&code).expect("code"),
            format!("\"{expected}\"")
        );
    }
}
