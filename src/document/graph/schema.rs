use serde::{Deserialize, Serialize};

pub const DOCUMENT_GRAPH_SCHEMA_VERSION: u32 = 1;
pub const DOCUMENT_GRAPH_ALGORITHM_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocumentGraphSchema {
    pub schema_version: u32,
    pub algorithm_version: u32,
}

impl DocumentGraphSchema {
    pub const CURRENT: Self = Self {
        schema_version: DOCUMENT_GRAPH_SCHEMA_VERSION,
        algorithm_version: DOCUMENT_GRAPH_ALGORITHM_VERSION,
    };

    pub fn is_supported(self) -> bool {
        self.schema_version == DOCUMENT_GRAPH_SCHEMA_VERSION
            && self.algorithm_version == DOCUMENT_GRAPH_ALGORITHM_VERSION
    }
}
