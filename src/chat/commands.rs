#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction {
    Empty,
    UserMessage(String),
    Slash(SlashCommand),
    IncompleteSlash(SlashDraft),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommand {
    Chat,
    Lang(Option<String>),
    Translate { from: String, to: String },
    AnswerLanguage(String),
    Settings,
    Ingest(String),
    Query(String),
    Inspect(String),
    Facts(FactsCommand),
    Evidence(String),
    Debug(String),
    Trace,
    Clear(Option<String>),
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactsCommand {
    pub selector: String,
    pub role: crate::engine::DebugFactRole,
    pub limit: Option<usize>,
    pub page: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlashDraft {
    pub command: String,
    pub args: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlashSuggestion {
    pub label: String,
    pub replacement: String,
    pub description: String,
}

pub fn parse_input(input: &str) -> InputAction {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return InputAction::Empty;
    }
    if !trimmed.starts_with('/') {
        return InputAction::UserMessage(trimmed.to_string());
    }

    let rest = trimmed.trim_start_matches('/');
    let mut parts = rest.splitn(2, char::is_whitespace);
    let command = parts.next().unwrap_or("").trim().to_lowercase();
    let args = parts.next().unwrap_or("").trim().to_string();

    match command.as_str() {
        "chat" => InputAction::Slash(SlashCommand::Chat),
        "lang" => match args.to_lowercase().as_str() {
            "en" | "pl" => InputAction::Slash(SlashCommand::Lang(Some(args.to_lowercase()))),
            "auto" => InputAction::Slash(SlashCommand::Lang(None)),
            _ => InputAction::IncompleteSlash(SlashDraft { command, args }),
        },
        "translate" => {
            let bits: Vec<&str> = args.split_whitespace().collect();
            match bits.as_slice() {
                [from, to] if (*from == "auto" || is_supported_lang(from)) && is_supported_lang(to) => {
                    InputAction::Slash(SlashCommand::Translate {
                        from: (*from).to_string(),
                        to: (*to).to_string(),
                    })
                }
                _ => InputAction::IncompleteSlash(SlashDraft { command, args }),
            }
        }
        "answer-lang" => match args.to_lowercase().as_str() {
            "auto" | "source" | "en" | "pl" => InputAction::Slash(SlashCommand::AnswerLanguage(args.to_lowercase())),
            _ => InputAction::IncompleteSlash(SlashDraft { command, args }),
        },
        "settings" => InputAction::Slash(SlashCommand::Settings),
        "ingest" => InputAction::Slash(SlashCommand::Ingest(args)),
        "query" => InputAction::Slash(SlashCommand::Query(args)),
        "inspect" => InputAction::Slash(SlashCommand::Inspect(args)),
        "facts" => match parse_facts(&args) {
            Some(command) => InputAction::Slash(SlashCommand::Facts(command)),
            None => InputAction::IncompleteSlash(SlashDraft { command, args }),
        },
        "evidence" => {
            if args.trim().is_empty() {
                InputAction::IncompleteSlash(SlashDraft { command, args })
            } else {
                InputAction::Slash(SlashCommand::Evidence(args))
            }
        }
        "debug" => match args.to_lowercase().as_str() {
            "interlingua" | "entities" | "query" | "pipeline" | "sources" | "snapshot" | "trace" => {
                InputAction::Slash(SlashCommand::Debug(args.to_lowercase()))
            }
            _ => InputAction::IncompleteSlash(SlashDraft { command, args }),
        },
        "trace" => InputAction::Slash(SlashCommand::Trace),
        "clear" => match args.to_lowercase().as_str() {
            "" => InputAction::Slash(SlashCommand::Clear(None)),
            "conversation" | "translation" | "all" => InputAction::Slash(SlashCommand::Clear(Some(args.to_lowercase()))),
            _ => InputAction::IncompleteSlash(SlashDraft { command, args }),
        },
        "quit" => InputAction::Slash(SlashCommand::Quit),
        _ => InputAction::IncompleteSlash(SlashDraft { command, args }),
    }
}

pub fn visible_suggestions(input: &str) -> Vec<SlashSuggestion> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return vec![];
    }
    if matches!(
        parse_input(trimmed),
        InputAction::Slash(
            SlashCommand::Chat
                | SlashCommand::Lang(_)
                | SlashCommand::Translate { .. }
                | SlashCommand::Settings
        )
    ) {
        return vec![];
    }

    let query = trimmed.to_lowercase();
    visible_catalog()
        .into_iter()
        .filter(|item| {
            item.replacement.starts_with(&query)
                || item.label.starts_with(query.trim_start_matches('/'))
        })
        .collect()
}

