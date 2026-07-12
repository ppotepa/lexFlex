use crate::core::graph::{DialogueGraph, EdgeKind, GraphNode};
use crate::core::interlingua::*;

/// Split text on sentence boundaries (. ? !).
pub fn split_sentence_boundaries(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if ch == '.' || ch == '?' || ch == '!' {
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        }
    }
    let tail = current.trim().to_string();
    if !tail.is_empty() {
        sentences.push(tail);
    }
    if sentences.is_empty() && !text.trim().is_empty() {
        sentences.push(text.trim().to_string());
    }
    sentences
}

type EntitySnap = (NodeId, Option<String>, ConceptId);

fn collect_entities(graph: &crate::core::graph::LinguisticGraph) -> Vec<EntitySnap> {
    graph
        .nodes
        .iter()
        .filter_map(|n| match n {
            GraphNode::Entity(e) => Some((e.id, e.name.clone(), e.concept.clone())),
            _ => None,
        })
        .collect()
}

/// Track discourse context within a multi-sentence utterance.
pub fn track_discourse(utterance: &mut Utterance) {
    let mut discourse = utterance.discourse.clone().unwrap_or_default();
    let mut recent: Vec<(NodeId, usize)> = Vec::new();
    let mut focus: Vec<NodeId> = Vec::new();
    let mut coref_edges: Vec<EdgeId> = Vec::new();

    let n = utterance.sentences.len();
    let mut entity_snaps: Vec<Vec<EntitySnap>> = Vec::with_capacity(n);
    let mut frame_ids: Vec<Option<NodeId>> = Vec::with_capacity(n);

    for sentence in &utterance.sentences {
        if let Some(ref g) = sentence.graph {
            entity_snaps.push(collect_entities(g));
            frame_ids.push(g.frame_ids().into_iter().next());
        } else {
            entity_snaps.push(vec![]);
            frame_ids.push(None);
        }
    }

    let mut next_sentence_edges: Vec<(usize, NodeId, NodeId)> = Vec::new();
    let mut coref_pairs: Vec<(usize, NodeId, NodeId)> = Vec::new();
    let mut focus_edges: Vec<(usize, NodeId)> = Vec::new();
    let mut recent_edges: Vec<(usize, NodeId)> = Vec::new();

    let mut prev_frame: Option<NodeId> = None;
    for si in 0..n {
        if let (Some(prev), Some(cur)) = (prev_frame, frame_ids[si]) {
            next_sentence_edges.push((si, prev, cur));
        }
        prev_frame = frame_ids[si];

        for (i, &(eid, ref name, ref concept)) in entity_snaps[si].iter().enumerate() {
            recent.push((eid, si * 100 + i));
            recent_edges.push((si, eid));
            if i == 0 {
                focus.push(eid);
                focus_edges.push((si, eid));
            }

            if let Some(n) = name {
                for prev_si in 0..si {
                    for &(peid, ref pname, ref pconcept) in &entity_snaps[prev_si] {
                        if pname.as_deref() == Some(n.as_str())
                            || (pconcept == concept && pname.is_some())
                        {
                            coref_pairs.push((si, peid, eid));
                        }
                    }
                }
            }
        }
    }

    for (si, prev, cur) in next_sentence_edges {
        if let Some(g) = utterance.sentences[si].graph.as_mut() {
            let eid = g.add_edge(prev, cur, EdgeKind::NextSentence);
            coref_edges.push(eid);
        }
    }
    for (si, peid, eid) in coref_pairs {
        if let Some(g) = utterance.sentences[si].graph.as_mut() {
            let e1 = g.add_edge(peid, eid, EdgeKind::Corefers);
            let e2 = g.add_edge(eid, peid, EdgeKind::Corefers);
            coref_edges.push(e1);
            coref_edges.push(e2);
        }
    }
    for (si, eid) in focus_edges {
        if let Some(g) = utterance.sentences[si].graph.as_mut() {
            g.add_edge(eid, eid, EdgeKind::InFocus);
        }
    }
    for (si, eid) in recent_edges {
        if let Some(g) = utterance.sentences[si].graph.as_mut() {
            g.add_edge(eid, eid, EdgeKind::RecentMention);
        }
    }

    discourse.entities_in_focus = focus;
    discourse.recent_mentions = recent;
    discourse.coref_edges = coref_edges;
    utterance.discourse = Some(discourse);
}

/// Link entity references across utterances in a dialogue.
pub fn link_cross_utterance_context(dialogue: &mut DialogueGraph) {
    let mut prev_entities: Vec<EntitySnap> = Vec::new();
    for ui in 0..dialogue.utterances.len() {
        let mut pairs: Vec<(NodeId, NodeId)> = Vec::new();
        for sentence in &dialogue.utterances[ui].sentences {
            if let Some(ref g) = sentence.graph {
                for (eid, name, concept) in collect_entities(g) {
                    for (peid, pname, pconcept) in &prev_entities {
                        if name == *pname
                            || (concept == *pconcept && name.is_some() && pname.is_some())
                        {
                            pairs.push((*peid, eid));
                        }
                    }
                }
            }
        }
        for sentence in &mut dialogue.utterances[ui].sentences {
            if let Some(g) = sentence.graph.as_mut() {
                for (peid, eid) in &pairs {
                    let edge_id = g.add_edge(*peid, *eid, EdgeKind::Corefers);
                    dialogue.cross_edges.push(crate::core::graph::Edge {
                        id: edge_id,
                        from: *peid,
                        to: *eid,
                        kind: EdgeKind::Corefers,
                    });
                    let topic_id = g.add_edge(*peid, *eid, EdgeKind::ContinuesTopic);
                    dialogue.cross_edges.push(crate::core::graph::Edge {
                        id: topic_id,
                        from: *peid,
                        to: *eid,
                        kind: EdgeKind::ContinuesTopic,
                    });
                }
            }
        }
        for sentence in &dialogue.utterances[ui].sentences {
            if let Some(ref g) = sentence.graph {
                prev_entities.extend(collect_entities(g));
            }
        }
    }
}

/// Query recent entity mentions within an utterance graph layer.
pub fn recent_entities_of_type(utterance: &Utterance, concept: &str, window: usize) -> Vec<NodeId> {
    let c = concept.to_uppercase();
    utterance
        .discourse
        .as_ref()
        .map(|d| {
            d.recent_mentions
                .iter()
                .rev()
                .take(window)
                .filter_map(|(nid, _)| {
                    utterance.sentences.iter().find_map(|s| {
                        s.graph.as_ref().and_then(|g| {
                            g.nodes.iter().find_map(|n| match n {
                                GraphNode::Entity(e)
                                    if e.id == *nid && e.concept.0 == c =>
                                {
                                    Some(e.id)
                                }
                                _ => None,
                            })
                        })
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_boundaries_basic() {
        let parts = split_sentence_boundaries("Tomek ma kota. On go kocha.");
        assert_eq!(parts.len(), 2);
    }
}