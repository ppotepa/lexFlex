#![forbid(unsafe_code)]

use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmTeacherRequest {
    pub prompt: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmTeacherResponse {
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderBudget {
    pub max_prompt_bytes: usize,
    pub max_response_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRequest {
    pub prompt: String,
    pub model: String,
    pub prompt_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderArtifact {
    pub provider: String,
    pub content: String,
    pub content_hash: String,
    pub request: ProviderRequest,
}

impl ProviderArtifact {
    pub fn verify(&self) -> Result<(), ProviderError> {
        if self.provider.trim().is_empty() {
            return Err(ProviderError::EmptyField("provider"));
        }
        if self.request.model.trim().is_empty() {
            return Err(ProviderError::EmptyField("model"));
        }
        if self.request.prompt_version.trim().is_empty() {
            return Err(ProviderError::EmptyField("prompt_version"));
        }
        let mut hasher = Sha256::new();
        hasher.update(self.content.as_bytes());
        let expected = format!("sha256:{:x}", hasher.finalize());
        if self.content_hash != expected {
            return Err(ProviderError::ContentHashMismatch);
        }
        Ok(())
    }

    pub fn into_document(
        self,
        document_id: String,
    ) -> Result<lexflex_documents::Document, lexflex_documents::DocumentError> {
        self.verify()
            .map_err(|_| lexflex_documents::DocumentError::SourceHashMismatch)?;
        let document = lexflex_documents::ingest(document_id, &self.content)?;
        if document.source_hash != self.content_hash {
            return Err(lexflex_documents::DocumentError::SourceHashMismatch);
        }
        Ok(document)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProviderError {
    #[error("provider prompt exceeds budget")]
    PromptBudget,
    #[error("provider response exceeds budget")]
    ResponseBudget,
    #[error("provider request field is empty: {0}")]
    EmptyField(&'static str),
    #[error("provider artifact content hash mismatch")]
    ContentHashMismatch,
    #[error("provider cache limit must be greater than zero")]
    InvalidCacheLimit,
    #[error("provider cache identity mismatch")]
    CacheIdentityMismatch,
    #[error("provider cache serialization failed: {0}")]
    CacheSerialization(String),
    #[error("provider document commit failed: {0}")]
    DocumentCommit(String),
}

pub fn ingest_artifact_transactionally(
    store: &mut lexflex_documents::DocumentStore,
    artifact: ProviderArtifact,
    document_id: String,
) -> Result<lexflex_documents::Document, ProviderError> {
    let document = artifact
        .into_document(document_id)
        .map_err(|error| ProviderError::DocumentCommit(error.to_string()))?;
    let mut candidate = store.clone();
    candidate
        .insert(document.clone())
        .map_err(|error| ProviderError::DocumentCommit(error.to_string()))?;
    candidate
        .verify()
        .map_err(|error| ProviderError::DocumentCommit(error.to_string()))?;
    *store = candidate;
    Ok(document)
}

pub trait KnowledgeProvider {
    fn fetch(&self, request: ProviderRequest) -> Result<ProviderArtifact, ProviderError>;
}

pub trait LlmProvider: KnowledgeProvider {}

impl<T: KnowledgeProvider + ?Sized> LlmProvider for T {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProviderCache {
    max_entries: usize,
    artifacts: BTreeMap<String, ProviderArtifact>,
}

impl<'de> Deserialize<'de> for ProviderCache {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            max_entries: usize,
            artifacts: BTreeMap<String, ProviderArtifact>,
        }
        let wire = Wire::deserialize(deserializer)?;
        if wire.max_entries == 0 {
            return Err(D::Error::custom(ProviderError::InvalidCacheLimit));
        }
        for (key, artifact) in &wire.artifacts {
            artifact.verify().map_err(D::Error::custom)?;
            let expected = request_key(&artifact.request).map_err(D::Error::custom)?;
            if key != &expected {
                return Err(D::Error::custom(ProviderError::CacheIdentityMismatch));
            }
        }
        if wire.artifacts.len() > wire.max_entries {
            return Err(D::Error::custom("provider cache exceeds configured limit"));
        }
        Ok(Self {
            max_entries: wire.max_entries,
            artifacts: wire.artifacts,
        })
    }
}

impl ProviderCache {
    pub fn new(max_entries: usize) -> Result<Self, ProviderError> {
        if max_entries == 0 {
            return Err(ProviderError::InvalidCacheLimit);
        }
        Ok(Self {
            max_entries,
            artifacts: BTreeMap::new(),
        })
    }

    pub fn get_or_fetch<P: LlmProvider>(
        &mut self,
        provider: &P,
        request: ProviderRequest,
    ) -> Result<ProviderArtifact, ProviderError> {
        let key = request_key(&request)?;
        if let Some(artifact) = self.artifacts.get(&key) {
            artifact.verify()?;
            if artifact.request != request {
                return Err(ProviderError::CacheIdentityMismatch);
            }
            return Ok(artifact.clone());
        }
        let artifact = provider.fetch(request)?;
        artifact.verify()?;
        if self.artifacts.len() >= self.max_entries {
            if let Some(first) = self.artifacts.keys().next().cloned() {
                self.artifacts.remove(&first);
            }
        }
        self.artifacts.insert(key, artifact.clone());
        Ok(artifact)
    }

    pub fn len(&self) -> usize {
        self.artifacts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }
}

fn request_key(request: &ProviderRequest) -> Result<String, ProviderError> {
    let bytes = serde_json::to_vec(request)
        .map_err(|error| ProviderError::CacheSerialization(error.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixtureProvider {
    budget: ProviderBudget,
}

impl FixtureProvider {
    pub fn new(budget: ProviderBudget) -> Self {
        Self { budget }
    }
}

impl KnowledgeProvider for FixtureProvider {
    fn fetch(&self, request: ProviderRequest) -> Result<ProviderArtifact, ProviderError> {
        if request.model.is_empty() {
            return Err(ProviderError::EmptyField("model"));
        }
        if request.prompt_version.is_empty() {
            return Err(ProviderError::EmptyField("prompt_version"));
        }
        if request.prompt.len() > self.budget.max_prompt_bytes {
            return Err(ProviderError::PromptBudget);
        }
        if request.prompt.len() > self.budget.max_response_bytes {
            return Err(ProviderError::ResponseBudget);
        }
        let mut hasher = Sha256::new();
        hasher.update(request.prompt.as_bytes());
        let content_hash = format!("sha256:{:x}", hasher.finalize());
        Ok(ProviderArtifact {
            provider: "fixture".into(),
            content: request.prompt.clone(),
            content_hash,
            request,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_provider_is_bounded_and_untrusted() {
        let provider = FixtureProvider::new(ProviderBudget {
            max_prompt_bytes: 8,
            max_response_bytes: 8,
        });
        let artifact = provider
            .fetch(ProviderRequest {
                prompt: "observe".into(),
                model: "fixture".into(),
                prompt_version: "v1".into(),
            })
            .unwrap();
        assert_eq!(artifact.provider, "fixture");
        artifact.verify().unwrap();
        let mut corrupt = artifact.clone();
        corrupt.content.push('!');
        assert_eq!(corrupt.verify(), Err(ProviderError::ContentHashMismatch));
        assert_eq!(
            artifact
                .into_document("provider-doc".into())
                .unwrap()
                .document_id,
            "provider-doc"
        );
        assert!(matches!(
            provider.fetch(ProviderRequest {
                prompt: "too long!".into(),
                model: "fixture".into(),
                prompt_version: "v1".into()
            }),
            Err(ProviderError::PromptBudget)
        ));
    }

    #[test]
    fn provider_cache_replays_verified_artifacts_with_a_bound() {
        let provider = FixtureProvider::new(ProviderBudget {
            max_prompt_bytes: 32,
            max_response_bytes: 32,
        });
        let mut cache = ProviderCache::new(1).expect("cache");
        let request = ProviderRequest {
            prompt: "one".into(),
            model: "fixture".into(),
            prompt_version: "v1".into(),
        };
        let first = cache
            .get_or_fetch(&provider, request.clone())
            .expect("fetch");
        let second = cache.get_or_fetch(&provider, request).expect("replay");
        assert_eq!(first, second);
        assert_eq!(cache.len(), 1);
        cache
            .get_or_fetch(
                &provider,
                ProviderRequest {
                    prompt: "two".into(),
                    model: "fixture".into(),
                    prompt_version: "v1".into(),
                },
            )
            .expect("second fetch");
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn corrupted_provider_cache_is_rejected_on_deserialize() {
        let provider = FixtureProvider::new(ProviderBudget {
            max_prompt_bytes: 32,
            max_response_bytes: 32,
        });
        let mut cache = ProviderCache::new(1).expect("cache");
        cache
            .get_or_fetch(
                &provider,
                ProviderRequest {
                    prompt: "one".into(),
                    model: "fixture".into(),
                    prompt_version: "v1".into(),
                },
            )
            .expect("fetch");
        let mut value = serde_json::to_value(&cache).expect("serialize");
        value["artifacts"]
            .as_object_mut()
            .expect("map")
            .values_mut()
            .next()
            .expect("artifact")["content_hash"] = serde_json::Value::String("bad".into());
        assert!(serde_json::from_value::<ProviderCache>(value).is_err());
    }

    #[test]
    fn provider_artifact_commits_to_document_store_transactionally() {
        let provider = FixtureProvider::new(ProviderBudget {
            max_prompt_bytes: 32,
            max_response_bytes: 32,
        });
        let artifact = provider
            .fetch(ProviderRequest {
                prompt: "Paris".into(),
                model: "fixture".into(),
                prompt_version: "v1".into(),
            })
            .expect("fetch");
        let mut store = lexflex_documents::DocumentStore::default();
        let document = ingest_artifact_transactionally(&mut store, artifact, "provider-doc".into())
            .expect("commit");
        assert_eq!(document.document_id, "provider-doc");
        assert_eq!(store.documents().count(), 1);

        let duplicate = provider
            .fetch(ProviderRequest {
                prompt: "Paris".into(),
                model: "fixture".into(),
                prompt_version: "v1".into(),
            })
            .expect("fetch duplicate");
        assert!(
            ingest_artifact_transactionally(&mut store, duplicate, "provider-doc".into()).is_err()
        );
        assert_eq!(store.documents().count(), 1);
    }
}
