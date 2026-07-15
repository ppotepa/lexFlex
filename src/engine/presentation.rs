use std::collections::BTreeMap;

use crate::query::{AnswerStatus, DocumentAnswer, QueryExecutionResultRow};

use super::{
    DebugCard, DebugPresentation, EngineError, EnginePresentation, EnginePresentationBlock,
    EnginePresentationSection, EnginePresentationTone, EngineStatus, ResponseMeta,
};

pub(crate) fn conversation(
    meta: &ResponseMeta,
    text: Option<&str>,
    answer: Option<&DocumentAnswer>,
) -> EnginePresentation {
    let mut sections = Vec::new();
    sections.push(EnginePresentationSection {
        title: "Answer".into(),
        tone: tone_from_status(meta.status),
        blocks: vec![EnginePresentationBlock::Text(
            text.map(str::to_string)
                .or_else(|| answer.and_then(|value| value.text.clone()))
                .unwrap_or_else(|| humanize_status(meta.status)),
        )],
    });
    if let Some(answer) = answer {
        if !answer.rows.is_empty() {
            sections.push(EnginePresentationSection {
                title: "Rows".into(),
                tone: EnginePresentationTone::Info,
                blocks: vec![EnginePresentationBlock::BulletList(
                    answer.rows.iter().take(8).map(humanize_answer_row).collect(),
                )],
            });
        }
        if !answer.conflicts.is_empty() {
            sections.push(EnginePresentationSection {
                title: "Conflicts".into(),
                tone: EnginePresentationTone::Warning,
                blocks: vec![EnginePresentationBlock::BulletList(answer.conflicts.clone())],
            });
        }
    }
    let mut hints = Vec::new();
    if meta.status == EngineStatus::Unknown {
        hints.push("No evidence-backed answer was produced for this turn.".into());
    }
    EnginePresentation {
        title: "Conversation".into(),
        summary: text
            .map(str::to_string)
            .or_else(|| answer.and_then(|value| value.text.clone()))
            .unwrap_or_else(|| humanize_status(meta.status)),
        status_line: humanize_status(meta.status),
        tone: tone_from_status(meta.status),
        sections,
        hints,
    }
}

pub(crate) fn translation(
    meta: &ResponseMeta,
    text: Option<&str>,
    source_language: &str,
    target_language: &str,
    knowledge_committed: bool,
    translation_snapshot_id: &str,
) -> EnginePresentation {
    EnginePresentation {
        title: "Translation".into(),
        summary: text
            .map(str::to_string)
            .unwrap_or_else(|| "No surface translation was produced.".into()),
        status_line: humanize_status(meta.status),
        tone: tone_from_status(meta.status),
        sections: vec![
            EnginePresentationSection {
                title: "Output".into(),
                tone: tone_from_status(meta.status),
                blocks: vec![EnginePresentationBlock::Text(
                    text.map(str::to_string)
                        .unwrap_or_else(|| "No surface translation was produced.".into()),
                )],
            },
            EnginePresentationSection {
                title: "Context".into(),
                tone: EnginePresentationTone::Info,
                blocks: vec![EnginePresentationBlock::Fields(vec![
                    ("direction".into(), format!("{source_language} -> {target_language}")),
                    (
                        "knowledge".into(),
                        if knowledge_committed {
                            "session updated".into()
                        } else {
                            "session unchanged".into()
                        },
                    ),
                    ("translation_snapshot".into(), shorten_id(translation_snapshot_id)),
                ])],
            },
        ],
        hints: Vec::new(),
    }
}

pub(crate) fn ingest(
    meta: &ResponseMeta,
    source_id: &str,
    bundle_id: &str,
    source_sha256: &str,
    bundle_sha256: &str,
) -> EnginePresentation {
    EnginePresentation {
        title: "Ingest".into(),
        summary: format!("Source {source_id} was ingested into {bundle_id}."),
        status_line: humanize_status(meta.status),
        tone: tone_from_status(meta.status),
        sections: vec![
            EnginePresentationSection {
                title: "Result".into(),
                tone: EnginePresentationTone::Success,
                blocks: vec![EnginePresentationBlock::Text(format!(
                    "Source {source_id} was ingested into bundle {bundle_id}."
                ))],
            },
            EnginePresentationSection {
                title: "Artifacts".into(),
                tone: EnginePresentationTone::Info,
                blocks: vec![EnginePresentationBlock::Fields(vec![
                    ("source_sha256".into(), shorten_id(source_sha256)),
                    ("bundle_sha256".into(), shorten_id(bundle_sha256)),
                ])],
            },
        ],
        hints: Vec::new(),
    }
}

