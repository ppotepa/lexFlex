use crate::document::graph::{
    DocumentGraph, DocumentGraphNode,
};
use std::collections::BTreeMap;

use super::hash::document_entity_resolution_hash;
use super::id::{
    DocumentEntityResolutionIdFactory, EntityClusterId, ResolutionDecisionId, ResolutionId,
    ResolutionMentionRef,
};
use super::model::{
    DocumentEntityResolution, EntityResolutionDecision, EntityResolutionDecisionKind, MentionResolutionProfile, MentionSourceForm, ResolvedEntityCluster,
};
use super::options::DocumentEntityResolutionOptions;
use super::resolver_support::{build_profile, choose_resolution, summarize};
use super::schema::DocumentEntityResolutionSchema;

pub struct DocumentEntityResolver {
    options: DocumentEntityResolutionOptions,
}

impl Default for DocumentEntityResolver {
    fn default() -> Self {
        Self {
            options: DocumentEntityResolutionOptions::default(),
        }
    }
}

impl DocumentEntityResolver {
    pub fn with_options(options: DocumentEntityResolutionOptions) -> Self {
        Self { options }
    }

    pub fn resolve(&self, graph: &DocumentGraph) -> Result<DocumentEntityResolution, super::DocumentEntityResolutionError> {
        let options_sha256 = self.options.fingerprint();
        let resolution_id = DocumentEntityResolutionIdFactory::resolution(
            &graph.source_document_id,
            &graph.id,
            &options_sha256,
            &graph.graph_sha256,
        );

        let mut mention_profiles = BTreeMap::new();
        let mut mention_order = Vec::new();
        let mut graph_candidate_atoms = Vec::new();
        let mut synthetic_mentions = BTreeMap::new();
        let mut synthetic_mention_order = Vec::new();
        let mut decisions = BTreeMap::new();
        let mut decision_order = Vec::new();
        let mut clusters = BTreeMap::new();
        let mut cluster_order = Vec::new();
        let diagnostics = Vec::new();
        let mut cluster_by_mention: BTreeMap<ResolutionMentionRef, EntityClusterId> = BTreeMap::new();
        let mut next_cluster_ordinal = 0usize;
        let mut next_decision_ordinal = 0usize;
        let mut next_synthetic_ordinal = 0usize;
        let mut accepted_mentions: Vec<ResolutionMentionRef> = Vec::new();

        for node_id in &graph.node_order {
            let Some(node) = graph.nodes.get(node_id) else { continue; };
            if let DocumentGraphNode::EntityCandidate(candidate) = node {
                graph_candidate_atoms.push(candidate.id.clone());
            }
        }

        for node_id in &graph.node_order {
            let Some(node) = graph.nodes.get(node_id) else { continue; };
            let DocumentGraphNode::Mention(mention) = node else { continue; };
            let mention_ref = ResolutionMentionRef::Graph(mention.id.clone());
            let profile = build_profile(graph, mention_ref.clone(), mention);
            mention_order.push(mention_ref.clone());
            mention_profiles.insert(mention_ref.clone(), profile.clone());

            let (stage, kind, selected_target, score, alternatives, evidence, rejections) =
                choose_resolution(
                    &profile,
                    &accepted_mentions,
                    &mention_profiles,
                    self.options.clone(),
                );
            let decision_id = DocumentEntityResolutionIdFactory::decision(&resolution_id, next_decision_ordinal);
            next_decision_ordinal += 1;
            decision_order.push(decision_id.clone());
            let mut selected_cluster = None;
            let mut hard_evidence = Vec::new();
            match kind {
                EntityResolutionDecisionKind::Seeded => {
                    ensure_cluster(
                        &resolution_id,
                        &mut next_cluster_ordinal,
                        &mut cluster_order,
                        &mut clusters,
                        &mut cluster_by_mention,
                        mention_ref.clone(),
                        &profile,
                        &decision_id,
                    );
                    selected_cluster = None;
                }
                EntityResolutionDecisionKind::Accepted | EntityResolutionDecisionKind::HardAccepted => {
                    if let Some(target) = &selected_target {
                        if let Some(cluster_id) = cluster_by_mention.get(target).cloned() {
                            selected_cluster = Some(cluster_id.clone());
                            if let Some(cluster) = clusters.get_mut(&cluster_id) {
                                if !cluster.mention_refs.iter().any(|existing| existing == &mention_ref) {
                                    cluster.mention_refs.push(mention_ref.clone());
                                }
                                if cluster.canonical_name.is_none() {
                                    cluster.canonical_name = profile
                                        .normalized_surface
                                        .clone()
                                        .or_else(|| profile.exact_surface.clone());
                                }
                                if !cluster.decision_ids.iter().any(|existing| existing == &decision_id) {
                                    cluster.decision_ids.push(decision_id.clone());
                                }
                            }
                            cluster_by_mention.insert(mention_ref.clone(), cluster_id);
                        }
                    }
                    if selected_cluster.is_none() {
                        let cluster_id = ensure_cluster(
                            &resolution_id,
                            &mut next_cluster_ordinal,
                            &mut cluster_order,
                            &mut clusters,
                            &mut cluster_by_mention,
                            mention_ref.clone(),
                            &profile,
                            &decision_id,
                        );
                        selected_cluster = Some(cluster_id);
                    }
                    if matches!(kind, EntityResolutionDecisionKind::HardAccepted) {
                        if let Some(entity_id) = profile.semantic_entity_id {
                            hard_evidence
                                .push(format!("hard evidence: semantic_entity_id={}", entity_id.0));
                        }
                    }
                }
                _ => {}
            }
            decisions.insert(
                decision_id.clone(),
                EntityResolutionDecision {
                    id: decision_id.clone(),
                    mention: mention_ref.clone(),
                    stage,
                    kind,
                    selected_cluster,
                    selected_target: selected_target.clone(),
                    score,
                    threshold: self.options.acceptance_threshold,
                    margin: self.options.ambiguity_margin,
                    alternatives,
                    evidence: evidence
                        .into_iter()
                        .chain(hard_evidence.into_iter())
                        .collect(),
                    rejections,
                    sentence_id: mention.source_sentence_id.clone(),
                },
            );
            if matches!(
                kind,
                EntityResolutionDecisionKind::Seeded
                    | EntityResolutionDecisionKind::Accepted
                    | EntityResolutionDecisionKind::HardAccepted
            ) {
                accepted_mentions.push(mention_ref.clone());
            }

            if matches!(profile.source_form, MentionSourceForm::ZeroSubject) && self.options.detect_zero_subjects {
                let synthetic_id = DocumentEntityResolutionIdFactory::synthetic(&resolution_id, next_synthetic_ordinal);
                next_synthetic_ordinal += 1;
                synthetic_mention_order.push(synthetic_id.clone());
                synthetic_mentions.insert(
                    synthetic_id.clone(),
                    super::model::SyntheticResolutionMention {
                        id: synthetic_id,
                        source_sentence_id: mention.source_sentence_id.clone(),
                        semantic_sentence_id: mention.semantic_sentence_id.clone(),
                        frame_occurrence_id: mention.frame_occurrence_id.clone(),
                        subject_role: mention.role_context,
                        concept: mention.concept.clone(),
                        features: mention.features.clone(),
                        anchor: mention.anchor.clone(),
                        origin: "PolishZeroSubject".to_string(),
                        evidence: vec!["zero-subject heuristic".to_string()],
                    },
                );
            }
        }

        let source_graph_diagnostics = graph.diagnostics.clone();
        let summary = summarize(
            &mention_profiles,
            &synthetic_mentions,
            &decisions,
            &clusters,
            &diagnostics,
        );
        let mut resolution = DocumentEntityResolution {
            schema: DocumentEntityResolutionSchema::CURRENT,
            id: resolution_id,
            source_document_id: graph.source_document_id.clone(),
            source_graph_id: graph.id.clone(),
            source_graph_sha256: graph.graph_sha256.clone(),
            source_sha256: graph.source_sha256.clone(),
            options: self.options.clone(),
            options_sha256,
            graph_candidate_atoms,
            synthetic_mentions,
            synthetic_mention_order,
            mention_profiles,
            mention_order,
            decisions,
            decision_order,
            clusters,
            cluster_order,
            diagnostics,
            source_graph_diagnostics,
            summary,
            resolution_sha256: String::new(),
        };
        resolution.resolution_sha256 = document_entity_resolution_hash(&resolution)
            .map_err(|_| super::DocumentEntityResolutionError::InvalidGraph)?;
        Ok(resolution)
    }
}

