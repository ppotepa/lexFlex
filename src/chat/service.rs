use crate::api::LexFlexAPI;
use crate::chat::session::ChatContext;
use crate::chat::settings::{ChatSettings, Verbosity};
use crate::chat::transcript::MessageBlock;
use crate::chat::TraceMode;
use lexflex_learner::{DeductionResult, LexicalDeductionService, Language as LearnerLanguage};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub title: String,
    pub blocks: Vec<MessageBlock>,
    pub notes: Vec<String>,
    pub latency_ms: u128,
}

#[derive(Debug, Clone)]
pub struct WikipediaArticle {
    pub title: String,
    pub text: String,
    pub url: String,
}

#[derive(Debug)]
pub enum ChatJobOutput {
    Response(ChatResponse),
    WikipediaArticle {
        question: String,
        language: String,
        article: WikipediaArticle,
    },
}

#[derive(Debug)]
pub struct ChatServiceError(pub String);

impl Display for ChatServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ChatServiceError {}

pub struct ChatService {
    api: LexFlexAPI,
    offline: bool,
    trace_mode: TraceMode,
}

#[derive(Debug)]
pub enum ChatJob {
    Chat {
        input: String,
        context: ChatContext,
        history: Vec<ChatTurn>,
    },
    WikipediaLookup {
        question: String,
        language: String,
        title: String,
    },
    Translate {
        input: String,
        context: ChatContext,
        settings: ChatSettings,
    },
    Explain {
        input: String,
        context: ChatContext,
    },
    Parse {
        input: String,
        context: ChatContext,
    },
    Learn {
        input: String,
        context: ChatContext,
    },
}

#[derive(Debug, Clone)]
pub struct ChatTurn {
    pub role: String,
    pub content: String,
}

#[derive(Debug)]
pub struct ChatJobResult {
    pub label: String,
    pub result: Result<ChatJobOutput, String>,
}

pub struct ChatWorker {
    sender: Sender<ChatJob>,
    receiver: Receiver<ChatJobResult>,
}

impl ChatWorker {
    pub fn spawn(service: ChatService) -> Self {
        let (sender, jobs) = mpsc::channel();
        let (results, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            while let Ok(job) = jobs.recv() {
                let (label, result) = match job {
                    ChatJob::Chat {
                        input,
                        context,
                        history,
                    } => (
                        "chat".to_string(),
                        service.chat_message(&input, &context, &history)
                            .map(ChatJobOutput::Response)
                            .map_err(|err| err.to_string()),
                    ),
                    ChatJob::WikipediaLookup {
                        question,
                        language,
                        title,
                    } => (
                        "wikipedia".to_string(),
                        service
                            .fetch_wikipedia_article(&title, &language)
                            .map(|article| ChatJobOutput::WikipediaArticle {
                                question,
                                language,
                                article,
                            })
                            .map_err(|err| err.to_string()),
                    ),
                    ChatJob::Translate {
                        input,
                        context,
                        settings,
                    } => (
                        "translate".to_string(),
                        service
                            .translate_message(&input, &context, &settings)
                            .map(ChatJobOutput::Response)
                            .map_err(|err| err.to_string()),
                    ),
                    ChatJob::Explain { input, context } => (
                        "explain".to_string(),
                        service.explain_message(&input, &context)
                            .map(ChatJobOutput::Response)
                            .map_err(|err| err.to_string()),
                    ),
                    ChatJob::Parse { input, context } => (
                        "parse".to_string(),
                        service.parse_message(&input, &context)
                            .map(ChatJobOutput::Response)
                            .map_err(|err| err.to_string()),
                    ),
                    ChatJob::Learn { input, context } => (
                        "learn".to_string(),
                        service.learn_word(&input, &context)
                            .map(ChatJobOutput::Response)
                            .map_err(|err| err.to_string()),
                    ),
                };
                if results.send(ChatJobResult { label, result }).is_err() {
                    break;
                }
            }
        });
        Self { sender, receiver }
    }

    pub fn submit(&self, job: ChatJob) -> Result<(), ChatServiceError> {
        self.sender
            .send(job)
            .map_err(|_| ChatServiceError("chat backend worker stopped".to_string()))
    }

    pub fn try_receive(&self) -> Option<ChatJobResult> {
        match self.receiver.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => None,
        }
    }
}

