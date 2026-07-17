use lexflex_engine::EngineSessionState;
use lexflex_model::{canonical_hash, CanonicalDigest};

#[test]
fn new_session_verifies_against_current_hashes() {
    let model_hash: CanonicalDigest = canonical_hash("model").expect("model hash");
    let language_hash: CanonicalDigest = canonical_hash("language").expect("language hash");
    let state = EngineSessionState::new(
        "session:test",
        model_hash.clone(),
        language_hash.clone(),
    )
    .expect("session");
    assert_eq!(state.verify(&model_hash, &language_hash), Ok(()));
}
