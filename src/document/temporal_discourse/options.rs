use serde::{Deserialize, Serialize};

use super::id::options_fingerprint_bytes;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentTemporalDiscourseOptions {
    pub reference_time: Option<String>,
    pub enable_implicit_temporal_inference: bool,
    pub enable_implicit_discourse_inference: bool,
    pub enable_event_coreference: bool,
    pub enable_generation_plan: bool,
}

impl Default for DocumentTemporalDiscourseOptions {
    fn default() -> Self {
        Self {
            reference_time: None,
            enable_implicit_temporal_inference: false,
            enable_implicit_discourse_inference: false,
            enable_event_coreference: true,
            enable_generation_plan: true,
        }
    }
}

impl DocumentTemporalDiscourseOptions {
    pub fn fingerprint(&self) -> String {
        options_fingerprint_bytes(self)
    }
}
