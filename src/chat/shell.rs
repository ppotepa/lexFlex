use crate::chat::commands::{parse_input, visible_suggestions, InputAction, SlashCommand};
use crate::chat::navigation;
use crate::chat::service::{ChatJob, ChatJobOutput, ChatJobResult, ChatProgressEvent, ChatWorker, ChatWorkerEvent};
use crate::chat::session::{EngineMode, ChatSession, FocusTarget, OverlayKind, TranscriptSelection};
use crate::chat::tui::input::{
    backspace, delete_forward, insert_char, insert_newline, insert_text, move_down, move_left,
    move_right, move_up, recall_next_input, recall_previous_input,
};
use crate::chat::tui::render::{command_popup_rect, draw, layout_for, ChatLayout};
use crate::chat::widgets::chat_window::{max_scroll_for_height, selection_at_row_offset, turn_at_row_offset};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::error::Error;
use std::io::Stdout;
use std::time::Duration;

pub fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    session: &mut ChatSession,
    service: crate::chat::service::ChatService,
) -> Result<(), Box<dyn Error>> {
    if session.mouse_mode.is_application() {
        execute!(terminal.backend_mut(), EnableMouseCapture)?;
    }
    let worker = ChatWorker::spawn(service);
    let tick_rate = Duration::from_millis(50);
    loop {
        for _ in 0..4 {
            let Some(event) = worker.try_receive() else { break; };
            match event {
                ChatWorkerEvent::Progress(progress) => apply_progress(session, progress),
                ChatWorkerEvent::Result(result) => {
                    apply_job_result(session, result);
                    break;
                }
            }
        }
        session.tick_pending();
        terminal.draw(|frame| draw(frame, session))?;
        if session.should_quit {
            break;
        }
        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key) => {
                    if handle_key_event(session, terminal, &worker, key)? {
                        break;
                    }
                }
                Event::Resize(_, _) => {}
                Event::Mouse(mouse) => {
                    let layout = current_layout(terminal)?;
                    handle_mouse_event(session, mouse, layout);
                }
                Event::Paste(text) => {
                    if matches!(session.overlays.focus, FocusTarget::Composer) {
                        insert_text(&mut session.composer, &text);
                        sync_command_popup(session);
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn handle_key_event(
    session: &mut ChatSession,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    worker: &ChatWorker,
    key: KeyEvent,
) -> Result<bool, Box<dyn Error>> {
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return Ok(false);
    }
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && !key.modifiers.contains(KeyModifiers::SHIFT)
        && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
    {
        return Ok(true);
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('l')) {
        session.clear_session();
        session.push_system_message("screen cleared.");
        return Ok(false);
    }
    if key.modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT)
        && matches!(key.code, KeyCode::Char('b') | KeyCode::Char('B'))
    {
        match copy_transcript_to_clipboard(session) {
            Ok(()) => session.push_system_message("chat transcript copied to clipboard."),
            Err(error) => session.push_error_message(format!("copy failed: {error}")),
        }
        return Ok(false);
    }
    if key.modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT)
        && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
    {
        match copy_selected_to_clipboard(session) {
            Ok(()) => session.push_system_message("selected chat entry copied to clipboard."),
            Err(error) => session.push_error_message(format!("copy failed: {error}")),
        }
        return Ok(false);
    }
    if key.modifiers.contains(KeyModifiers::ALT) && matches!(key.code, KeyCode::Char('v')) {
        let verbosity = session.cycle_verbosity();
        session.push_system_message(format!("verbosity set to {}.", verbosity.as_str()));
        return Ok(false);
    }
    if key.modifiers.contains(KeyModifiers::ALT) && matches!(key.code, KeyCode::Char('m')) {
        session.mouse_mode = session.mouse_mode.toggle();
        if session.mouse_mode.is_application() {
            execute!(terminal.backend_mut(), EnableMouseCapture)?;
            session.push_system_message("mouse mode set to application.");
        } else {
            execute!(terminal.backend_mut(), DisableMouseCapture)?;
            session.push_system_message("mouse mode set to native selection.");
        }
        return Ok(false);
    }
    if matches!(key.code, KeyCode::Tab) {
        if matches!(session.overlays.focus, FocusTarget::CommandPopup) {
            close_overlay(session);
        }
        session.toggle_main_focus();
        return Ok(false);
    }
    if matches!(key.code, KeyCode::F(6)) {
        session.overlays.focus = match session.overlays.focus {
            FocusTarget::Composer => FocusTarget::Transcript,
            FocusTarget::Transcript => FocusTarget::Composer,
            FocusTarget::CommandPopup => FocusTarget::Composer,
        };
        return Ok(false);
    }
    if key.modifiers.contains(KeyModifiers::ALT) && matches!(key.code, KeyCode::Up) {
        session.select_previous_message();
        return Ok(false);
    }
    if key.modifiers.contains(KeyModifiers::ALT) && matches!(key.code, KeyCode::Down) {
        session.select_next_message();
        return Ok(false);
    }
    match session.overlays.focus {
        FocusTarget::CommandPopup => handle_command_popup_key(session, key),
        FocusTarget::Transcript => handle_transcript_key(session, key),
        FocusTarget::Composer => handle_composer_key(session, worker, key)?,
    }
    Ok(false)
}

