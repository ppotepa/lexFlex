use lexflex_model::{
    AssertionId, ConceptId, EntityId, EvidenceId, LanguageId, ModelPackageId, ParameterId,
    QualifierId, VariableId, WorldId,
};
use serde::de::DeserializeOwned;

fn assert_invalid_json<T>()
where
    T: DeserializeOwned,
{
    assert!(serde_json::from_str::<T>(r#""bad id""#).is_err());
}

fn assert_valid_round_trip<T>(value: &str)
where
    T: DeserializeOwned + serde::Serialize + PartialEq + std::fmt::Debug,
{
    let json = format!(r#""{value}""#);
    let parsed = serde_json::from_str::<T>(&json).expect("valid json id");
    let encoded = serde_json::to_string(&parsed).expect("serialize id");
    assert_eq!(encoded, json);
}

#[test]
fn canonical_model_ids_validate_through_serde() {
    assert!(serde_json::from_str::<ConceptId>(r#""""#).is_err());
    assert!(serde_json::from_str::<ConceptId>(r#""bad id""#).is_err());
    assert!(serde_json::from_str::<ConceptId>("\"bad\nid\"").is_err());
    assert!(serde_json::from_str::<ConceptId>("\"bad\tid\"").is_err());

    assert_valid_round_trip::<AssertionId>("assertion:test/value");
    assert_valid_round_trip::<ConceptId>("concept:test.value");
    assert_valid_round_trip::<EntityId>("entity:test-value");
    assert_valid_round_trip::<EvidenceId>("evidence:test_value");
    assert_valid_round_trip::<LanguageId>("lang:test");
    assert_valid_round_trip::<ModelPackageId>("package:test");
    assert_valid_round_trip::<ParameterId>("parameter:test");
    assert_valid_round_trip::<QualifierId>("qualifier:test");
    assert_valid_round_trip::<VariableId>("var:test");
    assert_valid_round_trip::<WorldId>("world:test");
}

#[test]
fn invalid_json_is_rejected_for_all_canonical_id_families() {
    assert_invalid_json::<AssertionId>();
    assert_invalid_json::<ConceptId>();
    assert_invalid_json::<EntityId>();
    assert_invalid_json::<EvidenceId>();
    assert_invalid_json::<LanguageId>();
    assert_invalid_json::<ModelPackageId>();
    assert_invalid_json::<ParameterId>();
    assert_invalid_json::<QualifierId>();
    assert_invalid_json::<VariableId>();
    assert_invalid_json::<WorldId>();
}
