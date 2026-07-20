use lexflex_language::FeatureStructure;

pub(crate) fn merge_subject_predicate_features(
    subject: &FeatureStructure,
    predicate: &FeatureStructure,
) -> Result<FeatureStructure, ()> {
    subject.merged(predicate).map_err(|_| ())
}
