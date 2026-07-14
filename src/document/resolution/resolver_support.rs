use crate::core::interlingua::{Definiteness, Reference};
use crate::document::graph::{DocumentGraph, DocumentMentionKind, MentionAnchor};
use crate::document::id::ParagraphId;
use std::collections::BTreeMap;

use super::id::ResolutionMentionRef;
use super::model::{
    EntityResolutionDecision, EntityResolutionDecisionKind,
    EntityResolutionStage, MentionResolutionProfile, MentionSourceForm, ResolutionAlternative,
    ResolutionAlternativeKind, ResolvedEntityCluster,
};
use super::options::DocumentEntityResolutionOptions;
use super::summary::EntityResolutionSummary;

pub(crate) fn build_profile(
    graph: &DocumentGraph,
    mention_ref: ResolutionMentionRef,
    mention: &crate::document::graph::DocumentMentionNode,
) -> MentionResolutionProfile {
    let source_form = classify_source_form(mention);
    let is_proper_name = matches!(source_form, MentionSourceForm::ProperName);
    let is_pronoun = matches!(
        source_form,
        MentionSourceForm::Pronoun
            | MentionSourceForm::ReflexivePronoun
            | MentionSourceForm::PossessivePronoun
    );
    let is_reflexive = matches!(source_form, MentionSourceForm::ReflexivePronoun);
    let is_modifier = matches!(mention.mention_kind, DocumentMentionKind::Modifier);
    let is_group = matches!(mention.mention_kind, DocumentMentionKind::CoordinationGroup);
    MentionResolutionProfile {
        mention: mention_ref,
        graph_mention_id: Some(mention.id.clone()),
        source_sentence_id: mention.source_sentence_id.clone(),
        paragraph_id: graph
            .source_sentence_node(&mention.source_sentence_id)
            .map(|node| node.paragraph_id.clone())
            .unwrap_or_else(|| {
                ParagraphId::new(mention.source_sentence_id.as_str())
                    .expect("sentence id is a valid paragraph fallback")
            }),
        source_sentence_ordinal: graph
            .source_sentence_node(&mention.source_sentence_id)
            .map(|node| node.document_ordinal)
            .unwrap_or(0),
        paragraph_ordinal: graph
            .source_sentence_node(&mention.source_sentence_id)
            .map(|node| node.paragraph_ordinal)
            .unwrap_or(0),
        semantic_sentence_id: mention.semantic_sentence_id.clone(),
        frame_occurrence_id: mention.frame_occurrence_id.clone(),
        mention_kind: mention.mention_kind,
        source_form,
        exact_surface: match &mention.anchor {
            MentionAnchor::Exact { .. } | MentionAnchor::Discontinuous { .. } => mention.name.clone(),
            MentionAnchor::SentenceScoped { .. } | MentionAnchor::Unknown => None,
        },
        normalized_surface: mention.normalized_name.clone(),
        concept: mention.concept.clone(),
        features: mention.features.clone(),
        reference: mention.reference.clone(),
        anchor: mention.anchor.clone(),
        semantic_entity_id: mention.semantic_entity_id,
        role_context: mention.role_context,
        role_ordinal: mention.role_ordinal,
        is_subject_like: is_subject_like(mention.role_context),
        is_group,
        is_modifier,
        is_pronoun,
        is_proper_name,
        is_reflexive,
        is_zero_subject: matches!(source_form, MentionSourceForm::ZeroSubject),
    }
}

