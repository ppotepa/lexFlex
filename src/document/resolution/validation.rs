use super::error::DocumentEntityResolutionValidationError;
use super::hash::document_entity_resolution_hash;
use super::model::DocumentEntityResolution;
use super::resolver_support::compatible;
use crate::document::graph::DocumentGraph;
use crate::document::graph::DocumentGraphNode;
use crate::document::resolution::EntityResolutionDecisionKind;
use std::collections::BTreeSet;

pub struct DocumentEntityResolutionValidator;

impl DocumentEntityResolutionValidator {
    pub fn validate(
        resolution: &DocumentEntityResolution,
    ) -> Result<(), Vec<DocumentEntityResolutionValidationError>> {
        let mut errors = Vec::new();
        if !resolution.schema.is_supported() {
            errors.push(DocumentEntityResolutionValidationError::SchemaVersionMismatch);
        }
        validate_coverage(resolution, &mut errors);
        validate_decisions(resolution, &mut errors);
        validate_clusters(resolution, &mut errors);
        validate_hash(resolution, &mut errors);
        sort_errors(&mut errors);
        errors.dedup();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn validate_intrinsic(
        resolution: &DocumentEntityResolution,
    ) -> Result<(), Vec<DocumentEntityResolutionValidationError>> {
        Self::validate(resolution)
    }

    pub fn validate_against_graph(
        resolution: &DocumentEntityResolution,
        graph: &DocumentGraph,
    ) -> Result<(), Vec<DocumentEntityResolutionValidationError>> {
        let mut errors = Vec::new();
        if let Err(mut intrinsic_errors) = Self::validate(resolution) {
            errors.append(&mut intrinsic_errors);
        }

        let graph_mentions: BTreeSet<_> = graph
            .node_order
            .iter()
            .filter_map(|node_id| match graph.nodes.get(node_id) {
                Some(DocumentGraphNode::Mention(mention)) => Some(mention.id.clone()),
                _ => None,
            })
            .collect();
        for mention in &resolution.mention_order {
            let Some(profile) = resolution.mention_profiles.get(mention) else {
                errors.push(DocumentEntityResolutionValidationError::MissingMentionProfile);
                continue;
            };
            if let Some(graph_id) = &profile.graph_mention_id {
                if !graph_mentions.contains(graph_id) {
                    errors.push(DocumentEntityResolutionValidationError::GraphMentionNotCovered);
                }
            }
        }

        sort_errors(&mut errors);
        errors.dedup();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn validate_coverage(
    resolution: &DocumentEntityResolution,
    errors: &mut Vec<DocumentEntityResolutionValidationError>,
) {
    if resolution.mention_order.len() != resolution.mention_profiles.len() {
        errors.push(DocumentEntityResolutionValidationError::MissingMentionProfile);
    }
    if resolution.decision_order.len() != resolution.decisions.len() {
        errors.push(DocumentEntityResolutionValidationError::MissingDecision);
    }
    if resolution.cluster_order.len() != resolution.clusters.len() {
        errors.push(DocumentEntityResolutionValidationError::MissingCluster);
    }
    if resolution.synthetic_mention_order.len() != resolution.synthetic_mentions.len() {
        errors.push(DocumentEntityResolutionValidationError::MissingSyntheticMention);
    }
}

fn validate_hash(
    resolution: &DocumentEntityResolution,
    errors: &mut Vec<DocumentEntityResolutionValidationError>,
) {
    if let Ok(expected) = document_entity_resolution_hash(resolution) {
        if expected != resolution.resolution_sha256 {
            errors.push(DocumentEntityResolutionValidationError::HashMismatch);
        }
    }
}

fn validate_decisions(
    resolution: &DocumentEntityResolution,
    errors: &mut Vec<DocumentEntityResolutionValidationError>,
) {
    for decision in resolution.decisions.values() {
        for cluster in [decision.antecedent_cluster.as_ref(), decision.result_cluster.as_ref()].into_iter().flatten() {
            if !resolution.clusters.contains_key(cluster) {
                errors.push(DocumentEntityResolutionValidationError::DecisionSelectedClusterMissing);
            }
        }
        if matches!(decision.kind, EntityResolutionDecisionKind::Accepted | EntityResolutionDecisionKind::HardAccepted)
            && decision.selected_target.is_some()
            && decision.antecedent_cluster.is_none()
        {
            errors.push(DocumentEntityResolutionValidationError::DecisionSelectedClusterMissing);
        }
        match decision.kind {
            EntityResolutionDecisionKind::Seeded
            | EntityResolutionDecisionKind::Ambiguous
            | EntityResolutionDecisionKind::Deferred
            | EntityResolutionDecisionKind::Unresolved
            | EntityResolutionDecisionKind::Excluded => {
                if decision.selected_cluster.is_some() {
                    errors.push(DocumentEntityResolutionValidationError::DecisionSelectedClusterNotAllowed);
                }
            }
            EntityResolutionDecisionKind::HardAccepted => {
                if decision.selected_cluster.is_none() {
                    errors.push(DocumentEntityResolutionValidationError::DecisionSelectedClusterMissing);
                }
                if !decision
                    .evidence
                    .iter()
                    .any(|line| line.contains("hard evidence"))
                {
                    errors.push(DocumentEntityResolutionValidationError::DecisionHardAcceptedWithoutEvidence);
                }
            }
            EntityResolutionDecisionKind::Accepted => {
                if decision.selected_cluster.is_none() {
                    errors.push(DocumentEntityResolutionValidationError::DecisionSelectedClusterMissing);
                }
            }
        }
    }
}

fn validate_clusters(
    resolution: &DocumentEntityResolution,
    errors: &mut Vec<DocumentEntityResolutionValidationError>,
) {
    let mut seen_mentions = BTreeSet::new();
    for cluster in resolution.clusters.values() {
        if cluster.mention_refs.is_empty() {
            errors.push(DocumentEntityResolutionValidationError::ClusterMissingRepresentative);
            continue;
        }
        let Some(representative_profile) = resolution.mention_profiles.get(&cluster.representative) else {
            errors.push(DocumentEntityResolutionValidationError::ClusterMissingRepresentative);
            continue;
        };
        for mention in &cluster.mention_refs {
            if !seen_mentions.insert(mention.clone()) {
                errors.push(DocumentEntityResolutionValidationError::ClusterDuplicateMention);
            }
            let Some(profile) = resolution.mention_profiles.get(mention) else {
                errors.push(DocumentEntityResolutionValidationError::ClusterMentionNotResolved);
                continue;
            };
            if !compatible(representative_profile, profile) {
                errors.push(DocumentEntityResolutionValidationError::ClusterIncompatibleMentions);
            }
        }
    }
}

fn sort_errors(errors: &mut [DocumentEntityResolutionValidationError]) {
    errors.sort_by_key(|error| format!("{error:?}"));
}
