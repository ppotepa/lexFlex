use crate::document::compilation::DocumentCompilation;
use crate::document::graph::DocumentGraph;
use crate::document::knowledge::DocumentKnowledgeExtraction;
use crate::document::resolution::DocumentEntityResolution;
use crate::document::temporal_discourse::DocumentTemporalDiscourse;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentArtifactBundle {
    pub source_document_id: String,
    pub source_sha256: String,
    pub compilation: DocumentCompilation,
    pub graph: DocumentGraph,
    pub entity_resolution: DocumentEntityResolution,
    pub temporal_discourse: DocumentTemporalDiscourse,
    pub knowledge: DocumentKnowledgeExtraction,
    pub bundle_sha256: String,
}

impl DocumentArtifactBundle {
    pub fn validate_lineage(&self) -> Result<(), String> {
        if self.compilation.document.id.to_string() != self.source_document_id {
            return Err("compilation document does not match source document".into());
        }
        if self.compilation.document.source_sha256 != self.source_sha256 {
            return Err("source hash does not match compilation".into());
        }
        if self.graph.source_document_id != self.compilation.document.id
            || self.graph.source_sha256 != self.source_sha256
        {
            return Err("graph document lineage mismatch".into());
        }
        if self.entity_resolution.source_graph_id != self.graph.id {
            return Err("entity resolution graph lineage mismatch".into());
        }
        if self.temporal_discourse.source_graph_id != self.graph.id {
            return Err("temporal graph lineage mismatch".into());
        }
        if self.knowledge.source_graph_id != self.graph.id {
            return Err("knowledge graph lineage mismatch".into());
        }
        if self.knowledge.source_resolution_id.as_ref() != Some(&self.entity_resolution.id) {
            return Err("knowledge resolution lineage mismatch".into());
        }
        if self.knowledge.source_temporal_discourse_id.as_ref()
            != Some(&self.temporal_discourse.id)
        {
            return Err("knowledge temporal lineage mismatch".into());
        }
        if self.knowledge.source_sha256 != self.source_sha256 {
            return Err("knowledge source lineage mismatch".into());
        }
        let expected = self.recompute_hash().map_err(|error| error.to_string())?;
        if expected != self.bundle_sha256 {
            return Err("bundle hash mismatch".into());
        }
        Ok(())
    }

    pub fn recompute_hash(&self) -> Result<String, serde_json::Error> {
        let mut value = self.clone();
        value.bundle_sha256.clear();
        let bytes = serde_json::to_vec(&value)?;
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Ok(format!("{:x}", hasher.finalize()))
    }
}
