use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityResolutionSummary {
    pub mentions_total: usize,
    pub synthetic_mentions_total: usize,
    pub decisions_total: usize,
    pub accepted: usize,
    pub hard_accepted: usize,
    pub ambiguous: usize,
    pub deferred: usize,
    pub unresolved: usize,
    pub excluded: usize,
    pub clusters_total: usize,
    pub resolved_clusters: usize,
    pub unresolved_clusters: usize,
    pub diagnostics_info: usize,
    pub diagnostics_warning: usize,
    pub diagnostics_error: usize,
    pub diagnostics_fatal: usize,
}

pub fn summarize_entity_resolution(
    resolution: &crate::document::resolution::DocumentEntityResolution,
) -> EntityResolutionSummary {
    resolution.summary.clone()
}
