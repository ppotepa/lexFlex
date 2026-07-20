use lexflex_engine::api::text::{
    TextAnalysis, TextAnalysisAlternative, TextAnalysisAlternativeInput, TextAnalysisInput,
    TextAnalysisKind,
};
use lexflex_language::LanguageId;
use lexflex_model::{canonical_hash, SemanticExpression, SemanticValue, SourceSpan};
use lexflex_parser::{ParseMetrics, ParseScore};

fn analysis() -> TextAnalysis {
    let expression = SemanticExpression::Value(SemanticValue::Boolean(true));
    TextAnalysis::new(TextAnalysisInput {
        source_id: "source:test:analysis".into(),
        language: LanguageId::new("en").expect("language"),
        span: SourceSpan::new(0, 4).expect("span"),
        kind: TextAnalysisKind::Assertion,
        canonical_expression: expression,
        variables: Default::default(),
        projection: Vec::new(),
        formal_steps: 1,
        parser_metrics: ParseMetrics::default(),
        derivations: None,
    })
    .expect("analysis")
}

#[test]
fn valid_analysis_round_trips() {
    let value = analysis();
    let bytes = serde_json::to_vec(&value).expect("serialize");
    let restored: TextAnalysis = serde_json::from_slice(&bytes).expect("deserialize");
    assert_eq!(restored, value);
}

#[test]
fn analysis_constructor_enforces_goal_projection_contract() {
    let input = TextAnalysisInput {
        source_id: "source".into(),
        language: lexflex_language::LanguageId::new_unchecked("en"),
        span: lexflex_model::SourceSpan::new(0, 1).expect("span"),
        kind: TextAnalysisKind::Goal,
        canonical_expression: lexflex_model::SemanticExpression::Value(true.into()),
        variables: Default::default(),
        projection: Vec::new(),
        formal_steps: 0,
        parser_metrics: Default::default(),
        derivations: None,
    };
    assert!(TextAnalysis::new(input).is_err());
}

#[test]
fn stale_analysis_hash_is_rejected() {
    let value = analysis();
    let mut json: serde_json::Value = serde_json::to_value(value).expect("json");
    json["canonical_hash"] =
        serde_json::to_value(canonical_hash("wrong").expect("hash")).expect("digest");
    assert!(serde_json::from_value::<TextAnalysis>(json).is_err());
}

#[test]
fn assertion_alternative_accepts_empty_query_shape() {
    let alternative = TextAnalysisAlternative::new(
        TextAnalysisKind::Assertion,
        TextAnalysisAlternativeInput {
            canonical_expression: SemanticExpression::Value(SemanticValue::Boolean(true)),
            variables: Default::default(),
            projection: Vec::new(),
            formal_steps: 1,
            parser_metrics: ParseMetrics::default(),
            derivations: None,
            score: ParseScore::lexical(0),
        },
    )
    .expect("assertion alternative");
    assert_eq!(alternative.kind(), TextAnalysisKind::Assertion);
}
