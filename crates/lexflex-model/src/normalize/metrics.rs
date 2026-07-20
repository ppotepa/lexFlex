use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizationReport {
    pub changed: bool,
    pub input_nodes: usize,
    pub output_nodes: usize,
    pub max_depth: usize,
    pub renamed_bound_variables: usize,
    pub flattened_logical_nodes: usize,
    pub removed_duplicates: usize,
}
