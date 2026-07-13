//! IL-derived semantic digest for chat, CLI, and tracing.

use crate::core::graph::{self, GraphNode, LinguisticGraph};
use crate::core::interlingua::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticDigest {
    pub clauses: Vec<ClauseDigest>,
    pub discourse: DiscourseDigest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClauseDigest {
    pub index: usize,
    pub frame_type: String,
    pub verb_concept: String,
    pub roles: Vec<RoleBinding>,
    pub constructions: Vec<String>,
    pub tense: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleBinding {
    pub role: String,
    pub concept: String,
    pub name: Option<String>,
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscourseDigest {
    pub current_topic: Option<EntityRef>,
    pub in_focus: Vec<EntityRef>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityRef {
    pub concept: String,
    pub name: Option<String>,
}

/// Build a deterministic digest from a fully parsed utterance (post-discourse).
pub fn summarize_utterance(utterance: &Utterance) -> SemanticDigest {
    let clauses = utterance
        .sentences
        .iter()
        .enumerate()
        .map(|(i, s)| summarize_sentence(i, s))
        .collect();
    let discourse = summarize_discourse(utterance);
    SemanticDigest { clauses, discourse }
}

fn summarize_sentence(index: usize, sentence: &Sentence) -> ClauseDigest {
    let graph = sentence.graph.as_ref();
    let frame = sentence.frames.first().cloned().unwrap_or(Frame::Statement {
        subject: Entity::new(ConceptId::new("unknown")),
        property: Entity::new(ConceptId::new("unknown")),
        verb_concept: "BE".into(),
    });

    let mut constructions: Vec<String> = sentence
        .constructions
        .iter()
        .map(|c| c.construction_concept.0.clone())
        .collect();
    if constructions.is_empty() {
        constructions = sentence
            .construction_concepts
            .iter()
            .map(|c| c.0.clone())
            .collect();
    }
    constructions.sort();
    constructions.dedup();

    ClauseDigest {
        index,
        frame_type: frame.frame_type_name().to_string(),
        verb_concept: verb_concept_of(&frame),
        roles: role_bindings_from_frame(&frame, graph),
        constructions,
        tense: sentence.tense.map(|t| format!("{:?}", t)),
    }
}

fn verb_concept_of(frame: &Frame) -> String {
    match frame {
        Frame::Transfer { verb_concept, .. } => verb_concept.clone(),
        Frame::Motion { verb_concept, .. } => verb_concept.clone(),
        Frame::Creation { verb_concept, .. } => verb_concept.clone(),
        Frame::Destruction { verb_concept, .. } => verb_concept.clone(),
        Frame::Perception { verb_concept, .. } => verb_concept.clone(),
        Frame::Cognition { verb_concept, .. } => verb_concept.clone(),
        Frame::Emotion { verb_concept, .. } => verb_concept.clone(),
        Frame::Communication { verb_concept, .. } => verb_concept.clone(),
        Frame::Statement { verb_concept, .. } => verb_concept.clone(),
        Frame::Existence { verb_concept, .. } => verb_concept.clone(),
        Frame::Possession { verb_concept, .. } => verb_concept.clone(),
        Frame::Consumption { verb_concept, .. } => verb_concept.clone(),
        Frame::Custom { name, .. } => name.clone(),
    }
}

fn display_entity_name(entity: &Entity, graph: Option<&LinguisticGraph>) -> Option<String> {
    if entity.concept.0 == "DUMMY_SUBJECT" {
        return None;
    }
    if entity.concept.0 == "PERSON" {
        return entity.name.clone();
    }
    if entity
        .name
        .as_ref()
        .map_or(false, |n| n.chars().next().map_or(false, |c| c.is_uppercase()))
    {
        return entity.name.clone();
    }
    if let Some(g) = graph {
        if let Some(eid) = graph::find_entity_node_id(g, entity) {
            let words = g.realizing_words_for_entity(eid);
            if let Some(w) = words.first() {
                return Some(w.lemma.clone());
            }
        }
    }
    entity.name.clone()
}

fn role_bindings_from_frame(frame: &Frame, graph: Option<&LinguisticGraph>) -> Vec<RoleBinding> {
    let pairs: Vec<(SemanticRole, &Entity)> = match frame {
        Frame::Transfer {
            agent,
            recipient,
            theme,
            ..
        } => vec![
            (SemanticRole::Agent, agent),
            (SemanticRole::Recipient, recipient),
            (SemanticRole::Theme, theme),
        ],
        Frame::Motion {
            mover,
            source,
            goal,
            path,
            ..
        } => {
            let mut v = vec![(SemanticRole::Agent, mover)];
            if let Some(s) = source {
                v.push((SemanticRole::Source, s));
            }
            if let Some(g) = goal {
                v.push((SemanticRole::Goal, g));
            }
            if let Some(p) = path {
                v.push((SemanticRole::Location, p));
            }
            v
        }
        Frame::Creation {
            creator,
            created,
            material,
            ..
        } => {
            let mut v = vec![
                (SemanticRole::Creator, creator),
                (SemanticRole::Created, created),
            ];
            if let Some(m) = material {
                v.push((SemanticRole::Theme, m));
            }
            v
        }
        Frame::Destruction {
            agent,
            patient,
            instrument,
            ..
        } => {
            let mut v = vec![
                (SemanticRole::Agent, agent),
                (SemanticRole::Patient, patient),
            ];
            if let Some(i) = instrument {
                v.push((SemanticRole::Instrument, i));
            }
            v
        }
        Frame::Perception {
            experiencer,
            stimulus,
            ..
        } => vec![
            (SemanticRole::Experiencer, experiencer),
            (SemanticRole::Stimulus, stimulus),
        ],
        Frame::Cognition {
            cognizer,
            content,
            ..
        } => vec![
            (SemanticRole::Cognizer, cognizer),
            (SemanticRole::Theme, content),
        ],
        Frame::Emotion {
            experiencer,
            stimulus,
            ..
        } => vec![
            (SemanticRole::Experiencer, experiencer),
            (SemanticRole::Stimulus, stimulus),
        ],
        Frame::Communication {
            speaker,
            addressee,
            message,
            ..
        } => {
            let mut v = vec![
                (SemanticRole::Speaker, speaker),
                (SemanticRole::Theme, message),
            ];
            if let Some(a) = addressee {
                v.push((SemanticRole::Recipient, a));
            }
            v
        }
        Frame::Statement {
            subject,
            property,
            ..
        } => vec![
            (SemanticRole::Topic, subject),
            (SemanticRole::Theme, property),
        ],
        Frame::Existence { entity, location, .. } => {
            let mut v = vec![(SemanticRole::Theme, entity)];
            if let Some(l) = location {
                v.push((SemanticRole::Location, l));
            }
            v
        }
        Frame::Possession {
            possessor,
            possessed,
            ..
        } => vec![
            (SemanticRole::Agent, possessor),
            (SemanticRole::Theme, possessed),
        ],
        Frame::Consumption { agent, patient, .. } => vec![
            (SemanticRole::Agent, agent),
            (SemanticRole::Patient, patient),
        ],
        Frame::Custom { roles, .. } => roles.iter().map(|(r, e)| (*r, e)).collect(),
    };

    pairs
        .into_iter()
        .filter(|(_, e)| e.concept.0 != "unknown" && e.concept.0 != "DUMMY_SUBJECT")
        .map(|(role, entity)| RoleBinding {
            role: format!("{:?}", role),
            concept: entity.concept.0.clone(),
            name: display_entity_name(entity, graph),
            reference: reference_label(&entity.reference),
        })
        .collect()
}

fn reference_label(reference: &Reference) -> String {
    match reference {
        Reference::Direct => "Direct".into(),
        Reference::Anaphoric(antecedent) => format!("Anaphoric({antecedent})"),
        Reference::Cataphoric(antecedent) => format!("Cataphoric({antecedent})"),
        Reference::Deictic => "Deictic".into(),
        Reference::Generic => "Generic".into(),
        Reference::Unresolved => "Unresolved".into(),
    }
}

fn summarize_discourse(utterance: &Utterance) -> DiscourseDigest {
    let discourse = utterance.discourse.as_ref();
    let graph = utterance
        .sentences
        .iter()
        .find_map(|s| s.graph.as_ref());

    let current_topic = discourse
        .and_then(|d| d.current_topic)
        .and_then(|nid| graph.and_then(|g| entity_ref_from_node(g, nid)));

    let mut in_focus = Vec::new();
    if let Some(d) = discourse {
        if let Some(g) = graph {
            for nid in &d.entities_in_focus {
                if let Some(er) = entity_ref_from_node(g, *nid) {
                    push_unique_entity_ref(&mut in_focus, er);
                }
            }
        }
    }
    if in_focus.is_empty() {
        if let Some(g) = graph {
            for nid in g.in_focus_entities() {
                if let Some(er) = entity_ref_from_node(g, nid) {
                    push_unique_entity_ref(&mut in_focus, er);
                }
            }
        }
    }
    if in_focus.is_empty() {
        for sentence in &utterance.sentences {
            if let Some(agent) = sentence.frames.first().and_then(|f| f.agent_entity()) {
                push_unique_entity_ref(
                    &mut in_focus,
                    EntityRef {
                        concept: agent.concept.0.clone(),
                        name: agent.name.clone(),
                    },
                );
            }
        }
    }

    DiscourseDigest {
        current_topic,
        in_focus,
    }
}

fn push_unique_entity_ref(list: &mut Vec<EntityRef>, er: EntityRef) {
    if !list
        .iter()
        .any(|x| x.name == er.name && x.concept == er.concept)
    {
        list.push(er);
    }
}

fn entity_ref_from_node(graph: &LinguisticGraph, nid: NodeId) -> Option<EntityRef> {
    graph.nodes.get(nid.0 as usize).and_then(|n| match n {
        GraphNode::Entity(e) => {
            let entity = Entity {
                concept: e.concept.clone(),
                name: e.name.clone(),
                features: e.features.clone(),
                reference: Reference::Direct,
                id: None,
                coordination: None,
                adjectives: vec![],
            };
            Some(EntityRef {
                concept: e.concept.0.clone(),
                name: display_entity_name(&entity, Some(graph)),
            })
        }
        _ => None,
    })
}

/// Human-readable multi-line summary for chat/CLI.
pub fn format_digest_human(digest: &SemanticDigest) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Semantic digest ({} clause{})\n",
        digest.clauses.len(),
        if digest.clauses.len() == 1 { "" } else { "s" }
    ));
    for clause in &digest.clauses {
        out.push_str(&format!(
            "\nClause {} — {} [{}]\n",
            clause.index + 1,
            clause.frame_type,
            clause.verb_concept
        ));
        for role in &clause.roles {
            let name = role.name.as_deref().unwrap_or("—");
            let label = role_display_label(&role.role);
            out.push_str(&format!(
                "  {}: {} ({}) [{}]\n",
                label, name, role.reference, role.concept
            ));
        }
        if !clause.constructions.is_empty() {
            out.push_str(&format!(
                "  Constructions: {}\n",
                clause.constructions.join(", ")
            ));
        }
        if let Some(ref tense) = clause.tense {
            out.push_str(&format!("  Tense: {tense}\n"));
        }
    }
    out.push_str("\nDiscourse\n");
    if let Some(ref topic) = digest.discourse.current_topic {
        let name = topic.name.as_deref().unwrap_or("—");
        out.push_str(&format!("  Topic: {name} [{}]\n", topic.concept));
    } else {
        out.push_str("  Topic: —\n");
    }
    if digest.discourse.in_focus.is_empty() {
        out.push_str("  In focus: —\n");
    } else {
        let names: Vec<String> = digest
            .discourse
            .in_focus
            .iter()
            .map(|e| e.name.clone().unwrap_or_else(|| e.concept.clone()))
            .collect();
        out.push_str(&format!("  In focus: {}\n", names.join(", ")));
    }
    out
}

/// JSON serialization of digest.
pub fn format_digest_json(digest: &SemanticDigest) -> String {
    serde_json::to_string_pretty(digest).unwrap_or_else(|_| "{}".into())
}

fn role_display_label(role: &str) -> &str {
    match role {
        "Agent" | "Cognizer" | "Creator" | "Speaker" | "Experiencer" => "Actor",
        _ => role,
    }
}

/// Emit structured digest for RUST_LOG tracing consumers (`RUST_LOG=lexflex=trace`).
pub fn trace_digest(stage: &str, utterance: &Utterance) {
    let digest = summarize_utterance(utterance);
    let json = format_digest_json(&digest);
    tracing::debug!(
        stage = stage,
        clause_count = digest.clauses.len(),
        digest_json = %json,
        "lexflex_semantic_digest"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_label_formats_anaphoric() {
        assert_eq!(
            reference_label(&Reference::Anaphoric("Tomek".into())),
            "Anaphoric(Tomek)"
        );
    }
}