pub(crate) fn inspection(
    meta: &ResponseMeta,
    title: &str,
    summary: &str,
    values: &BTreeMap<String, String>,
) -> EnginePresentation {
    EnginePresentation {
        title: title.into(),
        summary: summary.into(),
        status_line: humanize_status(meta.status),
        tone: tone_from_status(meta.status),
        sections: vec![EnginePresentationSection {
            title: "Fields".into(),
            tone: EnginePresentationTone::Info,
            blocks: vec![EnginePresentationBlock::Fields(
                values
                    .iter()
                    .map(|(key, value)| {
                        let rendered = if key.contains("sha256") || key.ends_with("_id") {
                            shorten_id(value)
                        } else {
                            value.clone()
                        };
                        (key.replace('_', " "), rendered)
                    })
                    .collect(),
            )],
        }],
        hints: Vec::new(),
    }
}

pub(crate) fn answer(meta: &ResponseMeta, answer: &DocumentAnswer) -> EnginePresentation {
    let mut sections = vec![EnginePresentationSection {
        title: "Answer".into(),
        tone: tone_from_answer_status(answer.status),
        blocks: vec![EnginePresentationBlock::Text(
            answer
                .text
                .clone()
                .unwrap_or_else(|| format!("Answer status: {:?}", answer.status)),
        )],
    }];
    if !answer.rows.is_empty() {
        sections.push(EnginePresentationSection {
            title: "Rows".into(),
            tone: EnginePresentationTone::Info,
            blocks: vec![EnginePresentationBlock::BulletList(
                answer.rows.iter().take(8).map(humanize_answer_row).collect(),
            )],
        });
    }
    if !answer.evidence.is_empty() {
        sections.push(EnginePresentationSection {
            title: "Evidence".into(),
            tone: EnginePresentationTone::Info,
            blocks: vec![EnginePresentationBlock::BulletList(answer.evidence.clone())],
        });
    }
    if !answer.conflicts.is_empty() {
        sections.push(EnginePresentationSection {
            title: "Conflicts".into(),
            tone: EnginePresentationTone::Warning,
            blocks: vec![EnginePresentationBlock::BulletList(answer.conflicts.clone())],
        });
    }
    EnginePresentation {
        title: "Answer".into(),
        summary: answer
            .text
            .clone()
            .unwrap_or_else(|| format!("Answer status: {:?}", answer.status)),
        status_line: humanize_status(meta.status),
        tone: tone_from_status(meta.status),
        sections,
        hints: Vec::new(),
    }
}

pub(crate) fn error(meta: &ResponseMeta, error: &EngineError) -> EnginePresentation {
    let mut blocks = vec![EnginePresentationBlock::Text(humanize_error(error))];
    if !meta.diagnostics.is_empty() {
        blocks.push(EnginePresentationBlock::BulletList(
            meta.diagnostics
                .iter()
                .map(|value| format!("Diagnostic: {value}"))
                .collect(),
        ));
    }
    EnginePresentation {
        title: "Error".into(),
        summary: humanize_error(error),
        status_line: humanize_status(meta.status),
        tone: EnginePresentationTone::Error,
        sections: vec![EnginePresentationSection {
            title: "Problem".into(),
            tone: EnginePresentationTone::Error,
            blocks,
        }],
        hints: Vec::new(),
    }
}

pub(crate) fn from_debug(value: &DebugPresentation) -> EnginePresentation {
    let sections = value
        .cards
        .iter()
        .map(debug_card_to_section)
        .collect::<Vec<_>>();
    EnginePresentation {
        title: value.title.clone(),
        summary: value.summary.clone(),
        status_line: value.status_line.clone(),
        tone: EnginePresentationTone::Info,
        sections,
        hints: value.hints.clone(),
    }
}

