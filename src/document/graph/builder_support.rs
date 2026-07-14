use crate::core::graph::LinguisticGraph;
use crate::core::interlingua::{Entity, Frame};
use crate::core::interlingua::SemanticRole;
use std::collections::BTreeMap;

use super::error::DocumentGraphBuildError;
use super::id::{GraphEdgeId, GraphNodeId};
use super::node_entity::{DocumentMentionNode, EntityCandidateKind, EntityCandidateNode, EntityCandidateState};
use super::edge::DocumentGraphEdge;
use super::model::DocumentGraphNode;

pub(crate) fn push_node(
    nodes: &mut BTreeMap<GraphNodeId, DocumentGraphNode>,
    order: &mut Vec<GraphNodeId>,
    node: DocumentGraphNode,
) -> Result<(), DocumentGraphBuildError> {
    let id = node.id().clone();
    if nodes.insert(id.clone(), node).is_some() {
        return Err(DocumentGraphBuildError::DuplicateNode { id });
    }
    order.push(id);
    Ok(())
}

pub(crate) fn push_edge(
    edges: &mut BTreeMap<GraphEdgeId, DocumentGraphEdge>,
    order: &mut Vec<GraphEdgeId>,
    edge: DocumentGraphEdge,
) -> Result<(), DocumentGraphBuildError> {
    let id = edge.id.clone();
    if edges.insert(id.clone(), edge).is_some() {
        return Err(DocumentGraphBuildError::DuplicateEdge { id });
    }
    order.push(id);
    Ok(())
}

pub(crate) fn frame_verb_concept(frame: &Frame) -> String {
    match frame {
        Frame::Transfer { verb_concept, .. }
        | Frame::Motion { verb_concept, .. }
        | Frame::Creation { verb_concept, .. }
        | Frame::Destruction { verb_concept, .. }
        | Frame::Perception { verb_concept, .. }
        | Frame::Cognition { verb_concept, .. }
        | Frame::Emotion { verb_concept, .. }
        | Frame::Communication { verb_concept, .. }
        | Frame::Statement { verb_concept, .. }
        | Frame::Existence { verb_concept, .. }
        | Frame::Possession { verb_concept, .. }
        | Frame::Consumption { verb_concept, .. } => verb_concept.clone(),
        Frame::Custom { name, .. } => name.clone(),
    }
}

pub(crate) fn frame_role_entities(frame: &Frame) -> Vec<(SemanticRole, &Entity)> {
    match frame {
        Frame::Transfer {
            agent,
            recipient,
            theme,
            ..
        } => vec![
            (SemanticRole::Agent, agent),
            (SemanticRole::Recipient, recipient),
            (SemanticRole::Theme, theme),
        ],
        Frame::Motion {
            mover,
            source,
            goal,
            path,
            ..
        } => {
            let mut roles = vec![(SemanticRole::Agent, mover)];
            if let Some(entity) = source {
                roles.push((SemanticRole::Source, entity));
            }
            if let Some(entity) = goal {
                roles.push((SemanticRole::Goal, entity));
            }
            if let Some(entity) = path {
                roles.push((SemanticRole::Location, entity));
            }
            roles
        }
        Frame::Creation {
            creator,
            created,
            material,
            ..
        } => {
            let mut roles = vec![
                (SemanticRole::Creator, creator),
                (SemanticRole::Created, created),
            ];
            if let Some(entity) = material {
                roles.push((SemanticRole::Instrument, entity));
            }
            roles
        }
        Frame::Destruction {
            agent,
            patient,
            instrument,
            ..
        } => {
            let mut roles = vec![
                (SemanticRole::Agent, agent),
                (SemanticRole::Patient, patient),
            ];
            if let Some(entity) = instrument {
                roles.push((SemanticRole::Instrument, entity));
            }
            roles
        }
        Frame::Perception {
            experiencer,
            stimulus,
            ..
        } => vec![
            (SemanticRole::Experiencer, experiencer),
            (SemanticRole::Stimulus, stimulus),
        ],
        Frame::Cognition {
            cognizer,
            content,
            ..
        } => vec![
            (SemanticRole::Cognizer, cognizer),
            (SemanticRole::Content, content),
        ],
        Frame::Emotion {
            experiencer,
            stimulus,
            ..
        } => vec![
            (SemanticRole::Experiencer, experiencer),
            (SemanticRole::Stimulus, stimulus),
        ],
        Frame::Communication {
            speaker,
            addressee,
            message,
            ..
        } => {
            let mut roles = vec![
                (SemanticRole::Speaker, speaker),
                (SemanticRole::Message, message),
            ];
            if let Some(entity) = addressee {
                roles.push((SemanticRole::Recipient, entity));
            }
            roles
        }
        Frame::Statement {
            subject,
            property,
            ..
        } => vec![
            (SemanticRole::Topic, subject),
            (SemanticRole::Theme, property),
        ],
        Frame::Existence { entity, location, .. } => {
            let mut roles = vec![(SemanticRole::Theme, entity)];
            if let Some(entity) = location {
                roles.push((SemanticRole::Location, entity));
            }
            roles
        }
        Frame::Possession {
            possessor,
            possessed,
            ..
        } => vec![
            (SemanticRole::Agent, possessor),
            (SemanticRole::Theme, possessed),
        ],
        Frame::Consumption { agent, patient, .. } => vec![
            (SemanticRole::Agent, agent),
            (SemanticRole::Patient, patient),
        ],
        Frame::Custom { roles, .. } => roles.iter().map(|(role, entity)| (*role, entity)).collect(),
    }
}

pub(crate) fn normalize_name(name: &str) -> String {
    let mut out = String::new();
    let mut last_was_space = false;
    for ch in name.trim().chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                out.push(' ');
                last_was_space = true;
            }
        } else {
            out.extend(ch.to_lowercase());
            last_was_space = false;
        }
    }
    out
}

pub(crate) fn construction_link(
    _construction: &crate::core::interlingua::ConstructionInstance,
    _graph: Option<&LinguisticGraph>,
) -> Option<GraphNodeId> {
    None
}

pub(crate) fn candidate_for_mention(mention: &DocumentMentionNode, kind: EntityCandidateKind) -> EntityCandidateNode {
    EntityCandidateNode {
        id: GraphNodeId::generated("placeholder".to_string()),
        candidate_kind: kind,
        state: EntityCandidateState::Introduced,
        canonical_name: mention.name.clone(),
        normalized_name: mention.normalized_name.clone(),
        primary_concept: mention.concept.clone(),
        features: mention.features.clone(),
        mention_ids: vec![],
        semantic_entity_ids: mention.semantic_entity_id.into_iter().collect(),
        reference_kinds: vec![mention.reference.clone()],
    }
}

pub(crate) fn entity_from_mention(mention: &DocumentMentionNode) -> Entity {
    Entity {
        concept: mention.concept.clone(),
        name: mention.name.clone(),
        features: mention.features.clone(),
        reference: mention.reference.clone(),
        id: mention.semantic_entity_id,
        coordination: None,
        adjectives: vec![],
    }
}

pub(crate) fn unresolved_reason(status: crate::document::compilation::SentenceProcessingStatus) -> String {
    match status {
        crate::document::compilation::SentenceProcessingStatus::Failed => {
            "sentence compilation failed".into()
        }
        crate::document::compilation::SentenceProcessingStatus::Unresolved => {
            "sentence analysis produced no usable frame".into()
        }
        _ => "sentence semantics are unavailable".into(),
    }
}