pub(crate) fn choose_resolution(
    profile: &MentionResolutionProfile,
    accepted_mentions: &[ResolutionMentionRef],
    profiles: &BTreeMap<ResolutionMentionRef, MentionResolutionProfile>,
    options: DocumentEntityResolutionOptions,
) -> (
    EntityResolutionStage,
    EntityResolutionDecisionKind,
    Option<ResolutionMentionRef>,
    i32,
    Vec<ResolutionAlternative>,
    Vec<String>,
    Vec<String>,
) {
    let mut alternatives = Vec::new();
    let mut evidence = Vec::new();
    let mut rejections = Vec::new();
    let mut scored: Vec<(ResolutionMentionRef, i32, ResolutionAlternativeKind, String)> = Vec::new();

    for target in accepted_mentions {
        let Some(target_profile) = profiles.get(target) else {
            continue;
        };
        let sentence_distance = profile
            .source_sentence_ordinal
            .abs_diff(target_profile.source_sentence_ordinal);
        if sentence_distance > options.max_sentence_distance {
            rejections.push(format!("{target}:sentence_distance_exceeded"));
            continue;
        }
        let paragraph_distance = profile
            .paragraph_ordinal
            .abs_diff(target_profile.paragraph_ordinal);
        if paragraph_distance > options.max_paragraph_distance {
            rejections.push(format!("{target}:paragraph_distance_exceeded"));
            continue;
        }
        if profile.paragraph_id != target_profile.paragraph_id {
            if (profile.is_pronoun || target_profile.is_pronoun)
                && !options.allow_cross_paragraph_pronouns
            {
                rejections.push(format!("{target}:cross_paragraph_pronoun"));
                continue;
            }
            if (profile.is_proper_name || target_profile.is_proper_name)
                && !options.allow_cross_paragraph_proper_names
            {
                rejections.push(format!("{target}:cross_paragraph_proper_name"));
                continue;
            }
        }
        if profile.is_reflexive && !options.resolve_reflexives {
            rejections.push(format!("{target}:reflexive_disabled"));
            continue;
        }
        if profile.is_zero_subject && !options.detect_zero_subjects {
            rejections.push(format!("{target}:zero_subject_disabled"));
            continue;
        }
        if !compatible(profile, target_profile) {
            rejections.push(format!("{target}:incompatible"));
            continue;
        }
        let score = score_pair(profile, target_profile);
        scored.push((
            target.clone(),
            score,
            ResolutionAlternativeKind::Compatible,
            "compatible".to_string(),
        ));
    }

    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.to_string().cmp(&b.0.to_string())));
    for (target, score, kind, reason) in scored.iter().take(options.max_alternatives) {
        alternatives.push(ResolutionAlternative {
            target: target.clone(),
            score: *score,
            confidence_milli: score_to_confidence(*score),
            kind: *kind,
            reason: reason.clone(),
        });
    }

    let (stage, kind, selected_target, score) = if matches!(
        profile.source_form,
        MentionSourceForm::Generic | MentionSourceForm::Modifier | MentionSourceForm::Unknown
    ) {
        (
            stage_for(profile),
            EntityResolutionDecisionKind::Excluded,
            None,
            0,
        )
    } else if let Some(best) = scored.first() {
        let second = scored.get(1).map(|item| item.1).unwrap_or(i32::MIN / 2);
        if best.1 >= options.acceptance_threshold && best.1 - second >= options.ambiguity_margin {
            let hard_accepted = profile.semantic_entity_id.is_some()
                && profiles
                    .get(&best.0)
                    .and_then(|target| target.semantic_entity_id)
                    == profile.semantic_entity_id;
            (
                stage_for(profile),
                if hard_accepted {
                    EntityResolutionDecisionKind::HardAccepted
                } else {
                    EntityResolutionDecisionKind::Accepted
                },
                Some(best.0.clone()),
                best.1,
            )
        } else if best.1 >= options.acceptance_threshold {
            (
                stage_for(profile),
                EntityResolutionDecisionKind::Ambiguous,
                None,
                best.1,
            )
        } else if best.1 >= options.acceptance_threshold - options.ambiguity_margin {
            (
                stage_for(profile),
                EntityResolutionDecisionKind::Deferred,
                None,
                best.1,
            )
        } else {
            (
                stage_for(profile),
                EntityResolutionDecisionKind::Unresolved,
                None,
                best.1,
            )
        }
    } else {
        (
            stage_for(profile),
            EntityResolutionDecisionKind::Seeded,
            None,
            0,
        )
    };

    if selected_target.is_none()
        && matches!(
            kind,
            EntityResolutionDecisionKind::Accepted | EntityResolutionDecisionKind::HardAccepted
        )
    {
        rejections.push("accepted_without_target".to_string());
    }

    evidence.push(format!("source_form={:?}", profile.source_form));
    evidence.push(format!("concept={}", profile.concept));
    (
        stage,
        kind,
        selected_target,
        score,
        alternatives,
        evidence,
        rejections,
    )
}

#[allow(dead_code)]
pub(crate) fn cluster_key_for_target(
    profiles: &BTreeMap<ResolutionMentionRef, MentionResolutionProfile>,
    target: &ResolutionMentionRef,
) -> Option<String> {
    profiles.get(target).map(cluster_key_for_profile)
}

