use crate::chat::service::ChatResponse;
use crate::chat::conversation_qa::ConversationKnowledgeSession;
use crate::chat::semantic::SemanticConversationMemory;
use crate::chat::settings::{ChatSettings, Verbosity};
use crate::chat::transcript::{MessageBlock, MessageMeta, MessageRole, Transcript};
use crate::chat::trace::ChatTraceEntry;
use crate::runtime::DocumentArtifactBundle;
use crate::chat::{ChatOptions, TraceMode};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatMode {
    Chat,
    Translate,
}

impl ChatMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ChatMode::Chat => "chat",
            ChatMode::Translate => "translate",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatContext {
    pub source_lang: String,
    pub target_lang: String,
    pub mode: ChatMode,
    #[serde(default)]
    pub chat_language_override: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub data_dir: String,
    pub trace_mode: TraceMode,
    pub offline: bool,
    pub endpoint_label: Option<String>,
    pub model_label: Option<String>,
    pub system_prompt_label: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ComposerState {
    pub input: String,
    pub cursor: usize,
    pub history: Vec<String>,
    pub history_cursor: Option<usize>,
    pub history_draft: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayKind {
    CommandPalette,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusTarget {
    Composer,
    CommandPopup,
}

#[derive(Debug, Clone, Default)]
pub struct CommandPopupState {
    pub suggestions: Vec<crate::chat::commands::SlashSuggestion>,
    pub selected_index: usize,
}

#[derive(Debug, Clone)]
pub struct OverlayState {
    pub active: Option<OverlayKind>,
    pub focus: FocusTarget,
    pub command_popup: CommandPopupState,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            active: None,
            focus: FocusTarget::Composer,
            command_popup: CommandPopupState::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TranscriptViewport {
    pub scroll_offset: u16,
    pub stick_to_bottom: bool,
}

impl TranscriptViewport {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
            stick_to_bottom: true,
        }
    }

    pub fn page_up(&mut self, amount: u16, max_scroll: u16) {
        let current = if self.stick_to_bottom {
            max_scroll
        } else {
            self.scroll_offset
        };
        self.scroll_offset = current.saturating_sub(amount);
        self.stick_to_bottom = false;
    }

    pub fn page_down(&mut self, amount: u16, max_scroll: u16) {
        let next = self.scroll_offset.saturating_add(amount);
        if next >= max_scroll {
            self.jump_to_bottom();
        } else {
            self.scroll_offset = next;
            self.stick_to_bottom = false;
        }
    }

    pub fn set_scroll(&mut self, offset: u16, max_scroll: u16) {
        if offset >= max_scroll {
            self.jump_to_bottom();
        } else {
            self.scroll_offset = offset;
            self.stick_to_bottom = false;
        }
    }

    pub fn jump_to_bottom(&mut self) {
        self.scroll_offset = u16::MAX;
        self.stick_to_bottom = true;
    }
}

#[cfg(test)]
mod tests {
    use super::TranscriptViewport;

    #[test]
    fn viewport_page_up_disables_stick_to_bottom() {
        let mut viewport = TranscriptViewport::new();
        viewport.set_scroll(10, 20);
        viewport.page_up(3, 10);
        assert_eq!(viewport.scroll_offset, 7);
        assert!(!viewport.stick_to_bottom);
    }

    #[test]
    fn viewport_page_down_to_end_restores_stick() {
        let mut viewport = TranscriptViewport::new();
        viewport.set_scroll(6, 10);
        viewport.page_down(5, 10);
        assert_eq!(viewport.scroll_offset, u16::MAX);
        assert!(viewport.stick_to_bottom);
    }
}

#[derive(Debug, Clone)]
pub struct PendingRequest {
    pub label: String,
    pub started_at: Instant,
}

#[derive(Debug, Clone)]
pub struct ChatSession {
    pub context: ChatContext,
    pub settings: ChatSettings,
    pub transcript: Transcript,
    pub knowledge: ConversationKnowledgeSession,
    pub semantic: Option<SemanticConversationMemory>,
    pub trace_log: Vec<ChatTraceEntry>,
    pub document_bundles: Vec<DocumentArtifactBundle>,
    pub composer: ComposerState,
    pub overlays: OverlayState,
    pub pending: Option<PendingRequest>,
    pub backend_ready: bool,
    pub should_quit: bool,
    pub runtime: RuntimeConfig,
    pub viewport: TranscriptViewport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub context: ChatContext,
    pub settings: ChatSettings,
    pub transcript: Transcript,
}

impl ChatSession {
    pub fn new(options: ChatOptions) -> Self {
        let semantic = crate::api::LexFlexAPI::builder()
            .data_dir(&options.data_dir)
            .build()
            .ok()
            .map(SemanticConversationMemory::new);
        let endpoint_label = options
            .base_url
            .clone()
            .or_else(|| std::env::var("LEXFLEX_LLM_BASE_URL").ok());
        let model_label = options
            .model
            .clone()
            .or_else(|| std::env::var("LEXFLEX_LLM_MODEL").ok());
        let system_prompt_label = options
            .system_prompt
            .clone()
            .or_else(|| std::env::var("LEXFLEX_LLM_SYSTEM_PROMPT").ok());
        Self {
            context: ChatContext {
                source_lang: options.from,
                target_lang: options.to,
                mode: ChatMode::Chat,
                chat_language_override: None,
            },
            settings: ChatSettings::default(),
            transcript: Transcript::new(),
            knowledge: ConversationKnowledgeSession::default(),
            semantic,
            trace_log: Vec::new(),
            document_bundles: Vec::new(),
            composer: ComposerState::default(),
            overlays: OverlayState::default(),
            pending: None,
            backend_ready: true,
            should_quit: false,
            runtime: RuntimeConfig {
                data_dir: options.data_dir,
                trace_mode: options.trace_mode,
                offline: options.offline,
                endpoint_label,
                model_label,
                system_prompt_label,
            },
            viewport: TranscriptViewport::new(),
        }
    }

    pub fn push_user_message(&mut self, text: String) -> usize {
        let turn = self.transcript.next_turn;
        self.transcript.push(
            MessageRole::User,
            "you",
            vec![MessageBlock::Paragraph(text)],
            MessageMeta::default(),
        );
        self.viewport.jump_to_bottom();
        turn
    }

    pub fn push_system_message(&mut self, text: impl Into<String>) {
        self.transcript.push(
            MessageRole::System,
            "system",
            vec![MessageBlock::Paragraph(text.into())],
            MessageMeta::default(),
        );
        self.viewport.jump_to_bottom();
    }

    pub fn push_error_message(&mut self, text: impl Into<String>) {
        self.transcript.push(
            MessageRole::Error,
            "error",
            vec![MessageBlock::Paragraph(text.into())],
            MessageMeta::default(),
        );
        self.viewport.jump_to_bottom();
    }

    pub fn push_response(&mut self, response: ChatResponse, command: Option<String>) {
        let mut blocks = response.blocks;
        if !response.notes.is_empty() {
            blocks.push(MessageBlock::BulletList(response.notes));
        }
        self.transcript.push(
            MessageRole::Assistant,
            response.title,
            blocks,
            MessageMeta {
                latency_ms: Some(response.latency_ms),
                command,
            },
        );
        self.viewport.jump_to_bottom();
    }

    pub fn record_trace(&mut self, entry: ChatTraceEntry) {
        self.trace_log.push(entry);
    }

    pub fn trace_notes_for(&self, turn: usize) -> Vec<String> {
        self.trace_log
            .iter()
            .find(|entry| entry.turn == turn)
            .map(ChatTraceEntry::as_notes)
            .unwrap_or_default()
    }

    pub fn finish_trace(
        &mut self,
        turn: usize,
        facts_added: usize,
        answer_status: Option<String>,
        answer: Option<String>,
        evidence: Vec<String>,
        fallback: Option<String>,
    ) {
        if let Some(entry) = self.trace_log.iter_mut().find(|entry| entry.turn == turn) {
            entry.facts_added = facts_added;
            entry.answer_status = answer_status;
            entry.answer = answer;
            entry.evidence = evidence;
            entry.fallback = fallback;
        }
    }

    pub fn clear_session(&mut self) {
        self.transcript.clear();
        self.knowledge.clear();
        if let Some(semantic) = self.semantic.as_mut() {
            semantic.clear();
        }
        self.trace_log.clear();
        self.document_bundles.clear();
        self.composer = ComposerState::default();
        self.pending = None;
        self.viewport = TranscriptViewport::new();
    }

    pub fn set_translation_direction(&mut self, from: String, to: String) {
        self.context.source_lang = from;
        self.context.target_lang = to;
    }

    pub fn set_chat_language_override(&mut self, language: Option<String>) {
        self.context.chat_language_override = language;
    }

    pub fn chat_language_for(&self, text: &str) -> String {
        self.context
            .chat_language_override
            .clone()
            .unwrap_or_else(|| {
                crate::chat::conversation_qa::detect_language(text, &self.context.source_lang)
            })
    }

    pub fn chat_language_label(&self) -> String {
        self.context
            .chat_language_override
            .clone()
            .unwrap_or_else(|| "auto".to_string())
    }

    pub fn context_for_language(&self, language: String) -> ChatContext {
        ChatContext {
            source_lang: language,
            ..self.context.clone()
        }
    }

    pub fn set_mode(&mut self, mode: ChatMode) {
        self.context.mode = mode;
    }

    pub fn cycle_verbosity(&mut self) -> Verbosity {
        self.settings.verbosity = self.settings.verbosity.next();
        self.settings.verbosity
    }

    pub fn status_line(&self) -> String {
        match &self.pending {
            Some(pending) => format!(
                "working on {} for {} ms",
                pending.label,
                pending.started_at.elapsed().as_millis()
            ),
            None => format!(
                "{} | chat:{} | translate:{} -> {} | {} | facts:{}",
                self.context.mode.as_str(),
                self.chat_language_label(),
                self.context.source_lang,
                self.context.target_lang,
                self.settings.verbosity.as_str(),
                self.knowledge.fact_count(),
            ),
        }
    }

    pub fn save_snapshot(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }
        let snapshot = SessionSnapshot {
            context: self.context.clone(),
            settings: self.settings.clone(),
            transcript: self.transcript.clone(),
        };
        fs::write(path, serde_json::to_string_pretty(&snapshot)?)?;
        Ok(())
    }
}
