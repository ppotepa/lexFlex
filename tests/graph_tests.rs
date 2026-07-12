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

// ─── Phrase layer (Phase 2 completion) ───────────────────────────────────────

#[test]
fn test_phrase_nodes_materialized() {
    let graph = graph_from_parse("Tomek mieszka z żoną.", "pl");
    let phrases: Vec<_> = graph.phrase_nodes().collect();
    assert!(!phrases.is_empty(), "NP/PP/VP phrase nodes expected");
    assert!(phrases.iter().any(|p| p.phrase_type == "VP"));
    assert!(phrases.iter().any(|p| p.phrase_type == "PP" || p.phrase_type == "NP"));
}

// ─── Concept layer (Phase 4) ─────────────────────────────────────────────────

#[test]
fn test_concept_nodes_and_evokes_edges() {
    let graph = graph_from_parse("Tomek ma kota.", "pl");
    assert!(!graph.concept_nodes().collect::<Vec<_>>().is_empty());
    assert!(graph.has_edge_kind(&EdgeKind::EvokesConcept));
    let verbs = graph.find_verbs();
    assert!(verbs.iter().any(|v| v.evokes.is_some()));
}

// ─── Path / construction matching (Phase 5) ───────────────────────────────────

#[test]
fn test_accompaniment_construction_paths() {
    let graph = graph_from_parse("Tomek mieszka z żoną i córką.", "pl");
    let paths = graph.find_accompaniment_paths();
    assert!(!paths.is_empty(), "verb→prep→noun accompaniment path expected");
    assert!(paths[0].len() >= 3);
    assert!(graph.edges.iter().any(|e| matches!(
        e.kind,
        EdgeKind::PartOfConstruction(ref c) if c == "Accompaniment"
    )));
}

// ─── Discourse / context (Phase 3) ───────────────────────────────────────────

#[test]
fn test_multi_sentence_corefers_and_next_sentence() {
    let api = build_api();
    let il = api
        .parse("Tomek ma kota. Tomek ma psa.", "pl")
        .expect("multi-sentence parse");
    let utt = il.as_natural().expect("natural");
    assert_eq!(utt.sentences.len(), 2);

    let d = utt.discourse.as_ref().expect("discourse tracked");
    assert!(!d.recent_mentions.is_empty());
    assert!(!d.entities_in_focus.is_empty());

    let g1 = utt.sentences[0].graph.as_ref().expect("graph s1");
    let g2 = utt.sentences[1].graph.as_ref().expect("graph s2");
    assert!(g1.edges.iter().any(|e| matches!(e.kind, EdgeKind::Corefers))
        || g2.edges.iter().any(|e| matches!(e.kind, EdgeKind::Corefers)));
    assert!(g2.edges.iter().any(|e| matches!(e.kind, EdgeKind::NextSentence))
        || g1.frame_ids().len() > 0);
}

#[test]
fn test_recent_entities_query() {
    use lexflex::core::context::recent_entities_of_type;
    let api = build_api();
    let il = api.parse("Tomek ma kota. Tomek ma psa.", "pl").unwrap();
    let utt = il.as_natural().unwrap();
    let recent = recent_entities_of_type(utt, "PERSON", 3);
    assert!(!recent.is_empty());
}

// ─── Dialogue graph (Phase 7) ──────────────────────────────────────────────────

#[test]
fn test_dialogue_graph_cross_utterance() {
    let api = build_api();
    let dialogue = api
        .parse_dialogue(&["Tomek ma kota.", "Tomek ma psa."], "pl")
        .expect("dialogue parse");
    assert_eq!(dialogue.utterances.len(), 2);
    assert!(!dialogue.cross_edges.is_empty() || dialogue.utterances[1]
        .sentences
        .first()
        .and_then(|s| s.graph.as_ref())
        .map(|g| g.edges.iter().any(|e| matches!(e.kind, EdgeKind::Corefers)))
        .unwrap_or(false));
}

