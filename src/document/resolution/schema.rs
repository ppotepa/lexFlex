use serde::{Deserialize, Serialize};

pub const ENTITY_RESOLUTION_SCHEMA_VERSION: u32 = 1;
pub const ENTITY_RESOLUTION_ALGORITHM_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocumentEntityResolutionSchema {
    pub schema_version: u32,
    pub algorithm_version: u32,
}

impl DocumentEntityResolutionSchema {
    pub const CURRENT: Self = Self {
        schema_version: ENTITY_RESOLUTION_SCHEMA_VERSION,
        algorithm_version: ENTITY_RESOLUTION_ALGORITHM_VERSION,
    };

    pub fn is_supported(self) -> bool {
        self.schema_version == ENTITY_RESOLUTION_SCHEMA_VERSION
            && self.algorithm_version == ENTITY_RESOLUTION_ALGORITHM_VERSION
    }
}