impl ChatService {
    pub fn new(data_dir: &str, offline: bool, trace_mode: TraceMode) -> Result<Self, ChatServiceError> {
        let api = LexFlexAPI::builder()
            .data_dir(data_dir)
            .build()
            .map_err(|err| ChatServiceError(format!("Failed to build lexFlex backend: {}", err)))?;
        Ok(Self {
            api,
            offline,
            trace_mode,
        })
    }

    pub fn translate_message(
        &self,
        input: &str,
        context: &ChatContext,
        settings: &ChatSettings,
    ) -> Result<ChatResponse, ChatServiceError> {
        let started = Instant::now();
        let translation = self
            .api
            .translate(input, &context.source_lang, &context.target_lang)
            .map_err(|err| ChatServiceError(err.to_string()))?;
        let digest = if matches!(settings.verbosity, Verbosity::Detailed) {
            Some(
                self.api
                    .explain_human(input, &context.source_lang)
                    .map_err(|err| ChatServiceError(format!("Digest error: {}", err)))?,
            )
        } else {
            None
        };
        let parse_dump = if matches!(settings.verbosity, Verbosity::Detailed)
            && matches!(self.trace_mode, TraceMode::Full)
        {
            self.api
                .parse(input, &context.source_lang)
                .ok()
                .map(|il| format!("{:#?}", il))
        } else {
            None
        };

        Ok(shape_translation_response(
            translation,
            digest,
            parse_dump,
            context,
            settings,
            self.trace_mode,
            started.elapsed().as_millis(),
        ))
    }

