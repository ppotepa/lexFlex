//! Canonical construction concept IDs.
//! Source of truth: `data/concepts/concepts.ron`.

use crate::core::interlingua::{ConceptId, ConstructionInstance, Frame, Sentence};
use crate::core::graph::{EdgeKind, LinguisticGraph};

pub const IDENTIFICATION: &str = "IDENTIFICATION";
pub const DEMONSTRATIVE_REFERENCE: &str = "DEMONSTRATIVE_REFERENCE";
pub const AGE_IDIOM: &str = "AGE_IDIOM";
pub const ACCOMPANIMENT: &str = "ACCOMPANIMENT";
pub const TOPIC_CONTINUATION: &str = "TOPIC_CONTINUATION";
pub const ZERO_ANAPHORA: &str = "ZERO_ANAPHORA";
pub const CONTINUING_AGENT: &str = "CONTINUING_AGENT";
pub const CLAUSE: &str = "CLAUSE";

/// True when graph has a `PartOfConstruction` edge for this canonical concept ID.
pub fn graph_has_construction(graph: &LinguisticGraph, name: &str) -> bool {
    graph.edges.iter().any(|e| {
        matches!(
            &e.kind,
            EdgeKind::PartOfConstruction(c) if c == name
        )
    })
}

/// Record a construction concept on sentence metadata.
pub fn record_construction_concept(concepts: &mut Vec<ConceptId>, concept: &str) {
    if !concepts.iter().any(|c| c.0 == concept) {
        concepts.push(ConceptId::new(concept));
    }
}

/// Attach a construction concept edge on the graph.
pub fn attach_construction_edge(
    graph: &mut LinguisticGraph,
    entity_nid: crate::core::interlingua::NodeId,
    concept: &str,
) {
    graph.add_edge(
        entity_nid,
        entity_nid,
        EdgeKind::PartOfConstruction(concept.into()),
    );
}

/// Attach construction to both sentence metadata and graph (no overlapping borrows on sentence.graph).
pub fn attach_construction(
    construction_concepts: &mut Vec<ConceptId>,
    graph: &mut LinguisticGraph,
    entity_nid: crate::core::interlingua::NodeId,
    concept: &str,
) {
    record_construction_concept(construction_concepts, concept);
    attach_construction_edge(graph, entity_nid, concept);
}

/// Detect primary construction concept for a frame from graph edges and frame shape.
pub fn primary_construction_for_frame(
    graph: Option<&LinguisticGraph>,
    frame: &Frame,
) -> &'static str {
    if let Some(g) = graph {
        if graph_has_construction(g, ZERO_ANAPHORA) {
            return ZERO_ANAPHORA;
        }
        if graph_has_construction(g, IDENTIFICATION) {
            return IDENTIFICATION;
        }
        if graph_has_construction(g, DEMONSTRATIVE_REFERENCE) {
            return DEMONSTRATIVE_REFERENCE;
        }
        if graph_has_construction(g, AGE_IDIOM) {
            return AGE_IDIOM;
        }
        if graph_has_construction(g, ACCOMPANIMENT) {
            return ACCOMPANIMENT;
        }
        if graph_has_construction(g, TOPIC_CONTINUATION) {
            return TOPIC_CONTINUATION;
        }
    }
    match frame {
        Frame::Existence { location, verb_concept, .. }
            if location.is_none() && (verb_concept == "BE" || verb_concept == "IDENTITY") =>
        {
            IDENTIFICATION
        }
        Frame::Statement { verb_concept, .. } if verb_concept == "BE" => IDENTIFICATION,
        _ => CLAUSE,
    }
}

/// Build the IL construction tree from frames and graph data.
pub fn build_construction_tree(sentence: &mut Sentence) {
    let graph = sentence.graph.as_ref();
    sentence.constructions = sentence
        .frames
        .iter()
        .map(|frame| ConstructionInstance {
            construction_concept: ConceptId::new(primary_construction_for_frame(graph, frame)),
            inner: frame.clone(),
        })
        .collect();
    sync_frames_from_constructions(sentence);
}

pub fn sync_frames_from_constructions(sentence: &mut Sentence) {
    if !sentence.constructions.is_empty() {
        sentence.frames = sentence
            .constructions
            .iter()
            .map(|c| c.inner.clone())
            .collect();
    }
}
