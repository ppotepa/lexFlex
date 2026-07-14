use crate::document::compilation::DocumentCompilation;
use crate::document::graph::{DocumentGraph, DocumentGraphNode};
use crate::document::resolution::DocumentEntityResolution;
use crate::document::temporal_discourse::DocumentTemporalDiscourse;
use crate::core::interlingua::{Entity, Frame, Polarity, Quantifier, SemanticRole};
use serde_json::json;
use std::collections::BTreeMap;

use super::hash::document_knowledge_hash;
use super::id::{
    KnowledgeExtractionIdFactory, ClaimId,
};
use super::model::{
    ClaimAttribution, ClaimFactuality, ClaimStatus, ClaimWorldRef,
    ContradictionSet, DocumentClaim, DocumentKnowledgeExtraction, KnowledgeDiagnostic,
    KnowledgeDiagnosticSeverity, KnowledgeObjectRef, KnowledgePredicateRef, KnowledgeQualifier,
    KnowledgeRelationKind, KnowledgeSubjectRef, KnowledgeExtractionSummary,
    PropositionOccurrence,
};
use super::options::DocumentKnowledgeExtractionOptions;
use super::schema::DocumentKnowledgeSchema;

#[derive(Debug, thiserror::Error)]
pub enum DocumentKnowledgeError {
    #[error("document service failed: {0}")]
    Document(#[from] crate::document::service::DocumentServiceError),
    #[error("graph service failed: {0}")]
    Graph(#[from] crate::document::graph::DocumentGraphServiceError),
    #[error("resolution service failed: {0}")]
    Resolution(#[from] crate::document::resolution::DocumentEntityResolutionServiceError),
    #[error("temporal-discourse service failed: {0}")]
    TemporalDiscourse(#[from] crate::document::temporal_discourse::DocumentTemporalDiscourseError),
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct DocumentKnowledgeService {
    options: DocumentKnowledgeExtractionOptions,
}

impl Default for DocumentKnowledgeService {
    fn default() -> Self {
        Self {
            options: DocumentKnowledgeExtractionOptions::default(),
        }
    }
}

impl DocumentKnowledgeService {
    pub fn with_options(options: DocumentKnowledgeExtractionOptions) -> Self {
        Self { options }
    }

    pub fn extract(
        &self,
        compilation: &DocumentCompilation,
        graph: &DocumentGraph,
        resolution: Option<&DocumentEntityResolution>,
        temporal: Option<&DocumentTemporalDiscourse>,
    ) -> Result<DocumentKnowledgeExtraction, DocumentKnowledgeError> {
        let artifact_id = KnowledgeExtractionIdFactory::artifact(
            &compilation.document.id,
            &graph.id,
            resolution.map(|value| &value.id),
            temporal.map(|value| &value.id),
            &self.options,
        );

        let mut proposition_occurrences = BTreeMap::new();
        let mut proposition_order = Vec::new();
        let mut values = BTreeMap::new();
        let mut value_order = Vec::new();
        let mut claims: BTreeMap<ClaimId, DocumentClaim> = BTreeMap::new();
        let mut claim_order = Vec::new();
        let mut contradiction_sets = BTreeMap::new();
        let mut contradiction_order = Vec::new();
        let diagnostics: Vec<KnowledgeDiagnostic> = Vec::new();

        for sentence in compilation.ordered_results() {
            let Some(utterance) = sentence.semantics.as_ref() else { continue };
            for semantic_sentence in &utterance.sentences {
                for (frame_ordinal, frame) in semantic_sentence.frames.iter().enumerate() {
                    let Some((subject, predicate, mut object)) = frame_fact(frame, resolution) else {
                        continue;
                    };
                    if let Some(number) = semantic_sentence.quantification.as_ref().and_then(|value| match value {
                        Quantifier::Numerical(number) => Some(*number),
                        _ => None,
                    }) {
                        if matches!(frame, Frame::Statement { .. }) {
                            let value_id = KnowledgeExtractionIdFactory::value(&artifact_id, value_order.len());
                            values.insert(value_id.clone(), super::model::KnowledgeValue::Integer(i128::from(number)));
                            value_order.push(value_id.clone());
                            object = KnowledgeObjectRef::Value(value_id);
                        }
                    }
                    let factuality = if semantic_sentence.question.is_some() {
                        ClaimFactuality::Questioned
                    } else if matches!(semantic_sentence.polarity, Polarity::Negative) {
                        ClaimFactuality::Negated
                    } else {
                        ClaimFactuality::Asserted
                    };
                    let world = if semantic_sentence.question.is_some() {
                        ClaimWorldRef::Question
                    } else {
                        ClaimWorldRef::Actual
                    };
                    let occurrence_id = KnowledgeExtractionIdFactory::proposition(
                        &artifact_id,
                        proposition_order.len(),
                    );
                    let evidence = frame_evidence(graph, &sentence.sentence_id, frame);
                    let source_spans = frame_source_spans(graph, &sentence.sentence_id, frame);
                    let occurrence = PropositionOccurrence {
                        id: occurrence_id.clone(),
                        source_sentence_id: sentence.sentence_id.clone(),
                        semantic_sentence_id: graph.semantic_sentences_for_source(&sentence.sentence_id).first().map(|node| node.id.clone()),
                        frame_occurrence_id: graph.frames_for_source(&sentence.sentence_id).get(frame_ordinal).map(|node| node.id.clone()),
                        event_id: None,
                        event_cluster_id: None,
                        subject,
                        predicate,
                        object,
                        qualifiers: vec![KnowledgeQualifier::SourceSentence(
                            sentence.sentence_id.clone(),
                        )],
                        factuality,
                        world,
                        attribution: Some(ClaimAttribution {
                            source_sentence_id: Some(sentence.sentence_id.clone()),
                            source_document_id: compilation.document.id.clone(),
                            attributed_to: None,
                            nested_content: false,
                        }),
                        temporal_scope: None,
                        confidence_milli: 700,
                        evidence,
                        source_spans,
                    };
                    proposition_order.push(occurrence_id.clone());
                    proposition_occurrences.insert(occurrence_id, occurrence);
                }
            }
        }

        let mut claims_by_signature = BTreeMap::<String, ClaimId>::new();
        for occurrence_id in &proposition_order {
            let Some(occurrence) = proposition_occurrences.get(occurrence_id) else { continue };
            let signature = signature_hash(occurrence);
            if let Some(claim_id) = claims_by_signature.get(&signature) {
                if let Some(claim) = claims.get_mut(claim_id) {
                    claim.occurrence_ids.push(occurrence_id.clone());
                    claim.confidence_milli = claim.confidence_milli.max(occurrence.confidence_milli);
                }
                continue;
            }
            let claim_id = KnowledgeExtractionIdFactory::claim(&artifact_id, claim_order.len());
            claims_by_signature.insert(signature.clone(), claim_id.clone());
            claim_order.push(claim_id.clone());
            let status = match occurrence.factuality {
                ClaimFactuality::Asserted => ClaimStatus::Supported,
                ClaimFactuality::Negated => ClaimStatus::Contradicted,
                ClaimFactuality::Questioned | ClaimFactuality::Conditional | ClaimFactuality::Hypothetical => ClaimStatus::NonFactual,
                _ => ClaimStatus::Unresolved,
            };
            claims.insert(claim_id.clone(), DocumentClaim {
                id: claim_id,
                signature_sha256: signature,
                occurrence_ids: vec![occurrence_id.clone()],
                status,
                canonical_subject: occurrence.subject.clone(),
                canonical_predicate: occurrence.predicate.clone(),
                canonical_object: occurrence.object.clone(),
                canonical_qualifiers: occurrence.qualifiers.clone(),
                factuality: occurrence.factuality.clone(),
                world: occurrence.world.clone(),
                confidence_milli: occurrence.confidence_milli,
            });
        }

        if self.options.include_contradictions {
            let mut seen = BTreeMap::<String, Vec<ClaimId>>::new();
            for claim_id in &claim_order {
                if let Some(claim) = claims.get(claim_id) {
                    let key = format!("{:?}|{:?}|{:?}", claim.canonical_subject, claim.canonical_predicate, claim.world);
                    seen.entry(key).or_default().push(claim_id.clone());
                }
            }
            for (_, claim_ids) in seen {
                let has_negation = claim_ids.iter().any(|id| claims.get(id).is_some_and(|claim| matches!(claim.status, ClaimStatus::Contradicted)));
                let different_values = claim_ids.iter().filter_map(|id| claims.get(id).map(|claim| format!("{:?}", claim.canonical_object))).collect::<std::collections::BTreeSet<_>>().len() > 1;
                let functional = claim_ids.first().and_then(|id| claims.get(id)).is_some_and(|claim| functional_predicate(&claim.canonical_predicate));
                let conflicting = claim_ids.len() > 1 && (has_negation || (functional && different_values));
                if conflicting {
                    for claim_id in &claim_ids {
                        if let Some(claim) = claims.get_mut(claim_id) {
                            if matches!(claim.status, ClaimStatus::Supported | ClaimStatus::Contradicted) {
                                claim.status = ClaimStatus::Contested;
                            }
                        }
                    }
                    let contradiction_id = KnowledgeExtractionIdFactory::contradiction_set(
                        &artifact_id,
                        contradiction_order.len(),
                    );
                    contradiction_order.push(contradiction_id.clone());
                    contradiction_sets.insert(
                        contradiction_id.clone(),
                        ContradictionSet {
                            id: contradiction_id,
                            claim_ids,
                            kind: "incompatible-scope-values".into(),
                            scope_label: None,
                            status: "Open".into(),
                        },
                    );
                }
            }
        }

        let summary = KnowledgeExtractionSummary {
            proposition_occurrences_total: proposition_occurrences.len(),
            claims_total: claims.len(),
            values_total: values.len(),
            qualifiers_total: proposition_occurrences.values().map(|occ| occ.qualifiers.len()).sum(),
            contradiction_sets_total: contradiction_sets.len(),
            diagnostics_info: diagnostics
                .iter()
                .filter(|diagnostic| matches!(diagnostic.severity, KnowledgeDiagnosticSeverity::Info))
                .count(),
            diagnostics_warning: diagnostics
                .iter()
                .filter(|diagnostic| matches!(diagnostic.severity, KnowledgeDiagnosticSeverity::Warning))
                .count(),
            diagnostics_error: diagnostics
                .iter()
                .filter(|diagnostic| matches!(diagnostic.severity, KnowledgeDiagnosticSeverity::Error))
                .count(),
        };

        let mut artifact = DocumentKnowledgeExtraction {
            schema: DocumentKnowledgeSchema::CURRENT,
            id: artifact_id,
            source_document_id: compilation.document.id.clone(),
            source_graph_id: graph.id.clone(),
            source_graph_sha256: graph.graph_sha256.clone(),
            source_resolution_id: resolution.map(|value| value.id.clone()),
            source_resolution_sha256: resolution.map(|value| value.resolution_sha256.clone()),
            source_temporal_discourse_id: temporal.map(|value| value.id.clone()),
            source_temporal_discourse_sha256: temporal.map(|value| value.temporal_discourse_sha256.clone()),
            source_sha256: compilation.document.source_sha256.clone(),
            options: self.options.clone(),
            options_sha256: self.options.fingerprint(),
            proposition_occurrences,
            proposition_order,
            values,
            value_order,
            claims,
            claim_order,
            contradiction_sets,
            contradiction_order,
            diagnostics,
            summary,
            knowledge_sha256: String::new(),
        };
        artifact.knowledge_sha256 = document_knowledge_hash(&artifact)?;
        Ok(artifact)
    }
}

fn signature_hash(occurrence: &PropositionOccurrence) -> String {
    let value = json!({
        "subject": occurrence.subject,
        "predicate": occurrence.predicate,
        "object": occurrence.object,
        "factuality": occurrence.factuality,
        "world": occurrence.world,
        "attribution": occurrence.attribution,
        "temporal_scope": occurrence.temporal_scope,
        "qualifiers": occurrence.qualifiers,
    });
    let bytes = serde_json::to_vec(&value).unwrap_or_default();
    let mut hasher = sha2::Sha256::new();
    use sha2::Digest;
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn frame_fact(
    frame: &Frame,
    resolution: Option<&DocumentEntityResolution>,
) -> Option<(KnowledgeSubjectRef, KnowledgePredicateRef, KnowledgeObjectRef)> {
    let relation = |subject: &Entity, predicate: KnowledgeRelationKind, object: &Entity| {
        Some((
            entity_ref(subject, resolution)?,
            KnowledgePredicateRef::Relation(predicate),
            entity_object(object, resolution)?,
        ))
    };
    match frame {
        Frame::Statement { subject, property, .. } => relation(
            subject,
            KnowledgeRelationKind::HasProperty,
            property,
        ),
        Frame::Existence { entity, location: Some(location), .. } => relation(
            entity,
            KnowledgeRelationKind::LocatedAt,
            location,
        ),
        Frame::Possession { possessor, possessed, .. } => relation(
            possessor,
            KnowledgeRelationKind::Possesses,
            possessed,
        ),
        Frame::Motion { mover, goal: Some(goal), .. } => relation(
            mover,
            KnowledgeRelationKind::MovesTo,
            goal,
        ),
        Frame::Creation { creator, created, .. } => relation(
            creator,
            KnowledgeRelationKind::Creates,
            created,
        ),
        Frame::Destruction { agent, patient, .. } => relation(
            agent,
            KnowledgeRelationKind::Destroys,
            patient,
        ),
        Frame::Perception { experiencer, stimulus, .. } => relation(
            experiencer,
            KnowledgeRelationKind::Perceives,
            stimulus,
        ),
        Frame::Cognition { cognizer, content, .. } => relation(
            cognizer,
            KnowledgeRelationKind::Thinks,
            content,
        ),
        Frame::Emotion { experiencer, stimulus, .. } => relation(
            experiencer,
            KnowledgeRelationKind::Feels,
            stimulus,
        ),
        Frame::Communication { speaker, message, .. } => relation(
            speaker,
            KnowledgeRelationKind::Says,
            message,
        ),
        Frame::Consumption { agent, patient, .. } => relation(
            agent,
            KnowledgeRelationKind::Destroys,
            patient,
        ),
        Frame::Custom { name, roles } if matches!(name.as_str(), "CAPITAL_OF" | "LOCATED_IN" | "CREATED_BY" | "BORN_IN" | "WORKS_FOR") => {
            let subject = roles.iter().find(|(role, _)| matches!(role, SemanticRole::Topic | SemanticRole::Agent))?.1.clone();
            let object = roles.iter().find(|(role, _)| matches!(role, SemanticRole::Location | SemanticRole::Beneficiary | SemanticRole::Theme))?.1.clone();
            Some((
                entity_ref(&subject, resolution)?,
                KnowledgePredicateRef::Custom(name.clone()),
                entity_object(&object, resolution)?,
            ))
        }
        _ => None,
    }
}

fn functional_predicate(predicate: &KnowledgePredicateRef) -> bool {
    matches!(predicate,
        KnowledgePredicateRef::Custom(name) if matches!(name.as_str(), "CAPITAL_OF" | "POPULATION" | "DATE_OF")
    ) || matches!(predicate, KnowledgePredicateRef::Property(name) if matches!(name.as_str(), "POPULATION" | "DATE_OF"))
}

fn frame_evidence(graph: &DocumentGraph, sentence_id: &crate::document::SentenceId, frame: &Frame) -> Vec<String> {
    let names = frame
        .entities()
        .iter()
        .filter_map(|entity| entity.name.as_deref().map(str::to_lowercase))
        .collect::<std::collections::BTreeSet<_>>();
    let mut evidence = vec![format!("source_sentence:{sentence_id}"), format!("frame:{}", frame.frame_type_name())];
    for node_id in &graph.node_order {
        let Some(DocumentGraphNode::Mention(mention)) = graph.nodes.get(node_id) else { continue };
        if mention.source_sentence_id != *sentence_id { continue; }
        if mention.name.as_deref().map(str::to_lowercase).is_some_and(|name| names.contains(&name)) {
            evidence.push(format!("mention:{}", mention.id));
            for span in mention.anchor.spans() {
                evidence.push(format!("source_span:{}:{}", span.start, span.end));
            }
        }
    }
    evidence.sort();
    evidence.dedup();
    evidence
}

fn frame_source_spans(graph: &DocumentGraph, sentence_id: &crate::document::SentenceId, frame: &Frame) -> Vec<crate::document::span::SourceSpan> {
    let names = frame.entities().iter().filter_map(|entity| entity.name.as_deref().map(str::to_lowercase)).collect::<std::collections::BTreeSet<_>>();
    let mut spans = Vec::new();
    for node_id in &graph.node_order {
        let Some(DocumentGraphNode::Mention(mention)) = graph.nodes.get(node_id) else { continue };
        if mention.source_sentence_id == *sentence_id && mention.name.as_deref().map(str::to_lowercase).is_some_and(|name| names.contains(&name)) {
            spans.extend(mention.anchor.spans());
        }
    }
    spans.sort_by_key(|span| (span.start, span.end));
    spans.dedup();
    spans
}

fn entity_ref(entity: &Entity, resolution: Option<&DocumentEntityResolution>) -> Option<KnowledgeSubjectRef> {
    resolution.and_then(|resolution| find_cluster(entity, resolution))
        .map(KnowledgeSubjectRef::EntityCluster)
        .or_else(|| entity.name.as_ref().map(|_| KnowledgeSubjectRef::Unresolved))
}

fn entity_object(entity: &Entity, resolution: Option<&DocumentEntityResolution>) -> Option<KnowledgeObjectRef> {
    resolution.and_then(|resolution| find_cluster(entity, resolution))
        .map(KnowledgeObjectRef::EntityCluster)
        .or_else(|| entity.name.clone().map(KnowledgeObjectRef::TextLiteral))
}

fn find_cluster(entity: &Entity, resolution: &DocumentEntityResolution) -> Option<super::super::resolution::EntityClusterId> {
    resolution
        .mention_profiles
        .iter()
        .find(|(_, profile)| {
            // Two absent semantic IDs do not establish identity.  Without
            // this guard every unresolved entity matched the first profile
            // because `None == None`.
            (profile.semantic_entity_id.is_some()
                && entity.id.is_some()
                && profile.semantic_entity_id == entity.id)
                || (profile.concept == entity.concept
                    && profile.normalized_surface.as_deref()
                        == entity.name.as_deref().map(str::to_lowercase).as_deref())
        })
        .and_then(|(mention, _)| resolution.decisions.values().find(|decision| decision.mention == *mention))
        // `selected_cluster` is the legacy compatibility projection for an
        // antecedent selection and is intentionally empty for seeded
        // mentions.  Knowledge facts need the cluster produced for the
        // mention itself, which is carried by `result_cluster`.
        .and_then(|decision| decision.result_cluster.clone())
}
