use crate::CatalogValidationIssue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogValidationReport {
    pub issues: Vec<CatalogValidationIssue>,
}

impl CatalogValidationReport {
    pub fn is_clean(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn into_result(self) -> Result<(), Self> {
        if self.is_clean() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl std::fmt::Display for CatalogValidationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = self
            .issues
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ");
        write!(f, "{text}")
    }
}