fn handle_transcript_key(session: &mut ChatSession, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::F(6) => session.overlays.focus = FocusTarget::Composer,
        KeyCode::Up => session.select_previous_message(),
        KeyCode::Down => session.select_next_message(),
        KeyCode::Left => {
            if let Some(selection) = session.selected_selection().cloned() {
                match selection {
                    TranscriptSelection::Node { turn, key } => {
                        session.select_node(turn, key.clone());
                        let _ = session.toggle_selected_node(&key);
                    }
                    TranscriptSelection::Message { turn } => {
                        let _ = session.transcript.set_collapsed(turn, true);
                    }
                }
            }
        }
        KeyCode::Right => {
            if let Some(selection) = session.selected_selection().cloned() {
                match selection {
                    TranscriptSelection::Node { turn, key } => {
                        session.select_node(turn, key.clone());
                        let _ = session.toggle_selected_node(&key);
                    }
                    TranscriptSelection::Message { turn } => {
                        let _ = session.transcript.set_collapsed(turn, false);
                    }
                }
            }
        }
        KeyCode::Char('+') | KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(selection) = session.selected_selection().cloned() {
                match selection {
                    TranscriptSelection::Node { turn, key } => {
                        session.select_node(turn, key.clone());
                        let _ = session.toggle_selected_node(&key);
                    }
                    TranscriptSelection::Message { turn } => {
                        if !session.toggle_first_collapsed_node() {
                            if let Some(message) = session.transcript.messages.iter().find(|message| message.turn == turn) {
                                let _ = session.transcript.set_collapsed(turn, !message.collapsed);
                            }
                        }
                    }
                }
            } else if !session.toggle_first_collapsed_node() {
                if let Some(turn) = session.selected_turn {
                    if let Some(message) = session.transcript.messages.iter().find(|message| message.turn == turn) {
                        let _ = session.collapse_selected(!message.collapsed);
                    }
                }
            }
        }
        KeyCode::Char('-') => {
            if let Some(selection) = session.selected_selection().cloned() {
                match selection {
                    TranscriptSelection::Node { turn, key } => {
                        session.select_node(turn, key.clone());
                        let _ = session.toggle_selected_node(&key);
                    }
                    TranscriptSelection::Message { turn } => {
                        if !session.collapse_first_expanded_node() {
                            let _ = session.transcript.set_collapsed(turn, true);
                        }
                    }
                }
            } else if !session.collapse_first_expanded_node() {
                let _ = session.collapse_selected(true);
            }
        }
        _ => {}
    }
}

fn handle_command_popup_key(session: &mut ChatSession, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => close_overlay(session),
        KeyCode::Up => {
            if session.overlays.command_popup.selected_index > 0 {
                session.overlays.command_popup.selected_index -= 1;
            } else {
                session.overlays.focus = FocusTarget::Composer;
            }
        }
        KeyCode::Down => {
            let len = session.overlays.command_popup.suggestions.len();
            if session.overlays.command_popup.selected_index + 1 < len {
                session.overlays.command_popup.selected_index += 1;
            }
        }
        KeyCode::Enter => {
            if let Some(item) = session
                .overlays
                .command_popup
                .suggestions
                .get(session.overlays.command_popup.selected_index)
            {
                session.composer.input = item.replacement.clone();
                session.composer.cursor = session.composer.input.len();
                close_overlay(session);
                sync_command_popup(session);
            }
        }
        _ => {
            session.overlays.focus = FocusTarget::Composer;
        }
    }
}

fn handle_composer_key(
    session: &mut ChatSession,
    worker: &ChatWorker,
    key: KeyEvent,
) -> Result<(), Box<dyn Error>> {
    match key.code {
        KeyCode::Esc => {
            if session.overlays.active.is_some() {
                close_overlay(session);
            } else {
                session.composer.input.clear();
                session.composer.cursor = 0;
                session.composer.history_cursor = None;
                session.composer.history_draft = None;
            }
        }
        KeyCode::Char(ch) => {
            if !key.modifiers.contains(KeyModifiers::CONTROL)
                && !key.modifiers.contains(KeyModifiers::ALT)
            {
                insert_char(&mut session.composer, ch);
                sync_command_popup(session);
            }
        }
        KeyCode::Backspace => {
            backspace(&mut session.composer);
            sync_command_popup(session);
        }
        KeyCode::Delete => {
            delete_forward(&mut session.composer);
            sync_command_popup(session);
        }
        KeyCode::Left => move_left(&mut session.composer),
        KeyCode::Right => move_right(&mut session.composer),
        KeyCode::Home => session.composer.cursor = 0,
        KeyCode::End => {
            if session.composer.input.is_empty() {
                navigation::jump_to_bottom(session);
            } else {
                session.composer.cursor = session.composer.input.len();
            }
        }
        KeyCode::Up => {
            if session.composer.input.contains('\n') {
                move_up(&mut session.composer);
            } else {
                recall_previous_input(&mut session.composer);
            }
        }
        KeyCode::Down => {
            if matches!(session.overlays.active, Some(OverlayKind::CommandPalette))
                && !session.overlays.command_popup.suggestions.is_empty()
            {
                session.overlays.focus = FocusTarget::CommandPopup;
            } else if session.composer.input.contains('\n') {
                move_down(&mut session.composer);
            } else {
                recall_next_input(&mut session.composer);
            }
        }
        KeyCode::PageUp | KeyCode::PageDown => {}
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
            insert_newline(&mut session.composer);
            sync_command_popup(session);
        }
        KeyCode::Enter => submit_input(session, worker)?,
        _ => {}
    }
    Ok(())
}