#[test]
fn test_graph_coordination_drives_plural_agreement() {
    use lexflex::core::graph;
    let graph = graph_from_parse("Tomek i Iza ma kota.", "pl");
    let api = build_api();
    let il = api.parse("Tomek i Iza ma kota.", "pl").unwrap();
    let sentence = &il.as_natural().unwrap().sentences[0];
    let agent = &sentence.frames[0].entities()[0];
    assert!(graph::has_coordination_topology(&graph, agent));
    assert!(graph::entity_needs_plural_agreement(agent, sentence.graph.as_ref()));
}

#[test]
fn test_age_idiom_uses_year_concept_not_lemma_hacks() {
    let api = build_api();
    let il = api.parse("Mam 27 lat.", "pl").unwrap();
    let sentence = &il.as_natural().unwrap().sentences[0];
    match &sentence.frames[0] {
        Frame::Possession { verb_concept, possessed, .. } => {
            assert_eq!(verb_concept, "BE");
            assert_eq!(possessed.concept.0, "YEAR");
        }
        other => panic!("expected Possession: {:?}", other),
    }
    assert_graph_il_frame_sync(sentence);
    let out = api.translate("Mam 27 lat.", "pl", "en").unwrap();
    assert_eq!(out, "I am 27 years old.");
}

#[test]
fn test_construction_registry_find_construction() {
    let graph = graph_from_parse("Tomek mieszka z żoną i córką.", "pl");
    assert!(!graph.constructions.is_empty());
    let verbs: Vec<_> = graph.find_verbs().into_iter().map(|v| v.id).collect();
    let found = verbs.iter().find_map(|&vid| graph.find_construction(vid, "Accompaniment"));
    assert!(found.is_some(), "Accompaniment construction should be discoverable");
}

#[test]
fn test_word_navig_next_and_coreference_chain() {
    let api = build_api();
    let il = api.parse("Tomek ma kota. Tomek ma psa.", "pl").unwrap();
    let utt = il.as_natural().unwrap();
    let g = utt.sentences[1].graph.as_ref().unwrap();
    let words: Vec<_> = g.word_nodes().collect();
    assert!(words[0].navig_next(g).is_some());
    let entities: Vec<_> = g.entity_ids();
    if entities.len() >= 2 {
        let chain = g.coreference_chain(entities[0]);
        assert!(!chain.is_empty());
    }
}

#[test]
fn test_in_focus_and_recent_mention_queries() {
    let api = build_api();
    let il = api.parse("Tomek ma kota.", "pl").unwrap();
    let g = il.as_natural().unwrap().sentences[0].graph.as_ref().unwrap();
    assert!(!g.in_focus_entities().is_empty() || !g.recent_mention_entities().is_empty());
}

#[test]
fn test_graph_inference_sets_location_role() {
    let graph = graph_from_parse("Tomek mieszka z żoną.", "pl");
    let api = build_api();
    let il = api.parse("Tomek mieszka z żoną.", "pl").unwrap();
    let sentence = &il.as_natural().unwrap().sentences[0];
    if let Frame::Existence { location: Some(loc), .. } = &sentence.frames[0] {
        assert_eq!(loc.features.case, Some(Case::Instrumental));
        assert_eq!(loc.features.semantic_role, Some(SemanticRole::Location));
    }
    let paths = graph.find_paths().starting_with_verb().then_preposition(&["z"]).then_noun_phrase().paths();
    assert!(!paths.is_empty());
}

#[test]
fn test_dialogue_continues_topic_edges() {
    let api = build_api();
    let dialogue = api
        .parse_dialogue(&["Tomek ma kota.", "Tomek ma psa."], "pl")
        .unwrap();
    assert!(dialogue.cross_edges.iter().any(|e| e.kind == EdgeKind::ContinuesTopic)
        || dialogue.cross_edges.iter().any(|e| e.kind == EdgeKind::Corefers));
}

#[test]
fn test_translate_dialogue() {
    let api = build_api();
    let outs = api
        .translate_dialogue(&["Tomek ma kota.", "Tomek ma psa."], "pl", "en")
        .expect("dialogue translate");
    assert_eq!(outs.len(), 2);
    assert!(!outs[0].is_empty());
    assert!(!outs[1].is_empty());
}