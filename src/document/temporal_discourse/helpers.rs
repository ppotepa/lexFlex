use crate::document::compilation::DocumentCompilation;
use crate::document::resolution::DocumentEntityResolution;
use crate::document::DocumentSentence;
use std::collections::BTreeMap;

use super::id::{
    DiscourseRelationId, DocumentTemporalDiscourseIdFactory, EventCoreferenceClusterId,
    EventCoreferenceDecisionId, TemporalExpressionId,
    TemporalRelationId,
};
use super::model::{
    DiscourseRelation, DiscourseRelationKind, EventCoreferenceCluster, EventCoreferenceDecision,
    EventCoreferenceDecisionKind, EventProfile,
    TemporalClosureConflict, TemporalExpression, TemporalExpressionKind, TemporalNormalizedValue,
    TemporalRelation, TemporalRelationKind,
};

pub(crate) fn temporal_relation_for_sentence(
    temporal: Option<crate::core::interlingua::TemporalReference>,
    normalized: &TemporalNormalizedValue,
) -> TemporalRelationKind {
    match (temporal.as_ref(), normalized) {
        (Some(crate::core::interlingua::TemporalReference::Relative { offset_days, .. }), TemporalNormalizedValue::Relative { .. }) if *offset_days < 0 => TemporalRelationKind::Before,
        (Some(crate::core::interlingua::TemporalReference::Relative { offset_days, .. }), TemporalNormalizedValue::Relative { .. }) if *offset_days > 0 => TemporalRelationKind::After,
        (Some(crate::core::interlingua::TemporalReference::Duration { .. }), _) => TemporalRelationKind::Overlaps,
        (Some(crate::core::interlingua::TemporalReference::Absolute { .. }), _) => TemporalRelationKind::Simultaneous,
        _ => TemporalRelationKind::Unknown,
    }
}

pub(crate) fn temporal_expression_kind(normalized: &TemporalNormalizedValue) -> TemporalExpressionKind {
    match normalized {
        TemporalNormalizedValue::Absolute { .. } => TemporalExpressionKind::Absolute,
        TemporalNormalizedValue::Relative { .. } => TemporalExpressionKind::Relative,
        TemporalNormalizedValue::Duration { .. } => TemporalExpressionKind::Duration,
        TemporalNormalizedValue::Frequency { .. } => TemporalExpressionKind::Frequency,
        TemporalNormalizedValue::PartOfDay { .. } => TemporalExpressionKind::PartOfDay,
        TemporalNormalizedValue::Symbolic { .. } => TemporalExpressionKind::Unknown,
    }
}

pub(crate) fn normalize_temporal(
    temporal: Option<crate::core::interlingua::TemporalReference>,
    text: &str,
    reference_time: Option<&str>,
) -> TemporalNormalizedValue {
    match temporal {
        Some(crate::core::interlingua::TemporalReference::Absolute { timestamp }) => TemporalNormalizedValue::Absolute {
            iso_timestamp: Some(timestamp),
        },
        Some(crate::core::interlingua::TemporalReference::Relative { offset_days, anchor }) => TemporalNormalizedValue::Relative {
            offset_days,
            anchor: format!("{anchor:?}"),
        },
        Some(crate::core::interlingua::TemporalReference::Duration { days }) => {
            TemporalNormalizedValue::Duration { days }
        }
        Some(crate::core::interlingua::TemporalReference::Frequency { times, period }) => {
            TemporalNormalizedValue::Frequency { times, period }
        }
        Some(crate::core::interlingua::TemporalReference::Deictic { word }) => TemporalNormalizedValue::Symbolic {
            label: reference_time
                .map(|value| format!("deictic:{word}:anchored:{value}"))
                .unwrap_or_else(|| format!("deictic:{word}")),
        },
        None => TemporalNormalizedValue::Symbolic {
            label: reference_time
                .map(|value| format!("anchored:{value}:{text}"))
                .unwrap_or_else(|| format!("document-context:{text}")),
        },
    }
}

pub(crate) fn temporal_evidence(
    temporal: Option<crate::core::interlingua::TemporalReference>,
    text: &str,
    normalized: &TemporalNormalizedValue,
) -> Vec<String> {
    let mut evidence = vec![format!("temporal={temporal:?}")];
    evidence.push(format!("text={text}"));
    evidence.push(format!("normalized={normalized:?}"));
    evidence
}

pub(crate) fn detect_discourse_cue(text: &str) -> Option<&'static str> {
    let lower = text.trim_start().to_lowercase();
    let first = lower.split_whitespace().next().unwrap_or("");
    match first {
        "because" | "ponieważ" | "bo" => Some("cause"),
        "therefore" | "thereupon" | "więc" | "zatem" => Some("result"),
        "then" | "potem" | "następnie" => Some("sequence"),
        "however" | "jednak" | "ale" => Some("contrast"),
        _ => None,
    }
}

pub(crate) fn cue_relation(text: &str) -> Option<(DiscourseRelationKind, Option<String>, bool)> {
    let cue = detect_discourse_cue(text)?;
    let relation = match cue {
        "cause" => Some((DiscourseRelationKind::Cause, Some("cause->result".to_string()), false)),
        "result" => Some((DiscourseRelationKind::Result, Some("result->cause".to_string()), false)),
        "sequence" => Some((DiscourseRelationKind::Sequence, Some("left-to-right".to_string()), true)),
        "contrast" => Some((DiscourseRelationKind::Contrast, Some("contrast".to_string()), false)),
        _ => None,
    }?;
    Some(relation)
}

