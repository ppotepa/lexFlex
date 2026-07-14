use crate::chat::commands::{parse_input, visible_suggestions, InputAction, SlashCommand};
use crate::chat::conversation_qa::{
    question_lookup_title, ConversationAnswer, ConversationOutcome,
};
use crate::chat::navigation;
use crate::chat::service::{ChatJob, ChatJobOutput, ChatJobResult, ChatTurn, ChatWorker};
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
use crate::query::AnswerStatus;

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
            let history = conversation_history(session);
            let turn = session.push_user_message(text.clone());
            match session.context.mode {
                ChatMode::Chat => {
                    handle_chat_message(session, worker, text, turn, history);
                }
                ChatMode::Translate => {
                    session.pending = Some(PendingRequest {
                        label: "translate".to_string(),
                        started_at: Instant::now(),
                    });
                    if let Err(err) = worker.submit(ChatJob::Translate {
                        input: text,
                        context: session.context.clone(),
                        settings: session.settings.clone(),
                    }) {
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
            SlashCommand::Qa => {
                session.set_mode(ChatMode::Chat);
                session.push_system_message("/qa is an alias for /chat; normal conversation mode enabled.");
            }
            SlashCommand::Ask(text) => {
                session.push_user_message(input.clone());
                let language = session.chat_language_for(&text);
                let answer = session.knowledge.answer_question(&text, &language);
                session.push_response(conversation_answer_to_chat_response(answer), None);
            }
            SlashCommand::Wiki(args) => {
                let Some((title, text)) = parse_wiki_args(&args) else {
                    session.push_error_message("use /wiki <title> :: <text> to ingest a Wikipedia snapshot");
                    return Ok(());
                };
                let language = session.chat_language_for(text);
                let report = session
                    .knowledge
                    .ingest_wikipedia_article(title, text, &language);
                session.push_system_message(format!(
                    "ingested wikipedia source {} with {} facts.",
                    title,
                    report.fact_ids.len()
                ));
                if !report.fact_summaries.is_empty() {
                    session.push_system_message(report.fact_summaries.join(" | "));
                }
            }
            SlashCommand::Explain(text) => {
                session.push_user_message(input.clone());
                let context = session.context_for_language(session.chat_language_for(&text));
                session.pending = Some(PendingRequest {
                    label: "explain".to_string(),
                    started_at: Instant::now(),
                });
                if let Err(err) = worker.submit(ChatJob::Explain {
                    input: text,
                    context,
                }) {
                    session.pending = None;
                    session.push_error_message(err.to_string());
                }
            }
            SlashCommand::Parse(text) => {
                session.push_user_message(input.clone());
                let context = session.context_for_language(session.chat_language_for(&text));
                session.pending = Some(PendingRequest {
                    label: "parse".to_string(),
                    started_at: Instant::now(),
                });
                if let Err(err) = worker.submit(ChatJob::Parse {
                    input: text,
                    context,
                }) {
                    session.pending = None;
                    session.push_error_message(err.to_string());
                }
            }
            SlashCommand::Learn(text) => {
                session.push_user_message(input.clone());
                let context = session.context_for_language(session.chat_language_for(&text));
                session.pending = Some(PendingRequest {
                    label: "learn".to_string(),
                    started_at: Instant::now(),
                });
                if let Err(err) = worker.submit(ChatJob::Learn {
                    input: text,
                    context,
                }) {
                    session.pending = None;
                    session.push_error_message(err.to_string());
                }
            }
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

fn parse_wiki_args(args: &str) -> Option<(&str, &str)> {
    let (title, text) = args.split_once("::")?;
    let title = title.trim();
    let text = text.trim();
    if title.is_empty() || text.is_empty() {
        None
    } else {
        Some((title, text))
    }
}

fn conversation_history(session: &ChatSession) -> Vec<ChatTurn> {
    session
        .transcript
        .messages
        .iter()
        .filter_map(|message| {
            let content = message.blocks.iter().find_map(|block| match block {
                crate::chat::transcript::MessageBlock::Paragraph(text) => Some(text.clone()),
                _ => None,
            })?;
            let role = match message.role {
                crate::chat::transcript::MessageRole::User => "user",
                crate::chat::transcript::MessageRole::Assistant => "assistant",
                _ => return None,
            };
            Some(ChatTurn {
                role: role.to_string(),
                content,
            })
        })
        .collect()
}

fn handle_chat_message(
    session: &mut ChatSession,
    worker: &ChatWorker,
    text: String,
    turn: usize,
    history: Vec<ChatTurn>,
) {
    let language = session.chat_language_for(&text);
    if let Some(mut trace) = session
        .semantic
        .as_ref()
        .and_then(|semantic| semantic.inspect(&text, &language).ok())
    {
        trace.turn = turn;
        session.record_trace(trace);
    }
    if let Some(semantic) = session.semantic.as_mut() {
        if crate::chat::conversation_qa::looks_like_question(&text, &language) {
            if let Ok(Some(answer)) = semantic.answer(&text, &language) {
                session.finish_trace(
                    turn,
                    0,
                    Some("supported".to_string()),
                    Some(answer.text.clone()),
                    answer.evidence.clone(),
                    None,
                );
                let mut notes = answer.evidence;
                if trace_is_visible(session) {
                    notes.extend(session.trace_notes_for(turn));
                }
                session.push_response(crate::chat::service::ChatResponse {
                    title: "qa".to_string(),
                    blocks: vec![crate::chat::transcript::MessageBlock::Paragraph(answer.text)],
                    notes,
                    latency_ms: 0,
                }, None);
                return;
            }
        } else if let Ok(added) = semantic.observe(&text, &language) {
            if added > 0 {
                session.finish_trace(turn, added, None, None, Vec::new(), None);
                if trace_is_visible(session) {
                    session.push_system_message(session.trace_notes_for(turn).join(" | "));
                }
                session.push_system_message(format!("stored {added} semantic fact(s)."));
                return;
            }
        }
    }
    if crate::chat::conversation_qa::looks_like_question(&text, &language) {
        let answer = session.knowledge.answer_question(&text, &language);
        session.finish_trace(
            turn,
            0,
            Some(format!("{:?}", answer.status)),
            Some(answer.text.clone()),
            answer
                .evidence
                .iter()
                .map(|item| item.text.clone())
                .collect(),
            Some("conversation knowledge fallback".to_string()),
        );
        if !matches!(answer.status, AnswerStatus::Unknown | AnswerStatus::Unsupported) {
            apply_knowledge_outcome(session, ConversationOutcome::Answered(answer));
            return;
        }
        if let Some(title) = question_lookup_title(&text, &language) {
            session.pending = Some(PendingRequest {
                label: format!("wikipedia:{title}"),
                started_at: Instant::now(),
            });
            if let Err(err) = worker.submit(ChatJob::WikipediaLookup {
                question: text,
                language,
                title,
            }) {
                session.pending = None;
                session.push_error_message(err.to_string());
            }
            return;
        }
        if trace_is_visible(session) {
            session.push_system_message(session.trace_notes_for(turn).join(" | "));
        }
        apply_knowledge_outcome(session, ConversationOutcome::Answered(answer));
        return;
    }

    let outcome = session.knowledge.observe_text(turn, &text, &language);
    if let ConversationOutcome::Ingested(report) = &outcome {
        session.finish_trace(turn, report.fact_ids.len(), None, None, Vec::new(), None);
        if !report.fact_ids.is_empty() {
            apply_knowledge_outcome(session, outcome);
            return;
        }
    }
    session.finish_trace(
        turn,
        0,
        None,
        None,
        Vec::new(),
        Some("LLM conversation fallback".to_string()),
    );
    session.pending = Some(PendingRequest {
        label: "chat".to_string(),
        started_at: Instant::now(),
    });
    if let Err(err) = worker.submit(ChatJob::Chat {
        input: text,
        context: session.context_for_language(language),
        history,
    }) {
        session.pending = None;
        session.push_error_message(err.to_string());
    }
}

fn trace_is_visible(session: &ChatSession) -> bool {
    matches!(session.runtime.trace_mode, crate::chat::TraceMode::Full)
        || matches!(session.settings.verbosity, crate::chat::Verbosity::Detailed)
}

fn apply_knowledge_outcome(session: &mut ChatSession, outcome: ConversationOutcome) {
    match outcome {
        ConversationOutcome::Answered(answer) => {
            session.push_response(conversation_answer_to_chat_response(answer), None);
        }
        ConversationOutcome::Ingested(report) => {
            session.push_system_message(format!(
                "stored {} fact(s) from {}.",
                report.fact_ids.len(),
                report.source_label
            ));
            if !report.fact_summaries.is_empty() {
                session.push_system_message(report.fact_summaries.join(" | "));
            }
        }
        ConversationOutcome::Unsupported(note) => {
            session.push_error_message(note);
        }
    }
}

fn conversation_answer_to_chat_response(answer: ConversationAnswer) -> crate::chat::service::ChatResponse {
    let mut notes = answer.notes;
    if notes.is_empty() {
        notes = answer
            .evidence
            .iter()
            .map(|item| format!(
                "{} [{}..{}]",
                item.text,
                item.span.start,
                item.span.end
            ))
            .collect();
    }
    crate::chat::service::ChatResponse {
        title: "qa".to_string(),
        blocks: vec![crate::chat::transcript::MessageBlock::Paragraph(answer.text)],
        notes,
        latency_ms: 0,
    }
}

fn apply_job_result(session: &mut ChatSession, result: ChatJobResult) {
    session.pending = None;
    match result.result {
        Ok(ChatJobOutput::Response(response)) => session.push_response(response, None),
        Ok(ChatJobOutput::WikipediaArticle {
            question,
            language,
            article,
        }) => {
            let pipeline_result = crate::api::LexFlexAPI::builder()
                .data_dir(&session.runtime.data_dir)
                .build()
                .map_err(|error| error.to_string())
                .and_then(|api| {
                    crate::runtime::LexFlexDocumentEngine::new(api)
                        .ingest_document_bundle(&article.text, &language)
                        .map_err(|error| error.to_string())
                });
            let report = session
                .knowledge
                .ingest_wikipedia_article_with_url(
                    &article.title,
                    &article.text,
                    &language,
                    Some(&article.url),
                );
            if report.fact_ids.is_empty() {
                session.push_error_message(format!(
                    "Wikipedia source '{}' was fetched, but no supported facts were extracted.",
                    article.title
                ));
            } else {
                let answer = session.knowledge.answer_question(&question, &language);
                let turn = session
                    .transcript
                    .messages
                    .iter()
                    .rev()
                    .find(|message| matches!(message.role, crate::chat::transcript::MessageRole::User))
                    .map(|message| message.turn);
                if let Some(turn) = turn {
                    session.finish_trace(
                        turn,
                        0,
                        Some(format!("{:?}", answer.status)),
                        Some(answer.text.clone()),
                        answer
                            .evidence
                            .iter()
                            .map(|item| item.text.clone())
                            .collect(),
                        Some("Wikipedia source fallback".to_string()),
                    );
                }
                let mut response = conversation_answer_to_chat_response(answer);
                response.notes.push(format!("source: {}", article.url));
                match pipeline_result {
                    Ok(bundle) => {
                        response.notes.push(format!(
                            "document pipeline: sentences={} graph_nodes={} claims={} hash={}",
                            bundle.compilation.summary.total_sentences,
                            bundle.graph.nodes.len(),
                            bundle.knowledge.summary.claims_total,
                            bundle.source_sha256
                        ));
                        session.document_bundles.push(bundle);
                    }
                    Err(error) => response.notes.push(format!("document pipeline error: {error}")),
                }
                if trace_is_visible(session) {
                    if let Some(turn) = turn {
                        response.notes.extend(session.trace_notes_for(turn));
                    }
                }
                session.push_response(response, None);
            }
        }
        Err(error) => session.push_error_message(error),
    }
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
