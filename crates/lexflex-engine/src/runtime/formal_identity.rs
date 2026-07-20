use lexflex_model::{
    canonical_hash, CanonicalDigest, CanonicalHashError, SemanticType, VariableId,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct FormalAlternativeKey {
    pub(crate) expression_hash: CanonicalDigest,
    pub(crate) variable_hash: CanonicalDigest,
    pub(crate) projection_hash: CanonicalDigest,
}

pub(crate) fn formal_alternative_key(
    expression_hash: CanonicalDigest,
    variables: &BTreeMap<VariableId, SemanticType>,
    projection: &[VariableId],
) -> Result<FormalAlternativeKey, CanonicalHashError> {
    Ok(FormalAlternativeKey {
        expression_hash,
        variable_hash: canonical_hash(variables)?,
        projection_hash: canonical_hash(projection)?,
    })
}
