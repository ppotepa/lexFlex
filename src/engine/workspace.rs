use super::types::*;
use crate::runtime::DocumentArtifactBundle;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionClaimRef { pub claim_id: ClaimId, pub bundle_id: BundleId }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SessionIndexes { pub claims_by_bundle: BTreeMap<BundleId, Vec<ClaimId>> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionWorkspace {
    pub session_id: SessionId,
    pub parent_snapshot_id: Option<SnapshotId>,
    pub active_source_ids: Vec<SourceId>,
    pub sources: BTreeMap<SourceId, SourceSnapshot>,
    pub bundles: BTreeMap<BundleId, DocumentArtifactBundle>,
    pub claims: BTreeMap<ClaimId, SessionClaimRef>,
    pub entities: BTreeMap<String, String>,
    pub indexes: SessionIndexes,
    pub snapshot_id: SnapshotId,
    pub snapshot_sha256: String,
}

impl SessionWorkspace {
    pub fn new(session_id: impl Into<String>) -> Self { let mut s = Self { session_id: session_id.into(), parent_snapshot_id: None, active_source_ids: vec![], sources: BTreeMap::new(), bundles: BTreeMap::new(), claims: BTreeMap::new(), entities: BTreeMap::new(), indexes: SessionIndexes::default(), snapshot_id: "snapshot:0".into(), snapshot_sha256: String::new() }; s.refresh_hash(); s }
    pub fn clear(&mut self) { let parent = self.snapshot_id.clone(); self.active_source_ids.clear(); self.sources.clear(); self.bundles.clear(); self.claims.clear(); self.entities.clear(); self.indexes = SessionIndexes::default(); self.parent_snapshot_id = Some(parent.clone()); self.snapshot_id = format!("snapshot:clear:{}", stable_hash(&parent)); self.refresh_hash(); }
    pub fn add_bundle(&mut self, source: &SourceSnapshot, bundle: DocumentArtifactBundle) -> BundleId {
        let id = format!("bundle:{}:{}", source.source_id, source.content_sha256);
        for claim in &bundle.knowledge.claim_order { self.claims.insert(claim.to_string(), SessionClaimRef { claim_id: claim.to_string(), bundle_id: id.clone() }); }
        self.indexes.claims_by_bundle.insert(id.clone(), bundle.knowledge.claim_order.iter().map(ToString::to_string).collect());
        self.active_source_ids.push(source.source_id.clone()); self.active_source_ids.sort(); self.active_source_ids.dedup(); self.sources.insert(source.source_id.clone(), source.clone()); self.bundles.insert(id.clone(), bundle); self.parent_snapshot_id = Some(self.snapshot_id.clone()); self.snapshot_id = format!("snapshot:{}", self.snapshot_sha256); self.refresh_hash(); id
    }
    pub fn refresh_hash(&mut self) {
        let source_manifest = self.sources.iter().map(|(id, source)| (id, &source.content_sha256)).collect::<Vec<_>>();
        let bundle_manifest = self.bundles.keys().collect::<Vec<_>>();
        let manifest = (&self.session_id, &self.parent_snapshot_id, &self.active_source_ids, source_manifest, bundle_manifest, &self.snapshot_id);
        let bytes = serde_json::to_vec(&manifest).expect("workspace serialization");
        let mut h = Sha256::new(); h.update(bytes); self.snapshot_sha256 = format!("{:x}", h.finalize());
    }
    pub fn sole_bundle(&self) -> Option<&DocumentArtifactBundle> { self.bundles.values().next() }
    pub fn validate(&self) -> Result<(), String> {
        let mut copy = self.clone();
        let expected = copy.snapshot_sha256.clone();
        copy.refresh_hash();
        if expected != copy.snapshot_sha256 { return Err("session snapshot hash mismatch".into()); }
        if self.sources.keys().collect::<Vec<_>>() != self.active_source_ids.iter().collect::<Vec<_>>() { return Err("active source index mismatch".into()); }
        for source in self.sources.values() {
            if source.content_sha256.is_empty() || source.text.is_empty() { return Err(format!("invalid source snapshot: {}", source.source_id)); }
            let mut hash = Sha256::new();
            hash.update(source.text.as_bytes());
            if source.content_sha256 != format!("{:x}", hash.finalize()) { return Err(format!("source content hash mismatch: {}", source.source_id)); }
        }
        for (bundle_id, bundle) in &self.bundles {
            bundle.validate_lineage()?;
            if bundle.source_metadata.language.is_empty() { return Err(format!("bundle has no source language: {bundle_id}")); }
        }
        for (claim_id, claim) in &self.claims {
            if !self.bundles.contains_key(&claim.bundle_id) { return Err(format!("claim {claim_id} references missing bundle {}", claim.bundle_id)); }
            if !self.indexes.claims_by_bundle.get(&claim.bundle_id).is_some_and(|ids| ids.contains(claim_id)) { return Err(format!("claim index missing {claim_id}")); }
        }
        Ok(())
    }
}

fn stable_hash(value: &str) -> String { let mut h = Sha256::new(); h.update(value.as_bytes()); format!("{:x}", h.finalize()) }