#[allow(dead_code)]
pub(crate) fn cluster_key_for_profile(profile: &MentionResolutionProfile) -> String {
    if let Some(entity_id) = profile.semantic_entity_id {
        format!("entity:{}", entity_id.0)
    } else {
        format!("mention:{}", profile.mention)
    }
}

pub(crate) fn stage_for(profile: &MentionResolutionProfile) -> EntityResolutionStage {
    match profile.source_form {
        MentionSourceForm::ReflexivePronoun => EntityResolutionStage::Reflexive,
        MentionSourceForm::ProperName => EntityResolutionStage::ProperName,
        MentionSourceForm::Pronoun => EntityResolutionStage::Pronoun,
        MentionSourceForm::PossessivePronoun => EntityResolutionStage::Possessive,
        MentionSourceForm::DefiniteDescription | MentionSourceForm::BareCommonNoun => {
            EntityResolutionStage::DefiniteDescription
        }
        MentionSourceForm::DemonstrativeDescription => EntityResolutionStage::Demonstrative,
        MentionSourceForm::ZeroSubject => EntityResolutionStage::ZeroAnaphora,
        MentionSourceForm::Generic
        | MentionSourceForm::Unknown
        | MentionSourceForm::Modifier
        | MentionSourceForm::Group => EntityResolutionStage::Fallback,
        MentionSourceForm::IndefiniteDescription => EntityResolutionStage::Fallback,
    }
}

pub(crate) fn score_pair(left: &MentionResolutionProfile, right: &MentionResolutionProfile) -> i32 {
    let mut score = 0;
    if left.semantic_entity_id.is_some() && left.semantic_entity_id == right.semantic_entity_id {
        score += 10_000;
    }
    if left.normalized_surface.is_some() && left.normalized_surface == right.normalized_surface {
        score += 850;
    }
    if left.concept == right.concept {
        score += 260;
    }
    if compatible(left, right) {
        score += 120;
    }
    let sentence_distance = left
        .source_sentence_ordinal
        .abs_diff(right.source_sentence_ordinal) as i32;
    score -= 35 * sentence_distance;
    if left.paragraph_id == right.paragraph_id {
        score += 100;
    }
    if left.is_subject_like && right.is_subject_like {
        score += 220;
    }
    if left.is_pronoun || right.is_pronoun {
        score += 180;
    }
    if left.is_reflexive || right.is_reflexive {
        score += 1200;
    }
    if left.is_proper_name || right.is_proper_name {
        score += 200;
    }
    score
}

pub(crate) fn score_to_confidence(score: i32) -> u16 {
    score.clamp(0, 1000) as u16
}

pub(crate) fn compatible(left: &MentionResolutionProfile, right: &MentionResolutionProfile) -> bool {
    left.concept == right.concept
        && feature_option_compatible(left.features.gender, right.features.gender)
        && feature_option_compatible(left.features.number, right.features.number)
        && feature_option_compatible(left.features.person, right.features.person)
        && feature_option_compatible(left.features.animacy, right.features.animacy)
        && feature_option_compatible(left.features.countability, right.features.countability)
        && feature_option_compatible(left.features.concreteness, right.features.concreteness)
}

fn feature_option_compatible<T: PartialEq>(left: Option<T>, right: Option<T>) -> bool {
    left.is_none() || right.is_none() || left == right
}

fn is_subject_like(role: crate::core::interlingua::SemanticRole) -> bool {
    matches!(
        role,
        crate::core::interlingua::SemanticRole::Agent
            | crate::core::interlingua::SemanticRole::Experiencer
            | crate::core::interlingua::SemanticRole::Speaker
            | crate::core::interlingua::SemanticRole::Cognizer
            | crate::core::interlingua::SemanticRole::Topic
            | crate::core::interlingua::SemanticRole::Theme
    )
}

