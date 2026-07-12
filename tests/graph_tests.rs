use lexflex::api::LexFlexAPI;
use lexflex::core::graph::{self, EdgeKind, GraphNode, LinguisticGraph};
use lexflex::core::interlingua::*;

fn build_api() -> LexFlexAPI {
    LexFlexAPI::builder()
        .data_dir("data")
        .build()
        .expect("Failed to initialize lexFlex")
}

fn graph_from_parse(sentence: &str, lang: &str) -> LinguisticGraph {
    let api = build_api();
    let il = api.parse(sentence, lang).expect("parse failed");
    let utt = il.as_natural().expect("natural utterance");
    utt.sentences[0]
        .graph
        .clone()
        .expect("graph must be attached after parse")
}

fn token_count(sentence: &str) -> usize {
    sentence.split_whitespace().count()
}

// ─── Surface graph structure (Phase 1) ───────────────────────────────────────

#[test]
fn test_pl_surface_graph_word_count_matches_tokens() {
    let input = "Tomek mieszka z żoną i córką.";
    let graph = graph_from_parse(input, "pl");
    assert_eq!(graph.word_count(), token_count(input));
    assert!(graph.validate_linear_chain(token_count(input)));
}

#[test]
fn test_en_surface_graph_word_count_matches_tokens() {
    let input = "I live with a wife and a daughter.";
    let graph = graph_from_parse(input, "en");
    assert_eq!(graph.word_count(), token_count(input));
    assert!(graph.validate_linear_chain(token_count(input)));
}

#[test]
fn test_surface_graph_linear_order() {
    let api = build_api();
    for (input, lang) in [
        ("Tomek ma kota.", "pl"),
        ("Tom has a cat.", "en"),
    ] {
        let il = api.parse(input, lang).unwrap();
        let g = il.as_natural().unwrap().sentences[0].graph.as_ref().unwrap();
        let words: Vec<_> = g.word_nodes().collect();
        assert_eq!(words.len(), token_count(input));
        for w in &words[..words.len().saturating_sub(1)] {
            assert!(w.next.is_some(), "missing next edge for {}", w.form);
            assert!(g.prev_word(w.next.unwrap(), 1).is_some());
        }
    }
}

// ─── Semantic edges (Phase 2) ──────────────────────────────────────────────────

#[test]
fn test_pl_existence_has_realizes_and_has_role() {
    let graph = graph_from_parse("Tomek mieszka z żoną.", "pl");
    assert!(graph.has_edge_kind(&EdgeKind::Realizes));
    assert!(graph.edges.iter().any(|e| matches!(e.kind, EdgeKind::HasRole(_))));
}

#[test]
fn test_en_existence_has_realizes_and_has_role() {
    let graph = graph_from_parse("I live with a wife.", "en");
    assert!(graph.has_edge_kind(&EdgeKind::Realizes));
    assert!(graph.edges.iter().any(|e| matches!(e.kind, EdgeKind::HasRole(_))));
}

#[test]
fn test_pl_accompaniment_coordination_in_graph() {
    let graph = graph_from_parse("Tomek mieszka z żoną i córką.", "pl");
    assert!(
        graph.has_edge_kind(&EdgeKind::CoordinatesWith)
            || graph.edges_of_kind(&EdgeKind::Realizes).len() >= 3,
        "coordination or multiple realizes expected"
    );
}

// ─── Query / navigation (Phase 5 primitives) ─────────────────────────────────

#[test]
fn test_graph_next_prev_traversal() {
    let graph = graph_from_parse("Tom has a cat.", "en");
    let first = graph.word_nodes().next().unwrap().id;
    let second = graph.next_word(first, 1).unwrap().id;
    assert_eq!(graph.prev_word(second, 1).unwrap().id, first);
}