fn visible_catalog() -> Vec<SlashSuggestion> {
    vec![
        SlashSuggestion {
            label: "chat".to_string(),
            replacement: "/chat".to_string(),
            description: "Switch to normal conversation mode".to_string(),
        },
        SlashSuggestion {
            label: "lang".to_string(),
            replacement: "/lang auto".to_string(),
            description: "Detect Polish or English for every chat message".to_string(),
        },
        SlashSuggestion {
            label: "lang".to_string(),
            replacement: "/lang pl".to_string(),
            description: "Force Polish for chat messages".to_string(),
        },
        SlashSuggestion {
            label: "lang".to_string(),
            replacement: "/lang en".to_string(),
            description: "Force English for chat messages".to_string(),
        },
        SlashSuggestion {
            label: "translate".to_string(),
            replacement: "/translate en pl".to_string(),
            description: "Set translation direction to English -> Polish".to_string(),
        },
        SlashSuggestion {
            label: "translate".to_string(),
            replacement: "/translate pl en".to_string(),
            description: "Set translation direction to Polish -> English".to_string(),
        },
        SlashSuggestion {
            label: "facts".to_string(),
            replacement: "/facts ".to_string(),
            description: "List evidence-backed facts from the current engine session".to_string(),
        },
        SlashSuggestion {
            label: "evidence".to_string(),
            replacement: "/evidence ".to_string(),
            description: "Inspect evidence for a claim ID or the last fact number".to_string(),
        },
        SlashSuggestion {
            label: "debug".to_string(),
            replacement: "/debug interlingua".to_string(),
            description: "Inspect the last turn's engine debug views".to_string(),
        },
    ]
}

fn parse_facts(args: &str) -> Option<FactsCommand> {
    let tokens = shell_tokens(args)?;
    let mut selector = Vec::new();
    let mut role = crate::engine::DebugFactRole::Any;
    let mut limit = Some(12usize);
    let mut page = 1usize;
    let mut index = 0;
    let mut explicit_limit = false;
    let mut explicit_page = false;
    let mut all = false;
    while index < tokens.len() {
        match tokens[index].as_str() {
            "--role" => {
                index += 1;
                role = match tokens.get(index).map(String::as_str) {
                    Some("any") => crate::engine::DebugFactRole::Any,
                    Some("subject") => crate::engine::DebugFactRole::Subject,
                    Some("object") => crate::engine::DebugFactRole::Object,
                    _ => return None,
                };
            }
            "--limit" => {
                if all || explicit_limit { return None }
                index += 1;
                let value = tokens.get(index)?.parse::<usize>().ok()?;
                if value == 0 { return None }
                limit = Some(value);
                explicit_limit = true;
            }
            "--page" => {
                if explicit_page { return None }
                index += 1;
                let value = tokens.get(index)?.parse::<usize>().ok()?;
                if value == 0 { return None }
                page = value;
                explicit_page = true;
            }
            "--all" => {
                if all || explicit_limit { return None }
                all = true;
                limit = None;
            }
            value if value.starts_with("--") => return None,
            value => selector.push(value.to_string()),
        }
        index += 1;
    }
    let selector = selector.join(" ").trim().to_string();
    if selector.is_empty() { return None }
    Some(FactsCommand { selector, role, limit, page })
}

fn shell_tokens(input: &str) -> Option<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    for ch in input.chars() {
        match (quote, ch) {
            (Some(active), value) if value == active => quote = None,
            (Some(_), value) => current.push(value),
            (None, '\'' | '"') => quote = Some(ch),
            (None, value) if value.is_whitespace() => {
                if !current.is_empty() { tokens.push(std::mem::take(&mut current)); }
            }
            (None, value) => current.push(value),
        }
    }
    if quote.is_some() { return None }
    if !current.is_empty() { tokens.push(current); }
    Some(tokens)
}

