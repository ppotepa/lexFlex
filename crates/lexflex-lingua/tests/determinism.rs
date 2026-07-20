#[path = "support/mod.rs"]
mod support;

use lexflex_model::canonical_hash;

#[test]
fn compile_execute_and_hash_are_deterministic() {
    let program = support::model::capital_program();

    let first = support::compile_and_execute(program.clone()).expect("first");
    let expected_bytes = serde_json::to_vec(&first.value).expect("bytes");
    let expected_hash = canonical_hash(&first.value).expect("hash");

    for _ in 0..100 {
        let result = support::compile_and_execute(program.clone()).expect("execute");
        assert_eq!(
            serde_json::to_vec(&result.value).expect("bytes"),
            expected_bytes
        );
        assert_eq!(canonical_hash(&result.value).expect("hash"), expected_hash);
    }
}