    pub fn chat_message(
        &self,
        input: &str,
        context: &ChatContext,
        history: &[ChatTurn],
    ) -> Result<ChatResponse, ChatServiceError> {
        let base_url = std::env::var("LEXFLEX_LLM_BASE_URL").map_err(|_| {
            ChatServiceError(
                "chat backend is not configured; set LEXFLEX_LLM_BASE_URL or use /translate"
                    .to_string(),
            )
        })?;
        let model = std::env::var("LEXFLEX_LLM_MODEL")
            .unwrap_or_else(|_| "gemma2:2b".to_string());
        let system_prompt = std::env::var("LEXFLEX_LLM_SYSTEM_PROMPT").unwrap_or_else(|_| {
            format!(
                "You are lexFlex chat. Answer naturally in {}. Use the conversation context. Do not invent facts when the user asks about the imported knowledge; say when information is unavailable.",
                context.source_lang
            )
        });
        let mut messages = vec![serde_json::json!({
            "role": "system",
            "content": system_prompt,
        })];
        messages.extend(history.iter().map(|turn| {
            serde_json::json!({ "role": turn.role, "content": turn.content })
        }));
        messages.push(serde_json::json!({ "role": "user", "content": input }));

        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": 0.2,
            "stream": false,
        });
        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|err| ChatServiceError(format!("failed to create chat runtime: {err}")))?;
        let started = Instant::now();
        let content = runtime.block_on(async {
            let response = reqwest::Client::new()
                .post(url)
                .json(&body)
                .send()
                .await
                .map_err(|err| ChatServiceError(format!("chat request failed: {err}")))?
                .error_for_status()
                .map_err(|err| ChatServiceError(format!("chat backend returned an error: {err}")))?;
            let payload: serde_json::Value = response
                .json()
                .await
                .map_err(|err| ChatServiceError(format!("invalid chat response: {err}")))?;
            payload["choices"][0]["message"]["content"]
                .as_str()
                .filter(|value| !value.trim().is_empty())
                .map(str::to_string)
                .ok_or_else(|| ChatServiceError("chat backend returned no answer".to_string()))
        })?;
        Ok(ChatResponse {
            title: "chat".to_string(),
            blocks: vec![MessageBlock::Paragraph(content)],
            notes: vec![],
            latency_ms: started.elapsed().as_millis(),
        })
    }

    pub fn fetch_wikipedia_article(
        &self,
        title: &str,
        language: &str,
    ) -> Result<WikipediaArticle, ChatServiceError> {
        if self.offline {
            return Err(ChatServiceError(
                "Wikipedia lookup is disabled in offline mode; use /wiki <title> :: <text> to import a snapshot".to_string(),
            ));
        }
        let lang = match language.to_lowercase().as_str() {
            "pl" | "en" => language.to_lowercase(),
            _ => "en".to_string(),
        };
        let encoded_title = urlencoding::encode(title);
        let url = format!(
            "https://{}.wikipedia.org/w/api.php?action=query&generator=search&gsrsearch={}&gsrlimit=1&prop=extracts&explaintext=1&exlimit=1&redirects=1&format=json",
            lang, encoded_title
        );
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|err| ChatServiceError(format!("failed to create Wikipedia runtime: {err}")))?;
        runtime.block_on(async {
            let response = reqwest::Client::new()
                .get(&url)
                .header("User-Agent", "lexflex/0.1 conversation-knowledge")
                .send()
                .await
                .map_err(|err| ChatServiceError(format!("Wikipedia request failed: {err}")))?
                .error_for_status()
                .map_err(|err| ChatServiceError(format!("Wikipedia returned an error: {err}")))?;
            let payload: serde_json::Value = response
                .json()
                .await
                .map_err(|err| ChatServiceError(format!("invalid Wikipedia response: {err}")))?;
            let pages = payload["query"]["pages"]
                .as_object()
                .ok_or_else(|| ChatServiceError(format!("Wikipedia article not found: {title}")))?;
            let page = pages
                .values()
                .next()
                .ok_or_else(|| ChatServiceError(format!("Wikipedia article not found: {title}")))?;
            let extract = page["extract"]
                .as_str()
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| ChatServiceError(format!("Wikipedia article not found: {title}")))?;
            let resolved_title = page["title"]
                .as_str()
                .unwrap_or(title)
                .to_string();
            let article_url = format!(
                "https://{}.wikipedia.org/wiki/{}",
                lang,
                urlencoding::encode(&resolved_title).replace("%20", "_")
            );
            Ok(WikipediaArticle {
                title: resolved_title,
                text: extract.to_string(),
                url: article_url,
            })
        })
    }

    pub fn explain_message(
        &self,
        text: &str,
        context: &ChatContext,
    ) -> Result<ChatResponse, ChatServiceError> {
        let started = Instant::now();
        let digest = self
            .api
            .explain_human(text, &context.source_lang)
            .map_err(|err| ChatServiceError(format!("Explain error: {}", err)))?;
        Ok(ChatResponse {
            title: "explain".to_string(),
            blocks: vec![MessageBlock::Paragraph(digest)],
            notes: vec![],
            latency_ms: started.elapsed().as_millis(),
        })
    }

    pub fn parse_message(
        &self,
        text: &str,
        context: &ChatContext,
    ) -> Result<ChatResponse, ChatServiceError> {
        let started = Instant::now();
        let parsed = self
            .api
            .parse(text, &context.source_lang)
            .map_err(|err| ChatServiceError(format!("Parse error: {}", err)))?;
        Ok(ChatResponse {
            title: "parse".to_string(),
            blocks: vec![MessageBlock::Paragraph(format!("{:#?}", parsed))],
            notes: vec![],
            latency_ms: started.elapsed().as_millis(),
        })
    }

    pub fn learn_word(
        &self,
        text: &str,
        context: &ChatContext,
    ) -> Result<ChatResponse, ChatServiceError> {
        let started = Instant::now();
        let language =
            LearnerLanguage::from_str(&context.source_lang).unwrap_or(LearnerLanguage::Pl);
        let service = if self.offline {
            LexicalDeductionService::new_offline()
        } else {
            LexicalDeductionService::new()
        };
        let rt = tokio::runtime::Runtime::new()
            .map_err(|err| ChatServiceError(format!("Failed to create async runtime: {}", err)))?;
        let result = rt
            .block_on(async { service.deduce(text, language, Some(text)).await })
            .map_err(|err| ChatServiceError(format!("Learn error: {}", err)))?;
        Ok(ChatResponse {
            title: "learn".to_string(),
            blocks: vec![MessageBlock::Paragraph(format_deduction_result(&result))],
            notes: vec![],
            latency_ms: started.elapsed().as_millis(),
        })
    }
}