fn is_supported_lang(lang: &str) -> bool {
    matches!(lang.to_lowercase().as_str(), "pl" | "en")
}

#[cfg(test)]
mod tests {
    use super::{parse_input, visible_suggestions, InputAction, SlashCommand};

    #[test]
    fn parses_translate_commands() {
        assert_eq!(
            parse_input("/translate en pl"),
            InputAction::Slash(SlashCommand::Translate {
                from: "en".to_string(),
                to: "pl".to_string(),
            })
        );
        assert_eq!(
            parse_input("/translate pl en"),
            InputAction::Slash(SlashCommand::Translate {
                from: "pl".to_string(),
                to: "en".to_string(),
            })
        );
        assert!(matches!(
            parse_input("/translate auto pl"),
            InputAction::Slash(SlashCommand::Translate { from, to }) if from == "auto" && to == "pl"
        ));
    }

    #[test]
    fn parses_chat_language_override() {
        assert_eq!(
            parse_input("/lang en"),
            InputAction::Slash(SlashCommand::Lang(Some("en".to_string())))
        );
        assert_eq!(
            parse_input("/lang pl"),
            InputAction::Slash(SlashCommand::Lang(Some("pl".to_string())))
        );
        assert_eq!(parse_input("/lang auto"), InputAction::Slash(SlashCommand::Lang(None)));
    }

    #[test]
    fn invalid_translate_is_incomplete() {
        assert!(matches!(
            parse_input("/translate de pl"),
            InputAction::IncompleteSlash(_)
        ));
    }

    #[test]
    fn parses_settings_and_utility_commands() {
        assert_eq!(parse_input("/chat"), InputAction::Slash(SlashCommand::Chat));
        assert_eq!(
            parse_input("/settings"),
            InputAction::Slash(SlashCommand::Settings)
        );
        assert_eq!(parse_input("/clear"), InputAction::Slash(SlashCommand::Clear(None)));
        assert_eq!(parse_input("/clear translation"), InputAction::Slash(SlashCommand::Clear(Some("translation".into()))));
        assert_eq!(parse_input("/clear all"), InputAction::Slash(SlashCommand::Clear(Some("all".into()))));
        assert_eq!(parse_input("/answer-lang source"), InputAction::Slash(SlashCommand::AnswerLanguage("source".into())));
        assert_eq!(parse_input("/quit"), InputAction::Slash(SlashCommand::Quit));
    }

    #[test]
    fn parses_facts_commands() {
        assert_eq!(
            parse_input("/facts New York --role subject --limit 10"),
            InputAction::Slash(SlashCommand::Facts(super::FactsCommand {
                selector: "New York".into(),
                role: crate::engine::DebugFactRole::Subject,
                limit: Some(10),
                page: 1,
            }))
        );
        assert_eq!(
            parse_input("/facts \"New York\" --all"),
            InputAction::Slash(SlashCommand::Facts(super::FactsCommand {
                selector: "New York".into(),
                role: crate::engine::DebugFactRole::Any,
                limit: None,
                page: 1,
            }))
        );
        assert!(matches!(parse_input("/facts Poland --all --limit 2"), InputAction::IncompleteSlash(_)));
        assert!(matches!(parse_input("/facts"), InputAction::IncompleteSlash(_)));
        assert_eq!(
            parse_input("/evidence 2"),
            InputAction::Slash(SlashCommand::Evidence("2".into()))
        );
        assert_eq!(
            parse_input("/debug trace"),
            InputAction::Slash(SlashCommand::Debug("trace".into()))
        );
    }

    #[test]
    fn suggestions_show_supported_actions() {
        let translate = visible_suggestions("/tr");
        assert!(translate.iter().any(|item| item.replacement == "/translate en pl"));
        let settings = visible_suggestions("/set");
        assert!(settings.is_empty());
        let utility = visible_suggestions("/exp");
        assert!(utility.is_empty());
        assert!(visible_suggestions("/fa").iter().any(|item| item.replacement == "/facts "));
        assert!(visible_suggestions("/ev").iter().any(|item| item.replacement == "/evidence "));
    }
}
