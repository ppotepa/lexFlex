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
    Settings,
    Ingest(String),
    Query(String),
    Inspect(String),
    Trace,
    Clear,
    Quit,
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
                [from, to] if is_supported_lang(from) && is_supported_lang(to) => {
                    InputAction::Slash(SlashCommand::Translate {
                        from: (*from).to_string(),
                        to: (*to).to_string(),
                    })
                }
                _ => InputAction::IncompleteSlash(SlashDraft { command, args }),
            }
        }
        "settings" => InputAction::Slash(SlashCommand::Settings),
        "ingest" => InputAction::Slash(SlashCommand::Ingest(args)),
        "query" => InputAction::Slash(SlashCommand::Query(args)),
        "inspect" => InputAction::Slash(SlashCommand::Inspect(args)),
        "trace" => InputAction::Slash(SlashCommand::Trace),
        "clear" => InputAction::Slash(SlashCommand::Clear),
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
    ]
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
        assert_eq!(parse_input("/clear"), InputAction::Slash(SlashCommand::Clear));
        assert_eq!(parse_input("/quit"), InputAction::Slash(SlashCommand::Quit));
    }

    #[test]
    fn suggestions_only_show_translation_actions() {
        let translate = visible_suggestions("/tr");
        assert!(translate.iter().any(|item| item.replacement == "/translate en pl"));
        let settings = visible_suggestions("/set");
        assert!(settings.is_empty());
        let utility = visible_suggestions("/exp");
        assert!(utility.is_empty());
    }
}