fn shape_translation_response(
    translation: String,
    digest: Option<String>,
    parse_dump: Option<String>,
    context: &ChatContext,
    settings: &ChatSettings,
    trace_mode: TraceMode,
    latency_ms: u128,
) -> ChatResponse {
    let mut blocks = vec![MessageBlock::Paragraph(translation)];
    let mut notes = vec![];

    match settings.verbosity {
        Verbosity::Compact => {}
        Verbosity::Normal => {
            blocks.push(MessageBlock::KeyValue {
                key: "direction".to_string(),
                value: format!("{} -> {}", context.source_lang, context.target_lang),
            });
        }
        Verbosity::Detailed => {
            blocks.push(MessageBlock::KeyValue {
                key: "direction".to_string(),
                value: format!("{} -> {}", context.source_lang, context.target_lang),
            });
            if let Some(digest) = digest {
                blocks.push(MessageBlock::Paragraph(digest));
            }
            if let Some(parse_dump) = parse_dump {
                blocks.push(MessageBlock::Paragraph(parse_dump));
            }
            notes.push(format!("trace mode: {}", trace_mode.as_str()));
            notes.push(format!("latency: {} ms", latency_ms));
        }
    }

    ChatResponse {
        title: "assistant".to_string(),
        blocks,
        notes,
        latency_ms,
    }
}

fn format_deduction_result(result: &DeductionResult) -> String {
    let mut out = String::new();
    out.push_str(&format!("word: {}\nlang: {}\n", result.word, result.lang));
    if let Some(lemma) = &result.surface.lemma {
        out.push_str(&format!("lemma: {}\n", lemma));
    }
    if let Some(pos) = &result.surface.pos {
        out.push_str(&format!("pos: {}\n", pos));
    }
    if let Some(best) = &result.best_concept {
        out.push_str(&format!(
            "best concept: {} ({:.2})\nreason: {}\n",
            best.concept_id, best.confidence, best.reason
        ));
    }
    if let Some(prop) = &result.concept_proposal {
        out.push_str(&format!(
            "\nproposal: {}\nconfidence: {:.2}\nreason: {}\n",
            prop.concept_id, prop.confidence, prop.reason
        ));
    }
    if !result.lexicon_proposals.is_empty() {
        out.push_str("\nlexicon proposals:\n");
        for proposal in &result.lexicon_proposals {
            out.push_str(&format!(
                " - [{}] {} => {}\n",
                proposal.lang, proposal.key, proposal.concept_id
            ));
        }
    }
    if !result.sources_used.is_empty() {
        out.push_str(&format!("\nsources: {}\n", result.sources_used.join(", ")));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::shape_translation_response;
    use crate::chat::session::{ChatContext, ChatMode};
    use crate::chat::settings::{ChatSettings, Verbosity};

    #[test]
    fn detailed_shape_includes_extra_blocks() {
        let context = ChatContext {
            source_lang: "pl".to_string(),
            target_lang: "en".to_string(),
            mode: ChatMode::Translate,
            chat_language_override: None,
        };
        let settings = ChatSettings {
            verbosity: Verbosity::Detailed,
        };
        let response = shape_translation_response(
            "hello".to_string(),
            Some("digest".to_string()),
            None,
            &context,
            &settings,
            crate::chat::TraceMode::Brief,
            42,
        );
        assert_eq!(response.blocks.len(), 3);
        assert_eq!(response.notes.len(), 2);
    }
}
