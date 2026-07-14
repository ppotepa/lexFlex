use serde::{Deserialize, Serialize};

pub const KNOWLEDGE_EXTRACTION_SCHEMA_VERSION: u32 = 1;
pub const KNOWLEDGE_EXTRACTION_ALGORITHM_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocumentKnowledgeSchema {
    pub schema_version: u32,
    pub algorithm_version: u32,
}

impl DocumentKnowledgeSchema {
    pub const CURRENT: Self = Self {
        schema_version: KNOWLEDGE_EXTRACTION_SCHEMA_VERSION,
        algorithm_version: KNOWLEDGE_EXTRACTION_ALGORITHM_VERSION,
    };

    pub fn is_supported(self) -> bool {
        self.schema_version == KNOWLEDGE_EXTRACTION_SCHEMA_VERSION
            && self.algorithm_version == KNOWLEDGE_EXTRACTION_ALGORITHM_VERSION
    }
}
