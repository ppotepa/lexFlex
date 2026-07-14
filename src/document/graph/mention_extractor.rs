use crate::core::graph::LinguisticGraph;
use crate::core::interlingua::{Entity, Frame};
use crate::document::model::{Document, DocumentSentence};

use super::anchor::{anchor_from_realizing_words, sentence_scoped_anchor, MentionAnchorSource};
use super::builder_support::{frame_role_entities, normalize_name};
use super::id::{DocumentGraphIdFactory, GraphId, GraphNodeId};
use super::mention_path::{mention_path_string, MentionPathSegment};
use super::node_entity::{DocumentMentionKind, DocumentMentionNode};

pub struct MentionExtractor;

impl MentionExtractor {
    pub fn root_mentions(
        document: &Document,
        sentence: &DocumentSentence,
        graph_id: &GraphId,
        semantic_sentence_id: GraphNodeId,
        frame_occurrence_id: GraphNodeId,
        frame: &Frame,
        semantic_sentence_ordinal: usize,
        frame_ordinal: usize,
        graph: Option<&LinguisticGraph>,
        mention_ordinal: &mut usize,
    ) -> Vec<DocumentMentionNode> {
        let mut mentions = Vec::new();
        for (role_ordinal, (role, entity)) in frame_role_entities(frame).into_iter().enumerate() {
            let path = mention_path_string(&[
                MentionPathSegment::Frame {
                    semantic_sentence_ordinal,
                    frame_ordinal,
                },
                MentionPathSegment::Role {
                    role,
                    occurrence: role_ordinal,
                },
            ]);
            let (anchor, anchor_source) = match graph
                .and_then(|g| find_graph_entity_node(g, entity).map(|node_id| (g, node_id)))
            {
                Some((g, node_id)) => match anchor_from_realizing_words(document, sentence, g, node_id) {
                    Ok(anchor) => (anchor, MentionAnchorSource::LinguisticRealizingWords),
                    Err(_) => (
                        sentence_scoped_anchor(sentence),
                        MentionAnchorSource::SentenceFallbackNoRealizingWords,
                    ),
                },
                None => match graph {
                    Some(_) => (
                        sentence_scoped_anchor(sentence),
                        MentionAnchorSource::SentenceFallbackNoEntityBinding,
                    ),
                    None => (
                        sentence_scoped_anchor(sentence),
                        MentionAnchorSource::SentenceFallbackNoGraph,
                    ),
                },
            };
            let mention_id = DocumentGraphIdFactory::mention(graph_id, *mention_ordinal);
            *mention_ordinal += 1;
            mentions.push(DocumentMentionNode {
                id: mention_id,
                source_sentence_id: sentence.id.clone(),
                semantic_sentence_id: semantic_sentence_id.clone(),
                frame_occurrence_id: frame_occurrence_id.clone(),
                mention_kind: DocumentMentionKind::RoleRoot,
                role_context: role,
                role_ordinal,
                concept: entity.concept.clone(),
                name: entity.name.clone(),
                normalized_name: entity.name.as_deref().map(normalize_name),
                features: entity.features.clone(),
                reference: entity.reference.clone(),
                semantic_entity_id: entity.id,
                anchor,
                anchor_source,
                entity_path: path,
                parent_mention_id: None,
                nested_ordinal: None,
            });
        }
        mentions
    }

    pub fn nested_mentions(
        document: &Document,
        sentence: &DocumentSentence,
        graph_id: &GraphId,
        semantic_sentence_id: GraphNodeId,
        frame_occurrence_id: GraphNodeId,
        root: &DocumentMentionNode,
        entity: &Entity,
        graph: Option<&LinguisticGraph>,
        mention_ordinal: &mut usize,
    ) -> Vec<DocumentMentionNode> {
        let mut mentions = Vec::new();
        if let Some(coordination) = &entity.coordination {
            for (ordinal, item) in coordination.items.iter().enumerate() {
                let id = DocumentGraphIdFactory::mention(graph_id, *mention_ordinal);
                *mention_ordinal += 1;
                let path = format!("{}/coord-{ordinal:04}", root.entity_path);
                mentions.push(DocumentMentionNode {
                    id,
                    source_sentence_id: sentence.id.clone(),
                    semantic_sentence_id: semantic_sentence_id.clone(),
                    frame_occurrence_id: frame_occurrence_id.clone(),
                    mention_kind: DocumentMentionKind::CoordinationMember,
                    role_context: root.role_context,
                    role_ordinal: root.role_ordinal,
                    concept: item.concept.clone(),
                    name: item.name.clone(),
                    normalized_name: item.name.as_deref().map(normalize_name),
                    features: item.features.clone(),
                    reference: item.reference.clone(),
                    semantic_entity_id: item.id,
                    anchor: root.anchor.clone(),
                    anchor_source: MentionAnchorSource::ParentMentionInherited,
                    entity_path: path,
                    parent_mention_id: Some(root.id.clone()),
                    nested_ordinal: Some(ordinal),
                });
            }
        }
        for (ordinal, adjective) in entity.adjectives.iter().enumerate() {
            let id = DocumentGraphIdFactory::mention(graph_id, *mention_ordinal);
            *mention_ordinal += 1;
            let path = format!("{}/mod-{ordinal:04}", root.entity_path);
            let (anchor, anchor_source) = match graph
                .and_then(|g| find_graph_entity_node(g, adjective).map(|node_id| (g, node_id)))
            {
                Some((g, node_id)) => match anchor_from_realizing_words(document, sentence, g, node_id) {
                    Ok(anchor) => (anchor, MentionAnchorSource::LinguisticRealizingWords),
                    Err(_) => (root.anchor.clone(), MentionAnchorSource::ParentMentionInherited),
                },
                None => (root.anchor.clone(), MentionAnchorSource::ParentMentionInherited),
            };
            mentions.push(DocumentMentionNode {
                id,
                source_sentence_id: sentence.id.clone(),
                semantic_sentence_id: semantic_sentence_id.clone(),
                frame_occurrence_id: frame_occurrence_id.clone(),
                mention_kind: DocumentMentionKind::Modifier,
                role_context: root.role_context,
                role_ordinal: root.role_ordinal,
                concept: adjective.concept.clone(),
                name: adjective.name.clone(),
                normalized_name: adjective.name.as_deref().map(normalize_name),
                features: adjective.features.clone(),
                reference: adjective.reference.clone(),
                semantic_entity_id: adjective.id,
                anchor,
                anchor_source,
                entity_path: path,
                parent_mention_id: Some(root.id.clone()),
                nested_ordinal: Some(ordinal),
            });
        }
        mentions
    }
}

fn find_graph_entity_node(graph: &LinguisticGraph, entity: &Entity) -> Option<crate::core::interlingua::NodeId> {
    graph.nodes.iter().find_map(|node| match node {
        crate::core::graph::GraphNode::Entity(value)
            if value.concept == entity.concept && value.name == entity.name =>
        {
            Some(value.id)
        }
        _ => None,
    })
}