fn handle_mouse_event(
    session: &mut ChatSession,
    mouse: MouseEvent,
    layout: ChatLayout,
) {
    let chat_area = layout.chat;
    let max_scroll = max_scroll_for_height(&session.transcript.messages, chat_area, &session.expanded_nodes);
    let popup_area = if matches!(session.overlays.active, Some(OverlayKind::CommandPalette)) {
        Some(command_popup_rect(
            layout.chat,
            layout.footer,
            session.overlays.command_popup.suggestions.len() as u16,
        ))
    } else {
        None
    };
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) if popup_area.is_some_and(|area| in_rect(mouse.column, mouse.row, area)) => {
            session.overlays.focus = FocusTarget::CommandPopup;
            if let Some(area) = popup_area {
                let row_offset = mouse.row.saturating_sub(area.y.saturating_add(1));
                let index = row_offset as usize;
                if index < session.overlays.command_popup.suggestions.len() {
                    session.overlays.command_popup.selected_index = index;
                }
                if mouse.column >= area.x && mouse.column < area.x.saturating_add(area.width) {
                    if let Some(item) = session
                        .overlays
                        .command_popup
                        .suggestions
                        .get(session.overlays.command_popup.selected_index)
                    {
                        session.composer.input = item.replacement.clone();
                        session.composer.cursor = session.composer.input.len();
                        close_overlay(session);
                        sync_command_popup(session);
                    }
                }
            }
        }
        MouseEventKind::ScrollUp if in_rect(mouse.column, mouse.row, layout.chat) => {
            session.overlays.focus = FocusTarget::Transcript;
            navigation::page_up(session, 3, max_scroll)
        }
        MouseEventKind::ScrollDown if in_rect(mouse.column, mouse.row, layout.chat) => {
            session.overlays.focus = FocusTarget::Transcript;
            navigation::page_down(session, 3, max_scroll)
        }
        MouseEventKind::Down(MouseButton::Left) => {
            if in_rect(mouse.column, mouse.row, layout.footer) {
                session.overlays.focus = FocusTarget::Composer;
                if session.overlays.active.is_some() {
                    close_overlay(session);
                }
                return;
            }
            if !in_rect(mouse.column, mouse.row, layout.chat) {
                return;
            }
            let row_offset = mouse.row.saturating_sub(chat_area.y.saturating_add(1));
            let scroll = if session.viewport.stick_to_bottom {
                max_scroll
            } else {
                session.viewport.scroll_offset.min(max_scroll)
            };
            if let Some(target) = selection_at_row_offset(
                &session.transcript.messages,
                chat_area,
                scroll,
                row_offset,
                &session.expanded_nodes,
            ) {
                match target {
                    TranscriptSelection::Message { turn } => {
                        session.select_message(turn);
                    }
                    TranscriptSelection::Node { turn, key } => {
                        session.select_node(turn, key.clone());
                        let _ = session.toggle_selected_node(&key);
                    }
                }
                session.overlays.focus = FocusTarget::Transcript;
            } else if let Some(turn) = turn_at_row_offset(
                &session.transcript.messages,
                chat_area,
                scroll,
                row_offset,
                &session.expanded_nodes,
            ) {
                session.select_message(turn);
                session.overlays.focus = FocusTarget::Transcript;
            }
        }
        MouseEventKind::Down(MouseButton::Right) => {
            if matches!(session.overlays.focus, FocusTarget::Transcript) {
                if let Some(turn) = session.selected_turn {
                    if let Some(message) = session.transcript.messages.iter().find(|message| message.turn == turn) {
                        let _ = session.collapse_selected(!message.collapsed);
                    }
                }
            }
        }
        _ => {}
    }
}

fn in_rect(column: u16, row: u16, area: ratatui::layout::Rect) -> bool {
    column >= area.x
        && column < area.x.saturating_add(area.width)
        && row >= area.y
        && row < area.y.saturating_add(area.height)
}

