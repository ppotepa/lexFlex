use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use crate::document::knowledge::DocumentKnowledgeExtraction;
use crate::query::{DocumentAnswer, QueryError, QueryInterlingua, QueryService};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceIdError(pub String);

impl fmt::Display for WorkspaceIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for WorkspaceIdError {}

macro_rules! workspace_id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(pub String);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = WorkspaceIdError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                validate_id(s)?;
                Ok(Self(s.to_string()))
            }
        }
    };
}

fn validate_id(value: &str) -> Result<(), WorkspaceIdError> {
    if value.is_empty() {
        return Err(WorkspaceIdError("id must not be empty".into()));
    }
    if value.len() > 128 {
        return Err(WorkspaceIdError("id too long".into()));
    }
    if !value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '.')) {
        return Err(WorkspaceIdError("invalid id characters".into()));
    }
    Ok(())
}

workspace_id_type!(KnowledgeWorkspaceId);
workspace_id_type!(KnowledgeSnapshotId);
workspace_id_type!(DocumentBundleId);
workspace_id_type!(GlobalEntityId);
workspace_id_type!(GlobalEventId);
workspace_id_type!(GlobalClaimId);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentArtifactBundleManifest {
    pub id: DocumentBundleId,
    pub document_id: String,
    pub compilation_id: String,
    pub graph_id: String,
    pub entity_resolution_id: Option<String>,
    pub temporal_discourse_id: Option<String>,
    pub knowledge_id: Option<String>,
    pub source_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeWorkspaceManifest {
    pub id: KnowledgeWorkspaceId,
    pub schema: u32,
    pub bundle_ids: Vec<DocumentBundleId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalEntityResolution {
    pub id: GlobalEntityId,
    pub document_bundle_id: DocumentBundleId,
    pub source_cluster_ref: String,
    pub selected_global_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalEventResolution {
    pub id: GlobalEventId,
    pub document_bundle_id: DocumentBundleId,
    pub source_cluster_ref: String,
    pub selected_global_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalClaimIndexEntry {
    pub id: GlobalClaimId,
    pub document_bundle_id: DocumentBundleId,
    pub claim_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeWorkspaceSnapshot {
    pub id: KnowledgeSnapshotId,
    pub workspace_id: KnowledgeWorkspaceId,
    pub parent_snapshot_id: Option<KnowledgeSnapshotId>,
    pub active_bundle_ids: Vec<DocumentBundleId>,
    pub bundles: BTreeMap<DocumentBundleId, DocumentArtifactBundleManifest>,
    pub global_entities: BTreeMap<GlobalEntityId, GlobalEntityResolution>,
    pub global_events: BTreeMap<GlobalEventId, GlobalEventResolution>,
    pub global_claims: BTreeMap<GlobalClaimId, GlobalClaimIndexEntry>,
    pub snapshot_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceAnswer {
    pub workspace_id: KnowledgeWorkspaceId,
    pub snapshot_id: KnowledgeSnapshotId,
    pub answer: DocumentAnswer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceQueryResult {
    pub workspace_id: KnowledgeWorkspaceId,
    pub snapshot_id: KnowledgeSnapshotId,
    pub answers: Vec<DocumentAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeWorkspaceError(pub String);

impl fmt::Display for KnowledgeWorkspaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for KnowledgeWorkspaceError {}

#[derive(Default)]
pub struct KnowledgeWorkspaceService {
    bundles: BTreeMap<DocumentBundleId, DocumentArtifactBundleManifest>,
}

impl KnowledgeWorkspaceService {
    pub fn ingest_bundle(
        &mut self,
        bundle: DocumentArtifactBundleManifest,
    ) -> Result<(), KnowledgeWorkspaceError> {
        self.bundles.insert(bundle.id.clone(), bundle);
        Ok(())
    }

    pub fn snapshot(&self, workspace_id: KnowledgeWorkspaceId) -> KnowledgeWorkspaceSnapshot {
        let bundle_ids = self.bundles.keys().cloned().collect::<Vec<_>>();
        KnowledgeWorkspaceSnapshot {
            id: KnowledgeSnapshotId(format!("{workspace_id}:snapshot")),
            workspace_id,
            parent_snapshot_id: None,
            active_bundle_ids: bundle_ids.clone(),
            bundles: self.bundles.clone(),
            global_entities: BTreeMap::new(),
            global_events: BTreeMap::new(),
            global_claims: BTreeMap::new(),
            snapshot_sha256: String::new(),
        }
    }

    pub fn answer_query(
        &self,
        query: QueryInterlingua,
        knowledge: &DocumentKnowledgeExtraction,
    ) -> Result<DocumentAnswer, QueryError> {
        QueryService::answer_document_query(query, knowledge)
    }
}