fn classify_source_form(mention: &crate::document::graph::DocumentMentionNode) -> MentionSourceForm {
    let name = mention.normalized_name.as_deref().unwrap_or("");
    if matches!(mention.mention_kind, DocumentMentionKind::Modifier) {
        return MentionSourceForm::Modifier;
    }
    if matches!(mention.mention_kind, DocumentMentionKind::CoordinationGroup) {
        return MentionSourceForm::Group;
    }
    if matches!(mention.reference, Reference::Generic) {
        return MentionSourceForm::Generic;
    }
    if is_reflexive_name(name) {
        return MentionSourceForm::ReflexivePronoun;
    }
    if is_pronoun_name(name) {
        return MentionSourceForm::Pronoun;
    }
    if is_possessive_name(name) {
        return MentionSourceForm::PossessivePronoun;
    }
    if is_demonstrative_name(name) {
        return MentionSourceForm::DemonstrativeDescription;
    }
    if mention.features.definiteness == Some(Definiteness::Definite) {
        return MentionSourceForm::DefiniteDescription;
    }
    if mention.name.is_some() && !name.is_empty() && !name.contains(' ') {
        return MentionSourceForm::ProperName;
    }
    if matches!(mention.reference, Reference::Unresolved) {
        return MentionSourceForm::Unknown;
    }
    if matches!(mention.reference, Reference::Cataphoric(_)) {
        return MentionSourceForm::DemonstrativeDescription;
    }
    if matches!(mention.reference, Reference::Anaphoric(_)) {
        return MentionSourceForm::BareCommonNoun;
    }
    MentionSourceForm::Unknown
}

fn is_pronoun_name(name: &str) -> bool {
    matches!(
        name,
        "i" | "me"
            | "you"
            | "he"
            | "him"
            | "she"
            | "her"
            | "it"
            | "we"
            | "us"
            | "they"
            | "them"
            | "ja"
            | "mnie"
            | "mi"
            | "ty"
            | "ciebie"
            | "ci"
            | "on"
            | "go"
            | "jego"
            | "mu"
            | "ona"
            | "ją"
            | "jej"
            | "ono"
            | "je"
            | "my"
            | "nas"
            | "nam"
            | "wy"
            | "was"
            | "wam"
            | "oni"
            | "one"
            | "ich"
            | "im"
    )
}

fn is_possessive_name(name: &str) -> bool {
    matches!(name, "my" | "your" | "his" | "her" | "its" | "our" | "their" | "swój")
}

fn is_reflexive_name(name: &str) -> bool {
    matches!(
        name,
        "myself"
            | "yourself"
            | "himself"
            | "herself"
            | "itself"
            | "ourselves"
            | "themselves"
            | "się"
            | "siebie"
            | "sobie"
            | "sobą"
    )
}

fn is_demonstrative_name(name: &str) -> bool {
    matches!(name, "this" | "that" | "these" | "those" | "ten" | "ta" | "to" | "tamten")
}

pub(crate) fn summarize(
    mention_profiles: &BTreeMap<ResolutionMentionRef, MentionResolutionProfile>,
    synthetic_mentions: &BTreeMap<
        super::id::SyntheticMentionId,
        super::model::SyntheticResolutionMention,
    >,
    decisions: &BTreeMap<super::id::ResolutionDecisionId, EntityResolutionDecision>,
    clusters: &BTreeMap<super::id::EntityClusterId, ResolvedEntityCluster>,
    diagnostics: &[super::diagnostic::EntityResolutionDiagnostic],
) -> EntityResolutionSummary {
    let mut summary = EntityResolutionSummary {
        mentions_total: mention_profiles.len(),
        synthetic_mentions_total: synthetic_mentions.len(),
        decisions_total: decisions.len(),
        clusters_total: clusters.len(),
        ..Default::default()
    };
    for decision in decisions.values() {
        match decision.kind {
            EntityResolutionDecisionKind::Accepted => summary.accepted += 1,
            EntityResolutionDecisionKind::HardAccepted => summary.hard_accepted += 1,
            EntityResolutionDecisionKind::Ambiguous => summary.ambiguous += 1,
            EntityResolutionDecisionKind::Deferred => summary.deferred += 1,
            EntityResolutionDecisionKind::Unresolved => summary.unresolved += 1,
            EntityResolutionDecisionKind::Excluded => summary.excluded += 1,
            EntityResolutionDecisionKind::Seeded => summary.accepted += 1,
        }
    }
    for cluster in clusters.values() {
        if cluster.mention_refs.len() > 1 {
            summary.resolved_clusters += 1;
        } else {
            summary.unresolved_clusters += 1;
        }
    }
    for diagnostic in diagnostics {
        match diagnostic.severity {
            super::diagnostic::EntityResolutionDiagnosticSeverity::Info => summary.diagnostics_info += 1,
            super::diagnostic::EntityResolutionDiagnosticSeverity::Warning => {
                summary.diagnostics_warning += 1
            }
            super::diagnostic::EntityResolutionDiagnosticSeverity::Error => summary.diagnostics_error += 1,
            super::diagnostic::EntityResolutionDiagnosticSeverity::Fatal => summary.diagnostics_fatal += 1,
        }
    }
    summary
}