fn submit_input(session: &mut ChatSession, worker: &ChatWorker) -> Result<(), Box<dyn Error>> {
    if session.pending.is_some() {
        return Ok(());
    }
    let input = session.composer.input.trim().to_string();
    if input.is_empty() {
        session.composer.input.clear();
        session.composer.cursor = 0;
        return Ok(());
    }

    session.composer.history.push(input.clone());
    session.composer.history_cursor = None;
    session.composer.history_draft = None;
    session.composer.input.clear();
    session.composer.cursor = 0;
    close_overlay(session);

    match parse_input(&input) {
        InputAction::Empty => {}
        InputAction::UserMessage(text) => {
            let turn = session.push_user_message(text.clone());
            match session.context.mode {
                EngineMode::Conversation => {
                    handle_chat_message(session, worker, text, turn);
                }
                EngineMode::Translation => {
                    submit_translate(session, worker, text);
                }
            }
        }
        InputAction::Slash(command) => match command {
            SlashCommand::Chat => {
                session.set_mode(EngineMode::Conversation);
                session.push_system_message(
                    "chat mode enabled. Ask questions or continue the conversation.",
                );
            }
            SlashCommand::Lang(language) => {
                session.set_chat_language_override(language.clone());
                let label = language.unwrap_or_else(|| "auto".to_string());
                session.push_system_message(format!("chat language set to {label}."));
            }
            SlashCommand::Translate { from, to } => {
                session.set_translation_direction(from.clone(), to.clone());
                session.set_mode(EngineMode::Translation);
                submit_engine_request(session, worker, "translation configure", crate::engine::EngineRequest::Translation(
                    crate::engine::TranslationRequest::Configure {
                        from: if from == "auto" { crate::engine::LanguageMode::Auto } else { crate::engine::LanguageMode::Explicit(from) },
                        to: crate::core::interlingua::LanguageId::new(&to),
                    }
                ));
            }
            SlashCommand::AnswerLanguage(language) => {
                let language = match language.as_str() {
                    "auto" => crate::engine::AnswerLanguage::Auto,
                    "source" => crate::engine::AnswerLanguage::Source,
                    value => crate::engine::AnswerLanguage::Explicit(crate::core::interlingua::LanguageId::new(value)),
                };
                submit_engine_request(session, worker, "answer language", crate::engine::EngineRequest::Conversation(
                    crate::engine::ConversationRequest::SetAnswerLanguage { language }
                ));
            }
            SlashCommand::Settings => {
                let verbosity = session.cycle_verbosity();
                session.push_system_message(format!("verbosity set to {}.", verbosity.as_str()));
            }
            SlashCommand::Ingest(text) => {
                session.push_user_message(input.clone());
                submit_ingest(session, worker, text);
            }
            SlashCommand::Query(text) => {
                match serde_json::from_str::<crate::query::QueryInterlingua>(&text) {
                    Ok(query) => { submit_query(session, worker, query); }
                    Err(err) => session.push_error_message(format!("/query expects QueryInterlingua JSON: {err}")),
                }
            }
            SlashCommand::Inspect(target) => {
                let target = match target.as_str() { "sources" => crate::engine::InspectTarget::Sources, "bundles" => crate::engine::InspectTarget::Bundles, _ => crate::engine::InspectTarget::Session };
                submit_inspect(session, worker, target);
            }
            SlashCommand::Facts(command) => {
                session.push_user_message(input.clone());
                submit_engine_request(
                    session,
                    worker,
                    "engine debug",
                    crate::engine::EngineRequest::Debug {
                        command: crate::engine::DebugCommand::Facts(crate::engine::FactsDebugQuery {
                            selector: command.selector,
                            role: command.role,
                            limit: command.limit,
                            page: command.page,
                        }),
                    },
                );
            }
            SlashCommand::Evidence(reference) => {
                let Some(claim_id) = session.claim_id_for_debug_reference(&reference) else {
                    session.push_error_message("unknown evidence reference; use /evidence <claim-id|fact-number> after /facts.");
                    return Ok(());
                };
                session.push_user_message(input.clone());
                submit_engine_request(
                    session,
                    worker,
                    "engine evidence",
                    crate::engine::EngineRequest::Debug {
                        command: crate::engine::DebugCommand::Evidence(crate::engine::EvidenceDebugQuery {
                            claim_id,
                        }),
                    },
                );
            }
            SlashCommand::Debug(target) => {
                session.push_user_message(input.clone());
                submit_engine_request(
                    session,
                    worker,
                    "engine debug",
                    crate::engine::EngineRequest::Debug {
                        command: crate::engine::DebugCommand::Inspect(match target.as_str() {
                            "interlingua" => crate::engine::DebugInspectTarget::Interlingua,
                            "entities" => crate::engine::DebugInspectTarget::Entities,
                            "query" => crate::engine::DebugInspectTarget::Query,
                            "pipeline" => crate::engine::DebugInspectTarget::Pipeline,
                            "sources" => crate::engine::DebugInspectTarget::Sources,
                            "snapshot" => crate::engine::DebugInspectTarget::Snapshot,
                            "trace" => crate::engine::DebugInspectTarget::Trace { run_id: None },
                            _ => unreachable!(),
                        }),
                    },
                );
            }
            SlashCommand::Trace => submit_engine_request(
                session,
                worker,
                "engine trace",
                crate::engine::EngineRequest::Debug {
                    command: crate::engine::DebugCommand::Inspect(
                        crate::engine::DebugInspectTarget::Trace { run_id: None },
                    ),
                },
            ),
            SlashCommand::Clear(target) => {
                let target = match target.as_deref() {
                    Some("conversation") => crate::engine::ClearTarget::Conversation,
                    Some("translation") => crate::engine::ClearTarget::Translation,
                    Some("all") => crate::engine::ClearTarget::All,
                    None if matches!(session.context.mode, EngineMode::Translation) => crate::engine::ClearTarget::Translation,
                    None => crate::engine::ClearTarget::Conversation,
                    _ => unreachable!(),
                };
                submit_clear(session, worker, target)
            }
            SlashCommand::Quit => session.should_quit = true,
        },
        InputAction::IncompleteSlash(draft) => {
            if draft.command == "translate" {
                session.push_error_message(
                    "invalid translate command; use /translate en pl or /translate pl en.",
                );
            } else if draft.command == "lang" {
                session.push_error_message("invalid language command; use /lang en, /lang pl, or /lang auto.");
            } else if draft.command == "facts" {
                session.push_error_message("invalid facts command; use /facts <entity> [--role any|subject|object] [--limit N|--all] [--page N].");
            } else if draft.command == "evidence" {
                session.push_error_message("invalid evidence command; use /evidence <claim-id|fact-number>.");
            } else if draft.command == "debug" {
                session.push_error_message("invalid debug target; use /debug interlingua|entities|query|pipeline|sources|snapshot|trace.");
            } else if draft.command == "answer-lang" {
                session.push_error_message("invalid answer language; use /answer-lang auto|source|pl|en.");
            } else if draft.command == "clear" {
                session.push_error_message("invalid clear target; use /clear [conversation|translation|all].");
            } else {
                session.push_error_message(format!("unknown or incomplete command: /{}", draft.command));
            }
        }
    }
    Ok(())
}

