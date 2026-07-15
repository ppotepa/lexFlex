use crate::chat::settings::{ChatSettings, Verbosity};
use crate::chat::transcript::{MessageBlock, MessageMeta, MessageRole, Transcript, TranscriptNode};
use crate::chat::trace::ChatTraceEntry;
use crate::chat::{ChatOptions, TraceMode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineMode {
    Conversation,
    Translation,
}

impl EngineMode {
    pub fn as_str(self) -> &'static str {
        match self {
            EngineMode::Conversation => "conversation",
            EngineMode::Translation => "translation",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatContext {
    pub source_lang: String,
    pub target_lang: String,
    pub mode: EngineMode,
    #[serde(default)]
    pub chat_language_override: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub data_dir: String,
    pub trace_mode: TraceMode,
    pub offline: bool,
    pub source_policy: crate::engine::SourceFetchPolicy,
    pub engine_session_id: String,
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
    Transcript,
    CommandPopup,
}

impl FocusTarget {
    pub fn is_composer(self) -> bool {
        matches!(self, Self::Composer)
    }

    pub fn is_transcript(self) -> bool {
        matches!(self, Self::Transcript)
    }

    pub fn is_popup(self) -> bool {
        matches!(self, Self::CommandPopup)
    }
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
    use super::{ChatOptions, ChatSession, FocusTarget, PendingRequest, TranscriptViewport};
    use crate::chat::TraceMode;
    use crate::chat::transcript::{MessageBlock, MessageMeta, TranscriptNode};
    use std::time::Instant;

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

    #[test]
    fn streaming_message_receives_live_trace_and_final_answer() {
        let mut session = ChatSession::new(ChatOptions {
            from: "pl".into(), to: "en".into(), data_dir: "data".into(), offline: true,
            trace_mode: TraceMode::Full, source_policy: crate::engine::SourceFetchPolicy::SnapshotOnly,
        });
        session.start_stream("engine");
        session.pending = Some(PendingRequest {
            label: "engine".into(), started_at: Instant::now(),
            current_stage: "queued".into(), current_stage_path: "queued".into(),
            spinner_index: 0,
        });
        session.append_stream_progress(&crate::chat::service::ChatProgressEvent {
            run_id: "run:00000001".into(),
            request_id: "request:test".into(),
            sequence: 1,
            stage: "language.detected".into(),
            label: "Detected language".into(),
            details: vec!["Language: en".into()],
        }, true);
        session.finish_stream(
            "conversation".into(),
            vec![crate::chat::transcript::MessageBlock::Paragraph("Paris".into())],
            crate::chat::transcript::MessageMeta { latency_ms: Some(1), command: None },
            true,
        );
        assert_eq!(session.transcript.messages[0].title, "conversation");
        assert!(matches!(session.transcript.messages[0].blocks[0], crate::chat::transcript::MessageBlock::Paragraph(_)));
        assert!(session.pending.is_none());
    }

    #[test]
    fn toggling_selected_node_flips_expansion_state() {
        let mut session = ChatSession::new(ChatOptions {
            from: "pl".into(), to: "en".into(), data_dir: "data".into(), offline: true,
            trace_mode: TraceMode::Full, source_policy: crate::engine::SourceFetchPolicy::SnapshotOnly,
        });
        session.selected_turn = Some(7);
        assert!(session.toggle_selected_node("facts.0"));
        assert!(session.node_expanded(7, "facts.0", false));
        assert!(session.toggle_selected_node("facts.0"));
        assert!(!session.node_expanded(7, "facts.0", false));
    }

    #[test]
    fn explicit_tree_state_can_close_default_open_nodes() {
        let mut session = ChatSession::new(ChatOptions {
            from: "pl".into(),
            to: "en".into(),
            data_dir: "data".into(),
            offline: true,
            trace_mode: TraceMode::Full,
            source_policy: crate::engine::SourceFetchPolicy::SnapshotOnly,
        });
        session.transcript.push(
            crate::chat::transcript::MessageRole::EngineFacts,
            "facts",
            vec![MessageBlock::Tree(vec![TranscriptNode {
                key: "facts".into(),
                label: "Facts".into(),
                summary: None,
                tone: None,
                expanded: true,
                blocks: vec![],
                children: vec![TranscriptNode {
                    key: "facts.0".into(),
                    label: "First fact".into(),
                    summary: None,
                    tone: None,
                    expanded: false,
                    blocks: vec![],
                    children: vec![],
                }],
            }])],
            MessageMeta::default(),
        );
        session.selected_turn = Some(1);
        assert!(session.node_expanded(1, "facts", true));
        assert!(session.toggle_selected_node("facts"));
        assert!(!session.node_expanded(1, "facts", true));
        assert!(session.toggle_selected_node("facts"));
        assert!(session.node_expanded(1, "facts", false));
        assert!(session.toggle_first_collapsed_node());
        assert!(session.node_expanded(1, "facts.0", false));
    }

    #[test]
    fn main_focus_toggles_between_composer_and_transcript() {
        let mut session = ChatSession::new(ChatOptions {
            from: "pl".into(),
            to: "en".into(),
            data_dir: "data".into(),
            offline: true,
            trace_mode: TraceMode::Full,
            source_policy: crate::engine::SourceFetchPolicy::SnapshotOnly,
        });
        assert!(matches!(session.overlays.focus, FocusTarget::Composer));
        session.toggle_main_focus();
        assert!(matches!(session.overlays.focus, FocusTarget::Transcript));
        session.toggle_main_focus();
        assert!(matches!(session.overlays.focus, FocusTarget::Composer));
        assert!(!session.mouse_capture_enabled);
    }
}

#[derive(Debug, Clone)]
pub struct PendingRequest {
    pub label: String,
    pub started_at: Instant,
    pub current_stage: String,
    pub current_stage_path: String,
    pub spinner_index: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ActivityState {
    pub label: String,
    pub current_stage: String,
    pub current_stage_path: String,
    pub details: Vec<String>,
    pub spinner_index: usize,
    pub started_at: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub title: String,
    pub blocks: Vec<MessageBlock>,
    pub notes: Vec<String>,
    pub latency_ms: u128,
}

#[derive(Debug, Clone)]
pub struct ChatSession {
    pub context: ChatContext,
    pub settings: ChatSettings,
    pub transcript: Transcript,
    pub trace_log: Vec<ChatTraceEntry>,
    pub composer: ComposerState,
    pub overlays: OverlayState,
    pub pending: Option<PendingRequest>,
    pub activity: ActivityState,
    pub last_trace_run_id: Option<String>,
    pub last_debug_claim_ids: Vec<String>,
    pub mouse_capture_enabled: bool,
    pub should_quit: bool,
    pub runtime: RuntimeConfig,
    pub viewport: TranscriptViewport,
    pub selected_turn: Option<usize>,
    pub expanded_nodes: BTreeMap<(usize, String), bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub context: ChatContext,
    pub settings: ChatSettings,
    pub transcript: Transcript,
}

impl ChatSession {
    pub fn new(options: ChatOptions) -> Self {
        let verbosity = match options.trace_mode {
            TraceMode::Off => Verbosity::Compact,
            TraceMode::Brief => Verbosity::Detailed,
            TraceMode::Full => Verbosity::FullStack,
        };
        Self {
            context: ChatContext {
                source_lang: options.from,
                target_lang: options.to,
                mode: EngineMode::Conversation,
                chat_language_override: None,
            },
            settings: ChatSettings { verbosity },
            transcript: Transcript::new(),
            trace_log: Vec::new(),
            composer: ComposerState::default(),
            overlays: OverlayState::default(),
            pending: None,
            activity: ActivityState::default(),
            mouse_capture_enabled: false,
            should_quit: false,
            runtime: RuntimeConfig {
                data_dir: options.data_dir,
                trace_mode: options.trace_mode,
                offline: options.offline,
                source_policy: options.source_policy,
                engine_session_id: "chat-session".into(),
            },
            last_trace_run_id: None,
            last_debug_claim_ids: Vec::new(),
            viewport: TranscriptViewport::new(),
            selected_turn: None,
            expanded_nodes: BTreeMap::new(),
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
        self.selected_turn = Some(turn);
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
        self.selected_turn = self.transcript.messages.last().map(|message| message.turn);
        self.viewport.jump_to_bottom();
    }

    pub fn push_error_message(&mut self, text: impl Into<String>) {
        self.transcript.push(
            MessageRole::Error,
            "error",
            vec![MessageBlock::Paragraph(text.into())],
            MessageMeta::default(),
        );
        self.selected_turn = self.transcript.messages.last().map(|message| message.turn);
        self.viewport.jump_to_bottom();
    }

    pub fn remember_debug_claim_ids(&mut self, claim_ids: Vec<String>) {
        self.last_debug_claim_ids = claim_ids;
    }

    pub fn claim_id_for_debug_reference(&self, reference: &str) -> Option<String> {
        let trimmed = reference.trim();
        if let Ok(index) = trimmed.parse::<usize>() {
            if index == 0 {
                return None;
            }
            return self.last_debug_claim_ids.get(index - 1).cloned();
        }
        (!trimmed.is_empty()).then(|| trimmed.to_string())
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
        self.selected_turn = self.transcript.messages.last().map(|message| message.turn);
        self.viewport.jump_to_bottom();
    }

    pub fn start_stream(&mut self, label: &str) -> usize {
        let started_at = Instant::now();
        self.activity = ActivityState {
            label: label.to_string(),
            current_stage: "queued".into(),
            current_stage_path: "queued".into(),
            details: Vec::new(),
            spinner_index: 0,
            started_at: Some(started_at),
        };
        self.pending = Some(PendingRequest {
            label: label.into(),
            started_at,
            current_stage: "queued".into(),
            current_stage_path: "queued".into(),
            spinner_index: 0,
        });
        0
    }

    pub fn append_stream_progress(&mut self, event: &crate::chat::service::ChatProgressEvent, full: bool) {
        let Some(pending) = self.pending.as_mut() else { return; };
        pending.current_stage = event.label.clone();
        pending.current_stage_path = event.stage.clone();
        pending.spinner_index = pending.spinner_index.wrapping_add(1);
        self.activity.label = pending.label.clone();
        self.activity.current_stage = event.label.clone();
        self.activity.current_stage_path = event.stage.clone();
        self.activity.spinner_index = pending.spinner_index;
        self.activity.details = if full { event.details.clone() } else { Vec::new() };
        self.activity.started_at = Some(pending.started_at);
        if self.viewport.stick_to_bottom { self.viewport.jump_to_bottom(); }
    }

    pub fn tick_pending(&mut self) {
        if let Some(pending) = self.pending.as_mut() {
            pending.spinner_index = pending.spinner_index.wrapping_add(1);
            self.activity.spinner_index = pending.spinner_index;
        }
    }

    pub fn finish_stream(&mut self, title: String, blocks: Vec<MessageBlock>, meta: MessageMeta, include_trace: bool) {
        self.finish_stream_as(MessageRole::Assistant, title, blocks, meta, include_trace);
    }

    pub fn finish_stream_as(&mut self, role: MessageRole, title: String, blocks: Vec<MessageBlock>, meta: MessageMeta, include_trace: bool) {
        let was_stick_to_bottom = self.viewport.stick_to_bottom;
        let Some(pending) = self.pending.take() else {
            self.transcript.push(role, title, blocks, meta);
            self.viewport.jump_to_bottom();
            return;
        };
        self.activity = ActivityState::default();
        let mut blocks = blocks;
        if include_trace {
            blocks.push(MessageBlock::Trace(vec![format!(
                "trace: {} -> {}",
                pending.current_stage_path, pending.current_stage
            )]));
        }
        self.transcript.push(role, title, blocks, meta);
        self.selected_turn = self.transcript.messages.last().map(|message| message.turn);
        if was_stick_to_bottom { self.viewport.jump_to_bottom(); }
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
        self.trace_log.clear();
        self.composer = ComposerState::default();
        self.pending = None;
        self.activity = ActivityState::default();
        self.last_trace_run_id = None;
        self.viewport = TranscriptViewport::new();
        self.selected_turn = None;
        self.expanded_nodes.clear();
    }

    pub fn toggle_main_focus(&mut self) {
        self.overlays.focus = match self.overlays.focus {
            FocusTarget::Composer | FocusTarget::CommandPopup => FocusTarget::Transcript,
            FocusTarget::Transcript => FocusTarget::Composer,
        };
    }

    pub fn set_translation_direction(&mut self, from: String, to: String) {
        self.context.source_lang = from;
        self.context.target_lang = to;
    }

    pub fn set_chat_language_override(&mut self, language: Option<String>) {
        self.context.chat_language_override = language;
    }

    pub fn chat_language_for(&self, text: &str) -> String {
        let _ = text;
        self.context.chat_language_override.clone().unwrap_or_else(|| "auto".into())
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

    pub fn set_mode(&mut self, mode: EngineMode) {
        self.context.mode = mode;
    }

    pub fn cycle_verbosity(&mut self) -> Verbosity {
        self.settings.verbosity = self.settings.verbosity.next();
        self.settings.verbosity
    }

    pub fn status_line(&self) -> String {
        match &self.pending {
            Some(pending) => format!(
                "{} {} | {} | {} ms",
                spinner(pending.spinner_index),
                pending.label,
                if pending.current_stage.is_empty() { "queued" } else { pending.current_stage.as_str() },
                pending.started_at.elapsed().as_millis()
            ),
            None => format!(
                "focus:{} | mouse:{} | {} | chat:{} | translate:{} -> {} | {}",
                match self.overlays.focus {
                    FocusTarget::Composer => "composer",
                    FocusTarget::Transcript => "transcript",
                    FocusTarget::CommandPopup => "commands",
                },
                if self.mouse_capture_enabled { "application" } else { "native" },
                self.context.mode.as_str(),
                self.chat_language_label(),
                self.context.source_lang,
                self.context.target_lang,
                self.settings.verbosity.as_str(),
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

    pub fn select_previous_message(&mut self) {
        if self.transcript.messages.is_empty() {
            self.selected_turn = None;
            return;
        }
        let turns = self.transcript.messages.iter().map(|message| message.turn).collect::<Vec<_>>();
        let current = self.selected_turn.unwrap_or_else(|| *turns.last().unwrap_or(&1));
        let next = turns
            .iter()
            .rev()
            .find(|turn| **turn < current)
            .copied()
            .or_else(|| turns.first().copied());
        self.selected_turn = next;
        self.viewport.stick_to_bottom = false;
    }

    pub fn select_next_message(&mut self) {
        if self.transcript.messages.is_empty() {
            self.selected_turn = None;
            return;
        }
        let turns = self.transcript.messages.iter().map(|message| message.turn).collect::<Vec<_>>();
        let current = self.selected_turn.unwrap_or_else(|| turns[0]);
        let next = turns
            .iter()
            .find(|turn| **turn > current)
            .copied()
            .or_else(|| turns.last().copied());
        self.selected_turn = next;
    }

    pub fn collapse_selected(&mut self, collapsed: bool) -> bool {
        let Some(turn) = self.selected_turn else { return false; };
        self.transcript.set_collapsed(turn, collapsed)
    }

    pub fn toggle_selected_node(&mut self, node_key: &str) -> bool {
        let Some(turn) = self.selected_turn else { return false; };
        let key = (turn, node_key.to_string());
        let default = self.node_default_expanded(turn, node_key);
        let current = self.expanded_nodes.get(&key).copied().unwrap_or(default);
        self.expanded_nodes.insert(key, !current);
        true
    }

    pub fn node_expanded(&self, turn: usize, node_key: &str, default: bool) -> bool {
        self.expanded_nodes
            .get(&(turn, node_key.to_string()))
            .copied()
            .unwrap_or(default)
    }

    pub fn toggle_first_collapsed_node(&mut self) -> bool {
        let Some(turn) = self.selected_turn else { return false; };
        let Some(key) = self.find_node(turn, |expanded| !expanded) else { return false; };
        self.toggle_selected_node(&key)
    }

    pub fn collapse_first_expanded_node(&mut self) -> bool {
        let Some(turn) = self.selected_turn else { return false; };
        let Some(key) = self.find_node(turn, |expanded| expanded) else { return false; };
        self.toggle_selected_node(&key)
    }

    fn node_default_expanded(&self, turn: usize, node_key: &str) -> bool {
        self.transcript
            .messages
            .iter()
            .find(|message| message.turn == turn)
            .and_then(|message| find_node(&message.blocks, node_key))
            .map(|node| node.expanded)
            .unwrap_or(false)
    }

    fn find_node<F>(&self, turn: usize, predicate: F) -> Option<String>
    where
        F: Fn(bool) -> bool,
    {
        let message = self.transcript.messages.iter().find(|message| message.turn == turn)?;
        find_node_matching(&message.blocks, turn, &self.expanded_nodes, &predicate)
    }

    pub fn export_transcript_text(&self) -> String {
        self.transcript.export_plain_text()
    }
}

fn spinner(index: usize) -> &'static str {
    ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"][index % 10]
}

fn find_node<'a>(blocks: &'a [MessageBlock], key: &str) -> Option<&'a TranscriptNode> {
    for block in blocks {
        if let MessageBlock::Tree(nodes) = block {
            if let Some(node) = find_node_in_nodes(nodes, key) {
                return Some(node);
            }
        }
    }
    None
}

fn find_node_in_nodes<'a>(nodes: &'a [TranscriptNode], key: &str) -> Option<&'a TranscriptNode> {
    for node in nodes {
        if node.key == key {
            return Some(node);
        }
        if let Some(found) = find_node_in_nodes(&node.children, key) {
            return Some(found);
        }
    }
    None
}

fn find_node_matching<F>(
    blocks: &[MessageBlock],
    turn: usize,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
    predicate: &F,
) -> Option<String>
where
    F: Fn(bool) -> bool,
{
    for block in blocks {
        if let MessageBlock::Tree(nodes) = block {
            if let Some(found) = find_node_matching_in_nodes(nodes, turn, expanded_nodes, predicate) {
                return Some(found);
            }
        }
    }
    None
}

fn find_node_matching_in_nodes<F>(
    nodes: &[TranscriptNode],
    turn: usize,
    expanded_nodes: &BTreeMap<(usize, String), bool>,
    predicate: &F,
) -> Option<String>
where
    F: Fn(bool) -> bool,
{
    for node in nodes {
        let expanded = expanded_nodes
            .get(&(turn, node.key.clone()))
            .copied()
            .unwrap_or(node.expanded);
        if predicate(expanded) {
            return Some(node.key.clone());
        }
        if let Some(found) = find_node_matching_in_nodes(&node.children, turn, expanded_nodes, predicate) {
            return Some(found);
        }
    }
    None
}
