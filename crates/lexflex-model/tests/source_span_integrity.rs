use lexflex_model::SourceSpan;

#[test]
fn valid_source_span_round_trips() {
    let span = SourceSpan::new(1, 3).expect("valid span");
    let encoded = serde_json::to_string(&span).expect("serialize");
    let decoded: SourceSpan = serde_json::from_str(&encoded).expect("deserialize");

    assert_eq!(decoded.start(), 1);
    assert_eq!(decoded.end(), 3);
}

#[test]
fn invalid_source_span_deserialize_fails() {
    let result = serde_json::from_value::<SourceSpan>(serde_json::json!({
        "start": 10,
        "end": 1
    }));

    assert!(result.is_err());
}