pub(crate) fn sentence_pair_relation(
    sentences: &[&DocumentSentence],
    index: usize,
    text: &str,
) -> Option<(TemporalRelationKind, Vec<String>)> {
    let previous = sentences.get(index.checked_sub(1)?);
    let previous_text = previous
        .map(|sentence| sentence.id.to_string())
        .unwrap_or_default();
    let lower_tokens: Vec<&str> = text
        .split(|ch: char| !ch.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect();
    let has_token = |needle: &str| lower_tokens.iter().any(|token| token.eq_ignore_ascii_case(needle));
    if has_token("before") || has_token("przed") {
        return Some((TemporalRelationKind::Before, vec![text.to_string(), previous_text]));
    }
    if has_token("after") || has_token("po") {
        return Some((TemporalRelationKind::After, vec![text.to_string(), previous_text]));
    }
    None
}

pub(crate) fn build_event_coreference(
    artifact_id: &str,
    event_profiles: &BTreeMap<crate::document::graph::GraphNodeId, EventProfile>,
    decisions: &mut BTreeMap<EventCoreferenceDecisionId, EventCoreferenceDecision>,
    decision_order: &mut Vec<EventCoreferenceDecisionId>,
    clusters: &mut BTreeMap<EventCoreferenceClusterId, EventCoreferenceCluster>,
    cluster_order: &mut Vec<EventCoreferenceClusterId>,
) {
    for profile in event_profiles.values() {
        let cluster_id = DocumentTemporalDiscourseIdFactory::event_cluster(artifact_id, cluster_order.len());
        cluster_order.push(cluster_id.clone());
        clusters.insert(
            cluster_id.clone(),
            EventCoreferenceCluster {
                id: cluster_id.clone(),
                representative_event_profile_id: profile.id.clone(),
                event_profile_ids: vec![profile.id.clone()],
                canonical_frame_type: profile.frame_type.clone(),
                canonical_verb_concept: profile.verb_concept.clone(),
                confidence_milli: 800,
            },
        );
        let decision_id =
            DocumentTemporalDiscourseIdFactory::event_decision(artifact_id, decision_order.len());
        decision_order.push(decision_id.clone());
        decisions.insert(
            decision_id.clone(),
            EventCoreferenceDecision {
                id: decision_id,
                event_profile_id: profile.id.clone(),
                selected_cluster: Some(cluster_id),
                kind: if profile.participant_count > 0 {
                    EventCoreferenceDecisionKind::Accepted
                } else {
                    EventCoreferenceDecisionKind::Seeded
                },
                score: if profile.participant_count > 0 { 500 } else { 0 },
                evidence: vec![format!("frame_type={}", profile.frame_type)],
            },
        );
    }
}

pub(crate) fn build_ordered_steps(
    resolution: Option<&DocumentEntityResolution>,
    temporal_expressions: &BTreeMap<TemporalExpressionId, TemporalExpression>,
    discourse_relations: &BTreeMap<DiscourseRelationId, DiscourseRelation>,
) -> Vec<String> {
    let mut steps = Vec::new();
    if let Some(resolution) = resolution {
        steps.push(format!("entity_clusters={}", resolution.cluster_order.len()));
    }
    steps.push(format!("temporal_expressions={}", temporal_expressions.len()));
    steps.push(format!("discourse_relations={}", discourse_relations.len()));
    steps
}

pub(crate) fn summarize_temporal_counts(
    compilation: &DocumentCompilation,
    event_profiles: &BTreeMap<crate::document::graph::GraphNodeId, EventProfile>,
    temporal_expressions: &BTreeMap<TemporalExpressionId, TemporalExpression>,
    temporal_relations: &BTreeMap<TemporalRelationId, TemporalRelation>,
    temporal_conflicts: &[TemporalClosureConflict],
    event_coreference_decisions: &BTreeMap<EventCoreferenceDecisionId, EventCoreferenceDecision>,
    event_coreference_clusters: &BTreeMap<EventCoreferenceClusterId, EventCoreferenceCluster>,
    discourse_relations: &BTreeMap<DiscourseRelationId, DiscourseRelation>,
    diagnostics: &[super::model::DocumentTemporalDiscourseDiagnostic],
) -> super::model::DocumentTemporalDiscourseSummary {
    super::model::DocumentTemporalDiscourseSummary {
        sentences_total: compilation.document.sentences.len(),
        event_profiles_total: event_profiles.len(),
        temporal_expressions_total: temporal_expressions.len(),
        temporal_relations_total: temporal_relations.len(),
        temporal_conflicts_total: temporal_conflicts.len(),
        event_decisions_total: event_coreference_decisions.len(),
        event_clusters_total: event_coreference_clusters.len(),
        discourse_relations_total: discourse_relations.len(),
        diagnostics_info: diagnostics
            .iter()
            .filter(|diagnostic| matches!(diagnostic.severity, super::model::DocumentTemporalDiscourseDiagnosticSeverity::Info))
            .count(),
        diagnostics_warning: diagnostics
            .iter()
            .filter(|diagnostic| matches!(diagnostic.severity, super::model::DocumentTemporalDiscourseDiagnosticSeverity::Warning))
            .count(),
        diagnostics_error: diagnostics
            .iter()
            .filter(|diagnostic| matches!(diagnostic.severity, super::model::DocumentTemporalDiscourseDiagnosticSeverity::Error))
            .count(),
        diagnostics_fatal: diagnostics
            .iter()
            .filter(|diagnostic| matches!(diagnostic.severity, super::model::DocumentTemporalDiscourseDiagnosticSeverity::Fatal))
            .count(),
    }
}
