use crate::engine::{LexFlexEngine, EngineRequest, EngineResponse, TraceEvent};
use serde_json::Value;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Mutex;

#[derive(Debug)]
pub struct ChatServiceError(pub String);

impl Display for ChatServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { f.write_str(&self.0) }
}
impl Error for ChatServiceError {}

pub struct ChatService { engine: Mutex<LexFlexEngine> }

#[derive(Debug)]
pub enum ChatJob { Engine { request: EngineRequest } }

#[derive(Debug)]
pub enum ChatJobOutput { Engine(EngineResponse) }

#[derive(Debug)]
pub struct ChatJobResult { pub label: String, pub result: Result<ChatJobOutput, String> }

#[derive(Debug, Clone)]
pub struct ChatProgressEvent {
    pub run_id: String,
    pub request_id: String,
    pub sequence: u64,
    pub stage: String,
    pub label: String,
    pub details: Vec<String>,
}

#[derive(Debug)]
pub enum ChatWorkerEvent {
    Progress(ChatProgressEvent),
    Result(ChatJobResult),
}

pub struct ChatWorker { sender: Sender<ChatJob>, receiver: Receiver<ChatWorkerEvent> }

impl ChatWorker {
    pub fn spawn(service: ChatService) -> Self {
        let (sender, jobs) = mpsc::channel();
        let (results, receiver) = mpsc::channel();
        let progress = results.clone();
        if let Ok(mut engine) = service.engine.lock() {
            engine.set_observer(std::sync::Arc::new(move |event| {
                let _ = progress.send(ChatWorkerEvent::Progress(humanize_progress_event(event)));
            }));
        }
        std::thread::spawn(move || {
            while let Ok(ChatJob::Engine { request }) = jobs.recv() {
                let result = service.handle_engine(request).map(ChatJobOutput::Engine).map_err(|error| error.to_string());
                if results.send(ChatWorkerEvent::Result(ChatJobResult { label: "engine".into(), result })).is_err() { break; }
            }
        });
        Self { sender, receiver }
    }
    pub fn submit(&self, job: ChatJob) -> Result<(), ChatServiceError> { self.sender.send(job).map_err(|_| ChatServiceError("engine worker stopped".into())) }
    pub fn try_receive(&self) -> Option<ChatWorkerEvent> { match self.receiver.try_recv() { Ok(value) => Some(value), Err(TryRecvError::Empty | TryRecvError::Disconnected) => None } }
}

fn humanize_progress_event(event: TraceEvent) -> ChatProgressEvent {
    let label = match event.stage.as_str() {
        "request" => "Accepted request",
        "runtime.data_root" => "Loaded runtime configuration",
        "language.detected" => "Detected language",
        "language.parse_failed" => "Parse failed",
        "source.discovery" => "Derived source candidates",
        "source.selected" => "Selected source",
        "source.auto.failed" => "Source acquisition failed",
        "query.plan" => "Built query plan",
        "query.execution" => "Executed query",
        "answer.unknown" => "No supported answer yet",
        "debug.facts.selected" => "Prepared fact page",
        "debug.selector" => "Parsed fact selector",
        "debug.entity_matches" => "Resolved matching entities",
        "debug.claims.scanned" => "Scanned claims",
        "debug.response" => "Prepared debug response",
        _ => event.stage.as_str(),
    }
    .to_string();
    ChatProgressEvent {
        run_id: event.run_id,
        request_id: event.request_id,
        sequence: event.sequence,
        stage: event.stage.clone(),
        label,
        details: summarize_payload(&event.stage, event.payload.as_ref()),
    }
}

fn summarize_payload(stage: &str, payload: Option<&Value>) -> Vec<String> {
    let Some(payload) = payload else { return Vec::new(); };
    match stage {
        "language.detected" => payload
            .get("language")
            .and_then(Value::as_str)
            .map(|value| vec![format!("Language: {value}")])
            .unwrap_or_default(),
        "source.discovery" => payload
            .get("candidates")
            .and_then(Value::as_array)
            .map(|values| {
                let names = values.iter().filter_map(Value::as_str).collect::<Vec<_>>();
                if names.is_empty() {
                    Vec::new()
                } else {
                    vec![format!("Candidates: {}", names.join(", "))]
                }
            })
            .unwrap_or_default(),
        "source.selected" => {
            let mut details = Vec::new();
            if let Some(title) = payload.get("title").and_then(Value::as_str) {
                details.push(format!("Source: {title}"));
            }
            if let Some(origin) = payload.get("origin").and_then(Value::as_str) {
                details.push(format!("Origin: {origin}"));
            }
            details
        }
        "query.execution" => payload
            .get("row_count")
            .and_then(Value::as_u64)
            .map(|count| vec![format!("Rows: {count}")])
            .unwrap_or_default(),
        "debug.facts.selected" => {
            let total = payload.get("total").and_then(Value::as_u64);
            let returned = payload.get("returned").and_then(Value::as_u64);
            match (returned, total) {
                (Some(returned), Some(total)) => vec![format!("Returned {returned} of {total}")],
                _ => Vec::new(),
            }
        }
        "language.parse_failed" => payload
            .get("error")
            .map(compact_value)
            .map(|value| vec![value])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn compact_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        _ => serde_json::to_string(value).unwrap_or_else(|_| "<unavailable>".into()),
    }
}

impl ChatService {
    pub fn new(data_dir: &str, offline: bool, _trace_mode: crate::chat::TraceMode, source_policy: crate::engine::SourceFetchPolicy) -> Result<Self, ChatServiceError> {
        let session_id = "chat-session";
        let mut engine = LexFlexEngine::new(data_dir, session_id, offline).map_err(|error| ChatServiceError(format!("engine initialization failed: {error:?}")))?;
        engine = engine.with_auto_source_policy(if offline { crate::engine::SourceFetchPolicy::SnapshotOnly } else { source_policy });
        let store_path = engine
            .store
            .as_ref()
            .and_then(|store| store.path_for(session_id).ok());
        if store_path.as_ref().is_some_and(|path| path.exists()) {
            if let Err(error) = engine.load_session() {
                tracing::warn!("chat session load failed; starting fresh: {error:?}");
                engine
                    .store
                    .as_ref()
                    .ok_or_else(|| ChatServiceError("session store disabled".into()))?
                    .quarantine_session(session_id)
                    .map_err(|quarantine_error| {
                        ChatServiceError(format!(
                            "session load failed: {error:?}; quarantine failed: {quarantine_error:?}"
                        ))
                    })?;
                engine = LexFlexEngine::new(data_dir, session_id, offline)
                    .map_err(|init_error| ChatServiceError(format!("engine reinitialization failed: {init_error:?}")))?;
                engine = engine.with_auto_source_policy(if offline {
                    crate::engine::SourceFetchPolicy::SnapshotOnly
                } else {
                    source_policy
                });
            }
        }
        Ok(Self { engine: Mutex::new(engine) })
    }
    pub fn handle_engine(&self, request: EngineRequest) -> Result<EngineResponse, ChatServiceError> {
        let mut engine = self
            .engine
            .lock()
            .map_err(|_| ChatServiceError("engine lock poisoned".into()))?;
        let response = engine.handle(request);
        engine
            .save_session()
            .map_err(|error| ChatServiceError(format!("session save failed: {error:?}")))?;
        Ok(response)
    }
}
