use crate::document::knowledge::{DecimalValue, DocumentKnowledgeExtraction, KnowledgeValue};

use super::super::{
    AnswerStatus, QueryExecutionResult, QueryExecutionResultRow, QueryIntent, QueryInterlingua,
};

pub(super) fn answer_status(
    query: &QueryInterlingua,
    execution: &QueryExecutionResult,
) -> AnswerStatus {
    if execution.rows.is_empty()
        || (matches!(
            query.evidence_policy,
            crate::query::QueryEvidencePolicy::Required
        ) && execution.rows.iter().all(|row| row.evidence.is_empty()))
    {
        return AnswerStatus::Unknown;
    }
    match query.intent {
        QueryIntent::Count => AnswerStatus::Exact,
        QueryIntent::Boolean => boolean_status(execution),
        QueryIntent::Explain => AnswerStatus::Supported,
        QueryIntent::Contradiction if execution.rows.len() > 1 => AnswerStatus::Conflicting,
        QueryIntent::LookupEntity
        | QueryIntent::LookupProperty
        | QueryIntent::LookupRelation
        | QueryIntent::Temporal
        | QueryIntent::Causal
        | QueryIntent::Comparison
        | QueryIntent::Provenance => {
            if execution.rows.len() == 1 {
                AnswerStatus::Exact
            } else {
                AnswerStatus::Supported
            }
        }
        _ => AnswerStatus::Supported,
    }
}

pub(super) fn answer_text(
    query: &QueryInterlingua,
    execution: &QueryExecutionResult,
    status: AnswerStatus,
    knowledge: &DocumentKnowledgeExtraction,
) -> Option<String> {
    let text = match query.intent {
        QueryIntent::Count => execution
            .rows
            .first()
            .and_then(|row| row.columns.get("count"))
            .cloned(),
        QueryIntent::Boolean => Some(
            match status {
                AnswerStatus::Exact => "yes",
                AnswerStatus::No | AnswerStatus::Partial => "no",
                AnswerStatus::Conflicting => "conflicting",
                _ => "unknown",
            }
            .to_string(),
        ),
        QueryIntent::Explain => Some(format!("matched {}", execution.rows.len())),
        QueryIntent::Contradiction => Some(format!("conflicts {}", execution.rows.len())),
        QueryIntent::Provenance => Some(render_provenance_text(execution)),
        QueryIntent::LookupEntity
        | QueryIntent::LookupRelation
        | QueryIntent::LookupProperty
        | QueryIntent::Temporal
        | QueryIntent::Causal
        | QueryIntent::Comparison => render_lookup_text(execution, knowledge),
        _ => None,
    };
    text.or_else(|| Some(format!("{status:?}")))
}

fn boolean_status(execution: &QueryExecutionResult) -> AnswerStatus {
    let positive = execution.rows.iter().any(|row| {
        matches!(
            row.columns.get("status").map(String::as_str),
            Some("Supported" | "Contested" | "AttributedOnly")
        )
    });
    let negative = execution.rows.iter().any(|row| {
        matches!(
            row.columns.get("status").map(String::as_str),
            Some("Contradicted" | "NonFactual")
        )
    });
    match (positive, negative) {
        (true, true) => AnswerStatus::Conflicting,
        (true, false) => AnswerStatus::Exact,
        (false, true) => AnswerStatus::No,
        (false, false) => AnswerStatus::Unknown,
    }
}

fn render_lookup_text(
    execution: &QueryExecutionResult,
    knowledge: &DocumentKnowledgeExtraction,
) -> Option<String> {
    if execution.rows.is_empty() {
        return None;
    }
    let mut values = execution
        .rows
        .iter()
        .map(|row| render_row_value(row, knowledge))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    Some(values.join("; "))
}

fn render_provenance_text(execution: &QueryExecutionResult) -> String {
    let mut values = execution
        .rows
        .iter()
        .filter_map(|row| row.columns.get("claim").cloned())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    if values.is_empty() {
        "unknown".to_string()
    } else {
        values.join("; ")
    }
}

fn render_row_value(row: &QueryExecutionResultRow, knowledge: &DocumentKnowledgeExtraction) -> String {
    if let Some(object) = row.columns.get("object") {
        if let Some(value) = object.strip_prefix("value:") {
            return render_value_by_id(value, knowledge).unwrap_or_else(|| object.clone());
        }
        if !object.starts_with("unknown") {
            return object.clone();
        }
    }
    if let Some(subject) = row.columns.get("subject") {
        return subject.clone();
    }
    row.columns.get("summary").cloned().unwrap_or_default()
}

fn render_value_by_id(value_id: &str, knowledge: &DocumentKnowledgeExtraction) -> Option<String> {
    knowledge.values.iter().find_map(|(id, value)| {
        (id.to_string() == value_id).then(|| render_knowledge_value(value))
    })
}

fn render_knowledge_value(value: &KnowledgeValue) -> String {
    match value {
        KnowledgeValue::Integer(value) => value.to_string(),
        KnowledgeValue::Decimal(value) => render_decimal(value),
        KnowledgeValue::Range { minimum, maximum } => {
            let left = minimum.as_ref().map(render_decimal).unwrap_or_else(|| "?".into());
            let right = maximum.as_ref().map(render_decimal).unwrap_or_else(|| "?".into());
            format!("{left}..{right}")
        }
        KnowledgeValue::Approximate(inner) => {
            format!("approximately {}", render_knowledge_value(inner))
        }
        KnowledgeValue::Quantity(quantity) => {
            let amount = render_decimal(&quantity.amount);
            match &quantity.unit {
                Some(unit) => format!("{amount} {unit}"),
                None => amount,
            }
        }
        KnowledgeValue::Date(value) => match (value.month, value.day) {
            (Some(month), Some(day)) => format!("{:04}-{:02}-{:02}", value.year, month, day),
            (Some(month), None) => format!("{:04}-{:02}", value.year, month),
            (None, _) => format!("{:04}", value.year),
        },
        KnowledgeValue::Duration { days } => format!("{days} days"),
        KnowledgeValue::Frequency { times, period } => format!("{times}/{period}"),
        KnowledgeValue::Text(value) => value.clone(),
        KnowledgeValue::Boolean(value) => value.to_string(),
        KnowledgeValue::Unknown => "unknown".to_string(),
    }
}

fn render_decimal(value: &DecimalValue) -> String {
    let digits = value.mantissa.to_string();
    let sign = if value.sign < 0 { "-" } else { "" };
    if value.scale == 0 {
        return format!("{sign}{digits}");
    }
    let scale = value.scale as usize;
    if digits.len() <= scale {
        let padded = format!("{:0>width$}", digits, width = scale + 1);
        let split = padded.len() - scale;
        return format!("{sign}{}.{}", &padded[..split], &padded[split..]);
    }
    let split = digits.len() - scale;
    format!("{sign}{}.{}", &digits[..split], &digits[split..])
}
