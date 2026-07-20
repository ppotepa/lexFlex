use lexflex_engine::EngineSessionState;
use lexflex_model::CanonicalDigest;

fn hashes() -> (CanonicalDigest, CanonicalDigest) {
    (
        lexflex_model::canonical_hash("model").expect("hash"),
        lexflex_model::canonical_hash("language").expect("hash"),
    )
}

#[test]
fn invalid_session_id_is_rejected_during_state_deserialization() {
    let (model, language) = hashes();
    let state = EngineSessionState::new("valid-session", model, language).expect("state");
    let mut value: serde_json::Value =
        serde_json::from_slice(&serde_json::to_vec(&state).expect("serialize")).expect("json");
    value["session_id"] = serde_json::Value::String("../escape".into());
    assert!(serde_json::from_value::<EngineSessionState>(value).is_err());
}
