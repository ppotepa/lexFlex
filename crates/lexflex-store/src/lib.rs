#![forbid(unsafe_code)]

use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredArtifact {
    pub artifact_id: String,
    pub sha256: String,
    pub kind: String,
}

impl StoredArtifact {
    pub fn from_bytes(kind: impl Into<String>, bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let sha256 = format!("{:x}", hasher.finalize());
        Self {
            artifact_id: format!("artifact:{sha256}"),
            sha256,
            kind: kind.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRecord<T> {
    pub schema: u32,
    pub session_id: String,
    pub payload: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SessionStoreError {
    #[error("invalid session id: {0}")]
    InvalidSessionId(String),
    #[error("session not found: {path}")]
    NotFound { path: PathBuf },
    #[error("io error at {path}: {kind:?}")]
    Io {
        path: PathBuf,
        kind: std::io::ErrorKind,
    },
    #[error("serialization error at {path}: {message}")]
    Serde { path: PathBuf, message: String },
    #[error("unsupported session schema {found} (expected {expected})")]
    SchemaMismatch { found: u32, expected: u32 },
    #[error("session id mismatch (file has {found}, requested {requested})")]
    SessionIdMismatch { found: String, requested: String },
}

#[derive(Debug, Clone)]
pub struct SessionStore<T> {
    root: PathBuf,
    schema: u32,
    marker: PhantomData<T>,
}

impl<T> SessionStore<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn new(root: impl Into<PathBuf>, schema: u32) -> Self {
        Self {
            root: root.into(),
            schema,
            marker: PhantomData,
        }
    }

    pub fn load(&self, session_id: &str) -> Result<T, SessionStoreError> {
        validate_session_id(session_id)?;
        let path = self.session_path(session_id);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(SessionStoreError::NotFound { path });
            }
            Err(error) => {
                return Err(SessionStoreError::Io {
                    path,
                    kind: error.kind(),
                });
            }
        };
        let record: SessionRecord<T> =
            serde_json::from_slice(&bytes).map_err(|error| SessionStoreError::Serde {
                path: path.clone(),
                message: error.to_string(),
            })?;
        if record.schema != self.schema {
            return Err(SessionStoreError::SchemaMismatch {
                found: record.schema,
                expected: self.schema,
            });
        }
        if record.session_id != session_id {
            return Err(SessionStoreError::SessionIdMismatch {
                found: record.session_id,
                requested: session_id.to_string(),
            });
        }
        Ok(record.payload)
    }

    pub fn save(&self, session_id: &str, payload: &T) -> Result<StoredArtifact, SessionStoreError> {
        validate_session_id(session_id)?;
        fs::create_dir_all(&self.root).map_err(|error| SessionStoreError::Io {
            path: self.root.clone(),
            kind: error.kind(),
        })?;
        let path = self.session_path(session_id);
        let record = SessionRecord {
            schema: self.schema,
            session_id: session_id.to_string(),
            payload: payload.clone(),
        };
        let bytes =
            serde_json::to_vec_pretty(&record).map_err(|error| SessionStoreError::Serde {
                path: path.clone(),
                message: error.to_string(),
            })?;
        let artifact = StoredArtifact::from_bytes("session", &bytes);
        let temp_path = path.with_extension("json.tmp");
        fs::write(&temp_path, bytes).map_err(|error| SessionStoreError::Io {
            path: temp_path.clone(),
            kind: error.kind(),
        })?;
        fs::rename(&temp_path, &path).map_err(|error| {
            let _ = fs::remove_file(&temp_path);
            SessionStoreError::Io {
                path: path.clone(),
                kind: error.kind(),
            }
        })?;
        Ok(artifact)
    }

    pub fn session_path(&self, session_id: &str) -> PathBuf {
        self.root.join(format!("{session_id}.json"))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

fn validate_session_id(session_id: &str) -> Result<(), SessionStoreError> {
    let ok = !session_id.is_empty()
        && session_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
    if ok {
        Ok(())
    } else {
        Err(SessionStoreError::InvalidSessionId(session_id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct Payload {
        value: String,
    }

    fn temp_root() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        std::env::temp_dir().join(format!("lexflex-store-{stamp}"))
    }

    #[test]
    fn session_round_trip_is_versioned_and_atomic() {
        let store = SessionStore::<Payload>::new(temp_root(), 7);
        let payload = Payload {
            value: "hello".into(),
        };
        let artifact = store.save("demo-session", &payload).expect("save");
        assert_eq!(artifact.kind, "session");
        let loaded = store.load("demo-session").expect("load");
        assert_eq!(loaded, payload);
        let _ = fs::remove_dir_all(store.root());
    }

    #[test]
    fn raw_legacy_schema_is_rejected() {
        let store = SessionStore::<Payload>::new(temp_root(), 7);
        fs::create_dir_all(store.root()).expect("create dir");
        let legacy = serde_json::to_vec_pretty(&SessionRecord {
            schema: 1,
            session_id: "demo-session".into(),
            payload: Payload {
                value: "legacy".into(),
            },
        })
        .expect("serialize");
        fs::write(store.session_path("demo-session"), legacy).expect("write");
        let err = store.load("demo-session").expect_err("must reject");
        assert!(matches!(err, SessionStoreError::SchemaMismatch { .. }));
        let _ = fs::remove_dir_all(store.root());
    }

    #[test]
    fn missing_session_is_reported_explicitly() {
        let store = SessionStore::<Payload>::new(temp_root(), 7);
        let err = store.load("demo-session").expect_err("must reject");
        assert!(matches!(err, SessionStoreError::NotFound { .. }));
    }

    #[test]
    fn corrupted_session_is_reported_explicitly() {
        let store = SessionStore::<Payload>::new(temp_root(), 7);
        fs::create_dir_all(store.root()).expect("create dir");
        fs::write(store.session_path("demo-session"), b"not-json").expect("write");
        let err = store.load("demo-session").expect_err("must reject");
        assert!(matches!(err, SessionStoreError::Serde { .. }));
        let _ = fs::remove_dir_all(store.root());
    }
}