fn handle_chat_message(
    session: &mut ChatSession,
    worker: &ChatWorker,
    text: String,
    _turn: usize,
) {
    submit_user_turn(session, worker, text);
}

fn apply_job_result(session: &mut ChatSession, result: ChatJobResult) {
    let latency_ms = session
        .pending
        .as_ref()
        .map(|pending| pending.started_at.elapsed().as_millis())
        .unwrap_or(0);
    match result.result {
        Ok(ChatJobOutput::Engine(response)) => apply_engine_response(session, response, latency_ms),
        Err(error) => {
            session.pending = None;
            session.push_error_message(error);
        }
    }
}

fn apply_progress(session: &mut ChatSession, event: ChatProgressEvent) {
    let full = matches!(session.settings.verbosity, crate::chat::Verbosity::FullStack)
        || matches!(session.runtime.trace_mode, crate::chat::TraceMode::Full);
    session.append_stream_progress(&event, full);
}

fn apply_engine_response(session: &mut ChatSession, response: crate::engine::EngineResponse, latency_ms: u128) {
    use crate::engine::EngineResponse;
    let (title, presentation, mut notes, meta, role) = match response {
        EngineResponse::Conversation(value) => {
            let notes = value.answer.as_ref().map(|answer| answer.evidence.clone()).unwrap_or_default();
            ("conversation", value.presentation, notes, value.meta, crate::chat::transcript::MessageRole::Assistant)
        }
        EngineResponse::Translation(value) => {
            ("translation", value.presentation, Vec::new(), value.meta, crate::chat::transcript::MessageRole::Assistant)
        }
        EngineResponse::Ingest(value) => {
            ("ingest", value.presentation, Vec::new(), value.meta, crate::chat::transcript::MessageRole::Assistant)
        }
        EngineResponse::Answer { meta, answer, presentation } => {
            ("answer", presentation, answer.evidence, meta, crate::chat::transcript::MessageRole::Assistant)
        }
        EngineResponse::Inspection(value) => {
            ("inspection", value.presentation, vec![], value.meta, crate::chat::transcript::MessageRole::Assistant)
        }
        EngineResponse::Debug(value) => {
            session.remember_debug_claim_ids(value.facts.iter().map(|fact| fact.claim_id.clone()).collect());
            ("engine debug", value.ui, Vec::new(), value.meta, debug_role_for(&value.presentation.category))
        }
        EngineResponse::Error { meta, error: _, presentation } => {
            ("engine error", presentation, vec![], meta, crate::chat::transcript::MessageRole::Error)
        }
    };
    session.last_trace_run_id = Some(meta.run_id.clone());
    notes.extend(response_meta_notes(session, &meta));
    let mut blocks = presentation_blocks(&presentation, session.settings.verbosity);
    if !notes.is_empty() {
        blocks.push(crate::chat::transcript::MessageBlock::BulletList(notes));
    }
    let include_trace = matches!(session.settings.verbosity, crate::chat::Verbosity::FullStack)
        || matches!(session.runtime.trace_mode, crate::chat::TraceMode::Full);
    session.finish_stream_as(
        role,
        title.into(),
        blocks,
        crate::chat::transcript::MessageMeta { latency_ms: Some(latency_ms), command: None },
        include_trace,
    );
}

