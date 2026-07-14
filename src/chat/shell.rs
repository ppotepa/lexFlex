use crate::chat::commands::{parse_input, visible_suggestions, InputAction, SlashCommand};
use crate::chat::navigation;
use crate::chat::service::{ChatJob, ChatJobOutput, ChatJobResult, ChatWorker};
use crate::chat::session::{ChatMode, ChatSession, FocusTarget, OverlayKind, PendingRequest};
use crate::chat::tui::input::{
    backspace, delete_forward, insert_char, insert_newline, insert_text, move_down, move_left,
    move_right, move_up, recall_next_input, recall_previous_input,
};
use crate::chat::tui::render::{draw, main_layout};
use crate::chat::widgets::chat_window::max_scroll_for_height;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::error::Error;
use std::io::Stdout;
use std::time::{Duration, Instant};

pub fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    session: &mut ChatSession,
    service: crate::chat::service::ChatService,
) -> Result<(), Box<dyn Error>> {
    let worker = ChatWorker::spawn(service);
    let tick_rate = Duration::from_millis(50);
    loop {
        while let Some(result) = worker.try_receive() {
            apply_job_result(session, result);
        }
        terminal.draw(|frame| draw(frame, session))?;
        if session.should_quit {
            break;
        }
        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key) => {
                    let chat_area = current_chat_area(terminal)?;
                    if handle_key_event(session, &worker, key, chat_area)? {
                        break;
                    }
                }
                Event::Resize(_, _) => {}
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
    worker: &ChatWorker,
    key: KeyEvent,
    chat_area: ratatui::layout::Rect,
) -> Result<bool, Box<dyn Error>> {
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return Ok(false);
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
        return Ok(true);
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('l')) {
        session.clear_session();
        session.push_system_message("screen cleared.");
        return Ok(false);
    }
    if key.modifiers.contains(KeyModifiers::ALT) && matches!(key.code, KeyCode::Char('v')) {
        let verbosity = session.cycle_verbosity();
        session.push_system_message(format!("verbosity set to {}.", verbosity.as_str()));
        return Ok(false);
    }

    match session.overlays.focus {
        FocusTarget::CommandPopup => handle_command_popup_key(session, key),
        FocusTarget::Composer => handle_composer_key(session, worker, key, chat_area)?,
    }
    Ok(false)
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
    chat_area: ratatui::layout::Rect,
) -> Result<(), Box<dyn Error>> {
    let max_scroll = max_scroll_for_height(&session.transcript.messages, chat_area);
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
        KeyCode::PageUp => navigation::page_up(session, 8, max_scroll),
        KeyCode::PageDown => navigation::page_down(session, 8, max_scroll),
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => {
            insert_newline(&mut session.composer);
            sync_command_popup(session);
        }
        KeyCode::Enter => submit_input(session, worker)?,
        _ => {}
    }
    Ok(())
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
                ChatMode::Chat => {
                    handle_chat_message(session, worker, text, turn);
                }
                ChatMode::Translate => {
                    session.pending = Some(PendingRequest {
                        label: "translate".to_string(),
                        started_at: Instant::now(),
                    });
                    if let Err(err) = worker.submit(ChatJob::Engine { request: crate::engine::EngineRequest::Translate { text, from: crate::core::interlingua::LanguageId::new(&session.context.source_lang), to: crate::core::interlingua::LanguageId::new(&session.context.target_lang) } }) {
                        session.pending = None;
                        session.push_error_message(err.to_string());
                    }
                }
            }
        }
        InputAction::Slash(command) => match command {
            SlashCommand::Chat => {
                session.set_mode(ChatMode::Chat);
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
                session.set_mode(ChatMode::Translate);
                session.push_system_message(format!("translation direction set to {} -> {}.", from, to));
            }
            SlashCommand::Settings => {
                let verbosity = session.cycle_verbosity();
                session.push_system_message(format!("verbosity set to {}.", verbosity.as_str()));
            }
            SlashCommand::Ingest(text) => {
                session.push_user_message(input.clone());
                let language = session.chat_language_for(&text);
                let request = crate::engine::SourceRequest::wikipedia_snapshot(text, crate::core::interlingua::LanguageId::new(&language));
                session.pending = Some(PendingRequest { label: "ingest".into(), started_at: Instant::now() });
                if let Err(err) = worker.submit(ChatJob::Engine { request: crate::engine::EngineRequest::IngestSource { source: request } }) { session.pending = None; session.push_error_message(err.to_string()); }
            }
            SlashCommand::Query(text) => {
                match serde_json::from_str::<crate::query::QueryInterlingua>(&text) {
                    Ok(query) => {
                        session.pending = Some(PendingRequest { label: "query".into(), started_at: Instant::now() });
                        if let Err(err) = worker.submit(ChatJob::Engine { request: crate::engine::EngineRequest::Query { query } }) {
                            session.pending = None;
                            session.push_error_message(err.to_string());
                        }
                    }
                    Err(err) => session.push_error_message(format!("/query expects QueryInterlingua JSON: {err}")),
                }
            }
            SlashCommand::Inspect(target) => {
                session.pending = Some(PendingRequest { label: "inspect".into(), started_at: Instant::now() });
                let target = match target.as_str() { "sources" => crate::engine::InspectTarget::Sources, "bundles" => crate::engine::InspectTarget::Bundles, _ => crate::engine::InspectTarget::Session };
                if let Err(err) = worker.submit(ChatJob::Engine { request: crate::engine::EngineRequest::Inspect { target } }) { session.pending = None; session.push_error_message(err.to_string()); }
            }
            SlashCommand::Trace => session.push_system_message("engine trace visibility follows the configured trace mode."),
            SlashCommand::Clear => { session.clear_session(); session.push_system_message("session cleared."); }
            SlashCommand::Quit => session.should_quit = true,
        },
        InputAction::IncompleteSlash(draft) => {
            if draft.command == "translate" {
                session.push_error_message(
                    "invalid translate command; use /translate en pl or /translate pl en.",
                );
            } else if draft.command == "lang" {
                session.push_error_message("invalid language command; use /lang en, /lang pl, or /lang auto.");
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
    let _language = session.chat_language_for(&text);
    session.pending = Some(PendingRequest {
        label: "engine".to_string(),
        started_at: Instant::now(),
    });
    let language = session.context.chat_language_override.clone().map(crate::engine::LanguageMode::Explicit).unwrap_or(crate::engine::LanguageMode::Auto);
    if let Err(err) = worker.submit(ChatJob::Engine { request: crate::engine::EngineRequest::UserTurn { text, language } }) {
        session.pending = None;
        session.push_error_message(err.to_string());
    }
}

fn apply_job_result(session: &mut ChatSession, result: ChatJobResult) {
    session.pending = None;
    match result.result {
        Ok(ChatJobOutput::Engine(response)) => apply_engine_response(session, response),
        Err(error) => session.push_error_message(error),
    }
}

fn apply_engine_response(session: &mut ChatSession, response: crate::engine::EngineResponse) {
    use crate::engine::EngineResponse;
    let (title, text, notes) = match response {
        EngineResponse::Conversation(value) => (
            "conversation",
            value.text.unwrap_or_else(|| format!("status: {:?}", value.meta.status)),
            value.answer.map(|answer| answer.evidence).unwrap_or_default(),
        ),
        EngineResponse::Translation(value) => ("translation", value.text, vec![format!("{} -> {}", value.meta.session_snapshot_id, value.meta.request_id)]),
        EngineResponse::Ingest(value) => ("ingest", format!("ingested {}", value.source_id), vec![format!("snapshot: {}", value.meta.session_snapshot_id), format!("bundle: {}", value.bundle_id)]),
        EngineResponse::Answer { answer, .. } => ("answer", answer.text.unwrap_or_else(|| format!("status: {:?}", answer.status)), answer.evidence),
        EngineResponse::Inspection(value) => ("inspection", serde_json::to_string_pretty(&value.values).unwrap_or_default(), vec![]),
        EngineResponse::Error { error, .. } => ("engine error", format!("{error:?}"), vec![]),
    };
    session.push_response(crate::chat::session::ChatResponse { title: title.into(), blocks: vec![crate::chat::transcript::MessageBlock::Paragraph(text)], notes, latency_ms: 0 }, None);
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

fn current_chat_area(
    terminal: &Terminal<CrosstermBackend<Stdout>>,
) -> Result<ratatui::layout::Rect, Box<dyn Error>> {
    let size = terminal.size()?;
    let root = main_layout(size);
    Ok(root[1])
}
