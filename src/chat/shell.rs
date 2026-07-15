use crate::chat::commands::{parse_input, visible_suggestions, InputAction, SlashCommand};
use crate::chat::navigation;
use crate::chat::service::{ChatJob, ChatJobOutput, ChatJobResult, ChatWorker, ChatWorkerEvent};
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
        while let Some(event) = worker.try_receive() {
            match event {
                ChatWorkerEvent::Progress(progress) => apply_progress(session, progress),
                ChatWorkerEvent::Result(result) => apply_job_result(session, result),
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
                    submit_translate(session, worker, text);
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
                submit_ingest(session, worker, text);
            }
            SlashCommand::Query(text) => {
                match serde_json::from_str::<crate::query::QueryInterlingua>(&text) {
                    Ok(query) => {
                        submit_query(session, worker, query);
                    }
                    Err(err) => session.push_error_message(format!("/query expects QueryInterlingua JSON: {err}")),
                }
            }
            SlashCommand::Inspect(target) => {
                let target = match target.as_str() { "sources" => crate::engine::InspectTarget::Sources, "bundles" => crate::engine::InspectTarget::Bundles, _ => crate::engine::InspectTarget::Session };
                submit_inspect(session, worker, target);
            }
            SlashCommand::Trace => push_trace_message(session),
            SlashCommand::Clear => submit_clear(session, worker),
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

fn apply_progress(session: &mut ChatSession, event: crate::engine::TraceEvent) {
    let full = matches!(session.settings.verbosity, crate::chat::Verbosity::FullStack)
        || matches!(session.runtime.trace_mode, crate::chat::TraceMode::Full);
    session.append_stream_progress(&event, full);
}

fn apply_engine_response(session: &mut ChatSession, response: crate::engine::EngineResponse, latency_ms: u128) {
    use crate::engine::EngineResponse;
    let (title, text, mut notes, meta) = match response {
        EngineResponse::Conversation(value) => {
            let notes = value.answer.as_ref().map(|answer| answer.evidence.clone()).unwrap_or_default();
            ("conversation", value.text.unwrap_or_else(|| format!("status: {:?}", value.meta.status)), notes, value.meta)
        }
        EngineResponse::Translation(value) => ("translation", value.text, Vec::new(), value.meta),
        EngineResponse::Ingest(value) => ("ingest", format!("ingested {}", value.source_id), vec![format!("bundle: {}", value.bundle_id)], value.meta),
        EngineResponse::Answer { meta, answer } => ("answer", answer.text.unwrap_or_else(|| format!("status: {:?}", answer.status)), answer.evidence, meta),
        EngineResponse::Inspection(value) => ("inspection", serde_json::to_string_pretty(&value.values).unwrap_or_default(), vec![], value.meta),
        EngineResponse::Error { meta, error } => ("engine error", format!("{error:?}"), vec![], meta),
    };
    session.last_trace_run_id = Some(meta.run_id.clone());
    notes.extend(response_meta_notes(session, &meta));
    let mut blocks = vec![crate::chat::transcript::MessageBlock::Paragraph(text)];
    if !notes.is_empty() {
        blocks.push(crate::chat::transcript::MessageBlock::BulletList(notes));
    }
    let include_trace = matches!(session.settings.verbosity, crate::chat::Verbosity::FullStack)
        || matches!(session.runtime.trace_mode, crate::chat::TraceMode::Full);
    session.finish_stream(
        title.into(),
        blocks,
        crate::chat::transcript::MessageMeta { latency_ms: Some(latency_ms), command: None },
        include_trace,
    );
}

fn response_meta_notes(session: &ChatSession, meta: &crate::engine::ResponseMeta) -> Vec<String> {
    let mut notes = vec![
        format!("status: {:?}", meta.status),
        format!("snapshot: {}", meta.session_snapshot_id),
        format!("request: {}", meta.request_id),
    ];
    if !meta.diagnostics.is_empty() {
        notes.push(format!("diagnostics: {}", meta.diagnostics.join(", ")));
    }
    if !matches!(session.settings.verbosity, crate::chat::Verbosity::Compact) {
        notes.extend(meta.artifact_hashes.iter().map(|(key, value)| format!("{key}: {value}")));
    }
    if !matches!(session.runtime.trace_mode, crate::chat::TraceMode::Off) {
        notes.push(format!("trace: {}", meta.run_id));
    }
    notes
}

fn load_trace_notes(session: &ChatSession, run_id: &str) -> Vec<String> {
    let store = crate::engine::SessionStore::new(&session.runtime.data_dir);
    let Ok(trace) = store.load_trace(&session.runtime.engine_session_id, run_id) else {
        return vec![format!("trace: unavailable for {run_id}")];
    };
    let mut errors = Vec::new();
    let mut events = Vec::new();
    for (index, line) in trace.lines().enumerate() {
        match serde_json::from_str::<crate::engine::TraceEvent>(line) {
            Ok(event) => events.push(event),
            Err(error) => errors.push(format!("trace: corrupt line {}: {}", index + 1, error)),
        }
    }
    if events.is_empty() {
        if errors.is_empty() {
            return vec![format!("trace: empty for {run_id}")];
        }
        return errors;
    }
    if matches!(session.runtime.trace_mode, crate::chat::TraceMode::Brief) {
        events.sort_by(|left, right| left.stage.cmp(&right.stage));
        let mut stages = events.into_iter().map(|event| event.stage).collect::<Vec<_>>();
        stages.dedup();
        let mut notes = vec![format!("trace: {}", stages.join(" -> "))];
        notes.extend(errors);
        return notes;
    }
    let mut notes = events
        .into_iter()
        .map(|event| match event.payload {
            Some(payload) => format!("trace {} {}", event.stage, payload),
            None => format!("trace {}", event.stage),
        })
        .collect::<Vec<_>>();
    notes.extend(errors);
    notes
}

fn push_trace_message(session: &mut ChatSession) {
    let store = crate::engine::SessionStore::new(&session.runtime.data_dir);
    let run_id = session.last_trace_run_id.clone().or_else(|| store.latest_trace(&session.runtime.engine_session_id).ok().map(|value| value.0));
    let Some(run_id) = run_id else {
        session.push_system_message("no trace available yet.");
        return;
    };
    let notes = load_trace_notes(session, &run_id);
    session.push_response(crate::chat::session::ChatResponse {
        title: "trace".into(),
        blocks: vec![crate::chat::transcript::MessageBlock::Paragraph(format!("trace for {run_id}"))],
        notes,
        latency_ms: 0,
    }, None);
}

fn submit_engine_request(
    session: &mut ChatSession,
    worker: &ChatWorker,
    label: &str,
    request: crate::engine::EngineRequest,
) {
    session.pending = Some(PendingRequest {
        label: label.into(),
        started_at: Instant::now(),
        stream_turn: session.start_stream(label),
        current_stage: "queued".into(),
        progress_lines: Vec::new(),
        spinner_index: 0,
    });
    if let Err(err) = worker.submit(ChatJob::Engine { request }) {
        session.pending = None;
        session.push_error_message(err.to_string());
    }
}

fn submit_clear(session: &mut ChatSession, worker: &ChatWorker) {
    session.clear_session();
    submit_engine_request(session, worker, "clear", crate::engine::EngineRequest::ClearSession);
}

fn submit_ingest(session: &mut ChatSession, worker: &ChatWorker, title: String) {
    let language = session.chat_language_for(&title);
    let request = crate::engine::SourceRequest::wikipedia_snapshot(title, crate::core::interlingua::LanguageId::new(&language));
    submit_engine_request(
        session,
        worker,
        "ingest",
        crate::engine::EngineRequest::IngestSource { source: request },
    );
}

fn submit_query(session: &mut ChatSession, worker: &ChatWorker, query: crate::query::QueryInterlingua) {
    submit_engine_request(
        session,
        worker,
        "query",
        crate::engine::EngineRequest::Query { query },
    );
}

fn submit_inspect(session: &mut ChatSession, worker: &ChatWorker, target: crate::engine::InspectTarget) {
    submit_engine_request(
        session,
        worker,
        "inspect",
        crate::engine::EngineRequest::Inspect { target },
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
        crate::engine::EngineRequest::Translate {
            text,
            from: crate::core::interlingua::LanguageId::new(&session.context.source_lang),
            to: crate::core::interlingua::LanguageId::new(&session.context.target_lang),
        },
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
        crate::engine::EngineRequest::UserTurn { text, language },
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

fn current_chat_area(
    terminal: &Terminal<CrosstermBackend<Stdout>>,
) -> Result<ratatui::layout::Rect, Box<dyn Error>> {
    let size = terminal.size()?;
    let root = main_layout(size);
    Ok(root[1])
}