fn presentation_blocks(
    presentation: &crate::engine::EnginePresentation,
    verbosity: crate::chat::Verbosity,
) -> Vec<crate::chat::transcript::MessageBlock> {
    use crate::chat::transcript::{MessageBlock, TranscriptNode};
    let mut nodes = Vec::new();
    nodes.push(TranscriptNode {
        key: "summary".into(),
        label: presentation.title.clone(),
        summary: Some(presentation.summary.clone()),
        tone: Some(format!("{:?}", presentation.tone)),
        expanded: true,
        blocks: vec![
            MessageBlock::Paragraph(presentation.summary.clone()),
            MessageBlock::KeyValue { key: "status".into(), value: presentation.status_line.clone() },
        ],
        children: Vec::new(),
    });
    for (index, section) in presentation.sections.iter().enumerate() {
        let mut children = Vec::new();
        for (block_index, block) in section.blocks.iter().enumerate() {
            match block {
                crate::engine::EnginePresentationBlock::Text(value) => children.push(TranscriptNode {
                    key: format!("section.{index}.text.{block_index}"),
                    label: "Detail".into(),
                    summary: Some(value.clone()),
                    tone: Some(format!("{:?}", section.tone)),
                    expanded: false,
                    blocks: vec![MessageBlock::Paragraph(value.clone())],
                    children: Vec::new(),
                }),
                crate::engine::EnginePresentationBlock::Fields(values) => children.push(TranscriptNode {
                    key: format!("section.{index}.fields.{block_index}"),
                    label: "Fields".into(),
                    summary: Some(format!("{} field(s)", values.len())),
                    tone: Some(format!("{:?}", section.tone)),
                    expanded: false,
                    blocks: values
                        .iter()
                        .map(|(key, value)| MessageBlock::KeyValue { key: key.clone(), value: value.clone() })
                        .collect(),
                    children: Vec::new(),
                }),
                crate::engine::EnginePresentationBlock::BulletList(values) if !values.is_empty() => children.push(TranscriptNode {
                    key: format!("section.{index}.bullets.{block_index}"),
                    label: "Items".into(),
                    summary: Some(format!("{} item(s)", values.len())),
                    tone: Some(format!("{:?}", section.tone)),
                    expanded: false,
                    blocks: vec![MessageBlock::BulletList(values.clone())],
                    children: Vec::new(),
                }),
                crate::engine::EnginePresentationBlock::Notice { tone, message } if !matches!(verbosity, crate::chat::Verbosity::Compact) => children.push(TranscriptNode {
                    key: format!("section.{index}.notice.{block_index}"),
                    label: format!("{tone:?}"),
                    summary: Some(message.clone()),
                    tone: Some(format!("{tone:?}")),
                    expanded: false,
                    blocks: vec![MessageBlock::Notice { tone: format!("{tone:?}"), text: message.clone() }],
                    children: Vec::new(),
                }),
                crate::engine::EnginePresentationBlock::Timeline(values) if !values.is_empty() => children.push(TranscriptNode {
                    key: format!("section.{index}.timeline.{block_index}"),
                    label: "Timeline".into(),
                    summary: Some(format!("{} item(s)", values.len())),
                    tone: Some(format!("{:?}", section.tone)),
                    expanded: false,
                    blocks: vec![MessageBlock::Timeline(values.clone())],
                    children: Vec::new(),
                }),
                crate::engine::EnginePresentationBlock::TechnicalRefs(values) if matches!(verbosity, crate::chat::Verbosity::FullStack) && !values.is_empty() => children.push(TranscriptNode {
                    key: format!("section.{index}.technical.{block_index}"),
                    label: "Technical details".into(),
                    summary: Some(format!("{} ref(s)", values.len())),
                    tone: Some(format!("{:?}", section.tone)),
                    expanded: false,
                    blocks: vec![MessageBlock::TechnicalList(values.clone())],
                    children: Vec::new(),
                }),
                _ => {}
            }
        }
        nodes.push(TranscriptNode {
            key: format!("section.{index}"),
            label: section.title.clone(),
            summary: None,
            tone: Some(format!("{:?}", section.tone)),
            expanded: false,
            blocks: Vec::new(),
            children,
        });
    }
    if !presentation.hints.is_empty() {
        nodes.push(TranscriptNode {
            key: "hints".into(),
            label: "Hints".into(),
            summary: Some(format!("{} hint(s)", presentation.hints.len())),
            tone: Some("info".into()),
            expanded: false,
            blocks: vec![MessageBlock::BulletList(presentation.hints.iter().map(|hint| format!("hint: {hint}")).collect())],
            children: Vec::new(),
        });
    }
    vec![MessageBlock::Tree(nodes)]
}

fn humanize_status(status: crate::engine::EngineStatus) -> String {
    match status {
        crate::engine::EngineStatus::Ok => "Completed successfully.".into(),
        crate::engine::EngineStatus::Unknown => "The engine could not produce an evidence-backed answer.".into(),
        crate::engine::EngineStatus::Unsupported => "This request is not supported by the current engine path.".into(),
        crate::engine::EngineStatus::Error => "The engine failed while processing the request.".into(),
    }
}

fn shorten_hash(value: &str) -> String {
    if value.len() <= 16 {
        value.to_string()
    } else {
        format!("{}..{}", &value[..8], &value[value.len().saturating_sub(8)..])
    }
}

fn debug_role_for(
    category: &crate::engine::DebugPresentationCategory,
) -> crate::chat::transcript::MessageRole {
    match category {
        crate::engine::DebugPresentationCategory::Facts => crate::chat::transcript::MessageRole::EngineFacts,
        crate::engine::DebugPresentationCategory::Evidence => crate::chat::transcript::MessageRole::EngineEvidence,
        crate::engine::DebugPresentationCategory::Interlingua => crate::chat::transcript::MessageRole::EngineInterlingua,
        crate::engine::DebugPresentationCategory::Entities => crate::chat::transcript::MessageRole::EngineEntities,
        crate::engine::DebugPresentationCategory::Query => crate::chat::transcript::MessageRole::EngineQuery,
        crate::engine::DebugPresentationCategory::Pipeline => crate::chat::transcript::MessageRole::EnginePipeline,
        crate::engine::DebugPresentationCategory::Sources => crate::chat::transcript::MessageRole::EngineSources,
        crate::engine::DebugPresentationCategory::Snapshot => crate::chat::transcript::MessageRole::EngineSnapshot,
        crate::engine::DebugPresentationCategory::Trace => crate::chat::transcript::MessageRole::EngineTrace,
        crate::engine::DebugPresentationCategory::Diagnostics => crate::chat::transcript::MessageRole::Error,
    }
}

