use lexflex_language::FeatureStructure;

pub(crate) fn expected_features_match(
    expected: &FeatureStructure,
    actual: &FeatureStructure,
) -> bool {
    actual.contains_all(expected)
}
