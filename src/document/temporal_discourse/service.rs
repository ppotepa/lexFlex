use crate::document::compilation::DocumentCompilation;
use crate::document::graph::DocumentGraph;
use crate::document::resolution::DocumentEntityResolution;
use std::collections::BTreeMap;

use super::hash::document_temporal_discourse_hash;
use super::id::DocumentTemporalDiscourseIdFactory;
use super::model::{
    DiscourseRelation, DocumentReferenceTime, DocumentTemporalDiscourse,
    DocumentTemporalDiscourseDiagnostic, DocumentTemporalDiscourseDiagnosticSeverity, EventProfile, EventTemporalAssignment, ResolvedDocumentGenerationPlan, TemporalExpression,
    TemporalClosureConflict, TemporalRelation, TemporalRelationKind, EventTemporalRef,
};
use super::options::DocumentTemporalDiscourseOptions;
use super::schema::DocumentTemporalDiscourseSchema;
use super::helpers::{
    build_event_coreference, build_ordered_steps, cue_relation,
    normalize_temporal, sentence_pair_relation, summarize_temporal_counts,
    temporal_evidence, temporal_expression_kind, temporal_relation_for_sentence,
};

#[derive(Debug, thiserror::Error)]
pub enum DocumentTemporalDiscourseError {
    #[error("graph artifact missing sentence node {0}")]
    MissingSentence(String),
    #[error("document service failed: {0}")]
    Document(#[from] crate::document::service::DocumentServiceError),
    #[error("graph service failed: {0}")]
    Graph(#[from] crate::document::graph::DocumentGraphServiceError),
    #[error("entity resolution failed: {0}")]
    Resolution(#[from] crate::document::resolution::DocumentEntityResolutionServiceError),
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct DocumentTemporalDiscourseService {
    options: DocumentTemporalDiscourseOptions,
}

impl Default for DocumentTemporalDiscourseService {
    fn default() -> Self {
        Self {
            options: DocumentTemporalDiscourseOptions::default(),
        }
    }
}

impl DocumentTemporalDiscourseService {
    pub fn new(_api: &crate::api::LexFlexAPI) -> Self {
        Self::default()
    }

    pub fn with_options(options: DocumentTemporalDiscourseOptions) -> Self {
        Self { options }
    }

    pub fn resolve(
        &self,
        compilation: &DocumentCompilation,
        graph: &DocumentGraph,
        resolution: Option<&DocumentEntityResolution>,
    ) -> Result<DocumentTemporalDiscourse, DocumentTemporalDiscourseError> {
        let artifact_id = DocumentTemporalDiscourseIdFactory::artifact(
            &compilation.document.id,
            &graph.id,
            resolution.map(|value| &value.id),
            &self.options.fingerprint(),
            &graph.graph_sha256,
            resolution.map(|value| value.resolution_sha256.as_str()),
        );
        let reference_time = DocumentReferenceTime {
            id: DocumentTemporalDiscourseIdFactory::reference_time(&compilation.document.id),
            label: self
                .options
                .reference_time
                .clone()
                .unwrap_or_else(|| "document-context".to_string()),
            civil_date: None,
            clock_time: None,
            timezone_offset: None,
            explicit: self.options.reference_time.is_some(),
        };

        let mut event_profiles = BTreeMap::new();
        let mut event_profile_order = Vec::new();
        let mut temporal_expressions = BTreeMap::new();
        let mut temporal_expression_order = Vec::new();
        let mut event_assignments = BTreeMap::new();
        let mut event_assignment_order = Vec::new();
        let mut temporal_relations = BTreeMap::new();
        let mut temporal_relation_order = Vec::new();
        let mut event_coreference_decisions = BTreeMap::new();
        let mut event_coreference_decision_order = Vec::new();
        let mut event_coreference_clusters = BTreeMap::new();
        let mut event_coreference_cluster_order = Vec::new();
        let mut discourse_units = BTreeMap::new();
        let mut discourse_unit_order = Vec::new();
        let mut discourse_relations = BTreeMap::new();
        let mut discourse_relation_order = Vec::new();
        let mut diagnostics = Vec::new();

        let mut sentence_index = 0usize;
        for sentence in compilation.document.ordered_sentences() {
            let Some(graph_sentence) = graph.source_sentence_node(&sentence.id) else {
                return Err(DocumentTemporalDiscourseError::MissingSentence(sentence.id.to_string()));
            };
            let semantic_sentence = graph
                .semantic_sentences_for_source(&sentence.id)
                .first()
                .copied();
            let text = compilation
                .document
                .sentence_text(&sentence.id)
                .unwrap_or("")
                .trim()
                .to_string();
            discourse_units.insert(sentence.id.clone(), text.clone());
            discourse_unit_order.push(sentence.id.clone());
            if let Some((kind, canonical_direction, implicit)) = cue_relation(&text) {
                if let Some(previous_sentence) = compilation.document.ordered_sentences().get(sentence_index.saturating_sub(1)).filter(|_| sentence_index > 0) {
                    let relation_id = DocumentTemporalDiscourseIdFactory::discourse_relation(&artifact_id, discourse_relation_order.len());
                    let relation = DiscourseRelation {
                        id: relation_id.clone(),
                        from_sentence_id: previous_sentence.id.clone(),
                        to_sentence_id: sentence.id.clone(),
                        kind,
                        canonical_direction,
                        cue_evidence: vec![text.clone()],
                        implicit,
                    };
                    discourse_relations.insert(relation_id.clone(), relation);
                    discourse_relation_order.push(relation_id);
                } else {
                    diagnostics.push(document_temporal_discourse_diagnostic("DISCOURSE_CUE_WITHOUT_PREDECESSOR", &text));
                }
            }

            for event in graph.events_for_source(&sentence.id) {
                let profile_id = event.id.clone();
                event_profile_order.push(profile_id.clone());
                event_profiles.insert(
                    profile_id.clone(),
                    EventProfile {
                        id: profile_id.clone(),
                        sentence_id: sentence.id.clone(),
                        event_node_id: event.id.clone(),
                        frame_occurrence_id: event.frame_occurrence_id.clone(),
                        frame_type: event.frame_type.clone(),
                        verb_concept: event.verb_concept.clone(),
                        participant_count: event.participant_count,
                        polarity: semantic_sentence
                            .map(|semantic| format!("{:?}", semantic.polarity))
                            .unwrap_or_else(|| "Positive".to_string()),
                        aspect: semantic_sentence
                            .and_then(|semantic| semantic.aspect)
                            .map(|aspect| format!("{aspect:?}")),
                    },
                );

                let temporal_id = DocumentTemporalDiscourseIdFactory::temporal_expression(
                    &artifact_id,
                    temporal_expression_order.len(),
                );
                let normalized = normalize_temporal(
                    semantic_sentence.and_then(|semantic| semantic.temporal.clone()),
                    &text,
                    self.options.reference_time.as_deref(),
                );
                temporal_expressions.insert(
                    temporal_id.clone(),
                    TemporalExpression {
                        id: temporal_id.clone(),
                        sentence_id: sentence.id.clone(),
                        source_text: text.clone(),
                        kind: temporal_expression_kind(&normalized),
                        normalized: normalized.clone(),
                        explicit_reference_time: if reference_time.explicit {
                            Some(reference_time.id.clone())
                        } else {
                            None
                        },
                        evidence: temporal_evidence(
                            semantic_sentence.and_then(|semantic| semantic.temporal.clone()),
                            &text,
                            &normalized,
                        ),
                    },
                );
                temporal_expression_order.push(temporal_id.clone());

                let assignment_id = DocumentTemporalDiscourseIdFactory::event_assignment(
                    &artifact_id,
                    event_assignment_order.len(),
                );
                let relation = temporal_relation_for_sentence(
                    semantic_sentence.and_then(|semantic| semantic.temporal.clone()),
                    &normalized,
                );
                event_assignments.insert(
                    assignment_id.clone(),
                    EventTemporalAssignment {
                        id: assignment_id.clone(),
                        event_ref: EventTemporalRef::Event(profile_id.clone()),
                        event_profile_id: profile_id.clone(),
                        temporal_expression_id: Some(temporal_id.clone()),
                        relation: relation.clone(),
                        evidence: vec![text.clone()],
                    },
                );
                event_assignment_order.push(assignment_id);
                let relation_id = DocumentTemporalDiscourseIdFactory::temporal_relation(
                    &artifact_id,
                    temporal_relation_order.len(),
                );
                temporal_relations.insert(
                    relation_id.clone(),
                    TemporalRelation {
                        id: relation_id.clone(),
                        from_event_id: profile_id.clone(),
                        to_event_id: graph_sentence.id.clone(),
                        kind: relation,
                        evidence: vec![text.clone()],
                    },
                );
                temporal_relation_order.push(relation_id);
            }

            let ordered_sentences = compilation.document.ordered_sentences();
            if let Some(relation) = sentence_pair_relation(&ordered_sentences, sentence_index, &text) {
                let current_endpoint = semantic_sentence
                    .map(|semantic| semantic.id.clone())
                    .unwrap_or_else(|| graph_sentence.id.clone());
                let previous_endpoint = ordered_sentences
                    .get(sentence_index.saturating_sub(1))
                    .and_then(|previous_sentence| graph.semantic_sentences_for_source(&previous_sentence.id).first().copied())
                    .map(|semantic| semantic.id.clone())
                    .unwrap_or_else(|| current_endpoint.clone());
                let relation_id = DocumentTemporalDiscourseIdFactory::temporal_relation(
                    &artifact_id,
                    temporal_relation_order.len(),
                );
                temporal_relations.insert(
                    relation_id.clone(),
                    TemporalRelation {
                        id: relation_id.clone(),
                        from_event_id: previous_endpoint,
                        to_event_id: current_endpoint,
                        kind: relation.0,
                        evidence: relation.1,
                    },
                );
                temporal_relation_order.push(relation_id);
            }

            sentence_index += 1;
        }

        if self.options.enable_event_coreference {
            build_event_coreference(
                &artifact_id,
                &event_profiles,
                &mut event_coreference_decisions,
                &mut event_coreference_decision_order,
                &mut event_coreference_clusters,
                &mut event_coreference_cluster_order,
            );
        }

        let (temporal_closure, temporal_conflicts) = build_temporal_closure(&artifact_id, &temporal_relations);

        if self.options.enable_generation_plan {
            diagnostics.push(DocumentTemporalDiscourseDiagnostic {
                code: "GENERATION_PLAN_BUILT".into(),
                message: "generation plan derived from entity, temporal, and discourse directives".into(),
                severity: DocumentTemporalDiscourseDiagnosticSeverity::Info,
            });
        }

        let generation_plan = if self.options.enable_generation_plan {
            Some(ResolvedDocumentGenerationPlan {
                entity_cluster_refs: resolution
                    .map(|r| r.cluster_order.clone())
                    .unwrap_or_default(),
                temporal_directives: temporal_expressions
                    .values()
                    .map(|expr| format!("{:?}", expr.normalized))
                    .collect(),
                discourse_directives: discourse_relations
                    .values()
                    .map(|relation| format!("{:?}:{:?}", relation.kind, relation.canonical_direction))
                    .collect(),
                ordered_steps: build_ordered_steps(
                    resolution,
                    &temporal_expressions,
                    &discourse_relations,
                ),
            })
        } else {
            None
        };

        let summary = summarize_temporal_counts(
            compilation,
            &event_profiles,
            &temporal_expressions,
            &temporal_relations,
            &temporal_conflicts,
            &event_coreference_decisions,
            &event_coreference_clusters,
            &discourse_relations,
            &diagnostics,
        );

        let mut artifact = DocumentTemporalDiscourse {
            schema: DocumentTemporalDiscourseSchema::CURRENT,
            id: artifact_id,
            source_document_id: compilation.document.id.clone(),
            source_graph_id: graph.id.clone(),
            source_graph_sha256: graph.graph_sha256.clone(),
            source_resolution_id: resolution.map(|value| value.id.clone()),
            source_resolution_sha256: resolution.map(|value| value.resolution_sha256.clone()),
            source_sha256: compilation.document.source_sha256.clone(),
            options: self.options.clone(),
            options_sha256: self.options.fingerprint(),
            reference_time,
            event_profiles,
            event_profile_order,
            temporal_expressions,
            temporal_expression_order,
            event_assignments,
            event_assignment_order,
            temporal_relations,
            temporal_relation_order,
            temporal_closure,
            temporal_conflicts,
            event_coreference_decisions,
            event_coreference_decision_order,
            event_coreference_clusters,
            event_coreference_cluster_order,
            discourse_units,
            discourse_unit_order,
            discourse_relations,
            discourse_relation_order,
            generation_plan,
            diagnostics,
            summary,
            temporal_discourse_sha256: String::new(),
        };
        artifact.temporal_discourse_sha256 = document_temporal_discourse_hash(&artifact)?;
        Ok(artifact)
    }
}

fn build_temporal_closure(
    artifact_id: &str,
    direct: &BTreeMap<super::id::TemporalRelationId, TemporalRelation>,
) -> (Vec<TemporalRelation>, Vec<TemporalClosureConflict>) {
    let mut edges = BTreeMap::<(crate::document::graph::GraphNodeId, crate::document::graph::GraphNodeId), TemporalRelationKind>::new();
    for relation in direct.values() {
        if !matches!(relation.kind, TemporalRelationKind::Before | TemporalRelationKind::After) { continue; }
        let key = (relation.from_event_id.clone(), relation.to_event_id.clone());
        if let Some(previous) = edges.insert(key.clone(), relation.kind.clone()) {
            if previous != relation.kind {
                // Retain the conflict below; the direct relation remains
                // inspectable and is never silently discarded.
                edges.insert(key, previous);
            }
        }
    }
    let mut changed = true;
    while changed {
        changed = false;
        let snapshot = edges.clone();
        for ((left, middle), left_kind) in &snapshot {
            for ((candidate_middle, right), right_kind) in &snapshot {
                if middle != candidate_middle || left_kind != right_kind { continue; }
                let key = (left.clone(), right.clone());
                if !edges.contains_key(&key) {
                    edges.insert(key, left_kind.clone());
                    changed = true;
                }
            }
        }
    }
    let mut closure = Vec::new();
    for (ordinal, ((from, to), kind)) in edges.iter().enumerate() {
        let id = DocumentTemporalDiscourseIdFactory::temporal_relation(artifact_id, 10_000 + ordinal);
        closure.push(TemporalRelation { id, from_event_id: from.clone(), to_event_id: to.clone(), kind: kind.clone(), evidence: vec!["transitive-closure".into()] });
    }
    let mut conflicts = Vec::new();
    for ((from, to), kind) in &edges {
        let inverse = match kind { TemporalRelationKind::Before => TemporalRelationKind::After, TemporalRelationKind::After => TemporalRelationKind::Before, _ => continue };
        if edges.get(&(to.clone(), from.clone())) == Some(&inverse) {
            conflicts.push(TemporalClosureConflict { left_event_id: from.clone(), right_event_id: to.clone(), relation: kind.clone(), reason: "opposite temporal directions".into() });
        }
    }
    conflicts.sort_by_key(|conflict| (conflict.left_event_id.clone(), conflict.right_event_id.clone()));
    conflicts.dedup_by(|left, right| left.left_event_id == right.left_event_id && left.right_event_id == right.right_event_id);
    (closure, conflicts)
}

fn document_temporal_discourse_diagnostic(
    code: &str,
    message: &str,
) -> DocumentTemporalDiscourseDiagnostic {
    DocumentTemporalDiscourseDiagnostic {
        code: code.into(),
        message: message.into(),
        severity: DocumentTemporalDiscourseDiagnosticSeverity::Warning,
    }
}
