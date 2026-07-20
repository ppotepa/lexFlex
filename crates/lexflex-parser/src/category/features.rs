use crate::category::outcome::CategoryMismatch;
use lexflex_language::FeatureStructure;

pub(crate) fn unify_expected_features(
    expected: &FeatureStructure,
    actual: &FeatureStructure,
) -> Result<(), CategoryMismatch> {
    for (name, expected_value) in &expected.values {
        match actual.get(name) {
            None => {
                return Err(CategoryMismatch::MissingFeature {
                    name: name.clone(),
                    expected: expected_value.clone(),
                });
            }
            Some(actual_value) if actual_value != expected_value => {
                return Err(CategoryMismatch::FeatureValue {
                    name: name.clone(),
                    expected: expected_value.clone(),
                    actual: actual_value.clone(),
                });
            }
            Some(_) => {}
        }
    }
    Ok(())
}
