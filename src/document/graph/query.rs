use crate::core::interlingua::SemanticRole;
use crate::document::id::{ParagraphId, SentenceId};

use super::{
    DocumentEventNode, DocumentGraph, DocumentGraphEdge, DocumentMentionNode, DocumentParagraphGraphNode,
    DocumentRootNode, DocumentSourceSentenceNode, EntityCandidateNode, FrameOccurrenceNode,
    GraphNodeId, SemanticSentenceOccurrenceNode, UnresolvedFragmentNode,
};

pub struct DocumentGraphQuery<'a> {
    graph: &'a DocumentGraph,
}

pub struct EventParticipantView<'a> {
    pub event: &'a DocumentEventNode,
    pub candidate: &'a EntityCandidateNode,
    pub mention: &'a DocumentMentionNode,
    pub role: SemanticRole,
    pub role_ordinal: usize,
}

impl<'a> DocumentGraphQuery<'a> {
    pub fn new(graph: &'a DocumentGraph) -> Self {
        Self { graph }
    }

    pub fn document_root(&self) -> Option<&'a DocumentRootNode> {
        self.graph.document_root()
    }

    pub fn paragraph_node(&self, paragraph_id: &ParagraphId) -> Option<&'a DocumentParagraphGraphNode> {
        self.graph.nodes.values().find_map(|node| match node {
            super::DocumentGraphNode::Paragraph(value) if &value.paragraph_id == paragraph_id => Some(value),
            _ => None,
        })
    }

    pub fn source_sentence_node(&self, sentence_id: &SentenceId) -> Option<&'a DocumentSourceSentenceNode> {
        self.graph.source_sentence_node(sentence_id)
    }

    pub fn semantic_sentences_in_source(&self, sentence_id: &SentenceId) -> Vec<&'a SemanticSentenceOccurrenceNode> {
        self.graph.semantic_sentences_for_source(sentence_id)
    }

    pub fn frames_in_sentence(&self, sentence_id: &SentenceId) -> Vec<&'a FrameOccurrenceNode> {
        self.graph.frames_for_source(sentence_id)
    }

    pub fn mentions_in_sentence(&self, sentence_id: &SentenceId) -> Vec<&'a DocumentMentionNode> {
        self.graph.mentions_for_source(sentence_id)
    }

    pub fn unresolved_fragment(&self, sentence_id: &SentenceId) -> Option<&'a UnresolvedFragmentNode> {
        self.graph.unresolved_fragment(sentence_id)
    }

    pub fn outgoing(&self, node_id: &GraphNodeId) -> Vec<&'a DocumentGraphEdge> {
        self.graph.outgoing(node_id)
    }

    pub fn incoming(&self, node_id: &GraphNodeId) -> Vec<&'a DocumentGraphEdge> {
        self.graph.incoming(node_id)
    }
}
