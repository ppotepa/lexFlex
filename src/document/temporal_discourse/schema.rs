use serde::{Deserialize, Serialize};

pub const TEMPORAL_DISCOURSE_SCHEMA_VERSION: u32 = 1;
pub const TEMPORAL_DISCOURSE_ALGORITHM_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocumentTemporalDiscourseSchema {
    pub schema_version: u32,
    pub algorithm_version: u32,
}

impl DocumentTemporalDiscourseSchema {
    pub const CURRENT: Self = Self {
        schema_version: TEMPORAL_DISCOURSE_SCHEMA_VERSION,
        algorithm_version: TEMPORAL_DISCOURSE_ALGORITHM_VERSION,
    };

    pub fn is_supported(self) -> bool {
        self.schema_version == TEMPORAL_DISCOURSE_SCHEMA_VERSION
            && self.algorithm_version == TEMPORAL_DISCOURSE_ALGORITHM_VERSION
    }
}