fn debug_card_to_section(card: &DebugCard) -> EnginePresentationSection {
    let mut blocks = Vec::new();
    if !card.body.is_empty() {
        blocks.push(EnginePresentationBlock::Text(card.body.join("\n")));
    }
    if !card.fields.is_empty() {
        blocks.push(EnginePresentationBlock::Fields(
            card.fields
                .iter()
                .filter(|(key, _)| !matches!(key.as_str(), "claim_id" | "bundle_id"))
                .map(|(key, value)| {
                    let rendered = if key.contains("sha256") || key.ends_with("_id") {
                        shorten_id(value)
                    } else {
                        value.clone()
                    };
                    (key.clone(), rendered)
                })
                .collect(),
        ));
    }
    if !card.evidence.is_empty() {
        blocks.push(EnginePresentationBlock::BulletList(card.evidence.clone()));
    }
    if !card.artifact_refs.is_empty() {
        blocks.push(EnginePresentationBlock::TechnicalRefs(
            card.artifact_refs.iter().map(|value| shorten_id(value)).collect(),
        ));
    }
    if let Some(status) = &card.status {
        blocks.push(EnginePresentationBlock::Notice {
            tone: tone_from_card_status(status),
            message: status.clone(),
        });
    }
    EnginePresentationSection {
        title: card.title.clone(),
        tone: card
            .status
            .as_deref()
            .map(tone_from_card_status)
            .unwrap_or(EnginePresentationTone::Info),
        blocks,
    }
}

pub(crate) fn humanize_status(status: EngineStatus) -> String {
    match status {
        EngineStatus::Ok => "Completed successfully.".into(),
        EngineStatus::Unknown => "The engine could not produce an evidence-backed answer.".into(),
        EngineStatus::Unsupported => "This request is not supported by the current engine path.".into(),
        EngineStatus::Error => "The engine failed while processing the request.".into(),
    }
}

pub(crate) fn humanize_error(error: &EngineError) -> String {
    match error {
        EngineError::InvalidRequest(value) => format!("Invalid request: {value}"),
        EngineError::SourceUnavailable(value) => format!("Source unavailable: {value}"),
        EngineError::Source(value) => format!("Source error: {value}"),
        EngineError::Pipeline(value) => format!("Pipeline error: {value}"),
        EngineError::Query(value) => format!("Query error: {value}"),
        EngineError::Persistence(value) => format!("Persistence error: {value}"),
    }
}

fn tone_from_status(status: EngineStatus) -> EnginePresentationTone {
    match status {
        EngineStatus::Ok => EnginePresentationTone::Success,
        EngineStatus::Unknown => EnginePresentationTone::Unknown,
        EngineStatus::Unsupported => EnginePresentationTone::Warning,
        EngineStatus::Error => EnginePresentationTone::Error,
    }
}

fn tone_from_answer_status(status: AnswerStatus) -> EnginePresentationTone {
    match status {
        AnswerStatus::Exact | AnswerStatus::Supported => EnginePresentationTone::Success,
        AnswerStatus::Partial | AnswerStatus::Ambiguous => EnginePresentationTone::Warning,
        AnswerStatus::Conflicting => EnginePresentationTone::Warning,
        AnswerStatus::Unknown | AnswerStatus::Unsupported | AnswerStatus::InvalidQuery | AnswerStatus::No => {
            EnginePresentationTone::Unknown
        }
    }
}

fn tone_from_card_status(status: &str) -> EnginePresentationTone {
    match status {
        "Supported" | "Ok" => EnginePresentationTone::Success,
        "Contested" | "Warning" => EnginePresentationTone::Warning,
        "Error" => EnginePresentationTone::Error,
        _ => EnginePresentationTone::Info,
    }
}

pub(crate) fn humanize_answer_row(row: &QueryExecutionResultRow) -> String {
    if row.columns.is_empty() {
        return "Empty answer row".into();
    }
    row.columns
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn shorten_id(value: &str) -> String {
    if value.len() <= 24 {
        value.to_string()
    } else {
        format!("{}..{}", &value[..10], &value[value.len().saturating_sub(10)..])
    }
}
