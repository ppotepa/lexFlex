use lexflex_engine::{
    api::{input::TextInput, request::EngineRequest, response::EngineResponse},
    runtime::LexFlexRuntime,
    session::EngineSessionState,
    EngineErrorCode,
};
use lexflex_model::LanguageId;
use lexflex_store::{SessionPersistence, SessionStoreError, StoredArtifact};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Default)]
struct FailingStore {
    payload: Arc<Mutex<Option<EngineSessionState>>>,
    fail_save: Arc<AtomicBool>,
}

impl FailingStore {
    fn fail_saves(&self) {
        self.fail_save.store(true, Ordering::SeqCst);
    }

    fn payload(&self) -> Option<EngineSessionState> {
        self.payload.lock().expect("payload lock").clone()
    }
}

impl SessionPersistence<EngineSessionState> for FailingStore {
    fn load(&self, session_id: &str) -> Result<EngineSessionState, SessionStoreError> {
        self.payload
            .lock()
            .expect("payload lock")
            .clone()
            .ok_or_else(|| SessionStoreError::NotFound {
                path: PathBuf::from(format!("{session_id}.json")),
            })
    }

    fn save(
        &self,
        _session_id: &str,
        payload: &EngineSessionState,
    ) -> Result<StoredArtifact, SessionStoreError> {
        if self.fail_save.load(Ordering::SeqCst) {
            return Err(SessionStoreError::Io {
                path: PathBuf::from("failing-store.json"),
                kind: std::io::ErrorKind::Other,
            });
        }
        *self.payload.lock().expect("payload lock") = Some(payload.clone());
        Ok(StoredArtifact {
            artifact_id: "artifact:test".into(),
            sha256: "test".into(),
            kind: "session".into(),
        })
    }
}

fn runtime_with_store(store: FailingStore, suffix: &str) -> LexFlexRuntime {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../");
    LexFlexRuntime::with_persistence(
        format!("rollback-{suffix}-{stamp}"),
        std::env::temp_dir(),
        repo_root.join("data/model"),
        repo_root.join("data/languages"),
        Box::new(store),
    )
    .expect("runtime")
}

fn inspect(runtime: &mut LexFlexRuntime) -> (usize, usize, String) {
    match runtime.handle(EngineRequest::InspectSession) {
        EngineResponse::SessionInspection {
            assertion_count,
            evidence_count,
            snapshot_hash,
            ..
        } => (assertion_count, evidence_count, snapshot_hash),
        other => panic!("unexpected inspect response: {other:?}"),
    }
}

fn ask_capital(runtime: &mut LexFlexRuntime) -> Vec<lexflex_lingua::QuerySolution> {
    match runtime.handle(EngineRequest::AskText {
        input: TextInput {
            source_id: "source:rollback:question".into(),
            language: LanguageId::new("en").expect("language"),
            text: "What is the capital of France?".into(),
        },
        evidence_policy: lexflex_lingua::EvidencePolicy::Required,
        limit: Some(10),
    }) {
        EngineResponse::TextAnswer { solutions, .. } => solutions,
        other => panic!("unexpected ask response: {other:?}"),
    }
}

#[test]
fn store_error_during_ingest_restores_session_state() {
    let store = FailingStore::default();
    let mut runtime = runtime_with_store(store.clone(), "ingest");
    let before = inspect(&mut runtime);
    let query_before = ask_capital(&mut runtime);
    let persisted_before = store.payload();
    store.fail_saves();

    let response = runtime.handle(EngineRequest::IngestText {
        input: TextInput {
            source_id: "source:rollback:ingest".into(),
            language: LanguageId::new("en").expect("language"),
            text: "Paris is the capital of France.".into(),
        },
    });

    match response {
        EngineResponse::Error { code, .. } => {
            assert_eq!(code, EngineErrorCode::StoreError);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    assert_eq!(before, inspect(&mut runtime));
    assert_eq!(query_before, ask_capital(&mut runtime));
    assert_eq!(persisted_before, store.payload());
}

#[test]
fn store_error_during_clear_restores_session_state() {
    let store = FailingStore::default();
    let mut runtime = runtime_with_store(store.clone(), "clear");
    let seed = runtime.handle(EngineRequest::IngestText {
        input: TextInput {
            source_id: "source:rollback:seed".into(),
            language: LanguageId::new("en").expect("language"),
            text: "Paris is the capital of France.".into(),
        },
    });
    assert!(matches!(seed, EngineResponse::TextIngested { .. }));
    let before = inspect(&mut runtime);
    let query_before = ask_capital(&mut runtime);
    assert_eq!(query_before.len(), 1);
    let persisted_before = store.payload();
    store.fail_saves();

    let response = runtime.handle(EngineRequest::ClearSession);

    match response {
        EngineResponse::Error { code, .. } => {
            assert_eq!(code, EngineErrorCode::StoreError);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    assert_eq!(before, inspect(&mut runtime));
    assert_eq!(query_before, ask_capital(&mut runtime));
    assert_eq!(persisted_before, store.payload());
}
