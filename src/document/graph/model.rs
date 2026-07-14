use crate::document::id::{DocumentId, SentenceId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{
    DocumentBlockGraphNode, DocumentEventNode, DocumentGraphDiagnostic, DocumentGraphEdge,
    DocumentGraphSummary, DocumentMentionNode, DocumentParagraphGraphNode, DocumentRootNode,
    DocumentSourceSentenceNode, EntityCandidateNode, FrameOccurrenceNode, GraphEdgeId,
    GraphNodeId, ConstructionOccurrenceNode, SemanticSentenceOccurrenceNode, UnresolvedFragmentNode,
    DocumentGraphSchema, DocumentGraphBuildOptions,
};

pub use super::schema::{DOCUMENT_GRAPH_ALGORITHM_VERSION, DOCUMENT_GRAPH_SCHEMA_VERSION};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum DocumentBlockGraphKind {
    Paragraph,
    PreservedWhitespace,
    PreservedRaw,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum DocumentGraphNode {
    Document(DocumentRootNode),
    Block(DocumentBlockGraphNode),
    Paragraph(DocumentParagraphGraphNode),
    SourceSentence(DocumentSourceSentenceNode),
    SemanticSentence(SemanticSentenceOccurrenceNode),
    ConstructionOccurrence(ConstructionOccurrenceNode),
    FrameOccurrence(FrameOccurrenceNode),
    Mention(DocumentMentionNode),
    EntityCandidate(EntityCandidateNode),
    Event(DocumentEventNode),
    UnresolvedFragment(UnresolvedFragmentNode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentGraphNodeKind {
    Document,
    Block,
    Paragraph,
    SourceSentence,
    SemanticSentence,
    ConstructionOccurrence,
    FrameOccurrence,
    Mention,
    EntityCandidate,
    Event,
    UnresolvedFragment,
}

impl DocumentGraphNode {
    pub fn id(&self) -> &GraphNodeId {
        match self {
            Self::Document(node) => &node.id,
            Self::Block(node) => &node.id,
            Self::Paragraph(node) => &node.id,
            Self::SourceSentence(node) => &node.id,
            Self::SemanticSentence(node) => &node.id,
            Self::ConstructionOccurrence(node) => &node.id,
            Self::FrameOccurrence(node) => &node.id,
            Self::Mention(node) => &node.id,
            Self::EntityCandidate(node) => &node.id,
            Self::Event(node) => &node.id,
            Self::UnresolvedFragment(node) => &node.id,
        }
    }

    pub fn kind(&self) -> DocumentGraphNodeKind {
        match self {
            Self::Document(_) => DocumentGraphNodeKind::Document,
            Self::Block(_) => DocumentGraphNodeKind::Block,
            Self::Paragraph(_) => DocumentGraphNodeKind::Paragraph,
            Self::SourceSentence(_) => DocumentGraphNodeKind::SourceSentence,
            Self::SemanticSentence(_) => DocumentGraphNodeKind::SemanticSentence,
            Self::ConstructionOccurrence(_) => DocumentGraphNodeKind::ConstructionOccurrence,
            Self::FrameOccurrence(_) => DocumentGraphNodeKind::FrameOccurrence,
            Self::Mention(_) => DocumentGraphNodeKind::Mention,
            Self::EntityCandidate(_) => DocumentGraphNodeKind::EntityCandidate,
            Self::Event(_) => DocumentGraphNodeKind::Event,
            Self::UnresolvedFragment(_) => DocumentGraphNodeKind::UnresolvedFragment,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentGraph {
    pub schema: DocumentGraphSchema,
    pub id: super::GraphId,
    pub source_document_id: DocumentId,
    pub source_language: crate::core::interlingua::LanguageId,
    pub source_sha256: String,
    pub compilation_sha256: String,
    pub build_options: DocumentGraphBuildOptions,
    pub options_sha256: String,
    pub nodes: BTreeMap<GraphNodeId, DocumentGraphNode>,
    pub edges: BTreeMap<GraphEdgeId, DocumentGraphEdge>,
    pub node_order: Vec<GraphNodeId>,
    pub edge_order: Vec<GraphEdgeId>,
    pub source_diagnostics: Vec<crate::document::DocumentDiagnostic>,
    pub diagnostics: Vec<DocumentGraphDiagnostic>,
    pub summary: DocumentGraphSummary,
    pub graph_sha256: String,
}

impl DocumentGraph {
    pub fn document_root(&self) -> Option<&DocumentRootNode> {
        self.nodes.values().find_map(|node| match node {
            DocumentGraphNode::Document(root) => Some(root),
            _ => None,
        })
    }

    pub fn node(&self, id: &GraphNodeId) -> Option<&DocumentGraphNode> {
        self.nodes.get(id)
    }

    pub fn source_sentence_node(&self, sentence_id: &SentenceId) -> Option<&DocumentSourceSentenceNode> {
        self.nodes.values().find_map(|node| match node {
            DocumentGraphNode::SourceSentence(value) if &value.sentence_id == sentence_id => Some(value),
            _ => None,
        })
    }

    pub fn semantic_sentences_for_source(&self, sentence_id: &SentenceId) -> Vec<&SemanticSentenceOccurrenceNode> {
        self.nodes
            .values()
            .filter_map(|node| match node {
                DocumentGraphNode::SemanticSentence(value) if &value.source_sentence_id == sentence_id => Some(value),
                _ => None,
            })
            .collect()
    }

    pub fn frames_for_source(&self, sentence_id: &SentenceId) -> Vec<&FrameOccurrenceNode> {
        self.nodes
            .values()
            .filter_map(|node| match node {
                DocumentGraphNode::FrameOccurrence(value) if &value.source_sentence_id == sentence_id => Some(value),
                _ => None,
            })
            .collect()
    }

    pub fn mentions_for_source(&self, sentence_id: &SentenceId) -> Vec<&DocumentMentionNode> {
        self.nodes
            .values()
            .filter_map(|node| match node {
                DocumentGraphNode::Mention(value) if &value.source_sentence_id == sentence_id => Some(value),
                _ => None,
            })
            .collect()
    }

    pub fn candidates_for_source(&self, sentence_id: &SentenceId) -> Vec<&EntityCandidateNode> {
        self.nodes
            .values()
            .filter_map(|node| match node {
                DocumentGraphNode::EntityCandidate(value) if value.mention_ids.iter().any(|id| {
                    self.nodes.get(id).and_then(|node| match node {
                        DocumentGraphNode::Mention(mention) => Some(&mention.source_sentence_id),
                        _ => None,
                    }) == Some(sentence_id)
                }) => Some(value),
                _ => None,
            })
            .collect()
    }

    pub fn events_for_source(&self, sentence_id: &SentenceId) -> Vec<&DocumentEventNode> {
        self.nodes
            .values()
            .filter_map(|node| match node {
                DocumentGraphNode::Event(value) if &value.source_sentence_id == sentence_id => Some(value),
                _ => None,
            })
            .collect()
    }

    pub fn unresolved_fragment(&self, sentence_id: &SentenceId) -> Option<&UnresolvedFragmentNode> {
        self.nodes.values().find_map(|node| match node {
            DocumentGraphNode::UnresolvedFragment(value) if &value.sentence_id == sentence_id => Some(value),
            _ => None,
        })
    }

    pub fn outgoing(&self, node_id: &GraphNodeId) -> Vec<&DocumentGraphEdge> {
        self.edge_order
            .iter()
            .filter_map(|id| self.edges.get(id))
            .filter(|edge| &edge.from == node_id)
            .collect()
    }

    pub fn incoming(&self, node_id: &GraphNodeId) -> Vec<&DocumentGraphEdge> {
        self.edge_order
            .iter()
            .filter_map(|id| self.edges.get(id))
            .filter(|edge| &edge.to == node_id)
            .collect()
    }

    pub fn to_canonical_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(text: &str) -> Result<Self, super::DocumentGraphSerializationError> {
        let graph: Self = serde_json::from_str(text)?;
        if !graph.schema.is_supported() {
            return Err(super::DocumentGraphSerializationError::UnsupportedSchema);
        }
        let expected = super::document_graph_hash(&graph)?;
        if expected != graph.graph_sha256 {
            return Err(super::DocumentGraphSerializationError::HashMismatch);
        }
        super::validation::validate_intrinsic(&graph)
            .map_err(|_| super::DocumentGraphSerializationError::InvalidIntrinsicGraph)?;
        Ok(graph)
    }
}
