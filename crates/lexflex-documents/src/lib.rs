#![forbid(unsafe_code)]

use lexflex_model::{
    canonical_hash, AssertionCatalogError, CanonicalDigest, ConceptCatalog, Evidence,
    EvidenceError, EvidenceSet, ResourceBudget, SemanticAssertion, SemanticExpression, SourceSpan,
    WorldId,
};
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const MAX_PERSISTED_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSegment {
    pub segment_id: String,
    pub start: u64,
    pub end: u64,
    pub text: String,
    pub analysis: Option<SemanticExpression>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    pub document_id: String,
    pub source_text: String,
    pub source_hash: String,
    pub segments: Vec<DocumentSegment>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct DocumentStore {
    documents: std::collections::BTreeMap<String, Document>,
}

impl<'de> Deserialize<'de> for DocumentStore {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            documents: std::collections::BTreeMap<String, Document>,
        }
        let wire = Wire::deserialize(deserializer)?;
        let documents = wire.documents;
        let store = Self { documents };
        store.verify().map_err(D::Error::custom)?;
        Ok(store)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentLimits {
    pub max_bytes: usize,
    pub max_segments: usize,
}

impl Default for DocumentLimits {
    fn default() -> Self {
        Self {
            max_bytes: ResourceBudget::default().max_document_bytes,
            max_segments: ResourceBudget::default().max_document_segments,
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DocumentError {
    #[error("document id is empty")]
    EmptyDocumentId,
    #[error("document already exists: {0}")]
    DuplicateDocument(String),
    #[error("document does not exist: {0}")]
    MissingDocument(String),
    #[error("document segment is invalid")]
    InvalidSegment,
    #[error("document source hash mismatch")]
    SourceHashMismatch,
    #[error("document exceeds configured resource limits")]
    ResourceLimit,
    #[error("document store IO error: {0}")]
    PersistenceIo(String),
    #[error("document store serialization error: {0}")]
    PersistenceSerde(String),
    #[error("document store payload exceeds resource limit")]
    PersistenceTooLarge,
}

pub fn ingest(document_id: String, text: &str) -> Result<Document, DocumentError> {
    ingest_with_limits(document_id, text, DocumentLimits::default())
}

pub fn ingest_with_limits(
    document_id: String,
    text: &str,
    limits: DocumentLimits,
) -> Result<Document, DocumentError> {
    if document_id.trim().is_empty() {
        return Err(DocumentError::EmptyDocumentId);
    }
    if text.len() > limits.max_bytes {
        return Err(DocumentError::ResourceLimit);
    }
    let mut segments = Vec::new();
    let mut offset = 0u64;
    for (index, line) in text.split('\n').enumerate() {
        let end = offset + line.len() as u64;
        if !line.trim().is_empty() {
            if segments.len() == limits.max_segments {
                return Err(DocumentError::ResourceLimit);
            }
            segments.push(DocumentSegment {
                segment_id: format!("{document_id}:segment:{index}"),
                start: offset,
                end,
                text: line.to_owned(),
                analysis: None,
            });
        }
        offset = end + 1;
    }
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let source_hash = format!("sha256:{:x}", hasher.finalize());
    Ok(Document {
        document_id,
        source_text: text.to_owned(),
        source_hash,
        segments,
    })
}

impl DocumentStore {
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, DocumentError> {
        let file = std::fs::File::open(path)
            .map_err(|error| DocumentError::PersistenceIo(error.to_string()))?;
        let mut bytes = Vec::new();
        file.take((MAX_PERSISTED_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|error| DocumentError::PersistenceIo(error.to_string()))?;
        if bytes.len() > MAX_PERSISTED_BYTES {
            return Err(DocumentError::PersistenceTooLarge);
        }
        serde_json::from_slice(&bytes)
            .map_err(|error| DocumentError::PersistenceSerde(error.to_string()))
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), DocumentError> {
        self.verify()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| DocumentError::PersistenceIo(error.to_string()))?;
        }
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| DocumentError::PersistenceSerde(error.to_string()))?;
        let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temporary = path.with_file_name(format!(
            "{}.{}.{}.tmp",
            path.file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("store"),
            std::process::id(),
            sequence
        ));
        let mut temporary_file = std::fs::File::create(&temporary)
            .map_err(|error| DocumentError::PersistenceIo(error.to_string()))?;
        use std::io::Write;
        temporary_file
            .write_all(&bytes)
            .and_then(|_| temporary_file.sync_all())
            .map_err(|error| DocumentError::PersistenceIo(error.to_string()))?;
        std::fs::rename(&temporary, path).map_err(|error| {
            let _ = std::fs::remove_file(&temporary);
            DocumentError::PersistenceIo(error.to_string())
        })
    }

    pub fn insert(&mut self, document: Document) -> Result<(), DocumentError> {
        verify_document(&document)?;
        if self.documents.contains_key(&document.document_id) {
            return Err(DocumentError::DuplicateDocument(document.document_id));
        }
        self.documents
            .insert(document.document_id.clone(), document);
        Ok(())
    }

    pub fn remove(&mut self, document_id: &str) -> Result<Document, DocumentError> {
        let mut candidate = self.clone();
        let removed = candidate
            .documents
            .remove(document_id)
            .ok_or_else(|| DocumentError::MissingDocument(document_id.to_owned()))?;
        candidate.verify()?;
        *self = candidate;
        Ok(removed)
    }

    pub fn replace(&mut self, document_id: &str, text: &str) -> Result<(), DocumentError> {
        if !self.documents.contains_key(document_id) {
            return Err(DocumentError::MissingDocument(document_id.to_owned()));
        }
        let mut candidate = self.clone();
        let replacement = ingest(document_id.to_owned(), text)?;
        verify_document(&replacement)?;
        candidate
            .documents
            .insert(document_id.to_owned(), replacement);
        candidate.verify()?;
        *self = candidate;
        Ok(())
    }

    pub fn get(&self, document_id: &str) -> Option<&Document> {
        self.documents.get(document_id)
    }

    pub fn attach_analysis(
        &mut self,
        document_id: &str,
        segment_id: &str,
        expression: SemanticExpression,
    ) -> Result<(), DocumentError> {
        let mut candidate = self.clone();
        let document = candidate
            .documents
            .get_mut(document_id)
            .ok_or_else(|| DocumentError::MissingDocument(document_id.to_owned()))?;
        let segment = document
            .segments
            .iter_mut()
            .find(|segment| segment.segment_id == segment_id)
            .ok_or(DocumentError::InvalidSegment)?;
        segment.analysis = Some(
            expression
                .normalized()
                .map_err(|_| DocumentError::InvalidSegment)?,
        );
        candidate.verify()?;
        *self = candidate;
        Ok(())
    }
    pub fn documents(&self) -> impl Iterator<Item = &Document> {
        self.documents.values()
    }

    pub fn find_expression(
        &self,
        expression: &SemanticExpression,
    ) -> Result<Vec<(&Document, &DocumentSegment)>, lexflex_model::CanonicalHashError> {
        let wanted = semantic_digest(expression)?;
        let mut matches = Vec::new();
        for document in self.documents.values() {
            for segment in &document.segments {
                if segment
                    .analysis
                    .as_ref()
                    .and_then(|analysis| analysis.canonical_hash().ok())
                    .as_ref()
                    == Some(&wanted)
                {
                    matches.push((document, segment));
                }
            }
        }
        Ok(matches)
    }

    pub fn evidence_for_segment(
        &self,
        document_id: &str,
        segment_id: &str,
    ) -> Result<Evidence, DocumentError> {
        let document = self
            .documents
            .get(document_id)
            .ok_or_else(|| DocumentError::MissingDocument(document_id.to_owned()))?;
        let segment = document
            .segments
            .iter()
            .find(|segment| segment.segment_id == segment_id)
            .ok_or(DocumentError::InvalidSegment)?;
        let digest = document
            .source_hash
            .strip_prefix("sha256:")
            .and_then(|value| CanonicalDigest::new(value.to_owned()).ok())
            .ok_or(DocumentError::SourceHashMismatch)?;
        Evidence::create(
            document.document_id.clone(),
            Some(
                SourceSpan::new(segment.start, segment.end)
                    .map_err(|_| DocumentError::InvalidSegment)?,
            ),
            Some(digest),
        )
        .map_err(|error| match error {
            EvidenceError::InvalidSpan { .. } | EvidenceError::EmptySourceId => {
                DocumentError::InvalidSegment
            }
            EvidenceError::CanonicalHash(_)
            | EvidenceError::IdMismatch { .. }
            | EvidenceError::KeyMismatch { .. }
            | EvidenceError::ConflictingEvidence(_) => DocumentError::SourceHashMismatch,
        })
    }

    pub fn create_assertion_for_segment(
        &self,
        document_id: &str,
        segment_id: &str,
        expression: SemanticExpression,
        world: WorldId,
        catalog: &ConceptCatalog,
    ) -> Result<SemanticAssertion, DocumentErrorOrAssertion> {
        let evidence = self.evidence_for_segment(document_id, segment_id)?;
        SemanticAssertion::create(
            expression,
            EvidenceSet::singleton(evidence).map_err(DocumentErrorOrAssertion::Evidence)?,
            world,
            catalog,
        )
        .map_err(DocumentErrorOrAssertion::Assertion)
    }
    pub fn verify(&self) -> Result<(), DocumentError> {
        self.documents.values().try_for_each(verify_document)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DocumentErrorOrAssertion {
    #[error(transparent)]
    Document(#[from] DocumentError),
    #[error(transparent)]
    Evidence(#[from] EvidenceError),
    #[error(transparent)]
    Assertion(#[from] AssertionCatalogError),
}

fn verify_document(document: &Document) -> Result<(), DocumentError> {
    if document.document_id.trim().is_empty() {
        return Err(DocumentError::EmptyDocumentId);
    }
    let mut previous_end = 0;
    for segment in &document.segments {
        if segment.segment_id.trim().is_empty()
            || segment.start < previous_end
            || segment.end < segment.start
            || segment.end - segment.start != segment.text.len() as u64
        {
            return Err(DocumentError::InvalidSegment);
        }
        previous_end = segment.end;
    }
    let mut hasher = Sha256::new();
    hasher.update(document.source_text.as_bytes());
    let expected = format!("sha256:{:x}", hasher.finalize());
    if document.source_hash != expected {
        return Err(DocumentError::SourceHashMismatch);
    }
    Ok(())
}

pub fn semantic_digest(
    expression: &SemanticExpression,
) -> Result<CanonicalDigest, lexflex_model::CanonicalHashError> {
    canonical_hash(expression)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingestion_preserves_segment_provenance() {
        let document = ingest("doc-1".into(), "Paris is a city.\nFrance is a country.").unwrap();
        assert_eq!(document.segments.len(), 2);
        assert_eq!(
            (document.segments[0].start, document.segments[0].end),
            (0, 16)
        );
        assert_eq!(document.segments[1].segment_id, "doc-1:segment:1");
    }

    #[test]
    fn invalid_document_is_rejected_without_store_mutation() {
        let mut store = DocumentStore::default();
        let document = ingest("doc-1".into(), "Paris").unwrap();
        store.insert(document.clone()).unwrap();
        let mut corrupt = document;
        corrupt.segments[0].end += 1;
        assert_eq!(store.insert(corrupt), Err(DocumentError::InvalidSegment));
        assert_eq!(store.documents().count(), 1);
    }

    #[test]
    fn source_hash_corruption_is_rejected() {
        let mut document = ingest("doc-1".into(), "Paris").unwrap();
        document.source_hash = "sha256:corrupt".into();
        let mut store = DocumentStore::default();
        assert_eq!(
            store.insert(document),
            Err(DocumentError::SourceHashMismatch)
        );
        assert_eq!(store.documents().count(), 0);
    }

    #[test]
    fn segment_analysis_is_normalized_transactionally() {
        let mut store = DocumentStore::default();
        let document = ingest("doc-1".into(), "Paris").unwrap();
        let segment_id = document.segments[0].segment_id.clone();
        store.insert(document).unwrap();
        store
            .attach_analysis(
                "doc-1",
                &segment_id,
                SemanticExpression::Entity(lexflex_model::EntityId::new_unchecked("entity:paris")),
            )
            .unwrap();
        assert!(store.get("doc-1").unwrap().segments[0].analysis.is_some());
        let matches = store
            .find_expression(&SemanticExpression::Entity(
                lexflex_model::EntityId::new_unchecked("entity:paris"),
            ))
            .unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(
            store
                .evidence_for_segment("doc-1", &segment_id)
                .unwrap()
                .source_id(),
            "doc-1"
        );
    }

    #[test]
    fn document_assertion_enters_the_standard_knowledge_transaction() {
        let source = std::fs::read_to_string("../../data/model/concepts.ron").unwrap();
        let catalog: ConceptCatalog = ron::from_str(&source).unwrap();
        let mut documents = DocumentStore::default();
        let document = ingest("doc-1".into(), "Paris is true.").unwrap();
        let segment_id = document.segments[0].segment_id.clone();
        documents.insert(document).unwrap();
        let assertion = documents
            .create_assertion_for_segment(
                "doc-1",
                &segment_id,
                SemanticExpression::Equals {
                    left: Box::new(SemanticExpression::Value(true.into())),
                    right: Box::new(SemanticExpression::Value(true.into())),
                },
                WorldId::new_unchecked("world:default"),
                &catalog,
            )
            .unwrap();
        let mut snapshot = lexflex_engine::KnowledgeSnapshot::new().unwrap();
        snapshot.upsert(assertion, &catalog).unwrap();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot.evidence_count(), 1);
        snapshot.verify(&catalog).unwrap();
    }

    #[test]
    fn document_limits_reject_before_segmentation() {
        assert_eq!(
            ingest_with_limits(
                "doc-1".into(),
                "one\ntwo",
                DocumentLimits {
                    max_bytes: 64,
                    max_segments: 1,
                },
            ),
            Err(DocumentError::ResourceLimit)
        );
        assert_eq!(
            ingest_with_limits(
                "doc-1".into(),
                "oversized",
                DocumentLimits {
                    max_bytes: 4,
                    max_segments: 10,
                },
            ),
            Err(DocumentError::ResourceLimit)
        );
    }

    #[test]
    fn document_replace_is_atomic_and_refreshes_provenance() {
        let mut store = DocumentStore::default();
        store
            .insert(ingest("doc-1".into(), "old").unwrap())
            .unwrap();
        let old_hash = store.get("doc-1").unwrap().source_hash.clone();
        store.replace("doc-1", "new\ncontent").unwrap();
        assert_ne!(store.get("doc-1").unwrap().source_hash, old_hash);
        assert_eq!(store.get("doc-1").unwrap().segments.len(), 2);
        let before = store.clone();
        assert_eq!(
            store.replace("missing", "ignored"),
            Err(DocumentError::MissingDocument("missing".into()))
        );
        assert_eq!(store, before);
    }

    #[test]
    fn corrupted_store_deserialization_fails_without_repair() {
        let mut store = DocumentStore::default();
        store
            .insert(ingest("doc".into(), "Paris").expect("document"))
            .expect("insert");
        let mut value: serde_json::Value = serde_json::to_value(&store).expect("serialize");
        value["documents"]["doc"]["source_hash"] = serde_json::Value::String("bad".into());
        let error = serde_json::from_value::<DocumentStore>(value).expect_err("corruption");
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn document_store_file_round_trip_is_verified_and_atomic() {
        let root = std::env::temp_dir().join(format!("lexflex-documents-{}", std::process::id()));
        let path = root.join("store.json");
        let mut store = DocumentStore::default();
        store
            .insert(ingest("doc".into(), "Paris").expect("document"))
            .expect("insert");
        store.save_to_file(&path).expect("save");
        let loaded = DocumentStore::load_from_file(&path).expect("load");
        assert_eq!(loaded, store);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn concurrent_document_store_saves_leave_one_valid_file() {
        let root = std::env::temp_dir().join(format!(
            "lexflex-documents-concurrent-{}",
            std::process::id()
        ));
        let path = root.join("store.json");
        let mut store = DocumentStore::default();
        store
            .insert(ingest("doc-1".into(), "Paris").unwrap())
            .unwrap();
        let workers = (0..8)
            .map(|_| {
                let path = path.clone();
                let store = store.clone();
                std::thread::spawn(move || store.save_to_file(&path).expect("save"))
            })
            .collect::<Vec<_>>();
        for worker in workers {
            worker.join().expect("worker");
        }
        assert!(DocumentStore::load_from_file(&path).is_ok());
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn oversized_persisted_store_is_rejected_before_deserialization() {
        let root = std::env::temp_dir().join(format!(
            "lexflex-documents-oversized-{}",
            std::process::id()
        ));
        let path = root.join("store.json");
        std::fs::create_dir_all(&root).expect("root");
        std::fs::write(&path, vec![b'{'; MAX_PERSISTED_BYTES + 1]).expect("payload");
        assert_eq!(
            DocumentStore::load_from_file(&path),
            Err(DocumentError::PersistenceTooLarge)
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
