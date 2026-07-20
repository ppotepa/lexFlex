#![forbid(unsafe_code)]

use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const MAX_SESSION_RECORD_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredArtifact {
    pub artifact_id: String,
    pub sha256: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SessionId(String);

impl SessionId {
    pub fn new(value: impl Into<String>) -> Result<Self, SessionStoreError> {
        let value = value.into();
        validate_session_id(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SessionId {
    type Error = SessionStoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<SessionId> for String {
    fn from(value: SessionId) -> Self {
        value.0
    }
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
    #[error("session record exceeds resource limit")]
    ResourceLimit,
}

#[derive(Debug, Clone)]
pub struct SessionStore<T> {
    root: PathBuf,
    schema: u32,
    marker: PhantomData<T>,
}

pub trait SessionPersistence<T>: Send + Sync {
    fn load(&self, session_id: &str) -> Result<T, SessionStoreError>;

    fn save(&self, session_id: &str, payload: &T) -> Result<StoredArtifact, SessionStoreError>;
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
        let bytes = match read_bounded(&path) {
            Ok(bytes) => bytes,
            Err(SessionStoreError::ResourceLimit) => {
                return Err(SessionStoreError::ResourceLimit);
            }
            Err(SessionStoreError::Io {
                kind: std::io::ErrorKind::NotFound,
                ..
            }) => {
                return Err(SessionStoreError::NotFound { path });
            }
            Err(error) => return Err(error),
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
        let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let temp_path = path.with_file_name(format!(
            "{}.{}.{}.tmp",
            session_id,
            std::process::id(),
            sequence
        ));
        let mut temporary_file =
            fs::File::create(&temp_path).map_err(|error| SessionStoreError::Io {
                path: temp_path.clone(),
                kind: error.kind(),
            })?;
        use std::io::Write;
        temporary_file
            .write_all(&bytes)
            .and_then(|_| temporary_file.sync_all())
            .map_err(|error| SessionStoreError::Io {
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

    pub fn migrate<F>(
        &self,
        session_id: &str,
        from_schema: u32,
        migrate_payload: F,
    ) -> Result<StoredArtifact, SessionStoreError>
    where
        F: FnOnce(serde_json::Value) -> Result<T, String>,
    {
        validate_session_id(session_id)?;
        let path = self.session_path(session_id);
        let bytes = match read_bounded(&path) {
            Ok(bytes) => bytes,
            Err(SessionStoreError::Io {
                kind: std::io::ErrorKind::NotFound,
                ..
            }) => return Err(SessionStoreError::NotFound { path }),
            Err(error) => return Err(error),
        };
        let raw: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|error| SessionStoreError::Serde {
                path: path.clone(),
                message: error.to_string(),
            })?;
        let schema = raw
            .get("schema")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| SessionStoreError::Serde {
                path: path.clone(),
                message: "session record has no schema".into(),
            })? as u32;
        if schema != from_schema {
            return Err(SessionStoreError::SchemaMismatch {
                found: schema,
                expected: from_schema,
            });
        }
        let stored_id = raw
            .get("session_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| SessionStoreError::Serde {
                path: path.clone(),
                message: "session record has no session_id".into(),
            })?;
        if stored_id != session_id {
            return Err(SessionStoreError::SessionIdMismatch {
                found: stored_id.to_owned(),
                requested: session_id.to_owned(),
            });
        }
        let payload = raw
            .get("payload")
            .cloned()
            .ok_or_else(|| SessionStoreError::Serde {
                path,
                message: "session record has no payload".into(),
            })?;
        let payload = migrate_payload(payload).map_err(|message| SessionStoreError::Serde {
            path: self.session_path(session_id),
            message,
        })?;
        self.save(session_id, &payload)
    }

    pub fn session_path(&self, session_id: &str) -> PathBuf {
        self.root.join(format!("{session_id}.json"))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, SessionStoreError> {
    use std::io::Read;
    let file = fs::File::open(path).map_err(|error| SessionStoreError::Io {
        path: path.to_owned(),
        kind: error.kind(),
    })?;
    let mut bytes = Vec::new();
    file.take((MAX_SESSION_RECORD_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| SessionStoreError::Io {
            path: path.to_owned(),
            kind: error.kind(),
        })?;
    if bytes.len() > MAX_SESSION_RECORD_BYTES {
        return Err(SessionStoreError::ResourceLimit);
    }
    Ok(bytes)
}

impl<T> SessionPersistence<T> for SessionStore<T>
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync,
{
    fn load(&self, session_id: &str) -> Result<T, SessionStoreError> {
        SessionStore::load(self, session_id)
    }

    fn save(&self, session_id: &str, payload: &T) -> Result<StoredArtifact, SessionStoreError> {
        SessionStore::save(self, session_id, payload)
    }
}

impl<T> SessionPersistence<T> for Box<dyn SessionPersistence<T>> {
    fn load(&self, session_id: &str) -> Result<T, SessionStoreError> {
        self.as_ref().load(session_id)
    }

    fn save(&self, session_id: &str, payload: &T) -> Result<StoredArtifact, SessionStoreError> {
        self.as_ref().save(session_id, payload)
    }
}

pub fn validate_session_id(session_id: &str) -> Result<(), SessionStoreError> {
    const MAX_SESSION_ID_LENGTH: usize = 128;
    let ok = !session_id.is_empty()
        && session_id.len() <= MAX_SESSION_ID_LENGTH
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
    fn session_id_is_validated_and_round_trips_through_serde() {
        let id = SessionId::new("demo-session_1").expect("valid id");
        assert_eq!(id.as_str(), "demo-session_1");
        let encoded = serde_json::to_string(&id).expect("serialize");
        let decoded: SessionId = serde_json::from_str(&encoded).expect("deserialize");
        assert_eq!(decoded, id);
    }

    #[test]
    fn session_id_rejects_paths_and_empty_values() {
        for value in ["", "../escape", "bad/id", "white space"] {
            assert!(
                SessionId::new(value).is_err(),
                "accepted invalid id: {value}"
            );
        }
        assert!(SessionId::new("x".repeat(129)).is_err());
    }

    #[test]
    fn migration_is_explicit_and_failure_preserves_old_record() {
        let root = temp_root();
        let store = SessionStore::<Payload>::new(&root, 2);
        fs::create_dir_all(store.root()).expect("create dir");
        let old = serde_json::json!({
            "schema": 1,
            "session_id": "demo-session",
            "payload": {"value": "legacy"}
        });
        let path = store.session_path("demo-session");
        fs::write(&path, serde_json::to_vec(&old).expect("serialize")).expect("write");
        let before = fs::read(&path).expect("read");

        let failure = store
            .migrate("demo-session", 1, |_| {
                Err("unsupported legacy payload".into())
            })
            .expect_err("migration must fail");
        assert!(matches!(failure, SessionStoreError::Serde { .. }));
        assert_eq!(fs::read(&path).expect("read"), before);

        store
            .migrate("demo-session", 1, |payload| {
                Ok(Payload {
                    value: payload["value"].as_str().unwrap_or_default().to_owned(),
                })
            })
            .expect("migration");
        assert_eq!(store.load("demo-session").expect("load").value, "legacy");
        let _ = fs::remove_dir_all(root);
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
        let entries = fs::read_dir(store.root())
            .expect("read store root")
            .collect::<Result<Vec<_>, _>>()
            .expect("read entries");
        assert_eq!(entries.len(), 1, "temporary file must not remain");
        let _ = fs::remove_dir_all(store.root());
    }

    #[test]
    fn concurrent_saves_use_distinct_temporary_files() {
        let root = temp_root();
        let store = SessionStore::<Payload>::new(&root, 7);
        let workers = (0..16)
            .map(|index| {
                let store = store.clone();
                std::thread::spawn(move || {
                    store
                        .save(
                            "demo-session",
                            &Payload {
                                value: format!("value-{index}"),
                            },
                        )
                        .expect("concurrent save");
                })
            })
            .collect::<Vec<_>>();
        for worker in workers {
            worker.join().expect("worker should finish");
        }
        let loaded = store.load("demo-session").expect("final record is valid");
        assert!(loaded.value.starts_with("value-"));
        let entries = fs::read_dir(store.root())
            .expect("read store root")
            .collect::<Result<Vec<_>, _>>()
            .expect("read entries");
        assert_eq!(entries.len(), 1, "temporary files must not remain");
        let _ = fs::remove_dir_all(root);
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

    #[test]
    fn oversized_session_is_rejected_before_deserialization() {
        let store = SessionStore::<Payload>::new(temp_root(), 7);
        fs::create_dir_all(store.root()).expect("create dir");
        fs::write(
            store.session_path("demo-session"),
            vec![b'{'; MAX_SESSION_RECORD_BYTES + 1],
        )
        .expect("write oversized session");
        assert_eq!(
            store.load("demo-session"),
            Err(SessionStoreError::ResourceLimit)
        );
        let _ = fs::remove_dir_all(store.root());
    }
}
