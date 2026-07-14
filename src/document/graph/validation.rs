use crate::document::compilation::DocumentCompilation;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use super::{document_graph_hash, summarize_document_graph, DocumentGraph, DocumentGraphBuildOptions, DocumentGraphNode};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentGraphValidationError {
    SchemaVersionMismatch,
    AlgorithmVersionMismatch,
    GraphIdMismatch,
    SourceDocumentIdMismatch,
    SourceLanguageMismatch,
    SourceHashMismatch,
    CompilationHashMismatch,
    OptionsHashMismatch,
    MissingDocumentRoot,
    MultipleDocumentRoots,
    NodeKeyMismatch,
    EdgeKeyMismatch,
    DuplicateNodeOrderId,
    DuplicateEdgeOrderId,
    NodeOrderCoverageMismatch,
    EdgeOrderCoverageMismatch,
    DanglingEdgeFrom,
    DanglingEdgeTo,
    MissingBlockNode,
    UnexpectedBlockNode,
    MissingParagraphNode,
    UnexpectedParagraphNode,
    MissingSourceSentenceNode,
    UnexpectedSourceSentenceNode,
    MissingSemanticSentenceNode,
    UnexpectedSemanticSentenceNode,
    MissingFrameOccurrence,
    UnexpectedFrameOccurrence,
    MissingMention,
    UnexpectedMention,
    MissingEntityCandidate,
    UnexpectedEntityCandidate,
    MissingEvent,
    UnexpectedEvent,
    MissingUnresolvedFragment,
    UnexpectedUnresolvedFragment,
    SummaryMismatch,
    GraphHashMismatch,
}

pub struct DocumentGraphValidator;