#[test]
fn test_graph_entity_realizing_words_lookup() {
    let graph = graph_from_parse("Tomek ma kota.", "pl");
    let entity_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter_map(|n| match n {
            lexflex::core::graph::GraphNode::Entity(e) => Some(e),
            _ => None,
        })
        .collect();
    let cat_entity = entity_nodes
        .iter()
        .find(|e| e.concept.0 == "CAT" || e.name.as_deref() == Some("kot"))
        .expect("CAT entity node");
    let realizing = graph.realizing_words_for_entity(cat_entity.id);
    assert!(!realizing.is_empty());
}

#[test]
fn test_graph_find_verbs_and_instrumental_feature() {
    let graph = graph_from_parse("Tomek mieszka z żoną.", "pl");
    let verbs = graph.find_verbs();
    assert!(!verbs.is_empty());
    let instrumental = graph.find_words_with_feature(|f| f.case == Some(Case::Instrumental));
    assert!(!instrumental.is_empty(), "instrumental case on accompaniment NP");
}

/// Assert graph FrameNode verb_concept and entity concepts match the final IL frame.
fn assert_graph_il_frame_sync(sentence: &Sentence) {
    let graph = sentence.graph.as_ref().expect("graph must exist");
    let frame = sentence.frames.first().expect("frame must exist");

    let frame_node = graph
        .nodes
        .iter()
        .find_map(|n| match n {
            GraphNode::Frame(f) => Some(f),
            _ => None,
        })
        .expect("FrameNode must exist");

    let il_verb = graph::frame_verb_concept(frame);

    assert_eq!(
        frame_node.verb_concept.0, il_verb,
        "graph FrameNode verb_concept must match IL frame"
    );

    for entity in frame.entities() {
        let in_graph = graph.nodes.iter().any(|n| match n {
            GraphNode::Entity(e) => e.concept == entity.concept && e.name == entity.name,
            _ => false,
        });
        assert!(
            in_graph,
            "IL entity {:?}/{:?} must have matching EntityNode in graph",
            entity.concept.0,
            entity.name
        );
    }
}

#[test]
fn test_graph_frame_matches_il_after_age_idiom() {
    let api = build_api();
    let il = api.parse("Mam 27 lat.", "pl").expect("parse");
    let sentence = &il.as_natural().expect("natural").sentences[0];

    match &sentence.frames[0] {
        Frame::Possession { verb_concept, possessed, .. } => {
            assert_eq!(verb_concept, "BE");
            assert_eq!(possessed.concept.0, "YEAR");
        }
        other => panic!("expected Possession frame, got {:?}", other),
    }

    assert_graph_il_frame_sync(sentence);

    let graph = sentence.graph.as_ref().unwrap();
    let verb = graph.find_verbs().into_iter().next().expect("verb word");
    assert_eq!(
        verb.evokes.as_ref().map(|c| c.0.as_str()),
        Some("BE"),
        "verb word evokes must be BE after age idiom, not stale HAVE"
    );
}

#[test]
fn test_graph_frame_matches_il_existence_and_possession() {
    let api = build_api();
    for (input, lang) in [
        ("Tomek mieszka z żoną.", "pl"),
        ("Tomek ma kota.", "pl"),
        ("I live with a wife.", "en"),
        ("Tom has a cat.", "en"),
    ] {
        let il = api.parse(input, lang).unwrap();
        let sentence = &il.as_natural().unwrap().sentences[0];
        assert_graph_il_frame_sync(sentence);
    }
}

#[test]
fn test_graph_instrumental_drives_with_prep_decision() {
    let graph = graph_from_parse("Tomek mieszka z żoną.", "pl");
    let loc_entity = graph
        .nodes
        .iter()
        .filter_map(|n| match n {
            lexflex::core::graph::GraphNode::Entity(e) if e.concept.0 == "WIFE" => Some(e.id),
            _ => None,
        })
        .next()
        .expect("WIFE entity");
    assert!(graph.location_uses_instrumental(loc_entity));
}