fn response_meta_notes(session: &ChatSession, meta: &crate::engine::ResponseMeta) -> Vec<String> {
    let mut notes = vec![
        format!("status: {}", humanize_status(meta.status)),
        format!("snapshot: {}", meta.session_snapshot_id),
        format!("request: {}", meta.request_id),
    ];
    if !meta.diagnostics.is_empty() {
        notes.push(format!("diagnostics: {}", meta.diagnostics.join(", ")));
    }
    if !matches!(session.settings.verbosity, crate::chat::Verbosity::Compact) {
        notes.extend(
            meta.artifact_hashes
                .iter()
                .map(|(key, value)| format!("{key}: {}", shorten_hash(value))),
        );
    }
    if !matches!(session.runtime.trace_mode, crate::chat::TraceMode::Off) {
        notes.push(format!("trace: {}", meta.run_id));
    }
    notes
}

fn submit_engine_request(
    session: &mut ChatSession,
    worker: &ChatWorker,
    label: &str,
    request: crate::engine::EngineRequest,
) {
    session.start_stream(label);
    if let Err(err) = worker.submit(ChatJob::Engine { request }) {
        session.pending = None;
        session.activity = Default::default();
        session.push_error_message(err.to_string());
    }
}

fn copy_transcript_to_clipboard(session: &ChatSession) -> Result<(), String> {
    let text = session.export_transcript_text();
    copy_text_to_clipboard(&text)
}

fn copy_selected_to_clipboard(session: &ChatSession) -> Result<(), String> {
    let Some(selection) = session.selected_selection() else {
        return Err("no transcript entry selected".into());
    };
    let turn = selection.turn();
    let Some(message) = session.transcript.messages.iter().find(|message| message.turn == turn) else {
        return Err("selected transcript entry no longer exists".into());
    };
    let mut text = vec![format!("[{} #{}]", message.title, message.turn)];
    match selection {
        TranscriptSelection::Message { .. } => {
            text.extend(message.blocks.iter().map(render_block_for_copy));
        }
        TranscriptSelection::Node { key, .. } => {
            if let Some(node_text) = render_selected_node_for_copy(&message.blocks, key) {
                text.push(node_text);
            } else {
                text.extend(message.blocks.iter().map(render_block_for_copy));
            }
        }
    }
    copy_text_to_clipboard(&text.join("\n"))
}

fn render_block_for_copy(block: &crate::chat::transcript::MessageBlock) -> String {
    match block {
        crate::chat::transcript::MessageBlock::Section(value)
        | crate::chat::transcript::MessageBlock::Paragraph(value) => value.clone(),
        crate::chat::transcript::MessageBlock::KeyValue { key, value } => format!("{key}: {value}"),
        crate::chat::transcript::MessageBlock::BulletList(values) => values.iter().map(|value| format!("- {value}")).collect::<Vec<_>>().join("\n"),
        crate::chat::transcript::MessageBlock::Notice { tone, text } => format!("[{tone}] {text}"),
        crate::chat::transcript::MessageBlock::Timeline(values) => values.iter().map(|value| format!("> {value}")).collect::<Vec<_>>().join("\n"),
        crate::chat::transcript::MessageBlock::TechnicalList(values) => values.iter().map(|value| format!("# {value}")).collect::<Vec<_>>().join("\n"),
        crate::chat::transcript::MessageBlock::Trace(values) => values.join("\n"),
        crate::chat::transcript::MessageBlock::Tree(nodes) => nodes.iter().map(render_node_for_copy).collect::<Vec<_>>().join("\n"),
    }
}

fn render_node_for_copy(node: &crate::chat::transcript::TranscriptNode) -> String {
    let mut lines = vec![match &node.summary {
        Some(summary) => format!("{}: {summary}", node.label),
        None => node.label.clone(),
    }];
    lines.extend(node.blocks.iter().map(render_block_for_copy));
    lines.extend(node.children.iter().map(render_node_for_copy));
    lines.join("\n")
}

fn render_selected_node_for_copy(blocks: &[crate::chat::transcript::MessageBlock], key: &str) -> Option<String> {
    for block in blocks {
        if let crate::chat::transcript::MessageBlock::Tree(nodes) = block {
            if let Some(node) = find_node_for_copy(nodes, key) {
                return Some(render_node_for_copy(node));
            }
        }
    }
    None
}

fn find_node_for_copy<'a>(
    nodes: &'a [crate::chat::transcript::TranscriptNode],
    key: &str,
) -> Option<&'a crate::chat::transcript::TranscriptNode> {
    for node in nodes {
        if node.key == key {
            return Some(node);
        }
        if let Some(found) = find_node_for_copy(&node.children, key) {
            return Some(found);
        }
    }
    None
}

fn copy_text_to_clipboard(text: &str) -> Result<(), String> {
    for (program, args) in [
        ("wl-copy", Vec::<&str>::new()),
        ("xclip", vec!["-selection", "clipboard"]),
        ("xsel", vec!["--clipboard", "--input"]),
        ("pbcopy", Vec::<&str>::new()),
    ] {
        let mut command = std::process::Command::new(program);
        command.args(args);
        command.stdin(std::process::Stdio::piped());
        command.stdout(std::process::Stdio::null());
        command.stderr(std::process::Stdio::null());
        let Ok(mut child) = command.spawn() else { continue; };
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            if stdin.write_all(text.as_bytes()).is_err() {
                let _ = child.wait();
                continue;
            }
        }
        if child.wait().map(|status| status.success()).unwrap_or(false) {
            return Ok(());
        }
    }
    Err("no clipboard utility found; tried wl-copy, xclip, xsel, pbcopy".into())
}

