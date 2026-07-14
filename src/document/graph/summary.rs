use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{DocumentGraphDiagnostic, DocumentGraphNode, DocumentGraphEdge, DocumentGraphNodeKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DocumentGraphSummary {
    pub nodes_total: usize,
    pub edges_total: usize,
    pub document_nodes: usize,
    pub block_nodes: usize,
    pub paragraph_nodes: usize,
    pub source_sentence_nodes: usize,
    pub semantic_sentence_nodes: usize,
    pub construction_nodes: usize,
    pub frame_occurrence_nodes: usize,
    pub mention_nodes: usize,
    pub role_root_mentions: usize,
    pub coordination_group_mentions: usize,
    pub coordination_member_mentions: usize,
    pub modifier_mentions: usize,
    pub entity_candidate_nodes: usize,
    pub group_candidates: usize,
    pub modifier_candidates: usize,
    pub ambiguous_candidates: usize,
    pub conflict_candidates: usize,
    pub unresolved_candidates: usize,
    pub event_nodes: usize,
    pub unresolved_fragment_nodes: usize,
    pub primary_candidate_edges: usize,
    pub event_participant_edges: usize,
    pub same_surface_edges: usize,
    pub possible_antecedent_edges: usize,
    pub source_diagnostics: usize,
    pub graph_diagnostics_info: usize,
    pub graph_diagnostics_warning: usize,
    pub graph_diagnostics_error: usize,
    pub graph_diagnostics_fatal: usize,
}

pub fn summarize_document_graph(
    nodes: &BTreeMap<super::id::GraphNodeId, DocumentGraphNode>,
    edges: &BTreeMap<super::id::GraphEdgeId, DocumentGraphEdge>,
    source_diagnostics: &[crate::document::DocumentDiagnostic],
    graph_diagnostics: &[DocumentGraphDiagnostic],
) -> DocumentGraphSummary {
    let mut summary = DocumentGraphSummary {
        nodes_total: nodes.len(),
        edges_total: edges.len(),
        source_diagnostics: source_diagnostics.len(),
        ..Default::default()
    };
    for node in nodes.values() {
        match node.kind() {
            DocumentGraphNodeKind::Document => summary.document_nodes += 1,
            DocumentGraphNodeKind::Block => summary.block_nodes += 1,
            DocumentGraphNodeKind::Paragraph => summary.paragraph_nodes += 1,
            DocumentGraphNodeKind::SourceSentence => summary.source_sentence_nodes += 1,
            DocumentGraphNodeKind::SemanticSentence => summary.semantic_sentence_nodes += 1,
            DocumentGraphNodeKind::ConstructionOccurrence => summary.construction_nodes += 1,
            DocumentGraphNodeKind::FrameOccurrence => summary.frame_occurrence_nodes += 1,
            DocumentGraphNodeKind::Mention => summary.mention_nodes += 1,
            DocumentGraphNodeKind::EntityCandidate => summary.entity_candidate_nodes += 1,
            DocumentGraphNodeKind::Event => summary.event_nodes += 1,
            DocumentGraphNodeKind::UnresolvedFragment => summary.unresolved_fragment_nodes += 1,
        }
    }
    for edge in edges.values() {
        use super::DocumentGraphEdgeKind::*;
        match edge.kind {
            RefersToCandidate { primary: true } => summary.primary_candidate_edges += 1,
            RefersToCandidate { primary: false } => {}
            EventParticipant { .. } => summary.event_participant_edges += 1,
            CandidateSameSurface { .. } => summary.same_surface_edges += 1,
            CandidatePossibleAntecedent { .. } => summary.possible_antecedent_edges += 1,
            _ => {}
        }
    }
    for diagnostic in graph_diagnostics {
        match diagnostic.severity {
            super::DocumentGraphDiagnosticSeverity::Info => summary.graph_diagnostics_info += 1,
            super::DocumentGraphDiagnosticSeverity::Warning => summary.graph_diagnostics_warning += 1,
            super::DocumentGraphDiagnosticSeverity::Error => summary.graph_diagnostics_error += 1,
            super::DocumentGraphDiagnosticSeverity::Fatal => summary.graph_diagnostics_fatal += 1,
        }
    }
    summary
}
