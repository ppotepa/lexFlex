mod alpha;
mod error;
mod logical;
mod metrics;
mod normalizer;

pub use error::NormalizationError;
pub use metrics::NormalizationReport;
pub use normalizer::{normalize_expression, NormalizationBudget, SemanticNormalizer};
