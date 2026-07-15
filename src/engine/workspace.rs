use super::types::*;
use crate::runtime::DocumentArtifactBundle;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const ENGINE_SESSION_SCHEMA: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionClaimRef { pub claim_id: ClaimId, pub bundle_id: BundleId }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DebugSessionContext {
    pub last_user_turn: Option<DebugTurnContext>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DebugTurnContext {
    pub input_text: String,
    pub detected_language: Option<String>,
    pub parse_status: String,
    pub sentence_count: usize,
    pub question_kind: Option<String>,
    pub predicate: Option<String>,
    pub projection: Option<String>,
    pub source_candidates: Vec<String>,
    pub selected_bundle_id: Option<BundleId>,
    pub query_json: Option<String>,
    pub plan_steps: Vec<String>,
    pub plan_operators: Vec<String>,
    pub execution_rows: Vec<String>,
    pub answer_status: Option<String>,
    pub answer_text: Option<String>,
    pub diagnostics: Vec<String>,
    pub run_id: Option<RunId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SessionIndexes { pub claims_by_bundle: BTreeMap<BundleId, Vec<ClaimId>> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationWorkspace {
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
    #[serde(default)]
    pub next_run_ordinal: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationTurn {
    pub turn_id: String,
    pub source_language: crate::core::interlingua::LanguageId,
    pub target_language: crate::core::interlingua::LanguageId,
    pub source_text: String,
    pub source_sha256: String,
    pub interlingua: crate::core::interlingua::Interlingua,
    pub interlingua_sha256: String,
    pub output: Option<String>,
    pub output_sha256: Option<String>,
    pub source_id: SourceId,
    pub bundle_id: BundleId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationWorkspace {
    pub session_id: SessionId,
    pub parent_snapshot_id: Option<SnapshotId>,
    pub turn_order: Vec<String>,
    pub turns: BTreeMap<String, TranslationTurn>,
    pub default_source_language: LanguageMode,
    pub default_target_language: crate::core::interlingua::LanguageId,
    pub snapshot_id: SnapshotId,
    pub snapshot_sha256: String,
}

impl TranslationWorkspace {
    pub fn new(session_id: impl Into<String>) -> Self {
        let mut value = Self {
            session_id: session_id.into(),
            parent_snapshot_id: None,
            turn_order: Vec::new(),
            turns: BTreeMap::new(),
            default_source_language: LanguageMode::Explicit("pl".into()),
            default_target_language: crate::core::interlingua::LanguageId::new("en"),
            snapshot_id: "translation:0".into(),
            snapshot_sha256: String::new(),
        };
        value.refresh_hash();
        value
    }

    pub fn configure(&mut self, from: LanguageMode, to: crate::core::interlingua::LanguageId) {
        self.default_source_language = from;
        self.default_target_language = to;
    }

    pub fn add_turn(&mut self, turn: TranslationTurn) {
        self.parent_snapshot_id = Some(self.snapshot_id.clone());
        self.turn_order.push(turn.turn_id.clone());
        self.turns.insert(turn.turn_id.clone(), turn);
        self.refresh_hash();
    }

    pub fn clear(&mut self) {
        self.parent_snapshot_id = Some(self.snapshot_id.clone());
        self.turn_order.clear();
        self.turns.clear();
        self.refresh_hash();
    }

    pub fn refresh_hash(&mut self) {
        let turn_manifest = self.turns.iter().map(|(id, turn)| (
            id,
            &turn.source_language,
            &turn.target_language,
            &turn.source_sha256,
            &turn.interlingua_sha256,
            &turn.output_sha256,
            &turn.source_id,
            &turn.bundle_id,
        )).collect::<Vec<_>>();
        let manifest = (
            &self.session_id,
            &self.parent_snapshot_id,
            &self.turn_order,
            turn_manifest,
            &self.default_source_language,
            &self.default_target_language,
        );
        let bytes = serde_json::to_vec(&manifest).expect("translation workspace serialization");
        let mut h = Sha256::new();
        h.update(bytes);
        self.snapshot_sha256 = format!("{:x}", h.finalize());
        self.snapshot_id = format!("translation:{}", &self.snapshot_sha256[..16]);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.turn_order.len() != self.turns.len() { return Err("translation turn order mismatch".into()); }
        if self.turn_order.iter().any(|id| !self.turns.contains_key(id)) { return Err("translation turn missing".into()); }
        let mut copy = self.clone();
        let expected = copy.snapshot_sha256.clone();
        copy.refresh_hash();
        if expected != copy.snapshot_sha256 { return Err("translation snapshot hash mismatch".into()); }
        Ok(())
    }
}

fn default_engine_session_schema() -> u32 { ENGINE_SESSION_SCHEMA }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineSession {
    #[serde(default = "default_engine_session_schema")]
    pub schema: u32,
    pub session_id: SessionId,
    pub conversation: ConversationWorkspace,
    pub translation: TranslationWorkspace,
    #[serde(default)]
    pub debug: DebugSessionContext,
    pub answer_language: AnswerLanguage,
    pub snapshot_id: SnapshotId,
    pub snapshot_sha256: String,
    #[serde(default)]
    pub next_run_ordinal: u64,
}

impl EngineSession {
    pub fn new(session_id: impl Into<String>) -> Self {
        let session_id = session_id.into();
        let mut value = Self {
            schema: ENGINE_SESSION_SCHEMA,
            session_id: session_id.clone(),
            conversation: ConversationWorkspace::new(session_id.clone()),
            translation: TranslationWorkspace::new(session_id),
            debug: DebugSessionContext::default(),
            answer_language: AnswerLanguage::Auto,
            snapshot_id: "session:0".into(),
            snapshot_sha256: String::new(),
            next_run_ordinal: 0,
        };
        value.refresh_hash();
        value
    }

    pub fn next_run_id(&mut self) -> String {
        self.next_run_ordinal = self.next_run_ordinal.saturating_add(1);
        format!("run:{:08}", self.next_run_ordinal)
    }

    pub fn refresh_hash(&mut self) {
        let manifest = (
            self.schema,
            &self.session_id,
            &self.conversation.snapshot_sha256,
            &self.translation.snapshot_sha256,
            &self.answer_language,
        );
        let bytes = serde_json::to_vec(&manifest).expect("engine session serialization");
        let mut h = Sha256::new(); h.update(bytes);
        self.snapshot_sha256 = format!("{:x}", h.finalize());
        self.snapshot_id = format!("session:{}", &self.snapshot_sha256[..16]);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != ENGINE_SESSION_SCHEMA { return Err(format!("unsupported session schema: {}", self.schema)); }
        self.conversation.validate()?;
        self.translation.validate()?;
        let mut copy = self.clone();
        let expected = copy.snapshot_sha256.clone();
        copy.refresh_hash();
        if expected != copy.snapshot_sha256 { return Err("engine session hash mismatch".into()); }
        Ok(())
    }
}

impl std::ops::Deref for EngineSession {
    type Target = ConversationWorkspace;
    fn deref(&self) -> &Self::Target { &self.conversation }
}

impl std::ops::DerefMut for EngineSession {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.conversation }
}

impl ConversationWorkspace {
    pub fn new(session_id: impl Into<String>) -> Self { let mut s = Self { session_id: session_id.into(), parent_snapshot_id: None, active_source_ids: vec![], sources: BTreeMap::new(), bundles: BTreeMap::new(), claims: BTreeMap::new(), entities: BTreeMap::new(), indexes: SessionIndexes::default(), snapshot_id: "snapshot:0".into(), snapshot_sha256: String::new(), next_run_ordinal: 0 }; s.refresh_hash(); s }
    pub fn next_run_id(&mut self) -> String { self.next_run_ordinal = self.next_run_ordinal.saturating_add(1); format!("run:{:08}", self.next_run_ordinal) }
    pub fn clear(&mut self) { let parent = self.snapshot_id.clone(); self.active_source_ids.clear(); self.sources.clear(); self.bundles.clear(); self.claims.clear(); self.entities.clear(); self.indexes = SessionIndexes::default(); self.parent_snapshot_id = Some(parent.clone()); self.snapshot_id = format!("snapshot:clear:{}", stable_hash(&parent)); self.refresh_hash(); }
    pub fn add_bundle(&mut self, source: &SourceSnapshot, bundle: DocumentArtifactBundle) -> BundleId {
        let id = format!("bundle:{}:{}", source.source_id, source.content_sha256);
        for claim in &bundle.knowledge.claim_order { self.claims.insert(claim.to_string(), SessionClaimRef { claim_id: claim.to_string(), bundle_id: id.clone() }); }
        self.indexes.claims_by_bundle.insert(id.clone(), bundle.knowledge.claim_order.iter().map(ToString::to_string).collect());
        self.active_source_ids.push(source.source_id.clone()); self.active_source_ids.sort(); self.active_source_ids.dedup(); self.sources.insert(source.source_id.clone(), source.clone()); self.bundles.insert(id.clone(), bundle); self.parent_snapshot_id = Some(self.snapshot_id.clone()); self.snapshot_id = format!("snapshot:{}", self.snapshot_sha256); self.refresh_hash(); id
    }
    pub fn remove_sources(&mut self, source_ids: &[SourceId]) {
        let removed = source_ids.iter().cloned().collect::<std::collections::BTreeSet<_>>();
        let removed_bundles = self.bundles.keys()
            .filter(|bundle_id| removed.iter().any(|source_id| bundle_id.starts_with(&format!("bundle:{source_id}:"))))
            .cloned().collect::<std::collections::BTreeSet<_>>();
        self.parent_snapshot_id = Some(self.snapshot_id.clone());
        self.active_source_ids.retain(|id| !removed.contains(id));
        self.sources.retain(|id, _| !removed.contains(id));
        self.bundles.retain(|id, _| !removed_bundles.contains(id));
        self.claims.retain(|_, claim| !removed_bundles.contains(&claim.bundle_id));
        self.indexes.claims_by_bundle.retain(|id, _| !removed_bundles.contains(id));
        self.refresh_hash();
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
