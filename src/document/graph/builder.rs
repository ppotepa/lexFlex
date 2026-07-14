use crate::document::compilation::DocumentCompilation;
use crate::document::model::DocumentBlockKind;

use super::builder_support::{
    candidate_for_mention, construction_link, entity_from_mention, frame_verb_concept, push_edge,
    push_node, unresolved_reason,
};
use super::compatibility::{DefaultEntityCompatibility, EntityCompatibility};
use super::edge::{DocumentGraphEdge, DocumentGraphEdgeKind};
use super::error::DocumentGraphBuildError;
use super::frame_adapter::FrameGraphAdapter;
use super::hash::document_graph_hash;
use super::id::DocumentGraphIdFactory;
use super::mention_extractor::MentionExtractor;
use super::model::{
    DocumentBlockGraphKind, DocumentGraph, DocumentGraphNode,
};
use super::node_entity::{DocumentEventNode, EntityCandidateKind, EntityCandidateNode};
use super::node_semantic::{
    ConstructionOccurrenceNode, ConstructionOccurrenceSource, FrameOccurrenceNode,
    SemanticSentenceOccurrenceNode, UnresolvedFragmentNode,
};
use super::node_structural::{
    DocumentBlockGraphNode, DocumentParagraphGraphNode, DocumentRootNode, DocumentSourceSentenceNode,
};
use super::options::DocumentGraphBuildOptions;
use super::schema::DocumentGraphSchema;
use super::summary::summarize_document_graph;
use super::validation::DocumentGraphValidator;
use std::collections::BTreeMap;

pub struct DocumentGraphBuilder<C: EntityCompatibility = DefaultEntityCompatibility> {
    options: DocumentGraphBuildOptions,
    _compatibility: C,
}

impl Default for DocumentGraphBuilder<DefaultEntityCompatibility> {
    fn default() -> Self {
        Self {
            options: DocumentGraphBuildOptions::default(),
            _compatibility: DefaultEntityCompatibility,
        }
    }
}

impl<C: EntityCompatibility> DocumentGraphBuilder<C> {
    pub fn with_options(options: DocumentGraphBuildOptions) -> DocumentGraphBuilder<DefaultEntityCompatibility> {
        DocumentGraphBuilder {
            options,
            _compatibility: DefaultEntityCompatibility,
        }
    }

