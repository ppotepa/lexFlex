use crate::model::{
    CanonicalDocumentSemantics, CanonicalEntityMention, CanonicalFrame, CanonicalRole,
    CanonicalSentence, CanonicalDocumentGraphArtifacts, CanonicalDocumentResolutionArtifacts,
};
use lexflex::core::graph::{frame_role_entities, frame_verb_concept};
use lexflex::core::interlingua::{Entity, Frame, Sentence, Utterance};
use lexflex::document::compilation::DocumentCompilation;
use lexflex::document::graph::{DocumentGraph, DocumentGraphValidator, DocumentGraphBuildOptions};
use lexflex::document::resolution::DocumentEntityResolutionValidator;

pub fn canonicalize_utterance(utterance: &Utterance) -> CanonicalDocumentSemantics {
    let mut entities = Vec::new();
    let sentences = utterance
        .sentences
        .iter()
        .enumerate()
        .map(|(index, sentence)| canonical_sentence(index, sentence, &mut entities))
        .collect();
    let unresolved_count = entities
        .iter()
        .filter(|entity| entity.reference == "Unresolved")
        .count();
    CanonicalDocumentSemantics {
        sentences,
        entities,
        unresolved_count,
    }
}

pub fn canonicalize_document_graph(
    compilation: &DocumentCompilation,
    graph: &DocumentGraph,
    deterministic: bool,
) -> CanonicalDocumentGraphArtifacts {
    let valid = DocumentGraphValidator::validate(compilation, graph, &DocumentGraphBuildOptions::default()).is_ok();
    let dangling_edges = graph
        .edges
        .values()
        .filter(|edge| !graph.nodes.contains_key(&edge.from) || !graph.nodes.contains_key(&edge.to))
        .count();
    CanonicalDocumentGraphArtifacts {
        schema_version: graph.schema.schema_version,
        algorithm_version: graph.schema.algorithm_version,
        graph_sha256: graph.graph_sha256.clone(),
        build_success: true,
        valid,
        deterministic,
        nodes_total: graph.summary.nodes_total,
        edges_total: graph.summary.edges_total,
        block_nodes: graph.summary.block_nodes,
        paragraph_nodes: graph.summary.paragraph_nodes,
        source_sentence_nodes: graph.summary.source_sentence_nodes,
        semantic_sentence_nodes: graph.summary.semantic_sentence_nodes,
        frame_occurrence_nodes: graph.summary.frame_occurrence_nodes,
        mention_nodes: graph.summary.mention_nodes,
        unresolved_fragment_nodes: graph.summary.unresolved_fragment_nodes,
        dangling_edges,
        fatal_graph_diagnostics: graph.summary.graph_diagnostics_fatal,
    }
}

pub fn canonicalize_document_resolution(
    resolution: &lexflex::document::resolution::DocumentEntityResolution,
    deterministic: bool,
) -> CanonicalDocumentResolutionArtifacts {
    CanonicalDocumentResolutionArtifacts {
        schema_version: resolution.schema.schema_version,
        algorithm_version: resolution.schema.algorithm_version,
        resolution_sha256: resolution.resolution_sha256.clone(),
        valid: DocumentEntityResolutionValidator::validate(resolution).is_ok(),
        deterministic,
        mentions_total: resolution.summary.mentions_total,
        synthetic_mentions_total: resolution.summary.synthetic_mentions_total,
        decisions_total: resolution.summary.decisions_total,
        clusters_total: resolution.summary.clusters_total,
        accepted: resolution.summary.accepted,
        hard_accepted: resolution.summary.hard_accepted,
        ambiguous: resolution.summary.ambiguous,
        deferred: resolution.summary.deferred,
        unresolved: resolution.summary.unresolved,
        excluded: resolution.summary.excluded,
        diagnostics_info: resolution.summary.diagnostics_info,
        diagnostics_warning: resolution.summary.diagnostics_warning,
        diagnostics_error: resolution.summary.diagnostics_error,
        diagnostics_fatal: resolution.summary.diagnostics_fatal,
    }
}

fn canonical_sentence(
    index: usize,
    sentence: &Sentence,
    entities: &mut Vec<CanonicalEntityMention>,
) -> CanonicalSentence {
    let mut frames = sentence
        .frames
        .iter()
        .map(|frame| canonical_frame(frame, entities))
        .collect::<Vec<_>>();
    let mut constructions: Vec<String> = sentence
        .construction_concepts
        .iter()
        .map(|c| c.0.clone())
        .collect();
    constructions.sort();
    constructions.dedup();
    frames.sort_by(|a, b| a.frame_type.cmp(&b.frame_type).then(a.verb_concept.cmp(&b.verb_concept)));
    CanonicalSentence {
        index,
        frames,
        tense: sentence.tense.map(|t| format!("{t:?}")),
        aspect: sentence.aspect.map(|a| format!("{a:?}")),
        polarity: format!("{:?}", sentence.polarity),
        modality: sentence.modality.map(|m| format!("{m:?}")),
        illocution: format!("{:?}", sentence.illocution),
        voice: sentence.voice.map(|v| format!("{v:?}")),
        temporal: sentence.temporal.as_ref().map(|t| format!("{t:?}")),
        quantification: sentence.quantification.as_ref().map(|q| format!("{q:?}")),
        constructions,
    }
}

fn canonical_frame(frame: &Frame, entities: &mut Vec<CanonicalEntityMention>) -> CanonicalFrame {
    let mut roles = frame_role_entities(frame)
        .into_iter()
        .map(|(role, entity)| {
            let mention = canonical_entity(entity);
            entities.push(mention.clone());
            CanonicalRole {
                role: format!("{role:?}"),
                concept: entity.concept.0.clone(),
                name: entity.name.clone(),
                reference: mention.reference.clone(),
            }
        })
        .collect::<Vec<_>>();
    roles.sort_by(|a, b| a.role.cmp(&b.role).then(a.concept.cmp(&b.concept)));
    CanonicalFrame {
        frame_type: frame.frame_type_name().to_string(),
        verb_concept: frame_verb_concept(frame).to_string(),
        roles,
    }
}

fn canonical_entity(entity: &Entity) -> CanonicalEntityMention {
    CanonicalEntityMention {
        concept: entity.concept.0.clone(),
        name: entity.name.clone(),
        reference: format_reference(entity),
    }
}

fn format_reference(entity: &Entity) -> String {
    format!("{:?}", entity.reference)
}
