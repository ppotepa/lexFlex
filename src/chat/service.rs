use crate::engine::{ConversationEngine, EngineRequest, EngineResponse, TraceEvent};
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

pub struct ChatService { engine: Mutex<ConversationEngine> }

#[derive(Debug)]
pub enum ChatJob { Engine { request: EngineRequest } }

#[derive(Debug)]
pub enum ChatJobOutput { Engine(EngineResponse) }

#[derive(Debug)]
pub struct ChatJobResult { pub label: String, pub result: Result<ChatJobOutput, String> }

#[derive(Debug)]
pub enum ChatWorkerEvent {
    Progress(TraceEvent),
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
                let _ = progress.send(ChatWorkerEvent::Progress(event));
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

impl ChatService {
    pub fn new(data_dir: &str, offline: bool, _trace_mode: crate::chat::TraceMode, source_policy: crate::engine::SourceFetchPolicy) -> Result<Self, ChatServiceError> {
        let mut engine = ConversationEngine::new(data_dir, "chat-session", offline).map_err(|error| ChatServiceError(format!("engine initialization failed: {error:?}")))?;
        engine = engine.with_auto_source_policy(if offline { crate::engine::SourceFetchPolicy::SnapshotOnly } else { source_policy });
        let store_path = engine
            .store
            .as_ref()
            .and_then(|store| store.path_for("chat-session").ok());
        if store_path.as_ref().is_some_and(|path| path.exists()) {
            engine
                .load_session()
                .map_err(|error| ChatServiceError(format!("session load failed: {error:?}")))?;
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