    pub fn build(&self, compilation: &DocumentCompilation) -> Result<DocumentGraph, DocumentGraphBuildError> {
        crate::document::compilation::DocumentCompilationValidator::validate(compilation).map_err(|errors| DocumentGraphBuildError::InvalidCompilation { errors })?;
        let options_sha256 = self.options.fingerprint();
        let graph_id = DocumentGraphIdFactory::graph(
            &compilation.document.id,
            DocumentGraphSchema::CURRENT,
            &options_sha256,
        );
        let mut nodes = BTreeMap::new();
        let mut edges = BTreeMap::new();
        let mut node_order = Vec::new();
        let mut edge_order = Vec::new();
        let diagnostics = Vec::new();
        let mut edge_ordinal = 0usize;

        push_node(
            &mut nodes,
            &mut node_order,
            DocumentGraphNode::Document(DocumentRootNode {
                id: DocumentGraphIdFactory::document_root(&graph_id),
                document_id: compilation.document.id.clone(),
                source_language: compilation.document.source_language.clone(),
                source_sha256: compilation.document.source_sha256.clone(),
                source_len_bytes: compilation.document.source().len(),
            }),
        )?;

        let root_id = DocumentGraphIdFactory::document_root(&graph_id);
        let mut included_blocks = Vec::new();
        for block_id in compilation.document.block_order() {
            let Some(block) = compilation.document.block(block_id) else { continue; };
            if !self.options.include_preserved_blocks && matches!(block.kind, DocumentBlockKind::PreservedWhitespace | DocumentBlockKind::PreservedRaw) {
                continue;
            }
            let node_id = DocumentGraphIdFactory::block(&graph_id, included_blocks.len());
            push_node(
                &mut nodes,
                &mut node_order,
                DocumentGraphNode::Block(DocumentBlockGraphNode {
                    id: node_id.clone(),
                    block_id: block.id.clone(),
                    ordinal: block.ordinal,
                    kind: match &block.kind {
                        DocumentBlockKind::Paragraph(_) => DocumentBlockGraphKind::Paragraph,
                        DocumentBlockKind::PreservedWhitespace => DocumentBlockGraphKind::PreservedWhitespace,
                        DocumentBlockKind::PreservedRaw => DocumentBlockGraphKind::PreservedRaw,
                    },
                    span: block.span.clone(),
                }),
            )?;
            push_edge(
                &mut edges,
                &mut edge_order,
                DocumentGraphEdge {
                    id: DocumentGraphIdFactory::edge(&graph_id, edge_ordinal),
                    from: root_id.clone(),
                    to: node_id.clone(),
                    kind: DocumentGraphEdgeKind::ContainsBlock,
                },
            )?;
            edge_ordinal += 1;
            included_blocks.push((block_id.clone(), node_id));
        }
        for pair in included_blocks.windows(2) {
            push_edge(
                &mut edges,
                &mut edge_order,
                DocumentGraphEdge {
                    id: DocumentGraphIdFactory::edge(&graph_id, edge_ordinal),
                    from: pair[0].1.clone(),
                    to: pair[1].1.clone(),
                    kind: DocumentGraphEdgeKind::NextBlock,
                },
            )?;
            edge_ordinal += 1;
        }

        let mut paragraph_nodes = BTreeMap::new();
        for (ordinal, paragraph) in compilation.document.ordered_paragraphs().into_iter().enumerate() {
            let node_id = DocumentGraphIdFactory::paragraph(&graph_id, ordinal);
            paragraph_nodes.insert(paragraph.id.clone(), node_id.clone());
            push_node(
                &mut nodes,
                &mut node_order,
                DocumentGraphNode::Paragraph(DocumentParagraphGraphNode {
                    id: node_id.clone(),
                    paragraph_id: paragraph.id.clone(),
                    ordinal,
                    block_id: paragraph.block_id.clone(),
                    span: paragraph.span.clone(),
                }),
            )?;
            if let Some(block_node) = included_blocks
                .iter()
                .find(|(id, _)| *id == paragraph.block_id)
                .map(|(_, node)| node.clone())
            {
                push_edge(
                    &mut edges,
                    &mut edge_order,
                    DocumentGraphEdge {
                        id: DocumentGraphIdFactory::edge(&graph_id, edge_ordinal),
                        from: block_node,
                        to: node_id.clone(),
                        kind: DocumentGraphEdgeKind::ContainsParagraph,
                    },
                )?;
                edge_ordinal += 1;
            }
        }
        for pair in compilation.document.ordered_paragraphs().windows(2) {
            let left = paragraph_nodes.get(&pair[0].id).cloned().unwrap();
            let right = paragraph_nodes.get(&pair[1].id).cloned().unwrap();
            push_edge(
                &mut edges,
                &mut edge_order,
                DocumentGraphEdge {
                    id: DocumentGraphIdFactory::edge(&graph_id, edge_ordinal),
                    from: left,
                    to: right,
                    kind: DocumentGraphEdgeKind::NextParagraph,
                },
            )?;
            edge_ordinal += 1;
        }

        let mut source_sentence_nodes = BTreeMap::new();
        for (ordinal, sentence) in compilation.document.ordered_sentences().into_iter().enumerate() {
            let node_id = DocumentGraphIdFactory::source_sentence(&graph_id, ordinal);
            source_sentence_nodes.insert(sentence.id.clone(), node_id.clone());
            let result = compilation.sentence_results.get(&sentence.id).ok_or_else(|| DocumentGraphBuildError::MissingSentenceResult {
                sentence_id: sentence.id.clone(),
            })?;
            push_node(
                &mut nodes,
                &mut node_order,
                DocumentGraphNode::SourceSentence(DocumentSourceSentenceNode {
                    id: node_id.clone(),
                    sentence_id: sentence.id.clone(),
                    paragraph_id: sentence.paragraph_id.clone(),
                    document_ordinal: sentence.document_ordinal,
                    paragraph_ordinal: sentence.paragraph_ordinal,
                    raw_span: sentence.raw_span.clone(),
                    content_span: sentence.content_span.clone(),
                    compilation_status: result.status,
                    semantic_sentence_count: result.semantics.as_ref().map(|u| u.sentences.len()).unwrap_or(0),
                    diagnostic_codes: result.diagnostics.iter().map(|diagnostic| diagnostic.code.clone()).collect(),
                }),
            )?;
            if let Some(paragraph_node) = paragraph_nodes.get(&sentence.paragraph_id) {
                push_edge(
                    &mut edges,
                    &mut edge_order,
                    DocumentGraphEdge {
                        id: DocumentGraphIdFactory::edge(&graph_id, edge_ordinal),
                        from: paragraph_node.clone(),
                        to: node_id.clone(),
                        kind: DocumentGraphEdgeKind::ContainsSourceSentence,
                    },
                )?;
                edge_ordinal += 1;
            }
        }
        for pair in compilation.document.ordered_sentences().windows(2) {
            let left = source_sentence_nodes.get(&pair[0].id).cloned().unwrap();
            let right = source_sentence_nodes.get(&pair[1].id).cloned().unwrap();
            push_edge(
                &mut edges,
                &mut edge_order,
                DocumentGraphEdge {
                    id: DocumentGraphIdFactory::edge(&graph_id, edge_ordinal),
                    from: left,
                    to: right,
                    kind: DocumentGraphEdgeKind::NextSourceSentence,
                },
            )?;
            edge_ordinal += 1;
        }

        let mut semantic_ordinal = 0usize;
        let mut frame_global_ordinal = 0usize;
        let mut mention_ordinal = 0usize;
        let mut candidate_ordinal = 0usize;
        let mut event_ordinal = 0usize;
        let mut construction_global_ordinal = 0usize;
        for sentence in compilation.document.ordered_sentences() {
            let Some(result) = compilation.sentence_results.get(&sentence.id) else { continue; };
            let source_sentence_id = source_sentence_nodes.get(&sentence.id).cloned().unwrap();
            if let Some(semantics) = result.semantics.as_ref() {
                for (semantic_sentence_ordinal, semantic_sentence) in semantics.sentences.iter().enumerate() {
                    let semantic_id = DocumentGraphIdFactory::semantic_sentence(&graph_id, sentence.document_ordinal, semantic_sentence_ordinal);
                    push_node(
                        &mut nodes,
                        &mut node_order,
                        DocumentGraphNode::SemanticSentence(SemanticSentenceOccurrenceNode {
                            id: semantic_id.clone(),
                            source_sentence_id: sentence.id.clone(),
                            semantic_ordinal: semantic_ordinal,
                            tense: semantic_sentence.tense,
                            aspect: semantic_sentence.aspect,
                            polarity: semantic_sentence.polarity,
                            modality: semantic_sentence.modality,
                            illocution: semantic_sentence.illocution,
                            voice: semantic_sentence.voice,
                            reflexive: semantic_sentence.reflexive,
                            temporal: semantic_sentence.temporal.clone(),
                            quantification: semantic_sentence.quantification.clone(),
                            construction_concepts: semantic_sentence.constructions.iter().map(|c| c.construction_concept.clone()).collect(),
                            linguistic_graph_present: semantic_sentence.graph.is_some(),
                            frame_count: semantic_sentence.frames.len(),
                            construction_count: semantic_sentence.constructions.len() + semantic_sentence.construction_concepts.len(),
                        }),
                    )?;
                    push_edge(&mut edges, &mut edge_order, DocumentGraphEdge {
                        id: DocumentGraphIdFactory::edge(&graph_id, edge_ordinal),
                        from: source_sentence_id.clone(),
                        to: semantic_id.clone(),
                        kind: DocumentGraphEdgeKind::HasSemanticSentence,
                    })?;
                    edge_ordinal += 1;

                    for (construction_ordinal, construction) in semantic_sentence.constructions.iter().enumerate() {
                        let node_id = DocumentGraphIdFactory::construction(&graph_id, construction_global_ordinal);
                        construction_global_ordinal += 1;
                        let linked = construction_link(construction, semantic_sentence.graph.as_ref());
                        push_node(
                            &mut nodes,
                            &mut node_order,
                            DocumentGraphNode::ConstructionOccurrence(ConstructionOccurrenceNode {
                                id: node_id.clone(),
                                source_sentence_id: sentence.id.clone(),
                                semantic_sentence_id: semantic_id.clone(),
                                construction_ordinal,
                                construction_concept: construction.construction_concept.clone(),
                                source: ConstructionOccurrenceSource::ExplicitConstruction,
                                inner_frame_type: Some(construction.inner.frame_type_name().to_string()),
                                inner_verb_concept: Some(frame_verb_concept(&construction.inner)),
                                linked_frame_id: linked,
                            }),
                        )?;
                        push_edge(&mut edges, &mut edge_order, DocumentGraphEdge {
                            id: DocumentGraphIdFactory::edge(&graph_id, edge_ordinal),
                            from: semantic_id.clone(),
                            to: node_id,
                            kind: DocumentGraphEdgeKind::HasConstruction,
                        })?;
                        edge_ordinal += 1;
                    }

                    for (frame_ordinal, frame) in semantic_sentence.frames.iter().enumerate() {
                        let frame_id = DocumentGraphIdFactory::frame(&graph_id, frame_global_ordinal);
                        let _binding = FrameGraphAdapter::new(semantic_sentence.graph.as_ref()).bind_frame(frame, frame_ordinal);
                        push_node(
                            &mut nodes,
                            &mut node_order,
                            DocumentGraphNode::FrameOccurrence(FrameOccurrenceNode {
                                id: frame_id.clone(),
                                source_sentence_id: sentence.id.clone(),
                                semantic_sentence_id: semantic_id.clone(),
                                semantic_sentence_ordinal,
                                frame_ordinal,
                                global_frame_ordinal: frame_global_ordinal,
                                frame_type: frame.frame_type_name().to_string(),
                                verb_concept: frame_verb_concept(frame),
                                required_roles: frame.required_roles(),
                                role_count: frame.required_roles().len(),
                                source_provenance_ids: result.provenance.steps.iter().map(|step| step.id.clone()).collect(),
                            }),
                        )?;
                        frame_global_ordinal += 1;
                        push_edge(&mut edges, &mut edge_order, DocumentGraphEdge {
                            id: DocumentGraphIdFactory::edge(&graph_id, edge_ordinal),
                            from: semantic_id.clone(),
                            to: frame_id.clone(),
                            kind: DocumentGraphEdgeKind::HasFrameOccurrence,
                        })?;
                        edge_ordinal += 1;
                        let mut mention_root_nodes = MentionExtractor::root_mentions(
                            &compilation.document,
                            sentence,
                            &graph_id,
                            semantic_id.clone(),
                            frame_id.clone(),
                            frame,
                            semantic_sentence_ordinal,
                            frame_ordinal,
                            semantic_sentence.graph.as_ref(),
                            &mut mention_ordinal,
                        );
                        for mention in mention_root_nodes.drain(..) {
                            let mention_id = mention.id.clone();
                            let candidate_id = DocumentGraphIdFactory::candidate(&graph_id, candidate_ordinal);
                            candidate_ordinal += 1;
                            let candidate = candidate_for_mention(&mention, EntityCandidateKind::Referent);
                            let candidate = EntityCandidateNode {
                                id: candidate_id.clone(),
                                candidate_kind: candidate.candidate_kind,
                                state: candidate.state,
                                canonical_name: candidate.canonical_name,
                                normalized_name: candidate.normalized_name,
                                primary_concept: candidate.primary_concept,
                                features: candidate.features,
                                mention_ids: vec![mention_id.clone()],
                                semantic_entity_ids: candidate.semantic_entity_ids,
                                reference_kinds: vec![mention.reference.clone()],
                            };
                            push_node(&mut nodes, &mut node_order, DocumentGraphNode::Mention(mention.clone()))?;
                            push_node(&mut nodes, &mut node_order, DocumentGraphNode::EntityCandidate(candidate.clone()))?;
                            push_edge(&mut edges, &mut edge_order, DocumentGraphEdge {
                                id: DocumentGraphIdFactory::edge(&graph_id, edge_ordinal),
                                from: frame_id.clone(),
                                to: mention_id.clone(),
                                kind: DocumentGraphEdgeKind::HasMention,
                            })?;
                            edge_ordinal += 1;
                            push_edge(&mut edges, &mut edge_order, DocumentGraphEdge {
                                id: DocumentGraphIdFactory::edge(&graph_id, edge_ordinal),
                                from: mention_id.clone(),
                                to: candidate_id.clone(),
                                kind: DocumentGraphEdgeKind::RefersToCandidate { primary: true },
                            })?;
                            edge_ordinal += 1;
                            for nested in MentionExtractor::nested_mentions(
                                &compilation.document,
                                sentence,
                                &graph_id,
                                semantic_id.clone(),
                                frame_id.clone(),
                                &mention,
                                &entity_from_mention(&mention),
                                semantic_sentence.graph.as_ref(),
                                &mut mention_ordinal,
                            ) {
                                let nested_id = nested.id.clone();
                                push_node(&mut nodes, &mut node_order, DocumentGraphNode::Mention(nested.clone()))?;
                                push_edge(&mut edges, &mut edge_order, DocumentGraphEdge {
                                    id: DocumentGraphIdFactory::edge(&graph_id, edge_ordinal),
                                    from: mention_id.clone(),
                                    to: nested_id,
                                    kind: DocumentGraphEdgeKind::CoordinationMember { ordinal: nested.nested_ordinal.unwrap_or(0) },
                                })?;
                                edge_ordinal += 1;
                            }
                        }

                        let event_id = DocumentGraphIdFactory::event(&graph_id, event_ordinal);
                        event_ordinal += 1;
                        push_node(
                            &mut nodes,
                            &mut node_order,
                            DocumentGraphNode::Event(DocumentEventNode {
                                id: event_id.clone(),
                                frame_occurrence_id: frame_id.clone(),
                                source_sentence_id: sentence.id.clone(),
                                semantic_sentence_id: semantic_id.clone(),
                                global_event_ordinal: event_ordinal - 1,
                                frame_type: frame.frame_type_name().to_string(),
                                verb_concept: frame_verb_concept(frame),
                                participant_count: frame.required_roles().len(),
                            }),
                        )?;
                        push_edge(&mut edges, &mut edge_order, DocumentGraphEdge {
                            id: DocumentGraphIdFactory::edge(&graph_id, edge_ordinal),
                            from: frame_id.clone(),
                            to: event_id.clone(),
                            kind: DocumentGraphEdgeKind::RepresentsEvent,
                        })?;
                        edge_ordinal += 1;
                    }
                }
            } else if self.options.create_unresolved_fragments {
                let unresolved_id = DocumentGraphIdFactory::unresolved(&graph_id, semantic_ordinal);
                semantic_ordinal += 1;
                push_node(
                    &mut nodes,
                    &mut node_order,
                    DocumentGraphNode::UnresolvedFragment(UnresolvedFragmentNode {
                        id: unresolved_id.clone(),
                        sentence_id: sentence.id.clone(),
                        status: result.status,
                        content_span: sentence.content_span.clone(),
                        diagnostic_codes: result.diagnostics.iter().map(|d| d.code.clone()).collect(),
                        reason: unresolved_reason(result.status),
                    }),
                )?;
                push_edge(&mut edges, &mut edge_order, DocumentGraphEdge {
                    id: DocumentGraphIdFactory::edge(&graph_id, edge_ordinal),
                    from: source_sentence_id.clone(),
                    to: unresolved_id,
                    kind: DocumentGraphEdgeKind::HasUnresolvedFragment,
                })?;
                edge_ordinal += 1;
            }
        }

        let source_diagnostics = compilation.diagnostics.clone();
        let summary = summarize_document_graph(&nodes, &edges, &source_diagnostics, &diagnostics);
        let mut graph = DocumentGraph {
            schema: DocumentGraphSchema::CURRENT,
            id: graph_id,
            source_document_id: compilation.document.id.clone(),
            source_language: compilation.document.source_language.clone(),
            source_sha256: compilation.document.source_sha256.clone(),
            compilation_sha256: compilation.compilation_sha256.clone(),
            build_options: self.options.clone(),
            options_sha256,
            nodes,
            edges,
            node_order,
            edge_order,
            source_diagnostics,
            diagnostics,
            summary,
            graph_sha256: String::new(),
        };
        graph.graph_sha256 = document_graph_hash(&graph)?;
        DocumentGraphValidator::validate(compilation, &graph, &self.options).map_err(|errors| DocumentGraphBuildError::InvalidFinalGraph { errors })?;
        Ok(graph)
    }
}