fn submit_clear(session: &mut ChatSession, worker: &ChatWorker, target: crate::engine::ClearTarget) {
    session.clear_session();
    submit_engine_request(session, worker, "clear", crate::engine::EngineRequest::Session(crate::engine::SessionRequest::Clear { target }));
}

fn submit_ingest(session: &mut ChatSession, worker: &ChatWorker, title: String) {
    let language = session.chat_language_for(&title);
    let request = crate::engine::SourceRequest::wikipedia_snapshot(title, crate::core::interlingua::LanguageId::new(&language));
    submit_engine_request(
        session,
        worker,
        "ingest",
        crate::engine::EngineRequest::Conversation(crate::engine::ConversationRequest::IngestSource { source: request }),
    );
}

fn submit_query(session: &mut ChatSession, worker: &ChatWorker, query: crate::query::QueryInterlingua) {
    submit_engine_request(
        session,
        worker,
        "query",
        crate::engine::EngineRequest::Conversation(crate::engine::ConversationRequest::Query { query }),
    );
}

fn submit_inspect(session: &mut ChatSession, worker: &ChatWorker, target: crate::engine::InspectTarget) {
    submit_engine_request(
        session,
        worker,
        "inspect",
        crate::engine::EngineRequest::Conversation(crate::engine::ConversationRequest::Inspect { target }),
    );
}

fn submit_translate(
    session: &mut ChatSession,
    worker: &ChatWorker,
    text: String,
) {
    submit_engine_request(
        session,
        worker,
        "translate",
        crate::engine::EngineRequest::Translation(crate::engine::TranslationRequest::Turn {
            text,
            from: Some(if session.context.source_lang == "auto" { crate::engine::LanguageMode::Auto } else { crate::engine::LanguageMode::Explicit(session.context.source_lang.clone()) }),
            to: Some(crate::core::interlingua::LanguageId::new(&session.context.target_lang)),
        }),
    );
}

fn submit_user_turn(session: &mut ChatSession, worker: &ChatWorker, text: String) {
    let language = session
        .context
        .chat_language_override
        .clone()
        .map(crate::engine::LanguageMode::Explicit)
        .unwrap_or(crate::engine::LanguageMode::Auto);
    submit_engine_request(
        session,
        worker,
        "engine",
        crate::engine::EngineRequest::Conversation(crate::engine::ConversationRequest::Turn { text, language }),
    );
}

fn sync_command_popup(session: &mut ChatSession) {
    let suggestions = visible_suggestions(&session.composer.input);
    if session.composer.input.trim().starts_with('/')
        && !suggestions.is_empty()
    {
        session.overlays.command_popup.suggestions = suggestions;
        session.overlays.command_popup.selected_index = 0;
        session.overlays.active = Some(OverlayKind::CommandPalette);
        if !matches!(session.overlays.focus, FocusTarget::CommandPopup) {
            session.overlays.focus = FocusTarget::Composer;
        }
    } else if matches!(session.overlays.active, Some(OverlayKind::CommandPalette)) {
        close_overlay(session);
    }
}

fn close_overlay(session: &mut ChatSession) {
    session.overlays.active = None;
    session.overlays.focus = FocusTarget::Composer;
    session.overlays.command_popup.suggestions.clear();
    session.overlays.command_popup.selected_index = 0;
}

fn current_layout(
    terminal: &Terminal<CrosstermBackend<Stdout>>,
)-> Result<ChatLayout, Box<dyn Error>> {
    let size = terminal.size()?;
    Ok(layout_for(size))
}

#[cfg(test)]
mod tests {
    use super::handle_transcript_key;
    use crate::chat::session::{ChatSession, FocusTarget};
    use crate::chat::transcript::{MessageBlock, MessageMeta, MessageRole, TranscriptNode};
    use crate::chat::{ChatOptions, TraceMode};
    use crate::engine::SourceFetchPolicy;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn session_with_fact_tree() -> ChatSession {
        let mut session = ChatSession::new(ChatOptions {
            from: "pl".into(),
            to: "en".into(),
            data_dir: "data".into(),
            offline: true,
            trace_mode: TraceMode::Full,
            source_policy: SourceFetchPolicy::SnapshotOnly,
        });
        session.transcript.push(
            MessageRole::EngineFacts,
            "facts",
            vec![MessageBlock::Tree(vec![TranscriptNode {
                key: "facts".into(),
                label: "Facts".into(),
                summary: None,
                tone: None,
                expanded: true,
                blocks: vec![],
                children: vec![TranscriptNode {
                    key: "facts.answer".into(),
                    label: "Answer".into(),
                    summary: None,
                    tone: None,
                    expanded: false,
                    blocks: vec![MessageBlock::Paragraph("evidence".into())],
                    children: vec![],
                }],
            }])],
            MessageMeta::default(),
        );
        session.select_message(1);
        session.overlays.focus = FocusTarget::Transcript;
        session
    }

    #[test]
    fn plus_and_minus_control_tree_nodes_in_transcript_focus() {
        let mut session = session_with_fact_tree();
        handle_transcript_key(
            &mut session,
            KeyEvent::new(KeyCode::Char('+'), KeyModifiers::NONE),
        );
        assert!(session.node_expanded(1, "facts.answer", false));

        handle_transcript_key(
            &mut session,
            KeyEvent::new(KeyCode::Char('-'), KeyModifiers::NONE),
        );
        assert!(!session.node_expanded(1, "facts", true));
    }
}
