use lexflex_model::{canonical_hash, EntityId, LanguageId, VariableId};

#[test]
fn identifier_boundaries_reject_control_and_non_ascii_values() {
    let invalid = ["\n", "\0", "ż", "value space", "value?", "value#"];
    for value in invalid {
        assert!(
            EntityId::new(value).is_err(),
            "accepted invalid entity: {value:?}"
        );
        assert!(
            LanguageId::new(value).is_err(),
            "accepted invalid language: {value:?}"
        );
        assert!(
            VariableId::new(value).is_err(),
            "accepted invalid variable: {value:?}"
        );
    }
}

#[test]
fn identifier_serde_does_not_repair_invalid_payloads() {
    for payload in ["\"\"", "\"bad value\"", "\"value?\"", "null", "123"] {
        assert!(
            serde_json::from_str::<EntityId>(payload).is_err(),
            "accepted {payload}"
        );
        assert!(
            serde_json::from_str::<LanguageId>(payload).is_err(),
            "accepted {payload}"
        );
    }
}

#[test]
fn canonical_hash_is_deterministic_across_repeated_adversarial_values() {
    let values = [
        String::new(),
        "a".repeat(1024),
        "unicode: żółć".to_owned(),
        "line\nfeed\tvalue".to_owned(),
    ];
    for value in values {
        let first = canonical_hash(&value).expect("hash value");
        let second = canonical_hash(&value).expect("hash value");
        assert_eq!(first, second);
    }
}
