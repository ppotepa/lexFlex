use serde::{Deserialize, Serialize};

use super::id::options_fingerprint_bytes;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentGraphBuildOptions {
    pub include_preserved_blocks: bool,
    pub include_constructions: bool,
    pub include_coordination_members: bool,
    pub include_modifier_mentions: bool,
    pub create_events: bool,
    pub create_unresolved_fragments: bool,
    pub link_shared_entity_ids: bool,
    pub link_unique_explicit_references: bool,
    pub emit_same_surface_relations: bool,
}

impl Default for DocumentGraphBuildOptions {
    fn default() -> Self {
        Self {
            include_preserved_blocks: true,
            include_constructions: true,
            include_coordination_members: true,
            include_modifier_mentions: true,
            create_events: true,
            create_unresolved_fragments: true,
            link_shared_entity_ids: true,
            link_unique_explicit_references: true,
            emit_same_surface_relations: true,
        }
    }
}

impl DocumentGraphBuildOptions {
    pub fn fingerprint(&self) -> String {
        options_fingerprint_bytes(self)
    }
}