impl DocumentGraphValidator {
    pub fn validate(
        compilation: &DocumentCompilation,
        graph: &DocumentGraph,
        options: &DocumentGraphBuildOptions,
    ) -> Result<(), Vec<DocumentGraphValidationError>> {
        let mut errors = Vec::new();
        validate_metadata(compilation, graph, options, &mut errors);
        validate_keys(graph, &mut errors);
        validate_coverage(compilation, graph, options, &mut errors);
        validate_summary(graph, &mut errors);
        validate_hash(graph, &mut errors);
        sort_errors(&mut errors);
        errors.dedup();
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

pub fn validate_intrinsic(graph: &DocumentGraph) -> Result<(), Vec<DocumentGraphValidationError>> {
    let mut errors = Vec::new();
    validate_keys(graph, &mut errors);
    validate_summary(graph, &mut errors);
    validate_hash(graph, &mut errors);
    sort_errors(&mut errors);
    errors.dedup();
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

fn validate_metadata(
    compilation: &DocumentCompilation,
    graph: &DocumentGraph,
    options: &DocumentGraphBuildOptions,
    errors: &mut Vec<DocumentGraphValidationError>,
) {
    if graph.source_document_id != compilation.document.id {
        errors.push(DocumentGraphValidationError::SourceDocumentIdMismatch);
    }
    if graph.source_language != compilation.document.source_language {
        errors.push(DocumentGraphValidationError::SourceLanguageMismatch);
    }
    if graph.source_sha256 != compilation.document.source_sha256 {
        errors.push(DocumentGraphValidationError::SourceHashMismatch);
    }
    if graph.compilation_sha256 != compilation.compilation_sha256 {
        errors.push(DocumentGraphValidationError::CompilationHashMismatch);
    }
    if graph.build_options != *options {
        errors.push(DocumentGraphValidationError::OptionsHashMismatch);
    }
    if !graph.schema.is_supported() {
        errors.push(DocumentGraphValidationError::SchemaVersionMismatch);
    }
}

fn validate_keys(graph: &DocumentGraph, errors: &mut Vec<DocumentGraphValidationError>) {
    let mut node_ids = BTreeSet::new();
    for (key, node) in &graph.nodes {
        if key != node.id() {
            errors.push(DocumentGraphValidationError::NodeKeyMismatch);
        }
        if !node_ids.insert(key.clone()) {
            errors.push(DocumentGraphValidationError::DuplicateNodeOrderId);
        }
    }
    let mut edge_ids = BTreeSet::new();
    for (key, edge) in &graph.edges {
        if key != &edge.id {
            errors.push(DocumentGraphValidationError::EdgeKeyMismatch);
        }
        if !edge_ids.insert(key.clone()) {
            errors.push(DocumentGraphValidationError::DuplicateEdgeOrderId);
        }
    }
}

fn validate_coverage(
    compilation: &DocumentCompilation,
    graph: &DocumentGraph,
    options: &DocumentGraphBuildOptions,
    errors: &mut Vec<DocumentGraphValidationError>,
) {
    let block_count = compilation
        .document
        .block_order()
        .iter()
        .filter(|id| {
            compilation
                .document
                .block(id)
                .map(|block| options.include_preserved_blocks || matches!(block.kind, crate::document::DocumentBlockKind::Paragraph(_)))
                .unwrap_or(false)
        })
        .count();
    let paragraph_count = compilation.document.paragraphs().len();
    let sentence_count = compilation.document.sentences().len();

    let actual_blocks = graph.nodes.values().filter(|node| matches!(node, DocumentGraphNode::Block(_))).count();
    if actual_blocks != block_count {
        errors.push(DocumentGraphValidationError::MissingBlockNode);
    }
    let actual_paragraphs = graph.nodes.values().filter(|node| matches!(node, DocumentGraphNode::Paragraph(_))).count();
    if actual_paragraphs != paragraph_count {
        errors.push(DocumentGraphValidationError::MissingParagraphNode);
    }
    let actual_sentences = graph.nodes.values().filter(|node| matches!(node, DocumentGraphNode::SourceSentence(_))).count();
    if actual_sentences != sentence_count {
        errors.push(DocumentGraphValidationError::MissingSourceSentenceNode);
    }

    if graph.document_root().is_none() {
        errors.push(DocumentGraphValidationError::MissingDocumentRoot);
    }

    let expected_semantic_count: usize = compilation
        .ordered_results()
        .into_iter()
        .filter_map(|result| result.semantics.as_ref())
        .map(|utt| utt.sentences.len())
        .sum();
    let actual_semantic_count = graph
        .nodes
        .values()
        .filter(|node| matches!(node, DocumentGraphNode::SemanticSentence(_)))
        .count();
    if actual_semantic_count != expected_semantic_count {
        errors.push(DocumentGraphValidationError::MissingSemanticSentenceNode);
    }

    let expected_frame_count: usize = compilation
        .ordered_results()
        .into_iter()
        .filter_map(|result| result.semantics.as_ref())
        .map(|utt| utt.sentences.iter().map(|sentence| sentence.frames.len()).sum::<usize>())
        .sum();
    let actual_frame_count = graph
        .nodes
        .values()
        .filter(|node| matches!(node, DocumentGraphNode::FrameOccurrence(_)))
        .count();
    if actual_frame_count != expected_frame_count {
        errors.push(DocumentGraphValidationError::MissingFrameOccurrence);
    }
}

fn validate_summary(graph: &DocumentGraph, errors: &mut Vec<DocumentGraphValidationError>) {
    let expected = summarize_document_graph(&graph.nodes, &graph.edges, &graph.source_diagnostics, &graph.diagnostics);
    if graph.summary != expected {
        errors.push(DocumentGraphValidationError::SummaryMismatch);
    }
}

fn validate_hash(graph: &DocumentGraph, errors: &mut Vec<DocumentGraphValidationError>) {
    if let Ok(expected) = document_graph_hash(graph) {
        if expected != graph.graph_sha256 {
            errors.push(DocumentGraphValidationError::GraphHashMismatch);
        }
    }
}

fn sort_errors(errors: &mut [DocumentGraphValidationError]) {
    errors.sort_by_key(|error| format!("{error:?}"));
}