fn ensure_cluster(
    resolution_id: &ResolutionId,
    next_cluster_ordinal: &mut usize,
    cluster_order: &mut Vec<EntityClusterId>,
    clusters: &mut BTreeMap<EntityClusterId, ResolvedEntityCluster>,
    cluster_by_mention: &mut BTreeMap<ResolutionMentionRef, EntityClusterId>,
    representative: ResolutionMentionRef,
    profile: &MentionResolutionProfile,
    decision_id: &ResolutionDecisionId,
) -> EntityClusterId {
    if let Some(cluster_id) = cluster_by_mention.get(&representative).cloned() {
        return cluster_id;
    }
    let cluster_id =
        DocumentEntityResolutionIdFactory::cluster(resolution_id, *next_cluster_ordinal);
    *next_cluster_ordinal += 1;
    cluster_order.push(cluster_id.clone());
    clusters.insert(
        cluster_id.clone(),
        ResolvedEntityCluster {
            id: cluster_id.clone(),
            representative: representative.clone(),
            mention_refs: vec![representative.clone()],
            canonical_name: profile
                .normalized_surface
                .clone()
                .or_else(|| profile.exact_surface.clone()),
            aliases: Vec::new(),
            canonical_concept: profile.concept.clone(),
            compatible_concepts: vec![profile.concept.clone()],
            features: profile.features.clone(),
            decision_ids: vec![decision_id.clone()],
            confidence_milli: 1000,
        },
    );
    cluster_by_mention.insert(representative, cluster_id.clone());
    cluster_id
}
