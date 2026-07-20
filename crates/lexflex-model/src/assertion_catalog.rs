use crate::{
    AssertionError, ConceptCatalog, ExpressionTypeChecker, ExpressionTypeEnvironment,
    ExpressionTypeError, SemanticAssertion, SemanticType,
};
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AssertionCatalogError {
    #[error("assertion integrity failed: {0}")]
    Integrity(#[from] AssertionError),

    #[error("assertion expression type failed: {0}")]
    Type(#[from] ExpressionTypeError),

    #[error("assertion expression must be Boolean, found {0:?}")]
    NonBoolean(SemanticType),
}

pub(crate) fn verify_assertion_with_catalog(
    assertion: &SemanticAssertion,
    catalog: &ConceptCatalog,
) -> Result<(), AssertionCatalogError> {
    assertion.verify()?;

    let checker = ExpressionTypeChecker::new(catalog);
    let mut environment = ExpressionTypeEnvironment::new(catalog);
    let value_type = checker.infer(assertion.expression(), &mut environment)?;

    if value_type != SemanticType::Boolean {
        return Err(AssertionCatalogError::NonBoolean(value_type));
    }

    Ok(())
}
