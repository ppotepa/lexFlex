use crate::document::compilation::DocumentCompilation;
use crate::document::graph::DocumentGraph;
use crate::document::knowledge::DocumentKnowledgeExtraction;
use crate::document::resolution::DocumentEntityResolution;
use crate::document::temporal_discourse::DocumentTemporalDiscourse;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SourceMetadata {
    pub title: String,
    pub language: String,
    pub uri: Option<String>,
    pub revision: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentArtifactBundle {
    pub source_document_id: String,
    pub source_sha256: String,
    pub source_metadata: SourceMetadata,
    pub compilation: DocumentCompilation,
    pub graph: DocumentGraph,
    pub entity_resolution: DocumentEntityResolution,
    pub temporal_discourse: DocumentTemporalDiscourse,
    pub knowledge: DocumentKnowledgeExtraction,
    pub summary: String,
    pub artifact_checksums: BTreeMap<String, String>,
    pub complete: bool,
    pub bundle_sha256: String,
}

impl DocumentArtifactBundle {
    pub fn validate_lineage(&self) -> Result<(), String> {
        if !self.complete { return Err("incomplete artifact bundle is not publishable".into()); }
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
        for (name, checksum) in &self.artifact_checksums {
            if checksum.is_empty() { return Err(format!("empty artifact checksum: {name}")); }
        }
        let expected_checksums = [
            ("compilation", self.compilation.compilation_sha256.as_str()),
            ("graph", self.graph.graph_sha256.as_str()),
            ("resolution", self.entity_resolution.resolution_sha256.as_str()),
            ("temporal_discourse", self.temporal_discourse.temporal_discourse_sha256.as_str()),
            ("knowledge", self.knowledge.knowledge_sha256.as_str()),
        ];
        for (name, expected) in expected_checksums {
            if self.artifact_checksums.get(name).map(String::as_str) != Some(expected) { return Err(format!("artifact checksum mismatch: {name}")); }
        }
        let expected = self.recompute_hash().map_err(|error| error.to_string())?;
        if expected != self.bundle_sha256 {
            return Err("bundle hash mismatch".into());
        }
        Ok(())
    }

    pub fn recompute_hash(&self) -> Result<String, serde_json::Error> {
        // The bundle checksum is a semantic lineage fingerprint. Hashing the
        // complete nested serde graph made persistence sensitive to incidental
        // map iteration order in legacy nested models. The artifact checksums
        // already cover each pipeline stage, so the bundle hash only needs the
        // canonical manifest and stage identities.
        let value = serde_json::json!({
            "source_document_id": self.source_document_id,
            "source_sha256": self.source_sha256,
            "source_metadata": self.source_metadata,
            "summary": self.summary,
            "artifact_checksums": self.artifact_checksums,
            "compilation_id": self.compilation.compilation_sha256,
            "graph_id": self.graph.graph_sha256,
            "resolution_id": self.entity_resolution.resolution_sha256,
            "temporal_discourse_id": self.temporal_discourse.temporal_discourse_sha256,
            "knowledge_id": self.knowledge.knowledge_sha256,
        });
        let bytes = serde_json::to_vec(&value)?;
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Ok(format!("{:x}", hasher.finalize()))
    }
}
