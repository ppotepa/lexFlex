use crate::engine::{ConversationEngine, EngineRequest, EngineResponse};
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

pub struct ChatWorker { sender: Sender<ChatJob>, receiver: Receiver<ChatJobResult> }

impl ChatWorker {
    pub fn spawn(service: ChatService) -> Self {
        let (sender, jobs) = mpsc::channel();
        let (results, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            while let Ok(ChatJob::Engine { request }) = jobs.recv() {
                let result = service.handle_engine(request).map(ChatJobOutput::Engine).map_err(|error| error.to_string());
                if results.send(ChatJobResult { label: "engine".into(), result }).is_err() { break; }
            }
        });
        Self { sender, receiver }
    }
    pub fn submit(&self, job: ChatJob) -> Result<(), ChatServiceError> { self.sender.send(job).map_err(|_| ChatServiceError("engine worker stopped".into())) }
    pub fn try_receive(&self) -> Option<ChatJobResult> { match self.receiver.try_recv() { Ok(value) => Some(value), Err(TryRecvError::Empty | TryRecvError::Disconnected) => None } }
}

impl ChatService {
    pub fn new(data_dir: &str, offline: bool, _trace_mode: crate::chat::TraceMode) -> Result<Self, ChatServiceError> {
        let engine = ConversationEngine::new(data_dir, "chat-session", offline).map_err(|error| ChatServiceError(format!("engine initialization failed: {error:?}")))?;
        Ok(Self { engine: Mutex::new(engine) })
    }
    pub fn handle_engine(&self, request: EngineRequest) -> Result<EngineResponse, ChatServiceError> {
        self.engine.lock().map_err(|_| ChatServiceError("engine lock poisoned".into())).map(|mut engine| engine.handle(request))
    }
}
